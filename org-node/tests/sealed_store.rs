#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The sealed per-Persona store (SDD-asxwp8; S4, task T3 of
//! docs/plans/2026-10-06-org-io-transport.md): one store per Persona, sealed
//! under a key derived from that Persona's device secret, given out as
//! `StoreWrite` bytes and taken back in by `PersonaStore::unseal`. org-node
//! touches no file: these tests play org-io, holding each returned write as
//! "the disk" and each secret's bytes as "the keychain".

use std::path::Path;

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use org_members::{DevicePublicKey, Handle, MemberId, Name, Surname};
use org_node::keys::DeviceSecret;
use org_node::reconcile::Reconciled;
use org_node::service::{OrgService, StoreChange};
use org_node::store::{PersonaStore, SealedStore, StoreData, StoreRefusal, StoreWrite};
use org_node::test_fixtures::{self, ChainSlots};
use org_node::{ChainAccount, Joiner, OrgNodeError, PersonaId, ProvisionalTarget};
use rand::rngs::OsRng;
use sha2::Sha256;
use zeroize::Zeroizing;

fn h(text: &str) -> Handle {
    Handle::parse(text).unwrap()
}
fn nm(text: &str) -> Name {
    Name::parse(text).unwrap()
}
fn sn(text: &str) -> Surname {
    Surname::parse(text).unwrap()
}

/// The seed bytes of `secret`, as org-io's keychain would hold them.
fn keychain_bytes(secret: &DeviceSecret) -> Vec<u8> {
    let mut kept = vec![];
    secret
        .lend_to_keychain(|bytes| {
            kept.extend_from_slice(bytes);
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap();
    kept
}

/// A `DeviceSecret` re-made from keychain bytes: what org-io reads back.
fn from_keychain(bytes: &[u8]) -> DeviceSecret {
    DeviceSecret::from_keychain_bytes(Zeroizing::new(bytes.to_vec())).unwrap()
}

/// A new Persona on `svc`: its id, its seed bytes (the test's keychain) and
/// the write that created its store.
fn new_persona(svc: &mut OrgService, handle: &str) -> (PersonaId, Vec<u8>, StoreWrite) {
    let (persona_id, secret, write) = svc.create_persona(&mut OsRng, h(handle), nm("Test"), sn("User")).unwrap();
    (persona_id, keychain_bytes(&secret), write)
}

/// The plaintext a write sealed: the store it unseals to, re-encoded as the
/// store encodes it.
fn plaintext_of(write: &StoreWrite, seed: &[u8]) -> Vec<u8> {
    let store = PersonaStore::unseal(&write.sealed, write.device_key, from_keychain(seed)).unwrap();
    postcard::to_allocvec(store.data()).unwrap()
}

fn nonce_of(write: &StoreWrite) -> Vec<u8> {
    write.sealed.as_bytes()[..24].to_vec()
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| window == needle)
}

fn joiner(seed: u8, handle: &str) -> Joiner {
    Joiner {
        handle: h(handle),
        name: nm("Join"),
        surname: sn("Er"),
        member_key: test_fixtures::member_key(seed),
        device_key: test_fixtures::device_key(seed + 1),
    }
}

/// `pid` founds an Organisation on `svc` over `chain`; returns it and the
/// two writes (the genesis update kept, the commit).
fn found(svc: &mut OrgService, chain: &ChainSlots, pid: &PersonaId) -> (org_node::OrgId, Vec<StoreWrite>) {
    let (update, kept) = svc.create_organisation(&mut OsRng, pid).unwrap();
    let org_id = chain.apply_genesis(update.resulting_root, update.org_pub_key);
    let (_, committed) =
        svc.commit_genesis(&mut OsRng, pid, org_id, ChainAccount::new([0x5a; 32]), chain.get(&org_id)).unwrap();
    (org_id, vec![kept, committed])
}

/// The MemberId under which `device` is listed in `svc`'s record of `org_id`.
fn member_of(svc: &OrgService, org_id: org_node::OrgId, device: DevicePublicKey) -> MemberId {
    let record = svc.list_orgs().into_iter().find(|record| record.org_id == org_id).unwrap();
    record.trie_members.iter().find(|member| member.device_keys.contains(&device)).unwrap().id
}

fn no_secrets(_: &[DevicePublicKey]) -> Vec<DeviceSecret> {
    panic!("a commit that keeps this node listed must not ask for a device secret")
}

// ---- LLR-bg538c: unseal ------------------------------------------------------

// verifies: LLR-bg538c
#[test]
fn a_sealed_store_unseals_with_its_secret_and_device_key() {
    let mut svc = OrgService::new(vec![]).unwrap();
    let (_, seed, write) = new_persona(&mut svc, "alice");
    let store = PersonaStore::unseal(&write.sealed, write.device_key, from_keychain(&seed)).unwrap();
    assert_eq!(store.device_key(), write.device_key);
    assert_eq!(store.data().persona.handle.as_str(), "alice");
    assert_eq!(store.data().persona.device_key, write.device_key);
}

// verifies: LLR-bg538c
#[test]
fn unseal_refuses_each_malformed_input_with_its_own_error() {
    let mut svc = OrgService::new(vec![]).unwrap();
    let (_, seed, write) = new_persona(&mut svc, "alice");
    let device_key = write.device_key;

    // Below nonce(24) + tag(16): too short, whatever the bytes.
    for length in [0usize, 23, 39] {
        let short = SealedStore::from_bytes(vec![0xab; length]);
        assert!(
            matches!(
                PersonaStore::unseal(&short, device_key, from_keychain(&seed)),
                Err(OrgNodeError::StoreRefused(StoreRefusal::TooShort))
            ),
            "{length} bytes"
        );
    }

    // One ciphertext bit flipped: not authentic.
    let mut flipped = write.sealed.as_bytes().to_vec();
    flipped[30] ^= 0x01;
    assert!(matches!(
        PersonaStore::unseal(&SealedStore::from_bytes(flipped), device_key, from_keychain(&seed)),
        Err(OrgNodeError::StoreRefused(StoreRefusal::NotAuthentic))
    ));

    // Another Persona's secret: a mismatch, found before any decryption
    // (the bytes would not even pass the length check).
    let (_, other_seed, _) = new_persona(&mut svc, "bob");
    assert!(matches!(
        PersonaStore::unseal(&write.sealed, device_key, from_keychain(&other_seed)),
        Err(OrgNodeError::DeviceSecretMismatch)
    ));
    assert!(matches!(
        PersonaStore::unseal(&SealedStore::from_bytes(vec![]), device_key, from_keychain(&other_seed)),
        Err(OrgNodeError::DeviceSecretMismatch)
    ));

    // Sealed under the right key, but bytes that do not decode.
    let undecodable = test_fixtures::seal_plaintext_for_test(&from_keychain(&seed), device_key, &[0xff; 7], &mut OsRng);
    assert!(matches!(
        PersonaStore::unseal(&undecodable, device_key, from_keychain(&seed)),
        Err(OrgNodeError::StoreRefused(StoreRefusal::Undecodable))
    ));

    // Sealed under the right key, decodes, but names another Device.
    let mut data: StoreData =
        PersonaStore::unseal(&write.sealed, device_key, from_keychain(&seed)).unwrap().data().clone();
    data.persona.device_key = test_fixtures::device_key(0x61);
    let foreign = test_fixtures::seal_plaintext_for_test(
        &from_keychain(&seed),
        device_key,
        &postcard::to_allocvec(&data).unwrap(),
        &mut OsRng,
    );
    assert!(matches!(
        PersonaStore::unseal(&foreign, device_key, from_keychain(&seed)),
        Err(OrgNodeError::DeviceSecretMismatch)
    ));
}

// ---- LLR-2qqqx6: the store key -----------------------------------------------

/// The key LLR-2qqqx6 states, computed here from first principles.
fn hkdf_key(seed: &[u8], device_key: &DevicePublicKey) -> [u8; 32] {
    let mut key = [0u8; 32];
    Hkdf::<Sha256>::new(Some(device_key.as_bytes()), seed).expand(b"ods/persona-store/v1", &mut key).unwrap();
    key
}

fn open_directly(sealed: &SealedStore, key: &[u8; 32]) -> Vec<u8> {
    let (nonce, ciphertext) = sealed.as_bytes().split_at(24);
    XChaCha20Poly1305::new(key.into()).decrypt(XNonce::from_slice(nonce), ciphertext).unwrap()
}

// verifies: LLR-2qqqx6
#[test]
fn the_store_key_is_hkdf_sha256_of_the_seed_under_the_versioned_label() {
    // A fixed seed, sealed by the fixture the store's own seal shares.
    let seed = [0x07u8; 32];
    let secret = from_keychain(&seed);
    let device_key = secret.device_key();
    let mut svc = OrgService::new(vec![]).unwrap();
    let (_, created_seed, created) = new_persona(&mut svc, "alice");
    let mut data: StoreData = PersonaStore::unseal(&created.sealed, created.device_key, from_keychain(&created_seed))
        .unwrap()
        .data()
        .clone();
    data.persona.device_key = device_key;
    let bytes = postcard::to_allocvec(&data).unwrap();
    let sealed = test_fixtures::seal_plaintext_for_test(&secret, device_key, &bytes, &mut OsRng);
    let key = hkdf_key(&seed, &device_key);
    assert_eq!(open_directly(&sealed, &key), bytes);
    let unsealed = PersonaStore::unseal(&sealed, device_key, from_keychain(&seed)).unwrap();
    assert_eq!(unsealed.device_key(), device_key);

    // The key is neither the seed nor the ed25519 secret scalar.
    assert_ne!(key, seed);
    assert_ne!(key, ed25519_dalek::SigningKey::from_bytes(&seed).to_scalar_bytes());
}

// verifies: LLR-2qqqx6
#[test]
fn a_created_store_opens_under_the_hkdf_key_of_its_secret() {
    let mut svc = OrgService::new(vec![]).unwrap();
    let (_, seed, write) = new_persona(&mut svc, "alice");
    let plaintext = open_directly(&write.sealed, &hkdf_key(&seed, &write.device_key));
    let data: StoreData = postcard::from_bytes(&plaintext).unwrap();
    assert_eq!(data.persona.handle.as_str(), "alice");
}

/// `plaintext` sealed directly under `key`, with a fresh nonce: a store as a
/// sealer using `key` would have written it.
fn seal_directly(plaintext: &[u8], key: &[u8; 32]) -> SealedStore {
    let nonce: [u8; 24] = rand::random();
    let ciphertext = XChaCha20Poly1305::new(key.into()).encrypt(XNonce::from_slice(&nonce), plaintext).unwrap();
    SealedStore::from_bytes([nonce.as_slice(), &ciphertext].concat())
}

// Abnormal (review round 1 of S4, finding-12): a store sealed under a key
// derived any other way than LLR-2qqqx6's — the seed itself, HKDF without
// the Device-key salt, another salt, another label, the ed25519 secret
// scalar — does not open (`NotAuthentic`); the same plaintext sealed under
// the stated key does, so the refusals are the key's alone.
// verifies: LLR-2qqqx6
#[test]
fn a_store_sealed_under_any_other_key_derivation_is_refused() {
    let mut svc = OrgService::new(vec![]).unwrap();
    let (_, seed, write) = new_persona(&mut svc, "alice");
    let device_key = write.device_key;
    let plaintext = open_directly(&write.sealed, &hkdf_key(&seed, &device_key));
    let expand = |salt: Option<&[u8]>, info: &[u8]| {
        let mut key = [0u8; 32];
        Hkdf::<Sha256>::new(salt, &seed).expand(info, &mut key).unwrap();
        key
    };
    let raw_seed: [u8; 32] = seed.as_slice().try_into().unwrap();
    let scalar = ed25519_dalek::SigningKey::from_bytes(&raw_seed).to_scalar_bytes();
    let other_salt = [0x5au8; 32];
    for (how, key) in [
        ("the seed itself", raw_seed),
        ("HKDF without the Device-key salt", expand(None, b"ods/persona-store/v1")),
        ("HKDF under another salt", expand(Some(&other_salt), b"ods/persona-store/v1")),
        ("HKDF under another label", expand(Some(device_key.as_bytes()), b"ods/persona-store/v2")),
        ("the ed25519 secret scalar", scalar),
    ] {
        let refused = PersonaStore::unseal(&seal_directly(&plaintext, &key), device_key, from_keychain(&seed)).err();
        assert_eq!(refused, Some(OrgNodeError::StoreRefused(StoreRefusal::NotAuthentic)), "{how}");
    }
    let stated = seal_directly(&plaintext, &hkdf_key(&seed, &device_key));
    assert!(PersonaStore::unseal(&stated, device_key, from_keychain(&seed)).is_ok(), "the stated key opens it");
}

fn org_node_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn rust_sources(dir: &Path, out: &mut Vec<(String, String)>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            out.push((path.display().to_string(), std::fs::read_to_string(&path).unwrap()));
        }
    }
}

// verifies: LLR-2qqqx6
#[test]
fn no_argon2_and_no_passphrase_remain() {
    let mut files = vec![];
    rust_sources(&org_node_dir().join("src"), &mut files);
    let manifest = org_node_dir().join("Cargo.toml");
    files.push((manifest.display().to_string(), std::fs::read_to_string(&manifest).unwrap()));
    for (file, text) in files {
        for needle in ["argon2", "Argon2", "passphrase", "APP_SALT"] {
            assert!(!text.contains(needle), "{file} names `{needle}`");
        }
    }
}

// ---- LLR-bvt5up: every change returns its write --------------------------------

// verifies: LLR-bvt5up
#[test]
fn every_mutating_operation_returns_one_store_write_with_a_fresh_nonce() {
    let chain = ChainSlots::new();
    let mut svc = OrgService::new(vec![]).unwrap();
    let (pid, seed, created) = new_persona(&mut svc, "alice");
    let own = created.device_key;
    let mut writes = vec![created];

    let (org_id, founded) = found(&mut svc, &chain, &pid);
    writes.extend(founded);

    // An admission built, discarded, built again and committed.
    let (update, built) = svc.admit_member(&mut OsRng, org_id, &joiner(0x21, "bob")).unwrap();
    writes.push(built);
    writes.push(
        svc.discard_provisional(&mut OsRng, ProvisionalTarget::Org(org_id), update.resulting_root, update.org_pub_key)
            .unwrap(),
    );
    let (update, built) = svc.admit_member(&mut OsRng, org_id, &joiner(0x21, "bob")).unwrap();
    writes.push(built);
    let epoch = svc.list_orgs()[0].epoch;
    chain.apply_update(org_id, update.resulting_root, update.org_pub_key, epoch).unwrap();
    let (_, change) = svc.commit_update(&mut OsRng, org_id, chain.get(&org_id), no_secrets).unwrap();
    let StoreChange::Sealed(committed) = change else { panic!("the commit keeps this node listed") };
    writes.push(committed);

    // A revocation of the joiner, built.
    let bob = member_of(&svc, org_id, test_fixtures::device_key(0x22));
    let (_, built) = svc.revoke_member(&mut OsRng, org_id, bob).unwrap();
    writes.push(built);

    // An expectation, then a refused operation, then the same expectation.
    let elsewhere = org_node::OrgId::new([0x77; 20]);
    let expected = svc.expect_admission(&mut OsRng, &pid, elsewhere).unwrap();
    let before_refusal = plaintext_of(&expected, &seed);
    writes.push(expected);
    assert!(matches!(
        svc.create_organisation(&mut OsRng, &pid),
        Err(OrgNodeError::PersonaAlreadyBound { .. })
    ));
    let again = svc.expect_admission(&mut OsRng, &pid, elsewhere).unwrap();
    assert_eq!(plaintext_of(&again, &seed), before_refusal, "the refusal changed nothing");
    writes.push(again);

    assert_eq!(writes.len(), 10);
    for write in &writes {
        assert_eq!(write.device_key, own, "each write is the changed Persona's");
    }
    for pair in writes.windows(2) {
        assert_ne!(nonce_of(&pair[0]), nonce_of(&pair[1]), "consecutive writes reuse no nonce");
    }
}

// verifies: LLR-bvt5up
#[test]
fn no_sealed_store_holds_the_device_seed() {
    let chain = ChainSlots::new();
    let mut svc = OrgService::new(vec![]).unwrap();
    let (pid, seed, created) = new_persona(&mut svc, "alice");
    let mut writes = vec![created];
    let (org_id, founded) = found(&mut svc, &chain, &pid);
    writes.extend(founded);
    let (_, built) = svc.admit_member(&mut OsRng, org_id, &joiner(0x21, "bob")).unwrap();
    writes.push(built);
    for write in &writes {
        let plaintext = plaintext_of(write, &seed);
        assert!(!contains(&plaintext, &seed), "the device seed is not in the store");
        assert!(!contains(write.sealed.as_bytes(), &seed), "nor in the sealed bytes");
    }
}

/// The text of every production `pub fn` in `file` (not gated on
/// `test-support`) whose signature names `needle`, joined across lines up to
/// its `{` or `;`.
fn public_signatures_naming(file: &str, needle: &str) -> Vec<String> {
    let text = std::fs::read_to_string(org_node_dir().join("src").join(file)).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    let mut found = vec![];
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !(trimmed.starts_with("pub fn ") || trimmed.starts_with("pub async fn ")) {
            continue;
        }
        let test_only = lines[..index]
            .iter()
            .rev()
            .take_while(|above| above.trim_start().starts_with("#[") || above.trim_start().starts_with("///"))
            .any(|above| above.contains("feature = \"test-support\""));
        if test_only {
            continue;
        }
        let mut signature = String::new();
        for continuation in &lines[index..] {
            let end = continuation.find(['{', ';']);
            signature.push_str(continuation[..end.unwrap_or(continuation.len())].trim());
            signature.push(' ');
            if end.is_some() {
                break;
            }
        }
        if signature.contains(needle) {
            found.push(signature);
        }
    }
    found
}

/// The parameter list of `signature`: between its first `(` and the `)`
/// that closes it.
fn parameters(signature: &str) -> &str {
    let open = signature.find('(').unwrap();
    let mut depth = 0;
    for (offset, character) in signature[open..].char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return &signature[open + 1..open + offset];
                }
            }
            _ => {}
        }
    }
    &signature[open + 1..]
}

fn function_name(signature: &str) -> &str {
    let after = &signature[signature.find("fn ").unwrap() + 3..];
    &after[..after.find(['(', '<']).unwrap()]
}

// verifies: LLR-bvt5up
#[test]
fn saving_needs_no_device_secret() {
    // The operations that may sign S3a's acknowledgements: their secrets
    // come from the caller's closure, on the signing path only. (*S4 T4,
    // 2026-10-13:* `commit_checked` replaced the transitional receive pair.)
    let signing = ["commit_update", "reconcile", "commit_checked"];
    let mut takers = vec![];
    for file in ["store.rs", "service.rs"] {
        for signature in public_signatures_naming(file, "DeviceSecret") {
            let name = function_name(&signature).to_string();
            let params = parameters(&signature);
            if !params.contains("DeviceSecret") {
                continue;
            }
            if signing.contains(&name.as_str()) {
                let closure_returns = params.matches("-> Vec<DeviceSecret>").count();
                assert_eq!(
                    params.matches("DeviceSecret").count(),
                    closure_returns,
                    "{file}: `{name}` names DeviceSecret outside its device_secrets closure: {signature}"
                );
                assert_eq!(closure_returns, 1, "{file}: `{name}` takes one device_secrets closure");
            }
            takers.push(name);
        }
    }
    takers.sort();
    // *2026-10-14 (S4, task T14):* the transitional `bind_endpoint` left the
    // list with the endpoint.
    let mut expected: Vec<String> =
        ["unseal", "commit_update", "reconcile", "commit_checked"]
            .iter()
            .map(|name| name.to_string())
            .collect();
    expected.sort();
    assert_eq!(takers, expected, "only these take a DeviceSecret; create_persona returns one and takes none");
}

// ---- S3a's signing, over DeviceSecret ------------------------------------------

// verifies: LLR-hby4jr, LLR-uw7nmv
#[test]
fn a_commit_that_removes_this_node_signs_with_the_supplied_secret_and_reads_none_otherwise() {
    let chain = ChainSlots::new();
    let mut svc = OrgService::new(vec![]).unwrap();
    let (pid, seed, created) = new_persona(&mut svc, "alice");
    let own = created.device_key;
    let (org_id, _) = found(&mut svc, &chain, &pid);

    // An admission keeps this node listed: the closure is never called.
    let (update, _) = svc.admit_member(&mut OsRng, org_id, &joiner(0x21, "bob")).unwrap();
    chain.apply_update(org_id, update.resulting_root, update.org_pub_key, svc.list_orgs()[0].epoch).unwrap();
    let (_, change) = svc.commit_update(&mut OsRng, org_id, chain.get(&org_id), no_secrets).unwrap();
    assert!(matches!(change, StoreChange::Sealed(_)));

    // Revoking this node's own Member removes it.
    let alice = member_of(&svc, org_id, own);
    let (update, _) = svc.revoke_member(&mut OsRng, org_id, alice).unwrap();
    chain.apply_update(org_id, update.resulting_root, update.org_pub_key, svc.list_orgs()[0].epoch).unwrap();

    // No secret supplied: refused, nothing changed, no store change.
    let mut asked = vec![];
    let refused = svc.commit_update(&mut OsRng, org_id, chain.get(&org_id), |keys: &[DevicePublicKey]| {
        asked.push(keys.to_vec());
        Vec::new()
    });
    assert!(matches!(refused, Err(OrgNodeError::DeviceSecretNotSupplied { .. })));
    assert_eq!(asked, vec![vec![own]], "asked once, for the bound Persona's Device");
    assert_eq!(svc.list_personas().len(), 1, "the refusal kept the Persona");
    assert_eq!(svc.list_orgs().len(), 1, "and the record");

    // The secret supplied: one acknowledgement, verifying under that key.
    let mut asked = vec![];
    let (outcome, change) = svc
        .commit_update(&mut OsRng, org_id, chain.get(&org_id), |keys: &[DevicePublicKey]| {
            asked.push(keys.to_vec());
            vec![from_keychain(&seed)]
        })
        .unwrap();
    assert_eq!(asked, vec![vec![own]]);
    assert_eq!(outcome.acknowledgements.len(), 1);
    for acknowledgement in &outcome.acknowledgements {
        assert_eq!(acknowledgement.device, own);
        let signature = ed25519_dalek::Signature::from_bytes(&acknowledgement.signature.0);
        own.verifying_key().verify_strict(&acknowledgement.signed_bytes(), &signature).unwrap();
    }
    assert!(matches!(change, StoreChange::Forgotten { device_key } if device_key == own));
}

// verifies: LLR-pba7yu
#[test]
fn a_removal_returns_store_change_forgotten_for_that_personas_device_key() {
    for by_reconcile in [false, true] {
        let chain = ChainSlots::new();
        let mut svc = OrgService::new(vec![]).unwrap();
        let (pid, seed, created) = new_persona(&mut svc, "alice");
        let own = created.device_key;
        let (other, _, _) = new_persona(&mut svc, "carol");
        let other_before = postcard::to_allocvec(svc.store_data(&other).unwrap()).unwrap();
        let (org_id, _) = found(&mut svc, &chain, &pid);
        let (update, _) = svc.admit_member(&mut OsRng, org_id, &joiner(0x21, "bob")).unwrap();
        chain.apply_update(org_id, update.resulting_root, update.org_pub_key, svc.list_orgs()[0].epoch).unwrap();
        let _kept = svc.commit_update(&mut OsRng, org_id, chain.get(&org_id), no_secrets).unwrap();
        let alice = member_of(&svc, org_id, own);
        let (update, _) = svc.revoke_member(&mut OsRng, org_id, alice).unwrap();
        chain.apply_update(org_id, update.resulting_root, update.org_pub_key, svc.list_orgs()[0].epoch).unwrap();

        let change = if by_reconcile {
            let (reconciled, change) = svc
                .reconcile(&mut OsRng, org_id, chain.get(&org_id), |_: &[DevicePublicKey]| vec![from_keychain(&seed)])
                .unwrap();
            assert!(matches!(reconciled, Reconciled::Removed { .. }));
            change.expect("a removal changes the store")
        } else {
            svc.commit_update(&mut OsRng, org_id, chain.get(&org_id), |_: &[DevicePublicKey]| vec![from_keychain(&seed)])
                .unwrap()
                .1
        };
        assert!(matches!(change, StoreChange::Forgotten { device_key } if device_key == own), "by_reconcile {by_reconcile}");
        assert!(svc.list_personas().iter().all(|persona| persona.persona_id != pid), "the Persona is gone");
        assert!(svc.list_orgs().is_empty(), "with its Organisation");
        assert_eq!(postcard::to_allocvec(svc.store_data(&other).unwrap()).unwrap(), other_before, "the other store is unchanged");
    }
}

// ---- LLR-rdda86: one store per Persona -------------------------------------------

// verifies: LLR-rdda86
#[test]
fn two_stores_with_one_device_key_are_refused() {
    let mut svc = OrgService::new(vec![]).unwrap();
    let (_, seed, write) = new_persona(&mut svc, "alice");
    let first = PersonaStore::unseal(&write.sealed, write.device_key, from_keychain(&seed)).unwrap();
    let second = PersonaStore::unseal(&write.sealed, write.device_key, from_keychain(&seed)).unwrap();
    assert!(matches!(OrgService::new(vec![first, second]), Err(OrgNodeError::DuplicatePersonaStore)));
}

// verifies: LLR-rdda86
#[test]
fn an_operation_changes_only_its_personas_store() {
    let mut svc = OrgService::new(vec![]).unwrap();
    let (first, _, first_write) = new_persona(&mut svc, "alice");
    let (second, _, _) = new_persona(&mut svc, "bob");
    let second_before = postcard::to_allocvec(svc.store_data(&second).unwrap()).unwrap();
    let (_, write) = svc.create_organisation(&mut OsRng, &first).unwrap();
    assert_eq!(write.device_key, first_write.device_key, "the write is the first Persona's");
    assert_eq!(postcard::to_allocvec(svc.store_data(&second).unwrap()).unwrap(), second_before);
    assert!(svc.genesis_provisional_updates(&second).is_empty());
    assert_eq!(svc.genesis_provisional_updates(&first).len(), 1);
}

// verifies: LLR-rdda86
#[test]
fn list_personas_returns_one_record_per_store() {
    let mut svc = OrgService::new(vec![]).unwrap();
    let mut created = vec![];
    for handle in ["alice", "bob", "carol"] {
        created.push(new_persona(&mut svc, handle));
    }
    let listed: Vec<PersonaId> = svc.list_personas().iter().map(|persona| persona.persona_id.clone()).collect();
    assert_eq!(listed, created.iter().map(|(pid, _, _)| pid.clone()).collect::<Vec<_>>());

    // Reopened from the writes: the same Personas, one store each.
    let stores = created
        .iter()
        .map(|(_, seed, write)| PersonaStore::unseal(&write.sealed, write.device_key, from_keychain(seed)).unwrap())
        .collect();
    let reopened = OrgService::new(stores).unwrap();
    let relisted: Vec<PersonaId> = reopened.list_personas().iter().map(|persona| persona.persona_id.clone()).collect();
    assert_eq!(relisted, listed);
}

// Normal and abnormal (S4, task T9; LLR-rdda86's note of 2026-10-13): org-io
// puts back a Persona's store as its file holds it after a failed write, and
// takes one out when the keychain refused its new Persona; one store per
// Device key either way, and the other stores untouched.
// verifies: LLR-rdda86
#[test]
fn restore_store_replaces_or_adds_one_store_and_drop_store_removes_one() {
    let mut svc = OrgService::new(vec![]).unwrap();
    let (alice, alice_seed, alice_created) = new_persona(&mut svc, "alice");
    let (bob, bob_seed, bob_created) = new_persona(&mut svc, "bob");
    let org_id = org_node::OrgId::new([0x07; 20]);
    let _ahead = svc.expect_admission(&mut OsRng, &alice, org_id).unwrap();
    let bob_before = format!("{:?}", svc.store_data(&bob).unwrap());

    // The store as its file held it replaces the one that ran ahead.
    let alice_on_disk = PersonaStore::unseal(&alice_created.sealed, alice_created.device_key, from_keychain(&alice_seed)).unwrap();
    svc.restore_store(alice_on_disk);
    assert_eq!(svc.list_personas().len(), 2, "replaced, not added");
    assert!(svc.store_data(&alice).unwrap().expected_admissions.is_empty(), "the store is the file's again");
    assert_eq!(format!("{:?}", svc.store_data(&bob).unwrap()), bob_before, "another Persona's store is untouched");

    // Dropped, and dropped again: the second finds nothing.
    assert!(svc.drop_store(&bob_created.device_key));
    assert!(!svc.drop_store(&bob_created.device_key));
    assert!(svc.store_data(&bob).is_none());
    assert_eq!(svc.list_personas().len(), 1);

    // A store the service does not hold is added back.
    let bob_on_disk = PersonaStore::unseal(&bob_created.sealed, bob_created.device_key, from_keychain(&bob_seed)).unwrap();
    svc.restore_store(bob_on_disk);
    assert_eq!(svc.list_personas().len(), 2);
    assert_eq!(format!("{:?}", svc.store_data(&bob).unwrap()), bob_before);
}

// ---- LLR-e5kqhg: create_persona --------------------------------------------------

// verifies: LLR-e5kqhg
#[test]
fn create_persona_returns_the_secret_and_keeps_only_the_public_key() {
    let mut svc = OrgService::new(vec![]).unwrap();
    let (pid, secret, write) = svc.create_persona(&mut OsRng, h("alice"), nm("A"), sn("U")).unwrap();
    let record = svc.list_personas().into_iter().find(|persona| persona.persona_id == pid).unwrap().clone();
    assert_eq!(secret.device_key(), record.device_key);
    assert_eq!(write.device_key, record.device_key);
    let (_, device_key) = svc.persona_public_keys(&pid).unwrap();
    assert_eq!(device_key, secret.device_key());
}

/// A random source that returns one byte for ever: two draws from two of
/// them give the same seed.
struct ConstRng(u8);

impl rand::RngCore for ConstRng {
    fn next_u32(&mut self) -> u32 {
        u32::from_le_bytes([self.0; 4])
    }
    fn next_u64(&mut self) -> u64 {
        u64::from_le_bytes([self.0; 8])
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.fill(self.0);
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        dest.fill(self.0);
        Ok(())
    }
}

impl rand::CryptoRng for ConstRng {}

// Abnormal: the random source repeats, so the second Persona's device seed
// is one the service already holds a store for. Refused with
// `DuplicatePersonaStore`, no write, and the service keeps one Persona, the
// first (S4, task T23).
// verifies: LLR-e5kqhg
#[test]
fn create_persona_refuses_a_seed_whose_store_is_already_held_and_keeps_nothing() {
    let mut svc = OrgService::new(vec![]).unwrap();
    let (first, secret, _) = svc.create_persona(&mut ConstRng(0x42), h("alice"), nm("A"), sn("U")).unwrap();
    let refused = svc.create_persona(&mut ConstRng(0x42), h("bob"), nm("B"), sn("U"));
    assert!(matches!(refused, Err(OrgNodeError::DuplicatePersonaStore)), "{:?}", refused.map(|(pid, _, _)| pid));
    let personas = svc.list_personas();
    assert_eq!(personas.len(), 1, "nothing kept for the refused Persona");
    assert_eq!((&personas[0].persona_id, personas[0].device_key), (&first, secret.device_key()));
    // The contrast: another seed is a second Persona.
    let (_, other, _) = svc.create_persona(&mut ConstRng(0x43), h("bob"), nm("B"), sn("U")).unwrap();
    assert_ne!(other.device_key(), secret.device_key());
    assert_eq!(svc.list_personas().len(), 2);
}

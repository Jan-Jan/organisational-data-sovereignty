#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The Organisation public and private key. A created Organisation publishes
//! the X25519 public key of a secret generated for it alone, distinct from every key of its
//! genesis record, and that secret is kept in the creator's encrypted store
//! (REQ-ech45n), and every Member's record holds it once admitted
//! (LLR-qw8tyu).
//!
//! The gate: `cargo test -p org-node --features app,test-support --test organisation_key`
//!
//! The key pair is drawn by `propose_founding`; the custody tests run the
//! founding through the operations (LLR-qw8tyu).

use org_node::chain::OrgState;
use org_node::error::OrgNodeError;
use org_node::test_fixtures::{device_keypair, ChainSlots};
use org_node::{Epoch, MemberSeed, OrgPrivateKey, OrgPublicKey};
use rand::rngs::OsRng;

mod support;
use support::{found_on_chain, handle, parsed_name, parsed_surname, ConstRng, Node};

use support::sweep;

// The Organisation private key never reaches a log: the record's Debug output
// names the field and does not print the key. The key is held in a redacted
// secret type (REQ-y7tsft), so the assertion reads the key's bytes, not its
// `Debug`. REQ-ech45n says nothing of how a record renders, so the test
// verifies the derived LLR only.
// verifies: LLR-2dvhz8
#[test]
fn the_organisation_private_key_is_not_in_the_record_debug_output() {
    let chain = ChainSlots::new();
    let mut node = Node::new(support::open_store("debug", "a"));
    let persona_id = node.propose_and_keep_founding(&mut OsRng, handle("admin"), parsed_name("Admin"), parsed_surname("User")).unwrap();
    found_on_chain(&mut node, &chain, &persona_id);
    let record = &node.list_orgs()[0];
    let secret = record.org_private_key_for_test().clone();

    let printed = format!("{record:?}");
    assert!(printed.contains("org_private_key"), "the field is named in the output");
    let bytes = secret.expose_secret();
    assert!(!printed.contains(&format!("{bytes:?}")), "the Organisation private key is printed");
    assert!(!printed.contains(&support::hex(bytes)), "the Organisation private key is printed in hex");
}

// The parse: canonical, non-small-order X25519 keys only, by person's rule.
// verifies: REQ-8jb4ny, LLR-3jjgtw
#[test]
fn an_organisation_public_key_that_is_not_a_valid_x25519_key_is_refused() {
    let valid = MemberSeed::from([0x42u8; 32]).x25519_keypair().public_bytes();
    assert_eq!(OrgPublicKey::parse(&valid).unwrap().as_bytes(), &valid);

    let mut top_bit = valid;
    top_bit[31] |= 0x80; // at least 2^255: not canonical
    let mut small_order = [0u8; 32];
    small_order[0] = 1; // u = 1 has order 4: every clamped private key gives it the same shared secret
    let mut prime = [0xffu8; 32]; // 2^255 - 19: not canonical
    prime[0] = 0xed;
    prime[31] = 0x7f;
    for bad in [[0u8; 32], small_order, top_bit, prime] {
        assert_eq!(
            OrgPublicKey::parse(&bad),
            Err(OrgNodeError::InvalidOrgPublicKey),
            "{bad:02x?} must be refused"
        );
    }
}

// The edge: an Organisation state read from the chain is refused, with the
// typed error, when its key is not valid, and accepted with its fields
// intact when it is.
// verifies: REQ-8jb4ny
#[test]
fn an_organisation_state_read_with_an_invalid_key_is_refused() {
    assert_eq!(
        OrgState::from_chain([7u8; 32], [0u8; 32], 3),
        Err(OrgNodeError::InvalidOrgPublicKey)
    );
    let key = MemberSeed::from([0x42u8; 32]).x25519_keypair().public_bytes();
    let state = OrgState::from_chain([7u8; 32], key, 3).unwrap();
    assert_eq!(state.org_pub_key.as_bytes(), &key);
    assert_eq!(state.root_hash.as_bytes(), &[7u8; 32]);
    assert_eq!(state.epoch, Epoch::new(3));
}

// A receive against a chain answer with an invalid Organisation public key is
// org-io's, with the chain read; the two tests above are the parse's
// evidence.

// The distinctness check at its own interface, `OrgPublicKey::
// ensure_distinct_from` (SDD-swtd3w; TEST_GUIDELINES Seams 2, criterion 4):
// a key equal to a genesis Member-as-a-group key or DevicePublicKey is
// refused, a fresh key accepted. Through `propose_founding` an X25519 key
// drawn at creation equals an ed25519 DevicePublicKey only by a 2^-128
// accident, which no boundary fake can arrange (it would need the discrete
// log of a Device key); a DevicePublicKey whose bytes happen to be a valid
// X25519 key stands in for the accident. `propose_founding`'s Member-key
// clause is verified through the operation below.
// verifies: LLR-c759yd
#[test]
fn an_organisation_key_equal_to_any_genesis_member_or_device_key_is_refused() {
    use org_members::{Handle, MemberId, MemberLeaf, Name, Surname};
    let device = (0u8..=255)
        .map(|seed_byte| device_keypair([seed_byte; 32]))
        .find(|keypair| OrgPublicKey::parse(keypair.verifying_key().as_bytes()).is_ok())
        .expect("some DevicePublicKey is also a valid X25519 key");
    let member = MemberSeed::from([0x71u8; 32]).x25519_keypair();
    let leaf = MemberLeaf::new(
        MemberId::new([1u8; 32]),
        Handle::parse("admin").unwrap(),
        member.member_key().unwrap(),
        Name::parse("Admin").unwrap(),
        Surname::parse("User").unwrap(),
        vec![device.device_key().unwrap()],
    )
    .unwrap();
    let members = [leaf];

    let as_device = OrgPublicKey::parse(device.verifying_key().as_bytes()).unwrap();
    assert_eq!(
        as_device.ensure_distinct_from(&members),
        Err(OrgNodeError::DuplicateOrgPublicKey),
        "equal to a DevicePublicKey"
    );
    let as_member = member.org_public_key().unwrap();
    assert_eq!(
        as_member.ensure_distinct_from(&members),
        Err(OrgNodeError::DuplicateOrgPublicKey),
        "equal to a Member-as-a-group key"
    );
    let fresh = OrgPrivateKey::from([0x72u8; 32]).x25519_keypair().org_public_key().unwrap();
    assert_eq!(fresh.ensure_distinct_from(&members), Ok(()), "a fresh key is accepted");
}

// Abnormal: through `propose_founding`, with randomness faked at its boundary
// (`ConstRng`: every seed drawn is the same) so that the Organisation key pair
// drawn is the founder's member key pair, the founding is refused with
// `DuplicateOrgPublicKey`, adding no store and returning no secret.
// verifies: LLR-9qavm7
#[test]
fn propose_founding_refuses_an_organisation_key_equal_to_the_founders_member_key() {
    let mut svc = org_node::OrgService::new(vec![]).unwrap();
    let refused = svc.propose_founding(&mut ConstRng(0x42), "Ada", "Lovelace", "ada", "Acme".into()).map(|(device, _, _)| device);
    assert_eq!(refused.unwrap_err(), OrgNodeError::DuplicateOrgPublicKey);
    assert!(svc.list_personas().is_empty(), "no store is added");
    let (_, _, _) = svc.propose_founding(&mut OsRng, "Ada", "Lovelace", "ada", "Acme".into()).expect("a fresh draw is accepted");
}

// Normal case: the founding's public key is the
// X25519 public key of the private key drawn for it, which the record
// takes at the founding's adoption; no member or device key of the founding
// record equals it, and it is neither the member seed nor the Device seed.
// verifies: REQ-ech45n, LLR-9qavm7, LLR-qw8tyu
#[test]
fn a_created_organisation_publishes_a_fresh_key_no_genesis_key_equals() {
    let mut founder = sweep::found();
    let chain = sweep::founding_state(&founder);
    let device_seed = support::seed_bytes(&founder.secret());
    sweep::adopt(&mut founder, chain);
    let record = sweep::record_of(&founder);
    let published = *chain.org_pub_key.as_bytes();
    assert_eq!(*record.org_pub_key.as_bytes(), published, "the record holds the key the founding proposed");
    let key = record.org_private_key_for_test().clone();
    assert_eq!(key.x25519_keypair().public_bytes(), published, "the public key of the private key held");
    for member in &record.trie_members {
        assert_ne!(&published, member.member_key.as_bytes(), "a Member-as-a-group key of the founding record");
        for device_key in &member.device_keys {
            assert_ne!(&published, device_key.as_bytes(), "a DevicePublicKey of the founding record");
        }
    }
    let persona = &support::held_store(&founder.svc, &founder.device).persona;
    assert_ne!(key.expose_secret(), persona.member_seed_for_test().expose_secret(), "the private key is the member seed");
    assert_ne!(key.expose_secret().as_slice(), device_seed.as_slice(), "the private key is the device seed");
}

// The Organisation private key: in the provisional
// founding and then in the record, it reaches the disk only inside the
// sealed store, never in clear, and only the Persona's own Device secret
// opens it.
// verifies: REQ-ech45n, LLR-qw8tyu
#[test]
fn the_organisation_private_key_is_kept_only_in_the_encrypted_store() {
    use org_node::store::PersonaStore;
    use org_node::test_fixtures::device_secret;

    let mut founder = sweep::found();
    let before_adoption = founder.last_write().sealed.as_bytes().to_vec();
    let chain = sweep::founding_state(&founder);
    sweep::adopt(&mut founder, chain);
    let key = sweep::record_of(&founder).org_private_key_for_test().clone();
    assert!(!support::contains(&before_adoption, key.expose_secret()), "the founding's key is in the file in clear");
    let write = founder.last_write();
    assert!(!support::contains(write.sealed.as_bytes(), key.expose_secret()), "the record's key is in the file in clear");
    assert!(
        PersonaStore::unseal(&write.sealed, write.device_key, device_secret([0x99; 32])).is_err(),
        "only under the Persona's own device secret"
    );
    let reopened = founder.reopened_service();
    let on_disk = support::held_store(&reopened, &founder.device).record.as_ref().unwrap();
    assert_eq!(on_disk.org_private_key_for_test().clone(), key, "the sealed store holds the key");
}

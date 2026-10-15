#![cfg(feature = "app")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Encrypted-at-rest evidence for `PersonaStore` (REQ-hzm4kt): the store
//! round-trips through its sealed bytes, another Device's secret yields an
//! error rather than data, and neither a persona secret nor the Organisation
//! private key appears in the clear in the sealed bytes.
//!
//! *Adapted 2026-10-08 (S4, task T3 of
//! docs/plans/2026-10-06-org-io-transport.md):* org-node writes no file and
//! takes no passphrase. A store is sealed under a key derived from its
//! Persona's device secret (LLR-2qqqx6) and handed out as a `StoreWrite`,
//! whose bytes org-io writes; "the file" below is those bytes, and "the wrong
//! passphrase" is another Device's secret. Annotations unchanged.

use org_members::{Handle, MemberId, Name, RootHash, Surname};
use org_node::ids::OrgId;
use org_node::store::{
    ExpectedAdmission, OrgRecord, PersonaRecord, PersonaStatus, PersonaStore, ProvisionalChange, ProvisionalUpdate,
    SealedStore, StoreData, StoreRefusal, StoreWrite,
};
use org_node::service::OrgService;
use org_node::test_fixtures::{device_secret, empty_store, store_for_test};
use org_node::{Epoch, MemberSeed, OrgNodeError, OrgPrivateKey, OrgPublicKey, PersonaId, SequenceNumber};
use rand::rngs::OsRng;

fn persona(member_seed: [u8; 32], device_seed: [u8; 32]) -> PersonaRecord {
    PersonaRecord::from(org_node::store::PersonaRecordParts {
        persona_id: PersonaId::new("p1".to_string()),
        org_id: None,
        handle: Handle::parse("alice").unwrap(),
        name: Name::parse("A").unwrap(),
        surname: Surname::parse("U").unwrap(),
        member_seed: MemberSeed::from(member_seed),
        device_key: device_secret(device_seed).device_key(),
        member_id: None,
        status: PersonaStatus::Proposed,
    })
}

/// An Organisation no store below names: what `sealed` expects an admission
/// to, so the write it takes changes nothing a test reads back.
const UNNAMED_ORG: [u8; 20] = [0xEE; 20];

/// `data`, as its Persona's store whose device seed is `device_seed`, opened
/// in an `OrgService` and written by one of its operations: the `StoreWrite`
/// `expect_admission` returns (LLR-bvt5up).
///
/// *Moved 2026-10-15 (S4 TEST review round 1, finding-5; Existing suites 1):*
/// this sealed the store through the test hook `seal_for_test`, below the
/// seam. The bytes the tests read are now those an operation hands org-io to
/// write; the assertions on them are unchanged. The store is still opened
/// from `data` with `store_for_test`: it stands for the store org-io would
/// have unsealed and handed to `OrgService::new`. Mutation checks, that day
/// (each removed): `seal_bytes` appending the plaintext in place of the
/// ciphertext failed the four tests that read `sealed`'s bytes back or look
/// for secrets in them; `seal_bytes` with no nonce draw failed
/// `each_save_draws_a_fresh_nonce`; `PersonaStore::unseal` with no
/// device-secret check failed `wrong_passphrase_yields_error_not_data`.
fn sealed(device_seed: [u8; 32], data: StoreData) -> StoreWrite {
    let persona_id = data.persona.persona_id.clone();
    let mut service = OrgService::new(vec![store_for_test(&device_secret(device_seed), data)]).unwrap();
    service.expect_admission(&mut OsRng, &persona_id, OrgId::new(UNNAMED_ORG)).unwrap()
}

/// The sealed bytes of `write`, opened with the secret of `device_seed`.
fn unseal(write: &StoreWrite, device_seed: [u8; 32]) -> Result<PersonaStore, OrgNodeError> {
    PersonaStore::unseal(&write.sealed, write.device_key, device_secret(device_seed))
}

/// An org record holding the Organisation private key `key`, with the
/// remaining fields fixed.
fn org_record(key: [u8; 32]) -> OrgRecord {
    OrgRecord::from(org_node::store::OrgRecordParts {
        org_id: OrgId::new([5u8; 20]),
        root_hash: RootHash::new([0x11u8; 32]),
        org_pub_key: OrgPrivateKey::from(key).x25519_keypair().org_public_key().unwrap(),
        epoch: Epoch::new(3),
        last_seq: SequenceNumber::new(2),
        trie_members: Vec::new(),
        proxy_account: None,
        org_private_key: OrgPrivateKey::from(key),
        kept_change_set: None,
    })
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}

// verifies: REQ-hzm4kt, LLR-wusj89, LLR-8mfjey
#[test]
fn round_trips_encrypted_through_disk() {
    let write = sealed([2u8; 32], empty_store(persona([1u8; 32], [2u8; 32])));

    // Reopen with the right secret.
    let s2 = unseal(&write, [2u8; 32]).unwrap();
    assert_eq!(s2.data().persona.handle.as_str(), "alice");

    // Another Device's secret must fail.
    assert!(unseal(&write, [3u8; 32]).is_err());
}

// verifies: REQ-hzm4kt, LLR-t4u66w
#[test]
fn wrong_passphrase_yields_error_not_data() {
    let write = sealed([8u8; 32], empty_store(persona([7u8; 32], [8u8; 32])));
    let err = unseal(&write, [9u8; 32]).err().expect("must fail");
    assert_eq!(err, OrgNodeError::DeviceSecretMismatch, "got {err}");
    // The right secret over bytes that do not authenticate: an error too.
    let mut tampered = write.sealed.as_bytes().to_vec();
    tampered[24] ^= 0x80;
    let tampered = StoreWrite { device_key: write.device_key, sealed: SealedStore::from_bytes(tampered) };
    let err = unseal(&tampered, [8u8; 32]).err().expect("must fail");
    assert_eq!(err, OrgNodeError::StoreRefused(StoreRefusal::NotAuthentic), "got {err}");
}

// verifies: REQ-hzm4kt, LLR-s78sh7
#[test]
fn seeds_do_not_appear_in_the_file() {
    let member_seed = [0x5au8; 32];
    let device_seed = [0xa5u8; 32];
    let bytes = sealed(device_seed, empty_store(persona(member_seed, device_seed))).sealed.as_bytes().to_vec();
    assert!(!contains(&bytes, &member_seed), "member seed in clear");
    assert!(!contains(&bytes, &device_seed), "device seed in clear");
    assert!(!contains(&bytes, b"alice"), "handle in clear");
}

// *Renamed 2026-10-06 (change worktree-org-node-org-key-pair).* Was
// `organisation_secret_does_not_appear_in_the_file`; LLR-s78sh7 names the
// Organisation private key in the secret's place.
// verifies: REQ-hzm4kt, LLR-s78sh7
#[test]
fn organisation_private_key_does_not_appear_in_the_file() {
    let key = [0x7eu8; 32];
    let mut data = empty_store(persona([1u8; 32], [2u8; 32]));
    data.orgs.push(org_record(key));
    let bytes = sealed([2u8; 32], data).sealed.as_bytes().to_vec();
    assert!(!contains(&bytes, &key), "Organisation private key in clear");
}

// ---- added 2026-10-03 by the architecture tooth ----------------------------

/// Sealed bytes of `len` zero bytes, as the store of device seed `[2; 32]`.
fn zeros(len: usize) -> StoreWrite {
    StoreWrite { device_key: device_secret([2u8; 32]).device_key(), sealed: SealedStore::from_bytes(vec![0u8; len]) }
}

// A file too short to hold the nonce is refused as such, rather than being
// read as a nonce plus an empty ciphertext. The nonce is 24 bytes, so every
// length below that is this case; the boundary and a few below it are probed.
// *Adapted 2026-10-08 (S4, task T3; LLR-q5n28x's dated note):* the bound is
// the nonce and the 16-byte authentication tag, 40 bytes, so 39 is probed too.
// verifies: REQ-hzm4kt, LLR-q5n28x
#[test]
fn a_file_shorter_than_the_nonce_is_refused() {
    for len in [0usize, 1, 23, 24, 39] {
        let err = unseal(&zeros(len), [2u8; 32])
            .err()
            .unwrap_or_else(|| panic!("a {len}-byte file opened as a store"));
        assert_eq!(err, OrgNodeError::StoreRefused(StoreRefusal::TooShort), "{len} bytes");
        let rendered = format!("{err}");
        assert!(
            rendered.contains("too short"),
            "a {len}-byte file must be refused as too short, got: {rendered}"
        );
    }
}

// Exactly the nonce length and no ciphertext is NOT the "too short" case: it
// passes the length check and fails to decrypt, which is a different verdict.
// This is what makes the bound `< 24` rather than `<= 24`.
// *Adapted 2026-10-08 (S4, task T3; LLR-q5n28x's dated note):* the bound is
// the nonce and the tag, so the boundary probed is 40 bytes, `< 40` rather
// than `<= 40`.
// verifies: REQ-hzm4kt, LLR-q5n28x
#[test]
fn a_file_of_exactly_the_nonce_length_is_refused_as_undecryptable_not_as_short() {
    let err = unseal(&zeros(40), [2u8; 32]).err().expect("opened a 40-byte file");
    let rendered = format!("{err}");
    assert!(!rendered.contains("too short"), "got the wrong verdict: {rendered}");
    assert_eq!(err, OrgNodeError::StoreRefused(StoreRefusal::NotAuthentic), "got: {rendered}");
}

// Nonce freshness. The falsifiability sweep of 2026-10-03 replaced
// `rng.fill_bytes(&mut nonce)` with nothing — a fixed all-zero nonce on every
// save — and the whole suite stayed green: nothing here was asserting that the
// nonce ever changed. XChaCha20-Poly1305 does not survive nonce reuse under
// the same key, and the key is derived from the Persona's device secret, so
// it IS the same key across seals of one store. This case is what makes that
// mutation red.
// *Moved 2026-10-15 (S4 TEST review round 1, finding-5):* the seals are the
// writes four calls of one operation return on one open store, not
// `seal_for_test`'s; the repeated expectation changes nothing after the
// first, so only the nonce can differ between them.
// verifies: REQ-hzm4kt, LLR-8mfjey
#[test]
fn each_save_draws_a_fresh_nonce() {
    let data = empty_store(persona([1u8; 32], [2u8; 32]));
    let persona_id = data.persona.persona_id.clone();
    let mut service = OrgService::new(vec![store_for_test(&device_secret([2u8; 32]), data)]).unwrap();

    let mut nonces = Vec::new();
    for _ in 0..4 {
        let write = service.expect_admission(&mut OsRng, &persona_id, OrgId::new(UNNAMED_ORG)).unwrap();
        nonces.push(write.sealed.as_bytes()[..24].to_vec());
    }

    for (i, a) in nonces.iter().enumerate() {
        for (j, b) in nonces.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "saves {i} and {j} reused a nonce");
            }
        }
    }
    // And none of them is the all-zero nonce a dropped draw would leave.
    assert!(nonces.iter().all(|n| n.iter().any(|b| *b != 0)));
}

// ---- added 2026-10-07 by stage S3 (docs/plans/2026-10-06-org-io-commit-workflow.md, T2)

const ORG_1: [u8; 20] = [1; 20];
const ORG_2: [u8; 20] = [2; 20];

/// A valid Organisation public key; which one does not matter here.
fn org_public_key() -> OrgPublicKey {
    OrgPrivateKey::from([0x42; 32]).x25519_keypair().org_public_key().unwrap()
}

/// The postcard bytes of `value`. The store types hold secrets and so have no
/// `PartialEq`; two values are compared by their encoding.
fn encoded<T: serde::Serialize>(value: &T) -> Vec<u8> {
    postcard::to_allocvec(value).unwrap()
}

/// *Adapted 2026-10-08 (S4, task T3):* one store holds one Persona. Its
/// Persona is bound to org 1; the store holds a record, one Change-set
/// provisional update and one expectation of each Organisation, plus an
/// unbound genesis update its Persona built.
fn two_organisation_store() -> StoreData {
    let persona = PersonaRecord::from(org_node::store::PersonaRecordParts {
        persona_id: PersonaId::new("p1".to_string()),
        org_id: Some(OrgId::new(ORG_1)),
        handle: Handle::parse("h1").unwrap(),
        name: Name::parse("N").unwrap(),
        surname: Surname::parse("S").unwrap(),
        member_seed: MemberSeed::from([1; 32]),
        device_key: device_secret([11; 32]).device_key(),
        member_id: Some(MemberId::new([1; 32])),
        status: PersonaStatus::Active,
    });
    let record = |org_id: OrgId| {
        OrgRecord::from(org_node::store::OrgRecordParts {
            org_id,
            root_hash: RootHash::new([org_id.as_bytes()[0]; 32]),
            org_pub_key: org_public_key(),
            epoch: Epoch::new(4),
            last_seq: SequenceNumber::new(4),
            trie_members: vec![],
            proxy_account: None,
            org_private_key: OrgPrivateKey::from([org_id.as_bytes()[0]; 32]),
            kept_change_set: Some(vec![org_id.as_bytes()[0]]),
        })
    };
    let update = |org_id: Option<OrgId>, number: u8| {
        ProvisionalUpdate::from(org_node::store::ProvisionalUpdateParts {
            org_id,
            persona_id: PersonaId::new("p1".to_string()),
            base_root: org_id.map(|org| RootHash::new([org.as_bytes()[0]; 32])),
            resulting_root: RootHash::new([number + 50; 32]),
            seq: SequenceNumber::new(5),
            org_pub_key: org_public_key(),
            change: ProvisionalChange::ChangeSet {
                change_set: vec![number],
                org_private_key: OrgPrivateKey::from([number + 60; 32]),
            },
        })
    };
    let (org_1, org_2) = (OrgId::new(ORG_1), OrgId::new(ORG_2));
    StoreData {
        persona,
        orgs: vec![record(org_1), record(org_2)],
        provisional_updates: vec![update(Some(org_1), 1), update(None, 1), update(Some(org_2), 2)],
        expected_admissions: vec![ExpectedAdmission { org_id: org_1 }, ExpectedAdmission { org_id: org_2 }],
        outstanding_invites: vec![],
    }
}

/// verifies: LLR-pba7yu
///
/// Normal: forgetting org 2 removes its record, provisional update and
/// expectation; org 1's data, the Persona and its genesis update are
/// unchanged and in order; nothing names org 2. Forgetting org 1, which the
/// store's one Persona is bound to, removes the Persona with its keys: no
/// store is left (S4, task T3).
#[test]
fn forget_organisation_removes_everything_of_one_organisation_and_nothing_else() {
    let data = two_organisation_store();
    let input_bytes = encoded(&data);
    let after = data.forget_organisation(OrgId::new(ORG_2)).expect("org 2 is not the Persona's");
    assert_eq!(encoded(&after.persona), encoded(&data.persona));
    assert_eq!(encoded(&after.orgs), encoded(&vec![data.orgs[0].clone()]));
    assert_eq!(after.provisional_updates, vec![data.provisional_updates[0].clone(), data.provisional_updates[1].clone()]);
    assert_eq!(after.expected_admissions, vec![ExpectedAdmission { org_id: OrgId::new(ORG_1) }]);
    let after_bytes = encoded(&after);
    assert!(!contains(&after_bytes, &ORG_2), "no tombstone, marker or copy names org 2");
    assert!(!contains(&after_bytes, &[2u8; 32]), "org 2's private key is gone");
    assert!(data.forget_organisation(OrgId::new(ORG_1)).is_none(), "the Persona goes with its Organisation");
    assert_eq!(encoded(&data), input_bytes, "the input is unchanged");
}

/// verifies: LLR-pba7yu
///
/// Abnormal: forgetting an Organisation the store does not hold returns an
/// equal copy.
#[test]
fn forget_organisation_of_an_unheld_organisation_changes_nothing() {
    let data = two_organisation_store();
    assert_eq!(encoded(&data.forget_organisation(OrgId::new([9; 20])).unwrap()), encoded(&data));
}

/// verifies: LLR-pba7yu
///
/// Boundary: forgetting the Organisation of the store's Persona after the
/// other leaves no store; forgetting from a store whose Persona is unbound
/// leaves that Persona alone.
#[test]
fn forget_organisation_of_the_last_organisation_leaves_an_empty_store() {
    let only_org_1 = two_organisation_store().forget_organisation(OrgId::new(ORG_2)).unwrap();
    assert!(only_org_1.forget_organisation(OrgId::new(ORG_1)).is_none());
    let unbound = empty_store(persona([1u8; 32], [2u8; 32]));
    assert_eq!(encoded(&unbound.forget_organisation(OrgId::new(ORG_1)).unwrap()), encoded(&unbound));
}

/// verifies: LLR-d9778a
///
/// `kept_change_set` round-trips through the sealed store, `None` and `Some`.
#[test]
fn kept_change_set_round_trips_through_the_store() {
    for kept in [None, Some(vec![1u8, 2, 3])] {
        let mut data = two_organisation_store();
        data.orgs[0].kept_change_set = kept.clone();
        let write = sealed([11; 32], data);
        let reopened = unseal(&write, [11; 32]).unwrap();
        assert_eq!(reopened.data().orgs[0].kept_change_set, kept);
        assert_eq!(reopened.data().orgs[1].kept_change_set, Some(vec![2]));
    }
}

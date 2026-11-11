//! Two-step absence proofs (SDD-57vaj4): a Vacant proof that a MemberId
//! holds no Member, or a Held proof that the Member it holds lacks a
//! DevicePublicKey, checked against a Membership root.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use common::{keyed, member_content_bytes, member_id, member_key, positioned_device_key};
use org_members::hasher::{Blake3DeviceRootHasher, Blake3Hasher};
use org_members::trie::OrgTrie;
use org_members::types::{Handle, MemberId, MemberLeaf, Name, RootHash, Surname};
use org_members::{AbsenceProof, OrgMembersError};
use sparse_merkle_tree::ProofEnding;

type TestTrie = OrgTrie<Blake3Hasher>;

fn leaf(handle: &str, devices: &[&str]) -> MemberLeaf {
    leaf_at(*member_id(handle).as_bytes(), handle, "Test", "User", devices)
}

fn leaf_at(id: [u8; 32], handle: &str, name: &str, surname: &str, devices: &[&str]) -> MemberLeaf {
    let keys = devices
        .iter()
        .enumerate()
        .map(|(position, seed)| positioned_device_key(seed, position as u8))
        .collect();
    MemberLeaf::new(
        MemberId::new(id),
        Handle::parse(handle).unwrap(),
        member_key(&format!("{handle}-mk")),
        Name::parse(name).unwrap(),
        Surname::parse(surname).unwrap(),
        keys,
    )
    .unwrap()
}

/// A calculated three-member organisation.
fn org() -> TestTrie {
    TestTrie::genesis(vec![
        leaf("alice", &["alice-d1", "alice-d2"]),
        leaf("bob", &["bob-d1"]),
        leaf("carol", &["carol-d1"]),
    ])
    .unwrap()
}

/// `org()` after alice's second device is removed and recalculated.
fn org_without_alice_d2() -> TestTrie {
    org()
        .delete_p2p_device(&member_id("alice"), &positioned_device_key("alice-d2", 1), member_key("alice-mk2"))
        .unwrap()
        .recalculate()
        .unwrap()
        .0
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| window == needle)
}

fn verifies(trie: &TestTrie, proof: &AbsenceProof, id: &MemberId, device: &org_members::types::DevicePublicKey) -> Result<(), OrgMembersError> {
    proof.verify::<Blake3Hasher>(&trie.root_hash().unwrap(), id, device)
}

/// verifies: LLR-bh53rr, LLR-v3cysh, REQ-sb2frw
#[test]
fn prove_absent_refuses_an_uncalculated_trie() {
    let pending = org().add_member(leaf("dave", &["dave-d1"])).unwrap();
    assert_eq!(
        pending.prove_absent(&member_id("eve"), &positioned_device_key("eve-d1", 0)).unwrap_err(),
        OrgMembersError::HashesNotCalculated
    );
    assert_eq!(
        pending.prove_absent(&member_id("alice"), &positioned_device_key("alice-d9", 9)).unwrap_err(),
        OrgMembersError::HashesNotCalculated
    );
}

/// verifies: LLR-bh53rr, LLR-v3cysh, REQ-sb2frw
#[test]
fn prove_absent_refuses_a_device_the_member_holds() {
    assert_eq!(
        org().prove_absent(&member_id("alice"), &positioned_device_key("alice-d2", 1)).unwrap_err(),
        OrgMembersError::DeviceStillHeld
    );
}

/// verifies: LLR-bh53rr, REQ-sb2frw
#[test]
fn prove_absent_ignores_a_key_another_member_holds() {
    let trie = org();
    let proof = trie.prove_absent(&member_id("alice"), &positioned_device_key("bob-d1", 0)).unwrap();
    assert_eq!(verifies(&trie, &proof, &member_id("alice"), &positioned_device_key("bob-d1", 0)), Ok(()));
}

/// verifies: LLR-v6smww, LLR-bh53rr, REQ-yyuxh8, REQ-sb2frw
///
/// A MemberId holding no Member gets a Vacant proof, with no Member data.
#[test]
fn a_vacant_proof_carries_no_member_data() {
    let trie = org();
    let proof = trie.prove_absent(&member_id("ghost"), &positioned_device_key("ghost-d1", 0)).unwrap();
    assert!(proof.is_vacant());
    assert!(proof.member().is_none() && proof.inclusion().is_none() && proof.device_root_absence().is_none());
    let bytes = postcard::to_allocvec(&proof).unwrap();
    for name in ["alice", "bob", "carol"] {
        assert!(!contains(&bytes, name.as_bytes()), "{name}");
        assert!(!contains(&bytes, member_key(&format!("{name}-mk")).as_bytes()), "{name}'s key");
    }
}

/// verifies: LLR-v6smww, LLR-bh53rr, REQ-yyuxh8, REQ-sb2frw
///
/// A Member lacking the device gets a Held proof: its own fields and device
/// root, and no DevicePublicKey — not the absent one, not its held ones.
#[test]
fn a_held_proof_carries_the_member_without_device_keys() {
    let trie = org_without_alice_d2();
    let proof = trie.prove_absent(&member_id("alice"), &positioned_device_key("alice-d2", 1)).unwrap();
    let member = proof.member().expect("held");
    let alice = trie.get(&member_id("alice")).unwrap();
    assert_eq!(member.id(), alice.id());
    assert_eq!(member.handle(), alice.handle());
    assert_eq!(member.person_key(), alice.p2p_key());
    assert_eq!(member.name(), alice.name());
    assert_eq!(member.surname(), alice.surname());
    assert_eq!(
        Ok(*member.device_root()),
        person::compute_device_root::<Blake3DeviceRootHasher>(alice.p2p_devices())
    );
    assert_eq!(proof.device_root_absence().unwrap().map().len(), 1, "a depth-4 device proof");
    assert_eq!(proof.inclusion().unwrap().map().len(), 32, "a depth-256 inclusion proof");
    let bytes = postcard::to_allocvec(&proof).unwrap();
    for key in [positioned_device_key("alice-d1", 0), positioned_device_key("alice-d2", 1)] {
        assert!(!contains(&bytes, key.as_bytes()), "a DevicePublicKey in the proof");
    }
    for name in ["bob", "carol"] {
        assert!(!contains(&bytes, name.as_bytes()), "{name}");
    }
}

/// verifies: LLR-54gp2f, LLR-25nmwc, REQ-65xrgh, REQ-sb2frw
///
/// Every honest proof verifies: an absent member, a deleted member, a removed
/// device.
#[test]
fn honest_proofs_verify() {
    let trie = org();
    let ghost = trie.prove_absent(&member_id("ghost"), &positioned_device_key("ghost-d1", 0)).unwrap();
    assert_eq!(verifies(&trie, &ghost, &member_id("ghost"), &positioned_device_key("ghost-d1", 0)), Ok(()));

    let without_bob = trie.delete_member(&member_id("bob")).unwrap().recalculate().unwrap().0;
    let deleted = without_bob.prove_absent(&member_id("bob"), &positioned_device_key("bob-d1", 0)).unwrap();
    assert!(deleted.is_vacant());
    assert_eq!(verifies(&without_bob, &deleted, &member_id("bob"), &positioned_device_key("bob-d1", 0)), Ok(()));

    let removed = org_without_alice_d2();
    let gone = removed.prove_absent(&member_id("alice"), &positioned_device_key("alice-d2", 1)).unwrap();
    assert!(!gone.is_vacant());
    assert_eq!(verifies(&removed, &gone, &member_id("alice"), &positioned_device_key("alice-d2", 1)), Ok(()));
}

/// verifies: LLR-54gp2f, LLR-v3cysh, REQ-65xrgh, RC-j2znx8
///
/// The root is checked first, for each step.
#[test]
fn a_proof_is_refused_against_another_root() {
    let removed = org_without_alice_d2();
    let other = RootHash::new([0; 32]);
    let held = removed.prove_absent(&member_id("alice"), &positioned_device_key("alice-d2", 1)).unwrap();
    assert_eq!(
        held.verify::<Blake3Hasher>(&other, &member_id("alice"), &positioned_device_key("alice-d2", 1)),
        Err(OrgMembersError::AbsenceProofRootMismatch)
    );
    let vacant = removed.prove_absent(&member_id("ghost"), &positioned_device_key("ghost-d1", 0)).unwrap();
    assert_eq!(
        vacant.verify::<Blake3Hasher>(&other, &member_id("ghost"), &positioned_device_key("ghost-d1", 0)),
        Err(OrgMembersError::AbsenceProofRootMismatch)
    );
    // The previous root: alice still held alice-d2 there.
    let before = org().root_hash().unwrap();
    assert_eq!(
        held.verify::<Blake3Hasher>(&before, &member_id("alice"), &positioned_device_key("alice-d2", 1)),
        Err(OrgMembersError::AbsenceProofRootMismatch)
    );
}

/// verifies: LLR-54gp2f, REQ-65xrgh
///
/// A proof answers for its own MemberId and device only.
#[test]
fn a_proof_is_refused_for_another_member_or_device() {
    let removed = org_without_alice_d2();
    let held = removed.prove_absent(&member_id("alice"), &positioned_device_key("alice-d2", 1)).unwrap();
    assert_eq!(
        verifies(&removed, &held, &member_id("bob"), &positioned_device_key("alice-d2", 1)),
        Err(OrgMembersError::AbsenceProofRootMismatch),
        "a Held proof for another MemberId"
    );
    let vacant = removed.prove_absent(&member_id("ghost"), &positioned_device_key("ghost-d1", 0)).unwrap();
    assert_eq!(
        verifies(&removed, &vacant, &member_id("bob"), &positioned_device_key("ghost-d1", 0)),
        Err(OrgMembersError::AbsenceProofRootMismatch),
        "a Vacant proof for a held MemberId at another position"
    );
    // A device at another position than the proof's: the device step does not resolve.
    assert_eq!(
        verifies(&removed, &held, &member_id("alice"), &positioned_device_key("other", 5)),
        Err(OrgMembersError::AbsenceProofRootMismatch)
    );
}

/// verifies: LLR-25nmwc, LLR-v3cysh, REQ-65xrgh
///
/// A resolving Held proof whose device step ends in the asked key is
/// `DeviceStillHeld`: a proof for a key at alice-d1's position, asked about
/// alice-d1.
#[test]
fn a_proof_ending_in_the_asked_device_is_refused_as_held() {
    let trie = org();
    let probe = positioned_device_key("probe", 0); // alice-d1's position, another key
    let proof = trie.prove_absent(&member_id("alice"), &probe).unwrap();
    assert_eq!(verifies(&trie, &proof, &member_id("alice"), &probe), Ok(()));
    assert!(matches!(proof.device_root_absence().unwrap().ending(), ProofEnding::Occupied { .. }));
    assert_eq!(
        verifies(&trie, &proof, &member_id("alice"), &positioned_device_key("alice-d1", 0)),
        Err(OrgMembersError::DeviceStillHeld)
    );
}

/// verifies: LLR-25nmwc, LLR-9g2kjg, REQ-65xrgh
///
/// A Vacant proof that resolves to the root while ending in the asked
/// MemberId's own leaf is refused as malformed: built from alice's inclusion
/// path and her leaf's two hashes, as only a forger would build it.
#[test]
fn a_vacant_proof_ending_in_the_member_is_refused() {
    let trie = org();
    let held = trie.prove_absent(&member_id("alice"), &positioned_device_key("probe", 5)).unwrap();
    let inclusion = held.inclusion().unwrap();
    let alice = trie.get(&member_id("alice")).unwrap();
    let device_root = held.member().unwrap().device_root();
    let identity_hash = keyed(b"org-members::member-identity____", &[alice.id().as_bytes()]);
    let content_hash = keyed(b"org-members::member-content_____", &[&member_content_bytes(&alice, device_root.as_bytes())]);
    let forged = sparse_merkle_tree::AbsenceProof::<256>::from_parts(
        inclusion.map().to_vec(),
        inclusion.siblings().to_vec(),
        ProofEnding::Occupied { identity_hash, content_hash },
    )
    .unwrap();
    let forged: AbsenceProof =
        serde_json::from_value(serde_json::json!({ "Vacant": serde_json::to_value(&forged).unwrap() })).unwrap();
    assert_eq!(
        verifies(&trie, &forged, &member_id("alice"), &positioned_device_key("probe", 5)),
        Err(OrgMembersError::AbsenceProofMalformed)
    );
}

/// verifies: LLR-9g2kjg, LLR-54gp2f, REQ-65xrgh
#[test]
fn a_proof_survives_the_wire_and_still_verifies() {
    let removed = org_without_alice_d2();
    for (id, probe) in [("alice", positioned_device_key("alice-d2", 1)), ("ghost", positioned_device_key("ghost-d1", 0))] {
        let proof = removed.prove_absent(&member_id(id), &probe).unwrap();
        let back: AbsenceProof = postcard::from_bytes(&postcard::to_allocvec(&proof).unwrap()).unwrap();
        assert_eq!(back, proof);
        assert_eq!(verifies(&removed, &back, &member_id(id), &probe), Ok(()));
    }
}

/// verifies: LLR-9g2kjg, LLR-v3cysh, REQ-ds8ryr
///
/// Whatever a field's own parse or an inner proof's bounded parse refuses,
/// parsing refuses as `AbsenceProofMalformed`; postcard sees its custom
/// error.
#[test]
fn the_wire_refuses_malformed_proofs() {
    let removed = org_without_alice_d2();
    let proof = removed.prove_absent(&member_id("alice"), &positioned_device_key("alice-d2", 1)).unwrap();
    let json = serde_json::to_value(&proof).unwrap();
    let mut bad_handle = json.clone();
    bad_handle["Held"]["member"]["handle"] = serde_json::json!("NOT A HANDLE");
    let mut bad_depth = json.clone();
    bad_depth["Held"]["device_root_absence"][0] = serde_json::json!(8);
    let mut bad_inclusion = json.clone();
    bad_inclusion["Held"]["inclusion"][1] = serde_json::json!([]);
    for broken in [bad_handle, bad_depth, bad_inclusion, serde_json::json!({ "Other": 1 })] {
        let error = serde_json::from_value::<AbsenceProof>(broken).unwrap_err();
        assert_eq!(error.to_string(), "malformed absence proof");
    }
    let bytes = postcard::to_allocvec(&proof).unwrap();
    let mut truncated = bytes.clone();
    truncated.pop();
    assert_eq!(postcard::from_bytes::<AbsenceProof>(&truncated), Err(postcard::Error::SerdeDeCustom));
}

fn last_bit_flipped(mut bytes: [u8; 32]) -> [u8; 32] {
    bytes[31] ^= 1;
    bytes
}

/// verifies: LLR-v6smww, REQ-yyuxh8, RC-qa2758
///
/// A neighbour sharing all but the last bit of the MemberId: neither a
/// Vacant nor a Held proof next to it carries its handle, names, member key
/// or device key.
#[test]
fn a_proof_next_to_a_neighbour_carries_none_of_its_data() {
    let subject: [u8; 32] = blake3::hash(b"subject").into();
    let neighbour = last_bit_flipped(subject);
    let genesis = TestTrie::genesis(vec![
        leaf_at(subject, "subject", "Test", "User", &["subject-d1", "subject-d2"]),
        leaf_at(neighbour, "neighbour", "Zebulon", "Quixote", &["neighbour-d1"]),
    ])
    .unwrap();
    let no_neighbour = |bytes: &[u8]| {
        for needle in [
            b"neighbour".as_slice(),
            b"Zebulon".as_slice(),
            b"Quixote".as_slice(),
            member_key("neighbour-mk").as_bytes().as_slice(),
            positioned_device_key("neighbour-d1", 0).as_bytes().as_slice(),
        ] {
            assert!(!contains(bytes, needle));
        }
    };
    let id = MemberId::new(subject);
    let without = genesis.delete_member(&id).unwrap().recalculate().unwrap().0;
    let vacant = without.prove_absent(&id, &positioned_device_key("subject-d1", 0)).unwrap();
    no_neighbour(&postcard::to_allocvec(&vacant).unwrap());
    assert_eq!(verifies(&without, &vacant, &id, &positioned_device_key("subject-d1", 0)), Ok(()));
    let removed = genesis
        .delete_p2p_device(&id, &positioned_device_key("subject-d2", 1), member_key("subject-mk2"))
        .unwrap()
        .recalculate()
        .unwrap()
        .0;
    let held = removed.prove_absent(&id, &positioned_device_key("subject-d2", 1)).unwrap();
    no_neighbour(&postcard::to_allocvec(&held).unwrap());
    assert_eq!(verifies(&removed, &held, &id, &positioned_device_key("subject-d2", 1)), Ok(()));
}

/// verifies: LLR-54gp2f, LLR-bh53rr
///
/// The lowest and highest MemberIds (all bits 0, all bits 1), present and
/// absent, and a one-member organisation.
#[test]
fn proofs_at_the_edges_verify() {
    const LOWEST: [u8; 32] = [0; 32];
    const HIGHEST: [u8; 32] = [0xff; 32];
    let edges = TestTrie::genesis(vec![
        leaf_at(LOWEST, "low", "Test", "User", &["low-d1"]),
        leaf_at(HIGHEST, "high", "Test", "User", &["high-d1"]),
    ])
    .unwrap();
    for id in [LOWEST, HIGHEST] {
        let proof = edges.prove_absent(&MemberId::new(id), &positioned_device_key("probe", 7)).unwrap();
        assert_eq!(verifies(&edges, &proof, &MemberId::new(id), &positioned_device_key("probe", 7)), Ok(()));
    }
    let one = TestTrie::genesis(vec![leaf("solo", &["solo-d1"])]).unwrap();
    for id in [LOWEST, HIGHEST, *member_id("solo").as_bytes()] {
        let proof = one.prove_absent(&MemberId::new(id), &positioned_device_key("probe", 7)).unwrap();
        assert_eq!(verifies(&one, &proof, &MemberId::new(id), &positioned_device_key("probe", 7)), Ok(()));
    }
    let empty = TestTrie::genesis(vec![]).unwrap();
    let proof = empty.prove_absent(&MemberId::new(HIGHEST), &positioned_device_key("probe", 7)).unwrap();
    assert!(proof.vacancy().unwrap().siblings().is_empty());
    assert_eq!(verifies(&empty, &proof, &MemberId::new(HIGHEST), &positioned_device_key("probe", 7)), Ok(()));
}

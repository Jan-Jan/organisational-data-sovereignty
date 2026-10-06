mod common;

use common::{default_level_count, device_key, member_id, member_key};
use org_members::hasher::{Blake3Hasher, TrieHasher};
use org_members::proof::{AbsenceProof, ProofEnding};
use org_members::smt::DefaultHashes;
use org_members::trie::OrgTrie;
use org_members::types::{Handle, MemberId, MemberLeaf, Name, NodeHash, RootHash, Surname};
use org_members::OrgMembersError;

type TestTrie = OrgTrie<Blake3Hasher>;

fn leaf(name: &str, devices: &[&str]) -> MemberLeaf {
    MemberLeaf::new(
        member_id(name),
        Handle::parse(name).unwrap(),
        member_key(&format!("{name}-mk")),
        Name::parse("Test").unwrap(),
        Surname::parse("User").unwrap(),
        devices.iter().map(|seed| device_key(seed)).collect(),
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
    let changed = org()
        .delete_p2p_device(
            &member_id("alice"),
            &device_key("alice-d2"),
            member_key("alice-mk2"),
        )
        .unwrap();
    changed.recalculate().unwrap().0
}

/// verifies: LLR-4xz255, REQ-535jcd
#[test]
fn prove_absent_refuses_an_uncalculated_trie() {
    let pending = org().add_member(leaf("dave", &["dave-d1"])).unwrap();
    assert_eq!(
        pending
            .prove_absent(&member_id("zed"), &device_key("zed-d1"))
            .unwrap_err(),
        OrgMembersError::HashesNotCalculated
    );
}

/// verifies: LLR-4xz255, LLR-2dcnbp, REQ-535jcd
#[test]
fn prove_absent_refuses_a_device_the_member_holds() {
    assert_eq!(
        org()
            .prove_absent(&member_id("alice"), &device_key("alice-d1"))
            .unwrap_err(),
        OrgMembersError::DeviceStillHeld
    );
}

/// verifies: LLR-4xz255, REQ-535jcd
///
/// A key held by a different Member does not stop the proof (owner ruling
/// 2026-10-06; the never-again rule belongs to the unit holding the
/// revoked-device list).
#[test]
fn prove_absent_ignores_a_key_another_member_holds() {
    assert!(org()
        .prove_absent(&member_id("alice"), &device_key("bob-d1"))
        .is_ok());
}

/// verifies: LLR-25tpdp, REQ-yyuxh8
#[test]
fn proof_for_an_absent_member_carries_no_member_data() {
    let proof = org()
        .prove_absent(&member_id("zed"), &device_key("zed-d1"))
        .unwrap();
    assert_eq!(proof.ending(), &ProofEnding::Empty);
    assert_eq!(
        proof.siblings().len() + default_level_count(proof.default_map()),
        256
    );
}

/// verifies: LLR-25tpdp, REQ-yyuxh8
#[test]
fn proof_for_a_removed_device_carries_only_that_member() {
    let trie = org_without_alice_d2();
    let proof = trie
        .prove_absent(&member_id("alice"), &device_key("alice-d2"))
        .unwrap();
    match proof.ending() {
        ProofEnding::Leaf(member) => {
            assert_eq!(member.id(), &member_id("alice"));
            assert_eq!(member, &trie.get(&member_id("alice")).unwrap());
        }
        ProofEnding::Empty => panic!("expected alice's leaf"),
    }
    assert_eq!(
        proof.siblings().len() + default_level_count(proof.default_map()),
        256
    );
}

/// verifies: LLR-utp6x4, LLR-dgzy7e, LLR-p2p8qy, REQ-tk2qqj
#[test]
fn proof_for_an_absent_member_verifies() {
    let trie = org();
    let root = trie.root_hash().unwrap();
    let proof = trie
        .prove_absent(&member_id("zed"), &device_key("zed-d1"))
        .unwrap();
    assert_eq!(
        proof.verify::<Blake3Hasher>(&root, &member_id("zed"), &device_key("zed-d1")),
        Ok(())
    );
}

/// verifies: LLR-utp6x4, LLR-dgzy7e, LLR-p2p8qy, REQ-tk2qqj
///
/// A deleted member's path ends in an empty leaf at depth 256, not an empty
/// subtree higher up: the walk must handle both.
#[test]
fn proof_for_a_deleted_member_verifies() {
    let trie = org()
        .delete_member(&member_id("bob"))
        .unwrap()
        .recalculate()
        .unwrap()
        .0;
    let root = trie.root_hash().unwrap();
    let proof = trie
        .prove_absent(&member_id("bob"), &device_key("bob-d1"))
        .unwrap();
    assert_eq!(proof.ending(), &ProofEnding::Empty);
    assert_eq!(
        proof.verify::<Blake3Hasher>(&root, &member_id("bob"), &device_key("bob-d1")),
        Ok(())
    );
}

/// verifies: LLR-utp6x4, LLR-dgzy7e, LLR-p2p8qy, REQ-tk2qqj
#[test]
fn proof_for_a_removed_device_verifies() {
    let trie = org_without_alice_d2();
    let root = trie.root_hash().unwrap();
    let proof = trie
        .prove_absent(&member_id("alice"), &device_key("alice-d2"))
        .unwrap();
    assert_eq!(
        proof.verify::<Blake3Hasher>(&root, &member_id("alice"), &device_key("alice-d2")),
        Ok(())
    );
}

/// verifies: LLR-dgzy7e, LLR-2dcnbp, REQ-tk2qqj, RC-j2znx8
///
/// A proof genuine under the new root fails against the old one: the
/// stale-root case org-io must prevent (HAZ-adm7gv) is at least never
/// accepted across roots.
#[test]
fn proof_is_refused_against_another_root() {
    let old_root = org().root_hash().unwrap();
    let trie = org_without_alice_d2();
    let proof = trie
        .prove_absent(&member_id("alice"), &device_key("alice-d2"))
        .unwrap();
    assert_eq!(
        proof.verify::<Blake3Hasher>(&old_root, &member_id("alice"), &device_key("alice-d2")),
        Err(OrgMembersError::AbsenceProofRootMismatch)
    );
}

/// verifies: LLR-dgzy7e, LLR-2dcnbp, REQ-tk2qqj
#[test]
fn proof_is_refused_for_another_member_id() {
    let trie = org();
    let root = trie.root_hash().unwrap();
    let proof = trie
        .prove_absent(&member_id("zed"), &device_key("zed-d1"))
        .unwrap();
    assert_eq!(
        proof.verify::<Blake3Hasher>(&root, &member_id("bob"), &device_key("zed-d1")),
        Err(OrgMembersError::AbsenceProofRootMismatch)
    );
}

/// verifies: LLR-p2p8qy, LLR-2dcnbp, REQ-tk2qqj
///
/// The root matches, but the leaf holds the device asked about.
#[test]
fn proof_is_refused_when_the_member_holds_the_device() {
    let trie = org();
    let root = trie.root_hash().unwrap();
    let proof = trie
        .prove_absent(&member_id("alice"), &device_key("alice-d9"))
        .unwrap();
    assert_eq!(
        proof.verify::<Blake3Hasher>(&root, &member_id("alice"), &device_key("alice-d1")),
        Err(OrgMembersError::DeviceStillHeld)
    );
}

/// verifies: LLR-4rju5r, LLR-2dcnbp
#[test]
fn from_parts_refuses_a_count_that_does_not_match_the_map() {
    let all_default = [0xffu8; 32];
    assert_eq!(
        AbsenceProof::from_parts(
            all_default,
            vec![NodeHash::new([7; 32])],
            ProofEnding::Empty
        )
        .unwrap_err(),
        OrgMembersError::AbsenceProofMalformed
    );
    assert_eq!(
        AbsenceProof::from_parts([0u8; 32], vec![], ProofEnding::Empty).unwrap_err(),
        OrgMembersError::AbsenceProofMalformed
    );
}

/// verifies: LLR-4rju5r, LLR-dgzy7e, REQ-tk2qqj
#[test]
fn a_tampered_sibling_is_refused_by_root() {
    let trie = org_without_alice_d2();
    let root = trie.root_hash().unwrap();
    let proof = trie
        .prove_absent(&member_id("alice"), &device_key("alice-d2"))
        .unwrap();
    let mut siblings = proof.siblings().to_vec();
    siblings[0] = NodeHash::new([0xab; 32]);
    let tampered =
        AbsenceProof::from_parts(*proof.default_map(), siblings, proof.ending().clone()).unwrap();
    assert_eq!(
        tampered.verify::<Blake3Hasher>(&root, &member_id("alice"), &device_key("alice-d2")),
        Err(OrgMembersError::AbsenceProofRootMismatch)
    );
}

/// verifies: LLR-4rju5r, REQ-tk2qqj
#[test]
fn a_proof_survives_the_wire_and_still_verifies() {
    let trie = org_without_alice_d2();
    let root = trie.root_hash().unwrap();
    let proof = trie
        .prove_absent(&member_id("alice"), &device_key("alice-d2"))
        .unwrap();
    let bytes = postcard::to_allocvec(&proof).unwrap();
    let back: AbsenceProof = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(back, proof);
    assert_eq!(
        back.verify::<Blake3Hasher>(&root, &member_id("alice"), &device_key("alice-d2")),
        Ok(())
    );
}

/// The wire shape, written independently of the crate, so a test can send
/// what the crate itself would never produce.
#[derive(serde::Serialize)]
struct RawProof {
    default_map: [u8; 32],
    siblings: Vec<[u8; 32]>,
    ending: Option<MemberLeaf>,
}

/// verifies: LLR-4rju5r, LLR-2dcnbp, REQ-ds8ryr
#[test]
fn the_wire_refuses_inconsistent_or_oversized_proofs() {
    let mismatch = RawProof {
        default_map: [0xff; 32],
        siblings: vec![[1; 32]],
        ending: None,
    };
    let err = postcard::from_bytes::<AbsenceProof>(&postcard::to_allocvec(&mismatch).unwrap())
        .unwrap_err();
    assert!(matches!(err, postcard::Error::SerdeDeCustom), "{err:?}");

    let oversized = RawProof {
        default_map: [0; 32],
        siblings: vec![[1; 32]; 257],
        ending: None,
    };
    assert!(
        postcard::from_bytes::<AbsenceProof>(&postcard::to_allocvec(&oversized).unwrap()).is_err()
    );
}

/// verifies: LLR-4rju5r, REQ-ds8ryr
///
/// The length prefix claims a million siblings but only 257 follow. Refusing
/// at the 257th shows the decoder stops at the bound; reading on would end in
/// `DeserializeUnexpectedEnd` instead.
#[test]
fn the_wire_stops_reading_siblings_at_the_bound() {
    let mut bytes = vec![0u8; 32];
    bytes.extend_from_slice(&[0xc0, 0x84, 0x3d]); // varint 1_000_000
    bytes.extend(std::iter::repeat_n(1u8, 257 * 32));
    let err = postcard::from_bytes::<AbsenceProof>(&bytes).unwrap_err();
    assert!(matches!(err, postcard::Error::SerdeDeCustom), "{err:?}");
}

/// verifies: LLR-4rju5r, REQ-tk2qqj
#[test]
fn an_empty_ending_proof_survives_the_wire_and_still_verifies() {
    let trie = org();
    let root = trie.root_hash().unwrap();
    let proof = trie
        .prove_absent(&member_id("zed"), &device_key("zed-d1"))
        .unwrap();
    assert_eq!(proof.ending(), &ProofEnding::Empty);
    let bytes = postcard::to_allocvec(&proof).unwrap();
    let back: AbsenceProof = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(back, proof);
    assert_eq!(
        back.verify::<Blake3Hasher>(&root, &member_id("zed"), &device_key("zed-d1")),
        Ok(())
    );
}

/// `bytes` with its last bit (bit 255, the leaf-level turn) flipped.
fn last_bit_flipped(mut bytes: [u8; 32]) -> [u8; 32] {
    bytes[31] ^= 1;
    bytes
}

/// A Member at a chosen MemberId, with its own handle, names and keys.
fn leaf_at(id: [u8; 32], handle: &str, name: &str, surname: &str, devices: &[&str]) -> MemberLeaf {
    MemberLeaf::new(
        MemberId::new(id),
        Handle::parse(handle).unwrap(),
        member_key(&format!("{handle}-mk")),
        Name::parse(name).unwrap(),
        Surname::parse(surname).unwrap(),
        devices.iter().map(|seed| device_key(seed)).collect(),
    )
    .unwrap()
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| window == needle)
}

/// Fails unless none of the neighbour's handle, name, surname, member key or
/// device key appears in `bytes`.
fn assert_no_neighbour_data(bytes: &[u8]) {
    let member_key = member_key("neighbour-mk");
    let device_key = device_key("neighbour-d1");
    for (what, needle) in [
        ("handle", b"neighbour".as_slice()),
        ("name", b"Zebulon".as_slice()),
        ("surname", b"Quixote".as_slice()),
        ("member key", member_key.as_bytes().as_slice()),
        ("device key", device_key.as_bytes().as_slice()),
    ] {
        assert!(!contains(bytes, needle), "proof bytes carry the neighbour's {what}");
    }
}

/// verifies: LLR-25tpdp, REQ-yyuxh8, RC-qa2758
///
/// Abnormal input: the neighbour shares all but the last bit of the path, so
/// its leaf is the level-0 sibling, in an organisation where several members
/// sit next to each other.
#[test]
fn a_proof_next_to_a_neighbour_carries_none_of_its_data() {
    let subject: [u8; 32] = blake3::hash(b"subject").into();
    let neighbour = last_bit_flipped(subject);
    let mut near = subject;
    near[31] ^= 2;
    let mut nearer = subject;
    nearer[31] ^= 3;
    let genesis = TestTrie::genesis(vec![
        leaf_at(subject, "subject", "Test", "User", &["subject-d1", "subject-d2"]),
        leaf_at(neighbour, "neighbour", "Zebulon", "Quixote", &["neighbour-d1"]),
        leaf_at(near, "near", "Test", "User", &["near-d1"]),
        leaf_at(nearer, "nearer", "Test", "User", &["nearer-d1"]),
    ])
    .unwrap();
    let id = MemberId::new(subject);

    // (a) an absent id whose level-0 sibling is the neighbour's leaf.
    let without_subject = genesis.delete_member(&id).unwrap().recalculate().unwrap().0;
    let absent = without_subject
        .prove_absent(&id, &device_key("subject-d1"))
        .unwrap();
    assert_no_neighbour_data(&postcard::to_allocvec(&absent).unwrap());
    assert_eq!(absent.ending(), &ProofEnding::Empty);
    assert_eq!(
        absent.verify::<Blake3Hasher>(
            &without_subject.root_hash().unwrap(),
            &id,
            &device_key("subject-d1")
        ),
        Ok(())
    );

    // (b) a removed device of the Member whose level-0 sibling is the neighbour.
    let removed = without_device(&genesis, subject, "subject-d2", "subject-mk2");
    let device_gone = removed
        .prove_absent(&id, &device_key("subject-d2"))
        .unwrap();
    assert_no_neighbour_data(&postcard::to_allocvec(&device_gone).unwrap());
    assert!(matches!(device_gone.ending(), ProofEnding::Leaf(_)));
    assert_eq!(
        device_gone.verify::<Blake3Hasher>(
            &removed.root_hash().unwrap(),
            &id,
            &device_key("subject-d2")
        ),
        Ok(())
    );
}

const LOWEST: [u8; 32] = [0u8; 32];
const HIGHEST: [u8; 32] = [0xffu8; 32];

/// Proves `device` absent under `id` in `trie` and verifies it against the root.
fn proves_and_verifies(trie: &TestTrie, id: [u8; 32], device: &str) {
    let id = MemberId::new(id);
    let proof = trie.prove_absent(&id, &device_key(device)).unwrap();
    assert_eq!(
        proof.verify::<Blake3Hasher>(&trie.root_hash().unwrap(), &id, &device_key(device)),
        Ok(()),
        "{id:?}"
    );
}

/// `trie` after `id`'s Member drops `device` and rotates to `new_key`.
fn without_device(trie: &TestTrie, id: [u8; 32], device: &str, new_key: &str) -> TestTrie {
    trie.delete_p2p_device(&MemberId::new(id), &device_key(device), member_key(new_key))
        .unwrap()
        .recalculate()
        .unwrap()
        .0
}

/// verifies: LLR-utp6x4
///
/// Boundary: the lowest and highest MemberIds (all bits 0, all bits 1), absent
/// from a multi-member organisation.
#[test]
fn proofs_at_the_lowest_and_highest_ids_verify_when_absent() {
    let trie = org();
    proves_and_verifies(&trie, LOWEST, "low-d1");
    proves_and_verifies(&trie, HIGHEST, "high-d1");
}

/// verifies: LLR-utp6x4
///
/// Boundary: Members at the lowest and highest MemberIds, each with a removed
/// Device key, in a multi-member organisation.
#[test]
fn proofs_at_the_lowest_and_highest_ids_verify_for_a_removed_device() {
    let trie = TestTrie::genesis(vec![
        leaf_at(LOWEST, "low", "Test", "User", &["low-d1", "low-d2"]),
        leaf_at(HIGHEST, "high", "Test", "User", &["high-d1", "high-d2"]),
        leaf("alice", &["alice-d1"]),
        leaf("bob", &["bob-d1"]),
    ])
    .unwrap();
    let trie = without_device(&trie, LOWEST, "low-d2", "low-mk2");
    let trie = without_device(&trie, HIGHEST, "high-d2", "high-mk2");
    proves_and_verifies(&trie, LOWEST, "low-d2");
    proves_and_verifies(&trie, HIGHEST, "high-d2");
}

/// verifies: LLR-utp6x4
///
/// Boundary: a one-member organisation, its Member at either extreme; the
/// other extreme is absent.
#[test]
fn proofs_in_a_one_member_organisation_verify() {
    for (present, absent, handle) in [(LOWEST, HIGHEST, "low"), (HIGHEST, LOWEST, "high")] {
        let d1 = format!("{handle}-d1");
        let d2 = format!("{handle}-d2");
        let trie =
            TestTrie::genesis(vec![leaf_at(present, handle, "Test", "User", &[&d1, &d2])])
                .unwrap();
        proves_and_verifies(&trie, absent, "other-d1");
        let trie = without_device(&trie, present, &d2, &format!("{handle}-mk2"));
        proves_and_verifies(&trie, present, &d2);
        proves_and_verifies(&trie, absent, "other-d1");
    }
}

/// The root of a one-Member organisation whose Member at `id` hashes to
/// `leaf_hash`: folded up `id`'s path with every sibling its level's default.
fn lone_member_root(leaf_hash: NodeHash, id: &MemberId) -> RootHash {
    let defaults = DefaultHashes::compute::<Blake3Hasher>();
    let root = (0..256u16).fold(leaf_hash, |computed, level| {
        let sibling = defaults.at_level(level).unwrap();
        if id.bit(255 - level).unwrap() {
            Blake3Hasher::hash_member_node(sibling, &computed)
        } else {
            Blake3Hasher::hash_member_node(&computed, sibling)
        }
    });
    RootHash::from(root)
}

/// verifies: LLR-25tpdp, LLR-utp6x4
///
/// The map's layout is bit `level % 8` of byte `level / 8`, level 0 the
/// leaf's; only non-default siblings travel, from the leaf's level up.
#[test]
fn a_proof_sends_only_non_default_siblings() {
    // (a) One Member at the lowest id; the highest id differs in the first
    // bit, so only the top-level sibling (level 255, the Member's subtree) is
    // not default.
    let lone =
        TestTrie::genesis(vec![leaf_at(LOWEST, "low", "Test", "User", &["low-d1"])]).unwrap();
    let proof = lone
        .prove_absent(&MemberId::new(HIGHEST), &device_key("high-d1"))
        .unwrap();
    let mut expected_map = [0xffu8; 32];
    expected_map[31] = 0x7f;
    assert_eq!(proof.default_map(), &expected_map);
    assert_eq!(proof.siblings().len(), 1);
    let defaults = DefaultHashes::compute::<Blake3Hasher>();
    assert_eq!(
        RootHash::from(Blake3Hasher::hash_member_node(
            &proof.siblings()[0],
            defaults.at_level(255).unwrap()
        )),
        lone.root_hash().unwrap()
    );

    // (b) Two Members differing only in the last bit: the removed-device
    // proof's one explicit sibling is the neighbour's leaf, at level 0.
    let subject: [u8; 32] = blake3::hash(b"subject").into();
    let neighbour = last_bit_flipped(subject);
    let neighbour_leaf = leaf_at(neighbour, "neighbour", "Zebulon", "Quixote", &["neighbour-d1"]);
    let pair = TestTrie::genesis(vec![
        leaf_at(subject, "subject", "Test", "User", &["subject-d1", "subject-d2"]),
        neighbour_leaf.clone(),
    ])
    .unwrap();
    let pair = without_device(&pair, subject, "subject-d2", "subject-mk2");
    let proof = pair
        .prove_absent(&MemberId::new(subject), &device_key("subject-d2"))
        .unwrap();
    let mut expected_map = [0xffu8; 32];
    expected_map[0] = 0xfe;
    assert_eq!(proof.default_map(), &expected_map);
    assert_eq!(proof.siblings().len(), 1);
    let neighbour_alone = TestTrie::genesis(vec![neighbour_leaf]).unwrap();
    assert_eq!(
        lone_member_root(proof.siblings()[0], &MemberId::new(neighbour)),
        neighbour_alone.root_hash().unwrap()
    );
}

/// verifies: LLR-4xz255, REQ-535jcd
///
/// alice's own path keeps hashed siblings after an uncalculated change, so
/// only the calculated-trie guard refuses.
#[test]
fn prove_absent_refuses_an_uncalculated_trie_even_on_a_hashed_path() {
    let pending = org()
        .delete_p2p_device(
            &member_id("alice"),
            &device_key("alice-d2"),
            member_key("alice-mk2"),
        )
        .unwrap();
    assert_eq!(
        pending
            .prove_absent(&member_id("alice"), &device_key("alice-d2"))
            .unwrap_err(),
        OrgMembersError::HashesNotCalculated
    );
}

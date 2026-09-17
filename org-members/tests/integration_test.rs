use ed25519_dalek::SigningKey;
use org_members::hasher::{Blake3Hasher, TrieHasher};
use org_members::trie::OrgTrie;
use org_members::types::{P2pDeviceKey, MemberId, NodeHash, P2pMemberKey, MemberLeaf, RootHash, MAX_DEVICES};
use org_members::OrgMembersError;

type TestTrie = OrgTrie<Blake3Hasher>;

/// Deterministically derives a MemberId from a seed string for test reproducibility.
fn member_id(seed: &str) -> MemberId {
    let hash: [u8; 32] = blake3::hash(seed.as_bytes()).into();
    MemberId::new(hash)
}

fn member_key(seed: &str) -> P2pMemberKey {
    let mut bytes = [0u8; 32];
    let hash: [u8; 32] = blake3::hash(seed.as_bytes()).into();
    bytes.copy_from_slice(&hash);
    P2pMemberKey::new(SigningKey::from_bytes(&bytes).verifying_key())
}

fn device_key(seed: &str) -> P2pDeviceKey {
    let mut bytes = [0u8; 32];
    let hash: [u8; 32] = blake3::hash(seed.as_bytes()).into();
    bytes.copy_from_slice(&hash);
    P2pDeviceKey::new(SigningKey::from_bytes(&bytes).verifying_key())
}

fn alice() -> MemberLeaf {
    MemberLeaf::new(
        member_id("alice-id"),
        "alice",
        member_key("alice-mk"),
        "Alice",
        "Smith",
        vec![device_key("alice-d1")])
    .unwrap()
}

fn bob() -> MemberLeaf {
    MemberLeaf::new(
        member_id("bob-id"),
        "bob",
        member_key("bob-mk"),
        "Bob",
        "Jones",
        vec![device_key("bob-d1")])
    .unwrap()
}

fn charlie() -> MemberLeaf {
    MemberLeaf::new(
        member_id("charlie-id"),
        "charlie",
        member_key("charlie-mk"),
        "Charlie",
        "Brown",
        vec![device_key("charlie-d1")])
    .unwrap()
}

fn jan_jan() -> MemberLeaf {
    MemberLeaf::new(
        member_id("jan-jan-id"),
        "jan-jan",
        member_key("jan-jan-mk"),
        "Jan-Jan",
        "Gödel",
        vec![device_key("jan-jan-d1"), device_key("jan-jan-d2")])
    .unwrap()
}

fn diana() -> MemberLeaf {
    MemberLeaf::new(
        member_id("diana-id"),
        "diana",
        member_key("diana-mk"),
        "Diana",
        "Prince",
        vec![device_key("diana-d1")])
    .unwrap()
}

// --- Genesis tests ---

#[test]
fn genesis_single_member() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    assert_eq!(trie.member_count(), 1);
    assert!(trie.is_calculated());
    assert!(trie.contains(&member_id("alice-id")));
    assert!(!trie.contains(&member_id("bob-id")));
}

/// verifies: REQ-m8aexh, REQ-kmvc96, LLR-ch2pkw, LLR-5w2jx8
///
/// The normal case for both: distinct, non-confusable handles are accepted.
/// Without it those two requirements would be evidenced only by rejections.
///
/// It is LLR-5w2jx8's normal case too, added 2026-09-17: five handles that
/// render differently must receive five different skeletons. The item's other
/// three carriers are all refusals, so without this one the skeleton function
/// would be evidenced only by handles it rejects.
#[test]
fn genesis_multiple_members() {
    let trie = TestTrie::genesis(vec![alice(), bob(), charlie(), jan_jan(), diana()]).unwrap();
    assert_eq!(trie.member_count(), 5);
    assert!(trie.contains_handle("alice"));
    assert!(trie.contains_handle("bob"));
    assert!(trie.contains_handle("charlie"));
    assert!(trie.contains_handle("jan-jan"));
    assert!(trie.contains_handle("diana"));
}

/// verifies: REQ-crjxk8, LLR-ch2pkw
#[test]
fn genesis_duplicate_id_fails() {
    let err = TestTrie::genesis(vec![alice(), alice()]);
    assert_eq!(err.unwrap_err(), OrgMembersError::DuplicateId);
}

/// verifies: REQ-kmvc96, LLR-ch2pkw
#[test]
fn genesis_duplicate_handle_different_id_fails() {
    let m1 = alice();
    let m2 = MemberLeaf::new(
        member_id("different-id"),
        "alice", // same handle as m1
        member_key("different-mk"),
        "Alice2",
        "Different",
        vec![device_key("d")])
    .unwrap();
    let err = TestTrie::genesis(vec![m1, m2]);
    assert_eq!(err.unwrap_err(), OrgMembersError::DuplicateHandle);
}

/// verifies: LLR-wm5hpc
#[test]
fn genesis_empty_is_ok() {
    let trie = TestTrie::genesis(vec![]).unwrap();
    assert_eq!(trie.member_count(), 0);
    assert!(trie.is_calculated());
}

// --- Insert tests ---

// No `verifies:` annotation: this asserts member count, handle presence and
// delta shape, and nothing about the identifier — neither clause of REQ-crjxk8
// could break and make it fail.
#[test]
fn insert_adds_member() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let trie = trie.add_member(bob()).unwrap();
    assert!(!trie.is_calculated());
    assert_eq!(trie.member_count(), 2);
    assert!(trie.contains_handle("bob"));

    let (trie, delta) = trie.recalculate().unwrap();
    assert!(trie.is_calculated());
    assert_eq!(delta.upserted().len(), 1);
    assert!(delta.removed().is_empty());
}

/// verifies: REQ-crjxk8, LLR-fv75ec
#[test]
fn insert_duplicate_id_fails() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let err = trie.add_member(alice());
    assert_eq!(err.unwrap_err(), OrgMembersError::DuplicateId);
}

/// verifies: REQ-kmvc96, LLR-fv75ec
#[test]
fn insert_duplicate_handle_different_id_fails() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let imposter = MemberLeaf::new(
        member_id("imposter-id"),
        "alice",
        member_key("imposter-mk"),
        "I'm",
        "Alice",
        vec![device_key("imposter-d")])
    .unwrap();
    let err = trie.add_member(imposter);
    assert_eq!(err.unwrap_err(), OrgMembersError::DuplicateHandle);
}

// --- update_name_surname tests ---

/// verifies: LLR-g6arcs
#[test]
fn update_name_surname_changes_pii() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let root_before = trie.root_hash().unwrap();

    let trie = trie
        .update_name_surname(&member_id("alice-id"), "Alyx", "Wonderland")
        .unwrap();
    let (trie, _) = trie.recalculate().unwrap();

    let member = trie.get(&member_id("alice-id")).unwrap();
    assert_eq!(member.name(), "Alyx");
    assert_eq!(member.surname(), "Wonderland");
    // Other fields unchanged
    assert_eq!(member.handle(), "alice");
    assert_eq!(member.p2p_key(), &member_key("alice-mk"));
    assert_eq!(member.p2p_device_count(), 1);
    assert_ne!(trie.root_hash().unwrap(), root_before);
}

/// verifies: LLR-g6arcs
#[test]
fn update_name_surname_nfc_normalizes() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let trie = trie
        .update_name_surname(&member_id("alice-id"), "e\u{0301}ric", "X")
        .unwrap();
    let (trie, _) = trie.recalculate().unwrap();
    let member = trie.get(&member_id("alice-id")).unwrap();
    assert_eq!(member.name(), "\u{00E9}ric"); // NFC composed
}

/// verifies: LLR-v3jqau
#[test]
fn update_name_surname_nonexistent_fails() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let err = trie.update_name_surname(&member_id("ghost-id"), "X", "Y");
    assert_eq!(err.unwrap_err(), OrgMembersError::IdNotFound);
}

// --- update_handle tests ---

/// verifies: REQ-crjxk8, REQ-kmvc96, LLR-mmst86
///
/// The member id is unchanged by a rename, which is what makes a grant made
/// to the member survive the rename instead of following the handle.
#[test]
fn update_handle_renames_member() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let trie = trie.update_handle(&member_id("alice-id"), "alicia").unwrap();
    let (trie, _) = trie.recalculate().unwrap();

    assert!(!trie.contains_handle("alice"));
    assert!(trie.contains_handle("alicia"));
    assert!(trie.contains(&member_id("alice-id")));
    assert_eq!(trie.get_by_handle("alicia").unwrap().name(), "Alice");
}

/// verifies: LLR-v3jqau
#[test]
fn update_handle_nonexistent_fails() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let err = trie.update_handle(&member_id("ghost-id"), "newname");
    assert_eq!(err.unwrap_err(), OrgMembersError::IdNotFound);
}

/// verifies: REQ-h5ret5, LLR-mmst86
#[test]
fn update_handle_rejects_invalid() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let err = trie.update_handle(&member_id("alice-id"), "Alice"); // uppercase
    assert!(matches!(
        err.unwrap_err(),
        OrgMembersError::InvalidHandle(_)
    ));
}

/// verifies: REQ-kmvc96, LLR-mmst86
#[test]
fn update_handle_rejects_collision() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let err = trie.update_handle(&member_id("alice-id"), "bob");
    assert_eq!(err.unwrap_err(), OrgMembersError::DuplicateHandle);
}

// --- rotate_p2p_key tests ---

/// verifies: REQ-crjxk8, LLR-k89ahd
///
/// The identifier is unchanged by a key replacement, which is the other half of
/// REQ-crjxk8's independence claim; `update_handle_renames_member` covers
/// independence from the handle.
#[test]
fn rotate_p2p_key_changes_only_key() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let root_before = trie.root_hash().unwrap();
    let new_key = member_key("alice-rotated");

    let trie = trie
        .rotate_p2p_key(&member_id("alice-id"), new_key)
        .unwrap();
    let (trie, _) = trie.recalculate().unwrap();

    let member = trie.get(&member_id("alice-id")).unwrap();
    assert_eq!(member.p2p_key(), &new_key);
    // Other fields unchanged
    assert_eq!(member.handle(), "alice");
    assert_eq!(member.name(), "Alice");
    assert_eq!(member.p2p_device_count(), 1);
    assert_ne!(trie.root_hash().unwrap(), root_before);
}

/// verifies: LLR-k89ahd, LLR-v3jqau
#[test]
fn rotate_p2p_key_nonexistent_fails() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let err = trie.rotate_p2p_key(&member_id("ghost-id"), member_key("any"));
    assert_eq!(err.unwrap_err(), OrgMembersError::IdNotFound);
}

// --- add_p2p_device tests ---

/// verifies: REQ-xdx2c2, LLR-4phmjf
#[test]
fn add_p2p_device_adds_a_device() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let new_device = device_key("alice-d2");

    let trie = trie
        .add_p2p_device(&member_id("alice-id"), new_device)
        .unwrap();
    let (trie, _) = trie.recalculate().unwrap();

    let member = trie.get(&member_id("alice-id")).unwrap();
    assert_eq!(member.p2p_device_count(), 2);
    assert!(member.has_p2p_device(&device_key("alice-d1")));
    assert!(member.has_p2p_device(&new_device));
    // Key unchanged
    assert_eq!(member.p2p_key(), &member_key("alice-mk"));
}

/// verifies: REQ-xdx2c2, LLR-4phmjf
#[test]
fn add_p2p_device_rejects_duplicate() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let err = trie.add_p2p_device(&member_id("alice-id"), device_key("alice-d1"));
    assert_eq!(err.unwrap_err(), OrgMembersError::DuplicateDevice);
}

/// verifies: REQ-xdx2c2, LLR-pys2ek, LLR-4phmjf
///
/// The bound itself (MAX_DEVICES, 4 today) is design data, not part of the
/// requirement — see the note on REQ-xdx2c2 in the requirements ledger. This
/// test is the only place the number is currently exercised.
#[test]
fn add_p2p_device_rejects_when_full() {
    // alice starts with 1 device; add 3 more to fill (max 4)
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let trie = trie
        .add_p2p_device(&member_id("alice-id"), device_key("d2"))
        .unwrap();
    let trie = trie
        .add_p2p_device(&member_id("alice-id"), device_key("d3"))
        .unwrap();
    let trie = trie
        .add_p2p_device(&member_id("alice-id"), device_key("d4"))
        .unwrap();
    // 5th device should fail
    let err = trie.add_p2p_device(&member_id("alice-id"), device_key("d5"));
    assert_eq!(err.unwrap_err(), OrgMembersError::DeviceSlotsFull);
}

/// verifies: LLR-v3jqau
#[test]
fn add_p2p_device_nonexistent_member_fails() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let err = trie.add_p2p_device(&member_id("ghost-id"), device_key("d"));
    assert_eq!(err.unwrap_err(), OrgMembersError::IdNotFound);
}

// --- delete_p2p_device tests ---

/// verifies: REQ-ewdg2q, LLR-s97ywt
#[test]
fn delete_p2p_device_removes_and_rotates_key() {
    let trie = TestTrie::genesis(vec![jan_jan()]).unwrap();
    // jan-jan has 2 devices: d1 and d2
    let new_key = member_key("jan-rotated");

    let trie = trie
        .delete_p2p_device(
            &member_id("jan-jan-id"),
            &device_key("jan-jan-d1"),
            new_key,
        )
        .unwrap();
    let (trie, _) = trie.recalculate().unwrap();

    let member = trie.get(&member_id("jan-jan-id")).unwrap();
    assert_eq!(member.p2p_device_count(), 1);
    assert!(!member.has_p2p_device(&device_key("jan-jan-d1")));
    assert!(member.has_p2p_device(&device_key("jan-jan-d2")));
    assert_eq!(member.p2p_key(), &new_key);
}

/// verifies: REQ-r784fu, REQ-ewdg2q, LLR-s97ywt
#[test]
fn delete_p2p_device_last_device_isolates() {
    // alice has 1 device. Removing it leaves her in isolated state (0 devices).
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let new_key = member_key("alice-isolated");

    let trie = trie
        .delete_p2p_device(
            &member_id("alice-id"),
            &device_key("alice-d1"),
            new_key,
        )
        .unwrap();
    let (trie, _) = trie.recalculate().unwrap();

    let member = trie.get(&member_id("alice-id")).unwrap();
    assert_eq!(member.p2p_device_count(), 0);
    assert_eq!(member.p2p_key(), &new_key);
}

// Not annotated to REQ-ewdg2q: removing the key replacement from
// delete_p2p_device leaves this test passing, so it does not carry that
// requirement's behaviour.
/// verifies: LLR-s97ywt
///
/// It does carry LLR-s97ywt, annotated 2026-09-17, and specifically the "in
/// one operation" clause -- the item's abnormal-input side, which its two
/// other carriers (both successful deletions) do not reach. Swallowing
/// `remove_device`'s error so the key is rotated anyway reds this test with an
/// `Ok(OrgTrie { .. })` where `DeviceNotFound` was asserted: the key would
/// have been replaced without the device being removed, which is the
/// conjunction the item forbids.
#[test]
fn delete_p2p_device_unknown_device_fails() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let err = trie.delete_p2p_device(
        &member_id("alice-id"),
        &device_key("does-not-exist"),
        member_key("new"),
    );
    assert_eq!(err.unwrap_err(), OrgMembersError::DeviceNotFound);
}

/// verifies: LLR-v3jqau
#[test]
fn delete_p2p_device_nonexistent_member_fails() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let err = trie.delete_p2p_device(
        &member_id("ghost-id"),
        &device_key("alice-d1"),
        member_key("new"),
    );
    assert_eq!(err.unwrap_err(), OrgMembersError::IdNotFound);
}

// --- emergency_isolate_member tests ---

/// verifies: REQ-r784fu, LLR-w92psx
#[test]
fn emergency_isolate_member_removes_all_devices_and_rotates_key() {
    let trie = TestTrie::genesis(vec![jan_jan()]).unwrap();
    // jan-jan has 2 devices
    let new_key = member_key("jan-isolated");

    let trie = trie
        .emergency_isolate_member(&member_id("jan-jan-id"), new_key)
        .unwrap();
    let (trie, _) = trie.recalculate().unwrap();

    let member = trie.get(&member_id("jan-jan-id")).unwrap();
    assert_eq!(member.p2p_device_count(), 0);
    assert!(!member.has_p2p_device(&device_key("jan-jan-d1")));
    assert!(!member.has_p2p_device(&device_key("jan-jan-d2")));
    assert_eq!(member.p2p_key(), &new_key);
    // Other PII unchanged
    assert_eq!(member.handle(), "jan-jan");
    assert_eq!(member.surname(), "Gödel");
}

/// verifies: REQ-r784fu, LLR-w92psx
#[test]
fn emergency_isolate_member_keeps_member_in_trie() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let trie = trie
        .emergency_isolate_member(&member_id("alice-id"), member_key("new"))
        .unwrap();
    let (trie, _) = trie.recalculate().unwrap();

    assert_eq!(trie.member_count(), 2); // alice still there, just isolated
    assert!(trie.contains(&member_id("alice-id")));
    assert!(trie.contains_handle("alice"));
}

/// verifies: REQ-r784fu, LLR-w92psx
#[test]
fn emergency_isolate_member_then_readd_device_unisolates() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let trie = trie
        .emergency_isolate_member(&member_id("alice-id"), member_key("recovered"))
        .unwrap();
    // Now re-add a device
    let trie = trie
        .add_p2p_device(&member_id("alice-id"), device_key("alice-recovered-d"))
        .unwrap();
    let (trie, _) = trie.recalculate().unwrap();

    let member = trie.get(&member_id("alice-id")).unwrap();
    assert_eq!(member.p2p_device_count(), 1);
    assert_eq!(member.p2p_key(), &member_key("recovered"));
}

/// verifies: REQ-r784fu, LLR-w92psx, LLR-v3jqau
#[test]
fn emergency_isolate_member_nonexistent_fails() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let err = trie.emergency_isolate_member(&member_id("ghost-id"), member_key("any"));
    assert_eq!(err.unwrap_err(), OrgMembersError::IdNotFound);
}

// --- Delete tests ---

/// verifies: LLR-j4d38d
///
/// Added 2026-09-17. LLR-j4d38d has two halves and its other carrier,
/// `delete_member_frees_the_handle_for_reuse`, only reaches one: reuse is
/// gated by the *skeleton* index, so dropping
/// `new_handle_index.remove(existing.handle())` from `delete_by_id` leaves
/// that test green and reds this one at `!trie.contains_handle("alice")`.
/// Measured on 2026-09-17, both ways round. This is also the item's
/// abnormal-input side: a lookup of a handle no member holds.
#[test]
fn delete_removes_member() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let trie = trie.delete_member(&member_id("alice-id")).unwrap();
    let (trie, delta) = trie.recalculate().unwrap();

    assert_eq!(trie.member_count(), 1);
    assert!(!trie.contains_handle("alice"));
    assert!(trie.contains_handle("bob"));
    assert_eq!(delta.removed().len(), 1);
}

/// verifies: LLR-v3jqau
#[test]
fn delete_nonexistent_fails() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let err = trie.delete_member(&member_id("eve-id"));
    assert_eq!(err.unwrap_err(), OrgMembersError::IdNotFound);
}

/// verifies: LLR-j4d38d
///
/// Deleting a member removes their handle and skeleton from the indexes, so a
/// later member may take the handle. The doc comment on `delete_member` has
/// claimed this since the operation was written; nothing tested it.
#[test]
fn delete_member_frees_the_handle_for_reuse() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let after = trie.delete_member(alice().id()).unwrap();

    // A different member, same handle as the departed one.
    let successor = MemberLeaf::new(
        member_id("successor"),
        "alice",
        member_key("s"),
        "Alicia",
        "Brown",
        vec![device_key("s-dev")],
    )
    .unwrap();

    assert!(
        after.add_member(successor).is_ok(),
        "the departed member's handle must be available again"
    );
}

// --- Immutability tests ---

/// verifies: REQ-d3prca, LLR-tk4qxu
#[test]
fn insert_does_not_mutate_original() {
    let original = TestTrie::genesis(vec![alice()]).unwrap();
    let original_root = original.root_hash().unwrap();
    let _modified = original.add_member(bob()).unwrap();

    assert_eq!(original.member_count(), 1);
    assert_eq!(original.root_hash().unwrap(), original_root);
    assert!(!original.contains_handle("bob"));
}

/// verifies: REQ-d3prca, LLR-tk4qxu
#[test]
fn delete_does_not_mutate_original() {
    let original = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let original_root = original.root_hash().unwrap();
    let _modified = original.delete_member(&member_id("alice-id")).unwrap();

    assert_eq!(original.member_count(), 2);
    assert_eq!(original.root_hash().unwrap(), original_root);
    assert!(original.contains_handle("alice"));
}

// --- Delta and CandidateTrie tests ---

/// verifies: REQ-4umsuz, LLR-y38jfk, LLR-au8het, LLR-7tdqv9, LLR-juxk9q
///
/// The whole-pipeline normal case. Three further items were annotated onto it
/// on 2026-09-17 because each was carried by refusals alone, and each of the
/// three executes here and reds under its own mutation:
///
/// - LLR-au8het -- the change set is anchored to the record it was computed
///   against. Inverting `apply_delta`'s base-root comparison reds this test
///   with `DeltaBaseMismatch`.
/// - LLR-7tdqv9 -- verification accepts a candidate whose root matches.
///   Inverting `verify_against`'s comparison reds it with
///   `VerificationFailed`.
/// - LLR-juxk9q -- the upserted record (charlie) is checked for handle
///   uniqueness and confusability at apply time. Moving the index insert above
///   the check reds it with `DuplicateHandle`, the upsert colliding with the
///   entry the check itself just wrote.
///
/// On LLR-y38jfk it carries the second clause only. "A candidate exposes no member query" is a
/// type-level guarantee -- `CandidateTrie` declares exactly `root_hash()` and
/// `verify_against()` -- and the plan's named mutation (have `apply_delta`
/// return the trie directly) is a signature change: T7 measured it as seven
/// compile errors across this file, not a discriminating red. What this test
/// does assert is that only `verify_against` yields a usable record, and it
/// reds when `verify_against` hands back a record built with an empty
/// handle index.
#[test]
fn delta_apply_and_verify() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();

    let updated = trie.add_member(charlie()).unwrap();
    let updated = updated.delete_member(&member_id("alice-id")).unwrap();
    let (updated, delta) = updated.recalculate().unwrap();

    let candidate = trie.apply_delta(&delta).unwrap();
    let verified = candidate.verify_against(&updated.root_hash().unwrap()).unwrap();

    assert_eq!(verified.root_hash().unwrap(), updated.root_hash().unwrap());
    assert_eq!(verified.member_count(), 2);
    assert!(!verified.contains_handle("alice"));
    assert!(verified.contains_handle("bob"));
    assert!(verified.contains_handle("charlie"));
}

/// verifies: REQ-4umsuz, LLR-au8het
#[test]
fn delta_base_mismatch_fails() {
    let parity_trie = TestTrie::genesis(vec![alice()]).unwrap();
    let other_trie = TestTrie::genesis(vec![bob()]).unwrap();

    let modified = parity_trie.add_member(charlie()).unwrap();
    let (_, delta) = modified.recalculate().unwrap();

    let err = other_trie.apply_delta(&delta);
    assert_eq!(err.unwrap_err(), OrgMembersError::DeltaBaseMismatch);
}

/// verifies: REQ-4umsuz, LLR-7tdqv9, LLR-y38jfk
///
/// LLR-y38jfk added 2026-09-17: this is that item's abnormal-input side --
/// only verification against the *expected* root yields a usable record, so a
/// verification against any other root must yield none. Deleting the guard in
/// `verify_against` reds it with an `Ok(OrgTrie { .. })` where an error was
/// asserted.
///
/// On LLR-7tdqv9 it carries the refusal clause. "The candidate is consumed either way" is a
/// type-level guarantee -- `verify_against` takes `self` by value, so a test
/// that reused the candidate afterwards would not compile, and there is no
/// mutation of the running code that can observe it.
#[test]
fn candidate_verify_wrong_root_fails() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let modified = trie.add_member(bob()).unwrap();
    let (_, delta) = modified.recalculate().unwrap();

    let candidate = trie.apply_delta(&delta).unwrap();
    let wrong_root = RootHash::from_bytes([0xFF; 32]);
    let err = candidate.verify_against(&wrong_root);
    assert_eq!(err.unwrap_err(), OrgMembersError::VerificationFailed);
}

// --- calculate_delta tests (long-offline catch-up) ---

/// verifies: LLR-h7stq2
///
/// Clause: the change set transforms `old` into the receiver. Reds when
/// `calculate_delta` reverses the diff direction -- the delta then removes an
/// id `old` does not hold and `apply_delta` returns
/// `MalformedDelta("removed id not present in trie")`.
#[test]
fn calculate_delta_then_apply_roundtrips() {
    // Member alice's view (the "old" trie) diverged from the latest org state.
    let old_trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();

    // Meanwhile the canonical trie advanced.
    let current = old_trie.add_member(charlie()).unwrap();
    let current = current.delete_member(&member_id("alice-id")).unwrap();
    let (current, _) = current.recalculate().unwrap();

    // Computing the trie delta from old → current produces the change set
    // needed to catch up. Applying it on the old trie yields the current root.
    let catchup_delta = current.calculate_delta(&old_trie).unwrap();

    let candidate = old_trie.apply_delta(&catchup_delta).unwrap();
    let verified = candidate.verify_against(&current.root_hash().unwrap()).unwrap();

    assert_eq!(verified.root_hash().unwrap(), current.root_hash().unwrap());
}

/// verifies: LLR-8jttpb
#[test]
fn calculate_delta_returns_removed_and_upserted_leaves() {
    let old_trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let new_trie = old_trie.add_member(charlie()).unwrap();
    let new_trie = new_trie.delete_member(&member_id("bob-id")).unwrap();
    let (new_trie, _) = new_trie.recalculate().unwrap();

    let delta = new_trie.calculate_delta(&old_trie).unwrap();

    assert_eq!(delta.base_root(), &old_trie.root_hash().unwrap());
    assert_eq!(delta.removed().len(), 1);
    assert_eq!(delta.removed()[0], member_id("bob-id"));
    assert_eq!(delta.upserted().len(), 1);
    assert_eq!(delta.upserted()[0].id(), &member_id("charlie-id"));
}

/// verifies: LLR-8jttpb
#[test]
fn calculate_delta_empty_when_tries_identical() {
    let trie_a = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let trie_b = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    assert_eq!(trie_a.root_hash().unwrap(), trie_b.root_hash().unwrap());

    let delta = trie_a.calculate_delta(&trie_b).unwrap();
    assert!(delta.is_empty());
    assert_eq!(delta.base_root(), &trie_b.root_hash().unwrap());
}

/// verifies: LLR-h7stq2
///
/// Clause: refused when either record has uncomputed hashes. Reds when the
/// `is_calculated()` guard is dropped -- `calculate_delta` then returns `Ok`.
/// It is the only carrier that mutation reds.
///
/// Deliberately carries no REQ-avmu3j annotation. It reads like REQ-avmu3j
/// coverage and is not: `calculate_delta` has its own is_calculated() guard, so
/// this test passes unchanged even with the root-reporting behaviour REQ-avmu3j
/// requires removed entirely. `root_hash_errs_until_recalculated` is the test
/// that fails under that mutation.
#[test]
fn calculate_delta_fails_when_hashes_not_calculated() {
    let trie_a = TestTrie::genesis(vec![alice()]).unwrap();
    let trie_b = trie_a.add_member(bob()).unwrap(); // pending mutations, not recalculated
    let err = trie_b.calculate_delta(&trie_a);
    assert_eq!(err.unwrap_err(), OrgMembersError::HashesNotCalculated);
}

/// verifies: LLR-h7stq2
///
/// Clause: the change set transforms `old` into the receiver. Reds when
/// `calculate_delta` reverses the diff direction -- `forward.removed().len()`
/// is then 1 rather than 0.
#[test]
fn calculate_delta_reversed_args_produces_inverse_delta() {
    // The convention is `new.calculate_delta(&old)`. If a caller flips the
    // arguments, they get the INVERSE delta (one that undoes the changes).
    // The base_root in each delta unambiguously identifies which trie it
    // applies to, so misapplication is always caught at apply time.
    let v1 = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let v2 = v1.add_member(charlie()).unwrap();
    let (v2, _) = v2.recalculate().unwrap();

    let forward = v2.calculate_delta(&v1).unwrap();
    let inverse = v1.calculate_delta(&v2).unwrap();

    // Forward: base = v1, adds charlie
    assert_eq!(forward.base_root(), &v1.root_hash().unwrap());
    assert_eq!(forward.removed().len(), 0);
    assert_eq!(forward.upserted().len(), 1);
    assert_eq!(forward.upserted()[0].id(), &member_id("charlie-id"));

    // Inverse: base = v2, removes charlie
    assert_eq!(inverse.base_root(), &v2.root_hash().unwrap());
    assert_eq!(inverse.removed().len(), 1);
    assert_eq!(inverse.removed()[0], member_id("charlie-id"));
    assert_eq!(inverse.upserted().len(), 0);

    // Forward applied to v1 yields v2.
    let cand = v1.apply_delta(&forward).unwrap();
    let after = cand.verify_against(&v2.root_hash().unwrap()).unwrap();
    assert_eq!(after.root_hash().unwrap(), v2.root_hash().unwrap());

    // Inverse applied to v2 yields v1 (round-trips back).
    let cand = v2.apply_delta(&inverse).unwrap();
    let back = cand.verify_against(&v1.root_hash().unwrap()).unwrap();
    assert_eq!(back.root_hash().unwrap(), v1.root_hash().unwrap());
}

/// verifies: LLR-h7stq2
///
/// Measured negative on the direction mutation: this test stays green when
/// `calculate_delta` reverses the diff, because it asserts only the
/// `DeltaBaseMismatch` guard and never inspects the delta's contents. What it
/// does carry is that `calculate_delta` anchors the delta to `old`: stamping
/// the receiver's root as `base_root` instead reds it, the wrong-side apply
/// then getting past the guard and failing with a canonical-form error.
#[test]
fn apply_delta_to_wrong_side_after_reversed_calc_fails() {
    // Following on from the reversed-args test: applying the forward delta to
    // v2 (the wrong side) and applying the inverse delta to v1 (the wrong
    // side) both fail with DeltaBaseMismatch -- the base_root catches the
    // user error.
    let v1 = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let v2 = v1.add_member(charlie()).unwrap();
    let (v2, _) = v2.recalculate().unwrap();

    let forward = v2.calculate_delta(&v1).unwrap(); // intended for v1
    let inverse = v1.calculate_delta(&v2).unwrap(); // intended for v2

    // forward applied to v2: forward.base_root = v1.root, v2.root != v1.root -> mismatch
    let err = v2.apply_delta(&forward);
    assert_eq!(err.unwrap_err(), OrgMembersError::DeltaBaseMismatch);

    // inverse applied to v1: inverse.base_root = v2.root, v1.root != v2.root -> mismatch
    let err = v1.apply_delta(&inverse);
    assert_eq!(err.unwrap_err(), OrgMembersError::DeltaBaseMismatch);
}

/// verifies: REQ-4umsuz, LLR-au8het
///
/// Carries both of LLR-au8het's clauses. The sanity assertion that
/// `delta.base_root()` is v1's root reds when `recalculate()` stamps the new
/// root into the delta instead of the base; the `DeltaBaseMismatch` assertion
/// reds when `apply_delta` drops the base comparison (the stale delta then
/// falls through to an unrelated canonical-form error).
#[test]
fn apply_delta_stale_delta_fails() {
    // Realistic scenario: trie evolves v1 -> v2 -> v3. A delta computed for
    // v1 -> v2 should NOT apply to v3 (the receiver already moved past it).
    // base_root of the delta == v1's root, but v3.root_hash() != v1.root_hash().
    let v1 = TestTrie::genesis(vec![alice()]).unwrap();
    let v2 = v1.add_member(bob()).unwrap();
    let (v2, delta_v1_to_v2) = v2.recalculate().unwrap();

    let v3 = v2.add_member(charlie()).unwrap();
    let (v3, _) = v3.recalculate().unwrap();

    // Sanity: v1 → v2 delta has base_root = v1.root_hash()
    assert_eq!(delta_v1_to_v2.base_root(), &v1.root_hash().unwrap());

    // Applying the v1→v2 delta to v3 must fail (stale: v3 has moved past v2)
    let err = v3.apply_delta(&delta_v1_to_v2);
    assert_eq!(err.unwrap_err(), OrgMembersError::DeltaBaseMismatch);
}

// --- Members iteration ---

#[test]
fn members_returns_all() {
    let trie = TestTrie::genesis(vec![alice(), bob(), charlie(), jan_jan(), diana()]).unwrap();
    let all = trie.members();
    assert_eq!(all.len(), 5);
}

// --- Handle validation tests ---

fn leaf_with_handle(handle: &str) -> Result<MemberLeaf, OrgMembersError> {
    MemberLeaf::new(
        member_id("k"),
        handle,
        member_key("k"),
        "A",
        "B",
        vec![device_key("d")])
}

/// verifies: REQ-h5ret5, LLR-xzqs9r
#[test]
fn handle_valid_ascii() {
    assert!(leaf_with_handle("alice").is_ok());
    assert!(leaf_with_handle("bob-jones").is_ok());
}

/// verifies: REQ-h5ret5, LLR-xzqs9r
#[test]
fn handle_empty_rejected() {
    assert!(leaf_with_handle("").is_err());
}

/// verifies: REQ-h5ret5, LLR-xzqs9r
#[test]
fn handle_dot_rejected() {
    assert!(leaf_with_handle("alice.bob").is_err());
}

/// verifies: REQ-h5ret5, LLR-xzqs9r
#[test]
fn handle_uppercase_rejected() {
    assert!(leaf_with_handle("Alice").is_err());
    assert!(leaf_with_handle("BOB").is_err());
}

/// verifies: REQ-h5ret5, LLR-xzqs9r
#[test]
fn handle_nfc_normalized() {
    let m1 = leaf_with_handle("e\u{0301}ric").unwrap();
    let m2 = leaf_with_handle("\u{00E9}ric").unwrap();
    assert_eq!(m1.handle(), m2.handle());
}

/// verifies: REQ-h5ret5, LLR-xzqs9r
#[test]
fn handle_mixed_script_rejected() {
    let mixed = "\u{0430}lice"; // Cyrillic а + Latin lice
    assert!(leaf_with_handle(mixed).is_err());
}

/// verifies: REQ-h5ret5, LLR-xzqs9r
#[test]
fn handle_single_script_unicode_ok() {
    assert!(leaf_with_handle("\u{0430}\u{043B}\u{0438}\u{0441}\u{0430}").is_ok());
}

/// verifies: REQ-h5ret5, LLR-xzqs9r
#[test]
fn handle_hyphen_allowed() {
    assert!(leaf_with_handle("jan-jan").is_ok());
}

/// verifies: LLR-xzqs9r
#[test]
fn handle_digits_allowed() {
    assert!(leaf_with_handle("alice42").is_ok());
}

/// verifies: REQ-h5ret5, LLR-xzqs9r
///
/// The 128-byte clause. The annotated proptest cannot reach it: its strategy
/// caps generated handles at 64 characters.
#[test]
fn handle_too_long_rejected() {
    // MAX_HANDLE_LEN is 128 bytes after NFC normalization. A 128-char ASCII
    // handle is at the cap (valid); 129 exceeds it and must be rejected with
    // InvalidHandle. This is the one validate_handle branch the fuzz strategy
    // (capped at 64 chars) could never reach.
    let at_cap = "a".repeat(128);
    assert!(leaf_with_handle(&at_cap).is_ok());

    let over_cap = "a".repeat(129);
    assert!(matches!(
        leaf_with_handle(&over_cap),
        Err(OrgMembersError::InvalidHandle(_))
    ));
}

// --- Confusable detection tests ---

/// verifies: REQ-m8aexh, LLR-5w2jx8, LLR-ch2pkw
#[test]
fn genesis_rejects_confusables() {
    // Find two handles whose UTS#39 skeletons match by probing candidates at runtime.
    // We can't hard-code a confusable pair safely because the skeleton table is
    // a library-controlled mapping that could shift. The test would silently pass
    // if our pair stopped being confusable. Probe instead, then assert rejection.
    let (h1, h2) = find_confusable_pair()
        .expect("test setup: no confusable pair found among candidates");

    let m1 = MemberLeaf::new(
        member_id("k1"),
        &h1,
        member_key("k1"),
        "A",
        "B",
        vec![device_key("d1")])
    .unwrap();
    let m2 = MemberLeaf::new(
        member_id("k2"),
        &h2,
        member_key("k2"),
        "A",
        "B",
        vec![device_key("d2")])
    .unwrap();
    let err = TestTrie::genesis(vec![m1, m2]).unwrap_err();
    assert_eq!(err, OrgMembersError::ConfusableHandle);
}

/// verifies: REQ-m8aexh, LLR-5w2jx8, LLR-fv75ec
#[test]
fn insert_rejects_confusable_handle() {
    let (h1, h2) =
        find_confusable_pair().expect("test setup: no confusable pair found among candidates");
    let m1 = MemberLeaf::new(
        member_id("k1"),
        &h1,
        member_key("k1"),
        "A",
        "B",
        vec![device_key("d1")])
    .unwrap();
    let m2 = MemberLeaf::new(
        member_id("k2"),
        &h2,
        member_key("k2"),
        "A",
        "B",
        vec![device_key("d2")])
    .unwrap();

    let trie = TestTrie::genesis(vec![m1]).unwrap();
    let err = trie.add_member(m2).unwrap_err();
    assert_eq!(err, OrgMembersError::ConfusableHandle);
}

/// verifies: REQ-m8aexh, LLR-5w2jx8, LLR-mmst86
#[test]
fn update_rejects_confusable_handle() {
    let (h1, h2) =
        find_confusable_pair().expect("test setup: no confusable pair found among candidates");
    // Two members, neither confusable initially.
    let m1 = MemberLeaf::new(
        member_id("k1"),
        &h1,
        member_key("k1"),
        "A",
        "B",
        vec![device_key("d1")])
    .unwrap();
    let m2 = alice();

    let trie = TestTrie::genesis(vec![m1, m2]).unwrap();

    // Now try to update alice's handle to a confusable of h1.
    let err = trie.update_handle(&member_id("alice-id"), &h2).unwrap_err();
    assert_eq!(err, OrgMembersError::ConfusableHandle);
}

/// Searches a small pool of candidate handles for a pair with matching UTS#39
/// skeletons, suitable for confusable-detection tests. Returns None if no
/// pair was found (in which case the test will skip / signal a setup issue).
fn find_confusable_pair() -> Option<(String, String)> {
    use org_members::types::handle_skeleton;
    // Candidates passing handle validation (lowercase, no '.', valid identifier chars).
    let candidates = [
        "paypal", "paypa1", "h0use", "house", "g00gle", "google", "ab1", "abl", "amaz0n", "amazon",
        "g0t", "got", "0lice", "olice", "01ice", "alice",
    ];
    for (i, &a) in candidates.iter().enumerate() {
        let sk_a = handle_skeleton(a);
        for &b in &candidates[i + 1..] {
            if a != b && sk_a == handle_skeleton(b) {
                return Some((a.to_string(), b.to_string()));
            }
        }
    }
    None
}

// --- MemberLeaf tests ---

#[test]
fn member_leaf_nfc_normalization() {
    let m1 = MemberLeaf::new(
        member_id("k"),
        "alice",
        member_key("k"),
        "e\u{0301}",
        "X",
        vec![device_key("d")])
    .unwrap();
    let m2 = MemberLeaf::new(
        member_id("k"),
        "alice",
        member_key("k"),
        "\u{00E9}",
        "X",
        vec![device_key("d")])
    .unwrap();
    assert_eq!(m1.name(), m2.name());
}

/// verifies: LLR-pys2ek
#[test]
fn member_leaf_too_many_devices() {
    let devices: Vec<_> = (0..5).map(|i| device_key(&format!("d{}", i))).collect();
    let err = MemberLeaf::new(
        member_id("k"),
        "alice",
        member_key("k"),
        "Alice",
        "Smith",
        devices);
    assert_eq!(err.unwrap_err(), OrgMembersError::DeviceSlotsFull);
}

/// verifies: LLR-paxj7b
#[test]
fn member_leaf_empty_devices() {
    let err = MemberLeaf::new(
        member_id("k"),
        "alice",
        member_key("k"),
        "Alice",
        "Smith",
        vec![]);
    assert_eq!(err.unwrap_err(), OrgMembersError::EmptyDeviceList);
}

/// verifies: LLR-4czn8t
#[test]
fn member_leaf_debug_redacts_pii() {
    let debug = format!("{:?}", jan_jan());
    assert!(debug.contains("[REDACTED]"));
    assert!(!debug.contains("Jan-Jan"));
    assert!(!debug.contains("Gödel"));
    assert!(!debug.contains("jan-jan"));
}

/// verifies: LLR-paxj7b
///
/// Added 2026-09-17. LLR-paxj7b's two other carriers are both refusals of the
/// zero-device record; this is the normal case the rejection is a boundary of
/// -- a record with the minimum device count is constructed and its fields
/// read back. Widening `MemberLeaf::new`'s guard from `is_empty()` to
/// `len() <= 1` reds it. Note the red arrives through the `alice()` helper's
/// `unwrap()` at the top of this file, not at an assertion below.
#[test]
fn member_leaf_has_id_handle_and_key() {
    let leaf = alice();
    assert_eq!(leaf.id(), &member_id("alice-id"));
    assert_eq!(leaf.handle(), "alice");
    assert_eq!(leaf.p2p_key(), &member_key("alice-mk"));
}

// --- Deterministic root hash ---

/// verifies: LLR-4n8zqx
#[test]
fn same_members_same_root_hash() {
    let trie1 = TestTrie::genesis(vec![alice(), bob(), charlie()]).unwrap();
    let trie2 = TestTrie::genesis(vec![alice(), bob(), charlie()]).unwrap();
    assert_eq!(trie1.root_hash().unwrap(), trie2.root_hash().unwrap());
}

/// verifies: LLR-4n8zqx
#[test]
fn different_insertion_order_same_root() {
    let trie_abc = TestTrie::genesis(vec![alice(), bob(), charlie()]).unwrap();
    let trie_cba = TestTrie::genesis(vec![charlie(), bob(), alice()]).unwrap();
    assert_eq!(trie_abc.root_hash().unwrap(), trie_cba.root_hash().unwrap());
}

// --- Multiple mutations before recalculate ---

/// verifies: LLR-n7nya3
#[test]
fn batch_mutations_then_recalculate() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();

    let trie = trie.add_member(bob()).unwrap();
    let trie = trie.add_member(charlie()).unwrap();
    let trie = trie.add_member(jan_jan()).unwrap();
    let trie = trie.delete_member(&member_id("alice-id")).unwrap();

    let (trie, delta) = trie.recalculate().unwrap();

    assert_eq!(trie.member_count(), 3);
    assert!(!trie.contains_handle("alice"));
    assert!(trie.contains_handle("bob"));
    assert!(trie.contains_handle("charlie"));
    assert!(trie.contains_handle("jan-jan"));

    assert_eq!(delta.removed().len(), 1);
    assert_eq!(delta.upserted().len(), 3);
}

// --- Jan-Jan specific tests ---

#[test]
fn jan_jan_has_two_devices() {
    let trie = TestTrie::genesis(vec![jan_jan()]).unwrap();
    let member = trie.get_by_handle("jan-jan").unwrap();
    assert_eq!(member.p2p_device_count(), 2);
    assert_eq!(member.name(), "Jan-Jan");
    assert_eq!(member.surname(), "Gödel");
}

#[test]
fn member_lookup_by_id() {
    let trie = TestTrie::genesis(vec![alice(), bob(), jan_jan()]).unwrap();

    let found = trie.get(&member_id("jan-jan-id")).unwrap();
    assert_eq!(found.name(), "Jan-Jan");

    let found = trie.get(&member_id("alice-id")).unwrap();
    assert_eq!(found.name(), "Alice");

    assert!(trie.get(&member_id("eve-id")).is_none());
}

// --- Lookup by handle ---

/// verifies: LLR-ub6dw9
#[test]
fn get_by_handle() {
    let trie = TestTrie::genesis(vec![alice(), bob(), jan_jan()]).unwrap();

    let found = trie.get_by_handle("jan-jan").unwrap();
    assert_eq!(found.name(), "Jan-Jan");

    let found = trie.get_by_handle("alice").unwrap();
    assert_eq!(found.name(), "Alice");

    assert!(trie.get_by_handle("eve").is_none());
}

/// verifies: LLR-ub6dw9
#[test]
fn contains_handle() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    assert!(trie.contains_handle("alice"));
    assert!(trie.contains_handle("bob"));
    assert!(!trie.contains_handle("charlie"));
}

// --- Pending changes (review before recalculate) ---

#[test]
fn pending_changes_empty_when_no_mutations() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let pending = trie.pending_changes().unwrap();
    assert!(pending.is_empty());
    assert!(!trie.has_pending_changes());
}

#[test]
fn pending_changes_reflects_mutations() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let trie = trie.add_member(charlie()).unwrap();
    let trie = trie.add_member(jan_jan()).unwrap();
    let trie = trie.delete_member(&member_id("alice-id")).unwrap();

    assert!(trie.has_pending_changes());

    let pending = trie.pending_changes().unwrap();
    assert_eq!(pending.removed().len(), 1);
    assert_eq!(pending.upserted().len(), 2);
    assert_eq!(
        pending.base_root(),
        &TestTrie::genesis(vec![alice(), bob()]).unwrap().root_hash().unwrap()
    );
}

#[test]
fn pending_changes_idempotent() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let trie = trie.add_member(bob()).unwrap();

    let pending1 = trie.pending_changes().unwrap();
    let pending2 = trie.pending_changes().unwrap();
    assert_eq!(pending1.upserted().len(), pending2.upserted().len());
    assert_eq!(pending1.removed().len(), pending2.removed().len());

    assert!(trie.has_pending_changes());
}

#[test]
fn pending_changes_matches_recalculate_delta() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let trie = trie.add_member(charlie()).unwrap();
    let trie = trie.delete_member(&member_id("alice-id")).unwrap();

    let preview = trie.pending_changes().unwrap();
    let (_, committed) = trie.recalculate().unwrap();

    assert_eq!(preview.removed().len(), committed.removed().len());
    assert_eq!(preview.upserted().len(), committed.upserted().len());
    assert_eq!(preview.base_root(), committed.base_root());
}

#[test]
fn pending_changes_after_recalculate_is_empty() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let trie = trie.add_member(bob()).unwrap();
    let (trie, _) = trie.recalculate().unwrap();

    assert!(!trie.has_pending_changes());
    assert!(trie.pending_changes().unwrap().is_empty());
}

// --- Key rotation through delta (I-1) ---

#[test]
fn member_key_rotation_through_delta() {
    // Peer A and Peer B start with the same trie.
    let starting_members = vec![alice(), bob()];
    let trie_a = TestTrie::genesis(starting_members.clone()).unwrap();
    let trie_b = TestTrie::genesis(starting_members).unwrap();
    assert_eq!(trie_a.root_hash().unwrap(), trie_b.root_hash().unwrap());

    // Peer A rotates alice's P2pMemberKey (handle and id unchanged).
    let trie_a = trie_a
        .rotate_p2p_key(&member_id("alice-id"), member_key("alice-rotated"))
        .unwrap();
    let (trie_a, delta) = trie_a.recalculate().unwrap();

    // Peer B applies the delta and verifies.
    let candidate = trie_b.apply_delta(&delta).unwrap();
    let trie_b = candidate.verify_against(&trie_a.root_hash().unwrap()).unwrap();

    // Both peers see the new key.
    let on_a = trie_a.get(&member_id("alice-id")).unwrap();
    let on_b = trie_b.get(&member_id("alice-id")).unwrap();
    assert_eq!(on_a.p2p_key(), &member_key("alice-rotated"));
    assert_eq!(on_b.p2p_key(), &member_key("alice-rotated"));
    assert_eq!(trie_a.root_hash().unwrap(), trie_b.root_hash().unwrap());
}

// --- Adversarial apply_delta (I-2) ---

/// verifies: LLR-juxk9q
///
/// Reds when the skeleton/uniqueness block in `apply_delta` is skipped:
/// `apply_delta` then returns `Ok` and `unwrap_err()` panics.
///
/// Measured negative on the other half of the item. That same block returns
/// `DuplicateHandle` as well as `ConfusableHandle`, and removing the whole
/// block reds exactly one test in this file -- this one. Nothing asserts that
/// an upsert carrying a handle already held by another member is refused at
/// apply time, so the "handle uniqueness" clause of LLR-juxk9q is carried in
/// name only. The confusability clause is genuinely carried.
#[test]
fn apply_delta_rejects_confusable_in_upsert() {
    let (h1, h2) =
        find_confusable_pair().expect("test setup: no confusable pair found among candidates");

    // Receiver trie holds a member with handle h1.
    let m1 = MemberLeaf::new(
        member_id("k1"),
        &h1,
        member_key("k1"),
        "A",
        "B",
        vec![device_key("d1")])
    .unwrap();
    let trie = TestTrie::genesis(vec![m1]).unwrap();

    // Craft an adversarial delta whose base matches `trie` but adds a confusable.
    // Start from a real delta to get the right base_root, then swap the upserts.
    let (_, mut delta) = trie.add_member(bob()).unwrap().recalculate().unwrap();
    org_members::delta::test_support::delta_set_removed(&mut delta, Vec::new());
    org_members::delta::test_support::delta_set_upserted(
        &mut delta,
        vec![MemberLeaf::new(
            member_id("k2"),
            &h2,
            member_key("k2"),
            "A",
            "B",
            vec![device_key("d2")],
        )
        .unwrap()],
    );

    let err = trie.apply_delta(&delta).unwrap_err();
    assert_eq!(err, OrgMembersError::ConfusableHandle);
}

// --- Send + Sync (recommendation 4) ---

#[test]
fn orgtrie_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<OrgTrie<Blake3Hasher>>();
    assert_send_sync::<MemberLeaf>();
    assert_send_sync::<MemberId>();
    assert_send_sync::<P2pMemberKey>();
    assert_send_sync::<P2pDeviceKey>();
}

// --- Serde validation (C-1) ---

#[cfg(feature = "serde")]
/// verifies: REQ-shk82j, REQ-h5ret5, LLR-68tka5
#[test]
fn deserialize_rejects_invalid_handle() {
    use postcard::{from_bytes, to_allocvec};
    let valid = alice();
    let bytes = to_allocvec(&valid).unwrap();

    // Round-trip the valid case to confirm baseline.
    let _: MemberLeaf = from_bytes(&bytes).unwrap();

    // Build a wire payload with an uppercase handle directly via the same
    // serde shape, bypassing MemberLeaf::new.
    #[derive(serde::Serialize)]
    struct EvilLeaf<'a> {
        id: MemberId,
        handle: &'a str,
        p2p_key: P2pMemberKey,
        name: &'a str,
        surname: &'a str,
        p2p_devices: org_members::types::P2pDeviceSlots,
    }

    let p2p_devices = {
        let leaf = alice();
        let dev_bytes = to_allocvec(&leaf).unwrap();
        let leaf2: MemberLeaf = from_bytes(&dev_bytes).unwrap();
        org_members::types::P2pDeviceSlots::new(leaf2.p2p_devices().to_vec()).unwrap()
    };

    let evil = EvilLeaf {
        id: *valid.id(),
        handle: "Alice",  // Uppercase -- should be rejected on deserialize.
        p2p_key: *valid.p2p_key(),
        name: "A",
        surname: "B",
        p2p_devices,
    };
    let evil_bytes = to_allocvec(&evil).unwrap();
    let result: Result<MemberLeaf, _> = from_bytes(&evil_bytes);
    assert!(
        result.is_err(),
        "deserialize must reject MemberLeaf with uppercase handle"
    );
}

#[cfg(feature = "serde")]
/// verifies: LLR-xyv6p9
#[test]
fn deserialize_accepts_empty_device_list() {
    // Empty device list IS valid on the wire because emergency_isolate_member
    // produces a member with 0 devices, and that state must roundtrip through
    // delta sync. MemberLeaf::new still requires ≥1 for normal creation.
    use postcard::{from_bytes, to_allocvec};
    let empty_devices: Vec<P2pDeviceKey> = vec![];
    let bytes = to_allocvec(&empty_devices).unwrap();
    let result: Result<org_members::types::P2pDeviceSlots, _> = from_bytes(&bytes);
    assert!(
        result.is_ok(),
        "deserialize must accept empty device list (for isolated members)"
    );
    assert_eq!(result.unwrap().device_count(), 0);
}

#[cfg(feature = "serde")]
/// verifies: LLR-paxj7b
#[test]
fn member_leaf_new_rejects_empty_device_list() {
    let err = MemberLeaf::new(
        member_id("k"),
        "alice",
        member_key("k"),
        "A",
        "B",
        vec![],
    );
    assert_eq!(err.unwrap_err(), OrgMembersError::EmptyDeviceList);
}

// --- Error variant smoke tests (Task 1 of Hyperbridge fixes) ---

/// verifies: LLR-sa3ugj
#[test]
fn malformed_delta_error_displays_reason() {
    let err = OrgMembersError::MalformedDelta("test reason");
    assert_eq!(format!("{}", err), "malformed delta: test reason");
}

/// verifies: LLR-sa3ugj
#[test]
fn field_too_long_error_displays_field_and_max() {
    let err = OrgMembersError::FieldTooLong { field: "name", max: 128 };
    assert_eq!(format!("{}", err), "field too long: name exceeds 128 bytes after NFC normalization");
}

// --- H-3: name/surname length caps ---

/// verifies: LLR-w5nkbu
#[test]
fn member_leaf_new_rejects_oversized_name() {
    let long_name = "a".repeat(129);
    let err = MemberLeaf::new(
        member_id("k"),
        "alice",
        member_key("k"),
        &long_name,
        "B",
        vec![device_key("d")],
    );
    assert_eq!(
        err.unwrap_err(),
        OrgMembersError::FieldTooLong { field: "name", max: 128 }
    );
}

/// verifies: LLR-w5nkbu
#[test]
fn member_leaf_new_rejects_oversized_surname() {
    let long_surname = "b".repeat(129);
    let err = MemberLeaf::new(
        member_id("k"),
        "alice",
        member_key("k"),
        "A",
        &long_surname,
        vec![device_key("d")],
    );
    assert_eq!(
        err.unwrap_err(),
        OrgMembersError::FieldTooLong { field: "surname", max: 128 }
    );
}

/// verifies: LLR-w5nkbu
#[test]
fn member_leaf_new_accepts_max_length_name_and_surname() {
    let name_128 = "a".repeat(128);
    let surname_128 = "b".repeat(128);
    let ok = MemberLeaf::new(
        member_id("k"),
        "alice",
        member_key("k"),
        &name_128,
        &surname_128,
        vec![device_key("d")],
    );
    assert!(ok.is_ok());
}

#[cfg(feature = "serde")]
/// verifies: LLR-w5nkbu
#[test]
fn deserialize_rejects_oversized_name() {
    use postcard::{from_bytes, to_allocvec};
    use org_members::types::P2pDeviceSlots;

    #[derive(serde::Serialize)]
    struct WireLeaf<'a> {
        id: MemberId,
        handle: &'a str,
        p2p_key: P2pMemberKey,
        name: &'a str,
        surname: &'a str,
        p2p_devices: P2pDeviceSlots,
    }
    let long_name = "a".repeat(200);
    let wire = WireLeaf {
        id: member_id("k"),
        handle: "alice",
        p2p_key: member_key("k"),
        name: &long_name,
        surname: "B",
        p2p_devices: P2pDeviceSlots::new(vec![device_key("d")]).unwrap(),
    };
    let bytes = to_allocvec(&wire).unwrap();
    let result: Result<MemberLeaf, _> = from_bytes(&bytes);
    assert!(result.is_err());
}

#[cfg(feature = "serde")]
/// verifies: LLR-w5nkbu
#[test]
fn deserialize_rejects_oversized_surname() {
    use postcard::{from_bytes, to_allocvec};
    use org_members::types::P2pDeviceSlots;

    #[derive(serde::Serialize)]
    struct WireLeaf<'a> {
        id: MemberId,
        handle: &'a str,
        p2p_key: P2pMemberKey,
        name: &'a str,
        surname: &'a str,
        p2p_devices: P2pDeviceSlots,
    }
    let long_surname = "b".repeat(200);
    let wire = WireLeaf {
        id: member_id("k"),
        handle: "alice",
        p2p_key: member_key("k"),
        name: "A",
        surname: &long_surname,
        p2p_devices: P2pDeviceSlots::new(vec![device_key("d")]).unwrap(),
    };
    let bytes = to_allocvec(&wire).unwrap();
    let result: Result<MemberLeaf, _> = from_bytes(&bytes);
    assert!(result.is_err());
}

/// verifies: LLR-w5nkbu, LLR-g6arcs
#[test]
fn update_name_surname_rejects_oversized_name() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let long_name = "a".repeat(129);
    let err = trie.update_name_surname(&member_id("alice-id"), &long_name, "Smith");
    assert_eq!(
        err.unwrap_err(),
        OrgMembersError::FieldTooLong { field: "name", max: 128 }
    );
}

/// verifies: LLR-w5nkbu, LLR-g6arcs
#[test]
fn update_name_surname_rejects_oversized_surname() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let long_surname = "b".repeat(129);
    let err = trie.update_name_surname(&member_id("alice-id"), "Alice", &long_surname);
    assert_eq!(
        err.unwrap_err(),
        OrgMembersError::FieldTooLong { field: "surname", max: 128 }
    );
}

// --- H-2: P2pDeviceSlots deserialize rejects non-canonical wire form ---

#[cfg(feature = "serde")]
/// verifies: REQ-shk82j, LLR-xyv6p9
#[test]
fn deserialize_rejects_unsorted_devices() {
    use postcard::{from_bytes, to_allocvec};
    let d1 = device_key("d1");
    let d2 = device_key("d2");
    let (lo, hi) = if d1.as_bytes() < d2.as_bytes() { (d1, d2) } else { (d2, d1) };
    let unsorted_wire: Vec<P2pDeviceKey> = vec![hi, lo];
    let bytes = to_allocvec(&unsorted_wire).unwrap();
    let result: Result<org_members::types::P2pDeviceSlots, _> = from_bytes(&bytes);
    assert!(result.is_err(), "deserialize must reject unsorted device list");
}

#[cfg(feature = "serde")]
/// verifies: REQ-shk82j, LLR-xyv6p9
#[test]
fn deserialize_rejects_duplicate_devices() {
    use postcard::{from_bytes, to_allocvec};
    let d = device_key("d1");
    let dup_wire: Vec<P2pDeviceKey> = vec![d, d];
    let bytes = to_allocvec(&dup_wire).unwrap();
    let result: Result<org_members::types::P2pDeviceSlots, _> = from_bytes(&bytes);
    assert!(result.is_err(), "deserialize must reject duplicate devices");
}

#[cfg(feature = "serde")]
/// verifies: REQ-shk82j, REQ-xdx2c2, LLR-pys2ek, LLR-xyv6p9
#[test]
fn deserialize_rejects_too_many_devices() {
    use postcard::{from_bytes, to_allocvec};
    let many: Vec<P2pDeviceKey> = (0..5).map(|i| device_key(&format!("d{}", i))).collect();
    // Sort so we hit the count check, not the order check.
    let mut sorted = many.clone();
    sorted.sort();
    let bytes = to_allocvec(&sorted).unwrap();
    let result: Result<org_members::types::P2pDeviceSlots, _> = from_bytes(&bytes);
    assert!(result.is_err(), "deserialize must reject more than MAX_DEVICES");
}

#[cfg(feature = "serde")]
/// verifies: REQ-shk82j, LLR-xyv6p9
#[test]
fn deserialize_accepts_sorted_unique_devices() {
    use postcard::{from_bytes, to_allocvec};
    let d1 = device_key("d1");
    let d2 = device_key("d2");
    let (lo, hi) = if d1.as_bytes() < d2.as_bytes() { (d1, d2) } else { (d2, d1) };
    let canonical: Vec<P2pDeviceKey> = vec![lo, hi];
    let bytes = to_allocvec(&canonical).unwrap();
    let result: org_members::types::P2pDeviceSlots = from_bytes(&bytes).unwrap();
    assert_eq!(result.device_count(), 2);
}

// --- H-1: apply_delta rejects non-canonical Delta ---

/// verifies: LLR-xmpqn2
///
/// Clause: every removal is present in the record. Reds when the presence
/// check is dropped -- the error degrades to `InvariantViolated` raised inside
/// the apply loop, so the `MalformedDelta` assertion fails.
#[test]
fn apply_delta_rejects_stale_removal() {
    // After H-1: a removal of an id not present in the trie is MalformedDelta.
    // (Previously this was silently tolerated -- see commit history for the
    // prior test apply_delta_ignores_stale_removal which is now removed.)
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let ghost_id = member_id("ghost-id");
    let crafted = trie.delete_member(&member_id("alice-id")).unwrap();
    let (_target, mut delta) = crafted.recalculate().unwrap();
    let mut new_removed = delta.removed().to_vec();
    new_removed.push(ghost_id);
    new_removed.sort();
    org_members::delta::test_support::delta_set_removed(&mut delta, new_removed);

    let err = trie.apply_delta(&delta).unwrap_err();
    assert!(matches!(err, OrgMembersError::MalformedDelta(_)));
}

/// verifies: LLR-8jttpb, LLR-xmpqn2
///
/// For LLR-xmpqn2, clause: removals strictly increasing. Reds when the
/// `removed.windows(2)` check is dropped -- `apply_delta` then returns `Ok`.
///
/// For LLR-8jttpb:
/// T5 measured that neither `calculate_delta` carrier named in the plan reds
/// when `smt::diff_tries` descends right before left -- both hold a single
/// removal and a single upsert, so no ordering is observable. This test's
/// setup probe ("≥2 removals in decreasing order" after reversing) is the only
/// assertion in this file that the diff walk yields strictly increasing
/// identifiers, so LLR-8jttpb is annotated here as well. It reds as a setup
/// panic, not at the `MalformedDelta` assertion.
#[test]
fn apply_delta_rejects_unsorted_removed() {
    let trie = TestTrie::genesis(vec![alice(), bob(), charlie()]).unwrap();
    let modified = trie
        .delete_member(&member_id("alice-id")).unwrap()
        .delete_member(&member_id("bob-id")).unwrap();
    let (_target, mut delta) = modified.recalculate().unwrap();
    let mut rev = delta.removed().to_vec();
    rev.reverse();
    if rev.len() < 2 || rev[0] < rev[1] {
        panic!("test setup expected ≥2 removals in decreasing order");
    }
    org_members::delta::test_support::delta_set_removed(&mut delta, rev);

    let err = trie.apply_delta(&delta).unwrap_err();
    assert!(matches!(err, OrgMembersError::MalformedDelta(_)));
}

/// verifies: LLR-xmpqn2
///
/// Clause: removals strictly increasing (the duplicate case). Reds when the
/// `removed.windows(2)` check is dropped -- the second removal then hits an
/// absent id in the apply loop and the error becomes `InvariantViolated`,
/// failing the `MalformedDelta` assertion.
#[test]
fn apply_delta_rejects_duplicate_in_removed() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let modified = trie.delete_member(&member_id("alice-id")).unwrap();
    let (_target, mut delta) = modified.recalculate().unwrap();
    let one = delta.removed()[0];
    org_members::delta::test_support::delta_set_removed(&mut delta, vec![one, one]);

    let err = trie.apply_delta(&delta).unwrap_err();
    assert!(matches!(err, OrgMembersError::MalformedDelta(_)));
}

/// verifies: LLR-xmpqn2
///
/// Clause: upserts strictly increasing (the duplicate case). Reds when the
/// `upserted.windows(2)` check is dropped -- `apply_delta` then returns `Ok`.
#[test]
fn apply_delta_rejects_duplicate_in_upserted() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let modified = trie.add_member(bob()).unwrap();
    let (_target, mut delta) = modified.recalculate().unwrap();
    let one = delta.upserted()[0].clone();
    org_members::delta::test_support::delta_set_upserted(&mut delta, vec![one.clone(), one]);

    let err = trie.apply_delta(&delta).unwrap_err();
    assert!(matches!(err, OrgMembersError::MalformedDelta(_)));
}

/// verifies: LLR-xmpqn2
///
/// Clause: upserts strictly increasing. Reds when the `upserted.windows(2)`
/// check is dropped -- `apply_delta` then returns `Ok`.
#[test]
fn apply_delta_rejects_unsorted_upserted() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let modified = trie.add_member(bob()).unwrap().add_member(charlie()).unwrap();
    let (_target, mut delta) = modified.recalculate().unwrap();
    if delta.upserted().len() < 2 {
        panic!("test setup expected ≥2 upserts");
    }
    let mut rev = delta.upserted().to_vec();
    rev.reverse();
    org_members::delta::test_support::delta_set_upserted(&mut delta, rev);

    let err = trie.apply_delta(&delta).unwrap_err();
    assert!(matches!(err, OrgMembersError::MalformedDelta(_)));
}

/// verifies: LLR-xmpqn2
///
/// Clause: `removed` and `upserted` are disjoint. Reds when the two-pointer
/// merge's `Ordering::Equal` arm advances instead of rejecting -- `apply_delta`
/// then returns `Ok`. It is the only carrier this mutation reds.
#[test]
fn apply_delta_rejects_id_in_both_removed_and_upserted() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let modified = trie
        .rotate_p2p_key(&member_id("alice-id"), member_key("alice-rotated"))
        .unwrap();
    let (_target, mut delta) = modified.recalculate().unwrap();
    org_members::delta::test_support::delta_set_removed(&mut delta, vec![member_id("alice-id")]);

    let err = trie.apply_delta(&delta).unwrap_err();
    assert!(matches!(err, OrgMembersError::MalformedDelta(_)));
}

/// verifies: LLR-xmpqn2
///
/// Clause: every upsert observably changes the record. Reds when the
/// "identical to existing trie state" check is dropped -- `apply_delta` then
/// returns `Ok`. It is the only carrier this mutation reds.
#[test]
fn apply_delta_rejects_noop_upsert() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let modified = trie.add_member(charlie()).unwrap();
    let (_target, mut delta) = modified.recalculate().unwrap();
    let mut up = delta.upserted().to_vec();
    up.push(alice());
    up.sort_by(|a, b| a.id().cmp(b.id()));
    org_members::delta::test_support::delta_set_upserted(&mut delta, up);

    let err = trie.apply_delta(&delta).unwrap_err();
    assert!(matches!(err, OrgMembersError::MalformedDelta(_)));
}

/// verifies: LLR-xmpqn2
///
/// The acceptance half: a canonical delta must still be accepted. Measured
/// negative -- this test stays green under all five relaxing mutations
/// (drop increasing-removed, drop presence, drop increasing-upserted, drop
/// disjointness, drop no-op), because relaxing a check cannot break an honest
/// delta. It reds only under a polarity flip that makes the check
/// over-strict (`removed` presence test inverted to `is_some()`), and that
/// mutation is blunt: it reds four tests, of which this is one.
#[test]
fn apply_delta_canonical_delta_still_works() {
    // Sanity: the strict checks must not break honest round-trips.
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let updated = trie.add_member(charlie()).unwrap();
    let updated = updated.delete_member(&member_id("alice-id")).unwrap();
    let (updated, delta) = updated.recalculate().unwrap();

    let candidate = trie.apply_delta(&delta).unwrap();
    let verified = candidate.verify_against(&updated.root_hash().unwrap()).unwrap();
    assert_eq!(verified.root_hash().unwrap(), updated.root_hash().unwrap());
}

/// verifies: REQ-avmu3j, LLR-n7nya3
///
/// The abnormal case: a mutated trie must refuse to report a root at all,
/// rather than reporting the pre-mutation one. `root_hash()` is called by most
/// tests in this file, so the function was thoroughly EXECUTED before this
/// test existed -- but only ever on a calculated trie. Nothing asserted the
/// error, which is the half the requirement is about.
#[test]
fn root_hash_errs_until_recalculated() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let (trie, _) = trie.recalculate().unwrap();
    let published = trie.root_hash().unwrap();

    let mutated = trie.add_member(charlie()).unwrap();
    assert!(!mutated.is_calculated());
    assert_eq!(mutated.root_hash().unwrap_err(), OrgMembersError::HashesNotCalculated);

    // And the normal case: recalculation produces a root, and it is not the
    // one the pre-mutation trie published.
    let (mutated, _) = mutated.recalculate().unwrap();
    assert_ne!(mutated.root_hash().unwrap(), published);
}

// A `deserialize_revalidates_handle` test stood here briefly. It was removed:
// `deserialize_rejects_invalid_handle`, above, already asserted the same
// invariant and had done so since before the requirements work began. Writing
// a second one was a mistake made by trusting a doc-comment ("gated so a plain
// cargo test skips") over the file it describes; the annotation it was written
// to carry now sits on the existing test instead.

/// verifies: LLR-72p8bz
///
/// The four hash domains are separated: the same bytes hashed as a member leaf,
/// a member node, a device leaf and a device node yield four different values.
/// Without separation, a device key could be presented as a member leaf.
#[test]
fn hasher_domains_are_separated() {
    let input = [7u8; 32];
    let node = NodeHash::new(input);

    let member_leaf = Blake3Hasher::hash_member_leaf(&input);
    let device_leaf = Blake3Hasher::hash_device_leaf(&input);
    let member_node = Blake3Hasher::hash_member_node(&node, &node);
    let device_node = Blake3Hasher::hash_device_node(&node, &node);

    let all = [member_leaf, device_leaf, member_node, device_node];
    for i in 0..all.len() {
        for j in (i + 1)..all.len() {
            assert_ne!(all[i], all[j], "domains {i} and {j} collide");
        }
    }
}

/// verifies: LLR-kdhd2v
///
/// Device keys are held sorted, so the order they are supplied in does not
/// change the member record's contribution to the root.
#[test]
fn device_slot_order_does_not_change_the_root() {
    let d1 = device_key("dev-a");
    let d2 = device_key("dev-b");

    let forward = MemberLeaf::new(
        member_id("m"),
        "alice",
        member_key("k"),
        "Alice",
        "Anderson",
        vec![d1, d2],
    )
    .unwrap();
    let reverse = MemberLeaf::new(
        member_id("m"),
        "alice",
        member_key("k"),
        "Alice",
        "Anderson",
        vec![d2, d1],
    )
    .unwrap();

    let a = TestTrie::genesis(vec![forward])
        .unwrap()
        .recalculate()
        .unwrap()
        .0
        .root_hash()
        .unwrap();
    let b = TestTrie::genesis(vec![reverse])
        .unwrap()
        .recalculate()
        .unwrap()
        .0
        .root_hash()
        .unwrap();

    assert_eq!(a, b);
}

/// verifies: LLR-kdhd2v, LLR-pys2ek
///
/// Every one of the MAX_DEVICES slots reaches the root. `to_fixed_slots` is
/// sized by MAX_DEVICES while `compute_device_root` is sized by the literal 4;
/// nothing but this test couples them, and a device that does not reach the
/// root is invisible to every verifier.
///
/// LLR-pys2ek added 2026-09-17: this is the item's normal case, and it is
/// where its second clause -- MAX_DEVICES is 4 *because* the sub-trie has four
/// slots -- is actually asserted. Its other three carriers are all refusals of
/// a fifth device. Raising MAX_DEVICES to 5 reds this test at `device slot 4
/// does not reach the root`, which is precisely the coupling the clause
/// states.
///
/// Rewritten 2026-09-17 (independent review, finding-3): the first version
/// substituted one fixed key, `device_key("replacement")`, into each slot, and
/// the MAX_DEVICES 4 -> 5 red depended on that key's BLAKE3 digest happening to
/// sort after the fourth key of the base set. Changing that one seed string
/// left the mutation GREEN. The substitute is now the pool's largest key by
/// construction, so the red no longer turns on a digest coincidence and is
/// reproduced under any device-key seeds.
#[test]
fn every_device_slot_reaches_the_root() {
    // `P2pDeviceSlots` stores devices in sorted order, so which slot a key
    // lands in is decided by its digest, not by the order it is written here.
    // Sort a pool of MAX_DEVICES + 1 keys and take the first MAX_DEVICES as
    // the base: `base[i]` is then the key in slot `i`, by construction rather
    // than by coincidence. The spare is the pool's largest, so substituting it
    // for any base key leaves it in the LAST slot -- which is the only
    // substitution that can be invisible to a root that hashes fewer slots
    // than MAX_DEVICES. Picking an arbitrary replacement key instead makes the
    // detection depend on where that one digest happens to sort.
    let mut pool: Vec<P2pDeviceKey> =
        (0..=MAX_DEVICES).map(|i| device_key(&format!("dev-{i}"))).collect();
    pool.sort();
    let base: Vec<P2pDeviceKey> = pool[..MAX_DEVICES].to_vec();
    let spare = pool[MAX_DEVICES];

    let root_of = |devices: Vec<P2pDeviceKey>| {
        let leaf = MemberLeaf::new(
            member_id("m"),
            "alice",
            member_key("k"),
            "Alice",
            "Anderson",
            devices,
        )
        .unwrap();
        TestTrie::genesis(vec![leaf])
            .unwrap()
            .recalculate()
            .unwrap()
            .0
            .root_hash()
            .unwrap()
    };

    let all = root_of(base.clone());

    // Vary each slot in turn; every one must move the root.
    for i in 0..MAX_DEVICES {
        let mut varied = base.clone();
        varied[i] = spare;
        assert_ne!(all, root_of(varied), "device slot {i} does not reach the root");
    }
}

// --- SDD-d9svdj: addressing and the empty-subtree defaults ---

/// verifies: LLR-zbe553
///
/// Index 0 is the most significant bit of byte 0, and index 255 the least
/// significant bit of byte 31. The SMT traverses by this, so the convention is
/// the addressing scheme, not a detail.
#[test]
fn member_id_bit_indexes_msb_first() {
    let mut bytes = [0u8; 32];
    bytes[0] = 0b1000_0000;
    bytes[31] = 0b0000_0001;
    let id = MemberId::new(bytes);

    assert!(id.bit(0), "index 0 must be the MSB of byte 0");
    assert!(!id.bit(1));
    assert!(id.bit(255), "index 255 must be the LSB of byte 31");
    assert!(!id.bit(254));
}

/// verifies: LLR-wm5hpc, LLR-4n8zqx
///
/// Every level's empty-subtree hash is precomputed, so a trie emptied of its
/// members is indistinguishable from one that never held any.
///
/// LLR-4n8zqx added 2026-09-17: this is that item's boundary case, and the
/// only one that puts a *history* behind the member set rather than a
/// different insertion order. Its two other carriers compare two tries built
/// by insertion alone; here the two tries have the same (empty) member set and
/// different pasts. Making `smt::remove` leave a residue -- `Node::empty` at
/// the level-1 default instead of the empty-leaf default -- reds it, the root
/// of an empty organisation then depending on how it got there.
#[test]
fn add_then_delete_returns_to_the_empty_root() {
    let empty = TestTrie::genesis(vec![])
        .unwrap()
        .recalculate()
        .unwrap()
        .0
        .root_hash()
        .unwrap();

    let populated = TestTrie::genesis(vec![])
        .unwrap()
        .add_member(alice())
        .unwrap();
    let emptied = populated
        .delete_member(alice().id())
        .unwrap()
        .recalculate()
        .unwrap()
        .0
        .root_hash()
        .unwrap();

    assert_eq!(empty, emptied);
}

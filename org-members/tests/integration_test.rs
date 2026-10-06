mod common;

use common::{device_key, member_key};
use org_members::hasher::{Blake3Hasher, TrieHasher};
use org_members::trie::OrgTrie;
use org_members::types::{
    Handle, DevicePublicKey, MemberId, Name, NodeHash, PersonPublicKey, MemberLeaf, RootHash, Surname,
    MAX_DEVICES, MAX_NAME_LEN,
};
use org_members::OrgMembersError;
use person::DeviceTrieHasher;

type TestTrie = OrgTrie<Blake3Hasher>;

/// Deterministically derives a MemberId from a seed string for test reproducibility.
fn member_id(seed: &str) -> MemberId {
    let hash: [u8; 32] = blake3::hash(seed.as_bytes()).into();
    MemberId::new(hash)
}


fn h(s: &str) -> Handle {
    Handle::parse(s).unwrap()
}

fn nm(s: &str) -> Name {
    Name::parse(s).unwrap()
}

fn sn(s: &str) -> Surname {
    Surname::parse(s).unwrap()
}

fn alice() -> MemberLeaf {
    MemberLeaf::new(
        member_id("alice-id"),
        h("alice"),
        member_key("alice-mk"),
        nm("Alice"),
        sn("Smith"),
        vec![device_key("alice-d1")])
    .unwrap()
}

fn bob() -> MemberLeaf {
    MemberLeaf::new(
        member_id("bob-id"),
        h("bob"),
        member_key("bob-mk"),
        nm("Bob"),
        sn("Jones"),
        vec![device_key("bob-d1")])
    .unwrap()
}

fn charlie() -> MemberLeaf {
    MemberLeaf::new(
        member_id("charlie-id"),
        h("charlie"),
        member_key("charlie-mk"),
        nm("Charlie"),
        sn("Brown"),
        vec![device_key("charlie-d1")])
    .unwrap()
}

fn jan_jan() -> MemberLeaf {
    MemberLeaf::new(
        member_id("jan-jan-id"),
        h("jan-jan"),
        member_key("jan-jan-mk"),
        nm("Jan-Jan"),
        sn("Gödel"),
        vec![device_key("jan-jan-d1"), device_key("jan-jan-d2")])
    .unwrap()
}

fn diana() -> MemberLeaf {
    MemberLeaf::new(
        member_id("diana-id"),
        h("diana"),
        member_key("diana-mk"),
        nm("Diana"),
        sn("Prince"),
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
    assert!(trie.contains_handle(&h("alice")));
    assert!(trie.contains_handle(&h("bob")));
    assert!(trie.contains_handle(&h("charlie")));
    assert!(trie.contains_handle(&h("jan-jan")));
    assert!(trie.contains_handle(&h("diana")));
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
        h("alice"), // same handle as m1
        member_key("different-mk"),
        nm("Alice2"),
        sn("Different"),
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
    assert!(trie.contains_handle(&h("bob")));

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
        h("alice"),
        member_key("imposter-mk"),
        nm("I'm"),
        sn("Alice"),
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
        .update_name_surname(&member_id("alice-id"), nm("Alyx"), sn("Wonderland"))
        .unwrap();
    let (trie, _) = trie.recalculate().unwrap();

    let member = trie.get(&member_id("alice-id")).unwrap();
    assert_eq!(member.name().as_str(), "Alyx");
    assert_eq!(member.surname().as_str(), "Wonderland");
    // Other fields unchanged
    assert_eq!(member.handle().as_str(), "alice");
    assert_eq!(member.p2p_key(), &member_key("alice-mk"));
    assert_eq!(member.p2p_device_count(), 1);
    assert_ne!(trie.root_hash().unwrap(), root_before);
}

/// verifies: LLR-g6arcs
#[test]
fn update_name_surname_nfc_normalizes() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let trie = trie
        .update_name_surname(&member_id("alice-id"), nm("e\u{0301}ric"), sn("X"))
        .unwrap();
    let (trie, _) = trie.recalculate().unwrap();
    let member = trie.get(&member_id("alice-id")).unwrap();
    assert_eq!(member.name().as_str(), "\u{00E9}ric"); // NFC composed
}

/// verifies: LLR-g6arcs
///
/// Abnormal side: the inputs sit at the edges `Name` and `Surname` admit. The
/// name is exactly `MAX_NAME_LEN` bytes and is stored byte for byte; the
/// surname is parsed from decomposed (NFD) text and is stored in NFC form.
/// Only those two fields change: handle, p2p key and devices are untouched.
#[test]
fn update_name_surname_stores_boundary_and_nfd_fields_only() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let before = trie.get(&member_id("alice-id")).unwrap();
    let longest = "n".repeat(MAX_NAME_LEN);

    let trie = trie
        .update_name_surname(&member_id("alice-id"), nm(&longest), sn("Mu\u{0308}ller"))
        .unwrap();
    let (trie, _) = trie.recalculate().unwrap();

    let member = trie.get(&member_id("alice-id")).unwrap();
    assert_eq!(member.name().as_str(), longest);
    assert_eq!(member.name().as_str().len(), MAX_NAME_LEN);
    assert_eq!(member.surname().as_str(), "M\u{00fc}ller");
    assert_eq!(member.handle(), before.handle());
    assert_eq!(member.p2p_key(), before.p2p_key());
    assert_eq!(member.p2p_devices(), before.p2p_devices());
}

/// verifies: LLR-v3jqau
///
/// Abnormal side: `update_name_surname` on an id no member holds is refused
/// with `IdNotFound`, and the trie it was called on is left as it was.
#[test]
fn update_name_surname_unknown_id_leaves_trie_unchanged() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let root_before = trie.root_hash().unwrap();
    let err = trie
        .update_name_surname(&member_id("ghost-id"), nm("Ghost"), sn("Writer"))
        .unwrap_err();
    assert_eq!(err, OrgMembersError::IdNotFound);
    assert_eq!(trie.member_count(), 1);
    assert!(!trie.contains(&member_id("ghost-id")));
    assert_eq!(trie.root_hash().unwrap(), root_before);
    assert_eq!(trie.get(&member_id("alice-id")).unwrap().name().as_str(), "Alice");
}

/// verifies: LLR-v3jqau
#[test]
fn update_name_surname_nonexistent_fails() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let err = trie.update_name_surname(&member_id("ghost-id"), nm("X"), sn("Y"));
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
    let trie = trie.update_handle(&member_id("alice-id"), h("alicia")).unwrap();
    let (trie, _) = trie.recalculate().unwrap();

    assert!(!trie.contains_handle(&h("alice")));
    assert!(trie.contains_handle(&h("alicia")));
    assert!(trie.contains(&member_id("alice-id")));
    assert_eq!(trie.get_by_handle(&h("alicia")).unwrap().name().as_str(), "Alice");
}

/// verifies: LLR-ub6dw9
///
/// Abnormal side: a handle no member holds any more -- vacated by a rename,
/// then by a delete -- is not found by either lookup.
#[test]
fn handle_index_forgets_renamed_and_deleted_handles() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let trie = trie.update_handle(&member_id("alice-id"), h("alicia")).unwrap();

    assert!(trie.get_by_handle(&h("alice")).is_none());
    assert!(!trie.contains_handle(&h("alice")));
    assert_eq!(trie.get_by_handle(&h("alicia")).unwrap().id(), &member_id("alice-id"));
    assert!(trie.contains_handle(&h("alicia")));

    let trie = trie.delete_member(&member_id("alice-id")).unwrap();
    assert!(trie.get_by_handle(&h("alicia")).is_none());
    assert!(!trie.contains_handle(&h("alicia")));
    assert!(trie.get_by_handle(&h("bob")).is_some());
}

/// verifies: LLR-f3zrwd
///
/// Abnormal side: the handle index matches exactly. A valid handle that is
/// confusable with a held one is not that member's handle and is not found.
#[test]
fn handle_lookup_is_exact_not_confusable() {
    let (held, lookalike) =
        find_confusable_pair().expect("test setup: no confusable pair found among candidates");
    let trie = TestTrie::genesis(vec![leaf_with_handle(&held).unwrap()]).unwrap();

    assert!(trie.get_by_handle(&h(&held)).is_some());
    assert!(trie.get_by_handle(&h(&lookalike)).is_none(), "{lookalike:?} matched {held:?}");
    assert!(!trie.contains_handle(&h(&lookalike)));
}

/// verifies: LLR-v3jqau
#[test]
fn update_handle_nonexistent_fails() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let err = trie.update_handle(&member_id("ghost-id"), h("newname"));
    assert_eq!(err.unwrap_err(), OrgMembersError::IdNotFound);
}

/// verifies: REQ-h5ret5, LLR-xzqs9r
///
/// An invalid handle never reaches `update_handle`: it is refused when the
/// caller parses it, which is the only way to obtain a `Handle` to pass.
#[test]
fn handle_for_update_rejects_invalid() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let err = Handle::parse("Alice") // uppercase
        .and_then(|nh| trie.update_handle(&member_id("alice-id"), nh));
    assert!(matches!(
        err.unwrap_err(),
        OrgMembersError::InvalidHandle(_)
    ));
}

/// verifies: REQ-kmvc96, LLR-mmst86
#[test]
fn update_handle_rejects_collision() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let err = trie.update_handle(&member_id("alice-id"), h("bob"));
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
    assert_eq!(member.handle().as_str(), "alice");
    assert_eq!(member.name().as_str(), "Alice");
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

/// verifies: LLR-k89ahd
///
/// Owner ruling 2026-10-03: rotating to the exact current key is refused.
/// The refusal is atomic by construction (`&self` → `Result<Self, _>`);
/// the evidence is the error itself.
#[test]
fn rotate_p2p_key_rejects_unchanged_key() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let current = *trie.get(&member_id("alice-id")).unwrap().p2p_key();
    let err = trie.rotate_p2p_key(&member_id("alice-id"), current);
    assert_eq!(err.unwrap_err(), OrgMembersError::P2pKeyNotReplaced);
}

/// verifies: LLR-k89ahd, LLR-v3jqau
///
/// An absent member reports `IdNotFound` before any key comparison.
#[test]
fn rotate_p2p_key_nonexistent_with_any_key_reports_id() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    // A key that is some member's current key, and one that is nobody's: the
    // absent id decides the error either way.
    let held = *trie.get(&member_id("alice-id")).unwrap().p2p_key();
    for key in [held, member_key("ghost-rotated")] {
        let err = trie.rotate_p2p_key(&member_id("ghost-id"), key);
        assert_eq!(err.unwrap_err(), OrgMembersError::IdNotFound);
    }
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

/// verifies: REQ-ewdg2q, LLR-s97ywt
///
/// PR-zz4exm's reproducing test. Passing the member's current key back as
/// the replacement must be refused whole (owner decision 2026-10-03). The
/// evidence is the `Err(P2pKeyNotReplaced)` itself: the operation takes
/// `&self` and returns `Result<Self, _>`, so an `Err` carries no trie and
/// the refusal is atomic by construction — re-reading `trie` afterwards
/// could not fail and would prove nothing.
#[test]
fn delete_p2p_device_rejects_unchanged_key() {
    let trie = TestTrie::genesis(vec![jan_jan()]).unwrap();
    let current = *trie.get(&member_id("jan-jan-id")).unwrap().p2p_key();
    let err = trie.delete_p2p_device(
        &member_id("jan-jan-id"),
        &device_key("jan-jan-d1"),
        current,
    );
    assert_eq!(err.unwrap_err(), OrgMembersError::P2pKeyNotReplaced);
}

/// verifies: REQ-ewdg2q, LLR-s97ywt
///
/// The last-device path, which would otherwise isolate the member, must
/// refuse the same way. As above, the `Err(P2pKeyNotReplaced)` is the
/// evidence; atomicity holds by construction (`&self` -> `Result<Self, _>`).
#[test]
fn delete_p2p_device_last_device_rejects_unchanged_key() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let current = *trie.get(&member_id("alice-id")).unwrap().p2p_key();
    let err = trie.delete_p2p_device(
        &member_id("alice-id"),
        &device_key("alice-d1"),
        current,
    );
    assert_eq!(err.unwrap_err(), OrgMembersError::P2pKeyNotReplaced);
}

/// verifies: LLR-s97ywt
///
/// Check order: an unknown device with the current key reports
/// DeviceNotFound, not P2pKeyNotReplaced.
#[test]
fn delete_p2p_device_unknown_device_with_unchanged_key_reports_device() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let current = *trie.get(&member_id("alice-id")).unwrap().p2p_key();
    let err = trie.delete_p2p_device(
        &member_id("alice-id"),
        &device_key("does-not-exist"),
        current,
    );
    assert_eq!(err.unwrap_err(), OrgMembersError::DeviceNotFound);
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
    assert_eq!(member.handle().as_str(), "jan-jan");
    assert_eq!(member.surname().as_str(), "Gödel");
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
    assert!(trie.contains_handle(&h("alice")));
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

/// verifies: REQ-ewdg2q, LLR-w92psx
/// The `Err(P2pKeyNotReplaced)` is the evidence; the refusal is atomic by
/// construction (`&self` -> `Result<Self, _>`, the `Err` carries no trie).
#[test]
fn emergency_isolate_member_rejects_unchanged_key() {
    let trie = TestTrie::genesis(vec![jan_jan()]).unwrap();
    let current = *trie.get(&member_id("jan-jan-id")).unwrap().p2p_key();
    let err = trie.emergency_isolate_member(&member_id("jan-jan-id"), current);
    assert_eq!(err.unwrap_err(), OrgMembersError::P2pKeyNotReplaced);
}

/// verifies: LLR-w92psx, LLR-v3jqau
/// An absent member reports IdNotFound before any key comparison.
#[test]
fn emergency_isolate_member_nonexistent_with_any_key_reports_id() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    // alice's current key, offered for an absent member: there is no current
    // key to compare against, so the lookup must fail first.
    let alice_key = *trie.get(&member_id("alice-id")).unwrap().p2p_key();
    let err = trie.emergency_isolate_member(&member_id("ghost-id"), alice_key);
    assert_eq!(err.unwrap_err(), OrgMembersError::IdNotFound);
}

/// verifies: LLR-w92psx
/// An already-isolated member (zero devices) still refuses its current key,
/// and accepts a different one.
#[test]
fn emergency_isolate_member_already_isolated_rejects_unchanged_key() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let k1 = member_key("isolated-k1");
    let trie = trie
        .emergency_isolate_member(&member_id("alice-id"), k1)
        .unwrap();
    assert_eq!(trie.get(&member_id("alice-id")).unwrap().p2p_device_count(), 0);

    // The Err is the evidence; `trie` is untouched by construction.
    let err = trie.emergency_isolate_member(&member_id("alice-id"), k1);
    assert_eq!(err.unwrap_err(), OrgMembersError::P2pKeyNotReplaced);

    let k2 = member_key("isolated-k2");
    let trie = trie
        .emergency_isolate_member(&member_id("alice-id"), k2)
        .unwrap();
    let (trie, _) = trie.recalculate().unwrap();
    let member = trie.get(&member_id("alice-id")).unwrap();
    assert_eq!(member.p2p_key(), &k2);
    assert_eq!(member.p2p_device_count(), 0);
}

// --- Delete tests ---

/// verifies: LLR-j4d38d
///
/// Added 2026-09-17. LLR-j4d38d has two halves and its other carrier,
/// `delete_member_frees_the_handle_for_reuse`, only reaches one: reuse is
/// gated by the *skeleton* index, so dropping
/// `new_handle_index.remove(existing.handle())` from `delete_by_id` leaves
/// that test green and reds this one at `!trie.contains_handle(&h("alice"))`.
/// Measured on 2026-09-17, both ways round. This is also the item's
/// abnormal-input side: a lookup of a handle no member holds.
#[test]
fn delete_removes_member() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let trie = trie.delete_member(&member_id("alice-id")).unwrap();
    let (trie, delta) = trie.recalculate().unwrap();

    assert_eq!(trie.member_count(), 1);
    assert!(!trie.contains_handle(&h("alice")));
    assert!(trie.contains_handle(&h("bob")));
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
        h("alice"),
        member_key("s"),
        nm("Alicia"),
        sn("Brown"),
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
    assert!(!original.contains_handle(&h("bob")));
}

/// verifies: REQ-d3prca, LLR-tk4qxu
#[test]
fn delete_does_not_mutate_original() {
    let original = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let original_root = original.root_hash().unwrap();
    let _modified = original.delete_member(&member_id("alice-id")).unwrap();

    assert_eq!(original.member_count(), 2);
    assert_eq!(original.root_hash().unwrap(), original_root);
    assert!(original.contains_handle(&h("alice")));
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
    assert!(!verified.contains_handle(&h("alice")));
    assert!(verified.contains_handle(&h("bob")));
    assert!(verified.contains_handle(&h("charlie")));
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
    let wrong_root = RootHash::new([0xFF; 32]);
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
        Handle::parse(handle)?,
        member_key("k"),
        nm("A"),
        sn("B"),
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
    // InvalidHandle. This is the one Handle::parse branch the fuzz strategy
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
    // Pick two valid handles whose UTS#39 skeletons match, by computing the
    // skeletons directly with unicode_security rather than asking the trie.
    // We can't hard-code a confusable pair safely because the skeleton table is
    // a library-controlled mapping that could shift; the oracle re-derives the
    // pair from that table, independently of HandleSkeleton::of, and the
    // assertion below then checks the trie agrees.
    let (h1, h2) = find_confusable_pair()
        .expect("test setup: no confusable pair found among candidates");

    let m1 = MemberLeaf::new(
        member_id("k1"),
        h(&h1),
        member_key("k1"),
        nm("A"),
        sn("B"),
        vec![device_key("d1")])
    .unwrap();
    let m2 = MemberLeaf::new(
        member_id("k2"),
        h(&h2),
        member_key("k2"),
        nm("A"),
        sn("B"),
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
        h(&h1),
        member_key("k1"),
        nm("A"),
        sn("B"),
        vec![device_key("d1")])
    .unwrap();
    let m2 = MemberLeaf::new(
        member_id("k2"),
        h(&h2),
        member_key("k2"),
        nm("A"),
        sn("B"),
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
        h(&h1),
        member_key("k1"),
        nm("A"),
        sn("B"),
        vec![device_key("d1")])
    .unwrap();
    let m2 = alice();

    let trie = TestTrie::genesis(vec![m1, m2]).unwrap();

    // Now try to update alice's handle to a confusable of h1.
    let err = trie.update_handle(&member_id("alice-id"), h(&h2)).unwrap_err();
    assert_eq!(err, OrgMembersError::ConfusableHandle);
}

/// Finds two distinct handles that `Handle::parse` accepts and whose UTS#39
/// skeletons match, computed directly with
/// `unicode_security::confusable_detection::skeleton` (as `fuzz_tests.rs`
/// does). The oracle is independent of the trie and of `HandleSkeleton::of`,
/// so a skeleton function that stops detecting confusables makes the callers'
/// `ConfusableHandle` assertions fail instead of changing the pair chosen.
/// Returns None if no pair was found (in which case the test will signal a
/// setup issue).
fn find_confusable_pair() -> Option<(String, String)> {
    use unicode_security::confusable_detection::skeleton;
    let candidates = [
        "paypal", "paypa1", "h0use", "house", "g00gle", "google", "ab1", "abl", "amaz0n", "amazon",
        "g0t", "got", "0lice", "olice", "01ice", "alice",
    ];
    let valid: Vec<&str> = candidates.into_iter().filter(|c| Handle::parse(c).is_ok()).collect();
    let skel = |s: &str| -> String { skeleton(s).collect() };
    for (i, &a) in valid.iter().enumerate() {
        for &b in &valid[i + 1..] {
            if skel(a) == skel(b) {
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
        h("alice"),
        member_key("k"),
        nm("e\u{0301}"),
        sn("X"),
        vec![device_key("d")])
    .unwrap();
    let m2 = MemberLeaf::new(
        member_id("k"),
        h("alice"),
        member_key("k"),
        nm("\u{00E9}"),
        sn("X"),
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
        h("alice"),
        member_key("k"),
        nm("Alice"),
        sn("Smith"),
        devices);
    assert_eq!(err.unwrap_err(), OrgMembersError::DeviceSlotsFull);
}

/// verifies: LLR-paxj7b
#[test]
fn member_leaf_empty_devices() {
    let err = MemberLeaf::new(
        member_id("k"),
        h("alice"),
        member_key("k"),
        nm("Alice"),
        sn("Smith"),
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
    assert_eq!(leaf.handle().as_str(), "alice");
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
    assert!(!trie.contains_handle(&h("alice")));
    assert!(trie.contains_handle(&h("bob")));
    assert!(trie.contains_handle(&h("charlie")));
    assert!(trie.contains_handle(&h("jan-jan")));

    assert_eq!(delta.removed().len(), 1);
    assert_eq!(delta.upserted().len(), 3);
}

// --- Jan-Jan specific tests ---

#[test]
fn jan_jan_has_two_devices() {
    let trie = TestTrie::genesis(vec![jan_jan()]).unwrap();
    let member = trie.get_by_handle(&h("jan-jan")).unwrap();
    assert_eq!(member.p2p_device_count(), 2);
    assert_eq!(member.name().as_str(), "Jan-Jan");
    assert_eq!(member.surname().as_str(), "Gödel");
}

#[test]
fn member_lookup_by_id() {
    let trie = TestTrie::genesis(vec![alice(), bob(), jan_jan()]).unwrap();

    let found = trie.get(&member_id("jan-jan-id")).unwrap();
    assert_eq!(found.name().as_str(), "Jan-Jan");

    let found = trie.get(&member_id("alice-id")).unwrap();
    assert_eq!(found.name().as_str(), "Alice");

    assert!(trie.get(&member_id("eve-id")).is_none());
}

// --- Lookup by handle ---

/// verifies: LLR-ub6dw9
#[test]
fn get_by_handle() {
    let trie = TestTrie::genesis(vec![alice(), bob(), jan_jan()]).unwrap();

    let found = trie.get_by_handle(&h("jan-jan")).unwrap();
    assert_eq!(found.name().as_str(), "Jan-Jan");

    let found = trie.get_by_handle(&h("alice")).unwrap();
    assert_eq!(found.name().as_str(), "Alice");

    assert!(trie.get_by_handle(&h("eve")).is_none());
}

/// verifies: LLR-ub6dw9
#[test]
fn contains_handle() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    assert!(trie.contains_handle(&h("alice")));
    assert!(trie.contains_handle(&h("bob")));
    assert!(!trie.contains_handle(&h("charlie")));
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

    // Peer A rotates alice's PersonPublicKey (handle and id unchanged).
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
/// The same block also returns `DuplicateHandle`. On 2026-09-17 nothing in this
/// file asserted that half; it is now carried by
/// `apply_delta_rejects_upsert_taking_handle_of_untouched_member`,
/// `apply_delta_rejects_two_upserts_claiming_one_handle` and the proptest
/// `apply_delta_never_admits_a_handle_collision` (tests/fuzz_tests.rs), all
/// annotated `verifies: LLR-juxk9q` and each measured red with only that
/// return deleted (2026-10-03), and by `membership_conformance`
/// (tests/mbt_conformance.rs).
#[test]
fn apply_delta_rejects_confusable_in_upsert() {
    let (h1, h2) =
        find_confusable_pair().expect("test setup: no confusable pair found among candidates");

    // Receiver trie holds a member with handle h1.
    let m1 = MemberLeaf::new(
        member_id("k1"),
        h(&h1),
        member_key("k1"),
        nm("A"),
        sn("B"),
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
            h(&h2),
            member_key("k2"),
            nm("A"),
            sn("B"),
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
    assert_send_sync::<PersonPublicKey>();
    assert_send_sync::<DevicePublicKey>();
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
        p2p_key: PersonPublicKey,
        name: &'a str,
        surname: &'a str,
        p2p_devices: org_members::types::DeviceSlots,
    }

    let p2p_devices = {
        let leaf = alice();
        let dev_bytes = to_allocvec(&leaf).unwrap();
        let leaf2: MemberLeaf = from_bytes(&dev_bytes).unwrap();
        org_members::types::DeviceSlots::parse(leaf2.p2p_devices().to_vec()).unwrap()
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
    let empty_devices: Vec<DevicePublicKey> = vec![];
    let bytes = to_allocvec(&empty_devices).unwrap();
    let result: Result<org_members::types::DeviceSlots, _> = from_bytes(&bytes);
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
        h("alice"),
        member_key("k"),
        nm("A"),
        sn("B"),
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
    let err = Name::parse(&long_name).map_err(OrgMembersError::from).and_then(|name| {
        MemberLeaf::new(
            member_id("k"),
            h("alice"),
            member_key("k"),
            name,
            sn("B"),
            vec![device_key("d")],
        )
    });
    assert_eq!(
        err.unwrap_err(),
        OrgMembersError::FieldTooLong { field: "name", max: 128 }
    );
}

/// verifies: LLR-w5nkbu
#[test]
fn member_leaf_new_rejects_oversized_surname() {
    let long_surname = "b".repeat(129);
    let err = Surname::parse(&long_surname).map_err(OrgMembersError::from).and_then(|surname| {
        MemberLeaf::new(
            member_id("k"),
            h("alice"),
            member_key("k"),
            nm("A"),
            surname,
            vec![device_key("d")],
        )
    });
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
        h("alice"),
        member_key("k"),
        nm(&name_128),
        sn(&surname_128),
        vec![device_key("d")],
    );
    assert!(ok.is_ok());
}

#[cfg(feature = "serde")]
/// verifies: LLR-w5nkbu, LLR-68tka5
#[test]
fn deserialize_rejects_oversized_name() {
    use postcard::{from_bytes, to_allocvec};
    use org_members::types::DeviceSlots;

    #[derive(serde::Serialize)]
    struct WireLeaf<'a> {
        id: MemberId,
        handle: &'a str,
        p2p_key: PersonPublicKey,
        name: &'a str,
        surname: &'a str,
        p2p_devices: DeviceSlots,
    }
    let long_name = "a".repeat(200);
    let wire = WireLeaf {
        id: member_id("k"),
        handle: "alice",
        p2p_key: member_key("k"),
        name: &long_name,
        surname: "B",
        p2p_devices: DeviceSlots::parse(vec![device_key("d")]).unwrap(),
    };
    let bytes = to_allocvec(&wire).unwrap();
    let result: Result<MemberLeaf, _> = from_bytes(&bytes);
    assert!(result.is_err());
}

#[cfg(feature = "serde")]
/// verifies: LLR-w5nkbu, LLR-68tka5
#[test]
fn deserialize_rejects_oversized_surname() {
    use postcard::{from_bytes, to_allocvec};
    use org_members::types::DeviceSlots;

    #[derive(serde::Serialize)]
    struct WireLeaf<'a> {
        id: MemberId,
        handle: &'a str,
        p2p_key: PersonPublicKey,
        name: &'a str,
        surname: &'a str,
        p2p_devices: DeviceSlots,
    }
    let long_surname = "b".repeat(200);
    let wire = WireLeaf {
        id: member_id("k"),
        handle: "alice",
        p2p_key: member_key("k"),
        name: "A",
        surname: &long_surname,
        p2p_devices: DeviceSlots::parse(vec![device_key("d")]).unwrap(),
    };
    let bytes = to_allocvec(&wire).unwrap();
    let result: Result<MemberLeaf, _> = from_bytes(&bytes);
    assert!(result.is_err());
}

/// verifies: LLR-w5nkbu
///
/// An oversized name never reaches `update_name_surname`: `Name::parse`
/// refuses it first.
#[test]
fn name_for_update_rejects_oversized() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let long_name = "a".repeat(129);
    let err = Name::parse(&long_name)
        .map_err(OrgMembersError::from)
        .and_then(|name| trie.update_name_surname(&member_id("alice-id"), name, sn("Smith")));
    assert_eq!(
        err.unwrap_err(),
        OrgMembersError::FieldTooLong { field: "name", max: 128 }
    );
}

/// verifies: LLR-w5nkbu
///
/// An oversized surname never reaches `update_name_surname`:
/// `Surname::parse` refuses it first.
#[test]
fn surname_for_update_rejects_oversized() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let long_surname = "b".repeat(129);
    let err = Surname::parse(&long_surname)
        .map_err(OrgMembersError::from)
        .and_then(|surname| trie.update_name_surname(&member_id("alice-id"), nm("Alice"), surname));
    assert_eq!(
        err.unwrap_err(),
        OrgMembersError::FieldTooLong { field: "surname", max: 128 }
    );
}

// --- H-2: DeviceSlots deserialize rejects non-canonical wire form ---

/// postcard keeps no message, so the rule a refused set of DevicePublicKeys
/// broke is read from serde_json, which keeps it.
#[cfg(feature = "serde")]
fn assert_device_set_refused(wire: &[DevicePublicKey], rule: &str) {
    let json = serde_json::to_string(wire).unwrap();
    let error = serde_json::from_str::<org_members::types::DeviceSlots>(&json).unwrap_err();
    assert!(error.is_data(), "{error}");
    assert!(error.to_string().starts_with(rule), "{error}");
}

#[cfg(feature = "serde")]
/// verifies: REQ-shk82j, LLR-xyv6p9
#[test]
fn deserialize_rejects_unsorted_devices() {
    use postcard::{from_bytes, to_allocvec};
    let d1 = device_key("d1");
    let d2 = device_key("d2");
    let (lo, hi) = if d1.as_bytes() < d2.as_bytes() { (d1, d2) } else { (d2, d1) };
    let unsorted_wire: Vec<DevicePublicKey> = vec![hi, lo];
    let bytes = to_allocvec(&unsorted_wire).unwrap();
    let result: Result<org_members::types::DeviceSlots, _> = from_bytes(&bytes);
    assert_eq!(result, Err(postcard::Error::SerdeDeCustom), "deserialize must reject unsorted device list");
    assert_device_set_refused(&unsorted_wire, "device slots must be strictly increasing");
}

#[cfg(feature = "serde")]
/// verifies: REQ-shk82j, LLR-xyv6p9
#[test]
fn deserialize_rejects_duplicate_devices() {
    use postcard::{from_bytes, to_allocvec};
    let d = device_key("d1");
    let dup_wire: Vec<DevicePublicKey> = vec![d, d];
    let bytes = to_allocvec(&dup_wire).unwrap();
    let result: Result<org_members::types::DeviceSlots, _> = from_bytes(&bytes);
    assert_eq!(result, Err(postcard::Error::SerdeDeCustom), "deserialize must reject duplicate devices");
    assert_device_set_refused(&dup_wire, "device slots must be strictly increasing");
}

#[cfg(feature = "serde")]
/// verifies: REQ-shk82j, REQ-xdx2c2, LLR-pys2ek, LLR-xyv6p9
#[test]
fn deserialize_rejects_too_many_devices() {
    use postcard::{from_bytes, to_allocvec};
    let many: Vec<DevicePublicKey> = (0..5).map(|i| device_key(&format!("d{}", i))).collect();
    // Sort so we hit the count check, not the order check.
    let mut sorted = many.clone();
    sorted.sort();
    let bytes = to_allocvec(&sorted).unwrap();
    let result: Result<org_members::types::DeviceSlots, _> = from_bytes(&bytes);
    assert_eq!(result, Err(postcard::Error::SerdeDeCustom), "deserialize must reject more than MAX_DEVICES");
    assert_device_set_refused(&sorted, "device slots exceed MAX_DEVICES");
}

#[cfg(feature = "serde")]
/// verifies: REQ-shk82j, LLR-xyv6p9
#[test]
fn deserialize_accepts_sorted_unique_devices() {
    use postcard::{from_bytes, to_allocvec};
    let d1 = device_key("d1");
    let d2 = device_key("d2");
    let (lo, hi) = if d1.as_bytes() < d2.as_bytes() { (d1, d2) } else { (d2, d1) };
    let canonical: Vec<DevicePublicKey> = vec![lo, hi];
    let bytes = to_allocvec(&canonical).unwrap();
    let result: org_members::types::DeviceSlots = from_bytes(&bytes).unwrap();
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

// --- Collisions that survive the release of outgoing handles (LLR-n5t6bn) ---

/// `leaf` with its handle replaced and every other field unchanged.
fn with_handle(leaf: &MemberLeaf, handle: &str) -> MemberLeaf {
    MemberLeaf::new(
        *leaf.id(),
        h(handle),
        *leaf.p2p_key(),
        leaf.name().clone(),
        leaf.surname().clone(),
        leaf.p2p_devices().to_vec())
    .unwrap()
}

/// A delta based on `trie`'s root that removes nothing and upserts exactly
/// `upserted`, put in identifier order so only the handle check can refuse it.
fn upsert_only_delta(trie: &TestTrie, mut upserted: Vec<MemberLeaf>) -> org_members::delta::Delta {
    let base_member = MemberLeaf::new(
        member_id("delta-base"),
        h("deltabase"),
        member_key("delta-base"),
        nm("D"),
        sn("B"),
        vec![device_key("delta-base-d1")])
    .unwrap();
    let (_, mut delta) = trie.add_member(base_member).unwrap().recalculate().unwrap();
    upserted.sort_by(|a, b| a.id().cmp(b.id()));
    org_members::delta::test_support::delta_set_removed(&mut delta, Vec::new());
    org_members::delta::test_support::delta_set_upserted(&mut delta, upserted);
    delta
}

/// verifies: LLR-n5t6bn, LLR-juxk9q
///
/// Both upserted members give up their own handle and claim the same new one.
/// Releasing the outgoing handles first must not let the second claim through.
/// Reds when the post-release handle check in `apply_delta` is skipped, and
/// when only its `DuplicateHandle` return is deleted (measured 2026-10-03) --
/// `apply_delta` then returns `Ok` and `unwrap_err()` panics.
#[test]
fn apply_delta_rejects_two_upserts_claiming_one_handle() {
    let trie = TestTrie::genesis(vec![alice(), bob(), charlie()]).unwrap();
    let delta = upsert_only_delta(
        &trie,
        vec![with_handle(&alice(), "dave"), with_handle(&bob(), "dave")],
    );

    let err = trie.apply_delta(&delta).unwrap_err();
    assert!(matches!(err, OrgMembersError::DuplicateHandle), "got {err:?}");
}

/// verifies: LLR-n5t6bn, LLR-juxk9q
///
/// Alice takes charlie's handle while bob lawfully takes alice's released one.
/// Charlie is neither removed nor upserted, so his handle is never released and
/// alice's claim must still be refused. This is also the only test asserting
/// that an upsert taking another member's handle is refused at apply time.
/// Reds when the post-release handle check is skipped, when only its
/// `DuplicateHandle` return is deleted (measured 2026-10-03), and when the
/// release loop also releases each upsert's incoming handle.
#[test]
fn apply_delta_rejects_upsert_taking_handle_of_untouched_member() {
    let trie = TestTrie::genesis(vec![alice(), bob(), charlie()]).unwrap();
    let delta = upsert_only_delta(
        &trie,
        vec![with_handle(&alice(), "charlie"), with_handle(&bob(), "alice")],
    );

    let err = trie.apply_delta(&delta).unwrap_err();
    assert!(matches!(err, OrgMembersError::DuplicateHandle), "got {err:?}");
}

/// verifies: LLR-n5t6bn
///
/// Same shape as above with a confusable instead of an exact match: alice
/// takes a handle confusable with one an untouched member holds, while bob
/// lawfully takes alice's released handle. Reds when the post-release check
/// is skipped, and when the release loop also releases each upsert's incoming
/// handle's skeleton.
#[test]
fn apply_delta_rejects_confusable_of_untouched_member_after_release() {
    let (h1, h2) =
        find_confusable_pair().expect("test setup: no confusable pair found among candidates");
    let holder = MemberLeaf::new(
        member_id("k1"),
        h(&h1),
        member_key("k1"),
        nm("A"),
        sn("B"),
        vec![device_key("d1")])
    .unwrap();
    let trie = TestTrie::genesis(vec![holder, alice(), bob()]).unwrap();
    let delta = upsert_only_delta(
        &trie,
        vec![with_handle(&alice(), &h2), with_handle(&bob(), "alice")],
    );

    let err = trie.apply_delta(&delta).unwrap_err();
    assert!(matches!(err, OrgMembersError::ConfusableHandle), "got {err:?}");
}

// --- Root hash after an irregular history (LLR-4n8zqx) ---

/// verifies: LLR-4n8zqx
///
/// The abnormal-input side of "the root is determined by the member set
/// alone". `same_members_same_root_hash` and
/// `different_insertion_order_same_root` build the set by genesis only; this
/// test reaches the same set {alice, bob, charlie} through a history no
/// genesis takes, and asserts it reports the root a direct genesis reports:
///
/// - rejected operations interleaved (duplicate id, a new member taking a
///   held handle, a present member taking a held handle, a stale delta);
/// - a member deleted and re-added across a `recalculate()`;
/// - a member deleted for good, so its slot was occupied and is empty again;
/// - a member's handle changed away and back across a `recalculate()`.
///
/// It then sends the result to a receiver on a different base (a trie holding
/// two members the history never had) via `calculate_delta` and
/// `apply_delta`, and asserts the candidate passes `verify_against` the
/// direct root. The `matches!` asserts only confirm each rejection really
/// happened, so the history contains them; they are not root assertions.
///
/// Measured red (2026-10-03), with both genesis-only tests staying green:
/// `smt::remove` writing an empty leaf with a non-default hash (a tombstone
/// left in the hash) and `smt::insert` mixing a replaced leaf's device root
/// into its successor's (an update leaving history in the hash) both fail
/// the sender's root assertion; the diff walk dropping removals fails the
/// receiver's `verify_against`, with the sender's root still correct.
#[test]
fn irregular_history_reaches_same_root_as_direct_build() {
    let direct = TestTrie::genesis(vec![alice(), bob(), charlie()]).unwrap();
    let direct_root = direct.root_hash().unwrap();

    // Rejection 1: duplicate id.
    let start = TestTrie::genesis(vec![charlie(), diana()]).unwrap();
    let impostor = MemberLeaf::new(
        member_id("charlie-id"),
        h("mallory"),
        member_key("mallory-mk"),
        nm("Mallory"),
        sn("Impostor"),
        vec![device_key("mallory-d1")])
    .unwrap();
    let rejected = start.add_member(impostor);
    assert!(matches!(rejected, Err(OrgMembersError::DuplicateId)), "got {rejected:?}");

    let (trie, _) = start.add_member(alice()).unwrap().recalculate().unwrap();

    // Rejection 2: a new member taking a held handle.
    let handle_thief = MemberLeaf::new(
        member_id("thief-id"),
        h("alice"),
        member_key("thief-mk"),
        nm("T"),
        sn("H"),
        vec![device_key("thief-d1")])
    .unwrap();
    let rejected = trie.add_member(handle_thief);
    assert!(matches!(rejected, Err(OrgMembersError::DuplicateHandle)), "got {rejected:?}");

    // Delete charlie and re-add them across a recalculate.
    let (trie, _) = trie.delete_member(&member_id("charlie-id")).unwrap().recalculate().unwrap();
    let (trie, _) = trie.add_member(charlie()).unwrap().recalculate().unwrap();

    // Rejection 3: a present member taking a held handle on update.
    let rejected = trie.update_handle(&member_id("alice-id"), h("charlie"));
    assert!(matches!(rejected, Err(OrgMembersError::DuplicateHandle)), "got {rejected:?}");

    // Change alice's handle away and back across a recalculate.
    let (trie, _) = trie.update_handle(&member_id("alice-id"), h("alicia")).unwrap().recalculate().unwrap();
    assert!(trie.contains_handle(&h("alicia")), "test setup: alice's handle must have moved");
    let (trie, _) = trie.update_handle(&member_id("alice-id"), h("alice")).unwrap().recalculate().unwrap();

    // Rejection 4: a delta built on another root is stale against `trie`.
    let (_, stale) = TestTrie::genesis(vec![charlie()])
        .unwrap()
        .add_member(jan_jan())
        .unwrap()
        .recalculate()
        .unwrap();
    let rejected = trie.apply_delta(&stale);
    assert!(matches!(rejected, Err(OrgMembersError::DeltaBaseMismatch)), "got {:?}", rejected.err());

    // Add bob and delete diana for good.
    let trie = trie.add_member(bob()).unwrap();
    let (sender, _) = trie.delete_member(&member_id("diana-id")).unwrap().recalculate().unwrap();

    let mut sender_members = sender.members();
    sender_members.sort_by(|a, b| a.id().cmp(b.id()));
    let mut direct_members = direct.members();
    direct_members.sort_by(|a, b| a.id().cmp(b.id()));
    assert_eq!(sender_members, direct_members, "test setup: the history must end at the direct member set");
    assert_eq!(sender.root_hash().unwrap(), direct_root);

    // Received from a different base.
    let base = TestTrie::genesis(vec![diana(), jan_jan()]).unwrap();
    let delta = sender.calculate_delta(&base).unwrap();
    let received = base
        .apply_delta(&delta)
        .unwrap()
        .verify_against(&direct_root)
        .expect("the received trie must verify against the direct root");
    assert_eq!(received.root_hash().unwrap(), direct_root);
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
    let published = trie.root_hash().unwrap();

    let mutated = trie.add_member(charlie()).unwrap();
    assert!(!mutated.is_calculated());
    assert_eq!(mutated.root_hash().unwrap_err(), OrgMembersError::HashesNotCalculated);

    // And the normal case: recalculation produces a root, and it is not the
    // one the pre-mutation trie published.
    let (mutated, _) = mutated.recalculate().unwrap();
    assert_ne!(mutated.root_hash().unwrap(), published);
}

/// verifies: LLR-j35sxz
///
/// The abnormal case: every way a trie comes back calculated -- `genesis()`,
/// `recalculate()` and `verify_against()` -- refuses a further `recalculate()`.
/// Its node hashes are write-once, so there is nothing left to fill.
#[test]
fn recalculate_refuses_a_calculated_trie() {
    let genesis = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    assert_eq!(genesis.recalculate().unwrap_err(), OrgMembersError::HashesAlreadyCalculated);

    let (recalculated, delta) = genesis.add_member(charlie()).unwrap().recalculate().unwrap();
    assert_eq!(recalculated.recalculate().unwrap_err(), OrgMembersError::HashesAlreadyCalculated);

    let verified = genesis
        .apply_delta(&delta)
        .unwrap()
        .verify_against(&recalculated.root_hash().unwrap())
        .unwrap();
    assert_eq!(verified.recalculate().unwrap_err(), OrgMembersError::HashesAlreadyCalculated);

    // The refusal does not depend on the trie holding members: an empty
    // organisation's genesis trie is calculated too.
    let empty = TestTrie::genesis(vec![]).unwrap();
    assert_eq!(empty.recalculate().unwrap_err(), OrgMembersError::HashesAlreadyCalculated);

    // Nor on how many it holds: a one-member organisation's genesis trie is
    // refused like any other.
    let single = TestTrie::genesis(vec![alice()]).unwrap();
    assert_eq!(single.recalculate().unwrap_err(), OrgMembersError::HashesAlreadyCalculated);
}

/// verifies: LLR-j35sxz
///
/// The normal case: a mutated trie recalculates, and the refusal starts exactly
/// when its pending changes end -- the trie that comes back is refused.
#[test]
fn recalculate_succeeds_until_no_changes_are_pending() {
    let genesis = TestTrie::genesis(vec![alice()]).unwrap();
    let mutated = genesis.add_member(bob()).unwrap();
    assert!(mutated.has_pending_changes());

    let (calculated, _) = mutated.recalculate().unwrap();
    assert!(!calculated.has_pending_changes());
    assert_eq!(calculated.recalculate().unwrap_err(), OrgMembersError::HashesAlreadyCalculated);
}

/// verifies: LLR-j35sxz
///
/// The normal case at the boundary: a trie mutated and then reverted to its
/// original members has an empty change set, but it was mutated, so it is not
/// refused. It recalculates to an empty delta based on, and arriving at, the
/// original root.
#[test]
fn recalculate_accepts_a_mutated_trie_with_an_empty_change_set() {
    let genesis = TestTrie::genesis(vec![alice()]).unwrap();
    let original = genesis.root_hash().unwrap();

    let reverted = genesis
        .add_member(bob())
        .unwrap()
        .delete_member(bob().id())
        .unwrap();
    assert!(reverted.has_pending_changes());
    assert!(reverted.pending_changes().unwrap().is_empty());

    let (recalculated, delta) = reverted.recalculate().unwrap();
    assert!(delta.is_empty());
    assert_eq!(delta.base_root(), &original);
    assert_eq!(recalculated.root_hash().unwrap(), original);
}

// A `deserialize_revalidates_handle` test stood here briefly. It was removed:
// `deserialize_rejects_invalid_handle`, above, already asserted the same
// invariant and had done so since before the requirements work began. Writing
// a second one was a mistake made by trusting a doc-comment ("gated so a plain
// cargo test skips") over the file it describes; the annotation it was written
// to carry now sits on the existing test instead.

/// True when `bytes` is the encoding of no DevicePublicKey: not 32 bytes long,
/// or 32 bytes `DevicePublicKey::parse` refuses.
fn encodes_no_device_public_key(bytes: &[u8]) -> bool {
    match <[u8; 32]>::try_from(bytes) {
        Ok(key_bytes) => {
            DevicePublicKey::parse(&key_bytes) == Err(person::IdentityError::InvalidDeviceKey)
        }
        Err(_) => true,
    }
}

/// verifies: LLR-jfj6pc
///
/// An empty device slot hashes the sentinel in the device-leaf domain, and an
/// occupied one hashes its DevicePublicKey's bytes there; the sentinel being
/// no DevicePublicKey's encoding keeps an empty slot from hashing as a device.
#[test]
fn the_device_empty_sentinel_encodes_no_device_public_key() {
    let sentinel = <Blake3Hasher as DeviceTrieHasher>::DEVICE_EMPTY_SENTINEL;
    assert!(encodes_no_device_public_key(sentinel), "the sentinel is a DevicePublicKey");
}

/// verifies: LLR-jfj6pc
///
/// The check discriminates: a DevicePublicKey's bytes fail it, 32 bytes that
/// are no DevicePublicKey (the identity point) and a 31-byte string pass it.
#[test]
fn the_sentinel_check_tells_a_device_public_key_from_other_bytes() {
    assert!(!encodes_no_device_public_key(device_key("d1").as_bytes()));
    let mut identity = [0u8; 32];
    identity[0] = 1;
    assert!(encodes_no_device_public_key(&identity));
    assert!(encodes_no_device_public_key(&device_key("d1").as_bytes()[..31]));
}

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
        h("alice"),
        member_key("k"),
        nm("Alice"),
        sn("Anderson"),
        vec![d1, d2],
    )
    .unwrap();
    let reverse = MemberLeaf::new(
        member_id("m"),
        h("alice"),
        member_key("k"),
        nm("Alice"),
        sn("Anderson"),
        vec![d2, d1],
    )
    .unwrap();

    let a = TestTrie::genesis(vec![forward]).unwrap().root_hash().unwrap();
    let b = TestTrie::genesis(vec![reverse]).unwrap().root_hash().unwrap();

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
    // `DeviceSlots` stores devices in sorted order, so which slot a key
    // lands in is decided by its digest, not by the order it is written here.
    // Sort a pool of MAX_DEVICES + 1 keys and take the first MAX_DEVICES as
    // the base: `base[i]` is then the key in slot `i`, by construction rather
    // than by coincidence. The spare is the pool's largest, so substituting it
    // for any base key leaves it in the LAST slot -- which is the only
    // substitution that can be invisible to a root that hashes fewer slots
    // than MAX_DEVICES. Picking an arbitrary replacement key instead makes the
    // detection depend on where that one digest happens to sort.
    let mut pool: Vec<DevicePublicKey> =
        (0..=MAX_DEVICES).map(|i| device_key(&format!("dev-{i}"))).collect();
    pool.sort();
    let base: Vec<DevicePublicKey> = pool[..MAX_DEVICES].to_vec();
    let spare = pool[MAX_DEVICES];

    let root_of = |devices: Vec<DevicePublicKey>| {
        let leaf = MemberLeaf::new(
            member_id("m"),
            h("alice"),
            member_key("k"),
            nm("Alice"),
            sn("Anderson"),
            devices,
        )
        .unwrap();
        TestTrie::genesis(vec![leaf]).unwrap().root_hash().unwrap()
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
    let empty = TestTrie::genesis(vec![]).unwrap().root_hash().unwrap();

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

// --- Key uniqueness across the organisation (LLR-v6gfc7, LLR-fym7dy, LLR-gjj6bx) ---
//
// `member_key(s)` and `device_key(s)` derive the same 32 bytes from the same
// seed, so `member_key("alice-d1")` is alice's enrolled device key read as a
// member key. The refusals below are atomic by construction: every operation
// takes `&self` and returns `Result<Self, _>`, so an `Err` carries no trie.

/// A leaf with a fixed name, for key-uniqueness fixtures.
fn keyed_leaf(seed: &str, mk: PersonPublicKey, devices: Vec<DevicePublicKey>) -> MemberLeaf {
    MemberLeaf::new(member_id(&format!("{seed}-id")), h(seed), mk, nm("Key"), sn("Holder"), devices).unwrap()
}

/// Every key held in `trie`, member and device keys together, as bytes.
fn held_key_bytes(trie: &TestTrie) -> Vec<[u8; 32]> {
    let mut keys = Vec::new();
    for m in trie.members() {
        keys.push(*m.p2p_key().as_bytes());
        keys.extend(m.p2p_devices().iter().map(|d| *d.as_bytes()));
    }
    keys
}

fn assert_keys_unique(trie: &TestTrie) {
    let mut keys = held_key_bytes(trie);
    let n = keys.len();
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), n, "a key is held in two places");
}

/// verifies: LLR-v6gfc7
///
/// Normal case: members whose member and device keys are all distinct are
/// accepted, and the record holds each key once.
#[test]
fn genesis_accepts_distinct_keys() {
    let trie = TestTrie::genesis(vec![alice(), bob(), jan_jan()]).unwrap();
    assert_keys_unique(&trie);
    assert_eq!(held_key_bytes(&trie).len(), 7);
}

/// verifies: LLR-v6gfc7
#[test]
fn genesis_rejects_member_key_held_by_two_members() {
    let other = keyed_leaf("eve", member_key("alice-mk"), vec![device_key("eve-d1")]);
    let err = TestTrie::genesis(vec![alice(), other]).unwrap_err();
    assert_eq!(err, OrgMembersError::DuplicateKey);
}

/// verifies: LLR-v6gfc7
#[test]
fn genesis_rejects_device_key_enrolled_under_two_members() {
    let other = keyed_leaf("eve", member_key("eve-mk"), vec![device_key("alice-d1")]);
    let err = TestTrie::genesis(vec![alice(), other]).unwrap_err();
    assert_eq!(err, OrgMembersError::DuplicateKey);
}

/// verifies: LLR-v6gfc7
#[test]
fn genesis_rejects_member_key_equal_to_another_members_device_key() {
    let other = keyed_leaf("eve", member_key("alice-d1"), vec![device_key("eve-d1")]);
    let err = TestTrie::genesis(vec![alice(), other]).unwrap_err();
    assert_eq!(err, OrgMembersError::DuplicateKey);
    // And the other way round: a device key equal to an earlier member key.
    let other = keyed_leaf("eve", member_key("eve-mk"), vec![device_key("alice-mk")]);
    let err = TestTrie::genesis(vec![alice(), other]).unwrap_err();
    assert_eq!(err, OrgMembersError::DuplicateKey);
}

/// verifies: LLR-v6gfc7
///
/// Boundary: a single member whose member key is one of its own devices.
#[test]
fn genesis_rejects_member_key_equal_to_own_device_key() {
    let leaf = keyed_leaf("eve", member_key("eve-d2"), vec![device_key("eve-d1"), device_key("eve-d2")]);
    let err = TestTrie::genesis(vec![leaf]).unwrap_err();
    assert_eq!(err, OrgMembersError::DuplicateKey);
}

/// verifies: LLR-v6gfc7
///
/// Check order: `DuplicateId` and the handle checks come before `DuplicateKey`.
#[test]
fn genesis_reports_id_and_handle_before_shared_key() {
    // Same id, same keys.
    let err = TestTrie::genesis(vec![alice(), alice()]).unwrap_err();
    assert_eq!(err, OrgMembersError::DuplicateId);
    // Different id, same handle, same member key.
    let twin = MemberLeaf::new(
        member_id("alice-twin-id"),
        h("alice"),
        member_key("alice-mk"),
        nm("A"),
        sn("B"),
        vec![device_key("alice-d1")],
    )
    .unwrap();
    let err = TestTrie::genesis(vec![alice(), twin]).unwrap_err();
    assert_eq!(err, OrgMembersError::DuplicateHandle);
}

/// verifies: LLR-v6gfc7
#[test]
fn add_member_rejects_member_key_held_by_another_member() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let other = keyed_leaf("eve", member_key("alice-mk"), vec![device_key("eve-d1")]);
    assert_eq!(trie.add_member(other).unwrap_err(), OrgMembersError::DuplicateKey);
}

/// verifies: LLR-v6gfc7
#[test]
fn add_member_rejects_device_key_held_by_another_member() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    // Another member's device key.
    let other = keyed_leaf("eve", member_key("eve-mk"), vec![device_key("alice-d1")]);
    assert_eq!(trie.add_member(other).unwrap_err(), OrgMembersError::DuplicateKey);
    // Another member's member key, enrolled as a device.
    let other = keyed_leaf("eve", member_key("eve-mk"), vec![device_key("alice-mk")]);
    assert_eq!(trie.add_member(other).unwrap_err(), OrgMembersError::DuplicateKey);
}

/// verifies: LLR-v6gfc7
#[test]
fn add_member_rejects_member_key_equal_to_a_device_key() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    // Another member's device.
    let other = keyed_leaf("eve", member_key("alice-d1"), vec![device_key("eve-d1")]);
    assert_eq!(trie.add_member(other).unwrap_err(), OrgMembersError::DuplicateKey);
    // Its own device.
    let own = keyed_leaf("eve", member_key("eve-d1"), vec![device_key("eve-d1")]);
    assert_eq!(trie.add_member(own).unwrap_err(), OrgMembersError::DuplicateKey);
}

/// verifies: LLR-v6gfc7
///
/// Check order: `DuplicateId`, then the handle checks, then `DuplicateKey`.
#[test]
fn add_member_reports_id_and_handle_before_shared_key() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    assert_eq!(trie.add_member(alice()).unwrap_err(), OrgMembersError::DuplicateId);
    let twin = MemberLeaf::new(
        member_id("alice-twin-id"),
        h("alice"),
        member_key("alice-mk"),
        nm("A"),
        sn("B"),
        vec![device_key("alice-d1")],
    )
    .unwrap();
    assert_eq!(trie.add_member(twin).unwrap_err(), OrgMembersError::DuplicateHandle);
}

/// verifies: LLR-v6gfc7
///
/// Normal case: only keys held now count. A deleted member's keys are no
/// longer held, so a new member may be given them (the caller duty in README
/// item 11 still applies — the crate keeps no history).
#[test]
fn add_member_accepts_keys_no_longer_held() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let trie = trie.delete_member(&member_id("bob-id")).unwrap();
    let reuse = keyed_leaf("eve", member_key("bob-mk"), vec![device_key("bob-d1")]);
    let trie = trie.add_member(reuse).unwrap();
    assert_keys_unique(&trie);
}

/// verifies: LLR-v6gfc7
#[test]
fn add_p2p_device_rejects_key_held_by_another_member() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let id = member_id("alice-id");
    // Bob's device, bob's member key, and alice's own member key.
    for seed in ["bob-d1", "bob-mk", "alice-mk"] {
        let err = trie.add_p2p_device(&id, device_key(seed)).unwrap_err();
        assert_eq!(err, OrgMembersError::DuplicateKey, "device {seed}");
    }
}

/// verifies: LLR-v6gfc7
///
/// Check order: `IdNotFound`, `DuplicateDevice` and `DeviceSlotsFull` all come
/// before `DuplicateKey`.
#[test]
fn add_p2p_device_reports_earlier_refusals_before_shared_key() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let err = trie.add_p2p_device(&member_id("ghost-id"), device_key("bob-d1")).unwrap_err();
    assert_eq!(err, OrgMembersError::IdNotFound);
    // Alice's own device is a DuplicateDevice, not a DuplicateKey.
    let err = trie.add_p2p_device(&member_id("alice-id"), device_key("alice-d1")).unwrap_err();
    assert_eq!(err, OrgMembersError::DuplicateDevice);
    // A full member offered a key another member holds.
    let mut full = trie.clone();
    for seed in ["alice-d2", "alice-d3", "alice-d4"] {
        full = full.add_p2p_device(&member_id("alice-id"), device_key(seed)).unwrap();
    }
    let err = full.add_p2p_device(&member_id("alice-id"), device_key("bob-d1")).unwrap_err();
    assert_eq!(err, OrgMembersError::DeviceSlotsFull);
}

/// verifies: LLR-v6gfc7
///
/// Normal case: a device key freed by `delete_p2p_device` is no longer held.
#[test]
fn add_p2p_device_accepts_key_no_longer_held() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let trie = trie
        .delete_p2p_device(&member_id("bob-id"), &device_key("bob-d1"), member_key("bob-mk2"))
        .unwrap();
    let trie = trie.add_p2p_device(&member_id("alice-id"), device_key("bob-d1")).unwrap();
    assert_keys_unique(&trie);
}

/// verifies: LLR-fym7dy
#[test]
fn rotate_p2p_key_rejects_key_held_elsewhere() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let id = member_id("alice-id");
    // Bob's member key, bob's device, alice's own device.
    for seed in ["bob-mk", "bob-d1", "alice-d1"] {
        let err = trie.rotate_p2p_key(&id, member_key(seed)).unwrap_err();
        assert_eq!(err, OrgMembersError::DuplicateKey, "key {seed}");
    }
}

/// verifies: LLR-fym7dy
///
/// Check order: `IdNotFound` and `P2pKeyNotReplaced` come before
/// `DuplicateKey`. The current key is held (by the member itself), and still
/// reports `P2pKeyNotReplaced`.
#[test]
fn rotate_p2p_key_reports_earlier_refusals_before_shared_key() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let err = trie.rotate_p2p_key(&member_id("ghost-id"), member_key("bob-mk")).unwrap_err();
    assert_eq!(err, OrgMembersError::IdNotFound);
    let err = trie.rotate_p2p_key(&member_id("alice-id"), member_key("alice-mk")).unwrap_err();
    assert_eq!(err, OrgMembersError::P2pKeyNotReplaced);
}

/// verifies: LLR-fym7dy
///
/// Normal case: a key this member held before, and no longer holds, is not
/// held anywhere and is accepted (history is the caller's, README item 11).
#[test]
fn rotate_p2p_key_accepts_key_no_longer_held() {
    let trie = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let id = member_id("alice-id");
    let trie = trie.rotate_p2p_key(&id, member_key("alice-mk2")).unwrap();
    let trie = trie.rotate_p2p_key(&id, member_key("alice-mk")).unwrap();
    assert_eq!(trie.get(&id).unwrap().p2p_key(), &member_key("alice-mk"));
    assert_keys_unique(&trie);
}

/// verifies: LLR-fym7dy
///
/// The removed device's own key is held before the operation, so it is refused
/// as the replacement even though the result would not hold it twice.
#[test]
fn delete_p2p_device_rejects_removed_devices_own_key() {
    let trie = TestTrie::genesis(vec![jan_jan()]).unwrap();
    let err = trie
        .delete_p2p_device(&member_id("jan-jan-id"), &device_key("jan-jan-d1"), member_key("jan-jan-d1"))
        .unwrap_err();
    assert_eq!(err, OrgMembersError::DuplicateKey);
    // Last device too: alice's only device.
    let trie = TestTrie::genesis(vec![alice()]).unwrap();
    let err = trie
        .delete_p2p_device(&member_id("alice-id"), &device_key("alice-d1"), member_key("alice-d1"))
        .unwrap_err();
    assert_eq!(err, OrgMembersError::DuplicateKey);
}

/// verifies: LLR-fym7dy
#[test]
fn delete_p2p_device_rejects_key_held_elsewhere() {
    let trie = TestTrie::genesis(vec![jan_jan(), bob()]).unwrap();
    let id = member_id("jan-jan-id");
    // Bob's member key, bob's device, jan-jan's remaining device.
    for seed in ["bob-mk", "bob-d1", "jan-jan-d2"] {
        let err = trie
            .delete_p2p_device(&id, &device_key("jan-jan-d1"), member_key(seed))
            .unwrap_err();
        assert_eq!(err, OrgMembersError::DuplicateKey, "key {seed}");
    }
}

/// verifies: LLR-fym7dy
///
/// Check order: `IdNotFound`, `DeviceNotFound`, `P2pKeyNotReplaced`, then
/// `DuplicateKey`.
#[test]
fn delete_p2p_device_reports_earlier_refusals_before_shared_key() {
    let trie = TestTrie::genesis(vec![jan_jan(), bob()]).unwrap();
    let err = trie
        .delete_p2p_device(&member_id("ghost-id"), &device_key("jan-jan-d1"), member_key("bob-mk"))
        .unwrap_err();
    assert_eq!(err, OrgMembersError::IdNotFound);
    let err = trie
        .delete_p2p_device(&member_id("jan-jan-id"), &device_key("does-not-exist"), member_key("bob-mk"))
        .unwrap_err();
    assert_eq!(err, OrgMembersError::DeviceNotFound);
    let err = trie
        .delete_p2p_device(&member_id("jan-jan-id"), &device_key("jan-jan-d1"), member_key("jan-jan-mk"))
        .unwrap_err();
    assert_eq!(err, OrgMembersError::P2pKeyNotReplaced);
}

/// verifies: LLR-fym7dy
#[test]
fn emergency_isolate_member_rejects_key_held_elsewhere() {
    let trie = TestTrie::genesis(vec![jan_jan(), bob()]).unwrap();
    let id = member_id("jan-jan-id");
    // Bob's member key, bob's device, and both of jan-jan's own (removed) devices.
    for seed in ["bob-mk", "bob-d1", "jan-jan-d1", "jan-jan-d2"] {
        let err = trie.emergency_isolate_member(&id, member_key(seed)).unwrap_err();
        assert_eq!(err, OrgMembersError::DuplicateKey, "key {seed}");
    }
}

/// verifies: LLR-fym7dy
///
/// Check order: `IdNotFound`, `P2pKeyNotReplaced`, then `DuplicateKey`.
#[test]
fn emergency_isolate_member_reports_earlier_refusals_before_shared_key() {
    let trie = TestTrie::genesis(vec![jan_jan(), bob()]).unwrap();
    let err = trie.emergency_isolate_member(&member_id("ghost-id"), member_key("bob-mk")).unwrap_err();
    assert_eq!(err, OrgMembersError::IdNotFound);
    let err = trie
        .emergency_isolate_member(&member_id("jan-jan-id"), member_key("jan-jan-mk"))
        .unwrap_err();
    assert_eq!(err, OrgMembersError::P2pKeyNotReplaced);
}

/// A delta anchored at `base` (which must be calculated) carrying `removed`
/// and `upserted`, sorted into canonical order.
fn forged_delta(
    base: &TestTrie,
    mut removed: Vec<MemberId>,
    mut upserted: Vec<MemberLeaf>,
) -> org_members::delta::Delta {
    let mut delta = base.calculate_delta(base).unwrap();
    removed.sort();
    upserted.sort_by(|a, b| a.id().cmp(b.id()));
    org_members::delta::test_support::delta_set_removed(&mut delta, removed);
    org_members::delta::test_support::delta_set_upserted(&mut delta, upserted);
    delta
}

/// `leaf` with its member key and devices replaced, everything else kept.
fn rekeyed(leaf: &MemberLeaf, mk: PersonPublicKey, devices: Vec<DevicePublicKey>) -> MemberLeaf {
    MemberLeaf::new(
        *leaf.id(),
        leaf.handle().clone(),
        mk,
        leaf.name().clone(),
        leaf.surname().clone(),
        devices,
    )
    .unwrap()
}

/// verifies: LLR-gjj6bx
///
/// Normal cases: a fresh member with fresh keys, and an existing member whose
/// device set changes while it keeps the member key it already held.
#[test]
fn apply_delta_accepts_distinct_keys() {
    let base = TestTrie::genesis(vec![alice(), jan_jan()]).unwrap();
    let jan = base.get(&member_id("jan-jan-id")).unwrap();
    let delta = forged_delta(
        &base,
        vec![],
        vec![
            bob(),
            rekeyed(&jan, member_key("jan-jan-mk"), vec![device_key("jan-jan-d2"), device_key("jan-jan-d3")]),
        ],
    );
    let candidate = base.apply_delta(&delta).unwrap();
    let root = candidate.root_hash();
    let trie = candidate.verify_against(&root).unwrap();
    assert_keys_unique(&trie);
}

/// verifies: LLR-gjj6bx
#[test]
fn apply_delta_rejects_upsert_sharing_a_key_with_the_record() {
    let base = TestTrie::genesis(vec![alice()]).unwrap();
    for (mk, dk) in [
        ("alice-mk", "eve-d1"), // member key held by alice
        ("eve-mk", "alice-d1"), // device enrolled under alice
        ("alice-d1", "eve-d1"), // member key equal to alice's device
        ("eve-mk", "alice-mk"), // device equal to alice's member key
    ] {
        let eve = keyed_leaf("eve", member_key(mk), vec![device_key(dk)]);
        let delta = forged_delta(&base, vec![], vec![eve]);
        let err = base.apply_delta(&delta).unwrap_err();
        assert_eq!(err, OrgMembersError::DuplicateKey, "member {mk}, device {dk}");
    }
}

/// verifies: LLR-gjj6bx
#[test]
fn apply_delta_rejects_two_upserts_sharing_a_key() {
    let base = TestTrie::genesis(vec![alice()]).unwrap();
    let eve = keyed_leaf("eve", member_key("shared"), vec![device_key("eve-d1")]);
    let ivy = keyed_leaf("ivy", member_key("ivy-mk"), vec![device_key("shared")]);
    let delta = forged_delta(&base, vec![], vec![eve, ivy]);
    assert_eq!(base.apply_delta(&delta).unwrap_err(), OrgMembersError::DuplicateKey);
}

/// verifies: LLR-gjj6bx
#[test]
fn apply_delta_rejects_upsert_whose_member_key_is_its_own_device() {
    let base = TestTrie::genesis(vec![alice()]).unwrap();
    let eve = keyed_leaf("eve", member_key("eve-d1"), vec![device_key("eve-d1")]);
    let delta = forged_delta(&base, vec![], vec![eve]);
    assert_eq!(base.apply_delta(&delta).unwrap_err(), OrgMembersError::DuplicateKey);
}

/// `current`'s change set against `base`, applied to `base` and verified
/// against `current`'s root. Panics if any step refuses.
fn assert_change_set_reproduces(base: &TestTrie, current: &TestTrie) {
    let (current, _) = current.recalculate().unwrap();
    let root = current.root_hash().unwrap();
    let delta = current.calculate_delta(base).unwrap();
    let candidate = base.apply_delta(&delta).unwrap();
    let trie = candidate.verify_against(&root).unwrap();
    assert_eq!(trie.root_hash().unwrap(), root);
    assert_keys_unique(&trie);
}

/// verifies: LLR-gjj6bx
///
/// The change set of two accepted direct operations -- delete a device under a
/// fresh member key, then rotate to the removed device's key, which is no
/// longer held -- gives the member a key held in the base record. Its result
/// holds every key once, so `apply_delta` accepts it: the result is checked,
/// not the base.
#[test]
fn apply_delta_accepts_change_set_of_delete_device_then_rotate_to_its_key() {
    let base = TestTrie::genesis(vec![jan_jan()]).unwrap();
    let jan = member_id("jan-jan-id");
    let current = base
        .delete_p2p_device(&jan, &device_key("jan-jan-d1"), member_key("jan-jan-mk2"))
        .unwrap()
        .rotate_p2p_key(&jan, member_key("jan-jan-d1"))
        .unwrap();
    assert_change_set_reproduces(&base, &current);
}

/// verifies: LLR-gjj6bx
///
/// The change set of deleting a member and then adding a new member under the
/// deleted member's member key: accepted step by step, so accepted as a
/// change set.
#[test]
fn apply_delta_accepts_change_set_of_delete_member_then_add_with_its_key() {
    let base = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let eve = keyed_leaf("eve", member_key("bob-mk"), vec![device_key("eve-d1")]);
    let current = base.delete_member(&member_id("bob-id")).unwrap().add_member(eve).unwrap();
    assert_change_set_reproduces(&base, &current);
}

/// verifies: LLR-gjj6bx
///
/// An existing member takes the member key of a member removed in the same
/// change set: the result holds the key once, so the change set is accepted.
#[test]
fn apply_delta_accepts_member_key_freed_by_a_removed_member() {
    let base = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let al = base.get(&member_id("alice-id")).unwrap();
    let up = rekeyed(&al, member_key("bob-mk"), vec![device_key("alice-d1")]);
    let delta = forged_delta(&base, vec![member_id("bob-id")], vec![up]);
    let candidate = base.apply_delta(&delta).unwrap();
    let root = candidate.root_hash();
    let trie = candidate.verify_against(&root).unwrap();
    assert_keys_unique(&trie);
    assert_eq!(trie.get(&member_id("alice-id")).unwrap().p2p_key(), &member_key("bob-mk"));
}

/// verifies: LLR-gjj6bx
///
/// Check order: the existing checks come first. A delta that would also share
/// a key reports the handle collision or the malformed shape.
#[test]
fn apply_delta_reports_earlier_refusals_before_shared_key() {
    let base = TestTrie::genesis(vec![alice()]).unwrap();
    // Same handle as alice, alice's member key.
    let twin = MemberLeaf::new(
        member_id("alice-twin-id"),
        h("alice"),
        member_key("alice-mk"),
        nm("A"),
        sn("B"),
        vec![device_key("twin-d1")],
    )
    .unwrap();
    let delta = forged_delta(&base, vec![], vec![twin]);
    assert_eq!(base.apply_delta(&delta).unwrap_err(), OrgMembersError::DuplicateHandle);

    // Unsorted upserts, both sharing alice's member key.
    let eve = keyed_leaf("eve", member_key("alice-mk"), vec![device_key("eve-d1")]);
    let ivy = keyed_leaf("ivy", member_key("alice-mk"), vec![device_key("ivy-d1")]);
    let mut delta = forged_delta(&base, vec![], vec![eve, ivy]);
    let mut rev = delta.upserted().to_vec();
    rev.reverse();
    org_members::delta::test_support::delta_set_upserted(&mut delta, rev);
    assert!(matches!(base.apply_delta(&delta).unwrap_err(), OrgMembersError::MalformedDelta(_)));
}

/// verifies: LLR-gjj6bx
///
/// Two present members swap member keys: rotate alice to a fresh key, bob to
/// alice's old key, alice to bob's old key -- each step accepted on the record
/// it ran on. The change set the software produces moves each key between two
/// upserted members, so the key check must release every outgoing key before
/// checking any incoming one (as the handle check does, LLR-n5t6bn), else the
/// receiver refuses the producer's lawful edit (REQ-wx3wpv). Reds when
/// `delta_key_index` re-indexes each upsert before releasing the next one's
/// keys (measured 2026-10-03).
#[test]
fn apply_delta_accepts_change_set_of_member_key_swap() {
    let base = TestTrie::genesis(vec![alice(), bob()]).unwrap();
    let (a, b) = (member_id("alice-id"), member_id("bob-id"));
    let current = base
        .rotate_p2p_key(&a, member_key("swap-k3"))
        .unwrap()
        .rotate_p2p_key(&b, member_key("alice-mk"))
        .unwrap()
        .rotate_p2p_key(&a, member_key("bob-mk"))
        .unwrap();
    assert_eq!(current.get(&a).unwrap().p2p_key(), &member_key("bob-mk"));
    assert_eq!(current.get(&b).unwrap().p2p_key(), &member_key("alice-mk"));
    assert_change_set_reproduces(&base, &current);
}

/// verifies: LLR-gjj6bx
///
/// A device key moves between two present members in one change set, in both
/// directions (so in both identifier orders): the holder drops the device
/// under a fresh member key, then the other member enrols it. Each step is
/// accepted on the record it ran on, so the change set is accepted. Reds when
/// `delta_key_index` re-indexes each upsert before releasing the next one's
/// keys (measured 2026-10-03).
#[test]
fn apply_delta_accepts_change_set_of_device_key_moving_between_members() {
    for (from, to) in [("alice", "bob"), ("bob", "alice")] {
        let base = TestTrie::genesis(vec![alice(), bob()]).unwrap();
        let (f, t) = (member_id(&format!("{from}-id")), member_id(&format!("{to}-id")));
        let moved = device_key(&format!("{from}-d1"));
        let current = base
            .add_p2p_device(&f, device_key(&format!("{from}-d2")))
            .unwrap()
            .delete_p2p_device(&f, &moved, member_key(&format!("{from}-mk2")))
            .unwrap()
            .add_p2p_device(&t, moved)
            .unwrap();
        assert!(current.get(&t).unwrap().p2p_devices().contains(&moved), "{from} -> {to}");
        assert_change_set_reproduces(&base, &current);
    }
}

/// verifies: LLR-gjj6bx, LLR-v6gfc7
///
/// The record a verified candidate becomes keeps the key index: a key the
/// delta introduced is refused afterwards.
#[test]
fn apply_delta_result_refuses_keys_the_delta_introduced() {
    let base = TestTrie::genesis(vec![alice()]).unwrap();
    let delta = forged_delta(&base, vec![], vec![bob()]);
    let candidate = base.apply_delta(&delta).unwrap();
    let root = candidate.root_hash();
    let trie = candidate.verify_against(&root).unwrap();
    let err = trie.rotate_p2p_key(&member_id("alice-id"), member_key("bob-d1")).unwrap_err();
    assert_eq!(err, OrgMembersError::DuplicateKey);
    // And a key the delta removed is free again.
    let delta = forged_delta(&trie, vec![member_id("bob-id")], vec![]);
    let candidate = trie.apply_delta(&delta).unwrap();
    let root = candidate.root_hash();
    let trie = candidate.verify_against(&root).unwrap();
    trie.rotate_p2p_key(&member_id("alice-id"), member_key("bob-d1")).unwrap();
}

/// verifies: REQ-t46uad, LLR-f3zrwd
#[test]
fn handle_query_reports_invalid_absent_and_held() {
    let trie = TestTrie::genesis(vec![alice()]).unwrap();

    // Invalid: refused before any lookup can run.
    assert!(matches!(Handle::parse("Alice"), Err(OrgMembersError::InvalidHandle(_))));

    // Valid, held by no member.
    let absent = Handle::parse("zoe").unwrap();
    assert_eq!(trie.get_by_handle(&absent), None);
    assert!(!trie.contains_handle(&absent));

    // Valid, held.
    let held = Handle::parse("alice").unwrap();
    assert_eq!(trie.get_by_handle(&held).map(|m| *m.id()), Some(member_id("alice-id")));
    assert!(trie.contains_handle(&held));
}

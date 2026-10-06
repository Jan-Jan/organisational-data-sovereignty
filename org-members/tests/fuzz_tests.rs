mod common;

use common::{default_level_count, device_key, member_id, member_key};
use org_members::hasher::Blake3Hasher;
use org_members::trie::OrgTrie;
use org_members::types::{Handle, DevicePublicKey, MemberId, Name, PersonPublicKey, MemberLeaf, Surname};
use proptest::prelude::*;

type TestTrie = OrgTrie<Blake3Hasher>;

const HANDLES: &[&str] = &[
    "alice", "bob", "charlie", "jan-jan", "diana", "eve", "frank", "grace",
    "hank", "iris", "jack", "kate",
];

fn arb_handle_idx() -> impl Strategy<Value = usize> {
    0..HANDLES.len()
}

/// `recalculate()` on a trie with pending changes, which must succeed.
fn recalculate_pending(
    trie: &TestTrie,
) -> Result<(TestTrie, org_members::delta::Delta), TestCaseError> {
    trie.recalculate()
        .map_err(|e| TestCaseError::fail(format!("recalculate of a pending trie failed: {:?}", e)))
}


fn make_member(handle: &str, variant: u8) -> Option<MemberLeaf> {
    let id = member_id(&format!("{}-id-{}", handle, variant));
    let mk = member_key(&format!("{}-mk-{}", handle, variant));
    let dk = device_key(&format!("{}-d-{}", handle, variant));
    MemberLeaf::new(id, Handle::parse(handle).ok()?, mk, nm("Test"), sn("User"), vec![dk]).ok()
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

// ============================================================
// Handle validation fuzzing
// ============================================================

proptest! {
    /// verifies: REQ-ds8ryr, REQ-h5ret5, LLR-h9gs32
    #[test]
    fn handle_validation_never_panics(s in "\\PC{0,64}") {
        if let Ok(normalized) = Handle::parse(&s) {
            prop_assert!(!normalized.as_str().is_empty());
            for ch in normalized.as_str().chars() {
                prop_assert!(!ch.is_uppercase(), "validated handle contains uppercase: {:?}", ch);
            }
            prop_assert!(!normalized.as_str().contains('.'), "validated handle contains '.'");
        }
    }
}

// ============================================================
// Trie operation invariants
// ============================================================

#[derive(Debug, Clone)]
enum Op {
    Insert(usize),
    Update(usize, u8),
    /// Update member at `id_idx`, retargeting their handle to `handle_idx`'s handle.
    /// Stress-tests the handle-collision-during-update path.
    UpdateRehandle(usize, usize),
    Delete(usize),
    Recalculate,
}

fn arb_op() -> impl Strategy<Value = Op> {
    prop_oneof![
        arb_handle_idx().prop_map(Op::Insert),
        (arb_handle_idx(), any::<u8>()).prop_map(|(idx, v)| Op::Update(idx, v)),
        (arb_handle_idx(), arb_handle_idx()).prop_map(|(a, b)| Op::UpdateRehandle(a, b)),
        arb_handle_idx().prop_map(Op::Delete),
        Just(Op::Recalculate),
    ]
}

proptest! {
    /// verifies: REQ-ds8ryr, LLR-h9gs32
    #[test]
    fn trie_ops_never_panic_and_count_consistent(ops in proptest::collection::vec(arb_op(), 0..30)) {
        let mut trie = TestTrie::genesis(vec![]).unwrap();

        for op in &ops {
            match op {
                Op::Insert(idx) => {
                    let handle = HANDLES[*idx];
                    if let Some(m) = make_member(handle, 0) {
                        if let Ok(new_trie) = trie.add_member(m) {
                            trie = new_trie;
                        }
                    }
                }
                Op::Update(idx, variant) => {
                    // Rotate the p2p_key (the most common update).
                    let id = member_id(&format!("{}-id-0", HANDLES[*idx]));
                    let mk = member_key(&format!("{}-mk-{}", HANDLES[*idx], *variant));
                    if let Ok(new_trie) = trie.rotate_p2p_key(&id, mk) {
                        trie = new_trie;
                    }
                }
                Op::UpdateRehandle(id_idx, handle_idx) => {
                    // Retarget the member's handle to another candidate handle.
                    // If handle_idx's handle is already taken by another member,
                    // this should fail with DuplicateHandle (must not panic).
                    let id = member_id(&format!("{}-id-0", HANDLES[*id_idx]));
                    let new_handle = HANDLES[*handle_idx];
                    if let Ok(new_trie) = trie.update_handle(&id, h(new_handle)) {
                        trie = new_trie;
                    }
                }
                Op::Delete(idx) => {
                    let id = member_id(&format!("{}-id-0", HANDLES[*idx]));
                    if let Ok(new_trie) = trie.delete_member(&id) {
                        trie = new_trie;
                    }
                }
                Op::Recalculate => {
                    // A pending trie recalculates; a calculated one is refused.
                    if trie.has_pending_changes() {
                        let (new_trie, _) = recalculate_pending(&trie)?;
                        trie = new_trie;
                    } else {
                        prop_assert_eq!(
                            trie.recalculate().unwrap_err(),
                            OrgMembersError::HashesAlreadyCalculated
                        );
                    }
                }
            }

            let actual = trie.members().len();
            prop_assert_eq!(
                trie.member_count(), actual,
                "member_count={} but actual members={}",
                trie.member_count(), actual,
            );
        }
    }
}

// ============================================================
// Delta roundtrip invariant
// ============================================================

#[derive(Debug, Clone)]
enum DeltaOp {
    Insert(usize),
    Delete(usize),
}

fn arb_delta_op() -> impl Strategy<Value = DeltaOp> {
    prop_oneof![
        arb_handle_idx().prop_map(DeltaOp::Insert),
        arb_handle_idx().prop_map(DeltaOp::Delete),
    ]
}

proptest! {
    /// verifies: REQ-4umsuz
    #[test]
    fn delta_roundtrip(
        initial_indices in proptest::collection::vec(arb_handle_idx(), 0..6),
        ops in proptest::collection::vec(arb_delta_op(), 1..10),
    ) {
        let mut seen = vec![false; HANDLES.len()];
        let mut initial = Vec::new();
        for idx in &initial_indices {
            if !seen[*idx] {
                if let Some(m) = make_member(HANDLES[*idx], 0) {
                    initial.push(m);
                    seen[*idx] = true;
                }
            }
        }

        let trie_a = TestTrie::genesis(initial).unwrap();

        let mut trie_b = trie_a.clone();
        for op in &ops {
            match op {
                DeltaOp::Insert(idx) => {
                    if let Some(m) = make_member(HANDLES[*idx], 0) {
                        if let Ok(t) = trie_b.add_member(m) {
                            trie_b = t;
                        }
                    }
                }
                DeltaOp::Delete(idx) => {
                    let id = member_id(&format!("{}-id-0", HANDLES[*idx]));
                    if let Ok(t) = trie_b.delete_member(&id) {
                        trie_b = t;
                    }
                }
            }
        }

        // No op succeeded: trie_b is still the calculated genesis, nothing to send.
        if !trie_b.has_pending_changes() {
            return Ok(());
        }
        let (trie_b, delta) = recalculate_pending(&trie_b)?;

        if delta.is_empty() {
            return Ok(());
        }

        let candidate = match trie_a.apply_delta(&delta) {
            Ok(c) => c,
            Err(_) => return Ok(()),
        };

        let verified = candidate
            .verify_against(&trie_b.root_hash().unwrap())
            .map_err(|e| TestCaseError::fail(format!("delta roundtrip verification failed: {:?}", e)))?;

        prop_assert_eq!(verified.root_hash().unwrap(), trie_b.root_hash().unwrap());
        prop_assert_eq!(verified.member_count(), trie_b.member_count());
    }
}

// ============================================================
// Diff roundtrip invariant
// ============================================================

proptest! {
    /// verifies: LLR-h9gs32
    #[test]
    fn calculate_delta_roundtrip(
        initial_indices in proptest::collection::vec(arb_handle_idx(), 0..6),
        ops in proptest::collection::vec(arb_delta_op(), 1..10),
    ) {
        let mut seen = vec![false; HANDLES.len()];
        let mut initial = Vec::new();
        for idx in &initial_indices {
            if !seen[*idx] {
                if let Some(m) = make_member(HANDLES[*idx], 0) {
                    initial.push(m);
                    seen[*idx] = true;
                }
            }
        }

        let trie_a = TestTrie::genesis(initial).unwrap();

        let mut trie_b = trie_a.clone();
        for op in &ops {
            match op {
                DeltaOp::Insert(idx) => {
                    if let Some(m) = make_member(HANDLES[*idx], 0) {
                        if let Ok(t) = trie_b.add_member(m) {
                            trie_b = t;
                        }
                    }
                }
                DeltaOp::Delete(idx) => {
                    let id = member_id(&format!("{}-id-0", HANDLES[*idx]));
                    if let Ok(t) = trie_b.delete_member(&id) {
                        trie_b = t;
                    }
                }
            }
        }

        // No op succeeded: trie_b is still the calculated genesis, nothing to diff.
        if !trie_b.has_pending_changes() {
            return Ok(());
        }
        let (trie_b, _) = recalculate_pending(&trie_b)?;

        let diff_delta = match trie_b.calculate_delta(&trie_a) {
            Ok(d) => d,
            Err(_) => return Ok(()),
        };

        if diff_delta.is_empty() {
            return Ok(());
        }

        let candidate = match trie_a.apply_delta(&diff_delta) {
            Ok(c) => c,
            Err(_) => return Ok(()),
        };

        let verified = candidate
            .verify_against(&trie_b.root_hash().unwrap())
            .map_err(|e| TestCaseError::fail(format!("diff roundtrip verification failed: {:?}", e)))?;

        prop_assert_eq!(verified.root_hash().unwrap(), trie_b.root_hash().unwrap());
        prop_assert_eq!(verified.member_count(), trie_b.member_count());
    }
}

// ============================================================
// Immutability invariant
// ============================================================

proptest! {
    /// verifies: REQ-d3prca
    #[test]
    fn mutations_preserve_original(
        initial_indices in proptest::collection::vec(arb_handle_idx(), 1..4),
        op_idx in arb_handle_idx(),
    ) {
        let mut seen = vec![false; HANDLES.len()];
        let mut initial = Vec::new();
        for idx in &initial_indices {
            if !seen[*idx] {
                if let Some(m) = make_member(HANDLES[*idx], 0) {
                    initial.push(m);
                    seen[*idx] = true;
                }
            }
        }

        if initial.is_empty() {
            return Ok(());
        }

        let original = TestTrie::genesis(initial).unwrap();
        let original_root = original.root_hash().unwrap();
        let original_count = original.member_count();

        let handle = HANDLES[op_idx];
        if let Some(m) = make_member(handle, 99) {
            let _ = original.add_member(m);
        }

        let id = member_id(&format!("{}-id-0", handle));
        let _ = original.delete_member(&id);

        prop_assert_eq!(original.root_hash().unwrap(), original_root);
        prop_assert_eq!(original.member_count(), original_count);
    }
}

// ============================================================
// Handle uniqueness after release: whatever a canonical delta
// removes and re-handles, apply_delta never admits a post-state
// in which two members share a handle or a skeleton.
// ============================================================

proptest! {
    /// verifies: LLR-n5t6bn, LLR-juxk9q
    ///
    /// Reds when only `apply_delta`'s `DuplicateHandle` return is deleted
    /// (measured 2026-10-03).
    #[test]
    fn apply_delta_never_admits_a_handle_collision(
        removed_mask in 0u8..16,
        claims in proptest::collection::vec((arb_handle_idx(), arb_handle_idx(), any::<u8>()), 1..6),
    ) {
        let initial: Vec<_> = HANDLES
            .iter()
            .take(4)
            .filter_map(|h| make_member(h, 0))
            .collect();
        let base = TestTrie::genesis(initial.clone()).unwrap();

        let mut removed: Vec<MemberId> = initial
            .iter()
            .enumerate()
            .filter(|(i, _)| removed_mask & (1 << i) != 0)
            .map(|(_, m)| *m.id())
            .collect();
        removed.sort();

        // Each claim gives the member seeded by `id_idx` the handle at
        // `handle_idx`, existing or new, keeping the delta canonical.
        let mut upserted: Vec<MemberLeaf> = Vec::new();
        for (id_idx, handle_idx, variant) in &claims {
            let seed = HANDLES[*id_idx];
            let id = member_id(&format!("{}-id-0", seed));
            if removed.contains(&id) || upserted.iter().any(|m| *m.id() == id) {
                continue;
            }
            let leaf = MemberLeaf::new(
                id,
                h(HANDLES[*handle_idx]),
                member_key(&format!("{}-mk-{}", seed, variant)),
                nm("Test"),
                sn("User"),
                vec![device_key(&format!("{}-d-0", seed))],
            )
            .unwrap();
            if base.get(&id).as_ref() == Some(&leaf) {
                continue;
            }
            upserted.push(leaf);
        }
        upserted.sort_by(|a, b| a.id().cmp(b.id()));
        if removed.is_empty() && upserted.is_empty() {
            return Ok(());
        }

        let (_, mut delta) = base
            .add_member(make_member("zoe", 0).unwrap())
            .unwrap()
            .recalculate()
            .unwrap();
        org_members::delta::test_support::delta_set_removed(&mut delta, removed);
        org_members::delta::test_support::delta_set_upserted(&mut delta, upserted);

        let candidate = match base.apply_delta(&delta) {
            Ok(c) => c,
            Err(_) => return Ok(()),
        };
        let root = candidate.root_hash();
        let after = candidate
            .verify_against(&root)
            .map_err(|e| TestCaseError::fail(format!("candidate failed its own root: {:?}", e)))?;

        let members = after.members();
        let mut skeletons: Vec<String> = members
            .iter()
            .map(|m| unicode_security::confusable_detection::skeleton(m.handle().as_str()).collect())
            .collect();
        skeletons.sort();
        skeletons.dedup();
        prop_assert_eq!(
            skeletons.len(),
            members.len(),
            "apply_delta admitted a handle collision: {:?}",
            members.iter().map(|m| m.handle().to_string()).collect::<Vec<_>>(),
        );
    }
}

// ============================================================
// H-1 / H-2 canonicality fuzz: every non-canonical mutation of
// an honest delta must be rejected by apply_delta with
// OrgMembersError::MalformedDelta.
// ============================================================

use org_members::OrgMembersError;

#[derive(Debug, Clone)]
enum Mutator {
    /// Append a stale (unused) removal.
    AppendStaleRemoval,
    /// Duplicate the last entry in `removed`.
    DuplicateLastRemoved,
    /// Duplicate the last entry in `upserted`.
    DuplicateLastUpserted,
    /// Reverse `removed` (force unsorted, only meaningful when len >= 2).
    ReverseRemoved,
    /// Reverse `upserted` (force unsorted, only meaningful when len >= 2).
    ReverseUpserted,
    /// Append a no-op upsert (clone of a current trie leaf that is NOT already
    /// in the upsert list; chooses the first handle from HANDLES that fits).
    AppendNoopUpsert,
    /// Move the first removed id into upserted as a leaf-clone (id in both sides).
    MoveRemovedIntoUpserted,
}

fn arb_mutator() -> impl Strategy<Value = Mutator> {
    prop_oneof![
        Just(Mutator::AppendStaleRemoval),
        Just(Mutator::DuplicateLastRemoved),
        Just(Mutator::DuplicateLastUpserted),
        Just(Mutator::ReverseRemoved),
        Just(Mutator::ReverseUpserted),
        Just(Mutator::AppendNoopUpsert),
        Just(Mutator::MoveRemovedIntoUpserted),
    ]
}

proptest! {
    /// verifies: LLR-8jttpb, LLR-h9gs32
    #[test]
    fn delta_canonicality_fuzz(
        seed_ops in proptest::collection::vec(arb_delta_op(), 1..8),
        mutator in arb_mutator(),
    ) {
        // 1. Build a honest base trie and honest delta.
        // HANDLES[0..4] are unique, so no dedup needed.
        let initial: Vec<_> = HANDLES
            .iter()
            .take(4)
            .filter_map(|h| make_member(h, 0))
            .collect();
        let base = TestTrie::genesis(initial).unwrap();

        let mut work = base.clone();
        for op in &seed_ops {
            match op {
                DeltaOp::Insert(idx) => {
                    if let Some(m) = make_member(HANDLES[*idx], 0) {
                        if let Ok(t) = work.add_member(m) { work = t; }
                    }
                }
                DeltaOp::Delete(idx) => {
                    let id = member_id(&format!("{}-id-0", HANDLES[*idx]));
                    if let Ok(t) = work.delete_member(&id) { work = t; }
                }
            }
        }
        // No seed op succeeded: work is still the calculated base, no delta.
        if !work.has_pending_changes() {
            return Ok(());
        }
        let (_target, mut delta) = recalculate_pending(&work)?;
        if delta.is_empty() {
            return Ok(());
        }

        // 2. Apply the mutator. If the mutation can't be applied to this delta
        //    shape, early-return Ok(()) — proptest will still explore shapes
        //    where the mutator fires.
        match mutator {
            Mutator::AppendStaleRemoval => {
                let ghost = member_id("zzz-fuzz-ghost-id-xyzzy");
                let mut r = delta.removed().to_vec();
                if r.contains(&ghost) || base.contains(&ghost) {
                    return Ok(());
                }
                r.push(ghost);
                r.sort();
                org_members::delta::test_support::delta_set_removed(&mut delta, r);
            }
            Mutator::DuplicateLastRemoved => {
                let r = delta.removed();
                if r.is_empty() { return Ok(()); }
                let mut new = r.to_vec();
                new.push(*r.last().unwrap());
                org_members::delta::test_support::delta_set_removed(&mut delta, new);
            }
            Mutator::DuplicateLastUpserted => {
                let u = delta.upserted();
                if u.is_empty() { return Ok(()); }
                let mut new: Vec<MemberLeaf> = u.to_vec();
                new.push(u.last().unwrap().clone());
                org_members::delta::test_support::delta_set_upserted(&mut delta, new);
            }
            Mutator::ReverseRemoved => {
                let r = delta.removed();
                if r.len() < 2 { return Ok(()); }
                let mut new = r.to_vec();
                new.reverse();
                org_members::delta::test_support::delta_set_removed(&mut delta, new);
            }
            Mutator::ReverseUpserted => {
                let u = delta.upserted();
                if u.len() < 2 { return Ok(()); }
                let mut new: Vec<MemberLeaf> = u.to_vec();
                new.reverse();
                org_members::delta::test_support::delta_set_upserted(&mut delta, new);
            }
            Mutator::AppendNoopUpsert => {
                let removed: Vec<_> = delta.removed().to_vec();
                let upserted_ids: Vec<_> = delta.upserted().iter().map(|m| *m.id()).collect();
                let leaf = match base.members().into_iter().find(|m| {
                    !removed.contains(m.id()) && !upserted_ids.contains(m.id())
                }) {
                    Some(l) => l,
                    None => return Ok(()),
                };
                let mut new: Vec<MemberLeaf> = delta.upserted().to_vec();
                new.push(leaf);
                new.sort_by(|a, b| a.id().cmp(b.id()));
                org_members::delta::test_support::delta_set_upserted(&mut delta, new);
            }
            Mutator::MoveRemovedIntoUpserted => {
                let r = delta.removed();
                if r.is_empty() { return Ok(()); }
                let collide_id = r[0];
                let leaf = match base.get(&collide_id) {
                    Some(l) => l,
                    None => return Ok(()),
                };
                let mut new: Vec<MemberLeaf> = delta.upserted().to_vec();
                new.push(leaf);
                new.sort_by(|a, b| a.id().cmp(b.id()));
                org_members::delta::test_support::delta_set_upserted(&mut delta, new);
            }
        }

        // 3. apply_delta must now reject with MalformedDelta.
        let err = match base.apply_delta(&delta) {
            Ok(_) => return Err(TestCaseError::fail(
                "apply_delta accepted a non-canonical delta (mutator ran but check missed)"
            )),
            Err(e) => e,
        };
        prop_assert!(
            matches!(err, OrgMembersError::MalformedDelta(_)),
            "expected MalformedDelta, got {:?}", err,
        );
    }
}

// ============================================================
// Device-removal key replacement: removing a device or isolating
// a member must replace the member-as-a-group key, or change
// nothing (PR-zz4exm). Routine rotation is held to the same
// contract (LLR-k89ahd, owner ruling 2026-10-03).
// ============================================================

/// Members in the device-removal pool. Small, so ops collide often.
const REMOVAL_POOL: usize = 3;
/// Device seeds per member. Wider than MAX_DEVICES so some ops name a
/// device the member does not hold, or overflow the slots.
const DEVICE_POOL: usize = 6;

/// The replacement key an op supplies.
#[derive(Debug, Clone)]
enum KeyChoice {
    /// The member's key at the time the op runs.
    Current,
    /// `pool_key(member, v)`. Variant 0 is every member's genesis
    /// key, so this also lands on the current key at times.
    Seeded(u8),
}

fn arb_key_choice() -> impl Strategy<Value = KeyChoice> {
    prop_oneof![
        2 => Just(KeyChoice::Current),
        3 => (0..4u8).prop_map(KeyChoice::Seeded),
    ]
}

#[derive(Debug, Clone)]
enum DeviceOp {
    DeleteDevice(usize, usize, KeyChoice),
    Isolate(usize, KeyChoice),
    AddDevice(usize, usize),
    Rotate(usize, KeyChoice),
}

fn arb_device_op() -> impl Strategy<Value = DeviceOp> {
    let member = 0..REMOVAL_POOL;
    let device = 0..DEVICE_POOL;
    prop_oneof![
        3 => (member.clone(), device.clone(), arb_key_choice())
            .prop_map(|(m, d, k)| DeviceOp::DeleteDevice(m, d, k)),
        2 => (member.clone(), arb_key_choice()).prop_map(|(m, k)| DeviceOp::Isolate(m, k)),
        2 => (member.clone(), device).prop_map(|(m, d)| DeviceOp::AddDevice(m, d)),
        2 => (member, arb_key_choice()).prop_map(|(m, k)| DeviceOp::Rotate(m, k)),
    ]
}

impl DeviceOp {
    fn member_idx(&self) -> usize {
        match self {
            DeviceOp::DeleteDevice(m, _, _)
            | DeviceOp::Isolate(m, _)
            | DeviceOp::AddDevice(m, _)
            | DeviceOp::Rotate(m, _) => *m,
        }
    }
}

fn pool_id(member_idx: usize) -> MemberId {
    member_id(&format!("{}-id-0", HANDLES[member_idx]))
}

fn pool_device(member_idx: usize, device_idx: usize) -> DevicePublicKey {
    device_key(&format!("{}-d-{}", HANDLES[member_idx], device_idx))
}

fn pool_key(member_idx: usize, variant: u8) -> PersonPublicKey {
    member_key(&format!("{}-mk-{}", HANDLES[member_idx], variant))
}

/// A pool member enrolled with devices 0..3 and genesis key variant 0.
fn pool_member(member_idx: usize) -> MemberLeaf {
    let devices = (0..3).map(|d| pool_device(member_idx, d)).collect();
    MemberLeaf::new(
        pool_id(member_idx),
        h(HANDLES[member_idx]),
        pool_key(member_idx, 0),
        nm("Test"),
        sn("User"),
        devices,
    )
    .unwrap()
}

fn resolve_key(choice: &KeyChoice, member_idx: usize, current: PersonPublicKey) -> PersonPublicKey {
    match choice {
        KeyChoice::Current => current,
        KeyChoice::Seeded(v) => pool_key(member_idx, *v),
    }
}

proptest! {
    /// verifies: LLR-s97ywt, LLR-w92psx, LLR-k89ahd
    ///
    /// Every successful `delete_p2p_device` / `emergency_isolate_member` /
    /// `rotate_p2p_key` installs the supplied key and that key differs from
    /// the one it replaces. `P2pKeyNotReplaced` is returned only when the
    /// supplied key equalled the current key. A refusal is atomic by
    /// construction — the operations take `&self` and return
    /// `Result<Self, _>`, so an `Err` carries no trie — so the property checks
    /// the error, not a re-read of the unchanged input trie.
    #[test]
    fn device_removal_never_keeps_key(
        ops in proptest::collection::vec(arb_device_op(), 1..40),
    ) {
        let mut trie = TestTrie::genesis((0..REMOVAL_POOL).map(pool_member).collect()).unwrap();

        for op in &ops {
            let member_idx = op.member_idx();
            let id = pool_id(member_idx);
            let before = trie.get(&id).unwrap();
            let current = *before.p2p_key();

            let (result, supplied) = match op {
                DeviceOp::DeleteDevice(_, d, choice) => {
                    let key = resolve_key(choice, member_idx, current);
                    (trie.delete_p2p_device(&id, &pool_device(member_idx, *d), key), Some(key))
                }
                DeviceOp::Isolate(_, choice) => {
                    let key = resolve_key(choice, member_idx, current);
                    (trie.emergency_isolate_member(&id, key), Some(key))
                }
                DeviceOp::AddDevice(_, d) => {
                    (trie.add_p2p_device(&id, pool_device(member_idx, *d)), None)
                }
                DeviceOp::Rotate(_, choice) => {
                    let key = resolve_key(choice, member_idx, current);
                    (trie.rotate_p2p_key(&id, key), Some(key))
                }
            };

            // Only the key-replacing ops carry the key-replacement property.
            let Some(supplied) = supplied else {
                if let Ok(next) = result {
                    trie = next;
                }
                continue;
            };

            match result {
                Ok(next) => {
                    let after = next.get(&id).unwrap();
                    prop_assert!(
                        *after.p2p_key() != current,
                        "{:?} succeeded but kept the member's key", op,
                    );
                    prop_assert!(*after.p2p_key() == supplied, "{:?} installed another key", op);
                    match op {
                        DeviceOp::DeleteDevice(_, d, _) => {
                            prop_assert!(!after.has_p2p_device(&pool_device(member_idx, *d)));
                            prop_assert_eq!(after.p2p_device_count(), before.p2p_device_count() - 1);
                        }
                        DeviceOp::Rotate(..) => {
                            prop_assert_eq!(after.p2p_device_count(), before.p2p_device_count());
                        }
                        _ => prop_assert_eq!(after.p2p_device_count(), 0),
                    }
                    trie = next;
                }
                Err(OrgMembersError::P2pKeyNotReplaced) => {
                    prop_assert!(
                        supplied == current,
                        "{:?} refused with P2pKeyNotReplaced for a fresh key", op,
                    );
                    // Check order: an absent device reports DeviceNotFound
                    // before any key comparison.
                    if let DeviceOp::DeleteDevice(_, d, _) = op {
                        prop_assert!(
                            before.has_p2p_device(&pool_device(member_idx, *d)),
                            "{:?} reported P2pKeyNotReplaced for an absent device", op,
                        );
                    }
                }
                Err(err) => {
                    // Pool members are never deleted, so no IdNotFound, and
                    // pool keys are never held by another member or as a
                    // device, so no DuplicateKey (keys_stay_unique covers
                    // that): the only other error is a delete of an absent
                    // device, whatever key was supplied.
                    let DeviceOp::DeleteDevice(_, d, _) = op else {
                        return Err(TestCaseError::fail(format!("{:?} failed with {:?}", op, err)));
                    };
                    prop_assert_eq!(err, OrgMembersError::DeviceNotFound);
                    prop_assert!(!before.has_p2p_device(&pool_device(member_idx, *d)));
                }
            }
        }
    }
}

// ============================================================
// Key uniqueness (LLR-v6gfc7, LLR-fym7dy, LLR-gjj6bx): every key in
// the organisation is held in exactly one place. Member and device
// keys are drawn from one small shared pool, so ops collide often,
// in every direction (member/member, device/device, member/device).
// ============================================================

/// Members in the uniqueness pool.
const UNIQ_MEMBERS: usize = 4;
/// Shared key seeds. Small, so a supplied key is often already held.
const UNIQ_KEYS: usize = 10;

fn uniq_id(m: usize) -> MemberId {
    member_id(&format!("uniq-{}-id", HANDLES[m]))
}

fn uniq_bytes(k: usize) -> [u8; 32] {
    *uniq_mk(k).as_bytes()
}

fn uniq_mk(k: usize) -> PersonPublicKey {
    member_key(&format!("uniq-key-{}", k))
}

fn uniq_dk(k: usize) -> DevicePublicKey {
    device_key(&format!("uniq-key-{}", k))
}

fn uniq_leaf(m: usize, mk: usize, devices: &[usize]) -> Option<MemberLeaf> {
    MemberLeaf::new(
        uniq_id(m),
        h(HANDLES[m]),
        uniq_mk(mk),
        nm("Test"),
        sn("User"),
        devices.iter().map(|d| uniq_dk(*d)).collect(),
    )
    .ok()
}

/// Every key the trie holds, with repetitions.
fn held_keys(trie: &TestTrie) -> Vec<[u8; 32]> {
    let mut keys = Vec::new();
    for m in trie.members() {
        keys.extend(leaf_key_bytes(&m));
    }
    keys
}

fn leaf_key_bytes(leaf: &MemberLeaf) -> Vec<[u8; 32]> {
    let mut keys = vec![*leaf.p2p_key().as_bytes()];
    keys.extend(leaf.p2p_devices().iter().map(|d| *d.as_bytes()));
    keys
}

/// Every key, with repetitions, of the record a change set removing `removed`
/// and upserting `leaf` produces from `base`.
fn delta_result_keys(base: &TestTrie, removed: &[MemberId], leaf: &MemberLeaf) -> Vec<[u8; 32]> {
    let mut keys: Vec<[u8; 32]> = base
        .members()
        .iter()
        .filter(|x| x.id() != leaf.id() && !removed.contains(x.id()))
        .flat_map(leaf_key_bytes)
        .collect();
    keys.extend(leaf_key_bytes(leaf));
    keys
}

fn has_repeat(keys: &[[u8; 32]]) -> bool {
    let mut sorted = keys.to_vec();
    sorted.sort();
    sorted.windows(2).any(|w| w[0] == w[1])
}

#[derive(Debug, Clone)]
enum UniqOp {
    AddMember(usize, usize, usize, usize),
    DeleteMember(usize),
    AddDevice(usize, usize),
    Rotate(usize, usize),
    /// Delete the member's `(d mod count)`-th device, replacing the key with `k`.
    DeleteDevice(usize, usize, usize),
    Isolate(usize, usize),
    /// Upsert member `m` through a forged delta against the recalculated
    /// record: member key `mk`, and, when it holds no device or is new, the
    /// single device `dk`. Exercises apply_delta.
    DeltaUpsert(usize, usize, usize),
    /// Remove member `m` and give member `n` member `m`'s member key, in one
    /// delta: the key was held in the base, and the result holds it once.
    DeltaHandOver(usize, usize),
}

fn arb_uniq_op() -> impl Strategy<Value = UniqOp> {
    let m = 0..UNIQ_MEMBERS;
    let k = 0..UNIQ_KEYS;
    prop_oneof![
        2 => (m.clone(), k.clone(), k.clone(), k.clone())
            .prop_map(|(m, a, b, c)| UniqOp::AddMember(m, a, b, c)),
        1 => m.clone().prop_map(UniqOp::DeleteMember),
        2 => (m.clone(), k.clone()).prop_map(|(m, k)| UniqOp::AddDevice(m, k)),
        2 => (m.clone(), k.clone()).prop_map(|(m, k)| UniqOp::Rotate(m, k)),
        2 => (m.clone(), 0..4usize, k.clone()).prop_map(|(m, d, k)| UniqOp::DeleteDevice(m, d, k)),
        1 => (m.clone(), k.clone()).prop_map(|(m, k)| UniqOp::Isolate(m, k)),
        2 => (m.clone(), k.clone(), k).prop_map(|(m, a, b)| UniqOp::DeltaUpsert(m, a, b)),
        1 => (m.clone(), m).prop_map(|(a, b)| UniqOp::DeltaHandOver(a, b)),
    ]
}

/// `trie` if it has no pending changes (genesis, or a `verify_against()`
/// result), otherwise the trie `recalculate()` returns.
fn calculated(trie: &TestTrie) -> TestTrie {
    if trie.has_pending_changes() {
        trie.recalculate().unwrap().0
    } else {
        trie.clone()
    }
}

/// A delta against `base` (calculated) with the given removals and upserts.
fn uniq_delta(
    base: &TestTrie,
    removed: Vec<MemberId>,
    upserted: Vec<MemberLeaf>,
) -> org_members::delta::Delta {
    let mut delta = base.calculate_delta(base).unwrap();
    org_members::delta::test_support::delta_set_removed(&mut delta, removed);
    org_members::delta::test_support::delta_set_upserted(&mut delta, upserted);
    delta
}

proptest! {
    /// verifies: LLR-v6gfc7, LLR-fym7dy, LLR-gjj6bx
    ///
    /// After every successful operation each key is held in exactly one
    /// place, and a key-replacing direct operation never installed a key
    /// that was held before it. `DuplicateKey` is returned only when a key
    /// was genuinely held — for a direct operation, before it or twice in the
    /// record it would produce; for a change set, twice in the record it
    /// would produce (LLR-gjj6bx checks the result only). And after every
    /// step, the change set of everything accepted so far, applied to the
    /// genesis record, is accepted and reproduces the current root.
    #[test]
    fn keys_stay_unique(
        initial in proptest::collection::vec((0..UNIQ_KEYS, 0..UNIQ_KEYS), 1..=UNIQ_MEMBERS),
        ops in proptest::collection::vec(arb_uniq_op(), 1..40),
    ) {
        // Genesis itself: refused exactly when the members share a key.
        let leaves: Vec<MemberLeaf> = initial
            .iter()
            .enumerate()
            .filter_map(|(m, (mk, dk))| uniq_leaf(m, *mk, &[*dk]))
            .collect();
        let all: Vec<[u8; 32]> = leaves.iter().flat_map(leaf_key_bytes).collect();
        let mut trie = match TestTrie::genesis(leaves) {
            Ok(t) => {
                prop_assert!(!has_repeat(&all), "genesis accepted a shared key");
                t
            }
            Err(e) => {
                prop_assert_eq!(e, OrgMembersError::DuplicateKey);
                prop_assert!(has_repeat(&all), "genesis refused distinct keys");
                return Ok(());
            }
        };
        let start = trie.clone();

        for op in &ops {
            let held = held_keys(&trie);
            let is_held = |k: &[u8; 32]| held.contains(k);

            // `genuine`: the key-uniqueness rule really does refuse this op.
            // `fresh`: the keys a success must not have found held before.
            let (result, genuine, fresh): (Result<TestTrie, OrgMembersError>, bool, Vec<[u8; 32]>) = match op {
                UniqOp::AddMember(m, mk, d1, d2) => {
                    let Some(leaf) = uniq_leaf(*m, *mk, &[*d1, *d2]) else { continue };
                    let keys = leaf_key_bytes(&leaf);
                    let genuine = has_repeat(&keys) || keys.iter().any(is_held);
                    (trie.add_member(leaf), genuine, keys)
                }
                UniqOp::DeleteMember(m) => (trie.delete_member(&uniq_id(*m)), false, vec![]),
                UniqOp::AddDevice(m, k) => {
                    (trie.add_p2p_device(&uniq_id(*m), uniq_dk(*k)), is_held(&uniq_bytes(*k)), vec![uniq_bytes(*k)])
                }
                UniqOp::Rotate(m, k) => {
                    (trie.rotate_p2p_key(&uniq_id(*m), uniq_mk(*k)), is_held(&uniq_bytes(*k)), vec![uniq_bytes(*k)])
                }
                UniqOp::DeleteDevice(m, d, k) => {
                    let Some(leaf) = trie.get(&uniq_id(*m)) else { continue };
                    let devices = leaf.p2p_devices();
                    if devices.is_empty() { continue; }
                    let device = devices[*d % devices.len()];
                    (
                        trie.delete_p2p_device(&uniq_id(*m), &device, uniq_mk(*k)),
                        is_held(&uniq_bytes(*k)),
                        vec![uniq_bytes(*k)],
                    )
                }
                UniqOp::Isolate(m, k) => {
                    (trie.emergency_isolate_member(&uniq_id(*m), uniq_mk(*k)), is_held(&uniq_bytes(*k)), vec![uniq_bytes(*k)])
                }
                UniqOp::DeltaUpsert(m, mk, dk) => {
                    let base = calculated(&trie);
                    let old = base.get(&uniq_id(*m));
                    let leaf = match &old {
                        Some(o) if o.p2p_device_count() > 0 => MemberLeaf::new(
                            *o.id(), o.handle().clone(), uniq_mk(*mk), o.name().clone(), o.surname().clone(), o.p2p_devices().to_vec(),
                        ).ok(),
                        _ => uniq_leaf(*m, *mk, &[*dk]),
                    };
                    let Some(leaf) = leaf else { continue };
                    // A change set is judged on the record it produces only.
                    let genuine = has_repeat(&delta_result_keys(&base, &[], &leaf));
                    let delta = uniq_delta(&base, vec![], vec![leaf]);
                    let result = base
                        .apply_delta(&delta)
                        .and_then(|c| { let r = c.root_hash(); c.verify_against(&r) });
                    (result, genuine, vec![])
                }
                UniqOp::DeltaHandOver(from, to) => {
                    if from == to { continue; }
                    let base = calculated(&trie);
                    let (Some(gone), Some(kept)) = (base.get(&uniq_id(*from)), base.get(&uniq_id(*to))) else { continue };
                    if kept.p2p_device_count() == 0 { continue; }
                    let Ok(leaf) = MemberLeaf::new(
                        *kept.id(), kept.handle().clone(), *gone.p2p_key(), kept.name().clone(), kept.surname().clone(), kept.p2p_devices().to_vec(),
                    ) else { continue };
                    // The key was held in the base, by `from`; only the
                    // resulting record counts.
                    let genuine = has_repeat(&delta_result_keys(&base, &[*gone.id()], &leaf));
                    let delta = uniq_delta(&base, vec![*gone.id()], vec![leaf]);
                    let result = base
                        .apply_delta(&delta)
                        .and_then(|c| { let r = c.root_hash(); c.verify_against(&r) });
                    (result, genuine, vec![])
                }
            };

            match result {
                Ok(next) => {
                    prop_assert!(!has_repeat(&held_keys(&next)), "{:?} left a key held twice", op);
                    prop_assert!(!genuine, "{:?} succeeded with a key already held", op);
                    for k in &fresh {
                        prop_assert!(!is_held(k), "{:?} installed a key held before it", op);
                    }
                    trie = next;
                }
                Err(OrgMembersError::DuplicateKey) => {
                    prop_assert!(genuine, "{:?} refused with DuplicateKey but no key was held", op);
                }
                Err(_) => {}
            }

            // Whatever sequence was accepted, its change set against genesis
            // is accepted and reproduces the root.
            let current = calculated(&trie);
            let root = current.root_hash().unwrap();
            let delta = current.calculate_delta(&start).unwrap();
            let replayed = start
                .apply_delta(&delta)
                .and_then(|c| c.verify_against(&root));
            prop_assert!(replayed.is_ok(), "change set after {:?} refused: {:?}", op, replayed.err());
            prop_assert_eq!(replayed.unwrap().root_hash().unwrap(), root);
        }
    }
}

// ============================================================
// Absence proofs (SDD-57vaj4)
// ============================================================

/// Bytes in an absence proof's wire form: a default-sibling map, siblings and
/// an ending. The sibling count is either the one the map implies or any count
/// up to past the 256 bound, so decoding succeeds often enough for `verify` to
/// run. Uniformly random bytes almost never decode to a proof.
fn arb_wire_proof() -> impl Strategy<Value = Vec<u8>> {
    let ending = prop_oneof![
        Just(None),
        arb_handle_idx().prop_map(|index| make_member(HANDLES[index], 0)),
    ];
    (any::<[u8; 32]>(), any::<bool>(), 0usize..300, any::<[u8; 32]>(), ending).prop_map(
        |(map, consistent, free_len, fill, ending)| {
            let len = if consistent { 256 - default_level_count(&map) } else { free_len };
            let siblings: Vec<[u8; 32]> = (0..len)
                .map(|index| {
                    let mut sibling = fill;
                    sibling[0] ^= index as u8;
                    sibling
                })
                .collect();
            postcard::to_allocvec(&(map, siblings, ending)).unwrap()
        },
    )
}

proptest! {
    /// verifies: REQ-ds8ryr, LLR-2dcnbp, LLR-4rju5r
    #[test]
    fn absence_proof_decoding_and_verify_never_panic(
        bytes in prop_oneof![
            proptest::collection::vec(any::<u8>(), 0..2048),
            arb_wire_proof(),
        ],
        target in any::<[u8; 32]>(),
    ) {
        if let Ok(proof) = postcard::from_bytes::<org_members::AbsenceProof>(&bytes) {
            let root = org_members::types::RootHash::new(target);
            let _ = proof.verify::<Blake3Hasher>(&root, &MemberId::new(target), &device_key("probe"));
        }
    }

    /// verifies: LLR-utp6x4, LLR-dgzy7e, LLR-4xz255, REQ-535jcd, REQ-tk2qqj
    #[test]
    fn every_honest_absence_proof_verifies(
        handle_indexes in proptest::collection::btree_set(0usize..HANDLES.len(), 1..6),
        absent_seed in any::<u64>(),
    ) {
        let fail = |error: OrgMembersError| TestCaseError::fail(format!("{error:?}"));
        let members: Vec<MemberLeaf> = handle_indexes
            .iter()
            .filter_map(|&index| make_member(HANDLES[index], 0))
            .collect();
        let trie = TestTrie::genesis(members).map_err(fail)?;
        let root = trie.root_hash().map_err(fail)?;
        let absent = member_id(&format!("absent-{absent_seed}"));
        let probe = device_key("probe-device");
        let proof = trie.prove_absent(&absent, &probe).map_err(fail)?;
        prop_assert_eq!(proof.verify::<Blake3Hasher>(&root, &absent, &probe), Ok(()));
    }
}

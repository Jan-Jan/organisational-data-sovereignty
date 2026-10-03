use ed25519_dalek::SigningKey;
use org_members::hasher::Blake3Hasher;
use org_members::trie::OrgTrie;
use org_members::types::{validate_handle, P2pDeviceKey, MemberId, P2pMemberKey, MemberLeaf};
use proptest::prelude::*;

type TestTrie = OrgTrie<Blake3Hasher>;

const HANDLES: &[&str] = &[
    "alice", "bob", "charlie", "jan-jan", "diana", "eve", "frank", "grace",
    "hank", "iris", "jack", "kate",
];

fn arb_handle_idx() -> impl Strategy<Value = usize> {
    0..HANDLES.len()
}

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

fn make_member(handle: &str, variant: u8) -> Option<MemberLeaf> {
    let id = member_id(&format!("{}-id-{}", handle, variant));
    let mk = member_key(&format!("{}-mk-{}", handle, variant));
    let dk = device_key(&format!("{}-d-{}", handle, variant));
    MemberLeaf::new(id, handle, mk, "Test", "User", vec![dk]).ok()
}

// ============================================================
// Handle validation fuzzing
// ============================================================

proptest! {
    /// verifies: REQ-ds8ryr, REQ-h5ret5, LLR-h9gs32
    #[test]
    fn handle_validation_never_panics(s in "\\PC{0,64}") {
        if let Ok(normalized) = validate_handle(&s) {
            prop_assert!(!normalized.is_empty());
            for ch in normalized.chars() {
                prop_assert!(!ch.is_uppercase(), "validated handle contains uppercase: {:?}", ch);
            }
            prop_assert!(!normalized.contains('.'), "validated handle contains '.'");
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
                    if let Ok(new_trie) = trie.update_handle(&id, new_handle) {
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
                    if let Ok((new_trie, _)) = trie.recalculate() {
                        let actual = new_trie.members().len();
                        prop_assert_eq!(
                            new_trie.member_count(), actual,
                            "member_count={} but actual members={}",
                            new_trie.member_count(), actual,
                        );
                        trie = new_trie;
                    }
                }
            }
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

        let (trie_b, delta) = match trie_b.recalculate() {
            Ok(r) => r,
            Err(_) => return Ok(()),
        };

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

        let (trie_b, _) = match trie_b.recalculate() {
            Ok(r) => r,
            Err(_) => return Ok(()),
        };

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
        let (_target, mut delta) = match work.recalculate() {
            Ok(r) => r,
            Err(_) => return Ok(()),
        };
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
                if r.iter().any(|x| *x == ghost) || base.contains(&ghost) {
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
// nothing (PR-zz4exm).
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
    Rotate(usize, u8),
}

fn arb_device_op() -> impl Strategy<Value = DeviceOp> {
    let member = 0..REMOVAL_POOL;
    let device = 0..DEVICE_POOL;
    prop_oneof![
        3 => (member.clone(), device.clone(), arb_key_choice())
            .prop_map(|(m, d, k)| DeviceOp::DeleteDevice(m, d, k)),
        2 => (member.clone(), arb_key_choice()).prop_map(|(m, k)| DeviceOp::Isolate(m, k)),
        2 => (member.clone(), device).prop_map(|(m, d)| DeviceOp::AddDevice(m, d)),
        1 => (member, 0..4u8).prop_map(|(m, v)| DeviceOp::Rotate(m, v)),
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

fn pool_device(member_idx: usize, device_idx: usize) -> P2pDeviceKey {
    device_key(&format!("{}-d-{}", HANDLES[member_idx], device_idx))
}

fn pool_key(member_idx: usize, variant: u8) -> P2pMemberKey {
    member_key(&format!("{}-mk-{}", HANDLES[member_idx], variant))
}

/// A pool member enrolled with devices 0..3 and genesis key variant 0.
fn pool_member(member_idx: usize) -> MemberLeaf {
    let devices = (0..3).map(|d| pool_device(member_idx, d)).collect();
    MemberLeaf::new(
        pool_id(member_idx),
        HANDLES[member_idx],
        pool_key(member_idx, 0),
        "Test",
        "User",
        devices,
    )
    .unwrap()
}

fn resolve_key(choice: &KeyChoice, member_idx: usize, current: P2pMemberKey) -> P2pMemberKey {
    match choice {
        KeyChoice::Current => current,
        KeyChoice::Seeded(v) => pool_key(member_idx, *v),
    }
}

proptest! {
    /// verifies: LLR-s97ywt, LLR-w92psx
    ///
    /// Every successful `delete_p2p_device` / `emergency_isolate_member`
    /// installs the supplied key and that key differs from the one it
    /// replaces. `P2pKeyNotReplaced` is returned only when the supplied key
    /// equalled the current key. A refusal is atomic by construction — the
    /// operations take `&self` and return `Result<Self, _>`, so an `Err`
    /// carries no trie — so the property checks the error, not a re-read of
    /// the unchanged input trie.
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
                DeviceOp::Rotate(_, v) => {
                    (trie.rotate_p2p_key(&id, pool_key(member_idx, *v)), None)
                }
            };

            // Only the removal ops carry the key-replacement property.
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
                    if let DeviceOp::DeleteDevice(_, d, _) = op {
                        prop_assert!(!after.has_p2p_device(&pool_device(member_idx, *d)));
                        prop_assert_eq!(after.p2p_device_count(), before.p2p_device_count() - 1);
                    } else {
                        prop_assert_eq!(after.p2p_device_count(), 0);
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
                    // isolate has no other refusal: the only other error is a
                    // delete of an absent device, whatever key was supplied.
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

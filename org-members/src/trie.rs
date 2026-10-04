use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;

use hashbrown::HashMap;

use crate::delta::{CandidateTrie, Delta};
use crate::error::OrgMembersError;
use crate::hasher::TrieHasher;
use crate::node::Node;
use crate::smt::{self, DefaultHashes};
use crate::normalize::to_nfc;
use crate::types::{
    handle_skeleton, validate_handle, MemberId, MemberLeaf, P2pDeviceKey, P2pDeviceSlots,
    P2pMemberKey, RootHash,
};

/// An immutable binary Sparse Merkle Tree for organisation membership.
///
/// Mutations (`add_member`, `delete_member`, `update_handle`, `rotate_p2p_key`,
/// `add_p2p_device`, `delete_p2p_device`, `emergency_isolate_member`,
/// `update_name_surname`) return a new trie via path-copying with lazy hash
/// computation. Call `recalculate()` to fill all pending hashes.
///
/// Maintains a skeleton index for UTS#39 confusable/homoglyph detection.
///
/// # Performance note
///
/// `skeleton_index`, `handle_index` and `key_index` are full `HashMap`s cloned on every
/// mutation -- O(N) memory per mutation regardless of how few members changed.
/// At 1000 members that's ~150KB allocation churn per mutation. Fine at the
/// design's target scale (1000 members, 1% monthly turnover) but undoes the
/// path-copying optimization for larger orgs.
///
/// Future optimization: wrap indexes in `Arc<Indexes>` shared across path-copies,
/// only rebuilt at `recalculate()`. For uncommitted mutations, find pending
/// leaves by walking the trie under unhashed nodes (subtrees whose root has an
/// unset `Once` cell). See discussion in the org-members worktree history.
pub struct OrgTrie<H: TrieHasher> {
    root: Arc<Node>,
    defaults: Arc<DefaultHashes>,
    member_count: usize,
    cached_root_hash: Option<RootHash>,
    /// The trie root at the time of the last `recalculate()` (or `genesis()`).
    /// Always populated -- every constructor sets it, and mutations propagate
    /// it forward unchanged. Used as the diff base for `pending_changes()`.
    last_calculated_root: Arc<Node>,
    /// Maps skeleton → handle string for confusable detection.
    skeleton_index: HashMap<String, String>,
    /// Maps handle → MemberId for handle-based lookups.
    /// Necessary because handle and id are independent (handle can change rarely
    /// while id stays the same).
    handle_index: HashMap<String, MemberId>,
    /// Maps every key held in the organisation -- each member key and each
    /// enrolled device key, by its 32 encoded bytes -- to the member holding
    /// it. One entry per key is the LLR-v6gfc7 invariant.
    key_index: KeyIndex,
    _hasher: core::marker::PhantomData<H>,
}

/// Key bytes → holder. See `OrgTrie::key_index`.
pub(crate) type KeyIndex = HashMap<[u8; 32], MemberId>;

/// The 32-byte encodings of every key `leaf` holds: its member key, then its
/// devices. A member key and a device key with the same bytes are the same key.
fn leaf_keys(leaf: &MemberLeaf) -> impl Iterator<Item = [u8; 32]> + '_ {
    core::iter::once(*leaf.p2p_key().as_bytes())
        .chain(leaf.p2p_devices().iter().map(|d| *d.as_bytes()))
}

/// Records every key `leaf` holds in `index`. Fails with `DuplicateKey` if
/// any of them is already held -- by another member, or by `leaf` itself
/// (its member key equal to one of its devices). On error `index` is left
/// partly updated; callers work on a clone and drop it.
fn index_leaf_keys(index: &mut KeyIndex, leaf: &MemberLeaf) -> Result<(), OrgMembersError> {
    for key in leaf_keys(leaf) {
        if index.insert(key, *leaf.id()).is_some() {
            return Err(OrgMembersError::DuplicateKey);
        }
    }
    Ok(())
}

fn unindex_leaf_keys(index: &mut KeyIndex, leaf: &MemberLeaf) {
    for key in leaf_keys(leaf) {
        index.remove(&key);
    }
}

impl<H: TrieHasher> OrgTrie<H> {
    /// Creates a genesis trie from initial members.
    /// Checks both id and handle uniqueness (including confusables), then
    /// key uniqueness: no key may be held in two places -- two members' member
    /// keys, a device enrolled under two members, or a member key equal to any
    /// enrolled device key, its own included -- else `DuplicateKey`
    /// (LLR-v6gfc7). Check order per member: `DuplicateId`, handle checks,
    /// `DuplicateKey`.
    pub fn genesis(members: Vec<MemberLeaf>) -> Result<Self, OrgMembersError> {
        let defaults = Arc::new(DefaultHashes::compute::<H>());
        let mut root = smt::empty_root(&defaults);
        let mut count = 0;
        let mut skeleton_index = HashMap::new();
        let mut handle_index = HashMap::new();
        let mut key_index = KeyIndex::new();

        for member in members {
            // Check for duplicate key
            if smt::get_member(&root, member.id()).is_some() {
                return Err(OrgMembersError::DuplicateId);
            }

            // Check for duplicate/confusable handle
            let skeleton = handle_skeleton(member.handle());
            if let Some(existing) = skeleton_index.get(&skeleton) {
                if existing != member.handle() {
                    return Err(OrgMembersError::ConfusableHandle);
                } else {
                    return Err(OrgMembersError::DuplicateHandle);
                }
            }

            index_leaf_keys(&mut key_index, &member)?;

            skeleton_index.insert(skeleton, member.handle().to_owned());
            handle_index.insert(member.handle().to_owned(), *member.id());
            root = smt::insert::<H>(&root, member, &defaults);
            count += 1;
        }

        let root_hash = smt::recalculate_hashes::<H>(&root)?;

        Ok(Self {
            root: root.clone(),
            defaults,
            member_count: count,
            cached_root_hash: Some(root_hash.into()),
            last_calculated_root: root,
            skeleton_index,
            handle_index,
            key_index,
            _hasher: core::marker::PhantomData,
        })
    }

    /// Returns the root hash. Returns `Err(HashesNotCalculated)` if there are
    /// pending mutations -- call `recalculate()` first.
    pub fn root_hash(&self) -> Result<RootHash, OrgMembersError> {
        self.cached_root_hash.ok_or(OrgMembersError::HashesNotCalculated)
    }

    pub fn is_calculated(&self) -> bool {
        self.cached_root_hash.is_some()
    }

    pub fn member_count(&self) -> usize {
        self.member_count
    }

    pub fn contains(&self, id: &MemberId) -> bool {
        smt::get_member(&self.root, id).is_some()
    }

    pub fn contains_handle(&self, handle: &str) -> bool {
        // NFC-normalize input so callers passing decomposed Unicode still match.
        let normalized = to_nfc(handle);
        self.handle_index.contains_key(normalized.as_str())
    }

    pub fn get(&self, id: &MemberId) -> Option<MemberLeaf> {
        smt::get_member(&self.root, id)
    }

    pub fn get_by_handle(&self, handle: &str) -> Option<MemberLeaf> {
        let normalized = to_nfc(handle);
        let id = self.handle_index.get(normalized.as_str())?;
        smt::get_member(&self.root, id)
    }

    pub fn members(&self) -> Vec<MemberLeaf> {
        smt::collect_members(&self.root)
    }

    // ====================================================================
    // Public domain operations
    // ====================================================================

    /// Adds a new member. Fails if the member id already exists, the handle is
    /// already taken, or the handle confusably collides with an existing handle.
    /// New members must have ≥1 device (enforced by `MemberLeaf::new`).
    ///
    /// A leaf holding a key already held in the organisation -- its member key
    /// or a device key equal to another member's key or any enrolled device
    /// key -- or whose member key equals one of its own devices is refused with
    /// `DuplicateKey` (LLR-v6gfc7). Check order: `DuplicateId`, handle checks,
    /// `DuplicateKey`.
    ///
    /// Caller duty: `MemberId`s must be fresh random values. Re-adding a
    /// deleted member's id is outside the contract (the trie keeps no record
    /// of deleted ids and will accept it). A member re-admitted under a new id
    /// is a new member. The crate refuses only keys held when the operation
    /// runs and keeps no key history: never give a new member a key no longer
    /// held, other than the keys the person's previous membership held when it
    /// was deleted. A re-admitted member may bring those keys (a new member
    /// under a new id; owner ruling, 2026-10-03);
    /// where a removed device was compromised, fresh keys are the joiner's
    /// choice, not a check the crate makes (README, "Security checks the
    /// caller MUST perform", item 11).
    pub fn add_member(&self, leaf: MemberLeaf) -> Result<Self, OrgMembersError> {
        self.insert_leaf(leaf)
    }

    /// Removes a member by id. Their handle becomes available for re-use.
    ///
    /// Caller duty: deletion is permanent. The id must never be re-used --
    /// the trie keeps no record of it and will not refuse it (see
    /// `add_member`). The member's keys are no longer held once it is deleted,
    /// so the crate accepts them again. A re-admitted member may bring them (a
    /// new member under a new id; owner ruling, 2026-10-03), but they must
    /// never be given to anyone else; where a removed device was compromised,
    /// fresh keys are the joiner's choice (README,
    /// "Security checks the caller MUST perform", item 11).
    pub fn delete_member(&self, id: &MemberId) -> Result<Self, OrgMembersError> {
        self.delete_by_id(id)
    }

    /// Updates a member's name and surname. Both are NFC-normalized.
    pub fn update_name_surname(
        &self,
        id: &MemberId,
        name: &str,
        surname: &str,
    ) -> Result<Self, OrgMembersError> {
        let existing = smt::get_member(&self.root, id).ok_or(OrgMembersError::IdNotFound)?;
        let nfc_name = to_nfc(name);
        if nfc_name.len() > crate::types::MAX_NAME_LEN {
            return Err(OrgMembersError::FieldTooLong {
                field: "name",
                max: crate::types::MAX_NAME_LEN,
            });
        }
        let nfc_surname = to_nfc(surname);
        if nfc_surname.len() > crate::types::MAX_SURNAME_LEN {
            return Err(OrgMembersError::FieldTooLong {
                field: "surname",
                max: crate::types::MAX_SURNAME_LEN,
            });
        }
        let new_leaf = existing.with_name_surname(nfc_name, nfc_surname);
        self.update_leaf(new_leaf)
    }

    /// Updates a member's handle. The new handle must pass validation
    /// (UTS#39, NFC, lowercase, no `.`, single-script) and must not be
    /// taken by, or confusably collide with, another member's handle.
    pub fn update_handle(
        &self,
        id: &MemberId,
        new_handle: &str,
    ) -> Result<Self, OrgMembersError> {
        let existing = smt::get_member(&self.root, id).ok_or(OrgMembersError::IdNotFound)?;
        let validated = validate_handle(new_handle)?;
        let new_leaf = existing.with_handle(validated);
        self.update_leaf(new_leaf)
    }

    /// Rotates a member's peer-to-peer key -- the "member-as-a-group" key
    /// the local-first software uses to identify the member when granting
    /// access. All other fields, including device set, are unchanged.
    ///
    /// A replacement key equal to the member's current key replaces nothing,
    /// so it is refused with `P2pKeyNotReplaced` (LLR-k89ahd). A replacement
    /// key held anywhere in the organisation before the operation -- another
    /// member's key or any enrolled device key, this member's own devices
    /// included -- is refused with `DuplicateKey` (LLR-fym7dy). Either refusal
    /// changes nothing. Check order: `IdNotFound`, `P2pKeyNotReplaced`,
    /// `DuplicateKey`.
    ///
    /// Caller duty: the crate refuses only keys held when the operation runs
    /// and keeps no key history. Never supply a key no longer held -- a member
    /// key used earlier and since replaced, by this member or any other, or the
    /// device key of a device removed earlier: a device holding its secret
    /// keeps or regains access (README, "Security checks the caller MUST
    /// perform", item 11).
    pub fn rotate_p2p_key(
        &self,
        id: &MemberId,
        new_p2p_key: P2pMemberKey,
    ) -> Result<Self, OrgMembersError> {
        let existing = smt::get_member(&self.root, id).ok_or(OrgMembersError::IdNotFound)?;
        if existing.p2p_key() == &new_p2p_key {
            return Err(OrgMembersError::P2pKeyNotReplaced);
        }
        self.refuse_held_key(new_p2p_key.as_bytes())?;
        let new_leaf = existing.with_p2p_key(new_p2p_key);
        self.update_leaf(new_leaf)
    }

    /// Adds a peer-to-peer device key to a member. Fails if the device is
    /// already present or the member already has `MAX_DEVICES` (4) devices.
    /// Does NOT rotate the p2p_key -- a new device is trusted with the
    /// current key.
    ///
    /// A device key already held in the organisation -- enrolled under
    /// another member, or equal to any member key, this member's own included
    /// -- is refused with `DuplicateKey` (LLR-v6gfc7); the refusal changes
    /// nothing. Check order: `IdNotFound`, `DeviceSlotsFull`,
    /// `DuplicateDevice`, `DuplicateKey`.
    pub fn add_p2p_device(
        &self,
        id: &MemberId,
        device: P2pDeviceKey,
    ) -> Result<Self, OrgMembersError> {
        let existing = smt::get_member(&self.root, id).ok_or(OrgMembersError::IdNotFound)?;
        let new_slots = existing.p2p_device_slots().add_device(device)?;
        self.refuse_held_key(device.as_bytes())?;
        let new_leaf = existing.with_p2p_device_slots(new_slots);
        self.update_leaf(new_leaf)
    }

    /// Deletes a peer-to-peer device key from a member AND rotates the
    /// member's p2p_key. Both are required: the deleted device had access to
    /// the old p2p_key (the member-as-a-group key in the local-first software,
    /// which the device derived secrets from), so rotating it invalidates
    /// that access. If the deleted device was the last one, the member
    /// becomes isolated (zero devices).
    ///
    /// A replacement key equal to the member's current key would leave the
    /// removed device holding a live key, so it is refused with
    /// `P2pKeyNotReplaced`. A replacement key held anywhere in the organisation
    /// before the operation -- another member's key or any enrolled device
    /// key, the key of the device being removed included -- is refused with
    /// `DuplicateKey` (LLR-fym7dy). Either refusal changes nothing: the device
    /// stays enrolled and the key is kept. Check order: `IdNotFound`,
    /// `DeviceNotFound`, `P2pKeyNotReplaced`, `DuplicateKey`.
    ///
    /// Caller duty: the crate refuses only keys held when the operation runs
    /// and keeps no key history. Never supply a key no longer held -- a member
    /// key used earlier and since replaced, by this member or any other, or the
    /// device key of a device removed earlier: a device holding its secret
    /// keeps or regains access (README, "Security checks the caller MUST
    /// perform", item 11).
    pub fn delete_p2p_device(
        &self,
        id: &MemberId,
        device: &P2pDeviceKey,
        new_p2p_key: P2pMemberKey,
    ) -> Result<Self, OrgMembersError> {
        let existing = smt::get_member(&self.root, id).ok_or(OrgMembersError::IdNotFound)?;
        let new_slots = existing.p2p_device_slots().remove_device(device)?;
        if existing.p2p_key() == &new_p2p_key {
            return Err(OrgMembersError::P2pKeyNotReplaced);
        }
        // Checked against the record before the operation, so the removed
        // device's own key counts as held.
        self.refuse_held_key(new_p2p_key.as_bytes())?;
        let new_leaf = existing
            .with_p2p_device_slots(new_slots)
            .with_p2p_key(new_p2p_key);
        self.update_leaf(new_leaf)
    }

    /// Emergency operation: removes ALL of a member's devices AND rotates the
    /// p2p_key. Use when multiple devices are compromised or the situation is
    /// unclear and you want to cut off all access at once. The member remains
    /// in the trie with zero devices (isolated state). Re-add a device via
    /// `add_p2p_device` to un-isolate.
    ///
    /// A replacement key equal to the member's current key would leave the
    /// removed devices holding a live key, so it is refused with
    /// `P2pKeyNotReplaced`. A replacement key held anywhere in the organisation
    /// before the operation -- another member's key or any enrolled device
    /// key, the keys of the devices being removed included -- is refused with
    /// `DuplicateKey` (LLR-fym7dy). Either refusal changes nothing: every
    /// device stays enrolled and the key is kept. Check order: `IdNotFound`,
    /// `P2pKeyNotReplaced`, `DuplicateKey`.
    ///
    /// Caller duty: the crate refuses only keys held when the operation runs
    /// and keeps no key history. Never supply a key no longer held -- a member
    /// key used earlier and since replaced, by this member or any other, or the
    /// device key of a device removed earlier: a device holding its secret
    /// keeps or regains access (README, "Security checks the caller MUST
    /// perform", item 11).
    pub fn emergency_isolate_member(
        &self,
        id: &MemberId,
        new_p2p_key: P2pMemberKey,
    ) -> Result<Self, OrgMembersError> {
        let existing = smt::get_member(&self.root, id).ok_or(OrgMembersError::IdNotFound)?;
        if existing.p2p_key() == &new_p2p_key {
            return Err(OrgMembersError::P2pKeyNotReplaced);
        }
        // Checked against the record before the operation, so the removed
        // devices' keys count as held.
        self.refuse_held_key(new_p2p_key.as_bytes())?;
        let empty_slots = P2pDeviceSlots::new(Vec::new())?;
        let new_leaf = existing
            .with_p2p_device_slots(empty_slots)
            .with_p2p_key(new_p2p_key);
        self.update_leaf(new_leaf)
    }

    // ====================================================================
    // Crate-private helpers used by the domain operations and apply_delta
    // ====================================================================

    fn insert_leaf(&self, leaf: MemberLeaf) -> Result<Self, OrgMembersError> {
        if smt::get_member(&self.root, leaf.id()).is_some() {
            return Err(OrgMembersError::DuplicateId);
        }

        let mut new_skeleton_index = self.skeleton_index.clone();
        let mut new_handle_index = self.handle_index.clone();
        let skeleton = handle_skeleton(leaf.handle());
        if let Some(existing) = new_skeleton_index.get(&skeleton) {
            if existing != leaf.handle() {
                return Err(OrgMembersError::ConfusableHandle);
            } else {
                return Err(OrgMembersError::DuplicateHandle);
            }
        }
        new_skeleton_index.insert(skeleton, leaf.handle().to_owned());
        new_handle_index.insert(leaf.handle().to_owned(), *leaf.id());

        let mut new_key_index = self.key_index.clone();
        index_leaf_keys(&mut new_key_index, &leaf)?;

        let new_root = smt::insert::<H>(&self.root, leaf, &self.defaults);

        Ok(Self {
            root: new_root,
            defaults: self.defaults.clone(),
            member_count: self.member_count
                .checked_add(1)
                .ok_or(OrgMembersError::InvariantViolated)?,
            cached_root_hash: None,
            last_calculated_root: self.last_calculated_root.clone(),
            skeleton_index: new_skeleton_index,
            handle_index: new_handle_index,
            key_index: new_key_index,
            _hasher: core::marker::PhantomData,
        })
    }

    /// `DuplicateKey` if `key` is held anywhere in the organisation now.
    fn refuse_held_key(&self, key: &[u8; 32]) -> Result<(), OrgMembersError> {
        if self.key_index.contains_key(key) {
            return Err(OrgMembersError::DuplicateKey);
        }
        Ok(())
    }

    fn update_leaf(&self, leaf: MemberLeaf) -> Result<Self, OrgMembersError> {
        let existing =
            smt::get_member(&self.root, leaf.id()).ok_or(OrgMembersError::IdNotFound)?;

        let mut new_skeleton_index = self.skeleton_index.clone();
        let mut new_handle_index = self.handle_index.clone();

        // Re-index the leaf's keys. The operations that change keys refuse a
        // key held before the operation first; this keeps the index exact and
        // the invariant enforced on every path through here.
        let mut new_key_index = self.key_index.clone();
        unindex_leaf_keys(&mut new_key_index, &existing);
        index_leaf_keys(&mut new_key_index, &leaf)?;

        if existing.handle() != leaf.handle() {
            let old_skeleton = handle_skeleton(existing.handle());
            new_skeleton_index.remove(&old_skeleton);
            new_handle_index.remove(existing.handle());

            let new_skeleton = handle_skeleton(leaf.handle());
            if let Some(existing_handle) = new_skeleton_index.get(&new_skeleton) {
                if existing_handle != leaf.handle() {
                    return Err(OrgMembersError::ConfusableHandle);
                } else {
                    return Err(OrgMembersError::DuplicateHandle);
                }
            }
            new_skeleton_index.insert(new_skeleton, leaf.handle().to_owned());
            new_handle_index.insert(leaf.handle().to_owned(), *leaf.id());
        }

        let new_root = smt::insert::<H>(&self.root, leaf, &self.defaults);

        Ok(Self {
            root: new_root,
            defaults: self.defaults.clone(),
            member_count: self.member_count,
            cached_root_hash: None,
            last_calculated_root: self.last_calculated_root.clone(),
            skeleton_index: new_skeleton_index,
            handle_index: new_handle_index,
            key_index: new_key_index,
            _hasher: core::marker::PhantomData,
        })
    }

    fn delete_by_id(&self, id: &MemberId) -> Result<Self, OrgMembersError> {
        let existing = smt::get_member(&self.root, id).ok_or(OrgMembersError::IdNotFound)?;

        let new_root = smt::remove(&self.root, id, &self.defaults);

        let mut new_skeleton_index = self.skeleton_index.clone();
        let mut new_handle_index = self.handle_index.clone();
        let skeleton = handle_skeleton(existing.handle());
        new_skeleton_index.remove(&skeleton);
        new_handle_index.remove(existing.handle());
        let mut new_key_index = self.key_index.clone();
        unindex_leaf_keys(&mut new_key_index, &existing);

        Ok(Self {
            root: new_root,
            defaults: self.defaults.clone(),
            member_count: self.member_count
                .checked_sub(1)
                .ok_or(OrgMembersError::InvariantViolated)?,
            cached_root_hash: None,
            last_calculated_root: self.last_calculated_root.clone(),
            skeleton_index: new_skeleton_index,
            handle_index: new_handle_index,
            key_index: new_key_index,
            _hasher: core::marker::PhantomData,
        })
    }

    /// Returns the delta of changes accumulated since the last `recalculate()`.
    ///
    /// Does NOT compute hashes or change trie state -- safe to call multiple times
    /// for review. Admins can use this to inspect pending changes before agreeing
    /// to commit a new root hash via `recalculate()`.
    ///
    /// Returns an empty delta if no changes are pending.
    /// Returns `Err(InvariantViolated)` if the internal `last_calculated_root`
    /// invariant is broken (its hash must always be populated).
    pub fn pending_changes(&self) -> Result<Delta, OrgMembersError> {
        let (removed, upserted) = smt::diff_tries(&self.last_calculated_root, &self.root);
        // Invariant: last_calculated_root's hash is always populated -- every
        // constructor sets it via genesis(), recalculate(), or from_candidate(),
        // and mutations propagate it unchanged.
        let base_hash = self
            .last_calculated_root
            .hash()
            .ok_or(OrgMembersError::InvariantViolated)?;
        Ok(Delta {
            base_root: (*base_hash).into(),
            removed,
            upserted,
        })
    }

    /// Returns true if there are uncommitted mutations since the last `recalculate()`.
    /// When false, the trie is calculated and `recalculate()` refuses it.
    pub fn has_pending_changes(&self) -> bool {
        self.cached_root_hash.is_none()
    }

    /// Walks the trie bottom-up, filling every unset `spin::Once` hash cell.
    /// Returns the trie (now fully hashed) and a delta of all pending changes.
    ///
    /// Returns `Err(HashesAlreadyCalculated)` if the trie has not been mutated
    /// since it was calculated: a trie from `genesis()`, `recalculate()` or
    /// `verify_against()` already has every cell filled. A mutated trie
    /// recalculates even when its change set is empty (LLR-j35sxz).
    pub fn recalculate(&self) -> Result<(Self, Delta), OrgMembersError> {
        if !self.has_pending_changes() {
            return Err(OrgMembersError::HashesAlreadyCalculated);
        }
        let delta = self.pending_changes()?;
        let root_hash = smt::recalculate_hashes::<H>(&self.root)?;

        Ok((
            Self {
                root: self.root.clone(),
                defaults: self.defaults.clone(),
                member_count: self.member_count,
                cached_root_hash: Some(root_hash.into()),
                last_calculated_root: self.root.clone(),
                skeleton_index: self.skeleton_index.clone(),
                handle_index: self.handle_index.clone(),
                key_index: self.key_index.clone(),
                _hasher: core::marker::PhantomData,
            },
            delta,
        ))
    }

    /// Checks the canonical-form invariant on a received delta. Returns
    /// `Ok(())` if every rule holds:
    ///
    /// - `delta.removed` is strictly increasing by id, and every id exists
    ///   in `self.root` (no stale removals).
    /// - `delta.upserted` is strictly increasing by id, and every leaf differs
    ///   from the current state at that id (no no-op upserts).
    /// - `delta.removed` and `delta.upserted` have no id in common.
    ///
    /// Honest deltas produced by `recalculate()`, `calculate_delta()`, and
    /// `pending_changes()` satisfy these by construction via the SMT diff
    /// walk. Non-canonical wire-form variants are rejected here so that the
    /// set of `Delta` **values** `apply_delta` will accept between two roots
    /// is narrowed to one.
    ///
    /// **This is a constraint on the decoded value, not on the bytes.**
    /// Corrected 2026-09-17: this paragraph used to say the check made "the
    /// postcard byte string accepted by `apply_delta` the unique encoding of
    /// the transition", which is false. `MemberLeaf`'s `Deserialize` impl
    /// normalises rather than rejects, so several distinct byte strings decode
    /// to one `Delta` and all of them pass these rules. Key dedup, replay
    /// caches and change identity on the decoded `Delta` or on the
    /// `(base_root, target_root)` pair — never on the encoding. See the
    /// `Delta` type's own doc comment in `delta.rs` for the counterexample.
    fn validate_canonical_delta(&self, delta: &Delta) -> Result<(), OrgMembersError> {
        use core::cmp::Ordering;

        // removed: strictly increasing, every id present.
        for pair in delta.removed.windows(2) {
            if pair[0] >= pair[1] {
                return Err(OrgMembersError::MalformedDelta(
                    "removed not strictly increasing",
                ));
            }
        }
        for id in &delta.removed {
            if smt::get_member(&self.root, id).is_none() {
                return Err(OrgMembersError::MalformedDelta(
                    "removed id not present in trie",
                ));
            }
        }
        // upserted: strictly increasing, each leaf produces an observable change.
        for pair in delta.upserted.windows(2) {
            if pair[0].id() >= pair[1].id() {
                return Err(OrgMembersError::MalformedDelta(
                    "upserted not strictly increasing by id",
                ));
            }
        }
        for leaf in &delta.upserted {
            if let Some(existing) = smt::get_member(&self.root, leaf.id()) {
                if &existing == leaf {
                    return Err(OrgMembersError::MalformedDelta(
                        "upserted leaf identical to existing trie state",
                    ));
                }
            }
        }
        // removed and upserted disjoint (two-pointer merge over sorted ids).
        let mut ri = 0;
        let mut ui = 0;
        while ri < delta.removed.len() && ui < delta.upserted.len() {
            match delta.removed[ri].cmp(delta.upserted[ui].id()) {
                Ordering::Less => ri += 1,
                Ordering::Greater => ui += 1,
                Ordering::Equal => {
                    return Err(OrgMembersError::MalformedDelta(
                        "id appears in both removed and upserted",
                    ));
                }
            }
        }

        Ok(())
    }

    /// Applies a received delta. Returns CandidateTrie (must verify before use).
    ///
    /// Rejects deltas that are not in canonical form (H-1):
    /// - `removed` MUST be strictly increasing by id; every id MUST exist in the trie.
    /// - `upserted` MUST be strictly increasing by id; every leaf MUST produce an
    ///   observable change vs. the current trie state at that id.
    /// - `removed` and `upserted` MUST be disjoint.
    ///
    /// After every other check, a delta whose resulting record would hold a
    /// key in two places is refused with `DuplicateKey` (LLR-gjj6bx). Only the
    /// resulting record is checked, not keys held in this one: a delta
    /// collapses a sequence of direct operations, each judged on the record it
    /// ran on, so `delete_p2p_device(D, K1)` then `rotate_p2p_key(K_D)` is
    /// accepted step by step and its delta must be accepted too. Never
    /// supplying a key no longer held is the caller's duty on both paths
    /// (README, "Security checks the caller MUST perform", item 11).
    ///
    /// Together the canonical-form rules narrow the accepted `Delta` **value** for a transition
    /// from `base_root` to the resulting root to one. They do **not** make the
    /// postcard encoding of that value unique: corrected 2026-09-17, after an
    /// independent review measured two distinct byte strings (141 and 142
    /// bytes, an NFC name and its NFD form) that both decode, both apply here,
    /// and both verify against the same target root. `MemberLeaf`'s
    /// `Deserialize` normalises `name`, `surname` and `handle` instead of
    /// rejecting non-canonical forms, where `P2pDeviceSlots`' `Deserialize`
    /// rejects. Callers keying dedup, replay caches or change identity on
    /// bytes must key on the decoded `Delta`, or on the
    /// `(base_root, target_root)` pair, instead.
    ///
    /// See `delta.rs` for the full statement and
    /// `docs/superpowers/specs/2026-05-28-org-members-hyperbridge-review.md`
    /// for the threat model — that spec asserts the byte-uniqueness invariant
    /// and carries a dated note at its head saying it was disproved.
    pub fn apply_delta(&self, delta: &Delta) -> Result<CandidateTrie<H>, OrgMembersError> {
        let current_root = self
            .cached_root_hash
            .ok_or(OrgMembersError::HashesNotCalculated)?;

        if delta.base_root != current_root {
            return Err(OrgMembersError::DeltaBaseMismatch);
        }

        self.validate_canonical_delta(delta)?;

        // --- Apply (every operation is now guaranteed meaningful) ---
        let mut root = self.root.clone();
        let mut count = self.member_count;
        let mut new_skeleton_index = self.skeleton_index.clone();
        let mut new_handle_index = self.handle_index.clone();

        for id in &delta.removed {
            // Existence verified in validate_canonical_delta; the lookup here is
            // to extract the handle/skeleton for index maintenance.
            let existing = smt::get_member(&root, id).ok_or(OrgMembersError::InvariantViolated)?;
            new_skeleton_index.remove(&handle_skeleton(existing.handle()));
            new_handle_index.remove(existing.handle());
            root = smt::remove(&root, id, &self.defaults);
            count = count.checked_sub(1).ok_or(OrgMembersError::InvariantViolated)?;
        }

        // LLR-n5t6bn: release every outgoing handle before checking any
        // incoming one, so a handle moving between two present members is
        // judged against the record after the whole set, in any identifier
        // order (PR-vf5hdm).
        for member in &delta.upserted {
            if let Some(old) = smt::get_member(&root, member.id()) {
                if old.handle() != member.handle() {
                    new_skeleton_index.remove(&handle_skeleton(old.handle()));
                    new_handle_index.remove(old.handle());
                }
            }
        }

        for member in &delta.upserted {
            let existing = smt::get_member(&root, member.id());

            let needs_check = match &existing {
                Some(old) => old.handle() != member.handle(),
                None => true,
            };

            if needs_check {
                let skeleton = handle_skeleton(member.handle());
                if let Some(existing_handle) = new_skeleton_index.get(&skeleton) {
                    if existing_handle != member.handle() {
                        return Err(OrgMembersError::ConfusableHandle);
                    } else {
                        return Err(OrgMembersError::DuplicateHandle);
                    }
                }
                new_skeleton_index.insert(skeleton, member.handle().to_owned());
                new_handle_index.insert(member.handle().to_owned(), *member.id());
            }

            root = smt::insert::<H>(&root, member.clone(), &self.defaults);
            if existing.is_none() {
                count = count.checked_add(1).ok_or(OrgMembersError::InvariantViolated)?;
            }
        }

        let new_key_index = self.delta_key_index(delta)?;

        let root_hash = smt::recalculate_hashes::<H>(&root)?;

        Ok(CandidateTrie {
            root,
            defaults: self.defaults.clone(),
            member_count: count,
            root_hash: root_hash.into(),
            skeleton_index: new_skeleton_index,
            handle_index: new_handle_index,
            key_index: new_key_index,
            _hasher: core::marker::PhantomData,
        })
    }

    /// The key index of the record `delta` produces, or `DuplicateKey`
    /// (LLR-gjj6bx) if that record would hold a key in two places. Keys held
    /// only in the base record are not checked: see `apply_delta`.
    ///
    /// Runs after every other `apply_delta` check, so its refusal comes last.
    /// `delta` must already have passed `validate_canonical_delta`.
    fn delta_key_index(&self, delta: &Delta) -> Result<KeyIndex, OrgMembersError> {
        let mut index = self.key_index.clone();

        for id in &delta.removed {
            let old = smt::get_member(&self.root, id).ok_or(OrgMembersError::InvariantViolated)?;
            unindex_leaf_keys(&mut index, &old);
        }
        // Drop every replaced leaf's keys before adding any new ones, so a key
        // moving between two upserted leaves -- a handover or a swap -- is
        // judged on the final record in any identifier order, as the handle
        // check is (LLR-n5t6bn).
        for member in &delta.upserted {
            if let Some(old) = smt::get_member(&self.root, member.id()) {
                unindex_leaf_keys(&mut index, &old);
            }
        }
        for member in &delta.upserted {
            index_leaf_keys(&mut index, member)?;
        }
        Ok(index)
    }

    /// Computes the delta that transforms `old` into `self`.
    pub fn calculate_delta(&self, old: &OrgTrie<H>) -> Result<Delta, OrgMembersError> {
        if !self.is_calculated() || !old.is_calculated() {
            return Err(OrgMembersError::HashesNotCalculated);
        }

        let (removed, upserted) = smt::diff_tries(&old.root, &self.root);

        Ok(Delta {
            base_root: old.root_hash()?,
            removed,
            upserted,
        })
    }

    pub(crate) fn from_candidate(
        root: Arc<Node>,
        defaults: Arc<DefaultHashes>,
        member_count: usize,
        root_hash: RootHash,
        skeleton_index: HashMap<String, String>,
        handle_index: HashMap<String, MemberId>,
        key_index: KeyIndex,
    ) -> Self {
        Self {
            root: root.clone(),
            defaults,
            member_count,
            cached_root_hash: Some(root_hash),
            last_calculated_root: root,
            skeleton_index,
            handle_index,
            key_index,
            _hasher: core::marker::PhantomData,
        }
    }
}

impl<H: TrieHasher> Clone for OrgTrie<H> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            defaults: self.defaults.clone(),
            member_count: self.member_count,
            cached_root_hash: self.cached_root_hash,
            last_calculated_root: self.last_calculated_root.clone(),
            skeleton_index: self.skeleton_index.clone(),
            handle_index: self.handle_index.clone(),
            key_index: self.key_index.clone(),
            _hasher: core::marker::PhantomData,
        }
    }
}

impl<H: TrieHasher> fmt::Debug for OrgTrie<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OrgTrie")
            .field("member_count", &self.member_count)
            .field("root_hash", &self.cached_root_hash)
            .field("is_calculated", &self.is_calculated())
            .finish()
    }
}

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;

use hashbrown::HashMap;

use crate::error::OrgMembersError;
use crate::hasher::TrieHasher;
use crate::node::Node;
use crate::smt::DefaultHashes;
use crate::trie::{KeyIndex, OrgTrie};
use crate::types::{Handle, HandleSkeleton, MemberId, MemberLeaf, RootHash};

/// A set of changes anchored to a specific base trie root.
///
/// # Canonical-form invariant
///
/// Every `Delta` accepted by `OrgTrie::apply_delta` is in canonical form:
///
/// - `removed` is strictly increasing by `MemberId` and every id is present in
///   the trie at `base_root`.
/// - `upserted` is strictly increasing by `MemberId` and every leaf produces
///   an observable change vs. the current state at that id.
/// - `removed` and `upserted` are disjoint.
///
/// Combined with the fact that `recalculate()`, `calculate_delta()`, and
/// `pending_changes()` all produce canonical deltas by construction (via
/// `diff_recursive`'s left-then-right SMT traversal), this gives the higher-
/// level layer a guarantee about *structure*: for any `(base_root,
/// target_root)` pair there is exactly one `Delta` **value** `apply_delta`
/// will accept, and every delta this crate produces is already in that form.
///
/// **This is not byte-level uniqueness, and must not be relied on as if it
/// were.** Corrected 2026-09-17 after an independent review found the earlier
/// wording ("exactly one postcard byte string") false. `MemberLeaf`'s
/// `Deserialize` *normalises* rather than rejects: its `handle`, `name` and
/// `surname` decode through `parse`, which stores the NFC form, so
/// an NFD-encoded leaf and its NFC equivalent are two distinct postcard byte
/// strings that decode to the same `MemberLeaf` and produce the same root. The
/// encoding is therefore **not injective** on the deserialisation path.
/// (Contrast `DeviceSlots`, whose `Deserialize` genuinely rejects
/// non-canonical forms.)
///
/// Anything upstream that needs byte-level identity — dedup by encoded bytes,
/// a signature over encoded bytes treated as an identifier for the change, a
/// replay guard keyed on the encoding — must supply that property itself; it
/// does not come from here. Making the encoding injective would be a
/// behaviour change needing its own red-first test, and is deliberately not
/// done in the change that corrected this comment.
///
/// # What this crate does NOT do
///
/// `Delta` is scoped only by `base_root`. The following are the caller's
/// responsibility and MUST be enforced upstream of `apply_delta`:
///
/// - **Authentication** — establish who sent the `postcard(Delta)` bytes
///   before applying (a signature over them, or a connection-authenticated
///   sender checked against known devices), or, as org-node does by owner
///   ruling, rest every decision on a trusted root read independently of
///   the sender.
/// - **Organisation binding** — wrap deltas in `(org_id, seq,
///   postcard(Delta))` envelopes; the lib has no notion of which
///   organisation a delta belongs to.
/// - **Replay protection across time** — `base_root` rejects deltas once the
///   trie has moved past their parent, but a trie that revisits a prior root
///   would accept a stale delta. Use a monotonic sequence number in the
///   envelope.
/// - **Authority** — `apply_delta` accepts any well-formed change; whether
///   its author is allowed to make this change (quorum, role-based veto, rate
///   limits) is policy that lives above this crate.
/// - **Independent trusted root** — `CandidateTrie::verify_against`'s
///   `expected_root` argument must come from a path the attacker cannot
///   control (on-chain commit, signed admin attestation, etc.), not from the
///   same payload as the delta.
///
/// See `org-members/README.md` for the full enumeration of upstream security
/// responsibilities, and `docs/superpowers/specs/2026-05-28-org-members-
/// hyperbridge-review.md` for the threat model.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Delta {
    pub(crate) base_root: RootHash,
    pub(crate) removed: Vec<MemberId>,
    pub(crate) upserted: Vec<MemberLeaf>,
}

impl Delta {
    pub fn base_root(&self) -> &RootHash {
        &self.base_root
    }

    /// Member ids that were removed.
    pub fn removed(&self) -> &[MemberId] {
        &self.removed
    }

    pub fn upserted(&self) -> &[MemberLeaf] {
        &self.upserted
    }

    pub fn is_empty(&self) -> bool {
        self.removed.is_empty() && self.upserted.is_empty()
    }
}

/// Internal helpers for integration tests that need to construct adversarial
/// deltas (e.g., stale or duplicate removals, confusable upserts) without
/// going through `recalculate()`. Gated behind the `test-helpers` feature so
/// production builds cannot reach these mutators.
#[cfg(feature = "test-helpers")]
#[doc(hidden)]
pub mod test_support {
    use super::*;

    pub fn delta_set_removed(delta: &mut Delta, ids: Vec<MemberId>) {
        delta.removed = ids;
    }

    pub fn delta_set_upserted(delta: &mut Delta, leaves: Vec<MemberLeaf>) {
        delta.upserted = leaves;
    }
}

/// Result of `apply_delta()`. Cannot query members -- can only verify or drop.
pub struct CandidateTrie<H: TrieHasher> {
    pub(crate) root: Arc<Node>,
    pub(crate) defaults: Arc<DefaultHashes>,
    pub(crate) member_count: usize,
    pub(crate) root_hash: RootHash,
    pub(crate) skeleton_index: HashMap<HandleSkeleton, Handle>,
    pub(crate) handle_index: HashMap<Handle, MemberId>,
    pub(crate) key_index: KeyIndex,
    pub(crate) _hasher: core::marker::PhantomData<H>,
}

impl<H: TrieHasher> CandidateTrie<H> {
    pub fn root_hash(&self) -> RootHash {
        self.root_hash
    }

    pub fn verify_against(self, expected_root: &RootHash) -> Result<OrgTrie<H>, OrgMembersError> {
        if self.root_hash != *expected_root {
            return Err(OrgMembersError::VerificationFailed);
        }

        Ok(OrgTrie::from_candidate(
            self.root,
            self.defaults,
            self.member_count,
            self.root_hash,
            self.skeleton_index,
            self.handle_index,
            self.key_index,
        ))
    }
}

impl<H: TrieHasher> fmt::Debug for CandidateTrie<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CandidateTrie")
            .field("root_hash", &self.root_hash)
            .field("member_count", &self.member_count)
            .finish()
    }
}

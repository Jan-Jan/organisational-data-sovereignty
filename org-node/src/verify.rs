//! The chain-free checks a received Envelope must pass before anything of it
//! is kept (LLR-fuq379, LLR-76bhw9). Nothing about who delivered it is
//! checked here, and no chain state is taken: whether a kept change becomes
//! current is decided by `verify` against the chain (LLR-8528ud).
use org_members::delta::Delta;
use org_members::hasher::Blake3Hasher;
use org_members::trie::OrgTrie;

use crate::envelope::Envelope;
use crate::error::OrgNodeError;
use crate::ids::OrgId;
use crate::sequence::SeqGuard;

pub type Trie = OrgTrie<Blake3Hasher>;

/// Inputs that pin what the receiver already trusts about the org.
pub struct VerifyContext {
    /// The org we expect this envelope to be for.
    pub expected_org_id: OrgId,
    /// Replay guard for this org.
    pub seq_guard: SeqGuard,
}

/// The checks that need no chain, in this order: the Organisation, the
/// Sequence number, the Change set decode, the base root. Returns the decoded
/// Change set (LLR-fuq379).
fn chain_free(local_trie: &Trie, envelope: &Envelope, ctx: &VerifyContext) -> Result<Delta, OrgNodeError> {
    if envelope.org_id != ctx.expected_org_id {
        return Err(OrgNodeError::OrgIdMismatch);
    }
    ctx.seq_guard.check(envelope.parent_seq)?;
    let delta = envelope.decode_delta()?;
    // apply_delta also checks the base, but the specific error comes first.
    if delta.base_root() != &local_trie.root_hash()? {
        return Err(OrgNodeError::DeltaBaseMismatch);
    }
    Ok(delta)
}

/// The chain-free checks, so the receive path refuses a stranger's Envelope
/// with no chain state (LLR-fuq379, RC-mj6gjq). Takes no chain state.
pub fn check_chain_free(local_trie: &Trie, envelope: &Envelope, ctx: &VerifyContext) -> Result<(), OrgNodeError> {
    chain_free(local_trie, envelope, ctx).map(|_| ())
}

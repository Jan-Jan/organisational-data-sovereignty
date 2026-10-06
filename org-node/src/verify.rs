//! verify-against-chain: the single security property of the PoC. A received
//! envelope is committed only if applying its delta reproduces a root that
//! independently matches the on-chain root at a newer epoch. Nothing about who
//! delivered it is checked (REQ-ag6kqm). See spec §5.2.
use org_members::delta::Delta;
use org_members::hasher::Blake3Hasher;
use org_members::trie::OrgTrie;

use crate::chain::ChainReader;
use crate::envelope::Envelope;
use crate::error::OrgNodeError;
use crate::ids::OrgId;
use crate::sequence::SeqGuard;
use crate::types::Epoch;

pub type Trie = OrgTrie<Blake3Hasher>;

/// Inputs that pin what the receiver already trusts about the org.
pub struct VerifyContext {
    /// The org we expect this envelope to be for.
    pub expected_org_id: OrgId,
    /// Replay guard for this org.
    pub seq_guard: SeqGuard,
    /// The last on-chain epoch this receiver has already committed (0 if none).
    pub last_committed_epoch: Epoch,
}

/// The result of a successful verification: the new committed trie and the
/// advanced guards. Caller persists these atomically.
///
/// `Debug` is derived for test convenience; `OrgTrie`'s `Debug` output includes
/// the full node tree (member leaves redact PII, but it is still large). Avoid
/// logging `VerifiedUpdate` at trace/debug level in production code.
#[derive(Debug)]
pub struct VerifiedUpdate {
    pub trie: Trie,
    pub seq_guard: SeqGuard,
    pub epoch: Epoch,
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

/// The chain-free half of verify-against-chain, so a caller can refuse a
/// stranger's Envelope before it reads the chain (LLR-fuq379, RC-mj6gjq).
/// Touches no `ChainReader`.
pub fn check_chain_free(local_trie: &Trie, envelope: &Envelope, ctx: &VerifyContext) -> Result<(), OrgNodeError> {
    chain_free(local_trie, envelope, ctx).map(|_| ())
}

/// Verify an envelope against the local trie and an independent chain oracle.
///
/// Order is security-critical: cheap checks on what the envelope claims
/// first, chain read and root match last. Returns the committed
/// trie or a typed rejection; never panics, never mutates `local_trie`.
pub fn verify_envelope_against_chain<C: ChainReader>(
    local_trie: &Trie,
    envelope: &Envelope,
    ctx: &VerifyContext,
    chain: &C,
) -> Result<VerifiedUpdate, OrgNodeError> {
    // 1–4. Org binding, replay, decode, base root: no chain read yet.
    let delta = chain_free(local_trie, envelope, ctx)?;
    // 5. Apply → candidate.
    let candidate = local_trie.apply_delta(&delta)?;
    // 6. Independent trusted root + epoch from the chain.
    let on_chain = chain
        .get_org_state(&ctx.expected_org_id)
        .map_err(OrgNodeError::Chain)?
        .ok_or(OrgNodeError::OrgNotOnChain)?;
    if on_chain.epoch <= ctx.last_committed_epoch {
        return Err(OrgNodeError::StaleEpoch { got: on_chain.epoch.get(), last: ctx.last_committed_epoch.get() });
    }
    // The Sequence number is the epoch of the state verified against
    // (REQ-txvtm9), so no sender can set the mark beyond what the chain has
    // reached.
    if envelope.parent_seq.get() != on_chain.epoch.get() {
        return Err(OrgNodeError::SeqNotEpoch { seq: envelope.parent_seq.get(), epoch: on_chain.epoch.get() });
    }
    // 7. The decisive check: recomputed root must equal the on-chain root.
    let committed = candidate
        .verify_against(&on_chain.root_hash)
        .map_err(|_| OrgNodeError::RootMismatch)?;

    let mut seq_guard = ctx.seq_guard;
    seq_guard.advance(envelope.parent_seq);
    Ok(VerifiedUpdate { trie: committed, seq_guard, epoch: on_chain.epoch })
}

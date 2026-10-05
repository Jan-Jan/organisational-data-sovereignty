//! verify-against-chain: the single security property of the PoC. A received
//! envelope is committed only if applying its delta reproduces a root that
//! independently matches the on-chain root at a newer epoch. See spec §5.2.
use ed25519_dalek::VerifyingKey;
use org_members::hasher::Blake3Hasher;
use org_members::trie::OrgTrie;

use crate::chain::ChainReader;
use crate::envelope::SignedDeltaEnvelope;
use crate::error::OrgNodeError;
use crate::ids::OrgId;
use crate::sequence::SeqGuard;
use crate::types::Epoch;

pub type Trie = OrgTrie<Blake3Hasher>;

/// Inputs that pin what the receiver already trusts about the org.
pub struct VerifyContext<'a> {
    /// The org we expect this envelope to be for.
    pub expected_org_id: OrgId,
    /// The author's member key, learned out-of-band / from the trie.
    pub author_member_key: &'a VerifyingKey,
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

/// Verify an envelope against the local trie and an independent chain oracle.
///
/// Order is security-critical: cheap authenticity checks first, chain read and
/// root match last. Returns the committed trie or a typed rejection; never
/// panics, never mutates `local_trie`.
pub fn verify_envelope_against_chain<C: ChainReader>(
    local_trie: &Trie,
    envelope: &SignedDeltaEnvelope,
    ctx: &VerifyContext<'_>,
    chain: &C,
) -> Result<VerifiedUpdate, OrgNodeError> {
    // 1. Org binding.
    if envelope.org_id != ctx.expected_org_id {
        return Err(OrgNodeError::OrgIdMismatch);
    }
    // 2. Authenticity — before touching delta bytes.
    if !envelope.verify_signature(ctx.author_member_key) {
        return Err(OrgNodeError::BadSignature);
    }
    // 3. Replay.
    ctx.seq_guard.check(envelope.parent_seq)?;
    // 4. Decode the delta (typed error on malformed/non-canonical wire form).
    let delta = envelope.decode_delta()?;
    // 5. Base-root must match the local trie (apply_delta also checks this, but
    //    we surface the specific error before doing work).
    if delta.base_root() != &local_trie.root_hash()? {
        return Err(OrgNodeError::DeltaBaseMismatch);
    }
    // 6. Apply → candidate.
    let candidate = local_trie.apply_delta(&delta)?;
    // 7. Independent trusted root + epoch from the chain.
    let on_chain = chain
        .get_org_state(&ctx.expected_org_id)
        .map_err(OrgNodeError::Chain)?
        .ok_or(OrgNodeError::OrgNotOnChain)?;
    if on_chain.epoch <= ctx.last_committed_epoch {
        return Err(OrgNodeError::StaleEpoch { got: on_chain.epoch.get(), last: ctx.last_committed_epoch.get() });
    }
    // 8. The decisive check: recomputed root must equal the on-chain root.
    let committed = candidate
        .verify_against(&on_chain.root_hash)
        .map_err(|_| OrgNodeError::RootMismatch)?;

    let mut seq_guard = ctx.seq_guard;
    seq_guard.advance(envelope.parent_seq);
    Ok(VerifiedUpdate { trie: committed, seq_guard, epoch: on_chain.epoch })
}


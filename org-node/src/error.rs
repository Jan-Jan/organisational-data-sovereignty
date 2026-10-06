//! Typed errors for org-node. Every rejection path in verify-against-chain
//! maps to a distinct variant so the UI can surface *why* a change was rejected.
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum OrgNodeError {
    #[error("envelope org_id does not match the expected org")]
    OrgIdMismatch,

    /// The Organisation state read from the chain carries an Organisation
    /// public key that is not a valid X25519 public key (REQ-8jb4ny).
    #[error("organisation public key read from the chain is not a valid X25519 public key")]
    InvalidOrgPublicKey,

    #[error("stale or replayed parent_seq: got {got}, last seen {last_seen}")]
    StaleSeq { got: u64, last_seen: u64 },

    /// The Envelope's Sequence number is not the epoch of the Organisation
    /// state it was verified against (REQ-txvtm9). Distinct from `StaleSeq`:
    /// the number may be above the mark and still not be the chain's.
    #[error("parent_seq {seq} is not the on-chain epoch {epoch}")]
    SeqNotEpoch { seq: u64, epoch: u64 },

    #[error("envelope delta failed to decode")]
    MalformedDelta,

    #[error("delta base_root does not match the local trie root")]
    DeltaBaseMismatch,

    #[error("no on-chain state found for org")]
    OrgNotOnChain,

    #[error("recomputed root does not match the on-chain root")]
    RootMismatch,

    #[error("on-chain epoch {got} is not newer than the last committed epoch {last}")]
    StaleEpoch { got: u64, last: u64 },

    #[error("chain read failed: {0}")]
    Chain(String),

    #[error("org-members error: {0}")]
    Trie(org_members::OrgMembersError),

    /// A decoded Persona store, record snapshot or Join request holds a value
    /// its type's parse refuses; `field` names it. LLR-8bum44.
    #[error("invalid {field}: {reason}")]
    InvalidField { field: &'static str, reason: String },
}

impl From<org_members::OrgMembersError> for OrgNodeError {
    fn from(e: org_members::OrgMembersError) -> Self {
        OrgNodeError::Trie(e)
    }
}

/// `person`'s parse errors reach org-node as org-members reports them.
impl From<person::IdentityError> for OrgNodeError {
    fn from(e: person::IdentityError) -> Self {
        OrgNodeError::Trie(e.into())
    }
}

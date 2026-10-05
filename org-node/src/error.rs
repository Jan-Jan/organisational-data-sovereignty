//! Typed errors for org-node. Every rejection path in verify-against-chain
//! maps to a distinct variant so the UI can surface *why* a change was rejected.
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum OrgNodeError {
    #[error("envelope org_id does not match the expected org")]
    OrgIdMismatch,

    #[error("envelope signature verification failed")]
    BadSignature,

    #[error("stale or replayed parent_seq: got {got}, last seen {last_seen}")]
    StaleSeq { got: u64, last_seen: u64 },

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

    /// 32 bytes offered as an Organisation public key that do not decompress
    /// to an Edwards point. LLR-mmdu38.
    #[error("invalid key: the bytes are not a point on the curve")]
    InvalidKey,

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

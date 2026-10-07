//! Typed errors for org-node. Every rejection path in verify-against-chain
//! maps to a distinct variant so the UI can surface *why* a change was rejected.
use thiserror::Error;

use crate::ids::OrgId;
use crate::types::{Epoch, PersonaId};

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

    /// A decoded Persona store or record snapshot holds a value
    /// its type's parse refuses; `field` names it. LLR-8bum44.
    #[error("invalid {field}: {reason}")]
    InvalidField { field: &'static str, reason: String },

    /// A first admission whose Organisation matches no expectation the app
    /// declared (LLR-mxskg9, LLR-s8xp7m).
    #[error("first admission not expected for organisation {org_id:?}")]
    AdmissionNotExpected { org_id: OrgId },

    /// A first admission whose verified Membership record lists none of this
    /// node's Personas (LLR-mxskg9, LLR-3f5h7b, REQ-kt877x).
    #[error("first admission to organisation {org_id:?} admits none of this node's personas")]
    AdmissionNotOurs { org_id: OrgId },

    /// Keeping a provisional update would bring one Organisation's
    /// provisional updates above `limit` bytes (LLR-mxskg9, LLR-jq7qh7).
    #[error("provisional updates for one Organisation would exceed {limit} bytes")]
    ProvisionalLimit { limit: usize },

    /// No provisional update this node keeps produces the Organisation state
    /// the chain carries (LLR-mxskg9, LLR-wzqqg9).
    #[error("no provisional update produces the state the chain carries")]
    NoProvisionalUpdate,

    /// The Persona is already bound to an Organisation, and a Persona is
    /// bound to at most one (LLR-mxskg9, LLR-6z5xya, LLR-eyc4ud, REQ-yp75u9).
    #[error("persona {} is already bound to an organisation", persona_id.as_str())]
    PersonaAlreadyBound { persona_id: PersonaId },

    /// A received Wire message that does not decode — an
    /// Organisation-information message without its record snapshot or
    /// Organisation private key among them (LLR-j5vbqj, LLR-xn5pwc,
    /// REQ-c29s93).
    #[error("received wire message is malformed")]
    MalformedMessage,

    /// An Organisation-information message whose Organisation private key's
    /// public half is not the chain's Organisation public key (LLR-j5vbqj,
    /// LLR-ba2ejp, REQ-bwx7eg).
    #[error("organisation private key received for organisation {org_id:?} is not the chain's organisation key")]
    OrgKeyMismatch { org_id: OrgId },

    /// A revocation about an Organisation this node holds no record of
    /// (LLR-j5vbqj, LLR-38e2kn, REQ-vxqc5g).
    #[error("revocation for organisation {org_id:?}, of which this node holds no record")]
    RevocationNotHeld { org_id: OrgId },

    /// A revocation after whose verified Membership record this node's
    /// Device is still listed (LLR-j5vbqj, LLR-pt32fx, REQ-3dsweu).
    #[error("revocation for organisation {org_id:?} leaves this node's device in the record")]
    RevocationNotForThisDevice { org_id: OrgId },

    /// A reconcile for an Organisation of which this node holds no record
    /// (LLR-n67aw8).
    #[error("organisation {org_id:?} is not held by this node")]
    OrgNotHeld { org_id: OrgId },

    /// An Organisation state read from the chain older than the record
    /// (LLR-n67aw8).
    #[error(
        "chain state for organisation {org_id:?} is at epoch {}, older than the record's epoch {}",
        chain_epoch.get(),
        record_epoch.get()
    )]
    StaleChainState { org_id: OrgId, chain_epoch: Epoch, record_epoch: Epoch },

    /// An Organisation state at the record's epoch with a different root
    /// (LLR-n67aw8).
    #[error("chain state for organisation {org_id:?} is at the record's epoch with a different root")]
    ChainStateConflict { org_id: OrgId },

    /// A revocation whose absence proof does not verify (LLR-n67aw8).
    #[error("revocation for organisation {org_id:?} refused: its absence proof does not verify ({cause})")]
    RevocationProofRefused { org_id: OrgId, cause: org_members::OrgMembersError },

    /// An acknowledgement about an Organisation of which this node holds no
    /// record (LLR-n67aw8).
    #[error("acknowledgement for organisation {org_id:?}, of which this node holds no record")]
    AcknowledgementNotHeld { org_id: OrgId },

    /// An acknowledgement naming an epoch after the record's (LLR-n67aw8).
    #[error("acknowledgement for organisation {org_id:?} names an epoch after the record's")]
    AcknowledgementFromFuture { org_id: OrgId },

    /// An acknowledgement from a Device the record still lists (LLR-n67aw8).
    #[error("acknowledgement for organisation {org_id:?} comes from a device the record still lists")]
    AcknowledgementForListedDevice { org_id: OrgId },

    /// An acknowledgement whose signature does not verify (LLR-n67aw8).
    #[error("acknowledgement for organisation {org_id:?} carries a signature that does not verify")]
    AcknowledgementSignatureInvalid { org_id: OrgId },

    /// A Persona bound to the Organisation whose device seed the caller did
    /// not supply (LLR-n67aw8, LLR-hby4jr).
    #[error("device secret for organisation {org_id:?} was not supplied")]
    DeviceSecretNotSupplied { org_id: OrgId },

    /// `send_update` to a Device the record does not list and the outcome
    /// holds no notice for (LLR-n67aw8, LLR-6ymd6d).
    #[error("no revocation for organisation {org_id:?} is held for this recipient")]
    NoRevocationForRecipient { org_id: OrgId },

    /// An update for a held Organisation, or a revocation, from a Device the
    /// record does not list (LLR-2r2fha, LLR-kzgjz8).
    #[error("message for organisation {org_id:?} from a device its record does not list")]
    SenderNotListed { org_id: OrgId },

    /// An acknowledgement delivered by a Device other than the one it names
    /// (LLR-5azhry, LLR-3aysup).
    #[error("acknowledgement for organisation {org_id:?} was not sent by the device it names")]
    AcknowledgementNotFromItsDevice { org_id: OrgId },
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

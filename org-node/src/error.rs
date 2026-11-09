//! Typed errors for org-node. Every refusal of a received body, of a
//! verification against the chain and of an operation maps to a distinct
//! variant, so the UI can surface *why* something was refused.
use thiserror::Error;

use crate::ids::{InviteId, OrgId};
use org_members::{DevicePublicKey, Handle, MemberId};

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

    /// A received Change set's parent sequence is not `epoch`, the next
    /// epoch after the record the receiving store holds (LLR-76bhw9). Raised
    /// on receipt, which reads no chain. Distinct from `StaleSeq`: the
    /// number may be above the mark and still not be the next epoch.
    #[error("the Change set's parent sequence {seq} is not {epoch}, the next epoch after the held record")]
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
    /// its type's parse refuses; `field` names it. LLR-7yu2xy.
    #[error("invalid {field}: {reason}")]
    InvalidField { field: &'static str, reason: String },

    /// A first admission whose Organisation matches no expectation the app
    /// declared (LLR-3sddey, LLR-e6w3nr).
    #[error("first admission not expected for organisation {org_id:?}")]
    AdmissionNotExpected { org_id: OrgId },

    /// A first admission whose verified Membership record lists none of this
    /// node's Personas (LLR-3sddey, LLR-q4puca, REQ-kt877x).
    #[error("first admission to organisation {org_id:?} admits none of this node's personas")]
    AdmissionNotOurs { org_id: OrgId },

    /// Keeping a provisional change, a provisional organisation or a
    /// revocation notice would bring one Organisation's epoch state above
    /// `limit` bytes (LLR-m76r4k, LLR-3sddey).
    #[error("provisional changes and organisations for one Organisation would exceed {limit} bytes")]
    ProvisionalLimit { limit: usize },

    /// A received Wire message that does not decode — among them a message
    /// with no key, or a key that is not 32 bytes (LLR-94rzrk, LLR-xr88a8,
    /// REQ-zxgd46).
    #[error("received wire message is malformed")]
    MalformedMessage,

    /// A Wire message whose encoding is over the bound, refused on encode and,
    /// before decoding, on decode (LLR-sc6zuh, LLR-8kh3zf).
    #[error("wire message too large: {length} bytes (max {max})", max = crate::wire::MAX_MESSAGE)]
    MessageTooLarge { length: usize },

    /// A revocation about an Organisation this node holds no record of
    /// (LLR-3sddey, LLR-95av3e, REQ-vxqc5g).
    #[error("revocation for organisation {org_id:?}, of which this node holds no record")]
    RevocationNotHeld { org_id: OrgId },

    /// A revocation after whose verified Membership record this node's
    /// Device is still listed (LLR-3sddey, LLR-jhy6t6, REQ-3dsweu).
    #[error("revocation for organisation {org_id:?} leaves this node's device in the record")]
    RevocationNotForThisDevice { org_id: OrgId },

    /// An operation on an Organisation of which this node holds no record
    /// (LLR-3sddey).
    #[error("organisation {org_id:?} is not held by this node")]
    OrgNotHeld { org_id: OrgId },

    /// An Organisation state read from the chain older than the record
    /// (LLR-3sddey).
    #[error(
        "chain state for organisation {org_id:?} is at epoch {}, older than the record's epoch {}",
        chain_epoch.get(),
        record_epoch.get()
    )]
    StaleChainState { org_id: OrgId, chain_epoch: Epoch, record_epoch: Epoch },

    /// An Organisation state at the record's epoch with a different root
    /// (LLR-3sddey).
    #[error("chain state for organisation {org_id:?} is at the record's epoch with a different root")]
    ChainStateConflict { org_id: OrgId },

    /// A revocation whose absence proof does not verify (LLR-3sddey).
    #[error("revocation for organisation {org_id:?} refused: its absence proof does not verify ({cause})")]
    RevocationProofRefused { org_id: OrgId, cause: org_members::OrgMembersError },

    /// An acknowledgement about an Organisation of which this node holds no
    /// record (LLR-3sddey).
    #[error("acknowledgement for organisation {org_id:?}, of which this node holds no record")]
    AcknowledgementNotHeld { org_id: OrgId },

    /// An acknowledgement naming an epoch after the record's (LLR-3sddey).
    #[error("acknowledgement for organisation {org_id:?} names an epoch after the record's")]
    AcknowledgementFromFuture { org_id: OrgId },

    /// An acknowledgement from a Device the record still lists (LLR-3sddey).
    #[error("acknowledgement for organisation {org_id:?} comes from a device the record still lists")]
    AcknowledgementForListedDevice { org_id: OrgId },

    /// An acknowledgement whose signature does not verify (LLR-3sddey).
    #[error("acknowledgement for organisation {org_id:?} carries a signature that does not verify")]
    AcknowledgementSignatureInvalid { org_id: OrgId },

    /// A Persona bound to the Organisation whose device seed the caller did
    /// not supply (LLR-3sddey, LLR-5tvn7g).
    #[error("device secret for organisation {org_id:?} was not supplied")]
    DeviceSecretNotSupplied { org_id: OrgId },

    /// An update for a held Organisation, or a revocation, from a Device the
    /// record does not list (LLR-283x6f).
    #[error("message for organisation {org_id:?} from a device its record does not list")]
    SenderNotListed { org_id: OrgId },

    /// An acknowledgement delivered by a Device other than the one it names
    /// (LLR-5azhry, LLR-xudkh4).
    #[error("acknowledgement for organisation {org_id:?} was not sent by the device it names")]
    AcknowledgementNotFromItsDevice { org_id: OrgId },

    /// Sealed store bytes `PersonaStore::unseal` refuses (LLR-bg538c).
    #[error("persona store refused: {0}")]
    StoreRefused(StoreRefusal),

    /// A device secret whose Device public key is not the one named, or a
    /// store whose Persona names another Device (LLR-bg538c).
    #[error("Device seed does not match the Persona store's DevicePublicKey")]
    DeviceSecretMismatch,

    /// Two Persona stores with the same Device public key (LLR-f9x3vf).
    #[error("two Persona stores hold the same DevicePublicKey")]
    DuplicatePersonaStore,

    /// An admission request whose (Organisation, invite identifier) pair the
    /// store does not hold as outstanding (LLR-6gkxpg).
    #[error("invite {invite_id:?} of organisation {org_id:?} is not outstanding at this node")]
    InviteNotOutstanding { org_id: OrgId, invite_id: InviteId },

    /// An expected admission to an Organisation another Persona of this node
    /// holds or already expects (LLR-w2uyyh, REQ-uy7kfq). Names the
    /// Organisation and no Persona.
    #[error("organisation {org_id:?} is already held or awaited by another persona of this node")]
    OrganisationHeldByAnotherPersona { org_id: OrgId },

    /// `propose_updated_organisation` on a store holding no provisional
    /// change (LLR-n6ssue).
    #[error("no provisional change is held to propose")]
    NoProvisionalChange,

    /// The last verification found the chain at a later epoch than the
    /// record; recording or proposing waits for the next one (LLR-zk7fx2).
    #[error("organisation {org_id:?} is behind the chain at epoch {}", chain_epoch.get())]
    OrganisationBehind { org_id: OrgId, chain_epoch: Epoch },

    /// An invite identifier with no kept admission request (LLR-mq7n6u,
    /// LLR-ag8hjd).
    #[error("no admission request is kept for invite {invite_id:?}")]
    NoAdmissionRequestKept { invite_id: InviteId },

    /// An invite identifier an `Admit` change already names (LLR-mq7n6u).
    #[error("an admission is already recorded for invite {invite_id:?}")]
    AdmissionAlreadyRecorded { invite_id: InviteId },

    /// No preferred Handle of the request is free and none was chosen
    /// (LLR-mq7n6u).
    #[error("none of the preferred handles is free and no handle was chosen")]
    NoFreeHandle,

    /// The chosen Handle is held by a member or another recorded admission
    /// (LLR-mq7n6u). The message leaves the Handle out: a Handle is personal
    /// data, which only its `Debug` redacts.
    #[error("the chosen handle is already taken")]
    HandleTaken { handle: Handle },

    /// A MemberId the current record does not hold (LLR-f3hru6).
    #[error("member {member_id:?} is not in the current record")]
    MemberNotHeld { member_id: MemberId },

    /// A MemberId a `Revoke` change already names (LLR-f3hru6).
    #[error("a revocation of member {member_id:?} is already recorded")]
    RevocationAlreadyRecorded { member_id: MemberId },

    /// A store holding no provisional founding (LLR-nx87k5).
    #[error("this store holds no provisional founding")]
    NoProvisionalFounding,

    /// A founding that already records another proxy or Organisation
    /// (LLR-nx87k5, LLR-qqqg4b).
    #[error("the founding already records another proxy or organisation")]
    FoundingProxyAlreadyRecorded,

    /// A founding whose proxy is not recorded, so not yet written
    /// (LLR-rr558z).
    #[error("the founding has no proxy recorded: it is not written to the chain")]
    FoundingNotWritten,

    /// A store holding no provisional Member (LLR-g32hjn).
    #[error("this store holds no provisional member")]
    NoProvisionalMember,

    /// An organisation answer this node did not ask for: it is not behind
    /// (LLR-kd7u2r).
    #[error("organisation {org_id:?} was not requested: this node is not behind")]
    OrganisationNotRequested { org_id: OrgId },

    /// No Persona store with the DevicePublicKey named (LLR-f9x3vf). Boxed
    /// as `AcknowledgementNotKept` is.
    #[error("no persona store with DevicePublicKey {device_key:?} is held")]
    PersonaNotHeld { device_key: Box<DevicePublicKey> },

    /// No Persona store with the Persona identifier named (LLR-437fvx).
    #[error("no persona store with identifier {persona_id:?} is held")]
    PersonaIdNotHeld { persona_id: PersonaId },

    /// An admission request whose DevicePublicKey is not the Device that
    /// sent it (LLR-6gkxpg).
    #[error("admission request for organisation {org_id:?} names a device other than its sender")]
    AdmissionKeyNotSender { org_id: OrgId },

    /// A store this Persona's removal left in the removed form (LLR-myfk8w).
    #[error("this persona was removed from organisation {org_id:?}")]
    OrganisationRemoved { org_id: OrgId },

    /// No kept acknowledgement names this recipient (LLR-85kves). Boxed:
    /// a `DevicePublicKey` holds its decompressed point, and an unboxed one
    /// would make every `Result<_, OrgNodeError>` large (clippy
    /// `result_large_err`).
    #[error("no acknowledgement is kept for {recipient:?}")]
    AcknowledgementNotKept { recipient: Box<DevicePublicKey> },

    /// A drawn Organisation public key equal to a key the Organisation
    /// already holds: the record's Organisation public key, or a
    /// Member-as-a-group key or DevicePublicKey of the record it would sign
    /// (LLR-9qavm7, LLR-qghs95). org-node's own refusal, not org-members'.
    #[error("the drawn organisation public key equals a key the organisation already holds")]
    DuplicateOrgPublicKey,
}

/// Why `PersonaStore::unseal` refused the sealed bytes (LLR-bg538c). No
/// variant carries a byte of the input.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum StoreRefusal {
    /// Fewer bytes than a nonce and an authentication tag.
    #[error("too short to hold a sealed store")]
    TooShort,
    /// The ciphertext does not authenticate under the derived key.
    #[error("the ciphertext does not authenticate")]
    NotAuthentic,
    /// The plaintext does not decode as a store.
    #[error("the plaintext does not decode")]
    Undecodable,
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

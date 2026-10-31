//! The values the handle's read surface returns (SDD-z3ychz): ids, names, public keys, roots, epochs, and the summaries
//! below, which copy the public fields of org-node's records and leave the
//! secrets they hold behind. The queries are `OrgIo`'s own
//! methods, each taking the service's lock inside; the proposal summaries are `propose.rs`'s.

use org_node::store::{MemberSnapshot, PersonaRecordView, PersonaStatus, RecordView};
use org_node::{
    DevicePublicKey, Epoch, Handle, MemberId, Name, OrgId, OrgNodeError, OrgPublicKey, PersonPublicKey, PersonaId, RootHash,
    Surname,
};

use crate::keychain::KeychainError;
use crate::send::SendError;
use crate::store_file::StoreFileError;
use crate::Cause;

/// A Persona's public fields: its Member seed is not copied (and the store
/// holds no device seed).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonaSummary {
    pub persona_id: PersonaId,
    pub org_id: Option<OrgId>,
    /// `None` for a provisional Member, which has no Handle until its admission.
    pub handle: Option<Handle>,
    pub name: Name,
    pub surname: Surname,
    pub member_id: Option<MemberId>,
    pub status: PersonaStatus,
    /// The Persona's DevicePublicKey.
    pub device_key: DevicePublicKey,
}

impl From<PersonaRecordView> for PersonaSummary {
    fn from(record: PersonaRecordView) -> Self {
        Self {
            persona_id: record.persona_id,
            org_id: record.org_id,
            handle: record.handle,
            name: record.name,
            surname: record.surname,
            member_id: record.member_id,
            status: record.status,
            device_key: record.device_key,
        }
    }
}

/// An Organisation's public fields: its root, public key, epoch and Members.
/// The Organisation private key, the proxy account and the kept Change set
/// are not copied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgSummary {
    pub org_id: OrgId,
    /// The Organisation name its store keeps (LLR-pn3hgc).
    pub name: String,
    pub root_hash: RootHash,
    pub org_pub_key: OrgPublicKey,
    pub epoch: Epoch,
    pub members: Vec<MemberSnapshot>,
}

impl OrgSummary {
    pub(crate) fn of(record: &RecordView, name: &str) -> Self {
        Self {
            org_id: record.org_id,
            name: name.to_string(),
            root_hash: record.root_hash,
            org_pub_key: record.org_pub_key,
            epoch: record.epoch,
            members: record.members.clone(),
        }
    }
}

/// What org-io says of the Person: always provisional, since org-io neither writes a Person
/// hash to the chain nor verifies one (LLR-mas8vc).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PersonStatus {
    Provisional,
}

/// The Person's public fields (LLR-mas8vc): its Device seed stays in the keychain, the store
/// key in the open Person store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonSummary {
    pub name: Name,
    pub surname: Surname,
    pub device_key: DevicePublicKey,
    pub person_key: PersonPublicKey,
    pub status: PersonStatus,
}

impl PersonSummary {
    pub(crate) fn of(person: org_node::PersonView) -> Self {
        Self {
            name: person.person.name().clone(),
            surname: person.person.surname().clone(),
            device_key: person.device_key,
            person_key: person.person_key,
            status: PersonStatus::Provisional,
        }
    }
}

/// An Invite's public fields (LLR-npw5zg): the Organisation name and id, the
/// invitee name the inviter typed, the issuing Device, its address text and
/// the invite identifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InviteSummary {
    pub org_name: String,
    pub org_id: OrgId,
    pub invitee_name: String,
    pub issuer: DevicePublicKey,
    pub issuer_addresses: Vec<String>,
    pub invite_id: org_node::invitation::InviteId,
}

/// One admission request a held Organisation keeps (LLR-dm2h8u): no address
/// and no key but the request's DevicePublicKey.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeptAdmissionRequestSummary {
    pub org_id: OrgId,
    pub invite_id: org_node::invitation::InviteId,
    pub name: Name,
    pub surname: Surname,
    pub preferred_handles: Vec<Handle>,
    pub device_key: DevicePublicKey,
    pub invitee_name: String,
}

/// A kept recipient's last outcome in the removal's delivery (LLR-rav7dr).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcknowledgementOutcome {
    /// No round has sent to it yet.
    NotYetSent,
    /// It answered with a verdict other than `Accepted`.
    Refused,
    /// No verdict: a failed dial or stream, no byte in time, no address; the send's own reason.
    NotDelivered { reason: Cause<SendError> },
    /// It answered `Accepted`, but the acceptance was not recorded: the
    /// acknowledgement stays kept and is sent again at the next round
    /// (LLR-6ux6yw).
    AcceptedNotRecorded { reason: AcceptanceFailure },
}

/// Why an acknowledgement a recipient accepted stays kept (LLR-6ux6yw,
/// LLR-rav7dr), with the error received.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcceptanceFailure {
    /// org-node refused to delete it, with the refusal it returned: nothing was written.
    Refused(Cause<OrgNodeError>),
    /// The store write of org-node's deletion failed: the Persona was reloaded from its file,
    /// or, when that failed too, dropped.
    Store(Cause<crate::StoreFailure>),
}

impl std::fmt::Display for AcceptanceFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Refused(error) => write!(formatter, "accepted, but org-node refused to delete it: {error}"),
            Self::Store(error) => write!(formatter, "accepted, but the store write failed: {error}"),
        }
    }
}

/// The refusal or the failed write, typed.
impl std::error::Error for AcceptanceFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Refused(error) => Some(error.get()),
            Self::Store(error) => Some(error.get()),
        }
    }
}

/// The step of a removed Persona's deletion that failed (LLR-3fxxpp,
/// LLR-upqj5g), with the error it received.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeletionFailure {
    /// The store file's rename or removal, or the listing of the `removed-` files.
    StoreFile(StoreFileError),
    /// The keychain item's deletion (shared: the keychain's refusal names a parsed key).
    Keychain(Cause<KeychainError>),
    /// org-node's list of the kept acknowledgements could not be read, so whether the Persona is
    /// still held cannot be told.
    Listing(Cause<OrgNodeError>),
    /// `OrgIo::open` judged no deletion: the handle's clock read before the Unix epoch.
    ClockBeforeEpoch,
    /// A `removed-` file kept beside its Persona's live store file, with its keychain item
    /// (PR-wch3zu).
    LeftoverKept,
}

impl std::fmt::Display for DeletionFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StoreFile(error) => write!(formatter, "store file: {error}"),
            Self::Keychain(error) => write!(formatter, "keychain item: {error}"),
            Self::Listing(error) => write!(formatter, "{LISTING_FAILED}: org-node refused it: {error}"),
            Self::ClockBeforeEpoch => write!(formatter, "no deletion judged at open: {CLOCK_BEFORE_EPOCH}"),
            Self::LeftoverKept => {
                formatter.write_str("removed- file kept: its Persona's store file is live, so its keychain item stays")
            }
        }
    }
}

/// The error the failed step received, typed.
impl std::error::Error for DeletionFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::StoreFile(error) => Some(error),
            Self::Keychain(error) => Some(error.get()),
            Self::Listing(error) => Some(error.get()),
            Self::ClockBeforeEpoch | Self::LeftoverKept => None,
        }
    }
}

/// Why a round of the delivery sent nothing (LLR-bv6wkz, LLR-rav7dr).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RoundFailure {
    /// org-node's list of the kept acknowledgements could not be read.
    Listing(Cause<OrgNodeError>),
    /// The handle's clock read before the Unix epoch.
    ClockBeforeEpoch,
}

impl std::fmt::Display for RoundFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Listing(error) => write!(formatter, "{LISTING_FAILED}: org-node refused it: {error}"),
            Self::ClockBeforeEpoch => write!(formatter, "no round run: {CLOCK_BEFORE_EPOCH}"),
        }
    }
}

/// org-node's refusal, typed, for a failed listing.
impl std::error::Error for RoundFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Listing(error) => Some(error.get()),
            Self::ClockBeforeEpoch => None,
        }
    }
}

/// How a failed read of org-node's list is named.
const LISTING_FAILED: &str = "the list of kept acknowledgements could not be read";

/// How a clock reading before the Unix epoch is named (`UnverifiedReason::ClockBeforeEpoch`'s
/// text, and part of the deletion's and the round's).
pub(crate) const CLOCK_BEFORE_EPOCH: &str = "the clock reads before the Unix epoch";

/// The removal's delivery as it stands (LLR-rav7dr): recipients'
/// DevicePublicKeys and outcomes, never an acknowledgement's bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcknowledgementReport {
    pub org_id: OrgId,
    pub recipients: Vec<(DevicePublicKey, AcknowledgementOutcome)>,
    /// Seconds since the Unix epoch, by the handle's clock.
    pub next_round: Option<u64>,
    /// The kept verification time plus 7 days, in seconds since the Unix
    /// epoch. Still given once the store is dropped and only a deletion
    /// failure is retried; `None` only when no removal is kept to have one:
    /// a `removed-` file kept beside a live store that keeps no removal
    /// (LLR-rav7dr).
    pub deadline: Option<u64>,
    pub last_deletion_failure: Option<DeletionFailure>,
    /// Why the last round sent nothing: org-node refused its list of the kept
    /// acknowledgements, or the clock read before the Unix epoch (LLR-rav7dr,
    /// LLR-bv6wkz); the next due round reads it again.
    pub last_round_failure: Option<RoundFailure>,
}

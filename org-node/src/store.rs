//! The sealed per-Persona store (SDD-asxwp8, SDD-undd7q): one Persona's view
//! of its one Organisation — the current record and the epoch state beside
//! it (LLR-f5wrjx) — AEAD-sealed under a key derived from that Persona's
//! device secret (LLR-2qqqx6), given out as `StoreWrite` bytes and taken back
//! in by `PersonaStore::unseal` (LLR-bg538c, LLR-7yu2xy). org-node names no
//! path and touches no file: org-io writes and reads the bytes.
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use zeroize::Zeroizing;

use org_members::{DevicePublicKey, Handle, MemberId, Name, PersonPublicKey, RootHash, Surname};

pub use crate::error::StoreRefusal;
use crate::ids::OrgId;
use crate::invitation::{EndpointAddresses, InviteId};
use crate::keys::DeviceSecret;
use crate::removal::RawAcknowledgement;
use crate::revocation::{Acknowledgement, RevocationNotice};
use crate::types::{ChainAccount, Epoch, MemberSeed, OrgPrivateKey, OrgPublicKey, PersonaId, SequenceNumber};
use crate::removal::StoreContent;
use crate::verify_org::VerificationTime;
use crate::OrgNodeError;

/// Names the field whose parse refused a decoded value (LLR-7yu2xy).
/// postcard drops the message of an error raised inside `Deserialize`, so a
/// record with a fallible field is decoded as its `Raw…` mirror and parsed
/// here, field by field, instead.
pub(crate) fn parse_field<T, E: core::fmt::Display>(
    field: &'static str,
    parsed: Result<T, E>,
) -> Result<T, OrgNodeError> {
    parsed.map_err(|e| OrgNodeError::InvalidField { field, reason: e.to_string() })
}

/// One to three distinct preferred Handles, each parsed (LLR-7yu2xy,
/// LLR-rjphn4): a Handle that does not parse, the count or a repeat is
/// refused naming `field`.
pub(crate) fn parse_preferred_handles(field: &'static str, raw: &[String]) -> Result<Vec<Handle>, OrgNodeError> {
    let handles = raw.iter().map(|handle| parse_field(field, Handle::parse(handle))).collect::<Result<Vec<_>, _>>()?;
    check_preferred_handles(field, &handles)?;
    Ok(handles)
}

/// One to three pairwise distinct Handles, else `InvalidField` naming `field`.
pub(crate) fn check_preferred_handles(field: &'static str, handles: &[Handle]) -> Result<(), OrgNodeError> {
    if handles.is_empty() || handles.len() > 3 {
        return Err(OrgNodeError::InvalidField {
            field,
            reason: format!("expected one to three preferred handles, got {}", handles.len()),
        });
    }
    for (index, handle) in handles.iter().enumerate() {
        if handles[..index].contains(handle) {
            return Err(OrgNodeError::InvalidField { field, reason: "a preferred handle is repeated".to_owned() });
        }
    }
    Ok(())
}

/// A Persona: the one identity a store holds, bound to at most one
/// Organisation for its life (LLR-pv2rdy, LLR-qqqg4b). The member seed is
/// crate-private and redacted under `Debug`; the record is not `Serialize`
/// outside test builds, and the store encodes it through
/// `PersonaRecordEncoding`. The record holds the Device public key, never the
/// Device seed.
#[derive(Clone, Debug)]
pub struct PersonaRecord {
    pub persona_id: PersonaId,
    pub org_id: Option<OrgId>,
    pub name: Name,
    pub surname: Surname,
    /// Set for a founder at founding, and on a first admission to the Handle
    /// the adopted record gives the Persona's MemberId; `None` for a
    /// provisional Member (LLR-pv2rdy).
    pub handle: Option<Handle>,
    pub(crate) member_seed: MemberSeed,
    pub device_key: DevicePublicKey,
    pub member_id: Option<MemberId>,
    pub status: PersonaStatus,
}

/// Every field of a `PersonaRecord`, public, for tests that build one
/// (test builds only).
#[cfg(feature = "test-support")]
pub struct PersonaRecordParts {
    pub persona_id: PersonaId,
    pub org_id: Option<OrgId>,
    pub name: Name,
    pub surname: Surname,
    pub handle: Option<Handle>,
    pub member_seed: MemberSeed,
    pub device_key: DevicePublicKey,
    pub member_id: Option<MemberId>,
    pub status: PersonaStatus,
}

#[cfg(feature = "test-support")]
impl From<PersonaRecordParts> for PersonaRecord {
    fn from(parts: PersonaRecordParts) -> Self {
        Self {
            persona_id: parts.persona_id,
            org_id: parts.org_id,
            name: parts.name,
            surname: parts.surname,
            handle: parts.handle,
            member_seed: parts.member_seed,
            device_key: parts.device_key,
            member_id: parts.member_id,
            status: parts.status,
        }
    }
}

#[cfg(feature = "test-support")]
impl PersonaRecord {
    /// The stored Member seed (test builds only).
    pub fn member_seed_for_test(&self) -> &MemberSeed {
        &self.member_seed
    }
}

/// A `PersonaRecord` as the store encodes it. Encodes exactly as
/// `RawPersonaRecord` decodes.
#[derive(Serialize)]
struct PersonaRecordEncoding<'a> {
    persona_id: &'a PersonaId,
    org_id: &'a Option<OrgId>,
    name: &'a Name,
    surname: &'a Surname,
    handle: &'a Option<Handle>,
    member_seed: &'a MemberSeed,
    device_key: &'a DevicePublicKey,
    member_id: &'a Option<MemberId>,
    status: &'a PersonaStatus,
}

impl<'a> From<&'a PersonaRecord> for PersonaRecordEncoding<'a> {
    fn from(record: &'a PersonaRecord) -> Self {
        Self {
            persona_id: &record.persona_id,
            org_id: &record.org_id,
            name: &record.name,
            surname: &record.surname,
            handle: &record.handle,
            member_seed: &record.member_seed,
            device_key: &record.device_key,
            member_id: &record.member_id,
            status: &record.status,
        }
    }
}

/// Test builds encode a record as the store does.
#[cfg(feature = "test-support")]
impl Serialize for PersonaRecord {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        PersonaRecordEncoding::from(self).serialize(serializer)
    }
}

/// A `PersonaRecord` as decoded, before parsing (LLR-7yu2xy).
#[derive(Deserialize)]
pub(crate) struct RawPersonaRecord {
    persona_id: PersonaId,
    org_id: Option<OrgId>,
    name: String,
    surname: String,
    handle: Option<String>,
    member_seed: MemberSeed,
    device_key: [u8; 32],
    member_id: Option<MemberId>,
    status: PersonaStatus,
}

impl TryFrom<RawPersonaRecord> for PersonaRecord {
    type Error = OrgNodeError;

    fn try_from(raw: RawPersonaRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            persona_id: raw.persona_id,
            org_id: raw.org_id,
            name: parse_field("persona.name", Name::parse(&raw.name))?,
            surname: parse_field("persona.surname", Surname::parse(&raw.surname))?,
            handle: raw.handle.map(|handle| parse_field("persona.handle", Handle::parse(&handle))).transpose()?,
            member_seed: raw.member_seed,
            device_key: parse_field("persona.device_key", DevicePublicKey::parse(&raw.device_key))?,
            member_id: raw.member_id,
            status: raw.status,
        })
    }
}

/// A `PersonaRecord` decoded on its own goes through the same parse.
impl<'de> Deserialize<'de> for PersonaRecord {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawPersonaRecord::deserialize(deserializer)?;
        PersonaRecord::try_from(raw).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersonaStatus {
    Proposed,
    Active,
    Revoked,
}

/// The current record of the Persona's Organisation (spec §4.3). The
/// Organisation private key is crate-private and the record is not
/// `Serialize` outside test builds; the store encodes it through
/// `OrgRecordEncoding`. It always holds the private key of the epoch it
/// adopted, and no earlier one (LLR-78jfrd).
#[derive(Clone, Debug)]
pub struct OrgRecord {
    pub org_id: OrgId,
    pub root_hash: RootHash,
    pub org_pub_key: OrgPublicKey,
    pub epoch: Epoch,
    pub last_seq: SequenceNumber,
    pub trie_members: Vec<MemberSnapshot>,
    /// The proxy account recorded for a founding, held opaquely and only
    /// handed back (LLR-x5asbx); `None` on a record a first admission made.
    pub proxy_account: Option<ChainAccount>,
    /// The Organisation's X25519 private key, required, as its plain 32
    /// bytes with no option tag (LLR-78jfrd). Renders under `Debug` as its
    /// redacted secret type.
    pub(crate) org_private_key: OrgPrivateKey,
    /// The Change set the adopted epoch was reached by (LLR-p8vpgz).
    pub kept_change_set: Option<Vec<u8>>,
    /// The postcard `Vec<MemberSnapshot>` of the record the kept Change set
    /// extends (LLR-p8vpgz).
    pub kept_base_snapshot: Option<Vec<u8>>,
}

/// Every field of an `OrgRecord`, public, for tests that build one (test
/// builds only).
#[cfg(feature = "test-support")]
pub struct OrgRecordParts {
    pub org_id: OrgId,
    pub root_hash: RootHash,
    pub org_pub_key: OrgPublicKey,
    pub epoch: Epoch,
    pub last_seq: SequenceNumber,
    pub trie_members: Vec<MemberSnapshot>,
    pub proxy_account: Option<ChainAccount>,
    pub org_private_key: OrgPrivateKey,
    pub kept_change_set: Option<Vec<u8>>,
    pub kept_base_snapshot: Option<Vec<u8>>,
}

#[cfg(feature = "test-support")]
impl From<OrgRecordParts> for OrgRecord {
    fn from(parts: OrgRecordParts) -> Self {
        Self {
            org_id: parts.org_id,
            root_hash: parts.root_hash,
            org_pub_key: parts.org_pub_key,
            epoch: parts.epoch,
            last_seq: parts.last_seq,
            trie_members: parts.trie_members,
            proxy_account: parts.proxy_account,
            org_private_key: parts.org_private_key,
            kept_change_set: parts.kept_change_set,
            kept_base_snapshot: parts.kept_base_snapshot,
        }
    }
}

impl OrgRecord {
    /// Whether the record lists `device` as a Device of one of its members.
    pub(crate) fn lists_device(&self, device: &DevicePublicKey) -> bool {
        self.trie_members.iter().any(|member| member.device_keys.contains(device))
    }

    /// Every Device the record lists except `own`, in record order (member,
    /// then slot).
    pub(crate) fn devices_other_than(&self, own: DevicePublicKey) -> impl Iterator<Item = DevicePublicKey> + '_ {
        self.trie_members.iter().flat_map(|member| member.device_keys.iter().copied()).filter(move |device| *device != own)
    }
}

#[cfg(feature = "test-support")]
impl OrgRecord {
    /// The stored Organisation private key (test builds only).
    pub fn org_private_key_for_test(&self) -> &OrgPrivateKey {
        &self.org_private_key
    }
}

/// An `OrgRecord` as the store encodes it. Encodes exactly as `RawOrgRecord`
/// decodes.
#[derive(Serialize)]
struct OrgRecordEncoding<'a> {
    org_id: &'a OrgId,
    root_hash: &'a RootHash,
    org_pub_key: &'a OrgPublicKey,
    epoch: &'a Epoch,
    last_seq: &'a SequenceNumber,
    trie_members: &'a [MemberSnapshot],
    proxy_account: &'a Option<ChainAccount>,
    org_private_key: &'a OrgPrivateKey,
    kept_change_set: &'a Option<Vec<u8>>,
    kept_base_snapshot: &'a Option<Vec<u8>>,
}

impl<'a> From<&'a OrgRecord> for OrgRecordEncoding<'a> {
    fn from(record: &'a OrgRecord) -> Self {
        Self {
            org_id: &record.org_id,
            root_hash: &record.root_hash,
            org_pub_key: &record.org_pub_key,
            epoch: &record.epoch,
            last_seq: &record.last_seq,
            trie_members: &record.trie_members,
            proxy_account: &record.proxy_account,
            org_private_key: &record.org_private_key,
            kept_change_set: &record.kept_change_set,
            kept_base_snapshot: &record.kept_base_snapshot,
        }
    }
}

/// Test builds encode a record as the store does.
#[cfg(feature = "test-support")]
impl Serialize for OrgRecord {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        OrgRecordEncoding::from(self).serialize(serializer)
    }
}

/// An `OrgRecord` as decoded, before parsing (LLR-7yu2xy).
#[derive(Deserialize)]
pub(crate) struct RawOrgRecord {
    org_id: OrgId,
    root_hash: RootHash,
    org_pub_key: [u8; 32],
    epoch: Epoch,
    last_seq: SequenceNumber,
    trie_members: Vec<RawMemberSnapshot>,
    proxy_account: Option<ChainAccount>,
    org_private_key: OrgPrivateKey,
    kept_change_set: Option<Vec<u8>>,
    kept_base_snapshot: Option<Vec<u8>>,
}

impl TryFrom<RawOrgRecord> for OrgRecord {
    type Error = OrgNodeError;

    fn try_from(raw: RawOrgRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            org_id: raw.org_id,
            root_hash: raw.root_hash,
            org_pub_key: parse_field("org.org_pub_key", OrgPublicKey::parse(&raw.org_pub_key))?,
            epoch: raw.epoch,
            last_seq: raw.last_seq,
            trie_members: parse_members(raw.trie_members)?,
            proxy_account: raw.proxy_account,
            org_private_key: raw.org_private_key,
            kept_change_set: raw.kept_change_set,
            kept_base_snapshot: raw.kept_base_snapshot,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawMemberSnapshot")]
pub struct MemberSnapshot {
    pub id: MemberId,
    pub handle: Handle,
    pub name: Name,
    pub surname: Surname,
    pub member_key: PersonPublicKey,
    pub device_keys: Vec<DevicePublicKey>,
}

/// A `MemberSnapshot` as decoded, before parsing (LLR-7yu2xy).
#[derive(Deserialize)]
pub(crate) struct RawMemberSnapshot {
    id: MemberId,
    handle: String,
    name: String,
    surname: String,
    member_key: [u8; 32],
    device_keys: Vec<[u8; 32]>,
}

impl TryFrom<RawMemberSnapshot> for MemberSnapshot {
    type Error = OrgNodeError;

    fn try_from(raw: RawMemberSnapshot) -> Result<Self, Self::Error> {
        Ok(Self {
            id: raw.id,
            handle: parse_field("member.handle", Handle::parse(&raw.handle))?,
            name: parse_field("member.name", Name::parse(&raw.name))?,
            surname: parse_field("member.surname", Surname::parse(&raw.surname))?,
            member_key: parse_field("member.member_key", PersonPublicKey::parse(&raw.member_key))?,
            device_keys: raw
                .device_keys
                .into_iter()
                .map(|key| parse_field("member.device_keys", DevicePublicKey::parse(&key)))
                .collect::<Result<_, _>>()?,
        })
    }
}

/// Each raw member snapshot, parsed (`member.*`).
pub(crate) fn parse_members(raw: Vec<RawMemberSnapshot>) -> Result<Vec<MemberSnapshot>, OrgNodeError> {
    raw.into_iter().map(MemberSnapshot::try_from).collect()
}

/// The bound on one Organisation's provisional changes, provisional
/// organisations and kept revocation notices in their stored form: 1 MiB
/// (LLR-m76r4k). A separate constant from `wire::MAX_MESSAGE`, so changing
/// one is a decision about the other.
pub const MAX_PROVISIONAL_BYTES: usize = 1 << 20;

/// The bound on the epoch state (LLR-m76r4k): the postcard encoding of
/// `(provisional_changes, provisional_organisations,
/// kept_revocation_notices)` as `data` holds them at most
/// `MAX_PROVISIONAL_BYTES`, else `ProvisionalLimit`. Every keep calls it on
/// the copy it changed, so a refusal keeps nothing.
pub(crate) fn ensure_within_bound(data: &StoreData) -> Result<(), OrgNodeError> {
    let organisations: Vec<ProvisionalOrganisationEncoding<'_>> =
        data.provisional_organisations.iter().map(ProvisionalOrganisationEncoding::from).collect();
    let length = postcard::to_allocvec(&(&data.provisional_changes, &organisations, &data.kept_revocation_notices))
        .map_err(|error| OrgNodeError::Chain(format!("epoch state encode: {error}")))?
        .len();
    if length > MAX_PROVISIONAL_BYTES {
        return Err(OrgNodeError::ProvisionalLimit { limit: MAX_PROVISIONAL_BYTES });
    }
    Ok(())
}

/// An Admin's change, recorded and not yet calculated (LLR-b2hgd4): it
/// holds no Membership root, no Change set, no Sequence number and no key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum ProvisionalChange {
    /// Admit the kept admission request for `invite_id` under `handle`.
    Admit { invite_id: InviteId, handle: Handle },
    /// Delete the member `member_id`.
    Revoke { member_id: MemberId },
}

impl ProvisionalChange {
    /// Whether this change is the `Admit` of the request kept for `invite_id`.
    pub(crate) fn admits(&self, invite_id: InviteId) -> bool {
        matches!(self, ProvisionalChange::Admit { invite_id: named, .. } if *named == invite_id)
    }
}

/// A `ProvisionalChange` as decoded, before parsing.
#[derive(Deserialize)]
pub(crate) enum RawProvisionalChange {
    Admit { invite_id: InviteId, handle: String },
    Revoke { member_id: MemberId },
}

impl TryFrom<RawProvisionalChange> for ProvisionalChange {
    type Error = OrgNodeError;

    fn try_from(raw: RawProvisionalChange) -> Result<Self, Self::Error> {
        Ok(match raw {
            RawProvisionalChange::Admit { invite_id, handle } => {
                ProvisionalChange::Admit { invite_id, handle: parse_field("provisional.handle", Handle::parse(&handle))? }
            }
            RawProvisionalChange::Revoke { member_id } => ProvisionalChange::Revoke { member_id },
        })
    }
}

/// Where a provisional organisation came from (LLR-s3vq6d). Origin plays no
/// part in the match that adopts one (LLR-8528ud).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Origin {
    Own,
    Received { from: DevicePublicKey },
    Answer { from: DevicePublicKey },
}

#[derive(Deserialize)]
pub(crate) enum RawOrigin {
    Own,
    Received { from: [u8; 32] },
    Answer { from: [u8; 32] },
}

impl TryFrom<RawOrigin> for Origin {
    type Error = OrgNodeError;

    fn try_from(raw: RawOrigin) -> Result<Self, Self::Error> {
        let parse = |key: [u8; 32]| parse_field("provisional.origin", DevicePublicKey::parse(&key));
        Ok(match raw {
            RawOrigin::Own => Origin::Own,
            RawOrigin::Received { from } => Origin::Received { from: parse(from)? },
            RawOrigin::Answer { from } => Origin::Answer { from: parse(from)? },
        })
    }
}

/// One calculated next epoch, kept until verification adopts or drops it
/// (LLR-s3vq6d). Its identity is `(root, org_pub_key)`; `org_pub_key` is the
/// X25519 public half of `org_private_key`, which is crate-private
/// (LLR-qw8tyu). Not `Serialize` outside test builds.
#[derive(Clone, Debug)]
pub struct ProvisionalOrganisation {
    pub epoch: Epoch,
    pub base_root: RootHash,
    pub members: Vec<MemberSnapshot>,
    pub root: RootHash,
    pub org_pub_key: OrgPublicKey,
    pub(crate) org_private_key: OrgPrivateKey,
    pub change_set: Option<Vec<u8>>,
    pub base_snapshot: Option<Vec<u8>>,
    pub origin: Origin,
    pub applied: Vec<ProvisionalChange>,
    /// The time of the first verification that found it one epoch ahead
    /// of the chain (LLR-4tsgfh); `None` until then, and always for an
    /// `Own` one.
    pub first_ahead_at: Option<VerificationTime>,
}

/// Every field of a `ProvisionalOrganisation`, public (test builds only).
#[cfg(feature = "test-support")]
pub struct ProvisionalOrganisationParts {
    pub epoch: Epoch,
    pub base_root: RootHash,
    pub members: Vec<MemberSnapshot>,
    pub root: RootHash,
    pub org_pub_key: OrgPublicKey,
    pub org_private_key: OrgPrivateKey,
    pub change_set: Option<Vec<u8>>,
    pub base_snapshot: Option<Vec<u8>>,
    pub origin: Origin,
    pub applied: Vec<ProvisionalChange>,
    pub first_ahead_at: Option<VerificationTime>,
}

#[cfg(feature = "test-support")]
impl From<ProvisionalOrganisationParts> for ProvisionalOrganisation {
    fn from(parts: ProvisionalOrganisationParts) -> Self {
        Self {
            epoch: parts.epoch,
            base_root: parts.base_root,
            members: parts.members,
            root: parts.root,
            org_pub_key: parts.org_pub_key,
            org_private_key: parts.org_private_key,
            change_set: parts.change_set,
            base_snapshot: parts.base_snapshot,
            origin: parts.origin,
            applied: parts.applied,
            first_ahead_at: parts.first_ahead_at,
        }
    }
}

#[cfg(feature = "test-support")]
impl ProvisionalOrganisation {
    /// The Organisation private key it holds (test builds only).
    pub fn org_private_key_for_test(&self) -> &OrgPrivateKey {
        &self.org_private_key
    }
}

/// A provisional organisation as `OrgService::provisional_organisations`
/// returns it (LLR-8ru6mv): its public values and whether it is the
/// current proposal; no private key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProvisionalOrganisationView {
    pub epoch: Epoch,
    pub base_epoch: Epoch,
    pub root: RootHash,
    pub org_pub_key: OrgPublicKey,
    pub origin: Origin,
    /// An `Own` proposal whose base is the current record and whose
    /// applied changes are the store's provisional changes.
    pub current_proposal: bool,
}

/// One Persona's store as `OrgService::persona_view` returns it
/// (LLR-8ru6mv): copies of its public values; no
/// seed, no secret and no private key. How org-io reads a store in every
/// build (`store_data` is test-support only).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonaView {
    pub device_key: DevicePublicKey,
    /// The Organisation the Persona is bound to (LLR-qqqg4b).
    pub org_id: Option<OrgId>,
    pub org_name: String,
    pub record: Option<RecordView>,
    pub founding: Option<FoundingView>,
    /// The Organisation a provisional Member asked to join.
    pub member_of: Option<OrgId>,
}

/// A Persona record's public values, as `OrgService::persona_records`
/// lists them (LLR-8ru6mv): every field of `PersonaRecord` but the member
/// seed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonaRecordView {
    pub persona_id: PersonaId,
    pub org_id: Option<OrgId>,
    pub name: Name,
    pub surname: Surname,
    pub handle: Option<Handle>,
    pub device_key: DevicePublicKey,
    pub member_id: Option<MemberId>,
    pub status: PersonaStatus,
}

impl PersonaRecord {
    /// This record's public values (LLR-8ru6mv).
    pub(crate) fn view(&self) -> PersonaRecordView {
        PersonaRecordView {
            persona_id: self.persona_id.clone(),
            org_id: self.org_id,
            name: self.name.clone(),
            surname: self.surname.clone(),
            handle: self.handle.clone(),
            device_key: self.device_key,
            member_id: self.member_id,
            status: self.status,
        }
    }
}

/// The current record's public values (LLR-8ru6mv).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordView {
    pub org_id: OrgId,
    pub root_hash: RootHash,
    pub org_pub_key: OrgPublicKey,
    pub epoch: Epoch,
    pub members: Vec<MemberSnapshot>,
}

/// A provisional founding's public values and its recorded proxy
/// (LLR-8ru6mv).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoundingView {
    pub epoch: Epoch,
    pub root: RootHash,
    pub org_pub_key: OrgPublicKey,
    pub proxy: Option<ChainAccount>,
}

#[derive(Serialize)]
struct ProvisionalOrganisationEncoding<'a> {
    epoch: &'a Epoch,
    base_root: &'a RootHash,
    members: &'a [MemberSnapshot],
    root: &'a RootHash,
    org_pub_key: &'a OrgPublicKey,
    org_private_key: &'a OrgPrivateKey,
    change_set: &'a Option<Vec<u8>>,
    base_snapshot: &'a Option<Vec<u8>>,
    origin: &'a Origin,
    applied: &'a [ProvisionalChange],
    first_ahead_at: &'a Option<VerificationTime>,
}

impl<'a> From<&'a ProvisionalOrganisation> for ProvisionalOrganisationEncoding<'a> {
    fn from(organisation: &'a ProvisionalOrganisation) -> Self {
        Self {
            epoch: &organisation.epoch,
            base_root: &organisation.base_root,
            members: &organisation.members,
            root: &organisation.root,
            org_pub_key: &organisation.org_pub_key,
            org_private_key: &organisation.org_private_key,
            change_set: &organisation.change_set,
            base_snapshot: &organisation.base_snapshot,
            origin: &organisation.origin,
            applied: &organisation.applied,
            first_ahead_at: &organisation.first_ahead_at,
        }
    }
}

/// Test builds encode a provisional organisation as the store does.
#[cfg(feature = "test-support")]
impl Serialize for ProvisionalOrganisation {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ProvisionalOrganisationEncoding::from(self).serialize(serializer)
    }
}

#[derive(Deserialize)]
pub(crate) struct RawProvisionalOrganisation {
    epoch: Epoch,
    base_root: RootHash,
    members: Vec<RawMemberSnapshot>,
    root: RootHash,
    org_pub_key: [u8; 32],
    org_private_key: OrgPrivateKey,
    change_set: Option<Vec<u8>>,
    base_snapshot: Option<Vec<u8>>,
    origin: RawOrigin,
    applied: Vec<RawProvisionalChange>,
    first_ahead_at: Option<VerificationTime>,
}

impl TryFrom<RawProvisionalOrganisation> for ProvisionalOrganisation {
    type Error = OrgNodeError;

    fn try_from(raw: RawProvisionalOrganisation) -> Result<Self, Self::Error> {
        Ok(Self {
            epoch: raw.epoch,
            base_root: raw.base_root,
            members: parse_members(raw.members)?,
            root: raw.root,
            org_pub_key: parse_field("provisional.org_pub_key", OrgPublicKey::parse(&raw.org_pub_key))?,
            org_private_key: raw.org_private_key,
            change_set: raw.change_set,
            base_snapshot: raw.base_snapshot,
            origin: Origin::try_from(raw.origin)?,
            applied: raw.applied.into_iter().map(ProvisionalChange::try_from).collect::<Result<_, _>>()?,
            first_ahead_at: raw.first_ahead_at,
        })
    }
}

/// A founding proposed and not yet adopted (LLR-d4gpxm): the founding
/// record, its root, the Organisation key pair drawn for it and, once
/// written, its pure proxy (LLR-nx87k5). The private key is crate-private.
#[derive(Clone, Debug)]
pub struct ProvisionalFounding {
    pub epoch: Epoch,
    pub members: Vec<MemberSnapshot>,
    pub root: RootHash,
    pub org_pub_key: OrgPublicKey,
    pub(crate) org_private_key: OrgPrivateKey,
    pub proxy: Option<ChainAccount>,
}

/// Every field of a `ProvisionalFounding`, public (test builds only).
#[cfg(feature = "test-support")]
pub struct ProvisionalFoundingParts {
    pub epoch: Epoch,
    pub members: Vec<MemberSnapshot>,
    pub root: RootHash,
    pub org_pub_key: OrgPublicKey,
    pub org_private_key: OrgPrivateKey,
    pub proxy: Option<ChainAccount>,
}

#[cfg(feature = "test-support")]
impl From<ProvisionalFoundingParts> for ProvisionalFounding {
    fn from(parts: ProvisionalFoundingParts) -> Self {
        Self {
            epoch: parts.epoch,
            members: parts.members,
            root: parts.root,
            org_pub_key: parts.org_pub_key,
            org_private_key: parts.org_private_key,
            proxy: parts.proxy,
        }
    }
}

#[cfg(feature = "test-support")]
impl ProvisionalFounding {
    /// The Organisation private key it holds (test builds only).
    pub fn org_private_key_for_test(&self) -> &OrgPrivateKey {
        &self.org_private_key
    }
}

#[derive(Serialize)]
struct ProvisionalFoundingEncoding<'a> {
    epoch: &'a Epoch,
    members: &'a [MemberSnapshot],
    root: &'a RootHash,
    org_pub_key: &'a OrgPublicKey,
    org_private_key: &'a OrgPrivateKey,
    proxy: &'a Option<ChainAccount>,
}

impl<'a> From<&'a ProvisionalFounding> for ProvisionalFoundingEncoding<'a> {
    fn from(founding: &'a ProvisionalFounding) -> Self {
        Self {
            epoch: &founding.epoch,
            members: &founding.members,
            root: &founding.root,
            org_pub_key: &founding.org_pub_key,
            org_private_key: &founding.org_private_key,
            proxy: &founding.proxy,
        }
    }
}

/// Test builds encode a provisional founding as the store does.
#[cfg(feature = "test-support")]
impl Serialize for ProvisionalFounding {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ProvisionalFoundingEncoding::from(self).serialize(serializer)
    }
}

#[derive(Deserialize)]
pub(crate) struct RawProvisionalFounding {
    epoch: Epoch,
    members: Vec<RawMemberSnapshot>,
    root: RootHash,
    org_pub_key: [u8; 32],
    org_private_key: OrgPrivateKey,
    proxy: Option<ChainAccount>,
}

impl TryFrom<RawProvisionalFounding> for ProvisionalFounding {
    type Error = OrgNodeError;

    fn try_from(raw: RawProvisionalFounding) -> Result<Self, Self::Error> {
        Ok(Self {
            epoch: raw.epoch,
            members: parse_members(raw.members)?,
            root: raw.root,
            org_pub_key: parse_field("founding.org_pub_key", OrgPublicKey::parse(&raw.org_pub_key))?,
            org_private_key: raw.org_private_key,
            proxy: raw.proxy,
        })
    }
}

/// The Persona of a store that asked to join from an Invite, before its
/// first adoption (LLR-6acxu9): no secret, every value parsed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProvisionalMember {
    pub org_id: OrgId,
    pub invite_id: InviteId,
    pub issuer: DevicePublicKey,
    pub issuer_addresses: EndpointAddresses,
    pub name: Name,
    pub surname: Surname,
    pub preferred_handles: Vec<Handle>,
    pub member_key: PersonPublicKey,
    pub device_key: DevicePublicKey,
}

#[derive(Deserialize)]
pub(crate) struct RawProvisionalMember {
    org_id: OrgId,
    invite_id: InviteId,
    issuer: [u8; 32],
    issuer_addresses: Vec<String>,
    name: String,
    surname: String,
    preferred_handles: Vec<String>,
    member_key: [u8; 32],
    device_key: [u8; 32],
}

impl TryFrom<RawProvisionalMember> for ProvisionalMember {
    type Error = OrgNodeError;

    fn try_from(raw: RawProvisionalMember) -> Result<Self, Self::Error> {
        Ok(Self {
            org_id: raw.org_id,
            invite_id: raw.invite_id,
            issuer: parse_field("provisional_member.issuer", DevicePublicKey::parse(&raw.issuer))?,
            issuer_addresses: parse_field(
                "provisional_member.issuer_addresses",
                EndpointAddresses::parse(raw.issuer_addresses),
            )?,
            name: parse_field("provisional_member.name", Name::parse(&raw.name))?,
            surname: parse_field("provisional_member.surname", Surname::parse(&raw.surname))?,
            preferred_handles: parse_preferred_handles("provisional_member.preferred_handles", &raw.preferred_handles)?,
            member_key: parse_field("provisional_member.member_key", PersonPublicKey::parse(&raw.member_key))?,
            device_key: parse_field("provisional_member.device_key", DevicePublicKey::parse(&raw.device_key))?,
        })
    }
}

/// An admission request kept as received (LLR-v3av9t): the Device the
/// transport authenticated (`from`, equal to `device_key`) and the request's
/// values, each parsed, with the addresses it carried.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct KeptAdmissionRequest {
    pub org_id: OrgId,
    pub invite_id: InviteId,
    pub from: DevicePublicKey,
    pub name: Name,
    pub surname: Surname,
    pub preferred_handles: Vec<Handle>,
    pub member_key: PersonPublicKey,
    pub device_key: DevicePublicKey,
    pub addresses: EndpointAddresses,
}

#[derive(Deserialize)]
pub(crate) struct RawKeptAdmissionRequest {
    org_id: OrgId,
    invite_id: InviteId,
    from: [u8; 32],
    name: String,
    surname: String,
    preferred_handles: Vec<String>,
    member_key: [u8; 32],
    device_key: [u8; 32],
    addresses: Vec<String>,
}

impl TryFrom<RawKeptAdmissionRequest> for KeptAdmissionRequest {
    type Error = OrgNodeError;

    fn try_from(raw: RawKeptAdmissionRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            org_id: raw.org_id,
            invite_id: raw.invite_id,
            from: parse_field("admission.from", DevicePublicKey::parse(&raw.from))?,
            name: parse_field("admission.name", Name::parse(&raw.name))?,
            surname: parse_field("admission.surname", Surname::parse(&raw.surname))?,
            preferred_handles: parse_preferred_handles("admission.preferred_handles", &raw.preferred_handles)?,
            member_key: parse_field("admission.member_key", PersonPublicKey::parse(&raw.member_key))?,
            device_key: parse_field("admission.device_key", DevicePublicKey::parse(&raw.device_key))?,
            addresses: parse_field("admission.addresses", EndpointAddresses::parse(raw.addresses))?,
        })
    }
}

/// A first admission the store expects: the Organisation alone (REQ-8amu2a).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpectedAdmission {
    pub org_id: OrgId,
}

/// An Invite this store's Persona issued and not yet settled, with the
/// invitee name the inviter typed for it (LLR-f5wrjx, LLR-h9f5c9).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutstandingInvite {
    pub org_id: OrgId,
    pub invite_id: InviteId,
    pub invitee_name: String,
}

/// A revocation notice kept with the Device the transport authenticated for it (LLR-gb7p7c).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct KeptNotice {
    pub notice: RevocationNotice,
    pub from: DevicePublicKey,
}

#[derive(Deserialize)]
pub(crate) struct RawKeptNotice {
    notice: RevocationNotice,
    from: [u8; 32],
}

impl TryFrom<RawKeptNotice> for KeptNotice {
    type Error = OrgNodeError;

    fn try_from(raw: RawKeptNotice) -> Result<Self, Self::Error> {
        Ok(Self {
            notice: raw.notice,
            from: parse_field("org.kept_revocation_notices", DevicePublicKey::parse(&raw.from))?,
        })
    }
}

/// A Device an adopted epoch dropped, held as revoked (LLR-bgj2ff).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct RevokedDevice {
    pub member_id: MemberId,
    pub device_key: DevicePublicKey,
}

#[derive(Deserialize)]
pub(crate) struct RawRevokedDevice {
    member_id: MemberId,
    device_key: [u8; 32],
}

impl TryFrom<RawRevokedDevice> for RevokedDevice {
    type Error = OrgNodeError;

    fn try_from(raw: RawRevokedDevice) -> Result<Self, Self::Error> {
        Ok(Self {
            member_id: raw.member_id,
            device_key: parse_field("org.revoked_devices", DevicePublicKey::parse(&raw.device_key))?,
        })
    }
}

/// An entry of the acknowledged record: an acknowledgement accepted from a Device held as revoked,
/// with the MemberId that Device was listed under (LLR-py43n3).
// TODO(S5): the S5 revoked-device CRDT replaces this record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AcknowledgedEntry {
    pub acknowledgement: Acknowledgement,
    pub member_id: MemberId,
}

#[derive(Deserialize)]
pub(crate) struct RawAcknowledgedEntry {
    acknowledgement: RawAcknowledgement,
    member_id: MemberId,
}

impl TryFrom<RawAcknowledgedEntry> for AcknowledgedEntry {
    type Error = OrgNodeError;

    fn try_from(raw: RawAcknowledgedEntry) -> Result<Self, Self::Error> {
        Ok(Self { acknowledgement: raw.acknowledgement.parse("org.acknowledged")?, member_id: raw.member_id })
    }
}

/// One Persona's view of one Organisation (LLR-f5wrjx), in encoding order:
/// the Persona, the current record, the Organisation name and the epoch
/// state beside the record, the acknowledged record last. `behind` is the one field that is not encoded:
/// it lives in the open store and is `None` after `unseal` (LLR-zk7fx2). Not
/// `Serialize` outside test builds: it holds the member seed and
/// Organisation private keys; `seal` encodes it through `StoreDataEncoding`.
#[derive(Clone, Debug)]
pub struct StoreData {
    pub persona: PersonaRecord,
    pub record: Option<OrgRecord>,
    pub org_name: String,
    pub provisional_changes: Vec<ProvisionalChange>,
    pub provisional_organisations: Vec<ProvisionalOrganisation>,
    pub founding: Option<ProvisionalFounding>,
    pub member: Option<ProvisionalMember>,
    pub kept_admission_requests: Vec<KeptAdmissionRequest>,
    pub kept_revocation_notices: Vec<KeptNotice>,
    pub expected_admissions: Vec<ExpectedAdmission>,
    pub outstanding_invites: Vec<OutstandingInvite>,
    // TODO(S5): the S5 revoked-device CRDT replaces revoked_devices (LLR-bgj2ff).
    pub revoked_devices: Vec<RevokedDevice>,
    pub address_book: Vec<(DevicePublicKey, EndpointAddresses)>,
    /// The acknowledged record (LLR-py43n3): one entry per Device held as revoked, only appended.
    // TODO(S5): the S5 revoked-device CRDT replaces this record.
    pub acknowledged: Vec<AcknowledgedEntry>,
    pub behind: Option<Epoch>,
}

impl StoreData {
    /// A store holding `persona` and nothing else: no record, an empty
    /// Organisation name and no epoch state.
    pub fn new(persona: PersonaRecord) -> Self {
        Self {
            persona,
            record: None,
            org_name: String::new(),
            provisional_changes: Vec::new(),
            provisional_organisations: Vec::new(),
            founding: None,
            member: None,
            kept_admission_requests: Vec::new(),
            kept_revocation_notices: Vec::new(),
            expected_admissions: Vec::new(),
            outstanding_invites: Vec::new(),
            revoked_devices: Vec::new(),
            address_book: Vec::new(),
            acknowledged: Vec::new(),
            behind: None,
        }
    }

    /// Set the address-book entry of `device_key` to `addresses`, replacing
    /// an earlier one (LLR-h9d65p). One entry per DevicePublicKey.
    pub(crate) fn set_address(&mut self, device_key: DevicePublicKey, addresses: EndpointAddresses) {
        match self.address_book.iter_mut().find(|(key, _)| *key == device_key) {
            Some((_, kept)) => *kept = addresses,
            None => self.address_book.push((device_key, addresses)),
        }
    }

    /// The admission request kept for `invite_id`, if any.
    pub(crate) fn kept_request(&self, invite_id: InviteId) -> Option<&KeptAdmissionRequest> {
        self.kept_admission_requests.iter().find(|request| request.invite_id == invite_id)
    }

    /// Whether a provisional organisation of `root` and `org_pub_key` is
    /// held already.
    pub(crate) fn holds_provisional_identity(&self, root: &RootHash, org_pub_key: &OrgPublicKey) -> bool {
        self.provisional_organisations.iter().any(|held| held.root == *root && held.org_pub_key == *org_pub_key)
    }
}

/// The store plaintext as decoded, before parsing (LLR-7yu2xy).
#[derive(Deserialize)]
pub(crate) struct RawStoreData {
    persona: RawPersonaRecord,
    record: Option<RawOrgRecord>,
    org_name: String,
    provisional_changes: Vec<RawProvisionalChange>,
    provisional_organisations: Vec<RawProvisionalOrganisation>,
    founding: Option<RawProvisionalFounding>,
    member: Option<RawProvisionalMember>,
    kept_admission_requests: Vec<RawKeptAdmissionRequest>,
    kept_revocation_notices: Vec<RawKeptNotice>,
    expected_admissions: Vec<ExpectedAdmission>,
    outstanding_invites: Vec<OutstandingInvite>,
    revoked_devices: Vec<RawRevokedDevice>,
    address_book: Vec<([u8; 32], Vec<String>)>,
    acknowledged: Vec<RawAcknowledgedEntry>,
}

impl TryFrom<RawStoreData> for StoreData {
    type Error = OrgNodeError;

    fn try_from(raw: RawStoreData) -> Result<Self, Self::Error> {
        Ok(Self {
            persona: PersonaRecord::try_from(raw.persona)?,
            record: raw.record.map(OrgRecord::try_from).transpose()?,
            org_name: raw.org_name,
            provisional_changes: raw
                .provisional_changes
                .into_iter()
                .map(ProvisionalChange::try_from)
                .collect::<Result<_, _>>()?,
            provisional_organisations: raw
                .provisional_organisations
                .into_iter()
                .map(ProvisionalOrganisation::try_from)
                .collect::<Result<_, _>>()?,
            founding: raw.founding.map(ProvisionalFounding::try_from).transpose()?,
            member: raw.member.map(ProvisionalMember::try_from).transpose()?,
            kept_admission_requests: raw
                .kept_admission_requests
                .into_iter()
                .map(KeptAdmissionRequest::try_from)
                .collect::<Result<_, _>>()?,
            kept_revocation_notices: raw
                .kept_revocation_notices
                .into_iter()
                .map(KeptNotice::try_from)
                .collect::<Result<_, _>>()?,
            expected_admissions: raw.expected_admissions,
            outstanding_invites: raw.outstanding_invites,
            revoked_devices: raw.revoked_devices.into_iter().map(RevokedDevice::try_from).collect::<Result<_, _>>()?,
            address_book: raw
                .address_book
                .into_iter()
                .map(|(key, addresses)| {
                    Ok((
                        parse_field("org.address_book", DevicePublicKey::parse(&key))?,
                        parse_field("org.address_book", EndpointAddresses::parse(addresses))?,
                    ))
                })
                .collect::<Result<_, OrgNodeError>>()?,
            acknowledged: raw
                .acknowledged
                .into_iter()
                .map(AcknowledgedEntry::try_from)
                .collect::<Result<_, _>>()?,
            behind: None,
        })
    }
}

/// A `StoreData` decoded on its own goes through the same parse as `unseal`.
impl<'de> Deserialize<'de> for StoreData {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawStoreData::deserialize(deserializer)?;
        StoreData::try_from(raw).map_err(serde::de::Error::custom)
    }
}

/// `StoreData` as the store encodes it: every field but `behind`.
#[derive(Serialize)]
struct StoreDataEncoding<'a> {
    persona: PersonaRecordEncoding<'a>,
    record: Option<OrgRecordEncoding<'a>>,
    org_name: &'a str,
    provisional_changes: &'a [ProvisionalChange],
    provisional_organisations: Vec<ProvisionalOrganisationEncoding<'a>>,
    founding: Option<ProvisionalFoundingEncoding<'a>>,
    member: &'a Option<ProvisionalMember>,
    kept_admission_requests: &'a [KeptAdmissionRequest],
    kept_revocation_notices: &'a [KeptNotice],
    expected_admissions: &'a [ExpectedAdmission],
    outstanding_invites: &'a [OutstandingInvite],
    revoked_devices: &'a [RevokedDevice],
    address_book: &'a [(DevicePublicKey, EndpointAddresses)],
    acknowledged: &'a [AcknowledgedEntry],
}

impl<'a> From<&'a StoreData> for StoreDataEncoding<'a> {
    fn from(data: &'a StoreData) -> Self {
        Self {
            persona: PersonaRecordEncoding::from(&data.persona),
            record: data.record.as_ref().map(OrgRecordEncoding::from),
            org_name: &data.org_name,
            provisional_changes: &data.provisional_changes,
            provisional_organisations: data.provisional_organisations.iter().map(ProvisionalOrganisationEncoding::from).collect(),
            founding: data.founding.as_ref().map(ProvisionalFoundingEncoding::from),
            member: &data.member,
            kept_admission_requests: &data.kept_admission_requests,
            kept_revocation_notices: &data.kept_revocation_notices,
            expected_admissions: &data.expected_admissions,
            outstanding_invites: &data.outstanding_invites,
            revoked_devices: &data.revoked_devices,
            address_book: &data.address_book,
            acknowledged: &data.acknowledged,
        }
    }
}

/// Test builds encode the store's data as `seal` does.
#[cfg(feature = "test-support")]
impl Serialize for StoreData {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        StoreDataEncoding::from(self).serialize(serializer)
    }
}

/// A sealed Persona store: `nonce(24) ‖ ciphertext`, the bytes org-io writes
/// to and reads from that Persona's file (LLR-bg538c).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SealedStore(Vec<u8>);

impl SealedStore {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// One store's change, sealed: the bytes org-io writes to the file of the
/// Persona whose DevicePublicKey it names.
#[must_use = "org-io must write the sealed store before reporting success (org-io's write-before-success requirement)"]
#[derive(Debug)]
pub struct StoreWrite {
    pub device_key: DevicePublicKey,
    pub sealed: SealedStore,
}

/// The nonce, then the authentication tag: the shortest sealed store.
const NONCE_LENGTH: usize = 24;
const TAG_LENGTH: usize = 16;

/// The HKDF info string of the store key (LLR-2qqqx6): versioned, so a
/// change of derivation is a new label.
const STORE_KEY_INFO: &[u8] = b"ods/persona-store/v1";

/// One Persona's open store: the store key, held only as the cipher
/// (LLR-2qqqx6), the DevicePublicKey it was opened for, and its content
/// (LLR-6zaeb6). No path: org-node touches no file.
pub struct PersonaStore {
    key: XChaCha20Poly1305,
    device_key: DevicePublicKey,
    content: StoreContent,
}

/// The store encryption key (LLR-scgk5j): private to this file, no
/// `Display`, not `Copy`, redacted `Debug`, wiped on drop.
struct StoreKey(Zeroizing<[u8; 32]>);

impl core::fmt::Debug for StoreKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("StoreKey([REDACTED])")
    }
}

/// HKDF-SHA-256 of the device seed, salted with the Device public key, under
/// `label` (LLR-2qqqx6): `STORE_KEY_INFO` for a Persona store, the
/// Person store's own label for the Person store (LLR-73sgz2).
fn derive_key(secret: &DeviceSecret, device_key: &DevicePublicKey, label: &[u8]) -> Result<StoreKey, OrgNodeError> {
    let mut key = Zeroizing::new([0u8; 32]);
    Hkdf::<Sha256>::new(Some(device_key.as_bytes()), secret.seed())
        .expand(label, key.as_mut())
        .map_err(|_| OrgNodeError::Chain("store key derivation failed".into()))?;
    Ok(StoreKey(key))
}

/// The AEAD cipher every sealed store is sealed and opened with: the one name
/// for the library's type outside this module.
pub(crate) type StoreCipher = XChaCha20Poly1305;

/// The cipher of the store key `secret` derives for `device_key` under
/// `label`.
pub(crate) fn labelled_cipher(
    secret: &DeviceSecret,
    device_key: &DevicePublicKey,
    label: &[u8],
) -> Result<XChaCha20Poly1305, OrgNodeError> {
    let key = derive_key(secret, device_key, label)?;
    XChaCha20Poly1305::new_from_slice(key.0.as_ref()).map_err(|error| OrgNodeError::Chain(format!("bad key length: {error}")))
}

/// The cipher of the Persona store key `secret` derives for `device_key`.
pub(crate) fn cipher(secret: &DeviceSecret, device_key: &DevicePublicKey) -> Result<XChaCha20Poly1305, OrgNodeError> {
    labelled_cipher(secret, device_key, STORE_KEY_INFO)
}

/// The cipher and the authenticated plaintext of `sealed`, a store sealed
/// for `device_key` under `label`, with `secret`, which is
/// consumed and dropped before decryption. The refusals every sealed store
/// shares, in their order (LLR-bg538c, LLR-73sgz2): a secret of another
/// Device (`DeviceSecretMismatch`, before any derivation), fewer bytes than
/// a nonce and a tag (`TooShort`), a ciphertext that does not authenticate
/// (`NotAuthentic`).
pub(crate) fn open_sealed(
    sealed: &SealedStore,
    device_key: &DevicePublicKey,
    secret: DeviceSecret,
    label: &[u8],
) -> Result<(XChaCha20Poly1305, Zeroizing<Vec<u8>>), OrgNodeError> {
    if secret.device_key() != *device_key {
        return Err(OrgNodeError::DeviceSecretMismatch);
    }
    let bytes = sealed.as_bytes();
    if bytes.len() < NONCE_LENGTH + TAG_LENGTH {
        return Err(OrgNodeError::StoreRefused(StoreRefusal::TooShort));
    }
    let key = labelled_cipher(&secret, device_key, label)?;
    drop(secret);
    let (nonce, ciphertext) = bytes.split_at(NONCE_LENGTH);
    let plaintext = Zeroizing::new(
        key.decrypt(XNonce::from_slice(nonce), ciphertext)
            .map_err(|_| OrgNodeError::StoreRefused(StoreRefusal::NotAuthentic))?,
    );
    Ok((key, plaintext))
}

/// The `Debug` rendering of the store key `secret` derives (LLR-scgk5j).
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub fn store_key_debug_for_test(secret: &DeviceSecret) -> Result<String, OrgNodeError> {
    Ok(format!("{:?}", derive_key(secret, &secret.device_key(), STORE_KEY_INFO)?))
}

/// Whether `StoreKey` implements `Display` and whether it is `Copy`, in that
/// order (LLR-scgk5j).
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub fn store_key_traits_for_test() -> (bool, bool) {
    (crate::implements_display!(StoreKey), crate::implements_copy!(StoreKey))
}

/// `nonce(24) ‖ ciphertext` of `plaintext`, with a fresh random nonce.
pub(crate) fn seal_bytes<R: RngCore + CryptoRng>(
    key: &XChaCha20Poly1305,
    plaintext: &[u8],
    rng: &mut R,
) -> Result<SealedStore, OrgNodeError> {
    let mut nonce = [0u8; NONCE_LENGTH];
    rng.fill_bytes(&mut nonce);
    let ct = key
        .encrypt(XNonce::from_slice(&nonce), plaintext)
        .map_err(|e| OrgNodeError::Chain(format!("encrypt failed: {e}")))?;
    let mut out = Vec::with_capacity(NONCE_LENGTH + ct.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ct);
    Ok(SealedStore(out))
}

impl PersonaStore {
    /// Open the sealed store of the Persona whose DevicePublicKey is `device_key`
    /// with its secret, which is consumed (LLR-bg538c, LLR-7yu2xy). Refused,
    /// each with its own error and opening nothing: a secret of another
    /// Device (`DeviceSecretMismatch`, before any decryption); fewer bytes
    /// than a nonce and a tag (`TooShort`); a ciphertext that does not
    /// authenticate (`NotAuthentic`); a plaintext that does not decode
    /// (`Undecodable`) or holds a value its type's parse refuses
    /// (`InvalidField`); and a store whose Persona names another Device
    /// (`DeviceSecretMismatch`). The behind marker opens `None`.
    pub fn unseal(sealed: &SealedStore, device_key: DevicePublicKey, secret: DeviceSecret) -> Result<Self, OrgNodeError> {
        let (key, plaintext) = open_sealed(sealed, &device_key, secret, STORE_KEY_INFO)?;
        let content = match plaintext.first() {
            Some(0) => {
                let raw: RawStoreData = postcard::from_bytes(&plaintext[1..])
                    .map_err(|_| OrgNodeError::StoreRefused(StoreRefusal::Undecodable))?;
                let data = StoreData::try_from(raw)?;
                if data.persona.device_key != device_key {
                    return Err(OrgNodeError::DeviceSecretMismatch);
                }
                StoreContent::Active(data)
            }
            Some(1) => {
                let raw: crate::removal::RawRemovedStore = postcard::from_bytes(&plaintext[1..])
                    .map_err(|_| OrgNodeError::StoreRefused(StoreRefusal::Undecodable))?;
                StoreContent::Removed(crate::removal::RemovedStore::try_from(raw)?)
            }
            Some(_) => return Err(OrgNodeError::InvalidField { field: "store", reason: "unknown store form".into() }),
            None => return Err(OrgNodeError::StoreRefused(StoreRefusal::Undecodable)),
        };
        Ok(Self { key, device_key, content })
    }

    /// An open store over `data`, keyed from `secret`, whose DevicePublicKey must
    /// be `data`'s Persona's. The secret is borrowed: the operation that
    /// adds a store returns it to its caller.
    pub(crate) fn create(secret: &DeviceSecret, data: StoreData) -> Result<Self, OrgNodeError> {
        if secret.device_key() != data.persona.device_key {
            return Err(OrgNodeError::DeviceSecretMismatch);
        }
        let persona_device_key = data.persona.device_key;
        let key = cipher(secret, &persona_device_key)?;
        Ok(Self { key, device_key: persona_device_key, content: StoreContent::Active(data) })
    }

    /// The Device public key this store was opened for.
    pub fn device_key(&self) -> DevicePublicKey {
        self.device_key
    }

    /// The store's content, in either form (LLR-6zaeb6).
    pub fn content(&self) -> &StoreContent {
        &self.content
    }

    /// The Active store's data; a removed store refuses with
    /// `OrganisationRemoved` (LLR-myfk8w).
    pub fn data(&self) -> Result<&StoreData, OrgNodeError> {
        match &self.content {
            StoreContent::Active(data) => Ok(data),
            StoreContent::Removed(removed) => Err(OrgNodeError::OrganisationRemoved { org_id: removed.org_id }),
        }
    }

    /// The Active store's data, mutable; a removed store refuses with
    /// `OrganisationRemoved` (LLR-myfk8w).
    pub fn data_mut(&mut self) -> Result<&mut StoreData, OrgNodeError> {
        match &mut self.content {
            StoreContent::Active(data) => Ok(data),
            StoreContent::Removed(removed) => Err(OrgNodeError::OrganisationRemoved { org_id: removed.org_id }),
        }
    }

    /// Put `content` in place of the held content, after it was sealed
    /// (LLR-2d5a7m, LLR-85kves).
    pub(crate) fn replace_content(&mut self, content: StoreContent) {
        self.content = content;
    }

    /// This store, sealed with a fresh 24-byte nonce (LLR-ay8nvu).
    pub(crate) fn seal<R: RngCore + CryptoRng>(&self, rng: &mut R) -> Result<StoreWrite, OrgNodeError> {
        self.seal_content(&self.content, rng)
    }

    /// `content`, sealed in its form under this store's key (LLR-6zaeb6).
    pub(crate) fn seal_content<R: RngCore + CryptoRng>(
        &self,
        content: &StoreContent,
        rng: &mut R,
    ) -> Result<StoreWrite, OrgNodeError> {
        match content {
            StoreContent::Active(data) => self.seal_data(data, rng),
            StoreContent::Removed(removed) => {
                let mut plaintext = Zeroizing::new(vec![1u8]);
                plaintext.extend(
                    postcard::to_allocvec(removed).map_err(|error| OrgNodeError::Chain(format!("store encode: {error}")))?,
                );
                Ok(StoreWrite { device_key: self.device_key, sealed: seal_bytes(&self.key, &plaintext, rng)? })
            }
        }
    }

    /// `data`, sealed in its Active form under this store's key as this
    /// store's write: what a change would write, before it is adopted.
    pub(crate) fn seal_data<R: RngCore + CryptoRng>(&self, data: &StoreData, rng: &mut R) -> Result<StoreWrite, OrgNodeError> {
        let mut plaintext = Zeroizing::new(vec![0u8]);
        plaintext.extend(
            postcard::to_allocvec(&StoreDataEncoding::from(data))
                .map_err(|error| OrgNodeError::Chain(format!("store encode: {error}")))?,
        );
        Ok(StoreWrite { device_key: self.device_key, sealed: seal_bytes(&self.key, &plaintext, rng)? })
    }
}

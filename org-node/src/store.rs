//! Encrypted-at-rest persistence for personas and org records (spec §4.2–4.5).
//! PoC simplification S9: passphrase-derived key, not OS keychain.
use std::path::PathBuf;

use argon2::Argon2;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use org_members::{DevicePublicKey, Handle, MemberId, Name, PersonPublicKey, RootHash, Surname};

use crate::chain::OrgState;
use crate::ids::OrgId;
use crate::types::{
    ChainAccount, DeviceSeed, Epoch, MemberSeed, OrgPrivateKey, OrgPublicKey, PersonaId,
    SequenceNumber,
};
use crate::OrgNodeError;

/// Names the field whose parse refused a decoded value (LLR-8bum44).
/// postcard drops the message of an error raised inside `Deserialize`, so a
/// record with a fallible field is decoded as its `Raw…` mirror and parsed
/// here, field by field, instead.
pub(crate) fn parse_field<T, E: core::fmt::Display>(
    field: &'static str,
    parsed: Result<T, E>,
) -> Result<T, OrgNodeError> {
    parsed.map_err(|e| OrgNodeError::InvalidField { field, reason: e.to_string() })
}

/// A Persona's handle, name and surname, parsed (LLR-q6n25z). The one place
/// Persona details are parsed: where a Persona is created and where a stored
/// Persona record is loaded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonaDetails {
    pub handle: Handle,
    pub name: Name,
    pub surname: Surname,
}

impl PersonaDetails {
    /// Parses each detail in turn; the first refused is reported as
    /// `InvalidField` naming it (`persona.handle`, `persona.name`,
    /// `persona.surname`) (LLR-q6n25z).
    pub fn parse(handle: &str, name: &str, surname: &str) -> Result<Self, OrgNodeError> {
        Ok(Self {
            handle: parse_field("persona.handle", Handle::parse(handle))?,
            name: parse_field("persona.name", Name::parse(name))?,
            surname: parse_field("persona.surname", Surname::parse(surname))?,
        })
    }
}

/// A locally-held identity, one per org (spec §4.2). Seeds held in their secret
/// types; `Debug` redacts them (PR-hqwpg9).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "RawPersonaRecord")]
pub struct PersonaRecord {
    pub persona_id: PersonaId,
    pub org_id: Option<OrgId>,
    pub handle: Handle,
    pub name: Name,
    pub surname: Surname,
    pub member_seed: MemberSeed,
    pub device_seed: DeviceSeed,
    pub member_id: Option<MemberId>,
    pub status: PersonaStatus,
}

/// A `PersonaRecord` as decoded, before parsing (LLR-8bum44).
#[derive(Deserialize)]
pub(crate) struct RawPersonaRecord {
    persona_id: PersonaId,
    org_id: Option<OrgId>,
    handle: String,
    name: String,
    surname: String,
    member_seed: MemberSeed,
    device_seed: DeviceSeed,
    member_id: Option<MemberId>,
    status: PersonaStatus,
}

impl TryFrom<RawPersonaRecord> for PersonaRecord {
    type Error = OrgNodeError;

    fn try_from(raw: RawPersonaRecord) -> Result<Self, Self::Error> {
        let PersonaDetails { handle, name, surname } =
            PersonaDetails::parse(&raw.handle, &raw.name, &raw.surname)?;
        Ok(Self {
            persona_id: raw.persona_id,
            org_id: raw.org_id,
            handle,
            name,
            surname,
            member_seed: raw.member_seed,
            device_seed: raw.device_seed,
            member_id: raw.member_id,
            status: raw.status,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersonaStatus {
    Proposed,
    Active,
    Revoked,
}

/// A member's local view of an org (spec §4.3).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "RawOrgRecord")]
pub struct OrgRecord {
    pub org_id: OrgId,
    pub root_hash: RootHash,
    pub org_pub_key: OrgPublicKey,
    pub epoch: Epoch,
    pub last_seq: SequenceNumber,
    pub trie_members: Vec<MemberSnapshot>,
    /// The proxy account the app handed `commit_genesis`, held opaquely and
    /// only handed back (LLR-dzte8x); `None` on a record created by a first
    /// admission.
    pub proxy_account: Option<ChainAccount>,
    /// The Organisation's X25519 private key, required (LLR-byjvd9,
    /// REQ-ech45n, REQ-szq3ud): the creating node's first from its genesis
    /// update, every other node's from the Wire message that admitted it
    /// (LLR-3fwykc, LLR-ckk5nz); each later commit replaces it with the key
    /// of the epoch it reaches, and no earlier key is kept (LLR-6s785x,
    /// LLR-4kh9w9). It renders under `Debug` as its redacted secret type
    /// (LLR-2dvhz8).
    pub org_private_key: OrgPrivateKey,
    /// The Change set bytes of the Envelope the last commit verified, `None`
    /// on a record `commit_genesis` creates; a first admission keeps the
    /// admitting Envelope's Change set (LLR-d9778a). Sent to nobody.
    pub kept_change_set: Option<Vec<u8>>,
}

/// An `OrgRecord` as decoded, before parsing (LLR-8bum44).
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
            trie_members: raw
                .trie_members
                .into_iter()
                .map(MemberSnapshot::try_from)
                .collect::<Result<_, _>>()?,
            proxy_account: raw.proxy_account,
            org_private_key: raw.org_private_key,
            kept_change_set: raw.kept_change_set,
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

/// A `MemberSnapshot` as decoded, before parsing (LLR-8bum44).
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
                .iter()
                .map(|k| parse_field("member.device_keys", DevicePublicKey::parse(k)))
                .collect::<Result<_, _>>()?,
        })
    }
}

/// The bound on one Organisation's provisional updates in their stored form:
/// the wire frame's 1 MiB (LLR-jq7qh7). A separate constant from
/// `transport::MAX_FRAME`, so changing one is a decision about the other.
pub const MAX_PROVISIONAL_BYTES: usize = 1 << 20;

/// An update this node built and keeps until the chain agrees with it
/// (REQ-xs4ab8, LLR-95753m).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawProvisionalUpdate")]
pub struct ProvisionalUpdate {
    /// `None` for genesis: there is no identifier until the chain write.
    pub org_id: Option<OrgId>,
    /// The Persona that built it. For a genesis update, which has no
    /// `org_id`, this is what groups it: its identity is `(persona_id,
    /// resulting_root, org_pub_key)` and its bound is measured over the
    /// genesis updates of the same Persona (LLR-95753m, LLR-jq7qh7).
    pub persona_id: PersonaId,
    /// The record's root it applies to; `None` for genesis.
    pub base_root: Option<RootHash>,
    /// The Membership root it produces.
    pub resulting_root: RootHash,
    /// The epoch it produces on the chain: the record's epoch plus one, or 1
    /// for genesis (REQ-txvtm9, Decision 16).
    pub seq: SequenceNumber,
    /// The Organisation public key the app publishes with the root.
    pub org_pub_key: OrgPublicKey,
    pub change: ProvisionalChange,
}

impl ProvisionalUpdate {
    /// The group this update belongs to: its Organisation's, or for a
    /// genesis update the Persona's that built it.
    pub(crate) fn target(&self) -> ProvisionalTarget {
        match self.org_id {
            Some(org_id) => ProvisionalTarget::Org(org_id),
            None => ProvisionalTarget::Genesis(self.persona_id.clone()),
        }
    }
}

/// A group of provisional updates: an Organisation's, or the genesis updates
/// one Persona built, which have no Organisation yet. What
/// `OrgService::discard_provisional` names (LLR-7cmp38) and what the bound is
/// measured over (LLR-jq7qh7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProvisionalTarget {
    Org(OrgId),
    Genesis(PersonaId),
}

impl ProvisionalTarget {
    /// Whether `update` is one of the provisional updates this target names.
    pub(crate) fn names(&self, update: &ProvisionalUpdate) -> bool {
        match self {
            Self::Org(org_id) => update.org_id == Some(*org_id),
            Self::Genesis(persona_id) => update.org_id.is_none() && &update.persona_id == persona_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProvisionalChange {
    /// The founding Member, and the Organisation private key whose public key
    /// is the update's, kept until `commit_genesis` (LLR-qjz3q4).
    Genesis { members: Vec<MemberSnapshot>, org_private_key: OrgPrivateKey },
    /// postcard of the Change set of an admission or a revocation, and the
    /// private key of the fresh Organisation key pair drawn for it once its
    /// root was calculated (LLR-e2b7gv, LLR-qjz3q4), kept until
    /// `commit_update` takes it into the record (LLR-6s785x).
    ChangeSet { change_set: Vec<u8>, org_private_key: OrgPrivateKey },
}

/// A `ProvisionalUpdate` as decoded, before parsing (LLR-8bum44).
#[derive(Deserialize)]
pub(crate) struct RawProvisionalUpdate {
    org_id: Option<OrgId>,
    persona_id: PersonaId,
    base_root: Option<RootHash>,
    resulting_root: RootHash,
    seq: SequenceNumber,
    org_pub_key: [u8; 32],
    change: RawProvisionalChange,
}

#[derive(Deserialize)]
pub(crate) enum RawProvisionalChange {
    Genesis { members: Vec<RawMemberSnapshot>, org_private_key: OrgPrivateKey },
    ChangeSet { change_set: Vec<u8>, org_private_key: OrgPrivateKey },
}

impl TryFrom<RawProvisionalUpdate> for ProvisionalUpdate {
    type Error = OrgNodeError;

    fn try_from(raw: RawProvisionalUpdate) -> Result<Self, Self::Error> {
        Ok(Self {
            org_id: raw.org_id,
            persona_id: raw.persona_id,
            base_root: raw.base_root,
            resulting_root: raw.resulting_root,
            seq: raw.seq,
            org_pub_key: parse_field("provisional.org_pub_key", OrgPublicKey::parse(&raw.org_pub_key))?,
            change: match raw.change {
                RawProvisionalChange::Genesis { members, org_private_key } => ProvisionalChange::Genesis {
                    members: members.into_iter().map(MemberSnapshot::try_from).collect::<Result<_, _>>()?,
                    org_private_key,
                },
                RawProvisionalChange::ChangeSet { change_set, org_private_key } => ProvisionalChange::ChangeSet { change_set, org_private_key },
            },
        })
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StoreData {
    pub personas: Vec<PersonaRecord>,
    pub orgs: Vec<OrgRecord>,
    /// Updates this node built and keeps until the chain agrees with them
    /// (LLR-95753m); kept through `insert_provisional`.
    pub provisional_updates: Vec<ProvisionalUpdate>,
    /// Expectations the app declared, at most once each (LLR-9zfnmb).
    pub expected_admissions: Vec<ExpectedAdmission>,
}

impl StoreData {
    /// Keep `update`: replace the stored update with its identity, else
    /// append; refuse — changing nothing — when its Organisation's group
    /// would exceed `MAX_PROVISIONAL_BYTES` in postcard form (LLR-95753m,
    /// LLR-jq7qh7). Identity: `(org_id, resulting_root, org_pub_key)`, or for
    /// genesis `(persona_id, resulting_root, org_pub_key)`: two updates for one
    /// change with different fresh keys are two updates.
    pub fn insert_provisional(&mut self, update: ProvisionalUpdate) -> Result<(), OrgNodeError> {
        let target = update.target();
        let same_identity = |u: &ProvisionalUpdate| {
            target.names(u) && u.resulting_root == update.resulting_root && u.org_pub_key == update.org_pub_key
        };
        // `Vec<&ProvisionalUpdate>` encodes exactly as `Vec<ProvisionalUpdate>`.
        let mut group: Vec<&ProvisionalUpdate> =
            self.provisional_updates.iter().filter(|u| target.names(u) && !same_identity(u)).collect();
        group.push(&update);
        let len = postcard::to_allocvec(&group)
            .map_err(|e| OrgNodeError::Chain(format!("provisional encode: {e}")))?
            .len();
        if len > MAX_PROVISIONAL_BYTES {
            return Err(OrgNodeError::ProvisionalLimit { limit: MAX_PROVISIONAL_BYTES });
        }
        match self.provisional_updates.iter().position(same_identity) {
            Some(i) => self.provisional_updates[i] = update,
            None => self.provisional_updates.push(update),
        }
        Ok(())
    }

    /// The stored provisional update for `org_id` whose resulting root and
    /// Organisation public key `chain` holds: the update the chain carries,
    /// if this node built it (LLR-cmdrp9, LLR-gr8x3r).
    pub(crate) fn held_update_for(&self, org_id: OrgId, chain: &OrgState) -> Option<&ProvisionalUpdate> {
        self.provisional_updates.iter().find(|update| {
            update.org_id == Some(org_id)
                && update.resulting_root == chain.root_hash
                && update.org_pub_key == chain.org_pub_key
        })
    }

    /// The store without anything of `org_id` (LLR-pba7yu): its record, its
    /// provisional updates, its expectations, its Personas and the genesis
    /// updates they built. Adds nothing in their place; everything else keeps
    /// its order.
    pub fn forget_organisation(&self, org_id: OrgId) -> StoreData {
        let forgotten_personas: Vec<&PersonaId> = self
            .personas
            .iter()
            .filter(|persona| persona.org_id == Some(org_id))
            .map(|persona| &persona.persona_id)
            .collect();
        let keeps_update = |update: &&ProvisionalUpdate| match update.org_id {
            Some(update_org_id) => update_org_id != org_id,
            None => !forgotten_personas.contains(&&update.persona_id),
        };
        StoreData {
            personas: self.personas.iter().filter(|persona| persona.org_id != Some(org_id)).cloned().collect(),
            orgs: self.orgs.iter().filter(|record| record.org_id != org_id).cloned().collect(),
            provisional_updates: self.provisional_updates.iter().filter(keeps_update).cloned().collect(),
            expected_admissions: self
                .expected_admissions
                .iter()
                .filter(|expectation| expectation.org_id != org_id)
                .copied()
                .collect(),
        }
    }
}

/// A first admission the app declared it expects: the Organisation alone
/// (REQ-8amu2a, LLR-95753m).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpectedAdmission {
    pub org_id: OrgId,
}

/// The store plaintext as decoded, before parsing (LLR-8bum44).
#[derive(Deserialize)]
pub(crate) struct RawStoreData {
    personas: Vec<RawPersonaRecord>,
    orgs: Vec<RawOrgRecord>,
    provisional_updates: Vec<RawProvisionalUpdate>,
    expected_admissions: Vec<ExpectedAdmission>,
}

impl TryFrom<RawStoreData> for StoreData {
    type Error = OrgNodeError;

    fn try_from(raw: RawStoreData) -> Result<Self, Self::Error> {
        Ok(Self {
            personas: raw.personas.into_iter().map(PersonaRecord::try_from).collect::<Result<_, _>>()?,
            orgs: raw.orgs.into_iter().map(OrgRecord::try_from).collect::<Result<_, _>>()?,
            provisional_updates: raw
                .provisional_updates
                .into_iter()
                .map(ProvisionalUpdate::try_from)
                .collect::<Result<_, _>>()?,
            expected_admissions: raw.expected_admissions,
        })
    }
}

/// Encrypted file store. On-disk layout: `nonce(24) ‖ ciphertext`.
pub struct PersonaStore {
    path: PathBuf,
    key: XChaCha20Poly1305,
    data: StoreData,
}

/// The store encryption key (LLR-scgk5j): private to this file, no
/// `Display`, not `Copy`, redacted `Debug`.
struct StoreKey([u8; 32]);

impl StoreKey {
    fn expose_secret(&self) -> &[u8; 32] {
        &self.0
    }
}

impl core::fmt::Debug for StoreKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("StoreKey([REDACTED])")
    }
}

fn derive_key(passphrase: &str, salt: &[u8]) -> Result<StoreKey, OrgNodeError> {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|e| OrgNodeError::Chain(format!("kdf failed: {e}")))?;
    Ok(StoreKey(key))
}

/// The `Debug` rendering of the key `passphrase` derives (LLR-scgk5j).
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub fn store_key_debug_for_test(passphrase: &str) -> Result<String, OrgNodeError> {
    Ok(format!("{:?}", derive_key(passphrase, APP_SALT)?))
}

/// Whether `StoreKey` implements `Display` and whether it is `Copy`, in that
/// order (LLR-scgk5j).
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub fn store_key_traits_for_test() -> (bool, bool) {
    (crate::implements_display!(StoreKey), crate::implements_copy!(StoreKey))
}

/// 32-byte fixed app salt — PoC simplification S9.
const APP_SALT: &[u8] = b"ods-phase2-personastore-v1______";

fn cipher(passphrase: &str) -> Result<XChaCha20Poly1305, OrgNodeError> {
    XChaCha20Poly1305::new_from_slice(derive_key(passphrase, APP_SALT)?.expose_secret())
        .map_err(|e| OrgNodeError::Chain(format!("bad key length: {e}")))
}

/// `nonce(24) ‖ ciphertext` of `plaintext`, with a fresh random nonce.
fn seal<R: RngCore + CryptoRng>(
    key: &XChaCha20Poly1305,
    plaintext: &[u8],
    rng: &mut R,
) -> Result<Vec<u8>, OrgNodeError> {
    let mut nonce = [0u8; 24];
    rng.fill_bytes(&mut nonce);
    let ct = key
        .encrypt(XNonce::from_slice(&nonce), plaintext)
        .map_err(|e| OrgNodeError::Chain(format!("encrypt failed: {e}")))?;
    let mut out = Vec::with_capacity(24 + ct.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ct);
    Ok(out)
}

/// Writes `plaintext` to `path` encrypted exactly as `save` writes a store,
/// so a test can present content this software never writes (LLR-8bum44).
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub fn seal_for_test<R: RngCore + CryptoRng>(
    path: &std::path::Path,
    passphrase: &str,
    plaintext: &[u8],
    rng: &mut R,
) -> Result<(), OrgNodeError> {
    let sealed = seal(&cipher(passphrase)?, plaintext, rng)?;
    std::fs::write(path, sealed).map_err(|e| OrgNodeError::Chain(e.to_string()))
}

impl PersonaStore {
    /// Open or create a store at `path` using `passphrase`.
    /// PoC uses a fixed application salt constant (simplification S9).
    /// A store holding a value its type's parse refuses is refused whole,
    /// naming the field (LLR-8bum44).
    pub fn open(path: PathBuf, passphrase: &str) -> Result<Self, OrgNodeError> {
        let key = cipher(passphrase)?;
        let data = if path.exists() {
            let blob =
                std::fs::read(&path).map_err(|e| OrgNodeError::Chain(e.to_string()))?;
            if blob.len() < 24 {
                return Err(OrgNodeError::Chain("store file too short".into()));
            }
            let (nonce_bytes, ct) = blob.split_at(24);
            let pt = key
                .decrypt(XNonce::from_slice(nonce_bytes), ct)
                .map_err(|_| {
                    OrgNodeError::Chain("decrypt failed (wrong passphrase?)".into())
                })?;
            let raw: RawStoreData = postcard::from_bytes(&pt)
                .map_err(|e| OrgNodeError::Chain(format!("store decode: {e}")))?;
            StoreData::try_from(raw)?
        } else {
            StoreData::default()
        };
        Ok(Self { path, key, data })
    }

    pub fn data(&self) -> &StoreData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut StoreData {
        &mut self.data
    }

    /// Encrypt and write the store. Generates a fresh 24-byte random nonce per save.
    pub fn save<R: RngCore + CryptoRng>(&self, rng: &mut R) -> Result<(), OrgNodeError> {
        let pt = postcard::to_allocvec(&self.data)
            .map_err(|e| OrgNodeError::Chain(format!("store encode: {e}")))?;
        let out = seal(&self.key, &pt, rng)?;
        std::fs::write(&self.path, out).map_err(|e| OrgNodeError::Chain(e.to_string()))
    }
}

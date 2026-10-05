//! Encrypted-at-rest persistence for personas and org records (spec §4.2–4.5).
//! PoC simplification S9: passphrase-derived key, not OS keychain.
use std::path::PathBuf;

use argon2::Argon2;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use org_members::{Handle, MemberId, Name, P2pDeviceKey, P2pMemberKey, RootHash, Surname};

use crate::ids::OrgId;
use crate::types::{
    ChainAccount, DeviceSeed, Epoch, MemberSeed, OrgPublicKey, OrgSecret, PersonaId, SequenceNumber,
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
    pub org_secret: Option<OrgSecret>,
    pub last_seq: SequenceNumber,
    pub admin_member_key: P2pMemberKey,
    pub trie_members: Vec<MemberSnapshot>,
    /// Pure-proxy AccountId32 `P` persisted at genesis so `submit_update` can
    /// build the `proxied(P, ...)` call after a restart.  `None` on member-side
    /// records (only the admin that called `create_organisation` stores P).
    pub proxy_account: Option<ChainAccount>,
}

/// An `OrgRecord` as decoded, before parsing (LLR-8bum44).
#[derive(Deserialize)]
pub(crate) struct RawOrgRecord {
    org_id: OrgId,
    root_hash: RootHash,
    org_pub_key: [u8; 32],
    epoch: Epoch,
    org_secret: Option<OrgSecret>,
    last_seq: SequenceNumber,
    admin_member_key: [u8; 32],
    trie_members: Vec<RawMemberSnapshot>,
    /// Defaults to `None` so stores written before this field was added still
    /// decode correctly.
    #[serde(default)]
    proxy_account: Option<ChainAccount>,
}

impl TryFrom<RawOrgRecord> for OrgRecord {
    type Error = OrgNodeError;

    fn try_from(raw: RawOrgRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            org_id: raw.org_id,
            root_hash: raw.root_hash,
            org_pub_key: parse_field("org.org_pub_key", OrgPublicKey::parse(&raw.org_pub_key))?,
            epoch: raw.epoch,
            org_secret: raw.org_secret,
            last_seq: raw.last_seq,
            admin_member_key: parse_field(
                "org.admin_member_key",
                P2pMemberKey::parse(&raw.admin_member_key),
            )?,
            trie_members: raw
                .trie_members
                .into_iter()
                .map(MemberSnapshot::try_from)
                .collect::<Result<_, _>>()?,
            proxy_account: raw.proxy_account,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "RawMemberSnapshot")]
pub struct MemberSnapshot {
    pub id: MemberId,
    pub handle: Handle,
    pub name: Name,
    pub surname: Surname,
    pub member_key: P2pMemberKey,
    pub device_keys: Vec<P2pDeviceKey>,
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
            member_key: parse_field("member.member_key", P2pMemberKey::parse(&raw.member_key))?,
            device_keys: raw
                .device_keys
                .iter()
                .map(|k| parse_field("member.device_keys", P2pDeviceKey::parse(k)))
                .collect::<Result<_, _>>()?,
        })
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StoreData {
    pub personas: Vec<PersonaRecord>,
    pub orgs: Vec<OrgRecord>,
    /// Invites imported by B before first admission.  Keyed by org_id so
    /// `receive_and_verify` can cross-check the sender's device key against
    /// the admin_device_key the invite asserts.  Cleared after first commit.
    pub pending_invites: Vec<PendingInvite>,
}

/// The store plaintext as decoded, before parsing (LLR-8bum44).
#[derive(Deserialize)]
pub(crate) struct RawStoreData {
    personas: Vec<RawPersonaRecord>,
    orgs: Vec<RawOrgRecord>,
    pending_invites: Vec<RawPendingInvite>,
}

impl TryFrom<RawStoreData> for StoreData {
    type Error = OrgNodeError;

    fn try_from(raw: RawStoreData) -> Result<Self, Self::Error> {
        Ok(Self {
            personas: raw.personas.into_iter().map(PersonaRecord::try_from).collect::<Result<_, _>>()?,
            orgs: raw.orgs.into_iter().map(OrgRecord::try_from).collect::<Result<_, _>>()?,
            pending_invites: raw
                .pending_invites
                .into_iter()
                .map(PendingInvite::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}

/// Minimal fields from an `Invite` that must survive store round-trips.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "RawPendingInvite")]
pub struct PendingInvite {
    pub org_id: OrgId,
    pub admin_device_key: P2pDeviceKey,
    pub admin_member_key: P2pMemberKey,
    pub org_pub_key: OrgPublicKey,
}

/// A `PendingInvite` as decoded, before parsing (LLR-8bum44).
#[derive(Deserialize)]
pub(crate) struct RawPendingInvite {
    org_id: OrgId,
    admin_device_key: [u8; 32],
    admin_member_key: [u8; 32],
    org_pub_key: [u8; 32],
}

impl TryFrom<RawPendingInvite> for PendingInvite {
    type Error = OrgNodeError;

    fn try_from(raw: RawPendingInvite) -> Result<Self, Self::Error> {
        Ok(Self {
            org_id: raw.org_id,
            admin_device_key: parse_field(
                "pending_invite.admin_device_key",
                P2pDeviceKey::parse(&raw.admin_device_key),
            )?,
            admin_member_key: parse_field(
                "pending_invite.admin_member_key",
                P2pMemberKey::parse(&raw.admin_member_key),
            )?,
            org_pub_key: parse_field(
                "pending_invite.org_pub_key",
                OrgPublicKey::parse(&raw.org_pub_key),
            )?,
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

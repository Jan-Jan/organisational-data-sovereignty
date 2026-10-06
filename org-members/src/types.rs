use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use unicode_security::GeneralSecurityProfile;
use unicode_security::MixedScript;

use crate::error::OrgMembersError;
use crate::normalize::to_nfc;

pub use person::{
    DevicePublicKey, DeviceSlots, Name, NodeHash, PersonPublicKey, Surname, MAX_DEVICES,
    MAX_NAME_LEN, MAX_SURNAME_LEN,
};

/// Maximum byte length of a handle (after NFC normalization). Caps memory
/// exposure from adversarial wire-format inputs. Email local-parts are
/// limited to 64 octets by RFC 5321; 128 leaves headroom for legitimate
/// non-ASCII handles after NFC expansion.
pub const MAX_HANDLE_LEN: usize = 128;

/// Immutable member identifier. Used as the SMT key and as a stable reference
/// to a member regardless of changes to their handle or p2p key.
///
/// The 32 bytes are opaque -- the caller is responsible for generating unique,
/// immutable values (e.g., random bytes, or a hash of a stable input).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MemberId(pub(crate) [u8; 32]);

impl MemberId {
    /// Wraps 32 opaque bytes as a member id.
    ///
    /// The library does NOT validate these bytes -- the caller is responsible
    /// for ensuring ids are unique within the organisation and effectively
    /// random (so the SMT tree stays well-distributed across the 256-bit
    /// keyspace). Suggested generators: cryptographic random bytes, or a hash
    /// of a stable per-member input (e.g. an enrollment artifact).
    ///
    /// **Uniqueness is the caller's responsibility.** Two members independently
    /// constructed with the same byte pattern (including `[0u8; 32]`) will
    /// collide -- `add_member` will reject the second one with `DuplicateId`,
    /// but the situation should be avoided. Don't hard-code id values.
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Returns the bit at the given index (0 = MSB of byte 0, 255 = LSB of byte 31).
    /// Used for SMT path traversal.
    pub fn bit(&self, index: u16) -> bool {
        let byte_idx = (index / 8) as usize;
        let bit_idx = 7 - (index % 8);
        (self.0[byte_idx] >> bit_idx) & 1 == 1
    }
}

impl From<[u8; 32]> for MemberId {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl fmt::Debug for MemberId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MemberId({:02x}{:02x}{:02x}{:02x}..)", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

/// The 32 encoded bytes of a key held in the organisation -- member key or
/// device key alike: the same bytes are the same key (LLR-v6gfc7). Tag type.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct HeldKey([u8; 32]);

impl From<&PersonPublicKey> for HeldKey {
    fn from(k: &PersonPublicKey) -> Self {
        Self(*k.as_bytes())
    }
}

impl From<&DevicePublicKey> for HeldKey {
    fn from(k: &DevicePublicKey) -> Self {
        Self(*k.as_bytes())
    }
}

/// The UTS#39 skeleton of a handle: handles rendering alike share one
/// (LLR-5w2jx8). Tag type, built only from a `Handle`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct HandleSkeleton(String);

impl HandleSkeleton {
    pub(crate) fn of(handle: &Handle) -> Self {
        use unicode_security::confusable_detection::skeleton;
        Self(skeleton(handle.as_str()).collect())
    }
}

/// A validated member handle (REQ-h5ret5): NFC, non-empty, at most
/// `MAX_HANDLE_LEN` bytes, lowercase, no `.`, UTS#39 identifier characters or
/// `-`, single-script. `parse` is the only way in; PII, so `Debug` redacts.
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
pub struct Handle(String);

impl Handle {
    /// Normalizes to NFC and enforces the handle rules. LLR-xzqs9r.
    pub fn parse(value: &str) -> Result<Self, OrgMembersError> {
        if value.is_empty() {
            return Err(OrgMembersError::InvalidHandle(
                "handle must not be empty".to_string(),
            ));
        }
        let normalized = to_nfc(value);
        if normalized.len() > MAX_HANDLE_LEN {
            return Err(OrgMembersError::InvalidHandle(format!(
                "handle exceeds {} bytes after NFC normalization",
                MAX_HANDLE_LEN
            )));
        }
        for ch in normalized.chars() {
            if ch.is_uppercase() {
                return Err(OrgMembersError::InvalidHandle(
                    "handle must be lowercase".to_string(),
                ));
            }
            if ch == '.' {
                return Err(OrgMembersError::InvalidHandle(
                    "handle must not contain '.'".to_string(),
                ));
            }
            if ch == '-' {
                continue;
            }
            if !ch.identifier_allowed() {
                return Err(OrgMembersError::InvalidHandle(format!(
                    "character {:?} not allowed by UTS#39",
                    ch
                )));
            }
        }
        if !normalized.is_single_script() {
            return Err(OrgMembersError::InvalidHandle(
                "handle must not mix scripts".to_string(),
            ));
        }
        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for Handle {
    type Error = OrgMembersError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl TryFrom<String> for Handle {
    type Error = OrgMembersError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

/// serde's `into`.
impl From<Handle> for String {
    fn from(value: Handle) -> Self {
        value.0
    }
}

impl fmt::Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Redacted: a handle is PII.
impl fmt::Debug for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Handle([REDACTED])")
    }
}

/// The externally-meaningful root of an org trie. Wraps the same bytes as
/// `NodeHash` but is type-distinct: only the trie root is a `RootHash`,
/// intermediate node hashes are `NodeHash`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RootHash(pub(crate) [u8; 32]);

impl RootHash {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for RootHash {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl From<NodeHash> for RootHash {
    fn from(h: NodeHash) -> Self {
        Self(*h.as_bytes())
    }
}

impl fmt::Debug for RootHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RootHash({:02x}{:02x}{:02x}{:02x}..)",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

/// A single member leaf in the trie. All PII fields are redacted in Debug.
///
/// Serde is derived: the field order is the wire order, and each field's own
/// `Deserialize` validates it, so a wire payload cannot bypass the handle,
/// name or surname rules.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MemberLeaf {
    /// Immutable member id. Used as the SMT key.
    id: MemberId,
    /// Validated, NFC-normalized handle (PII). Can change rarely.
    handle: Handle,
    /// The member's peer-to-peer key -- the "member-as-a-group" key used by
    /// the local-first software to grant access at the member level. Can
    /// change over time. Future versions may also add an on-chain key.
    p2p_key: PersonPublicKey,
    name: Name,
    surname: Surname,
    p2p_devices: DeviceSlots,
}

impl MemberLeaf {
    /// Constructs a new member leaf.
    ///
    /// Handle, name and surname are valid by construction (their `parse`).
    /// Requires ≥1 device -- new members must have at least one device. (An
    /// existing member can be reduced to zero devices via
    /// `emergency_isolate_member` on the trie.)
    pub fn new(
        id: MemberId,
        handle: Handle,
        p2p_key: PersonPublicKey,
        name: Name,
        surname: Surname,
        p2p_devices: Vec<DevicePublicKey>,
    ) -> Result<Self, OrgMembersError> {
        if p2p_devices.is_empty() {
            return Err(OrgMembersError::EmptyDeviceList);
        }
        let p2p_devices = DeviceSlots::parse(p2p_devices)?;
        Ok(Self { id, handle, p2p_key, name, surname, p2p_devices })
    }

    // === Crate-private field modifiers ===
    //
    // These produce modified copies of `self`. They bypass MemberLeaf::new's
    // invariants and are intended for use by the trie's domain operations
    // (update_name_surname, update_handle, rotate_p2p_key, add/delete p2p_device,
    // emergency_isolate_member). They do not check trie-level rules
    // (uniqueness, confusables, held keys); the newtype arguments carry field
    // validity.

    pub(crate) fn with_name_surname(mut self, name: Name, surname: Surname) -> Self {
        self.name = name;
        self.surname = surname;
        self
    }

    pub(crate) fn with_handle(mut self, handle: Handle) -> Self {
        self.handle = handle;
        self
    }

    pub(crate) fn with_p2p_key(mut self, key: PersonPublicKey) -> Self {
        self.p2p_key = key;
        self
    }

    pub(crate) fn with_p2p_device_slots(mut self, slots: DeviceSlots) -> Self {
        self.p2p_devices = slots;
        self
    }

    pub fn id(&self) -> &MemberId {
        &self.id
    }

    pub fn p2p_key(&self) -> &PersonPublicKey {
        &self.p2p_key
    }

    pub fn handle(&self) -> &Handle {
        &self.handle
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn surname(&self) -> &Surname {
        &self.surname
    }

    pub fn p2p_devices(&self) -> &[DevicePublicKey] {
        self.p2p_devices.devices()
    }

    pub fn has_p2p_device(&self, device: &DevicePublicKey) -> bool {
        self.p2p_devices.has_device(device)
    }

    pub fn p2p_device_count(&self) -> usize {
        self.p2p_devices.device_count()
    }

    pub(crate) fn p2p_device_slots(&self) -> &DeviceSlots {
        &self.p2p_devices
    }

    /// Canonical byte encoding for hashing. Crate-private because external
    /// callers cannot meaningfully compute `p2p_device_sub_trie_root` without
    /// invoking internal device-trie hashing -- exposing this would invite
    /// callers to produce bytes that don't match what the trie actually hashes.
    pub(crate) fn canonical_bytes(&self, p2p_device_sub_trie_root: &NodeHash) -> Vec<u8> {
        let mut buf = Vec::new();
        // id: 32 bytes raw
        buf.extend_from_slice(self.id.as_bytes());
        // handle_len + handle bytes
        let handle_bytes = self.handle.as_str().as_bytes();
        buf.extend_from_slice(&(handle_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(handle_bytes);
        // p2p_key: 32 bytes raw
        buf.extend_from_slice(self.p2p_key.as_bytes());
        // name_len + name bytes
        let name_bytes = self.name.as_str().as_bytes();
        buf.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(name_bytes);
        // surname_len + surname bytes
        let surname_bytes = self.surname.as_str().as_bytes();
        buf.extend_from_slice(&(surname_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(surname_bytes);
        // p2p_device_sub_trie_root: 32 bytes raw
        buf.extend_from_slice(p2p_device_sub_trie_root.as_bytes());
        buf
    }
}

impl fmt::Debug for MemberLeaf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemberLeaf")
            .field("id", &self.id)
            .field("handle", &"[REDACTED]")
            .field("p2p_key", &self.p2p_key)
            .field("name", &"[REDACTED]")
            .field("surname", &"[REDACTED]")
            .field("p2p_devices", &self.p2p_devices)
            .finish()
    }
}

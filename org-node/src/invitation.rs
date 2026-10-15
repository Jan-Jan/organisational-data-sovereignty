//! The Invite and the Invite reply (SDD-xshv5b): the Invite a Member sends and
//! the Invite reply the invitee returns, as Blobs (Base64 of postcard),
//! parsed here, naming the field that fails (LLR-b7wgpf, LLR-8bum44).
//! org-io runs the exchange, and the service
//! keeps the outstanding pairs in the sealed store (`OrgService::build_invite`
//! and the operations beside it). Nothing here is verified: an Invite's
//! sender and its Organisation name are whatever the sender chose.

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};

use org_members::{DevicePublicKey, Handle, Name, PersonPublicKey, Surname};

use crate::ids::OrgId;
use crate::store::parse_field;
use crate::OrgNodeError;

/// The identifier an Invite carries and its reply echoes: 32 bytes the
/// inviting node drew at random. Not secret. It binds a reply to the Invite
/// this node issued and travels in no Wire message (LLR-ms8njy).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InviteId([u8; 32]);

impl InviteId {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// An Invite as parsed: what the inviter typed, the Organisation it names,
/// the inviter's Devices, and the identifier its reply must echo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invite {
    pub org_name: String,
    pub org_id: OrgId,
    pub invitee_name: String,
    pub inviter_device_keys: Vec<DevicePublicKey>,
    pub invite_id: InviteId,
}

/// An Invite reply as parsed: the Invite it answers and the Persona the
/// invitee chose to join with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InviteReply {
    pub org_id: OrgId,
    pub invite_id: InviteId,
    pub member_key: PersonPublicKey,
    pub device_key: DevicePublicKey,
    pub handle: Handle,
    pub name: Name,
    pub surname: Surname,
}

#[derive(Serialize, Deserialize)]
struct WireInvite {
    org_name: String,
    org_id: Vec<u8>,
    invitee_name: String,
    inviter_device_keys: Vec<Vec<u8>>,
    invite_id: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct WireReply {
    org_id: Vec<u8>,
    invite_id: Vec<u8>,
    member_key: Vec<u8>,
    device_key: Vec<u8>,
    handle: String,
    name: String,
    surname: String,
}

/// Exactly `N` bytes, else `InvalidField` naming `field`.
fn exact<const N: usize>(field: &'static str, bytes: &[u8]) -> Result<[u8; N], OrgNodeError> {
    bytes.try_into().map_err(|_| OrgNodeError::InvalidField {
        field,
        reason: format!("expected {N} bytes, got {}", bytes.len()),
    })
}

/// The value inside the Blob `blob`, else `InvalidField` naming `what`
/// (`invite` or `reply`).
fn unarmour<T: for<'de> Deserialize<'de>>(what: &'static str, blob: &str) -> Result<T, OrgNodeError> {
    let bytes = STANDARD
        .decode(blob.trim())
        .map_err(|error| OrgNodeError::InvalidField { field: what, reason: format!("not a Blob: {error}") })?;
    postcard::from_bytes(&bytes)
        .map_err(|error| OrgNodeError::InvalidField { field: what, reason: format!("does not decode: {error}") })
}

fn armour<T: Serialize>(value: &T) -> Result<String, OrgNodeError> {
    postcard::to_allocvec(value)
        .map(|bytes| STANDARD.encode(bytes))
        .map_err(|error| OrgNodeError::Chain(format!("blob encode: {error}")))
}

/// `bytes` armoured as a Blob, whatever they hold, so a fuzz target reaches
/// the decode behind the armour (test builds only).
#[cfg(feature = "test-support")]
pub fn armour_for_test(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}

impl Invite {
    /// Parse an Invite Blob, naming the field that fails (LLR-b7wgpf).
    pub fn parse(blob: &str) -> Result<Self, OrgNodeError> {
        let wire: WireInvite = unarmour("invite", blob)?;
        Ok(Self {
            org_name: wire.org_name,
            org_id: OrgId::new(exact("invite.org_id", &wire.org_id)?),
            invitee_name: wire.invitee_name,
            inviter_device_keys: wire
                .inviter_device_keys
                .iter()
                .map(|key| {
                    parse_field(
                        "invite.inviter_device_keys",
                        DevicePublicKey::parse(&exact("invite.inviter_device_keys", key)?),
                    )
                })
                .collect::<Result<_, _>>()?,
            invite_id: InviteId::new(exact("invite.invite_id", &wire.invite_id)?),
        })
    }

    /// This Invite as a Blob.
    pub fn encode(&self) -> Result<String, OrgNodeError> {
        armour(&WireInvite {
            org_name: self.org_name.clone(),
            org_id: self.org_id.as_bytes().to_vec(),
            invitee_name: self.invitee_name.clone(),
            inviter_device_keys: self.inviter_device_keys.iter().map(|key| key.as_bytes().to_vec()).collect(),
            invite_id: self.invite_id.as_bytes().to_vec(),
        })
    }

    /// A Blob from raw parts, so a test can present one this software never
    /// writes (test builds only).
    #[cfg(feature = "test-support")]
    pub fn wire_for_test(org_name: &str, org_id: &[u8], invitee: &str, keys: &[[u8; 32]], invite_id: &[u8]) -> String {
        armour(&WireInvite {
            org_name: org_name.into(),
            org_id: org_id.to_vec(),
            invitee_name: invitee.into(),
            inviter_device_keys: keys.iter().map(|key| key.to_vec()).collect(),
            invite_id: invite_id.to_vec(),
        })
        .unwrap_or_default()
    }
}

impl InviteReply {
    /// Parse an Invite reply Blob, naming the field that fails (LLR-b7wgpf).
    pub fn parse(blob: &str) -> Result<Self, OrgNodeError> {
        let wire: WireReply = unarmour("reply", blob)?;
        Ok(Self {
            org_id: OrgId::new(exact("reply.org_id", &wire.org_id)?),
            invite_id: InviteId::new(exact("reply.invite_id", &wire.invite_id)?),
            member_key: parse_field("reply.member_key", PersonPublicKey::parse(&exact("reply.member_key", &wire.member_key)?))?,
            device_key: parse_field("reply.device_key", DevicePublicKey::parse(&exact("reply.device_key", &wire.device_key)?))?,
            handle: parse_field("reply.handle", Handle::parse(&wire.handle))?,
            name: parse_field("reply.name", Name::parse(&wire.name))?,
            surname: parse_field("reply.surname", Surname::parse(&wire.surname))?,
        })
    }

    /// This reply as a Blob.
    pub fn encode(&self) -> Result<String, OrgNodeError> {
        armour(&WireReply {
            org_id: self.org_id.as_bytes().to_vec(),
            invite_id: self.invite_id.as_bytes().to_vec(),
            member_key: self.member_key.as_bytes().to_vec(),
            device_key: self.device_key.as_bytes().to_vec(),
            handle: self.handle.to_string(),
            name: self.name.to_string(),
            surname: self.surname.to_string(),
        })
    }

    /// A Blob from raw parts, so a test can present one this software never
    /// writes (test builds only).
    #[cfg(feature = "test-support")]
    pub fn wire_for_test(
        org_id: &[u8],
        invite_id: &[u8],
        member_key: &[u8; 32],
        device_key: &[u8; 32],
        handle: &str,
        name: &str,
        surname: &str,
    ) -> String {
        armour(&WireReply {
            org_id: org_id.to_vec(),
            invite_id: invite_id.to_vec(),
            member_key: member_key.to_vec(),
            device_key: device_key.to_vec(),
            handle: handle.into(),
            name: name.into(),
            surname: surname.into(),
        })
        .unwrap_or_default()
    }
}

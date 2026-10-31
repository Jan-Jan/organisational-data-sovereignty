//! The Invite (SDD-n2x3gb) and the endpoint addresses (SDD-42cjqk): the Invite a Member
//! sends, as a Blob (Base64 of postcard), parsed here, naming the field that
//! fails; and the endpoint addresses one Device keeps for another
//! (`EndpointAddresses`, LLR-h4gwns). An invitee answers an Invite with an
//! admission request (LLR-96z2k5).
//! org-io runs the exchange, and the service
//! keeps the outstanding pairs in the sealed store (`OrgService::build_invite`
//! and the operations beside it). Nothing here is verified: an Invite's
//! sender and its Organisation name are whatever the sender chose.

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};

use org_members::DevicePublicKey;

pub use crate::ids::{EndpointAddresses, EndpointAddressesError, InviteId};
use crate::ids::OrgId;
use crate::store::parse_field;
use crate::OrgNodeError;

/// An Invite as parsed (LLR-9jxwq4, LLR-h9f5c9): the Organisation name the
/// issuing store holds, the Organisation, the invitee name the inviter
/// typed, the one issuing Device and its endpoint addresses, and the
/// identifier an admission request must carry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invite {
    pub org_name: String,
    pub org_id: OrgId,
    pub invitee_name: String,
    pub issuer: DevicePublicKey,
    pub issuer_addresses: EndpointAddresses,
    pub invite_id: InviteId,
}

/// The Invite as the Blob holds it, every field in the form it is parsed
/// from, so each can be refused by name.
#[derive(Serialize, Deserialize)]
struct WireInvite {
    org_name: Vec<u8>,
    org_id: Vec<u8>,
    invitee_name: Vec<u8>,
    issuer: Vec<u8>,
    issuer_addresses: Vec<String>,
    invite_id: Vec<u8>,
}

/// UTF-8 text, else `InvalidField` naming `field`.
fn text(field: &'static str, bytes: Vec<u8>) -> Result<String, OrgNodeError> {
    parse_field(field, String::from_utf8(bytes))
}

/// Exactly `N` bytes, else `InvalidField` naming `field`.
fn exact<const N: usize>(field: &'static str, bytes: &[u8]) -> Result<[u8; N], OrgNodeError> {
    bytes.try_into().map_err(|_| OrgNodeError::InvalidField {
        field,
        reason: format!("expected {N} bytes, got {}", bytes.len()),
    })
}

/// The value inside the Blob `blob`, else `InvalidField` naming `what`
/// (`invite`).
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
    /// Parse an Invite Blob (LLR-9jxwq4): text that is not a Blob of an
    /// Invite is refused naming `invite`; then each field is parsed by its
    /// type in order, the first that fails refused naming it —
    /// `invite.org_name` and `invite.invitee_name` (UTF-8 text),
    /// `invite.org_id` (twenty bytes), `invite.issuer`
    /// (`DevicePublicKey::parse`), `invite.issuer_addresses`
    /// (`EndpointAddresses::parse`), `invite.invite_id` (thirty-two bytes).
    /// Pure: a refused Invite acts on nothing.
    pub fn parse(blob: &str) -> Result<Self, OrgNodeError> {
        let wire: WireInvite = unarmour("invite", blob)?;
        Ok(Self {
            org_name: text("invite.org_name", wire.org_name)?,
            org_id: OrgId::new(exact("invite.org_id", &wire.org_id)?),
            invitee_name: text("invite.invitee_name", wire.invitee_name)?,
            issuer: parse_field("invite.issuer", DevicePublicKey::parse(&exact("invite.issuer", &wire.issuer)?))?,
            issuer_addresses: parse_field("invite.issuer_addresses", EndpointAddresses::parse(wire.issuer_addresses))?,
            invite_id: InviteId::new(exact("invite.invite_id", &wire.invite_id)?),
        })
    }

    /// This Invite as a Blob; it parses back to an equal Invite.
    pub fn encode(&self) -> Result<String, OrgNodeError> {
        armour(&WireInvite {
            org_name: self.org_name.clone().into_bytes(),
            org_id: self.org_id.as_bytes().to_vec(),
            invitee_name: self.invitee_name.clone().into_bytes(),
            issuer: self.issuer.as_bytes().to_vec(),
            issuer_addresses: self.issuer_addresses.as_slice().to_vec(),
            invite_id: self.invite_id.as_bytes().to_vec(),
        })
    }

    /// A Blob from raw parts, so a test can present one this software never
    /// writes (test builds only), or the refusal its encoding returns.
    #[cfg(feature = "test-support")]
    pub fn wire_for_test(
        org_name: &[u8],
        org_id: &[u8],
        invitee_name: &[u8],
        issuer: &[u8],
        issuer_addresses: &[&str],
        invite_id: &[u8],
    ) -> Result<String, OrgNodeError> {
        armour(&WireInvite {
            org_name: org_name.to_vec(),
            org_id: org_id.to_vec(),
            invitee_name: invitee_name.to_vec(),
            issuer: issuer.to_vec(),
            issuer_addresses: issuer_addresses.iter().map(|address| address.to_string()).collect(),
            invite_id: invite_id.to_vec(),
        })
    }
}

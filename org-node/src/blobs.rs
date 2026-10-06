//! Out-of-band exchange blobs (spec story 2). Base64(postcard(..)) for copy/paste.
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};

use org_members::{DevicePublicKey, Handle, Name, PersonPublicKey, Surname};

use crate::ids::OrgId;
use crate::store::parse_field;
use crate::types::OrgPublicKey;
use crate::OrgNodeError;

/// A → B: enough for B to read the org slot and to dial A. Nothing in it is
/// compared with a message B receives (REQ-xa6smf, REQ-ztdza4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invite {
    pub org_id: OrgId,
    pub org_pub_key: OrgPublicKey,
    /// Parsed on decode: a valid X25519 Member-as-a-group key or no Invite.
    pub admin_member_key: PersonPublicKey,
    /// Parsed on decode: a valid DevicePublicKey or no Invite.
    pub admin_device_key: DevicePublicKey,
    /// postcard-encoded iroh EndpointAddr for dialing A.
    pub admin_node_addr: Vec<u8>,
}

/// B → A: B's proposed persona, so A can mint a member_id and add B.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawJoinRequest")]
pub struct JoinRequest {
    pub handle: Handle,
    pub name: Name,
    pub surname: Surname,
    pub member_key: PersonPublicKey,
    pub device_key: DevicePublicKey,
    /// postcard-encoded iroh EndpointAddr for dialing B.
    pub node_addr: Vec<u8>,
}

/// A Join request as decoded, before parsing (LLR-8bum44).
#[derive(Deserialize)]
pub(crate) struct RawJoinRequest {
    handle: String,
    name: String,
    surname: String,
    member_key: [u8; 32],
    device_key: [u8; 32],
    node_addr: Vec<u8>,
}

impl TryFrom<RawJoinRequest> for JoinRequest {
    type Error = OrgNodeError;

    fn try_from(raw: RawJoinRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            handle: parse_field("join_request.handle", Handle::parse(&raw.handle))?,
            name: parse_field("join_request.name", Name::parse(&raw.name))?,
            surname: parse_field("join_request.surname", Surname::parse(&raw.surname))?,
            member_key: parse_field("join_request.member_key", PersonPublicKey::parse(&raw.member_key))?,
            device_key: parse_field("join_request.device_key", DevicePublicKey::parse(&raw.device_key))?,
            node_addr: raw.node_addr,
        })
    }
}

/// Encode a blob to a Base64 string (STANDARD alphabet, padded).
pub fn encode<T: Serialize>(v: &T) -> Result<String, OrgNodeError> {
    let bytes = postcard::to_allocvec(v)
        .map_err(|e| OrgNodeError::Chain(format!("blob encode: {e}")))?;
    Ok(STANDARD.encode(&bytes))
}

/// Decode a blob from a Base64 string.
pub fn decode<T: for<'de> Deserialize<'de>>(s: &str) -> Result<T, OrgNodeError> {
    let bytes = STANDARD
        .decode(s)
        .map_err(|e| OrgNodeError::Chain(format!("blob base64: {e}")))?;
    postcard::from_bytes(&bytes)
        .map_err(|e| OrgNodeError::Chain(format!("blob decode: {e}")))
}

/// Decode a Join request blob; a value its type's parse refuses is reported
/// with the field's name (LLR-8bum44).
pub fn decode_join_request(s: &str) -> Result<JoinRequest, OrgNodeError> {
    let raw: RawJoinRequest = decode(s)?;
    JoinRequest::try_from(raw)
}

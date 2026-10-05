//! Out-of-band exchange blobs (spec story 2). Base64(postcard(..)) for copy/paste.
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};

use crate::ids::OrgId;
use crate::OrgNodeError;

/// A → B: enough for B to read the org slot and to dial / authenticate A.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invite {
    pub org_id: OrgId,
    pub org_pub_key: [u8; 32],
    pub admin_member_key: [u8; 32],
    pub admin_device_key: [u8; 32],
    /// postcard-encoded iroh EndpointAddr for dialing A.
    pub admin_node_addr: Vec<u8>,
}

/// B → A: B's proposed persona, so A can mint a member_id and add B.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinRequest {
    pub handle: String,
    pub name: String,
    pub surname: String,
    pub member_key: [u8; 32],
    pub device_key: [u8; 32],
    /// postcard-encoded iroh EndpointAddr for dialing B.
    pub node_addr: Vec<u8>,
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


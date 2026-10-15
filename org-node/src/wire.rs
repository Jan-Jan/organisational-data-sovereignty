//! The Wire message: its three kinds, its encoding and its bound. Pure
//! values; the length prefix the channel puts in front of a message belongs
//! to the transport, not here.
use serde::{Deserialize, Serialize};

use crate::envelope::Envelope;
use crate::error::OrgNodeError;
use crate::ids::OrgId;
use crate::revocation::{Acknowledgement, RevocationNotice};
use crate::types::OrgPrivateKey;

/// The largest encoded Wire message accepted or produced (a defensive cap
/// against a hostile peer): 1 MiB, generous for a delta, a record snapshot
/// and a key (LLR-8kh3zf, LLR-sc6zuh).
pub const MAX_MESSAGE: usize = 1024 * 1024;

/// One message over the org-node channel, of one of three kinds
/// (LLR-tajh9d). The kind follows the recipient, not the operation
/// (REQ-3dsweu): a Device the sending node's committed record lists receives
/// Organisation information, a Device it removed a revocation notice; a
/// revoked Device answers with an acknowledgement. No kind carries an invite
/// identifier (LLR-ms8njy). `Debug` is derived: the only secret any kind
/// holds is an `OrgPrivateKey`, whose own `Debug` redacts it (LLR-ecxc76).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WireMessage {
    /// Index 0: the committed Envelope; the record it extends, postcard
    /// `Vec<MemberSnapshot>` as it stood before the commit, so a joiner with
    /// no record can rebuild the trie the delta's `base_root` names
    /// (LLR-bg3vsw); and the Organisation private key of the epoch the update
    /// reaches (REQ-szq3ud). A body without either does not decode
    /// (REQ-c29s93).
    OrgInformation { envelope: Envelope, record_snapshot: Vec<u8>, org_private_key: OrgPrivateKey },
    /// Index 1: the revoked Device's identity and an absence proof, no
    /// Envelope (LLR-dc45ur).
    Revocation(RevocationNotice),
    /// Index 2: a revoked Device's signed acknowledgement (LLR-378cj4).
    Acknowledgement(Acknowledgement),
}

impl WireMessage {
    /// The Organisation any kind names (LLR-tajh9d).
    pub fn org_id(&self) -> OrgId {
        match self {
            Self::OrgInformation { envelope, .. } => envelope.org_id,
            Self::Revocation(notice) => notice.org_id,
            Self::Acknowledgement(acknowledgement) => acknowledgement.org_id,
        }
    }
}

/// Encode a Wire message as its postcard body, with no length prefix
/// (LLR-er2x8n). An encoding over [`MAX_MESSAGE`] is refused (LLR-sc6zuh).
pub fn encode_message(message: &WireMessage) -> Result<Vec<u8>, OrgNodeError> {
    let body = postcard::to_allocvec(message).map_err(|_| OrgNodeError::MalformedMessage)?;
    if body.len() > MAX_MESSAGE {
        return Err(OrgNodeError::MessageTooLarge { length: body.len() });
    }
    Ok(body)
}

/// Decode a Wire message from its postcard body. A body over [`MAX_MESSAGE`]
/// is refused before decoding (LLR-8kh3zf); a body that is not a Wire
/// message, or has bytes left over after one of any kind, is
/// `MalformedMessage` (LLR-pkruy8, LLR-tajh9d, LLR-dc45ur, LLR-378cj4).
pub fn decode_message(body: &[u8]) -> Result<WireMessage, OrgNodeError> {
    if body.len() > MAX_MESSAGE {
        return Err(OrgNodeError::MessageTooLarge { length: body.len() });
    }
    match postcard::take_from_bytes(body) {
        Ok((message, [])) => Ok(message),
        _ => Err(OrgNodeError::MalformedMessage),
    }
}

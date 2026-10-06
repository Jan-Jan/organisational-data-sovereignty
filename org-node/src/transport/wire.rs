//! The wire payload exchanged over the channel, with length-prefixed framing.
use serde::{Deserialize, Serialize};

use crate::envelope::Envelope;
use crate::types::OrgPrivateKey;
use crate::transport::{TransportError, MAX_FRAME};

/// One message over the org-node channel, of one of two kinds (LLR-js9dsu).
/// The kind follows the recipient, not the operation (REQ-3dsweu): a Device
/// the sending node's committed record lists receives Organisation
/// information, any other Device a revocation. Neither kind carries an
/// invite identifier (LLR-ms8njy). `Debug` is derived: the only secret
/// either kind holds is an `OrgPrivateKey`, whose own `Debug` redacts it
/// (LLR-ecxc76).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WireMessage {
    /// Index 0: the committed Envelope; the record it extends, postcard
    /// `Vec<MemberSnapshot>` as it stood before the commit, so a joiner with
    /// no record can rebuild the trie the delta's `base_root` names
    /// (LLR-bg3vsw); and the Organisation private key of the epoch the update
    /// reaches (REQ-szq3ud). A body without either does not decode
    /// (REQ-c29s93).
    OrgInformation { envelope: Envelope, record_snapshot: Vec<u8>, org_private_key: OrgPrivateKey },
    /// Index 1: the committed Envelope alone — no snapshot, no key.
    Revocation { envelope: Envelope },
}

impl WireMessage {
    /// The Envelope of either kind (LLR-js9dsu).
    pub fn envelope(&self) -> &Envelope {
        match self {
            Self::OrgInformation { envelope, .. } | Self::Revocation { envelope } => envelope,
        }
    }
}

/// Encode a WireMessage as `len(u32 LE) ‖ postcard(msg)`.
pub fn encode_frame(msg: &WireMessage) -> Result<Vec<u8>, TransportError> {
    let body = postcard::to_allocvec(msg).map_err(|_| TransportError::Malformed)?;
    if body.len() > MAX_FRAME {
        return Err(TransportError::FrameTooLarge(body.len()));
    }
    let mut framed = Vec::with_capacity(4 + body.len());
    framed.extend_from_slice(&(body.len() as u32).to_le_bytes());
    framed.extend_from_slice(&body);
    Ok(framed)
}

/// Decode the postcard body (already de-framed) into a WireMessage.
pub fn decode_body(body: &[u8]) -> Result<WireMessage, TransportError> {
    if body.len() > MAX_FRAME {
        return Err(TransportError::FrameTooLarge(body.len()));
    }
    postcard::from_bytes(body).map_err(|_| TransportError::Malformed)
}


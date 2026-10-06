//! The wire payload exchanged over the channel, with length-prefixed framing.
use serde::{Deserialize, Serialize};

use crate::envelope::Envelope;
use crate::types::{InviteId, OrgSecret};
use crate::transport::{TransportError, MAX_FRAME};

/// One message over the org-node channel: an Envelope carrying one delta,
/// plus the Organisation secret the sender chose to hand over, if any.
///
/// `genesis_snapshot` carries postcard-encoded `Vec<MemberSnapshot>` (from the
/// `app` feature store module): the record the Envelope extends, as it stood
/// before the sender's commit. `send_update` includes it in every update, so
/// a recipient with no record of the Organisation (a first admission) can
/// rebuild the trie the delta's `base_root` names.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireMessage {
    pub envelope: Envelope,
    /// The Organisation secret, redacted in `Debug`.
    pub org_secret: Option<OrgSecret>,
    /// postcard(Vec<MemberSnapshot>): the record the Envelope extends.
    pub genesis_snapshot: Option<Vec<u8>>,
    /// The invite identifier an admission is delivered under (LLR-ms8njy,
    /// REQ-8amu2a); `None` for every other update.
    pub invite_id: Option<InviteId>,
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


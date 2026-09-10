//! The wire payload exchanged over the channel, with length-prefixed framing.
use serde::{Deserialize, Serialize};

use crate::envelope::SignedDeltaEnvelope;
use crate::transport::{TransportError, MAX_FRAME};

/// One message over the org-node channel: a signed delta, plus (on admission)
/// the org secret key handed to a newly verified member.
///
/// `genesis_snapshot` carries postcard-encoded `Vec<MemberSnapshot>` (from the
/// `app` feature store module).  It is included in admission messages so the
/// recipient can reconstruct the genesis trie when it has no prior OrgRecord,
/// enabling `verify_envelope_against_chain` to pass the `base_root` check.
/// `None` for non-admission messages (e.g. revocations).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireMessage {
    pub envelope: SignedDeltaEnvelope,
    pub org_secret: Option<[u8; 32]>,
    /// postcard(Vec<MemberSnapshot>) — genesis members; None for non-admission.
    pub genesis_snapshot: Option<Vec<u8>>,
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


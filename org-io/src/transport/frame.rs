//! The framing of one Wire message on its stream (LLR-fa7jt8, LLR-k2y6nn,
//! LLR-2smrvx): `len(u32 LE) ‖ body`,
//! where `body` is exactly what org-node's `encode_message` returns and its
//! `decode_message` takes. The length prefix is the transport's; the bound on
//! the body is org-node's codec's.

use org_node::wire::{encode_message, WireMessage, MAX_MESSAGE};

use super::TransportError;

/// The largest body a frame carries: the Wire message's bound.
pub const MAX_FRAME: usize = MAX_MESSAGE;

/// The most bytes read from one inbound stream: a body at the bound and its
/// prefix (LLR-k2y6nn).
pub(crate) const MAX_STREAM_READ: usize = MAX_FRAME + 4;

/// Frame a Wire message for the channel (LLR-fa7jt8).
pub fn encode_frame(message: &WireMessage) -> Result<Vec<u8>, TransportError> {
    frame_body(&encode_message(message)?)
}

/// Frame one already encoded body: its length as four little-endian bytes,
/// then the body, and nothing else. A body over [`MAX_FRAME`] is refused.
pub(crate) fn frame_body(body: &[u8]) -> Result<Vec<u8>, TransportError> {
    if body.len() > MAX_FRAME {
        return Err(TransportError::FrameTooLarge(body.len()));
    }
    let mut framed = Vec::with_capacity(4 + body.len());
    framed.extend_from_slice(&(body.len() as u32).to_le_bytes());
    framed.extend_from_slice(body);
    Ok(framed)
}

/// The body of a stream read whole: the four-byte prefix discarded without
/// being compared with the body, and a stream shorter than four bytes taken
/// whole (LLR-2smrvx).
pub(crate) fn body_of(stream: &[u8]) -> &[u8] {
    if stream.len() >= 4 {
        &stream[4..]
    } else {
        stream
    }
}

//! Device-to-device transport over iroh QUIC (SDD-8uyg4s). The peer's authenticated EndpointId is its
//! DevicePublicKey, so a connection proves device-key custody. Every endpoint
//! is bound from its Device's OS keychain item
//! (`OrgEndpoint::bind_from_keychain`, LLR-d5y7ks).
//!
//! The endpoint module and its items are crate-private in a production build: no type reachable from
//! org-io's public API holds an `OrgEndpoint`, and two callers bind one:
//! `Endpoints`, which keeps the node's endpoints (LLR-d5y7ks, LLR-d2pzzk),
//! and the transport preflight, `preflight::check_transport`, which binds a
//! Device's endpoint for its check and drops it once the check is made
//! (LLR-h9tcat). Test builds
//! (`test-support`) re-export them for the transport's tests.

mod endpoint;
pub mod frame;

use org_node::{DevicePublicKey, OrgNodeError};

use crate::keychain::KeychainError;

#[cfg(feature = "test-support")]
pub use endpoint::{BindPlan, InboundReply, OrgEndpoint, PendingConnection, CONNECTION_TIME_LIMIT};
#[cfg(not(feature = "test-support"))]
pub(crate) use endpoint::{BindPlan, InboundReply, OrgEndpoint, PendingConnection, CONNECTION_TIME_LIMIT};
pub use frame::{encode_frame, MAX_FRAME};

/// Selects how an endpoint is bound and how peers are dialled.
///
/// - `Loopback`: relay disabled, binds on `127.0.0.1`, dials the
///   addresses kept for the peer's DevicePublicKey.  Used by offline tests and
///   same-machine demo runs.
/// - `Networked`: uses iroh's `presets::N0` (n0 relay servers + DNS/Pkarr
///   discovery), binds on all interfaces, and dials purely by `EndpointId`
///   so iroh's discovery/relay resolves connectivity.  Required for two
///   laptops over the internet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportMode {
    /// Relay disabled; bind loopback; dial by `EndpointAddr`.
    Loopback,
    /// n0 relay + DNS discovery; bind wildcard; dial by `EndpointId`.
    Networked,
}

/// ALPN protocol id of the ODS channel. It is the id org-node's transport
/// used, so nodes built with that transport interoperate with this one.
pub const ALPN: &[u8] = b"/ods/org-node/1";

/// The one byte the receive side writes back on an inbound stream, after
/// the body (LLR-hhgdk7): whether the message was acted on, and for an
/// admission request, which of org-node's verdicts it got.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Acted on, and its store change (if any) written.
    Accepted,
    /// Refused, or failed, at any step.
    Refused,
    /// An admission request this store already keeps from the sender.
    AlreadyReceived,
    /// An admission request for an invite another Device already used.
    InviteUsedByAnotherDevice,
}

impl Verdict {
    /// The byte on the stream: `0x00` accepted, `0x01` refused, `0x02`
    /// already received, `0x03` invite used by another Device.
    pub fn as_byte(self) -> u8 {
        match self {
            Self::Accepted => 0x00,
            Self::Refused => 0x01,
            Self::AlreadyReceived => 0x02,
            Self::InviteUsedByAnotherDevice => 0x03,
        }
    }

    /// The verdict a byte read back is, or `None` for any other byte
    /// (LLR-fwz3tb).
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Self::Accepted),
            0x01 => Some(Self::Refused),
            0x02 => Some(Self::AlreadyReceived),
            0x03 => Some(Self::InviteUsedByAnotherDevice),
            _ => None,
        }
    }
}

/// How long a send waits for the verdict byte after finishing its stream
/// (LLR-fwz3tb).
pub const VERDICT_TIME_LIMIT: std::time::Duration = std::time::Duration::from_secs(10);

/// The endpoint identity and socket a kept address names — the form
/// `OrgEndpoint::addresses` emits (LLR-v9dkrv): the endpoint identity as 64
/// lower-case hex, `@`, the socket address — or `None` for text of any
/// other form (the disk and the wire are edges: parsed).
pub(crate) fn parse_kept_address(text: &str) -> Option<([u8; 32], std::net::SocketAddr)> {
    let (id, socket) = text.split_once('@')?;
    if id.len() != 64 || !id.chars().all(|character| character.is_ascii_digit() || ('a'..='f').contains(&character)) {
        return None;
    }
    let id: [u8; 32] = hex::decode(id).ok()?.try_into().ok()?;
    Some((id, socket.parse().ok()?))
}

/// Why the transport did not bind, connect, accept or read. Public data
/// only: reasons as text, lengths, Device public keys and the keychain's
/// typed refusals, which name no byte of an item (LLR-8erfbu).
#[derive(Debug)]
pub enum TransportError {
    Bind(String),
    Connect(String),
    /// One inbound connection failed before it carried a stream.
    Accept(String),
    /// The endpoint is closed: it accepts nothing more.
    EndpointClosed,
    Stream(String),
    /// One inbound connection took longer than the limit, from its
    /// handshake to the end of its body, and was dropped (LLR-8uf9a5).
    TimedOut(std::time::Duration),
    FrameTooLarge(usize),
    Malformed,
    /// The keychain gave no seed for the Device key (LLR-d5y7ks). Boxed, as
    /// the next variant: a Device public key is a parsed point, and the
    /// error travels on every transport result.
    Keychain(Box<KeychainError>),
    /// The keychain item of the Device key holds another key's seed; nothing
    /// was bound (LLR-d5y7ks).
    KeychainKeyMismatch(Box<DevicePublicKey>),
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bind(reason) => write!(formatter, "iroh bind error: {reason}"),
            Self::Connect(reason) => write!(formatter, "iroh connect error: {reason}"),
            Self::Accept(reason) => write!(formatter, "iroh accept error: {reason}"),
            Self::EndpointClosed => write!(formatter, "iroh accept error: endpoint closed"),
            Self::Stream(reason) => write!(formatter, "stream error: {reason}"),
            Self::TimedOut(limit) => write!(formatter, "inbound connection dropped: no body within {} s", limit.as_secs_f64()),
            Self::FrameTooLarge(length) => write!(formatter, "frame too large: {length} bytes (max {MAX_FRAME})"),
            Self::Malformed => write!(formatter, "malformed wire message"),
            Self::Keychain(error) => write!(formatter, "keychain: {error}"),
            Self::KeychainKeyMismatch(device_key) => write!(
                formatter,
                "the keychain item of DevicePublicKey {} holds another key's Device seed; nothing bound",
                hex::encode(device_key.as_bytes())
            ),
        }
    }
}

impl std::error::Error for TransportError {}

impl From<OrgNodeError> for TransportError {
    /// The codec's two refusals as the transport reports them.
    fn from(error: OrgNodeError) -> Self {
        match error {
            OrgNodeError::MessageTooLarge { length } => TransportError::FrameTooLarge(length),
            _ => TransportError::Malformed,
        }
    }
}

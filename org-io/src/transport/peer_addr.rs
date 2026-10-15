//! The peer address as text (SDD-wr8m5n): the address a Loopback send dials, given by the operator as
//! the hexadecimal of a postcard-encoded endpoint address. The parse is
//! total (LLR-2b2ghe): blank text is no address, and every other text is an
//! address or a refusal that names the field. The app passes text and names
//! no iroh type; `PeerAddr` keeps the parsed address inside the transport
//! module, the one place org-io names iroh (LLR-3vb4k9), and `lib.rs`
//! re-exports this module as `org_io::peer_addr`.

/// A parsed peer address: an endpoint identity and the transport addresses
/// it is reached at, as the operator gave them. Only `parse_peer_addr` (and
/// a test build) makes one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerAddr(iroh::EndpointAddr);

impl PeerAddr {
    /// The endpoint identity the address names: the Device public key's
    /// bytes, when the address is a Device's.
    pub fn endpoint_id(&self) -> [u8; 32] {
        *self.0.id.as_bytes()
    }

    /// The address as `parse_peer_addr` takes it, bare hexadecimal.
    pub fn to_hex(&self) -> String {
        // An `EndpointAddr` always encodes: postcard fails only on a writer
        // that runs out of room or a type it cannot represent.
        hex::encode(postcard::to_allocvec(&self.0).unwrap_or_default())
    }

    /// The address to dial, for the transport module alone.
    pub(super) fn endpoint_addr(&self) -> &iroh::EndpointAddr {
        &self.0
    }

    /// The address `addr` (test builds only: production addresses come from
    /// the parse).
    #[cfg(feature = "test-support")]
    pub fn for_test(addr: iroh::EndpointAddr) -> Self {
        Self(addr)
    }
}

/// Why a peer address was refused. Each names what it refused and carries
/// the decoder's reason: the hex step names the IPC argument the text
/// arrives in, `peer_addr_blob`; the decode step names the value it failed
/// to decode, `peer_addr`. The text is the operator's own input, not a
/// secret.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PeerAddrError {
    /// The text, after at most one `0x`, is not hexadecimal.
    Hex(String),
    /// The bytes are not exactly one postcard-encoded endpoint address.
    Decode(String),
}

impl std::fmt::Display for PeerAddrError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hex(reason) => write!(formatter, "peer_addr_blob hex: {reason}"),
            Self::Decode(reason) => write!(formatter, "peer_addr decode: {reason}"),
        }
    }
}

impl std::error::Error for PeerAddrError {}

/// Parse a peer address given as text (LLR-2b2ghe): blank or
/// whitespace-only text is `None`; otherwise, after trimming and at most one
/// `0x`, the hexadecimal of exactly one postcard-encoded endpoint address
/// is `Some`; anything else is refused, and nothing panics.
pub fn parse_peer_addr(text: &str) -> Result<Option<PeerAddr>, PeerAddrError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let hex_text = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    let bytes = hex::decode(hex_text).map_err(|error| PeerAddrError::Hex(error.to_string()))?;
    let (addr, rest) = postcard::take_from_bytes::<iroh::EndpointAddr>(&bytes)
        .map_err(|error| PeerAddrError::Decode(error.to_string()))?;
    if !rest.is_empty() {
        return Err(PeerAddrError::Decode(format!("{} bytes after the address", rest.len())));
    }
    Ok(Some(PeerAddr(addr)))
}

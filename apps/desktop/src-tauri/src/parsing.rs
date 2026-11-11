//! Input parsing at the command boundary.

use org_io::node::{DevicePublicKey, InviteId, MemberId, OrgId};

/// REQ-sjkp8z. Accepts 40 hex characters, with or without a `0x` prefix.
///
/// ONE optional prefix, not a run of them. `trim_start_matches` strips
/// REPEATEDLY, so it accepted `0x0x` + 40 hex characters as a well-formed
/// identifier — malformed input silently normalised into a valid `OrgId` at a
/// boundary parser. `strip_prefix` removes at most one, leaving a second `0x`
/// to count toward the width and be refused there, which is the correct
/// answer. Gated by `tests/org_id_parsing.rs`.
pub fn parse_org_id(s: &str) -> Result<OrgId, String> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.len() != 40 {
        return Err(format!("org_id must be 40 hex chars, got {}", s.len()));
    }
    let bytes = hex::decode(s).map_err(|e| format!("org_id hex: {e}"))?;
    let mut arr = [0u8; 20];
    arr.copy_from_slice(&bytes);
    Ok(OrgId::new(arr))
}

/// Thirty-two bytes from hex, with at most one leading `0x`: a remainder
/// that is not hex is refused with `<field> hex: <reason>`, one that is not 32
/// bytes with `<field> must be 32 bytes (64 hex chars)`.
fn parse_32_bytes(field: &str, hex_text: &str) -> Result<[u8; 32], String> {
    let hex_text = hex_text.strip_prefix("0x").unwrap_or(hex_text);
    let bytes = hex::decode(hex_text).map_err(|error| format!("{field} hex: {error}"))?;
    <[u8; 32]>::try_from(bytes.as_slice()).map_err(|_| format!("{field} must be 32 bytes (64 hex chars)"))
}

/// LLR-7n4hgr: a Member identifier (64 hex chars, one optional `0x`).
pub fn parse_member_id(member_id: &str) -> Result<MemberId, String> {
    parse_32_bytes("member_id", member_id).map(MemberId::new)
}

/// LLR-r8jeu8: an invite identifier (64 hex chars, one optional `0x`).
pub fn parse_invite_id(invite_id: &str) -> Result<InviteId, String> {
    parse_32_bytes("invite_id", invite_id).map(InviteId::new)
}

/// LLR-rhc555: a founding's founder, its Device key (64 hex chars, one
/// optional `0x`).
pub fn parse_founder(founder: &str) -> Result<DevicePublicKey, String> {
    let bytes = parse_32_bytes("founder", founder)?;
    DevicePublicKey::parse(&bytes).map_err(|error| format!("founder: {error}"))
}

//! Input parsing at the command boundary.

use org_io::node::OrgId;

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

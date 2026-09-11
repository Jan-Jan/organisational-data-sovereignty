//! Runtime-version → decoder dispatch. `for_runtime(spec_version)` is the
//! one place that maps a number from `Rpc::runtime_version().spec_version`
//! to a static decoder impl. Adding support for a new Paseo AH runtime
//! version means adding a `decode::v_paseo_ah_<N>` module and a match arm
//! here — nothing else changes.
//!
//! Tested from `on-chain-client/tests/runtime_version_dispatch.rs`, not
//! from a `#[cfg(test)]` module here: this unit's gate reads its evidence
//! from `test_paths` (`on-chain-client/tests`), so an annotated test
//! written in `src` would be read by no gate.

use super::{DecodeError, Decoder, v_paseo_ah};

/// Paseo AH runtime spec_version pinned at Stage 2 implementation time.
/// The exact value will be confirmed (and possibly extended to a range of
/// known-good versions) in Task 10 once we've round-tripped the live
/// endpoint. For now this is what the decoder is wired against; any other
/// `spec_version` returns `DecodeError::UnsupportedRuntime`.
pub const PASEO_AH_SPEC_VERSION: u32 = v_paseo_ah::SPEC_VERSION;

/// Look up the decoder for a given runtime `spec_version`. Returns
/// `DecodeError::UnsupportedRuntime` if no compiled-in decoder matches —
/// callers (`OrgRegistryClient::get_org_state` in Task 5) should refuse
/// to read state in that case rather than fall back to a guess, since
/// pallet-revive's storage layout can change across runtimes.
pub fn for_runtime(spec_version: u32) -> Result<&'static dyn Decoder, DecodeError> {
    match spec_version {
        PASEO_AH_SPEC_VERSION => Ok(&v_paseo_ah::DECODER),
        _ => Err(DecodeError::UnsupportedRuntime { spec_version }),
    }
}

//! On-chain write path: build update() calldata, drive a threshold-1 pure-proxy
//! multisig, and submit extrinsics via subxt. Productionised from
//! on-chain-client/tests/common; decoupled from chopsticks (submit returns the
//! extrinsic hash; block production is the caller's concern).
#![cfg(feature = "chain")]

pub mod calldata;
pub mod multisig;
pub mod proxy;
pub mod submit;

use thiserror::Error;

/// Errors from the on-chain write path.
#[derive(Debug, Error)]
pub enum WriteError {
    #[error("subxt error: {0}")]
    Subxt(String),
    #[error("expected on-chain event not found: {0}")]
    EventNotFound(&'static str),
    #[error("malformed event field: {0}")]
    MalformedEvent(&'static str),
    #[error("malformed call: {0}")]
    MalformedCall(&'static str),
}

use subxt::config::PolkadotConfig;
use subxt::extrinsics::ExtrinsicEvents;

/// Outcome of submitting one signer's contribution to an org-controlling call.
///
/// Today's controllers (a single admin signing directly, or a threshold-1
/// multisig) always execute in one transaction, so only `Executed` is produced.
/// The variant set is defined now so callers handle the pending case rather than
/// silently treating it as success. `#[must_use]` means a dropped outcome warns,
/// and `into_executed()` is the explicit "this flow can't be left pending" assert
/// used by genesis / single-admin / threshold-1 sites. When the threshold-≥2 flow
/// is added it produces `ApprovalRecorded` for non-final approvals, and those
/// guards turn a pending approval into an error instead of a false success.
#[derive(Debug)]
#[must_use = "a dispatch may be a pending multisig approval, not an executed call; \
              handle it (e.g. via `into_executed()`)"]
pub enum DispatchOutcome {
    /// The inner call executed in this transaction; carries the executed call's
    /// own events (e.g. `Proxy.PureCreated`, the `Revive.update` effect).
    Executed(ExtrinsicEvents<PolkadotConfig>),
    /// The signature was recorded but the multisig threshold is not yet met —
    /// the inner call has NOT executed. Reserved for the threshold-≥2 flow; its
    /// payload (call hash, timepoint, approvals) is finalised when that ships.
    #[allow(dead_code)]
    ApprovalRecorded,
}

impl DispatchOutcome {
    /// Require that the call executed in this transaction and return its events.
    /// Errors if it was only a (threshold-≥2) approval that did not meet the
    /// threshold. Single-admin, threshold-1, and every genesis step can never be
    /// left pending, so they use this to assert execution rather than silently
    /// proceeding on an un-executed call.
    pub fn into_executed(self) -> Result<ExtrinsicEvents<PolkadotConfig>, WriteError> {
        match self {
            DispatchOutcome::Executed(events) => Ok(events),
            DispatchOutcome::ApprovalRecorded => Err(WriteError::Subxt(
                "multisig approval recorded but threshold not met; \
                 threshold-≥2 dispatch is not supported here"
                    .into(),
            )),
        }
    }
}

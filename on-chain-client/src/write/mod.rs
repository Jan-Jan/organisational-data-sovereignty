//! The chain writer (SDD-yg7n55): stands up an Organisation's slot and submits
//! each later update through its pure proxy. Compiled only with the `write`
//! feature (LLR-rxs5ec). Holds no key: the signatory key is an argument of
//! every call.

use core::fmt;

pub mod calldata;
pub mod ceremony;
pub mod events;
pub mod multisig;
pub mod proxy;
pub mod subxt_ops;

pub use ceremony::{genesis, submit_update, FUND_AMOUNT};

use crate::types::OrgAdmin;
use subxt::dynamic::Value;

/// A 32-byte chain account: a signatory, a co-signatory or a pure proxy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountId(pub [u8; 32]);

/// The genesis step that failed (LLR-z9vugt).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenesisStep {
    CreatePure,
    Fund,
    MapAccount,
    RecordGenesis,
}

/// Every way a write can fail; no outcome but an executed call is success.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteError {
    /// A genesis step did not execute.
    Step { step: GenesisStep, reason: String },
    /// A multisig approval was recorded but the update did not execute.
    PendingApproval,
    /// The extrinsic executed but the call it made through the proxy failed
    /// (`Proxy.ProxyExecuted` carried this dispatch error).
    InnerCallFailed(String),
    Subxt(String),
    EventNotFound(&'static str),
    MalformedEvent(&'static str),
    MalformedCall(&'static str),
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriteError::Step { step, reason } => write!(f, "genesis step {step:?} failed: {reason}"),
            WriteError::PendingApproval => f.write_str("multisig approval recorded; the update did not execute"),
            WriteError::InnerCallFailed(e) => write!(f, "the call made through the proxy failed: {e}"),
            WriteError::Subxt(m) => write!(f, "subxt error: {m}"),
            WriteError::EventNotFound(m) => write!(f, "expected on-chain event not found: {m}"),
            WriteError::MalformedEvent(m) => write!(f, "malformed event field: {m}"),
            WriteError::MalformedCall(m) => write!(f, "malformed call: {m}"),
        }
    }
}

impl std::error::Error for WriteError {}

/// What one dispatch under the controller did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use = "a dispatch may be a pending multisig approval, not an executed call"]
pub enum DispatchOutcome {
    Executed,
    ApprovalRecorded,
}

/// What genesis returns: the pure proxy and the Organisation admin derived
/// from it (`h160_of(proxy)`), which is the Organisation's on-chain id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Genesis {
    pub proxy: AccountId,
    pub admin: OrgAdmin,
}

/// The seam the composition is written against (SDD-yg7n55): create a pure
/// proxy, fund an account, dispatch a call under the controller.
/// `SubxtWriteOps` implements it over subxt; a gated test substitutes it.
#[allow(async_fn_in_trait)]
pub trait WriteOps {
    type Signer;
    async fn create_pure(&self, signatory: &Self::Signer, co_signatories: &[AccountId]) -> Result<AccountId, WriteError>;
    async fn fund(&self, signatory: &Self::Signer, dest: AccountId, amount: u128) -> Result<(), WriteError>;
    async fn dispatch(
        &self,
        signatory: &Self::Signer,
        co_signatories: &[AccountId],
        call: Value,
    ) -> Result<DispatchOutcome, WriteError>;
}

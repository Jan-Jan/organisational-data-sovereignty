//! ODS Phase 2 node logic. See docs/superpowers/specs/2026-06-15-ods-phase-2-poc-design.md.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod chain;
pub mod envelope;
pub mod error;
pub mod ids;
pub mod keys;
pub mod sequence;
pub mod types;
pub mod verify;

#[cfg(feature = "transport")]
pub mod transport;

#[cfg(feature = "chain")]
pub mod chain_read;
#[cfg(feature = "chain")]
pub mod chain_write;
#[cfg(feature = "chain")]
pub mod ceremony;

#[cfg(feature = "chain")]
pub use chain_read::OnChainReader;

#[cfg(feature = "app")]
pub mod blobs;
#[cfg(feature = "app")]
pub mod store;
#[cfg(feature = "app")]
pub mod service;
#[cfg(feature = "app")]
pub mod preflight;

#[cfg(feature = "app")]
pub use service::{ChainOps, MockChainOps, OrgService, ReceiveOutcome, SelfDeleteOutcome};
#[cfg(feature = "app")]
pub use service::SubxtChainOps;

// Deterministic fixtures for this crate's unit tests and, under the
// `test-support` feature (never enabled in production builds), for the
// integration tests in `tests/` that carry the traceability annotations.
#[cfg(any(test, feature = "test-support"))]
pub mod test_fixtures;

/// Test seam for crate-internal items that carry a low-level requirement.
///
/// `test_paths` in this unit's guardrails config reads `verifies:` annotations
/// only from `org-node/tests`, so an item whose evidence lives in a `src`
/// unit test cannot be traced. This module is the route by which such an item
/// reaches an integration test. It adds no behaviour, and it is never part of
/// a production build (`test-support` is not enabled by `app`).
#[cfg(all(feature = "test-support", feature = "chain"))]
pub mod test_support {
    use subxt::dynamic::Value;
    use subxt::ext::scale_value::Composite;
    use subxt::transactions::StaticPayload;

    use crate::chain_write::WriteError;
    use crate::types::ChainAccount;

    /// Wrapper over the crate-private `chain_write::multisig::build_dispatch_tx`,
    /// which carries LLR-f74xwb. A wrapper rather than a re-export because the
    /// function stays `pub(crate)`: the production surface is unchanged by this
    /// seam existing.
    pub fn build_dispatch_tx(
        other_signatories: &[ChainAccount],
        call: Value,
    ) -> Result<StaticPayload<Composite<()>>, WriteError> {
        crate::chain_write::multisig::build_dispatch_tx(other_signatories, call)
    }
}

pub use chain::{ChainReader, OrgState};
pub use envelope::SignedDeltaEnvelope;
pub use error::OrgNodeError;
pub use ids::OrgId;
pub use keys::SigningKeypair;
pub use sequence::SeqGuard;
pub use types::{ChainAccount, DeviceSeed, Epoch, MemberSeed, OrgPublicKey, OrgSecret, PersonaId, SequenceNumber};
pub use verify::{verify_envelope_against_chain, VerifyContext, VerifiedUpdate};

// org-members types that appear in org-node's public interface and that the
// app names, re-exported so the app depends on org-node alone for them.
pub use org_members::{MemberId, RootHash};

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
// The Wire message and its codec: values only, no feature gate.
pub mod wire;
// The notice and the acknowledgement the Wire message carries; the functions
// that read the store stay under `app`.
pub mod revocation;

#[cfg(feature = "app")]
pub mod store;
// The Invite and the Invite reply.
#[cfg(feature = "app")]
pub mod invitation;
#[cfg(feature = "app")]
pub mod service;
#[cfg(feature = "app")]
pub mod reconcile;

#[cfg(feature = "app")]
pub use service::{
    Checked, CheckedMessage, CommitOutcome, Joiner, OrgService, OutgoingMessage, ProvisionalTarget, ReceiveKind,
    ReceiveOutcome, StoreChange,
};
#[cfg(feature = "app")]
pub use store::{SealedStore, StoreWrite};

// Deterministic fixtures for this crate's unit tests and, under the
// `test-support` feature (never enabled in production builds), for the
// integration tests in `tests/` that carry the traceability annotations.
#[cfg(any(test, feature = "test-support"))]
pub mod test_fixtures;

pub use chain::OrgState;
pub use envelope::Envelope;
pub use error::OrgNodeError;
pub use ids::OrgId;
pub use keys::{DeviceSecret, SigningKeypair};
pub use sequence::SeqGuard;
pub use types::{
    ChainAccount, Epoch, MemberSeed, OrgPrivateKey, OrgPublicKey, PersonaId, SequenceNumber,
};
pub use verify::{verify_envelope_against_chain, VerifyContext, VerifiedUpdate};
pub use wire::WireMessage;

// org-members types that appear in org-node's public interface and that the
// app names, re-exported so the app depends on org-node alone for them.
pub use org_members::{DevicePublicKey, Handle, MemberId, Name, OrgMembersError, PersonPublicKey, RootHash, Surname};

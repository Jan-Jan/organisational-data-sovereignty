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
// The removed store form, kept after this node's own removal (SDD-z4m57w).
#[cfg(feature = "app")]
pub mod removal;
// The Invite and the endpoint addresses.
#[cfg(feature = "app")]
pub mod invitation;
#[cfg(feature = "app")]
pub mod service;
#[cfg(feature = "app")]
pub mod verify_org;
// The user's own Person, kept apart from every Persona (SDD-27p679).
#[cfg(feature = "app")]
pub mod person_store;

#[cfg(feature = "app")]
pub use invitation::EndpointAddresses;
#[cfg(feature = "app")]
pub use person_store::{PersonStore, PersonStoreContent, PersonView};
#[cfg(feature = "app")]
pub use service::{AdmissionRequestOutcome, Checked, KeptAcknowledgements, OrgService, OutgoingMessage, StoreChange};
#[cfg(feature = "app")]
pub use store::{
    KeptAdmissionRequest, Origin, OutstandingInvite, ProvisionalChange, ProvisionalFounding, ProvisionalMember,
    ProvisionalOrganisation, RevokedDevice, SealedStore, StoreWrite, MAX_PROVISIONAL_BYTES,
};
#[cfg(feature = "app")]
pub use verify_org::{AheadLimit, DiscardedChange, Inconsistency, VerificationTime, Verified};

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
pub use verify::VerifyContext;
pub use wire::WireMessage;

// org-members types that appear in org-node's public interface and that the
// app names, re-exported so the app depends on org-node alone for them.
pub use org_members::{DevicePublicKey, Handle, MemberId, Name, OrgMembersError, PersonPublicKey, RootHash, Surname};
// The Person value the Person store holds, so org-io names no `person::`
// path (LLR-5uf4u2).
pub use person::Person;

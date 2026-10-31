//! Test harness shared by org-io integration tests:
//! - `chopsticks_fork`, `chopsticks_reorg`, `conn`: minimal chopsticks spawn
//!   and subxt client helpers, the same as on-chain-client/tests/common/
//!   (which is not a public API).
//! - `handles`: handles over `FakeChain`, Personas, the one founding helper
//!   (`founded`) and transport helpers for the tests that drive `OrgIo`;
//!   needs `test-support`.
//! - `journey`: the demo journey's steps over the handle's own surface;
//!   needs `test-support`.
//! - `chain_writer_contract`: the founding assertions `FakeChain`'s writer
//!   and the production writer both meet; needs `test-support`.
//! - `chain_contract`: the read, window, update and signatory-set assertions
//!   `FakeChain` and the production halves both meet; needs `test-support`.

#![allow(dead_code)] // each integration test only uses a subset of helpers

#[cfg(feature = "test-support")]
pub mod chain_contract;
#[cfg(feature = "test-support")]
pub mod chain_writer_contract;
pub mod chopsticks_fork;
pub mod chopsticks_reorg;
pub mod conn;
#[cfg(feature = "test-support")]
pub mod handles;
#[cfg(feature = "test-support")]
pub mod journey;

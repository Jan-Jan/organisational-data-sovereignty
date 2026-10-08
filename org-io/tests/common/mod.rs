//! Test harness shared by org-io integration tests. Each file has a header
//! comment indicating its origin:
//! - `chopsticks_fork`, `chopsticks_reorg`, `conn`: minimal chopsticks spawn
//!   and subxt client helpers, duplicated from on-chain-client/tests/common/
//!   (which is not a public API); copied from org-node's with the preflight,
//!   2026-10-07.
//! - `handles`: handles over `FakeChain`, Personas and transport helpers for
//!   the tests that drive `OrgIo`; needs `test-support`.

#![allow(dead_code)] // each integration test only uses a subset of helpers

pub mod chopsticks_fork;
pub mod chopsticks_reorg;
pub mod conn;
#[cfg(feature = "test-support")]
pub mod handles;

//! Test harness shared by integration tests in `tests/`.
//!
//! What's here:
//!
//! - [`chopsticks_fork`]: spawn / teardown a chopsticks-Paseo fork from a
//!   Rust test, reusing `../on-chain/scripts/chopsticks-config.yml`.
//! - [`h160_mapper`]: pure `h160_of(account_id_32)` mirroring
//!   pallet-revive's mapping. Fixture-pinned (Risk #5 of the spec).
//! - [`chopsticks_reorg`]: `induce_reorg` driving chopsticks's dev_*
//!   JSON-RPC extensions.
//! - [`submit`]: subxt-based single-account submitter for `OrgRegistry.
//!   update(...)`, for the scenarios where an account acts as its own
//!   admin.

#![allow(dead_code)] // each integration test only uses a subset of helpers

pub mod chopsticks_fork;
pub mod chopsticks_reorg;
pub mod conn;
pub mod h160_mapper;
pub mod multisig;
pub mod proxy;
pub mod submit;

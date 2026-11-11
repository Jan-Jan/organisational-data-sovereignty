//! The contract of `WriteOps`'s two reads (TEST_GUIDELINES Doubles 3): the
//! same assertions, run against `write_compose.rs`'s substitute `FakeOps`
//! (gated) and against `SubxtWriteOps` on a chopsticks fork
//! (`write_genesis_e2e.rs`, outside every gate). Each states what the two
//! reads answer for an account in one of the chain states `found_at`
//! decides from (LLR-jk76wf, LLR-95puh9), always read at the account's own
//! H160 (`h160_of`), as `found_at` reads it.
//!
//! Included by `#[path]` from both targets; it is no test target itself.

use on_chain_client::write::{AccountId, WriteOps};
use on_chain_client::{h160_of, OrgAdmin};

/// The admin (H160) `found_at` reads an account's mapping at.
pub fn admin_of(account: AccountId) -> OrgAdmin {
    OrgAdmin(h160_of(account.0))
}

/// An account the chain does not hold (a pure proxy just created, nothing
/// sent to it): not mapped, and a free balance of zero.
pub async fn untouched<O: WriteOps>(ops: &O, account: AccountId) {
    assert_eq!(ops.mapped_account(admin_of(account)).await, Ok(None), "an untouched account is not mapped");
    assert_eq!(ops.free_balance(account).await, Ok(0), "an untouched account holds nothing");
}

/// An account funded with `amount` and not mapped: not mapped, and its free
/// balance is exactly `amount`.
pub async fn funded_unmapped<O: WriteOps>(ops: &O, account: AccountId, amount: u128) {
    assert_eq!(ops.mapped_account(admin_of(account)).await, Ok(None), "funding maps nothing");
    assert_eq!(ops.free_balance(account).await, Ok(amount), "the free balance is what was sent");
}

/// An account mapped in pallet-revive from its own origin: its H160 maps to
/// the account itself, and its free balance is at most `funded` (the
/// mapping's deposit is held from it).
pub async fn mapped_to_itself<O: WriteOps>(ops: &O, account: AccountId, funded: u128) {
    assert_eq!(ops.mapped_account(admin_of(account)).await, Ok(Some(account)), "the H160 maps to the account");
    let balance = ops.free_balance(account).await.expect("the balance read answers");
    assert!(balance <= funded, "mapping adds nothing to the free balance: {balance} > {funded}");
}

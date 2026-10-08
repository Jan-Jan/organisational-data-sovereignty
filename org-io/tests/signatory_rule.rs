#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The own-admin check (SDD-f4khqn): the pure rule, then the handle over FakeChain.

use std::sync::atomic::Ordering;

use on_chain_client::write::multisig::multi_account_id;
use on_chain_client::write::AccountId;
use org_io::node::OrgId;
use org_io::signatory::{admin_status, controller_of, AdminStatus, OwnAdminError};
use org_io::test_support::FakeChain;
use org_io::OrgIo;

mod common;
use common::handles::handle;

const OWN: AccountId = AccountId([1u8; 32]);
const CO_SIGNER: AccountId = AccountId([2u8; 32]);
const STRANGER: AccountId = AccountId([3u8; 32]);
const PROXY: AccountId = AccountId([9u8; 32]);

fn org() -> OrgId {
    OrgId::new([7u8; 20])
}

fn configured(tag: &str, chain: &FakeChain, co_signers: Vec<AccountId>) -> OrgIo {
    handle(tag, "a", chain).with_own_account(OWN, co_signers)
}

// verifies: LLR-qhyc3n
#[test]
fn the_controller_is_the_own_account_alone_or_the_threshold_one_multisig() {
    assert_eq!(controller_of(OWN, &[]), OWN);
    let expected = multi_account_id(&[OWN, CO_SIGNER], 1);
    assert_eq!(controller_of(OWN, &[CO_SIGNER]), expected);
    assert_eq!(controller_of(CO_SIGNER, &[OWN]), expected, "either order gives one controller");
}

// verifies: LLR-qhyc3n
#[test]
fn a_co_signer_equal_to_the_own_account_is_not_counted_twice() {
    let expected = multi_account_id(&[OWN, CO_SIGNER], 1);
    assert_eq!(controller_of(OWN, &[CO_SIGNER, OWN]), expected);
    assert_eq!(controller_of(OWN, &[OWN]), OWN, "only the own account remains: no multisig");
}

// verifies: LLR-vyd5d3
#[test]
fn admin_when_the_mapped_proxy_lists_the_controller() {
    let controller = controller_of(OWN, &[CO_SIGNER]);
    assert_eq!(admin_status(controller, Some(PROXY), &[STRANGER, controller]), AdminStatus::Admin);
    assert_eq!(admin_status(OWN, Some(PROXY), &[OWN]), AdminStatus::Admin, "no co-signers: the own account");
}

// verifies: LLR-vyd5d3
#[test]
fn not_admin_for_an_unmapped_h160_no_delegates_or_other_delegates() {
    let controller = controller_of(OWN, &[CO_SIGNER]);
    assert_eq!(admin_status(controller, None, &[controller]), AdminStatus::NotAdmin, "unmapped H160");
    assert_eq!(admin_status(controller, Some(PROXY), &[]), AdminStatus::NotAdmin, "no delegates");
    assert_eq!(admin_status(controller, Some(PROXY), &[STRANGER]), AdminStatus::NotAdmin);
    assert_eq!(
        admin_status(controller, Some(PROXY), &[OWN]),
        AdminStatus::NotAdmin,
        "with co-signers configured, the own account alone is not the controller"
    );
}

// verifies: LLR-c9fyun
#[tokio::test]
async fn a_node_holding_no_signatory_key_is_refused_with_not_configured() {
    let chain = FakeChain::new();
    chain.map_original_account(org(), PROXY);
    chain.set_delegates(PROXY, vec![OWN]);
    let io = handle("own-admin-no-key", "a", &chain);
    assert_eq!(io.is_own_admin(org()).await, Err(OwnAdminError::NotConfigured));
    assert_eq!(chain.signatory_reads(), 0, "nothing is read without a key");
}

// verifies: LLR-c9fyun
#[tokio::test]
async fn a_failed_signatory_read_is_an_error_never_not_admin() {
    let chain = FakeChain::new();
    chain.map_original_account(org(), PROXY);
    chain.set_delegates(PROXY, vec![OWN]);
    let io = configured("own-admin-read-fails", &chain, vec![]);

    chain.original_account_failing.store(true, Ordering::SeqCst);
    let answer = io.is_own_admin(org()).await;
    assert!(matches!(answer, Err(OwnAdminError::Read(ref reason)) if reason.contains("node unreachable")), "{answer:?}");

    chain.original_account_failing.store(false, Ordering::SeqCst);
    chain.delegates_failing.store(true, Ordering::SeqCst);
    let answer = io.is_own_admin(org()).await;
    assert!(matches!(answer, Err(OwnAdminError::Read(ref reason)) if reason.contains("node unreachable")), "{answer:?}");
}

// verifies: LLR-c9fyun
#[tokio::test]
async fn a_configured_account_listed_as_delegate_is_admin_and_delegates_are_read_only_for_a_mapped_h160() {
    let chain = FakeChain::new();
    let io = configured("own-admin-listed", &chain, vec![CO_SIGNER]);

    assert_eq!(io.is_own_admin(org()).await, Ok(AdminStatus::NotAdmin), "unmapped H160");
    assert_eq!(chain.signatory_reads(), 1, "no delegates read for an unmapped H160");

    chain.map_original_account(org(), PROXY);
    chain.set_delegates(PROXY, vec![STRANGER, multi_account_id(&[OWN, CO_SIGNER], 1)]);
    assert_eq!(io.is_own_admin(org()).await, Ok(AdminStatus::Admin));
    assert_eq!(chain.signatory_reads(), 3, "the mapped account, then its delegates");

    let alone = configured("own-admin-alone", &chain, vec![]);
    assert_eq!(alone.is_own_admin(org()).await, Ok(AdminStatus::NotAdmin), "the own account alone is not listed");
}

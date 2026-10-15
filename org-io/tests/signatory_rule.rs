#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The own-admin check (SDD-f4khqn): the pure rule, then the handle over FakeChain.

use std::sync::atomic::Ordering;

use on_chain_client::write::multisig::multi_account_id;
use on_chain_client::write::AccountId;
use org_node::OrgId;
use org_io::signatory::{admin_status, controller_of, AdminStatus, OwnAdminError, ProxyDefinition, SignatorySetView};
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

/// The runtime's `Any` index in the sets below, unless a test says otherwise.
const ANY: u8 = 0;
/// Another proxy type's index (a runtime's `NonTransfer`, say).
const RESTRICTED: u8 = 3;

fn definition(delegate: AccountId, proxy_type: u8, delay: u32) -> ProxyDefinition {
    ProxyDefinition { delegate, proxy_type, delay }
}

fn mapped(definitions: Vec<ProxyDefinition>) -> SignatorySetView {
    SignatorySetView { original_account: Some(PROXY), definitions, any_proxy_type: ANY }
}

// PR-wk7zwq's reproduction: a controller named only through a restricted
// type or a delay cannot have its writes dispatched at once, so it is not an
// admin but `DelegateCannotDispatch`. Before: `Admin` for any definition
// naming the controller, whatever its type and delay.
// verifies: LLR-2hkqv2, LLR-vyd5d3, PR-wk7zwq
#[tokio::test]
async fn a_restricted_or_delayed_delegate_cannot_dispatch() {
    let controller = controller_of(OWN, &[CO_SIGNER]);
    let restricted = mapped(vec![definition(controller, RESTRICTED, 0)]);
    assert_eq!(admin_status(controller, &restricted), AdminStatus::DelegateCannotDispatch);
    let delayed = mapped(vec![definition(controller, ANY, 10)]);
    assert_eq!(admin_status(controller, &delayed), AdminStatus::DelegateCannotDispatch);
    let both = mapped(vec![definition(controller, RESTRICTED, 0), definition(controller, ANY, 1)]);
    assert_eq!(admin_status(controller, &both), AdminStatus::DelegateCannotDispatch);

    let chain = FakeChain::new();
    chain.map_original_account(org(), PROXY);
    chain.set_definitions(PROXY, vec![definition(controller, RESTRICTED, 0), definition(STRANGER, ANY, 0)]);
    let io = configured("own-admin-restricted", &chain, vec![CO_SIGNER]);
    assert_eq!(io.is_own_admin(org()).await, Ok(AdminStatus::DelegateCannotDispatch));
}

// Normal: `Admin` exactly when some definition is `(controller, Any, 0)`,
// wherever it sorts — the writer forces `Any`, so a restricted definition
// sorted first does not hide it. Abnormal: no definitions, only other
// delegates, and the own account while co-signers are configured are
// `NotAdmin`.
// verifies: LLR-2hkqv2, LLR-vyd5d3
#[test]
fn only_controller_any_zero_is_admin() {
    let controller = controller_of(OWN, &[CO_SIGNER]);
    let sorted_after = mapped(vec![definition(controller, RESTRICTED, 0), definition(controller, ANY, 0)]);
    assert_eq!(admin_status(controller, &sorted_after), AdminStatus::Admin);
    assert_eq!(admin_status(OWN, &mapped(vec![definition(OWN, ANY, 0)])), AdminStatus::Admin, "no co-signers");

    assert_eq!(admin_status(controller, &mapped(vec![])), AdminStatus::NotAdmin, "no definitions");
    let others = mapped(vec![definition(STRANGER, ANY, 0), definition(STRANGER, RESTRICTED, 5)]);
    assert_eq!(admin_status(controller, &others), AdminStatus::NotAdmin, "only other delegates");
    assert_eq!(
        admin_status(controller, &mapped(vec![definition(OWN, ANY, 0)])),
        AdminStatus::NotAdmin,
        "with co-signers configured, the own account alone is not the controller"
    );
}

// Abnormal: an H160 that maps to no account is `NotAdmin`, even with
// definitions that would otherwise make the controller an admin or a
// delegate that cannot dispatch.
// verifies: LLR-2hkqv2, LLR-vyd5d3
#[test]
fn an_unmapped_account_is_not_admin_whatever_the_definitions() {
    let controller = controller_of(OWN, &[]);
    for definitions in [vec![definition(controller, ANY, 0)], vec![definition(controller, RESTRICTED, 0)], vec![]] {
        let unmapped = SignatorySetView { original_account: None, definitions, any_proxy_type: ANY };
        assert_eq!(admin_status(controller, &unmapped), AdminStatus::NotAdmin, "{unmapped:?}");
    }
}

// The `Any` index is the one the read resolved from the runtime's metadata,
// never a constant: where the runtime's `Any` is variant 3, a type-0
// definition is not `Admin` and a type-3 one is.
// verifies: LLR-2hkqv2, LLR-vyd5d3
#[tokio::test]
async fn the_any_index_is_the_reads_not_a_constant() {
    let controller = controller_of(OWN, &[]);
    let set = |proxy_type| SignatorySetView {
        original_account: Some(PROXY),
        definitions: vec![definition(controller, proxy_type, 0)],
        any_proxy_type: 3,
    };
    assert_eq!(admin_status(controller, &set(0)), AdminStatus::DelegateCannotDispatch);
    assert_eq!(admin_status(controller, &set(3)), AdminStatus::Admin);

    let chain = FakeChain::new();
    chain.set_any_proxy_type(3);
    chain.map_original_account(org(), PROXY);
    chain.set_definitions(PROXY, vec![definition(OWN, 0, 0)]);
    let io = configured("own-admin-any-index", &chain, vec![]);
    assert_eq!(io.is_own_admin(org()).await, Ok(AdminStatus::DelegateCannotDispatch));
    chain.set_definitions(PROXY, vec![definition(OWN, 3, 0)]);
    assert_eq!(io.is_own_admin(org()).await, Ok(AdminStatus::Admin));
}

// Abnormal: the one signatory-set read failing — at the mapped account or at
// the definitions — is `OwnAdminError::Read`, never an answer; restored, the
// same handle answers.
// verifies: LLR-2hkqv2
#[tokio::test]
async fn a_failed_signatory_set_read_is_an_error_never_not_admin() {
    let chain = FakeChain::new();
    chain.map_original_account(org(), PROXY);
    chain.set_definitions(PROXY, vec![definition(OWN, ANY, 0)]);
    let io = configured("own-admin-set-read-fails", &chain, vec![]);

    for switch in [&chain.original_account_failing, &chain.delegates_failing] {
        switch.store(true, Ordering::SeqCst);
        let answer = io.is_own_admin(org()).await;
        assert!(matches!(answer, Err(OwnAdminError::Read(ref reason)) if reason.contains("node unreachable")), "{answer:?}");
        switch.store(false, Ordering::SeqCst);
    }
    assert_eq!(io.is_own_admin(org()).await, Ok(AdminStatus::Admin));
}

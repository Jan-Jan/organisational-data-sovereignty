#![cfg(all(feature = "test-support", feature = "write"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! What a finalised dispatch did, read from its events (LLR-24mwew):
//! pallet-proxy's `Proxy.proxy` succeeds as an extrinsic when its inner call
//! fails, so the inner result in `Proxy.ProxyExecuted` decides.

use on_chain_client::write::events::{dispatch_result, observed_event, outcome_of, ObservedEvent};
use on_chain_client::write::proxy::{is_proxied, map_account_call, proxied};
use on_chain_client::write::{AccountId, DispatchOutcome, WriteError};
use std::cell::Cell;
use subxt::dynamic::Value;
use subxt::ext::scale_value::Composite;

const PROXY: AccountId = AccountId([0x5a; 32]);

/// `ProxyExecuted`'s fields as subxt decodes them: `{ result: <value> }`.
fn proxy_executed_fields(result: Value) -> Composite<()> {
    Composite::named(vec![("result".to_string(), result)])
}

/// A `DispatchError::Module` as pallet-revive's revert arrives.
fn module_error() -> Value {
    Value::variant(
        "Module",
        Composite::named(vec![
            ("index".to_string(), Value::u128(60)),
            ("error".to_string(), Value::unnamed_composite(vec![Value::u128(3), Value::u128(0)])),
        ]),
    )
}

// verifies: LLR-24mwew, REQ-aat4yt
#[test]
fn a_proxy_executed_result_of_ok_decodes_to_ok() {
    let fields = proxy_executed_fields(Value::variant("Ok", Composite::unnamed(vec![Value::unnamed_composite(vec![])])));
    assert_eq!(dispatch_result(&fields), Ok(Ok(())));
}

// verifies: LLR-24mwew, REQ-aat4yt, REQ-6jefu2
#[test]
fn a_proxy_executed_result_of_err_decodes_to_err_carrying_the_error_text() {
    let fields = proxy_executed_fields(Value::variant("Err", Composite::unnamed(vec![module_error()])));
    let text = match dispatch_result(&fields) {
        Ok(Err(text)) => text,
        other => panic!("expected an inner error, got {other:?}"),
    };
    assert!(text.contains("Module"), "{text}");
    assert!(text.contains("60"), "{text}");
}

// verifies: LLR-24mwew, REQ-aat4yt
#[test]
fn a_proxy_executed_without_a_result_field_is_a_typed_decode_failure() {
    let fields = Composite::named(vec![("other".to_string(), Value::variant("Ok", Composite::unnamed(vec![])))]);
    assert_eq!(dispatch_result(&fields), Err(WriteError::MalformedEvent("ProxyExecuted has no result")));
    assert_eq!(
        dispatch_result(&Composite::unnamed(vec![])),
        Err(WriteError::MalformedEvent("ProxyExecuted has no result"))
    );
}

// verifies: LLR-24mwew, REQ-aat4yt
#[test]
fn a_proxy_executed_result_that_is_not_ok_or_err_is_a_typed_decode_failure() {
    assert_eq!(
        dispatch_result(&proxy_executed_fields(Value::u128(0))),
        Err(WriteError::MalformedEvent("ProxyExecuted result is not a variant"))
    );
    assert_eq!(
        dispatch_result(&proxy_executed_fields(Value::variant("Pending", Composite::unnamed(vec![])))),
        Err(WriteError::MalformedEvent("ProxyExecuted result is neither Ok nor Err"))
    );
}

// verifies: LLR-24mwew, REQ-aat4yt
#[test]
fn observed_event_decodes_proxy_executed_and_ignores_the_fields_of_other_events() {
    let err = proxy_executed_fields(Value::variant("Err", Composite::unnamed(vec![module_error()])));
    let read = Cell::new(false);
    assert!(matches!(
        observed_event("Proxy", "ProxyExecuted", fields_reader(Ok(err), &read)),
        Ok(ObservedEvent::ProxyExecuted(Err(_)))
    ));
    assert!(read.take(), "ProxyExecuted's fields are decoded");
    assert_eq!(
        observed_event("Proxy", "ProxyExecuted", fields_reader(Err(WriteError::Subxt("decode fields".into())), &read)),
        Err(WriteError::Subxt("decode fields".into()))
    );
    assert!(read.take(), "ProxyExecuted's fields are decoded");
    let unused = proxy_executed_fields(Value::variant("Ok", Composite::unnamed(vec![])));
    assert_eq!(observed_event("Proxy", "PureCreated", fields_reader(Ok(unused.clone()), &read)), Ok(ObservedEvent::Other));
    assert!(!read.get(), "an event other than ProxyExecuted is not decoded");
    assert_eq!(observed_event("System", "ProxyExecuted", fields_reader(Ok(unused), &read)), Ok(ObservedEvent::Other));
    assert!(!read.get(), "an event other than ProxyExecuted is not decoded");
}

/// The one closure type every `observed_event` call in this file passes, so
/// they share one instantiation (llvm-cov counts each apart): it yields
/// `fields` and records in `read` that it was called.
fn fields_reader(
    fields: Result<Composite<()>, WriteError>,
    read: &Cell<bool>,
) -> impl FnOnce() -> Result<Composite<()>, WriteError> + '_ {
    move || {
        read.set(true);
        fields
    }
}

// verifies: LLR-24mwew, REQ-aat4yt, REQ-6jefu2
#[test]
fn a_proxied_call_whose_inner_result_is_ok_executed() {
    let events = [ObservedEvent::Other, ObservedEvent::ProxyExecuted(Ok(())), ObservedEvent::Other];
    assert_eq!(outcome_of(true, &events), Ok(DispatchOutcome::Executed));
}

// verifies: LLR-24mwew, REQ-aat4yt, REQ-6jefu2
#[test]
fn a_proxied_call_whose_inner_result_is_an_error_is_a_typed_failure() {
    let err = "Module { index: 60, error: [3, 0, 0, 0] }".to_string();
    let events = [ObservedEvent::ProxyExecuted(Err(err.clone())), ObservedEvent::Other];
    assert_eq!(outcome_of(true, &events), Err(WriteError::InnerCallFailed(err)));
}

// verifies: LLR-24mwew, REQ-aat4yt
#[test]
fn any_inner_error_among_several_proxy_executed_events_is_a_failure() {
    let events = [ObservedEvent::ProxyExecuted(Ok(())), ObservedEvent::ProxyExecuted(Err("BadOrigin".into()))];
    assert_eq!(outcome_of(true, &events), Err(WriteError::InnerCallFailed("BadOrigin".into())));
}

// verifies: LLR-24mwew, REQ-aat4yt, REQ-6jefu2
#[test]
fn a_proxied_call_without_proxy_executed_is_a_failure() {
    assert_eq!(
        outcome_of(true, &[ObservedEvent::Other]),
        Err(WriteError::EventNotFound("Proxy.ProxyExecuted"))
    );
    assert_eq!(outcome_of(true, &[]), Err(WriteError::EventNotFound("Proxy.ProxyExecuted")));
}

// verifies: LLR-24mwew
#[test]
fn a_call_not_made_through_the_proxy_executed_on_a_successful_extrinsic() {
    assert_eq!(outcome_of(false, &[ObservedEvent::Other]), Ok(DispatchOutcome::Executed));
    assert_eq!(outcome_of(false, &[]), Ok(DispatchOutcome::Executed));
}

// verifies: LLR-24mwew
#[test]
fn is_proxied_is_true_only_for_proxy_proxy() {
    assert!(is_proxied(&proxied(PROXY, map_account_call())));
    assert!(!is_proxied(&map_account_call()));
    assert!(!is_proxied(&on_chain_client::write::proxy::create_pure_call()));
}

// verifies: LLR-24mwew
#[test]
fn is_proxied_is_false_for_a_call_that_is_not_a_variant() {
    assert!(!is_proxied(&Value::u128(0)));
    assert!(!is_proxied(&Value::unnamed_composite(vec![Value::variant("proxy", Composite::unnamed(vec![]))])));
}

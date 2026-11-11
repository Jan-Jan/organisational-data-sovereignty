//! What a finalised dispatch did, decided from its events (LLR-24mwew).

use subxt::ext::scale_value::{Composite, ValueDef};

use super::{DispatchOutcome, WriteError};

/// One event as the writer reads it: `Proxy.ProxyExecuted` with its decoded
/// inner result; any other event is `Other` and its fields are not decoded.
/// `fields` decodes the event's fields; its error is returned unchanged.
pub fn observed_event(
    pallet: &str,
    event: &str,
    fields: impl FnOnce() -> Result<Composite<()>, WriteError>,
) -> Result<ObservedEvent, WriteError> {
    if pallet == "Proxy" && event == "ProxyExecuted" {
        Ok(ObservedEvent::ProxyExecuted(dispatch_result(&fields()?)?))
    } else {
        Ok(ObservedEvent::Other)
    }
}

/// `ProxyExecuted { result: DispatchResult }`'s fields to the inner call's
/// result: `Ok(())`, or the error rendered as text. A missing `result`, or a
/// `result` that is not an `Ok`/`Err` variant, is `WriteError::MalformedEvent`.
pub fn dispatch_result(fields: &Composite<()>) -> Result<Result<(), String>, WriteError> {
    let result = match fields {
        Composite::Named(named) => named.iter().find(|(n, _)| n == "result").map(|(_, v)| v),
        Composite::Unnamed(unnamed) => unnamed.first(),
    }
    .ok_or(WriteError::MalformedEvent("ProxyExecuted has no result"))?;
    let ValueDef::Variant(variant) = &result.value else {
        return Err(WriteError::MalformedEvent("ProxyExecuted result is not a variant"));
    };
    match variant.name.as_str() {
        "Ok" => Ok(Ok(())),
        "Err" => Ok(Err(variant.values.values().map(|v| v.to_string()).collect::<Vec<_>>().join(", "))),
        _ => Err(WriteError::MalformedEvent("ProxyExecuted result is neither Ok nor Err")),
    }
}

/// A dispatched extrinsic's event, as much of it as the writer reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedEvent {
    /// `Proxy.ProxyExecuted`: the inner call's result, the error as text.
    ProxyExecuted(Result<(), String>),
    Other,
}

/// What a finalised, successful dispatch did. A proxied call executed only
/// when its events carry a `ProxyExecuted` and none carries an error; a call
/// not made through the proxy executed when its extrinsic succeeded.
pub fn outcome_of(proxied: bool, events: &[ObservedEvent]) -> Result<DispatchOutcome, WriteError> {
    if !proxied {
        return Ok(DispatchOutcome::Executed);
    }
    let mut seen = false;
    for event in events {
        match event {
            ObservedEvent::ProxyExecuted(Err(e)) => return Err(WriteError::InnerCallFailed(e.clone())),
            ObservedEvent::ProxyExecuted(Ok(())) => seen = true,
            ObservedEvent::Other => {}
        }
    }
    if seen {
        Ok(DispatchOutcome::Executed)
    } else {
        Err(WriteError::EventNotFound("Proxy.ProxyExecuted"))
    }
}

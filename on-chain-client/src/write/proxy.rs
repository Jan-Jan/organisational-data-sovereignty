//! The calls a pure proxy is created, mapped and acted through (LLR-ywhd23).

use subxt::dynamic::Value;
use subxt::ext::scale_value::{Composite, ValueDef};

use super::AccountId;

/// `Proxy.create_pure { proxy_type: Any, delay: 0, index: 0 }`.
pub fn create_pure_call() -> Value {
    Value::variant(
        "Proxy",
        Composite::unnamed(vec![Value::variant(
            "create_pure",
            Composite::named(vec![
                ("proxy_type".to_string(), Value::variant("Any", Composite::unnamed(vec![]))),
                ("delay".to_string(), Value::u128(0)),
                ("index".to_string(), Value::u128(0)),
            ]),
        )]),
    )
}

/// `Proxy.proxy { real: Id(proxy), force_proxy_type: None, call }`.
pub fn proxied(proxy: AccountId, call: Value) -> Value {
    Value::variant(
        "Proxy",
        Composite::unnamed(vec![Value::variant(
            "proxy",
            Composite::named(vec![
                ("real".to_string(), Value::variant("Id", Composite::unnamed(vec![Value::from_bytes(proxy.0.as_slice())]))),
                ("force_proxy_type".to_string(), Value::variant("None", Composite::unnamed(vec![]))),
                ("call".to_string(), call),
            ]),
        )]),
    )
}

/// Whether `call` is `Proxy.proxy`, i.e. a call made through the proxy whose
/// inner result only `Proxy.ProxyExecuted` reports (LLR-24mwew).
pub fn is_proxied(call: &Value) -> bool {
    let ValueDef::Variant(pallet) = &call.value else {
        return false;
    };
    if pallet.name != "Proxy" {
        return false;
    }
    matches!(
        pallet.values.values().next().map(|inner| &inner.value),
        Some(ValueDef::Variant(c)) if c.name == "proxy"
    )
}

/// `Revive.map_account {}`, dispatched once as the proxy before it can call
/// the contract.
pub fn map_account_call() -> Value {
    Value::variant("Revive", Composite::unnamed(vec![Value::variant("map_account", Composite::unnamed(vec![]))]))
}

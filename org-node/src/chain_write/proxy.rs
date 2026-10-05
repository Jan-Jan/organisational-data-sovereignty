//! Pure-proxy creation and call-wrapping. Decoupled from chopsticks via BlockSink.
//! Lifted verbatim from on-chain-client/tests/common/proxy.rs; errors retyped to
//! WriteError; &ChopsticksHandle + mine_block replaced with sink: &dyn BlockSink +
//! sink.settle().await?; .unwrap()/.expect() replaced with ? / WriteError.
#![cfg(feature = "chain")]

use subxt::OnlineClient;
use subxt::config::PolkadotConfig;
use subxt::dynamic::Value;
use subxt::ext::scale_value::{Composite, Primitive, ValueDef};
use subxt_signer::sr25519::Keypair;

use crate::chain_write::WriteError;
use crate::chain_write::multisig::dispatch_org_call;
use crate::ids::OrgId;
use crate::types::ChainAccount;

/// Abstraction over "make the chain advance so a just-submitted extrinsic is
/// observable". Chopsticks tests implement this by calling dev_newBlock; a live
/// chain implementation waits for finalisation. Keeps the write path agnostic.
#[async_trait::async_trait]
pub trait BlockSink: Send + Sync {
    /// Advance/settle the chain so the just-submitted extrinsic is included,
    /// and return the hash of the block that includes it. (Chopsticks: the
    /// dev_newBlock hash; live chain: the best block the tx landed in.)
    async fn settle(&self) -> Result<[u8; 32], WriteError>;
}

/// `RuntimeCall::Proxy(Call::create_pure { proxy_type: Any, delay: 0,
/// index: 0 })` as a dynamic value.
fn create_pure_call() -> Value {
    Value::variant(
        "Proxy",
        Composite::unnamed(vec![Value::variant(
            "create_pure",
            Composite::named(vec![
                (
                    "proxy_type".to_string(),
                    Value::variant("Any", Composite::unnamed(vec![])),
                ),
                ("delay".to_string(), Value::u128(0)),
                ("index".to_string(), Value::u128(0)),
            ]),
        )]),
    )
}

/// Wrap `call` so it executes with `pure_proxy` as origin:
/// `RuntimeCall::Proxy(Call::proxy { real: Id(P), force_proxy_type:
/// None, call })`.
pub fn proxied(pure_proxy: ChainAccount, call: Value) -> Value {
    Value::variant(
        "Proxy",
        Composite::unnamed(vec![Value::variant(
            "proxy",
            Composite::named(vec![
                (
                    "real".to_string(),
                    Value::variant(
                        "Id",
                        Composite::unnamed(vec![Value::from_bytes(pure_proxy.as_bytes().as_slice())]),
                    ),
                ),
                (
                    "force_proxy_type".to_string(),
                    Value::variant("None", Composite::unnamed(vec![])),
                ),
                ("call".to_string(), call),
            ]),
        )]),
    )
}

/// `RuntimeCall::Revive(Call::map_account {})` as a dynamic value.
///
/// pallet-revive keeps an `AddressMapper` table linking each substrate
/// `AccountId32` to the H160 it acts as inside the EVM. A fresh pure proxy
/// must dispatch `Revive.map_account` (which takes no arguments) once after
/// being funded, AS ITSELF, before it can submit `OrgRegistry.update`.
pub fn map_account_call() -> Value {
    Value::variant(
        "Revive",
        Composite::unnamed(vec![Value::variant(
            "map_account",
            Composite::unnamed(vec![]),
        )]),
    )
}

fn add_proxy_call(delegate: ChainAccount) -> Value {
    Value::variant(
        "Proxy",
        Composite::unnamed(vec![Value::variant(
            "add_proxy",
            Composite::named(vec![
                (
                    "delegate".to_string(),
                    Value::variant(
                        "Id",
                        Composite::unnamed(vec![Value::from_bytes(delegate.as_bytes().as_slice())]),
                    ),
                ),
                (
                    "proxy_type".to_string(),
                    Value::variant("Any", Composite::unnamed(vec![])),
                ),
                ("delay".to_string(), Value::u128(0)),
            ]),
        )]),
    )
}

fn remove_proxy_call(delegate: ChainAccount) -> Value {
    Value::variant(
        "Proxy",
        Composite::unnamed(vec![Value::variant(
            "remove_proxy",
            Composite::named(vec![
                (
                    "delegate".to_string(),
                    Value::variant(
                        "Id",
                        Composite::unnamed(vec![Value::from_bytes(delegate.as_bytes().as_slice())]),
                    ),
                ),
                (
                    "proxy_type".to_string(),
                    Value::variant("Any", Composite::unnamed(vec![])),
                ),
                ("delay".to_string(), Value::u128(0)),
            ]),
        )]),
    )
}

/// Create the org's pure proxy `P` under the current controller and return `P`.
/// Single admin (`others` empty): the admin signs `Proxy.create_pure` directly
/// and becomes `P`'s delegate. Threshold-1 multisig: the multisig creates `P`.
/// `P` is read from the `Proxy.PureCreated` event of the executed extrinsic.
pub async fn create_pure(
    sink: &dyn BlockSink,
    api: &OnlineClient<PolkadotConfig>,
    signer: &Keypair,
    others: &[ChainAccount],
) -> Result<ChainAccount, WriteError> {
    // create_pure must EXECUTE to emit PureCreated; a (threshold-≥2) pending
    // approval has no proxy to return, so `into_executed` rejects it.
    let events = dispatch_org_call(sink, api, signer, others, create_pure_call())
        .await?
        .into_executed()?;
    for ev in events.iter() {
        let ev = ev.map_err(|e| WriteError::Subxt(format!("event iter: {e}")))?;
        if ev.pallet_name() == "Proxy" && ev.event_name() == "PureCreated" {
            let fields: Composite<()> = ev
                .decode_fields_unchecked_as()
                .map_err(|e| WriteError::Subxt(format!("decode fields: {e}")))?;
            return account32_from_named_field(&fields, "pure");
        }
    }
    Err(WriteError::EventNotFound("Proxy.PureCreated"))
}

/// Rotate P's controlling multisig from OLD (signer_old + others_old) to
/// NEW (the 32-byte multisig account `new_multi`). Two proxied calls,
/// each in its own block: add_proxy(new) then remove_proxy(old_multi).
pub async fn rotate(
    sink: &dyn BlockSink,
    api: &OnlineClient<PolkadotConfig>,
    pure_proxy: ChainAccount,
    signer_old: &Keypair,
    others_old: &[ChainAccount],
    old_multi: ChainAccount,
    new_multi: ChainAccount,
) -> Result<(), WriteError> {
    dispatch_org_call(sink, api, signer_old, others_old, proxied(pure_proxy, add_proxy_call(new_multi)))
        .await?
        .into_executed()?;
    dispatch_org_call(sink, api, signer_old, others_old, proxied(pure_proxy, remove_proxy_call(old_multi)))
        .await?
        .into_executed()?;
    Ok(())
}

/// The OrgId of the Organisation whose pure proxy is `p`: `h160_of(P)`.
pub fn org_id_of(p: ChainAccount) -> OrgId {
    OrgId::new(on_chain_client::h160_of(*p.as_bytes()))
}

/// Pull a 32-byte AccountId out of a named event field. The dynamic
/// Value for an AccountId32 is a composite wrapping 32 u8 primitives
/// (possibly nested one level — newtype). Handles both shapes.
fn account32_from_named_field(
    fields: &Composite<()>,
    name: &str,
) -> Result<ChainAccount, WriteError> {
    let Composite::Named(named) = fields else {
        return Err(WriteError::MalformedEvent("event fields not named"));
    };
    let (_, value) = named
        .iter()
        .find(|(n, _)| n == name)
        .ok_or(WriteError::EventNotFound("named field not found"))?;
    collect_account32(value)
        .map(ChainAccount::new)
        .ok_or(WriteError::MalformedEvent("field is not a 32-byte account"))
}

fn collect_account32(value: &Value<()>) -> Option<[u8; 32]> {
    match &value.value {
        ValueDef::Composite(c) => {
            let inner: Vec<&Value<()>> = match c {
                Composite::Named(n) => n.iter().map(|(_, v)| v).collect(),
                Composite::Unnamed(u) => u.iter().collect(),
            };
            if inner.len() == 1 {
                return collect_account32(inner[0]);
            }
            if inner.len() == 32 {
                let mut out = [0u8; 32];
                for (i, v) in inner.iter().enumerate() {
                    match &v.value {
                        ValueDef::Primitive(Primitive::U128(b)) if *b <= 255 => {
                            out[i] = *b as u8;
                        }
                        _ => return None,
                    }
                }
                return Some(out);
            }
            None
        }
        _ => None,
    }
}

//! `WriteOps` over subxt, and the sinks that drive or await block production.

use futures_util::future::{self, Either};
use subxt::OnlineClient;
use subxt::config::PolkadotConfig;
use subxt::dynamic::{self, Value};
use subxt::ext::scale_value::{Composite, Primitive, ValueDef};
use subxt::extrinsics::ExtrinsicEvents;
use subxt_signer::sr25519::Keypair;

use super::events::{observed_event, outcome_of, ObservedEvent};
use super::multisig::build_dispatch_tx;
use super::proxy::{create_pure_call, is_proxied};
use super::{AccountId, DispatchOutcome, WriteError, WriteOps};

/// Makes the chain advance while a submitted extrinsic waits for finality.
#[allow(async_fn_in_trait)]
pub trait BlockSink {
    async fn settle(&self) -> Result<(), WriteError>;
}

/// The live-chain sink: a live chain finalises on its own, so it never
/// resolves and the extrinsic's own finality is the only completion signal.
pub struct FinalitySink;

impl BlockSink for FinalitySink {
    async fn settle(&self) -> Result<(), WriteError> {
        future::pending::<()>().await;
        Ok(())
    }
}

pub struct SubxtWriteOps<S: BlockSink> {
    api: OnlineClient<PolkadotConfig>,
    sink: S,
}

impl<S: BlockSink> SubxtWriteOps<S> {
    pub fn new(api: OnlineClient<PolkadotConfig>, sink: S) -> Self {
        Self { api, sink }
    }

    /// Sign and submit `tx`, settling blocks until it is finalised; an
    /// `ExtrinsicFailed` is an error.
    async fn submit_and_watch<Call: subxt::transactions::Payload>(
        &self,
        signer: &Keypair,
        tx: &Call,
    ) -> Result<ExtrinsicEvents<PolkadotConfig>, WriteError> {
        let progress = self
            .api
            .tx()
            .await
            .map_err(|e| WriteError::Subxt(format!("tx_client: {e}")))?
            .sign_and_submit_then_watch_default(tx, signer)
            .await
            .map_err(|e| WriteError::Subxt(format!("submit: {e}")))?;
        let fin = progress.wait_for_finalized_success();
        futures_util::pin_mut!(fin);
        loop {
            let settle = self.sink.settle();
            futures_util::pin_mut!(settle);
            match future::select(fin.as_mut(), settle).await {
                Either::Left((res, _)) => {
                    return res.map_err(|e| WriteError::Subxt(format!("extrinsic dispatch failed: {e}")));
                }
                Either::Right((settled, _)) => settled?,
            }
        }
    }
}

impl<S: BlockSink> WriteOps for SubxtWriteOps<S> {
    type Signer = Keypair;

    async fn create_pure(&self, signatory: &Keypair, co_signatories: &[AccountId]) -> Result<AccountId, WriteError> {
        let tx = build_dispatch_tx(co_signatories, create_pure_call())?.into_payload();
        let events = self.submit_and_watch(signatory, &tx).await?;
        for ev in events.iter() {
            let ev = ev.map_err(|e| WriteError::Subxt(format!("event iter: {e}")))?;
            if ev.pallet_name() == "Proxy" && ev.event_name() == "PureCreated" {
                let fields: Composite<()> =
                    ev.decode_fields_unchecked_as().map_err(|e| WriteError::Subxt(format!("decode fields: {e}")))?;
                return account32_from_named_field(&fields, "pure");
            }
        }
        Err(WriteError::EventNotFound("Proxy.PureCreated"))
    }

    async fn fund(&self, signatory: &Keypair, dest: AccountId, amount: u128) -> Result<(), WriteError> {
        let dest = Value::variant("Id", Composite::unnamed(vec![Value::from_bytes(dest.0.as_slice())]));
        let tx = dynamic::tx("Balances", "transfer_keep_alive", vec![dest, Value::u128(amount)]);
        self.submit_and_watch(signatory, &tx).await.map(|_| ())
    }

    async fn dispatch(
        &self,
        signatory: &Keypair,
        co_signatories: &[AccountId],
        call: Value,
    ) -> Result<DispatchOutcome, WriteError> {
        let proxied = is_proxied(&call);
        let tx = build_dispatch_tx(co_signatories, call)?.into_payload();
        let events = self.submit_and_watch(signatory, &tx).await?;
        outcome_of(proxied, &observe(&events)?)
    }
}

/// The extrinsic's events as the writer reads them (LLR-24mwew's input); the
/// decoding is `observed_event`'s, this only fetches.
fn observe(events: &ExtrinsicEvents<PolkadotConfig>) -> Result<Vec<ObservedEvent>, WriteError> {
    events
        .iter()
        .map(|ev| {
            let ev = ev.map_err(|e| WriteError::Subxt(format!("event iter: {e}")))?;
            observed_event(ev.pallet_name(), ev.event_name(), || {
                ev.decode_fields_unchecked_as().map_err(|e| WriteError::Subxt(format!("decode fields: {e}")))
            })
        })
        .collect()
}

fn account32_from_named_field(fields: &Composite<()>, name: &str) -> Result<AccountId, WriteError> {
    let Composite::Named(named) = fields else {
        return Err(WriteError::MalformedEvent("event fields not named"));
    };
    let (_, value) = named
        .iter()
        .find(|(n, _)| n == name)
        .ok_or(WriteError::EventNotFound("named field not found"))?;
    collect_account32(value)
        .map(AccountId)
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

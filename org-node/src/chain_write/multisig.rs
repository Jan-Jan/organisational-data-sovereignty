//! Threshold-1 multisig: pseudo-account derivation + dispatch + funding.
//! `multi_account_id` mirrors pallet_multisig::Pallet::multi_account_id.
//! Lifted verbatim from on-chain-client/tests/common/multisig.rs; errors
//! retyped to WriteError; .unwrap()/.expect() replaced with ? / WriteError.
#![cfg(feature = "chain")]

use blake2::Blake2bVar;
use blake2::digest::{Update as _, VariableOutput};
use parity_scale_codec::Encode;
use subxt::OnlineClient;
use subxt::config::PolkadotConfig;
use subxt::dynamic::{self, Value};
use subxt::ext::scale_value::{Composite, ValueDef};
use subxt::extrinsics::ExtrinsicEvents;
use subxt::transactions::StaticPayload;
use subxt_signer::sr25519::Keypair;

use crate::chain_write::{DispatchOutcome, WriteError};
use crate::chain_write::proxy::BlockSink;

/// Turn a composed RuntimeCall `Value` — shaped `Variant(pallet,
/// Unnamed([Variant(call_name, fields)]))` by `proxied`/`create_pure_call` — into
/// a top-level extrinsic payload. Used by the single-admin (direct) dispatch
/// path, which submits the call itself rather than nesting it in
/// `as_multi_threshold_1`.
fn runtime_call_to_tx(call: Value) -> Result<StaticPayload<Composite<()>>, WriteError> {
    let ValueDef::Variant(pallet_var) = call.value else {
        return Err(WriteError::MalformedCall("call is not a RuntimeCall variant"));
    };
    let pallet_name = pallet_var.name;
    let inner = pallet_var
        .values
        .into_values()
        .next()
        .ok_or(WriteError::MalformedCall("RuntimeCall has no inner call"))?;
    let ValueDef::Variant(call_var) = inner.value else {
        return Err(WriteError::MalformedCall("inner call is not a variant"));
    };
    Ok(subxt::dynamic::tx(pallet_name, call_var.name, call_var.values))
}

/// 100 PAS (Paseo AH uses 10 decimals). Generous budget for existential
/// deposit + fees + pallet-revive storage deposits in scenario tests.
pub const FUND_AMOUNT: u128 = 1_000_000_000_000;

/// pallet-multisig pseudo-account: `blake2_256(scale_encode((
/// b"modlpy/utilisuba", sorted_signers, threshold)))`. Mirrors
/// `pallet_multisig::Pallet::multi_account_id`.
pub fn multi_account_id(signers: &[[u8; 32]], threshold: u16) -> [u8; 32] {
    let mut sorted: Vec<[u8; 32]> = signers.to_vec();
    sorted.sort();
    let entropy = (b"modlpy/utilisuba", sorted, threshold).encode();
    blake2_256(&entropy)
}

fn blake2_256(data: &[u8]) -> [u8; 32] {
    // 32 is always a valid blake2b output size; the only invalid sizes are
    // 0 and >64. Map construction failure to a compile-time-unreachable path
    // by returning zeroes (but in practice this branch is never taken).
    let mut hasher = match Blake2bVar::new(32) {
        Ok(h) => h,
        Err(_) => return [0u8; 32],
    };
    hasher.update(data);
    let mut out = [0u8; 32];
    // The output buffer is exactly the declared size — finalize_variable only
    // fails if the buffer is longer than the output length we declared (32).
    // This branch is statically unreachable.
    let _ = hasher.finalize_variable(&mut out);
    out
}

/// Submit `tx` signed by `signer`, drive block production via `sink`, and return
/// the events of the extrinsic IN THE BLOCK IT ACTUALLY LANDED IN, after it is
/// finalized. Returns `Err` (surfacing the dispatch error) if the extrinsic
/// finalized with `ExtrinsicFailed`.
///
/// This replaces the old "submit fire-and-forget, then read events at whatever
/// block `sink.settle()` returned" pattern, which was only correct on chopsticks
/// (its sink mines exactly the tx's block). On a live chain "the next finalized
/// block" is almost never the tx's block, so events were read from the wrong
/// place and dispatch failures were invisible.
///
/// Works on both backends: each `sink.settle()` mines one block on chopsticks
/// (the tx may need more than one due to mempool lag) or waits for a newer
/// finalized block on a live chain. We keep settling until THIS extrinsic is
/// finalized, so we never hang waiting for a block that the test harness hasn't
/// mined yet.
async fn submit_and_watch<Call: subxt::transactions::Payload>(
    api: &OnlineClient<PolkadotConfig>,
    sink: &dyn BlockSink,
    signer: &Keypair,
    tx: &Call,
) -> Result<ExtrinsicEvents<PolkadotConfig>, WriteError> {
    let progress = api
        .tx()
        .await
        .map_err(|e| WriteError::Subxt(format!("tx_client: {e}")))?
        .sign_and_submit_then_watch_default(tx, signer)
        .await
        .map_err(|e| WriteError::Subxt(format!("submit: {e}")))?;

    let fin = progress.wait_for_finalized_success();
    tokio::pin!(fin);
    loop {
        tokio::select! {
            res = &mut fin => {
                return res
                    .map_err(|e| WriteError::Subxt(format!("extrinsic dispatch failed: {e}")));
            }
            settled = sink.settle() => {
                // Advanced/produced a block; loop to re-check finalization.
                // Propagate a sink error (e.g. dev_newBlock RPC failure).
                settled?;
            }
        }
    }
}

/// Build the extrinsic that dispatches `call` under the current controller:
/// `others` empty ⇒ the call as a top-level extrinsic (single admin, direct);
/// `others` non-empty ⇒ `Multisig.as_multi_threshold_1(others, call)`
/// (threshold-1 "any one of N"). Pure (no chain I/O) so it is unit-testable.
pub(crate) fn build_dispatch_tx(
    other_signatories: &[[u8; 32]],
    call: Value,
) -> Result<StaticPayload<Composite<()>>, WriteError> {
    if other_signatories.is_empty() {
        return runtime_call_to_tx(call);
    }
    let mut sorted_others: Vec<[u8; 32]> = other_signatories.to_vec();
    sorted_others.sort();
    let others: Vec<Value> = sorted_others
        .iter()
        .map(|id| Value::from_bytes(id.as_slice()))
        .collect();
    Ok(dynamic::tx(
        "Multisig",
        "as_multi_threshold_1",
        Composite::unnamed(vec![Value::unnamed_composite(others), call]),
    ))
}

/// Dispatch `call` as the org's controller, contributing `signer`'s signature,
/// driving block production via `sink`, and returning the structured outcome.
/// Single admin (empty `others`) and threshold-1 multisig both execute in one
/// transaction, so this returns `DispatchOutcome::Executed`.
pub async fn dispatch_org_call(
    sink: &dyn BlockSink,
    api: &OnlineClient<PolkadotConfig>,
    signer: &Keypair,
    other_signatories: &[[u8; 32]],
    call: Value,
) -> Result<DispatchOutcome, WriteError> {
    let tx = build_dispatch_tx(other_signatories, call)?;
    let events = submit_and_watch(api, sink, signer, &tx).await?;
    Ok(DispatchOutcome::Executed(events))
}

/// Transfer `amount` plancks from `from` to the 32-byte account `dest`
/// via `Balances.transfer_keep_alive`, driving the chain via `sink` and waiting
/// for the transfer to finalize successfully. Used to fund multisig pseudo-
/// accounts and pure proxies (existential deposit + fees + revive storage
/// deposits).
pub async fn fund(
    sink: &dyn BlockSink,
    api: &OnlineClient<PolkadotConfig>,
    from: &Keypair,
    dest: [u8; 32],
    amount: u128,
) -> Result<(), WriteError> {
    let dest_value = Value::variant(
        "Id",
        Composite::unnamed(vec![Value::from_bytes(dest.as_slice())]),
    );
    let tx = dynamic::tx(
        "Balances",
        "transfer_keep_alive",
        vec![dest_value, Value::u128(amount)],
    );
    submit_and_watch(api, sink, from, &tx).await?;
    Ok(())
}


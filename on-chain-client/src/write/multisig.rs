//! The multisig that controls an Organisation's slot: its pseudo-account
//! (LLR-gjx3jn, LLR-kqxw9t) and the shape of a dispatch under it (LLR-qs8jfw).

use blake2::Blake2bVar;
use blake2::digest::{Update as _, VariableOutput};
use parity_scale_codec::Encode;
use subxt::dynamic::{self, Value};
use subxt::ext::scale_value::{Composite, ValueDef};
use subxt::transactions::StaticPayload;

use super::{AccountId, WriteError};

/// pallet-multisig's pseudo-account: `blake2_256(scale((b"modlpy/utilisuba",
/// sorted signatories, threshold)))`.
pub fn multi_account_id(signatories: &[AccountId], threshold: u16) -> AccountId {
    let mut sorted: Vec<[u8; 32]> = signatories.iter().map(|a| a.0).collect();
    sorted.sort();
    AccountId(blake2_256(&(b"modlpy/utilisuba", sorted, threshold).encode()))
}

fn blake2_256(data: &[u8]) -> [u8; 32] {
    // 32 is a valid blake2b output size, so construction cannot fail; the
    // zero fallback is unreachable and keeps the function total.
    let mut hasher = match Blake2bVar::new(32) {
        Ok(h) => h,
        Err(_) => return [0u8; 32],
    };
    hasher.update(data);
    let mut out = [0u8; 32];
    let _ = hasher.finalize_variable(&mut out);
    out
}

/// An extrinsic as plain values: which pallet, which call, which fields.
#[derive(Clone, Debug, PartialEq)]
pub struct DispatchTx {
    pub pallet: String,
    pub call: String,
    pub fields: Composite<()>,
}

impl DispatchTx {
    /// The subxt payload this extrinsic is submitted as.
    pub fn into_payload(self) -> StaticPayload<Composite<()>> {
        dynamic::tx(self.pallet, self.call, self.fields)
    }
}

/// A RuntimeCall value — `Variant(pallet, Unnamed([Variant(call, fields)]))`
/// — as a top-level extrinsic.
fn runtime_call_to_tx(call: Value) -> Result<DispatchTx, WriteError> {
    let ValueDef::Variant(pallet_var) = call.value else {
        return Err(WriteError::MalformedCall("call is not a RuntimeCall variant"));
    };
    let inner = pallet_var
        .values
        .into_values()
        .next()
        .ok_or(WriteError::MalformedCall("RuntimeCall has no inner call"))?;
    let ValueDef::Variant(call_var) = inner.value else {
        return Err(WriteError::MalformedCall("inner call is not a variant"));
    };
    Ok(DispatchTx { pallet: pallet_var.name, call: call_var.name, fields: call_var.values })
}

/// `call` under the controller: itself when there are no co-signatories,
/// otherwise `Multisig.as_multi_threshold_1(sorted co-signatories, call)`.
/// Every dispatch the writer makes goes through this function (LLR-qs8jfw).
pub fn build_dispatch_tx(co_signatories: &[AccountId], call: Value) -> Result<DispatchTx, WriteError> {
    if co_signatories.is_empty() {
        return runtime_call_to_tx(call);
    }
    let mut sorted = co_signatories.to_vec();
    sorted.sort();
    let others: Vec<Value> = sorted.iter().map(|a| Value::from_bytes(a.0.as_slice())).collect();
    Ok(DispatchTx {
        pallet: "Multisig".into(),
        call: "as_multi_threshold_1".into(),
        fields: Composite::unnamed(vec![Value::unnamed_composite(others), call]),
    })
}

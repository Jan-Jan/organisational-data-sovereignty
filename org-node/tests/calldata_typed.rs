#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The typed calldata paths (LLR-ayrdr8): the `update` calldata that the
//! write path builds from a `RootHash`, an `OrgPublicKey` and an `Epoch` is
//! byte-for-byte the calldata pinned before the refactor. The pins in
//! `encoding_golden.rs` cover the untyped `build_update_calldata` only; a typed
//! wrapper that swapped the root and the key would pass them. The values below
//! are copies of `GOLDEN_GENESIS_CALLDATA` and `GOLDEN_UPDATE_CALLDATA` from
//! that file (which is never edited), for the same inputs: the root and the key
//! hold different bytes, so a swap changes the output.

use org_members::RootHash;
use org_node::chain_write::calldata::{revive_update_runtime_call, update_calldata};
use org_node::{Epoch, OrgPublicKey};
use subxt::ext::scale_value::{Composite, Primitive, Value, ValueDef};

const GOLDEN_GENESIS_CALLDATA: &str = "f1bc537b333333333333333333333333333333333333333333333333333333333333333322222222222222222222222222222222222222222222222222222222222222220000000000000000000000000000000000000000000000000000000000000000";
const GOLDEN_UPDATE_CALLDATA: &str = "f1bc537b666666666666666666666666666666666666666666666666666666666666666622222222222222222222222222222222222222222222222222222222222222220000000000000000000000000000000000000000000000000000000000000007";

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// The pinned inputs: (root, Organisation public key, epoch, pinned calldata).
fn pinned() -> [(RootHash, OrgPublicKey, Epoch, &'static str); 2] {
    let key = OrgPublicKey::parse(&[0x22; 32]).expect("the pinned key is a curve point");
    [
        (RootHash::new([0x33; 32]), key, Epoch::new(0), GOLDEN_GENESIS_CALLDATA),
        (RootHash::new([0x66; 32]), key, Epoch::new(7), GOLDEN_UPDATE_CALLDATA),
    ]
}

fn variant<'a>(v: &'a Value, name: &str) -> &'a Composite<()> {
    match &v.value {
        ValueDef::Variant(var) if var.name == name => &var.values,
        other => panic!("expected variant {name}, got {other:?}"),
    }
}

/// The `data` field of the `Revive.call` runtime call, as bytes.
fn revive_call_data(call: &Value) -> Vec<u8> {
    let Composite::Unnamed(outer) = variant(call, "Revive") else { panic!("Revive holds one unnamed call") };
    let Composite::Named(fields) = variant(&outer[0], "call") else { panic!("call has named fields") };
    let (_, data) = fields.iter().find(|(n, _)| n == "data").expect("call has a data field");
    let ValueDef::Composite(Composite::Unnamed(bytes)) = &data.value else { panic!("data is a byte list") };
    bytes
        .iter()
        .map(|b| match &b.value {
            ValueDef::Primitive(Primitive::U128(n)) => u8::try_from(*n).unwrap(),
            other => panic!("data byte is not an integer: {other:?}"),
        })
        .collect()
}

/// `update_calldata` is what both typed write paths build their calldata
/// with: `revive_update_runtime_call` (the proxied path `SubxtChainOps` uses)
/// and `submit::submit_update` (the direct path, which needs a live chain and
/// is therefore covered through this function).
/// verifies: LLR-ayrdr8
#[test]
fn typed_update_calldata_is_the_pinned_calldata() {
    for (root, key, epoch, golden) in pinned() {
        assert_eq!(hex(&update_calldata(root, key, epoch)), golden);
    }
}

/// verifies: LLR-ayrdr8
#[test]
fn the_revive_runtime_call_carries_the_pinned_calldata() {
    for (root, key, epoch, golden) in pinned() {
        let call = revive_update_runtime_call([0xab; 20], root, key, epoch);
        assert_eq!(hex(&revive_call_data(&call)), golden);
    }
}

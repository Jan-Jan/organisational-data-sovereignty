#![cfg(all(feature = "chain", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The pure parts of the write path: update calldata (SDD-msb6xh) and the
//! threshold-1 multisig derivation (SDD-rq6nv4).
//!
//! Relocated from the `#[cfg(test)]` modules in
//! `org-node/src/chain_write/calldata.rs` and
//! `org-node/src/chain_write/multisig.rs` so the `verifies:` annotations sit
//! under `test_paths`. `build_dispatch_tx` is crate-private and is reached
//! through `org_node::test_support`, the seam described in the decomposition.
//!
//! Everything else in `chain_write` is SDD-z85ux9, which carries no low-level
//! requirements because this unit's gate cannot reach it.

use org_node::chain_write::calldata::{build_update_calldata, UPDATE_SELECTOR};
use org_node::chain_write::multisig::multi_account_id;
use org_node::test_support::build_dispatch_tx;
use org_node::ChainAccount;

// Ported 2026-10-05 to the org-node type-safety change: the multisig
// derivation and dispatch take `ChainAccount`s, and the runtime call a
// `RootHash`, an `OrgPublicKey` and an `Epoch`. `build_update_calldata` is the
// plain-bytes encoder at the edge and is unchanged.
fn acct(b: u8) -> ChainAccount {
    ChainAccount::new([b; 32])
}

// verifies: LLR-rv4vux
#[test]
fn update_calldata_is_exactly_one_hundred_bytes_in_the_declared_order() {
    let root = [0x11u8; 32];
    let key = [0x22u8; 32];
    let data = build_update_calldata(root, key, 7);
    assert_eq!(data.len(), 100);
    assert_eq!(&data[0..4], &UPDATE_SELECTOR);
    assert_eq!(&data[4..36], &root);
    assert_eq!(&data[36..68], &key);
}

// verifies: LLR-txqmz4
#[test]
fn the_expected_epoch_occupies_the_low_sixteen_bytes_big_endian() {
    let data = build_update_calldata([0x11u8; 32], [0x22u8; 32], 7);
    // 31 zero bytes then 0x07 — big-endian in a uint256 field.
    assert_eq!(data[99], 7);
    assert!(data[68..99].iter().all(|b| *b == 0));

    // A value wider than eight bytes still lands in the low sixteen, and the
    // high sixteen stay zero.
    let wide = build_update_calldata([0u8; 32], [0u8; 32], u128::MAX);
    assert!(wide[68..84].iter().all(|b| *b == 0), "high 16 bytes must stay zero");
    assert!(wide[84..100].iter().all(|b| *b == 0xff), "low 16 bytes carry the value");
}

// verifies: LLR-66h529
#[test]
fn the_update_selector_is_the_declared_four_bytes() {
    // keccak256("update(bytes32,bytes32,uint256)")[..4]
    assert_eq!(UPDATE_SELECTOR, [0xf1, 0xbc, 0x53, 0x7b]);
}

// verifies: LLR-463d89
#[test]
fn the_derived_account_does_not_depend_on_signer_order() {
    let a = acct(1);
    let b = acct(2);
    assert_eq!(multi_account_id(&[a, b], 1), multi_account_id(&[b, a], 1));

    // Three signers, so the property is not an artefact of a two-element swap.
    let c = acct(3);
    assert_eq!(multi_account_id(&[a, b, c], 1), multi_account_id(&[c, a, b], 1));
}

// verifies: LLR-8m3bwj
#[test]
fn the_derived_account_depends_on_the_threshold() {
    let a = acct(1);
    let b = acct(2);
    assert_ne!(multi_account_id(&[a, b], 1), multi_account_id(&[a, b], 2));
}

// verifies: LLR-463d89
#[test]
fn the_derived_account_depends_on_the_signers() {
    assert_ne!(multi_account_id(&[acct(1)], 1), multi_account_id(&[acct(2)], 1));
}

// verifies: LLR-f74xwb
#[test]
fn dispatch_is_direct_without_other_signatories_and_wrapped_with_them() {
    use subxt::dynamic::Value;
    use subxt::ext::scale_value::Composite;

    // A minimal composed RuntimeCall: System.remark { remark }.
    let call = || {
        Value::variant(
            "System",
            Composite::unnamed(vec![Value::variant(
                "remark",
                Composite::named(vec![("remark".to_string(), Value::from_bytes([1u8; 4]))]),
            )]),
        )
    };

    let direct = build_dispatch_tx(&[], call()).expect("direct payload");
    assert_eq!(direct.pallet_name(), "System");
    assert_eq!(direct.call_name(), "remark");

    let multi = build_dispatch_tx(&[acct(9)], call()).expect("multisig payload");
    assert_eq!(multi.pallet_name(), "Multisig");
    assert_eq!(multi.call_name(), "as_multi_threshold_1");
}

// `revive_update_runtime_call` is pure and synchronous, and every name and
// constant in it is matched against runtime metadata, so a typo here is a
// call the chain rejects. The expected value is written out by hand, with the
// constants as literals, mirroring `on-chain-client/tests/common/submit.rs`:
// renaming a field, changing a constant or passing other calldata reddens it.
// Added 2026-10-04 by review round 6, which found this function filed under
// the item with no low-level requirements.
// verifies: LLR-rc74nq
#[test]
fn update_call_names_every_field_and_constant_the_runtime_matches() {
    use org_node::chain_write::calldata::revive_update_runtime_call;
    use subxt::dynamic::Value;
    use subxt::ext::scale_value::Composite;

    // Distinct bytes, so the order of `dest` is observed too: a uniform
    // address could not tell it from its reverse (review round 7).
    let contract: [u8; 20] = core::array::from_fn(|i| i as u8 + 1);
    let root = org_node::RootHash::new([0x11u8; 32]);
    let key = org_node::OrgPublicKey::from(&org_node::MemberSeed::from([0x22u8; 32]).signing_keypair().member_key());
    let epoch = org_node::Epoch::new(7);

    let expected = Value::variant(
        "Revive",
        Composite::unnamed(vec![Value::variant(
            "call",
            Composite::named(vec![
                (
                    "dest".to_string(),
                    Value::unnamed_composite(contract.iter().map(|b| Value::u128(u128::from(*b)))),
                ),
                ("value".to_string(), Value::u128(0)),
                (
                    "weight_limit".to_string(),
                    Value::named_composite([
                        ("ref_time", Value::u128(1_000_000_000_000)),
                        ("proof_size", Value::u128(4_000_000)),
                    ]),
                ),
                ("storage_deposit_limit".to_string(), Value::u128(10_000_000_000_000)),
                (
                    "data".to_string(),
                    Value::from_bytes(build_update_calldata(*root.as_bytes(), *key.as_bytes(), 7u128)),
                ),
            ]),
        )]),
    );
    assert_eq!(revive_update_runtime_call(contract, root, key, epoch), expected);
}

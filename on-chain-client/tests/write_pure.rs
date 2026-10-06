#![cfg(all(feature = "test-support", feature = "write"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The pure half of the chain writer (SDD-yg7n55): calldata, the runtime calls
//! it wraps, the multisig account and the dispatch shape. Moved from
//! org-node's `chain_write_pure` and `calldata_typed` targets with the code,
//! and re-annotated to this unit's requirements.

use on_chain_client::write::calldata::{
    build_update_calldata, revive_update_runtime_call, STORAGE_DEPOSIT_LIMIT, UPDATE_SELECTOR,
    WEIGHT_PROOF_SIZE, WEIGHT_REF_TIME,
};
use on_chain_client::write::multisig::{build_dispatch_tx, multi_account_id};
use on_chain_client::write::proxy::{map_account_call, proxied};
use on_chain_client::write::{AccountId, WriteError};
use on_chain_client::{Epoch, OnChainRootHash, OrgPubKey};
use subxt::dynamic::Value;
use subxt::ext::scale_value::Composite;

const GOLDEN_GENESIS_CALLDATA: &str = "f1bc537b333333333333333333333333333333333333333333333333333333333333333322222222222222222222222222222222222222222222222222222222222222220000000000000000000000000000000000000000000000000000000000000000";
const GOLDEN_UPDATE_CALLDATA: &str = "f1bc537b666666666666666666666666666666666666666666666666666666666666666622222222222222222222222222222222222222222222222222222222222222220000000000000000000000000000000000000000000000000000000000000007";

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
fn acct(b: u8) -> AccountId {
    AccountId([b; 32])
}
fn remark() -> Value {
    Value::variant(
        "System",
        Composite::unnamed(vec![Value::variant(
            "remark",
            Composite::named(vec![("remark".to_string(), Value::from_bytes([1u8; 4]))]),
        )]),
    )
}

// Normal: the pinned calldata of a genesis and an update, carried over from
// org-node's golden values (encoding_golden, calldata_typed).
// verifies: LLR-yvq33e, REQ-aat4yt, REQ-6jefu2
#[test]
fn update_calldata_is_the_pinned_hundred_bytes() {
    let genesis = build_update_calldata(OnChainRootHash([0x33; 32]), OrgPubKey([0x22; 32]), Epoch(0));
    let update = build_update_calldata(OnChainRootHash([0x66; 32]), OrgPubKey([0x22; 32]), Epoch(7));
    assert_eq!(hex(&genesis), GOLDEN_GENESIS_CALLDATA);
    assert_eq!(hex(&update), GOLDEN_UPDATE_CALLDATA);
    assert_eq!(update.len(), 100);
    assert_eq!(&update[0..4], &UPDATE_SELECTOR);
    assert_eq!(&update[4..36], &[0x66; 32]);
    assert_eq!(&update[36..68], &[0x22; 32]);
}

// Abnormal (boundary): an all-zero root and key at the largest epoch keep the
// layout — the fields do not shift or bleed into each other.
// verifies: LLR-yvq33e
#[test]
fn calldata_at_the_boundary_values_keeps_its_layout() {
    let data = build_update_calldata(OnChainRootHash([0; 32]), OrgPubKey([0xff; 32]), Epoch(u64::MAX));
    assert_eq!(data.len(), 100);
    assert!(data[4..36].iter().all(|b| *b == 0));
    assert!(data[36..68].iter().all(|b| *b == 0xff));
}

// Normal: the epoch is the low sixteen bytes of a big-endian word.
// verifies: LLR-5varjf, REQ-aat4yt
#[test]
fn the_expected_epoch_occupies_the_low_sixteen_bytes_big_endian() {
    let data = build_update_calldata(OnChainRootHash([0x11; 32]), OrgPubKey([0x22; 32]), Epoch(7));
    assert_eq!(data[99], 7);
    assert!(data[68..99].iter().all(|b| *b == 0));
}

// Abnormal (boundary): the largest epoch leaves the high sixteen bytes zero.
// verifies: LLR-5varjf
#[test]
fn the_largest_epoch_leaves_the_high_sixteen_bytes_zero() {
    let data = build_update_calldata(OnChainRootHash([0; 32]), OrgPubKey([0; 32]), Epoch(u64::MAX));
    assert!(data[68..84].iter().all(|b| *b == 0), "high sixteen bytes zero");
    assert_eq!(&data[84..100], &u128::from(u64::MAX).to_be_bytes());
}

fn keccak4(signature: &str) -> [u8; 4] {
    use tiny_keccak::{Hasher, Keccak};
    let mut k = Keccak::v256();
    k.update(signature.as_bytes());
    let mut out = [0u8; 32];
    k.finalize(&mut out);
    [out[0], out[1], out[2], out[3]]
}

// Normal: the selector is recomputed independently from the signature.
// verifies: LLR-d4ftc8, REQ-aat4yt
#[test]
fn the_selector_is_the_keccak_of_the_update_signature() {
    assert_eq!(UPDATE_SELECTOR, keccak4("update(bytes32,bytes32,uint256)"));
}

// Abnormal: a drifted signature yields a different constant, so drift shows
// as a changed selector here.
// verifies: LLR-d4ftc8
#[test]
fn a_drifted_signature_is_a_different_selector() {
    assert_ne!(UPDATE_SELECTOR, keccak4("update(bytes32,bytes32,uint128)"));
    assert_ne!(UPDATE_SELECTOR, keccak4("update(bytes32,bytes32)"));
}

fn expected_call(contract: [u8; 20], data: Vec<u8>) -> Value {
    Value::variant(
        "Revive",
        Composite::unnamed(vec![Value::variant(
            "call",
            Composite::named(vec![
                ("dest".to_string(), Value::unnamed_composite(contract.iter().map(|b| Value::u128(u128::from(*b))))),
                ("value".to_string(), Value::u128(0)),
                (
                    "weight_limit".to_string(),
                    Value::named_composite([
                        ("ref_time", Value::u128(1_000_000_000_000)),
                        ("proof_size", Value::u128(4_000_000)),
                    ]),
                ),
                ("storage_deposit_limit".to_string(), Value::u128(10_000_000_000_000)),
                ("data".to_string(), Value::from_bytes(data)),
            ]),
        )]),
    )
}

// Normal: every field name and constant the runtime matches by name.
// verifies: LLR-m7wmmx, REQ-aat4yt
#[test]
fn the_revive_call_names_every_field_and_constant() {
    let contract: [u8; 20] = core::array::from_fn(|i| i as u8 + 1);
    let (root, key, epoch) = (OnChainRootHash([0x11; 32]), OrgPubKey([0x22; 32]), Epoch(7));
    assert_eq!(
        revive_update_runtime_call(contract, root, key, epoch),
        expected_call(contract, build_update_calldata(root, key, epoch))
    );
    assert_eq!((WEIGHT_REF_TIME, WEIGHT_PROOF_SIZE, STORAGE_DEPOSIT_LIMIT), (1_000_000_000_000, 4_000_000, 10_000_000_000_000));
}

// Abnormal: two calls differing only in the epoch differ only in `data`.
// verifies: LLR-m7wmmx
#[test]
fn a_different_epoch_changes_only_the_data_field() {
    let contract = [0xab; 20];
    let (root, key) = (OnChainRootHash([0x11; 32]), OrgPubKey([0x22; 32]));
    let at_8 = revive_update_runtime_call(contract, root, key, Epoch(8));
    assert_ne!(revive_update_runtime_call(contract, root, key, Epoch(7)), at_8);
    assert_eq!(at_8, expected_call(contract, build_update_calldata(root, key, Epoch(8))));
}

// Normal: `proxied` and `map_account_call` build the named calls.
// verifies: LLR-ywhd23, REQ-aat4yt, REQ-6jefu2
#[test]
fn proxied_and_map_account_build_the_named_calls() {
    let expected = Value::variant(
        "Proxy",
        Composite::unnamed(vec![Value::variant(
            "proxy",
            Composite::named(vec![
                ("real".to_string(), Value::variant("Id", Composite::unnamed(vec![Value::from_bytes([7u8; 32])]))),
                ("force_proxy_type".to_string(), Value::variant("None", Composite::unnamed(vec![]))),
                ("call".to_string(), remark()),
            ]),
        )]),
    );
    assert_eq!(proxied(acct(7), remark()), expected);
    assert_eq!(
        map_account_call(),
        Value::variant("Revive", Composite::unnamed(vec![Value::variant("map_account", Composite::unnamed(vec![]))]))
    );
}

// Abnormal: wrapping is total — a proxied call wrapped again nests unchanged.
// verifies: LLR-ywhd23
#[test]
fn proxied_wraps_any_call_unchanged_even_another_proxied_call() {
    let inner = proxied(acct(1), remark());
    let outer = proxied(acct(2), inner.clone());
    assert_eq!(outer, proxied(acct(2), inner));
    assert_ne!(outer, proxied(acct(1), remark()));
}

// Normal: the derived account ignores the order the signatories came in.
// verifies: LLR-gjx3jn, REQ-6jefu2
#[test]
fn the_derived_account_does_not_depend_on_signatory_order() {
    let (a, b, c) = (acct(1), acct(2), acct(3));
    assert_eq!(multi_account_id(&[a, b], 1), multi_account_id(&[b, a], 1));
    assert_eq!(multi_account_id(&[a, b, c], 1), multi_account_id(&[c, a, b], 1));
    assert_ne!(multi_account_id(&[acct(1)], 1), multi_account_id(&[acct(2)], 1));
}

// Abnormal (boundary): an empty set and a set with a repeated signatory
// still derive deterministically, independent of order.
// verifies: LLR-gjx3jn
#[test]
fn empty_and_repeated_signatory_sets_derive_deterministically() {
    assert_eq!(multi_account_id(&[], 1), multi_account_id(&[], 1));
    let (a, b) = (acct(1), acct(2));
    assert_eq!(multi_account_id(&[a, a, b], 1), multi_account_id(&[b, a, a], 1));
    assert_ne!(multi_account_id(&[a, a, b], 1), multi_account_id(&[a, b], 1));
}

// Normal: the threshold is part of the derivation.
// verifies: LLR-kqxw9t, REQ-6jefu2
#[test]
fn the_derived_account_depends_on_the_threshold() {
    assert_ne!(multi_account_id(&[acct(1), acct(2)], 1), multi_account_id(&[acct(1), acct(2)], 2));
}

// Abnormal (boundary): thresholds zero and u16::MAX are distinct from 1 too.
// verifies: LLR-kqxw9t
#[test]
fn boundary_thresholds_derive_distinct_accounts() {
    let set = [acct(1), acct(2)];
    let ids = [0u16, 1, u16::MAX].map(|t| multi_account_id(&set, t));
    assert_ne!(ids[0], ids[1]);
    assert_ne!(ids[1], ids[2]);
    assert_ne!(ids[0], ids[2]);
}

// Normal: no co-signatories dispatches the call itself; with them, it is
// wrapped in `as_multi_threshold_1` with the co-signatories sorted by bytes.
// verifies: LLR-qs8jfw, REQ-6jefu2, REQ-aat4yt
#[test]
fn dispatch_is_direct_without_co_signatories_and_sorted_multisig_with_them() {
    let direct = build_dispatch_tx(&[], remark()).unwrap();
    assert_eq!((direct.pallet.as_str(), direct.call.as_str()), ("System", "remark"));
    let multi = build_dispatch_tx(&[acct(9), acct(3)], remark()).unwrap();
    assert_eq!((multi.pallet.as_str(), multi.call.as_str()), ("Multisig", "as_multi_threshold_1"));
    assert_eq!(
        multi.fields,
        Composite::unnamed(vec![
            Value::unnamed_composite(vec![Value::from_bytes([3u8; 32]), Value::from_bytes([9u8; 32])]),
            remark(),
        ])
    );
    assert_eq!(build_dispatch_tx(&[acct(3), acct(9)], remark()).unwrap(), multi);
}

// Abnormal: a direct dispatch of a value that is not a RuntimeCall is refused
// with `MalformedCall` at each of `runtime_call_to_tx`'s three arms (owed by
// the robustness table in org-node/docs/architecture/2026-10-03-decomposition.md,
// where the multisig item that held this code before it moved here is listed).
// verifies: LLR-qs8jfw
#[test]
fn a_value_that_is_not_a_runtime_call_is_refused_on_each_arm() {
    assert_eq!(
        build_dispatch_tx(&[], Value::u128(1)).unwrap_err(),
        WriteError::MalformedCall("call is not a RuntimeCall variant")
    );
    assert_eq!(
        build_dispatch_tx(&[], Value::variant("System", Composite::unnamed(vec![]))).unwrap_err(),
        WriteError::MalformedCall("RuntimeCall has no inner call")
    );
    assert_eq!(
        build_dispatch_tx(&[], Value::variant("System", Composite::unnamed(vec![Value::u128(1)]))).unwrap_err(),
        WriteError::MalformedCall("inner call is not a variant")
    );
}

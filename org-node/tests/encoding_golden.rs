#![cfg(feature = "app")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Pins the bytes org-node stores, sends and submits, so the type-level
//! refactor of the org-node type-safety change cannot silently change them
//! (LLR-ayrdr8): the postcard plaintext of a Persona store, the postcard body
//! of an admission Wire message and the record snapshot inside it, the Base64
//! text of an Invite and a Join request, and the EVM calldata of a genesis and
//! an update.
//!
//! Values captured on worktree-org-node-type-safety @ 813f85b (2026-10-04),
//! before the refactor, by the generator in
//! docs/plans/2026-10-04-org-node-type-safety.md (T1). Never edit this file to
//! make it pass. Each test decodes a pinned value with whatever the field
//! types are at the time, requires it to encode back to the same bytes, and
//! reads named fields through serde's data model (`serde_json::Value`), which
//! is the same for a newtype and the plain value it wraps. The first catches
//! an asymmetric encoding change; the second catches a symmetric one, such as
//! two fields swapping places, which a byte round trip alone would miss.

use org_node::blobs::{self, Invite, JoinRequest};
use org_node::chain_write::calldata::build_update_calldata;
use org_node::service::first_admission_base;
use org_node::store::{MemberSnapshot, StoreData};
use org_node::transport::wire::{decode_body, encode_frame};
use serde_json::{json, Value};

const GOLDEN_STORE: &str = "0107702d616c69636501aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa05616c69636505416c69636505536d697468111111111111111111111111111111111111111111111111111111111111111112121212121212121212121212121212121212121212121212121212121212120101010101010101010101010101010101010101010101010101010101010101010101aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa3333333333333333333333333333333333333333333333333333333333333333d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c97787370301444444444444444444444444444444444444444444444444444444444444444402d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873702010101010101010101010101010101010101010101010101010101010101010105616c69636505416c69636505536d697468d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21020202020202020202020202020202020202020202020202020202020202020203626f6203426f62054a6f6e6573884b8857f4eaa1613c61504db34d4beaf346517a0e31de3cddd4d9b4201d9d0b01a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f001555555555555555555555555555555555555555555555555555555555555555501bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737";
const GOLDEN_WIRE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa029101b642ec203b364f4807ea74b0a63ab680c1c94d38b23246b1042f520394477e860001020202020202020202020202020202020202020202020202020202020202020203626f628139770ea87d175f56a35466c34c7ecccb8d8a91b4ee37a25df60f5b8fc9b3940454657374045573657201ed4928c628d1c2c6eae90338905995612959273a5c63f93636c14614ac8737d140523943b8c0e60f3412a3d5e2972b519b6e73eb2cc0ad9ebd217ca11640ebc38fc478dc6c7a3cbe94da88916e7295f66c52dff6a5c9c31eddfca932bdc245c10301444444444444444444444444444444444444444444444444444444444444444401720101010101010101010101010101010101010101010101010101010101010101010561646d696e04546573740455736572d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701ca93ac1705187071d67b83c7ff0efe8108e8ec4530575d7726879333dbdabe7c";
const GOLDEN_INVITE: &str = "qqqqqqqqqqqqqqqqqqqqqqqqqqrQSrIydCu0qzoTaL1GFeTm0CJKtxoBa6+FIKMyyXeHN9BKsjJ0K7SrOhNovUYV5ObQIkq3GgFrr4UgozLJd4c3IEBA42TBDyvsnB/lAKHNTCR8idZQoB7X6CyrqGeHfCEDAQID";
const GOLDEN_JOIN_REQUEST: &str = "A2JvYgNCb2IFSm9uZXOIS4hX9OqhYTxhUE2zTUvq80ZReg4x3jzd1Nm0IB2dC6CapfR6Z1mAL/lV+NwtKhSlyZ0jvpf4ZBJ/+Tg0VaTwAwQFBg==";
const GOLDEN_GENESIS_CALLDATA: &str = "f1bc537b333333333333333333333333333333333333333333333333333333333333333322222222222222222222222222222222222222222222222222222222222222220000000000000000000000000000000000000000000000000000000000000000";
const GOLDEN_UPDATE_CALLDATA: &str = "f1bc537b666666666666666666666666666666666666666666666666666666666666666622222222222222222222222222222222222222222222222222222222222222220000000000000000000000000000000000000000000000000000000000000007";

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

/// The value as serde's data model sees it, field names included.
fn model<T: serde::Serialize>(value: &T) -> Value {
    serde_json::to_value(value).unwrap()
}

/// verifies: LLR-ayrdr8
#[test]
fn persona_store_plaintext_is_pinned() {
    let bytes = unhex(GOLDEN_STORE);
    let data: StoreData = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(hex(&postcard::to_allocvec(&data).unwrap()), GOLDEN_STORE);

    let m = model(&data);
    let persona = &m["personas"][0];
    assert_eq!(persona["persona_id"], json!("p-alice"));
    assert_eq!(persona["org_id"][0], json!(0xaa));
    assert_eq!(persona["handle"], json!("alice"));
    assert_eq!(persona["name"], json!("Alice"));
    assert_eq!(persona["surname"], json!("Smith"));
    assert_eq!(persona["member_seed"][31], json!(0x11));
    assert_eq!(persona["device_seed"][31], json!(0x12));
    assert_eq!(persona["member_id"][0], json!(0x01));
    assert_eq!(persona["status"], json!("Active"));
    let org = &m["orgs"][0];
    assert_eq!(org["org_id"][0], json!(0xaa));
    assert_eq!(org["root_hash"][0], json!(0x33));
    assert_eq!(org["org_pub_key"][0], json!(0xd0));
    assert_eq!(org["epoch"], json!(3));
    assert_eq!(org["org_secret"][31], json!(0x44));
    assert_eq!(org["last_seq"], json!(2));
    assert_eq!(org["admin_member_key"][0], json!(0xd0));
    assert_eq!(org["proxy_account"][0], json!(0x55));
    let bob = &org["trie_members"][1];
    assert_eq!(bob["id"][0], json!(0x02));
    assert_eq!(bob["handle"], json!("bob"));
    assert_eq!(bob["name"], json!("Bob"));
    assert_eq!(bob["surname"], json!("Jones"));
    assert_eq!(bob["member_key"][0], json!(0x88));
    assert_eq!(bob["device_keys"][0][0], json!(0xa0));
    let invite = &m["pending_invites"][0];
    assert_eq!(invite["org_id"][0], json!(0xbb));
    assert_eq!(invite["admin_device_key"][0], json!(0x20));
    assert_eq!(invite["admin_member_key"][0], json!(0xd0));
    assert_eq!(invite["org_pub_key"][0], json!(0xd0));
}

/// verifies: LLR-ayrdr8
#[test]
fn admission_wire_message_is_pinned() {
    let body = unhex(GOLDEN_WIRE);
    let msg = decode_body(&body).unwrap();
    let framed = encode_frame(&msg).unwrap();
    assert_eq!(hex(&framed[4..]), GOLDEN_WIRE);

    let m = model(&msg);
    assert_eq!(m["envelope"]["org_id"][0], json!(0xaa));
    assert_eq!(m["envelope"]["parent_seq"], json!(2));
    assert_eq!(m["org_secret"][31], json!(0x44));

    let snapshot = msg.genesis_snapshot.as_deref().expect("an admission carries a record snapshot");
    let members: Vec<MemberSnapshot> = postcard::from_bytes(snapshot).unwrap();
    assert_eq!(postcard::to_allocvec(&members).unwrap(), snapshot);
    let admin = &model(&members)[0];
    assert_eq!(admin["handle"], json!("admin"));
    assert_eq!(admin["member_key"][0], json!(0xd0));
    assert!(first_admission_base(Some(snapshot)).is_ok(), "the pinned snapshot rebuilds a record");
}

/// verifies: LLR-ayrdr8
#[test]
fn invite_and_join_request_text_is_pinned() {
    let invite: Invite = blobs::decode(GOLDEN_INVITE).unwrap();
    assert_eq!(blobs::encode(&invite).unwrap(), GOLDEN_INVITE);
    let m = model(&invite);
    assert_eq!(m["org_id"][0], json!(0xaa));
    assert_eq!(m["org_pub_key"][0], json!(0xd0));
    assert_eq!(m["admin_member_key"][0], json!(0xd0));
    assert_eq!(m["admin_device_key"][0], json!(0x20));
    assert_eq!(m["admin_node_addr"], json!([1, 2, 3]));

    let join_request: JoinRequest = blobs::decode(GOLDEN_JOIN_REQUEST).unwrap();
    assert_eq!(blobs::encode(&join_request).unwrap(), GOLDEN_JOIN_REQUEST);
    let m = model(&join_request);
    assert_eq!(m["handle"], json!("bob"));
    assert_eq!(m["name"], json!("Bob"));
    assert_eq!(m["surname"], json!("Jones"));
    assert_eq!(m["member_key"][0], json!(0x88));
    assert_eq!(m["device_key"][0], json!(0xa0));
    assert_eq!(m["node_addr"], json!([4, 5, 6]));
}

/// verifies: LLR-ayrdr8
#[test]
fn genesis_and_update_calldata_is_pinned() {
    assert_eq!(hex(&build_update_calldata([0x33; 32], [0x22; 32], 0)), GOLDEN_GENESIS_CALLDATA);
    assert_eq!(hex(&build_update_calldata([0x66; 32], [0x22; 32], 7)), GOLDEN_UPDATE_CALLDATA);
}

/// Abnormal input: a pinned value cut short is refused, never decoded into
/// something else.
/// verifies: LLR-ayrdr8
#[test]
fn truncated_pinned_values_are_refused() {
    let store = unhex(GOLDEN_STORE);
    assert!(postcard::from_bytes::<StoreData>(&store[..store.len() - 1]).is_err());
    let body = unhex(GOLDEN_WIRE);
    assert!(decode_body(&body[..body.len() - 1]).is_err());
    assert!(blobs::decode::<Invite>(&GOLDEN_INVITE[..GOLDEN_INVITE.len() - 4]).is_err());
    assert!(blobs::decode::<JoinRequest>(&GOLDEN_JOIN_REQUEST[..GOLDEN_JOIN_REQUEST.len() - 4]).is_err());
}

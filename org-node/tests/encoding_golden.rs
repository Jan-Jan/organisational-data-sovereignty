#![cfg(feature = "app")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Pins the bytes org-node stores and sends, so a change of a type cannot
//! silently change them (LLR-ayrdr8, LLR-mkvx2k): the postcard plaintext of a
//! Persona store in its two forms, and the postcard bodies of the two kinds of
//! Wire message and the record snapshot inside the first. The EVM calldata of
//! the chain writes is pinned by on-chain-client's `write_pure`.
//!
//! Each value is derived by the postcard format rule from the fields stated
//! beside it, never captured from the code. Never edit this file to make it
//! pass. Each test decodes a pinned value with whatever the field
//! types are at the time, requires it to encode back to the same bytes, and
//! reads named fields through serde's data model (`serde_json::Value`), which
//! is the same for a newtype and the plain value it wraps. The first catches
//! an asymmetric encoding change; the second catches a symmetric one, such as
//! two fields swapping places, which a byte round trip alone would miss.

use org_node::removal::StoreContent;
use org_node::service::first_admission_base;
use org_node::store::MemberSnapshot;
use org_node::wire::{decode_message, encode_message, WireMessage};
use serde_json::{json, Value};

// GOLDEN_WIRE is Organisation information (LLR-js9dsu): `00`, the variant
// index ‖ the Envelope (`org_id` aa×20, parent sequence 2, its Change set
// bytes; no signature, LLR-e7s4ye) ‖ the record snapshot (one Member, "admin"
// "Test" "User", member key d04a…8737, Device ca93…be7c) ‖ the Organisation
// private key (44×32), the last two without an option tag. Derived by the
// format rule from those fields, never captured from the code.
const GOLDEN_WIRE: &str = "00aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa029101b642ec203b364f4807ea74b0a63ab680c1c94d38b23246b1042f520394477e860001020202020202020202020202020202020202020202020202020202020202020203626f628139770ea87d175f56a35466c34c7ecccb8d8a91b4ee37a25df60f5b8fc9b3940454657374045573657201ed4928c628d1c2c6eae90338905995612959273a5c63f93636c14614ac8737d1720101010101010101010101010101010101010101010101010101010101010101010561646d696e04546573740455736572d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701ca93ac1705187071d67b83c7ff0efe8108e8ec4530575d7726879333dbdabe7c4444444444444444444444444444444444444444444444444444444444444444";
// GOLDEN_REVOCATION is a revocation notice (LLR-dc45ur), 123 bytes, derived
// from the notice's field order, never captured from the code: `01` ‖
// `org_id` (`aa`x20) ‖ `member_id` (`02`x32) ‖ `device` (the snapshot's
// Device key `ca93…be7c` from GOLDEN_WIRE) ‖ the proof, the empty tree's
// absence proof in the sparse-merkle-tree unit's serde form: `00` (Vacant) ‖
// the depth-256 absence proof as the tuple (depth, map, siblings, ending):
// depth 256 as a postcard varint (`8002`) ‖ the map, `20` (its length, 32) ‖
// `00`x32 (the map marks the siblings that are NOT their level's
// empty-subtree hash, so none) ‖ `siblings` (`00`, none) ‖ `ending` (`00`,
// Empty).
const GOLDEN_REVOCATION: &str = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0202020202020202020202020202020202020202020202020202020202020202ca93ac1705187071d67b83c7ff0efe8108e8ec4530575d7726879333dbdabe7c0080022000000000000000000000000000000000000000000000000000000000000000000000";

// The store plaintext is a `StoreContent` (LLR-mkvx2k, LLR-d925uu, LLR-6zaeb6): its variant index, then
// its fields. No migration (LLR-78jfrd: a store of an earlier layout is not read). Both values are derived by the format rule from the
// fields below, never captured from the code: varint lengths, counts, epochs and variant indices; `00`/`01`
// option tags; strings as length ‖ UTF-8; keys, identifiers and roots as their raw bytes; a Signature64 as
// length `40` ‖ 64 bytes; an absence proof in the sparse-merkle-tree form, `00` ‖ `8002` ‖ `20` ‖ `00`x32 ‖
// `00` ‖ `00` (as GOLDEN_REVOCATION's).
// Keys shared with the Wire pins: K = d04a…8737 (Organisation public key and alice's member key),
// DA = 2040…7c21 (alice's Device), MB = 884b…9d0b, DB = a09a…a4f0, DC = ca93…be7c, DD = ed49…37d1.
// GOLDEN_ACTIVE_STORE = `00` ‖ StoreData in LLR-d925uu's order:
//   persona  — "p-alice", org_id Some(aa×20), name "Alice", surname "Smith", handle Some("alice"),
//              member_seed 11×32, device_key DA, member_id Some(01×32), status Active (`01`);
//   record   — Some: org_id aa×20, root 33×32, org_pub_key K, epoch 3, last_seq 3,
//              members [alice (01×32, "alice", "Alice", "Smith", K, [DA]), bob (02×32, "bob", "Bob",
//              "Jones", MB, [DB])], proxy Some(55×32), org_private_key 44×32,
//              kept_change_set Some([9,8,7,6]), kept_base_snapshot Some([5,4,3,2,1]);
//   org_name "Acme";
//   provisional_changes [Admit { ee×32, "carol" }, Revoke { 02×32 }];
//   provisional_organisations — Own (epoch 4, base 33×32, [alice], root 66×32, K, 99×32,
//              Some([9,8,7,6]), Some([5,4,3,2,1]), Own, applied [Revoke { 02×32 }], first_ahead_at None);
//              Received (epoch 4, base 33×32, [alice, bob], root 77×32, K, 88×32, Some([1,2]),
//              Some([3]), Received { DB }, [], None); Answer (epoch 5, base 33×32, [alice], root 22×32, K,
//              5b×32, None, None, Answer { DB }, [], None);
//              each ends with `first_ahead_at` (LLR-s3vq6d, LLR-4tsgfh), here `00` (None), after `applied`
//              (added 2026-10-30, merge review MR3-N1: three `00` tags, 2105 → 2108 bytes);
//   founding None; member None;
//   kept_admission_requests [aa×20, ee×32, from DC, "Carol", "King", ["carol", "cking"], MB, DC,
//              ["192.0.2.1:4433"]];
//   kept_revocation_notices [aa×20, 02×32, DB, proof];
//   expected_admissions [cc×20]; outstanding_invites [aa×20, ee×32, "Carol King"];
//   revoked_devices [03×32, DD]; address_book [(DB, ["198.51.100.7:4433"])].
// GOLDEN_REMOVED_STORE = `01` ‖ org_id aa×20 ‖ verified_at 1_760_000_000 (`80f09dc706`) ‖ two kept
//   acknowledgements, each ack (aa×20, 01×32, DA, epoch 4, root 66×32, signature 5a×64): the first for DB
//   with Some(["198.51.100.7:4433"]), the second for DC with None.
// The bytes are derived by the postcard format rule, field by field, from the values above, by a
// pure-Python script with no org-node code, and not captured from the code.
const GOLDEN_ACTIVE_STORE: &str = "0007702d616c69636501aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa05416c69636505536d6974680105616c6963651111111111111111111111111111111111111111111111111111111111111111204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c210101010101010101010101010101010101010101010101010101010101010101010101aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa3333333333333333333333333333333333333333333333333333333333333333d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737030302010101010101010101010101010101010101010101010101010101010101010105616c69636505416c69636505536d697468d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21020202020202020202020202020202020202020202020202020202020202020203626f6203426f62054a6f6e6573884b8857f4eaa1613c61504db34d4beaf346517a0e31de3cddd4d9b4201d9d0b01a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f00155555555555555555555555555555555555555555555555555555555555555554444444444444444444444444444444444444444444444444444444444444444010409080706010505040302010441636d650200eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee056361726f6c0102020202020202020202020202020202020202020202020202020202020202020304333333333333333333333333333333333333333333333333333333333333333301010101010101010101010101010101010101010101010101010101010101010105616c69636505416c69636505536d697468d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c216666666666666666666666666666666666666666666666666666666666666666d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873799999999999999999999999999999999999999999999999999999999999999990104090807060105050403020100010102020202020202020202020202020202020202020202020202020202020202020004333333333333333333333333333333333333333333333333333333333333333302010101010101010101010101010101010101010101010101010101010101010105616c69636505416c69636505536d697468d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21020202020202020202020202020202020202020202020202020202020202020203626f6203426f62054a6f6e6573884b8857f4eaa1613c61504db34d4beaf346517a0e31de3cddd4d9b4201d9d0b01a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f07777777777777777777777777777777777777777777777777777777777777777d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873788888888888888888888888888888888888888888888888888888888888888880102010201010301a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f0000005333333333333333333333333333333333333333333333333333333333333333301010101010101010101010101010101010101010101010101010101010101010105616c69636505416c69636505536d697468d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c212222222222222222222222222222222222222222222222222222222222222222d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c97787375b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b5b000002a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f00000000001aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeca93ac1705187071d67b83c7ff0efe8108e8ec4530575d7726879333dbdabe7c054361726f6c044b696e6702056361726f6c05636b696e67884b8857f4eaa1613c61504db34d4beaf346517a0e31de3cddd4d9b4201d9d0bca93ac1705187071d67b83c7ff0efe8108e8ec4530575d7726879333dbdabe7c010e3139322e302e322e313a3434333301aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0202020202020202020202020202020202020202020202020202020202020202a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f0008002200000000000000000000000000000000000000000000000000000000000000000000001cccccccccccccccccccccccccccccccccccccccc01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee0a4361726f6c204b696e67010303030303030303030303030303030303030303030303030303030303030303ed4928c628d1c2c6eae90338905995612959273a5c63f93636c14614ac8737d101a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f001113139382e35312e3130302e373a34343333";
const GOLDEN_REMOVED_STORE: &str = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa80f09dc70602aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0101010101010101010101010101010101010101010101010101010101010101204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21046666666666666666666666666666666666666666666666666666666666666666405a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5aa09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f00101113139382e35312e3130302e373a34343333aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0101010101010101010101010101010101010101010101010101010101010101204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21046666666666666666666666666666666666666666666666666666666666666666405a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5aca93ac1705187071d67b83c7ff0efe8108e8ec4530575d7726879333dbdabe7c00";

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unhex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).unwrap())
        .collect()
}

/// The value as serde's data model sees it, field names included.
fn model<Encodable: serde::Serialize>(encodable: &Encodable) -> Value {
    serde_json::to_value(encodable).unwrap()
}

/// verifies: LLR-mkvx2k, LLR-d925uu, LLR-6zaeb6, LLR-p8vpgz, LLR-s3vq6d
#[test]
fn persona_store_plaintext_is_pinned_in_its_active_form() {
    let bytes = unhex(GOLDEN_ACTIVE_STORE);
    assert_eq!(bytes.len(), 2108);
    let content: StoreContent = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(hex(&postcard::to_allocvec(&content).unwrap()), GOLDEN_ACTIVE_STORE);

    // serde_json's map (no `preserve_order`) lists keys sorted, so this checks
    // the set of fields; their order is pinned by the bytes above and by the
    // named-field reads below.
    let content_model = model(&content);
    let store_data = &content_model["Active"];
    let fields: Vec<&str> = store_data.as_object().unwrap().keys().map(String::as_str).collect();
    let mut expected = [
        "persona", "record", "org_name", "provisional_changes", "provisional_organisations", "founding", "member",
        "kept_admission_requests", "kept_revocation_notices", "expected_admissions", "outstanding_invites",
        "revoked_devices", "address_book",
    ];
    expected.sort_unstable();
    assert_eq!(fields, expected, "LLR-d925uu's fields; no behind marker, no provisional_updates");
    let persona = &store_data["persona"];
    assert_eq!(persona["persona_id"], json!("p-alice"));
    assert_eq!(persona["handle"], json!("alice"));
    assert_eq!(persona["name"], json!("Alice"));
    assert_eq!(persona["member_seed"][31], json!(0x11));
    assert_eq!(persona["device_key"][0], json!(0x20));
    assert_eq!(persona["status"], json!("Active"));
    let record = &store_data["record"];
    assert_eq!(record["epoch"], json!(3));
    assert_eq!(record["proxy_account"][0], json!(0x55));
    assert_eq!(record["org_private_key"][0], json!(0x44), "every record holds the key (LLR-78jfrd)");
    assert_eq!(record["kept_change_set"], json!([9, 8, 7, 6]));
    assert_eq!(record["kept_base_snapshot"], json!([5, 4, 3, 2, 1]), "LLR-p8vpgz");
    assert_eq!(record["trie_members"][1]["handle"], json!("bob"));
    assert_eq!(store_data["org_name"], json!("Acme"));
    assert_eq!(store_data["provisional_changes"][0]["Admit"]["handle"], json!("carol"));
    assert_eq!(store_data["provisional_changes"][1]["Revoke"]["member_id"][0], json!(0x02));
    let organisations = &store_data["provisional_organisations"];
    assert_eq!(organisations[0]["origin"], json!("Own"));
    assert_eq!(organisations[0]["applied"][0]["Revoke"]["member_id"][0], json!(0x02));
    assert_eq!(organisations[1]["origin"]["Received"]["from"][0], json!(0xa0));
    assert_eq!(organisations[2]["origin"]["Answer"]["from"][0], json!(0xa0));
    assert_eq!(organisations[2]["change_set"], json!(null));
    for index in 0..3 {
        assert_eq!(organisations[index]["first_ahead_at"], json!(null), "LLR-4tsgfh's stamp, unset");
    }
    assert_eq!(organisations[1]["org_private_key"][0], json!(0x88));
    assert_eq!(store_data["founding"], json!(null));
    assert_eq!(store_data["member"], json!(null));
    let request = &store_data["kept_admission_requests"][0];
    assert_eq!(request["from"][0], json!(0xca));
    assert_eq!(request["preferred_handles"], json!(["carol", "cking"]));
    assert_eq!(request["addresses"], json!(["192.0.2.1:4433"]));
    assert_eq!(store_data["kept_revocation_notices"][0]["member_id"][0], json!(0x02));
    assert_eq!(store_data["expected_admissions"][0]["org_id"][0], json!(0xcc));
    assert_eq!(store_data["outstanding_invites"][0]["invitee_name"], json!("Carol King"));
    assert_eq!(store_data["revoked_devices"][0]["device_key"][0], json!(0xed));
    assert_eq!(store_data["address_book"][0][1], json!(["198.51.100.7:4433"]));
}

/// verifies: LLR-mkvx2k, LLR-6zaeb6
#[test]
fn a_removed_store_is_pinned_in_its_removed_form() {
    let bytes = unhex(GOLDEN_REMOVED_STORE);
    assert_eq!(bytes.len(), 476);
    let content: StoreContent = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(hex(&postcard::to_allocvec(&content).unwrap()), GOLDEN_REMOVED_STORE);
    let content_model = model(&content);
    let removed = &content_model["Removed"];
    let fields: Vec<&str> = removed.as_object().unwrap().keys().map(String::as_str).collect();
    // Keys listed sorted (see above); the order is pinned by the bytes.
    assert_eq!(fields, ["acknowledgements", "org_id", "verified_at"], "nothing else is kept");
    assert_eq!(removed["verified_at"], json!(1_760_000_000u64));
    let first = &removed["acknowledgements"][0];
    let kept: Vec<&str> = first.as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(kept, ["ack", "recipient", "recipient_addresses"]);
    assert_eq!(first["ack"]["epoch"], json!(4));
    assert_eq!(first["recipient"][0], json!(0xa0));
    assert_eq!(first["recipient_addresses"], json!(["198.51.100.7:4433"]));
    assert_eq!(removed["acknowledgements"][1]["recipient"][0], json!(0xca));
    assert_eq!(removed["acknowledgements"][1]["recipient_addresses"], json!(null));
}

/// verifies: LLR-mkvx2k, LLR-g3wmen, LLR-xr88a8
#[test]
fn organisation_information_wire_message_is_pinned() {
    let body = unhex(GOLDEN_WIRE);
    let msg = decode_message(&body).unwrap();
    assert_eq!(hex(&encode_message(&msg).unwrap()), GOLDEN_WIRE);

    let message_model = model(&msg);
    let information = &message_model["OrgInformation"];
    assert_eq!(information["envelope"]["org_id"][0], json!(0xaa));
    assert_eq!(information["envelope"]["parent_seq"], json!(2));
    assert_eq!(information["org_private_key"][31], json!(0x44));
    assert!(information.get("invite_id").is_none() && information.get("org_secret").is_none());

    let WireMessage::OrgInformation { record_snapshot, .. } = &msg else { panic!("Organisation information") };
    let members: Vec<MemberSnapshot> = postcard::from_bytes(record_snapshot).unwrap();
    assert_eq!(postcard::to_allocvec(&members).unwrap(), *record_snapshot);
    let admin = &model(&members)[0];
    assert_eq!(admin["handle"], json!("admin"));
    assert_eq!(admin["member_key"][0], json!(0xd0));
    assert!(first_admission_base(record_snapshot).is_ok(), "the pinned snapshot rebuilds a record");
}

// A revocation holds a notice and no Envelope. The notice's proof is in the
// sparse-merkle-tree form: 38 bytes.
/// verifies: LLR-mkvx2k, LLR-g3wmen, LLR-xr88a8, LLR-dc45ur
#[test]
fn revocation_wire_message_is_pinned() {
    let body = unhex(GOLDEN_REVOCATION);
    let msg = decode_message(&body).unwrap();
    assert_eq!(hex(&encode_message(&msg).unwrap()), GOLDEN_REVOCATION);
    let message_model = model(&msg);
    let notice = message_model["Revocation"].as_object().unwrap();
    let fields: Vec<&str> = notice.keys().map(String::as_str).collect();
    assert_eq!(fields.len(), 4, "org_id, member_id, device and proof, nothing else: {fields:?}");
    assert_eq!(notice["org_id"][0], json!(0xaa));
    assert!(notice.get("envelope").is_none() && notice.get("org_private_key").is_none());
    assert_eq!(body.len(), 123, "the variant index, 20 + 32 + 32 identity bytes and a 38-byte proof");
}

/// Abnormal input: a pinned value cut short is refused, never decoded into
/// something else: both store plaintexts and both Wire messages.
/// verifies: LLR-mkvx2k
#[test]
fn truncated_pinned_values_are_refused() {
    for pinned in [GOLDEN_ACTIVE_STORE, GOLDEN_REMOVED_STORE] {
        let store = unhex(pinned);
        assert!(postcard::from_bytes::<StoreContent>(&store[..store.len() - 1]).is_err());
    }
    let body = unhex(GOLDEN_WIRE);
    assert!(decode_message(&body[..body.len() - 1]).is_err());
    let body = unhex(GOLDEN_REVOCATION);
    assert!(decode_message(&body[..body.len() - 1]).is_err());
}

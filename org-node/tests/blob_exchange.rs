#![cfg(feature = "app")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Out-of-band exchange blobs (SDD-vee2fq).
//!
//! Relocated from the `#[cfg(test)]` module in `org-node/src/blobs.rs` so the
//! `verifies:` annotations sit under `test_paths`.

use base64::{engine::general_purpose::STANDARD, Engine};
use org_node::blobs::{decode, encode, Invite, JoinRequest};
use org_node::ids::OrgId;

fn an_invite() -> Invite {
    Invite {
        org_id: OrgId::new([0xabu8; 20]),
        org_pub_key: [1u8; 32],
        admin_member_key: [2u8; 32],
        admin_device_key: [3u8; 32],
        admin_node_addr: vec![4, 5, 6],
    }
}

fn a_join_request() -> JoinRequest {
    JoinRequest {
        handle: "bob".into(),
        name: "Bob".into(),
        surname: "Builder".into(),
        member_key: [7u8; 32],
        device_key: [8u8; 32],
        node_addr: vec![9, 10],
    }
}

// verifies: REQ-xa6smf, LLR-g9vmbx
#[test]
fn an_invite_round_trips_carrying_every_field() {
    let original = an_invite();
    let back: Invite = decode(&encode(&original).unwrap()).unwrap();
    assert_eq!(back, original);
    // Named individually, so a field dropped from the struct is a failure here
    // and not merely a changed equality.
    assert_eq!(back.org_id, original.org_id);
    assert_eq!(back.org_pub_key, original.org_pub_key);
    assert_eq!(back.admin_member_key, original.admin_member_key);
    assert_eq!(back.admin_device_key, original.admin_device_key);
    assert_eq!(back.admin_node_addr, original.admin_node_addr);
}

// verifies: REQ-xa6smf, LLR-kkj64b
#[test]
fn a_join_request_round_trips_carrying_every_field() {
    let original = a_join_request();
    let back: JoinRequest = decode(&encode(&original).unwrap()).unwrap();
    assert_eq!(back, original);
    assert_eq!(back.handle, original.handle);
    assert_eq!(back.member_key, original.member_key);
    assert_eq!(back.device_key, original.device_key);
    assert_eq!(back.node_addr, original.node_addr);
}

// verifies: REQ-9g6as6, LLR-8qxwst
#[test]
fn a_string_that_is_not_base64_is_refused() {
    let result: Result<Invite, _> = decode("not!valid!base64!!!");
    let rendered = format!("{}", result.unwrap_err());
    // The verdict must be the armour's, not the payload decoder's. `is_err()`
    // alone would be satisfied either way: the sweep of 2026-10-03 replaced
    // the base64 failure with `unwrap_or_default()`, so bad armour yielded
    // empty bytes that postcard then refused — a different refusal, for a
    // different reason, with the same shape.
    assert!(
        rendered.contains("base64"),
        "a base64 failure must be reported as one, got: {rendered}"
    );
}

// verifies: REQ-9g6as6, LLR-tcft2r
#[test]
fn valid_base64_that_is_not_a_blob_is_refused() {
    let armoured = STANDARD.encode(b"garbage bytes that are not valid postcard");
    let result: Result<Invite, _> = decode(&armoured);
    assert!(result.is_err());
}

// verifies: REQ-9g6as6, LLR-tcft2r
#[test]
fn an_invite_does_not_decode_as_a_join_request() {
    // The two blobs share an encoding; the type is the only thing telling them
    // apart, so a mismatch must be an error rather than a reinterpretation.
    let armoured = encode(&an_invite()).unwrap();
    let as_join: Result<JoinRequest, _> = decode(&armoured);
    assert!(as_join.is_err(), "an Invite decoded as a JoinRequest");
}

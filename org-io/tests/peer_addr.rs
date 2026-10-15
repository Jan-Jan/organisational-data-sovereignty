#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The peer-address parse (SDD-wr8m5n, split from SDD-v2bq89 2026-10-15; S4, task T12): a peer address as
//! text, blank for none, otherwise the hexadecimal of a postcard-encoded
//! endpoint address after at most one `0x`. The parse moved from the app's
//! commands (`parse_peer_addr`). *2026-10-15 (S4, TEST review finding-4):*
//! the app's peer-address cases that task T12 rewrote onto the parse here
//! were removed; the items they verified state what the app's commands do
//! with the text and moved back to the app, whose IPC tests verify them.

use org_io::peer_addr::{parse_peer_addr, PeerAddr, PeerAddrError};

/// A well-formed endpoint address carrying an identity and no transport
/// address, as the app's `encoded_endpoint_addr` built it.
fn endpoint_addr() -> iroh::EndpointAddr {
    let id = iroh::SecretKey::from_bytes(&[0x2a; 32]).public();
    id.into()
}

fn encoded_endpoint_addr() -> Vec<u8> {
    postcard::to_allocvec(&endpoint_addr()).expect("an EndpointAddr encodes")
}

// verifies: LLR-2b2ghe
#[test]
fn blank_text_is_no_address() {
    for blank in ["", " ", "\n", "\t \r\n"] {
        assert_eq!(parse_peer_addr(blank).unwrap(), None, "{blank:?}");
    }
}

// verifies: LLR-2b2ghe
#[test]
fn bare_and_zero_x_prefixed_hex_of_an_endpoint_addr_is_that_address() {
    let hex_text = hex::encode(encoded_endpoint_addr());
    let expected = PeerAddr::for_test(endpoint_addr());
    for text in [hex_text.clone(), format!("0x{hex_text}"), format!("  {hex_text}\n"), hex_text.to_uppercase()] {
        assert_eq!(parse_peer_addr(&text).unwrap(), Some(expected.clone()), "{text:?}");
    }
    assert_eq!(expected.endpoint_id(), *endpoint_addr().id.as_bytes(), "the address names the identity it encodes");
}

// verifies: LLR-2b2ghe
#[test]
fn text_that_is_not_hex_is_refused_naming_the_field() {
    let hex_text = hex::encode(encoded_endpoint_addr());
    for bad in ["zz".to_string(), "abc".to_string(), "0".to_string(), format!("{hex_text}0"), format!("0x0x{hex_text}")] {
        let refused = parse_peer_addr(&bad).unwrap_err();
        assert!(matches!(refused, PeerAddrError::Hex(_)), "{bad:?}: {refused:?}");
        assert!(refused.to_string().starts_with("peer_addr"), "names the field: {refused}");
    }
}

// verifies: LLR-2b2ghe
#[test]
fn hex_of_bytes_that_are_not_exactly_an_endpoint_addr_is_refused_naming_the_field() {
    let trailing = format!("{}00", hex::encode(encoded_endpoint_addr()));
    // A bare `0x` is not blank: it is the hexadecimal of no bytes, which
    // encode no address.
    for bad in ["00", "ff", "deadbeef", trailing.as_str(), "0x"] {
        let refused = parse_peer_addr(bad).unwrap_err();
        assert!(matches!(refused, PeerAddrError::Decode(_)), "{bad:?}: {refused:?}");
        assert!(refused.to_string().starts_with("peer_addr"), "names the field: {refused}");
    }
}

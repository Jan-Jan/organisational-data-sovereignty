#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! verifies: LLR-2b2ghe
//! The peer-address parse is total (S4, task T12): on any text it returns
//! no address, an address, or a refusal naming the field, and never panics.
//! A text that parses to an address is re-encoded and parsed again to the
//! same address.

use org_io::peer_addr::{parse_peer_addr, PeerAddrError};

fn main() {
    bolero::check!().with_type::<String>().for_each(|input: &String| match parse_peer_addr(input) {
        Ok(None) => assert!(input.trim().is_empty(), "only blank text is no address: {input:?}"),
        Ok(Some(addr)) => {
            let again = parse_peer_addr(&addr.to_hex()).expect("a parsed address re-encodes");
            assert_eq!(again, Some(addr));
        }
        Err(refused) => {
            assert!(matches!(refused, PeerAddrError::Hex(_) | PeerAddrError::Decode(_)));
            assert!(refused.to_string().starts_with("peer_addr"), "names the field: {refused}");
        }
    });
}

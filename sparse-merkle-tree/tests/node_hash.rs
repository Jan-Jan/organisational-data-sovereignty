//! NodeHash: the 32-byte hash type the tree is built from.

#![allow(clippy::expect_used)]

use sparse_merkle_tree::NodeHash;

/// verifies: LLR-jd4phg
#[test]
fn a_node_hash_wraps_any_32_bytes() {
    let mut bytes = [0xff; 32];
    bytes[0] = 0x01;
    bytes[1] = 0xab;
    let hash = NodeHash::from(bytes);
    assert_eq!(hash.as_bytes(), &bytes);
    assert_eq!(hash, NodeHash::new(bytes));
    assert_eq!(format!("{hash:?}"), "NodeHash(01abffff..)");
    assert_eq!(format!("{:?}", NodeHash::new([0; 32])), "NodeHash(00000000..)");
}

/// verifies: LLR-jd4phg
#[cfg(feature = "serde")]
#[test]
fn a_node_hash_encodes_as_its_32_bytes() {
    let hash = NodeHash::new([7; 32]);
    let encoded = postcard::to_allocvec(&hash).expect("encode");
    assert_eq!(encoded, [7u8; 32].to_vec());
    assert_eq!(postcard::from_bytes::<NodeHash>(&encoded).expect("decode"), hash);
}

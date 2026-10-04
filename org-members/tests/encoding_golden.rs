//! Pins the wire bytes of a member record and the root of a member set, so a
//! type-level refactor cannot silently change what is signed or hashed.
//! Values captured on master @ 0f85cb9 (2026-10-04), before the newtype
//! refactor. Never update them to make this test pass.
use ed25519_dalek::SigningKey;
use org_members::hasher::Blake3Hasher;
use org_members::trie::OrgTrie;
use org_members::types::{Handle, MemberId, MemberLeaf, Name, P2pDeviceKey, P2pMemberKey, Surname};

const GOLDEN_ALICE_LEAF: &str = "902e667049de9cc485facf64b9a88b7ddf9ce1b5f4e7fb61aa7146e201d644bd05616c6963658092f31ae728a56911fc508958c73fadbe6a9b050aec7114e4412a3dc87298c805416c69636505536d697468015726e5bd4b887fcc929566a772f0fd5eb37d5695b4b60d4b298886b53c0bc567";
const GOLDEN_ROOT: &str = "2e81afbd4e9425357c2c26b6bb485e08d60b1a0606a02a8f8f9e76ac7282a19d";

fn seed(s: &str) -> [u8; 32] {
    blake3::hash(s.as_bytes()).into()
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

// LEAF-CONSTRUCTION: the only lines T4 may change in this file.
fn leaf(tag: &str, handle: &str, name: &str, surname: &str) -> MemberLeaf {
    MemberLeaf::new(
        MemberId::new(seed(&format!("{tag}-id"))),
        Handle::parse(handle).unwrap(),
        P2pMemberKey::new(SigningKey::from_bytes(&seed(&format!("{tag}-mk"))).verifying_key()),
        Name::parse(name).unwrap(),
        Surname::parse(surname).unwrap(),
        vec![P2pDeviceKey::new(SigningKey::from_bytes(&seed(&format!("{tag}-d1"))).verifying_key())],
    )
    .unwrap()
}

/// verifies: LLR-377ddr
#[test]
fn member_record_wire_bytes_are_pinned() {
    let alice = leaf("alice", "alice", "Alice", "Smith");
    assert_eq!(hex(&postcard::to_allocvec(&alice).unwrap()), GOLDEN_ALICE_LEAF);
    let back: MemberLeaf = postcard::from_bytes(&postcard::to_allocvec(&alice).unwrap()).unwrap();
    assert_eq!(back, alice);
}

/// verifies: LLR-377ddr
#[test]
fn member_set_root_is_pinned() {
    let alice = leaf("alice", "alice", "Alice", "Smith");
    let bob = leaf("bob", "bob", "Bob", "Jones");
    let trie = OrgTrie::<Blake3Hasher>::genesis(vec![alice, bob]).unwrap();
    assert_eq!(hex(trie.root_hash().unwrap().as_bytes()), GOLDEN_ROOT);
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}

/// verifies: LLR-377ddr, LLR-68tka5
///
/// Abnormal side: the pinned record with its handle's first byte flipped
/// from 'a' to 'A' is refused on decode, and a one-character change to the
/// handle changes the encoded bytes.
#[test]
fn tampered_member_record_is_refused_and_handle_is_bound_in_bytes() {
    let at = GOLDEN_ALICE_LEAF.find("05616c696365").expect("length-prefixed handle in golden");
    assert_eq!(GOLDEN_ALICE_LEAF.matches("05616c696365").count(), 1);
    let first_byte = at + 2;
    let mut tampered = String::from(GOLDEN_ALICE_LEAF);
    tampered.replace_range(first_byte..first_byte + 2, "41");
    assert!(postcard::from_bytes::<MemberLeaf>(&unhex(GOLDEN_ALICE_LEAF)).is_ok());
    assert!(postcard::from_bytes::<MemberLeaf>(&unhex(&tampered)).is_err());

    let alicf = leaf("alice", "alicf", "Alice", "Smith");
    assert_ne!(hex(&postcard::to_allocvec(&alicf).unwrap()), GOLDEN_ALICE_LEAF);
}

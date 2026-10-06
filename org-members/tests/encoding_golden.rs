//! Pins the wire bytes of a member record and the root of a member set, so a
//! type-level refactor cannot silently change what is encoded or hashed.
//! Values captured on master @ 0f85cb9 (2026-10-04), before the newtype
//! refactor. Never update them to make this test pass.
//! Re-derived once (2026-10-04) when the member key became X25519: only the
//! member-key input changed. The evidence is in
//! `docs/plans/2026-10-04-person-shared-types-implementation.md`, the block
//! "Steps 5, 6, 8 for org-members done (9e93bfc)": the old member key (last
//! byte 0xc8, top bit set) is not a canonical X25519 key; the two leaf
//! encodings differ only in the 32 member-key bytes (`8092f31a…98c8` →
//! `bf74dcf9…be78`); `GOLDEN_ROOT` `2e81afbd…a19d` → `4740435a…efd4`; hashing
//! is unchanged (task T8, which showed the move to `person`'s types
//! byte-identical against the 0f85cb9 values before the key changed).
//! `OLD_GOLDEN_ALICE_LEAF` keeps the 0f85cb9 record, and
//! `the_record_differs_from_the_0f85cb9_record_only_in_its_member_key` shows
//! the first of those claims. The old root cannot be recomputed here: the old
//! key is refused by `PersonPublicKey`, so no member set holding it can be
//! built.
mod common;

use common::dual_key_bytes;
use ed25519_dalek::SigningKey;
use org_members::hasher::Blake3Hasher;
use org_members::trie::OrgTrie;
use org_members::types::{Handle, MemberId, MemberLeaf, Name, DevicePublicKey, PersonPublicKey, Surname};

const GOLDEN_ALICE_LEAF: &str = "902e667049de9cc485facf64b9a88b7ddf9ce1b5f4e7fb61aa7146e201d644bd05616c696365bf74dcf9814eaf82aa907e23790285763c28cd82bad0bdb8954a09d67d7dbe7805416c69636505536d697468015726e5bd4b887fcc929566a772f0fd5eb37d5695b4b60d4b298886b53c0bc567";
const GOLDEN_ROOT: &str = "4740435a055ef3825a1a96a9a72361c3bc22bd2c47a992cda21161048d17efd4";
/// The pinned record as captured on master @ 0f85cb9, with the ed25519 member
/// key of the seed "alice-mk".
const OLD_GOLDEN_ALICE_LEAF: &str = "902e667049de9cc485facf64b9a88b7ddf9ce1b5f4e7fb61aa7146e201d644bd05616c6963658092f31ae728a56911fc508958c73fadbe6a9b050aec7114e4412a3dc87298c805416c69636505536d697468015726e5bd4b887fcc929566a772f0fd5eb37d5695b4b60d4b298886b53c0bc567";

fn seed(s: &str) -> [u8; 32] {
    blake3::hash(s.as_bytes()).into()
}


fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

// LEAF-CONSTRUCTION: the only lines a change of key or field types may touch.
fn leaf(tag: &str, handle: &str, name: &str, surname: &str) -> MemberLeaf {
    MemberLeaf::new(
        MemberId::new(seed(&format!("{tag}-id"))),
        Handle::parse(handle).unwrap(),
        PersonPublicKey::parse(&dual_key_bytes(&format!("{tag}-mk"))).expect("dual-valid bytes"),
        Name::parse(name).unwrap(),
        Surname::parse(surname).unwrap(),
        vec![DevicePublicKey::try_from(SigningKey::from_bytes(&seed(&format!("{tag}-d1"))).verifying_key()).expect("prime-order key")],
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

/// verifies: LLR-377ddr
///
/// The record encodes as it did on 0f85cb9 except for the member key: put the
/// old key's bytes where the new key's are and the old pinned bytes come back.
#[test]
fn the_record_differs_from_the_0f85cb9_record_only_in_its_member_key() {
    let alice = leaf("alice", "alice", "Alice", "Smith");
    let mut encoded = postcard::to_allocvec(&alice).unwrap();
    let new_key = alice.p2p_key().as_bytes();
    let at = encoded
        .windows(32)
        .position(|window| window == new_key)
        .expect("the member key is in the record");
    let old_key = SigningKey::from_bytes(&seed("alice-mk")).verifying_key().to_bytes();
    assert!(PersonPublicKey::parse(&old_key).is_err(), "the old key is not an X25519 key");
    encoded[at..at + 32].copy_from_slice(&old_key);
    assert_eq!(hex(&encoded), OLD_GOLDEN_ALICE_LEAF);
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

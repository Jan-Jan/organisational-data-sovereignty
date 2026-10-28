use person::DeviceRootHasher;
use sparse_merkle_tree::{NodeHash, TreeHasher};

/// The Membership record's hashing: the tree's member leaf, node and
/// empty-leaf domains (`TreeHasher`), a MemberId's identity and a member
/// leaf's content (LLR-4rdqyn), and the device root tree's domains through
/// `DeviceRootHasher` (LLR-uudst2).
///
/// Not object-safe (static methods). Use with generics: `OrgTrie<H: MemberHasher>`.
pub trait MemberHasher: TreeHasher + Clone + Send + Sync {
    type DeviceRootHasher: person::DeviceRootHasher;

    /// A MemberId's identity hash (member-identity domain).
    fn member_identity_hash(id: &[u8; 32]) -> NodeHash;

    /// A member leaf's content hash (member-content domain).
    fn member_content_hash(content: &[u8]) -> NodeHash;
}

/// What an empty member position hashes in the member-leaf domain.
pub const MEMBER_EMPTY_SENTINEL: &[u8] = b"EMPTY_SENTINEL_ORG_MEMBERS_V1";

/// `org-members::member-leaf`, padded with `_` to 32 bytes (LLR-4rdqyn).
const MEMBER_LEAF_KEY: &[u8; 32] = b"org-members::member-leaf________";

/// `org-members::member-node`, padded with `_` to 32 bytes (LLR-4rdqyn).
const MEMBER_NODE_KEY: &[u8; 32] = b"org-members::member-node________";

/// `org-members::member-identity`, padded with `_` to 32 bytes (LLR-4rdqyn).
const MEMBER_IDENTITY_KEY: &[u8; 32] = b"org-members::member-identity____";

/// `org-members::member-content`, padded with `_` to 32 bytes (LLR-4rdqyn).
const MEMBER_CONTENT_KEY: &[u8; 32] = b"org-members::member-content_____";

/// `org-members::device-leaf`, padded with `_` to 32 bytes (LLR-4rdqyn).
const DEVICE_LEAF_KEY: &[u8; 32] = b"org-members::device-leaf________";

/// `org-members::device-node`, padded with `_` to 32 bytes (LLR-4rdqyn).
const DEVICE_NODE_KEY: &[u8; 32] = b"org-members::device-node________";

/// `org-members::device-identity`, padded with `_` to 32 bytes (LLR-4rdqyn).
const DEVICE_IDENTITY_KEY: &[u8; 32] = b"org-members::device-identity____";

/// `org-members::device-content`, padded with `_` to 32 bytes (LLR-4rdqyn).
const DEVICE_CONTENT_KEY: &[u8; 32] = b"org-members::device-content_____";

fn keyed(key: &[u8; 32], parts: &[&[u8]]) -> NodeHash {
    let mut hasher = blake3::Hasher::new_keyed(key);
    for part in parts {
        hasher.update(part);
    }
    NodeHash::new(hasher.finalize().into())
}

/// The member domains: blake3 keyed per domain (LLR-4rdqyn).
#[derive(Clone, Debug)]
pub struct Blake3Hasher;

/// The device domains of a Member's device root tree (LLR-jfj6pc, LLR-4rdqyn).
#[derive(Clone, Debug)]
pub struct Blake3DeviceRootHasher;

impl TreeHasher for Blake3Hasher {
    fn leaf(identity_hash: &NodeHash, content_hash: &NodeHash) -> NodeHash {
        keyed(MEMBER_LEAF_KEY, &[identity_hash.as_bytes(), content_hash.as_bytes()])
    }

    fn node(left: &NodeHash, right: &NodeHash) -> NodeHash {
        keyed(MEMBER_NODE_KEY, &[left.as_bytes(), right.as_bytes()])
    }

    fn empty_leaf() -> NodeHash {
        keyed(MEMBER_LEAF_KEY, &[MEMBER_EMPTY_SENTINEL])
    }
}

impl MemberHasher for Blake3Hasher {
    type DeviceRootHasher = Blake3DeviceRootHasher;

    fn member_identity_hash(id: &[u8; 32]) -> NodeHash {
        keyed(MEMBER_IDENTITY_KEY, &[id])
    }

    fn member_content_hash(content: &[u8]) -> NodeHash {
        keyed(MEMBER_CONTENT_KEY, &[content])
    }
}

impl TreeHasher for Blake3DeviceRootHasher {
    fn leaf(identity_hash: &NodeHash, content_hash: &NodeHash) -> NodeHash {
        keyed(DEVICE_LEAF_KEY, &[identity_hash.as_bytes(), content_hash.as_bytes()])
    }

    fn node(left: &NodeHash, right: &NodeHash) -> NodeHash {
        keyed(DEVICE_NODE_KEY, &[left.as_bytes(), right.as_bytes()])
    }

    fn empty_leaf() -> NodeHash {
        keyed(DEVICE_LEAF_KEY, &[Self::DEVICE_EMPTY_SENTINEL])
    }
}

impl DeviceRootHasher for Blake3DeviceRootHasher {
    const DEVICE_EMPTY_SENTINEL: &'static [u8] = b"EMPTY_SENTINEL_ORG_MEMBERS_DEVICE_V1";

    fn device_identity_hash(key: &[u8; 32]) -> NodeHash {
        keyed(DEVICE_IDENTITY_KEY, &[key])
    }

    fn device_content_hash() -> NodeHash {
        keyed(DEVICE_CONTENT_KEY, &[])
    }
}

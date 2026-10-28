//! The Membership record's leaf on the `sparse-merkle-tree` unit
//! (SDD-d9svdj, LLR-t465mb): a `MemberLeaf` held with its device root.

use alloc::vec::Vec;

use person::compute_device_root;
use sparse_merkle_tree::{Leaf, NodeHash, SparseMerkleTree};

use crate::error::OrgMembersError;
use crate::hasher::MemberHasher;
use crate::types::{Handle, MemberId, MemberLeaf, Name, PersonPublicKey, Surname};

/// The Membership record's depth: a MemberId's 256 bits (LLR-zbe553).
pub const MEMBER_DEPTH: usize = 256;

/// A member leaf and its device root, computed when it enters the tree.
/// LLR-t465mb.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct TreeMember {
    member: MemberLeaf,
    device_root: NodeHash,
}

impl TreeMember {
    pub(crate) fn new<H: MemberHasher>(member: MemberLeaf) -> Result<Self, OrgMembersError> {
        let device_root = compute_device_root::<H::DeviceRootHasher>(member.p2p_devices())?;
        Ok(Self { member, device_root })
    }

    pub(crate) fn member(&self) -> &MemberLeaf {
        &self.member
    }

    pub(crate) fn device_root(&self) -> &NodeHash {
        &self.device_root
    }
}

impl<H: MemberHasher> Leaf<H> for TreeMember {
    type Identity = MemberId;

    fn identity(&self) -> &MemberId {
        self.member.id()
    }

    fn position_bytes(identity: &MemberId) -> [u8; 32] {
        member_position_bytes(identity)
    }

    fn identity_hash(identity: &MemberId) -> NodeHash {
        member_identity_hash::<H>(identity)
    }

    fn content_hash(&self) -> NodeHash {
        member_content_hash::<H>(
            self.member.handle(),
            self.member.p2p_key(),
            self.member.name(),
            self.member.surname(),
            &self.device_root,
        )
    }
}

pub(crate) type MemberTree<H> = SparseMerkleTree<TreeMember, H, MEMBER_DEPTH>;

/// A Member's position bytes: its MemberId's 32 bytes (LLR-zbe553).
pub(crate) fn member_position_bytes(id: &MemberId) -> [u8; 32] {
    *id.as_bytes()
}

/// A MemberId's identity hash, in the member-identity domain. LLR-t465mb.
pub(crate) fn member_identity_hash<H: MemberHasher>(id: &MemberId) -> NodeHash {
    H::member_identity_hash(id.as_bytes())
}

/// The member-content hash of a leaf's canonical bytes without its
/// MemberId: handle, member key, name, surname and device root, each string
/// prefixed by its u32 little-endian byte length. LLR-t465mb.
pub(crate) fn member_content_hash<H: MemberHasher>(
    handle: &Handle,
    p2p_key: &PersonPublicKey,
    name: &Name,
    surname: &Surname,
    device_root: &NodeHash,
) -> NodeHash {
    let mut bytes = Vec::new();
    let handle = handle.as_str().as_bytes();
    bytes.extend_from_slice(&(handle.len() as u32).to_le_bytes());
    bytes.extend_from_slice(handle);
    bytes.extend_from_slice(p2p_key.as_bytes());
    let name = name.as_str().as_bytes();
    bytes.extend_from_slice(&(name.len() as u32).to_le_bytes());
    bytes.extend_from_slice(name);
    let surname = surname.as_str().as_bytes();
    bytes.extend_from_slice(&(surname.len() as u32).to_le_bytes());
    bytes.extend_from_slice(surname);
    bytes.extend_from_slice(device_root.as_bytes());
    H::member_content_hash(&bytes)
}

/// The member under `id` in `tree`.
pub(crate) fn member_of<H: MemberHasher>(tree: &MemberTree<H>, id: &MemberId) -> Option<MemberLeaf> {
    tree.get(id).map(|held| held.member().clone())
}

/// `tree` with every hash calculated; a tree with none unset as it is.
/// `from_leaves` keeps the same rule inside the tree unit, which exports no
/// such method: its `calculate` refuses a calculated tree.
pub(crate) fn calculated<H: MemberHasher>(tree: MemberTree<H>) -> Result<MemberTree<H>, OrgMembersError> {
    if tree.is_calculated() {
        Ok(tree)
    } else {
        Ok(tree.calculate()?)
    }
}

/// The MemberIds `before` holds and `after` does not, and the member leaves
/// `after` holds that `before` does not hold as they are, each in increasing
/// MemberId order (LLR-8jttpb).
pub(crate) fn changes_between<H: MemberHasher>(before: &MemberTree<H>, after: &MemberTree<H>) -> (Vec<MemberId>, Vec<MemberLeaf>) {
    let (removed, upserted) = MemberTree::<H>::changes_between(before, after);
    (removed, upserted.iter().map(|held| held.member().clone()).collect())
}

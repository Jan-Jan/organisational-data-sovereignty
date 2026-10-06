use alloc::sync::Arc;
use alloc::vec::Vec;

use person::compute_device_root;

use crate::error::OrgMembersError;
use crate::hasher::TrieHasher;
use crate::node::{Node, NodeKind};
use crate::proof::ProofEnding;
use crate::types::{MemberId, MemberLeaf, NodeHash};

/// Depth of the Sparse Merkle Tree (256 bits = 256 levels).
pub const SMT_DEPTH: u16 = 256;

const MEMBER_EMPTY_SENTINEL: &[u8] = b"EMPTY_SENTINEL_ORG_MEMBERS_V1";

/// Precomputed default hashes for each level of the SMT.
pub struct DefaultHashes {
    hashes: Vec<NodeHash>,
}

impl DefaultHashes {
    pub fn compute<H: TrieHasher>() -> Self {
        let mut hashes = Vec::with_capacity(SMT_DEPTH as usize + 1);
        let leaf_hash = H::hash_member_leaf(MEMBER_EMPTY_SENTINEL);
        hashes.push(leaf_hash);
        for i in 1..=SMT_DEPTH as usize {
            let prev = &hashes[i - 1];
            let h = H::hash_member_node(prev, prev);
            hashes.push(h);
        }
        Self { hashes }
    }

    /// The default hash at `level` (0 = a leaf, 256 = the root), or
    /// `IndexOutOfRange` for a level of 257 or more (LLR-wm5hpc, LLR-7jkcba).
    pub fn at_level(&self, level: u16) -> Result<&NodeHash, OrgMembersError> {
        self.hashes
            .get(usize::from(level))
            .ok_or(OrgMembersError::IndexOutOfRange)
    }

    pub fn empty_leaf(&self) -> &NodeHash {
        &self.hashes[0]
    }
}

pub fn empty_root(defaults: &DefaultHashes) -> Result<Arc<Node>, OrgMembersError> {
    Ok(Arc::new(Node::empty(*defaults.at_level(SMT_DEPTH)?)))
}

/// Inserts a member leaf at the position determined by its id bits.
pub fn insert<H: TrieHasher>(
    root: &Arc<Node>,
    member: MemberLeaf,
    defaults: &DefaultHashes,
) -> Result<Arc<Node>, OrgMembersError> {
    let id = *member.id();
    let device_root = compute_device_root::<H>(member.p2p_device_slots());
    let new_leaf = Arc::new(Node::leaf(member, device_root));
    let mut bits = id.path_bits();
    insert_at(root, &mut bits, new_leaf, 0, defaults)
}

/// Removes a member at the position determined by the id bits.
pub fn remove(
    root: &Arc<Node>,
    id: &MemberId,
    defaults: &DefaultHashes,
) -> Result<Arc<Node>, OrgMembersError> {
    let empty_leaf = Arc::new(Node::empty(*defaults.empty_leaf()));
    insert_at(root, &mut id.path_bits(), empty_leaf, 0, defaults)
}

fn insert_at(
    node: &Arc<Node>,
    bits: &mut dyn Iterator<Item = bool>,
    new_leaf: Arc<Node>,
    depth: u16,
    defaults: &DefaultHashes,
) -> Result<Arc<Node>, OrgMembersError> {
    if depth == SMT_DEPTH {
        return Ok(new_leaf);
    }
    let go_right = bits.next().ok_or(OrgMembersError::InvariantViolated)?;
    let (left, right) = match &node.kind {
        NodeKind::Internal { left, right } => (left.clone(), right.clone()),
        NodeKind::Empty | NodeKind::Leaf(_) => {
            let default_child = Arc::new(Node::empty(*defaults.at_level(SMT_DEPTH - depth - 1)?));
            (default_child.clone(), default_child)
        }
    };
    let (new_left, new_right) = if go_right {
        (left, insert_at(&right, bits, new_leaf, depth + 1, defaults)?)
    } else {
        (insert_at(&left, bits, new_leaf, depth + 1, defaults)?, right)
    };
    Ok(Arc::new(Node::internal(new_left, new_right)))
}

/// Recursively computes hashes for all nodes with empty OnceCell.
/// Returns Err if an Empty node is encountered without a precomputed hash
/// (invariant violation -- Empty nodes must always be constructed with a hash).
pub fn recalculate_hashes<H: TrieHasher>(
    node: &Arc<Node>,
) -> Result<NodeHash, OrgMembersError> {
    if let Some(h) = node.hash() {
        return Ok(*h);
    }

    let computed = match &node.kind {
        NodeKind::Internal { left, right } => {
            let left_hash = recalculate_hashes::<H>(left)?;
            let right_hash = recalculate_hashes::<H>(right)?;
            H::hash_member_node(&left_hash, &right_hash)
        }
        NodeKind::Leaf(payload) => {
            let canonical = payload.member.canonical_bytes(&payload.device_root);
            H::hash_member_leaf(&canonical)
        }
        NodeKind::Empty => {
            return Err(OrgMembersError::InvariantViolated);
        }
    };

    node.set_hash(computed);
    Ok(computed)
}

/// Looks up a member by id, traversing the SMT by id bits.
pub fn get_member(root: &Arc<Node>, id: &MemberId) -> Option<MemberLeaf> {
    let mut current = root.clone();
    for go_right in id.path_bits() {
        match &current.kind {
            NodeKind::Internal { left, right } => {
                current = if go_right { right.clone() } else { left.clone() };
            }
            NodeKind::Empty | NodeKind::Leaf(_) => return None,
        }
    }
    match &current.kind {
        NodeKind::Leaf(payload) => Some(payload.member.clone()),
        _ => None,
    }
}

/// The 256 sibling hashes along `id`'s path, index `level` holding the sibling
/// at that level (0 = the leaf's level), and what the path ends in. Below an
/// empty subtree every sibling is its level's default (LLR-wm5hpc). Refuses
/// with `HashesNotCalculated` when a sibling's hash is unset (LLR-utp6x4).
pub(crate) fn path(
    root: &Arc<Node>,
    id: &MemberId,
    defaults: &DefaultHashes,
) -> Result<(Vec<NodeHash>, ProofEnding), OrgMembersError> {
    let mut top_down = Vec::with_capacity(usize::from(SMT_DEPTH));
    let mut bits = id.path_bits();
    let mut current = root.clone();
    let mut depth: u16 = 0;
    let end = loop {
        if depth == SMT_DEPTH {
            break match &current.kind {
                NodeKind::Leaf(payload) => ProofEnding::Leaf(payload.member.clone()),
                NodeKind::Empty => ProofEnding::Empty,
                NodeKind::Internal { .. } => return Err(OrgMembersError::InvariantViolated),
            };
        }
        match &current.kind {
            NodeKind::Internal { left, right } => {
                let go_right = bits.next().ok_or(OrgMembersError::InvariantViolated)?;
                let (next, sibling) = if go_right { (right, left) } else { (left, right) };
                top_down.push(*sibling.hash().ok_or(OrgMembersError::HashesNotCalculated)?);
                current = next.clone();
                depth += 1;
            }
            NodeKind::Empty => {
                while depth < SMT_DEPTH {
                    top_down.push(*defaults.at_level(SMT_DEPTH - depth - 1)?);
                    depth += 1;
                }
                break ProofEnding::Empty;
            }
            NodeKind::Leaf(_) => return Err(OrgMembersError::InvariantViolated),
        }
    };
    top_down.reverse();
    Ok((top_down, end))
}

/// Collects all member leaves in the trie.
pub fn collect_members(root: &Arc<Node>) -> Vec<MemberLeaf> {
    let mut members = Vec::new();
    collect_members_recursive(root, &mut members);
    members
}

fn collect_members_recursive(node: &Arc<Node>, members: &mut Vec<MemberLeaf>) {
    match &node.kind {
        NodeKind::Internal { left, right } => {
            collect_members_recursive(left, members);
            collect_members_recursive(right, members);
        }
        NodeKind::Leaf(payload) => {
            members.push(payload.member.clone());
        }
        NodeKind::Empty => {}
    }
}

/// Computes the diff between two tries. Both must have calculated hashes.
/// Returns (removed_ids, upserted_leaves).
pub fn diff_tries(
    old: &Arc<Node>,
    new: &Arc<Node>,
) -> (Vec<MemberId>, Vec<MemberLeaf>) {
    let mut removed = Vec::new();
    let mut upserted = Vec::new();
    diff_recursive(old, new, &mut removed, &mut upserted);
    (removed, upserted)
}

fn diff_recursive(
    old: &Arc<Node>,
    new: &Arc<Node>,
    removed: &mut Vec<MemberId>,
    upserted: &mut Vec<MemberLeaf>,
) {
    if let (Some(old_h), Some(new_h)) = (old.hash(), new.hash()) {
        if old_h == new_h {
            return;
        }
    }

    match (&old.kind, &new.kind) {
        (
            NodeKind::Internal {
                left: ol,
                right: or,
            },
            NodeKind::Internal {
                left: nl,
                right: nr,
            },
        ) => {
            diff_recursive(ol, nl, removed, upserted);
            diff_recursive(or, nr, removed, upserted);
        }
        (NodeKind::Leaf(old_p), NodeKind::Leaf(new_p)) => {
            if old_p.member != new_p.member {
                upserted.push(new_p.member.clone());
            }
        }
        (NodeKind::Leaf(payload), NodeKind::Empty) => {
            removed.push(*payload.member.id());
        }
        (NodeKind::Empty, NodeKind::Leaf(payload)) => {
            upserted.push(payload.member.clone());
        }
        (NodeKind::Empty, NodeKind::Empty) => {}
        (NodeKind::Internal { left, right }, _) => {
            diff_recursive(left, new, removed, upserted);
            diff_recursive(right, new, removed, upserted);
        }
        (_, NodeKind::Internal { left, right }) => {
            diff_recursive(old, left, removed, upserted);
            diff_recursive(old, right, removed, upserted);
        }
    }
}

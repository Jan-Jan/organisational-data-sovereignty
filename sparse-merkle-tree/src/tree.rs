//! The tree store (SDD-rh9u6z): an immutable sparse Merkle tree of a depth
//! fixed at build, path-copied, with compact single-leaf subtrees and
//! write-once node hashes.

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;
use core::marker::PhantomData;

use crate::bits::{bit, path_bits};
use crate::defaults::{DefaultHashes, Depth};
use crate::error::TreeError;
use crate::hash::NodeHash;
use crate::leaf::{leaf_hash, Leaf, TreeHasher};
use crate::node::{collect, joined, Child, Kind, Node};
use crate::proof::{AbsenceProof, InclusionProof, Path, ProofEnding};

/// An immutable sparse Merkle tree of depth `DEPTH` over leaves `L`, hashed
/// under `H`. `insert` and `remove` return a new tree sharing every
/// unchanged node with the receiver (LLR-8uem8c).
///
/// A depth of 1 to 256 compiles (LLR-ggu9bh):
///
/// ```
/// # use sparse_merkle_tree::{Leaf, NodeHash, SparseMerkleTree, TreeHasher};
/// # struct Zero;
/// # impl TreeHasher for Zero {
/// #     fn leaf(_: &NodeHash, _: &NodeHash) -> NodeHash { NodeHash::new([0; 32]) }
/// #     fn node(_: &NodeHash, _: &NodeHash) -> NodeHash { NodeHash::new([0; 32]) }
/// #     fn empty_leaf() -> NodeHash { NodeHash::new([0; 32]) }
/// # }
/// # struct Unit;
/// # impl Leaf<Zero> for Unit {
/// #     type Identity = u8;
/// #     fn identity(&self) -> &u8 { &0 }
/// #     fn position_bytes(_: &u8) -> [u8; 32] { [0; 32] }
/// #     fn identity_hash(_: &u8) -> NodeHash { NodeHash::new([0; 32]) }
/// #     fn content_hash(&self) -> NodeHash { NodeHash::new([0; 32]) }
/// # }
/// let _tree = SparseMerkleTree::<Unit, Zero, 1>::new();
/// ```
///
/// and a depth of 0 does not:
///
/// ```compile_fail
/// # use sparse_merkle_tree::{Leaf, NodeHash, SparseMerkleTree, TreeHasher};
/// # struct Zero;
/// # impl TreeHasher for Zero {
/// #     fn leaf(_: &NodeHash, _: &NodeHash) -> NodeHash { NodeHash::new([0; 32]) }
/// #     fn node(_: &NodeHash, _: &NodeHash) -> NodeHash { NodeHash::new([0; 32]) }
/// #     fn empty_leaf() -> NodeHash { NodeHash::new([0; 32]) }
/// # }
/// # struct Unit;
/// # impl Leaf<Zero> for Unit {
/// #     type Identity = u8;
/// #     fn identity(&self) -> &u8 { &0 }
/// #     fn position_bytes(_: &u8) -> [u8; 32] { [0; 32] }
/// #     fn identity_hash(_: &u8) -> NodeHash { NodeHash::new([0; 32]) }
/// #     fn content_hash(&self) -> NodeHash { NodeHash::new([0; 32]) }
/// # }
/// let _tree = SparseMerkleTree::<Unit, Zero, 0>::new();
/// ```
///
/// and neither does a depth above 256:
///
/// ```compile_fail
/// # use sparse_merkle_tree::{Leaf, NodeHash, SparseMerkleTree, TreeHasher};
/// # struct Zero;
/// # impl TreeHasher for Zero {
/// #     fn leaf(_: &NodeHash, _: &NodeHash) -> NodeHash { NodeHash::new([0; 32]) }
/// #     fn node(_: &NodeHash, _: &NodeHash) -> NodeHash { NodeHash::new([0; 32]) }
/// #     fn empty_leaf() -> NodeHash { NodeHash::new([0; 32]) }
/// # }
/// # struct Unit;
/// # impl Leaf<Zero> for Unit {
/// #     type Identity = u8;
/// #     fn identity(&self) -> &u8 { &0 }
/// #     fn position_bytes(_: &u8) -> [u8; 32] { [0; 32] }
/// #     fn identity_hash(_: &u8) -> NodeHash { NodeHash::new([0; 32]) }
/// #     fn content_hash(&self) -> NodeHash { NodeHash::new([0; 32]) }
/// # }
/// let _tree = SparseMerkleTree::<Unit, Zero, 257>::new();
/// ```
pub struct SparseMerkleTree<L, H, const DEPTH: usize> {
    root: Child<L>,
    defaults: Arc<DefaultHashes>,
    hasher: PhantomData<fn() -> H>,
}

impl<L: Leaf<H>, H: TreeHasher, const DEPTH: usize> SparseMerkleTree<L, H, DEPTH> {
    /// The empty tree; its empty-subtree hashes are computed here and shared
    /// by every tree derived from it. LLR-ggu9bh, LLR-yuyn8f.
    pub fn new() -> Self {
        let () = Depth::<DEPTH>::IN_RANGE;
        Self {
            root: None,
            defaults: Arc::new(DefaultHashes::compute::<H>(DEPTH)),
            hasher: PhantomData,
        }
    }

    /// The calculated tree holding `leaves`, inserted in the order given, or
    /// the first `Collision`. LLR-9qncuc.
    pub fn from_leaves<I: IntoIterator<Item = L>>(leaves: I) -> Result<Self, TreeError> {
        let mut tree = Self::new();
        for leaf in leaves {
            tree = tree.insert(leaf)?;
        }
        if tree.is_calculated() {
            Ok(tree)
        } else {
            tree.calculate()
        }
    }

    /// The root of `from_leaves(leaves)`. LLR-9qncuc.
    pub fn root_of<I: IntoIterator<Item = L>>(leaves: I) -> Result<NodeHash, TreeError> {
        Self::from_leaves(leaves)?.root()
    }

    /// The hash of an empty subtree at `level`, or `LevelOutOfRange` above
    /// `DEPTH`. LLR-yuyn8f.
    pub fn empty_subtree_hash(&self, level: usize) -> Result<NodeHash, TreeError> {
        self.defaults.at(level)
    }

    /// Whether no node hash is unset. LLR-639mz9.
    pub fn is_calculated(&self) -> bool {
        self.root.as_ref().is_none_or(|node| node.hash().is_some())
    }

    /// The root, or `HashesNotCalculated` while a hash is unset. LLR-3y5pq6.
    pub fn root(&self) -> Result<NodeHash, TreeError> {
        match &self.root {
            None => self.defaults.at(DEPTH),
            Some(node) => node.hash().ok_or(TreeError::HashesNotCalculated),
        }
    }

    /// Fills every unset hash cell bottom-up and returns the calculated tree,
    /// or `HashesAlreadyCalculated` when none is unset. LLR-3y5pq6.
    pub fn calculate(&self) -> Result<Self, TreeError> {
        if self.is_calculated() {
            return Err(TreeError::HashesAlreadyCalculated);
        }
        self.hash_of(self.root.as_ref(), 0)?;
        Ok(self.clone())
    }

    /// A new tree holding `leaf` at its position: replacing a leaf of the
    /// same identity there, refusing with `Collision` when a leaf of another
    /// identity holds it. LLR-ycf27m, LLR-8uem8c.
    pub fn insert(&self, leaf: L) -> Result<Self, TreeError> {
        let position_bytes = L::position_bytes(leaf.identity());
        let root = Self::insert_at(self.root.as_ref(), 0, &position_bytes, Arc::new(leaf))?;
        Ok(self.with_root(Some(root)))
    }

    /// A new tree without the leaf of `identity`, or `LeafNotHeld`.
    /// LLR-ycf27m, LLR-8uem8c.
    pub fn remove(&self, identity: &L::Identity) -> Result<Self, TreeError> {
        let position_bytes = L::position_bytes(identity);
        let root = Self::remove_at(self.root.as_ref(), 0, &position_bytes, identity)?;
        Ok(self.with_root(root))
    }

    /// The held leaf of `identity`. LLR-639mz9.
    pub fn get(&self, identity: &L::Identity) -> Option<&L> {
        let position_bytes = L::position_bytes(identity);
        let mut current = self.root.as_ref();
        let mut depth = 0;
        while let Some(node) = current {
            match &node.kind {
                Kind::Leaf(leaf) => return (leaf.identity() == identity).then_some(&**leaf),
                Kind::Internal { left, right } => {
                    current = if bit(&position_bytes, depth) { right.as_ref() } else { left.as_ref() };
                    depth += 1;
                }
            }
        }
        None
    }

    /// Whether a leaf of `identity` is held. LLR-639mz9.
    pub fn contains(&self, identity: &L::Identity) -> bool {
        self.get(identity).is_some()
    }

    /// Every held leaf, in strictly increasing position order. LLR-639mz9.
    pub fn leaves(&self) -> Vec<&L> {
        let mut found = Vec::new();
        collect(self.root.as_ref(), &mut found);
        found.into_iter().map(|leaf| &**leaf).collect()
    }

    /// An absence proof for `identity`, ending `Empty` when its position is
    /// in an empty subtree and `Occupied` when a leaf of another identity
    /// holds it; `LeafHeld` when a leaf of `identity` is held,
    /// `HashesNotCalculated` on an uncalculated tree. LLR-5tww69.
    pub fn prove_absent(&self, identity: &L::Identity) -> Result<AbsenceProof<DEPTH>, TreeError> {
        let position_bytes = L::position_bytes(identity);
        let (mut siblings, ending) = self.path(&position_bytes)?;
        let ending = match ending {
            None => ProofEnding::Empty,
            Some(held) if held.identity() == identity => return Err(TreeError::LeafHeld),
            Some(held) => {
                let held_position_bytes = L::position_bytes(held.identity());
                let fork = (siblings.len()..DEPTH).find(|&index| bit(&position_bytes, index) != bit(&held_position_bytes, index));
                match fork {
                    // The occupant's compact subtree is the sibling where the
                    // positions part; `identity`'s side below it is empty.
                    Some(fork) => {
                        self.pad(&mut siblings, fork)?;
                        siblings.push(self.compact_hash(&held, fork + 1)?);
                        ProofEnding::Empty
                    }
                    None => ProofEnding::Occupied {
                        identity_hash: L::identity_hash(held.identity()),
                        content_hash: held.content_hash(),
                    },
                }
            }
        };
        self.pad(&mut siblings, DEPTH)?;
        Ok(AbsenceProof { path: Path::from_siblings(&siblings, &self.defaults)?, ending })
    }

    /// The held leaf of `identity` and its inclusion proof; `LeafNotHeld`
    /// when none is held, `HashesNotCalculated` on an uncalculated tree.
    /// LLR-88rm99.
    pub fn prove_included(&self, identity: &L::Identity) -> Result<(Arc<L>, InclusionProof<DEPTH>), TreeError> {
        let position_bytes = L::position_bytes(identity);
        let (mut siblings, ending) = self.path(&position_bytes)?;
        let held = ending.filter(|held| held.identity() == identity).ok_or(TreeError::LeafNotHeld)?;
        self.pad(&mut siblings, DEPTH)?;
        Ok((held, InclusionProof { path: Path::from_siblings(&siblings, &self.defaults)? }))
    }

    /// The sibling hash at every depth of `position_bytes`'s walk, depth 0 first,
    /// and the leaf the walk ends in, if any.
    fn path(&self, position_bytes: &[u8; 32]) -> Result<(Vec<NodeHash>, Option<Arc<L>>), TreeError> {
        if !self.is_calculated() {
            return Err(TreeError::HashesNotCalculated);
        }
        let mut siblings = Vec::with_capacity(DEPTH);
        let mut current = self.root.as_ref();
        while let Some(node) = current {
            match &node.kind {
                Kind::Leaf(leaf) => return Ok((siblings, Some(Arc::clone(leaf)))),
                Kind::Internal { left, right } => {
                    let depth = siblings.len();
                    let (next, sibling) = if bit(position_bytes, depth) { (right, left) } else { (left, right) };
                    siblings.push(self.stored_hash(sibling.as_ref(), depth + 1)?);
                    current = next.as_ref();
                }
            }
        }
        Ok((siblings, None))
    }

    fn stored_hash(&self, child: Option<&Arc<Node<L>>>, depth: usize) -> Result<NodeHash, TreeError> {
        match child {
            None => self.defaults.at(DEPTH - depth),
            Some(node) => node.hash().ok_or(TreeError::HashesNotCalculated),
        }
    }

    /// Siblings for the depths from the end of the walk to `depth`: empty
    /// subtrees, the sibling at depth `d` being at level `DEPTH - 1 - d`.
    fn pad(&self, siblings: &mut Vec<NodeHash>, depth: usize) -> Result<(), TreeError> {
        while siblings.len() < depth {
            siblings.push(self.defaults.at(DEPTH - 1 - siblings.len())?);
        }
        Ok(())
    }

    fn with_root(&self, root: Child<L>) -> Self {
        Self { root, defaults: Arc::clone(&self.defaults), hasher: PhantomData }
    }

    fn insert_at(
        child: Option<&Arc<Node<L>>>,
        depth: usize,
        position_bytes: &[u8; 32],
        leaf: Arc<L>,
    ) -> Result<Arc<Node<L>>, TreeError> {
        let Some(node) = child else {
            return Ok(Node::leaf(leaf));
        };
        match &node.kind {
            Kind::Leaf(held) if held.identity() == leaf.identity() => Ok(Node::leaf(leaf)),
            Kind::Leaf(held) => Self::split(depth, Arc::clone(held), leaf),
            Kind::Internal { left, right } => Ok(if bit(position_bytes, depth) {
                Node::internal(left.clone(), Some(Self::insert_at(right.as_ref(), depth + 1, position_bytes, leaf)?))
            } else {
                Node::internal(Some(Self::insert_at(left.as_ref(), depth + 1, position_bytes, leaf)?), right.clone())
            }),
        }
    }

    /// Two leaves of the subtree at `depth`: an internal node over both where
    /// their positions first differ, and a lone-child internal node on the
    /// shared side at every level above it; `Collision` when the first
    /// `DEPTH` bits never differ. LLR-ycf27m, LLR-kep2ju.
    fn split(depth: usize, held: Arc<L>, leaf: Arc<L>) -> Result<Arc<Node<L>>, TreeError> {
        let held_position_bytes = L::position_bytes(held.identity());
        let position_bytes = L::position_bytes(leaf.identity());
        let fork = (depth..DEPTH)
            .find(|&index| bit(&held_position_bytes, index) != bit(&position_bytes, index))
            .ok_or(TreeError::Collision)?;
        let (left, right) = if bit(&position_bytes, fork) { (held, leaf) } else { (leaf, held) };
        let mut node = Node::internal(Some(Node::leaf(left)), Some(Node::leaf(right)));
        for index in (depth..fork).rev() {
            node = if bit(&position_bytes, index) {
                Node::internal(None, Some(node))
            } else {
                Node::internal(Some(node), None)
            };
        }
        Ok(node)
    }

    fn remove_at(
        child: Option<&Arc<Node<L>>>,
        depth: usize,
        position_bytes: &[u8; 32],
        identity: &L::Identity,
    ) -> Result<Child<L>, TreeError> {
        let node = child.ok_or(TreeError::LeafNotHeld)?;
        match &node.kind {
            Kind::Leaf(held) if held.identity() == identity => Ok(None),
            Kind::Leaf(_) => Err(TreeError::LeafNotHeld),
            Kind::Internal { left, right } => Ok(Some(if bit(position_bytes, depth) {
                joined(left.clone(), Self::remove_at(right.as_ref(), depth + 1, position_bytes, identity)?)
            } else {
                joined(Self::remove_at(left.as_ref(), depth + 1, position_bytes, identity)?, right.clone())
            })),
        }
    }

    /// The hash of `child` at `depth`, filling unset cells bottom-up.
    fn hash_of(&self, child: Option<&Arc<Node<L>>>, depth: usize) -> Result<NodeHash, TreeError> {
        let Some(node) = child else {
            return self.defaults.at(DEPTH - depth);
        };
        if let Some(hash) = node.hash() {
            return Ok(hash);
        }
        let hash = match &node.kind {
            Kind::Leaf(leaf) => self.compact_hash(leaf, depth)?,
            Kind::Internal { left, right } => H::node(
                &self.hash_of(left.as_ref(), depth + 1)?,
                &self.hash_of(right.as_ref(), depth + 1)?,
            ),
        };
        Ok(node.set_hash(hash))
    }

    /// A compact leaf at `depth`: its leaf hash folded up from level 0, the
    /// level's empty-subtree hash as the sibling at every level, left or
    /// right by its position bits. LLR-kep2ju.
    pub(crate) fn compact_hash(&self, leaf: &L, depth: usize) -> Result<NodeHash, TreeError> {
        let position_bytes = L::position_bytes(leaf.identity());
        let mut hash = leaf_hash::<L, H>(leaf);
        for (level, right) in path_bits(&position_bytes, DEPTH).skip(depth).rev().enumerate() {
            let empty = self.defaults.at(level)?;
            hash = if right { H::node(&empty, &hash) } else { H::node(&hash, &empty) };
        }
        Ok(hash)
    }
}

impl<L, H, const DEPTH: usize> SparseMerkleTree<L, H, DEPTH>
where
    L: Leaf<H> + PartialEq,
    L::Identity: Clone,
    H: TreeHasher,
{
    /// The identities `before` holds and `after` does not, and the leaves
    /// `after` holds and `before` does not hold as they are, each in strictly
    /// increasing position order. Either tree may be uncalculated. LLR-u2r4vj.
    pub fn changes_between(before: &Self, after: &Self) -> (Vec<L::Identity>, Vec<Arc<L>>) {
        let mut removed = Vec::new();
        let mut upserted = Vec::new();
        Self::changes_between_at(before.root.as_ref(), after.root.as_ref(), 0, &mut removed, &mut upserted);
        (removed, upserted)
    }

    /// Left before right; a pair that is one node, or whose hashes are both
    /// set and equal, is skipped.
    fn changes_between_at(
        before: Option<&Arc<Node<L>>>,
        after: Option<&Arc<Node<L>>>,
        depth: usize,
        removed: &mut Vec<L::Identity>,
        upserted: &mut Vec<Arc<L>>,
    ) {
        match (before, after) {
            (None, None) => {}
            (Some(old), None) => {
                let mut gone = Vec::new();
                collect(Some(old), &mut gone);
                removed.extend(gone.into_iter().map(|leaf| leaf.identity().clone()));
            }
            (None, Some(new)) => {
                let mut added = Vec::new();
                collect(Some(new), &mut added);
                upserted.extend(added.into_iter().map(Arc::clone));
            }
            (Some(old), Some(new))
                if Arc::ptr_eq(old, new) || (old.hash().is_some() && old.hash() == new.hash()) => {}
            (Some(old), Some(new)) => match (&old.kind, &new.kind) {
                (Kind::Leaf(was), Kind::Leaf(now)) => {
                    if was.identity() != now.identity() {
                        removed.push(was.identity().clone());
                        upserted.push(Arc::clone(now));
                    } else if was != now {
                        upserted.push(Arc::clone(now));
                    }
                }
                (
                    Kind::Internal { left: old_left, right: old_right },
                    Kind::Internal { left: new_left, right: new_right },
                ) => {
                    Self::changes_between_at(old_left.as_ref(), new_left.as_ref(), depth + 1, removed, upserted);
                    Self::changes_between_at(old_right.as_ref(), new_right.as_ref(), depth + 1, removed, upserted);
                }
                (Kind::Internal { left, right }, Kind::Leaf(now)) => {
                    let (new_left, new_right) = Self::lowered(now, depth);
                    Self::changes_between_at(left.as_ref(), new_left.as_ref(), depth + 1, removed, upserted);
                    Self::changes_between_at(right.as_ref(), new_right.as_ref(), depth + 1, removed, upserted);
                }
                (Kind::Leaf(was), Kind::Internal { left, right }) => {
                    let (old_left, old_right) = Self::lowered(was, depth);
                    Self::changes_between_at(old_left.as_ref(), left.as_ref(), depth + 1, removed, upserted);
                    Self::changes_between_at(old_right.as_ref(), right.as_ref(), depth + 1, removed, upserted);
                }
            },
        }
    }

    /// A compact leaf at `depth` seen one level down: on the side its
    /// position names, with nothing on the other.
    fn lowered(leaf: &Arc<L>, depth: usize) -> (Child<L>, Child<L>) {
        let node = Some(Node::leaf(Arc::clone(leaf)));
        if bit(&L::position_bytes(leaf.identity()), depth) {
            (None, node)
        } else {
            (node, None)
        }
    }
}

impl<L: Leaf<H>, H: TreeHasher, const DEPTH: usize> Default for SparseMerkleTree<L, H, DEPTH> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L, H, const DEPTH: usize> Clone for SparseMerkleTree<L, H, DEPTH> {
    fn clone(&self) -> Self {
        Self { root: self.root.clone(), defaults: Arc::clone(&self.defaults), hasher: PhantomData }
    }
}

impl<L, H, const DEPTH: usize> fmt::Debug for SparseMerkleTree<L, H, DEPTH> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("SparseMerkleTree").field("depth", &DEPTH).finish_non_exhaustive()
    }
}

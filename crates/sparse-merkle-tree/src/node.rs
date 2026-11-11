//! The store's nodes: a compact leaf at any level, or an internal node.
//! An empty subtree is no node. LLR-kep2ju, LLR-3y5pq6.

use alloc::sync::Arc;

use spin::Once;

use crate::hash::NodeHash;

/// A subtree: `None` is empty.
pub(crate) type Child<L> = Option<Arc<Node<L>>>;

pub(crate) enum Kind<L> {
    /// The only leaf of the subtree this node tops.
    Leaf(Arc<L>),
    Internal { left: Child<L>, right: Child<L> },
}

/// A node and its write-once hash cell. LLR-3y5pq6.
pub(crate) struct Node<L> {
    hash: Once<NodeHash>,
    pub(crate) kind: Kind<L>,
}

impl<L> Node<L> {
    pub(crate) fn leaf(leaf: Arc<L>) -> Arc<Self> {
        Arc::new(Self { hash: Once::new(), kind: Kind::Leaf(leaf) })
    }

    pub(crate) fn internal(left: Child<L>, right: Child<L>) -> Arc<Self> {
        Arc::new(Self { hash: Once::new(), kind: Kind::Internal { left, right } })
    }

    pub(crate) fn hash(&self) -> Option<NodeHash> {
        self.hash.get().copied()
    }

    /// Fills the cell if unset and returns what it holds.
    pub(crate) fn set_hash(&self, hash: NodeHash) -> NodeHash {
        *self.hash.call_once(|| hash)
    }
}

/// The node over `left` and `right`, where a lone leaf beside an empty
/// subtree rises to take its parent's place, so the store keeps one form per
/// set of leaves. LLR-kep2ju.
pub(crate) fn joined<L>(left: Child<L>, right: Child<L>) -> Arc<Node<L>> {
    let lone = match (&left, &right) {
        (None, Some(only)) | (Some(only), None) => match &only.kind {
            Kind::Leaf(leaf) => Some(Arc::clone(leaf)),
            Kind::Internal { .. } => None,
        },
        _ => None,
    };
    match lone {
        Some(leaf) => Node::leaf(leaf),
        None => Node::internal(left, right),
    }
}

/// Every leaf under `child`, left before right.
pub(crate) fn collect<'a, L>(child: Option<&'a Arc<Node<L>>>, found: &mut alloc::vec::Vec<&'a Arc<L>>) {
    if let Some(node) = child {
        match &node.kind {
            Kind::Leaf(leaf) => found.push(leaf),
            Kind::Internal { left, right } => {
                collect(left.as_ref(), found);
                collect(right.as_ref(), found);
            }
        }
    }
}

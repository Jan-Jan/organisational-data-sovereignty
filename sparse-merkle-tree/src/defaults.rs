//! The depth bound and the empty-subtree hashes. LLR-ggu9bh, LLR-yuyn8f.

use alloc::vec::Vec;

use crate::error::TreeError;
use crate::hash::NodeHash;
use crate::leaf::TreeHasher;

/// The depths a tree may have: 1 to 256. A function naming
/// `Depth::<D>::IN_RANGE` fails to compile for any other `D`, wherever it is
/// used. LLR-ggu9bh.
pub(crate) struct Depth<const D: usize>;

impl<const D: usize> Depth<D> {
    pub(crate) const IN_RANGE: () = assert!(D != 0 && D <= 256, "a sparse Merkle tree's depth is 1 to 256");

    /// `D` as the serde form writes it: a u16, which `D` fits because the
    /// bound of `IN_RANGE` is asserted here too, at compile time.
    #[cfg(feature = "serde")]
    pub(crate) const AS_U16: u16 = {
        assert!(D != 0 && D <= 256, "a sparse Merkle tree's depth is 1 to 256");
        D as u16
    };
}

/// The hash of an empty subtree at every level 0..=depth: level 0 is
/// `H::empty_leaf()`, level `i` is `H::node` of level `i - 1` with itself.
/// LLR-yuyn8f.
pub(crate) struct DefaultHashes {
    levels: Vec<NodeHash>,
}

impl DefaultHashes {
    pub(crate) fn compute<H: TreeHasher>(depth: usize) -> Self {
        let mut levels = Vec::with_capacity(depth + 1);
        let mut hash = H::empty_leaf();
        levels.push(hash);
        for _ in 0..depth {
            hash = H::node(&hash, &hash);
            levels.push(hash);
        }
        Self { levels }
    }

    /// `LevelOutOfRange` above the depth, without indexing out of bounds.
    pub(crate) fn at(&self, level: usize) -> Result<NodeHash, TreeError> {
        self.levels.get(level).copied().ok_or(TreeError::LevelOutOfRange)
    }
}

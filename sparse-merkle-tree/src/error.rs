//! Every refusal of the crate, one variant per check. LLR-smf4zq.

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeError {
    /// The leaf's position holds a leaf of another identity. LLR-ycf27m.
    Collision,
    /// No leaf of that identity is held. LLR-ycf27m, LLR-88rm99.
    LeafNotHeld,
    /// A leaf of that identity is held, so it has no absence proof. LLR-5tww69.
    LeafHeld,
    /// A node hash is unset. LLR-3y5pq6.
    HashesNotCalculated,
    /// No node hash is unset. LLR-3y5pq6.
    HashesAlreadyCalculated,
    /// A level above the tree's depth. LLR-yuyn8f.
    LevelOutOfRange,
    /// A proof's depth, map or siblings do not fit its type. LLR-77zy97.
    ProofMalformed,
    /// A proof does not resolve to the root. LLR-53ezgd, LLR-c7bdee.
    ProofRootMismatch,
    /// An absence proof ending in a leaf of the identity asked about. LLR-53ezgd.
    ProofLeafHeld,
}

impl fmt::Display for TreeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Collision => "position holds a leaf of another identity",
            Self::LeafNotHeld => "no leaf of that identity is held",
            Self::LeafHeld => "a leaf of that identity is held",
            Self::HashesNotCalculated => "hashes not calculated",
            Self::HashesAlreadyCalculated => "hashes already calculated",
            Self::LevelOutOfRange => "level above the tree's depth",
            Self::ProofMalformed => "malformed proof",
            Self::ProofRootMismatch => "proof does not resolve to the root",
            Self::ProofLeafHeld => "proof ends in a leaf of that identity",
        })
    }
}

impl core::error::Error for TreeError {}

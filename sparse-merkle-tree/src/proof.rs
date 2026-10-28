//! Absence and inclusion proofs over a tree of fixed depth (SDD-ur6uvu).

use alloc::vec;
use alloc::vec::Vec;

use crate::bits::path_bits;
use crate::defaults::{DefaultHashes, Depth};
use crate::error::TreeError;
use crate::hash::NodeHash;
use crate::leaf::{leaf_hash, Leaf, TreeHasher};

/// What an absence proof's path ends in: an empty position, or the identity
/// hash and content hash of the leaf of another identity holding it, and
/// nothing else of that leaf. LLR-m5c3vw.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProofEnding {
    Empty,
    Occupied { identity_hash: NodeHash, content_hash: NodeHash },
}

/// Whether `map` marks `level`: bit `level % 8` of byte `level / 8`.
fn marked(map: &[u8], level: usize) -> bool {
    map.get(level / 8).is_some_and(|byte| (byte >> (level % 8)) & 1 == 1)
}

/// A path's siblings, from the leaf's level up: a map of `DEPTH` bits marking
/// the siblings that are not their level's empty-subtree hash, and those
/// siblings in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Path<const DEPTH: usize> {
    pub(crate) map: Vec<u8>,
    pub(crate) siblings: Vec<NodeHash>,
}

impl<const DEPTH: usize> Path<DEPTH> {
    /// From the sibling at every depth of a walk, depth 0 first.
    pub(crate) fn from_siblings(by_depth: &[NodeHash], defaults: &DefaultHashes) -> Result<Self, TreeError> {
        let mut map = vec![0u8; DEPTH.div_ceil(8)];
        let mut siblings = Vec::new();
        for (level, sibling) in by_depth.iter().rev().enumerate() {
            if *sibling != defaults.at(level)? {
                // Never refuses: the map is DEPTH.div_ceil(8) bytes and both
                // callers pass DEPTH siblings, so `level` is below DEPTH.
                let byte = map.get_mut(level / 8).ok_or(TreeError::LevelOutOfRange)?;
                *byte |= 1 << (level % 8);
                siblings.push(*sibling);
            }
        }
        Ok(Self { map, siblings })
    }

    /// `ProofMalformed` unless the map is `DEPTH` bits, marks no level at or
    /// above `DEPTH`, and marks as many levels as there are siblings.
    /// LLR-77zy97.
    pub(crate) fn parse(map: Vec<u8>, siblings: Vec<NodeHash>) -> Result<Self, TreeError> {
        let () = Depth::<DEPTH>::IN_RANGE;
        if map.len() != DEPTH.div_ceil(8) {
            return Err(TreeError::ProofMalformed);
        }
        let beyond = (DEPTH..map.len() * 8).any(|level| marked(&map, level));
        let marks = (0..DEPTH).filter(|&level| marked(&map, level)).count();
        if beyond || marks != siblings.len() {
            return Err(TreeError::ProofMalformed);
        }
        Ok(Self { map, siblings })
    }

    /// `start` folded from the leaf's level up along `position_bytes`, one sibling
    /// per level: the next marked sibling, or the level's empty-subtree hash.
    fn fold<H: TreeHasher>(&self, start: NodeHash, position_bytes: &[u8; 32]) -> Result<NodeHash, TreeError> {
        let defaults = DefaultHashes::compute::<H>(DEPTH);
        let mut explicit = self.siblings.iter();
        let mut hash = start;
        for (level, right) in path_bits(position_bytes, DEPTH).rev().enumerate() {
            let sibling = if marked(&self.map, level) {
                *explicit.next().ok_or(TreeError::ProofMalformed)?
            } else {
                defaults.at(level)?
            };
            hash = if right { H::node(&sibling, &hash) } else { H::node(&hash, &sibling) };
        }
        Ok(hash)
    }
}

/// A proof that a tree of depth `DEPTH` holds no leaf of an identity.
/// Its fields are private: it comes from `prove_absent`, `from_parts` or
/// decoding. LLR-m5c3vw, LLR-77zy97.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbsenceProof<const DEPTH: usize> {
    pub(crate) path: Path<DEPTH>,
    pub(crate) ending: ProofEnding,
}

impl<const DEPTH: usize> AbsenceProof<DEPTH> {
    /// LLR-77zy97.
    pub fn from_parts(map: Vec<u8>, siblings: Vec<NodeHash>, ending: ProofEnding) -> Result<Self, TreeError> {
        Ok(Self { path: Path::parse(map, siblings)?, ending })
    }

    pub fn map(&self) -> &[u8] {
        &self.path.map
    }

    pub fn siblings(&self) -> &[NodeHash] {
        &self.path.siblings
    }

    pub fn ending(&self) -> &ProofEnding {
        &self.ending
    }

    /// Folds from `H::empty_leaf()` (an `Empty` ending) or from
    /// `H::leaf(identity_hash, content_hash)` (an `Occupied` one) along
    /// `identity`'s position; `ProofRootMismatch` when that is not `root`,
    /// before any other check; then `ProofLeafHeld` when the ending is the
    /// leaf of `identity` itself. LLR-53ezgd.
    pub fn verify<L: Leaf<H>, H: TreeHasher>(&self, root: &NodeHash, identity: &L::Identity) -> Result<(), TreeError> {
        let start = match &self.ending {
            ProofEnding::Empty => H::empty_leaf(),
            ProofEnding::Occupied { identity_hash, content_hash } => H::leaf(identity_hash, content_hash),
        };
        if self.path.fold::<H>(start, &L::position_bytes(identity))? != *root {
            return Err(TreeError::ProofRootMismatch);
        }
        match &self.ending {
            ProofEnding::Occupied { identity_hash, .. } if *identity_hash == L::identity_hash(identity) => {
                Err(TreeError::ProofLeafHeld)
            }
            _ => Ok(()),
        }
    }
}

/// A proof that a tree of depth `DEPTH` holds a given leaf; the leaf is given
/// to the verifier, not carried. LLR-tz3nfz.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InclusionProof<const DEPTH: usize> {
    pub(crate) path: Path<DEPTH>,
}

impl<const DEPTH: usize> InclusionProof<DEPTH> {
    /// LLR-77zy97.
    pub fn from_parts(map: Vec<u8>, siblings: Vec<NodeHash>) -> Result<Self, TreeError> {
        Ok(Self { path: Path::parse(map, siblings)? })
    }

    pub fn map(&self) -> &[u8] {
        &self.path.map
    }

    pub fn siblings(&self) -> &[NodeHash] {
        &self.path.siblings
    }

    /// Folds from `leaf`'s hash along its identity's position;
    /// `ProofRootMismatch` when that is not `root`. LLR-c7bdee.
    pub fn verify<L: Leaf<H>, H: TreeHasher>(&self, root: &NodeHash, leaf: &L) -> Result<(), TreeError> {
        if self.path.fold::<H>(leaf_hash::<L, H>(leaf), &L::position_bytes(leaf.identity()))? != *root {
            return Err(TreeError::ProofRootMismatch);
        }
        Ok(())
    }
}

/// The serde form (LLR-77zy97): the tuple (depth, map, siblings, ending) for
/// an absence proof and (depth, map, siblings) for an inclusion proof. A
/// decode refuses a depth other than `DEPTH` first, then a map or sibling
/// list at its first element past the bound, before storing it, and ends in
/// `from_parts`. Every refusal is `TreeError::ProofMalformed`, which reaches
/// a serde format as its custom error.
#[cfg(feature = "serde")]
mod wire {
    use alloc::vec::Vec;
    use core::fmt;
    use core::marker::PhantomData;

    use serde::de::{DeserializeSeed, Error as _, SeqAccess, Visitor};
    use serde::ser::SerializeTuple;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{AbsenceProof, InclusionProof, ProofEnding};
    use crate::defaults::Depth;
    use crate::error::TreeError;
    use crate::hash::NodeHash;

    /// A sequence of at most `max` elements.
    struct Bounded<T> {
        max: usize,
        element: PhantomData<T>,
    }

    impl<T> Bounded<T> {
        fn new(max: usize) -> Self {
            Self { max, element: PhantomData }
        }
    }

    impl<'de, T: Deserialize<'de>> DeserializeSeed<'de> for Bounded<T> {
        type Value = Vec<T>;
        fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Vec<T>, D::Error> {
            deserializer.deserialize_seq(self)
        }
    }

    impl<'de, T: Deserialize<'de>> Visitor<'de> for Bounded<T> {
        type Value = Vec<T>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "a sequence of at most {} elements", self.max)
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Vec<T>, A::Error> {
            let mut elements = Vec::new();
            while let Some(element) = sequence.next_element::<T>()? {
                if elements.len() == self.max {
                    return Err(A::Error::custom(TreeError::ProofMalformed));
                }
                elements.push(element);
            }
            Ok(elements)
        }
    }

    fn part<'de, A: SeqAccess<'de>, S: DeserializeSeed<'de>>(sequence: &mut A, seed: S) -> Result<S::Value, A::Error> {
        sequence
            .next_element_seed(seed)?
            .ok_or_else(|| A::Error::custom(TreeError::ProofMalformed))
    }

    /// The depth, checked, then the map and the siblings, bounded.
    fn path<'de, A: SeqAccess<'de>, const DEPTH: usize>(sequence: &mut A) -> Result<(Vec<u8>, Vec<NodeHash>), A::Error> {
        let depth: u16 = part(sequence, PhantomData::<u16>)?;
        if usize::from(depth) != DEPTH {
            return Err(A::Error::custom(TreeError::ProofMalformed));
        }
        let map = part(sequence, Bounded::<u8>::new(DEPTH.div_ceil(8)))?;
        let siblings = part(sequence, Bounded::<NodeHash>::new(DEPTH))?;
        Ok((map, siblings))
    }

    impl<const DEPTH: usize> Serialize for AbsenceProof<DEPTH> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut tuple = serializer.serialize_tuple(4)?;
            tuple.serialize_element(&Depth::<DEPTH>::AS_U16)?;
            tuple.serialize_element(&self.path.map)?;
            tuple.serialize_element(&self.path.siblings)?;
            tuple.serialize_element(&self.ending)?;
            tuple.end()
        }
    }

    impl<const DEPTH: usize> Serialize for InclusionProof<DEPTH> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut tuple = serializer.serialize_tuple(3)?;
            tuple.serialize_element(&Depth::<DEPTH>::AS_U16)?;
            tuple.serialize_element(&self.path.map)?;
            tuple.serialize_element(&self.path.siblings)?;
            tuple.end()
        }
    }

    struct AbsenceVisitor<const DEPTH: usize>;

    impl<'de, const DEPTH: usize> Visitor<'de> for AbsenceVisitor<DEPTH> {
        type Value = AbsenceProof<DEPTH>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("an absence proof: depth, map, siblings, ending")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
            let (map, siblings) = path::<A, DEPTH>(&mut sequence)?;
            let ending = part(&mut sequence, PhantomData::<ProofEnding>)?;
            AbsenceProof::from_parts(map, siblings, ending).map_err(A::Error::custom)
        }
    }

    struct InclusionVisitor<const DEPTH: usize>;

    impl<'de, const DEPTH: usize> Visitor<'de> for InclusionVisitor<DEPTH> {
        type Value = InclusionProof<DEPTH>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("an inclusion proof: depth, map, siblings")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
            let (map, siblings) = path::<A, DEPTH>(&mut sequence)?;
            InclusionProof::from_parts(map, siblings).map_err(A::Error::custom)
        }
    }

    impl<'de, const DEPTH: usize> Deserialize<'de> for AbsenceProof<DEPTH> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            deserializer.deserialize_tuple(4, AbsenceVisitor::<DEPTH>)
        }
    }

    impl<'de, const DEPTH: usize> Deserialize<'de> for InclusionProof<DEPTH> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            deserializer.deserialize_tuple(3, InclusionVisitor::<DEPTH>)
        }
    }
}

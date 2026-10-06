//! Absence proofs (SDD-57vaj4): a proof that a MemberId holds no Member, or
//! that the Member it holds lacks a Device key, checked against a membership
//! root. The only code that interprets a leaf for this purpose.

use alloc::vec::Vec;

use person::compute_device_root;

use crate::error::OrgMembersError;
use crate::hasher::TrieHasher;
use crate::smt::{DefaultHashes, SMT_DEPTH};
use crate::types::{DevicePublicKey, MemberId, MemberLeaf, NodeHash, RootHash};

/// What an absence proof's path ends in (LLR-25tpdp).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofEnding {
    /// The MemberId holds no Member: the proof carries no Member data.
    Empty,
    /// The Member under the proof's MemberId, and no one else.
    Leaf(MemberLeaf),
}

impl ProofEnding {
    /// Whether the ending is a Member that holds `device`.
    pub(crate) fn holds(&self, device: &DevicePublicKey) -> bool {
        match self {
            Self::Empty => false,
            Self::Leaf(member) => member.has_p2p_device(device),
        }
    }
}

/// A proof of absence (LLR-25tpdp). Fields are private; the only ways to get
/// one are `OrgTrie::prove_absent` and parsing (LLR-4rju5r).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(try_from = "wire::RawAbsenceProof", into = "wire::RawAbsenceProof")
)]
pub struct AbsenceProof {
    default_map: [u8; 32],
    /// The non-default siblings, from the leaf's level up.
    siblings: Vec<NodeHash>,
    ending: ProofEnding,
}

/// Whether a default-sibling map marks `level` (bit `level % 8` of byte
/// `level / 8`) as that level's default hash.
fn is_default_level(map: &[u8; 32], level: u16) -> bool {
    map.get(usize::from(level / 8))
        .is_some_and(|byte| (byte >> (level % 8)) & 1 == 1)
}

fn mark_default_level(map: &mut [u8; 32], level: u16) -> Result<(), OrgMembersError> {
    let byte = map
        .get_mut(usize::from(level / 8))
        .ok_or(OrgMembersError::InvariantViolated)?;
    *byte |= 1 << (level % 8);
    Ok(())
}

fn default_level_count(map: &[u8; 32]) -> usize {
    map.iter().map(|byte| byte.count_ones() as usize).sum()
}

impl AbsenceProof {
    /// Builds a proof from a path walk (LLR-utp6x4), sending only the
    /// siblings that are not their level's default (LLR-25tpdp).
    pub(crate) fn from_path(
        siblings: Vec<NodeHash>,
        ending: ProofEnding,
        defaults: &DefaultHashes,
    ) -> Result<Self, OrgMembersError> {
        let mut default_map = [0u8; 32];
        let mut explicit_siblings = Vec::new();
        for (level, hash) in (0..SMT_DEPTH).zip(siblings) {
            if hash == *defaults.at_level(level)? {
                mark_default_level(&mut default_map, level)?;
            } else {
                explicit_siblings.push(hash);
            }
        }
        Ok(Self {
            default_map,
            siblings: explicit_siblings,
            ending,
        })
    }

    /// Parses a proof from its parts: refuses with `AbsenceProofMalformed`
    /// unless the number of siblings is the number of levels the map does not
    /// mark default, which caps it at 256 (LLR-4rju5r).
    pub fn from_parts(
        default_map: [u8; 32],
        siblings: Vec<NodeHash>,
        ending: ProofEnding,
    ) -> Result<Self, OrgMembersError> {
        if siblings.len() + default_level_count(&default_map) != usize::from(SMT_DEPTH) {
            return Err(OrgMembersError::AbsenceProofMalformed);
        }
        Ok(Self {
            default_map,
            siblings,
            ending,
        })
    }

    /// The default-sibling map: bit `level % 8` of byte `level / 8` set means
    /// the sibling at `level` (0 = the leaf's level) is that level's default
    /// hash and is not among `siblings()`.
    pub fn default_map(&self) -> &[u8; 32] {
        &self.default_map
    }

    /// The siblings that are not their level's default, from the leaf's
    /// level up.
    pub fn siblings(&self) -> &[NodeHash] {
        &self.siblings
    }

    pub fn ending(&self) -> &ProofEnding {
        &self.ending
    }

    /// Accepts the proof only when it resolves to `root` along `id`'s path
    /// (LLR-dgzy7e) and its ending shows `device` absent (LLR-p2p8qy).
    pub fn verify<H: TrieHasher>(
        &self,
        root: &RootHash,
        id: &MemberId,
        device: &DevicePublicKey,
    ) -> Result<(), OrgMembersError> {
        let defaults = DefaultHashes::compute::<H>();
        let mut computed = match &self.ending {
            ProofEnding::Empty => *defaults.empty_leaf(),
            ProofEnding::Leaf(member) => {
                let device_root = compute_device_root::<H>(member.p2p_device_slots());
                H::hash_member_leaf(&member.canonical_bytes(&device_root))
            }
        };
        let mut explicit_siblings = self.siblings.iter();
        // `path_bits` runs from the root down; the fold runs from the leaf up.
        for (level, went_right) in (0..SMT_DEPTH).zip(id.path_bits().rev()) {
            let sibling = if is_default_level(&self.default_map, level) {
                *defaults.at_level(level)?
            } else {
                *explicit_siblings
                    .next()
                    .ok_or(OrgMembersError::AbsenceProofMalformed)?
            };
            computed = if went_right {
                H::hash_member_node(&sibling, &computed)
            } else {
                H::hash_member_node(&computed, &sibling)
            };
        }
        // No surplus check: from_path and from_parts fix the count (LLR-4rju5r).
        if computed.as_bytes() != root.as_bytes() {
            return Err(OrgMembersError::AbsenceProofRootMismatch);
        }
        if self.ending.holds(device) {
            return Err(OrgMembersError::DeviceStillHeld);
        }
        Ok(())
    }
}

#[cfg(feature = "serde")]
mod wire {
    use alloc::vec::Vec;
    use core::fmt;

    use serde::de::{Error as _, SeqAccess, Visitor};
    use serde::{Deserialize, Deserializer, Serialize};

    use super::{AbsenceProof, ProofEnding};
    use crate::error::OrgMembersError;
    use crate::smt::SMT_DEPTH;
    use crate::types::{MemberLeaf, NodeHash};

    #[derive(Serialize, Deserialize)]
    pub(super) struct RawAbsenceProof {
        default_map: [u8; 32],
        #[serde(deserialize_with = "bounded_siblings")]
        siblings: Vec<[u8; 32]>,
        ending: Option<MemberLeaf>,
    }

    /// Refuses a 257th sibling while decoding, so memory never grows past
    /// the bound (LLR-4rju5r).
    fn bounded_siblings<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<[u8; 32]>, D::Error> {
        struct BoundedSiblings;
        impl<'de> Visitor<'de> for BoundedSiblings {
            type Value = Vec<[u8; 32]>;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("at most 256 sibling hashes")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut siblings = Vec::new();
                while let Some(hash) = seq.next_element::<[u8; 32]>()? {
                    if siblings.len() == usize::from(SMT_DEPTH) {
                        return Err(A::Error::custom(OrgMembersError::AbsenceProofMalformed));
                    }
                    siblings.push(hash);
                }
                Ok(siblings)
            }
        }
        deserializer.deserialize_seq(BoundedSiblings)
    }

    impl From<AbsenceProof> for RawAbsenceProof {
        fn from(proof: AbsenceProof) -> Self {
            Self {
                default_map: proof.default_map,
                siblings: proof.siblings.iter().map(|hash| *hash.as_bytes()).collect(),
                ending: match proof.ending {
                    ProofEnding::Empty => None,
                    ProofEnding::Leaf(member) => Some(member),
                },
            }
        }
    }

    impl TryFrom<RawAbsenceProof> for AbsenceProof {
        type Error = OrgMembersError;
        fn try_from(raw: RawAbsenceProof) -> Result<Self, Self::Error> {
            AbsenceProof::from_parts(
                raw.default_map,
                raw.siblings.into_iter().map(NodeHash::new).collect(),
                match raw.ending {
                    None => ProofEnding::Empty,
                    Some(member) => ProofEnding::Leaf(member),
                },
            )
        }
    }
}

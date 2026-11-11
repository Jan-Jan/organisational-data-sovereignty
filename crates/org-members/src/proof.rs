//! Absence proofs (SDD-57vaj4): a Vacant proof that a MemberId holds no
//! Member, or a Held proof that the Member it holds lacks a DevicePublicKey,
//! composed of the `sparse-merkle-tree` unit's proofs.

use alloc::boxed::Box;
use core::fmt;

use person::{DeviceLeaf, DEVICE_ROOT_DEPTH};
use sparse_merkle_tree::{InclusionProof, Leaf, NodeHash, TreeError};

use crate::error::OrgMembersError;
use crate::hasher::MemberHasher;
use crate::member_tree::{
    member_content_hash, member_identity_hash, member_position_bytes, TreeMember, MEMBER_DEPTH,
};
use crate::types::{DevicePublicKey, Handle, MemberId, Name, PersonPublicKey, RootHash, Surname};

/// The Member's leaf without its DevicePublicKeys: its MemberId, handle,
/// Member-as-a-group key, name, surname and device root. LLR-v6smww.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProvenMember {
    id: MemberId,
    handle: Handle,
    person_key: PersonPublicKey,
    name: Name,
    surname: Surname,
    device_root: NodeHash,
}

impl ProvenMember {
    pub(crate) fn of(held: &TreeMember) -> Self {
        let member = held.member();
        Self {
            id: *member.id(),
            handle: member.handle().clone(),
            person_key: *member.p2p_key(),
            name: member.name().clone(),
            surname: member.surname().clone(),
            device_root: *held.device_root(),
        }
    }

    pub fn id(&self) -> &MemberId {
        &self.id
    }

    pub fn handle(&self) -> &Handle {
        &self.handle
    }

    pub fn person_key(&self) -> &PersonPublicKey {
        &self.person_key
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn surname(&self) -> &Surname {
        &self.surname
    }

    pub fn device_root(&self) -> &NodeHash {
        &self.device_root
    }
}

impl fmt::Debug for ProvenMember {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("ProvenMember")
            .field("id", &self.id)
            .field("handle", &"[REDACTED]")
            .field("person_key", &self.person_key)
            .field("name", &"[REDACTED]")
            .field("surname", &"[REDACTED]")
            .field("device_root", &self.device_root)
            .finish()
    }
}

/// The leaf a Held proof's fields rebuild: hashed as `TreeMember` hashes.
impl<H: MemberHasher> Leaf<H> for ProvenMember {
    type Identity = MemberId;

    fn identity(&self) -> &MemberId {
        &self.id
    }

    fn position_bytes(identity: &MemberId) -> [u8; 32] {
        member_position_bytes(identity)
    }

    fn identity_hash(identity: &MemberId) -> NodeHash {
        member_identity_hash::<H>(identity)
    }

    fn content_hash(&self) -> NodeHash {
        member_content_hash::<H>(&self.handle, &self.person_key, &self.name, &self.surname, &self.device_root)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
enum Kind {
    Vacant(sparse_merkle_tree::AbsenceProof<MEMBER_DEPTH>),
    Held {
        member: Box<ProvenMember>,
        inclusion: InclusionProof<MEMBER_DEPTH>,
        device_root_absence: sparse_merkle_tree::AbsenceProof<DEVICE_ROOT_DEPTH>,
    },
}

/// A proof of absence (LLR-v6smww). It holds no DevicePublicKey and no other
/// Member's data; its fields are private, and it comes from
/// `OrgTrie::prove_absent` or parsing (LLR-9g2kjg).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbsenceProof(Kind);

impl AbsenceProof {
    pub(crate) fn vacant(vacancy: sparse_merkle_tree::AbsenceProof<MEMBER_DEPTH>) -> Self {
        Self(Kind::Vacant(vacancy))
    }

    pub(crate) fn held(
        member: ProvenMember,
        inclusion: InclusionProof<MEMBER_DEPTH>,
        device_root_absence: sparse_merkle_tree::AbsenceProof<DEVICE_ROOT_DEPTH>,
    ) -> Self {
        Self(Kind::Held { member: Box::new(member), inclusion, device_root_absence })
    }

    pub fn is_vacant(&self) -> bool {
        matches!(self.0, Kind::Vacant(_))
    }

    /// A Vacant proof's vacancy: the tree's proof, at depth 256, that its
    /// MemberId is vacant.
    pub fn vacancy(&self) -> Option<&sparse_merkle_tree::AbsenceProof<MEMBER_DEPTH>> {
        match &self.0 {
            Kind::Vacant(vacancy) => Some(vacancy),
            Kind::Held { .. } => None,
        }
    }

    /// A Held proof's Member without its DevicePublicKeys.
    pub fn member(&self) -> Option<&ProvenMember> {
        match &self.0 {
            Kind::Held { member, .. } => Some(member.as_ref()),
            Kind::Vacant(_) => None,
        }
    }

    /// A Held proof's inclusion proof at depth 256.
    pub fn inclusion(&self) -> Option<&InclusionProof<MEMBER_DEPTH>> {
        match &self.0 {
            Kind::Held { inclusion, .. } => Some(inclusion),
            Kind::Vacant(_) => None,
        }
    }

    /// A Held proof's device-root absence: the tree's proof, at depth 4 under
    /// its device root, that the DevicePublicKey is absent.
    pub fn device_root_absence(&self) -> Option<&sparse_merkle_tree::AbsenceProof<DEVICE_ROOT_DEPTH>> {
        match &self.0 {
            Kind::Held { device_root_absence, .. } => Some(device_root_absence),
            Kind::Vacant(_) => None,
        }
    }

    /// A Vacant proof as the absence of `id` under `root`; a Held proof as
    /// the inclusion under `root`, at `id`'s position, of the leaf its fields
    /// rebuild, then the absence of `device` under its device root. Either
    /// step not resolving is `AbsenceProofRootMismatch`, before any other
    /// check (LLR-54gp2f); then `DeviceStillHeld` when the device step shows
    /// `device` held, `AbsenceProofMalformed` when a Vacant step shows `id`
    /// held (LLR-25nmwc).
    pub fn verify<H: MemberHasher>(&self, root: &RootHash, id: &MemberId, device: &DevicePublicKey) -> Result<(), OrgMembersError> {
        let root = NodeHash::new(*root.as_bytes());
        match &self.0 {
            Kind::Vacant(vacancy) => vacancy.verify::<TreeMember, H>(&root, id).map_err(|error| match error {
                TreeError::ProofLeafHeld => OrgMembersError::AbsenceProofMalformed,
                other => other.into(),
            }),
            Kind::Held { member, inclusion, device_root_absence } => {
                if member.id != *id {
                    return Err(OrgMembersError::AbsenceProofRootMismatch);
                }
                inclusion.verify::<ProvenMember, H>(&root, member)?;
                device_root_absence.verify::<DeviceLeaf, H::DeviceRootHasher>(&member.device_root, device)?;
                Ok(())
            }
        }
    }
}

/// Parsing goes through each field's validating `Deserialize` and each inner
/// proof's bounded parse; whatever they refuse is `AbsenceProofMalformed`,
/// which a postcard caller sees as its custom error (LLR-9g2kjg).
#[cfg(feature = "serde")]
mod wire {
    use serde::de::Error as _;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{AbsenceProof, Kind};
    use crate::error::OrgMembersError;

    impl Serialize for AbsenceProof {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            self.0.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for AbsenceProof {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            Kind::deserialize(deserializer)
                .map(AbsenceProof)
                // Whatever a field or inner proof refuses is one refusal,
                // `AbsenceProofMalformed` (LLR-9g2kjg).
                .map_err(|_| D::Error::custom(OrgMembersError::AbsenceProofMalformed))
        }
    }
}

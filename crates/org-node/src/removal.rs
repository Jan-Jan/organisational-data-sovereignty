//! SDD-z4m57w: the node's own removal and what it keeps. A store whose Persona verified its own
//! removal holds only the signed acknowledgements, their recipients' keys and addresses and the time of
//! the verification (LLR-2d5a7m, LLR-6zaeb6). Values only: no file, no keychain, no clock.
use org_members::{DevicePublicKey, MemberId, RootHash};
use serde::{Deserialize, Serialize};

use crate::error::OrgNodeError;
use crate::ids::OrgId;
use crate::invitation::EndpointAddresses;
use crate::revocation::{Acknowledgement, Signature64};
use crate::store::{parse_field, MemberSnapshot, StoreData};
use crate::types::Epoch;
use crate::verify_org::VerificationTime;

/// A Persona store's plaintext (LLR-6zaeb6): postcard writes the variant index, then the fields.
/// `Serialize` in test builds only, as `StoreData`'s; `Deserialize` through `StoreData` and
/// `RemovedStore`'s raw mirrors (`unseal` decodes each form itself).
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "test-support", derive(Serialize))]
// One content per open store, never in a collection: the size gap between the forms costs nothing,
// and boxing `Active` would put an indirection on every operation's read.
#[allow(clippy::large_enum_variant)]
pub enum StoreContent {
    /// Index 0: the Persona's view of its Organisation.
    Active(StoreData),
    /// Index 1: what is left after the Persona's own removal.
    Removed(RemovedStore),
}

#[cfg(feature = "test-support")]
impl PartialEq for StoreContent {
    /// Compared by encoding (test builds only): `StoreData` holds secret types and has no `PartialEq`.
    /// A content that does not encode equals nothing, itself included, so a
    /// comparison of two such contents fails rather than passing as `None == None`.
    fn eq(&self, other: &Self) -> bool {
        match (postcard::to_allocvec(self), postcard::to_allocvec(other)) {
            (Ok(own), Ok(theirs)) => own == theirs,
            _ => false,
        }
    }
}

/// The removed store (LLR-6zaeb6): in this order the Organisation, the time of the verification that
/// found the removal, and the acknowledgements still to deliver.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawRemovedStore")]
pub struct RemovedStore {
    pub org_id: OrgId,
    pub verified_at: VerificationTime,
    pub acknowledgements: Vec<KeptAcknowledgement>,
}

/// One acknowledgement kept for one recipient, with that recipient's addresses when the store kept any
/// (LLR-6zaeb6, LLR-6fsnma).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct KeptAcknowledgement {
    pub ack: Acknowledgement,
    pub recipient: DevicePublicKey,
    pub recipient_addresses: Option<EndpointAddresses>,
}

/// The removed store as decoded, before parsing (LLR-6zaeb6; the `Raw…` rule of LLR-7yu2xy).
#[derive(Deserialize)]
pub(crate) struct RawRemovedStore {
    org_id: OrgId,
    verified_at: VerificationTime,
    acknowledgements: Vec<RawKeptAcknowledgement>,
}

#[derive(Deserialize)]
pub(crate) struct RawKeptAcknowledgement {
    ack: RawAcknowledgement,
    recipient: [u8; 32],
    recipient_addresses: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub(crate) struct RawAcknowledgement {
    org_id: OrgId,
    member_id: MemberId,
    device: [u8; 32],
    epoch: Epoch,
    root: RootHash,
    signature: Signature64,
}

impl RawAcknowledgement {
    /// The acknowledgement with its Device key re-parsed, a refusal naming `field`.
    pub(crate) fn parse(self, field: &'static str) -> Result<Acknowledgement, OrgNodeError> {
        Ok(Acknowledgement {
            org_id: self.org_id,
            member_id: self.member_id,
            device: parse_field(field, DevicePublicKey::parse(&self.device))?,
            epoch: self.epoch,
            root: self.root,
            signature: self.signature,
        })
    }
}

impl TryFrom<RawRemovedStore> for RemovedStore {
    type Error = OrgNodeError;

    fn try_from(raw: RawRemovedStore) -> Result<Self, Self::Error> {
        let acknowledgements = raw
            .acknowledgements
            .into_iter()
            .map(|kept| {
                Ok(KeptAcknowledgement {
                    ack: kept.ack.parse("removed.ack.device")?,
                    recipient: parse_field("removed.recipient", DevicePublicKey::parse(&kept.recipient))?,
                    recipient_addresses: kept
                        .recipient_addresses
                        .map(|addresses| parse_field("removed.recipient_addresses", EndpointAddresses::parse(addresses)))
                        .transpose()?,
                })
            })
            .collect::<Result<_, OrgNodeError>>()?;
        Ok(Self { org_id: raw.org_id, verified_at: raw.verified_at, acknowledgements })
    }
}

impl StoreData {
    /// Consume the store, keeping only the Organisation, the verification time and `acknowledgements`
    /// (LLR-2d5a7m). Everything else — the record and its key pair, the Organisation name, every
    /// provisional change and organisation, the founding, the provisional Member, every expectation,
    /// invite, kept request and notice, the revoked Devices, the address book and the Persona with its
    /// member seed — is dropped with `self`. Called from one place, `OrgService::seal_removal`
    /// (LLR-9jn3z8). A store with no record is refused: `OrgNotHeld` when it is bound to an
    /// Organisation, `OrgNotOnChain` when it is bound to none.
    pub fn into_removed(
        self,
        acknowledgements: Vec<KeptAcknowledgement>,
        verified_at: VerificationTime,
    ) -> Result<RemovedStore, OrgNodeError> {
        let org_id = match (&self.record, self.persona.org_id) {
            (Some(record), _) => record.org_id,
            (None, Some(org_id)) => return Err(OrgNodeError::OrgNotHeld { org_id }),
            (None, None) => return Err(OrgNodeError::OrgNotOnChain),
        };
        Ok(RemovedStore { org_id, verified_at, acknowledgements })
    }
}

impl RemovedStore {
    /// Remove the kept acknowledgement whose recipient is `recipient`, with its key and addresses;
    /// true when one was kept (LLR-85kves). Called only from `OrgService::acknowledgement_accepted`.
    pub(crate) fn remove_acknowledgement(&mut self, recipient: &DevicePublicKey) -> bool {
        match self.acknowledgements.iter().position(|kept| &kept.recipient == recipient) {
            Some(position) => {
                self.acknowledgements.remove(position);
                true
            }
            None => false,
        }
    }
}

/// How many recipients to take from `candidate_count` candidates (LLR-6fsnma's `k` for `n`): 3 for five
/// or more, 2 for three or four, 1 for one or two, 0 for none.
pub(crate) fn recipient_count(candidate_count: usize) -> usize {
    match candidate_count {
        0 => 0,
        1 | 2 => 1,
        3 | 4 => 2,
        _ => 3,
    }
}

/// The recipients of a removal (LLR-6fsnma). The candidates are the distinct DevicePublicKeys that
/// `previous` lists other than `own`. For a removal by a match (`matched` is `Some`), the ones the
/// match still lists come first, then the others; for a removal by a notice, all of them. Each group
/// is in ascending order of the key's bytes, and the first `recipient_count` of the candidate count
/// are taken. When `delivering`, the Device the removal came from, is a candidate, it is the first,
/// followed by the first `recipient_count` − 1 of the others in that order.
pub(crate) fn recipients(
    previous: &[MemberSnapshot],
    own: &DevicePublicKey,
    matched: Option<&[MemberSnapshot]>,
    delivering: Option<&DevicePublicKey>,
) -> Vec<DevicePublicKey> {
    let mut candidates: Vec<DevicePublicKey> = Vec::new();
    for device in previous.iter().flat_map(|member| member.device_keys.iter()) {
        if device != own && !candidates.contains(device) {
            candidates.push(*device);
        }
    }
    let taken = recipient_count(candidates.len());
    candidates.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    let still_listed = |device: &DevicePublicKey| {
        matched.is_some_and(|members| members.iter().any(|member| member.device_keys.contains(device)))
    };
    let (mut ordered, rest): (Vec<_>, Vec<_>) = candidates.into_iter().partition(|device| still_listed(device));
    ordered.extend(rest);
    // A candidate exists only when `taken` is not zero, so a found delivering Device is always taken.
    if let Some(delivering_key) = delivering {
        if let Some(position) = ordered.iter().position(|device| device == delivering_key) {
            let first = ordered.remove(position);
            ordered.insert(0, first);
        }
    }
    ordered.truncate(taken);
    ordered
}

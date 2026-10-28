//! Proofs under a Person hash (SDD-bx9mm6): produced from a Person, parsed,
//! and checked against a Person hash, keeping nothing between calls.

use sparse_merkle_tree::{AbsenceProof, NodeHash, TreeError};

use crate::definition::Person;
use crate::device_key::DevicePublicKey;
use crate::device_root::{compute_device_root, prove_device_absent_in, DeviceLeaf, DEVICE_ROOT_DEPTH};
use crate::device_root_hasher::PersonDeviceRootHasher;
use crate::encoding::EncodingVersion;
use crate::error::IdentityError;
use crate::hash::{encode_v1, hash_encoding, PersonHash};
use crate::name::{Name, Surname};
use crate::person_key::PersonPublicKey;

/// A proof that a DevicePublicKey is not held under a Person hash: the
/// definition's fields but its devices, its device root, and an absence proof
/// of depth 4 under that root. It holds no DevicePublicKey. Its fields are
/// private; it comes from `prove_device_absent` or decoding. LLR-985v9t.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonAbsenceProof {
    version: EncodingVersion,
    name: Name,
    surname: Surname,
    person_key: Option<PersonPublicKey>,
    device_root: NodeHash,
    absence: AbsenceProof<DEVICE_ROOT_DEPTH>,
}

/// A proof under `V1` that `person` does not hold `device`, or
/// `DeviceStillHeld` when it does. LLR-cve3d6.
pub fn prove_device_absent(person: &Person, device: &DevicePublicKey) -> Result<PersonAbsenceProof, IdentityError> {
    let devices = person.devices().devices();
    let absence = prove_device_absent_in::<PersonDeviceRootHasher>(devices, device)?;
    Ok(PersonAbsenceProof {
        version: EncodingVersion::V1,
        name: person.name().clone(),
        surname: person.surname().clone(),
        person_key: person.person_key().copied(),
        device_root: compute_device_root::<PersonDeviceRootHasher>(devices)?,
        absence,
    })
}

impl PersonAbsenceProof {
    pub fn version(&self) -> EncodingVersion {
        self.version
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn surname(&self) -> &Surname {
        &self.surname
    }

    pub fn person_key(&self) -> Option<&PersonPublicKey> {
        self.person_key.as_ref()
    }

    pub fn device_root(&self) -> &NodeHash {
        &self.device_root
    }

    pub fn device_root_absence(&self) -> &AbsenceProof<DEVICE_ROOT_DEPTH> {
        &self.absence
    }

    /// `PersonHashMismatch` unless the proof's fields hash to `hash` under its
    /// version, before any other check; then `DeviceRootMismatch` unless the
    /// absence proof resolves to the device root for `device`, and
    /// `DeviceStillHeld` when it shows `device` held. LLR-z8h2w8.
    pub fn verify(&self, hash: &PersonHash, device: &DevicePublicKey) -> Result<(), IdentityError> {
        let encoding = match self.version {
            EncodingVersion::V1 => encode_v1(&self.name, &self.surname, self.person_key.as_ref(), &self.device_root),
        };
        if hash_encoding(self.version, &encoding) != *hash {
            return Err(IdentityError::PersonHashMismatch);
        }
        self.absence
            .verify::<DeviceLeaf, PersonDeviceRootHasher>(&self.device_root, device)
            .map_err(|error| match error {
                TreeError::ProofLeafHeld => IdentityError::DeviceStillHeld,
                // Only `ProofRootMismatch` arrives here: the proof was parsed
                // through `from_parts` (no `ProofMalformed`) at depth 4 (no
                // `LevelOutOfRange`), and `verify` returns no other variant.
                _ => IdentityError::DeviceRootMismatch,
            })
    }
}

/// The serde form: a struct of the six fields, the version as its number.
/// Every refusal, of any field, is `ProofMalformed` (LLR-985v9t, LLR-3dvfjb).
#[cfg(feature = "serde")]
mod wire {
    use serde::de::Error as _;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use sparse_merkle_tree::{AbsenceProof, NodeHash};

    use super::PersonAbsenceProof;
    use crate::device_root::DEVICE_ROOT_DEPTH;
    use crate::encoding::EncodingVersion;
    use crate::error::IdentityError;
    use crate::name::{Name, Surname};
    use crate::person_key::PersonPublicKey;

    #[derive(Serialize)]
    #[serde(rename = "PersonAbsenceProof")]
    struct Outgoing<'a> {
        version: u16,
        name: &'a Name,
        surname: &'a Surname,
        person_key: Option<&'a PersonPublicKey>,
        device_root: &'a NodeHash,
        absence: &'a AbsenceProof<DEVICE_ROOT_DEPTH>,
    }

    #[derive(Deserialize)]
    #[serde(rename = "PersonAbsenceProof", deny_unknown_fields)]
    struct Incoming {
        version: u16,
        name: Name,
        surname: Surname,
        person_key: Option<PersonPublicKey>,
        device_root: NodeHash,
        absence: AbsenceProof<DEVICE_ROOT_DEPTH>,
    }

    impl Serialize for PersonAbsenceProof {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            Outgoing {
                version: u16::from(self.version),
                name: &self.name,
                surname: &self.surname,
                person_key: self.person_key.as_ref(),
                device_root: &self.device_root,
                absence: &self.absence,
            }
            .serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for PersonAbsenceProof {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            // Any field's decoding error, the absence proof's `TreeError::ProofMalformed`
            // among them, is one refusal (LLR-985v9t).
            let malformed = |_| D::Error::custom(IdentityError::ProofMalformed);
            let fields = Incoming::deserialize(deserializer).map_err(malformed)?;
            // Only `UnsupportedEncodingVersion` arrives; no `TreeError` can.
            let version = EncodingVersion::try_from(fields.version).map_err(|_| D::Error::custom(IdentityError::ProofMalformed))?;
            Ok(Self {
                version,
                name: fields.name,
                surname: fields.surname,
                person_key: fields.person_key,
                device_root: fields.device_root,
                absence: fields.absence,
            })
        }
    }
}

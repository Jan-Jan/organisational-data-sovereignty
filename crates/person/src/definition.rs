//! The Person definition: an individual's own record. SDD-4r2x79.

use crate::error::IdentityError;
use crate::group_key::check_group_key;
use crate::name::{Name, Surname};
use crate::person_key::PersonPublicKey;
use crate::slots::DeviceSlots;

/// A validated Person definition: a name, a surname, an optional
/// PersonPublicKey and at most `MAX_DEVICES` DevicePublicKeys held as device
/// slots (LLR-4ku56h). `Person::new` is the only way to build one, decoding
/// included (LLR-63pkfc).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Person {
    name: Name,
    surname: Surname,
    person_key: Option<PersonPublicKey>,
    devices: DeviceSlots,
}

impl Person {
    /// Refuses a definition whose PersonPublicKey breaks the group-key rules
    /// against its device slots (`check_group_key`). LLR-sjkmr6, LLR-kbhc43.
    pub fn new(
        name: Name,
        surname: Surname,
        person_key: Option<PersonPublicKey>,
        devices: DeviceSlots,
    ) -> Result<Self, IdentityError> {
        check_group_key(person_key.as_ref(), &devices)?;
        Ok(Self {
            name,
            surname,
            person_key,
            devices,
        })
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

    pub fn devices(&self) -> &DeviceSlots {
        &self.devices
    }
}

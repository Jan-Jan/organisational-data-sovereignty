//! `Person`'s serde implementations, compiled only with the `serde` feature.
//! Decoding goes through each field's own decoder and then `Person::new`.
//! LLR-63pkfc.

use serde::ser::SerializeStruct;

use crate::definition::Person;
use crate::name::{Name, Surname};
use crate::person_key::PersonPublicKey;
use crate::slots::DeviceSlots;

impl serde::Serialize for Person {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut fields = serializer.serialize_struct("Person", 4)?;
        fields.serialize_field("name", self.name())?;
        fields.serialize_field("surname", self.surname())?;
        fields.serialize_field("person_key", &self.person_key())?;
        fields.serialize_field("devices", self.devices())?;
        fields.end()
    }
}

/// The four fields as decoded, each through its own validating decoder, not
/// yet checked against one another.
#[derive(serde::Deserialize)]
#[serde(rename = "Person", deny_unknown_fields)]
struct PersonFields {
    name: Name,
    surname: Surname,
    person_key: Option<PersonPublicKey>,
    devices: DeviceSlots,
}

impl<'de> serde::Deserialize<'de> for Person {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let fields = PersonFields::deserialize(deserializer)?;
        Person::new(
            fields.name,
            fields.surname,
            fields.person_key,
            fields.devices,
        )
        .map_err(serde::de::Error::custom)
    }
}

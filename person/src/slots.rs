use alloc::vec::Vec;
use core::fmt;

use crate::device_key::DevicePublicKey;
use crate::device_root::same_device_position;
use crate::error::IdentityError;

/// Maximum number of devices, held below the device root tree's 16 positions
/// (owner ruling 2026-10-23). LLR-xsx6gg.
pub const MAX_DEVICES: usize = 8;

/// A bounded, sorted, duplicate-free set of DevicePublicKeys at distinct device
/// positions. LLR-xsx6gg.
#[derive(Clone, PartialEq, Eq)]
pub struct DeviceSlots {
    slots: Vec<DevicePublicKey>,
}

impl DeviceSlots {
    pub fn parse(mut devices: Vec<DevicePublicKey>) -> Result<Self, IdentityError> {
        if devices.len() > MAX_DEVICES {
            return Err(IdentityError::DeviceSlotsFull);
        }
        devices.sort();
        if devices
            .windows(2)
            .any(|pair| matches!(pair, [first, second] if first == second))
        {
            return Err(IdentityError::DuplicateDevice);
        }
        if devices.iter().any(|key| position_shared(&devices, key)) {
            return Err(IdentityError::DevicePositionTaken);
        }
        Ok(Self { slots: devices })
    }

    pub fn devices(&self) -> &[DevicePublicKey] {
        &self.slots
    }

    pub fn has_device(&self, device: &DevicePublicKey) -> bool {
        self.slots.binary_search(device).is_ok()
    }

    pub fn device_count(&self) -> usize {
        self.slots.len()
    }

    /// Whether a key of the set other than `candidate` has `candidate`'s
    /// device position. LLR-vraz7n.
    pub fn position_taken(&self, candidate: &DevicePublicKey) -> bool {
        position_shared(&self.slots, candidate)
    }

    pub fn add_device(&self, device: DevicePublicKey) -> Result<Self, IdentityError> {
        if self.slots.len() >= MAX_DEVICES {
            return Err(IdentityError::DeviceSlotsFull);
        }
        if self.has_device(&device) {
            return Err(IdentityError::DuplicateDevice);
        }
        if self.position_taken(&device) {
            return Err(IdentityError::DevicePositionTaken);
        }
        let mut slots = self.slots.clone();
        slots.push(device);
        slots.sort();
        Ok(Self { slots })
    }

    pub fn remove_device(&self, device: &DevicePublicKey) -> Result<Self, IdentityError> {
        let index = self
            .slots
            .binary_search(device)
            .map_err(|_| IdentityError::DeviceNotFound)?;
        let mut slots = self.slots.clone();
        slots.remove(index);
        Ok(Self { slots })
    }
}

/// Whether a key of `keys` other than `candidate` has `candidate`'s device
/// position.
fn position_shared(keys: &[DevicePublicKey], candidate: &DevicePublicKey) -> bool {
    keys.iter().any(|held| held != candidate && same_device_position(held, candidate))
}

impl TryFrom<Vec<DevicePublicKey>> for DeviceSlots {
    type Error = IdentityError;

    fn try_from(devices: Vec<DevicePublicKey>) -> Result<Self, Self::Error> {
        Self::parse(devices)
    }
}

impl fmt::Debug for DeviceSlots {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "DeviceSlots({})", self.slots.len())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for DeviceSlots {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.slots.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DeviceSlots {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SlotsVisitor;

        impl<'de> serde::de::Visitor<'de> for SlotsVisitor {
            type Value = DeviceSlots;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    formatter,
                    "at most {MAX_DEVICES} DevicePublicKeys in strictly increasing order, at distinct device positions"
                )
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut sequence: A,
            ) -> Result<DeviceSlots, A::Error> {
                let mut slots: Vec<DevicePublicKey> = Vec::with_capacity(MAX_DEVICES);
                while let Some(device) = sequence.next_element::<DevicePublicKey>()? {
                    if slots.len() == MAX_DEVICES {
                        return Err(serde::de::Error::custom("device slots exceed MAX_DEVICES"));
                    }
                    if slots.last().is_some_and(|last| *last >= device) {
                        return Err(serde::de::Error::custom(
                            "device slots must be strictly increasing (sorted, no duplicates)",
                        ));
                    }
                    if position_shared(&slots, &device) {
                        return Err(serde::de::Error::custom("two DevicePublicKeys share a device position"));
                    }
                    slots.push(device);
                }
                Ok(DeviceSlots { slots })
            }
        }

        deserializer.deserialize_seq(SlotsVisitor)
    }
}

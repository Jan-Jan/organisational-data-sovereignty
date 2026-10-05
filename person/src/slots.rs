use alloc::vec::Vec;
use core::fmt;

use crate::device_key::DevicePublicKey;
use crate::error::IdentityError;

/// Maximum number of devices: the device sub-trie has four slots.
pub const MAX_DEVICES: usize = 4;

/// A bounded, sorted, duplicate-free set of device keys. LLR-9p34cv.
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

    pub fn add_device(&self, device: DevicePublicKey) -> Result<Self, IdentityError> {
        if self.slots.len() >= MAX_DEVICES {
            return Err(IdentityError::DeviceSlotsFull);
        }
        if self.has_device(&device) {
            return Err(IdentityError::DuplicateDevice);
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

    /// The slots padded with `None` to `MAX_DEVICES`, in sorted order.
    pub(crate) fn to_fixed_slots(&self) -> [Option<DevicePublicKey>; MAX_DEVICES] {
        let mut fixed = [None; MAX_DEVICES];
        for (slot, device) in fixed.iter_mut().zip(self.slots.iter()) {
            *slot = Some(*device);
        }
        fixed
    }
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
                    "at most {MAX_DEVICES} device keys in strictly increasing order"
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
                    slots.push(device);
                }
                Ok(DeviceSlots { slots })
            }
        }

        deserializer.deserialize_seq(SlotsVisitor)
    }
}

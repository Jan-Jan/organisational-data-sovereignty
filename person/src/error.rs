/// Every rejection by a constructor in this crate. Decoding rejects through
/// the deserializer's error type, with this type's message where the rule is
/// one of these. LLR-eeq89n.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IdentityError {
    #[error("field too long: {field} exceeds {max} bytes after NFC normalization")]
    FieldTooLong { field: &'static str, max: usize },

    #[error("invalid device public key")]
    InvalidDeviceKey,

    #[error("invalid person public key")]
    InvalidPersonKey,

    #[error("device slots full (max 4)")]
    DeviceSlotsFull,

    #[error("duplicate device")]
    DuplicateDevice,

    #[error("device not found")]
    DeviceNotFound,
}

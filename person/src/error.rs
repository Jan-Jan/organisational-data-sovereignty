/// Every rejection by a constructor or a Person operation in this crate.
/// Decoding rejects through the deserializer's error type, with this type's
/// message where the rule is one of these. LLR-eeq89n, LLR-3n3kxx.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IdentityError {
    #[error("field too long: {field} exceeds {max} bytes after NFC normalization")]
    FieldTooLong { field: &'static str, max: usize },

    #[error("invalid device public key")]
    InvalidDeviceKey,

    #[error("invalid person public key")]
    InvalidPersonKey,

    #[error("device slots full (max 8)")]
    DeviceSlotsFull,

    #[error("duplicate device")]
    DuplicateDevice,

    #[error("device not found")]
    DeviceNotFound,

    /// Two DevicePublicKeys of one set at one device position. LLR-xsx6gg.
    #[error("device position taken by another device public key")]
    DevicePositionTaken,

    /// A proof of absence asked for a DevicePublicKey the Person holds.
    /// LLR-cve3d6, LLR-z8h2w8.
    #[error("device public key still held")]
    DeviceStillHeld,

    /// A Person absence proof whose fields do not hash to the given Person hash.
    /// LLR-z8h2w8.
    #[error("proof does not resolve to the person hash")]
    PersonHashMismatch,

    /// A Person absence proof whose absence proof does not resolve to its
    /// device root. LLR-z8h2w8.
    #[error("proof does not resolve to the device root")]
    DeviceRootMismatch,

    /// A Person absence proof that does not decode. LLR-985v9t, LLR-3dvfjb.
    #[error("malformed proof")]
    ProofMalformed,

    #[error("person public key held with no device public key")]
    KeyWithoutDevice,

    #[error("person public key missing: one or more device public keys are held")]
    MissingPersonKey,

    #[error("person public key equals a device public key")]
    PersonKeyIsDeviceKey,

    #[error("person public key not rotated: the device public keys changed and it did not")]
    PersonKeyNotRotated,

    #[error("unsupported encoding version {0}")]
    UnsupportedEncodingVersion(u16),
}

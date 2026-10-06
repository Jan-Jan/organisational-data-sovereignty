use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OrgMembersError {
    #[error("duplicate handle")]
    DuplicateHandle,

    #[error("duplicate member id")]
    DuplicateId,

    #[error("member id not found")]
    IdNotFound,

    #[error("invalid handle: {0}")]
    InvalidHandle(String),

    #[error("confusable handle")]
    ConfusableHandle,

    #[error("duplicate device")]
    DuplicateDevice,

    #[error("device not found")]
    DeviceNotFound,

    #[error("replacement p2p key equals the current key")]
    P2pKeyNotReplaced,

    /// A member key or device key that would be held in two places in this
    /// organisation: already held by another member, as any enrolled device
    /// (for a replacement key, the device the operation removes included), or
    /// by the same leaf as both member key and device. Keys no longer held are
    /// not refused. LLR-v6gfc7, LLR-fym7dy, LLR-gjj6bx.
    #[error("key already held in this organisation")]
    DuplicateKey,

    #[error("invalid device public key")]
    InvalidDeviceKey,

    #[error("invalid person public key")]
    InvalidPersonKey,

    #[error("device slots full (max 4)")]
    DeviceSlotsFull,

    #[error("member must have at least one device")]
    EmptyDeviceList,

    #[error("delta base root mismatch")]
    DeltaBaseMismatch,

    #[error("verification failed")]
    VerificationFailed,

    #[error("serialization error")]
    SerializationError,

    #[error("hashes not calculated")]
    HashesNotCalculated,

    /// `recalculate()` on a calculated trie, one not mutated since it was
    /// calculated: its node hashes are write-once and already filled.
    /// LLR-j35sxz.
    #[error("hashes already calculated")]
    HashesAlreadyCalculated,

    /// A bit index of 256 or more given to `MemberId::bit`, or a level of 257
    /// or more given to `DefaultHashes::at_level`. LLR-7jkcba (PR-jq43gx).
    #[error("index out of range")]
    IndexOutOfRange,

    /// The Member under an absence proof's MemberId holds the Device key.
    /// LLR-4xz255, LLR-p2p8qy.
    #[error("device key still held by the member")]
    DeviceStillHeld,

    /// An absence proof whose sibling hashes do not match its default map.
    /// LLR-4rju5r.
    #[error("malformed absence proof")]
    AbsenceProofMalformed,

    /// An absence proof that does not resolve to the root it was checked
    /// against. LLR-dgzy7e.
    #[error("absence proof does not resolve to the root")]
    AbsenceProofRootMismatch,

    #[error("internal invariant violated")]
    InvariantViolated,

    #[error("malformed delta: {0}")]
    MalformedDelta(&'static str),

    #[error("field too long: {field} exceeds {max} bytes after NFC normalization")]
    FieldTooLong { field: &'static str, max: usize },
}

/// Exhaustive on purpose, with no wildcard arm: a variant `person` adds stops
/// this crate compiling until its mapping is chosen. LLR-28ekrv.
impl From<person::IdentityError> for OrgMembersError {
    fn from(e: person::IdentityError) -> Self {
        use person::IdentityError as I;
        match e {
            I::FieldTooLong { field, max } => Self::FieldTooLong { field, max },
            I::InvalidDeviceKey => Self::InvalidDeviceKey,
            I::InvalidPersonKey => Self::InvalidPersonKey,
            I::DeviceSlotsFull => Self::DeviceSlotsFull,
            I::DuplicateDevice => Self::DuplicateDevice,
            I::DeviceNotFound => Self::DeviceNotFound,
            // Only person's Person operations produce these, and org-members
            // calls none of them. Provisional: see LLR-28ekrv.
            I::KeyWithoutDevice
            | I::MissingPersonKey
            | I::PersonKeyIsDeviceKey
            | I::PersonKeyNotRotated
            | I::UnsupportedEncodingVersion(_) => Self::InvariantViolated,
        }
    }
}

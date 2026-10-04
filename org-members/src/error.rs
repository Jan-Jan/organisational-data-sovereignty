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

    #[error("internal invariant violated")]
    InvariantViolated,

    #[error("malformed delta: {0}")]
    MalformedDelta(&'static str),

    #[error("field too long: {field} exceeds {max} bytes after NFC normalization")]
    FieldTooLong { field: &'static str, max: usize },
}

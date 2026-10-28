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

    #[error("device slots full (max 8)")]
    DeviceSlotsFull,

    /// A DevicePublicKey at a device position another DevicePublicKey of the
    /// member holds.
    /// LLR-5jh6t3.
    #[error("device position taken by another device public key of the member")]
    DevicePositionTaken,

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

    /// The Member under an absence proof's MemberId holds the Device key.
    /// LLR-bh53rr, LLR-25nmwc.
    #[error("device key still held by the member")]
    DeviceStillHeld,

    /// An absence proof that does not parse, or a Vacant proof ending in the
    /// leaf of the MemberId asked about. LLR-9g2kjg, LLR-25nmwc.
    #[error("malformed absence proof")]
    AbsenceProofMalformed,

    /// An absence proof that does not resolve to the root it was checked
    /// against. LLR-54gp2f.
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
/// this crate compiling until its mapping is chosen. LLR-hbb4je.
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
            I::DevicePositionTaken => Self::DevicePositionTaken,
            I::DeviceStillHeld => Self::DeviceStillHeld,
            // Only person's Person operations and Person-hash proofs produce
            // these, and org-members calls none of them (LLR-hbb4je).
            I::KeyWithoutDevice
            | I::MissingPersonKey
            | I::PersonKeyIsDeviceKey
            | I::PersonKeyNotRotated
            | I::UnsupportedEncodingVersion(_)
            | I::PersonHashMismatch
            | I::DeviceRootMismatch
            | I::ProofMalformed => Self::InvariantViolated,
        }
    }
}

/// Exhaustive on purpose: every refusal of the tree reaches this crate's
/// callers as one of its own variants, never as `TreeError`. LLR-w5yahk.
impl From<sparse_merkle_tree::TreeError> for OrgMembersError {
    fn from(error: sparse_merkle_tree::TreeError) -> Self {
        use sparse_merkle_tree::TreeError;
        match error {
            TreeError::HashesNotCalculated => Self::HashesNotCalculated,
            TreeError::HashesAlreadyCalculated => Self::HashesAlreadyCalculated,
            TreeError::ProofMalformed => Self::AbsenceProofMalformed,
            TreeError::ProofRootMismatch => Self::AbsenceProofRootMismatch,
            TreeError::ProofLeafHeld => Self::DeviceStillHeld,
            // A Member's position is its MemberId and `DuplicateId` and
            // `IdNotFound` are checked first, so none of the first three
            // reaches here. Nor does `LevelOutOfRange`: the tree raises it
            // only for a level above its depth, org-members asks for no
            // level, and every level the tree walks is its own, bounded by
            // its depth.
            TreeError::Collision | TreeError::LeafNotHeld | TreeError::LeafHeld | TreeError::LevelOutOfRange => {
                Self::InvariantViolated
            }
        }
    }
}

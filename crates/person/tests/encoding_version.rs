//! EncodingVersion: the closed set of Person hash encodings this unit implements.

use person::encoding::EncodingVersion;
use person::IdentityError;

/// verifies: LLR-rde6tk
#[test]
fn version_one_is_v1_and_back() {
    assert_eq!(EncodingVersion::try_from(1u16), Ok(EncodingVersion::V1));
    assert_eq!(u16::from(EncodingVersion::V1), 1);
}

/// verifies: LLR-rde6tk, LLR-3n3kxx
#[test]
fn every_other_version_is_refused_and_named() {
    for value in [0u16, 2, 3, 255, 256, u16::MAX] {
        assert_eq!(
            EncodingVersion::try_from(value),
            Err(IdentityError::UnsupportedEncodingVersion(value)),
            "version {value}"
        );
    }
    assert_eq!(
        IdentityError::UnsupportedEncodingVersion(2).to_string(),
        "unsupported encoding version 2"
    );
}

//! How a `person::IdentityError` becomes an `OrgMembersError` (LLR-28ekrv).
//!
//! The five variants only `person`'s Person operations produce map to
//! `InvariantViolated`, since org-members calls none of those operations.
//! The variants shared with org-members keep their names and fields; their
//! constructor tests (`newtypes.rs`, `device_key_decoding.rs`,
//! `member_key_decoding.rs`) cover them, and this file checks them once more
//! so the whole mapping is stated in one place.

use org_members::OrgMembersError;
use person::IdentityError;

// verifies: LLR-28ekrv
#[test]
fn person_only_variants_map_to_invariant_violated() {
    let person_only_variants = [
        IdentityError::KeyWithoutDevice,
        IdentityError::MissingPersonKey,
        IdentityError::PersonKeyIsDeviceKey,
        IdentityError::PersonKeyNotRotated,
        IdentityError::UnsupportedEncodingVersion(0),
        IdentityError::UnsupportedEncodingVersion(2),
        IdentityError::UnsupportedEncodingVersion(u16::MAX),
    ];
    for variant in person_only_variants {
        assert_eq!(
            OrgMembersError::from(variant.clone()),
            OrgMembersError::InvariantViolated,
            "{variant:?}"
        );
    }
}

// verifies: LLR-28ekrv
#[test]
fn shared_variants_keep_their_names_and_fields() {
    let field_too_long = IdentityError::FieldTooLong { field: "name", max: 128 };
    let expected = [
        (field_too_long, OrgMembersError::FieldTooLong { field: "name", max: 128 }),
        (IdentityError::InvalidDeviceKey, OrgMembersError::InvalidDeviceKey),
        (IdentityError::InvalidPersonKey, OrgMembersError::InvalidPersonKey),
        (IdentityError::DeviceSlotsFull, OrgMembersError::DeviceSlotsFull),
        (IdentityError::DuplicateDevice, OrgMembersError::DuplicateDevice),
        (IdentityError::DeviceNotFound, OrgMembersError::DeviceNotFound),
    ];
    for (person_error, org_members_error) in expected {
        assert_eq!(OrgMembersError::from(person_error), org_members_error);
    }
}

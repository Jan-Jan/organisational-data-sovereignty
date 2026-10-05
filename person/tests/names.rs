//! Name and Surname: NFC normalization, the byte bound, redacted Debug.

// A failed expect is the test failing; the panic-denying lints guard the
// library, not its tests.
#![allow(clippy::expect_used)]

use person::{IdentityError, Name, Surname, MAX_NAME_LEN, MAX_SURNAME_LEN};

/// verifies: LLR-ayu93n, REQ-r7mytp
#[test]
fn names_are_stored_in_nfc() {
    let decomposed = "Jose\u{0301}"; // e + combining acute
    let name = Name::parse(decomposed).expect("valid name");
    assert_eq!(name.as_str(), "Jos\u{00e9}");
    let surname = Surname::parse(decomposed).expect("valid surname");
    assert_eq!(surname.as_str(), "Jos\u{00e9}");
}

/// verifies: LLR-ayu93n, REQ-r7mytp
#[test]
fn the_bound_applies_after_normalisation_and_names_the_field() {
    let at_bound = "a".repeat(MAX_NAME_LEN);
    assert!(Name::parse(&at_bound).is_ok());
    let over = "a".repeat(MAX_NAME_LEN + 1);
    assert_eq!(
        Name::parse(&over),
        Err(IdentityError::FieldTooLong {
            field: "name",
            max: MAX_NAME_LEN
        })
    );
    let at_bound = "a".repeat(MAX_SURNAME_LEN);
    assert_eq!(
        Surname::parse(&at_bound).expect("at the bound").as_str(),
        at_bound
    );
    let over = "a".repeat(MAX_SURNAME_LEN + 1);
    assert_eq!(
        Surname::parse(&over),
        Err(IdentityError::FieldTooLong {
            field: "surname",
            max: MAX_SURNAME_LEN
        })
    );
}

/// A value over the bound only in its decomposed form is accepted: the bound
/// is measured on the NFC form, which is also what is stored.
///
/// verifies: LLR-ayu93n, REQ-r7mytp
#[test]
fn a_value_over_the_bound_only_before_normalisation_is_accepted() {
    let decomposed = "e\u{0301}".repeat(50); // e + combining acute, ×50
    let composed = "\u{00e9}".repeat(50);
    assert_eq!(decomposed.len(), 150);
    assert_eq!(composed.len(), 100);
    assert!(decomposed.len() > MAX_NAME_LEN && composed.len() <= MAX_NAME_LEN);
    assert!(decomposed.len() > MAX_SURNAME_LEN && composed.len() <= MAX_SURNAME_LEN);

    let name = Name::parse(&decomposed).expect("NFC form is within the bound");
    assert_eq!(name.as_str(), composed);
    let surname = Surname::parse(&decomposed).expect("NFC form is within the bound");
    assert_eq!(surname.as_str(), composed);
}

/// A value within the bound before normalisation and over it after is
/// rejected: U+0958 DEVANAGARI LETTER QA is a composition exclusion, so its
/// NFC form is U+0915 U+093C, twice its UTF-8 length. A bound checked before
/// normalisation would accept it.
///
/// verifies: LLR-ayu93n, REQ-r7mytp
#[test]
fn a_value_over_the_bound_only_after_normalisation_is_rejected() {
    let unnormalised = "\u{0958}".repeat(40);
    let nfc = "\u{0915}\u{093c}".repeat(40);
    assert_eq!(unnormalised.len(), 120);
    assert_eq!(nfc.len(), 240);
    assert!(unnormalised.len() <= MAX_NAME_LEN && nfc.len() > MAX_NAME_LEN);
    assert!(unnormalised.len() <= MAX_SURNAME_LEN && nfc.len() > MAX_SURNAME_LEN);

    assert_eq!(
        Name::parse(&unnormalised),
        Err(IdentityError::FieldTooLong {
            field: "name",
            max: MAX_NAME_LEN
        })
    );
    assert_eq!(
        Surname::parse(&unnormalised),
        Err(IdentityError::FieldTooLong {
            field: "surname",
            max: MAX_SURNAME_LEN
        })
    );
    let within = "\u{0958}".repeat(21); // NFC: 126 bytes
    assert_eq!(
        Name::parse(&within)
            .expect("NFC form within the bound")
            .as_str(),
        "\u{0915}\u{093c}".repeat(21)
    );
}

/// verifies: LLR-ayu93n, REQ-r7mytp
#[test]
fn try_from_goes_through_parse() {
    let over = "a".repeat(MAX_NAME_LEN + 1);
    assert!(Name::try_from(over.as_str()).is_err());
    assert!(Name::try_from(over).is_err());
    let decomposed = "Jose\u{0301}";
    assert_eq!(
        Name::try_from(decomposed).expect("valid").as_str(),
        "Jos\u{00e9}"
    );
}

/// verifies: LLR-ayu93n, REQ-r7mytp
#[test]
fn surname_try_from_goes_through_parse() {
    let over = "a".repeat(MAX_SURNAME_LEN + 1);
    let expected = Err(IdentityError::FieldTooLong {
        field: "surname",
        max: MAX_SURNAME_LEN,
    });
    assert_eq!(Surname::try_from(over.as_str()), expected);
    assert_eq!(Surname::try_from(over), expected);
    let decomposed = "Jose\u{0301}";
    assert_eq!(
        Surname::try_from(decomposed).expect("valid").as_str(),
        "Jos\u{00e9}"
    );
    assert_eq!(
        Surname::try_from(String::from(decomposed))
            .expect("valid")
            .as_str(),
        "Jos\u{00e9}"
    );
}

/// verifies: LLR-ayu93n, REQ-r7mytp
#[cfg(feature = "serde")]
#[test]
fn surname_decoding_goes_through_parse() {
    let over = "a".repeat(MAX_SURNAME_LEN + 1);
    let bytes = postcard::to_allocvec(&over).expect("encode string");
    assert!(postcard::from_bytes::<Surname>(&bytes).is_err());
    let decomposed = postcard::to_allocvec(&"Jose\u{0301}").expect("encode string");
    assert_eq!(
        postcard::from_bytes::<Surname>(&decomposed)
            .expect("decode")
            .as_str(),
        "Jos\u{00e9}"
    );
}

/// verifies: LLR-ayu93n, REQ-r7mytp
#[cfg(feature = "serde")]
#[test]
fn decoding_goes_through_parse() {
    let over = "a".repeat(MAX_NAME_LEN + 1);
    let bytes = postcard::to_allocvec(&over).expect("encode string");
    assert!(postcard::from_bytes::<Name>(&bytes).is_err());
    let decomposed = postcard::to_allocvec(&"Jose\u{0301}").expect("encode string");
    assert_eq!(
        postcard::from_bytes::<Name>(&decomposed)
            .expect("decode")
            .as_str(),
        "Jos\u{00e9}"
    );
}

/// verifies: LLR-fbqs2r
#[test]
fn debug_redacts_personal_data() {
    for value in ["Alice", "", "Jose\u{0301}"] {
        let name = Name::parse(value).expect("valid name");
        assert_eq!(format!("{name:?}"), "Name([REDACTED])");
        let surname = Surname::parse(value).expect("valid surname");
        assert_eq!(format!("{surname:?}"), "Surname([REDACTED])");
    }
}

/// Display is not redacted: it is how a caller gets the value out as text.
/// verifies: LLR-fbqs2r
#[test]
fn display_and_into_string_give_the_nfc_value() {
    let name = Name::parse("Jose\u{0301}").expect("valid name");
    assert_eq!(name.to_string(), "Jos\u{00e9}");
    assert_eq!(String::from(name), "Jos\u{00e9}");
    let surname = Surname::parse("Jose\u{0301}").expect("valid surname");
    assert_eq!(surname.to_string(), "Jos\u{00e9}");
    assert_eq!(String::from(surname), "Jos\u{00e9}");
}

/// verifies: LLR-5za6mp
#[cfg(feature = "serde")]
#[test]
fn names_encode_as_the_string_of_their_nfc_value() {
    let name = Name::parse("Jose\u{0301}").expect("valid name");
    let surname = Surname::parse("Jose\u{0301}").expect("valid surname");
    let expected = postcard::to_allocvec("Jos\u{00e9}").expect("encode string");
    assert_eq!(postcard::to_allocvec(&name).expect("encode"), expected);
    assert_eq!(postcard::to_allocvec(&surname).expect("encode"), expected);
}

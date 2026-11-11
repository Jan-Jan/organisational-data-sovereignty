//! Parsing an organisation identifier at the command boundary.
//!
//! Requirement carried here: REQ-sjkp8z throughout — a well-formed identifier
//! is accepted, and every malformed one is refused with a message rather than
//! panicking or silently truncating.

use ods_poc_lib::parsing::parse_org_id;
use org_io::node::OrgId;

// verifies: LLR-ecaw34
#[test]
fn forty_hex_characters_are_accepted() {
    assert_eq!(parse_org_id(&"aa".repeat(20)), Ok(OrgId::new([0xaa; 20])));
}

// verifies: LLR-ecaw34
#[test]
fn a_zero_x_prefix_is_accepted_and_parses_identically() {
    let bare = "ab".repeat(20);
    let prefixed = format!("0x{bare}");
    assert_eq!(parse_org_id(&prefixed), parse_org_id(&bare));
    assert_eq!(parse_org_id(&prefixed), Ok(OrgId::new([0xab; 20])));
}

// verifies: LLR-ecaw34
#[test]
fn uppercase_hex_is_accepted() {
    assert_eq!(parse_org_id(&"AB".repeat(20)), Ok(OrgId::new([0xab; 20])));
}

// verifies: LLR-ecaw34
#[test]
fn a_zero_x_prefix_with_forty_characters_after_it_is_accepted() {
    // 42 characters in total. The prefix is stripped BEFORE the width is
    // measured, so this is the well-formed case and not an over-long one.
    let s = format!("0x{}", "cd".repeat(20));
    assert_eq!(s.len(), 42);
    assert_eq!(parse_org_id(&s), Ok(OrgId::new([0xcd; 20])));
}

// verifies: LLR-ecaw34
#[test]
fn an_empty_string_is_refused() {
    let err = parse_org_id("").expect_err("an empty identifier is not an identifier");
    assert!(err.contains("40 hex chars"), "{err}");
    assert!(err.contains("got 0"), "{err}");
}

// verifies: LLR-ecaw34
#[test]
fn thirty_nine_characters_are_refused() {
    let s = "a".repeat(39);
    let err = parse_org_id(&s).expect_err("one character short is short");
    assert!(err.contains("got 39"), "{err}");
}

// verifies: LLR-ecaw34
#[test]
fn forty_one_characters_are_refused() {
    let s = "a".repeat(41);
    let err = parse_org_id(&s).expect_err("one character long is long");
    assert!(err.contains("got 41"), "{err}");
}

// verifies: LLR-ecaw34
#[test]
fn an_odd_length_is_refused_on_width_not_on_hex_decoding() {
    // 7 characters: odd, and far from 40. The width check must reject it before
    // hex::decode is reached, so the message is the width message.
    let err = parse_org_id("abcdef0").expect_err("7 characters is not 40");
    assert!(err.contains("40 hex chars"), "{err}");
    assert!(!err.contains("org_id hex"), "width is checked first: {err}");
}

// verifies: LLR-ecaw34
#[test]
fn a_non_hex_character_at_the_first_position_is_refused() {
    let s = format!("z{}", "a".repeat(39));
    assert_eq!(s.len(), 40);
    let err = parse_org_id(&s).expect_err("z is not a hex digit");
    assert!(err.contains("org_id hex"), "{err}");
}

// verifies: LLR-ecaw34
#[test]
fn a_non_hex_character_at_the_last_position_is_refused() {
    let s = format!("{}z", "a".repeat(39));
    assert_eq!(s.len(), 40);
    let err = parse_org_id(&s).expect_err("the last character is checked too");
    assert!(err.contains("org_id hex"), "{err}");
}

// verifies: LLR-ecaw34
#[test]
fn forty_non_hex_characters_are_refused() {
    // The right width and the wrong alphabet: the width check passes and the
    // decode must refuse, rather than the wrong 20 bytes being produced.
    let s = "zz".repeat(20);
    let err = parse_org_id(&s).expect_err("the whole string is outside the alphabet");
    assert!(err.contains("org_id hex"), "{err}");
}

// verifies: LLR-ecaw34
#[test]
fn a_doubled_zero_x_prefix_is_refused() {
    // REQ-sjkp8z permits ONE optional `0x` prefix — its rationale says "the
    // `0x` prefix", singular. A REPEATED strip (`trim_start_matches`) eats
    // both and hands a 40-character body to the width check, silently
    // NORMALISING malformed input at a boundary parser. One strip leaves
    // `0x` + 40 characters = 42, which the width check refuses, and that
    // refusal is the correct answer.
    let s = format!("0x0x{}", "ef".repeat(20));
    assert_eq!(s.len(), 44);
    let err = parse_org_id(&s).expect_err("exactly one `0x` prefix is permitted, not two");
    assert!(err.contains("40 hex chars"), "{err}");
    assert!(
        err.contains("got 42"),
        "one prefix is stripped and the second `0x` counts toward the width: {err}"
    );
}

// verifies: LLR-ecaw34
#[test]
fn a_zero_x_in_the_middle_is_refused() {
    // The prefix is a PREFIX. A `0x` anywhere else is two characters of the
    // identifier itself, and `x` is outside the hex alphabet, so the right
    // width with an embedded `0x` must be refused by the decode rather than
    // stripped out of the middle.
    let s = format!("{}0x{}", "ab".repeat(9), "cd".repeat(10));
    assert_eq!(s.len(), 40);
    let err = parse_org_id(&s).expect_err("`0x` is not a hex digit pair");
    assert!(err.contains("org_id hex"), "{err}");
}

// verifies: LLR-ecaw34
#[test]
fn the_bare_prefix_alone_is_refused() {
    // A prefix with no identifier behind it is not an identifier. It is
    // refused on WIDTH — `got 0` — rather than decoding to the empty byte
    // string and being copied into a 20-byte array.
    let err = parse_org_id("0x").expect_err("a prefix alone names no organisation");
    assert!(err.contains("40 hex chars"), "{err}");
    assert!(err.contains("got 0"), "{err}");
}

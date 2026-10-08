#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The development seed and co-signer parse (SDD-789u6d): total, single
//! `0x` strip, errors that name the variable and the rule and carry no part
//! of the value.

use org_io::custody::{parse_co_signer, parse_seed, ConfigError, ConfigRule, ConfigVariable};

const SEED_HEX: &str = "e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a";

/// True when `rendered` contains any four-character window of `input`.
fn leaks(rendered: &str, input: &str) -> bool {
    let body: Vec<char> = input.strip_prefix("0x").unwrap_or(input).chars().collect();
    body.windows(4).any(|window| rendered.contains(&window.iter().collect::<String>()))
}

fn assert_refused(result: Result<impl std::fmt::Debug, ConfigError>, input: &str, variable: ConfigVariable) -> ConfigRule {
    let error = result.unwrap_err();
    assert_eq!(error.variable, variable);
    assert!(!leaks(&error.to_string(), input), "Display leaks the input: {error}");
    assert!(!leaks(&format!("{error:?}"), input), "Debug leaks the input: {error:?}");
    error.rule
}

// verifies: LLR-4ax2m6
#[test]
fn a_seed_of_64_hex_characters_parses_with_or_without_one_prefix_and_in_either_case() {
    let plain = parse_seed(SEED_HEX).unwrap();
    let prefixed = parse_seed(&format!("0x{SEED_HEX}")).unwrap();
    let upper = parse_seed(&SEED_HEX.to_uppercase()).unwrap();
    assert!(plain.same_bytes_as(&prefixed));
    assert!(plain.same_bytes_as(&upper));
}

// verifies: LLR-4ax2m6, LLR-gc6kwy
#[test]
fn a_malformed_seed_is_refused_by_rule_and_the_error_carries_none_of_it() {
    let doubled = format!("0x0x{SEED_HEX}");
    let short = &SEED_HEX[..63];
    let long = format!("{SEED_HEX}a");
    let not_hex = format!("{}g", &SEED_HEX[..63]);
    let non_ascii = format!("{}é", &SEED_HEX[..62]);
    assert_eq!(assert_refused(parse_seed(""), "", ConfigVariable::AdminSeed), ConfigRule::Empty);
    assert_eq!(assert_refused(parse_seed("0x"), "0x", ConfigVariable::AdminSeed), ConfigRule::Empty);
    assert_eq!(assert_refused(parse_seed(short), short, ConfigVariable::AdminSeed), ConfigRule::WrongLength);
    assert_eq!(assert_refused(parse_seed(&long), &long, ConfigVariable::AdminSeed), ConfigRule::WrongLength);
    assert_eq!(assert_refused(parse_seed(&doubled), &doubled, ConfigVariable::AdminSeed), ConfigRule::WrongLength);
    assert_eq!(assert_refused(parse_seed(&not_hex), &not_hex, ConfigVariable::AdminSeed), ConfigRule::NotHex);
    assert_eq!(assert_refused(parse_seed(&non_ascii), &non_ascii, ConfigVariable::AdminSeed), ConfigRule::NotHex);
}

// verifies: LLR-9fy622
#[test]
fn a_co_signer_of_64_hex_characters_parses_to_its_account() {
    let account = parse_co_signer(&format!("0x{SEED_HEX}")).unwrap();
    assert_eq!(hex_of(&account.0), SEED_HEX);
}

// verifies: LLR-9fy622, LLR-gc6kwy
#[test]
fn a_malformed_co_signer_is_refused_by_rule_and_the_error_carries_none_of_it() {
    let doubled = format!("0x0x{SEED_HEX}");
    let odd = format!("{SEED_HEX}0");
    let not_hex = format!("{}z", &SEED_HEX[..63]);
    assert_eq!(assert_refused(parse_co_signer(""), "", ConfigVariable::CoSigner), ConfigRule::Empty);
    assert_eq!(assert_refused(parse_co_signer(&doubled), &doubled, ConfigVariable::CoSigner), ConfigRule::WrongLength);
    assert_eq!(assert_refused(parse_co_signer(&odd), &odd, ConfigVariable::CoSigner), ConfigRule::WrongLength);
    assert_eq!(assert_refused(parse_co_signer(&not_hex), &not_hex, ConfigVariable::CoSigner), ConfigRule::NotHex);
}

// verifies: LLR-gc6kwy
#[test]
fn each_refusal_names_its_variable_and_rule() {
    let seed_error = parse_seed("abc").unwrap_err().to_string();
    assert!(seed_error.starts_with("ODS_ADMIN_SEED"), "{seed_error}");
    assert!(seed_error.contains("64 hexadecimal characters"), "{seed_error}");
    let co_signer_error = parse_co_signer("abc").unwrap_err().to_string();
    assert!(co_signer_error.starts_with("ODS_COSIGNER_PUB"), "{co_signer_error}");
}

fn hex_of(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

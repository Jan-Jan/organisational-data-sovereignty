//! Crate-level discipline of the person unit.

use person::IdentityError;

/// The lines of the `[name]` table of a TOML manifest: from its header to the
/// next table header, trimmed, comments and blank lines dropped.
fn toml_table<'a>(manifest: &'a str, name: &str) -> Vec<&'a str> {
    let header = format!("[{name}]");
    manifest
        .lines()
        .map(str::trim)
        .skip_while(|line| *line != header)
        .skip(1)
        .take_while(|line| !line.starts_with('['))
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

/// verifies: LLR-64muuw, REQ-vxx8k3
#[test]
fn crate_is_no_std_and_denies_panics_and_indexing() {
    let lib = include_str!("../src/lib.rs");
    let lines: Vec<&str> = lib.lines().map(str::trim).collect();
    assert!(lines.contains(&"#![no_std]"), "person must be no_std");
    assert!(
        lines.contains(&"#![deny(clippy::indexing_slicing)]"),
        "person must deny indexing and slicing"
    );
    let lints = toml_table(include_str!("../Cargo.toml"), "lints");
    assert_eq!(
        lints,
        ["workspace = true"],
        "person must inherit the workspace lints"
    );
    let workspace_clippy = toml_table(include_str!("../../Cargo.toml"), "workspace.lints.clippy");
    for lint in ["unwrap_used", "expect_used", "panic"] {
        let denied = format!("{lint} = \"deny\"");
        assert!(
            workspace_clippy.contains(&denied.as_str()),
            "the workspace lints must deny clippy::{lint}"
        );
    }
}

/// A fixture check of the helper above: a `workspace = true` outside
/// `[lints]` is not read as part of it.
#[test]
fn the_lints_table_is_read_by_its_header() {
    let manifest = "[package]\nworkspace = true\n[lints]\n# none\n[dependencies]\nx = 1\n";
    assert!(toml_table(manifest, "lints").is_empty());
    assert_eq!(toml_table(manifest, "package"), ["workspace = true"]);
}

/// A fixture check of the enum, not a verification: the variants differ from
/// one another and in their messages. It cannot see which variant a
/// constructor returns; the tests of each constructor check that.
#[test]
fn every_rejection_is_a_distinct_named_variant() {
    let variants = [
        IdentityError::FieldTooLong {
            field: "name",
            max: 128,
        },
        IdentityError::InvalidDeviceKey,
        IdentityError::InvalidPersonKey,
        IdentityError::DeviceSlotsFull,
        IdentityError::DuplicateDevice,
        IdentityError::DeviceNotFound,
    ];
    for (index, variant) in variants.iter().enumerate() {
        for other in &variants[index + 1..] {
            assert_ne!(variant, other);
            assert_ne!(variant.to_string(), other.to_string());
        }
    }
}

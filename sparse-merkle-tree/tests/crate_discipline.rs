//! The crate's discipline: no_std, no I/O, the workspace lints, and its
//! dependencies.

#![allow(clippy::expect_used)]

use std::fs;
use std::path::Path;

const MANIFEST: &str = include_str!("../Cargo.toml");
const LIB: &str = include_str!("../src/lib.rs");

/// The non-comment, non-empty lines of `[name]` in a TOML manifest.
fn table<'a>(manifest: &'a str, name: &str) -> Vec<&'a str> {
    let header = format!("[{name}]");
    manifest
        .lines()
        .skip_while(|line| line.trim() != header)
        .skip(1)
        .take_while(|line| !line.trim_start().starts_with('['))
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

/// verifies: LLR-byz55h
#[test]
fn the_crate_is_no_std_and_denies_indexing() {
    let attributes: Vec<&str> = LIB.lines().map(str::trim).collect();
    assert!(attributes.contains(&"#![no_std]"));
    assert!(attributes.contains(&"#![deny(clippy::indexing_slicing)]"));
    assert_eq!(table(MANIFEST, "lints"), ["workspace = true"]);
    let workspace = include_str!("../../Cargo.toml");
    let lints = table(workspace, "workspace.lints.clippy");
    for lint in ["unwrap_used", "expect_used", "panic"] {
        assert!(
            lints.iter().any(|line| line.starts_with(lint) && line.contains("\"deny\"")),
            "the workspace lints must deny clippy::{lint}"
        );
    }
}

/// verifies: LLR-byz55h
#[test]
fn the_source_names_no_std_item_and_no_io() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for entry in fs::read_dir(source).expect("src is readable") {
        let path = entry.expect("entry").path();
        let text = fs::read_to_string(&path).expect("source is UTF-8");
        // Without `std` there is no file, socket or console to reach.
        for forbidden in ["std::", "extern crate std"] {
            let hits = text
                .lines()
                .filter(|line| !line.trim_start().starts_with("//"))
                .any(|line| line.contains(forbidden));
            assert!(!hits, "{} names {forbidden}", path.display());
        }
    }
}

/// verifies: LLR-c4eunm
#[test]
fn the_only_dependencies_are_spin_and_optional_serde() {
    let names: Vec<&str> = table(MANIFEST, "dependencies")
        .iter()
        .filter_map(|line| line.split('=').next())
        .map(str::trim)
        .collect();
    assert_eq!(names, ["spin", "serde"]);
    let serde = table(MANIFEST, "dependencies")
        .into_iter()
        .find(|line| line.starts_with("serde"))
        .expect("serde is listed");
    assert!(serde.contains("optional = true"), "{serde}");
    assert_eq!(table(MANIFEST, "features"), ["default = [\"serde\"]", "serde = [\"dep:serde\"]"]);
}

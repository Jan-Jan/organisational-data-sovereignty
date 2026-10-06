#![cfg(all(feature = "test-support", feature = "write"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! LLR-rxs5ec: the writer compiles only with `write`, which adds exactly
//! `subxt-signer` and `blake2`, and this crate names no consumer.

const MANIFEST: &str = include_str!("../Cargo.toml");
const LIB: &str = include_str!("../src/lib.rs");

/// The text of one `[table]` of the manifest, up to the next table header.
fn table(header: &str) -> &'static str {
    let start = MANIFEST.find(header).unwrap_or_else(|| panic!("no {header}")) + header.len();
    let rest = &MANIFEST[start..];
    &rest[..rest.find("\n[").unwrap_or(rest.len())]
}

/// Manifest lines that are not comments.
fn code_lines() -> impl Iterator<Item = &'static str> {
    MANIFEST.lines().filter(|l| !l.trim_start().starts_with('#'))
}

// Normal: the feature exists, names exactly the two crates, and gates the module.
// verifies: LLR-rxs5ec, REQ-6jefu2, REQ-aat4yt
#[test]
fn the_write_feature_adds_exactly_the_signer_and_the_hasher() {
    let features = table("[features]");
    let write = features
        .lines()
        .find(|l| l.trim_start().starts_with("write ="))
        .expect("a `write` feature");
    assert_eq!(write.trim(), r#"write = ["client", "dep:subxt-signer", "dep:blake2"]"#);
    let deps = table("[dependencies]");
    for dep in ["subxt-signer", "blake2"] {
        let line = deps
            .lines()
            .find(|l| l.trim_start().starts_with(&format!("{dep} =")))
            .unwrap_or_else(|| panic!("{dep} must be a normal dependency"));
        assert!(line.contains("optional = true"), "{dep} must be optional: {line}");
    }
    assert!(
        LIB.contains("#[cfg(feature = \"write\")]\npub mod write;"),
        "the write module must be compiled only under `write`"
    );
    assert!(!table("[features]").lines().any(|l| l.trim_start().starts_with("default") && l.contains("write")));
}

// Abnormal: no other feature pulls the two crates in, and no line of the
// manifest names a consumer of this crate.
// verifies: LLR-rxs5ec
#[test]
fn no_other_feature_enables_the_writer_and_no_consumer_is_named() {
    for line in table("[features]").lines().filter(|l| !l.trim_start().starts_with("write =")) {
        assert!(
            !line.contains("subxt-signer") && !line.contains("blake2") && !line.contains("\"write\""),
            "only `write` may enable the writer's crates: {line}"
        );
    }
    for line in code_lines() {
        assert!(!line.contains("org-node") && !line.contains("org-members"), "consumer named: {line}");
    }
}

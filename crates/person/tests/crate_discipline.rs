//! Crate-level discipline of the person unit.

#![allow(clippy::expect_used)]

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
    let workspace_clippy = toml_table(include_str!("../../../Cargo.toml"), "workspace.lints.clippy");
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
        IdentityError::KeyWithoutDevice,
        IdentityError::MissingPersonKey,
        IdentityError::PersonKeyIsDeviceKey,
        IdentityError::PersonKeyNotRotated,
        IdentityError::UnsupportedEncodingVersion(2),
    ];
    for (index, variant) in variants.iter().enumerate() {
        for other in &variants[index + 1..] {
            assert_ne!(variant, other);
            assert_ne!(variant.to_string(), other.to_string());
        }
    }
}

/// The variants the Person operations add, one per rule, each with a message
/// that names its rule.
/// verifies: LLR-3n3kxx
#[test]
fn the_person_variants_name_their_rule() {
    let messages = [
        (
            IdentityError::KeyWithoutDevice,
            "person public key held with no device public key",
        ),
        (
            IdentityError::MissingPersonKey,
            "person public key missing: one or more device public keys are held",
        ),
        (
            IdentityError::PersonKeyIsDeviceKey,
            "person public key equals a device public key",
        ),
        (
            IdentityError::PersonKeyNotRotated,
            "person public key not rotated: the device public keys changed and it did not",
        ),
        (
            IdentityError::UnsupportedEncodingVersion(7),
            "unsupported encoding version 7",
        ),
    ];
    for (variant, message) in messages {
        assert_eq!(variant.to_string(), message);
    }
    assert_ne!(
        IdentityError::UnsupportedEncodingVersion(2),
        IdentityError::UnsupportedEncodingVersion(3)
    );
}

#[path = "support/public_signatures.rs"]
mod public_signatures;

use public_signatures::{crate_public_signatures, public_signatures};

/// A source with every shape the scan must read: a signature over several
/// lines, a public enum's variants, a public struct's public fields and a
/// trait impl's methods; and every shape it must not: a crate-private item,
/// a private field, a private method, a comment, an attribute and a body.
const SHAPES: &str = r#"pub fn f(
    a: u8,
) -> Result<(), TreeError> {
    let inner: TreeError = todo();
}
pub(crate) type T = DeviceTree;
pub use x::y;
/// A doc line naming TreeError.
pub enum E {
    #[error("{0}")]
    Tree(TreeError),
    Held { cause: TreeError },
}
pub struct S {
    pub tree: SparseMerkleTree,
    private: DeviceTree,
}
pub struct W(pub DeviceTree);
impl From<TreeError> for E {
    fn from(error: TreeError) -> Self {
        let inner = 1;
    }
}
impl S {
    fn helper() -> TreeError {}
    pub fn get(&self) -> Result<(), TreeError> {
    }
}
"#;

/// verifies: LLR-3dvfjb
///
/// No public signature of the crate's own (function, method, trait-impl
/// method, type, re-export, enum variant or public field) names the tree's
/// error or its tree type, so no `TreeError` crosses this unit's own
/// functions. The tree's proof types, shared by design, are not checked.
/// The scan is first run over `SHAPES`, so a scan blind to one of those
/// shapes fails here instead of passing the source unread.
#[test]
fn no_public_signature_names_the_tree_or_its_error() {
    assert_eq!(
        public_signatures(SHAPES),
        [
            "pub fn f( a: u8, ) -> Result<(), TreeError> { ",
            "pub use x::y; ",
            "pub enum E { ",
            "Tree(TreeError), ",
            "Held { cause: TreeError }, ",
            "pub struct S { ",
            "pub tree: SparseMerkleTree, ",
            "pub struct W(pub DeviceTree); ",
            "fn from(error: TreeError) -> Self { ",
            "pub fn get(&self) -> Result<(), TreeError> { ",
        ]
    );
    let source_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for (path, signature) in crate_public_signatures(&source_dir) {
        for name in ["TreeError", "SparseMerkleTree", "DeviceTree"] {
            assert!(!signature.contains(name), "{} exposes {name}: {signature}", path.display());
        }
    }
}

//! How a `sparse_merkle_tree::TreeError` becomes an `OrgMembersError`
//! (LLR-w5yahk): never as the tree's own type.

#![allow(clippy::expect_used)]

use org_members::OrgMembersError;
use sparse_merkle_tree::TreeError;

// One copy of the scan that both the person and the org-members test crates
// can reach: no shared test crate exists.
#[path = "../../person/tests/support/public_signatures.rs"]
mod public_signatures;

use public_signatures::{crate_public_signatures, public_signatures};

/// verifies: LLR-w5yahk
#[test]
fn every_tree_error_maps_to_an_org_members_error() {
    let expected = [
        (TreeError::HashesNotCalculated, OrgMembersError::HashesNotCalculated),
        (TreeError::HashesAlreadyCalculated, OrgMembersError::HashesAlreadyCalculated),
        (TreeError::Collision, OrgMembersError::InvariantViolated),
        (TreeError::LeafNotHeld, OrgMembersError::InvariantViolated),
        (TreeError::LeafHeld, OrgMembersError::InvariantViolated),
        (TreeError::LevelOutOfRange, OrgMembersError::InvariantViolated),
        (TreeError::ProofMalformed, OrgMembersError::AbsenceProofMalformed),
        (TreeError::ProofRootMismatch, OrgMembersError::AbsenceProofRootMismatch),
        (TreeError::ProofLeafHeld, OrgMembersError::DeviceStillHeld),
    ];
    for (tree_error, org_members_error) in expected {
        assert_eq!(OrgMembersError::from(tree_error), org_members_error, "{tree_error:?}");
    }
}

/// Whether a public signature names `TreeError` anywhere but a function's
/// parameters: in what a function returns, or in a type, re-export, enum
/// variant or public field.
fn returns_or_holds_tree_error(signature: &str) -> bool {
    if !signature.contains("TreeError") {
        return false;
    }
    if !signature.contains("fn ") {
        return true;
    }
    signature.split_once("->").is_some_and(|(_, returned)| returned.contains("TreeError"))
}

/// verifies: LLR-w5yahk
///
/// No public signature of the crate's own (function, method, trait-impl
/// method, type, re-export, enum variant or public field) returns or holds
/// the tree's error; the mapping into `OrgMembersError` takes it as its
/// parameter. The tree's proof types, shared by design, are not checked.
/// The scan and the predicate are first run over a source with each shape,
/// so a scan blind to one fails here instead of passing the source unread.
#[test]
fn no_public_signature_returns_or_holds_the_tree_error() {
    let shapes = r#"impl From<TreeError> for OrgMembersError {
    fn from(error: TreeError) -> Self {
    }
}
pub enum E {
    Tree(TreeError),
}
pub struct S {
    pub cause: TreeError,
}
pub fn f() -> Result<(), TreeError> {
}
"#;
    let flagged: Vec<String> =
        public_signatures(shapes).into_iter().filter(|signature| returns_or_holds_tree_error(signature)).collect();
    assert_eq!(flagged, ["Tree(TreeError), ", "pub cause: TreeError, ", "pub fn f() -> Result<(), TreeError> { "]);
    let source_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for (path, signature) in crate_public_signatures(&source_dir) {
        assert!(
            !returns_or_holds_tree_error(&signature),
            "{} exposes TreeError: {signature}",
            path.display()
        );
    }
}

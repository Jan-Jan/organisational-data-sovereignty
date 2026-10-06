#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Absences the chain-authority change requires: things org-node must not
//! contain. The compiler enforces most of them for callers; these tests make
//! each one observable in the gate by reading the source and the manifest. A
//! removed name must not appear in `src/` at all — reword any comment that
//! would name it.

use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Every `.rs` file under `org-node/src`, with its text.
pub fn sources() -> Vec<(PathBuf, String)> {
    let mut files = vec![];
    rust_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut files);
    files
        .into_iter()
        .map(|p| {
            let s = std::fs::read_to_string(&p).unwrap();
            (p, s)
        })
        .collect()
}

pub fn assert_absent(needle: &str, why: &str) {
    for (path, text) in sources() {
        assert!(!text.contains(needle), "{} contains `{needle}`: {why}", path.display());
    }
}

/// `assert_absent`, over the files whose path ends with `suffix`.
pub fn assert_absent_in(suffix: &str, needle: &str, why: &str) {
    for (path, text) in sources().into_iter().filter(|(p, _)| p.ends_with(suffix)) {
        assert!(!text.contains(needle), "{} contains `{needle}`: {why}", path.display());
    }
}

/// The `[dependencies]` table of org-node's manifest.
pub fn normal_dependencies() -> String {
    let manifest = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    let start = manifest.find("[dependencies]").unwrap() + "[dependencies]".len();
    let rest = &manifest[start..];
    rest[..rest.find("\n[").unwrap_or(rest.len())].to_string()
}

// org-node defines, exports and imports no Invite or Join request, decodes no
// armoured text, and returns no dialling address (absences: no input side).
// verifies: LLR-g9vmbx, LLR-zj88e6, LLR-qezw3n, LLR-836z24, LLR-8qxwst, REQ-qn2erx
#[test]
fn org_node_holds_no_invitation_exchange() {
    for (needle, why) in [
        ("pub mod blobs", "no blob module"),
        ("fn export_invite", "LLR-zj88e6"),
        ("fn import_invite", "LLR-g9vmbx"),
        ("fn export_join_request", "LLR-437fvx"),
        ("fn import_join_request", "LLR-836z24"),
        ("struct Invite {", "LLR-g9vmbx"),
        ("struct JoinRequest {", "LLR-836z24"),
        ("PendingInvite", "no pending invites"),
        ("pending_invites", "no pending invites"),
        // a call, not the definition, which stays in transport/endpoint.rs
        (".node_addr_for_dial(", "LLR-qezw3n: no value carries a dialling address"),
        ("base64", "LLR-8qxwst: no armoured text"),
    ] {
        assert_absent(needle, why);
    }
    assert!(!normal_dependencies().lines().any(|l| l.trim_start().starts_with("base64")), "no base64 dependency");
}

// org-node has no administrator: no record carries one and no Persona is
// looked up by one (absences: no input side).
// verifies: LLR-xq9nrq, LLR-rys5nx, LLR-g76zqd
#[test]
fn org_node_has_no_administrator_key() {
    assert_absent("admin_member_key", "OrgRecord has no administrator key");
    assert_absent("admin_device_key", "no administrator device key");
    assert_absent("fn admin_persona_for_org", "no Persona is looked up by an administrator key");
}

// The service's chain seam writes nothing; the proxy account is never handed
// to it (absences, no input side).
// verifies: LLR-65py3d, LLR-3v5nu9, REQ-xs4ab8
#[test]
fn the_service_presents_no_chain_write() {
    assert_absent_in("service.rs", "async fn submit_", "ChainOps presents no write, so nothing can take the proxy account to the chain");
}

// org-node builds no calldata, derives no multisig account, holds no
// signatory key and dispatches nothing (absences: no input side). The
// account is opaque: nothing converts it to subxt's type.
// verifies: LLR-rv4vux, LLR-rc74nq, LLR-txqmz4, LLR-66h529, LLR-463d89, LLR-8m3bwj, LLR-f74xwb, LLR-s7whrn, REQ-xs4ab8
#[test]
fn org_node_writes_nothing_to_the_chain() {
    for (needle, why) in [
        ("pub mod chain_write", "no chain-write module"),
        ("pub mod ceremony", "no genesis ceremony"),
        ("fn build_update_calldata", "LLR-rv4vux"),
        ("fn update_calldata", "LLR-txqmz4"),
        ("fn revive_update_runtime_call", "LLR-rc74nq"),
        ("UPDATE_SELECTOR", "LLR-66h529"),
        ("fn multi_account_id", "LLR-463d89"),
        ("fn build_dispatch_tx", "LLR-f74xwb"),
        ("fn dispatch_org_call", "LLR-f74xwb"),
        ("sr25519", "LLR-8m3bwj: no signatory key"),
        ("subxt_signer", "LLR-8m3bwj"),
        ("FinalitySink", "no block sink"),
        // a use of subxt's type, not the word: types.rs's doc names the
        // chain's account type to say what `ChainAccount` holds
        ("utils::AccountId32", "LLR-s7whrn: the account is never converted"),
        ("AccountId32::", "LLR-s7whrn: the account is never converted"),
        ("async fn submit_", "no submission"),
    ] {
        assert_absent(needle, why);
    }
    let deps = normal_dependencies();
    for dep in ["subxt-signer", "blake2", "parity-scale-codec"] {
        assert!(!deps.lines().any(|l| l.trim_start().starts_with(dep)), "{dep} is not a dependency");
    }
}

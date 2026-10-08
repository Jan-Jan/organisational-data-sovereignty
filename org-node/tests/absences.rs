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

// The Organisation secret is gone: org-node defines, holds, takes and returns
// none, and the Organisation private key is the only Organisation key
// material it holds (LLR-qsjde3).
// verifies: LLR-qsjde3
#[test]
fn org_node_holds_no_organisation_secret() {
    assert_absent("OrgSecret", "LLR-qsjde3: no Organisation secret type");
    assert_absent("org_secret", "LLR-qsjde3: no record, message or operation holds one");
}

// The invite identifier never travels between peers: org-node defines no type
// for it, no Wire message or expectation holds one, and `send_update` takes
// none (REQ-8amu2a as amended).
// verifies: LLR-ms8njy, LLR-48jakr
#[test]
fn org_node_holds_no_invite_identifier() {
    assert_absent("InviteId", "LLR-ms8njy: no invite identifier type");
    assert_absent("invite_id", "LLR-ms8njy, LLR-48jakr: no field or argument carries one");
}

// The service writes nothing to the chain; the proxy account is never handed
// to a chain write (absences, no input side).
// verifies: LLR-65py3d, LLR-3v5nu9, REQ-xs4ab8
#[test]
fn the_service_presents_no_chain_write() {
    assert_absent_in("service.rs", "async fn submit_", "the service has no submission, so nothing can take the proxy account to the chain");
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

// org-node does no chain IO (ruling B, change worktree-org-io-create): no
// chain library, no chain seam, no chain read, no `chain` feature.
// verifies: LLR-mn5c2q, LLR-65py3d
#[test]
fn org_node_names_no_chain_library_and_reads_no_chain() {
    for (needle, why) in [
        // Paths, not the bare words: `types.rs`'s doc comment names subxt's
        // account type in prose, as `org_node_writes_nothing_to_the_chain` notes.
        ("on_chain_client::", "the chain is org-io's"),
        ("subxt::", "the chain is org-io's"),
        ("ChainOps", "org-node takes the chain state as a value"),
        ("ChainReader", "verification takes the chain state as a value"),
        ("read_state", "org-node reads no chain"),
        ("OnChainReader", "deleted with its cache (PR-k2xxaq)"),
        ("OrgStateCache", "deleted (PR-k2xxaq)"),
        ("connect_chain_client", "the connection is org-io's"),
    ] {
        assert_absent(needle, why);
    }
    let dependencies = normal_dependencies();
    for crate_name in ["subxt", "on-chain-client", "async-trait", "hex"] {
        assert!(!dependencies.contains(&format!("{crate_name} =")), "org-node depends on {crate_name}");
    }
    assert!(
        !std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap().contains("chain = ["),
        "org-node has no `chain` feature"
    );
}

// --- S3 (commit workflow): scans of the code, not of its comments ----------
//
// The scans below read code only: every `//` comment (doc comments included)
// is cut before matching, so a comment may name what the code must not do.
// Where a pattern can span lines, it is matched against the code with all
// whitespace removed ("squeezed").

/// `text` with every `//` comment cut to the end of its line.
fn code_of(text: &str) -> String {
    text.lines()
        .map(|line| match line.find("//") {
            Some(comment_start) => &line[..comment_start],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `code` with all whitespace removed.
fn squeezed(code: &str) -> String {
    code.chars().filter(|character| !character.is_whitespace()).collect()
}

fn is_identifier_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

/// Byte offsets of `needle` in `haystack` where it starts a whole identifier:
/// the character before it is not part of an identifier.
fn identifier_occurrences(haystack: &str, needle: &str) -> Vec<usize> {
    haystack
        .match_indices(needle)
        .map(|(offset, _)| offset)
        .filter(|&offset| !haystack[..offset].chars().next_back().is_some_and(is_identifier_char))
        .collect()
}

/// The expression that starts at `start` in squeezed code: up to the first
/// `,` or `;` outside brackets, or the bracket that closes around it.
fn expression_from(squeezed_code: &str, start: usize) -> &str {
    let mut depth = 0i32;
    for (offset, character) in squeezed_code[start..].char_indices() {
        match character {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' if depth == 0 => return &squeezed_code[start..start + offset],
            ')' | ']' | '}' => depth -= 1,
            ',' | ';' if depth == 0 => return &squeezed_code[start..start + offset],
            _ => {}
        }
    }
    &squeezed_code[start..]
}

/// `code` without the item `fn <function_name>`: from its `fn` keyword to
/// the brace that closes its body. Unchanged if there is no such function.
fn without_function(code: &str, function_name: &str) -> String {
    let Some(fn_start) = code.find(&format!("fn {function_name}(")) else {
        return code.to_string();
    };
    let body_start = fn_start + code[fn_start..].find('{').unwrap();
    let mut depth = 0i32;
    for (offset, character) in code[body_start..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let body_end = body_start + offset + 1;
                    return format!("{}{}", &code[..fn_start], &code[body_end..]);
                }
            }
            _ => {}
        }
    }
    panic!("fn {function_name} has no closing brace");
}

/// The name of the `fn` whose signature most recently precedes `offset` in
/// `code`: the function a call at `offset` sits in.
fn enclosing_function(code: &str, offset: usize) -> Option<String> {
    code[..offset].lines().rev().find_map(|line| {
        let mut signature = line.trim_start();
        while let Some(rest) = ["pub(crate) ", "pub(super) ", "pub ", "async ", "const ", "unsafe "]
            .iter()
            .find_map(|qualifier| signature.strip_prefix(qualifier))
        {
            signature = rest;
        }
        let name: String = signature.strip_prefix("fn ")?.chars().take_while(|&character| is_identifier_char(character)).collect();
        Some(name)
    })
}

/// revocation.rs and reconcile.rs are values in, values out: no `async`, no
/// chain seam, no transport, endpoint or file system, and no store taken by
/// mutable reference (LLR-xgefn8).
/// verifies: LLR-xgefn8
#[test]
fn revocation_and_reconcile_do_no_io() {
    let forbidden_names = [
        "async ",
        ".await",
        "impl Future",
        "ChainOps",
        "ChainReader",
        "read_state",
        "OrgEndpoint",
        "crate::transport",
        "iroh",
        "tokio",
        "std::fs",
        "std::io",
        "std::net",
        "File::",
        "Store::",
        ".save(",
    ];
    for file in ["revocation.rs", "reconcile.rs"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(file);
        let code = code_of(&std::fs::read_to_string(&path).unwrap());
        for name in forbidden_names {
            assert!(!code.contains(name), "{file} names `{name}` (LLR-xgefn8: no IO)");
        }
        let squeezed_code = squeezed(&code);
        for borrow in ["&mutStoreData", "&mutself"] {
            assert!(!squeezed_code.contains(borrow), "{file} takes `{borrow}` (LLR-xgefn8: the store is taken as &StoreData)");
        }
    }
}

/// No code in org-node removes an `OrgRecord` or a `PersonaRecord` except
/// `StoreData::forget_organisation`: no removing method on `orgs` or
/// `personas`, no rebuild of either through a filter, no reassignment to an
/// empty list and no `mem::take`/`mem::replace` of either (LLR-23sfdh).
/// verifies: LLR-23sfdh
#[test]
fn only_forget_organisation_removes_records_and_personas() {
    let removing_methods =
        ["retain(", "retain_mut(", "remove(", "swap_remove(", "clear()", "drain(", "truncate(", "pop()", "split_off(", "dedup"];
    for (path, text) in sources() {
        let mut code = code_of(&text);
        if path.ends_with("store.rs") {
            code = without_function(&code, "forget_organisation");
        }
        let squeezed_code = squeezed(&code);
        let file = path.display();
        for collection in ["orgs", "personas"] {
            for method in removing_methods {
                let call = format!("{collection}.{method}");
                assert!(identifier_occurrences(&squeezed_code, &call).is_empty(), "{file} calls `{call}` (LLR-23sfdh)");
            }
            for binding in [format!("{collection}:"), format!("{collection}=")] {
                for offset in identifier_occurrences(&squeezed_code, &binding) {
                    let value = expression_from(&squeezed_code, offset + binding.len());
                    if binding.ends_with('=') && value.starts_with('=') {
                        continue; // a comparison, `==`
                    }
                    assert!(
                        !value.contains(".filter(") && !value.contains(".retain"),
                        "{file} rebuilds `{collection}` through a filter: `{binding}{value}` (LLR-23sfdh)"
                    );
                    if binding.ends_with('=') {
                        for empty in ["Vec::new()", "vec![]", "Default::default()"] {
                            assert!(!value.contains(empty), "{file} empties `{collection}`: `{binding}{value}` (LLR-23sfdh)");
                        }
                    }
                }
            }
            for swap in ["mem::take(", "mem::replace("] {
                for offset in squeezed_code.match_indices(swap).map(|(offset, _)| offset) {
                    let argument = expression_from(&squeezed_code, offset + swap.len());
                    assert!(!argument.ends_with(collection), "{file} takes `{collection}` out with `{swap}{argument}` (LLR-23sfdh)");
                }
            }
        }
    }
}

/// `forget_organisation` is defined once, in store.rs (`OrgService` has none
/// of its own), and called from exactly two functions: `revocation::accept`
/// and the commit paths' one removal step, `removal_step` in service.rs
/// (LLR-23sfdh).
/// verifies: LLR-23sfdh
#[test]
fn forget_organisation_is_defined_once_and_called_only_from_accept_and_the_removal_step() {
    let mut definitions = Vec::new();
    let mut callers = Vec::new();
    for (path, text) in sources() {
        let code = code_of(&text);
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        definitions.extend(identifier_occurrences(&code, "fn forget_organisation").iter().map(|_| file_name.clone()));
        for (offset, _) in code.match_indices("forget_organisation") {
            let preceding = code[..offset].trim_end();
            if preceding.ends_with("fn") {
                continue;
            }
            assert!(
                !code[..offset].chars().next_back().is_some_and(is_identifier_char),
                "{file_name} names an identifier ending in forget_organisation"
            );
            callers.push(format!("{file_name}::{}", enclosing_function(&code, offset).unwrap_or_default()));
        }
    }
    assert_eq!(definitions, ["store.rs"], "forget_organisation is defined once, in store.rs (LLR-23sfdh)");
    callers.sort();
    assert_eq!(
        callers,
        ["revocation.rs::accept", "service.rs::removal_step"],
        "forget_organisation is called from accept and the removal step only (LLR-23sfdh)"
    );
}

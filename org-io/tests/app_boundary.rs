#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The app reaches org-node, the chain and the key only through org-io (LLR-n3zmt6).

use std::path::Path;

/// The manifest's dependency tables, each as text.
fn dependency_tables(manifest: &str) -> Vec<(String, String)> {
    let mut tables = vec![];
    let mut current: Option<(String, String)> = None;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            if let Some(table) = current.take() {
                tables.push(table);
            }
            if trimmed.contains("dependencies") {
                current = Some((trimmed.to_string(), String::new()));
            }
        } else if let Some((_, body)) = current.as_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    tables.extend(current);
    tables
}

/// Every `pub use` statement in `text`, from `pub use` to its `;`, comment
/// lines dropped.
fn pub_use_statements(text: &str) -> Vec<String> {
    let code: String = text.lines().filter(|line| !line.trim_start().starts_with("//")).collect::<Vec<_>>().join("\n");
    let mut statements = vec![];
    let mut rest = code.as_str();
    while let Some(start) = rest.find("pub use ") {
        let statement = &rest[start..];
        let end = statement.find(';').map_or(statement.len(), |end| end + 1);
        statements.push(statement[..end].to_string());
        rest = &statement[end..];
    }
    statements
}

/// org-node's types the app must not reach through org-io: the service, the
/// store and its records, the Device secret, and the Wire messages.
const NOT_REEXPORTED: [&str; 12] = [
    "OrgService",
    "PersonaStore",
    "DeviceSecret",
    "DeviceSeed",
    "MemberSeed",
    "SealedStore",
    "StoreData",
    "PersonaRecord",
    "OrgRecord",
    "StoreWrite",
    "wire",
    "WireMessage",
];

// `org_io::node` re-exports an explicit list of the value types the app
// names, so org-node's service, store, secret and Wire types are not
// reachable through org-io.
// verifies: LLR-3zdw8v
#[test]
fn org_io_reexports_no_service_store_secret_or_wire_type_of_org_node() {
    let lib = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs")).unwrap();
    let statements = pub_use_statements(&lib);
    assert!(statements.iter().any(|statement| statement.contains("org_node")), "no re-export of org-node read: {statements:#?}");
    let mut found = vec![];
    for statement in statements.iter().filter(|statement| statement.contains("org_node")) {
        if statement.contains("org_node as") {
            found.push(format!("the whole of org-node: {statement}"));
        }
        for name in NOT_REEXPORTED {
            if statement.split(|character: char| !character.is_alphanumeric() && character != '_').any(|word| word == name) {
                found.push(format!("{name}: {statement}"));
            }
        }
    }
    assert!(found.is_empty(), "org-io re-exports org-node's internals:\n{}", found.join("\n"));
}

// verifies: LLR-n3zmt6
#[test]
fn the_app_depends_on_org_io_and_on_no_other_unit_or_chain_library() {
    let manifest = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../app/src-tauri/Cargo.toml")).unwrap();
    let tables = dependency_tables(&manifest);
    assert!(tables.iter().any(|(name, body)| name == "[dependencies]" && body.lines().any(|line| line.starts_with("org-io "))));
    for (name, body) in &tables {
        for forbidden in ["org-node", "org-members", "on-chain-client", "person", "subxt", "subxt-signer"] {
            assert!(
                !body.lines().any(|line| line.starts_with(&format!("{forbidden} ")) || line.starts_with(&format!("{forbidden}="))),
                "{name} names {forbidden}"
            );
        }
    }
}

/// Private-key and seed types, and the holders that carry one, by name.
/// Public keys and accounts are not key material here: they may be shown
/// (owner clarification, 2026-10-08), so `device_key`, `member_key` and
/// the like are not flagged.
const KEY_TYPES: [&str; 14] = [
    "DeviceSeed",
    "MemberSeed",
    "OrgPrivateKey",
    "SigningKeypair",
    "X25519Keypair",
    "SignatoryKey",
    "SeedBytes",
    "Keypair",
    "SecretKey",
    "SigningKey",
    "PersonaRecord",
    "StoreData",
    "OrgRecord",
    "ProvisionalUpdate",
];

/// Field-name fragments that mean private-key material.
const KEY_FIELD_FRAGMENTS: [&str; 3] = ["seed", "secret", "private"];

/// The text between the `{` that follows `from` and its matching `}`, with
/// comment lines dropped; `None` for a type with no braced body.
fn braced_body(text: &str, from: usize) -> Option<String> {
    let rest = &text[from..];
    let declaration_end = rest.find([';', '{'])?;
    if rest.as_bytes()[declaration_end] == b';' {
        return None;
    }
    let mut depth = 0;
    for (offset, character) in rest[declaration_end..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let body = &rest[declaration_end + 1..declaration_end + offset];
                    return Some(body.lines().filter(|line| !line.trim_start().starts_with("//")).collect::<Vec<_>>().join("\n"));
                }
            }
            _ => {}
        }
    }
    None
}

// The app's IPC-facing types are what its commands return and its events
// carry: every type in app/src-tauri/src that derives `Serialize`, and every
// `#[tauri::command]` signature. None names a private-key or seed type or
// has a field named for one; public keys are allowed.
// verifies: REQ-v4tfap, RC-2xufsr
#[test]
fn no_ipc_facing_type_of_the_app_carries_key_material() {
    let app_src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../app/src-tauri/src");
    let mut ipc_types = vec![];
    let mut carriers = vec![];
    for entry in std::fs::read_dir(&app_src).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        let mut offset = 0;
        for (index, line) in lines.iter().enumerate() {
            let line_start = offset;
            offset += line.len() + 1;
            let trimmed = line.trim_start();
            let declares = ["struct ", "enum ", "pub struct ", "pub enum ", "pub(crate) struct ", "pub(crate) enum "]
                .iter()
                .any(|keyword| trimmed.starts_with(keyword));
            if declares {
                let derives_serialize = lines[..index]
                    .iter()
                    .rev()
                    .take_while(|above| above.trim_start().starts_with("#[") || above.trim_start().starts_with("///"))
                    .any(|above| {
                        above.contains("derive")
                            && above.split([',', '(', ')', ' ']).any(|word| matches!(word, "Serialize" | "serde::Serialize"))
                    });
                if !derives_serialize {
                    continue;
                }
                ipc_types.push(trimmed.to_string());
                let body = braced_body(&text, line_start).unwrap_or_default();
                for key_type in KEY_TYPES {
                    if body.split(|character: char| !character.is_alphanumeric() && character != '_').any(|word| word == key_type) {
                        carriers.push(format!("{}:{}: {trimmed} carries {key_type}", path.display(), index + 1));
                    }
                }
                let lowered = body.to_lowercase();
                for fragment in KEY_FIELD_FRAGMENTS {
                    if lowered.contains(fragment) {
                        carriers.push(format!("{}:{}: {trimmed} has a field named for {fragment}", path.display(), index + 1));
                    }
                }
            }
            if trimmed.starts_with("#[tauri::command]") {
                let signature: String = lines[index + 1..]
                    .iter()
                    .take_while(|line| !line.trim_end().ends_with('{'))
                    .chain(lines[index + 1..].iter().find(|line| line.trim_end().ends_with('{')))
                    .map(|line| line.trim())
                    .collect::<Vec<_>>()
                    .join(" ");
                ipc_types.push(signature.clone());
                for key_type in KEY_TYPES {
                    if signature.split(|character: char| !character.is_alphanumeric() && character != '_').any(|word| word == key_type) {
                        carriers.push(format!("{}:{}: a command names {key_type}: {signature}", path.display(), index + 2));
                    }
                }
            }
        }
    }
    // The DTOs, the event payloads, the status and the commands are all found.
    for expected in ["PersonaDto", "OrgDto", "AdmissionRequestDto", "ConnectionStatus", "MembershipUpdated", "fn list_personas"] {
        assert!(ipc_types.iter().any(|found| found.contains(expected)), "{expected} was not read: {ipc_types:#?}");
    }
    assert!(carriers.is_empty(), "IPC-facing types carrying key material:\n{}", carriers.join("\n"));
}

/// Every `.rs` file under `dir`, its subfolders included.
fn rust_files_under(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = vec![];
    let mut to_read = vec![dir.to_path_buf()];
    while let Some(folder) = to_read.pop() {
        for entry in std::fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                to_read.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }
    files
}

/// Whether `code` (comments removed, its lines joined) names org-node's
/// `service` module: `org_node::service`, or `service` inside a
/// `use org_node::{…}` group at any depth.
fn names_org_node_service(code: &str) -> bool {
    if code.contains("org_node::service") {
        return true;
    }
    code.match_indices("org_node::{").any(|(at, opening)| {
        let group = &code[at + opening.len()..];
        let mut depth = 1;
        let end = group
            .char_indices()
            .find(|(_, character)| {
                match character {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
                depth == 0
            })
            .map_or(group.len(), |(index, _)| index);
        group[..end].split(|character: char| !(character.is_alphanumeric() || character == '_')).any(|word| word == "service")
    })
}

// The app reaches no operation of org-node's that keeps or adopts — no `check_received`, no
// org-node `verify`, no service of its own — so every adoption goes
// through org-io's verification and every receipt through its receive step. Every file under
// `app/src-tauri/src` is read, its subfolders included, and a `use org_node::{…}` group naming
// `service` counts as naming it.
// verifies: LLR-ntag3g
#[test]
fn the_app_calls_no_adopting_operation_and_builds_no_service() {
    let app_src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../app/src-tauri/src");
    let mut calls = vec![];
    let files = rust_files_under(&app_src);
    assert!(files.len() > 3, "the scan read the app's sources ({})", files.len());
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        let code: Vec<&str> = text.lines().map(|line| line.split("//").next().unwrap_or("")).collect();
        if names_org_node_service(&code.join(" ")) {
            calls.push(format!("{}: names org_node::service", path.display()));
        }
        for (index, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            for operation in [".check_received(", "OrgService::new(", "org_node::service", ".messages_for_current_epoch(", ".acknowledgement_accepted("] {
                if code.contains(operation) {
                    calls.push(format!("{}:{}: {operation}", path.display(), index + 1));
                }
            }
            if code.contains(".verify(") && !code.contains("verify_organisation") {
                calls.push(format!("{}:{}: org-node's verify", path.display(), index + 1));
            }
        }
    }
    assert!(calls.is_empty(), "the app reaches org-node around org-io:\n{}", calls.join("\n"));
}

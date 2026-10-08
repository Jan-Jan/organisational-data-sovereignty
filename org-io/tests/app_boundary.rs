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
    for expected in ["PersonaDto", "OrgDto", "InviteReplyDto", "ConnectionStatus", "MembershipUpdated", "fn list_personas"] {
        assert!(ipc_types.iter().any(|found| found.contains(expected)), "{expected} was not read: {ipc_types:#?}");
    }
    assert!(carriers.is_empty(), "IPC-facing types carrying key material:\n{}", carriers.join("\n"));
}

// The app reaches org-node's chain-judging operations only through org-io's
// sequences (review round 2, finding 2): its sources call no commit,
// reconcile or apply phase, and build no `OrgService` of their own, from
// which every one of them would be reachable. Prepare is chain-free and is
// not listed.
// verifies: LLR-7pj5af
#[test]
fn the_app_calls_no_chain_judging_operation_and_builds_no_service() {
    let app_src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../app/src-tauri/src");
    let mut calls = vec![];
    for entry in std::fs::read_dir(&app_src).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        for (index, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            let operations = [
                ".commit_genesis(",
                ".commit_update(",
                ".reconcile(",
                ".apply_receive(",
                ".apply_self_delete(",
                "OrgService::new(",
            ];
            for operation in operations.iter().filter(|operation| line.contains(*operation)) {
                calls.push(format!("{}:{}: {operation}", path.display(), index + 1));
            }
        }
    }
    assert!(calls.is_empty(), "the app reaches org-node around org-io:\n{}", calls.join("\n"));
}

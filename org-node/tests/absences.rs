#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Absences org-node's requirements state: things org-node must not
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
        } else if path.extension().is_some_and(|extension| extension == "rs") {
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
        .map(|path| {
            let text = std::fs::read_to_string(&path).unwrap();
            (path, text)
        })
        .collect()
}

/// The text of `src/<file>`.
pub fn source_of(file: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(file)).unwrap()
}

/// Every method that removes entries from a list, as a scan for a removal
/// matches it: `dedup` with its variants.
const REMOVING_METHODS: [&str; 10] =
    ["retain(", "retain_mut(", "remove(", "swap_remove(", "clear()", "drain(", "truncate(", "pop()", "split_off(", "dedup"];

pub fn assert_absent(needle: &str, why: &str) {
    for (path, text) in sources() {
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

// org-node defines, exports and imports no Join request and no Invite
// reply, and returns no dialling address (absences: no input side). The
// Invite is org-node's: the service builds it (`fn build_invite`,
// LLR-zj88e6), and armoured text reaches org-node only through the Invite
// parse, so `base64` is named only in `invitation.rs` (LLR-8qxwst). What a
// joining Device composes reaches the inviting store only as an admission
// request `check_received` parses (LLR-96z2k5): no Joiner, no Invite reply,
// nothing builds or checks one. org-node builds no address to dial
// (LLR-ds228z).
// verifies: LLR-zj88e6, LLR-ds228z, LLR-96z2k5, LLR-8qxwst
#[test]
fn org_node_holds_no_invitation_exchange() {
    for (needle, why) in [
        ("pub mod blobs", "no blob module"),
        ("fn export_join_request", "LLR-437fvx"),
        ("fn import_join_request", "LLR-96z2k5"),
        ("struct JoinRequest {", "LLR-96z2k5"),
        ("struct Joiner {", "LLR-96z2k5: no Joiner type"),
        ("struct InviteReply {", "LLR-96z2k5: no Invite-reply type"),
        ("fn build_reply", "LLR-96z2k5: no Invite reply is built"),
        ("fn check_reply", "LLR-96z2k5: no Invite reply is checked"),
        ("PendingInvite", "no pending invites"),
        ("pending_invites", "no pending invites"),
        (".node_addr_for_dial(", "LLR-ds228z: org-node builds no address to dial"),
    ] {
        assert_absent(needle, why);
    }
    let service = source_of("service.rs");
    for present in ["fn build_invite", "fn check_received"] {
        assert!(
            service.contains(present),
            "LLR-zj88e6, LLR-96z2k5: the service builds the Invite and parses the admission request ({present})"
        );
    }
    for (path, text) in sources() {
        if !path.ends_with("invitation.rs") {
            assert!(!text.contains("base64"), "{} contains `base64`: LLR-8qxwst, armour only in invitation.rs", path.display());
        }
    }
}

// org-node has no administrator: no record carries one and no Persona is
// looked up by one (absences: no input side).
// verifies: LLR-q4puca, LLR-rys5nx
#[test]
fn org_node_has_no_administrator_key() {
    assert_absent("admin_member_key", "OrgRecord has no administrator key");
    assert_absent("admin_device_key", "no administrator device key");
    assert_absent("fn admin_persona_for_org", "no Persona is looked up by an administrator key");
}

// org-node has no Organisation secret: it defines, holds, takes and returns
// none, and the Organisation private key is the only Organisation key
// material it holds (LLR-qsjde3).
// verifies: LLR-qsjde3
#[test]
fn org_node_holds_no_organisation_secret() {
    assert_absent("OrgSecret", "LLR-qsjde3: no Organisation secret type");
    assert_absent("org_secret", "LLR-qsjde3: no record, message or operation holds one");
}

// The invite identifier travels in one Wire message only: of the six kinds,
// `AdmissionRequest` alone has an `invite_id` field (LLR-g3wmen); no other
// kind, and no expected admission, holds one. Read from `wire.rs`'s enum,
// variant by variant, and from the `ExpectedAdmission` struct body.
// verifies: LLR-g3wmen
#[test]
fn only_the_admission_request_carries_an_invite_identifier() {
    let wire = code_of(&source_of("wire.rs"));
    let start = wire.find("pub enum WireMessage {").expect("the WireMessage enum");
    let body = &wire[start..start + wire[start..].find("\n}").expect("its closing brace")];
    let mut variants: Vec<(String, String)> = vec![];
    for line in body.lines().skip(1) {
        let first = line.trim_start().chars().next();
        if line.starts_with("    ") && !line.starts_with("     ") && first.is_some_and(char::is_uppercase) {
            let name: String = line.trim_start().chars().take_while(|character| character.is_alphanumeric()).collect();
            variants.push((name, String::new()));
        }
        if let Some((_, text)) = variants.last_mut() {
            text.push_str(line);
            text.push('\n');
        }
    }
    let names: Vec<&str> = variants.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(
        names,
        ["OrgInformation", "Revocation", "Acknowledgement", "AdmissionRequest", "OrganisationRequest", "Organisation"],
        "the six kinds, in index order"
    );
    for (name, text) in &variants {
        let carries = text.contains("invite_id") || text.contains("InviteId");
        assert_eq!(carries, name == "AdmissionRequest", "{name}: only the admission request carries an invite identifier");
    }
    let store = source_of("store.rs");
    let start = store.find("pub struct ExpectedAdmission {").expect("the ExpectedAdmission struct");
    let expectation = &store[start..start + store[start..].find('}').expect("its closing brace")];
    assert!(expectation.contains("org_id"), "the scan reads the struct body: {expectation}");
    for needle in ["invite_id", "InviteId"] {
        assert!(!expectation.contains(needle), "LLR-g3wmen: an expectation names the Organisation alone: {expectation}");
    }
    for kind in ["RevocationNotice {", "Acknowledgement {"] {
        let revocation = source_of("revocation.rs");
        let start = revocation.find(&format!("pub struct {kind}")).expect("the struct");
        let fields = &revocation[start..start + revocation[start..].find('}').expect("its closing brace")];
        assert!(!fields.contains("invite_id"), "{kind} carries no invite identifier");
    }
}

// The service writes nothing to the chain, and the proxy account is passed
// to no operation: no function of org-node submits, synchronous or not; no
// public function but `record_founding_proxy` takes a `ChainAccount`; and
// every line of code that names the proxy (`proxy`, `proxy_account`) is
// reached only from the operations LLR-x5asbx names as reading or setting
// it — `propose_founding` (none yet), `record_founding_proxy`,
// `proxy_account`, `persona_view` (the founding's copy, handed back) and
// `verify` (the founding's adoption, and an adoption that keeps it). The
// store's own encoding (struct fields and their conversions, outside any
// operation) is where it reaches the disk sealed.
// verifies: LLR-7en38k, LLR-x5asbx
#[test]
fn the_service_presents_no_chain_write() {
    for (path, text) in sources() {
        let code = code_of(&text);
        assert!(identifier_occurrences(&code, "fn submit_").is_empty(), "{}: a submission (LLR-7en38k)", path.display());
        for start in identifier_occurrences(&code, "pub fn ") {
            let signature = squeezed(&code[start..start + code[start..].find(['{', ';']).expect("a signature's end")]);
            let parameters = signature.split("->").next().expect("the parameters");
            let name: String = signature["pubfn".len()..].chars().take_while(|&character| is_identifier_char(character)).collect();
            assert!(
                name == "record_founding_proxy" || !parameters.contains("ChainAccount"),
                "{}: `{name}` takes a ChainAccount (LLR-x5asbx: the proxy is passed to no operation)",
                path.display()
            );
        }
    }
    let mut reaching: Vec<String> = vec![];
    for (path, text) in sources() {
        let code = code_of(&text);
        let file = path.file_name().expect("a file name").to_string_lossy().to_string();
        for needle in ["proxy_account", "proxy"] {
            for offset in identifier_occurrences(&code, needle) {
                if code[offset + needle.len()..].chars().next().is_some_and(is_identifier_char) {
                    continue; // a longer identifier: `proxy_account` when looking for `proxy`
                }
                let Some(function) = enclosing_function(&code, offset) else { continue }; // a field or a message
                if file == "store.rs" && ["from", "try_from"].contains(&function.as_str()) {
                    continue; // the store's encoding of the field
                }
                if is_public(&function) {
                    reaching.push(format!("{file}::{function}"));
                } else {
                    reaching.extend(public_entry_points_reaching(&function));
                }
            }
        }
    }
    reaching.sort();
    reaching.dedup();
    assert_eq!(
        reaching,
        [
            "service.rs::persona_view",
            "service.rs::propose_founding",
            "service.rs::proxy_account",
            "service.rs::record_founding_proxy",
            "service.rs::verify",
        ],
        "LLR-x5asbx: no other operation reads or changes the proxy account"
    );
}

// org-node builds no calldata, derives no multisig account, holds no
// signatory key and dispatches nothing (absences: no input side). The
// account is opaque: nothing converts it to subxt's type.
// verifies: LLR-rv4vux, LLR-rc74nq, LLR-txqmz4, LLR-66h529, LLR-463d89, LLR-8m3bwj, LLR-f74xwb, LLR-s7whrn, REQ-dhr58q, REQ-4qh2h8
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
        assert!(!deps.lines().any(|line| line.trim_start().starts_with(dep)), "{dep} is not a dependency");
    }
}

// org-node does no chain IO (LLR-7en38k): no chain library, seam or read, no `chain` feature, no operation that
// writes to the chain, and the one operation that judges against the
// chain, `verify`, takes the state as a value and is synchronous; the
// source defines no other judging operation (`commit_*`, `reconcile`).
// verifies: LLR-7en38k
#[test]
fn org_node_names_no_chain_library_and_only_verify_judges_against_the_chain() {
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
        ("fn commit_genesis", "LLR-7en38k: verify alone judges against the chain"),
        ("fn commit_update", "LLR-7en38k: verify alone judges against the chain"),
        ("fn commit_checked", "LLR-7en38k: verify alone judges against the chain"),
        ("fn reconcile", "LLR-7en38k: verify alone judges against the chain"),
        ("async fn submit_", "LLR-7en38k: org-node presents no chain write"),
        ("async fn verify", "LLR-7en38k: verify is synchronous"),
    ] {
        assert_absent(needle, why);
    }
    let service = source_of("service.rs");
    let start = service.find("pub fn verify").expect("OrgService::verify");
    let signature = &service[start..start + service[start..].find('{').expect("verify's body")];
    assert!(signature.contains("Option<OrgState>"), "LLR-7en38k: verify takes the state as a value: {signature}");
    let verify_org = source_of("verify_org.rs");
    assert!(!verify_org.contains("async"), "LLR-7en38k: verify_org.rs is synchronous");
    let dependencies = normal_dependencies();
    for crate_name in ["subxt", "on-chain-client", "async-trait", "hex"] {
        assert!(!dependencies.contains(&format!("{crate_name} =")), "org-node depends on {crate_name}");
    }
    assert!(
        !std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap().contains("chain = ["),
        "org-node has no `chain` feature"
    );
}

// --- org-node is sans-IO -----------------------------------------------------

/// The whole of org-node's manifest.
fn manifest() -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap()
}

/// What in `text` names `std`'s file system or environment in a form the
/// plain `std::fs` needle misses: `fs` or
/// `env` inside a grouped `std::{…}` import, and any environment read by
/// path (`env::var(`, `env::var_os(`, `env::vars(`, `std::env`).
fn file_and_environment_reads(text: &str) -> Vec<String> {
    let mut found = vec![];
    for needle in ["std::env", "env::var(", "env::var_os(", "env::vars(", "env::vars_os("] {
        if text.contains(needle) {
            found.push(needle.to_owned());
        }
    }
    let mut rest = text;
    while let Some(start) = rest.find("std::{") {
        let group = &rest[start + "std::{".len()..];
        let mut depth = 1;
        let end = group
            .char_indices()
            .find(|&(_, character)| {
                depth += match character {
                    '{' => 1,
                    '}' => -1,
                    _ => 0,
                };
                depth == 0
            })
            .map_or(group.len(), |(index, _)| index);
        // Each top-level item of the group: `fs`, `fs::File`, `env::{self, var}` …
        let mut item_depth = 0;
        let mut item_start = 0;
        let mut items = vec![];
        for (index, character) in group[..end].char_indices() {
            match character {
                '{' => item_depth += 1,
                '}' => item_depth -= 1,
                ',' if item_depth == 0 => {
                    items.push(&group[item_start..index]);
                    item_start = index + 1;
                }
                _ => {}
            }
        }
        items.push(&group[item_start..end]);
        for item in items {
            let head = item.trim().split("::").next().unwrap_or_default().trim();
            if head == "fs" || head == "env" {
                found.push(format!("std::{{{}}}", item.trim()));
            }
        }
        rest = &group[end..];
    }
    found
}

/// The self-check `org_node_is_sans_io` runs before its real scan: the
/// helper catches each grouped import and environment read, and passes
/// grouped imports of neither. A helper that stops catching them would let
/// the real scan pass on a crate that does IO, so the annotated test fails
/// instead.
fn assert_the_scan_catches_grouped_file_imports_and_environment_reads() {
    for (text, expected) in [
        ("use std::{fs, io};", vec!["std::{fs}"]),
        ("use std::{io::Read, fs::File};", vec!["std::{fs::File}"]),
        ("use std::{\n    collections::{BTreeMap, HashMap},\n    env::{self, var},\n};", vec!["std::{env::{self, var}}"]),
        ("let x = env::var(\"X\");", vec!["env::var("]),
        ("let x = std::env::var(\"X\");", vec!["std::env", "env::var("]),
        ("let x = env::var_os(\"X\");", vec!["env::var_os("]),
    ] {
        assert_eq!(file_and_environment_reads(text), expected, "{text}");
    }
    for clean in ["use std::{fmt, io};", "use std::{collections::{BTreeMap, HashSet}, fmt::Debug};", "let envelope = 1;"] {
        assert!(file_and_environment_reads(clean).is_empty(), "{clean}");
    }
}

// org-node binds no endpoint, sends and receives nothing, opens no file and
// awaits nothing: it has no transport, no file IO, no async and no keychain.
// The transport and its preflight check are org-io's. A grouped
// `use std::{fs, …}` import and an environment read (`env::var(` and its
// kin) are caught too (`file_and_environment_reads`), and that helper is
// checked on strings first, so a helper that misses grouped imports fails
// this test.
// verifies: LLR-g2jeqd, REQ-x3jrh8
#[test]
fn org_node_is_sans_io() {
    assert_the_scan_catches_grouped_file_imports_and_environment_reads();
    for (needle, why) in [
        ("iroh", "LLR-g2jeqd: the transport is org-io's"),
        ("OrgEndpoint", "LLR-g2jeqd: org-node has no endpoint"),
        ("pub mod transport", "LLR-g2jeqd: no transport module"),
        ("crate::transport", "LLR-g2jeqd: no transport module"),
        ("fn bind_endpoint", "LLR-g2jeqd: nothing binds an endpoint"),
        ("fn send_body", "LLR-g2jeqd: nothing sends"),
        ("fn receive_body", "LLR-g2jeqd: nothing receives"),
        ("std::fs", "REQ-x3jrh8: org-node opens no file"),
        ("PathBuf", "REQ-x3jrh8: org-node names no file path"),
        ("tokio", "org-node runs no runtime"),
        ("async fn", "org-node awaits nothing"),
        (".await", "org-node awaits nothing"),
        ("keyring", "org-node never touches the OS keychain (REQ-26mjqa)"),
        ("ODS_DEVICE_SEED", "no seed from the environment: the preflight left with owner ruling 7"),
        ("DeviceSeed", "the device seed is the move-only DeviceSecret"),
        ("fn check_transport", "the transport check is org-io's (owner ruling 7)"),
    ] {
        assert_absent(needle, why);
    }
    for (path, text) in sources() {
        let found = file_and_environment_reads(&text);
        assert!(found.is_empty(), "{} reads a file or the environment: {found:?} (REQ-x3jrh8)", path.display());
    }
    let dependencies = normal_dependencies();
    for crate_name in ["iroh", "tokio", "async-trait", "argon2", "keyring"] {
        assert!(!dependencies.contains(&format!("{crate_name} =")), "org-node depends on {crate_name}");
    }
    let manifest = manifest();
    for needle in ["iroh", "tokio", "transport = [", "\"transport\"", "name = \"preflight\""] {
        assert!(!manifest.contains(needle), "org-node's manifest names `{needle}` (LLR-g2jeqd: no transport, no runtime)");
    }
}

// No value org-node returns carries a dialling address: org-node holds no
// endpoint address type and no function that builds one for a dial.
// `EndpointAddr` is matched as a whole identifier (the transport's endpoint
// address type), so the opaque address list `EndpointAddresses`
// (LLR-h4gwns) does not match it.
// verifies: LLR-ds228z
#[test]
fn org_node_holds_no_dialling_address() {
    for (needle, why) in [
        ("node_addr_for_dial", "LLR-ds228z: no dialling address is built"),
        ("peer_addr", "LLR-ds228z: no peer address is taken or returned"),
    ] {
        assert_absent(needle, why);
    }
    for (path, text) in sources() {
        let whole = text.match_indices("EndpointAddr").any(|(offset, _)| {
            !text[offset + "EndpointAddr".len()..].chars().next().is_some_and(is_identifier_char)
        });
        assert!(!whole, "{} names `EndpointAddr`: LLR-ds228z: no transport address type", path.display());
    }
}

// --- Scans of the code, not of its comments ----------------------------------
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
    let Some(fn_start) =
        code.find(&format!("fn {function_name}(")).or_else(|| code.find(&format!("fn {function_name}<")))
    else {
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

// --- One deleting path, no IO and no clock -----------------------------------

/// The name of the innermost `fn` whose body holds byte `offset` of `code`
/// (comment-free code: its braces balance), or `None` outside every body.
fn enclosing_function(code: &str, offset: usize) -> Option<String> {
    let mut innermost: Option<(usize, String)> = None;
    for start in identifier_occurrences(code, "fn ") {
        if start >= offset {
            break;
        }
        let name: String = code[start + 3..].trim_start().chars().take_while(|&character| is_identifier_char(character)).collect();
        let Some(open) = code[start..].find(['{', ';']).map(|found| start + found) else { continue };
        if code.as_bytes()[open] == b';' || open > offset {
            continue; // a body-less declaration, or a body that starts after `offset`
        }
        let mut depth = 0i32;
        let mut close = code.len();
        for (index, character) in code[open..].char_indices() {
            match character {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        close = open + index;
                        break;
                    }
                }
                _ => {}
            }
        }
        if offset < close && innermost.as_ref().is_none_or(|(held, _)| open > *held) {
            innermost = Some((open, name));
        }
    }
    innermost.map(|(_, name)| name)
}

/// The files defining `name`, and the callers, as `file::function`, of every
/// call of `name(` (or `name::<`) in `src/` other than its definition.
fn callers_of(name: &str) -> (Vec<String>, Vec<String>) {
    let mut definitions = vec![];
    let mut callers = vec![];
    for (path, text) in sources() {
        let code = code_of(&text);
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        for offset in identifier_occurrences(&code, name) {
            let after = &code[offset + name.len()..];
            if !(after.starts_with('(') || after.starts_with("::<") || after.starts_with('<')) {
                continue;
            }
            if code[..offset].trim_end().ends_with("fn") {
                definitions.push(file.clone());
                continue;
            }
            let caller = enclosing_function(&code, offset)
                .unwrap_or_else(|| panic!("{file}: a call of `{name}` outside every function, which the scan cannot place"));
            callers.push(format!("{file}::{caller}"));
        }
    }
    callers.sort();
    (definitions, callers)
}

/// Whether `src/` defines a `pub fn` named `function` (a `pub(crate)` one is
/// not public).
fn is_public(function: &str) -> bool {
    sources().iter().any(|(_, text)| {
        let code = code_of(text);
        code.contains(&format!("pub fn {function}(")) || code.contains(&format!("pub fn {function}<"))
    })
}

/// The public functions, as `file::function`, from which a call of `name`
/// in `src/` is reached: its callers, and the callers of each non-public
/// one among them, followed up to the first public function on each path.
/// How the private steps between are named or split is not looked at.
fn public_entry_points_reaching(name: &str) -> Vec<String> {
    let mut entries = vec![];
    let mut seen: Vec<String> = vec![];
    let mut frontier = vec![name.to_string()];
    while let Some(current) = frontier.pop() {
        for caller in callers_of(&current).1 {
            if seen.contains(&caller) {
                continue;
            }
            seen.push(caller.clone());
            let function = caller.split("::").nth(1).expect("a caller is `file::function`").to_string();
            if is_public(&function) {
                entries.push(caller);
            } else {
                frontier.push(function);
            }
        }
    }
    entries.sort();
    entries
}

/// `into_removed` is defined once, in removal.rs, and reached from one
/// public function: `verify`. Private callers are followed up to the public
/// ones, which is what LLR-9jn3z8 states ("`verify`'s removal"), so how the
/// private steps between are named or split is not pinned.
// verifies: LLR-9jn3z8
#[test]
fn into_removed_is_called_only_from_verify() {
    let (definitions, _) = callers_of("into_removed");
    assert_eq!(definitions, ["removal.rs"]);
    assert_eq!(public_entry_points_reaching("into_removed"), ["service.rs::verify"]);
}

/// A kept acknowledgement is removed only on behalf of
/// `acknowledgement_accepted`: every removing call on an `acknowledgements`
/// list in `src/` is followed up to the public functions that reach it, and
/// that is the one public function found. No private helper is named or
/// exempted: the scan anchors on the removing calls and the public entry
/// point LLR-9jn3z8 names.
// verifies: LLR-9jn3z8
#[test]
fn a_kept_acknowledgement_is_removed_only_on_acceptance() {
    let mut reaching: Vec<String> = vec![];
    for (path, text) in sources() {
        let code = code_of(&text);
        let squeezed_code = squeezed(&code);
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        for method in REMOVING_METHODS {
            for offset in identifier_occurrences(&code, &format!("acknowledgements.{method}")) {
                let function = enclosing_function(&code, offset)
                    .unwrap_or_else(|| panic!("{file}: an `acknowledgements.{method}` outside every function"));
                if is_public(&function) {
                    reaching.push(format!("{file}::{function}"));
                } else {
                    reaching.extend(public_entry_points_reaching(&function));
                }
            }
            // A call split over lines (`acknowledgements\n.remove(`) is caught too.
            let call = format!("acknowledgements.{method}");
            assert_eq!(
                identifier_occurrences(&squeezed_code, &call).len(),
                identifier_occurrences(&code, &call).len(),
                "{file}: a `{call}` the scan cannot place"
            );
        }
    }
    reaching.sort();
    reaching.dedup();
    assert_eq!(reaching, ["service.rs::acknowledgement_accepted"]);
}

/// Nothing in org-node removes a store except `drop_store` (org-io's
/// local-IO exception): no removing method on `stores`, no rebuild of it
/// through a filter, no reassignment to an empty list and no
/// `mem::take`/`mem::replace` of it. Nothing empties the current record:
/// `record` is never set to `None`, taken, or swapped out. The Persona is
/// never reset. A removed form is built only on behalf of `verify` and
/// rebuilt only by `acknowledgement_accepted`.
// verifies: LLR-9jn3z8
#[test]
fn only_the_removal_and_the_accepted_acknowledgement_delete() {
    let service = code_of(&source_of("service.rs"));
    let drop_store = &service[service.find("fn drop_store(").expect("OrgService::drop_store")..];
    let drop_store = squeezed(&drop_store[..drop_store.find("\n    }\n").expect("the end of drop_store")]);
    // Any removing method, not `retain` alone.
    assert!(
        REMOVING_METHODS.iter().any(|method| drop_store.contains(&format!("stores.{method}"))),
        "drop_store removes a store"
    );
    for (path, text) in sources() {
        let mut code = code_of(&text);
        if path.ends_with("service.rs") {
            code = without_function(&code, "drop_store");
        }
        let squeezed_code = squeezed(&code);
        let file = path.display();
        for method in REMOVING_METHODS {
            assert!(identifier_occurrences(&squeezed_code, &format!("stores.{method}")).is_empty(), "{file} removes a store");
        }
        for binding in ["stores:", "stores="] {
            for offset in identifier_occurrences(&squeezed_code, binding) {
                let expression = expression_from(&squeezed_code, offset + binding.len());
                if binding.ends_with('=') && expression.starts_with('=') {
                    continue; // a comparison, `==`
                }
                assert!(!expression.contains(".filter(") && !expression.contains(".retain"), "{file} rebuilds `stores` through a filter");
                if binding.ends_with('=') {
                    for empty in ["Vec::new()", "vec![]", "Default::default()"] {
                        assert!(!expression.contains(empty), "{file} empties `stores`: `{binding}{expression}`");
                    }
                }
            }
        }
        for swap in ["mem::take(", "mem::replace("] {
            for offset in squeezed_code.match_indices(swap).map(|(offset, _)| offset) {
                let argument = expression_from(&squeezed_code, offset + swap.len());
                assert!(!argument.ends_with("stores"), "{file} takes `stores` out with `{swap}{argument}`");
            }
        }
        for emptying in ["record=None", "record.take()", "mem::take(&mutdata.record)", "mem::replace(&mutdata.record"] {
            assert!(!squeezed_code.contains(emptying), "{file}: `{emptying}` removes the current record (LLR-9jn3z8)");
        }
        for reset in ["persona=", "member_seed="] {
            for offset in identifier_occurrences(&squeezed_code, reset) {
                assert!(squeezed_code[offset + reset.len()..].starts_with('='), "{file} rewrites `{reset}` (LLR-9jn3z8)");
            }
        }
    }
    // A removed form is built by the removal and rebuilt only on an acceptance; `StoreContent::Removed(removed)`
    // is a pattern, not a construction. A private builder is followed up to
    // the public functions that reach it, so its own name is not pinned.
    let mut builders: Vec<String> = identifier_occurrences(&service, "StoreContent::Removed(")
        .into_iter()
        .filter(|&offset| !service[offset..].starts_with("StoreContent::Removed(removed)"))
        .map(|offset| enclosing_function(&service, offset).expect("a removed form built outside every function"))
        .flat_map(|builder| {
            if is_public(&builder) {
                vec![format!("service.rs::{builder}")]
            } else {
                public_entry_points_reaching(&builder)
            }
        })
        .collect();
    builders.sort();
    builders.dedup();
    assert_eq!(builders, ["service.rs::acknowledgement_accepted", "service.rs::verify"]);
}

/// `verify` and the functions it calls in revocation.rs, verify_org.rs and
/// removal.rs are values in, values out: no `async`, no chain, transport,
/// endpoint or file system, and no clock (LLR-g2jeqd); no public function of
/// theirs takes the store or itself mutably.
// verifies: LLR-g2jeqd
#[test]
fn verification_revocation_and_removal_do_no_io_and_read_no_clock() {
    let forbidden = [
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
        ".save(",
        "SystemTime",
        "Instant",
        "std::time",
    ];
    for file in ["revocation.rs", "verify_org.rs", "removal.rs"] {
        let code = code_of(&source_of(file));
        for name in forbidden {
            assert!(!code.contains(name), "{file} names `{name}` (LLR-g2jeqd)");
        }
        // The public functions' signatures, as LLR-g2jeqd states: a private helper may change the
        // fresh copy an adoption builds (`settle_admitted(next: &mut StoreData, …)`), and the removed
        // store's own list is changed through `acknowledgement_accepted` (LLR-9jn3z8), never here.
        // `pub fn` only, which `is_public` calls public; in verify_org.rs, whose functions are all
        // `pub(crate)` since merge review MR1 (finding-6) and are the ones `verify` calls, those.
        let prefixes: &[&str] = if file == "verify_org.rs" { &["pub fn ", "pub(crate) fn "] } else { &["pub fn "] };
        let signatures: Vec<String> = prefixes
            .iter()
            .flat_map(|prefix| identifier_occurrences(&code, prefix))
            .map(|start| squeezed(&code[start..start + code[start..].find(['{', ';']).unwrap()]))
            .collect();
        assert!(!signatures.is_empty(), "{file}: the scan reads its public functions");
        for signature in &signatures {
            for borrow in ["&mutStoreData", "&mutself"] {
                assert!(!signature.contains(borrow), "{file}: `{signature}` takes `{borrow}` (LLR-g2jeqd: the store is taken as a value)");
            }
        }
    }
    let service = code_of(&source_of("service.rs"));
    let verify = &service[service.find("pub fn verify<").expect("OrgService::verify")..];
    let verify = &verify[..verify.find("\n    }\n").expect("the end of verify")];
    for name in forbidden {
        assert!(!verify.contains(name), "verify names `{name}` (LLR-g2jeqd)");
    }
    assert!(verify.contains("chain: Option<OrgState>"), "verify takes the chain state as a value");
    assert!(!service.contains("SystemTime") && !service.contains("Instant::now"), "service.rs reads no clock: the time is the caller's");
}

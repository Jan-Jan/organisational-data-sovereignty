#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! What org-io must not contain (LLR-3zdw8v): read from its own source, in
//! the style of org-node/tests/absences.rs.

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

fn sources() -> Vec<(PathBuf, String)> {
    sources_under(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"))
}

fn sources_under(dir: &Path) -> Vec<(PathBuf, String)> {
    let mut files = vec![];
    rust_files(dir, &mut files);
    files
        .into_iter()
        .map(|path| {
            let text = std::fs::read_to_string(&path).unwrap();
            (path, text)
        })
        .collect()
}

// verifies: LLR-3zdw8v
#[test]
fn the_seed_is_read_once_and_only_behind_dev_seed() {
    let mut reads = vec![];
    for (path, text) in sources() {
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            if line.contains("ODS_ADMIN_SEED") && line.contains("env::var") {
                reads.push(format!("{}:{}", path.display(), index + 1));
                // The read must sit inside `signatory_from_environment`: the
                // nearest `fn` line above it is that function's, and one of
                // the three lines above that is the dev-seed gate.
                let function_line = (0..index)
                    .rev()
                    .find(|&earlier| lines[earlier].contains("fn "))
                    .unwrap();
                assert!(
                    lines[function_line].contains("fn signatory_from_environment"),
                    "{}:{}: the seed is read outside signatory_from_environment",
                    path.display(),
                    index + 1
                );
                let gate_window = &lines[function_line.saturating_sub(3)..function_line];
                assert!(
                    gate_window
                        .iter()
                        .any(|attribute| attribute.contains("#[cfg(feature = \"dev-seed\")]")),
                    "{}:{}: signatory_from_environment is not behind dev-seed",
                    path.display(),
                    function_line + 1
                );
            }
        }
    }
    assert_eq!(reads.len(), 1, "exactly one read of ODS_ADMIN_SEED: {reads:?}");
}

// verifies: LLR-3zdw8v
#[test]
fn the_key_holders_derive_no_debug_and_have_no_display() {
    for (path, text) in sources() {
        for holder in ["SignatoryKey", "SeedBytes"] {
            let declaration = format!("pub struct {holder}");
            if let Some(position) = text.find(&declaration) {
                let before = &text[..position];
                let attributes: Vec<&str> = before
                    .lines()
                    .rev()
                    .take_while(|line| {
                        line.trim_start().starts_with("#[") || line.trim_start().starts_with("///")
                    })
                    .collect();
                assert!(
                    attributes.iter().all(|line| !line.contains("Debug")),
                    "{}: {holder} derives Debug",
                    path.display()
                );
            }
            assert!(
                !text.contains(&format!("Display for {holder}")),
                "{}: {holder} implements Display",
                path.display()
            );
        }
    }
}

// verifies: LLR-3zdw8v
#[test]
fn org_io_names_no_device_or_member_private_key() {
    for (path, text) in sources() {
        for needle in ["ed25519_dalek::SigningKey", "SigningKeypair", "DeviceSeed", "MemberSeed"] {
            assert!(!text.contains(needle), "{} names `{needle}`", path.display());
        }
    }
}

// *Clarified 2026-10-08 (S4, task T3; plan §17.2 rule 5):* org-io now
// receives a device private key, as a move-only `DeviceSecret`, from
// org-node's `create_persona`. The type is named only in its sanctioned
// places: in org-io, the transitional holder `device_secrets.rs` (and, from
// task T8, the keychain module `keychain.rs`); in org-node, `keys.rs`, which
// defines it, and `create_persona`, the one production public function that
// returns one (`unseal` and the signing operations take one).
// *Narrowed 2026-10-14 (S4 review round 1, finding 9):* the deleted
// transitional holder `device_secrets.rs` is no longer sanctioned; a file of
// that name naming the type now fails the scan. Mutation check, that day: a
// planted `src/device_secrets.rs` holding a `DeviceSecret` failed it
// ("…/device_secrets.rs names DeviceSecret"); the file was removed.
// verifies: LLR-3zdw8v
#[test]
fn device_secret_is_named_only_in_its_sanctioned_places() {
    let sanctioned = ["keychain.rs"];
    // The identifier itself, not a longer one it begins (`DeviceSecrets`,
    // `DeviceSecretMismatch`) or ends.
    let names_the_type = |text: &str| {
        text.match_indices("DeviceSecret").any(|(offset, needle)| {
            let is_identifier = |character: char| character.is_alphanumeric() || character == '_';
            let before = text[..offset].chars().next_back().is_some_and(is_identifier);
            let after = text[offset + needle.len()..].chars().next().is_some_and(is_identifier);
            !before && !after
        })
    };
    let mut named_in = vec![];
    for (path, text) in sources() {
        if names_the_type(&text) {
            let file = path.file_name().unwrap().to_string_lossy().to_string();
            assert!(sanctioned.contains(&file.as_str()), "{} names DeviceSecret", path.display());
            named_in.push(file);
        }
    }
    // *Adapted 2026-10-13 (S4, task T8):* the keychain module replaced the
    // transitional holder, which is gone.
    assert!(named_in.iter().any(|file| file == "keychain.rs"), "the keychain module reads and writes the secrets");

    let org_node_src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../org-node/src");
    let mut returners = vec![];
    for (path, text) in sources_under(&org_node_src) {
        if ["keys.rs", "test_fixtures.rs"].iter().any(|name| path.ends_with(name)) {
            continue;
        }
        let lines: Vec<&str> = text.lines().collect();
        for signature in public_signatures(&text) {
            let in_production =
                !signature.only_in_test_builds() && !enclosing_item_only_in_test_builds(&lines, signature.line - 1);
            if in_production && signature.returns().contains("DeviceSecret") {
                let after = &signature.signature[signature.signature.find("fn ").unwrap() + 3..];
                returners.push(after[..after.find(['(', '<']).unwrap()].to_string());
            }
        }
    }
    assert_eq!(returners, ["create_persona"], "org-node hands out a DeviceSecret from create_persona alone");
}

/// One public function as written: its whole signature, from `pub fn` (or
/// `pub async fn`) to the `{` or `;` that ends it, joined across lines, and
/// the attribute lines directly above it.
struct PublicSignature {
    line: usize,
    signature: String,
    attributes: Vec<String>,
}

impl PublicSignature {
    /// The return type: after the first `->` outside every `(..)` and `<..>`,
    /// so a closure parameter's `impl FnOnce() -> T` is not taken for it.
    fn returns(&self) -> &str {
        let bytes = self.signature.as_bytes();
        let mut depth = 0i32;
        let mut index = 0;
        while index < bytes.len() {
            match bytes[index] {
                b'-' if bytes.get(index + 1) == Some(&b'>') => {
                    if depth == 0 {
                        return &self.signature[index + 2..];
                    }
                    index += 1;
                }
                b'(' | b'<' => depth += 1,
                b')' | b'>' => depth -= 1,
                _ => {}
            }
            index += 1;
        }
        ""
    }

    fn only_in_test_builds(&self) -> bool {
        self.attributes.iter().any(|line| line.contains("feature = \"test-support\""))
    }
}

/// Whether the top-level item (`impl`, `struct`, …) enclosing line `index`
/// is compiled only with `test-support`.
fn enclosing_item_only_in_test_builds(lines: &[&str], index: usize) -> bool {
    let Some(item) = (0..index).rev().find(|&earlier| {
        let line = lines[earlier];
        !line.starts_with(' ') && !line.starts_with('}') && !line.starts_with("#[") && !line.starts_with("//") && !line.is_empty()
    }) else {
        return false;
    };
    lines[..item]
        .iter()
        .rev()
        .take_while(|above| above.trim_start().starts_with("#[") || above.trim_start().starts_with("///"))
        .any(|above| above.contains("feature = \"test-support\""))
}

fn public_signatures(text: &str) -> Vec<PublicSignature> {
    let lines: Vec<&str> = text.lines().collect();
    let mut signatures = vec![];
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !(trimmed.starts_with("pub fn ") || trimmed.starts_with("pub async fn ")) {
            continue;
        }
        let mut signature = String::new();
        for continuation in &lines[index..] {
            let end = continuation.find(['{', ';']);
            signature.push_str(continuation[..end.unwrap_or(continuation.len())].trim());
            signature.push(' ');
            if end.is_some() {
                break;
            }
        }
        let attributes = lines[..index]
            .iter()
            .rev()
            .take_while(|above| above.trim_start().starts_with("#[") || above.trim_start().starts_with("///"))
            .map(|above| above.to_string())
            .collect();
        signatures.push(PublicSignature { line: index + 1, signature, attributes });
    }
    signatures
}

/// The attribute lines directly above `pub struct {name}` in `text`.
fn struct_attributes(text: &str, name: &str) -> Option<Vec<String>> {
    let position = text.find(&format!("pub struct {name} "))?;
    Some(
        text[..position]
            .lines()
            .rev()
            .take_while(|line| line.trim_start().starts_with("#[") || line.trim_start().starts_with("///"))
            .map(str::to_string)
            .collect(),
    )
}

// org-node's operations that judge against the chain take the state org-io
// read only through `OrgIo`'s own methods: no public method of lib.rs hands
// out `&mut OrgService`, and none named for a chain-judging operation is
// offered outside `impl OrgIo` (where they are the sequenced wrappers).
// verifies: LLR-7pj5af, LLR-9r2bxd
#[test]
fn the_handle_offers_no_chain_judging_operation_outside_its_own_sequence() {
    let lib = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs")).unwrap();
    for signature in public_signatures(&lib).iter().filter(|signature| !signature.only_in_test_builds()) {
        assert!(
            !signature.returns().contains("&mut OrgService"),
            "lib.rs:{}: hands out org-node's whole service: {}",
            signature.line,
            signature.signature
        );
    }
    let lines: Vec<&str> = lib.lines().collect();
    for signature in public_signatures(&lib) {
        let item = (0..signature.line - 1).rev().find(|&earlier| lines[earlier].starts_with("impl")).map(|earlier| lines[earlier]);
        if item.is_some_and(|line| line.starts_with("impl OrgIo")) {
            continue;
        }
        // *S4 T4, 2026-10-13:* `fn check_received` joins the list; `fn commit_`
        // already covers `commit_checked`.
        for judging in ["fn commit_", "fn reconcile", "fn prepare_", "fn apply_", "fn check_received", "fn discard_provisional"] {
            assert!(!signature.signature.contains(judging), "lib.rs:{}: offers {judging}", signature.line);
        }
    }
}

// No other way to a whole `OrgService` (review round 2, finding 2): no
// public field of org-io holds one and no public function outside test
// builds returns one by value, so a refused connect cannot hand the app the
// service whose chain-judging operations org-io sequences.
// verifies: LLR-7pj5af, LLR-9r2bxd
#[test]
fn org_io_hands_out_no_bare_service() {
    let mut ways_out = vec![];
    for (path, text) in sources() {
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            let field = line.trim_start();
            if field.starts_with("pub ") && !field.starts_with("pub fn") && !field.starts_with("pub async fn") && field.contains(": OrgService") {
                ways_out.push(format!("{}:{}: a public OrgService field: {field}", path.display(), index + 1));
            }
        }
        let in_production = |signature: &&PublicSignature| {
            !signature.only_in_test_builds() && !enclosing_item_only_in_test_builds(&lines, signature.line - 1)
        };
        for signature in public_signatures(&text).iter().filter(in_production) {
            let returns = signature.returns().trim();
            let by_value = returns.split(|character: char| !character.is_alphanumeric() && character != '_' && character != '&')
                .any(|word| word == "OrgService");
            if by_value {
                ways_out.push(format!("{}:{}: returns an OrgService: {}", path.display(), signature.line, signature.signature));
            }
        }
    }
    assert!(ways_out.is_empty(), "ways to a whole OrgService:\n{}", ways_out.join("\n"));
}

fn custody_source() -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/custody.rs")).unwrap()
}

// Every copy of the sr25519 seed org-io makes is wiped: the environment
// value is held `Zeroizing`; the parse writes the bytes in place into
// `SeedBytes`, which holds them on the heap so a move copies only a pointer;
// no `[u8; 32]` temporary is built, returned or bound on the path. The one
// copy left is the by-value argument subxt-signer's `from_secret_key` takes
// (SOUP), passed straight from `SeedBytes`.
// verifies: LLR-c4bktx
#[test]
fn the_seed_path_keeps_no_unwiped_copy_of_the_seed() {
    assert_eq!(
        std::mem::size_of::<org_io::custody::SeedBytes>(),
        std::mem::size_of::<usize>(),
        "SeedBytes holds its bytes behind a pointer"
    );
    let custody = custody_source();
    for temporary in ["[0u8; 32]", "-> Result<[u8; 32]", "-> [u8; 32]", "let seed_bytes", "let bytes ="] {
        assert!(!custody.contains(temporary), "custody.rs builds a seed temporary: {temporary}");
    }
    assert_eq!(custody.matches("seed.bytes()").count(), 1, "the seed's bytes are read once");
    assert!(custody.contains("Keypair::from_secret_key(*seed.bytes())"), "and only as from_secret_key's argument");
    let environment_read = &custody[custody.find("fn signatory_from_environment").unwrap()..];
    assert!(environment_read.contains("Zeroizing::new("), "the environment value is held Zeroizing");
}

// `same_bytes_as` compares the secret in non-constant time: a test helper,
// absent from production builds.
// verifies: LLR-c4bktx
#[test]
fn seed_bytes_offers_no_comparison_outside_test_builds() {
    let custody = custody_source();
    for signature in public_signatures(&custody).iter().filter(|signature| signature.signature.contains("fn same_bytes_as")) {
        assert!(signature.only_in_test_builds(), "custody.rs:{}: same_bytes_as is public in production", signature.line);
    }
}

// The `node` re-export reaches all of org-node's public surface (the
// handle's own surface is an allowlist since review round 3: see
// `the_handle_returns_only_allowlisted_public_types`); this reads that
// surface for a way out for a
// stored device private key: a public seed field, a public function handing
// out a seed, a key pair or the iroh endpoint (whose `secret_key()` is the
// device key), or a production `Serialize` on the records that hold seeds.
// keys.rs and types.rs are exempt: their conversions need a seed in hand.
// verifies: LLR-3zdw8v
#[test]
fn org_node_hands_out_no_stored_device_private_key() {
    let org_node_src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../org-node/src");
    let mut ways_out = vec![];
    for (path, text) in sources_under(&org_node_src) {
        let file = path.display();
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            if line.trim_start().starts_with("pub device_seed") && !enclosing_item_only_in_test_builds(&lines, index) {
                ways_out.push(format!("{file}:{}: a public device_seed field", index + 1));
            }
        }
        let exempt = ["keys.rs", "types.rs", "test_fixtures.rs"].iter().any(|name| path.ends_with(name));
        let in_production = |signature: &&PublicSignature| {
            !signature.only_in_test_builds() && !enclosing_item_only_in_test_builds(&lines, signature.line - 1)
        };
        let signatures = if exempt { vec![] } else { public_signatures(&text) };
        for signature in signatures.iter().filter(in_production) {
            for key in ["DeviceSeed", "SigningKeypair", "iroh::Endpoint", "SecretKey"] {
                if signature.returns().contains(key) {
                    ways_out.push(format!("{file}:{}: hands out {key}: {}", signature.line, signature.signature));
                }
            }
        }
        for holder in ["PersonaRecord", "StoreData"] {
            ways_out.extend(production_serialize(&file.to_string(), &text, holder));
        }
    }
    assert!(ways_out.is_empty(), "ways out for a stored device private key:\n{}", ways_out.join("\n"));
}

// verifies: LLR-u2pk5y
#[test]
fn the_handle_hands_out_no_key() {
    let lib = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs")).unwrap();
    let submit = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/submit.rs")).unwrap();
    // Whole signatures, joined across lines: a return type on a continuation
    // line is read as well (review round 1, finding 8). *S4 T11,
    // 2026-10-13:* the receive step's `impl OrgIo` is in receiver.rs.
    let receiver = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/receiver.rs")).unwrap();
    for signature in public_signatures(&lib).into_iter().chain(public_signatures(&receiver)) {
        // Private keys and seeds only: a public key or account may be handed
        // out (owner clarification, 2026-10-08), so subxt-signer's
        // `PublicKey` is not flagged, its `Keypair` is.
        for needle in ["SignatoryKey", "SeedBytes", "Keypair"] {
            assert!(
                !signature.signature.contains(needle),
                "lib.rs:{}: a public OrgIo method names {needle}: {}",
                signature.line,
                signature.signature
            );
        }
    }
    // The fields: between the declaration's opening and closing braces.
    let org_io_struct = &lib[lib.find("pub struct OrgIo {").unwrap() + "pub struct OrgIo {".len()..];
    let org_io_body = &org_io_struct[..org_io_struct.find('}').unwrap()];
    assert!(!org_io_body.contains("pub "), "OrgIo has no public field");
    assert!(!submit.contains("pub signatory"), "OnChainWriter's key field is private");
    for (text, holder) in [(&lib, "OrgIo"), (&submit, "OnChainWriter")] {
        let position = text.find(&format!("pub struct {holder}")).unwrap();
        let attributes = text[..position]
            .lines()
            .rev()
            .take_while(|line| line.trim_start().starts_with("#[") || line.trim_start().starts_with("///"));
        assert!(attributes.into_iter().all(|line| !line.contains("Debug")), "{holder} derives Debug");
    }
}

/// Each production `Serialize` on `holder` in `text`: a derive not gated on
/// `test-support`, or a hand-written impl without that gate above it.
fn production_serialize(file: &str, text: &str, holder: &str) -> Vec<String> {
    let mut found = vec![];
    let lines: Vec<&str> = text.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        let hand_written = line.contains(&format!("Serialize for {holder} "))
            && !line.contains("Deserialize for")
            && line.trim_start().starts_with("impl");
        let gated = index > 0 && lines[..index].iter().rev().take(3).any(|above| above.contains("feature = \"test-support\""));
        if hand_written && !gated {
            found.push(format!("{file}:{}: {holder} is Serialize in production: {}", index + 1, line.trim()));
        }
    }
    if let Some(attributes) = struct_attributes(text, holder) {
        let derives_serialize = |line: &&String| {
            line.split([',', '(', ')']).any(|word| matches!(word.trim(), "Serialize" | "serde::Serialize"))
        };
        for attribute in attributes.iter().filter(derives_serialize) {
            if !attribute.contains("cfg_attr(feature = \"test-support\"") {
                found.push(format!("{file}: {holder} is Serialize in production: {}", attribute.trim()));
            }
        }
    }
    found
}

// The owner's rule is no private key, X25519 secrets included (review round
// 2, finding 1): the Organisation private key and the Member seed. Read
// through the same surface as the device-seed scan above: no public field
// holding one (`OrgRecord.org_private_key`, `PersonaRecord.member_seed`, or
// `ProvisionalUpdate.change`, whose variants carry the next Organisation
// private key), no public function returning one or an `X25519Keypair`, and
// no production `Serialize` on the records that hold one.
// verifies: LLR-3zdw8v
#[test]
fn org_node_hands_out_no_stored_x25519_secret() {
    let org_node_src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../org-node/src");
    let mut ways_out = vec![];
    for (path, text) in sources_under(&org_node_src) {
        let file = path.display().to_string();
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            let field = line.trim_start();
            let holds_secret = ["pub member_seed", "pub org_private_key", "pub change: ProvisionalChange"]
                .iter()
                .any(|declaration| field.starts_with(declaration));
            if holds_secret && !enclosing_item_only_in_test_builds(&lines, index) {
                ways_out.push(format!("{file}:{}: a public secret-holding field: {field}", index + 1));
            }
        }
        let exempt = ["keys.rs", "types.rs", "test_fixtures.rs"].iter().any(|name| path.ends_with(name));
        let in_production = |signature: &&PublicSignature| {
            !signature.only_in_test_builds() && !enclosing_item_only_in_test_builds(&lines, signature.line - 1)
        };
        let signatures = if exempt { vec![] } else { public_signatures(&text) };
        for signature in signatures.iter().filter(in_production) {
            for secret in ["MemberSeed", "OrgPrivateKey", "X25519Keypair", "ProvisionalChange"] {
                if signature.returns().contains(secret) {
                    ways_out.push(format!("{file}:{}: hands out {secret}: {}", signature.line, signature.signature));
                }
            }
        }
        for holder in ["OrgRecord", "PersonaRecord", "ProvisionalUpdate", "StoreData"] {
            ways_out.extend(production_serialize(&file, &text, holder));
        }
    }
    assert!(ways_out.is_empty(), "ways out for a stored X25519 secret:\n{}", ways_out.join("\n"));
}

/// Every type a production public method of the handle may name in its
/// return type (review round 3, finding 1): the handle itself, its views,
/// public-only summaries, ids, public keys, roots, epochs, errors and
/// org-node's receive outcome values, which carry public data only (signed
/// acknowledgements). Containers
/// and path segments are listed too. Anything else is a way out until it is
/// reviewed and added here.
const HANDLE_RETURN_ALLOWLIST: &[&str] = &[
    // Containers, primitives and path segments.
    "Result", "Option", "Vec", "Box", "Self", "String", "bool", "usize", "u64", "crate", "org_node", "connect",
    "signatory", "view", "std",
    // The handle and its public-only views and summaries.
    // *S4 T5, 2026-10-13:* `Submitted`, what `submit_commit_send` reports.
    // *S4 T11, 2026-10-13:* `NodeBuilders` and `NodeView` left: their
    // methods are `OrgIo`'s own, each returning owned public values.
    "OrgIo", "PersonaSummary", "OrgSummary", "BuiltUpdate", "ConnectFailed", "Submitted",
    // Ids, public keys, roots, epochs.
    "OrgId", "PersonaId", "MemberId", "DevicePublicKey", "PersonPublicKey", "OrgPublicKey", "RootHash", "Epoch",
    // Answers and errors.
    // *S4 T4, 2026-10-13:* `SelfDeleteOutcome` left with the self-delete
    // path; `Received` holds a `ReceiveOutcome` or a verified acknowledgement.
    // *S4 T5, 2026-10-13:* `CommitOutcome` left: its Organisation-information
    // bodies carry the Organisation private key.
    "AdminStatus", "OwnAdminError", "OrgNodeError", "ReceiveOutcome", "Received",
    // *S4 T9, 2026-10-13:* the store files' errors, public data only: paths,
    // Device public keys, the steps and reasons of failed file operations,
    // and the keychain's and org-node's typed refusals.
    "OpenError", "CreateError", "OperationError",
    // *S4 T10, 2026-10-13:* what `start_receiving` refuses with, public data
    // only: reasons as text, lengths, Device public keys and the keychain's
    // typed refusals (no endpoint, no key, no body).
    "TransportError",
    // *S4 T11, 2026-10-13:* what the receive step returns, public data
    // only: a `Received` holds a `ReceiveOutcome` (ids, epochs, roots, the
    // signed acknowledgements) or a `VerifiedAcknowledgement` (ids, an
    // epoch, a root, a Device public key, a signature); a `ReceiveError`
    // holds a `TerminalCause` and org-node's, the keychain's, the store's
    // and the transport's typed refusals. No Organisation information, key
    // or body.
    "ReceiveError", "TerminalCause", "VerifiedAcknowledgement",
    // *S4 T12, 2026-10-13:* the per-Device send report `Submitted` carries
    // (Device public keys and typed send errors, which hold reasons as text
    // and the transport's typed refusals) and the parsed peer address (an
    // endpoint identity and the transport addresses the operator gave). No
    // body, key or seed.
    "SendReport", "SendError", "PeerAddr",
    // *S4 T13, 2026-10-14:* the invitation exchange's answers, public data
    // only: an imported `InviteReply` (an Organisation id, an invite
    // identifier, a member key, a Device public key, a handle, a name and
    // a surname — what its sender put in the Blob), an `AdmitReport` (a
    // MemberId and a `SendReport`) and an `ExchangeError` (org-node's typed
    // refusals, a `StoreFailure`, the submission's report as text, or an
    // `AdmitReport` with an `OperationError`). The Blobs are `String`s
    // carrying the same public values. No body, key or seed.
    "InviteReply", "AdmitReport", "ExchangeError",
    // *Reconciled with S3b-io part 1, 2026-10-14:* S3b-io's line also listed
    // `CommitOutcome` and `SelfDeleteOutcome`, which S4 removed (T5, T4,
    // above); its additions follow.
    // The publish state's reports (SDD-6mpmsw, task T14), reviewed 2026-10-08:
    // `Settled` holds an OrgId, an Epoch, strings and a `Reconciliation`
    // (OrgIds, Epochs, DevicePublicKeys, strings). No private key, no seed,
    // no acknowledgement. *2026-10-13 (task T15d):* `DiscardRefused` left the
    // list with the public discards (owner ruling of that day).
    "Settled",
    // Task T15, reviewed 2026-10-13: `Reconciliation` holds OrgIds, Epochs,
    // DevicePublicKeys and strings (a removal's acknowledgements stay in
    // org-io's memory); `Verification` holds a string. No private key, no
    // seed, no acknowledgement.
    "Reconciliation", "Verification",
    // Task T16a, reviewed 2026-10-13: `AdminStatus` (above) gained
    // `DelegateCannotDispatch`, a variant holding nothing; the signatory set
    // it is judged from (`SignatorySetView`: an account, proxy definitions
    // and a type index, public chain data) is not returned by the handle.
];

/// The identifiers a return type names, lifetimes dropped.
fn named_types(returns: &str) -> Vec<String> {
    let mut without_lifetimes = String::new();
    let mut characters = returns.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\'' {
            while characters.peek().is_some_and(|next| next.is_alphanumeric() || *next == '_') {
                characters.next();
            }
            continue;
        }
        without_lifetimes.push(character);
    }
    without_lifetimes
        .split(|character: char| !character.is_alphanumeric() && character != '_')
        .filter(|word| !word.is_empty())
        .map(str::to_string)
        .collect()
}

// The handle's read surface is an allowlist, not a name scan (review round
// 3, finding 1): every public method of `OrgIo` (*S4 T11:* wherever its
// `impl` is; S2's `NodeBuilders` and `NodeView` are folded into it) and
// `ConnectFailed` outside test builds returns only allowlisted types, so
// no method hands out `OrgService` (whose `endpoint()` reaches `recv_one`
// and `send`), `OrgEndpoint`, `WireMessage`, `OrgRecord`, `PersonaRecord`,
// `ProvisionalUpdate` or anything else unreviewed; and no handle type
// dereferences to another type, which would offer that type's methods.
// verifies: LLR-3zdw8v, LLR-7pj5af, LLR-9r2bxd
#[test]
fn the_handle_returns_only_allowlisted_public_types() {
    let mut ways_out = vec![];
    for (path, text) in sources() {
        let file = path.display().to_string();
        for (index, line) in text.lines().enumerate() {
            let derefs = line.starts_with("impl") && (line.contains("Deref for") || line.contains("DerefMut for"));
            if derefs && ["OrgIo", "ConnectFailed"].iter().any(|holder| line.contains(holder)) {
                ways_out.push(format!("{file}:{}: a handle type dereferences: {line}", index + 1));
            }
        }
    }
    let methods = handle_methods();
    for (file, signature) in &methods {
        for named in named_types(signature.returns()) {
            if !HANDLE_RETURN_ALLOWLIST.contains(&named.as_str()) {
                ways_out.push(format!("{file}:{}: returns {named}, not allowlisted: {}", signature.line, signature.signature));
            }
        }
    }
    assert!(methods.len() > 10, "the scan found the handle's methods ({})", methods.len());
    assert!(ways_out.is_empty(), "the handle's surface beyond the allowlist:\n{}", ways_out.join("\n"));
}

/// Every production public method of the handle's types (`OrgIo`,
/// `ConnectFailed`), with its file. *S4 T11, 2026-10-13:* `NodeBuilders`
/// and `NodeView` are gone, folded into `OrgIo`.
fn handle_methods() -> Vec<(String, PublicSignature)> {
    let handle_items = ["impl OrgIo", "impl crate::OrgIo", "impl ConnectFailed"];
    let mut methods = vec![];
    for (path, text) in sources() {
        let file = path.display().to_string();
        let lines: Vec<&str> = text.lines().collect();
        for signature in public_signatures(&text) {
            let item = (0..signature.line - 1).rev().find(|&earlier| lines[earlier].starts_with("impl")).map(|earlier| lines[earlier]);
            if !item.is_some_and(|line| handle_items.iter().any(|handle_item| line.starts_with(handle_item))) {
                continue;
            }
            if signature.only_in_test_builds() || enclosing_item_only_in_test_builds(&lines, signature.line - 1) {
                continue;
            }
            methods.push((file.clone(), signature));
        }
    }
    methods
}

/// The public types of org-node's sources that hold an `OutgoingMessage`:
/// the type itself and, to a fixed point, every `pub struct` or `pub enum`
/// whose body names one that does.
fn outgoing_holders() -> Vec<String> {
    let org_node_src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../org-node/src");
    let mut items: Vec<(String, String)> = vec![];
    for (_, text) in sources_under(&org_node_src) {
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            let Some(rest) = line.strip_prefix("pub struct ").or_else(|| line.strip_prefix("pub enum ")) else {
                continue;
            };
            let name: String = rest.chars().take_while(|character| character.is_alphanumeric() || *character == '_').collect();
            let body: Vec<&str> = lines[index..].iter().take_while(|body_line| !body_line.starts_with('}')).copied().collect();
            items.push((name, code_lines(&body.join("\n")).join("\n")));
        }
    }
    let mut holders = vec!["OutgoingMessage".to_string()];
    loop {
        let found: Vec<String> = items
            .iter()
            .filter(|(name, body)| !holders.contains(name) && named_types(body).iter().any(|named| holders.contains(named)))
            .map(|(name, _)| name.clone())
            .collect();
        if found.is_empty() {
            return holders;
        }
        holders.extend(found);
    }
}

// An Organisation-information body carries the Organisation private key
// (S4, task T5; plan §17.2 rule 2): no handle method returns a type holding
// an `OutgoingMessage` — `CommitOutcome` left the allowlist and
// `submit_commit_send` returns `Submitted`, public data only. The holders
// are found in org-node's source; the scan is checked to find
// `CommitOutcome` among them.
// *2026-10-14 (S4, task T18):* org-node's LLR for the commit's bodies (that
// every recipient's body is built by the commit) was cited here too; it is
// not exported to org-io, so the cross-unit ID is removed. This test
// verifies org-io's handle rule, LLR-3zdw8v, as before.
// verifies: LLR-3zdw8v
#[test]
fn no_handle_method_returns_a_type_holding_an_outgoing_body() {
    let holders = outgoing_holders();
    assert!(holders.iter().any(|holder| holder == "CommitOutcome"), "the scan finds CommitOutcome: {holders:?}");
    let ways_out: Vec<String> = handle_methods()
        .iter()
        .flat_map(|(file, signature)| {
            named_types(signature.returns())
                .into_iter()
                .filter(|named| holders.contains(named))
                .map(move |named| format!("{file}:{}: returns {named}: {}", signature.line, signature.signature))
        })
        .collect();
    assert!(ways_out.is_empty(), "handle methods returning an outgoing body:\n{}", ways_out.join("\n"));
}

// The pre-write guard of `submit_commit_send` copies no record and no held
// provisional update, each of which holds an Organisation private key
// (review round 4, finding 2): from its signature to the chain write, no
// code line clones or calls `provisional_updates(`.
// verifies: LLR-qhjp6g
#[test]
fn the_pre_write_guard_copies_no_record_and_no_held_update() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("submit.rs");
    let text = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.iter().position(|line| line.contains("pub async fn submit_commit_send")).unwrap();
    let write = (start..lines.len()).find(|&index| lines[index].contains("bounded(writer.update(")).unwrap();
    let copies: Vec<String> = (start..write)
        .filter(|&index| !lines[index].trim_start().starts_with("//"))
        .filter(|&index| ["clone()", "cloned()", "provisional_updates("].iter().any(|copy| lines[index].contains(copy)))
        .map(|index| format!("{}:{}: {}", path.display(), index + 1, lines[index].trim()))
        .collect();
    assert!(write > start + 5, "the scan found the guard ({start}..{write})");
    assert!(copies.is_empty(), "the pre-write guard copies:\n{}", copies.join("\n"));
}

/// A source's lines with `//` comments (doc comments included) removed.
fn code_lines(text: &str) -> Vec<&str> {
    text.lines().map(|line| line.split("//").next().unwrap_or("")).collect()
}

/// The `[dependencies]` table of a manifest, as `name = …` lines: from its
/// header to the next table header.
fn dependency_names(manifest: &str) -> Vec<String> {
    let mut names = vec![];
    let mut in_dependencies = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_dependencies = line == "[dependencies]";
            continue;
        }
        if in_dependencies && !line.starts_with('#') {
            if let Some((name, _)) = line.split_once('=') {
                names.push(name.trim().to_string());
            }
        }
    }
    names
}

// The keychain crate is named in the keychain module and the test-only
// in-memory store alone; its in-memory stores (keyring's own `mock`, org-io's
// `MemoryKeychain`) only under `test-support`, which `lib.rs` compiles only
// with that feature; and the crate is built with its native backends only.
// *Abnormal case, 2026-10-14 (S4, task T23):* a planted file
// `src/t23_planted.rs` calling `keyring::Entry::new` failed this scan
// ("…/t23_planted.rs names keyring"); the file was removed.
// verifies: LLR-677fqq, LLR-36jer8
#[test]
fn keyring_is_named_only_in_the_keychain_module_and_its_mock_only_under_test_support() {
    let mut keychain_files = vec![];
    for (path, text) in sources() {
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        let code = code_lines(&text).join("\n");
        if code.contains("keyring::") {
            assert!(["keychain.rs", "test_support.rs"].contains(&file.as_str()), "{} names keyring", path.display());
            keychain_files.push(file.clone());
        }
        for in_memory in ["mock", "MemoryKeychain"] {
            if code.contains(in_memory) {
                assert_eq!(file, "test_support.rs", "{} names `{in_memory}`", path.display());
            }
        }
    }
    keychain_files.sort();
    assert_eq!(keychain_files, ["keychain.rs", "test_support.rs"], "the scan found the keychain's two files");

    let lib = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs")).unwrap();
    let lib_lines: Vec<&str> = lib.lines().collect();
    let declaration = lib_lines.iter().position(|line| line.trim() == "pub mod test_support;").unwrap();
    assert_eq!(lib_lines[declaration - 1].trim(), "#[cfg(feature = \"test-support\")]", "test_support is test-only");

    let manifest = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    let keyring_line = manifest.lines().find(|line| line.starts_with("keyring =")).unwrap();
    assert!(keyring_line.contains("default-features = false"), "{keyring_line}");
    let features_start = keyring_line.find("features = [").unwrap() + "features = [".len();
    let features_list = &keyring_line[features_start..];
    let mut features: Vec<&str> =
        features_list[..features_list.find(']').unwrap()].split(',').map(|feature| feature.trim().trim_matches('"')).collect();
    features.sort_unstable();
    assert_eq!(features, ["apple-native", "crypto-rust", "sync-secret-service", "windows-native"]);
}

// REQ-hw82ue, as amended 2026-10-07: org-io is the only unit that names a
// keychain crate.
// verifies: LLR-677fqq
#[test]
fn no_lower_unit_depends_on_a_keychain_crate() {
    let keychain_crates =
        ["keyring", "security-framework", "secret-service", "dbus-secret-service", "windows-native-keyring-store", "linux-keyutils"];
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for unit in ["org-node", "person", "org-members", "on-chain-client"] {
        let manifest = std::fs::read_to_string(root.join(unit).join("Cargo.toml")).unwrap();
        let names = dependency_names(&manifest);
        assert!(!names.is_empty(), "{unit}: the scan found its dependencies");
        for name in names {
            assert!(
                !keychain_crates.contains(&name.as_str()) && !name.contains("keyring") && !name.contains("credential"),
                "{unit} depends on the keychain crate `{name}`"
            );
        }
    }
}

/// `line` with the text of its string literals removed, so a message that
/// mentions a seed is not taken for code that handles one.
fn without_strings(line: &str) -> String {
    let mut code = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for character in line.chars() {
        if in_string {
            match (escaped, character) {
                (false, '\\') => escaped = true,
                (false, '"') => in_string = false,
                _ => escaped = false,
            }
        } else if character == '"' {
            in_string = true;
        } else {
            code.push(character);
        }
    }
    code
}

/// Every function of `text`, as its name and its code lines (comments
/// removed), from its declaration to the brace that closes its body; a
/// declaration without a body (a trait's) is skipped.
fn functions(text: &str) -> Vec<(String, Vec<(usize, String)>)> {
    let code = code_lines(text);
    let mut found = vec![];
    for (start, line) in code.iter().enumerate() {
        let trimmed = line.trim_start();
        let declares = ["fn ", "pub fn ", "async fn ", "pub async fn ", "pub(crate) fn ", "pub(crate) async fn ", "pub(super) fn "];
        let Some(after) = declares.iter().find_map(|declaration| trimmed.strip_prefix(declaration)) else {
            continue;
        };
        let name = after[..after.find(['(', '<']).unwrap_or(after.len())].to_string();
        let mut depth = 0i32;
        let mut opened = false;
        let mut body = vec![];
        for (index, continuation) in code.iter().enumerate().skip(start) {
            let stripped = without_strings(continuation);
            body.push((index + 1, continuation.to_string()));
            if !opened && stripped.trim_end().ends_with(';') {
                break;
            }
            for character in stripped.chars() {
                match character {
                    '{' => {
                        depth += 1;
                        opened = true;
                    }
                    '}' => depth -= 1,
                    _ => {}
                }
            }
            if opened && depth == 0 {
                found.push((name.clone(), body));
                break;
            }
        }
    }
    found
}

/// The secret-handling functions of the keychain module `text` (those whose
/// code names a secret, the item's bytes or the seed) and every line
/// of them that copies, formats or serialises: `(functions, violations)`.
fn keychain_copies(text: &str) -> (Vec<String>, Vec<String>) {
    // `Secret` and `secret` cover `DeviceSecret`, `get_secret`, `set_secret`
    // and every binding the module gives a secret.
    let handles_a_secret = ["Secret", "secret", "seed", "lend_to_keychain", "Zeroizing"];
    let copies = [".clone()", ":?}", ":#?}", "dbg!", "to_vec", "to_owned()", "Vec::from", "Serialize", "copy_from_slice"];
    let (mut scanned, mut violations) = (vec![], vec![]);
    for (name, body) in functions(text) {
        if !body.iter().any(|(_, line)| handles_a_secret.iter().any(|needle| without_strings(line).contains(needle))) {
            continue;
        }
        for (line_number, line) in &body {
            for copy in copies {
                if line.contains(copy) {
                    violations.push(format!("{line_number}: `{copy}` in {name}: {}", line.trim()));
                }
            }
        }
        scanned.push(name);
    }
    (scanned, violations)
}

// *Widened 2026-10-14 (S4 review round 1, finding 9):* the scan read only the
// lines naming `get_secret` or `seed`; it now reads every line of every
// function of the keychain module that handles a secret (whose code names a
// `DeviceSecret`, the item's bytes, the seed or its `Zeroizing` buffer),
// string literals aside, so a copy made one line away from the read is seen.
// Mutation check, 2026-10-14: the module's text, read into the test, with
// `let kept = item.clone();` planted in memory above `with_endpoint_seed`'s
// `Ok(build(seed))` (a line the old scan skipped) failed this scan
// ("147: `.clone()` in with_endpoint_seed"); the plant was removed.
// verifies: LLR-677fqq
#[test]
fn keychain_bytes_reach_no_clone_debug_or_serialise() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("keychain.rs");
    let text = std::fs::read_to_string(&path).unwrap();
    let (scanned, violations) = keychain_copies(&text);
    for expected in [
        "read_device_secret",
        "write_device_secret",
        "with_endpoint_seed",
        "keep",
        "for_signing",
        "for_signing_noting_refusal",
        "unseal",
        "device_secrets_for_signing",
    ] {
        assert!(scanned.iter().any(|name| name == expected), "the scan read `{expected}`: {scanned:?}");
    }
    assert!(violations.is_empty(), "{}: the keychain bytes are copied or formatted:\n{}", path.display(), violations.join("\n"));
}

// The stores before S4 (S4, task T9): one passphrase store,
// `persona_store.bin`, and the app's `outstanding_invites.json` beside it.
// Outstanding invites live only inside the sealed stores now, and no code in
// `org-io/src` names either file, comments included.
// *Abnormal case, 2026-10-14 (S4, task T23):* a planted file
// `src/t23_planted.rs` holding the string "persona_store.bin" failed this
// scan ("…/t23_planted.rs names `persona_store.bin`"); the file was removed.
// verifies: LLR-t4v69b
#[test]
fn no_code_names_the_old_files() {
    let mut scanned = 0;
    for (path, text) in sources() {
        scanned += 1;
        for old in ["outstanding_invites.json", "persona_store.bin"] {
            assert!(!text.contains(old), "{} names `{old}`", path.display());
        }
    }
    assert!(scanned > 5, "the scan read org-io's sources ({scanned})");
}

/// The name of the function whose body holds line `index`: the nearest
/// line above it that declares one.
fn enclosing_function(lines: &[&str], index: usize) -> Option<String> {
    let declaration = (0..=index).rev().find(|&earlier| {
        let trimmed = lines[earlier].trim_start();
        ["fn ", "pub fn ", "async fn ", "pub async fn ", "pub(crate) fn ", "pub(crate) async fn "]
            .iter()
            .any(|start| trimmed.starts_with(start))
    })?;
    let after = &lines[declaration][lines[declaration].find("fn ")? + 3..];
    Some(after[..after.find(['(', '<']).unwrap_or(after.len())].to_string())
}

// iroh's secret-key type is named in one function only (S4, task T10):
// `bind_from_keychain` builds it from the keychain item's lent seed and moves
// it into the endpoint builder. Nothing in `org-io/src` reads it back
// (`to_bytes()`, `.secret()`, the endpoint's `.secret_key()`), clones it
// or formats it; and no public type has a public field holding an iroh
// `Endpoint` or an `OrgEndpoint`. Comments are not code: the scan reads code
// lines only.
// verifies: LLR-d5y7ks
#[test]
fn iroh_secret_key_is_named_in_one_function_only() {
    let mut naming_functions = vec![];
    for (path, text) in sources() {
        let code = code_lines(&text);
        for (index, line) in code.iter().enumerate() {
            if line.contains("SecretKey") {
                let function = enclosing_function(&code, index);
                assert_eq!(
                    function.as_deref(),
                    Some("bind_from_keychain"),
                    "{}:{}: SecretKey named outside bind_from_keychain: {line}",
                    path.display(),
                    index + 1
                );
                naming_functions.push(format!("{}:{}", path.display(), index + 1));
                for formatted in ["{:?}", "{:#?}", ".clone()", "dbg!"] {
                    assert!(!line.contains(formatted), "{}:{}: `{formatted}` beside SecretKey: {line}", path.display(), index + 1);
                }
            }
            for read_back in ["to_bytes()", ".secret()", ".secret_key()"] {
                assert!(!line.contains(read_back), "{}:{}: `{read_back}`: {line}", path.display(), index + 1);
            }
            let trimmed = line.trim_start();
            let public_field = trimmed.starts_with("pub ")
                && !trimmed.contains("fn ")
                && !trimmed.starts_with("pub use")
                && !trimmed.starts_with("pub mod")
                && trimmed.contains(':');
            if public_field {
                let field_type = &trimmed[trimmed.find(':').unwrap() + 1..];
                for holder in ["Endpoint", "OrgEndpoint", "Endpoints"] {
                    assert!(
                        !named_types(field_type).iter().any(|named| named == holder),
                        "{}:{}: a public field holds `{holder}`: {line}",
                        path.display(),
                        index + 1
                    );
                }
            }
        }
    }
    assert!(!naming_functions.is_empty(), "the scan found bind_from_keychain's use of SecretKey");
}

/// The endpoint's items: the type that holds an iroh `Endpoint`, its
/// constructor, the plan it is bound for, and an inbound stream's send half.
const ENDPOINT_ITEMS: [&str; 4] = ["OrgEndpoint", "BindPlan", "InboundReply", "bind_from_keychain"];

/// Whether the attribute lines directly above line `index` gate it to test
/// builds.
fn gated_to_test_builds(lines: &[&str], index: usize) -> bool {
    lines[..index]
        .iter()
        .rev()
        .take_while(|above| above.trim_start().starts_with("#[") || above.trim_start().starts_with("///"))
        .any(|above| above.contains("#[cfg(feature = \"test-support\")]"))
}

// No type reachable from org-io's public API holds an iroh `Endpoint` or an
// `OrgEndpoint` (LLR-d5y7ks), and only `Endpoints` (LLR-d2pzzk) and the
// transport preflight's `check_transport` (LLR-h9tcat, which drops its
// endpoint once the check is made) bind one: in a production build, no `pub mod` or `pub use` exposes the
// endpoint module or one of its items, and no production public function
// outside that module returns one. Only test builds (`test-support`) reach
// them, as the transport's tests do. *Added 2026-10-14 (S4 review round 1,
// finding 1):* `iroh_secret_key_is_named_in_one_function_only` checked
// public fields only, while `transport` re-exported `OrgEndpoint` and its
// public constructor.
// verifies: LLR-d5y7ks
#[test]
fn the_endpoint_and_its_constructor_are_reachable_only_in_test_builds() {
    let mut exposures = vec![];
    let mut declarations = 0;
    let mut returners = 0;
    for (path, text) in sources() {
        let code = code_lines(&text);
        for (index, line) in code.iter().enumerate() {
            let trimmed = line.trim_start();
            if !(trimmed.starts_with("pub mod ") || trimmed.starts_with("pub use ") || trimmed.starts_with("pub type ")) {
                continue;
            }
            // The whole declaration, joined across lines up to its `;`.
            let mut declaration = String::new();
            for continuation in &code[index..] {
                declaration.push_str(continuation.trim());
                declaration.push(' ');
                if continuation.contains(';') {
                    break;
                }
            }
            let names = named_types(&declaration);
            let exposes = names.iter().any(|named| named == "endpoint" || ENDPOINT_ITEMS.contains(&named.as_str()));
            if exposes {
                declarations += 1;
                if !gated_to_test_builds(&code, index) {
                    exposures.push(format!("{}:{}: {}", path.display(), index + 1, declaration.trim()));
                }
            }
        }
        if path.ends_with("transport/endpoint.rs") {
            continue;
        }
        let lines: Vec<&str> = text.lines().collect();
        for signature in public_signatures(&text) {
            let returns_an_item = named_types(signature.returns()).iter().any(|named| ENDPOINT_ITEMS.contains(&named.as_str()));
            if !returns_an_item {
                continue;
            }
            returners += 1;
            if !signature.only_in_test_builds() && !enclosing_item_only_in_test_builds(&lines, signature.line - 1) {
                exposures.push(format!("{}:{}: {}", path.display(), signature.line, signature.signature));
            }
        }
    }
    assert!(declarations >= 1, "the scan found the transport's re-export of the endpoint ({declarations})");
    assert!(returners >= 1, "the scan found `endpoint_for_test`'s return ({returners})");
    assert!(exposures.is_empty(), "reachable from a production build:\n{}", exposures.join("\n"));
}

// The transport preflight reads no key and no seed from the environment
// (owner ruling 12, 2026-10-08): neither the preflight module nor its binary
// names the device-seed or device-key variables, `DeviceSeed` or
// `from_keychain_bytes`; no `ODS_` variable either one names is a key or a
// seed; and iroh's secret-key type is still named only in
// `bind_from_keychain` (`iroh_secret_key_is_named_in_one_function_only`).
// verifies: LLR-h9tcat, LLR-d5y7ks
#[test]
fn the_preflight_reads_no_key_or_seed_from_the_environment() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut variables = vec![];
    for file in ["preflight.rs", "bin/preflight.rs"] {
        let text = std::fs::read_to_string(root.join(file)).unwrap();
        for absent in ["ODS_DEVICE_SEED", "ODS_DEVICE_KEY", "DeviceSeed", "from_keychain_bytes", "SecretKey"] {
            assert!(!text.contains(absent), "{file} names `{absent}`");
        }
        for (offset, _) in text.match_indices("\"ODS_") {
            let name: String = text[offset + 1..].chars().take_while(|character| *character != '"').collect();
            assert!(!name.contains("KEY") && !name.contains("SEED"), "{file} names the variable {name}");
            variables.push(name);
        }
        assert!(text.contains("check_transport"), "{file} runs the transport check");
    }
    for expected in ["ODS_DATA_DIR", "ODS_TRANSPORT"] {
        assert!(variables.iter().any(|name| name == expected), "the binary reads {expected}: {variables:?}");
    }
}

// org-io deletes nothing itself (REQ-cz753a): no file of its source removes
// a file or directory, forgets an Organisation, or drops a store's records or
// Personas; and org-node's discard is reached only from `publish.rs`.
// *Amended 2026-10-13 (owner ruling of that day, background discard):* the
// clause that the two public discards take `&mut self` is replaced by
// `no_public_method_discards_a_provisional_update`.
// *Reconciled with S3b-io part 1, 2026-10-14:* S4's store files (task T9)
// remove a file in two places, both exempted by name: `write_store` removes
// its own temporary file after a failed step, and `remove_store` removes a
// forgotten Persona's store file, which `LocalStore::store` calls only for
// org-node's `StoreChange::Forgotten` — org-node's verified removal,
// executed by org-io (LLR-amv4mb). The scan reads S4's `code_lines`
// (trailing comments dropped too) in place of S3b-io's comment-line filter.
// verifies: LLR-cc59ys
#[test]
fn org_io_removes_nothing_itself_and_discards_only_in_publish() {
    let mut found = vec![];
    for (path, text) in sources() {
        let file = path.display().to_string();
        let code = code_lines(&text);
        for (index, line) in code.iter().enumerate() {
            let removes = ["remove_file", "remove_dir", "forget_organisation"].iter().any(|call| line.contains(call));
            let drops_records = ["orgs", "personas"].iter().any(|field| {
                [".retain(", ".remove(", ".clear("].iter().any(|call| line.contains(&format!("{field}{call}")))
            });
            let store_file_step = path.ends_with("store_file.rs")
                && matches!(enclosing_function(&code, index).as_deref(), Some("write_store" | "remove_store"));
            if (removes || drops_records) && !store_file_step {
                found.push(format!("{file}:{}: removes: {}", index + 1, line.trim()));
            }
            if line.contains("discard_provisional(") && !path.ends_with("publish.rs") {
                found.push(format!("{file}:{}: discards outside publish.rs: {}", index + 1, line.trim()));
            }
        }
    }
    assert!(found.is_empty(), "org-io deletes outside org-node's verified paths:\n{}", found.join("\n"));
}

// The app reaches the store files only through the handle's operations
// (REQ-cz753a): no public function of a production build writes, renames or
// removes a file, or calls the store file functions that do. Added
// 2026-10-15 (S4 review round 2, finding-2): `store_file::write_store` and
// `store_file::remove_store` were public, and the scan above exempts them by
// name without asking who can call them.
// verifies: LLR-cc59ys
#[test]
fn no_public_function_writes_or_removes_a_file() {
    let file_calls = ["remove_file", "remove_dir", "fs::write", "rename(", "OpenOptions", "File::create", "write_store(", "remove_store("];
    let mut found = vec![];
    let mut scanned = 0;
    for (path, text) in sources() {
        if path.ends_with("test_support.rs") {
            continue;
        }
        let lines: Vec<&str> = text.lines().collect();
        for (name, body) in functions(&text) {
            let (declared_at, declaration) = &body[0];
            let trimmed = declaration.trim_start();
            if !(trimmed.starts_with("pub fn ") || trimmed.starts_with("pub async fn ")) {
                continue;
            }
            if gated_to_test_builds(&lines, declared_at - 1) || enclosing_item_only_in_test_builds(&lines, declared_at - 1) {
                continue;
            }
            scanned += 1;
            for (line_number, line) in &body {
                let code = without_strings(line);
                if let Some(call) = file_calls.iter().find(|call| code.contains(*call)) {
                    found.push(format!("{}:{line_number}: public `{name}` calls `{call}`", path.display()));
                }
            }
        }
    }
    assert!(scanned >= 10, "the scan found org-io's public functions ({scanned})");
    assert!(found.is_empty(), "a public function writes or removes a file:\n{}", found.join("\n"));
}

// The app discards nothing (owner ruling of 2026-10-13): no file of org-io's
// source declares a public discard, only a private or crate-visible one.
// *Reconciled with S3b-io part 1, 2026-10-14:* reads S4's `code_lines`.
// verifies: LLR-cc59ys
#[test]
fn no_public_method_discards_a_provisional_update() {
    let mut found = vec![];
    for (path, text) in sources() {
        let file = path.display().to_string();
        for (index, line) in code_lines(&text).iter().enumerate() {
            if ["pub fn discard", "pub async fn discard"].iter().any(|declared| line.contains(declared)) {
                found.push(format!("{file}:{}: a public discard: {}", index + 1, line.trim()));
            }
        }
    }
    assert!(found.is_empty(), "org-io offers a public discard:\n{}", found.join("\n"));
}

// iroh is one adapter's dependency (S4, ARCHITECTURE review finding-1): only
// the transport module, `src/transport/`, and the test-only support,
// `src/test_support.rs`, name `iroh::` in code; every other file reaches the
// transport through org-io's own types. Comments are not code.
// verifies: LLR-3vb4k9
#[test]
fn iroh_is_named_only_in_the_transport_module_and_test_support() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut naming = vec![];
    let mut outside = vec![];
    for (path, text) in sources() {
        let relative = path.strip_prefix(&src).unwrap().to_string_lossy().replace('\\', "/");
        for (index, line) in code_lines(&text).iter().enumerate() {
            if !line.contains("iroh::") {
                continue;
            }
            naming.push(relative.clone());
            if !(relative.starts_with("transport/") || relative == "test_support.rs") {
                outside.push(format!("{relative}:{}: {}", index + 1, line.trim()));
            }
        }
    }
    assert!(naming.iter().any(|file| file == "transport/endpoint.rs"), "the scan found the transport's iroh calls: {naming:?}");
    assert!(outside.is_empty(), "iroh named outside the transport module:\n{}", outside.join("\n"));
}

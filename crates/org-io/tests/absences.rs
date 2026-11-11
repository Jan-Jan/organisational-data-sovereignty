#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! What org-io must not contain (LLR-3zdw8v): read from its own source, in
//! the style of crates/org-node/tests/absences.rs.

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

// org-io receives a device private key, as a move-only `DeviceSecret`, from
// org-node's `create_persona`. The type is named only in its sanctioned
// places: in org-io, the keychain module `keychain.rs`; in org-node,
// `keys.rs`, which defines it, and `create_persona`, the one production
// public function that returns one (`unseal` and the signing operations take
// one). Any other file of org-io naming the type fails the scan.
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
    returners.sort();
    assert_eq!(
        returners,
        ["create", "propose_founding", "propose_member"],
        "org-node hands out a DeviceSecret from PersonStore::create, propose_founding and propose_member alone"
    );
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

// No other way to a whole `OrgService`: no
// public field of org-io holds one and no public function outside test
// builds returns one by value, so a refused connect cannot hand the app the
// service whose chain-judging operations org-io sequences.
// verifies: LLR-n2srhc
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
// handle's own surface is an allowlist: see
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
    // `OnChainWriter` is in `write.rs`.
    let submit = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/write.rs")).unwrap();
    // Whole signatures, joined across lines: a return type on a continuation
    // line is read as well. The receive step's `impl OrgIo` is in receiver.rs.
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

// The owner's rule is no private key, X25519 secrets included: the Organisation private key and the Member seed. Read
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
            // org-node's `ProvisionalChange` is the Admin's change (an invite identifier and a
            // Handle, or a MemberId) and holds no key, so neither list names it.
            let holds_secret = ["pub member_seed", "pub org_private_key"]
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
            for secret in ["MemberSeed", "OrgPrivateKey", "X25519Keypair"] {
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
/// return type (LLR-gh4rw3): containers and path segments, the handle and
/// its public-only values, ids, public keys, roots, epochs and typed errors.
/// Anything else is a way out until it is reviewed and added here.
const HANDLE_RETURN_ALLOWLIST: &[&str] = &[
    // Containers, primitives and path segments.
    "Result", "Option", "Vec", "Box", "Self", "String", "bool", "usize", "u64", "crate", "org_node", "connect",
    "view", "send", "std",
    // The handle and its connect refusal.
    "OrgIo", "ConnectFailed",
    // Ids, public keys, roots, epochs.
    "OrgId", "PersonaId", "MemberId", "DevicePublicKey", "PersonPublicKey", "OrgPublicKey", "RootHash", "Epoch",
    // The queries' public-only summaries.
    "PersonaSummary", "OrgSummary", "PersonSummary", "KeptAdmissionRequestSummary", "AcknowledgementReport",
    "RemovedDevice", "RemovedDevicesRefused",
    // LLR-gh4rw3's list: what the domain methods return.
    "ProposedFounding", "ProposedMember", "InviteSummary", "ProvisionalChangeSummary", "ProposedOrganisation",
    "HeldTarget", "WriteReport", "WriteRefused", "Verified", "WriteOutcome", "SendReport", "SendRefused", "Delivery",
    "RequestError", "Received",
    // Typed refusals: reasons as text, paths, Device public keys, the
    // keychain's, the store files', the transport's and org-node's typed
    // refusals.
    "OrgNodeError", "OpenError", "CreateError", "OperationError", "ExchangeError",
    "TransportError", "ReceiveError", "TerminalCause",
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

// The handle's read surface is an allowlist, not a name scan: every public
// method of `OrgIo` (wherever its `impl` is) and `ConnectFailed` outside test builds returns only allowlisted types, so
// no method hands out `OrgService` (whose `endpoint()` reaches `recv_one`
// and `send`), `OrgEndpoint`, `WireMessage`, `OrgRecord`, `PersonaRecord`,
// `ProvisionalUpdate` or anything else unreviewed; and no handle type
// dereferences to another type, which would offer that type's methods
// beyond the exact surface LLR-zhdc9f states.
// verifies: LLR-gh4rw3, LLR-zhdc9f
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
/// `ConnectFailed`), with its file.
fn handle_methods() -> Vec<(String, PublicSignature)> {
    let handle_items = ["impl OrgIo", "impl crate::OrgIo", "impl ConnectFailed"];
    let mut methods = vec![];
    for (path, text) in sources() {
        let file = path.display().to_string();
        let lines: Vec<&str> = text.lines().collect();
        for signature in public_signatures(&text) {
            let impl_line = (0..signature.line - 1).rev().find(|&earlier| lines[earlier].starts_with("impl")).map(|earlier| lines[earlier]);
            if !impl_line.is_some_and(|line| handle_items.iter().any(|handle_item| line.starts_with(handle_item))) {
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

/// The types LLR-gh4rw3 names, each defined in org-io.
const DOMAIN_RETURN_TYPES: &[&str] = &[
    "PersonSummary", "ProposedFounding", "ProposedMember", "InviteSummary", "ProvisionalChangeSummary",
    "ProposedOrganisation", "WriteReport", "WriteRefused", "Verified", "WriteOutcome", "SendReport", "SendRefused",
    "Delivery", "RequestError", "Received", "KeptAdmissionRequestSummary", "AcknowledgementReport",
    "RemovedDevice", "RemovedDevicesRefused",
];

/// What no returned type may hold, at any depth org-io defines.
const SECRET_HOLDERS: &[&str] = &[
    "OrgPrivateKey", "DeviceSecret", "SealedStore", "StoreKey", "SecretKey", "MemberSeed", "OutgoingMessage",
    "OrgService", "PersonaStore", "PersonStore", "WireMessage", "OrgEndpoint", "KeptAcknowledgements",
    "Acknowledgement", "HandleRng",
];

/// The body of the org-io type `name` (`pub struct`, `pub enum` or `pub
/// type`, with fields, a tuple or generic parameters), comments dropped, from
/// its declaration to its closing brace or parenthesis (or its `;`), with its
/// file; `None` when org-io defines no such type.
fn type_body(name: &str) -> Option<(String, String)> {
    for (path, text) in sources() {
        let code = code_lines(&text);
        for (index, line) in code.iter().enumerate() {
            let declares = ["pub struct ", "pub enum ", "pub type "].iter().any(|keyword| {
                line.trim_start().strip_prefix(keyword).and_then(|rest| rest.strip_prefix(name)).is_some_and(|after| {
                    after.chars().next().is_none_or(|next| !(next.is_alphanumeric() || next == '_'))
                })
            });
            if !declares {
                continue;
            }
            let mut body = vec![];
            for continuation in &code[index..] {
                body.push(*continuation);
                if continuation.trim_end().ends_with(';') && body.len() == 1 {
                    break;
                }
                let closes = continuation.starts_with('}') || continuation.starts_with(')');
                if closes || (body.len() == 1 && continuation.trim_end().ends_with("{}")) {
                    break;
                }
            }
            return Some((path.display().to_string(), body.join("\n")));
        }
    }
    None
}

/// The production public methods of `OrgIo` (wherever its `impl` is), by
/// name, de-duplicated, with their signatures.
fn org_io_methods() -> Vec<(String, String)> {
    let mut methods = vec![];
    for (_, text) in sources() {
        let lines: Vec<&str> = text.lines().collect();
        for signature in public_signatures(&text) {
            let impl_line = (0..signature.line - 1).rev().find(|&earlier| lines[earlier].starts_with("impl")).map(|earlier| lines[earlier]);
            if !impl_line.is_some_and(|line| line.starts_with("impl OrgIo") || line.starts_with("impl crate::OrgIo")) {
                continue;
            }
            if signature.only_in_test_builds() || enclosing_item_only_in_test_builds(&lines, signature.line - 1) {
                continue;
            }
            let after = &signature.signature[signature.signature.find("fn ").unwrap() + 3..];
            let name = after[..after.find(['(', '<']).unwrap()].to_string();
            methods.push((name, signature.signature.clone()));
        }
    }
    methods.sort();
    methods.dedup_by(|later, earlier| later.0 == earlier.0);
    methods
}

// The handle's production surface is exactly the domain's (LLR-zhdc9f):
// the methods and queries the design names, `open`, `connect` and
// `not_configured`; none of the removed ones; no
// parameter is a `PeerAddr`; no method sends, deletes or resends an
// acknowledgement.
// verifies: LLR-zhdc9f
#[test]
fn the_handles_production_methods_are_exactly_the_domain_surface() {
    let expected = [
        "acknowledgement_report", "admission_requests", "cancel_invite", "connect", "issue_invite",
        "next_received", "not_configured", "open", "organisation", "organisations", "person", "personas",
        "propose_admission", "propose_founding_organisation", "propose_member", "propose_person", "propose_revocation",
        "propose_updated_organisation", "read_invite", "removed_devices", "request_admission", "request_organisation", "send_organisation",
        "start_receiving", "verification", "verify_all", "verify_organisation", "write_organisation_to_chain",
    ];
    let methods = org_io_methods();
    let names: Vec<&str> = methods.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, expected, "the handle's production methods");
    for (name, signature) in &methods {
        assert!(!signature.contains("PeerAddr"), "{name} takes a peer address: {signature}");
        assert!(name == "acknowledgement_report" || !name.contains("acknowledg"), "{name}: no acknowledgement command");
    }
}

// None of the methods LLR-zhdc9f names as removed is among the handle's
// production public methods, the own-admin check (`is_own_admin`, owner
// ruling 6) among them.
// verifies: LLR-zhdc9f
#[test]
fn the_handle_offers_none_of_the_removed_methods() {
    let removed = [
        "is_own_admin", "create_persona", "found_organisation", "admit_member", "revoke_member", "expect_admission", "produce_reply",
        "import_reply", "admit_reply", "submit_commit_send", "settle_unknown", "reconcile", "reconcile_held",
    ];
    for (name, signature) in org_io_methods() {
        assert!(!removed.contains(&name.as_str()), "the handle offers the removed `{name}`: {signature}");
    }
}

// Every type the domain methods return is in the allowlist, every handle
// method's return type names allowlisted types only, and no allowlisted
// type org-io defines holds a secret — read through every org-io type its
// body names, to a fixed point (LLR-gh4rw3).
// verifies: LLR-gh4rw3
#[test]
fn every_returned_type_is_in_the_handle_return_allowlist_and_holds_no_secret() {
    let mut found = vec![];
    for name in DOMAIN_RETURN_TYPES {
        if !HANDLE_RETURN_ALLOWLIST.contains(name) {
            found.push(format!("{name}: not allowlisted"));
        }
    }
    for (file, signature) in handle_methods() {
        for named in named_types(signature.returns()) {
            if !HANDLE_RETURN_ALLOWLIST.contains(&named.as_str()) {
                found.push(format!("{file}:{}: returns {named}, not allowlisted", signature.line));
            }
        }
    }
    // The handle itself, and the refusal that hands it back, hold org-node's
    // service by design; their way out is the method surface scanned above.
    let handle_types = ["OrgIo", "ConnectFailed"];
    let mut to_read: Vec<String> =
        HANDLE_RETURN_ALLOWLIST.iter().filter(|name| !handle_types.contains(name)).map(|name| name.to_string()).collect();
    let mut read: Vec<String> = handle_types.iter().map(|name| name.to_string()).collect();
    let mut defined = 0;
    while let Some(name) = to_read.pop() {
        if read.contains(&name) {
            continue;
        }
        read.push(name.clone());
        let Some((file, body)) = type_body(&name) else { continue };
        defined += 1;
        for named in named_types(&body) {
            if SECRET_HOLDERS.contains(&named.as_str()) {
                found.push(format!("{file}: {name} holds {named}"));
            } else if !read.contains(&named) {
                to_read.push(named);
            }
        }
    }
    assert!(defined >= DOMAIN_RETURN_TYPES.len(), "the scan read org-io's returned types ({defined})");
    assert!(found.is_empty(), "returned types beyond the allowlist, or holding a secret:\n{}", found.join("\n"));
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

// An Organisation-information body carries the Organisation private key:
// no handle method returns a type holding
// an `OutgoingMessage` — an organisation-request answer travels in
// org-node's `Checked`, which no handle method returns. The holders
// are found in org-node's source; the scan is checked to find
// `Checked` among them.
// verifies: LLR-3zdw8v
#[test]
fn no_handle_method_returns_a_type_holding_an_outgoing_body() {
    let holders = outgoing_holders();
    assert!(holders.iter().any(|holder| holder == "Checked"), "the scan finds Checked (its organisation-request answer): {holders:?}");
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

// REQ-hw82ue: org-io is the only unit that names a keychain crate.
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

// The scan reads every line of every function of the keychain module that
// handles a secret (whose code names a `DeviceSecret`, the item's bytes, the
// seed or its `Zeroizing` buffer), string literals aside, so a copy made one
// line away from the read is seen.
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
        "for_signing_noting_refusal",
        "unseal",
    ] {
        assert!(scanned.iter().any(|name| name == expected), "the scan read `{expected}`: {scanned:?}");
    }
    assert!(violations.is_empty(), "{}: the keychain bytes are copied or formatted:\n{}", path.display(), violations.join("\n"));
}

// The former stores, one passphrase store `persona_store.bin` and the app's
// `outstanding_invites.json` beside it, are named nowhere: outstanding
// invites live only inside the sealed stores, and no code in `crates/org-io/src`
// names either file, comments included.
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

// iroh's secret-key type is named in one function only:
// `bind_from_keychain` builds it from the keychain item's lent seed and moves
// it into the endpoint builder. Nothing in `crates/org-io/src` reads it back
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
// them, as the transport's tests do.
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

// org-io removes, renames or
// deletes nothing itself but in the Persona deletion. File removals and
// renames are only in `store_file::write_store` (its temporary file, in
// `replace_file`, which the Person file's write shares),
// `mark_removed` and `remove_removed_file`; those two and the keychain's
// `delete_device_secret` are called only from retention.rs, the
// deletion's module (SDD-wyn7b6); no store
// record is retained, removed or cleared, and nothing is discarded.
// verifies: LLR-vvd8dn
#[test]
fn org_io_removes_nothing_itself_but_in_the_persona_deletion() {
    let mut found = vec![];
    for (path, text) in sources() {
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        let code = code_lines(&text);
        for (index, line) in code.iter().enumerate() {
            let line = without_strings(line);
            let in_function = enclosing_function(&code, index);
            let file_step = file == "store_file.rs"
                && matches!(in_function.as_deref(), Some("write_store" | "replace_file" | "mark_removed" | "remove_removed_file"));
            let removes = ["remove_file", "remove_dir", "fs::rename"].iter().any(|call| line.contains(call));
            if removes && !file_step {
                found.push(format!("{file}:{}: removes or renames: {}", index + 1, line.trim()));
            }
            let deletion_call = ["mark_removed(", "remove_removed_file(", "delete_device_secret("].iter().any(|call| line.contains(call));
            let declares = line.contains("fn mark_removed") || line.contains("fn remove_removed_file") || line.contains("fn delete_device_secret");
            let defining_file = file == "keychain.rs" && line.contains("delete_device_secret(");
            if deletion_call && !declares && !defining_file && file != "retention.rs" {
                found.push(format!("{file}:{}: deletes outside retention.rs: {}", index + 1, line.trim()));
            }
            let drops_records = [".retain(", ".remove(", ".clear("].iter().any(|call| line.contains(call))
                && ["record", "provisional", "members"].iter().any(|field| line.contains(field));
            if drops_records || line.contains("discard_") {
                found.push(format!("{file}:{}: drops records itself: {}", index + 1, line.trim()));
            }
        }
    }
    assert!(found.is_empty(), "org-io deletes outside the Persona deletion:\n{}", found.join("\n"));
}

// No public function of a
// production build writes, renames or removes a file, or calls the store
// file or keychain functions that do.
// verifies: LLR-vvd8dn
#[test]
fn no_public_function_writes_renames_or_removes_a_file() {
    let file_calls = [
        "remove_file", "remove_dir", "fs::write", "rename(", "OpenOptions", "File::create", "write_store(", "mark_removed(",
        "remove_removed_file(", "delete_device_secret(",
    ];
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
    assert!(scanned >= 20, "the scan found org-io's public functions ({scanned})");
    assert!(found.is_empty(), "a public function writes, renames or removes a file:\n{}", found.join("\n"));
}

// No public function of
// org-io discards or forgets a provisional change or organisation.
// verifies: LLR-vvd8dn
#[test]
fn no_public_method_discards_a_provisional_change_or_organisation() {
    let mut found = vec![];
    for (path, text) in sources() {
        for (index, line) in code_lines(&text).iter().enumerate() {
            let trimmed = line.trim_start();
            let Some(after) = trimmed.strip_prefix("pub fn ").or_else(|| trimmed.strip_prefix("pub async fn ")) else { continue };
            let name = &after[..after.find(['(', '<']).unwrap_or(after.len())];
            if ["discard", "forget", "drop_provisional", "cancel_provisional", "drop_change"].iter().any(|word| name.contains(word)) {
                found.push(format!("{}:{}: a public discard: {}", path.display(), index + 1, line.trim()));
            }
        }
    }
    assert!(found.is_empty(), "org-io offers a public discard:\n{}", found.join("\n"));
}

// iroh is one adapter's dependency: only
// the transport module, `src/transport/`, and the test-only support,
// `src/test_support.rs`, name `iroh::` in code; every other file reaches the
// transport through org-io's own types. Comments are not code.
// LLR-4k43dm states this over the addresses kept per Device key.
// verifies: LLR-4k43dm
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

// The handle's one random source (SDD-3qwjs2): no file of `crates/org-io/src` but `rng.rs` (the
// handle's `OsRng`) and `test_support.rs` (the test seams) names a generator. Code lines only.
// verifies: LLR-4qd5fx
#[test]
fn no_file_outside_rng_and_test_support_names_a_generator() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let generators = ["OsRng", "thread_rng", "StdRng", "SmallRng", "from_entropy"];
    let mut found = vec![];
    let mut named_in_rng = false;
    for (path, text) in sources() {
        let relative = path.strip_prefix(&src).unwrap().to_string_lossy().replace('\\', "/");
        for (index, line) in code_lines(&text).iter().enumerate() {
            for generator in generators.iter().filter(|generator| line.contains(**generator)) {
                match relative.as_str() {
                    "rng.rs" => named_in_rng = true,
                    "test_support.rs" => {}
                    _ => found.push(format!("{relative}:{}: `{generator}`: {}", index + 1, line.trim())),
                }
            }
        }
    }
    assert!(named_in_rng, "the scan found the handle's OsRng in rng.rs");
    assert!(found.is_empty(), "a generator named outside rng.rs and test_support.rs:\n{}", found.join("\n"));
}

// No caller supplies or sees the random source: no `pub fn` of `crates/org-io/src` (signatures joined
// across lines, as LLR-u2pk5y's scan reads them) names `RngCore`, `CryptoRng` (which
// `CryptoRngCore` contains) or `HandleRng`; `test_support.rs` is excepted.
// verifies: LLR-ctm86b
#[test]
fn no_public_method_takes_or_returns_a_random_source() {
    let mut found = vec![];
    let mut scanned = 0;
    for (path, text) in sources() {
        if path.ends_with("test_support.rs") {
            continue;
        }
        for signature in public_signatures(&text) {
            scanned += 1;
            for name in ["RngCore", "CryptoRng", "HandleRng"] {
                if signature.signature.contains(name) {
                    found.push(format!("{}:{}: names `{name}`: {}", path.display(), signature.line, signature.signature));
                }
            }
        }
    }
    assert!(scanned >= 10, "the scan found org-io's public functions ({scanned})");
    assert!(found.is_empty(), "a public function takes or returns a random source:\n{}", found.join("\n"));
    let lib = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs")).unwrap();
    for line in code_lines(&lib) {
        let field = line.trim_start();
        assert!(!(field.starts_with("pub ") && field.contains("HandleRng")), "a public field holds the source: {field}");
    }
}

// org-io names the Person only through org-node's types (owner ruling 2026-10-15: no org-io →
// person edge): no table of its manifest names a `person` crate — as a key, as a
// `[dependencies.person]` table of any kind, or renamed through `package = "person"` — and no
// code line of `crates/org-io/src` has a `use person` or `person::` path.
// verifies: LLR-b97usg
#[test]
fn org_io_names_no_person_crate() {
    let manifest = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    for line in manifest.lines().map(str::trim) {
        let unspaced = line.replace(' ', "");
        let names_the_crate = unspaced.starts_with("person=")
            || unspaced.starts_with("person.")
            || (unspaced.starts_with('[') && unspaced.ends_with("dependencies.person]"))
            || unspaced.contains("package=\"person\"");
        assert!(!names_the_crate, "the manifest names the person crate: {line}");
    }
    let mut scanned = 0;
    for (path, text) in sources() {
        scanned += 1;
        for (index, line) in code_lines(&text).iter().enumerate() {
            let names_the_crate = line.contains("use person")
                || line.match_indices("person::").any(|(at, _)| line[..at].chars().next_back().is_none_or(|before| !(before.is_alphanumeric() || before == '_')));
            assert!(!names_the_crate, "{}:{}: names the person crate: {}", path.display(), index + 1, line.trim());
        }
    }
    assert!(scanned > 5, "the scan read org-io's sources ({scanned})");
}

// Adoption has one way in (SDD-t4z8sz): org-node's `verify` is called in `crates/org-io/src` only from
// `verify.rs`; no receive, write or send path adopts. Code lines only.
// verifies: LLR-3ame3g
#[test]
fn org_nodes_verification_is_called_only_from_verify_rs() {
    let mut callers = vec![];
    for (path, text) in sources() {
        for (index, line) in code_lines(&text).iter().enumerate() {
            if line.contains(".verify(") {
                callers.push(format!("{}:{}", path.file_name().unwrap().to_string_lossy(), index + 1));
            }
        }
    }
    assert!(callers.iter().any(|caller| caller.starts_with("verify.rs:")), "the scan found verify.rs's call: {callers:?}");
    assert!(callers.iter().all(|caller| caller.starts_with("verify.rs:")), "org-node's verification called elsewhere: {callers:?}");
}

/// org-node's reads that hand out a value holding a key field: `OrgRecord`
/// (the Organisation private key), `PersonaRecord` (the member seed),
/// `PersonStoreContent` (the Person seed) and `StoreData` (all of them).
const KEY_CARRYING_READS: [&str; 4] = ["list_orgs(", "list_personas(", ".data()", "store_data("];

// org-io's production code reads org-node's stores only through the key-free
// views (`persona_view`, `persona_records`, `PersonStore::view` and their
// siblings): no line of `crates/org-io/src` outside test-support code calls a read
// that hands out a key-carrying value. Code lines only, string literals
// aside; `test_support.rs`, and any function or item gated on
// `test-support`, is exempt.
// verifies: LLR-jqb5z7
#[test]
fn org_io_reads_org_node_stores_only_through_key_free_views() {
    let mut reads = vec![];
    let mut scanned = 0;
    for (path, text) in sources() {
        if path.ends_with("test_support.rs") {
            continue;
        }
        scanned += 1;
        let lines: Vec<&str> = text.lines().collect();
        let code = code_lines(&text);
        for (index, line) in code.iter().enumerate() {
            let line = without_strings(line);
            if !KEY_CARRYING_READS.iter().any(|read| line.contains(read)) {
                continue;
            }
            let declaration = (0..=index).rev().find(|&earlier| {
                let trimmed = code[earlier].trim_start();
                ["fn ", "pub fn ", "async fn ", "pub async fn ", "pub(crate) fn ", "pub(crate) async fn "]
                    .iter()
                    .any(|start| trimmed.starts_with(start))
            });
            let in_test_builds = declaration.is_some_and(|declaration| gated_to_test_builds(&lines, declaration))
                || enclosing_item_only_in_test_builds(&lines, index);
            if !in_test_builds {
                reads.push(format!("{}:{}: {}", path.display(), index + 1, line.trim()));
            }
        }
    }
    assert!(scanned > 5, "the scan read org-io's sources ({scanned})");
    assert!(reads.is_empty(), "org-io reads org-node through a key-carrying value:\n{}", reads.join("\n"));
}

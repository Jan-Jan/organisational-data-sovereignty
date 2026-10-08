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
        for judging in ["fn commit_", "fn reconcile", "fn prepare_", "fn apply_", "fn discard_provisional"] {
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
    // line is read as well (review round 1, finding 8).
    for signature in public_signatures(&lib) {
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
/// org-node's outcome values, which carry public data only (Envelopes,
/// member snapshots, absence proofs, signed acknowledgements). Containers
/// and path segments are listed too. Anything else is a way out until it is
/// reviewed and added here.
const HANDLE_RETURN_ALLOWLIST: &[&str] = &[
    // Containers, primitives and path segments.
    "Result", "Option", "Vec", "Box", "Self", "String", "bool", "usize", "u64", "crate", "org_node", "connect",
    "signatory", "view", "std",
    // The handle and its public-only views and summaries.
    "OrgIo", "NodeBuilders", "NodeView", "PersonaSummary", "OrgSummary", "BuiltUpdate", "ConnectFailed",
    // Ids, public keys, roots, epochs.
    "OrgId", "PersonaId", "MemberId", "DevicePublicKey", "PersonPublicKey", "OrgPublicKey", "RootHash", "Epoch",
    // Answers and errors.
    "AdminStatus", "OwnAdminError", "OrgNodeError", "CommitOutcome", "ReceiveOutcome", "SelfDeleteOutcome",
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
// 3, finding 1): every public method of `OrgIo`, `NodeBuilders`, `NodeView`
// and `ConnectFailed` outside test builds returns only allowlisted types, so
// no method hands out `OrgService` (whose `endpoint()` reaches `recv_one`
// and `send`), `OrgEndpoint`, `WireMessage`, `OrgRecord`, `PersonaRecord`,
// `ProvisionalUpdate` or anything else unreviewed; and no handle type
// dereferences to another type, which would offer that type's methods.
// verifies: LLR-3zdw8v, LLR-7pj5af, LLR-9r2bxd
#[test]
fn the_handle_returns_only_allowlisted_public_types() {
    let handle_items = ["impl OrgIo", "impl NodeBuilders", "impl<'handle> NodeBuilders", "impl NodeView", "impl<'handle> NodeView", "impl ConnectFailed"];
    let mut ways_out = vec![];
    let mut methods_seen = 0;
    for (path, text) in sources() {
        let file = path.display().to_string();
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            let derefs = line.starts_with("impl") && (line.contains("Deref for") || line.contains("DerefMut for"));
            if derefs && ["OrgIo", "NodeBuilders", "NodeView"].iter().any(|holder| line.contains(holder)) {
                ways_out.push(format!("{file}:{}: a handle type dereferences: {line}", index + 1));
            }
        }
        for signature in public_signatures(&text) {
            let item = (0..signature.line - 1).rev().find(|&earlier| lines[earlier].starts_with("impl")).map(|earlier| lines[earlier]);
            if !item.is_some_and(|line| handle_items.iter().any(|handle_item| line.starts_with(handle_item))) {
                continue;
            }
            if signature.only_in_test_builds() || enclosing_item_only_in_test_builds(&lines, signature.line - 1) {
                continue;
            }
            methods_seen += 1;
            for named in named_types(signature.returns()) {
                if !HANDLE_RETURN_ALLOWLIST.contains(&named.as_str()) {
                    ways_out.push(format!("{file}:{}: returns {named}, not allowlisted: {}", signature.line, signature.signature));
                }
            }
        }
    }
    assert!(methods_seen > 10, "the scan found the handle's methods ({methods_seen})");
    assert!(ways_out.is_empty(), "the handle's surface beyond the allowlist:\n{}", ways_out.join("\n"));
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

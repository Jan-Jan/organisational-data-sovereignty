#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(
    clippy::await_holding_lock,
    reason = "`fresh_keychain`'s lock serialises the tests over keyring's process-global credential builder for their whole run; each test has its own runtime"
)]
//! The handle's random source (SDD-3qwjs2): one
//! cryptographic generator `OrgIo` owns, behind its own lock; every org-node draw an `OrgIo`
//! method makes and the store file's temporary suffix come from it (LLR-4qd5fx), a test build
//! may replace it (LLR-5acgfg), and a failing entropy source refuses org-io's own draw
//! (LLR-94nn6q).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use org_node::service::OrgService;
use org_io::store_file::StoreFileStep;
use org_io::test_support::FakeChain;
use org_io::{HeldTarget, OrgIo, StoreFailure, WriteRefused};
use rand::rngs::StdRng;
use rand::{CryptoRng, RngCore, SeedableRng};

mod common;
use common::handles::{file_of, fresh_keychain, handle, names_in, store_dir};

/// A source that hands out what `inner` draws and records the length of every request.
struct Recording {
    inner: StdRng,
    requests: Arc<Mutex<Vec<usize>>>,
}

impl RngCore for Recording {
    fn next_u32(&mut self) -> u32 {
        self.requests.lock().unwrap().push(4);
        self.inner.next_u32()
    }
    fn next_u64(&mut self) -> u64 {
        self.requests.lock().unwrap().push(8);
        self.inner.next_u64()
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.requests.lock().unwrap().push(dest.len());
        self.inner.fill_bytes(dest);
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl CryptoRng for Recording {}

fn sources() -> Vec<(PathBuf, String)> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                out.push(path);
            }
        }
    }
    let mut paths = vec![];
    walk(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut paths);
    paths.into_iter().map(|path| {
        let text = std::fs::read_to_string(&path).unwrap();
        (path, text)
    }).collect()
}

/// A source's lines with `//` comments (doc comments included) removed.
fn code_lines(text: &str) -> Vec<&str> {
    text.lines().map(|line| line.split("//").next().unwrap_or("")).collect()
}

/// Observed: a founding proposed through the handle, over a seeded source, draws from that
/// source, its last draw the store file's 8-byte temporary suffix; two handles over the same
/// seed propose the same founder key, root and Organisation key, so no draw came from anywhere
/// else. Read: every call `org-io/src` makes of an org-node operation that takes a random source
/// passes `&mut *rng`, and every `rng` bound in `org-io/src` is the handle's (`.rng.lock()`), so
/// no other source reaches org-node. `propose_founding_organisation` makes both draws.
// verifies: LLR-4qd5fx
#[tokio::test(flavor = "multi_thread")]
async fn every_org_node_draw_and_the_temporary_suffix_come_from_the_handles_source() {
    let (_guard, _keychain) = fresh_keychain();
    let recorded = |party: &str| {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recording = Recording { inner: StdRng::seed_from_u64(1), requests: requests.clone() };
        (handle("rng-source", party, &FakeChain::new()).with_rng_for_test(Box::new(recording)), requests)
    };
    let (io, requests) = recorded("a");
    let first = io.propose_founding_organisation("Alice", "Doe", "alice", "Acme").await.unwrap();
    let drawn = requests.lock().unwrap().clone();
    assert!(drawn.len() > 1, "org-node's draws and the suffix, all from the handle's source: {drawn:?}");
    assert_eq!(drawn.last(), Some(&8), "the last draw is the 8-byte temporary suffix: {drawn:?}");
    let (again, _) = recorded("b");
    let second = again.propose_founding_organisation("Alice", "Doe", "alice", "Acme").await.unwrap();
    assert_eq!(
        (first.founder, first.root, first.org_public_key),
        (second.founder, second.root, second.org_public_key),
        "the same seed, the same founding: every key was drawn from the handle's source"
    );

    let drawing = [
        "check_received(", "propose_founding(", "record_founding_proxy(", "propose_member(", "record_admission(",
        "record_revocation(", "propose_updated_organisation(", ".verify(", "build_invite(", "cancel_invite(",
        "acknowledgement_accepted(", "PersonStore::create(", "write_store(", "write_person_store(",
    ];
    let mut calls = 0;
    let mut other_sources = vec![];
    for (path, text) in sources() {
        if path.ends_with("test_support.rs") {
            continue;
        }
        let lines = code_lines(&text);
        for (index, line) in lines.iter().enumerate() {
            if line.contains("let mut rng") && !line.contains(".rng.lock()") {
                other_sources.push(format!("{}:{}: {}", path.display(), index + 1, line.trim()));
            }
            for operation in drawing.iter().filter(|operation| line.contains(**operation)) {
                if line.contains("fn ") {
                    continue;
                }
                calls += 1;
                let rest = &line[line.find(operation).unwrap() + operation.len()..];
                let arguments = if rest.trim().is_empty() { lines.get(index + 1).copied().unwrap_or("") } else { rest };
                let passes_the_handles = arguments.trim_start().starts_with("&mut *rng")
                    || arguments.contains("&self.rng")
                    || arguments.contains("&self.local.rng")
                    || arguments.contains("&local.rng");
                if !passes_the_handles {
                    other_sources.push(format!("{}:{}: `{operation}` without the handle's source: {}", path.display(), index + 1, line.trim()));
                }
            }
        }
    }
    assert!(calls >= 2, "the scan found org-io's drawing calls ({calls})");
    assert!(other_sources.is_empty(), "a draw from another source:\n{}", other_sources.join("\n"));
}

/// A test build replaces the source with a seeded one, so a key pair reproduces: a Person
/// proposed over the same seed has the same keys, over another seed or the operating system's
/// source other keys (seen through `propose_person`, not raw draws); no
/// production file replaces or seeds it, and `open` and `connect` take none.
// verifies: LLR-5acgfg
#[tokio::test]
async fn a_seeded_source_reproduces_a_draw_under_test_support() {
    let (_guard, _keychain) = fresh_keychain();
    let seeded = |seed: u64| {
        OrgIo::not_configured(OrgService::new(Vec::new()).unwrap()).with_rng_for_test(Box::new(StdRng::seed_from_u64(seed)))
    };
    let keys = |io: OrgIo| async move {
        let person = io.propose_person("Alice", "Doe").await.unwrap();
        (person.device_key, person.person_key)
    };
    let drawn = keys(seeded(7)).await;
    assert_eq!(drawn, keys(seeded(7)).await, "the same seed, the same key pairs");
    assert_ne!(drawn, keys(seeded(8)).await, "another seed, other key pairs");
    assert_ne!(drawn, keys(OrgIo::not_configured(OrgService::new(Vec::new()).unwrap())).await, "the operating system's source is not the seeded one");

    let lib = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs")).unwrap();
    assert!(lib.contains("#[cfg(feature = \"test-support\")]\npub mod test_support;"), "test_support is compiled only with test-support");
    for (path, text) in sources() {
        if path.ends_with("test_support.rs") {
            continue;
        }
        for line in code_lines(&text) {
            for replacing in ["fn with_rng_for_test", "seed_from_u64", "from_seed", "fn replaced_by"] {
                let declared_for_test = replacing == "fn replaced_by" && path.ends_with("rng.rs");
                // `SignatoryKey::from_seed` (custody.rs) builds the sr25519 signatory key from its
                // parsed `SeedBytes`; it is no generator.
                let signatory_key = replacing == "from_seed" && (line.contains("SignatoryKey") || line.contains("SeedBytes"));
                assert!(
                    declared_for_test || signatory_key || !line.contains(replacing),
                    "{} replaces or seeds the source: {}",
                    path.display(),
                    line.trim()
                );
            }
        }
    }
    let rng = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/rng.rs")).unwrap();
    let replaced_by = rng.find("fn replaced_by").unwrap();
    assert!(rng[..replaced_by].lines().rev().nth(1).unwrap().contains("#[cfg(feature = \"test-support\")]"), "replaced_by is test-only");
    for entry in ["pub fn open(", "pub async fn connect("] {
        for declared in lib.match_indices(entry) {
            let signature = &lib[declared.0..declared.0 + lib[declared.0..].find('{').unwrap()];
            assert!(!signature.contains("Rng"), "{entry} takes a random source: {signature}");
        }
    }
}

/// A source whose fallible draw fails, as the operating system's does when its entropy source is
/// gone, while its infallible draws still answer from `inner`: org-node's draws (`fill_bytes`)
/// go through, so the handle's operation reaches org-io's own draw (`try_fill_bytes`), the one
/// LLR-94nn6q states. (A source failing both ways panics inside org-node's draw first, before
/// any store change exists, as LLR-94nn6q states; org-io's own draw is then never reached.)
struct FallibleDrawFails {
    inner: StdRng,
}

impl RngCore for FallibleDrawFails {
    fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }
    fn next_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.inner.fill_bytes(dest);
    }
    fn try_fill_bytes(&mut self, _dest: &mut [u8]) -> Result<(), rand::Error> {
        Err(rand::Error::new("entropy source unavailable"))
    }
}

impl CryptoRng for FallibleDrawFails {}

/// A failing entropy source refuses org-io's own draw as `CreateTemporary` naming the random
/// source, writes nothing, leaves no temporary file and falls back to no other source; the
/// Persona is reloaded from its file. No code in `org-io/src` catches a panic. The failed write
/// is the founding write's proxy record, made and written by the handle.
// verifies: LLR-94nn6q
#[tokio::test(flavor = "multi_thread")]
async fn a_failing_entropy_source_refuses_the_store_write_and_writes_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let io = handle("rng-failing", "a", &FakeChain::new());
    let device_key = io.propose_founding_organisation("Alice", "Doe", "alice", "Acme").await.unwrap().founder;
    let dir = store_dir("rng-failing", "a");
    let file = file_of(&dir, &device_key);
    let (bytes, names) = (std::fs::read(&file).unwrap(), names_in(&dir));

    let io = io.with_rng_for_test(Box::new(FallibleDrawFails { inner: StdRng::seed_from_u64(3) }));
    let refused = io.write_organisation_to_chain(HeldTarget::Founding(device_key)).await;
    let Err(WriteRefused::Store { failure: StoreFailure::Reloaded { failed }, .. }) = refused else {
        panic!("not reported as a failed write, reloaded: {refused:?}")
    };
    assert_eq!(failed.step, StoreFileStep::CreateTemporary, "{failed}");
    assert!(failed.reason.contains("random source"), "the refusal names the random source: {failed}");
    assert_eq!(std::fs::read(&file).unwrap(), bytes, "nothing written");
    assert_eq!(names_in(&dir), names, "no temporary file");
    // The Organisation is recorded with the proxy: the handle lists the reloaded Persona bound
    // to none (read through the handle's query, not org-node's store).
    let held = io.personas().await.into_iter().find(|persona| persona.device_key == device_key).expect("the Persona is held").org_id;
    assert!(held.is_none(), "the Persona was reloaded from its file: no proxy recorded");
    for (path, text) in sources() {
        assert!(!text.contains("catch_unwind"), "{} catches a panic", path.display());
    }
}

/// A source failing both ways, as the operating system's does when its entropy source is gone:
/// its infallible draws panic, as rand_core's `OsRng` does on a `getrandom` error, and its
/// fallible draw refuses.
struct EveryDrawFails;

impl RngCore for EveryDrawFails {
    fn next_u32(&mut self) -> u32 {
        panic!("entropy source unavailable")
    }
    fn next_u64(&mut self) -> u64 {
        panic!("entropy source unavailable")
    }
    fn fill_bytes(&mut self, _dest: &mut [u8]) {
        panic!("entropy source unavailable")
    }
    fn try_fill_bytes(&mut self, _dest: &mut [u8]) -> Result<(), rand::Error> {
        Err(rand::Error::new("entropy source unavailable"))
    }
}

impl CryptoRng for EveryDrawFails {}

/// The draw org-node makes through `fill_bytes` (an Invite's identifier) panics inside the
/// org-node call when the entropy source fails, before any store change exists: the operation
/// ends in that panic, uncaught, and the store file and the directory are as they were.
// verifies: LLR-94nn6q
#[tokio::test(flavor = "multi_thread")]
async fn a_failing_draw_inside_org_node_panics_before_any_store_change() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "rng-org-node-draw";
    let io = handle(tag, "a", &FakeChain::new());
    let (org_id, founder) = common::handles::founded(&io, "alice").await;
    let dir = store_dir(tag, "a");
    let file = file_of(&dir, &founder);
    let (bytes, names) = (std::fs::read(&file).unwrap(), names_in(&dir));

    let io = Arc::new(io.with_rng_for_test(Box::new(EveryDrawFails)));
    let issuing = tokio::spawn({
        let io = io.clone();
        async move { io.issue_invite(org_id, "Bob").await.map(|_| ()) }
    });
    let ended = issuing.await;
    assert!(ended.as_ref().is_err_and(tokio::task::JoinError::is_panic), "the draw's panic, uncaught: {ended:?}");
    assert_eq!(std::fs::read(&file).unwrap(), bytes, "nothing written");
    assert_eq!(names_in(&dir), names, "no temporary file");
}

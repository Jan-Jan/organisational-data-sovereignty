//! Helpers shared by org-node's test targets. Not a test target: each target
//! that uses it declares `mod support;`.
//!
//! The story steps — `found_on_chain`, `setup`, `admit_b`, `remove_b` here
//! and the one-Persona steps of [`sweep`] — run the operations that leave
//! their state (`propose_founding`, `record_founding_proxy`,
//! `propose_member`, `build_invite`, `build_admission_request`,
//! `check_received`, `record_admission`, `record_revocation`,
//! `propose_updated_organisation`, `messages_for_current_epoch` and
//! `verify`), with `ChainSlots` or an `OrgState` standing in for org-io's
//! chain write and read, and keep every returned write on the party's disk.
//! A test that checks one of those operations calls it itself. No step
//! builds a store by hand: [`fixtures`] builds a store from parts, and
//! `Node::put` and `tamper` seal a state this software never writes.
//!
//! A [`Node`] plays org-io's part: its [`Keychain`] keeps each secret's
//! bytes, and its [`Disk`] keeps the latest sealed store per Persona, updated
//! from every returned write. What "reached the disk" is what the writes
//! carried: `Node::reopened` and `Node::reopened_service` unseal the
//! [`Disk`] with the [`Keychain`].
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// Each target compiles this module and calls a subset of it, so an item one
// target leaves unused is not dead. An item no target calls is: compile every
// target with this allow removed and delete what all of them report.
#![allow(dead_code)]

pub mod fixtures;
pub mod sweep;

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, LazyLock, Mutex};

use org_members::{DevicePublicKey, Handle, MemberId, MemberLeaf, Name, Surname};
use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::invitation::Invite;
use org_node::keys::{DeviceSecret, SigningKeypair, X25519Keypair};
use org_node::service::{Checked, OrgService, StoreChange};
use org_node::store::{MemberSnapshot, OrgRecord, PersonaRecord, PersonaStore, SealedStore, StoreData, StoreWrite};
use org_node::test_fixtures::{device_keypair, seal_for_test, store_for_test, ChainSlots, Trie};
use org_node::verify_org::{VerificationTime, Verified};
use org_node::wire::{encode_message, WireMessage};
use org_node::{ChainAccount, OrgPrivateKey, OrgState, PersonaId, SequenceNumber};
use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::Zeroizing;

/// org-io's keychain, played by the test: each Persona's device seed bytes,
/// by Device key. Clones share one map.
#[derive(Clone, Default)]
pub struct Keychain(Arc<Mutex<HashMap<[u8; 32], Vec<u8>>>>);

impl Keychain {
    /// Keep `secret`'s bytes, as org-io writes a new Persona's keychain item.
    pub fn keep(&self, secret: &DeviceSecret) {
        self.0.lock().unwrap().insert(*secret.device_key().as_bytes(), seed_bytes(secret));
    }

    /// The secret of `device_key`, read back as org-io reads a keychain item.
    pub fn secret(&self, device_key: &DevicePublicKey) -> Option<DeviceSecret> {
        let bytes = self.0.lock().unwrap().get(device_key.as_bytes()).cloned()?;
        Some(DeviceSecret::from_keychain_bytes(Zeroizing::new(bytes)).unwrap())
    }

    /// The seed bytes of `device_key`.
    pub fn seed(&self, device_key: &DevicePublicKey) -> [u8; 32] {
        self.0.lock().unwrap().get(device_key.as_bytes()).expect("a keychain item").as_slice().try_into().unwrap()
    }
}

/// The store files of one party: each Persona's latest sealed store, by its
/// Device key's bytes.
type StoreFiles = BTreeMap<[u8; 32], (DevicePublicKey, SealedStore)>;

/// org-io's store files, played by the test: the latest sealed store of
/// each Persona, by Device key. Clones share one map.
#[derive(Clone, Default)]
pub struct Disk(Arc<Mutex<StoreFiles>>);

impl Disk {
    pub fn write(&self, write: StoreWrite) {
        self.0.lock().unwrap().insert(*write.device_key.as_bytes(), (write.device_key, write.sealed));
    }

    pub fn apply(&self, change: StoreChange) {
        match change {
            StoreChange::Sealed(write) => self.write(write),
        }
    }

    /// Every sealed store, unsealed with `keychain`'s secrets.
    pub fn unseal(&self, keychain: &Keychain) -> Vec<PersonaStore> {
        self.0
            .lock()
            .unwrap()
            .values()
            .map(|(device_key, sealed)| {
                PersonaStore::unseal(sealed, *device_key, keychain.secret(device_key).expect("a keychain item")).unwrap()
            })
            .collect()
    }

    /// The sealed store of `device`.
    pub fn sealed_of(&self, device: &DevicePublicKey) -> SealedStore {
        self.0.lock().unwrap().get(device.as_bytes()).expect("the store on disk").1.clone()
    }

    /// Every sealed byte, in Device-key order (to prove nothing was written:
    /// each write draws a fresh nonce).
    pub fn bytes(&self) -> Vec<u8> {
        self.0.lock().unwrap().values().flat_map(|(_, sealed)| sealed.as_bytes().to_vec()).collect()
    }
}

/// One party's org-io storage in a test: its keychain and its store files.
#[derive(Clone, Default)]
pub struct Party {
    pub keychain: Keychain,
    pub disk: Disk,
}

/// Every party's storage, by test tag and party name: `open_store` starts
/// one afresh, `party` and `store_bytes` read it back.
static PARTIES: LazyLock<Mutex<HashMap<(String, String), Party>>> = LazyLock::new(Default::default);

/// A fresh, empty storage for `party` in test `tag`, replacing any earlier
/// one.
pub fn open_store(tag: &str, party: &str) -> Party {
    let fresh = Party::default();
    PARTIES.lock().unwrap().insert((tag.to_string(), party.to_string()), fresh.clone());
    fresh
}

/// `party`'s storage in test `tag`, as `open_store` registered it.
pub fn party(tag: &str, party: &str) -> Party {
    PARTIES.lock().unwrap().get(&(tag.to_string(), party.to_string())).cloned().expect("open_store first")
}

/// The raw bytes of a party's store files (to prove nothing was written).
pub fn store_bytes(tag: &str, party_name: &str) -> Vec<u8> {
    party(tag, party_name).disk.bytes()
}

/// Make `change` to the store of `persona_id` that reached `party`'s disk in
/// test `tag`, and seal it back there as org-io would hold it: a store this
/// software never writes. Reopen it with `Node::reopened`.
pub fn tamper(tag: &str, party_name: &str, persona_id: &PersonaId, change: impl FnOnce(&mut StoreData)) {
    let party = party(tag, party_name);
    let stores = party.disk.unseal(&party.keychain);
    let store = stores.iter().find(|store| &store.data().unwrap().persona.persona_id == persona_id).expect("the Persona's store");
    let mut store_data = store.data().unwrap().clone();
    change(&mut store_data);
    let secret = party.keychain.secret(&store.device_key()).expect("a keychain item");
    let rewritten = store_for_test(&secret, store_data);
    party.disk.write(seal_for_test(&rewritten, &mut OsRng));
}

/// An `OrgService` with the keychain and store files org-io keeps. Derefs
/// to the service for every read. It holds no chain view: `check_received`
/// takes no chain.
pub struct Node {
    pub svc: OrgService,
    pub keychain: Keychain,
    pub disk: Disk,
}

impl std::ops::Deref for Node {
    type Target = OrgService;
    fn deref(&self) -> &OrgService {
        &self.svc
    }
}

impl std::ops::DerefMut for Node {
    fn deref_mut(&mut self) -> &mut OrgService {
        &mut self.svc
    }
}

impl Node {
    /// A node with no Persona over `party`'s storage.
    pub fn new(party: Party) -> Self {
        Self::over(OrgService::new(vec![]).unwrap(), party)
    }

    /// A node over what reached `party`'s disk: its stores unsealed with its
    /// keychain.
    pub fn reopened(party: Party) -> Self {
        Self::over(OrgService::new(party.disk.unseal(&party.keychain)).unwrap(), party)
    }

    fn over(svc: OrgService, party: Party) -> Self {
        Self { svc, keychain: party.keychain, disk: party.disk }
    }

    /// A service over what reached this node's disk: every store unsealed
    /// with its keychain.
    pub fn reopened_service(&self) -> OrgService {
        OrgService::new(self.disk.unseal(&self.keychain)).unwrap()
    }

    /// The store of `device` as it reached this node's disk, unsealed.
    pub fn store_on_disk(&self, device: &DevicePublicKey) -> PersonaStore {
        self.disk.unseal(&self.keychain).into_iter().find(|store| &store.device_key() == device).expect("the store on disk")
    }

    /// A new Persona proposing to found "Acme" (`propose_founding`): the
    /// secret kept in the keychain, the sealed store on the disk and in the
    /// service. Its store holds the provisional founding, no proxy yet and
    /// no record; `found_on_chain` writes and adopts it.
    pub fn propose_and_keep_founding<R: rand_core::RngCore + rand_core::CryptoRng>(
        &mut self,
        rng: &mut R,
        handle: Handle,
        name: Name,
        surname: Surname,
    ) -> Result<PersonaId, OrgNodeError> {
        let (device, secret, change) =
            self.svc.propose_founding(rng, name.as_str(), surname.as_str(), handle.as_str(), "Acme".into())?;
        self.keep(&secret, change);
        Ok(persona_with_device(&self.svc, &device).persona_id)
    }

    /// A new Persona proposing to join the Organisation of `invite`
    /// (`propose_member`), its secret and store kept as
    /// `propose_and_keep_founding` keeps them.
    pub fn propose_member<R: rand_core::RngCore + rand_core::CryptoRng>(
        &mut self,
        rng: &mut R,
        invite: &Invite,
        name: &str,
        surname: &str,
        handles: &[&str],
    ) -> Result<PersonaId, OrgNodeError> {
        let (device, secret, change) = self.svc.propose_member(rng, invite, name, surname, handles)?;
        self.keep(&secret, change);
        Ok(persona_with_device(&self.svc, &device).persona_id)
    }

    /// A new Persona's secret in the keychain and its first write on the
    /// disk, as org-io keeps them.
    fn keep(&mut self, secret: &DeviceSecret, change: StoreChange) {
        self.keychain.keep(secret);
        self.disk.apply(change);
    }

    /// The Device key of `persona_id`'s store.
    pub fn device_of(&self, persona_id: &PersonaId) -> DevicePublicKey {
        persona_of(&self.svc, persona_id).device_key
    }

    /// `record_founding_proxy` of `device`'s founding for `org_id`, with
    /// `the_proxy`, its write kept on the disk.
    pub fn record_proxy(&mut self, device: &DevicePublicKey, org_id: OrgId) {
        let change = self.svc.record_founding_proxy(&mut OsRng, device, org_id, the_proxy()).unwrap();
        self.disk.apply(change);
    }

    /// `verify` of `device`'s store against `chain` at `story_time`, asking
    /// for no Device secret; its write, if any, kept on the disk.
    pub fn verify_and_keep(&mut self, device: &DevicePublicKey, chain: Option<OrgState>) -> Verified {
        let (verified, change) = self.svc.verify(&mut OsRng, device, chain, story_time(), org_node::AheadLimit(1200), never_asked).unwrap();
        if let Some(change) = change {
            self.disk.apply(change);
        }
        verified
    }

    /// `verify_and_keep` of `device`'s store against `chain`, which must
    /// adopt.
    pub fn adopt(&mut self, device: &DevicePublicKey, chain: OrgState) {
        let verified = self.verify_and_keep(device, Some(chain));
        assert!(matches!(verified, Verified::Adopted { .. }), "{verified:?}");
    }

    /// `propose_updated_organisation` from `device`'s held provisional
    /// changes, its write kept: the chain state a write of the proposal
    /// leaves.
    pub fn propose(&mut self, device: &DevicePublicKey) -> OrgState {
        let change = self.svc.propose_updated_organisation(&mut OsRng, device).unwrap();
        self.disk.apply(change);
        let own = held_store(&self.svc, device).provisional_organisations.last().expect("the own provisional organisation");
        OrgState { root_hash: own.root, org_pub_key: own.org_pub_key, epoch: own.epoch }
    }

    /// `propose` from `device`'s store, the proposal written to `chain` as
    /// org-io writes it, then adopted.
    pub fn propose_and_adopt(&mut self, device: &DevicePublicKey, chain: &ChainSlots, org_id: OrgId) {
        let proposal = self.propose(device);
        let written = chain.get(&org_id).expect("the Organisation's slot").epoch;
        chain.apply_update(org_id, proposal.root_hash, proposal.org_pub_key, written).unwrap();
        self.adopt(device, chain.get(&org_id).unwrap());
    }

    /// An Invite issued from `device`'s store for `invitee`, with no
    /// addresses; its write kept on the disk.
    pub fn invite(&mut self, device: &DevicePublicKey, invitee: &str) -> Invite {
        let (invite, change) = self.svc.build_invite(&mut OsRng, device, invitee.to_owned(), vec![]).unwrap();
        self.disk.apply(change);
        invite
    }

    /// The body of `device`'s message of the current epoch for `recipient`.
    pub fn message_for(&self, device: &DevicePublicKey, recipient: &DevicePublicKey) -> Vec<u8> {
        self.svc
            .messages_for_current_epoch(device)
            .unwrap()
            .into_iter()
            .find(|message| message.recipient == *recipient)
            .expect("a message of the epoch for that Device")
            .body
    }

    /// Seal `store_data` as an operation would, write it to the disk and hold it
    /// in the service in place of the store with its Device key.
    pub fn put(&mut self, store_data: StoreData) {
        let secret = self.keychain.secret(&store_data.persona.device_key).expect("a keychain item");
        let store = store_for_test(&secret, store_data);
        self.disk.write(seal_for_test(&store, &mut OsRng));
        self.svc.restore_store(store);
    }

    /// The store data of `persona_id`, read only.
    pub fn store_of(&self, persona_id: &PersonaId) -> &StoreData {
        held_store(&self.svc, &self.device_of(persona_id))
    }

    /// What org-io does on a receive: `check_received` in `device`'s store,
    /// the store change, if any, on the disk (the check may keep what it
    /// received).
    pub fn receive(&mut self, device: &DevicePublicKey, sender: DevicePublicKey, body: &[u8]) -> Result<Checked, OrgNodeError> {
        let (checked, change) = self.svc.check_received(&mut OsRng, device, sender, body)?;
        if let Some(change) = change {
            self.disk.apply(change);
        }
        Ok(checked)
    }
}

/// `check_received` of `body` from `sender` in the store of `persona_id`
/// on `node`, its store change returned and not kept on the disk.
pub fn check(node: &mut Node, persona_id: &PersonaId, sender: DevicePublicKey, body: &[u8]) -> Result<(Checked, Option<StoreChange>), OrgNodeError> {
    let device_key = node.device_of(persona_id);
    node.svc.check_received(&mut OsRng, &device_key, sender, body)
}

/// The admission request of `member_device`'s store in `member`, kept by
/// `inviter_device`'s store in `inviter`.
pub fn request_admission(inviter: &mut Node, inviter_device: &DevicePublicKey, member: &Node, member_device: &DevicePublicKey) {
    let request = member.build_admission_request(member_device, vec![]).unwrap();
    assert_eq!(request.recipient, *inviter_device, "the request goes to the Invite's issuer");
    let checked = inviter.receive(inviter_device, *member_device, &request.body).unwrap();
    assert!(matches!(checked, Checked::AdmissionRequest { .. }), "{checked:?}");
}

/// The Persona record of the store `device` names, cloned.
fn persona_with_device(svc: &OrgService, device: &DevicePublicKey) -> PersonaRecord {
    svc.list_personas().into_iter().find(|persona| &persona.device_key == device).expect("the new Persona").clone()
}

/// The store `device` names in `svc`.
pub fn held_store<'s>(svc: &'s OrgService, device: &DevicePublicKey) -> &'s StoreData {
    svc.store_data(device).expect("the store is held")
}

/// The encoded `StoreData` of the store `device` names: equal bytes, equal
/// store (the behind marker is not encoded).
pub fn encoded_store(svc: &OrgService, device: &DevicePublicKey) -> Vec<u8> {
    postcard::to_allocvec(held_store(svc, device)).unwrap()
}

/// The encoded `StoreData` of every store `svc` holds, in store order.
pub fn encoded_stores(svc: &OrgService) -> Vec<Vec<u8>> {
    svc.list_personas().iter().map(|persona| encoded_store(svc, &persona.device_key)).collect()
}

/// The `field` of an `InvalidField` refusal.
pub fn refused_field(error: OrgNodeError) -> &'static str {
    match error {
        OrgNodeError::InvalidField { field, .. } => field,
        other => panic!("expected InvalidField, got {other:?}"),
    }
}

/// The text of each production `pub fn` signature in org-node's `src/<file>`,
/// up to its body or `;`, its lines joined by spaces: functions gated on
/// `test-support`, by their own attribute or by their top-level `impl`
/// block's, are left out.
pub fn public_signatures(file: &str) -> Vec<String> {
    signatures_starting(file, "pub fn ")
}

/// As `public_signatures`, for each `pub(crate) fn`: `verify_org.rs`'s
/// functions are all crate-visible (merge review MR1, finding-6).
pub fn crate_signatures(file: &str) -> Vec<String> {
    signatures_starting(file, "pub(crate) fn ")
}

fn signatures_starting(file: &str, prefix: &str) -> Vec<String> {
    let text = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(file)).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    let mut signatures = vec![];
    let mut in_test_impl = false;
    for (index, line) in lines.iter().enumerate() {
        if line.starts_with("impl") && index > 0 && lines[index - 1].contains("feature = \"test-support\"") {
            in_test_impl = true;
        } else if *line == "}" {
            in_test_impl = false;
        }
        if in_test_impl || !line.trim_start().starts_with(prefix) {
            continue;
        }
        if lines[..index]
            .iter()
            .rev()
            .take_while(|above| above.trim_start().starts_with("#[") || above.trim_start().starts_with("///"))
            .any(|above| above.contains("test-support"))
        {
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
        signatures.push(signature.trim_end().to_string());
    }
    signatures
}

/// Each `.rs` file under org-node's `src/`, with its text.
pub fn src_rust_files() -> Vec<(std::path::PathBuf, String)> {
    fn collect(dir: &std::path::Path, found: &mut Vec<(std::path::PathBuf, String)>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(&path, found);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let text = std::fs::read_to_string(&path).unwrap();
                found.push((path, text));
            }
        }
    }
    let mut found = Vec::new();
    collect(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut found);
    found
}

/// A store change a test does not read, or an operation's optional one: the
/// service under test holds the store the change sealed, and the test reads
/// that store from the service, so the write org-io would make is not what
/// the test checks.
pub fn unread(_change: impl Into<Option<StoreChange>>) {}

/// A trait probe for `Clone`, resolved at compile time.
/// `CloneProbe::<Probed>::IS_CLONE` names the inherent constant when
/// `Probed: Clone`, which takes precedence over a trait's, and otherwise
/// falls back to the trait's default, `false`. This is the pattern of the
/// `impls` crate.
pub struct CloneProbe<Probed: ?Sized>(std::marker::PhantomData<Probed>);

pub trait NotCloneFallback {
    const IS_CLONE: bool = false;
}

impl<Probed: ?Sized> NotCloneFallback for CloneProbe<Probed> {}

// `Clone` implies `Sized`, so this impl takes no `?Sized`.
impl<Probed: Clone> CloneProbe<Probed> {
    pub const IS_CLONE: bool = true;
}

/// The Device-secret source of a verification whose test gives it none:
/// asked, it holds no secret.
pub fn none_held(_: &[DevicePublicKey]) -> Vec<DeviceSecret> {
    Vec::new()
}

/// The time every verification of these tests is made at.
pub fn story_time() -> VerificationTime {
    VerificationTime(1_790_000_000)
}

/// The Device-secret source of a verification that removes nobody: it must
/// never be called (LLR-qvp7mq, LLR-5tvn7g).
pub fn never_asked(_: &[DevicePublicKey]) -> Vec<DeviceSecret> {
    panic!("no Device secret is asked for by a verification that removes nobody")
}

/// The Organisation a founding of these tests is written as, where the test
/// picks the identifier itself.
pub fn the_org() -> OrgId {
    OrgId::new([0x0a; 20])
}

/// The pure proxy recorded for every founding of these tests.
pub fn the_proxy() -> ChainAccount {
    ChainAccount::new([0x5a; 32])
}

/// The seed bytes of `secret`, as org-io's keychain holds them.
pub fn seed_bytes(secret: &DeviceSecret) -> Vec<u8> {
    let mut bytes = vec![];
    secret
        .lend_to_keychain(|seed| {
            bytes.extend_from_slice(seed);
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap();
    bytes
}

/// A second `DeviceSecret` of `secret`'s seed, as org-io reopens its
/// keychain item.
pub fn secret_copy(secret: &DeviceSecret) -> DeviceSecret {
    DeviceSecret::from_keychain_bytes(Zeroizing::new(seed_bytes(secret))).unwrap()
}

/// Whether `needle` occurs in `haystack`.
pub fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| window == needle)
}

/// `bytes` in lower-case hex, two digits a byte.
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// `text` with its whitespace removed.
pub fn squash(text: &str) -> String {
    text.chars().filter(|symbol| !symbol.is_whitespace()).collect()
}

/// Every form in which `secret` could appear in debug output, with whitespace
/// removed: hex (lower and upper case) of its leading bytes; the leading
/// elements of a decimal list (`[208, 209, …`) and of a hex list (`[0xd0, …`,
/// `[d0, …`); and the whole of each `{:?}` rendering of the array
/// (`{:?}`, `{:#?}`, `{:x?}`, `{:X?}`, `{:#x?}`, `{:#X?}`).
fn secret_renderings(secret: &[u8; 32]) -> Vec<String> {
    let lower = hex(&secret[..4]);
    let head = &secret[..3];
    let list =
        |render: &dyn Fn(u8) -> String| head.iter().map(|byte| render(*byte)).collect::<Vec<_>>().join(",");
    vec![
        lower.to_uppercase(),
        lower,
        list(&|byte| byte.to_string()),
        list(&|byte| format!("{byte:#04x}")),
        list(&|byte| format!("{byte:#04X}")),
        format!("[{}", list(&|byte| format!("{byte:x}"))),
        format!("[{}", list(&|byte| format!("{byte:X}"))),
        squash(&format!("{secret:?}")),
        squash(&format!("{secret:#?}")),
        squash(&format!("{secret:x?}")),
        squash(&format!("{secret:X?}")),
        squash(&format!("{secret:#x?}")),
        squash(&format!("{secret:#X?}")),
    ]
}

/// Fails if `rendered` holds `secret` in any of its `secret_renderings`.
pub fn assert_not_rendered(rendered: &str, secret: &[u8; 32], what: &str) {
    let squashed = squash(rendered);
    for form in secret_renderings(secret) {
        assert!(!squashed.contains(&form), "{what}: debug output renders the secret as {form:?}");
    }
}

/// A Device key no store lists.
pub fn relay_device() -> DevicePublicKey {
    SigningKeypair::generate(&mut OsRng).device_key().unwrap()
}

/// The rogue relay's device seed — a third device, neither A's nor B's.
pub const ROGUE_SEED: [u8; 32] = [0x33u8; 32];

/// The Handle `text` parses to.
pub fn handle(text: &str) -> Handle {
    Handle::parse(text).unwrap()
}
/// The Name `text` parses to.
pub fn parsed_name(text: &str) -> Name {
    Name::parse(text).unwrap()
}
/// The Surname `text` parses to.
pub fn parsed_surname(text: &str) -> Surname {
    Surname::parse(text).unwrap()
}

/// A random source that fills every draw with one byte: the boundary fake of
/// randomness. Every seed drawn from it is the same 32 bytes.
pub struct ConstRng(pub u8);

impl rand_core::RngCore for ConstRng {
    fn next_u32(&mut self) -> u32 {
        u32::from_le_bytes([self.0; 4])
    }
    fn next_u64(&mut self) -> u64 {
        u64::from_le_bytes([self.0; 8])
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.fill(self.0);
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        dest.fill(self.0);
        Ok(())
    }
}

impl rand_core::CryptoRng for ConstRng {}

/// A random source that counts the bytes drawn from it and delegates to
/// `OsRng`, so a test can show how much an operation drew.
#[derive(Default)]
pub struct CountingRng {
    pub drawn: usize,
}

impl rand_core::RngCore for CountingRng {
    fn next_u32(&mut self) -> u32 {
        self.drawn += 4;
        OsRng.next_u32()
    }
    fn next_u64(&mut self) -> u64 {
        self.drawn += 8;
        OsRng.next_u64()
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.drawn += dest.len();
        OsRng.fill_bytes(dest)
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl rand_core::CryptoRng for CountingRng {}

/// A persona's record, cloned.
pub fn persona_of(svc: &OrgService, persona_id: &PersonaId) -> PersonaRecord {
    svc.list_personas()
        .into_iter()
        .find(|persona| &persona.persona_id == persona_id)
        .expect("persona not found")
        .clone()
}

/// The device keypair of a persona, from its secret in `node`'s keychain.
pub fn persona_device_keypair(node: &Node, persona_id: &PersonaId) -> SigningKeypair {
    device_keypair(device_seed_of(node, persona_id))
}

/// `svc`'s record of `org_id`, cloned.
pub fn record_of(svc: &OrgService, org_id: OrgId) -> OrgRecord {
    svc.list_orgs().into_iter().find(|record| record.org_id == org_id).expect("record").clone()
}

/// The persisted snapshot of a trie member.
pub fn snapshot_of(leaf: &MemberLeaf) -> MemberSnapshot {
    MemberSnapshot {
        id: *leaf.id(),
        handle: leaf.handle().clone(),
        name: leaf.name().clone(),
        surname: leaf.surname().clone(),
        member_key: *leaf.p2p_key(),
        device_keys: leaf.p2p_devices().to_vec(),
    }
}

/// Story 1: `persona_id`, whose founding `propose_and_keep_founding` proposed, founds the
/// Organisation: org-io's genesis write of the founding's root and key
/// (`chain`, at epoch 1), `record_proxy`, and `verify` against the slot,
/// which adopts the founding record. Every write is kept on the disk.
pub fn found_on_chain(node: &mut Node, chain: &ChainSlots, persona_id: &PersonaId) -> OrgId {
    let device = node.device_of(persona_id);
    let founding = node.persona_view(&device).unwrap().founding.expect("a provisional founding");
    let org_id = chain.apply_genesis(founding.root, founding.org_pub_key);
    node.record_proxy(&device, org_id);
    node.adopt(&device, chain.get(&org_id).unwrap());
    org_id
}

/// `message`, of the same kind, with its Envelope replaced.
pub fn with_envelope(message: &WireMessage, envelope: org_node::Envelope) -> WireMessage {
    match message {
        WireMessage::OrgInformation { record_snapshot, org_private_key, .. } => WireMessage::OrgInformation {
            envelope,
            record_snapshot: record_snapshot.clone(),
            org_private_key: org_private_key.clone(),
        },
        other => panic!("only Organisation information carries an Envelope, got {other:?}"),
    }
}

/// Organisation information carrying `envelope`, an empty snapshot and a
/// key no chain holds: for tests of the chain-free checks, which refuse the
/// Envelope before the snapshot or the key is looked at.
pub fn carrying(envelope: org_node::Envelope) -> WireMessage {
    WireMessage::OrgInformation { envelope, record_snapshot: Vec::new(), org_private_key: OrgPrivateKey::from([0u8; 32]) }
}

/// The Envelope of the Organisation information `message`; only that kind holds
/// one.
pub fn envelope_of(message: &WireMessage) -> &org_node::Envelope {
    match message {
        WireMessage::OrgInformation { envelope, .. } => envelope,
        other => panic!("Organisation information, got {other:?}"),
    }
}

/// The Organisation information `message`, with its Organisation private key
/// replaced.
pub fn with_key(message: &WireMessage, org_private_key: OrgPrivateKey) -> WireMessage {
    match message {
        WireMessage::OrgInformation { envelope, record_snapshot, .. } => WireMessage::OrgInformation {
            envelope: envelope.clone(),
            record_snapshot: record_snapshot.clone(),
            org_private_key,
        },
        other => panic!("only Organisation information carries a key, got {other:?}"),
    }
}

/// Stories 1–2: A proposes and founds the Organisation (epoch 1); B, unless
/// the test says not to, proposes to join it with A's Invite (`invite`),
/// and otherwise proposes a founding of its own, which binds it to nothing
/// and expects no admission.
pub struct Setup {
    pub chain: ChainSlots,
    pub node_a: Node,
    pub node_b: Node,
    pub org_id: OrgId,
    pub a_persona_id: PersonaId,
    pub b_persona_id: PersonaId,
    pub b_device_keypair: SigningKeypair,
    pub invite: Option<Invite>,
}

pub fn setup(tag: &str) -> Setup {
    setup_with(tag, true)
}

pub fn setup_with(tag: &str, prepare: bool) -> Setup {
    let chain = ChainSlots::new();
    let mut node_a = Node::new(open_store(tag, "a"));
    let mut node_b = Node::new(open_store(tag, "b"));

    let a_persona_id = node_a.propose_and_keep_founding(&mut OsRng, handle("admin"), parsed_name("Admin"), parsed_surname("User")).unwrap();
    let org_id = found_on_chain(&mut node_a, &chain, &a_persona_id);
    assert_eq!(chain.get(&org_id).unwrap().epoch, org_node::Epoch::new(1));

    let (b_persona_id, invite) = if prepare {
        let a_device = node_a.device_of(&a_persona_id);
        let invite = node_a.invite(&a_device, "Bob");
        (node_b.propose_member(&mut OsRng, &invite, "Bob", "Builder", &["bob"]).unwrap(), Some(invite))
    } else {
        (node_b.propose_and_keep_founding(&mut OsRng, handle("bob"), parsed_name("Bob"), parsed_surname("Builder")).unwrap(), None)
    };
    let b_device_keypair = persona_device_keypair(&node_b, &b_persona_id);
    Setup { chain, node_a, node_b, org_id, a_persona_id, b_persona_id, b_device_keypair, invite }
}

/// `node`'s `receive` of `message`, encoded, in the store of `persona_id`, from
/// the Device of `sender_seed`.
pub fn receive_message_from(
    node: &mut Node,
    persona_id: &PersonaId,
    sender_seed: [u8; 32],
    message: &WireMessage,
) -> Result<Checked, OrgNodeError> {
    let sender = device_keypair(sender_seed).device_key().unwrap();
    let device = node.device_of(persona_id);
    node.receive(&device, sender, &encode_message(message).unwrap())
}

/// The device seed of `persona_id` in `node`'s keychain, as bytes.
pub fn device_seed_of(node: &Node, persona_id: &PersonaId) -> [u8; 32] {
    node.keychain.seed(&persona_of(node, persona_id).device_key)
}

/// The calculated trie of `record`'s members.
pub fn trie_of(record: &OrgRecord) -> Trie {
    trie_of_members(&record.trie_members)
}

/// The trie leaf of `member`.
pub fn leaf_of(member: &MemberSnapshot) -> MemberLeaf {
    MemberLeaf::new(
        member.id,
        member.handle.clone(),
        member.member_key,
        member.name.clone(),
        member.surname.clone(),
        member.device_keys.clone(),
    )
    .unwrap()
}

/// The calculated trie of `members`.
pub fn trie_of_members(members: &[MemberSnapshot]) -> Trie {
    Trie::genesis(members.iter().map(leaf_of).collect()).unwrap()
}

/// The trie's members, in the trie's order, as a record holds them.
pub fn snapshots_of(trie: &Trie) -> Vec<MemberSnapshot> {
    trie.members().iter().map(snapshot_of).collect()
}

/// Stories 3–4: B's admission through the operations. B builds its
/// admission request for A's Invite and A keeps it (`check_received`); A
/// records the admission, proposes the next epoch, writes it to the chain
/// and adopts it; A's message of the current epoch reaches B, which keeps it
/// and adopts it against the same slot. Every write is kept on its party's
/// disk.
pub fn admit_b(mut story: Setup) -> Setup {
    let invite = story.invite.clone().expect("B proposed to join A's Organisation (setup with prepare)");
    let a_device = story.node_a.device_of(&story.a_persona_id);
    let b_device = story.node_b.device_of(&story.b_persona_id);
    request_admission(&mut story.node_a, &a_device, &story.node_b, &b_device);
    let change = story.node_a.svc.record_admission(&mut OsRng, &a_device, invite.invite_id, None).unwrap();
    story.node_a.disk.apply(change);
    story.node_a.propose_and_adopt(&a_device, &story.chain, story.org_id);

    let body = story.node_a.message_for(&a_device, &b_device);
    let checked = story.node_b.receive(&b_device, a_device, &body).unwrap();
    assert!(matches!(checked, Checked::ProvisionalOrganisationKept { .. }), "{checked:?}");
    story.node_b.adopt(&b_device, story.chain.get(&story.org_id).unwrap());
    story
}

/// Story 5 on A's side: A records B's revocation, proposes the next epoch,
/// writes it to the chain and adopts it (B's Device held as revoked); B's
/// store is left as it was. Returns the setup and B's MemberId.
pub fn remove_b(mut story: Setup) -> (Setup, MemberId) {
    let a_device = story.node_a.device_of(&story.a_persona_id);
    let b_device = story.node_b.device_of(&story.b_persona_id);
    let b_id = record_of(&story.node_a, story.org_id)
        .trie_members
        .iter()
        .find(|member| member.device_keys.contains(&b_device))
        .expect("B is listed")
        .id;
    let change = story.node_a.svc.record_revocation(&mut OsRng, &a_device, b_id).unwrap();
    story.node_a.disk.apply(change);
    story.node_a.propose_and_adopt(&a_device, &story.chain, story.org_id);
    (story, b_id)
}

/// Organisation information for the epoch after `record`, as a listed
/// Device would send it: the Member of seed `member_seed` added under a
/// drawn MemberId, the Change set from `record` to that, `record`'s members
/// as the record snapshot and a fresh Organisation private key. Returns the
/// message and the next record's root.
pub fn information_after(record: &OrgRecord, member_seed: u8) -> (WireMessage, org_members::RootHash) {
    use org_node::test_fixtures::{device_key, member_key};
    let mut id = [0u8; 32];
    OsRng.fill_bytes(&mut id);
    let leaf = MemberLeaf::new(
        MemberId::new(id),
        handle(&format!("joiner{member_seed}")),
        member_key(member_seed),
        parsed_name("Joe"),
        parsed_surname("Joiner"),
        vec![device_key(member_seed.wrapping_add(1))],
    )
    .unwrap();
    let (next, delta) = trie_of(record).add_member(leaf).unwrap().recalculate().unwrap();
    let envelope = org_node::Envelope::build(record.org_id, SequenceNumber::new(record.epoch.get() + 1), &delta).unwrap();
    let message = WireMessage::OrgInformation {
        envelope,
        record_snapshot: postcard::to_allocvec(&record.trie_members).unwrap(),
        org_private_key: X25519Keypair::generate(&mut OsRng).org_private_key(),
    };
    (message, next.root_hash().unwrap())
}

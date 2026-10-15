//! Story helpers shared by org-node's service-level test targets. Not a test
//! target: each target that uses it declares `mod support;`.
//!
//! The story operations — `found`, `prepare_to_join`, `joiner_of`, `admit`,
//! `revoke` — are the only place a test performs a story step, so a change to
//! the service API changes their bodies and not the tests that call them.
//!
//! Since ruling B (change `worktree-org-io-create`) org-node reads no chain:
//! its chain-judging operations take the state as a value. A test drives a
//! [`Node`]: an `OrgService` with the chain view (`CountingChain`) a test's
//! org-io reads through, whose methods run the sequence org-io runs — read
//! the state, then call the synchronous operation with it; on a receive
//! (`Node::receive_from`), `check_received`, a read only when it returns
//! `Checked::ReadChain`, then `commit_checked`.
//!
//! Since S4 task T4 no story helper uses an endpoint: a "send" is the body
//! the sender's commit built for the recipient (`body_for`, since T5 the
//! outcome's own body; T4's `body_to` rebuilt it), posted with the
//! sending Device's key to an [`Inbox`] (the stand-in for the address the
//! sender dialled), and a "receive" takes it from there into
//! `Node::receive_from`. Since S4 task T14 org-node has no endpoint and no
//! async operation: every helper is synchronous, and the dial tests are
//! org-io's.
//!
//! Since S4 task T3 org-node writes no file and keeps no device seed: every
//! change returns its `StoreWrite` (or `StoreChange`), and `create_persona`
//! returns the Persona's `DeviceSecret`. A [`Node`] plays org-io's part: its
//! [`Keychain`] keeps each secret's bytes, and its [`Disk`] keeps the latest
//! sealed store per Persona, updated from every returned write. What
//! "reached the disk" is what the returned writes carried: `reopen_store`
//! unseals the [`Disk`] with the [`Keychain`].
#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use org_members::{DevicePublicKey, Handle, MemberId, Name, Surname};
use org_node::chain::OrgState;
use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::invitation::InviteId;
use org_node::keys::{DeviceSecret, SigningKeypair};
use org_node::reconcile::Reconciled;
use org_node::revocation::VerifiedAcknowledgement;
use org_node::service::{Checked, CommitOutcome, OrgService, ReceiveOutcome, StoreChange};
use org_node::store::{
    ExpectedAdmission, MemberSnapshot, OrgRecord, PersonaRecord, PersonaStore, ProvisionalChange, ProvisionalUpdate,
    SealedStore, StoreData, StoreWrite,
};
use org_node::test_fixtures::{device_keypair, ChainSlots};
use org_node::{ChainAccount, Joiner, OrgPrivateKey, ProvisionalTarget};
use org_node::wire::{decode_message, encode_message, WireMessage};
use org_node::{Epoch, PersonaId, RootHash};
use rand::rngs::OsRng;
use zeroize::Zeroizing;

/// org-io's keychain, played by the test: each Persona's device seed bytes,
/// by Device key. Clones share one map.
#[derive(Clone, Default)]
pub struct Keychain(Arc<Mutex<HashMap<[u8; 32], Vec<u8>>>>);

impl Keychain {
    /// Keep `secret`'s bytes, as org-io writes a new Persona's keychain item.
    pub fn keep(&self, secret: &DeviceSecret) {
        let mut bytes = vec![];
        secret
            .lend_to_keychain(|seed| {
                bytes.extend_from_slice(seed);
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap();
        self.0.lock().unwrap().insert(*secret.device_key().as_bytes(), bytes);
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

    /// The secrets of `device_keys` this keychain holds: org-io's
    /// `device_secrets` closure.
    pub fn secrets(&self, device_keys: &[DevicePublicKey]) -> Vec<DeviceSecret> {
        device_keys.iter().filter_map(|key| self.secret(key)).collect()
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
            StoreChange::Forgotten { device_key } => {
                self.0.lock().unwrap().remove(device_key.as_bytes());
            }
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

    /// Every sealed store, by its Device key.
    pub fn sealed(&self) -> Vec<(DevicePublicKey, SealedStore)> {
        self.0.lock().unwrap().values().cloned().collect()
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
/// one afresh, `reopen_store` and `store_bytes` read it back.
static PARTIES: LazyLock<Mutex<HashMap<(String, String), Party>>> = LazyLock::new(Default::default);

/// A fresh, empty storage for `party` in test `tag`, replacing any earlier
/// one. (It took the store passphrase until S4 task T3; the store key comes
/// from each Persona's device secret.)
pub fn open_store(tag: &str, party: &str) -> Party {
    let fresh = Party::default();
    PARTIES.lock().unwrap().insert((tag.to_string(), party.to_string()), fresh.clone());
    fresh
}

/// `party`'s storage in test `tag`, as `open_store` registered it.
pub fn party(tag: &str, party: &str) -> Party {
    PARTIES.lock().unwrap().get(&(tag.to_string(), party.to_string())).cloned().expect("open_store first")
}

/// What reached `party`'s disk in test `tag`: every Persona store it holds,
/// unsealed, merged in one view of the shape the single store had before
/// S4 task T3.
pub fn reopen_store(tag: &str, party_name: &str) -> Reopened {
    let party = party(tag, party_name);
    Reopened::of(&party.disk.unseal(&party.keychain))
}

/// The raw bytes of a party's store files (to prove nothing was written).
pub fn store_bytes(tag: &str, party_name: &str) -> Vec<u8> {
    party(tag, party_name).disk.bytes()
}

/// A service over what reached `party`'s disk in test `tag`.
pub fn reopened_service(tag: &str, party_name: &str) -> OrgService {
    let party = party(tag, party_name);
    OrgService::new(party.disk.unseal(&party.keychain)).unwrap()
}

/// Make `change` to the store of `persona_id` that reached `party`'s disk in
/// test `tag`, and seal it back there as org-io would hold it: a store this
/// software never writes. Reopen it with `Node::reopened`.
pub fn tamper(tag: &str, party_name: &str, persona_id: &PersonaId, change: impl FnOnce(&mut StoreData)) {
    let party = party(tag, party_name);
    let stores = party.disk.unseal(&party.keychain);
    let store = stores.iter().find(|store| &store.data().persona.persona_id == persona_id).expect("the Persona's store");
    let mut data = store.data().clone();
    change(&mut data);
    let secret = party.keychain.secret(&store.device_key()).expect("a keychain item");
    let rewritten = org_node::test_fixtures::store_for_test(&secret, data);
    party.disk.write(org_node::test_fixtures::seal_for_test(&rewritten, &mut OsRng));
}

/// Every Persona store of one party, merged: the Personas, records,
/// provisional updates and expectations in store order.
pub struct Reopened {
    pub personas: Vec<PersonaRecord>,
    pub orgs: Vec<OrgRecord>,
    pub provisional_updates: Vec<ProvisionalUpdate>,
    pub expected_admissions: Vec<ExpectedAdmission>,
    /// The outstanding Invites (S4, task T6).
    pub outstanding_invites: Vec<(OrgId, InviteId)>,
}

impl Reopened {
    fn of(stores: &[PersonaStore]) -> Self {
        Self {
            personas: stores.iter().map(|store| store.data().persona.clone()).collect(),
            orgs: stores.iter().flat_map(|store| store.data().orgs.clone()).collect(),
            provisional_updates: stores.iter().flat_map(|store| store.data().provisional_updates.clone()).collect(),
            expected_admissions: stores.iter().flat_map(|store| store.data().expected_admissions.clone()).collect(),
            outstanding_invites: stores.iter().flat_map(|store| store.data().outstanding_invites.clone()).collect(),
        }
    }

    /// The merged view itself, so `reopen_store(..).data().orgs` reads as it
    /// did when one store held every Persona.
    pub fn data(&self) -> &Self {
        self
    }
}

/// An `OrgService` and the chain view its org-io reads through, with the
/// keychain and store files org-io keeps. Derefs to the service for every
/// read; the methods below shadow its changing operations with org-io's
/// part around them: the chain read before, the keychain and the disk after.
pub struct Node {
    pub svc: OrgService,
    pub view: CountingChain,
    pub keychain: Keychain,
    pub disk: Disk,
    /// The Device this node's sends come from, when a test names one
    /// (`sending_as`); else the first Persona bound to the Organisation.
    pub sends_as: Option<DevicePublicKey>,
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
    pub fn new(party: Party, view: impl Into<CountingChain>) -> Self {
        Self::over(OrgService::new(vec![]).unwrap(), party, view)
    }

    /// A node over what reached `party`'s disk: its stores unsealed with its
    /// keychain.
    pub fn reopened(party: Party, view: impl Into<CountingChain>) -> Self {
        Self::over(OrgService::new(party.disk.unseal(&party.keychain)).unwrap(), party, view)
    }

    fn over(svc: OrgService, party: Party, view: impl Into<CountingChain>) -> Self {
        Self { svc, view: view.into(), keychain: party.keychain, disk: party.disk, sends_as: None }
    }

    /// This node's sends come from `device`'s Device.
    pub fn sending_as(self, device: &SigningKeypair) -> Self {
        Self { sends_as: Some(device.device_key().unwrap()), ..self }
    }

    /// The Device a send of this node about `org_id` comes from: the one
    /// `sending_as` named, else the first Persona bound to `org_id`.
    pub fn sender_for(&self, org_id: OrgId) -> DevicePublicKey {
        self.sends_as.unwrap_or_else(|| {
            self.svc
                .list_personas()
                .into_iter()
                .find(|persona| persona.org_id == Some(org_id))
                .map(|persona| persona.device_key)
                .expect("a Persona bound to the Organisation")
        })
    }

    /// `OrgService::create_persona`: the secret kept in the keychain, the
    /// write on the disk.
    pub fn create_persona<R: rand_core::RngCore + rand_core::CryptoRng>(
        &mut self,
        rng: &mut R,
        handle: Handle,
        name: Name,
        surname: Surname,
    ) -> Result<PersonaId, OrgNodeError> {
        let (persona_id, secret, write) = self.svc.create_persona(rng, handle, name, surname)?;
        self.keychain.keep(&secret);
        self.disk.write(write);
        Ok(persona_id)
    }

    pub fn create_organisation<R: rand_core::RngCore + rand_core::CryptoRng>(
        &mut self,
        rng: &mut R,
        persona_id: &PersonaId,
    ) -> Result<ProvisionalUpdate, OrgNodeError> {
        let (update, write) = self.svc.create_organisation(rng, persona_id)?;
        self.disk.write(write);
        Ok(update)
    }

    pub fn discard_provisional<R: rand_core::RngCore + rand_core::CryptoRng>(
        &mut self,
        rng: &mut R,
        target: ProvisionalTarget,
        resulting_root: RootHash,
        org_pub_key: org_node::OrgPublicKey,
    ) -> Result<(), OrgNodeError> {
        let write = self.svc.discard_provisional(rng, target, resulting_root, org_pub_key)?;
        self.disk.write(write);
        Ok(())
    }

    pub fn admit_member<R: rand_core::RngCore + rand_core::CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
        joiner: &Joiner,
    ) -> Result<ProvisionalUpdate, OrgNodeError> {
        let (update, write) = self.svc.admit_member(rng, org_id, joiner)?;
        self.disk.write(write);
        Ok(update)
    }

    pub fn revoke_member<R: rand_core::RngCore + rand_core::CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
        member_id: MemberId,
    ) -> Result<ProvisionalUpdate, OrgNodeError> {
        let (update, write) = self.svc.revoke_member(rng, org_id, member_id)?;
        self.disk.write(write);
        Ok(update)
    }

    /// `OrgService::expect_admission` for the Persona a joining node would
    /// reply with: its first unbound Persona, else its first. Since S4 task
    /// T3 an expectation is one Persona's (LLR-rdda86); a test with several
    /// unbound Personas names one with `expect_admission_for`.
    pub fn expect_admission<R: rand_core::RngCore + rand_core::CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
    ) -> Result<(), OrgNodeError> {
        let personas = self.svc.list_personas();
        let joining = personas.iter().find(|persona| persona.org_id.is_none()).or(personas.first());
        let persona_id = joining.map(|persona| persona.persona_id.clone()).expect("a Persona");
        self.expect_admission_for(rng, &persona_id, org_id)
    }

    /// `OrgService::expect_admission` for `persona_id`.
    pub fn expect_admission_for<R: rand_core::RngCore + rand_core::CryptoRng>(
        &mut self,
        rng: &mut R,
        persona_id: &PersonaId,
        org_id: OrgId,
    ) -> Result<(), OrgNodeError> {
        let write = self.svc.expect_admission(rng, persona_id, org_id)?;
        self.disk.write(write);
        Ok(())
    }

    /// Read the state of `org_id`, then `OrgService::commit_genesis` with it.
    pub fn commit_genesis<R: rand_core::RngCore + rand_core::CryptoRng>(
        &mut self,
        rng: &mut R,
        persona_id: &PersonaId,
        org_id: OrgId,
        proxy_account: ChainAccount,
    ) -> Result<ReceiveOutcome, OrgNodeError> {
        let state = self.view.read(org_id)?;
        let (outcome, write) = self.svc.commit_genesis(rng, persona_id, org_id, proxy_account, state)?;
        self.disk.write(write);
        Ok(outcome)
    }

    /// Read the state of `org_id` back after the write, then
    /// `OrgService::commit_update` with it, the secrets from the keychain.
    pub fn commit_update<R: rand_core::RngCore + rand_core::CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
    ) -> Result<CommitOutcome, OrgNodeError> {
        let state = self.view.read(org_id)?;
        let keychain = self.keychain.clone();
        let (outcome, change) = self.svc.commit_update(rng, org_id, state, |keys| keychain.secrets(keys))?;
        self.disk.apply(change);
        Ok(outcome)
    }

    /// Read the state of `org_id`, then `OrgService::reconcile` with it.
    pub fn reconcile<R: rand_core::RngCore + rand_core::CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
    ) -> Result<Reconciled, OrgNodeError> {
        let state = self.view.read(org_id)?;
        let keychain = self.keychain.clone();
        let (reconciled, change) = self.svc.reconcile(rng, org_id, state, |keys| keychain.secrets(keys))?;
        if let Some(change) = change {
            self.disk.apply(change);
        }
        Ok(reconciled)
    }

    /// What org-io does on a receive (S4 task T4): `check_received(sender,
    /// body)`; for a message to read, the state of the Organisation it
    /// names, then `commit_checked` with it, the secrets from the keychain,
    /// the store change on the disk. An acknowledgement is decided in the
    /// check, with no read.
    pub fn receive_from<R: rand_core::RngCore + rand_core::CryptoRng>(
        &mut self,
        rng: &mut R,
        sender: DevicePublicKey,
        body: &[u8],
    ) -> Result<Received, OrgNodeError> {
        match self.svc.check_received(sender, body)? {
            Checked::Acknowledged(verified) => Ok(Received::Acknowledged(verified)),
            Checked::ReadChain(checked) => {
                let state = self.view.read(checked.org_id())?;
                let keychain = self.keychain.clone();
                let (outcome, change) = self.svc.commit_checked(rng, checked, state, |keys| keychain.secrets(keys))?;
                self.disk.apply(change);
                Ok(Received::Committed(outcome))
            }
        }
    }

    /// `receive_from` the body posted to `inbox`.
    pub fn receive_posted(&mut self, inbox: &Inbox) -> Result<Received, OrgNodeError> {
        let (sender, body) = inbox.take();
        self.receive_from(&mut OsRng, sender, &body)
    }

    /// The store data of `persona_id`, read only.
    pub fn store_of(&self, persona_id: &PersonaId) -> &StoreData {
        self.svc.store_data(persona_id).expect("the Persona's store")
    }
}

/// What `Node::receive_from` decided: a committed message, or an
/// acknowledgement decided in the check.
// One value per received message, moved once to the test: the
// acknowledgement's size costs nothing worth a box.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Received {
    Committed(ReceiveOutcome),
    Acknowledged(VerifiedAcknowledgement),
}

impl Received {
    /// The committed outcome, for a test that delivered a message to commit;
    /// an acknowledgement there is a failed test.
    pub fn committed(self) -> ReceiveOutcome {
        match self {
            Self::Committed(outcome) => outcome,
            Self::Acknowledged(verified) => panic!("decided as an acknowledgement: {verified:?}"),
        }
    }

    /// The acknowledgement the check decided; a commit there is a failed
    /// test.
    pub fn acknowledged(self) -> VerifiedAcknowledgement {
        match self {
            Self::Acknowledged(verified) => verified,
            Self::Committed(outcome) => panic!("committed, not an acknowledgement: {outcome:?}"),
        }
    }
}

/// Where a send is delivered (S4 task T4): the value-driven stand-in for the
/// address a sender dialled. A send posts the sending Device's key and the
/// body; the receiving side takes the first posted. Clones share one queue.
#[derive(Clone)]
pub struct Inbox {
    posts: std::sync::mpsc::Sender<Post>,
    queue: Arc<Mutex<std::sync::mpsc::Receiver<Post>>>,
}

/// One delivery: the sending Device's key and the body.
type Post = (DevicePublicKey, Vec<u8>);

impl Default for Inbox {
    fn default() -> Self {
        let (posts, queue) = std::sync::mpsc::channel();
        Self { posts, queue: Arc::new(Mutex::new(queue)) }
    }
}

impl Inbox {
    /// Deliver `body` as sent by `sender`'s Device.
    pub fn post(&self, sender: DevicePublicKey, body: Vec<u8>) {
        self.posts.send((sender, body)).unwrap();
    }

    /// The first body posted and its sender; a test takes it after the send
    /// it waits for, so none is a failed test.
    pub fn take(&self) -> (DevicePublicKey, Vec<u8>) {
        self.queue.lock().unwrap().try_recv().expect("nothing was delivered")
    }
}

/// A receive a test runs after the send it waits for: the value-driven
/// stand-in for a spawned receiving task. `run` runs the receive then, on
/// the body the send posted. (Until S4 task T14 a call site awaited it, as
/// it had awaited a spawned task.)
pub struct Deferred<T>(Box<dyn FnOnce() -> T + Send>);

impl<T> Deferred<T> {
    pub fn run(self) -> T {
        (self.0)()
    }
}

/// A Device that relays what it is handed: the value-driven stand-in for an
/// endpoint bound from that Device's secret.
#[derive(Clone, Copy, Debug)]
pub struct Relay {
    device_key: DevicePublicKey,
}

impl Relay {
    pub fn new(device: &SigningKeypair) -> Self {
        Self { device_key: device.device_key().unwrap() }
    }

    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self::new(&device_keypair(seed))
    }

    pub fn device_key(&self) -> DevicePublicKey {
        self.device_key
    }

    /// Post `msg`, encoded, to `to` as sent by this Device.
    pub fn send(&self, to: Inbox, msg: &WireMessage) -> Result<(), OrgNodeError> {
        to.post(self.device_key, encode_message(msg)?);
        Ok(())
    }
}

/// The rogue relay's device seed — a third device, neither A's nor B's.
pub const ROGUE_SEED: [u8; 32] = [0x33u8; 32];

pub fn h(s: &str) -> Handle {
    Handle::parse(s).unwrap()
}
pub fn nm(s: &str) -> Name {
    Name::parse(s).unwrap()
}
pub fn sn(s: &str) -> Surname {
    Surname::parse(s).unwrap()
}

/// One party's scratch directory, unique per target, test and process (no
/// store file lives there since S4 task T3).
pub fn store_dir(tag: &str, party: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "ods-{}-{tag}-{party}-{}",
        env!("CARGO_CRATE_NAME"),
        std::process::id()
    ))
}

/// A persona's record, cloned.
pub fn persona_of(svc: &OrgService, persona_id: &PersonaId) -> PersonaRecord {
    svc.list_personas()
        .into_iter()
        .find(|p| &p.persona_id == persona_id)
        .expect("persona not found")
        .clone()
}

/// The device keypair of a persona, from its secret in `node`'s keychain.
pub fn device_kp(node: &Node, persona_id: &PersonaId) -> SigningKeypair {
    device_keypair(device_seed_of(node, persona_id))
}

/// `svc`'s record of `org_id`, cloned.
pub fn rec_of(svc: &OrgService, org_id: OrgId) -> OrgRecord {
    svc.list_orgs().into_iter().find(|o| o.org_id == org_id).expect("record").clone()
}

/// The record of `org_id` as it reached the disk, cloned.
pub fn disk_rec_of(store: &Reopened, org_id: OrgId) -> OrgRecord {
    store.orgs.iter().find(|o| o.org_id == org_id).expect("record on disk").clone()
}

/// The MemberId of the member with `handle` in `rec`.
pub fn id_by_handle(rec: &OrgRecord, handle: &str) -> MemberId {
    rec.trie_members.iter().find(|m| m.handle.as_str() == handle).expect("member in the record").id
}

/// The receive `spawn_receive` returns: B's node back with its result.
pub type Receiving = Deferred<(Node, Result<ReceiveOutcome, OrgNodeError>)>;

/// B waits for a send: returns the inbox a sender delivers to and the
/// receive that takes the first body posted there into B's `receive_from`
/// (a message to commit), yielding `(svc_b, result)`.
pub fn spawn_receive(mut svc_b: Node) -> (Inbox, Receiving) {
    let inbox = Inbox::default();
    let posted = inbox.clone();
    let receive = Deferred(Box::new(move || {
        let result = svc_b.receive_posted(&posted).map(Received::committed);
        (svc_b, result)
    }));
    (inbox, receive)
}

/// A Device (from `seed`) that captures the first message sent to it: the
/// inbox a sender delivers to, and the capture yielding that Device as a
/// [`Relay`], the sender's Device and the message. A rogue relay R
/// (`ROGUE_SEED`) relays it on; a sink (random seed) only holds it.
pub fn spawn_recv_one(seed: [u8; 32]) -> (Inbox, Deferred<(Relay, DevicePublicKey, WireMessage)>) {
    let inbox = Inbox::default();
    let posted = inbox.clone();
    let capture = Deferred(Box::new(move || {
        let (sender, body) = posted.take();
        (Relay::from_seed(seed), sender, decode_message(&body).expect("a Wire message"))
    }));
    (inbox, capture)
}

/// The proxy account the app hands `commit_genesis` in these tests.
pub fn test_proxy() -> ChainAccount {
    ChainAccount::new([0x5a; 32])
}

/// The Organisation private key a provisional update holds.
pub fn private_key_of(update: &ProvisionalUpdate) -> OrgPrivateKey {
    match update.change_for_test() {
        ProvisionalChange::Genesis { org_private_key, .. } | ProvisionalChange::ChangeSet { org_private_key, .. } => {
            org_private_key.clone()
        }
    }
}

/// Story 1: `pid` founds an Organisation — the genesis update is built,
/// org-io's chain write is stood in for by the slots, and the update
/// committed against the state read back.
pub fn found(svc: &mut Node, chain: &ChainSlots, pid: &PersonaId) -> OrgId {
    let update = svc.create_organisation(&mut OsRng, pid).expect("build the genesis update");
    let org_id = chain.apply_genesis(update.resulting_root, update.org_pub_key);
    svc.commit_genesis(&mut OsRng, pid, org_id, test_proxy()).expect("commit genesis");
    org_id
}

/// Story 2, the joining node's half: whatever it must hold before an
/// admission to `org_id` reaches it.
pub fn prepare_to_join(svc_b: &mut Node, org_id: OrgId) {
    svc_b.expect_admission(&mut OsRng, org_id).expect("expect the admission");
}

/// The joiner `pid` of `svc` is admitted as: the Persona's details and its
/// two public keys, as the app reads them from org-node (LLR-437fvx).
pub fn joiner_of(svc: &OrgService, pid: &PersonaId) -> Joiner {
    let p = persona_of(svc, pid);
    let (member_key, device_key) = svc.persona_public_keys(pid).unwrap();
    Joiner { handle: p.handle, name: p.name, surname: p.surname, member_key, device_key }
}

/// Story 3 without the send: build the admission of `joiner`, write the
/// chain (mock), commit. Returns the joiner's new MemberId and the outcome.
pub fn commit_admission(
    svc: &mut Node,
    chain: &ChainSlots,
    org_id: OrgId,
    joiner: &Joiner,
) -> Result<(MemberId, CommitOutcome), OrgNodeError> {
    let update = svc.admit_member(&mut OsRng, org_id, joiner)?;
    chain.apply_update(org_id, update.resulting_root, update.org_pub_key, rec_of(svc, org_id).epoch)?;
    let outcome = svc.commit_update(&mut OsRng, org_id)?;
    let id = rec_of(svc, org_id)
        .trie_members
        .iter()
        .find(|m| m.member_key == joiner.member_key)
        .map(|m| m.id)
        .expect("the joiner is in the committed record");
    Ok((id, outcome))
}

/// Story 5 without the send: build the removal of `member_id`, write the
/// chain (mock), commit. Returns the outcome.
pub fn commit_removal(
    svc: &mut Node,
    chain: &ChainSlots,
    org_id: OrgId,
    member_id: MemberId,
) -> Result<CommitOutcome, OrgNodeError> {
    let update = svc.revoke_member(&mut OsRng, org_id, member_id)?;
    chain.apply_update(org_id, update.resulting_root, update.org_pub_key, rec_of(svc, org_id).epoch)?;
    svc.commit_update(&mut OsRng, org_id)}

/// The body the committed `outcome` holds for `recipient` (LLR-gdfqn9), or
/// `None`: the commit built none for it (S4 task T5; it replaces T4's
/// `body_to`, which rebuilt the body from the record at send time).
pub fn body_for(outcome: &CommitOutcome, recipient: DevicePublicKey) -> Option<Vec<u8>> {
    outcome.outgoing.iter().find(|message| message.recipient == recipient).map(|message| message.body.clone())
}

/// The Wire message `outcome` holds for `recipient`, decoded; fails the test
/// when the commit built none for it.
pub fn message_for(outcome: &CommitOutcome, recipient: DevicePublicKey) -> WireMessage {
    decode_message(&body_for(outcome, recipient).expect("the commit built a body for the recipient")).unwrap()
}

/// The revocation notices among `outcome`'s bodies, in order (S3a's
/// `revocations` field until S4 task T5).
pub fn notices_in(outcome: &CommitOutcome) -> Vec<org_node::revocation::RevocationNotice> {
    outcome
        .outgoing
        .iter()
        .filter_map(|message| match decode_message(&message.body).unwrap() {
            WireMessage::Revocation(notice) => Some(notice),
            _ => None,
        })
        .collect()
}

/// Post the body `outcome` holds for `recipient` to `to`, from the Device
/// `svc` sends as. A recipient the commit built no body for fails the test.
pub fn send(svc: &Node, outcome: &CommitOutcome, recipient: DevicePublicKey, to: &Inbox) -> Result<(), OrgNodeError> {
    let body = body_for(outcome, recipient).expect("the commit built a body for the recipient");
    to.post(svc.sender_for(outcome.org_id), body);
    Ok(())
}

/// The first Device of the Member with `handle` in `record`.
pub fn device_by_handle(record: &OrgRecord, handle: &str) -> DevicePublicKey {
    record.trie_members.iter().find(|m| m.handle.as_str() == handle).and_then(|m| m.device_keys.first().copied()).unwrap()
}

/// Story 3: admit `joiner` into `org_id`: build, write the chain (mock),
/// commit, send to `to`. Returns the joiner's new MemberId.
pub fn admit(
    svc: &mut Node,
    chain: &ChainSlots,
    org_id: OrgId,
    joiner: &Joiner,
    to: Inbox,
) -> Result<MemberId, OrgNodeError> {
    let (id, outcome) = commit_admission(svc, chain, org_id, joiner)?;
    send(svc, &outcome, joiner.device_key, &to)?;
    Ok(id)
}

/// The removed member's first Device in `svc`'s record of `org_id`.
fn first_device_of(svc: &Node, org_id: OrgId, member_id: MemberId) -> DevicePublicKey {
    rec_of(svc, org_id)
        .trie_members
        .iter()
        .find(|m| m.id == member_id)
        .and_then(|m| m.device_keys.first().copied())
        .expect("the removed member has a device")
}

/// Story 5: revoke `member_id`: build, write the chain (mock), commit, send
/// the committed revocation to the removed member's first device.
pub fn revoke(
    svc: &mut Node,
    chain: &ChainSlots,
    org_id: OrgId,
    member_id: MemberId,
    to: Inbox,
) -> Result<(), OrgNodeError> {
    let recipient = first_device_of(svc, org_id, member_id);
    revoke_and_tell(svc, chain, org_id, member_id, recipient, to)}

/// Story 5, told to `recipient`: build, write the chain (mock), commit, and
/// send the committed removal to `recipient` at `to`.
pub fn revoke_and_tell(
    svc: &mut Node,
    chain: &ChainSlots,
    org_id: OrgId,
    member_id: MemberId,
    recipient: DevicePublicKey,
    to: Inbox,
) -> Result<(), OrgNodeError> {
    let outcome = commit_removal(svc, chain, org_id, member_id)?;
    send(svc, &outcome, recipient, &to)
}

/// `msg`, of the same kind, with its Envelope replaced.
pub fn with_envelope(msg: &WireMessage, envelope: org_node::Envelope) -> WireMessage {
    match msg {
        WireMessage::OrgInformation { record_snapshot, org_private_key, .. } => WireMessage::OrgInformation {
            envelope,
            record_snapshot: record_snapshot.clone(),
            org_private_key: org_private_key.clone(),
        },
        WireMessage::Revocation(_) => panic!("a revocation carries no Envelope"),
        WireMessage::Acknowledgement(_) => panic!("an acknowledgement carries no Envelope"),
    }
}

/// Organisation information carrying `envelope`, an empty snapshot and a
/// key no chain holds: for tests of the chain-free checks, which refuse the
/// Envelope before the snapshot or the key is looked at. (Until S3 T4 those
/// tests sent the Envelope as a revocation, which holds a notice now.)
pub fn carrying(envelope: org_node::Envelope) -> WireMessage {
    WireMessage::OrgInformation { envelope, record_snapshot: Vec::new(), org_private_key: OrgPrivateKey::from([0u8; 32]) }
}

/// The Envelope of the Organisation information `msg`; only that kind holds
/// one (LLR-js9dsu).
pub fn envelope_of(msg: &WireMessage) -> &org_node::Envelope {
    match msg {
        WireMessage::OrgInformation { envelope, .. } => envelope,
        other => panic!("Organisation information, got {other:?}"),
    }
}

/// The Organisation information `msg`, with its record snapshot replaced.
pub fn with_snapshot(msg: &WireMessage, record_snapshot: Vec<u8>) -> WireMessage {
    match msg {
        WireMessage::OrgInformation { envelope, org_private_key, .. } => WireMessage::OrgInformation {
            envelope: envelope.clone(),
            record_snapshot,
            org_private_key: org_private_key.clone(),
        },
        other => panic!("only Organisation information carries a snapshot, got {other:?}"),
    }
}

/// The Organisation information `msg`, with its Organisation private key
/// replaced.
pub fn with_key(msg: &WireMessage, org_private_key: OrgPrivateKey) -> WireMessage {
    match msg {
        WireMessage::OrgInformation { envelope, record_snapshot, .. } => WireMessage::OrgInformation {
            envelope: envelope.clone(),
            record_snapshot: record_snapshot.clone(),
            org_private_key,
        },
        other => panic!("only Organisation information carries a key, got {other:?}"),
    }
}

/// Stories 1–2 over the service API: A creates a persona and the organisation
/// (epoch 1); B creates a persona, declares it expects the admission (unless
/// the test says not to), and A reads the joiner it admits B as. A sends
/// from its founding Persona's Device; no endpoint is bound (S4 task T4).
pub struct Setup {
    pub chain: ChainSlots,
    pub svc_a: Node,
    pub svc_b: Node,
    pub org_id: OrgId,
    pub pid_a: PersonaId,
    pub pid_b: PersonaId,
    pub b_device_kp: SigningKeypair,
    pub joiner_b: Joiner,
}

pub fn setup(tag: &str) -> Setup {
    setup_with(tag, true)}

pub fn setup_with(tag: &str, prepare: bool) -> Setup {
    setup_over_both(tag, prepare, CountingChain::over, CountingChain::over)}

/// `setup_with`, with A's org-io reading the chain through `a_chain(chain)`
/// and B's through `b_chain(chain)`.
pub fn setup_over_both(
    tag: &str,
    prepare: bool,
    a_chain: impl FnOnce(ChainSlots) -> CountingChain,
    b_chain: impl FnOnce(ChainSlots) -> CountingChain,
) -> Setup {
    let chain = ChainSlots::new();

    let mut svc_a = Node::new(open_store(tag, "a"), a_chain(chain.clone()));
    let mut svc_b = Node::new(open_store(tag, "b"), b_chain(chain.clone()));

    // Story 1: A creates persona + org.
    let pid_a = svc_a.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let org_id = found(&mut svc_a, &chain, &pid_a);
    assert_eq!(chain.get(&org_id).unwrap().epoch, Epoch::new(1));

    // Story 2: B creates persona and prepares to join A's organisation.
    let pid_b = svc_b.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("Builder")).unwrap();
    if prepare {
        prepare_to_join(&mut svc_b, org_id);
    }

    // The joiner A admits B as.
    let joiner_b = joiner_of(&svc_b, &pid_b);
    assert_eq!(joiner_b.handle.as_str(), "bob");

    let b_device_kp = device_kp(&svc_b, &pid_b);

    Setup { chain, svc_a, svc_b, org_id, pid_a, pid_b, b_device_kp, joiner_b }
}

/// Story 3+4 directly: A admits B, B receives from A's device and commits
/// epoch 2. Returns the setup with B's service carrying the committed record.
pub fn admit_b_directly(mut s: Setup) -> Setup {
    let (b_addr, b_task) = spawn_receive(s.svc_b);

    admit(&mut s.svc_a, &s.chain, s.org_id, &s.joiner_b, b_addr)
        .expect("admit_member(B) failed");
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, Epoch::new(2), "admitting B must bump to epoch 2");

    let (svc_b, outcome) = b_task.run();
    let outcome = outcome.expect("B's direct admission from A must verify");
    assert_eq!(outcome.epoch, Epoch::new(2));
    assert_eq!(svc_b.list_orgs().len(), 1);
    assert_eq!(svc_b.list_orgs()[0].epoch, Epoch::new(2));
    assert_eq!(svc_b.list_orgs()[0].trie_members.len(), 2, "A + B");
    s.svc_b = svc_b;
    s
}

/// A creates a further persona C and returns the joiner it is admitted as.
/// A's founding Persona stays the first bound to the Organisation, so A
/// still sends from its device.
pub fn joiner_for_c(svc_a: &mut Node) -> Joiner {
    let pid_c = svc_a.create_persona(&mut OsRng, h("carol"), nm("Carol"), sn("Coder")).unwrap();
    let jr = joiner_of(svc_a, &pid_c);
    assert_eq!(jr.handle.as_str(), "carol");
    jr
}

/// A's genuine admission of `joiner`, as sent to the joiner's Device, not
/// delivered to B.
pub fn captured_admission(s: &mut Setup, joiner: &Joiner) -> WireMessage {
    let (_, outcome) = commit_admission(&mut s.svc_a, &s.chain, s.org_id, joiner).expect("admit the joiner");
    message_for(&outcome, joiner.device_key)
}

/// A admits a further persona C; the admission is captured, not delivered
/// to B.
pub fn captured_admission_of_c(s: &mut Setup) -> WireMessage {
    let joiner_c = joiner_for_c(&mut s.svc_a);
    captured_admission(s, &joiner_c)}

/// Send `msg` to `addr` from a relay device that is neither A nor B. No
/// record lists it, so since the owner rulings of 2026-10-07 (LLR-2r2fha,
/// LLR-kzgjz8, LLR-3aysup) a held Organisation's receiver refuses what it
/// sends: use it for first admissions and refusal cases, `deliver_from`
/// otherwise.
pub fn deliver(to: Inbox, msg: &WireMessage) {
    deliver_from([0x5bu8; 32], to, msg);
}

/// Send `msg` to `to` from the Device of `sender_seed`.
pub fn deliver_from(sender_seed: [u8; 32], to: Inbox, msg: &WireMessage) {
    Relay::from_seed(sender_seed).send(to, msg).expect("an encodable message");
}

/// Send `body` to `to` from a relay device: bytes no `WireMessage` encodes
/// to.
pub fn deliver_raw(to: Inbox, body: &[u8]) {
    to.post(Relay::from_seed([0x5cu8; 32]).device_key(), body.to_vec());
}

/// The encoded member snapshots of `trie`, as a first admission carries them.
pub fn snapshot_bytes(trie: &org_members::trie::OrgTrie<org_members::hasher::Blake3Hasher>) -> Vec<u8> {
    let members: Vec<MemberSnapshot> = trie
        .members()
        .iter()
        .map(|m| MemberSnapshot {
            id: *m.id(),
            handle: m.handle().clone(),
            name: m.name().clone(),
            surname: m.surname().clone(),
            member_key: *m.p2p_key(),
            device_keys: m.p2p_devices().to_vec(),
        })
        .collect();
    postcard::to_allocvec(&members).unwrap()
}

/// The state source a test's org-io reads through, over shared `ChainSlots`:
/// it counts reads, can hide an Organisation (answer `None` for it) and can
/// fail every read.
#[derive(Clone)]
pub struct CountingChain {
    slots: ChainSlots,
    reads: Arc<AtomicUsize>,
    hidden: Arc<Mutex<Vec<OrgId>>>,
    failing: Arc<AtomicBool>,
}

impl From<ChainSlots> for CountingChain {
    fn from(slots: ChainSlots) -> Self {
        Self::over(slots)
    }
}

impl CountingChain {
    pub fn over(slots: ChainSlots) -> Self {
        Self { slots, reads: Default::default(), hidden: Default::default(), failing: Default::default() }
    }

    /// One read of `org`'s state: counted, then the failing, hidden or slot
    /// answer.
    pub fn read(&self, org: OrgId) -> Result<Option<OrgState>, OrgNodeError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        if self.failing.load(Ordering::SeqCst) {
            return Err(OrgNodeError::Chain("chain unreachable".into()));
        }
        if self.hidden.lock().unwrap().contains(&org) {
            return Ok(None);
        }
        Ok(self.slots.get(&org))
    }
    pub fn reads(&self) -> usize {
        self.reads.load(Ordering::SeqCst)
    }
    pub fn hide(&self, org: OrgId) {
        self.hidden.lock().unwrap().push(org);
    }
    pub fn fail_reads(&self) {
        self.failing.store(true, Ordering::SeqCst);
    }
}

/// `setup`, with B's org-io reading the chain through its own
/// `CountingChain`, returned (B's view).
pub fn setup_counted(tag: &str) -> (Setup, CountingChain) {
    let s = setup(tag);
    let counting = s.svc_b.view.clone();
    (s, counting)
}

/// `setup`, with A and B both reading the chain through one `CountingChain`:
/// its count is every chain read either node makes.
pub fn setup_counting(tag: &str) -> (Setup, CountingChain) {
    let shared: Arc<Mutex<Option<CountingChain>>> = Default::default();
    let counting_for = |shared: Arc<Mutex<Option<CountingChain>>>| {
        move |c: ChainSlots| -> CountingChain {
            let mut slot = shared.lock().unwrap();
            slot.get_or_insert_with(|| CountingChain::over(c)).clone()
        }
    };
    let s = setup_over_both(tag, true, counting_for(shared.clone()), counting_for(shared.clone()));
    let counting = shared.lock().unwrap().clone().expect("setup_over_both calls both closures");
    (s, counting)
}

/// Deliver `msg` to `svc`'s `receive_from` from a random relay Device no
/// record lists, and return its result (a message to commit). Since the
/// owner rulings of 2026-10-07 (LLR-2r2fha, LLR-kzgjz8, LLR-3aysup) such a
/// sender is refused for every message about a held Organisation: use this
/// for the refusal cases and first admissions, and `deliver_from_to_receive`
/// from a listed Device otherwise.
pub fn deliver_to_receive(svc: &mut Node, msg: WireMessage) -> Result<ReceiveOutcome, OrgNodeError> {
    deliver_from_to_receive(svc, rand::random(), msg)}

/// Deliver `msg` to `svc`'s `receive_from` from the Device of
/// `sender_seed`, and return its result (a message to commit).
pub fn deliver_from_to_receive(
    svc: &mut Node,
    sender_seed: [u8; 32],
    msg: WireMessage,
) -> Result<ReceiveOutcome, OrgNodeError> {
    receive_message_from(svc, sender_seed, &msg).map(Received::committed)
}

/// `svc`'s `receive_from` for `msg`, encoded, from the Device of
/// `sender_seed`.
pub fn receive_message_from(svc: &mut Node, sender_seed: [u8; 32], msg: &WireMessage) -> Result<Received, OrgNodeError> {
    let sender = Relay::from_seed(sender_seed).device_key();
    svc.receive_from(&mut OsRng, sender, &encode_message(msg).unwrap())
}

/// The device seed of `persona_id` in `node`'s keychain, as bytes: the
/// Device a test sends from when the message must come from that Persona's
/// Device.
pub fn device_seed_of(node: &Node, persona_id: &PersonaId) -> [u8; 32] {
    node.keychain.seed(&persona_of(node, persona_id).device_key)
}

/// A calculated trie of one Member that lists neither `member` nor `device`:
/// an absence proof from it verifies under its root, not under any chain's.
pub fn trie_without(member: MemberId, device: DevicePublicKey) -> org_node::test_fixtures::Trie {
    use org_node::test_fixtures::{device_key, member_key};
    let other = MemberId::new([0x5e; 32]);
    assert_ne!(other, member);
    assert_ne!(device_key(0x5f), device);
    let leaf = org_members::MemberLeaf::new(other, h("elsewhere"), member_key(0x5e), nm("Else"), sn("Where"), vec![device_key(0x5f)])
        .unwrap();
    org_node::test_fixtures::Trie::genesis(vec![leaf]).unwrap()
}

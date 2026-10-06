#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The Organisation public and private key. A created Organisation publishes
//! the X25519 public key of a secret generated for it alone, distinct from every key of its
//! genesis record, and that secret is kept in the creator's encrypted store
//! (REQ-ech45n); no other node's record holds it today (LLR-3fwykc).
//!
//! The gate: `cargo test -p org-node --features app,test-support --test organisation_key`

use std::path::PathBuf;
use std::time::Duration;

use org_members::{Handle, Name, OrgMembersError, PersonPublicKey, RootHash, Surname};
use org_node::chain::OrgState;
use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::keys::SigningKeypair;
use org_node::service::{ChainOps, MockChainOps, OrgService};
use org_node::transport::endpoint::OrgEndpoint;
use org_node::{ChainAccount, DeviceSeed, Epoch, MemberSeed, OrgPrivateKey, OrgPublicKey, OrgSecret, PersonaId};
use org_node::store::{OrgRecord, PersonaStatus, PersonaStore};
use rand::rngs::OsRng;
use rand::{CryptoRng, RngCore};

const PASSWORD: &str = "pw_org_key";

fn h(s: &str) -> Handle {
    Handle::parse(s).unwrap()
}
fn nm(s: &str) -> Name {
    Name::parse(s).unwrap()
}
fn sn(s: &str) -> Surname {
    Surname::parse(s).unwrap()
}

/// Fresh encrypted store under `temp_dir()`, unique per test and party.
fn open_store(tag: &str, party: &str) -> (PersonaStore, PathBuf) {
    let dir = std::env::temp_dir().join(format!(
        "ods-organisation-key-{tag}-{party}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("store.bin");
    (PersonaStore::open(path.clone(), PASSWORD).unwrap(), path)
}

/// The Organisation public key the chain holds for `org_id`, as bytes.
fn published_key(chain: &MockChainOps, org_id: &OrgId) -> [u8; 32] {
    *chain.get(org_id).unwrap().org_pub_key.as_bytes()
}

/// The Organisation public key a record holds, as bytes.
fn record_key(rec: &OrgRecord) -> [u8; 32] {
    *rec.org_pub_key.as_bytes()
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}

/// A random source that returns the same byte every time: makes every
/// secret a node draws identical, so the Organisation private key equals the
/// persona's member seed.
struct ConstRng(u8);

impl RngCore for ConstRng {
    fn next_u32(&mut self) -> u32 {
        u32::from_le_bytes([self.0; 4])
    }
    fn next_u64(&mut self) -> u64 {
        u64::from_le_bytes([self.0; 8])
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.fill(self.0);
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        dest.fill(self.0);
        Ok(())
    }
}

impl CryptoRng for ConstRng {}

// Normal case: the published key is the X25519 public key of the secret the
// creator holds, and no member or device of the genesis record holds it.
// verifies: REQ-ech45n, LLR-sj7cd5, LLR-3fwykc
#[tokio::test(flavor = "multi_thread")]
async fn a_created_organisation_publishes_a_fresh_key_no_genesis_key_equals() {
    let chain = MockChainOps::new();
    let (store, _) = open_store("fresh", "a");
    let mut svc = OrgService::new(store, Box::new(chain.clone()));
    let pid = svc.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let org_id = svc.create_organisation(&mut OsRng, &pid).await.unwrap();

    let published = published_key(&chain, &org_id);
    let rec = svc.list_orgs()[0].clone();
    assert_eq!(record_key(&rec), published, "the record holds the key it published");
    assert!(PersonPublicKey::parse(&published).is_ok(), "a valid X25519 public key");

    let secret = rec.org_private_key.expect("the creator holds the Organisation private key");
    assert_eq!(
        secret.x25519_keypair().public_bytes(),
        published,
        "the published key is the public key of the secret held"
    );
    for m in &rec.trie_members {
        assert_ne!(&published, m.member_key.as_bytes(), "a Member-as-a-group key of the genesis record");
        for d in &m.device_keys {
            assert_ne!(&published, d.as_bytes(), "a DevicePublicKey of the genesis record");
        }
    }
    let persona = &svc.list_personas()[0];
    assert_ne!(secret.expose_secret(), persona.member_seed.expose_secret(), "the Organisation private key is the member seed");
    assert_ne!(secret.expose_secret(), persona.device_seed.expose_secret(), "the Organisation private key is the device seed");
}

// Freshness: one persona founding two Organisations publishes two keys. A
// key derived from the persona's own secrets would be the same twice.
// verifies: REQ-ech45n
#[tokio::test(flavor = "multi_thread")]
async fn two_organisations_of_one_persona_publish_two_keys() {
    let chain = MockChainOps::new();
    let (store, _) = open_store("two-orgs", "a");
    let mut svc = OrgService::new(store, Box::new(chain.clone()));
    let pid = svc.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let org_1 = svc.create_organisation(&mut OsRng, &pid).await.unwrap();
    let org_2 = svc.create_organisation(&mut OsRng, &pid).await.unwrap();
    assert_ne!(published_key(&chain, &org_1), published_key(&chain, &org_2));
}

// The Organisation private key is in the store, under the passphrase, and
// never in the store file in clear. (Renamed 2026-10-05 by review round 1:
// "Organisation secret" is org-node's term for `org_secret`, not this key.)
// verifies: REQ-ech45n, LLR-3fwykc
#[tokio::test(flavor = "multi_thread")]
async fn the_organisation_private_key_is_kept_only_in_the_encrypted_store() {
    let chain = MockChainOps::new();
    let (store, path) = open_store("at-rest", "a");
    let mut svc = OrgService::new(store, Box::new(chain.clone()));
    let pid = svc.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let org_id = svc.create_organisation(&mut OsRng, &pid).await.unwrap();
    let secret = svc.list_orgs()[0].org_private_key.clone().unwrap();

    let reopened = PersonaStore::open(path.clone(), PASSWORD).unwrap();
    let rec = reopened.data().orgs.iter().find(|o| o.org_id == org_id).unwrap();
    assert_eq!(rec.org_private_key, Some(secret.clone()), "the store holds the Organisation private key");
    assert!(PersonaStore::open(path.clone(), "wrong").is_err(), "only under the passphrase");

    let bytes = std::fs::read(&path).unwrap();
    assert!(!contains(&bytes, secret.expose_secret()), "the Organisation private key is in the file in clear");
}

// The Organisation private key never reaches a log: the record's Debug output
// names the field and does not print the key. (Renamed 2026-10-05 by review
// round 1, as above. Since the merge of master `1feb608` the key is held in a
// redacted secret type, as the Organisation secret is (PR-hqwpg9, resolved
// there), so the assertion reads the key's bytes, not its `Debug`.)
// *Re-traced 2026-10-05 by review round 4 (finding-3).* It verified
// REQ-ech45n too. LLR-2dvhz8 is derived since review round 3, and REQ-ech45n
// says nothing of how a record renders.
// verifies: LLR-2dvhz8
#[tokio::test(flavor = "multi_thread")]
async fn the_organisation_private_key_is_not_in_the_record_debug_output() {
    let chain = MockChainOps::new();
    let (store, _) = open_store("debug", "a");
    let mut svc = OrgService::new(store, Box::new(chain.clone()));
    let pid = svc.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    svc.create_organisation(&mut OsRng, &pid).await.unwrap();
    let rec = &svc.list_orgs()[0];
    let secret = rec.org_private_key.clone().unwrap();

    let printed = format!("{rec:?}");
    assert!(printed.contains("org_private_key"), "the field is named in the output");
    let bytes = secret.expose_secret();
    assert!(!printed.contains(&format!("{bytes:?}")), "the Organisation private key is printed");
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    assert!(!printed.contains(&hex), "the Organisation private key is printed in hex");
}

/// A chain that counts the genesis submissions it is handed and otherwise
/// behaves as `MockChainOps`.
#[derive(Clone)]
struct GenesisCountingChain {
    inner: MockChainOps,
    geneses: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl GenesisCountingChain {
    fn new() -> Self {
        Self { inner: MockChainOps::new(), geneses: Default::default() }
    }

    fn geneses(&self) -> usize {
        self.geneses.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl ChainOps for GenesisCountingChain {
    async fn submit_genesis(
        &self,
        genesis_root: RootHash,
        org_pub_key: OrgPublicKey,
    ) -> Result<(OrgId, Option<ChainAccount>), OrgNodeError> {
        self.geneses.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.inner.submit_genesis(genesis_root, org_pub_key).await
    }

    async fn submit_update(
        &self,
        org_id: OrgId,
        new_root: RootHash,
        org_pub_key: OrgPublicKey,
        expected_epoch: Epoch,
        proxy_account: Option<ChainAccount>,
    ) -> Result<(), OrgNodeError> {
        self.inner.submit_update(org_id, new_root, org_pub_key, expected_epoch, proxy_account).await
    }

    async fn read_state(&self, org_id: OrgId) -> Result<Option<OrgState>, OrgNodeError> {
        self.inner.read_state(org_id).await
    }
}

// Abnormal case: a random source that repeats itself makes the Organisation
// private key equal to the member seed, so the published key would be the
// admin's Member-as-a-group key. Creation is refused before the chain is
// written: no genesis reaches the chain (review round 2, finding-5). Nothing
// is recorded. The same chain does receive the genesis of a creation that
// is not refused.
// verifies: REQ-ech45n, LLR-sj7cd5
#[tokio::test(flavor = "multi_thread")]
async fn an_organisation_key_equal_to_a_genesis_key_is_refused() {
    let chain = GenesisCountingChain::new();
    let (store, _) = open_store("collision", "a");
    let mut svc = OrgService::new(store, Box::new(chain.clone()));
    let mut rng = ConstRng(0x5c);
    let pid = svc.create_persona(&mut rng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let err = svc.create_organisation(&mut rng, &pid).await.unwrap_err();
    assert_eq!(err, OrgNodeError::Trie(OrgMembersError::DuplicateKey));
    assert_eq!(chain.geneses(), 0, "the chain must not be written");
    assert!(svc.list_orgs().is_empty(), "no Organisation recorded");
    assert_eq!(svc.list_personas()[0].status, PersonaStatus::Proposed);

    svc.create_organisation(&mut OsRng, &pid).await.expect("a fresh key is accepted");
    assert_eq!(chain.geneses(), 1, "the accepted creation is written");
}

const NET: Duration = Duration::from_secs(30);

/// A chain whose reads carry an Organisation public key that is not a
/// valid X25519 key: the bytes a corrupted or hostile chain answer would
/// carry, passed through the node's one edge for chain state.
struct InvalidKeyChain(MockChainOps);

#[async_trait::async_trait]
impl ChainOps for InvalidKeyChain {
    async fn submit_genesis(
        &self,
        genesis_root: RootHash,
        org_pub_key: OrgPublicKey,
    ) -> Result<(OrgId, Option<ChainAccount>), OrgNodeError> {
        self.0.submit_genesis(genesis_root, org_pub_key).await
    }

    async fn submit_update(
        &self,
        org_id: OrgId,
        new_root: RootHash,
        org_pub_key: OrgPublicKey,
        expected_epoch: Epoch,
        proxy_account: Option<ChainAccount>,
    ) -> Result<(), OrgNodeError> {
        self.0.submit_update(org_id, new_root, org_pub_key, expected_epoch, proxy_account).await
    }

    async fn read_state(&self, org_id: OrgId) -> Result<Option<OrgState>, OrgNodeError> {
        match self.0.read_state(org_id).await? {
            Some(s) => OrgState::from_chain(*s.root_hash.as_bytes(), [0u8; 32], s.epoch.get()).map(Some),
            None => Ok(None),
        }
    }
}

/// The device keypair of a persona, from its persisted `device_seed`.
fn device_kp(svc: &OrgService, persona_id: &PersonaId) -> SigningKeypair {
    let seed = svc
        .list_personas()
        .iter()
        .find(|p| &p.persona_id == persona_id)
        .map(|p| p.device_seed.clone())
        .expect("persona not found");
    seed.signing_keypair()
}

// The parse: canonical, non-small-order X25519 keys only, by person's rule.
// verifies: REQ-8jb4ny, LLR-3jjgtw
#[test]
fn an_organisation_public_key_that_is_not_a_valid_x25519_key_is_refused() {
    let valid = MemberSeed::from([0x42u8; 32]).x25519_keypair().public_bytes();
    assert_eq!(OrgPublicKey::parse(&valid).unwrap().as_bytes(), &valid);

    let mut top_bit = valid;
    top_bit[31] |= 0x80; // at least 2^255: not canonical
    let mut one = [0u8; 32];
    one[0] = 1; // small order
    let mut prime = [0xffu8; 32]; // 2^255 - 19: not canonical
    prime[0] = 0xed;
    prime[31] = 0x7f;
    for bad in [[0u8; 32], one, top_bit, prime] {
        assert_eq!(
            OrgPublicKey::parse(&bad),
            Err(OrgNodeError::InvalidOrgPublicKey),
            "{bad:02x?} must be refused"
        );
    }
}

// The edge: an Organisation state read from the chain is refused, with the
// typed error, when its key is not valid, and accepted with its fields
// intact when it is.
// verifies: REQ-8jb4ny
#[test]
fn an_organisation_state_read_with_an_invalid_key_is_refused() {
    assert_eq!(
        OrgState::from_chain([7u8; 32], [0u8; 32], 3),
        Err(OrgNodeError::InvalidOrgPublicKey)
    );
    let key = MemberSeed::from([0x42u8; 32]).x25519_keypair().public_bytes();
    let state = OrgState::from_chain([7u8; 32], key, 3).unwrap();
    assert_eq!(state.org_pub_key.as_bytes(), &key);
    assert_eq!(state.root_hash.as_bytes(), &[7u8; 32]);
    assert_eq!(state.epoch, Epoch::new(3));
}

// "Shall act on no Envelope verified against it": B, whose chain answers
// with an invalid Organisation public key, receives A's genuine admission
// from A's own device, the invite's administrator. The receive fails with
// the typed error and B commits nothing.
// verifies: REQ-8jb4ny
#[tokio::test(flavor = "multi_thread")]
async fn a_receive_against_a_state_with_an_invalid_key_commits_nothing() {
    let chain = MockChainOps::new();
    let (store_a, _) = open_store("invalid-key", "a");
    let (store_b, _) = open_store("invalid-key", "b");
    let mut svc_a = OrgService::new(store_a, Box::new(chain.clone()));
    let mut svc_b = OrgService::new(store_b, Box::new(InvalidKeyChain(chain.clone())));

    let pid_a = svc_a.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let org_id = svc_a.create_organisation(&mut OsRng, &pid_a).await.unwrap();
    let a_device = device_kp(&svc_a, &pid_a);
    let mut svc_a = svc_a.with_endpoint(OrgEndpoint::bind(&a_device).await.unwrap());

    let pid_b = svc_b.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("Builder")).unwrap();
    svc_b.import_invite(&mut OsRng, &svc_a.export_invite(org_id).unwrap()).unwrap();
    let jr = OrgService::import_join_request(&svc_b.export_join_request(&pid_b).unwrap()).unwrap();

    let ep_b = OrgEndpoint::bind(&device_kp(&svc_b, &pid_b)).await.unwrap();
    let b_addr = ep_b.inner().addr();
    let mut svc_b = svc_b.with_endpoint(ep_b);
    let b_task = tokio::spawn(async move {
        let r = tokio::time::timeout(NET, svc_b.receive_and_verify(&mut OsRng))
            .await
            .expect("B receive_and_verify timed out");
        (svc_b, r)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;

    tokio::time::timeout(
        NET,
        svc_a.admit_member(&mut OsRng, org_id, &jr, b_addr, Some(OrgSecret::from([0xffu8; 32]))),
    )
    .await
    .expect("admit_member(B) timed out")
    .expect("admit_member(B) failed");

    let (svc_b, r) = b_task.await.unwrap();
    assert_eq!(r.unwrap_err(), OrgNodeError::InvalidOrgPublicKey);
    assert!(svc_b.list_orgs().is_empty(), "B must not have committed an OrgRecord");
    let persona_b = svc_b.list_personas().iter().find(|p| p.persona_id == pid_b).unwrap();
    assert_ne!(persona_b.status, PersonaStatus::Active);
}

// LLR-sj7cd5's device-key clause, which no service-level test can reach: an
// X25519 key drawn at creation equals an ed25519 DevicePublicKey only by a 2^-128
// accident. So the check is tested at its own interface. A DevicePublicKey whose
// bytes happen to be a valid X25519 key stands in for that accident. Review
// round 1 (finding-9) found the device half of the check removable with the
// gate green (M7).
// verifies: REQ-ech45n, LLR-sj7cd5
#[test]
fn an_organisation_key_equal_to_any_genesis_member_or_device_key_is_refused() {
    use org_members::{Handle, MemberId, MemberLeaf, Name, Surname};
    let device = (0u8..=255)
        .map(|b| DeviceSeed::from([b; 32]).signing_keypair())
        .find(|d| OrgPublicKey::parse(d.verifying_key().as_bytes()).is_ok())
        .expect("some DevicePublicKey is also a valid X25519 key");
    let member = MemberSeed::from([0x71u8; 32]).x25519_keypair();
    let leaf = MemberLeaf::new(
        MemberId::new([1u8; 32]),
        Handle::parse("admin").unwrap(),
        member.member_key().unwrap(),
        Name::parse("Admin").unwrap(),
        Surname::parse("User").unwrap(),
        vec![device.device_key().unwrap()],
    )
    .unwrap();
    let members = [leaf];

    let as_device = OrgPublicKey::parse(device.verifying_key().as_bytes()).unwrap();
    assert_eq!(
        as_device.ensure_distinct_from(&members),
        Err(OrgNodeError::Trie(OrgMembersError::DuplicateKey)),
        "equal to a DevicePublicKey"
    );
    let as_member = member.org_public_key().unwrap();
    assert_eq!(
        as_member.ensure_distinct_from(&members),
        Err(OrgNodeError::Trie(OrgMembersError::DuplicateKey)),
        "equal to a Member-as-a-group key"
    );
    let fresh = OrgPrivateKey::from([0x72u8; 32]).x25519_keypair().org_public_key().unwrap();
    assert_eq!(fresh.ensure_distinct_from(&members), Ok(()), "a fresh key is accepted");
}

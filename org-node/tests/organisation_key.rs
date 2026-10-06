#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The Organisation public and private key. A created Organisation publishes
//! the X25519 public key of a secret generated for it alone, distinct from every key of its
//! genesis record, and that secret is kept in the creator's encrypted store
//! (REQ-ech45n), and every Member's record holds it once admitted (LLR-3fwykc).
//!
//! The gate: `cargo test -p org-node --features app,test-support --test organisation_key`

use std::path::PathBuf;

use org_members::OrgMembersError;
use org_node::chain::OrgState;
use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::service::{ChainOps, MockChainOps, OrgService};
use org_node::store::{OrgRecord, PersonaStatus, PersonaStore};
use org_node::transport::endpoint::OrgEndpoint;
use org_node::{DeviceSeed, Epoch, MemberSeed, OrgPrivateKey, OrgPublicKey, PersonPublicKey};
use rand::rngs::OsRng;
use rand::{CryptoRng, RngCore};

mod support;
use support::{
    admit, device_kp, found, h, joiner_of, nm, prepare_to_join, private_key_of, sn, spawn_receive,
    store_dir, test_proxy,
};

const PASSWORD: &str = "pw_org_key";

/// A fresh encrypted store, unique per test and party, and its file.
fn open_store(tag: &str, party: &str) -> (PersonaStore, PathBuf) {
    (support::open_store(tag, party, PASSWORD), store_dir(tag, party).join("store.bin"))
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

/// A random source that returns `first` for its first 32 bytes and `then`
/// after: a member id of `first` bytes, then an Organisation key pair drawn
/// from `then` bytes.
struct TwoRng {
    first: u8,
    then: u8,
    served: usize,
}

impl TwoRng {
    fn new(first: u8, then: u8) -> Self {
        Self { first, then, served: 0 }
    }
}

impl RngCore for TwoRng {
    fn next_u32(&mut self) -> u32 {
        let mut b = [0u8; 4];
        self.fill_bytes(&mut b);
        u32::from_le_bytes(b)
    }
    fn next_u64(&mut self) -> u64 {
        let mut b = [0u8; 8];
        self.fill_bytes(&mut b);
        u64::from_le_bytes(b)
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for byte in dest {
            *byte = if self.served < 32 { self.first } else { self.then };
            self.served += 1;
        }
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl CryptoRng for TwoRng {}

// Normal case: the published key is the X25519 public key of the secret the
// creator holds, and no member or device of the genesis record holds it.
// verifies: REQ-ech45n, LLR-sj7cd5, LLR-3fwykc
#[tokio::test(flavor = "multi_thread")]
async fn a_created_organisation_publishes_a_fresh_key_no_genesis_key_equals() {
    let chain = MockChainOps::new();
    let (store, _) = open_store("fresh", "a");
    let mut svc = OrgService::new(store, Box::new(chain.clone()));
    let pid = svc.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let update = svc.create_organisation(&mut OsRng, &pid).unwrap();
    let published = *update.org_pub_key.as_bytes();
    assert!(PersonPublicKey::parse(&published).is_ok(), "a valid X25519 public key");
    assert_eq!(
        private_key_of(&update).x25519_keypair().public_bytes(),
        published,
        "the update's key is the public key of the private key it holds"
    );

    let org_id = chain.apply_genesis(update.resulting_root, update.org_pub_key);
    svc.commit_genesis(&mut OsRng, &pid, org_id, test_proxy()).await.unwrap();
    assert_eq!(published_key(&chain, &org_id), published, "the chain carries the update's key");
    let rec = svc.list_orgs()[0].clone();
    assert_eq!(record_key(&rec), published, "the record holds the key it published");

    let secret = rec.org_private_key;
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
#[test]
fn two_organisations_of_one_persona_publish_two_keys() {
    let (store, _) = open_store("two-orgs", "a");
    let mut svc = OrgService::new(store, Box::new(MockChainOps::new()));
    let pid = svc.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let first = svc.create_organisation(&mut OsRng, &pid).unwrap();
    let second = svc.create_organisation(&mut OsRng, &pid).unwrap();
    assert_ne!(first.org_pub_key, second.org_pub_key);
}

// The Organisation private key is in the store, under the passphrase, and
// never in the store file in clear. (Renamed 2026-10-05 by review round 1:
// "Organisation secret" was org-node's term for another value, removed since.)
// verifies: REQ-ech45n, LLR-3fwykc, LLR-qjz3q4
#[tokio::test(flavor = "multi_thread")]
async fn the_organisation_private_key_is_kept_only_in_the_encrypted_store() {
    let chain = MockChainOps::new();
    let (store, path) = open_store("at-rest", "a");
    let mut svc = OrgService::new(store, Box::new(chain.clone()));
    let pid = svc.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let update = svc.create_organisation(&mut OsRng, &pid).unwrap();
    let secret = private_key_of(&update);

    let reopened = PersonaStore::open(path.clone(), PASSWORD).unwrap();
    assert_eq!(reopened.data().provisional_updates, vec![update.clone()], "the update holds the key");
    let bytes = std::fs::read(&path).unwrap();
    assert!(!contains(&bytes, secret.expose_secret()), "the key waiting in the update is in the file in clear");

    let org_id = chain.apply_genesis(update.resulting_root, update.org_pub_key);
    svc.commit_genesis(&mut OsRng, &pid, org_id, test_proxy()).await.unwrap();
    assert_eq!(svc.list_orgs()[0].org_private_key, secret.clone());

    let reopened = PersonaStore::open(path.clone(), PASSWORD).unwrap();
    let rec = reopened.data().orgs.iter().find(|o| o.org_id == org_id).unwrap();
    assert_eq!(rec.org_private_key, secret.clone(), "the store holds the Organisation private key");
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
    found(&mut svc, &chain, &pid).await;
    let rec = &svc.list_orgs()[0];
    let secret = rec.org_private_key.clone();

    let printed = format!("{rec:?}");
    assert!(printed.contains("org_private_key"), "the field is named in the output");
    let bytes = secret.expose_secret();
    assert!(!printed.contains(&format!("{bytes:?}")), "the Organisation private key is printed");
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    assert!(!printed.contains(&hex), "the Organisation private key is printed in hex");
}

// Abnormal case: a random source that repeats itself makes the Organisation
// private key equal to the member seed, so the published key would be the
// founder's Member-as-a-group key. Creation is refused before any genesis
// update is kept (review round 2, finding-5): nothing is kept, recorded or
// written. The same Persona's creation that is not refused keeps one update.
// It is also LLR-s6qnht's abnormal case: create_organisation refuses the pair.
// verifies: REQ-ech45n, LLR-sj7cd5, LLR-s6qnht
#[test]
fn an_organisation_key_equal_to_a_genesis_key_is_refused() {
    let (store, path) = open_store("collision", "a");
    let mut svc = OrgService::new(store, Box::new(MockChainOps::new()));
    let mut rng = ConstRng(0x5c);
    let pid = svc.create_persona(&mut rng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let before = std::fs::read(&path).unwrap();
    let err = svc.create_organisation(&mut rng, &pid).unwrap_err();
    assert_eq!(err, OrgNodeError::Trie(OrgMembersError::DuplicateKey));
    assert!(svc.genesis_provisional_updates(&pid).is_empty(), "no genesis update kept");
    assert_eq!(std::fs::read(&path).unwrap(), before, "nothing written");
    assert!(svc.list_orgs().is_empty(), "no Organisation recorded");
    assert_eq!(svc.list_personas()[0].status, PersonaStatus::Proposed);

    svc.create_organisation(&mut OsRng, &pid).expect("a fresh key is accepted");
    assert_eq!(svc.genesis_provisional_updates(&pid).len(), 1, "the accepted creation is kept");
}

// LLR-e2b7gv, LLR-sj7cd5 as amended: a key pair drawn for an admission whose
// public key is the record's current Organisation public key, or a
// Member-as-a-group key of the resulting record (the joiner's), is refused
// with DuplicateKey — no update kept, the record's keys unchanged, nothing
// written. A fresh draw is then accepted.
// It is also LLR-ghja3x's abnormal case: never the record's current key.
// verifies: LLR-e2b7gv, LLR-sj7cd5, REQ-stx9v3, LLR-ghja3x
#[tokio::test(flavor = "multi_thread")]
async fn an_update_key_equal_to_the_records_or_to_a_key_of_the_resulting_record_is_refused() {
    let chain = MockChainOps::new();
    let (store, path) = open_store("update-collision", "a");
    let mut svc = OrgService::new(store, Box::new(chain.clone()));
    let pid = svc.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    // A genesis whose founder id is 0x11…11 and whose Organisation private key is 0x3d…3d.
    let update = svc.create_organisation(&mut TwoRng::new(0x11, 0x3d), &pid).unwrap();
    let org_id = chain.apply_genesis(update.resulting_root, update.org_pub_key);
    svc.commit_genesis(&mut OsRng, &pid, org_id, test_proxy()).await.unwrap();
    let rec = svc.list_orgs()[0].clone();
    let carol = org_node::Joiner {
        handle: h("carol"),
        name: nm("Carol"),
        surname: sn("Coder"),
        member_key: MemberSeed::from([0x4e; 32]).x25519_keypair().member_key().unwrap(),
        device_key: DeviceSeed::from([0x4f; 32]).signing_keypair().device_key().unwrap(),
    };
    let before = std::fs::read(&path).unwrap();
    // 0x3d: the record's current key; 0x4e: the joiner's Member key.
    for then in [0x3d, 0x4e] {
        let err = svc.admit_member(&mut TwoRng::new(0x22, then), org_id, &carol).unwrap_err();
        assert_eq!(err, OrgNodeError::Trie(OrgMembersError::DuplicateKey), "draw {then:#x}");
        assert!(svc.provisional_updates(org_id).is_empty(), "no update kept");
        let now = svc.list_orgs()[0].clone();
        assert_eq!((now.org_pub_key, now.org_private_key.clone()), (rec.org_pub_key, rec.org_private_key.clone()));
        assert_eq!(std::fs::read(&path).unwrap(), before, "nothing written");
    }
    svc.admit_member(&mut OsRng, org_id, &carol).expect("a fresh draw is accepted");
}

// Abnormal (LLR-tax3pm): a revocation draws a fresh key pair like every
// update, never the record's current Organisation public key. A draw equal to
// the record's current key (removing Carol), or to a Member-as-a-group key of
// the resulting record (Carol's, removing the founder), is refused with
// DuplicateKey — no update kept, the record's keys unchanged, nothing
// written. A fresh draw is then accepted.
// verifies: LLR-tax3pm
#[tokio::test(flavor = "multi_thread")]
async fn a_revocation_key_equal_to_the_records_or_to_a_key_of_the_resulting_record_is_refused() {
    let chain = MockChainOps::new();
    let (store, path) = open_store("revoke-collision", "a");
    let mut svc = OrgService::new(store, Box::new(chain.clone()));
    let pid = svc.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let org_id = found(&mut svc, &chain, &pid).await;
    let carol = org_node::Joiner {
        handle: h("carol"),
        name: nm("Carol"),
        surname: sn("Coder"),
        member_key: MemberSeed::from([0x4e; 32]).x25519_keypair().member_key().unwrap(),
        device_key: DeviceSeed::from([0x4f; 32]).signing_keypair().device_key().unwrap(),
    };
    // Carol's admission, whose Organisation private key is 0x3e…3e.
    let update = svc.admit_member(&mut TwoRng::new(0x22, 0x3e), org_id, &carol).unwrap();
    chain.apply_update(org_id, update.resulting_root, update.org_pub_key, Epoch::new(1)).unwrap();
    svc.commit_update(&mut OsRng, org_id).await.unwrap();
    let rec = svc.list_orgs()[0].clone();
    assert_eq!(rec.org_private_key, OrgPrivateKey::from([0x3e; 32]), "the record holds the admission's key");
    let carol_id = rec.trie_members.iter().find(|m| m.member_key == carol.member_key).unwrap().id;
    let founder_id = rec.trie_members.iter().find(|m| m.member_key != carol.member_key).unwrap().id;
    let before = std::fs::read(&path).unwrap();
    // 0x3e: the record's current key; 0x4e: Carol's Member key, left in the
    // record that removing the founder produces.
    for (removed, draw) in [(carol_id, 0x3e), (founder_id, 0x4e)] {
        let err = svc.revoke_member(&mut ConstRng(draw), org_id, removed).unwrap_err();
        assert_eq!(err, OrgNodeError::Trie(OrgMembersError::DuplicateKey), "draw {draw:#x}");
        assert!(svc.provisional_updates(org_id).is_empty(), "no update kept");
        let now = svc.list_orgs()[0].clone();
        assert_eq!((now.org_pub_key, now.org_private_key.clone()), (rec.org_pub_key, rec.org_private_key.clone()));
        assert_eq!(now.trie_members, rec.trie_members, "the record unchanged");
        assert_eq!(std::fs::read(&path).unwrap(), before, "nothing written");
    }
    let accepted = svc.revoke_member(&mut OsRng, org_id, carol_id).expect("a fresh draw is accepted");
    assert_ne!(accepted.org_pub_key, rec.org_pub_key);
}

/// A chain whose reads carry an Organisation public key that is not a
/// valid X25519 key: the bytes a corrupted or hostile chain answer would
/// carry, passed through the node's one edge for chain state.
struct InvalidKeyChain(MockChainOps);

#[async_trait::async_trait]
impl ChainOps for InvalidKeyChain {
    async fn read_state(&self, org_id: OrgId) -> Result<Option<OrgState>, OrgNodeError> {
        match self.0.read_state(org_id).await? {
            Some(s) => OrgState::from_chain(*s.root_hash.as_bytes(), [0u8; 32], s.epoch.get()).map(Some),
            None => Ok(None),
        }
    }
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
// from A's own device. The receive fails with the typed error and B commits
// nothing.
// verifies: REQ-8jb4ny
#[tokio::test(flavor = "multi_thread")]
async fn a_receive_against_a_state_with_an_invalid_key_commits_nothing() {
    let chain = MockChainOps::new();
    let (store_a, _) = open_store("invalid-key", "a");
    let (store_b, _) = open_store("invalid-key", "b");
    let mut svc_a = OrgService::new(store_a, Box::new(chain.clone()));
    let mut svc_b = OrgService::new(store_b, Box::new(InvalidKeyChain(chain.clone())));

    let pid_a = svc_a.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let org_id = found(&mut svc_a, &chain, &pid_a).await;
    let a_device = device_kp(&svc_a, &pid_a);
    let mut svc_a = svc_a.with_endpoint(OrgEndpoint::bind(&a_device).await.unwrap());

    let pid_b = svc_b.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("Builder")).unwrap();
    let joiner = joiner_of(&svc_b, &pid_b);
    prepare_to_join(&mut svc_b, org_id);
    let b_device = device_kp(&svc_b, &pid_b);
    let (b_addr, b_task) = spawn_receive(svc_b, &b_device).await;

    // A builds the admission, the chain carries it (the mock stands in for
    // the app's write), A commits it and sends it to B.
    admit(&mut svc_a, &chain, org_id, &joiner, b_addr).await.expect("admit B");

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

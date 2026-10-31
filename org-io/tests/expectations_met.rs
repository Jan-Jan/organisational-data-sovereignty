#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(
    clippy::await_holding_lock,
    reason = "`fresh_keychain`'s lock serialises the tests over keyring's process-global credential builder for their whole run; each test has its own runtime"
)]
//! The expectations org-io declared on org-node, shown to hold through
//! org-io (owner's ruling 1 of 2026-10-14). Each REQ below was `expects:
//! org-node` and is met by a requirement org-node exports; a met expectation
//! is an ordinary REQ and owes a test here, at org-io's interface (the
//! `OrgIo` handle, its store files and its endpoints):
//!
//! - REQ-x6kdse: a Persona's store opens from its sealed bytes and its
//!   keychain secret, and every change reaches its file whole;
//! - REQ-qza96g: a store file opens only under its own Persona's secret, and
//!   keeps the Device public key, never the seed;
//! - REQ-v6r2gd: the device secret is persisted nowhere but the keychain,
//!   and renders no bytes;
//! - REQ-2xgsbn: the outstanding invites are kept in the store files and
//!   settled by an adoption; an Invite that does not parse changes no file;
//! - REQ-u894ty: each recipient of a sent epoch gets the body org-node built
//!   for it, the removed Device included;
//! - REQ-2yw5y3: one path keeps Organisation information, an admission
//!   request and a revocation notice, adopting none; a refused body reads no
//!   chain and writes nothing.
//!
//! Every test runs over fresh directories and a fresh in-memory keychain, one
//! test at a time under `fresh_keychain`'s lock: keyring's default
//! credential builder is process-global.

use std::path::{Path, PathBuf};

use org_node::store::{PersonaStore, SealedStore};
use org_node::{DevicePublicKey, OrgNodeError};
use org_io::send::Delivery;
use org_io::test_support::{dial_direct_as, FakeChain, MemoryKeychain, Refusal};
use org_io::transport::TransportMode;
use org_io::{HeldTarget, KeychainError, OpenError, OrgIo, ReceiveError, Received, Verified, WriteReport};

mod common;
use common::handles::{
    create, device_key_of, file_of, fresh_keychain, held_store, keep_device_seed, persona_of, reopened_from, store_bytes,
};
use common::journey;

const SERVICE: &str = "ods-device-seed";

/// A fresh, empty data directory for `party` of test `tag`.
fn fresh_dir(tag: &str, party: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("org-io-expectations-{tag}-{party}-{}", std::process::id()));
    // A directory an earlier run left is cleared; none there is fine.
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// `OrgIo::open` over `dir`, Loopback, with `chain` substituted.
fn open(dir: &Path, chain: &FakeChain) -> OrgIo {
    OrgIo::open_for_test(dir, TransportMode::Loopback, chain).unwrap()
}

fn open_refused(dir: &Path) -> OpenError {
    match OrgIo::open(dir, TransportMode::Loopback) {
        Ok(_) => panic!("opened, and it should have been refused"),
        Err(error) => error,
    }
}

/// Every file's bytes in `dir`, by name.
fn files_in(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files: Vec<(String, Vec<u8>)> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            (path.file_name().unwrap().to_string_lossy().to_string(), std::fs::read(&path).unwrap())
        })
        .collect();
    files.sort();
    files
}

/// The 32-byte Device seed the keychain holds for `device_key`.
fn seed_of(device_key: &DevicePublicKey) -> [u8; 32] {
    org_io::test_support::keychain::with_endpoint_seed(device_key, |seed| *seed).unwrap()
}

// ---- REQ-x6kdse: open from sealed bytes and the secret; every change whole ---

/// Whether `haystack` contains `needle` anywhere.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| window == needle)
}

// Each store-changing operation of the handle — the founding's
// proposal, its proxy recorded by the write, its adoption, an Invite issued,
// an admission request kept by the receive step, the admission and the
// updated Organisation proposed, and their adoption — leaves a file that a
// fresh `OrgIo::open` unseals with the keychain item into exactly the store
// the handle holds: org-io wrote the complete sealed store each time.
// verifies: REQ-x6kdse
#[tokio::test(flavor = "multi_thread")]
async fn every_store_change_reaches_the_file_whole_and_reopens_as_held() {
    let (_guard, _keychain) = fresh_keychain();
    let dir = fresh_dir("whole", "a");
    let chain = FakeChain::new();
    let io = open(&dir, &chain);
    let founder = journey::propose_founding(&io, "alice").await;
    let alice = persona_of(&io, &founder).await;
    // What the handle holds is seen through its own queries: a fresh `open`
    // of the directory shows the same Persona, Organisation and kept
    // requests.
    let reopens_as_held = |step: &'static str| {
        let (dir, chain, io, alice) = (&dir, &chain, &io, &alice);
        async move {
            let reopened = open(dir, chain);
            assert_eq!(held_store(&reopened, alice).await, held_store(io, alice).await, "after {step}: the file holds the whole store");
        }
    };
    reopens_as_held("propose_founding_organisation").await;
    let written = io.write_organisation_to_chain(HeldTarget::Founding(founder)).await.unwrap();
    assert!(matches!(written, WriteReport::Included { .. }), "{written:?}");
    reopens_as_held("write_organisation_to_chain (the founding's proxy recorded)").await;
    assert!(matches!(io.verify_organisation(HeldTarget::Founding(founder)).await, Verified::Adopted { .. }));
    reopens_as_held("verify_organisation (the founding adopted)").await;
    let org = io.organisations().await[0].org_id;
    let invite = io.issue_invite(org, "Bob").await.unwrap();
    reopens_as_held("issue_invite").await;

    io.start_receiving().await.unwrap();
    let joiner = open(&fresh_dir("whole", "b"), &FakeChain::sharing_slots_with(&chain));
    let member = joiner.propose_member(&invite, "Bob", "Builder", &["bob"]).await.unwrap();
    let (kept, _) = tokio::join!(journey::next(&io), joiner.request_admission(org));
    assert!(matches!(kept, Ok(Received::AdmissionRequestKept { .. })), "{kept:?}");
    reopens_as_held("next_received (the admission request kept)").await;
    io.propose_admission(org, member.invite_id, None).await.unwrap();
    reopens_as_held("propose_admission").await;
    io.propose_updated_organisation(org).await.unwrap();
    reopens_as_held("propose_updated_organisation").await;
    let written = io.write_organisation_to_chain(HeldTarget::Organisation(org)).await.unwrap();
    assert!(matches!(written, WriteReport::Included { .. }), "{written:?}");
    assert!(matches!(io.verify_organisation(HeldTarget::Organisation(org)).await, Verified::Adopted { .. }));
    reopens_as_held("verify_organisation (the admission adopted)").await;
    assert_eq!(files_in(&dir).len(), 1, "one store for the one Persona, and nothing else");
}

// Abnormal: a store file whose bytes org-node cannot unseal — empty,
// truncated, one byte flipped, or not a store at all — refuses the open,
// naming the file and its Device key, and the file is left as it was.
// verifies: REQ-x6kdse
#[tokio::test(flavor = "multi_thread")]
async fn a_store_file_org_node_cannot_unseal_is_refused_naming_it_and_left_untouched() {
    let (_guard, _keychain) = fresh_keychain();
    let dir = fresh_dir("unsealable", "a");
    let io = open(&dir, &FakeChain::new());
    let alice = create(&io, "alice").await;
    let device_key = device_key_of(&io, &alice).await;
    let file = file_of(&dir, &device_key);
    let sealed = std::fs::read(&file).unwrap();
    drop(io);
    let mut flipped = sealed.clone();
    let middle = flipped.len() / 2;
    flipped[middle] ^= 0x01;
    for (case, bytes) in [
        ("empty", Vec::new()),
        ("truncated", sealed[..sealed.len() / 2].to_vec()),
        ("one byte flipped", flipped),
        ("not a store", b"not a sealed store".to_vec()),
    ] {
        std::fs::write(&file, &bytes).unwrap();
        match open_refused(&dir) {
            OpenError::Unseal { device_key: named, path, .. } => {
                assert_eq!((named, path), (device_key, file.clone()), "{case}: the refusal names the file and its key");
            }
            other => panic!("{case}: not org-node's unseal refusal: {other:?}"),
        }
        assert_eq!(std::fs::read(&file).unwrap(), bytes, "{case}: the file is left as it was");
    }
}

// ---- REQ-qza96g: sealed under the Persona's own secret; no seed kept ---------

// A founding proposed, written and adopted through the handle
// leaves a store file that holds the Device seed in no form, opens under its
// own Persona's keychain item, and keeps that Persona's Device public key.
// verifies: REQ-qza96g
#[tokio::test(flavor = "multi_thread")]
async fn a_store_file_opens_under_its_own_item_and_keeps_the_device_key_not_the_seed() {
    let (_guard, _keychain) = fresh_keychain();
    let dir = fresh_dir("own-item", "a");
    let chain = FakeChain::new();
    let io = open(&dir, &chain);
    let (_, device_key) = common::handles::founded(&io, "alice").await;
    let seed = seed_of(&device_key);
    let sealed = SealedStore::from_bytes(std::fs::read(file_of(&dir, &device_key)).unwrap());
    assert!(!contains(sealed.as_bytes(), &seed), "the sealed file holds no seed");

    let reopened = open(&dir, &chain);
    let persona = persona_of(&reopened, &device_key).await;
    assert_eq!(device_key_of(&reopened, &persona).await, device_key, "the store keeps the Device public key");
    let store = PersonaStore::unseal(&sealed, device_key, io.device_secret_for_test(&device_key).unwrap()).unwrap();
    assert_eq!(store.device_key(), device_key);
    let contents = format!("{:?}", store.data());
    assert!(!contents.contains(&hex::encode(seed)), "the unsealed store holds no seed, as hex");
    assert!(!contents.contains(&format!("{:?}", seed)), "the unsealed store holds no seed, as bytes");
}

// Abnormal: another Persona's sealed store under this Persona's file name
// is refused by the unseal (the store key is this Persona's alone); an item
// holding another Device's valid seed is refused before any unseal; neither
// open writes anything.
// verifies: REQ-qza96g
#[tokio::test(flavor = "multi_thread")]
async fn a_store_file_opens_under_no_other_personas_secret() {
    let (_guard, keychain) = fresh_keychain();
    let dir = fresh_dir("other-secret", "a");
    let io = open(&dir, &FakeChain::new());
    let alice = create(&io, "alice").await;
    let bob = create(&io, "bob").await;
    let (alice_key, bob_key) = (device_key_of(&io, &alice).await, device_key_of(&io, &bob).await);
    let alice_file = file_of(&dir, &alice_key);
    let alice_sealed = std::fs::read(&alice_file).unwrap();
    drop(io);

    std::fs::write(&alice_file, std::fs::read(file_of(&dir, &bob_key)).unwrap()).unwrap();
    match open_refused(&dir) {
        OpenError::Unseal { device_key, .. } => assert_eq!(device_key, alice_key, "Bob's store does not open as Alice's"),
        other => panic!("not org-node's unseal refusal: {other:?}"),
    }

    std::fs::write(&alice_file, &alice_sealed).unwrap();
    keychain.insert(SERVICE, &hex::encode(alice_key.as_bytes()), seed_of(&bob_key).to_vec());
    match open_refused(&dir) {
        OpenError::Keychain { error: KeychainError::KeyMismatch { device_key }, .. } => assert_eq!(device_key, alice_key),
        other => panic!("not the keychain's key mismatch: {other:?}"),
    }
    assert_eq!(std::fs::read(&alice_file).unwrap(), alice_sealed, "nothing written");
}

// ---- REQ-v6r2gd: the secret is opaque and persisted only in the keychain -----

// After a Persona is created by a founding's proposal and its
// store changed by the write, the adoption and an Invite, no file in the
// data directory holds the Device seed; the keychain holds exactly one item,
// by Device key; and the secret org-io reads back renders no bytes.
// verifies: REQ-v6r2gd
#[tokio::test(flavor = "multi_thread")]
async fn the_device_seed_is_in_the_keychain_alone_and_renders_no_bytes() {
    let (_guard, keychain) = fresh_keychain();
    let dir = fresh_dir("seed-nowhere", "a");
    let io = open(&dir, &FakeChain::new());
    let (org, device_key) = common::handles::founded(&io, "alice").await;
    io.issue_invite(org, "Bob Builder").await.unwrap();
    let seed = seed_of(&device_key);

    let files = files_in(&dir);
    assert!(!files.is_empty(), "the handle wrote its store");
    for (name, bytes) in files {
        assert!(!contains(&bytes, &seed), "{name} holds the Device seed");
    }
    assert_eq!(keychain.items(), vec![(SERVICE.to_string(), hex::encode(device_key.as_bytes()))], "one item, by Device key");
    let rendered = format!("{:?}", io.device_secret_for_test(&device_key).unwrap());
    assert!(!rendered.contains(&hex::encode(seed)) && !rendered.contains(&format!("{:?}", seed)), "{rendered}");
}

// Abnormal: with the keychain's item gone, malformed, or the keychain
// locked, the store file alone opens nothing — nothing on disk stands in for
// the secret — and the file is left as it was.
// verifies: REQ-v6r2gd
#[tokio::test(flavor = "multi_thread")]
async fn without_its_keychain_item_a_store_file_opens_nothing() {
    let (_guard, keychain) = fresh_keychain();
    let dir = fresh_dir("no-item", "a");
    let io = open(&dir, &FakeChain::new());
    let alice = create(&io, "alice").await;
    let device_key = device_key_of(&io, &alice).await;
    let file = file_of(&dir, &device_key);
    let sealed = std::fs::read(&file).unwrap();
    drop(io);

    keychain.refuse_as(Some(Refusal::Locked));
    let locked = open_refused(&dir);
    keychain.refuse_as(None);
    assert!(matches!(locked, OpenError::Keychain { error: KeychainError::Locked { .. }, .. }), "{locked:?}");

    keychain.insert(SERVICE, &hex::encode(device_key.as_bytes()), vec![0x07; 31]);
    let malformed = open_refused(&dir);
    assert!(matches!(malformed, OpenError::Keychain { error: KeychainError::Malformed { .. }, .. }), "{malformed:?}");

    MemoryKeychain::default().install();
    match open_refused(&dir) {
        OpenError::KeychainItemMissing { device_key: named, path } => assert_eq!((named, path), (device_key, file.clone())),
        other => panic!("not a missing item: {other:?}"),
    }
    assert_eq!(std::fs::read(&file).unwrap(), sealed, "the file is left as it was");
}

// ---- REQ-2xgsbn: the outstanding invites, kept in the store files and settled -

// REQ-2xgsbn (the outstanding invites are kept in the store files and settled by an adoption):
// the issuing Persona's file keeps the invite across a reopen, and the adoption that admits the
// invitee removes it, as the file then shows. Every step is the handle's own operation — the
// founding proposed, written and adopted, the Invite issued, the joiner's request kept by the
// receive step, the admission proposed, written and adopted by `verify_organisation` — and a
// fresh `open` of the directory shows what reached the file.
// verifies: REQ-2xgsbn
#[tokio::test(flavor = "multi_thread")]
async fn outstanding_invites_are_kept_in_the_store_files_and_settled_by_adoption() {
    let (_guard, _keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let dir_a = fresh_dir("invites-settled", "a");
    let alice = open(&dir_a, &chain);
    let (org_id, founder) = common::handles::founded(&alice, "alice").await;
    let invite = alice.issue_invite(org_id, "Bob").await.unwrap();
    // What reached the file, read apart from the handle.
    let kept = |io: &OrgIo| reopened_from(&dir_a, io).store_data(&founder).unwrap().outstanding_invites.len();
    assert_eq!(kept(&open(&dir_a, &chain)), 1, "the invite is kept in the file across a reopen");

    alice.start_receiving().await.unwrap();
    let bob = open(&fresh_dir("invites-settled", "b"), &FakeChain::sharing_slots_with(&chain));
    let joiner = bob.propose_member(&invite, "Bob", "Builder", &["bob"]).await.unwrap();
    let (request, _) = tokio::join!(journey::next(&alice), bob.request_admission(org_id));
    assert!(matches!(request, Ok(Received::AdmissionRequestKept { .. })), "{request:?}");
    alice.propose_admission(org_id, joiner.invite_id, None).await.unwrap();
    alice.propose_updated_organisation(org_id).await.unwrap();
    let written = alice.write_organisation_to_chain(HeldTarget::Organisation(org_id)).await.unwrap();
    assert!(matches!(written, WriteReport::Included { .. }), "{written:?}");
    assert_eq!(kept(&open(&dir_a, &chain)), 1, "kept until the adoption");
    assert!(matches!(alice.verify_organisation(HeldTarget::Organisation(org_id)).await, Verified::Adopted { .. }));
    assert_eq!(kept(&open(&dir_a, &chain)), 0, "the adoption that admits the invitee settles the invite, in the file");
}

// REQ-2xgsbn: a Blob that does not parse changes no file and writes no item.
// verifies: REQ-2xgsbn
#[tokio::test]
async fn an_invite_that_does_not_parse_changes_no_file() {
    let (_guard, keychain) = fresh_keychain();
    let chain = FakeChain::new();
    let dir = fresh_dir("invite-unparsed", "b");
    let bob = open(&dir, &chain);
    let (files, items) = (files_in(&dir), keychain.items());
    for blob in ["", "not base64 !", "AAAA"] {
        assert!(bob.propose_member(blob, "Bob", "Builder", &["bob"]).await.is_err(), "{blob:?} parsed");
    }
    assert_eq!((files_in(&dir), keychain.items()), (files, items), "no file changed and no item written");
}

// REQ-u894ty's two tests, over `send_organisation`.

// Networked semantics aside (Loopback here): A revokes Bob from {Alice, Bob,
// Carol} and sends the adopted epoch to their real receiving handles. The
// report names Bob and Carol, never A's own Device; Carol, still listed,
// keeps the Organisation's information and Bob, removed, the revocation —
// each from A's Device.
// verifies: REQ-u894ty
#[tokio::test(flavor = "multi_thread")]
async fn each_recipient_receives_the_body_org_node_built_for_the_adopted_epoch() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "expect-per-recipient";
    let admin = journey::admin(tag).await;
    let bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    let carol = journey::member(&admin, tag, "c", "carol", &[&bob]).await;
    journey::revoke(&admin, bob.device_key).await;

    let (received, report) = journey::epoch_to(&admin, &[&bob.io, &carol.io]).await;
    let mut reported: Vec<_> = report.per_device.iter().map(|(recipient, _)| *recipient.as_bytes()).collect();
    reported.sort();
    let mut expected = vec![*bob.device_key.as_bytes(), *carol.device_key.as_bytes()];
    expected.sort();
    assert_eq!(reported, expected, "Bob and Carol, never the sender's own Device");
    assert!(report.per_device.iter().all(|(_, sent)| matches!(sent, Delivery::Delivered)), "{report:?}");
    assert!(
        matches!(received[1], Ok(Received::ProvisionalOrganisationReceived { from, .. }) if from == admin.device_key),
        "Carol keeps the information: {:?}",
        received[1]
    );
    assert!(
        matches!(received[0], Ok(Received::RevocationNoticeKept { from, .. }) if from == admin.device_key),
        "Bob keeps the revocation: {:?}",
        received[0]
    );
}

// Boundary: revoking the last other Member leaves a record listing the
// sender alone; the one recipient is the removed Device, and its real
// receiving handle keeps the revocation.
// verifies: REQ-u894ty
#[tokio::test(flavor = "multi_thread")]
async fn the_last_other_member_removed_is_still_sent_its_revocation_by_send_organisation() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "expect-last-removed";
    let admin = journey::admin(tag).await;
    let bob = journey::member(&admin, tag, "b", "bob", &[]).await;
    journey::revoke(&admin, bob.device_key).await;
    assert_eq!(admin.io.organisation(admin.org).await.unwrap().members.len(), 1, "the sender alone");
    let (to_bob, report) = journey::deliver_epoch(&admin.io, admin.org, &bob.io).await;
    assert_eq!(report.per_device.len(), 1, "{report:?}");
    assert_eq!(report.per_device[0].0, bob.device_key, "the removed Device, which the record no longer names");
    assert!(matches!(to_bob, Ok(Received::RevocationNoticeKept { from, .. }) if from == admin.device_key), "{to_bob:?}");
}

// One path, `next_received`, keeps the three kinds in turn — the admission request at
// the Admin, the admitting epoch at the joiner, the revocation notice at the
// removed Member — each in its store file before the report, none adopted.
// verifies: REQ-2yw5y3
#[tokio::test(flavor = "multi_thread")]
async fn one_receive_path_keeps_organisation_information_an_admission_request_and_a_notice() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "expect-one-path";
    let admin = journey::admin(tag).await;
    admin.io.start_receiving().await.unwrap();
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    bob.io.start_receiving().await.unwrap();
    let (kept, _) = journey::request_over_transport(&admin, &bob).await;
    assert!(matches!(kept, Ok(Received::AdmissionRequestKept { .. })), "{kept:?}");
    assert_eq!(admin.io.admission_requests().await.len(), 1, "the request is kept");
    journey::propose_admission(&admin, &bob).await;
    journey::write_and_adopt(&admin).await;
    let (information, _) = journey::deliver_epoch(&admin.io, admin.org, &bob.io).await;
    assert!(matches!(information, Ok(Received::ProvisionalOrganisationReceived { .. })), "{information:?}");
    assert!(bob.io.organisations().await.is_empty(), "kept, not adopted");
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Adopted { .. })]));
    journey::revoke(&admin, bob.device_key).await;
    let (notice, _) = journey::deliver_epoch(&admin.io, admin.org, &bob.io).await;
    assert!(matches!(notice, Ok(Received::RevocationNoticeKept { .. })), "{notice:?}");
    assert_eq!(bob.io.organisations().await.len(), 1, "kept, nothing removed on receipt");
}

// Abnormal: a body org-node's check refuses reads no
// chain and writes nothing.
// verifies: REQ-2yw5y3
#[tokio::test(flavor = "multi_thread")]
async fn a_body_the_check_refuses_reads_no_chain_and_writes_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "expect-check-refuses";
    let admin = journey::admin(tag).await;
    admin.io.start_receiving().await.unwrap();
    let (files, reads) = (store_bytes(tag, "a"), admin.chain.reads());
    let stranger = keep_device_seed([0x64; 32]);
    let addr = journey::address_of(&admin.io).await;
    let (refused, _) = tokio::join!(journey::next(&admin.io), dial_direct_as(&stranger, addr, b"not a wire message"));
    assert!(matches!(refused, Err(ReceiveError::Refused(OrgNodeError::MalformedMessage))), "{refused:?}");
    assert_eq!(admin.chain.reads(), reads, "no chain read after a refused check");
    assert_eq!(store_bytes(tag, "a"), files, "nothing written");
}

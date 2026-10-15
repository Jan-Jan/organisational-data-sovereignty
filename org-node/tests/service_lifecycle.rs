#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The chain-operations boundary (SDD-ueh4tm: since 2026-10-08 the state
//! value each chain-judging operation takes, and the test-side
//! `ChainSlots`), and persona and Organisation genesis (SDD-89es4z). The
//! endpoint lifecycle's tests moved to org-io with the endpoint (S4, task
//! T10).
//!
//! Relocated from the `#[cfg(test)]` module in `org-node/src/service.rs` so
//! the `verifies:` annotations sit under `test_paths`. The seam cases and the
//! endpoint-lifecycle cases are new.
//!
//! *Adapted 2026-10-08 (S4, task T3):* the service writes no file and keeps
//! no device seed. "The disk" is the `StoreWrite` an operation returns, and
//! the endpoint is bound from the `DeviceSecret` `create_persona` returned
//! (the transitional `bind_endpoint`). Annotations unchanged.

use org_node::chain::OrgState;
use org_node::ids::OrgId;
use org_node::store::{PersonaStatus, PersonaStore, StoreWrite};
use org_node::DeviceSecret;
use zeroize::Zeroizing;
use org_node::test_fixtures::ChainSlots;
use org_node::OrgService;
use org_node::{Epoch, OrgPublicKey, RootHash};
use rand::rngs::OsRng;

mod support;
use support::{h, nm, sn};

/// A valid Organisation public key: `OrgPublicKey` is built only through
/// `parse` (REQ-8jb4ny). Was the bytes `[2u8; 32]` before the merge of
/// 2026-10-05.
fn org_key() -> OrgPublicKey {
    org_node::OrgPrivateKey::from([2u8; 32]).x25519_keypair().org_public_key().unwrap()
}

/// A service with no Persona.
fn service() -> OrgService {
    OrgService::new(vec![]).unwrap()
}

/// The seed bytes of `secret`, as org-io's keychain would hold them.
fn bytes_of(secret: &DeviceSecret) -> Vec<u8> {
    let mut bytes = vec![];
    secret
        .lend_to_keychain(|seed| {
            bytes.extend_from_slice(seed);
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap();
    bytes
}

/// A secret re-made from keychain bytes.
fn secret_from(bytes: &[u8]) -> DeviceSecret {
    DeviceSecret::from_keychain_bytes(Zeroizing::new(bytes.to_vec())).unwrap()
}

/// What `write` sealed, opened with the secret of `bytes`.
fn reload(write: &StoreWrite, bytes: &[u8]) -> PersonaStore {
    PersonaStore::unseal(&write.sealed, write.device_key, secret_from(bytes)).unwrap()
}

// verifies: REQ-hzm4kt, LLR-tev8h8, LLR-s7yu4k, LLR-v82xds
#[test]
fn a_new_persona_is_proposed_with_an_id_derived_from_its_member_key() {
    let mut svc = service();
    let (pid, secret, write) = svc.create_persona(&mut OsRng, h("alice"), nm("Alice"), sn("Smith")).unwrap();

    assert!(!pid.as_str().is_empty());
    assert_eq!(svc.list_personas().len(), 1);
    let rec = &svc.list_personas()[0];
    assert_eq!(rec.status, PersonaStatus::Proposed);
    assert_eq!(rec.org_id, None);
    assert_eq!(rec.member_id, None);

    // The identifier is derived from the member key, so it is reproducible
    // from the stored seed rather than drawn independently. On this branch
    // the member key is the X25519 public key of the member seed (merged
    // 2026-10-05; master derived it from an ed25519 verifying key).
    let member_kp = rec.member_seed_for_test().x25519_keypair();
    let expected: String = member_kp
        .public_bytes()
        .iter()
        .take(16)
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(pid.as_str(), expected);

    // The member and device keypairs are independent of each other.
    assert_ne!(rec.member_seed_for_test().expose_secret().as_slice(), bytes_of(&secret).as_slice());

    // It survives a reload from what it wrote.
    let reopened = reload(&write, &bytes_of(&secret));
    let svc2 = OrgService::new(vec![reopened]).unwrap();
    assert_eq!(svc2.list_personas().len(), 1);
    assert_eq!(svc2.list_personas()[0].handle.as_str(), "alice");
}

// Rewritten 2026-10-06 (T9 of docs/plans/2026-10-05-chain-authority.md):
// creation no longer writes the chain, records or activates anything; it
// keeps a genesis provisional update. The record, the activation and
// LLR-q3aj8z moved to `commit_paths`'s
// `commit_genesis_creates_the_record_once_the_chain_carries_the_root`.
// verifies: LLR-68yd3j, REQ-xs4ab8
#[test]
fn creating_an_organisation_writes_only_a_genesis_update() {
    let chain = ChainSlots::new();
    let chain_view = chain.clone();
    let mut svc = service();

    let (pid, secret, _) = svc.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let (update, write) = svc.create_organisation(&mut OsRng, &pid).unwrap();

    assert!(svc.list_orgs().is_empty(), "no record");
    assert_eq!(svc.list_personas()[0].status, PersonaStatus::Proposed);
    assert_eq!(svc.list_personas()[0].member_id, None);
    // No slot: the id the mock's genesis would derive from this root is empty.
    let mut would_be = [0u8; 20];
    would_be.copy_from_slice(&update.resulting_root.as_bytes()[..20]);
    assert!(chain_view.get(&OrgId::new(would_be)).is_none(), "the chain is unchanged");

    // "…and the store is written before it returns." Read back from the
    // write it returned, so an update that only ever existed in memory fails
    // here.
    let reloaded = reload(&write, &bytes_of(&secret));
    assert!(reloaded.data().orgs.is_empty(), "no record reaches the disk");
    assert_eq!(reloaded.data().provisional_updates.len(), 1, "the genesis update must reach the disk");
}

// verifies: LLR-hg3xzf
#[test]
fn clones_of_the_mock_chain_share_one_state() {
    let chain = ChainSlots::new();
    let view = chain.clone();
    let org = OrgId::new([4u8; 20]);
    assert!(view.get(&org).is_none());

    chain.set(
        org,
        OrgState { root_hash: RootHash::new([1u8; 32]), org_pub_key: org_key(), epoch: Epoch::new(3) },
    );
    // Written through one handle, visible through the other — which is what
    // lets two services under test observe the same chain.
    assert_eq!(view.get(&org).unwrap().epoch, Epoch::new(3));
}

// The mock refuses an update whose expected epoch is not the slot's current
// one, and leaves the slot as it was, as the contract's compare-and-swap does.
// Disabling the check was green: every story writes the mock chain at the
// right epoch.
// Added 2026-10-05 by review round 8.
// verifies: LLR-ryzr8m
#[test]
fn the_mock_chain_refuses_an_update_at_the_wrong_epoch() {
    let chain = ChainSlots::new();
    let org = OrgId::new([6u8; 20]);
    let at_three = OrgState { root_hash: RootHash::new([1u8; 32]), org_pub_key: org_key(), epoch: Epoch::new(3) };
    chain.set(org, at_three);
    for wrong in [Epoch::new(2), Epoch::new(4)] {
        let err = chain.apply_update(org, RootHash::new([9u8; 32]), org_key(), wrong).unwrap_err();
        assert!(
            matches!(&err, org_node::OrgNodeError::Chain(m) if m.contains("epoch mismatch")),
            "expected epoch {wrong:?} must be refused, got {err:?}"
        );
        assert_eq!(chain.get(&org).unwrap(), at_three, "a refused update leaves the slot");
    }
    chain.apply_update(org, RootHash::new([9u8; 32]), org_key(), Epoch::new(3)).unwrap();
    assert_eq!(chain.get(&org).unwrap().epoch, Epoch::new(4), "the right epoch is accepted and advances");
}

// *Rewritten 2026-10-08 (ruling B, change worktree-org-io-create).* Was
// `the_seam_is_a_trait_object_a_substitute_can_stand_in_for` (LLR-65py3d,
// whose annotation moved to `absences.rs`'s
// `org_node_names_no_chain_library_and_reads_no_chain`): there is no seam.
// Two services over one `ChainSlots`: one commits its genesis against the
// state it is given, and the other's view of the same slots holds that state.
// verifies: LLR-hg3xzf
#[test]
fn two_services_over_one_set_of_chain_slots_see_the_same_state() {
    let chain = ChainSlots::new();
    let other_view = chain.clone();
    let mut svc = service();
    let other = service();
    let (pid, _, _) = svc.create_persona(&mut OsRng, h("s"), nm("S"), sn("T")).unwrap();
    let (update, _) = svc.create_organisation(&mut OsRng, &pid).unwrap();
    let org = chain.apply_genesis(update.resulting_root, update.org_pub_key);
    let (committed, _) =
        svc.commit_genesis(&mut OsRng, &pid, org, org_node::ChainAccount::new([1; 32]), chain.get(&org)).unwrap();
    assert!(other.list_orgs().is_empty(), "the other service holds no record");
    let seen = other_view.get(&org).expect("the other view sees the slot");
    assert_eq!((seen.root_hash, seen.epoch), (committed.root, committed.epoch));
}

// Normal: the stand-in for the genesis write makes a slot at epoch one and
// returns its id; abnormal: a second genesis with the same root gets its own
// slot and leaves the first as it was.
// verifies: LLR-ryzr8m
#[test]
fn the_mock_genesis_makes_a_slot_at_epoch_one() {
    let chain = ChainSlots::new();
    let root = RootHash::new([3u8; 32]);
    let first = chain.apply_genesis(root, org_key());
    assert_eq!(chain.get(&first).unwrap(), OrgState { root_hash: root, org_pub_key: org_key(), epoch: Epoch::new(1) });
    let second = chain.apply_genesis(root, org_key());
    assert_ne!(first, second);
    assert_eq!(chain.get(&first).unwrap().epoch, Epoch::new(1));
}

// *Moved 2026-10-13 (S4, task T10 of docs/plans/2026-10-06-org-io-transport.md):*
// the three endpoint-lifecycle tests that stood here moved to org-io
// (`org-io/tests/endpoints.rs`) with the endpoint, rewritten onto org-io's
// endpoints, one per Persona, each bound from its keychain item.


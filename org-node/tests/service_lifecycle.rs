#![cfg(feature = "app")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The chain-operations seam (SDD-ueh4tm), persona and Organisation genesis
//! (SDD-89es4z), and the endpoint lifecycle (SDD-b8tuv3).
//!
//! Relocated from the `#[cfg(test)]` module in `org-node/src/service.rs` so
//! the `verifies:` annotations sit under `test_paths`. The seam cases and the
//! endpoint-lifecycle cases are new.

use org_node::chain::OrgState;
use org_node::ids::OrgId;
use org_node::store::{PersonaStatus, PersonaStore};
use org_node::transport::TransportMode;
use org_node::{MockChainOps, OrgService};
use org_members::{Handle, Name, RootHash, Surname};
use org_node::{Epoch, OrgPublicKey};
use rand::rngs::OsRng;

fn h(s: &str) -> Handle {
    Handle::parse(s).unwrap()
}
fn nm(s: &str) -> Name {
    Name::parse(s).unwrap()
}
fn sn(s: &str) -> Surname {
    Surname::parse(s).unwrap()
}

/// An Organisation public key for the mock's state: any curve point will do.
fn org_key() -> OrgPublicKey {
    OrgPublicKey::parse(&[0u8; 32]).unwrap()
}

fn store_at(tag: &str) -> (PersonaStore, std::path::PathBuf) {
    let dir = std::env::temp_dir()
        .join(format!("ods-svc-lifecycle-{}-{}", std::process::id(), tag));
    // Cleared first: the directory is keyed on the process id, and a reused
    // pid would otherwise open a store an earlier run left behind. Review
    // round 6 measured exactly that: three unrelated tests failing with a
    // populated store on a mutation run, and the same mutation clean on re-run.
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("store.bin");
    (PersonaStore::open(path.clone(), "pw").unwrap(), path)
}

// verifies: REQ-hzm4kt, LLR-tev8h8, LLR-s7yu4k, LLR-v82xds
#[test]
fn a_new_persona_is_proposed_with_an_id_derived_from_its_member_key() {
    let (store, path) = store_at("persona");
    let mut svc = OrgService::new(store, Box::new(MockChainOps::new()));
    let pid = svc.create_persona(&mut OsRng, h("alice"), nm("Alice"), sn("Smith")).unwrap();

    assert!(!pid.as_str().is_empty());
    assert_eq!(svc.list_personas().len(), 1);
    let rec = &svc.list_personas()[0];
    assert_eq!(rec.status, PersonaStatus::Proposed);
    assert_eq!(rec.org_id, None);
    assert_eq!(rec.member_id, None);

    // The identifier is derived from the member verifying key, so it is
    // reproducible from the stored seed rather than drawn independently.
    let member_kp = rec.member_seed.signing_keypair();
    let expected: String = member_kp
        .verifying_key()
        .as_bytes()
        .iter()
        .take(16)
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(pid.as_str(), expected);

    // The member and device keypairs are independent of each other.
    assert_ne!(rec.member_seed.expose_secret(), rec.device_seed.expose_secret());

    // It survives a reload from disk.
    let reopened = PersonaStore::open(path, "pw").unwrap();
    let svc2 = OrgService::new(reopened, Box::new(MockChainOps::new()));
    assert_eq!(svc2.list_personas().len(), 1);
    assert_eq!(svc2.list_personas()[0].handle.as_str(), "alice");
}

// verifies: LLR-68yd3j, LLR-q3aj8z
#[tokio::test]
async fn creating_an_organisation_advances_the_chain_and_activates_the_persona() {
    let (store, path) = store_at("org");
    let chain = MockChainOps::new();
    let chain_view = chain.clone();
    let mut svc = OrgService::new(store, Box::new(chain));

    let pid = svc.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let org_id = svc.create_organisation(&mut OsRng, &pid).await.unwrap();

    let state = chain_view.get(&org_id).unwrap();
    assert_eq!(state.epoch, Epoch::new(1));
    assert_eq!(svc.list_orgs().len(), 1);
    assert_eq!(svc.list_personas()[0].status, PersonaStatus::Active);
    // LLR-q3aj8z: founding leaves the Persona's member id as it was, which on
    // a fresh Persona is none. The administrator's MemberId is in the record's
    // member snapshots. Added 2026-10-05 by review round 8.
    assert_eq!(svc.list_personas()[0].member_id, None);

    // "…and the store is written before it returns." Added 2026-10-04 after
    // review round 1, which found deleting `self.store.save(rng)?` from
    // `create_organisation` left the whole gate green. Read back from DISK, so
    // a record that only ever existed in memory fails here.
    let reloaded = PersonaStore::open(path, "pw").unwrap();
    assert_eq!(reloaded.data().orgs.len(), 1, "the new org must reach the disk");
    assert_eq!(reloaded.data().orgs[0].org_id, org_id);
    assert_eq!(reloaded.data().orgs[0].epoch, Epoch::new(1));
    // The record holds the genesis root it published. Added 2026-10-04 by
    // review round 7: zeroing it was green, and the app displays it.
    assert_eq!(svc.list_orgs()[0].root_hash, state.root_hash);
    assert_eq!(reloaded.data().orgs[0].root_hash, state.root_hash);
    assert_eq!(
        reloaded.data().personas[0].status,
        PersonaStatus::Active,
        "the persona's activation must be persisted too"
    );
}

// verifies: LLR-hg3xzf
#[test]
fn clones_of_the_mock_chain_share_one_state() {
    let chain = MockChainOps::new();
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
// Disabling the check was green: every story submits at the right epoch.
// Added 2026-10-05 by review round 8.
// verifies: LLR-ryzr8m
#[tokio::test]
async fn the_mock_chain_refuses_an_update_at_the_wrong_epoch() {
    use org_node::ChainOps;
    let chain = MockChainOps::new();
    let org = OrgId::new([6u8; 20]);
    let at_three = OrgState { root_hash: RootHash::new([1u8; 32]), org_pub_key: org_key(), epoch: Epoch::new(3) };
    chain.set(org, at_three);
    for wrong in [Epoch::new(2), Epoch::new(4)] {
        let err = chain.submit_update(org, RootHash::new([9u8; 32]), org_key(), wrong, None).await.unwrap_err();
        assert!(
            matches!(&err, org_node::OrgNodeError::Chain(m) if m.contains("epoch mismatch")),
            "expected epoch {wrong:?} must be refused, got {err:?}"
        );
        assert_eq!(chain.get(&org).unwrap(), at_three, "a refused update leaves the slot");
    }
    chain.submit_update(org, RootHash::new([9u8; 32]), org_key(), Epoch::new(3), None).await.unwrap();
    assert_eq!(chain.get(&org).unwrap().epoch, Epoch::new(4), "the right epoch is accepted and advances");
}

// verifies: LLR-65py3d
#[tokio::test]
async fn the_seam_is_a_trait_object_a_substitute_can_stand_in_for() {
    // `OrgService::new` takes `Box<dyn ChainOps>`, so this compiles only while
    // the seam stays object-safe and substitutable — which is the property
    // SDD-ueh4tm exists for.
    let (store, _) = store_at("seam");
    let boxed: Box<dyn org_node::ChainOps> = Box::new(MockChainOps::new());
    let mut svc = OrgService::new(store, boxed);
    let pid = svc.create_persona(&mut OsRng, h("s"), nm("S"), sn("T")).unwrap();
    assert!(svc.create_organisation(&mut OsRng, &pid).await.is_ok());
}

// Rewritten 2026-10-04 after review round 2. The previous body compared the
// DEVICE KEY returned by the first and second call, which is derived from the
// persona's seed and is therefore the same key whether or not a second
// endpoint was bound: removing the `if self.endpoint.is_none()` guard left
// this test green. The bound socket addresses are what distinguishes one
// endpoint from two — a second bind takes a fresh ephemeral port.
// verifies: REQ-ztdza4, LLR-6zjzn2
#[tokio::test]
async fn the_endpoint_is_bound_once_and_the_same_one_is_returned_after() {
    let (store, _) = store_at("endpoint");
    let mut svc = OrgService::new(store, Box::new(MockChainOps::new()));
    svc.set_transport_mode(TransportMode::Loopback);
    let pid = svc.create_persona(&mut OsRng, h("ep"), nm("Ep"), sn("User")).unwrap();

    let device_seed = svc.list_personas()[0].device_seed.clone();
    let expected = device_seed.signing_keypair().device_key();

    let first_ep = svc.ensure_endpoint(&pid).await.unwrap();
    let first_key = first_ep.device_key();
    let first_sockets = first_ep.inner().bound_sockets();
    assert_eq!(first_key.as_bytes(), expected.as_bytes());
    assert!(!first_sockets.is_empty(), "no sockets bound");

    // A second call returns the SAME endpoint rather than binding another, so
    // it is listening on the same ports. Two binds would differ here.
    let second_ep = svc.ensure_endpoint(&pid).await.unwrap();
    assert_eq!(
        second_ep.inner().bound_sockets(),
        first_sockets,
        "the second call bound a new endpoint instead of returning the first"
    );
    assert_eq!(second_ep.device_key().as_bytes(), first_key.as_bytes());
}

// Added 2026-10-04 after review round 2, which found that LLR-cns6q6's
// load-bearing word — the NAMED persona's device seed — rested on nothing:
// every gated test gave its service exactly one persona, so replacing
// `find_persona(persona_id)` with "the first persona in the store" left the
// whole gate green. The clause that matters is that a device holding more
// than one persona binds the transport identity the caller asked for.
// verifies: REQ-ztdza4, LLR-cns6q6
#[tokio::test]
async fn the_endpoint_binds_the_named_personas_device_not_the_first_personas() {
    let (store, _) = store_at("endpoint-named");
    let mut svc = OrgService::new(store, Box::new(MockChainOps::new()));
    svc.set_transport_mode(TransportMode::Loopback);

    let first_pid = svc.create_persona(&mut OsRng, h("one"), nm("One"), sn("User")).unwrap();
    let second_pid = svc.create_persona(&mut OsRng, h("two"), nm("Two"), sn("User")).unwrap();
    assert_eq!(svc.list_personas().len(), 2);

    let first_seed = svc.list_personas()[0].device_seed.clone();
    let second_seed = svc.list_personas()[1].device_seed.clone();
    assert_ne!(first_seed, second_seed, "two personas must hold distinct device seeds");
    let first_key = first_seed.signing_keypair().device_key();
    let second_key = second_seed.signing_keypair().device_key();

    // Bind for the SECOND persona, which is not the first in the store.
    assert_ne!(second_pid, first_pid);
    let bound = svc.ensure_endpoint(&second_pid).await.unwrap().device_key();
    assert_eq!(
        bound.as_bytes(),
        second_key.as_bytes(),
        "the endpoint must authenticate the named persona's device key"
    );
    assert_ne!(
        bound.as_bytes(),
        first_key.as_bytes(),
        "the endpoint bound the first persona's device key, not the named one"
    );
}

// LLR-ecz9a6 states only the Loopback clause: what this test observes is that
// a service in Loopback mode binds loopback sockets and nothing else. That
// `ensure_endpoint` reads `self.transport_mode` rather than a constant is NOT
// evidenced here and cannot be at this gate — review round 2 found that
// hard-coding `TransportMode::Loopback` in `ensure_endpoint` leaves the whole
// gate green, because `OrgService::new` already sets Loopback and no gated
// test may set Networked (it would reach the public internet). Recorded as a
// gap; the `test-support` relay-injecting constructor closes it.
// verifies: REQ-db6s7q, LLR-ecz9a6
#[tokio::test]
async fn a_service_in_loopback_mode_binds_loopback_sockets_only() {
    let (store, _) = store_at("mode");
    let mut svc = OrgService::new(store, Box::new(MockChainOps::new()));
    svc.set_transport_mode(TransportMode::Loopback);
    let pid = svc.create_persona(&mut OsRng, h("m"), nm("M"), sn("User")).unwrap();

    let ep = svc.ensure_endpoint(&pid).await.unwrap();
    // Loopback mode: every bound socket is a loopback address.
    let sockets = ep.inner().bound_sockets();
    assert!(!sockets.is_empty(), "no sockets bound");
    for s in sockets {
        assert!(s.ip().is_loopback(), "bound a non-loopback address: {s}");
    }
}

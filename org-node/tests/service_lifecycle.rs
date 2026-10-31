#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The chain-operations boundary (SDD-ueh4tm: the state value each
//! chain-judging operation takes, and the test-side `ChainSlots`), and
//! Persona and Organisation genesis (SDD-89es4z). The endpoint lifecycle's
//! tests are org-io's, with the endpoint.
//!
//! The service writes no file and keeps no device seed: "the disk" is the
//! `StoreWrite` an operation returns. The two services over one set of slots
//! found through `support::found_on_chain`.

use org_node::chain::OrgState;
use org_node::ids::OrgId;
use org_node::test_fixtures::ChainSlots;
use org_node::{Epoch, OrgPublicKey, RootHash};
use rand::rngs::OsRng;

mod support;
use support::{found_on_chain, handle, parsed_name, open_store, record_of, parsed_surname, Node};

use support::sweep;

// A new Persona: `propose_founding` and
// `propose_member` each draw a member seed and a Device seed apart, keep
// the member seed and the Device public key and no Device seed, return the
// Device secret by value, and record the Persona Proposed with no member
// identifier (the founding's with no Organisation, the provisional
// Member's bound to its Invite's), its identifier the first sixteen bytes
// of its member key in lowercase hex; the store survives a reload from the
// write it returned.
// verifies: REQ-hzm4kt, LLR-d4gpxm, LLR-6acxu9, LLR-s7yu4k, LLR-t2adzg
#[test]
fn a_new_persona_is_proposed_with_an_id_derived_from_its_member_key() {
    use org_node::store::PersonaStatus;
    use org_node::OrgService;

    let hex16 = |bytes: [u8; 32]| bytes.iter().take(16).map(|byte| format!("{byte:02x}")).collect::<String>();

    let mut svc = OrgService::new(vec![]).unwrap();
    let (device, secret, change) = svc.propose_founding(&mut OsRng, "Alice", "Smith", "alice", "Acme".to_owned()).unwrap();
    let persona = &support::held_store(&svc, &device).persona;
    assert_eq!(
        (persona.status, persona.org_id, persona.member_id),
        (PersonaStatus::Proposed, None, None),
        "LLR-t2adzg: a founding's Persona is unbound"
    );
    assert_eq!(persona.persona_id.as_str(), hex16(persona.member_seed_for_test().x25519_keypair().public_bytes()));
    assert_eq!(persona.device_key, device, "the Device public key is kept");
    assert_eq!(secret.device_key(), device);
    assert_ne!(
        persona.member_seed_for_test().expose_secret().as_slice(),
        support::seed_bytes(&secret).as_slice(),
        "the two seeds are drawn apart"
    );
    let founder = sweep::Side::new(svc, device, secret, change);
    let reloaded = founder.reopened_service();
    assert_eq!(support::held_store(&reloaded, &device).persona.handle.as_ref().map(|handle| handle.as_str()), Some("alice"));

    let (mut inviter, _) = sweep::founded();
    let invite = sweep::invite(&mut inviter, "Bob");
    let bob = sweep::propose_member(&invite, "Bob", &["bob"]);
    let persona = &support::held_store(&bob.svc, &bob.device).persona;
    assert_eq!((persona.status, persona.member_id, persona.handle.is_none()), (PersonaStatus::Proposed, None, true));
    assert_eq!(persona.org_id, Some(invite.org_id), "LLR-t2adzg: a provisional Member's Persona is bound to its Invite's Organisation");
    assert_eq!(persona.persona_id.as_str(), hex16(persona.member_seed_for_test().x25519_keypair().public_bytes()));
    assert_ne!(persona.member_seed_for_test().expose_secret().as_slice(), support::seed_bytes(&bob.secret()).as_slice());
    let reloaded = bob.reopened_service();
    let reloaded_persona = &support::held_store(&reloaded, &bob.device).persona;
    assert_eq!(reloaded_persona.persona_id.as_str(), hex16(reloaded_persona.member_seed_for_test().x25519_keypair().public_bytes()));
}

// LLR-t2adzg: a founding's Persona is recorded Proposed with no Organisation and no
// member id; recording its proxy binds it and gives it no member id; a
// provisional Member's Persona is recorded Proposed, bound to its Invite's
// Organisation, with no member id; only the first adoption in each store
// sets the member id, to the one the adopted record gives that Persona's
// Device, and recording an admission, proposing and a later adoption
// change it in no store.
// verifies: LLR-t2adzg
#[test]
fn only_the_first_adoption_sets_a_personas_member_id() {
    use org_node::store::PersonaStatus;
    use org_node::OrgService;

    // The founding: Proposed, unbound, no member id.
    let mut svc = OrgService::new(vec![]).unwrap();
    let (device, secret, change) = svc.propose_founding(&mut OsRng, "Ada", "Lovelace", "ada", "Acme".to_owned()).unwrap();
    let persona = &support::held_store(&svc, &device).persona;
    assert_eq!((persona.status, persona.org_id, persona.member_id), (PersonaStatus::Proposed, None, None));
    let mut founder = sweep::Side::new(svc, device, secret, change);

    // Its proxy recorded: bound to the Organisation, still no member id.
    let change =
        founder.node.svc.record_founding_proxy(&mut OsRng, &founder.device, support::the_org(), support::the_proxy()).unwrap();
    founder.keep(change);
    let persona = &support::held_store(&founder.svc, &founder.device).persona;
    assert_eq!((persona.status, persona.org_id, persona.member_id), (PersonaStatus::Proposed, Some(support::the_org()), None));

    // The founding's adoption, the first in A's store, sets the founder's member id.
    let founder_id = support::held_store(&founder.svc, &founder.device).founding.as_ref().unwrap().members[0].id;
    let chain = sweep::founding_state(&founder);
    sweep::adopt(&mut founder, chain);
    assert_eq!(support::held_store(&founder.svc, &founder.device).persona.member_id, Some(founder_id), "the first adoption sets it");

    // A provisional Member: Proposed, bound to the Invite's Organisation, no member id.
    let invite = sweep::invite(&mut founder, "Bob");
    let mut bob = sweep::propose_member(&invite, "Bob", &["bob"]);
    let persona = &support::held_store(&bob.svc, &bob.device).persona;
    assert_eq!((persona.status, persona.org_id, persona.member_id), (PersonaStatus::Proposed, Some(invite.org_id), None));

    // Recording Bob's admission and proposing the next epoch set no member id.
    sweep::request_admission(&mut founder, &bob);
    let change = founder.node.svc.record_admission(&mut OsRng, &founder.device, invite.invite_id, None).unwrap();
    founder.keep(change);
    let next = sweep::propose_next(&mut founder);
    assert_eq!(
        support::held_store(&founder.svc, &founder.device).persona.member_id,
        Some(founder_id),
        "recording and proposing change none"
    );
    assert_eq!(support::held_store(&bob.svc, &bob.device).persona.member_id, None, "no adoption in Bob's store yet");

    // A's second adoption keeps A's member id; Bob's first adoption sets his.
    sweep::adopt(&mut founder, next);
    assert_eq!(support::held_store(&founder.svc, &founder.device).persona.member_id, Some(founder_id), "a later adoption leaves it");
    let body = sweep::message_for(&founder, &bob.device);
    sweep::receive_and_adopt(&mut bob, founder.device, &body, next);
    let admitted = sweep::record_of(&bob)
        .trie_members
        .iter()
        .find(|member| member.device_keys.contains(&bob.device))
        .expect("Bob's Device is listed")
        .id;
    assert_eq!(support::held_store(&bob.svc, &bob.device).persona.member_id, Some(admitted), "Bob's first adoption sets his");
}

/// A valid Organisation public key: `OrgPublicKey` is built only through
/// `parse` (REQ-8jb4ny).
fn org_key() -> OrgPublicKey {
    org_node::OrgPrivateKey::from([2u8; 32]).x25519_keypair().org_public_key().unwrap()
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
// No story reaches this check: every story writes the mock chain at the
// right epoch.
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
            matches!(&err, org_node::OrgNodeError::Chain(message) if message.contains("epoch mismatch")),
            "expected epoch {wrong:?} must be refused, got {err:?}"
        );
        assert_eq!(chain.get(&org).unwrap(), at_three, "a refused update leaves the slot");
    }
    chain.apply_update(org, RootHash::new([9u8; 32]), org_key(), Epoch::new(3)).unwrap();
    assert_eq!(chain.get(&org).unwrap().epoch, Epoch::new(4), "the right epoch is accepted and advances");
}

// Two services over one `ChainSlots`: one founds an Organisation and writes
// its slot (`support::found_on_chain`: `propose_founding`, `record_founding_proxy`
// and `verify`), and the other's view of the same slots holds that state.
// verifies: LLR-hg3xzf
#[test]
fn two_services_over_one_set_of_chain_slots_see_the_same_state() {
    let chain = ChainSlots::new();
    let other_view = chain.clone();
    let mut node = Node::new(open_store("lifecycle-two", "a"));
    let other = Node::new(open_store("lifecycle-two", "b"));
    let persona_id = node.propose_and_keep_founding(&mut OsRng, handle("s"), parsed_name("S"), parsed_surname("T")).unwrap();
    let org = found_on_chain(&mut node, &chain, &persona_id);
    let record = record_of(&node, org);
    assert!(other.list_orgs().is_empty(), "the other service holds no record");
    let seen = other_view.get(&org).expect("the other view sees the slot");
    assert_eq!((seen.root_hash, seen.epoch), (record.root_hash, record.epoch));
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

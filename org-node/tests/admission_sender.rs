#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Integration tests: who delivers a message to a node's receive
//! (`check_received`), over the service API, each body delivered by value
//! with its sending Device's key, and a shared `ChainSlots`. Offline — no
//! live chain, no relay.
//!
//! What is decided in the check: a revocation from a Device B's record does
//! not list is refused before anything is used; a Persona's two public keys;
//! and the admission and adoption cases, run through the operations.
//!
//! The gate:
//! `cargo test -p org-node --features app,test-support --test admission_sender`

mod support;
use support::*;

use org_node::error::OrgNodeError;
use org_node::revocation::RevocationNotice;
use org_node::test_fixtures::device_keypair;
use org_node::wire::WireMessage;
use org_node::PersonaId;

// R stands between A and B: B's revocation notice, relayed by R, a Device
// B's record does not list, is refused with `SenderNotListed` before it is
// used; B keeps its record and its Persona. The notice is the one A's
// removal of B would send (B's identity and a proof of its absence from A's
// record without B), built directly; B's admission and removal run through
// the operations (`support::admit_b`, `remove_b`).
// verifies: LLR-283x6f
#[test]
fn a_revocation_relayed_by_a_non_member_is_refused() {
    let story = admit_b(setup("revoke-relay"));
    let b_id = story.node_b.list_personas()[0].member_id.expect("B has a member id");
    let b_device = story.b_device_keypair.device_key().unwrap();
    let (mut story, _) = remove_b(story);
    let without_b = trie_of(&record_of(&story.node_a, story.org_id));
    let notice = RevocationNotice {
        org_id: story.org_id,
        member_id: b_id,
        device: b_device,
        proof: without_b.prove_absent(&b_id, &b_device).unwrap(),
    };
    let before = store_bytes("revoke-relay", "b");
    let b_persona_id = story.b_persona_id.clone();
    let refused =
        receive_message_from(&mut story.node_b, &b_persona_id, ROGUE_SEED, &WireMessage::Revocation(notice)).map(|_| ());
    assert_eq!(refused, Err(OrgNodeError::SenderNotListed { org_id: story.org_id }));
    assert_eq!(story.node_b.list_orgs().len(), 1, "B keeps its record");
    assert_eq!(story.node_b.list_personas().len(), 1, "B keeps its Persona");
    assert_eq!(store_bytes("revoke-relay", "b"), before, "nothing written");
}

// LLR-437fvx: org-node hands the app a Persona's two public keys, each from
// its own seed, and no dialling address; an unknown Persona is refused and
// nothing is written. The absence half (no Invite, no Join request, no
// address) is `org_node_holds_no_invitation_exchange` in tests/absences.rs.
// verifies: LLR-437fvx, LLR-ctzkv7
#[test]
fn persona_public_keys_returns_the_personas_two_keys() {
    let story = setup("persona-keys");
    let persona_b = persona_of(&story.node_b, &story.b_persona_id);

    let (member_key, device_key) = story.node_b.persona_public_keys(&story.b_persona_id).expect("B's own Persona");
    assert_eq!(
        (member_key, device_key),
        (
            persona_b.member_seed_for_test().x25519_keypair().member_key().unwrap(),
            device_keypair(story.node_b.keychain.seed(&persona_b.device_key)).device_key().unwrap()
        ),
        "each key from the Persona's own seed"
    );
    assert_ne!(member_key.as_bytes(), device_key.as_bytes(), "the two keys must be distinct");

    // Abnormal: a Persona this node does not hold is refused, and nothing is
    // written.
    let before = store_bytes("persona-keys", "b");
    let nobody = PersonaId::new("nobody".into());
    assert_eq!(
        story.node_b.persona_public_keys(&nobody).err(),
        Some(OrgNodeError::PersonaIdNotHeld { persona_id: nobody.clone() }),
        "refused naming the Persona identifier"
    );
    assert_eq!(store_bytes("persona-keys", "b"), before, "the refusal writes nothing");
}

// LLR-exaa9u, LLR-f9x3vf: a Persona is Proposed until the first adoption in
// its own store sets it Active (the founding's for a founder, a first
// admission's for a provisional Member); its Organisation is the one it was
// given and does not change; and an operation on one store of the service
// changes no other store. Two stores with one Device key are refused, and an
// unknown Device key names no store.
// verifies: LLR-exaa9u, LLR-f9x3vf
#[test]
fn the_first_adoption_marks_its_persona_active_and_changes_no_other_store() {
    use org_node::store::PersonaStatus;
    use org_node::verify_org::Verified;
    use org_node::{OrgId, OrgService, SigningKeypair};
    use rand::rngs::OsRng;

    // A founder: Proposed, bound once its proxy is recorded, Active at the
    // founding's adoption.
    let mut founder = sweep::found();
    let persona = &support::held_store(&founder.svc, &founder.device).persona;
    assert_eq!((persona.status, persona.org_id), (PersonaStatus::Proposed, Some(support::the_org())));
    let chain = sweep::founding_state(&founder);
    let adopted = sweep::verify(&mut founder, chain);
    assert!(matches!(adopted, Verified::Adopted { own: true, .. }), "{adopted:?}");
    let persona = &support::held_store(&founder.svc, &founder.device).persona;
    assert_eq!((persona.status, persona.org_id), (PersonaStatus::Active, Some(support::the_org())));

    // A provisional Member of a second Organisation, its store held by
    // the same service as the founder's founded store.
    let other = OrgId::new([0x0b; 20]);
    let (mut other_founder, _) = sweep::founded_at(other);
    let pending = sweep::admission(&mut other_founder, "Bea", "bea");
    let mut both = OrgService::new(vec![founder.store_on_disk(), pending.member.store_on_disk()]).unwrap();
    let bea = pending.member.device;
    assert_eq!(support::held_store(&both, &bea).persona.status, PersonaStatus::Proposed);
    assert_eq!(support::held_store(&both, &bea).persona.org_id, Some(other));
    let founder_before = support::encoded_store(&both, &founder.device);

    // Bea's first adoption, in the shared service: Bea is Active and bound
    // to the Organisation she was given; the founder's store is
    // byte-for-byte as it was.
    let (checked, change) = both.check_received(&mut OsRng, &bea, other_founder.device, &pending.body).unwrap();
    assert!(matches!(checked, org_node::service::Checked::ProvisionalOrganisationKept { .. }), "{checked:?}");
    assert!(change.is_some(), "the kept provisional organisation is sealed");
    let (verified, change) = both.verify(&mut OsRng, &bea, Some(pending.chain), support::story_time(), org_node::AheadLimit(1200), support::never_asked).unwrap();
    assert!(matches!(verified, Verified::Adopted { own: false, .. }), "{verified:?}");
    assert!(change.is_some(), "the first adoption is sealed");
    let persona = &support::held_store(&both, &bea).persona;
    assert_eq!((persona.status, persona.org_id), (PersonaStatus::Active, Some(other)));
    assert_eq!(support::encoded_store(&both, &founder.device), founder_before, "A's store is unchanged by Bea's adoption");
    let persona_a = &support::held_store(&both, &founder.device).persona;
    assert_eq!((persona_a.status, persona_a.org_id), (PersonaStatus::Active, Some(support::the_org())), "never rebound");

    // LLR-f9x3vf: two stores with one Device key, and an unknown key.
    let duplicate = OrgService::new(vec![other_founder.store_on_disk(), other_founder.store_on_disk()]);
    assert!(matches!(duplicate, Err(OrgNodeError::DuplicatePersonaStore)), "{:?}", duplicate.err());
    let unknown = SigningKeypair::generate(&mut OsRng).device_key().unwrap();
    let refused = both.propose_updated_organisation(&mut OsRng, &unknown);
    assert!(matches!(refused, Err(OrgNodeError::PersonaNotHeld { .. })), "{:?}", refused.err());
}

// LLR-x5asbx: the proxy account `record_founding_proxy` is given is held in
// the provisional founding, becomes the record's at the founding's adoption,
// never reaches the disk in clear, is handed back unchanged by
// `proxy_account`, and no admission, revocation, proposal or later adoption
// changes it; a record a first admission created holds none.
// verifies: LLR-x5asbx
#[test]
fn the_proxy_account_recorded_for_a_founding_is_kept_and_handed_back() {
    use rand::rngs::OsRng;

    let mut founder = sweep::found();
    let founding = support::held_store(&founder.svc, &founder.device).founding.as_ref().expect("a provisional founding");
    assert_eq!(founding.proxy, Some(support::the_proxy()), "held in the provisional founding");
    let sealed = founder.last_write().sealed.as_bytes().to_vec();
    assert!(!support::contains(&sealed, support::the_proxy().as_bytes()), "never on disk in clear");

    let chain = sweep::founding_state(&founder);
    sweep::adopt(&mut founder, chain);
    assert_eq!(sweep::record_of(&founder).proxy_account, Some(support::the_proxy()), "the record's from the adoption");
    assert_eq!(founder.svc.proxy_account(support::the_org()).unwrap(), Some(support::the_proxy()));

    let mut bob = sweep::admission(&mut founder, "Bob", "bob");
    assert_eq!(founder.svc.proxy_account(support::the_org()).unwrap(), Some(support::the_proxy()), "an admission leaves it");
    sweep::receive_and_adopt(&mut bob.member, founder.device, &bob.body, bob.chain);
    assert_eq!(bob.member.svc.proxy_account(support::the_org()).unwrap(), None, "a first admission's record holds none");

    let b_id = sweep::member_id_of(&bob.member);
    let change = founder.node.svc.record_revocation(&mut OsRng, &founder.device, b_id).unwrap();
    founder.keep(change);
    assert_eq!(
        founder.svc.proxy_account(support::the_org()).unwrap(),
        Some(support::the_proxy()),
        "a recorded revocation leaves it"
    );
    let next = sweep::propose_next(&mut founder);
    assert_eq!(founder.svc.proxy_account(support::the_org()).unwrap(), Some(support::the_proxy()), "a proposal leaves it");
    sweep::adopt(&mut founder, next);
    assert_eq!(
        sweep::record_of(&founder).proxy_account,
        Some(support::the_proxy()),
        "an adoption that replaces the record keeps it"
    );

    let sealed = founder.last_write().sealed.as_bytes().to_vec();
    assert!(!support::contains(&sealed, support::the_proxy().as_bytes()), "still never in clear");
    let reopened = founder.reopened_service();
    assert_eq!(reopened.proxy_account(support::the_org()).unwrap(), Some(support::the_proxy()), "and read back from the sealed store");
}

// Normal case of REQ-d9g6nt: the founder's MemberId and an admitted Member's
// are drawn, not computed from a key, and differ from each other; two
// foundings with the same details give two founders two identifiers
// (LLR-rjg3m2).
// verifies: REQ-d9g6nt, LLR-4rv4kr, LLR-rjg3m2
#[test]
fn member_ids_are_not_derived_from_keys() {
    let (mut founder_side, _) = sweep::founded();
    let founder = sweep::record_of(&founder_side).trie_members[0].clone();
    assert_eq!(support::held_store(&founder_side.svc, &founder_side.device).persona.member_id, Some(founder.id));
    assert_ne!(founder.id.as_bytes(), founder.member_key.as_bytes(), "the founder's id is its member key");
    assert_ne!(founder.id.as_bytes(), founder.device_keys[0].as_bytes(), "the founder's id is its device key");

    let mut bob = sweep::admission(&mut founder_side, "Bob", "bob");
    sweep::receive_and_adopt(&mut bob.member, founder_side.device, &bob.body, bob.chain);
    let admitted = sweep::record_of(&founder_side)
        .trie_members
        .iter()
        .find(|member| member.device_keys.contains(&bob.member.device))
        .expect("Bob is in A's record")
        .clone();
    assert_ne!(admitted.id.as_bytes(), admitted.member_key.as_bytes(), "Bob's id is his member key");
    assert_ne!(admitted.id.as_bytes(), admitted.device_keys[0].as_bytes(), "Bob's id is his device key");
    assert_ne!(admitted.id, founder.id, "Bob's id equals the founder's");
    assert_eq!(sweep::member_id_of(&bob.member), admitted.id, "Bob's Persona holds the id A's record gives him");

    let first = sweep::found();
    let second = sweep::found();
    let id_of = |side: &sweep::Side| support::held_store(&side.svc, &side.device).founding.as_ref().unwrap().members[0].id;
    assert_ne!(id_of(&first), id_of(&second), "two foundings, two founder identifiers");
}

// Abnormal case of REQ-d9g6nt: the same kept admission request, with the
// same Member-as-a-group key and DevicePublicKey, proposed twice gets two
// identifiers; the adopted one is later revoked; and no identifier is ever
// issued twice across three rounds.
// verifies: REQ-d9g6nt, LLR-4rv4kr
#[test]
fn readmission_with_same_keys_gets_a_fresh_member_id() {
    use org_node::MemberId;
    use rand::rngs::OsRng;

    let (mut founder, _) = sweep::founded();
    let mut issued: Vec<MemberId> = vec![sweep::record_of(&founder).trie_members[0].id];
    for round in 0..3 {
        let invite = sweep::invite(&mut founder, "Bob");
        let bob = sweep::propose_member(&invite, "Bob", &["bob"]);
        let member = support::held_store(&bob.svc, &bob.device).member.as_ref().unwrap();
        let (member_key, device) = (member.member_key, member.device_key);
        sweep::request_admission(&mut founder, &bob);
        let change = founder.node.svc.record_admission(&mut OsRng, &founder.device, invite.invite_id, None).unwrap();
        founder.keep(change);
        let first = sweep::propose_next(&mut founder);
        let second = sweep::propose_next(&mut founder);
        let ids = [sweep::proposed_id_of(&founder, first, &device), sweep::proposed_id_of(&founder, second, &device)];
        assert_ne!(ids[0], ids[1], "round {round}: the same keys proposed twice got one id");
        for id in ids {
            assert!(!issued.contains(&id), "round {round}: an id was issued again");
            assert_ne!(id.as_bytes(), member_key.as_bytes(), "round {round}: id is the member key");
            assert_ne!(id.as_bytes(), device.as_bytes(), "round {round}: id is the device key");
            issued.push(id);
        }
        sweep::adopt(&mut founder, second);
        assert!(
            sweep::record_of(&founder).trie_members.iter().any(|member| member.id == ids[1]),
            "round {round}: adopted under the second id"
        );
        sweep::revocation(&mut founder, ids[1]);
        assert!(
            !sweep::record_of(&founder).trie_members.iter().any(|member| member.id == ids[1]),
            "round {round}: revoked"
        );
    }
}

// LLR-95av3e, LLR-j6j95z: a first admission is attempted only from
// Organisation information. A's genuine message to provisional Member B,
// relabelled a revocation, is refused as `OrgNotHeld` by a store holding no
// record, with nothing kept, the expectation in place and the Persona
// Proposed; the genuine message is then kept, its snapshot decoded, and
// adopted.
// verifies: REQ-d9g6nt, REQ-vxqc5g, LLR-j6j95z, LLR-95av3e
#[test]
fn a_revocation_about_an_organisation_not_held_is_refused_before_the_chain() {
    use org_node::store::PersonaStatus;
    use org_node::verify_org::Verified;
    use rand::rngs::OsRng;

    let (mut founder, _) = sweep::founded();
    let mut pending = sweep::admission(&mut founder, "Bob", "bob");
    let mut relabelled = pending.body.clone();
    relabelled[0] = 1; // the Revocation variant's index (LLR-xr88a8); the Organisation identifier follows unchanged
    let before = support::encoded_store(&pending.member.svc, &pending.member.device);
    let refused = pending.member.node.svc.check_received(&mut OsRng, &pending.member.device, founder.device, &relabelled);
    assert_eq!(refused.unwrap_err(), OrgNodeError::OrgNotHeld { org_id: support::the_org() });
    assert_eq!(support::encoded_store(&pending.member.svc, &pending.member.device), before, "nothing kept");
    let bob_store = support::held_store(&pending.member.svc, &pending.member.device);
    assert!(bob_store.record.is_none(), "no record");
    assert_eq!(bob_store.expected_admissions.len(), 1, "the expectation is kept");
    assert_eq!(bob_store.persona.status, PersonaStatus::Proposed);

    let verified = sweep::receive_and_adopt(&mut pending.member, founder.device, &pending.body, pending.chain);
    assert!(matches!(verified, Verified::Adopted { .. }), "{verified:?}");
    assert_eq!(sweep::record_of(&pending.member).org_id, support::the_org());
}

// LLR-34c5y3, LLR-degg8t: A's record takes B's removal only through the
// adoption of the provisional organisation the chain carries, and the
// adoption reaches A's disk: one member, the chain's epoch and root, the
// Sequence number equal to the epoch. The removed Device's message for
// the epoch is a Revocation holding its notice and nothing else.
// verifies: LLR-34c5y3, LLR-degg8t, REQ-mr5abb
#[test]
fn a_revocation_reaches_the_administrators_disk() {
    use org_node::wire::decode_message;
    use org_node::SequenceNumber;

    let (mut founder, _) = sweep::founded();
    let mut bob = sweep::admission(&mut founder, "Bob", "bob");
    sweep::receive_and_adopt(&mut bob.member, founder.device, &bob.body, bob.chain);
    let b_id = sweep::member_id_of(&bob.member);
    assert_eq!(sweep::record_of(&founder).trie_members.len(), 2, "A and B before the revocation");

    let chain = sweep::revocation(&mut founder, b_id);
    let record = sweep::record_of(&founder);
    assert_eq!(record.trie_members.len(), 1, "A holds itself only");
    assert_eq!((record.epoch, record.root_hash, record.org_pub_key), (chain.epoch, chain.root_hash, chain.org_pub_key));
    assert_eq!(record.last_seq, SequenceNumber::new(chain.epoch.get()), "the mark is the adopted epoch");

    let messages = founder.svc.messages_for_current_epoch(&founder.device).unwrap();
    let to_b =
        messages.iter().find(|message| message.recipient == bob.member.device).expect("a message for the removed Device");
    let WireMessage::Revocation(notice) = decode_message(&to_b.body).unwrap() else {
        panic!("the removed Device is sent a Revocation")
    };
    assert_eq!((notice.org_id, notice.member_id, notice.device), (support::the_org(), b_id, bob.member.device));

    let reopened = founder.reopened_service();
    let on_disk = support::held_store(&reopened, &founder.device).record.as_ref().expect("the record on disk");
    assert_eq!(on_disk.trie_members.len(), 1, "the removal reached the disk");
    assert!(!on_disk.trie_members.iter().any(|member| member.id == b_id));
    assert_eq!(
        (on_disk.epoch, on_disk.root_hash, on_disk.last_seq),
        (chain.epoch, chain.root_hash, SequenceNumber::new(chain.epoch.get()))
    );
}

// LLR-34c5y3, LLR-g2jeqd: the messages of the adopted removal are built and
// never delivered; the adopted removal stays in A's record, in memory and on
// disk, and building the messages twice gives the same bytes and changes
// nothing: whether an epoch's messages are ever sent changes neither the
// outcome nor the store.
// verifies: LLR-34c5y3, LLR-g2jeqd
#[test]
fn building_the_revocation_messages_leaves_the_adopted_removal_in_place() {
    let (mut founder, _) = sweep::founded();
    let mut bob = sweep::admission(&mut founder, "Bob", "bob");
    sweep::receive_and_adopt(&mut bob.member, founder.device, &bob.body, bob.chain);
    let b_id = sweep::member_id_of(&bob.member);
    let chain = sweep::revocation(&mut founder, b_id);

    let before = support::encoded_store(&founder.svc, &founder.device);
    let bodies = |side: &sweep::Side| -> Vec<Vec<u8>> {
        side.svc.messages_for_current_epoch(&side.device).unwrap().into_iter().map(|message| message.body).collect()
    };
    let first = bodies(&founder);
    assert!(!first.is_empty(), "B's revocation is built");
    let again = bodies(&founder);
    assert_eq!(again, first, "building the messages twice gives the same bytes");
    drop(first); // never delivered
    assert_eq!(support::encoded_store(&founder.svc, &founder.device), before, "building the messages changes nothing");
    assert!(!sweep::record_of(&founder).trie_members.iter().any(|member| member.id == b_id), "the adopted removal stays");
    assert_eq!(sweep::record_of(&founder).epoch, chain.epoch);
    let reopened = founder.reopened_service();
    let on_disk = support::held_store(&reopened, &founder.device).record.as_ref().unwrap();
    assert!(!on_disk.trie_members.iter().any(|member| member.id == b_id), "and on A's disk");
    assert_eq!(on_disk.epoch, chain.epoch);
}

// A first admission writes the chain's Organisation public key (LLR-rys5nx:
// the key of the chain state handed to `verify`, never a value the message
// carried), the Organisation private key the message carried, which A's
// record holds (LLR-qw8tyu), and the Persona's member id, status and
// Organisation (LLR-exaa9u: Active at the first adoption in its store), in
// memory and on disk.
// verifies: LLR-exaa9u, LLR-rys5nx, LLR-qw8tyu
#[test]
fn a_first_admission_records_the_chains_key_the_private_key_and_the_member() {
    use org_node::store::PersonaStatus;

    let (mut founder, _) = sweep::founded();
    let mut bob = sweep::admission(&mut founder, "Bob", "bob");
    sweep::receive_and_adopt(&mut bob.member, founder.device, &bob.body, bob.chain);
    let held_by_a = sweep::record_of(&founder).org_private_key_for_test().clone();
    let record = sweep::record_of(&bob.member);
    assert_eq!(record.org_pub_key, bob.chain.org_pub_key, "the chain's key");
    assert_eq!(record.org_private_key_for_test().clone(), held_by_a, "the key the message carried, A's");
    assert_eq!(held_by_a.x25519_keypair().org_public_key().unwrap(), bob.chain.org_pub_key, "the private half of the chain's key");
    let member = record
        .trie_members
        .iter()
        .find(|member| member.device_keys.contains(&bob.member.device))
        .expect("B's Device is listed");
    let persona = &support::held_store(&bob.member.svc, &bob.member.device).persona;
    assert_eq!(persona.member_id, Some(member.id));
    assert_eq!(persona.status, PersonaStatus::Active);
    assert_eq!(persona.org_id, Some(support::the_org()));

    let reopened = bob.member.reopened_service();
    let on_disk = support::held_store(&reopened, &bob.member.device).record.as_ref().unwrap();
    assert_eq!(on_disk.org_pub_key, bob.chain.org_pub_key);
    assert_eq!(on_disk.org_private_key_for_test().clone(), held_by_a);
}

// Regression: an admitted Member holds the Organisation private key, the one
// the founder's record holds and the private half of the chain's key.
// verifies: LLR-qw8tyu, REQ-szq3ud, REQ-ju6vn2
#[test]
fn pr_szkat6_an_admitted_member_holds_the_organisation_private_key() {
    let (mut founder, _) = sweep::founded();
    let mut bob = sweep::admission(&mut founder, "Bob", "bob");
    sweep::receive_and_adopt(&mut bob.member, founder.device, &bob.body, bob.chain);
    let held = sweep::record_of(&bob.member).org_private_key_for_test().clone();
    assert_eq!(held, sweep::record_of(&founder).org_private_key_for_test().clone(), "the key the founder holds");
    assert_eq!(held.x25519_keypair().org_public_key().unwrap(), bob.chain.org_pub_key);
}

// Regression: Organisation information whose Organisation private key a relay
// substituted is refused from the relay (`SenderNotListed`) and, from A's
// listed Device, kept under the public key derived from the substituted key:
// no match for the chain, so verification reports Behind and B's record and
// both keys stay as they were. The genuine message is then kept and adopted
// with the chain's key. Which record is adopted is decided by the chain
// state alone, never by the delivering Device.
// verifies: LLR-8528ud, LLR-qw8tyu, REQ-mr5abb, LLR-4tsgfh
#[test]
fn pr_ve9zw8_an_update_carrying_a_substituted_key_is_refused_from_a_relay_and_not_adopted_from_a_listed_device() {
    use org_node::service::Checked;
    use org_node::verify_org::Verified;
    use org_node::wire::{decode_message, encode_message};
    use org_node::OrgPrivateKey;
    use rand::rngs::OsRng;

    let (mut founder, _) = sweep::founded();
    let mut bob = sweep::admission(&mut founder, "Bob", "bob");
    sweep::receive_and_adopt(&mut bob.member, founder.device, &bob.body, bob.chain);
    let mut bob = bob.member;
    let carol = sweep::admission(&mut founder, "Carol", "carol");
    let genuine = sweep::message_for(&founder, &bob.device);
    let WireMessage::OrgInformation { envelope, record_snapshot, .. } = decode_message(&genuine).unwrap() else {
        panic!("Organisation information")
    };
    let substituted = encode_message(&WireMessage::OrgInformation {
        envelope,
        record_snapshot,
        org_private_key: OrgPrivateKey::from([0x42u8; 32]),
    })
    .unwrap();
    let held = sweep::record_of(&bob).clone();
    let before = support::encoded_store(&bob.svc, &bob.device);

    let from_relay = bob.node.svc.check_received(&mut OsRng, &bob.device, support::relay_device(), &substituted);
    assert_eq!(from_relay.unwrap_err(), OrgNodeError::SenderNotListed { org_id: support::the_org() });
    assert_eq!(support::encoded_store(&bob.svc, &bob.device), before, "nothing kept from the relay");

    let checked = sweep::receive(&mut bob, founder.device, &substituted);
    assert!(matches!(checked, Checked::ProvisionalOrganisationKept { .. }), "{checked:?}");
    let (verified, change) = bob.node.svc.verify(&mut OsRng, &bob.device, Some(carol.chain), support::story_time(), org_node::AheadLimit(1200), support::never_asked).unwrap();
    assert!(matches!(verified, Verified::Behind { .. }), "no match for the chain: {verified:?}");
    // The substituted one, at the chain's epoch and no match, can never be
    // adopted, so the verification discards it (LLR-4tsgfh, merge review MR3-N1).
    bob.keep(change.expect("the discard is sealed"));
    assert!(support::held_store(&bob.svc, &bob.device).provisional_organisations.is_empty(), "the substitute is discarded");
    let after = sweep::record_of(&bob);
    assert_eq!(
        (after.epoch, after.last_seq, after.root_hash, after.org_pub_key),
        (held.epoch, held.last_seq, held.root_hash, held.org_pub_key)
    );
    assert_eq!(after.org_private_key_for_test().clone(), held.org_private_key_for_test().clone());

    let verified = sweep::receive_and_adopt(&mut bob, founder.device, &genuine, carol.chain);
    assert!(matches!(verified, Verified::Adopted { own: false, .. }), "{verified:?}");
    assert_eq!(sweep::record_of(&bob).org_pub_key, carol.chain.org_pub_key);
    assert_eq!(
        sweep::record_of(&bob).org_private_key_for_test().x25519_keypair().org_public_key().unwrap(),
        carol.chain.org_pub_key
    );
}

// LLR-94rzrk: each adopted epoch gives B's record the key pair A's record
// holds, the public half the chain's, in memory and on disk; the same
// message with its key cut off or one byte short does not decode and is
// refused before anything is kept.
// verifies: LLR-94rzrk, REQ-ju6vn2
#[test]
fn each_adopted_update_gives_the_record_the_new_key_pair() {
    use rand::rngs::OsRng;

    let (mut founder, _) = sweep::founded();
    let mut bob = sweep::admission(&mut founder, "Bob", "bob");
    sweep::receive_and_adopt(&mut bob.member, founder.device, &bob.body, bob.chain);
    let mut bob = bob.member;
    for name in ["Carol", "Dave"] {
        let next = sweep::admission(&mut founder, name, &name.to_lowercase());
        let body = sweep::message_for(&founder, &bob.device);
        let before = support::encoded_store(&bob.svc, &bob.device);
        for cut in [body[..body.len() - 32].to_vec(), body[..body.len() - 1].to_vec()] {
            let refused = bob.node.svc.check_received(&mut OsRng, &bob.device, founder.device, &cut);
            assert_eq!(refused.unwrap_err(), OrgNodeError::MalformedMessage, "{name}: {} bytes", cut.len());
        }
        assert_eq!(support::encoded_store(&bob.svc, &bob.device), before, "{name}: nothing kept from the cut bodies");
        sweep::receive_and_adopt(&mut bob, founder.device, &body, next.chain);
        let record = sweep::record_of(&bob);
        assert_eq!(record.org_pub_key, next.chain.org_pub_key, "{name}");
        assert_eq!(
            record.org_private_key_for_test().clone(),
            sweep::record_of(&founder).org_private_key_for_test().clone(),
            "{name}"
        );
        assert_eq!(record.org_private_key_for_test().x25519_keypair().org_public_key().unwrap(), record.org_pub_key, "{name}: a pair");
    }
    let reopened = bob.reopened_service();
    let on_disk = support::held_store(&reopened, &bob.device).record.as_ref().unwrap();
    assert_eq!(on_disk.org_pub_key, sweep::record_of(&founder).org_pub_key, "on disk");
    assert_eq!(
        on_disk.org_private_key_for_test().clone(),
        sweep::record_of(&founder).org_private_key_for_test().clone(),
        "on disk"
    );
}

// Regression: A's genuine Organisation information about C's admission,
// relayed to B by a Device B's record does not list, is refused with
// `SenderNotListed` with nothing kept and no change; the same body from A's
// listed Device is kept and adopted.
// verifies: LLR-283x6f
#[test]
fn pr_u4c2vp_an_update_relayed_by_a_non_member_is_refused() {
    use rand::rngs::OsRng;

    let (mut founder, _) = sweep::founded();
    let mut bob = sweep::admission(&mut founder, "Bob", "bob");
    sweep::receive_and_adopt(&mut bob.member, founder.device, &bob.body, bob.chain);
    let mut bob = bob.member;
    let epoch_before = sweep::record_of(&bob).epoch;
    let carol = sweep::admission(&mut founder, "Carol", "carol");
    let body = sweep::message_for(&founder, &bob.device);
    let before = support::encoded_store(&bob.svc, &bob.device);
    let refused = bob.node.svc.check_received(&mut OsRng, &bob.device, support::relay_device(), &body);
    assert_eq!(refused.unwrap_err(), OrgNodeError::SenderNotListed { org_id: support::the_org() });
    assert_eq!(support::encoded_store(&bob.svc, &bob.device), before, "nothing kept");
    assert!(support::held_store(&bob.svc, &bob.device).provisional_organisations.is_empty());
    sweep::receive_and_adopt(&mut bob, founder.device, &body, carol.chain);
    assert!(sweep::record_of(&bob).epoch > epoch_before, "from A's listed Device it is adopted");
}

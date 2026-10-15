#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The one receive path, by values (SDD-esg2kc; S4, task T4 of
//! docs/plans/2026-10-06-org-io-transport.md): `check_received(sender,
//! body)` decodes the body and runs every check that needs no chain state,
//! returning either an acknowledgement it decided or a `CheckedMessage`
//! naming the Organisation to read; `commit_checked` decides that message
//! against the state read for it and returns the outcome and the one store
//! change. No endpoint: each body is the one the sending node's commit built
//! for the recipient (`support::body_for`, since S4 task T5), or one built
//! here with `encode_message`, and the sender is a Device key the test names.

mod support;
use support::*;

use std::cell::Cell;
use std::path::Path;

use org_members::DevicePublicKey;
use org_node::chain::OrgState;
use org_node::error::OrgNodeError;
use org_node::keys::DeviceSecret;
use org_node::service::{Checked, CheckedMessage, OrgService, ReceiveKind, ReceiveOutcome, StoreChange};
use org_node::test_fixtures::{bob_absence_notice, device_keypair};
use org_node::wire::{decode_message, encode_message, WireMessage};
use org_node::{Epoch, OrgPrivateKey, PersonaId};
use rand::rngs::OsRng;

/// The Device of the rogue relay: listed in no record.
fn rogue() -> DevicePublicKey {
    device_keypair(ROGUE_SEED).device_key().unwrap()
}

/// The Device key of `persona_id` on `node`.
fn device_of(node: &Node, persona_id: &PersonaId) -> DevicePublicKey {
    node.persona_public_keys(persona_id).unwrap().1
}

/// The plaintext of every store `svc` holds, as the store encodes it.
fn plaintext(svc: &OrgService) -> Vec<Vec<u8>> {
    svc.list_personas()
        .iter()
        .map(|persona| postcard::to_allocvec(svc.store_data(&persona.persona_id).unwrap()).unwrap())
        .collect()
}

/// `checked`'s message, read for: `commit_checked` against `state`, with
/// `node`'s keychain as the secret source; also returns whether the source
/// was called.
fn commit(
    node: &mut Node,
    checked: CheckedMessage,
    state: Option<OrgState>,
) -> (Result<(ReceiveOutcome, StoreChange), OrgNodeError>, bool) {
    let asked = Cell::new(false);
    let keychain = node.keychain.clone();
    let result = node.svc.commit_checked(&mut OsRng, checked, state, |keys: &[DevicePublicKey]| -> Vec<DeviceSecret> {
        asked.set(true);
        keychain.secrets(keys)
    });
    (result, asked.get())
}

/// `check_received` must return a message to read; the chain's state for it.
fn read_chain(node: &Node, sender: DevicePublicKey, body: &[u8]) -> (CheckedMessage, Option<OrgState>) {
    match node.svc.check_received(sender, body) {
        Ok(Checked::ReadChain(checked)) => {
            let state = node.view.read(checked.org_id()).unwrap();
            (checked, state)
        }
        Ok(Checked::Acknowledged(verified)) => panic!("decided as an acknowledgement: {verified:?}"),
        Err(error) => panic!("refused in the check: {error:?}"),
    }
}

/// A refusal changed nothing: no store change returned, the secret source
/// never called, every store's plaintext as before.
fn refused_with_nothing_changed(
    result: (Result<(ReceiveOutcome, StoreChange), OrgNodeError>, bool),
    before: &[Vec<u8>],
    node: &Node,
) -> OrgNodeError {
    let (result, asked) = result;
    assert!(!asked, "the secret source is not called on a refusal");
    assert_eq!(plaintext(&node.svc), before, "the stores are unchanged");
    match result {
        Ok((outcome, change)) => panic!("committed: {outcome:?}, {change:?}"),
        Err(error) => error,
    }
}

/// A's admission of B, committed, and its body to B from A's Device.
fn admission_body(s: &mut Setup) -> (DevicePublicKey, Vec<u8>) {
    let joiner = s.joiner_b.clone();
    let (_, outcome) = commit_admission(&mut s.svc_a, &s.chain, s.org_id, &joiner).unwrap();
    (device_of(&s.svc_a, &s.pid_a), body_for(&outcome, joiner.device_key).unwrap())
}

/// B admitted (epoch 2), then A's admission of C, committed, and its body
/// to B from A's Device.
fn update_body(tag: &str) -> (Setup, DevicePublicKey, Vec<u8>) {
    let mut s = admit_b_directly(setup(tag));
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let (_, outcome) = commit_admission(&mut s.svc_a, &s.chain, s.org_id, &joiner_c).unwrap();
    let body = body_for(&outcome, s.joiner_b.device_key).unwrap();
    let sender = device_of(&s.svc_a, &s.pid_a);
    (s, sender, body)
}

/// B admitted (epoch 2), then A's removal of B, committed (epoch 3): the
/// setup, the chain's state before the removal, and the notice body A sends
/// B's Device, from A's Device.
fn notice_body(tag: &str) -> (Setup, OrgState, DevicePublicKey, Vec<u8>) {
    let mut s = admit_b_directly(setup(tag));
    let before = s.chain.get(&s.org_id).unwrap();
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let outcome = commit_removal(&mut s.svc_a, &s.chain, s.org_id, b_id).unwrap();
    let body = body_for(&outcome, s.joiner_b.device_key).unwrap();
    assert!(matches!(decode_message(&body).unwrap(), WireMessage::Revocation(_)));
    let sender = device_of(&s.svc_a, &s.pid_a);
    (s, before, sender, body)
}

// ---- check_received ------------------------------------------------------------

// verifies: LLR-zetn7a
#[test]
fn a_body_that_does_not_decode_is_malformed_and_changes_nothing() {
    let s = setup("rp-malformed");
    let before = (plaintext(&s.svc_b), store_bytes("rp-malformed", "b"));
    for body in [&[][..], &[0xff, 0x00, 0x01][..], &[0x07; 64][..]] {
        assert!(matches!(s.svc_b.check_received(rogue(), body), Err(OrgNodeError::MalformedMessage)));
    }
    assert_eq!((plaintext(&s.svc_b), store_bytes("rp-malformed", "b")), before);
    assert!(s.svc_b.expected_admissions().len() == 1, "the expectation stays");
}

// B expects an admission to the Organisation and holds no record of it: a
// notice for it is refused as not held, not as an unexpected admission.
// verifies: LLR-zetn7a, LLR-38e2kn
#[test]
fn an_unheld_revocation_is_refused_before_any_expectation_check() {
    let s = setup("rp-unheld-notice");
    let body = encode_message(&WireMessage::Revocation(bob_absence_notice(s.org_id))).unwrap();
    let refused = s.svc_b.check_received(device_of(&s.svc_a, &s.pid_a), &body).map(|_| ());
    assert_eq!(refused, Err(OrgNodeError::RevocationNotHeld { org_id: s.org_id }));
}

// verifies: LLR-zetn7a, LLR-s8xp7m
#[test]
fn an_unexpected_first_admission_is_refused() {
    let mut s = setup_with("rp-unexpected", false);
    let (sender, body) = admission_body(&mut s);
    let refused = s.svc_b.check_received(sender, &body).map(|_| ());
    assert_eq!(refused, Err(OrgNodeError::AdmissionNotExpected { org_id: s.org_id }));
}

// verifies: LLR-zetn7a
#[test]
fn the_checked_message_names_the_organisation_to_read() {
    let mut s = setup("rp-names-org");
    let (sender, body) = admission_body(&mut s);
    let Ok(Checked::ReadChain(checked)) = s.svc_b.check_received(sender, &body) else { panic!("a message to read") };
    assert_eq!(checked.org_id(), s.org_id);
    let printed = format!("{checked:?}");
    assert!(printed.contains("CheckedMessage"), "{printed}");
}

/// The text of `org-node/src/service.rs`.
fn service_source() -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/service.rs")).unwrap()
}

/// The body of the item opened by the line `opening` in `source`, up to its
/// closing brace in column 0.
fn item_body<'a>(source: &'a str, opening: &str) -> &'a str {
    let start = source.find(opening).unwrap_or_else(|| panic!("no `{opening}`"));
    let rest = &source[start..];
    &rest[..rest.find("\n}").unwrap()]
}

// The only way to a `CheckedMessage` is `check_received`: it has no public
// field, and no public function outside `impl OrgService` returns one.
// verifies: LLR-zetn7a
#[test]
fn a_checked_message_has_no_public_constructor() {
    let source = service_source();
    let fields = item_body(&source, "pub struct CheckedMessage {");
    assert!(!fields.lines().skip(1).any(|line| line.trim_start().starts_with("pub ")), "a public field:\n{fields}");
    let service = item_body(&source, "impl OrgService {");
    let outside = source.replace(service, "");
    for (index, line) in outside.lines().enumerate() {
        let line = line.trim_start();
        if line.starts_with("pub fn") || line.starts_with("pub(crate) fn") || line.starts_with("pub fn") {
            assert!(!line.contains("CheckedMessage"), "service.rs:{}: a constructor outside OrgService: {line}", index + 1);
        }
    }
    let returning: Vec<&str> =
        service.lines().filter(|line| line.contains("pub fn") && line.contains("-> Result<Checked,")).collect();
    assert_eq!(returning.len(), 1, "only check_received returns one: {returning:?}");
    assert!(returning[0].contains("pub fn check_received("), "{returning:?}");
}

// verifies: LLR-2r2fha
#[test]
fn an_update_for_a_held_organisation_from_an_unlisted_sender_is_refused_in_the_check() {
    let (s, sender, body) = update_body("rp-update-unlisted");
    let reads = s.svc_b.view.reads();
    let refused = s.svc_b.check_received(rogue(), &body).map(|_| ());
    assert_eq!(refused, Err(OrgNodeError::SenderNotListed { org_id: s.org_id }));
    assert_eq!(s.svc_b.view.reads(), reads, "no Checked, so no read");
    assert!(matches!(s.svc_b.check_received(sender, &body), Ok(Checked::ReadChain(_))), "from A's Device it passes");
}

// verifies: LLR-2r2fha
#[test]
fn a_first_admission_is_accepted_from_any_sender() {
    let mut s = setup("rp-first-any");
    let (_, body) = admission_body(&mut s);
    let Ok(Checked::ReadChain(checked)) = s.svc_b.check_received(rogue(), &body) else { panic!("a message to read") };
    assert_eq!(checked.org_id(), s.org_id);
}

// verifies: LLR-kzgjz8, LLR-r7zm39
#[test]
fn a_revocation_from_an_unlisted_sender_is_refused_in_the_check() {
    let (s, _, sender, body) = notice_body("rp-notice-unlisted");
    let refused = s.svc_b.check_received(rogue(), &body).map(|_| ());
    assert_eq!(refused, Err(OrgNodeError::SenderNotListed { org_id: s.org_id }));
    assert!(matches!(s.svc_b.check_received(sender, &body), Ok(Checked::ReadChain(_))), "from A's Device it passes");
}

// B's acknowledgement of its removal: decided in the check, from the Device
// it names only, and only when the record no longer lists that Device.
// verifies: LLR-3aysup, LLR-5azhry
#[test]
fn an_acknowledgement_is_decided_in_the_check_from_its_named_device_only() {
    let tag = "rp-ack";
    let (mut s, _, sender, body) = notice_body(tag);
    let members_listing_b = rec_of(&s.svc_b, s.org_id).trie_members;
    let (checked, state) = read_chain(&s.svc_b, sender, &body);
    let (outcome, _) = commit(&mut s.svc_b, checked, state).0.unwrap();
    let acknowledgement = outcome.acknowledgements[0].clone();
    let ack_body = encode_message(&WireMessage::Acknowledgement(acknowledgement.clone())).unwrap();
    let b_device = s.joiner_b.device_key;

    let before = plaintext(&s.svc_a);
    match s.svc_a.check_received(b_device, &ack_body) {
        Ok(Checked::Acknowledged(verified)) => assert_eq!(verified.into_inner(), acknowledgement),
        other => panic!("not decided as an acknowledgement: {other:?}"),
    }
    assert_eq!(plaintext(&s.svc_a), before, "no store change");

    let relayed = s.svc_a.check_received(rogue(), &ack_body).map(|_| ());
    assert_eq!(relayed, Err(OrgNodeError::AcknowledgementNotFromItsDevice { org_id: s.org_id }));

    // A record that still lists B's Device (a store this software never
    // writes, sealed back as org-io would hold it).
    tamper(tag, "a", &s.pid_a, |data| data.orgs[0].trie_members = members_listing_b);
    let listing = reopened_service(tag, "a");
    let listed = listing.check_received(b_device, &ack_body).map(|_| ());
    assert_eq!(listed, Err(OrgNodeError::AcknowledgementForListedDevice { org_id: s.org_id }));
}

// ---- commit_checked: one test per arm -------------------------------------------

// verifies: LLR-6qjmhf
#[test]
fn a_first_admission_commits_as_admitted() {
    let mut s = setup("rp-admitted");
    let (sender, body) = admission_body(&mut s);
    let (checked, state) = read_chain(&s.svc_b, sender, &body);
    let (result, asked) = commit(&mut s.svc_b, checked, state);
    let (outcome, change) = result.unwrap();
    assert_eq!((outcome.org_id, outcome.epoch, outcome.kind), (s.org_id, Epoch::new(2), ReceiveKind::Admitted));
    assert!(outcome.acknowledgements.is_empty());
    assert!(!asked, "a commit that keeps this node asks for no secret");
    assert!(matches!(change, StoreChange::Sealed(_)));
    assert_eq!(s.svc_b.list_orgs().len(), 1);
}

// verifies: LLR-6qjmhf
#[test]
fn an_update_commits_as_updated() {
    let (mut s, sender, body) = update_body("rp-updated");
    let (checked, state) = read_chain(&s.svc_b, sender, &body);
    let (result, asked) = commit(&mut s.svc_b, checked, state);
    let (outcome, change) = result.unwrap();
    assert_eq!((outcome.epoch, outcome.kind), (Epoch::new(3), ReceiveKind::Updated));
    assert_eq!(outcome.root, s.chain.get(&s.org_id).unwrap().root_hash);
    assert!(!asked);
    assert!(matches!(change, StoreChange::Sealed(_)));
    assert_eq!(s.svc_b.list_orgs()[0].trie_members.len(), 3, "A + B + C");
}

// Organisation information whose committed record no longer lists B: the
// one removal step signs B's acknowledgement and forgets the Organisation,
// and B's store with it.
// verifies: LLR-6qjmhf, LLR-b27jr6
#[test]
fn the_nodes_own_removal_forgets_the_organisation() {
    let mut s = admit_b_directly(setup("rp-removed"));
    let before = rec_of(&s.svc_a, s.org_id);
    let b_id = id_by_handle(&before, "bob");
    // *Amended 2026-10-13 (S4, task T5):* A's commit sends B its notice, not
    // Organisation information, so the information the removal's Envelope
    // travels in is built here from the update A committed.
    let update = s.svc_a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, before.epoch).unwrap();
    s.svc_a.commit_update(&mut OsRng, s.org_id).unwrap();
    let org_node::store::ProvisionalChange::ChangeSet { change_set, org_private_key } = update.change_for_test().clone()
    else {
        panic!("a Change set")
    };
    let information = WireMessage::OrgInformation {
        envelope: org_node::Envelope { org_id: s.org_id, parent_seq: update.seq, delta_bytes: change_set },
        record_snapshot: postcard::to_allocvec(&before.trie_members).unwrap(),
        org_private_key,
    };
    let body = encode_message(&information).unwrap();
    let (checked, state) = read_chain(&s.svc_b, device_of(&s.svc_a, &s.pid_a), &body);
    let (result, asked) = commit(&mut s.svc_b, checked, state);
    let (outcome, change) = result.unwrap();
    assert_eq!((outcome.epoch, outcome.kind), (Epoch::new(3), ReceiveKind::Removed));
    assert!(asked, "the removal signs");
    assert_eq!(outcome.acknowledgements.len(), 1);
    assert_eq!(outcome.acknowledgements[0].device, s.joiner_b.device_key);
    assert!(matches!(change, StoreChange::Forgotten { device_key } if device_key == s.joiner_b.device_key));
    assert!(s.svc_b.list_orgs().is_empty() && s.svc_b.list_personas().is_empty());
}

// verifies: LLR-6qjmhf, LLR-tx8ruv
#[test]
fn an_accepted_revocation_commits_as_revoked() {
    let (mut s, _, sender, body) = notice_body("rp-revoked");
    let (checked, state) = read_chain(&s.svc_b, sender, &body);
    let (result, asked) = commit(&mut s.svc_b, checked, state);
    let (outcome, change) = result.unwrap();
    assert_eq!((outcome.org_id, outcome.epoch, outcome.kind), (s.org_id, Epoch::new(3), ReceiveKind::Revoked));
    assert!(asked, "the acknowledgement is signed with the supplied secret");
    assert!(matches!(change, StoreChange::Forgotten { device_key } if device_key == s.joiner_b.device_key));
    // Signed with B's secret: A's check verifies the signature under B's Device.
    let ack_body = encode_message(&WireMessage::Acknowledgement(outcome.acknowledgements[0].clone())).unwrap();
    assert!(matches!(s.svc_a.check_received(s.joiner_b.device_key, &ack_body), Ok(Checked::Acknowledged(_))));
}

// The notice decided against the state before the removal, under which B's
// Device is still listed: refused, nothing changed.
// verifies: LLR-6qjmhf, LLR-pt32fx
#[test]
fn a_revocation_that_leaves_this_device_listed_is_refused() {
    let (mut s, before_removal, sender, body) = notice_body("rp-still-listed");
    let (checked, _) = read_chain(&s.svc_b, sender, &body);
    let before = plaintext(&s.svc_b);
    let result = commit(&mut s.svc_b, checked, Some(before_removal));
    let error = refused_with_nothing_changed(result, &before, &s.svc_b);
    assert!(matches!(error, OrgNodeError::RevocationProofRefused { org_id, .. } if org_id == s.org_id), "{error:?}");
}

// verifies: LLR-6qjmhf, LLR-ba2ejp
#[test]
fn a_carried_key_that_does_not_match_the_chain_is_refused() {
    let (mut s, sender, body) = update_body("rp-carried-key");
    let tampered = encode_message(&with_key(&decode_message(&body).unwrap(), OrgPrivateKey::from([0x42; 32]))).unwrap();
    let (checked, state) = read_chain(&s.svc_b, sender, &tampered);
    let before = plaintext(&s.svc_b);
    let error = refused_with_nothing_changed(commit(&mut s.svc_b, checked, state), &before, &s.svc_b);
    assert_eq!(error, OrgNodeError::OrgKeyMismatch { org_id: s.org_id });
}

// verifies: LLR-6qjmhf
#[test]
fn no_chain_state_is_org_not_on_chain() {
    let (mut s, sender, body) = update_body("rp-no-state");
    let (checked, _) = read_chain(&s.svc_b, sender, &body);
    let before = plaintext(&s.svc_b);
    let error = refused_with_nothing_changed(commit(&mut s.svc_b, checked, None), &before, &s.svc_b);
    assert_eq!(error, OrgNodeError::OrgNotOnChain);

    let (mut s, _, sender, body) = notice_body("rp-no-state-notice");
    let (checked, _) = read_chain(&s.svc_b, sender, &body);
    let before = plaintext(&s.svc_b);
    let error = refused_with_nothing_changed(commit(&mut s.svc_b, checked, None), &before, &s.svc_b);
    assert_eq!(error, OrgNodeError::OrgNotOnChain);
}

// ---- source scans -----------------------------------------------------------------

/// The parameter list of `pub fn <name>` in `source`, generics included.
fn parameters_of(source: &str, name: &str) -> String {
    let start = source.find(&format!("pub fn {name}")).unwrap_or_else(|| panic!("no `pub fn {name}`"));
    let rest = &source[start..];
    rest[..rest.find("->").unwrap()].to_string()
}

// The sender is a value only `check_received` takes; `commit_checked` names a
// Device key only in its secret source; neither names a transport identity.
// verifies: LLR-6qjmhf
#[test]
fn only_the_check_takes_the_sender() {
    let source = service_source();
    let check = parameters_of(&source, "check_received");
    assert_eq!(check.matches("DevicePublicKey").count(), 1, "{check}");
    assert!(check.contains("sender: DevicePublicKey"), "{check}");
    let start = source.find("pub fn commit_checked").unwrap();
    let rest = &source[start..];
    let commit = &rest[..rest.find('{').unwrap()];
    let closure = commit.find("device_secrets: impl FnOnce(&[DevicePublicKey])").expect("the secret source");
    assert_eq!(commit.matches("DevicePublicKey").count(), 1, "named outside the closure: {commit}");
    assert!(commit[closure..].contains("DevicePublicKey"));
    for signature in [check.as_str(), commit] {
        assert!(!signature.contains("EndpointId") && !signature.contains("iroh"), "{signature}");
    }
}

// PR-q8r32t stays resolved: the `RevocationNotHeld` refusal is reached
// through `check_received`, so it is no unreachable branch.
// *Changed 2026-10-15 (S4 TEST review round 1, finding-10; Seams 1):* the
// second half asserted that revocation.rs constructs the error, which pinned
// the file it is built in; it now drives `check_received` with a notice
// about an Organisation no store holds and observes the refusal.
// *Changed 2026-10-15 (S4 TEST review round 2, finding-5; Seams 1):* the
// first half, a scan of service.rs's source for `RevocationNotHeld {`, is
// dropped; the behaviour is what is asserted. So an unreachable `if false`
// branch returning the error inside `check_received` no longer fails this
// test (it changes no behaviour; that mutation failed only the scan, now
// gone). Mutation check, that day (removed): `notice_store` starting from
// `RevocationNotForThisDevice` in place of `RevocationNotHeld`, so the
// unheld refusal is never reached, failed the remaining assertion.
// *Renamed 2026-10-15 (S4 TEST review round 3, notes):* was
// `no_receive_branch_is_unreachable`, a name that claimed more than the
// test checks; it now says what it checks.
// verifies: LLR-zetn7a
#[test]
fn an_unheld_organisations_notice_is_refused_as_not_held() {
    let s = setup("rp-no-unreachable");
    let body = encode_message(&WireMessage::Revocation(bob_absence_notice(s.org_id))).unwrap();
    let refused = s.svc_b.check_received(device_of(&s.svc_a, &s.pid_a), &body).map(|_| ());
    assert_eq!(refused, Err(OrgNodeError::RevocationNotHeld { org_id: s.org_id }), "check_received reaches the refusal");
}

#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The node's own provisional updates and their commit (REQ-xs4ab8,
//! REQ-tqap3r, REQ-uv3v5w): built and kept without touching the chain or the
//! record, committed only once they verify against the chain.

mod support;

use org_node::chain::OrgState;
use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::service::ProvisionalTarget;
use org_node::store::{MemberSnapshot, PersonaStatus, ProvisionalChange, ProvisionalUpdate, ProvisionalUpdateParts};
use org_node::test_fixtures::{device_key, member_key, org_public_key, ChainSlots};
use org_node::wire::WireMessage;
use org_node::{Envelope, Epoch, Joiner, PersonaId, RootHash, SequenceNumber};
use rand::rngs::OsRng;
use support::*;

/// A Persona on a fresh service over `chain`, and its genesis update.
fn genesis_built(tag: &str, chain: &ChainSlots) -> (Node, PersonaId, ProvisionalUpdate) {
    let mut svc = Node::new(open_store(tag, "a"), chain.clone());
    let pid = svc.create_persona(&mut OsRng, h("alice"), nm("Alice"), sn("Smith")).unwrap();
    let update = svc.create_organisation(&mut OsRng, &pid).unwrap();
    (svc, pid, update)
}

// Normal: creation keeps a genesis update — no record, no binding, no chain
// call, no endpoint — and the update states what the app will publish: the
// public key of the private key it holds, at epoch 1.
// It is also LLR-6z5xya's normal case: an unbound Persona is not refused.
// verifies: REQ-xs4ab8, REQ-ech45n, LLR-s6qnht, LLR-qjz3q4, LLR-68yd3j, LLR-nvn3wk, LLR-6z5xya
#[test]
fn create_organisation_keeps_a_genesis_update_and_nothing_else() {
    let chain = ChainSlots::new();
    let (svc, pid, update) = genesis_built("create", &chain);
    let persona = persona_of(&svc, &pid);
    let (member_key, device_key) = svc.persona_public_keys(&pid).unwrap();
    assert_eq!((update.org_id, update.base_root, update.seq), (None, None, SequenceNumber::new(1)));
    assert_eq!(update.persona_id, pid);
    assert_eq!(private_key_of(&update).x25519_keypair().org_public_key().unwrap(), update.org_pub_key);
    assert_ne!(update.org_pub_key.as_bytes(), member_key.as_bytes(), "a fresh key, not the founder's (REQ-ech45n)");
    let ProvisionalChange::Genesis { members, .. } = update.change_for_test() else { panic!("a genesis change") };
    assert_eq!(members.len(), 1);
    assert_eq!((members[0].member_key, members[0].device_keys.clone()), (member_key, vec![device_key]));
    assert!(svc.list_orgs().is_empty(), "no record");
    assert_eq!((persona.status, persona.org_id), (PersonaStatus::Proposed, None), "no binding");
    // *2026-10-14 (S4, task T14):* no endpoint to look at: org-node has none
    // (`absences::org_node_is_sans_io`).
    assert_eq!(svc.genesis_provisional_updates(&pid), vec![update.clone()]);
    let disk = reopen_store("create", "a");
    assert_eq!(disk.data().provisional_updates, vec![update]);
    assert!(disk.data().orgs.is_empty());
}

// Abnormal (LLR-nvn3wk): reading provisional updates changes nothing, and an
// unknown Persona or Organisation has none.
// verifies: LLR-nvn3wk
#[test]
fn reading_provisional_updates_changes_nothing() {
    let chain = ChainSlots::new();
    let (svc, _pid, _update) = genesis_built("read-only", &chain);
    let before = store_bytes("read-only", "a");
    assert!(svc.genesis_provisional_updates(&PersonaId::new("nobody".into())).is_empty());
    assert!(svc.provisional_updates(OrgId::new([9; 20])).is_empty());
    assert_eq!(store_bytes("read-only", "a"), before);
}

// Normal and abnormal (REQ-d9g6nt): the founding Member's id is drawn, not
// derived — the same Persona founding twice gets two ids, neither a key —
// and each genesis has its own Organisation key pair.
// verifies: REQ-d9g6nt, REQ-ech45n, LLR-rjg3m2
#[test]
fn the_founding_member_id_and_the_organisation_key_are_drawn_not_derived() {
    let chain = ChainSlots::new();
    let (mut svc, pid, first) = genesis_built("ids", &chain);
    let second = svc.create_organisation(&mut OsRng, &pid).unwrap();
    let id_of = |u: &ProvisionalUpdate| match u.change_for_test() {
        ProvisionalChange::Genesis { members, .. } => (members[0].id, members[0].member_key),
        ProvisionalChange::ChangeSet { .. } => panic!("genesis"),
    };
    let ((a, key), (b, _)) = (id_of(&first), id_of(&second));
    assert_ne!(a, b);
    assert_ne!(a.as_bytes(), key.as_bytes());
    assert_ne!(first.org_pub_key, second.org_pub_key);
    assert_eq!(svc.genesis_provisional_updates(&pid).len(), 2, "both kept");
}

// Normal: once the chain carries the root and key at epoch 1, commit_genesis
// creates the record with the chain's values, mark 1, the private key and the
// proxy account, binds the Persona, consumes the update, writes the store,
// and binds no endpoint.
// verifies: REQ-tqap3r, REQ-uv3v5w, LLR-wzqqg9, LLR-qjz3q4, LLR-3fwykc, LLR-w3fhhg, LLR-q3aj8z, LLR-dzte8x, LLR-3v5nu9, LLR-4tcxsu, LLR-mkj4bz
#[test]
fn commit_genesis_creates_the_record_once_the_chain_carries_the_root() {
    let chain = ChainSlots::new();
    let (mut svc, pid, update) = genesis_built("commit-genesis", &chain);
    let org_id = chain.apply_genesis(update.resulting_root, update.org_pub_key);
    let outcome = svc.commit_genesis(&mut OsRng, &pid, org_id, test_proxy()).unwrap();
    assert_eq!((outcome.org_id, outcome.epoch, outcome.root), (org_id, Epoch::new(1), update.resulting_root));
    let rec = rec_of(&svc, org_id);
    assert_eq!(
        (rec.root_hash, rec.org_pub_key, rec.epoch, rec.last_seq),
        (update.resulting_root, update.org_pub_key, Epoch::new(1), SequenceNumber::new(1))
    );
    assert_eq!(rec.org_private_key_for_test(), &private_key_of(&update));
    assert_eq!(rec.proxy_account, Some(test_proxy()));
    assert_eq!(svc.proxy_account(org_id).unwrap(), Some(test_proxy()));
    let p = persona_of(&svc, &pid);
    // *Amended 2026-10-07 (PR-eqs4fs):* this pinned `member_id: None`, the
    // defect; the founder is bound with the genesis leaf's MemberId.
    assert_eq!((p.status, p.org_id, p.member_id), (PersonaStatus::Active, Some(org_id), Some(rec.trie_members[0].id)));
    assert!(svc.genesis_provisional_updates(&pid).is_empty(), "the update is consumed");
    // *2026-10-14 (S4, task T14):* "a commit binds no endpoint" is an
    // absence now: org-node has none (`absences::org_node_is_sans_io`).
    let disk = reopen_store("commit-genesis", "a");
    assert_eq!(disk.data().orgs[0].proxy_account, Some(test_proxy()));
    assert!(disk.data().provisional_updates.is_empty());
}

// Abnormal: each refusal of commit_genesis returns its error and leaves the
// record list, the update and the Persona exactly as they were, unwritten.
// verifies: REQ-tqap3r, LLR-wzqqg9, LLR-ewkg85, LLR-mxskg9
#[test]
fn commit_genesis_refusals_change_nothing_and_write_nothing() {
    let chain = ChainSlots::new();
    let (mut svc, pid, update) = genesis_built("genesis-refused", &chain);
    let state = |root, key, epoch| OrgState { root_hash: root, org_pub_key: key, epoch: Epoch::new(epoch) };
    let unknown = OrgId::new([0x99; 20]);
    let (other_root, other_key, at_zero, at_two) =
        (OrgId::new([0x98; 20]), OrgId::new([0x97; 20]), OrgId::new([0x96; 20]), OrgId::new([0x95; 20]));
    chain.set(other_root, state(RootHash::new([0x01; 32]), update.org_pub_key, 1));
    chain.set(other_key, state(update.resulting_root, org_public_key(), 1));
    chain.set(at_zero, state(update.resulting_root, update.org_pub_key, 0));
    chain.set(at_two, state(update.resulting_root, update.org_pub_key, 2));
    let before = store_bytes("genesis-refused", "a");
    for (org, expected) in [
        (unknown, OrgNodeError::OrgNotOnChain),
        (other_root, OrgNodeError::NoProvisionalUpdate),
        (other_key, OrgNodeError::NoProvisionalUpdate),
        (at_zero, OrgNodeError::SeqNotEpoch { seq: 1, epoch: 0 }),
        (at_two, OrgNodeError::SeqNotEpoch { seq: 1, epoch: 2 }),
    ] {
        assert_eq!(svc.commit_genesis(&mut OsRng, &pid, org, test_proxy()).unwrap_err(), expected);
        assert!(svc.list_orgs().is_empty());
        assert_eq!(svc.genesis_provisional_updates(&pid), vec![update.clone()]);
        assert_eq!(persona_of(&svc, &pid).status, PersonaStatus::Proposed);
        assert_eq!(store_bytes("genesis-refused", "a"), before, "nothing written");
    }
}

// Abnormal (owner ruling 2026-10-06 on LLR-wzqqg9): another signatory of a
// 1-of-N multisig updated the Organisation before the founding node committed
// its genesis. Admitting and then revoking a Member brings the chain back to
// the genesis root and key at epoch 3: commit_genesis refuses with
// SeqNotEpoch, and the founding node keeps its genesis update, holds no
// record and stays unbound, the store unwritten — it obtains the record from
// an update another Member sends. Where the other signatory's update left
// another root, no genesis update produces the state, and the refusal is
// NoProvisionalUpdate, with the same nothing-changed guarantee.
// verifies: LLR-wzqqg9, LLR-ewkg85
#[test]
fn a_chain_past_epoch_one_refuses_commit_genesis_and_keeps_the_update() {
    let chain = ChainSlots::new();
    let (mut svc, pid, update) = genesis_built("genesis-past-one", &chain);
    let org_id = chain.apply_genesis(update.resulting_root, update.org_pub_key);
    let before = store_bytes("genesis-past-one", "a");
    let moved_on = |root, epoch| OrgState { root_hash: root, org_pub_key: update.org_pub_key, epoch: Epoch::new(epoch) };
    for (state, expected) in [
        (moved_on(update.resulting_root, 3), OrgNodeError::SeqNotEpoch { seq: 1, epoch: 3 }),
        (moved_on(RootHash::new([0x0b; 32]), 2), OrgNodeError::NoProvisionalUpdate),
    ] {
        chain.set(org_id, state);
        assert_eq!(svc.commit_genesis(&mut OsRng, &pid, org_id, test_proxy()).unwrap_err(), expected);
        assert!(svc.list_orgs().is_empty(), "no record");
        assert_eq!(svc.genesis_provisional_updates(&pid), vec![update.clone()], "the genesis update stays");
        let p = persona_of(&svc, &pid);
        assert_eq!((p.status, p.org_id), (PersonaStatus::Proposed, None), "no binding");
        assert_eq!(store_bytes("genesis-past-one", "a"), before, "nothing written");
    }
}

// Abnormal: a stored genesis update whose members do not rebuild to the root
// the chain carries is refused with RootMismatch.
// verifies: LLR-wzqqg9, LLR-ewkg85
#[test]
fn a_genesis_whose_members_do_not_rebuild_the_root_is_refused() {
    let chain = ChainSlots::new();
    let (svc, pid, update) = genesis_built("genesis-mismatch", &chain);
    drop(svc);
    let forged = ProvisionalUpdate::from(ProvisionalUpdateParts { resulting_root: RootHash::new([0x42; 32]), ..update.into() });
    tamper("genesis-mismatch", "a", &pid, |data| data.insert_provisional(forged.clone()).unwrap());
    let mut svc = Node::reopened(party("genesis-mismatch", "a"), chain.clone());
    let org = chain.apply_genesis(forged.resulting_root, forged.org_pub_key);
    let before = store_bytes("genesis-mismatch", "a");
    assert_eq!(svc.commit_genesis(&mut OsRng, &pid, org, test_proxy()).unwrap_err(), OrgNodeError::RootMismatch);
    assert!(svc.list_orgs().is_empty());
    assert_eq!(store_bytes("genesis-mismatch", "a"), before);
}

// Abnormal (review finding-2, 2026-10-07): a stored genesis update of the
// named Persona whose members rebuild the chain's root but list no leaf with
// that Persona's Device — here another Persona's genesis update, re-addressed
// to Alice — is refused with NoProvisionalUpdate: no record, no binding, the
// store unwritten.
// verifies: LLR-wzqqg9
#[test]
fn a_genesis_listing_no_leaf_with_the_personas_device_is_refused() {
    let chain = ChainSlots::new();
    let (mut svc, pid, own) = genesis_built("genesis-not-ours", &chain);
    let pid_bob = svc.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("Jones")).unwrap();
    let bobs = svc.create_organisation(&mut OsRng, &pid_bob).unwrap();
    drop(svc);
    let forged = ProvisionalUpdate::from(ProvisionalUpdateParts { persona_id: pid.clone(), ..bobs.into() });
    tamper("genesis-not-ours", "a", &pid, |data| data.insert_provisional(forged.clone()).unwrap());
    let mut svc = Node::reopened(party("genesis-not-ours", "a"), chain.clone());
    let org = chain.apply_genesis(forged.resulting_root, forged.org_pub_key);
    let before = store_bytes("genesis-not-ours", "a");
    assert_eq!(
        svc.commit_genesis(&mut OsRng, &pid, org, test_proxy()).unwrap_err(),
        OrgNodeError::NoProvisionalUpdate
    );
    assert!(svc.list_orgs().is_empty(), "no record");
    let p = persona_of(&svc, &pid);
    assert_eq!((p.status, p.org_id, p.member_id), (PersonaStatus::Proposed, None, None), "no binding");
    assert_eq!(svc.genesis_provisional_updates(&pid), vec![own, forged], "both updates stay");
    assert_eq!(store_bytes("genesis-not-ours", "a"), before, "nothing written");
}

// (2026-10-08, change worktree-org-io-create, ruling B:
// `a_failed_chain_read_refuses_commit_genesis` left with the read. A failed
// read never reaches org-node; org-io's read-back test after a write takes
// its place.)

/// Another joiner, from fixed seeds.
fn joiner_from(seed: u8, handle: &str) -> Joiner {
    Joiner { handle: h(handle), name: nm("Test"), surname: sn("Joiner"), member_key: member_key(seed), device_key: device_key(seed + 1) }
}

/// A founded Organisation (A) and a joiner (B's Persona) not yet admitted.
fn founded(tag: &str) -> (ChainSlots, Node, OrgId, Joiner, Node) {
    let chain = ChainSlots::new();
    let mut a = Node::new(open_store(tag, "a"), chain.clone());
    let pid_a = a.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let org = found(&mut a, &chain, &pid_a);
    let mut b = Node::new(open_store(tag, "b"), chain.clone());
    let pid_b = b.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("Builder")).unwrap();
    let joiner = joiner_of(&b, &pid_b);
    (chain, a, org, joiner, b)
}

// Normal: admission keeps a provisional update — base the record's root,
// Sequence number the epoch it produces, a fresh key pair — and changes
// nothing else; a second admission before the first commits is built on the
// same record.
// verifies: REQ-xs4ab8, REQ-txvtm9, REQ-stx9v3, LLR-rb8r65, LLR-ghja3x, LLR-e2b7gv, LLR-qjz3q4, LLR-nvn3wk
#[test]
fn admit_member_keeps_a_provisional_update_and_changes_nothing_else() {
    let (chain, mut a, org, joiner, _b) = founded("admit-provisional");
    let rec = rec_of(&a, org);
    let update = a.admit_member(&mut OsRng, org, &joiner).unwrap();
    assert_eq!(update.org_id, Some(org));
    assert_eq!(update.base_root, Some(rec.root_hash));
    assert_eq!(update.seq, SequenceNumber::new(rec.epoch.get() + 1), "the epoch it produces");
    assert_eq!(update.seq, SequenceNumber::new(rec.last_seq.get() + 1), "the mark equals the epoch");
    assert_ne!(update.org_pub_key, rec.org_pub_key, "a fresh key pair, not the record's (REQ-stx9v3)");
    assert_eq!(
        private_key_of(&update).x25519_keypair().org_public_key().unwrap(),
        update.org_pub_key,
        "the update holds its private half"
    );
    assert_ne!(update.resulting_root, rec.root_hash);
    assert!(matches!(update.change_for_test(), ProvisionalChange::ChangeSet { .. }));
    assert_eq!(chain.get(&org).unwrap().epoch, Epoch::new(1), "no chain write");
    assert_eq!(rec_of(&a, org).trie_members.len(), rec.trie_members.len(), "record unchanged");
    let now = rec_of(&a, org);
    assert_eq!(
        (now.org_pub_key, now.org_private_key_for_test()),
        (rec.org_pub_key, rec.org_private_key_for_test()),
        "building changes neither of the record's keys"
    );
    // *2026-10-14 (S4, task T14):* no endpoint to look at: org-node has none
    // (`absences::org_node_is_sans_io`).
    let second = a.admit_member(&mut OsRng, org, &joiner_from(0x61, "carol")).unwrap();
    assert_eq!((second.base_root, second.seq), (update.base_root, update.seq), "built on the same record");
    assert_ne!(second.org_pub_key, update.org_pub_key, "each update draws its own pair");
    assert_eq!(a.provisional_updates(org).len(), 2);
}

// Normal and abnormal (LLR-gwk4nk): the by-reference query answers true for
// the held update's four public fields, and false when any one of them is
// another's — the Organisation, the base root, the resulting root, the
// public key; it writes nothing.
// verifies: LLR-gwk4nk
#[test]
fn holds_provisional_matches_all_four_public_fields_and_writes_nothing() {
    let (_chain, mut a, org, joiner, _b) = founded("holds-provisional");
    let update = a.admit_member(&mut OsRng, org, &joiner).unwrap();
    let base = update.base_root.unwrap();
    let (resulting, key) = (update.resulting_root, update.org_pub_key);
    let before = store_bytes("holds-provisional", "a");
    assert!(a.holds_provisional(org, base, resulting, key), "the held update");
    assert!(!a.holds_provisional(OrgId::new([9; 20]), base, resulting, key), "another Organisation");
    assert!(!a.holds_provisional(org, resulting, resulting, key), "another base root");
    assert!(!a.holds_provisional(org, base, base, key), "another resulting root");
    let record_key = rec_of(&a, org).org_pub_key;
    assert!(!a.holds_provisional(org, base, resulting, record_key), "another public key: the record's");
    assert_eq!(store_bytes("holds-provisional", "a"), before, "the query writes nothing");
}

// Abnormal (LLR-vdyu65): an admission into an Organisation this node holds no
// record of is refused with OrgNotOnChain — it is not built on the record the
// node does hold — and keeps nothing under either Organisation, the held
// record unchanged and nothing written.
// verifies: LLR-vdyu65
#[test]
fn admitting_into_an_organisation_not_held_touches_no_other() {
    let (_chain, mut a, org, joiner, _b) = founded("admit-unheld");
    let rec = rec_of(&a, org);
    let before = store_bytes("admit-unheld", "a");
    let unheld = OrgId::new([0x99; 20]);
    assert_eq!(a.admit_member(&mut OsRng, unheld, &joiner).unwrap_err(), OrgNodeError::OrgNotOnChain);
    assert!(a.provisional_updates(unheld).is_empty());
    assert!(a.provisional_updates(org).is_empty(), "nothing kept under the Organisation held");
    let after = rec_of(&a, org);
    assert_eq!((after.root_hash, after.epoch, after.last_seq), (rec.root_hash, rec.epoch, rec.last_seq));
    assert_eq!(after.trie_members, rec.trie_members);
    assert_eq!(store_bytes("admit-unheld", "a"), before, "nothing written");
}

// Normal: a provisional update the chain carries is committed as a received
// update would be, and the outgoing update is the Envelope and the record as
// it stood before. No endpoint is bound.
// verifies: REQ-tqap3r, REQ-jy6ybw, LLR-cmdrp9, LLR-6s785x, LLR-bg3vsw, LLR-4tcxsu, LLR-cja9zv
#[test]
fn commit_update_commits_a_provisional_update_the_chain_carries() {
    let (chain, mut a, org, joiner, _b) = founded("commit-update");
    let before = rec_of(&a, org);
    let update = a.admit_member(&mut OsRng, org, &joiner).unwrap();
    chain.apply_update(org, update.resulting_root, update.org_pub_key, before.epoch).unwrap();
    let out = a.commit_update(&mut OsRng, org).unwrap();
    assert_eq!((out.org_id, out.epoch, out.root), (org, Epoch::new(2), update.resulting_root));
    let ProvisionalChange::ChangeSet { change_set, org_private_key } = update.change_for_test().clone() else { panic!() };
    // *Amended 2026-10-13 (S4, task T5):* the outgoing update is the body the
    // commit built for the joiner.
    let WireMessage::OrgInformation { envelope, record_snapshot, .. } = message_for(&out, joiner.device_key) else {
        panic!("Organisation information")
    };
    assert_eq!(envelope, Envelope { org_id: org, parent_seq: update.seq, delta_bytes: change_set });
    let sent: Vec<MemberSnapshot> = postcard::from_bytes(&record_snapshot).unwrap();
    assert_eq!(sent, before.trie_members, "the record as it stood before the commit");
    let after = rec_of(&a, org);
    assert_eq!((after.root_hash, after.epoch, after.last_seq), (update.resulting_root, Epoch::new(2), update.seq));
    assert_eq!(after.trie_members.len(), 2);
    assert_eq!(
        (after.org_pub_key, after.org_private_key_for_test()),
        (update.org_pub_key, &org_private_key),
        "the record takes the update's key pair and keeps no earlier one"
    );
    let disk = reopen_store("commit-update", "a").data().orgs[0].clone();
    assert_eq!(disk.epoch, Epoch::new(2));
    assert_eq!((disk.org_pub_key, disk.org_private_key_for_test()), (update.org_pub_key, &org_private_key));
    // *2026-10-14 (S4, task T14):* "a commit binds no endpoint and sends
    // nothing" is an absence now: org-node has no endpoint
    // (`absences::org_node_is_sans_io`).
}

// Abnormal: every refusal of commit_update returns its error and leaves the
// record and the provisional updates as they were, nothing written.
// verifies: REQ-tqap3r, LLR-cmdrp9, LLR-6s785x, LLR-ewkg85
#[test]
fn commit_update_refusals_change_nothing_and_write_nothing() {
    let (chain, mut a, org, joiner, _b) = founded("update-refused");
    let update = a.admit_member(&mut OsRng, org, &joiner).unwrap();
    let rec = rec_of(&a, org);
    let before = store_bytes("update-refused", "a");
    let check = |a: &Node| {
        let now = rec_of(a, org);
        assert_eq!(now.root_hash, rec.root_hash);
        assert_eq!(
            (now.org_pub_key, now.org_private_key_for_test()),
            (rec.org_pub_key, rec.org_private_key_for_test()),
            "the record's keys are unchanged"
        );
        assert_eq!(a.provisional_updates(org), vec![update.clone()]);
        assert_eq!(store_bytes("update-refused", "a"), before, "nothing written");
    };
    // no record
    assert_eq!(a.commit_update(&mut OsRng, OrgId::new([0x99; 20])).unwrap_err(), OrgNodeError::OrgNotOnChain);
    check(&a);
    // the chain carries a root no provisional update produces (still genesis)
    assert_eq!(a.commit_update(&mut OsRng, org).unwrap_err(), OrgNodeError::NoProvisionalUpdate);
    check(&a);
    // the chain carries the update's root under a key no provisional update holds
    chain.set(org, OrgState { root_hash: update.resulting_root, org_pub_key: org_public_key(), epoch: Epoch::new(rec.epoch.get() + 1) });
    assert_eq!(a.commit_update(&mut OsRng, org).unwrap_err(), OrgNodeError::NoProvisionalUpdate);
    check(&a);
    // the chain carries the root but at the epoch already committed
    chain.set(org, OrgState { root_hash: update.resulting_root, org_pub_key: update.org_pub_key, epoch: rec.epoch });
    assert_eq!(
        a.commit_update(&mut OsRng, org).unwrap_err(),
        OrgNodeError::StaleEpoch { got: rec.epoch.get(), last: rec.epoch.get() }
    );
    check(&a);
    // the chain carries the root two epochs on: not the update's Sequence number
    chain.set(org, OrgState { root_hash: update.resulting_root, org_pub_key: update.org_pub_key, epoch: Epoch::new(rec.epoch.get() + 2) });
    assert_eq!(
        a.commit_update(&mut OsRng, org).unwrap_err(),
        OrgNodeError::SeqNotEpoch { seq: update.seq.get(), epoch: rec.epoch.get() + 2 }
    );
    check(&a);
}

// Abnormal: given no chain state, the commit is refused with `OrgNotOnChain`
// and the update kept, nothing written. *Rewritten 2026-10-08 (ruling B,
// change worktree-org-io-create):* was
// `commit_update_refuses_when_the_chain_fails_or_is_silent`; its failing half
// left with the read (org-io's read-back test takes its place).
// verifies: LLR-ewkg85
#[test]
fn commit_update_refuses_when_the_chain_holds_no_state() {
    let s = admit_b_directly(setup("update-chain-fails"));
    let mut b = s.svc_b;
    b.admit_member(&mut OsRng, s.org_id, &joiner_from(0x61, "carol")).unwrap();
    let before = store_bytes("update-chain-fails", "b");
    assert_eq!(
        b.svc.commit_update(&mut OsRng, s.org_id, None, |_| Vec::new()).unwrap_err(),
        OrgNodeError::OrgNotOnChain
    );
    assert_eq!(b.provisional_updates(s.org_id).len(), 1);
    assert_eq!(store_bytes("update-chain-fails", "b"), before, "nothing written");
}

// Normal: a commit discards every provisional update for that Organisation
// built on the old base and keeps other Organisations' — on the node's own
// commit and on a received one.
// verifies: REQ-uv3v5w, LLR-mkj4bz, LLR-2r2fha
#[test]
fn a_commit_discards_the_provisional_updates_it_orphans() {
    let mut s = admit_b_directly(setup("orphans"));
    let joiner_c = joiner_for_c(&mut s.svc_a);
    // B builds an admission of its own (B is bound to the Organisation).
    s.svc_b.admit_member(&mut OsRng, s.org_id, &joiner_from(0x71, "dora")).unwrap();
    // A builds two on the same base, the chain takes the first.
    let first = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_c).unwrap();
    s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_from(0x73, "erin")).unwrap();
    // A second Organisation on A's node, with a pending admission of its own.
    let pid_a2 = s.svc_a.create_persona(&mut OsRng, h("admin2"), nm("Admin"), sn("Two")).unwrap();
    let org_2 = found(&mut s.svc_a, &s.chain, &pid_a2);
    s.svc_a.admit_member(&mut OsRng, org_2, &joiner_from(0x75, "fay")).unwrap();
    s.chain.apply_update(s.org_id, first.resulting_root, first.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    let out = s.svc_a.commit_update(&mut OsRng, s.org_id).unwrap();
    assert!(s.svc_a.provisional_updates(s.org_id).is_empty(), "both built on the old base are gone");
    assert_eq!(s.svc_a.provisional_updates(org_2).len(), 1, "another Organisation's are kept");
    // B receives A's commit: its own provisional update is orphaned too.
    let info = message_for(&out, s.joiner_b.device_key);
    // *Rewritten 2026-10-07 (S3 T12a):* from a relay B's record does not
    // list, the commit is refused (LLR-2r2fha) and B's update is kept; from
    // A's listed Device it is committed and B's update discarded.
    let (b_addr, b_task) = spawn_receive(s.svc_b);
    deliver(b_addr, &info);
    let (svc_b, result) = b_task.run();
    assert_eq!(result.map(|_| ()), Err(OrgNodeError::SenderNotListed { org_id: s.org_id }));
    assert_eq!(svc_b.provisional_updates(s.org_id).len(), 1, "a refused message discards nothing");
    let (b_addr, b_task) = spawn_receive(svc_b);
    deliver_from(device_seed_of(&s.svc_a, &s.pid_a), b_addr, &info);
    let (svc_b, result) = b_task.run();
    result.unwrap();
    assert!(svc_b.provisional_updates(s.org_id).is_empty());
}

// Normal: the kind follows the recipient, from the node's record alone. A
// Device the committed record lists receives Organisation information — the
// committed Envelope, the record as it stood before the commit, the key the
// record holds, none taken from the caller; a Device neither record lists
// has no body. Sent under the first bound Persona's device, to the full
// address in Loopback mode, writing nothing. After a removal the removed
// Device receives its notice and nothing else, and a Device still listed
// receives Organisation information.
// *Rewritten 2026-10-13 (S4, task T5):* over the bodies the commit returns;
// `send_update`'s refusal of an unlisted, un-noticed Device became "no body
// for it", and the dial half sent a body through the transitional
// `send_body` (until T14 deleted it; see below). The founder's own Device,
// the sender's, has no body; C, a further Persona on A's node, is the
// Device still listed after the removal.
// *2026-10-13 (S4, task T12):* the Loopback dial item this test also
// verified moved to org-io with the send (`org-io/tests/send_fanout.rs`
// verifies it there); its ID left this line, which keeps the others.
// verifies: LLR-6ymd6d, LLR-8hdu9x, LLR-bg3vsw, LLR-2xzys9, LLR-48jakr, LLR-t4znbk, REQ-szq3ud, REQ-3dsweu
#[test]
fn send_update_sends_organisation_information_to_a_listed_device_and_a_revocation_to_any_other() {
    let (chain, mut a, org, joiner, _b) = founded("send");
    let joiner_c = joiner_for_c(&mut a);
    let update = a.admit_member(&mut OsRng, org, &joiner).unwrap();
    chain.apply_update(org, update.resulting_root, update.org_pub_key, Epoch::new(1)).unwrap();
    let out = a.commit_update(&mut OsRng, org).unwrap();
    let update_c = a.admit_member(&mut OsRng, org, &joiner_c).unwrap();
    let before_c = rec_of(&a, org);
    chain.apply_update(org, update_c.resulting_root, update_c.org_pub_key, Epoch::new(2)).unwrap();
    let out_c = a.commit_update(&mut OsRng, org).unwrap();
    let on_disk = store_bytes("send", "a");
    let rec = rec_of(&a, org);
    let first_bound = a.list_personas().into_iter().find(|p| p.org_id == Some(org)).unwrap().clone();
    let founder_device = first_bound.device_key;
    let ProvisionalChange::ChangeSet { change_set, .. } = update_c.change_for_test().clone() else { panic!() };
    assert_eq!(
        message_for(&out_c, joiner.device_key),
        WireMessage::OrgInformation {
            envelope: Envelope { org_id: org, parent_seq: update_c.seq, delta_bytes: change_set },
            record_snapshot: postcard::to_allocvec(&before_c.trie_members).unwrap(),
            org_private_key: rec.org_private_key_for_test().clone(),
        },
        "the joiner's Device is listed"
    );
    assert_eq!(body_for(&out_c, device_key(0x7a)), None, "a Device neither record lists has no body");
    assert_eq!(body_for(&out_c, founder_device), None, "the sender's own Device has no body");

    // *2026-10-14 (S4, task T14):* the dial half (the transitional
    // `send_body` to a real endpoint) went with that send; org-io dials
    // (`org-io/tests/send_fanout.rs`). What stays is org-node's half: the
    // commit names the first bound Persona as the sender, and reading the
    // body writes nothing.
    assert_eq!(out.sender, first_bound.persona_id, "the commit names the first bound Persona");
    assert!(matches!(message_for(&out, joiner.device_key), WireMessage::OrgInformation { .. }));
    assert_eq!(store_bytes("send", "a"), on_disk, "reading the bodies writes nothing");

    let joiner_id = rec.trie_members.iter().find(|m| m.member_key == joiner.member_key).unwrap().id;
    let removal = a.revoke_member(&mut OsRng, org, joiner_id).unwrap();
    chain.apply_update(org, removal.resulting_root, removal.org_pub_key, Epoch::new(3)).unwrap();
    let out = a.commit_update(&mut OsRng, org).unwrap();
    assert_eq!(message_for(&out, joiner.device_key), WireMessage::Revocation(notices_in(&out)[0].clone()), "the removed Device");
    assert_eq!(notices_in(&out).len(), 1);
    assert!(matches!(message_for(&out, joiner_c.device_key), WireMessage::OrgInformation { .. }), "a Device still listed");
}

// *Deleted 2026-10-14 (S4, task T14):* the test of the transitional
// `send_body`'s refusals (no Persona bound, or Loopback with no address)
// went with that send. org-io's send refuses with its typed errors
// (`org-io/tests/send_fanout.rs`); LLR-2xzys9's remaining half, that the
// commit names the sending Persona, is asserted by
// `send_update_sends_organisation_information_to_a_listed_device_and_a_revocation_to_any_other`.

// Abnormal (LLR-6ymd6d): an outgoing update naming an Organisation the node
// holds no record of is refused with OrgNotOnChain — the kind is chosen from
// that Organisation's record alone, never from another record the node holds
// — before an endpoint is bound, so nothing is sent; nothing is written. The
// recipient is one the held record lists, so a kind chosen from that record
// would be Organisation information carrying its key.
// *Rewritten 2026-10-13 (S4, task T5):* there is no send-time choice left
// to point at another Organisation (`send_update` gone): the bodies are
// built inside the commit from the committed Organisation's record alone.
// With a second Organisation on the node, whose record lists another
// Device, a commit to the first builds bodies only for the first's Devices,
// each naming the first and carrying its key, never the second's; building
// them writes nothing beyond the commit and binds no endpoint.
// verifies: LLR-6ymd6d
#[test]
fn send_update_refuses_an_organisation_it_holds_no_record_of() {
    let (chain, mut a, org, joiner, _b) = founded("send-unheld");
    let pid_a2 = a.create_persona(&mut OsRng, h("admin2"), nm("Admin"), sn("Two")).unwrap();
    let org_2 = found(&mut a, &chain, &pid_a2);
    let fay = joiner_from(0x75, "fay");
    commit_admission(&mut a, &chain, org_2, &fay).unwrap();
    let update = a.admit_member(&mut OsRng, org, &joiner).unwrap();
    chain.apply_update(org, update.resulting_root, update.org_pub_key, Epoch::new(1)).unwrap();
    let out = a.commit_update(&mut OsRng, org).unwrap();
    assert_eq!(out.outgoing.iter().map(|message| message.recipient).collect::<Vec<_>>(), vec![joiner.device_key]);
    assert_eq!(body_for(&out, fay.device_key), None, "no body for the other Organisation's Device");
    let WireMessage::OrgInformation { envelope, org_private_key, .. } = message_for(&out, joiner.device_key) else {
        panic!("Organisation information")
    };
    assert_eq!(envelope.org_id, org);
    assert_eq!(&org_private_key, rec_of(&a, org).org_private_key_for_test());
    assert_ne!(&org_private_key, rec_of(&a, org_2).org_private_key_for_test(), "not the other record's key");
    // *2026-10-14 (S4, task T14):* "nothing bound: nothing sent" is an
    // absence now: org-node has no endpoint (`absences::org_node_is_sans_io`).
}

// Abnormal (LLR-bg3vsw): a joiner that missed its own admission message holds
// no record when the next update reaches it. The Organisation information it
// then receives carries the record as it stood before that update — which
// already lists the joiner — so it rebuilds the trie the update applies to and
// commits it as its first admission. The record as it stands after the update
// is not a base the update applies to and is refused, nothing written.
// verifies: LLR-bg3vsw
#[test]
fn a_joiner_that_missed_its_admission_rebuilds_from_the_next_updates_snapshot() {
    let mut s = setup("missed-admission");
    let joiner_b = s.joiner_b.clone();
    let _lost = captured_admission(&mut s, &joiner_b);
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let update = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_c).unwrap();
    let before = rec_of(&s.svc_a, s.org_id);
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, before.epoch).unwrap();
    let out = s.svc_a.commit_update(&mut OsRng, s.org_id).unwrap();
    let to_b = message_for(&out, joiner_b.device_key);
    assert!(matches!(to_b, WireMessage::OrgInformation { .. }), "B is listed");

    let after_snapshot = postcard::to_allocvec(&rec_of(&s.svc_a, s.org_id).trie_members).unwrap();
    let on_disk = store_bytes("missed-admission", "b");
    let (addr, task) = spawn_receive(s.svc_b);
    deliver(addr, &with_snapshot(&to_b, after_snapshot));
    let (svc_b, result) = task.run();
    assert!(result.is_err(), "the record after the update is not its base: {result:?}");
    assert!(svc_b.list_orgs().is_empty(), "no record");
    assert_eq!(store_bytes("missed-admission", "b"), on_disk, "nothing written");

    let (addr, task) = spawn_receive(svc_b);
    deliver(addr, &to_b);
    let (svc_b, result) = task.run();
    assert_eq!(result.expect("B rebuilds from the snapshot before the update").epoch, Epoch::new(3));
    let rec = rec_of(&svc_b, s.org_id);
    assert_eq!(rec.trie_members, rec_of(&s.svc_a, s.org_id).trie_members, "A + B + C");
    assert_eq!(persona_of(&svc_b, &s.pid_b).status, PersonaStatus::Active);
}

// Abnormal (REQ-fwfku9's "keep nothing"): an admission the bound refuses
// writes nothing to disk.
// verifies: REQ-fwfku9, LLR-jq7qh7
#[test]
fn an_admission_refused_by_the_bound_writes_nothing() {
    let (chain, a, org, joiner, _b) = founded("bound-refused");
    let founder = a.list_personas()[0].persona_id.clone();
    drop(a);
    tamper("bound-refused", "a", &founder, |data| {
        let big = ProvisionalUpdate::from(ProvisionalUpdateParts {
            org_id: Some(org),
            persona_id: data.persona.persona_id.clone(),
            base_root: Some(data.orgs[0].root_hash),
            resulting_root: RootHash::new([0x42; 32]),
            seq: SequenceNumber::new(2),
            org_pub_key: data.orgs[0].org_pub_key,
            change: ProvisionalChange::ChangeSet { change_set: vec![0; org_node::store::MAX_PROVISIONAL_BYTES - 200], org_private_key: org_node::OrgPrivateKey::from([0x5e; 32]) },
        });
        data.insert_provisional(big).unwrap();
    });
    let mut a = Node::reopened(party("bound-refused", "a"), chain.clone());
    let before = store_bytes("bound-refused", "a");
    assert!(matches!(a.admit_member(&mut OsRng, org, &joiner), Err(OrgNodeError::ProvisionalLimit { .. })));
    assert_eq!(store_bytes("bound-refused", "a"), before);
    assert_eq!(a.provisional_updates(org).len(), 1);
}

// Normal: discarding a genesis update removes it — and with it the
// Organisation private key it holds — and saves before returning: the store
// reopened from disk holds no provisional update.
// verifies: LLR-7cmp38, REQ-hhva9d
#[test]
fn discarding_a_genesis_update_removes_it_and_its_private_key_and_saves() {
    let chain = ChainSlots::new();
    let (mut svc, pid, update) = genesis_built("discard-genesis", &chain);
    svc.discard_provisional(&mut OsRng, ProvisionalTarget::Genesis(pid.clone()), update.resulting_root, update.org_pub_key).unwrap();
    assert!(svc.genesis_provisional_updates(&pid).is_empty());
    let disk = reopen_store("discard-genesis", "a");
    assert!(disk.data().provisional_updates.is_empty(), "the update and its private key are gone from disk");
    assert!(disk.data().orgs.is_empty(), "no record");
}

// Normal: discarding one of two admissions keeps the other and leaves the
// record as it was; the discard is saved.
// verifies: LLR-7cmp38, REQ-hhva9d
#[test]
fn discarding_one_of_two_updates_keeps_the_other_and_the_record() {
    let (_chain, mut a, org, joiner, _b) = founded("discard-one");
    let first = a.admit_member(&mut OsRng, org, &joiner).unwrap();
    let second = a.admit_member(&mut OsRng, org, &joiner_from(0x61, "carol")).unwrap();
    let before = rec_of(&a, org);
    a.discard_provisional(&mut OsRng, ProvisionalTarget::Org(org), first.resulting_root, first.org_pub_key).unwrap();
    assert_eq!(a.provisional_updates(org), vec![second.clone()]);
    let after = rec_of(&a, org);
    assert_eq!((after.root_hash, after.epoch, after.last_seq), (before.root_hash, before.epoch, before.last_seq));
    assert_eq!(after.trie_members, before.trie_members);
    assert_eq!(reopen_store("discard-one", "a").data().provisional_updates, vec![second]);
}

// Abnormal: a root and key no provisional update for the target holds — an
// unknown root, a known root under the wrong target or another update's key —
// is refused with NoProvisionalUpdate, nothing changed and nothing written.
// verifies: LLR-7cmp38, REQ-hhva9d
#[test]
fn discarding_an_unknown_root_is_refused_and_writes_nothing() {
    let (_chain, mut a, org, joiner, _b) = founded("discard-unknown");
    let first = a.admit_member(&mut OsRng, org, &joiner).unwrap();
    let second = a.admit_member(&mut OsRng, org, &joiner_from(0x61, "carol")).unwrap();
    let before = store_bytes("discard-unknown", "a");
    let pid = a.list_personas()[0].persona_id.clone();
    for (target, root, key) in [
        (ProvisionalTarget::Org(org), RootHash::new([0xAB; 32]), first.org_pub_key),
        (ProvisionalTarget::Org(OrgId::new([0x99; 20])), first.resulting_root, first.org_pub_key),
        (ProvisionalTarget::Genesis(pid), first.resulting_root, first.org_pub_key),
        (ProvisionalTarget::Org(org), first.resulting_root, second.org_pub_key),
    ] {
        assert_eq!(a.discard_provisional(&mut OsRng, target, root, key).unwrap_err(), OrgNodeError::NoProvisionalUpdate);
        assert_eq!(a.provisional_updates(org), vec![first.clone(), second.clone()]);
        assert_eq!(store_bytes("discard-unknown", "a"), before, "nothing written");
    }
}

// Abnormal: a genesis update is discarded only under the Persona that built
// it; another Persona's id, or an Organisation target, is refused and the
// update and its private key stay, nothing written.
// verifies: LLR-7cmp38, REQ-hhva9d
#[test]
fn a_genesis_update_is_discarded_only_under_its_own_persona() {
    let chain = ChainSlots::new();
    let (mut svc, pid, update) = genesis_built("discard-other-persona", &chain);
    let other = svc.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("Builder")).unwrap();
    let before = store_bytes("discard-other-persona", "a");
    for target in [ProvisionalTarget::Genesis(other), ProvisionalTarget::Org(OrgId::new([0x99; 20]))] {
        assert_eq!(
            svc.discard_provisional(&mut OsRng, target, update.resulting_root, update.org_pub_key).unwrap_err(),
            OrgNodeError::NoProvisionalUpdate
        );
        assert_eq!(svc.genesis_provisional_updates(&pid), vec![update.clone()]);
        assert_eq!(store_bytes("discard-other-persona", "a"), before, "nothing written");
    }
}

// Normal: a failed founding (its genesis update never reached the chain), then
// a successful one: the commit removes the Persona's other genesis update,
// with the Organisation private key it holds, in the same save — the store
// reopened from disk holds no genesis update of that Persona.
// verifies: LLR-pk6mhv
// *Reconciled with S3b-io part 1, 2026-10-14:* org-node runs no runtime since
// S4 task T14, so this is a plain `#[test]` and `commit_genesis` is not
// awaited; the merged view of the Persona stores stands in for the one store.
#[test]
fn a_founding_drops_the_personas_other_genesis_updates() {
    let chain = ChainSlots::new();
    let (mut svc, pid, failed) = genesis_built("founding-drops", &chain);
    let founded = svc.create_organisation(&mut OsRng, &pid).unwrap();
    let org_id = chain.apply_genesis(founded.resulting_root, founded.org_pub_key);
    svc.commit_genesis(&mut OsRng, &pid, org_id, test_proxy()).unwrap();
    assert!(svc.genesis_provisional_updates(&pid).is_empty(), "the failed founding's update is gone");
    let disk = reopen_store("founding-drops", "a");
    assert!(
        !disk.data().provisional_updates.iter().any(|u| u.org_pub_key == failed.org_pub_key),
        "the failed founding's update and its private key are gone from disk"
    );
    assert!(disk.data().provisional_updates.is_empty());
    assert_eq!(disk.data().orgs.len(), 1);
}

// Abnormal: a refused founding removes no genesis update of the Persona, and
// writes nothing.
// verifies: LLR-pk6mhv
// *Reconciled with S3b-io part 1, 2026-10-14:* a plain `#[test]`, no await
// (org-node is sans-IO since S4 task T14).
#[test]
fn a_refused_founding_drops_no_genesis_update() {
    let chain = ChainSlots::new();
    let (mut svc, pid, first) = genesis_built("founding-refused-keeps", &chain);
    let second = svc.create_organisation(&mut OsRng, &pid).unwrap();
    let before = store_bytes("founding-refused-keeps", "a");
    let not_on_chain = OrgId::new([0x99; 20]);
    assert_eq!(
        svc.commit_genesis(&mut OsRng, &pid, not_on_chain, test_proxy()).unwrap_err(),
        OrgNodeError::OrgNotOnChain
    );
    assert_eq!(svc.genesis_provisional_updates(&pid), vec![first, second]);
    assert_eq!(store_bytes("founding-refused-keeps", "a"), before, "nothing written");
}

// Abnormal: a founding drops only its own Persona's genesis updates; another
// Persona's genesis update is kept, on disk too.
// verifies: LLR-pk6mhv
// *Reconciled with S3b-io part 1, 2026-10-14:* a plain `#[test]`, no await
// (org-node is sans-IO since S4 task T14); since S4 task T3 the other
// Persona's update is in its own store, which the merged disk view includes.
#[test]
fn a_founding_keeps_another_personas_genesis_update() {
    let chain = ChainSlots::new();
    let (mut svc, pid, founded) = genesis_built("founding-keeps-other", &chain);
    let other = svc.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("Builder")).unwrap();
    let kept = svc.create_organisation(&mut OsRng, &other).unwrap();
    let org_id = chain.apply_genesis(founded.resulting_root, founded.org_pub_key);
    svc.commit_genesis(&mut OsRng, &pid, org_id, test_proxy()).unwrap();
    assert_eq!(svc.genesis_provisional_updates(&other), vec![kept.clone()]);
    assert_eq!(reopen_store("founding-keeps-other", "a").data().provisional_updates, vec![kept]);
}

// LLR-95753m, LLR-7cmp38: two removals of the same Member have the same
// resulting root and differ only in their fresh key pairs. Both are kept, and
// discarding one by its root and key removes that one alone, with its private
// key, and saves.
// verifies: LLR-95753m, LLR-7cmp38, REQ-hhva9d
#[test]
fn two_updates_for_the_same_change_are_two_and_one_is_discarded_alone() {
    let s = admit_b_directly(setup("same-change"));
    let mut a = s.svc_a;
    let b_id = id_by_handle(&rec_of(&a, s.org_id), "bob");
    let first = a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    let second = a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    assert_eq!(first.resulting_root, second.resulting_root, "the same change");
    assert_ne!(first.org_pub_key, second.org_pub_key);
    assert_eq!(a.provisional_updates(s.org_id), vec![first.clone(), second.clone()], "two, not one replaced");
    a.discard_provisional(&mut OsRng, ProvisionalTarget::Org(s.org_id), first.resulting_root, first.org_pub_key).unwrap();
    assert_eq!(a.provisional_updates(s.org_id), vec![second.clone()]);
    assert_eq!(reopen_store("same-change", "a").data().provisional_updates, vec![second]);
}

// LLR-6s785x, LLR-cmdrp9: of two updates for the same change, commit_update
// commits the one whose key the chain carries and takes its private key; a
// state carrying the root under a key neither holds commits nothing.
// verifies: LLR-6s785x, LLR-cmdrp9, REQ-jy6ybw
#[test]
fn commit_update_selects_the_update_by_its_root_and_its_key() {
    let s = admit_b_directly(setup("select-by-key"));
    let mut a = s.svc_a;
    let b_id = id_by_handle(&rec_of(&a, s.org_id), "bob");
    let first = a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    let second = a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    let next = Epoch::new(rec_of(&a, s.org_id).epoch.get() + 1);
    s.chain.set(s.org_id, OrgState { root_hash: first.resulting_root, org_pub_key: org_public_key(), epoch: next });
    assert_eq!(a.commit_update(&mut OsRng, s.org_id).unwrap_err(), OrgNodeError::NoProvisionalUpdate);
    s.chain.set(s.org_id, OrgState { root_hash: second.resulting_root, org_pub_key: second.org_pub_key, epoch: next });
    a.commit_update(&mut OsRng, s.org_id).unwrap();
    let rec = rec_of(&a, s.org_id);
    assert_eq!((rec.org_pub_key, rec.org_private_key_for_test()), (second.org_pub_key, &private_key_of(&second)));
    assert_ne!(rec.org_private_key_for_test(), &private_key_of(&first));
}

// Normal: revocation keeps a provisional update — base, the epoch it
// produces, a fresh key pair, no signature anywhere — and touches neither the
// chain nor the record.
// verifies: REQ-xs4ab8, REQ-txvtm9, REQ-stx9v3, LLR-6dc598, LLR-tax3pm, LLR-e2b7gv
#[test]
fn revoke_member_keeps_a_provisional_update_and_changes_nothing_else() {
    let s = admit_b_directly(setup("revoke-provisional"));
    let mut a = s.svc_a;
    let rec = rec_of(&a, s.org_id);
    let b_id = id_by_handle(&rec, "bob");
    let update = a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    assert_eq!((update.org_id, update.base_root), (Some(s.org_id), Some(rec.root_hash)));
    assert_eq!(update.seq, SequenceNumber::new(rec.epoch.get() + 1));
    assert_ne!(update.org_pub_key, rec.org_pub_key, "a fresh key pair (REQ-stx9v3)");
    assert_eq!(private_key_of(&update).x25519_keypair().org_public_key().unwrap(), update.org_pub_key);
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, rec.epoch, "no chain write");
    assert!(rec_of(&a, s.org_id).trie_members.iter().any(|m| m.id == b_id), "record unchanged");
    assert_eq!(a.provisional_updates(s.org_id), vec![update]);
}

// Abnormal: a member the record does not hold cannot be revoked, and
// nothing is kept.
// verifies: LLR-6dc598
#[test]
fn revoking_a_member_the_record_does_not_hold_keeps_nothing() {
    let s = setup("revoke-unknown");
    let mut a = s.svc_a;
    let err = a.revoke_member(&mut OsRng, s.org_id, org_node::MemberId::new([0x77; 32])).unwrap_err();
    assert!(matches!(err, OrgNodeError::Trie(_)), "{err:?}");
    assert!(a.provisional_updates(s.org_id).is_empty());
}

// Normal and abnormal: the revoking record takes the removal only through
// commit_update — not from revoke_member, not from a failed send — and the
// proxy account is neither read nor changed by any of it.
// verifies: REQ-tqap3r, LLR-qg9utu, LLR-drgdy8, LLR-3v5nu9, LLR-t4znbk
#[test]
fn the_revoking_record_takes_the_removal_only_through_commit_update() {
    let s = admit_b_directly(setup("revoke-commit"));
    let mut a = s.svc_a;
    let b_id = id_by_handle(&rec_of(&a, s.org_id), "bob");
    let proxy_before = a.proxy_account(s.org_id).unwrap();
    let update = a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    assert!(rec_of(&a, s.org_id).trie_members.iter().any(|m| m.id == b_id));
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&a, s.org_id).epoch).unwrap();
    let out = a.commit_update(&mut OsRng, s.org_id).unwrap();
    assert!(!rec_of(&a, s.org_id).trie_members.iter().any(|m| m.id == b_id), "committed");
    let device = s.joiner_b.device_key;
    // *Amended 2026-10-13 (S4, task T5):* the failed send is the transitional
    // `send_body` of the body the commit built, with no endpoint bound.
    // *2026-10-14 (S4, task T14):* that send is deleted (the send is
    // org-io's); the body the commit built is never delivered.
    assert!(body_for(&out, device).is_some(), "B's revocation is built");
    drop(out); // never delivered
    assert!(!rec_of(&a, s.org_id).trie_members.iter().any(|m| m.id == b_id), "an undelivered body undoes nothing");
    assert_eq!(a.proxy_account(s.org_id).unwrap(), proxy_before);
}

// Normal (LLR-b27jr6 on commit_update): a node that commits its own removal
// signs one acknowledgement per Persona bound to the Organisation, returned
// in the outcome, then forgets the Organisation — record, provisional
// updates and those Personas — in memory and on disk.
// *Rewritten 2026-10-07 (S3 T9):* the Personas were marked Revoked and no
// acknowledgement was signed; they are deleted through the one removal step
// (now stated by LLR-jd82t6; the retired removal item's ID was dropped from
// this annotation 2026-10-13, S4 task T9).
// verifies: REQ-uxv2x2, LLR-b27jr6, LLR-6p4pj2
#[test]
fn a_node_that_commits_its_own_removal_forgets_the_organisation() {
    let s = admit_b_directly(setup("own-removal"));
    let mut b = s.svc_b;
    let b_id = id_by_handle(&rec_of(&b, s.org_id), "bob");
    b.admit_member(&mut OsRng, s.org_id, &joiner_from(0x61, "carol")).unwrap();
    let update = b.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&b, s.org_id).epoch).unwrap();
    let outcome = b.commit_update(&mut OsRng, s.org_id).unwrap();
    assert_eq!(outcome.acknowledgements.iter().map(|ack| ack.member_id).collect::<Vec<_>>(), vec![b_id]);
    let chain = s.chain.get(&s.org_id).unwrap();
    assert_eq!((outcome.acknowledgements[0].epoch, outcome.acknowledgements[0].root), (chain.epoch, chain.root_hash));
    assert!(b.list_orgs().iter().all(|o| o.org_id != s.org_id), "the record is gone");
    assert!(b.provisional_updates(s.org_id).is_empty(), "and every provisional update for it");
    assert!(b.list_personas().iter().all(|p| p.persona_id != s.pid_b), "and the Persona");
    let disk = reopen_store("own-removal", "b");
    assert!(disk.data().orgs.iter().all(|o| o.org_id != s.org_id), "on disk too");
    assert!(disk.data().provisional_updates.iter().all(|u| u.org_id != Some(s.org_id)));
    assert!(disk.data().personas.iter().all(|p| p.persona_id != s.pid_b));
}

// Normal (PR-eqs4fs, 2026-10-07): the founding Persona is bound with the
// genesis record's MemberId, so a founder that commits its own removal signs
// one acknowledgement for its MemberId with its Device, as every other bound
// Persona does. Before the fix the founder's `member_id` stayed `None` after
// `commit_genesis` and `sign_acknowledgements` skipped it silently: no
// acknowledgement, the Organisation forgotten all the same.
// verifies: REQ-y99c9w, LLR-hby4jr, LLR-b27jr6, PR-eqs4fs
#[test]
fn a_founder_that_commits_its_own_removal_signs_its_acknowledgement() {
    let s = admit_b_directly(setup("founder-removal"));
    let mut a = s.svc_a;
    let genesis_founder = rec_of(&a, s.org_id).trie_members.iter().find(|m| m.handle == h("admin")).unwrap().id;
    let bound_member_id = persona_of(&a, &s.pid_a).member_id;
    let (_, founder_device) = a.persona_public_keys(&s.pid_a).unwrap();
    let update = a.revoke_member(&mut OsRng, s.org_id, genesis_founder).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&a, s.org_id).epoch).unwrap();
    let outcome = a.commit_update(&mut OsRng, s.org_id).unwrap();
    assert_eq!(
        outcome.acknowledgements.iter().map(|ack| (ack.member_id, ack.device)).collect::<Vec<_>>(),
        vec![(genesis_founder, founder_device)]
    );
    let ack = &outcome.acknowledgements[0];
    let chain = s.chain.get(&s.org_id).unwrap();
    assert_eq!((ack.org_id, ack.epoch, ack.root), (s.org_id, chain.epoch, chain.root_hash));
    let signature = ed25519_dalek::Signature::from_bytes(&ack.signature.0);
    ed25519_dalek::VerifyingKey::from_bytes(founder_device.as_bytes())
        .unwrap()
        .verify_strict(&ack.signed_bytes(), &signature)
        .expect("signed by the founder's Device");
    assert!(a.list_orgs().iter().all(|o| o.org_id != s.org_id), "the Organisation is forgotten");
    assert_eq!(bound_member_id, Some(genesis_founder), "commit_genesis bound the founder with the genesis MemberId");
}

// Abnormal: a commit that keeps one of the node's Personas is an update,
// nothing forgotten.
// verifies: LLR-b27jr6, LLR-jsx922
#[test]
fn a_commit_that_keeps_one_of_our_personas_is_an_update() {
    let mut s = admit_b_directly(setup("keeps-us"));
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let (sink, _task) = spawn_recv_one(rand::random());
    admit(&mut s.svc_a, &s.chain, s.org_id, &joiner_c, sink).unwrap();
    let c_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "carol");
    let (sink, _task) = spawn_recv_one(rand::random());
    revoke(&mut s.svc_a, &s.chain, s.org_id, c_id, sink).unwrap();
    assert_eq!(rec_of(&s.svc_a, s.org_id).trie_members.len(), 2);
    assert_eq!(rec_of(&s.svc_a, s.org_id).epoch, Epoch::new(4));
    assert_eq!(persona_of(&s.svc_a, &s.pid_a).status, PersonaStatus::Active);
}

// Abnormal (review round 1 of this change, finding-1; owner ruling
// 2026-10-06: one Persona, one Organisation). The reviewer's scenario: a
// Persona founds org 1, then founds a second Organisation. Before the fix the
// second founding succeeded and moved the Persona's binding, so org 1 could
// no longer be managed. Now it is refused with the Persona named, keeping no
// genesis update and writing nothing, and org 1 stays manageable.
// verifies: LLR-6z5xya, REQ-yp75u9, PR-mdv38y
#[test]
fn a_persona_bound_to_an_organisation_cannot_found_another() {
    let chain = ChainSlots::new();
    let mut svc = Node::new(open_store("found-twice", "a"), chain.clone());
    let pid = svc.create_persona(&mut OsRng, h("alice"), nm("Alice"), sn("Smith")).unwrap();
    let org_1 = found(&mut svc, &chain, &pid);
    let before = store_bytes("found-twice", "a");

    let err = svc.create_organisation(&mut OsRng, &pid).unwrap_err();
    assert_eq!(err, OrgNodeError::PersonaAlreadyBound { persona_id: pid.clone() });
    assert!(svc.genesis_provisional_updates(&pid).is_empty(), "no genesis update kept");
    assert_eq!(persona_of(&svc, &pid).org_id, Some(org_1), "the binding stands");
    assert_eq!(store_bytes("found-twice", "a"), before, "nothing written");

    // Org 1 is still managed through its Persona.
    let update = svc.admit_member(&mut OsRng, org_1, &joiner_from(0x61, "carol")).expect("org 1 is manageable");
    assert_eq!(update.org_id, Some(org_1));
    assert_eq!(update.persona_id, pid);
}

// Abnormal: a Persona builds two genesis updates while unbound, and the first
// commits. The second commit_genesis would rebind it, so it is refused with
// the Persona named: no record, the binding and the update kept, nothing
// written (LLR-ewkg85).
// verifies: LLR-eyc4ud, REQ-yp75u9
#[test]
fn commit_genesis_refuses_a_persona_bound_to_another_organisation() {
    let chain = ChainSlots::new();
    let (mut svc, pid, first) = genesis_built("genesis-bound", &chain);
    let second = svc.create_organisation(&mut OsRng, &pid).unwrap();
    let org_1 = chain.apply_genesis(first.resulting_root, first.org_pub_key);
    let org_2 = chain.apply_genesis(second.resulting_root, second.org_pub_key);
    svc.commit_genesis(&mut OsRng, &pid, org_1, test_proxy()).unwrap();
    let before = store_bytes("genesis-bound", "a");

    let err = svc.commit_genesis(&mut OsRng, &pid, org_2, test_proxy()).unwrap_err();
    assert_eq!(err, OrgNodeError::PersonaAlreadyBound { persona_id: pid.clone() });
    assert_eq!(svc.list_orgs().len(), 1, "no second record");
    assert_eq!(persona_of(&svc, &pid).org_id, Some(org_1), "the binding stands");
    // *Amended 2026-10-13 (LLR-pk6mhv, owner ruling of that day, background
    // discard):* this pinned `vec![second]`, "the update stays"; the first
    // founding's commit now drops the Persona's other genesis updates, so the
    // refused second commit finds none and removes none. *Accepted by the
    // owner 2026-10-13: the refusal stays, in org-node.*
    assert!(svc.genesis_provisional_updates(&pid).is_empty(), "the first founding dropped it");
    assert_eq!(store_bytes("genesis-bound", "a"), before, "nothing written");
}

/// verifies: LLR-d9778a, LLR-mkj4bz
///
/// Normal: genesis keeps none; a first admission keeps the Change set of the
/// update that admitted the node (REQ-uv3v5w; amended 2026-10-07, review
/// finding-1); each later commit keeps exactly the committed Envelope's
/// Change set, replacing the earlier one, on disk too.
#[test]
fn every_commit_keeps_the_change_set_that_produced_the_record() {
    let s = setup("kept-cs");
    assert_eq!(rec_of(&s.svc_a, s.org_id).kept_change_set, None, "genesis keeps none");
    let mut s = admit_b_directly(s);
    let first = rec_of(&s.svc_a, s.org_id).kept_change_set.expect("A keeps its admission's Change set");
    assert_eq!(
        rec_of(&s.svc_b, s.org_id).kept_change_set,
        Some(first.clone()),
        "a first admission keeps the admitting Change set"
    );
    assert_eq!(disk_rec_of(&reopen_store("kept-cs", "b"), s.org_id).kept_change_set, Some(first.clone()));
    assert_eq!(disk_rec_of(&reopen_store("kept-cs", "a"), s.org_id).kept_change_set, Some(first.clone()));
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let update = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_c).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    let outcome = s.svc_a.commit_update(&mut OsRng, s.org_id).unwrap();
    let kept = rec_of(&s.svc_a, s.org_id).kept_change_set.unwrap();
    assert_eq!(kept, envelope_of(&message_for(&outcome, s.joiner_b.device_key)).delta_bytes);
    assert_ne!(kept, first, "the earlier one is replaced");
}

/// verifies: LLR-mkj4bz
///
/// Abnormal: a refused commit keeps the earlier Change set.
#[test]
fn a_refused_commit_keeps_the_earlier_change_set() {
    let mut s = admit_b_directly(setup("kept-cs-refused"));
    let before = rec_of(&s.svc_a, s.org_id).kept_change_set;
    assert!(before.is_some(), "the admission's commit kept its Change set");
    assert!(s.svc_a.commit_update(&mut OsRng, s.org_id).is_err(), "nothing new on the chain");
    assert_eq!(rec_of(&s.svc_a, s.org_id).kept_change_set, before);
}

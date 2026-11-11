#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The messages of the adopted current epoch (SDD-qzv5mn): built from the
//! record's kept Change set and base snapshot, after adoption, changing
//! nothing, so a send that fails or never happens is repeated with the
//! same bodies.

mod support;
use support::{CloneProbe, NotCloneFallback};

use org_members::hasher::Blake3Hasher;
use org_node::envelope::Envelope;
use org_node::error::OrgNodeError;
use org_node::service::OutgoingMessage;
use org_node::store::{MemberSnapshot, OrgRecord};
use org_node::wire::{decode_message, WireMessage};
use org_node::{DevicePublicKey, SequenceNumber};

/// Ada's store, its admission and revocation proposed and adopted.
mod bodies {
    use org_node::chain::OrgState;
    use org_node::invitation::InviteId;
    use org_node::service::OrgService;
    use org_node::store::{KeptAdmissionRequest, MemberSnapshot, OutstandingInvite, StoreData};
    use org_node::test_fixtures::{device_key, member_key};
    use org_node::{Handle, Name, Surname, VerificationTime, Verified};
    use rand::rngs::OsRng;

    pub use crate::support::fixtures::*;
    pub use crate::support::the_org;

    /// Eve, listed with two Devices.
    pub fn eve_with_two_devices() -> MemberSnapshot {
        snapshot(0x05, "eve", 0x65, &[EVE, 0xe2])
    }

    pub fn invite_id() -> InviteId {
        invite_id_of(0x11)
    }

    /// Ada's store: the record at epoch 3 listing Ada, Bob and Eve;
    /// Carol's request kept with its outstanding invite.
    pub fn admin_store() -> StoreData {
        let mut store_data = ada_store(3, &[ada(), bob(), eve_with_two_devices()]);
        store_data.kept_admission_requests = vec![KeptAdmissionRequest {
            org_id: the_org(),
            invite_id: invite_id(),
            from: device_key(CAROL),
            name: Name::parse("Carol").unwrap(),
            surname: Surname::parse("Coder").unwrap(),
            preferred_handles: vec![Handle::parse("carol").unwrap()],
            member_key: member_key(0x63),
            device_key: device_key(CAROL),
            addresses: addresses(&["carol.example:4433"]),
        }];
        store_data.outstanding_invites =
            vec![OutstandingInvite { org_id: the_org(), invite_id: invite_id(), invitee_name: "Carol".to_string() }];
        store_data
    }

    /// Ada's store after recording Carol's admission (`admit`) and Eve's
    /// revocation (`revoke`), proposing, and adopting the proposal. The
    /// service holds the store, so the sealed writes are not read.
    pub fn adopted(admit: bool, revoke: bool) -> OrgService {
        let mut svc = service(ADA, admin_store());
        if admit {
            crate::support::unread(svc.record_admission(&mut OsRng, &ada_device(), invite_id(), None).unwrap());
        }
        if revoke {
            crate::support::unread(svc.record_revocation(&mut OsRng, &ada_device(), eve_with_two_devices().id).unwrap());
        }
        crate::support::unread(svc.propose_updated_organisation(&mut OsRng, &ada_device()).unwrap());
        let held = &held(&svc, ADA).provisional_organisations[0];
        let state = OrgState { root_hash: held.root, org_pub_key: held.org_pub_key, epoch: held.epoch };
        let (verified, _sealed) = svc
            .verify(&mut OsRng, &ada_device(), Some(state), VerificationTime(1), org_node::AheadLimit(1200), crate::support::none_held)
            .unwrap();
        assert!(matches!(verified, Verified::Adopted { .. }), "fixture: adopted");
        svc
    }
}

/// Every Device `record` lists, in record order (member, then slot).
fn devices_of(record: &OrgRecord) -> Vec<DevicePublicKey> {
    record.trie_members.iter().flat_map(|member| member.device_keys.iter().copied()).collect()
}

fn recipients(messages: &[OutgoingMessage]) -> Vec<DevicePublicKey> {
    messages.iter().map(|message| message.recipient).collect()
}

/// The adopted record and the members of the record it extends.
fn record_and_base(svc: &org_node::service::OrgService) -> (OrgRecord, Vec<MemberSnapshot>) {
    let record = bodies::held(svc, bodies::ADA).record.clone().unwrap();
    let base: Vec<MemberSnapshot> = postcard::from_bytes(record.kept_base_snapshot.as_ref().unwrap()).unwrap();
    (record, base)
}

/// The Organisation information every listed Device but Ada's receives:
/// the adopted epoch's Envelope, the base snapshot and the record's key.
fn assert_information(body: &[u8], record: &OrgRecord) {
    let WireMessage::OrgInformation { envelope, record_snapshot, org_private_key } = decode_message(body).unwrap() else {
        panic!("Organisation information")
    };
    assert_eq!(
        envelope,
        Envelope {
            org_id: record.org_id,
            parent_seq: SequenceNumber::new(record.epoch.get()),
            delta_bytes: record.kept_change_set.clone().unwrap()
        }
    );
    assert_eq!(Some(record_snapshot), record.kept_base_snapshot.clone());
    assert_eq!(&org_private_key, record.org_private_key_for_test());
}

/// A revocation notice for `recipient`, a Device of the removed Member
/// `removed`, proving its absence from `record`.
fn assert_notice(body: &[u8], recipient: DevicePublicKey, removed: &MemberSnapshot, record: &OrgRecord) {
    let WireMessage::Revocation(notice) = decode_message(body).unwrap() else { panic!("a revocation notice") };
    assert_eq!(notice.device, recipient);
    assert!(removed.device_keys.contains(&recipient), "fixture: {recipient:?} is a Device of the removed Member");
    assert_eq!(notice.member_id, removed.id, "the notice names the removed Member");
    assert_eq!(notice.org_id, record.org_id);
    notice.proof.verify::<Blake3Hasher>(&record.root_hash, &notice.member_id, &notice.device).unwrap();
}

// One Organisation-information body per Device the adopted record lists
// other than Ada's, in record order, then one revocation notice per Device
// the base lists and the record does not (Eve's two, in slot order).
// verifies: LLR-degg8t
#[test]
fn messages_for_the_current_epoch_inform_every_listed_device_but_the_own_then_notify_every_removed_one() {
    let svc = bodies::adopted(true, true);
    let (record, base) = record_and_base(&svc);
    let messages = svc.messages_for_current_epoch(&bodies::ada_device()).unwrap();
    let listed: Vec<_> = devices_of(&record).into_iter().filter(|device| *device != bodies::ada_device()).collect();
    let removed: Vec<_> = base
        .iter()
        .flat_map(|member| member.device_keys.iter().copied())
        .filter(|device| !devices_of(&record).contains(device))
        .collect();
    assert_eq!(removed, bodies::eve_with_two_devices().device_keys, "fixture: Eve's two Devices are removed");
    assert_eq!(recipients(&messages), [listed.clone(), removed].concat());
    for message in &messages[..listed.len()] {
        assert_information(&message.body, &record);
    }
    for message in &messages[listed.len()..] {
        assert_notice(&message.body, message.recipient, &bodies::eve_with_two_devices(), &record);
    }
}

// A store with no record is refused with `OrgNotHeld`; no body of an
// adopted epoch carries the invite identifier the store holds.
// verifies: LLR-degg8t
#[test]
fn a_store_with_no_record_is_refused_and_no_body_carries_an_invite_id() {
    let (provisional, key) = bodies::provisional_member();
    assert_eq!(provisional.messages_for_current_epoch(&key).unwrap_err(), OrgNodeError::OrgNotHeld { org_id: bodies::the_org() });
    let svc = bodies::adopted(true, true);
    let invite_id = bodies::invite_id();
    for message in svc.messages_for_current_epoch(&bodies::ada_device()).unwrap() {
        assert!(!message.body.windows(32).any(|window| window == invite_id.as_bytes()), "no invite id in a body");
    }
}

// `Debug` renders the recipient and the body's length, and nothing else.
// verifies: LLR-rsjs8e
#[test]
fn outgoing_message_debug_renders_the_recipient_and_the_length_only() {
    let message = OutgoingMessage { recipient: bodies::ada_device(), body: vec![0x5a; 40] };
    assert_eq!(format!("{message:?}"), format!("OutgoingMessage {{ recipient: {:?}, body_len: 40 }}", bodies::ada_device()));
}

// An adopted admission sends Organisation information to every listed
// Device but Ada's, Carol's among them, and no notice.
// verifies: LLR-degg8t
#[test]
fn an_adopted_admission_sends_information_to_every_listed_device_but_the_own() {
    let svc = bodies::adopted(true, false);
    let (record, _base) = record_and_base(&svc);
    let messages = svc.messages_for_current_epoch(&bodies::ada_device()).unwrap();
    let listed: Vec<_> = devices_of(&record).into_iter().filter(|device| *device != bodies::ada_device()).collect();
    assert_eq!(recipients(&messages), listed);
    assert!(listed.contains(&org_node::test_fixtures::device_key(bodies::CAROL)), "Carol is informed");
    for message in &messages {
        assert_information(&message.body, &record);
    }
}

// An adopted removal informs the listed Devices first, then notifies each
// removed Device.
// verifies: LLR-degg8t
#[test]
fn an_adopted_removal_sends_revocations_to_every_removed_device_after_the_listed_ones() {
    let svc = bodies::adopted(false, true);
    let (record, _base) = record_and_base(&svc);
    let messages = svc.messages_for_current_epoch(&bodies::ada_device()).unwrap();
    let listed: Vec<_> = devices_of(&record).into_iter().filter(|device| *device != bodies::ada_device()).collect();
    assert_eq!(recipients(&messages), [listed.clone(), bodies::eve_with_two_devices().device_keys].concat());
    for message in &messages[listed.len()..] {
        assert_notice(&message.body, message.recipient, &bodies::eve_with_two_devices(), &record);
    }
}

// A record that keeps no Change set (here one no adoption has replaced) returns no
// message.
// verifies: LLR-degg8t
#[test]
fn a_record_with_no_kept_change_set_returns_no_message() {
    let svc = bodies::service(bodies::ADA, bodies::admin_store());
    assert!(svc.messages_for_current_epoch(&bodies::ada_device()).unwrap().is_empty());
}

/// Every rendering of an 8-byte window of `bytes` a `Debug` could print:
/// the decimal list and lower-case hex.
fn renderings(bytes: &[u8]) -> Vec<String> {
    bytes
        .windows(8)
        .flat_map(|window| {
            let decimal = format!("{window:?}");
            [decimal.trim_matches(|character| character == '[' || character == ']').to_string(), support::hex(window)]
        })
        .collect()
}

// The messages' `Debug` shows each body's length and no window of a body, nor of the
// record's private key.
// verifies: LLR-rsjs8e
#[test]
fn outgoing_message_debug_shows_no_body_byte() {
    let svc = bodies::adopted(true, true);
    let (record, _base) = record_and_base(&svc);
    let messages = svc.messages_for_current_epoch(&bodies::ada_device()).unwrap();
    let rendered = format!("{messages:?}");
    for message in &messages {
        assert!(rendered.contains(&format!("body_len: {}", message.body.len())));
        for window in renderings(&message.body) {
            assert!(!rendered.contains(&window), "a body window {window} in {rendered}");
        }
    }
    for window in renderings(record.org_private_key_for_test().expose_secret()) {
        assert!(!rendered.contains(&window), "the key's bytes {window} in {rendered}");
    }
}

// `OutgoingMessage`, which
// may carry an Organisation private key, cannot be cloned; the probe is
// checked against a type whose answer is known.
// verifies: LLR-rsjs8e
#[test]
#[allow(
    clippy::assertions_on_constants,
    reason = "the probe's answers are compile-time constants by design; asserting them at run time is the test"
)]
fn outgoing_message_is_not_clone() {
    assert!(CloneProbe::<Vec<u8>>::IS_CLONE, "the probe sees a Clone type");
    assert!(!CloneProbe::<OutgoingMessage>::IS_CLONE, "OutgoingMessage is not Clone");
}

// `messages_for_current_epoch` changes no byte of the store, and a second call returns the same
// recipients and bodies.
// verifies: LLR-degg8t
#[test]
fn messages_for_the_current_epoch_change_nothing_and_repeat() {
    let svc = bodies::adopted(true, true);
    let before = postcard::to_allocvec(bodies::held(&svc, bodies::ADA)).unwrap();
    let first = svc.messages_for_current_epoch(&bodies::ada_device()).unwrap();
    let second = svc.messages_for_current_epoch(&bodies::ada_device()).unwrap();
    let pairs = |messages: &[OutgoingMessage]| -> Vec<(DevicePublicKey, Vec<u8>)> {
        messages.iter().map(|message| (message.recipient, message.body.clone())).collect()
    };
    assert_eq!(pairs(&first), pairs(&second));
    assert_eq!(postcard::to_allocvec(bodies::held(&svc, bodies::ADA)).unwrap(), before);
}

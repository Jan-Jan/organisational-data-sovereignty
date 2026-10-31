#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(
    clippy::await_holding_lock,
    reason = "`fresh_keychain`'s lock serialises the tests over keyring's process-global credential builder for their whole run; each test has its own runtime"
)]
//! The requests a Device sends (SDD-dhh3pw): the admission request to
//! the Invite's issuer at the Invite's addresses, and the organisation
//! request to the record's other Devices or the issuer, each carrying the
//! sender's own endpoint addresses and writing nothing. Each request goes to
//! a real receiving handle, whose receive step keeps it and writes the
//! verdict; a stand-in stands only to drop a reply (`dropping_one`) or at
//! the address of another Device. Every test runs one at a time under
//! `fresh_keychain`'s lock, over a fresh in-memory keychain. The answers to
//! these requests are tested in `receive_step.rs`.

use std::sync::Arc;
use std::time::Duration;

use org_io::send::{Delivery, SendError, SendRefused};
use org_io::test_support::FakeChain;
use org_io::transport::{BindPlan, OrgEndpoint};
use org_io::{Received, ReceiveError, RequestError, Verified};
use org_node::invitation::Invite;
use org_node::OrgNodeError;
use org_node::{DevicePublicKey, EndpointAddresses, OrgId};

mod common;
use common::handles::{fresh_keychain, handle, keep_device_seed, store_bytes, NET};
use common::journey;

/// The Admin's endpoint, as `issue_invite` bound it.
async fn issuer_endpoint(admin: &journey::Admin) -> Arc<OrgEndpoint> {
    let persona_id = admin.io.personas().await[0].persona_id.clone();
    admin.io.endpoint_for_test(&persona_id).await.unwrap()
}

/// A fault injector at the issuer's endpoint, before its handle receives:
/// accept one body on `endpoint` and drop its reply, so nothing is written
/// back; the authenticated sender and the body.
async fn dropping_one(endpoint: &OrgEndpoint) -> (DevicePublicKey, Vec<u8>) {
    let (sender, body, reply) = tokio::time::timeout(NET, endpoint.recv_answerable()).await.unwrap().unwrap();
    drop(reply);
    (sender, body)
}

// Normal: the provisional Member's endpoint is bound by the request, and the
// request goes to the issuing Device, whose real receive step keeps it from
// the Member's Device with the Invite's identifiers. It carries that
// endpoint's addresses: once the admission is adopted, the Admin's send to
// the Member, which in Loopback dials only addresses kept for its key,
// reaches the Member's receiving handle there.
// verifies: LLR-adgk9z
#[tokio::test(flavor = "multi_thread")]
async fn request_admission_binds_builds_with_its_own_addresses_and_sends_to_the_issuer() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "request-admission-sends";
    let admin = journey::admin(tag).await;
    admin.io.start_receiving().await.unwrap();
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    assert_eq!(bob.io.bound_endpoints_for_test().await, 0, "nothing bound before the request");

    let (requested, kept) = tokio::join!(bob.io.request_admission(admin.org), journey::next(&admin.io));
    assert!(matches!(requested, Ok(Delivery::Delivered)), "{requested:?}");
    match kept {
        Ok(Received::AdmissionRequestKept { org_id, invite_id, from, .. }) => {
            assert_eq!((org_id, invite_id, from), (admin.org, bob.invite_id, bob.device_key), "from the provisional Member's Device");
        }
        other => panic!("not kept: {other:?}"),
    }
    assert_eq!(bob.io.bound_endpoints_for_test().await, 1, "its endpoint bound");
    bob.io.start_receiving().await.unwrap();
    journey::propose_admission(&admin, &bob).await;
    journey::write_and_adopt(&admin).await;
    let (at_bob, sent) = journey::deliver_epoch(&admin.io, admin.org, &bob.io).await;
    assert!(matches!(sent.per_device.as_slice(), [(_, Delivery::Delivered)]), "{sent:?}");
    assert!(matches!(at_bob, Ok(Received::ProvisionalOrganisationReceived { .. })), "at the addresses the request carried: {at_bob:?}");
}

// Each verdict is reported as itself, from the issuer's real receive step:
// Bob's first request `Delivered`, the same again `AlreadyReceived`, Carol's
// with the same Invite `InviteUsedByAnotherDevice`, and Bob's once his
// admission is adopted, the invite settled, `Refused`. Silence (the reply
// dropped at the issuer's endpoint before its handle receives) is
// `NotDelivered`, and the same body goes out on every call. Nothing is
// written; a Persona holding no provisional Member is refused with nothing
// sent.
// verifies: LLR-adgk9z
#[tokio::test(flavor = "multi_thread")]
async fn request_admission_reports_each_outcome_and_writes_nothing() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "request-admission-outcomes";
    let admin = journey::admin(tag).await;
    let blob = admin.io.issue_invite(admin.org, "Bob").await.unwrap();
    let bob = handle(tag, "b", &FakeChain::sharing_slots_with(&admin.chain));
    let proposed = bob.propose_member(&blob, "Bob", "Builder", &["bob"]).await.unwrap();
    let carol = handle(tag, "c", &FakeChain::sharing_slots_with(&admin.chain));
    carol.propose_member(&blob, "Carol", "Cook", &["carol"]).await.unwrap();
    let issuer = issuer_endpoint(&admin).await;
    let files = store_bytes(tag, "b");
    let mut outcomes = vec![];
    let mut bodies = vec![];
    for _ in 0..2 {
        let (requested, (_, body)) = tokio::join!(bob.request_admission(admin.org), dropping_one(&issuer));
        outcomes.push(format!("{:?}", requested.unwrap()));
        bodies.push(body);
    }

    admin.io.start_receiving().await.unwrap();
    let (requested, _) = tokio::join!(bob.request_admission(admin.org), journey::next(&admin.io));
    outcomes.push(format!("{:?}", requested.unwrap()));
    for asking in [&bob, &carol] {
        let (requested, silent) = tokio::join!(asking.request_admission(admin.org), journey::reports_nothing(&admin.io, Duration::from_secs(2)));
        assert!(silent, "a repeated or used invite is not reported");
        outcomes.push(format!("{:?}", requested.unwrap()));
    }
    admin.io.propose_admission(admin.org, proposed.invite_id, None).await.unwrap();
    admin.io.propose_updated_organisation(admin.org).await.unwrap();
    journey::write_and_adopt(&admin).await;
    let (requested, refused) = tokio::join!(bob.request_admission(admin.org), journey::next(&admin.io));
    assert!(matches!(refused, Err(ReceiveError::Refused(_))), "the invite is settled: {refused:?}");
    outcomes.push(format!("{:?}", requested.unwrap()));

    let expected = ["NotDelivered", "NotDelivered", "Delivered", "AlreadyReceived", "InviteUsedByAnotherDevice", "Refused"];
    for (outcome, expected) in outcomes.iter().zip(expected) {
        assert!(outcome.starts_with(expected), "{outcomes:?}");
    }
    assert_eq!(outcomes.len(), expected.len(), "{outcomes:?}");
    assert_eq!(bodies[0], bodies[1], "the same body on every call");
    assert_eq!(store_bytes(tag, "b"), files, "nothing written");
    let refused = admin.io.request_admission(admin.org).await;
    assert!(matches!(refused, Err(RequestError::NoProvisionalMember { .. })), "{refused:?}");
}

// Loopback: an Invite whose addresses are another Device's is not dialled
// (`AddressNotTheIssuer`), and one with none is `NoLoopbackAddress`; nothing
// reaches the other Device.
// verifies: LLR-qa4nrt
#[tokio::test(flavor = "multi_thread")]
async fn loopback_dials_only_an_invite_address_of_the_issuer() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "request-admission-loopback";
    let admin = journey::admin(tag).await;
    let blob = admin.io.issue_invite(admin.org, "Carol").await.unwrap();
    let rogue = keep_device_seed([0x5d; 32]);
    let (rogue_at, mut rogue_gets, _rogue) = journey::answering(rogue, BindPlan::Loopback, vec![]).await;

    for (party, addresses, expected) in [("c", rogue_at, "AddressNotTheIssuer"), ("d", vec![], "NoLoopbackAddress")] {
        let mut invite = Invite::parse(&blob).unwrap();
        invite.issuer_addresses = EndpointAddresses::parse(addresses).unwrap();
        let joining = handle(tag, party, &FakeChain::sharing_slots_with(&admin.chain));
        joining.propose_member(&invite.encode().unwrap(), "Carol", "Cook", &["carol"]).await.unwrap();
        let outcome = joining.request_admission(admin.org).await.unwrap();
        let matched = match &outcome {
            Delivery::NotDelivered { reason: SendError::AddressNotTheIssuer } => "AddressNotTheIssuer",
            Delivery::NotDelivered { reason: SendError::NoLoopbackAddress } => "NoLoopbackAddress",
            _ => "other",
        };
        assert_eq!(matched, expected, "{outcome:?}");
    }
    assert!(journey::next_arrival(&mut rogue_gets, Duration::from_secs(2)).await.is_none(), "nothing reaches the other Device");
}

// A provisional Member, holding no record and behind, asks its Invite's
// issuer, whose real receive step answers at the addresses the request
// carried: Bob's own receiving handle keeps the answer. An Admin asks every
// other Device its record lists, each at the addresses kept for it, in
// record order: each Member's receive step answers at the Admin's own
// addresses, which the Admin's receive step refuses (it asked while not
// behind). An Organisation no Persona here holds is refused with nothing
// sent.
// verifies: LLR-uf5cyy
#[tokio::test(flavor = "multi_thread")]
async fn request_organisation_asks_every_listed_device_or_the_invites_issuer() {
    let (_guard, _keychain) = fresh_keychain();
    let tag = "request-organisation";
    let admin = journey::admin(tag).await;
    let carol = journey::member(&admin, tag, "c", "carol", &[]).await;
    let dave = journey::member(&admin, tag, "d", "dave", &[&carol]).await;
    let asking = admin.io.request_organisation(admin.org);
    let ((report, at_carol, at_dave), at_admin) =
        journey::while_receiving(&admin.io, async { tokio::join!(asking, journey::next(&carol.io), journey::next(&dave.io)) }).await;
    let report = report.unwrap();
    let asked: Vec<_> = report.per_device.iter().map(|(recipient, _)| *recipient).collect();
    let record = admin.io.organisation(admin.org).await.unwrap();
    let listed: Vec<_> =
        record.members.iter().flat_map(|member| member.device_keys.iter().copied()).filter(|listed| *listed != admin.device_key).collect();
    assert_eq!(asked, listed, "every other Device of the record, in record order");
    let mut others = vec![carol.device_key, dave.device_key];
    others.sort_by_key(|device_key| *device_key.as_bytes());
    let mut sorted = asked.clone();
    sorted.sort_by_key(|device_key| *device_key.as_bytes());
    assert_eq!(sorted, others, "Carol's and Dave's Devices, the record's others");
    assert!(report.per_device.iter().all(|(_, asked)| matches!(asked, Delivery::Delivered)), "{report:?}");
    for answered in [&at_carol, &at_dave] {
        assert!(
            matches!(answered, Ok(Received::OrganisationRequestAnswered { requester, delivery: Delivery::Refused, .. }) if *requester == admin.device_key),
            "answered at the Admin's own addresses: {answered:?}"
        );
    }
    assert_eq!(at_admin.len(), 2, "{at_admin:?}");
    assert!(
        at_admin.iter().all(|refused| matches!(refused, Err(ReceiveError::Refused(OrgNodeError::OrganisationNotRequested { .. })))),
        "{at_admin:?}"
    );

    let bob = journey::invited(&admin, tag, "b", "bob").await;
    journey::admit(&admin, &bob).await;
    assert!(matches!(bob.io.verify_all().await.as_slice(), [(_, Verified::Behind { .. })]), "Bob is behind");
    let (report, answered, kept) = tokio::join!(bob.io.request_organisation(admin.org), journey::next(&admin.io), journey::next(&bob.io));
    let report = report.unwrap();
    assert!(matches!(report.per_device.as_slice(), [(issuer, Delivery::Delivered)] if *issuer == admin.device_key), "the issuer alone: {report:?}");
    assert!(
        matches!(answered, Ok(Received::OrganisationRequestAnswered { requester, delivery: Delivery::Delivered, .. }) if requester == bob.device_key),
        "{answered:?}"
    );
    assert!(
        matches!(kept, Ok(Received::ProvisionalOrganisationReceived { from, .. }) if from == admin.device_key),
        "the answer reached Bob's endpoint at the addresses his request carried: {kept:?}"
    );
    let elsewhere = OrgId::new([0x98; 20]);
    assert!(matches!(admin.io.request_organisation(elsewhere).await, Err(SendRefused::NotHeld { .. })));
}

// Abnormal: the requesting Persona's
// endpoint cannot be bound (the keychain locked). Each request is refused as
// a failed bind naming the keychain's refusal, nothing reaches the issuer and
// nothing is written; once the keychain answers, the request goes out.
// verifies: LLR-adgk9z, LLR-uf5cyy
#[tokio::test(flavor = "multi_thread")]
async fn a_request_whose_endpoint_cannot_be_bound_is_refused_as_a_failed_bind() {
    let (_guard, keychain) = fresh_keychain();
    let tag = "request-bind-refused";
    let admin = journey::admin(tag).await;
    admin.io.start_receiving().await.unwrap();
    let bob = journey::invited(&admin, tag, "b", "bob").await;
    let files = store_bytes(tag, "b");
    let locked = org_io::KeychainError::Locked { device_key: bob.device_key };

    keychain.refuse_as(Some(org_io::test_support::Refusal::Locked));
    let admission = bob.io.request_admission(admin.org).await;
    let organisation = bob.io.request_organisation(admin.org).await;
    keychain.refuse_as(None);
    match admission {
        Err(RequestError::Bind(org_io::transport::TransportError::Keychain(refused))) => assert_eq!(*refused, locked),
        other => panic!("the admission request was not refused as a failed bind: {other:?}"),
    }
    match organisation {
        Err(SendRefused::Bind(org_io::transport::TransportError::Keychain(refused))) => assert_eq!(*refused, locked),
        other => panic!("the organisation request was not refused as a failed bind: {other:?}"),
    }
    assert!(journey::reports_nothing(&admin.io, Duration::from_secs(2)).await, "nothing reached the issuer");
    assert_eq!(store_bytes(tag, "b"), files, "nothing written");
    assert_eq!(bob.io.bound_endpoints_for_test().await, 0, "no endpoint kept");

    let (requested, kept) = tokio::join!(bob.io.request_admission(admin.org), journey::next(&admin.io));
    assert!(matches!(requested, Ok(Delivery::Delivered)), "once the keychain answers: {requested:?}");
    assert!(matches!(kept, Ok(Received::AdmissionRequestKept { .. })), "{kept:?}");
}

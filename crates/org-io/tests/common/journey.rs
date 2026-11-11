//! The demo journey's steps over the handle's own surface: a founding
//! written and adopted, an admission request kept by the receive step, the
//! next epoch proposed, written and adopted, and a chain write left unknown.
//! No helper installs a keychain: the test does, before its first call.
//!
//! Every state a helper builds is reached through the handle's own
//! operations — a message a Device keeps arrives over the transport and is
//! taken by its receive step, which writes it — and no helper calls org-node
//! through the handle or writes a store change itself. What a helper reads of
//! org-node's store it reads from the files the handle wrote (`handles::reopened_from`).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::atomic::Ordering;
use std::sync::Arc;

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::Poll;

use org_io::send::{Delivery, SendReport};
use org_io::test_support::{dial_direct_as, FakeChain};
use org_io::transport::{BindPlan, InboundReply, OrgEndpoint, Verdict, CONNECTION_TIME_LIMIT};
use org_io::{HeldTarget, OrgIo, ReceiveError, Received, Verified, WriteReport};
use org_node::invitation::InviteId;
use org_node::wire::{decode_message, encode_message, WireMessage};
use org_node::{DevicePublicKey, Epoch, MemberId, OrgId, OrgPrivateKey, OrgPublicKey, OrgState, RootHash};
use tokio::sync::mpsc;

use super::handles::{founded, handle, reopened_from, store_dir, NET};

/// A handle whose founding was written and adopted: its chain, the
/// Organisation, the founder's Device key and its data directory.
pub struct Admin {
    pub io: OrgIo,
    pub chain: FakeChain,
    pub org: OrgId,
    pub device_key: DevicePublicKey,
    pub dir: PathBuf,
}

/// A joiner's handle whose provisional Member was proposed from an Invite of
/// `org`: its Device key, the invite identifier and its data directory.
pub struct Joining {
    pub io: OrgIo,
    pub chain: FakeChain,
    pub device_key: DevicePublicKey,
    pub invite_id: InviteId,
    pub dir: PathBuf,
}

/// Propose a founding on `io`; its founder Device key.
pub async fn propose_founding(io: &OrgIo, handle_text: &str) -> DevicePublicKey {
    io.propose_founding_organisation("Alice", "Admin", handle_text, "Acme").await.unwrap().founder
}

/// Party `a` of test `tag`: a founding proposed, written in its two steps
/// and adopted by its verification.
pub async fn admin(tag: &str) -> Admin {
    let chain = FakeChain::new();
    let io = handle(tag, "a", &chain);
    let (org, device_key) = founded(&io, "alice").await;
    Admin { io, chain, org, device_key, dir: store_dir(tag, "a") }
}

/// Party `party` of test `tag` takes an Invite `admin` issues and proposes
/// its provisional Member from it.
pub async fn invited(admin: &Admin, tag: &str, party: &str, handle_text: &str) -> Joining {
    let invite = admin.io.issue_invite(admin.org, "Bob").await.unwrap();
    let chain = FakeChain::sharing_slots_with(&admin.chain);
    let io = handle(tag, party, &chain);
    let proposed = io.propose_member(&invite, "Bob", "Builder", &[handle_text]).await.unwrap();
    Joining { io, chain, device_key: proposed.member_device, invite_id: proposed.invite_id, dir: store_dir(tag, party) }
}

/// `joining`'s admission request kept by `admin`'s receive step: both
/// receive, `joining` asks over the transport, and `admin` keeps the request
/// and writes it.
pub async fn keep_request(admin: &Admin, joining: &Joining) {
    admin.io.start_receiving().await.unwrap();
    joining.io.start_receiving().await.unwrap();
    let (kept, requested) = request_over_transport(admin, joining).await;
    assert!(matches!(kept, Ok(Received::AdmissionRequestKept { .. })), "{kept:?}");
    assert!(matches!(requested, Delivery::Delivered), "{requested:?}");
}

/// `admin` proposes the admission of `joining` and the updated
/// Organisation; the epoch proposed.
pub async fn propose_admission(admin: &Admin, joining: &Joining) -> Epoch {
    admin.io.propose_admission(admin.org, joining.invite_id, None).await.unwrap();
    admin.io.propose_updated_organisation(admin.org).await.unwrap().epoch
}

/// Write `admin`'s proposed epoch and adopt it by verification.
pub async fn write_and_adopt(admin: &Admin) {
    let written = admin.io.write_organisation_to_chain(HeldTarget::Organisation(admin.org)).await.unwrap();
    assert!(matches!(written, WriteReport::Included { .. }), "{written:?}");
    let verified = admin.io.verify_organisation(HeldTarget::Organisation(admin.org)).await;
    assert!(matches!(verified, Verified::Adopted { .. }), "{verified:?}");
}

/// `joining` admitted at `admin`'s next epoch: its request kept by the
/// receive step, proposed, written, adopted.
pub async fn admit(admin: &Admin, joining: &Joining) {
    keep_request(admin, joining).await;
    propose_admission(admin, joining).await;
    write_and_adopt(admin).await;
}

/// The MemberId `admin`'s current record gives the Device `device`.
pub async fn member_id_of(admin: &Admin, device_key: DevicePublicKey) -> MemberId {
    let record = admin.io.organisation(admin.org).await.unwrap();
    record.members.iter().find(|member| member.device_keys.contains(&device_key)).unwrap().id
}

/// Write `target`'s provisional organisation while the chain never
/// answers, crossing the 90-second bound on a paused clock (current-thread
/// runtime only): with `lands`, the write moves the chain first (an update's
/// write only: the fake chain's founding never lands then hangs). The
/// report is `Unknown`.
pub async fn unknown_write(io: &OrgIo, chain: &FakeChain, target: HeldTarget, lands: bool) {
    let switch = if lands { &chain.write_lands_then_hangs } else { &chain.write_hanging };
    switch.store(true, Ordering::SeqCst);
    tokio::time::pause();
    let report = io.write_organisation_to_chain(target).await.unwrap();
    tokio::time::resume();
    switch.store(false, Ordering::SeqCst);
    assert!(matches!(report, WriteReport::Unknown { .. }), "{report:?}");
}

/// An Organisation public key no handle holds the private half of.
pub fn stranger_key(seed: u8) -> OrgPublicKey {
    OrgPrivateKey::from([seed; 32]).x25519_keypair().org_public_key().unwrap()
}

/// Put `org` on the chain at `epoch` with a root and key no handle
/// proposed: another Admin's write.
pub fn chain_moved_elsewhere(chain: &FakeChain, org: OrgId, epoch: u64) {
    let state = OrgState { root_hash: RootHash::new([0x77; 32]), org_pub_key: stranger_key(0x78), epoch: Epoch::new(epoch) };
    chain.slots().set(org, state);
}

/// The addresses `endpoint` holds, as the text org-io keeps for a Device
/// key: `<64 lower-case hex endpoint id>@<socket address>`, one per bound
/// socket.
pub fn kept_addresses(endpoint: &OrgEndpoint) -> Vec<String> {
    let id = hex::encode(endpoint.device_key().as_bytes());
    endpoint.bound_sockets().into_iter().map(|socket| format!("{id}@{socket}")).collect()
}

/// Bodies a receiver got, each with the sender its handshake authenticated.
pub type Arrivals = mpsc::UnboundedReceiver<(DevicePublicKey, Vec<u8>)>;

/// A stand-in recipient, for fault injection only: an endpoint of `device`
/// (its keychain item must exist) bound for `plan`, answering the n-th body
/// with `verdicts[n]` — `None` drops the reply, so nothing is written back —
/// and every later body with `Accepted`. A normal-case test sends to a real
/// receiving handle instead; this one stands where the test needs what a
/// real receive step does not do: a verdict byte it never writes for that
/// body (`AlreadyReceived` or `InviteUsedByAnotherDevice` to an
/// Organisation's information), a dropped reply, or an address of another
/// Device that must receive nothing. Its `Accepted` answers are setup
/// (TEST_GUIDELINES Owned services 2): the real step's `Accepted` is
/// verified by the normal-case tests. Its kept addresses, its arrivals, and
/// the endpoint (held open by its task too).
pub async fn answering(device_key: DevicePublicKey, plan: BindPlan, verdicts: Vec<Option<Verdict>>) -> (Vec<String>, Arrivals, Arc<OrgEndpoint>) {
    let endpoint = Arc::new(OrgEndpoint::bind_from_keychain(&device_key, plan).await.unwrap());
    let addresses = kept_addresses(&endpoint);
    let (forward, arrivals) = mpsc::unbounded_channel();
    let serving = endpoint.clone();
    tokio::spawn(async move {
        let mut verdicts = verdicts.into_iter();
        while let Ok((sender, body, reply)) = serving.recv_answerable().await {
            // The test may have stopped reading; a body no one reads is dropped.
            let _ = forward.send((sender, body));
            match verdicts.next().unwrap_or(Some(Verdict::Accepted)) {
                Some(verdict) => {
                    tokio::spawn(reply.answer(verdict));
                }
                None => drop(reply),
            }
        }
    });
    (addresses, arrivals, endpoint)
}

/// Addresses of `device` at which nothing listens, a fault injector: an
/// endpoint bound, read and closed.
pub async fn dead_addresses(device_key: DevicePublicKey) -> Vec<String> {
    let endpoint = OrgEndpoint::bind_from_keychain(&device_key, BindPlan::Loopback).await.unwrap();
    let addresses = kept_addresses(&endpoint);
    endpoint.inner().close().await;
    addresses
}

/// The next body `arrivals` holds within `limit`, or `None`.
pub async fn next_arrival(arrivals: &mut Arrivals, limit: std::time::Duration) -> Option<(DevicePublicKey, Vec<u8>)> {
    tokio::time::timeout(limit, arrivals.recv()).await.ok().flatten()
}

/// `admin` revokes the Member whose Device is `device`, and writes and
/// adopts the epoch that removes it.
pub async fn revoke(admin: &Admin, device_key: DevicePublicKey) {
    let member = member_id_of(admin, device_key).await;
    admin.io.propose_revocation(admin.org, member).await.unwrap();
    admin.io.propose_updated_organisation(admin.org).await.unwrap();
    write_and_adopt(admin).await;
}

/// `admin`'s current epoch's messages, as org-node gives them for the store
/// the handle wrote: recipient and body, in org-node's order.
pub fn current_messages(admin: &Admin) -> Vec<(DevicePublicKey, Vec<u8>)> {
    let node = reopened_from(&admin.dir, &admin.io);
    node.messages_for_current_epoch(&admin.device_key).unwrap().into_iter().map(|message| (message.recipient, message.body)).collect()
}

// ---- over the transport: the receive step keeps what arrives ----------------

/// `io`'s next received result, within `NET`.
pub async fn next(io: &OrgIo) -> Result<Received, ReceiveError> {
    tokio::time::timeout(NET, io.next_received()).await.unwrap()
}

/// Whether `io`'s receive step reports nothing within `limit`: no body
/// reached it, or each one that did kept nothing new.
pub async fn reports_nothing(io: &OrgIo, limit: std::time::Duration) -> bool {
    tokio::time::timeout(limit, io.next_received()).await.is_err()
}

/// `work`, run while `io`'s receive step takes every body that reaches it:
/// the work's output and what the step reported meanwhile, in order.
pub async fn while_receiving<T>(io: &OrgIo, work: impl Future<Output = T>) -> (T, Vec<Result<Received, ReceiveError>>) {
    let mut reported = vec![];
    let mut work = std::pin::pin!(work);
    loop {
        tokio::select! {
            biased;
            received = io.next_received() => {
                let ended = matches!(received, Err(ReceiveError::Terminal(_)));
                reported.push(received);
                if ended {
                    return (work.await, reported);
                }
            }
            output = &mut work => return (output, reported),
        }
    }
}

/// `joining` asks for admission while `admin`'s receive step takes it.
pub async fn request_over_transport(admin: &Admin, joining: &Joining) -> (Result<Received, ReceiveError>, Delivery) {
    let (received, requested) = tokio::join!(next(&admin.io), joining.io.request_admission(admin.org));
    (received, requested.unwrap())
}

/// `from` sends its current epoch of `org` while `to` receives one message.
pub async fn deliver_epoch(from: &OrgIo, org: OrgId, to: &OrgIo) -> (Result<Received, ReceiveError>, SendReport) {
    let (received, sent) = tokio::join!(next(to), from.send_organisation(org));
    (received, sent.unwrap())
}

/// `joining` admitted over the transport: it receives, asks, is kept,
/// proposed, written and adopted by `admin`, is sent the epoch, keeps it,
/// and adopts it by its own verification. `admin` must be receiving.
pub async fn admit_over_transport(admin: &Admin, joining: &Joining) {
    admit_among(admin, joining, &[]).await;
}

/// `admit_over_transport`, with the earlier Members `current` (each
/// receiving) sent the same epoch, keeping it and adopting it too, so that
/// every Device of the record stays current.
pub async fn admit_among(admin: &Admin, joining: &Joining, current: &[&Joining]) {
    joining.io.start_receiving().await.unwrap();
    let (kept, requested) = request_over_transport(admin, joining).await;
    assert!(matches!(kept, Ok(Received::AdmissionRequestKept { .. })), "{kept:?}");
    assert!(matches!(requested, Delivery::Delivered), "{requested:?}");
    propose_admission(admin, joining).await;
    write_and_adopt(admin).await;
    let receivers: Vec<&OrgIo> = std::iter::once(&joining.io).chain(current.iter().map(|member| &member.io)).collect();
    let (received, sent) = epoch_to(admin, &receivers).await;
    for received in &received {
        assert!(matches!(received, Ok(Received::ProvisionalOrganisationReceived { .. })), "{received:?}");
    }
    assert!(sent.per_device.iter().all(|(_, delivery)| matches!(delivery, Delivery::Delivered)), "{sent:?}");
    for io in receivers {
        let verified = io.verify_all().await;
        assert!(matches!(verified.as_slice(), [(_, Verified::Adopted { .. })]), "{verified:?}");
    }
}

/// Every future of `futures`, polled together; their outputs in order.
pub async fn all<F: Future>(futures: Vec<F>) -> Vec<F::Output> {
    let mut futures: Vec<Pin<Box<F>>> = futures.into_iter().map(Box::pin).collect();
    let mut outputs: Vec<Option<F::Output>> = futures.iter().map(|_| None).collect();
    std::future::poll_fn(|context| {
        let mut pending = false;
        for (future, output) in futures.iter_mut().zip(outputs.iter_mut()) {
            if output.is_none() {
                match future.as_mut().poll(context) {
                    Poll::Ready(finished) => *output = Some(finished),
                    Poll::Pending => pending = true,
                }
            }
        }
        if pending {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    })
    .await;
    outputs.into_iter().map(|output| output.expect("every future finished")).collect()
}

/// `admin` sends its current epoch while each of `receivers` (receiving)
/// takes one message: each receiver's result, in `receivers`' order, and
/// the send's report.
pub async fn epoch_to(admin: &Admin, receivers: &[&OrgIo]) -> (Vec<Result<Received, ReceiveError>>, SendReport) {
    let (received, sent) = tokio::join!(all(receivers.iter().map(|io| next(io)).collect()), admin.io.send_organisation(admin.org));
    (received, sent.unwrap())
}

/// `member` (adopted, receiving, the only other Device) revoked by `admin`,
/// sent its notice, and removed by its own verification; its kept
/// acknowledgements, recipient and body, as org-node lists them.
pub async fn removed_over_transport(admin: &Admin, member: &Joining) -> Vec<(DevicePublicKey, Vec<u8>)> {
    revoke(admin, member.device_key).await;
    let (received, _) = deliver_epoch(&admin.io, admin.org, &member.io).await;
    assert!(matches!(received, Ok(Received::RevocationNoticeKept { .. })), "{received:?}");
    let verified = member.io.verify_all().await;
    assert!(matches!(verified.as_slice(), [(_, Verified::Removed { .. })]), "{verified:?}");
    kept_acknowledgement_bodies(member)
}

/// `member`'s kept acknowledgements: recipient and encoded body, in
/// org-node's order.
pub fn kept_acknowledgement_bodies(member: &Joining) -> Vec<(DevicePublicKey, Vec<u8>)> {
    let node = reopened_from(&member.dir, &member.io);
    let kept = node.kept_acknowledgements(&member.device_key).unwrap().expect("a removed store");
    kept.acknowledgements.into_iter().map(|(message, _)| (message.recipient, message.body)).collect()
}

/// The addresses `io`'s first Persona's endpoint holds, in the kept text
/// form (`kept_addresses`); the endpoint is bound if it is not.
pub async fn addresses_of(io: &OrgIo) -> Vec<String> {
    let persona_id = io.personas().await[0].persona_id.clone();
    kept_addresses(&io.endpoint_for_test(&persona_id).await.unwrap())
}

/// `body` dialled directly to `to` from `device_key`'s identity while `to`
/// receives: what `to` reported and the bytes written back, the dial within
/// `NET`.
pub async fn dialled(to: &OrgIo, device_key: &DevicePublicKey, body: &[u8]) -> (Result<Received, ReceiveError>, Vec<u8>) {
    let addr = address_of(to).await;
    let (received, read_back) = tokio::join!(next(to), tokio::time::timeout(NET, dial_direct_as(device_key, addr, body)));
    (received, read_back.expect("the dial ended in time").unwrap())
}

/// The dialable address of `io`'s first Persona's endpoint.
pub async fn address_of(io: &OrgIo) -> iroh::EndpointAddr {
    let persona_id = io.personas().await[0].persona_id.clone();
    io.endpoint_for_test(&persona_id).await.unwrap().node_addr_for_dial()
}

// ---- through the receive step, no hand-written state ------------------------

/// A Member of `admin`'s Organisation (party `party` of `tag`), admitted
/// over the transport (`admit_among`): `admin` and the earlier Members
/// `current` must be receiving, or are made to.
pub async fn member(admin: &Admin, tag: &str, party: &str, handle_text: &str, current: &[&Joining]) -> Joining {
    admin.io.start_receiving().await.unwrap();
    let joining = invited(admin, tag, party, handle_text).await;
    admit_among(admin, &joining, current).await;
    joining
}

/// `joining` admitted at `admin`'s next epoch with `admin` keeping
/// `addresses` for it: the admission request org-node builds from the store
/// `joining`'s handle wrote, carrying `addresses`, is dialled from
/// `joining`'s Device identity to `admin`'s endpoint and kept by `admin`'s
/// receive step (`admin` receiving, or made to); then the admission is
/// proposed, written and adopted. `joining` is sent nothing.
pub async fn admit_keeping(admin: &Admin, joining: &Joining, addresses: Vec<String>) {
    admin.io.start_receiving().await.unwrap();
    let request = reopened_from(&joining.dir, &joining.io).build_admission_request(&joining.device_key, addresses).unwrap();
    let to = address_of(&admin.io).await;
    let (read_back, kept) = tokio::join!(dial_direct_as(&joining.device_key, to, &request.body), next(&admin.io));
    assert_eq!(read_back.unwrap(), vec![Verdict::Accepted.as_byte()], "the request kept");
    assert!(matches!(kept, Ok(Received::AdmissionRequestKept { .. })), "{kept:?}");
    propose_admission(admin, joining).await;
    write_and_adopt(admin).await;
}

/// Whether `body` is a removal acknowledgement.
pub fn is_acknowledgement(body: &[u8]) -> bool {
    matches!(decode_message(body), Ok(WireMessage::Acknowledgement(_)))
}

/// `io`'s receive step, taking every body that reaches it, until its loop
/// ends; then nothing more.
async fn take_everything(io: &OrgIo) {
    loop {
        if let Err(ReceiveError::Terminal(_)) = io.next_received().await {
            std::future::pending::<()>().await;
        }
    }
}

/// `member` (receiving) keeps `addresses` for `peer_device`, a Device of
/// `org`'s record: an organisation request carrying them is dialled from
/// that Device's identity (its keychain item) to `member`'s endpoint, and
/// `member`'s receive step keeps them, writes them and sends its answer to
/// them. The answer reaches whatever listens there: a test endpoint takes a
/// body that is not an acknowledgement and answers it `Accepted`. Or it
/// reaches `peer`, the handle of `peer_device`, at a path `member`'s
/// endpoint remembers; `peer`'s receive step takes it there.
pub async fn keep_addresses(member: &OrgIo, org: OrgId, peer: &OrgIo, peer_device: DevicePublicKey, addresses: Vec<String>) {
    let body = encode_message(&WireMessage::OrganisationRequest { org_id: org, addresses }).unwrap();
    let to = address_of(member).await;
    let kept = async { tokio::join!(dial_direct_as(&peer_device, to, &body), next(member)) };
    let (read_back, received) = tokio::select! {
        kept = kept => kept,
        () = take_everything(peer) => unreachable!("the receive step never ends by itself"),
    };
    assert_eq!(read_back.unwrap(), vec![Verdict::Accepted.as_byte()], "the request kept");
    assert!(matches!(received, Ok(Received::OrganisationRequestAnswered { .. })), "{received:?}");
}

/// An endpoint of `device` whose addresses are soon dead, a fault injector:
/// it answers every body `Accepted` (setup: the answer to the request that
/// keeps these addresses) until the test closes it, after which nothing
/// listens there (iroh gives up a dial after about 30 s). Its addresses and
/// the endpoint.
pub async fn soon_dead(device_key: DevicePublicKey) -> (Vec<String>, Arc<OrgEndpoint>) {
    let (addresses, _arrivals, endpoint) = answering(device_key, BindPlan::Loopback, vec![]).await;
    (addresses, endpoint)
}

/// Every body that reaches `endpoint`, with its authenticated sender and the
/// reply its verdict goes back through, in the order the bodies complete.
/// Each inbound connection is read in a task of its own, as the receive
/// step's accept loop reads them (LLR-8uf9a5), so one connection that gives
/// no body (one the dialler gave up, here the answer to the request that
/// kept these addresses when it went out over another path the dialler
/// knew) holds up no other until its 30-second limit.
pub fn each_inbound(endpoint: Arc<OrgEndpoint>) -> mpsc::UnboundedReceiver<(DevicePublicKey, Vec<u8>, InboundReply)> {
    let (forward, taken) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Ok(pending) = endpoint.next_connection().await {
            let forward = forward.clone();
            tokio::spawn(async move {
                // A connection that brings no body within the limit has nothing to forward; the
                // tests read only bodies that arrive.
                if let Ok(received) = pending.receive(CONNECTION_TIME_LIMIT).await {
                    // The test may have stopped reading; a body no one reads is dropped.
                    let _ = forward.send(received);
                }
            });
        }
    });
    taken
}

/// `answering`, for the acknowledgement tests: a body that is not an
/// acknowledgement (the answer to the organisation request that kept these
/// addresses) is answered `Accepted` and neither forwarded nor counted
/// against `verdicts`. A stand-in recipient: the normal-case tests send to a
/// real receiving handle. It injects what a real receive step does not
/// write for an acknowledgement (`AlreadyReceived`,
/// `InviteUsedByAnotherDevice`, a dropped reply), and it scripts the two
/// verdicts the real step does write, `Accepted` and `Refused`, under the
/// contract test `the_scripted_recipients_verdicts_are_the_real_receive_steps`
/// (acknowledgements.rs), which runs the same assertions over it and over the
/// real step.
pub async fn answering_acknowledgements(
    device_key: DevicePublicKey,
    plan: BindPlan,
    verdicts: Vec<Option<Verdict>>,
) -> (Vec<String>, Arrivals, Arc<OrgEndpoint>) {
    let endpoint = Arc::new(OrgEndpoint::bind_from_keychain(&device_key, plan).await.unwrap());
    let addresses = kept_addresses(&endpoint);
    let (forward, arrivals) = mpsc::unbounded_channel();
    let mut taken = each_inbound(endpoint.clone());
    tokio::spawn(async move {
        let mut verdicts = verdicts.into_iter();
        while let Some((sender, body, reply)) = taken.recv().await {
            if !is_acknowledgement(&body) {
                tokio::spawn(reply.answer(Verdict::Accepted));
                continue;
            }
            // The test may have stopped reading; a body no one reads is dropped.
            let _ = forward.send((sender, body));
            match verdicts.next().unwrap_or(Some(Verdict::Accepted)) {
                Some(verdict) => {
                    tokio::spawn(reply.answer(verdict));
                }
                None => drop(reply),
            }
        }
    });
    (addresses, arrivals, endpoint)
}

/// `member` revoked by `admin`, and the epoch that removes it sent: `member`
/// keeps the notice (its next verification removes it; the test makes that
/// call), and each of `current` (receiving) keeps the epoch.
pub async fn revoked(admin: &Admin, member: &Joining, current: &[&Joining]) {
    revoke(admin, member.device_key).await;
    let receivers: Vec<&OrgIo> = std::iter::once(&member.io).chain(current.iter().map(|other| &other.io)).collect();
    let (received, _) = epoch_to(admin, &receivers).await;
    assert!(matches!(received[0], Ok(Received::RevocationNoticeKept { .. })), "{:?}", received[0]);
    for other in &received[1..] {
        assert!(matches!(other, Ok(Received::ProvisionalOrganisationReceived { .. })), "{other:?}");
    }
}

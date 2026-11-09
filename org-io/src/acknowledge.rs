//! The removal acknowledgements' delivery (SDD-3my66m): after verification
//! reports this node removed, one task per removed Persona sends each
//! acknowledgement org-node keeps to its recipient, from the Persona's own
//! identity over a direct-only bind, or, for a dial by key alone in
//! Networked mode, a bind with the relay server and address lookup
//! (LLR-zzsqj4, LLR-spckm6, LLR-u2j6u8), in rounds on the schedule of
//! LLR-bv6wkz read from the handle's clock.
//! org-io learns them only from org-node's list, and removes one only
//! through org-node's `acknowledgement_accepted` (LLR-sn5ded, LLR-6ux6yw).
//! The task holds the node's lock only to read the list and around an
//! acceptance and its store write, never across a dial. When the removed
//! data is due for deletion (`retention::deletion_due`), the task hands it
//! to the deletion (SDD-wyn7b6, `retention.rs`).

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use org_node::service::{KeptAcknowledgements, OrgService};
use org_node::{DevicePublicKey, OrgId, OrgNodeError};
use tokio::task::JoinHandle;

use crate::clock::Clock;
use crate::retention::{delete_from_task, deletion_due, retry_deletion, DeletionDue, OpenFailure, DEADLINE_SECONDS, RETRY_SECONDS};
use crate::send::{send_one, sockets_for, Delivery, SendError};
use crate::transport::{BindPlan, OrgEndpoint, TransportError, TransportMode};
use crate::view::{AcceptanceFailure, AcknowledgementOutcome, AcknowledgementReport, DeletionFailure, RoundFailure};
use crate::Cause;

/// The first rounds' offsets from the verification time; then hourly
/// after the last.
const FIRST_ROUNDS: [u64; 4] = [0, 60, 300, 1_800];

/// The first due time after `now` (LLR-bv6wkz): T, T+1 min, T+5 min,
/// T+30 min, then T+30 min+k hours; `None` once it would be at or past
/// T+7 days. Pure; due times already passed are skipped, not caught up.
pub fn next_round_after(verified_at: u64, now: u64) -> Option<u64> {
    let deadline = verified_at.saturating_add(DEADLINE_SECONDS);
    let after_first = verified_at.saturating_add(FIRST_ROUNDS[3]);
    let due = match FIRST_ROUNDS.iter().map(|offset| verified_at.saturating_add(*offset)).find(|due| *due > now) {
        Some(due) => due,
        None => {
            let hours = (now - after_first) / 3_600 + 1;
            after_first.saturating_add(hours.saturating_mul(3_600))
        }
    };
    (due < deadline).then_some(due)
}

/// What the report reads of one removal's delivery.
#[derive(Clone, Default)]
pub(crate) struct Progress {
    pub(crate) device_key: Option<DevicePublicKey>,
    pub(crate) outcomes: HashMap<DevicePublicKey, AcknowledgementOutcome>,
    pub(crate) next_round: Option<u64>,
    pub(crate) deletion_failure: Option<DeletionFailure>,
    pub(crate) round_failure: Option<RoundFailure>,
    /// The kept removal's deadline, recorded while its store was held, so the
    /// report keeps it once the store is dropped and only a deletion failure
    /// is retried (LLR-rav7dr).
    pub(crate) deadline: Option<u64>,
}

/// The delivery tasks running, one at most per removed Persona, and what
/// each has done so far.
#[derive(Default)]
pub(crate) struct Deliveries {
    running: Mutex<HashMap<DevicePublicKey, JoinHandle<()>>>,
    /// The hourly retry of leftover `removed-` files (LLR-3fxxpp), when one runs.
    pub(crate) leftovers: Mutex<Option<JoinHandle<()>>>,
    progress: Mutex<HashMap<OrgId, Progress>>,
    /// The Device keys of the removed stores this handle holds, in the order
    /// found: those `open` found, then each a delivery's start records
    /// (LLR-5rxqr8, LLR-rav7dr). A deleted one is forgotten.
    removed: Mutex<Vec<DevicePublicKey>>,
    /// Wakes a waiting task to read the clock again (a settable clock in
    /// tests; never notified in production).
    pub(crate) wake: tokio::sync::Notify,
}

impl Deliveries {
    pub(crate) fn running(&self) -> MutexGuard<'_, HashMap<DevicePublicKey, JoinHandle<()>>> {
        self.running.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The delivery of `org_id` as it stands, when one has started.
    pub(crate) fn progress_of(&self, org_id: OrgId) -> Option<Progress> {
        self.progress.lock().unwrap_or_else(PoisonError::into_inner).get(&org_id).cloned()
    }

    pub(crate) fn update(&self, org_id: OrgId, change: impl FnOnce(&mut Progress)) {
        change(self.progress.lock().unwrap_or_else(PoisonError::into_inner).entry(org_id).or_default());
    }

    /// Drop `org_id`'s progress: its Persona is deleted (the report is `None`).
    pub(crate) fn forget(&self, org_id: OrgId) {
        self.progress.lock().unwrap_or_else(PoisonError::into_inner).remove(&org_id);
    }

    /// Remember `device_key` as a removed store's, once.
    pub(crate) fn remember_removed(&self, device_key: DevicePublicKey) {
        let mut removed = self.removed.lock().unwrap_or_else(PoisonError::into_inner);
        if !removed.contains(&device_key) {
            removed.push(device_key);
        }
    }

    /// Remember each of `store_keys` whose store in `service` keeps a
    /// removal: `OrgIo::open`'s, after its deletions.
    pub(crate) fn remember_removals_held(&self, service: &OrgService, store_keys: &[DevicePublicKey]) {
        for device_key in store_keys {
            // `open` unsealed each of `store_keys` into `service` just now, so
            // none is refused; `None` is a store with no removal to remember.
            if matches!(service.kept_acknowledgements(device_key), Ok(Some(_))) {
                self.remember_removed(*device_key);
            }
        }
    }

    /// Forget `device_key`: its Persona is deleted.
    pub(crate) fn forget_removed(&self, device_key: &DevicePublicKey) {
        self.removed.lock().unwrap_or_else(PoisonError::into_inner).retain(|held| held != device_key);
    }

    /// The removed stores' Device keys this handle remembers.
    pub(crate) fn removed_keys(&self) -> Vec<DevicePublicKey> {
        self.removed.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    /// Record the deletion failures found at open, so the report names them
    /// (LLR-3fxxpp), with the deadline of the removal each concerns
    /// (LLR-rav7dr).
    pub(crate) fn record_failures(&self, failures: Vec<OpenFailure>) {
        for OpenFailure { org_id, failure, deadline } in failures {
            self.update(org_id, |progress| {
                progress.deletion_failure = Some(failure);
                progress.deadline = progress.deadline.or(deadline);
            });
        }
    }

    /// Stop every task (the handle is dropped: a restart resumes them).
    fn abort_all(&self) {
        for task in self.running().values() {
            task.abort();
        }
        if let Some(task) = self.leftovers.lock().unwrap_or_else(PoisonError::into_inner).as_ref() {
            task.abort();
        }
    }
}

/// The handle's share of the deliveries: dropping the handle stops its tasks.
#[derive(Default)]
pub(crate) struct Tasks(pub(crate) Arc<Deliveries>);

impl Drop for Tasks {
    fn drop(&mut self) {
        self.0.abort_all();
    }
}

/// What a delivery task shares with the handle.
#[derive(Clone)]
pub(crate) struct Context {
    pub(crate) service: Arc<tokio::sync::Mutex<OrgService>>,
    pub(crate) mode: TransportMode,
    /// The handle's bind plan, with the relay server and address lookup in
    /// Networked mode: the one a dial by key alone binds (LLR-spckm6).
    pub(crate) bind_plan: BindPlan,
    pub(crate) deliveries: Arc<Deliveries>,
    /// The handle's store write: its data directory and its random source.
    pub(crate) store: crate::StoreWriter,
    pub(crate) clock: Arc<dyn Clock>,
}

/// The direct-only plan for `mode` (LLR-spckm6).
fn direct_plan(mode: TransportMode) -> BindPlan {
    match mode {
        TransportMode::Loopback => BindPlan::Loopback,
        TransportMode::Networked => BindPlan::Direct,
    }
}

/// Start `device_key`'s delivery, unless one runs (LLR-zzsqj4): called
/// once the removed store is written, and at start for each kept removal
/// (LLR-5rxqr8). A store holding no kept removal starts none.
pub(crate) async fn start(context: Context, device_key: DevicePublicKey) {
    let kept = context.service.lock().await.kept_acknowledgements(&device_key);
    // Called right after the removed store was written, or for a key `open`
    // or a delivery's start found keeping a removal: a refusal or `None` is a
    // store deleted since, with nothing left to deliver.
    let Ok(Some(kept)) = kept else { return };
    // Recorded before `start` returns, so the report and a later resume find
    // it without listing the data directory (LLR-5rxqr8, LLR-rav7dr).
    context.deliveries.remember_removed(device_key);
    let mut running = context.deliveries.running();
    if running.get(&device_key).is_some_and(|task| !task.is_finished()) {
        return;
    }
    let left = kept.acknowledgements.len();
    let task = tokio::spawn(deliver(context.clone(), device_key, kept.org_id, kept.verified_at.0, left));
    running.insert(device_key, task);
}

/// Resume, at start, the delivery of each of `device_keys` whose store
/// keeps a removal whose deadline the clock has not reached (LLR-5rxqr8);
/// its rounds stay anchored at the kept verification time. A removal past
/// its deadline still held here is one whose deletion at open failed: its
/// deletion is tried again an hour later, then hourly (LLR-3fxxpp),
/// with no round. One keeping no acknowledgement is started,
/// and its task deletes it at once (LLR-n649th). With the clock reading
/// before the Unix epoch no deadline is judged: each is started, and its
/// task reports the condition as its round's failure (LLR-bv6wkz).
pub(crate) async fn resume(context: Context, device_keys: Vec<DevicePublicKey>) {
    let now = context.clock.now();
    for device_key in device_keys {
        let kept = context.service.lock().await.kept_acknowledgements(&device_key);
        // Every key here is a removed store's that `open` or a delivery's start found keeping a
        // removal: a refusal or `None` is a store deleted since, with nothing left to resume.
        let Ok(Some(kept)) = kept else { continue };
        let Some(now) = now else {
            start(context.clone(), device_key).await;
            continue;
        };
        match deletion_due(kept.acknowledgements.len(), kept.verified_at.0, now) {
            Some(DeletionDue::Expired) => {
                // Kept for the report: a failure after the rename drops the
                // store, and the report still gives the removal's deadline
                // (LLR-rav7dr).
                let deadline = kept.verified_at.0.saturating_add(DEADLINE_SECONDS);
                context.deliveries.update(kept.org_id, |progress| progress.deadline = Some(deadline));
                retry_deletion(context.clone(), device_key, kept.org_id, Some(now.saturating_add(RETRY_SECONDS)));
            }
            Some(DeletionDue::NoneKept) | None => start(context.clone(), device_key).await,
        }
    }
}

/// The report of LLR-rav7dr over `service`: the removal of `org_id` held
/// by the Persona whose delivery started, or else by one of the removed
/// stores `deliveries` remembers.
pub(crate) fn report(service: &OrgService, deliveries: &Deliveries, org_id: OrgId) -> Option<AcknowledgementReport> {
    let progress = deliveries.progress_of(org_id).unwrap_or_default();
    let candidates = match progress.device_key {
        Some(device_key) => vec![device_key],
        None => deliveries.removed_keys(),
    };
    // A candidate org-node refuses holds no store any more (deleted, or
    // dropped after a failed write): it holds no removal to report, which is
    // what the refusal says; the deletion failure, if any, is reported below.
    let kept = candidates
        .iter()
        .find_map(|device_key| service.kept_acknowledgements(device_key).ok().flatten().filter(|kept| kept.org_id == org_id));
    let Some(kept) = kept else {
        return progress.deletion_failure.map(|failure| AcknowledgementReport {
            org_id,
            recipients: Vec::new(),
            next_round: None,
            deadline: progress.deadline,
            last_deletion_failure: Some(failure),
            last_round_failure: None,
        });
    };
    let recipients = kept
        .acknowledgements
        .iter()
        .map(|(message, _)| {
            let outcome = progress.outcomes.get(&message.recipient).cloned().unwrap_or(AcknowledgementOutcome::NotYetSent);
            (message.recipient, outcome)
        })
        .collect();
    Some(AcknowledgementReport {
        org_id,
        recipients,
        next_round: progress.next_round,
        deadline: Some(kept.verified_at.0.saturating_add(DEADLINE_SECONDS)),
        last_deletion_failure: progress.deletion_failure,
        last_round_failure: progress.round_failure,
    })
}

/// The delivery task: bind the Persona's endpoint direct-only, then run
/// one round at once and the next at each due time after the clock's
/// reading (LLR-bv6wkz); when none is kept (`left`, at the start too) or
/// the deadline is reached, delete the Persona (LLR-n649th, LLR-vj2nkb).
/// A clock reading before the Unix epoch is never time 0: no deletion is
/// judged and no round run on it, it is that round's failure in the
/// report, and the round is tried again a retry period later (LLR-bv6wkz).
/// `org_id`, `verified_at` and `left` are the kept removal's, as `start`
/// read them.
async fn deliver(context: Context, device_key: DevicePublicKey, org_id: OrgId, verified_at: u64, mut left: usize) {
    let deadline = verified_at.saturating_add(DEADLINE_SECONDS);
    context.deliveries.update(org_id, |progress| {
        progress.device_key = Some(device_key);
        progress.deadline = Some(deadline);
    });
    let mut endpoint = None;
    loop {
        let Some(now) = context.clock.now() else {
            context.deliveries.update(org_id, |progress| {
                progress.round_failure = Some(RoundFailure::ClockBeforeEpoch);
                progress.next_round = None;
            });
            wait_for(&context, None).await;
            continue;
        };
        if deletion_due(left, verified_at, now).is_some() {
            delete_from_task(&context, org_id, &device_key, endpoint).await;
            return;
        }
        // A bind that fails is every recipient's outcome for the round, its
        // cause in the reason; the next round binds again (LLR-spckm6).
        let bound = match endpoint {
            Some(ref bound) => Ok(bound),
            None => match OrgEndpoint::bind_from_keychain(&device_key, direct_plan(context.mode)).await {
                Ok(fresh) => Ok(&*endpoint.insert(fresh)),
                Err(error) => Err(error),
            },
        };
        // A refused list leaves `left` as it was: the round failed, and the
        // next due round reads the list again.
        let failure = match round(&context, &device_key, org_id, bound).await {
            Ok(still_kept) => {
                left = still_kept;
                None
            }
            Err(failure) => Some(failure),
        };
        context.deliveries.update(org_id, |progress| progress.round_failure = failure);
        if left == 0 {
            continue;
        }
        // A reading before the Unix epoch here is reported at the loop's top.
        let Some(now) = context.clock.now() else { continue };
        let next = next_round_after(verified_at, now);
        context.deliveries.update(org_id, |progress| progress.next_round = next);
        // With no round left before it, the task wakes at the deadline.
        wait_until(&context, next.unwrap_or(deadline)).await;
    }
}

/// Wait until the handle's clock reads `due` (LLR-bv6wkz): sleep for the
/// difference, or until woken, and read the clock again. A reading before
/// the Unix epoch ends the wait at once, never taken as time 0: the caller
/// reads the clock again and reports it.
pub(crate) async fn wait_until(context: &Context, due: u64) {
    loop {
        let Some(now) = context.clock.now() else { return };
        if now >= due {
            return;
        }
        tokio::select! {
            () = tokio::time::sleep(std::time::Duration::from_secs(due - now)) => {}
            () = context.deliveries.wake.notified() => {}
        }
    }
}

/// Wait for a retry due at `due` (LLR-3fxxpp, LLR-bv6wkz). With no due
/// time — the clock read before the Unix epoch when it was set — or a
/// reading before the epoch during the wait, wait one retry period by the
/// runtime's timer instead, or until woken.
pub(crate) async fn wait_for(context: &Context, due: Option<u64>) {
    if let Some(due) = due {
        wait_until(context, due).await;
        if context.clock.now().is_some() {
            return;
        }
    }
    tokio::select! {
        () = tokio::time::sleep(std::time::Duration::from_secs(RETRY_SECONDS)) => {}
        () = context.deliveries.wake.notified() => {}
    }
}

/// One round (LLR-u2j6u8): every kept acknowledgement, in org-node's
/// order, each recipient tried at its kept addresses in order until one
/// accepts, or by its key alone in Networked mode when none may be dialled; one recipient's outcome never stops the others. An `Accepted`
/// has org-node delete the acknowledgement and the store written before
/// the next recipient (LLR-6ux6yw); any other outcome is recorded and
/// nothing else (LLR-xwz3nu). How many are still kept, or why org-node
/// refused the list the round reads.
async fn round(
    context: &Context,
    device_key: &DevicePublicKey,
    org_id: OrgId,
    endpoint: Result<&OrgEndpoint, TransportError>,
) -> Result<usize, RoundFailure> {
    let kept = match listing(context, device_key).await {
        Ok(Some(kept)) => kept,
        // The Persona's store is gone, or keeps no removal: nothing is left
        // to deliver, and the task ends.
        Ok(None) | Err(ListingFailure::PersonaGone) => return Ok(0),
        // Any other failure is this round's failure; the acknowledgements
        // stay kept for the next due round (LLR-rav7dr, LLR-bv6wkz).
        Err(ListingFailure::Failed(error)) => return Err(RoundFailure::Listing(error)),
    };
    let mut left = kept.acknowledgements.len();
    for (message, addresses) in kept.acknowledgements {
        let recipient = message.recipient;
        let delivery = match &endpoint {
            Err(error) => Delivery::NotDelivered { reason: SendError::SenderEndpoint(format!("the direct bind failed: {error}")) },
            Ok(endpoint) => {
                let addresses = addresses.map(|addresses| addresses.as_slice().to_vec()).unwrap_or_default();
                try_addresses(context, device_key, endpoint, &recipient, &addresses, &message.body).await
            }
        };
        let outcome = match delivery {
            Delivery::Delivered => match accepted(context, device_key, recipient).await {
                Ok(still_kept) => {
                    left = still_kept;
                    continue;
                }
                // The acknowledgement stays kept: this round's outcome says why.
                Err(reason) => AcknowledgementOutcome::AcceptedNotRecorded { reason },
            },
            Delivery::NotDelivered { reason } => AcknowledgementOutcome::NotDelivered { reason: Cause::new(reason) },
            Delivery::Refused | Delivery::AlreadyReceived | Delivery::InviteUsedByAnotherDevice => AcknowledgementOutcome::Refused,
        };
        context.deliveries.update(org_id, |progress| {
            progress.outcomes.insert(recipient, outcome);
        });
    }
    Ok(left)
}

/// Why org-io's read of org-node's list of kept acknowledgements gave no
/// list.
pub(crate) enum ListingFailure {
    /// org-node holds no store of the Persona (`PersonaNotHeld`): it is gone.
    PersonaGone,
    /// Any other refusal of org-node's, kept.
    Failed(Cause<OrgNodeError>),
}

/// org-node's list of `device_key`'s kept acknowledgements in `service`
/// (LLR-sn5ded): the one read the delivery and the deletion make. No store
/// org-node opens makes it refuse but by `PersonaNotHeld`, so `Failed` is
/// reached by no test (LLR-bv6wkz, LLR-3fxxpp).
pub(crate) fn list_kept(service: &OrgService, device_key: &DevicePublicKey) -> Result<Option<KeptAcknowledgements>, ListingFailure> {
    service.kept_acknowledgements(device_key).map_err(|refused| match refused {
        OrgNodeError::PersonaNotHeld { .. } => ListingFailure::PersonaGone,
        other => ListingFailure::Failed(Cause::new(other)),
    })
}

/// `list_kept`, under the node's lock.
async fn listing(context: &Context, device_key: &DevicePublicKey) -> Result<Option<KeptAcknowledgements>, ListingFailure> {
    let service = context.service.lock().await;
    list_kept(&service, device_key)
}

/// `recipient` accepted (LLR-6ux6yw): under the lock, org-node deletes its
/// acknowledgement and the sealed store is written; how many are left. A
/// failed write reloads the Persona (the acknowledgement stays kept); a
/// refusal writes nothing. Either is the reason the round reports
/// (LLR-rav7dr).
async fn accepted(context: &Context, device_key: &DevicePublicKey, recipient: DevicePublicKey) -> Result<usize, AcceptanceFailure> {
    let mut service = context.service.lock().await;
    let accepted = {
        let mut rng = context.store.rng.lock();
        service.acknowledgement_accepted(&mut *rng, device_key, recipient)
    };
    // Both failures leave the acknowledgement kept, as LLR-6ux6yw states: a
    // refusal (`AcknowledgementNotKept`) changed nothing, and a failed write
    // reloaded the Persona from its file, which still keeps it (or, when the
    // reload failed too, dropped it, its file kept for the next open). The
    // next round sends it again, so neither is lost; the count is kept.
    let (left, change) = accepted.map_err(|refused| AcceptanceFailure::Refused(Cause::new(refused)))?;
    context.store.store(&mut service, change).map_err(|failed| AcceptanceFailure::Store(Cause::new(failed)))?;
    Ok(left)
}

/// The dials one delivery to `recipient` makes, each a socket list
/// (LLR-u2j6u8): each socket the send path's address rule (LLR-34tfm8)
/// gives, alone and in order. With none, Networked mode dials once by the
/// key alone, an empty socket list, as `send_all` does: discovery finds
/// the key (REQ-kn8zg6). Loopback mode has no discovery, so it dials
/// nothing. The rule's two Loopback refusals, no address kept and none the
/// recipient's, both mean no address may be dialled.
pub(crate) fn dial_plan(mode: TransportMode, recipient: &DevicePublicKey, addresses: &[String]) -> Vec<Vec<SocketAddr>> {
    let sockets = sockets_for(mode, recipient, Some(addresses)).unwrap_or_default();
    if sockets.is_empty() && mode == TransportMode::Networked {
        return vec![Vec::new()];
    }
    sockets.into_iter().map(|socket| vec![socket]).collect()
}

/// Make each dial of the plan in order, stopping at the first `Delivered`;
/// the last dial's outcome otherwise, `NoAddress` when nothing was dialled.
/// A dial with a socket goes from the direct-only `endpoint`; the dial by
/// key alone, from an endpoint bound for it (`dial_by_key`).
async fn try_addresses(
    context: &Context,
    device_key: &DevicePublicKey,
    endpoint: &OrgEndpoint,
    recipient: &DevicePublicKey,
    addresses: &[String],
    body: &[u8],
) -> Delivery {
    let mut last = Delivery::NotDelivered { reason: SendError::NoAddress };
    for sockets in dial_plan(context.mode, recipient, addresses) {
        last = if sockets.is_empty() {
            dial_by_key(context, device_key, recipient, body).await
        } else {
            send_one(endpoint, recipient, &sockets, body).await
        };
        if matches!(last, Delivery::Delivered) {
            break;
        }
    }
    last
}

/// The one dial by `recipient`'s key alone (LLR-spckm6): the direct-only
/// endpoint has no address lookup, so it could never find the key; this
/// dial goes from an endpoint of `device_key`'s identity bound with the
/// relay server and address lookup, closed once the verdict is read. A bind
/// that fails is this recipient's outcome; the next round binds again.
async fn dial_by_key(context: &Context, device_key: &DevicePublicKey, recipient: &DevicePublicKey, body: &[u8]) -> Delivery {
    let relayed = match OrgEndpoint::bind_from_keychain(device_key, context.bind_plan.clone()).await {
        Ok(relayed) => relayed,
        Err(error) => return Delivery::NotDelivered { reason: SendError::SenderEndpoint(format!("the relay bind failed: {error}")) },
    };
    let delivery = send_one(&relayed, recipient, &[], body).await;
    relayed.close().await;
    delivery
}

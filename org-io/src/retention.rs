//! The removed Persona's data retention and deletion (SDD-wyn7b6): a
//! removed Persona's store file and keychain item are kept while its
//! acknowledgements are delivered (SDD-3my66m), and deleted at the
//! deadline, or at once when none is kept, by the one rule `deletion_due`.
//! The deletion renames its store file `removed-<hex>.store`, deletes its
//! keychain item, removes the renamed file and drops its store (LLR-vj2nkb,
//! LLR-n649th); `OrgIo::open` finishes leftover `removed-` files and deletes
//! expired or empty removals (LLR-upqj5g); a failed step is reported in its
//! Organisation's report and retried hourly and at the next open, where the
//! retry of a leftover `removed-` file reports nothing new (LLR-3fxxpp).

use std::path::Path;
use std::sync::PoisonError;

use org_node::service::OrgService;
use org_node::{DevicePublicKey, OrgId};

use crate::acknowledge::{list_kept, wait_for, Context, ListingFailure};
use crate::transport::OrgEndpoint;
use crate::view::DeletionFailure;
use crate::{keychain, store_file};

/// The removal's deletion deadline after its verification time: 7 days.
pub(crate) const DEADLINE_SECONDS: u64 = 604_800;

/// One hour, the retry period of a failed deletion (LLR-3fxxpp).
pub(crate) const RETRY_SECONDS: u64 = 3_600;

/// Why a kept removal's data is due for deletion (LLR-vj2nkb, LLR-n649th).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeletionDue {
    /// The handle's clock reached the kept verification time plus 7 days.
    Expired,
    /// No acknowledgement is left to deliver.
    NoneKept,
}

/// The removed-Persona deletion rule (LLR-vj2nkb, LLR-n649th): due once
/// `now` reaches `verified_at` plus 7 days (`Expired`), or else once no
/// acknowledgement is kept, `left` being 0 (`NoneKept`); `None` while the
/// delivery goes on. Pure.
pub(crate) fn deletion_due(left: usize, verified_at: u64, now: u64) -> Option<DeletionDue> {
    if now >= verified_at.saturating_add(DEADLINE_SECONDS) {
        Some(DeletionDue::Expired)
    } else if left == 0 {
        Some(DeletionDue::NoneKept)
    } else {
        None
    }
}

/// The keychain item, then the removed file (LLR-vj2nkb's last two steps).
fn finish_removed(data_dir: &Path, device_key: &DevicePublicKey) -> Result<(), DeletionFailure> {
    keychain::delete_device_secret(device_key).map_err(|error| DeletionFailure::Keychain(crate::Cause::new(error)))?;
    store_file::remove_removed_file(data_dir, device_key).map_err(DeletionFailure::StoreFile)
}

/// Delete the Persona of `device_key` (LLR-vj2nkb, LLR-3fxxpp), under the
/// caller's lock: rename its file — on failure nothing else, the Persona
/// stays — then the item, then the file, then drop its store (dropped too
/// when a later step fails, which the retry finishes). A handle with no
/// data directory only drops the store.
fn delete_now(data_dir: Option<&Path>, service: &mut OrgService, device_key: &DevicePublicKey) -> Result<(), DeletionFailure> {
    let Some(data_dir) = data_dir else {
        service.drop_store(device_key);
        return Ok(());
    };
    store_file::mark_removed(data_dir, device_key).map_err(DeletionFailure::StoreFile)?;
    let finished = finish_removed(data_dir, device_key);
    service.drop_store(device_key);
    finished
}

/// Finish every deletion a `removed-<hex>.store` file in `data_dir`
/// records, unsealing nothing (LLR-upqj5g): `OrgIo::open`'s first step and
/// the hourly retry's (LLR-3fxxpp). A failure is left for the next try.
/// A leftover whose key also names a live `persona-` file (a restored
/// backup or a copy; org-io's own deletion renames the store first) is
/// left untouched, its item kept while its store is live (RC-hd5des,
/// LLR-upqj5g); those keys are returned to be reported.
pub(crate) fn finish_leftovers(data_dir: &Path) -> Vec<DevicePublicKey> {
    let mut kept = vec![];
    // A listing that fails finishes nothing and loses nothing: each
    // `removed-` file still names its key for the next try. At open, the
    // listing of the Persona files that follows refuses the open on a
    // lasting failure (`OpenError::ListDataDir`); in the hourly retry,
    // `may_hold_leftovers` keeps the retry going.
    for device_key in store_file::list_removed(data_dir).unwrap_or_default() {
        if store_file::store_path(data_dir, &device_key).exists() {
            kept.push(device_key);
            continue;
        }
        // A failure leaves the `removed-` file naming the key: the next
        // hourly retry or open tries again (LLR-3fxxpp). It is not reported:
        // its Persona is dropped from the service, so no
        // Organisation's report holds it; the file is the record.
        let _ = finish_removed(data_dir, &device_key);
    }
    kept
}

/// A deletion failure `OrgIo::open` found, by Organisation, with the
/// deadline of the removal it concerns when the store keeps one (LLR-rav7dr,
/// LLR-3fxxpp).
pub(crate) struct OpenFailure {
    pub(crate) org_id: OrgId,
    pub(crate) failure: DeletionFailure,
    pub(crate) deadline: Option<u64>,
}

/// The reports of the leftovers `finish_leftovers` kept beside a live
/// store, by the Organisation that store's Persona is bound to (LLR-upqj5g).
/// A live store that keeps a removal is reported with that removal's
/// deadline; one that keeps none has no deadline.
pub(crate) fn kept_leftover_failures(service: &OrgService, kept: &[DevicePublicKey]) -> Vec<OpenFailure> {
    service
        .persona_records()
        .into_iter()
        .filter(|persona| kept.contains(&persona.device_key))
        .filter_map(|persona| {
            let deadline = removal_deadline(service, &persona.device_key);
            persona.org_id.map(|org_id| OpenFailure { org_id, failure: DeletionFailure::LeftoverKept, deadline })
        })
        .collect()
}

/// The deadline of the removal `device_key`'s store keeps: its verification
/// time plus 7 days; `None` when it keeps none (LLR-rav7dr).
fn removal_deadline(service: &OrgService, device_key: &DevicePublicKey) -> Option<u64> {
    match service.kept_acknowledgements(device_key) {
        Ok(Some(kept)) => Some(kept.verified_at.0.saturating_add(DEADLINE_SECONDS)),
        _ => None,
    }
}

/// `OrgIo::open`'s step after unsealing (LLR-upqj5g): delete each Persona
/// whose removal `deletion_due` finds due by `now`; the failures, by
/// Organisation (LLR-3fxxpp). `store_keys` are the store files open listed
/// and unsealed into `service`. With no `now` (the clock read before the
/// Unix epoch), no deletion is judged: every kept removal is left and its
/// Organisation reports the condition.
pub(crate) fn delete_expired_at_open(
    data_dir: &Path,
    service: &mut OrgService,
    store_keys: &[DevicePublicKey],
    now: Option<u64>,
) -> Vec<OpenFailure> {
    let mut failures = vec![];
    for device_key in store_keys {
        // `open` unsealed every one of `store_keys` into `service` just now, so none is refused;
        // `None` is a store with no removal, which is not touched.
        let Ok(Some(kept)) = service.kept_acknowledgements(device_key) else { continue };
        // Kept with the failure: a failure after the rename drops the store,
        // and the report still gives the removal's deadline (LLR-rav7dr).
        let deadline = Some(kept.verified_at.0.saturating_add(DEADLINE_SECONDS));
        let Some(now) = now else {
            failures.push(OpenFailure { org_id: kept.org_id, failure: DeletionFailure::ClockBeforeEpoch, deadline });
            continue;
        };
        if deletion_due(kept.acknowledgements.len(), kept.verified_at.0, now).is_some() {
            if let Err(failure) = delete_now(Some(data_dir), service, device_key) {
                failures.push(OpenFailure { org_id: kept.org_id, failure, deadline });
            }
        }
    }
    failures
}

/// Whether `data_dir` may hold a leftover `removed-` file: it lists one, or
/// it cannot be listed, so a leftover cannot be ruled out and the hourly
/// retry goes on (LLR-3fxxpp).
fn may_hold_leftovers(data_dir: &Path) -> bool {
    !matches!(store_file::list_removed(data_dir), Ok(removed) if removed.is_empty())
}

/// Start the hourly retry of the leftover `removed-` files in the handle's
/// data directory, when there is one and none runs (LLR-3fxxpp).
pub(crate) fn start_leftover_retry(context: Context) {
    let Some(data_dir) = context.store.data_dir.clone() else { return };
    if !may_hold_leftovers(&data_dir) {
        return;
    }
    let deliveries = context.deliveries.clone();
    let mut leftovers = deliveries.leftovers.lock().unwrap_or_else(PoisonError::into_inner);
    if leftovers.as_ref().is_some_and(|task| !task.is_finished()) {
        return;
    }
    let first = retry_due(&context);
    *leftovers = Some(tokio::spawn(retry_leftovers(context, data_dir, first)));
}

/// An hour after the clock's reading: the next retry (LLR-3fxxpp). `None`
/// for a reading before the Unix epoch, never taken as time 0: the wait is
/// then one retry period by the runtime's timer (`wait_for`).
fn retry_due(context: &Context) -> Option<u64> {
    context.clock.now().map(|now| now.saturating_add(RETRY_SECONDS))
}

/// Retry, at `due` and then hourly, every leftover `removed-` file in
/// `data_dir` until none is left (LLR-3fxxpp).
async fn retry_leftovers(context: Context, data_dir: std::path::PathBuf, mut due: Option<u64>) {
    while may_hold_leftovers(&data_dir) {
        wait_for(&context, due).await;
        // A leftover kept beside a live store stays reported as `OrgIo::open`
        // found it (`kept_leftover_failures`); the retry adds no report.
        finish_leftovers(&data_dir);
        due = retry_due(&context);
    }
}

/// Delete the Persona from its task: close the endpoint, then retry hourly
/// until done, reporting each failure (LLR-vj2nkb, LLR-3fxxpp). Once done,
/// the removal's progress is forgotten: the report is `None`.
pub(crate) async fn delete_from_task(
    context: &Context,
    org_id: OrgId,
    device_key: &DevicePublicKey,
    endpoint: Option<OrgEndpoint>,
) {
    if let Some(endpoint) = endpoint {
        endpoint.close().await;
    }
    loop {
        let attempt = {
            let mut service = context.service.lock().await;
            // Only `PersonaNotHeld` means the Persona is no longer held; any
            // other failure of the read cannot tell, so it is this attempt's
            // failure, reported and retried (LLR-3fxxpp).
            let still_held = match list_kept(&service, device_key) {
                Ok(Some(_)) => Ok(true),
                Ok(None) | Err(ListingFailure::PersonaGone) => Ok(false),
                Err(ListingFailure::Failed(error)) => Err(DeletionFailure::Listing(error)),
            };
            match (still_held, context.store.data_dir.as_deref()) {
                (Err(failure), _) => Err(failure),
                (Ok(true), data_dir) => delete_now(data_dir, &mut service, device_key),
                // Only a deletion the rename began is finished: a store
                // dropped otherwise keeps its file and item. A listing that
                // fails cannot tell which, so it is a failed step, reported
                // and retried (LLR-3fxxpp).
                (Ok(false), Some(data_dir)) => match store_file::list_removed(data_dir) {
                    Ok(removed) if removed.contains(device_key) => finish_removed(data_dir, device_key),
                    Ok(_) => Ok(()),
                    Err(error) => Err(DeletionFailure::StoreFile(error)),
                },
                (Ok(false), None) => Ok(()),
            }
        };
        match attempt {
            Ok(()) => {
                context.deliveries.forget(org_id);
                context.deliveries.forget_removed(device_key);
                return;
            }
            Err(failure) => {
                // The retry is due an hour after the failure, read before
                // the report shows it.
                let due = retry_due(context);
                context.deliveries.update(org_id, |progress| {
                    progress.deletion_failure = Some(failure);
                    progress.next_round = None;
                });
                wait_for(context, due).await;
            }
        }
    }
}

/// Start the hourly retry, from `due`, of the deletion of `device_key`'s
/// Persona that failed at open (LLR-3fxxpp), unless a task of it runs.
pub(crate) fn retry_deletion(context: Context, device_key: DevicePublicKey, org_id: OrgId, due: Option<u64>) {
    let mut running = context.deliveries.running();
    if running.get(&device_key).is_some_and(|task| !task.is_finished()) {
        return;
    }
    let task_context = context.clone();
    let task = tokio::spawn(async move {
        wait_for(&task_context, due).await;
        delete_from_task(&task_context, org_id, &device_key, None).await;
    });
    running.insert(device_key, task);
}

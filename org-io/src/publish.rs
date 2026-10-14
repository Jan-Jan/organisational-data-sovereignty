//! The publish state (SDD-6mpmsw): the per-Organisation state org-io keeps
//! between calls — a chain write whose outcome is unknown, each
//! Organisation's verification status — and the sequences over it: settling
//! that write by reading the chain (LLR-t98pk9) and discarding its update
//! once it can no longer execute, and reconciling a held Organisation with
//! the chain (LLR-k5pzsz, LLR-kp95ty). Every other discard is org-node's own,
//! with each commit (LLR-fptp57, LLR-36zp69); the app discards nothing. The
//! state is in memory, in one `OrgIo` field (`PublishState`), so stage S4's
//! move of the handle behind its own lock is one field. org-io deletes
//! nothing here itself: every removal is one of org-node's (LLR-cc59ys).

use std::collections::HashMap;

use org_node::reconcile::Reconciled;
use org_node::revocation::Acknowledgement;
use org_node::service::{OrgService, ProvisionalTarget};
use org_node::{CommitOutcome, DevicePublicKey, Epoch, OrgId, OrgNodeError, OrgState};
use rand_core::{CryptoRng, RngCore};

use crate::chain_read::StateReader;
use crate::view::BuiltUpdate;

/// What settling a chain write whose outcome was unknown found
/// (LLR-t98pk9). Public data only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Settled {
    /// The Organisation has no write whose outcome is unknown; nothing read.
    NoUnknownWrite,
    /// The write executed: its update is committed at `epoch` and was sent
    /// to the Device its submission named; `sent` is the send's result,
    /// whose failure does not unwind the commit.
    Committed { org_id: OrgId, epoch: Epoch, sent: Result<(), String> },
    /// The chain moved past the record to another root: the reconcile's
    /// result for that state.
    Reconciled(Reconciliation),
    /// The chain is still at the record's epoch past the write's last valid
    /// block: the write can no longer execute, and its update is discarded.
    NotExecuted { org_id: OrgId },
    /// The outcome is still unknown: within the write's window, or the chain
    /// could not be read.
    StillUnknown { org_id: OrgId, reason: String },
}

/// The result of reconciling one held Organisation with a state read from
/// the chain (LLR-k5pzsz). Public data only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reconciliation {
    InStep { org_id: OrgId, epoch: Epoch },
    /// A held update was committed at `epoch`; `recipients` are the Devices
    /// it is for.
    Committed { org_id: OrgId, epoch: Epoch, recipients: Vec<DevicePublicKey> },
    Behind { org_id: OrgId, chain_epoch: Epoch },
    /// The committed update removed this node; its acknowledgements are kept
    /// in org-io's memory, never returned.
    Removed { org_id: OrgId },
    Refused { org_id: OrgId, reason: String },
    /// The chain could not be read, or its state was refused at parse: no
    /// org-node operation ran (LLR-kp95ty).
    Unverified { org_id: OrgId, reason: String },
}

/// Whether an Organisation's last reconcile read a state from the chain
/// (LLR-kp95ty). Public data only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verification {
    /// The last reconcile read a state, whatever org-node decided.
    Verified,
    /// The last reconcile could not read a state, for `reason`.
    Unverified { reason: String },
}

/// A chain write that timed out, as its submission named it (LLR-2yjuaj).
struct UnknownWrite {
    update: BuiltUpdate,
    recipient: DevicePublicKey,
    peer_addr: Option<iroh::EndpointAddr>,
    last_valid_block: u64,
    /// The epoch the write expected the chain at: the contract executes it
    /// only from that epoch (review round 1, finding-3).
    expected_epoch: Epoch,
}

/// Whether `chain_state` holds `update`'s Membership root and Organisation
/// public key: its write executed.
fn holds_update(chain_state: &OrgState, update: &BuiltUpdate) -> bool {
    chain_state.root_hash == update.resulting_root && chain_state.org_pub_key == update.org_pub_key
}

/// The publish state of every Organisation, held by `OrgIo` in one field.
/// Its fields are private: it hands out nothing.
#[derive(Default)]
pub struct PublishState {
    unknown_writes: HashMap<OrgId, UnknownWrite>,
    /// Each reconciled Organisation's verification status (LLR-kp95ty).
    verifications: HashMap<OrgId, Verification>,
    /// Acknowledgements org-node signed when a reconcile removed this node,
    /// kept for stage S3b-io part 2's delivery.
    acknowledgements: Vec<Acknowledgement>,
}

impl PublishState {
    /// Record the write of `update` as having an unknown outcome
    /// (LLR-2yjuaj).
    pub(crate) fn record_unknown(
        &mut self,
        update: BuiltUpdate,
        recipient: DevicePublicKey,
        peer_addr: Option<iroh::EndpointAddr>,
        last_valid_block: u64,
        expected_epoch: Epoch,
    ) {
        self.unknown_writes
            .insert(update.org_id, UnknownWrite { update, recipient, peer_addr, last_valid_block, expected_epoch });
    }

    /// Whether `update` is the update of its Organisation's unknown write
    /// (review round 1, finding-5).
    pub(crate) fn is_unknown_write(&self, update: &BuiltUpdate) -> bool {
        self.unknown_writes.get(&update.org_id).is_some_and(|unknown| unknown.update == *update)
    }

    /// `org_id`'s verification status; `None` until a reconcile of it ran
    /// (LLR-kp95ty).
    pub(crate) fn verification(&self, org_id: OrgId) -> Option<Verification> {
        self.verifications.get(&org_id).cloned()
    }
}

impl Settled {
    /// What a submission refused because of this settle reports
    /// (LLR-nqvbb6).
    pub(crate) fn submission_report(&self) -> String {
        let not_written = "this update was not written";
        match self {
            Settled::Committed { epoch, sent: Ok(()), .. } => format!(
                "an earlier chain write for this Organisation, whose outcome was unknown, executed: it was committed \
                 at epoch {} and sent to its own recipient; {not_written}",
                epoch.get()
            ),
            Settled::Committed { epoch, sent: Err(error), .. } => format!(
                "an earlier chain write for this Organisation, whose outcome was unknown, executed: it was committed \
                 at epoch {}, but its send to its own recipient failed ({error}); {not_written}",
                epoch.get()
            ),
            Settled::Reconciled(reconciliation) => format!(
                "an earlier chain write for this Organisation, whose outcome was unknown, was settled by reconciling \
                 with the chain ({reconciliation:?}); {not_written}"
            ),
            Settled::StillUnknown { reason, .. } => {
                format!("a chain write for this Organisation has an unknown outcome ({reason}); {not_written}")
            }
            Settled::NoUnknownWrite | Settled::NotExecuted { .. } => String::new(),
        }
    }
}

/// Settle `org_id`'s unknown write (LLR-t98pk9): read the finalised block,
/// then the state, once each, and decide; every result but `StillUnknown`
/// clears the write. A record already past the epoch the write expected
/// settles it as not executed with no read.
pub(crate) async fn settle_unknown<R: RngCore + CryptoRng + Send>(
    state: &mut PublishState,
    svc: &mut OrgService,
    reader: &dyn StateReader,
    rng: &mut R,
    org_id: OrgId,
) -> Settled {
    let Some((update, last_valid_block, expected_epoch)) =
        state.unknown_writes.get(&org_id).map(|unknown| (unknown.update, unknown.last_valid_block, unknown.expected_epoch))
    else {
        return Settled::NoUnknownWrite;
    };
    let still_unknown = |reason: String| Settled::StillUnknown { org_id, reason };
    // The record moved past the epoch the write expected: the chain is at
    // least at the record's epoch, so the contract's epoch check can never
    // pass the write. Had it executed, the record could have moved past it
    // only by committing it, and every commit of it org-io drives clears the
    // unknown write. Settled at once, with no read (review round 1,
    // finding-3).
    if held_epoch(svc, org_id).is_some_and(|epoch| epoch > expected_epoch) {
        return not_executed(state, svc, rng, org_id, update);
    }
    let finalised = match reader.finalised_block().await {
        Ok(number) => number,
        Err(error) => return still_unknown(format!("reading the finalised block failed: {error}")),
    };
    let chain_state = match reader.read_state(org_id).await {
        Ok(Some(chain_state)) => chain_state,
        Ok(None) => return still_unknown(OrgNodeError::OrgNotOnChain.to_string()),
        Err(error) => return still_unknown(format!("reading the chain failed: {error}")),
    };
    let record_epoch = held_epoch(svc, org_id);
    let executed = holds_update(&chain_state, &update);
    if !executed {
        match record_epoch {
            Some(epoch) if chain_state.epoch == epoch => {
                if finalised <= last_valid_block {
                    return still_unknown(format!(
                        "the chain is still at epoch {} and the write may yet execute until block {last_valid_block} \
                         (finalised: {finalised})",
                        epoch.get()
                    ));
                }
                return not_executed(state, svc, rng, org_id, update);
            }
            Some(epoch) if chain_state.epoch < epoch => {
                return still_unknown(format!(
                    "the chain read is at epoch {}, older than the record's {}",
                    chain_state.epoch.get(),
                    epoch.get()
                ));
            }
            _ => {}
        }
    }
    let Some(unknown) = state.unknown_writes.remove(&org_id) else {
        return Settled::NoUnknownWrite;
    };
    match svc.reconcile(rng, org_id, Some(chain_state)) {
        Ok(Reconciled::Committed { outcome, .. }) if executed => {
            let sent = svc.send_update(&outcome, unknown.recipient, unknown.peer_addr).await.map_err(|error| error.to_string());
            Settled::Committed { org_id, epoch: outcome.epoch, sent }
        }
        reconciled => Settled::Reconciled(reconciliation(state, svc, org_id, reconciled)),
    }
}

/// The unknown write of `org_id`, for `update`, can no longer execute:
/// discard its update, with the private key it holds — never before (owner
/// ruling D3) — and clear the write. An update org-node no longer holds (a
/// commit dropped it) is already gone. A discard that cannot save the store
/// keeps the write, to be tried again.
fn not_executed<R: RngCore + CryptoRng + Send>(
    state: &mut PublishState,
    svc: &mut OrgService,
    rng: &mut R,
    org_id: OrgId,
    update: BuiltUpdate,
) -> Settled {
    match svc.discard_provisional(rng, ProvisionalTarget::Org(org_id), update.resulting_root, update.org_pub_key) {
        Ok(()) | Err(OrgNodeError::NoProvisionalUpdate) => {}
        Err(error) => {
            return Settled::StillUnknown { org_id, reason: format!("discarding the update not executed failed: {error}") }
        }
    }
    state.unknown_writes.remove(&org_id);
    Settled::NotExecuted { org_id }
}

/// Reconcile `org_id` with the chain (LLR-k5pzsz): one read, handed to
/// org-node's `reconcile`. A failed read or a refused parse is `Unverified`
/// and runs no org-node operation; any state read records `Verified`
/// (LLR-kp95ty). Sends nothing, except a write whose outcome was unknown:
/// when the state read holds that write's update, the write is settled —
/// cleared, and its update, when this reconcile commits it, sent to the
/// Device its submission named (review round 1, finding-1).
pub(crate) async fn reconcile<R: RngCore + CryptoRng + Send>(
    state: &mut PublishState,
    svc: &mut OrgService,
    reader: &dyn StateReader,
    rng: &mut R,
    org_id: OrgId,
) -> Reconciliation {
    let chain_state = match reader.read_state(org_id).await {
        Ok(chain_state) => chain_state,
        Err(error) => {
            let reason = error.to_string();
            state.verifications.insert(org_id, Verification::Unverified { reason: reason.clone() });
            return Reconciliation::Unverified { org_id, reason };
        }
    };
    state.verifications.insert(org_id, Verification::Verified);
    // The unknown write the state shows executed: its outcome is known, so it
    // is cleared whatever org-node decides.
    let executed_write = match (&chain_state, state.unknown_writes.get(&org_id)) {
        (Some(chain_state), Some(unknown)) if holds_update(chain_state, &unknown.update) => state.unknown_writes.remove(&org_id),
        _ => None,
    };
    let reconciled = svc.reconcile(rng, org_id, chain_state);
    if let (Some(unknown), Ok(Reconciled::Committed { outcome, .. })) = (executed_write, &reconciled) {
        // The send's failure does not unwind the commit; its recipient is
        // among the result's `recipients`.
        let _ = svc.send_update(outcome, unknown.recipient, unknown.peer_addr).await;
    }
    reconciliation(state, svc, org_id, reconciled)
}

/// `reconcile` for every Organisation the store holds, in store order
/// (LLR-k5pzsz).
pub(crate) async fn reconcile_held<R: RngCore + CryptoRng + Send>(
    state: &mut PublishState,
    svc: &mut OrgService,
    reader: &dyn StateReader,
    rng: &mut R,
) -> Vec<Reconciliation> {
    let held: Vec<OrgId> = svc.list_orgs().iter().map(|record| record.org_id).collect();
    let mut results = Vec::with_capacity(held.len());
    for org_id in held {
        results.push(reconcile(state, svc, reader, rng, org_id).await);
    }
    results
}

/// org-node's reconcile result as org-io reports it (LLR-k5pzsz): public
/// data only; a removal's acknowledgements stay in `state`.
fn reconciliation(
    state: &mut PublishState,
    svc: &OrgService,
    org_id: OrgId,
    reconciled: Result<Reconciled, OrgNodeError>,
) -> Reconciliation {
    match reconciled {
        Ok(Reconciled::InStep) => match held_epoch(svc, org_id) {
            Some(epoch) => Reconciliation::InStep { org_id, epoch },
            None => Reconciliation::Refused { org_id, reason: OrgNodeError::OrgNotHeld { org_id }.to_string() },
        },
        Ok(Reconciled::Committed { outcome, .. }) => {
            Reconciliation::Committed { org_id, epoch: outcome.epoch, recipients: recipients(svc, &outcome) }
        }
        Ok(Reconciled::Behind { chain_epoch }) => Reconciliation::Behind { org_id, chain_epoch },
        Ok(Reconciled::Removed { acknowledgements, .. }) => {
            state.acknowledgements.extend(acknowledgements);
            Reconciliation::Removed { org_id }
        }
        Err(error) => Reconciliation::Refused { org_id, reason: error.to_string() },
    }
}

/// The epoch of `org_id`'s committed record; `None` when it is not held.
fn held_epoch(svc: &OrgService, org_id: OrgId) -> Option<Epoch> {
    svc.list_orgs().iter().find(|record| record.org_id == org_id).map(|record| record.epoch)
}

/// The Devices a committed update is for: every Device the committed record
/// lists other than this node's own, then each Device its notices name.
fn recipients(svc: &OrgService, outcome: &CommitOutcome) -> Vec<DevicePublicKey> {
    let own: Vec<DevicePublicKey> = svc
        .list_personas()
        .iter()
        .filter_map(|persona| svc.persona_public_keys(&persona.persona_id).ok().map(|(_, device)| device))
        .collect();
    let listed = svc
        .list_orgs()
        .iter()
        .filter(|record| record.org_id == outcome.org_id)
        .flat_map(|record| record.trie_members.iter().flat_map(|member| member.device_keys.iter().copied()))
        .filter(|device| !own.contains(device));
    listed.chain(outcome.revocations.iter().map(|notice| notice.device)).collect()
}

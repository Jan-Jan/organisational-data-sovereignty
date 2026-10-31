//! Verification (SDD-t4z8sz): read an Organisation's state at a finalised block, hand it to
//! org-node's verification with one reading of the handle's clock, write the store change it
//! returns, and report one outcome. The only caller of org-node's `verify` (LLR-3ame3g) and,
//! with `preflight.rs`, of `StateReader` (LLR-k3zmqb).

use org_node::invitation::InviteId;
use org_node::verify_org::{Inconsistency as NodeInconsistency, Verified as NodeVerified};
use org_node::{AheadLimit, DevicePublicKey, Epoch, Handle, MemberId, Name, OrgId, OrgNodeError, Surname, VerificationTime};

use crate::keychain::KeychainError;
use crate::write::{HeldTarget, UnknownWrite};
use crate::{Cause, OrgIo, StoreFailure};

/// Why a target was not verified (LLR-ce42ce, LLR-fzf2n3, LLR-4qyeut).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnverifiedReason {
    /// No verification of the target has run on this handle.
    NotYetVerified,
    /// No Persona here holds the target.
    NotHeld,
    /// The block or state read failed, or the parse refused the state.
    Read(ChainReadError),
    /// The chain holds no state for the Organisation.
    OrgNotOnChain,
    /// A founding whose write is not recorded.
    FoundingNotWritten,
    /// A founding with no proxy stored: nothing to read (LLR-4qyeut).
    NoProxyYet,
    /// The handle's clock read before the Unix epoch.
    ClockBeforeEpoch,
}

/// Why a verification's chain read failed (LLR-ce42ce): the block or state read failed, or the
/// parse refused the state. Its text is the reader's error, and its `source` is that error, typed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainReadError(Cause<OrgNodeError>);

impl std::fmt::Display for ChainReadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl std::error::Error for ChainReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.0.get())
    }
}

/// Why a state is inconsistent with the record (org-node's judgement, mirrored).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inconsistency {
    ChainBehindRecord { chain_epoch: Epoch, record_epoch: Epoch },
    OtherRoot { epoch: Epoch },
}

impl std::fmt::Display for UnverifiedReason {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotYetVerified => write!(formatter, "not yet verified on this handle"),
            Self::NotHeld => write!(formatter, "no Persona here holds the target"),
            Self::Read(error) => write!(formatter, "the chain read failed: {error}"),
            Self::OrgNotOnChain => write!(formatter, "the chain holds no state for the Organisation"),
            Self::FoundingNotWritten => write!(formatter, "the founding is not written to the chain"),
            Self::NoProxyYet => write!(formatter, "the founding has no proxy stored yet"),
            Self::ClockBeforeEpoch => formatter.write_str(crate::view::CLOCK_BEFORE_EPOCH),
        }
    }
}

impl std::fmt::Display for Inconsistency {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChainBehindRecord { chain_epoch, record_epoch } => write!(
                formatter,
                "the chain is at epoch {} below the record's epoch {}",
                chain_epoch.get(),
                record_epoch.get()
            ),
            Self::OtherRoot { epoch } => write!(formatter, "the chain holds another root at epoch {}", epoch.get()),
        }
    }
}

/// A provisional change an adoption left out (LLR-gaanm2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiscardedChangeSummary {
    Admission { invite_id: InviteId, name: Option<Name>, surname: Option<Surname>, handle: Handle },
    Revocation { member_id: MemberId, handle: Option<Handle> },
}

/// One verification's outcome (LLR-t2pve7): public data only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verified {
    InStep { epoch: Epoch },
    Adopted { epoch: Epoch, discarded: Vec<DiscardedChangeSummary>, own: bool },
    Removed { org_id: OrgId },
    Behind { chain_epoch: Epoch },
    Inconsistent { reason: Inconsistency },
    Unverified { reason: UnverifiedReason },
    /// org-node refused for another reason, `error`; nothing written.
    Refused { error: Cause<OrgNodeError> },
    /// The keychain refused the Device seed the removal signs with (LLR-8w2z64).
    Keychain { error: KeychainError },
    /// org-node's store change was not written, `failure`; the Persona was reloaded or dropped
    /// (LLR-zcw3xv).
    Store { failure: Cause<StoreFailure> },
    /// `outcome`, and what it settled of the target's unknown write (LLR-zey4mg); returned only
    /// when one was recorded.
    Settled { outcome: Box<Verified>, write: WriteOutcome },
}

/// What verification found of a chain write whose outcome was unknown (LLR-zey4mg). Public data
/// only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteOutcome {
    /// The verification adopted that provisional organisation.
    Adopted,
    /// Past its window, with the chain still at the epoch it was based on: it can no longer
    /// execute.
    NotExecuted,
    /// The chain is at or past the epoch it would have made, on another root.
    Deprecated,
    /// Nothing settles it yet; it stays recorded.
    StillUnknown,
}

/// What one verification read of the chain, when both reads answered.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ChainRead {
    pub(crate) finalised_block: u64,
    pub(crate) state: Option<org_node::OrgState>,
}

/// The settlement of `unknown` given the read and the outcome (LLR-zey4mg): adopted when the
/// chain holds exactly that write at the epoch it makes and the outcome adopted it (or found it
/// already in step); deprecated when the chain is at or past that epoch on another root; not
/// executed only for an update past its window with the chain at the base epoch; otherwise, and
/// on a failed read, still unknown. A founding has no window.
pub(crate) fn settle(unknown: &UnknownWrite, read: Option<&ChainRead>, outcome: &Verified) -> WriteOutcome {
    let Some(ChainRead { finalised_block, state: Some(state) }) = read else { return WriteOutcome::StillUnknown };
    let written_epoch = unknown.expected_epoch.get().saturating_add(1);
    let its_write =
        state.epoch.get() == written_epoch && state.root_hash == unknown.root && state.org_pub_key == unknown.key;
    if its_write && matches!(outcome, Verified::Adopted { .. } | Verified::InStep { .. }) {
        return WriteOutcome::Adopted;
    }
    if state.epoch.get() >= written_epoch && !its_write {
        return WriteOutcome::Deprecated;
    }
    match unknown.last_valid_block {
        Some(last_valid) if *finalised_block > last_valid && state.epoch == unknown.expected_epoch => {
            WriteOutcome::NotExecuted
        }
        _ => WriteOutcome::StillUnknown,
    }
}

impl OrgIo {
    /// Verify `target` (LLR-t2pve7) and keep the outcome for `verification`, then settle its
    /// unknown write, if one is recorded, against the same read (LLR-zey4mg); the settlement
    /// changes no store and is reported with the outcome.
    pub async fn verify_organisation(&self, target: HeldTarget) -> Verified {
        let (outcome, read) = self.verify_once(target).await;
        let mut publish = self.publish.lock().await;
        publish.verifications.insert(target, outcome.clone());
        let Some(unknown) = publish.unknown_write(&target) else { return outcome };
        let write = settle(&unknown, read.as_ref(), &outcome);
        if write != WriteOutcome::StillUnknown {
            publish.clear_unknown_write(&target);
        }
        Verified::Settled { outcome: Box::new(outcome), write }
    }

    /// Every Organisation, provisional founding and provisional Member this node holds, one at a
    /// time in store-file order (Device-key byte order), whatever each one's outcome (LLR-kq7z58).
    /// A removed store is none of them. Each outcome is returned beside the target it verified.
    pub async fn verify_all(&self) -> Vec<(HeldTarget, Verified)> {
        let mut targets: Vec<(DevicePublicKey, HeldTarget)> = {
            let service = self.service.lock().await;
            service
                .persona_records()
                .into_iter()
                .filter_map(|persona| {
                    // `persona_view` refuses only a key no store holds and a removed store, and
                    // `persona_records` lists neither: no target is left out by a refusal.
                    let view = service.persona_view(&persona.device_key).ok()?;
                    let target = match (&view.record, &view.founding, view.member_of) {
                        (Some(record), _, _) => HeldTarget::Organisation(record.org_id),
                        (None, Some(_), _) => HeldTarget::Founding(persona.device_key),
                        (None, None, Some(member_of)) => HeldTarget::Organisation(member_of),
                        (None, None, None) => return None,
                    };
                    Some((persona.device_key, target))
                })
                .collect()
        };
        targets.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
        let mut verified = Vec::with_capacity(targets.len());
        for (_, target) in targets {
            verified.push((target, self.verify_organisation(target).await));
        }
        verified
    }

    /// The last outcome of `target`; `Unverified` before any (LLR-ce42ce).
    pub async fn verification(&self, target: HeldTarget) -> Verified {
        let kept = self.publish.lock().await.verifications.get(&target).cloned();
        kept.unwrap_or(Verified::Unverified { reason: UnverifiedReason::NotYetVerified })
    }

    /// The store and Organisation `target` is read for: a held Organisation's record, else a
    /// provisional Member bound to it (LLR-fp2cat); a founding read at its stored proxy (LLR-4qyeut).
    async fn read_target(&self, target: HeldTarget) -> Result<(DevicePublicKey, OrgId), UnverifiedReason> {
        let service = self.service.lock().await;
        match target {
            HeldTarget::Organisation(org_id) => crate::record_holder_of(&service, org_id)
                .or_else(|| {
                    service.persona_records().into_iter().find_map(|persona| {
                        // As in `verify_all`: a listed Persona's view is never refused.
                        let view = service.persona_view(&persona.device_key).ok()?;
                        view.member_of.filter(|member_of| *member_of == org_id).map(|_| persona.device_key)
                    })
                })
                .map(|device_key| (device_key, org_id))
                .ok_or(UnverifiedReason::NotHeld),
            HeldTarget::Founding(founder) => {
                // Read through org-node's key-free Persona view (`persona_view`, which clones no private key).
                // It refuses only a key no store holds and a removed store: neither holds a founding,
                // which is what `NotHeld` says.
                let view = service.persona_view(&founder).map_err(|_| UnverifiedReason::NotHeld)?;
                let founding = view.founding.as_ref().ok_or(UnverifiedReason::NotHeld)?;
                match (founding.proxy, view.org_id) {
                    (Some(_), Some(org_id)) => Ok((founder, org_id)),
                    _ => Err(UnverifiedReason::NoProxyYet),
                }
            }
        }
    }

    /// The block read, then the state read, then one clock reading, then org-node's verification
    /// with the handle's ahead limit; the store change it returns is written, and nothing else
    /// (LLR-t2pve7, LLR-ce42ce, LLR-fzf2n3, LLR-22se2q). Beside the outcome, the read once both
    /// reads answered and the clock read after the epoch.
    async fn verify_once(&self, target: HeldTarget) -> (Verified, Option<ChainRead>) {
        let unverified = |reason| (Verified::Unverified { reason }, None);
        let (device_key, org_id) = match self.read_target(target).await {
            Ok(read) => read,
            Err(reason) => return unverified(reason),
        };
        // A handle with no chain holds no ahead limit: it reads nothing, refused as its reader
        // refuses every read (LLR-22se2q).
        let Some(ahead_limit) = self.ahead_limit else {
            let not_configured = OrgNodeError::Chain(crate::write::NOT_CONFIGURED.into());
            return unverified(UnverifiedReason::Read(ChainReadError(Cause::new(not_configured))));
        };
        let finalised_block = match self.reader.finalised_block().await {
            Ok(block) => block,
            Err(error) => return unverified(UnverifiedReason::Read(ChainReadError(Cause::new(error)))),
        };
        let state = match self.reader.read_state(org_id).await {
            Ok(state) => state,
            Err(error) => return unverified(UnverifiedReason::Read(ChainReadError(Cause::new(error)))),
        };
        let Some(now) = self.local.clock.now() else { return unverified(UnverifiedReason::ClockBeforeEpoch) };
        let read = Some(ChainRead { finalised_block, state });
        (self.judge(device_key, org_id, state, now, ahead_limit).await, read)
    }

    /// org-node's verification of `state` at `now` with `ahead_limit`, its store change written,
    /// its outcome mapped.
    async fn judge(
        &self,
        device_key: DevicePublicKey,
        org_id: OrgId,
        state: Option<org_node::OrgState>,
        now: u64,
        ahead_limit: AheadLimit,
    ) -> Verified {
        let unverified = |reason| Verified::Unverified { reason };
        let mut service = self.service.lock().await;
        let mut refused: Option<KeychainError> = None;
        let keychain = &self.local.keychain;
        let judged = {
            let mut rng = self.local.rng.lock();
            service.verify(&mut *rng, &device_key, state, VerificationTime(now), ahead_limit, |device_keys| {
                keychain.for_signing_noting_refusal(device_keys, &mut refused)
            })
        };
        let (outcome, change) = match judged {
            Ok(judged) => judged,
            Err(OrgNodeError::OrgNotOnChain) => return unverified(UnverifiedReason::OrgNotOnChain),
            Err(OrgNodeError::FoundingNotWritten) => return unverified(UnverifiedReason::FoundingNotWritten),
            Err(error @ OrgNodeError::DeviceSecretNotSupplied { .. }) => {
                return match refused {
                    Some(error) => Verified::Keychain { error },
                    None => Verified::Refused { error: Cause::new(error) },
                }
            }
            Err(error) => return Verified::Refused { error: Cause::new(error) },
        };
        if let Some(change) = change {
            if let Err(failure) = self.local.store(&mut service, change) {
                return Verified::Store { failure: Cause::new(failure) };
            }
        }
        match outcome {
            // org-node judges a store in step only against the record it holds
            // (`verify_org::judge`), so the record is found; were it not, no Persona here
            // holds the Organisation.
            NodeVerified::InStep => match crate::held_record(&service, org_id) {
                Some(record) => Verified::InStep { epoch: record.epoch },
                None => unverified(UnverifiedReason::NotHeld),
            },
            NodeVerified::Adopted { epoch, discarded, own } => {
                let discarded = discarded_summaries(&service, device_key, org_id, &discarded);
                Verified::Adopted { epoch, discarded, own }
            }
            NodeVerified::Removed { org_id, .. } => {
                // The removed store is written; with the node's lock released, the Persona's
                // endpoint is closed and its acknowledgements' delivery started (LLR-zzsqj4,
                // LLR-cqr26z).
                drop(service);
                self.local.endpoints.close(&device_key).await;
                crate::acknowledge::start(self.delivery_context(), device_key).await;
                Verified::Removed { org_id }
            }
            NodeVerified::Behind { chain_epoch } => Verified::Behind { chain_epoch },
            NodeVerified::Inconsistent { reason } => Verified::Inconsistent {
                reason: match reason {
                    NodeInconsistency::ChainBehindRecord { chain_epoch, record_epoch } => {
                        Inconsistency::ChainBehindRecord { chain_epoch, record_epoch }
                    }
                    NodeInconsistency::OtherRoot { epoch } => Inconsistency::OtherRoot { epoch },
                },
            },
        }
    }

}

/// org-node's left-out changes, unchanged and in its order, each with the names the still-kept
/// admission request gives and the Handle the adopted record gives a revoked Member (LLR-gaanm2).
fn discarded_summaries(
    service: &org_node::service::OrgService,
    device_key: DevicePublicKey,
    org_id: OrgId,
    discarded: &[org_node::verify_org::DiscardedChange],
) -> Vec<DiscardedChangeSummary> {
    // `admission_requests` refuses only a key no store holds and a removed store; the store
    // just adopted, and was written, so neither holds and no request is left out.
    let requests = service.admission_requests(&device_key).unwrap_or_default();
    // The same store holds the record it just adopted, so the record is found; a revoked Member
    // it does not list has no Handle to give (`handle: None`).
    let members = crate::held_record(service, org_id).map(|record| record.members).unwrap_or_default();
    discarded
        .iter()
        .map(|change| match change {
            org_node::verify_org::DiscardedChange::Admit { invite_id, handle } => {
                let request =
                    requests.iter().find(|(request, _)| request.invite_id == *invite_id).map(|(request, _)| *request);
                DiscardedChangeSummary::Admission {
                    invite_id: *invite_id,
                    name: request.map(|request| request.name.clone()),
                    surname: request.map(|request| request.surname.clone()),
                    handle: handle.clone(),
                }
            }
            org_node::verify_org::DiscardedChange::Revoke { member_id } => DiscardedChangeSummary::Revocation {
                member_id: *member_id,
                handle: members.iter().find(|member| member.id == *member_id).map(|member| member.handle.clone()),
            },
        })
        .collect()
}

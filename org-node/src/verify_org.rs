//! Verification of one Organisation against the chain (SDD-kegag2): the
//! judgement of a chain state against the store's record and its
//! provisional organisations. Pure functions over values (LLR-g2jeqd): no
//! chain, transport, endpoint, file-system or clock function is called.
use org_members::{Handle, MemberId, RootHash};
use serde::{Deserialize, Serialize};

use crate::chain::OrgState;
use crate::error::OrgNodeError;
use crate::ids::OrgId;
use crate::invitation::InviteId;
use crate::store::{
    KeptAdmissionRequest, MemberSnapshot, OrgRecord, Origin, PersonaStatus, ProvisionalChange, ProvisionalFounding,
    ProvisionalOrganisation, RevokedDevice, StoreData,
};
use crate::service::trie_from_snapshots;
use crate::types::{Epoch, SequenceNumber};

/// The caller's time of a verification, in seconds since the Unix epoch
/// (LLR-rr558z); org-node reads no clock and keeps it only on a removal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VerificationTime(pub u64);

/// How long, in seconds of `VerificationTime`, a provisional organisation
/// received one epoch ahead of the chain is kept unadopted (LLR-4tsgfh).
/// The caller's: org-node reads no clock and sets no default.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AheadLimit(pub u64);

/// Why a chain state contradicts the record (LLR-rr558z).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inconsistency {
    /// The chain is at an epoch below the record's.
    ChainBehindRecord { chain_epoch: Epoch, record_epoch: Epoch },
    /// The chain is at the record's epoch with another root.
    OtherRoot { epoch: Epoch },
}

/// A provisional change the adopted record did not include, reported by
/// the adoption that discarded it (LLR-d9x6xf).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiscardedChange {
    Admit { invite_id: InviteId, handle: Handle },
    Revoke { member_id: MemberId },
}

/// What `OrgService::verify` decided: exactly one outcome (LLR-rr558z).
#[derive(Debug, PartialEq, Eq)]
pub enum Verified {
    InStep,
    Behind { chain_epoch: Epoch },
    Inconsistent { reason: Inconsistency },
    /// A held provisional organisation was adopted (LLR-34c5y3).
    Adopted { epoch: Epoch, discarded: Vec<DiscardedChange>, own: bool },
    /// This Persona's own removal (LLR-jhy6t6): the store was sealed in its removed form keeping
    /// `acknowledgements_kept` acknowledgements.
    Removed { org_id: OrgId, acknowledgements_kept: usize },
}

/// What `judge` found: an outcome that adopts nothing, or what to adopt.
pub(crate) enum Judgement {
    Done(Verified),
    Adopt(usize),
    /// A provisional Member's store adopts the provisional organisation
    /// at this index (LLR-q4puca).
    AdoptFirst(usize),
    /// A founding store adopts its founding (LLR-gcm7bq).
    AdoptFounding,
}

/// The outcome of `chain` against `store` (LLR-rr558z, LLR-8528ud): with
/// a record, a lower epoch or the same epoch with another root is
/// inconsistent and the same epoch and root in step; a greater epoch is
/// the one held provisional organisation whose epoch, root and key are
/// the chain's, else behind. With no record, a founding store adopts a
/// founding the chain holds and a provisional Member's store a match,
/// else each is behind.
pub(crate) fn judge(store: &StoreData, chain: &OrgState) -> Judgement {
    let Some(record) = &store.record else {
        if let Some(founding) = &store.founding {
            return if founding_matches(founding, chain) {
                Judgement::AdoptFounding
            } else {
                Judgement::Done(Verified::Behind { chain_epoch: chain.epoch })
            };
        }
        return match matching(store, chain) {
            Some(index) if store.member.is_some() => Judgement::AdoptFirst(index),
            _ => Judgement::Done(Verified::Behind { chain_epoch: chain.epoch }),
        };
    };
    if chain.epoch < record.epoch {
        return Judgement::Done(Verified::Inconsistent {
            reason: Inconsistency::ChainBehindRecord { chain_epoch: chain.epoch, record_epoch: record.epoch },
        });
    }
    if chain.epoch == record.epoch {
        return Judgement::Done(if chain.root_hash == record.root_hash {
            Verified::InStep
        } else {
            Verified::Inconsistent { reason: Inconsistency::OtherRoot { epoch: chain.epoch } }
        });
    }
    match matching(store, chain) {
        Some(index) => Judgement::Adopt(index),
        None => Judgement::Done(Verified::Behind { chain_epoch: chain.epoch }),
    }
}

/// Whether a kept notice, tried in kept order, is one `revocation::accept` takes against `chain`
/// (LLR-jhy6t6). A refused notice stays kept and is skipped (LLR-f9mcsq).
pub(crate) fn accepted_notice(store: &StoreData, chain: &OrgState) -> bool {
    store.kept_revocation_notices.iter().any(|notice| crate::revocation::accept(store, notice, Some(*chain)).is_ok())
}

/// The one held provisional organisation whose epoch, root and
/// Organisation public key are the chain's (LLR-8528ud): own, received
/// or answer alike; identity (root, key) makes at most one.
pub(crate) fn matching(store: &StoreData, chain: &OrgState) -> Option<usize> {
    store.provisional_organisations.iter().position(|held| matches_chain(held, chain))
}

/// Whether `held` has the chain's epoch, root and Organisation public key
/// (LLR-8528ud).
fn matches_chain(held: &ProvisionalOrganisation, chain: &OrgState) -> bool {
    held.epoch == chain.epoch && held.root == chain.root_hash && held.org_pub_key == chain.org_pub_key
}

/// Discard from `store` each received or answer provisional organisation
/// that can never be adopted against `chain` at epoch C, and stamp each
/// one ahead (LLR-4tsgfh): below C, above C + 1, or at C and not
/// matching, discarded; at C + 1, stamped with `verified_at` by the first
/// verification that finds it there, then discarded by one more than
/// `ahead_limit` seconds after the stamp. `Own` ones and the provisional
/// changes are untouched, and nothing at all is when C is below the
/// store's own record: a record is never ahead of the chain, so that read
/// is stale or faulty and cannot be true. Returns the
/// store after it, or `None` when nothing changed.
pub(crate) fn discard_unadoptable(
    store: &StoreData,
    chain: &OrgState,
    verified_at: VerificationTime,
    ahead_limit: AheadLimit,
) -> Option<StoreData> {
    if store.record.as_ref().is_some_and(|record| chain.epoch < record.epoch) {
        return None;
    }
    let mut next_store = store.clone();
    let before = next_store.provisional_organisations.len();
    let next = chain.epoch.get().checked_add(1);
    let mut stamped = false;
    next_store.provisional_organisations.retain_mut(|held| {
        if matches!(held.origin, Origin::Own) {
            return true;
        }
        if held.epoch == chain.epoch {
            return matches_chain(held, chain);
        }
        if Some(held.epoch.get()) != next {
            return false;
        }
        match held.first_ahead_at {
            None => {
                held.first_ahead_at = Some(verified_at);
                stamped = true;
                true
            }
            Some(stamp) => verified_at.0.saturating_sub(stamp.0) <= ahead_limit.0,
        }
    });
    (stamped || next_store.provisional_organisations.len() != before).then_some(next_store)
}

/// The founding matches a chain state at epoch 1 holding its root and
/// key, its members rebuilding that root (LLR-8528ud, LLR-gcm7bq).
fn founding_matches(founding: &ProvisionalFounding, chain: &OrgState) -> bool {
    chain.epoch == Epoch::new(1)
        && founding.root == chain.root_hash
        && founding.org_pub_key == chain.org_pub_key
        && rebuilds(&founding.members, founding.root)
}

/// Whether `members` rebuild `root` (LLR-8528ud). Members that build no
/// trie give no root, so they do not give `root` either, and the trie error
/// is read as "does not rebuild": LLR-8528ud makes such a founding no match,
/// and `judge` reports a founding store with no match as `Behind`
/// (LLR-rr558z). Members that fail to build are not reported apart from
/// members that build another root.
fn rebuilds(members: &[MemberSnapshot], root: RootHash) -> bool {
    trie_from_snapshots(members)
        .and_then(|trie| trie.root_hash().map_err(OrgNodeError::Trie))
        .is_ok_and(|rebuilt| rebuilt == root)
}

/// Whether `record` lists a member with `request`'s Member-as-a-group key
/// and DevicePublicKey (LLR-d9x6xf, LLR-rzqs2u).
pub(crate) fn lists(record: &OrgRecord, request: &KeptAdmissionRequest) -> bool {
    record
        .trie_members
        .iter()
        .any(|member| member.member_key == request.member_key && member.device_keys.contains(&request.device_key))
}

/// The store after adopting the provisional organisation at `index` over
/// `previous`, and the outcome (LLR-34c5y3): the match's members, root,
/// key pair, epoch and Sequence number, Change set and base snapshot
/// replace the record's; the proxy account is kept; every provisional
/// change is removed, those left out reported (LLR-d9x6xf); provisional
/// organisations up to the epoch and every kept notice are dropped
/// (LLR-x8drb8); admitted requests are settled (LLR-rzqs2u) and dropped
/// Devices held as revoked (LLR-bgj2ff); the marker is cleared.
pub(crate) fn adopt(store: &StoreData, previous: &OrgRecord, index: usize) -> (StoreData, Verified) {
    let mut next = store.clone();
    let matched = next.provisional_organisations.remove(index);
    let own = matches!(matched.origin, Origin::Own);
    let epoch = matched.epoch;
    let record = OrgRecord {
        org_id: previous.org_id,
        root_hash: matched.root,
        org_pub_key: matched.org_pub_key,
        epoch,
        last_seq: SequenceNumber::new(epoch.get()),
        trie_members: matched.members,
        proxy_account: previous.proxy_account,
        org_private_key: matched.org_private_key,
        kept_change_set: matched.change_set,
        kept_base_snapshot: matched.base_snapshot,
    };
    let discarded = discarded_changes(store, &record);
    next.provisional_changes.clear();
    prune_after_adoption(&mut next, epoch);
    settle_admitted(&mut next, &record);
    hold_dropped_devices(&mut next, previous, &record);
    next.record = Some(record);
    (next, Verified::Adopted { epoch, discarded, own })
}

/// A provisional Member's first adoption (LLR-q4puca): only a match
/// listing one member with the provisional Member's Member-as-a-group key
/// and DevicePublicKey (else `AdmissionNotOurs`, nothing changed); the
/// record is created with the chain's Organisation public key and the
/// match's private key, members, root and kept values; the Persona takes
/// that member's MemberId and Handle; the Invite issuer's addresses
/// become its address-book entry (LLR-h9d65p, PR-8sgsb5); the provisional
/// Member and the expectation go; earlier provisional organisations and kept notices
/// go as on every adoption (LLR-x8drb8).
pub(crate) fn adopt_first(store: &StoreData, index: usize, chain: &OrgState) -> Result<(StoreData, Verified), OrgNodeError> {
    let member = store.member.as_ref().ok_or(OrgNodeError::NoProvisionalMember)?;
    let org_id = member.org_id;
    let ours = store.provisional_organisations[index]
        .members
        .iter()
        .find(|listed| listed.member_key == member.member_key && listed.device_keys.contains(&member.device_key))
        .ok_or(OrgNodeError::AdmissionNotOurs { org_id })?;
    let (member_id, handle) = (ours.id, ours.handle.clone());
    let mut next = store.clone();
    let matched = next.provisional_organisations.remove(index);
    let own = matches!(matched.origin, Origin::Own);
    let epoch = matched.epoch;
    next.record = Some(OrgRecord {
        org_id,
        root_hash: matched.root,
        org_pub_key: chain.org_pub_key,
        epoch,
        last_seq: SequenceNumber::new(epoch.get()),
        trie_members: matched.members,
        proxy_account: None,
        org_private_key: matched.org_private_key,
        kept_change_set: matched.change_set,
        kept_base_snapshot: matched.base_snapshot,
    });
    next.persona.member_id = Some(member_id);
    next.persona.status = PersonaStatus::Active; // LLR-exaa9u: Active at the first adoption
    next.persona.handle = Some(handle);
    next.set_address(member.issuer, member.issuer_addresses.clone());
    next.member = None;
    next.expected_admissions.retain(|expected| expected.org_id != org_id);
    prune_after_adoption(&mut next, epoch);
    Ok((next, Verified::Adopted { epoch, discarded: Vec::new(), own }))
}

/// A founding store's adoption (LLR-gcm7bq): the record is created at
/// epoch and Sequence number 1 from the founding's members, root and key
/// pair, with its proxy as `proxy_account` and no kept Change set or base
/// snapshot; the Persona takes the founder's MemberId; the founding goes.
pub(crate) fn adopt_founding(store: &StoreData) -> Result<(StoreData, Verified), OrgNodeError> {
    let mut next = store.clone();
    let founding = next.founding.take().ok_or(OrgNodeError::NoProvisionalFounding)?;
    let org_id = next.persona.org_id.ok_or(OrgNodeError::FoundingNotWritten)?;
    let founder = founding
        .members
        .iter()
        .find(|member| member.device_keys.contains(&next.persona.device_key))
        .map(|member| member.id)
        .ok_or(OrgNodeError::AdmissionNotOurs { org_id })?;
    next.record = Some(OrgRecord {
        org_id,
        root_hash: founding.root,
        org_pub_key: founding.org_pub_key,
        epoch: Epoch::new(1),
        last_seq: SequenceNumber::new(1),
        trie_members: founding.members,
        proxy_account: founding.proxy,
        org_private_key: founding.org_private_key,
        kept_change_set: None,
        kept_base_snapshot: None,
    });
    next.persona.member_id = Some(founder);
    next.persona.status = PersonaStatus::Active; // LLR-exaa9u: Active at the first adoption
    // Not `prune_after_adoption`: a founding store holds no provisional
    // organisation and no kept notice to drop. With no record, the receive
    // path keeps either only for an Organisation an expected admission
    // names (LLR-e6w3nr), and a founding store expects none; a proposal
    // needs a record (LLR-n6ssue). Only the marker is cleared.
    next.behind = None;
    Ok((next, Verified::Adopted { epoch: Epoch::new(1), discarded: Vec::new(), own: true }))
}

/// Each kept request whose keys `adopted` lists is removed with its
/// outstanding pair, and its addresses become its Device's address-book
/// entry, replacing any (LLR-rzqs2u, LLR-h9d65p).
fn settle_admitted(next: &mut StoreData, adopted: &OrgRecord) {
    let (settled, kept): (Vec<_>, Vec<_>) =
        std::mem::take(&mut next.kept_admission_requests).into_iter().partition(|request| lists(adopted, request));
    next.kept_admission_requests = kept;
    for request in settled {
        next.outstanding_invites
            .retain(|invite| !(invite.org_id == request.org_id && invite.invite_id == request.invite_id));
        next.set_address(request.device_key, request.addresses);
    }
}

/// What every adoption of a provisional organisation at `epoch` drops: the
/// provisional organisations up to the epoch and every kept notice
/// (LLR-x8drb8); and the behind marker, which it clears (LLR-zk7fx2).
fn prune_after_adoption(next: &mut StoreData, epoch: Epoch) {
    next.provisional_organisations.retain(|held| held.epoch > epoch);
    next.kept_revocation_notices.clear();
    next.behind = None;
}

/// Each Device `previous` lists and `adopted` does not, in snapshot and
/// slot order, held as revoked once (LLR-bgj2ff).
fn hold_dropped_devices(next: &mut StoreData, previous: &OrgRecord, adopted: &OrgRecord) {
    for member in &previous.trie_members {
        for device in &member.device_keys {
            let still_listed = adopted.trie_members.iter().any(|listed| listed.device_keys.contains(device));
            let held = next.revoked_devices.iter().any(|revoked| revoked.device_key == *device);
            if !still_listed && !held {
                next.revoked_devices.push(RevokedDevice { member_id: member.id, device_key: *device });
            }
        }
    }
}

/// The provisional changes `adopted` does not include, in recorded order
/// (LLR-d9x6xf): an `Admit` whose kept request's keys it does not list,
/// a `Revoke` whose MemberId it still holds.
fn discarded_changes(store: &StoreData, adopted: &OrgRecord) -> Vec<DiscardedChange> {
    let mut discarded = Vec::new();
    for change in &store.provisional_changes {
        match change {
            ProvisionalChange::Admit { invite_id, handle } => {
                let included = store.kept_request(*invite_id).is_some_and(|request| lists(adopted, request));
                if !included {
                    discarded.push(DiscardedChange::Admit { invite_id: *invite_id, handle: handle.clone() });
                }
            }
            ProvisionalChange::Revoke { member_id } => {
                let still_held = adopted.trie_members.iter().any(|member| member.id == *member_id);
                if still_held {
                    discarded.push(DiscardedChange::Revoke { member_id: *member_id });
                }
            }
        }
    }
    discarded
}

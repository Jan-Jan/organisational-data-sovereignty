//! One verification result as it crosses IPC and as the events item reads
//! it (SDD-6pjvhc's DTOs): the entry `verify_organisation` returns, the
//! `verification-result` payload, and the per-Device report of a send or a
//! request. Pure mappings from org-io's values; no I/O.

use serde::Serialize;

use org_io::node::{DevicePublicKey, OrgId};
use org_io::verify::DiscardedChangeSummary;
use org_io::{Delivery, SendReport, Verified, WriteOutcome};

use crate::follow_up::VerificationFollowUp;

/// One provisional change an adoption left out (LLR-n64wvr), tagged by
/// `kind`, ids as 64 hex characters. Every field is always present: a
/// name, surname or Handle org-io does not report is `null`, never absent.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DiscardedDto {
    Admission { invite_id: String, name: Option<String>, surname: Option<String>, handle: String },
    Revocation { member_id: String, handle: Option<String> },
}

impl From<&DiscardedChangeSummary> for DiscardedDto {
    fn from(change: &DiscardedChangeSummary) -> Self {
        match change {
            DiscardedChangeSummary::Admission { invite_id, name, surname, handle } => Self::Admission {
                invite_id: hex::encode(invite_id.as_bytes()),
                name: name.as_ref().map(ToString::to_string),
                surname: surname.as_ref().map(ToString::to_string),
                handle: handle.to_string(),
            },
            DiscardedChangeSummary::Revocation { member_id, handle } => Self::Revocation {
                member_id: hex::encode(member_id.as_bytes()),
                handle: handle.as_ref().map(ToString::to_string),
            },
        }
    }
}

/// What an entry names its target by (LLR-jf63e4): a
/// held Organisation by its identifier; a provisional founding by its
/// founder's Device key, always, with the identifier its proxy gave once
/// the proxy is recorded (by org-node's founding-proxy record) and none before.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryOrg {
    Id(OrgId),
    Founding { founder: DevicePublicKey, org_id: Option<OrgId> },
}

/// One verification's result, tagged by `result` (LLR-jf63e4); flattened
/// into `VerificationEntryDto`.
#[derive(Debug, Serialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum EntryResultDto {
    InStep { epoch: u64 },
    Adopted {
        epoch: u64,
        own: bool,
        discarded: Vec<DiscardedDto>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sends: Option<Vec<DeviceReportDto>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sends_refused: Option<String>,
    },
    Removed,
    Behind {
        chain_epoch: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        requests: Option<Vec<DeviceReportDto>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        requests_refused: Option<String>,
    },
    Inconsistent { reason: String },
    Unverified { reason: String },
}

/// What a verification settled of the target's unknown write (org-io's
/// `WriteOutcome`, LLR-jf63e4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WriteLabelDto {
    Adopted,
    NotExecuted,
    Deprecated,
    StillUnknown,
}

impl From<WriteOutcome> for WriteLabelDto {
    fn from(write: WriteOutcome) -> Self {
        match write {
            WriteOutcome::Adopted => Self::Adopted,
            WriteOutcome::NotExecuted => Self::NotExecuted,
            WriteOutcome::Deprecated => Self::Deprecated,
            WriteOutcome::StillUnknown => Self::StillUnknown,
        }
    }
}

/// One verification result as it crosses IPC (LLR-jf63e4): `org_id` as 40
/// hex characters, or `null` (never an empty string) for a provisional
/// founding with no proxy recorded; every founding entry carries `founder`
/// (64 hex), before its proxy is recorded and after;
/// the result tagged by `result`, each reason org-io's text; `write` only
/// for a verification that settled an unknown write.
#[derive(Debug, Serialize)]
pub struct VerificationEntryDto {
    pub org_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub founder: Option<String>,
    #[serde(flatten)]
    pub result: EntryResultDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write: Option<WriteLabelDto>,
}

/// LLR-jf63e4: org-io's `verified` for the Organisation `named` names, as
/// its entry. `Settled` is its inner outcome's entry with the `write`
/// label; `Refused` (an org-node refusal no other result names), `Store`
/// and `Keychain` (the store write or the keychain read a verification
/// needed failed) are `unverified`, with org-io's text as the reason.
pub fn entry_for(named: &EntryOrg, verified: &Verified) -> VerificationEntryDto {
    let (org_id, founder) = match named {
        EntryOrg::Id(org_id) => (Some(hex::encode(org_id.as_bytes())), None),
        EntryOrg::Founding { founder, org_id } => {
            (org_id.map(|org_id| hex::encode(org_id.as_bytes())), Some(hex::encode(founder.as_bytes())))
        }
    };
    let entry = |result| VerificationEntryDto { org_id: org_id.clone(), founder: founder.clone(), result, write: None };
    match verified {
        Verified::InStep { epoch } => entry(EntryResultDto::InStep { epoch: epoch.get() }),
        Verified::Adopted { epoch, discarded, own } => entry(EntryResultDto::Adopted {
            epoch: epoch.get(),
            own: *own,
            discarded: discarded.iter().map(DiscardedDto::from).collect(),
            sends: None,
            sends_refused: None,
        }),
        Verified::Removed { .. } => entry(EntryResultDto::Removed),
        Verified::Behind { chain_epoch } => entry(EntryResultDto::Behind {
            chain_epoch: chain_epoch.get(),
            requests: None,
            requests_refused: None,
        }),
        Verified::Inconsistent { reason } => entry(EntryResultDto::Inconsistent { reason: reason.to_string() }),
        Verified::Unverified { reason } => entry(EntryResultDto::Unverified { reason: reason.to_string() }),
        Verified::Refused { error } => entry(EntryResultDto::Unverified { reason: error.to_string() }),
        Verified::Store { failure } => entry(EntryResultDto::Unverified { reason: failure.to_string() }),
        Verified::Keychain { error } => entry(EntryResultDto::Unverified { reason: error.to_string() }),
        Verified::Settled { outcome, write } => {
            let mut settled = entry_for(named, outcome);
            settled.write = Some(WriteLabelDto::from(*write));
            settled
        }
    }
}

/// One Device's outcome of a send or a request (LLR-rywe8f, LLR-nkg8sb).
#[derive(Debug, Serialize)]
pub struct DeviceReportDto {
    pub device: String,
    #[serde(flatten)]
    pub result: DeviceResultDto,
}

#[derive(Debug, Serialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum DeviceResultDto {
    Sent,
    Failed { reason: String },
}

/// One recipient's delivery as the app reports it: `Delivered` is sent;
/// every other outcome is failed, `NotDelivered` with org-io's reason, the
/// three other verdicts with a fixed text.
fn device_result(delivery: &Delivery) -> DeviceResultDto {
    let reason = match delivery {
        Delivery::Delivered => return DeviceResultDto::Sent,
        Delivery::NotDelivered { reason } => reason.to_string(),
        Delivery::Refused => "refused by the recipient".to_string(),
        Delivery::AlreadyReceived => "already received".to_string(),
        Delivery::InviteUsedByAnotherDevice => "invite used by another device".to_string(),
    };
    DeviceResultDto::Failed { reason }
}

/// org-io's per-Device report, in its order.
pub fn device_reports(report: &SendReport) -> Vec<DeviceReportDto> {
    report
        .per_device
        .iter()
        .map(|(device, delivery)| DeviceReportDto { device: hex::encode(device.as_bytes()), result: device_result(delivery) })
        .collect()
}

/// LLR-k4rxyt. Attach org-io's answer to the send or the request
/// `follow_up` names to `entry`: a report is `sends` (an adoption's) or
/// `requests` (a behind result's), one entry per Device, and no refusal; a
/// refusal of the whole is an empty `sends` or `requests` beside
/// `sends_refused` or `requests_refused` with its text. A follow-up the
/// entry's result does not ask (a send beside anything but an adoption, a
/// request beside anything but behind) changes nothing.
pub fn attach_report(entry: &mut VerificationEntryDto, follow_up: VerificationFollowUp, report: Result<SendReport, String>) {
    let (devices, refused) = match report {
        Ok(report) => (device_reports(&report), None),
        Err(refused) => (Vec::new(), Some(refused)),
    };
    match (follow_up, &mut entry.result) {
        (VerificationFollowUp::Send(_), EntryResultDto::Adopted { sends, sends_refused, .. }) => {
            *sends = Some(devices);
            *sends_refused = refused;
        }
        (VerificationFollowUp::Request(_), EntryResultDto::Behind { requests, requests_refused, .. }) => {
            *requests = Some(devices);
            *requests_refused = refused;
        }
        (VerificationFollowUp::Send(_) | VerificationFollowUp::Request(_), _) => {}
    }
}

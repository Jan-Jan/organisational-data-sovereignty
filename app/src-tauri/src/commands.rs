//! Tauri command handlers for the ODS PoC app.
//!
//! Each command calls the org-io method of its name on the shared handle
//! (`AppState.org_io`, an `Arc<OrgIo>` taking no app-side lock, LLR-5wb4vd)
//! with values parsed at the IPC edge, passes no randomness (org-io holds
//! the one random source, LLR-vj5s28), and maps org-io's outcome to a DTO
//! carrying no secret, its refusal to text for the Tauri `Result<T, String>`
//! convention (SDD-6pjvhc).
//!
//! ## `start_receiver` and its events
//!
//! Each iteration produces `crate::events::ReceiverOutcome`s, and
//! `crate::events::emissions_for` maps each to the events it produces. The
//! mapping is total, it is gated by `tests/receiver_events.rs`, and it is the
//! only place the event names and payload shapes are written down.

use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime, State};

use org_io::node::{DevicePublicKey, OrgId, OrgNodeError};
use org_io::verify::UnverifiedReason;
use org_io::view::PersonStatus;
use org_io::{CreateError, OrgIo, OrgSummary, PersonSummary, PersonaSummary};
use org_io::{Delivery, HeldTarget, Verified, WriteReport};
use org_io::KeptAdmissionRequestSummary;

use crate::entry::{attach_report, entry_for, EntryOrg, EntryResultDto, VerificationEntryDto};
use crate::parsing::{parse_founder, parse_invite_id, parse_member_id, parse_org_id};
use crate::state::{AppState, ConnectionStatus};
use crate::follow_up::{self, VerificationFollowUp};
use crate::{events, policy};

// ---------------------------------------------------------------------------
// Serialisable DTOs
// ---------------------------------------------------------------------------

/// A serialisable view of a Persona (org-io's `PersonaSummary`: no key
/// material). `handle` is null for a provisional Member (LLR-pguhw5's six
/// fields).
#[derive(Debug, Serialize)]
pub struct PersonaDto {
    pub persona_id: String,
    pub org_id: Option<String>,
    pub handle: Option<String>,
    pub name: String,
    pub surname: String,
    pub status: String,
}

impl From<&PersonaSummary> for PersonaDto {
    fn from(p: &PersonaSummary) -> Self {
        Self {
            persona_id: p.persona_id.as_str().to_string(),
            org_id: p.org_id.map(|id| hex::encode(id.as_bytes())),
            handle: p.handle.as_ref().map(ToString::to_string),
            name: p.name.to_string(),
            surname: p.surname.to_string(),
            status: format!("{:?}", p.status),
        }
    }
}

/// A serialisable view of an Organisation (org-io's `OrgSummary`).
#[derive(Debug, Serialize)]
pub struct OrgDto {
    pub org_id: String,
    pub epoch: u64,
    pub root_hash: String,
    pub member_count: usize,
    pub verification: VerificationDto,
}

impl OrgDto {
    /// `summary`, with `verification` as `OrgIo::verification` returned it.
    pub fn new(summary: &OrgSummary, last: &Verified) -> Self {
        Self {
            org_id: hex::encode(summary.org_id.as_bytes()),
            epoch: summary.epoch.get(),
            root_hash: hex::encode(summary.root_hash.as_bytes()),
            member_count: summary.members.len(),
            verification: VerificationDto::of(last),
        }
    }
}

/// An Organisation's last verification as `list_orgs` reports it
/// (LLR-g8wfy9): `not_checked` until one ran on this handle.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum VerificationDto {
    NotChecked,
    InStep { epoch: u64 },
    Adopted { epoch: u64 },
    Removed,
    Behind { chain_epoch: u64 },
    Inconsistent { reason: String },
    Unverified { reason: String },
}

impl VerificationDto {
    /// `last`, as `OrgIo::verification` returned it: org-io's
    /// `Unverified { NotYetVerified }` (no verification yet) is
    /// `not_checked`; a settled verification is its outcome's status; a
    /// refusal, a store failure or a keychain refusal is `unverified` with
    /// org-io's text.
    fn of(last: &Verified) -> Self {
        match last {
            Verified::Unverified { reason: UnverifiedReason::NotYetVerified } => Self::NotChecked,
            Verified::InStep { epoch } => Self::InStep { epoch: epoch.get() },
            Verified::Adopted { epoch, .. } => Self::Adopted { epoch: epoch.get() },
            Verified::Removed { .. } => Self::Removed,
            Verified::Behind { chain_epoch } => Self::Behind { chain_epoch: chain_epoch.get() },
            Verified::Inconsistent { reason } => Self::Inconsistent { reason: reason.to_string() },
            Verified::Unverified { reason } => Self::Unverified { reason: reason.to_string() },
            Verified::Refused { error } => Self::Unverified { reason: error.to_string() },
            Verified::Store { failure } => Self::Unverified { reason: failure.to_string() },
            Verified::Keychain { error } => Self::Unverified { reason: error.to_string() },
            Verified::Settled { outcome, .. } => Self::of(outcome),
        }
    }
}

/// The Person as `propose_person` returns it (LLR-k57uqv): its Device key
/// as 64 hex characters, the names org-io stored, and its status.
#[derive(Debug, Serialize)]
pub struct PersonDto {
    pub device_public_key: String,
    pub name: String,
    pub surname: String,
    pub status: &'static str,
}

impl From<&PersonSummary> for PersonDto {
    fn from(person: &PersonSummary) -> Self {
        Self {
            device_public_key: hex::encode(person.device_key.as_bytes()),
            name: person.name.to_string(),
            surname: person.surname.to_string(),
            status: match person.status {
                PersonStatus::Provisional => "provisional",
            },
        }
    }
}

/// A Persona a Member proposal created: its id and its status
/// (`provisional_member`, LLR-rg9vc7). A founding's is `ProposedFoundingDto`.
#[derive(Debug, Serialize)]
pub struct ProposedPersonaDto {
    pub persona_id: String,
    pub status: &'static str,
}

/// A founding a proposal created (LLR-6ff735): its
/// Persona's id, its status `provisional_founding`, and `founder`, the
/// founder's Device key as 64 hex characters, which
/// `write_organisation_to_chain` takes (LLR-rhc555).
#[derive(Debug, Serialize)]
pub struct ProposedFoundingDto {
    pub persona_id: String,
    pub status: &'static str,
    pub founder: String,
}

/// An Invite as the invitee is shown it (LLR-b7ywy4). Nothing in it is
/// verified (RC-wzb48r).
#[derive(Debug, Serialize)]
pub struct InviteDto {
    pub org_name: String,
    pub org_id: String,
    pub invitee_name: String,
    pub issuer_device_key: String,
}

/// The text `request_admission` refuses an unconfirmed request with
/// (LLR-q2k9z8).
pub const NOT_CONFIRMED_ADMISSION_REQUEST: &str = "the user has not confirmed the admission request";

/// org-io's outcome of an admission request as it crosses IPC (LLR-8cz828),
/// tagged by `result`, each reason org-io's text unchanged. `refused`
/// carries no reason: the receiving Device's verdict carries none.
#[derive(Debug, Serialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum AdmissionDto {
    Delivered,
    AlreadyReceived,
    InviteUsedByAnotherDevice,
    Refused,
    NotDelivered { reason: String },
}

impl From<&Delivery> for AdmissionDto {
    fn from(outcome: &Delivery) -> Self {
        match outcome {
            Delivery::Delivered => Self::Delivered,
            Delivery::AlreadyReceived => Self::AlreadyReceived,
            Delivery::InviteUsedByAnotherDevice => Self::InviteUsedByAnotherDevice,
            Delivery::Refused => Self::Refused,
            Delivery::NotDelivered { reason } => Self::NotDelivered { reason: reason.to_string() },
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// org-io's refusal of a creation as text, a refused field named first
/// (`name: <reason>`, `surname: <reason>`, `handle: <reason>`) without org-node's record prefix.
fn field_refusal(error: &CreateError) -> String {
    match error {
        CreateError::Node(OrgNodeError::InvalidField { field, reason }) => {
            let field = ["person.", "persona.", "member."]
                .iter()
                .find_map(|prefix| field.strip_prefix(prefix))
                .unwrap_or(field);
            format!("{field}: {reason}")
        }
        other => other.to_string(),
    }
}

/// The id of the Persona org-io lists with `device_key`: a proposal names
/// its new Persona by DevicePublicKey (LLR-6ff735's and LLR-rg9vc7's notes).
async fn persona_id_for(org_io: &OrgIo, device_key: &DevicePublicKey) -> Result<String, String> {
    org_io
        .personas()
        .await
        .iter()
        .find(|persona| &persona.device_key == device_key)
        .map(|persona| persona.persona_id.as_str().to_string())
        .ok_or_else(|| format!("org-io lists no Persona with DevicePublicKey {}", hex::encode(device_key.as_bytes())))
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Why `propose_person` refused (LLR-k57uqv), in the app's own terms.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonRefusalKind {
    /// A Person is held already: one Person per device.
    PersonAlreadyHeld,
    /// Any other refusal; its text names the field where there is one.
    Refused,
}

/// `propose_person`'s refusal as it crosses IPC: its kind, and org-io's
/// text (a refused field named first, as `field_refusal` gives it).
#[derive(Debug, Serialize)]
pub struct PersonRefusalDto {
    pub kind: PersonRefusalKind,
    pub message: String,
}

impl From<&CreateError> for PersonRefusalDto {
    fn from(error: &CreateError) -> Self {
        let kind = match error {
            CreateError::PersonAlreadyHeld { .. } => PersonRefusalKind::PersonAlreadyHeld,
            CreateError::Node(_) | CreateError::Keychain(_) | CreateError::Store(_) | CreateError::FoundingNotListed { .. } => {
                PersonRefusalKind::Refused
            }
        };
        Self { kind, message: field_refusal(error) }
    }
}

/// LLR-k57uqv: propose the user's Person from a name and a surname (no
/// Handle); a refusal carries its kind and names the field, and creates
/// nothing.
#[tauri::command]
pub async fn propose_person(
    state: State<'_, AppState>,
    name: String,
    surname: String,
) -> Result<PersonDto, PersonRefusalDto> {
    let person = state.org_io.propose_person(&name, &surname).await.map_err(|error| PersonRefusalDto::from(&error))?;
    Ok(PersonDto::from(&person))
}

/// LLR-6ff735: propose a founding, which creates a new Persona; returns
/// that Persona's id and its founder's Device key. A refusal names the
/// field and leaves no new Persona.
#[tauri::command]
pub async fn propose_founding_organisation(
    state: State<'_, AppState>,
    name: String,
    surname: String,
    handle: String,
    org_name: String,
) -> Result<ProposedFoundingDto, String> {
    let founding = state
        .org_io
        .propose_founding_organisation(&name, &surname, &handle, &org_name)
        .await
        .map_err(|error| field_refusal(&error))?;
    let persona_id = persona_id_for(&state.org_io, &founding.founder).await?;
    Ok(ProposedFoundingDto {
        persona_id,
        status: "provisional_founding",
        founder: hex::encode(founding.founder.as_bytes()),
    })
}

/// LLR-y3zykm: issue an Invite to `org_id` for `invitee_name`; org-io's
/// Blob, unchanged. No Organisation name (the stored one) and no address
/// (the issuing Device's) is taken.
#[tauri::command]
pub async fn issue_invite(state: State<'_, AppState>, org_id: String, invitee_name: String) -> Result<String, String> {
    let org_id = parse_org_id(&org_id)?;
    state.org_io.issue_invite(org_id, &invitee_name).await.map_err(|error| error.to_string())
}

/// LLR-b7ywy4: read an Invite Blob; stores nothing.
#[tauri::command]
pub async fn read_invite(state: State<'_, AppState>, blob: String) -> Result<InviteDto, String> {
    let invite = state.org_io.read_invite(&blob).await.map_err(|error| error.to_string())?;
    Ok(InviteDto {
        org_name: invite.org_name.to_string(),
        org_id: hex::encode(invite.org_id.as_bytes()),
        invitee_name: invite.invitee_name.to_string(),
        issuer_device_key: hex::encode(invite.issuer.as_bytes()),
    })
}

/// LLR-rg9vc7: propose a provisional Member from an Invite Blob, its
/// preferred Handles handed to org-io in the order given; returns the new
/// Persona's id. Sends nothing.
#[tauri::command]
pub async fn propose_member(
    state: State<'_, AppState>,
    invite_blob: String,
    name: String,
    surname: String,
    preferred_handles: Vec<String>,
) -> Result<ProposedPersonaDto, String> {
    let handles: Vec<&str> = preferred_handles.iter().map(String::as_str).collect();
    let member = state
        .org_io
        .propose_member(&invite_blob, &name, &surname, &handles)
        .await
        .map_err(|error| error.to_string())?;
    let persona_id = persona_id_for(&state.org_io, &member.member_device).await?;
    Ok(ProposedPersonaDto { persona_id, status: "provisional_member" })
}

/// LLR-q2k9z8, LLR-8cz828: send the provisional Member's admission request,
/// only once the user confirmed, for the Organisation org-io lists for that
/// Persona; org-io's outcome, tagged.
#[tauri::command]
pub async fn request_admission(
    state: State<'_, AppState>,
    persona_id: String,
    confirmed: bool,
) -> Result<AdmissionDto, String> {
    if !confirmed {
        return Err(NOT_CONFIRMED_ADMISSION_REQUEST.to_string());
    }
    let persona = state
        .org_io
        .personas()
        .await
        .into_iter()
        .find(|persona| persona.persona_id.as_str() == persona_id)
        .ok_or_else(|| format!("no Persona {persona_id} is held here"))?;
    let org_id = persona.org_id.ok_or_else(|| format!("Persona {persona_id} belongs to no Organisation"))?;
    let outcome = state.org_io.request_admission(org_id).await.map_err(|error| error.to_string())?;
    Ok(AdmissionDto::from(&outcome))
}

/// The text `propose_admission` refuses an unconfirmed admission with
/// (LLR-cr9nmw).
pub const NOT_CONFIRMED_ADMISSION: &str = "the Admin has not confirmed the admission";

/// LLR-gmb4p8: a Handle that is absent, empty or only whitespace is none
/// (org-io chooses the first preferred one free); any other is the Admin's
/// choice, unchanged.
fn chosen_handle(handle: Option<&str>) -> Option<&str> {
    handle.filter(|handle| !handle.trim().is_empty())
}

/// LLR-cr9nmw, LLR-gmb4p8: record the admission of the request kept for
/// `invite_id`, once the Admin confirmed. It takes these four values alone.
#[tauri::command]
pub async fn propose_admission(
    state: State<'_, AppState>,
    org_id: String,
    invite_id: String,
    handle: Option<String>,
    confirmed: bool,
) -> Result<(), String> {
    let org_id = parse_org_id(&org_id)?;
    let invite_id = parse_invite_id(&invite_id)?;
    if !confirmed {
        return Err(NOT_CONFIRMED_ADMISSION.to_string());
    }
    state
        .org_io
        .propose_admission(org_id, invite_id, chosen_handle(handle.as_deref()))
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
}

/// LLR-7n4hgr: record the revocation of `member_id`; no peer address, and
/// nothing is sent.
#[tauri::command]
pub async fn propose_revocation(state: State<'_, AppState>, org_id: String, member_id: String) -> Result<(), String> {
    let org_id = parse_org_id(&org_id)?;
    let member_id = parse_member_id(&member_id)?;
    state.org_io.propose_revocation(org_id, member_id).await.map(|_| ()).map_err(|error| error.to_string())
}

/// LLR-vqz4s6: cancel the outstanding invite `invite_id` and its kept
/// request; org-io's refusal as its text.
#[tauri::command]
pub async fn cancel_invite(state: State<'_, AppState>, org_id: String, invite_id: String) -> Result<(), String> {
    let org_id = parse_org_id(&org_id)?;
    let invite_id = parse_invite_id(&invite_id)?;
    state.org_io.cancel_invite(org_id, invite_id).await.map_err(|error| error.to_string())
}

/// List all local personas (no key material returned).
#[tauri::command]
pub async fn list_personas(state: State<'_, AppState>) -> Result<Vec<PersonaDto>, String> {
    Ok(state.org_io.personas().await.iter().map(PersonaDto::from).collect())
}

/// List all local org records, each with its last verification.
#[tauri::command]
pub async fn list_orgs(state: State<'_, AppState>) -> Result<Vec<OrgDto>, String> {
    let mut orgs = Vec::new();
    for org in state.org_io.organisations().await {
        let last = state.org_io.verification(HeldTarget::Organisation(org.org_id)).await;
        orgs.push(OrgDto::new(&org, &last));
    }
    Ok(orgs)
}

/// Return the current connection status: what the running configuration was
/// actually built from, not what the environment currently says.
///
/// REQ-e4ah9h / REQ-bvx4nh / REQ-645jq9. Nothing here reads the environment.
#[tauri::command]
pub async fn connection_status<R: Runtime>(
    state: State<'_, AppState>,
    app_handle: AppHandle<R>,
) -> Result<ConnectionStatus, String> {
    // The handle stays in the signature because it is what makes this
    // command generic over the runtime, and a command that is not generic
    // over the runtime cannot be driven by `MockRuntime`.
    let _ = &app_handle;
    Ok(policy::connection_status_from_state(
        &state.data_dir,
        state.chain_endpoint.as_ref(),
        state.transport_mode,
    ))
}

// ---------------------------------------------------------------------------
// start_receiver: spawns a background task looping org-io's `next_received`
// ---------------------------------------------------------------------------

/// One iteration of the receiver loop, as outcomes: none, one, or (for a
/// terminal error) the failure and the stop (REQ-affyf5, REQ-jfxah3).
///
/// `org_io` is the shared handle (LLR-5wb4vd); no app-side lock is taken, so
/// the wait in `next_received` blocks no command.
async fn next_outcomes(org_io: &OrgIo) -> Vec<events::ReceiverOutcome> {
    match org_io.next_received().await {
        Ok(received) => vec![events::outcome_for_arrival(events::Arrival::from(received))],
        // RC-3rddh7 / REQ-kn5rtx / REQ-jfxah3: classified by the error's
        // variant, the loop's exit included (LLR-zw6jqy).
        Err(error) => events::outcomes_for_receive_error(&error),
    }
}

/// LLR-acb5g4, LLR-w7m3d4, LLR-2267hj. One verification by the body
/// `verify_and_send` runs (`verified_entry`, or `verified_entries` for
/// every target), so an own adoption is sent once and a behind
/// Organisation requested, with each entry mapped by
/// `events::outcomes_for_verification`; only an `adopted` entry's record is
/// read back. The receiver loop's follow-up and the periodic run both
/// announce verifications through this function.
pub async fn verification_outcomes(org_io: &OrgIo, target: Option<HeldTarget>) -> Vec<events::ReceiverOutcome> {
    // `Some` verifies that one target; `None` every target org-io holds, in its order.
    let entries = match target {
        Some(target) => vec![verified_entry(org_io, target).await],
        None => verified_entries(org_io).await,
    };
    let mut outcomes = Vec::new();
    for (org_id, entry) in entries {
        let record = match (&entry.result, org_id) {
            (EntryResultDto::Adopted { .. }, Some(org_id)) => org_io
                .organisation(org_id)
                .await
                .map(|organisation| (organisation.epoch.get(), hex::encode(organisation.root_hash.as_bytes()))),
            // A founding with no proxy names no Organisation whose record could be read.
            (EntryResultDto::Adopted { .. }, None) => None,
            (
                EntryResultDto::InStep { .. }
                | EntryResultDto::Removed
                | EntryResultDto::Behind { .. }
                | EntryResultDto::Inconsistent { .. }
                | EntryResultDto::Unverified { .. },
                _,
            ) => None,
        };
        outcomes.extend(events::outcomes_for_verification(&entry, record));
    }
    outcomes
}

/// LLR-w7m3d4: the receiver loop's follow-up for `org_id` (40 hex, as
/// `events` names it): one verification of that held Organisation.
async fn verify_received(org_io: &OrgIo, org_id: String) -> Vec<events::ReceiverOutcome> {
    match parse_org_id(&org_id) {
        Ok(org_id) => verification_outcomes(org_io, Some(HeldTarget::Organisation(org_id))).await,
        // Unreachable: `events` wrote this id from an OrgId. Reported, not panicked.
        Err(message) => vec![events::ReceiverOutcome::ReceiveError { message }],
    }
}

/// Spawn a background tokio task that loops org-io's `next_received` and
/// emits Tauri events. The task runs until the app is closed or the
/// endpoint returns a permanent error (endpoint closed).
///
/// REQ-6hgm8r: only one loop runs at a time. REQ-3hfggn: when the loop ends —
/// by break, by return, or by panic — the slot is released and a later call may
/// start a new one (HAZ-cfp4jb).
///
/// Returns immediately; the task runs in the background.
#[tauri::command]
pub async fn start_receiver<R: Runtime>(
    state: State<'_, AppState>,
    app_handle: AppHandle<R>,
) -> Result<(), String> {
    let Some(guard) = events::StartGuard::try_claim(&state.receiver_started) else {
        return Ok(()); // REQ-6hgm8r: a loop is already running.
    };

    let app_handle_clone = app_handle.clone();
    // LLR-5wb4vd: the loop holds its own clone of the shared handle.
    let org_io = Arc::clone(&state.org_io);
    tokio::spawn(async move {
        // REQ-3hfggn: the guard is moved in, so the slot is released when this
        // task ends — by break, by return, or by panic.
        let _guard = guard;
        // A bind that fails (`ReceiveError::Terminal`) ends the loop before
        // it begins.
        if let Err(error) = org_io.start_receiving().await {
            for outcome in &events::outcomes_for_receive_error(&error) {
                for emission in events::emissions_for(outcome) {
                    // Events are the only channel to the webview, so a
                    // failed emit has nowhere to be reported.
                    let _ = app_handle_clone.emit(emission.name, emission.payload);
                }
            }
            return;
        }
        // LLR-w7m3d4: each iteration announces what arrived and, before the
        // next message, the verification a received provisional
        // organisation or a kept notice asks for, with no user step.
        let org_io_ref: &OrgIo = &org_io;
        loop {
            let stopping = events::receive_iteration(
                move || next_outcomes(org_io_ref),
                move |org_id| verify_received(org_io_ref, org_id),
                |emission| {
                    // Events are the only channel to the webview, so a
                    // failed emit has nowhere to be reported.
                    let _ = app_handle_clone.emit(emission.name, emission.payload);
                },
            )
            .await;
            if stopping {
                break;
            }
        }
    });

    Ok(())
}

// ---------------------------------------------------------------------------
// Proposing the updated Organisation and writing it (LLR-2u3cga,
// LLR-rhc555, LLR-k4rhbs)
// ---------------------------------------------------------------------------

/// What `propose_updated_organisation` returns (LLR-2u3cga): org-io's
/// `ProposedOrganisation`, each root and key as 64 hex characters.
#[derive(Debug, Serialize)]
pub struct ProposedOrganisationDto {
    pub epoch: u64,
    pub root: String,
    pub org_public_key: String,
}

/// LLR-2u3cga: have org-io propose the next epoch from the held provisional
/// changes; writes nothing to the chain.
#[tauri::command]
pub async fn propose_updated_organisation(
    state: State<'_, AppState>,
    org_id: String,
) -> Result<ProposedOrganisationDto, String> {
    let org_id = parse_org_id(&org_id)?;
    let proposed = state.org_io.propose_updated_organisation(org_id).await.map_err(|error| error.to_string())?;
    Ok(ProposedOrganisationDto {
        epoch: proposed.epoch.get(),
        root: hex::encode(proposed.root.as_bytes()),
        org_public_key: hex::encode(proposed.org_public_key.as_bytes()),
    })
}

/// What an entry for `target` names: an Organisation by its id; a founding
/// by its founder, always, beside the id its founder's Persona holds once
/// its proxy is recorded and none before (LLR-jf63e4).
async fn entry_org_for(org_io: &OrgIo, target: &HeldTarget) -> EntryOrg {
    match target {
        HeldTarget::Organisation(org_id) => EntryOrg::Id(*org_id),
        HeldTarget::Founding(founder) => EntryOrg::Founding {
            founder: *founder,
            org_id: org_io
                .personas()
                .await
                .into_iter()
                .find(|persona| &persona.device_key == founder)
                .and_then(|persona| persona.org_id),
        },
    }
}

/// LLR-es49wp, LLR-nkg8sb: make the call `follow_up` names (decided by
/// SDD-6zqqm8's `after_verification`) and attach its report to `entry`
/// (LLR-k4rxyt).
async fn make_follow_up(org_io: &OrgIo, follow_up: VerificationFollowUp, entry: &mut VerificationEntryDto) {
    let report = match follow_up {
        VerificationFollowUp::Send(org_id) => org_io.send_organisation(org_id).await,
        VerificationFollowUp::Request(org_id) => org_io.request_organisation(org_id).await,
    };
    attach_report(entry, follow_up, report.map_err(|refused| refused.to_string()));
}

/// The entry for `verified`, after its send or request, beside the
/// Organisation it names (none for a founding with no proxy).
async fn entry_and_follow_up(
    org_io: &OrgIo,
    target: &HeldTarget,
    verified: &Verified,
) -> (Option<OrgId>, VerificationEntryDto) {
    let named = entry_org_for(org_io, target).await;
    let mut entry = entry_for(&named, verified);
    let org_id = match named {
        EntryOrg::Id(org_id) | EntryOrg::Founding { org_id: Some(org_id), .. } => Some(org_id),
        EntryOrg::Founding { org_id: None, .. } => None,
    };
    if let Some(follow_up) = follow_up::after_verification(verified, org_id) {
        make_follow_up(org_io, follow_up, &mut entry).await;
    }
    (org_id, entry)
}

/// One verification of `target` and its follow-up, the body
/// `verify_and_send` runs: its entry beside the Organisation it names.
async fn verified_entry(org_io: &OrgIo, target: HeldTarget) -> (Option<OrgId>, VerificationEntryDto) {
    let verified = org_io.verify_organisation(target).await;
    entry_and_follow_up(org_io, &target, &verified).await
}

/// Every target org-io holds, verified and followed up as `verified_entry`
/// does, the body `verify_all_and_send` runs: each entry beside the
/// Organisation it names.
async fn verified_entries(org_io: &OrgIo) -> Vec<(Option<OrgId>, VerificationEntryDto)> {
    let mut entries = Vec::new();
    for (target, verified) in org_io.verify_all().await {
        entries.push(entry_and_follow_up(org_io, &target, &verified).await);
    }
    entries
}

/// The body every verification runs (SDD-6pjvhc): `verify_organisation`'s,
/// the receiver loop's, the write's and the periodic run's.
pub async fn verify_and_send(org_io: &OrgIo, target: HeldTarget) -> VerificationEntryDto {
    let (_, entry) = verified_entry(org_io, target).await;
    entry
}

/// `verify_and_send` for every target org-io holds, in org-io's order,
/// each outcome beside the target it verified (LLR-jf63e4 with no
/// `org_id`; org-io's order of held targets).
pub async fn verify_all_and_send(org_io: &OrgIo) -> Vec<VerificationEntryDto> {
    verified_entries(org_io).await.into_iter().map(|(_, entry)| entry).collect()
}

/// LLR-jf63e4: verify one held Organisation (`org_id`), or every target
/// org-io holds, sending and requesting as `verify_and_send` does.
#[tauri::command]
pub async fn verify_organisation(
    state: State<'_, AppState>,
    org_id: Option<String>,
) -> Result<Vec<VerificationEntryDto>, String> {
    match org_id {
        Some(org_id) => {
            let org_id = parse_org_id(&org_id)?;
            Ok(vec![verify_and_send(&state.org_io, HeldTarget::Organisation(org_id)).await])
        }
        None => Ok(verify_all_and_send(&state.org_io).await),
    }
}

/// What `write_organisation_to_chain` returns (LLR-rhc555), tagged by
/// `result`; an included write carries its verification (LLR-k4rhbs).
#[derive(Debug, Serialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum WriteDto {
    /// `verification` is present whenever SDD-6zqqm8 asks one, which it
    /// does for every included write (LLR-bbfgk7).
    Included {
        epoch: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        verification: Option<VerificationEntryDto>,
    },
    Refused { reason: String },
    Unknown { reason: String },
}

/// `write_organisation_to_chain`'s body: org-io's write of `target`, then,
/// only when included, one verification of the same target (LLR-k4rhbs).
/// org-io's refusal before any chain call is the command's error text, as
/// org-io's `WriteRefused` displays it (public data only).
pub async fn write_then_verify(org_io: &OrgIo, target: HeldTarget) -> Result<WriteDto, String> {
    let report = org_io.write_organisation_to_chain(target).await.map_err(|error| error.to_string())?;
    // SDD-6zqqm8 decides whether the write asks a verification (LLR-bbfgk7).
    let verification =
        if follow_up::verifies_after_write(&report) { Some(verify_and_send(org_io, target).await) } else { None };
    Ok(match report {
        WriteReport::Included { epoch } => WriteDto::Included { epoch: epoch.get(), verification },
        WriteReport::Refused { failure } => WriteDto::Refused { reason: failure.to_string() },
        WriteReport::Unknown { reason } => WriteDto::Unknown { reason },
    })
}

/// The text `write_organisation_to_chain` refuses both or neither target
/// with (LLR-rhc555).
pub const EXACTLY_ONE_TARGET: &str = "write_organisation_to_chain takes exactly one of org_id or founder";

/// LLR-rhc555, LLR-k4rhbs: write the latest own provisional organisation of
/// `org_id`, or of the founding of `founder`, exactly one of them.
#[tauri::command]
pub async fn write_organisation_to_chain(
    state: State<'_, AppState>,
    org_id: Option<String>,
    founder: Option<String>,
) -> Result<WriteDto, String> {
    // LLR-97hujy: the org id is parsed before any other argument.
    let org_id = org_id.as_deref().map(parse_org_id).transpose()?;
    let target = match (org_id, founder.as_deref()) {
        (Some(org_id), None) => HeldTarget::Organisation(org_id),
        (None, Some(founder)) => HeldTarget::Founding(parse_founder(founder)?),
        _ => return Err(EXACTLY_ONE_TARGET.to_string()),
    };
    write_then_verify(&state.org_io, target).await
}

/// One kept admission request as `list_admission_requests` returns it
/// (LLR-y7nc6x): the `admission-requested` event's fields and encodings
/// (LLR-dp45mm).
#[derive(Debug, Serialize)]
pub struct AdmissionRequestDto {
    pub org_id: String,
    pub invite_id: String,
    pub invitee_name: String,
    pub name: String,
    pub surname: String,
    pub preferred_handles: Vec<String>,
    pub device_public_key: String,
}

impl From<&KeptAdmissionRequestSummary> for AdmissionRequestDto {
    fn from(kept: &KeptAdmissionRequestSummary) -> Self {
        Self {
            org_id: hex::encode(kept.org_id.as_bytes()),
            invite_id: hex::encode(kept.invite_id.as_bytes()),
            invitee_name: kept.invitee_name.clone(),
            name: kept.name.to_string(),
            surname: kept.surname.to_string(),
            preferred_handles: kept.preferred_handles.iter().map(ToString::to_string).collect(),
            device_public_key: hex::encode(kept.device_key.as_bytes()),
        }
    }
}

/// LLR-y7nc6x: every admission request org-io keeps, in its order; changes
/// nothing. The one command of the surface not named as its org-io method
/// (`admission_requests`).
#[tauri::command]
pub async fn list_admission_requests(state: State<'_, AppState>) -> Result<Vec<AdmissionRequestDto>, String> {
    Ok(state.org_io.admission_requests().await.iter().map(AdmissionRequestDto::from).collect())
}

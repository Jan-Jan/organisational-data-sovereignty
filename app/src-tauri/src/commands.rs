//! Tauri command handlers for the ODS PoC app.
//!
//! Each command calls the matching org-io operation on the shared handle
//! (`AppState.org_io`, an `Arc<OrgIo>` taking no app-side lock, LLR-5wb4vd),
//! and maps org-io's typed error → `String` for the Tauri `Result<T, String>`
//! convention. The app names no iroh type and no Blob format; org-io runs the
//! invitation exchange, the fan-out and the peer-address parse.
//!
//! ## `start_receiver` and its events
//!
//! The loop no longer decides what to announce. Each iteration produces a
//! `crate::events::ReceiverOutcome`, and `crate::events::emissions_for` maps
//! that outcome to the events it produces. The mapping is total, it is gated by
//! `tests/receiver_events.rs`, and it is the only place the event names and
//! payload shapes are written down — see that module for the vocabulary.
//!
//! The loop's own responsibility is reduced to three things it cannot delegate:
//! claiming the right to run (`events::StartGuard`), turning one service call
//! into one outcome (`next_outcomes`), and emitting.

use std::sync::Arc;

use rand::rngs::OsRng;
use rand::{CryptoRng, RngCore};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime, State};

use org_io::node::{Invite, MemberId, OrgId, OrgNodeError, PersonaDetails, PersonaId};
use org_io::{parse_peer_addr, OrgIo, OrgSummary, PersonaSummary, Received, Reconciliation, SendReport, Verification};

use crate::parsing::parse_org_id;
use crate::state::{AppState, ConnectionStatus};
use crate::{events, policy};

// ---------------------------------------------------------------------------
// Serialisable DTOs
// ---------------------------------------------------------------------------

/// A serialisable view of a Persona (org-io's `PersonaSummary`: no key
/// material).
#[derive(Debug, Serialize)]
pub struct PersonaDto {
    pub persona_id: String,
    pub org_id: Option<String>,
    pub handle: String,
    pub name: String,
    pub surname: String,
    pub status: String,
}

impl From<&PersonaSummary> for PersonaDto {
    fn from(p: &PersonaSummary) -> Self {
        Self {
            persona_id: p.persona_id.as_str().to_string(),
            org_id: p.org_id.map(|id| hex::encode(id.as_bytes())),
            handle: p.handle.to_string(),
            name: p.name.to_string(),
            surname: p.surname.to_string(),
            status: format!("{:?}", p.status),
        }
    }
}

/// A serialisable view of an Organisation (org-io's `OrgSummary`), with the
/// status its last reconcile left (LLR-8axx7h).
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
    pub fn new(summary: &OrgSummary, verification: Option<Verification>) -> Self {
        Self {
            org_id: hex::encode(summary.org_id.as_bytes()),
            epoch: summary.epoch.get(),
            root_hash: hex::encode(summary.root_hash.as_bytes()),
            member_count: summary.members.len(),
            verification: verification.map_or(VerificationDto::NotChecked, |verification| match verification {
                Verification::Verified => VerificationDto::Verified,
                Verification::Unverified { reason } => VerificationDto::Unverified { reason },
            }),
        }
    }
}

/// Whether an Organisation's last reconcile read its state from the chain
/// (LLR-8axx7h); `NotChecked` until one ran.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum VerificationDto {
    NotChecked,
    Verified,
    Unverified { reason: String },
}

/// One Organisation's reconcile result as it crosses IPC (LLR-ekz4zn):
/// org-io's `Reconciliation`, tagged by `result`, ids and keys as hex.
#[derive(Debug, Serialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum ReconciliationDto {
    InStep { org_id: String, epoch: u64 },
    Committed { org_id: String, epoch: u64, recipients: Vec<String> },
    Behind { org_id: String, chain_epoch: u64 },
    Removed { org_id: String },
    Refused { org_id: String, reason: String },
    Unverified { org_id: String, reason: String },
}

impl From<&Reconciliation> for ReconciliationDto {
    fn from(reconciliation: &Reconciliation) -> Self {
        let hex_id = |org_id: &OrgId| hex::encode(org_id.as_bytes());
        match reconciliation {
            Reconciliation::InStep { org_id, epoch } => Self::InStep { org_id: hex_id(org_id), epoch: epoch.get() },
            Reconciliation::Committed { org_id, epoch, recipients } => Self::Committed {
                org_id: hex_id(org_id),
                epoch: epoch.get(),
                recipients: recipients.iter().map(|device| hex::encode(device.as_bytes())).collect(),
            },
            Reconciliation::Behind { org_id, chain_epoch } => {
                Self::Behind { org_id: hex_id(org_id), chain_epoch: chain_epoch.get() }
            }
            Reconciliation::Removed { org_id } => Self::Removed { org_id: hex_id(org_id) },
            Reconciliation::Refused { org_id, reason } => Self::Refused { org_id: hex_id(org_id), reason: reason.clone() },
            Reconciliation::Unverified { org_id, reason } => {
                Self::Unverified { org_id: hex_id(org_id), reason: reason.clone() }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Create a new persona (no chain interaction).
///
/// The handle, name and surname are parsed here, at the command boundary,
/// by org-node's `PersonaDetails::parse`; a refusal names the field it is
/// about (`handle: …`, `name: …`, `surname: …`).
#[tauri::command]
pub async fn create_persona(
    state: State<'_, AppState>,
    handle: String,
    name: String,
    surname: String,
) -> Result<String, String> {
    let PersonaDetails { handle, name, surname } =
        PersonaDetails::parse(&handle, &name, &surname).map_err(|e| match e {
            OrgNodeError::InvalidField { field, reason } => {
                format!("{}: {reason}", field.trim_start_matches("persona."))
            }
            other => other.to_string(),
        })?;
    let org_io = &state.org_io;
    org_io.create_persona(&mut OsRng, handle, name, surname)
        .await
        .map(|id| id.as_str().to_string())
        .map_err(|error| error.to_string())
}

/// Create an organisation: the app decides to found it and hands that to
/// org-io, which has org-node build the genesis update, writes it to the
/// chain, and only then has org-node commit it (REQ-m8sgjk, REQ-nfr3n2).
/// Returns the org_id (40 hex chars).
#[tauri::command]
pub async fn create_organisation(
    state: State<'_, AppState>,
    persona_id: String,
) -> Result<String, String> {
    let org_io = &state.org_io;
    let org_id = org_io.found_organisation(&mut OsRng, &PersonaId::new(persona_id)).await?;
    Ok(hex::encode(org_id.as_bytes()))
}

/// Issue an Invite to `org_id`: org-io's exchange (`OrgIo::issue_invite`)
/// returns its Blob, its fresh invite identifier kept as outstanding in the
/// bound Persona's store.
#[tauri::command]
pub async fn export_invite(
    state: State<'_, AppState>,
    org_id: String,
    org_name: String,
    invitee_name: String,
) -> Result<String, String> {
    let oid = parse_org_id(&org_id)?;
    let org_io = &state.org_io;
    org_io.issue_invite(&mut OsRng, oid, &org_name, &invitee_name).await.map_err(|error| error.to_string())
}

/// An Invite as the invitee is shown it. Nothing in it is verified
/// (RC-wzb48r).
#[derive(Debug, Serialize)]
pub struct InviteDto {
    pub org_name: String,
    pub org_id: String,
    pub invitee_name: String,
    pub inviter_device_keys: Vec<String>,
    pub invite_id: String,
}

/// Parse an Invite Blob (org-node's parse, naming the field); stores nothing.
#[tauri::command]
pub async fn import_invite(blob: String) -> Result<InviteDto, String> {
    let invite = Invite::parse(&blob).map_err(|error| error.to_string())?;
    Ok(InviteDto {
        org_name: invite.org_name,
        org_id: hex::encode(invite.org_id.as_bytes()),
        invitee_name: invite.invitee_name,
        inviter_device_keys: invite.inviter_device_keys.iter().map(|k| hex::encode(k.as_bytes())).collect(),
        invite_id: hex::encode(invite.invite_id.as_bytes()),
    })
}

/// The reply to an Invite from `persona_id`, only once the user confirmed:
/// org-io's exchange (`OrgIo::produce_reply`) refuses without the
/// confirmation and for a Persona already bound.
#[tauri::command]
pub async fn produce_invite_reply(
    state: State<'_, AppState>,
    invite_blob: String,
    persona_id: String,
    confirmed: bool,
) -> Result<String, String> {
    let org_io = &state.org_io;
    org_io.produce_reply(&mut OsRng, &invite_blob, &PersonaId::new(persona_id), confirmed)
        .await
        .map_err(|error| error.to_string())
}

/// An Invite reply as the inviter is shown it before admitting.
#[derive(Debug, Serialize)]
pub struct InviteReplyDto {
    pub org_id: String,
    pub handle: String,
    pub name: String,
    pub surname: String,
    pub member_key: String,
    pub device_key: String,
}

/// Parse an Invite reply Blob naming an outstanding Invite (LLR-pmus9f;
/// org-io's `import_reply`); stores nothing.
#[tauri::command]
pub async fn import_invite_reply(state: State<'_, AppState>, blob: String) -> Result<InviteReplyDto, String> {
    let reply = state.org_io.import_reply(&blob).await.map_err(|error| error.to_string())?;
    Ok(InviteReplyDto {
        org_id: hex::encode(reply.org_id.as_bytes()),
        handle: reply.handle.to_string(),
        name: reply.name.to_string(),
        surname: reply.surname.to_string(),
        member_key: hex::encode(reply.member_key.as_bytes()),
        device_key: hex::encode(reply.device_key.as_bytes()),
    })
}

/// Admit the person an Invite reply names (REQ-m8sgjk): the app hands the
/// reply to org-io's exchange (`OrgIo::admit_reply`), which checks it
/// against the outstanding pairs for `org_id` (the operator's selection,
/// refused unless it is the reply's Organisation), has org-node build the
/// admission, writes it to the chain, has org-node commit it, sends it to
/// every recipient and settles the Invite. The command reports what org-io
/// returned (`report_sends`).
///
/// The org id is parsed here, then the peer address by org-io's
/// `parse_peer_addr`, both before the reply is. It takes no
/// Organisation secret or key (LLR-8krgzj): the key the admission's message
/// carries is the one org-node's record holds. Returns the new member_id as
/// 64 hex chars.
#[tauri::command]
pub async fn admit_member(
    state: State<'_, AppState>,
    org_id: String,
    reply_blob: String,
    peer_addr_blob: String,
) -> Result<String, String> {
    let oid = parse_org_id(&org_id)?;
    let peer_addr = parse_peer_addr(&peer_addr_blob).map_err(|error| error.to_string())?;
    let org_io = &state.org_io;
    let admitted = org_io.admit_reply(&mut OsRng, oid, &reply_blob, peer_addr).await.map_err(|error| error.to_string())?;
    report_sends(&admitted.sends)?;
    Ok(hex::encode(admitted.member_id.as_bytes()))
}

/// The committed update's sends as the command reports them: `Ok` when
/// every recipient's body was delivered, else the failures by Device key.
/// The update was committed either way, and the message says so.
fn report_sends(sends: &SendReport) -> Result<(), String> {
    match sends.failure_summary() {
        None => Ok(()),
        Some(failed) => Err(format!("the update was committed, but sending it failed: {failed}")),
    }
}

/// Revoke a member by member_id (64 hex chars). `peer_addr_blob` (hex of
/// an endpoint address, parsed by org-io's `parse_peer_addr`) is OPTIONAL:
/// leave it empty in Networked mode, where each recipient is reached by the
/// endpoint identity equal to its Device key. It is only needed for
/// same-machine Loopback dialling.
///
/// REQ-he8ejb. The `0x` prefix on `member_id_hex` is ONE optional prefix, not a
/// run of them — the same correction `parsing::parse_org_id` carries. A
/// repeated strip accepted `0x0x` + 64 hex characters as a well-formed member
/// id and revoked against it; one strip leaves a second `0x` for the hex decode
/// to refuse. Gated by `tests/ipc.rs`.
#[tauri::command]
pub async fn revoke_member(
    state: State<'_, AppState>,
    org_id: String,
    member_id_hex: String,
    peer_addr_blob: String,
) -> Result<(), String> {
    let oid = parse_org_id(&org_id)?;
    let member_bytes = hex::decode(member_id_hex.strip_prefix("0x").unwrap_or(&member_id_hex))
        .map_err(|e| format!("member_id hex: {e}"))?;
    if member_bytes.len() != 32 {
        return Err("member_id must be 32 bytes (64 hex chars)".into());
    }
    let mut member_id = [0u8; 32];
    member_id.copy_from_slice(&member_bytes);

    let peer_addr = parse_peer_addr(&peer_addr_blob).map_err(|error| error.to_string())?;

    // REQ-m8sgjk: org-io has org-node build the removal, writes it to the
    // chain, has org-node commit it, and sends it to every recipient the
    // commit names (org-io's fan-out); the command reports what org-io
    // returned.
    let org_io = &state.org_io;
    let update =
        org_io.revoke_member(&mut OsRng, oid, MemberId::new(member_id)).await.map_err(|error| error.to_string())?;
    let submitted = org_io.submit_commit_send(&mut OsRng, &update, peer_addr).await?;
    report_sends(&submitted.sends)
}

/// List all local personas (no key material returned).
#[tauri::command]
pub async fn list_personas(state: State<'_, AppState>) -> Result<Vec<PersonaDto>, String> {
    let org_io = &state.org_io;
    Ok(org_io.personas().await.iter().map(PersonaDto::from).collect())
}

/// List all local org records.
#[tauri::command]
pub async fn list_orgs(state: State<'_, AppState>) -> Result<Vec<OrgDto>, String> {
    let org_io = &state.org_io;
    // The handle's views and its verification status are `async`: the
    // publish state is behind its own lock.
    let mut orgs = Vec::new();
    for org in org_io.organisations().await {
        orgs.push(OrgDto::new(&org, org_io.verification(org.org_id).await));
    }
    Ok(orgs)
}

/// Ask org-io to reconcile every held Organisation with the chain
/// (LLR-ekz4zn, REQ-363366): the webview calls it at start and on the
/// user's request.
#[tauri::command]
pub async fn reconcile_organisations(state: State<'_, AppState>) -> Result<Vec<ReconciliationDto>, String> {
    Ok(reconcile_all(&state.org_io, &mut OsRng).await)
}

/// `reconcile_organisations`'s body: org-io's results, one per held
/// Organisation, in org-io's order. The handle's methods take `&self`.
pub async fn reconcile_all<R: RngCore + CryptoRng + Send>(org_io: &OrgIo, rng: &mut R) -> Vec<ReconciliationDto> {
    org_io.reconcile_held(rng).await.iter().map(ReconciliationDto::from).collect()
}

/// Return the current connection status: what the running configuration was
/// actually built from, not what the environment currently says.
///
/// REQ-e4ah9h / REQ-bvx4nh / REQ-645jq9. Nothing here reads the environment.
/// The previous handler re-derived the data directory from `ODS_DATA_DIR` /
/// `app_data_dir()`, duplicating `AppState::init`'s logic, so the directory it
/// reported could disagree with the one the store was opened in; and it read
/// `ODS_CHAIN_WS` / `ODS_CONTRACT_H160` directly, so it could report an
/// endpoint beside `chain_configured: false`.
#[tauri::command]
pub async fn connection_status<R: Runtime>(
    state: State<'_, AppState>,
    app_handle: AppHandle<R>,
) -> Result<ConnectionStatus, String> {
    // The handle is no longer read for anything. It stays in the signature
    // because it is what makes this command generic over the runtime, and a
    // command that is not generic over the runtime cannot be driven by
    // `MockRuntime` — i.e. cannot be tested across the IPC boundary at all.
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

/// One iteration of the receiver loop, as outcomes.
///
/// A `Vec` rather than a single outcome because a terminal error produces two:
/// the failure itself (REQ-affyf5) and then the loop's exit (REQ-jfxah3). Every
/// other iteration produces exactly one. The failure case lives in
/// `events::outcomes_for_receive_error`, where `tests/receiver_events.rs` can
/// gate it without an AppHandle; so does the outcome of a received message
/// (`events::outcome_for_arrival`), and this function stays a straight
/// translation.
///
/// `org_io` is the shared handle (LLR-5wb4vd); no app-side lock is taken, so
/// the wait in `next_received` blocks no command.
async fn next_outcomes(org_io: &OrgIo) -> Vec<events::ReceiverOutcome> {
    let (arrival, org_id) = match org_io.next_received(&mut OsRng).await {
        // The acknowledgements a Removed or Revoked outcome carries are
        // dropped until S3b-io sends them, which leaves the entry
        // unacknowledged — an outcome ruling C allows.
        Ok(Received::Committed(outcome)) => (events::Arrival::Committed(outcome.kind), outcome.org_id),
        // A revoked Device's acknowledgement verified; the UI is told nothing.
        Ok(Received::Acknowledged(verified)) => (events::Arrival::Acknowledged, verified.into_inner().org_id),
        // RC-3rddh7 / REQ-kn5rtx / REQ-jfxah3: classified by the error's
        // variant, the loop's exit included (LLR-zw6jqy).
        Err(error) => return events::outcomes_for_receive_error(&error),
    };
    // The record as read back after the message: epoch and root, or nothing.
    let record = org_io
        .organisation(org_id)
        .await
        .map(|organisation| (organisation.epoch.get(), hex::encode(organisation.root_hash.as_bytes())));
    vec![events::outcome_for_arrival(arrival, hex::encode(org_id.as_bytes()), record)]
}

/// Spawn a background tokio task that loops org-io's `next_received` and
/// emits Tauri events.  The task runs until the app is closed or the
/// endpoint returns a permanent error (endpoint closed).
///
/// REQ-6hgm8r: only one loop runs at a time. REQ-3hfggn: when the loop ends —
/// by break, by return, or by panic — the slot is released and a later call may
/// start a new one. The previous code set the flag before spawning and never
/// cleared it, so an exited loop kept the slot for the life of the process and
/// the app could not recover without a restart (HAZ-cfp4jb).
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
        // org-io receives only on the endpoints it was told to start; a bind
        // that fails (`ReceiveError::Terminal`) ends the loop before it
        // begins.
        if let Err(error) = org_io.start_receiving().await {
            for outcome in &events::outcomes_for_receive_error(&error) {
                for emission in events::emissions_for(outcome) {
                    // Events are the only channel to the webview, so a
                    // failed emit has nowhere to be reported; this task
                    // ends right after either way.
                    let _ = app_handle_clone.emit(emission.name, emission.payload);
                }
            }
            return;
        }
        loop {
            let outcomes = next_outcomes(&org_io).await;
            let stopping = outcomes
                .iter()
                .any(|o| matches!(o, events::ReceiverOutcome::Stopped { .. }));
            for outcome in &outcomes {
                for e in events::emissions_for(outcome) {
                    let _ = app_handle_clone.emit(e.name, e.payload);
                }
            }
            if stopping {
                break;
            }
        }
    });

    Ok(())
}

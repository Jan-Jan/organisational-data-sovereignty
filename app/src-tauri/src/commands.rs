//! Tauri command handlers for the ODS PoC app.
//!
//! Each command locks `AppState.service` (tokio async Mutex), calls the
//! matching `OrgService` method, and maps `OrgNodeError` → `String` for the
//! Tauri `Result<T, String>` convention.
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

use rand::rngs::OsRng;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

use org_node::service::SelfDeleteOutcome;
use org_node::store::{OrgRecord, PersonaDetails, PersonaRecord};
use org_node::{MemberId, OrgNodeError, OrgSecret, PersonaId};

use crate::invitation::{self, Invite};
use crate::parsing::parse_org_id;
use crate::state::{AppState, ConnectionStatus};
use crate::{events, policy};

// ---------------------------------------------------------------------------
// Serialisable DTOs
// ---------------------------------------------------------------------------

/// A serialisable view of a `PersonaRecord` (no key material).
#[derive(Debug, Serialize)]
pub struct PersonaDto {
    pub persona_id: String,
    pub org_id: Option<String>,
    pub handle: String,
    pub name: String,
    pub surname: String,
    pub status: String,
}

impl From<&PersonaRecord> for PersonaDto {
    fn from(p: &PersonaRecord) -> Self {
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

/// A serialisable view of an `OrgRecord`.
#[derive(Debug, Serialize)]
pub struct OrgDto {
    pub org_id: String,
    pub epoch: u64,
    pub root_hash: String,
    pub member_count: usize,
}

impl From<&OrgRecord> for OrgDto {
    fn from(o: &OrgRecord) -> Self {
        Self {
            org_id: hex::encode(o.org_id.as_bytes()),
            epoch: o.epoch.get(),
            root_hash: hex::encode(o.root_hash.as_bytes()),
            member_count: o.trie_members.len(),
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
    let mut svc = state.service.lock().await;
    svc.create_persona(&mut OsRng, handle, name, surname)
        .map(|id| id.as_str().to_string())
        .map_err(|e| e.to_string())
}

/// Create an organisation: org-node builds the genesis update, the app writes
/// it to the chain through on-chain-client, then org-node commits it
/// (REQ-nfr3n2, `crate::submit::found_organisation`).
/// Returns the org_id (40 hex chars).
#[tauri::command]
pub async fn create_organisation(
    state: State<'_, AppState>,
    persona_id: String,
) -> Result<String, String> {
    let mut svc = state.service.lock().await;
    let org_id =
        crate::submit::found_organisation(&mut svc, &*state.writer, &mut OsRng, &PersonaId::new(persona_id))
            .await?;
    Ok(hex::encode(org_id.as_bytes()))
}

/// Issue an Invite to `org_id` (LLR-9sraks): its Blob, with a fresh invite
/// identifier kept as outstanding.
#[tauri::command]
pub async fn export_invite(
    state: State<'_, AppState>,
    org_id: String,
    org_name: String,
    invitee_name: String,
) -> Result<String, String> {
    let oid = parse_org_id(&org_id)?;
    let svc = state.service.lock().await;
    let mut outstanding = state.outstanding.lock().await;
    invitation::issue_invite(&svc, &mut outstanding, &mut OsRng, oid, &org_name, &invitee_name)
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

/// Parse an Invite Blob (LLR-b7wgpf); stores nothing.
#[tauri::command]
pub async fn import_invite(blob: String) -> Result<InviteDto, String> {
    let invite = Invite::parse(&blob)?;
    Ok(InviteDto {
        org_name: invite.org_name,
        org_id: hex::encode(invite.org_id.as_bytes()),
        invitee_name: invite.invitee_name,
        inviter_device_keys: invite.inviter_device_keys.iter().map(|k| hex::encode(k.as_bytes())).collect(),
        invite_id: hex::encode(invite.invite_id.as_bytes()),
    })
}

/// The reply to an Invite from `persona_id`, only once the user confirmed
/// (LLR-w4mhd4).
#[tauri::command]
pub async fn produce_invite_reply(
    state: State<'_, AppState>,
    invite_blob: String,
    persona_id: String,
    confirmed: bool,
) -> Result<String, String> {
    let mut svc = state.service.lock().await;
    invitation::produce_reply(&mut svc, &mut OsRng, &invite_blob, &PersonaId::new(persona_id), confirmed)
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

/// Parse an Invite reply Blob naming an outstanding Invite (LLR-pmus9f);
/// stores nothing.
#[tauri::command]
pub async fn import_invite_reply(state: State<'_, AppState>, blob: String) -> Result<InviteReplyDto, String> {
    let reply = invitation::check_reply(&*state.outstanding.lock().await, &blob)?;
    Ok(InviteReplyDto {
        org_id: hex::encode(reply.org_id.as_bytes()),
        handle: reply.handle.to_string(),
        name: reply.name.to_string(),
        surname: reply.surname.to_string(),
        member_key: hex::encode(reply.member_key.as_bytes()),
        device_key: hex::encode(reply.device_key.as_bytes()),
    })
}

/// Admit the person an Invite reply names (LLR-gha5f6): org-node builds the
/// admission, the app writes it to the chain, then org-node commits it and
/// sends it to the reply's device under the reply's invite id (REQ-nfr3n2).
/// The target is the Organisation the reply's outstanding pair names;
/// `org_id` is the operator's selection and is refused unless it is that one.
///
/// The org id, then the Organisation secret (LLR-8krgzj), then the peer
/// address (LLR-ctrfz4, as `revoke_member` reads it) are parsed before the
/// reply is. Returns the new member_id as 64 hex chars.
#[tauri::command]
pub async fn admit_member(
    state: State<'_, AppState>,
    org_id: String,
    reply_blob: String,
    peer_addr_blob: String,
    org_secret_hex: Option<String>,
) -> Result<String, String> {
    let oid = parse_org_id(&org_id)?;
    let org_secret: Option<OrgSecret> = match org_secret_hex {
        Some(hex_str) => {
            let bytes = hex::decode(hex_str.trim_start_matches("0x")).map_err(|e| format!("org_secret hex: {e}"))?;
            let arr: [u8; 32] = bytes.try_into().map_err(|_| "org_secret must be 32 bytes".to_string())?;
            Some(OrgSecret::from(arr))
        }
        None => None,
    };
    let peer_addr = parse_peer_addr(&peer_addr_blob)?;
    let mut svc = state.service.lock().await;
    let mut outstanding = state.outstanding.lock().await;
    let member_id = invitation::admit_reply(
        &mut svc,
        &*state.writer,
        &mut outstanding,
        &mut OsRng,
        oid,
        &reply_blob,
        peer_addr,
        org_secret,
    )
    .await?;
    Ok(hex::encode(member_id.as_bytes()))
}

/// A peer address as the commands take it: blank (only whitespace) is
/// absent; otherwise hex of the postcard bytes of an iroh `EndpointAddr`,
/// needed for Loopback dialling only.
fn parse_peer_addr(peer_addr_blob: &str) -> Result<Option<iroh::EndpointAddr>, String> {
    let trimmed = peer_addr_blob.trim().trim_start_matches("0x");
    if trimmed.is_empty() {
        return Ok(None);
    }
    let addr_bytes = hex::decode(trimmed).map_err(|e| format!("peer_addr_blob hex: {e}"))?;
    postcard::from_bytes(&addr_bytes).map(Some).map_err(|e| format!("peer_addr decode: {e}"))
}

/// Revoke a member by member_id (64 hex chars). `peer_addr_blob` (hex-encoded
/// postcard bytes of the iroh EndpointAddr) is OPTIONAL: leave it empty in
/// Networked mode, where the revoked member is reached by EndpointId (derived
/// from its device key in the trie) via iroh discovery. It is only needed for
/// same-machine Loopback dialing.
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

    let peer_addr = parse_peer_addr(&peer_addr_blob)?;

    // REQ-nfr3n2: org-node builds the removal, the app writes it to the
    // chain, and only then does org-node commit it and send it to the revoked
    // member's device (`crate::submit::submit_commit_send`).
    let mut svc = state.service.lock().await;
    let member = MemberId::new(member_id);
    let recipient = svc
        .list_orgs()
        .iter()
        .find(|o| o.org_id == oid)
        .ok_or_else(|| OrgNodeError::OrgNotOnChain.to_string())?
        .trie_members
        .iter()
        .find(|m| m.id == member)
        .and_then(|m| m.device_keys.first().copied())
        .ok_or("member_id names no member of this organisation")?;
    let update = svc.revoke_member(&mut OsRng, oid, member).map_err(|e| e.to_string())?;
    crate::submit::submit_commit_send(&mut svc, &*state.writer, &mut OsRng, &update, recipient, peer_addr, None, None)
        .await
        .map(|_| ())
}

/// List all local personas (no key material returned).
#[tauri::command]
pub async fn list_personas(state: State<'_, AppState>) -> Result<Vec<PersonaDto>, String> {
    let svc = state.service.lock().await;
    Ok(svc.list_personas().iter().map(PersonaDto::from).collect())
}

/// List all local org records.
#[tauri::command]
pub async fn list_orgs(state: State<'_, AppState>) -> Result<Vec<OrgDto>, String> {
    let svc = state.service.lock().await;
    Ok(svc.list_orgs().iter().map(OrgDto::from).collect())
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
// start_receiver: spawns a background task looping receive_and_self_delete_if_revoked
// ---------------------------------------------------------------------------

/// One iteration of the receiver loop, as outcomes.
///
/// A `Vec` rather than a single outcome because a terminal error produces two:
/// the failure itself (REQ-affyf5) and then the loop's exit (REQ-jfxah3). Every
/// other iteration produces exactly one. The failure case lives in
/// `events::outcomes_for_receive_error`, where `tests/receiver_events.rs` can
/// gate it without an AppHandle.
///
/// This is where the three defects the register found are fixed, and each is
/// a return value rather than an emission, so `tests/receiver_events.rs` gates
/// the announcement and this function stays a straight translation.
async fn next_outcomes<R: Runtime>(app: &AppHandle<R>) -> Vec<events::ReceiverOutcome> {
    // Re-lock the service each iteration so other commands can proceed between
    // messages (the lock is held only for the duration of one receive+verify).
    let result = {
        let state: State<'_, AppState> = app.state::<AppState>();
        let mut svc = state.service.lock().await;
        svc.receive_and_self_delete_if_revoked(&mut OsRng).await
    };

    match result {
        // REQ-tw4cb5: SelfDeleted alone. The previous code also emitted an
        // `epoch-changed` carrying a literal 0 — and 0 is genesis, a reachable
        // and meaningful epoch, so the UI could not tell "deleted" from "reset
        // to genesis" (HAZ-5ha5vv).
        Ok(SelfDeleteOutcome::SelfDeleted { org_id }) => {
            vec![events::ReceiverOutcome::SelfDeleted {
                org_id: hex::encode(org_id.as_bytes()),
            }]
        }

        Ok(SelfDeleteOutcome::UpdatedNotRevoked { org_id }) => {
            // Re-read the org record to get the current epoch + root.
            let record = {
                let state: State<'_, AppState> = app.state::<AppState>();
                let svc = state.service.lock().await;
                svc.list_orgs()
                    .iter()
                    .find(|o| o.org_id == org_id)
                    .map(|o| (o.epoch.get(), hex::encode(o.root_hash.as_bytes())))
            };
            let org_id = hex::encode(org_id.as_bytes());
            match record {
                Some((epoch, root)) => {
                    vec![events::ReceiverOutcome::Updated { org_id, epoch, root }]
                }
                // REQ-2k7ys4 / REQ-dp95pv. The previous code was
                // `.unwrap_or((0, String::new()))`, which announced a verified
                // update at genesis with an empty root — indistinguishable from
                // a real one.
                None => vec![events::ReceiverOutcome::RecordUnreadable { org_id }],
            }
        }

        // RC-3rddh7 / REQ-kn5rtx / REQ-jfxah3: classify by the error's TYPE, and
        // append the loop's exit when the message is terminal. The previous code
        // routed every `Err` into `VerifyFailed`, so a transport hiccup or a
        // chain read failure was displayed as a root mismatch — a fresh instance
        // of HAZ-9fmhm4.
        Err(e) => events::outcomes_for_receive_error(&e),
    }
}

/// Spawn a background tokio task that loops `receive_and_self_delete_if_revoked`
/// and emits Tauri events.  The task runs until the app is closed or the
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
    tokio::spawn(async move {
        // REQ-3hfggn: the guard is moved in, so the slot is released when this
        // task ends — by break, by return, or by panic.
        let _guard = guard;
        loop {
            let outcomes = next_outcomes(&app_handle_clone).await;
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

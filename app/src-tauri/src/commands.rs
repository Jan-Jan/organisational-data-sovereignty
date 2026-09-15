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

use org_node::blobs::JoinRequest;
use org_node::service::SelfDeleteOutcome;
use org_node::store::{OrgRecord, PersonaRecord};

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
            persona_id: p.persona_id.clone(),
            org_id: p.org_id.map(|id| hex::encode(id.as_bytes())),
            handle: p.handle.clone(),
            name: p.name.clone(),
            surname: p.surname.clone(),
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
            epoch: o.epoch,
            root_hash: hex::encode(o.root_hash),
            member_count: o.trie_members.len(),
        }
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Create a new persona (no chain interaction).
#[tauri::command]
pub async fn create_persona(
    state: State<'_, AppState>,
    handle: String,
    name: String,
    surname: String,
) -> Result<String, String> {
    let mut svc = state.service.lock().await;
    svc.create_persona(&mut OsRng, &handle, &name, &surname)
        .map_err(|e| e.to_string())
}

/// Create an organisation: build genesis trie + submit to chain.
/// Returns the org_id (40 hex chars).
#[tauri::command]
pub async fn create_organisation(
    state: State<'_, AppState>,
    persona_id: String,
) -> Result<String, String> {
    let mut svc = state.service.lock().await;
    let org_id = svc
        .create_organisation(&mut OsRng, &persona_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(hex::encode(org_id.as_bytes()))
}

/// Export an invite blob for the given org.
#[tauri::command]
pub async fn export_invite(
    state: State<'_, AppState>,
    org_id: String,
) -> Result<String, String> {
    let oid = parse_org_id(&org_id)?;
    let svc = state.service.lock().await;
    svc.export_invite(oid).map_err(|e| e.to_string())
}

/// Import an invite blob; returns the org_id it's for.
#[tauri::command]
pub async fn import_invite(
    state: State<'_, AppState>,
    blob: String,
) -> Result<String, String> {
    let mut svc = state.service.lock().await;
    let inv = svc.import_invite(&mut OsRng, &blob).map_err(|e| e.to_string())?;
    Ok(hex::encode(inv.org_id.as_bytes()))
}

/// Export a join-request blob for the given persona.
#[tauri::command]
pub async fn export_join_request(
    state: State<'_, AppState>,
    persona_id: String,
) -> Result<String, String> {
    let svc = state.service.lock().await;
    svc.export_join_request(&persona_id).map_err(|e| e.to_string())
}

/// Decode and return the fields of a join-request blob (no persistence).
#[tauri::command]
pub async fn import_join_request(blob: String) -> Result<JoinRequestDto, String> {
    let jr = decode_join_request(&blob)?;
    Ok(JoinRequestDto {
        handle: jr.handle,
        name: jr.name,
        surname: jr.surname,
        member_key: hex::encode(jr.member_key),
        device_key: hex::encode(jr.device_key),
        has_node_addr: !jr.node_addr.is_empty(),
        node_addr_blob: hex::encode(&jr.node_addr),
    })
}

fn decode_join_request(blob: &str) -> Result<JoinRequest, String> {
    org_node::service::OrgService::import_join_request(blob).map_err(|e| e.to_string())
}

/// DTO for an import_join_request response.
#[derive(Debug, Serialize)]
pub struct JoinRequestDto {
    pub handle: String,
    pub name: String,
    pub surname: String,
    pub member_key: String,
    pub device_key: String,
    pub has_node_addr: bool,
    /// The raw node_addr bytes as hex — pass back to admit_member.
    pub node_addr_blob: String,
}

/// Admit a new member from a join-request blob.
///
/// The blob's `node_addr` is OPTIONAL: in `Networked` transport the joiner is
/// reached by its `EndpointId` (its device key) via iroh relay/DNS discovery, so
/// the blob carries no embedded address and the service ignores `peer_addr`. We
/// only need a full `EndpointAddr` for `Loopback` (same-machine) dialing.
///
/// Returns the new member_id as 64 hex chars.
#[tauri::command]
pub async fn admit_member(
    state: State<'_, AppState>,
    org_id: String,
    join_request_blob: String,
    org_secret_hex: Option<String>,
) -> Result<String, String> {
    use iroh::EndpointAddr;

    let oid = parse_org_id(&org_id)?;
    let jr = decode_join_request(&join_request_blob)?;

    // Resolve the joiner's iroh address. In Networked mode the blob carries no
    // node_addr (empty) — the peer is reached by EndpointId (its device key) via
    // relay/DNS discovery, and the service ignores peer_addr — so build an
    // id-only EndpointAddr from the device key. In Loopback the blob carries the
    // full EndpointAddr to dial.
    let peer_addr: EndpointAddr = if jr.node_addr.is_empty() {
        iroh::EndpointId::from_bytes(&jr.device_key)
            .map_err(|e| format!("device_key is not a valid iroh EndpointId: {e}"))?
            .into()
    } else {
        postcard::from_bytes(&jr.node_addr)
            .map_err(|e| format!("node_addr decode: {e}"))?
    };

    let org_secret: Option<[u8; 32]> = match org_secret_hex {
        Some(hex_str) => {
            let bytes = hex::decode(hex_str.trim_start_matches("0x"))
                .map_err(|e| format!("org_secret hex: {e}"))?;
            if bytes.len() != 32 {
                return Err("org_secret must be 32 bytes".into());
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            Some(arr)
        }
        None => None,
    };

    let mut svc = state.service.lock().await;
    let member_id = svc
        .admit_member(&mut OsRng, oid, &jr, peer_addr, org_secret)
        .await
        .map_err(|e| e.to_string())?;
    Ok(hex::encode(member_id))
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
    use iroh::EndpointAddr;

    let oid = parse_org_id(&org_id)?;
    let member_bytes = hex::decode(member_id_hex.strip_prefix("0x").unwrap_or(&member_id_hex))
        .map_err(|e| format!("member_id hex: {e}"))?;
    if member_bytes.len() != 32 {
        return Err("member_id must be 32 bytes (64 hex chars)".into());
    }
    let mut member_id = [0u8; 32];
    member_id.copy_from_slice(&member_bytes);

    // peer_addr is OPTIONAL: empty in Networked mode (the service reaches the
    // revoked member by EndpointId, derived from its device key in the trie).
    // Only Loopback needs the full EndpointAddr.
    let trimmed = peer_addr_blob.trim().trim_start_matches("0x");
    let peer_addr: Option<EndpointAddr> = if trimmed.is_empty() {
        None
    } else {
        let addr_bytes =
            hex::decode(trimmed).map_err(|e| format!("peer_addr_blob hex: {e}"))?;
        Some(postcard::from_bytes(&addr_bytes).map_err(|e| format!("peer_addr decode: {e}"))?)
    };

    let mut svc = state.service.lock().await;
    svc.revoke_member(&mut OsRng, oid, member_id, peer_addr)
        .await
        .map_err(|e| e.to_string())
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
                    .map(|o| (o.epoch, hex::encode(o.root_hash)))
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

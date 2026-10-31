//! ODS PoC Tauri app library.
//!
//! Wires `AppState` (the org-io handle + chain config) into the Tauri builder,
//! registers all Tauri commands, and exposes the `run()` entry point.

pub mod commands;
pub mod entry;
pub mod events;
pub mod follow_up;
pub mod parsing;
pub mod periodic;
pub mod policy;
pub mod state;

use std::sync::Arc;

use tauri::{Emitter, Manager};

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Resolve the data dir from Tauri's path resolver (may return an
            // error on platforms without a standard app-data location).
            let tauri_data_dir = app
                .path()
                .app_data_dir()
                .ok();

            let app_state = AppState::init(tauri_data_dir)
                .unwrap_or_else(|e| panic!("AppState init failed: {e}"));
            // SDD-tgu7dp: one periodic verification, started once, after a
            // successful init, with the interval init decided. Tauri's spawn,
            // not `tokio::spawn`: this closure runs outside a tokio runtime
            // context, where `tokio::spawn` panics.
            let org_io = Arc::clone(&app_state.org_io);
            let interval = app_state.verification_interval;
            let app_handle = app.handle().clone();
            app.manage(app_state);
            tauri::async_runtime::spawn(periodic::run_periodic_verification(
                org_io,
                interval,
                move |emission: events::Emission| {
                    // Events are the only channel to the webview; a failed
                    // emit has nowhere to be reported.
                    let _ = app_handle.emit(emission.name, emission.payload);
                },
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::propose_person,
            commands::propose_founding_organisation,
            commands::issue_invite,
            commands::read_invite,
            commands::propose_member,
            commands::request_admission,
            commands::propose_admission,
            commands::propose_revocation,
            commands::cancel_invite,
            commands::propose_updated_organisation,
            commands::write_organisation_to_chain,
            commands::verify_organisation,
            commands::list_admission_requests,
            commands::list_personas,
            commands::list_orgs,
            commands::connection_status,
            commands::start_receiver,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

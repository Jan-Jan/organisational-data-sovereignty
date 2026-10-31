#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The write and the verification it asks (LLR-rhc555, LLR-k4rhbs), the
//! verification's entries, sends and requests (LLR-jf63e4, LLR-es49wp,
//! LLR-nkg8sb, LLR-n64wvr, LLR-rywe8f) and the list's statuses
//! (LLR-g8wfy9), over org-io's substitute chain (`test_support::FakeChain`).
//!
//! Each test calls the interface its item names: the commands
//! `write_organisation_to_chain`, `verify_organisation` and `list_orgs`
//! (called as functions on a mock app's managed `AppState`; their argument
//! names over IPC are `ipc.rs`'s), `verify_and_send` (LLR-es49wp,
//! LLR-nkg8sb), and `entry_for`, `device_reports` and `OrgDto::new`, which
//! have their own items (LLR-73by2q, LLR-cugcu6, LLR-q96vdr) for the cases
//! the substitute chain cannot produce.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use ods_poc_lib::commands;
use ods_poc_lib::commands::OrgDto;
use ods_poc_lib::entry::{device_reports, entry_for, EntryOrg, EntryResultDto, VerificationEntryDto};
use ods_poc_lib::policy::TransportModeName;
use ods_poc_lib::state::AppState;
use org_io::node::{DevicePublicKey, OrgId};
use org_io::node::{Epoch, Handle, InviteId, MemberId, Name, Surname};
use org_io::store_file::{StoreFileError, StoreFileStep};
use org_io::test_support::FakeChain;
use org_io::verify::{DiscardedChangeSummary, Inconsistency, UnverifiedReason};
use org_io::{HeldTarget, OrgIo, StoreFailure, WriteReport};
use org_io::{Delivery, KeychainError, SendError, SendReport, Verified, WriteOutcome};
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::Manager;

/// A mock application managing a real `AppState` over `chain` in Loopback
/// mode, its store in a fresh directory: the commands are called on it.
struct App {
    _dir: tempfile::TempDir,
    app: tauri::App<MockRuntime>,
}

fn app_over(chain: &FakeChain) -> App {
    let dir = tempfile::tempdir().expect("tempdir");
    let state = AppState::for_test_over(dir.path().to_path_buf(), chain, TransportModeName::Loopback)
        .expect("AppState::for_test_over");
    let app = mock_builder().manage(state).build(mock_context(noop_assets())).expect("build mock app");
    App { _dir: dir, app }
}

impl App {
    /// The shared org-io handle the commands use.
    fn io(&self) -> Arc<OrgIo> {
        Arc::clone(&self.app.state::<AppState>().org_io)
    }

    /// `write_organisation_to_chain` for a founding, by its founder's key.
    async fn write_founding(&self, founder: DevicePublicKey) -> Result<serde_json::Value, String> {
        let written =
            commands::write_organisation_to_chain(self.app.state(), None, Some(hex::encode(founder.as_bytes()))).await;
        written.map(|report| wire(&report))
    }

    /// `write_organisation_to_chain` for a held Organisation, by its id.
    async fn write_org(&self, org: OrgId) -> Result<serde_json::Value, String> {
        let written = commands::write_organisation_to_chain(self.app.state(), Some(hex::encode(org.as_bytes())), None).await;
        written.map(|report| wire(&report))
    }

    /// `verify_organisation`, for one Organisation or (`None`) every target held.
    async fn verify(&self, org: Option<OrgId>) -> Vec<serde_json::Value> {
        let entries = commands::verify_organisation(self.app.state(), org.map(|org| hex::encode(org.as_bytes())))
            .await
            .expect("verify_organisation");
        entries.iter().map(wire).collect()
    }

    /// `list_orgs`.
    async fn list_orgs(&self) -> Vec<serde_json::Value> {
        commands::list_orgs(self.app.state()).await.expect("list_orgs").iter().map(wire).collect()
    }
}

/// A provisional founding "Acme" by "Alice Smith"; the founder's Device key.
async fn founding(io: &OrgIo) -> DevicePublicKey {
    io.propose_founding_organisation("Alice", "Smith", "alice", "Acme").await.expect("proposed").founder
}

/// The Organisation the founder's Persona now holds.
async fn org_of(io: &OrgIo, founder: DevicePublicKey) -> OrgId {
    io.personas()
        .await
        .into_iter()
        .find(|persona| persona.device_key == founder)
        .and_then(|persona| persona.org_id)
        .expect("the founding's Persona holds its org id")
}

/// A provisional founding whose next write will not return within its
/// bound: org-io reports a hanging proxy creation as refused (nothing is
/// stored), so the proxy is stored first by a write whose `found_at` is
/// refused, and the chain then hangs every write, the retry's `found_at`
/// among them (as org-io's own founding-unknown tests do).
async fn founding_whose_write_hangs(chain: &FakeChain, io: &OrgIo) -> DevicePublicKey {
    let founder = founding(io).await;
    chain.found_at_failing.store(true, Ordering::SeqCst);
    let first = io.write_organisation_to_chain(HeldTarget::Founding(founder)).await;
    assert!(matches!(first, Ok(WriteReport::Refused { .. })), "the proxy stored, the genesis not: {first:?}");
    chain.found_at_failing.store(false, Ordering::SeqCst);
    chain.write_hanging.store(true, Ordering::SeqCst);
    founder
}

fn wire(dto: &impl serde::Serialize) -> serde_json::Value {
    serde_json::to_value(dto).expect("serialises")
}

/// An Organisation founded and adopted over `chain`, with one admission
/// recorded and the next epoch proposed: Bob's Device asked over Loopback,
/// the inviter receiving. The invitee reads the same chain slots, so it can
/// verify what the inviter writes. The invitee's handle and its directory
/// are held so its endpoint stays up.
struct Admitted {
    inviter: App,
    org: OrgId,
    member_device: DevicePublicKey,
    invitee: Arc<OrgIo>,
    /// How many organisation requests Bob's Device has answered.
    invitee_answered: Arc<AtomicUsize>,
    _invitee_dir: tempfile::TempDir,
}

/// Start `io` receiving, its arrivals drained in the background until its
/// endpoint ends; the count of organisation requests it answered.
async fn receive(io: &Arc<OrgIo>) -> Arc<AtomicUsize> {
    io.start_receiving().await.expect("receiving");
    let receiving = Arc::clone(io);
    let answered = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&answered);
    tokio::spawn(async move {
        // Only the arrivals' effect on org-io's store is what these tests
        // read; a refused arrival is the receiver tests' (receiver_events.rs)
        // to check, so the drain goes on past it and stops only at the end.
        loop {
            match receiving.next_received().await {
                Err(org_io::ReceiveError::Terminal(_)) => break,
                Ok(org_io::Received::OrganisationRequestAnswered { .. }) => {
                    counted.fetch_add(1, Ordering::SeqCst);
                }
                _ => {}
            }
        }
    });
    answered
}

async fn admitted(chain: &FakeChain) -> Admitted {
    let inviter_app = app_over(chain);
    let inviter = inviter_app.io();
    let founder = founding(&inviter).await;
    let founded = inviter_app.write_founding(founder).await.expect("written");
    assert_eq!(founded["verification"]["result"], "adopted", "{founded}");
    let org = org_of(&inviter, founder).await;
    receive(&inviter).await;
    let blob = inviter.issue_invite(org, "Bob Builder").await.expect("issued");
    let invitee_dir = tempfile::tempdir().expect("tempdir");
    let invitee = OrgIo::open_for_test(invitee_dir.path(), org_io::transport::TransportMode::Loopback, &FakeChain::sharing_slots_with(chain))
        .expect("open");
    let member = invitee.propose_member(&blob, "Bob", "Builder", &["bob"]).await.expect("proposed");
    let invitee = Arc::new(invitee);
    // The invitee receives too, so a send or a request to Bob's Device is
    // answered at once rather than waiting out org-io's 30-second
    // connection limit.
    let invitee_answered = receive(&invitee).await;
    let outcome = invitee.request_admission(org).await.expect("requested");
    assert!(matches!(outcome, Delivery::Delivered), "{outcome:?}");
    let mut kept = inviter.admission_requests().await;
    for _ in 0..100 {
        if !kept.is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
        kept = inviter.admission_requests().await;
    }
    assert_eq!(kept.len(), 1, "the inviter keeps Bob's request");
    inviter.propose_admission(org, kept[0].invite_id, None).await.expect("recorded");
    inviter.propose_updated_organisation(org).await.expect("proposed");
    Admitted { inviter: inviter_app, org, member_device: member.member_device, invitee, invitee_answered, _invitee_dir: invitee_dir }
}

// Every verification path — the command (on mount and on the button), the
// write's, the receiver loop's and the periodic run's — verifies through the
// state's one org-io handle, so the handle's ahead limit is the one each
// path gives. It is twice the state's verification interval, for the
// default and for a configured one.
// verifies: LLR-hgrez9
#[test]
fn the_states_handle_holds_twice_the_verification_interval_as_its_ahead_limit() {
    let chain = FakeChain::new();
    for configured in [None, Some(Duration::from_secs(90))] {
        let dir = tempfile::tempdir().expect("tempdir");
        let state = match configured {
            None => AppState::for_test_over(dir.path().to_path_buf(), &chain, TransportModeName::Loopback),
            Some(interval) => {
                AppState::for_test_over_every(dir.path().to_path_buf(), &chain, TransportModeName::Loopback, interval)
            }
        }
        .expect("state");
        let interval = configured.unwrap_or(Duration::from_secs(600));
        assert_eq!(state.verification_interval, interval, "the periodic run's interval");
        assert_eq!(state.org_io.ahead_limit_for_test(), Some(interval * 2), "{interval:?}: the handle's ahead limit");
    }
}

// `init` builds the settings it hands `OrgIo::connect` with the one
// function the test constructors take their handle's ahead limit from, and
// that function is given the interval undoubled: the settings' ahead limit
// is twice the verification interval, for the least, the default and the
// greatest interval the resolver accepts (doubled without overflow).
// verifies: LLR-hgrez9
#[test]
fn the_chain_settings_init_hands_connect_hold_twice_the_verification_interval_as_their_ahead_limit() {
    const CONTRACT: &str = "0x00112233445566778899aabbccddeeff00112233";
    for seconds in [1u64, 90, 600, 4_294_967_295] {
        let interval = Duration::from_secs(seconds);
        let (settings, endpoint) =
            AppState::chain_settings_for_test(Some("ws://chain.test"), Some(CONTRACT), None, interval)
                .expect("both variables set and well-formed");
        assert_eq!(settings.ahead_limit, Duration::from_secs(2 * seconds), "{seconds} s: the settings' ahead limit");
        assert_eq!(settings.ws_url, "ws://chain.test", "{seconds} s: the endpoint is the variable's");
        assert_eq!(endpoint.contract_h160, CONTRACT, "{seconds} s: the endpoint records the address as given");
    }
}

// Normal, boundary and abnormal: `startup_chain`, given the chain variables
// and the verification interval as values, decides the interval and builds
// the chain settings. The settings' ahead limit is twice the interval
// decided: unset (the default 600 s), the least and the greatest the
// resolver accepts. A refused interval is refused; unset chain variables
// leave no settings, the interval still decided.
// verifies: LLR-hgrez9
#[test]
fn startup_chain_builds_the_settings_with_twice_the_interval_it_decides() {
    use std::ffi::OsStr;
    const CONTRACT: &str = "0x00112233445566778899aabbccddeeff00112233";
    for (raw, seconds) in [(None, 600u64), (Some("1"), 1), (Some("4294967295"), 4_294_967_295)] {
        let startup = AppState::startup_chain_for_test(Some("ws://chain.test"), Some(CONTRACT), None, raw.map(OsStr::new))
            .expect("the interval is accepted");
        assert_eq!(startup.verification_interval, Duration::from_secs(seconds), "{raw:?}: the interval decided");
        let (settings, endpoint) = startup.chain.expect("both chain variables set and well-formed");
        assert_eq!(settings.ahead_limit, Duration::from_secs(2 * seconds), "{raw:?}: the settings' ahead limit");
        assert_eq!(endpoint.contract_h160, CONTRACT, "{raw:?}: the endpoint is the variables'");
    }
    assert!(
        AppState::startup_chain_for_test(Some("ws://chain.test"), Some(CONTRACT), None, Some(OsStr::new("0"))).is_err(),
        "a refused interval refuses startup"
    );
    let unconfigured = AppState::startup_chain_for_test(None, None, None, None).expect("the default interval");
    assert_eq!(unconfigured.verification_interval, Duration::from_secs(600));
    assert_eq!(unconfigured.chain.err().as_deref(), Some("ODS_CHAIN_WS not set"), "the chain unconfigured");
}

// verifies: LLR-rhc555
#[tokio::test(start_paused = true)]
async fn write_organisation_to_chain_tags_included_refused_and_unknown() {
    // Included, with the epoch written.
    let chain = FakeChain::new();
    let app = app_over(&chain);
    let founder = founding(&app.io()).await;
    let included = app.write_founding(founder).await.expect("a report");
    assert_eq!(included["result"], "included", "{included}");
    assert_eq!(included["epoch"], 1, "{included}");

    // Refused, with org-io's reason unchanged.
    let chain = FakeChain::new();
    chain.write_failing.store(true, Ordering::SeqCst);
    let app = app_over(&chain);
    let founder = founding(&app.io()).await;
    let refused = app.write_founding(founder).await.expect("a report");
    assert_eq!(refused["result"], "refused", "{refused}");
    let mut keys: Vec<&str> = refused.as_object().expect("an object").keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["reason", "result"]);
    match app.io().write_organisation_to_chain(HeldTarget::Founding(founder)).await {
        Ok(WriteReport::Refused { failure }) => assert_eq!(refused["reason"], serde_json::json!(failure.to_string())),
        other => panic!("org-io refuses the same write again: {other:?}"),
    }

    // Unknown: the write does not return within the 90-second bound.
    let chain = FakeChain::new();
    let app = app_over(&chain);
    let founder = founding_whose_write_hangs(&chain, &app.io()).await;
    let unknown = app.write_founding(founder).await.expect("a report");
    assert_eq!(unknown["result"], "unknown", "{unknown}");
    let reason = unknown["reason"].as_str().expect("a reason");
    assert!(reason.contains("90"), "org-io's reason names the bound: {reason}");
}

// org-io's refusal before any chain call is the command's error text, as org-io displays it:
// an Organisation this node holds nothing for is refused with `NotHeld`'s text, not its
// Debug name.
// verifies: LLR-rhc555
#[tokio::test(start_paused = true)]
async fn a_refusal_before_the_chain_is_org_ios_display_text() {
    let chain = FakeChain::new();
    let app = app_over(&chain);
    let refused = app.write_org(OrgId::new([0x77; 20])).await.expect_err("refused before the chain");
    assert_eq!(refused, org_io::WriteRefused::NotHeld.to_string());
    assert_ne!(refused, "NotHeld", "not the Debug form");
}

// verifies: LLR-k4rhbs
#[tokio::test(start_paused = true)]
async fn an_included_write_is_verified_once_and_a_refused_or_unknown_one_is_not() {
    let chain = FakeChain::new();
    let app = app_over(&chain);
    let founder = founding(&app.io()).await;
    let included = app.write_founding(founder).await.expect("a report");
    assert_eq!(included["verification"]["result"], "adopted", "{included}");
    assert_eq!(chain.reads(), 1, "one state read: the verification's");
    assert_eq!(chain.finalised_reads(), 1, "one finalised-block read: the verification's");

    for (flag, expected) in [("write_failing", "refused"), ("write_hanging", "unknown")] {
        let chain = FakeChain::new();
        let app = app_over(&chain);
        let founder = match flag {
            "write_failing" => {
                chain.write_failing.store(true, Ordering::SeqCst);
                founding(&app.io()).await
            }
            _ => founding_whose_write_hangs(&chain, &app.io()).await,
        };
        let (reads, finalised_reads) = (chain.reads(), chain.finalised_reads());
        let report = app.write_founding(founder).await.expect("a report");
        assert_eq!(report["result"], expected, "{flag}: {report}");
        assert!(report.get("verification").is_none(), "{flag}: no verification is asked: {report}");
        assert_eq!(chain.reads(), reads, "{flag}: no state read");
        assert_eq!(chain.finalised_reads(), finalised_reads, "{flag}: no finalised-block read");
    }
}

// verifies: LLR-k4rhbs
#[tokio::test(flavor = "multi_thread")]
async fn an_included_founding_write_is_verified_by_its_founder_and_an_update_by_its_org_id() {
    // The founding: verified at its stored proxy by the founder's key, and
    // reported under the Organisation its adoption made.
    let chain = FakeChain::new();
    let app = app_over(&chain);
    let founder = founding(&app.io()).await;
    let founded = app.write_founding(founder).await.expect("a report");
    let org = org_of(&app.io(), founder).await;
    assert_eq!(founded["verification"]["result"], "adopted", "{founded}");
    assert_eq!(founded["verification"]["org_id"], serde_json::json!(hex::encode(org.as_bytes())));
    assert_eq!(founded["verification"]["epoch"], 1);

    // An update: verified by its org_id.
    let chain = FakeChain::new();
    let admitted = admitted(&chain).await;
    let updated = admitted.inviter.write_org(admitted.org).await.expect("a report");
    assert_eq!(updated["result"], "included", "{updated}");
    assert_eq!(updated["verification"]["result"], "adopted", "{updated}");
    assert_eq!(updated["verification"]["org_id"], serde_json::json!(hex::encode(admitted.org.as_bytes())));
    assert_eq!(updated["verification"]["epoch"], 2);
}

/// The ed25519 base point's canonical encoding: a valid Device public key.
const BASE_POINT: [u8; 32] = [
    0x58, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
    0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
];

fn device() -> DevicePublicKey {
    DevicePublicKey::parse(&BASE_POINT).unwrap()
}

/// The Organisation every constructed entry is for, and its id as hex.
fn the_org() -> (EntryOrg, String) {
    let org = OrgId::new([0x11; 20]);
    (EntryOrg::Id(org), hex::encode(org.as_bytes()))
}

fn store_failure() -> StoreFailure {
    StoreFailure::Reloaded {
        failed: StoreFileError { step: StoreFileStep::Write, path: "/data/store".into(), reason: "disk full".into() },
    }
}

fn store_failure_text() -> String {
    store_failure().to_string()
}

/// A verification org-node refused for a reason no other result names, and its text.
fn refused() -> (Verified, String) {
    let error = org_io::node::OrgNodeError::AdmissionNotOurs { org_id: OrgId::new([0x11; 20]) };
    let text = error.to_string();
    (Verified::Refused { error: org_io::Cause::new(error) }, text)
}

fn store() -> Verified {
    Verified::Store { failure: org_io::Cause::new(store_failure()) }
}

/// An Organisation founded and adopted over `chain`, written and verified
/// through `write_organisation_to_chain`.
async fn founded(chain: &FakeChain) -> (App, OrgId) {
    let app = app_over(chain);
    let founder = founding(&app.io()).await;
    app.write_founding(founder).await.expect("written");
    let org = org_of(&app.io(), founder).await;
    (app, org)
}

// verifies: LLR-73by2q
#[test]
fn entry_for_tags_each_result_and_maps_store_keychain_and_refused_to_unverified() {
    let (named, org) = the_org();
    let other_root = Inconsistency::OtherRoot { epoch: Epoch::new(3) };
    let cases = [
        (Verified::InStep { epoch: Epoch::new(3) }, serde_json::json!({ "result": "in_step", "org_id": org, "epoch": 3 })),
        (
            Verified::Adopted { epoch: Epoch::new(4), discarded: vec![], own: false },
            serde_json::json!({ "result": "adopted", "org_id": org, "epoch": 4, "own": false, "discarded": [] }),
        ),
        (
            Verified::Adopted { epoch: Epoch::new(4), discarded: vec![], own: true },
            serde_json::json!({ "result": "adopted", "org_id": org, "epoch": 4, "own": true, "discarded": [] }),
        ),
        (Verified::Removed { org_id: OrgId::new([0x11; 20]) }, serde_json::json!({ "result": "removed", "org_id": org })),
        (Verified::Behind { chain_epoch: Epoch::new(9) }, serde_json::json!({ "result": "behind", "org_id": org, "chain_epoch": 9 })),
        (
            Verified::Inconsistent { reason: other_root.clone() },
            serde_json::json!({ "result": "inconsistent", "org_id": org, "reason": other_root.to_string() }),
        ),
        (
            Verified::Unverified { reason: UnverifiedReason::OrgNotOnChain },
            serde_json::json!({ "result": "unverified", "org_id": org, "reason": UnverifiedReason::OrgNotOnChain.to_string() }),
        ),
    ];
    for (verified, expected) in cases {
        assert_eq!(wire(&entry_for(&named, &verified)), expected, "{verified:?}");
    }
    // The store write or keychain read a verification needed failed, or
    // org-node refused for a reason no other result names: unverified,
    // with org-io's text.
    let keychain = KeychainError::Locked { device_key: device() };
    for (verified, text) in [
        (store(), store_failure_text()),
        (Verified::Keychain { error: keychain.clone() }, keychain.to_string()),
        refused(),
    ] {
        assert_eq!(
            wire(&entry_for(&named, &verified)),
            serde_json::json!({ "result": "unverified", "org_id": org, "reason": text }),
            "{verified:?}"
        );
    }
}

// verifies: LLR-73by2q
// A verification that settled an unknown write is its inner outcome's
// entry, with the settlement as `write`; no other entry carries `write`.
#[test]
fn a_settled_verification_is_its_outcomes_entry_with_the_write_label() {
    let (named, org) = the_org();
    for (write, label) in [
        (WriteOutcome::Adopted, "adopted"),
        (WriteOutcome::NotExecuted, "not_executed"),
        (WriteOutcome::Deprecated, "deprecated"),
        (WriteOutcome::StillUnknown, "still_unknown"),
    ] {
        let settled = Verified::Settled {
            outcome: Box::new(Verified::Adopted { epoch: Epoch::new(4), discarded: vec![], own: false }),
            write,
        };
        assert_eq!(
            wire(&entry_for(&named, &settled)),
            serde_json::json!({ "result": "adopted", "org_id": org, "epoch": 4, "own": false, "discarded": [], "write": label }),
            "{settled:?}"
        );
    }
    let behind = Verified::Settled {
        outcome: Box::new(Verified::Behind { chain_epoch: Epoch::new(6) }),
        write: WriteOutcome::StillUnknown,
    };
    assert_eq!(
        wire(&entry_for(&named, &behind)),
        serde_json::json!({ "result": "behind", "org_id": org, "chain_epoch": 6, "write": "still_unknown" })
    );
    let plain = wire(&entry_for(&named, &Verified::InStep { epoch: Epoch::new(4) }));
    assert!(plain.get("write").is_none(), "{plain}");
}

// verifies: LLR-73by2q
// A founding is named by its founder always, beside its proxy's
// Organisation identifier once recorded and `null` before, never an empty
// string.
#[test]
fn entry_for_names_a_founding_by_its_founder_beside_its_org_id_or_null() {
    let no_proxy = wire(&entry_for(
        &EntryOrg::Founding { founder: device(), org_id: None },
        &Verified::Unverified { reason: UnverifiedReason::NoProxyYet },
    ));
    assert_eq!(
        no_proxy,
        serde_json::json!({
            "result": "unverified",
            "org_id": null,
            "founder": hex::encode(device().as_bytes()),
            "reason": UnverifiedReason::NoProxyYet.to_string(),
        })
    );
    let proxied = OrgId::new([0x22; 20]);
    let recorded = wire(&entry_for(
        &EntryOrg::Founding { founder: device(), org_id: Some(proxied) },
        &Verified::Unverified { reason: UnverifiedReason::NotYetVerified },
    ));
    assert_eq!(
        recorded,
        serde_json::json!({
            "result": "unverified",
            "org_id": hex::encode(proxied.as_bytes()),
            "founder": hex::encode(device().as_bytes()),
            "reason": UnverifiedReason::NotYetVerified.to_string(),
        })
    );
}

// verifies: LLR-jf63e4
// Over an unreadable chain, one Organisation verified by its `org_id` is
// unverified with org-io's reason unchanged.
#[tokio::test]
async fn verify_organisation_over_an_unreadable_chain_is_unverified_with_org_ios_reason() {
    let chain = FakeChain::new();
    let (app, held) = founded(&chain).await;
    chain.read_failing.store(true, Ordering::SeqCst);
    let reason = match app.io().verify_organisation(HeldTarget::Organisation(held)).await {
        Verified::Unverified { reason } => reason.to_string(),
        other => panic!("an unreadable chain is unverified: {other:?}"),
    };
    assert_eq!(
        app.verify(Some(held)).await,
        vec![serde_json::json!({ "result": "unverified", "org_id": hex::encode(held.as_bytes()), "reason": reason })]
    );
}

// verifies: LLR-jf63e4
// Boundary: a provisional founding whose proxy is not recorded has no
// Organisation identifier; with no `org_id` its entry carries `org_id`
// null and its founder, never an empty string.
#[tokio::test]
async fn a_founding_with_no_proxy_is_entered_with_a_null_org_id_and_its_founder() {
    let chain = FakeChain::new();
    let app = app_over(&chain);
    let founder = founding(&app.io()).await;
    let entries = app.verify(None).await;
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert_eq!(entries[0]["org_id"], serde_json::Value::Null, "{entries:?}");
    assert_eq!(entries[0]["founder"], serde_json::json!(hex::encode(founder.as_bytes())));
    assert_ne!(entries[0]["org_id"], serde_json::json!(""), "never an empty string");
}

// verifies: LLR-jf63e4
// A founding whose proxy is recorded and which no verification has
// adopted yet: its entry names the Organisation identifier the proxy gave
// and still carries its founder, the key the webview writes it by
// (LLR-rhc555), so a restarted app finds it in its first verification.
#[tokio::test]
async fn a_founding_with_its_proxy_recorded_is_entered_with_its_org_id_and_still_its_founder() {
    // The write creates and records the proxy, then its founding step is
    // refused, so the founding is held with its proxy and not adopted.
    let chain = FakeChain::new();
    let app = app_over(&chain);
    let founder = founding(&app.io()).await;
    chain.found_at_failing.store(true, Ordering::SeqCst);
    let refused = app.write_founding(founder).await.expect("a report");
    assert_eq!(refused["result"], "refused", "{refused}");
    let org = org_of(&app.io(), founder).await;
    let entries = app.verify(None).await;
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert_eq!(entries[0]["org_id"], serde_json::json!(hex::encode(org.as_bytes())), "{entries:?}");
    assert_eq!(entries[0]["founder"], serde_json::json!(hex::encode(founder.as_bytes())), "{entries:?}");
}

// verifies: LLR-es49wp
#[tokio::test(flavor = "multi_thread")]
async fn an_own_adoption_is_sent_once_and_nothing_else_is() {
    // This node's own adoption: sent once, one entry for Bob's Device.
    let chain = FakeChain::new();
    let admitted = admitted(&chain).await;
    let written = admitted.inviter.write_org(admitted.org).await.expect("a report");
    let entry = &written["verification"];
    assert_eq!(entry["result"], "adopted", "{written}");
    assert_eq!(entry["own"], true);
    let sends = entry["sends"].as_array().expect("an own adoption carries its sends");
    assert_eq!(sends.len(), 1, "one send per Device, not repeated: {sends:?}");
    assert_eq!(sends[0]["device"], serde_json::json!(hex::encode(admitted.member_device.as_bytes())));
    // The one send delivered it. This does not detect a doubled send: org-io
    // answers a repeated send of an epoch the recipient already holds as
    // accepted (`org-io/src/receiver.rs`, `Checked::AlreadyHeld`), so a second
    // send would also report `sent`, and the recipient reports nothing; the
    // single report per Device above is all this interface shows (PR-2t6ssm).
    assert_eq!(sends[0]["result"], "sent", "{sends:?}");
    assert!(entry.get("sends_refused").is_none(), "{entry}");

    // In step afterwards: nothing is sent.
    let inviter = admitted.inviter.io();
    let again = wire(&commands::verify_and_send(&inviter, HeldTarget::Organisation(admitted.org)).await);
    assert_eq!(again["result"], "in_step", "{again}");
    assert!(again.get("sends").is_none(), "{again}");

    // An adoption this node did not build (`own` false): the invitee, once
    // it holds what the inviter sent, adopts it from the chain and asks no send.
    let mut not_own = wire(&commands::verify_and_send(&admitted.invitee, HeldTarget::Organisation(admitted.org)).await);
    for _ in 0..100 {
        if not_own["result"] == "adopted" {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
        not_own = wire(&commands::verify_and_send(&admitted.invitee, HeldTarget::Organisation(admitted.org)).await);
    }
    assert_eq!(not_own["result"], "adopted", "{not_own}");
    assert_eq!(not_own["own"], false, "{not_own}");
    assert!(not_own.get("sends").is_none(), "{not_own}");
    assert!(not_own.get("sends_refused").is_none(), "{not_own}");
}

// verifies: LLR-n64wvr, LLR-73by2q
#[test]
fn discarded_lists_each_change_tagged_by_kind_and_is_empty_not_absent() {
    let (named, _) = the_org();
    let discarded = vec![
        DiscardedChangeSummary::Admission {
            invite_id: InviteId::new([0x22; 32]),
            name: Some(Name::parse("Bob").unwrap()),
            surname: Some(Surname::parse("Builder").unwrap()),
            handle: Handle::parse("bob").unwrap(),
        },
        DiscardedChangeSummary::Revocation { member_id: MemberId::new([0x33; 32]), handle: Some(Handle::parse("carol").unwrap()) },
        // No kept request gives the names; the adopted record no longer lists the Member.
        DiscardedChangeSummary::Admission {
            invite_id: InviteId::new([0x44; 32]),
            name: None,
            surname: None,
            handle: Handle::parse("dave").unwrap(),
        },
        DiscardedChangeSummary::Revocation { member_id: MemberId::new([0x55; 32]), handle: None },
    ];
    let entry = wire(&entry_for(&named, &Verified::Adopted { epoch: Epoch::new(2), discarded, own: true }));
    assert_eq!(
        entry["discarded"],
        serde_json::json!([
            { "kind": "admission", "invite_id": "22".repeat(32), "name": "Bob", "surname": "Builder", "handle": "bob" },
            { "kind": "revocation", "member_id": "33".repeat(32), "handle": "carol" },
            { "kind": "admission", "invite_id": "44".repeat(32), "name": null, "surname": null, "handle": "dave" },
            { "kind": "revocation", "member_id": "55".repeat(32), "handle": null },
        ])
    );
    // A value org-io does not report is present as `null`, never absent (LLR-n64wvr).
    for (index, field) in [(2, "name"), (2, "surname"), (3, "handle")] {
        let change = entry["discarded"][index].as_object().expect("an object");
        assert!(change.contains_key(field) && change[field].is_null(), "{field} present and null: {entry}");
    }
    let none = wire(&entry_for(&named, &Verified::Adopted { epoch: Epoch::new(2), discarded: vec![], own: true }));
    assert_eq!(none["discarded"], serde_json::json!([]), "empty, not absent: {none}");
}

// verifies: LLR-rywe8f, LLR-cugcu6
#[test]
fn sends_reports_each_device_sent_or_failed_beside_the_others() {
    // Five entries, in org-io's order: one delivered, four not, each failure
    // beside the others. (One valid key serves for all: the mapping is per
    // entry, not per key.)
    let report = SendReport {
        per_device: vec![
            (device(), Delivery::Delivered),
            (device(), Delivery::NotDelivered { reason: SendError::NoLoopbackAddress }),
            (device(), Delivery::Refused),
            (device(), Delivery::AlreadyReceived),
            (device(), Delivery::InviteUsedByAnotherDevice),
        ],
    };
    let wire_report = wire(&device_reports(&report));
    let hex_device = hex::encode(device().as_bytes());
    assert_eq!(
        wire_report,
        serde_json::json!([
            { "device": hex_device, "result": "sent" },
            { "device": hex_device, "result": "failed", "reason": SendError::NoLoopbackAddress.to_string() },
            { "device": hex_device, "result": "failed", "reason": "refused by the recipient" },
            { "device": hex_device, "result": "failed", "reason": "already received" },
            { "device": hex_device, "result": "failed", "reason": "invite used by another device" },
        ]),
        "a failure is reported beside the other results"
    );
    // Attached to an adopted entry, the other fields are LLR-jf63e4's.
    let (named, _) = the_org();
    let mut entry = entry_for(&named, &Verified::Adopted { epoch: Epoch::new(2), discarded: vec![], own: true });
    if let EntryResultDto::Adopted { sends, .. } = &mut entry.result {
        *sends = Some(device_reports(&report));
    }
    let entry = wire(&entry);
    assert_eq!(entry["result"], "adopted");
    assert_eq!(entry["epoch"], 2);
    assert_eq!(entry["own"], true);
    assert_eq!(entry["sends"], wire_report);
}

// verifies: LLR-nkg8sb
#[tokio::test(flavor = "multi_thread")]
async fn a_behind_result_is_requested_once_with_no_user_step() {
    let chain = FakeChain::new();
    let admitted = admitted(&chain).await;
    admitted.inviter.write_org(admitted.org).await.expect("written");
    let inviter = admitted.inviter.io();
    // Bob's Device adopts the epoch it was sent, so it holds the
    // Organisation and answers a request for it.
    let mut held = wire(&commands::verify_and_send(&admitted.invitee, HeldTarget::Organisation(admitted.org)).await);
    for _ in 0..100 {
        if held["result"] == "adopted" {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
        held = wire(&commands::verify_and_send(&admitted.invitee, HeldTarget::Organisation(admitted.org)).await);
    }
    assert_eq!(held["result"], "adopted", "{held}");
    chain.epoch_ahead.store(true, Ordering::SeqCst);
    let entry = wire(&commands::verify_and_send(&inviter, HeldTarget::Organisation(admitted.org)).await);
    assert_eq!(entry["result"], "behind", "{entry}");
    let requests = entry["requests"].as_array().expect("a behind result carries its requests");
    assert_eq!(requests.len(), 1, "every other Device of the record, asked once: {requests:?}");
    assert_eq!(requests[0]["device"], serde_json::json!(hex::encode(admitted.member_device.as_bytes())));
    // The one request reached Bob's Device, which holds the Organisation.
    assert_eq!(requests[0]["result"], "sent", "{requests:?}");
    assert!(entry.get("requests_refused").is_none(), "{entry}");
    // Bob's Device answered exactly one request: it was made once.
    for _ in 0..100 {
        if admitted.invitee_answered.load(Ordering::SeqCst) > 0 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(admitted.invitee_answered.load(Ordering::SeqCst), 1, "one request reached Bob's Device");
    // In step: no request.
    chain.epoch_ahead.store(false, Ordering::SeqCst);
    let in_step = wire(&commands::verify_and_send(&inviter, HeldTarget::Organisation(admitted.org)).await);
    assert_eq!(in_step["result"], "in_step", "{in_step}");
    assert!(in_step.get("requests").is_none(), "{in_step}");
}

// verifies: LLR-jf63e4, LLR-g8wfy9
#[tokio::test]
async fn verify_all_reports_in_step_and_lists_the_organisation_verified() {
    let chain = FakeChain::new();
    let (app, org) = founded(&chain).await;
    let hex_org = hex::encode(org.as_bytes());
    assert_eq!(app.verify(None).await, vec![serde_json::json!({ "result": "in_step", "org_id": hex_org, "epoch": 1 })]);
    let orgs = app.list_orgs().await;
    assert_eq!(orgs.len(), 1);
    assert_eq!(orgs[0]["verification"], serde_json::json!({ "status": "in_step", "epoch": 1 }));
}

// verifies: LLR-jf63e4, LLR-g8wfy9
#[tokio::test]
async fn verify_all_over_an_unreadable_chain_reports_unverified_with_the_reason() {
    let chain = FakeChain::new();
    let (app, org) = founded(&chain).await;
    chain.read_failing.store(true, Ordering::SeqCst);
    let entries = app.verify(None).await;
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert_eq!(entry["result"], "unverified", "{entry}");
    assert_eq!(entry["org_id"], hex::encode(org.as_bytes()));
    let reason = entry["reason"].as_str().unwrap();
    assert!(!reason.is_empty());
    let orgs = app.list_orgs().await;
    assert_eq!(orgs[0]["verification"]["status"], "unverified");
    assert_eq!(orgs[0]["verification"]["reason"], reason, "the list shows org-io's reason unchanged");
}

// verifies: LLR-q96vdr
// Each last verification as list_orgs reports it: none yet is not_checked,
// a removal is removed, a settled one its outcome's status, a refusal, a
// store failure or a keychain refusal unverified with its reason.
#[tokio::test]
async fn org_dto_reports_not_checked_removed_settled_and_refused_statuses() {
    let chain = FakeChain::new();
    let (app, org) = founded(&chain).await;
    let summary = app.io().organisation(org).await.expect("held");
    let status = |last: &Verified| wire(&OrgDto::new(&summary, last))["verification"].clone();
    assert_eq!(status(&Verified::Unverified { reason: UnverifiedReason::NotYetVerified }), serde_json::json!({ "status": "not_checked" }));
    assert_eq!(status(&Verified::Removed { org_id: org }), serde_json::json!({ "status": "removed" }));
    assert_eq!(
        status(&Verified::Settled { outcome: Box::new(Verified::InStep { epoch: Epoch::new(1) }), write: WriteOutcome::Adopted }),
        serde_json::json!({ "status": "in_step", "epoch": 1 })
    );
    assert_eq!(
        status(&Verified::Adopted { epoch: Epoch::new(2), discarded: vec![], own: true }),
        serde_json::json!({ "status": "adopted", "epoch": 2 })
    );
    let (refused, refused_text) = refused();
    assert_eq!(status(&refused), serde_json::json!({ "status": "unverified", "reason": refused_text }));
    assert_eq!(
        status(&store()),
        serde_json::json!({ "status": "unverified", "reason": store_failure_text() })
    );
    let keychain = KeychainError::Locked { device_key: device() };
    assert_eq!(
        status(&Verified::Keychain { error: keychain.clone() }),
        serde_json::json!({ "status": "unverified", "reason": keychain.to_string() })
    );
    assert_eq!(
        status(&Verified::Unverified { reason: UnverifiedReason::OrgNotOnChain }),
        serde_json::json!({ "status": "unverified", "reason": UnverifiedReason::OrgNotOnChain.to_string() })
    );
}

// ---------------------------------------------------------------------------
// The follow-up decision, tested at its own interface (SDD-6zqqm8; TEST_GUIDELINES Seams
// 2 criterion 4, an SDD item's own interface).
// ---------------------------------------------------------------------------

// verifies: LLR-292r26
#[test]
fn an_own_adoption_asks_a_send_and_behind_a_request_and_nothing_else_asks_anything() {
    use ods_poc_lib::follow_up::{after_verification, VerificationFollowUp};
    let org = OrgId::new([0x11; 20]);
    let own = Verified::Adopted { epoch: Epoch::new(4), discarded: vec![], own: true };
    let behind = Verified::Behind { chain_epoch: Epoch::new(9) };
    assert_eq!(after_verification(&own, Some(org)), Some(VerificationFollowUp::Send(org)));
    assert_eq!(after_verification(&behind, Some(org)), Some(VerificationFollowUp::Request(org)));
    // A settled verification follows its outcome.
    let settled = |outcome: &Verified, write| Verified::Settled { outcome: Box::new(outcome.clone()), write };
    assert_eq!(after_verification(&settled(&own, WriteOutcome::Adopted), Some(org)), Some(VerificationFollowUp::Send(org)));
    assert_eq!(
        after_verification(&settled(&behind, WriteOutcome::StillUnknown), Some(org)),
        Some(VerificationFollowUp::Request(org))
    );
    // Every other outcome asks nothing, an adoption this node did not build included.
    let keychain = KeychainError::Locked { device_key: device() };
    for other in [
        Verified::Adopted { epoch: Epoch::new(4), discarded: vec![], own: false },
        Verified::InStep { epoch: Epoch::new(3) },
        Verified::Removed { org_id: org },
        Verified::Inconsistent { reason: Inconsistency::OtherRoot { epoch: Epoch::new(3) } },
        Verified::Unverified { reason: UnverifiedReason::OrgNotOnChain },
        refused().0,
        store(),
        Verified::Keychain { error: keychain },
        settled(&Verified::InStep { epoch: Epoch::new(3) }, WriteOutcome::NotExecuted),
    ] {
        assert_eq!(after_verification(&other, Some(org)), None, "{other:?}");
    }
    // An entry naming no Organisation (a founding with no proxy) asks nothing, whatever its outcome.
    for outcome in [own, behind] {
        assert_eq!(after_verification(&outcome, None), None, "{outcome:?}");
    }
}

// verifies: LLR-bbfgk7
#[test]
fn only_an_included_write_asks_a_verification() {
    use ods_poc_lib::follow_up::verifies_after_write;
    assert!(verifies_after_write(&WriteReport::Included { epoch: Epoch::new(0) }));
    assert!(!verifies_after_write(&WriteReport::Refused { failure: org_io::WriteFailure::WindowReadTimedOut }));
    assert!(!verifies_after_write(&WriteReport::Unknown { reason: "timed out".to_string() }));
}

// ---------------------------------------------------------------------------
// Abnormal cases of the send and the request after a verification.
// ---------------------------------------------------------------------------

// verifies: LLR-k4rxyt
// org-io's refusal of a send or a request as a whole (the Organisation not
// held, its endpoint not bound) cannot be produced through
// `verify_and_send` over the substitute chain without faking org-io: the
// verification that asks the send has just found the Organisation held,
// and a refused bind needs the keychain the verification itself reads
// (TEST_GUIDELINES Seams 2 criteria 1 and 4).
#[test]
fn a_send_or_request_refused_as_a_whole_is_an_empty_report_and_its_text() {
    use ods_poc_lib::entry::attach_report;
    use ods_poc_lib::follow_up::VerificationFollowUp;
    let (named, org) = the_org();
    let org_id = OrgId::new([0x11; 20]);
    let refusal = "Organisation 1111 is not held here".to_string();

    let mut adopted = entry_for(&named, &Verified::Adopted { epoch: Epoch::new(4), discarded: vec![], own: true });
    attach_report(&mut adopted, VerificationFollowUp::Send(org_id), Err(refusal.clone()));
    assert_eq!(
        wire(&adopted),
        serde_json::json!({ "result": "adopted", "org_id": org, "epoch": 4, "own": true, "discarded": [], "sends": [], "sends_refused": refusal })
    );
    let mut behind = entry_for(&named, &Verified::Behind { chain_epoch: Epoch::new(9) });
    attach_report(&mut behind, VerificationFollowUp::Request(org_id), Err(refusal.clone()));
    assert_eq!(
        wire(&behind),
        serde_json::json!({ "result": "behind", "org_id": org, "chain_epoch": 9, "requests": [], "requests_refused": refusal })
    );

    // A report: its per-Device entries, and no refusal, failed Devices included.
    let report = || SendReport { per_device: vec![(device(), Delivery::Delivered), (device(), Delivery::Refused)] };
    let expected = serde_json::json!([
        { "device": hex::encode(device().as_bytes()), "result": "sent" },
        { "device": hex::encode(device().as_bytes()), "result": "failed", "reason": "refused by the recipient" },
    ]);
    let mut adopted = entry_for(&named, &Verified::Adopted { epoch: Epoch::new(4), discarded: vec![], own: true });
    attach_report(&mut adopted, VerificationFollowUp::Send(org_id), Ok(report()));
    assert_eq!(wire_field(&adopted, "sends"), expected);
    assert!(wire(&adopted).get("sends_refused").is_none());
    let mut behind = entry_for(&named, &Verified::Behind { chain_epoch: Epoch::new(9) });
    attach_report(&mut behind, VerificationFollowUp::Request(org_id), Ok(report()));
    assert_eq!(wire_field(&behind, "requests"), expected);
    assert!(wire(&behind).get("requests_refused").is_none());

    // Abnormal: a report for a follow-up its entry does not ask (a send
    // beside behind, a request beside an adoption, either beside in step)
    // changes nothing.
    for (verified, follow_up) in [
        (Verified::Behind { chain_epoch: Epoch::new(9) }, VerificationFollowUp::Send(org_id)),
        (Verified::Adopted { epoch: Epoch::new(4), discarded: vec![], own: true }, VerificationFollowUp::Request(org_id)),
        (Verified::InStep { epoch: Epoch::new(4) }, VerificationFollowUp::Send(org_id)),
    ] {
        let mut entry = entry_for(&named, &verified);
        let before = wire(&entry);
        attach_report(&mut entry, follow_up, Err(refusal.clone()));
        assert_eq!(wire(&entry), before, "{verified:?} with {follow_up:?}");
    }
}

/// `entry`'s `field`, as it crosses IPC.
fn wire_field(entry: &VerificationEntryDto, field: &str) -> serde_json::Value {
    wire(entry)[field].clone()
}

// verifies: LLR-nkg8sb
// Abnormal: a Device that refuses the request is reported failed with its
// reason beside the entry's other fields, and the request is still made
// once, with no refusal of the whole.
#[tokio::test(flavor = "multi_thread")]
async fn a_behind_request_a_device_refuses_is_reported_failed_for_that_device() {
    let chain = FakeChain::new();
    let admitted = admitted(&chain).await;
    admitted.inviter.write_org(admitted.org).await.expect("written");
    let inviter = admitted.inviter.io();
    chain.epoch_ahead.store(true, Ordering::SeqCst);
    let entry = wire(&commands::verify_and_send(&inviter, HeldTarget::Organisation(admitted.org)).await);
    assert_eq!(entry["result"], "behind", "{entry}");
    assert_eq!(
        entry["requests"],
        serde_json::json!([{
            "device": hex::encode(admitted.member_device.as_bytes()),
            "result": "failed",
            "reason": "refused by the recipient",
        }]),
        "{entry}"
    );
    assert!(entry.get("requests_refused").is_none(), "{entry}");
}

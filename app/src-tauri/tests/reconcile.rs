#![cfg(feature = "test-support")]
//! The reconcile command's body and its IPC shapes (LLR-ekz4zn, LLR-8axx7h),
//! over org-io's substitute chain (`test_support::FakeChain`). The command
//! with the chain not configured is driven across IPC in `ipc.rs`.
//!
//! *Reconciled with S4 (`worktree-org-io-transport`), 2026-10-14:* every
//! test keeps its name, its scenario and its `verifies:` line. The handle is
//! `OrgIo::open_for_test` over a data directory, its Personas' secrets in
//! the shared in-memory keychain (S4, tasks T9 and T15: the app names no
//! org-node service or passphrase store), and its methods are `async` and
//! take `&self` (S4, task T11). The app names org-io alone (S4, task T15),
//! so the Device key in the IPC-shape test is parsed from fixed bytes, not
//! drawn from org-node's `DeviceSeed`.

use std::sync::atomic::Ordering;

use ods_poc_lib::commands::{reconcile_all, OrgDto, ReconciliationDto};
use org_io::node::{DevicePublicKey, Epoch, Handle, Name, OrgId, Surname};
use org_io::test_support::{FakeChain, MemoryKeychain};
use org_io::transport::TransportMode;
use org_io::{OrgIo, Reconciliation};
use rand::rngs::OsRng;

fn handle(tag: &str, chain: &FakeChain) -> OrgIo {
    let dir = std::env::temp_dir().join(format!("ods-app-reconcile-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    MemoryKeychain::shared().install();
    OrgIo::open_for_test(&dir, TransportMode::Loopback, chain).unwrap()
}

async fn founded(tag: &str, chain: &FakeChain) -> (OrgIo, OrgId) {
    let io = handle(tag, chain);
    let pid = io
        .create_persona(&mut OsRng, Handle::parse("alice").unwrap(), Name::parse("Alice").unwrap(), Surname::parse("Smith").unwrap())
        .await
        .unwrap();
    let org = io.found_organisation(&mut OsRng, &pid).await.unwrap();
    (io, org)
}

async fn listed(io: &OrgIo) -> Vec<serde_json::Value> {
    let mut orgs = vec![];
    for org in io.organisations().await {
        orgs.push(serde_json::to_value(OrgDto::new(&org, io.verification(org.org_id).await)).unwrap());
    }
    orgs
}

// verifies: LLR-ekz4zn, LLR-8axx7h
#[tokio::test]
async fn reconcile_all_reports_in_step_and_lists_the_organisation_verified() {
    let chain = FakeChain::new();
    let (io, org) = founded("in-step", &chain).await;
    let hex_org = hex::encode(org.as_bytes());

    let results = reconcile_all(&io, &mut OsRng).await;
    let wire: Vec<serde_json::Value> = results.iter().map(|result| serde_json::to_value(result).unwrap()).collect();
    assert_eq!(wire, vec![serde_json::json!({ "result": "in_step", "org_id": hex_org, "epoch": 1 })]);

    let orgs = listed(&io).await;
    assert_eq!(orgs.len(), 1);
    assert_eq!(orgs[0]["verification"], serde_json::json!({ "status": "verified" }));
}

// verifies: LLR-ekz4zn, LLR-8axx7h
#[tokio::test]
async fn reconcile_all_over_an_unreadable_chain_reports_unverified_with_the_reason() {
    let chain = FakeChain::new();
    let (io, org) = founded("unreadable", &chain).await;
    chain.read_failing.store(true, Ordering::SeqCst);

    let results = reconcile_all(&io, &mut OsRng).await;
    assert_eq!(results.len(), 1);
    let wire = serde_json::to_value(&results[0]).unwrap();
    assert_eq!(wire["result"], "unverified", "{wire}");
    assert_eq!(wire["org_id"], hex::encode(org.as_bytes()));
    let reason = wire["reason"].as_str().unwrap();
    assert!(!reason.is_empty());

    let orgs = listed(&io).await;
    assert_eq!(orgs[0]["verification"]["status"], "unverified");
    assert_eq!(orgs[0]["verification"]["reason"], reason, "the list shows org-io's reason unchanged");
}

/// The ed25519 base point's canonical encoding: a valid Device public key
/// (torsion-free, not of small order).
const BASE_POINT: [u8; 32] = [
    0x58, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
    0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
];

// verifies: LLR-ekz4zn
#[test]
fn each_reconciliation_crosses_ipc_tagged_with_its_fields() {
    let org = OrgId::new([0x11; 20]);
    let hex_org = "11".repeat(20);
    let device = DevicePublicKey::parse(&BASE_POINT).unwrap();
    let cases = [
        (Reconciliation::InStep { org_id: org, epoch: Epoch::new(3) }, serde_json::json!({ "result": "in_step", "org_id": hex_org, "epoch": 3 })),
        (
            Reconciliation::Committed { org_id: org, epoch: Epoch::new(4), recipients: vec![device] },
            serde_json::json!({ "result": "committed", "org_id": hex_org, "epoch": 4, "recipients": [hex::encode(device.as_bytes())] }),
        ),
        (Reconciliation::Behind { org_id: org, chain_epoch: Epoch::new(9) }, serde_json::json!({ "result": "behind", "org_id": hex_org, "chain_epoch": 9 })),
        (Reconciliation::Removed { org_id: org }, serde_json::json!({ "result": "removed", "org_id": hex_org })),
        (
            Reconciliation::Refused { org_id: org, reason: "no".into() },
            serde_json::json!({ "result": "refused", "org_id": hex_org, "reason": "no" }),
        ),
        (
            Reconciliation::Unverified { org_id: org, reason: "down".into() },
            serde_json::json!({ "result": "unverified", "org_id": hex_org, "reason": "down" }),
        ),
    ];
    for (result, expected) in cases {
        assert_eq!(serde_json::to_value(ReconciliationDto::from(&result)).unwrap(), expected, "{result:?}");
    }
}

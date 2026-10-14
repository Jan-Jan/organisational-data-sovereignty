#![cfg(feature = "test-support")]
//! The reconcile command's body and its IPC shapes (LLR-ekz4zn, LLR-8axx7h),
//! over org-io's substitute chain (`test_support::FakeChain`). The command
//! with the chain not configured is driven across IPC in `ipc.rs`.

use std::sync::atomic::Ordering;

use ods_poc_lib::commands::{reconcile_all, OrgDto, ReconciliationDto};
use org_io::node::service::OrgService;
use org_io::node::store::PersonaStore;
use org_io::node::{DeviceSeed, Epoch, Handle, Name, OrgId, Surname};
use org_io::test_support::FakeChain;
use org_io::{OrgIo, Reconciliation};
use rand::rngs::OsRng;

fn handle(tag: &str, chain: &FakeChain) -> OrgIo {
    let dir = std::env::temp_dir().join(format!("ods-app-reconcile-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    OrgIo::for_test(OrgService::new(PersonaStore::open(dir.join("store.bin"), "pw").unwrap()), chain)
}

async fn founded(tag: &str, chain: &FakeChain) -> (OrgIo, OrgId) {
    let mut io = handle(tag, chain);
    let pid = io
        .node_mut()
        .create_persona(&mut OsRng, Handle::parse("alice").unwrap(), Name::parse("Alice").unwrap(), Surname::parse("Smith").unwrap())
        .unwrap();
    let org = io.found_organisation(&mut OsRng, &pid).await.unwrap();
    (io, org)
}

fn listed(io: &OrgIo) -> Vec<serde_json::Value> {
    io.view()
        .organisations()
        .iter()
        .map(|org| serde_json::to_value(OrgDto::new(org, io.verification(org.org_id))).unwrap())
        .collect()
}

// verifies: LLR-ekz4zn, LLR-8axx7h
#[tokio::test]
async fn reconcile_all_reports_in_step_and_lists_the_organisation_verified() {
    let chain = FakeChain::new();
    let (mut io, org) = founded("in-step", &chain).await;
    let hex_org = hex::encode(org.as_bytes());

    let results = reconcile_all(&mut io, &mut OsRng).await;
    let wire: Vec<serde_json::Value> = results.iter().map(|result| serde_json::to_value(result).unwrap()).collect();
    assert_eq!(wire, vec![serde_json::json!({ "result": "in_step", "org_id": hex_org, "epoch": 1 })]);

    let orgs = listed(&io);
    assert_eq!(orgs.len(), 1);
    assert_eq!(orgs[0]["verification"], serde_json::json!({ "status": "verified" }));
}

// verifies: LLR-ekz4zn, LLR-8axx7h
#[tokio::test]
async fn reconcile_all_over_an_unreadable_chain_reports_unverified_with_the_reason() {
    let chain = FakeChain::new();
    let (mut io, org) = founded("unreadable", &chain).await;
    chain.read_failing.store(true, Ordering::SeqCst);

    let results = reconcile_all(&mut io, &mut OsRng).await;
    assert_eq!(results.len(), 1);
    let wire = serde_json::to_value(&results[0]).unwrap();
    assert_eq!(wire["result"], "unverified", "{wire}");
    assert_eq!(wire["org_id"], hex::encode(org.as_bytes()));
    let reason = wire["reason"].as_str().unwrap();
    assert!(!reason.is_empty());

    let orgs = listed(&io);
    assert_eq!(orgs[0]["verification"]["status"], "unverified");
    assert_eq!(orgs[0]["verification"]["reason"], reason, "the list shows org-io's reason unchanged");
}

// verifies: LLR-ekz4zn
#[test]
fn each_reconciliation_crosses_ipc_tagged_with_its_fields() {
    let org = OrgId::new([0x11; 20]);
    let hex_org = "11".repeat(20);
    let device = DeviceSeed::from([7; 32]).signing_keypair().device_key().unwrap();
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

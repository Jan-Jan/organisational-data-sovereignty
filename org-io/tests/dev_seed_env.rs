#![cfg(feature = "dev-seed")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The development seed path (LLR-rgdx22): the one read of ODS_ADMIN_SEED,
//! compiled only with `dev-seed`. The tests in this binary mutate the
//! environment, so each holds `ENVIRONMENT` while it runs.

use std::sync::Mutex;

use org_io::custody::signatory_from_environment;

const SEED_HEX: &str = "e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a";

/// Held by every test here: no other thread reads the environment meanwhile.
static ENVIRONMENT: Mutex<()> = Mutex::new(());

// verifies: LLR-rgdx22
#[test]
fn the_development_build_reads_the_seed_once_and_none_when_unset() {
    let _environment = ENVIRONMENT.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::set_var("ODS_ADMIN_SEED", SEED_HEX);
    let key = signatory_from_environment().unwrap().unwrap();
    assert!(format!("{key:?}").starts_with("SignatoryKey(account "));
    std::env::set_var("ODS_ADMIN_SEED", "0x0x00");
    assert!(signatory_from_environment().is_err(), "a malformed seed is refused");
    std::env::remove_var("ODS_ADMIN_SEED");
    assert!(signatory_from_environment().unwrap().is_none());
}

// A co-signer equal to the node's own account would make `controller_of`
// and the writer disagree (the writer would dispatch with the sender among
// the other signatories), so `connect` refuses it before any connection;
// the address is unreachable, so a connect that went on would not return
// a refusal within the bound.
// verifies: LLR-qhyc3n
#[test]
fn connect_refuses_a_co_signer_equal_to_the_own_account() {
    use org_io::connect::{ConnectFailed, ConnectFailure};
    use org_io::custody::{parse_seed, SignatoryKey};

    let own_account = SignatoryKey::from_seed(parse_seed(SEED_HEX).unwrap()).unwrap().account_id();
    let dir = std::env::temp_dir().join(format!("org-io-own-co-signer-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let service = org_io::node::service::OrgService::new(
        org_io::node::store::PersonaStore::open(dir.join("store.bin"), "pw").unwrap(),
    );
    let settings = org_io::ChainSettings {
        ws_url: "ws://127.0.0.1:1".into(),
        contract_h160: [0u8; 20],
        co_signer: Some(format!("0x{}", hex::encode(own_account.0))),
    };

    let refused = {
        let _environment = ENVIRONMENT.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::set_var("ODS_ADMIN_SEED", SEED_HEX);
        // A runtime of its own, so the lock is never held across an await.
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let bound = std::time::Duration::from_secs(5);
        let connected =
            runtime.block_on(async { tokio::time::timeout(bound, org_io::OrgIo::not_configured(service).connect(settings)).await });
        std::env::remove_var("ODS_ADMIN_SEED");
        connected.expect("refused before any connection").err().expect("refused")
    };
    let ConnectFailed { reason, .. } = *refused;
    assert!(matches!(reason, ConnectFailure::CoSignerIsOwnAccount), "{reason}");
    assert_eq!(reason.to_string(), "ODS_COSIGNER_PUB is this node's own account; a co-signer must be another account");
}

// A seed that is set but is not UTF-8 is a malformed seed, refused by rule
// with no part of the value, not an unset one.
// verifies: LLR-rgdx22
#[cfg(unix)]
#[test]
fn a_seed_set_but_not_utf8_is_refused_as_malformed_not_reported_unset() {
    use org_io::custody::{ConfigRule, ConfigVariable};
    use std::os::unix::ffi::OsStrExt;

    let _environment = ENVIRONMENT.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut not_utf8 = SEED_HEX.as_bytes().to_vec();
    not_utf8[10] = 0xff;
    std::env::set_var("ODS_ADMIN_SEED", std::ffi::OsStr::from_bytes(&not_utf8));
    let refused = signatory_from_environment();
    std::env::remove_var("ODS_ADMIN_SEED");
    let error = refused.expect_err("a set, non-UTF-8 seed is refused, not reported unset");
    assert_eq!((error.variable, error.rule), (ConfigVariable::AdminSeed, ConfigRule::NotHex));
    assert!(!error.to_string().contains(&SEED_HEX[..4]), "{error}");
}

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The signatory key holder (LLR-c4bktx): no getter for the secret, a
//! hand-written `Debug` that shows only the start of the public key.

use org_io::custody::{parse_seed, SignatoryKey};

mod common;

const SEED_HEX: &str = "e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a";

fn windows_of(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    chars.windows(4).map(|window| window.iter().collect()).collect()
}

fn hex_bytes(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&text[index..index + 2], 16).unwrap())
        .collect()
}

// verifies: LLR-c4bktx
#[test]
fn the_key_answers_its_account_and_renders_only_its_start() {
    let key = SignatoryKey::from_seed(parse_seed(SEED_HEX).unwrap()).unwrap();
    let account = key.account_id();
    let expected = subxt_signer::sr25519::Keypair::from_secret_key(
        <[u8; 32]>::try_from(hex_bytes(SEED_HEX)).unwrap(),
    )
    .unwrap()
    .public_key()
    .0;
    assert_eq!(account.0, expected);
    let rendered = format!("{key:?}");
    let start: String = expected[..4].iter().map(|byte| format!("{byte:02x}")).collect();
    assert_eq!(rendered, format!("SignatoryKey(account {start}..)"));
}

// verifies: LLR-c4bktx
#[test]
fn neither_the_seed_nor_the_key_renders_any_part_of_the_secret() {
    let seed = parse_seed(SEED_HEX).unwrap();
    assert_eq!(format!("{seed:?}"), "SeedBytes(..)");
    let key = SignatoryKey::from_seed(seed).unwrap();
    let rendered = format!("{key:?}");
    for window in windows_of(SEED_HEX) {
        assert!(!rendered.contains(&window), "Debug renders part of the seed: {rendered}");
    }
}

// LLR-rgdx22, the `connect` half: built without `dev-seed`, `OrgIo::connect`
// reads no seed and refuses with the development-only message before any
// connection. The address is unreachable on purpose: a build that tried to
// connect would fail differently.
// verifies: LLR-rgdx22
#[cfg(not(feature = "dev-seed"))]
#[tokio::test]
async fn a_build_without_dev_seed_reads_no_seed_and_reports_why() {
    let dir = std::env::temp_dir().join(format!("org-io-no-dev-seed-{}", std::process::id()));
    // A directory an earlier run left is cleared; none there is fine.
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let service = org_node::service::OrgService::new(Vec::new()).unwrap();
    let settings = org_io::ChainSettings {
        ws_url: "ws://127.0.0.1:1".into(),
        contract_h160: [0u8; 20],
        co_signer: None,
        ahead_limit: std::time::Duration::from_secs(1_200),
    };
    let refused = org_io::OrgIo::not_configured(service).connect(settings).await.err().unwrap();
    assert_eq!(
        refused.reason.to_string(),
        "chain not configured: the signing seed (ODS_ADMIN_SEED) is read only by development builds (dev-seed)"
    );
}

// REQ-8zuka3 through LLR-rgdx22: built without `dev-seed`, the unconfigured
// handle's every write and every read says the signing seed is read only by
// development builds, and never tells the user to set ODS_ADMIN_SEED, which
// such a build never reads.
// verifies: LLR-rgdx22
#[cfg(not(feature = "dev-seed"))]
#[tokio::test]
async fn a_build_without_dev_seed_never_asks_for_the_seed_on_a_write_or_a_read() {
    use org_io::chain_read::StateReader;
    use org_io::write::HeldTarget;
    use org_io::WriteReport;
    use org_node::OrgId;

    // The unconfigured handle's founding write, through
    // `write_organisation_to_chain`, is refused at its first step.
    org_io::test_support::MemoryKeychain::shared().install();
    let handle = org_io::OrgIo::not_configured(org_node::service::OrgService::new(Vec::new()).unwrap());
    let proposed = handle.propose_founding_organisation("Alice", "Doe", "alice", "Acme").await.unwrap();
    let WriteReport::Refused { failure } = handle.write_organisation_to_chain(HeldTarget::Founding(proposed.founder)).await.unwrap() else {
        panic!("the founding write is not refused")
    };
    let write = failure.to_string();
    let read = org_io::connect::StateReaderNotConfigured.read_state(OrgId::new([7; 20])).await.unwrap_err().to_string();
    for message in [write, read] {
        assert!(message.contains("read only by development builds"), "{message}");
        assert!(!message.contains("set ODS_CHAIN_WS"), "{message}");
    }
}

// REQ-8zuka3 through LLR-rgdx22: built without
// `dev-seed`, the own-admin check's "not configured" report, and each
// signatory-set read the unconfigured handle refuses, say that the signing
// seed is read only by development builds.
// verifies: LLR-rgdx22
#[cfg(not(feature = "dev-seed"))]
#[tokio::test]
async fn a_build_without_dev_seed_says_why_the_own_admin_check_is_not_configured() {
    // The one `signatory_set` read needs no `AccountId`, only the `OrgId`.
    use org_node::OrgId;
    use org_io::signatory::{OwnAdminError, SignatorySetNotConfigured, SignatorySetReader};

    let dir = std::env::temp_dir().join(format!("org-io-own-admin-not-configured-{}", std::process::id()));
    // A directory an earlier run left is cleared; none there is fine.
    let _ = std::fs::remove_dir_all(&dir);
    let handle = org_io::OrgIo::open(&dir, org_io::transport::TransportMode::Loopback).unwrap();

    let refused = handle.is_own_admin(OrgId::new([7; 20])).await.unwrap_err();
    assert_eq!(refused, OwnAdminError::NotConfigured);
    let read = SignatorySetNotConfigured.signatory_set(OrgId::new([7; 20])).await.unwrap_err();
    for message in [refused.to_string(), read] {
        assert!(message.contains("read only by development builds"), "{message}");
        assert!(!message.contains("set ODS_CHAIN_WS"), "{message}");
    }
}

// LLR-rgdx22, the hand-back: a refused connect returns
// the service it was given, store and Personas intact, with a typed reason,
// so the caller builds its unconfigured handle without opening the store
// again. The co-signer is malformed and the address unreachable, so neither
// build reaches a chain: without `dev-seed` the refusal is the
// development-only one; with it, the seed is unset in this binary (or, were
// it set, the co-signer is refused).
// verifies: LLR-rgdx22
#[tokio::test]
async fn a_refused_connect_hands_back_the_service_it_was_given_and_says_why() {
    use org_io::connect::{ConnectFailed, ConnectFailure};

    let dir = std::env::temp_dir().join(format!("org-io-connect-hand-back-{}", std::process::id()));
    // A directory an earlier run left is cleared; none there is fine.
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // The Persona is made through the handle, as `common::handles::create_named`
    // makes one, its device secret in org-io's in-memory keychain; the
    // hand-back keeps both.
    org_io::test_support::MemoryKeychain::shared().install();
    let handle = org_io::OrgIo::not_configured(org_node::service::OrgService::new(Vec::new()).unwrap());
    let persona_id = common::handles::create_named(&handle, "alice", "Alice", "Smith").await;
    let settings = org_io::ChainSettings {
        ws_url: "ws://127.0.0.1:1".into(),
        contract_h160: [0u8; 20],
        co_signer: Some("not a key".into()),
        ahead_limit: std::time::Duration::from_secs(1_200),
    };

    let refused: Box<ConnectFailed> = handle.connect(settings).await.err().unwrap();
    let reason = &refused.reason;

    #[cfg(not(feature = "dev-seed"))]
    assert!(matches!(reason, ConnectFailure::DevSeedNotBuilt), "{reason}");
    #[cfg(feature = "dev-seed")]
    assert!(matches!(reason, ConnectFailure::SeedNotSet | ConnectFailure::Config(_)), "{reason}");
    // The same service: the unconfigured handle is built from it, and its
    // Persona is there and its keys are readable.
    let handle = refused.into_not_configured();
    let listed: Vec<_> = handle.personas().await.into_iter().map(|persona| persona.persona_id).collect();
    assert_eq!(listed, vec![persona_id.clone()]);
    handle.persona_public_keys(&persona_id).await.unwrap();
}

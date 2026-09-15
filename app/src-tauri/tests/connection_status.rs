//! The connection-status projection.
//!
//! Requirements carried here: REQ-e4ah9h (the verdict and the endpoint are one
//! fact and cannot disagree), REQ-bvx4nh (the endpoint reported is the one the
//! running configuration was built from, never re-read from the environment —
//! and, since the 2026-09-14 amendment, the data directory on the same terms),
//! REQ-645jq9 (the transport mode is reported).

use std::path::Path;

use ods_poc_lib::policy::{self, ChainEndpoint, TransportModeName};

fn endpoint() -> ChainEndpoint {
    ChainEndpoint {
        ws_url: "ws://127.0.0.1:9944".to_string(),
        contract_h160: "0x".to_string() + &"cd".repeat(20),
    }
}

// verifies: REQ-e4ah9h
#[test]
fn configured_chain_reports_its_endpoint_and_contract() {
    let ep = endpoint();
    let s = policy::connection_status_from_state(
        Path::new("/var/lib/ods"),
        Some(&ep),
        TransportModeName::Networked,
    );
    assert!(s.chain_configured);
    assert_eq!(s.chain_ws.as_deref(), Some("ws://127.0.0.1:9944"));
    assert_eq!(s.contract_h160, Some(ep.contract_h160));
    assert_eq!(s.data_dir, "/var/lib/ods");
}

// verifies: REQ-e4ah9h
#[test]
fn unconfigured_chain_reports_neither() {
    // The verdict and the endpoint come from one Option, so there is no state
    // in which the verdict is false and an endpoint is still reported. The
    // requirement is about observable behaviour, so it is asserted anyway: a
    // later edit could reintroduce two independent fields.
    let s = policy::connection_status_from_state(
        Path::new("/var/lib/ods"),
        None,
        TransportModeName::Networked,
    );
    assert!(!s.chain_configured);
    assert_eq!(s.chain_ws, None);
    assert_eq!(s.contract_h160, None);
}

// verifies: REQ-bvx4nh
#[test]
fn absent_chain_reports_false_and_no_endpoint_fields_whatever_the_ambient_env() {
    // The abnormal-input case for REQ-bvx4nh's FIRST clause: handed no built
    // chain, the projection reports chain_configured false and both endpoint
    // fields absent. It asserts that under whatever the ambient environment
    // happens to hold, which it reads only to quote in the failure message.
    //
    // What this test does NOT establish: it does NOT set ODS_CHAIN_WS, so it
    // does not demonstrate the second clause — that the endpoint is never
    // re-read from the process environment. Cargo runs integration tests as
    // threads of ONE process, so a test that mutated the environment would race
    // every other test in this binary. That trade is deliberate and is recorded
    // in the plan. The second clause rests instead on the signature of
    // `connection_status_from_state`: it takes all three inputs as parameters
    // and no environment access is reachable from it.
    let ambient = std::env::var("ODS_CHAIN_WS").ok();
    let s = policy::connection_status_from_state(
        Path::new("/var/lib/ods"),
        None,
        TransportModeName::Networked,
    );
    assert!(!s.chain_configured, "no chain was built, so none may be reported");
    assert_eq!(
        s.chain_ws, None,
        "the environment ({ambient:?}) must not be able to supply an endpoint"
    );
    assert_eq!(s.contract_h160, None);
}

// verifies: REQ-bvx4nh
#[test]
fn endpoint_comes_from_the_built_configuration() {
    // Two different built configurations must project two different endpoints:
    // the reported value tracks its argument and nothing else.
    let a = ChainEndpoint {
        ws_url: "ws://alpha:9944".to_string(),
        contract_h160: "0x".to_string() + &"11".repeat(20),
    };
    let b = ChainEndpoint {
        ws_url: "ws://beta:9944".to_string(),
        contract_h160: "0x".to_string() + &"22".repeat(20),
    };
    let sa = policy::connection_status_from_state(
        Path::new("/d"),
        Some(&a),
        TransportModeName::Networked,
    );
    let sb = policy::connection_status_from_state(
        Path::new("/d"),
        Some(&b),
        TransportModeName::Networked,
    );
    assert_eq!(sa.chain_ws.as_deref(), Some("ws://alpha:9944"));
    assert_eq!(sb.chain_ws.as_deref(), Some("ws://beta:9944"));
    assert_ne!(sa.contract_h160, sb.contract_h160);
}

// verifies: REQ-bvx4nh
#[test]
fn data_dir_comes_from_the_built_configuration() {
    // REQ-bvx4nh's data-directory clause, normal case. The handler no longer
    // re-derives the directory from ODS_DATA_DIR / app_data_dir(); it reports
    // the directory the store was actually opened in. Two materially different
    // paths are projected so that a constant — of any value — fails: the pair
    // is asserted equal to its own argument AND unequal to the other.
    let sa = policy::connection_status_from_state(
        Path::new("/var/lib/ods"),
        None,
        TransportModeName::Networked,
    );
    let sb = policy::connection_status_from_state(
        Path::new("/home/operator/.local/share/ods-poc"),
        None,
        TransportModeName::Networked,
    );
    assert_eq!(sa.data_dir, "/var/lib/ods");
    assert_eq!(sb.data_dir, "/home/operator/.local/share/ods-poc");
    assert_ne!(
        sa.data_dir, sb.data_dir,
        "the reported directory must track its argument, not a constant"
    );

    // And it is independent of the other two inputs: the same directory is
    // reported whether or not a chain was built, and in either transport mode.
    let ep = endpoint();
    let with_chain = policy::connection_status_from_state(
        Path::new("/var/lib/ods"),
        Some(&ep),
        TransportModeName::Loopback,
    );
    assert_eq!(with_chain.data_dir, "/var/lib/ods");
}

// verifies: REQ-bvx4nh
#[test]
fn abnormal_data_dir_paths_are_reported_verbatim() {
    // REQ-bvx4nh's data-directory clause, abnormal input. A path that is
    // empty, non-ASCII, relative, or very long is reported exactly as the
    // running configuration holds it — never normalised, absolutised, or
    // substituted with a fallback such as policy::DEV_DATA_DIR. Substituting
    // here is precisely the failure the clause was added to forbid: the
    // operator would be shown a directory the store was not opened in.
    let long_path = "/".to_string() + &"o".repeat(4096);
    let cases: [&str; 6] = [
        "",
        "/var/lib/ods-Ωμέγα/日本語/naïve",
        "relative/not/absolute",
        "/path with spaces/and\ttab",
        "/tmp/ods-poc-not-the-dev-default",
        &long_path,
    ];
    for raw in cases {
        let s = policy::connection_status_from_state(
            Path::new(raw),
            None,
            TransportModeName::Networked,
        );
        assert_eq!(
            s.data_dir, raw,
            "{raw:?} must be reported verbatim, not normalised or substituted"
        );
    }

    // The empty case deserves its own statement: it must stay empty rather
    // than becoming "." or the development fallback.
    let empty = policy::connection_status_from_state(
        Path::new(""),
        None,
        TransportModeName::Networked,
    );
    assert!(empty.data_dir.is_empty(), "an empty path stays empty");
    assert_ne!(empty.data_dir, policy::DEV_DATA_DIR);
}

// verifies: REQ-645jq9
#[test]
fn networked_transport_is_reported() {
    assert_eq!(
        policy::transport_mode_from(None),
        TransportModeName::Networked
    );
    let s = policy::connection_status_from_state(
        Path::new("/d"),
        None,
        policy::transport_mode_from(Some("networked")),
    );
    assert_eq!(s.transport_mode, TransportModeName::Networked);
}

// verifies: REQ-645jq9
#[test]
fn loopback_transport_is_reported() {
    assert_eq!(
        policy::transport_mode_from(Some("loopback")),
        TransportModeName::Loopback
    );
    let s = policy::connection_status_from_state(
        Path::new("/d"),
        None,
        policy::transport_mode_from(Some("loopback")),
    );
    assert_eq!(s.transport_mode, TransportModeName::Loopback);
}

// verifies: REQ-645jq9
#[test]
fn transport_mode_serialises_lowercase() {
    // The frontend types this field as 'networked' | 'loopback'; the wire form
    // is part of the requirement, not an implementation detail.
    let s = policy::connection_status_from_state(
        Path::new("/d"),
        None,
        TransportModeName::Loopback,
    );
    let v = serde_json::to_value(&s).expect("ConnectionStatus serialises");
    assert_eq!(v["transport_mode"], serde_json::json!("loopback"));

    let s = policy::connection_status_from_state(
        Path::new("/d"),
        None,
        TransportModeName::Networked,
    );
    let v = serde_json::to_value(&s).expect("ConnectionStatus serialises");
    assert_eq!(v["transport_mode"], serde_json::json!("networked"));
}

// verifies: REQ-645jq9
#[test]
fn unknown_transport_value_is_networked() {
    // Abnormal input: anything that is not the literal "loopback" is Networked,
    // which preserves the shipped behaviour exactly. A typo must not silently
    // produce a third, unreported mode.
    for raw in ["", "LOOPBACK", "loop back", "networked", "nonsense", "0"] {
        assert_eq!(
            policy::transport_mode_from(Some(raw)),
            TransportModeName::Networked,
            "{raw:?} must resolve to Networked"
        );
    }
}

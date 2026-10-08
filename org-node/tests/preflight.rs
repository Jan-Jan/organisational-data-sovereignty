#![cfg(feature = "transport")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Hermetic exercises of the transport preflight check: loopback PASS;
//! networked is only asserted to be BOUNDED — not hang — because it uses the
//! real presets::N0 builder whose relay reachability is environment-dependent.
//! The chain check moved to org-io with the chain connection (change
//! `worktree-org-io-create`).
use std::time::Duration;

use org_node::DeviceSeed;
use org_node::preflight::check_transport;
use org_node::transport::TransportMode;

#[tokio::test]
async fn transport_loopback_check_passes() {
    let device = DeviceSeed::from([7u8; 32]).signing_keypair();
    let res = check_transport(&device, TransportMode::Loopback, Duration::from_secs(5)).await;
    assert!(res.ok, "loopback transport check should pass: {res:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn transport_networked_check_is_bounded() {
    // The Networked check uses presets::N0 (real n0 relays); whether they are
    // reachable is environment-dependent, so we assert only that the check is
    // BOUNDED — it returns within ~2x its own timeout rather than hanging.
    // Either ok or !ok is acceptable; hanging is not.
    let device = DeviceSeed::from([42u8; 32]).signing_keypair();
    let inner_timeout = Duration::from_secs(5);
    let res = tokio::time::timeout(
        inner_timeout * 2 + Duration::from_secs(5),
        check_transport(&device, TransportMode::Networked, inner_timeout),
    )
    .await;
    assert!(res.is_ok(), "check_transport(Networked) hung beyond its timeout");
    let res = res.unwrap();
    assert_eq!(
        res.name.split('.').next(),
        Some("transport"),
        "unexpected check name: {res:?}"
    );
    eprintln!("networked check: ok={} {}", res.ok, res.detail);
}

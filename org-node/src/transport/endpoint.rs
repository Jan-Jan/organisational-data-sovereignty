//! OrgEndpoint: an iroh endpoint whose EndpointId is the device's DevicePublicKey.
//!
//! The QUIC handshake authenticates the remote endpoint's ed25519 key, so
//! `recv_one` returns the CRYPTOGRAPHICALLY AUTHENTICATED remote `DevicePublicKey`.
//! Authentication proves key custody, not membership; org-node's receive paths
//! compare the key with nothing (owner ruling, 2026-10-05).
use iroh::{
    EndpointAddr, EndpointId, RelayMode, TransportAddr,
    endpoint::{BindOpts, Connection, presets},
};
#[cfg(feature = "test-support")]
use iroh::{RelayMap, address_lookup::MemoryLookup};
use org_members::DevicePublicKey;

use crate::keys::SigningKeypair;
use crate::transport::{
    ALPN, MAX_FRAME, TransportError, TransportMode,
    wire::{WireMessage, decode_body, encode_frame},
};

/// An iroh endpoint bound to a device's ed25519 key.
///
/// Its `EndpointId` equals the device's `DevicePublicKey`, so successfully
/// completing the QUIC handshake proves device-secret custody.
pub struct OrgEndpoint {
    inner: iroh::Endpoint,
    device_key: DevicePublicKey,
}

impl OrgEndpoint {
    /// Bind an endpoint using `device`'s ed25519 seed as the iroh identity.
    ///
    /// Equivalent to `bind_with_mode(device, TransportMode::Loopback)`.
    ///
    /// Binds loopback only — `127.0.0.1:0` always, and `[::1]:0` where the
    /// host has IPv6 — with relay disabled and no discovery service
    /// configured. Both families are named because naming one replaces only
    /// its own family's default; see [`bind_with_mode`].
    ///
    /// Binding loopback rather than the wildcard (`0.0.0.0`, `[::]`) is what
    /// makes [`node_addr_for_dial`] return an address a peer can actually
    /// dial: the wildcard is not dialable, and an endpoint bound to it
    /// advertises whatever non-loopback address the host happens to have.
    ///
    /// **This was not true until 2026-10-03.** The builder named no bind
    /// address at all, so iroh bound the wildcard and the endpoint offered its
    /// LAN address to peers — the whole of PR-d4nye8. The paragraph above
    /// described the intended behaviour and was read for months as describing
    /// the actual behaviour, which is why REQ-db6s7q now states it where a
    /// gate can see it.
    ///
    /// [`node_addr_for_dial`]: OrgEndpoint::node_addr_for_dial
    /// [`bind_with_mode`]: OrgEndpoint::bind_with_mode
    pub async fn bind(device: &SigningKeypair) -> Result<Self, TransportError> {
        Self::bind_with_mode(device, TransportMode::Loopback).await
    }

    /// Bind an endpoint with an explicit [`TransportMode`].
    ///
    /// - [`TransportMode::Loopback`]: relay disabled, binds `127.0.0.1:0` and,
    ///   where the host supports it, `[::1]:0`. Used by offline tests and
    ///   same-machine demo runs.
    ///
    ///   **Bound and advertised addresses are loopback only** (REQ-db6s7q,
    ///   asserted by the tests carrying that ID). Confinement to this machine
    ///   additionally requires the relay to stay disabled, which no gated test
    ///   can observe — a relay home is acquired only after `online()`, so an
    ///   assertion made at bind time cannot see one. Treat `RelayMode::Disabled`
    ///   on this arm as load-bearing and unverified: changing it silently
    ///   un-confines the mode.
    /// - [`TransportMode::Networked`]: uses `presets::N0` (n0 relay servers +
    ///   DNS/Pkarr address discovery).  Binds on all interfaces (default iroh
    ///   bind).  Required for two laptops communicating across the internet.
    pub async fn bind_with_mode(
        device: &SigningKeypair,
        mode: TransportMode,
    ) -> Result<Self, TransportError> {
        let sk = iroh::SecretKey::from_bytes(device.device_seed().expose_secret());
        let inner = match mode {
            // `clear_ip_transports()` then two explicit loopback binds, rather
            // than the builder's defaults. iroh pre-configures a wildcard
            // socket per address family (`0.0.0.0` and `[::]`), and naming a
            // bind address only replaces the default for ITS OWN family — so
            // binding `127.0.0.1` alone would leave `[::]` listening on every
            // interface.
            //
            // *Corrected 2026-10-03 by the architecture tooth's falsifiability
            // sweep.* This comment used to say that naming both families is
            // what makes the "loopback only" claim true, "measured by a
            // mutation that drops the `[::1]` bind and watches `[::]`
            // reappear". **That mutation was never run.** What was measured
            // (verification record for `f635acc`) was dropping
            // `clear_ip_transports()` AND the `[::1]` bind together. Dropping
            // the `[::1]` bind alone, with `clear_ip_transports()` still in
            // place, was run on 2026-10-03 and is **green**: the transports
            // are already cleared, so no `[::]` reappears and the endpoint
            // stays loopback-only on IPv4 alone. The sentence credited a
            // measurement nobody had made.
            //
            // Putting the five measurements together: confinement needs every
            // NAMED address to be a loopback one — both address mutations,
            // `127.0.0.1:0` -> `0.0.0.0:0` and `[::1]:0` -> `[::]:0`, redden
            // `loopback_mode_binds_and_advertises_loopback_only` — and it
            // needs the pre-configured wildcards gone, for which
            // `clear_ip_transports()` and naming both families are **two
            // redundant mechanisms, either sufficient alone**. That is why
            // dropping either one by itself is green and dropping both is
            // red. What the second bind buys beyond that redundancy is
            // dual-stack reach, not confinement.
            //
            // `clear_ip_transports()` is NOT load-bearing on iroh 0.98.2 —
            // two user-defined binds already override both family defaults,
            // and dropping the call alone leaves the tests green and the
            // property intact. It is kept as defence in depth against a later
            // edit removing one of the two binds, and because it is the form
            // iroh's own documented example uses. Review rounds 3 and 4 both
            // measured this; the comment said "clearing first and naming both
            // is what makes the claim true", which credited it with more than
            // measurement supports.
            //
            // The missing bind address is PR-d4nye8, absent from this builder
            // since the file was written and verified by
            // `loopback_mode_binds_and_advertises_loopback_only`.
            //
            // The IPv6 bind is NOT required, and that asymmetry is deliberate.
            // `BindOpts::is_required` defaults to true, which would abort the
            // whole endpoint when `[::1]` cannot be bound — an IPv6-disabled
            // kernel, a minimal container, a locked-down CI runner. iroh's own
            // pre-configured `[::]` bind, which this replaces, is documented as
            // allowed to fail, so taking the default here would have narrowed
            // the set of hosts this crate runs on while fixing a defect whose
            // whole lesson is not to depend on the host silently. IPv4 loopback
            // stays required: without it there is no usable Loopback transport
            // at all, and failing closed is correct.
            TransportMode::Loopback => iroh::Endpoint::builder(presets::Minimal)
                .relay_mode(RelayMode::Disabled)
                .clear_ip_transports()
                .bind_addr("127.0.0.1:0")
                .map_err(|e| TransportError::Bind(e.to_string()))?
                .bind_addr_with_opts("[::1]:0", BindOpts::default().set_is_required(false))
                .map_err(|e| TransportError::Bind(e.to_string()))?
                .secret_key(sk)
                .alpns(vec![ALPN.to_vec()])
                .bind()
                .await
                .map_err(|e| TransportError::Bind(e.to_string()))?,
            TransportMode::Networked => {
                // presets::N0 configures:
                //   - n0's relay servers (RelayMode::Default via default_relay_mode())
                //   - PkarrPublisher to iroh.link (publishes this node's address)
                //   - DnsAddressLookup via iroh.link (resolves peer EndpointIds)
                // Together these allow two endpoints on different networks to find
                // each other purely by EndpointId.
                iroh::Endpoint::builder(presets::N0)
                    .secret_key(sk)
                    .alpns(vec![ALPN.to_vec()])
                    .bind()
                    .await
                    .map_err(|e| TransportError::Bind(e.to_string()))?
            }
        };
        Ok(Self {
            inner,
            device_key: device.device_key().map_err(|e| TransportError::Bind(format!("DevicePublicKey: {e}")))?,
        })
    }

    /// Bind a Networked-style endpoint whose ONLY relay is `relay_map` and
    /// whose peer-address resolution comes from `lookup`, with direct UDP
    /// paths filtered out so traffic is forced through the relay.
    ///
    /// This mirrors the `TransportMode::Networked` builder (same ALPN, same
    /// secret-key-from-seed) but swaps `presets::N0`'s real n0 relay + DNS
    /// discovery for an in-process relay and an in-memory address lookup.
    /// It exists only to make the Networked dial-by-`EndpointId` path
    /// hermetically testable; it is never used in production.
    ///
    /// Usage: bind both endpoints with a shared `MemoryLookup`, call
    /// [`online`](iroh::Endpoint::online) on each, seed the lookup with each
    /// endpoint's `addr()`, then dial with [`send_to_id`].
    ///
    /// `ca_roots_config(insecure_skip_verify())` is required because
    /// `iroh::test_utils::run_relay_server()` serves a self-signed TLS cert;
    /// without it `online()` never resolves (the relay TLS handshake fails).
    /// This is exactly what iroh's own relay integration tests do. It is
    /// test-only (gated behind `test-support`) and never compiled into
    /// production builds.
    ///
    /// [`send_to_id`]: OrgEndpoint::send_to_id
    #[cfg(feature = "test-support")]
    pub async fn bind_with_relay(
        device: &SigningKeypair,
        relay_map: RelayMap,
        lookup: MemoryLookup,
    ) -> Result<Self, TransportError> {
        let sk = iroh::SecretKey::from_bytes(device.device_seed().expose_secret());
        let inner = iroh::Endpoint::builder(presets::Minimal)
            .relay_mode(RelayMode::Custom(relay_map))
            .address_lookup(lookup)
            .addr_filter(iroh::address_lookup::AddrFilter::relay_only())
            .ca_roots_config(iroh::tls::CaRootsConfig::insecure_skip_verify())
            .secret_key(sk)
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await
            .map_err(|e| TransportError::Bind(e.to_string()))?;
        Ok(Self {
            inner,
            device_key: device.device_key().map_err(|e| TransportError::Bind(format!("DevicePublicKey: {e}")))?,
        })
    }

    /// This endpoint's device key (equal to its iroh `EndpointId`).
    pub fn device_key(&self) -> DevicePublicKey {
        self.device_key
    }

    /// The dialable address (EndpointId + current direct bound socket addresses).
    ///
    /// Built from [`iroh::Endpoint::id`] plus the UDP sockets returned by
    /// [`iroh::Endpoint::bound_sockets`].  This is reliable for loopback/LAN
    /// connections immediately after [`bind`] — `Endpoint::addr()` relies on
    /// async network-path discovery, which may not have run yet.
    ///
    /// Use this for out-of-band address exchange before calling [`send`].
    ///
    /// [`bind`]: OrgEndpoint::bind
    /// [`send`]: OrgEndpoint::send
    pub fn node_addr_for_dial(&self) -> EndpointAddr {
        EndpointAddr::from_parts(
            self.inner.id(),
            self.inner
                .bound_sockets()
                .into_iter()
                .map(TransportAddr::Ip),
        )
    }

    /// Access the raw iroh `Endpoint` (for tests / advanced callers).
    pub fn inner(&self) -> &iroh::Endpoint {
        &self.inner
    }

    /// Dial `peer` by its full `EndpointAddr`, open a bidirectional stream, and
    /// send one framed [`WireMessage`].
    ///
    /// Used in [`TransportMode::Loopback`]: the `EndpointAddr` carries explicit
    /// socket addresses (the caller's, for the peer) so iroh can connect
    /// without discovery.
    ///
    /// Waits for the send stream to be fully flushed before returning.  The
    /// connection is then explicitly closed (code 0) so the remote's
    /// `Connection::closed()` resolves promptly.
    pub async fn send(
        &self,
        peer: impl Into<EndpointAddr>,
        msg: &WireMessage,
    ) -> Result<(), TransportError> {
        self.send_conn(peer.into(), msg).await
    }

    /// Dial `peer` purely by its `EndpointId` (for [`TransportMode::Networked`]).
    ///
    /// `EndpointId` implements `Into<EndpointAddr>`, so iroh will attempt to
    /// resolve the peer's current address via the configured address-lookup
    /// services (Pkarr DNS) and relay through n0's relay servers if no direct
    /// path is available.  This works across NATs and firewalls as long as both
    /// endpoints have reached the relay.
    pub async fn send_to_id(
        &self,
        peer_id: EndpointId,
        msg: &WireMessage,
    ) -> Result<(), TransportError> {
        self.send_conn(peer_id.into(), msg).await
    }

    /// Shared send implementation — connects to `addr`, writes the message,
    /// and closes the connection.
    async fn send_conn(
        &self,
        addr: EndpointAddr,
        msg: &WireMessage,
    ) -> Result<(), TransportError> {
        let conn = self
            .inner
            .connect(addr, ALPN)
            .await
            .map_err(|e| TransportError::Connect(e.to_string()))?;
        let (mut send, _recv) = conn
            .open_bi()
            .await
            .map_err(|e| TransportError::Stream(e.to_string()))?;
        let framed = encode_frame(msg)?;
        send.write_all(&framed)
            .await
            .map_err(|e| TransportError::Stream(e.to_string()))?;
        send.finish()
            .map_err(|e| TransportError::Stream(e.to_string()))?;
        // stopped() returns once the peer has acknowledged receipt of all sent
        // data (after our finish()), so the message is delivered before we close.
        send.stopped()
            .await
            .map_err(|e| TransportError::Stream(e.to_string()))?;
        // Signal QUIC CONNECTION_CLOSE so the remote's conn.closed() resolves.
        conn.close(0u32.into(), b"done");
        Ok(())
    }

    /// Accept one inbound connection, read one framed [`WireMessage`], and
    /// return it together with the AUTHENTICATED remote device key.
    ///
    /// The remote key is extracted from the TLS certificate presented during
    /// the QUIC handshake — it is cryptographically bound to the peer's secret.
    pub async fn recv_one(&self) -> Result<(DevicePublicKey, WireMessage), TransportError> {
        let incoming = self
            .inner
            .accept()
            .await
            .ok_or_else(|| TransportError::Accept("endpoint closed".into()))?;
        let conn: Connection = incoming
            .await
            .map_err(|e| TransportError::Accept(e.to_string()))?;
        let remote_id: EndpointId = conn.remote_id();
        let verifying = ed25519_dalek::VerifyingKey::from_bytes(remote_id.as_bytes())
            .map_err(|_| TransportError::Malformed)?;
        let remote_key = DevicePublicKey::try_from(verifying).map_err(|_| TransportError::Malformed)?;

        let (_send, mut recv) = conn
            .accept_bi()
            .await
            .map_err(|e| TransportError::Stream(e.to_string()))?;
        // read_to_end gives the entire stream; strip the 4-byte length prefix
        // that encode_frame prepended (kept for framing-level forward compat).
        let raw = recv
            .read_to_end(MAX_FRAME + 4)
            .await
            .map_err(|e| TransportError::Stream(e.to_string()))?;
        let payload = if raw.len() >= 4 { &raw[4..] } else { &raw[..] };
        let msg = decode_body(payload)?;
        Ok((remote_key, msg))
    }
}

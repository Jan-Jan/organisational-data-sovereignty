//! OrgEndpoint: an iroh endpoint whose EndpointId is a Device's public key
//! (SDD-8uyg4s), bound from that
//! Device's OS keychain item (LLR-d5y7ks).
//!
//! The QUIC handshake authenticates the remote endpoint's ed25519 key, so
//! `recv_one` and `recv_body` return the CRYPTOGRAPHICALLY AUTHENTICATED
//! remote `DevicePublicKey` (LLR-v873fx). Authentication proves key custody,
//! not membership; org-node's receive path compares the key with the record
//! (S3a's member-sender rule): org-io hands it the key with the body.
//!
//! The module is crate-private in a production build (`transport` re-exports its items only under
//! `test-support`); the methods only the transport's tests call are compiled
//! only there.
use iroh::{
    EndpointAddr, EndpointId, RelayMode,
    endpoint::{BindOpts, Connection, SendStream, presets},
};
use std::future::Future;
use std::task::{Context, Waker};
use std::time::Duration;
#[cfg(feature = "test-support")]
use iroh::{RelayMap, TransportAddr, address_lookup::MemoryLookup};
use org_node::DevicePublicKey;
#[cfg(feature = "test-support")]
use org_node::wire::{WireMessage, decode_message};

#[cfg(feature = "test-support")]
use super::frame::encode_frame;
use super::frame::{MAX_STREAM_READ, body_of, frame_body};
use super::peer_addr::PeerAddr;
use super::{ALPN, TransportError, TransportMode, Verdict};
use crate::keychain;

/// How `bind_from_keychain` builds the endpoint: one of the two transport
/// modes, or, in test builds only, a Networked-style endpoint whose only
/// relay and address lookup are in-process (`Relay`).
#[derive(Clone)]
pub enum BindPlan {
    /// [`TransportMode::Loopback`].
    Loopback,
    /// [`TransportMode::Networked`].
    Networked,
    /// A Networked-style endpoint whose ONLY relay is the `RelayMap` and
    /// whose peer-address resolution comes from the `MemoryLookup`, with
    /// direct UDP paths filtered out so traffic is forced through the relay.
    /// It swaps `presets::N0`'s real n0 relay + DNS discovery for an
    /// in-process relay and an in-memory address lookup, to make the
    /// Networked dial-by-`EndpointId` path hermetically testable; never used
    /// in production.
    ///
    /// Usage: bind both endpoints with a shared `MemoryLookup`, call
    /// [`online`](OrgEndpoint::online) on each, seed the lookup with each
    /// endpoint's `inner().addr()`, then dial with [`send_to_id`](OrgEndpoint::send_to_id).
    ///
    /// `ca_roots_config(insecure_skip_verify())` is required because
    /// `iroh::test_utils::run_relay_server()` serves a self-signed TLS cert;
    /// without it `online()` never resolves. This is exactly what iroh's own
    /// relay integration tests do.
    #[cfg(feature = "test-support")]
    Relay(RelayMap, MemoryLookup),
}

impl From<TransportMode> for BindPlan {
    fn from(mode: TransportMode) -> Self {
        match mode {
            TransportMode::Loopback => BindPlan::Loopback,
            TransportMode::Networked => BindPlan::Networked,
        }
    }
}

/// An iroh endpoint bound to a Device's ed25519 key.
///
/// Its `EndpointId` equals the Device's `DevicePublicKey`, so successfully
/// completing the QUIC handshake proves device-secret custody. It holds the
/// Device's secret inside iroh's endpoint, which no production build hands
/// out (`inner` is test-only).
pub struct OrgEndpoint {
    inner: iroh::Endpoint,
    #[cfg(feature = "test-support")]
    device_key: DevicePublicKey,
}

impl OrgEndpoint {
    /// Bind an endpoint for `device_key` from its keychain item (LLR-d5y7ks):
    /// the item's seed is lent once to iroh's secret-key constructor, and its
    /// buffer is dropped and zeroised before the keychain call returns
    /// (`keychain::with_endpoint_seed`, LLR-677fqq); the key is moved into
    /// the endpoint builder for `plan` and named nowhere else in org-io. A
    /// keychain refusal is `TransportError::Keychain`; an item holding
    /// another key's seed is `TransportError::KeychainKeyMismatch`, refused
    /// before anything is bound.
    ///
    /// **Loopback** binds loopback only — `127.0.0.1:0` always, and `[::1]:0`
    /// where the host has IPv6 — with relay disabled and no discovery service
    /// configured (REQ-db6s7q, REQ-2wzfzv). Binding loopback rather than the
    /// wildcard is what makes [`node_addr_for_dial`] return an address a peer
    /// can actually dial: the wildcard is not dialable, and an endpoint bound
    /// to it advertises whatever non-loopback address the host happens to
    /// have. Confinement to this machine additionally requires the relay to
    /// stay disabled, which no gated test can observe — a relay home is
    /// acquired only after `online()`. Treat `RelayMode::Disabled` on this
    /// arm as load-bearing and unverified.
    ///
    /// **Networked** uses `presets::N0` (n0 relay servers + DNS/Pkarr address
    /// discovery) and binds on all interfaces (default iroh bind). Required
    /// for two laptops communicating across the internet.
    ///
    /// [`node_addr_for_dial`]: OrgEndpoint::node_addr_for_dial
    pub async fn bind_from_keychain(device_key: &DevicePublicKey, plan: BindPlan) -> Result<Self, TransportError> {
        let secret_key = keychain::with_endpoint_seed(device_key, iroh::SecretKey::from_bytes)
            .map_err(|refused| TransportError::Keychain(Box::new(refused)))?;
        if secret_key.public().as_bytes() != device_key.as_bytes() {
            return Err(TransportError::KeychainKeyMismatch(Box::new(*device_key)));
        }
        let builder = match plan {
            // `clear_ip_transports()` then two explicit loopback binds, rather
            // than the builder's defaults. iroh pre-configures a wildcard
            // socket per address family (`0.0.0.0` and `[::]`), and naming a
            // bind address only replaces the default for ITS OWN family.
            //
            // Measured on iroh 0.98.2 (org-node's verification records of
            // 2026-10-03): confinement needs every NAMED address to be a
            // loopback one — both address mutations, `127.0.0.1:0` ->
            // `0.0.0.0:0` and `[::1]:0` -> `[::]:0`, redden
            // `loopback_mode_binds_and_advertises_loopback_only` — and it
            // needs the pre-configured wildcards gone, for which
            // `clear_ip_transports()` and naming both families are two
            // redundant mechanisms, either sufficient alone. That is why
            // dropping either one by itself is green and dropping both is
            // red. `clear_ip_transports()` is kept as defence in depth and
            // because it is the form iroh's own documented example uses.
            //
            // The IPv6 bind is NOT required, deliberately (REQ-2wzfzv):
            // `BindOpts::is_required` defaults to true, which would abort the
            // whole endpoint when `[::1]` cannot be bound (an IPv6-disabled
            // kernel, a minimal container). IPv4 loopback stays required:
            // without it there is no usable Loopback transport at all, and
            // failing closed is correct.
            BindPlan::Loopback => iroh::Endpoint::builder(presets::Minimal)
                .relay_mode(RelayMode::Disabled)
                .clear_ip_transports()
                .bind_addr("127.0.0.1:0")
                .map_err(bind_error)?
                .bind_addr_with_opts("[::1]:0", BindOpts::default().set_is_required(false))
                .map_err(bind_error)?,
            // presets::N0 configures n0's relay servers, a Pkarr publisher
            // and a DNS address lookup via iroh.link: together these let two
            // endpoints on different networks find each other by EndpointId.
            BindPlan::Networked => iroh::Endpoint::builder(presets::N0),
            #[cfg(feature = "test-support")]
            BindPlan::Relay(relay_map, lookup) => iroh::Endpoint::builder(presets::Minimal)
                .relay_mode(RelayMode::Custom(relay_map))
                .address_lookup(lookup)
                .addr_filter(iroh::address_lookup::AddrFilter::relay_only())
                .ca_roots_config(iroh::tls::CaRootsConfig::insecure_skip_verify()),
        };
        let inner = builder
            .secret_key(secret_key)
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await
            .map_err(bind_error)?;
        Ok(Self {
            inner,
            #[cfg(feature = "test-support")]
            device_key: *device_key,
        })
    }

    /// This endpoint's device key (equal to its iroh `EndpointId`; test
    /// builds only).
    #[cfg(feature = "test-support")]
    pub fn device_key(&self) -> DevicePublicKey {
        self.device_key
    }

    /// The dialable address (EndpointId + current direct bound socket
    /// addresses; LLR-wx77j5), built from [`iroh::Endpoint::id`] plus the UDP
    /// sockets returned by [`iroh::Endpoint::bound_sockets`]. Reliable for
    /// loopback/LAN connections immediately after binding —
    /// `Endpoint::addr()` relies on async network-path discovery, which may
    /// not have run yet. Test builds only.
    #[cfg(feature = "test-support")]
    pub fn node_addr_for_dial(&self) -> EndpointAddr {
        EndpointAddr::from_parts(self.inner.id(), self.inner.bound_sockets().into_iter().map(TransportAddr::Ip))
    }

    /// The raw iroh `Endpoint`, test builds only: it holds the device's
    /// ed25519 private key, which no production build hands out (owner
    /// ruling 2026-10-06). Production callers get the four things they read
    /// from it, below: its address, its bound sockets, whether it has
    /// closed, and a wait until it is online.
    #[cfg(feature = "test-support")]
    pub fn inner(&self) -> &iroh::Endpoint {
        &self.inner
    }

    /// Whether the endpoint's address, as iroh currently knows it
    /// (`Endpoint::addr`), names a relay: the endpoint has a relay home.
    pub fn has_relay_home(&self) -> bool {
        self.inner.addr().relay_urls().next().is_some()
    }

    /// The UDP sockets the endpoint is bound to (`Endpoint::bound_sockets`).
    pub fn bound_sockets(&self) -> Vec<std::net::SocketAddr> {
        self.inner.bound_sockets()
    }

    /// Whether the endpoint has closed or started closing: it accepts and
    /// dials nothing more (LLR-d2pzzk as amended 2026-10-15). iroh exposes
    /// the start of a close only as the future `Endpoint::closed`, which
    /// resolves once the close has begun; it is polled once here, with a
    /// waker that wakes nothing, beside `Endpoint::is_closed` (the end).
    pub fn is_closed(&self) -> bool {
        let closing = std::pin::pin!(self.inner.closed());
        self.inner.is_closed() || closing.poll(&mut Context::from_waker(Waker::noop())).is_ready()
    }

    /// Wait until the endpoint is online (`Endpoint::online`).
    pub async fn online(&self) {
        self.inner.online().await
    }

    /// Dial `peer` by its full `EndpointAddr` (Loopback: the address the
    /// caller holds for the peer) and send one framed [`WireMessage`]. Test
    /// builds only.
    #[cfg(feature = "test-support")]
    pub async fn send(&self, peer: impl Into<EndpointAddr>, message: &WireMessage) -> Result<(), TransportError> {
        self.send_framed(peer.into(), &encode_frame(message)?).await
    }

    /// Dial `peer_id` purely by its `EndpointId` (Networked: iroh resolves
    /// the peer's address through its address lookup and relays when no
    /// direct path exists) and send one framed [`WireMessage`]. Test builds
    /// only.
    #[cfg(feature = "test-support")]
    pub async fn send_to_id(&self, peer_id: EndpointId, message: &WireMessage) -> Result<(), TransportError> {
        self.send_framed(peer_id.into(), &encode_frame(message)?).await
    }

    /// Send one already encoded body to `peer`, framed with its length
    /// prefix (org-node's commit builds the bodies; they are sent
    /// unchanged). A body over the bound is refused before dialling.
    pub async fn send_body(&self, peer: impl Into<EndpointAddr>, body: &[u8]) -> Result<(), TransportError> {
        let framed = frame_body(body)?;
        self.send_framed(peer.into(), &framed).await
    }

    /// `send_body` to the Device `device`, dialled purely by the endpoint
    /// identity equal to its Device public key (Networked: iroh's relay and
    /// address lookup resolve the path). Bytes that are no endpoint identity
    /// are `TransportError::Malformed`, refused before dialling.
    pub async fn send_body_to_device(&self, device: &DevicePublicKey, body: &[u8]) -> Result<(), TransportError> {
        match EndpointId::from_bytes(device.as_bytes()) {
            Ok(id) => self.send_body(id, body).await,
            Err(_) => Err(TransportError::Malformed),
        }
    }

    /// `send_body` at the full address `peer` (Loopback: the address the
    /// caller holds for the peer).
    pub async fn send_body_at(&self, peer: &PeerAddr, body: &[u8]) -> Result<(), TransportError> {
        self.send_body(peer.endpoint_addr().clone(), body).await
    }

    /// Connect to `addr`, open one bidirectional stream, write `framed`, wait
    /// for the peer to acknowledge all of it, and close the connection (code
    /// 0) so the remote's `Connection::closed()` resolves promptly.
    async fn send_framed(&self, addr: EndpointAddr, framed: &[u8]) -> Result<(), TransportError> {
        let stream_error = |error: &dyn std::fmt::Display| TransportError::Stream(error.to_string());
        let conn = self.inner.connect(addr, ALPN).await.map_err(|error| TransportError::Connect(error.to_string()))?;
        let (mut send, _recv) = conn.open_bi().await.map_err(|error| stream_error(&error))?;
        send.write_all(framed).await.map_err(|error| stream_error(&error))?;
        send.finish().map_err(|error| stream_error(&error))?;
        // stopped() returns once the peer has acknowledged receipt of all
        // sent data (after our finish()), so the message is delivered before
        // we close.
        send.stopped().await.map_err(|error| stream_error(&error))?;
        conn.close(0u32.into(), b"done");
        Ok(())
    }

    /// Accept one inbound connection, read one framed [`WireMessage`], and
    /// return it with the AUTHENTICATED remote device key (LLR-v873fx). Test
    /// builds only.
    #[cfg(feature = "test-support")]
    pub async fn recv_one(&self) -> Result<(DevicePublicKey, WireMessage), TransportError> {
        let (remote_key, body) = self.recv_body().await?;
        Ok((remote_key, decode_message(&body)?))
    }

    /// Accept one inbound connection and return the AUTHENTICATED remote
    /// device key with the body it sent, as
    /// [`PendingConnection::receive`] does. Test builds only.
    #[cfg(feature = "test-support")]
    pub async fn recv_body(&self) -> Result<(DevicePublicKey, Vec<u8>), TransportError> {
        let (remote_key, body, _unanswered) = self.recv_answerable().await?;
        Ok((remote_key, body))
    }

    /// As [`recv_body`](Self::recv_body), and keep the stream's send half
    /// for the verdict (LLR-unqfj5): dropping the reply unanswered writes
    /// nothing. The connection is bounded by [`CONNECTION_TIME_LIMIT`].
    /// Test builds only: the receive loop takes each connection with
    /// [`next_connection`](Self::next_connection) and reads it in a task of
    /// its own (LLR-8uf9a5).
    #[cfg(feature = "test-support")]
    pub async fn recv_answerable(&self) -> Result<(DevicePublicKey, Vec<u8>, InboundReply), TransportError> {
        self.next_connection().await?.receive(CONNECTION_TIME_LIMIT).await
    }

    /// The next inbound connection, before its handshake; `EndpointClosed`
    /// once the endpoint has closed.
    pub async fn next_connection(&self) -> Result<PendingConnection, TransportError> {
        self.inner.accept().await.map(PendingConnection).ok_or(TransportError::EndpointClosed)
    }
}

/// How long one inbound connection may take, from the start of its
/// handshake through its stream's open to the end of its body, before it is
/// dropped (LLR-8uf9a5, RC-96w7nf). Thirty seconds is several times what a
/// handshake and a body at the 1 MiB bound take over a slow relayed path,
/// and short enough that a peer which stalls holds one of its endpoint's
/// connection slots for half a minute at most.
pub const CONNECTION_TIME_LIMIT: Duration = Duration::from_secs(30);

/// One inbound connection, accepted by the endpoint but not yet handshaken.
pub struct PendingConnection(iroh::endpoint::Incoming);

impl PendingConnection {
    /// Complete the handshake, take the peer's one bidirectional stream and
    /// read its body, all within `limit` (LLR-8uf9a5): past it the
    /// connection is dropped, which closes it, and the result is
    /// `TransportError::TimedOut`. The body comes back with the
    /// AUTHENTICATED remote device key, taken from the key the handshake
    /// authenticated and never from the body (LLR-v873fx), undecoded
    /// (org-node's `check_received` decodes it), and the reply its verdict
    /// goes back through (LLR-unqfj5). At most `MAX_FRAME + 4` bytes are
    /// read from the stream, and a longer stream is a
    /// `TransportError::Stream` (LLR-k2y6nn); the four-byte prefix is
    /// discarded unchecked (LLR-2smrvx). Every connection whose handshake
    /// completes is taken, over a direct path or a relayed one, with no
    /// check of the remote key here: org-node's `check_received` judges the
    /// sender (LLR-wvgec4).
    pub async fn receive(self, limit: Duration) -> Result<(DevicePublicKey, Vec<u8>, InboundReply), TransportError> {
        tokio::time::timeout(limit, self.receive_unbounded()).await.map_err(|_| TransportError::TimedOut(limit))?
    }

    async fn receive_unbounded(self) -> Result<(DevicePublicKey, Vec<u8>, InboundReply), TransportError> {
        let conn: Connection = self.0.await.map_err(|error| TransportError::Accept(error.to_string()))?;
        let remote_id: EndpointId = conn.remote_id();
        let remote_key = DevicePublicKey::parse(remote_id.as_bytes()).map_err(|_| TransportError::Malformed)?;
        let (send, mut recv) = conn.accept_bi().await.map_err(|error| TransportError::Stream(error.to_string()))?;
        let stream = recv.read_to_end(MAX_STREAM_READ).await.map_err(|error| TransportError::Stream(error.to_string()))?;
        Ok((remote_key, body_of(&stream).to_vec(), InboundReply { connection: conn, send }))
    }
}

/// How long an answered connection is held open for the sender to read the
/// verdict and close; a sender that never closes is let go after it.
const VERDICT_LINGER: Duration = Duration::from_secs(10);

/// The send half of one inbound stream, kept for its verdict (LLR-unqfj5).
pub struct InboundReply {
    connection: Connection,
    send: SendStream,
}

impl InboundReply {
    /// Write the one verdict byte and finish the stream; nothing else is
    /// ever written on an inbound stream. The connection is then held until
    /// the sender closes it, or for [`VERDICT_LINGER`], so the byte is not
    /// lost to an early close. A sender gone already is not an error: it
    /// reads silence as a refusal.
    pub async fn answer(mut self, verdict: Verdict) {
        if self.send.write_all(&[verdict.as_byte()]).await.is_ok() && self.send.finish().is_ok() {
            let _ = tokio::time::timeout(VERDICT_LINGER, self.connection.closed()).await;
        }
    }
}

/// A failed bind, as text.
fn bind_error(error: impl std::fmt::Display) -> TransportError {
    TransportError::Bind(error.to_string())
}

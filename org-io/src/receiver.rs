//! The receive step (SDD-aw3nka): `OrgIo::start_receiving`
//! and `OrgIo::next_received`. The step waits for the next body on the
//! receive channel holding no lock on the node (LLR-9arvqj); it takes the
//! node only around org-node's `check_received`, releases it for the one
//! chain read, and takes it again around `commit_checked` and the store
//! change that follows (LLR-cvyyr8). A commit org-node refuses because the
//! record moved while the node was released is checked, read and committed
//! once more. The end of the receive loop is a typed
//! terminal condition, and nothing else is (LLR-szhe7j).

use org_node::revocation::VerifiedAcknowledgement;
use org_node::{Checked, DevicePublicKey, OrgNodeError, ReceiveOutcome};
use rand_core::{CryptoRng, RngCore};

use crate::endpoints::Inbound;
use crate::keychain::KeychainError;
use crate::transport::{TransportError, Verdict};
use crate::{OrgIo, StoreFailure};

/// What `OrgIo::next_received` received: a committed message, or an
/// acknowledgement decided against the store alone. Both carry public data
/// only (an outcome's acknowledgements are signed public statements).
// One value per received message, moved once to the caller: the
// acknowledgement's size costs nothing worth a box.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Received {
    Committed(ReceiveOutcome),
    Acknowledged(VerifiedAcknowledgement),
}

/// Why the receive loop ends (LLR-szhe7j).
#[derive(Debug)]
pub enum TerminalCause {
    /// Every accept task has ended, or none was started: nothing more can
    /// arrive.
    EndpointClosed,
    /// An endpoint could not be bound.
    BindFailed(TransportError),
}

/// Why the receive step returned no message (LLR-szhe7j): the terminal
/// condition, or one of the failures after which the loop goes on. Public
/// data only: org-node's, the keychain's, the store's and the transport's
/// typed refusals.
#[derive(Debug)]
pub enum ReceiveError {
    /// The receive loop ends.
    Terminal(TerminalCause),
    /// One inbound connection failed before it carried a body.
    Connection(TransportError),
    /// org-node refused the body, in its check or its commit.
    Refused(OrgNodeError),
    /// The chain read failed, or org-node's parse refused the state read;
    /// nothing was committed.
    Chain(OrgNodeError),
    /// The commit was made, but its store change was not written
    /// (LLR-murjr6, LLR-amv4mb).
    Store(StoreFailure),
    /// The keychain refused a secret the commit signs with (REQ-6d9xv6);
    /// nothing was committed. Boxed, as the transport's: it names a Device
    /// public key, a parsed point, and the error travels on every receive.
    Keychain(Box<KeychainError>),
}

impl std::fmt::Display for ReceiveError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Terminal(TerminalCause::EndpointClosed) => write!(formatter, "the receive loop ended: every endpoint closed"),
            Self::Terminal(TerminalCause::BindFailed(error)) => write!(formatter, "the receive loop ended: {error}"),
            Self::Connection(error) => write!(formatter, "an inbound connection failed: {error}"),
            Self::Refused(error) => write!(formatter, "{error}"),
            Self::Chain(error) => write!(formatter, "the chain read failed; nothing committed: {error}"),
            Self::Store(failure) => write!(formatter, "the message was committed, but {failure}"),
            Self::Keychain(error) => write!(formatter, "the keychain refused; nothing committed: {error}"),
        }
    }
}

impl std::error::Error for ReceiveError {}

impl OrgIo {
    /// Start receiving (LLR-z35vbh): bind the endpoint of every Persona the
    /// node holds, each from its keychain item, and accept on every one of
    /// them into the one receive channel `next_received` reads, each body
    /// with the sender the handshake authenticated. Calling it again binds
    /// and accepts on the endpoints of Personas created since. A bind that
    /// fails is `TerminalCause::BindFailed` (LLR-szhe7j); the endpoints bound
    /// before it keep accepting.
    pub async fn start_receiving(&self) -> Result<(), ReceiveError> {
        // The flag first, then the listing (review round 2, A6): a Persona
        // `create_persona` makes after the listing then sees the flag and is
        // accepted on there; one made before it is listed. Set after the
        // listing, a Persona created in between was missed by both.
        self.local.endpoints.mark_receiving().await;
        let device_keys: Vec<_> = self.service.lock().await.list_personas().into_iter().map(|persona| persona.device_key).collect();
        self.local
            .endpoints
            .start_accepting(&device_keys)
            .await
            .map_err(|error| ReceiveError::Terminal(TerminalCause::BindFailed(error)))
    }

    /// This handle, dropping each inbound connection that has given no body
    /// within `limit` rather than `CONNECTION_TIME_LIMIT` (LLR-8uf9a5; test
    /// builds only: a test of the limit need not wait half a minute).
    #[cfg(feature = "test-support")]
    pub fn with_connection_time_limit_for_test(mut self, limit: std::time::Duration) -> Self {
        self.local.endpoints.set_connection_time_limit(limit);
        self
    }

    /// Receive one message (LLR-9arvqj, LLR-cvyyr8, LLR-7pj5af): wait, with
    /// no lock on the node, for one body and its authenticated sender; hand
    /// both to org-node's `check_received`; a refusal or an acknowledgement
    /// (decided against the store alone) is returned with no chain read and
    /// no store write; a checked message is read for once, with the node
    /// released, and only then committed (`commit_checked`, signing with the
    /// keychain's secrets) and its store change written, before the outcome
    /// returns. A store change that was not written reloads the Persona from
    /// its file (LLR-murjr6, LLR-amv4mb). A commit refused because the record
    /// moved since the check (`RecordMovedSinceCheck`) is repeated once from
    /// the check.
    ///
    /// The sender is answered with one verdict byte
    /// (LLR-unqfj5): `Verdict::Accepted` when this returns a message,
    /// `Verdict::Refused` for every error, a store write that failed
    /// included.
    pub async fn next_received<R: RngCore + CryptoRng + Send>(&self, rng: &mut R) -> Result<Received, ReceiveError> {
        let Inbound { sender, body, verdict } = self.local.endpoints.next_inbound().await.map_err(|error| match error {
            TransportError::EndpointClosed => ReceiveError::Terminal(TerminalCause::EndpointClosed),
            other => ReceiveError::Connection(other),
        })?;
        let received = self.settle(sender, &body, rng).await;
        // A sender gone already reads silence as a refusal: nothing to do.
        let _ = verdict.send(if received.is_ok() { Verdict::Accepted } else { Verdict::Refused });
        received
    }

    /// `settle_once`; when org-node refuses the commit because the record
    /// its check read moved meanwhile (`RecordMovedSinceCheck`: a local
    /// commit landed while the node was released for the read), the check,
    /// the read and the commit are repeated once, under the same rules, and
    /// that result is returned (org-node's problem report on the stale
    /// check).
    async fn settle<R: RngCore + CryptoRng + Send>(&self, sender: DevicePublicKey, body: &[u8], rng: &mut R) -> Result<Received, ReceiveError> {
        match self.settle_once(sender, body, rng).await {
            Err(ReceiveError::Refused(OrgNodeError::RecordMovedSinceCheck { .. })) => self.settle_once(sender, body, rng).await,
            settled => settled,
        }
    }

    /// Check `body` from `sender`, then read, commit and write as
    /// `next_received` states.
    async fn settle_once<R: RngCore + CryptoRng + Send>(&self, sender: DevicePublicKey, body: &[u8], rng: &mut R) -> Result<Received, ReceiveError> {
        let checked = self.service.lock().await.check_received(sender, body).map_err(ReceiveError::Refused)?;
        let checked = match checked {
            Checked::Acknowledged(verified) => return Ok(Received::Acknowledged(verified)),
            Checked::ReadChain(checked) => checked,
        };
        let chain_state = self.reader.read_state(checked.org_id()).await.map_err(ReceiveError::Chain)?;
        let mut service = self.service.lock().await;
        let mut keychain_refused = None;
        let keychain = &self.local.keychain;
        let committed = service.commit_checked(rng, checked, chain_state, |device_keys| {
            keychain.for_signing_noting_refusal(device_keys, &mut keychain_refused)
        });
        let (outcome, change) = committed.map_err(|error| match (error, keychain_refused) {
            (OrgNodeError::DeviceSecretNotSupplied { .. }, Some(refused)) => ReceiveError::Keychain(Box::new(refused)),
            (error, _) => ReceiveError::Refused(error),
        })?;
        self.local.store(&mut service, change).map_err(ReceiveError::Store)?;
        Ok(Received::Committed(outcome))
    }
}

//! The receive step (SDD-2fc2cq): `OrgIo::start_receiving` and `OrgIo::next_received`. The step
//! waits for the next body on the receive channel holding no lock on the node (LLR-eeh4pq); it
//! takes the node only around org-node's `check_received` and the store write that follows
//! (LLR-kz9hj4), writes exactly one verdict byte (LLR-hhgdk7), and reads no chain and adopts
//! nothing. The end of the receive loop is a typed terminal condition, and nothing else is
//! (LLR-szhe7j). An organisation request's answer goes
//! out after the verdict, through the send path (LLR-898z24).

use org_node::invitation::InviteId;
use org_node::revocation::VerifiedAcknowledgement;
use org_node::wire::peek_admission_request;
use org_node::{AdmissionRequestOutcome, Checked, DevicePublicKey, Epoch, Handle, Name, OrgId, OrgNodeError, RootHash, Surname};

use crate::endpoints::Inbound;
use crate::send::{kept_for, send_all, Delivery, Outgoing, SendError};
use crate::transport::{TransportError, Verdict};
use crate::{OrgIo, StoreFailure};

/// What `OrgIo::next_received` kept or decided (LLR-x36szv): public data only. The field types of
/// `AdmissionRequestKept` are those of `KeptAdmissionRequestSummary`.
#[derive(Debug)]
pub enum Received {
    /// Organisation information, or an answer to this node's organisation request, kept as a
    /// provisional organisation for verification.
    ProvisionalOrganisationReceived { org_id: OrgId, epoch: Epoch, root: RootHash, from: DevicePublicKey },
    /// A first admission request, kept for the Admin.
    AdmissionRequestKept {
        org_id: OrgId,
        invite_id: InviteId,
        name: Name,
        surname: Surname,
        preferred_handles: Vec<Handle>,
        from: DevicePublicKey,
        invitee_name: String,
    },
    /// An organisation request answered: org-node's answer (the full record, or a revoked
    /// requester's notice) sent through the send path, and what became of it.
    OrganisationRequestAnswered { org_id: OrgId, requester: DevicePublicKey, delivery: Delivery },
    /// A revocation notice kept for this node; only a verification acts on it.
    RevocationNoticeKept { org_id: OrgId, from: DevicePublicKey },
    /// A revoked Device's acknowledgement, decided against the store alone.
    Acknowledged(VerifiedAcknowledgement),
}

/// Why the receive loop ends (LLR-szhe7j).
#[derive(Debug)]
pub enum TerminalCause {
    /// Every accept task has ended, or none was started: nothing more can arrive.
    EndpointClosed,
    /// An endpoint could not be bound.
    BindFailed(TransportError),
}

/// Why the receive step returned no message: the terminal condition (LLR-szhe7j), or one of the
/// failures after which the loop goes on. Public data only.
#[derive(Debug)]
pub enum ReceiveError {
    /// The receive loop ends.
    Terminal(TerminalCause),
    /// One inbound connection failed before it carried a body.
    Connection(TransportError),
    /// org-node's check refused the body; nothing was written.
    Refused(OrgNodeError),
    /// org-node's store change was not written; the Persona was reloaded or dropped (LLR-murjr6).
    Store(StoreFailure),
    /// org-node's check kept the body in `receiver`'s store, which is bound to no Organisation
    /// yet (a founding before its proxy): there is no Organisation to report it under. org-io's
    /// own decision, `0x01` written.
    NoOrganisation { receiver: Box<DevicePublicKey> },
}

impl std::fmt::Display for ReceiveError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Terminal(TerminalCause::EndpointClosed) => write!(formatter, "the receive loop ended: every endpoint closed"),
            Self::Terminal(TerminalCause::BindFailed(error)) => write!(formatter, "the receive loop ended: {error}"),
            Self::Connection(error) => write!(formatter, "an inbound connection failed: {error}"),
            Self::Refused(error) => write!(formatter, "{error}"),
            Self::Store(failure) => write!(formatter, "the message was kept, but {failure}"),
            Self::NoOrganisation { receiver } => write!(
                formatter,
                "the message reached Persona {}, which is bound to no Organisation yet",
                hex::encode(receiver.as_bytes())
            ),
        }
    }
}

impl std::error::Error for ReceiveError {}

/// What one body came to: a report for the app, nothing to report (an already-held provisional
/// organisation, a repeated admission request), or a failure.
enum Step {
    Report(Box<Received>),
    Silent,
    Failed(ReceiveError),
    /// An organisation request answered by org-node: its store change is written; the answer
    /// goes out after the verdict, with no lock held.
    Answer { org_id: OrgId, outgoing: Outgoing },
}

impl OrgIo {
    /// Start receiving (LLR-z35vbh): bind the endpoint of every Persona the
    /// node holds, each from its keychain item, and accept on every one of
    /// them into the one receive channel `next_received` reads, each body
    /// with the sender the handshake authenticated. Calling it again binds
    /// and accepts on the endpoints of Personas created since. A bind that
    /// fails is `TerminalCause::BindFailed` (LLR-szhe7j); the endpoints bound
    /// before it keep accepting. Once they accept, the delivery of every
    /// kept removal whose deadline the handle's clock has not reached is
    /// resumed, none twice (LLR-5rxqr8), and a leftover `removed-` file's
    /// deletion, or a deletion that failed at open, is retried hourly
    /// (LLR-3fxxpp).
    pub async fn start_receiving(&self) -> Result<(), ReceiveError> {
        // The flag first, then the listing: a Persona a proposal creates
        // after the listing then sees the flag and is accepted on there; one
        // created before it is listed. Set after the listing, a Persona
        // created in between would be missed by both.
        self.local.endpoints.mark_receiving().await;
        let device_keys: Vec<_> = self.service.lock().await.persona_records().into_iter().map(|persona| persona.device_key).collect();
        self.local
            .endpoints
            .start_accepting(&device_keys)
            .await
            .map_err(|error| ReceiveError::Terminal(TerminalCause::BindFailed(error)))?;
        // The removed stores the handle remembers (those `open` found and those
        // a delivery's start recorded), not a fresh listing (LLR-5rxqr8).
        crate::acknowledge::resume(self.delivery_context(), self.deliveries.0.removed_keys()).await;
        // A deletion left unfinished in a `removed-` file is retried
        // hourly while the handle runs (LLR-3fxxpp).
        crate::retention::start_leftover_retry(self.delivery_context());
        Ok(())
    }

    /// This handle, dropping each inbound connection that has given no body
    /// within `limit` rather than `CONNECTION_TIME_LIMIT` (LLR-8uf9a5; test
    /// builds only: a test of the limit need not wait half a minute).
    #[cfg(feature = "test-support")]
    pub fn with_connection_time_limit_for_test(mut self, limit: std::time::Duration) -> Self {
        self.local.endpoints.set_connection_time_limit(limit);
        self
    }
}

impl OrgIo {
    /// Receive (LLR-kz9hj4, LLR-eeh4pq): wait, with no lock on the node, for a body, the Device
    /// it reached and its authenticated sender; hand them to org-node's check; write the store
    /// change it returns; write one verdict byte; and report what was kept. A body that keeps
    /// nothing new is answered and not reported, and the step waits for the next.
    pub async fn next_received(&self) -> Result<Received, ReceiveError> {
        loop {
            let Inbound { receiver, sender, body, verdict } = self.local.endpoints.next_inbound().await.map_err(|error| match error {
                TransportError::EndpointClosed => ReceiveError::Terminal(TerminalCause::EndpointClosed),
                other => ReceiveError::Connection(other),
            })?;
            let (answer, step) = self.check_and_keep(receiver, sender, &body).await;
            // A sender gone already reads silence as a refusal: nothing to do.
            let _ = verdict.send(answer);
            match step {
                Step::Report(received) => return Ok(*received),
                Step::Silent => continue,
                Step::Failed(error) => return Err(error),
                Step::Answer { org_id, outgoing } => {
                    // From the Persona the request reached, on a new connection, at the
                    // addresses kept for the requester (LLR-898z24, LLR-34tfm8); never on the
                    // inbound connection or to its source address.
                    let requester = outgoing.recipient;
                    let mut report = send_all(&self.local.endpoints, receiver, vec![outgoing]).await;
                    // `send_all` reports one entry per `Outgoing`, so the report holds
                    // exactly one: the fallback is never taken.
                    let delivery = report
                        .per_device
                        .pop()
                        .map_or(Delivery::NotDelivered { reason: SendError::NoAddress }, |(_, delivery)| delivery);
                    return Ok(Received::OrganisationRequestAnswered { org_id, requester, delivery });
                }
            }
        }
    }

    /// org-node's check of `body` in `receiver`'s store and the store write after it, under the
    /// lock (LLR-kz9hj4, LLR-zkybw7); the verdict to write and the step's result.
    async fn check_and_keep(&self, receiver: DevicePublicKey, sender: DevicePublicKey, body: &[u8]) -> (Verdict, Step) {
        let mut service = self.service.lock().await;
        let checked = {
            let mut rng = self.local.rng.lock();
            service.check_received(&mut *rng, &receiver, sender, body)
        };
        let (checked, change) = match checked {
            Ok(checked) => checked,
            Err(refused) => return (Verdict::Refused, Step::Failed(ReceiveError::Refused(refused))),
        };
        if let Some(change) = change {
            if let Err(failure) = self.local.store(&mut service, change) {
                return (Verdict::Refused, Step::Failed(ReceiveError::Store(failure)));
            }
        }
        // org-node's check accepted the body in `receiver`'s store and its change is written, so
        // that store is held and not removed, the only stores `persona_view` refuses. What is left
        // is a store bound to no Organisation yet (a founding before its proxy), which names no
        // Organisation to report the message under.
        let Some(org_id) = service.persona_view(&receiver).ok().and_then(|view| view.org_id) else {
            return (Verdict::Refused, Step::Failed(ReceiveError::NoOrganisation { receiver: Box::new(receiver) }));
        };
        match checked {
            Checked::ProvisionalOrganisationKept { epoch } | Checked::OrganisationAnswerKept { epoch } => {
                // The same held store, under the same lock: the listing is not refused, and it
                // holds the epoch org-node just kept, so `Silent` here is not reached.
                let root = service.provisional_organisations(&receiver).ok().and_then(|held| {
                    held.into_iter().find(|provisional| provisional.epoch == epoch).map(|provisional| provisional.root)
                });
                match root {
                    Some(root) => (Verdict::Accepted, Step::Report(Box::new(Received::ProvisionalOrganisationReceived { org_id, epoch, root, from: sender }))),
                    None => (Verdict::Accepted, Step::Silent),
                }
            }
            Checked::AlreadyHeld => (Verdict::Accepted, Step::Silent),
            Checked::RevocationNoticeKept => (Verdict::Accepted, Step::Report(Box::new(Received::RevocationNoticeKept { org_id, from: sender }))),
            Checked::AdmissionRequest { outcome: AdmissionRequestOutcome::AlreadyReceived } => (Verdict::AlreadyReceived, Step::Silent),
            Checked::AdmissionRequest { outcome: AdmissionRequestOutcome::InviteUsedByAnotherDevice } => {
                (Verdict::InviteUsedByAnotherDevice, Step::Silent)
            }
            Checked::AdmissionRequest { outcome: AdmissionRequestOutcome::Accepted } => {
                drop(service);
                (Verdict::Accepted, self.admission_kept(org_id, body).await)
            }
            Checked::Acknowledged(verified) => (Verdict::Accepted, Step::Report(Box::new(Received::Acknowledged(verified)))),
            Checked::OrganisationRequest { answer } => {
                // Read after the write: the addresses the request carried.
                let addresses = kept_for(&service, &receiver, answer.recipient);
                (Verdict::Accepted, Step::Answer { org_id, outgoing: Outgoing { recipient: answer.recipient, body: answer.body, addresses } })
            }
        }
    }

    /// The report of a first admission request, filled after its write from the kept request
    /// `admission_requests` lists for its invite (LLR-zkybw7, LLR-x36szv).
    async fn admission_kept(&self, org_id: OrgId, body: &[u8]) -> Step {
        // org-node just accepted `body` as an admission request, so the peek reads it. The lookup
        // runs after the lock was released: a request missing there was removed in between (its
        // invite cancelled), and nothing kept is left to report.
        let Some((_, invite_id)) = peek_admission_request(body) else { return Step::Silent };
        let kept = self.admission_requests().await.into_iter().find(|kept| kept.org_id == org_id && kept.invite_id == invite_id);
        match kept {
            Some(kept) => Step::Report(Box::new(Received::AdmissionRequestKept {
                org_id,
                invite_id,
                name: kept.name,
                surname: kept.surname,
                preferred_handles: kept.preferred_handles,
                from: kept.device_key,
                invitee_name: kept.invitee_name,
            })),
            None => Step::Silent,
        }
    }
}

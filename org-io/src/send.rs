//! Sending a committed update (SDD-v2bq89): every body
//! org-node's commit built goes to its recipient from the endpoint of the
//! Persona the commit names as its sender (LLR-d2pzzk). In Networked mode
//! each recipient is dialled by the endpoint identity equal to its Device
//! key, every one is attempted whatever happened to the others, and each is
//! reported (LLR-znhmh2). In Loopback mode, which has no address book, only
//! the recipient whose Device key is the identity of the address the caller
//! holds is sent to, at that address; without an address, or with one of no
//! recipient, nothing is sent (LLR-r9sh6k). A send writes nothing and never
//! undoes the commit.

use org_node::{CommitOutcome, DevicePublicKey};

use crate::endpoints::Endpoints;
use crate::peer_addr::PeerAddr;
use crate::transport::{TransportError, TransportMode};

/// What became of each recipient's body, in the order the commit built
/// them (`CommitOutcome::outgoing`): Device public keys and typed errors,
/// public data only.
#[derive(Debug, Default)]
pub struct SendReport {
    pub per_device: Vec<(DevicePublicKey, Result<(), SendError>)>,
}

impl SendReport {
    /// Each failed send as `<DevicePublicKey hex>: <error>`, joined by `; `, in
    /// the report's order; `None` when every send succeeded.
    pub fn failure_summary(&self) -> Option<String> {
        let failed: Vec<String> = self
            .per_device
            .iter()
            .filter_map(|(device_key, sent)| {
                sent.as_ref().err().map(|error| format!("{}: {error}", hex::encode(device_key.as_bytes())))
            })
            .collect();
        if failed.is_empty() {
            None
        } else {
            Some(failed.join("; "))
        }
    }
}

/// Why one recipient's body was not delivered.
#[derive(Debug)]
pub enum SendError {
    /// Loopback mode and no address: nothing was sent to anyone.
    NoLoopbackAddress,
    /// Loopback mode and the address's endpoint identity is no recipient's
    /// Device key: nothing was sent to anyone.
    AddressNotARecipient,
    /// Loopback mode: the address is another recipient's, so this one was
    /// not dialled.
    NotAttemptedInLoopback,
    /// The sending Persona's endpoint could not be had, so nothing was sent.
    SenderEndpoint(String),
    /// The dial or the stream failed.
    Transport(TransportError),
}

impl std::fmt::Display for SendError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoLoopbackAddress => write!(formatter, "Loopback send requires the peer's address; nothing sent"),
            Self::AddressNotARecipient => {
                write!(formatter, "the peer address names no recipient's DevicePublicKey; nothing sent")
            }
            Self::NotAttemptedInLoopback => write!(formatter, "not attempted: Loopback sends to the addressed recipient only"),
            Self::SenderEndpoint(reason) => write!(formatter, "the sender's endpoint: {reason}"),
            Self::Transport(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for SendError {}

/// Why a whole send is refused before any endpoint is bound (LLR-r9sh6k).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WholeSendRefusal {
    /// Loopback mode and no address.
    NoLoopbackAddress,
    /// Loopback mode and the address's endpoint identity is no recipient's
    /// DevicePublicKey.
    AddressNotARecipient,
}

impl WholeSendRefusal {
    fn error(self) -> SendError {
        match self {
            Self::NoLoopbackAddress => SendError::NoLoopbackAddress,
            Self::AddressNotARecipient => SendError::AddressNotARecipient,
        }
    }
}

/// What the send does for one recipient once it goes ahead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RecipientStep {
    /// Networked: dial by the endpoint identity equal to the recipient's
    /// DevicePublicKey.
    DialByDevice,
    /// Loopback: dial at the address the caller holds, which is this
    /// recipient's.
    DialAtAddress,
    /// Loopback: the address is another recipient's; this one is not
    /// dialled.
    NotAttempted,
}

/// The send's decision, taken before any IO.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SendPlan {
    /// Nothing is sent; every recipient is reported with the refusal.
    Refused(WholeSendRefusal),
    /// One step per recipient, in the recipients' order.
    Steps(Vec<RecipientStep>),
}

/// Decide the send to `recipients` in `mode`, given the endpoint identity
/// of the address the caller holds, if any (LLR-znhmh2, LLR-r9sh6k). Pure:
/// it reads nothing and dials nothing. Networked dials every recipient by
/// its DevicePublicKey and ignores the address. Loopback refuses the whole
/// send without an address, or with one that is no recipient's; otherwise it
/// dials the addressed recipient at that address and no other.
pub(crate) fn plan_send(mode: TransportMode, recipients: &[DevicePublicKey], addressed: Option<[u8; 32]>) -> SendPlan {
    match mode {
        TransportMode::Networked => SendPlan::Steps(vec![RecipientStep::DialByDevice; recipients.len()]),
        TransportMode::Loopback => {
            let Some(addressed) = addressed else { return SendPlan::Refused(WholeSendRefusal::NoLoopbackAddress) };
            if !recipients.iter().any(|recipient| recipient.as_bytes() == &addressed) {
                return SendPlan::Refused(WholeSendRefusal::AddressNotARecipient);
            }
            let step = |recipient: &DevicePublicKey| {
                if recipient.as_bytes() == &addressed {
                    RecipientStep::DialAtAddress
                } else {
                    RecipientStep::NotAttempted
                }
            };
            SendPlan::Steps(recipients.iter().map(step).collect())
        }
    }
}

/// Send every body of `outcome` from the endpoint of `sender`, the Device of
/// the Persona the commit names, as `endpoints`' mode says (LLR-znhmh2,
/// LLR-r9sh6k): `plan_send` decides, and this function carries the plan
/// out. A refusal of the whole send is reported against every recipient; no
/// endpoint is bound before it.
pub(crate) async fn send_committed(
    endpoints: &Endpoints,
    sender: Result<DevicePublicKey, String>,
    outcome: &CommitOutcome,
    peer_addr: Option<PeerAddr>,
) -> SendReport {
    let recipients: Vec<DevicePublicKey> = outcome.outgoing.iter().map(|message| message.recipient).collect();
    let steps = match plan_send(endpoints.transport_mode(), &recipients, peer_addr.as_ref().map(PeerAddr::endpoint_id)) {
        SendPlan::Refused(refusal) => {
            return SendReport { per_device: recipients.into_iter().map(|recipient| (recipient, Err(refusal.error()))).collect() };
        }
        SendPlan::Steps(steps) => steps,
    };
    if outcome.outgoing.is_empty() {
        return SendReport::default();
    }
    let endpoint = match sender {
        Ok(device) => endpoints.endpoint(&device).await.map_err(|error| error.to_string()),
        Err(reason) => Err(reason),
    };
    let endpoint = match endpoint {
        Ok(endpoint) => endpoint,
        Err(reason) => {
            let per_device = recipients.into_iter().map(|recipient| (recipient, Err(SendError::SenderEndpoint(reason.clone())))).collect();
            return SendReport { per_device };
        }
    };
    let mut per_device = Vec::with_capacity(outcome.outgoing.len());
    for (message, step) in outcome.outgoing.iter().zip(steps) {
        let recipient = message.recipient;
        let sent = match (step, &peer_addr) {
            (RecipientStep::NotAttempted, _) => Err(SendError::NotAttemptedInLoopback),
            (RecipientStep::DialAtAddress, Some(addr)) => endpoint.send_body_at(addr, &message.body).await.map_err(SendError::Transport),
            // Cross-network: dial purely by the endpoint identity equal to
            // the recipient's DevicePublicKey; the transport resolves the
            // path. (`DialAtAddress` is planned only with an address.)
            (RecipientStep::DialByDevice, _) | (RecipientStep::DialAtAddress, None) => {
                endpoint.send_body_to_device(&recipient, &message.body).await.map_err(SendError::Transport)
            }
        };
        per_device.push((recipient, sent));
    }
    SendReport { per_device }
}

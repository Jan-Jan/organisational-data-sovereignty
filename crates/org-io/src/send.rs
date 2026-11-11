//! The send path (SDD-2bzgtp): every message org-io sends goes from the
//! sending Persona's endpoint, on a new connection, to a DevicePublicKey, at the
//! endpoint addresses org-node keeps for that key; one recipient after
//! another, each reported with the verdict byte it wrote back
//! (LLR-pxqdy4, LLR-34tfm8, LLR-hx5c2y, LLR-fwz3tb). A send writes no store
//! and reads no chain.

use std::net::SocketAddr;
use std::sync::Arc;

use org_node::service::{OrgService, OutgoingMessage};
use org_node::{DevicePublicKey, OrgId, OrgNodeError};

use crate::endpoints::Endpoints;
use crate::transport::{parse_kept_address, OrgEndpoint, TransportError, TransportMode, Verdict};
use crate::OrgIo;

/// What became of one recipient's message (LLR-fwz3tb). Public data only.
#[derive(Debug)]
pub enum Delivery {
    /// `0x00`.
    Delivered,
    /// `0x01`.
    Refused,
    /// `0x02`.
    AlreadyReceived,
    /// `0x03`.
    InviteUsedByAnotherDevice,
    /// Not sent, or no verdict came back.
    NotDelivered { reason: SendError },
}

/// Each recipient's outcome, in org-node's order: Device public keys and
/// typed outcomes, public data only.
#[derive(Debug, Default)]
pub struct SendReport {
    pub per_device: Vec<(DevicePublicKey, Delivery)>,
}

/// Why one recipient's message was not delivered.
#[derive(Debug)]
pub enum SendError {
    /// Loopback mode and no address kept for the recipient.
    NoLoopbackAddress,
    /// Loopback mode and no kept address's identity is the recipient's key.
    AddressNotARecipient,
    /// Loopback mode and no Invite address's identity is the issuer's key.
    AddressNotTheIssuer,
    /// No address may be dialled (the acknowledgement delivery).
    NoAddress,
    /// The sending Persona's endpoint could not be had; nothing was sent.
    SenderEndpoint(String),
    /// The stream ended, or the time limit passed, with no byte written back.
    NoVerdict,
    /// A byte that is none of the four verdicts.
    UnknownVerdict(u8),
    /// The dial or the stream failed.
    Transport(TransportError),
}

impl std::fmt::Display for SendError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoLoopbackAddress => write!(formatter, "Loopback: no address kept for the recipient; nothing sent"),
            Self::AddressNotARecipient => write!(formatter, "Loopback: no kept address is the recipient's; nothing sent"),
            Self::AddressNotTheIssuer => write!(formatter, "Loopback: no Invite address is the issuer's; nothing sent"),
            Self::NoAddress => write!(formatter, "no address may be dialled"),
            Self::SenderEndpoint(reason) => write!(formatter, "the sender's endpoint: {reason}"),
            Self::NoVerdict => write!(formatter, "no verdict within the time limit"),
            Self::UnknownVerdict(byte) => write!(formatter, "verdict byte 0x{byte:02x} is none of the four"),
            Self::Transport(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for SendError {}

/// Why a whole send was refused before anything was sent.
#[derive(Debug)]
pub enum SendRefused {
    /// No Persona here holds the Organisation (LLR-zrgj8z).
    NotHeld { org_id: OrgId },
    /// The sending Persona's endpoint could not be bound (a request's).
    Bind(TransportError),
    /// org-node refused to give the messages.
    Node(OrgNodeError),
}

impl std::fmt::Display for SendRefused {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotHeld { org_id } => crate::write_not_held(formatter, org_id),
            Self::Bind(error) => write!(formatter, "the sender's endpoint could not be bound: {error}"),
            Self::Node(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for SendRefused {}

/// One message to send, with the addresses kept for its recipient.
pub(crate) struct Outgoing {
    pub(crate) recipient: DevicePublicKey,
    pub(crate) body: Vec<u8>,
    pub(crate) addresses: Option<Vec<String>>,
}

/// The Persona bound to `org_id` — holding its current record, its
/// founding or a provisional Member of it — by its DevicePublicKey
/// (`crate::record_holder_of` matches a current record only).
pub(crate) fn bound_holder_of(service: &OrgService, org_id: OrgId) -> Option<DevicePublicKey> {
    service.persona_records().into_iter().find(|persona| persona.org_id == Some(org_id)).map(|persona| persona.device_key)
}

/// org-node's `messages`, each with the addresses `sender` keeps for its recipient.
fn outgoing_with_kept(service: &OrgService, sender: &DevicePublicKey, messages: Vec<OutgoingMessage>) -> Vec<Outgoing> {
    messages
        .into_iter()
        .map(|message| {
            let addresses = kept_for(service, sender, message.recipient);
            Outgoing { recipient: message.recipient, body: message.body, addresses }
        })
        .collect()
}

/// The addresses org-node keeps for `peer` in `device_key`'s store, as text.
/// Every caller asks right after an org-node call that succeeded on that same
/// store under the same lock, and `addresses_for` refuses only a key no store
/// holds and a removed store, so a refusal never reaches `None` here: `None`
/// is "no address kept".
pub(crate) fn kept_for(service: &OrgService, device_key: &DevicePublicKey, peer: DevicePublicKey) -> Option<Vec<String>> {
    service.addresses_for(device_key, peer).ok().flatten().map(|addresses| addresses.as_slice().to_vec())
}

/// The sockets to dial `recipient` at (LLR-34tfm8): in Networked mode
/// every parsed kept address, as a hint (none is fine: discovery finds the
/// key); in Loopback mode those whose identity is the recipient's key,
/// refusing with `NoLoopbackAddress` when none is kept and
/// `AddressNotARecipient` when none is the recipient's. Pure.
pub(crate) fn sockets_for(
    mode: TransportMode,
    recipient: &DevicePublicKey,
    addresses: Option<&[String]>,
) -> Result<Vec<SocketAddr>, SendError> {
    let addresses = addresses.unwrap_or(&[]);
    // A kept address that does not parse names no socket to dial, so it is passed over: in
    // Networked mode every address is only a hint (discovery finds the key), and in Loopback mode
    // a recipient none of whose kept addresses parses is refused with `AddressNotARecipient`.
    let parsed = addresses.iter().filter_map(|text| parse_kept_address(text));
    match mode {
        TransportMode::Networked => Ok(parsed.map(|(_, socket)| socket).collect()),
        TransportMode::Loopback => {
            if addresses.is_empty() {
                return Err(SendError::NoLoopbackAddress);
            }
            let recipients_own: Vec<SocketAddr> = parsed.filter(|(id, _)| id == recipient.as_bytes()).map(|(_, socket)| socket).collect();
            if recipients_own.is_empty() {
                Err(SendError::AddressNotARecipient)
            } else {
                Ok(recipients_own)
            }
        }
    }
}

/// What a dial's read-back is (LLR-fwz3tb).
pub(crate) fn delivery_of(read: Result<Vec<u8>, TransportError>) -> Delivery {
    match read {
        Err(error) => Delivery::NotDelivered { reason: SendError::Transport(error) },
        Ok(bytes) => match bytes.as_slice() {
            [] => Delivery::NotDelivered { reason: SendError::NoVerdict },
            [byte] => match Verdict::from_byte(*byte) {
                Some(Verdict::Accepted) => Delivery::Delivered,
                Some(Verdict::Refused) => Delivery::Refused,
                Some(Verdict::AlreadyReceived) => Delivery::AlreadyReceived,
                Some(Verdict::InviteUsedByAnotherDevice) => Delivery::InviteUsedByAnotherDevice,
                None => Delivery::NotDelivered { reason: SendError::UnknownVerdict(*byte) },
            },
            [byte, ..] => Delivery::NotDelivered { reason: SendError::UnknownVerdict(*byte) },
        },
    }
}

/// Send `body` to `recipient` at `sockets` from `endpoint`, and read its verdict.
pub(crate) async fn send_one(endpoint: &OrgEndpoint, recipient: &DevicePublicKey, sockets: &[SocketAddr], body: &[u8]) -> Delivery {
    delivery_of(endpoint.send_for_verdict(recipient, sockets, body).await)
}

/// Send every message of `outgoing`, in order, from `sender`'s endpoint
/// (LLR-hx5c2y), each recipient judged on its own addresses (LLR-34tfm8);
/// every recipient is attempted whatever happened to the others. When the
/// endpoint cannot be had, every recipient reports `SenderEndpoint`.
pub(crate) async fn send_all(endpoints: &Endpoints, sender: DevicePublicKey, outgoing: Vec<Outgoing>) -> SendReport {
    if outgoing.is_empty() {
        return SendReport::default();
    }
    let endpoint: Result<Arc<OrgEndpoint>, String> = endpoints.endpoint(&sender).await.map_err(|error| error.to_string());
    let mode = endpoints.transport_mode();
    let mut per_device = Vec::with_capacity(outgoing.len());
    for Outgoing { recipient, body, addresses } in outgoing {
        let delivery = match (&endpoint, sockets_for(mode, &recipient, addresses.as_deref())) {
            (Err(reason), _) => Delivery::NotDelivered { reason: SendError::SenderEndpoint(reason.clone()) },
            (Ok(_), Err(refused)) => Delivery::NotDelivered { reason: refused },
            (Ok(endpoint), Ok(sockets)) => send_one(endpoint, &recipient, &sockets, &body).await,
        };
        per_device.push((recipient, delivery));
    }
    SendReport { per_device }
}

impl OrgIo {
    /// Send the adopted current epoch of `org_id` (LLR-pxqdy4): under the
    /// node's lock, org-node's messages for it and the addresses kept for
    /// each recipient; with the lock released, each sent in org-node's
    /// order. No store write, no later send of its own; a repeated call
    /// sends the same messages.
    pub async fn send_organisation(&self, org_id: OrgId) -> Result<SendReport, SendRefused> {
        let (sender, outgoing) = {
            let service = self.service.lock().await;
            let sender = bound_holder_of(&service, org_id).ok_or(SendRefused::NotHeld { org_id })?;
            let messages = service.messages_for_current_epoch(&sender).map_err(|error| match error {
                OrgNodeError::OrgNotHeld { .. } => SendRefused::NotHeld { org_id },
                other => SendRefused::Node(other),
            })?;
            (sender, outgoing_with_kept(&service, &sender, messages))
        };
        Ok(send_all(&self.local.endpoints, sender, outgoing).await)
    }
}

/// Why `request_admission` sent nothing. Public data only.
#[derive(Debug)]
pub enum RequestError {
    /// No Persona here holds a provisional Member for `org_id`.
    NoProvisionalMember { org_id: OrgId },
    /// The provisional Member's endpoint could not be bound.
    Bind(TransportError),
    /// org-node refused to build the request.
    Node(OrgNodeError),
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoProvisionalMember { org_id } => {
                write!(formatter, "no provisional Member for Organisation {}", hex::encode(org_id.as_bytes()))
            }
            Self::Bind(error) => write!(formatter, "the endpoint could not be bound: {error}"),
            Self::Node(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for RequestError {}

impl OrgIo {
    /// Ask the Invite's issuer for admission (LLR-adgk9z, LLR-qa4nrt): the
    /// provisional Member's endpoint bound, the request built by org-node
    /// with that endpoint's addresses, sent to the issuer at the Invite's
    /// addresses (in Loopback mode only those whose identity is the
    /// issuer's key); writes nothing.
    pub async fn request_admission(&self, org_id: OrgId) -> Result<Delivery, RequestError> {
        let device_key = {
            let service = self.service.lock().await;
            bound_holder_of(&service, org_id).ok_or(RequestError::NoProvisionalMember { org_id })?
        };
        let endpoint = self.local.endpoints.endpoint(&device_key).await.map_err(RequestError::Bind)?;
        let mode = self.local.endpoints.transport_mode();
        let own_addresses = endpoint.addresses(mode);
        let (request, issuer_addresses) = {
            let service = self.service.lock().await;
            let request = service.build_admission_request(&device_key, own_addresses).map_err(|error| match error {
                OrgNodeError::NoProvisionalMember => RequestError::NoProvisionalMember { org_id },
                other => RequestError::Node(other),
            })?;
            let issuer_addresses = kept_for(&service, &device_key, request.recipient);
            (request, issuer_addresses)
        };
        let sockets = match sockets_for(mode, &request.recipient, issuer_addresses.as_deref()) {
            Ok(sockets) => sockets,
            Err(SendError::AddressNotARecipient) => return Ok(Delivery::NotDelivered { reason: SendError::AddressNotTheIssuer }),
            Err(refused) => return Ok(Delivery::NotDelivered { reason: refused }),
        };
        Ok(send_one(&endpoint, &request.recipient, &sockets, &request.body).await)
    }

    /// Ask for `org_id` (LLR-uf5cyy): every other Device of the current
    /// record, or the Invite's issuer for a provisional Member, each at the
    /// addresses kept for it; the request carries the sender's own
    /// addresses; writes nothing and sends again on every call.
    pub async fn request_organisation(&self, org_id: OrgId) -> Result<SendReport, SendRefused> {
        let device_key = {
            let service = self.service.lock().await;
            bound_holder_of(&service, org_id).ok_or(SendRefused::NotHeld { org_id })?
        };
        let endpoint = self.local.endpoints.endpoint(&device_key).await.map_err(SendRefused::Bind)?;
        let own_addresses = endpoint.addresses(self.local.endpoints.transport_mode());
        let outgoing = {
            let service = self.service.lock().await;
            let requests = service.build_organisation_request(&device_key, own_addresses).map_err(|error| match error {
                OrgNodeError::OrgNotHeld { .. } => SendRefused::NotHeld { org_id },
                other => SendRefused::Node(other),
            })?;
            outgoing_with_kept(&service, &device_key, requests)
        };
        Ok(send_all(&self.local.endpoints, device_key, outgoing).await)
    }
}

//! The Wire message: its six kinds, its encoding and its bound. Pure values;
//! the length prefix the channel puts in front of a message belongs to the
//! transport, not here.
use org_members::{DevicePublicKey, Handle, Name, PersonPublicKey, Surname};
use serde::{Deserialize, Serialize};

use crate::envelope::Envelope;
use crate::error::OrgNodeError;
use crate::ids::{InviteId, OrgId};
use crate::revocation::{Acknowledgement, RevocationNotice};
use crate::types::{Epoch, OrgPrivateKey};

/// The largest encoded Wire message accepted or produced (a defensive cap
/// against a hostile peer): 1 MiB, generous for a delta, a record snapshot
/// and a key (LLR-8kh3zf, LLR-sc6zuh).
pub const MAX_MESSAGE: usize = 1024 * 1024;

/// The variant index of Organisation information (LLR-xr88a8).
pub(crate) const ORG_INFORMATION_INDEX: u8 = 0;

/// The variant index of an admission request (LLR-xr88a8, LLR-g9n9w9).
const ADMISSION_REQUEST_INDEX: u8 = 3;

/// The variant index of an organisation answer (LLR-xr88a8).
pub(crate) const ORGANISATION_INDEX: u8 = 5;

/// The number of `WireMessage` kinds: every variant index is below it.
const KIND_COUNT: u8 = 6;

/// One message over the org-node channel, of one of six kinds (LLR-xr88a8),
/// encoded by postcard as the variant index followed by its fields in
/// order. No field is optional. Only `AdmissionRequest` carries an invite
/// identifier (LLR-g3wmen). `Debug` is derived: the only secret any kind
/// holds is an `OrgPrivateKey`, whose own `Debug` redacts it (LLR-4ces44).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WireMessage {
    /// Index 0: the Envelope of an adopted epoch; the record it extends,
    /// postcard `Vec<MemberSnapshot>`; and the Organisation private key of
    /// that epoch. A body without either does not decode (REQ-zxgd46).
    OrgInformation { envelope: Envelope, record_snapshot: Vec<u8>, org_private_key: OrgPrivateKey },
    /// Index 1: the revoked Device's identity and an absence proof, no
    /// Envelope (LLR-dc45ur).
    Revocation(RevocationNotice),
    /// Index 2: a revoked Device's signed acknowledgement (LLR-378cj4).
    Acknowledgement(Acknowledgement),
    /// Index 3: a joining Device's request to be admitted under an Invite:
    /// the Invite's identifiers first, the sending Device's own endpoint
    /// addresses last (as given; parsed on receipt, LLR-h4gwns).
    AdmissionRequest {
        org_id: OrgId,
        invite_id: InviteId,
        name: Name,
        surname: Surname,
        preferred_handles: Vec<Handle>,
        member_key: PersonPublicKey,
        device_key: DevicePublicKey,
        addresses: Vec<String>,
    },
    /// Index 4: a Device behind asks for the Organisation, giving its own
    /// endpoint addresses (as given; parsed on receipt, LLR-h4gwns).
    OrganisationRequest { org_id: OrgId, addresses: Vec<String> },
    /// Index 5: the answer to an organisation request from a listed Device:
    /// the current record's epoch, its members as a record snapshot and its
    /// Organisation private key (LLR-7kdhst).
    Organisation { org_id: OrgId, epoch: Epoch, record_snapshot: Vec<u8>, org_private_key: OrgPrivateKey },
}

impl WireMessage {
    /// The Organisation any kind names (LLR-xr88a8).
    pub fn org_id(&self) -> OrgId {
        match self {
            Self::OrgInformation { envelope, .. } => envelope.org_id,
            Self::Revocation(notice) => notice.org_id,
            Self::Acknowledgement(acknowledgement) => acknowledgement.org_id,
            Self::AdmissionRequest { org_id, .. }
            | Self::OrganisationRequest { org_id, .. }
            | Self::Organisation { org_id, .. } => *org_id,
        }
    }
}

/// Encode a Wire message as its postcard body, with no length prefix
/// (LLR-er2x8n). An encoding over [`MAX_MESSAGE`] is refused (LLR-sc6zuh).
pub fn encode_message(message: &WireMessage) -> Result<Vec<u8>, OrgNodeError> {
    let body = postcard::to_allocvec(message).map_err(|_| OrgNodeError::MalformedMessage)?;
    if body.len() > MAX_MESSAGE {
        return Err(OrgNodeError::MessageTooLarge { length: body.len() });
    }
    Ok(body)
}

/// Decode a Wire message from its postcard body. A body over [`MAX_MESSAGE`]
/// is refused before decoding (LLR-8kh3zf); a body that is not a Wire
/// message — an unknown variant index, a body that ends early, a key field
/// that is not 32 bytes — or has bytes left over after one of any kind, is
/// `MalformedMessage`, without a panic (LLR-xr88a8).
pub fn decode_message(body: &[u8]) -> Result<WireMessage, OrgNodeError> {
    if body.len() > MAX_MESSAGE {
        return Err(OrgNodeError::MessageTooLarge { length: body.len() });
    }
    match postcard::take_from_bytes(body) {
        Ok((message, [])) => Ok(message),
        _ => Err(OrgNodeError::MalformedMessage),
    }
}

/// The Organisation and invite identifier of an admission request, read
/// from the variant index and the next twenty and thirty-two bytes alone;
/// `None` for any other index or a shorter body. No byte after the invite
/// identifier is read (LLR-g9n9w9).
pub fn peek_admission_request(body: &[u8]) -> Option<(OrgId, InviteId)> {
    let (index, org_id) = peek_kind_and_org_id(body)?;
    if index != ADMISSION_REQUEST_INDEX {
        return None;
    }
    let invite_id: [u8; 32] = body.get(21..53)?.try_into().ok()?;
    Some((org_id, InviteId::new(invite_id)))
}

/// The variant index of a body and the Organisation every kind names first,
/// read from the index and the next twenty bytes alone (LLR-e6w3nr): every
/// kind's first field is its Organisation identifier, or (Organisation
/// information) its Envelope's. `None` for an index of no kind or a shorter
/// body.
pub(crate) fn peek_kind_and_org_id(body: &[u8]) -> Option<(u8, OrgId)> {
    let (&index, rest) = body.split_first()?;
    if index >= KIND_COUNT {
        return None;
    }
    let org_id: [u8; 20] = rest.get(..20)?.try_into().ok()?;
    Some((index, OrgId::new(org_id)))
}

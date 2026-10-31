//! The invitation operations: the Organisation's outstanding invites, `issue_invite` and
//! `cancel_invite` (SDD-zqv3v6), each writing the store org-node returns before it reports; an
//! imported Invite read, `read_invite` (SDD-9rbm8t); and the kept admission requests listed,
//! `admission_requests` (SDD-cxh9d8). Each goes through org-node's Invite operations.

use org_node::invitation::{Invite, InviteId};
use org_node::{OrgId, OrgNodeError};

use crate::transport::TransportError;
use crate::view::{InviteSummary, KeptAdmissionRequestSummary};
use crate::{OrgIo, StoreFailure};

/// Why an invitation operation returned nothing: public data only.
#[derive(Debug)]
pub enum ExchangeError {
    /// org-node refused: a Blob that does not parse (naming the field), or an invite with no
    /// kept admission request.
    Node(OrgNodeError),
    /// The operation's store write failed; the Persona was reloaded from its file, or dropped.
    Store(StoreFailure),
    /// The issuing Persona's endpoint could not be bound.
    Bind(TransportError),
    /// No Persona here holds `org_id`'s current record (org-io's lookup;
    /// nothing asked of org-node, nothing written).
    NotHeld { org_id: OrgId },
}

impl From<OrgNodeError> for ExchangeError {
    fn from(error: OrgNodeError) -> Self {
        Self::Node(error)
    }
}

impl std::fmt::Display for ExchangeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Node(error) => write!(formatter, "{error}"),
            Self::Store(failure) => write!(formatter, "nothing returned: {failure}"),
            Self::Bind(error) => write!(formatter, "this Persona's endpoint could not be bound: {error}"),
            Self::NotHeld { org_id } => crate::write_not_held(formatter, org_id),
        }
    }
}

impl std::error::Error for ExchangeError {}

impl OrgIo {
    /// An Invite for `org_id` from the Persona holding it (LLR-95e3jx, LLR-v9dkrv): that Persona's
    /// endpoint bound first and its addresses in the current mode taken, then org-node builds the
    /// Invite with an identifier drawn from the handle's source and keeps it outstanding; the store
    /// is written before the Blob is returned.
    pub async fn issue_invite(&self, org_id: OrgId, invitee_name: &str) -> Result<String, ExchangeError> {
        let device_key = crate::record_holder_of(&*self.service.lock().await, org_id).ok_or(ExchangeError::NotHeld { org_id })?;
        let addresses = self.local.endpoints.addresses(&device_key).await.map_err(ExchangeError::Bind)?;
        let mut service = self.service.lock().await;
        let built = {
            let mut rng = self.local.rng.lock();
            service.build_invite(&mut *rng, &device_key, invitee_name.to_string(), addresses)
        };
        let (invite, change) = built?;
        self.local.store(&mut service, change).map_err(ExchangeError::Store)?;
        Ok(invite.encode()?)
    }

    /// The public fields of `invite_blob` (LLR-npw5zg); writes and creates nothing.
    pub async fn read_invite(&self, invite_blob: &str) -> Result<InviteSummary, ExchangeError> {
        let invite = Invite::parse(invite_blob)?;
        Ok(InviteSummary {
            issuer_addresses: invite.issuer_addresses.as_slice().to_vec(),
            org_name: invite.org_name,
            org_id: invite.org_id,
            invitee_name: invite.invitee_name,
            issuer: invite.issuer,
            invite_id: invite.invite_id,
        })
    }

    /// Cancel `invite_id` of `org_id` (LLR-8ze6cc): org-node removes its kept admission request and
    /// the outstanding invite, and the store is written before `Ok`.
    pub async fn cancel_invite(&self, org_id: OrgId, invite_id: InviteId) -> Result<(), ExchangeError> {
        let mut service = self.service.lock().await;
        let device_key = crate::record_holder_of(&service, org_id).ok_or(ExchangeError::NotHeld { org_id })?;
        let cancelled = {
            let mut rng = self.local.rng.lock();
            service.cancel_invite(&mut *rng, &device_key, invite_id)
        };
        self.local.store(&mut service, cancelled?).map_err(ExchangeError::Store)
    }

    /// Every admission request the held Organisations keep (LLR-dm2h8u), in store-file order and,
    /// within a store, in org-node's; changes nothing.
    pub async fn admission_requests(&self) -> Vec<KeptAdmissionRequestSummary> {
        let service = self.service.lock().await;
        // `persona_view` and `admission_requests` refuse only a key no store holds and a removed
        // store, and `persona_records` lists neither: no refusal leaves a request out.
        let mut holders: Vec<_> = service
            .persona_records()
            .into_iter()
            .filter(|persona| service.persona_view(&persona.device_key).is_ok_and(|view| view.record.is_some()))
            .map(|persona| persona.device_key)
            .collect();
        holders.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        let mut listed = vec![];
        for device_key in holders {
            let Ok(requests) = service.admission_requests(&device_key) else { continue };
            listed.extend(requests.into_iter().map(|(request, invitee_name)| KeptAdmissionRequestSummary {
                org_id: request.org_id,
                invite_id: request.invite_id,
                name: request.name.clone(),
                surname: request.surname.clone(),
                preferred_handles: request.preferred_handles.clone(),
                device_key: request.device_key,
                invitee_name: invitee_name.to_string(),
            }));
        }
        listed
    }
}

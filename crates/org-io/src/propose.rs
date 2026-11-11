//! The Admin's provisional changes (SDD-7mhqaz) and the proposed Organisation (SDD-4drrpr; both
//! split from one item): recording an
//! admission or a revocation, and having org-node calculate the provisional organisation from all
//! of them; each writes the store org-node returns and touches no chain or transport. No
//! operation discards (LLR-vvd8dn).

use org_node::invitation::InviteId;
use org_node::store::{Origin, ProvisionalChange};
use org_node::{Epoch, Handle, MemberId, OrgId, OrgPublicKey, RootHash};

use crate::{OperationError, OrgIo};

/// A provisional change as recorded (LLR-un47j8, LLR-6kb33y): public data only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProvisionalChangeSummary {
    Admission { org_id: OrgId, invite_id: InviteId, handle: Handle },
    Revocation { org_id: OrgId, member_id: MemberId },
}

/// The provisional organisation `propose_updated_organisation` kept (LLR-5fchwt).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProposedOrganisation {
    pub org_id: OrgId,
    pub epoch: Epoch,
    pub root: RootHash,
    pub org_public_key: OrgPublicKey,
}

impl OrgIo {
    /// Record the admission of the request kept for `invite_id` (LLR-un47j8), with `chosen_handle`
    /// when one is given; the store is written before the change is returned with its Handle.
    pub async fn propose_admission(
        &self,
        org_id: OrgId,
        invite_id: InviteId,
        chosen_handle: Option<&str>,
    ) -> Result<ProvisionalChangeSummary, OperationError> {
        let mut service = self.service.lock().await;
        let device_key = crate::record_holder_of(&service, org_id).ok_or(OperationError::NotHeld { org_id })?;
        let recorded = {
            let mut rng = self.local.rng.lock();
            service.record_admission(&mut *rng, &device_key, invite_id, chosen_handle)
        };
        self.local.store(&mut service, recorded?).map_err(OperationError::Store)?;
        let handle = service.provisional_changes(&device_key)?.iter().rev().find_map(|change| match change {
            ProvisionalChange::Admit { invite_id: admitted, handle } if *admitted == invite_id => Some(handle.clone()),
            _ => None,
        });
        let handle = handle.ok_or(OperationError::AdmissionNotListed { invite_id })?;
        Ok(ProvisionalChangeSummary::Admission { org_id, invite_id, handle })
    }

    /// Record the revocation of `member_id` (LLR-6kb33y); the store is written before it returns.
    pub async fn propose_revocation(&self, org_id: OrgId, member_id: MemberId) -> Result<ProvisionalChangeSummary, OperationError> {
        let mut service = self.service.lock().await;
        let device_key = crate::record_holder_of(&service, org_id).ok_or(OperationError::NotHeld { org_id })?;
        let recorded = {
            let mut rng = self.local.rng.lock();
            service.record_revocation(&mut *rng, &device_key, member_id)
        };
        self.local.store(&mut service, recorded?).map_err(OperationError::Store)?;
        Ok(ProvisionalChangeSummary::Revocation { org_id, member_id })
    }

    /// Have org-node apply every held provisional change to the current record with one fresh
    /// Organisation key pair (LLR-5fchwt); the store is written before the proposal is returned.
    pub async fn propose_updated_organisation(&self, org_id: OrgId) -> Result<ProposedOrganisation, OperationError> {
        let mut service = self.service.lock().await;
        let device_key = crate::record_holder_of(&service, org_id).ok_or(OperationError::NotHeld { org_id })?;
        let proposed = {
            let mut rng = self.local.rng.lock();
            service.propose_updated_organisation(&mut *rng, &device_key)
        };
        self.local.store(&mut service, proposed?).map_err(OperationError::Store)?;
        let view = service
            .provisional_organisations(&device_key)?
            .into_iter()
            .rev()
            .find(|view| matches!(view.origin, Origin::Own) && view.current_proposal)
            .ok_or(OperationError::ProposalNotListed { org_id })?;
        Ok(ProposedOrganisation { org_id, epoch: view.epoch, root: view.root, org_public_key: view.org_pub_key })
    }
}

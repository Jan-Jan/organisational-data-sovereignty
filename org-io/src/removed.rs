//! The handle's removed-Devices query (SDD-z3ychz, LLR-3ydu9e): each Device an Organisation's store
//! holds as revoked, with the epoch its entry of the Acknowledged record names. Reads the store only.
use org_node::{DevicePublicKey, Epoch, MemberId, OrgId, OrgNodeError};

use crate::OrgIo;

/// One Device held as revoked, and the epoch its entry of the Acknowledged record names, if it has one
/// (LLR-3ydu9e). Public values only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemovedDevice {
    pub member_id: MemberId,
    pub device_key: DevicePublicKey,
    pub acknowledged_epoch: Option<Epoch>,
}

/// Why `OrgIo::removed_devices` listed nothing (LLR-3ydu9e).
#[derive(Debug)]
pub enum RemovedDevicesRefused {
    /// No Persona here holds a current record of the Organisation.
    NotHeld { org_id: OrgId },
    /// org-node refused the query. Not reached from `removed_devices`: org-node refuses only a
    /// key no store holds, a removed store and a store with no current record, and under the same
    /// lock `record_holder_of` has just found the key's store holding a current record. Kept as
    /// LLR-3ydu9e states it, so no refusal is folded into another.
    Node(OrgNodeError),
}

impl std::fmt::Display for RemovedDevicesRefused {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotHeld { org_id } => crate::write_not_held(formatter, org_id),
            Self::Node(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for RemovedDevicesRefused {}

impl OrgIo {
    /// Each Device the store holding `org_id`'s current record holds as revoked, in org-node's
    /// order, with the epoch its entry of the Acknowledged record names (LLR-3ydu9e). Reads no chain,
    /// takes no Device secret, sends and writes nothing.
    pub async fn removed_devices(&self, org_id: OrgId) -> Result<Vec<RemovedDevice>, RemovedDevicesRefused> {
        let service = self.service.lock().await;
        let device_key = crate::record_holder_of(&service, org_id).ok_or(RemovedDevicesRefused::NotHeld { org_id })?;
        let revoked = service.removed_devices(&device_key).map_err(RemovedDevicesRefused::Node)?;
        Ok(revoked
            .into_iter()
            .map(|removed| RemovedDevice {
                member_id: removed.member_id,
                device_key: removed.device_key,
                acknowledged_epoch: removed.acknowledgement.map(|ack| ack.epoch),
            })
            .collect())
    }
}

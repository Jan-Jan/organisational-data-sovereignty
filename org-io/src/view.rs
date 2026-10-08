//! The handle's read surface (SDD-z3ychz; review round 3, finding 1): an
//! allowlist, not org-node's whole service. `NodeView` offers only the
//! queries the app makes, and each returns public data only: ids, names,
//! public keys, roots, epochs, and the summaries below, which copy the
//! public fields of org-node's records and leave the secrets they hold
//! behind. `BuiltUpdate` names a provisional update org-node holds without
//! carrying it.

use org_node::service::OrgService;
use org_node::store::{MemberSnapshot, OrgRecord, PersonaRecord, PersonaStatus, ProvisionalUpdate};
use org_node::{DevicePublicKey, Epoch, Handle, MemberId, Name, OrgId, OrgNodeError, OrgPublicKey, PersonPublicKey, PersonaId, RootHash, Surname};

/// org-node's queries the app makes, over the handle's service, each
/// returning public data only (LLR-3zdw8v, clarified 2026-10-08, review
/// round 3).
pub struct NodeView<'handle> {
    service: &'handle OrgService,
}

impl<'handle> NodeView<'handle> {
    pub(crate) fn new(service: &'handle OrgService) -> Self {
        Self { service }
    }

    /// Every Persona this node holds, public fields only.
    pub fn personas(&self) -> Vec<PersonaSummary> {
        self.service.list_personas().iter().map(PersonaSummary::from).collect()
    }

    /// Every Organisation this node holds, public fields only.
    pub fn organisations(&self) -> Vec<OrgSummary> {
        self.service.list_orgs().iter().map(OrgSummary::from).collect()
    }

    /// The Organisation `org_id`, when this node holds it.
    pub fn organisation(&self, org_id: OrgId) -> Option<OrgSummary> {
        self.service.list_orgs().iter().find(|record| record.org_id == org_id).map(OrgSummary::from)
    }

    /// The Persona's Member-as-a-group key and DevicePublicKey (org-node's
    /// `persona_public_keys`).
    pub fn persona_public_keys(&self, persona_id: &PersonaId) -> Result<(PersonPublicKey, DevicePublicKey), OrgNodeError> {
        self.service.persona_public_keys(persona_id)
    }
}

/// A Persona's public fields: its seeds are not copied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonaSummary {
    pub persona_id: PersonaId,
    pub org_id: Option<OrgId>,
    pub handle: Handle,
    pub name: Name,
    pub surname: Surname,
    pub member_id: Option<MemberId>,
    pub status: PersonaStatus,
}

impl From<&PersonaRecord> for PersonaSummary {
    fn from(record: &PersonaRecord) -> Self {
        Self {
            persona_id: record.persona_id.clone(),
            org_id: record.org_id,
            handle: record.handle.clone(),
            name: record.name.clone(),
            surname: record.surname.clone(),
            member_id: record.member_id,
            status: record.status,
        }
    }
}

/// An Organisation's public fields: its root, public key, epoch and Members.
/// The Organisation private key, the proxy account and the kept Change set
/// are not copied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgSummary {
    pub org_id: OrgId,
    pub root_hash: RootHash,
    pub org_pub_key: OrgPublicKey,
    pub epoch: Epoch,
    pub members: Vec<MemberSnapshot>,
}

impl From<&OrgRecord> for OrgSummary {
    fn from(record: &OrgRecord) -> Self {
        Self {
            org_id: record.org_id,
            root_hash: record.root_hash,
            org_pub_key: record.org_pub_key,
            epoch: record.epoch,
            members: record.trie_members.clone(),
        }
    }
}

/// The provisional update an admission or a revocation built, by its
/// identity in org-node's store and the root it was built on; it carries no
/// part of the update itself (review round 3, finding 2). `submit_commit_send`
/// takes one and writes the update org-node holds under it, and only when
/// that update is still built on the record's current root.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltUpdate {
    pub org_id: OrgId,
    pub base_root: RootHash,
    pub resulting_root: RootHash,
    pub org_pub_key: OrgPublicKey,
}

impl BuiltUpdate {
    /// The name of `update`, an admission's or a revocation's; `None` for a
    /// genesis update, which has no Organisation or base root yet.
    pub(crate) fn of(update: &ProvisionalUpdate) -> Option<Self> {
        Some(Self {
            org_id: update.org_id?,
            base_root: update.base_root?,
            resulting_root: update.resulting_root,
            org_pub_key: update.org_pub_key,
        })
    }
}

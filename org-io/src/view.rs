//! The values the handle's read surface returns (SDD-z3ychz): ids, names, public keys, roots, epochs, and the summaries
//! below, which copy the public fields of org-node's records and leave the
//! secrets they hold behind. `BuiltUpdate` names a provisional update
//! org-node holds without carrying it. The queries are `OrgIo`'s own
//! methods, each taking the service's lock inside.

use org_node::store::{MemberSnapshot, OrgRecord, PersonaRecord, PersonaStatus, ProvisionalUpdate};
use org_node::{Epoch, Handle, MemberId, Name, OrgId, OrgPublicKey, PersonaId, RootHash, Surname};

use crate::send::SendReport;

/// A Persona's public fields: its Member seed is not copied (and the store
/// holds no device seed).
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

/// What `submit_commit_send` reports of a committed update:
/// the Organisation and the committed epoch and root, public data only. The
/// commit's `CommitOutcome` stays in org-io: its Organisation-information
/// bodies carry the Organisation private key (LLR-3zdw8v).
/// `sends` is what became of each recipient's body, by Device
/// public key (LLR-znhmh2, LLR-r9sh6k).
#[derive(Debug)]
pub struct Submitted {
    pub org_id: OrgId,
    pub epoch: Epoch,
    pub root: RootHash,
    pub sends: SendReport,
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

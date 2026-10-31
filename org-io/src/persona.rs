//! A new Persona per proposal (SDD-bzu2jg): each provisional founding and each provisional
//! Member is a new Persona, created by the operation that proposes it — keychain item, then
//! store file, then its endpoint when the node receives — and never a held one reused
//! (LLR-kna4zj, LLR-jvx6j9).

use org_node::invitation::{Invite, InviteId};
use org_node::{DevicePublicKey, Epoch, OrgId, OrgPublicKey, RootHash};

use crate::transport::TransportError;
use crate::{CreateError, OrgIo};

/// What `propose_founding_organisation` built (LLR-4yte89): public data only.
#[derive(Debug)]
pub struct ProposedFounding {
    pub founder: DevicePublicKey,
    pub epoch: Epoch,
    pub root: RootHash,
    pub org_public_key: OrgPublicKey,
    /// The bind that failed after the creation, when the node receives (LLR-jvx6j9).
    pub bind: Option<TransportError>,
}

/// What `propose_member` built (LLR-6d6wfw): public data only.
#[derive(Debug)]
pub struct ProposedMember {
    pub member_device: DevicePublicKey,
    pub org_id: OrgId,
    pub invite_id: InviteId,
    pub org_name: String,
    /// The bind that failed after the creation, when the node receives (LLR-jvx6j9).
    pub bind: Option<TransportError>,
}

impl OrgIo {
    /// Propose a founding (LLR-4yte89, LLR-kna4zj): org-node builds a new Persona holding the
    /// provisional founding of `org_name`, from keys drawn from the handle's source; its Device seed
    /// goes to its keychain item, then its store file; no chain is touched.
    #[allow(clippy::result_large_err, reason = "as propose_person's: a refusal names its Device public key, a parsed point")]
    pub async fn propose_founding_organisation(
        &self,
        name: &str,
        surname: &str,
        handle: &str,
        org_name: &str,
    ) -> Result<ProposedFounding, CreateError> {
        let mut service = self.service.lock().await;
        let proposed = {
            let mut rng = self.local.rng.lock();
            service.propose_founding(&mut *rng, name, surname, handle, org_name.to_string())
        };
        let (founder, secret, change) = proposed.map_err(CreateError::Node)?;
        if let Err(refused) = self.local.keychain.keep(secret) {
            service.drop_store(&founder);
            return Err(CreateError::Keychain(refused));
        }
        self.local.store(&mut service, change).map_err(CreateError::Store)?;
        // Read through org-node's key-free Persona view (`persona_view`, which clones no private key), not the test-support `store_data`.
        // The store was just written, so it is held and not removed, and `persona_view` refuses
        // nothing else; `propose_founding` gave it its founding.
        let founding = service
            .persona_view(&founder)
            .ok()
            .and_then(|view| view.founding)
            .map(|founding| (founding.epoch, founding.root, founding.org_pub_key));
        drop(service);
        let Some((epoch, root, org_public_key)) = founding else {
            return Err(CreateError::FoundingNotListed { founder });
        };
        let bind = self.local.endpoints.accept_if_receiving(&founder).await.err();
        Ok(ProposedFounding { founder, epoch, root, org_public_key, bind })
    }

    /// Propose this node as a Member of the Invite's Organisation (LLR-6d6wfw, LLR-kna4zj): the
    /// Invite parsed through org-node, a new Persona holding the provisional Member and its
    /// expected first admission, written as `propose_founding_organisation`'s; it sends nothing.
    #[allow(clippy::result_large_err, reason = "as propose_person's: a refusal names its Device public key, a parsed point")]
    pub async fn propose_member(
        &self,
        invite_blob: &str,
        name: &str,
        surname: &str,
        handles: &[&str],
    ) -> Result<ProposedMember, CreateError> {
        let invite = Invite::parse(invite_blob).map_err(CreateError::Node)?;
        let mut service = self.service.lock().await;
        let proposed = {
            let mut rng = self.local.rng.lock();
            service.propose_member(&mut *rng, &invite, name, surname, handles)
        };
        let (member_device, secret, change) = proposed.map_err(CreateError::Node)?;
        if let Err(refused) = self.local.keychain.keep(secret) {
            service.drop_store(&member_device);
            return Err(CreateError::Keychain(refused));
        }
        self.local.store(&mut service, change).map_err(CreateError::Store)?;
        drop(service);
        let bind = self.local.endpoints.accept_if_receiving(&member_device).await.err();
        Ok(ProposedMember { member_device, org_id: invite.org_id, invite_id: invite.invite_id, org_name: invite.org_name, bind })
    }
}

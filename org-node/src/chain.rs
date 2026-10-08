//! What the chain says about one Organisation, as a value. OrgState mirrors
//! the OrgRegistry slot: (rootHash, orgPubKey, epoch). See spec §4.4.
//! org-node reads no chain: org-io reads the state and hands it to the
//! operations that judge against it (ruling B, change `worktree-org-io-create`).
use org_members::RootHash;

use crate::error::OrgNodeError;
use crate::types::{Epoch, OrgPublicKey};

/// The on-chain state of one org, as stored in the OrgRegistry slot. The
/// trusted-root oracle: the root it holds MUST come from a path the delta
/// sender does not control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrgState {
    pub root_hash: RootHash,
    pub org_pub_key: OrgPublicKey,
    pub epoch: Epoch,
}

impl OrgState {
    /// The edge where an Organisation state read from the chain enters the
    /// node: its Organisation public key must be a valid X25519 public key,
    /// or the state is refused (REQ-8jb4ny).
    pub fn from_chain(root_hash: [u8; 32], org_pub_key: [u8; 32], epoch: u64) -> Result<Self, OrgNodeError> {
        Ok(Self {
            root_hash: RootHash::new(root_hash),
            org_pub_key: OrgPublicKey::parse(&org_pub_key)?,
            epoch: Epoch::new(epoch),
        })
    }
}

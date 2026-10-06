//! Envelope: the wire form for a trie change, bound to an Organisation and
//! a Sequence number. It carries no signature (REQ-ag6kqm): the chain's
//! Membership root at a newer epoch decides whether the change is committed.
use org_members::delta::Delta;
use serde::{Deserialize, Serialize};

use crate::error::OrgNodeError;
use crate::ids::OrgId;
use crate::types::SequenceNumber;

/// An org-bound, sequence-bound trie delta.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    pub org_id: OrgId,
    pub parent_seq: SequenceNumber,
    pub delta_bytes: Vec<u8>, // postcard(Delta)
}

impl Envelope {
    /// Author side: encode `delta` and bind it to (org, seq).
    pub fn build(org_id: OrgId, parent_seq: SequenceNumber, delta: &Delta) -> Result<Self, OrgNodeError> {
        // to_allocvec on a valid Delta is infallible in practice; reuse MalformedDelta for the unreachable encode error.
        let delta_bytes = postcard::to_allocvec(delta).map_err(|_| OrgNodeError::MalformedDelta)?;
        Ok(Self { org_id, parent_seq, delta_bytes })
    }

    /// Decode the inner Delta from postcard bytes.
    pub fn decode_delta(&self) -> Result<Delta, OrgNodeError> {
        postcard::from_bytes(&self.delta_bytes).map_err(|_| OrgNodeError::MalformedDelta)
    }
}

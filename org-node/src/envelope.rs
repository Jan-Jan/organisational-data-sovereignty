//! SignedDeltaEnvelope: the authenticated wire form for a trie change.
//! Transcript signed = org_id (20) ‖ parent_seq LE (8) ‖ delta_bytes.
use ed25519_dalek::{Signature, VerifyingKey};
use org_members::delta::Delta;
use serde::{Deserialize, Serialize};

use crate::error::OrgNodeError;
use crate::ids::OrgId;
use crate::keys::{verify, SigningKeypair};
use crate::types::SequenceNumber;

/// Serde helper: serialize/deserialize `[u8; 64]` as a fixed-length byte array.
/// serde's derive does not implement these for arrays larger than 32 in all
/// configurations; this helper bridges the gap for postcard (and any other
/// Serializer that supports byte-array hints).
mod sig_bytes {
    use serde::{Deserializer, Serializer};
    use serde::de::{Error, SeqAccess, Visitor};
    use core::fmt;

    pub fn serialize<S: Serializer>(bytes: &[u8; 64], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(bytes)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 64], D::Error> {
        struct Vis;
        impl<'de> Visitor<'de> for Vis {
            type Value = [u8; 64];
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("64-byte signature")
            }
            fn visit_bytes<E: Error>(self, v: &[u8]) -> Result<Self::Value, E> {
                v.try_into().map_err(|_| E::invalid_length(v.len(), &"64"))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut out = [0u8; 64];
                for b in out.iter_mut() {
                    *b = seq.next_element::<u8>()?
                        .ok_or_else(|| A::Error::invalid_length(0, &"64"))?;
                }
                Ok(out)
            }
        }
        d.deserialize_bytes(Vis)
    }
}

/// A signed, org-bound, sequence-bound trie delta.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedDeltaEnvelope {
    pub org_id: OrgId,
    pub parent_seq: SequenceNumber,
    pub delta_bytes: Vec<u8>, // postcard(Delta)
    #[serde(with = "sig_bytes")]
    pub signature: [u8; 64],
}

/// Build the exact byte transcript that gets signed/verified.
fn transcript(org_id: &OrgId, parent_seq: SequenceNumber, delta_bytes: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(20 + 8 + delta_bytes.len());
    buf.extend_from_slice(org_id.as_bytes());
    buf.extend_from_slice(&parent_seq.get().to_le_bytes());
    buf.extend_from_slice(delta_bytes);
    buf
}

impl SignedDeltaEnvelope {
    /// Author side: encode `delta`, bind it to (org, seq), and sign with `author`.
    pub fn build(
        org_id: OrgId,
        parent_seq: SequenceNumber,
        delta: &Delta,
        author: &SigningKeypair,
    ) -> Result<Self, OrgNodeError> {
        // to_allocvec on a valid Delta is infallible in practice; reuse MalformedDelta for the unreachable encode error.
        let delta_bytes = postcard::to_allocvec(delta).map_err(|_| OrgNodeError::MalformedDelta)?;
        let sig = author.sign(&transcript(&org_id, parent_seq, &delta_bytes));
        Ok(Self { org_id, parent_seq, delta_bytes, signature: sig.to_bytes() })
    }

    /// Decode the inner Delta from postcard bytes (no signature check).
    pub fn decode_delta(&self) -> Result<Delta, OrgNodeError> {
        postcard::from_bytes(&self.delta_bytes).map_err(|_| OrgNodeError::MalformedDelta)
    }

    /// Verify the signature against a *known* member verifying key.
    /// Does NOT check org_id/seq/root — that is verify.rs's job.
    pub fn verify_signature(&self, author_member_key: &VerifyingKey) -> bool {
        let sig = Signature::from_bytes(&self.signature);
        verify(author_member_key, &transcript(&self.org_id, self.parent_seq, &self.delta_bytes), &sig)
    }
}


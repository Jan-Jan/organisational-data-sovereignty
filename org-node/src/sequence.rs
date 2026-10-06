//! Monotonic replay guard for envelope parent_seq. The trie's base_root gives
//! natural protection while history moves forward; SeqGuard defends the edge
//! case where a root recurs (add-then-remove). See org-members README §4.
use crate::error::OrgNodeError;
use crate::types::SequenceNumber;

/// Tracks the highest parent_seq committed for one org.
#[derive(Clone, Copy, Debug, Default)]
pub struct SeqGuard {
    last_seen: SequenceNumber,
}

impl SeqGuard {
    /// Starts at 0 (no envelope committed yet; the genesis trie is sequence 0).
    /// The first envelope must therefore carry `parent_seq >= 1`.
    pub fn new() -> Self {
        Self { last_seen: SequenceNumber::new(0) }
    }

    pub fn from_last_seen(last_seen: SequenceNumber) -> Self {
        Self { last_seen }
    }

    pub fn last_seen(&self) -> SequenceNumber {
        self.last_seen
    }

    /// Accept `seq` only if strictly greater than the last seen. Does not mutate.
    pub fn check(&self, seq: SequenceNumber) -> Result<(), OrgNodeError> {
        if seq > self.last_seen {
            Ok(())
        } else {
            Err(OrgNodeError::StaleSeq { got: seq.get(), last_seen: self.last_seen.get() })
        }
    }

    /// Commit `seq` as the new high-water mark. MUST be called only AFTER an
    /// envelope has FULLY verified (org binding, sequence check, AND
    /// the on-chain root match). Calling it earlier — e.g. right after `check`
    /// but before the root match — would advance the replay watermark for an
    /// envelope that may still be rejected, creating a replay-protection bypass.
    /// Forward-only: a `seq` not greater than the current mark is ignored.
    pub fn advance(&mut self, seq: SequenceNumber) {
        if seq > self.last_seen {
            self.last_seen = seq;
        }
    }
}


# Problem reports — org-node's own ledger opens

Five reports. The first moved here from org-members' ledger, where it was
filed on 2026-09-02 because that was the only ledger; it is an org-node
defect, and D14 puts a provider's defect in the provider's ledger. Its
opened date is the original's, and so is its text but for the two mechanical
changes noted at the foot of the report; its `affects:` line, which named an
org-members control a consumer may not cite, now names org-node's own
control.
Three of the other four were found by the code survey behind org-node's hazard
analysis (`org-node/docs/risk/2026-09-09-org-node-hazards.md`), and the last
(the resolved problem report on the Wire message's genesis_snapshot doc-comment, opened 2026-09-10) by review round 7 of the same change, which
found a doc-comment contradicted by an assessment that change makes. All are
recorded before anyone investigates a fix, as `resolve-problem`
requires. None is fixed in this change: it is a hazard analysis, and changing
what `service.rs` does, or what `chain_read.rs` and `transport/wire.rs` say
about themselves, under a register written the same day would mix the two.

## The chain reader documents the wrong finality (moved 2026-09-09)

## The publish path writes the chain before the record

## Loopback admission delivers to an address the joiner did not prove

*Moved 2026-10-13 (S4, task T12,
ruling A):* the report this section held — in Loopback mode the admission and
the Organisation's key were sent to the address the caller passed, with no
check that the address's identity was the joiner's Device key, so an altered
address handed the key to whoever it named while the record admitted the
joiner's genuine keys — moved to org-io with the send, to
`org-io/docs/problems/2026-10-13-loopback-address.md`,
where it is resolved by sending in Loopback mode only to an address whose
identity is a recipient's Device key. Its `affects:` line named RC-b6mydy
and the Loopback dial item (which moved with it). org-node's transitional
`send_body` still dials the address unchecked; org-io no longer calls it, and
task T14 deletes it. *(Deleted 2026-10-14, S4 task T14.)* The hazard it
instantiated, HAZ-ep6uzs, is unchanged in this register (below).

## The revocation receive path commits from a sender it never checks

## The Wire message documents a field the revocation path always sets (2026-09-10)

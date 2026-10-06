# Problems — the Organisation key pair, from its last review round

Recorded 2026-10-07 by the independent review of change
`worktree-org-node-org-key-pair` (round 1, the last round: every finding low).
Neither fix is mechanical, so both stay open.

**PR-gnh3j2**: `send_update` sends the Organisation private key the record
holds when it runs, not the key of the epoch the outgoing Envelope reaches, so
if a later update was committed before an earlier one is sent, the receiver
gets an earlier Envelope beside a later epoch's key, which REQ-szq3ud's
wording ("the Organisation private key of the epoch the update reaches") does
not allow.
affects: REQ-szq3ud, LLR-6ymd6d
opened: 2026-10-07
status: open

Review finding-4. No key is disclosed beyond what the record's current
Devices may hold, and the receiver refuses the message (its key does not match
the chain's key at the Envelope's epoch, or the Envelope is stale), so nothing
is committed wrongly; the defect is a requirement the tree does not meet in a
rare ordering. The fix is either to refuse a send whose Envelope is not the
record's current epoch (a new typed refusal, which the app must classify) or
to reword REQ-szq3ud to the record's key at the time of sending.

**PR-q8r32t**: `receive_and_verify` holds two `RevocationNotHeld` refusals
that no input can reach, the `(None, None)` arm and the `let … else` on the
first-admission branch, because an earlier arm already refuses every
revocation about an Organisation the node does not hold; no test can exercise
them, and they read as further REQ-vxqc5g sites.
affects: REQ-vxqc5g, LLR-38e2kn
opened: 2026-10-07
status: open

Review finding-5, also noted and left alone by the change's deslop pass.
Removing them means restructuring the receive path so the first-admission data
travels in the match; `unreachable!` is denied by the workspace lints in
production code.

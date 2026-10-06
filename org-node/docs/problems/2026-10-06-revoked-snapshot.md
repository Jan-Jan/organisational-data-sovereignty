# A revoked device is sent every member's record

Filed by `worktree-org-members-absence-proofs` during the 2026-10-06 sweep
that compared master's chain-authority change with
`docs/plans/2026-10-06-org-io-roadmap.md`. It is recorded here, and fixed in
stage S3 of that roadmap.

**PR-qmvj83**: a revocation sends the revoked device a Wire message carrying
the member snapshots as they were before the update, every member's record
included, so a device removed from the Organisation — possibly a stolen
one — receives the full membership on its way out.
affects: LLR-8hdu9x, HAZ-vxabf9
opened: 2026-10-06
status: open

Where: LLR-8hdu9x (`org-node/docs/architecture/2026-10-03-decomposition.md`)
states that the Wire message `send_update` sends carries the member snapshots
from before the committed update, and the revocation path sends that message
to the revoked device.

Why it is a defect: the owner ruled on 2026-10-06 that a revoked device
receives only an absence proof against the current on-chain root, never the
record or a delta. org-members' REQ-535jcd, REQ-yyuxh8 and REQ-tk2qqj provide
those proofs. A proof for a removed device carries only that Member's own
leaf, and a proof for a removed Member carries no Member data at all. The
snapshot the revoked device receives today discloses every other Member's
identity and keys. That is the disclosure org-members' risk analysis for
absence proofs bounds to a single leaf.

Fix, in S3: the revocation path sends the revoked device an absence proof
and no snapshot. The reproducing test sends a revocation to a device and
asserts the Wire message it receives holds no other Member's data.

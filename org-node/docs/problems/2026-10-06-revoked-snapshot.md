# A revoked device is sent every member's record

Filed by `worktree-org-members-absence-proofs` during the 2026-10-06 sweep
that compared master's chain-authority change with
`docs/plans/2026-10-06-org-io-roadmap.md`. It is recorded here, and fixed in
stage S3 of that roadmap.

**PR-qmvj83**: a revocation sends the revoked device a Wire message carrying
the member snapshots as they were before the update, every member's record
included, so a device removed from the Organisation — possibly a stolen
one — receives the full membership on its way out.
affects: LLR-8hdu9x, LLR-6ymd6d, LLR-js9dsu, HAZ-vxabf9
opened: 2026-10-06
status: resolved
resolution: 2026-10-07 (change `worktree-org-io-commit-workflow`, S3 T8) — a
commit returns one revocation notice per removed Device, holding only that
Device's identity and an absence proof (LLR-kr5t6f, LLR-a8z7r5), and
`send_update` sends a removed Device its notice and nothing else (LLR-6ymd6d,
LLR-8hdu9x as amended); reproduced by `a_revoked_device_receives_only_its_notice`
(`org-node/tests/service_stories.rs`): red before, green after.

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

*Note 2026-10-07 (change `worktree-org-node-org-key-pair`, at its merge of
master `c036fab`):* that change makes the Wire message two kinds and sends a
Device the committed record does not list a revocation carrying the Envelope
alone, with no member snapshot and no Organisation private key (LLR-8hdu9x and
LLR-6ymd6d as amended there; test
`send_update_sends_organisation_information_to_a_listed_device_and_a_revocation_to_any_other`).
The disclosure described above, every Member's record, no longer occurs. The
item stays open for the owner's fix: the revoked Device receives an absence
proof rather than the Envelope's Change set, in S3.

*Note 2026-10-07 (change `worktree-org-io-commit-workflow`, stage S3, after
its merge of master `1f52c36`).* What remains, and what S3 fixes: the
revocation still carries the committed Envelope, whose Change set holds every
leaf the update upserts. With today's batches of one that is at most the
revoked Member's own leaf, but a provisional update is a batch (the key-pair
change's ruling), and then the Change set carries other Members' leaves. The
revoked Device also cannot act on it unless it holds the Change set's base
record and the chain still holds that update's root. S3 replaces the
Envelope with a notice holding only the Device's identity and an absence
proof (REQ-ps2gy2, LLR-kr5t6f, LLR-dc45ur; REQ-3dsweu, LLR-js9dsu,
LLR-6ymd6d and LLR-8hdu9x amended), verified against the chain's current
root (REQ-m2xh8q). The reproducing test, written red first in S3, revokes one
Device in a two-Member Organisation and asserts the message the revoked
Device receives decodes to a revocation holding no Envelope, Change set,
snapshot or key and no leaf of the other Member. Still open until S3 merges.

*Resolved 2026-10-07 (change `worktree-org-io-commit-workflow`, plan task
T12).* S3 T8 replaced the Envelope with the notice; see the `resolution:`
line above.

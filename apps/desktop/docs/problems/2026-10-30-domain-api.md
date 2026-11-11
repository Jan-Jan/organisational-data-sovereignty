# Problems — found while developing change `worktree-org-io-domain-api`

Recorded 2026-10-30 at the merge review of change
`worktree-org-io-domain-api` (merge review round 1, finding-3, recorded at
plan task T62; round 2, finding-4). The file is a DRAFT: the merging change
dates it and resolves the items it fixes.

**PR-2t6ssm**: a doubled send of this node's own adoption cannot be seen at the app's interface: `verify_and_send` attaches one report per Device, and a second send of the same epoch would report `sent` as the first did, so no app test can tell one send from two.
affects: LLR-es49wp, REQ-qv4d2z
opened: 2026-10-30
status: open

Where: `verify_and_send` (`apps/desktop/src-tauri/src/commands.rs`) asks org-io's
`send_organisation` once for an `adopted` result with `own` true
(LLR-es49wp), and the app test
`apps/desktop/src-tauri/tests/verification.rs::an_own_adoption_is_sent_once_and_nothing_else_is`
asserts one report per Device with the result `sent`. That assertion holds
for a doubled send too: org-io's receiver answers an Organisation
information it already holds (`Checked::AlreadyHeld`) with
`Verdict::Accepted` and reports nothing (`crates/org-io/src/receiver.rs`), which
the sender reports as delivered, and the app keeps only the per-Device
report of the send it attached. The request side is pinned (one request,
Bob's Device answering exactly once); the send side is not.

Why not fixed here: the send count is visible only inside org-io (or at a
recipient that counts what it is sent), not at any interface the app
owns; pinning it from the app needs either a counting seam on org-io's
transport or a recipient-side report of a repeated send, which is a change
to org-io's interface, not a mechanical app fix. Raised at merge review
round 1 (finding-3, partly fixed at T62) and round 2 (finding-4, whose
comment correction in `verification.rs` now cites this report).

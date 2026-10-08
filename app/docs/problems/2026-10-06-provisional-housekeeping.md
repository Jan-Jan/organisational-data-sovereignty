# Problem reports — the app's handling of kept provisional updates

**PR-924ftr**: the app never retries the commit of a provisional update whose
chain write timed out and then executed, and offers no way to discard one:
`submit_commit_send` (`app/src-tauri/src/submit.rs`) is the only caller of
org-node's `commit_update`, which commits whichever kept update matches the
chain root, so the next admission can commit the earlier, timed-out update and
send it to the new recipient under the new invite identifier; and every failed
founding leaves another genesis update holding an Organisation private key,
because no app command calls `discard_provisional`.
affects: REQ-nfr3n2
opened: 2026-10-06
status: open

*Re-pointed 2026-10-08 (change `worktree-org-io-create`, task T9).*
`submit_commit_send` is now org-io's (`org-io/src/submit.rs`), reached
through `OrgIo::submit_commit_send`; `app/src-tauri/src/submit.rs` is
deleted. The two low-level requirements this report affected — the
write-then-commit-then-send order and the 90-second bound — moved with it to
org-io's architecture ledger (`org-io/docs/architecture/2026-10-08-org-io.md`),
which does not export them, so they leave the `affects:` line and are named
here in prose; REQ-nfr3n2 stays, exported by org-io. The defect is
unchanged: since the move org-io reads the chain between the write and the
commit, but it still commits whichever kept update matches, and no app
command retries or discards one. It is still resolved by stage S3.

Found 2026-10-06 by independent review round 1 of change
`worktree-org-node-chain-authority` (finding-3, code, low). Under the review's
convergence rule a low finding whose fix is not mechanical is recorded as an
open problem item. The app's risk text for the timeout
(`app/docs/risk/2026-10-06-invitation.md`) says a write executed after the
timeout is reached by `commit_update` from the kept update; that holds for the
org-node operation but no app path invokes it for that purpose. The fix is an
app change of its own: a commit-retry command naming the update, a discard
command, and sending only the update the call submitted.

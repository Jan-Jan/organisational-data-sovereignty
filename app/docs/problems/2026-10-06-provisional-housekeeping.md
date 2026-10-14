# Problem reports — the app's handling of kept provisional updates

*Moved 2026-10-13 (change `worktree-org-io-publish`, task T17a; ruling A of
the owner's rulings of 2026-10-06, `docs/plans/2026-10-06-org-io-roadmap.md`).*
The one report this file defined — that a chain write which timed out and
then executed was never retried, so the next admission could commit the
earlier update and send it to the new recipient, and that no way existed to
discard a kept provisional update — moved, with its ID and opened date, to
org-io's problems ledger (the file in `org-io/docs/problems/` written by
change `worktree-org-io-publish`, slug `publish`), where it arrives resolved:
the code it describes has been org-io's since change `worktree-org-io-create`,
and org-io's settle of an unknown write and its background discard fix it.
This file defines no item since.

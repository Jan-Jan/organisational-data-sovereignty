# Problems — the signatory-set reads, from S2's review

Recorded 2026-10-08 by the independent review of change
`worktree-org-io-create` (round 1, finding-11). The file is dated, not a
DRAFT, because that change's ledgers were already finalized when the review
ran.

**PR-ksm2ua**: The two signatory-set reads, `Revive.OriginalAccount` (REQ-8p2veg) and `Proxy.Proxies` (REQ-v8jczx), each resolve "latest finalised" separately, so one signatory-set answer can combine an original account from one block with proxy delegates from a later one.
affects: REQ-8p2veg, REQ-v8jczx, SDD-rxfu6h
opened: 2026-10-08
status: resolved
resolution: 2026-10-13 (change worktree-org-io-publish, task T16a; owner ruling D5) — root cause: each read resolved the latest finalised block itself; fixed by `OrgRegistryClient::signatory_set` (REQ-b8x2py, LLR-sc79gq), which resolves the block once and reads both values and the `Any` index at it, org-io's only consumer now calling it alone; reproduced by `on-chain-client/tests/signatory_set_decode.rs` `the_signatory_set_is_read_at_one_block`.

Where: `fetch_storage_raw` calls `self.api.at_current_block()` on every call
(`on-chain-client/src/client.rs:263-268`), and each read calls it once. If a
block finalises between the two calls, and it changes the H160 mapping or
the proxy definitions, the pair describes no single chain state. Neither
requirement forbids this: each states its own read "at the latest finalised
block". No S2 caller acts on the answer (org-io's own-admin check is
implemented but not wired into the app); the first caller is S3b-io, stage
S3's org-io half, which should either read both at one pinned block or
state why the split is harmless.

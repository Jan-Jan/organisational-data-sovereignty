# Problem reports — the chain reader keeps a superseded state after a failed fetch

**PR-k2xxaq**: `OnChainReader::refresh` (`org-node/src/chain_read.rs`) returns
early when fetching the Organisation state from the chain fails (a network or
RPC error), leaving the last good state cached, so `get_org_state` keeps
serving a root and epoch the chain may already have superseded.
affects: LLR-mmdu38
opened: 2026-10-05
status: resolved
resolution: root cause — `OnChainReader::refresh` returned early on a failed
fetch without clearing its cached state. Fix — deleted with `OnChainReader`
and `OrgStateCache` (`org-node/src/chain_read.rs` removed) by task T6 of
change `worktree-org-io-create` (ruling B: org-node reads no chain and caches
no chain state; org-io reads the state afresh for each decision). LLR-mmdu38's
cache clause is retired with the cache. Verified by the absence test of
LLR-mn5c2q (`org-node/tests/absences.rs`,
`org_node_names_no_chain_library_and_reads_no_chain`, which refuses
`OnChainReader` and `OrgStateCache` anywhere in org-node's source).

Found 2026-10-05 by the fix for independent-review round 3 of the org-node
type-safety change. That change made a state refused at parse clear the cache
(`OrgStateCache::store_fetched`, fail closed); a fetch that fails before any
state arrives still leaves the cache as it was. The behaviour predates the
change. `OnChainReader` is not on the production Receive path:
`SubxtChainOps::read_state` caches nothing and both Receive paths read the
state afresh and abort on an error. By owner ruling (2026-10-05) this is
recorded and left for its own change: whether a failed fetch should clear the
cache, or keep it with an age bound, is a decision about availability against
staleness that the type-safety change does not make.

**2026-10-05, at the merge of master `1feb608` into
`worktree-person-shared-types`.** LLR-mmdu38 is amended in place there
(docs/plans/2026-10-05-switch-trim.md) and keeps its cache clause unchanged,
so this report stands against it.

**2026-10-08, change `worktree-org-io-create`, task T6.** Resolved by
deletion: `OnChainReader` and `OrgStateCache` are gone with
`chain_read.rs`; the absence is LLR-mn5c2q's test.

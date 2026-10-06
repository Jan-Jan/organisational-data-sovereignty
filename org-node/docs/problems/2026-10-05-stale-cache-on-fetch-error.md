# Problem reports — the chain reader keeps a superseded state after a failed fetch

**PR-k2xxaq**: `OnChainReader::refresh` (`org-node/src/chain_read.rs`) returns
early when fetching the Organisation state from the chain fails (a network or
RPC error), leaving the last good state cached, so `get_org_state` keeps
serving a root and epoch the chain may already have superseded.
affects: LLR-mmdu38
opened: 2026-10-05
status: open

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

# Risk analysis — the signatory-set reads (on-chain-client)

Run 2026-10-07 for change `worktree-org-io-create`, under IEC 62304 Class C
and the acceptability matrix in this ledger's README. The change adds a
derived read
(`crates/on-chain-client/docs/requirements/2026-10-08-signatory-reads.md`)
that org-io's own-admin check consumed.

## Derived requirements assessment

**REQ-8p2veg** (the account pallet-revive maps an Organisation's H160 to): no
hazard impact in S2. The consumer acts on the answer only by reporting it: a
wrong "admin" answer cannot authorise a write, because the chain refuses a
dispatch through the proxy from an account that is not its delegate. A value
that is not a 32-byte account is a typed error, reported as such, never read
as "not mapped", so a malformed chain value cannot pass for an absent mapping.

assesses: REQ-8p2veg

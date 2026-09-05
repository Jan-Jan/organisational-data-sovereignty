# Problem reports — the chain reader documents the wrong finality

**PR-hvg2dy**: `OnChainReader::get_org_state` in `org-node/src/chain_read.rs`
is documented as fetching "the latest (current best) state", while the read it
performs passes `at = None` to `OrgRegistryClient::get_org_state`, which the
client documents as reading at the latest **finalised** block. The two
statements cannot both be true, and the doc-comment is the one that is wrong.
affects: RC-9z65hw
opened: 2026-09-02
status: open

Found while writing the hazard analysis
(`docs/risk/2026-09-02-membership-hazards.md`), not by a failing test. The
register had to state which block the decisive control reads, and the sources
disagree: `on-chain-client/src/client.rs` documents `at = None` as the latest
finalised block, `org-node/src/preflight.rs` treats it as finalised, and the
reader's own doc-comment calls it current best.

Why this matters more than a stale comment usually would. The read is step 7 of
`verify_envelope_against_chain` — the independent trusted root that the whole
anchor rests on, and the reason a membership change is committed at all. Best
and finalised differ exactly when a reorg is in flight, which is the case the
control exists for: a root read from a best block can be reorged away, and a
revocation committed against it would be believed on evidence the chain later
discards. A reader who trusts this comment concludes the control is weaker than
it is and may add a redundant finality check; a reader who trusts it while
*writing* a second reader may implement the best-block read the comment
describes. Both directions are worse than the truth.

There is a second, smaller inaccuracy in the same place: the method returns a
cached snapshot, refreshed only when a caller awaits `refresh()`, so at verify
time no block is read at all. The freshness of the root therefore depends on
the caller's refresh discipline, which is not documented anywhere near the
control that depends on it.

Not a code defect as far as this analysis established — the finalised read is
the correct behaviour and appears to be what the code does. What is wrong is
the documentation of a security-relevant control, which under IEC 62304 Class C
is not a lesser category of defect. The fix is to correct the doc-comment to
state the finalised read and the caching behaviour, and to say which of the
three descriptions the code is actually contracted to.

Not fixed in this change: this is the risk-analysis change, and it is
documentation-only in the regulated ledgers. Touching crate source under it
would mix a hazard analysis with a code change, the same reason PR-zz4exm was
left open under the requirements tooth.

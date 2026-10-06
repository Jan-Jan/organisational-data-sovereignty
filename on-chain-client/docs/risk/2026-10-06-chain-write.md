# Risk analysis — writing the chain (on-chain-client)

Run 2026-10-05 for change `worktree-org-node-chain-authority`, under IEC 62304
Class C and the acceptability matrix in this ledger's README. The owner moved
the genesis ceremony and update submission from org-node into this unit
(`on-chain-client/docs/requirements/2026-10-06-chain-write.md`).
The behaviour moves unchanged; org-node's register analysed it where it was
(`org-node/docs/risk/2026-09-09-org-node-hazards.md`, the multisig dispatch and
publish-before-persist sections), and this file assesses what the move changes
for this unit.

## Derived requirements assessment

**REQ-6jefu2** (genesis: pure proxy, funding, mapping, genesis record) and
**REQ-aat4yt** (update submission through the proxy): no new hazard in this
unit. Both take the signatory key as an argument and store nothing, so key
custody stays with the caller, as it did in org-node, where the app supplied
the key. Each failure mode the earlier analysis named — a step that does not
execute, an update left pending approval — is a typed error by the
requirements' own wording, which is what lets a caller decline to act on an
update the chain did not take. What this unit gains is a write path its
reading requirements never had to consider: a write cannot corrupt what the
reader returns, because the reader reads the finalised Organisation slot and
the write goes through the contract, so the reader's controls are unaffected.

assesses: REQ-6jefu2, REQ-aat4yt

## Derived low-level requirement of the design (design-architecture, 2026-10-05)

**LLR-rxs5ec** (the writer compiles only with the `write` feature, which adds
exactly `subxt-signer` and `blake2`, and the crate depends on neither org-node
nor org-members): no new hazard. It is a containment property: a build without
`write` — every build of this unit's reader, and every consumer that does not
ask for the writer — compiles no code that can sign or submit, so the reading
requirements' evidence is gathered on the same code as before. Forbidding a
dependency on org-node or org-members keeps the dependency edge one-way
(org-node and the app depend on this unit), so the writer cannot come to rely
on a consumer's types or behaviour that this unit's register has not assessed.

assesses: LLR-rxs5ec

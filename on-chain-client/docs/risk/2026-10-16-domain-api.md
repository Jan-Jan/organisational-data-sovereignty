# Risk analysis — the genesis ceremony in two steps (on-chain-client)

Run 2026-10-16 for change `worktree-org-io-domain-api` (owner answer A4),
under IEC 62304 Class C and the acceptability matrix in this ledger's
README, for
`on-chain-client/docs/requirements/2026-10-16-domain-api.md`.

## Requirements assessment

**REQ-v86pyy** (creating the proxy, then founding at it): no new hazard. The
steps, their order, the expected epoch of zero and the step-naming errors
are REQ-6jefu2's, already assessed in
`on-chain-client/docs/risk/2026-10-06-chain-write.md`; splitting them only
returns the proxy account to the caller between creating and funding it, so
a founding interrupted after the proxy exists can be resumed at that proxy
rather than leaving a funded proxy no device can find.

**REQ-6vh3ht** (resuming the founding step): no new hazard. Deciding what is
done from the chain, never from memory, means a retry after a crash or a
restart neither funds a proxy twice nor skips a step the chain does not
show done. A mistaken skip could leave a proxy unfunded or unmapped, which
makes the next step fail with its step named; it cannot record a wrong
Membership root, which only the record step writes and which the contract
accepts once, at expected epoch zero.

assesses: REQ-v86pyy, REQ-6vh3ht

## Design assessment

Run 2026-10-20 for the design in
`on-chain-client/docs/architecture/2026-10-20-domain-api.md`.

**The resumption rules** (LLR-jk76wf, LLR-95puh9, LLR-g8qevz): no new
hazard. They decide from two chain reads at the latest finalised block —
the proxy's free balance and its pallet-revive mapping — and never from the
Organisation's slot, which only the record step writes and the contract
accepts once, at expected epoch zero. No rule can cause a wrong Membership
root or Organisation public key to be recorded, nor a record to be reported
as executed when it did not execute (LLR-chy957), so none feeds a hazard of
this unit's RMF, all of which concern what the chain is said to hold.

**Funding twice and a second proxy**: a cost, not a hazard. A founding
interrupted after a transfer was submitted but before it was finalised is
not shown funded at the finalised block, so a re-run may transfer
`FUND_AMOUNT` once more; a caller that runs `create_proxy` again rather than
re-using the proxy it was given leaves a funded, unmapped or unrecorded
proxy that no Organisation names. Either loses at most `FUND_AMOUNT` of the
signatory's balance per interrupted attempt and changes no Organisation
slot: no Membership decision reads an orphan proxy, and the Organisation
admin is derived from the proxy the founding completed at. Keeping the
proxy between the two steps is the consumer's (org-io's) requirement. A
repeated mapping is refused by pallet-revive and reported as `MapAccount`
not executed (LLR-chy957); the next re-run sees the proxy mapped.

assesses: LLR-jk76wf, LLR-95puh9, LLR-g8qevz, LLR-chy957

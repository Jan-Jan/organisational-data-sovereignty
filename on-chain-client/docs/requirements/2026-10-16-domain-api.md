# Requirements — the genesis ceremony in two steps

Added 2026-10-16 by change `worktree-org-io-domain-api` (owner answer A4 of
that day). Consumer: org-io, which stores the proxy's address between the
two steps so that a founding interrupted after the proxy exists can be
retried at the same proxy; named here in prose only, as this unit depends
on no other. Both items meet org-io's expectation of a two-step genesis,
and are assessed in
`on-chain-client/docs/risk/2026-10-16-domain-api.md`.

REQ-6jefu2 (the genesis ceremony in one call) is not superseded: the one-call
ceremony may stay as the composition of the two steps, and its behaviour is
unchanged.

**REQ-v86pyy**: The software shall offer the genesis ceremony of REQ-6jefu2
in two steps: one that, given a signatory key and the co-signatories,
creates a pure proxy controlled by that multisig (or by the signatory
directly when there are no co-signatories) and returns its account; and one
that, given the signatory key, the co-signatories, that proxy account, a
genesis Membership root and an Organisation public key, funds the proxy,
maps it in pallet-revive and records the genesis Membership root and
Organisation public key in its Organisation slot with an expected epoch of
zero, each step only after the previous one executed, returning the proxy
account and the Organisation admin, and returning a typed error naming the
step when any step does not execute.
satisfies: REQ-wd683g
exported: yes

**REQ-6vh3ht**: The software shall, when the founding step of REQ-v86pyy is
run on a proxy account that the chain shows already funded, or already
mapped in pallet-revive, skip that step, deciding what is done from reads
of the chain and never from anything it holds in memory, and carry out every
step the chain does not show done.
satisfies: REQ-wd683g
exported: yes

A retried founding therefore neither transfers the funding twice nor maps
the proxy twice. Recording the genesis root is not skipped: once an
Organisation slot holds a genesis record, a second record carrying an
expected epoch of zero is refused by the contract's expected-epoch check,
and the step is reported as not executed.

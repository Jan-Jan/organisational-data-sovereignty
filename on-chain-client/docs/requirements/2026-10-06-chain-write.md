# Requirements — writing the chain: genesis and update submission

Decided 2026-10-05 in the `grill-requirements` interview of change
`worktree-org-node-chain-authority`. The right to change an Organisation's data
lies in its on-chain multisig proxy, and the owner ruled that org-node no
longer writes to the chain: the genesis ceremony and update submission move
from org-node (`org-node/src/ceremony.rs`, `org-node/src/chain_write/`) to this
unit, and the app calls them. org-node keeps reading the chain through this
unit and verifies updates against what it reads.

Both requirements are `satisfies: derived`: they exist because of where the
owner placed the chain write, not because a system-needs document asked for it.
Terms: *Organisation admin*, *Organisation slot* and *Epoch* are defined in
`on-chain-client/docs/CONTEXT.md`; *Organisation*, *Organisation state* and
*Membership root* in the root `docs/CONTEXT.md`.

**REQ-6jefu2**: The software shall, given a signatory key, the co-signatories
of the multisig that is to control the Organisation, a genesis Membership root
and an Organisation public key, create a pure proxy controlled by that
multisig (or by the signatory directly when there are no co-signatories), fund
it, map it in pallet-revive, record the genesis Membership root and
Organisation public key in its Organisation slot with an expected epoch of
zero, and return the proxy account and the Organisation admin; and shall
return a typed error, naming the step, when any step does not execute,
including a call made through the proxy whose own result is a failure.
satisfies: derived
exported: yes

*Amended 2026-10-06 (independent review round 1, finding-2 and finding-5).*
"Signing key" read "Organisation public key" (the root glossary lists the
former under _Avoid_); "at epoch zero" meant the expected epoch the call
carries, which the contract turns into epoch 1; and a call through
pallet-proxy whose inner call fails is a step that did not execute, which the
first wording left implicit and the code did not check.

**REQ-aat4yt**: The software shall, given a signatory key, the co-signatories,
an Organisation's proxy account, a new Membership root, the Organisation
public key and the epoch the update expects the Organisation slot to hold,
submit the update to that Organisation slot through the proxy and return only
once it has executed, and shall return a typed error when the update fails —
including when the proxy reports the inner call failed — or is left pending
approval.
satisfies: derived
exported: yes

*Amended 2026-10-06 (independent review round 1, finding-2 and finding-5):*
"the signing key" reads "the Organisation public key", and the inner call's
failure reported by the proxy is named as a failure.

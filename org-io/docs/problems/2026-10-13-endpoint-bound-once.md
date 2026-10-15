# One endpoint per service

*Moved 2026-10-13 from org-node's problems ledger
(`org-node/docs/problems/2026-10-04-endpoint-bound-once.md`, which keeps a
prose pointer here), plan task T10 of
`docs/plans/2026-10-06-org-io-transport.md` (ruling A): the endpoint moved to
org-io, and the fix landed here. Its `affects:` line named, besides the
endpoint lifecycle items that moved with it, org-node's admission item and
two invitation items that left org-node in 2026-10-06; those are named in
prose below.*

Opened by org-node's architecture tooth
(`docs/plans/2026-10-03-org-node-architecture.md`) from review round 5.

**PR-8qsnhx**: `ensure_endpoint` binds the service's transport endpoint once,
from whichever Persona first needs it. Every later send reuses that endpoint,
whatever Persona the call names. A device administering two Organisations
through two Personas sends the second Organisation's admissions under the
first Persona's device key. The joiner's invite names the second Persona's
device, so the joiner refuses the push, after the administrator has already
moved the chain and its own record.
affects: SDD-b8tuv3, LLR-cns6q6, LLR-6zjzn2
opened: 2026-10-04
status: resolved
resolution: fixed 2026-10-13 (change `worktree-org-io-transport`, plan task
T10): org-io binds one endpoint per Persona, keyed by the Persona's Device
public key and bound lazily from that Device's keychain item
(`Endpoints`, `OrgEndpoint::bind_from_keychain`; LLR-d2pzzk, LLR-d5y7ks,
REQ-bupq5y), and sends each committed update from the endpoint of the
Persona the commit names as its sender. The pin is inverted:
`a_second_organisations_message_goes_out_under_its_own_personas_key`
(`org-io/tests/endpoints.rs`) founds two Organisations through two Personas
of one node and sees each admission arrive under its own Persona's Device
key, and two endpoints bound. LLR-6zjzn2 and LLR-cns6q6 are rewritten to the
per-Persona rule (dated notes in org-io's design ledger). org-node's
transitional `send_body` and `bind_endpoint`, which still bind once per
service, are no longer called by org-io; task T14 deletes them, with
org-node's pin of this report.

## What was observed

`pr_8qsnhx_a_second_organisations_admission_goes_out_under_the_first_personas_key`
(`org-node/tests/admission_sender.rs`) builds one administrator service with
two Personas and two Organisations, and injects no endpoint:

- The first admission binds the endpoint from Persona 1 and is accepted.
- The second admission reports success to the administrator and advances org
  2's on-chain epoch. The authenticated sender is Persona 1's device, and the
  invite names Persona 2's.

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`).* The joiner no longer refuses the second
admission: the owner ruled that nothing about the sender is checked, so the
joiner commits a chain-valid admission whoever delivers it. What remained of
the defect is that the second Organisation's admission goes out under Persona
1's device key, which the pin asserted by capturing the push at a relay.

*Noted 2026-10-13 (S4, task T10):* since S3a's member-sender rule (owner
ruling R1 of 2026-10-07) the receiver does check the sender of an update for
an Organisation it holds against the Devices its record lists, so a later
update sent under the wrong Persona's key would again be refused. The fix
removes the cause either way.

## Why the gate did not see it

The two-Organisation admission test injected Persona 2's endpoint by hand.
Every other administrator test had one Persona.

## Why it matters

It is a silent, unrecoverable failed admission: the published root and what a
member holds diverge without any caller mistake.

## Widened by review rounds 6 and 8 — the readers

The join-request and Invite exports read the bound endpoint's address, so
with the endpoint bound from another Persona each blob named two devices. Both
readers left org-node with the invitation exchange on 2026-10-06 (change
`worktree-org-node-chain-authority`, T7), with their pins; the sending side
remained until this fix.

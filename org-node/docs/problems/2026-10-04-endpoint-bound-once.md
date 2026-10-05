# Problem reports — org-node, one endpoint per service

Opened by the architecture tooth
(`docs/plans/2026-10-03-org-node-architecture.md`) from review round 5.

**PR-8qsnhx**: `ensure_endpoint` binds the service's transport endpoint once,
from whichever Persona first needs it. Every later send reuses that endpoint,
whatever Persona the call names. A device administering two Organisations
through two Personas sends the second Organisation's admissions under the
first Persona's device key. The joiner's invite names the second Persona's
device, so the joiner refuses the push, after the administrator has already
moved the chain and its own record.
affects: SDD-b8tuv3, SDD-rx2yvy, LLR-cns6q6, LLR-6zjzn2, LLR-437fvx, LLR-qezw3n
opened: 2026-10-04
status: open

## What was observed

`pr_8qsnhx_a_second_organisations_admission_goes_out_under_the_first_personas_key`
(`org-node/tests/admission_sender.rs`) builds one administrator service with
two Personas and two Organisations, and injects no endpoint:

- The first admission binds the endpoint from Persona 1 and is accepted.
- The second admission reports success to the administrator and advances org
  2's on-chain epoch. The joiner refuses it with `BadSignature`, because the
  authenticated sender is Persona 1's device and its invite names Persona 2's.

The joiner holds nothing. The administrator's record and the chain both list
it. Nothing is retried and no error reaches the administrator.

## Why the gate did not see it

`a_second_organisation_is_admitted_into_without_touching_the_first` injects
Persona 2's endpoint by hand with `with_endpoint`. Every other administrator
test has one Persona. Review round 4 declared the receive side's version of
this assumption, `personas.first()` on both receive paths, as a deliberately
unrefined behaviour. The sending side was not examined.

## Why it matters

It is a silent, unrecoverable failed admission. It is the same divergence
between the published root and what a member holds that
`org-node/docs/risk/2026-10-03-architecture-derived.md` §D assesses, reached
here without any caller mistake. LLR-cns6q6 ("the identity the transport
authenticates is that persona's device key") is true only for the Persona the
endpoint was first bound from, and it is narrowed to that in the ledger.

## What closing it looks like

An endpoint per Persona, or a rebind when the named Persona differs from the
bound one. Either way the service API changes, and that belongs in a change of
its own, under TDD with the pin test inverted. The receive side's
`personas.first()`, recorded in the decomposition as a deliberately unrefined
behaviour, has the same cure and should be closed by the same change.

## Widened by review round 6 — a reader

`export_join_request` reads the bound endpoint's address for the join
request's dialling address, and derives the device key from the Persona it is
asked for. With the endpoint bound from another Persona, the blob names two
different devices. The app binds its receiver from `personas.first()` and lets
any Persona export a join request, so the app can reach this. In Networked
mode an administrator dials by the device key, which nothing is listening on,
so the send fails after `submit_update` has moved the chain (PR-vt244s's
state). `pr_8qsnhx_a_join_request_advertises_the_bound_endpoint_not_its_personas`
pins it. The cure is the one above: an endpoint per Persona. Folded in for
that reason.

## Widened by review round 8 — a second reader

`export_invite` reads the same address for the Invite's
`admin_node_addr` (`org-node/src/service.rs:673`), and an empty one when no
endpoint is bound. (Amended 2026-10-05 by the org-node type-safety change, review round 7: citation correction only — the line read `:688` before that
change's edits to `service.rs`.) LLR-qezw3n states it. The Invite's device key is the
administrator Persona's, so with the endpoint bound from another Persona the
Invite names two devices, as the join request does. The joiner's sender check
uses the key, not the address, so this costs the dial-back only. Same cure.
`pr_8qsnhx_an_invite_advertises_the_bound_endpoint_not_its_administrators`
pins it (review round 9).

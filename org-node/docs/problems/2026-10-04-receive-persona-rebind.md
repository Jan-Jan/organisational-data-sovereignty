# Problem reports — org-node, the receive path rebinds Personas across Organisations

Opened by the architecture tooth
(`docs/plans/2026-10-03-org-node-architecture.md`) from review round 5, which
reproduced it on the tree that review round 4 had declared the lookup class
closed on.

**PR-mdv38y**: `receive_and_verify` chooses the Persona to mark Active from
every Persona whose device key is in the verified trie, whatever Organisation
that Persona is bound to, and then overwrites its Organisation binding and
member id. A device holding two Organisations has one Organisation's Persona
silently rebound to the other the moment the other's administrator enrols that
Persona's device key, after which ordinary updates and revocations are judged
against the wrong Organisation.
affects: SDD-8cpyfa, SDD-72ddm6, LLR-y2v8v2, LLR-jwhzh3, LLR-jsx922, LLR-e5c9ud, LLR-w3fhhg, LLR-q3aj8z
opened: 2026-10-04
status: open

## What was observed

The site is the `my_persona_id` selection in `receive_and_verify`
(`org-node/src/service.rs:1001-1013`): `.find` over every Persona, filtered on
device-key membership of the verified trie and on not being the administrator's Member-as-a-group key, **never on `p.org_id`**. The Persona it finds is then given
`org_id = Some(org_id)` and the new member id (`:1061-1065`).
(Amended 2026-10-05 by the org-node type-safety change, review round 7: citation correction only — the ranges read `:1040-1052` and `:1100-1104` before that
change's edits to `service.rs`.)
`PersonaRecord.org_id` holds one Organisation.

`pr_mdv38y_the_receive_path_rebinds_another_organisations_persona`
(`org-node/tests/admission_sender.rs`) pins the sequence. The device B holds
org 1 through Persona b1 and org 2 through Persona b2.

1. Org 2's administrator admits b1's join request. B receives it on
   `receive_and_verify`. **b1 is rebound to org 2** and its member id replaced.
2. Org 1's administrator admits somebody else and pushes the change to B on
   the self-delete path. B is still in org 1's trie, but no Persona now names
   org 1, so **B deletes its org 1 record**.
3. Org 2 revokes b2. b1, now bound to org 2, is still in org 2's trie, so
   **B stays in org 2**.

Every assertion in that test is the defect. Correcting it reddens them, and
that is the signal to rewrite the test against the corrected behaviour.

## Why the gate did not see it

Review round 4 wrote
`membership_of_one_organisation_is_judged_by_that_organisations_personas_alone`
for step 3's attack and delivered the enrolment through
`receive_and_self_delete_if_revoked`, the one receive path that never rebinds
a Persona. The test is a true test of that path, and LLR-jwhzh3 is true of that
path. It is not true of the device, because the other path rewrites the input
the self-delete path relies on.

## Why it matters

- **RC-wqgm2p does not hold for a rebound device.** That is the cooperative
  self-delete control in `org-node/docs/risk/2026-09-09-org-node-hazards.md`.
  Step 3 is a cooperating device that has learnt of its removal and carries on.
  The precondition is an administrator enrolling a device key the member uses
  in another Organisation. An administrator who is also a member of that other
  Organisation can read the key from its trie.
- Step 2 is the inverse: a device deleting an Organisation it is still a
  member of. That is a denial of service against the member, with no error.
- The same selection also excludes the administrator's Member-as-a-group key, and
  removing that exclusion leaves the gate green. That clause is part of this
  defect's site, and it is left unrefined until the selection is corrected.

## What closing it looks like

Scope the selection to Personas bound to this Organisation, or not yet bound:
`p.org_id.is_none() || p.org_id == Some(org_id)`. Write that, and the
administrator exclusion, into LLR-e5c9ud, under TDD with the pin test
inverted. It belongs in a change of its own. This one is an architecture
ledger and must not change production behaviour under cover of documenting it.
Whether one Persona may be bound to more than one Organisation is a model
question as well. The glossary says a device may hold several Personas, while
LLR-rjg3m2 has one Persona founding two Organisations. That question should be
settled before the fix, not by it.

## Widened by review round 6 — a second writer

`create_organisation` also writes the binding. LLR-w3fhhg requires it to bind
the founding Persona to the Organisation it founds, and when that Persona is
already a member elsewhere, the old binding is overwritten. The self-delete
path reads the binding. So B, a member of org 1, founds org Y, and the next
ordinary org 1 update makes B delete org 1 while B is still in org 1's trie.
`pr_mdv38y_founding_an_organisation_rebinds_a_member_persona` pins it.

**The cure named above does not reach this writer.** Scoping
`receive_and_verify`'s selection leaves `create_organisation` rebinding. The
underlying fault is the data model: `PersonaRecord.org_id` holds one
Organisation, while two operations each have reason to bind a Persona to a
second one. The fix is to decide the model first: one Persona per
Organisation, enforced at both writers, or a binding per Organisation. Then
correct both writers to it. Folded into this report, not opened as another,
because it is the same binding and the same cure decision.

*Annotated 2026-10-05 by review round 8.* The second writer leaves the
Persona's member id from the Organisation it was bound to before
(LLR-q3aj8z), so after it the Persona names a member id that is not in the
Organisation it is bound to. `pr_mdv38y_founding_an_organisation_rebinds_a_member_persona`
asserts it.

*Annotated 2026-10-05 by review round 1 of `worktree-worktree-person-shared-types`
(finding-6); reworded the same day by docs/plans/2026-10-05-switch-trim.md.* The
items the `affects:` line names are amended in place on that branch rather
than superseded, so the line names them alone.

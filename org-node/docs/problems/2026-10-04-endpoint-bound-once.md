# Problem reports — org-node, one endpoint per service

Opened by the architecture tooth from review round 5.

*Moved 2026-10-13 (S4, task T10,
ruling A):* the report this file held — the transport endpoint bound once per
service, from whichever Persona first needed it, so a device administering two
Organisations through two Personas sent the second Organisation's admissions
under the first Persona's device key — moved to org-io with the endpoint, to
`org-io/docs/problems/2026-10-13-endpoint-bound-once.md`,
where it is resolved by one endpoint per Persona. org-node's transitional
`send_body` and `bind_endpoint` still bind once per service and are pinned
by `pr_8qsnhx_a_second_organisations_admission_goes_out_under_the_first_personas_key`
(`org-node/tests/admission_sender.rs`); org-io calls neither, and task T14
deletes them with the pin. *(Deleted 2026-10-14, S4 task T14: both methods
and the pin are gone.)*

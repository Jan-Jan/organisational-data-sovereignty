# Open problems found by S4's code conformance review

Opened 2026-10-14 by the fix of S4's CODE conformance review (against
`.guardrails/templates/CODE_GUIDELINES.md`). Fixing it would change
behaviour, which a conformance fix may not do, so it is recorded here and
left open.

**PR-u784zs**: failures that are not chain reads are reported as
`OrgNodeError::Chain` ("chain read failed: …"): the store key derivation,
the store encode and the Blob encode; and `build_invite` on an Organisation
no Persona is bound to is refused as `OrgNotOnChain` rather than as an
Organisation not held by a bound Persona.
affects: SDD-asxwp8, SDD-xshv5b, LLR-nr8df3
opened: 2026-10-14
status: open

Where: `org-node/src/store.rs`, `derive_key` and `cipher` (store key) and
`seal_data` (store encode); `org-node/src/invitation.rs`, `armour` (Blob
encode); `org-node/src/service.rs`, `build_invite`. Finding-4 of the review
(Errors 1). The caller reads a local encoding failure as a chain read that
failed, and an unbound Organisation as one missing from the chain.

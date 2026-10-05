# Problem reports — persona and org records print secret key material in Debug

**PR-hqwpg9**: `PersonaRecord` and `OrgRecord` (`org-node/src/store.rs`) derive
`Debug` while holding the member seed, device seed and Organisation secret as
plain `[u8; 32]`, so any `{:?}` of a record, of `StoreData`, or of a value that
contains one writes that secret key material in clear to wherever the output
goes (logs, panic messages, test output).
affects: REQ-hzm4kt, RC-jjsz97
opened: 2026-10-04
status: resolved
resolution: root cause — the secrets were held as plain `[u8; 32]` in types
that derive `Debug` (`PersonaRecord`, `OrgRecord`, `StoreData` and the
`WireMessage` that carries the Organisation secret), so the derived `Debug`
printed their bytes, and the store key derived from the passphrase was a plain
`[u8; 32]` too. Fixed by the org-node type-safety change
(`worktree-org-node-type-safety`, plan
`docs/plans/2026-10-04-org-node-type-safety.md`): the member seed, device seed
and Organisation secret are held in `MemberSeed`, `DeviceSeed` and
`OrgSecret`, and the store key in `StoreKey`, each with a redacted `Debug` and
no `Display`; a seed becomes a key pair only through
`MemberSeed`/`DeviceSeed::signing_keypair` (`SigningKeypair::from_seed` and
`to_seed` are deleted) (REQ-y7tsft, RC-8a4xjb). Reproduced and gated by
`records_and_wire_messages_never_render_secret_bytes` in
org-node/tests/secret_redaction.rs, which failed against the pre-change records
(the member seed's bytes in the `Debug` output) and passes after.

Found 2026-10-04 during the `grill-requirements` interview for the
validated-newtype rule. REQ-hzm4kt keeps these secrets out of the store file
only; no requirement yet covers their other exits, so the fix starts with a
requirement (`grill-requirements`) and an `analyze-risks` pass. The fix is
deferred to the org-node type-safety follow-up change, where the seeds and the
Organisation secret get a secret newtype whose `Debug` is redacted.

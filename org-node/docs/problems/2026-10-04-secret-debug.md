# Problem reports — persona and org records print secret key material in Debug

**PR-hqwpg9**: `PersonaRecord` and `OrgRecord` (`org-node/src/store.rs`) derive
`Debug` while holding the member seed, device seed and Organisation secret as
plain `[u8; 32]`, so any `{:?}` of a record, of `StoreData`, or of a value that
contains one writes that secret key material in clear to wherever the output
goes (logs, panic messages, test output).
affects: REQ-hzm4kt, RC-jjsz97
opened: 2026-10-04
status: open

Found 2026-10-04 during the `grill-requirements` interview for the
validated-newtype rule. REQ-hzm4kt keeps these secrets out of the store file
only; no requirement yet covers their other exits, so the fix starts with a
requirement (`grill-requirements`) and an `analyze-risks` pass. The fix is
deferred to the org-node type-safety follow-up change, where the seeds and the
Organisation secret get a secret newtype whose `Debug` is redacted.

# Rare-branch measurement — `membership_conformance`, 20 runs (T6, decision 12)

Supporting notes for branch worktree-quint-connect-coupling; the verification record is docs/verification/2026-10-03-worktree-quint-connect-coupling.md.

Measured 2026-10-03 on branch `worktree-quint-connect-coupling-t6` (on top of
T7's PR-vf5hdm fix), quint 0.33.0, rust backend, `MAX_SAMPLES = 100`,
`MAX_STEPS = 15`, random seed per run. Command, run 20 times:

    cargo test -p org-members --test mbt_conformance membership_conformance -- --nocapture

All 20 runs passed. Each row counts the `coverage <action> -> <outcome>: <n>`
lines the test prints: in how many of the 20 runs the pair occurred, and its
per-run minimum, maximum and 20-run total (steps, across all traces of a run).

| Action | Outcome | Runs present (of 20) | Min/run | Max/run | Total |
|---|---|---|---|---|---|
| AddDevice | DuplicateDevice | 20 | 5 | 16 | 208 |
| AddDevice | IdNotFound | 20 | 87 | 120 | 2055 |
| AddDevice | Ok | 20 | 24 | 58 | 792 |
| AddMember | ConfusableHandle | 20 | 9 | 21 | 292 |
| AddMember | DuplicateId | 20 | 33 | 67 | 976 |
| AddMember | IdNotFound | 20 | 45 | 85 | 1269 |
| AddMember | Ok | 20 | 13 | 40 | 564 |
| ApplyDelta | ConfusableHandle | 20 | 4 | 11 | 146 |
| ApplyDelta | DeltaBaseMismatch | 20 | 40 | 71 | 1087 |
| ApplyDelta | NoOpUpsert | 20 | 7 | 29 | 391 |
| ApplyDelta | Ok | 20 | 82 | 133 | 2129 |
| ApplyDelta | RemoveUpsertOverlap | 20 | 3 | 10 | 115 |
| ApplyDelta | StaleRemoval | 20 | 27 | 63 | 893 |
| ApplyDelta | VerificationFailed | 20 | 24 | 46 | 683 |
| DeleteDevice | DeviceNotFound | 20 | 7 | 20 | 257 |
| DeleteDevice | IdNotFound | 20 | 83 | 128 | 2128 |
| DeleteDevice | Ok | 20 | 22 | 51 | 696 |
| DeleteMember | IdNotFound | 20 | 92 | 117 | 2080 |
| DeleteMember | Ok | 20 | 36 | 57 | 960 |
| Init | ConfusableHandle | 20 | 4 | 13 | 152 |
| Init | DeviceSlotsFull | 20 | 4 | 13 | 164 |
| Init | DuplicateId | 20 | 3 | 14 | 157 |
| Init | EmptyDeviceList | 20 | 5 | 15 | 187 |
| Init | Ok | 20 | 57 | 73 | 1340 |
| Isolate | IdNotFound | 20 | 94 | 139 | 2172 |
| Isolate | Ok | 20 | 29 | 67 | 980 |
| RotateKey | IdNotFound | 20 | 81 | 134 | 2039 |
| RotateKey | Ok | 20 | 33 | 64 | 981 |
| UpdateHandle | ConfusableHandle | 20 | 11 | 26 | 353 |
| UpdateHandle | IdNotFound | 20 | 86 | 129 | 2163 |
| UpdateHandle | Ok | 20 | 22 | 44 | 635 |
| UpdateNameSurname | IdNotFound | 20 | 85 | 119 | 2013 |
| UpdateNameSurname | Ok | 20 | 33 | 65 | 943 |

**Threshold (plan T6 step 6):** a pair present in fewer than 10 of 20 runs gets
a named scenario. Every pair above is present in 20 of 20, so no scenario was
added for this measurement.

**Pairs the model can produce that never occurred (0 of 20):**

- `AddDevice -> DeviceSlotsFull` — needs three successful `AddDevice` steps on
  one member before a fourth; already carried by the named scenario
  `scenarioDeviceSlotsFull` (`scenario_device_slots_full`, T4).

Unreachable in the model, so absent by construction, not by sampling:
`AddMember -> EmptyDeviceList` / `AddMember -> DeviceSlotsFull` (an added
record is always `mkLeaf`, one device) and `ApplyDelta -> DeviceSlotsFull`
(upserts are `mkLeaf` records or current records, which `deviceCapOk` holds at
four or fewer).

**Stability (plan T6 step 7):** the whole `mbt_conformance` target, 10
consecutive runs: 10 × `24 passed; 0 failed; 1 ignored` (the ignored test is
`preflight_probe`, run only as a subprocess by
`quint_preflight_names_a_missing_binary`), 9.9–11.5 s each.

## Re-tally after base merge (2026-10-03)

The table above predates the merge of master into this change. The merged
membership model also produces `P2pKeyNotReplaced` from `DeleteDevice` and
`Isolate` (PR-zz4exm's unchanged-replacement-key refusal), so the rarity rule
of ADR decision 12 is re-applied here (independent review round 3, finding 3).

Measured 2026-10-03 on the merged code (branch
`worktree-quint-connect-coupling-review3fix`, off
`worktree-quint-connect-coupling` at 754d667), quint 0.33.0, rust backend,
`MAX_SAMPLES = 100`, `MAX_STEPS = 15`, random seed per run, same command run
20 times. All 20 runs passed (7.5–17.4 s each).

| Action | Outcome | Runs present (of 20) | Min/run | Max/run | Total |
|---|---|---|---|---|---|
| AddDevice | DuplicateDevice | 20 | 6 | 18 | 219 |
| AddDevice | IdNotFound | 20 | 78 | 127 | 2121 |
| AddDevice | Ok | 20 | 24 | 48 | 718 |
| AddMember | ConfusableHandle | 20 | 7 | 26 | 271 |
| AddMember | DuplicateId | 20 | 29 | 66 | 958 |
| AddMember | IdNotFound | 20 | 50 | 89 | 1317 |
| AddMember | Ok | 20 | 20 | 32 | 535 |
| ApplyDelta | ConfusableHandle | 20 | 2 | 7 | 107 |
| ApplyDelta | DeltaBaseMismatch | 20 | 42 | 68 | 1065 |
| ApplyDelta | NoOpUpsert | 20 | 9 | 33 | 388 |
| ApplyDelta | Ok | 20 | 77 | 127 | 2106 |
| ApplyDelta | RemoveUpsertOverlap | 20 | 2 | 15 | 121 |
| ApplyDelta | StaleRemoval | 20 | 32 | 64 | 817 |
| ApplyDelta | VerificationFailed | 20 | 25 | 45 | 670 |
| DeleteDevice | DeviceNotFound | 20 | 4 | 20 | 233 |
| DeleteDevice | IdNotFound | 20 | 87 | 126 | 2174 |
| DeleteDevice | Ok | 20 | 19 | 39 | 584 |
| DeleteDevice | P2pKeyNotReplaced | 20 | 4 | 16 | 184 |
| DeleteMember | IdNotFound | 20 | 88 | 123 | 2144 |
| DeleteMember | Ok | 20 | 31 | 55 | 904 |
| Init | ConfusableHandle | 20 | 3 | 13 | 156 |
| Init | DeviceSlotsFull | 20 | 7 | 14 | 192 |
| Init | DuplicateId | 20 | 4 | 16 | 165 |
| Init | EmptyDeviceList | 20 | 4 | 16 | 192 |
| Init | Ok | 20 | 58 | 71 | 1295 |
| Isolate | IdNotFound | 20 | 81 | 126 | 2089 |
| Isolate | Ok | 20 | 28 | 46 | 701 |
| Isolate | P2pKeyNotReplaced | 20 | 5 | 17 | 227 |
| RotateKey | IdNotFound | 20 | 91 | 128 | 2182 |
| RotateKey | Ok | 20 | 32 | 66 | 927 |
| UpdateHandle | ConfusableHandle | 20 | 12 | 24 | 372 |
| UpdateHandle | IdNotFound | 20 | 92 | 136 | 2187 |
| UpdateHandle | Ok | 20 | 22 | 44 | 620 |
| UpdateNameSurname | IdNotFound | 20 | 95 | 125 | 2133 |
| UpdateNameSurname | Ok | 20 | 33 | 58 | 926 |

Every pair, the two new `P2pKeyNotReplaced` rows included, is present in 20 of
20 runs, so the threshold (fewer than 10 of 20 gets a named scenario) adds no
scenario. The rarest pairs per run are `ApplyDelta -> ConfusableHandle` and
`ApplyDelta -> RemoveUpsertOverlap` (minimum 2 per run each). As before,
`AddDevice -> DeviceSlotsFull` occurred in 0 of 20 runs and stays carried by
`scenario_device_slots_full`.

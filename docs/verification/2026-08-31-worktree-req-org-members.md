# Verification — first requirements, organisation membership (2026-08-31)

> **Corrected 2026-09-02, body left unedited.** Finding 13 of this record
> states that there is no arbitrary-bytes fuzz target for the `org-members`
> deserialisation path. That is false: `org-node/tests/fuzz_envelope_decode`
> feeds arbitrary bytes to a `SignedDeltaEnvelope` decode and then calls
> `decode_delta()`, which deserialises `org_members::delta::Delta` through the
> validating `Deserialize` impls. The target predates that record. The claim
> was carried forward unchecked into the Class C ADR and into the first draft of
> the hazard analysis before an independent reviewer caught it; the corrected
> statement — the target exists, is outside `test_paths`, is unannotated, has an
> empty corpus and is run by no lane — is in
> `docs/risk/2026-09-02-membership-hazards.md`. The body below is left as
> written for the same reason as any other record.


branch: worktree-req-org-members
reviewer: fresh general-purpose subagent, dispatched at merge-change step 6a with only the diff, the item grammars, the project rules and the class B ADR — no implementation narrative and no session history
verdict: fit to merge with conditions, 32 findings. The reviewer mutation-tested every verifies: annotation rather than reading them, and found three that did not verify the requirement they named plus three false factual claims of mine. All conditions met; the findings and their dispositions are below.
reproduced: yes — independently, not on the reviewer's word. The three inert annotations were confirmed by reading the guards they depend on (`calculate_delta` has its own `is_calculated()` check at trie.rs:572, separate from `root_hash`'s `cached_root_hash.ok_or()` at trie.rs:108, so a mutation of root-reporting cannot reach that test). REQ-h5ret5's falsity was confirmed at types.rs:204 (`handle.nfc().collect()` — normalize, not reject) and in its own annotated test at integration_test.rs:761, which asserts acceptance. The duplicate test was confirmed by finding `deserialize_rejects_invalid_handle` at integration_test.rs:1209, predating this work. The unchecked key replacement was confirmed by reading `delete_p2p_device` (trie.rs:233-245): no comparison against the outgoing key exists.

Change: ratchet tooth 4 — the project's first requirements. Twelve REQ items
for the membership capability, each `satisfies: derived` with a hazard-impact
assessment, `verifies:` annotations across the org-members test suite, one new
test, one problem report, and the glossary terms this interview settled.
Branched from `master` at `4bb5509`.
Plan: `docs/plans/2026-08-26-ratchet-gap-analysis.md`, tooth 4.

## The gate

Re-run on the corrected tree (`796a91f`).

| Gate | Result |
| --- | --- |
| `cargo test -p org-members` | Suites this change touches, run by me: `integration_test` 102 passed, `fuzz_tests` 6 passed, 0 failed. The full command additionally runs `mbt_conformance`, which the sandbox cannot execute (quint cannot create `~/.quint`); the project owner ran the full command on the pre-review tree and reported all tests passing. |
| `cargo test … on-chain-client --lib + 3 fuzz targets` | Untouched by this change; last measured green at `4bb5509`. |
| `quint typecheck` ×2 | Untouched by this change. |
| `make coverage` | exit 0. Unchanged from the baseline: org-members 93.50% lines / 92.20% regions, on-chain-client 53.57% / 59.19%. |
| `check-ids.sh --allow-draft-files` | exit 0 |
| `check-trace.sh` | exit 0 — `checked: REQ 12, HAZ 0, RC 0, SDD 0, LLR 0, PR 1`; `problems: open 1, oldest 0 days; limits age 30, open 10`. One `UNRESOLVED-PR PR-zz4exm (open 0 days, owner Jan-Jan)` warning, which is the ledger working rather than a failure. |
| `clippy -p org-members --lib -D warnings` | exit 0 |
| `check-review.sh` | Could not run — exit 2 on macOS awk (the toolkit defect recorded in the tooth 1 record). Hand-checked as before. |
| Coverage, against the class target | org-members meets it. on-chain-client's accepted shortfall (tooth 3 record) is unchanged and untouched here. |
| Working tree | clean |

**Coverage did not move**, and that is the most interesting number in this
change rather than a null result. Adding a test and removing a test left the
figures byte-identical, because both touched code the suite already executed.
The instrument was checked before that was believed: a single test run in
isolation reports 13.44% total, so the measurement is live.

## What was wrong, and what was built

Nothing was wrong: the crate was well tested and its invariants were written
down in `org-members/AGENTS.md`. What did not exist was any statement of what
the software is *required* to do, independent of what it happens to do, and
therefore any way to ask whether a test verifies a requirement or merely
passes.

Writing that statement down found things reading the code had not:

- **A requirement that was false.** REQ-h5ret5 as first drafted said non-NFC
  handles are rejected. They are normalized and accepted. The requirement's own
  annotated test asserts the acceptance — so the first draft contradicted its
  evidence, which is exactly what an annotation is for.
- **A behaviour with no assertion behind it.** `root_hash()` erroring before
  recalculation. The function is called throughout the suite, but only ever on a
  calculated trie.
- **A security-relevant gap in the library** (PR-zz4exm): device removal stores
  the caller's replacement key without comparing it to the outgoing one, so a
  caller passing the current key back removes the device and leaves its access
  intact. Found by asking what REQ-ewdg2q's "so that a removed device cannot
  derive access" actually requires.
- **Three annotations that named a requirement they could not verify** — found
  by the reviewer, not by me. See findings 7, 8 and 9.

## Review

**finding-1**: REQ-h5ret5 was false as written: it put NFC normalization inside the rejection predicate, while the software normalizes and accepts, and its own annotated test asserts acceptance.
disposition: Rewritten to separate the two — the software normalizes to NFC, and rejects a handle that after normalization is empty, over 128 bytes, uppercase, contains `.`, or mixes scripts. Confirmed at types.rs:204 before changing. This was the most serious of the requirement defects: a requirement contradicted by its own evidence.

**finding-2**: REQ-shk82j over-claimed "applying the same rules as for a member admitted directly". The wire path accepts an empty device set where the constructor rejects one (with a pre-existing test asserting it must), and uniqueness/confusable checks happen at `apply_delta`, not at decode.
disposition: Rewritten to name the rules it does apply, with a paragraph stating both exclusions and why the equivalence is deliberately not claimed.

**finding-3**: REQ-ewdg2q stated a guarantee the software does not enforce: `delete_p2p_device` stores the caller's replacement key with no comparison to the outgoing one.
disposition: The missing comparison is now a clause of the requirement, followed by a statement that it is unmet, and the gap is PR-zz4exm. The alternative — softening the requirement to match the code — was rejected: the operation exists to cut off a removed device, and a requirement that accommodates a call defeating that would document the bug as intended behaviour.

**finding-4**: Five requirements bundle more than one behaviour (REQ-crjxk8, REQ-xdx2c2, REQ-r784fu, REQ-4umsuz, REQ-ds8ryr), against the ledger's "one requirement per item".
disposition: Not split, and recorded as an accepted quality gap. Each clause has a test, none is untestable because of the bundling, and splitting them would multiply twelve items into about twenty on the first pass — better done when the LLRs arrive at tooth 6 and each clause finds its own design item. Named here so it is a decision rather than an oversight.

**finding-5**: REQ-crjxk8's first clause was closer to design than observable behaviour: "32-byte" is interface design and "immutable" is unfalsifiable from outside, since no operation exists that could change an id.
disposition: Rewritten as the observable consequence — the identifier does not change when the handle changes or when keys are replaced. Both halves now have annotated tests (`update_handle_renames_member`, and `rotate_p2p_key_changes_only_key`, newly annotated).

**finding-6**: The note under REQ-xdx2c2 claimed the number 4 was recorded only in `MAX_DEVICES` and one test. It is also in `org-members/AGENTS.md` and three times in the 2026-05-07 design document.
disposition: Corrected, and the point sharpened rather than dropped: the number lives in four places, none of which any gate reads, and writing the LLR is what gives it a controlled home.

**finding-7**: `calculate_delta_fails_when_hashes_not_calculated` claimed `verifies: REQ-avmu3j` and could not verify it — `calculate_delta` has its own `is_calculated()` guard and never reports a root. Proven by mutation: with root-reporting behaviour deleted, this test still passed while the new one failed.
disposition: Annotation removed, and a comment left in its place explaining why it looks like coverage and is not, so it is not re-added. Verified independently by reading trie.rs:572 against trie.rs:108. This is the finding that mattered most: without the new test, REQ-avmu3j would have carried an annotation and zero verification, which is the failure mode that makes the whole apparatus decorative.

**finding-8**: `delete_p2p_device_unknown_device_fails` claimed `verifies: REQ-ewdg2q` and was inert for it — proven by mutation, it passes with the key replacement removed. As REQ-ewdg2q's only abnormal-case annotation, that left the requirement with no abnormal-input coverage of its actual behaviour.
disposition: Annotation removed with an explanatory comment. The abnormal case that would genuinely cover it is the degenerate same-key call, which is PR-zz4exm's reproducing test — so this gap closes when that problem report is resolved, and it is listed in Gaps until then.

**finding-9**: `insert_adds_member` claimed `verifies: REQ-crjxk8` while asserting nothing about the identifier.
disposition: Annotation removed with an explanatory comment.

**finding-10**: `update_handle_renames_member` claims REQ-kmvc96 but renames to a free handle, so the rejection REQ-kmvc96 states is not exercised there.
disposition: Kept as that requirement's normal case, which is legitimate under the class B normal-and-abnormal rule, with the rejection covered by `update_handle_rejects_collision` and the acceptance case now also annotated on `genesis_multiple_members`. The annotation is accurate for the half it covers.

**finding-11**: Two of REQ-h5ret5's clauses were verified only by unannotated tests — the 128-byte bound by `handle_too_long_rejected`, the hyphen permission by `handle_hyphen_allowed` — and the annotated proptest cannot reach the length branch at all, its strategy capping handles at 64 characters.
disposition: Both tests annotated, with the proptest's blind spot noted on the length test so the gap is not rediscovered.

**finding-12**: REQ-shk82j's device-key-set half had no annotated test; the four pre-existing wire tests for device canonicality carried no annotation, nor did the better handle test `deserialize_rejects_invalid_handle`.
disposition: All five annotated.

**finding-13**: REQ-ds8ryr's "for any input, including … exceeds a documented limit" outruns its annotated evidence: the two proptests cannot generate an over-limit handle and never touch the wire path (confirmed by the reviewer via per-line execution counts — zero executions of the deserialize validation call from a fuzz_tests-only run).
disposition: The requirement is kept as written, because it is the right requirement for a library others embed, and the wire-path abnormal tests are now annotated to it indirectly through REQ-shk82j. The genuine remaining gap — no arbitrary-bytes fuzz target for the deserialize path — is recorded in Gaps and belongs with the project's standing "always include fuzz testing" rule.

**finding-14**: REQ-m8aexh had three abnormal cases and no annotated normal case.
disposition: `genesis_multiple_members` annotated as the normal case for it and for REQ-kmvc96.

**finding-15**: `root_hash_errs_until_recalculated` is sound and its doc-comment accurate; it fails under the mutation in finding 7.
disposition: Nothing to change.

**finding-16**: `deserialize_revalidates_handle`'s construction is sound — the lowercase control does isolate the handle rule as the cause of rejection, confirmed by mutation.
disposition: Superseded by finding-17: the test is sound and redundant, so it was deleted rather than kept.

**finding-17**: Two of that test's doc-comment claims were false. `deserialize_rejects_invalid_handle` had asserted the same invariant since before this work, and `fuzz_tests.rs` contains no serde usage at all, so the delta round-trip proptests cannot have been executing the deserialize path.
disposition: My test deleted, its annotation moved to the pre-existing test, and a comment left where it stood recording that it was written by trusting a doc-comment over the file it describes. Both false claims are withdrawn in the commit message. The zero-coverage observation was correct; the explanation attached to it was invented, and the true explanation is duller — the path was already covered *and* asserted.

**finding-18**: The zero coverage delta is true, verified two ways by the reviewer.
disposition: Nothing to change. Retained in the record because the observation survives even though my explanation of it did not: coverage identical across a test added and a test removed is a fact about what coverage measures.

**finding-19**: The deleted test was more fragile than the one it duplicated and was the only postcard test in the file lacking a `serde` feature gate.
disposition: Moot — it is gone.

**finding-20**: The derived-requirement handling is mechanically correct: all twelve IDs appear in the risk file, check-trace and check-ids both exit 0.
disposition: Nothing to change.

**finding-21**: The assessments are real rather than filler, and the weak ones say so.
disposition: Nothing to change.

**finding-22**: The risk file's preamble claimed several assessments hedge with "of a hazard not yet enumerated". The phrase appeared exactly once in the repository — in the preamble describing itself. So the disclaimer credited the file with a caution it did not exercise.
disposition: Rewritten to say plainly how to read every "Hazard impact: mitigates" line: as mitigating a hazard not yet enumerated, none of them anchored to a HAZ item because none exists. This mattered more than its size: in a file whose entire warrant is honesty about what has not been done, a false self-description is the worst available defect.

**finding-23**: No forbidden vocabulary remains; the glossary agrees with `org-members/AGENTS.md` on every shared term.
disposition: Nothing to change.

**finding-24**: The glossary's `_Avoid_` lists forbid "delta", "root hash" and "trie", which `org-members/AGENTS.md` mandates as crate vocabulary. Signposted rather than contradictory today, since CONTEXT.md names AGENTS.md the authority for crate-level terms until tooth 9.
disposition: Left as signposted, and flagged for tooth 9, which must resolve the two-level vocabulary or the documents will read as conflicting. Recorded in Gaps.

**finding-25**: Three lines left unwrapped past the margin by the vocabulary substitution.
disposition: Not fixed. Cosmetic, and reflowing regulated text for margin alignment is churn in a diff that reviewers have to read.

**finding-26**: Commit-message claim that `root_hash()` erroring before recalculation had no test asserting it — TRUE.
disposition: Nothing to correct.

**finding-27**: Commit-message claim that wire-format handle re-validation had no test asserting it — FALSE. `deserialize_rejects_invalid_handle` predates this work.
disposition: Withdrawn explicitly in the follow-up commit message rather than quietly corrected, since the original claim stands in the history. See finding-17.

**finding-28**: Commit-message claim that the delta round-trip proptests were executing the deserialize path incidentally — FALSE. `fuzz_tests.rs` contains no serde usage at all.
disposition: Withdrawn explicitly in the follow-up commit message. The observation it was invented to explain (finding-18) was real; the explanation was not.

**finding-29**: Commit-message claim that adding the test moved measured coverage by zero — TRUE, verified two ways by the reviewer.
disposition: Nothing to correct.

**finding-30**: Commit-message claim that the "group key" vocabulary conflict was fixed — TRUE.
disposition: Nothing to correct.

**finding-31**: Commit-message claim that twelve REQ items are each marked derived and each assessed — TRUE.
disposition: Nothing to correct.

**finding-32**: The false "only place the number 4 is recorded" claim inside the change.
disposition: Same as finding-6.

## Gaps

- **REQ-ewdg2q's second clause is unmet** and is tracked as PR-zz4exm, open, owner
  Jan-Jan, opened 2026-08-31. Until it is resolved, that requirement has no
  abnormal-input coverage of its own behaviour — the test that would provide it
  is the problem report's reproducing test.
- **No arbitrary-bytes fuzz target for the deserialize path** (finding 13), so
  REQ-ds8ryr's claim over hostile wire input rests on hand-written cases. The
  project's own rule is to always include fuzz testing; this is where it is
  missing.
- **Five requirements bundle multiple behaviours** (finding 4), accepted for this
  pass and better split when tooth 6's LLRs give each clause a design item.
- **No LLRs, no SDD items, no HAZ items.** These twelve REQs are derived and
  assessed, but nothing yet connects them to a software architecture or to an
  enumerated hazard. Teeth 5 and 6.
- **The glossary is partial and two-level** (finding 24): product terms here,
  crate terms in `org-members/AGENTS.md`, with `_Avoid_` entries that point in
  opposite directions until tooth 9 reconciles them.
- **Step 6c did not run** (macOS awk defect). This record is hand-checked against
  the field grammar, as the previous two were.
- **Requirements exist for one crate of seven.** `org-node`, `app/src-tauri`,
  `on-chain-client` and the spike crates have no requirements and are outside
  `strict_paths`.

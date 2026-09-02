# Verification — risk analysis: Class C, acceptability matrix, hazard register (2026-09-02)

branch: worktree-risk-membership
reviewer: two fresh general-purpose subagents, dispatched independently at merge-change step 6a with only the diff (`4da12d3..HEAD`), the ledger files and the installed skills as acceptance criteria — no implementation narrative and no session history. Two rather than one because IEC 62304 Class C asks for a thorough review and for considering two independent reviewers on critical items, and the safety classification plus the overall residual-risk conclusion are the critical items in this change. The lenses were split deliberately: reviewer 1 judged the ISO 14971 reasoning and was told another reviewer was checking the facts; reviewer 2 verified every factual claim about the code, the models and the traceability and was told another reviewer was judging the reasoning. Neither saw the other's report.
verdict: **not fit to merge**, from both reviewers independently, on non-overlapping grounds — 11 findings on the argument and 13 on the facts, of which 3 were false claims about the codebase and 3 were risk controls asserting more than their requirement or the code supports. Several findings bundled more than one defect, so they are disposed below as 29 blocks: two of those blocks are defects both reviewers found independently, and one is a quibble reviewer 1 offered without raising it as a finding. Every finding was verified by the author before action, 28 blocks were acted on, and one — finding-23 — was accepted as a known condition of the merge. Conditions met; the corrections are in `a3ae9c4`. The reviewers were not re-run against the corrected documents, which is this record's principal limitation — see "Limitations".
reproduced: partly, and the distinction matters here because this change ships no code. For the three false factual claims, yes, and independently of the reviewers: the fuzz target's existence and reach were confirmed by reading `org-node/tests/fuzz_envelope_decode/fuzz_target.rs` and following `decode_delta` into the `org-members` `Deserialize` impls; `ADMINS`/`THRESHOLD` were confirmed unused by grepping `quint/protocol.qnt` (three hits: two declarations and a comment); `on-chain/POST_POC.md`'s multisig recommendation was read in place and is the upgrade authority, and `org-node/src/chain_write/mod.rs` states threshold-1 in its own module docs. RC-mqtks7's wire-path bypass was reproduced by reading `validate_canonical_delta` and confirming no check relates a leaf's device set to its member-as-a-group key. The argument findings are not the kind of thing that reproduces — they were checked by re-reading the documents against the standard and against each other, and finding 3 was confirmed as a flat contradiction between two files in this same change. Nothing was reproduced by watching a test fail, because no test in this project asserts anything about these documents.

Change: the ISO 14971 risk analysis for organisation membership, in three
commits — `891bbf1` reclassified the project from IEC 62304 Class B to Class C
and set the acceptability matrix; `dc22647` wrote the hazard register under it;
`a3ae9c4` corrected both against the reviews below. Branched from `master` at
`4da12d3`. Documentation only: no crate source, no test, no CI job logic
changed. Six configuration comments and one CI job comment were corrected
because they still asserted Class B.
Plan: `docs/plans/2026-08-26-ratchet-gap-analysis.md` tooth 5, and the
`analyze-risks` skill.

## The gate

| Gate | Result |
| --- | --- |
| `cargo test -p org-members` | 102 + 6 + 1 + 2 passed, 0 failed. `mbt_conformance` ran (4.10s) rather than being skipped. |
| `cargo test … on-chain-client --lib + 3 fuzz targets` | 23 lib tests passed; three bolero targets ran ~1s each and passed. Required `CARGO_HOME=/tmp/cargo_home_fuzz`; this machine's `~/.cargo` is read-only under the agent sandbox. |
| `quint typecheck quint/membership.qnt` | exit 0. |
| `quint typecheck quint/protocol.qnt` | exit 0. |
| `make coverage` | exit 0. org-members 93.50% lines / 92.20% regions against floors 92/91; on-chain-client 53.57% / 59.28% against floors 52/58. Both metrics are statement coverage. **Decision coverage, mandatory under Class C, is not measured** — see Gaps. |
| `check-trace.sh` | exit 0. `checked: REQ 12, HAZ 8, RC 10, SDD 0, LLR 0, PR 2`. Two `UNRESOLVED-PR` warnings, both within limits: PR-zz4exm (open 2 days) and PR-hvg2dy (open 0 days), against `problem_age_days: 30` and `problem_open_max: 10`. |
| `check-ids.sh --allow-draft-files` | exit 0, no output. Covers the two draft ledger files this change adds. |
| `git status` | clean at the point of merge. |

## Findings from the independent review

Reviewer 1 judged the argument; reviewer 2 verified the facts. Numbering is
this record's, sequential across both reports.

**finding-1**: The overall residual-risk conclusion was circular. Every S3 severity in the register is derived from the three deployment contexts that make the project Class C; the conclusion then excluded those contexts as the control that made overall residual risk acceptable. You cannot hold severities that exist only under a use while excluding that use.
disposition: The conclusion is withdrawn and replaced. Overall residual risk against the intended use is now stated as UNACCEPTABLE — the software has not reached the use that makes it Class C — and the restriction on use is recorded as a consequence of that result rather than as a control that discharges it. The class/evaluation distinction the reviewer proposed is stated explicitly: the class is set by the planned intended use, the evaluation is against the present state, and lifting the restriction re-opens the evaluation and not the class. No test reddens without this; the check that fails without it is reading the register's severity derivation and its conclusion in the same sitting, which is what the reviewer did.

**finding-2**: The withdrawn conclusion discharged nine unacceptable risks with information for safety, the one control category this change's own acceptability matrix says "does not discharge an unacceptable risk on its own". The restriction was also stated nowhere a user would encounter it — not the repository README, not crate documentation, not an operator note — so it was not an implemented control under ISO 14971 clause 7.2, only an intention.
disposition: Both halves recorded in the register's conclusion, which now says so in as many words. Publishing the restriction where a user meets it is a new unticked item on `docs/plans/2026-08-26-ratchet-setup.md`, flagged as the owner's decision. The register states that until then it is the only place the conclusion exists.

**finding-3**: `docs/adr/2026-09-01-safety-class-c.md` said the clinical context alone would have sustained Class B and was the least severe of the three deployments; the register in the same change assessed clinical unavailability at S3 and made that the sole severity argument for HAZ-8suua9. Both cannot hold.
disposition: The ADR's ranking sentence is withdrawn in place, with the reason: severity is a property of the harm, and an organisational fallback does not make an injury less severe. The Class C conclusion never rested on it. Kept the register's S3.

**finding-4**: The register promised that probability was the probability of the harm and "carries the external conditions the pathway needs", and staked the S3 unavailability severity on fallbacks being counted in probability rather than severity — then counted a fallback in no estimate. Every probability was a fault-exposure argument.
disposition: The method section is rewritten. Probability is now defined as the probability of the hazardous situation arising; the situation-to-harm step is stated as explicitly unquantified, with a note that this is where fallbacks and deployment modelling would act. The claim that fallbacks were counted is withdrawn: "they are counted nowhere, and the honest statement is that the register estimates occurrence".

**finding-5**: The clause 8 overall evaluation covered 7 of 12 hazards, excluding the five recorded in prose — including admin-account compromise at S3/P1, described as the point every other control depends on. And the verdict rested on no stated criterion, because the project has no risk management plan defining overall acceptability.
disposition: The residual-risk section now tabulates the eight minted hazards and states the four prose hazards' residuals immediately after, concluding over all twelve. The absence of overall-acceptability criteria is stated in the register and added to the ratchet checklist as an open item.

**finding-6**: The "limitation of the ledger" complaint was over-applied by one hazard. Isolation applied to the wrong member could have been minted: RC-sq3yhp mitigates it and REQ-r784fu carries four `verifies:` annotations, so the trace closes. That was the author's reasoning failing, not the toolkit's.
disposition: Minted as HAZ-sc7wse, with `mitigates: HAZ-s39gbh, HAZ-sc7wse` on RC-sq3yhp. The register now explains the self-referential structure — the control's removal half creates the hazard, its reversibility half bounds it — rather than leaving it looking like a trick. The limitation section is narrowed to four hazards and says explicitly that the fifth was the author's error. `check-trace.sh` reports `HAZ 8` and exits 0, which is the mechanical confirmation the reviewer predicted.

**finding-7**: `docs/verification/2026-09-01-worktree-guardrails-051.md` still carried the false "`--strict` has never passed here" claim with no pointer to its correction, while the superseded Class B ADR in the same change got an in-place forward pointer. Two superseded artefacts, two standards.
disposition: A dated correction pointer is prepended to that record, body unedited — the ADR's treatment. The same was done for `docs/verification/2026-08-31-worktree-req-org-members.md`, which originated finding-13 below.

**finding-8**: The amendment to `docs/risk/2026-08-31-membership-derived.md` deleted the text it superseded, so a reader cannot see what was said — and then referred to "the shorthand this note originally asked for" after deleting the shorthand.
disposition: The superseded paragraph is now quoted in full as a block quote before the amendment, the treatment the ratchet-setup correction already used.

**finding-9**: The same amendment said "no severity or probability is assigned" and three paragraphs later stated "disclosure S3, unavailability S3".
disposition: Narrowed to "no severity or probability is assigned in the assessments below", which is what was meant and is true.

**finding-10**: The same amendment said "**Two** of the assessments called their own shots" and then listed three. Found independently by both reviewers.
disposition: Corrected to three.

**finding-11**: The tooth-5 cell in `docs/plans/2026-08-26-ratchet-gap-analysis.md` read as done and to-do at once — the DONE note was prepended to the surviving instruction text, where tooth 2 in the same table uses strikethrough.
disposition: The superseded instruction is struck through. Added, because it is the more useful half: the cell now records that the expectation about the quint invariants was wrong in one instructive case, `tauWindow` being entailed by its own action guard.

**finding-12**: `docs/adr/2026-09-01-safety-class-c.md` called the unmeasured decision coverage "an accepted, documented gap", asserting an acceptance nobody gave; `verify-before-merge` step 5 requires the user's explicit acceptance, recorded in the verification record.
disposition: Reworded to "a documented gap awaiting the owner's explicit acceptance at a merge", and the corresponding config comment now says a shortfall under Class C is not a gap an agent may accept. It is listed under Gaps below, unaccepted.

**finding-13**: FALSE CLAIM. The register stated in five places, and the Class C ADR repeated, that there is no arbitrary-bytes fuzz target over the `org-members` deserialisation path. `org-node/tests/fuzz_envelope_decode` feeds arbitrary bytes to a `SignedDeltaEnvelope` decode and then calls `decode_delta()`, which deserialises `org_members::delta::Delta` through the validating `Deserialize` impls; `fuzz_verify_against_chain` reaches the same decoder. Both predate this change. The claim was inherited from tooth 4's verification record and carried forward unchecked into two documents, and it was load-bearing in two residual-risk verdicts and one named future control.
disposition: Corrected in all five places in the register and in the ADR, each with the correction stated rather than silently replaced. The register now says the target exists, sits outside `test_paths`, carries no `verifies:` annotation, has an empty seed corpus and is run by no lane — "evidence that exists and counts for nothing, which is a worse position to be in than a known absence, because a known absence is visible". Not-minted control 3 changed from "write a target" to "annotate and run the existing target". The originating record carries a correction pointer. Verified independently before acting: read the target, followed `decode_delta` to `MemberLeaf::deserialize`, confirmed the file predates this change.

**finding-14**: FALSE CLAIM. The register said `protocol.qnt` "models a 2-of-n admin threshold", used to soften the admin-compromise hazard. `ADMINS` and `THRESHOLD` appear three times in the model: two declarations and one comment. No expression, guard or invariant consumes either. Read literally the constants are 2-of-2, not 2-of-n.
disposition: Clause deleted. The register now says the model declares the constants and models no threshold logic, and that a below-threshold rogue admin is modelled only as an actor who cannot advance the chain. Verified by grep before acting: three hits, one of them a comment.

**finding-15**: UNSUPPORTED CLAIM. The register cited `on-chain/POST_POC.md` for a per-organisation admin multisig and called the contract's `msg.sender` authorisation "deliberately agnostic". That document recommends a multisig for the **upgrade** authority (3-of-5 maintainers plus a timelock) and says nothing about an organisation's admin account; no project document takes the "deliberately agnostic" position. What the repository implements is threshold-1 by design — `org-node/src/chain_write/mod.rs` says so and its threshold-≥2 dispatch is dead code that errors if reached.
disposition: Citation dropped, characterisation withdrawn as the author's charitable reading, and the threshold-1 implementation stated. The register now says an organisation running this with a single-key admin account has no protection from any control in the register and that no document here tells an operator otherwise; writing one is not-minted control 9. Verified by reading POST_POC.md in place and the write-path module docs.

**finding-16**: RC-4u22ba was broader than REQ-crjxk8 and its second clause is implemented nowhere: it asserted that "every grant is made against that identifier rather than against the name it is displayed under", where `org-members` has no grant concept at all and the register's own scope section places grants outside the analysis. This broke the register's guarantee that RC-mqtks7 was the only aspirational clause.
disposition: The grant clause is cut; RC-4u22ba is now worded from REQ-crjxk8's text. The cut clause becomes not-minted control 8, and the register records why it was cut where the control is defined.

**finding-17**: RC-9z65hw asserted the result is usable only if its root matches "a root obtained independently of the change set itself". `CandidateTrie::verify_against` compares against a caller-supplied parameter; `org-members` documents the independent trusted root as an obligation on its callers, and REQ-4umsuz is carefully worded to avoid the claim.
disposition: RC-9z65hw narrowed to REQ-4umsuz's wording. The independence of the root source becomes not-minted control 5, and the register states that the crate disclaims it in its own documentation.

**finding-18**: RC-mqtks7's atomicity ("cannot be performed by halves") holds only on the direct API. `apply_delta` accepts any well-formed change set, and `validate_canonical_delta` checks ordering, presence, observable change and disjointness only — nothing relates a leaf's device set to its member-as-a-group key — so a change set can remove a device and leave the key. That is the same hazard by another route, and the register elsewhere criticises exactly this shape of gap.
disposition: The control is scoped to the direct membership API, as REQ-shk82j scopes its own weakening, and the wire-path bypass is written into HAZ-s39gbh's residual risk as the second of two defects in the control. New not-minted control 2 covers the delta-validation rule that would close it. Reproduced by reading `validate_canonical_delta` in full.

**finding-19**: The register said RC-4apk6w is "verified by structured proptests over constructed records". The five tests annotated `verifies: REQ-shk82j` are example-based unit tests over hand-constructed postcard payloads; the crate's proptests carry annotations for other requirements.
disposition: Corrected to five example-based unit tests, with the observation that the earlier wording overstated its evidence in the very sentence where it was being self-critical. Verified by grepping the annotations and reading one of the tests.

**finding-20**: Two clauses about `org-node/src/verify.rs` were wrong. "Exercised, including by a fuzz target" — nothing runs those tests: `org-node` appears in no `verify_commands`, no `Makefile` target and no CI job. And "reads the root from the latest finalised block" — the only non-mock reader returns a cached snapshot refreshed on an awaited `refresh()`, and its own doc-comment calls the read "current best", contradicting the client's documented `at = None` finalised semantics.
disposition: Reworded to "tested in-tree and run by nothing", with the seven unit tests and two fuzz targets named as existing but unrun, and to describe the cached snapshot. The doc-comment contradiction is opened as **PR-hvg2dy** rather than noted in passing, because it sits on the one control the whole anchor depends on. The four clauses the reviewer verified correct — envelope-independent reader, strictly-newer epoch, exact root match, `SeqGuard` before decode — are kept as written.

**finding-21**: "Three libfuzzer targets on the chain decode path" overstated count and engine. The third target is explicitly structured, not arbitrary-bytes, and none runs under libFuzzer in any lane — `verify_commands` and CI invoke them via `cargo test`, i.e. bolero's default generative engine.
disposition: Corrected to five bolero targets — two arbitrary-bytes and one structured round-trip on the chain decode path, two on the org-node path — with the note that all run under bolero's default engine and that libFuzzer is a separate deep-fuzz invocation each target's header documents.

**finding-22**: "Proves `tauWindow`" was wrong twice. The check is bounded — CI runs the simulator at 5000 samples and Apalache to depth 5, and the models' README is explicit that this is not verification of an implementation — and the invariant is entailed by the action guard: the only action admitting a write requires the staleness window and then stamps the staleness it just required, so `tauWindow` cannot fail unless the guard is deleted. The register also rendered `revocationSafety`'s settling precondition with only one of its two conjuncts, dropping the CGKA-token-epoch condition.
disposition: Both corrected. The register now says the invariant is checked rather than proved, gives the bounds, and states that it documents the policy without independently establishing anything about it. The settling precondition is stated with both conjuncts. This is the same shape of defect as tooth 4's inert `verifies:` annotations, and the gap-analysis tooth-5 cell now records that.

**finding-23**: Five documents reference the register by its post-merge name `docs/risk/2026-09-02-membership-hazards.md`. `finalize-docs.sh` assigns that name from the merge date and rewrites no references, and no gate catches a dangling file path — `DANGLING-REF` is about item IDs. Correct only if the merge lands on 2026-09-02. Found independently by both reviewers.
disposition: **Accepted as a condition of the merge, not fixed.** The merge is being handed over the same day, and `finalize-docs.sh` was read to confirm the rename produces exactly that name from this branch. If the merge slips, the five references need one edit before it lands; they are listed in the reviewer's report and in this finding. The alternative — undated relative references — would cost the ledger its property that a reference names the file that defines the thing.

**finding-24**: This change sets `safety_class: C` and left six configuration statements asserting Class B: two comment blocks in `.guardrails/config.yaml`, two in the `Makefile`, and two in `.github/workflows/rust.yml`.
disposition: All six corrected. The config now states that statement AND decision coverage are the target and that decision coverage is unmeasured; the CI job is relabelled the statement-coverage gate with a note that decision coverage is measured nowhere; the review-backstop comment now notes that it matters more under Class C.

**finding-25**: HAZ-h58jn6 said the registry contract "accepts any root at the next epoch, including a prior one" without noting the `NoOpUpdate` guard, which rejects re-publishing the current pair.
disposition: The guard is now named in the hazard statement. It does not affect the hazard, which is republication of a *prior* root, and the register says so.

**finding-26**: The requirements file's enumeration of where the number 4 lives omitted a doc-comment in `org-members/src/trie.rs` that restates it as "`MAX_DEVICES` (4)".
disposition: Added to the enumeration. Verified by grep: `trie.rs:213`.

**finding-27**: The register attributed tooth 7 the widening of `strict_paths` and `test_paths`; the gap analysis's tooth 7 names `strict_paths` only.
disposition: Not-minted control 4 now says tooth 7 covers `strict_paths` and that `test_paths` and a CI job have to come with it.

**finding-28**: PR-zz4exm records that `emergency_isolate_member` carries the same defect as `delete_p2p_device`, while HAZ-s39gbh's residual attributed it to `delete_p2p_device` alone.
disposition: HAZ-s39gbh's probability rationale now names both operations and notes that PR-zz4exm covers both.

**finding-29**: Reviewer 1's quibble, acted on: the ADR cited "IEC 62304 §4.3 step 2", which is not a subdivision label in the standard.
disposition: Reworded to "§4.3's classification decision turns on whether the harm reachable after risk controls external to the software could be serious injury or death".

## What the reviewers verified clean

Worth recording, because it is what makes the findings above meaningful.
Reviewer 2 re-derived the traceability by hand rather than trusting the exit
code: 8 HAZ items all with a mitigating RC, 10 RC items all with an
implementing REQ, 12 REQ items all with at least one `verifies:` annotation.
Seven of the ten controls were confirmed to assert behaviour their requirement
states and the code has — RC-n2taat against `validate_handle`, `handle_skeleton`
and the skeleton index checks; RC-3ppkf6, RC-ty8qdw, RC-3qn5xg, RC-sq3yhp,
RC-4apk6w, and RC-c4truv's no-panic half against the crate-root denial of
`unwrap`/`expect`/`panic!`. The requirements diff was confirmed
annotation-only. Four of the six clauses about `verify_envelope_against_chain`
were confirmed correct in source. `msg.sender` authorisation, prior-root
acceptance and `MAX_DEVICES = 4` were confirmed. PR-zz4exm's description of
`delete_p2p_device` was confirmed exact.

Reviewer 1 found the Class C argument correct rather than merely acceptable,
including the diagnosis of the earlier ADR's structural error, and confirmed
against the standard that an intervening human decision acts on probability and
never on severity. It also checked the four rigour claims this change attributes
to Class C against the installed skills and found all four accurate, and
confirmed HAZ-m2xfrm's S1 was not chosen to clear a bar, S2/P1 being equally
acceptable under the matrix.

## Gaps, carried forward

- **Decision coverage is unmeasured and now mandatory.** `cargo llvm-cov
  --branch` is unstable and fails on stable rustc; the installed nightly lacks
  `llvm-tools-preview` and the sandbox cannot add it. This is a shortfall
  against a requirement rather than a target, and per finding-12 it is **not
  accepted** — it awaits the owner's explicit acceptance at this merge or the
  toolchain component that closes it.
- **Two open problem reports.** PR-zz4exm (2 days), the unenforced
  replacement-key check, which is the one item that would close an unacceptable
  residual risk outright. PR-hvg2dy (0 days), opened by this change.
- **The restriction on use is unpublished.** It exists only in the risk ledger.
  Until it is written where a user meets it, ISO 14971 clause 7.2 verification
  of it is not possible.
- **No overall-acceptability criteria.** Clause 8 wants them in a risk
  management plan; this project has none.
- **`org-node` is outside every gate.** Its verify path carries the decisive
  control on the anchor, has seven unit tests and two fuzz targets, and is run
  by no lane.
- **Nine on-chain-client integration targets remain outside the merge gate**,
  needing `npm install` in `on-chain/scripts`.
- **`check-review.sh` has no CI backstop**, because this project merges locally
  without pull requests. Step 6c covers it on the merging machine.

## Limitations of this record

The corrections in `a3ae9c4` were **not** re-reviewed. Both reviewers judged
the documents as they stood at `dc22647`, and 26 of their 29 findings were
acted on afterwards by the author — who is not independent of the corrections.
For a change that ships no code, the corrections are the change, so this is the
weakest link in the evidence and is recorded rather than left for a reader to
notice. What stands behind them instead: every factual finding was verified by
the author against the source before action, by the reading recorded under
`reproduced:` above; the mechanical gates were re-run after the corrections and
are quoted above; and the corrections are stated in the documents as
corrections, quoting or naming what they replace, so a later reader can check
each one against the claim it supersedes.

A second-round review is the obvious remedy and was not run. Under Class C's
"thorough review" that is a defensible omission only because the corrections
narrowed claims rather than widening them — every one of them removed an
assertion or scoped it to what a requirement states — and a correction that
weakens a claim cannot introduce the class of defect these reviews were looking
for. The exception is finding-6, which *added* an item (HAZ-sc7wse); that one
is backed by the gate, which reports `HAZ 8` at exit 0 and would report
`UNMITIGATED-HAZARD` if RC-sq3yhp did not mitigate it.

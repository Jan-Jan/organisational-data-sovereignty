# Verification — org-node architecture, SOUP and low-level requirements (2026-10-03)

branch: worktree-guardrails-org-node-arch
reviewer: independent subagent, ten rounds (each a fresh agent in its own nested worktree, given the diff, plan and ledger excerpts and no implementation narrative; each ran every unit in the impact set's verify_commands itself and wrote its own mutations).
verdict: ACCEPTED for merge 2026-10-05 — round 1 rejected for rework with sixteen findings, round 2 with fifteen, round 3 with ten, round 4 with eight, round 5 with eleven, round 6 with ten, round 7 with nine, round 8 with twenty, round 9 with four and round 10 with two, all one hundred and five disposed of below. From round 7, by the owner's ruling of 2026-10-04 recorded in round 6's section, a production defect outside this change's claims that is booked and pinned does not block convergence. By the owner's second ruling, of 2026-10-05 and recorded in round 8's section, round 9 is narrow: it checks round 8's dispositions and the record, and a behaviour it finds stated by no requirement is recorded as a gap, not a blocker. Round 10 raised no `code` or `requirement` finding, only two one-word `record` counts, fixed mechanically; by merge-change step 6a that is the last review round. The merge gate passed at `5bd639f` (tree `9cae4297…`), with the coverage gap accepted by the owner.
reproduced: not applicable — this change fixes no problem report. It is tooth 4 of the ratchet for the `org-node` unit: the unit's first architecture ledger, its first measured SOUP inventory, and the low-level requirements IEC 62304 class C asks for. What stands in place of a reproduction is the falsifiability sweep below: **no low-level requirement here was accepted because a test was already green.** Each was watched failing under a named mutation first, and the eight clauses that could not be made to fail were either given a test that makes them fail, restated to what the gate can see, or recorded as gaps.
Opens: PR-b9wab3, PR-mdv38y, PR-8qsnhx, PR-xwek5e, PR-322qst (all booked and pinned, none resolved here). Resolves: none. Pre-flight: guardrails 0.5.1 as installed has no `merge-preflight.sh`, so its component checks were run directly: `check-units.sh` (and `--impact`), `check-ids.sh` and `check-trace.sh` per unit in the impact set, base-merged (`origin/master` `24317e3` and local `master` `f688a3c` both ancestors of HEAD), no draft files, no nested worktrees, and `check-review.sh`.

Change: org-node's first architecture ledger — **nineteen software items and one hundred and eleven low-level requirements** — with a measured SOUP inventory, the standing Overview in the unit's architecture README, a risk assessment of the derived low-level requirements, and the test relocation that makes the evidence traceable. Branched from `master` at `f635acc`; `master` merged in **six** times while open: `5850ed7` (merge commit `4f4ec19`), `f5e16d2` (`cb6a357`), `0f85cb9` (`4ebe1c0`), `582a0c2` (`23749b8`), `24317e3` (`f1262e2`) and `f688a3c` (`ed5b66d`). *Counts in this paragraph and the two lines above re-measured 2026-10-04 by review round 5, which found them restated rather than re-counted: `grep -c '^\*\*LLR-'` gave 100, `git log --merges master..HEAD` gave five, and `check-review.sh` gave findings 60 once round 5 was recorded. Review round 6 added LLR-rc74nq (101) and ten findings (70), rounds 7 and 8 added two and eight requirements (103, 111) and nine and twenty findings (79, 99), and the sentence is now in the past tense so it cannot go stale in the present.* *This sentence previously named `2405ede`, `5850ed7` and `f5e16d2`. Review round 2 found that `2405ede` is not a merge parent of this branch at all — it is an ancestor of `5850ed7` — and that the `0f85cb9` merge this record discusses two paragraphs further down was missing from the list; the plan gave a third account, also wrong. The three above are read off `git log --merges`, and the plan now carries them as a table rather than a sentence.*

Plan: `docs/plans/2026-10-03-org-node-architecture.md`.

Units touched: org-node (`check-units.sh --impact master..HEAD`). Impact set: **org-node touched, app dependent** — both gates run below.

## Why this change moved code

Tooth 4 was expected to be documentation. It is not, and the reason belongs
before the evidence rather than after it.

`test_paths` in `org-node/.guardrails/config.yaml` is `org-node/tests`, as it is
in all four units, and `check-trace.sh` reads `verifies:` annotations from
nowhere else. So a low-level requirement whose only evidence is a
`#[cfg(test)]` module under `src` **cannot be traced** — and twenty-two of this
unit's fifty-three gated tests were in exactly that position, covering the
identifiers, the key custody, the envelope, the blobs, the calldata, the
multisig derivation, the endpoint identity and the service lifecycle.

Three courses were available. Widening `test_paths` to include `src` would have
let a `verifies:` reference in *production* code satisfy MISSING-TEST,
weakening the gate for this unit to make a ledger look complete. Dropping the
untraceable requirements would have left seven of the nineteen items with no
evidence at all. The third is what this change did: **relocate those tests into
`tests/`**, the move this unit already made on 2026-09-09 for the verify, store
and frame cases.

So the diff carries `Cargo.toml` (six new test targets, two re-declared
dev-dependencies), `verify_commands` (the same six), one crate-private function
reachable through a new `org_node::test_support` seam, one new read accessor
(`OrgService::list_pending_invites`), and the deletion of the eight
`#[cfg(test)]` modules whose contents moved. **No production behaviour
changes.**

## The gate

Gate round 10, HEAD `ed5b66d`, tree
`59ebcdbe9b8a417497b9051e0f63643dac65d551`, clean worktree before and after,
quint 0.33.0. Every figure read from command output, not from exit codes;
per-target counts are read beside their `Running` line, never in sequence.
`master` is at `f688a3c` and merged (`ed5b66d`), and the impact set was
measured at this same tree.

*Gate round 9 measured tree `47d39f89…` at `3b68744`. Since then review round
9's remedies added one pin and a short-stream case (122 tests), and `master`
moved to `f688a3c`, which adds the `person` unit and touches none of this
change's paths. A first run of this gate at `6123b3e`, before that merge, had
`check-units.sh --impact` refuse: the branch was behind `master` on
`person/`, so it read as an unclaimed changed path. That is the lesson round 3
recorded, met again: merging `master` removed it.*

*Gate round 8 measured tree `4fb630b1…`, and review round 8 reproduced it at
`996a400`. Since then review round 8's remedies have added five tests (two in
`admission_sender`, one each in `service_lifecycle`, `verify_against_chain`
and `transport_handshake`) and eight requirements, and changed no production
code.*

*Round 7 measured tree `c95788e…`. Since then review round 7's remedies have
added three tests (a pin and two LLR carriers), root assertions in two more,
and LLR-mbjfq8 and LLR-379hnv.*

*Rounds 1 to 4 measured trees `3d88695`, `7e43115f…`, `d17f535…` and
`9f28a44…`; all are superseded. Since round 4, review round 4's remedies
added three tests and corrected the fuzz fixture. The `24317e3` merge brought
org-members' parse-at-the-edge newtypes, which broke two branch-added call
sites (fixed in `a760577`), and PR-hqwpg9.*

*org-members is not in the impact set and its gate was **not** run in round 5.
Round 4's table carried its figures from round 3's habit of running it anyway.
They described tree `9f28a44…`, and this table now carries no figure it did
not measure at the tree named above.*

**The impact set is `org-node` touched, `app` dependent** — two units, from
`check-units.sh --impact master..HEAD` run at the tree these figures describe.

*Corrected 2026-10-04 by review round 3, which found the previous round's gate
section naming three units at a tree where the tool prints two, along with
`tracked paths 462` where it prints 466 and `records 22` where it prints 23.
The cause is worth stating because it is subtle and will recur: the impact set
was measured **before** the `582a0c2` merge and the suites **after**, and both
were written up as one round. `git diff` compares trees, so a branch *behind*
`master` on a path reads as having touched it — which is why org-members was in
the set — and **merging `master` is what removed it**. The lesson is the same
one this record keeps relearning in other forms: a figure belongs to a tree,
and a round that moves the tree must re-read every figure, not the ones that
look likely to have changed. Running org-members' gate anyway was harmless and
it stayed green; printing a tool's output that the tool does not produce is
not.*

| Gate | Result |
| --- | --- |
| org-node `cargo test -p org-node --features app,test-support …` | **122 passed, 0 failed** — admission_sender 39, verify_against_chain 19, envelope_binding 8, chain_write_pure 8, service_lifecycle 8, store_at_rest 7, key_custody 6, value_types 6, transport_handshake 6, blob_exchange 5, wire_frame_bound 5, service_stories 3, transport_networked 1, lib 1 |
| org-node bolero targets | `fuzz_envelope_decode`, `fuzz_verify_against_chain`, `fuzz_first_admission_base` ran clean; `harness = false`, so none prints a `test result:` line and none contributes to the count above. **Two of the three were found this round to have been running a guard that never fired** — see the fuzz-target section in round 2's block; both are now proved live by mutation, and `fuzz_verify_against_chain`'s iteration rate fell from 201 440/s to 2 474/s as a result. At this tree: 171 333/s, 887/s and 2 909/s respectively *(order corrected by review round 9, which found gate round 9's last two transcribed against each other's targets)* (one-second budgets; the rates move with machine load), and `fuzz_verify_against_chain` now accepts inputs (review round 4, finding-2) |
| org-node `quint --version` | 0.33.0 (recorded, not pinned) |
| org-node `quint typecheck` protocol / ods_instances | ok, ok |
| org-node `quint run` ×5 (forkSafety, revocationSafety, revokedExcludedFromOrgSecret, tauWindow, convergence) | no violation, each `--max-steps=16 --max-samples=5000` |
| app `cargo test --manifest-path app/src-tauri/Cargo.toml …` | 78 passed, 0 failed — connection_status 10, ipc 9, org_id_parsing 14, receiver_events 27, receiver_guard 6, startup_policy 12 *(corrected by review round 5: three of these six were transcribed against the wrong target)* |
| app `npm --prefix app run check` | 194 files, **0 errors**, 1 warning (pre-existing: tsconfig `@types/node`) |
| app `npm --prefix app run test` | 30 passed, 4 files |
| `check-units.sh` | exit 0 — units 5 (the `person` unit arrived with `f688a3c`), **disclaimed 6**, tracked paths 514 |
| `check-trace.sh` org-node | exit 0 — **REQ 20**, HAZ 6, RC 9, **SDD 19, LLR 111**, **PR 13**; problems open **10**, at the limit of 10, oldest 26 days. It also prints two advisory UNMET-EXPECTATION lines (REQ-ysyu9g, REQ-q92yac, 29 days), which this table omitted until review round 2 noted the omission |
| `check-trace.sh` app | exit 0 — REQ 22, HAZ 6, RC 12, SDD 0, LLR 0, PR 6; problems open 5, oldest 21 days; one advisory UNMET-EXPECTATION (REQ-x3c8n2, 21 days) |
| `check-ids.sh` org-node / app | exit 0 / exit 0 |
| `check-review.sh` | exit 0 — records 25, findings 103, all disposed; provenance checked (run without `--branch`) |

`app/node_modules` was absent from this worktree and installed with
`npm --prefix app ci` before its two npm gates; it is gitignored, so the tree
above is unaffected.

## The falsifiability sweep of 2026-10-03

**The method.** Each low-level requirement's text is decomposed into
independently falsifiable claims. For each claim: write the smallest source
mutation that makes exactly that claim false, apply it, run this unit's whole
`verify_commands` cargo line with `--no-fail-fast` so all collateral is visible,
then restore from a pristine copy and prove the file byte-identical by SHA-256.

- **RED** — a named test fails.
- **GREEN** — nothing fails. This is a verdict **about the search, not about the
  clause**: a green falsifier proves the probe was poor until the obvious
  probes have been tried. No clause was cut on a single green.
- **LIBBROKEN** — the library itself stops compiling. That measures nothing,
  because no test runs, so the mutation is rewritten rather than recorded.

**The harness refuses a mutation that did not apply.** A pattern matching
nothing would leave the file untouched, the suite green, and the clause recorded
as unfalsifiable. The driver compares the file's digest before and after the
edit and reports `NOCHANGE` instead of scoring it. It also re-verifies the
digest after restoring, and aborts the whole run if a restore ever failed; none
did.

**Depth, and that it is not uniform.** The owner chose on 2026-10-03 to sweep
per clause on the seven items where ordering and refusal carry the security
property — SDD-na9nc3, SDD-8cpyfa, SDD-8uyg4s, SDD-af5vnt, SDD-kk2y3e,
SDD-d8ktxa, SDD-72ddm6 — and one mutation per low-level requirement elsewhere,
where each requirement is a single claim by construction. That is **less than
org-members and on-chain-client received**, both swept per clause throughout.
The asymmetry is recorded here rather than left to be inferred from the table.

### Outcomes

**123 runs over 110 distinct mutations.**

| Outcome | Runs |
|---|---|
| **RED** — a named test ran and failed | 103 |
| **GREEN** — nothing failed | 15 |
| **LIBBROKEN** — the library stopped compiling; rewritten and re-run | 5 |

**Four of the five LIBBROKEN mutations were rewritten and re-run to a RED
verdict. The fifth was not, and that was not noticed until review round 3.**
Rows 34 (`8n95rf-root-checked`), 47 (`wx77j5-reports-the-id`), 94 (`hg3xzf`)
and 110 (`8n95rf-root-decisive`) each have a later RED row — row 34's rewrite
appears under the new label `8n95rf-root-decisive`, which is why it reads as
having none. **Row 77, `z8fubr`, has no re-run anywhere in the 168 rows**, so
LLR-z8fubr had no attestation of any kind while this sentence claimed
otherwise. Round 3 established why no rewrite was possible, and that the
requirement was wrong rather than merely unprobed: see finding-2 of round 3
below. *This sentence previously read "Every one of the five"; it was a
summary nobody had counted, in the section whose whole subject is counting.*

**This section used to claim "no clause is left resting on a green", and that
was false.** Review round 1 wrote three mutations the sweep had not, and all
three were green: deleting `self.store.save(rng)?` from `admit_member` and from
`create_organisation`, and replacing the Networked arm of `admit_member` with
the Loopback body. The claim was true of the clauses the sweep *chose to
probe*, and a sweep is only ever evidence about its own search — which is the
rule this record states two paragraphs above and then overstated. The accurate
claim is narrower: **no clause probed here is left resting on a green**, and
round 1's three are resolved in the round-1 section below.

*Review round 2 then wrote twenty more and found eight further greens, four of
them the remaining `store.save` sites round 1's fix had not swept. Nineteen
clauses in this change were found resting on nothing; **eleven of the nineteen
were found by someone other than the author.** The sweep's own rows below are
unchanged and still accurate about what the sweep measured — what has changed
twice now is how much of the unit the sweep's search covered, and both
corrections went the same way.*

The fifteen green runs of the sweep proper resolve as:

| Green run | Resolution |
|---|---|
| `vf5mjx-epoch-carries` | `rejects_stale_epoch` compared epoch 1 against committed epoch 1 — **symmetric**, so exchanging `got` and `last` was invisible. New test with distinct numbers; re-probe **RED**. |
| `8mfjey-fresh-nonce` | Nothing asserted the store's nonce ever changed. New test `each_save_draws_a_fresh_nonce`; re-probe **RED**. |
| `j83kc8-leaves-record-unchanged` | Clearing the pending invite on a rejected first admission was unobserved. New assertion via `list_pending_invites()`; re-probe **RED**. |
| `cja9zv-store-is-saved` | Deleting `self.store.save(rng)?` was unobserved — nothing reloaded the store after a receive. New test reads the record back from disk; re-probe **RED**. |
| `cja9zv-seq-written` ×2 | The first probe hit only the *update* branch, which no test read; the second showed the *first-admission* branch was a different line. Assertions added for both; re-probes **RED** on each branch. |
| `q8emds-invite-consumed` | Never consuming the pending invite was unobserved. Covered by the same new test; re-probe **RED**. |
| `8qxwst` | `a_string_that_is_not_base64_is_refused` asserted only `is_err()`, which a `unwrap_or_default()` substitution still satisfies — the refusal came from postcard instead. Test now asserts the verdict names base64; re-probe **RED**. |
| `7gnrnz` | A poor probe: an unused `impl` block changes no behaviour. Replaced by one that makes the twenty bytes lossy; **RED**. |
| `4fbuy8-org-is-first` | The clause "before its signature is examined" is unfalsifiable *and* more than REQ-gju89b asks. **LLR restated** to "before the Change set it carries is decoded"; new probe **RED**. |
| `d6kvbx-advance-before-root` | "Only after the root match" is unobservable: `SeqGuard` is `Copy` and a rejection returns no guard at all, so no caller mark can move. **LLR restated** to what is observable; new probe **RED**. |
| `wwunf4-ipv6-bind-present` | A poor probe. With `clear_ip_transports()` in place, dropping the `[::1]` bind leaves the endpoint loopback-only on IPv4 — the clause stays true. The clause is RED under both address probes. |
| `6adc99-ipv6-optional` | **Genuinely unverifiable.** REQ-2wzfzv's clauses are about bind failures no gated test can produce. Kept by owner ruling; see Gaps. |
| `u6rq4s-checked-against-new-trie` | **Gap, recorded.** Checking the sender against the local record instead reddens nothing; the two tries differ only for a member the same change removes, and removals arrive through the other receive operation. *Closed 2026-10-04 by review round 5. The second half of that reason is false: a removal of somebody else arrives through `receive_and_verify`. `a_removal_relayed_by_the_member_it_removes_is_refused` now reddens under this mutation.* |
| `u6rq4s-skipped-on-first-admission` | **Gap, recorded.** Forcing the check to run on first admission too reddens nothing, because the administrator's device is in the trie it just admitted the node into — so the skip is *unnecessary*, not merely unverified. Owner ruling booked. |

### What the sweep found, in one line each

1. **A test comparing a value against itself.** `rejects_stale_epoch` used
   equal numbers, so half of what it claimed to pin was unpinned.
2. **Nonce freshness was unevidenced** — and this unit's own SOUP inventory
   records that XChaCha20-Poly1305 does not survive nonce reuse, under a key
   that is fixed across saves of one store. One deleted line away, with nothing
   to catch it.
3. **The receive path's committing half was barely evidenced.** Deleting the
   store write, zeroing the sequence mark on either branch, and never consuming
   the pending invite were all green. The checking half — every refusal, every
   ordering, every error variant — was already solid. The asymmetry was
   invisible until it was measured.
4. **Two low-level requirements claimed more than the gate can see**, and both
   were restated rather than kept as decoration.
5. **A production comment credited a measurement nobody had made.**
   `transport/endpoint.rs` said the `[::1]` bind was load-bearing, "measured by
   a mutation that drops the `[::1]` bind and watches `[::]` reappear". That
   mutation is green. What `f635acc`'s record actually measured was dropping
   `clear_ip_transports()` **and** that bind together. The comment is corrected
   in place, and now states the real finding: the two are redundant mechanisms,
   either sufficient alone.

Items 1, 2, 3 and 5 were found by mutation and by nothing else. Reading the
code against the requirements found none of them.

*Qualified 2026-10-04, after review round 2. This paragraph used to end "—
which is what the review rounds do", and that was wrong about the rounds: both
of them wrote mutations as well as reading, and between them they found eleven
further unfalsifiable clauses, every one by mutation. What distinguishes the
rounds from the sweep is not technique but **where the mutations were pointed**
— at the lines the author had no reason to suspect, which is exactly the set
the author cannot enumerate.*


## Review round 1 — 2026-10-04

A fresh subagent in its own nested worktree, given the diff, the plan, the
ledgers and the record, and no implementation narrative. It ran both units'
`verify_commands` itself and reported the counts it saw; they matched. It then
wrote its own mutations.

**Verdict: reject for rework. Sixteen findings, none disputed.** Six `code`,
two `requirement`, eight `record`.

**finding-1**: code — deleting `self.store.save(rng)?` from `admit_member` and,
separately, from `create_organisation` leaves the whole gate green, so the
persistence clauses of LLR-t4znbk and LLR-68yd3j rest on nothing — the twin, on
the administrator's write path, of the `cja9zv-store-is-saved` green the sweep
found and fixed on the receive path.
disposition: fixed. `an_admission_reaches_the_administrators_disk` and a disk
reload in `creating_an_organisation_advances_the_chain_and_activates_the_persona`;
both mutations re-probed **RED**. Accepted without reservation — the sweep had
already learned this exact lesson on one side of the seam and did not think to
look at the other.

**finding-2**: code — LLR-t4znbk's ordering clause ("only after the message has
been sent") is unfalsifiable; moving the record update above the send is green.
disposition: fixed. `a_failed_push_leaves_the_administrators_record_where_it_was`
dials a peer identity with no transport addresses, so the send fails
immediately; re-probe **RED**. (A loopback port with no listener does not work:
QUIC retries it for thirty seconds.)

**finding-3**: code — LLR-jn5jeh's Networked clause is credited to tests that
cannot observe it; no gated test puts an `OrgService` into
`TransportMode::Networked`, and replacing the arm with the Loopback body is
green.
disposition: requirement narrowed, gap recorded. The LLR now states only the
Loopback clause. The Networked arm cannot be brought into this gate —
`bind_with_mode(Networked)` uses `presets::N0`, so a gated test of it would
reach the public internet — and what would close it is a `test-support`
constructor taking an injected relay. Recorded in the decomposition under "Two
behaviours that are deliberately not refined" and booked.

**finding-4**: code — `the_two_comparing_rejections_carry_both_numbers…`
destructured literals it had just constructed and asserted they equalled
themselves, and checked the rendering symmetrically.
disposition: fixed. Rewritten as
`a_stale_sequence_rejection_carries_the_offered_number_and_the_mark`, which
takes the error from `SeqGuard::check` and pins the **order** of the two
numbers. Accepted without reservation: a test written to demonstrate that
assertions must observe what they claim could not observe what it claimed.

**finding-5**: code — `org_id_debug_is_…` used `[0xab; 20]`, so reversing the
iteration order in `OrgId`'s `Debug` was invisible.
disposition: fixed. The fixture bytes are now all different and the first and
last are pinned to the ends.

**finding-6**: code — the relocation silently dropped two assertions:
`decode_delta_round_trips`'s re-encoding check (encoding canonicality) and
`tampering_with_delta_bytes_breaks_signature`'s non-empty guard.
disposition: fixed, both restored into `envelope_binding.rs`. The reviewer
compared all twenty-two relocated tests against `git show master:<path>` and
found no other loss.

**finding-7**: requirement — fifteen LLRs cite a requirement that does not state
them (thirteen REQ-nhe2zu, two REQ-hzm4kt), and the real statement — that
org-node has no high-level requirement for its administrator-side write path —
is nowhere made.
disposition: fixed. All fifteen are now `satisfies: derived` and assessed in
`org-node/docs/risk/2026-10-03-architecture-derived.md` under three themes. The
SRS gap is stated there in terms and booked for a `grill-requirements` pass of
its own; it is not closed here, because writing high-level requirements is a
requirements change with its own review. The reviewer's supporting observation
is exact: this unit's own SOUP inventory records "—" for the requirements
`parity-scale-codec` and `blake2` support, for the very code whose LLRs claimed
REQ-nhe2zu.

**finding-8**: requirement — the robustness section accounted for two uncovered
items and passed over four in silence.
disposition: fixed. All six are now tabulated. Two of them (SDD-msb6xh,
SDD-rq6nv4) are argued rather than owed — a function total over its argument
types has no abnormal case. Four are owed and are in the Gaps below, SDD-rx2yvy
most materially: `admit_member` takes a malformed key or a duplicate handle
from a join request a stranger composes, and nothing tests either.

**finding-9**: record — the deviation claimed the chopsticks targets hold
eighteen tests; the tree holds seven.
disposition: fixed. The line counts were right; the test counts came from a
`grep` that counted helper functions. Corrected in place with the correction
stated, because a deviation of this size should not be argued from a figure
nobody measured.

**finding-10**: record — the per-LLR attestations both ledgers point at existed
in no document: the record had only aggregates, and the plan's table still read
"(pending)".
disposition: fixed. The appendix below is the attestation table, 129 rows. The
plan now points at it rather than holding a second copy.

**finding-11**: record — the risk file said "eighty-two … eighty of them",
a figure left behind when a requirement was cut.
disposition: fixed, and corrected twice: the split changed again once
finding-7 moved fifteen items to derived.

**finding-12**: record — `disclaimed 7` (it is 6) and `PR 6` (it is 7), the
latter because master `a547ff3` added PR-g7cfns.
disposition: fixed. The figures are re-measured at the tree this round names.
PR-g7cfns was resolved by `0f85cb9`, merged in after the finding, so the open
count is 4 again — stated in Gap 9 with the IDs rather than only a number.

**finding-13**: record — five stale measurements in the decomposition, three of
them contradicted by this change's own diff.
disposition: fixed, all re-measured: lib.rs 78 (not 51 — this change added the
`test_support` module to it), test_fixtures.rs 76 (not 64), service.rs 1572
(not 1619), `verify_envelope_against_chain` 42 lines (not 80), the unit 3770
lines over 24 files (not 4004).

**finding-14**: record — the Evidence section said "eight harnessed targets"
(there are fourteen) and allowed evidence to sit in a `#[cfg(test)]` module
under `src`, contradicting the README and the record and the entire reason this
change moved twenty-two tests.
disposition: fixed. Both corrected, and the one surviving `src` test is named
as carrying no annotation and no requirement.

**finding-15**: record — the SOUP inventory cited `multi_account_id_is_order_independent`,
a test name this same change deleted when it relocated it.
disposition: fixed.

**finding-16**: record — the config comment recorded 80 passed; the tree yields
89.
disposition: fixed, re-measured, with a note that the figure was not refreshed
when the sweep's own remedies added tests.

### What round 1 says about the sweep

The sweep ran 123 mutations and found eight clauses resting on nothing. Round 1
wrote three more mutations and found three further clauses resting on nothing —
**two of them the exact twin, on the other side of the same seam, of a gap the
sweep had already found and closed.** A sweep is evidence about its own search
and nothing more, and this round is the measurement of how much the search
missed: roughly one in four of the greens in this change were found by someone
other than its author.

## Review round 2 — 2026-10-04

A second fresh subagent in its own nested worktree, given the same inputs as
round 1 plus round 1's findings and dispositions, and told not to re-derive
round 1 but to answer two questions: *are round 1's dispositions actually true
of this tree*, and *what did both the author's sweep and round 1 miss* — with
the hint that round 1's own closing observation was that two of its three
misses were the mirror image, on the other side of a seam, of gaps the sweep
had already found and closed.

It re-ran both units' `verify_commands` and reported figures matching the
record's throughout. It re-ran the mutations round 1's dispositions claim
redden, and all of them reddened on the test named. It then wrote twenty
mutations of its own: **twelve RED, eight GREEN**.

**Verdict: reject for rework. Fifteen findings, none disputed.** Six `code`,
two `requirement`, seven `record`.

Every finding below was re-measured here before being accepted. The eight
greens were reproduced; the stale figures were re-counted; the merge history
was read off `git log --merges`.

**finding-1**: code — four more of the unit's eight `self.store.save` sites
rest on nothing: `revoke_member`, **both** branches of
`receive_and_self_delete_if_revoked`, and `import_invite` can each lose their
save with the whole gate green. Round 1's remedy closed the two sites round 1
named instead of sweeping the other five, which is the identical mistake one
level up from the one round 1 convicted.
disposition: fixed, and the **class** swept rather than the sites. All eight
are now read back from disk by a named test;
`a_revocation_reaches_the_administrators_disk`,
`a_self_delete_reaches_the_revoked_nodes_disk`,
`an_update_that_does_not_revoke_us_reaches_the_disk` and
`an_imported_invite_reaches_the_joiners_disk` are new, and the eighth,
`create_persona`, was probed and was already RED on
`a_new_persona_is_proposed_with_an_id_derived_from_its_member_key`. All four
mutations re-probed **RED**. The reviewer is right about the material one: on
the evidence this gate produced, a revoked node that restarted recovered the
Organisation record, the member snapshots and the Organisation secret.

**finding-2**: code — LLR-ecz9a6 rests on nothing: hard-coding
`TransportMode::Loopback` in `ensure_endpoint` leaves the gate green, because
`OrgService::new` already sets Loopback and the only annotated test sets it
again.
disposition: requirement narrowed, gap recorded — the same disposition round 1
gave LLR-jn5jeh, for the same reason. The requirement now states only the
Loopback clause, which `a_service_in_loopback_mode_binds_loopback_sockets_only`
does observe. The mode-following clause cannot be brought into this gate: the
only other value makes `bind_with_mode` use `presets::N0` and reach the public
internet. Gap 14; closed by the injected-relay constructor already booked for
Gap 7.

**finding-3**: code — LLR-cns6q6's load-bearing word, the **named** persona's
device seed, rests on nothing: every gated test gave its service exactly one
persona, so binding "the first persona in the store" is green.
disposition: fixed.
`the_endpoint_binds_the_named_personas_device_not_the_first_personas` creates
two personas and binds for the second; re-probe **RED**.

**finding-4**: code — no `ChainReader` reachable at this gate can return `Err`,
so LLR-8m99q2's "distinctly from a chain read that failed" and LLR-rm9x4z's "a
failure is the `Err` arm" rest on nothing, and SDD-pa6p7w is missing from the
robustness tabulation.
disposition: fixed, and the item is now **covered** rather than tabulated.
`FailingChain` in `tests/verify_against_chain.rs` and
`a_chain_read_that_fails_is_refused_as_chain_not_as_absence` pin the failure to
`Chain` and assert it is a different answer from absence; re-probe **RED**.

**finding-5**: code — LLR-ghja3x's "one greater than the record's last" rests
on nothing: `+ 1` → `+ 2` in `admit_member` is green, because both sides take
the mark from the same envelope and `SeqGuard` only requires strict increase.
disposition: fixed.
`the_admission_envelope_carries_the_mark_one_past_the_records_last` captures
the message A transmits and compares it against the mark A held *before* the
call; re-probe **RED**. It also pins the clause the reviewer did not mention —
that the envelope verifies under the administrator's Member-as-a-group key and not under
the device key the connection authenticates.

**finding-6**: code — LLR-6zjzn2 and LLR-jn5jeh are credited to tests that
cannot observe them. The bind-once test compares a device key that is the same
whether one endpoint was bound or two; the two LLR-jn5jeh tests never call
`admit_member` at all, and one of them is the by-id case round 1 narrowed the
requirement to exclude.
disposition: fixed, and the reviewer is right on both counts — this is the same
defect shape as round 1's findings 4 and 5, which is what the sweep says
reading alone does not find. The bind-once test now compares the **bound socket
addresses** across the two calls, which differ if a second endpoint was bound;
removing the guard reddens it. LLR-jn5jeh was removed from both transport
annotations and is carried by
`in_loopback_mode_the_joiner_is_dialled_at_the_full_address`, which calls
`admit_member` and sees the admission arrive at the endpoint the address it
carried names; the negative is the mutation `r2-jn5jeh-loopback-by-id`, for the
reason set out under "One the author caught" below — at this gate "it does not
arrive" cannot be asserted at all.
*The requirement was also reworded: it said "the full address carried in the
join request", and `admit_member` dials its `peer_addr` argument. The caller is
what reads that from the join request.*

**finding-7**: requirement — `revoke_member` is named in SDD-72ddm6's interface
and refined by no low-level requirement at all; the same holds for
`export_invite`, `import_invite`, `export_join_request` and
`import_join_request` under SDD-rx2yvy. The ledger declares exactly one item
with no LLRs and argues it at length; these are **undeclared partial items
inside items presented as fully refined**.
disposition: fixed. Ten low-level requirements written — LLR-6dc598,
LLR-tax3pm and LLR-qg9utu for the revocation path; LLR-zj88e6, LLR-9zfnmb,
LLR-437fvx and LLR-836z24 for the four out-of-band functions; LLR-ag9mgm,
LLR-rjg3m2 and LLR-j6j95z for REQ-d9g6nt (finding-11). Each has a named test
and, with one stated exception, a red by mutation. Six are `satisfies: derived`
and are assessed in `org-node/docs/risk/2026-10-03-architecture-derived.md`
under two new themes. **Finding this gap a second time, on the revocation half
of the write path a day after finding it on the admission half, is the
strongest evidence available that the SRS gap round 1 booked is real.**
`a_failed_revocation_push_leaves_the_administrators_record_where_it_was` is
written as the deliberate twin of round 1's admission-side fix.

**finding-8**: requirement — the robustness tabulation omits SDD-pa6p7w and
SDD-z85ux9, and SDD-rq6nv4's argument is weaker than stated:
`runtime_call_to_tx` has three untested `WriteError::MalformedCall` arms, and
*this change* made `build_dispatch_tx` publicly reachable through
`org_node::test_support`, so the abnormal input is one line of test away.
disposition: fixed, all three. SDD-pa6p7w is no longer on the list at all —
finding-4's fix covers it. SDD-z85ux9 is added, because an item with no gated
test has no abnormal case either and saying so is the honest bookkeeping.
SDD-rq6nv4 is moved from argued to owed, with the reason stated: the old
wording was true of this unit's call graph and not of the function, and it
stopped being a safe reading the moment this change exported the function.
Gap 15.

**finding-9**: record — the 129-row attestation appendix, added by round 1
precisely to back "each LLR is discharged by red by mutation", has **no row for
six low-level requirements** (LLR-rb8r65, LLR-ghja3x, LLR-bg3vsw, LLR-6qmq2g,
LLR-hs7g6j, LLR-65py3d), and row 92 is labelled `8m99q2-threshold` while the
test it reddened is annotated LLR-8m3bwj — so LLR-8m3bwj's only attestation is
filed under another item's ID and SDD-na9nc3 is credited with a row it did not
earn.
disposition: fixed. Row 92 is relabelled `8m3bwj-threshold`. Of the six, four
are probed and recorded in the appendix's round-2 block below (LLR-rb8r65,
LLR-bg3vsw, LLR-6qmq2g, LLR-hs7g6j); LLR-ghja3x is covered by finding-5's fix;
and **LLR-65py3d is not probed and will not be**, because `Box<dyn ChainOps>`
compiling at all is the property it states and a mutation that falsified it
would be a signature change rather than a measurement. That is now said in the
appendix and in the decomposition's Evidence section rather than left for a
reader to infer from a missing row — which is the shape of this finding.
LLR-f5kq88 remains unattested and remains Gap 4's business.

**finding-10**: record — the standing Overview in
`org-node/docs/architecture/README.md` still says "roughly 815 of **4004**
source lines". Round 1's finding-13 re-measured the unit at 3770 over 24 files
and corrected the dated decomposition but not the README.
disposition: fixed; re-measured here as 3770 over 24 files. This is the exact
failure `2026-10-03-transport-binding-scope.md` warns about in terms —
correcting a claim means finding every place it was made — and round 1 made it
while fixing a different instance of it.

**finding-11**: record — the decomposition opens saying it refines "the
nineteen high-level requirements". The unit has **twenty**, and the one omitted
is **REQ-d9g6nt**, which arrived with master `0f85cb9` *during* this change, is
verified directly by `fuzz_first_admission_base` and four `admission_sender`
tests, and is refined by no LLR and traced by no item. `check-trace.sh` cannot
see it — there is no UNTRACED-REQUIREMENT rule, only UNTRACED-DESIGN — so the
ledger was the only place it could be caught.
disposition: fixed. REQ-d9g6nt is now traced by SDD-89es4z, SDD-rx2yvy and
SDD-8cpyfa and refined by LLR-rjg3m2, LLR-ag9mgm and LLR-j6j95z, each annotated
on the tests that already verified it directly. The opening sentence says
twenty and names the two that are traced by nothing by design.

**finding-12**: record — the SOUP inventory's dev-dependency table still says
bolero drives "the **two** fuzz targets" and names two of three.
disposition: fixed, with the reason recorded in the file: this was the one
document in the change nobody re-measured after the `0f85cb9` merge. The
`postcard` row said "Two bolero fuzz targets" as well and is corrected too.

**finding-13**: record — three documents give three different accounts of the
merge history, and the plan's and the record's are both wrong: `2405ede` is not
a merge parent at all, and the `0f85cb9` merge is missing from the record's
list.
disposition: fixed in both. The plan now carries a table of the three merge
commits taken from `git log --merges master..HEAD`, with what each brought in.
The lesson is stated there: a merge history belongs in a table generated from
the repository, not in a sentence written from memory a week later.

**finding-14**: record — two SOUP rows name a requirement the component has
nothing to do with. `base64` (the armour around out-of-band blobs) and
`thiserror` (the whole rejection vocabulary) were both credited to REQ-eg5j8u,
which is solely about the 1 MiB wire-frame bound.
disposition: fixed; `base64` now names REQ-9g6as6, `thiserror` the three
requirements its vocabulary serves. The reviewer's framing is the finding that
matters: this is the same class of mis-citation round 1 found in the
architecture ledger, surviving in the inventory round 1 had cited as
*supporting evidence* for that finding.

**finding-15**: record — Gap 12 says the four open problem reports are "open at
25–26 days"; the gate reports 25, 25, 25 and 24.
disposition: fixed, and fixed structurally rather than by re-counting. Round 1
caught this entry going stale once; round 1's fix wrote a day range and it went
stale again in the opposite direction within a day. **A day count in a document
measures a tree that no longer exists; a fire date does not move.** The entry
now names the IDs and their fire dates and gives no day count at all.

### What round 2 says about round 1, and about the sweep

Round 1's dispositions were all true of this tree: the reviewer re-ran each
mutation and each reddened on the test named. What was not true was that round
1 had *finished*. Three of its six code fixes closed the instance and left the
class:

| Round 1 closed | Round 2 found still open |
|---|---|
| `save` in `admit_member` and `create_organisation` | `save` in `revoke_member`, both self-delete arms, `import_invite` |
| LLR-jn5jeh credited to tests that cannot observe it | LLR-6zjzn2 and LLR-cns6q6 credited to tests that cannot observe them |
| fifteen LLRs citing a requirement that does not state them | **twelve of the same fifteen** still annotated on tests as `verifies: REQ-nhe2zu` |

The third row is this round's own finding, not the reviewer's: round 1 moved
the fifteen to `satisfies: derived` in the ledger and left the identical claim
standing in twelve test annotations, where `check-trace.sh` reads it —
`service_lifecycle.rs` ×3, `chain_write_pure.rs` ×7 and `admission_sender.rs`
×2, on the multisig derivation, the calldata, the seam and the administrator's
write path, none of which REQ-nhe2zu is about. (The other three of the fifteen
were never miscited at the test level; their test names REQ-hzm4kt.) The twelve
annotations are removed here. REQ-nhe2zu keeps a resolving trace through the
four tests that do verify it: `happy_path_commits_when_root_matches_chain`,
`rejects_root_mismatch_when_chain_root_differs`,
`a_committed_admission_reaches_the_disk_and_consumes_the_invite` and
`five_stories_full_e2e`.

### One the author caught, while fixing round 2's — and what it cost to check

`in_loopback_mode_the_joiner_is_dialled_at_the_full_address` was written here
to close finding-6. Its first draft dialled the same peer twice, by full
address and then by identity alone, and asserted the second call returned
`is_err()`.

**It would have returned `Err`, but not for the reason claimed.** Re-offering
the same join request is refused by org-members' key-uniqueness rule
(`DuplicateKey`) *before the dial is ever attempted*, so the assertion would
have passed with the dial behaviour deleted — a test written to close a
finding about tests that cannot observe what they claim, which could not
observe what it claimed. That is the third instance in this change; round 1's
findings 4 and 5 were the other two, and all three are mine. **`is_err()` is
almost never an observation**: a rejection reached through the wrong guard is
still a rejection, and a test that only asks whether something failed cannot
say which guard fired.

The second draft fixed that — a different joiner, and the error matched by name
— **and then failed, which is the part worth recording.** Dialling a *live*
identity whose `EndpointAddr` carries no transports does not fail: iroh retries
it past 30 seconds, the suite's whole network budget. So the negative cannot be
asserted in a gated test at all. This is the same property PR-d4nye8's record
names from the other side — a dead loopback port does not refuse, it retries —
and the lesson generalises: **at this gate, "it does not arrive" is not
observable; only "it arrives" is.** Every negative about delivery here is
therefore carried by a mutation, not by an assertion. The shipped test observes
the positive and names the mutation that falsifies it
(`r2-jn5jeh-loopback-by-id`, row 149).

Three drafts to get one clause honestly evidenced is the real cost of this
discipline, and it is recorded rather than smoothed over, because the first
draft would have passed review on a reading.

**The measurement, as it stood after round 2.** The author's sweep ran 123
mutations and found 8 clauses resting on nothing. Round 1 wrote 3 and found 3.
Round 2 wrote 20 and found 8. Writing round 2's missing attestation rows then
found two more. Of those 21 unfalsifiable clauses, **11 were found by someone
other than the author** — and every one of the 11 was adjacent to something the
author had already found and fixed. That is not a sweep that was too small. It
is a sweep that stopped at the example instead of the rule, twice, and was
caught both times by a reader who had not written it. The last two were caught
by the bookkeeping round 2 insisted on, which is the section that follows.

*Round 3 then found seven more, and the appendix's own count table is the
current figure: **28 clauses, 18 of them found by someone other than the
author.** The paragraph above is left as round 2 wrote it, because a record
that silently restates its own measurements is the thing three rounds of this
review have been convicting.*

### A second one the author caught: two fuzz targets that never reached what they were named for

Round 2's finding-9 listed LLR-hs7g6j among the six low-level requirements
with no attestation row. Writing that row found something considerably worse
than a missing row, and it is the one defect in this change that neither the
sweep, nor round 1, nor round 2 had found.

**The probe.** LLR-hs7g6j says no sequence of bytes offered as an envelope to
`verify_envelope_against_chain` causes a panic or an abort. Its only evidence
is the bolero target `fuzz_verify_against_chain`. The mutation written to
discharge it — `local_trie.apply_delta(&delta)?` → `.unwrap()`, a panic on any
delta that fails to apply — came back **GREEN**.

**The measurement that explained it.** A `panic!()` was placed as the *first
statement* of `verify_envelope_against_chain` and the target run alone:

```
test fuzz_verify_against_chain ... run time: 1.000008s | iterations/s: 201440
                                   | rng inputs: 201442 | exit reason: max duration
```

**It passed.** Two hundred thousand iterations, an unconditional panic at the
function's entry, and the target reported success — because it never called
the function. Not once.

**Why.** The target guarded its call with
`if let Ok(env) = postcard::from_bytes::<SignedDeltaEnvelope>(bytes)`. A
`SignedDeltaEnvelope` needs about eighty-six bytes at minimum — a twenty-byte
Organisation id, a varint sequence number, a length-prefixed delta and a
sixty-four-byte signature — and bolero's default `&[u8]` generator produces
slices far shorter than that. The guard never passed. The target had been
exercising postcard's willingness to reject a short slice, and nothing else.

**The same guard appears in `fuzz_envelope_decode`**, whose doc comment said it
"also drives `decode_delta` on any successfully-parsed envelope". Panicking
inside `decode_delta` and running that target alone: it passes too. That half
had never run either. The target's *other* half — `postcard::from_bytes` on
arbitrary bytes, which is LLR-v2y6sw's actual subject — runs on every input and
was always sound, so the target was half vacuous rather than wholly so.

**`fuzz_first_admission_base` is sound.** It calls `first_admission_base(Some(bytes))`
unguarded, on every input. The difference is the guard, not the harness.

**The fix is a second input shape in each target, not a longer budget.** No
budget reaches a region the generator cannot produce. Each target now also uses
the fuzz bytes as the **delta** inside an otherwise well-formed envelope: for
`fuzz_verify_against_chain` that means the expected Organisation, an acceptable
sequence number and a genuine signature over those exact bytes, so every input
passes checks 1–3 and reaches the delta decode; for `fuzz_envelope_decode` no
signature is needed, because `decode_delta` reads only `delta_bytes`.

**Measured after the fix**, each target run alone:

| Mutation | Before | After |
|---|---|---|
| `panic!()` at `verify_envelope_against_chain`'s entry | passes | **fails** |
| `envelope.decode_delta()?` → `.unwrap()` | passes | **fails** |
| `panic!()` inside `decode_delta` (`fuzz_envelope_decode`) | passes | **fails** |

The iteration rate is the other half of the evidence:
`fuzz_verify_against_chain` ran at **201 440/s** when it was doing nothing and
runs at **2 474/s** now that it signs and verifies on every input — an
eighty-fold drop that is the work it was supposed to have been doing all along.

**What is still not reached, stated rather than papered over.**
`apply_delta`'s own error arm needs a delta whose `base_root` equals the local
trie's root — thirty-two bytes of exact match, which no generator will produce
and which check 5 screens for in any case. Re-probed after the fix: still
GREEN. That arm is defensive code no input at this gate can reach, and it is
Gap 17 rather than a clause claimed as covered. What LLR-hs7g6j now rests on is
checks 1 through 4, which the target can and does falsify.

**Why this one matters most.** A bolero target prints an iteration count and no
`test result:` line, so it contributes nothing to the pass count and looks
identical whether it is exercising a whole function or a single `if let` that
never fires. **Nothing in this gate — not the pass count, not `check-trace.sh`,
not either review round — could have distinguished the two.** The only thing
that did was a mutation deep inside the function coming back green, which is
exactly the measurement round 1's appendix was created to make routine, and
which the six missing rows round 2 found had been hiding. Three rounds of
review were needed to get from "the appendix is missing six rows" to "one of
the three fuzz targets has never run the code it is named after".

## Review round 3 — 2026-10-04

A third fresh subagent in its own nested worktree, given the same inputs as
rounds 1 and 2 plus both rounds' findings and dispositions, and asked two
questions: *are round 2's dispositions true of this tree*, and *what did the
sweep, round 1 and round 2 all miss* — with the standing hint that the author's
fixes keep closing the instance and leaving the class, and the round-2
fuzz-target finding's corollary: **is each piece of evidence actually reaching
the code it is credited with?**

It re-ran nine of round 2's mutations on the full cargo line: eight **RED on
the test the record names**, and `r2-ecz9a6-mode-constant` **GREEN**, which is
the one this change ships deliberately. It re-ran both fuzz-deepening claims
and both hold. **Round 2's dispositions are true of this tree.**

**Verdict: reject for rework. Ten findings, none disputed.** Three `code`, two
`requirement`, five `record`. Every one was re-measured here before being
accepted.

**finding-1**: code — four record lookups can ignore the identifier they are
given and return whatever record comes first in the store, with the whole gate
green: `find_org`, `find_org_mut`, `admin_persona_for_org` and
`update_persona_status`. Every gated test until now gave its service one
Organisation and one Persona.
disposition: fixed, and the **class** closed rather than the four sites.
`a_second_organisation_is_admitted_into_without_touching_the_first` builds two
Personas and two Organisations and admits a joiner into the second, asserting
org_1's record, its on-chain root and its sequence mark are where they were —
in memory and on disk. All four mutations re-probed **RED**. Two low-level
requirements written, LLR-vdyu65 and LLR-w3fhhg, because nothing in the ledger
said these functions act on the record the caller named.

**This is round 2's finding-3 one level over, and the reviewer is exactly
right about the shape.** That finding convicted `ensure_endpoint` for resting
on "every gated test gives its service exactly one persona"; the fix wrote a
two-persona test *for that call site* and left the lookups it was an instance
of. The failure this allowed is not small: an administrator holding two
Organisations calls `admit_member(org_2, …)`, the joiner is minted into org_1's
record, org_1's root is submitted on chain for the epoch, and the envelope is
signed by org_1's administrator — all green.

**finding-2**: requirement — LLR-z8fubr's load-bearing clause is false of the
tree, its only gated test is an assertion that cannot fail, and its one
appendix row is a LIBBROKEN that was never re-run although this record claimed
all five were.
disposition: fixed, all three parts, and the reviewer is right on each.
Measured here: `org-node/src` holds **fifty `OrgNodeError::Chain(String)`
construction sites** behind one variant whose `Display` is
`"chain read failed: {0}"`, so a wrong store passphrase, a malformed invite
blob, a missing Persona and an iroh send failure all render as chain read
failures. `error.rs`'s own doc comment has always carried the correct narrower
statement and this ledger generalised it without checking. LLR-z8fubr and
SDD-swtd3w's item text are narrowed to verify-against-chain; the `thiserror`
SOUP row is corrected; the catch-all is recorded as Gap 19 rather than closed,
because collapsing fifty sites into typed variants is a change to the
production error surface with its own review. The record's "Every one of the
five LIBBROKEN mutations was rewritten and re-run" is corrected to four, with
the reason row 34 reads as having no re-run (its rewrite is filed under a new
label) and why row 77 genuinely has none.

**finding-3**: code — `fuzz_first_admission_base` never decodes a single
`MemberSnapshot`: every input that gets past `postcard::from_bytes::<Vec<MemberSnapshot>>`
decodes to the **empty** vector, so `VerifyingKey::from_bytes`, `MemberLeaf::new`
and the `"bad member key"` / `"bad device key"` arms are reached by nothing.
The record called this target "sound" on the strength of its *call* being
unguarded, without probing its *reach*.
disposition: fixed, and the reviewer's framing is the lesson. Re-measured
here: a `panic!()` in `trie_from_snapshots`'s per-snapshot closure left the
target passing after **136 796 iterations**. The target now carries a second
shape that puts the fuzz bytes where the KEY MATERIAL goes inside a
well-formed snapshot vector, so every iteration reaches
`VerifyingKey::from_bytes` on a Member-as-a-group key and a device key; the same probe is
now **RED**, and so is a probe placed inside the `"bad member key"` arm itself
— which is one of the abnormal-input cases SDD-rx2yvy's robustness entry lists
as untested. Round 2 called this target sound because its *call* was
unguarded, having just convicted two others for their *reach*: the same
mistake one target over, in the same paragraph that diagnosed it.

**finding-4**: record — Gap 17 and `fuzz_verify_against_chain`'s doc comment
overstate how deep the deepened target reaches: no input produces a decodable
delta, so checks 5 through 8 are unreached, not merely `apply_delta`'s error
arm, and the `assert_eq!` inside each `if let Ok(out)` can never run.
disposition: fixed, and **fixed in the target rather than only in the prose**,
which is the better remedy the finding pointed at without asking for. Confirmed
here: a `panic!()` immediately after the decode left the target passing after
2 479 iterations. The target now carries a third shape — one byte of an
**honest** delta encoding overwritten at a fuzz-chosen offset with a
fuzz-chosen value, which is the only way past a thirty-two-byte base-root match
no generator produces. A `panic!()` at check 6 and again at check 8 now both
redden it. Gap 17 is rewritten to say what is actually unreached — one
defensive arm — rather than to summarise a region four checks wide, and the
target's doc comment with it. *Round 2 wrote "a decodable-but-inapplicable
delta reaches the base-root check and `apply_delta`" as a claim about the shape
it had just added, and did not probe it. A fix for an unprobed claim that is
itself an unprobed claim is the failure this change keeps repeating.*

**finding-5**: record — the round-3 gate section names a tree at which three of
its figures are not what the tools print (`--impact` gives two units, not
three; tracked paths 466, not 462; `check-review` records 23, not 22), and four
measured line counts went stale in the same merge.
disposition: fixed, all re-measured at the tree named. The reviewer's
diagnosis is exact and uncomfortable: the impact set was measured **before**
the `582a0c2` merge and the gates **after**, and the record presented both as
one round. Merging master is what *removed* org-members from the set — a branch
behind master on a path reads as having touched it, and after the merge it is
not behind — so commit `72e75b4`'s subject, "three units, measured after the
master merge", contradicts itself. Running org-members' gate anyway was
harmless; printing a figure the tool does not produce is not. Line counts now:
unit **3768** over 24 files, `service.rs` **1571**, `test_fixtures.rs` **75**,
`chain_genesis_e2e.rs` **507**.

**finding-6**: record — Gap 6 still carries the pre-round-2 account of the
robustness tabulation ("Four items", SDD-rq6nv4 "argued") and contradicts Gap
15 and the ledger two files over.
disposition: fixed. Round 2's fix went into the ledger and into Gap 15 and not
into this standing list two hundred lines above it. Gap 6 now names seven items
and points at the ledger's table as authoritative rather than restating it, so
there is one place to correct next time instead of three.

**finding-7**: record — the appendix's count table mis-states round 1's block
and therefore the totals, in the one table whose subject is how many clauses
were found resting on nothing.
disposition: fixed by counting the rows: round 1's block is 3 RED and 3 GREEN,
not 6 RED, and the totals are **132 / 31 / 5**, not 135 / 28 / 5. The prose two
paragraphs below had it right all along.

**finding-8**: record — `soup.md` still says the inventory was measured with
`master` merged in at `2405ede`, the exact commit round 2 established is not a
merge parent of this branch.
disposition: fixed. Round 2's finding-13 said "fixed in both" — the plan and
this record — and the SOUP file was the third document making the same claim,
in the file round 2's finding-12 had *just* convicted as the one nobody
re-measured after a merge. Two of three corrected is the instance, not the
class, for the third round running.

**finding-9**: requirement — `revoke_member`'s Loopback dial is named in
SDD-72ddm6's interface and refined by no low-level requirement, although its
admission twin has LLR-jn5jeh and round 2 wrote the other three twins for
exactly this reason.
disposition: fixed. LLR-pw369n written, covering the dial and the refusal
`admit_member` has no counterpart for — a Loopback revocation offered no
address cannot fall back to discovery and is refused before anything is sent.
The behaviour was evidenced and unstated rather than unevidenced: the reviewer
measured the Loopback arm dialling by identity instead and it reddens six
tests. Round 2 wrote three twins of a four-clause interface and stopped. *(The words "refused before anything is sent" above are wrong: the refusal
comes after the chain write. That is PR-b9wab3, corrected in round 4's
finding-4 and, for the test comment that repeated it, in round 5's finding-7.)*

**finding-10**: record — (a) the README says the nineteen items "group into
**four** layers" above a table with five rows; (b) SDD-kk2y3e is titled "The
signed **delta envelope**", and `org-node/docs/CONTEXT.md` lists "delta
envelope" in the `_Avoid_` line under **Envelope**.
disposition: both fixed. The item is now "The signed Envelope" in the ledger
and in the plan's table.

### What round 3 says, and what three rounds say together

Round 3's verdict names the pattern precisely: *every one of findings 1, 2 and
3 sits immediately beside something an earlier round already found and fixed* —
the record lookup one level over from `find_persona`, the sibling test in the
file round 1 rewrote, and the third fuzz target the round-2 deepening declared
sound without probing it.

That is now three rounds in a row, and the repetition is the finding:

| Round | Found | The sibling it left |
|---|---|---|
| sweep | 8 greens, including `cja9zv-store-is-saved` on the receive path | the same `save` on the write path |
| round 1 | that `save` in `admit_member` and `create_organisation` | the other four `save` sites |
| round 2 | `ensure_endpoint` resting on "one persona per store" | the four record lookups resting on "one Organisation per store" |
| round 2 | two fuzz targets standing on guards that never fired | the third, and the depth of the one it had just fixed |
| round 3 | the four lookups, the third target, the depth of the second | — *to be answered by round 4* |

**The author's instinct, measured over three rounds, is to fix the example.**
Not once was a finding disputed; not once was a fix wrong where it was applied;
and every single time, the fix stopped at the instance named in the report. An
independent reader did not catch a different *kind* of defect from the author —
rounds 1, 2 and 3 all used mutation, the same instrument the sweep used. What
they caught was the author's *stopping rule*.

**So the remedy that matters is not another test.** It is the question this
record now asks of every finding before it is closed: *what is this an instance
of, and where are the others?* The four `save` sites, the four lookups, the
three fuzz targets and the twelve stale `verifies:` annotations were each found
by someone asking that question after the author had not. Rounds 2 and 3 both
closed their findings as classes rather than instances — the persistence class,
the lookup class, all three fuzz targets — which is why round 3's table above
has an empty cell rather than a fifth row of the same shape.

Whether that holds is round 4's to measure, and a round is owed: round 3 raised
three `code` and two `requirement` findings, and a round raising either is never
the last one.

## Review round 4 — 2026-10-04

A fourth fresh subagent in its own nested worktree
(`worktree-guardrails-org-node-arch-r4`), given rounds 1 to 3 and their
dispositions, and asked the same two questions: *are round 3's dispositions
true of this tree*, and *what is round 3's fix an instance of*. It ran 31
mutations in four sequential batches, restoring every mutated file from a
pristine copy and proving it byte-identical by SHA-256.

It re-ran round 3's four lookup mutations and all four are **RED on the test
the record names**. It re-ran the `fuzz_first_admission_base` reach probes and
all are RED. Every figure in the round-4 gate table re-measured correct except
one (finding-5). **Round 3's dispositions are true of this tree**, as far as
they reach.

**Verdict: reject for rework. Eight findings.** Two `code`, two
`requirement`, four `record`. Seven are accepted after re-measurement here. One
did not reproduce, and that is recorded rather than smoothed over.

**finding-1**: code — thirteen further identifier-keyed lookups, all on the
two *receive* paths and in `import_invite`, can ignore the identifier they are
given with the whole gate green (101 passed). Round 3 closed the lookup class
on the administrator's side only.
disposition: fixed, and closed **as a class measured at the final tree**, not
as the four the report described in detail. Re-measured first: the four the
author judged worst — `r4-recv-commit-any-org`, `r4-selfdel-retain-none`,
`r4-recv-pending-invite-any-org`, `r4-recv-local-trie-any-org` — all GREEN.
`r4-selfdel-retain-none` deletes **every** Organisation record the node holds
when it is revoked from one. `r4-recv-commit-any-org` writes org 2's root,
epoch, sequence mark, member snapshots and **Organisation secret** into org
1's record. `r4-recv-pending-invite-any-org` makes the first-admission sender
cross-check, which RC-b6mydy names as the explicit trust root, compare against
whichever invite came first.

Round 3's test gave the *receiving* service one Persona and one Organisation.
It was a two-Organisation test of the sender and a one-Organisation test of
everyone else. Three tests now build a receiver holding two Organisations
through two Personas, with org 2's invite imported **ahead** of org 1's so
that a "first invite" lookup picks the wrong administrator:

- `a_receiver_holding_two_organisations_commits_into_the_one_the_change_names`
- `a_self_delete_removes_only_the_organisation_the_revocation_came_from`
- `membership_of_one_organisation_is_judged_by_that_organisations_personas_alone`

The third was written *after* the first two had been probed against the other
nine sites and two stayed green: `r4-selfdel-commit-any-org` (the "still a
member" branch commits into the wrong record) and `r4-selfdel-mine-any-org`
("am I still a member" consults every Persona, not this Organisation's). The
second needs a particular scenario to be observable: org 2's administrator
enrols the device key of B's **org 1** Persona. A membership test that
consults every Persona then keeps B in an Organisation it was just removed
from. That is why the clause matters: without it, an administrator could pin a
member in place by enrolling a key the member uses elsewhere.

All thirteen re-probed at the final tree: see the appendix. Eleven are
**RED**. The other two are **GREEN** by design: the `.first()` → `.last()`
endpoint-binding mutations. They are the declared absence under finding-3.
Two low-level requirements written, LLR-y2v8v2 (receive path) and LLR-jwhzh3
(self-delete path).

**finding-2**: code — `fuzz_verify_against_chain` never reaches a successful
verification, so the `assert_eq!` on an accepted update cannot run and round
3's finding-4 is not closed.
disposition: fixed in the target, and the reviewer's diagnosis is exact. The
target seeded its `MockChain` with `local.root_hash()`, the root *before* any
delta. Check 8 compared a post-apply root against a pre-apply one, so **no
input could be accepted, by construction**. Round 3 probed that check 8 was
*reached*. Reaching it is not passing it. The chain now holds the root the
honest delta produces, so an unperturbed shape-3 input is accepted. Measured,
target alone, three runs each:

- a `panic!()` after check 8 succeeds: **RED** all three runs, first within
  246 inputs;
- check 8 weakened to compare the candidate against **its own** root: **RED**
  all three runs, through the target's `assert_eq!`, on the first input.

On the full cargo line the second mutation reddens exactly one deterministic
test, `rejects_root_mismatch_when_chain_root_differs`. The fuzz target is now
a second, independent witness to the decisive check. Gap 17, the target's doc
comment and the 2026-09-09 hazard analysis's account of this target are all
corrected. The last of these said the acceptance branch "is not reached in
practice", which was true for a reason nobody had noticed.

**finding-3**: requirement — no low-level requirement states which
Organisation record the two receive paths act on, nor which Persona's device
key they bind the endpoint from.
disposition: fixed for the record, **declined with reasons for the Persona**.
LLR-y2v8v2 and LLR-jwhzh3 state the record clause, including which Personas
decide "still a member", and the three tests above carry them. For
`personas.first()` on both receive paths, no requirement is written. A
correctness clause would be false: a device with two Personas receives on
whichever it created first. A statement of the limitation could be carried by
no gated test, because every receive test injects its endpoint with
`with_endpoint` and the lookup is never reached. Writing it anyway would
satisfy MISSING-TEST with an annotation alone, which is the move four rounds
have convicted. It is recorded in the decomposition as a fourth deliberately
unrefined behaviour, and the fix (the caller names the Persona) is an API
change, booked.

**finding-4**: requirement — the risk assessment of LLR-pw369n contradicts
PR-b9wab3, which this same change opened. §G called the no-address refusal
"strictly fail-closed", while the PR measures it burning an on-chain epoch
first.
disposition: fixed. §G is rewritten from the measurement. The refusal reaches
the same hazard §D already assesses, by a second path: the published root
loses the member while the administrator's record keeps them. The one
mitigation is that here the caller gets an error. The decomposition's "refused
before anything is sent" is corrected in place. **No new hazard or control,
and not because the behaviour is benign.** The remedy is PR-b9wab3: a
precondition that needs no I/O and is checked after `submit_update`.

**finding-5**: record — the round-4 gate table's `tracked paths 466` is not
what `check-units.sh` prints at the tree it names. It prints 467; 466 is the
count one commit earlier, before round 3 added
`org-node/docs/problems/2026-10-04-revoke-precondition.md`.
disposition: fixed by re-measuring the whole gate at the tree round 4's
remedies produce, rather than patching one figure. Round 3's finding-5 has now
recurred one round later, in the same table, under a commit whose subject was
"name the tree the round-4 figures describe". The rule stands: a round that
moves the tree re-reads **every** figure.

**finding-6**: record — `fuzz_verify_against_chain`'s shape-1 comment claims
it reaches the hand-written `sig_bytes` visitor, and the reviewer measured that
it does not.
disposition: **did not reproduce, and recorded as such.** A `panic!()` in
`sig_bytes::visit_bytes` reddened `fuzz_verify_against_chain` in **five runs
out of five**. The reviewer's proposed mechanism, that lower throughput starved
the shape, is also wrong. Postcard calls `visit_bytes` with whatever bytes
remain even when the decode as a whole then fails, so reaching the visitor
never depended on assembling a whole envelope. But the reviewer's
corroborating probe is right and is the better finding: the body of shape 1's
`if let Ok(env)` is **never entered**, in this target or in
`fuzz_envelope_decode`. A `panic!()` there passed 165 787 iterations. No fuzz
target in this unit assembles a whole `SignedDeltaEnvelope` from arbitrary
bytes. Both targets' comments now claim the decode *attempt* and say which
half is dead. One nondeterministic run in the reviewer's sandbox and five in
the author's do not make the visitor's reach proven forever. It stays a
one-second generative run, which is Gap 18's point.

**finding-7**: record — the header comment of
`a_failed_push_leaves_the_administrators_record_where_it_was` describes a
mechanism, an unbound loopback port, that the body ten lines below says does
not work.
disposition: fixed. It was round 1's discarded first draft, left standing
above the fix. The comment now describes the address-less peer identity the
body uses, with a dated note.

**finding-8**: record — LLR-37cj3n says "Organisation public key";
`org-node/docs/CONTEXT.md` names that value the **Published signing key** and
lists the field name under `_Avoid_`.
disposition: fixed. Single occurrence, confirmed by grep.

### What round 4 says

| Round | Found | The sibling it left |
|---|---|---|
| round 3 | the four lookups, the third target, the depth of the second | the thirteen receive-side lookups; check 8 reached but never passed |
| round 4 | the thirteen; acceptance; the §G contradiction | — *to be answered by round 5* |

Round 3's table ended with the claim that rounds 2 and 3 had closed their
findings as classes. **Round 4 measured that claim and it was half true.** The
lookup class was closed for one call graph, `admit_member`'s. The fuzz class
was closed for *reach* and not for *acceptance*. Each time the class was drawn
around the example the report gave, one level wider than before and still
around the example.

What this round did differently is the one thing the earlier rounds did not:
before claiming the class closed, **every one of the thirteen sites was probed
again at the final tree**, including the four that had already gone red. That
second pass found two still green and drove a third test. A round that probes
only the sites it was shown would have stopped one test short again.

A round is still owed. Round 4 raised two `code` and two `requirement`
findings, and a round raising either is never the last one.


## Review round 5 — 2026-10-04

A fifth fresh subagent, in its own nested worktree detached at `6867e21` and
removed afterwards, given rounds 1 to 4 and asked the same questions. It ran:

- 26 full-line mutations, one after another, each restored and proved
  byte-identical by SHA-256;
- three probe tests, appended and then removed;
- four fuzz probes;
- every gate in the impact set.

**Round 4's dispositions are true of this tree, with one exception.** Twelve of
the thirteen `r4m-` rows reproduce exactly, verdict and named tests. Row 215
does not (finding-11). All of finding-2's fuzz probes reproduce. Finding-6's
"did not reproduce" was re-measured by the reviewer and holds: a `panic!()` in
`sig_bytes::visit_bytes` fails `fuzz_verify_against_chain` 5/5, and shape 1's
`if let Ok` body passes 2/2 over about 165 000 iterations.

**Verdict: reject for rework. Eleven findings.** Two `code`, two `requirement`,
seven `record`. Every one was re-measured here before being accepted, and none
is disputed. **The two `code` findings are real production defects. By the
owner's ruling (2026-10-04) they are booked as problem reports and pinned, not
fixed here.** This change is an architecture ledger and has not changed
production behaviour, and it keeps that boundary.

**finding-1**: code — the receive path's Persona lookup is not scoped to the
Organisation. `receive_and_verify` picks the Persona to mark Active from every
Persona whose device key is in the verified trie, and rebinds it. So
LLR-y2v8v2's Persona clause, the last clause of LLR-jwhzh3 and LLR-jsx922 as
worded are false of the device. Round 4's test passed only because it
delivered the attack on the one path that never rebinds.
disposition: booked as **PR-mdv38y**, and reproduced by the author before
being accepted. In the reviewer's probe, run unchanged on this tree:

- b1 is rebound from org 1 to org 2;
- an ordinary org 1 update makes B delete org 1;
- org 2 revoking b2 leaves B in org 2.

That last step is the pin attack round 4 declared closed.
`pr_mdv38y_the_receive_path_rebinds_another_organisations_persona` asserts
all three, so a fix reddens it. LLR-y2v8v2 loses its Persona clause, and
LLR-jwhzh3 loses "cannot keep it in this one". Both are withdrawn rather than
left standing over a test that asserts their opposite. LLR-jsx922 now says
what the code checks: a Persona *bound to this Organisation*. LLR-e5c9ud
states the selection as it is, with the PR named. **This is the fifth round
in which a fix closed the example and left the class.** The author wrote
LLR-jwhzh3's clause and its test in round 4, and probed the self-delete path
the clause is about. The author did not ask what else writes the binding that
path reads.

**finding-2**: code — `ensure_endpoint` binds once per service, so an
administrator of two Organisations through two Personas sends the second
Organisation's admissions under the first Persona's device key. The joiner
refuses with `BadSignature` after the chain has moved. LLR-cns6q6 is true of
the first call only.
disposition: booked as **PR-8qsnhx** and pinned by
`pr_8qsnhx_a_second_organisations_admission_goes_out_under_the_first_personas_key`,
which injects no endpoint. LLR-cns6q6 is narrowed to the persona named by the
call that first binds. The receive side's `personas.first()` has the same
cure, and its "deliberately not refined" entry now says the sending side is a
defect, not an absence. Measured: the shipped app reaches this, because it
calls `admit_member` and never injects an endpoint.

**finding-3**: requirement — LLR-u6rq4s's "the trie the message verified into"
was recorded as undischargeable on an argument that was false: that removals
arrive only on the other receive path. A gated test already delivered one
through `receive_and_verify`.
disposition: fixed. `a_removal_relayed_by_the_member_it_removes_is_refused`:
C is admitted and B commits it, A revokes C, and C relays its own removal to
B. `r5f-u6rq4s-old-trie`, the reviewer's mutation, is now **RED** on that
test alone. The decomposition's "two clauses" becomes one, with the false
argument quoted and corrected in place. Gap 3 and the sweep table's row are
annotated.

**finding-4**: requirement — behaviour on SDD-8cpyfa and SDD-72ddm6 that no
low-level requirement states, four items of it green under mutation: the
Organisation secret overwritten on update, the self-delete update branch's
sequence mark, the member-side record's administrator key, and the
administrator exclusion in the Persona selection.
disposition: fixed or booked, item by item.

- **Sequence mark:** LLR-sxd3tg written, and
  `an_update_that_does_not_revoke_us_reaches_the_disk` now compares
  `last_seq` before and after. `r5f-selfdel-update-no-lastseq` is RED.
- **Administrator key:** LLR-xq9nrq written, carried by
  `a_first_admission_records_the_signing_key_the_secret_and_the_member`.
  `r5f-recv-admin-key-zero` is RED.
- **Secret:** LLR-ckk5nz states first admission (evidenced) and the overwrite
  on update. The overwrite is **PR-xwek5e**: `revoke_member` sends
  `org_secret: None`, so a member that receives another member's revocation
  loses the secret. Which behaviour is right (keep, clear or rotate) is
  unruled, and the PR leaves the ruling to the owner.
  `pr_xwek5e_another_members_revocation_clears_the_receivers_secret` pins it,
  and `r5f-recv-secret-update-kept` (the "keep" fix) reddens it.
- **Member id:** LLR-e5c9ud written. It was already evidenced, and is now
  carried as well by the first-admission test.
- **Administrator exclusion:** stated in LLR-e5c9ud and **not** evidenced.
  `r5f-recv-persona-admin-filter` stays GREEN. It is inside PR-mdv38y's site,
  and the fix is where it gets a test.

**finding-5**: record — `_Avoid_` glossary terms remained in the ledger, and
round 4's "single occurrence, confirmed by grep" was wrong. A line-based grep
cannot see a term split across a line break.
disposition: fixed across every LLR and SDD body, by a whitespace-tolerant
scan, not a line grep:

- "Organisation public key" and "its public key" became the **Published
  signing key**;
- `parent_seq` became the **Sequence number**;
- "message" became **Wire message** in nine LLRs and two SDD items;
- "payload" became the Change set bytes.

The scan also caught "administrator key" in LLR-xq9nrq, written this round,
which is too close to the avoided "admin key". Three items use "message" in the
cryptographic sense (the bytes a signature covers) and are left as they are:
LLR-e58j8m, LLR-na7p4w and LLR-9fvb3y.

**finding-6**: record — counts restated, not re-counted, in five places.
disposition: re-measured. The record header now says five rounds, sixty
findings, one hundred LLRs and five merges. The risk file says 100 and 74, with
a fifth dated note under its own "wrong four times" paragraph. The config
comment carries the gate's figure. The plan's "three blocks" now states the
rule (one per round) instead of a count.

**finding-7**: record — corrections that did not reach every document making
the claim.
disposition: fixed in all four places.

- The test comment above
  `a_loopback_revocation_with_no_peer_address_is_refused_and_records_nothing`
  no longer says "before anything is signed or submitted".
- §D and PR-b9wab3 are reconciled, not made to agree by deletion. RC-6a2dke
  keeps integrity, and the cost of the divergence is liveness. Each document
  named one side.
- The README's Evidence section names its exceptions.
- The README's list of what the gate cannot observe is completed and points to
  where each item lives.

**finding-8**: record — line citations broken by this change. The hazard
analysis's `endpoint.rs` citations landed in `send_conn` after this branch's
edits.
disposition: fixed, and closed as a class. Every `org-node/src` citation in
that file was compared at `master` and at HEAD over the nine `src` files this
branch changed. Four `endpoint.rs` citations are re-resolved (334, 322–323,
313–342 and 292), each with a dated marker. The one other range that differs,
`multisig.rs:124-142`, differs only by `fn` becoming `pub(crate) fn`, and it
still covers `build_dispatch_tx`.

**finding-9**: record — the "deliberately not refined" headings promised eight
behaviours over sections holding four.
disposition: fixed. Each heading now names its subject, not a count.

**finding-10**: record — the gate table's app row gave three per-target
figures against the wrong targets.
disposition: fixed in gate round 6. The cause was transcription: test-result
lines were read in run order and labelled in `verify_commands` order. The
figures are now read beside their `Running` lines.

**finding-11**: record — the appendix records labels, not mutations, so a row
cannot be re-run exactly, and row 215 did not reproduce under any of three
readings.
disposition: fixed for rounds 4 and 5. The appendix now carries, for every
`r4f-`/`r4m-`/`r5f-` row, the file, the line, the text replaced and its
replacement. Rows from earlier rounds were driven from scratch files that no
longer exist, and that is said rather than reconstructed. Row 215 was re-run
twice from its exact spec: see the appendix.

Two further points the reviewer raised as unmeasured are now measured:

- §F's "the PoC's app binds one Persona to one Organisation per device" is
  **false**. Neither the commands nor the UI enforce it. The §F conclusion
  still holds, on the two-Organisation tests rather than on scope, and is
  corrected in place.
- The app receives only on `receive_and_self_delete_if_revoked`, so
  PR-mdv38y's rebinding step is reachable through the library and not through
  the app's receiver. Its PR says so.

### What round 5 says

| Round | Found | The sibling it left |
|---|---|---|
| round 4 | the thirteen receive-side lookups; acceptance; the §G contradiction | the binding those lookups read, written by the other path; the sending side's endpoint |
| round 5 | Persona rebinding; one endpoint per service; the secret overwrite | — *to be answered by round 6* |

Round 4's closing paragraph said probing every site again at the final tree
was what made the difference. Round 5 shows that was necessary and not
sufficient. Every site round 4 probed was correct **as a site**. The defect
was one level up: in **state shared between paths**. One path writes the
Persona binding and another path reads it, and no single-path test can see
what the writer does to the reader. The question to ask of the next fix is
not only "where are the other sites", but **"what else writes what this
reads"**.

A round is owed. Round 5 raised `code` and `requirement` findings.


## Review round 6 — 2026-10-04

A sixth fresh subagent, in a nested worktree detached at `93e317c` and removed
afterwards, given rounds 1 to 5, the owner's ruling that booked-and-pinned
production defects are not to be re-raised as unfixed, and round 5's lesson as
its brief: **"what else writes what this reads"**.

**Round 5's dispositions are true of this tree.** Seven `r5f-` rows re-run from
the appendix's recorded specs reproduce, both verdict and named tests. **Every
pin pins.** The reviewer applied the obvious fix for each booked defect as a
mutation, and each pin went red alone:

- PR-mdv38y, the selection scoped to this Organisation;
- PR-8qsnhx, a rebind when the named Persona differs;
- PR-xwek5e, both the "keep" and the "rotate" readings.

**Verdict: reject for rework. Ten findings.** Four `code`, five
`requirement`, one `record`. None disputed. The author reproduced finding-1
before accepting it.

### The owner's ruling on convergence — 2026-10-04

Six rounds have each found real defects, and from round 5 on most of them are
in **production code this change does not touch and, by an earlier ruling,
does not fix**. org-node's problem-report backlog reached its limit of ten in
this round. The owner ruled that, from round 7, **a production defect outside
this change's own claims, once booked as a problem report and pinned by a test
that reddens when it is fixed, does not block convergence**. Only defects in
what this change itself asserts block it: its requirements, its tests, its
records. The next change to org-node is a fix change that closes problem
reports.

The rule this record applied until now, that any `code` or `requirement`
finding means another round, is unchanged for findings about this change's own
claims. Round 7 is owed under it, because round 6 raised five `requirement`
findings and all five are about this change's ledger.

### The findings

**finding-1**: code — the node's own revocation, arriving on
`receive_and_verify`, is committed as an ordinary update. The node keeps the
record and its Persona stays Active, contrary to REQ-uxv2x2, which is not
scoped to a path, and to `revoke_member`'s own doc comment.
disposition: reproduced by the author on this tree (`Ok`, epoch 3, record
kept, B not in it, Persona Active, the same on disk), booked as
**PR-322qst**, and pinned by
`pr_322qst_an_own_revocation_on_the_ordinary_path_is_committed_not_self_deleted`.
SDD-8cpyfa now says it does not apply REQ-uxv2x2 and names the PR. The app's
receiver does not reach it. This is org-node's **tenth** open problem report,
at the limit.

**finding-2**: code — `create_organisation` is a second writer of the Persona
binding the self-delete path reads, which PR-mdv38y's stated cure does not
reach.
disposition: folded into **PR-mdv38y**, not opened as an eleventh report. It is
the same binding and the same model decision: one Persona per Organisation, or
a binding per Organisation. The report's cure now says so.
`pr_mdv38y_founding_an_organisation_rebinds_a_member_persona` pins it.
LLR-w3fhhg, which requires the rebind, now names the PR, and the PR's
`affects:` names LLR-w3fhhg.

**finding-3**: code — `export_join_request` reads the bound endpoint's address,
which another Persona may have written, so a join request can name two
devices.
disposition: folded into **PR-8qsnhx** as its reading side, with the same cure,
an endpoint per Persona.
`pr_8qsnhx_a_join_request_advertises_the_bound_endpoint_not_its_personas`
pins it. LLR-437fvx names it.

**finding-4**: code (test) — `service_lifecycle.rs::store_at`, added by this
change, keyed its store on the process id and never cleared it, so a reused
pid opened a populated store. The reviewer measured three unrelated failures
on one run that vanished on re-run. `five_stories_full_e2e` had the same
pattern.
disposition: fixed in both. Each now clears its directory first, as every
other temp-store helper in `org-node/tests` already did; the author audited
all eight `temp_dir()` call sites. This is a **measured mechanism** for row 215's
one-off, which round 5 withdrew as unexplained. It is the likelier
explanation, and it is recorded as likelier, not as proven.

**finding-5**: requirement — LLR-t4znbk and LLR-qg9utu state as required the
publish-before-persist order that PR-vt244s, open since 2026-09-09, books as
the defect. Their tests are pins presented as verification.
disposition: fixed in the ledger. Both LLRs now say they state PR-vt244s's
order without endorsing it, as LLR-ckk5nz and LLR-cns6q6 already did.
PR-vt244s's `affects:` names both. The tests keep their annotations, because
they verify what the LLRs now say.

**finding-6**: requirement — LLR-6zjzn2 states the bind-once behaviour that
PR-8qsnhx books as the defect, unflagged.
disposition: fixed in the same way.

**finding-7**: requirement — the root-hash write in LLR-t4znbk, LLR-qg9utu and
LLR-cja9zv could be removed with the gate green, because nothing in the
library reads `OrgRecord.root_hash`.
disposition: fixed. `an_admission_reaches_the_administrators_disk`,
`a_revocation_reaches_the_administrators_disk` and
`a_receiver_holding_two_organisations_commits_into_the_one_the_change_names`
now compare the record's root with the chain's, in memory and on disk. All
three mutations are re-probed: see the appendix.

**finding-8**: requirement — LLR-xq9nrq, LLR-ckk5nz and LLR-e5c9ud, written by
round 5, traced to REQ-xa6smf, which they do not refine. That took them out of
the derived-behaviour risk assessment, and the risk file said they "change no
assessment".
disposition: fixed. All three are `satisfies: derived` and assessed in the risk
file. LLR-e5c9ud's assessment records that RC-wqgm2p is **weakened** by
PR-mdv38y, without minting a new control, because the remedy is the fix and
the control already carries residual risk "not acceptable". The risk file's
claim is corrected in place.

**finding-9**: requirement — `revive_update_runtime_call`, pure and
synchronous, was filed in SDD-z85ux9, the item without low-level requirements,
under a list headed "the asynchronous functions only", and the async
`submit_and_watch` was left out.
disposition: fixed. The function moves to SDD-msb6xh with **LLR-rc74nq**
(derived, assessed), and
`update_call_names_every_field_and_constant_the_runtime_matches` pins every
name and constant the runtime matches. `submit_and_watch` joins SDD-z85ux9's
list.

**finding-10**: record — (a) SDD-z85ux9's size was never measured: "roughly
815 … a fifth" came from the whole files alone; (b) the risk file's "is 94" was
in the present tense; (c) LLR-ckk5nz's update clause was traced only to a test
that cannot falsify it.
disposition: (a) measured by brace-matching each part from its first doc
comment to its closing brace: **1125 of 3768 lines, about 30%**, corrected in
the decomposition, README, plan and Gap 1. (b) The sentence is now in the past
tense, with the current figures beside the section that last changed them.
(c) The PR-xwek5e pin now carries `verifies: LLR-ckk5nz`. The LLR states the
behaviour without endorsing it, and the pin is the only test that can falsify
that clause.

### An observation outside this change

The reviewer noted from reading, without testing, that **the shipped app
cannot complete a joiner's first admission**. Its only receive loop is
`receive_and_self_delete_if_revoked` (`app/src-tauri/src/commands.rs:339`),
which refuses with `OrgNotOnChain` when no record of the Organisation exists
(`org-node/src/service.rs:1293-1301`). The author confirmed by reading that no
other app path calls `receive_and_verify`, and that nothing in `app/docs`
records it. By the owner's ruling it is noted here and **booked for the next
app change**, as an app problem report. It is not opened here, because it is a
defect in the `app` unit and this change does not touch that unit's ledger.
Gap 20.

### What round 6 says

| Round | Found | The sibling it left |
|---|---|---|
| round 5 | Persona rebinding on one path; the sending endpoint; the secret overwrite | the binding's second writer; the endpoint's reader; the own-removal on the other path |
| round 6 | those three, by the shared-state method | — *to be answered by round 7* |

Round 5's method found round 6's code findings: list every piece of state,
then its writers and its readers. **The production defects are a cluster, not
a scatter.** All four of PR-mdv38y, PR-8qsnhx, PR-322qst and PR-xwek5e are
consequences of one design assumption: one Persona, one Organisation and one
endpoint per store. The code never states that assumption and the app does not
enforce it. Each is booked. Together they are the subject of the fix change
that follows this one, which should begin with the model decision PR-mdv38y
records, not with any one of the four.

*Corrected by review round 7. "All four … are consequences of one design
assumption" is false for two of them. The PR-322qst and PR-xwek5e pins both
reproduce on `setup()`/`admit_b_directly`, where every store holds one Persona
and one Organisation. The shared cause is real for PR-mdv38y and PR-8qsnhx.
PR-322qst is the own-removal rule applied on one receive path and not the
other, and PR-xwek5e is an unruled write-through of the secret. The fix change
still begins with the model decision. It is not the whole of it.*


## Review round 7 — 2026-10-04

The first round under the owner's convergence ruling. A seventh fresh
subagent, in a nested worktree detached at `5456376` and removed afterwards,
was told which findings block (about this change's own claims: `requirement`,
`test`, `record`) and which do not (`defect-outside-scope`).

**Round 6's dispositions are true of this tree.** All five `r6f-` rows
reproduce exactly, each RED on its one named test. **Every pin pins.** The
reviewer applied a plausible fix to each of the three new pins and to the
three older ones (PR-b9wab3, PR-vt244s on both paths, PR-8qsnhx on the sending
side), and each went red. The root assertions observe what they claim.

**Verdict: reject for rework. Nine findings.** Five `requirement`, one `test`,
two `record`, one `defect-outside-scope`. Eight block. Most are round 6's
classes in places round 6 did not reach: two **older** open problem reports
that requirements contradict without naming, and further unevidenced writes.

**finding-1**: requirement — LLR-3q63zv's reason ("a node being removed cannot
be required to find the remover") does not hold on the `UpdatedNotRevoked`
branch, where PR-u4c2vp, open since 2026-09-09, books the missing cross-check
as a defect. Nothing named the report, and the risk file said "the code, the
requirement and the control agree".
disposition: fixed. LLR-3q63zv names PR-u4c2vp for the update branch, and
PR-u4c2vp's `affects:` names LLR-3q63zv. The risk section is corrected in
place.
`pr_u4c2vp_an_update_relayed_by_a_non_member_is_committed_on_the_self_delete_path`
pins it, and carries the LLR. The reviewer's mutation adding the check is now
RED.

**finding-2**: requirement — LLR-jn5jeh, and the test carrying it, state the
behaviour PR-2dmjzj books as a defect: the Loopback dial does not check that
the address names the joiner.
disposition: fixed in the ledger. LLR-jn5jeh names PR-2dmjzj and says one of
its cures would replace the requirement. PR-2dmjzj's `affects:` names
LLR-jn5jeh. The reviewer measured that adding the check reddens sixteen tests,
including `five_stories_full_e2e` and three pins, so the fixtures themselves
depend on the defect. That is recorded under the LLR, so the fix change
expects it.

**finding-3**: requirement — two behaviours in SDD-8cpyfa and SDD-72ddm6 that
no requirement stated: (a) a first admission with no imported invite is
accepted on the chain anchor and signature alone; (b) the self-delete path
refuses an Organisation it holds no record of. Both were green under
mutation.
disposition: fixed. **LLR-mbjfq8** and **LLR-379hnv** are written, both
derived and assessed, and pinned by
`a_first_admission_with_no_imported_invite_rests_on_the_chain_alone` and
`the_self_delete_path_refuses_an_organisation_it_holds_no_record_of`. (b) is
the mechanism behind Gap 20.

**finding-4**: requirement — clauses whose only falsifier was an unannotated
pin (LLR-w3fhhg's overwrite, LLR-437fvx's address), LLR-sxd3tg's root clause
observed only by a test traced elsewhere, and PR-xwek5e's `affects:` naming
LLR-cja9zv, not LLR-ckk5nz. This is round 6's finding-10(c), fixed then
for one LLR and not for its siblings.
disposition: fixed, all four. The two pins carry `verifies:` for the clauses
they falsify. `an_update_that_does_not_revoke_us_reaches_the_disk`, which
carries LLR-sxd3tg, now compares B's root with the chain's. PR-xwek5e's
`affects:` is corrected.

**finding-5**: test — the update-call test used a uniform contract address,
so it could not see the byte order of `dest`.
disposition: fixed. The address is now twenty distinct bytes, and the
reviewer's `.iter().rev()` mutation is RED.

**finding-6**: requirement — the founding record's root was unevidenced:
round 6's root-hash class, missed at `create_organisation`.
disposition: fixed.
`creating_an_organisation_advances_the_chain_and_activates_the_persona`
compares the record's root with the chain's, in memory and on disk.

**finding-7**: record — (a) the README and round 6's section said all four
recent defects share one cause, and PR-322qst and PR-xwek5e reproduce on a
one-Persona, one-Organisation store; (b) "Eight of the items live … in
`service.rs`": seven do.
disposition: fixed, both. Round 6's paragraph is corrected in place: the
shared cause is real for PR-mdv38y and PR-8qsnhx, and the other two have
causes of their own. The decomposition says seven and names them.

**finding-8**: record — `_Avoid_` terms remained: "message" for Wire message,
"administrator key", and one stale citation in PR-b9wab3.
disposition: fixed. Every org-node document this change wrote was scanned with
code spans masked and line breaks tolerated, and twelve hits were corrected.
Kept, and named: error "messages", the three cryptographic uses, an ed25519
"public key", and one quotation of the old LLR wording. PR-b9wab3 now cites
`match mode` at `:1212`. The reviewer also observed PR-d4nye8's
`endpoint.rs:100-111`, stale at master and moved further by this change's
inserted comment, and it is re-resolved to `:134-140`.

**finding-9**: defect-outside-scope — `receive_and_self_delete_if_revoked`
reports a missing *local* record as `OrgNotOnChain`, although the chain holds
the Organisation, so a caller cannot tell the two apart.
disposition: **not booked**, because org-node is at its limit of ten open
problem reports. It is recorded as Gap 21 for the fix change, and LLR-379hnv
states the misnomer without endorsing it.

### What round 7 says

| Round | Found | The sibling it left |
|---|---|---|
| round 6 | the shared-state cluster; root writes on three paths; three mis-traced LLRs | the root write at founding; two older reports no LLR named; the siblings of the trace fix |
| round 7 | those | — *to be answered by round 8* |

**Every blocking finding in round 7 was in this change's own ledger, and none
required production code.** Under the ruling, round 8 is owed, because round 7
raised `requirement` findings about this change's claims. Two observations to
carry into it:

- The two older reports, PR-u4c2vp and PR-2dmjzj, were missed because rounds
  4 to 6 cross-checked the problem reports **this change opened** against the
  ledger, and not the ones that were already open. Before round 8, the author
  should check every open report's `affects:` and wording against every LLR,
  and not only the new ones.
- The trace class (a clause falsified only by a test traced elsewhere) has now
  been found in rounds 6 and 7. It is mechanical to check: for each mutation in
  the appendix, does the red test carry the LLR the mutation falsifies? That
  check should run before round 8, not inside it.

**Both checks were run by the author before round 8 was dispatched.**

- **Every open report against every claim.** org-node's ten open reports were
  read against the decomposition and the risk file. Eight were already
  reconciled. PR-w88sr9 contradicts nothing in the ledger. **PR-hqwpg9 did**:
  the risk file said the seeds and the Organisation secret "have no `Debug`
  written for them by this unit", while PR-hqwpg9, which arrived with the
  `24317e3` merge, books that the records holding them derive `Debug`. Corrected
  in place.
- **Every RED mutation in rounds 4 to 7 against the LLR it targets.**
  `tracecheck.py` maps each row's label to the LLR it was written to falsify,
  and asks whether any test it reddened carries that LLR. It found three rows
  with none.
  - `r4m-import-invite-any-org` falsifies LLR-9zfnmb's "per Organisation"
    clause, which only the two-Organisation fixture observes.
  - `r4m-recv-persona-any` falsifies LLR-e5c9ud, which did not exist when the
    row ran.
  - In both cases
    `a_receiver_holding_two_organisations_commits_into_the_one_the_change_names`
    is red and now carries the LLR.
  - The third, `r5f-recv-member-id-any`, is the probabilistic row already
    recorded in round 5's block. Its deterministic sibling,
    `r5f-recv-member-id-unset`, reddens the test that carries LLR-e5c9ud.

## Review round 8 — 2026-10-05

An eighth fresh subagent, in a nested worktree detached at `996a400` and
removed afterwards, under the same instructions as round 7. Its mutation run
was stopped once at the two-hour background limit with
`r8-recv-one-checks-prefix` applied to `endpoint.rs`. The reviewer restored
the file from HEAD, proved it identical by SHA-256, and re-ran that row and the
four after it. The change worktree was not edited.

**Round 7's dispositions are true of this tree.** All six `r7f-` rows
reproduce exactly. The new pin pins: adding the sender check PR-u4c2vp's
outline describes reddens only its pin. The two new carriers carry. The gate
reproduces the record's figures, except that `check-review.sh --branch` reports
"provenance NOT checked", where the record ran it without `--branch`.

**Verdict: reject for rework. Twenty findings.** Eleven `requirement`, nine
`record`, none `test` and none `defect-outside-scope`. All twenty block. They
are rounds 6 and 7's classes in places those rounds did not reach: behaviour
no requirement states, clauses falsified only by tests traced elsewhere, and
record text that a later correction did not reach.

### The owner's second ruling on convergence — 2026-10-05

Round 8 raised more blocking findings than round 7 (twenty against eight).
Six of them (2, 3, 4, 7, 8 and 9) between them name eight behaviours that no
requirement stated, each found by a new mutation somewhere in `service.rs` or
`endpoint.rs`. That search has no
natural end: every line no mutation has touched can yield another. The owner
ruled: **fix all twenty, then a narrow round 9.** Round 9 checks round 8's
dispositions and the record. It does not search for new unstated behaviour.
Anything of that kind it finds is recorded as a gap for the next change, not a
blocker. Then merge.

### The findings

**finding-1**: requirement — LLR-cja9zv states that the record is written
after verification succeeds, with no exception for a change that removes the
node itself. PR-322qst books that commit as a defect. Neither the LLR nor the
report's `affects:` named the other, and the pin carried no `verifies:`.
disposition: fixed. LLR-cja9zv is annotated with PR-322qst, the report's
`affects:` names LLR-cja9zv, and
`pr_322qst_an_own_revocation_on_the_ordinary_path_is_committed_not_self_deleted`
carries LLR-cja9zv. The reviewer's fix-probe (`r8f-322qst-fix-refuses`)
reddens only that pin.

**finding-2**: requirement — no LLR of SDD-72ddm6 stated the Wire message
`revoke_member` sends: the snapshot from before the removal, and no secret.
Emptying the snapshot was green, and the secret was observed only by the
PR-xwek5e pin, which carries LLR-ckk5nz.
disposition: fixed. **LLR-8hdu9x**, derived and assessed, names PR-xwek5e.
`a_revocation_reaches_the_administrators_disk` now compares the snapshot it
received with A's record before the removal, byte for byte, asserts that no
secret was sent, and carries LLR-8hdu9x. PR-xwek5e's `affects:` names it.

**finding-3**: requirement — `export_invite` writes the bound endpoint's
address, whichever Persona bound it, or an empty one if none is bound. No LLR
stated this, and emptying it was green.
disposition: fixed. **LLR-qezw3n**, derived and assessed, names PR-8qsnhx's
reading side. PR-8qsnhx's `affects:` and text name the second reader.
`the_out_of_band_blobs_carry_the_keys_their_holders_are_pinned_by` checks the
address against the bound endpoint's, and checks it is empty on a service with
no endpoint.

**finding-4**: requirement — the pure-proxy account was stated by nothing and
evidenced by nothing. `MockChainOps` returns none and ignores what it is
given, so not storing it at genesis and not passing it to an update were both
green.
disposition: fixed. **LLR-dzte8x** (stored at genesis and on disk),
**LLR-3v5nu9** (passed by `admit_member`) and **LLR-drgdy8** (passed by
`revoke_member`), all derived and assessed.
`the_proxy_account_from_genesis_is_kept_and_passed_on_every_update`
substitutes a `ChainOps` that returns a proxy account at genesis and records
the account each update is submitted with. The reviewer's two rows and a third
for the revocation site are red on it.

**finding-5**: requirement — LLR-q8emds could not be made to fail: consuming
the invite before verification was green.
disposition: fixed.
`a_first_admission_that_fails_verification_leaves_the_invite_pending` sends A's
genuine admission from A's own device key with the envelope's mark altered, so
it passes the invite cross-check and then fails verification. It asserts that
the invite is still pending in memory and on disk, and carries LLR-q8emds.

**finding-6**: requirement — LLR-379hnv's "nothing is written" was only partly
falsifiable: marking every Persona Revoked on the refusal was green.
disposition: fixed. The carrier now asserts that B's Persona is untouched in
memory (status, binding, member id) and that no Persona's status reaches the
disk changed.

**finding-7**: requirement — `MockChainOps`'s epoch check was stated by no
LLR, and disabling it was green.
disposition: fixed. **LLR-ryzr8m**, derived and assessed, under SDD-ueh4tm.
`the_mock_chain_refuses_an_update_at_the_wrong_epoch` offers it an epoch below
and an epoch above the slot's, each refused with the slot unchanged, and then
the right one.

**finding-8**: requirement — `create_organisation` leaves the founding
Persona's member id as it was, and no LLR said so.
disposition: fixed. **LLR-q3aj8z**, derived and assessed, names PR-mdv38y's
second-writer path, and PR-mdv38y's `affects:` names it. The founding test
asserts the member id is none. The PR-mdv38y founding pin asserts that it is
still the id from org 1.

**finding-9**: requirement — `recv_one` discards the length prefix without
checking it, and decodes a short stream whole. No LLR stated this.
disposition: fixed. **LLR-2smrvx**, derived and assessed: the prefix is
vestigial, because the stream's end delimits the frame and the read limit
bounds it. `the_length_prefix_is_not_checked_against_the_body` sends a
well-formed body behind a zero prefix and sees it accepted. Checking the
prefix reddens it.

**finding-10**: requirement (wrong trace) — LLR-duwz79's only carrier never
called `from_last_seen`. Starting every guard at zero reddened six tests and
none of them carried it.
disposition: fixed. `from_last_seen_starts_the_guard_at_the_given_mark`
carries LLR-duwz79, over four marks including zero and the largest the
watermark admits. `advance_moves_high_water_mark_forward_only` no longer
claims it.

**finding-11**: requirement (wrong trace) — LLR-wx77j5's identity clause was
falsified only by tests traced to LLR-437fvx and LLR-k2y6nn. Its carrier stayed
green.
disposition: fixed. `loopback_mode_binds_and_advertises_loopback_only` now
asserts that `node_addr_for_dial` reports the endpoint's own identity, equal
to the device key, and exactly the sockets the endpoint has bound.

**finding-12**: record — section A of the derived-risk file endorsed the order
PR-vt244s books as a defect ("the right direction of error") and did not name
the report.
disposition: fixed. The paragraph now names PR-vt244s and the hazard
register's publish-before-persist hazard and its not-minted control, says the
file does not endorse the order, and keeps the old wording quoted in the
correction note.

**finding-13**: record — the risk file's LLR-3q63zv section said a message on
this path "cannot cause a self-delete unless the chain itself says the node is
out". PR-mdv38y's pinned step 2 contradicts that.
disposition: fixed. The sentence carries the PR-mdv38y exception and names its
pin.

**finding-14**: record — the risk file said `export_invite` reads the stored
key "when a member re-shares an invite". Measured by the reviewer: on a member
device `export_invite` refuses at `:682`, before `:697`.
disposition: fixed. The sentence says what was measured.

**finding-15**: record — round 7's correction did not reach every document:
README "eight" items in `service.rs`, "one spans parts of six modules" in the
README and the plan, and `calldata.rs` listed among the files whose
asynchronous functions SDD-z85ux9 holds, although it has none.
disposition: fixed. The README says seven. The README and the plan say
SDD-z85ux9 spans seven whole files and parts of two more (`service.rs` and
`multisig.rs`). `calldata.rs` is removed from SDD-z85ux9's list. Its two
functions are SDD-msb6xh's. The measured 1125 lines did not count it, so the
figure is unchanged.

**finding-16**: record — the README said six known defects are pinned, and
left out PR-u4c2vp, whose pin was written in round 7.
disposition: fixed. Seven, with PR-u4c2vp named among those with a cause of
their own.

**finding-17**: record — Gap 12 said the 2026-10-04 reports fire on
2026-11-03. `check-trace.sh` fires at `age > 30`, so the date is 2026-11-04.
disposition: fixed, with a dated note in Gap 12.

**finding-18**: record — two counts in the risk file were stale in the present
tense: "Twenty-six are now" and "The count is now 71 of 100".
disposition: fixed. Both are in the past tense, and the note points to the
counts at the end of the file: 111, 40 derived and 71 inherited after this
round.

**finding-19**: record — terms the root `docs/CONTEXT.md` says to avoid:
nineteen hits of "member key", "revoked member" and "removed member". Round 7
had scanned against org-node's list only.
disposition: fixed in every document this change wrote. The scan was rerun
against the root list as well as org-node's, with code spans masked and line
breaks tolerated. The hits were corrected to "Member-as-a-group key", "the
Member being revoked", "the Member being removed" and "the Member it removes".
The SDD-sxp8hb heading is now "Device key and Member-as-a-group key custody",
in the plan's table too. Kept, and named:

- Five hits on five lines of `org-node/docs/risk/2026-09-09-org-node-hazards.md`
  (`:250`, `:264`, `:265`, `:499`, `:621`) and one "best block" in
  `org-node/docs/problems/2026-09-09-org-node-problems.md`. All were on
  `master` before this change, and none is in text this change wrote.
- "trie" and "delta" throughout. *Widened by review round 9, which found this
  exception stated as "where they name the implementation types `OrgTrie` and
  `Delta`" while many kept uses mean the Membership record itself ("B is still
  in org 1's trie").* The truth is that this change's documents use "trie" as
  the code does, for the Membership record a node holds, and "delta" for a
  Change set's encoding. Rewriting every use is not done here. It is recorded
  as a known divergence from the root glossary, for the change that next
  rewrites these documents.
- "user" in "user stories", and in master-era hazard text ("the user"); and
  "account" wherever it names a chain or multisig account, or means "an
  account of". *Named 2026-10-05 after review round 10, which counted them.
  The one other use, "what a user does" in the README's layer table, is
  corrected to "what a Persona does".*

**finding-20**: record — the decomposition said "one of the fourteen harnessed
targets" in `org-node/tests`, which holds thirteen.
disposition: fixed. Thirteen there, and fourteen harnessed targets with
`--lib`, which carries no requirement.

### Round 8's trace check of rounds 1 to 3, and one row it did not re-run

The reviewer extended the author's trace check to appendix rows 1–168, which
`tracecheck.py` had not covered. Rows 7 and 107 are findings 10 and 11. For row
73 (`vw2jn6-decides-on-verified`) the recorded red tests do not carry
LLR-vw2jn6. The reviewer did not re-run it. Its specification is not kept from
the 2026-10-03 sweep. Its two red tests both carry LLR-6p4pj2, whose
"absent from the **verified** trie" clause a decision on the wrong trie
falsifies, so the row is traced, but to LLR-6p4pj2. LLR-vw2jn6's ordering
clause was measured afresh: `r8f-vw2jn6-acts-before-verify` makes the
self-delete path delete the record when verification fails, and
`unverified_revocation_leaves_the_record_in_place`, which carries LLR-vw2jn6,
is red.

### What round 8 says

| Round | Found | The sibling it left |
|---|---|---|
| round 7 | two older reports no requirement named; two unstated behaviours; the trace class's siblings | eight more unstated behaviours; two mis-traces from rounds 1–3; record text round 7's corrections did not reach |
| round 8 | those | — *to be checked, not searched, by round 9* |

**None of round 8's findings needed production code**, and this round
changed none. The unstated-behaviour class is now bounded by ruling, not by
exhaustion. The verification record says so, rather than claiming the ledger
complete. Any further behaviour round 9 finds unstated is recorded as a gap.

## Review round 9 — 2026-10-05

The narrow round the owner's second ruling set. A ninth fresh subagent, in a
nested worktree detached at `4100137` and removed afterwards, checked round
8's dispositions and the record. Under the ruling it did not search for new
unstated behaviour. No run was killed, and every file was proved identical to
HEAD by SHA-256 after each run.

**Round 8's dispositions are true of this tree, with the four exceptions
below.** The gate reproduces at `4100137`, apart from the swapped bolero rates
(finding-3). All fifteen `r8f-` rows are RED on exactly the tests the appendix
names, each carrying its target LLR. Every ID the new risk entries cite exists,
and their factual claims hold. The recounts match: 111, 40 and 71
requirements; 99 findings; 283 runs, 224 RED, 54 GREEN and 5 LIBBROKEN; 76
clauses resting on nothing.

**Verdict: reject for rework. Four findings.** Two `requirement`, two
`record`, none `test` and none `defect-outside-scope`. All four block. Each
is a clause of a round-8 requirement, or a sentence of round 8's record.

**finding-1**: requirement — LLR-8hdu9x's receiver clause ("a receiver on
`receive_and_verify` stores that absence over the secret it held") was
falsified only by the PR-xwek5e pin, which carried LLR-ckk5nz.
disposition: fixed. The pin carries LLR-8hdu9x as well, and the ledger says
so. `r9f-8hdu9x-receiver-keeps-secret` is RED on it.

**finding-2**: requirement — two clauses of round-8 requirements had no test.
(a) LLR-qezw3n's "whichever Persona bound it": restricting the Invite's
address to the administrator's own endpoint was green, because no test
exported an Invite with the endpoint bound from another Persona. (b)
LLR-2smrvx's "a stream shorter than four bytes is decoded whole": refusing it
at the read was green.
disposition: fixed, both. (a)
`pr_8qsnhx_an_invite_advertises_the_bound_endpoint_not_its_administrators`
founds the Organisation with one Persona, binds the endpoint from another, and
asserts that the Invite names the first's device key and the second's address.
It pins PR-8qsnhx's second reader, and the report names it. (b)
`the_length_prefix_is_not_checked_against_the_body` also sends three bytes and
asserts that the decoder refuses them as `Malformed`. A refusal at the read
would be a stream error.

**finding-3**: record — the gate table's bolero rates were transcribed against
the wrong targets. `fuzz_first_admission_base` ran at 2 563/s and
`fuzz_verify_against_chain` at 909/s, and the table listed them the other way
round.
disposition: fixed, with a note at the figures.

**finding-4**: record — finding-19's disposition overstated what was fixed:
- Two "member key" hits remained in this record, outside quotation.
- The stated exception for "trie" was narrower than the uses kept, many of
  which mean the Membership record itself.
- The master-era hazard exceptions were five hits on four lines, not four
  hits.
- The ruling paragraph said "eight" of round 8's findings were unstated
  behaviours, where six findings name eight behaviours.
disposition: fixed:
- Both hits now read "Member-as-a-group key".
- The exception now says what was kept: "trie" and "delta" as the code uses
  them, recorded as a known divergence from the root glossary.
- The hazard count and the ruling paragraph are corrected.

### What round 9 says

The class persists even inside a round that checks its predecessor's work and
does not search: a clause written to state a behaviour can still rest on a
test traced elsewhere, or on no test at all. Round 9 found three such clauses
among round 8's new requirements, and no new behaviour. Its findings are
fewer and smaller than any earlier round's. A round 10 limited to these four
dispositions follows.

## Review round 10 — 2026-10-05

A tenth fresh subagent, in a nested worktree detached at `70406b2` and removed
afterwards, checked round 9's four dispositions and the `f688a3c` merge, and
re-measured the gate. No run was killed, and every file was proved identical to
HEAD by SHA-256.

**Every requirement and test disposition of round 9 is confirmed.** The three
`r9f-` rows are RED on exactly their named tests, each carrying its target
LLR. The reviewer's own mutation of the short-stream clause, always stripping
four bytes, is RED on the same test. A second mutation, decoding an empty
slice, is GREEN and equivalent: no body of 0 to 3 bytes can decode, so both
versions refuse it as `Malformed`. The ledger describes what is tested.
**The merge is master's tree on master's paths**: the 39 files `ed5b66d`
changed are exactly the 39 `f688a3c` changed, none was touched by this
branch, and `git diff f688a3c ed5b66d` is empty over them. The new `_Avoid_`
terms have no hits. **The gate reproduces every figure** at `70406b2`.

**Verdict: reject for rework. Two findings, both `record`, both one-word
counts.**

**finding-1**: record — the plan said `master` was merged "six" times and,
in the next sentence, that the branch "carries five merge commits".
`git log --merges master..HEAD` names six.
disposition: fixed. The plan says six in both places.

**finding-2**: record — round 8's finding-19 disposition, as round 9
corrected it, said "five hits on four lines" of the hazard register. They are
on five lines (`:250`, `:264`, `:265`, `:499`, `:621`).
disposition: fixed, with the five lines cited one by one.

**Also noted, non-blocking:** "user" and "account", root `_Avoid_` terms, in
this change's prose. Every use is now named among the kept exceptions, and the
one use that meant a Persona is corrected.

**No round 11 was run for these.** Both fixes are one word in a count, and the
author checked each against its source: `git log --merges master..HEAD | wc -l`
gives six, and the five hazard lines were re-read. This record says so rather
than calling them reviewed.

## The merge gate — 2026-10-05

`verify-before-merge`, dispatched to a fresh gate subagent at `fc783a4`, after
the deslop pass. Logs are in the author's scratch directory, `gate-final/`.

**The deslop pass** (`develop-change`) was run over the whole change diff
before the gate, and its fixes are in `e2baab7`. In `org-node/tests` it
extracted helpers for lookups repeated across rounds, removed seven no-op
lines, and reused one closure in the fuzz target. No test was renamed, and no
assertion or `verifies:` line was changed. The author read the diff: every
change observes the same values at the same points. The one citation it
moved, `fuzz_target.rs:161`, is re-resolved in `fc783a4`.

| Check | Result |
|---|---|
| 1. org-node `verify_commands` | cargo 122 passed, 0 failed; three bolero targets ran their budget clean (161 958/s, 2 761/s and 784/s for envelope_decode, first_admission_base and verify_against_chain); quint 0.33.0, two typechecks ok, five invariants no violation |
| 1. app `verify_commands` | cargo 78 passed, 0 failed; `npm run check` 194 files, 0 errors, 1 warning (tsconfig `node`, as before); `npm run test` 30 passed |
| 2. `check-trace.sh` | org-node exit 0: REQ 20, HAZ 6, RC 9, SDD 19, LLR 111, PR 13, open 10 of 10. app exit 0. `check-units.sh` exit 0: units 5, tracked paths 514; impact org-node touched, app dependent |
| 3. `check-ids.sh --allow-draft-files` | org-node and app exit 0, no output |
| 4. Implements map, the gate's half | all 111 LLRs carried by a `verifies:` test; none without one. LLR-hs7g6j is carried by `fuzz_verify_against_chain` alone |
| 4. the author's half | the appendix's mutation attestations stand in for `red -> green:` lines, as the plan's "Red → green attestations" states. The trace from a red row to a test carrying its LLR was checked mechanically by the author for rounds 4 to 7, and by review round 8 for rows 1 to 168. Rows 263 to 286 cover every LLR added since. The mis-traces those checks found are corrected above, and the LLRs no mutation could falsify are named in the Gaps. LLR-hs7g6j is carried by a fuzz target, whose reach round 4's block proves by mutation |
| 5. Coverage | **not configured in org-node or app**, a recorded gap against class C. **Accepted for this merge by the owner on 2026-10-05**, and left booked for tooth 5, which sets coverage floors for both units on one basis (`docs/plans/2026-09-05-ratchet-setup.md`) |
| 6. Robustness | twelve items have a normal and an abnormal case. Seven have none, as the decomposition's robustness table states. The gate found SDD-swtd3w accounted for nowhere, and the table not mentioning SDD-ueh4tm's one refusal case. Both are recorded in `be8c24f` |
| 7. `git status` | clean before and after |
| 8. Deslop | run, fixes applied in `e2baab7` |

The gate ran at `fc783a4`, and `be8c24f` changed only the decomposition's
robustness prose. It was re-dispatched at the final tree before merging (see
below).

## Gaps

1. **SDD-z85ux9 carries no low-level requirements** — 1125 of 3768 source
   lines, about 30% of the unit, measured by review round 6 (it read "roughly
   815 … a fifth", which was never measured), under a class that requires them. Argued in full
   in `org-node/docs/architecture/2026-10-03-decomposition.md`. The code *is*
   exercised, by `chain_genesis_e2e`, `finality_polling` and `preflight`; those
   three targets are excluded from `verify_commands` because they spawn a
   chopsticks fork, so no mutation there would redden this gate. Remedy —
   bringing them into the gate — is booked, not attempted.
2. **LLR-6adc99 / REQ-2wzfzv is unverifiable at this gate.** Both clauses are
   about bind failure and no test can produce either antecedent. `check-trace.sh`
   is green over it because MISSING-TEST is satisfied by the annotation alone —
   the **third** time this repository has recorded that property, now in a second
   unit. The `test-support` bind-address constructor that would close it is
   booked.
3. **Two clauses of LLR-u6rq4s are unevidenced**, as the table above records.
   The second of them says the first-admission skip is unnecessary; that is an
   owner's ruling to make, booked.

   *Narrowed 2026-10-04 by review round 5: **one** clause is unevidenced now.
   "The trie the message verified into" was recorded as undischargeable on the
   argument that removals arrive only on the other receive path. The gated
   `revocation_of_another_member_is_committed_not_self_deleted` already
   disproved that. Round 5's test for the clause is RED under the sweep's
   mutation. The first-admission clause and its owner ruling stand.*
4. **Four clauses are compile-enforced rather than test-enforced.** LLR-f5kq88
   ("`check` does not move the mark") holds because `check` takes `&self`;
   LLR-z8fubr's distinctness clause holds because the variants are an enum;
   LLR-836z24 ("`import_join_request` stores nothing") holds because the
   function takes no `&mut self`; and LLR-65py3d (the chain seam is a trait
   object a substitute can stand in for) holds because `Box<dyn ChainOps>`
   compiles. A mutation falsifying any of them must change a signature or a
   type, which breaks the library and measures nothing. This is **stronger**
   than a test, not weaker, but it is not evidence of the same kind and is
   named as such. *The last two were added 2026-10-04: review round 2 found
   LLR-65py3d among six low-level requirements with no attestation row, and
   the honest answer for it is this entry rather than a row.*
5. **Coverage is not measured for org-node**, and is not measured by this
   change. No `coverage_command` exists for this unit; under class C statement
   and decision coverage are mandatory. Pre-existing, recorded in
   `docs/plans/2026-09-05-ratchet-setup.md`.
6. **Seven items have no abnormal-input case of their own**, and the ledger's
   table in `2026-10-03-decomposition.md` is the authoritative list: SDD-89es4z
   (a Persona handle org-members will not accept), SDD-b8tuv3 (an endpoint bind
   that fails), SDD-ueh4tm (a `ChainOps` whose calls fail), SDD-z85ux9 (which
   has no gated test of any kind, so no abnormal one either), SDD-rq6nv4 (see
   Gap 15), SDD-msb6xh, and — most materially — **SDD-rx2yvy**, where
   `admit_member` takes a malformed member or device key, or a handle already
   held, from a join request a stranger composes, and nothing tests either
   path. Only SDD-msb6xh is argued rather than owed: `build_update_calldata` is
   total over its argument types. SDD-pa6p7w left this list in round 2, which
   gave it a failing `ChainReader`.

   *Rewritten 2026-10-04 by review round 3. This entry still read "Four items"
   and still called SDD-rq6nv4 argued — the exact sentence round 2's finding-8
   convicted. Round 2's fix was applied to the ledger and to Gap 15 and not to
   this standing list two hundred lines above it, so the record contradicted
   itself and the ledger. The instance, not the class, for the third round
   running; the list now points at the ledger's table rather than restating
   it, so there is one place to correct next time.*
7. **The Networked arm of `admit_member` and `revoke_member` is outside this
   gate** (review round 1, finding-3). `bind_with_mode(Networked)` uses
   `presets::N0`, so a gated test would reach the public internet; closing it
   needs a `test-support` constructor taking an injected relay. LLR-jn5jeh was
   narrowed to its Loopback clause rather than left claiming both.
8. **`MockChainOps` is compiled unconditionally**, not behind `test-support`
   as `test_fixtures` is, so nothing in the type system stops a production
   caller constructing a node that verifies against a chain it controls.
   Assessed in the risk ledger and booked; `app` is the only consumer and its
   wiring is a single construction site.
9. **org-node has no high-level requirement for its administrator-side write
   path** (review round 1, finding-7). Fifteen low-level requirements that
   claimed to refine one are now `satisfies: derived` and assessed. Closing the
   gap is a `grill-requirements` pass of its own and is booked.
10. **No advisory-database scan** backs the SOUP inventory; neither `cargo-audit`
   nor `cargo-deny` is installed in this toolchain. Same gap as
   `on-chain-client`'s inventory.
11. **The robustness split is a hand count**, not a measurement: nothing in
   `.guardrails/scripts` reads robustness sidedness.
12. **Four problem reports are open**, against `problem_age_days: 30`, and
   they are named rather than aged here — review round 1 found this entry
   stale against the tree, round 1's fix wrote "25–26 days", and review round 2
   found that stale too, in the opposite direction (the gate reports 25, 25, 25
   and 24). **A day count in a document measures a tree that no longer exists;
   a fire date does not move.** PR-vt244s, PR-2dmjzj and PR-u4c2vp fire
   **2026-10-10**; PR-w88sr9 fires **2026-10-11**. The unit holds seven problem
   reports in all; the other three are resolved, PR-g7cfns by master `0f85cb9`
   while this change was open. This change addresses none of them and the limit
   is unchanged.

   *Extended 2026-10-04 at gate round 5. The unit now holds **nine** problem
   reports, **six** open. The four above are joined by PR-b9wab3, which this
   change opened (review round 3), and PR-hqwpg9, which arrived with the
   `24317e3` merge. Both were opened 2026-10-04 and fire **2026-11-04**. The
   gate reports open 6 against a limit of 10.*

   *Extended again by review round 5, which opened three more: PR-mdv38y,
   PR-8qsnhx and PR-xwek5e, all on 2026-10-04 and all firing **2026-11-04**.
   The unit now holds **twelve** problem reports, **nine** open, against
   `problem_open_max: 10`. **One more open report and the gate fails on
   backlog.** The four oldest fire on 2026-10-10 and 2026-10-11 as above, so
   the next change to org-node should close problem reports, not open them.*

   *And again by review round 6, which opened PR-322qst and folded two further
   defects into PR-mdv38y and PR-8qsnhx so as not to exceed the limit. Thirteen
   problem reports, **ten open, at `problem_open_max: 10`**. The gate passes at
   ten and fails at eleven, so this change can book nothing further, and the
   owner has ruled that the next org-node change closes reports.*

   *Dates corrected 2026-10-05 by review round 8. The two paragraphs above
   said the 2026-10-04 reports fire on 2026-11-03. `check-trace.sh` flags a
   report when its age is **more than** `problem_age_days`, so a report opened
   2026-10-04 is first flagged on 2026-11-04, as the 2026-09-09 reports are
   first flagged on 2026-10-10. That date holds for PR-322qst too.*
13. **The sweep's depth is not uniform** across items, as stated above.
14. **`ensure_endpoint` consulting `self.transport_mode`** is unevidenced:
   hard-coding `TransportMode::Loopback` there leaves the gate green, because
   `OrgService::new` already sets Loopback and no gated test may set the only
   other value. LLR-ecz9a6 is narrowed to the clause the gate can see. Closed
   by the same `test-support` injected-relay constructor as Gap 7's Networked
   arm, and booked with it.
15. **`build_dispatch_tx`'s three `MalformedCall` arms are untested**, and
   this change made the function reachable from a test through
   `org_node::test_support`. The robustness table used to argue SDD-rq6nv4 away
   on the ground that "this unit never constructs a malformed `Value`", which
   was true of the unit's own call graph and not of the function; round 2
   found both halves. The item is now listed as owed rather than argued.
16. **`import_join_request` storing nothing is true by signature, not by
   test**: it is an associated function taking no `&mut self`, so no mutation
   can make it write without changing the signature, and LLR-836z24 therefore
   has an assertion behind it rather than a red. That is the honest status —
   the compiler is the evidence, and this entry exists so the next reader is
   not told a mutation proved it.
17. **`apply_delta`'s error arm in `verify_envelope_against_chain` is
   unreachable at this gate**, and that is now the *whole* of what is
   unreachable rather than, as this entry first claimed, a convenient summary
   of it. **Rewritten 2026-10-04 after review round 3**, which measured that a
   `panic!()` immediately after the decode left the target passing: no input
   produced a decodable delta at all, so checks 5 to 8 were unreached and the
   `assert_eq!` guarding each accepted update could never run. The deepened
   target now carries a third shape — one byte of an **honest** delta
   encoding, overwritten at a fuzz-chosen offset with a fuzz-chosen value —
   and a `panic!()` at check 6 and again at check 8 both redden it. What
   remains unreached is `apply_delta` returning `Err` on a delta whose
   `base_root` does match: check 5 screens for that before the call, so the
   arm is defensive. Closing it needs a deterministic test offering a delta
   that applies to the right base root and still fails — a `DuplicateKey`,
   say — which needs org-members fixtures this unit does not have. Booked.
   LLR-hs7g6j now rests on checks 1 through 8 with that one arm excepted.

   *Corrected 2026-10-04 by review round 4. "That is now the whole of what is
   unreachable" was false. The target seeded its chain with the root
   **before** any delta, so check 8 compared a post-apply root against a
   pre-apply one and failed for every delta that changed the trie: no input
   could be accepted, by construction, and the `assert_eq!` on an accepted
   update could not run. Reaching check 8 is not passing it. The chain now
   holds the root the honest delta produces; a `panic!()` after check 8
   succeeds reddens the target in all three runs, and so does a check 8
   weakened to compare the candidate against its own root — that one through
   the `assert_eq!`, on the first input. The one unreached arm remains
   `apply_delta`'s `Err` behind a matching base root, as above.*
18. **The gate cannot tell a live fuzz target from a dead one.** A bolero
   target prints an iteration count and no `test result:` line, so it
   contributes nothing to the pass count and reads identically whether it
   exercises a whole function or a guard that never fires. **All three of this
   unit's targets were found to be reaching less than they were credited
   with**, in two passes: round 2's section above found two of them standing on
   guards that never fired, and review round 3 found that the third,
   `fuzz_first_admission_base`, decoded only the **empty** snapshot vector, and
   that the one round 2 had just deepened still reached no further than the
   decode. Every one of the five was found by a mutation and by nothing else.
   *Review round 4 added a sixth, of a kind the first five were not: a target
   that reached its decisive check and could never pass it, because its
   fixture made acceptance impossible. A reach probe proves a line is
   executed; it says nothing about which way the branch on that line goes.*
   **Nothing in `.guardrails` measures fuzz-target reachability, and after this
   change nothing still does.** What would close it is a coverage run over the
   fuzz targets, which is the same `coverage_command` gap as Gap 5 and is
   booked with it.

   Until then, the discipline is the one this change learned three times: **a
   fuzz target's evidence is worth exactly what a mutation placed behind its
   deepest guard says it is**, and "the call is unguarded" — which is what
   round 2 said when it called `fuzz_first_admission_base` sound — is an
   argument about the call, not a measurement of the reach.
19. **`OrgNodeError::Chain(String)` is a catch-all with fifty construction
   sites.** Review round 3 measured them. Its `Display` is
   `"chain read failed: {0}"`, so a wrong store passphrase, a malformed invite
   blob, a missing Persona, a Loopback revocation offered no peer address and
   an iroh send failure all surface to a caller as chain read failures.
   LLR-z8fubr and SDD-swtd3w claimed "one typed variant per rejection path" of
   the *unit*; both are narrowed to verify-against-chain, which is what
   `error.rs`'s own doc comment always said and what the ten declared variants
   actually give. This is a real defect in the production error surface rather
   than only in its description — a caller cannot act differently on "the chain
   did not answer" and "your passphrase is wrong" — and it is **not closed
   here**, because collapsing fifty sites into typed variants is a change to
   that surface with its own review. Booked.
20. **The shipped app appears unable to complete a joiner's first admission**,
   noted by review round 6 from reading and confirmed by the author from
   reading, not by a test. The app's only receive loop is
   `receive_and_self_delete_if_revoked`, which refuses with `OrgNotOnChain`
   when no record exists. It is a defect in the `app` unit, outside this
   change, and is **booked for the next app change** as an app problem report,
   by the owner's ruling of 2026-10-04. It is recorded here so that it is not
   lost in the meantime.
21. **`receive_and_self_delete_if_revoked` names a missing local record
   `OrgNotOnChain`**, although the chain holds the Organisation, so no caller
   can tell the two apart. Found by review round 7 as a defect outside this
   change. It is **not booked**, because org-node is at its limit of ten open
   problem reports, and it is recorded here for the fix change. LLR-379hnv
   states the misnomer without endorsing it.

## Appendix — the per-mutation attestations

*Added 2026-10-04 by review round 1, which found that both the decomposition's
Evidence section and the plan's "Red → green attestations" table pointed at
per-mutation records that existed nowhere in the tree: the record carried only
aggregate counts, and the plan's table still read "(pending)". This appendix is
that record.*

Every row is one run. The mutation's label names the clause it falsifies —
`<llr-suffix>-<clause>`, so `wx3php-strictly-greater` is LLR-wx3php's
"strictly greater" clause. Where a label appears twice, the second run is a
re-probe after the clause's evidence or wording was changed; the later verdict
is the operative one. The "named test(s)" column is truncated to the first two
plus a count where a mutation reddened more.

Each run restored its file from a pristine copy afterwards and proved it
byte-identical by SHA-256; the driver aborts the whole sweep if a restore ever
fails, and none did.

| # | Mutation | Verdict | Named test(s) that failed |
|---|---|---|---|
| 1 | `wx3php-strictly-greater` | RED | rejects_equal_and_lower_seq, rejects_stale_seq, +1 more |
| 2 | `wx3php-refusal-variant` | RED | rejects_equal_and_lower_seq, rejects_stale_seq, +1 more |
| 3 | `wx3php-carries-offered` | RED | rejects_equal_and_lower_seq |
| 4 | `wx3php-carries-mark` | RED | rejects_equal_and_lower_seq |
| 5 | `uc7cej-moves-forward` | RED | advance_moves_high_water_mark_forward_only,happy_path_commits_when_root_matches_chain |
| 6 | `uc7cej-only-forward` | RED | advance_moves_high_water_mark_forward_only |
| 7 | `duwz79-starts-at-mark` | RED | check_does_not_advance_the_mark, rejects_equal_and_lower_seq, +3 more |
| 8 | `duwz79-reports-mark` | RED | advance_moves_high_water_mark_forward_only, check_does_not_advance_the_mark, +2 more |
| 9 | `e7s4ye-order` | RED | rejects_stale_seq_before_decoding_delta,the_signed_transcript_is_org_then_seq_little_endian_then_delta |
| 10 | `e7s4ye-endianness` | RED | rejects_stale_seq_before_decoding_delta, the_sequence_number_is_little_endian_in_the_transcript, +1 more |
| 11 | `e7s4ye-nothing-else` | RED | rejects_stale_seq_before_decoding_delta,the_signed_transcript_is_org_then_seq_little_endian_then_delta |
| 12 | `ybn5pr-org-bound` | RED | altering_the_organisation_identifier_breaks_the_signature, rejects_stale_seq_before_decoding_delta, +1 more |
| 13 | `cs4mpb-seq-bound` | RED | altering_the_sequence_number_breaks_the_signature, rejects_stale_seq_before_decoding_delta, +1 more |
| 14 | `9sknpa-delta-bound` | RED | altering_the_delta_bytes_breaks_the_signature, rejects_stale_seq_before_decoding_delta, +1 more |
| 15 | `p8uu47-signs-what-it-sends` | RED | a_revocation_relayed_by_a_non_member_is_still_acted_on, a_successful_verification_leaves_the_callers_trie_untouched, +16 more |
| 16 | `v2y6sw-error-variant` | RED | decode_delta_refuses_bytes_that_are_not_a_delta |
| 17 | `v2y6sw-no-panic` | RED | decode_delta_refuses_bytes_that_are_not_a_delta |
| 18 | `pzde8b-checks-the-key` | RED | a_rejected_verification_leaves_the_callers_trie_untouched, altering_the_delta_bytes_breaks_the_signature, +6 more |
| 19 | `4fbuy8-org-checked` | RED | rejects_wrong_org_before_decoding_delta,rejects_wrong_org_id |
| 20 | `4fbuy8-org-variant` | RED | rejects_wrong_org_before_decoding_delta,rejects_wrong_org_id |
| 21 | `4fbuy8-org-is-first` | GREEN | no test failed |
| 22 | `mcdh85-sig-checked` | RED | a_rejected_verification_leaves_the_callers_trie_untouched, rejects_bad_signature, +2 more |
| 23 | `mcdh85-sig-variant` | RED | rejects_bad_signature, rejects_bad_signature_before_decoding_delta, +1 more |
| 24 | `mcdh85-sig-before-decode` | RED | rejects_bad_signature_before_decoding_delta,rejects_stale_seq_before_decoding_delta |
| 25 | `xpbkp5-seq-checked` | RED | rejects_stale_seq,rejects_stale_seq_before_decoding_delta |
| 26 | `xpbkp5-seq-before-decode` | RED | rejects_stale_seq_before_decoding_delta |
| 27 | `992mbf-base-checked` | RED | rejects_a_delta_whose_base_root_is_not_the_local_root |
| 28 | `992mbf-base-variant` | RED | rejects_a_delta_whose_base_root_is_not_the_local_root |
| 29 | `8m99q2-absent-variant` | RED | rejects_when_org_absent_from_chain |
| 30 | `8m99q2-absent-vs-failure` | RED | rejects_when_org_absent_from_chain |
| 31 | `vf5mjx-epoch-checked` | RED | rejects_stale_epoch |
| 32 | `vf5mjx-epoch-strict` | RED | rejects_stale_epoch |
| 33 | `vf5mjx-epoch-carries` | GREEN | no test failed |
| 34 | `8n95rf-root-checked` | LIBBROKEN | library stopped compiling — measures nothing, rewrite |
| 35 | `8n95rf-root-variant` | RED | rejects_root_mismatch_when_chain_root_differs |
| 36 | `d6kvbx-advance-happens` | RED | happy_path_commits_when_root_matches_chain |
| 37 | `d6kvbx-advance-before-root` | GREEN | no test failed |
| 38 | `8hwqru-returns-new-trie` | RED | a_revocation_relayed_by_a_non_member_is_still_acted_on, a_successful_verification_leaves_the_callers_trie_untouched, +8 more |
| 39 | `ygn78w-secret-is-the-devices` | RED | a_message_for_an_organisation_absent_from_the_chain_is_refused, a_revocation_relayed_by_a_non_member_is_still_acted_on, +9 more |
| 40 | `ygn78w-field-is-the-devices` | RED | a_revocation_relayed_by_a_non_member_is_still_acted_on, endpoint_id_equals_device_key, +1 more |
| 41 | `v873fx-from-the-handshake` | RED | a_revocation_relayed_by_a_non_member_is_still_acted_on, delivers_and_verifies_admit_over_iroh, +7 more |
| 42 | `wwunf4-ipv6-bind-present` | GREEN | no test failed |
| 43 | `wwunf4-ipv4-bind-present` | RED | loopback_mode_holds_under_repeated_and_colliding_binds |
| 44 | `wwunf4-ipv4-is-loopback` | RED | loopback_mode_binds_and_advertises_loopback_only, loopback_mode_holds_under_repeated_and_colliding_binds, +1 more |
| 45 | `wwunf4-ipv6-is-loopback` | RED | loopback_mode_binds_and_advertises_loopback_only, loopback_mode_holds_under_repeated_and_colliding_binds, +1 more |
| 46 | `wx77j5-reports-bound-sockets` | RED | a_stream_longer_than_the_read_bound_is_refused_rather_than_buffered, loopback_mode_binds_and_advertises_loopback_only, +1 more |
| 47 | `wx77j5-reports-the-id` | LIBBROKEN | library stopped compiling — measures nothing, rewrite |
| 48 | `k2y6nn-read-is-bounded` | RED | a_stream_longer_than_the_read_bound_is_refused_rather_than_buffered |
| 49 | `6adc99-ipv6-optional` | GREEN | no test failed |
| 50 | `wusj89-no-cleartext-header` | RED | a_new_persona_is_proposed_with_an_id_derived_from_its_member_key, five_stories_full_e2e, +1 more |
| 51 | `wusj89-nonce-then-ciphertext` | RED | a_new_persona_is_proposed_with_an_id_derived_from_its_member_key, five_stories_full_e2e, +1 more |
| 52 | `8mfjey-fresh-nonce` | GREEN | no test failed |
| 53 | `t4u66w-passphrase-binds` | RED | round_trips_encrypted_through_disk,wrong_passphrase_yields_error_not_data |
| 54 | `q5n28x-short-file-refused` | RED | a_file_shorter_than_the_nonce_is_refused |
| 55 | `s78sh7-written-as-ciphertext` | RED | a_new_persona_is_proposed_with_an_id_derived_from_its_member_key, five_stories_full_e2e, +3 more |
| 56 | `j83kc8-invite-check-present` | RED | first_admission_from_a_device_other_than_the_invites_admin_is_rejected |
| 57 | `j83kc8-invite-check-sense` | RED | a_revocation_relayed_by_a_non_member_is_still_acted_on, first_admission_from_a_device_other_than_the_invites_admin_is_rejected, +5 more |
| 58 | `j83kc8-leaves-record-unchanged` | GREEN | no test failed |
| 59 | `u6rq4s-sender-check-present` | RED | update_relayed_by_a_non_member_after_admission_is_rejected |
| 60 | `u6rq4s-sender-check-sense` | RED | revocation_of_another_member_is_committed_not_self_deleted, update_from_the_admin_after_admission_is_committed, +1 more |
| 61 | `u6rq4s-checked-against-new-trie` | GREEN | no test failed |
| 62 | `u6rq4s-skipped-on-first-admission` | GREEN | no test failed |
| 63 | `37cj3n-author-from-the-chain` | RED | a_revocation_relayed_by_a_non_member_is_still_acted_on, five_stories_full_e2e, +4 more |
| 64 | `3wb7th-absent-variant` | RED | a_message_for_an_organisation_absent_from_the_chain_is_refused |
| 65 | `cja9zv-store-is-saved` | GREEN | no test failed |
| 66 | `cja9zv-epoch-written` | RED | update_from_the_admin_after_admission_is_committed |
| 67 | `cja9zv-seq-written` | GREEN | no test failed |
| 68 | `q8emds-invite-consumed` | GREEN | no test failed |
| 69 | `6p4pj2-record-deleted` | RED | a_revocation_relayed_by_a_non_member_is_still_acted_on,five_stories_full_e2e |
| 70 | `6p4pj2-persona-revoked` | RED | a_revocation_relayed_by_a_non_member_is_still_acted_on,five_stories_full_e2e |
| 71 | `jsx922-still-present-updates` | RED | revocation_of_another_member_is_committed_not_self_deleted |
| 72 | `jsx922-presence-test` | RED | revocation_of_another_member_is_committed_not_self_deleted |
| 73 | `vw2jn6-decides-on-verified` | RED | a_revocation_relayed_by_a_non_member_is_still_acted_on,five_stories_full_e2e |
| 74 | `3q63zv-no-sender-check` | RED | a_revocation_relayed_by_a_non_member_is_still_acted_on |
| 75 | `7gnrnz` | GREEN | no test failed |
| 76 | `gu6u53` | RED | org_id_debug_is_the_twenty_bytes_as_forty_lowercase_hex_digits |
| 77 | `z8fubr` | LIBBROKEN | library stopped compiling — measures nothing, rewrite |
| 78 | `7cgg8a` | RED | a_provider_error_is_carried_into_the_trie_variant_without_loss |
| 79 | `e58j8m` | RED | a_committed_admission_reaches_the_disk_and_consumes_the_invite, a_keypair_rebuilt_from_its_seed_signs_identically, +13 more |
| 80 | `ctzkv7` | RED | a_revocation_relayed_by_a_non_member_is_still_acted_on, delivers_and_verifies_admit_over_iroh, +6 more |
| 81 | `na7p4w` | RED | a_committed_admission_reaches_the_disk_and_consumes_the_invite, a_revocation_relayed_by_a_non_member_is_still_acted_on, +19 more |
| 82 | `9fvb3y` | RED | a_rejected_verification_leaves_the_callers_trie_untouched, a_signature_does_not_verify_over_a_different_message, +8 more |
| 83 | `rm9x4z` | RED | a_stale_epoch_names_the_chain_epoch_and_the_committed_epoch_the_right_way_round, a_successful_verification_leaves_the_callers_trie_untouched, +6 more |
| 84 | `g9vmbx` | RED | an_invite_round_trips_carrying_every_field |
| 85 | `kkj64b` | RED | a_committed_admission_reaches_the_disk_and_consumes_the_invite, a_join_request_round_trips_carrying_every_field, +8 more |
| 86 | `8qxwst` | GREEN | no test failed |
| 87 | `tcft2r` | RED | an_invite_does_not_decode_as_a_join_request,valid_base64_that_is_not_a_blob_is_refused |
| 88 | `rv4vux` | RED | update_calldata_is_exactly_one_hundred_bytes_in_the_declared_order |
| 89 | `txqmz4` | RED | the_expected_epoch_occupies_the_low_sixteen_bytes_big_endian |
| 90 | `66h529` | RED | the_update_selector_is_the_declared_four_bytes |
| 91 | `463d89` | RED | the_derived_account_does_not_depend_on_signer_order |
| 92 | `8m3bwj-threshold` | RED | the_derived_account_depends_on_the_threshold |
| 93 | `f74xwb` | RED | dispatch_is_direct_without_other_signatories_and_wrapped_with_them |
| 94 | `hg3xzf` | LIBBROKEN | library stopped compiling — measures nothing, rewrite |
| 95 | `tev8h8` | RED | a_new_persona_is_proposed_with_an_id_derived_from_its_member_key |
| 96 | `s7yu4k` | RED | a_new_persona_is_proposed_with_an_id_derived_from_its_member_key |
| 97 | `v82xds` | RED | a_new_persona_is_proposed_with_an_id_derived_from_its_member_key,first_admission_from_a_device_other_than_the_invites_admin_is_rejected |
| 98 | `6zjzn2` | RED | a_committed_admission_reaches_the_disk_and_consumes_the_invite, a_message_for_an_organisation_absent_from_the_chain_is_refused, +7 more |
| 99 | `cns6q6` | RED | *(the test was later renamed; see the round-2 block)* |
| 100 | `ecz9a6` | RED | *(the test was later renamed; see the round-2 block)* |
| 101 | `fa7jt8` | RED | frame_round_trips |
| 102 | `er2x8n` | RED | a_committed_admission_reaches_the_disk_and_consumes_the_invite, a_message_for_an_organisation_absent_from_the_chain_is_refused, +10 more |
| 103 | `sc6zuh` | RED | oversize_message_is_rejected_on_encode |
| 104 | `8kh3zf` | RED | oversize_body_is_rejected |
| 105 | `pkruy8` | RED | a_body_of_exactly_the_bound_is_not_refused_for_its_size,a_body_that_is_not_an_encoded_message_is_refused |
| 106 | `8mfjey-fresh-nonce` | RED | each_save_draws_a_fresh_nonce |
| 107 | `wx77j5-reports-the-id` | RED | a_stream_longer_than_the_read_bound_is_refused_rather_than_buffered |
| 108 | `4fbuy8-org-before-decode` | RED | rejects_wrong_org_before_decoding_delta |
| 109 | `vf5mjx-epoch-carries` | RED | a_stale_epoch_names_the_chain_epoch_and_the_committed_epoch_the_right_way_round |
| 110 | `8n95rf-root-decisive` | LIBBROKEN | library stopped compiling — measures nothing, rewrite |
| 111 | `d6kvbx-mark-is-the-envelopes` | RED | happy_path_commits_when_root_matches_chain |
| 112 | `8n95rf-root-decisive` | RED | rejects_root_mismatch_when_chain_root_differs |
| 113 | `7gnrnz` | RED | delivers_and_verifies_admit_over_iroh, org_id_debug_is_the_twenty_bytes_as_forty_lowercase_hex_digits, +2 more |
| 114 | `8qxwst` | RED | a_string_that_is_not_base64_is_refused |
| 115 | `hg3xzf` | RED | clones_of_the_mock_chain_share_one_state,and ten more |
| 116 | `j83kc8-leaves-record-unchanged` | RED | first_admission_from_a_device_other_than_the_invites_admin_is_rejected |
| 117 | `cja9zv-store-is-saved` | RED | a_committed_admission_reaches_the_disk_and_consumes_the_invite |
| 118 | `cja9zv-seq-written` | GREEN | no test failed |
| 119 | `q8emds-invite-consumed` | RED | a_committed_admission_reaches_the_disk_and_consumes_the_invite |
| 120 | `cja9zv-seq-written-update` | RED | update_from_the_admin_after_admission_is_committed |
| 121 | `cja9zv-seq-written-first` | RED | a_committed_admission_reaches_the_disk_and_consumes_the_invite |
| 122 | `992mbf-base-checked` | RED | rejects_a_delta_whose_base_root_is_not_the_local_root |
| 123 | `992mbf-base-variant` | RED | rejects_a_delta_whose_base_root_is_not_the_local_root |
| 124 | `f1-admit-save` | GREEN | no test failed — review round 1 finding-1, before the fix |
| 125 | `f1-genesis-save` | GREEN | no test failed — review round 1 finding-1, before the fix |
| 126 | `f3-networked-arm` | GREEN | no test failed — review round 1 finding-3, standing gap |
| 127 | `f1-admit-save` | RED | an_admission_reaches_the_administrators_disk |
| 128 | `f1-genesis-save` | RED | creating_an_organisation_advances_the_chain_and_activates_the_persona |
| 129 | `f2-send-failure-is-fatal` | RED | a_failed_push_leaves_the_administrators_record_where_it_was |

### Review round 2's block — 2026-10-04

Three groups, in the order they were run. Labels are prefixed `r2-` so they
cannot be confused with the sweep's own; the conventions above apply unchanged,
including that a repeated label's later verdict is the operative one.

**Group 1 — reproducing the reviewer's eight greens.** Every finding round 2
raised in the `code` category was re-measured here before being accepted. All
eight reproduced.

| # | Mutation | Verdict | Named test(s) that failed |
|---|---|---|---|
| 130 | `r2-rev-save` | GREEN | no test failed |
| 131 | `r2-selfdel-notrevoked-save` | GREEN | no test failed |
| 132 | `r2-selfdel-deleted-save` | GREEN | no test failed |
| 133 | `r2-import-invite-save` | GREEN | no test failed |
| 134 | `r2-create-persona-save` | RED | a_new_persona_is_proposed_with_an_id_derived_from_its_member_key |
| 135 | `r2-ecz9a6-mode-constant` | GREEN | no test failed |
| 136 | `r2-cns6q6-first-persona` | GREEN | no test failed |
| 137 | `r2-8m99q2-chain-variant` | GREEN | no test failed |
| 138 | `r2-ghja3x-seq-plus-two` | GREEN | no test failed |
| 139 | `r2-6zjzn2-no-guard` | RED | twelve tests, **none of them** `the_endpoint_is_bound_once_…`, the one annotated to LLR-6zjzn2 |
| 140 | `r2-jn5jeh-loopback-by-id` | RED | twelve tests, **neither of them** the two annotated to LLR-jn5jeh |

Row 134 is not a finding. `create_persona` is the eighth of the unit's eight
`store.save` sites and the reviewer named only four; it was probed to settle
the **class** rather than the four, and it was already evidenced. Rows 139 and
140 are RED and are findings anyway: a mutation that reddens twelve tests and
not the test written for the requirement is finding-6's whole point.

**Group 2 — re-probes after the fixes.**

| # | Mutation | Verdict | Named test(s) that failed |
|---|---|---|---|
| 141 | `r2-rev-save` | RED | a_revocation_reaches_the_administrators_disk |
| 142 | `r2-selfdel-notrevoked-save` | RED | an_update_that_does_not_revoke_us_reaches_the_disk |
| 143 | `r2-selfdel-deleted-save` | RED | a_self_delete_reaches_the_revoked_nodes_disk |
| 144 | `r2-import-invite-save` | RED | an_imported_invite_reaches_the_joiners_disk |
| 145 | `r2-cns6q6-first-persona` | RED | the_endpoint_binds_the_named_personas_device_not_the_first_personas |
| 146 | `r2-8m99q2-chain-variant` | RED | a_chain_read_that_fails_is_refused_as_chain_not_as_absence |
| 147 | `r2-ghja3x-seq-plus-two` | RED | the_admission_envelope_carries_the_mark_one_past_the_records_last |
| 148 | `r2-6zjzn2-no-guard` | RED | the_endpoint_is_bound_once_and_the_same_one_is_returned_after, a_revocation_reaches_the_administrators_disk, +14 more |
| 149 | `r2-jn5jeh-loopback-by-id` | RED | in_loopback_mode_the_joiner_is_dialled_at_the_full_address, the_admission_envelope_carries_the_mark_one_past_the_records_last, +15 more |
| 150 | `r2-ecz9a6-mode-constant` | **GREEN** | **still green, deliberately** — the requirement is narrowed to the clause the gate can see and the rest is Gap 14 |

Row 150 is the one green this change ships without closing, and it is the
second of its kind: round 1 made the identical disposition for LLR-jn5jeh's
Networked clause. Both are closed by the same booked constructor.

**Group 3 — one mutation per low-level requirement added by round 2.**

| # | Mutation | Verdict | Named test(s) that failed |
|---|---|---|---|
| 151 | `r2-6dc598-chain-advances` | RED | a_failed_revocation_push_leaves_the_administrators_record_where_it_was, a_revocation_reaches_the_administrators_disk, +5 more |
| 152 | `r2-tax3pm-signs-with-device-key` | RED | a_revocation_reaches_the_administrators_disk, a_revocation_relayed_by_a_non_member_is_still_acted_on, +3 more |
| 153 | `r2-zj88e6-invite-device-is-member` | RED | the_out_of_band_blobs_carry_the_keys_their_holders_are_pinned_by, a_committed_admission_reaches_the_disk_and_consumes_the_invite, +12 more |
| 154 | `r2-9zfnmb-append-not-replace` | RED | an_imported_invite_reaches_the_joiners_disk |
| 155 | `r2-437fvx-join-request-same-key` | RED | the_out_of_band_blobs_carry_the_keys_their_holders_are_pinned_by, a_committed_admission_reaches_the_disk_and_consumes_the_invite, +18 more |
| 156 | `r2-ag9mgm-id-from-key` | RED | member_ids_are_not_derived_from_keys, readmission_with_same_keys_gets_a_fresh_member_id |
| 157 | `r2-rjg3m2-id-from-key` | RED | member_ids_are_not_derived_from_keys, same_persona_founding_two_organisations_gets_two_admin_ids |
| 158 | `r2-j6j95z-reconstruct` | RED | first_admission_without_a_record_snapshot_is_refused |

LLR-qg9utu is row 141 — deleting `revoke_member`'s `save` is its mutation and
LLR-6dc598's ordering is row 151, so the three revocation requirements are
covered by rows 141, 151 and 152 between them. **LLR-836z24 has no row**, and
that is stated rather than hidden: `import_join_request` takes no `&mut self`,
so no mutation can make it write without changing its signature. The compiler
is its evidence.

**Group 4 — finding-9's six unattested low-level requirements.**

Round 1 wrote the appendix above to back the claim that each low-level
requirement is discharged by a red. Round 2 checked that claim against the
appendix and found six items with no row at all, and one row filed under the
wrong item's ID (row 92, relabelled). These are the six.

| # | Mutation | Verdict | Named test(s) that failed |
|---|---|---|---|
| 159 | `r2-rb8r65-chain-at-next-epoch` | RED | a_committed_admission_reaches_the_disk_and_consumes_the_invite, a_failed_push_leaves_the_administrators_record_where_it_was, +17 more |
| 160 | `r2-bg3vsw-post-add-snapshots` | RED | a_committed_admission_reaches_the_disk_and_consumes_the_invite, a_failed_revocation_push_leaves_the_administrators_record_where_it_was, +11 more |
| 161 | `r2-6qmq2g-verify-not-fatal` | RED | unverified_revocation_leaves_the_record_in_place |
| 162 | `r2-hs7g6j-panics-on-apply` | GREEN | no test failed — and the reason is the finding below |
| 163 | `r2-hs7g6j-panic-at-entry` | GREEN | no test failed — 201 442 iterations, the target never called the function |
| 164 | `r2-hs7g6j-panic-at-entry` (after the target was deepened) | RED | fuzz_verify_against_chain |
| 165 | `r2-hs7g6j-decode-unwrap` (deepened) | RED | fuzz_verify_against_chain |
| 166 | `r2-hs7g6j-panics-on-apply` (deepened) | GREEN | still unreachable: `apply_delta` needs a 32-byte base-root match. Gap 17 |
| 167 | `r2-v2y6sw-decode-delta-panic` | GREEN | no test failed — `fuzz_envelope_decode` never parsed an envelope either |
| 168 | `r2-v2y6sw-decode-delta-panic` (after the target was deepened) | RED | fuzz_envelope_decode |

LLR-ghja3x is row 147 and LLR-65py3d has no row for the same reason LLR-836z24
has none: `Box<dyn ChainOps>` compiling is the property, and a mutation that
falsified it would be a signature change. **That leaves LLR-f5kq88**, which has
no row and no type-system argument either; it is Gap 4 and has been since the
sweep.

### The count

| Block | Runs | RED | GREEN | LIBBROKEN |
|---|---|---|---|---|
| The author's sweep | 123 | 103 | 15 | 5 |
| Review round 1 | 6 | 3 | 3 | — |
| Review round 2 and its fixes | 39 | 26 | 13 | — |
| Review round 3 and its fixes | 9 | 5 | 4 | — |
| Review round 4 and its fixes | 44 | 33 | 11 | — |
| Review round 5 and its fixes | 36 | 28 | 8 | — |
| Review round 6's remedies | 5 | 5 | 0 | — |
| Review round 7's remedies | 6 | 6 | 0 | — |
| Review round 8's remedies | 15 | 15 | 0 | — |
| Review round 9's remedies | 3 | 3 | 0 | — |
| **Total** | **286** | **227** | **54** | **5** |

*Corrected 2026-10-04 by review round 3, which counted the rows. Round 1's
block is rows 124–129: three greens found, then the same three mutations
re-probed RED after the fixes — 3 RED and 3 GREEN, not 6 RED. The prose two
paragraphs below had it right ("Round 1 wrote 3 and found 3") while this table,
whose entire subject is how many clauses were found resting on nothing,
contradicted it. Every figure above is now a count of the rows rather than a
number carried over from a summary.*

The greens are what this appendix is for.

| Found by | Clauses resting on nothing |
|---|---|
| the author's sweep (123 mutations) | 8 |
| review round 1 (3 mutations) | 3 |
| review round 2 (20 mutations) | 8 |
| the author, writing round 2's missing attestation rows | 2 |
| review round 3 (its own mutations and reach probes) | 7 |
| review round 4 (eleven receive-side lookups, and acceptance) | 12 |
| review round 5 (five green mutations; three clauses false of the code) | 8 |
| review round 6 (three green root-hash writes; one misfiled function) | 4 |
| review round 7 (six green mutations; two LLRs contradicted by older reports) | 8 |
| review round 8 (eight unstated behaviours, two partly falsifiable clauses, two mis-traces, one LLR contradicted by a report, three risk-file claims false of the code) | 16 |
| review round 9 (one clause traced elsewhere, two clauses with no test) | 3 |
| **total** | **79** |

**Eighteen of the twenty-eight were found by someone who had not written
them**, and every one of those eighteen sat beside something the author had
already found and fixed — the other arm of a branch, the other side of a seam,
the other half of an interface, the next target along. A sweep is evidence
about its own search, and the measurement of this one, over three independent
rounds, is that it reliably stopped at the instance.

*Extended 2026-10-04 by review round 4: twelve more, so **thirty of forty** were
found by someone who had not written them. Round 4's are the receive side of
the seam round 3 closed on the sending side, and an acceptance branch round 3
had shown was reached but never shown was passable. The twelve are not a count of
GREEN rows. They are the eleven lookups the reviewer measured green that a
requirement now states, plus finding-2's acceptance branch, which appears in
the fuzz table. The eleven GREEN rows in the block are four reproductions,
three probes taken before the third test, and the four runs of the designed
absence.*

*Extended again by review round 5: eight more, so **thirty-eight of
forty-eight**. Five are mutations the gate did not see: the u6rq4s clause, the
sequence mark on the self-delete update, the administrator key, the secret
overwrite and the administrator exclusion. Three are clauses that were
**false of the code**, not unevidenced: LLR-y2v8v2's Persona clause,
LLR-jwhzh3's last clause and LLR-cns6q6. They were found by probe tests, not
mutations, because no mutation makes a false statement true. That is a kind of
finding a mutation sweep cannot produce at all.*

**The two in the fourth row are worth reading twice**, because they were found
neither by the sweep nor by an independent reader but by *writing the
attestation rows round 2 said were missing*. One of them is that
`fuzz_verify_against_chain` had never once called
`verify_envelope_against_chain`. The appendix round 1 demanded was not
bookkeeping; filling it in was the measurement — and round 3 then found that
the same filling-in had been done too shallowly twice more.

### Three runs in this block were discarded before they were recorded

Stated because a sweep that does not say how it policed itself is asking to be
believed rather than checked.

1. **Two sweeps ran concurrently for one cycle.** The second was launched while
   the first still had two mutations to go; both restore `org-node/src` from
   the same pristine copy and mutate the same file, so the overlapping run was
   measuring a tree neither of them had written. It was stopped and the
   affected mutation, `r2-rjg3m2-id-from-key`, was re-run alone. Both verdicts
   were RED; only the clean one is row 157.
2. **A test file was edited mid-sweep.** `r2-hs7g6j-panics-on-bad-delta`
   reddened on `in_loopback_mode_the_joiner_is_dialled_at_the_full_address` —
   a test that has nothing to do with `apply_delta` — because that test was
   being rewritten while the run was in flight. Discarded and re-run against a
   quiescent tree; row 162 is the re-run.
3. **Two mutations silently matched nothing**, and the driver refused them
   rather than scoring them. `r2-bg3vsw-post-add-snapshots` and
   `r2-6qmq2g-verify-not-fatal` were first written with `!` as the `s///`
   delimiter, and their replacements contain `vec![` and `!=`; perl ended the
   expression early and the file was left unchanged. **Both would have been
   recorded as GREEN** — an unfalsifiable clause — by a driver that did not
   check. The NOCHANGE guard is why this appendix contains no phantom greens,
   and this is the second time in this change it has caught one.

The first two break the same rule: **a mutation measures the tree it was
applied to, and nothing else may be touching that tree.** The driver proves the
mutated file is byte-identical after every run, which catches a bad restore; it
cannot catch another process, or an author, changing a *different* file in the
same checkout. That is a real limit of this harness and it is written down
rather than left for the next person to rediscover.

### Review round 3's block — 2026-10-04

Labels are prefixed `r3-`. Two groups: the reviewer's findings reproduced, then
the same mutations re-probed after the fixes. The fuzz-reach probes are
reported separately below the table, because they are run against a single
target rather than the whole cargo line and a row with "no test failed" in it
would understate what they measured.

| # | Mutation | Verdict | Named test(s) that failed |
|---|---|---|---|
| 169 | `r3-find-org-ignores-id` | GREEN | no test failed |
| 170 | `r3-find-org-mut-ignores-id` | GREEN | no test failed |
| 171 | `r3-admin-persona-ignores-org` | GREEN | no test failed |
| 172 | `r3-update-persona-status-ignores-id` | GREEN | no test failed |
| 173 | `r3-revoke-loopback-by-id` | RED | a_revocation_reaches_the_administrators_disk, a_revocation_relayed_by_a_non_member_is_still_acted_on, +4 more |
| 174 | `r3-find-org-ignores-id` | RED | a_second_organisation_is_admitted_into_without_touching_the_first |
| 175 | `r3-find-org-mut-ignores-id` | RED | a_second_organisation_is_admitted_into_without_touching_the_first |
| 176 | `r3-admin-persona-ignores-org` | RED | a_second_organisation_is_admitted_into_without_touching_the_first |
| 177 | `r3-update-persona-status-ignores-id` | RED | a_second_organisation_is_admitted_into_without_touching_the_first |

Row 173 is RED and is a finding anyway, for the same reason rows 139 and 140
were: the clause it falsifies is stated in no low-level requirement, so what
the gate was protecting was unwritten. LLR-pw369n states it.

**The fuzz-reach probes.** Each is a `panic!()` placed at a named point in the
production code, with the single bolero target run alone; a target that passes
is a target no input reaches that point in.

| Probe | Target | Before | After the target was deepened |
|---|---|---|---|
| `panic!()` in `trie_from_snapshots`'s per-snapshot closure | `fuzz_first_admission_base` | **passes**, 136 796 iterations | **fails** |
| `panic!()` inside the `"bad member key"` arm | `fuzz_first_admission_base` | (unreachable — the closure above was never entered) | **fails** |
| `panic!()` immediately after `envelope.decode_delta()?` | `fuzz_verify_against_chain` | **passes**, 2 479 iterations | **fails** |
| `panic!()` at check 6, before `local_trie.apply_delta` | `fuzz_verify_against_chain` | (unreachable) | **fails** |
| `panic!()` at check 8, the decisive root match | `fuzz_verify_against_chain` | (unreachable) | **fails** |

The iteration rates are the corroborating measurement, since a target that
reaches nothing is fast: `fuzz_verify_against_chain` ran at 201 440/s before
round 2 deepened it, 2 474/s after, and **873/s** after round 3 added the
perturbed-delta shape. `fuzz_first_admission_base` ran at 136 773/s before and
2 787/s after. Three rounds, three times the same lesson, and each time the
number that would have given it away was printed on every run and read by
nobody.

### Review round 4's block — 2026-10-04

Labels are prefixed `r4-`, `r4f-` and `r4m-`. Five groups, in order:

- rows 178–181: the reviewer's four worst lookups reproduced, before any fix;
- rows 182–185: the same four re-probed after the first two two-Organisation
  tests;
- rows 186–193: the other eight sites probed to settle the class, which left
  two GREEN that should not have been (190, 192) beside the designed absence
  (193);
- rows 194–195: those two re-probed after the third test;
- rows 196–208 (`r4f-`) and 209–221 (`r4m-`): **all thirteen sites**, line
  anchored, on the tree committed as `7aafcd1` and again after `master`
  (`24317e3`) was merged in. These are the operative rows.

Rows 197, 203, 210 and 216 are the `.first()` → `.last()` endpoint-binding
mutations, GREEN by design: every receive test injects its endpoint, so the
lookup is never reached. This is the absence the decomposition declares, not a
gap the round left open. Row 215 names one test more than row 202:
after the merge, `five_stories_full_e2e` also catches a Persona marked Active
by the wrong lookup ("B's persona must be Active", left `Proposed`). `master`
edited `service_stories.rs` in `24317e3`. *Withdrawn 2026-10-04 by review
round 5. That sentence implied a cause that was never measured. The fourth
failure did not recur in three later runs of the same mutation (rows 228, 256
and 257). It was a one-off, and the row is operative for its three named
tests only.*

| # | Mutation | Verdict | Named test(s) that failed |
|---|---|---|---|
| 178 | `r4-recv-commit-any-org` | GREEN | no test failed |
| 179 | `r4-selfdel-retain-none` | GREEN | no test failed |
| 180 | `r4-recv-pending-invite-any-org` | GREEN | no test failed |
| 181 | `r4-recv-local-trie-any-org` | GREEN | no test failed |
| 182 | `r4-recv-commit-any-org` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_self_delete_removes_only_the_organisation_the_revocation_came_from |
| 183 | `r4-selfdel-retain-none` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from |
| 184 | `r4-recv-pending-invite-any-org` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_self_delete_removes_only_the_organisation_the_revocation_came_from |
| 185 | `r4-recv-local-trie-any-org` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_self_delete_removes_only_the_organisation_the_revocation_came_from |
| 186 | `r4-recv-invite-retain-none` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_self_delete_removes_only_the_organisation_the_revocation_came_from |
| 187 | `r4-recv-persona-any` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_self_delete_removes_only_the_organisation_the_revocation_came_from |
| 188 | `r4-import-invite-any-org` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_self_delete_removes_only_the_organisation_the_revocation_came_from |
| 189 | `r4-selfdel-local-trie-any-org` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from |
| 190 | `r4-selfdel-commit-any-org` | GREEN | no test failed |
| 191 | `r4-selfdel-persona-any-org` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from |
| 192 | `r4-selfdel-mine-any-org` | GREEN | no test failed |
| 193 | `r4-recv-binds-last-persona` | GREEN | no test failed |
| 194 | `r4-selfdel-commit-any-org` | RED | membership_of_one_organisation_is_judged_by_that_organisations_personas_alone |
| 195 | `r4-selfdel-mine-any-org` | RED | membership_of_one_organisation_is_judged_by_that_organisations_personas_alone |
| 196 | `r4f-import-invite-any-org` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_self_delete_removes_only_the_organisation_the_revocation_came_from, +1 more |
| 197 | `r4f-recv-binds-last-persona` | GREEN | no test failed |
| 198 | `r4f-recv-local-trie-any-org` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from, a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, +1 more |
| 199 | `r4f-recv-pending-invite-any-org` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_self_delete_removes_only_the_organisation_the_revocation_came_from, +1 more |
| 200 | `r4f-recv-commit-any-org` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from, a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, +1 more |
| 201 | `r4f-recv-invite-retain-none` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_self_delete_removes_only_the_organisation_the_revocation_came_from, +1 more |
| 202 | `r4f-recv-persona-any` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from, a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, +1 more |
| 203 | `r4f-selfdel-binds-last-persona` | GREEN | no test failed |
| 204 | `r4f-selfdel-local-trie-any-org` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from, membership_of_one_organisation_is_judged_by_that_organisations_personas_alone |
| 205 | `r4f-selfdel-mine-any-org` | RED | membership_of_one_organisation_is_judged_by_that_organisations_personas_alone |
| 206 | `r4f-selfdel-commit-any-org` | RED | membership_of_one_organisation_is_judged_by_that_organisations_personas_alone |
| 207 | `r4f-selfdel-retain-none` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from, membership_of_one_organisation_is_judged_by_that_organisations_personas_alone |
| 208 | `r4f-selfdel-persona-any-org` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from |
| 209 | `r4m-import-invite-any-org` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from, a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, +1 more |
| 210 | `r4m-recv-binds-last-persona` | GREEN | no test failed |
| 211 | `r4m-recv-local-trie-any-org` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from, a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, +1 more |
| 212 | `r4m-recv-pending-invite-any-org` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_self_delete_removes_only_the_organisation_the_revocation_came_from, +1 more |
| 213 | `r4m-recv-commit-any-org` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from, a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, +1 more |
| 214 | `r4m-recv-invite-retain-none` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from, a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, +1 more |
| 215 | `r4m-recv-persona-any` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from, a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, +2 more |
| 216 | `r4m-selfdel-binds-last-persona` | GREEN | no test failed |
| 217 | `r4m-selfdel-local-trie-any-org` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from, membership_of_one_organisation_is_judged_by_that_organisations_personas_alone |
| 218 | `r4m-selfdel-mine-any-org` | RED | membership_of_one_organisation_is_judged_by_that_organisations_personas_alone |
| 219 | `r4m-selfdel-commit-any-org` | RED | membership_of_one_organisation_is_judged_by_that_organisations_personas_alone |
| 220 | `r4m-selfdel-retain-none` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from, membership_of_one_organisation_is_judged_by_that_organisations_personas_alone |
| 221 | `r4m-selfdel-persona-any-org` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from |

**The fuzz probes for finding-2.** Each probe is run against
`fuzz_verify_against_chain` alone, three runs each, on both trees. The last one
also runs on the full cargo line.

| Probe | Before the chain fixture was corrected | After, at `7aafcd1` | After, merged |
|---|---|---|---|
| `panic!()` once check 8 has succeeded (`verify.rs:84`) | **passes** (reviewer, 550 iterations) | fails 3/3, first within 246 inputs | fails 3/3, first within 112 inputs |
| check 8 compares the candidate against **its own** root | — | fails 3/3, through the target's `assert_eq!` (`fuzz_target.rs:161` at those trees; `:132` since the deslop pass's `e2baab7` moved the closure, unchanged, above shape 1) | fails 3/3, same line |
| the same, full cargo line | — | RED: `rejects_root_mismatch_when_chain_root_differs` only | RED: the same one test |

And finding-6's probes, from the author's re-measurement before any fix: a
`panic!()` in `sig_bytes::visit_bytes` **fails** `fuzz_verify_against_chain` in
five runs out of five. A `panic!()` inside shape 1's `if let Ok(env)` body
**passes** `fuzz_envelope_decode` over 165 787 iterations.

### Review round 5's block — 2026-10-04

Labels are prefixed `r5-` (the reviewer's) and `r5f-` (the author's re-probes
after the round-5 tests). The reviewer's `r5-r4m-` rows are its reproduction of
round 4's operative rows 209–221. The `r5f-` rows ran on the tree the round-5
tests produced: 109 tests on the line.

| # | Mutation | Verdict | Named test(s) that failed | Run by, at |
|---|---|---|---|---|
| 222 | `r5-r4m-import-invite-any-org` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_self_delete_removes_only_the_organisation_the_revocation_came_from, +1 more | reviewer, `6867e21` |
| 223 | `r5-r4m-recv-binds-last-persona` | GREEN | no test failed | reviewer, `6867e21` |
| 224 | `r5-r4m-recv-local-trie-any-org` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_self_delete_removes_only_the_organisation_the_revocation_came_from, +1 more | reviewer, `6867e21` |
| 225 | `r5-r4m-recv-pending-invite-any-org` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_self_delete_removes_only_the_organisation_the_revocation_came_from, +1 more | reviewer, `6867e21` |
| 226 | `r5-r4m-recv-commit-any-org` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_self_delete_removes_only_the_organisation_the_revocation_came_from, +1 more | reviewer, `6867e21` |
| 227 | `r5-r4m-recv-invite-retain-none` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_self_delete_removes_only_the_organisation_the_revocation_came_from, +1 more | reviewer, `6867e21` |
| 228 | `r5-r4m-recv-persona-any` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_self_delete_removes_only_the_organisation_the_revocation_came_from, +1 more | reviewer, `6867e21` |
| 229 | `r5-r4m-selfdel-binds-last-persona` | GREEN | no test failed | reviewer, `6867e21` |
| 230 | `r5-r4m-selfdel-local-trie-any-org` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from, membership_of_one_organisation_is_judged_by_that_organisations_personas_alone | reviewer, `6867e21` |
| 231 | `r5-r4m-selfdel-mine-any-org` | RED | membership_of_one_organisation_is_judged_by_that_organisations_personas_alone | reviewer, `6867e21` |
| 232 | `r5-r4m-selfdel-commit-any-org` | RED | membership_of_one_organisation_is_judged_by_that_organisations_personas_alone | reviewer, `6867e21` |
| 233 | `r5-r4m-selfdel-retain-none` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from, membership_of_one_organisation_is_judged_by_that_organisations_personas_alone | reviewer, `6867e21` |
| 234 | `r5-r4m-selfdel-persona-any-org` | RED | a_self_delete_removes_only_the_organisation_the_revocation_came_from | reviewer, `6867e21` |
| 235 | `r5-u6rq4s-old-trie` | GREEN | no test failed | reviewer, `6867e21` |
| 236 | `r5-recv-member-id-any` | RED | a_revocation_relayed_by_a_non_member_is_still_acted_on, a_self_delete_reaches_the_revoked_nodes_disk, +2 more | reviewer, `6867e21` |
| 237 | `r5-recv-member-id-unset` | RED | a_revocation_relayed_by_a_non_member_is_still_acted_on, a_self_delete_reaches_the_revoked_nodes_disk, +2 more | reviewer, `6867e21` |
| 238 | `r5-recv-persona-admin-filter` | GREEN | no test failed | reviewer, `6867e21` |
| 239 | `r5-recv-secret-update-kept` | GREEN | no test failed | reviewer, `6867e21` |
| 240 | `r5-recv-secret-first-none` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, five_stories_full_e2e | reviewer, `6867e21` |
| 241 | `r5-recv-no-invite-check` | RED | first_admission_from_a_device_other_than_the_invites_admin_is_rejected | reviewer, `6867e21` |
| 242 | `r5-recv-skip-sender-check` | RED | update_relayed_by_a_non_member_after_admission_is_rejected | reviewer, `6867e21` |
| 243 | `r5-selfdel-update-no-save` | RED | an_update_that_does_not_revoke_us_reaches_the_disk | reviewer, `6867e21` |
| 244 | `r5-selfdel-update-no-lastseq` | GREEN | no test failed | reviewer, `6867e21` |
| 245 | `r5-recv-admin-key-zero` | GREEN | no test failed | reviewer, `6867e21` |
| 246 | `r5-alt-recv-persona-none` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_revocation_relayed_by_a_non_member_is_still_acted_on, +7 more | reviewer, `6867e21` |
| 247 | `r5-alt-recv-mine-any` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_self_delete_removes_only_the_organisation_the_revocation_came_from, +1 more | reviewer, `6867e21` |
| 248 | `r5f-u6rq4s-old-trie` | RED | a_removal_relayed_by_the_member_it_removes_is_refused | author, after the round-5 tests |
| 249 | `r5f-recv-member-id-any` | RED | a_self_delete_reaches_the_revoked_nodes_disk, a_self_delete_removes_only_the_organisation_the_revocation_came_from, +1 more | author, after the round-5 tests |
| 250 | `r5f-recv-member-id-unset` | RED | a_self_delete_reaches_the_revoked_nodes_disk, a_revocation_relayed_by_a_non_member_is_still_acted_on, +4 more | author, after the round-5 tests |
| 251 | `r5f-recv-persona-admin-filter` | GREEN | no test failed | author, after the round-5 tests |
| 252 | `r5f-recv-secret-update-kept` | RED | pr_xwek5e_another_members_revocation_clears_the_receivers_secret | author, after the round-5 tests |
| 253 | `r5f-recv-secret-first-none` | RED | a_first_admission_records_the_signing_key_the_secret_and_the_member, a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, +2 more | author, after the round-5 tests |
| 254 | `r5f-selfdel-update-no-lastseq` | RED | an_update_that_does_not_revoke_us_reaches_the_disk | author, after the round-5 tests |
| 255 | `r5f-recv-admin-key-zero` | RED | a_first_admission_records_the_signing_key_the_secret_and_the_member | author, after the round-5 tests |
| 256 | `r5f-row215-a` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_self_delete_removes_only_the_organisation_the_revocation_came_from, +2 more | author, after the round-5 tests |
| 257 | `r5f-row215-b` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names, a_self_delete_removes_only_the_organisation_the_revocation_came_from, +2 more | author, after the round-5 tests |

Rows 222–234 reproduce rows 209–221. Twelve match exactly. Row
228 (`r5-r4m-recv-persona-any`) fails three tests where row 215 recorded
four. The author re-ran row 215's exact mutation twice (the last two rows of
this block), and both runs fail the same three tests plus the round-5 pin test,
with **no** `five_stories_full_e2e`. Row 215's fourth failure therefore did not
recur in three runs. Round 4's block attributed it to `master`'s edit of
`service_stories.rs`, and **that attribution was a guess**, recorded as if
measured. It is withdrawn: the failure was a one-off in an end-to-end test, not
a property of the mutation.

`r5f-recv-member-id-any` names three tests, not the four the reviewer saw. The
mutation makes the member-id lookup take the first member in trie order, and
MemberIds are drawn at random. So whether B's own member happens to come first,
and the mutation is then harmless, varies from run to run. The deterministic
mutation of the same write, `r5f-recv-member-id-unset`, is RED on six tests.

`r5f-recv-persona-admin-filter` stays **GREEN by design**. It is the
administrator exclusion stated in LLR-e5c9ud, inside PR-mdv38y's site, and the
fix is where it gets a test.

**The reviewer's fuzz probes**, single target, at `6867e21`:

| Probe | Target | Runs | Result |
|---|---|---|---|
| `panic!()` after check 8 succeeds (`verify.rs:84`) | `fuzz_verify_against_chain` | 3 | fails 3/3, within 4, 39 and 194 inputs |
| check 8 against the candidate's own root | `fuzz_verify_against_chain` | 3 | fails 3/3 |
| the same, full line | — | 1 | RED |
| `panic!()` in `sig_bytes::visit_bytes` (`envelope.rs:32`) | `fuzz_verify_against_chain` | 5 | fails 5/5, within 279, 58, 316, 190 and 861 inputs |
| `panic!()` in shape 1's `if let Ok` body | `fuzz_envelope_decode` | 2 | **passes** 2/2, 164 367 and 165 970 inputs |

### The mutations themselves, rounds 4 and 5

*Added 2026-10-04 by review round 5's finding-11. A label is not a mutation,
and row 215 could not be re-run exactly because only its label was recorded.
Every `r4f-`, `r4m-`, `r5-` and `r5f-` row above is one of these edits: one
line of the named file, the left text replaced by the right. The tree each ran
on is named in its block. Earlier rounds' rows were driven from scratch files
that no longer exist, and they are not reconstructed here, because a
reconstruction would be a new mutation wearing an old label.*

| Label (without prefix) | File:line | Replaced | With |
|---|---|---|---|
| `import-invite-any-org` | `org-node/src/service.rs:722` | `.find(\|p\| p.org_id == inv.org_id)` | `.find(\|_p\| true)` |
| `recv-binds-last-persona` | `org-node/src/service.rs:910` | `.first()` | `.last()` |
| `recv-local-trie-any-org` | `org-node/src/service.rs:957` | `.find(\|o\| o.org_id == org_id)` | `.find(\|_o\| true)` |
| `recv-pending-invite-any-org` | `org-node/src/service.rs:983` | `.find(\|p\| p.org_id == org_id)` | `.find(\|_p\| true)` |
| `recv-commit-any-org` | `org-node/src/service.rs:1071` | `.find(\|o\| o.org_id == org_id)` | `.find(\|_o\| true)` |
| `recv-invite-retain-none` | `org-node/src/service.rs:1093` | `.retain(\|p\| p.org_id != org_id)` | `.retain(\|_p\| false)` |
| `recv-persona-any` | `org-node/src/service.rs:1100` | `.find(\|p\| &p.persona_id == pid)` | `.find(\|_p\| true)` |
| `selfdel-binds-last-persona` | `org-node/src/service.rs:1272` | `.first()` | `.last()` |
| `selfdel-local-trie-any-org` | `org-node/src/service.rs:1299` | `.find(\|o\| o.org_id == org_id)` | `.find(\|_o\| true)` |
| `selfdel-mine-any-org` | `org-node/src/service.rs:1324` | `.filter(\|p\| p.org_id == Some(org_id))` | `.filter(\|_p\| true)` |
| `selfdel-commit-any-org` | `org-node/src/service.rs:1351` | `.find(\|o\| o.org_id == org_id)` | `.find(\|_o\| true)` |
| `selfdel-retain-none` | `org-node/src/service.rs:1363` | `.retain(\|o\| o.org_id != org_id)` | `.retain(\|_o\| false)` |
| `selfdel-persona-any-org` | `org-node/src/service.rs:1366` | `if p.org_id == Some(org_id) {` | `if true {` |
| `u6rq4s-old-trie` | `org-node/src/service.rs:1011` | `let sender_known = verified` | `let sender_known = (crate::verify::VerifiedUpdate { trie: local_trie.clone(), seq_guard: SeqGuard::from_last_seen(0), epoch: 0 })` |
| `recv-member-id-any` | `org-node/src/service.rs:1063` | `.find(\|m\| m.has_p2p_device(&my_device_key))` | `.find(\|_m\| true)` |
| `recv-member-id-unset` | `org-node/src/service.rs:1103` | `p.member_id = my_member_id;` | `p.member_id = None;` |
| `recv-persona-admin-filter` | `org-node/src/service.rs:1049` | `&& m.p2p_key().as_bytes() != chain_state.org_pub_key.as_ref()` | `&& true` |
| `recv-secret-update-kept` | `org-node/src/service.rs:1075` | `existing.org_secret = msg.org_secret;` | `existing.org_secret = existing.org_secret;` |
| `recv-secret-first-none` | `org-node/src/service.rs:1083` | `org_secret: msg.org_secret,` | `org_secret: None,` |
| `recv-no-invite-check` | `org-node/src/service.rs:985` | `if let Some(inv) = pending {` | `if let Some(inv) = pending.filter(\|_\| false) {` |
| `recv-skip-sender-check` | `org-node/src/service.rs:1016` | `if !sender_known {` | `if false && !sender_known {` |
| `selfdel-update-no-save` | `org-node/src/service.rs:1358` | `self.store.save(rng)?;` | `let _ = &rng;` |
| `selfdel-update-no-lastseq` | `org-node/src/service.rs:1354` | `rec.last_seq = verified.seq_guard.last_seen();` | `rec.last_seq = rec.last_seq;` |
| `recv-admin-key-zero` | `org-node/src/service.rs:1085` | `admin_member_key: chain_state.org_pub_key,` | `admin_member_key: [0u8; 32],` |
| `alt-recv-persona-none` | `org-node/src/service.rs:1100` | `.find(\|p\| &p.persona_id == pid)` | `.find(\|_p\| false)` |
| `alt-recv-mine-any` | `org-node/src/service.rs:1045` | `.find(\|p\| {` | `.find(\|p\| true \|\| {` |

The `r4m-` and `r5-r4m-` rows use the `r4f-` mutations at the same lines;
`master`'s `24317e3` merge moved no line of `service.rs`. The `r5f-` rows use
the `r5-` mutation of the same name, and `r5f-row215-a`/`-b` use `recv-persona-any`.
The fuzz probes of rounds 4 and 5 are listed with their edits in their own
tables.

### Review round 6's block — 2026-10-04

Labels are prefixed `r6f-`, the author's probes of round 6's remedies, on the
tree those remedies produced: 113 tests on the line. The reviewer's own runs
are in its scratch directory and are summarised in round 6's section. They
are not copied here, because each reproduced a row already above it.

| # | Mutation | Verdict | Named test(s) that failed |
|---|---|---|---|
| 258 | `r6f-admit-root-kept` | RED | an_admission_reaches_the_administrators_disk |
| 259 | `r6f-revoke-root-kept` | RED | a_revocation_reaches_the_administrators_disk |
| 260 | `r6f-recv-update-root-kept` | RED | a_receiver_holding_two_organisations_commits_into_the_one_the_change_names |
| 261 | `r6f-rc74nq-field-name` | RED | update_call_names_every_field_and_constant_the_runtime_matches |
| 262 | `r6f-rc74nq-constant` | RED | update_call_names_every_field_and_constant_the_runtime_matches |

Each is RED on exactly the test written for it, and on nothing else.

| Label (without prefix) | File:line | Replaced | With |
|---|---|---|---|
| `admit-root-kept` | `org-node/src/service.rs:884` | `org_rec.root_hash = new_root;` | `org_rec.root_hash = org_rec.root_hash;` |
| `revoke-root-kept` | `org-node/src/service.rs:1250` | `org_rec.root_hash = new_root;` | `org_rec.root_hash = org_rec.root_hash;` |
| `recv-update-root-kept` | `org-node/src/service.rs:1072` | `existing.root_hash = new_root;` | `existing.root_hash = existing.root_hash;` |
| `rc74nq-field-name` | `org-node/src/chain_write/calldata.rs:67` | `"weight_limit"` | `"gas_limit"` |
| `rc74nq-constant` | `org-node/src/chain_write/calldata.rs:15` | `4_000_000` | `4_000_001` |

### Review round 7's block — 2026-10-04

Labels are prefixed `r7f-`. These are the reviewer's round-7 mutations, re-run by the
author on the tree round 7's remedies produced (116 tests on the line), each
from the spec below. On round 7's tree every one of them was GREEN.

| # | Mutation | Verdict | Named test(s) that failed |
|---|---|---|---|
| 263 | `r7f-rc74nq-dest-reversed` | RED | update_call_names_every_field_and_constant_the_runtime_matches |
| 264 | `r7f-3q63zv-update-branch-checks-sender` | RED | pr_u4c2vp_an_update_relayed_by_a_non_member_is_committed_on_the_self_delete_path |
| 265 | `r7f-selfdel-no-record-error-retyped` | RED | the_self_delete_path_refuses_an_organisation_it_holds_no_record_of |
| 266 | `r7f-first-admission-without-invite-refused` | RED | a_first_admission_with_no_imported_invite_rests_on_the_chain_alone |
| 267 | `r7f-sxd3tg-root-kept` | RED | an_update_that_does_not_revoke_us_reaches_the_disk, revocation_of_another_member_is_committed_not_self_deleted |
| 268 | `r7f-68yd3j-genesis-root-zero` | RED | creating_an_organisation_advances_the_chain_and_activates_the_persona |

| Label (without prefix) | File:line | Replaced | With |
|---|---|---|---|
| `rc74nq-dest-reversed` | `org-node/src/chain_write/calldata.rs:50` | `.iter()` | `.iter().rev()` |
| `3q63zv-update-branch-checks-sender` | `org-node/src/service.rs:1333` | `if my_still_present {` | `if my_still_present && !verified.trie.members().iter().any(\|m\| m.has_p2p_device(&org_members::P2pDeviceKey::new(*remote_device_key.verifying_key()))) { return Err(OrgNodeError::BadSignature); } if my_still_present {` |
| `selfdel-no-record-error-retyped` | `org-node/src/service.rs:1301` | `.ok_or(OrgNodeError::OrgNotOnChain)?;` | `.ok_or(OrgNodeError::Chain("no local record".into()))?;` |
| `first-admission-without-invite-refused` | `org-node/src/service.rs:985` | `if let Some(inv) = pending {` | `if pending.is_none() { return Err(OrgNodeError::BadSignature); } if let Some(inv) = pending {` |
| `sxd3tg-root-kept` | `org-node/src/service.rs:1352` | `rec.root_hash = new_root;` | `rec.root_hash = rec.root_hash;` |
| `68yd3j-genesis-root-zero` | `org-node/src/service.rs:654` | `root_hash: genesis_root,` | `root_hash: [0u8; 32],` |

*The trace check.* `tracecheck.py` (author's scratch directory) maps each
rounds-4-to-7 RED row to the LLR its mutation targets, and checks that a red
test carries it. Twenty-nine rows were checked, and after the two annotations
round 7's section describes, one is left without a carrying test:
`r5f-recv-member-id-any`, the probabilistic row explained in round 5's block.

### Review round 8's block — 2026-10-05

Labels are prefixed `r8f-`. Thirteen are the reviewer's round-8 rows, re-run by
the author on the tree round 8's remedies produced (121 tests on the line),
each from the spec below. On round 8's tree every one of them was GREEN or, for
`duwz79` and `wx77j5`, RED only on tests carrying other requirements. Two are
the author's: `revoke-proxy-not-passed`, the revocation site of finding-4, and
`vw2jn6-acts-before-verify`, the fresh measurement of LLR-vw2jn6 that round
8's section describes.

| # | Mutation | Verdict | Named test(s) that failed |
|---|---|---|---|
| 269 | `r8f-322qst-fix-refuses` | RED | pr_322qst_an_own_revocation_on_the_ordinary_path_is_committed_not_self_deleted |
| 270 | `r8f-revoke-snapshot-none` | RED | a_revocation_reaches_the_administrators_disk |
| 271 | `r8f-revoke-secret-some` | RED | a_revocation_reaches_the_administrators_disk, pr_xwek5e_another_members_revocation_clears_the_receivers_secret |
| 272 | `r8f-export-invite-addr-empty` | RED | the_out_of_band_blobs_carry_the_keys_their_holders_are_pinned_by |
| 273 | `r8f-genesis-proxy-not-persisted` | RED | the_proxy_account_from_genesis_is_kept_and_passed_on_every_update |
| 274 | `r8f-admit-proxy-not-passed` | RED | the_proxy_account_from_genesis_is_kept_and_passed_on_every_update |
| 275 | `r8f-q8emds-consumed-before-verify` | RED | a_first_admission_that_fails_verification_leaves_the_invite_pending |
| 276 | `r8f-379hnv-revokes-personas` | RED | the_self_delete_path_refuses_an_organisation_it_holds_no_record_of |
| 277 | `r8f-mock-cas-off` | RED | the_mock_chain_refuses_an_update_at_the_wrong_epoch |
| 278 | `r8f-founding-sets-member-id` | RED | pr_mdv38y_founding_an_organisation_rebinds_a_member_persona, creating_an_organisation_advances_the_chain_and_activates_the_persona |
| 279 | `r8f-recv-one-checks-prefix` | RED | the_length_prefix_is_not_checked_against_the_body |
| 280 | `r8f-duwz79-starts-at-zero` | RED | a_stale_sequence_rejection_carries_the_offered_number_and_the_mark, from_last_seen_starts_the_guard_at_the_given_mark, check_does_not_advance_the_mark, rejects_equal_and_lower_seq, rejects_root_mismatch_when_chain_root_differs, rejects_stale_seq_before_decoding_delta, rejects_stale_seq |
| 281 | `r8f-wx77j5-reports-another-id` | RED | pr_8qsnhx_a_join_request_advertises_the_bound_endpoint_not_its_personas, loopback_mode_binds_and_advertises_loopback_only, a_stream_longer_than_the_read_bound_is_refused_rather_than_buffered, the_length_prefix_is_not_checked_against_the_body |
| 282 | `r8f-revoke-proxy-not-passed` | RED | the_proxy_account_from_genesis_is_kept_and_passed_on_every_update |
| 283 | `r8f-vw2jn6-acts-before-verify` | RED | unverified_revocation_leaves_the_record_in_place |

| Label (without prefix) | File:line | Replaced | With |
|---|---|---|---|
| `322qst-fix-refuses` | `org-node/src/service.rs:1054` | `// Find the member_id for our persona.` | `let still_in = self.store.data().personas.iter().filter(\|p\| p.org_id == Some(org_id)).any(\|p\| { let dk = SigningKeypair::from_seed(p.device_seed); verified.trie.members().iter().any(\|m\| m.has_p2p_device(&org_members::P2pDeviceKey::new(dk.verifying_key()))) }); if !is_first_admission && !still_in { return Err(OrgNodeError::OrgNotOnChain); }` |
| `revoke-snapshot-none` | `org-node/src/service.rs:1208` | `org_secret: None, genesis_snapshot }` | `org_secret: None, genesis_snapshot: genesis_snapshot.filter(\|_\| false) }` |
| `revoke-secret-some` | `org-node/src/service.rs:1208` | `org_secret: None,` | `org_secret: Some([7u8; 32]),` |
| `export-invite-addr-empty` | `org-node/src/service.rs:688` | `if let Some(ep) = &self.endpoint {` | `if let Some(ep) = self.endpoint.as_ref().filter(\|_\| false) {` |
| `genesis-proxy-not-persisted` | `org-node/src/service.rs:664` | `proxy_account,` | `proxy_account: proxy_account.filter(\|_\| false),` |
| `admit-proxy-not-passed` | `org-node/src/service.rs:838` | `org_epoch, proxy_account)` | `org_epoch, proxy_account.filter(\|_\| false))` |
| `q8emds-consumed-before-verify` | `org-node/src/service.rs:994` | `let seq_guard = SeqGuard::from_last_seen(last_seq);` | `if is_first_admission { self.store.data_mut().pending_invites.retain(\|p\| p.org_id != org_id); } let seq_guard = SeqGuard::from_last_seen(last_seq);` |
| `379hnv-revokes-personas` | `org-node/src/service.rs:1301` | `.ok_or(OrgNodeError::OrgNotOnChain)?;` | `.ok_or(OrgNodeError::OrgNotOnChain); if existing.is_err() { for p in self.store.data_mut().personas.iter_mut() { p.status = PersonaStatus::Revoked; } self.store.save(rng)?; } let existing = existing?;` |
| `mock-cas-off` | `org-node/src/service.rs:145` | `if state.epoch != expected_epoch {` | `if false && state.epoch != expected_epoch {` |
| `founding-sets-member-id` | `org-node/src/service.rs:1506` | `p.org_id = Some(org_id);` | `p.org_id = Some(org_id); p.member_id = Some([9u8; 32]);` |
| `recv-one-checks-prefix` | `org-node/src/transport/endpoint.rs:337` | `let payload = if raw.len() >= 4 { &raw[4..] } else { &raw[..] };` | `if raw.len() < 4 \|\| u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]) as usize != raw.len() - 4 { return Err(TransportError::Malformed); } let payload = &raw[4..];` |
| `duwz79-starts-at-zero` | `org-node/src/sequence.rs:20` | `Self { last_seen }` | `Self { last_seen: last_seen & 0 }` |
| `wx77j5-reports-another-id` | `org-node/src/transport/endpoint.rs:230` | `self.inner.id(),` | `iroh::SecretKey::from_bytes(&[3u8; 32]).public(),` |
| `revoke-proxy-not-passed` | `org-node/src/service.rs:1170` | `org_epoch, proxy_account)` | `org_epoch, proxy_account.filter(\|_\| false))` |
| `vw2jn6-acts-before-verify` | `org-node/src/service.rs:1316` | `verify_envelope_against_chain(&local_trie, &msg.envelope, &ctx, &chain_reader)?;` | `match verify_envelope_against_chain(&local_trie, &msg.envelope, &ctx, &chain_reader) { Ok(v) => v, Err(e) => { self.store.data_mut().orgs.retain(\|o\| o.org_id != org_id); return Err(e); } };` |

*The trace check.* Every row above is red on a test carrying the requirement
it targets: `322qst-fix-refuses` on the pin carrying LLR-cja9zv,
`duwz79-starts-at-zero` on `from_last_seen_starts_the_guard_at_the_given_mark`,
`wx77j5-reports-another-id` on `loopback_mode_binds_and_advertises_loopback_only`,
`vw2jn6-acts-before-verify` on `unverified_revocation_leaves_the_record_in_place`,
and each of the others on the carrier its requirement names.

### Review round 9's block — 2026-10-05

Labels are prefixed `r9f-`. These are the reviewer's three round-9 mutations,
re-run by the author on the tree round 9's remedies produced (122 tests on the
line). On round 9's tree the first was RED only on a test carrying another
requirement, and the other two were GREEN. Each is now RED on a test carrying
the requirement it targets.

| # | Mutation | Verdict | Named test(s) that failed |
|---|---|---|---|
| 284 | `r9f-8hdu9x-receiver-keeps-secret` | RED | pr_xwek5e_another_members_revocation_clears_the_receivers_secret |
| 285 | `r9f-qezw3n-own-persona-only` | RED | pr_8qsnhx_an_invite_advertises_the_bound_endpoint_not_its_administrators |
| 286 | `r9f-2smrvx-short-refused-as-stream` | RED | the_length_prefix_is_not_checked_against_the_body |

| Label (without prefix) | File:line | Replaced | With |
|---|---|---|---|
| `8hdu9x-receiver-keeps-secret` | `org-node/src/service.rs:1075` | `existing.org_secret = msg.org_secret;` | `existing.org_secret = msg.org_secret.or(existing.org_secret);` |
| `qezw3n-own-persona-only` | `org-node/src/service.rs:688` | `if let Some(ep) = &self.endpoint {` | `if let Some(ep) = self.endpoint.as_ref().filter(\|e\| e.node_addr_for_dial().id.as_bytes() == device_kp.verifying_key().as_bytes()) {` |
| `2smrvx-short-refused-as-stream` | `org-node/src/transport/endpoint.rs:337` | `else { &raw[..] }` | `else { return Err(TransportError::Stream("short".into())); }` |

# on-chain-client risk analysis — implementation plan

**Goal:** write the `on-chain-client` unit's first ISO 14971 risk analysis, mint
the requirements that realise its controls, and put the evidence for them where
the unit's gate reads it.
**Implements:** HAZ-werm85, HAZ-8s5chy, HAZ-95sc43, HAZ-v2cmtx, HAZ-xfg9cz,
HAZ-xd4urb, RC-5e3bdk, RC-6gfh8d, RC-d7r82e, RC-8w9wtp, RC-mugta4, RC-5ejucb,
RC-sxjnx9, RC-kemv75, RC-675a3h, REQ-9vwcwc, REQ-nygs7k, REQ-wnjz9j, REQ-88fp2h,
REQ-twdu84, REQ-hd6m9d, REQ-sx5b6g, REQ-axcxf7, REQ-tkhe3u, REQ-rz7fja,
REQ-9m2rnd, REQ-xudf25, REQ-4astjb, REQ-9wwenn, REQ-ntn4ss, REQ-gr2ver,
REQ-5zux82, REQ-52uc8f, REQ-n6v896, REQ-2qa5r5, REQ-5upq6n

The last five were minted in review round 1's fix round; see "Review round 1 —
findings and dispositions" for which finding each answers. Totals: **21
requirements, 9 controls**, 6 hazards, 5 problem reports.
**Safety class:** C (`on-chain-client/.guardrails/config.yaml`; no per-item
override). Every unit in this repository is class C —
`docs/adr/2026-09-05-units-and-per-unit-classes.md`.
**Verification:** `on-chain-client`'s `verify_commands`, extended by this change
with `--features test-support` and the **nine** new test targets — the seven the
plan first named, plus `log_ownership` added by T2b and `type_widths` added in
review round 1's fix round; then
`check-trace.sh` and `check-ids.sh` per unit over the impact set, and
`check-units.sh` once.

This is the second half of tooth 3 of `docs/plans/2026-09-05-ratchet-gap-analysis.md`.
The org-node half merged as the signed squash carrying
`docs/plans/2026-09-09-org-node-risk-analysis.md`. The `app` unit's analysis is
the next change, not this one.

**Filenames, corrected on 2026-09-17 — and this change never ran
`finalize-docs.sh` at all.** Every ledger file this plan names below is named by
its draft name, and those files kept those names on `master` from this change's
merge (`fbf17f0`, 2026-09-11) until the repair merged. `merge-change` step 3 was
skipped, so the three drafts were squashed onto the base branch as drafts. The
text below is left as written — this project dates its corrections rather than
rewriting the record — so read them as:

| written below | actual file |
|---|---|
| `…/requirements/DRAFT-worktree-guardrails-on-chain-client-risk-chain-reading.md` | `on-chain-client/docs/requirements/2026-09-10-chain-reading.md` |
| `…/risk/DRAFT-worktree-guardrails-on-chain-client-risk-on-chain-client-hazards.md` | `on-chain-client/docs/risk/2026-09-10-on-chain-client-hazards.md` |
| `…/problems/DRAFT-worktree-guardrails-on-chain-client-risk-on-chain-client-problems.md` | `on-chain-client/docs/problems/2026-09-10-on-chain-client-problems.md` |

**Why 2026-09-10 and not 2026-09-11, the merge date.** The owner decided this on
2026-09-17. Two reasons, each checkable:

1. `docs/verification/2026-09-11-worktree-guardrails-on-chain-client-risk.md`
   already asserts it, under Gaps: "The date on this record is 2026-09-11; the
   ledger files carry 2026-09-10 names. The change was finalized on the 10th and
   merged on the 11th." That was written as a statement of fact and it was never
   true, because step 3 never ran and the files carried draft names instead.
   Naming them 2026-09-10 makes the existing assertion true rather than leaving
   a certified record contradicted by the tree.
2. It is what both sibling units of this same tooth actually did. `org-node`'s
   half merged as `50a5254` on **2026-09-10** and its ledger files are
   `org-node/docs/risk/2026-09-09-org-node-hazards.md`,
   `org-node/docs/requirements/2026-09-09-verify-and-commit.md` and
   `org-node/docs/problems/2026-09-09-org-node-problems.md` — the day before the
   merge. `app`'s merged as `5c0c710` on **2026-09-15** and its files are
   `app/docs/risk/2026-09-14-app-hazards.md`,
   `app/docs/requirements/2026-09-14-tauri-shell.md` and
   `app/docs/problems/2026-09-14-app-problems.md` — again the day before. Both
   siblings are finalisation-dated, not merge-dated. 2026-09-10 puts this unit
   in the same convention as the other two thirds of the tooth.

**The three names were assigned by hand, and `finalize-docs.sh` was
deliberately bypassed for them.** That script takes no date argument — its
usage is `finalize-docs.sh [--dry-run]`, it stamps `today=$(date +%Y-%m-%d)`,
and it strips only the **current** branch's name prefix from the slug. Run here
on 2026-09-17, from the branch `worktree-guardrails-on-chain-client-finalize`,
against files still named `DRAFT-worktree-guardrails-on-chain-client-risk-…`, it
would have produced
`2026-09-17-worktree-guardrails-on-chain-client-risk-<slug>.md` — wrong in both
halves: the wrong date, and a stale branch prefix left inside the slug because
the prefix it strips is this change's branch, not the one that wrote the drafts.
Running the prescribed remedy six days late does not reproduce what the
prescribed remedy would have produced on the day. So the renames were done by
hand to the names the tool would have produced had it run when due, adjusted to
2026-09-10 for the two reasons above. `finalize-docs.sh --dry-run` was still run
at `merge-change` step 3 of the repair, with this unit's `GR_CONFIG`, and
correctly reported nothing left to rename (exit 0, no output) — the step was
performed, not skipped a second time.

**Honestly, this diverges from what the ledger READMEs say.** All three of
`on-chain-client/docs/{requirements,risk,problems}/README.md` define the dated
name as "merge date, assigned by `merge-change`", so a 2026-09-10 name is not
what those READMEs describe. The divergence is **pre-existing and already
recorded**: review of the org-node half raised exactly this as **finding-69** in
`docs/verification/2026-09-10-worktree-guardrails-org-node-risk.md` — "the ledger
files were finalised to merge-dated names on 2026-09-09 while fix rounds ran to
2026-09-10, so the dated names are not the merge date the ledger README defines
them as" — dispositioned "accepted, not fixed". So the convention in practice is
the finalisation date and the READMEs' wording has been wrong since tooth 2.
Reconciling the two is the owner's open item, filed in
`docs/plans/2026-09-05-ratchet-setup.md`, and is not done here.

**What the omission cost, which is more than three filenames.** A draft-named
ledger file is `DRAFT-FILE` under `check-ids.sh`, so this unit's id gate has been
exit 1 on `master` from 2026-09-11 until the repair merged — at least six days,
and no terminal date is written here because the repair had not merged when this
was written, and a span measured against a merge that has not happened is a
guess.

The **local** sequence could not have caught it. `verify-before-merge` runs
`check-ids.sh` **with** `--allow-draft-files`, which is exactly the flag that
forgives a draft-named file, and the only local run without the flag is
`merge-change` step 4 — the step the same omission also skipped. Skipping step 3
and skipping the gate that would have convicted the skip are the same omission,
so the change that caused it could not detect it locally.

**CI could have, and this is where the larger finding is.** `.github/workflows/rust.yml`
does not run `check-ids.sh` bare only at merge: its `guardrails` job is
`on: [push, pull_request]` and runs the per-unit loop **twice, conditionally** —
`check-ids.sh --allow-draft-files` when `github.event_name == 'pull_request'`,
and `check-ids.sh` **bare** when `github.event_name != 'pull_request'`. There are
therefore **two** bare runners, not one, and the second is CI on any push. Had
`fbf17f0` ever been pushed, that job would have convicted it on the first push to
`master`, the same day.

It was never pushed. **`origin/master` is still at `2bb1c21`, dated 2026-06-17,
and local `master` is 45 commits ahead of it.** Nothing has reached the remote in
three months, so no workflow has run on any of those 45 commits.

**And `rust.yml` is worse than dormant: it has never run once.** The workflow
was added by `4bb5509` (2026-08-27), which is itself one of the 45 unpushed
commits — `git merge-base --is-ancestor 4bb5509 origin/master` exits 1, and
`origin/master` has no `.github/workflows/rust.yml` at all. So the bare
`check-ids.sh` described in the paragraph above has never executed on any
commit in this repository's history. "Would have convicted `fbf17f0` on the
first push" remains true — the workflow would have gone up with the commits —
but it is a statement about a workflow with no execution record whatsoever, not
about a gate that used to fire and went quiet.

`quint.yml` is the one workflow with real history, and it splits per invariant.
It is present at `2bb1c21`, so its steps as they stood there ran on pushes up to
2026-06-17: `forkSafety`, `revocationSafety` and `revokedExcludedFromOrgSecret`
under the simulator at 5000 samples / 16 steps, the `mbtInv` run, the typechecks
and `quint test`, and the `apalache` job's bounded verify of those same three at
depth 5. `620b459` (2026-06-18) is the only commit touching the file since and is
also unpushed, so everything it added — the `tauWindow` and `convergence`
simulator runs, and their verifies at depths 5 and 3 — has **never** run. Two of
five randomised invariants and two of five bounded verifies have no execution
history at all.

What that actually costs is narrower than "all of CI", and the distinction
matters because overstating it makes the local gates look weaker than they are.
CI-only, with no gate running them anywhere in the merge sequence:

- the bare `check-ids.sh` per unit — with the caveat that a local bare run *does*
  exist, at `merge-change` step 4; what CI adds is a bare run **independent of
  the operator reaching that step**, which is exactly the independence this
  defect needed and did not have;
- `cargo clippy … --lib -- -D warnings` for org-members and on-chain-client, the
  panic-freedom denial — **prescribed locally but gated nowhere**, which is the
  precise charge: `org-members/AGENTS.md` line 117 gives
  `cargo build && cargo test && cargo clippy` as the crate's default command,
  and no Makefile target, no unit `verify_commands` entry and no step of
  `verify-before-merge` or `merge-change` names clippy at all;
- the `no_std` and `wasm32-unknown-unknown` compile checks for org-members —
  same shape again. `org-members/AGENTS.md` lines 24-25 make
  `cargo check --no-default-features --features serde --target
  wasm32-unknown-unknown` mandatory "after any dependency change" and lines
  120-122 list all three build configurations, so a developer is told to run
  them; nothing at merge does;
- in `quint.yml`: `quint typecheck quint/membership_mbt.qnt`,
  `quint typecheck quint/ods_instances.qnt`, `quint test quint/membership.qnt`,
  `quint run quint/membership_mbt.qnt --invariant=mbtInv`, the five randomised
  5000-sample invariant runs over `protocol.qnt`, and the `apalache` job's five
  bounded `quint verify`. **Not** the whole workflow: `quint typecheck
  quint/membership.qnt` and `quint typecheck quint/protocol.qnt` are verbatim
  `verify_commands` entries of org-members (and `protocol.qnt` of org-node too),
  and the `mbt` job's `cargo test --test mbt_conformance` is already inside
  org-members' `cargo test -p org-members` — `rust.yml`'s own comment calls that
  job "now redundant". And of those five randomised runs and five verifies, only
  three of each ever executed; `tauWindow` and `convergence` were added by the
  unpushed `620b459` and never have.

Two gates commonly put on this list do **not** belong on it.
**`check-signing.sh` runs locally at every merge**: `finish-merge.sh` line 130
runs it `--strict`, unconditionally, as guard 1 before any branch or worktree is
removed, and `rust.yml` runs it **non-strict** because a runner holds no public
key — so the local gate is the stricter of the two and CI would add an
independent backstop, not the only enforcement. **`make coverage` runs locally
too**: `Makefile` line 139 is `coverage: coverage-org-members
coverage-on-chain-client`, and those two targets are exactly the
`coverage_command:` entries the units declare, consumed by `verify-before-merge`
check 5. What is dormant there is only the **cross-platform re-run** — floors
calibrated on aarch64-darwin, enforced in CI on x86_64-linux against a floating
`stable` toolchain, a difference `rust.yml`'s `coverage` job comment flags itself.

That is the real reason nothing convicted this defect, and it is much larger than
this defect. It is filed as an owner decision in
`docs/plans/2026-09-05-ratchet-setup.md`.

**How it was actually found, which was luck and not mechanism.** It was found on
2026-09-17 during the org-members architecture change
(`docs/plans/2026-09-15-org-members-architecture.md`), but **not** by that
change's gates. That change's verification record states it plainly: its impact
set, computed with `check-units.sh --impact "master..HEAD"`, was "`org-members`
touched, `org-node` dependent, `app` touched", and "`on-chain-client` is outside
the set and its gates were not run for this change"
(`docs/verification/2026-09-17-worktree-guardrails-org-members-arch.md`). The red
result came from a **deliberate out-of-scope probe** — a hand-run
`GR_CONFIG=on-chain-client/.guardrails/config.yaml check-ids.sh` from that
change's worktree, recorded in its plan under "A defect on master, found by this
change's gate but not caused by it", and explicitly left unfixed there because
touching this unit would have pulled it into that change's impact set.

The structural point matters more than the attribution. **The dependency graph
alone could never have pulled `on-chain-client` into an org-members change's
impact set**: the four declared edges (`.guardrails/units.yaml`) are org-node →
org-members, org-node → on-chain-client, app → org-node, app → on-chain-client,
so org-members is a leaf and on-chain-client neither depends on it nor is
depended on by it. No amount of propagation over those edges reaches this unit
from that one.

That is a claim about the edges, and it must not be inflated into "never, by any
route". `check-units.sh --impact` has a second rule, stated in the script's own
header: "A change under the root `.guardrails/` maps to EVERY unit." So an
org-members change that also touched the root `.guardrails/` — a script upgrade,
a manifest edit — would have run on-chain-client's gates and convicted this
defect, and so would one that happened to touch both units' files, since impact
starts from *touched* units. The change that found this touched neither, which
is why its own record reads "`on-chain-client` is outside the set". The honest
statement is that the mechanism **as exercised by this change** could not have
caught it, and crediting it would make the process look sounder than it is. What
caught it was one engineer choosing to run a gate outside their scope.

## The interview

Asked one question at a time with a recommendation, 2026-09-10. Answers:

| Question | Answer |
|---|---|
| This change: on-chain-client alone, or with `app`? | **on-chain-client alone.** `app` is a different-shaped job — 784 lines of Rust, a SvelteKit frontend outside `strict_paths`, no tests and no `coverage_command`, so every control there needs test infrastructure built first. It is also the consumer of both providers and reads better once both registers exist. |
| org-node's expectation REQ-ysyu9g (read at the latest **finalised** block): meet it here? | **Record the obligation, meet it later.** Meeting it flips org-node's item from exempt to ordinary, so org-node must carry a `verifies:` test at this same merge — and its only honest test is a chopsticks target no gate runs. The register states the confirmed fact and carries the obligation as a not-minted control. Deadline 2026-12-05 unchanged. |
| The nine chopsticks/anvil targets no gate runs: what is this unit's evidence? | **Gated tests over the pure paths, fixture replay where bytes were captured.** The chain-dependent properties get prose hazards and not-minted controls that name the ungated targets, so the gap is visible instead of papered over. |
| `event_matches_contract()` unconditionally returns `true`: problem report, or fix? | **Fix it in this change.** The control is otherwise unimplementable and the fix is small and directly gatable. |

The severity and probability answers are in the register and were confirmed
separately once the hazard list was drafted.

## What the survey found

Read in full before any item was written: `src/lib.rs`, `src/types.rs`,
`src/state.rs`, `src/verify.rs`, `src/h160.rs`, `src/client.rs`,
`src/decode/mod.rs`, `src/decode/dispatch.rs`, `src/decode/v_paseo_ah.rs`, all
of `tests/`, the unit config, and `subxt-0.50.1/src/client/online_client.rs`.

Five findings drive this change.

1. **No contract-address filtering exists.** `parse_revive_event` decodes the
   `ContractEmitted` payload's `contract` field into `let _contract` and drops
   it; `event_matches_contract` is `let _ = (ev, contract); true`. The
   `OrgRegistryClient::contract` field's own doc-comment says "Events from
   other contract addresses are filtered out before reaching subscribers",
   which is false. Any contract on Asset Hub can emit a log with the
   OrgRegistry signature hash and a victim organisation's admin address, and
   every subscriber receives it as a genuine event with an attacker-chosen
   root. Fixed here (T2); recorded as PR-p5ngya, resolved by this change.

2. **The decoder is pinned at construction and never re-checked.**
   `from_client` resolves `dispatch::for_runtime(spec_version)` once. The
   doc-comment says the client "should be reconstructed" if the runtime
   upgrades mid-session; nothing enforces it, and `get_org_state` already
   resolves a block whose `spec_version()` it could compare. pallet-revive is
   pre-stable by this crate's own account. Open: PR-w5sk5k.

3. **`src/verify.rs` is seven lines of doc-comment and no code.** The module
   the design names as the one that "closes the loop with
   `org_members::CandidateTrie::verify_against`" contains nothing, so no
   verification happens in the unit named for it; the whole burden sits on the
   consumer. Open: PR-h4mb8y.

4. **The best lane's first backfill span is unbounded.** The seed is the
   latest **finalised** block, which on a live chain lags the best tip, so the
   first head notification backfills that whole span at one `at_block`
   round-trip per height. The code comment admits it. Open: PR-uq5r97.

5. **A reorg deeper than the notification gap is undetectable.** For a jump
   `n > last.number + 1` the code's own comment records that "no reorg signal
   is derivable", and the by-number backfill can only ever see canonical
   blocks. Open: PR-qpp28h.

6. **"An absent slot means no such organisation" is sound only because of a
   guard in a contract outside this unit.** `get_org_state` returns `Ok(None)`
   the moment any one of the three slots reads absent, and EVM storage cannot
   tell an unset slot from one holding zero. What makes that safe is
   `on-chain/src/OrgRegistry.sol`: `update` reverts with `ZeroValue()` if
   either `rootHash` or `orgPubKey` is zero, the epoch is `1` after genesis and
   `+1` thereafter, and all three fields are written in the one call — so every
   slot of an initialised organisation is non-zero and the three are always
   written together. Both halves of the client's comment ("the contract writes
   all three slots atomically", a partial read "shouldn't appear") are therefore
   true today. They are true because of a file in `on-chain/`, which
   `.guardrails/units.yaml` disclaims, and the contract's own tests for exactly
   these two reverts (`test_RevertsZeroValue_WhenRootHashIsZero`,
   `…WhenOrgPubKeyIsZero` in `on-chain/test/OrgRegistry.t.sol`) run in no CI
   lane in this repository — `.github/workflows/` holds `rust.yml` and
   `quint.yml`, and neither mentions `forge`. This is HAZ-xfg9cz's real shape:
   not a legitimately-zero field, which the contract forbids, but a
   load-bearing control implemented outside this codebase with no gated
   evidence, which a later contract revision could remove without any gate here
   noticing. ISO 14971 allows such a control; it has to be recorded as one, with
   the note that its implementation lies outside this unit.

7. **The storage layout the client hard-codes is confirmed against the
   contract.** `orgs` is the contract's only state variable and is `private`,
   so it sits at slot 0 and has no getter — which is why the slot must be read
   directly at all. `mapping(address => OrgState)` with a three-field struct at
   `S`, `S+1`, `S+2` matches `solidity_mapping_slot(admin, 0)` and
   `increment_slot`. The event shapes match too: `GenesisInitialized` indexes
   only `admin` (two topics, 64 bytes of data) and `RootUpdated` indexes `admin`
   and `epoch` (three topics, 96 bytes), exactly what `parse_genesis` and
   `parse_root_updated` require. The contract keys each organisation on
   `msg.sender`, so no caller can write another organisation's slot.

Two facts were established rather than assumed:

- **`get_org_state(_, None)` does read at the latest finalised block.** subxt
  0.50.1 documents `at_current_block` as "the current finalized block _at the
  time of instantiation_"
  (`subxt-0.50.1/src/client/online_client.rs:269`). So on-chain-client's
  doc-comment is right and the contradicting comment in
  `org-node/src/chain_read.rs` ("current best") is wrong. That contradiction is
  the open report in org-node's own ledger; this change does not touch it, and
  cites it by file rather than by ID, because a provider may not name a
  consumer's items.
- **Chopsticks finalises every `dev_newBlock` immediately.** Recorded in
  `on-chain-client/tests/reorg_cancels_proposed.rs`. So no chopsticks test can
  be evidence about real finality; the only such test is the `#[ignore]`d
  live-Paseo `smoldot_smoke`.

## Where the evidence has to move, and why

`test_paths` for this unit is `on-chain-client/tests`. Every existing unit test
lives in `#[cfg(test)]` modules inside `on-chain-client/src` — twenty-three of
them, all passing, none visible to `check-trace.sh`. An annotation written
there would be read by no gate, which is the exact false-green the org-node
half of this tooth was written to remove.

So the tests move. Nothing is deleted: each relocated test keeps its assertion
and its independent recomputation, gains a `verifies:` annotation, and is
joined by the class C abnormal-input cases its requirement needs. The `src`
copies are removed in the same task that adds the relocated one, so the two
never both exist.

Most of what needs testing is already public (`h160_of`,
`decode::dispatch::for_runtime`, the `Decoder` trait). Only `client.rs`'s
internals are not, which is what the new `test-support` feature exposes.

## Tasks

### T1 — expose what the tests need, and extract the reorg rule

**Files touched:** `on-chain-client/Cargo.toml`, `on-chain-client/src/lib.rs`,
`on-chain-client/src/client.rs`
**Parallel:** no (first; T2–T8 all read what it exposes)

Two changes, both behaviour-preserving.

**1a. A `test-support` feature.** In `on-chain-client/Cargo.toml`, under
`[features]`:

```toml
# Exposes crate internals to the annotated integration tests in
# on-chain-client/tests, which is where this unit's gate reads its evidence
# (test_paths in on-chain-client/.guardrails/config.yaml). Not part of the
# crate's supported API; off by default, and never enabled by a consumer.
test-support = []
```

In `on-chain-client/src/lib.rs`, after the existing `pub use` block:

```rust
/// Internals exposed only for this unit's annotated integration tests. See
/// the `test-support` feature's comment in `Cargo.toml`. Compiled only when
/// that feature is on, so the crate's default build and the `--lib` clippy
/// gate in CI are unaffected.
#[cfg(all(feature = "test-support", feature = "client"))]
pub mod test_support {
    pub use crate::client::internals::{
        ScanStep, increment_slot, scan_step, solidity_mapping_slot,
    };
}
```

**Corrected during execution.** The first draft of this snippet re-exported
the four items as `pub(crate)`, which does not compile: `error[E0364]` — a
`pub(crate)` item cannot be re-exported publicly, and `E0365` for the type.
The four therefore live in a `pub(crate) mod internals` inside `client.rs`,
declared `pub` within it. A `pub` item inside a non-public module is not
reachable from outside the crate, so with the feature off the crate's public
API is exactly what it was — the guarantee `pub(crate)` was there to give —
while `lib.rs` can legally re-export them. `ScanStep` gains
`#[derive(Debug, PartialEq, Eq)]`; its `BlockRef`/`BlockHash` fields are
already public types. The import path downstream tasks use,
`on_chain_client::test_support`, is unchanged.

**1b. Extract the best lane's step decision into a pure function.** The
decision now lives inside the `.scan()` closure in `best_lane`, where nothing
can reach it. Move it out verbatim — the closure keeps only the `Err` arm and
the call:

```rust
/// The best lane's per-notification decision, extracted from `best_lane`'s
/// `scan` closure so it can be tested without a chain. Given the last best
/// head actually processed and the new head's (number, hash, parent), returns
/// what to emit and advances `last`. Pure apart from that advance.
pub(crate) fn scan_step(
    last: &mut Option<BlockRef>,
    n: u64,
    h: BlockHash,
    p: BlockHash,
) -> ScanStep {
    match *last {
        // Dedup: same hash as the last head we processed — emit nothing,
        // don't advance.
        Some(prev) if prev.hash == h => ScanStep::Skip,
        _ => {
            let reorged = match *last {
                Some(prev) if n <= prev.number || (n == prev.number + 1 && p != prev.hash) => {
                    Some(prev)
                }
                _ => None,
            };
            let from = match *last {
                Some(prev) if n > prev.number => prev.number + 1,
                _ => n,
            };
            *last = Some(BlockRef { hash: h, number: n });
            ScanStep::Block { reorged, from, to: n }
        }
    }
}
```

and in `best_lane`:

```rust
let step: ScanStep = match block_res {
    Err(e) => ScanStep::Error(format!("block: {e}")),
    Ok(block) => scan_step(
        last,
        block.number(),
        BlockHash(block.hash().0),
        BlockHash(block.header().parent_hash.0),
    ),
};
```

**Verify:** `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --manifest-path
on-chain-client/Cargo.toml --lib` — still `23 passed; 0 failed`, and
`cargo clippy --manifest-path on-chain-client/Cargo.toml --lib -- -D warnings`
exits 0 (this is the gate CI runs; `[lints.clippy]` denies `unwrap_used`,
`expect_used` and `panic` at the crate root).

Then, with the feature on:
`cargo build --manifest-path on-chain-client/Cargo.toml --features test-support`
— compiles.

### T2 — the contract-address filter (the fix)

**Files touched:** `on-chain-client/src/state.rs`,
`on-chain-client/src/decode/mod.rs`,
`on-chain-client/src/decode/v_paseo_ah.rs`, `on-chain-client/src/client.rs`,
`on-chain-client/tests/fuzz_event_round_trip/fuzz_target.rs`,
`on-chain-client/tests/contract_address_filter.rs`
**Parallel:** no (serial, after T1)

**Trace IDs:** REQ-9vwcwc (implements RC-5e3bdk).

The parser must surrender the address it currently drops. In `src/state.rs`:

```rust
/// A decoded `OrgRegistry` event together with the H160 of the contract that
/// emitted it. The address is carried out of the decoder rather than dropped
/// because it is the only thing that distinguishes a genuine OrgRegistry log
/// from one any other contract can emit with the same signature hash and the
/// same indexed admin (HAZ-werm85).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmittedEvent {
    pub contract: [u8; 20],
    pub event: Event,
}
```

`Decoder::parse_revive_event` returns `Result<Option<EmittedEvent>, DecodeError>`;
`v_paseo_ah`'s impl binds `let contract: [u8; 20]` instead of `let _contract`
and wraps each parsed event. In `client.rs`'s `decode_contract_events`, replace
the `event_matches_contract` call with a direct comparison and **delete the
stub**:

```rust
let emitted = match decoder.parse_revive_event(payload) {
    Ok(Some(e)) => e,
    Ok(None) => continue,
    Err(e) => { out.push(Err(ClientError::Decode(e))); continue; }
};
// The decisive check: a log is this organisation's only if the contract that
// emitted it is the one this reader was constructed for. Any contract can
// emit a log carrying our signature hash and a victim's indexed admin
// (HAZ-werm85), so pallet+variant filtering is not enough.
if emitted.contract != contract {
    continue;
}
let parsed = emitted.event;
```

Correct the two doc-comments the defect leaves false: `parse_revive_event`'s
"`Ok(None)` for events from other contracts (signature mismatch on
`topics[0]`)" becomes "`Ok(None)` for a log whose first topic matches no
signature this decoder knows; filtering by emitting contract is the caller's
check, on the address returned in `EmittedEvent`", in both `decode/mod.rs` and
`v_paseo_ah.rs`.

`fuzz_event_round_trip` already generates a `contract` field and currently
discards it in the comparison; its expectation becomes
`Some(EmittedEvent { contract, event: expected })`, so the round-trip property
now covers the address too.

**The test** — `on-chain-client/tests/contract_address_filter.rs`, new target:

```rust
//! verifies: REQ-9vwcwc
//! The decoder surrenders the emitting contract's address, so a log from any
//! other contract can be told apart from this organisation's own.
```

Three cases: a log from the configured contract decodes and reports that
address; a log from a different contract with a *valid* OrgRegistry signature
and a valid indexed admin decodes but reports the other address, so the
comparison rejects it (the abnormal-input case — this is the spoofing attempt);
and the address is reported byte-for-byte for a `RootUpdated` as well as a
`GenesisInitialized`.

**Red by mutation:** revert the `emitted.contract != contract` comparison to
the old unconditional `true` and watch the spoofing case fail on the reported
address; restore with `git checkout -- on-chain-client/src/client.rs`.

**Verify:** the two `--lib` and clippy commands from T1, plus
`cargo test --manifest-path on-chain-client/Cargo.toml --features test-support
--test contract_address_filter --test fuzz_event_round_trip`.

### T2b — make the filter decision reachable by a gate

**Files touched:** `on-chain-client/src/client.rs`,
`on-chain-client/tests/log_ownership.rs`
**Parallel:** yes (with T3 and T9 — disjoint files; all after T2)

**Trace IDs:** REQ-9vwcwc, REQ-nygs7k (both implement RC-5e3bdk).

**Added during execution, because T2 measured that its own fix is unprotected.**
T2 followed the plan's red-by-mutation instruction — disable the
`emitted.contract != contract` comparison and watch the spoofing case fail —
and it did not fail: `--test contract_address_filter` still reported 3 passed
with the comparison switched off. The reason is structural. T2's tests verify
the *decoder's* half (the emitting address is carried out rather than dropped,
and reported byte-for-byte), which is real and necessary. The *caller's* half —
the comparison that actually discards the impostor — sits inside
`async fn decode_contract_events`, which needs a chain, so no gated test
reaches it. Every chopsticks target deploys exactly one OrgRegistry and hands
its address to `from_client`, so not one of them has a second contract to spoof
from either.

Left there, this change would fix the spoofing hole and leave the fix's
decisive line unguarded: anyone could delete it and every gate would stay
green. That is the same false-green shape the whole tooth exists to remove, so
it gets the same remedy T1 applied to the reorg rule — extract the decision
into a pure function the tests can reach.

In `client.rs`'s `internals` module:

```rust
/// Whether a decoded log belongs to the Organisation this reader watches: the
/// contract that emitted it must be the one the reader was constructed for,
/// and — when a filter is set — the event's admin must be the one filtered on.
/// Extracted from `decode_contract_events` so the decision can be tested
/// without a chain: the comparison it replaces was unreachable from every
/// gated test, which is HAZ-werm85's residual and was measured, not assumed.
pub fn log_is_ours(
    emitted: &EmittedEvent,
    configured_contract: &[u8; 20],
    admin_filter: Option<OrgAdmin>,
) -> bool {
    if emitted.contract != *configured_contract {
        return false;
    }
    match admin_filter {
        None => true,
        Some(wanted) => *event_admin(&emitted.event) == wanted,
    }
}
```

`event_matches_admin` becomes `event_admin`, returning the borrowed admin so
both callers share one match over the two event shapes. In
`decode_contract_events` the contract comparison and the `admin_filter` block
collapse into one call:

```rust
if !log_is_ours(&emitted, &contract, admin_filter) {
    continue;
}
out.push(Ok(wrap(emitted.event, block_ref)));
```

Behaviour is identical — the contract check already preceded the admin check,
and `None` already meant "no filter".

The admin filter gets the requirement it never had. It is behaviour with a
safety consequence and no item: a consumer that subscribes with
`Some(admin)` and receives another Organisation's events would form a
membership belief about the wrong Organisation. That is REQ-nygs7k, and writing
it is also what keeps the independent review from finding unmarked derived work
here.

**The test** — `on-chain-client/tests/log_ownership.rs`, reaching `log_is_ours`
through `on_chain_client::test_support`:

- `verifies: REQ-9vwcwc` — a log from the configured contract with no filter is
  ours; **the spoof** (a valid signature, a valid indexed admin, a different
  emitting contract) is not; a contract differing in exactly one byte, at the
  first and at the last position, is not; and the contract check dominates, so
  an impostor whose admin *matches* the filter is still rejected.
- `verifies: REQ-nygs7k` — with a filter set, a matching admin is ours and a
  non-matching one is not, for both `Genesis` and `Update`; with no filter, any
  admin from the configured contract is ours.

**Red by mutation, and this time observable — which is the point of the task.**
Replace the contract comparison's `!=` with `==` and watch the spoof cases fail;
separately make `admin_filter` ignored (`_ => true`) and watch the non-matching
admin case fail. Both restored with
`git checkout -- on-chain-client/src/client.rs` before staging.

**Verify:** `--features test-support --test log_ownership`; `--lib` unchanged at
23; `clippy --lib` and `clippy --lib --features test-support`, both
`-D warnings`, exit 0; `cargo check --all-targets` exit 0, since
`decode_contract_events` is compiled by the nine ungated targets.

### T3 — the event decoder's shape checks

**Files touched:** `on-chain-client/src/decode/v_paseo_ah.rs`,
`on-chain-client/tests/decode_revive_event.rs`
**Parallel:** no (serial, after T2 — same file)

**Trace IDs:** REQ-wnjz9j, REQ-88fp2h, REQ-twdu84 (implement RC-6gfh8d),
REQ-axcxf7 (implements RC-8w9wtp).

Relocate the seven event-decoder tests out of `v_paseo_ah.rs`'s `#[cfg(test)]`
module into `on-chain-client/tests/decode_revive_event.rs`, delete them from
`src` in the same commit, and add the abnormal-input cases each requirement
needs. Annotations, one per test:

- `verifies: REQ-wnjz9j` — unknown first topic yields nothing; no topics at all
  yields nothing.
- `verifies: REQ-88fp2h` — `GenesisInitialized` with one topic, with three
  topics; `RootUpdated` with two topics; each of the two with a data length one
  byte short and one byte long. The error names both counts.
- `verifies: REQ-twdu84` — a non-zero byte anywhere in the indexed address
  topic's twelve-byte padding is rejected; all twelve zero is accepted.
- `verifies: REQ-axcxf7` — one trailing byte after a well-formed payload is
  rejected; so is a payload truncated inside its `topics` length prefix.

The signature constants are recomputed in the test from the Solidity canonical
signature strings via `tests/fuzz_support/mod.rs`'s `sig_genesis()` /
`sig_root_updated()`, not imported from `src` — an independent derivation, and
it keeps the ABI-drift check that `event_signatures_match_solidity_abi`
performed. That test relocates too, under `verifies: REQ-wnjz9j`.

**Red by mutation:** for REQ-twdu84, change `unpack_address_topic`'s
`topic[..12]` guard to `topic[..11]` and watch the byte-11 case pass wrongly;
for REQ-88fp2h, change `parse_genesis`'s `data.len() != 64` to `< 64`.

**Verify:** `--lib` (now 16 passed, seven tests having moved out), clippy, and
`--features test-support --test decode_revive_event`.

### T4 — the state decoder's bounds

**Files touched:** `on-chain-client/src/decode/v_paseo_ah.rs`,
`on-chain-client/tests/decode_org_state.rs`
**Parallel:** no (serial, after T3 — same file)

**Trace IDs:** REQ-4astjb, REQ-9wwenn (implement RC-sxjnx9).

Relocate the four state-decoder tests and add the class C boundary cases:

- `verifies: REQ-4astjb` — 95, 97, 0, 32 and 64 bytes each rejected with
  `StorageLengthMismatch` naming both lengths; exactly 96 accepted.
- `verifies: REQ-9wwenn` — a non-zero byte in each of the epoch slot's leading
  24 positions rejected as `EpochOverflow`; `u64::MAX` accepted whole; zero
  accepted.

**Red by mutation:** change `bytes.len() != 96` to `< 96` and watch the 97-byte
case fail; change `bytes[..24]` to `bytes[..23]` and watch the byte-23 case
fail.

**Verify:** `--lib` (12 passed), clippy, `--test decode_org_state`.

### T5 — runtime-version dispatch

**Files touched:** `on-chain-client/src/decode/dispatch.rs`,
`on-chain-client/tests/runtime_version_dispatch.rs`
**Parallel:** yes (with T6, T7 — disjoint files; all after T1)

**Trace IDs:** REQ-hd6m9d (implements RC-d7r82e).

Relocate the two dispatch tests and add the neighbours: `SPEC_VERSION - 1`,
`SPEC_VERSION + 1`, `0` and `u32::MAX` each refused with
`UnsupportedRuntime { spec_version }` carrying the version asked for, and the
pinned version resolving to a decoder that decodes a known-good 96-byte blob —
so the test proves *which* decoder was selected, not merely that some
`&dyn Decoder` came back.

**Red by mutation:** widen the match arm to
`PASEO_AH_SPEC_VERSION | _ if true => ...` and watch the neighbour cases fail.

**Verify:** `--lib` (10 passed), clippy, `--test runtime_version_dispatch`.

### T6 — the AccountId32 → H160 mapping

**Files touched:** `on-chain-client/src/h160.rs`,
`on-chain-client/tests/h160_mapping.rs`
**Parallel:** yes (with T5, T7)

**Trace IDs:** REQ-tkhe3u, REQ-rz7fja (implement RC-mugta4).

Relocate the three `h160.rs` tests. Each already recomputes its expectation
independently; keep that. Add:

- `verifies: REQ-tkhe3u` — all twelve marker bytes `0xEE` takes the reverse
  path; the returned twenty bytes are the input's first twenty even when those
  bytes are themselves `0xEE` (the all-`0xEE` AccountId32 — the boundary where
  the two paths are hardest to tell apart).
- `verifies: REQ-rz7fja` — eleven of twelve marker bytes takes the forward
  path (the relocated near-miss), and so does each single non-`0xEE` byte at
  the twelve marker positions, tested one position at a time.

The test's header records what this control does **not** cover: it pins the
implementation against an independent recomputation of *our* reading of
pallet-revive, not against the runtime's own answer. The runtime is the only
authority, and the test that consults it — `tests/p_address_is_orgid.rs` —
runs in no gate. That is not-minted control 1 and part of HAZ-v2cmtx's
residual.

**Red by mutation:** change `account_id_32[20..32]` to `[21..32]` and watch the
eleven-of-twelve case fail.

**Verify:** `--lib` (7 passed), clippy, `--test h160_mapping`.

### T7 — the Solidity storage-slot derivation

**Files touched:** `on-chain-client/src/client.rs`,
`on-chain-client/tests/storage_slot_layout.rs`
**Parallel:** no (serial, after T2 — both edit `src/client.rs`)

**Trace IDs:** REQ-9m2rnd, REQ-xudf25 (implement RC-5ejucb).

The three `client.rs` tests relocate here, reaching the functions through
`on_chain_client::test_support`. `client.rs`'s `#[cfg(test)]` module is deleted
in this task's commit (T1 exposed the functions; nothing else in `src` reads
them), and with it the `use tiny_keccak::{Hasher, Keccak}` that T1 moved into
that module as its last consumer. That deletion is why this task cannot run
beside T2: both edit `src/client.rs`.

- `verifies: REQ-9m2rnd` — the relocated known vector, plus a second admin
  address, each recomputing `keccak256(pad12 || admin || u256_be(slot))` in the
  test; and a non-zero mapping slot index, so the formula's second operand is
  exercised rather than always zero.
- `verifies: REQ-xudf25` — offset 0 is the identity (relocated); the relocated
  `0xfe + 3` carry; a carry that propagates through four bytes
  (`…ff ff ff ff` + 1); and offset 255, the widest a `u8` can ask for.

**Red by mutation:** drop the `carry == 0` early break in `increment_slot` —
which is behaviour-preserving, so instead change `sum >> 8` to `0` and watch
the four-byte-carry case fail.

**Verify:** `--lib` (4 passed), clippy, `--features test-support --test
storage_slot_layout`.

### T8 — the best lane's reorg rule

**Files touched:** `on-chain-client/tests/best_lane_reorg_rule.rs`
**Parallel:** no (serial, after T1 — reads what T1 extracted)

**Trace IDs:** REQ-ntn4ss, REQ-gr2ver, REQ-5zux82 (implement RC-kemv75).

The first tests this logic has ever had. Reaches `scan_step` and `ScanStep`
through `on_chain_client::test_support`.

- `verifies: REQ-gr2ver` — a repeated head (same hash as the last processed)
  returns `Skip` and leaves `last` unchanged, so a depth-1 reorg cannot re-emit
  the replacement block's events twice.
- `verifies: REQ-ntn4ss` — a new head at `prev.number + 1` whose parent is
  `prev.hash` reports no reorg; the same height with a *different* parent
  reports `Reorged { discarded: prev }`; a head at `prev.number` (a rewind)
  and one below it both report it; and the discarded reference carries prev's
  hash **and** number, since a consumer keyed on the hash needs both.
- `verifies: REQ-5zux82` — from a seed at height 10, a head at 13 backfills
  `from: 11, to: 13`; a head at 11 backfills `11..=11`; a rewind to 9
  backfills `9..=9`; and the first head after a seed at the same height
  backfills that height alone. Ascending order is the `from..=to` contract the
  caller iterates.

The header records the two things the rule cannot do, both of which are open
reports rather than assertions here: a reorg deeper than the notification gap
is invisible to it (PR-qpp28h), and the span it returns for the first
notification after a seed at the finalised head is unbounded (PR-uq5r97). It
also records that this test covers the *decision*, and that whether subxt
delivers the notifications the decision is made on is checked only by
`tests/reorg_cancels_proposed.rs`, which runs in no gate.

**Red by mutation:** change `n <= prev.number` to `n < prev.number` and watch
the same-height rewind case fail; change `prev.number + 1` in the `from`
computation to `prev.number` and watch the backfill case fail.

**Verify:** `--features test-support --test best_lane_reorg_rule`.

### T9 — the fuzz targets' annotations

**Files touched:** `on-chain-client/tests/fuzz_decode_org_state/fuzz_target.rs`,
`on-chain-client/tests/fuzz_parse_revive_event/fuzz_target.rs`,
`on-chain-client/tests/fuzz_event_round_trip/fuzz_target.rs`
**Parallel:** no (serial, after T2 — T2 edits the third file)

**Trace IDs:** REQ-sx5b6g (implements RC-8w9wtp).

Three targets that already run in `verify_commands` and carry no annotation, so
`check-trace.sh` credits them with nothing. Add `verifies: REQ-sx5b6g` to each
header, and to each header the honest statement of what a `harness = false`
bolero run reports: iteration totals and an exit reason, not a pass count, and
one second of generated inputs per target under the gate's default engine — a
smoke depth, with libFuzzer depth a separate explicit invocation. This is the
same correction org-members' register carries about its own targets.

No source change and no red-by-mutation cycle: the requirement is
never-panics-on-arbitrary-bytes, which these targets already exercise. The
attestation for REQ-sx5b6g is therefore that the property is *pre-existing and
measured*, not that a test was watched failing — stated as such in the record
rather than dressed as a red→green.

### T10 — wire the new targets into the gate, the coverage measurement and CI

**Files touched:** `on-chain-client/Cargo.toml`,
`on-chain-client/.guardrails/config.yaml`, `Makefile`,
`.github/workflows/rust.yml`
**Parallel:** no (last of the code tasks)

Seven `[[test]]` entries, each `required-features = ["test-support"]` — cargo
refuses an explicit `--test` whose required features are off, which is how a
target named in the gate cannot silently not run (the org-node lesson from
tooth 1):

```toml
[[test]]
name = "contract_address_filter"
required-features = ["test-support"]
```

and the same for `decode_revive_event`, `decode_org_state`,
`runtime_version_dispatch`, `h160_mapping`, `storage_slot_layout`,
`best_lane_reorg_rule`.

`verify_commands` becomes one line, `--features test-support` added and the
seven targets named:

```
cargo test --manifest-path on-chain-client/Cargo.toml --features test-support --lib --test fuzz_decode_org_state --test fuzz_parse_revive_event --test fuzz_event_round_trip --test contract_address_filter --test decode_revive_event --test decode_org_state --test runtime_version_dispatch --test h160_mapping --test storage_slot_layout --test best_lane_reorg_rule
```

The same line replaces on-chain-client's step in `rust.yml`'s `test` job. The
`clippy` job is unchanged: `--lib` does not compile `test-support`, so the
exposure cannot regress the panic-freedom gate — which is the mistake the
org-node half of this tooth made and had to fix after a reviewer found it.

**The coverage measurement must move with the tests, and this is the half of
the task that can fail.** `coverage-on-chain-client` in the `Makefile` measures
`--lib` plus the same three fuzz targets, so it and `verify_commands` have
always measured the same thing. T3–T7 relocate twenty-three tests out of
`#[cfg(test)]` modules inside `on-chain-client/src`, and both halves of that
move push the `--lib` figure down: the relocated test bodies were themselves
library lines under `--lib` (fully covered, so they inflated the numerator),
and the library branches they exercised — the topic-count, data-length,
address-padding and length-mismatch arms above all — stop being reached by
`--lib` at all. The floors are `ON_CHAIN_CLIENT_LINES := 52` and
`ON_CHAIN_CLIENT_REGIONS := 58` against a measurement of 53.57 / 59.19, which
is about five covered lines of slack. So the relocation would fail the coverage
gate if the measurement were left alone.

Add the seven new targets to the `coverage-on-chain-client` recipe, in the same
order as `verify_commands`, so the two keep measuring the same suite. Then
measure:

```sh
CARGO_HOME=/tmp/cargo_home_fuzz make coverage-on-chain-client
```

and record the two figures in the verification record and in the config's
comment. The Makefile's rule about the floors is not negotiable and is the
reason this is a task step rather than a footnote: **raise a floor when the
real figure rises, never lower one to make a merge pass.** So there are exactly
two acceptable outcomes. If the figures rise — the expected outcome, since the
new targets reach error arms nothing reached before — raise both floors to one
point below the new measurement, matching the existing calibration rule and its
stated reason. If either figure falls, do **not** touch the floors: stop, report
the measurement, and leave the decision to the user, because lowering a floor
"belongs in a verification record with a reason, not in a hurry".

`cargo-llvm-cov` may not be installed here. If `make coverage-on-chain-client`
cannot run at all, that is not a licence to skip the step: report it as
unmeasured, leave both floors exactly as they are, and say so — an unmeasured
coverage gate is a gap for the record and for `verify-before-merge` check 5,
which is the user's call and no-one else's.

**Verify:** the full `verify_commands` line, the coverage command (or the
reason it could not run), and `git diff` on the config and the Makefile showing
no entry removed and no floor lowered.

### T11 — the register, the requirements ledger, the problem ledger, the glossary

**Files touched:**
`on-chain-client/docs/risk/DRAFT-worktree-guardrails-on-chain-client-risk-on-chain-client-hazards.md`,
`on-chain-client/docs/requirements/DRAFT-worktree-guardrails-on-chain-client-risk-chain-reading.md`,
`on-chain-client/docs/problems/DRAFT-worktree-guardrails-on-chain-client-risk-on-chain-client-problems.md`,
`on-chain-client/docs/CONTEXT.md`
**Parallel:** no (last; it describes what T1–T10 built)

#### The six hazards, as confirmed in the interview

Severities and probabilities were put to the owner once the list was drafted and
**confirmed as written**. Every hazard is S3 by the class C ADR's pathway: this
unit decides what the chain says the membership is, and a wrong or stale reading
feeds every consumer's access decision, which reaches the same serious-injury
pathway `docs/adr/2026-09-05-units-and-per-unit-classes.md` names. P2 is an
ordinary event needing no deliberate act; P1 needs a deliberate act plus a
precondition.

| Hazard | One-line shape | S/P | Controls |
|---|---|---|---|
| HAZ-werm85 | A log another contract emitted, accepted as this Organisation's own | S3/P2 | RC-5e3bdk |
| HAZ-8s5chy | A runtime whose layout this reader cannot decode, read anyway | S3/P2 | RC-d7r82e |
| HAZ-95sc43 | Hostile or malformed chain bytes crash or mislead the reader | S3/P2 | RC-6gfh8d, RC-8w9wtp |
| HAZ-v2cmtx | The Organisation's slot computed differently from the one the chain keeps | S3/P1 | RC-mugta4, RC-5ejucb |
| HAZ-xfg9cz | A partial or wrong-width storage read decoded as an Organisation state | S3/P2 (was P1) | RC-sxjnx9 |
| HAZ-xd4urb | An uncommitted observation acted on as committed | S3/P2 | RC-kemv75 |

P-values, argued rather than asserted. **werm85 P2**: permissionless — anyone
may deploy a contract, the indexed admin is public on-chain, and no precondition
beyond that is needed. **8s5chy P2**: Paseo AH upgrades routinely and no
deliberate act is involved; the decoder is pinned at construction and never
re-checked. **95sc43 P2**: any contract can emit a log, and the payload is
attacker-chosen bytes by construction. **v2cmtx P1**: needs an upstream
pallet-revive mapping change or a contract storage-layout change. **xfg9cz P1**:
needs a legitimately-zero field or a non-atomic contract write, both of which
`on-chain/src/OrgRegistry.sol` currently forbids. **xd4urb P2**: reorgs are
ordinary chain behaviour.

**Amended in review round 1, and the register is the authority on both.**
Finding-8: the probability scale above is qualified to "no deliberate act **by a
party whose cooperation the situation requires**", which is the reading that
reproduces every estimate in the file rather than the two readings the file was
silently using. Finding-2: **HAZ-xfg9cz is P2, not P1.** Its situation names
three routes and the P1 argument covered only two of them; the third — the
runtime API's answer changing shape — is the same ordinary event HAZ-8s5chy
assesses at P2, and the probability of a disjunction is the maximum over its
routes. The conclusion is unchanged, because the matrix makes S3 unacceptable at
P1 and P2 alike. The interview's severities and probabilities are recorded above
as they were confirmed; these two amendments came later, from the review, and
are argued in the register where each hazard stands.

#### The nine controls, and the requirements that realise them

Each control names its hazard with `mitigates:`; each requirement below carries
`(implements: RC-…)` and `satisfies: derived`. All twenty-one are derived — they
arise from how the system is partitioned and what this reader was built to do,
not from a written system-needs document — so **every one must be named in the
register's derived-requirements assessment** or `check-trace.sh` reports
`UNANALYZED-DERIVED`.

| Control | What it requires of the software | Requirements |
|---|---|---|
| RC-5e3bdk | A log reaches a subscriber only if the contract that emitted it is the one the reader was constructed for, and, where a filter is set, the event's admin is the one filtered on; the emitting address is reported with every decoded event | REQ-9vwcwc, REQ-5upq6n, REQ-nygs7k |
| RC-6gfh8d | A log whose topic count, data length or indexed-address padding does not match the signature it claims is refused with a typed error naming both the expected and the actual value; a well-formed log decodes to every field it carried | REQ-wnjz9j, REQ-88fp2h, REQ-twdu84, REQ-n6v896 |
| RC-8w9wtp | Any byte sequence yields a typed `Ok`/`Err`, never a panic or an abort | REQ-axcxf7, REQ-sx5b6g |
| RC-d7r82e | State is read only through a decoder compiled for the runtime version actually reported, and an unrecognised version is refused rather than guessed | REQ-hd6m9d |
| RC-mugta4 | The Organisation identifier is derived from an AccountId32 by exactly one of pallet-revive's two documented cases, selected on the full twelve-byte marker | REQ-tkhe3u, REQ-rz7fja |
| RC-5ejucb | The storage key for an Organisation's slot is derived by the Solidity mapping formula, with consecutive struct fields at consecutive slots | REQ-9m2rnd, REQ-xudf25 |
| RC-675a3h | Exactly the two event signatures the deployed contract declares are recognised, each derived from its canonical Solidity signature string, so an ABI drift is detected rather than silently changing which logs are read | REQ-52uc8f |
| RC-sxjnx9 | A storage blob is decoded only at exactly the expected width, an epoch outside the representable range is refused rather than truncated, and the public newtypes hold the widths the ABI gives them | REQ-2qa5r5, REQ-4astjb, REQ-9wwenn |
| RC-kemv75 | A best-block observation is reported as provisional, a discarded head is reported to the consumer with the identity of what was discarded, and no skipped height is silently omitted | REQ-ntn4ss, REQ-gr2ver, REQ-5zux82 |

Requirement text is written in the ledger, one behaviour each, in the form
`**<ID>**: The software shall <single, testable behaviour>. (implements: RC-…)`.
Each is already verified by the tests T1–T10 landed; the `verifies:`
annotations are in place and were confirmed to reach `check-trace.sh`, which
reports them as `DANGLING-REF` until this task mints the items.

#### The five problem reports

| ID | Shape | Status |
|---|---|---|
| PR-p5ngya | Events were never filtered by emitting contract, while the code's own doc-comment claimed they were | **resolved** by this change (T2, T2b) |
| PR-w5sk5k | The decoder is pinned at construction and never re-checked, though the doc-comment says the client "should be reconstructed" on a runtime upgrade and nothing enforces it | open |
| PR-h4mb8y | `src/verify.rs` is seven lines of doc-comment and no code, so the module the design names as closing the loop with `org-members` contains nothing | open |
| PR-uq5r97 | The best lane's first backfill span is unbounded, seeded from the finalised head which on a live chain lags the best tip | open |
| PR-qpp28h | A reorg deeper than the notification gap is undetectable, by the code's own admission | open |

Four open against `problem_open_max: 10`. Each open report needs `status: open`
and `opened: 2026-09-10`; PR-p5ngya needs `status: resolved` and the commit that
resolved it. Each open one also gets a dated item on the setup checklist, age
limit 2026-10-10 under `problem_age_days: 30`.

#### What the register must NOT claim

Five findings from T1–T10 belong in the residual assessments and the not-minted
list, not in the controls, because each is a gap this change leaves open:

1. **The caller's half of RC-5e3bdk was unreachable by any gate** until T2b
   extracted `log_is_ours`. Measured: with the comparison disabled, three tests
   still passed. It is now reachable — deleting the line reds three cases — but
   **no test in this repository exercises the real `decode_contract_events`
   path**, because every chopsticks target deploys one OrgRegistry and has no
   second contract to spoof from. That is HAZ-werm85's residual.
2. **`h160_of` is pinned against an independent recomputation of *our reading*
   of pallet-revive, not against the runtime's own answer.** The test that
   consults the runtime, `tests/p_address_is_orgid.rs`, runs in no gate. That is
   HAZ-v2cmtx's residual and not-minted control 1.
3. **"An absent slot means no such Organisation" rests on a control outside this
   unit.** `on-chain/src/OrgRegistry.sol`'s `ZeroValue()` revert and its
   all-three-fields-at-once write are what make it sound; that directory is
   disclaimed by `.guardrails/units.yaml` and its forge tests run in no CI lane
   here. ISO 14971 permits such a control; it must be recorded as one whose
   implementation lies outside this codebase. That is HAZ-xfg9cz's residual.
4. **`fuzz_event_round_trip` has an empty seed corpus** — measured, and visible
   in the run line, which carries no `corpus inputs:` field where the other two
   report 4 and 6. It has been visible in measured output since the 2026-08-26
   verification record without being written down as a gap. That is part of
   RC-8w9wtp's residual and a not-minted control.
5. **Finality and reorg behaviour have no gated evidence.** Chopsticks finalises
   every `dev_newBlock` immediately, so no chopsticks test can be evidence about
   real finality, and the only test that could be — the `#[ignore]`d live-Paseo
   `smoldot_smoke` — runs nowhere. RC-kemv75 covers the *decision*; whether
   subxt delivers the notifications the decision is made on is unevidenced. That
   is HAZ-xd4urb's residual.

Two further items for the not-minted list, both measured during this change:
**clippy runs `--lib` only**, so the nine new integration targets are outside
the panic-freedom gate that the crate root's `[lints.clippy]` declares; and
**the crate is not `rustfmt`-clean at baseline** with no `cargo fmt` step in the
workflow, which is a decision to record rather than a defect to fix here.

#### The conclusion this analysis must reach

Every assessed risk is S3, and under the RMF's acceptability matrix no S3 risk
at either P1 or P2 is acceptable. So **no assessed risk in this register is
acceptable, and the overall residual risk for the unit is UNACCEPTABLE against
the intended use** — the same conclusion org-members' register reached for the
membership capability and org-node's for the node. State it plainly; do not
soften it, and do not argue any severity down from recoverability, which the
class C ADR withdrew on 2026-09-02 (the argument a reviewer had to remove from
org-node's register in round 6).

The register carries: scope, method and the harm pathway; the six hazards above
with their situations, harms, severities, probabilities, chosen controls and
residual assessments; the hazards each control introduces; the further hazards
in prose; the not-minted controls; the residual-risk table and the overall
conclusion; and the derived-requirements assessment naming every one of the
twenty-one REQs, since all twenty-one are `satisfies: derived` and
`check-trace.sh` reports `UNANALYZED-DERIVED` for any the register never
mentions.

Citation discipline, measured three times on the org-node half: this unit is a
**provider** of org-node and app. It may not name a consumer's HAZ, RC or PR by
ID, and it may name a consumer's REQ only where that REQ is the `expects:` item
addressed to it. So org-node's expectations REQ-ysyu9g and REQ-q92yac are
named — they are addressed to on-chain-client and org-members respectively, and
only the first is this unit's business — while everything else in org-node's or
org-members' ledgers is cited by file path.

The glossary gains the terms this register uses that
`on-chain-client/docs/CONTEXT.md` does not yet define: Organisation slot,
Slot key, Emitting contract, Event signature, Best-block observation,
Finalised observation, Reorg notification, Runtime spec version, Epoch,
Organisation admin. Terms that cross the unit boundary (Organisation, Change
set, Membership root) stay in the root glossary and are not redefined here.

**Verify:** `check-trace.sh` and `check-ids.sh --allow-draft-files` with
`GR_CONFIG=on-chain-client/.guardrails/config.yaml`, both exit 0; then the same
two for every unit in `check-units.sh --impact`'s output, and `check-units.sh`
once.

## Red → green attestations

Filled in as each task's dispatch report lands, so `merge-change` step 6b
copies them from this file rather than from scrollback.

**T1 — done** (task commit `2bd1c98`, merged into the change branch).
`red -> green:` **no cycle applies, and this is the one task where that is the
honest answer.** T1 adds no test: it is the exposure and extraction T2–T8 read,
and its verification is that the existing suite is unchanged by it. Measured:
`cargo test --lib` 23 passed / 0 failed, identical to the pre-edit baseline;
`clippy --lib -- -D warnings` exit 0; `clippy --lib --features test-support --
-D warnings` exit 0; `build --features test-support` exit 0;
`check --all-targets` under default features exit 0 with no warnings. The
subagent additionally wrote a throwaway integration test importing all four
items through `on_chain_client::test_support`, ran it (1 passed) and deleted it
before staging — because `cargo build` cannot prove the exposure resolves from
an *external* crate, which is what T2–T8 depend on. It is not in the commit.
That run also confirmed `state::{BlockHash, BlockRef}` and `types::OrgAdmin`
are already public, so T8 needs nothing further exposed.

**T6 — done** (task commit `8018c2d`, merged into the change branch). Five
tests in `on-chain-client/tests/h160_mapping.rs`; `h160_of` itself byte-for-byte
unchanged. `--test h160_mapping` 5 passed / 0 failed; `--lib` 20 passed / 0
failed (the three relocated `h160::tests` gone from 23 and nothing else);
`clippy --lib` and `clippy --test h160_mapping`, both `-D warnings`, exit 0;
`check --all-targets` exit 0. No `--features test-support` needed — `h160_of`
is unconditionally public at the crate root.

`red -> green:` the implementation predates every one of these tests, so all
four window-pinning cases are red **by mutation**, each watched failing on the
reported bytes before the mutation was restored:

- `forward_path_taken_when_only_eleven_marker_bytes_are_present` — with
  `account_id_32[20..32]` narrowed to `[21..32]`, returned the first twenty
  bytes `[0; 20]` where the test's independently recomputed keccak answer was
  expected.
- `forward_path_taken_for_a_single_non_marker_byte_at_each_marker_position` —
  same mutation, failed at the "byte 20 cleared" position with `[0xEE; 20]`
  instead of the keccak answer.
- `reverse_path_returns_the_first_twenty_bytes` — with the reverse path's early
  return disabled, returned the keccak answer instead of `[0, 1, …, 19]`.
- `reverse_path_taken_when_the_whole_account_is_the_marker_byte` — same
  mutation, returned the keccak of `[0xEE; 32]` instead of `[0xEE; 20]`.

`forward_path_keccaks_then_truncates` is the relocated happy path and is green
under both mutations. That is the correct result for it and is recorded rather
than papered over: it is the normal case, and the four cases above are what pin
the window.

Two things the task changed that the plan did not ask for, both kept. **The
plan's single mutation reds two tests, not one** — `[21..32]` fails both the
eleven-of-twelve near-miss and the per-position sweep at position 20, which is
the sweep earning its place: the near-miss pins byte 20 alone, the sweep pins
all twelve. And **one doc-comment in `src/h160.rs` was corrected**: its module
header promised in the future tense that "a later integration test pins our
implementation against chopsticks-captured ground truth", when that test exists
as `tests/p_address_is_orgid.rs` and runs in no gate. The header now names both
tests and says which one a gate runs — the same fact the new test file records
as part of HAZ-v2cmtx's residual.

**T8 — done** (task commit `c1a32a2`, merged into the change branch). Fourteen
tests in `on-chain-client/tests/best_lane_reorg_rule.rs`, the only file in the
commit; `scan_step` itself unchanged. 14 passed / 0 failed under
`--features test-support --test best_lane_reorg_rule`; `--lib` still 23 passed
/ 0 failed after the final restore.

`red -> green:` the logic pre-dates T8 (T1 extracted it), so red came by
mutation of `scan_step` in `on-chain-client/src/client.rs` — **seven**
mutations, each applied, watched and restored with `git checkout --`, and every
one of the fourteen cases observed failing for the right reason under at least
one:

| Mutation | Cases watched failing, and what was observed |
|---|---|
| M1 `n <= prev.number` → `n < prev.number` (the plan's first named cycle) | `a_rewind_to_the_same_height_reports_the_discarded_head`, `the_first_head_at_the_seed_height_backfills_that_height_alone` — `reorged: None` where `Some(BlockRef { number: 10, .. })` was required |
| M2 `from` = `prev.number + 1` → `prev.number` (the plan's second) | `a_jump_backfills_every_skipped_height`, `a_single_step_backfills_that_height_alone`, `child_of_the_last_head_reports_no_reorg`, `next_height_with_a_foreign_parent_reports_the_discarded_head` — `from: 10` where 11 was required |
| M3 dedup guard narrowed so it never matches | `repeated_head_is_skipped_and_last_is_unchanged` — a spurious `Block { reorged: Some(prev), from: 10, to: 10 }` where `Skip` was required, i.e. a re-emission **and** a false reorg report; `repeated_hash_with_inconsistent_number_is_still_skipped` — `Block { from: 11, to: 42 }` |
| M4 dedup keyed on number instead of hash | `same_height_different_hash_is_not_a_repeat` — `Skip` for a depth-1 **replacement** block, which would drop its events entirely; `the_backfill_span_is_always_ascending` |
| M5 reorg detection disabled (`reorged` always `None`) | `a_rewind_below_the_last_height_reports_the_discarded_head`, `a_rewind_backfills_the_new_height_alone`, `the_discarded_reference_carries_both_hash_and_number` — no reorg report at all |
| M6 the `*last = Some(..)` advance removed | `the_first_notification_reports_no_reorg` — `last` still `None` after a processed head |
| M7 `from` overshoots on a non-extend (`_ => n + 1`) | `the_backfill_span_is_always_ascending` — `from 10..=9`, a descending span that iterates zero times and would silently read no blocks |

M4 and M7 are the two worth carrying into the record: each is a plausible
"simplification" of the rule that loses events **silently** — M4 by discarding a
reorg's replacement block, M7 by producing a span that reads nothing — and
neither would show up as an error anywhere. They are the argument for extracting
this rule in T1.

Four departures from the plan, all kept.

- **No `#![cfg(...)]` guard on the new target**, unlike the nine chain-dependent
  targets beside it, and the header says why. T10 makes `test-support` the
  target's `required-features`, so an explicit `--test` with the feature off is
  a cargo refusal; an inner `cfg` guard would instead compile to an empty binary
  reporting success having run nothing — the false green T10's design note
  exists to prevent. The cost, which nothing in the plan or CI pays, is that
  `--no-default-features --features test-support` is a compile error rather than
  a silent skip.
- **The trace correspondence is stated in the header.** REQ-ntn4ss is worded as
  "reports `Reorged { discarded: prev }`"; at `scan_step`'s level that is
  `ScanStep::Block { reorged: Some(prev), .. }`, and `best_lane`'s `.then()`
  stage is what turns it into `SubscribedEvent::Reorged`.
- **A jump case necessarily pins `reorged: None`.** The plan said the two known
  limitations would be open reports rather than assertions here, but a
  full-struct `assert_eq!` on a jump pins that field. The test asserts it and
  labels it, in both the header and its own doc comment, as *today's* behaviour
  under PR-qpp28h — not as a claim that nothing was discarded.
- **Three abnormal-input cases beyond the plan's list**: the same hash arriving
  with an inconsistent number and parent, the first notification with
  `last == None` (unreachable from `best_lane`, which seeds from the finalised
  head, but the rule must be defined for it), and the ascending-span property at
  the `u64` extremes, where the `+ 1` could wrap.

One thing to know rather than fix: the new target uses `panic!` in two match
arms. `[lints.clippy]` denies `panic` at package level, but CI's clippy job is
`--lib` only and the existing targets in `on-chain-client/tests` already use
`panic!` and `expect` freely, so no `allow` was added and none is needed. A
future decision to run clippy over `--all-targets` would touch those existing
targets too, not just this one.

**T5 — done** (task commit `f59c323`, merged into the change branch). Five
tests in `on-chain-client/tests/runtime_version_dispatch.rs`; `for_runtime`
unchanged. 5 passed / 0 failed; the unit's full `verify_commands` re-run green
(21 lib tests plus the three fuzz targets); `clippy --lib` and
`clippy --test runtime_version_dispatch`, both `-D warnings`, exit 0.

`red -> green:` two mutations were needed, not the one the plan named.

- The plan's mutation, widening the match to `PASEO_AH_SPEC_VERSION | _ if true
  =>`, reddened all four refusal cases, each watched failing with the version it
  was asked about resolving `Ok` instead of being refused:
  `version_one_below_the_pinned_one_is_refused` (2002001),
  `version_one_above_the_pinned_one_is_refused` (2002003),
  `zero_version_is_refused` (0) and `max_version_is_refused` (4294967295).
- **It left the positive case green**, so
  `pinned_version_resolves_to_a_decoder_that_decodes_the_pinned_layout` got its
  own: `PASEO_AH_SPEC_VERSION if false =>`, dropping the pinned arm, watched
  failing with "the pinned spec_version must resolve: `UnsupportedRuntime {
  spec_version: 2002002 }`". That case is the one that proves *which* decoder
  came back rather than that some `&dyn Decoder` did, so letting it pass
  unchallenged would have been the more expensive omission.

Also: a short doc-comment in `dispatch.rs` now records where its tests went and
why. `REQ-hd6m9d` does not exist until T11 mints it, so `check-trace.sh` reports
a dangling reference until then — expected, and not a finding.

**T2b — done** (task commit `0801aa1`, merged into the change branch). Seven
tests in `on-chain-client/tests/log_ownership.rs`. `decode_contract_events`'s
two checks collapsed into one `if !log_is_ours(&emitted, &contract,
admin_filter) { continue; }`, behaviour identical; `event_matches_admin` became
`event_admin(&Event) -> &OrgAdmin` and `internals::log_is_ours` is its only
caller. 7 passed / 0 failed; `contract_address_filter` still 3 passed; `--lib`
18 passed in both feature configurations; `clippy --lib` and
`clippy --lib --features test-support`, both `-D warnings`, exit 0;
`check --all-targets --features test-support` exit 0.

`red -> green:` all seven watched failing **before the implementation existed**,
on `error[E0432]: unresolved import on_chain_client::test_support::log_is_ours`
— the target did not compile because the extracted decision did not yet exist.

**The mutation is now observable, measured three ways** — the whole point of
the task, so the figures are recorded rather than summarised:

| Mutation of `src/client.rs` | Result |
|---|---|
| Contract comparison `!=` → `==` | **6 failed, 1 passed**, including the named `the_spoof_a_valid_log_from_another_contract_is_not_ours` |
| Contract check **deleted outright** — the "anyone could delete the decisive line" case T2 could not detect | **3 failed, 4 passed**: exactly the three rejection cases red, every acceptance case still green. This is the precise false green T2 measured, now red. |
| `admin_filter` ignored (`_ => true`) | **1 failed** — `with_a_filter_set_a_non_matching_admin_is_not_ours` |

Each restored with `git checkout --` before staging.

Three notes. The commit holds **three** files, not the plan's two: `src/lib.rs`
changed by one line to extend the `test_support` re-export, which the dispatch
note asked for; it intersects no other task's files. The plan's "`--lib`
unchanged at 23" was wrong for this branch — the true figure is **18** (23
baseline less T6's three and T5's two), and T2b adds no `#[test]` to `src`. And
**`cargo check --all-targets` without `--features test-support` fails**, because
`tests/best_lane_reorg_rule.rs` cannot see the feature-gated module: pre-existing
from T8, exit 0 with the feature on, and exactly what T10's `required-features`
entries exist to fix. T10 must confirm no gate and no CI step runs
`--all-targets` without the feature.

**T9 — done** (task commit `aee469e`, merged into the change branch). Header
annotation and prose only: each of the three fuzz targets now carries
`//! verifies: REQ-sx5b6g`. No source file touched, `--lib` unchanged at 18,
`clippy --lib -- -D warnings` exit 0, `check --all-targets --features
test-support` exit 0 with zero warnings.

`red -> green:` **no cycle applies, and the plan said why in advance.**
REQ-sx5b6g's property is never-panics-on-arbitrary-bytes, which all three
targets already exercised under `verify_commands` before this change. The
attestation is that the property is pre-existing and **measured**, not that a
test was watched failing, and no mutation was invented to dress it as a
red→green.

**What a `harness = false` bolero run actually reports**, measured twice with
the gate's own command, all six runs exit 0:

| Target | Run 1 | Run 2 | Corpus |
|---|---|---|---|
| `fuzz_decode_org_state` | 182,624 rng inputs | 261,916 | 4 |
| `fuzz_parse_revive_event` | 207,437 rng inputs | 241,990 | 6 |
| `fuzz_event_round_trip` | 23,404 rng inputs | 39,337 | **none** |

Every run ended `exit reason: max duration (1s - default) exceeded`. The
figures are a **wall-clock budget, not a case count**, and no run's number is
reproducible — two runs of the same target differ by 40%. So each header says
that rather than quoting one figure as if it were fixed: a green run reports run
time, iterations/s, corpus inputs, rng inputs and an exit reason, never a pass
count, and green means the process exhausted its second without panicking or
aborting. libFuzzer depth remains a separate explicit invocation that no lane in
this repository runs.

**The seed-corpus fact the register needs, per target.**
`fuzz_decode_org_state` has 4 files (`valid_epoch_7`, `epoch_u64_max`,
`epoch_overflow`, `wrong_length_95`); `fuzz_parse_revive_event` has 6
(`valid_genesis`, `valid_root_updated`, `wrong_topic_count`,
`bad_address_padding`, `trailing_byte`, `empty_topics`);
**`fuzz_event_round_trip` is empty — only `.gitkeep`.** That is visible in the
run line itself, which carries no `corpus inputs:` field where the other two
report 4 and 6. It is unseeded because seeding it is a *different* job:
`tests/regenerate_corpus.rs` seeds "the two raw-byte targets" only, and a file
dropped into this one would be consumed as `TypeGenerator` driver bytes for
`EventShape`, not as a `ContractEmitted` payload. So this is the one live corpus
gap of the three, and it is not neglect of the other two's kind. Every
`crashes/` directory holds only `.gitkeep`: no reproducer has ever been found
and committed.

Worth recording for the register's honesty about its own history: the
2026-08-26 verification record already printed "4 corpus inputs" and "6 corpus
inputs" for two targets and no figure for the third, so this gap has been
visible in measured output since then without ever being written down as a gap.
That is the same shape as the org-node half's finding about evidence that exists
and counts for nothing.

T9 also confirms the annotation reaches the checker: `check-trace.sh` for this
unit now reports `DANGLING-REF REQ-sx5b6g`, alongside the same line for every
other in-flight ID. Expected until T11 mints the requirements, and not a
finding. That the ID is *credited* as a `verifies:` reference — no
`MISSING-TEST REQ-sx5b6g` — can only be confirmed once the item exists, so T11
must check it.

**T3 — done** (task commit `4789446`, merged into the change branch). Nine
tests relocated out of `v_paseo_ah.rs`'s `#[cfg(test)]` module and deleted from
`src` in the same commit; **fifteen** arrive in
`on-chain-client/tests/decode_revive_event.rs`. The plan said seven — the file
held nine, the eight under `// ----- event decoder -----` plus
`event_signatures_match_solidity_abi`. The four `decode_org_state_*` tests are
untouched; they are T4's. 15 passed / 0 failed; `--lib` 9 passed (23 baseline
less T6's 3, T5's 2 and this task's 9); `clippy --lib` and
`clippy --test decode_revive_event`, both `-D warnings`, exit 0; the unit's
current `verify_commands` green with all three bolero targets clean.

`red -> green:` implementation pre-dates every test, so red came **by
mutation** — **thirteen** mutations, each applied, run and restored, with
`v_paseo_ah.rs` byte-verified identical afterwards. Every one of the fifteen
tests was observed failing under at least one, for its own reason:

| Mutation | Observed |
|---|---|
| M1 `topic[..12]` → `[..11]` (the plan's named cycle) | padding sweep fails at **position 11**: `Ok(Some(Genesis { admin: [32..51] }))` where `Err(InvalidAddressTopic)` was required |
| M2 `parse_genesis` `data.len() != 64` → `< 64` | genesis data sweep: `unwrap_err()` on `Ok` at the 65-byte case |
| M3 `parse_root_updated` `!= 96` → `< 96` | root-updated data sweep, same at 97 |
| M4 `topics.len() != 2` → `< 2` | `genesis_with_three_topics_is_rejected` decoded instead |
| M5 `topics.len() != 3` → `< 3` | `root_updated_with_four_topics_is_rejected` decoded instead |
| M6 unknown-signature arm decodes as genesis | `an_unknown_first_topic_yields_nothing` → `Err(InvalidTopicCount)`; the drift-string half of the ABI test decoded |
| M7 `SIG_GENESIS_INITIALIZED[0]` `0x8e`→`0x8f` | **8 tests red**, including the ABI guard and every genesis-shaped case collapsing to `Ok(None)` |
| M8 trailing-byte check removed | `any_trailing_bytes…` accepted one trailing byte |
| M9 topics decode `unwrap_or_default()` | `a_payload_that_ends_inside_its_topics…` → `Ok(None)` at cut 86 |
| **M10** `topics.len() != 2` → `> 2` | `genesis_with_one_topic_is_rejected` — **panicked inside `src`**: `index out of bounds: the len is 1 but the index is 1` at `v_paseo_ah.rs:141` |
| **M11** `topics.len() != 3` → `> 3` | `root_updated_with_two_topics_is_rejected` — same: `len is 2 but the index is 2` at `:169` |
| M12 `SIG_ROOT_UPDATED[0]` `0x24`→`0x25` | 7 tests red, the RootUpdated mirror of M7 |
| M13 empty-topics guard removed | `no_topics_at_all_yields_nothing` → `Err(InvalidTopicCount { expected: 2, actual: 0 })` |

**M10 and M11 matter beyond their red.** They show what the topic-count guards
actually are: the only thing standing between untrusted chain bytes and a
**panic** on `topics[1]` / `topics[2]` in a `no_std` library whose crate root
denies `panic`. The plan's named mutation (`!= 2` → `< 2`) reds only the
too-many side, so the too-few side needed its own — and that is the side where
the failure is a panic rather than a wrong answer. This is direct evidence for
RC-8w9wtp and belongs in HAZ-95sc43's block.

Two departures, both kept. **The ABI-drift test could not keep its original
form**: `SIG_GENESIS_INITIALIZED` and `SIG_ROOT_UPDATED` are `pub(super)` and
invisible to an integration test. It is re-expressed as *behaviour* — a log
topic-hashed from the canonical Solidity string must decode, one hashed from a
plausibly drifted string (a parameter dropped) must not — with
`support::sig_genesis()` / `sig_root_updated()` cross-checked against a locally
recomputed keccak. That is a stronger test than the original constant
comparison, because it exercises the decode path rather than a literal.
`tests/fuzz_support/mod.rs` was read and used, never edited. And two `src`
doc-comments were corrected: the module header now records where
`parse_revive_event`'s tests live and why, matching `dispatch.rs`'s note, and
`SIG_GENESIS_INITIALIZED`'s comment no longer names a `tests::` function that
has left the crate.

**T4 — done** (task commit `f3499ab`, merged into the change branch). Nine
tests in `on-chain-client/tests/decode_org_state.rs`; the four
`decode_org_state_*` tests relocated out of `v_paseo_ah.rs` and its
`#[cfg(test)]` module deleted entirely, since relocating them emptied it. 9
passed / 0 failed; `--lib` 5 passed (23 baseline − 3 T6 − 2 T5 − 9 T3 − 4 T4);
`clippy --lib` and `clippy --test decode_org_state`, both `-D warnings`, exit 0;
`check --all-targets --features test-support` exit 0 with **zero warnings**,
which is what proves no `use` item lost its last consumer when the module went;
T3's `decode_revive_event` re-run at 15 passed, since T4 edited the same source
file; the unit's current `verify_commands` green.

**This task was interrupted mid-flight by a token revocation and resumed.** All
six mutations below were personally observed in a single run *before* the
interruption, and the report states that explicitly rather than inferring any
of them — the resume instruction was to re-witness or say so plainly.

`red -> green:` six mutations, each applied, run, and restored from a pristine
copy with `cmp` confirming byte-identity afterwards (all six reported
`restored: identical`). Every one of the nine tests watched failing under at
least one:

| Mutation | Observed |
|---|---|
| **M1** `bytes.len() != 96` → `< 96` (the plan's named cycle) | `ninety_seven_bytes_is_rejected` decoded `Ok(OrgState { …, epoch: Epoch(0) })` where a refusal was required. 7 passed, 2 failed — **every too-short case stayed green.** |
| **M2** `!= 96` → `> 96` (the mirror the dispatch asked for) | `an_empty_blob_is_rejected_rather_than_read_out_of_bounds`, `ninety_five_bytes_is_rejected` and the slot-count sweep each **panicked inside `src`** at `v_paseo_ah.rs:77:41`, `:80:49` and `:79:43` — the three unguarded `copy_from_slice` calls below the length check. 6 passed, 3 failed. |
| **M3** `bytes[..24]` → `[..23]` (the plan's second cycle) | the epoch sweep red at **position 23**: `Epoch(7)` where `Err(EpochOverflow)` was required |
| **M4** `bytes[..24]` → `[..25]` (the mirror) | `the_largest_u64_epoch_is_accepted_whole` and the big-endian-weight sweep — `EpochOverflow` for a value inside `u64` range |
| **M5** `buf.copy_from_slice(&bytes[24..32])` → `[23..31]` | all three value-reading tests: `Epoch(72057594037927935)` for `u64::MAX`, `Epoch(0)` for the round trip, `Epoch(281474976710656)` for `72057594037927936` |
| **M6** `org_pub_key.copy_from_slice(&bytes[32..64])` → `[0..32]` | `exactly_ninety_six_bytes_decodes_every_field` — `org_pub_key` came back as the root-hash slot |

**M1 and M2 mirror T3's M10/M11 exactly, and that is now a pattern rather than
an incident.** The plan's named mutation reds only the too-long side; the
too-short side is where a wrong guard is a **panic** rather than a wrong
answer, in a `no_std` library whose crate root denies `panic` — and it is
reached by the most realistic abnormal input of the set, the **empty blob** a
storage read returns when it returns nothing. M3/M4 needed the same treatment:
neither reds the other's test, so the epoch window is pinned from both edges or
not at all. Two hazards' evidence rests on this: HAZ-95sc43 (hostile bytes) and
HAZ-xfg9cz (a partial or wrong-width read).

**M6 is a lesson about the relocated tests themselves.** Under the original
uniform `0xaa`/`0xbb` fills that mutation is *invisible* — swapping two
all-identical 32-byte runs changes nothing observable. The relocated test
replaced them with ascending runs, which is what makes the field-order
mutation detectable at all. A test can be green, relocated, annotated and still
blind.

Two tests beyond the plan's list, both kept: the big-endian-weight sweep over
bytes 24..32 (the epoch window's lower edge, the only thing M4 and M5 red) and
`a_whole_number_of_slots_other_than_three_is_rejected` (32/64/128/192 — a caller
that forgot the epoch slot, or concatenated two organisations' reads). One `src`
doc-comment was corrected: the module header referred readers to "the
`#[cfg(test)]` module below", which no longer exists.

**T7 — done** (task commit `fc22f50`, merged into the change branch). Eleven
tests in `on-chain-client/tests/storage_slot_layout.rs`; `client.rs`'s
`#[cfg(test)]` module and the `tiny_keccak` import T1 had moved into it are
gone, `internals`' own import untouched. `solidity_mapping_slot` and
`increment_slot` byte-for-byte unchanged. 11 passed / 0 failed; `--lib` 6 passed
(23 baseline − 3 T6 − 2 T5 − 9 T3 − 3 T7 — T4's four had not yet merged when
this ran); `clippy --lib` and `clippy --lib --features test-support`, both
`-D warnings`, exit 0; `check --all-targets --features test-support` exit 0 with
no warnings.

**This task lost all its work to the token revocation** — its worktree was clean
when it resumed — so everything below was done after the resume, from a re-read
of `src/client.rs` rather than from memory.

`red -> green:` eight mutations of `src/client.rs`, each applied, watched and
restored, the file `diff`-verified identical to a pristine copy afterwards.

| Mutation | Observed |
|---|---|
| **M0 — the plan's own suggestion**: drop `increment_slot`'s `carry == 0` early break | **11 passed, 0 failed — reddens nothing.** Behaviour-preserving, exactly as the plan warned. Recorded as a measured negative, not dressed as a cycle. |
| **M1** `carry = sum >> 8` → `carry = 0` (the plan's substitute) | **4 failed**: the four-byte carry (`…255,255,255,0` where byte 27 = 1 and four zeros were required), the 256-bit wrap (only the low byte cleared), offset 255, and the `0xfe + 3` carry |
| M2 `buf[12..32]` → `buf[0..20]` (right-pad the address) | **4 failed** — every mapping case except the zero admin, whose padding position is immaterial. Correct for it, and recorded rather than papered over. |
| M3 `map_slot` dropped (`let _ = map_slot;`) | **2 failed**, and nothing else — the measured justification for insisting on a non-zero index: `get_org_state` only ever passes 0, so without those two cases the second operand could be deleted invisibly |
| M4 `to_be_bytes` → `to_le_bytes` | **2 failed**, on a dedicated `assert_ne!` |
| M5 offset ignored | **5 failed**, including "orgPubKey sits at S + 1"; `offset_zero_is_the_identity` stayed green — which is why M7 was needed |
| M6 `hasher.update(&buf)` → `&buf[..32]` (hash one word) | **5 failed** — the only mutation that reds the zero-admin case |
| M7 `*byte = sum as u8` → `*byte = 0` | **5 failed**, including the identity case, the last test to be reddened |

**M0 is the entry worth keeping.** My plan named a mutation, warned in the same
breath that it was behaviour-preserving, and named a substitute — and T7 ran the
inert one anyway to confirm it reds nothing before using the substitute. That is
the right instinct: a mutation nobody ran is an assumption, and "reddens
nothing" is a measurement. M6 and M7 were needed beyond the plan's list because
no mutation of the plan's shape reds the zero-admin or the identity case.

**One real ABI narrowing found and recorded rather than fixed**:
`solidity_mapping_slot` takes a `u64` where Solidity's mapping index is a
`uint256`, and places it in the second word's low eight bytes. Not a live limit
— `orgs` sits at slot 0 — but indices above `u64::MAX` are unrepresentable, and
the `u64::MAX` case pins the placement against the `[32..40]` mistake a
`uint64`-for-`uint256` substitution invites. Five tests where the plan named
three for REQ-9m2rnd and six where it named four for REQ-xudf25; the additions
are the class C boundary cases (the zero admin, `u64::MAX` as index, the 256-bit
wrap, and the three struct keys derived as `get_org_state` derives them).

`client.rs`'s module header now records that its chain-free decisions are tested
from `on-chain-client/tests`, naming which target covers each of the four
`internals` items — matching the notes T3 and T5 left in `v_paseo_ah.rs` and
`dispatch.rs`.

**A harness lesson worth keeping** (it will recur): this session's worktree
guard refuses a compound shell line containing the string `git` *anywhere*, and
a heredoc writing the words "`.github`" into a file trips it. The Write tool is
the way through; the test header now says "the workflow directory" instead of
naming the path.

**T10 — done** (task commit `311e564`, merged into the change branch), **with
the coverage gate left failing for the user's decision.** Eight targets, not the
plan's seven — the list predated T2b's `log_ownership` — declared with
`required-features = ["test-support"]` and named in one stable order in
`Cargo.toml`, `verify_commands`, `rust.yml`'s `test` job and the
`coverage-on-chain-client` recipe. The `clippy` job is untouched, so the
panic-freedom gate still runs `--lib` only and the exposure cannot regress it.

**Full `verify_commands`: 71 passed, 0 failed, exit 0** — lib 2,
`best_lane_reorg_rule` 14, `contract_address_filter` 3, `decode_org_state` 9,
`decode_revive_event` 15, `h160_mapping` 5, `log_ownership` 7,
`runtime_version_dispatch` 5, `storage_slot_layout` 11. The three bolero
targets all exited on max duration, none on a failure: 256,684 / 233,137 /
38,186 iterations.

`red -> green:` no cycle applies, as in T1 and T9 — T10 declares targets rather
than adding behaviour. What *was* watched failing is a pre-existing build
defect, captured both ways: **`cargo check --all-targets` under default
features failed with E0432** (`could not find test_support in
on_chain_client`) before the `[[test]]` entries existed, and passes after.
Three files import the feature-gated module, not the two T2b and T3 predicted:
`log_ownership.rs`, `best_lane_reorg_rule.rs` and `storage_slot_layout.rs`.
Both variants of that command now pass.

### The coverage measurement, and why it is a stop

Measured with the eight targets in the recipe, and **independently re-measured
in the change worktree**: **42.91% lines / 43.47% regions**. Against the floors
as T10 found them — `ON_CHAIN_CLIENT_LINES := 52`, `ON_CHAIN_CLIENT_REGIONS :=
58` — `make coverage-on-chain-client` exited 2, and T10 stopped there rather
than moving a floor to make a merge pass, which is what the section title means
by "why it is a stop".

**It did not stay a stop, and this paragraph used to say it did.** Later in this
same change, on the owner's explicit decision, both floors were **recalibrated
downwards to 41 / 42** — one point below the new measurement, the same rule the
existing floors were set by — and the recipe then exits 0. The reason is the
arithmetic set out below: the relocation moved 214 executed test-body lines out
of `src`, which had been counted as covered *library* lines, so the old floors
were calibrated against a ruler that no longer exists; real library coverage
rose while the headline figure fell. That decision, its figures and the ruler
argument are recorded in "Coverage, and what the owner accepted" further down
this plan, in the `Makefile` beside the floors and in the unit config's
`coverage_command` comment.

Until review round 1 (finding-5), the two halves of this plan said opposite
things: this paragraph asserted "**No floor was touched**" while the Makefile,
the config comment and a green run all recorded the recalibration. The reviewer
was right, and the correction is the paragraph above: the stop was real, and it
was then resolved by a decision that belongs to the user and is written down —
lowering a floor "belongs in a verification record with a reason, not in a
hurry", and the reason is here.

The aggregate is misleading, and the per-file table is what makes it legible:

**SUPERSEDED — this table is the eight-target measurement, not today's.**
Review round 1's finding 1 later relocated the last two `#[cfg(test)]` tests
out of `src/types.rs` into `tests/type_widths.rs`, taking ten fully-covered
library lines out of the denominator: **`types.rs` is 3 lines, not 13**, and
**TOTAL is 491 lines at 41.75%, not 501 at 42.91%**. Every other row is
unchanged. The table is kept as written because it is the measurement the
recalibration decision was taken on; it is marked so a superseded figure
cannot be read as a current one. The current figures are in "Coverage, and
what the owner accepted", and the fall is set out in full under "Coverage fell
again". Marked by fix6b (review round 2 clean-up), the same discipline fix6c
applied to the acceptance table.

| File | Lines | Line coverage |
|---|---|---|
| `decode/v_paseo_ah.rs` | 118 | **100.00%** (was 96.23%) |
| `h160.rs` | 15 | **100.00%** |
| `types.rs` | 13 | **100.00%** |
| `decode/dispatch.rs` | 5 | **100.00%** |
| `decode/mod.rs` | 14 | **64.29%** (was 0.00%) |
| `client.rs` | 336 | **16.37%** |
| TOTAL | 501 | 42.91% |

Every module this change touched is at or near 100%. The aggregate is dragged
down by `client.rs` alone — 336 of 501 lines, two thirds of the crate, at
16.37% — which is the async subxt code exercised only by the nine
chopsticks/anvil targets that no gate runs. That shortfall is long-standing and
this change neither caused nor worsened it.

**What actually happened to the ruler.** The relocation moved 214 lines out of
`src` (715 → 501), and those lines were `#[cfg(test)]` test bodies which, being
executed, counted as 100%-covered *library* lines on both sides of the old
ratio. Netting them out of the 2026-08-26 measurement: 53.57% of 715 ≈ 383
covered, less 214 test-body lines ≈ **169 real library lines covered**. Today
the same figure is **215**. So real library coverage **rose**, from ≈33.7% to
42.91% of real library lines — while the headline number fell, because the old
headline was inflated by counting test code as covered product code.

**SUPERSEDED, in the same way and for the same reason.** "Today" in that
paragraph is the eight-target measurement: the figure is now **205** real
library lines covered, and **41.75%** of 491 real library lines, not 215 and
42.91% of 501. The conclusion is untouched — real library coverage still rose
from ≈33.7%, and the ruler argument is unaffected, because the ten lines that
left were fully covered on both sides of it. Marked rather than rewritten
(fix6b, review round 2 clean-up); the current figures are in "Coverage, and
what the owner accepted".

Three intermediate measurements, all on this tree, pin that:

**SUPERSEDED — the last row is the eight-target measurement, and both `501`
denominators are its basis.** The relocation that shrank the per-file table
shrank this one the same way: the crate is **491 lines, not 501**, and the
bottom row — the recipe is now **nine** targets, not eight — reads **41.75%
lines and 42.45% regions, not 42.91% and 43.47%**, from **205 covered lines
and 284 covered regions**. The middle row was not re-measured on the 491
basis, so read its percentage as belonging to the superseded one too, and the
recovery sentence under the table with it. The rows are kept as written
because they are the measurements the recalibration decision was taken on;
they are marked so a superseded figure cannot be read as a current one. Marked
by fix7 (review round 2 clean-up), the same discipline fix6b and fix6c applied
to the blocks above. The current figures are in "Coverage, and what the owner
accepted".

| Recipe | Lines | Regions | Denominator |
|---|---|---|---|
| old (`--lib` + 3 fuzz), 2026-08-26, before the relocation | 53.57% | 59.19% | 715 |
| old recipe, on this tree | 23.75% | 24.08% | 501 |
| new recipe (+ the eight targets) | 42.91% | 43.47% | 501 |

The eight targets recover nineteen of the thirty points the relocation cost.

**And one door this opens.** The Makefile records that per-file floors
(`--fail-under-file-lines`) "cannot be enabled yet: `decode/mod.rs` is at
0.00%, so any per-file floor above zero fails immediately." That file is now at
**64.29%**, so the blocker the comment names is gone. Per-file floors are the
fix for the known limit the same comment describes — that an average over a
crate lets a collapse in the well-covered decoder hide behind a rise in
`client.rs`. Not this change's work, but it is now available.

Both figures and this arithmetic are recorded in the `Makefile` beside the
floors and in the unit config's `coverage_command` comment, flagged OPEN, so
the verification record has the reason to hand rather than the scrollback.

**T11 — done** (task commit `592a8bd`, merged into the change branch). Four
documents: the register at 1179 lines, the requirements ledger at 193, the
problem ledger at 293, and the unit glossary filled with the ten terms. No new
IDs minted — a grep of all IDs in the four files returns exactly this plan's 36
plus org-node's REQ-ysyu9g.

`red -> green:` no cycle applies. T11 writes the items the tests T1–T10 already
verify; its verification is that every gate resolves them, which is the table
below.

**Every gate, every unit, green:**

| Command | Result |
|---|---|
| `check-trace.sh` on-chain-client | **exit 0** — `REQ 17, HAZ 6, RC 8, SDD 0, LLR 0, PR 5` |
| `check-ids.sh --allow-draft-files` on-chain-client | exit 0 |
| `check-trace.sh` / `check-ids.sh` org-members | exit 0 / exit 0 |
| `check-trace.sh` / `check-ids.sh` org-node | exit 0 / exit 0 |
| `check-trace.sh` / `check-ids.sh` app | exit 0 / exit 0 |
| `check-units.sh` (no flag) | **exit 0** — `units: 4, disclaimed 7; tracked paths 383` *(at this run — see below)* |

**The tracked-path count moved after this run.** `383` is what T11 measured and
is left as the record of that run; it is **384** as of 2026-09-10, re-measured
in fix6c, because review round 1's finding 1 added
`on-chain-client/tests/type_widths.rs`. The units and disclaimed counts are
unchanged. Recorded rather than overwritten, for the reason this plan gives
itself: a register that records measurements has to record when they stop
reproducing.

No `UNANALYZED-DERIVED`, no `MISSING-TEST`, no `DANGLING-REF`, no
`UNDECLARED-DEPENDENCY`, no `NON-EXPORTED-REF`. Every requirement is credited by
the annotations T1–T10 landed. on-chain-client's only remaining output is four
`UNRESOLVED-PR` roll-call warnings (open 0 days, against limits 30 / 10);
org-node's run still reports its two `UNMET-EXPECTATION`s and five open reports,
unchanged and exit 0.

**A prose error in this plan, found by T11 and corrected above.** Two sentences
in the T11 section said "sixteen requirements" where there are **seventeen** —
the count predates T2b, which added REQ-nygs7k for the admin filter. This
plan's own `Implements:` line and its control table both listed seventeen all
along, and `check-trace.sh` counts `REQ 17`. The register was written with
seventeen throughout. Corrected here because `merge-change` step 6b copies
from this file. (Seventeen was the figure at T11 and at the gate run above.
Review round 1 minted four more requirements and one more control, so the
change's final totals are **21 requirements and 9 controls**; the tool output
quoted in this section is the measurement of the day it was taken and is left
as it was recorded.)

**T11 touched a fifth file, deliberately: `docs/plans/2026-09-05-ratchet-setup.md`.**
The dispatch named four, but this plan's T11 section requires each open report
to get a dated checklist item, and the org-node half of this tooth did the same.
It added an "Added 2026-09-10" section carrying the four open reports at age
limit 2026-10-10, a problem-budget re-check (four open here, ten across three
ledgers, and every on-chain-client limit falling on one day), the carried-over
REQ-ysyu9g item now with the confirmed subxt fact, and the not-minted controls
that need an owner.

**What the register concludes, and what it refuses to soften.** Every assessed
risk is S3 and none is acceptable; the unit's overall residual risk is
**UNACCEPTABLE** against the intended use. No severity is argued down from
recoverability anywhere — including the runtime-refusal outage the controls
themselves introduce, which an earlier instinct would have called S2 and
recoverable, and which the class C ADR's 2026-09-02 withdrawal of that argument
forbids. All five findings the plan listed are recorded as **residuals**, not
controls: the unreachable call-site half of RC-5e3bdk under HAZ-werm85, the
`h160_of` pinning under HAZ-v2cmtx, the out-of-unit contract guard under
HAZ-xfg9cz, the empty `fuzz_event_round_trip` corpus under HAZ-95sc43, and the
absent finality evidence under HAZ-xd4urb. Sixteen not-minted controls, with
clippy-`--lib`-only and the rustfmt baseline as number 15.

**The register records this change's own failed measurement.** In its conclusion:
this change ran the prescribed red-by-mutation for the spoofing fix, watched it
stay **green**, and removed the false green by extraction. That is kept as the
argument for RC-5e3bdk's residual rather than summarised away — which is the
whole point of writing a register instead of a summary.

**A correction that applies to every task from T3 on.** Each task's Verify
paragraph names an expected `--lib` count (T3's "16 passed", T4's "12", T5's
"10", T6's "7", T7's "4"). Those figures were written as a *cumulative*
sequence — each assuming its predecessors had already relocated their modules —
but the tasks run in parallel worktrees off the same base, so in isolation each
sees only its own subtraction from the 23-test baseline. Measured: T5 saw 21
(23 − 2), T6 saw 20 (23 − 3). Neither is a failure; the plan's numbers were the
wrong shape. **The figure that matters is the one at the merge gate**, after
every relocation has landed, and it is the only one the verification record
should quote. Treat the per-task counts as "unchanged apart from this task's own
relocation" and check the arithmetic against 23, not against the plan.

**T2 — done** (task commit `62e1607`, merged into the change branch). Three
tests in `on-chain-client/tests/contract_address_filter.rs`; the emitting
contract's H160 is now carried out of the decoder as
`on_chain_client::state::EmittedEvent` and compared against the reader's
configured address, and `event_matches_contract` is gone. `--lib` 23 passed / 0
failed (unchanged — the two inline round-trip tests were rewrapped rather than
removed); `--test contract_address_filter` 3 passed / 0 failed;
`fuzz_event_round_trip` 39421 iterations, exit reason "max duration (1s -
default) exceeded"; `clippy --lib` and `clippy --lib --features test-support`
exit 0; **`cargo check --all-targets` exit 0 with no warnings**, which is what
proves the changed trait signature did not break any of the nine ungated
chopsticks targets — none of them calls the decoder directly.

`red -> green:` all three cases were watched failing **before the
implementation existed**, on `error[E0432]: no EmittedEvent in state` — the
decoder had nowhere to surrender the address — and then again under a
substituted mutation (report `[0u8; 20]` instead of the decoded address), with
`log_from_another_contract_with_a_valid_signature_is_rejected` failing
`left: [0…] right: [153; 20]`, "the decoder must report the impostor's address,
not the configured one".

**The plan's prescribed mutation is unobservable, and T2 measured it rather
than assuming it.** Disabling the caller's comparison
(`if false && emitted.contract != contract`) left `--test
contract_address_filter` at 3 passed. That is what T2b above exists to fix, and
it is HAZ-werm85's residual until T2b lands. Recorded here because a measured
negative result is evidence and would otherwise be lost.

Two smaller corrections, both kept. **`EmittedEvent` is reached as
`on_chain_client::state::EmittedEvent`** — `state` is already `pub`, so
`lib.rs` was not touched and the root `pub use` block is unchanged; T3 and T9
import it by that path. And **one inline test was renamed**:
`parse_event_from_other_contract_returns_none` →
`parse_event_with_unknown_signature_returns_none`, with its comment corrected,
because the old name asserted precisely the false belief the defect rested on —
a log from another contract now decodes rather than returning `None`, and
leaving the name would have re-stated the defect inside the suite. T3 relocates
that test under REQ-wnjz9j and should expect the new name. T2's own first draft
of the new test's header wrongly credited `tests/two_orgs_one_watcher.rs` as
covering the spoof; that target varies the *admin*, not the contract, and the
header was corrected before commit.

T2 introduced no new `rustfmt` drift: every `cargo fmt --check` hit in the files
it edited is on a pre-existing line, and the new test file is clean.

Recorded for the merge, not acted on: **the crate is not `rustfmt`-clean at
baseline**, independently of this change — `cargo fmt --check` wants
`use crate::decode::{DecodeError, Decoder, dispatch};` reflowed on a line this
change does not touch. There is no `cargo fmt` step in
`.github/workflows/rust.yml`, so it is not a gate and this change does not make
it one; adding it would touch pre-existing lines across the crate. T1's moved
`scan_step` body was therefore kept byte-for-byte as it stood in the closure,
so the diff reads as a pure move rather than a move plus a reflow.

## The gate, and the owner's acceptances

First full gate run, 2026-09-10, dispatched into the change worktree (not a
worktree of its own — check 7 reads *this* worktree's `git status`, and a fresh
one is clean by construction and proves nothing).

**Impact set, computed not judged** —
`check-units.sh --impact "master..HEAD"`: `on-chain-client` touched,
`org-node` dependent, `app` dependent. Three units. `org-members` is **not** in
the set; its coverage was measured anyway.

| Check | Result |
|---|---|
| `check-units.sh` (once) | exit 0 — `units: 4, disclaimed 7; tracked paths 383` *(at this run; **384** since fix6c re-measured — see the note under the T11 gate table above)* |
| `check-trace.sh` × 3 units | exit 0 each |
| `check-ids.sh --allow-draft-files` × 3 units | exit 0 each |
| on-chain-client `verify_commands` | **71 passed, 0 failed** |
| org-node `verify_commands` | **51 passed, 0 failed**; `quint typecheck` exit 0 |
| app `cargo test` | 0 passed, 0 failed — compile proof, as the config states |
| app `npm --prefix app run check` | **166 files, 0 errors**, exit 0 (see below) |
| `git status` | clean |

**Suite total across the impact set: 122 passed, 0 failed.** Plus five bolero
targets green with **no pass count** — `harness = false` reports an exit reason,
not a count — and one `quint typecheck` green with no counts, which the gate
subagent proved was not a no-op by typechecking a deliberately broken model and
watching it fail with `Error [QNT006]`.

Two gate findings, both resolved before the review:

1. **`npm --prefix app run check` did not run** on the first attempt: exit 127,
   `svelte-kit: command not found`, because `app/node_modules` is gitignored and
   absent from a fresh worktree — which `app/.guardrails/config.yaml` already
   warned of. The gate **reported it rather than skipping it**, which is the
   behaviour that makes the gate worth running. Remedy per
   `worktree-discipline`: the artifact was copied from the primary checkout,
   after confirming that checkout's uncommitted `app/package.json` edit is only
   an `allowScripts` entry for `fsevents@2.3.3` and changes no dependency, so
   the install is valid for the committed manifest. It then passed. One
   **pre-existing** warning remains — SvelteKit's generated
   `app/.svelte-kit/tsconfig.json` lists `"types": ["node"]` while `@types/node`
   has never been in `app/package.json` — so the output is not pristine, the
   cause predates this change, and adding the dependency belongs to the app
   unit's own work, not here.
2. **REQ-axcxf7 had no normal-case test annotated to it** (class C robustness,
   check 6). Both its annotated tests are refusals. Fixed as fix1 — see below.

### Coverage, and what the owner accepted

| Unit | Lines | Floor | Regions | Floor | Branches |
|---|---|---|---|---|---|
| on-chain-client — *at this run,* **SUPERSEDED** | 42.91% (215/501) | 41 | 43.47% (296/681) | 42 | `0 0 -` |
| on-chain-client — **current**, re-measured 2026-09-10 (fix6c) | **41.75% (205/491)** | 41 | **42.45% (284/669)** | 42 | `0 0 -` |
| org-members | 93.50% (935/1000) | 92 | 92.20% (1430/1551) | 91 | `0 0 -` |

**Why on-chain-client has two rows.** The first is the figure at the gate run
this section records, and it stopped reproducing later in the same change:
review round 1's finding 1 relocated the last two `#[cfg(test)]` tests out of
`src/types.rs` into `tests/type_widths.rs`, which took ten fully-covered
library lines out of the measurement (`types.rs` 13 → 3). 215 covered of 501
became **205 of 491**; 296 covered regions of 681 became **284 of 669**. The
counts in both rows are the raw covered/total llvm-cov reports, not derived
from the percentages. The current row is what
`CARGO_HOME=… make coverage-on-chain-client` prints today and is the figure the
floors are read against; the reasoning behind the fall is set out in full under
"Coverage fell again" below. The superseded row is kept because it is the
measurement the recalibration decision was taken on — but it is marked, which
is the discipline this plan states for itself.

Both floors met, exit 0 — at both measurements. **Meeting a floor is not
meeting the class C target**,
and the gate named two shortfalls as shortfalls. The owner was asked and
**accepted both as documented gaps** (2026-09-10), which is the acceptance
`verify-before-merge` check 5 reserves to the user:

1. **`client.rs` statement coverage, 16.37%** — 281 of its 336 lines
   unexercised. It dominates the aggregate and is the async subxt code only the
   nine ungated chopsticks/anvil targets reach. Unchanged in substance since
   2026-08-26; recorded in the register as HAZ-werm85's and HAZ-xd4urb's
   residual, not as a met target.
2. **Decision coverage is UNMEASURED in all four units** — both llvm-cov
   reports show `Branches 0 / Missed 0 / Cover -`, i.e. branch instrumentation
   is off and no decision figure exists at all. Class C mandates it. Also
   accepted: **org-node and app have no `coverage_command` at all**, so two of
   the three impacted units have no structural coverage measurement whatsoever.

None of the three is caused or worsened by this change. All stay tracked in
`docs/plans/2026-09-05-ratchet-setup.md`.

### Check 4's other half — mine to answer, and answered

The gate names which `verifies:` test carries which ID. That is its half. **Mine
is whether each of those tests was watched failing for the right reason before
it passed**, and a green run can never imply it. Read off the `red -> green:`
lines recorded above, task by task:

| Requirements | Attesting task | Reddened by |
|---|---|---|
| REQ-9vwcwc | T2, then T2b | T2: watched failing pre-implementation on `E0432` (no `EmittedEvent`), then under a substituted decoder mutation. T2b: seven cases on `E0432`, then three measured mutations including the **deleted contract check** (3 failed) |
| REQ-nygs7k | T2b | `E0432` pre-implementation; `admin_filter` ignored → 1 failed |
| REQ-wnjz9j, REQ-88fp2h, REQ-twdu84, REQ-axcxf7 | T3 | thirteen mutations, M1–M13; every one of the fifteen tests red under at least one |
| REQ-hd6m9d | T5 | the plan's mutation (4 refusal cases red) **plus a second** for the positive case, which the first left green |
| REQ-sx5b6g | T9 | **no cycle, by design** — pre-existing property, measured twice, stated as measured rather than dressed as a red→green |
| REQ-tkhe3u, REQ-rz7fja | T6 | four window-pinning cases red under two mutations |
| REQ-9m2rnd, REQ-xudf25 | T7 | eight mutations, after measuring the plan's own suggestion **inert** (M0, reds nothing) |
| REQ-4astjb, REQ-9wwenn | T4 | six mutations, mirrored on both edges; three cases **panicked inside `src`** on the too-short side |
| REQ-ntn4ss, REQ-gr2ver, REQ-5zux82 | T8 | seven mutations; all fourteen cases red under at least one |

**Every requirement is covered, with one deliberate exception and one
inheritance, both stated rather than papered over.**

- **REQ-sx5b6g has no red→green and should not have one.** Its property —
  never panic on arbitrary bytes — pre-dates this change, and T9 added
  annotations to targets that already exercised it. Inventing a mutation to
  manufacture a cycle would have been dishonest. It is attested as *measured*,
  twice, with the figures.
- **REQ-axcxf7's accepting-side attestation is inherited from T3, not from
  fix1.** The gate flagged this itself, correctly: fix1 was annotation-only and
  has no red→green of its own to offer. The two round-trip tests it annotated
  were created under T3 and reddened there — M7 (`SIG_GENESIS_INITIALIZED[0]`
  `0x8e`→`0x8f`) took the genesis round-trip red among eight, and M12
  (`SIG_ROOT_UPDATED[0]` `0x24`→`0x25`) took the `RootUpdated` round-trip red
  among seven. So the tests were watched failing; what fix1 changed is which
  requirement they answer for. That is the honest reading of
  `verify-before-merge`'s prohibition on re-annotating a test "green from the
  start": run 1's finding was a **robustness-completeness** gap under check 6,
  not a missing-`verifies:` gap under check 4, and the normal-case test was
  written under TDD with its accepting-boundary meaning documented at the time,
  not manufactured afterwards.

The HAZ and RC IDs carry no `verifies:` test and correctly so: hazards trace
through the RMF's `mitigates:` links and controls through `implements:` on the
requirements, both gated by `check-trace.sh`, which reported no
`UNMITIGATED-HAZARD` and no `UNIMPLEMENTED-CONTROL`.

### Excluded from every `verify_commands`, deliberately, and not attempted

The nine chopsticks/anvil targets in `on-chain-client/tests` and the three in
`org-node/tests` (`chain_genesis_e2e`, `finality_polling`, `preflight`). They
need a live fork plus `on-chain/scripts/node_modules`. Their exclusion is the
reason `client.rs` reads 16.37% and the reason finality and reorg delivery have
no gated evidence — both stated in the register rather than left to be inferred
from a coverage table.

## Review round 1 — findings and dispositions

Independent review, 2026-09-10, in a nested worktree with the diff, the
documents and the repository but no implementation narrative. The reviewer ran
every unit's `verify_commands` itself and reproduced this change's figures
exactly. **Twelve findings, three of them blocking.** Two were *measured* rather
than argued, which is why they are the two that matter most.

All twelve are accepted. None is disputed. Dispositions:

| # | Finding | Disposition |
|---|---|---|
| 1 | **BLOCKER.** Five documents claim all twenty-three `#[cfg(test)]` tests were relocated out of `src`; twenty-one were. `on-chain-client/src/types.rs:36-55` still holds two, unannotated and invisible to `check-trace.sh` — and `types.rs` is explicitly in scope. The gate's own `lib 2` line was the proof, unexamined. | **Fix (fix2a + fix2b).** Relocate both. The width test states a real ABI invariant and gets REQ-2qa5r5 under RC-sxjnx9; the `Display` test relocates without an annotation, which is legal — requirements need tests, tests do not need requirements. Then correct all five documents. |
| 3 | **BLOCKER.** All three tests in `contract_address_filter.rs` carry `verifies: REQ-9vwcwc` and **cannot go red if the decision breaks**: the file reimplements the predicate locally as `is_from_configured_contract` ("The caller's decision, verbatim", `:47-54`) instead of importing `log_is_ours`. Measured: with the real comparison deleted, this file reported 3 passed while `log_ownership` reported 3 failed. | **Fix (fix2a).** Import `on_chain_client::test_support::log_is_ours` and delete the local copy. A copy of a predicate tests the copy. This is the same defect class the change was written to remove, surviving inside the fix for it — T2 wrote this file before T2b created `log_is_ours`. |
| 5 | **BLOCKER.** The floors were lowered citing, in the past tense, a verification record that does not exist yet; and the plan's T10 section still says "No floor was touched", contradicting the Makefile and a green run. | **Fix (fix2a for Makefile/config wording, fix2b for the plan).** The record is written at step 6b, after review — so the citation was true-in-advance, which is not true. Reword to name where the decision *is being* recorded, and correct the plan's stale paragraph. |
| 2 | HAZ-xfg9cz names three routes; its P1 argument covers two and silently drops the third — a runtime-shape change, which HAZ-8s5chy assesses at **P2** for the same event. The file gives one event two probabilities. | **Fix (fix2b): raise HAZ-xfg9cz to P2.** The probability of a disjunction is the maximum over its routes, and one route is P2. Conservative, reproducible from the file, conclusion unchanged. |
| 4 | RC-8w9wtp and REQ-sx5b6g claim no decoder "allocates from an unchecked length taken out of the input". Nothing in `src` checks that; every such allocation is inside `parity-scale-codec`, a SOUP item, and `soup.md` is still the empty template. The fuzz targets evidence panic-freedom, not allocation size. | **Fix (fix2b): drop the allocation clause** from both, leaving the property the evidence actually supports, and record the `parity-scale-codec` dependency in HAZ-95sc43's residual with the same treatment `OrgRegistry.sol` gets under HAZ-xfg9cz. The reviewer is right that the register sets that standard for itself thirty lines earlier. |
| 6 | Not-minted controls 5, 11 and 14 have no owner, date or checklist item — and 11 and 14 are the **sole** named control for prose hazards assessed S3 and not acceptable. ISO 14971 requires options for an unacceptable risk to be carried to a decision. | **Fix (fix2b).** Add all three to the setup checklist, and disambiguate the two independently numbered not-minted series now in that file. |
| 7 | Two behaviours have no requirement: (a) *which* two event signatures are recognised — the ABI binding to the contract — is pinned by a test annotated REQ-wnjz9j, which states only the negative; (b) `fuzz_event_round_trip` asserts an inversion property while annotated to REQ-sx5b6g, whose arbitrary-bytes property that target never exercises. | **Fix (fix2a + fix2b).** Mint RC-675a3h under HAZ-v2cmtx for the ABI binding, with REQ-52uc8f; mint REQ-n6v896 for the inversion property under RC-6gfh8d and re-annotate the round-trip target to it. (b) is the sharper half: the annotation contradicts the target's own header. |
| 8 | The probability scale is defined once and applied two ways — HAZ-werm85 is P2 on a silent narrowing of "deliberate act" to "the victim's deliberate act", while two other entries count any act at all. | **Fix (fix2b): qualify the scale** in the Method section to "a deliberate act by a party whose cooperation the situation requires", which is the reading that reproduces all the estimates. Errs conservative; conclusion unchanged. |
| 9 | The two round-trip tests carry REQ-wnjz9j, which has no accepting side, so a decoder returning `Err` for unknown signatures would leave them green. | **Fix (fix2a):** drop REQ-wnjz9j from that pair; it has three dedicated tests already. |
| 10 | "The crate has twenty-two integration targets" — 11 + 9 = 20; `common/` and `fuzz_support/` are shared modules, not targets. Taken from a directory listing. | **Fix (fix2b):** twenty. |
| 11 | The plan says "seven new test targets"; there are eight. And the register argues "P1 rather than P0", where P0 is not on this project's scale. | **Fix (fix2b):** both. |
| 12 | REQ-9vwcwc bundles a delivery filter and a reporting obligation, verified by different tests in different files, against the ledger's single-behaviour form. | **Fix (fix2a + fix2b): split.** REQ-9vwcwc keeps delivery (verified in `log_ownership.rs`); REQ-5upq6n takes reporting (verified in `contract_address_filter.rs`, which after finding-3's fix is exactly what that file is for). |

Five new items, all minted before dispatch: **RC-675a3h**, **REQ-2qa5r5**,
**REQ-52uc8f**, **REQ-n6v896**, **REQ-5upq6n**. Requirement total goes 17 → 21;
controls 8 → 9.

**Two of the corrected figures land one higher than the finding states, because
the fix round itself changes them.** Finding-10 says "twenty" integration
targets and finding-11 says "eight" new test targets; both were counted against
the tree as the reviewer found it. Finding-1's fix adds `type_widths` as a ninth
gated target, so once both halves of the fix round land the true figures are
**twenty-one targets, twelve gated and nine ungated**, and **nine** new test
targets. The documents are written with those final numbers, not with the
reviewer's intermediate ones. Nothing else in either finding changes: the
reviewer's arithmetic was right (11 + 9 = 20, and `common/` and `fuzz_support/`
are modules rather than targets), and it is the fix round that moves the count.

**What the review found nothing against**, recorded because a review that
clears something has said something: all seventeen requirements satisfied by
code the reviewer read and cited; abnormal-input cases present for every one,
several beyond what class C asks; expectations recomputed independently rather
than imported; every new item in a DRAFT file with no definition moved; all 36
IDs well-formed; all controls implemented and all hazards mitigated; all
derived items assessed; the glossary complete and cross-boundary terms
resolving at the root; the `required-features` declarations genuinely making a
mis-named target an error rather than a silent skip; and **no residual softened
by a recoverability argument** — the runtime-outage entry refuses that argument
explicitly and cites the ADR's 2026-09-02 correction correctly.

### What the fix rounds actually did, including two things they found

**fix2a** (code, commit `6211554`) and **fix2b** (documents, `bd84089`) ran in
parallel on disjoint files; **fix3** (`395c318`) cleaned up after them.

Three results worth keeping beyond "the findings were fixed".

**The finding-3 fix is proved, not asserted.** With the contract comparison
deleted from `internals::log_is_ours`, `contract_address_filter.rs` now reports
**2 failed** where before the fix it reported 3 passed. The single case still
green is the acceptance path, which an accept-everything predicate must pass —
correct, not a residual hole. fix2a also declined to add `REQ-9vwcwc` to those
tests and said why: the delivery evidence already exists in `log_ownership.rs`
with its boundaries, so claiming it here would file a second *copy* of that
evidence rather than a second piece of it.

**The suite total did not go up, and the arithmetic is the point.** I predicted
71 + 2 = 73 after relocating the last two tests. It is still **71**: the old 71
*included* the `lib 2` line, and those two were precisely the tests in
`types.rs`. `--lib` is now 0, `type_widths` is 2. My prediction double-counted.

**Coverage fell again, to 41.75% / 42.45% over 491 lines**, and the floors were
correctly left at 41 / 42. Same arithmetic as the main relocation, once more and
smaller: ten fully-covered library lines left `src/types.rs` with the tests
(types.rs 13 → 3 lines, still 100%), so 215 covered of 501 became 205 of 491.
Nothing stopped being tested. But the slack is now **0.75 / 0.45 rather than a
full point**, which is tighter than the Makefile's own calibration rule prefers.
It is left alone deliberately and flagged here: raising a floor is a ratchet
step for a settled basis, not something to do in the same breath as a
re-baselining. It is a live consideration for the next change that touches this
crate.

**Two things the fix rounds found that the review had not.**

1. **HAZ-xfg9cz's residual grew from one count to two.** Raising it to P2
   forced fix2b to argue all three routes, which exposed that the third — a
   runtime shape change — has **no control at all**, and that a slot reading
   *absent* after such a change takes `get_org_state`'s `Ok(None)` shortcut
   rather than the width guard. That is a new gap, found by repairing a
   probability argument rather than by inspecting the code.
2. **A stale measurement in a file neither fix round owned.**
   `log_ownership.rs`'s header asserted as present fact that the sibling file
   "still reported 3 passed" under mutation — true when written, and falsified
   by fix2a three hours later. fix3 re-measured (1 passed / 2 failed today),
   kept the claim as **dated history** because it is the evidence for why the
   extraction happened, and added the reading that supersedes it. A register
   that records measurements has to record when they stop reproducing.

**And one error of mine, caught only by the merged state.** Review finding-7(a)
needed two halves — mint the requirement, and annotate the test that verifies
it. I gave the minting to fix2b and never gave the annotation to anyone, so
`REQ-52uc8f` landed with nothing pointing at it and `check-trace.sh` reported
`MISSING-TEST`. Neither subagent could have caught it: each was green against
the other's pending work, and only the merge showed it. fix3 repaired it, and
judged the annotation on its own evidence rather than on my say-so — settling
on `verifies: REQ-52uc8f, REQ-wnjz9j` because the accepting half establishes
"recognise these two" and the rejecting half establishes "and no other", which
is both requirements honestly. It also recorded a limit rather than papering
over it: **nothing gated compares those two signature literals against
`on-chain/src/OrgRegistry.sol`.** fix3 checked by hand that they are correct
today; the link is held by review across a unit boundary, not by a gate.

### fix4 — the two missing reds, established

Answering check 4 after review round 1 showed that **two of the five
requirements the fix rounds added had no watched red behind them**:
REQ-n6v896 was re-annotated onto a target that was green throughout, and
REQ-2qa5r5 was relocated from a test that was green throughout. A green test
with nothing behind it is what this change exists to stop accepting, so fix4
went and got the evidence. It produced **no file changes and no commit** — its
product is an observation.

**REQ-n6v896 — red established.** Mutation: in `parse_root_updated`
(`v_paseo_ah.rs`), `prev_root_hash.copy_from_slice(&data[64..96])` commented
out, leaving the field at its declared `[0u8; 32]` — literally the "loss or
defaulting" the requirement forbids. The target is `harness = false`, so the
red appears as a flipped run line (`iterations/s: 1.00 | rng inputs: 1 | exit
reason: test failure`) and a panic, not a count. bolero shrank the
counterexample to a single distinguishing byte:

```
assertion `left == right` failed: decoder must invert the canonical encoding
  left:  … prev_root_hash: OnChainRootHash([0, 0, …, 0])
  right: … prev_root_hash: OnChainRootHash([175, 0, …, 0])
```

Every other field round-tripped; only the dropped one differed. Restored,
`cmp` byte-identical, green again.

**REQ-2qa5r5 — red established four times, after the obvious mutation was
rejected as evidence.** The naive change (`Epoch(pub u64)` → `u32`) fails to
compile the *library*, so `type_widths` never ran — and fix4 refused to count
that: red-by-mutation means watching the **named test** fail for the named
reason, and a test that did not execute observed nothing. It found a mutation
that keeps the crate compiling instead — `#[repr(align(N))]`, which widens
`size_of` past the ABI field without changing the inner array's type — and ran
it once per width:

| Mutation | Observed |
|---|---|
| `align(32)` on `OrgAdmin` | `left: 32 / right: 20` — "must be exactly the ABI address" |
| `align(64)` on `OnChainRootHash` | `left: 64 / right: 32` |
| `align(64)` on `OrgPubKey` | `left: 64 / right: 32` |
| `align(16)` on `Epoch` | `left: 16 / right: 8` — "must be exactly the region `decode_uint256_to_u64` reads" |

All four widths the requirement names are individually gated by a live
assertion. `epoch_display_is_the_inner_value` stayed green throughout, so each
red is localised to the annotated case.

**And a finding nobody asked for, proved by negative control.** The header of
`type_widths.rs` says each width is "the ABI word minus the offset the decoder
reads from" — but `ABI_WORD`, `ABI_ADDRESS_OFFSET` and `ABI_EPOCH_OFFSET` are
`const`s declared **inside the test file**. Nothing in `src` feeds them, so
drift in the decoder's offsets cannot red this test. fix4 confirmed it rather
than assuming: shifting both regions in `v_paseo_ah.rs`
(`topic[..12]`→`topic[..8]`, `bytes[..24]`→`bytes[..16]`, each keeping its
width so it still compiles) left `type_widths` **green — 2 passed**.

That is not a hole, and the same control proved why: the identical mutation
immediately reds `decode_org_state.rs` — **4 failed** of 9. The offsets are
gated, elsewhere. The accurate statement, now recorded rather than left in a
comment that overclaims: `type_widths` gates the **newtypes' widths**; the
derivation from the decoder's regions is a comment mirroring `v_paseo_ah.rs`
**by hand**, kept in step by review, not by the compiler. Making it a real
coupling would mean exporting the offsets from `src`, which is a production
change this round rightly refused to make.

That is the fourth instance in this change of the same shape — a claim that is
true today and held by review rather than by a gate — after the contract's
`ZeroValue()` guard, the `h160_of` ground truth, and the two signature
literals. It belongs in the register beside the other three.

### fix5 — the pattern named, and a distinction drawn

fix5 (commit `aadabf2`) recorded fix4's finding in four places in the register
and wrote the synthesis: a new subsection, **"Four claims held by review, not by
a gate"**, under "Where the evidence is, and where it is not" — the one section
where a reader meets all four before meeting any of them individually.

The judgement that makes it worth more than four footnotes is one I did not ask
for. **It drew the line between the first three and the fourth.** The contract's
`ZeroValue()` guard, the `h160_of` ground truth and the two signature literals
each leave a *property* ungated, and the residuals carry that. The width
derivation leaves only a *comment's reasoning* ungated and carries no residual
at all. It is in the list because the failure mode is identical — a reader takes
a written claim for a checked one — even where the consequence is not. It also
records why all four were invisible to `check-trace.sh` **by construction**: a
gate can see that a test exists and runs, never that the test's reason for being
right lives in a file no gate reads.

Two smaller calls, both right. It gave HAZ-xfg9cz's residual a paragraph but
deliberately **no third residual count**, because what this one leaves ungated is
a sentence rather than a property. And it named the `parity-scale-codec`
panic-freedom paragraph as a *fifth relative* — the same standard applied to a
dependency rather than to a claim — and explicitly excluded it from the count
rather than silently inflating four to five.

Not-minted control **17** proposes the real fix (export the decoder's offsets
from `src` so the coupling is compiler-checked) and says plainly that it reduces
no residual risk, because `decode_org_state.rs` already reds on exactly the
drift it would catch. Adding it falsified the list's preamble — "each would
reduce a residual risk above" — so fix5 amended the preamble to name 17 as the
exception rather than leave a claim its own new entry contradicted.

## Review round 2 — findings and dispositions

Second independent review, fresh reviewer, given the artefacts and the
repository but **not** round 1's findings — so a mis-fix had to be found as a
defect rather than recognised as a correction. It ran every suite itself and
reproduced this change's load-bearing self-measurements, including fix4's
negative control. **Fourteen findings, five blocking.** All fourteen are
accepted; none is disputed.

| # | Finding | Disposition |
|---|---|---|
| 1 | **BLOCKER.** REQ-hd6m9d states a property the software does not have. It requires decoding "only through the decoder compiled for the Runtime spec version the chain reported", unqualified — but `from_client` resolves once at construction (`client.rs:137-144`) and `get_org_state` decodes through `self.decoder` forever after, which is exactly open report PR-w5sk5k. The ledger even asserts the opposite, that this requirement "is worded to the decision the function actually makes". Its only test exercises `dispatch::for_runtime`, never the client, so nothing gated can see the gap. | **Fix (fix6a).** Reword to the **resolution** decision the code actually makes, as REQ-ntn4ss is worded to its rule, and leave the per-decode obligation with PR-w5sk5k and its not-minted control. Class C does not permit a requirement asserting what an open defect says is absent. |
| 2 | **BLOCKER.** The pinned `SPEC_VERSION` — the constant RC-d7r82e turns on — is gated by nothing, **measured**: every case in `runtime_version_dispatch.rs` derives its expectation from the constant itself, so changing `2_002_002` to `3_003_003` leaves the whole gate at 71 passed while the crate is pinned to a runtime that does not exist. This is a **fifth** claim held by review, and the register asserts there are four. | **Fix (fix6a).** Record it as the fifth entry and add a residual sentence under HAZ-8s5chy. No code change: the failure is fail-closed, so it lands on the total-outage hazard already assessed S3/P2. The completeness claim is what makes it reportable. |
| 3 | **BLOCKER.** The ISO 14971 sweep of hazards introduced by controls is incomplete and self-contradictory: it says six controls "introduce nothing today" and names RC-6gfh8d and RC-sxjnx9 among them, while an entry above is headed "RC-sxjnx9 and RC-6gfh8d"; and **RC-kemv75 appears in neither list** — the one control of nine never checked for what it breaks, in the section whose whole purpose is that check. The conclusion's count of ten is computed over that sweep. | **Fix (fix6a).** Complete the sweep, resolve the contradiction, re-derive the count. The reviewer is right that this is the one section that cannot carry a bookkeeping error. |
| 4 | **BLOCKER.** HAZ-8s5chy's P2 does not reproduce from the scale as round 1's finding-8 amended it. Its argument still reads "no deliberate act **by any attacker**" — the attacker-centred vocabulary the amended Method section explicitly disowns — while the amended rule makes a runtime upgrade an act by "whoever ships the next version". The file claims every estimate re-derives under the new reading; for this one it does not. HAZ-xfg9cz's P2 was derived *from* this entry. | **Fix (fix6a).** Either re-argue in the amended vocabulary — the defensible line being that the scale's "party whose cooperation the situation requires" excludes the chain's own scheduled operation, which the scale must then say — or move to P1 and re-derive HAZ-xfg9cz. Verdict unchanged either way; the reasoning must reproduce. |
| 5 | **BLOCKER.** REQ-2qa5r5 has no abnormal-input case and no recorded exemption — the only one of twenty-one in that position. The cause is visible: it was minted **after** the gate run whose check-6 clearance the plan records, and that clearance was never re-run over the five new items. | **Fix (fix6a).** A sentence, not a test: it constrains a declaration and has no input domain. Record the same for REQ-n6v896, whose generator produces only valid shapes by construction. |
| 6 | Three test-header cross-references point at the wrong artefact, two because round 1's finding-12 split was not propagated: a layout-drift residual attributed to HAZ-xd4urb instead of HAZ-v2cmtx, and two pointers sending a reader to `contract_address_filter.rs` for REQ-9vwcwc, which that file's own header disclaims. | **Fix (fix6b).** One line each. |
| 7 | Four source citations stale or off by a line, all load-bearing: `types.rs:11-27` (fix2a added seven lines of doc-comment to that file in the same round that wrote the citation), the ABI-drift test at `:135` where it is `:168`, both signature constants one line early, and `fuzz_support/mod.rs` still naming a test this change deleted from `src`. | **Fix (fix6a for the register's three, fix6b for the test file).** |
| 8 | The plan records `tracked paths 383`; it is now **384**, because fix2a added a file after that run. And the coverage acceptance table still carries `42.91% (215/501)` / `43.47% (296/681)`, superseded four hundred lines later with no marker at the table — the parenthesised counts corrected nowhere (measured 205/491, 284/669). | **Fix (fix6c).** This is the discipline the plan states for itself. |
| 9 | The setup checklist calls on-chain-client's not-minted list "sixteen"; fix5 made it seventeen. And its panic-freedom item says "eight new integration targets"; there are nine. Same class as round 1's finding-11, in the file the fix rounds edited to answer round 1's finding-6. | **Fix (fix6c).** |
| 10 | The Makefile contradicts itself on the figure its recalibration turns on: `client.rs` at 16.37% in one note and **12.75%** in another (a pre-relocation figure), and both give the denominator as 501 where it is now 491 — uncaveated, in the file the recalibration names as holding its full reasoning. Plus a broken sentence fragment in the unit config. | **Fix (fix6c).** |
| 11 | REQ-nygs7k carries two behaviours — a delivery rule and an ordering rule — the defect round 1's finding-12 split REQ-9vwcwc for. Its ordering clause's test is annotated to REQ-9vwcwc, and a test carrying REQ-nygs7k exercises a no-filter case the requirement does not cover. | **Fix (fix6a for the wording, fix6b for annotations).** Coverage is not lost; the annotation map does not say what the ledger says. |
| 12 | REQ-axcxf7's second clause — "ends inside a field's own length prefix" — is exercised by neither test annotated to it. Both cuts land outside any prefix; the literal case is a cut at byte 21, inside `data`'s two-byte compact prefix, and it is untested. The behaviour *would* be refused, by SOUP. | **Fix (fix6b): add the missing case.** A genuine class C robustness gap, cheap to close. |
| 13 | REQ-2qa5r5's wording contradicts the unit glossary (the Organisation admin is the key the slot is *keyed on*, not a value read from it) and uses "Organisation public key", which neither glossary defines — both call it the *signing key*. | **Fix (fix6a).** |
| 14 | The two round-trip tests carry REQ-88fp2h but cannot red on its stated property — **measured**: one-sided weakening of all four shape guards reds the six dedicated refusals and leaves both round-trips green. Same structure as round 1's finding-9. A nit because they do execute the guards and serve as the normal-input case. | **Record (fix6c), do not re-annotate.** Worth stating so the record does not read nine annotations as nine independent reds. |

**What the reviewer cleared, explicitly**, because a review that clears
something has said something: all twenty-one requirements implemented and cited
bar finding-1; every one carrying a `verifies:` on a gated target; `--test`
with the feature off confirmed a cargo error; all forty-one IDs well-formed; no
definition moved out of a dated file; every control implemented, every hazard
mitigated, every derived item assessed; **no severity anywhere argued down from
recoverability** (grepped: the only mentions are the Method disclaimer and
RC-d7r82e's explicit refusal citing the ADR); the exclusion of the twelve
chopsticks targets stated everywhere it costs something; and the change's own
load-bearing self-measurements reproduced, fix4's negative control included.

### Finding 14, recorded rather than fixed

What REQ-88fp2h's annotations do and do not each prove.

Recorded here by fix6c. **Nothing is re-annotated for it, on the owner's
explicit decision**, and no test is added or removed. It is written down
because a `verifies:` count is easy to read as a count of independent reds, and
for this requirement it is not.

**The measurement, by the reviewer** — fix6c did not re-run the mutation and
does not claim to have; what fix6c did check, by reading, is that the four
guard sites are at the lines cited and that the annotation count below is what
the tree holds. REQ-88fp2h says a log whose topic count or `data` length
does not match the ABI for the signature it claims is refused with a typed
error naming both the expected and the actual value. Weakening all four shape
guards **one-sidedly** — `!=` to `<` at
`on-chain-client/src/decode/v_paseo_ah.rs:137` and `:164` (`topics.len()`) and
`:144` and `:171` (`data.len()`) — reds the six dedicated refusal cases and
leaves **both round-trip tests green**: 11 passed, 4 failed. So
`genesis_initialized_round_trips_every_field` and
`root_updated_round_trips_every_field`, which carry `verifies: REQ-88fp2h`
alongside REQ-twdu84 and REQ-axcxf7, cannot go red on REQ-88fp2h's stated
property under a one-sided weakening: they present shapes of exactly the right
size, which a `<` guard still accepts.

**Why this is a nit and not a hole.** The two round-trips *do* execute all four
guards, and they are REQ-88fp2h's **normal-input case** — the requirement's
check-6 obligation that a well-formed log of the right shape decodes rather
than being refused, which no refusal test can discharge. A mutation that
tightens rather than loosens a guard reds them, and only them. The
requirement's *refusal* red comes from its six dedicated cases, which is where
it should come from. Coverage of REQ-88fp2h is therefore complete; what the
annotation count overstates is independence, not coverage.

**The count, measured rather than repeated.** `verifies: REQ-88fp2h` appears
**eight** times, all in `on-chain-client/tests/decode_revive_event.rs` — two
round-trips (`:230`, `:274`) and six dedicated refusals (`:341`, `:362`,
`:386`, `:410`, `:439`, `:468`); a grep of the whole unit finds no ninth. The
finding-14 row above says "nine annotations"; that is off by one and the row is
left as the reviewer wrote it, with the correction here rather than a silent
edit to a transcribed finding. Read the eight as **six independent refusal reds
plus one normal-input pair**, not as eight independent reds.

**Same structure as round 1's finding 9**, and the same disposition reasoning:
an annotation that names a requirement it exercises but cannot falsify one-way
is worth stating in the verification record; it is not worth removing when the
test is the requirement's only normal-input evidence.

### Round 2's fix rounds — what they did, and what they found doing it

**fix6a** (register + ledger, `0ff48de`), **fix6b** (tests + one plan section,
`8970d53`), **fix6c** (plan, checklist, Makefile, config, `ae824a5`) and
**fix7** (`9dbd277`) between them dispositioned all fourteen findings. Four
results are worth more than "the findings were fixed".

**The overclaim was in two places, not one.** Finding 1 named REQ-hd6m9d;
fix6a checked the control behind it and found RC-d7r82e carrying the identical
false claim — state "decoded only through a decoder compiled for the
`spec_version` the chain actually reported". Both now state the **resolution**
decision the code makes, and RC-d7r82e gained an explicit "what this control
does not claim" paragraph. The per-decode obligation sits with PR-w5sk5k and
its not-minted control, where it belongs.

**fix6a verified all 115 citations, not the three it was asked about, and
found two more wrong.** The too-many-topics mutation panics were cited at two
lines that are `actual: topics.len(),` inside an error struct and cannot panic;
they now cite the actual indexing sites. Where it could not reproduce the
second site's panic message it gave a line range rather than inventing one, and
said so. A third citation called a slice expression a `copy_from_slice`.

**The probability scale now says what it means.** fix6a took the harder of the
two options on finding 4: rather than move HAZ-8s5chy to P1 — which would have
undone round 1's fix and called a total outage on every routine Paseo upgrade
"improbable", using P1 as a wastebasket the file itself warns against — it
amended the scale to define "a party whose cooperation the situation requires"
as *the one who has to decide to change something*, and named two actors who
are not: an adversary, and a system merely continuing to run. **A release is
P2; a decision to change a convention carried in a release is P1.** It then
re-derived all twelve estimates and reported every one reproduces — and
corrected the register's own claim that round 1 had already done that, which
round 1 had claimed and had missed one.

**The completed sweep found a real gap, and it did not move the total.**
RC-kemv75 — the one control of nine never checked for what it breaks —
introduces two situations: the first backfill span is as long as the finality
lag, and a deduplicated head is reported once and never again. Both S3/P2, both
not acceptable, both **further triggers** for hazards already counted rather
than new hazards, so the register's total stays at ten. The sweep now reads
five of nine with an entry, four introducing nothing, each with a reason.

Three smaller things, each recorded rather than smoothed over. fix6c **could
not reproduce** finding 14's mutation (the sandbox blocked that target) and
attributed the figure to the reviewer instead of claiming it, while verifying
by reading that all four guard sites are where cited. It also found the
reviewer's finding row says "nine annotations" where the tree holds **eight**,
and left the transcribed finding untouched — the correction sits in the note
beside it, because a finding reworded by the author is the author's finding.
And fix6b's new length-prefix case **passed first time**, stated plainly as
such: the SCALE codec already refused that input; what was missing was evidence
for the requirement's second clause, not a fix.

fix7 closed the annotation move that folding REQ-nygs7k's ordering clause into
REQ-9vwcwc implied, and added the honest caveat that **the re-pointed test
cannot be reddened by any mutation of the contract check** — it is a positive
delivery case, as is its sibling. Deleting the comparison reds three of the
seven cases in that file; these two are not among them, and cannot be.

## Review round 3 — findings and dispositions

Third independent review, fresh reviewer, given the artefacts but not the
previous 26 findings, and given an explicit **merge bar** — substance over
prose, since two rounds had already swept citations hard. It re-ran every
suite, and **re-ran fourteen of this change's own mutation measurements**,
reporting that each reproduced. **Eight findings: four above the bar, four
below.** All accepted.

| # | Finding | Bar | Disposition |
|---|---|---|---|
| 1 | **Two minted controls carry a leading clause no requirement realises**, while the register asserts each control "is realised by requirements carrying `(implements: RC-…)`". RC-kemv75 opens with the best-block/finalised distinction — none of its three requirements states it, and the tell is that *Finalised observation* is listed among the terms the requirements use and appears in no requirement. RC-d7r82e ends "the refusal is made at construction, before any read is possible" — stated by no requirement and reached by no gated test. `check-trace.sh` passes because each control has *some* implementing requirement; a clause-level check would not. | **ABOVE** | **Fix (fix8a): narrow both controls, record the removed clauses as not-minted controls.** I checked whether the alternative — minting two requirements — is open, and it is not: both clauses live in async code needing a live chain (`from_client` is async; the best/finalised distinction is produced by two lanes passing different `wrap` closures), so a new requirement would be `MISSING-TEST` on the next run. A requirement with no possible gated test is the defect this change exists to remove, not a fix for one. |
| 2 | **A sixth claim held by review, and the list asserts completeness at five.** The `OrgRegistry` storage layout and event shapes that RC-5ejucb and RC-6gfh8d hard-code were established in this change *by reading a disclaimed file*, and no gate re-establishes them — the register says so itself ("a contract revision that inserted a state variable ahead of `orgs`… no gate here would go red"), and its own not-minted control 9 names the layout alongside the `ZeroValue()` revert, but the five-claim list collects only the revert. | **ABOVE** | **Fix (fix8a).** Same shape as round 2's finding-2, accepted for the same reason: the completeness assertion is what makes it reportable. Add the sixth and re-derive "five" everywhere. |
| 3 | **A clause three documents say is gated, measured unobservable.** REQ-9vwcwc requires the contract comparison be made "**before**, and independently of, any other filter". The reviewer rewrote `log_is_ours` to evaluate the admin filter first and return the contract comparison last — the exact inversion — and `log_ownership` reported **7 passed, 0 failed**. Ordering inside a total boolean predicate is unobservable; what the cited test actually gates is the *consequence*, which does red. | **ABOVE** (low end) | **Fix (fix8a + fix8b): wording only.** Drop "before, and independently of", keep the observable "so that no other filter can admit a log from another contract", and correct all three sites. The safety-relevant half is genuinely gated and the dangerous reordering does red — but by the register's own stated method, a claim nothing can red is not a claim the evidence supports. |
| 4 | **Two requirements state properties the software does not have.** REQ-ntn4ss says a best head "at or below" the last processed is reported as a reorg — a re-notification of the *same* head takes the dedup arm and yields nothing, so it contradicts REQ-gr2ver directly below it, two requirements giving different answers for one input. REQ-5zux82 says a Best-block observation is reported "for every height" in the span; a height carrying no `OrgRegistry` event produces no notification, and the gated evidence asserts the span, not any reporting. | **ABOVE** | **Fix (fix8a): reword both** to what the code does and the tests assert — REQ-ntn4ss gains "whose hash differs from that head's"; REQ-5zux82 moves to reading the span. |
| 5 | Two register figures stale by one, same cause: `decode_revive_event.rs` called "(fifteen cases)" where it holds 16, and the `SPEC_VERSION` mutation recorded as "71 passed" where the reviewer re-measured **72**. Both pre-date fix6b's added test. The substance of the measurement — fail-open, nothing reds — reproduces exactly. | below | Fix (fix8a). |
| 6 | A count that does not add up in the paragraph introducing the problem ledger: "**Four** hazards … all four filed … **One** fixed … **The other four** stay open." Four minus one is three. The ledger is right (five reports, one resolved, four open); the register counts hazards in one sentence and reports in the next without saying so. | below | Fix (fix8a). |
| 7 | Citation drift, again on the same item: the ABI-drift test cited at `:168`, which is mid-comment; the `#[test]` is at `:174-175`. Round 2 moved this from `:135` to `:168` and it is still not the item's line. | below | Fix (fix8a). |
| 8 | REQ-5upq6n says the software "shall report … the Emitting contract" with every decoded event. The *decoder* does; the address is then dropped before a subscriber sees it, since `SubscribedEvent` carries no contract field. Intent is unambiguous from the register and the test header; the requirement's own text is the only place that does not say which surface reports. | below | Fix (fix8a): name the surface. |

**What round 3 cleared, and it is the most substantive clearance yet.** It
re-derived **all thirteen** probability estimates against the *current* Method
wording and reports every one reproduces — including the two the round-2
amendment was written to separate. It confirmed nothing leans on recoverability
and checked the ADR itself. It verified no test reimplements what it tests, and
found the suite genuinely mutation-sensitive: **fourteen of this change's own
measurements re-run and reproduced**, plus ten further mutations of its own,
each reddening a named case, plus a check that all three bolero targets fail on
an injected panic and on a broken inversion in *both* enum arms. It found the
requirements ledger's "what these requirements do not say" section "unusually
thorough", and credited the register for running the prescribed red-by-mutation
for RC-5e3bdk, getting a green, and restructuring the code rather than the
record.

### Round 3's fix rounds

**fix8a** (`f5ccb0d`), **fix8b** (`e0fc1d3`) and **fix8c** (`2a8003b`)
dispositioned all eight findings. Three results beyond "the findings were
fixed".

**fix8a found a third instance of finding 3 that the review had not.**
RC-5e3bdk carried the same unfalsifiable ordering clause as REQ-9vwcwc. Dropping
it from the requirement alone would have left it orphaned in the control — which
is finding 1's defect, created by fixing finding 3. Both narrowed, and the
change recorded at the control.

**It also judged the two newly-orphaned clauses into the review-held
collection**, not merely into the not-minted list, on the right test: the
register still leans on each — HAZ-xd4urb's framing rests on the best/finalised
distinction, the "not a degraded mode" argument on the construction-time
refusal. A claim the analysis relies on, true today, established by reading,
re-established by nothing. That section is now **eight** claims, from four two
rounds ago. The growth is not drift: each is a place this unit's safety argument
rests on something no gate will catch changing.

**SUPERSEDED — that section is now nine claims, not eight.** Review round 4's
finding 2 added the ninth on exactly the ground round 3's finding 2 above was
accepted on: the wiring between `log_is_ours` and its call site — that the
address compared is the one `from_client` was given, and that `continue` drops
the log — is established by reading, re-established by no gate, argued under
HAZ-werm85 and named by not-minted control 13, and was collected nowhere while
the list asserted completeness at eight. fix9a collected it and re-derived the
figure. A tenth relative is argued at its own site and deliberately left out of
the count, being a dependency rather than a claim. The sentence above is kept as
written because **eight** is what the register held when round 3 concluded; it
is marked so a superseded count cannot be read as a current one. Marked by fix10
(review round 4 clean-up), the same discipline fix6b, fix6c and fix7 applied to
the coverage blocks above.

**fix8b argued the checklist question rather than following convention.**
Not-minted control 17 has no checklist item because it reduces no residual risk;
controls 18 and 19 each trace to a hazard and each was a clause of a *minted*
control until round 3 — so after the narrowing, no minted control asserts either
behaviour and the queue is the only thing that remembers them. That is the
strongest case for an item, not the weakest. It also showed neither is
discharged by the existing chain-lane item: one needs assertions the ungated
targets do not make, and its committed half cannot come from chopsticks at all,
which finalises every block immediately; the other needs a target that does not
exist.

**And it stopped at a decision instead of guessing.** The two not-minted lists
were disambiguated by a note enumerating which numbers collide, and that
enumeration was wrong under both readings of what it meant. fix8b reported both
readings and left the choice. **My call: replace the list with a rule** — an
enumerated list of ambiguous numbers had gone stale twice in consecutive rounds,
each time a control was added, so it is fragile by construction and the
maintenance falls on whoever least expects it. fix8c replaced both notes with
"every reference names its unit", and its sweep found **ten** bare references
where the review had named two — the extra eight were short-form back-references
("like 18", "control 5's") that a search for the full phrase would miss.

## Review round 4 — findings and dispositions

Fourth independent review, same merge bar, told explicitly that finding nothing
above the bar was a valid answer and not to manufacture one. **Seven findings:
two above the bar, five below.** Both above-the-bar findings are wording; the
reviewer measured both rather than arguing them, and **neither touches the code,
a severity, a probability, or the conclusion**. All accepted.

| # | Finding | Bar | Disposition |
|---|---|---|---|
| 1 | **REQ-5zux82 and REQ-ntn4ss both turn on the word "extend", which neither glossary defines, and they use it in two incompatible senses.** Under the sense REQ-ntn4ss's own first clause forces (immediate child with the right parent), a multi-height jump does not extend, so REQ-5zux82 demands the new head's height alone while the code backfills the whole span — **measured**: narrowing `from` in `scan_step` to exactly that reading reds `a_jump_backfills_every_skipped_height`, 13 passed 1 failed. Under the other sense (`n > prev.number`, which this plan itself uses), REQ-ntn4ss's second clause forbids a reorg report the code makes and a test asserts. This is the residue of round 3's finding 4: that round fixed both requirements' quantifiers and left the term. | **ABOVE** (low end) | **Fix (fix9a).** Spell the exception out — "when the new head's height is at or below the last processed head's" — and drop REQ-ntn4ss's second clause, which settles nothing its first does not. The reviewer notes honestly that a reader resolving each requirement locally gets the right answer, which is why it ranks low. |
| 2 | **A ninth claim held by review, and the section asserts eight.** Under HAZ-werm85 the register says the wiring between `log_is_ours` and its call site — that the address compared is the one `from_client` was given, that `continue` drops the log rather than surfacing it — is "established by reading", with not-minted control 13 named as what would close it. That is the collected criterion word for word, and RC-5e3bdk's entire "reduced" residual depends on it: if `decode_contract_events` did not reach `log_is_ours`, this unit's most probable hazard would be mitigated by nothing. Argued at its own site, collected nowhere. | **ABOVE** | **Fix (fix9a).** Identical shape to round 3's finding 2, ruled above the bar for the identical reason. Add the ninth and re-derive "eight" at the thirteen sites the reviewer lists, plus this plan's round-3 section. |
| 3 | `type_widths.rs`'s header says the over-wide-epoch refusal is gated in `decode_org_state.rs` **and** `decode_revive_event.rs`. Measured: `EpochOverflow` appears only in the first; the event path's epoch has no gated case of its own, so REQ-9wwenn — worded generally — is evidenced on the storage path only. | below | Fix (fix9b). Below the bar because both paths share `decode_uint256_to_u64`, so a real weakening reds `decode_org_state`; the citation and its implied second source are what is wrong. |
| 4 | `fuzz_parse_revive_event`'s header sends a reader to `contract_address_filter.rs` for REQ-9vwcwc — the file whose own header declines to claim it. **Verbatim the defect round 2's finding 6 fixed in `decode_revive_event.rs`**; the identical sentence in the fuzz target was not swept. | below | Fix (fix9b). |
| 5 | The setup checklist says the controls introduce "three entries"; the register says four, the fourth added by round 2's finding 3. | below | Fix (fix9b). |
| 6 | Two minted controls still carry a sub-clause no requirement realises, against the register's clause-level claim: RC-d7r82e's "the decoder **a client reads through**" (which lives in `from_client`, async and ungated — the same place round 3 narrowed its sibling out of), and RC-675a3h's "detected by an independently recomputed comparison", which is a property of the *evidence*, not of the software, and which no requirement could state. | below | **Fix (fix9a), by softening the claim rather than narrowing further.** The substance of both is recorded loudly elsewhere; what is wrong is a sentence claiming more than clause-level realisation can deliver. A control clause realised by a gated test rather than by a requirement is legitimate — the register should say so instead of asserting a uniformity it does not have. |
| 7 | Two nits: REQ-5zux82's "for each event it finds" reads against REQ-nygs7k's withholding (resolved by specific-over-general, but unstated), and two counts of the introduced entries use different units — per control in one place, per entry in another. | below | Fix (fix9a). |

**What round 4 cleared.** All 21 requirements satisfied by code checked against
the implementing function. No test reimplements what it tests. **All thirteen
probability estimates re-derived against the current scale and every one
reproduces.** Nothing leans on recoverability — the only mention is the explicit
refusal, and the reviewer checked the ADR withdrawing it. No residual claim
stronger than its evidence: "each 'reduced, not acceptable' is enumerated by
count and each count matches what is there". **All 19 not-minted controls
accounted for** — eighteen queued with owners and dates, control 17 explicitly
exempted with a reason the reviewer confirmed by its own measurement — so **no
ISO 14971 gap**. Four of this change's own recorded measurements reproduced
exactly on the reviewer's machine, and five fresh mutations each reddened a
named case.

One observation it raised and deliberately did **not** file as a finding, which
belongs in the record anyway: the finalised lane's no-omission property rests
entirely on subxt's internal gap-filler — a SOUP behaviour, ungated, unnamed in
the register. It withheld it because RC-kemv75 is scoped to the best lane and
never claims it. That is the correct call and the fact is worth keeping.

### Round 4's fix rounds

**fix9a** (`1400625`), **fix9b** (`f27d286`) and **fix10** (`234813a`)
dispositioned all seven findings.

**fix9a corrected my disposition rather than following it.** I had passed the
reviewer's suggested wording through verbatim — "at or below the last processed
head's" — which leaves the *no head processed yet* branch unstated, and **two
gated tests exercise exactly that branch**. fix9a checked `scan_step`'s actual
`from` computation, found the `_` arm covers both `n <= prev.number` and
`last == None`, named both, and flagged the departure. It then re-read all
fourteen tests against the reworded pair and confirmed each matches — including
the two negative cases that previously matched only by implicature and now match
the text. Its sweep for the ninth claim also found more sites than the review
had: 16 lines and 21 numerals against a counted thirteen.

**It declined a tenth review-held claim and recorded why.** Softening the
clause-level claim made RC-d7r82e's client→decoder binding fit the collected
criterion — true today, established by reading, re-established by nothing. It
left it out because the list excludes defects and that gap is already carried as
PR-w5sk5k, and wrote the judgement down so a future round disagrees knowingly
rather than by accident.

**fix9b verified its own justification before writing it.** It needed to say
that the missing event-path epoch case is not a coverage hole because both paths
share `decode_uint256_to_u64`. Rather than assert it, it switched the guard off
and measured — `decode_org_state` 8 passed, 1 failed, on the named case — and
only then wrote the sentence. It also added a distinction nobody asked for: an
over-wide case on the event path would not be duplication, because it would gate
that the *refusal* is reachable there, not merely that the narrowing happens.

**fix10 drew the right line on the abandoned term.** Sweeping "extend" out of
test prose, it separated the three senses: the reorg rule's (rewritten), the
ordinary English one about a test file being enlarged (left), and
`Vec::extend_from_slice` (left). The aim was that no comment describes the rule
in vocabulary the requirements dropped, not that a word is banned. It also kept
"does not descend from", correctly — that is a statement about the chain's
parent structure, not a restatement of the rule.

**One loose end of round 4's own finding 4 is worth naming**, because it is this
change's recurring failure mode in miniature: the cross-reference fix9b
corrected in the fuzz target was **the identical sentence** an earlier round had
already corrected in a sibling file. One copy was swept; the other was not. Not
wrong reasoning — a true statement that stopped being true in one place and not
the other.

## Not in scope

Meeting REQ-ysyu9g (the interview's second answer). Fixing PR-w5sk5k,
PR-h4mb8y, PR-uq5r97 or PR-qpp28h. The `app` unit's risk analysis. Gating the
nine chopsticks/anvil targets or the live-Paseo smoke test. A
`coverage-on-chain-client` floor raise — the floor stays where the Makefile has
it, and the shortfall against the class C target stays recorded as a shortfall.
Decision coverage, still unmeasured in every unit. Anything under `on-chain/`,
which is disclaimed and holds the Solidity contract this unit reads.

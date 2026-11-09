# S3b-io part 2 — the revoked Device's side, on the domain API

**Goal (2026-11-02, §0):** a Member keeps each verified acknowledgement it receives and the app shows the Admin which removed Devices acknowledged; a removed Device sends its acknowledgements first to the Device it learned the revocation from; org-io's unused own-admin check is removed; the held-as-revoked list's residual is recorded.
**Goal (2026-10-16, superseded by §0):** a Device that verification finds removed from an Organisation deletes what it holds for that Organisation once the app has deleted its own data and confirmed, keeps only its keychain item and a small retained bundle, and sends its signed acknowledgement to up to three distinct Member Devices until three accept or two months pass, then deletes the item and the bundle; Members keep the acknowledgements they receive in a local revocation log, which the app shows the Admin; and org-io's unused own-admin check is removed.
**Implements:** (T13b, 2026-11-03) org-node REQ-h7y5ur, REQ-f8pu8a, REQ-32cp7f, SDD-acnpu4; org-io REQ-xghnh3, REQ-kn8zg6 (owner ruling Q1 (c), 2026-11-03); app REQ-yvzef9, SDD-g9ugu7, RC-z8cptv; and the LLRs under them (§5). Retires org-io REQ-ntkjk6 (ruling 6). Supersedes §10 of `docs/plans/2026-10-08-org-io-publish.md` (part 1's plan), whose T16c and T16d are dropped and whose other part 2 tasks are reshaped here (§2).
**Resolves:** PR-w88sr9 (obsolete, owner ruling 2026-11-08); PR-m9betm (T13b, by reading the code; pinned in T18); PR-y6ay44 (fixed in T18, owner ruling Q4 of 2026-11-03). PR-7ud5fv is out of scope (ruling Q4): it is left to a change of its own. PR-vt244s moved to change `worktree-org-io-domain-api` by owner ruling of 2026-10-16 (its write/verify/send split and pull delivery are PR-vt244s's own fix outline), with PR-k5tdxj's item 2.
**Safety class:** C (org-io; org-node; app; on-chain-client). No per-item override. *Corrected 2026-11-09 (merge review MR1-C6):* O2 was ruled keep (R3), and on-chain-client is still touched: ledger notes (R3) and seven test annotations (T18's Q6 sweep).
**Merge plan:** after `worktree-org-io-domain-api` merges (done: 7ac0825, merged here as c1bf729). *Corrected 2026-11-09 (MR1-O6):* the branch no longer holds a draft only; every task is merged (Progress).
**Verification:** each touched unit's `verify_commands` (`<unit>/.guardrails/config.yaml`), as at the time domain-api merges. Not run for this draft beyond the doc checks (§8).

## 0. Rescope after domain-api merged (2026-11-02) — this section wins over §1–§7

Domain-api merged to local master as `7ac0825` on 2026-10-31 and was merged into this branch (`c1bf729`). It carried owner ruling **Q4'**: on verifying its own removal a Device deletes everything at once except its signed acknowledgements (tiered 3/2/1) and the recipients' keys and addresses, with no app confirmation; acknowledgements are resent with backoff (REQ-jqmpz5), each deleted on `accepted`, and the store and keychain item go once none is left, within 7 days (REQ-98a3ty). A survey of the merged tree (2026-11-02) gave:

| Item | Status |
|---|---|
| T13b (a), (b), T16b, T16e, C2, O1, O3; ruling 3, the two-month limit, the retained bundle | **superseded by Q4'** (org-node `requirements/2026-10-14-batched-epoch.md` REQ-48zc3e/REQ-y9p27b; org-io `requirements/2026-10-14-domain-api.md`) |
| T13b (c), (d), (f), T16f | **done by domain-api** (REQ-k6kcgd, REQ-jqmpz5, REQ-98a3ty, REQ-sd7ayv, RC-45k3zg; `org-io/src/acknowledge.rs`, `retention.rs`), except the recipient choice (C4 below) |
| T13b (e), C1 | **done for removed entries** (`revoked_devices`, LLR-bgj2ff, `org-node/src/verify_org.rs:343-354`), without the S5 TODO and without the residual recorded |
| T16g | **open**: a Member checks an acknowledgement (LLR-5wg8cu, `Checked::Acknowledged`) and keeps nothing; the app's `AcknowledgementReceived` shows nothing |
| T16h, C3 | **open and larger**: `is_own_admin`, `signatory.rs`, REQ-ntkjk6 still in force; domain-api added LLR-7qdjqn (superseding LLR-c9fyun) and kept `is_own_admin` on `org-io/tests/absences.rs`'s allowlist |
| C4 | **open**: recipients are sorted by key bytes, still-listed first (`org-node/src/removal.rs:163-188`), addresses from the address book, `NoAddress` otherwise |

**Owner rulings 2026-11-02:**

- **R1 Scope:** all four — T16g, T16h, C4, and the ledger tidying (T18 below).
- **R2 C4:** the removed Device picks up to three recipients: **one is the Device it received the revocation from** (the epoch's sender, or the Member that answered with the notice), the rest from its list of last-known Organisation Devices.
- **R3 O2:** keep on-chain-client's signatory-set read and its tests, each with a dated note "no consumer since 2026-11-02; kept for the several-Admins stage".
- **R4 O4:** re-point `dev_seed_env.rs`'s co-signer refusal test to LLR-rgdx22 with a dated note on LLR-rgdx22.

**Remaining tasks** (each detailed at T13b against the merged names): **T13b** docs (requirements, risk, design for T16g, C4, T16h deletions, T18); **T16g** keep and show acknowledgements; **T16c4** recipient choice per R2; **T16h** drop `is_own_admin` (§7 footprint plus LLR-7qdjqn, the `absences.rs` allowlist entry, and LLR-c9fyun's chain); **T18** tidying: record on LLR-bgj2ff the residual (a Device admitted and removed between two epochs this Member adopted is never held as revoked, so it is refused) and the S5 TODO, resolve PR-m9betm (acknowledgements are now refused, not skipped: `org-node/src/revocation.rs:156`), optionally fix PR-y6ay44 and PR-7ud5fv; optionally wire `OrgIo::acknowledgement_report` (LLR-rav7dr) into the app. **T17b** close-out.

*Note 2026-11-03 (owner rulings Q1–Q6, §4):* T18 fixes PR-y6ay44 and leaves PR-7ud5fv out of this change (Q4); `OrgIo::acknowledgement_report` gets no app requirement and no app wiring in this change (Q2).

Written 2026-10-16 on branch `worktree-org-io-publish-2`, from local master `e2a65d8` (S4 merged), while domain-api's requirements and design were on its own branch (head `06a3951` when read). Every domain-api ID below is unmerged and is re-read at T13b.

## 1. Owner rulings that bind this plan

From 2026-10-16 (this session, recorded with domain-api's rulings Q1–Q4):

1. **Pull, not push.** An Admin whose write verification has adopted sends the epoch once to the Devices it concerns. There is no automatic resend, no backoff and no resend limit set by an Admin: a Device that missed the send learns of the new chain state at its next periodic verification and requests the Organisation itself. Domain-api carries this (its send, periodic verification and request items).
2. **A revoked Device learns by asking.** A Member answers an organisation request from a Device it holds as revoked with that Device's revocation notice and nothing else (domain-api's answer items).
3. **Verification deletes nothing.** When verification finds this node removed, it reports the Organisation removed. Deletion happens only after the app has deleted its own data for the Organisation and confirmed. *Part 2's* (§3, C2).
4. **The acknowledgement** goes to up to three distinct Member Devices and is retried until three accept or two months pass. A verified revocation notice that arrives later from another known Member may be acknowledged, and that counts as one of the three.
5. **The shared revoked-device CRDT (roadmap S5) is out of scope for now.** In its place each Device keeps **one local revocation log**: never shared, only appended to, kept in the sealed store, with two kinds of entry. A **removed** entry holds the Device keys an epoch removed, appended when this Device adopts that epoch; it gates the notice-only answer. An **acknowledged** entry holds each verified acknowledgement received. The log carries a TODO, in its requirement and in code, that the S5 CRDT replaces it. Residual: a Member that skipped the removing epoch has no **removed** entry and refuses that Device.
6. **`OrgIo::is_own_admin` is dropped.** It has no caller since T16c is dropped (task T16h).
7. **domain-api resolves PR-vt244s.** Domain-api merges first, then this change.

Still binding from earlier:

- **2026-10-06:** "Delete everything" includes the app's data, and the acknowledgement is sent only after the app confirms the deletion.
- **D2, second half (2026-10-08):** the revoked Device's retained bundle is a small file sealed under the forgotten Persona's keychain item, in a second, distinct domain derived from the same seed. It is deleted with that item.
- **D7 (2026-10-08):** "no relay" means no other Device forwards the acknowledgement. The revoked Device dials each kept Device key under its own identity, and iroh's relay server is allowed.
- **S4 ruling 11:** a forgotten Persona's keychain item is kept until the acknowledgement delivery ends or the limit passes, then deleted; that deletion is S3b-io's (LLR-amv4mb's note).
- **Roadmap ruling C**, cut down by ruling 1: only the revoked Device's acknowledgement limit survives, a fixed two months. With nothing set by an Admin there is nothing to store, so D6 no longer applies.

## 2. Part 1's §10, task by task

| Part 1 §10 | Here | Why |
|---|---|---|
| T13b mint proposed 5–9, 11 | **T13b**, reshaped | Proposed 5 and 6 (fan-out, resend) are domain-api's send and pull. Proposed 11 (limits) is dropped. Proposed 7–9 are minted against domain-api's names, plus the revocation log if C1 lands here. |
| T16b per-Persona state file + retained bundle | **T16b**, bundle only | Acknowledgements now go into org-node's store (domain-api's receive item) and the revocation log. No limits are left to store. |
| T16c retry limits | **dropped** | Ruling 1. `is_own_admin` loses its first caller and goes (T16h). |
| T16d revocation resend; delivery of a reconciled commit | **dropped** | Ruling 1. Domain-api's single send, periodic verification and request when behind. PR-vt244s goes with it (ruling 7). |
| T16e receiving a revocation | **T16e**, reshaped | Verification reports removed. The app deletes its data and confirms, then org-io deletes the store, keeps the item and the bundle, and records when the Persona was forgotten. |
| T16f sending the acknowledgement | **T16f**, extended | Ruling 4: a later notice from another Member counts. Retries at start and at each verification interval. Replaces `dial_direct_as`. |
| T16g receiving acknowledgements | **T16g**, shrunk | org-io's inbound side is domain-api's. What is left is the **acknowledged** entry and the app's view of it. |
| — | **T16h**, new | Drop `OrgIo::is_own_admin` (ruling 6). |
| T17b close-out | **T17b** | Resolves no PR. |

## 3. Coordination with `worktree-org-io-domain-api`

Each point needs one answer before T13b. Recommended answers are in **bold**.

- **C1 — who defines the revocation log.** Domain-api's answer items rest on "a Device it holds as revoked" (org-node's organisation-request answer, org-io's inbound and answer items), but no item there says how a Device comes to hold one. *Options:* (a) **domain-api adds the log's requirement with its **removed** entry**, since its answer depends on it; part 2 adds the **acknowledged** entry (T16g); (b) part 2 adds the whole log, and until then domain-api's "held as revoked" means "listed as removed by the Change set kept with the current record", which covers only the latest epoch. *Re-read at `06a3951`:* domain-api's design, commit `6c3e543`, came before ruling Q3. Its LLR-w7b9kd and LLR-283x6f still refuse every organisation request from a Device the current record doesn't list, so its next design pass has to define "held as revoked" either way.
- **C2 — deleting only after confirmation (ruling 3).** Domain-api left ruling Q4 unapplied: its verification item still says "forget the Organisation" (at the next epoch, and on an accepted notice), its no-deletion item still allows the removal step's deletion, and both follow org-node's deletion requirements in force, REQ-uxv2x2 and REQ-em28bq. *Options:* (a) **part 2 owns the change.** Domain-api merges with today's deletion semantics, and part 2's T13b supersedes REQ-uxv2x2, REQ-em28bq, domain-api's verification item and its no-deletion item together with the confirmation step, the retained bundle and the kept item; (b) domain-api applies ruling Q4 before it merges, and part 2 only builds on it. (a) keeps the deletion order, the bundle and the acknowledgement in one change and one review. *Re-read at `06a3951`:* the design deletes inside verification. LLR-jhy6t6 has `verify` return `Removed { acknowledgements }` with `StoreChange::Forgotten`, having signed the acknowledgements, and LLR-9jn3z8 makes that removal `forget_organisation`'s only caller. Under (a), part 2 also supersedes these two LLRs: `verify` reports removed and keeps the store, and a separate confirmed step forgets.
- **C3 — the own-admin check's items.** Domain-api's design adds to a `traces:` line that already cites REQ-f3eu9n (superseded by REQ-ntkjk6). When T16h removes the check, that trace has to drop the ID. Tell domain-api about T16h so it doesn't add new citations. Its org-io design also lists `is_own_admin` on the `OrgIo` surface it keeps; it should leave that method out of the list (or name T16h), and add no new caller.
- **C4 — addresses for the acknowledgement (domain-api answer A2).** Domain-api sends only to the endpoint addresses that org-node keeps per Device key, which are the addresses an admission or organisation request carried, and never to an inbound connection's source. A revoked Device's acknowledgement (T16f) therefore needs addresses for each kept Member Device as well as its key. *Options:* (a) **the bundle keeps, beside each kept Device key, the addresses org-node's address book holds for it at deletion. The delivering Device's addresses come from its revocation notice or answer, which therefore carries the sender's addresses, as requests do.** (b) In Networked mode, dial by key through discovery and the relay (D7) with no stored addresses, accepting that Loopback acknowledgements then reach only Devices whose addresses are known. (a) needs domain-api's notice and answer to carry the sender's addresses, which is a wire change better made there now than superseded later.

## 4. Open questions for the owner

- **O1 — is there a user step in "the app confirms"?** The 2026-10-06 ruling has the app confirming that *it* deleted its data. A step needing the user's consent would let whoever holds a stolen Device decline it for ever. **Recommended: no user step.** The app deletes its data for the Organisation and confirms at once, then tells the user the Organisation was removed.
- **O2 — on-chain-client's signatory-set read.** With `is_own_admin` gone, the pinned signatory-set read (REQ-b8x2py, LLR-sc79gq, LLR-n2vx5q) may have no consumer; T16h's footprint will say. *Options:* (a) **keep it as an exported library read with its tests, noted as having no consumer, for the several-Admins stage**; (b) remove it too. The chain write's forced `Any` proxy type (LLR-59kxsr) stays either way.
- **O3 — what drives the acknowledgement retry.** Once the store is deleted, the revoked Device has no Organisation for periodic verification to visit. **Recommended:** the app's verification interval (domain-api's REQ-ergn9z: configurable, default 10 minutes) also triggers an acknowledgement round for every retained bundle, and so does each start.

### Owner questions from T13b (2026-11-03)

T13b asked the owner nothing. Each question below had its recommended answer
drafted into the ledgers, every such item marked as pending the owner's ruling on its question;
since the rulings below, each such item names the ruling instead.
(These Q-numbers are T13b's; they are not §0's R-rulings or §4's O-questions.)

**Owner rulings 2026-11-03.** The owner accepted every recommendation:

- **Q1: (c).** No Wire change. In Networked mode an acknowledgement whose
  recipient has no kept address that may be dialled is dialled once by the
  recipient's key alone (REQ-kn8zg6, LLR-u2j6u8, superseding LLR-3w22wc);
  Loopback mode is unchanged and its residual stands. Task T16c4, steps 5–6.
- **Q2: no app requirement** for `OrgIo::acknowledgement_report`
  (LLR-rav7dr) in this change; it stays org-io's, exercised by its tests
  (the "not in this requirement" note on app REQ-yvzef9). No task wires it.
- **Q3: accept, keep nothing.** An acknowledgement from a Device the Member
  does not hold as revoked is accepted (REQ-b462sh unchanged) and nothing is
  kept (REQ-h7y5ur, LLR-xudkh4); there is no `AcknowledgementNotRevoked`.
  Task T16g, step 1's second test.
- **Q4: PR-y6ay44 is fixed in T18** with
  `revocation.rs::signed_bytes_are_the_domain_then_the_tuple_with_no_fallback`
  (`verifies: LLR-bs6yxf, PR-y6ay44`) and resolved by this change.
  **PR-7ud5fv is out of scope**: it is left to a change of its own, which
  supersedes LLR-zk7fx2.
- **Q5: the ratings and residuals as drafted.** App HAZ-2hqt34 S2/P2, with
  RC-z8cptv S2/P1, acceptable; the LLR-bgj2ff residual acceptable until
  roadmap S5; org-io's Loopback residual under Q1 (c) acceptable; no rating
  of HAZ-h6b34a or HAZ-7ubhwz changes.
- **Q6: re-point** the merged green tests listed under Q6 below and in §8 to
  their successors, each as *inherited* (never red-first evidence); each
  successor also gets its own red-first test. Tasks T16h, T16c4 and T16g.

The questions as T13b drafted them:

- **Q1 — addresses for the delivering Device (C4, ruling R2).** R2 cannot be
  met in every case without addresses. Domain-api sends only to addresses
  kept per Device key and never to an inbound connection's source; a notice,
  Organisation information and an organisation answer carry no sender
  addresses (`org-node/src/wire.rs`); and the acknowledgement's delivery
  dials only kept addresses, reporting `NoAddress` otherwise (LLR-3w22wc), in
  both modes. The removed Device has an address for the delivering Device
  only when its address book holds one: the issuer of its own Invite, or a
  Device whose organisation request reached it. A notice answered by a Member
  it reached through discovery, or an epoch sent by an Admin Device other
  than its Invite's issuer, leaves none. *Options:* (a) a Wire change:
  Organisation information, the revocation notice and the organisation
  answer carry the sender's own addresses last, as requests do, and the
  receiver keeps them in its address book (supersedes LLR-xr88a8,
  LLR-degg8t, LLR-7kdhst, LLR-j338wq, LLR-h9d65p, LLR-h4gwns, LLR-ds228z and
  org-io's send items); (b) no transport change: the delivering Device is
  first whatever the book holds, and when it holds none that acknowledgement
  is never delivered (one of up to three lost); (c) **recommended:** no Wire
  change; in Networked mode an acknowledgement whose recipient has no kept
  address is dialled once by the recipient's key alone, as org-io's send
  path already does (REQ-kn8zg6, LLR-u2j6u8, superseding LLR-3w22wc); Loopback
  mode is unchanged and keeps the residual, single-host only. (c) meets R2
  wherever discovery can find the key, at the cost of one LLR.
- **Q2 — an app requirement for `OrgIo::acknowledgement_report`
  (LLR-rav7dr)?** It reports a removed Device's own delivery state.
  **Recommended: no, not in this change.** The app already shows the
  Organisation removed; the report would be shown on the removed Device,
  which in the case that matters is lost or stolen; the Organisation's view
  of acknowledgements is the Admin's (T16g). It stays org-io's, exercised by
  its tests, like on-chain-client's read under R3. Drafted as a "not in this
  requirement" note on app REQ-yvzef9.
- **Q3 — an acknowledgement from a Device the Member does not hold as
  revoked.** REQ-b462sh accepts any acknowledgement from a Device the current
  record does not list. **Recommended: still accept it, keep nothing**
  (REQ-h7y5ur, LLR-xudkh4), so the acknowledged record is bounded by the
  Member's own removal history and the revoked Device still counts the
  acceptance. *Alternative:* refuse it (`AcknowledgementNotRevoked`), which
  sends the revoked Device to its next recipient but supersedes REQ-b462sh
  and LLR-5azhry and refuses Members in the LLR-bgj2ff residual.
- **Q4 — PR-y6ay44 and PR-7ud5fv in this change?** PR-y6ay44
  (`signed_bytes`' fallback, `org-node/src/revocation.rs`) fits: its fix
  outline (encode the tuple's fixed-size fields by hand, so no failure path
  exists) leaves LLR-bs6yxf as written, and it sits beside T18's PR-m9betm
  pin. PR-7ud5fv (`seal_removal` drops the behind marker before its write
  succeeds) does not fit as tidying: its fix contradicts the merged green
  test `own_removal.rs::a_store_put_back_over_a_removed_store_replaces_it_with_no_behind_marker`
  (`verifies: LLR-zk7fx2`), so it needs LLR-zk7fx2 superseded.
  **Recommended: fix PR-y6ay44 in T18; leave PR-7ud5fv to a change of its
  own.** org-node then holds 11 open problem reports of its 15.
- **Q5 — ratings and residuals drafted by the analyst.** App HAZ-2hqt34 S2/P2,
  with RC-z8cptv S2/P1, acceptable; the LLR-bgj2ff residual acceptable until
  roadmap S5; org-io's Loopback residual under Q1 (c) acceptable; and no
  rating of HAZ-h6b34a or HAZ-7ubhwz changes. **Recommended: accept as
  drafted.**
- **Q6 — re-annotating merged green tests.** Superseding an LLR leaves its
  merged tests verifying an out-of-force item (`OUT-OF-FORCE-VERIFIES`, below
  in §8). Each still passes under the successor and is re-pointed to it as
  *inherited* (never red-first evidence), as ruling R4 did for one test:
  `epoch_store.rs` ×3 (LLR-d925uu → LLR-f5wrjx), `own_removal.rs` ×2
  (LLR-vq5d5n → LLR-6fsnma), `receive_path.rs:764` (LLR-r4qtqe →
  LLR-gb7p7c), `receive_path.rs:839` (LLR-5wg8cu → LLR-xudkh4),
  `acknowledgements.rs:255` (LLR-3w22wc → LLR-u2j6u8), `absences.rs`'s
  surface test (LLR-4usfu2 → LLR-zhdc9f), `ipc.rs` ×4 (LLR-kk8gc7 →
  LLR-ce4hxz). `acknowledgements.rs:481` asserts Loopback's `NoAddress`,
  which LLR-u2j6u8 keeps, and is re-pointed the same way. **Recommended:
  accept**; each successor also gets its own red-first test.

Also for the coordinator (not the owner's question): org-io's oldest open
problem report, PR-8geawr, is 26 days old against org-io's `problem_age_days`
of 30; the merge gate fails on it from 2026-11-08 unless it is resolved or
accepted first. *2026-11-03:* the owner accepted PR-8geawr (`status:
accepted`, dated note in `org-io/docs/problems/2026-10-08-review.md`); its
fix, a contract test against the real chain, waits for the chopsticks or
live-chain lane.

## 5. Tasks

*Rewritten 2026-11-03 by T13b against master `7ac0825` (§0). The tasks of
2026-10-16 (T16b, T16e, T16f and the old T16g, T16h) are superseded by Q4'
and by these; T13b's own items are listed in its Progress row. Every
`verifies:` below is the exact annotation text. The owner's rulings
of 2026-11-03 on §4's Q1–Q6 (every recommendation accepted) are named where
a task depends on one; the tasks below are as those rulings require.*

Common to every task: work in the task worktree `develop-change` creates;
run each named test with the unit's own command and see it fail for the
reason given before writing code; commit unsigned with
`git -c commit.gpgsign=false commit`, no Co-Authored-By. The units' test
commands:

- org-node: `cargo test -p org-node --features app,test-support --test <target>`
- org-io: `cargo test -p org-io --features test-support --test <target>`
- app (Rust): `cargo test -p app --test <target>` from `app/src-tauri`
- app (webview): `npx vitest run tests/<file>` from `app`

### T13b — Requirements, risk and design (docs only) — done

See Progress. Items minted, superseded and retired are in the T13b
commit's message.

### T16h — Drop `OrgIo::is_own_admin` (ruling 6; §7 footprint and its 2026-11-03 additions)

**Files touched:** `org-io/src/signatory.rs` (deleted), `org-io/src/lib.rs`,
`org-io/src/test_support.rs`, `org-io/src/connect.rs`,
`org-io/tests/signatory_rule.rs` (deleted), `org-io/tests/signatory_key.rs`,
`org-io/tests/dev_seed_env.rs`, `org-io/tests/absences.rs`,
`org-io/tests/common/chain_contract.rs`, `org-io/tests/chain_write.rs`,
`org-io/tests/chain_writer_contract.rs`, `org-io/tests/common/handles.rs`,
`org-io/Cargo.toml`, `org-io/.guardrails/config.yaml`, `Makefile`,
`.github/workflows/rust.yml`
**Parallel:** no (first; T16g also edits `org-io/src/lib.rs` and
`org-io/tests/absences.rs`)

Removes behaviour: no red test of the removed behaviour. The absence is
pinned, watched red against a planted method.

1. In `org-io/tests/absences.rs` add, after
   `the_handles_production_methods_are_exactly_the_domain_surface`:

   ```rust
   // The own-admin check is gone (owner ruling 6): no `signatory` module,
   // no `is_own_admin`, `AdminStatus`, `OwnAdminError`,
   // `SignatorySetReader` or `with_own_account` in org-io's production or
   // test-support source.
   // verifies: LLR-zhdc9f
   #[test]
   fn org_io_offers_no_own_admin_check() {
       let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
       assert!(!src.join("signatory.rs").exists(), "src/signatory.rs is removed");
       for entry in std::fs::read_dir(&src).unwrap() {
           let path = entry.unwrap().path();
           if path.extension().is_some_and(|extension| extension == "rs") {
               let text = std::fs::read_to_string(&path).unwrap();
               for absent in ["is_own_admin", "AdminStatus", "OwnAdminError", "SignatorySetReader", "with_own_account", "mod signatory"] {
                   assert!(!text.contains(absent), "{}: names `{absent}`", path.display());
               }
           }
       }
   }
   ```

   Run `--test absences`: it fails (`src/signatory.rs is removed`).
2. Delete `org-io/src/signatory.rs` and `pub mod signatory;` from
   `org-io/src/lib.rs`; remove `is_own_admin`, `with_own_account`, the
   `signatory_reader`, `own_account` and `co_signers` fields and every place
   that sets them (`unconfigured`, `connect`, `for_test`, `open_for_test`),
   and the doc-comment mentions; in `org-io/src/connect.rs` change the
   comment's `(LLR-qhyc3n: …)` to `(LLR-rgdx22: the writer would dispatch
   with the sender among the other signatories)`, keeping
   `CoSignerIsOwnAccount`. In `org-io/src/test_support.rs` remove
   `SignatorySlots`, the slot setters, `signatory_reads`, the failing flags,
   the share in `sharing_slots_with` and the `SignatorySetReader` impl.
3. Delete `org-io/tests/signatory_rule.rs`; delete `signatory_key.rs`'s
   `a_build_without_dev_seed_says_why_the_own_admin_check_is_not_configured`.
   In `tests/common/chain_contract.rs` remove `signatory_set_contract`, the
   `signatory_set` field of `ChainUnderTest` and its import; in
   `chain_write.rs` (≈597) and `chain_writer_contract.rs` (≈140) drop the
   `signatory_set_contract` call and the field, and change each
   `// verifies: LLR-js7wgk, LLR-j4nkx5, LLR-bwerd2` to
   `// verifies: LLR-js7wgk, LLR-j4nkx5`.
4. In `dev_seed_env.rs` change `connect_refuses_a_co_signer_equal_to_the_own_account`'s
   `// verifies: LLR-qhyc3n` to `// verifies: LLR-rgdx22` (ruling R4;
   inherited: from LLR-qhyc3n).
5. In `absences.rs`: drop `"signatory"`, `"AdminStatus"`, `"OwnAdminError"`
   from `HANDLE_RETURN_ALLOWLIST` and the T16a note; in
   `the_handles_production_methods_are_exactly_the_domain_surface` drop
   `"is_own_admin"` from `expected`, its comment's "and `is_own_admin`", and
   change `// verifies: LLR-4usfu2` to `// verifies: LLR-zhdc9f` (inherited:
   from LLR-4usfu2; T16g adds `removed_devices` to the same list).
6. Drop `--test signatory_rule` from `org-io/.guardrails/config.yaml`'s
   first verify line, from `Makefile`'s `ORG_IO_COVERAGE_ARGS` (≈604) with
   the `signatory.rs` shortfall lines (≈288–332), from
   `.github/workflows/rust.yml` (≈104), and its `[[test]]` and the comment
   at `org-io/Cargo.toml:115-119`; drop the own-admin prose in
   `org-io/tests/common/handles.rs`.
7. Run `--test absences`: green. Plant `pub fn is_own_admin(&self) {}` in
   `lib.rs`: `org_io_offers_no_own_admin_check` fails; remove it. Then run
   org-io's whole first verify line and `cargo clippy -p org-io
   --all-features -- -D warnings`.

red -> green: `org-io/tests/absences.rs::org_io_offers_no_own_admin_check (verifies: LLR-zhdc9f)` — red: `src/signatory.rs is removed`; green after steps 2–6; re-red against a planted `pub fn is_own_admin`.
inherited: `dev_seed_env.rs::connect_refuses_a_co_signer_equal_to_the_own_account` — from LLR-qhyc3n (to LLR-rgdx22, ruling R4).
inherited: `absences.rs::the_handles_production_methods_are_exactly_the_domain_surface` — from LLR-4usfu2 (to LLR-zhdc9f).

Commit: `refactor(org-io): drop the own-admin check (ruling 6; REQ-ntkjk6 retired)`.

**Done 2026-11-03** (ce7b7d0, merged): red -> green and inherited lines as above, confirmed by the dispatch report; org-io verify line 1 318/318, line 2 11/11, build and both clippy clean. Footprint additions found: `test_support::ProductionChain`'s signatory reader and `controller` fields, FakeChain's `original_account_failing`/`delegates_failing`, chain_contract.rs clause 4 (clauses renumbered). Left for T18: `absences.rs::the_handle_returns_only_allowlisted_public_types` still cites the superseded LLR-4usfu2 beside LLR-gh4rw3; re-point that ID to LLR-zhdc9f (Q6).

**Owner ruling 2026-11-03:** org-io's `problem_age_days` raised from 30 to 60 (org-node and app already 60), instead of fixing or accepting PR-z3gg2b and the 2026-10-14 org-io PRs now.

### T16c4 — The delivering Device is the first recipient (ruling R2; Q1 (c))

**Files touched:** `org-node/src/store.rs`, `org-node/src/service.rs`,
`org-node/src/removal.rs`, `org-node/src/verify_org.rs`,
`org-node/tests/own_removal.rs`, `org-node/tests/receive_path.rs`,
`org-node/tests/epoch_store.rs`, `org-node/tests/support/fixtures.rs`,
`org-io/src/acknowledge.rs`, `org-io/src/test_support.rs`,
`org-io/tests/acknowledgements.rs`
**Parallel:** no (after T16h, which edits `org-io/src/test_support.rs`)

1. **The notice's sender is kept (LLR-gb7p7c).** In
   `org-node/tests/receive_path.rs`, in
   `a_notice_check_notice_passes_is_kept_and_nothing_is_deleted`, change
   `// verifies: LLR-r4qtqe` to `// verifies: LLR-gb7p7c`, and the two
   expectations to the kept form:

   ```rust
   expected.kept_revocation_notices.push(org_node::store::KeptNotice { notice: notice.clone(), from: ada_device });
   // …
   assert_eq!(
       reopened.data().unwrap().kept_revocation_notices,
       vec![org_node::store::KeptNotice { notice: notice.clone(), from: ada_device }]
   );
   ```

   Run `--test receive_path`: compile red (`KeptNotice` does not exist).
2. In `org-node/src/store.rs` add

   ```rust
   /// A revocation notice kept with the Device the transport authenticated for it (LLR-gb7p7c).
   #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
   pub struct KeptNotice {
       pub notice: RevocationNotice,
       pub from: DevicePublicKey,
   }
   ```

   with a `RawKeptNotice { notice: RevocationNotice, from: [u8; 32] }`
   mirror parsed by `parse_field("org.kept_revocation_notices",
   DevicePublicKey::parse(&raw.from))`; change
   `StoreData.kept_revocation_notices` and its raw and borrowed mirrors to
   `Vec<KeptNotice>`; in `service.rs`'s `Revocation` arm push `KeptNotice {
   notice, from: sender }`; in `verify_org.rs` make `accepted_notice`
   return `Option<&KeptNotice>`, the first in kept order `revocation::accept`
   takes, and `ensure_within_bound`'s measure take the new type unchanged.
   Port every fixture and test that builds a kept notice to
   `KeptNotice { notice, from: device_key(0x21) }` (`own_removal.rs` ≈84,
   ≈443, ≈574; `receive_path.rs`'s `changes_filling_the_bound`;
   `support/fixtures.rs`). Run `--test receive_path`: green.
3. **The delivering Device first (LLR-6fsnma).** In `own_removal.rs` add:

   ```rust
   /// `store_data` holding one provisional organisation for epoch 4 whose members are `next`, from `origin`.
   fn with_held_from(mut store_data: StoreData, next: Vec<MemberSnapshot>, seed: u8, origin: Origin) -> StoreData {
       let (private, public) = org_key_pair(seed);
       store_data.provisional_organisations.push(ProvisionalOrganisation::from(ProvisionalOrganisationParts {
           epoch: Epoch::new(4),
           base_root: store_data.record.as_ref().unwrap().root_hash,
           root: root_of(&next),
           members: next,
           org_pub_key: public,
           org_private_key: private,
           change_set: Some(vec![1]),
           base_snapshot: Some(vec![2]),
           origin,
           applied: Vec::new(),
           first_ahead_at: None,
       }));
       store_data
   }

   fn recipients_after(store_data: StoreData, chain: OrgState) -> Vec<DevicePublicKey> {
       let mut svc = own_service(store_data);
       let (verified, change) = svc
           .verify(&mut OsRng, &own(), Some(chain), VerificationTime(12), org_node::AheadLimit(1200), |_: &[DevicePublicKey]| vec![own_secret()])
           .unwrap();
       assert!(matches!(verified, Verified::Removed { .. }), "{verified:?}");
       removed_of(change).acknowledgements.iter().map(|kept| kept.recipient).collect()
   }

   // Normal: on a removal by a match the Device the match came from is the
   // first recipient, then the others in LLR-vq5d5n's order; with k = 1 it
   // is the only one. Each fixture picks a delivering Device the old order
   // would not take first. Boundary: an `Own` match has no delivering
   // Device, and the old order stands.
   // verifies: LLR-6fsnma
   #[test]
   fn the_device_a_match_came_from_is_the_first_recipient() {
       let all = ascending(&[0x31, 0x32, 0x33, 0x34, 0x35]);
       let next = others(5);
       let from_highest = with_held_from(member_data(others(5)), next.clone(), 0x63, Origin::Received { from: all[4] });
       assert_eq!(recipients_after(from_highest, chain_listing(&next, 4, 0x63)), vec![all[4], all[0], all[1]]);

       let two = ascending(&[0x31, 0x32]);
       let answered = with_held_from(member_data(others(2)), others(2), 0x63, Origin::Answer { from: two[1] });
       assert_eq!(recipients_after(answered, chain_listing(&others(2), 4, 0x63)), vec![two[1]], "k = 1: the delivering Device alone");

       let own_match = with_held_from(member_data(others(5)), next.clone(), 0x63, Origin::Own);
       assert_eq!(recipients_after(own_match, chain_listing(&next, 4, 0x63)), all[..3].to_vec(), "no delivering Device: the old order");
   }

   // Normal: on a removal by a kept notice its sender is the first
   // recipient, then the others ascending; of two kept notices the first
   // that `accept` takes names it.
   // verifies: LLR-6fsnma, LLR-gb7p7c
   #[test]
   fn the_sender_of_the_accepted_notice_is_the_first_recipient() {
       let all = ascending(&[0x31, 0x32, 0x33, 0x34, 0x35]);
       let mut store_data = member_data(others(5));
       let notice = notice_removing_own(&store_data);
       let mut refused = notice.clone();
       refused.member_id = MemberId::new([0x0f; 32]);
       store_data.kept_revocation_notices = vec![
           KeptNotice { notice: refused, from: all[2] },
           KeptNotice { notice, from: all[3] },
       ];
       let without = removed_members(&store_data);
       assert_eq!(recipients_after(store_data, chain_listing(&without, 4, 0x63)), vec![all[3], all[0], all[1]]);
   }
   ```

   (import `org_node::store::KeptNotice`). Run `--test own_removal`: both
   fail on the first `assert_eq!` (left `[all[0], all[1], all[2]]`).
4. In `removal.rs` give `recipients` a `delivering: Option<&DevicePublicKey>`
   parameter: after choosing `taken` and ordering as today, when
   `delivering` is a candidate and `taken > 0`, put it first and take the
   first `taken - 1` of the rest. In `service.rs` pass, for a match, the
   `from` of its `Origin` (`None` for `Own`) and, for a notice, the `from`
   of `accepted_notice`'s `KeptNotice` into `remove_own`. Update the doc
   comments to cite LLR-6fsnma. Run `--test own_removal`: green. Re-point
   `a_removal_by_a_match_keeps_three_two_one_or_no_recipients` and
   `the_still_listed_come_first_on_a_match_and_all_ascending_on_a_notice`
   to `// verifies: LLR-6fsnma` (inherited: from LLR-vq5d5n; their
   delivering Device, `with_received`'s `0x21`, is no candidate, so they
   stand).
5. **The dial by key (LLR-u2j6u8; owner ruling Q1 (c)).** In `org-io/src/acknowledge.rs`
   extract the pure `pub(crate) fn dial_plan(mode: TransportMode, recipient:
   &DevicePublicKey, addresses: &[String]) -> Vec<Vec<SocketAddr>>`: each
   dialable socket alone, in order; none dialable in Networked mode, one
   empty socket list (a dial by key); none in Loopback, no dial. Re-export it
   from `test_support` as `acknowledgement_dial_plan`. In
   `org-io/tests/acknowledgements.rs` add:

   ```rust
   // Normal: kept addresses are dialled one at a time, in order. Boundary:
   // with none that may be dialled, Networked mode dials once by the key
   // alone (an empty socket list) and Loopback mode dials nothing.
   // verifies: LLR-u2j6u8
   #[test]
   fn a_recipient_with_no_dialable_address_is_dialled_by_key_in_networked_mode_only() {
       use org_io::test_support::acknowledgement_dial_plan;
       use org_io::TransportMode;
       let recipient = keep_device_seed([0x77; 32]);
       let at = format!("{}@127.0.0.1:4433", hex::encode(recipient.as_bytes()));
       let two = vec![at.clone(), format!("{}@127.0.0.1:4434", hex::encode(recipient.as_bytes()))];
       assert_eq!(acknowledgement_dial_plan(TransportMode::Networked, &recipient, &two).len(), 2, "each address alone");
       assert_eq!(acknowledgement_dial_plan(TransportMode::Networked, &recipient, &[]), vec![Vec::new()], "one dial by key");
       assert!(acknowledgement_dial_plan(TransportMode::Loopback, &recipient, &[]).is_empty(), "Loopback: no dial");
   }
   ```

   (the address text is the form `parse_kept_address` reads; take it from
   the existing `journey::keep_addresses` callers if it differs). Run
   `--test acknowledgements`: compile red (no `acknowledgement_dial_plan`),
   then, with the function returning today's plan, red on `one dial by key`
   (left `[]`).
6. Make `try_addresses` iterate `dial_plan`; green. Re-point
   `a_round_tries_each_recipients_addresses_in_order_until_accepted` and
   `a_recipient_with_no_address_of_its_own_is_not_delivered_for_no_address`
   to `// verifies: LLR-u2j6u8` (inherited: from LLR-3w22wc).
7. Run org-node's and org-io's whole first verify lines and clippy.

red -> green: `org-node/tests/receive_path.rs::a_notice_check_notice_passes_is_kept_and_nothing_is_deleted (verifies: LLR-gb7p7c)` — compile red: no `KeptNotice`. (Rewritten for the successor; its old body is inherited coverage, the kept-form assertions the new evidence.)
red -> green: `org-node/tests/own_removal.rs::the_device_a_match_came_from_is_the_first_recipient (verifies: LLR-6fsnma)` — red: left `[all[0], all[1], all[2]]`.
red -> green: `org-node/tests/own_removal.rs::the_sender_of_the_accepted_notice_is_the_first_recipient (verifies: LLR-6fsnma, LLR-gb7p7c)` — red: left `[all[0], all[1], all[2]]`.
red -> green: `org-io/tests/acknowledgements.rs::a_recipient_with_no_dialable_address_is_dialled_by_key_in_networked_mode_only (verifies: LLR-u2j6u8)` — red: `one dial by key`, left `[]`.
inherited: `own_removal.rs::a_removal_by_a_match_keeps_three_two_one_or_no_recipients`, `the_still_listed_come_first_on_a_match_and_all_ascending_on_a_notice` — from LLR-vq5d5n; `acknowledgements.rs::a_round_tries_each_recipients_addresses_in_order_until_accepted`, `a_recipient_with_no_address_of_its_own_is_not_delivered_for_no_address` — from LLR-3w22wc.

Commits: `feat(org-node): the delivering Device is the first recipient (LLR-6fsnma, LLR-gb7p7c)`; `feat(org-io): an acknowledgement with no dialable address is dialled by key in Networked mode (LLR-u2j6u8)`.

**T16c4 done 2026-11-03** (merged; eda7951 + feat(org-io)). Dispatch report:

```
red -> green: org-node/tests/receive_path.rs::a_notice_check_notice_passes_is_kept_and_nothing_is_deleted (verifies: LLR-gb7p7c) — red: compile error, KeptNotice did not exist
red -> green: org-node/tests/own_removal.rs::the_device_a_match_came_from_is_the_first_recipient (verifies: LLR-6fsnma) — red: left [all[0], all[1], all[2]] right [all[4], all[0], all[1]]
red -> green: org-node/tests/own_removal.rs::the_sender_of_the_accepted_notice_is_the_first_recipient (verifies: LLR-6fsnma, LLR-gb7p7c) — red: left [all[0], all[1], all[2]] right [all[3], all[0], all[1]]
red -> green: org-io/tests/acknowledgements.rs::a_recipient_with_no_dialable_address_is_dialled_by_key_in_networked_mode_only (verifies: LLR-u2j6u8) — red: no acknowledgement_dial_plan, then "one dial by key" left [] right [[]]
inherited: own_removal.rs::a_removal_by_a_match_keeps_three_two_one_or_no_recipients — from LLR-vq5d5n (to LLR-6fsnma)
inherited: own_removal.rs::the_still_listed_come_first_on_a_match_and_all_ascending_on_a_notice — from LLR-vq5d5n (to LLR-6fsnma)
inherited: acknowledgements.rs::a_round_tries_each_recipients_addresses_in_order_until_accepted — from LLR-3w22wc (to LLR-u2j6u8)
inherited: acknowledgements.rs::a_recipient_with_no_address_of_its_own_is_not_delivered_for_no_address — from LLR-3w22wc (to LLR-u2j6u8)
```

Results: org-node 381/381, quint 5/5, clippy clean; org-io 319/319, dev-seed 11/11, build and clippy clean. GOLDEN_ACTIVE_STORE re-derived, 2108 → 2140 bytes (each kept notice encoded as (notice, from), 32-byte sender key spliced after the kept notice's proof; checked against an independent byte splice). Extra files ported: persona_records.rs, provisional_store.rs, verification.rs. Left for T18: receive_path.rs::a_kept_notice_counts_against_the_bound still cites LLR-r4qtqe beside LLR-m76r4k; acknowledgements.rs::already_received_or_invite_used_keeps_the_acknowledgement cites LLR-3w22wc beside LLR-xwz3nu (re-point by Q6). Left for T16g: store.rs doc comments citing LLR-d925uu.

### T16g — Keeping and showing acknowledgements

**Files touched:** `org-node/src/store.rs`, `org-node/src/service.rs`,
`org-node/tests/receive_path.rs`, `org-node/tests/own_removal.rs`,
`org-node/tests/epoch_store.rs`, `org-io/src/lib.rs`,
`org-io/src/removed.rs` (new), `org-io/tests/acknowledgement_inbound.rs`,
`org-io/tests/absences.rs`, `app/src-tauri/src/commands.rs`,
`app/src-tauri/src/lib.rs`, `app/src-tauri/tests/ipc.rs`,
`app/tests/contract/backend.json`, `app/src/lib/api.ts`,
`app/src/lib/removed.ts` (new), `app/src/lib/components/RemovedDevices.svelte`
(new), `app/src/routes/+page.svelte` (as merged: `app/src/lib/components/Revoke.svelte`, since the page has no Organisation selection; MR1-A4), `app/tests/removed.view.test.ts` (new)
**Parallel:** no (after T16c4: both edit `org-node/src/store.rs`,
`service.rs` and the org-node tests; after T16h: `org-io/src/lib.rs`,
`absences.rs`)

*org-node (SDD-acnpu4).*

1. In `org-node/tests/receive_path.rs` add:

   ```rust
   // Normal: an acknowledgement from a Device held as revoked is accepted,
   // kept once with the MemberId it was listed under, and sealed; a second
   // one from that Device, for another epoch, is accepted and keeps
   // nothing.
   // verifies: LLR-xudkh4, LLR-py43n3
   #[test]
   fn an_acknowledgement_from_a_device_held_as_revoked_is_kept_once_and_sealed() {
       let ada = org_node::test_fixtures::device_key(notices::ADA);
       let dave = org_node::test_fixtures::device_key(notices::DAVE);
       let ack = notices::ack_by_dave(3);
       let body = org_node::wire::encode_message(&org_node::wire::WireMessage::Acknowledgement(ack.clone())).unwrap();
       let mut store_data = notices::ada_store(3, &[notices::ada()]);
       store_data.revoked_devices = vec![org_node::store::RevokedDevice { member_id: notices::dave().id, device_key: dave }];
       let mut svc = notices::service(notices::ADA, store_data);

       let (checked, change) = svc.check_received(&mut rand::rngs::OsRng, &ada, dave, &body).unwrap();
       assert!(matches!(checked, org_node::service::Checked::Acknowledged(_)));
       let Some(org_node::service::StoreChange::Sealed(write)) = change else { panic!("kept and sealed") };
       let reopened = org_node::store::PersonaStore::unseal(&write.sealed, ada, org_node::test_fixtures::device_secret([notices::ADA; 32])).unwrap();
       assert_eq!(
           reopened.data().unwrap().acknowledged,
           vec![org_node::store::KeptAcknowledgementFrom { ack, member_id: notices::dave().id }]
       );

       let again = org_node::wire::encode_message(&org_node::wire::WireMessage::Acknowledgement(notices::ack_by_dave(2))).unwrap();
       let before = notices::encoded(&svc, notices::ADA);
       let (checked, change) = svc.check_received(&mut rand::rngs::OsRng, &ada, dave, &again).unwrap();
       assert!(matches!(checked, org_node::service::Checked::Acknowledged(_)));
       assert!(change.is_none(), "one entry per Device");
       assert_eq!(notices::encoded(&svc, notices::ADA), before);
   }

   // Abnormal: from a Device the record does not list and the store does
   // not hold as revoked, the acknowledgement is accepted and nothing is
   // kept (owner ruling Q3 of 2026-11-03).
   // verifies: LLR-xudkh4
   #[test]
   fn an_acknowledgement_from_a_device_not_held_as_revoked_is_accepted_and_not_kept() {
       let ada = org_node::test_fixtures::device_key(notices::ADA);
       let dave = org_node::test_fixtures::device_key(notices::DAVE);
       let body = org_node::wire::encode_message(&org_node::wire::WireMessage::Acknowledgement(notices::ack_by_dave(3))).unwrap();
       let mut svc = notices::service(notices::ADA, notices::ada_store(3, &[notices::ada()]));
       let (checked, change) = svc.check_received(&mut rand::rngs::OsRng, &ada, dave, &body).unwrap();
       assert!(matches!(checked, org_node::service::Checked::Acknowledged(_)));
       assert!(change.is_none());
       assert!(svc.store_data(&ada).unwrap().acknowledged.is_empty());
   }

   // Normal: each Device held as revoked, in held order, with the
   // acknowledgement kept from it or none. Abnormal: an unknown key is
   // `PersonaNotHeld`; a store with no record `OrgNotHeld`. It changes
   // nothing.
   // verifies: LLR-dsjht5
   #[test]
   fn removed_devices_lists_each_device_held_as_revoked_with_its_acknowledgement() {
       use org_node::service::RemovedDeviceView;
       use org_node::store::{KeptAcknowledgementFrom, RevokedDevice};
       let ada = org_node::test_fixtures::device_key(notices::ADA);
       let dave = org_node::test_fixtures::device_key(notices::DAVE);
       let erin = org_node::test_fixtures::device_key(0x2e);
       let mut store_data = notices::ada_store(3, &[notices::ada()]);
       store_data.revoked_devices = vec![
           RevokedDevice { member_id: notices::dave().id, device_key: dave },
           RevokedDevice { member_id: org_node::MemberId::new([0x0e; 32]), device_key: erin },
       ];
       store_data.acknowledged = vec![KeptAcknowledgementFrom { ack: notices::ack_by_dave(3), member_id: notices::dave().id }];
       let svc = notices::service(notices::ADA, store_data);
       let before = notices::encoded(&svc, notices::ADA);
       assert_eq!(
           svc.removed_devices(&ada).unwrap(),
           vec![
               RemovedDeviceView { member_id: notices::dave().id, device_key: dave, acknowledgement: Some(notices::ack_by_dave(3)) },
               RemovedDeviceView { member_id: org_node::MemberId::new([0x0e; 32]), device_key: erin, acknowledgement: None },
           ]
       );
       assert_eq!(notices::encoded(&svc, notices::ADA), before);
       assert!(matches!(svc.removed_devices(&org_node::test_fixtures::device_key(0x7f)), Err(org_node::error::OrgNodeError::PersonaNotHeld { .. })));
       let unrecorded = notices::service(notices::ADA, org_node::test_fixtures::empty_store(notices::persona(&notices::ada(), 0x61)));
       assert!(matches!(unrecorded.removed_devices(&ada), Err(org_node::error::OrgNodeError::OrgNotHeld { .. } | org_node::error::OrgNodeError::FoundingNotWritten)));
   }
   ```

   In `own_removal.rs` add:

   ```rust
   // Normal: an adoption keeps the acknowledged record whole. Abnormal (a
   // scan): the field and its append carry the S5 TODO.
   // verifies: LLR-py43n3
   #[test]
   fn an_adoption_keeps_the_acknowledged_record_and_the_field_carries_the_s5_todo() {
       let bob = snapshot(2, "bob", 0x11, &[0x21]);
       let mut store_data = member_data(vec![bob.clone()]);
       let entry = org_node::store::KeptAcknowledgementFrom { ack: sample_ack(0x5b), member_id: MemberId::new([9; 32]) };
       store_data.acknowledged = vec![entry.clone()];
       let next = vec![alice(), bob];
       let mut svc = own_service(with_received(store_data, next.clone(), 0x63));
       let (verified, _) =
           svc.verify(&mut OsRng, &own(), Some(chain_listing(&next, 4, 0x63)), VerificationTime(10), org_node::AheadLimit(1200), never_asked).unwrap();
       assert!(matches!(verified, Verified::Adopted { .. }), "{verified:?}");
       assert_eq!(svc.store_data(&own()).unwrap().acknowledged, vec![entry]);
       let source = include_str!("../src/store.rs");
       let field = source.find("pub acknowledged:").expect("the field");
       assert!(source[..field].rfind("TODO(S5): the S5 revoked-device CRDT replaces this record").is_some_and(|todo| field - todo < 400), "the TODO sits on the field");
       assert!(include_str!("../src/service.rs").contains("TODO(S5): the S5 revoked-device CRDT replaces this record"), "and on the append");
   }
   ```

   In `epoch_store.rs` add (and give `full_store()` one `KeptNotice` and one
   `acknowledged` entry):

   ```rust
   // Normal: the store's encoding is its fourteen encoded fields, in
   // LLR-f5wrjx's order, the kept notices with their senders and the
   // acknowledged record last; sealed and opened it is the same value.
   // verifies: LLR-f5wrjx
   #[test]
   fn store_data_holds_its_fourteen_fields_in_order_with_the_notices_senders_and_the_acknowledged_record() {
       let store_data = full_store();
       assert!(!store_data.acknowledged.is_empty() && !store_data.kept_revocation_notices.is_empty(), "fixture holds both");
       let in_order = postcard::to_allocvec(&(
           (&store_data.persona, &store_data.record, &store_data.org_name, &store_data.provisional_changes, &store_data.provisional_organisations),
           (&store_data.founding, &store_data.member, &store_data.kept_admission_requests, &store_data.kept_revocation_notices),
           (&store_data.expected_admissions, &store_data.outstanding_invites, &store_data.revoked_devices, &store_data.address_book),
           &store_data.acknowledged,
       ))
       .unwrap();
       let encoded = postcard::to_allocvec(&store_data).unwrap();
       assert_eq!(encoded, in_order, "the fields in LLR-f5wrjx's order and nothing else");
       let reopened = round_trip(store_data);
       assert_eq!(postcard::to_allocvec(&reopened).unwrap(), encoded, "every encoded field reads back unchanged");
   }
   ```

   Run `--test receive_path --test own_removal --test epoch_store`: compile
   red (no `acknowledged`, `KeptAcknowledgementFrom`, `removed_devices`,
   `RemovedDeviceView`).
2. In `store.rs` add, last in `StoreData` and its raw and borrowed mirrors:

   ```rust
   /// The acknowledged record (LLR-py43n3): one entry per Device held as revoked, only appended.
   // TODO(S5): the S5 revoked-device CRDT replaces this record.
   pub acknowledged: Vec<KeptAcknowledgementFrom>,
   ```

   with `KeptAcknowledgementFrom { pub ack: Acknowledgement, pub member_id:
   MemberId }` (`Clone, Debug, PartialEq, Eq, Serialize`; raw mirror parsing
   `ack.device` under `org.acknowledged`). In `service.rs`'s
   `Acknowledgement` arm, after `check_acknowledgement`: when
   `data.revoked_devices` holds `ack.device`, the record does not list it
   and `data.acknowledged` has no entry for it, append through
   `self.change` with the comment `// TODO(S5): the S5 revoked-device CRDT
   replaces this record (LLR-xudkh4)` and return `Some(StoreChange::Sealed)`.
   Add `pub fn removed_devices(&self, device_key) -> Result<Vec<RemovedDeviceView>,
   OrgNodeError>` and `RemovedDeviceView { pub member_id, pub device_key, pub
   acknowledgement: Option<Acknowledgement> }` (`Debug, PartialEq, Eq`) per
   LLR-dsjht5, using `current_record` for the refusals. Green.
3. Re-point `an_acknowledgement_is_decided_by_check_acknowledgement_with_the_sender_and_no_store_change`
   (`receive_path.rs:839`) to `// verifies: LLR-xudkh4` (inherited: from
   LLR-5wg8cu; its Dave is not held as revoked, so no store change still
   holds) and the three `epoch_store.rs` tests at ≈133, ≈161, ≈175 to
   `// verifies: LLR-f5wrjx` (inherited: from LLR-d925uu), the first one's
   comment "thirteen" kept as the 2026-10-16 count it checked.

*org-io.*

4. In `org-io/tests/acknowledgement_inbound.rs` add:

   ```rust
   // Normal: the Admin lists Bob as removed once its epoch removing him is
   // adopted, with no acknowledgement, and with one once Bob's own delivery
   // has been accepted by the Admin's receive step. Abnormal: an
   // Organisation no Persona holds is `NotHeld`.
   // verifies: LLR-3ydu9e
   #[tokio::test(flavor = "multi_thread")]
   async fn the_admin_lists_a_removed_device_and_then_its_acknowledgement() {
       use org_io::{RemovedDevice, RemovedDevicesRefused};
       let (_guard, _keychain) = fresh_keychain();
       let tag = "ack-in-removed-list";
       let admin = journey::admin(tag).await;
       admin.io.start_receiving().await.unwrap();
       let bob = journey::invited(&admin, tag, "b", "bob").await;
       journey::admit_over_transport(&admin, &bob).await;
       assert_eq!(admin.io.removed_devices(admin.org).await.unwrap(), vec![], "none removed yet");
       journey::revoke(&admin, bob.device_key).await;
       let listed = admin.io.removed_devices(admin.org).await.unwrap();
       assert!(matches!(listed.as_slice(), [RemovedDevice { device_key, acknowledged_epoch: None, .. }] if *device_key == bob.device_key), "{listed:?}");
       let (notice, _) = journey::deliver_epoch(&admin.io, admin.org, &bob.io).await;
       assert!(matches!(notice, Ok(Received::RevocationNoticeKept { .. })), "{notice:?}");
       let (received, verified) = tokio::join!(journey::next(&admin.io), bob.io.verify_all());
       assert!(matches!(verified.as_slice(), [(_, Verified::Removed { .. })]), "{verified:?}");
       assert!(matches!(received, Ok(Received::Acknowledged(_))), "{received:?}");
       let listed = admin.io.removed_devices(admin.org).await.unwrap();
       assert!(matches!(listed.as_slice(), [RemovedDevice { device_key, acknowledged_epoch: Some(_), .. }] if *device_key == bob.device_key), "{listed:?}");
       assert!(matches!(admin.io.removed_devices(org_node::OrgId::new([0xee; 20])).await, Err(RemovedDevicesRefused::NotHeld { .. })));
   }
   ```

   (`journey::revoke` adopts the removing epoch on the Admin, as
   `a_revoked_devices_acknowledgement_over_a_direct_connection_is_accepted_and_answered_accepted`
   relies on.) Compile red: no `removed_devices`.
5. Add `org-io/src/removed.rs` with `RemovedDevice`, `RemovedDevicesRefused`
   (`NotHeld { org_id }`, `Node(OrgNodeError)`, `Display`, `Error`) and
   `impl OrgIo { pub async fn removed_devices(&self, org_id) -> … }` per
   LLR-3ydu9e; `mod removed;` and `pub use crate::removed::{RemovedDevice,
   RemovedDevicesRefused};` in `lib.rs`. In `absences.rs` add
   `"removed_devices"` to `expected` (after `"request_organisation"`, the
   list is sorted) and `"RemovedDevice"`, `"RemovedDevicesRefused"` to
   `HANDLE_RETURN_ALLOWLIST` and `DOMAIN_RETURN_TYPES`. Green.

*app (SDD-6pjvhc, SDD-g9ugu7).*

6. In `app/src-tauri/tests/ipc.rs`: register `commands::list_removed_devices`
   in the harness list (≈101); in `product_registers_the_same_commands_as_the_harness`
   change `17`/"seventeen" to `18`/"eighteen" and `// verifies: LLR-kk8gc7`
   to `// verifies: LLR-ce4hxz` (that test is rewritten for the successor:
   red on the count); re-point the three other `LLR-kk8gc7`-only tests
   (≈318, ≈1351, ≈1617) to `LLR-ce4hxz` (inherited); and add:

   ```rust
   // Normal: a fresh store's Organisation-free call is refused as org-io
   // refuses it; abnormal: a malformed Organisation identifier is refused
   // at the IPC edge. The listing's shape is backend.json's
   // `list_removed_devices` entries, driven by the contract test.
   // verifies: LLR-dmfvv6
   #[test]
   fn list_removed_devices_refuses_an_unheld_or_malformed_organisation() {
       let harness = harness();
       let unheld = invoke(&harness, "list_removed_devices", serde_json::json!({ "orgId": "ab".repeat(20) }));
       assert!(unheld.as_ref().is_err_and(|text| text.contains("not held")), "{unheld:?}");
       let malformed = invoke(&harness, "list_removed_devices", serde_json::json!({ "orgId": "zz" }));
       assert!(malformed.is_err(), "{malformed:?}");
   }
   ```

   and to `app/tests/contract/backend.json` under `answers`:

   ```json
   "list_removed_devices": {
       "none": [],
       "one_acknowledged": [
           {
               "member_id": "0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e",
               "device_key": "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd",
               "acknowledged_epoch": 4
           }
       ]
   }
   ```

   with the contract test's driver for it built as
   `list_admission_requests`' is (an admitted then revoked Member whose
   acknowledgement the inviter received). Run `--test ipc`: red (17 ≠ 18;
   unknown command).
7. Add `list_removed_devices` in `commands.rs` per LLR-dmfvv6 and to
   `lib.rs`'s `generate_handler!`. Green.
8. Add `app/tests/removed.view.test.ts`:

   ```ts
   // @vitest-environment jsdom
   /**
    * SDD-g9ugu7: the removed Devices view's words (LLR-cs9bmw) and when it
    * reads the list (LLR-x9qbhu), over the faked Tauri boundary.
    */
   import { describe, it, expect, vi, beforeEach } from 'vitest';
   vi.mock('@tauri-apps/api/core', async () => ({ invoke: (await import('./fake-tauri')).tauri.invoke }));
   vi.mock('@tauri-apps/api/event', async () => ({ listen: (await import('./fake-tauri')).tauri.listen }));
   import { render, screen, fireEvent } from '@testing-library/svelte';
   import { tauri } from './fake-tauri';
   import { settle } from './page';
   import RemovedDevices from '../src/lib/components/RemovedDevices.svelte';
   import { removedDeviceLine } from '../src/lib/removed';

   const ORG = 'ab'.repeat(20);
   const ACKED = { member_id: '0e'.repeat(32), device_key: 'cd'.repeat(32), acknowledged_epoch: 4 };
   const SILENT = { member_id: '0f'.repeat(32), device_key: 'ce'.repeat(32), acknowledged_epoch: null };

   beforeEach(() => tauri.reset());

   describe('removed Devices view', () => {
   	// verifies: LLR-cs9bmw
   	it('words an acknowledgement as the Device statement, never as deletion', async () => {
   		expect(removedDeviceLine(ACKED)).toBe('acknowledged its removal at epoch 4');
   		expect(removedDeviceLine(SILENT)).toBe('no acknowledgement received by this Device');
   		tauri.answer('list_removed_devices', [ACKED, SILENT]);
   		render(RemovedDevices, { orgId: ORG });
   		await settle();
   		expect(screen.getByText('Removed Devices (Admin)')).toBeTruthy();
   		expect(
   			screen.getByText(
   				"An acknowledgement is the removed Device's own signed statement that it acted on its removal. It is not proof that its data was deleted."
   			)
   		).toBeTruthy();
   		const text = document.body.textContent ?? '';
   		expect(text.replace('not proof that its data was deleted', '')).not.toMatch(/deleted|erased|wiped/);
   	});

   	// verifies: LLR-cs9bmw
   	it('says so when no Device is removed, and shows a refusal as its text', async () => {
   		tauri.answer('list_removed_devices', []);
   		const first = render(RemovedDevices, { orgId: ORG });
   		await settle();
   		expect(screen.getByText('No removed Devices.')).toBeTruthy();
   		first.unmount();
   		tauri.refuse('list_removed_devices', 'organisation not held');
   		render(RemovedDevices, { orgId: ORG });
   		await settle();
   		expect(screen.getByText('organisation not held')).toBeTruthy();
   	});

   	// verifies: LLR-x9qbhu
   	it('reads the list when shown, on Refresh, and after a verification result for its Organisation only', async () => {
   		tauri.answer('list_removed_devices', [SILENT]);
   		render(RemovedDevices, { orgId: ORG });
   		await settle();
   		expect(tauri.calls('list_removed_devices')).toEqual([{ orgId: ORG }]);
   		await fireEvent.click(screen.getByRole('button', { name: 'Refresh' }));
   		await settle();
   		expect(tauri.calls('list_removed_devices')).toHaveLength(2);
   		tauri.emit('verification-result', { org_id: 'cd'.repeat(20), result: 'in_step' });
   		await settle();
   		expect(tauri.calls('list_removed_devices')).toHaveLength(2);
   		tauri.answer('list_removed_devices', [ACKED]);
   		tauri.emit('verification-result', { org_id: ORG, result: 'in_step' });
   		await settle();
   		expect(tauri.calls('list_removed_devices')).toHaveLength(3);
   		expect(screen.getByText('acknowledged its removal at epoch 4')).toBeTruthy();
   	});
   });
   ```

   (the `verification-result` payloads must match a `backend.json` event
   entry; use the in-step entry the file already holds, with `org_id`
   set). Run `npx vitest run tests/removed.view.test.ts`: red (no
   component).
9. Add `listRemovedDevices(orgId)` and `RemovedDeviceDto` to `api.ts`,
   `removedDeviceLine` to `app/src/lib/removed.ts`, the component, and show
   it with the Revoke panel in `+page.svelte` (as merged: inside `Revoke.svelte`, keyed on the selected Organisation; MR1-A4). Green. Then run every app
   verify command, `npx svelte-check` and clippy.

red -> green: `org-node/tests/receive_path.rs::an_acknowledgement_from_a_device_held_as_revoked_is_kept_once_and_sealed (verifies: LLR-xudkh4, LLR-py43n3)` — compile red: no `acknowledged`.
red -> green: `org-node/tests/receive_path.rs::an_acknowledgement_from_a_device_not_held_as_revoked_is_accepted_and_not_kept (verifies: LLR-xudkh4)` — compile red: no `acknowledged`.
red -> green: `org-node/tests/receive_path.rs::removed_devices_lists_each_device_held_as_revoked_with_its_acknowledgement (verifies: LLR-dsjht5)` — compile red: no `removed_devices`.
red -> green: `org-node/tests/own_removal.rs::an_adoption_keeps_the_acknowledged_record_and_the_field_carries_the_s5_todo (verifies: LLR-py43n3)` — compile red; then re-red against a planted `acknowledged.clear()` in `prune_after_adoption`.
red -> green: `org-node/tests/epoch_store.rs::store_data_holds_its_fourteen_fields_in_order_with_the_notices_senders_and_the_acknowledged_record (verifies: LLR-f5wrjx)` — compile red: no `acknowledged`.
red -> green: `org-io/tests/acknowledgement_inbound.rs::the_admin_lists_a_removed_device_and_then_its_acknowledgement (verifies: LLR-3ydu9e)` — compile red: no `removed_devices`.
red -> green: `app/src-tauri/tests/ipc.rs::product_registers_the_same_commands_as_the_harness (verifies: LLR-ce4hxz)` — red: 17 ≠ 18.
red -> green: `app/src-tauri/tests/ipc.rs::list_removed_devices_refuses_an_unheld_or_malformed_organisation (verifies: LLR-dmfvv6)` — red: unknown command.
red -> green: `app/tests/removed.view.test.ts` ×3 (verifies: LLR-cs9bmw; LLR-cs9bmw; LLR-x9qbhu) — red: no component.
inherited: `receive_path.rs::an_acknowledgement_is_decided_by_check_acknowledgement_with_the_sender_and_no_store_change` — from LLR-5wg8cu; `epoch_store.rs` ×3 — from LLR-d925uu; `ipc.rs` ×3 — from LLR-kk8gc7.

Commits: `feat(org-node): the acknowledged record (SDD-acnpu4)`; `feat(org-io): the removed-Devices query (LLR-3ydu9e)`; `feat(app): the removed Devices view (SDD-g9ugu7)`.

**T16g done 2026-11-04** (merged; 0df9647 org-node, e3006d8 + abe9e96 org-io, 86c866b app). Dispatch report:

```
red -> green: org-node/tests/receive_path.rs::an_acknowledgement_from_a_device_held_as_revoked_is_kept_once_and_sealed (verifies: LLR-xudkh4, LLR-py43n3) — compile red: no `acknowledged`, no `KeptAcknowledgementFrom`
red -> green: org-node/tests/receive_path.rs::an_acknowledgement_from_a_device_not_held_as_revoked_is_accepted_and_not_kept (verifies: LLR-xudkh4) — compile red: no `acknowledged`
red -> green: org-node/tests/receive_path.rs::removed_devices_lists_each_device_held_as_revoked_with_its_acknowledgement (verifies: LLR-dsjht5) — compile red: no `removed_devices`, no `RemovedDeviceView`
red -> green: org-node/tests/own_removal.rs::an_adoption_keeps_the_acknowledged_record_and_the_field_carries_the_s5_todo (verifies: LLR-py43n3) — compile red; re-red against a planted `acknowledged.clear()` (left [] right [entry])
red -> green: org-node/tests/epoch_store.rs::store_data_holds_its_fourteen_fields_in_order_with_the_notices_senders_and_the_acknowledged_record (verifies: LLR-f5wrjx) — compile red: no `acknowledged`
red -> green: org-io/tests/acknowledgement_inbound.rs::the_admin_lists_a_removed_device_and_then_its_acknowledgement (verifies: LLR-3ydu9e) — compile red: no `removed_devices`, `RemovedDevice`, `RemovedDevicesRefused`
red -> green: app/src-tauri/tests/ipc.rs::product_registers_the_same_commands_as_the_harness (verifies: LLR-ce4hxz) — compile red: no `commands::list_removed_devices`
red -> green: app/src-tauri/tests/ipc.rs::list_removed_devices_refuses_an_unheld_or_malformed_organisation (verifies: LLR-dmfvv6) — compile red: no `commands::list_removed_devices`
red -> green: app/src-tauri/tests/ipc.rs::the_removed_devices_answers_have_the_fakes_shapes (verifies: LLR-dmfvv6) — compile red: no `commands::list_removed_devices`
red -> green: app/tests/removed.view.test.ts ×3 (verifies: LLR-cs9bmw; LLR-cs9bmw; LLR-x9qbhu) — red: RemovedDevices.svelte and removed.ts did not exist (import failed)
inherited: receive_path.rs::an_acknowledgement_is_decided_by_check_acknowledgement_with_the_sender_and_no_store_change — from LLR-5wg8cu (to LLR-xudkh4)
inherited: receive_path.rs::an_acknowledgement_is_decided_in_the_check_from_its_named_device_only — from LLR-5wg8cu (to LLR-xudkh4)
inherited: receive_path.rs::a_held_organisations_messages_from_an_unlisted_device_are_refused_before_use — from LLR-5wg8cu (to LLR-xudkh4)
inherited: receive_chain_reads.rs::an_acknowledgement_is_checked_and_reported — from LLR-5wg8cu (to LLR-xudkh4)
inherited: epoch_store.rs::store_data_holds_its_fields_in_encoding_order_and_round_trips_through_the_sealed_store — from LLR-d925uu (to LLR-f5wrjx)
inherited: epoch_store.rs::store_data_holds_no_provisional_updates_field — from LLR-d925uu (to LLR-f5wrjx)
inherited: epoch_store.rs::an_outstanding_invite_keeps_its_invitee_name — from LLR-d925uu (to LLR-f5wrjx)
inherited: ipc.rs::list_personas_returns_an_empty_list_on_a_fresh_store — from LLR-kk8gc7 (to LLR-ce4hxz)
inherited: ipc.rs::the_harness_registers_exactly_the_eighteen_commands_and_refuses_the_old_names — from LLR-kk8gc7 (to LLR-ce4hxz)
inherited: ipc.rs::every_backend_entry_has_a_real_producer_here — from LLR-kk8gc7 (to LLR-ce4hxz)
```

Results: org-node 388/388, quint 5/5, clippy clean; org-io 331 (both lanes), build and clippy clean; app cargo 211/211, check 0/0, vitest 114/114; check-trace: no MISSING-TEST or OUT-OF-FORCE-VERIFIES left in any unit. GOLDEN_ACTIVE_STORE re-derived 2140 → 2355 bytes (one acknowledged entry, spliced by a format-rule script). Adaptations listed in the dispatch report (shared `RawAcknowledgement::parse`; NotHeld reuses org-io's `write_not_held` text; RemovedDevices mounted in Revoke.svelte, keyed on the selected Organisation; a third backend.json entry `one_unacknowledged`). Left for T18: encoding_golden.rs's `verifies:` still cites LLR-d925uu beside its successor; the Revoke-tab page tests call `list_removed_devices` with no fake answer.

### T18 — Tidying: PR-m9betm's pin, and PR-y6ay44 (owner ruling Q4)

**Files touched:** `org-node/src/store.rs`, `org-node/src/revocation.rs`,
`org-node/tests/own_removal.rs`, `org-node/tests/revocation.rs`,
`org-node/docs/problems/2026-10-25-domain-api.md` (*as merged, MR1-C7:* the Q6 sweep also re-annotated tests in org-io, app and on-chain-client; see T18's dispatch report)
**Parallel:** no (after T16g)

The LLR-bgj2ff note (residual, S5 TODO) and PR-m9betm's resolution were
written by T13b; this task adds the code TODO and the tests. PR-7ud5fv is
out of scope (owner ruling Q4 of 2026-11-03): its fix contradicts the merged green
`own_removal.rs::a_store_put_back_over_a_removed_store_replaces_it_with_no_behind_marker`
(`verifies: LLR-zk7fx2`), so it needs LLR-zk7fx2 superseded, a change of its
own.

1. In `store.rs` put `// TODO(S5): the S5 revoked-device CRDT replaces
   revoked_devices (LLR-bgj2ff).` on the `revoked_devices` field.
2. In `own_removal.rs` add:

   ```rust
   // Abnormal: a Persona with no MemberId is refused, nothing signed
   // (PR-m9betm: the skip is gone).
   // verifies: LLR-5tvn7g, PR-m9betm
   #[test]
   fn sign_acknowledgement_refuses_a_persona_with_no_member_id() {
       let store_data = member_data(vec![snapshot(2, "bob", 0x11, &[0x21])]);
       let mut persona = store_data.persona.clone();
       persona.member_id = None;
       let state = chain_listing(&store_data.record.as_ref().unwrap().trie_members, 4, 0x63);
       let refused = org_node::revocation::sign_acknowledgement(&persona, the_org(), &state, &own_secret());
       assert!(matches!(refused, Err(OrgNodeError::OrgNotHeld { org_id }) if org_id == the_org()));
   }
   ```

   Green on arrival (the fix is domain-api's): watch it red against a
   planted `let member_id = persona.member_id.unwrap_or(MemberId::new([0; 32]));`
   in `sign_acknowledgement`, then remove the plant.
3. **PR-y6ay44 (owner ruling Q4: fixed here).** In `org-node/tests/revocation.rs` add:

   ```rust
   // PR-y6ay44: the signed bytes are the domain, then exactly the tuple's
   // postcard encoding, built with no fallible step.
   // verifies: LLR-bs6yxf, PR-y6ay44
   #[test]
   fn signed_bytes_are_the_domain_then_the_tuple_with_no_fallback() {
       let ack = org_node::revocation::Acknowledgement {
           org_id: org(),
           member_id: bob_id(),
           device: device_key(BOB_DEVICE_SEED),
           epoch: Epoch::new(300),
           root: RootHash::new([0x66; 32]),
           signature: org_node::revocation::Signature64([0; 64]),
       };
       let tuple = postcard::to_allocvec(&(ack.org_id, ack.member_id, ack.device, ack.epoch, ack.root)).unwrap();
       assert_eq!(ack.signed_bytes(), [org_node::revocation::ACK_DOMAIN, &tuple[..]].concat());
       let source = include_str!("../src/revocation.rs");
       let body = &source[source.find("pub fn signed_bytes").unwrap()..];
       let body = &body[..body.find("\n    }\n").unwrap()];
       assert!(!body.contains("if let Ok"), "no fallback path in signed_bytes");
   }
   ```

   (epoch 300 makes the varint two bytes.) Red on the scan. Rewrite
   `signed_bytes` to append the fields' fixed-size bytes and the epoch's
   LEB128 varint written by hand, equal to the tuple's postcard encoding
   (the first assertion holds it). Resolve PR-y6ay44 with that test.
4. Run org-node's whole first verify line and clippy.

red -> green: `own_removal.rs::sign_acknowledgement_refuses_a_persona_with_no_member_id (verifies: LLR-5tvn7g, PR-m9betm)` — green on arrival; red against the planted default MemberId.
red -> green: `revocation.rs::signed_bytes_are_the_domain_then_the_tuple_with_no_fallback (verifies: LLR-bs6yxf, PR-y6ay44)` — red: `no fallback path in signed_bytes`.

Commit: `fix(org-node): tidying — PR-m9betm pinned, PR-y6ay44 fixed; revoked_devices' S5 TODO`.

**T18 done 2026-11-04** (merged; 4877623, 41476ac, 0e45f6e). Dispatch report:

```
red -> green: org-node/tests/own_removal.rs::sign_acknowledgement_refuses_a_persona_with_no_member_id (verifies: LLR-5tvn7g, PR-m9betm) — green on arrival (domain-api's fix); watched red on the OrgNotHeld assertion against a planted `persona.member_id.unwrap_or(MemberId::new([0; 32]))`, plant removed
red -> green: org-node/tests/revocation.rs::signed_bytes_are_the_domain_then_the_tuple_with_no_fallback (verifies: LLR-bs6yxf, PR-y6ay44) — red: "no fallback path in signed_bytes"; green after signed_bytes builds ACK_DOMAIN + tuple by hand (push_varint), equal to postcard's tuple encoding at epoch 300
inherited: org-io/tests/absences.rs::the_handle_returns_only_allowlisted_public_types — from LLR-4usfu2 (to LLR-zhdc9f)
inherited: org-node/tests/receive_path.rs::a_kept_notice_counts_against_the_bound — from LLR-r4qtqe (to LLR-gb7p7c)
inherited: org-io/tests/acknowledgements.rs::already_received_or_invite_used_keeps_the_acknowledgement — from LLR-3w22wc (to LLR-u2j6u8)
inherited: org-node/tests/encoding_golden.rs::persona_store_plaintext_is_pinned_in_its_active_form — from LLR-d925uu (to LLR-f5wrjx)
inherited: org-node/tests/value_types.rs::the_first_admission_refusals_name_their_organisation — from LLR-mxskg9 (dropped; successor LLR-3sddey does not state AdmissionNotExpected/AdmissionNotOurs; REQ-8amu2a, REQ-kt877x stay)
inherited: app/src-tauri/tests/ipc.rs::connection_status_is_registered_under_its_name — from LLR-kk8gc7 (to LLR-ce4hxz)
inherited: app/src-tauri/tests/ipc.rs::unknown_command_is_rejected — from LLR-kk8gc7 (to LLR-ce4hxz)
inherited: app/src-tauri/tests/receiver_events.rs::a_chain_failure_emits_no_verification_event_for_any_message — from LLR-7bk6qh (to LLR-wgr46u)
inherited: app/src-tauri/tests/receiver_events.rs::every_real_terminal_message_stops_the_loop — from LLR-a9rjtf (dropped; LLR-zw6jqy cited)
inherited: app/src-tauri/tests/receiver_events.rs::a_non_terminal_failure_does_not_stop_the_loop — from LLR-a9rjtf (dropped; LLR-zw6jqy cited)
inherited: app/src-tauri/tests/receiver_events.rs::the_stop_is_announced_after_the_failure_for_the_same_error — from LLR-a9rjtf (dropped; LLR-zw6jqy cited)
inherited: on-chain-client/tests/decode_org_state.rs::exactly_ninety_six_bytes_decodes_every_field — from LLR-nq7nhg (dropped; LLR-hezpr7 cited)
inherited: on-chain-client/tests/write_pure.rs::proxied_wraps_any_call_unchanged_even_another_proxied_call — from LLR-ywhd23 (dropped; LLR-59kxsr cited)
inherited: on-chain-client/tests/write_pure.rs::proxied_forces_the_any_proxy_type_by_name — from LLR-ywhd23 (dropped; LLR-59kxsr cited)
inherited: on-chain-client/tests/write_pure.rs::proxied_never_leaves_the_proxy_type_unforced_or_forced_by_index — from LLR-ywhd23 (dropped; LLR-59kxsr cited)
inherited: on-chain-client/tests/type_widths.rs::newtypes_have_the_widths_the_abi_gives_them — from REQ-2qa5r5 (dropped; REQ-54txzh cited)
inherited: on-chain-client/tests/signatory_set_decode.rs::definitions_decode_with_type_and_delay_in_stored_order — from LLR-a9bb7b (dropped; LLR-n2vx5q cited)
inherited: on-chain-client/tests/signatory_set_decode.rs::a_malformed_definitions_value_is_refused — from LLR-a9bb7b (dropped; LLR-n2vx5q cited)
```

Results: org-node 390/390, quint 5/5, clippy clean; org-io 320 + 11, build and clippy clean; app cargo 211, check 0/0, vitest 114; on-chain-client 73 + 74. check-trace/check-ids: nothing new in any unit; no verifies: line names a superseded or retired item. PR-y6ay44 resolved; PR-m9betm already resolved (T13b). The Revoke-tab tests in epoch.commands.test.ts answer list_removed_devices with backend.answers.list_removed_devices.none. TODO(S5) placed on store.rs's revoked_devices field. All plan tasks except T17b are done; next: the conformance reviews (develop-change).

### T17b — Close-out

**Files touched:** `docs/verification/<date>-worktree-org-io-publish-2.md` (new)
**Parallel:** no (last)

1. `check-trace.sh` and `check-ids.sh` for org-node, org-io, app and
   on-chain-client: no MISSING-TEST and no OUT-OF-FORCE-VERIFIES left.
2. `verify-before-merge`, then `merge-change`. Coverage floors for org-io:
   lines 80, regions 78, branches 82 (S4's); dropping the uncovered
   `signatory.rs` shell should raise org-io's. Resolves PR-m9betm (T13b,
   pinned in T18) and PR-y6ay44 (fixed in T18, ruling Q4); not PR-7ud5fv.

## 6. Order and parallelism

*Rewritten 2026-11-03.* T13b (done) → T16h → T16c4 → T16g → T18 → T17b, all
serial: T16h and T16g share `org-io/src/lib.rs` and `absences.rs`, T16h and
T16c4 `org-io/src/test_support.rs`, T16c4, T16g and T18 org-node's
`store.rs`, `service.rs` and `own_removal.rs`. T16c4 goes before T16g so the
store-format test (LLR-f5wrjx) is written once, with both new fields.

## 7. The own-admin check's footprint (T16h)

Surveyed 2026-10-16 on master `e2a65d8`. Nothing outside org-io's own tests calls the check; the app neither calls nor imports any of it.

**Code removed:**
- `org-io/src/signatory.rs`, all of it: `AdminStatus`, `OwnAdminError`, `controller_of` (LLR-qhyc3n), `admin_status`, `SignatorySetReader`, `own_admin_status`, `OnChainSignatorySetReader`, `SignatorySetNotConfigured`, and the `ProxyDefinition` / `SignatorySetView` re-exports. `pub mod signatory;` goes from `lib.rs:18` with it.
- In `org-io/src/lib.rs`:
  - `is_own_admin` (`:574-580`) and the test-only `with_own_account` (`:565-572`);
  - the `signatory_reader`, `own_account` and `co_signers` fields (`:89-92`), and the places that set them in `unconfigured`, `connect`, `for_test` and `open_for_test`;
  - the handle's doc comment and the "signatory-set reads" mentions.
- In `org-io/src/test_support.rs`, the `FakeChain` signatory slots: `SignatorySlots`, the slot setters, `signatory_reads`, the failing flags, the share in `sharing_slots_with`, and the `SignatorySetReader` impl.

**Code kept:**
- `connect`'s `co_signatories`, which the writer takes.
- The `CoSignerIsOwnAccount` refusal, which protects the writer.
- `submit::NOT_CONFIGURED`.
- Custody and `ODS_ADMIN_SEED` are untouched.

**Tests:**
- Deleted:
  - `org-io/tests/signatory_rule.rs`, all ten tests (LLR-qhyc3n, LLR-c9fyun, LLR-2hkqv2, LLR-vyd5d3; PR-wk7zwq's reproducing test among them);
  - `signatory_key.rs`'s `a_build_without_dev_seed_says_why_the_own_admin_check_is_not_configured` (LLR-rgdx22 keeps its other tests).
- Edited:
  - `absences.rs`: drop the `"signatory"`, `AdminStatus` and `OwnAdminError` allowlist entries and the T16a note, and add `org_io_offers_no_own_admin_check`.
  - `dev_seed_env.rs`'s `connect_refuses_a_co_signer_equal_to_the_own_account`. Its behaviour stays but it verifies LLR-qhyc3n, which goes. **Re-home it** to LLR-rgdx22's `ConnectFailure` list by a dated note. That changes the annotation of a merged green test, so it needs the owner's acceptance (O4).

**Items deleted:**
- org-io: REQ-ntkjk6 (and the superseded REQ-f3eu9n's chain), SDD-f4khqn, LLR-qhyc3n, LLR-c9fyun, LLR-2hkqv2 and the superseded LLR-vyd5d3.
- Each gets a deletion note naming ruling 6 and is described by name wherever it is cited, as T15d did.

**Items amended by dated notes:**
- SDD-z3ychz (the handle's contents).
- SDD-z85ux9 (the reader shell goes, and `traces:` drops REQ-f3eu9n, which is C3).
- LLR-rgdx22 (its round-3 own-admin clause).
- The risk assessments of REQ-f3eu9n and REQ-ntkjk6 (`org-io/docs/risk/2026-10-08-org-io.md:183`, `2026-10-13-publish.md:146`).
- PR-wk7zwq (resolved). Its reproducing test is deleted with the check; the writer's forced `Any` (LLR-59kxsr, tested in on-chain-client's `write_pure.rs`) is what remains of its fix.
- The app's RC-wzb48r note (`app/docs/risk/2026-10-06-invitation.md:39-47`): the signatory read it mentions is gone, and the control itself is unchanged.
- `docs/adr/2026-10-06-org-io-unit.md`'s own-admin bullet.
- Prose in `org-io/docs/architecture/README.md`, `org-io/AGENTS.md`, `org-io/Cargo.toml` and `org-io/tests/common/handles.rs`.

**Configuration:** drop `--test signatory_rule` from:
- `org-io/.guardrails/config.yaml` verify line 1;
- `Makefile`'s `ORG_IO_COVERAGE_ARGS`, with its shortfall rationale for `signatory.rs`;
- `.github/workflows/rust.yml:101`;
- `org-io/Cargo.toml`'s `[[test]]`.

Removing the uncovered reader shell should raise org-io's coverage.

**on-chain-client (O2):** after the drop, org-io's reader was the only consumer of the whole signatory-set read. That covers `OrgRegistryClient::signatory_set`, `PinnedBlock`, `fetch_storage_raw_at`, and all of `write/signatory_set.rs` including `any_proxy_type_index`. The chain write needs none of it, because `proxied()` forces `Any` by variant name. Under O2 (a) the read stays with its exported items (REQ-b8x2py, REQ-8p2veg, SDD-rxfu6h, LLR-sc79gq, LLR-n2vx5q, LLR-m3tjvp; resolved PR-ksm2ua) and its tests, each given a dated note "no consumer since 2026-10-16 (ruling 6); kept for the several-Admins stage". Under (b) they are deleted in the same way, and `ON_CHAIN_CLIENT_COVERAGE_ARGS` loses `signatory_set_decode` and `fuzz_signatory_set`.

- **O4 — re-annotating a merged green test.** `dev_seed_env.rs`'s co-signer refusal test keeps its behaviour, but its item (LLR-qhyc3n) goes with the check. **Recommended:** move its `verifies:` to LLR-rgdx22, whose `ConnectFailure` list already covers the refusal, with a dated note on LLR-rgdx22. The alternative is a new LLR under the writer's connect.

**Additions (T13b, 2026-11-03, against `7ac0825`).** Beyond the 2026-10-16
survey: LLR-7qdjqn (superseding LLR-c9fyun) and LLR-bwerd2 (the
`SignatorySetReader` seam) are retired with the rest; `absences.rs` lists
`is_own_admin` in `the_handles_production_methods_are_exactly_the_domain_surface`
(LLR-4usfu2, superseded by LLR-zhdc9f) and `"signatory"`, `AdminStatus`,
`OwnAdminError` in `HANDLE_RETURN_ALLOWLIST` (LLR-gh4rw3's note); the chain
seams' contract (`tests/common/chain_contract.rs`'s `signatory_set_contract`,
called from `chain_write.rs` ≈597 and `chain_writer_contract.rs` ≈140, both
`verifies: LLR-js7wgk, LLR-j4nkx5, LLR-bwerd2`) loses its signatory part;
`rust.yml`'s line is ≈104, not 101; `connect.rs:28` cites LLR-qhyc3n. SDD-z85ux9's
`traces:` held REQ-ntkjk6 (domain-api's re-trace of REQ-f3eu9n) and drops it.
Done in T13b: every ledger item and prose file above; left to T16h: the
code, tests and configuration.

## 8. Draft checks

*2026-11-03 (T13b, on `worktree-org-io-publish-2-t13b`).* Docs only. Each
unit's `check-ids.sh` exits 0 and `check-units.sh` exits 0. `check-trace.sh`
exits 1 for org-node, org-io and app, as T13b intends, with two kinds of
finding only, each cleared by a named task:

- `MISSING-TEST` on T13b's new items in force: org-node LLR-6fsnma,
  LLR-dsjht5, LLR-f5wrjx, LLR-gb7p7c, LLR-py43n3, LLR-xudkh4 and REQ-32cp7f,
  REQ-f8pu8a, REQ-h7y5ur, REQ-6tepxq (its one LLR, LLR-vq5d5n, is superseded
  by the untested LLR-6fsnma); org-io LLR-3ydu9e, LLR-u2j6u8, LLR-zhdc9f and
  REQ-xghnh3, REQ-kn8zg6; app LLR-ce4hxz, LLR-cs9bmw, LLR-dmfvv6, LLR-x9qbhu
  and REQ-yvzef9. Cleared by T16h, T16c4 and T16g.
- `OUT-OF-FORCE-VERIFIES` on the merged tests of the items T13b superseded or
  retired: org-node `epoch_store.rs` ×3, `own_removal.rs` ×2,
  `receive_path.rs:764`, `:839`; org-io `acknowledgements.rs:255`, `:481`,
  `dev_seed_env.rs:34`, `signatory_rule.rs` ×10 and `absences.rs`'s surface
  test; app `ipc.rs` ×4. Cleared by re-pointing (Q6) or deleting them in
  T16h, T16c4 and T16g.

on-chain-client's `check-trace.sh` exits 0. No `verify_commands` were run:
no code changed.

## Conformance round 1 (2026-11-04, head after T18)

Seven reviewers (review-conformance), one guidelines file each, split by unit. Full reports in the session scratchpad, p2r1-*.md. Findings and dispositions:

**TEST org-node** (docs/TEST_GUIDELINES.md) — 1 medium, 1 low.
- T1-N1 (medium, Seams 2): own_removal.rs:699-705 `sign_acknowledgement_refuses_a_persona_with_no_member_id` calls `revocation::sign_acknowledgement` directly; no LLR states that refusal; its `verifies: LLR-5tvn7g` contract (store unchanged) is unchecked. → fix (n1): test through `verify` with a Persona with no MemberId, assert the refusal and an unchanged store.
- T1-N2 (low, Seams 1): revocation.rs:176-179 source-scan for `if let Ok` is brittle and incomplete. → fix (n1): drop the scan; the equality assertion is the seam test (add boundary epochs 127, 128, u64::MAX).

**TEST org-io + on-chain-client** — 1 medium.
- T1-O1 (medium, Seams 1/2): acknowledgements.rs:502-524 tests crate-private `dial_plan` through the one-caller wrapper `test_support::acknowledgement_dial_plan`; stays green if `try_addresses` stops using it; LLR-u2j6u8's Networked rule and REQ-kn8zg6 have no test observing a delivery. → fix (o1): a handle-level Networked test (no kept address for the Admin, relay + lookup, round reports Accepted), then delete the plan test and the wrapper.

**TEST app** — 1 medium, 1 low.
- T1-A1 (medium, UI 4): removed.view.test.ts — loading state untested (`loaded &&` removable); rows → refusal → rows on Refresh untested. → fix (a1), plus the notes: assert the replaced row is gone (LLR-x9qbhu "each answer replaces"), another event reads nothing, a DOM test of Revoke tab → panel with the `{#key}` re-read on Organisation switch.
- T1-A2 (low, UI 1 vs Seams 2.4): the clause itself, not the diff. → recorded, not fixed here: a guideline tailoring is the owner's (as 2026-10-28's).

**CODE org-node** (.guardrails/templates/CODE_GUIDELINES.md) — 3 medium, 3 low.
- C1-N1 (medium, Comments 2): encoding_golden.rs:71, :75 "added 2026-11-03, plan task T16c4/T16g … 2108 → 2140 → 2355". → fix (n1).
- C1-N2 (medium, Comments 2): own_removal.rs:696 "(PR-m9betm: the skip is gone)". → fix (n1).
- C1-N3 (low, Comments 2): own_removal.rs:634 assertion message "the old order". → fix (n1).
- C1-N4 (medium, Names 4): `removed_devices`/`RemovedDeviceView` vs codebase `revoked_devices` "held as revoked". → dispatcher ruling: the glossary's user-facing term is "removed Device" (docs/CONTEXT.md, Acknowledged record: "What the app shows the Admin of removed Devices"); names stay (LLR-dsjht5, app `list_removed_devices`); fix (n1): define **Removed Device** in docs/CONTEXT.md (with A1-1) relating it to "held as revoked" and `revoked_devices`, and make the view's doc comment use it.
- C1-N5 (low, Names 2): service.rs:404-409 `.filter(|_| …)` standing in for an `if`. → fix (n1).
- C1-N6 (low, Names 3): own_removal.rs:629 `two`. → fix (n1).

**CODE org-io + on-chain-client** — 6 low (loop done for this review).
- C1-O1: org-io/.guardrails/config.yaml:65 "(was 30)". → fix (o1).
- C1-O2: test_support.rs:4 "former `FakeWriter`". → fix (o1).
- C1-O3: test_support.rs:391-399 one-caller wrapper. → goes with T1-O1 (o1).
- C1-O4: removed.rs `RemovedDevicesRefused::Node` unreached, `Error::source` unasked. → fix (o1): test the arm if reachable, else a comment why not; drop `source` if no requirement needs it.
- C1-O5: removed.rs:50-55 `views`/`view`. → fix (o1).
- C1-O6: acknowledgements.rs:512, :516 `two`. → fix (o1).

**CODE app** — 2 medium, 4 low.
- C1-A1 (medium, Names 1/4): RemovedDevices.svelte:25 `catch (e)` → `refusal`. → fix (a1).
- C1-A2 (medium, Names 3/4): RemovedDevices.svelte:42, removed.ts:14 `result` → `entry`. → fix (a1).
- C1-A3 (low): RemovedDevices.svelte:46 `fn`. → fix (a1).
- C1-A4 (low, Names 4): test `SILENT` vs fixture `one_unacknowledged`. → fix (a1).
- C1-A5 (low, Simplicity 4): test constants repeat backend.json. → fix (a1): read the fixture.
- C1-A6 (low, Errors 2): ipc.rs:1447 `is_ok_and` swallows a refusal into a 30 s timeout. → fix (a1); also the ipc.rs:1383 "Normal" and :1402 "two shapes" comments.

**ARCHITECTURE (all paths)** — 1 medium, 3 low.
- A1-1 (medium, Domain 2): "removed Device" public in three units and the UI, not in docs/CONTEXT.md. → fix (n1), with C1-N4.
- A1-2 (low, Domain 1): store.rs:993 `KeptAcknowledgementFrom.ack` uses an _Avoid_ synonym. → fix (n1): rename the new field (the existing `KeptAcknowledgement.ack` is not this change's).
- A1-3 (low, Boundaries 3): SDD-acnpu4 "kept … and listed …"; its code claim overlaps SDD-busuj3. → fix (n1): one-sentence purpose, code claim without the overlap (dated note).
- A1-4 (low, Boundaries 3): SDD-g9ugu7 "what … shows … and when it asks". → fix (a1): one-sentence purpose (dated note).

Fixers: n1 (org-node + docs/CONTEXT.md), o1 (org-io), a1 (app), in parallel (disjoint files). Then round 2 for the reviews with findings above low: TEST org-node, TEST org-io, TEST app, CODE org-node, CODE app, ARCH. CODE org-io had nothing above low: done.

**a1 merged 2026-11-04** (f87551a, 7de8940, 6ec96cd): T1-A1, C1-A1–A6, A1-4 fixed; T1-A2 left for the owner. app cargo 211, check 0/0, vitest 117; check-trace/check-ids nothing new.

```
red -> green: app/tests/removed.view.test.ts::while the first answer is in flight it says nothing of the list, then shows it (verifies: LLR-cs9bmw) — green on arrival; red against a mutant with `loaded &&` removed
red -> green: app/tests/removed.view.test.ts::a refused Refresh replaces the rows with its text, and the next answer replaces the text with rows (verifies: LLR-cs9bmw, LLR-x9qbhu) — green on arrival; red against a mutant keeping the refusal text after a successful load
red -> green: app/tests/removed.view.test.ts::shows the selected Organisation's removed Devices, and reads them again for another selected (verifies: LLR-x9qbhu) — green on arrival; red with the panel removed from Revoke.svelte
red -> green: app/tests/removed.view.test.ts::reads the list when shown, on Refresh, and after a verification result for its Organisation only (edited, verifies: LLR-x9qbhu) — new asserts red against an admission-requested subscription that loads, and against a merging (non-replacing) load
```

Noted by a1: Revoke.svelte's `{#key selectedOrgId}` is redundant (the panel's `$effect` re-reads on an `orgId` change); left in place, no user-visible test can tell it apart. State variable renamed `refusalText` to avoid shadowing.

**n1 merged 2026-11-04** (1cb686f..62a0c8b): T1-N1, T1-N2, C1-N1–N6, A1-1–A1-3 fixed. org-node 390/390; check-trace/check-ids nothing new. New **Removed Device** glossary entry in docs/CONTEXT.md; `KeptAcknowledgementFrom.ack` renamed `acknowledgement` (encoding positional, golden bytes unchanged; LLR-py43n3, LLR-xudkh4, LLR-dsjht5 dated notes). The T18 tests named above were replaced:

```
red -> green: org-node/tests/own_removal.rs::a_removal_of_a_persona_with_no_member_id_is_refused_with_nothing_deleted (verifies: LLR-5tvn7g, PR-m9betm) — green on arrival; red against a planted `persona.member_id.unwrap_or(MemberId::new([0; 32]))` (got Ok(Removed { acknowledgements_kept: 1 })); replaces sign_acknowledgement_refuses_a_persona_with_no_member_id
red -> green: org-node/tests/revocation.rs::signed_bytes_are_the_domain_then_the_tuple_at_each_varint_length (verifies: LLR-bs6yxf, PR-y6ay44) — green on arrival; red at "epoch 128" against a planted `while value > 0x80`; replaces signed_bytes_are_the_domain_then_the_tuple_with_no_fallback
```

Open for round 2's ARCH review: SDD-busuj3's 2026-11-03 traces note says LLR-xudkh4 is "under SDD-acnpu4", and LLR-xudkh4 sits in SDD-acnpu4's section, while SDD-acnpu4's amended code claim now names the append as SDD-busuj3's arm. The gate's quint runs are to be confirmed individually (n1 captured only forkSafety's exit alone).

**o1 merged 2026-11-04** (17212e5, b6b21d0, 363696d): C1-O1–O6 fixed; `test_support::acknowledgement_dial_plan` and the plan test deleted; `RemovedDevicesRefused::Node` kept with a comment why it is unreachable, `Error::source` dropped. org-io 331, check-trace/check-ids nothing new. T1-O1 partly: the handle-level test pins one dial by key reported as `Transport(_)`, not `NoAddress`:

```
red -> green: org-io/tests/acknowledgements.rs::networked_a_recipient_with_no_kept_address_is_dialled_by_its_key (verifies: LLR-u2j6u8) — green on arrival; red (NotDelivered { reason: NoAddress }) with the Networked branch of dial_plan disabled
```

**Finding by o1 (design conflict):** a dial by key alone can never succeed: LLR-cqr26z binds the delivery endpoint `BindPlan::Direct` (no relay, no address lookup), so iroh fails with `Connect("No addressing information available")` even with the recipient online and in the lookup. Q1 (c) was unreachable as designed.

**Owner ruling Q1' (2026-11-04):** in Networked mode only, a recipient with no usable address is dialled once from an endpoint bound with the iroh relay server and address lookup (D7 allows the relay server; only a forwarding Device is barred). LLR-cqr26z is superseded for that case only; every dial that has an address keeps the direct-only bind. Task **q1r** implements it (supersede LLR-cqr26z, amend REQ-kn8zg6/LLR-u2j6u8, assess the relay's view in risk; the handle test asserts `Accepted`; re-cover "none that parses: one dial by key" at the handle).

**q1r merged 2026-11-04** (67097c6): LLR-spckm6 minted, supersedes LLR-cqr26z (adds the one relay + lookup bind for a no-address dial in Networked mode); REQ-kn8zg6, LLR-u2j6u8, SDD-3my66m, LLR-4k43dm dated notes. Code: `Context::relay_plan` from `Endpoints::plan()`; `acknowledge::dial_by_key` binds from the keychain with the relay plan, one send, closes; a failed bind is that recipient's `NotDelivered { SenderEndpoint(..) }`. org-io 322 + 11, build and clippy clean; check-trace only LLR 161 → 162.

```
red -> green: org-io/tests/acknowledgements.rs::networked_a_recipient_with_no_kept_address_is_dialled_by_its_key (verifies: LLR-spckm6, LLR-u2j6u8) — red against the direct-only bind (NotDelivered { Transport(Connect("No addressing information available")) }); green: the Admin's receive step reports Acknowledged
red -> green: org-io/tests/acknowledgements.rs::networked_a_recipient_whose_only_kept_address_does_not_parse_is_dialled_by_its_key (verifies: LLR-spckm6, LLR-u2j6u8) — red the same way before the relay bind
red -> green: org-io/tests/acknowledgements.rs::networked_a_failed_relay_bind_is_the_recipients_outcome_and_the_next_round_binds_again (verifies: LLR-spckm6) — written after the code; red with src reverted to HEAD (timed out waiting for SenderEndpoint), green restored
inherited: org-io/tests/acknowledgements.rs::acknowledgements_go_from_the_removed_personas_own_identity_over_a_direct_bind — from LLR-cqr26z (to LLR-spckm6)
inherited: org-io/tests/acknowledgements.rs::a_failed_direct_bind_is_every_recipients_outcome_and_the_next_round_binds_again — from LLR-cqr26z (to LLR-spckm6)
```

Risk (org-io risk file, note under REQ-kn8zg6, no rating changed): for each dial by key, the relay/lookup operator (n0 under presets::N0) sees the removed Device's key and IP, the removed-Device → recipient key pair, and its recurrence on REQ-jqmpz5's schedule; no Organisation data or acknowledgement content (end-to-end QUIC). No existing hazard rates what the relay/lookup operator observes.

## Conformance round 2 (2026-11-04, head after q1r) — the last round

Seven reviewers again (all reruns, since q1r changed org-io code). Reports in the session scratchpad, p2r2-*.md. **No finding above low in any review: the conformance loop is complete.** Recurrence check against round 1: C2-N1 repeats C1-N4 (same names, same place) but at low, so it does not count. Lows are fixed by one fixer per unit (n2, o2, a2, disjoint files); no further reviewer.

**TEST org-node** — 1 low. T2-N1 (Existing suites 1/Seams 1): epoch_store.rs:194-197 `store_data_holds_no_provisional_updates_field` re-pointed in place but stays a source scan. → n2: move to the encoding seam. Notes taken up by n2: receive_path.rs:953-957 pin `OrgNotHeld`; add LLR-dsjht5's removed-store `OrganisationRemoved` case and LLR-xudkh4's failed-seal case where reachable. Recorded, not fixed: the PR-y6ay44 test cannot fail if the fallback returned, because the guarded serialisation cannot fail for these types; the fix removed the path, and the test pins the bytes (LLR-bs6yxf).
**TEST org-io + occ** — 1 low. T2-O1 (Seams 1): absences.rs:680-686 scans names no live LLR states. → o2: narrow to LLR-zhdc9f. Notes taken up by o2: LLR-spckm6's "other recipients dialled as before after a failed relay bind" and "relay endpoint closed once the outcome is read"; LLR-u2j6u8's "all kept addresses fail → no dial by key"; acknowledgements.rs:528-533's unasserted "direct endpoint stays bound"; on-chain-client/docs/architecture/2026-10-13-publish.md:64, :92 stale "new tests carry both IDs".
**TEST app** — none. Notes taken up by a2: a two-Device answer to check org-io's order (LLR-dmfvv6); LLR-cs9bmw's wording rule on the empty list and a refusal; ipc.rs:1283 and receiver_events.rs:971 stale LLR-kk8gc7/LLR-7bk6qh comments.
**CODE org-node** — 4 low. C2-N1 two names (`removed_devices` vs `revoked_devices`) → no change: dispatcher ruling of round 1, the glossary's Removed Device entry ties them. C2-N2 "sender" for the Delivering Device (service.rs:1011, own_removal.rs:642) → n2. C2-N3 `value`, `entry` → n2. C2-N4 removal.rs:196-197 one statement doing two things, `delivering` rebound → n2.
**CODE org-io + occ** — 2 low. C2-O1 `relay_plan` / `plan()` / "Networked plan" for the handle's bind plan → o2: one name, `bind_plan`. C2-O2 receive_step.rs:484 store tag "receive-ack-no-write" contradicts its test → o2.
**CODE app** — 4 low. C2-A1 `removed` for one Device and the list → a2. C2-A2 one-caller `rereadsAfter` → a2: inline. C2-A3 RemovedDevices.svelte:46 `subscribeAll(...).then` with no rejection handler → a2: show the refusal. C2-A4 ipc.rs:1402 comment retells the test → a2.
**ARCHITECTURE** — 3 low. A2-1 SDD-g9ugu7 text still has "and" → a2: amend in place to the note's sentence. A2-2 (Domain 3) "kept acknowledgement" means the outgoing one; the incoming entry is `KeptAcknowledgementFrom` → n2: rename to `AcknowledgedEntry` (org-node code, tests, LLR text with dated notes), o2: org-io/src/removed.rs prose and LLR-3ydu9e's wording to "an entry of the Acknowledged record". A2-3 two more `hex::encode` sites in the app → no change: the existing pattern (28 sites), a pure codec; Existing code 1.

**a2 merged 2026-11-04** (8 commits): TEST app notes, C2-A1–A4, A2-1 fixed. app cargo 212, check 0/0, vitest 118; checkers nothing new. SDD-g9ugu7 amended in place (purpose + code claim, since `rereadsAfter` was inlined into the component). The earlier test 'says so when no Device is removed, and shows a refusal as its text' was replaced by a superset test.

```
red -> green: app/src-tauri/tests/ipc.rs::list_removed_devices_answers_two_devices_in_org_ios_order (verifies: LLR-dmfvv6) — green on arrival; red against a planted `.rev()` in list_removed_devices
red -> green: app/tests/removed.view.test.ts::says so when no Device is removed, shows a refusal as its text, and neither shows a deletion word (verifies: LLR-cs9bmw) — green on arrival; red against "wiped" planted in the empty branch and "erased" in the refusal branch
red -> green: app/tests/removed.view.test.ts::a refused subscription is shown on the panel, and the rows still are (verifies: LLR-x9qbhu) — red before the C2-A3 fix (no refusal text, unhandled rejection)
```

**n2 merged 2026-11-04** (4a5575d..bc4fe8a): T2-N1 + notes, C2-N2–N4, A2-2 (org-node) fixed; `KeptAcknowledgementFrom` → `AcknowledgedEntry` (SDD-acnpu4, LLR-xudkh4, LLR-py43n3, LLR-f5wrjx amended in place; golden bytes unchanged). LLR-xudkh4's failed-seal case not fixed: unreachable (postcard cannot fail for these types; XChaCha20-Poly1305 only past its length limit). org-node 391, quint 5/5 (each run alone, exit 0), clippy clean; `cargo check --workspace --all-targets --all-features` clean.

```
red -> green: org-node/tests/own_removal.rs::the_removed_devices_of_a_removed_store_are_organisation_removed (verifies: LLR-dsjht5) — green on arrival; red against removed_devices returning Ok(vec![]) for a removed store
red -> green: org-node/tests/epoch_store.rs::store_data_holds_no_provisional_updates_field (rewritten at the encoding seam, verifies: LLR-f5wrjx) — red against a planted provisional_updates field in StoreDataEncoding
red -> green: org-node/tests/receive_path.rs::removed_devices_lists_each_device_held_as_revoked_with_its_acknowledgement (OrgNotHeld pinned, verifies: LLR-dsjht5) — red against removed_devices returning FoundingNotWritten
```

Renamed only: own_removal.rs::the_sender_of_the_accepted_notice_is_the_first_recipient → the_delivering_device_of_the_accepted_notice_is_the_first_recipient (verifies unchanged). Left by n2 for o2's wording: "the acknowledgement kept from it" in receive_path.rs:922 and service.rs:1227, :1617 doc comments.

**o2 merged 2026-11-04** (1494243..c6d4460): T2-O1 + notes, C2-O1 (`bind_plan`, LLR-spckm6 note), C2-O2, A2-2 (org-io: removed.rs prose, LLR-3ydu9e note), occ 2026-10-13-publish.md :64/:92 fixed. org-io 324 (2 ignored) + 11, build and clippy clean; checkers nothing new.

```
red -> green: org-io/tests/absences.rs::the_handle_offers_none_of_the_removed_methods (verifies: LLR-zhdc9f) — green on arrival; red against a planted `pub fn is_own_admin` on OrgIo; replaces org_io_offers_no_own_admin_check
red -> green: org-io/tests/acknowledgements.rs::networked_a_recipient_whose_kept_addresses_all_fail_is_not_dialled_by_its_key (verifies: LLR-u2j6u8) — green on arrival; red against a dial_plan mutant that always appends the dial by key
red -> green: org-io/tests/acknowledgements.rs::networked_after_a_failed_relay_bind_the_other_recipients_are_dialled_as_before (verifies: LLR-spckm6) — green on arrival; red (left 0, right 1) against a mutant ending the round after a relay-bind failure
```

Not fixed, recorded: LLR-spckm6's "closed once the dial's outcome is read" is unobservable at any seam (iroh 0.98 aborts an unclosed endpoint on drop; same report, same receive step) — dated note on LLR-spckm6 by the dispatcher. Dispatcher also reworded the remaining "the acknowledgement kept from it" comments (service.rs:1227, receive_path.rs:922; service.rs:1617 had none left) to "its entry of the Acknowledged record".

**Conformance loop complete 2026-11-04.** Next: check-traceability, then merge-change.

**Owner ruling 2026-11-09:** app PR-eecx3y (still applies under today's names; task stl did not resolve it) accepted like the other four app reports. check-traceability done 2026-11-08: check-ids, check-trace and check-units exit 0 for all four units.

**Owner rulings of 2026-11-08** (task stl, docs only):

1. *Resolve as obsolete.* org-node PR-w88sr9 (`WireMessage::genesis_snapshot`)
   is `resolved`: change `worktree-org-node-org-key-pair` (1f52c36) removed the
   field, and `WireMessage` is now an enum with no optional field whose
   revocation kind carries no snapshot. app PR-eecx3y is **not** resolved and
   stays `open`: `chain_ready` is gone, but the defect lives on under
   `AppState::chain_endpoint` and `ConnectionStatus::chain_configured`, still
   decided once in `AppState::init` and never re-evaluated
   (`app/src-tauri/src/policy.rs` cites PR-eecx3y on that field). The report
   already records the rename (its correction of 2026-09-14). Referred back
   to the owner.
2. *Accept.* app PR-5mc4d8, PR-h6xpnh, PR-j9f6kk and PR-w5dae4 are `accepted`,
   with a disposition of 2026-11-08: to be fixed in a later app change.
3. *Relay note, no new hazard.* The relay and lookup operator's view of a dial
   by key stays a risk note under REQ-kn8zg6
   (`org-io/docs/risk/2026-11-02-publish-2.md`, a dated line after the note of
   2026-11-04), to be revisited when self-hosted relays are planned.

## Merge review — round 1, the last round (merge-change step 4, 2026-11-09, commit b97eff4)

Four fresh reviewers, one per scope, with the review checklist; reports in the session scratchpad, pr2-mr1-{org-node,org-io,app,occ}.md. **No `code` or `requirement` finding above low in any scope: the last review round.** Mechanical lows are fixed by fixers m-n, m-o, m-a, m-c (disjoint files); the others are opened as problem reports or recorded. Base unchanged (local master 7ac0825), no `shared:` line.

**org-node** — tests 177/177; checkers exit 0. Verdict: fit to merge once the lows are dispositioned.
- MR1-N1 (code low): `removed_devices` lists every `revoked_devices` entry without checking that the current record no longer lists the Device; nothing refuses re-admitting a revoked Device key (dispatcher checked: no read of `revoked_devices` on admission). → **opened as a problem report** by m-n (org-node), not fixed here: needs a rule (refuse re-admission, or prune on re-admission).
- MR1-N2 (code low): no test of an invalid `KeptNotice.from` refusal (`org.kept_revocation_notices`). → m-n.
- MR1-N3 (code low): acknowledgement tests use the same MemberId as the held RevokedDevice; TODO scan misses `AcknowledgedEntry`. → m-n.
- MR1-N4 (requirement low): LLR-f5wrjx drops LLR-d925uu's "no `ProvisionalUpdate` type exists" clause silently. → m-n: dated note.
- MR1-N5 (requirement low): LLR-xudkh4 and LLR-f5wrjx carry rationale/history in item text. → m-n.
- MR1-N6 (requirement low): LLR-xudkh4's owning SDD stated two ways. → m-n: it is SDD-busuj3's (the code is `check_received`); notes on SDD-acnpu4 and SDD-busuj3.
- MR1-N7 (requirement low): the skill's review checklist forbids supersedes:/superseded-by:, `status: resolved` and PR IDs in verifies:, which this repo's guardrails (2bc3cca) require. → recorded: a skill/guardrails-version mismatch for the owner; the diff follows the repo.
- MR1-N8 (record): the plan header's Resolves omits PR-w88sr9. → dispatcher (header).

**org-io** — tests 152/152 (acknowledgements 45/45); checkers exit 0. Verdict: can merge.
- MR1-O1 (code low): acknowledgement_inbound.rs:302-330 matches `Some(_)` and `..`; never checks "reads no chain, writes nothing". → m-o.
- MR1-O2 (requirement low): REQ-k6kcgd's 2026-11-03 note lets the relay carry every send; only REQ-kn8zg6's dial by key is relayed. → m-o: dated note.
- MR1-O3 (requirement low): LLR-spckm6's untestable "closed once the outcome is read" clause. → m-o: dated note moving it to design prose.
- MR1-O4 (requirement low): SDD-3my66m's note cites `Endpoints::plan`. → m-o.
- MR1-O5 (requirement low): LLR-7qdjqn's deletion note names the replaced test. → m-o.
- MR1-O6 (record): Progress row and header "plan draft only" stale. → dispatcher.
- MR1-O7 (requirement low): the checklist's deletion rule vs this repo's supersession rule. → recorded with MR1-N7.

**app** — ipc 60, receiver_events 50, vitest 26; checkers exit 0. Verdict: mergeable after this round.
- MR1-A1 (requirement low): the refused-subscription label is in no item. → m-a: add it to LLR-x9qbhu (dated note).
- MR1-A2 (requirement low): SDD-6pjvhc / app glossary still say seventeen commands and cite LLR-kk8gc7. → m-a.
- MR1-A3 (code low): overlapping loads can land out of order, a stale answer replacing a newer one (HAZ-2hqt34's second misreading until the next read). → m-a: request counter, test with two deferred answers out of order.
- MR1-A4 (record): T16g's prose says `+page.svelte`; mounted in `Revoke.svelte`. → dispatcher.

**on-chain-client + root** — decode_org_state 9, type_widths 2, write_pure 25, signatory_set_decode 10, fuzz_signatory_set green; checkers exit 0. No code findings.
- MR1-C1 (requirement low): three notes still say tests carry superseded IDs (2026-09-28-decomposition.md:780-782, risk 2026-10-05-organisation-public-key.md:7-8, 2026-10-06-chain-write.md:224, :229-231). → m-c.
- MR1-C2 (requirement low): occ architecture README:90 says org-io calls the signatory-set reads → m-c. `.guardrails/units.yaml:63` names "the own-admin check" → **recorded, not fixed here**: a root .guardrails edit maps the change to every unit's impact set; left for the next guardrails change.
- MR1-C3 (requirement low): Makefile:326 still names `signatory.rs`. → m-c; the 6a gate measures org-io's coverage afresh.
- MR1-C4 (requirement low): ledgers cite the plan file (ADR 2026-10-06-org-io-unit.md:231-232, occ 2026-10-08-signatory-reads.md:39, 2026-10-13-publish.md:56). → m-c: cite the change and ruling 6.
- MR1-C5 (requirement low): o2's history-only notes under LLR-59kxsr and LLR-n2vx5q. → m-c: delete.
- MR1-C6, MR1-C7 (record): plan header "on-chain-client only if O2 removes its read"; T18's Files touched. → dispatcher.
- MR1-C8 (requirement low, predates): rust.yml runs only occ's first verify line. → already open as PR-hj4wwd; no new report.
- MR1-C9 (requirement low, predates, org-members): Makefile omits `tree_error_mapping` from ORG_MEMBERS_COVERAGE_ARGS. → recorded, not opened here: org-members is outside the impact set; for the owner.

**Fixers merged 2026-11-09** (no further reviewer: the round had nothing above low).

- **m-c** (6cce7d0): MR1-C1–C5 fixed (dated notes; Makefile's `signatory.rs` line removed; ledgers cite the change and ruling 6; o2's history-only notes deleted). Checkers nothing new; check-units 0; `make -n coverage-org-io` parses.
- **m-a** (c119e9b, 65f538f, 251e0ae): MR1-A1 (LLR-x9qbhu dated note: the refused-subscription label), MR1-A2 (SDD-6pjvhc and the Command glossary entry: eighteen commands, fourteen domain calls, LLR-ce4hxz), MR1-A3 (LLR-x9qbhu dated note: only the latest read's answer replaces the rows; `latestRead` counter). app cargo all ok (ipc 60, receiver_events 50), check 0/0, vitest 120.
- **m-n** (4d69866, 56fdbe2, 816d5f1, 2ec754a): MR1-N1 opened as **PR-gg4zwz** (org-node/docs/problems/2026-11-09-publish-2.md); MR1-N2–N6 fixed (LLR-d925uu note on the dropped clause; LLR-xudkh4 and LLR-f5wrjx amended in place; LLR-xudkh4 belongs to SDD-busuj3, notes on SDD-acnpu4 and SDD-busuj3). org-node green, clippy clean, quint 5/5 each alone.
- **m-o** (88389b8, 9d0ed69, f38f3ab): MR1-O1 test strengthened; MR1-O2 (REQ-k6kcgd note narrowed to the dial by key), MR1-O3 (LLR-spckm6's close clause moved to design prose), MR1-O4, MR1-O5 fixed. org-io 324 + 11, build and clippy clean.

```
red -> green: app/tests/removed.view.test.ts::an earlier read answered after a later one is dropped, so the latest answer stays shown (verifies: LLR-x9qbhu) — red before the counter (the stale answer showed "no acknowledgement received by this Device")
red -> green: app/tests/removed.view.test.ts::an earlier read refused after a later one was answered leaves the latest rows shown (verifies: LLR-x9qbhu) — red before the counter (the stale refusal replaced the rows)
red -> green: org-node/tests/persona_records.rs::unseal_refuses_an_invalid_field_of_each_epoch_state_kind_naming_it (new case, verifies: LLR-7yu2xy, LLR-f5wrjx) — red against the KeptNotice parse naming "org.revoked_devices"
red -> green: org-node/tests/receive_path.rs::an_acknowledgement_from_a_device_held_as_revoked_is_kept_once_and_sealed (strengthened, verifies: LLR-xudkh4, LLR-py43n3) — red against `member_id: ack.member_id`
red -> green: org-node/tests/own_removal.rs::an_adoption_keeps_the_acknowledged_record_and_the_field_carries_the_s5_todo (strengthened, verifies: LLR-py43n3) — red with the TODO(S5) above `AcknowledgedEntry` removed
red -> green: org-io/tests/acknowledgement_inbound.rs::the_admin_lists_a_removed_device_and_then_its_acknowledgement (strengthened, verifies: LLR-3ydu9e) — red against four mutants: a fixed Epoch(1), a wrong MemberId, a chain read in the query, a store write in the query
```

Next: rerun from step 1 (no reviewer), then step 5.

## Progress

| Step | Status | Commit |
|---|---|---|
| Plan drafted on `worktree-org-io-publish-2`, docs only, before domain-api merges; part 1's §10 marked superseded; the own-admin footprint surveyed (§7) | done | this commit |
| Re-read domain-api at `06a3951` (its clarifications of 2026-10-16): C1–C3 refined, C4 added (addresses for the acknowledgement) | done | this commit |
| C1–C4 answered with domain-api; O1–O4 ruled by the owner | superseded by §0 (R1–R4) | |
| domain-api merged; master merged here | done | c1bf729 |
| Rescoped against the merged tree; owner rulings R1–R4 (§0) | done | 45c1ca1 |
| T13b: requirements, risk and design for T16g, T16c4, T16h, T18; §5 rewritten; owner questions Q1–Q6 (§4) | done, unmerged on `worktree-org-io-publish-2-t13b` | see branch |
| Owner rulings on Q1–Q6: every recommendation accepted (§4); markers replaced in the ledgers and here; PR-8geawr accepted | done 2026-11-03, unmerged on `worktree-org-io-publish-2-rul` | see branch |
| T16h, T16c4, T16g, T18 | done (merged; records in §5) | |
| Conformance rounds 1–2; check-traceability; merge review round 1 (the last) | done | b97eff4 |
| T17b close-out (merge-change steps 5–7) | open | |

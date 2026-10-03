# Fix the two stale problem reports — Implementation Plan

**Goal:** resolve PR-zz4exm (device removal accepts the current key as its
replacement) and PR-hvg2dy (the chain reader's doc-comment names the wrong
finality), the two `STALE-PROBLEM` findings that fail every merge to master.
**Implements:** LLR-s97ywt, LLR-w92psx (amended 2026-10-03 in
`org-members/docs/architecture/2026-09-17-decomposition.md`), hence REQ-ewdg2q
and REQ-r784fu; resolves PR-zz4exm (org-members), PR-hvg2dy (org-node).
**Safety class:** C (org-members and org-node; no per-item overrides).
**Verification:** org-members `verify_commands` (`cargo test -p org-members`,
`quint typecheck quint/membership.qnt`, `quint typecheck quint/protocol.qnt`)
and coverage `make coverage-org-members`; org-node `verify_commands` (the
`cargo test -p org-node --features app,test-support ...` line and
`quint typecheck quint/protocol.qnt`); `check-trace.sh` per unit,
`check-ids.sh`, `check-units.sh`.

Environment note (measured 2026-10-03): quint 0.33.0 wants Rust evaluator
v0.7.0 and `~/.quint` holds only v0.6.0 and is not writable from the agent
sandbox, so `mbt_conformance` fails at baseline with "Quint returned non-zero
code". Running with `QUINT_HOME=<scratch dir>` lets quint fetch v0.7.0 there;
baseline is then green. `QUINT_HOME` is quint's own override (`config.js`).

## Owner decision (2026-10-03)

When the replacement member-as-a-group key equals the member's current key,
`delete_p2p_device` and `emergency_isolate_member` return an error and change
nothing: the device stays enrolled and the key is unchanged.

Design choices made under it:

- New variant `OrgMembersError::P2pKeyNotReplaced`, message
  `"replacement p2p key equals the current key"`. No downstream crate matches
  `OrgMembersError` exhaustively (grep, 2026-10-03), so the variant is additive.
- Check order: `IdNotFound` → (`delete_p2p_device` only) `DeviceNotFound` →
  `P2pKeyNotReplaced`. The model mirrors this order exactly; conformance
  compares error tags.
- `rotate_p2p_key` (LLR-k89ahd) is **not** changed. It removes no device, so
  REQ-ewdg2q does not reach it; a same-key rotation is a no-op that revokes
  nothing it claimed to. Raised with the owner, not folded in.

### T1 — Reject the unchanged replacement key in the crate

**Files touched:** `org-members/src/error.rs`, `org-members/src/trie.rs`,
`org-members/tests/integration_test.rs`
**Parallel:** no (first)

1. Add to `integration_test.rs` after `delete_p2p_device_nonexistent_member_fails`:

   ```rust
   /// verifies: REQ-ewdg2q, LLR-s97ywt
   ///
   /// PR-zz4exm's reproducing test. Passing the member's current key back as
   /// the replacement must be refused whole: the device stays enrolled and
   /// the key is unchanged (owner decision 2026-10-03).
   #[test]
   fn delete_p2p_device_rejects_unchanged_key() {
       let trie = TestTrie::genesis(vec![jan_jan()]).unwrap();
       let current = *trie.get(&member_id("jan-jan-id")).unwrap().p2p_key();
       let err = trie.delete_p2p_device(
           &member_id("jan-jan-id"),
           &device_key("jan-jan-d1"),
           current,
       );
       assert_eq!(err.unwrap_err(), OrgMembersError::P2pKeyNotReplaced);
       let member = trie.get(&member_id("jan-jan-id")).unwrap();
       assert!(member.has_p2p_device(&device_key("jan-jan-d1")));
       assert_eq!(member.p2p_device_count(), 2);
       assert_eq!(member.p2p_key(), &current);
   }

   /// verifies: REQ-ewdg2q, LLR-s97ywt
   ///
   /// The last-device path must refuse the same way: a refused removal does
   /// not isolate the member.
   #[test]
   fn delete_p2p_device_last_device_rejects_unchanged_key() { /* alice, alice-d1 */ }

   /// verifies: LLR-s97ywt
   ///
   /// Check order: an unknown device with the current key reports
   /// DeviceNotFound, not P2pKeyNotReplaced.
   #[test]
   fn delete_p2p_device_unknown_device_with_unchanged_key_reports_device() { .. }
   ```

   and after `emergency_isolate_member_nonexistent_fails`:

   ```rust
   /// verifies: REQ-ewdg2q, LLR-w92psx
   #[test]
   fn emergency_isolate_member_rejects_unchanged_key() { /* jan_jan, both devices kept, key kept */ }

   /// verifies: LLR-w92psx, LLR-v3jqau
   /// An absent member reports IdNotFound before any key comparison.
   #[test]
   fn emergency_isolate_member_nonexistent_with_any_key_reports_id() { .. }
   ```

   (The full bodies are in the test file; each asserts the error variant and
   that `trie.get(..)` on the *original* trie shows devices and key unchanged —
   the operations are persistent, so "changed nothing" is also asserted by the
   returned `Err` carrying no new trie.)
2. `cargo test -p org-members --test integration_test` → fails to compile:
   `no variant named P2pKeyNotReplaced`. Add the variant to `error.rs` with
   `#[error("replacement p2p key equals the current key")]`; re-run → the
   `rejects_unchanged_key` tests fail with `Ok(OrgTrie { .. })`. That is the
   red for the right reason (PR-zz4exm reproduced).
3. In `trie.rs`, `delete_p2p_device`: after `remove_device(device)?`, add
   `if existing.p2p_key() == &new_p2p_key { return Err(OrgMembersError::P2pKeyNotReplaced); }`;
   `emergency_isolate_member`: same check after the lookup. Update both
   doc-comments to state the refusal.
4. Re-run → green. Commit.

**Done (142d8e1, merged).** red -> green:
- `delete_p2p_device_rejects_unchanged_key` — E0599 (no variant), then `Ok(OrgTrie { member_count: 1, .. })` where `P2pKeyNotReplaced` was expected; green with the guard.
- `delete_p2p_device_last_device_rejects_unchanged_key` — E0599, then `Ok(OrgTrie { .. })`; green with the guard.
- `emergency_isolate_member_rejects_unchanged_key` — E0599, then `Ok(OrgTrie { .. })`; green with the guard.
- `delete_p2p_device_unknown_device_with_unchanged_key_reports_device` — never red by design: a check-order guard (DeviceNotFound precedes the key check), green before and after.
- `emergency_isolate_member_nonexistent_with_any_key_reports_id` — never red by design: check-order guard (IdNotFound precedes the key check).
Result: 119 passed (integration 113, fuzz 6). `mbt_conformance` red as predicted ("State invariant failed") — T3's red.

### T2 — Fuzz the device-removal operations

**Files touched:** `org-members/tests/fuzz_tests.rs`
**Parallel:** no (serial, after T1)

Add a proptest `device_removal_never_keeps_key` (`verifies: LLR-s97ywt,
LLR-w92psx`): a random sequence of `DeleteDevice(member, device, key_variant)`,
`Isolate(member, key_variant)`, `AddDevice`, `Rotate` ops over a small pool
where `key_variant` frequently equals the current key. Properties per op:
on `Ok`, the new key differs from the previous key; on
`Err(P2pKeyNotReplaced)`, the trie is the previous trie (devices and key
identical) and the supplied key equalled the current key; no panic. Red
check: temporarily dropping the T1 guard must red this test (record the
measurement in the commit message).

**Done (99601ff, merged).** red -> green:
- `device_removal_never_keeps_key` — with both guards removed, minimised to
  `[DeleteDevice(0, 0, Current)]` "succeeded but kept the member's key"; with
  only the isolate guard removed, minimised to `[Isolate(0, Current)]`, same
  message — each guard caught on its own. Guards restored (trie.rs diff empty),
  green. Result: 120 passed (fuzz 7, integration 113).

### T3 — Quint model and conformance driver gain the error

**Files touched:** `quint/membership.qnt`, `org-members/tests/mbt_conformance.rs`
**Parallel:** no (serial, after T1)

1. Before editing the model, run `cargo test -p org-members --test
   mbt_conformance` with T1 in: it must fail (the generator picks `gen` from
   {0,1,2} and members start at gen 0, so the current key is picked as the
   replacement in about a third of `DeleteDevice`/`Isolate` steps) — the
   driver maps the new error to `Other:P2pKeyNotReplaced`, which mismatches.
2. `mbt_conformance.rs` `err_tag`: map `P2pKeyNotReplaced` →
   `"P2pKeyNotReplaced"`. Still red: model says Ok.
3. `membership.qnt`: `deleteDevice` gains
   `else if (s.get(id).pKey == newKey) Err("P2pKeyNotReplaced")` after the
   `DeviceNotFound` branch; `isolate` gains it after `IdNotFound`. Add
   `run deleteDeviceSameKeyRejectedTest`, `run isolateSameKeyRejectedTest`
   (the named scenarios), and `run deleteDeviceUnknownBeforeSameKeyTest`
   (check order). `quint test quint/membership.qnt` green;
   `quint typecheck` green; conformance green.

**Done (9132e48, merged).** red -> green:
- `membership_conformance` with T1 only — "Specification and implementation states diverge / State invariant failed" (seed 0xeb822420); after the `err_tag` mapping still diverged 2/2 (model said Ok); green after the model change, 5/5 runs.
- `deleteDeviceSameKeyRejectedTest` — QNT508 Assertion failed (membership.qnt:208) before the model change; passes after.
- `isolateSameKeyRejectedTest` — QNT508 Assertion failed (membership.qnt:213); passes after.
- `deleteDeviceUnknownBeforeSameKeyTest` — never red by design (check-order guard).
- `membership_conformance` now also `verifies: LLR-s97ywt, LLR-w92psx`, earned by measurement: each crate guard removed alone reddened conformance 3/3 runs.
`quint test quint/membership.qnt`: 24 passing.

### T4 — Ledger updates for PR-zz4exm

**Files touched:** `org-members/docs/problems/2026-08-31-device-removal-key-check.md`,
`org-members/docs/requirements/2026-08-31-org-membership.md`,
`org-members/docs/risk/2026-09-02-membership-hazards.md`,
`org-members/docs/risk/2026-09-17-design-derived.md`
**Parallel:** no (serial, after T1–T3)

- PR-zz4exm: `status: resolved`, `resolved: <fixing commit>` note naming the
  tests. (The squash SHA is unknown until merge; the note cites this change's
  branch and its fixing worktree commit.)
- REQ-ewdg2q's "not met today" paragraph: dated note that the clause is met.
- Hazard register / design-derived risk: dated notes that RC-mqtks7's
  second clause is now implemented. HAZ-s39gbh is **not** re-scored here — its
  residual also rests on the wire-path bypass, which this change does not touch;
  re-evaluation belongs to `analyze-risks`.

**Done** (by the dispatcher; docs only). PR-zz4exm `status: resolved` with
a dated paragraph (the worktree SHA is squashed away, so the note names the
branch, plan and verification record). org-members `check-trace.sh`: open 2,
oldest 16 days — no STALE-PROBLEM.

### T5 — PR-hvg2dy: correct the chain reader's doc-comment

**Files touched:** `org-node/src/chain_read.rs`,
`org-node/docs/problems/2026-09-09-org-node-problems.md`
**Parallel:** yes (disjoint from T1–T4)

`refresh()` doc-comment: reads the Organisation state at the latest
**finalised** block (`at = None`, which org-node holds
on-chain-client to as REQ-ysyu9g) and caches it; `get_org_state` reads no block
and returns the snapshot of the last `refresh()`, so freshness is the caller's
refresh discipline. Documentation only, no behaviour change, so no new test;
org-node's verify line must stay green. Mark PR-hvg2dy `status: resolved`.

**Done (ac27530, merged).** red -> green: n/a — doc-comments only, no
behaviour to test. Facts checked against `on-chain-client/src/client.rs`
(`at = None` = latest finalised) and `org-node/src/service.rs:1528-1540`
(production reads through `ChainOps::read_state`, not this reader). The ledger
has no `resolved:` field (`check-trace.sh` accepts `status: open|resolved`
only), so resolution is a dated prose paragraph. org-node `check-trace.sh`:
exit 0, PR-hvg2dy absent from findings.

## Review round 1 fixes (merge-change 6a)

**fix-r1 (36d3fac, merged).** red -> green:
- `device_removal_never_keeps_key` (tightened; finding-3) — with the key check
  moved before `remove_device`, red: "DeleteDevice(0, 3, Current) reported
  P2pKeyNotReplaced for an absent device"; restored, green.
- `emergency_isolate_member_already_isolated_rejects_unchanged_key` (new;
  finding-5, `verifies: LLR-w92psx`) — with the isolate guard removed, red:
  `unwrap_err()` on `Ok` at integration_test.rs:613; restored, green.
Result: `cargo test -p org-members` fuzz 7, integration 114, mbt 1, 0 failed.

Dispatcher, docs: PR-z463w5 filed (finding-1, earlier-key reuse, open);
LLR-w92psx rationale extended for the already-isolated case (finding-5).

## Review rounds 2–3 fixes

Round 2 (docs, dispatcher): register summary correction (finding-1); PR-z463w5
extended to `emergency_isolate_member` and to key identity by encoding
(findings 2–3).

Round 3: dated corrections at every living statement that still described
PR-zz4exm or the chain-reader report as open, in both units and org-node's
README (findings 1–5, dispatcher); **fix-r3 (4f895cb, merged)** removed the
re-reads of the original persistent trie after a refused call, which could not
fail, and reworded the comments: atomicity holds by construction (`&self` →
`Result<Self, _>`), and the evidence is the `Err(P2pKeyNotReplaced)` itself
(finding-7). red -> green spot-check: delete guard disabled →
`delete_p2p_device_rejects_unchanged_key`,
`delete_p2p_device_last_device_rejects_unchanged_key`,
`device_removal_never_keeps_key` red; restored, green (fuzz 7, integration 114,
mbt 1).

## Review round 4 fixes (docs only, dispatcher)

finding-1 (requirement): PR-fzu25w filed — `delete_member` + `add_member` with
the same key and one device fewer removes a device with no key change; the
"met on the direct API" note qualified, and the register's "must go through
`delete_p2p_device`" corrected. Findings 2–4 (record): PR-z463w5 sequence count,
two review-fix ledgers' "has carried since" dispositions, and design-derived's
"closes outright" claim corrected.

## Review round 5

Gate green (tree 8dad8e82). Reviewer: claims hold, code correct, mutations all
caught; finding-1 (requirement): a removal may install the removed device's own
key — recorded as a third facet of PR-z463w5 (docs only).

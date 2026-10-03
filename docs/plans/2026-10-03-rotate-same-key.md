# rotate_p2p_key refuses the current key — Implementation Plan

**Goal:** apply the owner's 2026-10-03 rulings on the two problem reports the
previous change opened: `rotate_p2p_key` refuses the exact current key;
PR-z463w5 is otherwise resolved by ruling (a key no longer held is not
refused — see ruling 1 and the key-uniqueness scope extension); PR-fzu25w is resolved by ruling (a deleted `MemberId` is never
legitimately re-added).
**Implements:** LLR-k89ahd (amended 2026-10-03 in
`org-members/docs/architecture/2026-09-17-decomposition.md`; `satisfies: derived`,
assessed in `org-members/docs/risk/2026-09-17-design-derived.md`). Resolves
PR-z463w5 and PR-fzu25w (the latter by ruling).
**Safety class:** C (org-members).
**Verification:** org-members `verify_commands` (`cargo test -p org-members`,
`quint typecheck quint/membership.qnt`, `quint typecheck quint/protocol.qnt`),
`quint test quint/membership.qnt`, `make coverage-org-members`; dependents'
`verify_commands` per `check-units.sh --impact`; `check-trace.sh`, `check-ids.sh`,
`check-units.sh`. Quint runs need `QUINT_HOME=<scratch>` (quint 0.33 evaluator
v0.7.0; `~/.quint` is unwritable from the agent sandbox).

## Owner rulings (2026-10-03)

1. By contract only the member's *current* key is refused, by all three
   operations. A key held earlier, a device's own key, and a non-canonical
   encoding are not checked (PR-z463w5's facets) — not defects. (Narrowed
   by the key-uniqueness scope extension below: every key still held is
   refused with `DuplicateKey`, the removed device's own included; what stays
   unchecked is a key no longer held and a non-canonical encoding.)
2. `rotate_p2p_key` returns an error when given the exact current key.
3. A deleted member is deleted permanently. A member added later is a new
   member with a new `MemberId`, whatever its handle or keys; delegation is by
   `MemberId`, never by handle, because handles change. `MemberId`s are
   caller-generated random values, so re-adding a deleted id is a caller error
   outside the API's contract; the trie keeps no tombstones and does not
   enforce it (PR-fzu25w, resolved by ruling; the installed guardrails
   0.5.1 has no `accepted` status).

Design: reuse `OrgMembersError::P2pKeyNotReplaced` ("replacement p2p key
equals the current key"). Check order: `IdNotFound` → `P2pKeyNotReplaced`.

### T1 — rotate_p2p_key refuses the current key (crate, tests, fuzz, model)

**Files touched:** `org-members/src/trie.rs`,
`org-members/tests/integration_test.rs`, `org-members/tests/fuzz_tests.rs`,
`quint/membership.qnt`, `org-members/tests/mbt_conformance.rs`
**Parallel:** no (single task)

1. RED, integration: next to `rotate_p2p_key_nonexistent_fails`, add

   ```rust
   /// verifies: LLR-k89ahd
   ///
   /// Owner ruling 2026-10-03: rotating to the exact current key is refused.
   /// The refusal is atomic by construction (`&self` → `Result<Self, _>`);
   /// the evidence is the error itself.
   #[test]
   fn rotate_p2p_key_rejects_unchanged_key() {
       let trie = TestTrie::genesis(vec![alice()]).unwrap();
       let current = *trie.get(&member_id("alice-id")).unwrap().p2p_key();
       let err = trie.rotate_p2p_key(&member_id("alice-id"), current);
       assert_eq!(err.unwrap_err(), OrgMembersError::P2pKeyNotReplaced);
   }

   /// verifies: LLR-k89ahd, LLR-v3jqau
   /// An absent member reports IdNotFound before any key comparison.
   #[test]
   fn rotate_p2p_key_nonexistent_with_any_key_reports_id() { /* ghost-id → IdNotFound */ }
   ```

   Watch the first fail with `Ok(OrgTrie { .. })`.
2. RED, conformance: with the crate guard in and the model unchanged,
   `mbt_conformance` diverges (the generator draws `gen` from {0,1,2}; members
   start at gen 0). Then add the model branch in `rotateKey`
   (`else if (s.get(id).pKey == newKey) Err("P2pKeyNotReplaced")` after
   `IdNotFound`) and a `run rotateKeySameKeyRejectedTest` (watched fail first).
3. Fuzz: extend `fuzz_tests.rs` so rotation draws the current key often, and
   assert: on `Ok` the key changed; on `Err(P2pKeyNotReplaced)` the supplied key
   equalled the current key. Red with the guard removed.
4. GREEN: in `rotate_p2p_key`, after the lookup,
   `if existing.p2p_key() == &new_p2p_key { return Err(OrgMembersError::P2pKeyNotReplaced); }`;
   doc-comment states it. Existing callers of `rotate_p2p_key` in tests that
   rotate to a fresh key are unaffected; any that rotate to the same key must be
   found and judged, not silently changed.

### T2 — Ledger: the two rulings

**Files touched:** `org-members/docs/problems/2026-10-03-earlier-key-reuse.md`,
`org-members/docs/problems/2026-10-03-delete-readd-keeps-key.md`, and the dated
notes the previous change added that call those facets open gaps:
`org-members/docs/requirements/2026-08-31-org-membership.md`,
`org-members/docs/risk/2026-09-02-membership-hazards.md`,
`org-members/docs/risk/2026-09-17-design-derived.md`
**Parallel:** yes (disjoint from T1)

PR-z463w5 → `status: resolved` with a dated ruling paragraph (rotate fixed by
T1; other facets ruled outside the contract by the owner). PR-fzu25w →
`status: resolved` with a paragraph stating ruling 3. Dated notes at the
gap listings. HAZ-s39gbh still not re-scored (the wire path stands).

## Done

**T1 (b99f11f, merged).** red -> green:
- `rotate_p2p_key_rejects_unchanged_key` — before the guard: `unwrap_err()` on `Ok(OrgTrie { member_count: 1, .. })`.
- `rotate_p2p_key_nonexistent_with_any_key_reports_id` — never red by design (check order).
- `device_removal_never_keeps_key` (extended with `Rotate(_, KeyChoice)`, now also `verifies: LLR-k89ahd`) — before the guard: "Rotate(0, Current) succeeded but kept the member's key".
- `membership_conformance` — crate guard in, model unchanged: "Specification and implementation states diverge"; green after the model branch. Guard removed: red (so `LLR-k89ahd` added to its `verifies:` by the dispatcher).
- `rotateKeySameKeyRejectedTest` — QNT508 before the model branch.
`mbt_conformance.rs`'s driver needed no code change. `trie_ops_never_panic_and_count_consistent` may now rotate to the genesis key and get `Err`, which it ignores; it does not claim LLR-k89ahd.
Result: fuzz 7, integration 116, mbt 1, 0 failed; `quint test` 25 passing.

**T2 (dispatcher, docs).** PR-z463w5 and PR-fzu25w resolved by ruling; gap notes updated.

Coverage (gate, tree `91778ac4`): org-members lines 93.56% (floor 92), regions
92.24% (floor 91); decision coverage unmeasured (known class C gap); the new
guard's both outcomes are exercised by `rotate_p2p_key_rejects_unchanged_key`
and `rotate_p2p_key_changes_only_key`.

## Review round 1 fixes

finding-1/2 (record): the register, SRS note and design-derived now say the
rulings transfer the checks to the caller rather than remove the risk, and that
only the same-id re-add is out of contract — a fresh `MemberId` given a removed
device's key is in contract and caller-borne. finding-3 (requirement): security
check  11 added to `org-members/README.md`; caller-duty doc-comments on
`rotate_p2p_key`, `delete_p2p_device`, `emergency_isolate_member`, `add_member`,
`delete_member` (fix-r1). finding-4 (record): coverage recorded above.

## Review round 2 fixes (dispatcher)

finding-1 (record): SRS intro note now says the rulings move the checks to the
caller. finding-2 (requirement): README security check 11 and the three
doc-comments now name any key any removed device holds — another member's
earlier key or device key included. finding-3 (record): design-derived's
lead-in counts three points. finding-4 (record): the quint-connect follow-up is
in this change's verification record.

## Scope extension: key uniqueness (owner decision, after review round 3)

Round 3's finding-1 (requirement): one member could be given a key another
member holds. Owner decision 2026-10-03: **enforce in code, on every path
(apply_delta included), for member keys and device keys.** New derived LLRs in
`org-members/docs/architecture/DRAFT-worktree-rotate-same-key-key-uniqueness.md`:
LLR-v6gfc7 (the invariant; `genesis`, `add_member`, `add_p2p_device`),
LLR-fym7dy (key-replacing operations refuse any key held before the operation,
the removed device's included), LLR-gjj6bx (`apply_delta`). Assessed in
`org-members/docs/risk/DRAFT-worktree-rotate-same-key-key-uniqueness.md`.
**Implements** gains LLR-v6gfc7, LLR-fym7dy, LLR-gjj6bx.

New error `OrgMembersError::DuplicateKey` ("key already held in this
organisation"). Check orders (after every existing check of each operation):
- `genesis` / `add_member`: DuplicateId → handle/skeleton → **DuplicateKey**
  (the leaf's member key vs its own devices, and every key vs the record's keys).
- `add_p2p_device`: IdNotFound → DeviceSlotsFull → DuplicateDevice → **DuplicateKey**
  (corrected after T3: the slots check runs before the duplicate-device check).
- `rotate_p2p_key`: IdNotFound → P2pKeyNotReplaced → **DuplicateKey**.
- `delete_p2p_device`: IdNotFound → DeviceNotFound → P2pKeyNotReplaced → **DuplicateKey**
  (the key of the device being removed counts as held).
- `emergency_isolate_member`: IdNotFound → P2pKeyNotReplaced → **DuplicateKey**.
- `apply_delta`: existing checks in their current order → **DuplicateKey**.

### T3 — DuplicateKey in the crate (tests, fuzz)

**Files touched:** `org-members/src/error.rs`, `org-members/src/trie.rs`
(and `org-members/src/delta.rs` only if `CandidateTrie` must carry a new
index), `org-members/tests/integration_test.rs`, `org-members/tests/fuzz_tests.rs`
**Parallel:** no

Implementation shape: a key index like `handle_index` (key bytes → owner),
maintained by `insert_leaf`/`update_leaf`/`delete_by_id`, `genesis` and
`apply_delta`, carried through `CandidateTrie`. RED first per LLR, normal and
abnormal cases, `verifies:` the LLR IDs: shared member key across members via
`add_member`, `genesis`, `rotate_p2p_key`, `delete_p2p_device`,
`emergency_isolate_member`; a device key enrolled under two members; a member
key equal to its own or another member's device key; `delete_p2p_device` with
the removed device's own key; `apply_delta` producing a shared key, and giving a
member a key held in the base record; check-order tests where a refusal's
precedence matters. Fixture collisions in existing tests are test artifacts:
report each, fix the fixture, never weaken the check. A fuzz property asserting
the invariant holds after every successful operation.

### T4 — Quint model and conformance

**Files touched:** `quint/membership.qnt`, `quint/membership_mbt.qnt`,
`org-members/tests/mbt_conformance.rs` (and `quint/protocol.qnt` only if its
use of the shared helpers forces it — semantics unchanged)
**Parallel:** no (serial, after T3)

The model gains `DuplicateKey` with the same orders. For conformance to
exercise it, model key equality must equal real key equality: map member and
device keys through one derivation in the driver, give initial devices a
different `gen` from the initial member key, and let rotations and device
additions draw keys of other members. Add named `run` scenarios.

### T5 — Docs follow the code

README security check 11 (what the software now refuses vs what stays the
caller's), the five caller-duty doc-comments, PR-z463w5/PR-fzu25w notes, and
review round 3 findings 2–3.

**T3 (28e9a2c, merged).** 29 tests; red -> green: build failed with E0599 (no
`DuplicateKey`), then 18 tests `unwrap_err()` on `Ok` until the check existed;
10 never red by design (6 check-order, 4 accept-normal); `keys_stay_unique`
red with each check switched off ("genesis accepted a shared key", "Isolate(0,
9) succeeded with a key already held", "DeltaHandOver(2, 0) succeeded with a key
already held"). org-members: integration 146, fuzz 8, mbt 1, 0 failed; wasm32
check passes. Found: (a) LLR-gjj6bx's base-record clause makes the change-set
path refuse what the sequential direct API accepts — clause removed (T3b);
(b) org-node fixtures use one key as admin member key and device key — 16
org-node tests fail (T3c).

### T3b — apply_delta checks the resulting record only

**Files touched:** `org-members/src/trie.rs`, `org-members/tests/integration_test.rs`,
`org-members/tests/fuzz_tests.rs`. **Parallel:** yes (with T3c).

### T3c — org-node fixtures use a distinct admin device key

**Files touched:** `org-node/src/test_fixtures.rs` and any org-node test that
builds a leaf with one key as both member and device key. **Parallel:** yes.

**T3b (da0270a, merged).** red -> green: `apply_delta_accepts_change_set_of_delete_device_then_rotate_to_its_key`, `..._delete_member_then_add_with_its_key`, `apply_delta_accepts_member_key_freed_by_a_removed_member` — each `Err(DuplicateKey)` against the base-record clause; `keys_stay_unique` gained a genesis→current round-trip check and was red against that clause ("change set after DeltaUpsert(1, 1, 3) refused: Some(DuplicateKey)"). Clause removed; integration 147, fuzz 8, mbt 1, 0 failed.

**T3c (0225e18, merged).** org-node fixtures (`test_fixtures.rs`, `verify_against_chain.rs`, `transport_handshake.rs`, `transport_networked.rs`, `fuzz_verify_against_chain`) give the admin a distinct device key — in the transport tests the endpoint key the handshake authenticates, which the old fixture did not enrol. org-node: 33 passed / 20 failed + 1 fuzz panic → 53 passed, 0 failed, both fuzz targets ok; no assertion changed. app 78/78 unchanged. The snapshot-less first-admission fallback in `service.rs` now fails at `genesis` with `Trie(DuplicateKey)` instead of later at the base-root check; still closed, and no test reaches it.

**T4 (aa058a5, merged).** red -> green: extended generator + unchanged model diverged (seed 0x6dd1d523: `AddDevice(b, {b,0})` Ok in the model, DuplicateKey in the crate); 20 new `run`s failed before their model branches (3 `keysUnique` runs QNT404 until the helper existed); 13 never red by design (check order, accept cases). 61 passing after. Crate mutations, 3 seeds each: add_member, add_p2p_device, rotate, delete_p2p_device (after adding `DeleteEnrolledDevice`), isolate — red 3/3; apply_delta's result check green 3/3 (conformance feeds only canonical deltas of accepted mutations), so `membership_conformance` gains LLR-v6gfc7 and LLR-fym7dy but not LLR-gjj6bx, which the integration tests and `keys_stay_unique` carry. Also found: the model's addDevice checked DuplicateDevice before DeviceSlotsFull, the reverse of the crate — fixed and pinned. `update_leaf`'s re-index makes rotate's op-level held-key refusal redundant (backstop). Conformance does not reach genesis with members (driver calls it empty).

**T5 (consistency pass, whole-branch review).** One wording for refusals,
check orders and caller duties across the seven doc-comments, the error doc,
README check 11, LLR-fym7dy, the Quint comments and the ledger notes:
statements that the software refuses "only the current key" or does not refuse
"a device's own key" now say "a key no longer held" (decomposition note,
PR-z463w5 ruling, SRS note under REQ-ewdg2q, two HAZ-s39gbh notes, this plan's
ruling 1). README's "What the crate does on its own" gains key uniqueness. No
behaviour, `verifies:` line or item ID changed. org-members: integration 147,
fuzz 8, mbt 1; `quint test` 61; org-node lib 23, verify_against_chain 13,
transport_handshake 3, fuzz target ok; `cargo doc` clean.

## Merge of master (32e907c) and post-merge measurement

master brought the quint-connect coupling (model moved to `org-members/quint/`, split into `membership_types.qnt`; generator and driver rewritten) and two-phase handle handover in `apply_delta` (LLR-n5t6bn). Resolved by porting this change's DuplicateKey model, generator and driver changes into master's layout. The key index was already two-phase; two integration tests added and measured red against a one-phase index (`apply_delta_accepts_change_set_of_member_key_swap`, `..._device_key_moving_between_members`). master's `quint_preflight` probed `$HOME/.quint` only; it now probes `$QUINT_HOME` when set — the directory quint itself uses — so the gate runs without write access to `~/.quint` (deliberately read-only).

**meas (24efedb, merged).** Conformance mutations, 3 runs each: rotate's P2pKeyNotReplaced guard → `membership_conformance` red; add_member DuplicateKey → `scenario_add_member_duplicate` red; add_p2p_device, rotate (op check + re-index), delete_p2p_device, isolate → their scenarios and `membership_conformance` red; apply_delta result check → `scenario_apply_delta_duplicate` red; one-phase key index → `scenario_member_key_swap` red, `scenario_device_key_moves` green (no verifies line). genesis DuplicateKey is not reached by conformance (the `genesis_rejects_*` integration tests carry it). Verifies lines set to exactly the measured IDs. 32 passed, 0 failed.

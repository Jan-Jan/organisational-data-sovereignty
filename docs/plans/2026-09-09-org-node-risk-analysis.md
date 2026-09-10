# org-node risk analysis (tooth 3) — implementation plan

**Goal:** give the org-node unit its own ISO 14971 hazard register, the
requirements that realise its risk controls, and the test evidence the
traceability gate reads — relocating the verify-path unit tests into
`org-node/tests` and adding the abnormal-input cases — while moving the
chain-reader problem report into org-node's ledger and filing the two defects
the code survey found.
**Implements:** RC-6a2dke, RC-pm9kmx, RC-e5atck, RC-m4r75s, RC-e2uvje,
RC-gfn6kr, RC-b6mydy, RC-wqgm2p, RC-jjsz97 through REQ-wp2nyc, REQ-bvh8v6,
REQ-gju89b, REQ-ag6kqm, REQ-8gz8bu, REQ-nhe2zu, REQ-6yu72z, REQ-mr5abb,
REQ-9g6as6, REQ-bcxz96, REQ-eg5j8u, REQ-xa6smf, REQ-ztdza4, REQ-uxv2x2,
REQ-hzm4kt.
**Safety class:** C (org-node; every unit is C). No per-item overrides.
**Verification:** per unit over `check-units.sh --impact master..HEAD`. The
impact set is **all four units, every one of them `touched`**: org-members,
on-chain-client, org-node and app. The reason is the manifest: this change
edits `.guardrails/units.yaml`, and `check-units.sh` maps any change under the
root `.guardrails/` directory to every unit, on the ground that every unit's
gates ran under the old manifest or the old scripts
(`.guardrails/scripts/check-units.sh:186-192`). Measured 2026-09-10:

```
$ sh .guardrails/scripts/check-units.sh --impact master..HEAD
org-members	touched
on-chain-client	touched
org-node	touched
app	touched
```

The set was three when this plan was written — org-members and org-node
touched, app dependent on both — and on-chain-client joined it in fix round 3,
which corrected the export-surface comment in `.guardrails/units.yaml`. Nothing
about on-chain-client's code changed; the manifest edit is what puts it in the
blast radius, and running its gate is the safe direction. org-node's
`verify_commands` after this change:

```
cargo test -p org-node --features app,test-support --lib --test service_stories --test transport_handshake --test transport_networked --test fuzz_envelope_decode --test fuzz_verify_against_chain --test verify_against_chain --test wire_frame_bound --test store_at_rest --test admission_sender
quint typecheck quint/protocol.qnt
```

The other three units' `verify_commands` are unchanged by this change; the
gate runs each unit's own entries, whatever their number.

on-chain-client's entries, measured here because the corrected impact set adds
that unit to the merge's evidence — 2026-09-10, agent sandbox,
`CARGO_HOME=/tmp/cargo_home_fuzz`, exit 0: `cargo test --manifest-path
on-chain-client/Cargo.toml --lib --test fuzz_decode_org_state --test
fuzz_parse_revive_event --test fuzz_event_round_trip` gives **lib 23 passed, 0
failed**; the three bolero targets report no pass count, each printing its
iteration total and exiting on its one-second budget (fuzz_decode_org_state
265,607 rng inputs over 4 corpus inputs; fuzz_event_round_trip 39,349 rng
inputs; fuzz_parse_revive_event 240,443 rng inputs over 6 corpus inputs).

Branch `worktree-guardrails-org-node-risk`, change worktree
`.claude/worktrees/guardrails-org-node-risk`. Task worktrees nest at
`.worktrees/worktree-guardrails-org-node-risk-t<N>` inside it.

## What was decided, and by whom

The `analyze-risks` interview, 2026-09-08, owner Jan-Jan, one question at a
time:

1. **Scope: the whole unit.** Not only the eight places org-members' register
   names org-node, but the publish path, the iroh transport, the blob exchange
   and the persona store. Reason: the class C rationale for org-node is that
   it holds the keys and does the I/O, and the survey had already found a
   hazard in the publish path no other unit's analysis can see.
2. **Evidence: integration tests in `org-node/tests`.** The verify-path and
   sequence unit tests inside `src` are relocated there, annotated, and the
   abnormal-input cases added beside them; the new targets join
   `verify_commands` and the CI step. `test_paths` stays `org-node/tests`.
   Rejected: widening `test_paths` to `org-node/src`.
3. **The defects: problem reports now, fixes later.** PR-vt244s (publish
   before persist) and PR-2dmjzj (loopback admission unbound to the joiner's
   Device key) are filed open; their hazards are prose; their controls are
   not minted. Rejected: fixing them in this change; prose without reports.
   A third, PR-u4c2vp (the revocation receive path runs neither clause of
   RC-b6mydy, so its `UpdatedNotRevoked` branch commits a record from an
   unchecked sender), was added in fix round 1 under this same decision when
   review round 1 asked what scoped RC-b6mydy's second clause.
4. **PR-hvg2dy: move, keep open, retarget `affects:`.** To org-node's ledger
   with original text and date; `affects:` now RC-6a2dke; org-members'
   register reworded, with a dated correction, to name the file that holds the
   report rather than its ID.
5. **The expectations carry no `(implements:)` yet.** REQ-ysyu9g and
   REQ-q92yac are named in the register as the provider-side halves of
   RC-6a2dke and RC-wqgm2p's residuals; the annotation follows delivery.

## The code survey the register rests on

Made 2026-09-08 by a read-only subagent over every file of `org-node/src` and
`org-node/tests`, with `file:line` citations, and re-checked by the author
against the tree for every citation the register uses (`cite-check`, 42
ranges; two were corrected at the time). The author's check was not clean:
review round 1 found five further ranges that did not match the tree, and fix
round 1 corrected them — the two `SeqGuard` citations in `verify.rs`, the
`sequence.rs` range, `read_state` in `service.rs`, and the `chain_read.rs`
doc-comment range. Review round 2 found two more, corrected in fix round 2:
the `ChainOpsReader` construction (`service.rs:1011`, cited as `:1012-1013`,
which is the blank line and the call) and `org_pub_key` (cited as `:640`,
which is `admin_member_key`; the field is at `:636`). Of those two, only the
`org_pub_key` bullet below carries the wrong citation: it is the survey as it
was made, and `:640` in it should be `:636`. The `ChainOpsReader` bullet's
`:1532-1540` was right in the survey and is still right; the `:1012-1013`
error existed only in the register, which is now the corrected record for
both. The third correction is to the last bullet: panics are denied at
`lib.rs:2` and `Cargo.toml:22-25`, but `test_fixtures.rs:10` allows
`unwrap` and `expect` in a module the `test-support` feature compiles into the
library, and no gate runs clippy on this crate at all. The findings that
became hazards or reports:

- `receive_and_verify` reads the chain itself (`service.rs:922-926`), takes
  the author key from that state (`:928`), hands verification a one-shot
  adapter (`:1532-1540`), cross-checks the sender against the invite on first
  admission only when an invite exists (`:985-1000`), and against the
  verified record afterwards (`:1018-1027`); the store is written only after
  `Ok` (`:1077-1115`).
- `admit_member` and `revoke_member` write the chain (`:814`, `:1172`) before
  the record (`:864-871`, `:1254-1261`), with the send (`:836-853`,
  `:1218-1238`) between them returning early on error → PR-vt244s.
- Loopback `admit_member` dials `peer_addr` as given (`:836-842`); the
  networked branch derives the peer from the Device key (`:847-849`) →
  PR-2dmjzj.
- `receive_and_self_delete_if_revoked` discards the authenticated sender
  (`:1337`); the revocation message carries `org_secret: None` (`:1214`); the
  presence filter at `:1330` can yield a spurious self-delete.
- The transport authenticates a Device key and nothing more
  (`transport/endpoint.rs:5`, `:236-263`); `MAX_FRAME` enforced at
  `wire.rs:26-28`, `:37-39`, `endpoint.rs:257`; blobs unbounded and unsigned
  (`blobs.rs:9-45`).
- Store: Argon2 over a fixed salt (`store.rs:103-105`), XChaCha20-Poly1305,
  in-place non-atomic write (`:149`).
- Authorship is one key, `org_pub_key` = admin Member-as-a-group key
  (`service.rs:616`, `:640`); chain dispatch is threshold-1 or direct, the
  higher-threshold outcome dead (`chain_write/multisig.rs:124-142`,
  `chain_write/mod.rs:43-69`).
- No `verifies:` anywhere in org-node before this change; no reorganisation
  test anywhere (the `chopsticks_reorg.rs` helper only mines blocks); panics
  denied at `lib.rs:2` and `Cargo.toml:22-25`, none in non-test code.

## Files this change touches

Author's task (T1), done directly in the change worktree — the analysis is
the author's work and dispatching it would put the ledger in a subagent's
context instead of the interview's:

- `org-node/docs/risk/2026-09-09-org-node-hazards.md` — new: six HAZ, nine RC, seven prose hazards, residual-risk table, seventeen not-minted controls (fourteen at fix round 2; controls 15 and 16 added in fix round 5 with the two re-assessed introduced hazards, control 17 in fix round 6 with the third), derived assessments of the fifteen REQs.
- `org-node/docs/requirements/2026-09-09-verify-and-commit.md` — new: fifteen REQs.
- `org-node/docs/problems/2026-09-09-org-node-problems.md` — new: PR-hvg2dy (moved), PR-vt244s, PR-2dmjzj, PR-u4c2vp (filed in fix round 1 after review round 1 asked what scopes RC-b6mydy's second clause), and PR-w88sr9 (filed in fix round 7: the `WireMessage::genesis_snapshot` doc-comment contradicted by the revocation send path this change assesses). Five open reports, all under `problem_open_max: 10`.
- `org-members/docs/problems/2026-09-02-chain-reader-finality-doc.md` — the item removed, a dated pointer left; the file defines no item.
- `org-members/docs/risk/2026-09-02-membership-hazards.md` — wording kept, with dated 2026-09-09 corrections wherever this change makes it out of date, each naming what remains open; one word replaced, a problem report's ID, which now names the file that holds it, because a provider may not cite a consumer's items. No item altered.
- `org-node/docs/CONTEXT.md` — the unit glossary, filled.
- `org-node/.guardrails/config.yaml` — ledger comment; four targets added to `verify_commands`.
- `.github/workflows/rust.yml` — the same four targets in org-node's step.
- `org-node/Cargo.toml` — four `[[test]]` entries with `required-features`.
- `org-node/src/lib.rs`, `org-node/src/test_fixtures.rs` — fixtures exposed under `test-support` (`cfg(any(test, feature = "test-support"))`, `pub mod`).
- `org-node/tests/fuzz_envelope_decode/fuzz_target.rs`, `org-node/tests/fuzz_verify_against_chain/fuzz_target.rs` — `verifies:` annotations only.
- `org-node/tests/service_stories.rs` — `verifies:` annotations, and two tests beyond the original story test: the abnormal case of REQ-uxv2x2 added by T6 (another member's revocation is committed, not self-deleted) and a second abnormal case added in fix round 6 (an unverified revocation leaves the record in place).
- `org-node/tests/{verify_against_chain,wire_frame_bound,store_at_rest,admission_sender}.rs` — stubs (feature gate and a comment) so the `[[test]]` entries resolve before T2–T5 fill them.
- `docs/plans/2026-09-05-ratchet-gap-analysis.md` — tooth 3 row: org-node part done.
- `docs/plans/2026-09-05-ratchet-setup.md` — items added.
- this plan.

Eight more files the diff touches that this list omitted until fix round 5,
which audited it against `git diff --stat master...HEAD`. Four were edits no
round had recorded here:

- `.guardrails/units.yaml` — the export-surface comment corrected in fix round 3: org-node has seventeen requirements now, not two, and still exports none. This is the edit that makes the impact set all four units.
- `org-node/docs/requirements/2026-09-06-dependency-expectations.md` — a dated 2026-09-09 correction added in fix round 3, saying the register those two expectations waited for now exists and naming the hazard each stands against; extended in fix round 5 to repoint the finality report at org-node's ledger.
- `org-node/docs/risk/2026-09-06-dependency-expectations.md` — two dated 2026-09-09 corrections added in fix round 3, saying org-node now has hazards of its own and that its register does evaluate against the matrix.
- `org-node/tests/transport_networked.rs` — a comment corrected in fix round 3: the inline `genesis_and_admit` helper predates `test_fixtures` being reachable from an integration test, and is kept inline only to leave the test unchanged.

Four are source files that lost an inline `#[cfg(test)]` module when its tests
were relocated into `org-node/tests`, which the list described only from the
receiving end:

- `org-node/src/verify.rs`, `org-node/src/sequence.rs` — test modules removed by T2, their tests relocated into `org-node/tests/verify_against_chain.rs`.
- `org-node/src/transport/wire.rs` — test module removed by T3 (`org-node/tests/wire_frame_bound.rs`).
- `org-node/src/store.rs` — test module removed by T4 (`org-node/tests/store_at_rest.rs`).

With those eight entries the list matches the diff exactly: 29 files in
`git diff --stat master...HEAD`, 29 accounted for here, and no entry naming a
file the diff does not touch.

## Tasks T2–T5 — the test evidence

All four are dispatched under `develop-change`, one task worktree each, and
may run **in parallel**: their file sets are disjoint. Each relocates or adds
tests only; **no production behaviour changes**. Every test carries
`// verifies: <IDs>` on the line above its `#[test]`/`#[tokio::test]`
attribute (a `//` comment, so `ids_matching 'verifies:'` reads it).

**Red by mutation.** These requirements describe behaviour the crate already
has, so a new test passes on first run. That proves nothing, and the iron law
still applies: for each test the subagent makes the named temporary mutation
to the source under test, runs the test, watches it fail *for that reason*,
reverts the mutation with `git checkout -- <file>` (one plain command), and
runs the test green. The mutation is never committed. The dispatch report's
`red -> green:` line names the mutation. A test that stays green under its
mutation is a test that does not verify what it claims: fix the test, not
the annotation.

**Removing the relocated unit tests.** Where a task relocates a `#[cfg(test)]
mod tests` from `src`, it deletes that module from the source file in the
same commit, so the behaviour is tested once, where the gate reads it. The
`--lib` count drops accordingly (35 → 23: seven from `verify.rs`, two from
`sequence.rs`, two from `transport/wire.rs`, one from `store.rs`).
`envelope.rs` and `service.rs` keep their unit tests, and `envelope.rs:96`
still uses `crate::test_fixtures`, so the module stays; `service.rs`'s test
module does not use the fixtures.

Common preamble for every new test file:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used)]
```

plus the feature gate named per task. Build and run with, from the task
worktree root:

```sh
CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test <target>
```

(no `--offline`; the scratch home fetches what it lacks). Log to a file
outside the tree, e.g. `$TMPDIR/<target>.log`.

### T2 — verify_against_chain.rs

**Files touched:** `org-node/tests/verify_against_chain.rs`,
`org-node/src/verify.rs`, `org-node/src/sequence.rs`
**Parallel:** yes

Gate: `#![cfg(feature = "test-support")]`. Imports:

```rust
use org_members::RootHash;
use org_node::chain::{MockChain, OrgState};
use org_node::ids::OrgId;
use org_node::keys::SigningKeypair;
use org_node::sequence::SeqGuard;
use org_node::test_fixtures::{admit_member_delta, genesis_trie, Trie};
use org_node::verify::{verify_envelope_against_chain, VerifyContext};
use org_node::{OrgNodeError, SignedDeltaEnvelope};
```

`setup()` is the unit test's `setup()` verbatim (admin from seed `[1u8; 32]`,
`genesis_trie(&admin, &admin)`, `admit_member_delta(&admin)`,
`OrgId::new([5u8; 20])`, envelope at `parent_seq` 2, returns
`(admin, org, local, env, new_root)`). A `ctx()` helper builds the
`VerifyContext` with `SeqGuard::from_last_seen(1)` and
`last_committed_epoch: 1`; a `chain_at(org, root, epoch)` helper seeds a
`MockChain`.

Relocated, one `// verifies:` each — the seven from `verify.rs` and the two
from `sequence.rs`. Their bodies are the unit tests' bodies, plus what the fix
rounds added on top: `rejects_root_mismatch_when_chain_root_differs` gained
two assertions on the sequence-number high-water mark, and REQ-nhe2zu and
REQ-mr5abb alongside REQ-wp2nyc in its annotation, when a later review round
found REQ-mr5abb's rejection clause untested.

| test | verifies | mutation for red |
|---|---|---|
| `happy_path_commits_when_root_matches_chain` | REQ-nhe2zu | in `verify.rs`, replace `seq_guard.advance(envelope.parent_seq)` with nothing → `out.seq_guard.last_seen()` is 1, not 2 |
| `rejects_wrong_org_id` | REQ-gju89b | delete step 1's `if` |
| `rejects_bad_signature` | REQ-ag6kqm | delete step 2's `if` |
| `rejects_stale_seq` | REQ-6yu72z | delete step 3's `ctx.seq_guard.check(...)?;` |
| `rejects_when_org_absent_from_chain` | REQ-bvh8v6 | replace `.ok_or(OrgNodeError::OrgNotOnChain)?` with `.unwrap_or(OrgState { root_hash: RootHash::from_bytes([0u8; 32]), org_pub_key: [0u8; 32], epoch: 99 })` (compiles; test then fails with RootMismatch instead of OrgNotOnChain) |
| `rejects_root_mismatch_when_chain_root_differs` | REQ-wp2nyc | replace step 8's `verify_against(&on_chain.root_hash)` with `verify_against(&candidate.root_hash()?)` |
| `rejects_stale_epoch` | REQ-8gz8bu | change `<=` to `<` in step 7's epoch check |
| `rejects_equal_and_lower_seq` (SeqGuard) | REQ-6yu72z | in `sequence.rs` `check`, change `>` to `>=` |
| `advance_moves_high_water_mark_forward_only` | REQ-mr5abb | in `advance`, delete the `if` so it always assigns |

New, abnormal-input:

```rust
/// Transcript the envelope signs: org_id ‖ parent_seq (LE) ‖ delta_bytes.
fn sign_over(admin: &SigningKeypair, org: OrgId, seq: u64, delta_bytes: &[u8]) -> [u8; 64] {
    let mut t = Vec::new();
    t.extend_from_slice(org.as_bytes());
    t.extend_from_slice(&seq.to_le_bytes());
    t.extend_from_slice(delta_bytes);
    admin.sign(&t).to_bytes()
}

// verifies: REQ-gju89b
#[test]
fn rejects_wrong_org_before_decoding_delta() {
    // Garbage delta bytes: if the org check did not come first, the error
    // would be MalformedDelta.
    let (admin, org, local, _env, _) = setup();
    let garbage = SignedDeltaEnvelope {
        org_id: org, parent_seq: 2, delta_bytes: vec![0xff; 16],
        signature: sign_over(&admin, org, 2, &[0xff; 16]),
    };
    let ctx = VerifyContext { expected_org_id: OrgId::new([0xee; 20]), ..ctx(&admin) };
    assert_eq!(
        verify_envelope_against_chain(&local, &garbage, &ctx, &MockChain::new()).unwrap_err(),
        OrgNodeError::OrgIdMismatch
    );
}
```

(`VerifyContext` has no `Default`; write the struct out in full instead of
`..ctx(&admin)` if the borrow of `admin.verifying_key()` makes the helper
awkward — a local `let vk = admin.verifying_key();` and a full literal is the
simplest form.) Same shape for:

- `rejects_bad_signature_before_decoding_delta` — REQ-ag6kqm: garbage bytes
  signed by an imposter (`SigningKeypair::from_seed([0xaa; 32])`), expected
  org, honest author key in ctx → `BadSignature`, not `MalformedDelta`.
  Mutation: swap steps 2 and 4 in `verify.rs` (decode before signature).
- `rejects_stale_seq_before_decoding_delta` — REQ-6yu72z: garbage bytes
  correctly signed at `parent_seq` 1 against `SeqGuard::from_last_seen(1)` →
  `StaleSeq { got: 1, last_seen: 1 }`, not `MalformedDelta`. Mutation: swap
  steps 3 and 4.
- `check_does_not_advance_the_mark` — REQ-mr5abb: `let g =
  SeqGuard::from_last_seen(5); g.check(6).unwrap(); assert_eq!(g.last_seen(),
  5);` — `check` takes `&self`, so the assertion documents the contract;
  mutation for red: none is possible through a shared reference, so this test
  is **declared green-from-birth in the report** with that reason (the
  reviewer judges whether the contract is worth the row; the author says yes,
  because the REQ says "never on a rejection" and this is the line that makes
  it true).

Expected: `test result: ok. 13 passed` for the target; `--lib` shows 26 (the
seven verify and two sequence tests gone).

### T3 — wire_frame_bound.rs

**Files touched:** `org-node/tests/wire_frame_bound.rs`,
`org-node/src/transport/wire.rs`
**Parallel:** yes

Gate: `#![cfg(all(feature = "transport", feature = "test-support"))]`.
Relocate `frame_round_trips` and `oversize_body_is_rejected` from
`wire.rs`'s test module (using `org_node::test_fixtures::admit_member_delta`
for `sample_msg`, `org_node::transport::wire::{encode_frame, decode_body,
WireMessage}`, `org_node::transport::{TransportError, MAX_FRAME}`), delete
the module from `wire.rs`, and add:

```rust
// verifies: REQ-eg5j8u
#[test]
fn oversize_message_is_rejected_on_encode() {
    let mut msg = sample_msg();
    msg.genesis_snapshot = Some(vec![0u8; MAX_FRAME + 1]);
    assert!(matches!(encode_frame(&msg), Err(TransportError::FrameTooLarge(_))));
}
```

All three carry `// verifies: REQ-eg5j8u` (round trip = normal case; the two
rejections = abnormal). Mutations: for `oversize_body_is_rejected`, delete
the size check in `decode_body`; for `oversize_message_is_rejected_on_encode`,
delete it in `encode_frame`; for `frame_round_trips`, change the prefix to
big-endian on encode only (the test's own prefix assertion reads little-endian
and fails). Expected: `3 passed`; `--lib` down by two more.

### T4 — store_at_rest.rs

**Files touched:** `org-node/tests/store_at_rest.rs`, `org-node/src/store.rs`
**Parallel:** yes

Gate: `#![cfg(feature = "app")]`. Relocate `round_trips_encrypted_through_disk`
(it already asserts the wrong-passphrase failure; keep it as the normal case)
and delete the module from `store.rs`. Add:

```rust
// verifies: REQ-hzm4kt
#[test]
fn wrong_passphrase_yields_error_not_data() {
    let path = tmp_path("wrongpw");
    let mut s = PersonaStore::open(path.clone(), "correct horse").unwrap();
    s.data_mut().personas.push(persona([7u8; 32], [8u8; 32]));
    s.save(&mut OsRng).unwrap();
    let err = PersonaStore::open(path.clone(), "wrong").err().expect("must fail");
    assert!(err.to_string().contains("decrypt failed"), "got {err}");
}

// verifies: REQ-hzm4kt
#[test]
fn seeds_do_not_appear_in_the_file() {
    let path = tmp_path("plaintext");
    let mut s = PersonaStore::open(path.clone(), "pw").unwrap();
    let member_seed = [0x5au8; 32];
    let device_seed = [0xa5u8; 32];
    s.data_mut().personas.push(persona(member_seed, device_seed));
    s.save(&mut OsRng).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert!(!contains(&bytes, &member_seed), "member seed in clear");
    assert!(!contains(&bytes, &device_seed), "device seed in clear");
    assert!(!contains(&bytes, b"alice"), "handle in clear");
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}
```

with `persona(member_seed, device_seed) -> PersonaRecord` (fields as in the
relocated test, handle `"alice"`) and `tmp_path(suffix)` helpers, and
`use org_node::store::{PersonaRecord, PersonaStatus, PersonaStore}; use
rand::rngs::OsRng;` (check `rand` is a dev-dependency of org-node; it is used
by `service_stories`). Mutations: in `save`, write `pt` instead of `ct`
(plaintext on disk) → `seeds_do_not_appear_in_the_file` fails; in `open`,
replace the `decrypt` error branch with `StoreData::default()` on failure →
`wrong_passphrase_yields_error_not_data` fails; for the relocated test, the
plaintext mutation also fails its wrong-passphrase assertion. Expected:
`3 passed`; `--lib` down by one more. Remove the temp files at the end of
each test as the relocated test does.

### T5 — admission_sender.rs

**Files touched:** `org-node/tests/admission_sender.rs`
**Parallel:** yes

Gate: `#![cfg(all(feature = "app", feature = "test-support"))]`. Model the
setup on `service_stories.rs` (stores in `std::env::temp_dir()`, shared
`MockChainOps`, `OrgService::new`, `create_persona`, `create_organisation`,
`export_invite`/`import_invite`, `export_join_request`/`import_join_request`,
endpoints bound from the personas' `device_seed` so the authenticated sender
equals the invite's `admin_device_key`, `tokio::time::timeout` of 30 s around
every network step, `#[tokio::test(flavor = "multi_thread")]`). Three tests:

```rust
// verifies: REQ-xa6smf
#[tokio::test(flavor = "multi_thread")]
async fn first_admission_from_a_device_other_than_the_invites_admin_is_rejected() {
    // A: admin persona + org. B: persona, imports A's invite, exports a join
    // request. R: a rogue endpoint on a third device key, seed [0x33; 32].
    // A admits B but is given R's address as the peer address, so R receives
    // the admission WireMessage (envelope + org_secret + genesis snapshot).
    // R relays the identical WireMessage to B's endpoint.
    // B.receive_and_verify must return Err (the sender is R, not A's device),
    // B.list_orgs() must be empty, and B's persona must not be Active.
}

// verifies: REQ-ztdza4
#[tokio::test(flavor = "multi_thread")]
async fn update_from_the_admin_after_admission_is_committed() {
    // A admits B directly (B receives, commits epoch 2). A creates persona C,
    // imports C's join request, and admits C giving B's address as the peer
    // address, so B receives the C-admission envelope from A's own device.
    // B.receive_and_verify must return Ok with epoch 3 and B's record must
    // list three members.
}

// verifies: REQ-ztdza4
#[tokio::test(flavor = "multi_thread")]
async fn update_relayed_by_a_non_member_after_admission_is_rejected() {
    // As above up to B's admission. A admits C giving R's address; R receives
    // the envelope and relays it to B. B.receive_and_verify must return Err
    // and B's record must still be at epoch 2 with two members.
}
```

`OrgEndpoint::recv_one` returns `(WireMessage, P2pDeviceKey)` per the README
(check the actual tuple order in `transport/endpoint.rs:236` — the code
returns `(remote_key, msg)`); `OrgEndpoint::send(addr, &msg)` relays. B's
service needs an endpoint before each receive (`with_endpoint`); rebind a
fresh endpoint per receive as `service_stories` does, keeping the same
`b_addr` the admin was given. Mutations: for the first test, in
`service.rs:985-1000` delete the `return Err(OrgNodeError::BadSignature);`
inside the invite check → B accepts R's relay → the `is_err()` assertion
fails; for the third, delete the `return Err` in the `sender_known` block at
`:1018-1027`; for the second (the positive case), invert the check — replace
`if !sender_known` with `if sender_known` — so the genuine update from the
admin is rejected and the `Ok` assertion fails. Expected: `3 passed`, each
inside a few seconds on loopback.

### T6 — the abnormal case of REQ-uxv2x2 (added after the first gate)

**Files touched:** `org-node/tests/service_stories.rs`
**Parallel:** no (serial, after T2–T5 landed)

The first gate run (2026-09-09) found REQ-uxv2x2 with a normal-case test
only (story 5). The abnormal case: a revocation Change set that removes a
*different* member must be committed and must **not** self-delete. Add one
test to `service_stories.rs`, same infrastructure as `five_stories_full_e2e`:

```rust
// verifies: REQ-uxv2x2
#[tokio::test(flavor = "multi_thread")]
async fn revocation_of_another_member_is_committed_not_self_deleted() {
    // A creates persona + org; B is admitted directly (as in stories 1–4).
    // A creates persona C, imports C's join request, admits C giving B's
    // address as the peer address, so B receives and commits epoch 3 with
    // three members. A then revokes C giving B's address, so B receives the
    // revocation via receive_and_self_delete_if_revoked.
    // Expect SelfDeleteOutcome::UpdatedNotRevoked { org_id }, B's OrgRecord
    // still present at epoch 4 with two members (admin + B), B's persona
    // still Active.
}
```

Mutation for red: in `receive_and_self_delete_if_revoked`
(`service.rs:1325-1335`), force `let my_still_present = false;` → B deletes
its record on C's revocation → the `UpdatedNotRevoked` match and the
"record still present" assertions fail. Expected: `service_stories` 2 passed.

## After the tasks

Merge each task branch into the change branch with
`git -c commit.gpgsign=false merge --no-ff worktree-guardrails-org-node-risk-t<N>`,
remove the task worktree, delete the task branch; record each report's
`red -> green:` lines below as they land. Then `check-traceability`,
`verify-before-merge` (dispatched gate; log outside the tree), and
`merge-change` per unit over the impact set — all four units, org-members,
on-chain-client, org-node and app, each reported `touched` because the change
edits the root manifest (see **Verification** above) — with
the independent review at 6a in `.worktrees/worktree-guardrails-org-node-risk-review`,
the record at `docs/verification/<date>-worktree-guardrails-org-node-risk.md`,
and the signed squash by the owner.

## Red → green attestations (filled as the dispatch reports land)

Each line is the dispatched subagent's own attestation, copied from its
report: the mutation applied to the source under test, the failure observed,
then the revert and the green run. The subagent that ran the loop is the only
party that saw the red.

**fix6 — done** (review round 6's dispositions):

- `unverified_revocation_leaves_the_record_in_place` — REQ-uxv2x2 — in
  `receive_and_self_delete_if_revoked`, `verify_envelope_against_chain?`
  replaced by a `match` whose `Err(_)` arm ran the self-delete branch and
  returned `Ok(SelfDeleteOutcome::SelfDeleted)`; failed with `a revocation
  whose envelope fails verification must be rejected: SelfDeleted { org_id:
  … }`; reverted; green. What the test establishes: a revocation the node
  rejects must leave its record untouched — the abnormal-input case a gate
  round found REQ-uxv2x2 to be missing, the rejected message never reaching
  the delete branch at all.

**fix2 — done** (review round 2's dispositions; two commits, tests then
documents):

- `update_relayed_by_a_non_member_after_admission_is_rejected` — now
  `verifies: REQ-ztdza4, REQ-mr5abb` — the `sender_known` block moved to
  after the store write and `save`, so the record commits before the sender
  is checked; failed with `the Sequence-number high-water mark must not
  advance on a rejection / left: 2 right: 1`; reverted; green. The new
  assertion was ordered ahead of the pre-existing epoch assertion, which
  otherwise fired first and hid it.
- `rejects_root_mismatch_when_chain_root_differs` — now `verifies:
  REQ-wp2nyc, REQ-nhe2zu, REQ-mr5abb` — the first mutation tried (return an
  advanced guard before the root match) left the test **green**, because
  `SeqGuard` is `Copy` and the advanced value never reaches the caller on an
  `Err`. The mutation that reddens it gives `SeqGuard` interior mutability
  (`Cell<u64>`, `advance(&self)`) and advances before the root match; it
  failed with `a rejection at the root match must leave the high-water mark
  where it was / left: 2 right: 1`; reverted; green. So the assertion is not
  green from birth, but only barely: making it fail requires redesigning the
  type rather than slipping in the logic. Worth recording as the honest
  strength of that row.

fix2 also corrected three things the dispositions implied but did not name:
the not-minted control count (thirteen to fourteen), a setup-checklist item
for the new control, and a dated note in this section recording round 2's two
citation corrections. And it found that one instruction described a document
that does not exist — the requirements file lists no module-by-module
accounting of the surveyed modules, only the modules *without* a requirement
— so the accounting was made exhaustive there instead.

**fix1 — done** (review round 1's dispositions; two commits, source-and-test
then documents):

- `organisation_secret_does_not_appear_in_the_file` — REQ-hzm4kt — `save`
  made to write plaintext instead of ciphertext; failed with `org secret in
  clear`; reverted; green. Closes the evidence gap review finding-6 named:
  the Organisation secret lives in `OrgRecord.org_secret` and no test wrote
  one into a store before searching the file.
- `rejects_root_mismatch_when_chain_root_differs` now carries
  `verifies: REQ-wp2nyc, REQ-nhe2zu` (review finding-9). No new mutation was
  needed: T2's attestation for that test — step 8 made to verify against the
  candidate's own root, failing with `unwrap_err()` on an `Ok(VerifiedUpdate
  { .. })` — reddens both IDs, because the mutation makes the conditional
  commit rule commit where it must not.

fix1 also found a sixth stale citation the review had not flagged: the
register cited `org-node/src/verify.rs:46-85` for
`verify_envelope_against_chain`, whose `Ok(...)` is at `:86` and closing
brace at `:87`. Corrected to `:46-87` in the change worktree, as author work
on the author's own document, and reviewed in round 2 with everything else.

**T2 — done** (`org-node/tests/verify_against_chain.rs`; `verify.rs` and
`sequence.rs` test modules removed; target 13 passed; lib 26 at the time):

- `happy_path_commits_when_root_matches_chain` — REQ-nhe2zu — deleted
  `seq_guard.advance(envelope.parent_seq)` in `verify.rs`; failed with
  `out.seq_guard.last_seen()`: left 1, right 2; reverted; green.
- `rejects_wrong_org_id` — REQ-gju89b — deleted step 1's `if`; failed with
  left `OrgNotOnChain`, right `OrgIdMismatch`; reverted; green.
- `rejects_bad_signature` — REQ-ag6kqm — deleted step 2's `if`; failed with
  left `OrgNotOnChain`, right `BadSignature`; reverted; green.
- `rejects_stale_seq` — REQ-6yu72z — deleted step 3's
  `ctx.seq_guard.check(...)?;`; failed with `unwrap_err()` on an
  `Ok(VerifiedUpdate { .. seq_guard: SeqGuard { last_seen: 2 }, epoch: 2 })`;
  reverted; green.
- `rejects_when_org_absent_from_chain` — REQ-bvh8v6 —
  `.ok_or(OrgNotOnChain)?` replaced by `.unwrap_or(OrgState { root_hash:
  [0; 32], org_pub_key: [0; 32], epoch: 99 })`; failed with left
  `RootMismatch`, right `OrgNotOnChain`; reverted; green.
- `rejects_root_mismatch_when_chain_root_differs` — REQ-wp2nyc — step 8
  verifies against the candidate's own root (`let cand_root =
  candidate.root_hash(); candidate.verify_against(&cand_root)`; the plan's
  spelling did not compile, see surprises); failed with `unwrap_err()` on an
  `Ok(VerifiedUpdate { .. })`; reverted; green.
- `rejects_stale_epoch` — REQ-8gz8bu — step 7 `<=` changed to `<`; failed
  with `unwrap_err()` on an `Ok(VerifiedUpdate { .. epoch: 1 .. })`;
  reverted; green.
- `rejects_equal_and_lower_seq` — REQ-6yu72z — `sequence.rs` `check`: `>`
  changed to `>=`; failed with left `Ok(())`, right `Err(StaleSeq { got: 5,
  last_seen: 5 })`; reverted; green.
- `advance_moves_high_water_mark_forward_only` — REQ-mr5abb — `advance`'s
  `if` deleted (always assigns); failed with left 2, right 3; reverted;
  green.
- `rejects_wrong_org_before_decoding_delta` — REQ-gju89b — `decode_delta()?`
  moved ahead of step 1; failed with left `MalformedDelta`, right
  `OrgIdMismatch`; reverted; green.
- `rejects_bad_signature_before_decoding_delta` — REQ-ag6kqm —
  `decode_delta()?` moved between steps 1 and 2; failed with left
  `MalformedDelta`, right `BadSignature` (the wrong-org test stayed green, as
  it should); reverted; green.
- `rejects_stale_seq_before_decoding_delta` — REQ-6yu72z — `decode_delta()?`
  moved between steps 2 and 3; failed with left `MalformedDelta`, right
  `StaleSeq { got: 1, last_seen: 1 }` (org and signature tests stayed green);
  reverted; green.
- `check_does_not_advance_the_mark` — REQ-mr5abb — **green from birth**, as
  the plan declares: `check` takes `&self`, so no mutation through a shared
  reference can move the mark; the assertion documents the contract the type
  system enforces. It also stayed green under the `advance` mutation.

T2 surprises: the plan's root-mismatch mutation
(`verify_against(&candidate.root_hash()?)`) fails to compile twice over —
`root_hash()` on the applied candidate returns `RootHash`, not `Result`, and
`verify_against` moves `candidate` — so the equivalent above was used.
Deleting a check also reddens the matching before-decoding test with
`MalformedDelta`, and the `check` `>=` mutation reddens all three `StaleSeq`
tests: expected overlap, not mis-targeted tests. One harness stall after
mutation 4; resumed from `git status` with the mutation still applied, red
already in the log, revert and green completed normally.

**T3 — done** (`org-node/tests/wire_frame_bound.rs`; `transport/wire.rs`
test module removed; target 3 passed; lib 33 at the time, before T2/T4
landed):

- `frame_round_trips` — REQ-eg5j8u — length prefix changed to
  `to_be_bytes()` in `encode_frame` only; failed with `assertion left ==
  right failed: left: 184614912, right: 267` (the test's little-endian prefix
  read); reverted; green.
- `oversize_body_is_rejected` — REQ-eg5j8u — deleted the `body.len() >
  MAX_FRAME` check in `decode_body`; failed with `assertion failed:
  matches!(decode_body(&big), Err(TransportError::FrameTooLarge(_)))`;
  reverted; green.
- `oversize_message_is_rejected_on_encode` — REQ-eg5j8u — deleted the same
  check in `encode_frame`; failed with `assertion failed:
  matches!(encode_frame(&msg), Err(TransportError::FrameTooLarge(_)))`;
  reverted; green.

**T4 — done** (`org-node/tests/store_at_rest.rs`; `store.rs` test module
removed; target 3 passed; lib 34 at the time):

- `seeds_do_not_appear_in_the_file` — REQ-hzm4kt — `save` writes `pt`
  instead of `ct`; failed with `member seed in clear`; reverted; green.
- `round_trips_encrypted_through_disk` — REQ-hzm4kt — same plaintext
  mutation; failed at the correct-passphrase reopen with `Chain("decrypt
  failed (wrong passphrase?)")` (plaintext bytes do not decrypt — the right
  reason, reached one assertion earlier than the plan predicted); and with
  `open`'s decrypt-error branch replaced by `StoreData::default()`, failed
  with `assertion failed: PersonaStore::open(path, "wrong").is_err()`;
  reverted; green.
- `wrong_passphrase_yields_error_not_data` — REQ-hzm4kt — `open`'s
  decrypt-error branch replaced by `StoreData::default()`; failed with `must
  fail` (open returned Ok); reverted; green.

**T5 — done** (`org-node/tests/admission_sender.rs`; target 3 passed, each
test on its own loopback endpoints and its own store directory):

- `first_admission_from_a_device_other_than_the_invites_admin_is_rejected` —
  REQ-xa6smf — deleted `return Err(OrgNodeError::BadSignature);` inside the
  first-admission invite check (`service.rs:995`); failed with `B must reject
  a first admission whose sender is not the invite's admin device, got
  Ok(ReceiveOutcome { epoch: 2, .. })`; reverted; green.
- `update_from_the_admin_after_admission_is_committed` — REQ-ztdza4 —
  inverted `if !sender_known` to `if sender_known` (`service.rs:1024`);
  failed with `B must accept an update sent by the admin's own device:
  BadSignature`; reverted; green. (This inversion also fails the third test,
  as expected; that run showed 1 passed / 2 failed.)
- `update_relayed_by_a_non_member_after_admission_is_rejected` — REQ-ztdza4
  — deleted `return Err(OrgNodeError::BadSignature);` inside the
  `sender_known` block (`service.rs:1025`); failed with `B must reject an
  update whose sender is not a member device, got Ok(ReceiveOutcome { epoch:
  3, .. })`; reverted; green.

T5 note: persona C is created in A's store, per the plan's wording;
`admin_persona_for_org` matches by member key, so A stays the admin.

**T6 — done** (`org-node/tests/service_stories.rs`; target 2 passed):

- `revocation_of_another_member_is_committed_not_self_deleted` — REQ-uxv2x2
  — forced `let my_still_present = false;` in
  `receive_and_self_delete_if_revoked` (`service.rs:1325`, keeping the
  original expression as `let _ = …` so the borrows and the
  unused-variable lint still compile); failed with `B must not self-delete
  when a different member is revoked` (1 passed, 1 failed); reverted; green.

T6 note, and the reason this task existed: `five_stories_full_e2e` stays
green under that mutation, correctly — story 5 asserts `SelfDeleted`, which
is what the forced `false` produces. Only the abnormal case can see the
defect, which is exactly what the class C robustness rule is for. The first
gate run found the gap.

T4 surprise worth keeping: `git checkout -- store.rs` restores HEAD, which
re-adds the deleted test module, so the deletion had to be re-applied after
each revert; the committed file has the module removed and nothing else
changed. T3 avoided this by committing the module deletion before the
mutation cycles.

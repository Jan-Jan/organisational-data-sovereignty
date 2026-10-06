# Organisation key pair — Implementation Plan

**Goal:** implement change 3 of the chain-authority sequence: the
Organisation secret is gone and the Organisation private key reaches every
Member — every Organisation-information Wire message carries the key the
sending node's record holds, a receiver stores it only once its public half
is the chain's `org_pub_key` (RC-9cefcn), a revocation (sent only to a Device
the committed record no longer lists) carries none and is accepted only as the
receiver's own removal, every provisional update draws a fresh Organisation
key pair that its commit takes into the record, and the invite identifier
never travels between peers.

**Implements:**

- org-node requirements, new
  (`org-node/docs/requirements/2026-10-07-org-key-pair.md`):
  REQ-szq3ud, REQ-stx9v3, REQ-jy6ybw, REQ-c29s93, REQ-bwx7eg, REQ-ju6vn2,
  REQ-3dsweu, REQ-vxqc5g.
- org-node requirements, amended in place: REQ-8amu2a
  (`2026-10-06-chain-authority.md`), REQ-hzm4kt
  (`2026-09-09-verify-and-commit.md`), REQ-y7tsft (`2026-10-04-type-safety.md`).
- app requirement, amended in place: REQ-tcutr6
  (`app/docs/requirements/2026-10-06-invitation.md`).
- org-node control, new: RC-9cefcn
  (`org-node/docs/risk/2026-10-07-org-key-pair.md`;
  implemented by REQ-c29s93 and REQ-bwx7eg).
- org-node design, amended in place (`2026-10-03-decomposition.md`): SDD-swtd3w,
  SDD-kwncn7, SDD-af5vnt, SDD-89es4z, SDD-rx2yvy, SDD-8cpyfa, SDD-72ddm6.
- org-node low-level requirements, new
  (`org-node/docs/architecture/2026-10-07-org-key-pair.md`):
  LLR-js9dsu, LLR-ecxc76, LLR-j5vbqj, LLR-qsjde3, LLR-byjvd9, LLR-6ymd6d,
  LLR-e2b7gv, LLR-xn5pwc, LLR-ba2ejp, LLR-38e2kn, LLR-6s785x, LLR-4kh9w9,
  LLR-pt32fx.
- org-node low-level requirements, amended in place:
  in `2026-10-03-decomposition.md` LLR-37cj3n, LLR-8hdu9x, LLR-9zfnmb,
  LLR-bg3vsw, LLR-ckk5nz, LLR-ghja3x, LLR-j6j95z, LLR-jn5jeh, LLR-jsx922,
  LLR-jwhzh3, LLR-mbjfq8, LLR-q8emds, LLR-s78sh7, LLR-tax3pm, LLR-u6rq4s,
  LLR-y2v8v2; in `2026-10-04-type-safety.md` LLR-ayrdr8, LLR-bwb9pu,
  LLR-g76zqd, LLR-sz4xhc; in `2026-10-05-unsigned-envelope.md` LLR-2dvhz8,
  LLR-322xfu, LLR-3fwykc, LLR-sj7cd5; in `2026-10-06-chain-authority.md`
  LLR-2xzys9, LLR-48jakr, LLR-7cmp38, LLR-95753m, LLR-cmdrp9, LLR-ms8njy,
  LLR-mxskg9, LLR-qjz3q4, LLR-s6qnht, LLR-s8xp7m, LLR-wzqqg9.
- app low-level requirement, new
  (`app/docs/architecture/2026-10-07-org-key-pair.md`,
  under SDD-rmbr3t): LLR-q225ws.
- app low-level requirements, amended in place: LLR-8krgzj, LLR-ctrfz4 (note
  only) in `app/docs/architecture/2026-10-05-decomposition.md`; LLR-gha5f6,
  LLR-w4mhd4, LLR-f35pda (note only) in `app/docs/architecture/2026-10-06-invitation.md`.

**Amended for this change and already verified, test unchanged (no task
rewrites them):** LLR-322xfu (`node_value_types::the_organisation_private_key_is_a_secret_type_and_the_only_way_to_its_key_pair`
— the amendment only drops `OrgSecret` from the list it is compared with),
LLR-s6qnht (`commit_paths::create_organisation_keeps_a_genesis_update_and_nothing_else`
— `create_organisation` already draws its pair after the genesis root; the
amendment states the order), LLR-mxskg9 (`value_types`, `expected_admission`,
`provisional_store`, `commit_paths` — the amendment rewords
`AdmissionNotExpected`'s scope; T1 rewrites its doc comment). The hazards
HAZ-ep6uzs and HAZ-vxabf9 are amended in place
(`org-node/docs/risk/2026-09-09-org-node-hazards.md`); their controls are
RC-9cefcn (above) and REQ-stx9v3/REQ-3dsweu.

**Resolves:** PR-xwek5e (T5,
`org-node/docs/problems/2026-10-04-secret-cleared-on-update.md`), PR-szkat6
(T9, `org-node/docs/problems/2026-10-04-org-key-conflation.md`), PR-ve9zw8
(T9, `org-node/docs/problems/2026-10-05-org-secret-unauthenticated.md`).
Opened and resolved by this change: PR-g9u3xq
(`org-node/docs/problems/2026-10-07-key-rotation.md`,
already `status: resolved`; T8 writes its reproducing test and adds its name to
the resolution). Opened and left open: app PR-3ue4va
(`app/docs/problems/2026-10-07-update-fan-out.md`;
the fan-out is the `org-io` session's work).

**Safety class:** C (org-node), C (app), C (on-chain-client), from each
unit's `.guardrails/config.yaml`. No per-item overrides.

**Verification:** every `verify_commands` entry of the units, as this plan
leaves them.

org-node (`org-node/.guardrails/config.yaml`; T4 adds `--test fuzz_wire_decode`
after `--test fuzz_first_admission_base`):

```
cargo test -p org-node --features app,test-support --lib --test service_stories --test transport_handshake --test transport_networked --test fuzz_envelope_decode --test fuzz_verify_against_chain --test fuzz_first_admission_base --test fuzz_wire_decode --test verify_against_chain --test wire_frame_bound --test store_at_rest --test admission_sender --test value_types --test key_custody --test envelope_binding --test service_lifecycle --test encoding_golden --test node_value_types --test chain_read_state --test persona_records --test secret_redaction --test organisation_key --test receive_chain_reads --test expected_admission --test absences --test provisional_store --test commit_paths
quint --version
quint typecheck org-node/quint/protocol.qnt
quint typecheck org-node/quint/ods_instances.qnt
quint run org-node/quint/protocol.qnt --invariant=forkSafety --max-steps=16 --max-samples=5000
quint run org-node/quint/protocol.qnt --invariant=revocationSafety --max-steps=16 --max-samples=5000
quint run org-node/quint/protocol.qnt --invariant=revokedExcludedFromOrgSecret --max-steps=16 --max-samples=5000
quint run org-node/quint/protocol.qnt --invariant=tauWindow --max-steps=16 --max-samples=5000
quint run org-node/quint/protocol.qnt --invariant=convergence --max-steps=16 --max-samples=5000
```

(The Quint model is unchanged by this plan; its invariant
`revokedExcludedFromOrgSecret` names the model's abstraction, not org-node's
removed type, and stays as it is.)

app (`app/.guardrails/config.yaml`, unchanged by this plan):

```
cargo test --manifest-path app/src-tauri/Cargo.toml --features test-support --test startup_policy --test connection_status --test org_id_parsing --test receiver_events --test receiver_guard --test ipc --test csp_policy --test state_assembly --test submit_flow --test invitation
npm --prefix app run check
npm --prefix app run test
```

on-chain-client (`on-chain-client/.guardrails/config.yaml`, unchanged).
**on-chain-client is untouched by this change:** it is its own workspace, it
does not depend on org-node, no file under `on-chain-client/` changes, and its
own items name nothing this change amends (checked 2026-10-06: no
`org_secret`, `OrgSecret` or `InviteId` under `on-chain-client/src` or
`on-chain-client/tests`). Its lines are run once, at T12, because the merge
gate runs every unit:

```
cargo test --manifest-path on-chain-client/Cargo.toml --features test-support --lib --test fuzz_decode_org_state --test fuzz_parse_revive_event --test fuzz_event_round_trip --test contract_address_filter --test log_ownership --test decode_revive_event --test decode_org_state --test runtime_version_dispatch --test h160_mapping --test storage_slot_layout --test best_lane_reorg_rule --test type_widths
cargo test --manifest-path on-chain-client/Cargo.toml --features test-support,write --test write_manifest --test write_pure --test write_compose --test write_events
```

Gates, per unit (`<unit>` in org-node, app, on-chain-client):
`GR_CONFIG=<unit>/.guardrails/config.yaml sh .guardrails/scripts/check-trace.sh`,
`GR_CONFIG=<unit>/.guardrails/config.yaml sh .guardrails/scripts/check-ids.sh --allow-draft-files`,
and once `sh .guardrails/scripts/check-units.sh`. Coverage:
`make coverage-on-chain-client` (on-chain-client's `coverage_command`,
unchanged); org-node and the app have no `coverage_command` (a recorded gap
in their configs, unchanged). Clippy with `-D warnings` for org-node and the
app is clean.

---

## Environment (every task)

- Work in the task worktree the dispatcher creates from
  `worktree-org-node-org-key-pair` with
  `sh .guardrails/scripts/task-worktree.sh start tN` (N the task number), run
  in the change worktree; it also copies the ignored build directories. One
  git command per shell call; no `&&`, `;`, heredoc or loop on a line that
  mentions git; write any file whose text contains the word "git" with the
  Write tool. Helper scripts go in the session scratchpad with a `kpp-`
  prefix.
- `CARGO_HOME=/tmp/cargo_home_fuzz` on every cargo command (`~/.cargo` is
  read-only). Add `--offline` if the network is blocked: this plan adds no
  crate, so every lockfile already holds what it needs.
- Quint lines: `QUINT_HOME=<task worktree>/target/quint_home`, populated once
  with `mkdir -p <task worktree>/target/quint_home` then
  `cp -R ~/.quint/. <task worktree>/target/quint_home/`. Never write `~/.quint`.
- In a task worktree, `npm --prefix app ci --offline` restores
  `app/node_modules` before any `npm --prefix app run …`.
- **Lockfiles.** No task adds, removes or moves a dependency. If a cargo
  command changes `app/src-tauri/Cargo.lock` or the root `Cargo.lock`, restore
  it with `git checkout -- app/src-tauri/Cargo.lock` (or `-- Cargo.lock`) and
  say so in the attestation; a lockfile diff is never committed.
- Never use `sed` on a Rust, TypeScript or Svelte file; use the Edit tool
  (`replace_all` where this plan says "every occurrence").
- `test_paths`: org-node reads `org-node/tests`, the app `app/src-tauri/tests`
  and `app/tests`. Every `verifies:` annotation of this plan lives there,
  never in `src/`.
- Find an item's text with `GR_CONFIG=<unit>/.guardrails/config.yaml sh
  .guardrails/scripts/find-items.sh show <ID>` and its references with
  `… find-items.sh refs <ID>`; never read a whole ledger.
- Commit messages: imperative subject naming the task (`T<N>: …`), body
  listing the IDs verified. No Co-Authored-By line (owner preference). Commit
  unsigned in task worktrees (`git -c commit.gpgsign=false commit`); the
  dispatcher merges (`sh .guardrails/scripts/task-worktree.sh merge tN`).
- A removed name must not survive in `org-node/src`, even in a comment:
  `tests/absences.rs` scans the source text for `OrgSecret`, `org_secret`
  (from T2), `InviteId` and `invite_id` (from T3). Reword any comment that
  would name one.
- Each org-node task's last step runs, in its worktree,
  `CARGO_HOME=/tmp/cargo_home_fuzz cargo check -p org-node --all-targets --features app,test-support`
  (expected: `Finished`, no error), so the targets outside the gate (the
  chopsticks targets and `preflight`) never stop compiling unnoticed, and
  `CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy -p org-node --all-targets --features app,test-support -- -D warnings`
  (expected: `Finished`, no warning).
- Keys in tests: a Member-as-a-group key is
  `org_node::test_fixtures::member_key(seed)`, a DevicePublicKey
  `org_node::test_fixtures::device_key(seed)`, a fixed valid Organisation
  public key `org_node::test_fixtures::org_public_key()`.

## Order and parallelism

```
T1                                   org-node errors          (parallel with T2, T3)
T2 → T3 → T4 → T5 → T6 → T7 → T8 → T9   org-node (share service.rs, store.rs,
                                     tests/support/mod.rs, admission_sender.rs)
T4 also needs T1 (the new variants)
T10   serial after T9 and T1          app Rust
T11   parallel with every other task  app webview (api.ts, Admit.svelte, a vitest file)
T12   serial after T10 and T11        docs, problem checks, deslop, full gate
```

- **T1** touches `org-node/src/error.rs` and `org-node/tests/value_types.rs`
  only; no other task touches either, so it runs beside T2 and T3. T4 is the
  first task that uses a variant T1 adds.
- **T2–T9** are serial: each touches `org-node/src/service.rs` and
  `org-node/tests/support/mod.rs`, and most touch `org-node/src/store.rs`,
  `org-node/tests/admission_sender.rs` and `org-node/tests/encoding_golden.rs`.
- **T10** is the app's Rust side; it needs org-node's final interface (T9) and
  T1's variants.
- **T11** touches only `app/src/lib/api.ts`,
  `app/src/lib/components/Admit.svelte` and `app/tests/api.admit.test.ts`; no
  other task touches them and the webview does not build against org-node, so
  it may run at any time.
- **T12** reads everything.

**The app's Rust crate does not compile from T1 until T10.** T1 adds four
`OrgNodeError` variants, and `app/src-tauri/src/events.rs` matches
`OrgNodeError` exhaustively by design (E0004); T2 removes `OrgSecret`, which
`commands.rs`, `invitation.rs` and `submit.rs` import (E0432); T3 removes
`InviteId` and changes `expect_admission`; T2–T3 change `send_update`'s
arity (E0061). T1–T9 run org-node's verify line only and record
`cargo check --manifest-path app/src-tauri/Cargo.toml --features test-support --tests`
as expected-failing (the error codes above); T10 brings the app back and runs
its line. The webview (`npm --prefix app run check`, `npm --prefix app run
test`) builds throughout. `check-trace` is required free of MISSING-TEST for
this change's items only at T12. on-chain-client compiles throughout (it does
not depend on org-node).

## Decisions this plan takes that the inputs did not settle

1. **Seams and interim states.** The data model moves in four steps, each
   leaving org-node compiling and green: T2 replaces the Wire message's
   `org_secret: Option<OrgSecret>` by `org_private_key: Option<OrgPrivateKey>`
   and removes `OrgRecord.org_secret` (the record's key stays an `Option`
   until T7); T3 removes the invite identifier; T4 makes `WireMessage` the
   two-kind enum; T7 makes `OrgRecord.org_private_key` required. Between T4
   and T7 `send_update` refuses with
   `OrgNodeError::Chain("the record holds no Organisation private key")` an
   Organisation-information send from a record without a key; no record
   reaches that state in any test (genesis and every first admission store
   one), and T7 deletes the branch.
2. **Receive-path helpers in `service.rs`.** `commit_held(org_id, verified)`
   takes no key from T2 on (LLR-ckk5nz). The received key is written by
   `store_carried_key(data, org_id, key)` (T2–T8, private key only); T8 adds
   `set_record_keys(data, org_id, private, public)` for `commit_update`
   (LLR-6s785x), and T9 replaces `store_carried_key` by `set_record_keys`
   with the chain's public key on both receive paths (LLR-4kh9w9) and adds
   `check_carried_key(org_id, key, state)` (LLR-ba2ejp).
3. **Where a held revocation goes between T4 and T5.** T4 introduces the kind;
   a revocation about a held Organisation that leaves this node listed is
   still committed as an update (the record keeps its key) until T5 refuses
   it. T4's tests do not depend on that interim; T5's do.
4. **The kind is chosen before the Persona lookup.** `send_update` looks up
   the record first (`OrgNotOnChain` for none, LLR-6ymd6d), then the first
   bound Persona (LLR-2xzys9), then the Loopback address (LLR-pw369n). A node
   holding no record of the Organisation therefore refuses with
   `OrgNotOnChain` where it used to refuse with the Persona message; both
   send nothing and bind nothing.
5. **Display strings** of the four new variants (LLR-j5vbqj):
   `MalformedMessage` — "received wire message is malformed";
   `OrgKeyMismatch` — "organisation private key received for organisation
   {org_id:?} is not the chain's organisation key"; `RevocationNotHeld` —
   "revocation for organisation {org_id:?}, of which this node holds no
   record"; `RevocationNotForThisDevice` — "revocation for organisation
   {org_id:?} leaves this node's device in the record".
6. **Golden values** are derived by the format rule from the pinned values
   now in `org-node/tests/encoding_golden.rs`, never captured from the code
   (offsets are byte offsets into the plaintext or body):
   - T2: GOLDEN_STORE loses the record's Organisation secret, `01` ‖ `44`×32 at
     offsets 232–264 (33 bytes cut). GOLDEN_WIRE is byte-identical (the
     renamed field keeps its option tag and 32 bytes).
   - T3: GOLDEN_STORE loses the expectation's invite identifier, `ee`×32 at
     873–904 (the last 32 bytes); GOLDEN_WIRE loses `01` ‖ `ee`×32 at 317–349
     (the last 33 bytes).
   - T4: GOLDEN_WIRE becomes Organisation information: `00` ‖ Envelope (body
     bytes 0–167) ‖ snapshot (bytes 202–316: its length byte `72` and 114
     bytes, no option tag) ‖ the key, `44`×32 (bytes 169–200, no option tag);
     GOLDEN_REVOCATION is `01` ‖ the same Envelope.
   - T7: GOLDEN_STORE's record ends in its key, `44`×32, where the byte `00`
     (`org_private_key: None`) stood at offset 493.
   - T8: GOLDEN_STORE's Change-set update gains its key, `99`×32, inserted at
     offset 883, after its Change-set bytes `04 09080706`.
   The resulting strings are written out in full in each task.
7. **Test helpers** (`org-node/tests/support/mod.rs`): `private_key_held(rec)`
   (T2; the record's key as an `OrgPrivateKey`, whatever T7 does to the
   field's type), `revoke_and_tell(svc, chain, org, member, recipient, addr)`
   (T2; a removal sent to a Device the committed record lists),
   `envelope_only(envelope)` (T2; a Revocation from T4), `with_envelope`,
   `with_snapshot`, `with_key` (T4; a message of the same kind with one part
   replaced), `deliver_raw(addr, body)` (T6). The story helper `admit` loses
   its `secret` argument at T2.
8. **The app's invite identifier** (T10). org-node defines no `InviteId`
   (LLR-ms8njy), so the app defines its own
   `ods_poc_lib::invitation::InviteId([u8; 32])` with `new` and `as_bytes`,
   the same shape the app used; the Invite, the reply and the outstanding
   file are byte-identical.
9. **`revoke_and_send`** (T10). The recipient choice `revoke_member` makes
   (LLR-q225ws) moves out of the Tauri command into
   `pub async fn commands::revoke_and_send(svc, writer, rng, org_id, member,
   peer_addr)`, which the command calls, so a test can drive it with the
   substitute writer. Behaviour is unchanged.
10. **The four new variants in the app** (T10) are classified as receiver
    errors, not verdicts, by the rule LLR-7bk6qh's 2026-10-06 note states (a
    verdict is a variant `verify_envelope_against_chain` produces). This
    needs LLR-7bk6qh amended — see Open questions 1.
11. **Fuzzing.** T4 adds `fuzz_wire_decode` (`decode_body` on arbitrary
    bytes, and on bytes after an Organisation-information prefix), and
    adapts `fuzz_first_admission_base` to the snapshot argument that is no
    longer an `Option`.

---

### T1 — org-node: the four refusals of a received Wire message

**Files touched:** `org-node/src/error.rs`, `org-node/tests/value_types.rs`
**Parallel:** yes (with T2 and T3; T4 needs it)
**IDs verified:** LLR-j5vbqj (and, through it, REQ-c29s93, REQ-bwx7eg,
REQ-vxqc5g, REQ-3dsweu's typed-error clauses).
**Size:** ~60 lines.

**Step 1 — failing tests** (`org-node/tests/value_types.rs`). In
`every_rejection_variant_is_distinct_from_every_other`, replace the
annotation line `// verifies: REQ-9g6as6, REQ-bcxz96, LLR-z8fubr, LLR-mxskg9`
by `// verifies: REQ-9g6as6, REQ-bcxz96, LLR-z8fubr, LLR-mxskg9, LLR-j5vbqj`,
and after the line `        OrgNodeError::NoProvisionalUpdate,` insert:

```rust
        // A Persona already bound to an Organisation (REQ-yp75u9).
        OrgNodeError::PersonaAlreadyBound { persona_id: org_node::PersonaId::new("p".into()) },
        // The four refusals of a received Wire message (LLR-j5vbqj).
        OrgNodeError::MalformedMessage,
        OrgNodeError::OrgKeyMismatch { org_id: OrgId::new([1; 20]) },
        OrgNodeError::RevocationNotHeld { org_id: OrgId::new([1; 20]) },
        OrgNodeError::RevocationNotForThisDevice { org_id: OrgId::new([1; 20]) },
```

After `the_provisional_limit_refusal_names_the_limit`, add:

```rust
// Normal: the malformed-message refusal says what it refuses, and each of the
// other three names its Organisation. Abnormal: two Organisations' refusals
// differ, and the three refusals of one Organisation are distinct, in value
// and in message.
// verifies: LLR-j5vbqj
#[test]
fn the_received_message_refusals_say_what_they_refuse() {
    assert_eq!(OrgNodeError::MalformedMessage.to_string(), "received wire message is malformed");
    let org = OrgId::new([0xa0; 20]);
    let refusals = [
        OrgNodeError::OrgKeyMismatch { org_id: org },
        OrgNodeError::RevocationNotHeld { org_id: org },
        OrgNodeError::RevocationNotForThisDevice { org_id: org },
    ];
    for err in &refusals {
        assert!(err.to_string().contains(&format!("{org:?}")), "{err}");
    }
    assert_ne!(refusals[0], OrgNodeError::OrgKeyMismatch { org_id: OrgId::new([0xa1; 20]) });
    assert_ne!(refusals[1], refusals[2]);
    assert_ne!(refusals[0].to_string(), refusals[1].to_string());
    assert_ne!(refusals[1].to_string(), refusals[2].to_string());
}
```

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test value_types`.
Expected red: compile error E0599, `no variant or associated item named
MalformedMessage found for enum OrgNodeError` (and the three others).

**Step 2 — implementation** (`org-node/src/error.rs`). Replace the doc comment

```rust
    /// A first admission whose Organisation and invite identifier match no
    /// expectation the app declared (LLR-mxskg9, LLR-s8xp7m).
```

by

```rust
    /// A first admission whose Organisation matches no expectation the app
    /// declared (LLR-mxskg9, LLR-s8xp7m).
```

and after the `PersonaAlreadyBound { persona_id: PersonaId },` variant add:

```rust

    /// A received Wire message that does not decode — an
    /// Organisation-information message without its record snapshot or
    /// Organisation private key among them (LLR-j5vbqj, LLR-xn5pwc,
    /// REQ-c29s93).
    #[error("received wire message is malformed")]
    MalformedMessage,

    /// An Organisation-information message whose Organisation private key's
    /// public half is not the chain's Organisation public key (LLR-j5vbqj,
    /// LLR-ba2ejp, REQ-bwx7eg).
    #[error("organisation private key received for organisation {org_id:?} is not the chain's organisation key")]
    OrgKeyMismatch { org_id: OrgId },

    /// A revocation about an Organisation this node holds no record of
    /// (LLR-j5vbqj, LLR-38e2kn, REQ-vxqc5g).
    #[error("revocation for organisation {org_id:?}, of which this node holds no record")]
    RevocationNotHeld { org_id: OrgId },

    /// A revocation after whose verified Membership record this node's
    /// Device is still listed (LLR-j5vbqj, LLR-pt32fx, REQ-3dsweu).
    #[error("revocation for organisation {org_id:?} leaves this node's device in the record")]
    RevocationNotForThisDevice { org_id: OrgId },
```

**Step 3 — green.**
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test value_types`
→ `test result: ok. 9 passed; 0 failed`. Then the org-node cargo line of
**Verification** (without `--test fuzz_wire_decode`, which T4 adds) → every
target `ok`, 0 failed; the `cargo check --all-targets` and `clippy` lines of
Environment → `Finished`. App:
`CARGO_HOME=/tmp/cargo_home_fuzz cargo check --manifest-path app/src-tauri/Cargo.toml --features test-support --tests`
→ fails with E0004 (`non-exhaustive patterns: &OrgNodeError::MalformedMessage
… not covered`) in `app/src-tauri/src/events.rs` — expected until T10;
restore `app/src-tauri/Cargo.lock` if it changed.

**Step 4 — commit.** `git add org-node/src/error.rs org-node/tests/value_types.rs`,
then `git -c commit.gpgsign=false commit -m "T1: the four refusals of a received Wire message" -m "verifies: LLR-j5vbqj"`.

**Red→green attestations:**
- every_rejection_variant_is_distinct_from_every_other — red -> green: watched fail with E0599 (no variant MalformedMessage, OrgKeyMismatch, RevocationNotHeld, RevocationNotForThisDevice) before the variants existed; green after.
- the_received_message_refusals_say_what_they_refuse — red -> green: watched fail with the same E0599 before the variants existed; green after. Task done 2026-10-06, merged from worktree-org-node-org-key-pair-t1 (efecaee); org-node 229 passed, 0 failed; app crate E0004 at events.rs as expected until T10. Note: the plan's `clippy --all-targets -D warnings` expectation fails on test-file lint debt that predates this change (panic!/unwrap in fuzz_*, service_stories, store_at_rest, admission_sender, transport_handshake, key_custody, value_types); `--lib --bins` is clean, and clippy is not in verify_commands.

---

### T2 — org-node: the Organisation private key replaces the Organisation secret

**Files touched:** `org-node/src/types.rs`, `org-node/src/lib.rs`,
`org-node/src/store.rs`, `org-node/src/transport/wire.rs`,
`org-node/src/service.rs`, `org-node/tests/support/mod.rs`,
`org-node/tests/absences.rs`, `org-node/tests/admission_sender.rs`,
`org-node/tests/commit_paths.rs`, `org-node/tests/encoding_golden.rs`,
`org-node/tests/expected_admission.rs`, `org-node/tests/node_value_types.rs`,
`org-node/tests/organisation_key.rs`, `org-node/tests/persona_records.rs`,
`org-node/tests/receive_chain_reads.rs`, `org-node/tests/secret_redaction.rs`,
`org-node/tests/service_stories.rs`, `org-node/tests/store_at_rest.rs`,
`org-node/tests/transport_handshake.rs`, `org-node/tests/transport_networked.rs`,
`org-node/tests/wire_frame_bound.rs`
**Parallel:** no (serial; first of T2–T9; may run beside T1)
**IDs verified:** LLR-qsjde3, REQ-szq3ud, REQ-ju6vn2 (stored on commit),
LLR-ckk5nz (received key stored), LLR-3fwykc (every record holds the key),
LLR-s78sh7, REQ-hzm4kt, LLR-sz4xhc, LLR-bwb9pu, LLR-2dvhz8 (interim),
LLR-ayrdr8, LLR-y2v8v2, LLR-jwhzh3, LLR-wzqqg9, LLR-2xzys9, LLR-g76zqd;
reproduces PR-xwek5e and PR-szkat6 (resolved in T5 and T9).
**Size:** ~420 lines, most of it mechanical.

**Step 1 — test helpers** (`org-node/tests/support/mod.rs`).
- Replace `use org_node::{DeviceSeed, Epoch, InviteId, OrgSecret, PersonaId};`
  by `use org_node::{DeviceSeed, Epoch, InviteId, PersonaId};`.
- Delete the function `org_secret()` and its doc comment (`/// The
  Organisation secret every admission in these tests hands over.`).
- In `admit`, delete the parameter line `    secret: Option<OrgSecret>,` and
  replace
  `svc.send_update(&outcome.outgoing, joiner.device_key, Some(addr), secret, invite)`
  by `svc.send_update(&outcome.outgoing, joiner.device_key, Some(addr), invite)`.
- In `revoke`, replace
  `svc.send_update(&outcome.outgoing, recipient, addr, None, None)` by
  `svc.send_update(&outcome.outgoing, recipient, addr, None)`.
- In `admit_b_directly`, replace
  `admit(&mut s.svc_a, &s.chain, s.org_id, &s.joiner_b, b_addr, org_secret())`
  by `admit(&mut s.svc_a, &s.chain, s.org_id, &s.joiner_b, b_addr)`; in
  `captured_admission`, replace
  `admit(&mut s.svc_a, &s.chain, s.org_id, joiner, sink_addr, org_secret())`
  by `admit(&mut s.svc_a, &s.chain, s.org_id, joiner, sink_addr)`.
- After `revoke`, add:

```rust
/// Story 5, told to a Device the committed record still lists: build, write
/// the chain (mock), commit, and send the committed removal to `recipient`
/// at `addr`.
pub async fn revoke_and_tell(
    svc: &mut OrgService,
    chain: &MockChainOps,
    org_id: OrgId,
    member_id: MemberId,
    recipient: DevicePublicKey,
    addr: iroh::EndpointAddr,
) -> Result<(), OrgNodeError> {
    let update = svc.revoke_member(&mut OsRng, org_id, member_id)?;
    chain.apply_update(org_id, update.resulting_root, update.org_pub_key, rec_of(svc, org_id).epoch)?;
    let outcome = svc.commit_update(&mut OsRng, org_id).await?;
    tokio::time::timeout(NET, svc.send_update(&outcome.outgoing, recipient, Some(addr), None))
        .await
        .expect("send timed out")
}

/// The Organisation private key `rec` holds.
pub fn private_key_held(rec: &OrgRecord) -> OrgPrivateKey {
    rec.org_private_key.clone().expect("the record holds the Organisation private key")
}

/// A Wire message carrying `envelope` and nothing else.
pub fn envelope_only(envelope: org_node::Envelope) -> WireMessage {
    WireMessage { envelope, org_private_key: None, genesis_snapshot: None, invite_id: None }
}
```

**Step 2 — failing tests.**

`org-node/tests/absences.rs`, after `org_node_has_no_administrator_key`:

```rust
// The Organisation secret is gone: org-node defines, holds, takes and returns
// none, and the Organisation private key is the only Organisation key
// material it holds (LLR-qsjde3).
// verifies: LLR-qsjde3
#[test]
fn org_node_holds_no_organisation_secret() {
    assert_absent("OrgSecret", "LLR-qsjde3: no Organisation secret type");
    assert_absent("org_secret", "LLR-qsjde3: no record, message or operation holds one");
}
```

`org-node/tests/admission_sender.rs` — make these edits in this order (the
replace-all steps come last, after the tests that would otherwise be caught
by them are rewritten):

1. Replace the whole test `a_first_admission_records_the_chains_key_the_secret_and_the_member`
   (its comment block starting `// Three things a first admission writes`
   through its closing brace) by:

```rust
// Three things a first admission writes: the chain's Organisation public key,
// the Organisation private key the message carried — the one the admitting
// node's record holds (REQ-szq3ud) — and the admitted Persona's member id
// and status. A member's record holds the key since the change
// worktree-org-node-org-key-pair (LLR-3fwykc as amended); before it, only the
// creating node's did.
// *Renamed 2026-10-06 (change worktree-org-node-org-key-pair).* Was
// `a_first_admission_records_the_chains_key_the_secret_and_the_member`, after
// the Organisation secret this change removes.
// verifies: LLR-ckk5nz, LLR-e5c9ud, LLR-rys5nx, LLR-3fwykc
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_records_the_chains_key_the_private_key_and_the_member() {
    let s = admit_b_directly(setup("first-admission-fields").await).await;
    let published = s.chain.get(&s.org_id).unwrap().org_pub_key;
    let held_by_a = private_key_held(&rec_of(&s.svc_a, s.org_id));
    let rec = s.svc_b.list_orgs()[0].clone();
    assert_eq!(rec.org_pub_key, published);
    assert_eq!(private_key_held(&rec), held_by_a, "the key the message carried, the one A's record holds");

    let b_device = s.b_device_kp.device_key().unwrap();
    let b_member = rec
        .trie_members
        .iter()
        .find(|m| m.device_keys.contains(&b_device))
        .expect("B's device key is in the trie");
    let persona = persona_of(&s.svc_b, &s.pid_b);
    assert_eq!(persona.member_id, Some(b_member.id), "the Persona carries its member id");
    assert_eq!(persona.status, PersonaStatus::Active);
    assert_eq!(persona.org_id, Some(s.org_id));

    let reloaded = reopen_store("first-admission-fields", "b", "pw_b");
    assert_eq!(reloaded.data().orgs[0].org_pub_key, published);
    assert_eq!(private_key_held(&reloaded.data().orgs[0]), held_by_a);
}
```

2. Replace the whole test `pr_xwek5e_another_members_revocation_clears_the_receivers_secret`
   (its comment block starting `// PR-xwek5e — PINS A DEFECT.` through its
   closing brace) by:

```rust
// PR-xwek5e — reproduction. `revoke_member` sent no Organisation secret and
// `receive_and_verify` wrote whatever a message carried into the record, so a
// Member that received another Member's removal lost the secret while
// remaining a Member. The secret is gone: every message a node sends carries
// the Organisation private key its record holds (REQ-szq3ud), and the
// receiver stores it (REQ-ju6vn2). A removes C and tells B, still a Member.
// Red before the change (B's record held no key), green after.
// verifies: LLR-ckk5nz, REQ-szq3ud, REQ-ju6vn2, PR-xwek5e
#[tokio::test(flavor = "multi_thread")]
async fn pr_xwek5e_another_members_removal_leaves_the_receiver_holding_the_organisations_key() {
    let mut s = admit_b_directly(setup("pr-xwek5e").await).await;
    let jr_c = joiner_for_c(&mut s.svc_a);
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    let c_id = admit(&mut s.svc_a, &s.chain, s.org_id, &jr_c, b_addr).await.unwrap();
    let (svc_b, out) = b_task.await.unwrap();
    out.unwrap();

    let b_device = s.b_device_kp.device_key().unwrap();
    let (b_addr, b_task) = spawn_receive(svc_b, &s.b_device_kp).await;
    revoke_and_tell(&mut s.svc_a, &s.chain, s.org_id, c_id, b_device, b_addr).await.unwrap();
    let (svc_b, out) = b_task.await.unwrap();
    out.expect("B commits C's removal");
    let held = private_key_held(&rec_of(&svc_b, s.org_id));
    assert_eq!(held, private_key_held(&rec_of(&s.svc_a, s.org_id)), "PR-xwek5e: B holds the key A's record holds");
    assert_eq!(
        held.x25519_keypair().org_public_key().unwrap(),
        s.chain.get(&s.org_id).unwrap().org_pub_key,
        "the private half of the chain's key"
    );
    let reloaded = reopen_store("pr-xwek5e", "b", "pw_b");
    assert_eq!(private_key_held(&reloaded.data().orgs[0]), held, "and on disk");
}

// PR-szkat6 — reproduction. The Organisation private key was held only by the
// node that created the Organisation, and no admission gave it to a Member.
// Every message a node sends now carries the key its record holds
// (REQ-szq3ud), and the admitted node stores it (REQ-ju6vn2). Red before the
// change (B's record held none), green after.
// verifies: LLR-3fwykc, REQ-szq3ud, REQ-ju6vn2, PR-szkat6
#[tokio::test(flavor = "multi_thread")]
async fn pr_szkat6_an_admitted_member_holds_the_organisation_private_key() {
    let s = admit_b_directly(setup("pr-szkat6").await).await;
    let held = private_key_held(&rec_of(&s.svc_b, s.org_id));
    assert_eq!(held, private_key_held(&rec_of(&s.svc_a, s.org_id)), "the key the creating node holds");
    assert_eq!(held.x25519_keypair().org_public_key().unwrap(), s.chain.get(&s.org_id).unwrap().org_pub_key);
}
```

3. In `a_first_admission_relayed_by_another_device_is_committed`, replace
   `assert!(msg.org_secret.is_some() && msg.genesis_snapshot.is_some(), "a full admission message");`
   by `assert!(msg.org_private_key.is_some() && msg.genesis_snapshot.is_some(), "a full admission message");`.
4. In `a_message_for_an_organisation_absent_from_the_chain_is_refused`,
   replace `        org_secret: None,` by `        org_private_key: None,`.
5. In `a_revocation_reaches_the_administrators_disk`, replace the comment
   lines
   `    // LLR-8hdu9x: the Wire message carries the member snapshots as they were`
   `    // BEFORE the removal, and no Organisation secret. Added 2026-10-05 by`
   `    // review round 8: emptying the snapshot was green, and the secret was`
   `    // observed only by the PR-xwek5e pin.`
   by
   `    // LLR-8hdu9x: the Wire message carries the member snapshots as they were`
   `    // BEFORE the removal. Added 2026-10-05 by review round 8: emptying the`
   `    // snapshot was green.`
   and delete the line
   `    assert_eq!(msg.org_secret, None, "a revocation carries no Organisation secret");`.
6. In `in_loopback_mode_the_joiner_is_dialled_at_the_full_address`, replace
   `        msg.org_secret.is_some() && msg.genesis_snapshot.is_some(),` by
   `        msg.org_private_key.is_some() && msg.genesis_snapshot.is_some(),`.
7. In `a_loopback_revocation_without_an_address_burns_no_epoch`, replace
   `s.svc_a.send_update(&out.outgoing, s.joiner_b.device_key, None, None, None)`
   by `s.svc_a.send_update(&out.outgoing, s.joiner_b.device_key, None, None)`.
8. In `a_receiver_holding_two_organisations_commits_into_the_one_the_change_names`:
   replace `assert_ne!(one_before.org_secret, two_before.org_secret, "distinct secrets");`
   by `assert_ne!(one_before.org_private_key, two_before.org_private_key, "two Organisations, two keys");`;
   replace the three lines
   `        one_after.org_secret, one_before.org_secret,`
   `        "org 1's Organisation secret must not be overwritten by another Organisation's"`
   by
   `        one_after.org_private_key, one_before.org_private_key,`
   `        "org 1's Organisation private key must not be overwritten by another Organisation's"`;
   replace `assert_eq!(on_disk(s.org_1).org_secret, one_before.org_secret);` by
   `assert_eq!(on_disk(s.org_1).org_private_key, one_before.org_private_key);`.
9. In `a_self_delete_removes_only_the_organisation_the_revocation_came_from`:
   replace `assert_eq!(one_after.org_secret, one_before.org_secret, "org 1's secret must survive");`
   by `assert_eq!(one_after.org_private_key, one_before.org_private_key, "org 1's key must survive");`
   and `assert_eq!(reloaded.data().orgs[0].org_secret, one_before.org_secret);`
   by `assert_eq!(reloaded.data().orgs[0].org_private_key, one_before.org_private_key);`.
10. In `a_first_admission_binds_the_unbound_persona_not_one_bound_elsewhere`,
    replace `admit(&mut a2, &s.chain, org_2, &s.joiner_b, sink, None)` by
    `admit(&mut a2, &s.chain, org_2, &s.joiner_b, sink)`.
11. In `a_first_admission_records_the_chains_organisation_public_key`, replace
    the comment lines
    `// LLR-xq9nrq, amended 2026-10-05 (owner answer Q1): on a first admission the`
    `// record keeps the Organisation public key read from the chain in the same`
    `// operation, and no other key: no administrator key (T8 removed the field)`
    `// and no Organisation private key.`
    by
    `// LLR-xq9nrq, amended 2026-10-05 (owner answer Q1): on a first admission the`
    `// record keeps the Organisation public key read from the chain in the same`
    `// operation, and no administrator key (T8 removed the field). Since the`
    `// change worktree-org-node-org-key-pair it holds the Organisation private`
    `// key the message carried as well (REQ-ju6vn2).`
    and replace `    assert_eq!(rec.org_private_key, None, "no other key");` by
    `    assert_eq!(private_key_held(&rec), private_key_held(&rec_of(&s.svc_a, s.org_id)), "and the key A's record holds");`.
12. In `a_malformed_change_set_from_a_device_outside_the_record_is_refused_as_malformed`
    and in `the_persona_marked_active_is_the_first_whose_device_is_in_the_record`,
    replace
    `    let msg = WireMessage { envelope, org_secret: None, genesis_snapshot: None, invite_id: None };`
    by `    let msg = envelope_only(envelope);`.
13. Replace the import
    `use org_node::{DeviceSeed, Epoch, MemberSeed, OrgPublicKey, OrgSecret, PersonaId, RootHash, SequenceNumber};`
    by `use org_node::{DeviceSeed, Epoch, MemberSeed, OrgPublicKey, PersonaId, RootHash, SequenceNumber};`.
14. Replace every occurrence (`replace_all`) of `, org_secret())` by `)`; of
    `, Some(OrgSecret::from([0x11u8; 32])))` by `)`; of
    `, Some(OrgSecret::from([0x22u8; 32])))` by `)`. Then
    `grep -n "org_secret\|OrgSecret" org-node/tests/admission_sender.rs`
    → no output.

`org-node/tests/commit_paths.rs`:
- In `commit_genesis_creates_the_record_once_the_chain_carries_the_root`,
  delete `    assert_eq!(rec.org_secret, None);`.
- In `a_commit_discards_the_provisional_updates_it_orphans`, replace
  `org_secret: None, genesis_snapshot: Some(out.outgoing.record_snapshot), invite_id: None }`
  by `org_private_key: None, genesis_snapshot: Some(out.outgoing.record_snapshot), invite_id: None }`.
- Replace the whole test `send_update_sends_the_committed_update_and_writes_nothing`
  (comment through closing brace) by:

```rust
// Normal: send_update sends the committed Envelope, the earlier snapshot, the
// Organisation private key the node's record holds — taking none from its
// caller (REQ-szq3ud) — and exactly the invite identifier it is given, under
// the first bound Persona's device, to the full address in Loopback mode,
// and writes nothing.
// verifies: REQ-szq3ud, LLR-2xzys9, LLR-48jakr, LLR-jn5jeh, LLR-t4znbk, REQ-8amu2a
#[tokio::test(flavor = "multi_thread")]
async fn send_update_sends_the_committed_update_with_the_records_key_and_writes_nothing() {
    let (chain, mut a, org, joiner, _b) = founded("send").await;
    let update = a.admit_member(&mut OsRng, org, &joiner).unwrap();
    chain.apply_update(org, update.resulting_root, update.org_pub_key, Epoch::new(1)).unwrap();
    let out = a.commit_update(&mut OsRng, org).await.unwrap();
    let on_disk = store_bytes("send", "a");
    let invite = InviteId::new([0x6e; 32]);
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    a.send_update(&out.outgoing, joiner.device_key, Some(sink_addr), Some(invite)).await.unwrap();
    let (_ep, sender, msg) = sink.await.unwrap();
    assert_eq!(
        msg,
        WireMessage {
            envelope: out.outgoing.envelope.clone(),
            org_private_key: Some(private_key_held(&rec_of(&a, org))),
            genesis_snapshot: Some(out.outgoing.record_snapshot.clone()),
            invite_id: Some(invite),
        }
    );
    let first_bound = a.list_personas().iter().find(|p| p.org_id == Some(org)).unwrap().clone();
    assert_eq!(sender, first_bound.device_seed.signing_keypair().device_key().unwrap());
    assert_eq!(store_bytes("send", "a"), on_disk, "a send writes nothing");
    // Abnormal half of LLR-48jakr: no identifier passed, none sent.
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    a.send_update(&out.outgoing, joiner.device_key, Some(sink_addr), None).await.unwrap();
    assert_eq!(sink.await.unwrap().2.invite_id, None);
}
```

- In `send_update_refuses_without_a_bound_persona_or_a_loopback_address`,
  replace `Some(dead_addr([0x6f; 32])), None, None).await.is_err()` by
  `Some(dead_addr([0x6f; 32])), None).await.is_err()` and
  `a.send_update(&out.outgoing, joiner.device_key, None, None, None)` by
  `a.send_update(&out.outgoing, joiner.device_key, None, None)`.
- Delete the whole test `a_nodes_own_commit_keeps_its_secret` (its comment
  through its closing brace): LLR-ckk5nz no longer states that an own commit
  keeps the stored value; T8 writes the test of what it does
  (`commit_update_commits_a_provisional_update_the_chain_carries`, LLR-6s785x).
- In `the_revoking_record_takes_the_removal_only_through_commit_update`,
  replace `Some(dead_addr([0x7f; 32])), None, None).await.is_err()` by
  `Some(dead_addr([0x7f; 32])), None).await.is_err()`.
- In `a_commit_that_keeps_one_of_our_personas_is_an_update`, replace
  `admit(&mut s.svc_a, &s.chain, s.org_id, &joiner_c, sink, None)` by
  `admit(&mut s.svc_a, &s.chain, s.org_id, &joiner_c, sink)`.

`org-node/tests/encoding_golden.rs`:
- Above `const GOLDEN_STORE`, after the T8 paragraph, add:

```rust
//
// Re-pinned 2026-10-06 (change worktree-org-node-org-key-pair, T2): the
// Organisation secret removed from the record (`01` ‖ `44`x32 cut at offsets
// 232–264). Derived by the format rule from the value above, never captured
// from the code. GOLDEN_WIRE is unchanged: the Wire message's
// `org_private_key` holds the same option tag and 32 bytes the secret did.
```

  (The value above it is the current `GOLDEN_STORE`; copy that string into a
  new `// GOLDEN_STORE  = "…"` comment line before this paragraph, as the
  earlier re-pins did.)
- Replace the `GOLDEN_STORE` value by:

```
0107702d616c69636501aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa05616c69636505416c69636505536d697468111111111111111111111111111111111111111111111111111111111111111112121212121212121212121212121212121212121212121212121212121212120101010101010101010101010101010101010101010101010101010101010101010101aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa3333333333333333333333333333333333333333333333333333333333333333d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737030202010101010101010101010101010101010101010101010101010101010101010105616c69636505416c69636505536d697468d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21020202020202020202020202020202020202020202020202020202020202020203626f6203426f62054a6f6e6573884b8857f4eaa1613c61504db34d4beaf346517a0e31de3cddd4d9b4201d9d0b01a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f001555555555555555555555555555555555555555555555555555555555555555500020007702d616c69636500777777777777777777777777777777777777777777777777777777777777777701d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c97787370001010101010101010101010101010101010101010101010101010101010101010105616c69636505416c69636505536d697468d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21888888888888888888888888888888888888888888888888888888888888888801bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb07702d616c696365013333333333333333333333333333333333333333333333333333333333333333666666666666666666666666666666666666666666666666666666666666666603d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701040908070601cccccccccccccccccccccccccccccccccccccccceeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee
```

- In `persona_store_plaintext_is_pinned`, replace
  `    assert_eq!(org["org_secret"][31], json!(0x44));` by
  `    assert!(org.get("org_secret").is_none(), "no Organisation secret (LLR-qsjde3)");`.
- In `admission_wire_message_is_pinned`, replace
  `    assert_eq!(m["org_secret"][31], json!(0x44));` by
  `    assert_eq!(m["org_private_key"][31], json!(0x44));`.

`org-node/tests/expected_admission.rs`: in
`a_first_admission_that_lists_none_of_our_personas_is_refused`, replace
`admit(&mut s.svc_a, &s.chain, s.org_id, &joiner_b2, addr, org_secret())` by
`admit(&mut s.svc_a, &s.chain, s.org_id, &joiner_b2, addr)`.

`org-node/tests/node_value_types.rs`:
- In the `use org_node::{…}` list, delete `OrgSecret, `.
- In `secret_types_redact_debug_and_give_bytes_only_through_the_accessor`,
  delete the lines `    let org = OrgSecret::from(bytes);`,
  `    assert_eq!(org.expose_secret(), &bytes);`,
  `    assert_eq!(format!("{org:?}"), "OrgSecret([REDACTED])");`,
  `    assert_ne!(OrgSecret::from([1u8; 32]), org);` and
  `    assert!(!implements_display!(OrgSecret) && !implements_copy!(OrgSecret));`,
  and add after `    assert_eq!(member.clone(), member);`:
  `    assert_ne!(MemberSeed::from([1u8; 32]), member);`.
- In `secret_debug_is_the_same_whatever_the_bytes`, delete
  `        assert_eq!(format!("{:?}", Some(OrgSecret::from(bytes))), "Some(OrgSecret([REDACTED]))");`
  and add in its place
  `        assert_eq!(format!("{:?}", Some(DeviceSeed::from(bytes))), "Some(DeviceSeed([REDACTED]))");`.
- In `secret_types_serialise_as_the_plain_bytes`, replace the four lines from
  `    assert_eq!(plain(&OrgSecret::from(bytes)), plain(&bytes));` to
  `    assert!(postcard::from_bytes::<OrgSecret>(&plain(&bytes)[..31]).is_err());`
  by:

```rust
    let back: MemberSeed = postcard::from_bytes(&plain(&bytes)).unwrap();
    assert_eq!(back, MemberSeed::from(bytes));
    // Abnormal: 31 bytes are not a secret.
    assert!(postcard::from_bytes::<MemberSeed>(&plain(&bytes)[..31]).is_err());
```

  (The three tests keep `// verifies: LLR-sz4xhc`: the item names
  `MemberSeed` and `DeviceSeed` alone since its amendment.)

`org-node/tests/organisation_key.rs`: in the `use support::{…}` list delete
`org_secret, `; replace
`admit(&mut svc_a, &chain, org_id, &joiner, b_addr, org_secret()).await.expect("admit B");`
by `admit(&mut svc_a, &chain, org_id, &joiner, b_addr).await.expect("admit B");`;
replace the comment line
`// "Organisation secret" is org-node's term for \`org_secret\`, not this key.)`
by `// "Organisation secret" was org-node's term for another value, removed since.)`.

`org-node/tests/persona_records.rs`: in the `use org_node::{…}` list delete
`OrgSecret, `; in `org_with_member` delete `        org_secret: None,`; in
`struct WireOrg` delete `    org_secret: Option<OrgSecret>,`; in
`wire_store` delete `            org_secret: None,`.

`org-node/tests/receive_chain_reads.rs`: replace each of the three
`deliver(b_addr, &WireMessage { envelope, org_secret: None, genesis_snapshot: None, invite_id: None }).await;`
by `deliver(b_addr, &envelope_only(envelope)).await;`, and delete
`use org_node::transport::wire::WireMessage;` only if
`grep -n "WireMessage" org-node/tests/receive_chain_reads.rs` then shows no
other use (it does: the `garbled` message in
`a_held_organisation_absent_from_the_chain_is_refused_after_the_chain_free_checks`
— so keep the import).

`org-node/tests/secret_redaction.rs`:
- Module doc: replace `a member seed, a device seed or an Organisation secret`
  by `a member seed, a device seed or an Organisation private key`.
- Replace the import list entry `DeviceSeed, Envelope, Epoch, MemberSeed, OrgPrivateKey, OrgSecret, PersonaId, SequenceNumber,`
  by `DeviceSeed, Envelope, Epoch, MemberSeed, OrgPrivateKey, PersonaId, SequenceNumber,`.
- Replace `fn org(…)` and `fn wire(…)` by:

```rust
fn org(private: [u8; 32]) -> OrgRecord {
    OrgRecord {
        org_id: OrgId::new([5u8; 20]),
        root_hash: RootHash::new([0x11u8; 32]),
        org_pub_key: org_public_key(),
        epoch: Epoch::new(1),
        last_seq: SequenceNumber::new(0),
        trie_members: vec![],
        proxy_account: None,
        org_private_key: Some(OrgPrivateKey::from(private)),
    }
}

fn wire(private: [u8; 32]) -> WireMessage {
    let admin = MemberSeed::from([1u8; 32]).x25519_keypair();
    let (delta, _) = admit_member_delta(&admin);
    let envelope = Envelope::build(OrgId::new([5u8; 20]), SequenceNumber::new(1), &delta).unwrap();
    WireMessage { envelope, org_private_key: Some(OrgPrivateKey::from(private)), genesis_snapshot: None, invite_id: None }
}
```

- Replace the body of `records_and_wire_messages_never_render_secret_bytes`
  (keep its comments and `/// verifies: LLR-bwb9pu, LLR-2dvhz8`) by:

```rust
    let (member, device, private) = (sentinel(0xd0), sentinel(0x10), org_private());
    let p = persona(member, device);
    let o = org(private);
    let data = StoreData {
        personas: vec![p.clone()],
        orgs: vec![o.clone()],
        provisional_updates: vec![],
        expected_admissions: vec![],
    };
    let w = wire(private);
    for rendered in [format!("{p:?}"), format!("{p:#?}"), format!("{data:?}"), format!("{data:#?}")] {
        assert_not_rendered(&rendered, &member, "member seed");
        assert_not_rendered(&rendered, &device, "device seed");
        assert!(rendered.contains("MemberSeed([REDACTED])") && rendered.contains("DeviceSeed([REDACTED])"));
    }
    for rendered in [format!("{o:?}"), format!("{o:#?}"), format!("{data:?}"), format!("{data:#?}"), format!("{w:?}"), format!("{w:#?}")] {
        assert_not_rendered(&rendered, &private, "Organisation private key");
        assert!(rendered.contains("OrgPrivateKey([REDACTED])"), "{rendered}");
    }
    for rendered in [format!("{o:?}"), format!("{o:#?}"), format!("{data:?}"), format!("{data:#?}")] {
        assert!(rendered.contains("org_private_key"));
    }
```

- In `a_record_debug_says_whether_the_organisation_private_key_is_set`,
  replace `    let mut member_record = org(None);` by
  `    let mut member_record = org(org_private());` and
  `    let creator_record = org(None);` by
  `    let creator_record = org(org_private());`.
- In `secrets_at_the_high_byte_bound_and_many_records_stay_unrendered`,
  replace `        orgs: vec![org(Some(sentinel_down(0xdf))), org(None)],` by
  `        orgs: vec![org(sentinel_down(0xdf)), org(sentinel_down(0xcf))],`, and
  replace the two lines
  `        assert_not_rendered(&rendered, &sentinel_down(0xdf), "Organisation secret");`
  `        assert!(rendered.contains("org_secret: None"), "an absent secret still renders as None");`
  by
  `        assert_not_rendered(&rendered, &sentinel_down(0xdf), "Organisation private key");`
  `        assert_not_rendered(&rendered, &sentinel_down(0xcf), "Organisation private key");`.

`org-node/tests/service_stories.rs`: in the `use support::{…}` list replace
`org_secret, ` by `envelope_only, private_key_held, `; replace every occurrence
of `, org_secret())` by `)`; replace
`    assert_eq!(svc_b3.list_orgs()[0].org_secret, org_secret());` by
`    assert_eq!(private_key_held(&svc_b3.list_orgs()[0]), private_key_held(&svc_a.list_orgs()[0]));`;
in `revocation_from_an_unknown_device_leaves_the_record_in_place` replace
`    let msg = WireMessage { envelope, org_secret: None, genesis_snapshot: None, invite_id: None };`
by `    let msg = envelope_only(envelope);` and delete
`    use org_node::transport::wire::WireMessage;` from that function's `use`
lines.

`org-node/tests/store_at_rest.rs`:
- Module doc: replace `neither a persona secret nor an Organisation secret`
  by `neither a persona secret nor the Organisation private key`.
- Import: replace `use org_node::{DeviceSeed, Epoch, MemberSeed, OrgPrivateKey, OrgSecret, PersonaId, SequenceNumber};`
  by `use org_node::{DeviceSeed, Epoch, MemberSeed, OrgPrivateKey, PersonaId, SequenceNumber};`.
- Replace `org_record` and `organisation_secret_does_not_appear_in_the_file` by:

```rust
/// An org record holding the Organisation private key `key`, with the
/// remaining fields fixed.
fn org_record(key: [u8; 32]) -> OrgRecord {
    OrgRecord {
        org_id: OrgId::new([5u8; 20]),
        root_hash: RootHash::new([0x11u8; 32]),
        org_pub_key: OrgPrivateKey::from(key).x25519_keypair().org_public_key().unwrap(),
        epoch: Epoch::new(3),
        last_seq: SequenceNumber::new(2),
        trie_members: Vec::new(),
        proxy_account: None,
        org_private_key: Some(OrgPrivateKey::from(key)),
    }
}
```

```rust
// *Renamed 2026-10-06 (change worktree-org-node-org-key-pair).* Was
// `organisation_secret_does_not_appear_in_the_file`; LLR-s78sh7 names the
// Organisation private key in the secret's place.
// verifies: REQ-hzm4kt, LLR-s78sh7
#[test]
fn organisation_private_key_does_not_appear_in_the_file() {
    let path = tmp_path("orgkey");
    let mut s = PersonaStore::open(path.clone(), "pw").unwrap();
    let key = [0x7eu8; 32];
    s.data_mut().orgs.push(org_record(key));
    s.save(&mut OsRng).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert!(!contains(&bytes, &key), "Organisation private key in clear");
    let _ = std::fs::remove_file(&path);
}
```

`org-node/tests/transport_handshake.rs`: replace
`use org_node::{DeviceSeed, MemberSeed, OrgSecret};` by
`use org_node::{DeviceSeed, MemberSeed, OrgPrivateKey};`; replace
`org_secret: Some(OrgSecret::from([0xab; 32]))` by
`org_private_key: Some(OrgPrivateKey::from([0xab; 32]))`; in
`the_length_prefix_is_not_checked_against_the_body` replace
`WireMessage { envelope: env, org_secret: None, genesis_snapshot: None, invite_id: None }`
by `WireMessage { envelope: env, org_private_key: None, genesis_snapshot: None, invite_id: None }`.

`org-node/tests/transport_networked.rs`: in
`use org_node::{DeviceSeed, Epoch, MemberSeed, OrgPrivateKey, OrgSecret, SequenceNumber};`
delete `OrgSecret, `; replace `        org_secret: Some(OrgSecret::from([0xab; 32])),`
by `        org_private_key: Some(OrgPrivateKey::from([0xab; 32])),`.

`org-node/tests/wire_frame_bound.rs`: replace
`use org_node::{Envelope, MemberSeed, OrgSecret, SequenceNumber};` by
`use org_node::{Envelope, MemberSeed, OrgPrivateKey, SequenceNumber};` and
`org_secret: Some(OrgSecret::from([9u8; 32]))` by
`org_private_key: Some(OrgPrivateKey::from([9u8; 32]))`.

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test absences --test admission_sender --test commit_paths`.
Expected red: compile errors — E0560 (`struct WireMessage has no field named
org_private_key`), E0061 (`send_update` takes 5 arguments, 4 supplied). The
absence test alone, run as `--test absences` against the unchanged source,
fails at run time: `…/src/types.rs contains \`OrgSecret\``.

**Step 3 — implementation.**

`org-node/src/types.rs`: delete

```rust
secret_type!(
    /// The Organisation secret handed to a Member at admission.
    OrgSecret
);
```

and replace the doc of `OrgPrivateKey`'s `secret_type!` block by:

```rust
    /// The Organisation's X25519 private key: one per epoch, shared by every
    /// Member of the Organisation (REQ-ech45n, REQ-szq3ud, LLR-3fwykc). Its
    /// in-memory key pair is `keys::X25519Keypair` (LLR-98ufry).
```

`org-node/src/lib.rs`: in `pub use types::{…}` delete `OrgSecret, `.

`org-node/src/store.rs`:
- In `use crate::types::{…}` delete `OrgSecret, `.
- In `pub struct OrgRecord`, delete `    pub org_secret: Option<OrgSecret>,` and
  replace the doc of `org_private_key` by:

```rust
    /// The Organisation's X25519 private key (REQ-ech45n, REQ-szq3ud): the
    /// creating node's from its genesis update, every other node's from the
    /// Wire message that admitted it (LLR-3fwykc, LLR-ckk5nz). It renders
    /// under `Debug` as its redacted secret type (LLR-2dvhz8).
```

- In `pub(crate) struct RawOrgRecord`, delete `    org_secret: Option<OrgSecret>,`;
  in `impl TryFrom<RawOrgRecord> for OrgRecord`, delete
  `            org_secret: raw.org_secret,`.

`org-node/src/transport/wire.rs`: replace `use crate::types::{InviteId, OrgSecret};`
by `use crate::types::{InviteId, OrgPrivateKey};`; replace the struct's doc
comment line `/// plus the Organisation secret the sender chose to hand over, if any.`
by `/// plus the Organisation private key the sending node's record holds (REQ-szq3ud).`;
replace

```rust
    /// The Organisation secret, redacted in `Debug`.
    pub org_secret: Option<OrgSecret>,
```

by

```rust
    /// The Organisation private key the sending node's record holds,
    /// redacted in `Debug` (LLR-322xfu).
    pub org_private_key: Option<OrgPrivateKey>,
```

`org-node/src/service.rs`:
- Replace
  `use crate::types::{ChainAccount, Epoch, InviteId, OrgPublicKey, OrgSecret, PersonaId, SequenceNumber};`
  by
  `use crate::types::{ChainAccount, Epoch, InviteId, OrgPrivateKey, OrgPublicKey, PersonaId, SequenceNumber};`.
- In `commit_genesis`, delete `            org_secret: None,`.
- In `commit_update`, replace `            self.commit_held(org_id, &verified, None)?;`
  by `            self.commit_held(org_id, &verified)?;`.
- Replace the whole of `fn commit_held` (doc comment through closing brace)
  by:

```rust
    /// Write a verified update to a held record — root, epoch, mark and
    /// members together (LLR-cja9zv) — and drop the provisional updates it
    /// orphans (LLR-mkj4bz). It takes no key (LLR-ckk5nz): a received key is
    /// written by `store_carried_key`.
    fn commit_held(&mut self, org_id: OrgId, verified: &VerifiedUpdate) -> Result<(), OrgNodeError> {
        let root = verified.trie.root_hash().map_err(OrgNodeError::Trie)?;
        let snapshots: Vec<MemberSnapshot> = verified.trie.members().iter().map(snapshot_of).collect();
        let data = self.store.data_mut();
        let rec = data.orgs.iter_mut().find(|o| o.org_id == org_id).ok_or(OrgNodeError::OrgNotOnChain)?;
        rec.root_hash = root;
        rec.epoch = verified.epoch;
        rec.last_seq = verified.seq_guard.last_seen();
        rec.trie_members = snapshots;
        Self::discard_orphans(data, org_id, root);
        Ok(())
    }

    /// The record of `org_id` holds the Organisation private key a committed
    /// received update carried (REQ-ju6vn2, LLR-ckk5nz).
    fn store_carried_key(data: &mut StoreData, org_id: OrgId, key: Option<OrgPrivateKey>) -> Result<(), OrgNodeError> {
        let rec = data.orgs.iter_mut().find(|o| o.org_id == org_id).ok_or(OrgNodeError::OrgNotOnChain)?;
        rec.org_private_key = key;
        Ok(())
    }
```

- Replace `send_update`'s doc comment, signature and body down to and
  including the `let msg = WireMessage { … };` statement by:

```rust
    /// Send a committed update to `recipient`'s device, from the device of
    /// the first Persona bound to its Organisation (LLR-2xzys9), carrying the
    /// Organisation private key the node's record holds — none taken from
    /// the caller (REQ-szq3ud) — and exactly the invite identifier given
    /// (LLR-48jakr). Loopback dials `peer_addr` and refuses without one,
    /// before binding (LLR-jn5jeh, LLR-pw369n); Networked dials by
    /// `recipient`. Writes nothing (LLR-t4znbk).
    pub async fn send_update(
        &mut self,
        outgoing: &OutgoingUpdate,
        recipient: DevicePublicKey,
        peer_addr: Option<iroh::EndpointAddr>,
        invite_id: Option<InviteId>,
    ) -> Result<(), OrgNodeError> {
        let mode = self.transport_mode;
        let org_private_key = self.find_org(outgoing.envelope.org_id)?.org_private_key.clone();
        let persona_id = self.first_persona_bound_to(outgoing.envelope.org_id)?.persona_id.clone();
        let loopback_addr = match (mode, peer_addr) {
            (TransportMode::Loopback, None) => {
                return Err(OrgNodeError::Chain("Loopback send requires the peer's EndpointAddr".into()));
            }
            (TransportMode::Loopback, Some(addr)) => Some(addr),
            (TransportMode::Networked, _) => None,
        };
        let msg = WireMessage {
            envelope: outgoing.envelope.clone(),
            org_private_key,
            genesis_snapshot: Some(outgoing.record_snapshot.clone()),
            invite_id,
        };
```

- In `receive_and_verify`, in the `data.orgs.push(OrgRecord { … })` of the
  first admission, delete `                org_secret: msg.org_secret,` and
  replace `                org_private_key: None,` by
  `                org_private_key: msg.org_private_key.clone(),`; replace

```rust
            // A received update replaces the stored secret (LLR-ckk5nz).
            self.commit_held(org_id, &verified, Some(msg.org_secret))?;
```

  by

```rust
            // A committed received update stores the key it carried
            // (REQ-ju6vn2, LLR-ckk5nz).
            self.commit_held(org_id, &verified)?;
            Self::store_carried_key(self.store.data_mut(), org_id, msg.org_private_key.clone())?;
```

- In `receive_and_self_delete_if_revoked`, replace

```rust
            // Regular admit/update — commit the update normally; this path
            // never touched the secret.
            self.commit_held(org_id, &verified, None)?;
```

  by

```rust
            // Regular admit/update — commit the update normally; this path
            // keeps the key the record holds.
            self.commit_held(org_id, &verified)?;
```

- Run `grep -n "org_secret\|OrgSecret" org-node/src -r` → no output.

**Step 4 — green.**
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test absences --test admission_sender --test commit_paths`
→ absences `5 passed`, admission_sender `47 passed`, commit_paths `28 passed`,
0 failed. Then the full org-node cargo line → every target `ok`, 0 failed
(encoding_golden 3, node_value_types 12, secret_redaction 8, store_at_rest 7,
persona_records 15, service_stories 3); `cargo check --all-targets` and
`clippy` → `Finished`. App: still fails (E0004 from T1 if merged, and E0432
`unresolved import org_node::OrgSecret` in `commands.rs`, `invitation.rs`,
`submit.rs`) — expected until T10.

**Step 5 — commit.** `git add` each file of **Files touched** (one
`git add` call with the paths), then
`git -c commit.gpgsign=false commit -m "T2: the Organisation private key replaces the Organisation secret" -m "verifies: LLR-qsjde3, REQ-szq3ud, REQ-ju6vn2, LLR-ckk5nz, LLR-3fwykc, LLR-s78sh7, LLR-sz4xhc, LLR-bwb9pu, LLR-ayrdr8; reproduces PR-xwek5e, PR-szkat6"`.

**Red→green attestations:**
- org_node_holds_no_organisation_secret — red -> green: failed at run time on the unchanged source ("types.rs contains `OrgSecret`: LLR-qsjde3"); green after Step 3.
- pr_xwek5e_another_members_removal_leaves_the_receiver_holding_the_organisations_key — red -> green: written against the old API and run on the unchanged source; failed at run time in private_key_held (support/mod.rs:303, B's record held no key); green after Step 3.
- pr_szkat6_an_admitted_member_holds_the_organisation_private_key — red -> green: same run-time red on the unchanged source (B's record held no key); green after Step 3.
- a_first_admission_records_the_chains_key_the_private_key_and_the_member (rewritten, renamed) — red -> green: same run-time red (B's record held no key); green after Step 3.
- a_first_admission_records_the_chains_organisation_public_key (re-asserted) — red -> green: same run-time red (B's record held no key); green after Step 3.
- send_update_sends_the_committed_update_with_the_records_key_and_writes_nothing (rewritten, renamed) — red -> green: compile-time red (E0560 no field `org_private_key` on `WireMessage`, E0061 `send_update` arity); green after Step 3.
- organisation_private_key_does_not_appear_in_the_file (rewritten, renamed) — red -> green: compile-time red (`OrgRecord` still had `org_secret`); green after Step 3.
- records_and_wire_messages_never_render_secret_bytes, secrets_at_the_high_byte_bound_and_many_records_stay_unrendered (re-asserted) — red -> green: compile-time red (E0560 on `WireMessage.org_private_key`, `OrgRecord` missing `org_secret`); green after Step 3.
- persona_store_plaintext_is_pinned, admission_wire_message_is_pinned (re-pinned) — red -> green: compile-time red with the suite (E0061/E0560 in support); green after Step 3 against the re-pinned GOLDEN_STORE. The GOLDEN_STORE literal above has one `01` byte too many (906 bytes, the member id written as 33); the pinned value is the 905-byte one derived by the plan's rule (T8's value with bytes 232–264 cut).
- the two-Organisation tests in admission_sender (re-asserted on the key) — red -> green: compile-time red (E0061 `send_update` arity, E0609 no field `org_private_key`); green after Step 3. Task done 2026-10-06, merged from worktree-org-node-org-key-pair-t2 (362c5ae); org-node 229 passed, 0 failed. Slip fixed in execution: transport_handshake.rs already imported `OrgPrivateKey` (line 17), so line 9 imports `{DeviceSeed, MemberSeed}`. LLR-8hdu9x stays verified by a_revocation_reaches_the_administrators_disk.

---

### T3 — org-node: the invite identifier leaves org-node; an expectation names the Organisation alone

**Files touched:** `org-node/src/types.rs`, `org-node/src/lib.rs`,
`org-node/src/store.rs`, `org-node/src/transport/wire.rs`,
`org-node/src/service.rs`, `org-node/tests/support/mod.rs`,
`org-node/tests/absences.rs`, `org-node/tests/admission_sender.rs`,
`org-node/tests/commit_paths.rs`, `org-node/tests/encoding_golden.rs`,
`org-node/tests/expected_admission.rs`, `org-node/tests/persona_records.rs`,
`org-node/tests/provisional_store.rs`, `org-node/tests/secret_redaction.rs`,
`org-node/tests/transport_handshake.rs`, `org-node/tests/transport_networked.rs`,
`org-node/tests/wire_frame_bound.rs`
**Parallel:** no (serial, after T2; may run beside T1)
**IDs verified:** REQ-8amu2a, LLR-ms8njy, LLR-48jakr, LLR-9zfnmb,
LLR-s8xp7m, LLR-q8emds, LLR-mbjfq8, LLR-95753m (expectation clause),
LLR-ayrdr8, LLR-y2v8v2.
**Size:** ~230 lines.

**Step 1 — test helpers** (`org-node/tests/support/mod.rs`).
- Replace `use org_node::{DeviceSeed, Epoch, InviteId, PersonaId};` by
  `use org_node::{DeviceSeed, Epoch, PersonaId};`.
- Delete `invite_for` and its doc comment (`/// The invite identifier a
  story's joiner replies under: …`).
- Replace `prepare_to_join`'s body line
  `    svc_b.expect_admission(&mut OsRng, org_id, invite_for(&first_device(svc_b))).expect("expect the admission");`
  by `    svc_b.expect_admission(&mut OsRng, org_id).expect("expect the admission");`.
- In `admit`: replace its doc's last two lines
  `/// commit, send to \`addr\` under the joiner's invite identifier. Returns the`
  `/// joiner's new MemberId.` by
  `/// commit, send to \`addr\`. Returns the joiner's new MemberId.`; delete
  `    let invite = Some(invite_for(&joiner.device_key));`; replace
  `svc.send_update(&outcome.outgoing, joiner.device_key, Some(addr), invite)`
  by `svc.send_update(&outcome.outgoing, joiner.device_key, Some(addr))`.
- In `revoke`, replace `svc.send_update(&outcome.outgoing, recipient, addr, None)`
  by `svc.send_update(&outcome.outgoing, recipient, addr)`; in
  `revoke_and_tell`, replace
  `svc.send_update(&outcome.outgoing, recipient, Some(addr), None)` by
  `svc.send_update(&outcome.outgoing, recipient, Some(addr))`.
- In `envelope_only`, replace
  `    WireMessage { envelope, org_private_key: None, genesis_snapshot: None, invite_id: None }`
  by `    WireMessage { envelope, org_private_key: None, genesis_snapshot: None }`.

**Step 2 — failing tests.**

`org-node/tests/absences.rs`, after `org_node_holds_no_organisation_secret`:

```rust
// The invite identifier never travels between peers: org-node defines no type
// for it, no Wire message or expectation holds one, and `send_update` takes
// none (REQ-8amu2a as amended).
// verifies: LLR-ms8njy, LLR-48jakr
#[test]
fn org_node_holds_no_invite_identifier() {
    assert_absent("InviteId", "LLR-ms8njy: no invite identifier type");
    assert_absent("invite_id", "LLR-ms8njy, LLR-48jakr: no field or argument carries one");
}
```

`org-node/tests/expected_admission.rs`:
- Module doc: replace `against the chain only when its Organisation and invite identifier match an`
  `//! expectation the app declared, and committed only when the verified record`
  by `against the chain only when its Organisation matches an expectation the`
  `//! app declared, and committed only when the verified record`.
- Replace `use org_node::{InviteId, Joiner, MemberSeed};` by
  `use org_node::{Joiner, MemberSeed};`, and
  `fn expectation(org_id: OrgId, invite_id: InviteId) -> ExpectedAdmission { ExpectedAdmission { org_id, invite_id } }`
  (its three lines) by:

```rust
fn expectation(org_id: OrgId) -> ExpectedAdmission {
    ExpectedAdmission { org_id }
}
```

- In `an_expected_first_admission_is_committed_and_clears_the_expectation`,
  delete `    let id = invite_for(&s.joiner_b.device_key);` and replace
  `&[expectation(s.org_id, id)]` by `&[expectation(s.org_id)]`.
- In `an_expectation_for_one_organisation_admits_no_other`, delete
  `    let id = invite_for(&joiner.device_key);`; replace
  `svc_b2.expect_admission(&mut OsRng, elsewhere, id).unwrap();` by
  `svc_b2.expect_admission(&mut OsRng, elsewhere).unwrap();` and
  `&[expectation(elsewhere, id)]` by `&[expectation(elsewhere)]`.
- Delete the whole test
  `a_first_admission_under_another_invite_id_is_refused_and_keeps_the_expectation`
  (comment through closing brace): there is no invite identifier to mismatch;
  pre-emption is REQ-kt877x's, verified by
  `a_first_admission_that_lists_none_of_our_personas_is_refused`.
- In `a_first_admission_that_lists_none_of_our_personas_is_refused`: replace
  its comment line `// relayed under this node's invite identifier, verifies against the chain`
  by `// delivered to a node that expects an admission to that Organisation, verifies against the chain`;
  delete `    let id_b2 = invite_for(&joiner_b2.device_key);`; replace
  `svc_b2.expect_admission(&mut OsRng, s.org_id, id_b2).unwrap();` by
  `svc_b2.expect_admission(&mut OsRng, s.org_id).unwrap();`; replace
  `    // A admits B (epoch 2); the relay re-labels B's admission with B2's id.`
  by `    // A admits B (epoch 2); a relay hands B's admission to B2.`; replace
  `deliver(addr, &WireMessage { invite_id: Some(id_b2), ..of_b }).await;` by
  `deliver(addr, &of_b).await;`; replace
  `&[expectation(s.org_id, id_b2)]` by `&[expectation(s.org_id)]`.
- Replace the whole test `expect_admission_records_a_pair_once_and_reaches_the_disk`
  (comment through closing brace) by:

```rust
// Normal and abnormal: a declaration is recorded once however often it is
// made, another Organisation is a second entry, and each reaches the disk
// before `expect_admission` returns.
// *Renamed 2026-10-06 (change worktree-org-node-org-key-pair).* Was
// `expect_admission_records_a_pair_once_and_reaches_the_disk`: an expectation
// named an invite identifier too.
// verifies: REQ-8amu2a, LLR-9zfnmb, LLR-95753m
#[test]
fn expect_admission_records_an_organisation_once_and_reaches_the_disk() {
    let mut svc = OrgService::new(open_store("once", "b", "pw_b"), Box::new(MockChainOps::new()));
    let (org, other) = (OrgId::new([0x42; 20]), OrgId::new([0x43; 20]));
    svc.expect_admission(&mut OsRng, org).unwrap();
    svc.expect_admission(&mut OsRng, org).unwrap();
    assert_eq!(svc.expected_admissions(), &[expectation(org)]);
    assert_eq!(reopen_store("once", "b", "pw_b").data().expected_admissions, vec![expectation(org)]);
    svc.expect_admission(&mut OsRng, other).unwrap();
    assert_eq!(
        reopen_store("once", "b", "pw_b").data().expected_admissions,
        vec![expectation(org), expectation(other)]
    );
}
```

- In `a_refused_first_admission_leaves_the_expectation`, replace
  `&[expectation(s.org_id, invite_for(&joiner.device_key))]` by
  `&[expectation(s.org_id)]`.
- Replace the whole test `a_second_expectation_for_the_same_organisation_survives_the_commit`
  by:

```rust
// Normal: the commit clears only the expectation for the Organisation it
// admitted to; one for another Organisation stays. (At most one is held per
// Organisation, LLR-9zfnmb.)
// *Rewritten 2026-10-06 (change worktree-org-node-org-key-pair).* Was
// `a_second_expectation_for_the_same_organisation_survives_the_commit`, of a
// second invite identifier for the same Organisation.
// verifies: REQ-8amu2a, LLR-q8emds
#[tokio::test(flavor = "multi_thread")]
async fn an_expectation_for_another_organisation_survives_the_commit() {
    let mut s = setup("two-orgs-expected").await;
    let elsewhere = OrgId::new([0x77; 20]);
    s.svc_b.expect_admission(&mut OsRng, elsewhere).unwrap();
    let s = admit_b_directly(s).await;
    assert_eq!(s.svc_b.expected_admissions(), &[expectation(elsewhere)]);
}
```

- In `an_expected_first_admission_decodes_its_snapshot_before_reading_the_chain`,
  replace `&[expectation(s.org_id, invite_for(&joiner.device_key))]` by
  `&[expectation(s.org_id)]`.

`org-node/tests/admission_sender.rs`:
1. In `a_message_for_an_organisation_absent_from_the_chain_is_refused`, replace
   the two lines
   `    let absent_id = org_node::InviteId::new([0x31; 32]);`
   `    s.svc_b.expect_admission(&mut OsRng, OrgId::new([0xeeu8; 20]), absent_id).unwrap();`
   by `    s.svc_b.expect_admission(&mut OsRng, OrgId::new([0xeeu8; 20])).unwrap();`,
   and delete `        invite_id: Some(absent_id),`.
2. In `a_second_organisation_is_admitted_into_without_touching_the_first`,
   replace `svc_b.expect_admission(&mut OsRng, org_2, invite_for(&jr_b.device_key)).unwrap();`
   by `svc_b.expect_admission(&mut OsRng, org_2).unwrap();`.
3. In `two_org_receiver`, delete the two comment lines
   `    // Each Organisation's admission is expected under the invite identifier`
   `    // of the Persona admitted into it (b2 into org 2, b1 into org 1).`
   and replace
   `    svc_b.expect_admission(&mut OsRng, org_2, invite_for(&b2_device_kp.device_key().unwrap())).unwrap();`
   `    svc_b.expect_admission(&mut OsRng, org_1, invite_for(&b1_device_kp.device_key().unwrap())).unwrap();`
   by
   `    svc_b.expect_admission(&mut OsRng, org_2).unwrap();`
   `    svc_b.expect_admission(&mut OsRng, org_1).unwrap();`.
4. In `a_first_admission_binds_the_unbound_persona_not_one_bound_elsewhere`,
   replace its comment lines
   `// bound to org 1; B adds b2 and expects an admission to org 2 under b2's`
   `// invite identifier. Org 2 enrols b1 (the push is lost) and then b2, and`
   by `// bound to org 1; B adds b2 and expects an admission to org 2. Org 2`
   `// enrols b1 (the push is lost) and then b2, and`; replace
   `s.svc_b.expect_admission(&mut OsRng, org_2, invite_for(&b2_dev.device_key().unwrap())).unwrap();`
   by `s.svc_b.expect_admission(&mut OsRng, org_2).unwrap();`.
5. In `a_loopback_revocation_without_an_address_burns_no_epoch`, replace
   `s.svc_a.send_update(&out.outgoing, s.joiner_b.device_key, None, None)` by
   `s.svc_a.send_update(&out.outgoing, s.joiner_b.device_key, None)`.
6. Replace every occurrence of
   `.expect_admission(&mut OsRng, s.org_id, invite_for(&s.joiner_b.device_key)).unwrap()`
   by `.expect_admission(&mut OsRng, s.org_id).unwrap()` (three tests:
   `a_first_admission_to_an_expected_organisation_rests_on_the_chain_alone`,
   `a_first_admission_records_the_chains_organisation_public_key`,
   `a_first_admission_that_misses_the_chain_root_commits_nothing`).
   Then `grep -n "invite" org-node/tests/admission_sender.rs` → only prose
   (`Invite`, `invite`) remains, no `invite_for`, `InviteId` or `invite_id`.

`org-node/tests/commit_paths.rs`:
- Replace `use org_node::{Envelope, Epoch, InviteId, Joiner, PersonaId, RootHash, SequenceNumber};`
  by `use org_node::{Envelope, Epoch, Joiner, PersonaId, RootHash, SequenceNumber};`.
- In `a_commit_discards_the_provisional_updates_it_orphans`, replace
  `genesis_snapshot: Some(out.outgoing.record_snapshot), invite_id: None }` by
  `genesis_snapshot: Some(out.outgoing.record_snapshot) }`.
- Replace the whole test
  `send_update_sends_the_committed_update_with_the_records_key_and_writes_nothing`
  by:

```rust
// Normal: send_update sends the committed Envelope, the earlier snapshot and
// the Organisation private key the node's record holds — taking none from its
// caller, and no invite identifier (REQ-szq3ud, LLR-48jakr) — under the first
// bound Persona's device, to the full address in Loopback mode, and writes
// nothing.
// verifies: REQ-szq3ud, LLR-2xzys9, LLR-48jakr, LLR-jn5jeh, LLR-t4znbk
#[tokio::test(flavor = "multi_thread")]
async fn send_update_sends_the_committed_update_with_the_records_key_and_writes_nothing() {
    let (chain, mut a, org, joiner, _b) = founded("send").await;
    let update = a.admit_member(&mut OsRng, org, &joiner).unwrap();
    chain.apply_update(org, update.resulting_root, update.org_pub_key, Epoch::new(1)).unwrap();
    let out = a.commit_update(&mut OsRng, org).await.unwrap();
    let on_disk = store_bytes("send", "a");
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    a.send_update(&out.outgoing, joiner.device_key, Some(sink_addr)).await.unwrap();
    let (_ep, sender, msg) = sink.await.unwrap();
    assert_eq!(
        msg,
        WireMessage {
            envelope: out.outgoing.envelope.clone(),
            org_private_key: Some(private_key_held(&rec_of(&a, org))),
            genesis_snapshot: Some(out.outgoing.record_snapshot.clone()),
        }
    );
    let first_bound = a.list_personas().iter().find(|p| p.org_id == Some(org)).unwrap().clone();
    assert_eq!(sender, first_bound.device_seed.signing_keypair().device_key().unwrap());
    assert_eq!(store_bytes("send", "a"), on_disk, "a send writes nothing");
}
```

- In `send_update_refuses_without_a_bound_persona_or_a_loopback_address`,
  replace `Some(dead_addr([0x6f; 32])), None).await.is_err()` by
  `Some(dead_addr([0x6f; 32]))).await.is_err()` and
  `a.send_update(&out.outgoing, joiner.device_key, None, None)` by
  `a.send_update(&out.outgoing, joiner.device_key, None)`.
- In `the_revoking_record_takes_the_removal_only_through_commit_update`,
  replace `Some(dead_addr([0x7f; 32])), None).await.is_err()` by
  `Some(dead_addr([0x7f; 32]))).await.is_err()`.

`org-node/tests/encoding_golden.rs`:
- After the T2 paragraph, add a `// GOLDEN_STORE  = "…"` and a
  `// GOLDEN_WIRE   = "…"` comment line holding the current (T2) values, and:

```rust
//
// Re-pinned 2026-10-06 (change worktree-org-node-org-key-pair, T3): the
// invite identifier removed — from the expectation (`ee`x32 at offsets
// 873–904, the store's last 32 bytes) and from the Wire message (`01` ‖
// `ee`x32 at 317–349, its last 33 bytes). Derived by the format rule from the
// values above, never captured from the code.
```

- Replace the `GOLDEN_STORE` value by:

```
0107702d616c69636501aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa05616c69636505416c69636505536d697468111111111111111111111111111111111111111111111111111111111111111112121212121212121212121212121212121212121212121212121212121212120101010101010101010101010101010101010101010101010101010101010101010101aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa3333333333333333333333333333333333333333333333333333333333333333d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737030202010101010101010101010101010101010101010101010101010101010101010105616c69636505416c69636505536d697468d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21020202020202020202020202020202020202020202020202020202020202020203626f6203426f62054a6f6e6573884b8857f4eaa1613c61504db34d4beaf346517a0e31de3cddd4d9b4201d9d0b01a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f001555555555555555555555555555555555555555555555555555555555555555500020007702d616c69636500777777777777777777777777777777777777777777777777777777777777777701d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c97787370001010101010101010101010101010101010101010101010101010101010101010105616c69636505416c69636505536d697468d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21888888888888888888888888888888888888888888888888888888888888888801bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb07702d616c696365013333333333333333333333333333333333333333333333333333333333333333666666666666666666666666666666666666666666666666666666666666666603d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701040908070601cccccccccccccccccccccccccccccccccccccccc
```

- Replace the `GOLDEN_WIRE` value by:

```
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa029101b642ec203b364f4807ea74b0a63ab680c1c94d38b23246b1042f520394477e860001020202020202020202020202020202020202020202020202020202020202020203626f628139770ea87d175f56a35466c34c7ecccb8d8a91b4ee37a25df60f5b8fc9b3940454657374045573657201ed4928c628d1c2c6eae90338905995612959273a5c63f93636c14614ac8737d101444444444444444444444444444444444444444444444444444444444444444401720101010101010101010101010101010101010101010101010101010101010101010561646d696e04546573740455736572d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701ca93ac1705187071d67b83c7ff0efe8108e8ec4530575d7726879333dbdabe7c
```

- In `persona_store_plaintext_is_pinned`, replace
  `    assert_eq!(m["expected_admissions"][0]["invite_id"][31], json!(0xee));` by
  `    assert!(m["expected_admissions"][0].get("invite_id").is_none(), "an expectation names the Organisation alone");`.
- In `admission_wire_message_is_pinned`, replace
  `    assert_eq!(m["invite_id"][0], json!(0xee));` by
  `    assert!(m.get("invite_id").is_none(), "no invite identifier on the wire (LLR-ms8njy)");`.

`org-node/tests/persona_records.rs`: delete `use org_node::InviteId;`; in
`struct WireStore` replace `    expected_admissions: Vec<([u8; 20], [u8; 32])>,`
by `    expected_admissions: Vec<[u8; 20]>,`; in `wire_store` replace
`        expected_admissions: vec![([0x77; 20], [0x78; 32])],` by
`        expected_admissions: vec![[0x77; 20]],`; in
`a_store_with_every_record_kind_opens_with_every_field_parsed` replace
`        vec![ExpectedAdmission { org_id: OrgId::new([0x77; 20]), invite_id: InviteId::new([0x78; 32]) }]`
by `        vec![ExpectedAdmission { org_id: OrgId::new([0x77; 20]) }]`.

`org-node/tests/provisional_store.rs`: in
`use org_node::{Handle, InviteId, Name, OrgNodeError, OrgPrivateKey, PersonaId, RootHash, SequenceNumber, Surname};`
delete `InviteId, `; replace
`    let expectation = ExpectedAdmission { org_id: OrgId::new([0xcc; 20]), invite_id: InviteId::new([0xee; 32]) };`
by `    let expectation = ExpectedAdmission { org_id: OrgId::new([0xcc; 20]) };`.

`org-node/tests/secret_redaction.rs`: in `wire`, replace
`genesis_snapshot: None, invite_id: None }` by `genesis_snapshot: None }`.

`org-node/tests/transport_handshake.rs` (two literals),
`org-node/tests/transport_networked.rs` (one literal): delete
`, invite_id: None` / the line `        invite_id: None,`.

`org-node/tests/wire_frame_bound.rs`: in `sample_msg`, replace
`genesis_snapshot: None, invite_id: None }` by `genesis_snapshot: None }`;
delete the whole test `a_wire_message_carries_its_invite_id_and_refuses_a_short_one`
(its comments through its closing brace); its evidence is now
`absences::org_node_holds_no_invite_identifier` and the pinned Wire message.

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test absences --test expected_admission`.
Expected red: compile errors — E0061 (`expect_admission` takes 3 arguments,
2 supplied), E0063 (`missing field invite_id in initializer of
ExpectedAdmission`).

**Step 3 — implementation.**
- `org-node/src/types.rs`: delete the `InviteId` doc comment, struct and
  `impl InviteId { … }`.
- `org-node/src/lib.rs`: in `pub use types::{…}` delete `InviteId, `.
- `org-node/src/store.rs`: in `use crate::types::{…}` delete `InviteId, `;
  replace

```rust
/// A first admission the app declared it expects (REQ-8amu2a, LLR-95753m).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpectedAdmission {
    pub org_id: OrgId,
    pub invite_id: InviteId,
}
```

  by

```rust
/// A first admission the app declared it expects: the Organisation alone
/// (REQ-8amu2a, LLR-95753m).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpectedAdmission {
    pub org_id: OrgId,
}
```

- `org-node/src/transport/wire.rs`: replace `use crate::types::{InviteId, OrgPrivateKey};`
  by `use crate::types::OrgPrivateKey;`; delete

```rust
    /// The invite identifier an admission is delivered under (LLR-ms8njy,
    /// REQ-8amu2a); `None` for every other update.
    pub invite_id: Option<InviteId>,
```

- `org-node/src/service.rs`:
  - Replace `use crate::types::{ChainAccount, Epoch, InviteId, OrgPrivateKey, OrgPublicKey, PersonaId, SequenceNumber};`
    by `use crate::types::{ChainAccount, Epoch, OrgPrivateKey, OrgPublicKey, PersonaId, SequenceNumber};`.
  - In `send_update`: replace the doc lines
    `    /// the caller (REQ-szq3ud) — and exactly the invite identifier given`
    `    /// (LLR-48jakr). Loopback dials \`peer_addr\` and refuses without one,`
    by `    /// the caller (REQ-szq3ud) — and no invite identifier (LLR-48jakr).`
    `    /// Loopback dials \`peer_addr\` and refuses without one,`; delete the
    parameter line `        invite_id: Option<InviteId>,` and the field line
    `            invite_id,`.
  - In `receive_and_verify`, replace

```rust
        // A first admission is read only when the app expects it, for this
        // Organisation under this invite identifier — before its snapshot is
        // decoded or the chain is read (LLR-s8xp7m, RC-2ferct).
        let expectation = msg.invite_id.map(|invite_id| ExpectedAdmission { org_id, invite_id });
        if is_first_admission && !expectation.is_some_and(|e| self.store.data().expected_admissions.contains(&e)) {
            return Err(OrgNodeError::AdmissionNotExpected { org_id });
        }
```

    by

```rust
        // A first admission is read only when the app expects one to this
        // Organisation — before its snapshot is decoded or the chain is read
        // (LLR-s8xp7m, RC-2ferct).
        let expectation = ExpectedAdmission { org_id };
        if is_first_admission && !self.store.data().expected_admissions.contains(&expectation) {
            return Err(OrgNodeError::AdmissionNotExpected { org_id });
        }
```

    and replace

```rust
            if let Some(matched) = expectation {
                data.expected_admissions.retain(|e| *e != matched);
            }
```

    by `            data.expected_admissions.retain(|e| *e != expectation);`.
  - Replace `expect_admission` (doc through closing brace) by:

```rust
    /// Record that the app expects a first admission to `org_id`, at most
    /// once, and save before returning (LLR-9zfnmb).
    pub fn expect_admission<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
    ) -> Result<(), OrgNodeError> {
        let expectation = ExpectedAdmission { org_id };
        let expected = &mut self.store.data_mut().expected_admissions;
        if !expected.contains(&expectation) {
            expected.push(expectation);
        }
        self.store.save(rng)
    }
```

- `grep -rn "InviteId\|invite_id" org-node/src` → no output.

**Step 4 — green.**
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test absences --test expected_admission --test wire_frame_bound --test encoding_golden`
→ absences `6 passed`, expected_admission `8 passed`, wire_frame_bound
`5 passed`, encoding_golden `3 passed`, 0 failed. Full org-node cargo line →
every target `ok`, 0 failed (admission_sender 47, commit_paths 28);
`cargo check --all-targets` and `clippy` → `Finished`. App: still fails
(expected until T10).

**Step 5 — commit.** `git add` the files of **Files touched**, then
`git -c commit.gpgsign=false commit -m "T3: the invite identifier leaves org-node; an expectation names the Organisation" -m "verifies: REQ-8amu2a, LLR-ms8njy, LLR-48jakr, LLR-9zfnmb, LLR-s8xp7m, LLR-q8emds, LLR-mbjfq8, LLR-95753m, LLR-ayrdr8"`.

**Red→green attestations:**
- org_node_holds_no_invite_identifier — red -> green: watched fail ("types.rs contains `InviteId`: LLR-ms8njy"; absences 5 passed, 1 failed) before the implementation; green after (6 passed).
- expect_admission_records_an_organisation_once_and_reaches_the_disk (rewritten, renamed) — red -> green: compile red (E0061 expect_admission takes 3 arguments, E0063 missing field `invite_id` in `ExpectedAdmission`); green after.
- an_expectation_for_another_organisation_survives_the_commit (rewritten, renamed) — red -> green: compile red (E0061 on the 2-argument expect_admission call); green after.
- an_expected_first_admission_is_committed_and_clears_the_expectation, an_expectation_for_one_organisation_admits_no_other, a_first_admission_that_lists_none_of_our_personas_is_refused, a_refused_first_admission_leaves_the_expectation, an_expected_first_admission_decodes_its_snapshot_before_reading_the_chain (re-asserted) — red -> green: the same compile red (E0061 / E0063 in the target and support/mod.rs); green after.
- send_update_sends_the_committed_update_with_the_records_key_and_writes_nothing (re-asserted) — red -> green: compile red (E0061 send_update takes 4 arguments, in support/mod.rs); green after (commit_paths 28 passed).
- persona_store_plaintext_is_pinned, admission_wire_message_is_pinned (re-pinned) — red -> green: compile red with the suite (E0061/E0063); green after against the re-pinned values, which match the derivation rule byte for byte (encoding_golden 3 passed). Task done 2026-10-06, merged from worktree-org-node-org-key-pair-t3 (ab6e3e0); org-node 229 passed, 0 failed. Left for T12 / deslop: `support::first_device` is now unused; org-node README and architecture docs still name InviteId / invite_id.

---

### T4 — org-node: the Wire message has two kinds; the kind follows the recipient

**Files touched:** `org-node/src/transport/wire.rs`, `org-node/src/service.rs`,
`org-node/Cargo.toml`, `org-node/.guardrails/config.yaml`,
`org-node/tests/fuzz_wire_decode/fuzz_target.rs` (new),
`org-node/tests/fuzz_first_admission_base/fuzz_target.rs`,
`org-node/tests/support/mod.rs`, `org-node/tests/admission_sender.rs`,
`org-node/tests/commit_paths.rs`, `org-node/tests/encoding_golden.rs`,
`org-node/tests/expected_admission.rs`, `org-node/tests/persona_records.rs`,
`org-node/tests/receive_chain_reads.rs`, `org-node/tests/secret_redaction.rs`,
`org-node/tests/transport_handshake.rs`, `org-node/tests/transport_networked.rs`,
`org-node/tests/wire_frame_bound.rs`
**Parallel:** no (serial, after T3 and T1)
**IDs verified:** LLR-js9dsu, LLR-ecxc76, LLR-6ymd6d, LLR-8hdu9x, LLR-bg3vsw,
LLR-2xzys9, LLR-jn5jeh, LLR-38e2kn, LLR-j6j95z, LLR-ms8njy, LLR-48jakr,
LLR-bwb9pu, LLR-ayrdr8, REQ-3dsweu (send half), REQ-vxqc5g, REQ-szq3ud,
REQ-c29s93 (decode half), REQ-d9g6nt.
**Size:** ~480 lines.

**Step 1 — test helpers** (`org-node/tests/support/mod.rs`). Replace
`envelope_only` by:

```rust
/// A Wire message carrying `envelope` and nothing else: a revocation.
pub fn envelope_only(envelope: org_node::Envelope) -> WireMessage {
    WireMessage::Revocation { envelope }
}

/// `msg`, of the same kind, with its Envelope replaced.
pub fn with_envelope(msg: &WireMessage, envelope: org_node::Envelope) -> WireMessage {
    match msg {
        WireMessage::OrgInformation { record_snapshot, org_private_key, .. } => WireMessage::OrgInformation {
            envelope,
            record_snapshot: record_snapshot.clone(),
            org_private_key: org_private_key.clone(),
        },
        WireMessage::Revocation { .. } => WireMessage::Revocation { envelope },
    }
}

/// The Organisation information `msg`, with its record snapshot replaced.
pub fn with_snapshot(msg: &WireMessage, record_snapshot: Vec<u8>) -> WireMessage {
    match msg {
        WireMessage::OrgInformation { envelope, org_private_key, .. } => WireMessage::OrgInformation {
            envelope: envelope.clone(),
            record_snapshot,
            org_private_key: org_private_key.clone(),
        },
        WireMessage::Revocation { .. } => panic!("a revocation carries no snapshot"),
    }
}

/// The Organisation information `msg`, with its Organisation private key
/// replaced.
pub fn with_key(msg: &WireMessage, org_private_key: OrgPrivateKey) -> WireMessage {
    match msg {
        WireMessage::OrgInformation { envelope, record_snapshot, .. } => WireMessage::OrgInformation {
            envelope: envelope.clone(),
            record_snapshot: record_snapshot.clone(),
            org_private_key,
        },
        WireMessage::Revocation { .. } => panic!("a revocation carries no key"),
    }
}
```

**Step 2 — failing tests.**

`org-node/tests/wire_frame_bound.rs` — replace everything from the `use`
lines down to the end of `oversize_message_is_rejected_on_encode` by:

```rust
use org_node::ids::OrgId;
use org_node::test_fixtures::admit_member_delta;
use org_node::transport::wire::{decode_body, encode_frame, WireMessage};
use org_node::transport::{TransportError, MAX_FRAME};
use org_node::{Envelope, MemberSeed, OrgPrivateKey, SequenceNumber};

fn sample_envelope() -> Envelope {
    let (delta, _) = admit_member_delta(&MemberSeed::from([1u8; 32]).x25519_keypair());
    Envelope::build(OrgId::new([5u8; 20]), SequenceNumber::new(2), &delta).unwrap()
}

fn sample_msg() -> WireMessage {
    WireMessage::OrgInformation {
        envelope: sample_envelope(),
        record_snapshot: vec![1, 2, 3],
        org_private_key: OrgPrivateKey::from([9u8; 32]),
    }
}

/// The postcard body of `msg`, without the frame's length prefix.
fn body_of(msg: &WireMessage) -> Vec<u8> {
    encode_frame(msg).unwrap()[4..].to_vec()
}

// Normal: each kind round-trips through a frame, begins with its variant
// index (0 Organisation information, 1 revocation) and hands back its
// Envelope. Abnormal: a body whose index is neither 0 nor 1 does not decode.
// verifies: LLR-js9dsu, REQ-3dsweu
#[test]
fn the_two_kinds_round_trip_and_a_body_of_neither_kind_is_refused() {
    let revocation = WireMessage::Revocation { envelope: sample_envelope() };
    for msg in [sample_msg(), revocation.clone()] {
        assert_eq!(decode_body(&body_of(&msg)).unwrap(), msg);
        assert_eq!(msg.envelope(), &sample_envelope());
    }
    assert_eq!(body_of(&sample_msg())[0], 0);
    assert_eq!(body_of(&revocation)[0], 1);
    let mut other = body_of(&revocation);
    for index in [2u8, 3, 0x7f] {
        other[0] = index;
        assert!(matches!(decode_body(&other), Err(TransportError::Malformed)), "index {index}");
    }
}

// Abnormal: an Organisation-information body that ends before its record
// snapshot, or before the 32 bytes of its Organisation private key, does not
// decode, and does not panic.
// verifies: LLR-js9dsu, REQ-c29s93
#[test]
fn an_organisation_information_body_without_its_snapshot_or_key_does_not_decode() {
    let body = body_of(&sample_msg());
    let mut no_snapshot = body_of(&WireMessage::Revocation { envelope: sample_envelope() });
    no_snapshot[0] = 0; // Organisation information's index, then the Envelope and nothing
    for cut in [no_snapshot, body[..body.len() - 32].to_vec(), body[..body.len() - 1].to_vec()] {
        assert!(matches!(decode_body(&cut), Err(TransportError::Malformed)), "{} bytes", cut.len());
    }
}

// verifies: REQ-eg5j8u, LLR-fa7jt8, LLR-er2x8n
#[test]
fn frame_round_trips() {
    let msg = sample_msg();
    let framed = encode_frame(&msg).unwrap();
    // strip the 4-byte length prefix
    let len = u32::from_le_bytes(framed[0..4].try_into().unwrap()) as usize;
    assert_eq!(len, framed.len() - 4);
    let back = decode_body(&framed[4..]).unwrap();
    assert_eq!(back, msg);
}

// verifies: REQ-eg5j8u, LLR-8kh3zf
#[test]
fn oversize_body_is_rejected() {
    // A body claiming > MAX_FRAME must be rejected by decode_body.
    let big = vec![0u8; MAX_FRAME + 1];
    assert!(matches!(decode_body(&big), Err(TransportError::FrameTooLarge(_))));
}

// verifies: REQ-eg5j8u, LLR-sc6zuh
#[test]
fn oversize_message_is_rejected_on_encode() {
    let msg = WireMessage::OrgInformation {
        envelope: sample_envelope(),
        record_snapshot: vec![0u8; MAX_FRAME + 1],
        org_private_key: OrgPrivateKey::from([9u8; 32]),
    };
    assert!(matches!(encode_frame(&msg), Err(TransportError::FrameTooLarge(_))));
}
```

In `a_body_of_exactly_the_bound_is_not_refused_for_its_size`, replace the
comment lines
`    // 0xff, not zero: since the Envelope lost its signature (merged`
`    // 2026-10-05), an all-zero body IS a valid message — an empty envelope`
`    // with no secret and no snapshot — because nothing in the wire form needs`
`    // a non-zero length any more. Ten 0xff bytes after the Organisation`
`    // identifier are a varint that overflows its u64, so this body is not.`
by
`    // 0xff, not zero: an all-zero body IS a valid message — Organisation`
`    // information with an empty envelope, an empty snapshot and an all-zero`
`    // key. A leading run of 0xff bytes is a variant-index varint that`
`    // overflows, so this body is not.`

`org-node/tests/fuzz_wire_decode/fuzz_target.rs` (new):

```rust
//! Fuzz target: `decode_body` must never panic on arbitrary bytes, and a body
//! that decodes is one of the two kinds and decodes again after re-encoding
//! (LLR-js9dsu). Shape 2 puts the fuzz bytes after a well-formed
//! Organisation-information prefix (variant index 0 and an Envelope), so every
//! iteration reaches the record snapshot and the Organisation private key:
//! a body that ends before the key's 32 bytes is refused as `Malformed`
//! (REQ-c29s93).
//!
//! `harness = false` binary: a panic (the bolero failure signal) exits
//! non-zero and fails `cargo test`. Run alone with
//! `cargo test -p org-node --features transport --test fuzz_wire_decode`;
//! deep-fuzz with `cargo bolero test fuzz_wire_decode --engine libfuzzer`.
//!
//! verifies: REQ-9g6as6, LLR-js9dsu, REQ-c29s93

use bolero::check;
use org_node::envelope::Envelope;
use org_node::ids::OrgId;
use org_node::transport::wire::{decode_body, encode_frame, WireMessage};
use org_node::transport::TransportError;
use org_node::SequenceNumber;

fn main() {
    let envelope = Envelope { org_id: OrgId::new([5u8; 20]), parent_seq: SequenceNumber::new(1), delta_bytes: vec![1, 2, 3] };
    let mut info_prefix = postcard::to_allocvec(&WireMessage::Revocation { envelope }).expect("encode");
    info_prefix[0] = 0;
    check!().for_each(|bytes: &[u8]| {
        // Shape 1 — the bytes are the whole body.
        if let Ok(msg) = decode_body(bytes) {
            let framed = encode_frame(&msg).expect("a decoded message re-encodes");
            assert_eq!(decode_body(&framed[4..]).expect("and decodes again"), msg);
        }
        // Shape 2 — the bytes follow an Organisation-information prefix.
        let mut body = info_prefix.clone();
        body.extend_from_slice(bytes);
        match decode_body(&body) {
            Ok(WireMessage::OrgInformation { org_private_key, .. }) => {
                assert_eq!(org_private_key.expose_secret().len(), 32);
            }
            Ok(WireMessage::Revocation { .. }) => panic!("variant index 0 decoded as a revocation"),
            Err(TransportError::Malformed) => {}
            Err(other) => panic!("refused as {other:?}, not as Malformed"),
        }
    });
}
```

`org-node/Cargo.toml`: after the `fuzz_first_admission_base` `[[test]]`
entry add:

```toml

# `decode_body` on arbitrary bytes and on bytes after an Organisation-information
# prefix (LLR-js9dsu), change worktree-org-node-org-key-pair.
[[test]]
name = "fuzz_wire_decode"
path = "tests/fuzz_wire_decode/fuzz_target.rs"
harness = false
required-features = ["transport"]
```

`org-node/.guardrails/config.yaml`: in the first `verify_commands` line,
replace `--test fuzz_first_admission_base --test verify_against_chain` by
`--test fuzz_first_admission_base --test fuzz_wire_decode --test verify_against_chain`.

`org-node/tests/fuzz_first_admission_base/fuzz_target.rs`: replace the
comment `// Abnormal case of REQ-d9g6nt over arbitrary input: no snapshot is`
`// always refused, and arbitrary snapshot bytes never panic.` by
`// Abnormal case of REQ-d9g6nt over arbitrary input: arbitrary snapshot bytes`
`// never panic. (A first admission without a snapshot cannot be expressed:`
`// Organisation information requires one, LLR-js9dsu, LLR-j6j95z.)`; delete
the `match first_admission_base(None) { … }` block and its comment line
`    // No snapshot: refused with its own error, not some later failure.`;
replace `let _ = first_admission_base(Some(bytes));` by
`let _ = first_admission_base(bytes);` and
`let _ = first_admission_base(Some(&encoded));` by
`let _ = first_admission_base(&encoded);`; delete the import line
`use org_node::OrgNodeError;` (its only use was the deleted block).

`org-node/tests/persona_records.rs`: replace every occurrence of
`first_admission_base(Some(&bytes))` by `first_admission_base(&bytes)` and of
`first_admission_base(Some(bytes))` by `first_admission_base(bytes)`.

`org-node/tests/secret_redaction.rs`: replace `wire`'s last line by

```rust
    WireMessage::OrgInformation { envelope, record_snapshot: vec![], org_private_key: OrgPrivateKey::from(private) }
```

and add after `records_and_wire_messages_never_render_secret_bytes`:

```rust
// LLR-ecxc76: Organisation information renders its key as the redaction
// marker and none of its bytes; a revocation holds no key and renders none.
/// verifies: LLR-ecxc76, LLR-bwb9pu
#[test]
fn a_wire_message_of_either_kind_never_renders_the_key() {
    for private in [sentinel(0x40), sentinel_down(0xff)] {
        let info = wire(private);
        let revocation = WireMessage::Revocation { envelope: info.envelope().clone() };
        for rendered in [format!("{info:?}"), format!("{info:#?}")] {
            assert_not_rendered(&rendered, &private, "Organisation private key");
            assert!(rendered.contains("OrgPrivateKey([REDACTED])"), "{rendered}");
        }
        for rendered in [format!("{revocation:?}"), format!("{revocation:#?}")] {
            assert_not_rendered(&rendered, &private, "Organisation private key");
            assert!(!rendered.contains("OrgPrivateKey"), "a revocation holds no key: {rendered}");
        }
    }
}
```

`org-node/tests/commit_paths.rs`:
- In `a_commit_discards_the_provisional_updates_it_orphans`, replace the
  `deliver(b_addr, &WireMessage { … }).await;` line by:

```rust
    let info = WireMessage::OrgInformation {
        envelope: out.outgoing.envelope,
        record_snapshot: out.outgoing.record_snapshot,
        org_private_key: private_key_held(&rec_of(&s.svc_a, s.org_id)),
    };
    deliver(b_addr, &info).await;
```

- Replace the whole test
  `send_update_sends_the_committed_update_with_the_records_key_and_writes_nothing`
  by:

```rust
// Normal: send_update chooses the kind from the node's record alone. A Device
// the committed record lists receives Organisation information — the
// committed Envelope, the record as it stood before the commit, the key the
// record holds, none taken from the caller; any other Device a revocation,
// the Envelope and nothing else. Sent under the first bound Persona's device,
// to the full address in Loopback mode, writing nothing. After a removal the
// removed Device is the one not listed, and the founder's is still listed.
// verifies: LLR-6ymd6d, LLR-8hdu9x, LLR-bg3vsw, LLR-2xzys9, LLR-48jakr, LLR-jn5jeh, LLR-t4znbk, REQ-szq3ud, REQ-3dsweu
#[tokio::test(flavor = "multi_thread")]
async fn send_update_sends_organisation_information_to_a_listed_device_and_a_revocation_to_any_other() {
    let (chain, mut a, org, joiner, _b) = founded("send").await;
    let update = a.admit_member(&mut OsRng, org, &joiner).unwrap();
    chain.apply_update(org, update.resulting_root, update.org_pub_key, Epoch::new(1)).unwrap();
    let out = a.commit_update(&mut OsRng, org).await.unwrap();
    let on_disk = store_bytes("send", "a");
    let rec = rec_of(&a, org);
    let first_bound = a.list_personas().iter().find(|p| p.org_id == Some(org)).unwrap().clone();
    let founder_device = first_bound.device_seed.signing_keypair().device_key().unwrap();

    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    a.send_update(&out.outgoing, joiner.device_key, Some(sink_addr)).await.unwrap();
    let (_ep, sender, msg) = sink.await.unwrap();
    assert_eq!(
        msg,
        WireMessage::OrgInformation {
            envelope: out.outgoing.envelope.clone(),
            record_snapshot: out.outgoing.record_snapshot.clone(),
            org_private_key: private_key_held(&rec),
        },
        "the joiner's Device is listed"
    );
    assert_eq!(sender, founder_device);
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    a.send_update(&out.outgoing, device_key(0x7a), Some(sink_addr)).await.unwrap();
    assert_eq!(
        sink.await.unwrap().2,
        WireMessage::Revocation { envelope: out.outgoing.envelope.clone() },
        "a Device the record does not list"
    );
    assert_eq!(store_bytes("send", "a"), on_disk, "a send writes nothing");

    let joiner_id = rec.trie_members.iter().find(|m| m.member_key == joiner.member_key).unwrap().id;
    let removal = a.revoke_member(&mut OsRng, org, joiner_id).unwrap();
    chain.apply_update(org, removal.resulting_root, removal.org_pub_key, Epoch::new(2)).unwrap();
    let out = a.commit_update(&mut OsRng, org).await.unwrap();
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    a.send_update(&out.outgoing, joiner.device_key, Some(sink_addr)).await.unwrap();
    assert_eq!(
        sink.await.unwrap().2,
        WireMessage::Revocation { envelope: out.outgoing.envelope.clone() },
        "the removed Device"
    );
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    a.send_update(&out.outgoing, founder_device, Some(sink_addr)).await.unwrap();
    assert!(matches!(sink.await.unwrap().2, WireMessage::OrgInformation { .. }), "a Device still listed");
}
```

`org-node/tests/encoding_golden.rs`:
- Module doc: replace `postcard plaintext of a Persona store, and the postcard body of an`
  `//! admission Wire message and the record snapshot inside it.` by
  `postcard plaintext of a Persona store, and the postcard bodies of the two`
  `//! kinds of Wire message and the record snapshot inside the first.`
- Replace `use org_node::transport::wire::{decode_body, encode_frame};` by
  `use org_node::transport::wire::{decode_body, encode_frame, WireMessage};`.
- After the T3 paragraph add a `// GOLDEN_WIRE   = "…"` comment line holding
  the current (T3) wire value, and:

```rust
//
// Re-pinned 2026-10-06 (change worktree-org-node-org-key-pair, T4): the Wire
// message is an enum of two kinds (LLR-js9dsu). GOLDEN_WIRE is Organisation
// information: `00` ‖ the Envelope (bytes 0–167 of the value above) ‖ the
// record snapshot without its option tag (bytes 202–316) ‖ the Organisation
// private key without its option tag (bytes 169–200). GOLDEN_REVOCATION is
// `01` ‖ the same Envelope. Derived by the format rule, never captured from
// the code.
```

- Replace the `GOLDEN_WIRE` value by:

```
00aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa029101b642ec203b364f4807ea74b0a63ab680c1c94d38b23246b1042f520394477e860001020202020202020202020202020202020202020202020202020202020202020203626f628139770ea87d175f56a35466c34c7ecccb8d8a91b4ee37a25df60f5b8fc9b3940454657374045573657201ed4928c628d1c2c6eae90338905995612959273a5c63f93636c14614ac8737d1720101010101010101010101010101010101010101010101010101010101010101010561646d696e04546573740455736572d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701ca93ac1705187071d67b83c7ff0efe8108e8ec4530575d7726879333dbdabe7c4444444444444444444444444444444444444444444444444444444444444444
```

  and after it add:

```rust
const GOLDEN_REVOCATION: &str = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa029101b642ec203b364f4807ea74b0a63ab680c1c94d38b23246b1042f520394477e860001020202020202020202020202020202020202020202020202020202020202020203626f628139770ea87d175f56a35466c34c7ecccb8d8a91b4ee37a25df60f5b8fc9b3940454657374045573657201ed4928c628d1c2c6eae90338905995612959273a5c63f93636c14614ac8737d1";
```

- Replace the whole test `admission_wire_message_is_pinned` by:

```rust
// *Renamed 2026-10-06 (change worktree-org-node-org-key-pair, T4).* Was
// `admission_wire_message_is_pinned`.
/// verifies: LLR-ayrdr8, LLR-ms8njy, LLR-js9dsu
#[test]
fn organisation_information_wire_message_is_pinned() {
    let body = unhex(GOLDEN_WIRE);
    let msg = decode_body(&body).unwrap();
    let framed = encode_frame(&msg).unwrap();
    assert_eq!(hex(&framed[4..]), GOLDEN_WIRE);

    let m = model(&msg);
    let info = &m["OrgInformation"];
    assert_eq!(info["envelope"]["org_id"][0], json!(0xaa));
    assert_eq!(info["envelope"]["parent_seq"], json!(2));
    assert_eq!(info["org_private_key"][31], json!(0x44));
    assert!(info.get("invite_id").is_none() && info.get("org_secret").is_none());

    let WireMessage::OrgInformation { record_snapshot, .. } = &msg else { panic!("Organisation information") };
    let members: Vec<MemberSnapshot> = postcard::from_bytes(record_snapshot).unwrap();
    assert_eq!(postcard::to_allocvec(&members).unwrap(), *record_snapshot);
    let admin = &model(&members)[0];
    assert_eq!(admin["handle"], json!("admin"));
    assert_eq!(admin["member_key"][0], json!(0xd0));
    assert!(first_admission_base(record_snapshot).is_ok(), "the pinned snapshot rebuilds a record");
}

/// verifies: LLR-ayrdr8, LLR-ms8njy, LLR-js9dsu
#[test]
fn revocation_wire_message_is_pinned() {
    let body = unhex(GOLDEN_REVOCATION);
    let msg = decode_body(&body).unwrap();
    assert_eq!(hex(&encode_frame(&msg).unwrap()[4..]), GOLDEN_REVOCATION);
    let m = model(&msg);
    assert_eq!(m["Revocation"]["envelope"]["parent_seq"], json!(2));
    assert_eq!(m["Revocation"].as_object().unwrap().len(), 1, "the Envelope and nothing else");
    assert_eq!(body.len(), 169, "the variant index and the 168-byte Envelope");
}
```

- In `truncated_pinned_values_are_refused`, after its last line add:

```rust
    let body = unhex(GOLDEN_REVOCATION);
    assert!(decode_body(&body[..body.len() - 1]).is_err());
```

`org-node/tests/admission_sender.rs`:
1. Replace the import line
   `use org_node::{DeviceSeed, Epoch, MemberSeed, OrgPublicKey, PersonaId, RootHash, SequenceNumber};`
   by `use org_node::{DeviceSeed, Epoch, MemberSeed, OrgPrivateKey, OrgPublicKey, PersonaId, RootHash, SequenceNumber};`.
2. In `a_first_admission_relayed_by_another_device_is_committed`, replace
   `assert!(msg.org_private_key.is_some() && msg.genesis_snapshot.is_some(), "a full admission message");`
   by `assert!(matches!(msg, WireMessage::OrgInformation { .. }), "Organisation information");`.
3. In `a_message_for_an_organisation_absent_from_the_chain_is_refused`, replace
   the `let msg = WireMessage { … };` statement by:

```rust
    let msg = WireMessage::OrgInformation {
        envelope,
        record_snapshot: snapshot_bytes(&base),
        org_private_key: OrgPrivateKey::from([0x42u8; 32]),
    };
```

4. Replace the whole test `first_admission_without_a_record_snapshot_is_refused`
   (comment block `// Abnormal case of REQ-d9g6nt over the service:` through
   its closing brace) by:

```rust
// LLR-38e2kn and LLR-j6j95z: a first admission is attempted only from
// Organisation information, whose snapshot is required. A's genuine admission
// of B, relabelled by a relay as a revocation (its Envelope alone), is
// refused before the expectations are consulted or the chain is read: no
// record, the expectation kept, B's Persona untouched, nothing written. The
// genuine message then commits.
// *Rewritten 2026-10-06 (change worktree-org-node-org-key-pair, T4).* Was
// `first_admission_without_a_record_snapshot_is_refused`, of a message whose
// optional snapshot was stripped.
// verifies: REQ-d9g6nt, REQ-vxqc5g, LLR-j6j95z, LLR-38e2kn
#[tokio::test(flavor = "multi_thread")]
async fn a_revocation_about_an_organisation_not_held_is_refused_before_the_chain() {
    let (mut s, counting) = setup_counted("no-snapshot").await;
    let joiner = s.joiner_b.clone();
    let genuine = captured_admission(&mut s, &joiner).await;
    let relabelled = WireMessage::Revocation { envelope: genuine.envelope().clone() };
    let on_disk = store_bytes("no-snapshot", "b");
    let reads = counting.reads();
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(b_addr, &relabelled).await;
    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::RevocationNotHeld { org_id: s.org_id });
    assert_eq!(counting.reads(), reads, "no chain read");
    assert!(svc_b.list_orgs().is_empty(), "no record");
    assert_eq!(svc_b.expected_admissions().len(), 1, "the expectation is kept");
    let persona_b = persona_of(&svc_b, &s.pid_b);
    assert_eq!((persona_b.status, persona_b.org_id), (PersonaStatus::Proposed, None));
    assert_eq!(store_bytes("no-snapshot", "b"), on_disk, "nothing written");
    let (b_addr, b_task) = spawn_receive(svc_b, &s.b_device_kp).await;
    deliver(b_addr, &genuine).await;
    let (_svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.expect("the genuine admission commits").org_id, s.org_id);
}
```

5. In `a_revocation_reaches_the_administrators_disk`, replace the block from
   `    let (_sink_ep, _sender, msg) = sink.await.unwrap();` through the line
   `        "the snapshot is the record before the removal"` and its closing
   `    );` by:

```rust
    let (_sink_ep, _sender, msg) = sink.await.unwrap();

    // LLR-8hdu9x: the Device the removal removed is sent a revocation — the
    // committed Envelope and nothing else, no snapshot and no key.
    let WireMessage::Revocation { envelope } = &msg else { panic!("a revocation, got {msg:?}") };
```

   then, in the rest of that test, replace `msg.envelope.parent_seq` by
   `envelope.parent_seq` (two places), and replace its annotation
   `// verifies: LLR-qg9utu, LLR-8hdu9x` by
   `// verifies: LLR-qg9utu, LLR-8hdu9x, LLR-6ymd6d`.
6. In `in_loopback_mode_the_joiner_is_dialled_at_the_full_address`, replace
   the `assert!( msg.org_private_key.is_some() && msg.genesis_snapshot.is_some(), "a full admission message, not a bare update" );`
   statement by
   `assert!(matches!(msg, WireMessage::OrgInformation { .. }), "Organisation information, not a revocation");`.
7. In `a_first_admission_that_fails_verification_leaves_the_expectation`,
   replace `    let (_r_ep, _sender, mut msg) = r_task.await.unwrap();` and
   `    msg.envelope.delta_bytes = vec![0xffu8; 16];` by
   `    let (_r_ep, _sender, genuine) = r_task.await.unwrap();` and
   `    let msg = with_envelope(&genuine, org_node::Envelope { delta_bytes: vec![0xffu8; 16], ..genuine.envelope().clone() });`.
8. In both `a_sequence_number_beyond_the_chain_epoch_cannot_jam_the_receive_path`
   and `…_self_delete_path`, replace the two lines
   `    let mut jam = genuine.clone();` and
   `    jam.envelope.parent_seq = SequenceNumber::new(u64::MAX);` by
   `    let jam = with_envelope(&genuine, org_node::Envelope { parent_seq: SequenceNumber::new(u64::MAX), ..genuine.envelope().clone() });`.
9. In `the_persona_marked_active_is_the_first_whose_device_is_in_the_record`,
   replace `    let msg = envelope_only(envelope);` by:

```rust
    let msg = WireMessage::OrgInformation {
        envelope,
        record_snapshot: snapshot_bytes(&trie_of(&rec_a)),
        org_private_key: private_key_held(&rec_a),
    };
```

10. Replace every remaining occurrence of `msg.envelope.` by `msg.envelope().`
    (the tests `the_admission_envelope_carries_the_epoch_its_update_produced`
    and `in_loopback_mode_the_joiner_is_dialled_at_the_full_address`). Then
    `grep -n "\.envelope\.\|genesis_snapshot\|org_private_key\.is_some" org-node/tests/admission_sender.rs`
    → no output.

`org-node/tests/expected_admission.rs`:
- In `an_unexpected_first_admission_is_refused_before_the_snapshot_or_the_chain`,
  replace `    let garbled = WireMessage { genesis_snapshot: Some(vec![0xff; 4]), ..msg.clone() };`
  by `    let garbled = with_snapshot(&msg, vec![0xff; 4]);`.
- In `a_refused_first_admission_leaves_the_expectation`, replace
  `deliver(addr, &WireMessage { genesis_snapshot: Some(wrong_base), ..msg }).await;`
  by `deliver(addr, &with_snapshot(&msg, wrong_base)).await;`.
- Replace the whole test
  `an_expected_first_admission_decodes_its_snapshot_before_reading_the_chain`
  by:

```rust
// Abnormal: an expected first admission decodes its snapshot — and refuses
// one it cannot decode — before it reads the chain. (One without a snapshot
// is a revocation, refused before the expectations:
// `admission_sender::a_revocation_about_an_organisation_not_held_is_refused_before_the_chain`.)
// verifies: REQ-8amu2a, LLR-s8xp7m, LLR-j6j95z
#[tokio::test(flavor = "multi_thread")]
async fn an_expected_first_admission_decodes_its_snapshot_before_reading_the_chain() {
    let (mut s, counting) = setup_counted("decode-first").await;
    let joiner = s.joiner_b.clone();
    let msg = captured_admission(&mut s, &joiner).await;
    let before = counting.reads();
    let (addr, task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(addr, &with_snapshot(&msg, vec![0xff; 4])).await;
    let (svc_b, result) = task.await.unwrap();
    assert!(matches!(result, Err(OrgNodeError::Chain(_))), "{result:?}");
    assert_eq!(counting.reads(), before, "no chain read before the snapshot decodes");
    assert_eq!(svc_b.expected_admissions(), &[expectation(s.org_id)]);
}
```

- Delete `use org_node::transport::wire::WireMessage;` if
  `grep -n "WireMessage" org-node/tests/expected_admission.rs` shows no other
  use.

`org-node/tests/receive_chain_reads.rs`: in
`a_held_organisation_absent_from_the_chain_is_refused_after_the_chain_free_checks`,
replace
`    let garbled = WireMessage { envelope: Envelope { delta_bytes: vec![0xff; 16], ..msg.envelope.clone() }, ..msg.clone() };`
by
`    let garbled = with_envelope(&msg, Envelope { delta_bytes: vec![0xff; 16], ..msg.envelope().clone() });`;
delete `use org_node::transport::wire::WireMessage;`; in
`the_self_delete_path_refuses_an_unheld_organisation_without_a_chain_read`
replace `// verifies: LLR-379hnv` by `// verifies: LLR-379hnv, LLR-38e2kn`.

`org-node/tests/transport_handshake.rs`: replace the literal
`WireMessage { envelope: env.clone(), org_private_key: Some(OrgPrivateKey::from([0xab; 32])), genesis_snapshot: None }`
by
`WireMessage::OrgInformation { envelope: env.clone(), record_snapshot: vec![], org_private_key: OrgPrivateKey::from([0xab; 32]) }`,
and in `the_length_prefix_is_not_checked_against_the_body` the literal
`WireMessage { envelope: env, org_private_key: None, genesis_snapshot: None }`
by `WireMessage::Revocation { envelope: env }`.

`org-node/tests/transport_networked.rs`: replace the `let msg = WireMessage { … };`
literal by:

```rust
    let msg = WireMessage::OrgInformation {
        envelope: env.clone(),
        record_snapshot: vec![],
        org_private_key: OrgPrivateKey::from([0xab; 32]),
    };
```

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test wire_frame_bound`.
Expected red: compile error E0599 (`no variant named OrgInformation for
struct WireMessage` — it is still a struct).

**Step 3 — implementation.**

`org-node/src/transport/wire.rs`: replace the doc comment and the struct
`WireMessage` by:

```rust
/// One message over the org-node channel, of one of two kinds (LLR-js9dsu).
/// The kind follows the recipient, not the operation (REQ-3dsweu): a Device
/// the sending node's committed record lists receives Organisation
/// information, any other Device a revocation. Neither kind carries an
/// invite identifier (LLR-ms8njy). `Debug` is derived: the only secret
/// either kind holds is an `OrgPrivateKey`, whose own `Debug` redacts it
/// (LLR-ecxc76).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WireMessage {
    /// Index 0: the committed Envelope; the record it extends, postcard
    /// `Vec<MemberSnapshot>` as it stood before the commit, so a joiner with
    /// no record can rebuild the trie the delta's `base_root` names
    /// (LLR-bg3vsw); and the Organisation private key of the epoch the update
    /// reaches (REQ-szq3ud). A body without either does not decode
    /// (REQ-c29s93).
    OrgInformation { envelope: Envelope, record_snapshot: Vec<u8>, org_private_key: OrgPrivateKey },
    /// Index 1: the committed Envelope alone — no snapshot, no key.
    Revocation { envelope: Envelope },
}

impl WireMessage {
    /// The Envelope of either kind (LLR-js9dsu).
    pub fn envelope(&self) -> &Envelope {
        match self {
            Self::OrgInformation { envelope, .. } | Self::Revocation { envelope } => envelope,
        }
    }
}
```

`org-node/src/service.rs`:
- Replace `first_admission_base` and `encode_record_snapshot` (docs through
  closing braces) by:

```rust
/// The record a first admission extends, decoded from the snapshot its
/// Organisation-information message carries (REQ-d9g6nt, LLR-j6j95z).
// Public only for the fuzz target `fuzz_first_admission_base`; not API.
#[doc(hidden)]
pub fn first_admission_base(record_snapshot: &[u8]) -> Result<Trie, OrgNodeError> {
    let raw: Vec<RawMemberSnapshot> = postcard::from_bytes(record_snapshot)
        .map_err(|e| OrgNodeError::Chain(format!("record snapshot decode: {e}")))?;
    let snaps = raw.into_iter().map(MemberSnapshot::try_from).collect::<Result<Vec<_>, _>>()?;
    trie_from_snapshots(&snaps)
}

/// Encode the record a committed update extends, as an
/// Organisation-information message's `record_snapshot`;
/// `first_admission_base` decodes it.
fn encode_record_snapshot(snapshots: &[MemberSnapshot]) -> Result<Vec<u8>, OrgNodeError> {
    postcard::to_allocvec(snapshots)
        .map_err(|e| OrgNodeError::Chain(format!("record snapshot encode: {e}")))
}
```

- Replace `send_update`'s doc comment and its body down to and including
  the `let msg = WireMessage { … };` statement by:

```rust
    /// Send a committed update to `recipient`'s device, from the device of
    /// the first Persona bound to its Organisation (LLR-2xzys9). The kind
    /// follows the recipient (LLR-6ymd6d): a Device the node's record of the
    /// Organisation lists receives Organisation information — the Envelope,
    /// the record as it stood before the commit, and the Organisation
    /// private key that record holds, none taken from the caller
    /// (REQ-szq3ud); any other Device a revocation, the Envelope alone
    /// (REQ-3dsweu, LLR-8hdu9x). No invite identifier (LLR-48jakr). No
    /// record: `OrgNotOnChain`, nothing sent. Loopback dials `peer_addr` and
    /// refuses without one, before binding (LLR-jn5jeh, LLR-pw369n);
    /// Networked dials by `recipient`. Writes nothing (LLR-t4znbk).
    pub async fn send_update(
        &mut self,
        outgoing: &OutgoingUpdate,
        recipient: DevicePublicKey,
        peer_addr: Option<iroh::EndpointAddr>,
    ) -> Result<(), OrgNodeError> {
        let mode = self.transport_mode;
        let rec = self.find_org(outgoing.envelope.org_id)?;
        let msg = if rec.trie_members.iter().any(|m| m.device_keys.contains(&recipient)) {
            WireMessage::OrgInformation {
                envelope: outgoing.envelope.clone(),
                record_snapshot: outgoing.record_snapshot.clone(),
                org_private_key: rec
                    .org_private_key
                    .clone()
                    .ok_or_else(|| OrgNodeError::Chain("the record holds no Organisation private key".into()))?,
            }
        } else {
            WireMessage::Revocation { envelope: outgoing.envelope.clone() }
        };
        let persona_id = self.first_persona_bound_to(outgoing.envelope.org_id)?.persona_id.clone();
        let loopback_addr = match (mode, peer_addr) {
            (TransportMode::Loopback, None) => {
                return Err(OrgNodeError::Chain("Loopback send requires the peer's EndpointAddr".into()));
            }
            (TransportMode::Loopback, Some(addr)) => Some(addr),
            (TransportMode::Networked, _) => None,
        };
```

- In `receive_and_verify`, replace the lines from
  `        let msg = self.receive_one().await?;` down to and including the
  closing `};` of `let (local_trie, last_seq, last_epoch) = match &existing { … };`
  by:

```rust
        let msg = self.receive_one().await?;
        let envelope = msg.envelope().clone();
        let org_id = envelope.org_id;

        // The record this message extends. Nothing about the sender is
        // checked and no Invite is required (REQ-xa6smf, REQ-ztdza4, owner
        // ruling 2026-10-05): the chain decides.
        let existing = self.store.data().orgs.iter().find(|o| o.org_id == org_id).cloned();
        let is_first_admission = existing.is_none();
        // What the message carries besides its Envelope (LLR-js9dsu). A
        // revocation about an Organisation not held is refused before the
        // expectations are consulted or the chain is read (LLR-38e2kn).
        let (record_snapshot, carried_key) = match msg {
            WireMessage::OrgInformation { record_snapshot, org_private_key, .. } => {
                (Some(record_snapshot), Some(org_private_key))
            }
            WireMessage::Revocation { .. } if is_first_admission => {
                return Err(OrgNodeError::RevocationNotHeld { org_id });
            }
            WireMessage::Revocation { .. } => (None, None),
        };
        // A first admission is read only when the app expects one to this
        // Organisation — before its snapshot is decoded or the chain is read
        // (LLR-s8xp7m, RC-2ferct).
        let expectation = ExpectedAdmission { org_id };
        if is_first_admission && !self.store.data().expected_admissions.contains(&expectation) {
            return Err(OrgNodeError::AdmissionNotExpected { org_id });
        }
        let (local_trie, last_seq, last_epoch) = match (&existing, &record_snapshot) {
            (Some(rec), _) => (trie_from_snapshots(&rec.trie_members)?, rec.last_seq, rec.epoch),
            // The record a first admission extends: from the snapshot its
            // Organisation-information message carries (REQ-d9g6nt,
            // LLR-j6j95z).
            (None, Some(snapshot)) => (first_admission_base(snapshot)?, SequenceNumber::new(0), Epoch::new(0)),
            (None, None) => return Err(OrgNodeError::RevocationNotHeld { org_id }),
        };
```

  then replace `self.verify_received(&local_trie, &msg.envelope, &ctx)` by
  `self.verify_received(&local_trie, &envelope, &ctx)`; in the
  first-admission branch replace `        if is_first_admission {` and the line
  `            let data = self.store.data_mut();` by

```rust
        if is_first_admission {
            let Some(org_private_key) = carried_key else {
                return Err(OrgNodeError::RevocationNotHeld { org_id });
            };
            let data = self.store.data_mut();
```

  and `                org_private_key: msg.org_private_key.clone(),` by
  `                org_private_key: Some(org_private_key),`; replace

```rust
            // A committed received update stores the key it carried
            // (REQ-ju6vn2, LLR-ckk5nz).
            self.commit_held(org_id, &verified)?;
            Self::store_carried_key(self.store.data_mut(), org_id, msg.org_private_key.clone())?;
```

  by

```rust
            // A committed received update stores the key it carried
            // (REQ-ju6vn2, LLR-ckk5nz); a revocation carries none.
            self.commit_held(org_id, &verified)?;
            if let Some(key) = carried_key {
                Self::store_carried_key(self.store.data_mut(), org_id, Some(key))?;
            }
```

- In `receive_and_self_delete_if_revoked`, replace
  `        let org_id = msg.envelope.org_id;` by
  `        let envelope = msg.envelope().clone();` and
  `        let org_id = envelope.org_id;`, and
  `self.verify_received(&local_trie, &msg.envelope, &ctx)` by
  `self.verify_received(&local_trie, &envelope, &ctx)`.

**Step 4 — green.**
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test wire_frame_bound --test fuzz_wire_decode --test encoding_golden --test secret_redaction --test commit_paths --test admission_sender`
→ wire_frame_bound `7 passed`, fuzz_wire_decode exits 0, encoding_golden
`4 passed`, secret_redaction `9 passed`, commit_paths `28 passed`,
admission_sender `47 passed`, 0 failed. Full org-node cargo line (now with
`--test fuzz_wire_decode`) → every target `ok`, 0 failed;
`cargo check --all-targets` and `clippy` → `Finished`. App: still fails
(expected until T10).

**Step 5 — commit.** `git add` the files of **Files touched**, then
`git -c commit.gpgsign=false commit -m "T4: the Wire message has two kinds; the kind follows the recipient" -m "verifies: LLR-js9dsu, LLR-ecxc76, LLR-6ymd6d, LLR-8hdu9x, LLR-bg3vsw, LLR-38e2kn, LLR-j6j95z, REQ-3dsweu, REQ-vxqc5g, REQ-c29s93"`.

**Red→green attestations:**
- the_two_kinds_round_trip_and_a_body_of_neither_kind_is_refused — red -> green: watched fail (wire_frame_bound did not compile: E0223 `WireMessage::OrgInformation` on a struct, E0599 no method `envelope`) before the enum existed; green after.
- an_organisation_information_body_without_its_snapshot_or_key_does_not_decode — red -> green: same compile red of wire_frame_bound before the enum; green after.
- fuzz_wire_decode (new target) — red -> green: watched fail (E0223 on `WireMessage::Revocation` / `OrgInformation`) before the enum; exits 0 after.
- a_wire_message_of_either_kind_never_renders_the_key — red -> green: watched fail (secret_redaction did not compile: E0223/E0599, no two kinds, no `envelope()`); green after.
- send_update_sends_organisation_information_to_a_listed_device_and_a_revocation_to_any_other (rewritten, renamed) — red -> green: watched fail (commit_paths: 16 errors, E0223, no OrgInformation/Revocation kinds); green after.
- a_revocation_about_an_organisation_not_held_is_refused_before_the_chain (rewritten, renamed) — red -> green: watched fail (admission_sender: 27 errors, no Revocation kind / `envelope()`); green after.
- a_revocation_reaches_the_administrators_disk (re-asserted) — red -> green: same admission_sender red (no Revocation kind to match); green after.
- organisation_information_wire_message_is_pinned (renamed), revocation_wire_message_is_pinned (new), truncated_pinned_values_are_refused — red -> green: watched fail (encoding_golden did not compile: no `WireMessage::OrgInformation`, `first_admission_base` took `Option<&[u8]>`); green after. Derived literals match the plan's byte for byte (316 and 169 bytes).
- an_expected_first_admission_decodes_its_snapshot_before_reading_the_chain (re-asserted) — red -> green: watched fail (expected_admission: 11 errors, `with_snapshot` matches kinds that did not exist); green after. Task done 2026-10-06, merged from worktree-org-node-org-key-pair-t4 (2b96c0e); org-node 233 passed, 0 failed, 4 bolero targets exit 0, quint typechecks and 5 invariants clean. Adapted in execution: rustc reports E0223 rather than E0599 for the missing variants; two `&got.envelope` reads (transport_handshake.rs:127, transport_networked.rs:155) became `got.envelope()`; `clippy::panic` added to the allow lists of encoding_golden.rs and admission_sender.rs, and `clippy::expect_used, clippy::panic` to fuzz_wire_decode, following the test-file convention.

---

### T5 — org-node: a revocation is accepted only as the node's own removal

**Files touched:** `org-node/src/service.rs`,
`org-node/tests/admission_sender.rs`, `org-node/tests/service_stories.rs`,
`org-node/docs/problems/2026-10-04-secret-cleared-on-update.md`
**Parallel:** no (serial, after T4)
**IDs verified:** LLR-pt32fx, LLR-jsx922, LLR-u6rq4s, REQ-3dsweu (receive
half); resolves PR-xwek5e.
**Size:** ~150 lines.

**Step 1 — failing test** (`org-node/tests/admission_sender.rs`, after
`a_removal_relayed_by_the_member_it_removes_is_committed`):

```rust
// The owner's ruling on relabelling (REQ-3dsweu, LLR-pt32fx): a revocation is
// accepted only as the receiving Device's own removal. A relay takes the
// Organisation information A sends about C's admission and relabels it a
// revocation; B, whose Device the verified record still lists, refuses it on
// either receive path — its record and both keys, its Personas and its
// provisional updates unchanged, nothing written — and then commits the
// genuine message.
// verifies: LLR-pt32fx, LLR-jsx922, LLR-u6rq4s, REQ-3dsweu
#[tokio::test(flavor = "multi_thread")]
async fn a_revocation_that_leaves_this_device_listed_is_refused_on_both_paths() {
    let mut s = admit_b_directly(setup("relabelled").await).await;
    let dora = org_node::Joiner {
        handle: h("dora"),
        name: nm("Dora"),
        surname: sn("Diver"),
        member_key: org_node::test_fixtures::member_key(0x71),
        device_key: org_node::test_fixtures::device_key(0x72),
    };
    s.svc_b.admit_member(&mut OsRng, s.org_id, &dora).expect("B keeps a provisional update");
    let genuine = captured_admission_of_c(&mut s).await;
    let relabelled = WireMessage::Revocation { envelope: genuine.envelope().clone() };
    let before = rec_of(&s.svc_b, s.org_id);
    let pending = s.svc_b.provisional_updates(s.org_id);
    let b_binding = binding(&persona_of(&s.svc_b, &s.pid_b));
    let on_disk = store_bytes("relabelled", "b");

    let (addr, task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(addr, &relabelled).await;
    let (svc_b, result) = task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::RevocationNotForThisDevice { org_id: s.org_id });
    let (addr, task) = spawn_self_delete(svc_b, &s.b_device_kp).await;
    deliver(addr, &relabelled).await;
    let (svc_b, result) = task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::RevocationNotForThisDevice { org_id: s.org_id });

    let after = rec_of(&svc_b, s.org_id);
    assert_eq!(
        (after.epoch, after.last_seq, after.root_hash, after.org_pub_key),
        (before.epoch, before.last_seq, before.root_hash, before.org_pub_key)
    );
    assert_eq!(private_key_held(&after), private_key_held(&before), "the key is kept");
    assert_eq!(svc_b.provisional_updates(s.org_id), pending, "provisional updates kept");
    assert_eq!(binding(&persona_of(&svc_b, &s.pid_b)), b_binding, "the Persona as it was");
    assert_eq!(store_bytes("relabelled", "b"), on_disk, "nothing written");

    let (addr, task) = spawn_receive(svc_b, &s.b_device_kp).await;
    deliver(addr, &genuine).await;
    let (svc_b, result) = task.await.unwrap();
    assert_eq!(result.expect("the genuine message commits").epoch, Epoch::new(3));
    assert_eq!(rec_of(&svc_b, s.org_id).trie_members.len(), 3, "A + B + C");
}
```

Rewrite `a_removal_relayed_by_the_member_it_removes_is_committed`: replace its
comment block

```rust
// LLR-u6rq4s as amended 2026-10-05: the delivering device is compared with no
// Membership record. C relays its own removal to B on the ordinary receive
// path; C is in B's record before the change and not after it, and B commits
// the change because it matches the chain.
```

by

```rust
// LLR-u6rq4s as amended 2026-10-06: the delivering device is compared with no
// Membership record. A removes C and sends the Organisation information about
// it to C's endpoint — addressed to B's Device, which the committed record
// lists — and C relays it to B on the ordinary receive path; C is in B's
// record before the change and not after it, and B commits the change because
// it matches the chain. (A revocation C relayed would be refused by B, whose
// Device is still listed: `a_revocation_that_leaves_this_device_listed_is_refused_on_both_paths`.)
```

and replace

```rust
    // A revokes C and pushes the change to C's own endpoint.
    let (c_addr, c_task) = spawn_recv_one(c_seed).await;
    revoke(&mut s.svc_a, &s.chain, s.org_id, c_id, Some(c_addr))
        .await
        .expect("revoke C failed");
    let (c_ep, _sender, msg) = c_task.await.unwrap();
```

by

```rust
    // A revokes C and pushes the change, addressed to B's Device, to C's own
    // endpoint.
    let (c_addr, c_task) = spawn_recv_one(c_seed).await;
    revoke_and_tell(&mut s.svc_a, &s.chain, s.org_id, c_id, s.b_device_kp.device_key().unwrap(), c_addr)
        .await
        .expect("revoke C failed");
    let (c_ep, _sender, msg) = c_task.await.unwrap();
    assert!(matches!(msg, WireMessage::OrgInformation { .. }), "B's Device is listed");
```

`org-node/tests/service_stories.rs`, `revocation_of_another_member_is_committed_not_self_deleted`:
add `revoke_and_tell, ` to the `use support::{…}` list; replace

```rust
    revoke(&mut svc_a, &chain, org_id, c_member_id, Some(b_addr))
        .await
        .expect("revoke_member(C) failed");
```

by

```rust
    // Told to B's Device, which the committed record lists: Organisation
    // information (REQ-3dsweu).
    revoke_and_tell(&mut svc_a, &chain, org_id, c_member_id, b_device_kp.device_key().unwrap(), b_addr)
        .await
        .expect("revoke_member(C) failed");
```

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test admission_sender a_revocation_that_leaves_this_device_listed`.
Expected red (runtime): ``called `Result::unwrap_err()` on an `Ok` value:
ReceiveOutcome { … epoch: Epoch(3) … }`` — the relabelled message is
committed.

**Step 2 — implementation** (`org-node/src/service.rs`).
- In `receive_and_verify`, replace

```rust
        } else if self.still_member(org_id, &verified.trie) {
            // A committed received update stores the key it carried
            // (REQ-ju6vn2, LLR-ckk5nz); a revocation carries none.
            self.commit_held(org_id, &verified)?;
            if let Some(key) = carried_key {
                Self::store_carried_key(self.store.data_mut(), org_id, Some(key))?;
            }
```

  by

```rust
        } else if self.still_member(org_id, &verified.trie) {
            // A revocation is accepted only as this node's own removal
            // (LLR-pt32fx, REQ-3dsweu): one that leaves this node's Device
            // listed is refused, nothing written.
            let Some(key) = carried_key else {
                return Err(OrgNodeError::RevocationNotForThisDevice { org_id });
            };
            // A committed received update stores the key it carried
            // (REQ-ju6vn2, LLR-ckk5nz).
            self.commit_held(org_id, &verified)?;
            Self::store_carried_key(self.store.data_mut(), org_id, Some(key))?;
```

- In `receive_and_self_delete_if_revoked`, after
  `        let org_id = envelope.org_id;` add
  `        let is_revocation = matches!(msg, WireMessage::Revocation { .. });`,
  and replace

```rust
        if self.still_member(org_id, &verified.trie) {
            // Regular admit/update — commit the update normally; this path
            // keeps the key the record holds.
```

  by

```rust
        if self.still_member(org_id, &verified.trie) {
            // A revocation that leaves this node's Device listed is refused,
            // nothing written (LLR-pt32fx).
            if is_revocation {
                return Err(OrgNodeError::RevocationNotForThisDevice { org_id });
            }
            // Regular admit/update — commit the update normally; this path
            // keeps the key the record holds.
```

**Step 3 — resolve PR-xwek5e**
(`org-node/docs/problems/2026-10-04-secret-cleared-on-update.md`). Replace the
line `status: open` under `**PR-xwek5e**` by the two lines:

```
status: resolved
resolution: the Organisation secret is replaced by the Organisation private key, which every Organisation-information message carries (REQ-szq3ud) and a revocation — sent only to a Device the committed record no longer lists — never does (REQ-3dsweu), and a revocation is accepted only as the receiver's own removal (LLR-pt32fx); reproduced by `pr_xwek5e_another_members_removal_leaves_the_receiver_holding_the_organisations_key` and `a_revocation_that_leaves_this_device_listed_is_refused_on_both_paths` (org-node/tests/admission_sender.rs), red before, green after.
```

Check: `GR_CONFIG=org-node/.guardrails/config.yaml sh .guardrails/scripts/find-items.sh show PR-xwek5e`
prints `status: resolved` and the resolution.

**Step 4 — green.**
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test admission_sender --test service_stories`
→ admission_sender `48 passed`, service_stories `3 passed`, 0 failed. Full
org-node cargo line → every target `ok`, 0 failed; `cargo check
--all-targets` and `clippy` → `Finished`.

**Step 5 — commit.** `git add` the files of **Files touched**, then
`git -c commit.gpgsign=false commit -m "T5: a revocation is accepted only as the node's own removal (PR-xwek5e)" -m "verifies: LLR-pt32fx, LLR-jsx922, LLR-u6rq4s, REQ-3dsweu; resolves PR-xwek5e"`.

**Red→green attestations:**
- a_revocation_that_leaves_this_device_listed_is_refused_on_both_paths — red -> green: watched fail before the implementation ("called `Result::unwrap_err()` on an `Ok` value: ReceiveOutcome { … epoch: Epoch(3) … }" — the relabelled message was committed); green after.
- a_removal_relayed_by_the_member_it_removes_is_committed (rewritten) — red -> green: the test asserts behaviour that changes rather than a missing feature, so its red was shown by running the pre-rewrite text against the new service.rs: it failed with "a chain-valid removal is committed whoever relays it: RevocationNotForThisDevice { … }"; the rewritten text passes.
- revocation_of_another_member_is_committed_not_self_deleted (rewritten) — red -> green: as above: the pre-rewrite text against the new service.rs failed with "B must verify and commit C's revocation: RevocationNotForThisDevice { … }"; the rewritten text passes. Task done 2026-10-06, merged from worktree-org-node-org-key-pair-t5 (3adbdb3); org-node 234 passed, 0 failed; PR-xwek5e resolved in its dated file.

---

### T6 — org-node: a message that does not decode is refused with a typed error

**Files touched:** `org-node/src/service.rs`, `org-node/tests/support/mod.rs`,
`org-node/tests/receive_chain_reads.rs`
**Parallel:** no (serial, after T5)
**IDs verified:** LLR-xn5pwc, LLR-j5vbqj, REQ-c29s93 (receive half).
**Size:** ~110 lines.

**Step 1 — test helper** (`org-node/tests/support/mod.rs`, after `deliver`):

```rust
/// Send `body`, framed as `encode_frame` frames a message, to `addr` from a
/// relay device: bytes no `WireMessage` encodes to.
pub async fn deliver_raw(addr: iroh::EndpointAddr, body: &[u8]) {
    let relay = OrgEndpoint::bind(&DeviceSeed::from([0x5cu8; 32]).signing_keypair()).await.unwrap();
    let conn = relay.inner().connect(addr, org_node::transport::ALPN).await.expect("connect");
    let (mut send, _recv) = conn.open_bi().await.expect("open_bi");
    let mut framed = (body.len() as u32).to_le_bytes().to_vec();
    framed.extend_from_slice(body);
    send.write_all(&framed).await.expect("write");
    send.finish().expect("finish");
    // Hold the connection until the receiver has read the stream.
    let _ = tokio::time::timeout(NET, send.stopped()).await;
}
```

**Step 2 — failing tests** (`org-node/tests/receive_chain_reads.rs`). Add
`use org_node::transport::wire::{encode_frame, WireMessage};` after the
existing `use org_node::test_fixtures::admit_member_delta;`, and at the end of
the file:

```rust
/// Bodies of Organisation information that does not decode, from `genuine`:
/// its key cut off, its key one byte short, and no snapshot and no key.
fn keyless_bodies(genuine: &WireMessage) -> [Vec<u8>; 3] {
    let body = encode_frame(genuine).unwrap()[4..].to_vec();
    let mut no_snapshot = encode_frame(&WireMessage::Revocation { envelope: genuine.envelope().clone() }).unwrap()[4..].to_vec();
    no_snapshot[0] = 0;
    [body[..body.len() - 32].to_vec(), body[..body.len() - 1].to_vec(), no_snapshot]
}

// LLR-xn5pwc, REQ-c29s93: Organisation information without its Organisation
// private key, or with a key short of 32 bytes, does not decode, and both
// receive paths refuse it with the typed error before any record is consulted
// and with no chain read; the record is untouched and nothing is written.
// verifies: LLR-xn5pwc, LLR-j5vbqj, REQ-c29s93
#[tokio::test(flavor = "multi_thread")]
async fn a_message_without_its_organisation_private_key_is_refused_before_the_chain() {
    let (mut s, counting) = admitted("keyless").await;
    let genuine = captured_admission_of_c(&mut s).await;
    let held = rec_of(&s.svc_b, s.org_id);
    let on_disk = store_bytes("keyless", "b");
    let before = counting.reads();
    let mut svc_b = s.svc_b;
    for raw in keyless_bodies(&genuine) {
        let (addr, task) = spawn_receive(svc_b, &s.b_device_kp).await;
        deliver_raw(addr, &raw).await;
        let (back, result) = task.await.unwrap();
        assert_eq!(result.unwrap_err(), OrgNodeError::MalformedMessage, "{} bytes", raw.len());
        let (addr, task) = spawn_self_delete(back, &s.b_device_kp).await;
        deliver_raw(addr, &raw).await;
        let (back, result) = task.await.unwrap();
        assert_eq!(result.unwrap_err(), OrgNodeError::MalformedMessage, "{} bytes", raw.len());
        svc_b = back;
    }
    assert_eq!(counting.reads(), before, "no chain read");
    let after = rec_of(&svc_b, s.org_id);
    assert_eq!((after.epoch, after.root_hash, after.last_seq), (held.epoch, held.root_hash, held.last_seq));
    assert_eq!(store_bytes("keyless", "b"), on_disk, "nothing written");
}

// The same refusal on a first admission: before the expectations are
// consulted — the expectation stays — and with no chain read.
// verifies: LLR-xn5pwc, REQ-c29s93
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_without_its_key_is_refused_and_keeps_the_expectation() {
    let (mut s, counting) = setup_counted("keyless-first").await;
    let joiner = s.joiner_b.clone();
    let genuine = captured_admission(&mut s, &joiner).await;
    let on_disk = store_bytes("keyless-first", "b");
    let before = counting.reads();
    let mut svc_b = s.svc_b;
    for raw in keyless_bodies(&genuine) {
        let (addr, task) = spawn_receive(svc_b, &s.b_device_kp).await;
        deliver_raw(addr, &raw).await;
        let (back, result) = task.await.unwrap();
        assert_eq!(result.unwrap_err(), OrgNodeError::MalformedMessage, "{} bytes", raw.len());
        svc_b = back;
    }
    assert_eq!(counting.reads(), before, "no chain read");
    assert!(svc_b.list_orgs().is_empty());
    assert_eq!(svc_b.expected_admissions().len(), 1, "the expectation is kept");
    assert_eq!(store_bytes("keyless-first", "b"), on_disk, "nothing written");
}
```

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test receive_chain_reads`.
Expected red (runtime): `left: Chain("iroh recv: malformed wire message")`,
`right: MalformedMessage`.

**Step 3 — implementation** (`org-node/src/service.rs`). Replace
`use crate::transport::TransportMode;` by
`use crate::transport::{TransportError, TransportMode};`, and in
`receive_one` replace

```rust
        let (_sender, msg) = ep.recv_one().await.map_err(|e| OrgNodeError::Chain(format!("iroh recv: {e}")))?;
```

by

```rust
        // A body that does not decode — Organisation information without its
        // snapshot or key among them — is refused as such, before any record,
        // expectation or chain is consulted (LLR-xn5pwc, REQ-c29s93).
        let (_sender, msg) = ep.recv_one().await.map_err(|e| match e {
            TransportError::Malformed => OrgNodeError::MalformedMessage,
            other => OrgNodeError::Chain(format!("iroh recv: {other}")),
        })?;
```

and add to `receive_one`'s doc comment the sentence
`/// A body that does not decode is \`MalformedMessage\` (LLR-xn5pwc).`

**Step 4 — green.**
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test receive_chain_reads`
→ `9 passed`, 0 failed. Full org-node cargo line → every target `ok`, 0
failed (transport_handshake 6, whose `the_length_prefix_is_not_checked_against_the_body`
calls `recv_one` directly and still sees `TransportError::Malformed`);
`cargo check --all-targets` and `clippy` → `Finished`.

**Step 5 — commit.** `git add org-node/src/service.rs org-node/tests/support/mod.rs org-node/tests/receive_chain_reads.rs`,
then `git -c commit.gpgsign=false commit -m "T6: a message that does not decode is refused with a typed error" -m "verifies: LLR-xn5pwc, LLR-j5vbqj, REQ-c29s93"`.

**Red→green attestations:**
- a_message_without_its_organisation_private_key_is_refused_before_the_chain — red -> green: watched fail before the implementation (left: Chain("iroh recv: malformed wire message"), right: MalformedMessage; 403-byte body with its key cut off); green after the receive_one mapping.
- a_first_admission_without_its_key_is_refused_and_keeps_the_expectation — red -> green: watched fail before the implementation (left: Chain("iroh recv: malformed wire message"), right: MalformedMessage; 287 bytes); green after. Task done 2026-10-06, merged from worktree-org-node-org-key-pair-t6 (634fca0); org-node 236 passed, 0 failed.

---

### T7 — org-node: every record holds the Organisation private key

**Files touched:** `org-node/src/store.rs`, `org-node/src/service.rs`,
`org-node/tests/support/mod.rs`, `org-node/tests/commit_paths.rs`,
`org-node/tests/encoding_golden.rs`, `org-node/tests/organisation_key.rs`,
`org-node/tests/persona_records.rs`, `org-node/tests/secret_redaction.rs`,
`org-node/tests/store_at_rest.rs`
**Parallel:** no (serial, after T6)
**IDs verified:** LLR-byjvd9, LLR-2dvhz8, LLR-3fwykc, LLR-g76zqd,
LLR-ayrdr8, LLR-wzqqg9, REQ-ju6vn2, REQ-ech45n.
**Size:** ~180 lines.

**Step 1 — failing tests.**

`org-node/tests/persona_records.rs`:
- In `org_with_member`, replace `        org_private_key: None,` by
  `        org_private_key: OrgPrivateKey::from([0x5d; 32]),`.
- In `struct WireOrg`, replace `    org_private_key: Option<OrgPrivateKey>,` by
  `    org_private_key: OrgPrivateKey,`; in `wire_store`, replace
  `            org_private_key: None,` by
  `            org_private_key: OrgPrivateKey::from([0x5e; 32]),`.
- After `a_store_with_every_record_kind_opens_with_every_field_parsed`, add:

```rust
// LLR-byjvd9: every record carries the Organisation private key, encoded as
// its plain 32 bytes with no option tag, and it is read back. A store written
// before the change worktree-org-node-org-key-pair — the record ending in an
// option tag and no key — is refused, not read or migrated.
// verifies: LLR-byjvd9
#[test]
fn a_record_carries_its_organisation_private_key_and_a_store_without_one_is_refused() {
    let data = StoreData {
        personas: vec![],
        orgs: vec![org_with_member()],
        provisional_updates: vec![],
        expected_admissions: vec![],
    };
    let plaintext = postcard::to_allocvec(&data).unwrap();
    let key = org_with_member().org_private_key;
    // The record's last 32 bytes, before the two empty lists, are the key.
    assert_eq!(&plaintext[plaintext.len() - 34..plaintext.len() - 2], key.expose_secret(), "no option tag");
    let path = tmp_path("with-key");
    store::seal_for_test(&path, "pw", &plaintext, &mut OsRng).unwrap();
    assert_eq!(PersonaStore::open(path, "pw").unwrap().data().orgs[0].org_private_key, key);

    let mut legacy = plaintext[..plaintext.len() - 34].to_vec();
    legacy.extend_from_slice(&[0x00, 0x00, 0x00]); // `org_private_key: None`, then the two empty lists
    let path = tmp_path("without-key");
    store::seal_for_test(&path, "pw", &legacy, &mut OsRng).unwrap();
    assert!(PersonaStore::open(path, "pw").is_err(), "a record without the key is not read");
}
```

`org-node/tests/secret_redaction.rs`:
- In `org`, replace `        org_private_key: Some(OrgPrivateKey::from(private)),`
  by `        org_private_key: OrgPrivateKey::from(private),`.
- Replace the whole test `a_record_debug_says_whether_the_organisation_private_key_is_set`
  (its comment block through its closing brace) by:

```rust
// LLR-2dvhz8 as amended: the field is no longer optional, so a record renders
// it as its redacted secret type, never its bytes, whatever they are.
// *Rewritten 2026-10-06 (change worktree-org-node-org-key-pair).* Was
// `a_record_debug_says_whether_the_organisation_private_key_is_set`.
/// verifies: LLR-2dvhz8
#[test]
fn a_record_debug_renders_the_organisation_private_key_redacted() {
    for private in [org_private(), [0u8; 32], sentinel_down(0xff)] {
        let record = org(private);
        for rendered in [format!("{record:?}"), format!("{record:#?}")] {
            let squashed: String = rendered.chars().filter(|c| !c.is_whitespace()).collect();
            assert!(squashed.contains("org_private_key:OrgPrivateKey([REDACTED])"), "{rendered}");
            if private != [0u8; 32] {
                assert_not_rendered(&rendered, &private, "Organisation private key");
            }
        }
    }
}
```

`org-node/tests/store_at_rest.rs`: in `org_record`, replace
`        org_private_key: Some(OrgPrivateKey::from(key)),` by
`        org_private_key: OrgPrivateKey::from(key),`.

`org-node/tests/commit_paths.rs`: in
`commit_genesis_creates_the_record_once_the_chain_carries_the_root`, replace
`    assert_eq!(rec.org_private_key, Some(private_key_of(&update)));` by
`    assert_eq!(rec.org_private_key, private_key_of(&update));`.

`org-node/tests/organisation_key.rs`:
- In `a_created_organisation_publishes_a_fresh_key_no_genesis_key_equals`,
  replace `    let secret = rec.org_private_key.expect("the creator holds the Organisation private key");`
  by `    let secret = rec.org_private_key;`.
- In `the_organisation_private_key_is_kept_only_in_the_encrypted_store`,
  replace `    assert_eq!(svc.list_orgs()[0].org_private_key, Some(secret.clone()));`
  by `    assert_eq!(svc.list_orgs()[0].org_private_key, secret.clone());` and
  `    assert_eq!(rec.org_private_key, Some(secret.clone()), "the store holds the Organisation private key");`
  by `    assert_eq!(rec.org_private_key, secret.clone(), "the store holds the Organisation private key");`.
- In `the_organisation_private_key_is_not_in_the_record_debug_output`,
  replace `    let secret = rec.org_private_key.clone().unwrap();` by
  `    let secret = rec.org_private_key.clone();`.
- Module doc: replace `(REQ-ech45n); no other node's record holds it today (LLR-3fwykc).`
  by `(REQ-ech45n), and every Member's record holds it once admitted (LLR-3fwykc).`

`org-node/tests/encoding_golden.rs`:
- After the T4 paragraph add a `// GOLDEN_STORE  = "…"` comment line holding
  the current (T3) store value, and:

```rust
//
// Re-pinned 2026-10-06 (change worktree-org-node-org-key-pair, T7): the
// record's Organisation private key is required (LLR-byjvd9): the byte `00`
// (`org_private_key: None`) at offset 493 becomes `44`x32, the key with no
// option tag. Derived by the format rule, never captured from the code.
```

- Replace the `GOLDEN_STORE` value by:

```
0107702d616c69636501aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa05616c69636505416c69636505536d697468111111111111111111111111111111111111111111111111111111111111111112121212121212121212121212121212121212121212121212121212121212120101010101010101010101010101010101010101010101010101010101010101010101aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa3333333333333333333333333333333333333333333333333333333333333333d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737030202010101010101010101010101010101010101010101010101010101010101010105616c69636505416c69636505536d697468d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21020202020202020202020202020202020202020202020202020202020202020203626f6203426f62054a6f6e6573884b8857f4eaa1613c61504db34d4beaf346517a0e31de3cddd4d9b4201d9d0b01a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f00155555555555555555555555555555555555555555555555555555555555555554444444444444444444444444444444444444444444444444444444444444444020007702d616c69636500777777777777777777777777777777777777777777777777777777777777777701d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c97787370001010101010101010101010101010101010101010101010101010101010101010105616c69636505416c69636505536d697468d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21888888888888888888888888888888888888888888888888888888888888888801bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb07702d616c696365013333333333333333333333333333333333333333333333333333333333333333666666666666666666666666666666666666666666666666666666666666666603d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701040908070601cccccccccccccccccccccccccccccccccccccccc
```

- In `persona_store_plaintext_is_pinned`, replace
  `    assert_eq!(org["org_private_key"], Value::Null, "this branch's field, absent on a pinned record");`
  by `    assert_eq!(org["org_private_key"][0], json!(0x44), "every record holds the key (LLR-byjvd9)");`;
  then delete `Value, ` from `use serde_json::{json, Value};` if
  `grep -n "Value" org-node/tests/encoding_golden.rs` shows no other use
  (`fn model` returns `Value`: keep the import).

`org-node/tests/support/mod.rs`: replace `private_key_held`'s body line
`    rec.org_private_key.clone().expect("the record holds the Organisation private key")`
by `    rec.org_private_key.clone()`.

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test persona_records --test secret_redaction`.
Expected red: compile errors E0308 (`expected Option<OrgPrivateKey>, found
OrgPrivateKey`).

**Step 2 — implementation.**

`org-node/src/store.rs`:
- In `pub struct OrgRecord`, replace `    pub org_private_key: Option<OrgPrivateKey>,`
  by `    pub org_private_key: OrgPrivateKey,` and its doc comment's first line
  `    /// The Organisation's X25519 private key (REQ-ech45n, REQ-szq3ud): the`
  by `    /// The Organisation's X25519 private key, required (LLR-byjvd9,`
  `    /// REQ-ech45n, REQ-szq3ud): the`.
- In `pub(crate) struct RawOrgRecord`, replace
  `    org_private_key: Option<OrgPrivateKey>,` by
  `    org_private_key: OrgPrivateKey,`.

`org-node/src/service.rs`:
- In `commit_genesis`, replace `            org_private_key: Some(org_private_key),`
  by `            org_private_key,`.
- In `send_update`, replace

```rust
                org_private_key: rec
                    .org_private_key
                    .clone()
                    .ok_or_else(|| OrgNodeError::Chain("the record holds no Organisation private key".into()))?,
```

  by `                org_private_key: rec.org_private_key.clone(),`.
- In `receive_and_verify`'s first-admission record, replace
  `                org_private_key: Some(org_private_key),` by
  `                org_private_key,`; in the held branch replace
  `            Self::store_carried_key(self.store.data_mut(), org_id, Some(key))?;`
  by `            Self::store_carried_key(self.store.data_mut(), org_id, key)?;`.
- In `store_carried_key`, replace the parameter
  `key: Option<OrgPrivateKey>` by `key: OrgPrivateKey`.

**Step 3 — green.**
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test persona_records --test secret_redaction --test encoding_golden --test store_at_rest --test organisation_key --test commit_paths`
→ persona_records `16 passed`, secret_redaction `9 passed`, encoding_golden
`4 passed`, store_at_rest `7 passed`, organisation_key `9 passed`,
commit_paths `28 passed`, 0 failed. Full org-node cargo line → every target
`ok`, 0 failed; `cargo check --all-targets` and `clippy` → `Finished`.

**Step 4 — commit.** `git add` the files of **Files touched**, then
`git -c commit.gpgsign=false commit -m "T7: every record holds the Organisation private key" -m "verifies: LLR-byjvd9, LLR-2dvhz8, LLR-3fwykc, LLR-ayrdr8"`.

**Red→green attestations:**
- a_record_carries_its_organisation_private_key_and_a_store_without_one_is_refused — red -> green: watched fail before the implementation (persona_records did not compile: E0308 expected `Option<OrgPrivateKey>`, found `OrgPrivateKey`; E0599 no `expose_secret` on `Option`); green after (16 passed).
- a_record_debug_renders_the_organisation_private_key_redacted (rewritten, renamed) — red -> green: watched fail before the implementation (E0308 at secret_redaction.rs:92, expected `Option<OrgPrivateKey>`); green after (9 passed).
- persona_store_plaintext_is_pinned (re-pinned) — red -> green: compiled against the old code and failed at run time (encoding_golden.rs:113, DeserializeBadOption: the old `Option` field read 0x44 at offset 493 as an option tag); green after. Literal matches the derivation (873 -> 904 bytes). Task done 2026-10-06, merged from worktree-org-node-org-key-pair-t7 (a1236ba); org-node 237 passed, 0 failed. One first full-line run failed 2 admission_sender tests on an iroh handshake ("invalid peer certificate: UnknownIssuer") and a send timeout; the target passed alone and on a full rerun — watch for it at the gate.

---

### T8 — org-node: a fresh Organisation key pair with every provisional update, taken on commit

**Files touched:** `org-node/src/store.rs`, `org-node/src/service.rs`,
`org-node/tests/support/mod.rs`, `org-node/tests/commit_paths.rs`,
`org-node/tests/organisation_key.rs`, `org-node/tests/provisional_store.rs`,
`org-node/tests/persona_records.rs`, `org-node/tests/encoding_golden.rs`,
`org-node/tests/admission_sender.rs`,
`org-node/docs/problems/2026-10-07-key-rotation.md`
**Parallel:** no (serial, after T7)
**IDs verified:** LLR-e2b7gv, LLR-6s785x, LLR-cmdrp9, LLR-ghja3x, LLR-tax3pm,
LLR-qjz3q4, LLR-95753m, LLR-7cmp38, LLR-sj7cd5, LLR-ayrdr8, REQ-stx9v3,
REQ-jy6ybw; reproduces PR-g9u3xq.
**Size:** ~380 lines. Two red-green cycles, one commit (the build without the
commit would leave every sender's record key behind the chain's, and every
test that checks a Member's key against the chain red).

**Step 1 — test helper** (`org-node/tests/support/mod.rs`): replace
`private_key_of` (doc through closing brace) by:

```rust
/// The Organisation private key a provisional update holds.
pub fn private_key_of(update: &ProvisionalUpdate) -> OrgPrivateKey {
    match &update.change {
        ProvisionalChange::Genesis { org_private_key, .. } | ProvisionalChange::ChangeSet { org_private_key, .. } => {
            org_private_key.clone()
        }
    }
}
```

**Step 2 — failing tests, cycle A (building).**

`org-node/tests/commit_paths.rs`:
- In `admit_member_keeps_a_provisional_update_and_changes_nothing_else`:
  replace the annotation `// verifies: REQ-xs4ab8, REQ-txvtm9, LLR-rb8r65, LLR-ghja3x, LLR-nvn3wk`
  by `// verifies: REQ-xs4ab8, REQ-txvtm9, REQ-stx9v3, LLR-rb8r65, LLR-ghja3x, LLR-e2b7gv, LLR-qjz3q4, LLR-nvn3wk`;
  replace the comment line `// Sequence number the epoch it produces, the record's key — and changes`
  by `// Sequence number the epoch it produces, a fresh key pair — and changes`;
  replace `    assert_eq!(update.org_pub_key, rec.org_pub_key);` by:

```rust
    assert_ne!(update.org_pub_key, rec.org_pub_key, "a fresh key pair, not the record's (REQ-stx9v3)");
    assert_eq!(
        private_key_of(&update).x25519_keypair().org_public_key().unwrap(),
        update.org_pub_key,
        "the update holds its private half"
    );
```

  after `    assert_eq!(rec_of(&a, org).trie_members.len(), rec.trie_members.len(), "record unchanged");`
  add:

```rust
    let now = rec_of(&a, org);
    assert_eq!(
        (now.org_pub_key, private_key_held(&now)),
        (rec.org_pub_key, private_key_held(&rec)),
        "building changes neither of the record's keys"
    );
```

  and after
  `    assert_eq!((second.base_root, second.seq), (update.base_root, update.seq), "built on the same record");`
  add `    assert_ne!(second.org_pub_key, update.org_pub_key, "each update draws its own pair");`.
- In `revoke_member_keeps_a_provisional_update_and_changes_nothing_else`:
  replace its annotation by
  `// verifies: REQ-xs4ab8, REQ-txvtm9, REQ-stx9v3, LLR-6dc598, LLR-tax3pm, LLR-e2b7gv`;
  replace the comment line `// produces, the record's key, no signature anywhere — and touches neither the`
  by `// produces, a fresh key pair, no signature anywhere — and touches neither the`;
  replace
  `    assert_eq!((update.seq, update.org_pub_key), (SequenceNumber::new(rec.epoch.get() + 1), rec.org_pub_key));`
  by:

```rust
    assert_eq!(update.seq, SequenceNumber::new(rec.epoch.get() + 1));
    assert_ne!(update.org_pub_key, rec.org_pub_key, "a fresh key pair (REQ-stx9v3)");
    assert_eq!(private_key_of(&update).x25519_keypair().org_public_key().unwrap(), update.org_pub_key);
```

- `an_admission_refused_by_the_bound_writes_nothing`: replace
  `        change: ProvisionalChange::ChangeSet { change_set: vec![0; org_node::store::MAX_PROVISIONAL_BYTES - 200] },`
  by
  `        change: ProvisionalChange::ChangeSet { change_set: vec![0; org_node::store::MAX_PROVISIONAL_BYTES - 200], org_private_key: org_node::OrgPrivateKey::from([0x5e; 32]) },`.
- Discard tests (LLR-7cmp38 as amended: the operation takes the key):
  - `discarding_a_genesis_update_removes_it_and_its_private_key_and_saves`:
    replace `ProvisionalTarget::Genesis(pid.clone()), update.resulting_root)` by
    `ProvisionalTarget::Genesis(pid.clone()), update.resulting_root, update.org_pub_key)`.
  - `discarding_one_of_two_updates_keeps_the_other_and_the_record`: replace
    `ProvisionalTarget::Org(org), first.resulting_root)` by
    `ProvisionalTarget::Org(org), first.resulting_root, first.org_pub_key)`.
  - `discarding_an_unknown_root_is_refused_and_writes_nothing`: replace the
    comment's first two lines
    `// Abnormal: a root no provisional update for the target produces — an unknown`
    `// root, or a known root under the wrong target — is refused with`
    by `// Abnormal: a root and key no provisional update for the target holds — an`
    `// unknown root, a known root under the wrong target or another update's key — is refused with`;
    replace the loop head and body down to its closing `}` by:

```rust
    for (target, root, key) in [
        (ProvisionalTarget::Org(org), RootHash::new([0xAB; 32]), first.org_pub_key),
        (ProvisionalTarget::Org(OrgId::new([0x99; 20])), first.resulting_root, first.org_pub_key),
        (ProvisionalTarget::Genesis(pid), first.resulting_root, first.org_pub_key),
        (ProvisionalTarget::Org(org), first.resulting_root, second.org_pub_key),
    ] {
        assert_eq!(a.discard_provisional(&mut OsRng, target, root, key).unwrap_err(), OrgNodeError::NoProvisionalUpdate);
        assert_eq!(a.provisional_updates(org), vec![first.clone(), second.clone()]);
        assert_eq!(store_bytes("discard-unknown", "a"), before, "nothing written");
    }
```

  - `a_genesis_update_is_discarded_only_under_its_own_persona`: replace
    `svc.discard_provisional(&mut OsRng, target, update.resulting_root).unwrap_err(),`
    by `svc.discard_provisional(&mut OsRng, target, update.resulting_root, update.org_pub_key).unwrap_err(),`.
- After `a_genesis_update_is_discarded_only_under_its_own_persona` add:

```rust
// LLR-95753m, LLR-7cmp38: two removals of the same Member have the same
// resulting root and differ only in their fresh key pairs. Both are kept, and
// discarding one by its root and key removes that one alone, with its private
// key, and saves.
// verifies: LLR-95753m, LLR-7cmp38, REQ-hhva9d
#[tokio::test(flavor = "multi_thread")]
async fn two_updates_for_the_same_change_are_two_and_one_is_discarded_alone() {
    let s = admit_b_directly(setup("same-change").await).await;
    let mut a = s.svc_a;
    let b_id = id_by_handle(&rec_of(&a, s.org_id), "bob");
    let first = a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    let second = a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    assert_eq!(first.resulting_root, second.resulting_root, "the same change");
    assert_ne!(first.org_pub_key, second.org_pub_key);
    assert_eq!(a.provisional_updates(s.org_id), vec![first.clone(), second.clone()], "two, not one replaced");
    a.discard_provisional(&mut OsRng, ProvisionalTarget::Org(s.org_id), first.resulting_root, first.org_pub_key).unwrap();
    assert_eq!(a.provisional_updates(s.org_id), vec![second.clone()]);
    assert_eq!(reopen_store("same-change", "a", "pw_a").data().provisional_updates, vec![second]);
}
```

`org-node/tests/organisation_key.rs`: after `impl CryptoRng for ConstRng {}`
add:

```rust
/// A random source that returns `first` for its first 32 bytes and `then`
/// after: a member id of `first` bytes, then an Organisation key pair drawn
/// from `then` bytes.
struct TwoRng {
    first: u8,
    then: u8,
    served: usize,
}

impl TwoRng {
    fn new(first: u8, then: u8) -> Self {
        Self { first, then, served: 0 }
    }
}

impl RngCore for TwoRng {
    fn next_u32(&mut self) -> u32 {
        let mut b = [0u8; 4];
        self.fill_bytes(&mut b);
        u32::from_le_bytes(b)
    }
    fn next_u64(&mut self) -> u64 {
        let mut b = [0u8; 8];
        self.fill_bytes(&mut b);
        u64::from_le_bytes(b)
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for byte in dest {
            *byte = if self.served < 32 { self.first } else { self.then };
            self.served += 1;
        }
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl CryptoRng for TwoRng {}
```

and after `an_organisation_key_equal_to_a_genesis_key_is_refused` add:

```rust
// LLR-e2b7gv, LLR-sj7cd5 as amended: a key pair drawn for an admission whose
// public key is the record's current Organisation public key, or a
// Member-as-a-group key of the resulting record (the joiner's), is refused
// with DuplicateKey — no update kept, the record's keys unchanged, nothing
// written. A fresh draw is then accepted.
// verifies: LLR-e2b7gv, LLR-sj7cd5, REQ-stx9v3
#[tokio::test(flavor = "multi_thread")]
async fn an_update_key_equal_to_the_records_or_to_a_key_of_the_resulting_record_is_refused() {
    let chain = MockChainOps::new();
    let (store, path) = open_store("update-collision", "a");
    let mut svc = OrgService::new(store, Box::new(chain.clone()));
    let pid = svc.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    // A genesis whose founder id is 0x11…11 and whose Organisation private key is 0x3d…3d.
    let update = svc.create_organisation(&mut TwoRng::new(0x11, 0x3d), &pid).unwrap();
    let org_id = chain.apply_genesis(update.resulting_root, update.org_pub_key);
    svc.commit_genesis(&mut OsRng, &pid, org_id, test_proxy()).await.unwrap();
    let rec = svc.list_orgs()[0].clone();
    let carol = org_node::Joiner {
        handle: h("carol"),
        name: nm("Carol"),
        surname: sn("Coder"),
        member_key: MemberSeed::from([0x4e; 32]).x25519_keypair().member_key().unwrap(),
        device_key: DeviceSeed::from([0x4f; 32]).signing_keypair().device_key().unwrap(),
    };
    let before = std::fs::read(&path).unwrap();
    // 0x3d: the record's current key; 0x4e: the joiner's Member key.
    for then in [0x3d, 0x4e] {
        let err = svc.admit_member(&mut TwoRng::new(0x22, then), org_id, &carol).unwrap_err();
        assert_eq!(err, OrgNodeError::Trie(OrgMembersError::DuplicateKey), "draw {then:#x}");
        assert!(svc.provisional_updates(org_id).is_empty(), "no update kept");
        let now = svc.list_orgs()[0].clone();
        assert_eq!((now.org_pub_key, now.org_private_key.clone()), (rec.org_pub_key, rec.org_private_key.clone()));
        assert_eq!(std::fs::read(&path).unwrap(), before, "nothing written");
    }
    svc.admit_member(&mut OsRng, org_id, &carol).expect("a fresh draw is accepted");
}
```

`org-node/tests/provisional_store.rs`:
- In `change_set`, replace
  `        change: ProvisionalChange::ChangeSet { change_set: vec![0xab; bytes] },`
  by
  `        change: ProvisionalChange::ChangeSet { change_set: vec![0xab; bytes], org_private_key: OrgPrivateKey::from(CHANGE_SET_PRIVATE) },`
  and add above `fn change_set`:
  `const CHANGE_SET_PRIVATE: [u8; 32] = [0x99; 32];`.
- In `an_update_with_the_same_identity_replaces_and_others_are_kept`, replace
  its comment `// Abnormal: the same identity replaces rather than duplicates; different`
  `// identities — another root, another Persona's genesis — are all kept.` by
  `// Abnormal: the same identity replaces rather than duplicates; different`
  `// identities — another root, another key, another Persona's genesis — are all kept.`;
  before `    assert_eq!(data.provisional_updates.len(), 4);` add:

```rust
    // The same root under another key is another update (LLR-95753m).
    let other_key = ProvisionalUpdate {
        org_pub_key: OrgPrivateKey::from([0x55; 32]).x25519_keypair().org_public_key().unwrap(),
        ..change_set(0xbb, 0x66, 5)
    };
    data.insert_provisional(other_key).unwrap();
```

  and replace `    assert_eq!(data.provisional_updates.len(), 4);` by
  `    assert_eq!(data.provisional_updates.len(), 5);`.
- Replace the whole test `the_genesis_private_key_lives_only_in_its_provisional_update`
  by:

```rust
// Normal: a genesis update and a Change-set update each hold their
// Organisation private key once in the store's plaintext; abnormal: with the
// updates gone, no copy of either key remains anywhere in the store.
// *Rewritten 2026-10-06 (change worktree-org-node-org-key-pair).* Was
// `the_genesis_private_key_lives_only_in_its_provisional_update`: only a
// genesis update held a key.
// verifies: REQ-ech45n, REQ-stx9v3, LLR-qjz3q4
#[test]
fn every_updates_private_key_lives_only_in_its_provisional_update() {
    let mut data = StoreData::default();
    let update = genesis("p-alice", 0x10);
    let ProvisionalChange::Genesis { org_private_key, .. } = &update.change else { panic!("genesis") };
    assert_eq!(org_private_key.x25519_keypair().org_public_key().unwrap(), update.org_pub_key);
    data.insert_provisional(update).unwrap();
    data.insert_provisional(change_set(0xbb, 0x66, 4)).unwrap();
    let plaintext = postcard::to_allocvec(&data).unwrap();
    assert_eq!(contains(&plaintext, &PRIVATE), 1);
    assert_eq!(contains(&plaintext, &CHANGE_SET_PRIVATE), 1);
    assert!(data.orgs.is_empty());
    data.provisional_updates.clear();
    let plaintext = postcard::to_allocvec(&data).unwrap();
    assert_eq!(contains(&plaintext, &PRIVATE), 0);
    assert_eq!(contains(&plaintext, &CHANGE_SET_PRIVATE), 0);
}
```

`org-node/tests/persona_records.rs`: in `enum WireChange`, replace
`    ChangeSet { change_set: Vec<u8> },` by
`    ChangeSet { change_set: Vec<u8>, org_private_key: [u8; 32] },`.

`org-node/tests/encoding_golden.rs`:
- After the T7 paragraph add a `// GOLDEN_STORE  = "…"` comment line holding
  the current (T7) store value, and:

```rust
//
// Re-pinned 2026-10-06 (change worktree-org-node-org-key-pair, T8): a
// Change-set provisional update holds the private key of its fresh
// Organisation key pair (LLR-e2b7gv, LLR-qjz3q4): `99`x32 inserted at offset
// 883, after its Change-set bytes `04 09080706`. Derived by the format rule,
// never captured from the code.
```

- Replace the `GOLDEN_STORE` value by:

```
0107702d616c69636501aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa05616c69636505416c69636505536d697468111111111111111111111111111111111111111111111111111111111111111112121212121212121212121212121212121212121212121212121212121212120101010101010101010101010101010101010101010101010101010101010101010101aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa3333333333333333333333333333333333333333333333333333333333333333d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737030202010101010101010101010101010101010101010101010101010101010101010105616c69636505416c69636505536d697468d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21020202020202020202020202020202020202020202020202020202020202020203626f6203426f62054a6f6e6573884b8857f4eaa1613c61504db34d4beaf346517a0e31de3cddd4d9b4201d9d0b01a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f00155555555555555555555555555555555555555555555555555555555555555554444444444444444444444444444444444444444444444444444444444444444020007702d616c69636500777777777777777777777777777777777777777777777777777777777777777701d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c97787370001010101010101010101010101010101010101010101010101010101010101010105616c69636505416c69636505536d697468d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21888888888888888888888888888888888888888888888888888888888888888801bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb07702d616c696365013333333333333333333333333333333333333333333333333333333333333333666666666666666666666666666666666666666666666666666666666666666603d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737010409080706999999999999999999999999999999999999999999999999999999999999999901cccccccccccccccccccccccccccccccccccccccc
```

- In `persona_store_plaintext_is_pinned`, after
  `    assert_eq!(pu["change"]["ChangeSet"]["change_set"], json!([9, 8, 7, 6]));`
  add `    assert_eq!(pu["change"]["ChangeSet"]["org_private_key"][0], json!(0x99));`.

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test commit_paths --test provisional_store --test organisation_key`.
Expected red: compile errors E0026/E0063 (`ChangeSet` has no field
`org_private_key`), E0061 (`discard_provisional` takes 4 arguments, 5
supplied).

**Step 3 — implementation, cycle A** (`org-node/src/store.rs`,
`org-node/src/service.rs`).

`store.rs`:
- Replace

```rust
    /// postcard of the Change set, for an admission or a revocation.
    ChangeSet { change_set: Vec<u8> },
```

  by

```rust
    /// postcard of the Change set of an admission or a revocation, and the
    /// private key of the fresh Organisation key pair drawn for it once its
    /// root was calculated (LLR-e2b7gv, LLR-qjz3q4), kept until
    /// `commit_update` takes it into the record (LLR-6s785x).
    ChangeSet { change_set: Vec<u8>, org_private_key: OrgPrivateKey },
```

- In `enum RawProvisionalChange`, replace `    ChangeSet { change_set: Vec<u8> },`
  by `    ChangeSet { change_set: Vec<u8>, org_private_key: OrgPrivateKey },`; in
  the `TryFrom` replace
  `                RawProvisionalChange::ChangeSet { change_set } => ProvisionalChange::ChangeSet { change_set },`
  by
  `                RawProvisionalChange::ChangeSet { change_set, org_private_key } => ProvisionalChange::ChangeSet { change_set, org_private_key },`.
- In `ProvisionalUpdate`'s `persona_id` doc, replace
  `    /// \`org_id\`, this is what groups it: its identity is \`(persona_id,\``
  `    /// resulting_root)\` and its bound is measured over the genesis updates of`
  by `    /// \`org_id\`, this is what groups it: its identity is \`(persona_id,\``
  `    /// resulting_root, org_pub_key)\` and its bound is measured over the genesis updates of`.
- In `insert_provisional`, replace its doc's last line
  `    /// LLR-jq7qh7). Identity: \`(org_id, resulting_root)\`, or for genesis`
  `    /// \`(persona_id, resulting_root)\`.` by
  `    /// LLR-jq7qh7). Identity: \`(org_id, resulting_root, org_pub_key)\`, or for`
  `    /// genesis \`(persona_id, resulting_root, org_pub_key)\`: two updates for one`
  `    /// change with different fresh keys are two updates.`; and replace
  `        let same_identity = |u: &ProvisionalUpdate| target.names(u) && u.resulting_root == update.resulting_root;`
  by

```rust
        let same_identity = |u: &ProvisionalUpdate| {
            target.names(u) && u.resulting_root == update.resulting_root && u.org_pub_key == update.org_pub_key
        };
```

`service.rs`:
- Replace `discard_provisional` (doc through closing brace) by:

```rust
    /// Remove the one provisional update for `target` whose resulting root
    /// is `resulting_root` and whose Organisation public key is
    /// `org_pub_key`, with the Organisation private key it holds, and save;
    /// refuse one not stored with `NoProvisionalUpdate`, changing and writing
    /// nothing (LLR-7cmp38).
    pub fn discard_provisional<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        target: ProvisionalTarget,
        resulting_root: RootHash,
        org_pub_key: OrgPublicKey,
    ) -> Result<(), OrgNodeError> {
        let updates = &mut self.store.data_mut().provisional_updates;
        let position = updates
            .iter()
            .position(|u| u.resulting_root == resulting_root && u.org_pub_key == org_pub_key && target.names(u))
            .ok_or(OrgNodeError::NoProvisionalUpdate)?;
        updates.remove(position);
        self.store.save(rng)
    }
```

- Replace `keep_change_set` (doc through closing brace) by:

```rust
    /// Keep a Change set built on `rec` as a provisional update: its Sequence
    /// number is the epoch it produces, the record's plus one (LLR-ghja3x,
    /// REQ-txvtm9, Decision 16). Its root calculated, the batch ends and a
    /// fresh Organisation key pair is drawn for it (REQ-stx9v3, LLR-e2b7gv):
    /// distinct from the record's current key and from every key of the
    /// resulting record, else `DuplicateKey`, nothing kept or written. The
    /// update holds the private key; the record's keys are not changed. A
    /// refusal by the bound writes nothing (LLR-jq7qh7).
    fn keep_change_set<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        rec: &OrgRecord,
        persona_id: PersonaId,
        new_trie: &Trie,
        delta: &Delta,
    ) -> Result<ProvisionalUpdate, OrgNodeError> {
        let seq = SequenceNumber::new(rec.epoch.get() + 1);
        let resulting_root = new_trie.root_hash().map_err(OrgNodeError::Trie)?;
        let org_kp = X25519Keypair::generate(rng);
        let org_pub_key = org_kp.org_public_key()?;
        if org_pub_key == rec.org_pub_key {
            return Err(OrgNodeError::Trie(org_members::OrgMembersError::DuplicateKey));
        }
        org_pub_key.ensure_distinct_from(&new_trie.members())?;
        let update = ProvisionalUpdate {
            org_id: Some(rec.org_id),
            persona_id,
            base_root: Some(rec.root_hash),
            resulting_root,
            seq,
            org_pub_key,
            change: ProvisionalChange::ChangeSet {
                change_set: Envelope::build(rec.org_id, seq, delta)?.delta_bytes,
                org_private_key: org_kp.org_private_key(),
            },
        };
        self.store.data_mut().insert_provisional(update.clone())?;
        self.store.save(rng)?;
        Ok(update)
    }
```

- In `commit_update`, replace `        let ProvisionalChange::ChangeSet { change_set } = update.change else {`
  by `        let ProvisionalChange::ChangeSet { change_set, .. } = update.change else {`
  (cycle B uses the key).

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test commit_paths --test provisional_store --test organisation_key`
→ cycle A's tests pass; `commit_paths` reports the cycle-B tests below red
once they are written.

**Step 4 — failing tests, cycle B (committing).**

`org-node/tests/commit_paths.rs`:
- In `commit_update_commits_a_provisional_update_the_chain_carries`: replace
  its annotation by
  `// verifies: REQ-tqap3r, REQ-jy6ybw, LLR-cmdrp9, LLR-6s785x, LLR-bg3vsw, LLR-4tcxsu, LLR-cja9zv`;
  replace `    let ProvisionalChange::ChangeSet { change_set } = update.change.clone() else { panic!() };`
  by `    let ProvisionalChange::ChangeSet { change_set, org_private_key } = update.change.clone() else { panic!() };`;
  replace `    assert_eq!(reopen_store("commit-update", "a", "pw_a").data().orgs[0].epoch, Epoch::new(2));`
  by:

```rust
    assert_eq!(
        (after.org_pub_key, private_key_held(&after)),
        (update.org_pub_key, org_private_key.clone()),
        "the record takes the update's key pair and keeps no earlier one"
    );
    let disk = reopen_store("commit-update", "a", "pw_a").data().orgs[0].clone();
    assert_eq!(disk.epoch, Epoch::new(2));
    assert_eq!((disk.org_pub_key, private_key_held(&disk)), (update.org_pub_key, org_private_key));
```

- In `commit_update_refusals_change_nothing_and_write_nothing`: replace its
  annotation by `// verifies: REQ-tqap3r, LLR-cmdrp9, LLR-6s785x, LLR-ewkg85`;
  replace the `check` closure by:

```rust
    let check = |a: &OrgService| {
        let now = rec_of(a, org);
        assert_eq!(now.root_hash, rec.root_hash);
        assert_eq!(
            (now.org_pub_key, private_key_held(&now)),
            (rec.org_pub_key, private_key_held(&rec)),
            "the record's keys are unchanged"
        );
        assert_eq!(a.provisional_updates(org), vec![update.clone()]);
        assert_eq!(store_bytes("update-refused", "a"), before, "nothing written");
    };
```

  and before `    // the chain carries the root but at the epoch already committed` add:

```rust
    // the chain carries the update's root under a key no provisional update holds
    chain.set(org, OrgState { root_hash: update.resulting_root, org_pub_key: org_public_key(), epoch: Epoch::new(rec.epoch.get() + 1) });
    assert_eq!(a.commit_update(&mut OsRng, org).await.unwrap_err(), OrgNodeError::NoProvisionalUpdate);
    check(&a);
```

- After `two_updates_for_the_same_change_are_two_and_one_is_discarded_alone`
  add:

```rust
// LLR-6s785x, LLR-cmdrp9: of two updates for the same change, commit_update
// commits the one whose key the chain carries and takes its private key; a
// state carrying the root under a key neither holds commits nothing.
// verifies: LLR-6s785x, LLR-cmdrp9, REQ-jy6ybw
#[tokio::test(flavor = "multi_thread")]
async fn commit_update_selects_the_update_by_its_root_and_its_key() {
    let s = admit_b_directly(setup("select-by-key").await).await;
    let mut a = s.svc_a;
    let b_id = id_by_handle(&rec_of(&a, s.org_id), "bob");
    let first = a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    let second = a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    let next = Epoch::new(rec_of(&a, s.org_id).epoch.get() + 1);
    s.chain.set(s.org_id, OrgState { root_hash: first.resulting_root, org_pub_key: org_public_key(), epoch: next });
    assert_eq!(a.commit_update(&mut OsRng, s.org_id).await.unwrap_err(), OrgNodeError::NoProvisionalUpdate);
    s.chain.set(s.org_id, OrgState { root_hash: second.resulting_root, org_pub_key: second.org_pub_key, epoch: next });
    a.commit_update(&mut OsRng, s.org_id).await.unwrap();
    let rec = rec_of(&a, s.org_id);
    assert_eq!((rec.org_pub_key, private_key_held(&rec)), (second.org_pub_key, private_key_of(&second)));
    assert_ne!(private_key_held(&rec), private_key_of(&first));
}
```

`org-node/tests/admission_sender.rs`, after
`pr_szkat6_an_admitted_member_holds_the_organisation_private_key`:

```rust
// PR-g9u3xq — reproduction. Removing a Member did not replace the Organisation
// key pair, so a removed Device kept the key every remaining Member still
// used. Every provisional update now draws a fresh key pair once its root is
// calculated (REQ-stx9v3), the committing node takes it (REQ-jy6ybw), and the
// next Organisation information carries it. B, removed, held the key of the
// epoch before its removal and holds nothing after it; the chain's key after
// the removal is another, and C's admission brings yet another. Red before
// (the key never changed), green after.
// verifies: LLR-e2b7gv, LLR-6s785x, REQ-stx9v3, REQ-jy6ybw, PR-g9u3xq
#[tokio::test(flavor = "multi_thread")]
async fn pr_g9u3xq_a_removed_device_holds_no_key_used_after_its_removal() {
    let mut s = admit_b_directly(setup("pr-g9u3xq").await).await;
    let b_key = private_key_held(&rec_of(&s.svc_b, s.org_id));
    let b_id = s.svc_b.list_personas()[0].member_id.expect("B has a member id");
    let (b_addr, b_task) = spawn_self_delete(s.svc_b, &s.b_device_kp).await;
    revoke(&mut s.svc_a, &s.chain, s.org_id, b_id, Some(b_addr)).await.unwrap();
    let (svc_b, outcome) = b_task.await.unwrap();
    assert!(matches!(outcome.unwrap(), SelfDeleteOutcome::SelfDeleted { .. }));
    assert!(svc_b.list_orgs().is_empty(), "B holds no record, so no key");
    let after_removal = s.chain.get(&s.org_id).unwrap().org_pub_key;
    assert_ne!(b_key.x25519_keypair().org_public_key().unwrap(), after_removal, "the removal published a fresh key");
    assert_eq!(
        private_key_held(&rec_of(&s.svc_a, s.org_id)).x25519_keypair().org_public_key().unwrap(),
        after_removal,
        "A took it"
    );
    let next = captured_admission_of_c(&mut s).await;
    let WireMessage::OrgInformation { org_private_key, .. } = &next else { panic!("Organisation information") };
    assert_ne!(org_private_key, &b_key);
    let now = s.chain.get(&s.org_id).unwrap().org_pub_key;
    assert_eq!(org_private_key.x25519_keypair().org_public_key().unwrap(), now);
    assert_ne!(now, after_removal, "and C's admission another");
}
```

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test commit_paths --test admission_sender`.
Expected red (runtime): `commit_update_commits_a_provisional_update_the_chain_carries`
— `the record takes the update's key pair` (left: the genesis key);
`commit_update_selects_the_update_by_its_root_and_its_key` — `left:
Ok(CommitOutcome …)` where `NoProvisionalUpdate` was expected (selection by
root alone); `pr_g9u3xq_…` — `A took it`.

**Step 5 — implementation, cycle B** (`org-node/src/service.rs`).
- In `commit_update`, replace the selection
  `            .find(|u| u.org_id == Some(org_id) && u.resulting_root == state.root_hash)`
  by

```rust
            .find(|u| {
                u.org_id == Some(org_id) && u.resulting_root == state.root_hash && u.org_pub_key == state.org_pub_key
            })
```

  replace `        let ProvisionalChange::ChangeSet { change_set, .. } = update.change else {`
  by `        let ProvisionalChange::ChangeSet { change_set, org_private_key } = update.change else {`,
  and replace

```rust
        if self.still_member(org_id, &verified.trie) {
            self.commit_held(org_id, &verified)?;
        } else {
```

  by

```rust
        if self.still_member(org_id, &verified.trie) {
            self.commit_held(org_id, &verified)?;
            // The record takes the update's key pair and keeps no earlier
            // one (LLR-6s785x, REQ-jy6ybw).
            Self::set_record_keys(self.store.data_mut(), org_id, org_private_key, update.org_pub_key)?;
        } else {
```

  and in its doc comment replace
  `    /// carries its root, by the checks a received update passes (LLR-cmdrp9).`
  by `    /// carries its root and key, by the checks a received update passes,`
  `    /// taking the update's key pair into the record (LLR-cmdrp9, LLR-6s785x).`
- After `store_carried_key`, add:

```rust
    /// The record of `org_id` takes the Organisation key pair of the update
    /// just committed, replacing the one it held (LLR-6s785x).
    fn set_record_keys(
        data: &mut StoreData,
        org_id: OrgId,
        private: OrgPrivateKey,
        public: OrgPublicKey,
    ) -> Result<(), OrgNodeError> {
        let rec = data.orgs.iter_mut().find(|o| o.org_id == org_id).ok_or(OrgNodeError::OrgNotOnChain)?;
        rec.org_private_key = private;
        rec.org_pub_key = public;
        Ok(())
    }
```

**Step 6 — PR-g9u3xq's resolution names its test**
(`org-node/docs/problems/2026-10-07-key-rotation.md`).
Replace the three resolution lines

```
resolution: a fresh key pair is drawn with every provisional update's root
(REQ-stx9v3) and replaces the record's key on commit (REQ-jy6ybw); only the
Devices of the new record receive it (REQ-3dsweu). Owner ruling 2026-10-06.
```

by

```
resolution: a fresh key pair is drawn with every provisional update's root
(REQ-stx9v3) and replaces the record's key on commit (REQ-jy6ybw); only the
Devices of the new record receive it (REQ-3dsweu). Owner ruling 2026-10-06.
Reproduced by `pr_g9u3xq_a_removed_device_holds_no_key_used_after_its_removal`
(org-node/tests/admission_sender.rs): red before, green after.
```

**Step 7 — green.**
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test commit_paths --test provisional_store --test organisation_key --test admission_sender --test encoding_golden --test persona_records`
→ commit_paths `30 passed`, provisional_store `5 passed`, organisation_key
`10 passed`, admission_sender `49 passed`, encoding_golden `4 passed`,
persona_records `16 passed`, 0 failed. Full org-node cargo line → every
target `ok`, 0 failed; `cargo check --all-targets` and `clippy` → `Finished`.

**Step 8 — commit.** `git add` the files of **Files touched**, then
`git -c commit.gpgsign=false commit -m "T8: a fresh Organisation key pair with every provisional update, taken on commit" -m "verifies: LLR-e2b7gv, LLR-6s785x, LLR-cmdrp9, LLR-ghja3x, LLR-tax3pm, LLR-qjz3q4, LLR-95753m, LLR-7cmp38, LLR-sj7cd5, REQ-stx9v3, REQ-jy6ybw; reproduces PR-g9u3xq"`.

**Red→green attestations:**
- admit_member_keeps_a_provisional_update_and_changes_nothing_else (re-asserted) — red -> green: red under a structural stub (ChangeSet had its field, keep_change_set still copied the record's key): commit_paths.rs:244 assert_ne, update key == record key; green after cycle A.
- revoke_member_keeps_a_provisional_update_and_changes_nothing_else (re-asserted) — red -> green: red under the stub: commit_paths.rs:641 assert_ne (no fresh key pair); green after cycle A.
- an_update_key_equal_to_the_records_or_to_a_key_of_the_resulting_record_is_refused — red -> green: red under the stub: organisation_key.rs:276 unwrap_err on Ok (no draw, no refusal); green after cycle A.
- two_updates_for_the_same_change_are_two_and_one_is_discarded_alone — red -> green: red under the stub: commit_paths.rs:600 assert_ne(first.org_pub_key, second.org_pub_key); green after cycle A.
- discarding_an_unknown_root_is_refused_and_writes_nothing (re-asserted) — red -> green: red under the stub: commit_paths.rs:561 unwrap_err on Ok (another update's key was discarded); green after cycle A.
- every_updates_private_key_lives_only_in_its_provisional_update (rewritten, renamed) — red -> green: red at compile time only, before the field existed (E0559 ChangeSet has no field org_private_key); the test builds the update itself, so no runtime red is possible once the field exists; green after the field.
- an_update_with_the_same_identity_replaces_and_others_are_kept (re-asserted) — red -> green: red under the stub: provisional_store.rs:101 left 4, right 5 (same root under another key replaced the update); green after cycle A.
- commit_update_commits_a_provisional_update_the_chain_carries (re-asserted) — red -> green: passed under the stub (update and record shared one key); red after cycle A at commit_paths.rs:306 "the record takes the update's key pair"; green after cycle B.
- commit_update_refusals_change_nothing_and_write_nothing (re-asserted) — red -> green: red under the stub and after cycle A: commit_paths.rs:345 unwrap_err on Ok (a chain root under an unheld key was committed); green after cycle B.
- commit_update_selects_the_update_by_its_root_and_its_key — red -> green: red under the stub and after cycle A: commit_paths.rs:620 unwrap_err on Ok (selection by root alone); green after cycle B.
- pr_g9u3xq_a_removed_device_holds_no_key_used_after_its_removal — red -> green: red under the stub at :1520 "the removal published a fresh key" (keys equal), and after cycle A at :1521 "A took it"; green after cycle B.
- persona_store_plaintext_is_pinned (re-pinned) — red -> green: red before any source change: encoding_golden.rs:120 DeserializeBadVarint on the new GOLDEN_STORE; green once the ChangeSet field existed; literal matches its derivation. Task done 2026-10-06, merged from worktree-org-node-org-key-pair-t8 (c30866d); org-node 241 passed, 0 failed; PR-g9u3xq's resolution names its reproducing test. Adapted in execution: the plan's pr_g9u3xq text had E0382 (`s` partially moved by spawn_self_delete), fixed by restoring `s.svc_b` after `b_task.await`; cycle-B test edits applied with cycle A (E0027 otherwise).

---

### T9 — org-node: a received key is checked against the chain and taken with the chain's public key

**Files touched:** `org-node/src/service.rs`,
`org-node/tests/admission_sender.rs`,
`org-node/docs/problems/2026-10-05-org-secret-unauthenticated.md`,
`org-node/docs/problems/2026-10-04-org-key-conflation.md`
**Parallel:** no (serial, after T8)
**IDs verified:** LLR-ba2ejp, LLR-ckk5nz, LLR-4kh9w9, LLR-37cj3n, LLR-mbjfq8,
LLR-e5c9ud, LLR-xq9nrq, REQ-bwx7eg, REQ-ju6vn2, RC-9cefcn; resolves
PR-ve9zw8 and PR-szkat6.
**Size:** ~230 lines.

**Step 1 — failing tests** (`org-node/tests/admission_sender.rs`).

After `pr_g9u3xq_a_removed_device_holds_no_key_used_after_its_removal` add:

```rust
// PR-ve9zw8 — reproduction. The Organisation secret a node stored was
// authenticated by nothing: any relay could hand a Member a secret of its
// choosing. The receipt check (RC-9cefcn) refuses an Organisation private key
// whose X25519 public half is not the chain's key. A relay substitutes the
// key in A's genuine admission of B: B refuses it after verification and
// before any commit — no record, the expectation kept, B's Persona untouched,
// nothing written — and then commits the genuine message. Red before (B
// committed the substituted key), green after.
// verifies: LLR-ba2ejp, LLR-mbjfq8, REQ-bwx7eg, PR-ve9zw8
#[tokio::test(flavor = "multi_thread")]
async fn pr_ve9zw8_a_first_admission_carrying_a_substituted_key_is_refused() {
    let mut s = setup("pr-ve9zw8-first").await;
    let joiner = s.joiner_b.clone();
    let genuine = captured_admission(&mut s, &joiner).await;
    let substituted = with_key(&genuine, OrgPrivateKey::from([0x42u8; 32]));
    let on_disk = store_bytes("pr-ve9zw8-first", "b");
    let (addr, task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(addr, &substituted).await;
    let (svc_b, result) = task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::OrgKeyMismatch { org_id: s.org_id });
    assert!(svc_b.list_orgs().is_empty(), "no record");
    assert_eq!(svc_b.expected_admissions().len(), 1, "the expectation is kept");
    assert_eq!(persona_of(&svc_b, &s.pid_b).status, PersonaStatus::Proposed, "the Persona untouched");
    assert_eq!(store_bytes("pr-ve9zw8-first", "b"), on_disk, "nothing written");
    let (addr, task) = spawn_receive(svc_b, &s.b_device_kp).await;
    deliver(addr, &genuine).await;
    let (svc_b, result) = task.await.unwrap();
    result.expect("the genuine admission commits");
    assert_eq!(
        private_key_held(&rec_of(&svc_b, s.org_id)).x25519_keypair().org_public_key().unwrap(),
        s.chain.get(&s.org_id).unwrap().org_pub_key
    );
}

// PR-ve9zw8 on an Organisation already held, on both receive paths: the
// substituted key is refused and B's record — root, epoch, mark and both keys —
// is unchanged, nothing written; the genuine message then commits. The
// chain's key decides only whether the message commits (LLR-37cj3n).
// verifies: LLR-ba2ejp, LLR-37cj3n, REQ-bwx7eg, RC-9cefcn, PR-ve9zw8
#[tokio::test(flavor = "multi_thread")]
async fn pr_ve9zw8_an_update_carrying_a_substituted_key_is_refused_on_both_paths() {
    let mut s = admit_b_directly(setup("pr-ve9zw8-held").await).await;
    let genuine = captured_admission_of_c(&mut s).await;
    let substituted = with_key(&genuine, OrgPrivateKey::from([0x42u8; 32]));
    let held = rec_of(&s.svc_b, s.org_id);
    let on_disk = store_bytes("pr-ve9zw8-held", "b");
    let (addr, task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(addr, &substituted).await;
    let (svc_b, result) = task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::OrgKeyMismatch { org_id: s.org_id });
    let (addr, task) = spawn_self_delete(svc_b, &s.b_device_kp).await;
    deliver(addr, &substituted).await;
    let (svc_b, result) = task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::OrgKeyMismatch { org_id: s.org_id });
    let after = rec_of(&svc_b, s.org_id);
    assert_eq!(
        (after.epoch, after.last_seq, after.root_hash, after.org_pub_key),
        (held.epoch, held.last_seq, held.root_hash, held.org_pub_key)
    );
    assert_eq!(private_key_held(&after), private_key_held(&held));
    assert_eq!(store_bytes("pr-ve9zw8-held", "b"), on_disk, "nothing written");
    let (addr, task) = spawn_self_delete(svc_b, &s.b_device_kp).await;
    deliver(addr, &genuine).await;
    let (_svc_b, result) = task.await.unwrap();
    assert!(matches!(result, Ok(SelfDeleteOutcome::UpdatedNotRevoked { .. })), "{result:?}");
}

// LLR-ckk5nz, LLR-4kh9w9: a committed Organisation-information update gives
// the record the key it carried and the chain's public key — on either receive
// path, in memory and on disk — so the record's two keys are always a pair,
// and the same pair A's record holds.
// verifies: LLR-ckk5nz, LLR-4kh9w9, REQ-ju6vn2
#[tokio::test(flavor = "multi_thread")]
async fn a_committed_update_gives_the_record_the_new_key_pair_on_both_paths() {
    let mut s = admit_b_directly(setup("new-pair").await).await;
    let jr_c = joiner_for_c(&mut s.svc_a);
    let (addr, task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &jr_c, addr).await.unwrap();
    let (svc_b, result) = task.await.unwrap();
    result.unwrap();
    for (rec, place) in [
        (rec_of(&svc_b, s.org_id), "after receive_and_verify"),
        (disk_rec_of(&reopen_store("new-pair", "b", "pw_b"), s.org_id), "on disk"),
    ] {
        assert_eq!(rec.org_pub_key, s.chain.get(&s.org_id).unwrap().org_pub_key, "{place}");
        assert_eq!(private_key_held(&rec), private_key_held(&rec_of(&s.svc_a, s.org_id)), "{place}");
        assert_eq!(private_key_held(&rec).x25519_keypair().org_public_key().unwrap(), rec.org_pub_key, "{place}: a pair");
    }

    let pid_d = s.svc_a.create_persona(&mut OsRng, h("dave"), nm("Dave"), sn("Diver")).unwrap();
    let jr_d = joiner_of(&s.svc_a, &pid_d);
    let (addr, task) = spawn_self_delete(svc_b, &s.b_device_kp).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &jr_d, addr).await.unwrap();
    let (svc_b, result) = task.await.unwrap();
    assert!(matches!(result, Ok(SelfDeleteOutcome::UpdatedNotRevoked { .. })), "{result:?}");
    for (rec, place) in [
        (rec_of(&svc_b, s.org_id), "after the self-delete path"),
        (disk_rec_of(&reopen_store("new-pair", "b", "pw_b"), s.org_id), "on disk"),
    ] {
        assert_eq!(rec.org_pub_key, s.chain.get(&s.org_id).unwrap().org_pub_key, "{place}");
        assert_eq!(private_key_held(&rec), private_key_held(&rec_of(&s.svc_a, s.org_id)), "{place}");
        assert_eq!(private_key_held(&rec).x25519_keypair().org_public_key().unwrap(), rec.org_pub_key, "{place}: a pair");
    }
}
```

Replace the whole test `a_persona_whose_member_key_the_chain_publishes_is_still_bound`
(comment through closing brace) by:

```rust
// Normal side of LLR-e5c9ud as amended: the Persona whose device the trie
// holds is bound even when its member key is the key the chain publishes —
// the administrator-key exclusion is gone. The record keeps the chain's key,
// not one from the message (LLR-xq9nrq). Since the receipt check (LLR-ba2ejp)
// the message must carry that key's private half: here B's member seed.
// verifies: LLR-e5c9ud, LLR-xq9nrq, LLR-ba2ejp
#[tokio::test(flavor = "multi_thread")]
async fn a_persona_whose_member_key_the_chain_publishes_is_still_bound() {
    let mut s = setup("published-key").await;
    let joiner = s.joiner_b.clone();
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &joiner, sink_addr).await.unwrap();
    let msg = sink.await.unwrap().2;
    let state = s.chain.get(&s.org_id).unwrap();
    let published = OrgPublicKey::parse(joiner.member_key.as_bytes()).unwrap();
    s.chain.set(s.org_id, org_node::chain::OrgState { org_pub_key: published, ..state });
    let b_seed = *persona_of(&s.svc_b, &s.pid_b).member_seed.expose_secret();
    let msg = with_key(&msg, OrgPrivateKey::from(b_seed));
    let (addr, task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(addr, &msg).await;
    let (svc_b, result) = task.await.unwrap();
    result.unwrap();
    let p = persona_of(&svc_b, &s.pid_b);
    assert_eq!((p.status, p.org_id), (PersonaStatus::Active, Some(s.org_id)));
    assert!(p.member_id.is_some());
    assert_eq!(rec_of(&svc_b, s.org_id).org_pub_key, published);
}
```

In `a_first_admission_records_the_chains_key_the_private_key_and_the_member`,
after `    assert_eq!(private_key_held(&rec), held_by_a, "the key the message carried, the one A's record holds");`
add
`    assert_eq!(held_by_a.x25519_keypair().org_public_key().unwrap(), published, "the private half of the chain's key (LLR-ba2ejp)");`
and replace its annotation by
`// verifies: LLR-ckk5nz, LLR-e5c9ud, LLR-rys5nx, LLR-3fwykc, LLR-ba2ejp`.

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test admission_sender pr_ve9zw8 a_committed_update_gives`.
Expected red (runtime): both `pr_ve9zw8_…` — ``called `Result::unwrap_err()`
on an `Ok` value`` (the substituted key is committed);
`a_committed_update_gives_…` — `after receive_and_verify` (left: the genesis
public key, right: the rotated one).

**Step 2 — implementation** (`org-node/src/service.rs`).
- After `set_record_keys`, add:

```rust
    /// Refuse an Organisation private key whose X25519 public half is not the
    /// Organisation public key the chain state read carries (LLR-ba2ejp,
    /// RC-9cefcn).
    fn check_carried_key(org_id: OrgId, key: &OrgPrivateKey, state: &OrgState) -> Result<(), OrgNodeError> {
        if key.x25519_keypair().org_public_key()? == state.org_pub_key {
            Ok(())
        } else {
            Err(OrgNodeError::OrgKeyMismatch { org_id })
        }
    }
```

- Delete `store_carried_key` (doc through closing brace), and in
  `commit_held`'s doc replace `    /// orphans (LLR-mkj4bz). It takes no key (LLR-ckk5nz): a received key is`
  `    /// written by \`store_carried_key\`.` by
  `    /// orphans (LLR-mkj4bz). It takes no key (LLR-ckk5nz): the keys are`
  `    /// written by \`set_record_keys\`.`
- In `receive_and_verify`, after
  `        let (verified, chain_state) = self.verify_received(&local_trie, &envelope, &ctx).await?;`
  add:

```rust
        // The key Organisation information carries must be the private half
        // of the chain's key (LLR-ba2ejp, RC-9cefcn) — checked before the
        // own-Persona rule and before anything is written; a revocation
        // carries none.
        if let Some(key) = &carried_key {
            Self::check_carried_key(org_id, key, &chain_state)?;
        }
```

  and in the held branch replace

```rust
            // A committed received update stores the key it carried
            // (REQ-ju6vn2, LLR-ckk5nz).
            self.commit_held(org_id, &verified)?;
            Self::store_carried_key(self.store.data_mut(), org_id, key)?;
```

  by

```rust
            // A committed received update takes the key it carried and the
            // chain's public key, a pair (REQ-ju6vn2, LLR-ckk5nz, LLR-4kh9w9).
            self.commit_held(org_id, &verified)?;
            Self::set_record_keys(self.store.data_mut(), org_id, key, chain_state.org_pub_key)?;
```

- In `receive_and_self_delete_if_revoked`, replace
  `        let is_revocation = matches!(msg, WireMessage::Revocation { .. });` by:

```rust
        // The key Organisation information carries; a revocation carries none.
        let carried_key = match msg {
            WireMessage::OrgInformation { org_private_key, .. } => Some(org_private_key),
            WireMessage::Revocation { .. } => None,
        };
```

  replace `        let (verified, _) = self.verify_received(&local_trie, &envelope, &ctx).await?;`
  by:

```rust
        let (verified, chain_state) = self.verify_received(&local_trie, &envelope, &ctx).await?;
        // The receipt check, before any commit or record deletion (LLR-ba2ejp).
        if let Some(key) = &carried_key {
            Self::check_carried_key(org_id, key, &chain_state)?;
        }
```

  and replace the still-member branch down to its `return Ok(SelfDeleteOutcome::UpdatedNotRevoked { org_id });`
  by:

```rust
        if self.still_member(org_id, &verified.trie) {
            // A revocation that leaves this node's Device listed is refused,
            // nothing written (LLR-pt32fx).
            let Some(key) = carried_key else {
                return Err(OrgNodeError::RevocationNotForThisDevice { org_id });
            };
            // An ordinary update: the record takes the key it carried and the
            // chain's public key (LLR-ckk5nz, LLR-4kh9w9).
            self.commit_held(org_id, &verified)?;
            Self::set_record_keys(self.store.data_mut(), org_id, key, chain_state.org_pub_key)?;
            self.store.save(rng)?;
            return Ok(SelfDeleteOutcome::UpdatedNotRevoked { org_id });
        }
```

**Step 3 — resolve PR-ve9zw8 and PR-szkat6.**
- `org-node/docs/problems/2026-10-05-org-secret-unauthenticated.md`: replace
  `status: open` under `**PR-ve9zw8**` by:

```
status: resolved
resolution: the Organisation secret is replaced by the Organisation private key, which a receiver stores only if its X25519 public half is the chain's `org_pub_key` (RC-9cefcn, REQ-bwx7eg, LLR-ba2ejp); reproduced by `pr_ve9zw8_a_first_admission_carrying_a_substituted_key_is_refused` and `pr_ve9zw8_an_update_carrying_a_substituted_key_is_refused_on_both_paths` (org-node/tests/admission_sender.rs), red before (the substituted key was committed), green after.
```

- `org-node/docs/problems/2026-10-04-org-key-conflation.md`: replace
  `status: open` under `**PR-szkat6**` by:

```
status: resolved
resolution: every Organisation-information message carries the Organisation private key the sending node's record holds (REQ-szq3ud), and its receiver stores it once its public half is the chain's key (REQ-ju6vn2, REQ-bwx7eg); `OrgPublicKey` is parsed by person's X25519 rule (LLR-3jjgtw); reproduced by `pr_szkat6_an_admitted_member_holds_the_organisation_private_key` (org-node/tests/admission_sender.rs), red before (a member's record held none), green after.
```

- Check both with `GR_CONFIG=org-node/.guardrails/config.yaml sh .guardrails/scripts/find-items.sh show PR-ve9zw8`
  and `… show PR-szkat6` → `status: resolved`.

**Step 4 — green.**
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test admission_sender`
→ `52 passed`, 0 failed. Full org-node cargo line → every target `ok`, 0
failed; the five Quint lines (QUINT_HOME as in Environment) pass;
`cargo check --all-targets` and `clippy` → `Finished`. org-node's interface
is now final.

**Step 5 — commit.** `git add` the files of **Files touched**, then
`git -c commit.gpgsign=false commit -m "T9: a received key is checked against the chain and taken with its public key (PR-ve9zw8, PR-szkat6)" -m "verifies: LLR-ba2ejp, LLR-ckk5nz, LLR-4kh9w9, LLR-37cj3n, REQ-bwx7eg, REQ-ju6vn2, RC-9cefcn; resolves PR-ve9zw8, PR-szkat6"`.

**Red→green attestations:**
- pr_ve9zw8_a_first_admission_carrying_a_substituted_key_is_refused — red -> green: watched fail before the implementation (unwrap_err on Ok: ReceiveOutcome { … epoch: Epoch(2) … } — B committed the substituted key); green after.
- pr_ve9zw8_an_update_carrying_a_substituted_key_is_refused_on_both_paths — red -> green: watched fail before (unwrap_err on Ok: ReceiveOutcome { … epoch: Epoch(3) … }); green after.
- a_committed_update_gives_the_record_the_new_key_pair_on_both_paths — red -> green: watched fail before (left OrgPublicKey(81edea9e..) right OrgPublicKey(d4e3cf61..) — the record kept the genesis public key); green after.
- a_persona_whose_member_key_the_chain_publishes_is_still_bound (rewritten) — red -> green: passes before the implementation (nothing checked the key), so its red was shown two ways: with the comparison in check_carried_key inverted it fails with OrgKeyMismatch, and with the real implementation and the old body (no `with_key(b_seed)`) it fails the same way; green after. It verifies the accepting side of LLR-ba2ejp.
- a_first_admission_records_the_chains_key_the_private_key_and_the_member (extended) — red -> green: its new line already held since T8, so its red was shown with a stub in which set_record_keys does not write the private key: only the new line failed ("the private half of the chain's key (LLR-ba2ejp)"); stub reverted, green. Task done 2026-10-06, merged from worktree-org-node-org-key-pair-t9 (547fd16); org-node 244 passed, 0 failed; quint clean; PR-ve9zw8 and PR-szkat6 resolved in their dated files.

---

### T10 — app: org-node's new interface; whom each command sends to

**Files touched:** `app/src-tauri/src/commands.rs`,
`app/src-tauri/src/invitation.rs`, `app/src-tauri/src/submit.rs`,
`app/src-tauri/src/events.rs`, `app/src-tauri/tests/ipc.rs`,
`app/src-tauri/tests/invitation.rs`, `app/src-tauri/tests/submit_flow.rs`,
`app/src-tauri/tests/receiver_events.rs`
**Parallel:** no (serial, after T9 and T1)
**IDs verified:** LLR-q225ws, LLR-8krgzj, LLR-ctrfz4, LLR-gha5f6,
LLR-w4mhd4, LLR-f35pda, LLR-7bk6qh (see Open questions 1), REQ-tcutr6,
REQ-nfr3n2.
**Size:** ~260 lines.

**Precondition.** LLR-7bk6qh (`app/docs/architecture/2026-10-05-decomposition.md`)
lists, by name, the variants it classifies as receiver errors; it must be
amended by the design owner to add `MalformedMessage`, `OrgKeyMismatch`,
`RevocationNotHeld` and `RevocationNotForThisDevice` (Open questions 1) before
this task's commit is merged. The code below implements this plan's default.

**Step 1 — failing tests.**

`app/src-tauri/tests/ipc.rs`:
- Replace the comment block line
  `// \`admit_member\` parses the org id, then the Organisation secret, then the`
  `// peer address, and only then the reply. Its normal-case tests hand it the`
  by `// \`admit_member\` parses the org id, then the peer address, and only then`
  `// the reply. Its normal-case tests hand it the`.
- Replace `admit_args` by:

```rust
/// `admit_member`'s arguments, with the reply `"x"`.
fn admit_args(org_id: &str, peer_addr_blob: &str) -> serde_json::Value {
    serde_json::json!({ "orgId": org_id, "replyBlob": "x", "peerAddrBlob": peer_addr_blob })
}
```

- Delete the tests `admit_member_refuses_an_org_secret_that_is_not_32_bytes`
  and `admit_member_refuses_an_org_secret_that_is_not_hex` (each with its
  `// verifies: LLR-8krgzj` line), and replace
  `admit_member_does_not_refuse_a_32_byte_or_absent_org_secret` (annotation
  through closing brace) by:

```rust
// LLR-8krgzj as amended: `admit_member` takes no Organisation secret — an
// `orgSecretHex` a caller still sends is not read, whatever it holds — and a
// blank peer address is the absent one (LLR-ctrfz4).
// verifies: LLR-8krgzj, LLR-ctrfz4
#[test]
fn admit_member_takes_no_organisation_secret() {
    let h = harness();
    for stale in [serde_json::json!("zz".repeat(32)), serde_json::json!("aa"), serde_json::Value::Null] {
        let mut args = admit_args(&org_id_40(), "");
        args["orgSecretHex"] = stale.clone();
        let err = invoke_err(&h, "admit_member", args);
        assert_eq!(err, refused_for_the_reply(), "{stale}");
    }
}
```

- Replace every remaining occurrence of `, serde_json::Value::Null))` by
  `))` (the five `admit_args(…, serde_json::Value::Null)` calls). Then
  `grep -n "admit_args(.*Null" app/src-tauri/tests/ipc.rs` → no output.

`app/src-tauri/tests/receiver_events.rs`, in
`locally_reachable_variants_are_classified_as_receiver_errors`: append to the
comment (after `// raised before anything is verified.`):

```rust
    // The four refusals of a received Wire message are not verdicts of
    // `verify_envelope_against_chain` either: a message that does not decode
    // and a revocation about an Organisation not held are refused before
    // anything is verified, and a key that is not the chain's and a
    // revocation that leaves this node listed after it, on a rule about the
    // key or this node.
```

and after
`        OrgNodeError::PersonaAlreadyBound { persona_id: org_node::PersonaId::new("p".into()) },`
add:

```rust
        OrgNodeError::MalformedMessage,
        OrgNodeError::OrgKeyMismatch { org_id: org_node::OrgId::new([1; 20]) },
        OrgNodeError::RevocationNotHeld { org_id: org_node::OrgId::new([1; 20]) },
        OrgNodeError::RevocationNotForThisDevice { org_id: org_node::OrgId::new([1; 20]) },
```

`app/src-tauri/tests/invitation.rs`:
- Replace the import list
  `    admit_reply, check_reply, issue_invite, produce_reply, Invite, InviteReply, OutstandingInvites,`
  by `    admit_reply, check_reply, issue_invite, produce_reply, Invite, InviteId, InviteReply, OutstandingInvites,`.
- Replace `&org_node::InviteId::new([3; 32])` by `&InviteId::new([3; 32])`.
- Replace every occurrence of `, None, None)` by `, None)` (the six
  `admit_reply` calls with no address), and
  `admit_reply(&mut a, &writer, &mut outstanding, &mut OsRng, org, &reply_blob, Some(addr), None)`
  by `admit_reply(&mut a, &writer, &mut outstanding, &mut OsRng, org, &reply_blob, Some(addr))`.
  Then `grep -n "admit_reply(" app/src-tauri/tests/invitation.rs` → every
  call has seven arguments.
- In `a_confirmed_reply_carries_the_persona_and_declares_the_expected_admission`,
  replace
  `    assert_eq!(b.expected_admissions(), &[org_node::store::ExpectedAdmission { org_id: org, invite_id }]);`
  by `    assert_eq!(b.expected_admissions(), &[org_node::store::ExpectedAdmission { org_id: org }]);`.
- In `a_reply_is_acted_on_once_and_only_if_its_invite_is_outstanding`, replace
  its annotation `// verifies: LLR-gha5f6` by `// verifies: LLR-gha5f6, LLR-q225ws`
  and `    assert_eq!(msg.unwrap().1.invite_id, Some(id), "sent under the reply's invite id");`
  by:

```rust
    assert!(
        matches!(msg.unwrap().1, org_node::transport::wire::WireMessage::OrgInformation { .. }),
        "the reply's Device is listed: Organisation information, carrying no invite id"
    );
```

`app/src-tauri/tests/submit_flow.rs`:
- Add `use ods_poc_lib::commands::revoke_and_send;` and
  `use org_node::transport::wire::WireMessage;` to the imports.
- In `founding_writes_the_chain_then_commits`, replace
  `    assert!(rec.org_private_key.is_some());` by
  `    assert_eq!(rec.org_private_key.x25519_keypair().org_public_key().unwrap(), rec.org_pub_key, "the founder holds the Organisation's key pair");`.
- In `an_admission_is_written_then_committed_then_sent`, replace the two lines
  `    let invite = org_node::InviteId::new([0x44; 32]);` and
  `    b.expect_admission(&mut OsRng, org, invite).unwrap();` by
  `    b.expect_admission(&mut OsRng, org).unwrap();`; replace
  `submit_commit_send(&mut a, &writer, &mut OsRng, &update, device_key, Some(addr_b), None, Some(invite))`
  by `submit_commit_send(&mut a, &writer, &mut OsRng, &update, device_key, Some(addr_b))`.
- Replace every remaining occurrence of `, None, None, None)` by `, None)`
  (three `submit_commit_send` calls).
- At the end of the file add:

```rust
// LLR-q225ws: each command asks org-node for exactly one send, to exactly one
// DevicePublicKey, passing no key, secret or invite identifier; org-node
// chooses the kind from its committed record. The admission goes to the
// joiner's Device, which the record lists: Organisation information. The
// revocation goes to the removed Member's first Device, which it no longer
// lists: a revocation. Nothing else is sent.
// verifies: LLR-q225ws
#[tokio::test(flavor = "multi_thread")]
async fn each_command_sends_once_to_one_device_and_the_kind_follows_the_record() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let mut a = service("kinds", &chain);
    let pid = persona(&mut a, "alice");
    let org = found_organisation(&mut a, &writer, &mut OsRng, &pid).await.unwrap();
    let bob = joiner(0x61, "bob");
    let sink = OrgEndpoint::bind(&DeviceSeed::from([0x62; 32]).signing_keypair()).await.unwrap();

    let addr = sink.inner().addr();
    let task = tokio::spawn(async move {
        let got = sink.recv_one().await;
        (sink, got)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    let update = a.admit_member(&mut OsRng, org, &bob).unwrap();
    submit_commit_send(&mut a, &writer, &mut OsRng, &update, bob.device_key, Some(addr)).await.unwrap();
    let (sink, got) = task.await.unwrap();
    assert!(matches!(got.unwrap().1, WireMessage::OrgInformation { .. }), "the joiner's Device is listed");

    let bob_id = a.list_orgs()[0].trie_members.iter().find(|m| m.member_key == bob.member_key).unwrap().id;
    let addr = sink.inner().addr();
    let task = tokio::spawn(async move {
        let got = sink.recv_one().await;
        (sink, got)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    revoke_and_send(&mut a, &writer, &mut OsRng, org, bob_id, Some(addr)).await.unwrap();
    let (sink, got) = task.await.unwrap();
    assert!(matches!(got.unwrap().1, WireMessage::Revocation { .. }), "the removed Device is not listed");
    assert!(
        tokio::time::timeout(Duration::from_secs(2), sink.recv_one()).await.is_err(),
        "nothing else is sent"
    );
}
```

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --manifest-path app/src-tauri/Cargo.toml --features test-support --test submit_flow --test ipc`.
Expected red: compile errors — E0432 (`unresolved import org_node::OrgSecret`
in `src/commands.rs`, `src/invitation.rs`, `src/submit.rs`;
`ods_poc_lib::commands::revoke_and_send`), E0004 in `src/events.rs`.

**Step 2 — implementation.**

`app/src-tauri/src/invitation.rs`:
- Replace
  `use org_node::{DevicePublicKey, Handle, InviteId, Name, OrgId, OrgNodeError, OrgSecret, PersonPublicKey, PersonaId, Surname};`
  by
  `use org_node::{DevicePublicKey, Handle, Name, OrgId, OrgNodeError, PersonPublicKey, PersonaId, Surname};`.
- Before `/// An Invite as parsed:` add:

```rust
/// The identifier an Invite carries and its reply echoes (REQ-65xqp8): 32
/// bytes this device drew at random. Not secret. The app's own type: it binds
/// a reply to the Invite this device issued and never reaches org-node
/// (REQ-tcutr6 as amended).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InviteId([u8; 32]);

impl InviteId {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
```

- In `produce_reply`, replace its doc's last line
  `/// (LLR-rt8gdz); declares the expected admission under the invite id.` by
  `/// (LLR-rt8gdz); declares the expected admission to the Invite's`
  `/// Organisation (REQ-tcutr6 as amended).`, and
  `    svc.expect_admission(rng, invite.org_id, invite.invite_id).map_err(|e| e.to_string())?;`
  by `    svc.expect_admission(rng, invite.org_id).map_err(|e| e.to_string())?;`.
- In `admit_reply`, replace its doc lines
  `/// chain, to the Organisation its outstanding pair names, the admission`
  `/// carrying the reply's invite id; the Invite is settled once the admission`
  by `/// chain, to the Organisation its outstanding pair names, sent to the`
  `/// reply's Device alone (LLR-q225ws); the Invite is settled once the admission`;
  delete the parameter line `    org_secret: Option<OrgSecret>,`; replace

```rust
    let sent =
        submit_commit_send(svc, writer, rng, &update, reply.device_key, peer_addr, org_secret, Some(reply.invite_id)).await;
```

  by `    let sent = submit_commit_send(svc, writer, rng, &update, reply.device_key, peer_addr).await;`.

`app/src-tauri/src/submit.rs`:
- In the `use org_node::{…}` list delete `InviteId, ` and `OrgSecret, `.
- Replace `submit_commit_send`'s doc comment, attribute and signature by:

```rust
/// Write `update` to the chain; only once that executed, commit it and send
/// it to `recipient` (REQ-nfr3n2). org-node chooses the kind of message from
/// its committed record; the app passes it no key, secret or invite
/// identifier (LLR-q225ws).
pub async fn submit_commit_send<R: RngCore + CryptoRng + Send>(
    svc: &mut OrgService,
    writer: &dyn ChainWriter,
    rng: &mut R,
    update: &ProvisionalUpdate,
    recipient: DevicePublicKey,
    peer_addr: Option<iroh::EndpointAddr>,
) -> Result<CommitOutcome, String> {
```

  and replace `    svc.send_update(&outcome.outgoing, recipient, peer_addr, org_secret, invite_id)`
  by `    svc.send_update(&outcome.outgoing, recipient, peer_addr)`.

`app/src-tauri/src/commands.rs`:
- Replace `use org_node::service::SelfDeleteOutcome;` and
  `use org_node::{MemberId, OrgNodeError, OrgSecret, PersonaId};` by
  `use org_node::service::{OrgService, SelfDeleteOutcome};`,
  `use org_node::{CommitOutcome, MemberId, OrgId, OrgNodeError, PersonaId};`
  and `use rand::{CryptoRng, RngCore};`; add `use crate::submit::ChainWriter;`
  after `use crate::state::{AppState, ConnectionStatus};`.
- Replace `admit_member`'s doc comment and the function by:

```rust
/// Admit the person an Invite reply names (LLR-gha5f6): org-node builds the
/// admission, the app writes it to the chain, then org-node commits it and
/// sends it to the reply's device (REQ-nfr3n2, LLR-q225ws). The target is the
/// Organisation the reply's outstanding pair names; `org_id` is the
/// operator's selection and is refused unless it is that one.
///
/// The org id, then the peer address (LLR-ctrfz4, as `revoke_member` reads
/// it) are parsed before the reply is. It takes no Organisation secret or key
/// (LLR-8krgzj): the key the admission's message carries is the one
/// org-node's record holds. Returns the new member_id as 64 hex chars.
#[tauri::command]
pub async fn admit_member(
    state: State<'_, AppState>,
    org_id: String,
    reply_blob: String,
    peer_addr_blob: String,
) -> Result<String, String> {
    let oid = parse_org_id(&org_id)?;
    let peer_addr = parse_peer_addr(&peer_addr_blob)?;
    let mut svc = state.service.lock().await;
    let mut outstanding = state.outstanding.lock().await;
    let member_id = invitation::admit_reply(
        &mut svc,
        &*state.writer,
        &mut outstanding,
        &mut OsRng,
        oid,
        &reply_blob,
        peer_addr,
    )
    .await?;
    Ok(hex::encode(member_id.as_bytes()))
}
```

- In `revoke_member`, replace everything from
  `    // REQ-nfr3n2: org-node builds the removal, the app writes it to the`
  through the function's final `.map(|_| ())` by:

```rust
    let mut svc = state.service.lock().await;
    revoke_and_send(&mut svc, &*state.writer, &mut OsRng, oid, MemberId::new(member_id), peer_addr)
        .await
        .map(|_| ())
}

/// Revoke `member` from `org_id` (REQ-nfr3n2): org-node builds the removal,
/// the app writes it to the chain, and only then does org-node commit it and
/// send it, once, to the first DevicePublicKey of the removed Member's
/// snapshot in the record as it stood before the removal — which the
/// committed record no longer lists, so org-node sends that Device a
/// revocation (LLR-q225ws). The `revoke_member` command's body.
pub async fn revoke_and_send<R: RngCore + CryptoRng + Send>(
    svc: &mut OrgService,
    writer: &dyn ChainWriter,
    rng: &mut R,
    org_id: OrgId,
    member: MemberId,
    peer_addr: Option<iroh::EndpointAddr>,
) -> Result<CommitOutcome, String> {
    let recipient = svc
        .list_orgs()
        .iter()
        .find(|o| o.org_id == org_id)
        .ok_or_else(|| OrgNodeError::OrgNotOnChain.to_string())?
        .trie_members
        .iter()
        .find(|m| m.id == member)
        .and_then(|m| m.device_keys.first().copied())
        .ok_or("member_id names no member of this organisation")?;
    let update = svc.revoke_member(rng, org_id, member).map_err(|e| e.to_string())?;
    crate::submit::submit_commit_send(svc, writer, rng, &update, recipient, peer_addr).await
```

  (the `}` that closes `revoke_member` in the file now closes
  `revoke_and_send`; read the function after the edit to check the braces).

`app/src-tauri/src/events.rs`, in `classify_receive_error`: after the
comment line
`        // refusal about this node's own store, before anything is verified.`
add:

```rust
        // The four refusals of a received Wire message are not verdicts of
        // `verify_envelope_against_chain` either (LLR-j5vbqj): a message that
        // does not decode (`MalformedMessage`) and a revocation about an
        // Organisation not held (`RevocationNotHeld`) are refused before
        // anything is verified; a key that is not the chain's
        // (`OrgKeyMismatch`) and a revocation that leaves this node listed
        // (`RevocationNotForThisDevice`) after it, on a rule about the key or
        // this node.
```

and replace
`        | OrgNodeError::PersonaAlreadyBound { .. } => ReceiverOutcome::ReceiveError {`
by:

```rust
        | OrgNodeError::PersonaAlreadyBound { .. }
        | OrgNodeError::MalformedMessage
        | OrgNodeError::OrgKeyMismatch { .. }
        | OrgNodeError::RevocationNotHeld { .. }
        | OrgNodeError::RevocationNotForThisDevice { .. } => ReceiverOutcome::ReceiveError {
```

**Step 3 — green.** The app cargo line of **Verification** → every target
`ok`, 0 failed (ipc 33, invitation 17, submit_flow 6, receiver_events 32,
the others unchanged); restore `app/src-tauri/Cargo.lock` if it changed.
`CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy --manifest-path app/src-tauri/Cargo.toml --features test-support --all-targets -- -D warnings`
→ `Finished`. `npm --prefix app run check` → `0 errors` (unless T11 has not
run, in which case `Admit.svelte` still passes `null` as a fourth argument to
a function that takes it: 0 errors either way). The org-node cargo line is
unchanged and green.

**Step 4 — commit.** `git add` the files of **Files touched**, then
`git -c commit.gpgsign=false commit -m "T10: the app uses org-node's new interface; whom each command sends to" -m "verifies: LLR-q225ws, LLR-8krgzj, LLR-ctrfz4, LLR-gha5f6, LLR-w4mhd4, LLR-7bk6qh"`.

**Red→green attestations:**
- each_command_sends_once_to_one_device_and_the_kind_follows_the_record — red -> green: compile red first (E0432 unresolved `commands::revoke_and_send`); then runtime reds with stubs: a `todo!()` body panicked (feature missing), and a stub sending to a Member other than the removed one failed at submit_flow.rs:198 "the removed Device is not listed" (Organisation information was sent); green with the plan's recipient choice. (A first stub picking trie_members[0] passed by accident — the joiner sorted first — and was re-cut.)
- admit_member_takes_no_organisation_secret (rewritten, renamed) — red -> green: compile red (lib E0432 `org_node::OrgSecret`, E0061, E0004); green once admit_member drops `org_secret_hex`.
- locally_reachable_variants_are_classified_as_receiver_errors (extended) — red -> green: compile red (E0004 in events.rs), then runtime red with a stub classifying the four as verdicts ("MalformedMessage is reachable from a local condition and must not be a verdict", receiver_events.rs:765); green with them in the receiver-error arm.
- a_reply_is_acted_on_once_and_only_if_its_invite_is_outstanding (re-asserted) — red -> green: compile red (lib E0432/E0061; the old assertion read `WireMessage.invite_id`); green after admit_reply and submit_commit_send lose org_secret/invite_id.
- a_confirmed_reply_carries_the_persona_and_declares_the_expected_admission (re-asserted) — red -> green: compile red (E0061 expect_admission arity, E0432 `org_node::InviteId`); green after produce_reply calls expect_admission(rng, org_id).
- founding_writes_the_chain_then_commits, an_admission_is_written_then_committed_then_sent (re-asserted) — red -> green: compile red (lib E0432/E0061; `org_private_key.is_some()` on a now-required field); green after submit_commit_send takes six arguments. Task done 2026-10-06, merged from worktree-org-node-org-key-pair-t10 (227946f); app 158 passed, 0 failed; app clippy --all-targets clean; svelte-check 0 errors. Found in execution: in Loopback mode send_update dials `peer_addr` without checking it is `recipient`'s Device — this is the open PR-2dmjzj (stated by LLR-jn5jeh), not fixed by this change.

---

### T11 — app webview: `admitMember` takes no Organisation secret

**Files touched:** `app/src/lib/api.ts`, `app/src/lib/components/Admit.svelte`,
`app/tests/api.admit.test.ts` (new)
**Parallel:** yes (no other task touches these files; the webview does not
build against org-node)
**IDs verified:** LLR-8krgzj (webview clause).
**Size:** ~50 lines.

**Step 1 — failing test** (`app/tests/api.admit.test.ts`, new):

```ts
/**
 * LLR-8krgzj: the webview's admitMember passes admit_member exactly the org
 * id, the reply and the peer address — no Organisation secret.
 * (`Admit.svelte` passing no fourth argument is enforced by `npm run check`:
 * svelte-check refuses a call with more arguments than the function takes.)
 */

import { describe, it, expect, vi } from 'vitest';

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn(async () => '00'.repeat(32)) }));
vi.mock('@tauri-apps/api/core', () => ({ invoke }));
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));

import { admitMember } from '../src/lib/api';

describe('admitMember', () => {
	// verifies: LLR-8krgzj
	it('invokes admit_member with the org id, the reply and the peer address alone', async () => {
		await admitMember('01'.repeat(20), 'reply', 'addr');
		expect(invoke).toHaveBeenCalledWith('admit_member', {
			orgId: '01'.repeat(20),
			replyBlob: 'reply',
			peerAddrBlob: 'addr'
		});
		expect(admitMember.length).toBe(3);
	});
});
```

Run `npm --prefix app ci --offline` (task worktree), then
`npm --prefix app run test`. Expected red: `expected "spy" to be called with
arguments: [ 'admit_member', { orgId: …, replyBlob: 'reply', peerAddrBlob:
'addr' } ]` — the received object has `orgSecretHex: null` too.

**Step 2 — implementation.**
- `app/src/lib/api.ts`: replace

```ts
/** REQ-nfr3n2: admit through the chain, then commit and send. */
export function admitMember(
	orgId: string,
	replyBlob: string,
	peerAddrBlob: string,
	orgSecretHex: string | null = null
): Promise<string> {
	return invoke<string>('admit_member', { orgId, replyBlob, peerAddrBlob, orgSecretHex });
}
```

  by

```ts
/**
 * REQ-nfr3n2: admit through the chain, then commit and send. No Organisation
 * secret or key is passed: the key the admission carries is the one
 * org-node's record holds (LLR-8krgzj).
 */
export function admitMember(orgId: string, replyBlob: string, peerAddrBlob: string): Promise<string> {
	return invoke<string>('admit_member', { orgId, replyBlob, peerAddrBlob });
}
```

- `app/src/lib/components/Admit.svelte`: replace the two lines
  `			// No Organisation secret is typed into the UI; the backend is sent none.`
  `			memberId = await admitMember(selectedOrgId, pastedBlob.trim(), peerAddrBlob.trim(), null);`
  by
  `			memberId = await admitMember(selectedOrgId, pastedBlob.trim(), peerAddrBlob.trim());`.

**Step 3 — green.** `npm --prefix app run test` → `Tests 44 passed`;
`npm --prefix app run check` → `svelte-check found 0 errors`.

**Step 4 — commit.** `git add app/src/lib/api.ts app/src/lib/components/Admit.svelte app/tests/api.admit.test.ts`,
then `git -c commit.gpgsign=false commit -m "T11: admitMember takes no Organisation secret" -m "verifies: LLR-8krgzj"`.

**Red→green attestations:**
- admitMember invokes admit_member with the org id, the reply and the peer address alone — red -> green: watched fail (AssertionError: extra `"orgSecretHex": null` in the invoke args) before admitMember was cut to three parameters; green after. vitest 44 passed, svelte-check 0 errors. Task done 2026-10-06, merged from worktree-org-node-org-key-pair-t11 (47855a0).

---

### T12 — docs, problem checks, deslop and the full gate

**Files touched:** `org-node/README.md`, `org-node/AGENTS.md`,
`org-node/src/transport/mod.rs`,
`org-node/docs/architecture/2026-10-05-unsigned-envelope.md`,
`org-node/docs/requirements/2026-10-03-member-identity.md`,
`app/docs/architecture/2026-10-05-decomposition.md`,
`org-node/.guardrails/config.yaml`, and any `org-node/src` file Step 1 finds
a stale comment in (listed in the commit message)
**Parallel:** no (serial, after T10 and T11)
**IDs verified:** none new; this task makes the whole change verifiable.
**Size:** ~120 lines.

**Step 1 — deslop org-node's source comments.** Run each, and fix every hit
that describes removed behaviour (an Organisation secret, an invite
identifier on the wire, an optional snapshot or key, a key that never
changes), rewording rather than deleting where a comment still says
something true:

```
grep -rn -i "secret" org-node/src
grep -rn -i "invite" org-node/src
grep -rn -i "genesis_snapshot\|snapshot" org-node/src
grep -rn -i "held only\|only in the creating\|record's key\|None on every" org-node/src
```

Expected after the fix: `secret` hits name only the secret types
(`MemberSeed`, `DeviceSeed`, `OrgPrivateKey`, `expose_secret`, the
`secret_type!` macro) or the store key; `invite` hits are prose about the
app's Invite only (`ExpectedAdmission`'s doc, the Persona-binding comments);
`snapshot` hits are `record_snapshot`, `MemberSnapshot` and their docs. In
particular `org-node/src/transport/mod.rs`'s
`pub const MAX_FRAME: usize = 1 << 20; // 1 MiB — generous for a delta + secret.`
becomes `pub const MAX_FRAME: usize = 1 << 20; // 1 MiB — generous for a delta, a record snapshot and a key.`
Rerun `--test absences` after editing.

*Steps 1, 3 and 4 done 2026-10-06, merged from worktree-org-node-org-key-pair-t12
(6d37f46): docs and comments only, no test added; org-node 244 passed, app
158 passed, 0 failed. PR-xwek5e, PR-szkat6, PR-ve9zw8 and PR-g9u3xq resolved
with existing tests; PR-3ue4va open. Beyond the plan's text: stale citations
fixed in PR-g7cfns's resolution, SDD-kwncn7's file line, and dated rename notes
in 2026-10-03-decomposition.md and 2026-10-03-architecture-derived.md. Left for
Step 2: app check-trace fails NON-EXPORTED-REF on T10's comment citing
LLR-j5vbqj in app/src-tauri/src/events.rs.*

*Step 2 done 2026-10-06 (fe2dbac, on the change branch): the events.rs
comment cites the app's LLR-7bk6qh (NON-EXPORTED-REF cleared);
`set_record_keys` folded into `commit_held`; the two receive paths' held-record
commit shared as `commit_received`; receive paths destructure the message;
test helpers `first_device`, `private_key_held`, `envelope_only` removed;
`revoke`/`revoke_and_tell` share one body; duplicated assertion loops merged;
imports and comment wraps tidied. Left alone, with reasons: the golden-value
history block, dated rename/PR narratives, absence tests, the app's own
InviteId, two unreachable but harmless `RevocationNotHeld` arms in
`receive_and_verify`, small per-target test duplication. Suite after: org-node
244, app 158, vitest 44 passed, 0 failed; clippy clean; check-trace clean.*

**Step 2 — the whole-change deslop pass.** *(Placeholder, dispatched
separately by the dispatcher after Step 1, as in the chain-authority change:
a deslop review of `git diff master...HEAD -- org-node/src app/src-tauri/src app/src`
for names, comments and dead code; its findings are fixed in this task's
worktree and listed here when done.)*

**Step 3 — docs.**
- `org-node/README.md`: replace `` `DeviceSeed`, `OrgSecret`, `OrgPrivateKey`), `OrgPublicKey` (parsed through``
  by `` `DeviceSeed`, `OrgPrivateKey`), `OrgPublicKey` (parsed through``;
  replace the two lines from
  ``Messages are typed as `WireMessage { envelope: Envelope, org_secret: Option<OrgSecret>, genesis_snapshot: Option<Vec<u8>>, invite_id: Option<InviteId> }` ``
  through `identifier is present only on an admission).` by:

```
Messages are typed as `WireMessage::OrgInformation { envelope, record_snapshot, org_private_key }`
(index 0: the committed Envelope, the record it extends, and the Organisation
private key the sending node's record holds) or `WireMessage::Revocation { envelope }`
(index 1: the Envelope alone). The kind follows the recipient: a Device the
sender's committed record lists receives Organisation information, any other
a revocation. No message carries an invite identifier.
```

- `org-node/AGENTS.md`: replace `- **Secrets** (member/device seed, Organisation secret, Organisation private`
  `  key, store key)` by `- **Secrets** (member/device seed, Organisation private`
  `  key, store key)`.
- `org-node/docs/architecture/2026-10-05-unsigned-envelope.md`, the
  robustness table: in the LLR-rys5nx row replace
  `` `a_first_admission_records_the_chains_key_the_secret_and_the_member` `` by
  `` `a_first_admission_records_the_chains_key_the_private_key_and_the_member` ``
  and `the record holds the chain's Organisation public key and no other key (LLR-xq9nrq)`
  by `the record holds the chain's Organisation public key (LLR-xq9nrq)`; in
  the LLR-3fwykc row replace
  `` `a_first_admission_records_the_chains_key_the_secret_and_the_member`: a member's record holds `None` ``
  by `` `pr_szkat6_an_admitted_member_holds_the_organisation_private_key`: a member's record holds the Organisation's key ``;
  in the LLR-2dvhz8 row replace
  `` `a_record_debug_says_whether_the_organisation_private_key_is_set` (the unset key of a member's record) ``
  by `` `a_record_debug_renders_the_organisation_private_key_redacted` (boundary bytes) ``;
  add under the table: `*Rows for LLR-rys5nx, LLR-3fwykc and LLR-2dvhz8 updated 2026-10-06 (change worktree-org-node-org-key-pair): tests renamed or rewritten with the Organisation secret's removal.*`
- `org-node/docs/requirements/2026-10-03-member-identity.md`: replace
  `` `first_admission_without_a_record_snapshot_is_refused` in `` by
  `` `a_revocation_about_an_organisation_not_held_is_refused_before_the_chain` (renamed 2026-10-06, change worktree-org-node-org-key-pair) in ``.
- `app/docs/architecture/2026-10-05-decomposition.md`: in the Normal/Abnormal
  lines under LLR-8krgzj replace
  `Normal: \`admit_member_does_not_refuse_a_32_byte_or_absent_org_secret\` (ipc).`
  `Abnormal: \`admit_member_refuses_an_org_secret_that_is_not_32_bytes\`,`
  `\`admit_member_refuses_an_org_secret_that_is_not_hex\` (ipc).`
  by `Normal and abnormal: \`admit_member_takes_no_organisation_secret\` (ipc,`
  `a stale \`orgSecretHex\` of any value is not read);`
  `\`api.admit.test.ts\` (the webview passes three values).`; under
  LLR-ctrfz4 replace
  `Normal: \`admit_member_does_not_refuse_a_32_byte_or_absent_org_secret\`,`
  by `Normal: \`admit_member_takes_no_organisation_secret\`,`.
- `org-node/.guardrails/config.yaml`: above `verify_commands`, add a dated
  comment block: `# 2026-10-06 (change worktree-org-node-org-key-pair): --test fuzz_wire_decode added (T4); per-target counts at the final run: <the counts from Step 5>.`

**Step 4 — problems.** Run, and paste the output into the commit message:

```
GR_CONFIG=org-node/.guardrails/config.yaml sh .guardrails/scripts/find-items.sh show PR-xwek5e
GR_CONFIG=org-node/.guardrails/config.yaml sh .guardrails/scripts/find-items.sh show PR-szkat6
GR_CONFIG=org-node/.guardrails/config.yaml sh .guardrails/scripts/find-items.sh show PR-ve9zw8
GR_CONFIG=org-node/.guardrails/config.yaml sh .guardrails/scripts/find-items.sh show PR-g9u3xq
GR_CONFIG=app/.guardrails/config.yaml sh .guardrails/scripts/find-items.sh show PR-3ue4va
```

Expected: the first four `status: resolved`, each resolution naming a test
that exists (`grep -n "fn <name>" org-node/tests/admission_sender.rs` for
each); PR-3ue4va `status: open`.

**Step 5 — the full gate.** Run every command of this plan's
**Verification** header, from a clean `target` for org-node and the app
(QUINT_HOME per Environment), and the gates:

```
GR_CONFIG=org-node/.guardrails/config.yaml sh .guardrails/scripts/check-trace.sh
GR_CONFIG=app/.guardrails/config.yaml sh .guardrails/scripts/check-trace.sh
GR_CONFIG=on-chain-client/.guardrails/config.yaml sh .guardrails/scripts/check-trace.sh
GR_CONFIG=org-node/.guardrails/config.yaml sh .guardrails/scripts/check-ids.sh --allow-draft-files
GR_CONFIG=app/.guardrails/config.yaml sh .guardrails/scripts/check-ids.sh --allow-draft-files
GR_CONFIG=on-chain-client/.guardrails/config.yaml sh .guardrails/scripts/check-ids.sh --allow-draft-files
sh .guardrails/scripts/check-units.sh
make coverage-on-chain-client
```

Expected: every cargo line `test result: ok` with `0 failed`; the Quint
lines pass; `npm --prefix app run check` 0 errors, `npm --prefix app run
test` 44 passed; `check-trace` reports no MISSING-TEST (none of LLR-38e2kn,
LLR-4kh9w9, LLR-6s785x, LLR-6ymd6d, LLR-ba2ejp, LLR-byjvd9, LLR-e2b7gv,
LLR-ecxc76, LLR-j5vbqj, LLR-js9dsu, LLR-pt32fx, LLR-qsjde3, LLR-xn5pwc,
REQ-bwx7eg, REQ-c29s93, REQ-vxqc5g, LLR-q225ws), no DANGLING-REF,
UNIMPLEMENTED-CONTROL or UNTRACED-DESIGN; its UNRESOLVED-PR lines no longer
list PR-szkat6, PR-xwek5e or PR-ve9zw8, and the app's list PR-3ue4va (open,
age 0); `check-ids` and `check-units` exit 0; coverage at or above its floors;
clippy `-D warnings` clean for org-node and the app. **Known blocker:** org-node's
`check-trace` reports `UNDECLARED-DEPENDENCY PR-3ue4va` from the requirements
draft's ruling prose (Open questions 2); the merge waits for that wording to
be fixed by the requirements owner. Paste the outputs (counts per target)
into the commit message.

**Step 6 — commit.** `git add` the files changed, then
`git -c commit.gpgsign=false commit -m "T12: docs, problem checks, deslop and the full gate for the Organisation key pair" -m "<the Step 4 and Step 5 outputs>"`.

**Red→green attestations:** (no new test) — gate outputs: (pending)

---

## Implements → tasks

Every ID of **Implements** and the task whose test verifies it, at the
lowest level. An RC is implemented by the REQ beside it; an SDD is evidenced
through its LLRs; a REQ through its LLRs (and directly where named).

| ID | Task(s) | Evidence (target) |
|---|---|---|
| REQ-szq3ud | T2, T4 (via LLR-6ymd6d, LLR-qsjde3, LLR-8hdu9x) | admission_sender, commit_paths, absences |
| REQ-stx9v3 | T8 (via LLR-e2b7gv, LLR-ghja3x, LLR-tax3pm, LLR-qjz3q4, LLR-sj7cd5) | commit_paths, organisation_key, provisional_store, admission_sender |
| REQ-jy6ybw | T8 (via LLR-6s785x, LLR-cmdrp9) | commit_paths, admission_sender |
| REQ-c29s93 (RC-9cefcn) | T4 (LLR-js9dsu), T6 (LLR-xn5pwc), T1 (LLR-j5vbqj) | wire_frame_bound, fuzz_wire_decode, receive_chain_reads, value_types |
| REQ-bwx7eg (RC-9cefcn) | T9 (LLR-ba2ejp), T1 (LLR-j5vbqj) | admission_sender, value_types |
| REQ-ju6vn2 | T2, T9 (LLR-ckk5nz, LLR-4kh9w9), T7 (LLR-byjvd9) | admission_sender, persona_records |
| REQ-3dsweu | T4 (LLR-6ymd6d, LLR-8hdu9x, LLR-js9dsu), T5 (LLR-pt32fx, LLR-jsx922) | commit_paths, wire_frame_bound, admission_sender |
| REQ-vxqc5g | T4 (LLR-38e2kn), T1 (LLR-j5vbqj) | admission_sender, receive_chain_reads, value_types |
| REQ-8amu2a | T3 (LLR-9zfnmb, LLR-s8xp7m, LLR-q8emds, LLR-mbjfq8, LLR-ms8njy, LLR-48jakr) | expected_admission, absences, encoding_golden |
| REQ-hzm4kt | T2 (LLR-s78sh7) | store_at_rest |
| REQ-y7tsft | T2, T4, T7 (LLR-bwb9pu, LLR-ecxc76, LLR-2dvhz8) | secret_redaction |
| REQ-tcutr6 | T10 (LLR-w4mhd4) | invitation (app) |
| RC-9cefcn | T4, T6, T9 (REQ-c29s93, REQ-bwx7eg) | as above; `pr_ve9zw8_…_on_both_paths` names it |
| SDD-swtd3w | T1, T2, T3 (LLR-j5vbqj, LLR-qsjde3, LLR-ms8njy, LLR-sz4xhc) | value_types, absences, node_value_types |
| SDD-kwncn7 | T4 (LLR-js9dsu, LLR-ecxc76) | wire_frame_bound, fuzz_wire_decode, secret_redaction, encoding_golden |
| SDD-af5vnt | T2, T3, T7, T8 (LLR-s78sh7, LLR-95753m, LLR-byjvd9, LLR-qjz3q4, LLR-7cmp38) | store_at_rest, expected_admission, persona_records, provisional_store, commit_paths |
| SDD-89es4z | T8 (LLR-sj7cd5); LLR-s6qnht already verified | organisation_key, commit_paths |
| SDD-rx2yvy | T4, T8 (LLR-6ymd6d, LLR-e2b7gv, LLR-ghja3x, LLR-8hdu9x) | commit_paths, organisation_key |
| SDD-8cpyfa | T4, T5, T6, T8, T9 (LLR-38e2kn, LLR-pt32fx, LLR-xn5pwc, LLR-6s785x, LLR-ba2ejp, LLR-4kh9w9) | admission_sender, receive_chain_reads, commit_paths |
| SDD-72ddm6 | T5, T8, T9 (LLR-pt32fx, LLR-tax3pm, LLR-ba2ejp, LLR-jwhzh3) | admission_sender, commit_paths |
| LLR-js9dsu | T4 | wire_frame_bound, fuzz_wire_decode, encoding_golden |
| LLR-ecxc76 | T4 | secret_redaction |
| LLR-j5vbqj | T1 (+ T6) | value_types, receive_chain_reads |
| LLR-qsjde3 | T2 | absences |
| LLR-byjvd9 | T7 | persona_records |
| LLR-6ymd6d | T4 | commit_paths, admission_sender |
| LLR-e2b7gv | T8 | commit_paths, organisation_key, admission_sender |
| LLR-xn5pwc | T6 | receive_chain_reads |
| LLR-ba2ejp | T9 | admission_sender |
| LLR-38e2kn | T4 | admission_sender, receive_chain_reads |
| LLR-6s785x | T8 | commit_paths, admission_sender |
| LLR-4kh9w9 | T9 | admission_sender |
| LLR-pt32fx | T5 | admission_sender |
| LLR-37cj3n | T9 | admission_sender |
| LLR-8hdu9x | T4 | commit_paths, admission_sender |
| LLR-9zfnmb | T3 | expected_admission |
| LLR-bg3vsw | T4 | commit_paths |
| LLR-ckk5nz | T2, T9 | admission_sender |
| LLR-ghja3x | T8 | commit_paths |
| LLR-j6j95z | T4 | admission_sender, expected_admission |
| LLR-jn5jeh | T4 (and T2/T3 re-runs) | commit_paths, admission_sender |
| LLR-jsx922 | T5 | admission_sender, service_stories |
| LLR-jwhzh3 | T2 | admission_sender (`a_self_delete_removes_only_…`, `membership_of_one_organisation_…`) |
| LLR-mbjfq8 | T3 (re-run), T9 | expected_admission, admission_sender |
| LLR-q8emds | T3 | expected_admission |
| LLR-s78sh7 | T2 | store_at_rest |
| LLR-tax3pm | T8 | commit_paths |
| LLR-u6rq4s | T5 | admission_sender |
| LLR-y2v8v2 | T2 | admission_sender (`a_receiver_holding_two_organisations_…`) |
| LLR-ayrdr8 | T2, T3, T4, T7, T8 | encoding_golden |
| LLR-bwb9pu | T2, T4 | secret_redaction |
| LLR-g76zqd | T2 (absences re-run; records hold typed keys) | absences, persona_records |
| LLR-sz4xhc | T2 | node_value_types |
| LLR-2dvhz8 | T7 | secret_redaction |
| LLR-322xfu | already verified (header) | node_value_types |
| LLR-3fwykc | T2, T7 | admission_sender, organisation_key |
| LLR-sj7cd5 | T8 | organisation_key |
| LLR-2xzys9 | T2, T3, T4 | commit_paths |
| LLR-48jakr | T3, T4 | absences, commit_paths |
| LLR-7cmp38 | T8 | commit_paths |
| LLR-95753m | T3, T8 | expected_admission, provisional_store, commit_paths |
| LLR-cmdrp9 | T8 | commit_paths |
| LLR-ms8njy | T3, T4 | absences, encoding_golden |
| LLR-mxskg9 | already verified (header); T1 extends its test | value_types |
| LLR-qjz3q4 | T8 | commit_paths, provisional_store |
| LLR-s6qnht | already verified (header) | commit_paths |
| LLR-s8xp7m | T3 | expected_admission |
| LLR-wzqqg9 | T2, T7 (commit_genesis test re-asserted) | commit_paths |
| LLR-q225ws (app) | T10 | submit_flow, invitation |
| LLR-8krgzj (app) | T10, T11 | ipc, api.admit.test.ts |
| LLR-ctrfz4 (app) | T10 | ipc |
| LLR-gha5f6 (app) | T10 | invitation |
| LLR-w4mhd4 (app) | T10 | invitation |
| LLR-f35pda (app) | T10 (its tests recompile against the app's `InviteId`, unchanged in assertion) | invitation |

## Tests deleted, and what verifies their IDs instead

| Task | Deleted test | ID(s) | Replacing evidence |
|---|---|---|---|
| T2 | `commit_paths::a_nodes_own_commit_keeps_its_secret` | LLR-ckk5nz | T8's `commit_update_commits_a_provisional_update_the_chain_carries` (LLR-6s785x: the own commit takes the update's key); T2/T9's admission_sender tests for LLR-ckk5nz's received-key clause |
| T3 | `expected_admission::a_first_admission_under_another_invite_id_is_refused_and_keeps_the_expectation` | REQ-8amu2a, LLR-s8xp7m, LLR-ms8njy | `absences::org_node_holds_no_invite_identifier` (LLR-ms8njy); `expected_admission::an_expectation_for_one_organisation_admits_no_other`, `…_lists_none_of_our_personas_is_refused` (pre-emption, REQ-kt877x) |
| T3 | `wire_frame_bound::a_wire_message_carries_its_invite_id_and_refuses_a_short_one` | LLR-ms8njy, REQ-8amu2a | `absences::org_node_holds_no_invite_identifier`; `encoding_golden::*_wire_message_is_pinned` |
| T10 | `ipc::admit_member_refuses_an_org_secret_that_is_not_32_bytes`, `…_that_is_not_hex` | LLR-8krgzj | `ipc::admit_member_takes_no_organisation_secret`; `api.admit.test.ts` |

Renamed (rewritten under a new name): `a_first_admission_records_the_chains_key_the_secret_and_the_member`
→ `…_the_private_key_and_the_member` (T2); `pr_xwek5e_another_members_revocation_clears_the_receivers_secret`
→ `pr_xwek5e_another_members_removal_leaves_the_receiver_holding_the_organisations_key`
(T2); `send_update_sends_the_committed_update_and_writes_nothing` →
`send_update_sends_the_committed_update_with_the_records_key_and_writes_nothing`
(T2) → `send_update_sends_organisation_information_to_a_listed_device_and_a_revocation_to_any_other`
(T4); `organisation_secret_does_not_appear_in_the_file` →
`organisation_private_key_does_not_appear_in_the_file` (T2);
`expect_admission_records_a_pair_once_and_reaches_the_disk` →
`…_records_an_organisation_once_…` (T3);
`a_second_expectation_for_the_same_organisation_survives_the_commit` →
`an_expectation_for_another_organisation_survives_the_commit` (T3);
`first_admission_without_a_record_snapshot_is_refused` →
`a_revocation_about_an_organisation_not_held_is_refused_before_the_chain`
(T4); `admission_wire_message_is_pinned` →
`organisation_information_wire_message_is_pinned` (T4);
`a_record_debug_says_whether_the_organisation_private_key_is_set` →
`a_record_debug_renders_the_organisation_private_key_redacted` (T7);
`the_genesis_private_key_lives_only_in_its_provisional_update` →
`every_updates_private_key_lives_only_in_its_provisional_update` (T8);
`admit_member_does_not_refuse_a_32_byte_or_absent_org_secret` →
`admit_member_takes_no_organisation_secret` (T10). T12 updates the ledger
citations of the renamed ones.

## Existing tests annotated with an amended item, and the task that rewrites them

Found with a scan of every `verifies:` annotation under `org-node/tests`,
`org-node/src`, `app/src-tauri` and `app/src` for the IDs this change
amends. A test listed "re-run" asserts nothing the amendment changes; it is
re-run green by the task named.

| Test (target) | Amended ID(s) it names | Task |
|---|---|---|
| admission_sender: update_from_the_admin_after_admission_is_committed | LLR-u6rq4s | re-run T2, T4 |
| admission_sender: update_relayed_by_a_non_member_after_admission_is_committed | LLR-37cj3n, LLR-u6rq4s | re-run T2, T9 |
| admission_sender: a_committed_admission_reaches_the_disk_and_clears_the_expectation | LLR-q8emds | re-run T3 |
| admission_sender: an_admission_reaches_the_administrators_disk | LLR-cmdrp9 | re-run T8 |
| admission_sender: first_admission_without_a_record_snapshot_is_refused | LLR-j6j95z | rewritten T4 |
| admission_sender: a_revocation_reaches_the_administrators_disk | LLR-8hdu9x | T2, rewritten T4 |
| admission_sender: an_update_that_does_not_revoke_us_reaches_the_disk | LLR-jsx922 | re-run T5 |
| admission_sender: the_admission_envelope_carries_the_epoch_its_update_produced | LLR-ghja3x | T4 (`envelope()`), re-run T8 |
| admission_sender: in_loopback_mode_the_joiner_is_dialled_at_the_full_address | LLR-jn5jeh | T2, T4 |
| admission_sender: a_receiver_holding_two_organisations_commits_into_the_one_the_change_names | LLR-9zfnmb, LLR-q8emds, LLR-y2v8v2 | T2, T3 |
| admission_sender: a_self_delete_removes_only_the_organisation_the_revocation_came_from | LLR-jwhzh3 | T2 |
| admission_sender: membership_of_one_organisation_is_judged_by_that_organisations_personas_alone | LLR-jsx922, LLR-jwhzh3 | re-run T2, T5 |
| admission_sender: a_removal_relayed_by_the_member_it_removes_is_committed | LLR-u6rq4s | rewritten T5 |
| admission_sender: a_first_admission_records_the_chains_key_the_secret_and_the_member | LLR-3fwykc, LLR-ckk5nz | rewritten T2, extended T9 |
| admission_sender: pr_xwek5e_another_members_revocation_clears_the_receivers_secret | LLR-8hdu9x, LLR-ckk5nz | rewritten T2 |
| admission_sender: a_first_admission_to_an_expected_organisation_rests_on_the_chain_alone | LLR-mbjfq8, REQ-8amu2a | T3 |
| admission_sender: a_first_admission_that_fails_verification_leaves_the_expectation | LLR-q8emds | T4 |
| admission_sender: a_first_admission_that_misses_the_chain_root_commits_nothing | LLR-mbjfq8 | T3 |
| commit_paths: create_organisation_keeps_a_genesis_update_and_nothing_else | LLR-qjz3q4, LLR-s6qnht | re-run T8 |
| commit_paths: commit_genesis_creates_the_record_once_the_chain_carries_the_root | LLR-3fwykc, LLR-qjz3q4, LLR-wzqqg9 | T2, T7 |
| commit_paths: commit_genesis_refusals_change_nothing_and_write_nothing | LLR-mxskg9, LLR-wzqqg9 | re-run T7 |
| commit_paths: a_chain_past_epoch_one_refuses_commit_genesis_and_keeps_the_update | LLR-wzqqg9 | re-run T7 |
| commit_paths: a_genesis_whose_members_do_not_rebuild_the_root_is_refused | LLR-wzqqg9 | re-run T7 |
| commit_paths: admit_member_keeps_a_provisional_update_and_changes_nothing_else | LLR-ghja3x | rewritten T8 |
| commit_paths: commit_update_commits_a_provisional_update_the_chain_carries | LLR-bg3vsw, LLR-cmdrp9 | rewritten T8 |
| commit_paths: commit_update_refusals_change_nothing_and_write_nothing | LLR-cmdrp9 | rewritten T8 |
| commit_paths: send_update_sends_the_committed_update_and_writes_nothing | LLR-2xzys9, LLR-48jakr, LLR-8hdu9x, LLR-jn5jeh, REQ-8amu2a | rewritten T2, T3, T4 |
| commit_paths: send_update_refuses_without_a_bound_persona_or_a_loopback_address | LLR-2xzys9 | T2, T3 |
| commit_paths: a_nodes_own_commit_keeps_its_secret | LLR-ckk5nz | deleted T2 |
| commit_paths: the four discard tests | LLR-7cmp38 | rewritten T8 |
| commit_paths: revoke_member_keeps_a_provisional_update_and_changes_nothing_else | LLR-tax3pm | rewritten T8 |
| commit_paths: a_commit_that_keeps_one_of_our_personas_is_an_update | LLR-jsx922 | T2 |
| encoding_golden: all three | LLR-ayrdr8, LLR-ms8njy | T2, T3, T4, T7, T8 |
| expected_admission: all nine | LLR-mbjfq8, LLR-q8emds, LLR-s8xp7m, LLR-y2v8v2, LLR-ms8njy, LLR-mxskg9, LLR-95753m, LLR-9zfnmb, LLR-j6j95z, REQ-8amu2a | T3 (one deleted), T4 |
| node_value_types: three secret-type tests | LLR-sz4xhc | T2 |
| node_value_types: the_organisation_private_key_is_a_secret_type_… | LLR-322xfu | unchanged |
| organisation_key: a_created_organisation_publishes_a_fresh_key_…, the_organisation_private_key_is_kept_only_in_the_encrypted_store, the_organisation_private_key_is_not_in_the_record_debug_output | LLR-3fwykc, LLR-sj7cd5, LLR-qjz3q4, LLR-2dvhz8 | T7 |
| organisation_key: an_organisation_key_equal_to_a_genesis_key_is_refused, …_any_genesis_member_or_device_key_is_refused | LLR-sj7cd5 | re-run T8 |
| persona_records: create_persona_holds_…, a_persona_record_decoded_directly_… | LLR-g76zqd | re-run T2, T7 |
| provisional_store: all four annotated | LLR-95753m, LLR-g76zqd, LLR-qjz3q4, LLR-mxskg9 | T3, T8 |
| secret_redaction: four annotated | LLR-2dvhz8, LLR-bwb9pu, LLR-322xfu | T2, T4, T7 |
| service_lifecycle: a_new_persona_is_proposed_… | REQ-hzm4kt | re-run T2 |
| service_stories: five_stories_full_e2e | LLR-bg3vsw, LLR-q8emds | T2 |
| service_stories: revocation_of_another_member_is_committed_not_self_deleted | LLR-jsx922 | rewritten T5 |
| store_at_rest: seven | REQ-hzm4kt, LLR-s78sh7 | T2 (one rewritten), T7 |
| value_types: three annotated | LLR-mxskg9, REQ-8amu2a | T1 |
| wire_frame_bound: a_wire_message_carries_its_invite_id_… | LLR-ms8njy, REQ-8amu2a | deleted T3 |
| absences: org_node_has_no_administrator_key | LLR-g76zqd | re-run T2 |
| app ipc: three org-secret tests, four peer-address tests | LLR-8krgzj, LLR-ctrfz4 | T10 (two deleted, one rewritten, four re-argued) |
| app ipc: produce_invite_reply_refuses_without_confirmation_over_ipc | LLR-w4mhd4 | re-run T10 |
| app invitation: nine annotated | LLR-f35pda, LLR-w4mhd4, LLR-gha5f6 | T10 |

**Fuzz targets and golden encodings.** `fuzz_envelope_decode` and
`fuzz_verify_against_chain` read the Envelope and the verify function, which
this change does not touch: unchanged. `fuzz_first_admission_base` loses its
`None` prelude and takes the snapshot bytes (T4). `fuzz_wire_decode` is new
(T4). Every golden value is re-derived in T2, T3, T4, T7 and T8.

## Open questions and design gaps (this plan's default in brackets)

*Closed 2026-10-06, before T1 was dispatched:* 1 — LLR-7bk6qh amended in
place with the plan's default (all four are receiver errors); 2 — the
requirements draft cites the app's problem file by path; 3 — SDD-swtd3w's note
names four variants and traces REQ-3dsweu; 4 — LLR-rys5nx amended to "no
other public key". 5 and 6 stand as written.

1. **LLR-7bk6qh is not amended for the four new `OrgNodeError` variants**
   (design gap; blocks T10's merge, not its execution). The app's
   `classify_receive_error` matches `OrgNodeError` exhaustively, and
   LLR-7bk6qh lists by name the variants it classifies as receiver errors.
   [All four are receiver errors, by the rule the item's own 2026-10-06 note
   states: a verdict is a variant `verify_envelope_against_chain` produces;
   `OrgKeyMismatch` and `RevocationNotForThisDevice` arise after verification
   but on a rule about the key or this node, as `AdmissionNotOurs` does.] The
   design owner amends LLR-7bk6qh in place before T10 is merged.
2. **`UNDECLARED-DEPENDENCY PR-3ue4va` in org-node's `check-trace`** (gap in
   the requirements draft; blocks the merge gate). The ruling prose of
   `org-node/docs/requirements/2026-10-07-org-key-pair.md`
   names "the app's PR-3ue4va"; org-node's config does not depend on the app
   (and must not). [The requirements owner rewords the reference — e.g. "the
   app's fan-out problem report (`app/docs/problems/2026-10-07-update-fan-out.md`)"
   — before T12's gate.]
3. **SDD-swtd3w's amendment note says `OrgNodeError` gains three variants**
   (`MalformedMessage`, `OrgKeyMismatch`, `RevocationNotHeld`), while
   LLR-j5vbqj, under it, adds four (and `RevocationNotForThisDevice`), and the
   item's `traces:` line does not name REQ-3dsweu, which LLR-j5vbqj satisfies.
   [Implemented as LLR-j5vbqj states; the design owner corrects the note and
   the trace line.] Not blocking: `check-trace` does not compare them today.
4. **LLR-rys5nx** says a first-admission record "holds no other key of the
   Organisation", which now reads against REQ-ju6vn2 (the record holds the
   Organisation private key the message carried). [Read as "no other public
   key / no administrator key", its evident intent; T2 drops the
   "no other key" assertion from the test that verifies it.] The design owner
   may want to amend it in place; not blocking.
5. **Between T4 and T5 a held revocation that leaves this node listed is
   committed** as an update keeping the record's key (Decision 3). No test
   depends on the interim, and T5 follows directly.
6. **The fan-out** (PR-3ue4va, open): the app sends each committed update to
   one Device only; every other current Member stays on the earlier epoch's
   key until it receives Organisation information. By owner ruling this is
   the `org-io` session's work [not changed here].

## Self-review (plan-change step 9)

1. Every ID of **Implements** has a task whose test verifies it, at the
   lowest level (the table above): each new LLR has a new test in T1, T2, T4,
   T5, T6, T7, T8 or T9 (app LLR-q225ws in T10); each amended LLR is either
   rewritten by a task or listed as already verified with its unchanged test
   (LLR-322xfu, LLR-s6qnht, LLR-mxskg9); each REQ, the RC and each SDD is
   covered through its LLRs.
2. Every task's code steps show real code against the tree as the previous
   task leaves it, real commands, and the expected red and green. Where a step
   migrates many call sites it gives the exact rewrite (or the `replace_all`
   pattern) and the grep that proves none is left. Golden values are derived
   by the format rule, offsets stated (Decision 6), and written out in full.
3. Names are used consistently across tasks: `WireMessage::{OrgInformation
   { envelope, record_snapshot, org_private_key }, Revocation { envelope }}`
   and `WireMessage::envelope()`; `OrgNodeError::{MalformedMessage,
   OrgKeyMismatch { org_id }, RevocationNotHeld { org_id },
   RevocationNotForThisDevice { org_id }}`; `send_update(outgoing, recipient,
   peer_addr)` (from T3; T2's interim takes `invite_id` last);
   `expect_admission(rng, org_id)`, `ExpectedAdmission { org_id }`;
   `first_admission_base(&[u8])`; `ProvisionalChange::ChangeSet { change_set,
   org_private_key }`; `discard_provisional(rng, target, resulting_root,
   org_pub_key)`; service helpers `commit_held(org_id, verified)`,
   `store_carried_key` (T2–T8), `set_record_keys` (T8–), `check_carried_key`
   (T9–), `keep_change_set`; support helpers `private_key_held`,
   `private_key_of`, `revoke_and_tell`, `envelope_only`, `with_envelope`,
   `with_snapshot`, `with_key`, `deliver_raw`, `admit(svc, chain, org,
   joiner, addr)`; app `invitation::InviteId`, `admit_reply(svc, writer,
   outstanding, rng, org_id, reply_blob, peer_addr)`,
   `submit::submit_commit_send(svc, writer, rng, update, recipient,
   peer_addr)`, `commands::revoke_and_send(svc, writer, rng, org_id, member,
   peer_addr)`, `admitMember(orgId, replyBlob, peerAddrBlob)`.
4. Every task states **Files touched** and **Parallel**. The tasks marked
   parallel — T1 (`org-node/src/error.rs`, `org-node/tests/value_types.rs`)
   and T11 (`app/src/lib/api.ts`, `app/src/lib/components/Admit.svelte`,
   `app/tests/api.admit.test.ts`) — name no file any other task names. Every
   edit of `org-node/src/service.rs`, `org-node/tests/support/mod.rs`,
   `org-node/Cargo.toml` and `org-node/.guardrails/config.yaml` sits in the
   serial T2–T9 chain or T12; the app's Rust files are T10's alone.

## Hand-off

Execute with `develop-change`, task by task in the order above (T1, T2 and
T11 may start at once; T4 waits for T1 and T3), each in its own task worktree
(`sh .guardrails/scripts/task-worktree.sh start tN`), filling that task's
**Red→green attestations** with the red output, the green output and the
commit. Then `check-traceability`, `verify-before-merge` and `merge-change`,
after the design and requirements owners have closed Open questions 1 and 2.

## Gate 1 (2026-10-06, HEAD cc50f0f, tree f3e7584)

Every verify command of org-node, app and on-chain-client passed: cargo
244 + 158 + 73 + 34 passed, 0 failed, 0 ignored; vitest 44 passed; quint
typechecks exit 0 and five invariants "No violation found"; seven fuzz targets
ran 1 s each without failure (they print no counts). check-trace, check-ids
and check-units passed for every unit. Implements map: no uncovered ID.
org-node `clippy --all-targets` has 32 findings, all on test-file lines this
change did not touch (pre-existing debt); `--lib --bins` is clean, as is the
app's `--all-targets`.

Coverage: short of class C (on-chain-client 41.75% lines, 42.60% regions,
decisions unmeasured; org-node and app have no coverage command).
**Owner-accepted for this merge, 2026-10-07.**

Robustness: 13 LLRs lacked a normal or an abnormal test (LLR-6ymd6d, 8hdu9x,
4kh9w9, bg3vsw, ghja3x, tax3pm, s6qnht, 3fwykc, jwhzh3, xn5pwc; LLR-jn5jeh's
abnormal case is the open PR-2dmjzj, out of scope). Fix dispatched as task
`robust`; the gate runs again after it merges.

### Task robust — robustness gaps from gate 1 (done 2026-10-07, de38ca5)

New tests (red shown by a production mutation, then reverted):
- send_update_refuses_an_organisation_it_holds_no_record_of — verifies: LLR-6ymd6d — red -> green: with `send_update` looking up `orgs.first()` instead of the named Organisation, failed "left: Chain(\"no persona bound to organisation …\"), right: OrgNotOnChain"; green after revert.
- a_joiner_that_missed_its_admission_rebuilds_from_the_next_updates_snapshot — verifies: LLR-bg3vsw — red -> green: with `commit_update` encoding the after-snapshot, panicked "B rebuilds from the snapshot before the update: DeltaBaseMismatch"; green after revert.
- a_revocation_key_equal_to_the_records_or_to_a_key_of_the_resulting_record_is_refused — verifies: LLR-tax3pm — red -> green: with `keep_change_set`'s equality check made never-true, failed "unwrap_err() on an Ok value: ProvisionalUpdate { … }"; green after revert.

Annotations added to existing tests that already exercise the case:
LLR-8hdu9x and LLR-4kh9w9 (abnormal) on a_revocation_that_leaves_this_device_listed_is_refused_on_both_paths;
LLR-ghja3x (abnormal) on an_update_key_equal_to_the_records_or_to_a_key_of_the_resulting_record_is_refused;
LLR-s6qnht (abnormal) on an_organisation_key_equal_to_a_genesis_key_is_refused;
LLR-3fwykc (abnormal) on pr_ve9zw8_an_update_carrying_a_substituted_key_is_refused_on_both_paths;
LLR-jwhzh3 (abnormal) on the_self_delete_path_refuses_an_unheld_organisation_without_a_chain_read;
LLR-xn5pwc (normal) on a_committed_update_gives_the_record_the_new_key_pair_on_both_paths.
LLR-jn5jeh untouched (PR-2dmjzj). org-node 247 passed, 0 failed (a first run
timed out two admission_sender network tests under load; they passed on rerun).

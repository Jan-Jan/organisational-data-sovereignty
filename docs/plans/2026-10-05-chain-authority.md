# Chain authority — Implementation Plan

**Goal:** implement change 1 of the chain-authority sequence: org-node has no
administrator, builds provisional updates (create, admit, revoke) that it keeps
in the Persona store and commits only once they verify against the chain, runs
every chain-free check before it reads the chain, reads the chain for a first
admission only when the app expects it under that invite identifier, commits a
first admission only when it lists one of its own Personas, and writes nothing
to the chain; the chain writer moves to on-chain-client behind a `write`
feature; the invitation exchange and the chain submission move to the app.

*Revised 2026-10-06 after master (`5f7c177` person-types switch, `d8b9f9b` app
architecture ledger, `06c357b` guardrails 35570e8) was merged into this change
(`1f483c8`) and the owner ruled on pre-emption (`d233933`).* Master already
did what T4 planned — the Envelope carries no signature, `BadSignature` and
`VerifyContext`'s key are gone, nothing about a sender is checked on either
Receive operation, a first admission needs no Invite — and more: the
Member-as-a-group key and the Organisation public key are X25519 keys, genesis
draws its own Organisation key pair and keeps the private key (REQ-ech45n),
and an Envelope's Sequence number must equal the chain epoch it is verified
against (REQ-txvtm9). T4 is therefore withdrawn, and T5–T17 are re-derived
against the merged tree: every code block below is written against it, every
golden value is re-derived from master's pinned bytes, and every `verifies:`
list names the items as they now read. T1, T2 and T3 are done and merged;
their sections and attestations are unchanged.

**Implements:**

- org-node requirements: REQ-xs4ab8, REQ-uv3v5w, REQ-fwfku9, REQ-tqap3r,
  REQ-f2k4tr, REQ-8amu2a (as amended 2026-10-06), REQ-kt877x, REQ-hhva9d
  with LLR-7cmp38 (T10b), REQ-yp75u9 with LLR-6z5xya and LLR-eyc4ud (R1a);
  on-chain-client LLR-24mwew (R1b); app LLR-rt8gdz (R1c) (new,
  `org-node/docs/requirements/2026-10-06-chain-authority.md`);
  REQ-nhe2zu, REQ-xa6smf, REQ-uxv2x2 (`2026-09-09-verify-and-commit.md`),
  REQ-d9g6nt (`2026-10-03-member-identity.md`), REQ-qn2erx
  (`2026-10-04-type-safety.md`), REQ-txvtm9 and REQ-ech45n
  (`2026-10-05-envelope-authenticity.md`) — master's or amended in place; this
  change's rewritten tests verify them again on the new paths.
- org-node controls: RC-mj6gjq, RC-2ferct (new,
  `org-node/docs/risk/2026-10-06-chain-authority.md`).
- org-node design: the low-level requirements of (nineteen when this plan was
  written; twenty-two after LLR-7cmp38, LLR-6z5xya and LLR-eyc4ud)
  `org-node/docs/architecture/2026-10-06-chain-authority.md`
  (LLR-mxskg9, LLR-fuq379, LLR-ms8njy, LLR-95753m, LLR-jq7qh7, LLR-nvn3wk,
  LLR-qjz3q4, LLR-s6qnht, LLR-wzqqg9, LLR-2xzys9, LLR-48jakr, LLR-9ew26y,
  LLR-s8xp7m, LLR-3f5h7b, LLR-cmdrp9, LLR-ewkg85, LLR-4tcxsu, LLR-mkj4bz,
  LLR-b27jr6); the items amended in place in `2026-10-03-decomposition.md`
  whose code this plan changes — SDD-swtd3w, SDD-kwncn7, SDD-af5vnt,
  SDD-vee2fq, SDD-msb6xh, SDD-rq6nv4, SDD-ueh4tm, SDD-89es4z, SDD-rx2yvy,
  SDD-8cpyfa, SDD-72ddm6, SDD-z85ux9 and LLR-ctzkv7, LLR-g9vmbx, LLR-kkj64b,
  LLR-8qxwst, LLR-tcft2r, LLR-rv4vux, LLR-rc74nq, LLR-txqmz4, LLR-66h529,
  LLR-463d89, LLR-8m3bwj, LLR-f74xwb, LLR-65py3d, LLR-ryzr8m, LLR-68yd3j,
  LLR-rjg3m2, LLR-w3fhhg, LLR-q3aj8z, LLR-dzte8x, LLR-rb8r65, LLR-ghja3x,
  LLR-bg3vsw, LLR-jn5jeh, LLR-t4znbk, LLR-vdyu65, LLR-3v5nu9, LLR-zj88e6,
  LLR-qezw3n, LLR-9zfnmb, LLR-437fvx, LLR-836z24, LLR-j83kc8, LLR-cja9zv,
  LLR-3wb7th, LLR-mbjfq8, LLR-y2v8v2, LLR-xq9nrq, LLR-ckk5nz, LLR-e5c9ud,
  LLR-q8emds, LLR-379hnv, LLR-6dc598, LLR-tax3pm, LLR-qg9utu, LLR-pw369n,
  LLR-8hdu9x, LLR-drgdy8, LLR-6p4pj2, LLR-jsx922; in
  `2026-10-04-type-safety.md` LLR-s7whrn, LLR-ayrdr8, LLR-g76zqd, LLR-8bum44;
  in `2026-10-05-unsigned-envelope.md` LLR-rys5nx, LLR-sj7cd5, LLR-3fwykc,
  LLR-2dvhz8.
- on-chain-client: REQ-6jefu2, REQ-aat4yt
  (`on-chain-client/docs/requirements/2026-10-06-chain-write.md`);
  SDD-yg7n55 and LLR-a2acvh, LLR-z9vugt, LLR-qs8jfw, LLR-kv27gp, LLR-hun4wf,
  LLR-yvq33e, LLR-5varjf, LLR-d4ftc8, LLR-m7wmmx, LLR-ywhd23, LLR-gjx3jn,
  LLR-kqxw9t, LLR-rxs5ec
  (`on-chain-client/docs/architecture/2026-10-06-chain-write.md`)
  — done in T1 and T2.
- app: REQ-prjja8, REQ-tcutr6, REQ-65xqp8, REQ-ab2mfz, REQ-yazum3, REQ-nfr3n2,
  RC-wzb48r (`app/docs/requirements/2026-10-06-invitation.md`,
  `app/docs/risk/2026-10-06-invitation.md`);
  LLR-b7wgpf, LLR-9sraks, LLR-f35pda, LLR-w4mhd4, LLR-gha5f6, LLR-qhjp6g,
  LLR-be3zv9, LLR-n2u4uf
  (`app/docs/architecture/2026-10-06-invitation.md`);
  amended in place in `app/docs/architecture/2026-10-05-decomposition.md`:
  SDD-2pa6h6, SDD-rmbr3t, SDD-jx363y, SDD-6g3wnh, LLR-ctrfz4, LLR-pmus9f,
  LLR-7bk6qh, LLR-vzf8j2, LLR-4wcyqy.

**Amended for this change and already verified on master (no task):**
REQ-ag6kqm, REQ-ztdza4, RC-pm9kmx, RC-b6mydy, SDD-sxp8hb, SDD-kk2y3e,
SDD-na9nc3 (its key), SDD-pa6p7w, LLR-na7p4w, LLR-e58j8m, LLR-9fvb3y,
LLR-e7s4ye, LLR-p8uu47, LLR-ybn5pr, LLR-cs4mpb, LLR-9sknpa, LLR-pzde8b,
LLR-mcdh85, LLR-v873fx, LLR-z8fubr, LLR-37cj3n, LLR-u6rq4s, LLR-3q63zv,
LLR-s7yu4k. Master's switch-trim (`docs/plans/2026-10-05-switch-trim.md`)
wrote their tests; this plan changes none of their assertions.

**Resolves (owner ruling 2026-10-05):** PR-b9wab3 (T11,
`org-node/docs/problems/2026-10-04-revoke-precondition.md`), PR-322qst (T12,
`org-node/docs/problems/2026-10-04-own-revocation-on-receive.md`). PR-u4c2vp
(`org-node/docs/problems/2026-09-09-org-node-problems.md`) is resolved by
this change's ruling too — nothing about a sender is checked on any path —
and master's switch-trim already wrote that resolution and its test
(`pr_u4c2vp_an_update_relayed_by_a_non_member_is_committed_on_the_self_delete_path`),
so no task remains for it; T17 checks the resolution still names a test that
exists. PR-vt244s, PR-xwek5e, PR-szkat6, PR-ve9zw8 and PR-8qsnhx stay OPEN
(PR-szkat6 and PR-ve9zw8 are the key-pair change's); their pins are rewritten
where the code they pin moves, and still pin the defect. *(Updated after
review round 1:* PR-mdv38y, listed here at first, is resolved by this change —
REQ-yp75u9, task R1a. Opened by this change: PR-kwwap5, PR-924ftr (app),
PR-b795an (on-chain-client).)

**Safety class:** C (org-node), C (on-chain-client), C (app), from each
unit's `.guardrails/config.yaml`. No per-item overrides.

**Verification:** every `verify_commands` entry of the three units, as this
plan leaves them:

org-node (`org-node/.guardrails/config.yaml`):

```
cargo test -p org-node --features app,test-support --lib --test service_stories --test transport_handshake --test transport_networked --test fuzz_envelope_decode --test fuzz_verify_against_chain --test fuzz_first_admission_base --test verify_against_chain --test wire_frame_bound --test store_at_rest --test admission_sender --test value_types --test key_custody --test envelope_binding --test service_lifecycle --test encoding_golden --test node_value_types --test chain_read_state --test persona_records --test secret_redaction --test organisation_key --test receive_chain_reads --test expected_admission --test absences --test provisional_store --test commit_paths
quint --version
quint typecheck org-node/quint/protocol.qnt
quint typecheck org-node/quint/ods_instances.qnt
quint run org-node/quint/protocol.qnt --invariant=forkSafety --max-steps=16 --max-samples=5000
quint run org-node/quint/protocol.qnt --invariant=revocationSafety --max-steps=16 --max-samples=5000
quint run org-node/quint/protocol.qnt --invariant=revokedExcludedFromOrgSecret --max-steps=16 --max-samples=5000
quint run org-node/quint/protocol.qnt --invariant=tauWindow --max-steps=16 --max-samples=5000
quint run org-node/quint/protocol.qnt --invariant=convergence --max-steps=16 --max-samples=5000
```

The line loses `blob_exchange` (T7), `chain_write_pure` and `calldata_typed`
(T13), and gains `receive_chain_reads` (T5), `expected_admission` (T6),
`absences` (T7), `provisional_store` (T8) and `commit_paths` (T9).

on-chain-client (`on-chain-client/.guardrails/config.yaml`; the write line
gained `--test write_events` in R1b):

```
cargo test --manifest-path on-chain-client/Cargo.toml --features test-support --lib --test fuzz_decode_org_state --test fuzz_parse_revive_event --test fuzz_event_round_trip --test contract_address_filter --test log_ownership --test decode_revive_event --test decode_org_state --test runtime_version_dispatch --test h160_mapping --test storage_slot_layout --test best_lane_reorg_rule --test type_widths
cargo test --manifest-path on-chain-client/Cargo.toml --features test-support,write --test write_manifest --test write_pure --test write_compose --test write_events
```

app (`app/.guardrails/config.yaml`):

```
cargo test --manifest-path app/src-tauri/Cargo.toml --features test-support --test startup_policy --test connection_status --test org_id_parsing --test receiver_events --test receiver_guard --test ipc --test csp_policy --test state_assembly --test submit_flow --test invitation
npm --prefix app run check
npm --prefix app run test
```

The line gains `submit_flow` (T14) and `invitation` (T15).

Gates, per unit (`<unit>` in org-node, on-chain-client, app):
`GR_CONFIG=<unit>/.guardrails/config.yaml .guardrails/scripts/check-trace.sh`,
`GR_CONFIG=<unit>/.guardrails/config.yaml .guardrails/scripts/check-ids.sh --allow-draft-files`,
and once `.guardrails/scripts/check-units.sh`. Coverage:
`make coverage-on-chain-client` (on-chain-client's `coverage_command`;
unchanged by this plan — see Open questions); org-node and app have no
`coverage_command` (a recorded gap in their configs, unchanged).

---

## Environment (every task)

- Work in the task worktree the dispatcher creates from
  `worktree-org-node-chain-authority` (`.guardrails/scripts/task-worktree.sh
  start <tag>` in the change worktree; it also copies the ignored build
  directories). One git command per shell call; no `&&`, `;` or loops on a
  line that mentions git; write any file whose text contains the word "git"
  with the Write tool.
- `CARGO_HOME=/tmp/cargo_home_fuzz` on every cargo command (`~/.cargo` is
  read-only). Add `--offline` if the network is blocked; every crate this plan
  needs is already in one of the three lockfiles.
- Quint lines: `QUINT_HOME=<task worktree>/target/quint_home`, populated once
  with `mkdir -p <task worktree>/target/quint_home` then
  `cp -R ~/.quint/. <task worktree>/target/quint_home/`. Never write `~/.quint`.
- Never use `sed` on a Rust, TypeScript or Svelte file; use the Edit tool.
- `test_paths`: org-node reads `org-node/tests`, on-chain-client
  `on-chain-client/tests`, the app `app/src-tauri/tests` and `app/tests`. Every
  `verifies:` annotation of this plan lives there, never in `src/`.
- Find an item's text with `GR_CONFIG=<unit>/.guardrails/config.yaml
  .guardrails/scripts/find-items.sh show <ID>` and its references with
  `… find-items.sh refs <ID>`, rather than reading a whole ledger.
- Commit messages: imperative subject naming the task (`T<N>: …`), body
  listing the IDs verified. No Co-Authored-By line (owner preference).
  Commit unsigned in task worktrees (`git -c commit.gpgsign=false commit`);
  the dispatcher merges (`task-worktree.sh merge <tag>`).
- A removed name must not survive in `org-node/src` even in a comment: the
  absence tests (`tests/absences.rs`, from T7) scan the source text for it.
  Reword any comment that names a removed function.
- Each org-node task's last step runs, in its worktree:
  `CARGO_HOME=/tmp/cargo_home_fuzz cargo check -p org-node --all-targets --features app,test-support`
  so that targets outside the gate (the chopsticks targets and `preflight`)
  never stop compiling unnoticed.
- Keys in tests, as master left them: a Member-as-a-group key is
  `MemberSeed::from(bytes).x25519_keypair().member_key().unwrap()`, a device
  key `DeviceSeed::from(bytes).signing_keypair().device_key().unwrap()`, and
  `org_node::test_fixtures::{member_key, device_key, org_public_key}` give
  fixed valid ones. There is no `OrgPublicKey::from(&member_key)`.

## Three lockfiles, one rule

`Cargo.lock` (repository root: org-node, org-members, person, spikes),
`on-chain-client/Cargo.lock` (on-chain-client is its own workspace) and
`app/src-tauri/Cargo.lock` (the app is its own workspace, `[workspace]` at the
top of `app/src-tauri/Cargo.toml`) each govern one build. Measured
2026-10-05: subxt and subxt-signer are **0.50.3** in the root and app locks
and **0.50.1** in on-chain-client's; blake2 is 0.10.6 and parity-scale-codec
3.7.5 in all three.

- **The writer's gate evidence** (T1, T2) is built against
  `on-chain-client/Cargo.lock`: subxt 0.50.1, subxt-signer 0.50.1, blake2
  0.10.6. Both new normal dependencies are already in that lock as
  dev-dependencies, so enabling `write` adds no package; the only lock edit is
  the `dependencies = [...]` list of the `on-chain-client` package entry.
- **The writer as shipped** is built against `app/src-tauri/Cargo.lock` (T14):
  subxt and subxt-signer 0.50.3 — the versions the app already ships for its
  chain client today.
- **No version moves.** No task runs `cargo update`. Every task that touches
  a lockfile checks `git diff <lockfile>` and accepts only added/removed
  dependency edges and removed packages: no `version = ` line of a package
  that remains may change. A diff that changes one is a stop-and-ask.
- The disagreement between 0.50.1 (evidence) and 0.50.3 (shipped) is the one
  `on-chain-client/docs/architecture/soup.md` already records for subxt; T17
  corrects that file's sentence that says the root lock governs the shipped
  writer (it is the app's own lock). Aligning the two is an Open question for
  the owner, not something this plan does silently.
- Lockfile owners: root `Cargo.lock` — T7 and T13 only (both serial in the
  org-node chain); `on-chain-client/Cargo.lock` — T1 only (done);
  `app/src-tauri/Cargo.lock` — T14 and T15 (serial).

## Order and parallelism

```
T1 → T2                                  on-chain-client (done)
T3                                       org-node test helpers (done)
T4                                       withdrawn: master did it
T5 → T6 → T7 → T8 → T9 → T10 → T11 → T12   org-node (share service.rs)
T13   serial after T12                   org-node chain write removal
T14 → T15 → T16   serial after T13       app
T17   serial after T16                   docs, deslop, full gate
```

Every remaining task is serial: T5–T13 share `org-node/src/service.rs`,
`org-node/tests/support/mod.rs`, `org-node/Cargo.toml` and
`org-node/.guardrails/config.yaml`; T14–T16 share the app's crate and
`app/.guardrails/config.yaml`; T13 must precede T14 (the app moves to the
read-only seam and the on-chain-client writer once org-node's writer is
gone); T17 reads everything. T2 (done) ported `chain_genesis_e2e`, which T13
deletes.

**The app does not compile from T6 until T14.** T6 adds
`OrgNodeError::AdmissionNotExpected` and `AdmissionNotOurs`, which
`app/src-tauri/src/events.rs` matches exhaustively by design, and changes
`expect_admission`'s and the Wire message's shape; T7–T13 change or remove
every `OrgService` operation the app calls. T5–T13 run org-node's verify line
only; T14 brings the app back and runs its line. `check-trace` is required
green (only MISSING-TEST-free for this change's items, see T17) only at T17.

**Why the invitation exchange leaves before the store and the service
paths.** The store's `pending_invites` field has exactly one writer
(`import_invite`), so it cannot leave the store without the Invite leaving
org-node; T6 (expected admissions, the replacement) and T7 (the invitation
exchange out) therefore precede T8 (the rest of the store). The chain write
still leaves last (T13).

## Decisions this plan takes that the inputs did not settle

1. **`check_chain_free`.** `verify::check_chain_free` is a thin public
   wrapper over a private `chain_free` that returns the decoded `Delta`, so
   `verify_envelope_against_chain` decodes once. `VerifyContext` is as master
   left it (three fields, no key).
2. *(Withdrawn 2026-10-06: the Envelope's field names. Master settled them.)*
3. **Store and wire formats change in three tasks, each re-pinning first:**
   T6 appends `expected_admissions: Vec<ExpectedAdmission>` to `StoreData`
   and `invite_id: Option<InviteId>` to `WireMessage`; T7 removes
   `pending_invites`; T8 removes `OrgRecord.admin_member_key` and adds
   `provisional_updates` (between `orgs` and `expected_admissions`, the order
   LLR-95753m names). Each new golden value is derived from master's pinned
   value by the format rule (bytes cut or appended at stated offsets), never
   captured from the code; the derivations are stated in each task.
4. **`ProvisionalChange` variant order** is `Genesis { members,
   org_private_key }` (index 0) then `ChangeSet { change_set }` (index 1).
5. **Identity of a provisional update** (LLR-95753m): `(org_id,
   resulting_root)` for an Organisation's update, `(None, persona_id,
   resulting_root)` for a genesis update. The bound's group (LLR-jq7qh7) is
   every update with the same `org_id`, or for genesis every genesis update of
   the same Persona.
6. **"The first Persona bound to the Organisation"** (LLR-2xzys9, and the
   Persona that builds an admission or revocation) is the first
   `PersonaRecord` in store order whose `org_id` is `Some(org_id)`, whatever
   its status. With none, the operation refuses with
   `OrgNodeError::Chain("no persona bound to organisation <hex>")` — the
   catch-all, as every other non-verification refusal in `service.rs` is.
7. **`create_organisation`, `admit_member` and `revoke_member` become
   synchronous** (`pub fn`): none of them performs I/O any more.
8. **LLR-b27jr6 applies to an Organisation the node already held.** On a first
   admission no Persona is bound yet, so "holds the device key of no Persona
   bound" would be vacuously true. The first-admission case is REQ-kt877x's
   instead (owner, 2026-10-06): a verified first admission that lists none of
   the node's Personas is refused with `AdmissionNotOurs` (LLR-3f5h7b, T6),
   so one that commits always binds a Persona (LLR-e5c9ud).
9. **`FinalitySink` becomes a sink that never resolves** in on-chain-client
   (T1/T2, done); org-node's polling `FinalitySink` and its two
   `finality_polling` tests (no `verifies:`, outside the gate) are deleted with
   the polling loop in T13. The app bounds each writer call instead (T14,
   LLR-be3zv9).
10. **`build_dispatch_tx` returns a plain `DispatchTx`** (T1, done).
11. **`BlockSink::settle` returns `Result<(), WriteError>`** (T1, done).
12. **The app's `admit_member` command takes `org_id`, `reply_blob`,
    `peer_addr_blob` and an optional `org_secret_hex`** (LLR-8hdu9x: the
    secret is what the caller passes until the key-pair change). The Invite
    reply carries no address (REQ-tcutr6), so a Loopback admission needs the
    address pasted, as a Loopback revocation does; the peer address is parsed
    as `revoke_member` parses it (LLR-ctrfz4 as amended).
13. **Outstanding invite identifiers are persisted** by the app in
    `<data_dir>/outstanding_invites.json` (hex strings), so a reply arriving
    after a restart is still accepted (REQ-65xqp8, LLR-f35pda). They are not
    secret.
14. **Classification in the app** (REQ-kn5rtx, LLR-7bk6qh as amended): the
    verdicts are the variants `verify_envelope_against_chain` produces —
    `OrgIdMismatch`, `StaleSeq`, `SeqNotEpoch`, `MalformedDelta`,
    `DeltaBaseMismatch`, `RootMismatch`, `StaleEpoch`; `AdmissionNotExpected`,
    `AdmissionNotOurs`, `ProvisionalLimit` and `NoProvisionalUpdate` join
    `Chain`, `OrgNotOnChain`, `Trie`, `InvalidOrgPublicKey` and
    `InvalidField` as receiver errors.
15. **org-node re-exports `Handle`, `Name`, `Surname`, `PersonPublicKey`,
    `DevicePublicKey` and `InviteId`** (T6/T7), and from the service
    `Joiner`, `CommitOutcome` and `OutgoingUpdate`, because they are in
    org-node's interface and the app may depend on org-node alone for them
    (the rule `lib.rs` already states for `MemberId` and `RootHash`).
16. **Epochs and marks** (added 2026-10-06). The contract sets epoch 1 at
    genesis and E + 1 on each update, and an Envelope's Sequence number is the
    epoch it is verified against (REQ-txvtm9). So a provisional update built
    on a record at epoch E carries Sequence number E + 1
    (`SequenceNumber::new(rec.epoch.get() + 1)`), a genesis update carries 1,
    and `commit_genesis` refuses with `SeqNotEpoch { seq: 1, epoch }` any
    other epoch and writes the record's mark as 1, so every record's mark
    equals its epoch (master's `create_organisation` wrote mark 0 at
    epoch 1).
17. **Genesis selection matches the key** (added 2026-10-06). `commit_genesis`
    selects the genesis update whose `resulting_root` *and* `org_pub_key` are
    the chain's, so the private key the record keeps always has the chain's
    key as its public half (LLR-wzqqg9, LLR-qjz3q4).
18. **Test invite identifiers** (added 2026-10-06). A story's invite
    identifier is the joining node's device key bytes:
    `support::invite_for(&device_key) = InviteId::new(*device_key.as_bytes())`.
    `prepare_to_join` declares it for B's first Persona and `admit` passes it
    for the joiner it admits, so no story helper gains an argument and the
    forty-odd call sites stay as T3 left them. A test of a mismatch passes
    another identifier explicitly.
19. **The invite identifier on the wire** (added 2026-10-06) is
    `WireMessage`'s last field, `Option<InviteId>`; `send_update` (T10) and,
    until then, the old `admit_member` (T6) take it as an argument. A
    revocation or an ordinary update carries `None`.

---

### T1 — on-chain-client: the `write` feature and the pure chain writer

**Files touched:** `on-chain-client/Cargo.toml`, `on-chain-client/Cargo.lock`,
`on-chain-client/src/lib.rs`, `on-chain-client/src/write/mod.rs` (new),
`on-chain-client/src/write/calldata.rs` (new),
`on-chain-client/src/write/multisig.rs` (new),
`on-chain-client/src/write/proxy.rs` (new),
`on-chain-client/tests/write_manifest.rs` (new),
`on-chain-client/tests/write_pure.rs` (new),
`on-chain-client/.guardrails/config.yaml`
**Parallel:** yes (with T3 and with T4–T12)
**IDs verified:** LLR-rxs5ec, LLR-yvq33e, LLR-5varjf, LLR-d4ftc8, LLR-m7wmmx,
LLR-ywhd23, LLR-gjx3jn, LLR-kqxw9t, LLR-qs8jfw (with REQ-6jefu2, REQ-aat4yt
through them).
**Size:** ~380 lines (code moved from `org-node/src/chain_write/` with types
changed; tests moved from `org-node/tests/chain_write_pure.rs` and
`calldata_typed.rs` and re-annotated).

The code is copied from org-node, not moved: org-node keeps its copy until T13
deletes it. Types change to this unit's own (`OnChainRootHash`, `OrgPubKey`,
`Epoch`, `OrgAdmin`, the new `AccountId`).

**Step 1 — failing tests.** Create `on-chain-client/tests/write_manifest.rs`:

```rust
#![cfg(all(feature = "test-support", feature = "write"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! LLR-rxs5ec: the writer compiles only with `write`, which adds exactly
//! `subxt-signer` and `blake2`, and this crate names no consumer.

const MANIFEST: &str = include_str!("../Cargo.toml");
const LIB: &str = include_str!("../src/lib.rs");

/// The text of one `[table]` of the manifest, up to the next table header.
fn table(header: &str) -> &'static str {
    let start = MANIFEST.find(header).unwrap_or_else(|| panic!("no {header}")) + header.len();
    let rest = &MANIFEST[start..];
    &rest[..rest.find("\n[").unwrap_or(rest.len())]
}

/// Manifest lines that are not comments.
fn code_lines() -> impl Iterator<Item = &'static str> {
    MANIFEST.lines().filter(|l| !l.trim_start().starts_with('#'))
}

// Normal: the feature exists, names exactly the two crates, and gates the module.
// verifies: LLR-rxs5ec, REQ-6jefu2, REQ-aat4yt
#[test]
fn the_write_feature_adds_exactly_the_signer_and_the_hasher() {
    let features = table("[features]");
    let write = features
        .lines()
        .find(|l| l.trim_start().starts_with("write ="))
        .expect("a `write` feature");
    assert_eq!(write.trim(), r#"write = ["client", "dep:subxt-signer", "dep:blake2"]"#);
    let deps = table("[dependencies]");
    for dep in ["subxt-signer", "blake2"] {
        let line = deps
            .lines()
            .find(|l| l.trim_start().starts_with(&format!("{dep} =")))
            .unwrap_or_else(|| panic!("{dep} must be a normal dependency"));
        assert!(line.contains("optional = true"), "{dep} must be optional: {line}");
    }
    assert!(
        LIB.contains("#[cfg(feature = \"write\")]\npub mod write;"),
        "the write module must be compiled only under `write`"
    );
    assert!(!table("[features]").lines().any(|l| l.trim_start().starts_with("default") && l.contains("write")));
}

// Abnormal: no other feature pulls the two crates in, and no line of the
// manifest names a consumer of this crate.
// verifies: LLR-rxs5ec
#[test]
fn no_other_feature_enables_the_writer_and_no_consumer_is_named() {
    for line in table("[features]").lines().filter(|l| !l.trim_start().starts_with("write =")) {
        assert!(
            !line.contains("subxt-signer") && !line.contains("blake2") && !line.contains("\"write\""),
            "only `write` may enable the writer's crates: {line}"
        );
    }
    for line in code_lines() {
        assert!(!line.contains("org-node") && !line.contains("org-members"), "consumer named: {line}");
    }
}
```

Create `on-chain-client/tests/write_pure.rs`:

```rust
#![cfg(all(feature = "test-support", feature = "write"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The pure half of the chain writer (SDD-yg7n55): calldata, the runtime calls
//! it wraps, the multisig account and the dispatch shape. Moved from
//! org-node's `chain_write_pure` and `calldata_typed` targets with the code,
//! and re-annotated to this unit's requirements.

use on_chain_client::write::calldata::{
    build_update_calldata, revive_update_runtime_call, STORAGE_DEPOSIT_LIMIT, UPDATE_SELECTOR,
    WEIGHT_PROOF_SIZE, WEIGHT_REF_TIME,
};
use on_chain_client::write::multisig::{build_dispatch_tx, multi_account_id};
use on_chain_client::write::proxy::{map_account_call, proxied};
use on_chain_client::write::{AccountId, WriteError};
use on_chain_client::{Epoch, OnChainRootHash, OrgPubKey};
use subxt::dynamic::Value;
use subxt::ext::scale_value::Composite;

const GOLDEN_GENESIS_CALLDATA: &str = "f1bc537b333333333333333333333333333333333333333333333333333333333333333322222222222222222222222222222222222222222222222222222222222222220000000000000000000000000000000000000000000000000000000000000000";
const GOLDEN_UPDATE_CALLDATA: &str = "f1bc537b666666666666666666666666666666666666666666666666666666666666666622222222222222222222222222222222222222222222222222222222222222220000000000000000000000000000000000000000000000000000000000000007";

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
fn acct(b: u8) -> AccountId {
    AccountId([b; 32])
}
fn remark() -> Value {
    Value::variant(
        "System",
        Composite::unnamed(vec![Value::variant(
            "remark",
            Composite::named(vec![("remark".to_string(), Value::from_bytes([1u8; 4]))]),
        )]),
    )
}

// Normal: the pinned calldata of a genesis and an update, carried over from
// org-node's golden values (encoding_golden, calldata_typed).
// verifies: LLR-yvq33e, REQ-aat4yt, REQ-6jefu2
#[test]
fn update_calldata_is_the_pinned_hundred_bytes() {
    let genesis = build_update_calldata(OnChainRootHash([0x33; 32]), OrgPubKey([0x22; 32]), Epoch(0));
    let update = build_update_calldata(OnChainRootHash([0x66; 32]), OrgPubKey([0x22; 32]), Epoch(7));
    assert_eq!(hex(&genesis), GOLDEN_GENESIS_CALLDATA);
    assert_eq!(hex(&update), GOLDEN_UPDATE_CALLDATA);
    assert_eq!(update.len(), 100);
    assert_eq!(&update[0..4], &UPDATE_SELECTOR);
    assert_eq!(&update[4..36], &[0x66; 32]);
    assert_eq!(&update[36..68], &[0x22; 32]);
}

// Abnormal (boundary): an all-zero root and key at the largest epoch keep the
// layout — the fields do not shift or bleed into each other.
// verifies: LLR-yvq33e
#[test]
fn calldata_at_the_boundary_values_keeps_its_layout() {
    let data = build_update_calldata(OnChainRootHash([0; 32]), OrgPubKey([0xff; 32]), Epoch(u64::MAX));
    assert_eq!(data.len(), 100);
    assert!(data[4..36].iter().all(|b| *b == 0));
    assert!(data[36..68].iter().all(|b| *b == 0xff));
}

// Normal: the epoch is the low sixteen bytes of a big-endian word.
// verifies: LLR-5varjf, REQ-aat4yt
#[test]
fn the_expected_epoch_occupies_the_low_sixteen_bytes_big_endian() {
    let data = build_update_calldata(OnChainRootHash([0x11; 32]), OrgPubKey([0x22; 32]), Epoch(7));
    assert_eq!(data[99], 7);
    assert!(data[68..99].iter().all(|b| *b == 0));
}

// Abnormal (boundary): the largest epoch leaves the high sixteen bytes zero.
// verifies: LLR-5varjf
#[test]
fn the_largest_epoch_leaves_the_high_sixteen_bytes_zero() {
    let data = build_update_calldata(OnChainRootHash([0; 32]), OrgPubKey([0; 32]), Epoch(u64::MAX));
    assert!(data[68..84].iter().all(|b| *b == 0), "high sixteen bytes zero");
    assert_eq!(&data[84..100], &u128::from(u64::MAX).to_be_bytes());
}

fn keccak4(signature: &str) -> [u8; 4] {
    use tiny_keccak::{Hasher, Keccak};
    let mut k = Keccak::v256();
    k.update(signature.as_bytes());
    let mut out = [0u8; 32];
    k.finalize(&mut out);
    [out[0], out[1], out[2], out[3]]
}

// Normal: the selector is recomputed independently from the signature.
// verifies: LLR-d4ftc8, REQ-aat4yt
#[test]
fn the_selector_is_the_keccak_of_the_update_signature() {
    assert_eq!(UPDATE_SELECTOR, keccak4("update(bytes32,bytes32,uint256)"));
}

// Abnormal: a drifted signature yields a different constant, so drift shows
// as a changed selector here.
// verifies: LLR-d4ftc8
#[test]
fn a_drifted_signature_is_a_different_selector() {
    assert_ne!(UPDATE_SELECTOR, keccak4("update(bytes32,bytes32,uint128)"));
    assert_ne!(UPDATE_SELECTOR, keccak4("update(bytes32,bytes32)"));
}

fn expected_call(contract: [u8; 20], data: Vec<u8>) -> Value {
    Value::variant(
        "Revive",
        Composite::unnamed(vec![Value::variant(
            "call",
            Composite::named(vec![
                ("dest".to_string(), Value::unnamed_composite(contract.iter().map(|b| Value::u128(u128::from(*b))))),
                ("value".to_string(), Value::u128(0)),
                (
                    "weight_limit".to_string(),
                    Value::named_composite([
                        ("ref_time", Value::u128(1_000_000_000_000)),
                        ("proof_size", Value::u128(4_000_000)),
                    ]),
                ),
                ("storage_deposit_limit".to_string(), Value::u128(10_000_000_000_000)),
                ("data".to_string(), Value::from_bytes(data)),
            ]),
        )]),
    )
}

// Normal: every field name and constant the runtime matches by name.
// verifies: LLR-m7wmmx, REQ-aat4yt
#[test]
fn the_revive_call_names_every_field_and_constant() {
    let contract: [u8; 20] = core::array::from_fn(|i| i as u8 + 1);
    let (root, key, epoch) = (OnChainRootHash([0x11; 32]), OrgPubKey([0x22; 32]), Epoch(7));
    assert_eq!(
        revive_update_runtime_call(contract, root, key, epoch),
        expected_call(contract, build_update_calldata(root, key, epoch))
    );
    assert_eq!((WEIGHT_REF_TIME, WEIGHT_PROOF_SIZE, STORAGE_DEPOSIT_LIMIT), (1_000_000_000_000, 4_000_000, 10_000_000_000_000));
}

// Abnormal: two calls differing only in the epoch differ only in `data`.
// verifies: LLR-m7wmmx
#[test]
fn a_different_epoch_changes_only_the_data_field() {
    let contract = [0xab; 20];
    let (root, key) = (OnChainRootHash([0x11; 32]), OrgPubKey([0x22; 32]));
    let at_8 = revive_update_runtime_call(contract, root, key, Epoch(8));
    assert_ne!(revive_update_runtime_call(contract, root, key, Epoch(7)), at_8);
    assert_eq!(at_8, expected_call(contract, build_update_calldata(root, key, Epoch(8))));
}

// Normal: `proxied` and `map_account_call` build the named calls.
// verifies: LLR-ywhd23, REQ-aat4yt, REQ-6jefu2
#[test]
fn proxied_and_map_account_build_the_named_calls() {
    let expected = Value::variant(
        "Proxy",
        Composite::unnamed(vec![Value::variant(
            "proxy",
            Composite::named(vec![
                ("real".to_string(), Value::variant("Id", Composite::unnamed(vec![Value::from_bytes([7u8; 32])]))),
                ("force_proxy_type".to_string(), Value::variant("None", Composite::unnamed(vec![]))),
                ("call".to_string(), remark()),
            ]),
        )]),
    );
    assert_eq!(proxied(acct(7), remark()), expected);
    assert_eq!(
        map_account_call(),
        Value::variant("Revive", Composite::unnamed(vec![Value::variant("map_account", Composite::unnamed(vec![]))]))
    );
}

// Abnormal: wrapping is total — a proxied call wrapped again nests unchanged.
// verifies: LLR-ywhd23
#[test]
fn proxied_wraps_any_call_unchanged_even_another_proxied_call() {
    let inner = proxied(acct(1), remark());
    let outer = proxied(acct(2), inner.clone());
    assert_eq!(outer, proxied(acct(2), inner));
    assert_ne!(outer, proxied(acct(1), remark()));
}

// Normal: the derived account ignores the order the signatories came in.
// verifies: LLR-gjx3jn, REQ-6jefu2
#[test]
fn the_derived_account_does_not_depend_on_signatory_order() {
    let (a, b, c) = (acct(1), acct(2), acct(3));
    assert_eq!(multi_account_id(&[a, b], 1), multi_account_id(&[b, a], 1));
    assert_eq!(multi_account_id(&[a, b, c], 1), multi_account_id(&[c, a, b], 1));
    assert_ne!(multi_account_id(&[acct(1)], 1), multi_account_id(&[acct(2)], 1));
}

// Abnormal (boundary): an empty set and a set with a repeated signatory
// still derive deterministically, independent of order.
// verifies: LLR-gjx3jn
#[test]
fn empty_and_repeated_signatory_sets_derive_deterministically() {
    assert_eq!(multi_account_id(&[], 1), multi_account_id(&[], 1));
    let (a, b) = (acct(1), acct(2));
    assert_eq!(multi_account_id(&[a, a, b], 1), multi_account_id(&[b, a, a], 1));
    assert_ne!(multi_account_id(&[a, a, b], 1), multi_account_id(&[a, b], 1));
}

// Normal: the threshold is part of the derivation.
// verifies: LLR-kqxw9t, REQ-6jefu2
#[test]
fn the_derived_account_depends_on_the_threshold() {
    assert_ne!(multi_account_id(&[acct(1), acct(2)], 1), multi_account_id(&[acct(1), acct(2)], 2));
}

// Abnormal (boundary): thresholds zero and u16::MAX are distinct from 1 too.
// verifies: LLR-kqxw9t
#[test]
fn boundary_thresholds_derive_distinct_accounts() {
    let set = [acct(1), acct(2)];
    let ids = [0u16, 1, u16::MAX].map(|t| multi_account_id(&set, t));
    assert_ne!(ids[0], ids[1]);
    assert_ne!(ids[1], ids[2]);
    assert_ne!(ids[0], ids[2]);
}

// Normal: no co-signatories dispatches the call itself; with them, it is
// wrapped in `as_multi_threshold_1` with the co-signatories sorted by bytes.
// verifies: LLR-qs8jfw, REQ-6jefu2, REQ-aat4yt
#[test]
fn dispatch_is_direct_without_co_signatories_and_sorted_multisig_with_them() {
    let direct = build_dispatch_tx(&[], remark()).unwrap();
    assert_eq!((direct.pallet.as_str(), direct.call.as_str()), ("System", "remark"));
    let multi = build_dispatch_tx(&[acct(9), acct(3)], remark()).unwrap();
    assert_eq!((multi.pallet.as_str(), multi.call.as_str()), ("Multisig", "as_multi_threshold_1"));
    assert_eq!(
        multi.fields,
        Composite::unnamed(vec![
            Value::unnamed_composite(vec![Value::from_bytes([3u8; 32]), Value::from_bytes([9u8; 32])]),
            remark(),
        ])
    );
    assert_eq!(build_dispatch_tx(&[acct(3), acct(9)], remark()).unwrap(), multi);
}

// Abnormal: a direct dispatch of a value that is not a RuntimeCall is refused
// with `MalformedCall` at each of `runtime_call_to_tx`'s three arms (owed by
// org-node's robustness table, SDD-rq6nv4).
// verifies: LLR-qs8jfw
#[test]
fn a_value_that_is_not_a_runtime_call_is_refused_on_each_arm() {
    assert_eq!(
        build_dispatch_tx(&[], Value::u128(1)).unwrap_err(),
        WriteError::MalformedCall("call is not a RuntimeCall variant")
    );
    assert_eq!(
        build_dispatch_tx(&[], Value::variant("System", Composite::unnamed(vec![]))).unwrap_err(),
        WriteError::MalformedCall("RuntimeCall has no inner call")
    );
    assert_eq!(
        build_dispatch_tx(&[], Value::variant("System", Composite::unnamed(vec![Value::u128(1)]))).unwrap_err(),
        WriteError::MalformedCall("inner call is not a variant")
    );
}
```

**Step 2 — run, expect red.**
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test --manifest-path on-chain-client/Cargo.toml --features test-support,write --test write_manifest --test write_pure`
Expected: `error: the package 'on-chain-client' does not contain this feature: write`
(the right reason: the feature does not exist yet). Record it.

**Step 3 — the manifest.** In `on-chain-client/Cargo.toml`, under
`[features]` after the `smoldot` line, add:

```toml
# The chain writer (SDD-yg7n55): genesis ceremony and update submission,
# moved from org-node (change worktree-org-node-chain-authority). Never in
# `default`: a build without it compiles no code that can sign or submit
# (LLR-rxs5ec).
write = ["client", "dep:subxt-signer", "dep:blake2"]
```

Under `[dependencies]`, after the `subxt` entry, add:

```toml
# Behind `write` only (LLR-rxs5ec). Versions as the dev-dependencies below,
# which this lockfile already resolves (subxt-signer 0.50.1, blake2 0.10.6).
subxt-signer = { version = "0.50", default-features = false, features = ["sr25519", "subxt"], optional = true }
blake2 = { version = "0.10", optional = true }
```

Append, after the `type_widths` `[[test]]` entry:

```toml
# The chain writer's gated targets (SDD-yg7n55), run by their own
# verify_commands line with `--features test-support,write`, so the reader's
# targets above keep building without `write`.
[[test]]
name = "write_manifest"
required-features = ["test-support", "write"]

[[test]]
name = "write_pure"
required-features = ["test-support", "write"]
```

In `on-chain-client/src/lib.rs`, after `pub mod verify;` add exactly:

```rust
#[cfg(feature = "write")]
pub mod write;
```

**Step 4 — the module.** `on-chain-client/src/write/mod.rs`:

```rust
//! The chain writer (SDD-yg7n55): stands up an Organisation's slot and submits
//! each later update through its pure proxy. Compiled only with the `write`
//! feature (LLR-rxs5ec). Holds no key: the signatory key is an argument of
//! every call.

use core::fmt;

pub mod calldata;
pub mod multisig;
pub mod proxy;

/// A 32-byte chain account: a signatory, a co-signatory or a pure proxy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountId(pub [u8; 32]);

/// The genesis step that failed (LLR-z9vugt).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenesisStep {
    CreatePure,
    Fund,
    MapAccount,
    RecordGenesis,
}

/// Every way a write can fail; no outcome but an executed call is success.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteError {
    /// A genesis step did not execute.
    Step { step: GenesisStep, reason: String },
    /// A multisig approval was recorded but the update did not execute.
    PendingApproval,
    Subxt(String),
    EventNotFound(&'static str),
    MalformedEvent(&'static str),
    MalformedCall(&'static str),
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriteError::Step { step, reason } => write!(f, "genesis step {step:?} failed: {reason}"),
            WriteError::PendingApproval => f.write_str("multisig approval recorded; the update did not execute"),
            WriteError::Subxt(m) => write!(f, "subxt error: {m}"),
            WriteError::EventNotFound(m) => write!(f, "expected on-chain event not found: {m}"),
            WriteError::MalformedEvent(m) => write!(f, "malformed event field: {m}"),
            WriteError::MalformedCall(m) => write!(f, "malformed call: {m}"),
        }
    }
}

impl std::error::Error for WriteError {}

/// What one dispatch under the controller did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use = "a dispatch may be a pending multisig approval, not an executed call"]
pub enum DispatchOutcome {
    Executed,
    ApprovalRecorded,
}
```

`on-chain-client/src/write/calldata.rs` — copy
`org-node/src/chain_write/calldata.rs` and change only: the module doc's first
line to `//! EVM calldata and the \`Revive.call\` runtime call for
OrgRegistry.update (LLR-yvq33e, LLR-5varjf, LLR-d4ftc8, LLR-m7wmmx).`; drop
`#![cfg(feature = "chain")]`; make the three constants `pub`; replace
`use org_members::RootHash;` and `use crate::types::{Epoch, OrgPublicKey};`
with `use crate::types::{Epoch, OnChainRootHash, OrgPubKey};`; delete
`update_calldata`; and use these two signatures and bodies:

```rust
pub fn build_update_calldata(root: OnChainRootHash, key: OrgPubKey, expected_epoch: Epoch) -> Vec<u8> {
    let mut data = Vec::with_capacity(100);
    data.extend_from_slice(&UPDATE_SELECTOR);
    data.extend_from_slice(&root.0);
    data.extend_from_slice(&key.0);
    let mut epoch_be = [0u8; 32];
    epoch_be[16..32].copy_from_slice(&u128::from(expected_epoch.0).to_be_bytes());
    data.extend_from_slice(&epoch_be);
    data
}

pub fn revive_update_runtime_call(
    contract_h160: [u8; 20],
    root: OnChainRootHash,
    key: OrgPubKey,
    expected_epoch: Epoch,
) -> Value {
    let calldata = build_update_calldata(root, key, expected_epoch);
    // … the body of org-node's revive_update_runtime_call from
    // `let h160_bytes` to the end, unchanged.
}
```

(The `// …` comment is an instruction to paste that exact block, 33 lines from
`let h160_bytes: Vec<Value> = contract_h160` to the closing `)` of
`Value::variant("Revive", …)`, with its `weight_limit` comment.)

`on-chain-client/src/write/multisig.rs`:

```rust
//! The multisig that controls an Organisation's slot: its pseudo-account
//! (LLR-gjx3jn, LLR-kqxw9t) and the shape of a dispatch under it (LLR-qs8jfw).

use blake2::Blake2bVar;
use blake2::digest::{Update as _, VariableOutput};
use parity_scale_codec::Encode;
use subxt::dynamic::{self, Value};
use subxt::ext::scale_value::{Composite, ValueDef};
use subxt::transactions::StaticPayload;

use super::{AccountId, WriteError};

/// pallet-multisig's pseudo-account: `blake2_256(scale((b"modlpy/utilisuba",
/// sorted signatories, threshold)))`.
pub fn multi_account_id(signatories: &[AccountId], threshold: u16) -> AccountId {
    let mut sorted: Vec<[u8; 32]> = signatories.iter().map(|a| a.0).collect();
    sorted.sort();
    AccountId(blake2_256(&(b"modlpy/utilisuba", sorted, threshold).encode()))
}

fn blake2_256(data: &[u8]) -> [u8; 32] {
    // 32 is a valid blake2b output size, so construction cannot fail; the
    // zero fallback is unreachable and keeps the function total.
    let mut hasher = match Blake2bVar::new(32) {
        Ok(h) => h,
        Err(_) => return [0u8; 32],
    };
    hasher.update(data);
    let mut out = [0u8; 32];
    let _ = hasher.finalize_variable(&mut out);
    out
}

/// An extrinsic as plain values: which pallet, which call, which fields.
#[derive(Clone, Debug, PartialEq)]
pub struct DispatchTx {
    pub pallet: String,
    pub call: String,
    pub fields: Composite<()>,
}

impl DispatchTx {
    /// The subxt payload this extrinsic is submitted as.
    pub fn into_payload(self) -> StaticPayload<Composite<()>> {
        dynamic::tx(self.pallet, self.call, self.fields)
    }
}

/// A RuntimeCall value — `Variant(pallet, Unnamed([Variant(call, fields)]))`
/// — as a top-level extrinsic.
fn runtime_call_to_tx(call: Value) -> Result<DispatchTx, WriteError> {
    let ValueDef::Variant(pallet_var) = call.value else {
        return Err(WriteError::MalformedCall("call is not a RuntimeCall variant"));
    };
    let inner = pallet_var
        .values
        .into_values()
        .next()
        .ok_or(WriteError::MalformedCall("RuntimeCall has no inner call"))?;
    let ValueDef::Variant(call_var) = inner.value else {
        return Err(WriteError::MalformedCall("inner call is not a variant"));
    };
    Ok(DispatchTx { pallet: pallet_var.name, call: call_var.name, fields: call_var.values })
}

/// `call` under the controller: itself when there are no co-signatories,
/// otherwise `Multisig.as_multi_threshold_1(sorted co-signatories, call)`.
/// Every dispatch the writer makes goes through this function (LLR-qs8jfw).
pub fn build_dispatch_tx(co_signatories: &[AccountId], call: Value) -> Result<DispatchTx, WriteError> {
    if co_signatories.is_empty() {
        return runtime_call_to_tx(call);
    }
    let mut sorted = co_signatories.to_vec();
    sorted.sort();
    let others: Vec<Value> = sorted.iter().map(|a| Value::from_bytes(a.0.as_slice())).collect();
    Ok(DispatchTx {
        pallet: "Multisig".into(),
        call: "as_multi_threshold_1".into(),
        fields: Composite::unnamed(vec![Value::unnamed_composite(others), call]),
    })
}
```

`on-chain-client/src/write/proxy.rs`:

```rust
//! The calls a pure proxy is created, mapped and acted through (LLR-ywhd23).

use subxt::dynamic::Value;
use subxt::ext::scale_value::Composite;

use super::AccountId;

/// `Proxy.create_pure { proxy_type: Any, delay: 0, index: 0 }`.
pub fn create_pure_call() -> Value {
    Value::variant(
        "Proxy",
        Composite::unnamed(vec![Value::variant(
            "create_pure",
            Composite::named(vec![
                ("proxy_type".to_string(), Value::variant("Any", Composite::unnamed(vec![]))),
                ("delay".to_string(), Value::u128(0)),
                ("index".to_string(), Value::u128(0)),
            ]),
        )]),
    )
}

/// `Proxy.proxy { real: Id(proxy), force_proxy_type: None, call }`.
pub fn proxied(proxy: AccountId, call: Value) -> Value {
    Value::variant(
        "Proxy",
        Composite::unnamed(vec![Value::variant(
            "proxy",
            Composite::named(vec![
                ("real".to_string(), Value::variant("Id", Composite::unnamed(vec![Value::from_bytes(proxy.0.as_slice())]))),
                ("force_proxy_type".to_string(), Value::variant("None", Composite::unnamed(vec![]))),
                ("call".to_string(), call),
            ]),
        )]),
    )
}

/// `Revive.map_account {}`, dispatched once as the proxy before it can call
/// the contract.
pub fn map_account_call() -> Value {
    Value::variant("Revive", Composite::unnamed(vec![Value::variant("map_account", Composite::unnamed(vec![]))]))
}
```

**Step 5 — run, expect green.** Same command as Step 2. Expected:
`test result: ok. 2 passed` (write_manifest) and `test result: ok. 16 passed`
(write_pure). Then the reader's line, unchanged, to show `write` stays out of
it: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --manifest-path on-chain-client/Cargo.toml --features test-support --lib --test fuzz_decode_org_state --test fuzz_parse_revive_event --test fuzz_event_round_trip --test contract_address_filter --test log_ownership --test decode_revive_event --test decode_org_state --test runtime_version_dispatch --test h160_mapping --test storage_slot_layout --test best_lane_reorg_rule --test type_widths`
— all `ok`.

**Step 6 — lock and clippy.** `git diff on-chain-client/Cargo.lock` must show
only the `on-chain-client` package's `dependencies` list gaining `"blake2"`
and `"subxt-signer"`; no `version =` line changes.
`CARGO_HOME=/tmp/cargo_home_fuzz cargo tree --manifest-path on-chain-client/Cargo.toml --features write -e normal -i subxt-signer`
prints `subxt-signer v0.50.1`.
`CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy --manifest-path on-chain-client/Cargo.toml --features test-support,write --all-targets -- -D warnings`
is clean. Also confirm the reader build without `write` compiles no writer:
`cargo check --manifest-path on-chain-client/Cargo.toml` succeeds.

**Step 7 — config.** In `on-chain-client/.guardrails/config.yaml`, append a
second `verify_commands` item directly under the existing one (no comment
between list items — the config validator refuses that):

```yaml
  - cargo test --manifest-path on-chain-client/Cargo.toml --features test-support,write --test write_manifest --test write_pure
```

and above the `verify_commands:` key add a comment block:

```yaml
# 2026-10-05 (change worktree-org-node-chain-authority): the chain writer
# (SDD-yg7n55) moved here behind the `write` feature. Its targets run on a
# line of their own with `--features test-support,write`, so the reader's
# evidence is still gathered on a build without `write` (LLR-rxs5ec). The
# chopsticks target `write_genesis_e2e` is excluded, as every chopsticks
# target of this unit is: it needs a fork and on-chain/scripts/node_modules.
```

Commit: `T1: on-chain-client write feature and the pure chain writer`.

**Red→green attestations:** (DONE 2026-10-06, merged; 18 passed, 0 failed.)
Every test below was red at compile time — `error: the package
'on-chain-client' does not contain this feature: write` — before the feature
and the module it gates existed, and green once its implementation landed.
Compile-red only: none was watched failing at runtime against a wrong
implementation.

- the_write_feature_adds_exactly_the_signer_and_the_hasher — green after Cargo.toml/lib.rs
- no_other_feature_enables_the_writer_and_no_consumer_is_named — green after Cargo.toml
- update_calldata_is_the_pinned_hundred_bytes — green after src/write/calldata.rs
- calldata_at_the_boundary_values_keeps_its_layout — green after src/write/calldata.rs
- the_expected_epoch_occupies_the_low_sixteen_bytes_big_endian — green after src/write/calldata.rs
- the_largest_epoch_leaves_the_high_sixteen_bytes_zero — green after src/write/calldata.rs
- the_selector_is_the_keccak_of_the_update_signature — green after src/write/calldata.rs
- a_drifted_signature_is_a_different_selector — green after src/write/calldata.rs
- the_revive_call_names_every_field_and_constant — green after src/write/calldata.rs
- a_different_epoch_changes_only_the_data_field — green after src/write/calldata.rs
- proxied_and_map_account_build_the_named_calls — green after src/write/proxy.rs
- proxied_wraps_any_call_unchanged_even_another_proxied_call — green after src/write/proxy.rs
- the_derived_account_does_not_depend_on_signatory_order — green after src/write/multisig.rs
- empty_and_repeated_signatory_sets_derive_deterministically — green after src/write/multisig.rs
- the_derived_account_depends_on_the_threshold — green after src/write/multisig.rs
- boundary_thresholds_derive_distinct_accounts — green after src/write/multisig.rs
- dispatch_is_direct_without_co_signatories_and_sorted_multisig_with_them — green after src/write/multisig.rs
- a_value_that_is_not_a_runtime_call_is_refused_on_each_arm — green after src/write/multisig.rs

Surprises: Cargo.lock unchanged (blake2 and subxt-signer were already
dev-dependencies of the package); `clippy --all-targets -D warnings` is not
clean on this crate's pre-existing test targets with or without `write`
(baseline, not caused by T1; src/write/* and the new tests are clean).

---

### T2 — on-chain-client: composition, the subxt writer, the chopsticks port

**Files touched:** `on-chain-client/Cargo.toml`,
`on-chain-client/src/write/mod.rs`, `on-chain-client/src/write/ceremony.rs`
(new), `on-chain-client/src/write/subxt_ops.rs` (new),
`on-chain-client/tests/write_compose.rs` (new),
`on-chain-client/tests/write_genesis_e2e.rs` (new),
`on-chain-client/.guardrails/config.yaml`
**Parallel:** no (serial, after T1); parallel with T3–T12
**IDs verified:** LLR-a2acvh, LLR-z9vugt, LLR-kv27gp, LLR-hun4wf, REQ-6jefu2,
REQ-aat4yt (SDD-yg7n55).
**Size:** ~380 lines.

**Step 1 — failing tests.** `on-chain-client/tests/write_compose.rs`:

```rust
#![cfg(all(feature = "test-support", feature = "write"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The writer's composition (LLR-a2acvh, LLR-z9vugt, LLR-kv27gp, LLR-hun4wf),
//! driven through a substitute `WriteOps` that records each step and can fail
//! or answer `ApprovalRecorded` at any of them.

use std::sync::Mutex;

use on_chain_client::write::calldata::revive_update_runtime_call;
use on_chain_client::write::proxy::{map_account_call, proxied};
use on_chain_client::write::{
    genesis, submit_update, AccountId, DispatchOutcome, Genesis, GenesisStep, WriteError, WriteOps,
    FUND_AMOUNT,
};
use on_chain_client::{h160_of, Epoch, OnChainRootHash, OrgAdmin, OrgPubKey};
use subxt::dynamic::Value;

#[derive(Clone, Debug, PartialEq)]
enum Step {
    CreatePure(Vec<AccountId>),
    Fund(AccountId, u128),
    Dispatch(Vec<AccountId>, Value),
}

/// What the substitute does at step `n` (0-based, in the order asked).
#[derive(Clone, Copy, PartialEq)]
enum Answer {
    Ok,
    Fail,
    Approval,
}

struct FakeOps {
    steps: Mutex<Vec<Step>>,
    answers: Vec<Answer>,
}

const PROXY: AccountId = AccountId([0x5a; 32]);

impl FakeOps {
    fn answering(answers: &[Answer]) -> Self {
        Self { steps: Mutex::new(vec![]), answers: answers.to_vec() }
    }
    fn record(&self, s: Step) -> Answer {
        let mut steps = self.steps.lock().unwrap();
        steps.push(s);
        *self.answers.get(steps.len() - 1).unwrap_or(&Answer::Ok)
    }
    fn steps(&self) -> Vec<Step> {
        self.steps.lock().unwrap().clone()
    }
}

impl WriteOps for FakeOps {
    type Signer = ();
    async fn create_pure(&self, _: &(), co: &[AccountId]) -> Result<AccountId, WriteError> {
        match self.record(Step::CreatePure(co.to_vec())) {
            Answer::Fail => Err(WriteError::Subxt("create_pure refused".into())),
            _ => Ok(PROXY),
        }
    }
    async fn fund(&self, _: &(), dest: AccountId, amount: u128) -> Result<(), WriteError> {
        match self.record(Step::Fund(dest, amount)) {
            Answer::Fail => Err(WriteError::Subxt("transfer refused".into())),
            _ => Ok(()),
        }
    }
    async fn dispatch(&self, _: &(), co: &[AccountId], call: Value) -> Result<DispatchOutcome, WriteError> {
        match self.record(Step::Dispatch(co.to_vec(), call)) {
            Answer::Fail => Err(WriteError::Subxt("dispatch refused".into())),
            Answer::Approval => Ok(DispatchOutcome::ApprovalRecorded),
            Answer::Ok => Ok(DispatchOutcome::Executed),
        }
    }
}

const CONTRACT: [u8; 20] = [0xc0; 20];
fn co() -> Vec<AccountId> {
    vec![AccountId([2; 32]), AccountId([1; 32])]
}
fn root() -> OnChainRootHash {
    OnChainRootHash([0x33; 32])
}
fn key() -> OrgPubKey {
    OrgPubKey([0x22; 32])
}

// Normal: the four steps, in order, each with its exact call, and the result.
// verifies: LLR-a2acvh, REQ-6jefu2
#[tokio::test]
async fn genesis_creates_funds_maps_and_records_in_that_order() {
    let ops = FakeOps::answering(&[]);
    let out = genesis(&ops, &(), &co(), CONTRACT, root(), key()).await.unwrap();
    assert_eq!(out, Genesis { proxy: PROXY, admin: OrgAdmin(h160_of(PROXY.0)) });
    assert_eq!(
        ops.steps(),
        vec![
            Step::CreatePure(co()),
            Step::Fund(PROXY, FUND_AMOUNT),
            Step::Dispatch(co(), proxied(PROXY, map_account_call())),
            Step::Dispatch(co(), proxied(PROXY, revive_update_runtime_call(CONTRACT, root(), key(), Epoch(0)))),
        ]
    );
}

// Abnormal: a failure at each step is reported as that step, and no later
// step is attempted.
// verifies: LLR-z9vugt, REQ-6jefu2
#[tokio::test]
async fn a_failed_genesis_step_is_named_and_stops_the_ceremony() {
    let names = [GenesisStep::CreatePure, GenesisStep::Fund, GenesisStep::MapAccount, GenesisStep::RecordGenesis];
    for (i, step) in names.into_iter().enumerate() {
        let mut answers = vec![Answer::Ok; 4];
        answers[i] = Answer::Fail;
        let ops = FakeOps::answering(&answers);
        let err = genesis(&ops, &(), &co(), CONTRACT, root(), key()).await.unwrap_err();
        assert!(matches!(&err, WriteError::Step { step: s, .. } if *s == step), "step {i}: {err:?}");
        assert_eq!(ops.steps().len(), i + 1, "no step after {step:?} may run");
    }
}

// Abnormal: a dispatched genesis step that only records an approval is a
// failure of that step, not a success.
// verifies: LLR-z9vugt
#[tokio::test]
async fn an_approval_without_execution_fails_the_genesis_step() {
    for (i, step) in [(2usize, GenesisStep::MapAccount), (3, GenesisStep::RecordGenesis)] {
        let mut answers = vec![Answer::Ok; 4];
        answers[i] = Answer::Approval;
        let ops = FakeOps::answering(&answers);
        let err = genesis(&ops, &(), &co(), CONTRACT, root(), key()).await.unwrap_err();
        assert!(matches!(&err, WriteError::Step { step: s, .. } if *s == step), "{err:?}");
        assert_eq!(ops.steps().len(), i + 1);
    }
}

// Normal: exactly one dispatch, the proxied update at the expected epoch.
// verifies: LLR-kv27gp, REQ-aat4yt
#[tokio::test]
async fn submit_update_dispatches_one_proxied_update_and_succeeds_when_it_executes() {
    let ops = FakeOps::answering(&[]);
    submit_update(&ops, &(), &co(), CONTRACT, PROXY, root(), key(), Epoch(4)).await.unwrap();
    assert_eq!(
        ops.steps(),
        vec![Step::Dispatch(co(), proxied(PROXY, revive_update_runtime_call(CONTRACT, root(), key(), Epoch(4))))]
    );
}

// Abnormal: an approval that did not execute is PendingApproval; a failed
// dispatch returns the dispatch's own error.
// verifies: LLR-hun4wf, REQ-aat4yt
#[tokio::test]
async fn submit_update_reports_a_pending_approval_and_a_failed_dispatch() {
    let pending = FakeOps::answering(&[Answer::Approval]);
    assert_eq!(
        submit_update(&pending, &(), &co(), CONTRACT, PROXY, root(), key(), Epoch(4)).await.unwrap_err(),
        WriteError::PendingApproval
    );
    let failed = FakeOps::answering(&[Answer::Fail]);
    assert_eq!(
        submit_update(&failed, &(), &co(), CONTRACT, PROXY, root(), key(), Epoch(4)).await.unwrap_err(),
        WriteError::Subxt("dispatch refused".into())
    );
}
```

Add to `on-chain-client/Cargo.toml` after the `write_pure` entry:

```toml
[[test]]
name = "write_compose"
required-features = ["test-support", "write"]

# Chopsticks: outside every gate, like this unit's other chopsticks targets.
# Run: cargo test --manifest-path on-chain-client/Cargo.toml --features write
#      --test write_genesis_e2e -- --test-threads=1
[[test]]
name = "write_genesis_e2e"
required-features = ["write"]
```

**Step 2 — run, expect red.**
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test --manifest-path on-chain-client/Cargo.toml --features test-support,write --test write_compose`
Expected: `error[E0432]: unresolved imports on_chain_client::write::genesis, …
WriteOps, FUND_AMOUNT` (the composition does not exist yet).

**Step 3 — implement.** Append to `write/mod.rs`:

```rust
pub mod ceremony;
pub mod subxt_ops;

pub use ceremony::{genesis, submit_update, FUND_AMOUNT};

use crate::types::OrgAdmin;
use subxt::dynamic::Value;

/// What genesis returns: the pure proxy and the Organisation admin derived
/// from it (`h160_of(proxy)`), which is the Organisation's on-chain id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Genesis {
    pub proxy: AccountId,
    pub admin: OrgAdmin,
}

/// The seam the composition is written against (SDD-yg7n55): create a pure
/// proxy, fund an account, dispatch a call under the controller.
/// `SubxtWriteOps` implements it over subxt; a gated test substitutes it.
#[allow(async_fn_in_trait)]
pub trait WriteOps {
    type Signer;
    async fn create_pure(&self, signatory: &Self::Signer, co_signatories: &[AccountId]) -> Result<AccountId, WriteError>;
    async fn fund(&self, signatory: &Self::Signer, dest: AccountId, amount: u128) -> Result<(), WriteError>;
    async fn dispatch(
        &self,
        signatory: &Self::Signer,
        co_signatories: &[AccountId],
        call: Value,
    ) -> Result<DispatchOutcome, WriteError>;
}
```

`on-chain-client/src/write/ceremony.rs`:

```rust
//! Genesis and update submission over the `WriteOps` seam
//! (LLR-a2acvh, LLR-z9vugt, LLR-kv27gp, LLR-hun4wf).

use super::calldata::revive_update_runtime_call;
use super::proxy::{map_account_call, proxied};
use super::{AccountId, DispatchOutcome, Genesis, GenesisStep, WriteError, WriteOps};
use crate::h160::h160_of;
use crate::types::{Epoch, OnChainRootHash, OrgAdmin, OrgPubKey};

/// 100 PAS at 10 decimals: existential deposit, fees and revive deposits for
/// a fresh pure proxy.
pub const FUND_AMOUNT: u128 = 1_000_000_000_000;

fn failed(step: GenesisStep, e: WriteError) -> WriteError {
    WriteError::Step { step, reason: e.to_string() }
}

fn executed(step: GenesisStep, outcome: Result<DispatchOutcome, WriteError>) -> Result<(), WriteError> {
    match outcome {
        Ok(DispatchOutcome::Executed) => Ok(()),
        Ok(DispatchOutcome::ApprovalRecorded) => {
            Err(WriteError::Step { step, reason: "multisig approval recorded; the call did not execute".into() })
        }
        Err(e) => Err(failed(step, e)),
    }
}

/// Stand up an Organisation's slot: create the pure proxy under the
/// controller, fund it from the signatory, map it in pallet-revive, record
/// the genesis root and key at epoch zero — each step only after the previous
/// one succeeded (LLR-a2acvh); a failure names its step (LLR-z9vugt).
pub async fn genesis<O: WriteOps>(
    ops: &O,
    signatory: &O::Signer,
    co_signatories: &[AccountId],
    contract: [u8; 20],
    root: OnChainRootHash,
    key: OrgPubKey,
) -> Result<Genesis, WriteError> {
    let proxy = ops.create_pure(signatory, co_signatories).await.map_err(|e| failed(GenesisStep::CreatePure, e))?;
    ops.fund(signatory, proxy, FUND_AMOUNT).await.map_err(|e| failed(GenesisStep::Fund, e))?;
    executed(
        GenesisStep::MapAccount,
        ops.dispatch(signatory, co_signatories, proxied(proxy, map_account_call())).await,
    )?;
    let record = revive_update_runtime_call(contract, root, key, Epoch(0));
    executed(GenesisStep::RecordGenesis, ops.dispatch(signatory, co_signatories, proxied(proxy, record)).await)?;
    Ok(Genesis { proxy, admin: OrgAdmin(h160_of(proxy.0)) })
}

/// Submit one update to the Organisation's slot through its proxy; `Ok` only
/// when the dispatch executed (LLR-kv27gp, LLR-hun4wf).
#[allow(clippy::too_many_arguments)]
pub async fn submit_update<O: WriteOps>(
    ops: &O,
    signatory: &O::Signer,
    co_signatories: &[AccountId],
    contract: [u8; 20],
    proxy: AccountId,
    root: OnChainRootHash,
    key: OrgPubKey,
    expected_epoch: Epoch,
) -> Result<(), WriteError> {
    let call = proxied(proxy, revive_update_runtime_call(contract, root, key, expected_epoch));
    match ops.dispatch(signatory, co_signatories, call).await? {
        DispatchOutcome::Executed => Ok(()),
        DispatchOutcome::ApprovalRecorded => Err(WriteError::PendingApproval),
    }
}
```

`on-chain-client/src/write/subxt_ops.rs` — the subxt implementation (no
low-level requirement: reachable only against a chain, as SDD-yg7n55 records):

```rust
//! `WriteOps` over subxt, and the sinks that drive or await block production.

use futures_util::future::{self, Either};
use subxt::OnlineClient;
use subxt::config::PolkadotConfig;
use subxt::dynamic::{self, Value};
use subxt::ext::scale_value::{Composite, Primitive, ValueDef};
use subxt::extrinsics::ExtrinsicEvents;
use subxt_signer::sr25519::Keypair;

use super::multisig::build_dispatch_tx;
use super::proxy::create_pure_call;
use super::{AccountId, DispatchOutcome, WriteError, WriteOps};

/// Makes the chain advance while a submitted extrinsic waits for finality.
#[allow(async_fn_in_trait)]
pub trait BlockSink {
    async fn settle(&self) -> Result<(), WriteError>;
}

/// The live-chain sink: a live chain finalises on its own, so it never
/// resolves and the extrinsic's own finality is the only completion signal.
pub struct FinalitySink;

impl BlockSink for FinalitySink {
    async fn settle(&self) -> Result<(), WriteError> {
        future::pending::<()>().await;
        Ok(())
    }
}

pub struct SubxtWriteOps<S: BlockSink> {
    api: OnlineClient<PolkadotConfig>,
    sink: S,
}

impl<S: BlockSink> SubxtWriteOps<S> {
    pub fn new(api: OnlineClient<PolkadotConfig>, sink: S) -> Self {
        Self { api, sink }
    }

    /// Sign and submit `tx`, settling blocks until it is finalised; an
    /// `ExtrinsicFailed` is an error.
    async fn submit_and_watch<Call: subxt::transactions::Payload>(
        &self,
        signer: &Keypair,
        tx: &Call,
    ) -> Result<ExtrinsicEvents<PolkadotConfig>, WriteError> {
        let progress = self
            .api
            .tx()
            .await
            .map_err(|e| WriteError::Subxt(format!("tx_client: {e}")))?
            .sign_and_submit_then_watch_default(tx, signer)
            .await
            .map_err(|e| WriteError::Subxt(format!("submit: {e}")))?;
        let fin = progress.wait_for_finalized_success();
        futures_util::pin_mut!(fin);
        loop {
            let settle = self.sink.settle();
            futures_util::pin_mut!(settle);
            match future::select(fin.as_mut(), settle).await {
                Either::Left((res, _)) => {
                    return res.map_err(|e| WriteError::Subxt(format!("extrinsic dispatch failed: {e}")));
                }
                Either::Right((settled, _)) => settled?,
            }
        }
    }
}

impl<S: BlockSink> WriteOps for SubxtWriteOps<S> {
    type Signer = Keypair;

    async fn create_pure(&self, signatory: &Keypair, co_signatories: &[AccountId]) -> Result<AccountId, WriteError> {
        let tx = build_dispatch_tx(co_signatories, create_pure_call())?.into_payload();
        let events = self.submit_and_watch(signatory, &tx).await?;
        for ev in events.iter() {
            let ev = ev.map_err(|e| WriteError::Subxt(format!("event iter: {e}")))?;
            if ev.pallet_name() == "Proxy" && ev.event_name() == "PureCreated" {
                let fields: Composite<()> =
                    ev.decode_fields_unchecked_as().map_err(|e| WriteError::Subxt(format!("decode fields: {e}")))?;
                return account32_from_named_field(&fields, "pure");
            }
        }
        Err(WriteError::EventNotFound("Proxy.PureCreated"))
    }

    async fn fund(&self, signatory: &Keypair, dest: AccountId, amount: u128) -> Result<(), WriteError> {
        let dest = Value::variant("Id", Composite::unnamed(vec![Value::from_bytes(dest.0.as_slice())]));
        let tx = dynamic::tx("Balances", "transfer_keep_alive", vec![dest, Value::u128(amount)]);
        self.submit_and_watch(signatory, &tx).await.map(|_| ())
    }

    async fn dispatch(
        &self,
        signatory: &Keypair,
        co_signatories: &[AccountId],
        call: Value,
    ) -> Result<DispatchOutcome, WriteError> {
        let tx = build_dispatch_tx(co_signatories, call)?.into_payload();
        self.submit_and_watch(signatory, &tx).await?;
        Ok(DispatchOutcome::Executed)
    }
}
```

Below that, paste org-node's `account32_from_named_field` and
`collect_account32` (`org-node/src/chain_write/proxy.rs`, from
`fn account32_from_named_field` to the end of the file) with
`ChainAccount::new` replaced by `AccountId` and `Result<ChainAccount, _>` by
`Result<AccountId, _>`.

Add to `on-chain-client/Cargo.toml` `[dev-dependencies]` nothing: `tokio`
(rt, macros) is already there for `#[tokio::test]`.

**Step 4 — the chopsticks port.** `on-chain-client/tests/write_genesis_e2e.rs`
ports the write half of `org-node/tests/chain_genesis_e2e.rs`'s
`single_admin_genesis_e2e` (the org-node trie and verify half stays in
org-node until T13 deletes it; this unit may not depend on org-node):

```rust
#![cfg(all(feature = "write", feature = "dev-rpc"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Chopsticks, outside every gate: the chain writer against a Paseo-AH fork.
//! Run: pkill -f "chopsticks.*--config"; CARGO_HOME=/tmp/cargo_home_fuzz \
//!   cargo test --manifest-path on-chain-client/Cargo.toml --features write \
//!   --test write_genesis_e2e -- --test-threads=1 --nocapture

mod common;

use common::chopsticks_fork::{spawn_fork, ChopsticksHandle};
use common::chopsticks_reorg::mine_block;
use common::conn::legacy_client;
use on_chain_client::write::subxt_ops::{BlockSink, SubxtWriteOps};
use on_chain_client::write::{genesis, submit_update, WriteError};
use on_chain_client::{Epoch, OnChainRootHash, OrgPubKey, OrgRegistryClient};
use subxt_signer::sr25519::dev;

struct ChopsticksSink<'a> {
    handle: &'a ChopsticksHandle,
}

impl BlockSink for ChopsticksSink<'_> {
    async fn settle(&self) -> Result<(), WriteError> {
        mine_block(self.handle).await.map(|_| ()).map_err(|e| WriteError::Subxt(format!("{e:?}")))
    }
}
```

followed by `deploy_org_registry()` copied verbatim from
`org-node/tests/chain_genesis_e2e.rs` (its `CARGO_MANIFEST_DIR` join of
`../on-chain` resolves the same from this crate), and:

```rust
#[tokio::test(flavor = "multi_thread")]
async fn single_signatory_genesis_then_update_moves_the_slot() {
    let fork = spawn_fork().await.expect("spawn chopsticks fork");
    let contract = deploy_org_registry();
    let api = legacy_client(&fork.ws_url).await.expect("legacy subxt client");
    let ops = SubxtWriteOps::new(api.clone(), ChopsticksSink { handle: &fork });
    let alice = dev::alice();

    let g = genesis(&ops, &alice, &[], contract, OnChainRootHash([0x33; 32]), OrgPubKey([0x22; 32]))
        .await
        .expect("genesis");
    let reader = OrgRegistryClient::from_client(api.clone(), contract).await.expect("reader");
    let state = reader.get_org_state(g.admin, None).await.expect("read").expect("slot");
    assert_eq!((state.root_hash, state.epoch), (OnChainRootHash([0x33; 32]), Epoch(1)));

    submit_update(&ops, &alice, &[], contract, g.proxy, OnChainRootHash([0x66; 32]), OrgPubKey([0x22; 32]), Epoch(1))
        .await
        .expect("update");
    let state = reader.get_org_state(g.admin, None).await.expect("read").expect("slot");
    assert_eq!((state.root_hash, state.epoch), (OnChainRootHash([0x66; 32]), Epoch(2)));
}
```

If `tests/common/mod.rs` does not already declare `chopsticks_fork`,
`chopsticks_reorg` and `conn` as `pub mod`, use the declarations it has (read
it first); do not edit `tests/common`.

**Step 5 — run, expect green.**
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test --manifest-path on-chain-client/Cargo.toml --features test-support,write --test write_manifest --test write_pure --test write_compose`
→ `write_compose: test result: ok. 5 passed`, the other two as in T1.
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test --manifest-path on-chain-client/Cargo.toml --features write --test write_genesis_e2e --no-run`
→ compiles (it is not run here: chopsticks). Clippy as in T1 Step 6.

**Step 6 — config.** Replace the T1 writer line in
`on-chain-client/.guardrails/config.yaml` with:

```yaml
  - cargo test --manifest-path on-chain-client/Cargo.toml --features test-support,write --test write_manifest --test write_pure --test write_compose
```

Commit: `T2: on-chain-client writer composition, subxt implementation, chopsticks port`.

**Red→green attestations:** (DONE 2026-10-06, merged; 23 passed, 0 failed,
1 not run.) Each test was compile-red first (E0432: `genesis`,
`submit_update`, `Genesis`, `WriteOps`, `FUND_AMOUNT` unresolved), then
runtime-red against stub bodies returning `Err(Subxt("stub"))`, then green
after `src/write/ceremony.rs`:

- genesis_creates_funds_maps_and_records_in_that_order — runtime red: unwrap on Err
- a_failed_genesis_step_is_named_and_stops_the_ceremony — runtime red: error was not `WriteError::Step`
- an_approval_without_execution_fails_the_genesis_step — runtime red: error was not `Step{MapAccount}`
- submit_update_dispatches_one_proxied_update_and_succeeds_when_it_executes — runtime red: unwrap on Err
- submit_update_reports_a_pending_approval_and_a_failed_dispatch — runtime red: `Subxt("stub")` ≠ `PendingApproval`

Not run: `single_signatory_genesis_then_update_moves_the_slot`
(`tests/write_genesis_e2e.rs`, chopsticks, no `verifies:`, outside every
gate). The sandbox has no network and no `on-chain/scripts/node_modules`; it
compiles with `--no-run` only. Its predecessor in org-node ran in the
chopsticks lane, also outside the gate.

---

### T3 — org-node: story helpers shared by the service-level tests

**Files touched:** `org-node/tests/support/mod.rs` (new),
`org-node/tests/admission_sender.rs`, `org-node/tests/service_stories.rs`
**Parallel:** yes (with T1 and T2)
**IDs verified:** none new — a test refactor. Every annotation in the two
files is kept verbatim; the gate's pass counts must not change
(admission_sender 39, service_stories 3).
**Size:** ~350 lines (helpers moved out of `admission_sender.rs`, ~45 call
sites rewritten).

Why: T6, T7, T9, T10 and T11 each change how a story step is performed
(found, prepare to join, admit, revoke). Routing every call site through one
helper module now means each of those tasks changes a helper body once,
plus only the tests whose *assertions* change. `support/` is not a target
(Cargo auto-discovers only `tests/*.rs` and `tests/*/main.rs`); each target
that uses it declares `mod support;`.

**Step 1 — baseline.** Run
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test admission_sender --test service_stories`
and record the counts (expected `39 passed`, `3 passed`).

**Step 2 — create `org-node/tests/support/mod.rs`:**

```rust
//! Story helpers shared by org-node's service-level test targets. Not a test
//! target: each target that uses it declares `mod support;`.
//!
//! The story operations — `found`, `prepare_to_join`, `joiner_of`, `admit`,
//! `revoke` — are the only place a test performs a story step, so a change to
//! the service API changes their bodies and not the tests that call them.
#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use std::time::Duration;

use org_members::{Handle, MemberId, Name, P2pDeviceKey, Surname};
use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::keys::SigningKeypair;
use org_node::service::{MockChainOps, OrgService, ReceiveOutcome, SelfDeleteOutcome};
use org_node::store::{OrgRecord, PersonaRecord, PersonaStore};
use org_node::transport::endpoint::OrgEndpoint;
use org_node::transport::wire::WireMessage;
use org_node::{DeviceSeed, Epoch, OrgSecret, PersonaId};
use rand::rngs::OsRng;

pub const NET: Duration = Duration::from_secs(30);

/// The rogue relay's device seed — a third device, neither A's nor B's.
pub const ROGUE_SEED: [u8; 32] = [0x33u8; 32];

/// The Organisation secret every admission in these tests hands over.
pub fn org_secret() -> Option<OrgSecret> {
    Some(OrgSecret::from([0xffu8; 32]))
}

pub fn h(s: &str) -> Handle {
    Handle::parse(s).unwrap()
}
pub fn nm(s: &str) -> Name {
    Name::parse(s).unwrap()
}
pub fn sn(s: &str) -> Surname {
    Surname::parse(s).unwrap()
}

/// One party's store directory, unique per target, test and process.
pub fn store_dir(tag: &str, party: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "ods-{}-{tag}-{party}-{}",
        env!("CARGO_CRATE_NAME"),
        std::process::id()
    ))
}

/// A fresh encrypted store, wiping anything an earlier run left there.
pub fn open_store(tag: &str, party: &str, password: &str) -> PersonaStore {
    let dir = store_dir(tag, party);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    PersonaStore::open(dir.join("store.bin"), password).unwrap()
}

/// Reopen a store without wiping it: what actually reached the disk.
pub fn reopen_store(tag: &str, party: &str, password: &str) -> PersonaStore {
    PersonaStore::open(store_dir(tag, party).join("store.bin"), password).unwrap()
}

/// The raw bytes of a party's store file (to prove nothing was written).
pub fn store_bytes(tag: &str, party: &str) -> Vec<u8> {
    std::fs::read(store_dir(tag, party).join("store.bin")).unwrap()
}
```

Then paste, unchanged in body, these functions from
`org-node/tests/admission_sender.rs`, made `pub`: `persona_of`, `device_kp`,
`admin_keys`, `rec_of`, `disk_rec_of`, `id_by_handle`, `dead_addr`,
`spawn_b_receive` (renamed `spawn_receive`), `spawn_b_self_delete` (renamed
`spawn_self_delete`), `spawn_recv_one`. Then add the story operations, whose
bodies are today's API:

```rust
/// What an administrator admits a Persona as. Today the Join request; T6
/// makes it org-node's `Joiner`.
pub type JoinerOf = org_node::blobs::JoinRequest;

/// Story 1: `pid` founds an Organisation on `chain`.
pub async fn found(svc: &mut OrgService, chain: &MockChainOps, pid: &PersonaId) -> OrgId {
    let _ = chain;
    svc.create_organisation(&mut OsRng, pid).await.expect("found an organisation")
}

/// Story 2, the joining node's half: whatever it must hold before an
/// admission to `org_id` reaches it.
pub fn prepare_to_join(svc_a: &OrgService, svc_b: &mut OrgService, org_id: OrgId) {
    let blob = svc_a.export_invite(org_id).expect("export invite");
    svc_b.import_invite(&mut OsRng, &blob).expect("import invite");
}

/// The joiner `pid` of `svc` is admitted as.
pub fn joiner_of(svc: &OrgService, pid: &PersonaId) -> JoinerOf {
    OrgService::import_join_request(&svc.export_join_request(pid).unwrap()).unwrap()
}

/// The device key the joiner's node is reached by.
pub fn joiner_device(joiner: &JoinerOf) -> P2pDeviceKey {
    joiner.device_key
}

/// Story 3: admit `joiner` into `org_id`, delivering to `addr`.
pub async fn admit(
    svc: &mut OrgService,
    chain: &MockChainOps,
    org_id: OrgId,
    joiner: &JoinerOf,
    addr: iroh::EndpointAddr,
    secret: Option<OrgSecret>,
) -> Result<MemberId, OrgNodeError> {
    let _ = chain;
    tokio::time::timeout(NET, svc.admit_member(&mut OsRng, org_id, joiner, addr, secret))
        .await
        .expect("admit timed out")
}

/// Story 5: revoke `member_id` from `org_id`, delivering to `addr`.
pub async fn revoke(
    svc: &mut OrgService,
    chain: &MockChainOps,
    org_id: OrgId,
    member_id: MemberId,
    addr: Option<iroh::EndpointAddr>,
) -> Result<(), OrgNodeError> {
    let _ = chain;
    tokio::time::timeout(NET, svc.revoke_member(&mut OsRng, org_id, member_id, addr))
        .await
        .expect("revoke timed out")
}
```

Then move `struct Setup`, `setup`, `admit_b_directly` and
`join_request_for_c` from `admission_sender.rs` into the module, `pub`, with
these changes only: `Setup` gains `pub pid_a: PersonaId` and its field
`join_request_b: JoinRequest` becomes `joiner_b: JoinerOf`; in `setup` the
`create_organisation` call becomes `found(&mut svc_a, &chain, &pid_a).await`,
the invite export/import and its `assert_eq!(invite.org_id, org_id)` become
`prepare_to_join(&svc_a, &mut svc_b, org_id)`, and `join_request_of(&svc_b,
&pid_b)` becomes `joiner_of(&svc_b, &pid_b)`; in `admit_b_directly` the
`admit_member` call becomes
`admit(&mut s.svc_a, &s.chain, s.org_id, &s.joiner_b, b_addr, org_secret()).await.expect("admit_member(B) failed")`;
`join_request_for_c` is renamed `joiner_for_c` and returns `JoinerOf` via
`joiner_of(svc_a, &pid_c)`.

**Step 3 — `admission_sender.rs`.** Delete every helper now in `support`
(lines 38–252 of today's file: `org_secret` through `join_request_for_c`, and
`spawn_b_self_delete`); add `mod support;` and `use support::*;` after the
crate attributes; drop imports that become unused. Then, mechanically:

| Today | After T3 |
|---|---|
| `s.join_request_b` | `s.joiner_b` |
| `join_request_for_c(&mut x)` | `joiner_for_c(&mut x)` |
| `spawn_b_receive(..)` / `spawn_b_self_delete(..)` | `spawn_receive(..)` / `spawn_self_delete(..)` |
| `tokio::time::timeout(NET, X.admit_member(&mut OsRng, O, &J, A, S)).await.expect(..)` | `admit(&mut X, &CHAIN, O, &J, A, S).await` |
| `tokio::time::timeout(NET, X.revoke_member(&mut OsRng, O, M, A)).await.expect(..)` | `revoke(&mut X, &CHAIN, O, M, A).await` |
| `X.create_organisation(&mut OsRng, &P).await.unwrap()` | `found(&mut X, &CHAIN, &P).await` |
| an `export_invite` + `import_invite` pair used only to set B up | `prepare_to_join(&svc_a, &mut svc_b, org_id)` |
| `join_request_of(&svc, &pid)` | `joiner_of(&svc, &pid)` |

`CHAIN` is the `MockChainOps` the test already holds (`s.chain`, `chain`). The
trailing `.expect("…failed")`/`.unwrap()`/`.unwrap_err()` on the inner
`Result` is kept as written. Leave untouched (they are rewritten by the task
that removes what they assert): the direct invite/join-request calls in
`an_imported_invite_reaches_the_joiners_disk`,
`the_out_of_band_blobs_carry_the_keys_their_holders_are_pinned_by`,
`pr_8qsnhx_a_join_request_advertises_the_bound_endpoint_not_its_personas`,
`pr_8qsnhx_an_invite_advertises_the_bound_endpoint_not_its_administrators`,
and every call in `the_proxy_account_from_genesis_is_kept_and_passed_on_every_update`
(its chain is `ProxyChain`, not `MockChainOps`).

**Step 4 — `service_stories.rs`.** Add `mod support;` and
`use support::{admit, found, joiner_of, prepare_to_join, revoke, NET};`; in
all three tests replace the `create_organisation`, invite export/import,
join-request export/import, `admit_member` and `revoke_member` calls by the
table above (`chain` is each test's `MockChainOps`). Keep each test's own
local helpers and every assertion.

**Step 5 — green to green.** Re-run Step 1's command: the same counts, `0
failed`. Run the whole org-node line (Verification header, the line as it
stands before T4: today's config line). Commit: `T3: org-node story helpers
for the service-level tests`.

**Red→green attestations:** (refactor: green before and after, counts equal)
DONE 2026-10-06, merged. No test added. admission_sender 39 → 39,
service_stories 3 → 3; org-node's cargo gate line 167 passed / 0 failed before
and after. Departures from the plan text: service_stories keeps its local
`NET`; the `invite.org_id == org_id` assertions are kept via
`list_pending_invites()`; two tests on the "leave untouched" list call
`import_join_request(&export_join_request(..))` directly (they used the
removed `join_request_of`); store directory names now come from
CARGO_CRATE_NAME (`ods-admission_sender-…`).

---

### T4 — withdrawn: master already did it (2026-10-06)

The merge of master `5f7c177` into this change (`1f483c8`) brought every
behaviour T4 planned, with its tests, written by master's switch-trim
(`docs/plans/2026-10-05-switch-trim.md`):

- `Envelope` (renamed from `SignedDeltaEnvelope`) has three fields and no
  signature; `build` takes no key; `keys::verify`, `SigningKeypair::sign` and
  `OrgNodeError::BadSignature` are gone; `VerifyContext` holds no key
  (`org-node/src/envelope.rs`, `keys.rs`, `verify.rs`, `error.rs`; tests in
  `envelope_binding.rs`, `key_custody.rs`, `verify_against_chain.rs`,
  `value_types.rs`).
- Neither Receive operation checks anything about the sender, and a first
  admission needs no Invite (`receive_and_verify`,
  `receive_and_self_delete_if_revoked`; tests in `admission_sender.rs`,
  including the renamed sender tests this task listed).
- PR-u4c2vp is resolved, on this change's ruling, with its test
  `pr_u4c2vp_an_update_relayed_by_a_non_member_is_committed_on_the_self_delete_path`.
- The wire golden is re-pinned without the signature (`encoding_golden.rs`).

What T4 also planned and master did not do moves to the task that first
needs it: `org-node/tests/absences.rs` is created in T7, with the absences of
the invitation exchange (master's switch-trim verifies LLR-na7p4w's
"no key" by `verify_against_chain`, so `signing_keypair_offers_no_signature`
is not written). Nothing else remains.

**Red→green attestations:** none (withdrawn; no commit).

---

### T5 — org-node: every chain-free check before the chain is read

**Files touched:** `org-node/src/verify.rs`, `org-node/src/service.rs`,
`org-node/tests/support/mod.rs`, `org-node/tests/receive_chain_reads.rs`
(new), `org-node/tests/verify_against_chain.rs`,
`org-node/tests/admission_sender.rs`, `org-node/Cargo.toml`,
`org-node/.guardrails/config.yaml`
**Parallel:** no (serial, first of the org-node chain)
**IDs verified:** REQ-f2k4tr (RC-mj6gjq), LLR-fuq379, LLR-9ew26y,
LLR-3wb7th, LLR-379hnv, REQ-bvh8v6, REQ-mr5abb.
**Size:** ~350 lines.

Today (merged tree) both Receive operations call `ChainOps::read_state`
first and only then run verify-against-chain, whose own first four checks
need no chain; `receive_and_verify` also reads the chain before it decodes a
first admission's snapshot, because it takes the record's administrator key
from the chain's Organisation public key when no Invite was imported.

**Step 1 — support additions** (`org-node/tests/support/mod.rs`; no
behaviour of the service changes, so these compile against today's code).
Master's `support` already has `setup(tag)` and `setup_with(tag,
import_invite: bool)`. Generalise the second over B's chain:

```rust
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use org_node::chain::OrgState;
use org_node::service::ChainOps;
use org_node::store::MemberSnapshot;
use org_node::test_fixtures::Trie;
use org_node::{ChainAccount, OrgPublicKey, RootHash};

/// `setup_with`, with B's service reading the chain through `b_chain(chain)`.
pub async fn setup_over(
    tag: &str,
    prepare: bool,
    b_chain: impl FnOnce(MockChainOps) -> Box<dyn ChainOps>,
) -> Setup {
    // the body of today's `setup_with`, with `svc_b` built as
    // `OrgService::new(open_store(tag, "b", "pw_b"), b_chain(chain.clone()))`
    // and `prepare_to_join` called only when `prepare`
}

pub async fn setup_with(tag: &str, prepare: bool) -> Setup {
    setup_over(tag, prepare, |c| Box::new(c)).await
}

/// Send `msg` to `addr` from a relay device that is neither A nor B.
pub async fn deliver(addr: iroh::EndpointAddr, msg: &WireMessage) {
    let relay = OrgEndpoint::bind(&DeviceSeed::from([0x5bu8; 32]).signing_keypair()).await.unwrap();
    tokio::time::timeout(NET, relay.send(addr, msg)).await.expect("deliver timed out").expect("deliver failed");
}

/// The encoded member snapshots of `trie`, as a first admission carries them.
pub fn snapshot_bytes(trie: &Trie) -> Vec<u8> {
    let members: Vec<MemberSnapshot> = trie
        .members()
        .iter()
        .map(|m| MemberSnapshot {
            id: *m.id(),
            handle: m.handle().clone(),
            name: m.name().clone(),
            surname: m.surname().clone(),
            member_key: *m.p2p_key(),
            device_keys: m.p2p_devices().to_vec(),
        })
        .collect();
    postcard::to_allocvec(&members).unwrap()
}

/// A `ChainOps` over a shared `MockChainOps` that counts `read_state` calls,
/// can hide an Organisation (answer `None` for it) and can fail every read.
#[derive(Clone)]
pub struct CountingChain {
    pub inner: MockChainOps,
    reads: Arc<AtomicUsize>,
    hidden: Arc<Mutex<Vec<OrgId>>>,
    failing: Arc<AtomicBool>,
}

impl CountingChain {
    pub fn over(inner: MockChainOps) -> Self {
        Self { inner, reads: Default::default(), hidden: Default::default(), failing: Default::default() }
    }
    pub fn reads(&self) -> usize {
        self.reads.load(Ordering::SeqCst)
    }
    pub fn hide(&self, org: OrgId) {
        self.hidden.lock().unwrap().push(org);
    }
    pub fn fail_reads(&self) {
        self.failing.store(true, Ordering::SeqCst);
    }
}

#[async_trait::async_trait]
impl ChainOps for CountingChain {
    // Delegates while the trait still has it (T9 removes it).
    async fn submit_genesis(&self, root: RootHash, key: OrgPublicKey) -> Result<(OrgId, Option<ChainAccount>), OrgNodeError> {
        self.inner.submit_genesis(root, key).await
    }
    // Delegates while the trait still has it (T11 removes it).
    async fn submit_update(
        &self,
        org: OrgId,
        root: RootHash,
        key: OrgPublicKey,
        epoch: Epoch,
        proxy: Option<ChainAccount>,
    ) -> Result<(), OrgNodeError> {
        self.inner.submit_update(org, root, key, epoch, proxy).await
    }
    async fn read_state(&self, org: OrgId) -> Result<Option<OrgState>, OrgNodeError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        if self.failing.load(Ordering::SeqCst) {
            return Err(OrgNodeError::Chain("chain unreachable".into()));
        }
        if self.hidden.lock().unwrap().contains(&org) {
            return Ok(None);
        }
        self.inner.read_state(org).await
    }
}

/// `setup`, with B reading the chain through a `CountingChain`.
pub async fn setup_counted(tag: &str) -> (Setup, CountingChain) {
    let mut slot = None;
    let s = setup_over(tag, true, |c| {
        let counting = CountingChain::over(c);
        slot = Some(counting.clone());
        Box::new(counting)
    })
    .await;
    (s, slot.expect("setup_over calls its closure"))
}
```

(`org-node` has `async-trait` as a dev-dependency already: the app feature
pulls it in, and `organisation_key.rs` implements `ChainOps` the same way.)

**Step 2 — failing tests.** Append to `org-node/tests/verify_against_chain.rs`
(its `setup`, `ctx`, `chain_at` and `garbage` helpers are master's):

```rust
use org_node::verify::check_chain_free;
use std::cell::Cell;

/// A chain reader that counts its reads.
struct Counting {
    inner: MockChain,
    reads: Cell<usize>,
}
impl org_node::chain::ChainReader for Counting {
    fn get_org_state(&self, org: &OrgId) -> Result<Option<OrgState>, String> {
        self.reads.set(self.reads.get() + 1);
        self.inner.get_org_state(org)
    }
}

// Normal: an honest envelope passes the four chain-free checks.
// verifies: REQ-f2k4tr, LLR-fuq379
#[test]
fn check_chain_free_passes_an_honest_envelope() {
    let (org, local, env, _) = setup();
    assert_eq!(check_chain_free(&local, &env, &ctx(org)), Ok(()));
}

// Abnormal: each chain-free check refuses with its own error, in order.
// verifies: REQ-f2k4tr, LLR-fuq379
#[test]
fn check_chain_free_refuses_each_check_in_order() {
    let (org, local, _env, _) = setup();
    assert_eq!(check_chain_free(&local, &garbage(org, 2), &ctx(OrgId::new([0xee; 20]))), Err(OrgNodeError::OrgIdMismatch));
    assert_eq!(check_chain_free(&local, &garbage(org, 1), &ctx(org)), Err(OrgNodeError::StaleSeq { got: 1, last_seen: 1 }));
    assert_eq!(check_chain_free(&local, &garbage(org, 2), &ctx(org)), Err(OrgNodeError::MalformedDelta));
    let stranger = MemberSeed::from([0x5a; 32]).x25519_keypair();
    let (other_base, _) = admit_member_delta(&stranger);
    let wrong_base = Envelope::build(org, SequenceNumber::new(2), &other_base).unwrap();
    assert_eq!(check_chain_free(&local, &wrong_base, &ctx(org)), Err(OrgNodeError::DeltaBaseMismatch));
}

// Abnormal and normal: verify reads the chain only after the chain-free
// checks pass, and then exactly once.
// verifies: REQ-f2k4tr, LLR-fuq379
#[test]
fn verify_reads_the_chain_once_and_only_after_the_chain_free_checks() {
    let (org, local, env, new_root) = setup();
    let chain = Counting { inner: chain_at(org, new_root, Epoch::new(2)), reads: Cell::new(0) };
    let stale = Envelope { parent_seq: SequenceNumber::new(1), ..env.clone() };
    assert!(verify_envelope_against_chain(&local, &stale, &ctx(org), &chain).is_err());
    assert_eq!(chain.reads.get(), 0, "a chain-free refusal reads no chain");
    verify_envelope_against_chain(&local, &env, &ctx(org), &chain).unwrap();
    assert_eq!(chain.reads.get(), 1, "a passing envelope reads the chain once");
}
```

Create `org-node/tests/receive_chain_reads.rs`:

```rust
#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The receive paths run every chain-free check before they read the chain,
//! and read it at most once (REQ-f2k4tr, RC-mj6gjq; LLR-9ew26y, LLR-3wb7th,
//! LLR-379hnv), observed through a chain that counts its reads.

mod support;

use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::test_fixtures::admit_member_delta;
use org_node::transport::wire::WireMessage;
use org_node::{Envelope, Epoch, MemberSeed, SequenceNumber};
use support::*;

/// B admitted at epoch 2, B's chain reads counted.
async fn admitted(tag: &str) -> (Setup, CountingChain) {
    let (s, counting) = setup_counted(tag).await;
    (admit_b_directly(s).await, counting)
}

/// A admits C; the admission message is captured, not delivered to B.
async fn captured_admission_of_c(s: &mut Setup) -> WireMessage {
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &joiner_c, sink_addr, org_secret()).await.unwrap();
    sink.await.unwrap().2
}

/// A Change set on a base B does not hold.
fn foreign_delta() -> org_members::delta::Delta {
    admit_member_delta(&MemberSeed::from([0x5a; 32]).x25519_keypair()).0
}

// Normal: an update that passes the chain-free checks reads the chain once.
// verifies: REQ-f2k4tr, LLR-9ew26y
#[tokio::test(flavor = "multi_thread")]
async fn an_update_that_passes_the_chain_free_checks_reads_the_chain_once() {
    let (mut s, counting) = admitted("reads-once").await;
    let msg = captured_admission_of_c(&mut s).await;
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(b_addr, &msg).await;
    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap().epoch, Epoch::new(3));
    assert_eq!(counting.reads() - before, 1);
    assert_eq!(rec_of(&svc_b, s.org_id).trie_members.len(), 3);
}

// Abnormal: a replayed envelope is refused as stale with no chain read, and
// the record — mark included — is untouched.
// verifies: REQ-f2k4tr, REQ-mr5abb, LLR-9ew26y
#[tokio::test(flavor = "multi_thread")]
async fn a_stale_envelope_is_refused_without_a_chain_read() {
    let (mut s, counting) = admitted("stale").await;
    let msg = captured_admission_of_c(&mut s).await;
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(b_addr, &msg).await;
    let (svc_b, first) = b_task.await.unwrap();
    first.unwrap();
    let held = rec_of(&svc_b, s.org_id);
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(svc_b, &s.b_device_kp).await;
    deliver(b_addr, &msg).await;
    let (svc_b, again) = b_task.await.unwrap();
    assert!(matches!(again, Err(OrgNodeError::StaleSeq { .. })), "{again:?}");
    assert_eq!(counting.reads(), before, "no chain read for a stale envelope");
    let after = rec_of(&svc_b, s.org_id);
    assert_eq!((after.epoch, after.root_hash, after.last_seq), (held.epoch, held.root_hash, held.last_seq));
}

// Abnormal: a Change set built on another base is refused with no chain read.
// verifies: REQ-f2k4tr, LLR-9ew26y
#[tokio::test(flavor = "multi_thread")]
async fn a_change_set_on_another_base_is_refused_without_a_chain_read() {
    let (s, counting) = admitted("other-base").await;
    let mark = rec_of(&s.svc_b, s.org_id).last_seq;
    let envelope = Envelope::build(s.org_id, SequenceNumber::new(mark.get() + 1), &foreign_delta()).unwrap();
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(b_addr, &WireMessage { envelope, org_secret: None, genesis_snapshot: None }).await;
    let (_svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::DeltaBaseMismatch);
    assert_eq!(counting.reads(), before);
}

// Abnormal: Change set bytes that do not decode are refused with no chain read.
// verifies: REQ-f2k4tr, LLR-9ew26y
#[tokio::test(flavor = "multi_thread")]
async fn undecodable_change_set_bytes_are_refused_without_a_chain_read() {
    let (s, counting) = admitted("undecodable").await;
    let mark = rec_of(&s.svc_b, s.org_id).last_seq;
    let envelope = Envelope { org_id: s.org_id, parent_seq: SequenceNumber::new(mark.get() + 1), delta_bytes: vec![0xff; 16] };
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(b_addr, &WireMessage { envelope, org_secret: None, genesis_snapshot: None }).await;
    let (_svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::MalformedDelta);
    assert_eq!(counting.reads(), before);
}

// Abnormal: the self-delete path runs the same checks first.
// verifies: LLR-9ew26y
#[tokio::test(flavor = "multi_thread")]
async fn the_self_delete_path_refuses_a_stale_envelope_without_a_chain_read() {
    let (mut s, counting) = admitted("sd-stale").await;
    let msg = captured_admission_of_c(&mut s).await;
    let (b_addr, b_task) = spawn_self_delete(s.svc_b, &s.b_device_kp).await;
    deliver(b_addr, &msg).await;
    let (svc_b, first) = b_task.await.unwrap();
    first.unwrap();
    let before = counting.reads();
    let (b_addr, b_task) = spawn_self_delete(svc_b, &s.b_device_kp).await;
    deliver(b_addr, &msg).await;
    let (_svc_b, again) = b_task.await.unwrap();
    assert!(matches!(again, Err(OrgNodeError::StaleSeq { .. })), "{again:?}");
    assert_eq!(counting.reads(), before);
}

// Abnormal: an Organisation the node holds no record of is refused on the
// self-delete path before any chain read, and nothing is written.
// verifies: LLR-379hnv
#[tokio::test(flavor = "multi_thread")]
async fn the_self_delete_path_refuses_an_unheld_organisation_without_a_chain_read() {
    let (s, counting) = admitted("sd-unheld").await;
    let envelope = Envelope::build(OrgId::new([0xee; 20]), SequenceNumber::new(1), &foreign_delta()).unwrap();
    let on_disk = store_bytes("sd-unheld", "b");
    let before = counting.reads();
    let (b_addr, b_task) = spawn_self_delete(s.svc_b, &s.b_device_kp).await;
    deliver(b_addr, &WireMessage { envelope, org_secret: None, genesis_snapshot: None }).await;
    let (_svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::OrgNotOnChain);
    assert_eq!(counting.reads(), before);
    assert_eq!(store_bytes("sd-unheld", "b"), on_disk, "nothing written");
}

// Abnormal: for a held Organisation the chain no longer knows, the refusal
// comes only after the chain-free checks, from the one chain read.
// verifies: REQ-bvh8v6, LLR-3wb7th
#[tokio::test(flavor = "multi_thread")]
async fn a_held_organisation_absent_from_the_chain_is_refused_after_the_chain_free_checks() {
    let (mut s, counting) = admitted("held-absent").await;
    let msg = captured_admission_of_c(&mut s).await;
    counting.hide(s.org_id);
    let held = rec_of(&s.svc_b, s.org_id);
    // A chain-free failure is reported as itself, not as the absence.
    let garbled = WireMessage { envelope: Envelope { delta_bytes: vec![0xff; 16], ..msg.envelope.clone() }, ..msg.clone() };
    let before = counting.reads();
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(b_addr, &garbled).await;
    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::MalformedDelta);
    assert_eq!(counting.reads(), before);
    // An envelope that passes them meets the absence on the one read.
    let (b_addr, b_task) = spawn_receive(svc_b, &s.b_device_kp).await;
    deliver(b_addr, &msg).await;
    let (svc_b, result) = b_task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::OrgNotOnChain);
    assert_eq!(counting.reads() - before, 1);
    assert_eq!(rec_of(&svc_b, s.org_id).last_seq, held.last_seq);
}
```

`org-members` is already a dev-dependency of org-node (the `Delta` path
above); if `cargo check` says otherwise, import `org_node::test_fixtures`'s
`Delta` re-export instead.

Add to `org-node/Cargo.toml`:

```toml
# The receive paths read the chain only after the chain-free checks
# (LLR-9ew26y), change worktree-org-node-chain-authority.
[[test]]
name = "receive_chain_reads"
path = "tests/receive_chain_reads.rs"
required-features = ["app", "test-support"]
```

and append ` --test receive_chain_reads` to the org-node cargo line in
`org-node/.guardrails/config.yaml`, with a dated comment line above
`verify_commands:` ("2026-10-06, T5 of docs/plans/2026-10-05-chain-authority.md:
`receive_chain_reads` counts the receive paths' chain reads").

`admission_sender.rs`, `a_message_for_an_organisation_absent_from_the_chain_is_refused`
(REQ-bvh8v6, LLR-3wb7th): the message must carry the snapshot its Change set
extends, so that the first-admission path decodes it, passes the chain-free
checks and only then learns from the chain that the Organisation is absent.
Replace its message construction with:

```rust
    let stranger = org_node::MemberSeed::from([0x5au8; 32]).x25519_keypair();
    let base = org_node::test_fixtures::genesis_trie(&stranger, &org_node::test_fixtures::admin_device());
    let (delta, _) = org_node::test_fixtures::admit_member_delta(&stranger);
    let absent_org = OrgId::new([0xeeu8; 20]);
    let envelope = org_node::Envelope::build(absent_org, org_node::SequenceNumber::new(1), &delta).unwrap();
    let msg = WireMessage { envelope, org_secret: None, genesis_snapshot: Some(snapshot_bytes(&base)) };
```

and its comment with "A first admission whose snapshot and Change set are
consistent, for an Organisation the chain does not know: refused with
`OrgNotOnChain` after the chain-free checks (LLR-3wb7th)". Today's code reads
the chain first, so it is green before and after; its order is what
`a_held_organisation_absent_from_the_chain_is_refused_after_the_chain_free_checks`
reddens.

**Step 3 — run, expect red.** The org-node line with `--test
receive_chain_reads` added: compile error E0432 `no check_chain_free in
verify` (verify_against_chain). Comment out the three new verify tests
temporarily, run `receive_chain_reads`, record: `a_stale_envelope_…`,
`a_change_set_on_another_base…`, `undecodable_change_set_bytes…`,
`the_self_delete_path_refuses_a_stale…` FAILED (reads advanced by 1 — today's
early `read_state`), `the_self_delete_path_refuses_an_unheld…` FAILED for the
same reason, and `a_held_organisation_absent…` FAILED at its first assertion
(today's early read answers `OrgNotOnChain` where the chain-free decode should
have answered `MalformedDelta`). `an_update_that_passes_the_chain_free_checks_reads_the_chain_once`
passes before and after (today's path also reads once); it is the normal case
the others are measured against. Restore the commented tests.

**Step 4 — implement.** `org-node/src/verify.rs`:

```rust
use org_members::delta::Delta;

/// The checks that need no chain, in this order: the Organisation, the
/// Sequence number, the Change set decode, the base root. Returns the decoded
/// Change set (LLR-fuq379).
fn chain_free(local_trie: &Trie, envelope: &Envelope, ctx: &VerifyContext) -> Result<Delta, OrgNodeError> {
    if envelope.org_id != ctx.expected_org_id {
        return Err(OrgNodeError::OrgIdMismatch);
    }
    ctx.seq_guard.check(envelope.parent_seq)?;
    let delta = envelope.decode_delta()?;
    if delta.base_root() != &local_trie.root_hash()? {
        return Err(OrgNodeError::DeltaBaseMismatch);
    }
    Ok(delta)
}

/// The chain-free half of verify-against-chain, so a caller can refuse a
/// stranger's Envelope before it reads the chain (LLR-fuq379, RC-mj6gjq).
/// Touches no `ChainReader`.
pub fn check_chain_free(local_trie: &Trie, envelope: &Envelope, ctx: &VerifyContext) -> Result<(), OrgNodeError> {
    chain_free(local_trie, envelope, ctx).map(|_| ())
}
```

and in `verify_envelope_against_chain` replace steps 1–4 with
`let delta = chain_free(local_trie, envelope, ctx)?;` (apply, chain read,
epoch, Sequence-number-equals-epoch and root match follow unchanged; the chain
read happens once).

`org-node/src/service.rs`, `receive_and_verify` — between `recv_one` and the
commit, the order becomes:

```rust
        let org_id = msg.envelope.org_id;
        let existing = self.store.data().orgs.iter().find(|o| o.org_id == org_id).cloned();
        let is_first_admission = existing.is_none();
        let (local_trie, last_seq, last_epoch) = match &existing {
            Some(rec) => (trie_from_snapshots(&rec.trie_members)?, rec.last_seq, rec.epoch),
            // The record a first admission extends: from the snapshot it carries.
            None => (first_admission_base(msg.genesis_snapshot.as_deref())?, SequenceNumber::new(0), Epoch::new(0)),
        };
        let ctx = VerifyContext {
            expected_org_id: org_id,
            seq_guard: SeqGuard::from_last_seen(last_seq),
            last_committed_epoch: last_epoch,
        };
        // Every check that needs no chain, before the one chain read (LLR-9ew26y).
        crate::verify::check_chain_free(&local_trie, &msg.envelope, &ctx)?;
        let chain_state = self.chain.read_state(org_id).await?.ok_or(OrgNodeError::OrgNotOnChain)?;
        // Until T8 removes it: the record's administrator key, from the
        // imported Invite (LLR-rys5nx) or the chain's key just read (LLR-xq9nrq).
        let admin_member_key = match &existing {
            Some(rec) => rec.admin_member_key,
            None => match self.store.data().pending_invites.iter().find(|p| p.org_id == org_id) {
                Some(invite) => invite.admin_member_key,
                None => PersonPublicKey::parse(chain_state.org_pub_key.as_bytes())?,
            },
        };
        let verified =
            verify_envelope_against_chain(&local_trie, &msg.envelope, &ctx, &ChainOpsReader { state: chain_state })?;
```

replacing the early `read_state` and the old `existing`/`match` block; the
commit below is unchanged. `receive_and_self_delete_if_revoked`:

```rust
        let org_id = msg.envelope.org_id;
        // An Organisation not held is refused before any chain read (LLR-379hnv).
        let existing = self
            .store
            .data()
            .orgs
            .iter()
            .find(|o| o.org_id == org_id)
            .cloned()
            .ok_or(OrgNodeError::OrgNotOnChain)?;
        let local_trie = trie_from_snapshots(&existing.trie_members)?;
        let ctx = VerifyContext {
            expected_org_id: org_id,
            seq_guard: SeqGuard::from_last_seen(existing.last_seq),
            last_committed_epoch: existing.epoch,
        };
        crate::verify::check_chain_free(&local_trie, &msg.envelope, &ctx)?;
        let chain_state = self.chain.read_state(org_id).await?.ok_or(OrgNodeError::OrgNotOnChain)?;
        let verified =
            verify_envelope_against_chain(&local_trie, &msg.envelope, &ctx, &ChainOpsReader { state: chain_state })?;
```

**Step 5 — green.** The org-node line (with `receive_chain_reads`):
receive_chain_reads 7 passed, verify_against_chain 3 more than before, every
other target as before. `cargo check --all-targets` as in Environment.
Commit: `T5: chain-free checks before the chain read`.

**Red→green attestations:** (DONE 2026-10-06, merged; org-node cargo line
196 passed, 0 failed; app `cargo check` clean.)

- check_chain_free_passes_an_honest_envelope — compile-red (E0432 `org_node::verify::check_chain_free`); runtime-red against a stub returning `Err(Chain("stub"))`; green with the real body
- check_chain_free_refuses_each_check_in_order — compile-red (same); runtime-red against the stub (`Err(Chain("stub"))` ≠ `Err(OrgIdMismatch)`); green
- verify_reads_the_chain_once_and_only_after_the_chain_free_checks — compile-red (same target); runtime-red under a temporary mutation adding a chain read at the top of `verify_envelope_against_chain` (reads 1 ≠ 0); green with the mutation removed and `chain_free` in place
- an_update_that_passes_the_chain_free_checks_reads_the_chain_once — runtime-red at the intermediate step (new order with the old early `read_state` still present: reads 2 ≠ 1); green once the early read was removed
- a_stale_envelope_is_refused_without_a_chain_read — runtime-red on the old code (reads 3 ≠ 2)
- a_change_set_on_another_base_is_refused_without_a_chain_read — runtime-red on the old code (reads 2 ≠ 1)
- undecodable_change_set_bytes_are_refused_without_a_chain_read — runtime-red on the old code (reads 2 ≠ 1)
- the_self_delete_path_refuses_a_stale_envelope_without_a_chain_read — runtime-red on the old code (reads 3 ≠ 2)
- the_self_delete_path_refuses_an_unheld_organisation_without_a_chain_read — runtime-red on the old code (reads 2 ≠ 1)
- a_held_organisation_absent_from_the_chain_is_refused_after_the_chain_free_checks — runtime-red on the old code (`OrgNotOnChain` ≠ `MalformedDelta`)
- a_message_for_an_organisation_absent_from_the_chain_is_refused (rewritten) — the old message form is runtime-red under the new code (`Chain("first admission without a record snapshot")` ≠ `OrgNotOnChain`); rewritten to carry the snapshot, green

---

### T6 — org-node: expected admissions, the invite identifier, and the own-Persona rule

**Files touched:** `org-node/src/types.rs`, `org-node/src/lib.rs`,
`org-node/src/transport/wire.rs`, `org-node/src/store.rs`,
`org-node/src/error.rs`, `org-node/src/service.rs`,
`org-node/tests/support/mod.rs`, `org-node/tests/expected_admission.rs` (new),
`org-node/tests/encoding_golden.rs`, `org-node/tests/wire_frame_bound.rs`,
`org-node/tests/value_types.rs`, `org-node/tests/admission_sender.rs`,
`org-node/tests/receive_chain_reads.rs`, `org-node/tests/organisation_key.rs`,
`org-node/tests/service_stories.rs`, `org-node/tests/persona_records.rs`,
`org-node/tests/secret_redaction.rs`, `org-node/tests/store_at_rest.rs`,
`org-node/tests/transport_handshake.rs`, `org-node/tests/transport_networked.rs`,
`org-node/tests/fuzz_envelope_decode/fuzz_target.rs`,
`org-node/tests/fuzz_verify_against_chain/fuzz_target.rs`,
`org-node/Cargo.toml`, `org-node/.guardrails/config.yaml`
**Parallel:** no (serial, after T5)
**IDs verified:** REQ-8amu2a (RC-2ferct), REQ-kt877x, REQ-xa6smf,
LLR-s8xp7m, LLR-3f5h7b, LLR-ms8njy, LLR-9zfnmb, LLR-q8emds, LLR-mbjfq8,
LLR-j83kc8, LLR-y2v8v2, LLR-95753m (expectations), LLR-mxskg9
(`AdmissionNotExpected`, `AdmissionNotOurs`), LLR-j6j95z, LLR-ayrdr8 (store,
wire).
**Size:** ~520 lines (about 120 of them mechanical: the new Wire message field
in every test literal).

The owner's ruling of 2026-10-06 (REQ-8amu2a as amended, REQ-kt877x): an
expectation names the Organisation and the invite identifier; the admission's
Wire message carries the identifier; a first admission is read against the
chain only if both match an expectation, and committed only if the verified
record lists one of the node's own Personas; refusals keep every expectation;
the matching expectation is cleared when its admission commits. Until T10
introduces `send_update`, the old `admit_member` is what puts the identifier
on the wire (Decision 19).

**Step 1 — re-pin the store and wire goldens FIRST (red against today's
code).** In `org-node/tests/encoding_golden.rs`:

*Store.* `StoreData` gains `expected_admissions: Vec<ExpectedAdmission>` as
its last field, `ExpectedAdmission { org_id: OrgId, invite_id: InviteId }`.
Derivation from master's `GOLDEN_STORE` (676 bytes) by the format rule:
postcard appends the new field after `pending_invites`; a `Vec` of one is `01`
followed by the element, an `OrgId` is its 20 bytes and an `InviteId` its 32
bytes, with no length prefix (both are fixed arrays). The fixed store gains
one expectation, `OrgId([0xcc; 20])` with `InviteId([0xee; 32])`, so the new
value is master's followed by `01`, forty `c` and sixty-four `e` (729 bytes):

```rust
const GOLDEN_STORE: &str = "0107702d616c69636501aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa05616c69636505416c69636505536d697468111111111111111111111111111111111111111111111111111111111111111112121212121212121212121212121212121212121212121212121212121212120101010101010101010101010101010101010101010101010101010101010101010101aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa3333333333333333333333333333333333333333333333333333333333333333d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c97787370301444444444444444444444444444444444444444444444444444444444444444402d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873702010101010101010101010101010101010101010101010101010101010101010105616c69636505416c69636505536d697468d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21020202020202020202020202020202020202020202020202020202020202020203626f6203426f62054a6f6e6573884b8857f4eaa1613c61504db34d4beaf346517a0e31de3cddd4d9b4201d9d0b01a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f00155555555555555555555555555555555555555555555555555555555555555550001bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701cccccccccccccccccccccccccccccccccccccccceeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
```

*Wire.* `WireMessage` gains `invite_id: Option<InviteId>` after
`genesis_snapshot` (LLR-ms8njy). The pinned admission gains `Some([0xee;
32])`: master's `GOLDEN_WIRE` (317 bytes) followed by `01` and sixty-four `e`
(350 bytes):

```rust
const GOLDEN_WIRE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa029101b642ec203b364f4807ea74b0a63ab680c1c94d38b23246b1042f520394477e860001020202020202020202020202020202020202020202020202020202020202020203626f628139770ea87d175f56a35466c34c7ecccb8d8a91b4ee37a25df60f5b8fc9b3940454657374045573657201ed4928c628d1c2c6eae90338905995612959273a5c63f93636c14614ac8737d101444444444444444444444444444444444444444444444444444444444444444401720101010101010101010101010101010101010101010101010101010101010101010561646d696e04546573740455736572d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701ca93ac1705187071d67b83c7ff0efe8108e8ec4530575d7726879333dbdabe7c01eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
```

Keep master's two values in the file's comment block beside the new ones
(the pattern master's merge note uses), and add a dated paragraph: "Re-pinned
2026-10-06 (change worktree-org-node-chain-authority, T6): the store gains
`expected_admissions` (`01` ‖ `cc`×20 ‖ `ee`×32 appended) and the Wire message
`invite_id` (`01` ‖ `ee`×32 appended); derived by the format rule from the
values above." In `persona_store_plaintext_is_pinned` add
`assert_eq!(m["expected_admissions"][0]["org_id"][0], json!(0xcc));` and
`assert_eq!(m["expected_admissions"][0]["invite_id"][31], json!(0xee));`; in
`admission_wire_message_is_pinned` add
`assert_eq!(m["invite_id"][0], json!(0xee));` and `LLR-ms8njy` to its
annotation. Run `--test encoding_golden` → both FAILED (the round trip drops
the trailing bytes: the left/right hex differ by the suffix). Record.

**Step 2 — failing tests.**

`org-node/tests/support/mod.rs`:

```rust
use org_node::InviteId;

/// The invite identifier a story's joiner replies under: its device key's
/// bytes (Decision 18), so no story helper needs another argument.
pub fn invite_for(device: &DevicePublicKey) -> InviteId {
    InviteId::new(*device.as_bytes())
}

/// The device key of `svc`'s first Persona.
pub fn first_device(svc: &OrgService) -> DevicePublicKey {
    svc.list_personas()[0].device_seed.signing_keypair().device_key().unwrap()
}
```

`prepare_to_join` keeps its invite import (T7 removes it) and gains, last,
`svc_b.expect_admission(&mut OsRng, org_id, invite_for(&first_device(svc_b))).expect("expect the admission");`;
`admit` passes `Some(invite_for(&joiner_device(joiner)))` as the new last
argument of `admit_member`.

Create `org-node/tests/expected_admission.rs`:

```rust
#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Expected admissions (REQ-8amu2a, RC-2ferct): a first admission is read
//! against the chain only when its Organisation and invite identifier match an
//! expectation the app declared, and committed only when the verified record
//! lists one of this node's Personas (REQ-kt877x).

mod support;

use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::service::{MockChainOps, OrgService};
use org_node::store::ExpectedAdmission;
use org_node::test_fixtures::{admin_device, genesis_trie};
use org_node::transport::wire::WireMessage;
use org_node::{InviteId, MemberSeed};
use rand::rngs::OsRng;
use support::*;

/// A second joining node that expects nothing yet, reading through `counting`.
fn second_joiner(tag: &str, counting: &CountingChain) -> (OrgService, org_node::keys::SigningKeypair, JoinerOf) {
    let mut svc = OrgService::new(open_store(tag, "b2", "pw_b2"), Box::new(counting.clone()));
    let pid = svc.create_persona(&mut OsRng, h("bea"), nm("Bea"), sn("Second")).unwrap();
    let joiner = joiner_of(&svc, &pid);
    let kp = device_kp(&svc, &pid);
    (svc, kp, joiner)
}

/// A's genuine admission of `joiner`, captured rather than delivered.
async fn captured_admission(s: &mut Setup, joiner: &JoinerOf) -> WireMessage {
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, joiner, sink_addr, org_secret()).await.unwrap();
    sink.await.unwrap().2
}

fn expectation(org_id: OrgId, invite_id: InviteId) -> ExpectedAdmission {
    ExpectedAdmission { org_id, invite_id }
}

// Normal: an expected first admission commits and its expectation is
// cleared, in memory and on disk.
// verifies: REQ-8amu2a, LLR-s8xp7m, LLR-q8emds, LLR-mbjfq8
#[tokio::test(flavor = "multi_thread")]
async fn an_expected_first_admission_is_committed_and_clears_the_expectation() {
    let s = setup("expected").await;
    let id = invite_for(&s.joiner_b.device_key);
    assert_eq!(s.svc_b.expected_admissions(), &[expectation(s.org_id, id)]);
    let s = admit_b_directly(s).await;
    assert!(s.svc_b.expected_admissions().is_empty());
    assert!(reopen_store("expected", "b", "pw_b").data().expected_admissions.is_empty());
}

// Abnormal: an unexpected first admission — no expectation at all — is refused
// before its snapshot is decoded and without a chain read, and nothing is
// written.
// verifies: REQ-8amu2a, LLR-s8xp7m
#[tokio::test(flavor = "multi_thread")]
async fn an_unexpected_first_admission_is_refused_before_the_snapshot_or_the_chain() {
    let (mut s, counting) = setup_counted("unexpected").await;
    let (mut svc_b2, kp, joiner) = second_joiner("unexpected", &counting);
    let msg = captured_admission(&mut s, &joiner).await;
    let garbled = WireMessage { genesis_snapshot: Some(vec![0xff; 4]), ..msg.clone() };
    let on_disk = store_bytes("unexpected", "b2");
    let before = counting.reads();
    for m in [&msg, &garbled] {
        let (addr, task) = spawn_receive(svc_b2, &kp).await;
        deliver(addr, m).await;
        let (back, result) = task.await.unwrap();
        svc_b2 = back;
        assert_eq!(result.unwrap_err(), OrgNodeError::AdmissionNotExpected { org_id: s.org_id });
    }
    assert_eq!(counting.reads(), before, "no chain read");
    assert!(svc_b2.list_orgs().is_empty());
    assert_eq!(store_bytes("unexpected", "b2"), on_disk, "nothing written");
}

// Abnormal: an expectation names one Organisation and admits no other.
// verifies: REQ-8amu2a, LLR-s8xp7m, LLR-y2v8v2
#[tokio::test(flavor = "multi_thread")]
async fn an_expectation_for_one_organisation_admits_no_other() {
    let (mut s, counting) = setup_counted("other-org").await;
    let (mut svc_b2, kp, joiner) = second_joiner("other-org", &counting);
    let elsewhere = OrgId::new([0x42; 20]);
    let id = invite_for(&joiner.device_key);
    svc_b2.expect_admission(&mut OsRng, elsewhere, id).unwrap();
    let msg = captured_admission(&mut s, &joiner).await;
    let (addr, task) = spawn_receive(svc_b2, &kp).await;
    deliver(addr, &msg).await;
    let (svc_b2, result) = task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::AdmissionNotExpected { org_id: s.org_id });
    assert_eq!(svc_b2.expected_admissions(), &[expectation(elsewhere, id)], "the other expectation is untouched");
}

// Abnormal (pre-emption): for the right Organisation, a first admission under
// another invite identifier, or under none, is refused without a chain read
// and leaves the expectation; the genuine one then commits.
// verifies: REQ-8amu2a, LLR-s8xp7m, LLR-ms8njy
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_under_another_invite_id_is_refused_and_keeps_the_expectation() {
    let (mut s, counting) = setup_counted("other-id").await;
    let joiner = s.joiner_b.clone();
    let msg = captured_admission(&mut s, &joiner).await;
    let before = counting.reads();
    let mut svc_b = s.svc_b;
    for invite_id in [Some(InviteId::new([0x13; 32])), None] {
        let (addr, task) = spawn_receive(svc_b, &s.b_device_kp).await;
        deliver(addr, &WireMessage { invite_id, ..msg.clone() }).await;
        let (back, result) = task.await.unwrap();
        svc_b = back;
        assert_eq!(result.unwrap_err(), OrgNodeError::AdmissionNotExpected { org_id: s.org_id });
    }
    assert_eq!(counting.reads(), before, "no chain read");
    assert_eq!(svc_b.expected_admissions(), &[expectation(s.org_id, invite_for(&joiner.device_key))]);
    let (addr, task) = spawn_receive(svc_b, &s.b_device_kp).await;
    deliver(addr, &msg).await;
    let (svc_b, result) = task.await.unwrap();
    result.expect("the genuine admission still commits");
    assert!(svc_b.expected_admissions().is_empty());
}

// Abnormal (pre-emption, REQ-kt877x): a genuine admission of someone else,
// relayed under this node's invite identifier, verifies against the chain
// but lists none of this node's Personas: refused, nothing created, nothing
// written, the expectation kept; this node's own admission then commits.
// verifies: REQ-kt877x, LLR-3f5h7b, LLR-mxskg9
#[tokio::test(flavor = "multi_thread")]
async fn a_first_admission_that_lists_none_of_our_personas_is_refused() {
    let (mut s, counting) = setup_counted("not-ours").await;
    let (mut svc_b2, kp, joiner_b2) = second_joiner("not-ours", &counting);
    let id_b2 = invite_for(&joiner_b2.device_key);
    svc_b2.expect_admission(&mut OsRng, s.org_id, id_b2).unwrap();
    // A admits B (epoch 2); the relay re-labels B's admission with B2's id.
    let joiner_b = s.joiner_b.clone();
    let of_b = captured_admission(&mut s, &joiner_b).await;
    let on_disk = store_bytes("not-ours", "b2");
    let (addr, task) = spawn_receive(svc_b2, &kp).await;
    deliver(addr, &WireMessage { invite_id: Some(id_b2), ..of_b }).await;
    let (back, result) = task.await.unwrap();
    svc_b2 = back;
    assert_eq!(result.unwrap_err(), OrgNodeError::AdmissionNotOurs { org_id: s.org_id });
    assert!(svc_b2.list_orgs().is_empty(), "no record of an Organisation it is not in");
    assert_eq!(svc_b2.list_personas()[0].status, org_node::store::PersonaStatus::Proposed);
    assert_eq!(svc_b2.expected_admissions(), &[expectation(s.org_id, id_b2)]);
    assert_eq!(store_bytes("not-ours", "b2"), on_disk, "nothing written");
    // Normal: B2's own admission (epoch 3) commits.
    let (addr, task) = spawn_receive(svc_b2, &kp).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &joiner_b2, addr, org_secret()).await.unwrap();
    let (svc_b2, result) = task.await.unwrap();
    assert_eq!(result.unwrap().epoch, org_node::Epoch::new(3));
    assert!(svc_b2.expected_admissions().is_empty());
}

// Normal and abnormal: a declaration is recorded once however often it is
// made, a pair differing in either part is a second entry, and each reaches
// the disk before `expect_admission` returns.
// verifies: REQ-8amu2a, LLR-9zfnmb, LLR-95753m
#[test]
fn expect_admission_records_a_pair_once_and_reaches_the_disk() {
    let mut svc = OrgService::new(open_store("once", "b", "pw_b"), Box::new(MockChainOps::new()));
    let (org, other) = (OrgId::new([0x42; 20]), OrgId::new([0x43; 20]));
    let (x, y) = (InviteId::new([1; 32]), InviteId::new([2; 32]));
    svc.expect_admission(&mut OsRng, org, x).unwrap();
    svc.expect_admission(&mut OsRng, org, x).unwrap();
    assert_eq!(svc.expected_admissions(), &[expectation(org, x)]);
    assert_eq!(reopen_store("once", "b", "pw_b").data().expected_admissions, vec![expectation(org, x)]);
    svc.expect_admission(&mut OsRng, org, y).unwrap();
    svc.expect_admission(&mut OsRng, other, x).unwrap();
    assert_eq!(
        reopen_store("once", "b", "pw_b").data().expected_admissions,
        vec![expectation(org, x), expectation(org, y), expectation(other, x)]
    );
}

// Abnormal: a first admission refused after the expectation check leaves the
// expectation in place.
// verifies: REQ-8amu2a, LLR-q8emds
#[tokio::test(flavor = "multi_thread")]
async fn a_refused_first_admission_leaves_the_expectation() {
    let mut s = setup("refused-keeps").await;
    let joiner = s.joiner_b.clone();
    let msg = captured_admission(&mut s, &joiner).await;
    let stranger = MemberSeed::from([0x5a; 32]).x25519_keypair();
    let wrong_base = snapshot_bytes(&genesis_trie(&stranger, &admin_device()));
    let (addr, task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(addr, &WireMessage { genesis_snapshot: Some(wrong_base), ..msg }).await;
    let (svc_b, result) = task.await.unwrap();
    assert_eq!(result.unwrap_err(), OrgNodeError::DeltaBaseMismatch);
    assert_eq!(svc_b.expected_admissions(), &[expectation(s.org_id, invite_for(&joiner.device_key))]);
    assert!(svc_b.list_orgs().is_empty());
}

// Normal: the commit clears only the expectation it matched; another for the
// same Organisation (a second Invite) stays.
// verifies: REQ-8amu2a, LLR-q8emds
#[tokio::test(flavor = "multi_thread")]
async fn a_second_expectation_for_the_same_organisation_survives_the_commit() {
    let mut s = setup("two-ids").await;
    let spare = InviteId::new([0x77; 32]);
    s.svc_b.expect_admission(&mut OsRng, s.org_id, spare).unwrap();
    let s = admit_b_directly(s).await;
    assert_eq!(s.svc_b.expected_admissions(), &[expectation(s.org_id, spare)]);
}

// Abnormal: an expected first admission decodes its snapshot — and refuses
// one it lacks or cannot decode — before it reads the chain.
// verifies: REQ-8amu2a, LLR-s8xp7m, LLR-j6j95z
#[tokio::test(flavor = "multi_thread")]
async fn an_expected_first_admission_decodes_its_snapshot_before_reading_the_chain() {
    let (mut s, counting) = setup_counted("decode-first").await;
    let joiner = s.joiner_b.clone();
    let msg = captured_admission(&mut s, &joiner).await;
    let before = counting.reads();
    let mut svc_b = s.svc_b;
    for snapshot in [Some(vec![0xff; 4]), None] {
        let (addr, task) = spawn_receive(svc_b, &s.b_device_kp).await;
        deliver(addr, &WireMessage { genesis_snapshot: snapshot, ..msg.clone() }).await;
        let (back, result) = task.await.unwrap();
        svc_b = back;
        assert!(matches!(result, Err(OrgNodeError::Chain(_))), "{result:?}");
    }
    assert_eq!(counting.reads(), before, "no chain read before the snapshot decodes");
    assert_eq!(svc_b.expected_admissions(), &[expectation(s.org_id, invite_for(&joiner.device_key))]);
}
```

Add the `[[test]]` entry (name `expected_admission`, path
`tests/expected_admission.rs`, `required-features = ["app", "test-support"]`)
to `org-node/Cargo.toml`, and ` --test expected_admission` to the org-node
cargo line.

`wire_frame_bound.rs` — add (LLR-ms8njy, both sides):

```rust
// Normal: a frame carries its invite identifier, or its absence, unchanged.
// Abnormal: a body whose invite identifier is cut short does not decode.
// verifies: LLR-ms8njy, REQ-8amu2a
#[test]
fn a_wire_message_carries_its_invite_id_and_refuses_a_short_one() {
    for invite_id in [Some(org_node::InviteId::new([0x5c; 32])), None] {
        let msg = WireMessage { invite_id, ..small_message() };
        let framed = encode_frame(&msg).unwrap();
        assert_eq!(decode_body(&framed[4..]).unwrap(), msg);
    }
    let with_id = WireMessage { invite_id: Some(org_node::InviteId::new([0x5c; 32])), ..small_message() };
    let framed = encode_frame(&with_id).unwrap();
    let cut = &framed[4..framed.len() - 1];
    assert!(matches!(decode_body(cut), Err(TransportError::Malformed)));
}
```

(`small_message()` is the name of the file's existing in-bound message
builder; if it inlines one, extract it first with no behaviour change.)

`value_types.rs`: add `OrgNodeError::AdmissionNotExpected { org_id: OrgId::new([1; 20]) }`
and `OrgNodeError::AdmissionNotOurs { org_id: OrgId::new([1; 20]) }` to the
distinct list and `LLR-mxskg9` to its annotation, and add:

```rust
// Normal: each first-admission refusal names its Organisation; abnormal: two
// Organisations' refusals, and the two refusals of one, are not the same error.
// verifies: LLR-mxskg9, REQ-8amu2a, REQ-kt877x
#[test]
fn the_first_admission_refusals_name_their_organisation() {
    let org = OrgId::new([0xa0; 20]);
    let unexpected = OrgNodeError::AdmissionNotExpected { org_id: org };
    let not_ours = OrgNodeError::AdmissionNotOurs { org_id: org };
    for err in [&unexpected, &not_ours] {
        assert!(format!("{err}").contains(&format!("{org:?}")), "{err}");
    }
    assert_ne!(unexpected, OrgNodeError::AdmissionNotExpected { org_id: OrgId::new([0xa1; 20]) });
    assert_ne!(unexpected, not_ours);
}
```

**Every Wire message a test builds** gains `invite_id: None` (a literal with
three fields no longer compiles) — `receive_chain_reads.rs`,
`admission_sender.rs`, `service_stories.rs`, `wire_frame_bound.rs`,
`transport_handshake.rs`, `transport_networked.rs`, `secret_redaction.rs`, the
two fuzz targets — **except** a message that is a first admission to a node
expecting it, which carries `invite_id: Some(invite_for(&<joiner's device
key>))`. The fuzz target of `decode_body` needs no change beyond the literal:
arbitrary bytes now also exercise the new field.

**Every `StoreData { … }` literal** (`persona_records.rs`,
`secret_redaction.rs`, `store_at_rest.rs`) gains `expected_admissions:
vec![]`; in `persona_records.rs` the `WireStore` mirror gains
`expected_admissions: Vec<([u8; 20], [u8; 32])>` as its last field, and
`a_store_with_every_record_kind_opens_with_every_field_parsed` puts one
(`([0x77; 20], [0x78; 32])`) in it and asserts it reads back as
`ExpectedAdmission { org_id: OrgId::new([0x77; 20]), invite_id: InviteId::new([0x78; 32]) }`.

`admission_sender.rs`:

| Test | Change in T6 |
|---|---|
| `a_first_admission_with_no_imported_invite_rests_on_the_chain_alone` | rename `a_first_admission_to_an_expected_organisation_rests_on_the_chain_alone`; B0 (no invite, `setup_with(…, false)`) calls `svc_b.expect_admission(&mut OsRng, org_id, invite_for(&s.joiner_b.device_key))`; keep the "no invite" assertion until T7; `// verifies: REQ-xa6smf, REQ-8amu2a, LLR-mbjfq8`. Red today: no `expect_admission`. |
| `a_first_admission_with_no_imported_invite_that_misses_the_chain_root_commits_nothing` | B declares the expectation as above; add `assert_eq!(svc_b.expected_admissions().len(), 1, "a refusal keeps the expectation")`; annotation unchanged (REQ-xa6smf, LLR-mbjfq8). |
| `a_first_admission_with_no_imported_invite_records_the_chains_key_as_admin_member_key` | B declares the expectation; nothing else in T6 (T8 rewrites it). |
| `a_first_admission_whose_invite_names_another_organisation_key_is_committed` | B declares the expectation next to its tampered `import_invite`; nothing else in T6 (T7 removes the invite). |
| `first_admission_from_a_device_other_than_the_invites_admin_is_committed` | add `assert!(svc_b.expected_admissions().is_empty(), "the expectation is cleared on commit")`; annotation gains `LLR-j83kc8` if absent (REQ-xa6smf, LLR-j83kc8). |
| `a_committed_admission_reaches_the_disk_and_consumes_the_invite` | add `assert!(reloaded.data().expected_admissions.is_empty(), "the expectation is cleared on commit")`. |
| `a_first_admission_that_fails_verification_leaves_the_invite_pending` | add `assert_eq!(s.svc_b.expected_admissions().len(), 1);` after the refusal. |
| `two_org_receiver` and `a_receiver_holding_two_organisations_commits_into_the_one_the_change_names` | after each `import_invite` add `svc_b.expect_admission(&mut OsRng, org_N, invite_for(&<the device admitted into org_N>)).unwrap();`; next to each `list_pending_invites()` assertion add the same over `expected_admissions()` (`[0].org_id == org_2`). |
| `the_self_delete_path_refuses_an_organisation_it_holds_no_record_of` | add `assert_eq!(svc_b.expected_admissions().len(), 1, "the expectation is not cleared");`. |
| `first_admission_without_a_record_snapshot_is_refused` | its hand-built Wire message carries `invite_id: Some(invite_for(&s.joiner_b.device_key))`, so the refusal it asserts (missing snapshot) is still the one reached; no other change. |
| `a_message_for_an_organisation_absent_from_the_chain_is_refused` | B declares `expect_admission(&mut OsRng, absent_org, InviteId::new([0x31; 32]))` before `spawn_receive`; the message carries `invite_id: Some(InviteId::new([0x31; 32]))`. |
| `pr_mdv38y_the_receive_path_rebinds_another_organisations_persona`, `pr_8qsnhx_a_second_organisations_admission_goes_out_under_the_first_personas_key`, `pr_xwek5e_another_members_revocation_clears_the_receivers_secret`, `a_second_organisation_is_admitted_into_without_touching_the_first`, every test that builds its own joining service | add `expect_admission(&mut OsRng, <org>, invite_for(&<device admitted>))` for each Organisation the joining service is admitted into, next to its `import_invite` call. |

`organisation_key.rs`, `a_receive_against_a_state_with_an_invalid_key_commits_nothing`
(REQ-8jb4ny): its joining service declares the expectation next to its
`import_invite`, so the refusal it asserts (`InvalidOrgPublicKey` at the chain
read) is still the one reached. `service_stories.rs`: the stories go through
`support::prepare_to_join` and `support::admit`, which carry the new flow; in
`five_stories_full_e2e` add, after B's admission commits,
`assert!(svc_b.expected_admissions().is_empty(), "the expectation is cleared on commit")`
(its annotation already names LLR-q8emds).

**Step 3 — run, expect red.** Compile errors E0599/E0560 (`expect_admission`,
`expected_admissions`, `ExpectedAdmission`, `InviteId`, `invite_id`,
`AdmissionNotExpected`, `AdmissionNotOurs`). Add the types, the field, the
two variants and `expect_admission` returning `Ok(())` without recording,
`expected_admissions` returning `&[]`, and the extra `admit_member` argument
ignored; then run: the expected-admission tests FAIL on their assertions
(`an_unexpected_first_admission…` → `Ok(…)`, the admission commits;
`a_first_admission_that_lists_none_of_our_personas_is_refused` → `Ok(…)`, B2
commits a record it is not in; the goldens on their bytes). Record, then
implement.

**Step 4 — implement.**

`org-node/src/types.rs`:

```rust
/// The identifier an Invite carries and its reply echoes (REQ-8amu2a): 32
/// bytes the inviting app drew at random. Not secret.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

`lib.rs`: `InviteId` joins the `types` re-export line; the org-members
re-export line becomes
`pub use org_members::{DevicePublicKey, Handle, MemberId, Name, PersonPublicKey, RootHash, Surname};`
with its comment unchanged (Decision 15).

`transport/wire.rs`: `WireMessage` gains, after `genesis_snapshot`,

```rust
    /// The invite identifier an admission is delivered under (LLR-ms8njy,
    /// REQ-8amu2a); `None` for every other update.
    pub invite_id: Option<InviteId>,
```

`error.rs`:

```rust
    /// A first admission whose Organisation and invite identifier match no
    /// expectation the app declared (LLR-mxskg9, LLR-s8xp7m).
    #[error("first admission not expected for organisation {org_id:?}")]
    AdmissionNotExpected { org_id: OrgId },

    /// A first admission whose verified Membership record lists none of this
    /// node's Personas (LLR-mxskg9, LLR-3f5h7b, REQ-kt877x).
    #[error("first admission to organisation {org_id:?} admits none of this node's personas")]
    AdmissionNotOurs { org_id: OrgId },
```

(with `use crate::ids::OrgId;`). `store.rs`:

```rust
/// A first admission the app declared it expects (REQ-8amu2a, LLR-95753m).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpectedAdmission {
    pub org_id: OrgId,
    pub invite_id: InviteId,
}
```

and `StoreData` and `RawStoreData` gain, as their last field,
`pub expected_admissions: Vec<ExpectedAdmission>` (doc: "Expectations the app
declared, at most once each (LLR-9zfnmb)"), copied in `TryFrom`. No
`#[serde(default)]`: stores written before this change are not supported
(owner ruling).

`service.rs`:

```rust
    /// Record that the app expects a first admission to `org_id` under
    /// `invite_id`, at most once, and save before returning (LLR-9zfnmb).
    pub fn expect_admission<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
        invite_id: InviteId,
    ) -> Result<(), OrgNodeError> {
        let expectation = ExpectedAdmission { org_id, invite_id };
        let expected = &mut self.store.data_mut().expected_admissions;
        if !expected.contains(&expectation) {
            expected.push(expectation);
        }
        self.store.save(rng)
    }

    /// The first admissions the app declared it expects (read only).
    pub fn expected_admissions(&self) -> &[ExpectedAdmission] {
        &self.store.data().expected_admissions
    }
```

In `receive_and_verify`, before `first_admission_base` is called for an
Organisation with no record:

```rust
        let expectation = msg.invite_id.map(|invite_id| ExpectedAdmission { org_id, invite_id });
        if is_first_admission && !expectation.is_some_and(|e| self.store.data().expected_admissions.contains(&e)) {
            return Err(OrgNodeError::AdmissionNotExpected { org_id });
        }
```

After verification, once `my_member` is computed and before anything is
written:

```rust
        // A first admission commits only if it lists one of our Personas
        // (LLR-3f5h7b, REQ-kt877x); the refusal writes nothing.
        if is_first_admission && my_member.is_none() {
            return Err(OrgNodeError::AdmissionNotOurs { org_id });
        }
```

and in the commit, next to the pending-invite `retain`:

```rust
            if let (true, Some(matched)) = (is_first_admission, expectation) {
                data.expected_admissions.retain(|e| *e != matched);
            }
```

`admit_member` gains a last argument `invite_id: Option<InviteId>` and puts it
in the `WireMessage` it sends; `revoke_member` sends `invite_id: None`.

**Step 5 — green.** The org-node line: expected_admission 9 passed,
value_types one more, wire_frame_bound one more, encoding_golden as before
(2 re-pinned), admission_sender unchanged in count, everything else as after
T5. Commit: `T6: expected admissions with the invite identifier; own-Persona rule`.

**Red→green attestations:** (DONE 2026-10-06, merged; org-node cargo line
207 passed, 0 failed; app `cargo check` broken as planned — E0061 at
`commands.rs:239`, then E0004 at `events.rs:68`.)

- persona_store_plaintext_is_pinned (re-pinned 676 → 729 bytes, derived by script from the old pins, equal to the plan's values) — runtime-red against unchanged code (round trip dropped the appended `01‖cc×20‖ee×32`; truncated_pinned_values_are_refused also red); green once `StoreData` gained the field
- admission_wire_message_is_pinned (re-pinned 317 → 350 bytes) — runtime-red against unchanged code (`01‖ee×32` dropped); green once `WireMessage` gained `invite_id`
- the 9 expected_admission tests (an_expected_first_admission_is_committed_and_clears_the_expectation; an_unexpected_first_admission_is_refused_before_the_snapshot_or_the_chain; an_expectation_for_one_organisation_admits_no_other; a_first_admission_under_another_invite_id_is_refused_and_keeps_the_expectation; a_first_admission_that_lists_none_of_our_personas_is_refused; expect_admission_records_a_pair_once_and_reaches_the_disk; a_refused_first_admission_leaves_the_expectation; a_second_expectation_for_the_same_organisation_survives_the_commit; an_expected_first_admission_decodes_its_snapshot_before_reading_the_chain) — compile-red (E0432/E0599), then all 9 runtime-red against stubs (`expect_admission` recording nothing, `expected_admissions` returning `&[]`, `admit_member` ignoring `invite_id`); green with the real bodies
- a_wire_message_carries_its_invite_id_and_refuses_a_short_one — compile-red (E0560); runtime-red under a temporary `#[serde(skip)]`; green
- the_first_admission_refusals_name_their_organisation — compile-red; runtime-red under temporary error messages without `org_id`; green
- every_rejection_variant_is_distinct_from_every_other (two variants added) — compile-red only: with derived `PartialEq` no stub can make distinct variants compare equal
- a_store_with_every_record_kind_opens_with_every_field_parsed (rewritten) — compile-red; runtime-red under a temporary `TryFrom` dropping the field; green
- admission_sender rewrites: two_org_receiver (and its 4 dependants), a_first_admission_that_fails_verification_leaves_the_invite_pending, a_first_admission_with_no_imported_invite_that_misses_the_chain_root_commits_nothing, the_self_delete_path_refuses_an_organisation_it_holds_no_record_of — compile-red, then runtime-red against the stub; green
- first_admission_from_a_device_other_than_the_invites_admin_is_committed, a_committed_admission_reaches_the_disk_and_consumes_the_invite, a_first_admission_to_an_expected_organisation_rests_on_the_chain_alone, a_message_for_an_organisation_absent_from_the_chain_is_refused, five_stories_full_e2e — compile-red only (they pass against the `&[]` stub; without their `expect_admission` line they fail with `AdmissionNotExpected`)

Left for T17: the decomposition still cites the old test name
`a_first_admission_with_no_imported_invite_rests_on_the_chain_alone`.

---

### T7 — org-node: the invitation exchange leaves org-node

**Files touched:** `org-node/src/blobs.rs` (deleted), `org-node/src/lib.rs`,
`org-node/src/store.rs`, `org-node/src/service.rs`,
`org-node/tests/support/mod.rs`, `org-node/tests/blob_exchange.rs` (deleted),
`org-node/tests/absences.rs` (new), `org-node/tests/encoding_golden.rs`,
`org-node/tests/persona_records.rs`, `org-node/tests/admission_sender.rs`,
`org-node/tests/service_stories.rs`, `org-node/tests/organisation_key.rs`,
`org-node/tests/secret_redaction.rs`, `org-node/tests/store_at_rest.rs`,
`org-node/Cargo.toml`, `Cargo.lock`, `org-node/.guardrails/config.yaml`
**Parallel:** no (serial, after T6)
**IDs verified:** REQ-qn2erx, REQ-xa6smf, LLR-g9vmbx, LLR-zj88e6,
LLR-qezw3n, LLR-437fvx, LLR-836z24, LLR-kkj64b, LLR-8qxwst, LLR-tcft2r,
LLR-8bum44, LLR-g76zqd, LLR-ayrdr8 (store, Invite and Join request text
removed), LLR-ctzkv7 (re-annotated).
**Size:** ~480 lines, most of them deletions.

**Step 1 — re-pin the store golden FIRST.** `pending_invites` leaves
`StoreData`. Derivation: delete its encoding — `01` (one entry) and the entry
`bb…bb (20) ‖ 2040…7c21 (32) ‖ d04a…8737 (32) ‖ d04a…8737 (32)`, 117 bytes at
offsets 559–675 — from T6's value (729 bytes, leaving 612):

```rust
const GOLDEN_STORE: &str = "0107702d616c69636501aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa05616c69636505416c69636505536d697468111111111111111111111111111111111111111111111111111111111111111112121212121212121212121212121212121212121212121212121212121212120101010101010101010101010101010101010101010101010101010101010101010101aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa3333333333333333333333333333333333333333333333333333333333333333d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c97787370301444444444444444444444444444444444444444444444444444444444444444402d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873702010101010101010101010101010101010101010101010101010101010101010105616c69636505416c69636505536d697468d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21020202020202020202020202020202020202020202020202020202020202020203626f6203426f62054a6f6e6573884b8857f4eaa1613c61504db34d4beaf346517a0e31de3cddd4d9b4201d9d0b01a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f00155555555555555555555555555555555555555555555555555555555555555550001cccccccccccccccccccccccccccccccccccccccceeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
```

In `persona_store_plaintext_is_pinned` delete the
`invite[…]` assertions and add `assert!(m.get("pending_invites").is_none(),
"no pending invites");`. **Delete** `invite_and_join_request_text_is_pinned`
(LLR-ayrdr8), `GOLDEN_INVITE`, `GOLDEN_JOIN_REQUEST` and the `blobs` import:
the Invite, the Join request and their armour leave org-node; LLR-ayrdr8's
amended statement (store, wire) is verified by the store, wire and truncation
tests that remain. In `truncated_pinned_values_are_refused` delete the two
`blobs` lines. Doc comment: "Re-pinned 2026-10-06, T7: `pending_invites`
removed (offsets 559–675); the Invite and Join request values are gone with
their types." Run `--test encoding_golden` → `persona_store_plaintext_is_pinned`
FAILED (today's `RawStoreData` reads `01 cc…` as a pending-invite vector of
one and runs out of bytes). Record.

**Step 2 — failing tests.** `support/mod.rs`:

```rust
/// What an administrator admits a Persona as.
pub type JoinerOf = org_node::Joiner;

/// The joiner `pid` of `svc` is admitted as: the Persona's details and its
/// two public keys, as the app reads them from org-node (LLR-437fvx).
pub fn joiner_of(svc: &OrgService, pid: &PersonaId) -> JoinerOf {
    let p = persona_of(svc, pid);
    let (member_key, device_key) = svc.persona_public_keys(pid).unwrap();
    org_node::Joiner { handle: p.handle, name: p.name, surname: p.surname, member_key, device_key }
}
```

`prepare_to_join` keeps only the `expect_admission` line (no invite; its
`svc_a` argument stays, unused, so no call site changes). In
`admit_b_directly` delete the `admin_member_key` equality assertion and its
message (it compared the Invite's administrator with A's; T8 removes the
field).

`org-node/tests/absences.rs` (new):

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Absences the chain-authority change requires: things org-node must not
//! contain. The compiler enforces most of them for callers; these tests make
//! each one observable in the gate by reading the source and the manifest. A
//! removed name must not appear in `src/` at all — reword any comment that
//! would name it.

use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Every `.rs` file under `org-node/src`, with its text.
pub fn sources() -> Vec<(PathBuf, String)> {
    let mut files = vec![];
    rust_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut files);
    files.into_iter().map(|p| { let s = std::fs::read_to_string(&p).unwrap(); (p, s) }).collect()
}

pub fn assert_absent(needle: &str, why: &str) {
    for (path, text) in sources() {
        assert!(!text.contains(needle), "{} contains `{needle}`: {why}", path.display());
    }
}

/// `assert_absent`, over the files whose path ends with `suffix`.
pub fn assert_absent_in(suffix: &str, needle: &str, why: &str) {
    for (path, text) in sources().into_iter().filter(|(p, _)| p.ends_with(suffix)) {
        assert!(!text.contains(needle), "{} contains `{needle}`: {why}", path.display());
    }
}

/// The `[dependencies]` table of org-node's manifest.
pub fn normal_dependencies() -> String {
    let manifest = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    let start = manifest.find("[dependencies]").unwrap() + "[dependencies]".len();
    let rest = &manifest[start..];
    rest[..rest.find("\n[").unwrap_or(rest.len())].to_string()
}

// org-node defines, exports and imports no Invite or Join request, decodes no
// armoured text, and returns no dialling address (absences: no input side).
// verifies: LLR-g9vmbx, LLR-zj88e6, LLR-qezw3n, LLR-836z24, LLR-8qxwst, REQ-qn2erx
#[test]
fn org_node_holds_no_invitation_exchange() {
    for (needle, why) in [
        ("pub mod blobs", "no blob module"),
        ("fn export_invite", "LLR-zj88e6"),
        ("fn import_invite", "LLR-g9vmbx"),
        ("fn export_join_request", "LLR-437fvx"),
        ("fn import_join_request", "LLR-836z24"),
        ("struct Invite {", "LLR-g9vmbx"),
        ("struct JoinRequest {", "LLR-836z24"),
        ("PendingInvite", "no pending invites"),
        ("pending_invites", "no pending invites"),
        // a call, not the definition, which stays in transport/endpoint.rs
        (".node_addr_for_dial(", "LLR-qezw3n: no value carries a dialling address"),
        ("base64", "LLR-8qxwst: no armoured text"),
    ] {
        assert_absent(needle, why);
    }
    assert!(!normal_dependencies().lines().any(|l| l.trim_start().starts_with("base64")), "no base64 dependency");
}
```

(The needles end in ` {` because T6's `struct InviteId` contains
`struct Invite` as text; `grep -n "struct Invite {" org-node/src` must show
only `blobs.rs` before this task's deletions.) Before relying on the address needle,
`grep -rn "\.node_addr_for_dial(" org-node/src` must show only the
`service.rs` call sites this task deletes.

Add to `org-node/Cargo.toml`:

```toml
# What org-node must not contain after the chain-authority change
# (2026-10-06): reads org-node's own source and manifest.
[[test]]
name = "absences"
path = "tests/absences.rs"
```

`admission_sender.rs`:

| Test | Change in T7 |
|---|---|
| `the_out_of_band_blobs_carry_the_keys_their_holders_are_pinned_by` | replace with `persona_public_keys_returns_the_personas_two_keys` — `// verifies: LLR-437fvx, LLR-ctzkv7`; normal: for B's Persona the call returns `(member_seed.x25519_keypair().member_key().unwrap(), device_seed.signing_keypair().device_key().unwrap())` and the two keys' bytes differ; abnormal: an unknown `PersonaId::new("nobody".into())` is refused (`is_err()`) and the store is unchanged (`store_bytes` equal). The absence half (LLR-zj88e6, LLR-qezw3n, LLR-836z24) is `org_node_holds_no_invitation_exchange`. |
| `an_imported_invite_reaches_the_joiners_disk` | **delete** (LLR-9zfnmb): the Invite leaves org-node; LLR-9zfnmb's new statement is verified by `expect_admission_records_a_pair_once_and_reaches_the_disk` (T6). |
| `pr_8qsnhx_a_join_request_advertises_the_bound_endpoint_not_its_personas` | **delete** (LLR-437fvx): the Join request is gone, and with it PR-8qsnhx's reading side; LLR-437fvx is verified by `persona_public_keys_returns_the_personas_two_keys`. |
| `pr_8qsnhx_an_invite_advertises_the_bound_endpoint_not_its_administrators` | **delete** (LLR-qezw3n): the Invite's dialling address is gone; LLR-qezw3n is verified by `org_node_holds_no_invitation_exchange`. |
| `a_committed_admission_reaches_the_disk_and_consumes_the_invite` | rename `a_committed_admission_reaches_the_disk_and_clears_the_expectation`; delete the pending-invite assertion; `// verifies: REQ-nhe2zu, REQ-txvtm9, LLR-cja9zv, LLR-q8emds`. |
| `a_first_admission_that_fails_verification_leaves_the_invite_pending` | rename `a_first_admission_that_fails_verification_leaves_the_expectation`; delete the pending-invite assertions (T6's expectation assertion stays); annotation unchanged (LLR-q8emds). |
| `first_admission_from_a_device_other_than_the_invites_admin_is_committed` | rename `a_first_admission_relayed_by_another_device_is_committed`; delete the pending-invite assertion; annotation unchanged. |
| `a_first_admission_whose_invite_names_another_organisation_key_is_committed` | **delete** (REQ-xa6smf, LLR-j83kc8, LLR-rys5nx): there is no Invite to tamper with; REQ-xa6smf and LLR-j83kc8 stay verified by `a_first_admission_relayed_by_another_device_is_committed`, LLR-rys5nx by T8's rewrite of `a_first_admission_records_…`. Delete the `tampered` helper with it. |
| `a_first_admission_with_no_imported_invite_*` (three tests) | drop "no imported invite" from each name (`a_first_admission_to_an_expected_organisation_rests_on_the_chain_alone` keeps its T6 name; `…records_the_chains_key_as_admin_member_key` until T8; `…that_misses_the_chain_root_commits_nothing` → `a_first_admission_that_misses_the_chain_root_commits_nothing`); delete every "B imports no invite" assertion; `setup_with(…, false)` stays (no expectation is declared by setup). |
| `a_receiver_holding_two_organisations_commits_into_the_one_the_change_names`, `two_org_receiver` | delete the `import_invite` calls and every `list_pending_invites()` assertion (T6's `expected_admissions()` twins stay); comments saying "invite" say "expectation". |
| `the_self_delete_path_refuses_an_organisation_it_holds_no_record_of` | delete the pending-invite assertion (T6's twin stays). |
| `pr_8qsnhx_a_second_organisations_admission_goes_out_under_the_first_personas_key`, `pr_mdv38y_…`, `pr_xwek5e_…`, others with their own joining service | delete their `import_invite`/`export_invite` calls (T6's `expect_admission` stays); `join_request_of`/`import_join_request`/`export_join_request` → `joiner_of`. |
| `a_first_admission_records_the_invites_administrator_the_chains_key_the_secret_and_the_member` | rename `a_first_admission_records_the_chains_key_the_secret_and_the_member`; its `admin_member_key` assertion now expects the chain's Organisation public key (`PersonPublicKey::parse(chain.org_pub_key.as_bytes())`) — there is no Invite; T8 deletes the assertion. Annotation unchanged (LLR-ckk5nz, LLR-e5c9ud, LLR-rys5nx, LLR-3fwykc). |

Add one admission test pair (LLR-kkj64b):

```rust
// Normal: the Member admitted carries exactly the Joiner's five values.
// verifies: LLR-kkj64b, REQ-qn2erx
#[tokio::test(flavor = "multi_thread")]
async fn an_admitted_member_carries_exactly_the_joiners_values() {
    let s = admit_b_directly(setup("joiner-values").await).await;
    let rec = rec_of(&s.svc_a, s.org_id);
    let bob = rec.trie_members.iter().find(|m| m.handle == s.joiner_b.handle).expect("bob");
    assert_eq!(
        (&bob.name, &bob.surname, bob.member_key, bob.device_keys.as_slice()),
        (&s.joiner_b.name, &s.joiner_b.surname, s.joiner_b.member_key, &[s.joiner_b.device_key][..])
    );
}

// Abnormal: a Joiner whose member key the Organisation already holds is
// refused by the trie and nothing is admitted.
// verifies: LLR-kkj64b
#[tokio::test(flavor = "multi_thread")]
async fn a_joiner_with_a_member_key_already_held_is_refused() {
    let mut s = setup("joiner-dup").await;
    let (a_member_key, _) = s.svc_a.persona_public_keys(&s.pid_a).unwrap();
    let dup = org_node::Joiner { member_key: a_member_key, ..s.joiner_b.clone() };
    let before = rec_of(&s.svc_a, s.org_id);
    let (sink_addr, _sink) = spawn_recv_one(rand::random()).await;
    let err = admit(&mut s.svc_a, &s.chain, s.org_id, &dup, sink_addr, org_secret()).await.unwrap_err();
    assert!(matches!(err, OrgNodeError::Trie(_)), "{err:?}");
    assert_eq!(rec_of(&s.svc_a, s.org_id).trie_members.len(), before.trie_members.len());
}
```

(A duplicate key is org-members' `DuplicateKey` refusal — every key held once,
the owner's ruling of 2026-10-04 — which holds whatever the handle.)

`persona_records.rs`:
- **Delete** `a_valid_join_request_imports_with_every_field_parsed`,
  `a_non_nfc_join_request_imports_with_its_details_in_nfc`,
  `a_join_request_holding_an_invalid_value_is_refused_naming_the_field`,
  `a_valid_invite_imports_as_a_pending_invite`,
  `an_invite_holding_a_key_that_is_not_a_curve_point_is_refused_and_nothing_stored`
  (all LLR-8bum44): the imports are removed (REQ-qn2erx as amended);
  LLR-8bum44 stays verified by the store-open and record-snapshot tests in
  this file. Delete `join_request_blob`, `invite_blob` and the `blobs` import.
- `create_persona_holds_the_parsed_details_across_a_reopen` (LLR-g76zqd):
  replace the Join-request round trip by a read of the record and
  `persona_public_keys`: `let p = svc.list_personas()[0].clone();` then assert
  `svc.persona_public_keys(&pid).unwrap() == (p.member_seed.x25519_keypair().member_key().unwrap(), p.device_seed.signing_keypair().device_key().unwrap())`
  and that `p.handle`, `p.name`, `p.surname` equal the parsed values the test
  created the Persona with.
- `non_nfc_persona_details_are_stored_and_exported_in_nfc` (LLR-q6n25z):
  rename `non_nfc_persona_details_are_stored_in_nfc`; delete the
  `export_join_request` decode; keep the stored-record assertions.
- `WireStore` mirror: delete `pending_invites` and its element mirror; the
  any-one-field-invalid table: delete the `pending_invite.*` rows.
- Add (LLR-tcft2r, both sides):

```rust
// Abnormal: bytes that are not an encoded record snapshot are refused with a
// typed error, not a panic, and extend nothing.
// verifies: LLR-tcft2r, REQ-9g6as6
#[test]
fn bytes_that_are_not_a_record_snapshot_are_refused() {
    for bytes in [&[][..], &[0xff; 8][..], &[0x05, 0x00][..]] {
        assert!(org_node::service::first_admission_base(Some(bytes)).is_err(), "{bytes:?}");
    }
}
```

  and add `LLR-tcft2r` to the annotation of
  `a_valid_record_snapshot_decodes_with_every_field_parsed` (its normal side).

`service_stories.rs`: `support::joiner_of` and `prepare_to_join` carry the new
flow; delete any remaining `export_invite`/`import_invite` lines and the
`invite.org_id` assertions; update the story comments ("B imports A's invite"
→ "B declares it expects the admission"). The annotation of
`five_stories_full_e2e` keeps REQ-xa6smf (it still commits a first admission).

`organisation_key.rs`, `a_receive_against_a_state_with_an_invalid_key_commits_nothing`:
`import_invite`/`export_invite` deleted (T6's `expect_admission` stays);
`import_join_request(&export_join_request(..))` → the file's own
`Joiner { … }` from `persona_public_keys`.

`secret_redaction.rs`, `store_at_rest.rs`: delete `pending_invites` from
`StoreData` literals.

**Step 3 — run, expect red.** Compile errors (E0412 `Joiner`, E0599
`persona_public_keys`), then — with stubs `pub struct Joiner {…}` and a
`persona_public_keys` returning `Err` — runtime reds: the golden store test,
`persona_public_keys_returns_the_personas_two_keys` (`Err`),
`org_node_holds_no_invitation_exchange` (finds `pub mod blobs`). Record.

**Step 4 — implement.**
- Delete `org-node/src/blobs.rs`; in `lib.rs` delete `pub mod blobs;` and add
  `pub use service::Joiner;` to the `app` re-exports.
- `store.rs`: delete `PendingInvite`, `RawPendingInvite`, its `TryFrom`, and
  `pending_invites` from `StoreData`, `RawStoreData` and their `TryFrom`.
- `service.rs`: delete `export_invite`, `import_invite`,
  `export_join_request`, `import_join_request`, `list_pending_invites`, the
  `PendingInvite` import and the commit's pending-invite `retain`; the
  first-admission administrator key is the chain's alone
  (`PersonPublicKey::parse(chain_state.org_pub_key.as_bytes())?`, LLR-xq9nrq
  until T8 removes the field). Add:

```rust
/// The person an admission adds, as parsed values the app built from the
/// Invite reply it parsed (LLR-kkj64b).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Joiner {
    pub handle: Handle,
    pub name: Name,
    pub surname: Surname,
    pub member_key: PersonPublicKey,
    pub device_key: DevicePublicKey,
}
```

  `admit_member` takes `joiner: &Joiner` in place of the Join request (field
  names are the same, so its body changes only in the parameter name and the
  Networked arm's `joiner.device_key`). Add:

```rust
    /// The Persona's Member-as-a-group key and DevicePublicKey, each from its
    /// own seed, and no dialling address (LLR-437fvx).
    pub fn persona_public_keys(&self, persona_id: &PersonaId) -> Result<(PersonPublicKey, DevicePublicKey), OrgNodeError> {
        let p = self.find_persona(persona_id)?;
        Ok((p.member_seed.x25519_keypair().member_key()?, p.device_seed.signing_keypair().device_key()?))
    }
```

  (`DevicePublicKey` joins the org-members import.)
- `org-node/Cargo.toml`: delete `"dep:base64"` from `app`, the optional
  `base64` dependency, the `base64` dev-dependency (after confirming
  `grep -rn base64 org-node/tests` finds nothing), and the `blob_exchange`
  `[[test]]` entry; the `app` feature comment loses "+ out-of-band blobs".
- Delete `org-node/tests/blob_exchange.rs`: its five tests are removed with
  the behaviour — `an_invite_round_trips_carrying_every_field` (LLR-g9vmbx →
  `org_node_holds_no_invitation_exchange`),
  `a_join_request_round_trips_carrying_every_field` (LLR-kkj64b →
  `an_admitted_member_carries_exactly_the_joiners_values`),
  `a_string_that_is_not_base64_is_refused` (REQ-9g6as6, LLR-8qxwst →
  `org_node_holds_no_invitation_exchange`),
  `valid_base64_that_is_not_a_blob_is_refused` and
  `an_invite_does_not_decode_as_a_join_request` (REQ-9g6as6, LLR-tcft2r →
  `bytes_that_are_not_a_record_snapshot_are_refused`).
- `org-node/.guardrails/config.yaml`: delete ` --test blob_exchange` and add
  ` --test absences` on the cargo line; add a dated comment line ("T7: the
  invitation exchange left org-node; blob_exchange deleted; `absences` reads
  org-node's source for what it must not contain").

**Step 5 — green, and the lock.** The org-node line:
`git diff Cargo.lock` shows the `org-node` package losing `"base64"` (and the
`base64` package entry only if nothing else needs it); no `version =` line of
a remaining package changes. `cargo check --all-targets` per Environment.
Expected: encoding_golden one fewer, persona_records five fewer plus one,
admission_sender four fewer plus three, absences 1. Commit:
`T7: the invitation exchange leaves org-node`.

**Red→green attestations:** (DONE 2026-10-06, merged; org-node cargo line
196 passed, 0 failed — 207 − 11 deleted + 1 absences − … with admission_sender
46 → 44 (five removed, three added; the plan's "four fewer" undercounted).
App `cargo check` first error: E0432 `org_node::blobs` at `commands.rs:23`.)

- persona_store_plaintext_is_pinned (re-pinned 729 → 612 bytes, derived by script by removing offsets 559–675, equal to the plan's value) — runtime-red against unchanged code (`DeserializeUnexpectedEnd`); green once `pending_invites` left `StoreData`
- org_node_holds_no_invitation_exchange — runtime-red against the stubs (`src/lib.rs contains pub mod blobs`); green after the deletions
- persona_public_keys_returns_the_personas_two_keys — compile-red (E0599); runtime-red against an `Err` stub (via setup's `joiner_of`); green
- create_persona_holds_the_parsed_details_across_a_reopen (rewritten) — compile-red; runtime-red at its own assertion against the `Err` stub; green
- an_admitted_member_carries_exactly_the_joiners_values — compile-red (E0422 `Joiner`); runtime-red only through setup's `joiner_of`; green
- a_joiner_with_a_member_key_already_held_is_refused — compile-red; runtime-red only through setup's `joiner_of`; green
- bytes_that_are_not_a_record_snapshot_are_refused — **no red**: `first_admission_base` already refused these bytes before T7; the test carries coverage over from the deleted blob tests and passed on its first run
- admission_sender (43 via support), service_stories (3), organisation_key's a_receive_against_a_state_with_an_invalid_key_commits_nothing — compile-red, then runtime-red against the `Err` stub; green
- persona_records StoreData/WireStore rewrites — compile-red only (E0063); removing the field is the change, so no stub can redden them at runtime

Deleted, each with its replacement:
- invite_and_join_request_text_is_pinned (LLR-ayrdr8) → the remaining store/wire/truncation golden tests
- blob_exchange.rs: an_invite_round_trips_carrying_every_field (LLR-g9vmbx) → org_node_holds_no_invitation_exchange; a_join_request_round_trips_carrying_every_field (LLR-kkj64b) → an_admitted_member_carries_exactly_the_joiners_values; a_string_that_is_not_base64_is_refused (REQ-9g6as6, LLR-8qxwst) → org_node_holds_no_invitation_exchange; valid_base64_that_is_not_a_blob_is_refused and an_invite_does_not_decode_as_a_join_request (REQ-9g6as6, LLR-tcft2r) → bytes_that_are_not_a_record_snapshot_are_refused
- persona_records (LLR-8bum44): the five Join-request/Invite import tests → the store-open and record-snapshot tests in that file
- admission_sender: an_imported_invite_reaches_the_joiners_disk (LLR-9zfnmb) → expect_admission_records_a_pair_once_and_reaches_the_disk (T6); pr_8qsnhx_a_join_request_advertises_the_bound_endpoint_not_its_personas (LLR-437fvx) → persona_public_keys_returns_the_personas_two_keys; pr_8qsnhx_an_invite_advertises_the_bound_endpoint_not_its_administrators (LLR-qezw3n) → org_node_holds_no_invitation_exchange; a_first_admission_whose_invite_names_another_organisation_key_is_committed (REQ-xa6smf, LLR-j83kc8, LLR-rys5nx) → a_first_admission_relayed_by_another_device_is_committed and T8

Surprises: `cargo check` in app/src-tauri rewrote its Cargo.lock (drops base64
from org-node, bumps lock format 3 → 4); restored — that lock is T14's.

---

### T8 — org-node: no administrator key; the provisional-update store and its bound

**Files touched:** `org-node/src/store.rs`, `org-node/src/error.rs`,
`org-node/src/service.rs`, `org-node/tests/provisional_store.rs` (new),
`org-node/tests/encoding_golden.rs`, `org-node/tests/value_types.rs`,
`org-node/tests/persona_records.rs`, `org-node/tests/secret_redaction.rs`,
`org-node/tests/store_at_rest.rs`, `org-node/tests/admission_sender.rs`,
`org-node/tests/absences.rs`, `org-node/Cargo.toml`,
`org-node/.guardrails/config.yaml`
**Parallel:** no (serial, after T7)
**IDs verified:** REQ-xs4ab8 (storage half), REQ-fwfku9, REQ-ech45n,
LLR-95753m, LLR-jq7qh7, LLR-qjz3q4, LLR-mxskg9 (`ProvisionalLimit`),
LLR-8bum44 (`provisional.*`), LLR-g76zqd, LLR-xq9nrq, LLR-rys5nx, LLR-e5c9ud,
LLR-ayrdr8 (final store).
**Size:** ~470 lines.

**Step 1 — re-pin the store golden FIRST (final form).** Two changes:
`OrgRecord.admin_member_key` goes (32 bytes at offsets 266–297 of T7's value,
the `d04a…8737` right after `last_seq` `02`), and `provisional_updates`
enters between `orgs` and `expected_admissions` (at offset 527 after the cut)
with two updates, a vector of two prefixed `02`:

- a genesis update (224 bytes): `org_id None` → `00`; `persona_id "p-alice"`
  → `07 702d616c696365`; `base_root None` → `00`; `resulting_root 77×32`;
  `seq 1` → `01`; `org_pub_key` the fixture key `d04a…8737`; `change
  Genesis` → variant `00`, `members` a vector of one, `01`, holding the
  record's own `alice` snapshot (`01×32 ‖ 05 alice ‖ 05 Alice ‖ 05 Smith ‖
  d04a…8737 ‖ 01 2040…7c21`), then `org_private_key 88×32`;
- a Change-set update (133 bytes): `org_id Some(bb×20)` → `01 bb…bb`;
  `persona_id "p-alice"`; `base_root Some(33×32)` → `01 33…33`;
  `resulting_root 66×32`; `seq 3` → `03`; `org_pub_key d04a…8737`; `change
  ChangeSet { change_set: [9,8,7,6] }` → variant `01`, `04 09080706`.

Result (938 bytes):

```rust
const GOLDEN_STORE: &str = "0107702d616c69636501aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa05616c69636505416c69636505536d697468111111111111111111111111111111111111111111111111111111111111111112121212121212121212121212121212121212121212121212121212121212120101010101010101010101010101010101010101010101010101010101010101010101aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa3333333333333333333333333333333333333333333333333333333333333333d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737030144444444444444444444444444444444444444444444444444444444444444440202010101010101010101010101010101010101010101010101010101010101010105616c69636505416c69636505536d697468d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21020202020202020202020202020202020202020202020202020202020202020203626f6203426f62054a6f6e6573884b8857f4eaa1613c61504db34d4beaf346517a0e31de3cddd4d9b4201d9d0b01a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f001555555555555555555555555555555555555555555555555555555555555555500020007702d616c69636500777777777777777777777777777777777777777777777777777777777777777701d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c97787370001010101010101010101010101010101010101010101010101010101010101010105616c69636505416c69636505536d697468d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21888888888888888888888888888888888888888888888888888888888888888801bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb07702d616c696365013333333333333333333333333333333333333333333333333333333333333333666666666666666666666666666666666666666666666666666666666666666603d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701040908070601cccccccccccccccccccccccccccccccccccccccceeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
```

In `persona_store_plaintext_is_pinned`: delete
`assert_eq!(org["admin_member_key"]…)`; add
`assert!(org.get("admin_member_key").is_none(), "no administrator key");` and

```rust
    let genesis = &m["provisional_updates"][0];
    assert_eq!(genesis["org_id"], json!(null));
    assert_eq!(genesis["seq"], json!(1));
    assert_eq!(genesis["change"]["Genesis"]["members"][0]["handle"], json!("alice"));
    assert_eq!(genesis["change"]["Genesis"]["org_private_key"][0], json!(0x88));
    let pu = &m["provisional_updates"][1];
    assert_eq!(pu["org_id"][0], json!(0xbb));
    assert_eq!(pu["persona_id"], json!("p-alice"));
    assert_eq!(pu["base_root"][0], json!(0x33));
    assert_eq!(pu["resulting_root"][0], json!(0x66));
    assert_eq!(pu["seq"], json!(3));
    assert_eq!(pu["org_pub_key"][0], json!(0xd0));
    assert_eq!(pu["change"]["ChangeSet"]["change_set"], json!([9, 8, 7, 6]));
```

(If `OrgPrivateKey`'s `Serialize` does not render as a plain array in
serde's data model, read the 0x88 byte from the postcard bytes at its offset
instead; the round-trip assertion above it already pins the bytes.) Doc
comment: "Re-pinned 2026-10-06, T8 (final for this change): the
administrator key removed from the record (offsets 266–297); two provisional
updates, a genesis and a Change set, between the records and the
expectations — the store LLR-ayrdr8 names." Run → `persona_store_plaintext_is_pinned`
FAILED (decode error). Record.

**Step 2 — failing tests.** `org-node/tests/provisional_store.rs`:

```rust
#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The provisional updates a Persona store keeps (REQ-xs4ab8, REQ-fwfku9):
//! identity, persistence and the 1 MiB bound (LLR-95753m, LLR-jq7qh7), where
//! a genesis update keeps the Organisation private key (LLR-qjz3q4,
//! REQ-ech45n), and the field-naming parse on open (LLR-8bum44).

use org_node::ids::OrgId;
use org_node::store::{
    ExpectedAdmission, MemberSnapshot, PersonaStore, ProvisionalChange, ProvisionalUpdate, StoreData,
    MAX_PROVISIONAL_BYTES,
};
use org_node::test_fixtures::{device_key, member_key, org_public_key};
use org_node::{Handle, InviteId, Name, OrgNodeError, OrgPrivateKey, PersonaId, RootHash, SequenceNumber, Surname};
use rand::rngs::OsRng;

fn change_set(org: u8, root: u8, bytes: usize) -> ProvisionalUpdate {
    ProvisionalUpdate {
        org_id: Some(OrgId::new([org; 20])),
        persona_id: PersonaId::new("p-alice".into()),
        base_root: Some(RootHash::new([0x33; 32])),
        resulting_root: RootHash::new([root; 32]),
        seq: SequenceNumber::new(3),
        org_pub_key: org_public_key(),
        change: ProvisionalChange::ChangeSet { change_set: vec![0xab; bytes] },
    }
}

const PRIVATE: [u8; 32] = [0x88; 32];

fn genesis(persona: &str, root: u8) -> ProvisionalUpdate {
    ProvisionalUpdate {
        org_id: None,
        persona_id: PersonaId::new(persona.into()),
        base_root: None,
        resulting_root: RootHash::new([root; 32]),
        seq: SequenceNumber::new(1),
        org_pub_key: OrgPrivateKey::from(PRIVATE).x25519_keypair().org_public_key().unwrap(),
        change: ProvisionalChange::Genesis {
            members: vec![MemberSnapshot {
                id: org_node::MemberId::new([1; 32]),
                handle: Handle::parse("alice").unwrap(),
                name: Name::parse("Alice").unwrap(),
                surname: Surname::parse("Smith").unwrap(),
                member_key: member_key(0x21),
                device_keys: vec![device_key(0x22)],
            }],
            org_private_key: OrgPrivateKey::from(PRIVATE),
        },
    }
}

fn store_path(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("ods-provisional-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("store.bin")
}

fn contains(hay: &[u8], needle: &[u8]) -> usize {
    hay.windows(needle.len()).filter(|w| *w == needle).count()
}

// Normal: provisional updates and expected admissions reach the encrypted
// file and are read back, typed, when it is opened.
// verifies: REQ-xs4ab8, LLR-95753m, LLR-g76zqd
#[test]
fn provisional_updates_round_trip_through_the_encrypted_file() {
    let path = store_path("round-trip");
    let mut store = PersonaStore::open(path.clone(), "pw").unwrap();
    store.data_mut().insert_provisional(genesis("p-alice", 0x10)).unwrap();
    store.data_mut().insert_provisional(change_set(0xbb, 0x66, 4)).unwrap();
    let expectation = ExpectedAdmission { org_id: OrgId::new([0xcc; 20]), invite_id: InviteId::new([0xee; 32]) };
    store.data_mut().expected_admissions.push(expectation);
    store.save(&mut OsRng).unwrap();
    let reopened = PersonaStore::open(path, "pw").unwrap();
    assert_eq!(reopened.data().provisional_updates, vec![genesis("p-alice", 0x10), change_set(0xbb, 0x66, 4)]);
    assert_eq!(reopened.data().expected_admissions, vec![expectation]);
}

// Abnormal: the same identity replaces rather than duplicates; different
// identities — another root, another Persona's genesis — are all kept.
// verifies: REQ-xs4ab8, LLR-95753m
#[test]
fn an_update_with_the_same_identity_replaces_and_others_are_kept() {
    let mut data = StoreData::default();
    data.insert_provisional(change_set(0xbb, 0x66, 4)).unwrap();
    data.insert_provisional(change_set(0xbb, 0x66, 5)).unwrap();
    assert_eq!(data.provisional_updates, vec![change_set(0xbb, 0x66, 5)], "replaced, not added");
    data.insert_provisional(change_set(0xbb, 0x67, 4)).unwrap();
    data.insert_provisional(genesis("p-alice", 0x66)).unwrap();
    data.insert_provisional(genesis("p-bob", 0x66)).unwrap();
    assert_eq!(data.provisional_updates.len(), 4);
}

// Normal: a genesis update holds its Organisation private key, whose public
// key is the update's, once in the store's plaintext; abnormal: with the
// update gone, no copy of the key remains anywhere in the store.
// verifies: REQ-ech45n, LLR-qjz3q4
#[test]
fn the_genesis_private_key_lives_only_in_its_provisional_update() {
    let mut data = StoreData::default();
    let update = genesis("p-alice", 0x10);
    let ProvisionalChange::Genesis { org_private_key, .. } = &update.change else { panic!("genesis") };
    assert_eq!(org_private_key.x25519_keypair().org_public_key().unwrap(), update.org_pub_key);
    data.insert_provisional(update).unwrap();
    assert_eq!(contains(&postcard::to_allocvec(&data).unwrap(), &PRIVATE), 1);
    assert!(data.orgs.is_empty());
    data.provisional_updates.clear();
    assert_eq!(contains(&postcard::to_allocvec(&data).unwrap(), &PRIVATE), 0);
}

/// The postcard length of a one-update group whose change set is `n` bytes.
fn group_len(n: usize) -> usize {
    postcard::to_allocvec(&vec![change_set(0xbb, 0x66, n)]).unwrap().len()
}

// Normal (boundary): a group of exactly 1 MiB is kept; the bound is the
// wire frame's.
// verifies: REQ-fwfku9, LLR-jq7qh7
#[test]
fn a_group_of_exactly_the_bound_is_kept() {
    assert_eq!(MAX_PROVISIONAL_BYTES, org_node::transport::MAX_FRAME);
    let fixed = group_len(20_000) - 20_000; // the varint is three bytes from 16 384 up
    let n = MAX_PROVISIONAL_BYTES - fixed;
    assert_eq!(group_len(n), MAX_PROVISIONAL_BYTES);
    let mut data = StoreData::default();
    data.insert_provisional(change_set(0xbb, 0x66, n)).unwrap();
    assert_eq!(data.provisional_updates.len(), 1);
}

// Abnormal: one byte over, alone or as the sum of two, is refused with the
// limit named, and the store is exactly as it was; another Organisation's
// group is measured on its own.
// verifies: REQ-fwfku9, LLR-jq7qh7, LLR-mxskg9
#[test]
fn an_update_that_would_exceed_the_bound_is_refused_and_nothing_changes() {
    let fixed = group_len(20_000) - 20_000;
    let mut data = StoreData::default();
    assert_eq!(
        data.insert_provisional(change_set(0xbb, 0x66, MAX_PROVISIONAL_BYTES - fixed + 1)).unwrap_err(),
        OrgNodeError::ProvisionalLimit { limit: MAX_PROVISIONAL_BYTES }
    );
    assert!(data.provisional_updates.is_empty());
    data.insert_provisional(change_set(0xbb, 0x66, 600_000)).unwrap();
    let before = postcard::to_allocvec(&data).unwrap();
    assert_eq!(
        data.insert_provisional(change_set(0xbb, 0x67, 600_000)).unwrap_err(),
        OrgNodeError::ProvisionalLimit { limit: MAX_PROVISIONAL_BYTES }
    );
    assert_eq!(postcard::to_allocvec(&data).unwrap(), before, "StoreData exactly as it was");
    data.insert_provisional(change_set(0xdd, 0x67, 600_000)).unwrap();
    assert_eq!(data.provisional_updates.len(), 2, "another Organisation's group is its own");
}
```

`ProvisionalUpdate` must be `PartialEq` for these assertions, which needs
`OrgPrivateKey: PartialEq` (master's LLR-322xfu keeps `PartialEq`/`Eq`) and
`MemberSnapshot: PartialEq, Eq` (added below).

Add one test to `persona_records.rs` (it already has the `WireStore` mirror
and the `sealed_wire_store` helper) — first extend the mirror: `WireOrg`
loses `admin_member_key`, and `WireStore` gains `provisional_updates:
Vec<WireProvisional>` before `expected_admissions`, with

```rust
#[derive(Serialize)]
struct WireProvisional {
    org_id: Option<[u8; 20]>,
    persona_id: String,
    base_root: Option<[u8; 32]>,
    resulting_root: [u8; 32],
    seq: u64,
    org_pub_key: [u8; 32],
    change: WireChange,
}

#[derive(Serialize)]
enum WireChange {
    Genesis { members: Vec<WireMember>, org_private_key: [u8; 32] },
    #[allow(dead_code)]
    ChangeSet { change_set: Vec<u8> },
}
```

(`WireMember` is the mirror the file already has for member snapshots; use
its name as it is in the file.) The fixture `wire_store()` gains one genesis
provisional update built from the same member fixture; then:

```rust
// Abnormal: a stored provisional update whose key or member does not parse
// fails the open as a whole, naming the field.
// verifies: LLR-8bum44
#[test]
fn a_store_with_an_invalid_provisional_update_is_refused_naming_the_field() {
    for (field, corrupt) in [
        ("provisional.org_pub_key", (|s: &mut WireStore| s.provisional_updates[0].org_pub_key = off_curve_key()) as fn(&mut WireStore)),
        ("member.handle", |s: &mut WireStore| match &mut s.provisional_updates[0].change {
            WireChange::Genesis { members, .. } => members[0].handle = "Not A Handle".into(),
            WireChange::ChangeSet { .. } => unreachable!(),
        }),
    ] {
        let mut wire = wire_store();
        corrupt(&mut wire);
        let path = sealed_wire_store(&format!("bad-provisional-{field}"), &wire);
        let err = PersonaStore::open(path, PASSPHRASE).unwrap_err();
        assert!(matches!(&err, OrgNodeError::InvalidField { field: f, .. } if *f == field), "{field}: {err:?}");
    }
}
```

(`PASSPHRASE` is whatever the file's `sealed_wire_store` seals under; use its
name as it is.) In the any-one-field-invalid table delete the
`("org.admin_member_key", …)` row; in
`a_store_with_every_record_kind_opens_with_every_field_parsed` delete the
`admin_member_key` assertion and add one that the provisional update reads
back with `org_pub_key` parsed; in `org_with_member()` and every `OrgRecord`
literal delete `admin_member_key`.

`value_types.rs`: add `OrgNodeError::ProvisionalLimit { limit: 1 }` to the
distinct list and:

```rust
// Normal: the refusal names the limit in bytes; abnormal: two limits differ.
// verifies: LLR-mxskg9, REQ-fwfku9
#[test]
fn the_provisional_limit_refusal_names_the_limit() {
    let err = OrgNodeError::ProvisionalLimit { limit: 1_048_576 };
    assert_eq!(err.to_string(), "provisional updates for one Organisation would exceed 1048576 bytes");
    assert_ne!(err, OrgNodeError::ProvisionalLimit { limit: 1 });
}
```

`absences.rs`:

```rust
// org-node has no administrator: no record carries one and no Persona is
// looked up by one (absences: no input side).
// verifies: LLR-xq9nrq, LLR-rys5nx, LLR-g76zqd
#[test]
fn org_node_has_no_administrator_key() {
    assert_absent("admin_member_key", "OrgRecord has no administrator key");
    assert_absent("admin_device_key", "no administrator device key");
    assert_absent("fn admin_persona_for_org", "no Persona is looked up by an administrator key");
}
```

`admission_sender.rs`:

| Test | Change in T8 |
|---|---|
| `member_ids_are_not_derived_from_keys` | `admin_snap.member_key` is compared with `s.svc_a.persona_public_keys(&s.pid_a).unwrap().0` instead of `rec.admin_member_key`. |
| `a_first_admission_records_the_chains_key_the_secret_and_the_member` | delete the `admin_member_key` assertion; keep `org_pub_key == chain value`, the secret, the member id and `org_private_key == None`; annotation unchanged (LLR-ckk5nz, LLR-e5c9ud, LLR-rys5nx, LLR-3fwykc). |
| `a_first_admission_with_no_imported_invite_records_the_chains_key_as_admin_member_key` | rename `a_first_admission_records_the_chains_organisation_public_key`; assert the record's `org_pub_key` is the chain's, and no other key; `// verifies: LLR-xq9nrq, LLR-rys5nx`. |
| `the_admission_envelope_carries_the_epoch_its_update_produced` | delete the assertions on `before.admin_member_key`; keep the epoch and mark assertions (LLR-ghja3x, REQ-txvtm9). |
| `the_administrators_own_persona_is_never_the_one_a_receive_marks` (LLR-e5c9ud) | it pins the exclusion this task removes: rewrite to the amended statement — the Persona marked Active is the first whose DevicePublicKey is in the verified trie, whatever its member key — and rename `the_persona_marked_active_is_the_first_whose_device_is_in_the_record`; annotation unchanged. |
| new | see below |

```rust
// Normal side of LLR-e5c9ud as amended: the Persona whose device the trie
// holds is bound even when its member key is the key the chain publishes —
// the administrator-key exclusion is gone. And the record keeps the chain's
// key, not one from the message (LLR-xq9nrq).
// verifies: LLR-e5c9ud, LLR-xq9nrq
#[tokio::test(flavor = "multi_thread")]
async fn a_persona_whose_member_key_the_chain_publishes_is_still_bound() {
    let mut s = setup("published-key").await;
    let joiner = s.joiner_b.clone();
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &joiner, sink_addr, org_secret()).await.unwrap();
    let msg = sink.await.unwrap().2;
    let state = s.chain.get(&s.org_id).unwrap();
    let published = org_node::OrgPublicKey::parse(joiner.member_key.as_bytes()).unwrap();
    s.chain.set(s.org_id, org_node::chain::OrgState { org_pub_key: published, ..state });
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

(Red today: the exclusion skips B's Persona, so no Persona of B is in the
record by T6's rule and B refuses with `AdmissionNotOurs`.)

`secret_redaction.rs`, `store_at_rest.rs`: delete `admin_member_key` from
`OrgRecord` literals; add `provisional_updates: vec![]` to `StoreData`
literals.

Cargo: `[[test]] name = "provisional_store"`, `path =
"tests/provisional_store.rs"`, `required-features = ["app", "test-support"]`;
config: ` --test provisional_store`.

**Step 3 — run, expect red** (compile errors for the new types first; with
stub types and an `insert_provisional` that pushes unconditionally, the
bound and identity tests and the golden test FAIL on their assertions;
`org_node_has_no_administrator_key` finds `admin_member_key`;
`a_persona_whose_member_key…` FAILS with `AdmissionNotOurs`). Record.

**Step 4 — implement.** `org-node/src/error.rs`:

```rust
    /// Keeping a provisional update would bring one Organisation's
    /// provisional updates above `limit` bytes (LLR-mxskg9, LLR-jq7qh7).
    #[error("provisional updates for one Organisation would exceed {limit} bytes")]
    ProvisionalLimit { limit: usize },
```

`org-node/src/store.rs`:
- `OrgRecord`/`RawOrgRecord`/`TryFrom`: delete `admin_member_key`; the
  `proxy_account` doc becomes "The proxy account the app handed
  `commit_genesis`, held opaquely and only handed back (LLR-dzte8x); `None`
  on a record created by a first admission." Delete the `#[serde(default)]`
  on `proxy_account` and `org_private_key` and their "stores written before"
  comments (owner ruling: earlier stores are not supported).
- `MemberSnapshot`: derive `PartialEq, Eq` in addition.
- Add:

```rust
/// The bound on one Organisation's provisional updates in their stored form:
/// the wire frame's 1 MiB (LLR-jq7qh7). A separate constant from
/// `transport::MAX_FRAME`, so changing one is a decision about the other.
pub const MAX_PROVISIONAL_BYTES: usize = 1 << 20;

/// An update this node built and keeps until the chain agrees with it
/// (REQ-xs4ab8, LLR-95753m).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawProvisionalUpdate")]
pub struct ProvisionalUpdate {
    /// `None` for genesis: there is no identifier until the chain write.
    pub org_id: Option<OrgId>,
    /// The Persona that built it.
    pub persona_id: PersonaId,
    /// The record's root it applies to; `None` for genesis.
    pub base_root: Option<RootHash>,
    /// The Membership root it produces.
    pub resulting_root: RootHash,
    /// The epoch it produces on the chain: the record's epoch plus one, or 1
    /// for genesis (REQ-txvtm9, Decision 16).
    pub seq: SequenceNumber,
    /// The Organisation public key the app publishes with the root.
    pub org_pub_key: OrgPublicKey,
    pub change: ProvisionalChange,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProvisionalChange {
    /// The founding Member, and the Organisation private key whose public key
    /// is the update's, kept until `commit_genesis` (LLR-qjz3q4).
    Genesis { members: Vec<MemberSnapshot>, org_private_key: OrgPrivateKey },
    /// postcard of the Change set, for an admission or a revocation.
    ChangeSet { change_set: Vec<u8> },
}

/// A `ProvisionalUpdate` as decoded, before parsing (LLR-8bum44).
#[derive(Deserialize)]
pub(crate) struct RawProvisionalUpdate {
    org_id: Option<OrgId>,
    persona_id: PersonaId,
    base_root: Option<RootHash>,
    resulting_root: RootHash,
    seq: SequenceNumber,
    org_pub_key: [u8; 32],
    change: RawProvisionalChange,
}

#[derive(Deserialize)]
pub(crate) enum RawProvisionalChange {
    Genesis { members: Vec<RawMemberSnapshot>, org_private_key: OrgPrivateKey },
    ChangeSet { change_set: Vec<u8> },
}

impl TryFrom<RawProvisionalUpdate> for ProvisionalUpdate {
    type Error = OrgNodeError;

    fn try_from(raw: RawProvisionalUpdate) -> Result<Self, Self::Error> {
        Ok(Self {
            org_id: raw.org_id,
            persona_id: raw.persona_id,
            base_root: raw.base_root,
            resulting_root: raw.resulting_root,
            seq: raw.seq,
            org_pub_key: parse_field("provisional.org_pub_key", OrgPublicKey::parse(&raw.org_pub_key))?,
            change: match raw.change {
                RawProvisionalChange::Genesis { members, org_private_key } => ProvisionalChange::Genesis {
                    members: members.into_iter().map(MemberSnapshot::try_from).collect::<Result<_, _>>()?,
                    org_private_key,
                },
                RawProvisionalChange::ChangeSet { change_set } => ProvisionalChange::ChangeSet { change_set },
            },
        })
    }
}
```

- `StoreData` gains `pub provisional_updates: Vec<ProvisionalUpdate>` between
  `orgs` and `expected_admissions` (and `RawStoreData` gains
  `provisional_updates: Vec<RawProvisionalUpdate>` there, converted in
  `TryFrom`). Add `StoreData::insert_provisional` exactly as in the design
  (LLR-95753m, LLR-jq7qh7):

```rust
impl StoreData {
    /// Keep `update`: replace the stored update with its identity, else
    /// append; refuse — changing nothing — when its Organisation's group
    /// would exceed `MAX_PROVISIONAL_BYTES` in postcard form (LLR-95753m,
    /// LLR-jq7qh7). Identity: `(org_id, resulting_root)`, or for genesis
    /// `(persona_id, resulting_root)`.
    pub fn insert_provisional(&mut self, update: ProvisionalUpdate) -> Result<(), OrgNodeError> {
        let same_group = |u: &ProvisionalUpdate| match update.org_id {
            Some(org) => u.org_id == Some(org),
            None => u.org_id.is_none() && u.persona_id == update.persona_id,
        };
        let same_identity = |u: &ProvisionalUpdate| same_group(u) && u.resulting_root == update.resulting_root;
        let mut group: Vec<&ProvisionalUpdate> =
            self.provisional_updates.iter().filter(|u| same_group(u) && !same_identity(u)).collect();
        group.push(&update);
        let len = postcard::to_allocvec(&group)
            .map_err(|e| OrgNodeError::Chain(format!("provisional encode: {e}")))?
            .len();
        if len > MAX_PROVISIONAL_BYTES {
            return Err(OrgNodeError::ProvisionalLimit { limit: MAX_PROVISIONAL_BYTES });
        }
        match self.provisional_updates.iter().position(same_identity) {
            Some(i) => self.provisional_updates[i] = update,
            None => self.provisional_updates.push(update),
        }
        Ok(())
    }
}
```

(`Vec<&ProvisionalUpdate>` encodes exactly as `Vec<ProvisionalUpdate>`:
serde serialises through the reference.)

`org-node/src/service.rs`:
- delete `admin_persona_for_org`; add

```rust
    /// The first Persona in store order bound to `org_id`, whatever its
    /// status (LLR-2xzys9, Decision 6).
    fn first_persona_bound_to(&self, org_id: OrgId) -> Result<&PersonaRecord, OrgNodeError> {
        self.store
            .data()
            .personas
            .iter()
            .find(|p| p.org_id == Some(org_id))
            .ok_or_else(|| OrgNodeError::Chain(format!("no persona bound to organisation {org_id:?}")))
    }
```

  and use it wherever `admin_persona_for_org(org_id)?.persona_id` was read
  (admit, revoke).
- `create_organisation` and the first-admission `OrgRecord` literal drop
  `admin_member_key`; `receive_and_verify` drops the `admin_member_key`
  computation T5 left.
- In `receive_and_verify`, the Persona choice loses the exclusion: the
  `find_map` closure no longer compares the Persona's member key with
  anything; it binds the first Persona whose DevicePublicKey is in the
  verified trie (LLR-e5c9ud as amended).

**Step 5 — green.** The org-node line: provisional_store 6, persona_records
one more, value_types one more, absences 2, admission_sender one more,
encoding_golden as after T7. Commit: `T8: no administrator key; provisional-update store and bound`.

**Red→green attestations:** (DONE 2026-10-06, merged; org-node cargo line
205 passed, 0 failed; provisional_store has 5 tests (the 6th the plan counted
lives in persona_records). App `cargo check` first error unchanged (E0432
`org_node::blobs`); app/src-tauri/Cargo.lock restored. Genesis grouping field:
`ProvisionalUpdate.persona_id: PersonaId`.)

- persona_store_plaintext_is_pinned (re-pinned 612 → 938 bytes, derived by script from T7's pin — cut 32 bytes at 266, insert `02‖genesis(224)‖change set(133)` at 527 — byte-equal to the plan's value) — runtime-red against unchanged code (`DeserializeUnexpectedEnd`); green
- provisional_updates_round_trip_through_the_encrypted_file — compile-red; runtime-red under a temporary `TryFrom` dropping the field; green
- an_update_with_the_same_identity_replaces_and_others_are_kept — compile-red; runtime-red against a push-unconditionally stub; green
- the_genesis_private_key_lives_only_in_its_provisional_update — compile-red only: the property is the type's layout (the key exists only inside `ProvisionalChange::Genesis`), which no natural stub violates
- a_group_of_exactly_the_bound_is_kept — compile-red; runtime-red against an off-by-one mutant (`>=`); green on `>`
- an_update_that_would_exceed_the_bound_is_refused_and_nothing_changes — compile-red; runtime-red against the push stub; green
- a_store_with_an_invalid_provisional_update_is_refused_naming_the_field — compile-red; runtime-red under the drop-on-open `TryFrom`; green
- a_store_with_every_record_kind_opens_with_every_field_parsed (rewritten) — compile-red; runtime-red under the drop-on-open `TryFrom`; green
- a_store_with_any_one_field_invalid_is_refused_naming_exactly_that_field, a_store_with_two_invalid_fields_in_one_record_reports_the_first_in_record_order (rewritten) — compile-red only (removing the field is the change)
- every_rejection_variant_is_distinct_from_every_other (ProvisionalLimit added) — compile-red only (derived `PartialEq`)
- the_provisional_limit_refusal_names_the_limit — compile-red; runtime-red against a placeholder message; green
- org_node_has_no_administrator_key — runtime-red against the stubs (`src/store.rs contains admin_member_key`); green
- a_persona_whose_member_key_the_chain_publishes_is_still_bound — runtime-red (`AdmissionNotOurs`, as predicted); green once the exclusion went
- the_persona_marked_active_is_the_first_whose_device_is_in_the_record (rewritten, renamed) — runtime-red (founder's `member_id` None); green
- a_first_admission_records_the_chains_organisation_public_key, a_first_admission_records_the_chains_key_the_secret_and_the_member, member_ids_are_not_derived_from_keys, the_admission_envelope_carries_the_epoch_its_update_produced, a_second_organisation_is_admitted_into_without_touching_the_first (rewritten) — compile-red only (E0609 `admin_member_key`); each drops or replaces an assertion on the removed field, the rest passed before and after

Left for T17: a comment at `org-node/tests/service_stories.rs:328` still names
the old administrator lookup.

---

### T9 — org-node: genesis is provisional; `commit_genesis`; the Organisation private key waits in the update

**Files touched:** `org-node/src/service.rs`, `org-node/src/error.rs`,
`org-node/tests/support/mod.rs`, `org-node/tests/commit_paths.rs` (new),
`org-node/tests/service_lifecycle.rs`, `org-node/tests/organisation_key.rs`,
`org-node/tests/value_types.rs`, `org-node/tests/admission_sender.rs`,
`org-node/tests/service_stories.rs`, `org-node/Cargo.toml`,
`org-node/.guardrails/config.yaml`
**Parallel:** no (serial, after T8)
**IDs verified:** REQ-xs4ab8, REQ-tqap3r, REQ-uv3v5w (genesis), REQ-d9g6nt,
REQ-ech45n, LLR-s6qnht, LLR-wzqqg9, LLR-qjz3q4, LLR-sj7cd5, LLR-3fwykc,
LLR-2dvhz8, LLR-68yd3j, LLR-rjg3m2, LLR-w3fhhg, LLR-q3aj8z, LLR-dzte8x,
LLR-3v5nu9 (accessor), LLR-nvn3wk, LLR-ewkg85 (genesis half), LLR-4tcxsu
(genesis half), LLR-mkj4bz (genesis half), LLR-ryzr8m (`apply_genesis`),
LLR-mxskg9 (`NoProvisionalUpdate`).
**Size:** ~520 lines.

**Step 1 — failing tests.** `support/mod.rs`:

```rust
/// The proxy account the app hands `commit_genesis` in these tests.
pub fn test_proxy() -> ChainAccount {
    ChainAccount::new([0x5a; 32])
}

/// Story 1: `pid` founds an Organisation — the genesis update is built, the
/// app's chain write is stood in for by the mock, and the update committed.
pub async fn found(svc: &mut OrgService, chain: &MockChainOps, pid: &PersonaId) -> OrgId {
    let update = svc.create_organisation(&mut OsRng, pid).expect("build the genesis update");
    let org_id = chain.apply_genesis(update.resulting_root, update.org_pub_key);
    svc.commit_genesis(&mut OsRng, pid, org_id, test_proxy()).await.expect("commit genesis");
    org_id
}
```

In `CountingChain` delete the `submit_genesis` method.

Create `org-node/tests/commit_paths.rs`:

```rust
#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The node's own provisional updates and their commit (REQ-xs4ab8,
//! REQ-tqap3r, REQ-uv3v5w): built and kept without touching the chain or the
//! record, committed only once they verify against the chain.

mod support;

use org_node::chain::OrgState;
use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::service::{MockChainOps, OrgService};
use org_node::store::{PersonaStatus, PersonaStore, ProvisionalChange, ProvisionalUpdate};
use org_node::test_fixtures::org_public_key;
use org_node::{Epoch, PersonaId, RootHash, SequenceNumber};
use rand::rngs::OsRng;
use support::*;

/// A Persona on a fresh service over `chain`, and its genesis update.
fn genesis_built(tag: &str, chain: &MockChainOps) -> (OrgService, PersonaId, ProvisionalUpdate) {
    let mut svc = OrgService::new(open_store(tag, "a", "pw_a"), Box::new(chain.clone()));
    let pid = svc.create_persona(&mut OsRng, h("alice"), nm("Alice"), sn("Smith")).unwrap();
    let update = svc.create_organisation(&mut OsRng, &pid).unwrap();
    (svc, pid, update)
}

fn private_key_of(update: &ProvisionalUpdate) -> org_node::OrgPrivateKey {
    match &update.change {
        ProvisionalChange::Genesis { org_private_key, .. } => org_private_key.clone(),
        ProvisionalChange::ChangeSet { .. } => panic!("a genesis update"),
    }
}

// Normal: creation keeps a genesis update — no record, no binding, no chain
// call, no endpoint — and the update states what the app will publish: the
// public key of the private key it holds, at epoch 1.
// verifies: REQ-xs4ab8, REQ-ech45n, LLR-s6qnht, LLR-qjz3q4, LLR-68yd3j, LLR-nvn3wk
#[test]
fn create_organisation_keeps_a_genesis_update_and_nothing_else() {
    let chain = MockChainOps::new();
    let (svc, pid, update) = genesis_built("create", &chain);
    let persona = persona_of(&svc, &pid);
    let (member_key, device_key) = svc.persona_public_keys(&pid).unwrap();
    assert_eq!((update.org_id, update.base_root, update.seq), (None, None, SequenceNumber::new(1)));
    assert_eq!(update.persona_id, pid);
    assert_eq!(private_key_of(&update).x25519_keypair().org_public_key().unwrap(), update.org_pub_key);
    assert_ne!(update.org_pub_key.as_bytes(), member_key.as_bytes(), "a fresh key, not the founder's (REQ-ech45n)");
    let ProvisionalChange::Genesis { members, .. } = &update.change else { panic!("a genesis change") };
    assert_eq!(members.len(), 1);
    assert_eq!((members[0].member_key, members[0].device_keys.clone()), (member_key, vec![device_key]));
    assert!(svc.list_orgs().is_empty(), "no record");
    assert_eq!((persona.status, persona.org_id), (PersonaStatus::Proposed, None), "no binding");
    assert!(svc.endpoint().is_none());
    assert_eq!(svc.genesis_provisional_updates(&pid), vec![update.clone()]);
    let disk = reopen_store("create", "a", "pw_a");
    assert_eq!(disk.data().provisional_updates, vec![update]);
    assert!(disk.data().orgs.is_empty());
}

// Abnormal (LLR-nvn3wk): reading provisional updates changes nothing, and an
// unknown Persona or Organisation has none.
// verifies: LLR-nvn3wk
#[test]
fn reading_provisional_updates_changes_nothing() {
    let chain = MockChainOps::new();
    let (svc, _pid, _update) = genesis_built("read-only", &chain);
    let before = store_bytes("read-only", "a");
    assert!(svc.genesis_provisional_updates(&PersonaId::new("nobody".into())).is_empty());
    assert!(svc.provisional_updates(OrgId::new([9; 20])).is_empty());
    assert_eq!(store_bytes("read-only", "a"), before);
}

// Normal and abnormal (REQ-d9g6nt): the founding Member's id is drawn, not
// derived — the same Persona founding twice gets two ids, neither a key —
// and each genesis has its own Organisation key pair.
// verifies: REQ-d9g6nt, REQ-ech45n, LLR-rjg3m2
#[test]
fn the_founding_member_id_and_the_organisation_key_are_drawn_not_derived() {
    let chain = MockChainOps::new();
    let (mut svc, pid, first) = genesis_built("ids", &chain);
    let second = svc.create_organisation(&mut OsRng, &pid).unwrap();
    let id_of = |u: &ProvisionalUpdate| match &u.change {
        ProvisionalChange::Genesis { members, .. } => (members[0].id, members[0].member_key),
        ProvisionalChange::ChangeSet { .. } => panic!("genesis"),
    };
    let ((a, key), (b, _)) = (id_of(&first), id_of(&second));
    assert_ne!(a, b);
    assert_ne!(a.as_bytes(), key.as_bytes());
    assert_ne!(first.org_pub_key, second.org_pub_key);
    assert_eq!(svc.genesis_provisional_updates(&pid).len(), 2, "both kept");
}

// Normal: once the chain carries the root and key at epoch 1, commit_genesis
// creates the record with the chain's values, mark 1, the private key and the
// proxy account, binds the Persona, consumes the update, writes the store,
// and binds no endpoint.
// verifies: REQ-tqap3r, REQ-uv3v5w, LLR-wzqqg9, LLR-qjz3q4, LLR-3fwykc, LLR-w3fhhg, LLR-q3aj8z, LLR-dzte8x, LLR-3v5nu9, LLR-4tcxsu, LLR-mkj4bz
#[tokio::test]
async fn commit_genesis_creates_the_record_once_the_chain_carries_the_root() {
    let chain = MockChainOps::new();
    let (mut svc, pid, update) = genesis_built("commit-genesis", &chain);
    let org_id = chain.apply_genesis(update.resulting_root, update.org_pub_key);
    let outcome = svc.commit_genesis(&mut OsRng, &pid, org_id, test_proxy()).await.unwrap();
    assert_eq!((outcome.org_id, outcome.epoch, outcome.root), (org_id, Epoch::new(1), update.resulting_root));
    let rec = rec_of(&svc, org_id);
    assert_eq!(
        (rec.root_hash, rec.org_pub_key, rec.epoch, rec.last_seq),
        (update.resulting_root, update.org_pub_key, Epoch::new(1), SequenceNumber::new(1))
    );
    assert_eq!(rec.org_secret, None);
    assert_eq!(rec.org_private_key, Some(private_key_of(&update)));
    assert_eq!(rec.proxy_account, Some(test_proxy()));
    assert_eq!(svc.proxy_account(org_id).unwrap(), Some(test_proxy()));
    let p = persona_of(&svc, &pid);
    assert_eq!((p.status, p.org_id, p.member_id), (PersonaStatus::Active, Some(org_id), None));
    assert!(svc.genesis_provisional_updates(&pid).is_empty(), "the update is consumed");
    assert!(svc.endpoint().is_none(), "a commit binds no endpoint");
    let disk = reopen_store("commit-genesis", "a", "pw_a");
    assert_eq!(disk.data().orgs[0].proxy_account, Some(test_proxy()));
    assert!(disk.data().provisional_updates.is_empty());
}

// Abnormal: each refusal of commit_genesis returns its error and leaves the
// record list, the update and the Persona exactly as they were, unwritten.
// verifies: REQ-tqap3r, LLR-wzqqg9, LLR-ewkg85, LLR-mxskg9
#[tokio::test]
async fn commit_genesis_refusals_change_nothing_and_write_nothing() {
    let chain = MockChainOps::new();
    let (mut svc, pid, update) = genesis_built("genesis-refused", &chain);
    let state = |root, key, epoch| OrgState { root_hash: root, org_pub_key: key, epoch: Epoch::new(epoch) };
    let unknown = OrgId::new([0x99; 20]);
    let (other_root, other_key, at_zero, at_two) =
        (OrgId::new([0x98; 20]), OrgId::new([0x97; 20]), OrgId::new([0x96; 20]), OrgId::new([0x95; 20]));
    chain.set(other_root, state(RootHash::new([0x01; 32]), update.org_pub_key, 1));
    chain.set(other_key, state(update.resulting_root, org_public_key(), 1));
    chain.set(at_zero, state(update.resulting_root, update.org_pub_key, 0));
    chain.set(at_two, state(update.resulting_root, update.org_pub_key, 2));
    let before = store_bytes("genesis-refused", "a");
    for (org, expected) in [
        (unknown, OrgNodeError::OrgNotOnChain),
        (other_root, OrgNodeError::NoProvisionalUpdate),
        (other_key, OrgNodeError::NoProvisionalUpdate),
        (at_zero, OrgNodeError::SeqNotEpoch { seq: 1, epoch: 0 }),
        (at_two, OrgNodeError::SeqNotEpoch { seq: 1, epoch: 2 }),
    ] {
        assert_eq!(svc.commit_genesis(&mut OsRng, &pid, org, test_proxy()).await.unwrap_err(), expected);
        assert!(svc.list_orgs().is_empty());
        assert_eq!(svc.genesis_provisional_updates(&pid), vec![update.clone()]);
        assert_eq!(persona_of(&svc, &pid).status, PersonaStatus::Proposed);
        assert_eq!(store_bytes("genesis-refused", "a"), before, "nothing written");
    }
}

// Abnormal: a stored genesis update whose members do not rebuild to the root
// the chain carries is refused with RootMismatch.
// verifies: LLR-wzqqg9, LLR-ewkg85
#[tokio::test]
async fn a_genesis_whose_members_do_not_rebuild_the_root_is_refused() {
    let chain = MockChainOps::new();
    let (svc, pid, update) = genesis_built("genesis-mismatch", &chain);
    drop(svc);
    let forged = ProvisionalUpdate { resulting_root: RootHash::new([0x42; 32]), ..update };
    let mut store = PersonaStore::open(store_dir("genesis-mismatch", "a").join("store.bin"), "pw_a").unwrap();
    store.data_mut().insert_provisional(forged.clone()).unwrap();
    store.save(&mut OsRng).unwrap();
    let mut svc = OrgService::new(store, Box::new(chain.clone()));
    let org = chain.apply_genesis(forged.resulting_root, forged.org_pub_key);
    let before = store_bytes("genesis-mismatch", "a");
    assert_eq!(svc.commit_genesis(&mut OsRng, &pid, org, test_proxy()).await.unwrap_err(), OrgNodeError::RootMismatch);
    assert!(svc.list_orgs().is_empty());
    assert_eq!(store_bytes("genesis-mismatch", "a"), before);
}

// Abnormal: a chain read that fails is refused as `Chain`, nothing written.
// verifies: LLR-ewkg85
#[tokio::test]
async fn a_failed_chain_read_refuses_commit_genesis() {
    let counting = CountingChain::over(MockChainOps::new());
    counting.fail_reads();
    let mut svc = OrgService::new(open_store("genesis-chain-fails", "a", "pw_a"), Box::new(counting.clone()));
    let pid = svc.create_persona(&mut OsRng, h("alice"), nm("Alice"), sn("Smith")).unwrap();
    svc.create_organisation(&mut OsRng, &pid).unwrap();
    let before = store_bytes("genesis-chain-fails", "a");
    assert!(matches!(svc.commit_genesis(&mut OsRng, &pid, OrgId::new([1; 20]), test_proxy()).await, Err(OrgNodeError::Chain(_))));
    assert_eq!(store_bytes("genesis-chain-fails", "a"), before);
}
```

Cargo: `[[test]] name = "commit_paths"`, `path = "tests/commit_paths.rs"`,
`required-features = ["app", "test-support"]`; config: ` --test commit_paths`.

`organisation_key.rs` (master's REQ-ech45n tests; each keeps its annotation):

| Test | Change in T9 |
|---|---|
| `a_created_organisation_publishes_a_fresh_key_no_genesis_key_equals` (REQ-ech45n, LLR-sj7cd5, LLR-3fwykc) | `create_organisation` returns the genesis update (no `.await`); the key under test is `update.org_pub_key`; after `chain.apply_genesis` and `commit_genesis` (the file's own proxy value), the record's and the chain's key equal it and the record's private key's public half equals it. |
| `two_organisations_of_one_persona_publish_two_keys` (REQ-ech45n) | two `create_organisation` calls; the two updates' keys differ. |
| `the_organisation_private_key_is_kept_only_in_the_encrypted_store` (REQ-ech45n, LLR-3fwykc) | the raw file contains no private-key bytes both after `create_organisation` (the key is in the update) and after `commit_genesis` (in the record); add `LLR-qjz3q4` to its annotation. |
| `the_organisation_private_key_is_not_in_the_record_debug_output` (LLR-2dvhz8) | found through `create_organisation` + `apply_genesis` + `commit_genesis`. |
| `an_organisation_key_equal_to_a_genesis_key_is_refused` (REQ-ech45n, LLR-sj7cd5) | `create_organisation` (no `.await`) returns `Trie(DuplicateKey)`; assert `svc.genesis_provisional_updates(&pid).is_empty()` and the store file unchanged in place of "nothing submitted"; the accepting half keeps one update. |
| its two `ChainOps` substitutes | delete their `submit_genesis` methods. |
| `a_receive_against_a_state_with_an_invalid_key_commits_nothing` (REQ-8jb4ny) | A founds through `create_organisation` + `apply_genesis` + `commit_genesis`. |

`service_lifecycle.rs`:
- `creating_an_organisation_advances_the_chain_and_activates_the_persona`
  (LLR-68yd3j, LLR-q3aj8z) → rename
  `creating_an_organisation_writes_only_a_genesis_update`;
  `// verifies: LLR-68yd3j, REQ-xs4ab8`; body: `create_organisation` (no
  `.await`); assert `svc.list_orgs().is_empty()`, the Persona `Proposed`
  with `member_id == None`, the chain unchanged (no slot), and on disk `orgs`
  empty and `provisional_updates.len() == 1`. LLR-q3aj8z moves to
  `commit_genesis_creates_the_record_once_the_chain_carries_the_root`.
- `the_seam_is_a_trait_object_a_substitute_can_stand_in_for`: drop `.await`
  from `create_organisation` and delete its substitute's `submit_genesis`.
- add:

```rust
// Normal: the stand-in for the genesis write makes a slot at epoch one and
// returns its id; abnormal: a second genesis with the same root gets its own
// slot and leaves the first as it was.
// verifies: LLR-ryzr8m
#[test]
fn the_mock_genesis_makes_a_slot_at_epoch_one() {
    let chain = MockChainOps::new();
    let root = RootHash::new([3u8; 32]);
    let first = chain.apply_genesis(root, org_key());
    assert_eq!(chain.get(&first).unwrap(), OrgState { root_hash: root, org_pub_key: org_key(), epoch: Epoch::new(1) });
    let second = chain.apply_genesis(root, org_key());
    assert_ne!(first, second);
    assert_eq!(chain.get(&first).unwrap().epoch, Epoch::new(1));
}
```

`value_types.rs`: add `OrgNodeError::NoProvisionalUpdate` to the distinct list.

`admission_sender.rs`, `service_stories.rs`: every remaining direct
`create_organisation(...).await` becomes `found(...)` (with
`service_lifecycle.rs` and `organisation_key.rs`, above, these are the only
test files that call it: `grep -ln create_organisation org-node/tests/*.rs`).
In particular:
- `same_persona_founding_two_organisations_gets_two_admin_ids` → rename
  `same_persona_founding_two_organisations_gets_two_member_ids`; body via
  `found` twice; annotation unchanged (REQ-d9g6nt, LLR-rjg3m2).
- `pr_mdv38y_founding_an_organisation_rebinds_a_member_persona` (LLR-w3fhhg,
  LLR-q3aj8z): via `found`; its assertions (the binding is overwritten, the
  member id kept) are now `commit_genesis`'s, unchanged.
- `the_proxy_account_from_genesis_is_kept_and_passed_on_every_update`
  (LLR-dzte8x, LLR-3v5nu9, LLR-drgdy8): delete `ProxyChain::submit_genesis`;
  found with `create_organisation` + `chain.inner.apply_genesis` +
  `commit_genesis(&mut OsRng, &pid_a, org_id, proxy())`; the two
  `proxy_account` assertions stay and gain
  `assert_eq!(svc_a.proxy_account(org_id).unwrap(), Some(proxy()));`.
- `the_admission_envelope_carries_the_epoch_its_update_produced` (LLR-ghja3x,
  REQ-txvtm9): A's record after genesis now has mark 1 = epoch 1; the
  admission's Sequence number is 2 either way — no assertion changes, but
  check the "record mark before" value the test reads (1, not 0).
- `service_stories::five_stories_full_e2e`: drop `LLR-68yd3j` from its
  annotation (creation no longer activates anything; the new statement is
  verified in `commit_paths` and `service_lifecycle`).

Any test that asserted a freshly created record's `last_seq == 0` now expects
1 (Decision 16); `grep -n "last_seq" org-node/tests/*.rs` lists them.

**Step 2 — run, expect red.** Compile errors (`create_organisation` returns
`OrgId` and is async; no `commit_genesis`, `apply_genesis`,
`genesis_provisional_updates`, `provisional_updates`, `proxy_account`,
`NoProvisionalUpdate`). With stubs (`commit_genesis` returning
`Err(NoProvisionalUpdate)`, accessors returning empty), run: the commit and
creation tests FAIL on their assertions. Record.

**Step 3 — implement** (`org-node/src/service.rs`; `NoProvisionalUpdate` in
`error.rs` with `#[error("no provisional update produces the state the chain carries")]`):

- `ChainOps` loses `submit_genesis` (and its doc); `MockChainOps` loses its
  `submit_genesis` and gains `apply_genesis` (LLR-ryzr8m):

```rust
    /// Stand-in for the genesis chain write the app makes through
    /// on-chain-client: a slot at epoch one holding `root` and `key`, under an
    /// id derived from the root and a counter (LLR-ryzr8m).
    pub fn apply_genesis(&self, root: RootHash, key: OrgPublicKey) -> OrgId {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        let mut id_bytes = [0u8; 20];
        id_bytes.copy_from_slice(&root.as_bytes()[..20]);
        id_bytes[0] ^= g.next_id_seed;
        g.next_id_seed = g.next_id_seed.wrapping_add(1);
        let org_id = OrgId::new(id_bytes);
        g.slots.insert(org_id, OrgState { root_hash: root, org_pub_key: key, epoch: Epoch::new(1) });
        org_id
    }
```

- `SubxtChainOps` loses `submit_genesis`, the `proxy_map` field and its
  fallback in `submit_update`, which now takes the account from its argument
  or refuses: `let p = proxy_account.ok_or_else(|| OrgNodeError::Chain("no
  proxy account in the record".into()))?;`. Remove the `genesis_ceremony`
  import.
- `create_organisation` (now `pub fn`, no `async`):

```rust
    /// Build the genesis provisional update for `persona_id` and keep it: a
    /// fresh Organisation key pair whose private key the update holds, no
    /// chain operation, no record, no binding (LLR-s6qnht, LLR-qjz3q4,
    /// LLR-68yd3j, LLR-sj7cd5).
    pub fn create_organisation<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        persona_id: &PersonaId,
    ) -> Result<ProvisionalUpdate, OrgNodeError> {
        let (member_kp, device_kp, handle, name, surname) = self.persona_keys(persona_id)?;
        let founder = MemberLeaf::new(
            fresh_member_id(rng),
            handle,
            member_kp.member_key()?,
            name,
            surname,
            vec![device_kp.device_key()?],
        )
        .map_err(OrgNodeError::Trie)?;
        let trie = Trie::genesis(vec![founder.clone()]).map_err(OrgNodeError::Trie)?;
        // REQ-ech45n: drawn for this Organisation alone, its public key equal
        // to no key of the genesis record (LLR-sj7cd5).
        let org_kp = X25519Keypair::generate(rng);
        let org_pub_key = org_kp.org_public_key()?;
        org_pub_key.ensure_distinct_from(&trie.members())?;
        let update = ProvisionalUpdate {
            org_id: None,
            persona_id: persona_id.clone(),
            base_root: None,
            resulting_root: trie.root_hash().map_err(OrgNodeError::Trie)?,
            // Genesis produces epoch 1 on the contract (Decision 16).
            seq: SequenceNumber::new(1),
            org_pub_key,
            change: ProvisionalChange::Genesis {
                members: vec![snapshot_of(&founder)],
                org_private_key: org_kp.org_private_key(),
            },
        };
        self.store.data_mut().insert_provisional(update.clone())?;
        self.store.save(rng)?;
        Ok(update)
    }
```

- `commit_genesis`:

```rust
    /// Commit the genesis update `persona_id` built, once the chain carries
    /// its root and key at epoch 1: read the state once, select the update,
    /// require the epoch to be the update's Sequence number and the members
    /// to rebuild the root, then create the record with the chain's values,
    /// mark 1, the update's private key and `proxy_account`, consume the
    /// update and bind the Persona (LLR-wzqqg9, LLR-qjz3q4, LLR-w3fhhg,
    /// LLR-dzte8x). Any refusal changes and writes nothing (LLR-ewkg85); no
    /// endpoint is bound (LLR-4tcxsu).
    pub async fn commit_genesis<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        persona_id: &PersonaId,
        org_id: OrgId,
        proxy_account: ChainAccount,
    ) -> Result<ReceiveOutcome, OrgNodeError> {
        let state = self.chain.read_state(org_id).await?.ok_or(OrgNodeError::OrgNotOnChain)?;
        let update = self
            .store
            .data()
            .provisional_updates
            .iter()
            .find(|u| {
                u.org_id.is_none()
                    && &u.persona_id == persona_id
                    && u.resulting_root == state.root_hash
                    && u.org_pub_key == state.org_pub_key
            })
            .cloned()
            .ok_or(OrgNodeError::NoProvisionalUpdate)?;
        if state.epoch.get() != update.seq.get() {
            return Err(OrgNodeError::SeqNotEpoch { seq: update.seq.get(), epoch: state.epoch.get() });
        }
        let ProvisionalChange::Genesis { members, org_private_key } = update.change.clone() else {
            return Err(OrgNodeError::NoProvisionalUpdate);
        };
        if trie_from_snapshots(&members)?.root_hash().map_err(OrgNodeError::Trie)? != state.root_hash {
            return Err(OrgNodeError::RootMismatch);
        }
        let data = self.store.data_mut();
        data.orgs.push(OrgRecord {
            org_id,
            root_hash: state.root_hash,
            org_pub_key: state.org_pub_key,
            epoch: state.epoch,
            org_secret: None,
            last_seq: update.seq,
            trie_members: members,
            proxy_account: Some(proxy_account),
            org_private_key: Some(org_private_key),
        });
        data.provisional_updates.retain(|u| *u != update);
        Self::discard_orphans(data, org_id, state.root_hash);
        self.update_persona_status(persona_id, org_id, PersonaStatus::Active)?;
        self.store.save(rng)?;
        Ok(ReceiveOutcome { org_id, epoch: state.epoch, root: state.root_hash })
    }

    /// After a commit to `org_id` at `root`, drop every provisional update for
    /// that Organisation built on another base (LLR-mkj4bz).
    fn discard_orphans(data: &mut StoreData, org_id: OrgId, root: RootHash) {
        data.provisional_updates.retain(|u| u.org_id != Some(org_id) || u.base_root == Some(root));
    }

    /// The stored provisional updates for `org_id` (LLR-nvn3wk).
    pub fn provisional_updates(&self, org_id: OrgId) -> Vec<ProvisionalUpdate> {
        self.store.data().provisional_updates.iter().filter(|u| u.org_id == Some(org_id)).cloned().collect()
    }

    /// The genesis updates `persona_id` built (LLR-nvn3wk).
    pub fn genesis_provisional_updates(&self, persona_id: &PersonaId) -> Vec<ProvisionalUpdate> {
        self.store
            .data()
            .provisional_updates
            .iter()
            .filter(|u| u.org_id.is_none() && &u.persona_id == persona_id)
            .cloned()
            .collect()
    }

    /// The proxy account the record holds, unchanged and uninterpreted
    /// (LLR-3v5nu9).
    pub fn proxy_account(&self, org_id: OrgId) -> Result<Option<ChainAccount>, OrgNodeError> {
        Ok(self.find_org(org_id)?.proxy_account)
    }
```

  Imports: `crate::store::{ProvisionalChange, ProvisionalUpdate, StoreData}`.
  The old `create_organisation` body and its comments go. (`retain(|u| *u !=
  update)` compares the private key too, which `OrgPrivateKey`'s `PartialEq`
  allows.)

**Step 4 — green.** The org-node line: commit_paths 7 passed,
service_lifecycle one more, value_types one more, organisation_key unchanged
in count, the rest as after T8. Commit:
`T9: genesis is provisional; commit_genesis; the private key waits in the update`.

**Red→green attestations:** (DONE 2026-10-06, merged; org-node cargo line
214 passed, 0 failed — commit_paths 8 (one beyond the plan, the owner's
late-genesis ruling), service_lifecycle +1. App `cargo check`: E0432
`org_node::blobs` first, then E0407 `submit_genesis` at `state.rs:74`;
app/src-tauri/Cargo.lock restored.)

- create_organisation_keeps_a_genesis_update_and_nothing_else — compile-red (E0599/E0308); runtime-red against stubs (update built but neither kept nor saved); green
- reading_provisional_updates_changes_nothing — compile-red; passes against the stubs; runtime-red under a mutant widening the genesis filter (`||`); green after restore
- the_founding_member_id_and_the_organisation_key_are_drawn_not_derived — compile-red; runtime-red against the stubs (0 ≠ 2); also killed by a fixed-org-key mutant; green
- commit_genesis_creates_the_record_once_the_chain_carries_the_root — compile-red; runtime-red against the stubs (`NoProvisionalUpdate`); green
- commit_genesis_refusals_change_nothing_and_write_nothing — compile-red; runtime-red against the stubs; also killed by an epoch-check mutant (`!=` → `<`); green
- a_chain_past_epoch_one_refuses_commit_genesis_and_keeps_the_update (owner ruling 2026-10-06) — compile-red; runtime-red against the stubs; killed by the epoch-check mutant (committed at epoch 3); green. Asserts both refusal kinds (see LLR-wzqqg9's ruling note)
- a_genesis_whose_members_do_not_rebuild_the_root_is_refused — compile-red; runtime-red against the stubs (`NoProvisionalUpdate` ≠ `RootMismatch`); green
- a_failed_chain_read_refuses_commit_genesis — compile-red; runtime-red against the stubs; green
- the_mock_genesis_makes_a_slot_at_epoch_one — compile-red (E0599 `apply_genesis`); runtime-red against a no-slot stub; green
- creating_an_organisation_writes_only_a_genesis_update (rewritten, renamed) — compile-red; runtime-red against the stubs; green
- the_seam_is_a_trait_object_a_substitute_can_stand_in_for (rewritten) — compile-red only: the property is object safety
- organisation_key: a_created_organisation_publishes_a_fresh_key_no_genesis_key_equals, the_organisation_private_key_is_kept_only_in_the_encrypted_store, the_organisation_private_key_is_not_in_the_record_debug_output, an_organisation_key_equal_to_a_genesis_key_is_refused, a_receive_against_a_state_with_an_invalid_key_commits_nothing (rewritten) — compile-red; runtime-red against the stubs; green
- two_organisations_of_one_persona_publish_two_keys (rewritten) — compile-red; passes against the stubs; runtime-red under a fixed-org-key mutant; green after restore
- every_rejection_variant_is_distinct_from_every_other (NoProvisionalUpdate added) — compile-red only (derived `PartialEq`)
- admission_sender rewrites (same_persona_founding_two_organisations_gets_two_member_ids, the_proxy_account_from_genesis_is_kept_and_passed_on_every_update, pr_mdv38y_…) and the support-routed tests — compile-red, then runtime-red through `found`; green

T10b fixtures: `genesis_built(tag, &chain) -> (OrgService, PersonaId,
ProvisionalUpdate)` (private to commit_paths.rs; store under party "a",
password "pw_a"), `reopen_store(tag, party, password)`, `store_dir(tag,
party).join("store.bin")` / `store_bytes(tag, party)`, `found(svc, chain,
pid)`, `test_proxy()`.

---

### T10 — org-node: admission is provisional; `commit_update`; `send_update`

**Files touched:** `org-node/src/service.rs`, `org-node/src/lib.rs`,
`org-node/tests/support/mod.rs`, `org-node/tests/commit_paths.rs`,
`org-node/tests/admission_sender.rs`, `org-node/tests/service_stories.rs`,
`org-node/tests/organisation_key.rs`
**Parallel:** no (serial, after T9)
**IDs verified:** REQ-xs4ab8, REQ-tqap3r, REQ-uv3v5w, REQ-fwfku9 (service
clause), REQ-txvtm9, REQ-8amu2a (the send), LLR-rb8r65, LLR-ghja3x,
LLR-vdyu65, LLR-cmdrp9, LLR-ewkg85, LLR-4tcxsu, LLR-mkj4bz, LLR-bg3vsw,
LLR-jn5jeh, LLR-t4znbk, LLR-2xzys9, LLR-48jakr, LLR-pw369n (admission side),
LLR-8hdu9x, LLR-ckk5nz, LLR-nvn3wk, LLR-jq7qh7 (no write on refusal),
LLR-cja9zv.
**Size:** ~480 lines.

**Step 1 — failing tests.** `support/mod.rs` — the admission story becomes
the app's sequence (build, chain write stood in for by the mock, commit,
send):

```rust
/// Story 3: admit `joiner` into `org_id`: build, write the chain (mock),
/// commit, send to `addr` under the joiner's invite identifier. Returns the
/// joiner's new MemberId.
pub async fn admit(
    svc: &mut OrgService,
    chain: &MockChainOps,
    org_id: OrgId,
    joiner: &JoinerOf,
    addr: iroh::EndpointAddr,
    secret: Option<OrgSecret>,
) -> Result<MemberId, OrgNodeError> {
    let update = svc.admit_member(&mut OsRng, org_id, joiner)?;
    chain.apply_update(org_id, update.resulting_root, update.org_pub_key, rec_of(svc, org_id).epoch)?;
    let outcome = svc.commit_update(&mut OsRng, org_id).await?;
    let id = rec_of(svc, org_id)
        .trie_members
        .iter()
        .find(|m| m.member_key == joiner.member_key)
        .map(|m| m.id)
        .expect("the joiner is in the committed record");
    let invite = Some(invite_for(&joiner.device_key));
    tokio::time::timeout(NET, svc.send_update(&outcome.outgoing, joiner.device_key, Some(addr), secret, invite))
        .await
        .expect("send timed out")?;
    Ok(id)
}
```

Append to `commit_paths.rs`:

```rust
use org_node::store::MemberSnapshot;
use org_node::test_fixtures::{device_key, member_key};
use org_node::transport::wire::WireMessage;
use org_node::{Envelope, InviteId, Joiner};

/// Another joiner, from fixed seeds.
fn joiner_from(seed: u8, handle: &str) -> Joiner {
    Joiner { handle: h(handle), name: nm("Test"), surname: sn("Joiner"), member_key: member_key(seed), device_key: device_key(seed + 1) }
}

/// A founded Organisation (A) and a joiner (B's Persona) not yet admitted.
async fn founded(tag: &str) -> (MockChainOps, OrgService, OrgId, JoinerOf, OrgService) {
    let chain = MockChainOps::new();
    let mut a = OrgService::new(open_store(tag, "a", "pw_a"), Box::new(chain.clone()));
    let pid_a = a.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User")).unwrap();
    let org = found(&mut a, &chain, &pid_a).await;
    let mut b = OrgService::new(open_store(tag, "b", "pw_b"), Box::new(chain.clone()));
    let pid_b = b.create_persona(&mut OsRng, h("bob"), nm("Bob"), sn("Builder")).unwrap();
    let joiner = joiner_of(&b, &pid_b);
    (chain, a, org, joiner, b)
}

// Normal: admission keeps a provisional update — base the record's root,
// Sequence number the epoch it produces, the record's key — and changes
// nothing else; a second admission before the first commits is built on the
// same record.
// verifies: REQ-xs4ab8, REQ-txvtm9, LLR-rb8r65, LLR-ghja3x, LLR-nvn3wk
#[tokio::test]
async fn admit_member_keeps_a_provisional_update_and_changes_nothing_else() {
    let (chain, mut a, org, joiner, _b) = founded("admit-provisional").await;
    let rec = rec_of(&a, org);
    let update = a.admit_member(&mut OsRng, org, &joiner).unwrap();
    assert_eq!(update.org_id, Some(org));
    assert_eq!(update.base_root, Some(rec.root_hash));
    assert_eq!(update.seq, SequenceNumber::new(rec.epoch.get() + 1), "the epoch it produces");
    assert_eq!(update.seq, SequenceNumber::new(rec.last_seq.get() + 1), "the mark equals the epoch");
    assert_eq!(update.org_pub_key, rec.org_pub_key);
    assert_ne!(update.resulting_root, rec.root_hash);
    assert!(matches!(update.change, ProvisionalChange::ChangeSet { .. }));
    assert_eq!(chain.get(&org).unwrap().epoch, Epoch::new(1), "no chain write");
    assert_eq!(rec_of(&a, org).trie_members.len(), rec.trie_members.len(), "record unchanged");
    assert!(a.endpoint().is_none());
    let second = a.admit_member(&mut OsRng, org, &joiner_from(0x61, "carol")).unwrap();
    assert_eq!((second.base_root, second.seq), (update.base_root, update.seq), "built on the same record");
    assert_eq!(a.provisional_updates(org).len(), 2);
}

// Normal: a provisional update the chain carries is committed as a received
// update would be, and the outgoing update is the Envelope and the record as
// it stood before. No endpoint is bound.
// verifies: REQ-tqap3r, LLR-cmdrp9, LLR-bg3vsw, LLR-4tcxsu, LLR-cja9zv
#[tokio::test]
async fn commit_update_commits_a_provisional_update_the_chain_carries() {
    let (chain, mut a, org, joiner, _b) = founded("commit-update").await;
    let before = rec_of(&a, org);
    let update = a.admit_member(&mut OsRng, org, &joiner).unwrap();
    chain.apply_update(org, update.resulting_root, update.org_pub_key, before.epoch).unwrap();
    let out = a.commit_update(&mut OsRng, org).await.unwrap();
    assert_eq!((out.org_id, out.epoch, out.root), (org, Epoch::new(2), update.resulting_root));
    let ProvisionalChange::ChangeSet { change_set } = update.change.clone() else { panic!() };
    assert_eq!(out.outgoing.envelope, Envelope { org_id: org, parent_seq: update.seq, delta_bytes: change_set });
    let sent: Vec<MemberSnapshot> = postcard::from_bytes(&out.outgoing.record_snapshot).unwrap();
    assert_eq!(sent, before.trie_members, "the record as it stood before the commit");
    let after = rec_of(&a, org);
    assert_eq!((after.root_hash, after.epoch, after.last_seq), (update.resulting_root, Epoch::new(2), update.seq));
    assert_eq!(after.trie_members.len(), 2);
    assert_eq!(reopen_store("commit-update", "a", "pw_a").data().orgs[0].epoch, Epoch::new(2));
    assert!(a.endpoint().is_none(), "a commit binds no endpoint and sends nothing");
}

// Abnormal: every refusal of commit_update returns its error and leaves the
// record and the provisional updates as they were, nothing written.
// verifies: REQ-tqap3r, LLR-cmdrp9, LLR-ewkg85
#[tokio::test]
async fn commit_update_refusals_change_nothing_and_write_nothing() {
    let (chain, mut a, org, joiner, _b) = founded("update-refused").await;
    let update = a.admit_member(&mut OsRng, org, &joiner).unwrap();
    let rec = rec_of(&a, org);
    let before = store_bytes("update-refused", "a");
    let check = |a: &OrgService| {
        assert_eq!(rec_of(a, org).root_hash, rec.root_hash);
        assert_eq!(a.provisional_updates(org), vec![update.clone()]);
        assert_eq!(store_bytes("update-refused", "a"), before, "nothing written");
    };
    // no record
    assert_eq!(a.commit_update(&mut OsRng, OrgId::new([0x99; 20])).await.unwrap_err(), OrgNodeError::OrgNotOnChain);
    check(&a);
    // the chain carries a root no provisional update produces (still genesis)
    assert_eq!(a.commit_update(&mut OsRng, org).await.unwrap_err(), OrgNodeError::NoProvisionalUpdate);
    check(&a);
    // the chain carries the root but at the epoch already committed
    chain.set(org, OrgState { root_hash: update.resulting_root, org_pub_key: update.org_pub_key, epoch: rec.epoch });
    assert_eq!(
        a.commit_update(&mut OsRng, org).await.unwrap_err(),
        OrgNodeError::StaleEpoch { got: rec.epoch.get(), last: rec.epoch.get() }
    );
    check(&a);
    // the chain carries the root two epochs on: not the update's Sequence number
    chain.set(org, OrgState { root_hash: update.resulting_root, org_pub_key: update.org_pub_key, epoch: Epoch::new(rec.epoch.get() + 2) });
    assert_eq!(
        a.commit_update(&mut OsRng, org).await.unwrap_err(),
        OrgNodeError::SeqNotEpoch { seq: update.seq.get(), epoch: rec.epoch.get() + 2 }
    );
    check(&a);
}

// Abnormal: a chain read that fails or finds no state refuses the commit.
// verifies: LLR-ewkg85
#[tokio::test(flavor = "multi_thread")]
async fn commit_update_refuses_when_the_chain_fails_or_is_silent() {
    let (s, counting) = setup_counted("update-chain-fails").await;
    let s = admit_b_directly(s).await;
    let mut b = s.svc_b;
    b.admit_member(&mut OsRng, s.org_id, &joiner_from(0x61, "carol")).unwrap();
    counting.hide(s.org_id);
    assert_eq!(b.commit_update(&mut OsRng, s.org_id).await.unwrap_err(), OrgNodeError::OrgNotOnChain);
    counting.fail_reads();
    assert!(matches!(b.commit_update(&mut OsRng, s.org_id).await, Err(OrgNodeError::Chain(_))));
    assert_eq!(b.provisional_updates(s.org_id).len(), 1);
}

// Normal: a commit discards every provisional update for that Organisation
// built on the old base and keeps other Organisations' — on the node's own
// commit and on a received one.
// verifies: REQ-uv3v5w, LLR-mkj4bz
#[tokio::test(flavor = "multi_thread")]
async fn a_commit_discards_the_provisional_updates_it_orphans() {
    let mut s = admit_b_directly(setup("orphans").await).await;
    let joiner_c = joiner_for_c(&mut s.svc_a);
    // B builds an admission of its own (B is bound to the Organisation).
    s.svc_b.admit_member(&mut OsRng, s.org_id, &joiner_from(0x71, "dora")).unwrap();
    // A builds two on the same base, the chain takes the first.
    let first = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_c).unwrap();
    s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_from(0x73, "erin")).unwrap();
    // A second Organisation on A's node, with a pending admission of its own.
    let pid_a2 = s.svc_a.create_persona(&mut OsRng, h("admin2"), nm("Admin"), sn("Two")).unwrap();
    let org_2 = found(&mut s.svc_a, &s.chain, &pid_a2).await;
    s.svc_a.admit_member(&mut OsRng, org_2, &joiner_from(0x75, "fay")).unwrap();
    s.chain.apply_update(s.org_id, first.resulting_root, first.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    let out = s.svc_a.commit_update(&mut OsRng, s.org_id).await.unwrap();
    assert!(s.svc_a.provisional_updates(s.org_id).is_empty(), "both built on the old base are gone");
    assert_eq!(s.svc_a.provisional_updates(org_2).len(), 1, "another Organisation's are kept");
    // B receives A's commit: its own provisional update is orphaned too.
    let (b_addr, b_task) = spawn_receive(s.svc_b, &s.b_device_kp).await;
    deliver(b_addr, &WireMessage { envelope: out.outgoing.envelope, org_secret: None, genesis_snapshot: Some(out.outgoing.record_snapshot), invite_id: None }).await;
    let (svc_b, result) = b_task.await.unwrap();
    result.unwrap();
    assert!(svc_b.provisional_updates(s.org_id).is_empty());
}

// Normal: send_update sends the committed Envelope, the earlier snapshot,
// exactly the secret and the invite identifier it is given, under the first
// bound Persona's device, to the full address in Loopback mode, and writes
// nothing.
// verifies: LLR-2xzys9, LLR-48jakr, LLR-8hdu9x, LLR-jn5jeh, LLR-t4znbk, REQ-8amu2a
#[tokio::test(flavor = "multi_thread")]
async fn send_update_sends_the_committed_update_and_writes_nothing() {
    let (chain, mut a, org, joiner, _b) = founded("send").await;
    let update = a.admit_member(&mut OsRng, org, &joiner).unwrap();
    chain.apply_update(org, update.resulting_root, update.org_pub_key, Epoch::new(1)).unwrap();
    let out = a.commit_update(&mut OsRng, org).await.unwrap();
    let on_disk = store_bytes("send", "a");
    let invite = InviteId::new([0x6e; 32]);
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    a.send_update(&out.outgoing, joiner.device_key, Some(sink_addr), org_secret(), Some(invite)).await.unwrap();
    let (_ep, sender, msg) = sink.await.unwrap();
    assert_eq!(
        msg,
        WireMessage {
            envelope: out.outgoing.envelope.clone(),
            org_secret: org_secret(),
            genesis_snapshot: Some(out.outgoing.record_snapshot.clone()),
            invite_id: Some(invite),
        }
    );
    let first_bound = a.list_personas().iter().find(|p| p.org_id == Some(org)).unwrap().clone();
    assert_eq!(sender, first_bound.device_seed.signing_keypair().device_key().unwrap());
    assert_eq!(store_bytes("send", "a"), on_disk, "a send writes nothing");
    // Abnormal half of LLR-48jakr: no identifier passed, none sent.
    let (sink_addr, sink) = spawn_recv_one(rand::random()).await;
    a.send_update(&out.outgoing, joiner.device_key, Some(sink_addr), None, None).await.unwrap();
    assert_eq!(sink.await.unwrap().2.invite_id, None);
}

// Abnormal: no Persona bound to the Organisation, or Loopback with no
// address: refused, nothing sent, no endpoint bound.
// verifies: LLR-2xzys9, LLR-pw369n
#[tokio::test]
async fn send_update_refuses_without_a_bound_persona_or_a_loopback_address() {
    let (chain, mut a, org, joiner, b) = founded("send-refused").await;
    let update = a.admit_member(&mut OsRng, org, &joiner).unwrap();
    chain.apply_update(org, update.resulting_root, update.org_pub_key, Epoch::new(1)).unwrap();
    let out = a.commit_update(&mut OsRng, org).await.unwrap();
    let mut b = b; // holds no Persona bound to `org`
    assert!(b.send_update(&out.outgoing, joiner.device_key, Some(dead_addr([0x6f; 32])), None, None).await.is_err());
    assert!(b.endpoint().is_none());
    assert!(a.send_update(&out.outgoing, joiner.device_key, None, None, None).await.is_err());
    assert!(a.endpoint().is_none(), "refused before binding");
}

// Normal: a node's own commit keeps the secret it holds (a received update
// overwrites it; that is PR-xwek5e, pinned elsewhere).
// verifies: LLR-ckk5nz
#[tokio::test(flavor = "multi_thread")]
async fn a_nodes_own_commit_keeps_its_secret() {
    let s = admit_b_directly(setup("own-secret").await).await;
    let mut b = s.svc_b;
    assert_eq!(rec_of(&b, s.org_id).org_secret, org_secret());
    let update = b.admit_member(&mut OsRng, s.org_id, &joiner_from(0x61, "carol")).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&b, s.org_id).epoch).unwrap();
    b.commit_update(&mut OsRng, s.org_id).await.unwrap();
    assert_eq!(rec_of(&b, s.org_id).org_secret, org_secret());
}

// Abnormal (REQ-fwfku9's "keep nothing"): an admission the bound refuses
// writes nothing to disk.
// verifies: REQ-fwfku9, LLR-jq7qh7
#[tokio::test]
async fn an_admission_refused_by_the_bound_writes_nothing() {
    let (chain, a, org, joiner, _b) = founded("bound-refused").await;
    drop(a);
    let path = store_dir("bound-refused", "a").join("store.bin");
    let mut store = PersonaStore::open(path, "pw_a").unwrap();
    let big = ProvisionalUpdate {
        org_id: Some(org),
        persona_id: store.data().personas[0].persona_id.clone(),
        base_root: Some(store.data().orgs[0].root_hash),
        resulting_root: RootHash::new([0x42; 32]),
        seq: SequenceNumber::new(2),
        org_pub_key: store.data().orgs[0].org_pub_key,
        change: ProvisionalChange::ChangeSet { change_set: vec![0; org_node::store::MAX_PROVISIONAL_BYTES - 200] },
    };
    store.data_mut().insert_provisional(big).unwrap();
    store.save(&mut OsRng).unwrap();
    let mut a = OrgService::new(store, Box::new(chain.clone()));
    let before = store_bytes("bound-refused", "a");
    assert!(matches!(a.admit_member(&mut OsRng, org, &joiner), Err(OrgNodeError::ProvisionalLimit { .. })));
    assert_eq!(store_bytes("bound-refused", "a"), before);
    assert_eq!(a.provisional_updates(org).len(), 1);
}
```

`admission_sender.rs`:

| Test | Change in T10 |
|---|---|
| `an_admission_reaches_the_administrators_disk` | body unchanged (the helper commits); annotation → `// verifies: LLR-cmdrp9, LLR-cja9zv` (LLR-t4znbk, LLR-rb8r65, LLR-ghja3x are verified in `commit_paths`). |
| `a_failed_push_leaves_the_administrators_record_where_it_was` | rename `a_failed_send_leaves_the_committed_record_in_place`; `// verifies: LLR-t4znbk`; the helper returns the send's `Err`; assert the chain advanced by one **and** A's record advanced with it (epoch +1, member added, mark +1, on disk too) — the record is the commit's, never the send's. Red today: the record stays behind. |
| `the_admission_envelope_carries_the_epoch_its_update_produced` | unchanged (LLR-ghja3x, REQ-txvtm9); it now observes `send_update`'s Envelope. |
| `in_loopback_mode_the_joiner_is_dialled_at_the_full_address` | unchanged (LLR-jn5jeh); the helper's `send_update` dials. |
| `a_second_organisation_is_admitted_into_without_touching_the_first` | add: before committing into org 2, `svc_a.admit_member(.., org_2, ..)` leaves `provisional_updates(org_1)` and org 1's record unchanged; annotation `LLR-vdyu65, LLR-w3fhhg`. |
| `the_proxy_account_from_genesis_is_kept_and_passed_on_every_update` | the admission no longer reaches `ProxyChain::submit_update`: after it, `assert!(chain.seen.lock().unwrap().is_empty(), "admission submits nothing")`; the revocation assertion stays `vec![Some(proxy())]` until T11. Admission goes through `admit(&mut svc_a, &chain.inner, …)`. |
| every other `admit` caller | unchanged (the helper carries the new flow). |

`organisation_key.rs`: its two `ChainOps` substitutes keep `submit_update`
until T11; nothing else changes. `service_stories::five_stories_full_e2e`:
annotation drops `LLR-rb8r65, LLR-t4znbk` (verified in `commit_paths`) and
keeps the rest.

**Step 2 — run, expect red** (compile: `admit_member` signature, no
`commit_update`/`send_update`/`apply_update`/`CommitOutcome`). With stubs,
the `commit_paths` admission tests and
`a_failed_send_leaves_the_committed_record_in_place` FAIL on assertions.
Record.

**Step 3 — implement** (`service.rs`; `lib.rs` re-exports `CommitOutcome`
and `OutgoingUpdate` with the other service types):

```rust
/// The committed update a node sends: the Envelope and the encoded record as
/// it stood before the commit (LLR-cmdrp9, LLR-bg3vsw).
#[derive(Clone, Debug)]
pub struct OutgoingUpdate {
    pub envelope: Envelope,
    pub record_snapshot: Vec<u8>,
}

/// What `commit_update` returns (LLR-cmdrp9).
#[derive(Clone, Debug)]
pub struct CommitOutcome {
    pub org_id: OrgId,
    pub epoch: Epoch,
    pub root: RootHash,
    pub outgoing: OutgoingUpdate,
}
```

`MockChainOps` gains `apply_update` (the old `submit_update` body, inherent,
synchronous, `org_id, root, key, expected_epoch`, LLR-ryzr8m), and its
`ChainOps::submit_update` delegates to it (until T11 removes it).

```rust
    /// Build the admission of `joiner` into `org_id` and keep it: no chain
    /// operation, no endpoint, no send, the record unchanged (LLR-rb8r65,
    /// LLR-vdyu65).
    pub fn admit_member<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
        joiner: &Joiner,
    ) -> Result<ProvisionalUpdate, OrgNodeError> {
        let rec = self.find_org(org_id)?.clone();
        let persona_id = self.first_persona_bound_to(org_id)?.persona_id.clone();
        let leaf = MemberLeaf::new(
            fresh_member_id(rng),
            joiner.handle.clone(),
            joiner.member_key,
            joiner.name.clone(),
            joiner.surname.clone(),
            vec![joiner.device_key],
        )
        .map_err(OrgNodeError::Trie)?;
        let (new_trie, delta) = trie_from_snapshots(&rec.trie_members)?
            .add_member(leaf)
            .map_err(OrgNodeError::Trie)?
            .recalculate()
            .map_err(OrgNodeError::Trie)?;
        self.keep_change_set(rng, &rec, persona_id, &new_trie, &delta)
    }

    /// Keep a Change set built on `rec` as a provisional update: its Sequence
    /// number is the epoch it produces, the record's plus one (LLR-ghja3x,
    /// REQ-txvtm9, Decision 16).
    fn keep_change_set<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        rec: &OrgRecord,
        persona_id: PersonaId,
        new_trie: &Trie,
        delta: &Delta,
    ) -> Result<ProvisionalUpdate, OrgNodeError> {
        let update = ProvisionalUpdate {
            org_id: Some(rec.org_id),
            persona_id,
            base_root: Some(rec.root_hash),
            resulting_root: new_trie.root_hash().map_err(OrgNodeError::Trie)?,
            seq: SequenceNumber::new(rec.epoch.get() + 1),
            org_pub_key: rec.org_pub_key,
            change: ProvisionalChange::ChangeSet {
                change_set: postcard::to_allocvec(delta).map_err(|_| OrgNodeError::MalformedDelta)?,
            },
        };
        self.store.data_mut().insert_provisional(update.clone())?;
        self.store.save(rng)?;
        Ok(update)
    }

    /// Commit the node's own provisional update for `org_id` once the chain
    /// carries its root, by the checks a received update passes (LLR-cmdrp9).
    /// A refusal changes and writes nothing (LLR-ewkg85); nothing is bound or
    /// sent (LLR-4tcxsu).
    pub async fn commit_update<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
    ) -> Result<CommitOutcome, OrgNodeError> {
        let rec = self.find_org(org_id)?.clone();
        let state = self.chain.read_state(org_id).await?.ok_or(OrgNodeError::OrgNotOnChain)?;
        let update = self
            .store
            .data()
            .provisional_updates
            .iter()
            .find(|u| u.org_id == Some(org_id) && u.resulting_root == state.root_hash)
            .cloned()
            .ok_or(OrgNodeError::NoProvisionalUpdate)?;
        let ProvisionalChange::ChangeSet { change_set } = update.change else {
            return Err(OrgNodeError::NoProvisionalUpdate);
        };
        let envelope = Envelope { org_id, parent_seq: update.seq, delta_bytes: change_set };
        let ctx = VerifyContext {
            expected_org_id: org_id,
            seq_guard: SeqGuard::from_last_seen(rec.last_seq),
            last_committed_epoch: rec.epoch,
        };
        let local = trie_from_snapshots(&rec.trie_members)?;
        let verified = verify_envelope_against_chain(&local, &envelope, &ctx, &ChainOpsReader { state })?;
        let record_snapshot = encode_record_snapshot(&rec.trie_members)?;
        let root = self.commit_held(org_id, &verified, None)?;
        self.store.save(rng)?;
        Ok(CommitOutcome { org_id, epoch: verified.epoch, root, outgoing: OutgoingUpdate { envelope, record_snapshot } })
    }

    /// Write a verified update to a held record — root, epoch, mark and
    /// members together (LLR-cja9zv) — and drop the provisional updates it
    /// orphans (LLR-mkj4bz). `secret`: `Some(s)` replaces the stored secret
    /// (a received update, LLR-ckk5nz); `None` keeps it (the node's own).
    fn commit_held(
        &mut self,
        org_id: OrgId,
        verified: &VerifiedUpdate,
        secret: Option<Option<OrgSecret>>,
    ) -> Result<RootHash, OrgNodeError> {
        let root = verified.trie.root_hash().map_err(OrgNodeError::Trie)?;
        let snapshots: Vec<MemberSnapshot> = verified.trie.members().iter().map(snapshot_of).collect();
        let data = self.store.data_mut();
        let rec = data.orgs.iter_mut().find(|o| o.org_id == org_id).ok_or(OrgNodeError::OrgNotOnChain)?;
        rec.root_hash = root;
        rec.epoch = verified.epoch;
        rec.last_seq = verified.seq_guard.last_seen();
        rec.trie_members = snapshots;
        if let Some(secret) = secret {
            rec.org_secret = secret;
        }
        Self::discard_orphans(data, org_id, root);
        Ok(root)
    }

    /// Send a committed update to `recipient`'s device, from the device of
    /// the first Persona bound to its Organisation (LLR-2xzys9), carrying
    /// exactly the secret and invite identifier given (LLR-8hdu9x,
    /// LLR-48jakr). Loopback dials `peer_addr` and refuses without one
    /// (LLR-jn5jeh, LLR-pw369n); Networked dials by `recipient`. Writes
    /// nothing (LLR-t4znbk).
    pub async fn send_update(
        &mut self,
        outgoing: &OutgoingUpdate,
        recipient: DevicePublicKey,
        peer_addr: Option<iroh::EndpointAddr>,
        org_secret: Option<OrgSecret>,
        invite_id: Option<InviteId>,
    ) -> Result<(), OrgNodeError> {
        let mode = self.transport_mode;
        let persona_id = self.first_persona_bound_to(outgoing.envelope.org_id)?.persona_id.clone();
        if mode == TransportMode::Loopback && peer_addr.is_none() {
            return Err(OrgNodeError::Chain("Loopback send requires the peer's EndpointAddr".into()));
        }
        let msg = WireMessage {
            envelope: outgoing.envelope.clone(),
            org_secret,
            genesis_snapshot: Some(outgoing.record_snapshot.clone()),
            invite_id,
        };
        let ep = self.ensure_endpoint(&persona_id).await?;
        match (mode, peer_addr) {
            (TransportMode::Loopback, Some(addr)) => {
                ep.send(addr, &msg).await.map_err(|e| OrgNodeError::Chain(format!("iroh send: {e}")))
            }
            (TransportMode::Loopback, None) => {
                Err(OrgNodeError::Chain("Loopback send requires the peer's EndpointAddr".into()))
            }
            (TransportMode::Networked, _) => {
                let peer = iroh::EndpointId::from_bytes(recipient.as_bytes())
                    .map_err(|_| OrgNodeError::Chain("invalid recipient device key for EndpointId".into()))?;
                ep.send_to_id(peer, &msg)
                    .await
                    .map_err(|e| OrgNodeError::Chain(format!("iroh send (networked): {e}")))
            }
        }
    }
```

In `receive_and_verify`, the held-record branch of the commit becomes
`self.commit_held(org_id, &verified, Some(msg.org_secret.clone()))?;` and the
first-admission branch, after pushing the new record and clearing the
matched expectation, calls
`Self::discard_orphans(self.store.data_mut(), org_id, new_root)`. In
`receive_and_self_delete_if_revoked` the `UpdatedNotRevoked` branch becomes
`self.commit_held(org_id, &verified, None)?;` (that path never touched the
secret; unchanged). The old `admit_member` body (chain write, send, record
update, T6's `invite_id` argument) goes. Imports: `org_members::delta::Delta`,
`crate::verify::VerifiedUpdate`, `crate::types::InviteId`.

**Step 4 — green.** commit_paths 16 passed; admission_sender unchanged in
count; service_stories 3; the rest unchanged. Commit:
`T10: admission is provisional; commit_update and send_update`.

**Red→green attestations:** (DONE 2026-10-06, merged; org-node cargo line
223 passed, 0 failed; commit_paths 17. App `cargo check` errors unchanged;
Cargo.lock restored.) Every test below was compile-red first (E0061
`admit_member` arity; E0599 `commit_update`/`send_update`/`apply_update`).
Stages: A = all three operations stubbed; B = `admit_member` real, the other
two stubbed; C = `commit_update` real, `send_update` a no-op.

- admit_member_keeps_a_provisional_update_and_changes_nothing_else — red at A; killed by mutant `seq = epoch + 2`; green
- commit_update_commits_a_provisional_update_the_chain_carries — runtime-red at B; killed by mutant "commit_update does not save"; green
- commit_update_refusals_change_nothing_and_write_nothing — runtime-red at B (`Chain("stub")` ≠ `OrgNotOnChain`); green
- commit_update_refuses_when_the_chain_fails_or_is_silent — red at B and C only through setup; killed by mutant "absent chain state → NoProvisionalUpdate"; green
- a_commit_discards_the_provisional_updates_it_orphans — red at B and C only through setup; killed by mutant "commit_held skips discard_orphans"; green
- send_update_sends_the_committed_update_and_writes_nothing — runtime-red at C (receiver timed out against the no-op send); green
- send_update_refuses_without_a_bound_persona_or_a_loopback_address — runtime-red at C; killed by mutant "bind before the Loopback address check"; green
- a_nodes_own_commit_keeps_its_secret — red at B and C only through setup; killed by mutant "commit_update passes Some(None)"; green
- an_admission_refused_by_the_bound_writes_nothing — red at A; killed by mutant "save before insert_provisional"; green
- a_failed_send_leaves_the_committed_record_in_place (rewritten, renamed) — runtime-red at A; killed by the no-save mutant; green
- a_second_organisation_is_admitted_into_without_touching_the_first (rewritten) — runtime-red at A; killed by the no-save mutant; green
- the_proxy_account_from_genesis_is_kept_and_passed_on_every_update (rewritten) — runtime-red at A; green
- a_receive_against_a_state_with_an_invalid_key_commits_nothing (rewritten) — killed by mutant "send_update returns Ok at once"; green
- an_admission_reaches_the_administrators_disk, five_stories_full_e2e (annotation only) — killed by the no-save and no-op-send mutants respectively
- the_persona_marked_active_is_the_first_whose_device_is_in_the_record — went red under the real implementation (members now in trie order); fixture fixed to look the founder up by handle; green

Behaviour note: a committed record's `trie_members` now come from the verified
trie (trie order), as the receive path already did, not admission order.

---

### T10b — org-node: discarding a provisional update (REQ-hhva9d)

Added 2026-10-06 by the dispatcher on the owner's ruling (discard operation;
late genesis refused and joined as a Member — the latter is LLR-wzqqg9 as
written, verified in T9).

**Files touched:** `org-node/src/service.rs`, `org-node/src/lib.rs`,
`org-node/tests/commit_paths.rs`
**Parallel:** no (serial, after T10)
**IDs verified:** REQ-hhva9d, LLR-7cmp38.
**Size:** ~150 lines.

Use the fixtures `commit_paths.rs` already has after T9/T10: the helper that
builds a service holding one Persona and the helper that runs the admission
story up to a stored provisional update. Where this task writes
`genesis_service(tag)` and `held_org_with_provisional(tag)` below, use those
helpers by their actual names; do not add new fixtures that duplicate them.

**Step 1 — failing tests.** Append to `org-node/tests/commit_paths.rs`:

```rust
// verifies: LLR-7cmp38, REQ-hhva9d
#[tokio::test]
async fn discarding_a_genesis_update_removes_it_and_its_private_key_and_saves() {
    let (mut svc, persona_id, dir) = genesis_service("discard-genesis");
    let update = svc.create_organisation(&mut OsRng, &persona_id).await.unwrap();
    assert_eq!(svc.genesis_provisional_updates(&persona_id).len(), 1);

    svc.discard_provisional(
        ProvisionalTarget::Genesis(persona_id.clone()),
        update.resulting_root,
    )
    .unwrap();

    assert!(svc.genesis_provisional_updates(&persona_id).is_empty());
    // Saved before return: a service reopened on the same directory holds
    // no genesis update, so the private key it held is gone from disk too.
    let reopened = reopen_service(&dir);
    assert!(reopened.genesis_provisional_updates(&persona_id).is_empty());
}

// verifies: LLR-7cmp38, REQ-hhva9d
#[tokio::test]
async fn discarding_one_of_two_updates_keeps_the_other_and_the_record() {
    let (mut svc, org_id, first, second, dir) = held_org_with_provisional("discard-one");
    let record_before = svc.list_orgs().into_iter().find(|o| o.org_id == org_id).unwrap();

    svc.discard_provisional(ProvisionalTarget::Org(org_id), first.resulting_root).unwrap();

    let left: Vec<_> = svc.provisional_updates(&org_id).iter().map(|u| u.resulting_root).collect();
    assert_eq!(left, vec![second.resulting_root]);
    let record_after = svc.list_orgs().into_iter().find(|o| o.org_id == org_id).unwrap();
    assert_eq!(record_after.root_hash, record_before.root_hash);
    assert_eq!(record_after.epoch, record_before.epoch);
    let _ = dir;
}

// verifies: LLR-7cmp38, REQ-hhva9d
#[tokio::test]
async fn discarding_an_unknown_root_is_refused_and_writes_nothing() {
    let (mut svc, org_id, first, _second, dir) = held_org_with_provisional("discard-unknown");
    let bytes_before = std::fs::read(store_file(&dir)).unwrap();

    let err = svc
        .discard_provisional(ProvisionalTarget::Org(org_id), RootHash::new([0xAB; 32]))
        .unwrap_err();

    assert!(matches!(err, OrgNodeError::NoProvisionalUpdate));
    assert_eq!(svc.provisional_updates(&org_id).len(), 2);
    assert_eq!(std::fs::read(store_file(&dir)).unwrap(), bytes_before, "nothing written");
    // The genesis target with an Org's root is also unknown.
    let err = svc
        .discard_provisional(ProvisionalTarget::Genesis(PersonaId::new([7; 16])), first.resulting_root)
        .unwrap_err();
    assert!(matches!(err, OrgNodeError::NoProvisionalUpdate));
}
```

(`reopen_service`, `store_file`, `PersonaId::new`'s width: use what
`commit_paths.rs` and `tests/support` already provide for reopening a store and
locating its file, and the real `PersonaId` constructor; adjust only those
names.)

Run `cargo test -p org-node --features app,test-support --test commit_paths
discard`: compile-red (`discard_provisional`, `ProvisionalTarget` unresolved).
Then land the signature with a body returning `Ok(())` and watch the first two
tests fail at runtime (the update is still listed) and the third fail
(`Ok(())` where `NoProvisionalUpdate` was expected).

**Step 2 — implementation.** In `org-node/src/service.rs`:

```rust
/// Which provisional updates a discard names: an Organisation's, or the
/// genesis updates one Persona built (they have no Organisation yet).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProvisionalTarget {
    Org(OrgId),
    Genesis(PersonaId),
}

impl OrgService {
    /// LLR-7cmp38: remove the one provisional update for `target` whose
    /// resulting root is `resulting_root`, with any private key it holds, and
    /// save; refuse an unknown one and write nothing.
    pub fn discard_provisional(
        &mut self,
        target: ProvisionalTarget,
        resulting_root: RootHash,
    ) -> Result<(), OrgNodeError> {
        let updates = &mut self.store.data_mut().provisional_updates;
        let position = updates
            .iter()
            .position(|u| u.resulting_root == resulting_root && u.is_for(&target))
            .ok_or(OrgNodeError::NoProvisionalUpdate)?;
        updates.remove(position);
        self.store.save()
    }
}
```

and on the stored provisional update type (where T8 defined it):

```rust
impl ProvisionalUpdate {
    pub(crate) fn is_for(&self, target: &ProvisionalTarget) -> bool {
        match target {
            ProvisionalTarget::Org(org_id) => self.org_id.as_ref() == Some(org_id),
            ProvisionalTarget::Genesis(persona_id) => {
                self.org_id.is_none() && self.persona_id == *persona_id
            }
        }
    }
}
```

(`persona_id: PersonaId` is the field T8 defined on every provisional update:
the Persona that built it, which for a genesis update is how it is grouped.)
Export `ProvisionalTarget` from `lib.rs` beside
the other service types. `self.store.save()`'s error type is whatever the
store already returns through `OrgNodeError`.

**Step 3 — green.** The three tests pass; org-node's full cargo line passes.
Commit: `T10b: discard a provisional update`.

**Red→green attestations:** (DONE 2026-10-06, merged; org-node cargo line
passed, 0 failed; commit_paths 17 → 21. The subagent summed 217 passes over
the "test result" lines where T10 reported 223; neither run failed anything.
The verification gate's per-target counts are the figure of record.) Each test
was compile-red (E0432 `ProvisionalTarget`, E0599 `discard_provisional`),
then runtime-red against an `Ok(())` stub, then green:

- discarding_a_genesis_update_removes_it_and_its_private_key_and_saves — the genesis update was still listed
- discarding_one_of_two_updates_keeps_the_other_and_the_record — both updates were still listed
- discarding_an_unknown_root_is_refused_and_writes_nothing — `unwrap_err` on `Ok(())`
- a_genesis_update_is_discarded_only_under_its_own_persona (added beyond the sketch) — `unwrap_err` on `Ok(())` under another Persona's target

Departures: the operation takes `rng` (sealing the store on save), and
LLR-7cmp38 was amended to say so; the match is a private
`ProvisionalTarget::names(&ProvisionalUpdate)` in service.rs, so store.rs does
not depend on a service type.

---

### T11 — org-node: revocation is provisional; the chain seam is read-only (PR-b9wab3)

**Files touched:** `org-node/src/service.rs`,
`org-node/tests/support/mod.rs`, `org-node/tests/commit_paths.rs`,
`org-node/tests/admission_sender.rs`, `org-node/tests/service_lifecycle.rs`,
`org-node/tests/organisation_key.rs`, `org-node/tests/absences.rs`,
`org-node/docs/problems/2026-10-04-revoke-precondition.md`
**Parallel:** no (serial, after T10)
**IDs verified:** REQ-xs4ab8, REQ-tqap3r, REQ-txvtm9, LLR-6dc598,
LLR-tax3pm, LLR-qg9utu, LLR-drgdy8, LLR-pw369n, LLR-3v5nu9 (no operation
passes the account to the chain), LLR-65py3d, LLR-ryzr8m, LLR-8hdu9x,
LLR-t4znbk, PR-b9wab3 (resolved).
**Size:** ~340 lines.

**Step 1 — reproduce PR-b9wab3 against today's code (red for the right
reason).** In `admission_sender.rs`,
`a_loopback_revocation_with_no_peer_address_is_refused_and_records_nothing`
pins the defect (`epoch_before + 1`). Invert the pin only: replace that
assertion with

```rust
    assert_eq!(
        s.chain.get(&s.org_id).unwrap().epoch,
        epoch_before,
        "PR-b9wab3: a revocation refused for want of an address must not have moved the chain"
    );
```

and run `--test admission_sender`: FAILED, left `epoch 3`, right `epoch 2` —
the address check sits after the chain write. Record this red. (Its
`revoke` helper call still uses today's `revoke_member`.)

**Step 2 — the remaining failing tests.** `support/mod.rs`:

```rust
/// Story 5: revoke `member_id`: build, write the chain (mock), commit, send
/// the committed revocation to the removed member's first device.
pub async fn revoke(
    svc: &mut OrgService,
    chain: &MockChainOps,
    org_id: OrgId,
    member_id: MemberId,
    addr: Option<iroh::EndpointAddr>,
) -> Result<(), OrgNodeError> {
    let recipient = rec_of(svc, org_id)
        .trie_members
        .iter()
        .find(|m| m.id == member_id)
        .and_then(|m| m.device_keys.first().copied())
        .expect("the removed member has a device");
    let update = svc.revoke_member(&mut OsRng, org_id, member_id)?;
    chain.apply_update(org_id, update.resulting_root, update.org_pub_key, rec_of(svc, org_id).epoch)?;
    let outcome = svc.commit_update(&mut OsRng, org_id).await?;
    tokio::time::timeout(NET, svc.send_update(&outcome.outgoing, recipient, addr, None, None))
        .await
        .expect("send timed out")
}
```

`CountingChain`: delete its `submit_update` method.

`commit_paths.rs`:

```rust
// Normal: revocation keeps a provisional update — base, the epoch it
// produces, the record's key, no signature anywhere — and touches neither the
// chain nor the record.
// verifies: REQ-xs4ab8, REQ-txvtm9, LLR-6dc598, LLR-tax3pm
#[tokio::test(flavor = "multi_thread")]
async fn revoke_member_keeps_a_provisional_update_and_changes_nothing_else() {
    let s = admit_b_directly(setup("revoke-provisional").await).await;
    let mut a = s.svc_a;
    let rec = rec_of(&a, s.org_id);
    let b_id = id_by_handle(&rec, "bob");
    let update = a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    assert_eq!((update.org_id, update.base_root), (Some(s.org_id), Some(rec.root_hash)));
    assert_eq!((update.seq, update.org_pub_key), (SequenceNumber::new(rec.epoch.get() + 1), rec.org_pub_key));
    assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, rec.epoch, "no chain write");
    assert!(rec_of(&a, s.org_id).trie_members.iter().any(|m| m.id == b_id), "record unchanged");
    assert_eq!(a.provisional_updates(s.org_id), vec![update]);
}

// Abnormal: a member the record does not hold cannot be revoked, and
// nothing is kept.
// verifies: LLR-6dc598
#[tokio::test(flavor = "multi_thread")]
async fn revoking_a_member_the_record_does_not_hold_keeps_nothing() {
    let s = setup("revoke-unknown").await;
    let mut a = s.svc_a;
    let err = a.revoke_member(&mut OsRng, s.org_id, org_node::MemberId::new([0x77; 32])).unwrap_err();
    assert!(matches!(err, OrgNodeError::Trie(_)), "{err:?}");
    assert!(a.provisional_updates(s.org_id).is_empty());
}

// Normal and abnormal: the revoking record takes the removal only through
// commit_update — not from revoke_member, not from a failed send — and the
// proxy account is neither read nor changed by any of it.
// verifies: REQ-tqap3r, LLR-qg9utu, LLR-drgdy8, LLR-3v5nu9, LLR-t4znbk
#[tokio::test(flavor = "multi_thread")]
async fn the_revoking_record_takes_the_removal_only_through_commit_update() {
    let s = admit_b_directly(setup("revoke-commit").await).await;
    let mut a = s.svc_a;
    let b_id = id_by_handle(&rec_of(&a, s.org_id), "bob");
    let proxy_before = a.proxy_account(s.org_id).unwrap();
    let update = a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    assert!(rec_of(&a, s.org_id).trie_members.iter().any(|m| m.id == b_id));
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&a, s.org_id).epoch).unwrap();
    let out = a.commit_update(&mut OsRng, s.org_id).await.unwrap();
    assert!(!rec_of(&a, s.org_id).trie_members.iter().any(|m| m.id == b_id), "committed");
    let device = s.joiner_b.device_key;
    assert!(a.send_update(&out.outgoing, device, Some(dead_addr([0x7f; 32])), None, None).await.is_err());
    assert!(!rec_of(&a, s.org_id).trie_members.iter().any(|m| m.id == b_id), "a failed send undoes nothing");
    assert_eq!(a.proxy_account(s.org_id).unwrap(), proxy_before);
}
```

`service_lifecycle.rs`:
- `the_mock_chain_refuses_an_update_at_the_wrong_epoch` (LLR-ryzr8m): replace
  `chain.submit_update(org, root, key, epoch, None).await` with
  `chain.apply_update(org, root, key, epoch)` (synchronous) in both places;
  drop `use org_node::ChainOps;` and make the test `#[test] fn`.
- `the_seam_is_a_trait_object_a_substitute_can_stand_in_for` (LLR-65py3d):

```rust
// The seam presents reading and nothing else: a substitute that implements
// only `read_state` is a complete `ChainOps` and drives the service.
// verifies: LLR-65py3d
#[tokio::test]
async fn the_seam_is_a_trait_object_a_substitute_can_stand_in_for() {
    use org_node::ChainOps as _;
    struct ReadOnly(MockChainOps);
    #[async_trait::async_trait]
    impl org_node::ChainOps for ReadOnly {
        async fn read_state(&self, org: OrgId) -> Result<Option<OrgState>, org_node::OrgNodeError> {
            self.0.read_state(org).await
        }
    }
    let (store, _) = store_at("seam");
    let chain = MockChainOps::new();
    let mut svc = OrgService::new(store, Box::new(ReadOnly(chain.clone())));
    let pid = svc.create_persona(&mut OsRng, h("s"), nm("S"), sn("T")).unwrap();
    let update = svc.create_organisation(&mut OsRng, &pid).unwrap();
    let org = chain.apply_genesis(update.resulting_root, update.org_pub_key);
    svc.commit_genesis(&mut OsRng, &pid, org, org_node::ChainAccount::new([1; 32])).await.unwrap();
}
```

  (Red today: E0046, `submit_update` missing from the impl.)

`organisation_key.rs`: its two `ChainOps` substitutes delete `submit_update`.

`absences.rs`:

```rust
// The service's chain seam writes nothing; the proxy account is never handed
// to it (absences, no input side).
// verifies: LLR-65py3d, LLR-3v5nu9, REQ-xs4ab8
#[test]
fn the_service_presents_no_chain_write() {
    assert_absent_in("service.rs", "async fn submit_", "ChainOps presents no write, so nothing can take the proxy account to the chain");
}
```

`admission_sender.rs`:

| Test | Change in T11 |
|---|---|
| `a_loopback_revocation_with_no_peer_address_is_refused_and_records_nothing` (Step 1's inverted pin) | rename `a_loopback_revocation_without_an_address_burns_no_epoch`; `// verifies: LLR-pw369n, LLR-6dc598, PR-b9wab3`; body: `revoke_member` → chain epoch unchanged (no burn possible: it writes nothing); the app-style `apply_update` + `commit_update` move the chain and the record by exactly one; `send_update(&out.outgoing, device, None, None, None)` in Loopback → `Err`, the chain still at `epoch_before + 1` (the caller's one write, nothing more), the record committed, nothing sent (the sink receives nothing within 2 s). |
| `a_revocation_reaches_the_administrators_disk` | body via `revoke`; annotation → `// verifies: LLR-qg9utu, LLR-8hdu9x` (LLR-6dc598, LLR-tax3pm, LLR-pw369n are verified in `commit_paths` and the test above). |
| `a_failed_revocation_push_leaves_the_administrators_record_where_it_was` | rename `a_failed_revocation_send_leaves_the_committed_removal_in_place`; `// verifies: LLR-qg9utu, LLR-t4znbk`; assert `Err`, the removal **committed** in A's record and on disk. |
| `the_proxy_account_from_genesis_is_kept_and_passed_on_every_update` | rename `the_proxy_account_from_genesis_is_kept_and_handed_back`; delete `ProxyChain` (use a plain `MockChainOps`); keep the LLR-dzte8x assertions; after admitting and revoking C via the helpers assert `svc_a.proxy_account(org_id).unwrap() == Some(proxy())` (`// verifies: LLR-dzte8x, LLR-3v5nu9, LLR-drgdy8`). |
| every other `revoke` caller | unchanged (the helper carries the flow). |

**Step 3 — run, expect red** (compile: `revoke_member` arity and return;
`apply_update` on the mock is there since T10; E0046 in the seam test).
Record.

**Step 4 — implement.**
- `ChainOps` keeps only `read_state`; its doc: "Read-only access to the
  chain: org-node writes nothing to it (LLR-65py3d)." `MockChainOps` loses
  its `ChainOps::submit_update`. `SubxtChainOps` loses `submit_update`,
  `api`, `contract_h160`, `admin`, `others`, `settle_timeout` and `sink()`;
  it keeps `registry_client` and `read_state`; `new(registry_client:
  OrgRegistryClient) -> Self`. `FinalitySink` stays (deleted in T13).
- `revoke_member` (now `pub fn`):

```rust
    /// Build the removal of `member_id` from `org_id` and keep it: no chain
    /// operation, no endpoint, no send, the record unchanged (LLR-6dc598).
    pub fn revoke_member<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        org_id: OrgId,
        member_id: MemberId,
    ) -> Result<ProvisionalUpdate, OrgNodeError> {
        let rec = self.find_org(org_id)?.clone();
        let persona_id = self.first_persona_bound_to(org_id)?.persona_id.clone();
        let (new_trie, delta) = trie_from_snapshots(&rec.trie_members)?
            .delete_member(&member_id)
            .map_err(OrgNodeError::Trie)?
            .recalculate()
            .map_err(OrgNodeError::Trie)?;
        self.keep_change_set(rng, &rec, persona_id, &new_trie, &delta)
    }
```

  The old body (chain write, address check, send, record update) goes.
- `set_transport_mode`'s doc: "…before the first `ensure_endpoint`,
  `receive_and_verify` or `send_update` call".

**Step 5 — green; resolve PR-b9wab3.** The org-node line: commit_paths 19,
service_lifecycle unchanged in count, absences 3, admission_sender unchanged
in count. In `org-node/docs/problems/2026-10-04-revoke-precondition.md`
replace `status: open` with:

```
status: resolved
resolution: root cause — `revoke_member` wrote the chain before checking the
Loopback address it would need to send. Change
worktree-org-node-chain-authority (docs/plans/2026-10-05-chain-authority.md,
T11) removed the chain write from org-node: `revoke_member` builds a
provisional update and touches neither the chain nor the record, and the
address check moved to `send_update`, which runs after a commit the chain
already agrees with, so a missing address can no longer burn an epoch.
Reproduced by inverting the pin in
`a_loopback_revocation_without_an_address_burns_no_epoch`
(org-node/tests/admission_sender.rs): red before (the chain at epoch 3),
green after.
```

Commit: `T11: revocation is provisional; the chain seam is read-only`.

**Red→green attestations:** (DONE 2026-10-06, merged; org-node cargo line
231 passed, 0 failed, summed with awk over the 25 "test result" lines — the 3
fuzz targets print none and ran clean. 223 (T10) + 4 (T10b) + 4 (T11) = 231,
so T10b's "217" was a miscount. Per target: lib 1, absences 3,
admission_sender 45, calldata_typed 2, chain_read_state 3, chain_write_pure 8,
commit_paths 24, encoding_golden 4, envelope_binding 4, expected_admission 9,
key_custody 9, node_value_types 12, organisation_key 9, persona_records 15,
provisional_store 5, receive_chain_reads 7, secret_redaction 8,
service_lifecycle 9, service_stories 3, store_at_rest 7, transport_handshake 6,
transport_networked 1, value_types 8, verify_against_chain 23,
wire_frame_bound 6. PR-b9wab3 marked resolved in its dated file.)

- a_loopback_revocation_without_an_address_burns_no_epoch (PR-b9wab3 reproduction; rewritten, renamed) — runtime-red against the merged code ("left: Epoch(3), right: Epoch(2)": the chain was written before the address check); then compile-red (E0061); runtime-red against an `Err` stub and a "record-at-once" mutant; green
- revoke_member_keeps_a_provisional_update_and_changes_nothing_else — compile-red; runtime-red against the stub; killed by the record-at-once mutant; green
- revoking_a_member_the_record_does_not_hold_keeps_nothing — compile-red; runtime-red against the stub (`Chain("stub")` ≠ `Trie`); green
- the_revoking_record_takes_the_removal_only_through_commit_update — compile-red; runtime-red against the stub; killed by the record-at-once mutant (its proxy assertion is weak: a commit-drops-proxy mutant strips the proxy during setup; the next test kills that mutant); green
- the_service_presents_no_chain_write — runtime-red with the pre-T11 service.rs (`async fn submit_` present); green
- a_revocation_reaches_the_administrators_disk (rewritten via the 3-step helper) — runtime-red against the stub; killed by commit-no-save; green
- a_failed_revocation_send_leaves_the_committed_removal_in_place (rewritten, renamed) — runtime-red against the stub and the record-at-once mutant; killed by commit-no-save; green
- the_proxy_account_from_genesis_is_kept_and_handed_back (rewritten, renamed) — runtime-red against the stub and the record-at-once mutant; killed by commit-drops-proxy; green
- the_mock_chain_refuses_an_update_at_the_wrong_epoch (rewritten) — compile-red; killed by an epoch-lt mutant in `apply_update`; green
- the_seam_is_a_trait_object_a_substitute_can_stand_in_for (rewritten) — compile-red only: the property is the trait's shape

Mutants were applied by scratchpad/t11-mutate.py and service.rs restored after
each.

---

### T12 — org-node: own removal on every commit path (PR-322qst)

**Files touched:** `org-node/src/service.rs`,
`org-node/tests/admission_sender.rs`, `org-node/tests/commit_paths.rs`,
`org-node/docs/problems/2026-10-04-own-revocation-on-receive.md`
**Parallel:** no (serial, after T11)
**IDs verified:** REQ-uxv2x2, LLR-b27jr6, LLR-6p4pj2, LLR-jsx922, PR-322qst
(resolved).
**Size:** ~180 lines.

**Step 1 — invert the pin (red for the right reason).** In
`admission_sender.rs`,
`pr_322qst_an_own_revocation_on_the_ordinary_path_is_committed_not_self_deleted`
(LLR-cja9zv) → rename `an_own_revocation_on_the_ordinary_path_deletes_the_record`;
`// verifies: REQ-uxv2x2, LLR-b27jr6, PR-322qst`; keep its setup (B admitted,
A revokes B through `revoke`, delivered to B's `receive_and_verify`); assert:
`result` is `Ok`, `svc_b.list_orgs()` holds no record of `s.org_id`, B's
Persona is `Revoked`, the store on disk agrees, and
`svc_b.provisional_updates(s.org_id)` is empty (B first builds one with
`admit_member` so the clause has something to remove). Replace its "PINS A
DEFECT" comment. Run → FAILED (B commits an update at epoch 3, Persona
`Active`). Record. LLR-cja9zv keeps its evidence in
`update_from_the_admin_after_admission_is_committed`,
`a_committed_admission_reaches_the_disk_and_clears_the_expectation` and
`commit_update_commits_a_provisional_update_the_chain_carries`.

Append to `commit_paths.rs`:

```rust
// Normal (LLR-b27jr6 on commit_update): a node that commits its own removal
// forgets the Organisation — record, provisional updates — and revokes its
// Personas bound to it.
// verifies: REQ-uxv2x2, LLR-b27jr6, LLR-6p4pj2
#[tokio::test(flavor = "multi_thread")]
async fn a_node_that_commits_its_own_removal_forgets_the_organisation() {
    let s = admit_b_directly(setup("own-removal").await).await;
    let mut b = s.svc_b;
    let b_id = id_by_handle(&rec_of(&b, s.org_id), "bob");
    b.admit_member(&mut OsRng, s.org_id, &joiner_from(0x61, "carol")).unwrap();
    let update = b.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&b, s.org_id).epoch).unwrap();
    b.commit_update(&mut OsRng, s.org_id).await.unwrap();
    assert!(b.list_orgs().iter().all(|o| o.org_id != s.org_id));
    assert!(b.provisional_updates(s.org_id).is_empty());
    assert_eq!(persona_of(&b, &s.pid_b).status, PersonaStatus::Revoked);
}

// Abnormal: a commit that keeps one of the node's Personas is an update,
// nothing forgotten.
// verifies: LLR-b27jr6, LLR-jsx922
#[tokio::test(flavor = "multi_thread")]
async fn a_commit_that_keeps_one_of_our_personas_is_an_update() {
    let mut s = admit_b_directly(setup("keeps-us").await).await;
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let (sink, _task) = spawn_recv_one(rand::random()).await;
    admit(&mut s.svc_a, &s.chain, s.org_id, &joiner_c, sink, None).await.unwrap();
    let c_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "carol");
    let (sink, _task) = spawn_recv_one(rand::random()).await;
    revoke(&mut s.svc_a, &s.chain, s.org_id, c_id, Some(sink)).await.unwrap();
    assert_eq!(rec_of(&s.svc_a, s.org_id).trie_members.len(), 2);
    assert_eq!(persona_of(&s.svc_a, &s.pid_a).status, PersonaStatus::Active);
}
```

**Step 2 — implement.** In `service.rs` add:

```rust
    /// Whether any Persona bound to `org_id` has its device in `trie`.
    fn still_member(&self, org_id: OrgId, trie: &Trie) -> bool {
        self.store.data().personas.iter().filter(|p| p.org_id == Some(org_id)).any(|p| {
            p.device_seed
                .signing_keypair()
                .device_key()
                .is_ok_and(|device| trie.members().iter().any(|m| m.has_p2p_device(&device)))
        })
    }

    /// The node has been removed from `org_id`: delete its record and every
    /// provisional update for it, and mark its Personas bound to it Revoked
    /// (LLR-6p4pj2, LLR-b27jr6).
    fn forget_organisation(&mut self, org_id: OrgId) {
        let data = self.store.data_mut();
        data.orgs.retain(|o| o.org_id != org_id);
        data.provisional_updates.retain(|u| u.org_id != Some(org_id));
        for p in data.personas.iter_mut().filter(|p| p.org_id == Some(org_id)) {
            p.status = PersonaStatus::Revoked;
        }
    }
```

In `receive_and_verify`'s held-record branch, and in `commit_update`, before
`commit_held`: `if !self.still_member(org_id, &verified.trie) {
self.forget_organisation(org_id); } else { self.commit_held(…)?; }` (then
save and return the outcome as before; `commit_update`'s `root` is
`verified.trie.root_hash()?`). The first-admission branch is unchanged (its
rule is REQ-kt877x's, T6 — Decision 8). In `receive_and_self_delete_if_revoked`
replace the inline `my_still_present` computation and the self-delete block
by `still_member` and `forget_organisation` (so that path now also drops the
Organisation's provisional updates).

**Step 3 — green; resolve PR-322qst.** commit_paths 21, admission_sender
unchanged in count. In
`org-node/docs/problems/2026-10-04-own-revocation-on-receive.md` replace
`status: open` with:

```
status: resolved
resolution: root cause — the own-removal rule (REQ-uxv2x2) was implemented
on the self-delete path only. Change worktree-org-node-chain-authority
(docs/plans/2026-10-05-chain-authority.md, T12) applies it on every commit
of an Organisation already held (LLR-b27jr6): `receive_and_verify`,
`commit_update` and the self-delete path share one rule. Reproduced by
inverting the pin in `an_own_revocation_on_the_ordinary_path_deletes_the_record`
(org-node/tests/admission_sender.rs): red before (B committed epoch 3 and
stayed Active), green after.
```

Commit: `T12: own removal on every commit path`.

**Red→green attestations:** (DONE 2026-10-06, merged; org-node cargo line
233 passed, 0 failed — commit_paths 24 → 26, everything else as after T11.
PR-322qst marked resolved in its dated file.)

- an_own_revocation_on_the_ordinary_path_deletes_the_record (PR-322qst reproduction; rewritten from the old pin) — runtime-red against the merged code (B kept the record: "PR-322qst: B forgets the record"); also killed by the forget-keeps-provisional and forget-keeps-personas-active mutants; green
- a_node_that_commits_its_own_removal_forgets_the_organisation — runtime-red against the merged code (`commit_update` committed B's own removal as an update); killed by the same two mutants; green
- a_commit_that_keeps_one_of_our_personas_is_an_update — passed against the merged code (a guard on behaviour that already existed); runtime-red under a still_member-always-false mutant; green

Deleted: pr_322qst_an_own_revocation_on_the_ordinary_path_is_committed_not_self_deleted
(LLR-cja9zv) → an_own_revocation_on_the_ordinary_path_deletes_the_record;
LLR-cja9zv keeps its evidence in update_from_the_admin_after_admission_is_committed,
a_committed_admission_reaches_the_disk_and_clears_the_expectation and
commit_update_commits_a_provisional_update_the_chain_carries.

Departure: the forget branch of `receive_and_verify` saves and returns early,
skipping the "mark a Persona Active and bind it" block, so a Persona bound to
another Organisation is not rebound to the one just forgotten (no test covers
that edge).

---

### T13 — org-node: the chain write leaves org-node

**Files touched:** `org-node/src/chain_write/mod.rs` (deleted),
`org-node/src/chain_write/calldata.rs` (deleted),
`org-node/src/chain_write/multisig.rs` (deleted),
`org-node/src/chain_write/proxy.rs` (deleted),
`org-node/src/chain_write/submit.rs` (deleted), `org-node/src/ceremony.rs`
(deleted), `org-node/src/lib.rs`, `org-node/src/service.rs`,
`org-node/tests/chain_write_pure.rs` (deleted),
`org-node/tests/calldata_typed.rs` (deleted),
`org-node/tests/chain_genesis_e2e.rs` (deleted),
`org-node/tests/finality_polling.rs` (deleted),
`org-node/tests/encoding_golden.rs`, `org-node/tests/absences.rs`,
`org-node/Cargo.toml`, `Cargo.lock`, `org-node/.guardrails/config.yaml`
**Parallel:** no (serial, after T12)
**IDs verified:** LLR-rv4vux, LLR-rc74nq, LLR-txqmz4, LLR-66h529,
LLR-463d89, LLR-8m3bwj, LLR-f74xwb, LLR-s7whrn (opaque account), REQ-xs4ab8
("without writing to the chain"), LLR-ayrdr8 (calldata removed).
**Size:** ~900 lines deleted, ~60 written.

**Step 1 — failing test.** Append to `absences.rs`:

```rust
// org-node builds no calldata, derives no multisig account, holds no
// signatory key and dispatches nothing (absences: no input side). The
// account is opaque: nothing converts it to subxt's type.
// verifies: LLR-rv4vux, LLR-rc74nq, LLR-txqmz4, LLR-66h529, LLR-463d89, LLR-8m3bwj, LLR-f74xwb, LLR-s7whrn, REQ-xs4ab8
#[test]
fn org_node_writes_nothing_to_the_chain() {
    for (needle, why) in [
        ("pub mod chain_write", "no chain-write module"),
        ("pub mod ceremony", "no genesis ceremony"),
        ("fn build_update_calldata", "LLR-rv4vux"),
        ("fn update_calldata", "LLR-txqmz4"),
        ("fn revive_update_runtime_call", "LLR-rc74nq"),
        ("UPDATE_SELECTOR", "LLR-66h529"),
        ("fn multi_account_id", "LLR-463d89"),
        ("fn build_dispatch_tx", "LLR-f74xwb"),
        ("fn dispatch_org_call", "LLR-f74xwb"),
        ("sr25519", "LLR-8m3bwj: no signatory key"),
        ("subxt_signer", "LLR-8m3bwj"),
        ("FinalitySink", "no block sink"),
        // a use of subxt's type, not the word: types.rs's doc names the
        // chain's account type to say what `ChainAccount` holds
        ("utils::AccountId32", "LLR-s7whrn: the account is never converted"),
        ("AccountId32::", "LLR-s7whrn: the account is never converted"),
        ("async fn submit_", "no submission"),
    ] {
        assert_absent(needle, why);
    }
    let deps = normal_dependencies();
    for dep in ["subxt-signer", "blake2", "parity-scale-codec"] {
        assert!(!deps.lines().any(|l| l.trim_start().starts_with(dep)), "{dep} is not a dependency");
    }
}
```

Run `--test absences` → FAILED (`pub mod chain_write` found). Record.

**Step 2 — implement (deletions).**
- Delete `org-node/src/chain_write/` (five files) and `org-node/src/ceremony.rs`.
- `lib.rs`: delete `pub mod chain_write;`, `pub mod ceremony;` and the whole
  `test_support` module with its doc comment.
- `service.rs`, `mod subxt_impl`: delete `FinalitySink` and its `BlockSink`
  impl and every `chain_write`/`ceremony`/`subxt_signer`/`tokio::time`
  import; `pub use subxt_impl::SubxtChainOps;` (no `FinalitySink`). Keep
  `connect_chain_client`.
- `Cargo.toml`: `chain = ["dep:subxt", "dep:tokio", "dep:on-chain-client",
  "dep:async-trait", "dep:hex"]` (whatever of today's list remains needed by
  the read half; `cargo check` decides) and its comment "On-chain reads via
  on-chain-client; org-node writes nothing to the chain"; delete the
  optional `subxt-signer`, `blake2`, `parity-scale-codec` dependencies;
  delete the `[[test]]` entries `chain_genesis_e2e`, `finality_polling`,
  `chain_write_pure`, `calldata_typed`; then `grep -rn subxt_signer org-node/tests`
  — if nothing remains (expected: `preflight.rs` does not use it), delete the
  `subxt-signer` dev-dependency; keep `jsonrpsee`, `serde_json`, `libc`
  (`tests/common`, used by `preflight`).
- Delete `org-node/tests/chain_write_pure.rs` — its tests carried
  LLR-rv4vux, LLR-txqmz4, LLR-66h529, LLR-463d89, LLR-8m3bwj, LLR-f74xwb,
  LLR-rc74nq under their old statements; the behaviour moved to
  on-chain-client (T1: `write_pure`) and the new org-node statements (absence)
  are verified by `org_node_writes_nothing_to_the_chain`.
- Delete `org-node/tests/calldata_typed.rs` (LLR-ayrdr8): the calldata is
  on-chain-client's (pinned by `write_pure`'s
  `update_calldata_is_the_pinned_hundred_bytes`); LLR-ayrdr8's amended
  statement is verified by `encoding_golden`'s store, wire and truncation
  tests.
- Delete `org-node/tests/chain_genesis_e2e.rs` (chopsticks, no annotation,
  outside the gate): its write half lives on as on-chain-client's
  `write_genesis_e2e` (T2); its verify half is covered by the gated
  `verify_against_chain` and `commit_paths` targets against the mock.
- Delete `org-node/tests/finality_polling.rs` (chopsticks, no annotation):
  the poll-and-timeout loop it exercised is removed (Decision 9).
- `encoding_golden.rs`: delete `genesis_and_update_calldata_is_pinned`,
  `GOLDEN_GENESIS_CALLDATA`, `GOLDEN_UPDATE_CALLDATA` and the
  `build_update_calldata` import (LLR-ayrdr8; same reason as `calldata_typed`).
- `org-node/.guardrails/config.yaml`: delete ` --test chain_write_pure` and
  ` --test calldata_typed` from the cargo line; the comment block's
  chopsticks-target list becomes "the chopsticks-dependent target
  `preflight`", with a dated line ("T13: chain_genesis_e2e and
  finality_polling deleted with the chain write, which moved to
  on-chain-client").

**Step 3 — green and the lock.** The org-node line (now exactly the one in
this plan's Verification header) passes; `cargo check -p org-node
--all-targets --features app,test-support` passes (only `preflight` remains
outside the gate). `git diff Cargo.lock`: the `org-node` entry loses
`blake2`, `parity-scale-codec` and `subxt-signer`; packages no other member
needs may disappear; no `version =` line of a remaining package changes.
`CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy -p org-node --all-targets --features app,test-support -- -D warnings`
clean. Commit: `T13: the chain write leaves org-node`.

**Red→green attestations:** (DONE 2026-10-06, merged; org-node cargo line
223 passed, 0 failed — 233 − 8 chain_write_pure − 2 calldata_typed − 1
encoding_golden + 1 absences; summed over 23 "test result" lines. on-chain-client
line 1: 73 passed over 10 result lines (the merge run reported 78 for the same
line; T13 touched no on-chain-client file, so the difference is in which
result lines were summed — the gate's per-target counts settle it); line 2
(write): 23 passed. Root Cargo.lock: the org-node entry drops blake2,
parity-scale-codec and subxt-signer; packages subxt-signer 0.50.3, secrecy
0.10.3 and regex 1.13.1 leave; no `version =` line changed.)

- org_node_writes_nothing_to_the_chain — runtime-red before any deletion (`cargo test -p org-node --test absences`: 3 passed, 1 failed — "org-node/src/lib.rs contains `pub mod chain_write`"); green after the deletions

Deleted, with replacements: chain_write_pure ×8 (LLR-rv4vux, LLR-txqmz4,
LLR-66h529, LLR-463d89, LLR-8m3bwj, LLR-f74xwb, LLR-rc74nq) →
absences::org_node_writes_nothing_to_the_chain, the behaviour's evidence now in
on-chain-client's write_pure; calldata_typed ×2 and
encoding_golden::genesis_and_update_calldata_is_pinned (LLR-ayrdr8) →
encoding_golden's remaining tests, with the calldata pinned by
write_pure::update_calldata_is_the_pinned_hundred_bytes; chain_genesis_e2e and
finality_polling (chopsticks, unannotated, outside the gate) →
on-chain-client/tests/write_genesis_e2e.rs (T2).

Left for T17: the `hex` dev-dependency in org-node appears unused by the tests.

---

### T14 — app: the chain writer seam, submit-then-commit, and its timeout (REQ-nfr3n2)

**Files touched:** `app/src-tauri/Cargo.toml`, `app/src-tauri/Cargo.lock`,
`app/src-tauri/src/submit.rs` (new), `app/src-tauri/src/state.rs`,
`app/src-tauri/src/commands.rs`, `app/src-tauri/src/events.rs`,
`app/src-tauri/src/lib.rs`, `app/src-tauri/tests/submit_flow.rs` (new),
`app/src-tauri/tests/ipc.rs`, `app/src-tauri/tests/receiver_events.rs`,
`app/.guardrails/config.yaml`
**Parallel:** no (serial, after T13)
**IDs verified:** REQ-nfr3n2, LLR-qhjp6g, LLR-be3zv9; LLR-7bk6qh (as
amended: the four new receiver errors); LLR-6pmrma, LLR-ty85xv, LLR-n6twt7,
LLR-vzf8j2 (revoke half), LLR-4wcyqy (ipc tests on the revised surface).
**Size:** ~460 lines.

This task makes the app compile against the new org-node with the commands
that do not need the invitation exchange; T15 adds the invitation commands
back in their new form. Until then the ipc suite's tests of `export_invite`,
`import_invite`, `export_join_request`, `import_join_request` and
`admit_member` are deleted (listed in Step 1) and re-written in T15 under the
same LLRs; their LLRs show MISSING-TEST between the two tasks, which is
expected (check-trace is required green only at T17).

**Dispatcher addition (2026-10-06).** on-chain-client's writer has no timer
(no tokio), so its finality wait can wait forever (Open question 2).
`submit.rs` therefore wraps every writer call in
`tokio::time::timeout(Duration::from_secs(90), …)`, the settle timeout
org-node used before, and treats an elapsed timeout as a failed submission
under REQ-nfr3n2: org-node is asked neither to commit nor to send, and the
failure is reported (LLR-be3zv9).

**Step 1 — failing tests.** `app/src-tauri/tests/submit_flow.rs`:

```rust
#![cfg(feature = "test-support")]
//! REQ-nfr3n2: the app writes org-node's provisional update to the chain
//! through on-chain-client, and only once that write has executed asks
//! org-node to commit and send it; a failed or timed-out write asks for
//! neither and is reported (LLR-qhjp6g, LLR-be3zv9). The chain write is a
//! substitute here (`FakeWriter`) over org-node's `MockChainOps`, which the
//! service reads.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use ods_poc_lib::submit::{found_organisation, submit_commit_send, ChainWriter};
use org_node::service::{MockChainOps, OrgService};
use org_node::store::PersonaStore;
use org_node::transport::endpoint::OrgEndpoint;
use org_node::{ChainAccount, DeviceSeed, Epoch, Handle, Joiner, MemberSeed, Name, OrgId, OrgPublicKey, RootHash, Surname};
use rand::rngs::OsRng;

struct FakeWriter {
    chain: MockChainOps,
    failing: AtomicBool,
    hanging: AtomicBool,
    proxies: Mutex<HashMap<[u8; 32], OrgId>>,
}

impl FakeWriter {
    fn over(chain: &MockChainOps) -> Self {
        Self { chain: chain.clone(), failing: AtomicBool::new(false), hanging: AtomicBool::new(false), proxies: Mutex::new(HashMap::new()) }
    }
    /// A writer whose every call never finishes.
    fn never(chain: &MockChainOps) -> Self {
        let w = Self::over(chain);
        w.hanging.store(true, Ordering::SeqCst);
        w
    }
    async fn gate(&self) -> Result<(), String> {
        if self.hanging.load(Ordering::SeqCst) {
            std::future::pending::<()>().await;
        }
        if self.failing.load(Ordering::SeqCst) {
            return Err("node unreachable".into());
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl ChainWriter for FakeWriter {
    async fn genesis(&self, root: RootHash, key: OrgPublicKey) -> Result<(OrgId, ChainAccount), String> {
        self.gate().await?;
        let org = self.chain.apply_genesis(root, key);
        let mut proxy = [0u8; 32];
        proxy[..20].copy_from_slice(org.as_bytes());
        self.proxies.lock().unwrap().insert(proxy, org);
        Ok((org, ChainAccount::new(proxy)))
    }
    async fn update(&self, proxy: ChainAccount, root: RootHash, key: OrgPublicKey, epoch: Epoch) -> Result<(), String> {
        self.gate().await?;
        let org = *self.proxies.lock().unwrap().get(proxy.as_bytes()).ok_or("unknown proxy")?;
        self.chain.apply_update(org, root, key, epoch).map_err(|e| e.to_string())
    }
}

fn service(tag: &str, chain: &MockChainOps) -> OrgService {
    let dir = std::env::temp_dir().join(format!("ods-app-submit-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    OrgService::new(PersonaStore::open(dir.join("store.bin"), "pw").unwrap(), Box::new(chain.clone()))
}

fn persona(svc: &mut OrgService, handle: &str) -> org_node::PersonaId {
    svc.create_persona(&mut OsRng, Handle::parse(handle).unwrap(), Name::parse("Test").unwrap(), Surname::parse("User").unwrap())
        .unwrap()
}

fn joiner(seed: u8, handle: &str) -> Joiner {
    Joiner {
        handle: Handle::parse(handle).unwrap(),
        name: Name::parse("Test").unwrap(),
        surname: Surname::parse("Joiner").unwrap(),
        member_key: MemberSeed::from([seed; 32]).x25519_keypair().member_key().unwrap(),
        device_key: DeviceSeed::from([seed + 1; 32]).signing_keypair().device_key().unwrap(),
    }
}

// verifies: LLR-qhjp6g, LLR-be3zv9
#[tokio::test]
async fn founding_writes_the_chain_then_commits() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let mut a = service("found", &chain);
    let pid = persona(&mut a, "alice");
    let org = found_organisation(&mut a, &writer, &mut OsRng, &pid).await.unwrap();
    let rec = a.list_orgs().iter().find(|o| o.org_id == org).cloned().unwrap();
    assert_eq!((rec.epoch, rec.last_seq.get()), (Epoch::new(1), 1));
    assert!(rec.proxy_account.is_some());
    assert!(rec.org_private_key.is_some());
}

// verifies: LLR-qhjp6g
#[tokio::test]
async fn a_failed_genesis_write_commits_nothing_and_reports_the_failure() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    writer.failing.store(true, Ordering::SeqCst);
    let mut a = service("found-fails", &chain);
    let pid = persona(&mut a, "alice");
    let err = found_organisation(&mut a, &writer, &mut OsRng, &pid).await.unwrap_err();
    assert!(err.contains("node unreachable"), "the failure is reported: {err}");
    assert!(a.list_orgs().is_empty(), "nothing committed");
    assert_eq!(a.genesis_provisional_updates(&pid).len(), 1, "the update is kept for a retry");
}

// verifies: LLR-qhjp6g
#[tokio::test(flavor = "multi_thread")]
async fn an_admission_is_written_then_committed_then_sent() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let mut a = service("admit-a", &chain);
    let pid_a = persona(&mut a, "alice");
    let org = found_organisation(&mut a, &writer, &mut OsRng, &pid_a).await.unwrap();
    let mut b = service("admit-b", &chain);
    let pid_b = persona(&mut b, "bob");
    let invite = org_node::InviteId::new([0x44; 32]);
    b.expect_admission(&mut OsRng, org, invite).unwrap();
    let rec_b = b.list_personas()[0].clone();
    let (member_key, device_key) = b.persona_public_keys(&pid_b).unwrap();
    let bob = Joiner { handle: rec_b.handle, name: rec_b.name, surname: rec_b.surname, member_key, device_key };
    let ep_b = OrgEndpoint::bind(&rec_b.device_seed.signing_keypair()).await.unwrap();
    let addr_b = ep_b.inner().addr();
    let mut b = b.with_endpoint(ep_b);
    let task = tokio::spawn(async move {
        let r = tokio::time::timeout(Duration::from_secs(30), b.receive_and_verify(&mut OsRng)).await.unwrap();
        (b, r)
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    let update = a.admit_member(&mut OsRng, org, &bob).unwrap();
    let out = submit_commit_send(&mut a, &writer, &mut OsRng, &update, device_key, Some(addr_b), None, Some(invite)).await.unwrap();
    assert_eq!(out.epoch, Epoch::new(2));
    let (b, received) = task.await.unwrap();
    assert_eq!(received.unwrap().epoch, Epoch::new(2));
    // A device holding no proxy account cannot submit: B was admitted, it
    // did not found the Organisation, so its record holds none.
    let mut b = b;
    let carol = joiner(0x71, "carol");
    let update = b.admit_member(&mut OsRng, org, &carol).unwrap();
    let err = submit_commit_send(&mut b, &writer, &mut OsRng, &update, carol.device_key, None, None, None).await.unwrap_err();
    assert!(err.contains("proxy account"), "{err}");
    assert_eq!(b.list_orgs()[0].epoch, Epoch::new(2), "nothing committed on B");
}

// verifies: LLR-qhjp6g
#[tokio::test]
async fn a_failed_update_write_neither_commits_nor_sends() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let mut a = service("update-fails", &chain);
    let pid = persona(&mut a, "alice");
    let org = found_organisation(&mut a, &writer, &mut OsRng, &pid).await.unwrap();
    let bob = joiner(0x61, "bob");
    let update = a.admit_member(&mut OsRng, org, &bob).unwrap();
    writer.failing.store(true, Ordering::SeqCst);
    let err = submit_commit_send(&mut a, &writer, &mut OsRng, &update, bob.device_key, None, None, None).await.unwrap_err();
    assert!(err.contains("node unreachable"), "{err}");
    assert_eq!(a.list_orgs()[0].epoch, Epoch::new(1), "nothing committed");
    assert_eq!(a.provisional_updates(org).len(), 1, "the update is kept");
    assert!(a.endpoint().is_none(), "nothing sent: no endpoint was bound");
}

// verifies: LLR-be3zv9
#[tokio::test(start_paused = true)]
async fn a_submission_that_never_finishes_times_out_and_nothing_is_committed() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::never(&chain);
    let mut a = service("never", &chain);
    let pid = persona(&mut a, "alice");
    let err = found_organisation(&mut a, &writer, &mut OsRng, &pid).await.unwrap_err();
    assert!(err.contains("timed out"), "{err}");
    assert!(a.list_orgs().is_empty(), "nothing committed");
    assert_eq!(a.genesis_provisional_updates(&pid).len(), 1, "the update is kept");
    // The update path is bounded the same way.
    let writer = FakeWriter::over(&chain);
    let org = found_organisation(&mut a, &writer, &mut OsRng, &pid).await.unwrap();
    writer.hanging.store(true, Ordering::SeqCst);
    let bob = joiner(0x61, "bob");
    let update = a.admit_member(&mut OsRng, org, &bob).unwrap();
    let err = submit_commit_send(&mut a, &writer, &mut OsRng, &update, bob.device_key, None, None, None).await.unwrap_err();
    assert!(err.contains("timed out"), "{err}");
    assert_eq!(a.list_orgs()[0].epoch, Epoch::new(1), "nothing committed");
    assert!(a.endpoint().is_none(), "nothing sent");
}
```

(`list_orgs()[0]` is safe: each service holds one record. B's
`admit_member` succeeds because B's Persona was bound by its first admission
— LLR-e5c9ud. The "never" writer's red, before the timeout exists, is a hang:
run it alone under `timeout 30 cargo test … --test submit_flow -- --exact
a_submission_that_never_finishes_times_out_and_nothing_is_committed` and
record the hang.)

`receiver_events.rs` (LLR-7bk6qh as amended): in
`locally_reachable_variants_are_classified_as_receiver_errors` add
`OrgNodeError::AdmissionNotExpected { org_id: org_node::OrgId::new([1; 20]) }`,
`OrgNodeError::AdmissionNotOurs { org_id: org_node::OrgId::new([1; 20]) }`,
`OrgNodeError::ProvisionalLimit { limit: 1 }` and
`OrgNodeError::NoProvisionalUpdate` to the list (annotation LLR-7bk6qh
unchanged); the comment gains "an unexpected first admission is refused
before anything is verified, one that lists none of this node's Personas on
a rule about this node, and the other two never arise on receive".

`ipc.rs`: the handler list becomes `create_persona, create_organisation,
revoke_member, list_personas, list_orgs, connection_status, start_receiver`
(T15 adds the invitation commands back, restoring twelve); **delete**, to be
re-written in T15 under the same annotations:
`export_invite_rejects_a_short_org_id`, `export_invite_rejects_a_non_hex_org_id`,
`export_invite_does_not_refuse_a_well_formed_org_id` (LLR-vzf8j2),
`admit_member_refuses_a_malformed_org_id_with_the_parsers_message`,
`admit_member_does_not_refuse_a_well_formed_org_id` (LLR-vzf8j2),
`admit_member_refuses_an_org_secret_that_is_not_32_bytes`,
`admit_member_refuses_an_org_secret_that_is_not_hex` (LLR-8krgzj),
`admit_member_does_not_refuse_a_32_byte_or_absent_org_secret` (LLR-8krgzj,
LLR-ctrfz4), `admit_member_does_not_refuse_a_decodable_node_addr`,
`admit_member_refuses_a_node_addr_that_is_not_an_endpoint_addr`
(LLR-ctrfz4), `import_join_request_reports_the_request_it_decodes`,
`import_join_request_refuses_a_malformed_blob` (LLR-pmus9f), and the helpers
`join_request_with_node_addr` and `admit_args`. The `revoke_member` tests
(LLR-vzf8j2, LLR-6pmrma, LLR-ty85xv, LLR-n6twt7) stay, unchanged: an unknown
Organisation is still refused with `OrgNodeError::OrgNotOnChain`'s message.
The module comment's "all twelve handlers" → "every handler".

Add to `app/src-tauri/Cargo.toml`: `[[test]] name = "submit_flow"`,
`required-features = ["test-support"]`; change the on-chain-client
dependency to `features = ["dev-rpc", "write"]`; add `async-trait` and
`subxt-signer` to `[dependencies]` if `submit.rs` needs them and they are not
there (versions as the lock already resolves them). Add ` --test submit_flow`
to the app's cargo line in `app/.guardrails/config.yaml`.

**Step 2 — run, expect red.**
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test --manifest-path app/src-tauri/Cargo.toml --features test-support --test submit_flow`
→ the crate does not compile (E0432 `ods_poc_lib::submit`, and the
`commands.rs`/`state.rs`/`events.rs` errors left by T6–T13). Record.

**Step 3 — implement.** `app/src-tauri/src/submit.rs`:

```rust
//! Submitting org-node's provisional updates (REQ-nfr3n2): the app writes the
//! chain through on-chain-client, then asks org-node to commit and send;
//! when the write fails, or does not finish within `WRITE_TIMEOUT`, it asks
//! org-node for neither and reports the failure (LLR-qhjp6g, LLR-be3zv9).

use std::future::Future;
use std::time::Duration;

use async_trait::async_trait;
use on_chain_client::write::subxt_ops::{FinalitySink, SubxtWriteOps};
use on_chain_client::write::{genesis, submit_update, AccountId};
use on_chain_client::{OnChainRootHash, OrgPubKey};
use org_node::service::OrgService;
use org_node::store::ProvisionalUpdate;
use org_node::{
    ChainAccount, CommitOutcome, DevicePublicKey, Epoch, InviteId, OrgId, OrgNodeError, OrgPublicKey, OrgSecret, PersonaId,
    RootHash,
};
use rand::{CryptoRng, RngCore};

/// How long one chain write may take before the app gives up on it
/// (LLR-be3zv9): the settle timeout org-node used before the write moved.
pub const WRITE_TIMEOUT: Duration = Duration::from_secs(90);

/// The chain write as the app performs it.
#[async_trait]
pub trait ChainWriter: Send + Sync {
    /// Stand up the Organisation's slot: its id and its proxy account.
    async fn genesis(&self, root: RootHash, key: OrgPublicKey) -> Result<(OrgId, ChainAccount), String>;
    /// Move the slot, through `proxy`, from `expected_epoch`.
    async fn update(&self, proxy: ChainAccount, root: RootHash, key: OrgPublicKey, expected_epoch: Epoch)
        -> Result<(), String>;
}

/// One writer call, bounded by `WRITE_TIMEOUT`; a timeout is a failed write.
async fn bounded<T>(write: impl Future<Output = Result<T, String>>) -> Result<T, String> {
    tokio::time::timeout(WRITE_TIMEOUT, write)
        .await
        .map_err(|_| format!("the chain write timed out after {} s", WRITE_TIMEOUT.as_secs()))?
}

/// The production writer: on-chain-client's, over subxt.
pub struct OnChainWriter {
    pub ops: SubxtWriteOps<FinalitySink>,
    pub signatory: subxt_signer::sr25519::Keypair,
    pub co_signatories: Vec<AccountId>,
    pub contract: [u8; 20],
}

#[async_trait]
impl ChainWriter for OnChainWriter {
    async fn genesis(&self, root: RootHash, key: OrgPublicKey) -> Result<(OrgId, ChainAccount), String> {
        let g = genesis(&self.ops, &self.signatory, &self.co_signatories, self.contract, OnChainRootHash(*root.as_bytes()), OrgPubKey(*key.as_bytes()))
            .await
            .map_err(|e| e.to_string())?;
        Ok((OrgId::new(g.admin.0), ChainAccount::new(g.proxy.0)))
    }
    async fn update(&self, proxy: ChainAccount, root: RootHash, key: OrgPublicKey, expected_epoch: Epoch) -> Result<(), String> {
        submit_update(
            &self.ops,
            &self.signatory,
            &self.co_signatories,
            self.contract,
            AccountId(*proxy.as_bytes()),
            OnChainRootHash(*root.as_bytes()),
            OrgPubKey(*key.as_bytes()),
            on_chain_client::Epoch(expected_epoch.get()),
        )
        .await
        .map_err(|e| e.to_string())
    }
}

/// The writer when the chain is not configured: every write refused.
pub struct WriterNotConfigured;

#[async_trait]
impl ChainWriter for WriterNotConfigured {
    async fn genesis(&self, _: RootHash, _: OrgPublicKey) -> Result<(OrgId, ChainAccount), String> {
        Err("chain not configured: set ODS_CHAIN_WS, ODS_CONTRACT_H160, ODS_ADMIN_SEED".into())
    }
    async fn update(&self, _: ChainAccount, _: RootHash, _: OrgPublicKey, _: Epoch) -> Result<(), String> {
        Err("chain not configured: set ODS_CHAIN_WS, ODS_CONTRACT_H160, ODS_ADMIN_SEED".into())
    }
}

/// Found an Organisation: build the genesis update, write it, commit it.
pub async fn found_organisation<R: RngCore + CryptoRng + Send>(
    svc: &mut OrgService,
    writer: &dyn ChainWriter,
    rng: &mut R,
    persona_id: &PersonaId,
) -> Result<OrgId, String> {
    let update = svc.create_organisation(rng, persona_id).map_err(|e| e.to_string())?;
    let (org_id, proxy) = bounded(writer.genesis(update.resulting_root, update.org_pub_key))
        .await
        .map_err(|e| format!("chain write failed; nothing committed: {e}"))?;
    svc.commit_genesis(rng, persona_id, org_id, proxy).await.map_err(|e| e.to_string())?;
    Ok(org_id)
}

/// Write `update` to the chain; only once that executed, commit it and send
/// it to `recipient`, under `invite_id` for an admission (REQ-nfr3n2).
#[allow(clippy::too_many_arguments)]
pub async fn submit_commit_send<R: RngCore + CryptoRng + Send>(
    svc: &mut OrgService,
    writer: &dyn ChainWriter,
    rng: &mut R,
    update: &ProvisionalUpdate,
    recipient: DevicePublicKey,
    peer_addr: Option<iroh::EndpointAddr>,
    org_secret: Option<OrgSecret>,
    invite_id: Option<InviteId>,
) -> Result<CommitOutcome, String> {
    let org_id = update.org_id.ok_or("a genesis update is founded, not submitted")?;
    let rec = svc
        .list_orgs()
        .iter()
        .find(|o| o.org_id == org_id)
        .cloned()
        .ok_or_else(|| OrgNodeError::OrgNotOnChain.to_string())?;
    let proxy = rec
        .proxy_account
        .ok_or("this device holds no proxy account for the Organisation; only its founding device can submit")?;
    bounded(writer.update(proxy, update.resulting_root, update.org_pub_key, rec.epoch))
        .await
        .map_err(|e| format!("chain write failed; nothing committed or sent: {e}"))?;
    let outcome = svc.commit_update(rng, org_id).await.map_err(|e| e.to_string())?;
    svc.send_update(&outcome.outgoing, recipient, peer_addr, org_secret, invite_id)
        .await
        .map_err(|e| format!("committed at epoch {}, but the send failed: {e}", outcome.epoch.get()))?;
    Ok(outcome)
}
```

`state.rs`:
- `ChainNotConfigured` keeps only `read_state`.
- `AppState` gains `pub writer: Box<dyn crate::submit::ChainWriter>`;
  `assemble` takes it as a parameter; `for_test` passes
  `Box::new(WriterNotConfigured)` (its own signature unchanged, so
  `state_assembly.rs` is untouched).
- `build_chain_ops` returns `(Box<dyn ChainOps>, Box<dyn ChainWriter>,
  policy::ChainEndpoint)`; `connect_chain` builds
  `org_node::service::connect_chain_client(&ws_url, contract_h160)` → `(api,
  registry)`, then `org_node::SubxtChainOps::new(registry)` and
  `OnChainWriter { ops: SubxtWriteOps::new(api, FinalitySink), signatory:
  Keypair::from_secret_key(admin_seed)?, co_signatories: others.iter().map(|a| AccountId(*a.as_bytes())).collect(), contract: contract_h160 }`.
  `init` falls back to `(ChainNotConfigured, WriterNotConfigured, None)`.

`events.rs`: in the receive-error arm add `| OrgNodeError::AdmissionNotExpected { .. }
| OrgNodeError::AdmissionNotOurs { .. } | OrgNodeError::ProvisionalLimit { .. }
| OrgNodeError::NoProvisionalUpdate` (Decision 14).

`commands.rs`: delete the `JoinRequest` import, `export_invite`,
`import_invite`, `export_join_request`, `import_join_request`,
`decode_join_request`, `JoinRequestDto`, `admit_member` and `OrgSecret`
(re-added in T15); `create_organisation` becomes

```rust
    let mut svc = state.service.lock().await;
    let org_id = crate::submit::found_organisation(&mut svc, &*state.writer, &mut OsRng, &PersonaId::new(persona_id)).await?;
    Ok(hex::encode(org_id.as_bytes()))
```

and `revoke_member`, after parsing (unchanged), becomes

```rust
    let mut svc = state.service.lock().await;
    let member = MemberId::new(member_id);
    let recipient = svc
        .list_orgs()
        .iter()
        .find(|o| o.org_id == oid)
        .ok_or_else(|| OrgNodeError::OrgNotOnChain.to_string())?
        .trie_members
        .iter()
        .find(|m| m.id == member)
        .and_then(|m| m.device_keys.first().copied())
        .ok_or("member_id names no member of this organisation")?;
    let update = svc.revoke_member(&mut OsRng, oid, member).map_err(|e| e.to_string())?;
    crate::submit::submit_commit_send(&mut svc, &*state.writer, &mut OsRng, &update, recipient, peer_addr, None, None)
        .await
        .map(|_| ())
```

(`revoke_member_is_not_refused_for_an_empty_peer_addr` keeps passing: an
unknown organisation is still refused with `OrgNodeError::OrgNotOnChain`'s
message.) `lib.rs`: `pub mod submit;` and the handler list of the ipc
table above.

**Step 4 — green and the lock.** The app's cargo line (with
`--test submit_flow`): every target passes; submit_flow 5 passed.
`npm --prefix app run check` passes (the TypeScript still names the removed
commands as strings; nothing type-checks a command name); `npm --prefix app
run test` passes. `git diff app/src-tauri/Cargo.lock`: the
`on-chain-client` entry gains `blake2`/`subxt-signer`, the `org-node` entry
loses its removed dependencies; no `version =` line of a remaining package
changes. `CARGO_HOME=/tmp/cargo_home_fuzz cargo tree --manifest-path app/src-tauri/Cargo.toml -i subxt-signer`
prints exactly one `subxt-signer v0.50.3`. Commit:
`T14: app writes the chain through on-chain-client, then commits; 90 s bound`.

**Red→green attestations:** (DONE 2026-10-06, merged. App cargo line with
`--test submit_flow`: 128 passed, 0 failed — startup_policy 13,
connection_status 10, org_id_parsing 14, receiver_events 32, receiver_guard 6,
ipc 21 (33 − 12 deleted), csp_policy 24, state_assembly 3, submit_flow 5.
`npm run check`: 0 errors, 1 existing warning; `npm run test`: 36 passed.
org-node: 223 passed. app/src-tauri/Cargo.lock: on-chain-client gains blake2
and subxt-signer, org-node loses base64, blake2, parity-scale-codec and
subxt-signer; no version line changed; cargo's format bump `version = 3 → 4`
was set back to 3 and `cargo metadata --locked` passes.) Every test was
compile-red first (the app lib had 14 errors left by T6–T13), then:

- founding_writes_the_chain_then_commits — runtime-red against `Err("stub")` stubs; green
- a_failed_genesis_write_commits_nothing_and_reports_the_failure — runtime-red against the stubs ("stub" ≠ "node unreachable"); green
- an_admission_is_written_then_committed_then_sent — runtime-red against the stubs; green
- a_failed_update_write_neither_commits_nor_sends — runtime-red against the stubs; green
- a_submission_that_never_finishes_times_out_and_nothing_is_committed — runtime-red against the stubs ("stub" ≠ "timed out"); under a no-timeout mutant it hung (killed after 30 s with no result); green with the 90 s `tokio::time::timeout` on the paused clock
- locally_reachable_variants_are_classified_as_receiver_errors (rewritten, four variants added) — compile-red (E0004); runtime-red under a mutant classifying them as verdicts; green
- product_registers_the_same_commands_as_the_harness (rewritten, 12 → 7) — runtime-red with the harness trimmed and the old assertion kept; green

Deleted until T15 rewrites them under the same annotations: the export_invite,
admit_member, import_join_request ipc tests (LLR-vzf8j2, LLR-8krgzj,
LLR-ctrfz4, LLR-pmus9f). Commands absent until T15: export_invite,
import_invite, export_join_request, import_join_request, admit_member.
Departures: tokio gains `time` (and `test-util` as a dev-dependency); a
`ChainWiring` alias in state.rs; ODS_COSIGNER_PUB parses into on-chain-client's
`AccountId`.


**Files touched:** `app/src-tauri/Cargo.toml`, `app/src-tauri/Cargo.lock`,
`app/src-tauri/src/invitation.rs` (new), `app/src-tauri/src/state.rs`,
`app/src-tauri/src/commands.rs`, `app/src-tauri/src/lib.rs`,
`app/src-tauri/tests/invitation.rs` (new), `app/src-tauri/tests/ipc.rs`,
`app/.guardrails/config.yaml`
**Parallel:** no (serial, after T14)
**IDs verified:** REQ-prjja8, REQ-tcutr6, REQ-65xqp8, REQ-ab2mfz (the
backend's refusal without confirmation; the warning itself is T16),
REQ-yazum3, REQ-nfr3n2 (admission through the reply), LLR-b7wgpf,
LLR-9sraks, LLR-f35pda, LLR-w4mhd4, LLR-gha5f6, LLR-vzf8j2, LLR-8krgzj,
LLR-ctrfz4, LLR-pmus9f, LLR-4wcyqy.
**Size:** ~520 lines.

`Cargo.lock` here is `app/src-tauri/Cargo.lock`, edited only by T14 and this
task (serial). This task adds `base64 = "0.22"` to the app's
`[dependencies]`; the lock already resolves `base64 0.22.x` (confirm with
`cargo tree --manifest-path app/src-tauri/Cargo.toml -i base64@0.22` before
the edit); no new version may appear.

**Step 1 — failing tests.** `app/src-tauri/tests/invitation.rs`:

```rust
#![cfg(feature = "test-support")]
//! The invitation exchange (REQ-prjja8, REQ-tcutr6, REQ-65xqp8, REQ-ab2mfz,
//! REQ-yazum3): the Invite and the Invite reply as Blobs, parsed at this edge.

use ods_poc_lib::invitation::{
    admit_reply, check_reply, issue_invite, produce_reply, Invite, InviteReply, OutstandingInvites,
};
use ods_poc_lib::submit::{found_organisation, ChainWriter};
use org_node::service::{MockChainOps, OrgService};
use org_node::store::PersonaStore;
use org_node::{DeviceSeed, Handle, InviteId, MemberSeed, Name, OrgId, Surname};
use rand::rngs::OsRng;
```

followed by a `FakeWriter` identical to `submit_flow.rs`'s (copy it: two test
crates cannot share a module without a `tests/common`, and this is the only
duplicate), a `dir(tag)` helper (a fresh, wiped directory under `temp_dir()`
named for the tag and the process id), and:

```rust
/// A founded Organisation on A, and A's outstanding-invites file (path kept,
/// so a test can reopen what was written).
async fn founded(tag: &str, chain: &MockChainOps, writer: &FakeWriter) -> (OrgService, OrgId, OutstandingInvites, std::path::PathBuf) {
    let d = dir(&format!("{tag}-a"));
    let mut a = OrgService::new(PersonaStore::open(d.join("a.bin"), "pw").unwrap(), Box::new(chain.clone()));
    let pid = a.create_persona(&mut OsRng, Handle::parse("alice").unwrap(), Name::parse("Alice").unwrap(), Surname::parse("Smith").unwrap()).unwrap();
    let org = found_organisation(&mut a, writer, &mut OsRng, &pid).await.unwrap();
    let path = d.join("outstanding_invites.json");
    (a, org, OutstandingInvites::open(path.clone()).unwrap(), path)
}

fn joiner_service(tag: &str, chain: &MockChainOps) -> (OrgService, org_node::PersonaId) {
    let mut b = OrgService::new(PersonaStore::open(dir(&format!("{tag}-b")).join("b.bin"), "pw").unwrap(), Box::new(chain.clone()));
    let pid = b.create_persona(&mut OsRng, Handle::parse("bob").unwrap(), Name::parse("Bob").unwrap(), Surname::parse("Builder").unwrap()).unwrap();
    (b, pid)
}

fn keys() -> ([u8; 32], [u8; 32]) {
    let member = MemberSeed::from([8; 32]).x25519_keypair().member_key().unwrap();
    let device = DeviceSeed::from([7; 32]).signing_keypair().device_key().unwrap();
    (*member.as_bytes(), *device.as_bytes())
}

// verifies: LLR-9sraks
#[tokio::test]
async fn an_invite_carries_what_the_inviter_typed_and_a_fresh_outstanding_id() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (a, org, mut outstanding, _) = founded("issue", &chain, &writer).await;
    let blob = issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme Co-op", "Bob Builder").unwrap();
    let invite = Invite::parse(&blob).unwrap();
    assert_eq!((invite.org_name.as_str(), invite.org_id, invite.invitee_name.as_str()), ("Acme Co-op", org, "Bob Builder"));
    let (_, device) = a.persona_public_keys(&a.list_personas()[0].persona_id).unwrap();
    assert_eq!(invite.inviter_device_keys, vec![device]);
    assert!(outstanding.holds(&invite.invite_id));
    let other = Invite::parse(&issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme Co-op", "Carol").unwrap()).unwrap();
    assert_ne!(other.invite_id, invite.invite_id, "drawn at random");
}

// verifies: LLR-9sraks
#[tokio::test]
async fn no_invite_is_issued_for_an_organisation_no_persona_here_belongs_to() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (a, _org, mut outstanding, _) = founded("issue-none", &chain, &writer).await;
    assert!(issue_invite(&a, &mut outstanding, &mut OsRng, OrgId::new([9; 20]), "X", "Y").is_err());
    assert!(outstanding.is_empty());
}

// verifies: LLR-f35pda
#[tokio::test]
async fn outstanding_invite_ids_survive_a_reopen_and_settle_one_at_a_time() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (a, org, mut outstanding, path) = founded("reopen", &chain, &writer).await;
    let first = Invite::parse(&issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Bob").unwrap()).unwrap().invite_id;
    let second = Invite::parse(&issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Carol").unwrap()).unwrap().invite_id;
    let mut reopened = OutstandingInvites::open(path.clone()).unwrap();
    assert!(reopened.holds(&first) && reopened.holds(&second), "kept across a restart");
    reopened.settle(&first).unwrap();
    let again = OutstandingInvites::open(path).unwrap();
    assert!(!again.holds(&first) && again.holds(&second), "settle removes one and keeps the other");
}

// verifies: LLR-f35pda
#[test]
fn an_outstanding_invites_file_that_is_not_a_hex_list_is_refused() {
    let d = dir("bad-outstanding");
    for text in ["{}", "[\"zz\"]", "[\"abcd\"]"] {
        let path = d.join("outstanding_invites.json");
        std::fs::write(&path, text).unwrap();
        let err = OutstandingInvites::open(path).err().expect("refused");
        assert!(err.starts_with("outstanding invite"), "{text}: {err}");
    }
}

// verifies: LLR-b7wgpf
#[test]
fn an_invite_and_a_reply_that_parse_encode_back_to_the_same_blob() {
    let (mk, dk) = keys();
    let invite = Invite::wire_for_test("Acme", &[1; 20], "Bob", &[dk], &[3; 32]);
    assert_eq!(Invite::parse(&invite).unwrap().encode().unwrap(), invite);
    let reply = InviteReply::wire_for_test(&[1; 20], &[3; 32], &mk, &dk, "bob", "Bob", "Builder");
    assert_eq!(InviteReply::parse(&reply).unwrap().encode().unwrap(), reply);
}

// verifies: LLR-b7wgpf
#[test]
fn an_invite_or_reply_that_does_not_parse_is_refused_naming_the_field() {
    let (mk, dk) = keys();
    for (blob, field) in [
        (Invite::wire_for_test("O", &[1; 19], "N", &[dk], &[3; 32]), "invite.org_id"),
        (Invite::wire_for_test("O", &[1; 20], "N", &[[0xff; 32]], &[3; 32]), "invite.inviter_device_keys"),
        (Invite::wire_for_test("O", &[1; 20], "N", &[dk], &[3; 31]), "invite.invite_id"),
        ("not base64 !".to_string(), "invite"),
    ] {
        let err = Invite::parse(&blob).unwrap_err();
        assert!(err.starts_with(field), "{field}: {err}");
    }
    let reply = |org: &[u8], id: &[u8], mk: [u8; 32], dk: [u8; 32], handle: &str, name: &str, surname: &str| {
        InviteReply::wire_for_test(org, id, &mk, &dk, handle, name, surname)
    };
    let long = "a".repeat(129);
    for (blob, field) in [
        (reply(&[1; 21], &[3; 32], mk, dk, "bob", "Bob", "Builder"), "reply.org_id"),
        (reply(&[1; 20], &[3; 31], mk, dk, "bob", "Bob", "Builder"), "reply.invite_id"),
        (reply(&[1; 20], &[3; 32], [0xff; 32], dk, "bob", "Bob", "Builder"), "reply.member_key"),
        (reply(&[1; 20], &[3; 32], mk, [0xff; 32], "bob", "Bob", "Builder"), "reply.device_key"),
        (reply(&[1; 20], &[3; 32], mk, dk, "Bob", "Bob", "Builder"), "reply.handle"),
        (reply(&[1; 20], &[3; 32], mk, dk, "bob", &long, "Builder"), "reply.name"),
        (reply(&[1; 20], &[3; 32], mk, dk, "bob", "Bob", &long), "reply.surname"),
        ("not base64 !".to_string(), "reply"),
    ] {
        let err = InviteReply::parse(&blob).unwrap_err();
        assert!(err.starts_with(field), "{field}: {err}");
    }
}

// verifies: LLR-b7wgpf
#[tokio::test]
async fn a_reply_that_does_not_parse_acts_on_nothing() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (mut a, org, mut outstanding, _) = founded("bad-reply", &chain, &writer).await;
    issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Bob").unwrap();
    let before = a.list_orgs()[0].clone();
    assert!(admit_reply(&mut a, &writer, &mut outstanding, &mut OsRng, org, "not base64 !", None, None).await.is_err());
    assert!(!outstanding.is_empty(), "nothing settled");
    assert_eq!(a.list_orgs()[0].trie_members.len(), before.trie_members.len());
    assert!(a.provisional_updates(org).is_empty(), "no service call");
}

// verifies: LLR-w4mhd4
#[tokio::test]
async fn a_confirmed_reply_carries_the_persona_and_declares_the_expected_admission() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (a, org, mut outstanding, _) = founded("reply", &chain, &writer).await;
    let invite = issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Bob").unwrap();
    let invite_id = Invite::parse(&invite).unwrap().invite_id;
    let (mut b, pid) = joiner_service("reply", &chain);
    let reply = InviteReply::parse(&produce_reply(&mut b, &mut OsRng, &invite, &pid, true).unwrap()).unwrap();
    let (mk, dk) = b.persona_public_keys(&pid).unwrap();
    assert_eq!((reply.member_key, reply.device_key, reply.handle.as_str()), (mk, dk, "bob"));
    assert_eq!((reply.org_id, reply.invite_id), (org, invite_id));
    assert_eq!(b.expected_admissions(), &[org_node::store::ExpectedAdmission { org_id: org, invite_id }]);
}

// verifies: LLR-w4mhd4
#[tokio::test]
async fn no_reply_is_produced_and_nothing_declared_without_confirmation() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (a, org, mut outstanding, _) = founded("unconfirmed", &chain, &writer).await;
    let invite = issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Bob").unwrap();
    let (mut b, pid) = joiner_service("unconfirmed", &chain);
    let err = produce_reply(&mut b, &mut OsRng, &invite, &pid, false).unwrap_err();
    assert!(err.starts_with("confirm first"), "{err}");
    assert!(b.expected_admissions().is_empty());
    assert!(produce_reply(&mut b, &mut OsRng, &invite, &org_node::PersonaId::new("nobody".into()), true).is_err());
    assert!(b.expected_admissions().is_empty(), "an unknown Persona declares nothing");
}

// verifies: LLR-gha5f6
#[tokio::test(flavor = "multi_thread")]
async fn a_reply_is_acted_on_once_and_only_if_its_invite_is_outstanding() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (mut a, org, mut outstanding, _) = founded("acted-on", &chain, &writer).await;
    let invite = issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Bob").unwrap();
    let (mut b, pid) = joiner_service("acted-on", &chain);
    let reply_blob = produce_reply(&mut b, &mut OsRng, &invite, &pid, true).unwrap();
    // A reply to an Invite this device never issued is refused, acting on nothing.
    let stranger = OutstandingInvites::open(dir("acted-on-stranger").join("o.json")).unwrap();
    assert!(check_reply(&stranger, &reply_blob).is_err());
    // Acted on: written, committed, sent; the id is settled.
    let sink = org_node::transport::endpoint::OrgEndpoint::bind(&DeviceSeed::from([0x44; 32]).signing_keypair()).await.unwrap();
    let addr = sink.inner().addr();
    tokio::spawn(async move { let _ = sink.recv_one().await; });
    admit_reply(&mut a, &writer, &mut outstanding, &mut OsRng, org, &reply_blob, Some(addr), None).await.unwrap();
    let id = InviteReply::parse(&reply_blob).unwrap().invite_id;
    assert!(!outstanding.holds(&id), "settled once acted on");
    assert_eq!(a.list_orgs()[0].trie_members.len(), 2);
    // The same reply again is refused: its invite is no longer outstanding.
    assert!(admit_reply(&mut a, &writer, &mut outstanding, &mut OsRng, org, &reply_blob, None, None).await.is_err());
    assert_eq!(a.list_orgs()[0].trie_members.len(), 2, "acted on once");
}

// verifies: LLR-gha5f6
#[tokio::test]
async fn a_reply_whose_admission_fails_on_chain_stays_outstanding() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (mut a, org, mut outstanding, _) = founded("write-fails", &chain, &writer).await;
    let invite = issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Bob").unwrap();
    let (mut b, pid) = joiner_service("write-fails", &chain);
    let reply_blob = produce_reply(&mut b, &mut OsRng, &invite, &pid, true).unwrap();
    writer.failing.store(true, std::sync::atomic::Ordering::SeqCst);
    assert!(admit_reply(&mut a, &writer, &mut outstanding, &mut OsRng, org, &reply_blob, None, None).await.is_err());
    assert!(outstanding.holds(&InviteReply::parse(&reply_blob).unwrap().invite_id));
    assert_eq!(a.list_orgs()[0].trie_members.len(), 1, "nothing committed");
}

// verifies: LLR-gha5f6
#[tokio::test]
async fn a_reply_for_another_organisation_is_refused() {
    let chain = MockChainOps::new();
    let writer = FakeWriter::over(&chain);
    let (mut a, org, mut outstanding, _) = founded("other-org", &chain, &writer).await;
    let invite = issue_invite(&a, &mut outstanding, &mut OsRng, org, "Acme", "Bob").unwrap();
    let (mut b, pid) = joiner_service("other-org", &chain);
    let reply_blob = produce_reply(&mut b, &mut OsRng, &invite, &pid, true).unwrap();
    let elsewhere = OrgId::new([0x42; 20]);
    assert!(admit_reply(&mut a, &writer, &mut outstanding, &mut OsRng, elsewhere, &reply_blob, None, None).await.is_err());
    assert!(outstanding.holds(&InviteReply::parse(&reply_blob).unwrap().invite_id), "nothing settled");
    assert!(a.provisional_updates(org).is_empty(), "nothing built");
}
```

`Invite::wire_for_test` and `InviteReply::wire_for_test` are
`#[cfg(feature = "test-support")]` constructors of a Blob from raw parts (so
a test can present a Blob this software never writes), as org-node's
`seal_for_test` does for stores.

`ipc.rs`: the handler list gains `export_invite, import_invite,
produce_invite_reply, import_invite_reply, admit_member` (twelve again:
LLR-4wcyqy's statement holds); re-write the tests T14 deleted against the new
arguments, with the same annotations:
- `export_invite_rejects_a_short_org_id`, `export_invite_rejects_a_non_hex_org_id`,
  `export_invite_does_not_refuse_a_well_formed_org_id` (LLR-vzf8j2): args
  `{ "orgId", "orgName": "Acme", "inviteeName": "Bob" }`; the exact parser
  messages as before; a well-formed id is refused later, at "no Persona of
  this device belongs to that Organisation", not by the parser.
- `admit_member_refuses_a_malformed_org_id_with_the_parsers_message`,
  `admit_member_does_not_refuse_a_well_formed_org_id` (LLR-vzf8j2): args
  `{ "orgId", "replyBlob": "x", "peerAddrBlob": "", "orgSecretHex": null }`.
- `admit_member_refuses_an_org_secret_that_is_not_32_bytes`,
  `admit_member_refuses_an_org_secret_that_is_not_hex` (LLR-8krgzj),
  `admit_member_does_not_refuse_a_32_byte_or_absent_org_secret` (LLR-8krgzj,
  LLR-ctrfz4): the secret checks run before the reply is parsed, as the old
  handler's did.
- `admit_member_does_not_refuse_a_blank_or_decodable_peer_addr`,
  `admit_member_refuses_a_peer_addr_that_is_not_hex`,
  `admit_member_refuses_a_peer_addr_that_is_not_an_endpoint_addr`
  (LLR-ctrfz4 as amended): the peer address is parsed as `revoke_member`
  parses it, before the reply.
- `import_invite_reply_reports_the_reply_it_parses` (LLR-pmus9f): write
  `outstanding_invites.json` holding `["0303…03"]` (64 hex digits) into the
  harness's data directory before building the harness (`harness_at`), then
  invoke `import_invite_reply` with `InviteReply::wire_for_test(&[1; 20],
  &[3; 32], …)`; assert the DTO's six fields.
- `import_invite_reply_refuses_a_malformed_blob_or_an_unknown_invite`
  (LLR-pmus9f): a malformed Blob is refused with the parse's message naming
  the field; a well-formed reply whose invite id is not outstanding is
  refused with "this reply names no Invite this device has outstanding".
- add `produce_invite_reply_refuses_without_confirmation_over_ipc`
  (LLR-w4mhd4): `{ "inviteBlob": "x", "personaId": "p", "confirmed": false }`
  → a message beginning `confirm first`.

Cargo: `[[test]] name = "invitation"`, `required-features = ["test-support"]`;
config: ` --test invitation`.

**Step 2 — run, expect red.** E0432 `ods_poc_lib::invitation`. Record.

**Step 3 — implement** `app/src-tauri/src/invitation.rs`:

```rust
//! The invitation exchange (REQ-prjja8, REQ-tcutr6, REQ-65xqp8, REQ-yazum3):
//! the Invite a Member sends and the Invite reply the invitee returns, as
//! Blobs (Base64 of postcard), parsed at this edge, naming the field that
//! fails (LLR-b7wgpf). Nothing here is verified: an Invite's sender and its
//! Organisation name are whatever the sender chose (HAZ-qfb95k, RC-wzb48r).

use std::collections::BTreeSet;
use std::path::PathBuf;

use base64::{engine::general_purpose::STANDARD, Engine};
use org_node::service::{Joiner, OrgService};
use org_node::{DevicePublicKey, Handle, InviteId, Name, OrgId, OrgSecret, PersonPublicKey, PersonaId, Surname};
use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::submit::{submit_commit_send, ChainWriter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invite {
    pub org_name: String,
    pub org_id: OrgId,
    pub invitee_name: String,
    pub inviter_device_keys: Vec<DevicePublicKey>,
    pub invite_id: InviteId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InviteReply {
    pub org_id: OrgId,
    pub invite_id: InviteId,
    pub member_key: PersonPublicKey,
    pub device_key: DevicePublicKey,
    pub handle: Handle,
    pub name: Name,
    pub surname: Surname,
}

#[derive(Serialize, Deserialize)]
struct WireInvite {
    org_name: String,
    org_id: Vec<u8>,
    invitee_name: String,
    inviter_device_keys: Vec<Vec<u8>>,
    invite_id: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct WireReply {
    org_id: Vec<u8>,
    invite_id: Vec<u8>,
    member_key: Vec<u8>,
    device_key: Vec<u8>,
    handle: String,
    name: String,
    surname: String,
}

fn exact<const N: usize>(field: &str, bytes: &[u8]) -> Result<[u8; N], String> {
    bytes.try_into().map_err(|_| format!("{field}: expected {N} bytes, got {}", bytes.len()))
}

fn named<T, E: std::fmt::Display>(field: &str, r: Result<T, E>) -> Result<T, String> {
    r.map_err(|e| format!("{field}: {e}"))
}

fn unarmour<T: for<'de> Deserialize<'de>>(what: &str, blob: &str) -> Result<T, String> {
    let bytes = STANDARD.decode(blob.trim()).map_err(|e| format!("{what}: not a Blob: {e}"))?;
    postcard::from_bytes(&bytes).map_err(|e| format!("{what}: does not decode: {e}"))
}

fn armour<T: Serialize>(value: &T) -> Result<String, String> {
    postcard::to_allocvec(value).map(|b| STANDARD.encode(b)).map_err(|e| format!("encode: {e}"))
}

impl Invite {
    /// Parse an Invite Blob, naming the field that fails (LLR-b7wgpf).
    pub fn parse(blob: &str) -> Result<Self, String> {
        let w: WireInvite = unarmour("invite", blob)?;
        Ok(Self {
            org_name: w.org_name,
            org_id: OrgId::new(exact("invite.org_id", &w.org_id)?),
            invitee_name: w.invitee_name,
            inviter_device_keys: w
                .inviter_device_keys
                .iter()
                .map(|k| named("invite.inviter_device_keys", DevicePublicKey::parse(&exact("invite.inviter_device_keys", k)?)))
                .collect::<Result<_, _>>()?,
            invite_id: InviteId::new(exact("invite.invite_id", &w.invite_id)?),
        })
    }

    pub fn encode(&self) -> Result<String, String> {
        armour(&WireInvite {
            org_name: self.org_name.clone(),
            org_id: self.org_id.as_bytes().to_vec(),
            invitee_name: self.invitee_name.clone(),
            inviter_device_keys: self.inviter_device_keys.iter().map(|k| k.as_bytes().to_vec()).collect(),
            invite_id: self.invite_id.as_bytes().to_vec(),
        })
    }

    #[cfg(feature = "test-support")]
    pub fn wire_for_test(org_name: &str, org_id: &[u8], invitee: &str, keys: &[[u8; 32]], invite_id: &[u8]) -> String {
        armour(&WireInvite {
            org_name: org_name.into(),
            org_id: org_id.to_vec(),
            invitee_name: invitee.into(),
            inviter_device_keys: keys.iter().map(|k| k.to_vec()).collect(),
            invite_id: invite_id.to_vec(),
        })
        .unwrap_or_default()
    }
}
```

`InviteReply` follows the same shape: `parse` names `reply.org_id`,
`reply.invite_id`, `reply.member_key` (via `PersonPublicKey::parse`),
`reply.device_key`, `reply.handle` (`Handle::parse`), `reply.name`,
`reply.surname`, in that order; `encode`; `wire_for_test(org_id, invite_id,
member_key, device_key, handle, name, surname)`.

```rust
/// Invite identifiers this device issued and has not yet seen acted on,
/// kept in a file (LLR-f35pda, Decision 13).
pub struct OutstandingInvites {
    path: PathBuf,
    ids: BTreeSet<InviteId>,
}

impl OutstandingInvites {
    pub fn open(path: PathBuf) -> Result<Self, String> {
        let ids = match std::fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str::<Vec<String>>(&text)
                .map_err(|e| format!("outstanding invites {}: {e}", path.display()))?
                .iter()
                .map(|h| {
                    hex::decode(h)
                        .map_err(|e| format!("outstanding invites {}: {e}", path.display()))
                        .and_then(|b| exact("outstanding invite", &b))
                        .map(InviteId::new)
                })
                .collect::<Result<_, _>>()?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => BTreeSet::new(),
            Err(e) => return Err(format!("outstanding invites {}: {e}", path.display())),
        };
        Ok(Self { path, ids })
    }
    fn save(&self) -> Result<(), String> {
        let hexes: Vec<String> = self.ids.iter().map(|id| hex::encode(id.as_bytes())).collect();
        let text = serde_json::to_string(&hexes).map_err(|e| e.to_string())?;
        std::fs::write(&self.path, text).map_err(|e| format!("outstanding invites {}: {e}", self.path.display()))
    }
    pub fn issue<R: RngCore + CryptoRng>(&mut self, rng: &mut R) -> Result<InviteId, String> {
        let mut id = [0u8; 32];
        rng.fill_bytes(&mut id);
        self.ids.insert(InviteId::new(id));
        self.save()?;
        Ok(InviteId::new(id))
    }
    pub fn holds(&self, id: &InviteId) -> bool {
        self.ids.contains(id)
    }
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
    pub fn settle(&mut self, id: &InviteId) -> Result<(), String> {
        self.ids.remove(id);
        self.save()
    }
}

/// LLR-9sraks: an Invite for `org_id`, carrying the inviter's DevicePublicKeys
/// (every Persona of this device bound to it) and a fresh outstanding id.
pub fn issue_invite<R: RngCore + CryptoRng>(
    svc: &OrgService,
    outstanding: &mut OutstandingInvites,
    rng: &mut R,
    org_id: OrgId,
    org_name: &str,
    invitee_name: &str,
) -> Result<String, String> {
    let keys: Vec<DevicePublicKey> = svc
        .list_personas()
        .iter()
        .filter(|p| p.org_id == Some(org_id))
        .map(|p| svc.persona_public_keys(&p.persona_id).map(|(_, d)| d).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    if keys.is_empty() {
        return Err("no Persona of this device belongs to that Organisation".into());
    }
    let invite_id = outstanding.issue(rng)?;
    Invite { org_name: org_name.into(), org_id, invitee_name: invitee_name.into(), inviter_device_keys: keys, invite_id }.encode()
}

/// LLR-w4mhd4: the reply to `invite_blob` from `persona_id`, only once the
/// user confirmed; declares the expected admission under the invite id.
pub fn produce_reply<R: RngCore + CryptoRng>(
    svc: &mut OrgService,
    rng: &mut R,
    invite_blob: &str,
    persona_id: &PersonaId,
    confirmed: bool,
) -> Result<String, String> {
    if !confirmed {
        return Err("confirm first: nothing has verified who sent this Invite or the name it states, and the reply reveals the chosen Persona's handle, name and surname".into());
    }
    let invite = Invite::parse(invite_blob)?;
    let p = svc
        .list_personas()
        .iter()
        .find(|p| &p.persona_id == persona_id)
        .cloned()
        .ok_or("no such Persona")?;
    let (member_key, device_key) = svc.persona_public_keys(persona_id).map_err(|e| e.to_string())?;
    let reply = InviteReply {
        org_id: invite.org_id,
        invite_id: invite.invite_id,
        member_key,
        device_key,
        handle: p.handle,
        name: p.name,
        surname: p.surname,
    }
    .encode()?;
    svc.expect_admission(rng, invite.org_id, invite.invite_id).map_err(|e| e.to_string())?;
    Ok(reply)
}

/// LLR-gha5f6: a reply naming an outstanding Invite, else refused.
pub fn check_reply(outstanding: &OutstandingInvites, reply_blob: &str) -> Result<InviteReply, String> {
    let reply = InviteReply::parse(reply_blob)?;
    if !outstanding.holds(&reply.invite_id) {
        return Err("this reply names no Invite this device has outstanding".into());
    }
    Ok(reply)
}

/// LLR-gha5f6, LLR-qhjp6g: admit the person a reply names, through the
/// chain, the admission carrying the reply's invite id; the Invite is settled
/// once the admission has committed.
#[allow(clippy::too_many_arguments)]
pub async fn admit_reply<R: RngCore + CryptoRng + Send>(
    svc: &mut OrgService,
    writer: &dyn ChainWriter,
    outstanding: &mut OutstandingInvites,
    rng: &mut R,
    org_id: OrgId,
    reply_blob: &str,
    peer_addr: Option<iroh::EndpointAddr>,
    org_secret: Option<OrgSecret>,
) -> Result<org_node::MemberId, String> {
    let reply = check_reply(outstanding, reply_blob)?;
    if reply.org_id != org_id {
        return Err("this reply is for another Organisation".into());
    }
    let joiner = Joiner {
        handle: reply.handle.clone(),
        name: reply.name.clone(),
        surname: reply.surname.clone(),
        member_key: reply.member_key,
        device_key: reply.device_key,
    };
    let update = svc.admit_member(rng, org_id, &joiner).map_err(|e| e.to_string())?;
    let sent =
        submit_commit_send(svc, writer, rng, &update, reply.device_key, peer_addr, org_secret, Some(reply.invite_id)).await;
    let admitted = svc
        .list_orgs()
        .iter()
        .find(|o| o.org_id == org_id)
        .and_then(|o| o.trie_members.iter().find(|m| m.member_key == reply.member_key).map(|m| m.id));
    if let Some(id) = admitted {
        outstanding.settle(&reply.invite_id)?;
        sent?;
        return Ok(id);
    }
    sent.and(Err("the admission did not commit".into()))
}
```

`state.rs`: `AppState` gains `pub outstanding: tokio::sync::Mutex<OutstandingInvites>`,
opened in `assemble` at `data_dir.join("outstanding_invites.json")` (a file
that does not open refuses assembly with its message). `commands.rs` adds thin
handlers:

- `export_invite(org_id, org_name, invitee_name) -> String`: `parse_org_id`,
  then `issue_invite` under both locks.
- `import_invite(blob) -> InviteDto { org_name, org_id, invitee_name,
  inviter_device_keys: Vec<String>, invite_id }` (hex strings).
- `produce_invite_reply(invite_blob, persona_id, confirmed: bool) -> String`.
- `import_invite_reply(blob) -> InviteReplyDto { org_id, handle, name,
  surname, member_key, device_key }` via `check_reply` (LLR-pmus9f).
- `admit_member(org_id, reply_blob, peer_addr_blob, org_secret_hex) ->
  String`: `parse_org_id`; the 32-byte `org_secret_hex` check exactly as the
  old handler had it; `peer_addr_blob` as `revoke_member` parses it
  (LLR-ctrfz4); then `admit_reply`; returns the member id hex.

`lib.rs`: `pub mod invitation;` and the five handlers in the list.

**Step 4 — green.** The app's cargo line (with `--test invitation`):
invitation 12 passed; ipc back to its T13 count plus the
`produce_invite_reply` test; all `ok`. `npm --prefix app run check` and `run
test` pass. `git diff app/src-tauri/Cargo.lock` shows only the `ods-poc`
entry gaining `base64`. Commit: `T15: app invitation exchange backend`.

**Red→green attestations:** (DONE 2026-10-06, merged. App cargo line with
`--test invitation`: 154 passed, 0 failed — startup_policy 13,
connection_status 10, org_id_parsing 14, receiver_events 32, receiver_guard 6,
ipc 35, csp_policy 24, state_assembly 3, submit_flow 5, invitation 12. `npm run
check`: 0 errors; `npm run test`: 36 passed. app/src-tauri/Cargo.lock: the
`ods-poc` entry gains `base64 0.22.1` (already resolved); format kept at 3,
`cargo metadata --locked` passes. App LLR with no test after T15:
LLR-n2u4uf only (T16).) Every test was compile-red first (E0432
`ods_poc_lib::invitation`; E0433 for the five missing handlers), then run
against stub bodies:

- invitation.rs, runtime-red against the stubs then green: an_invite_carries_what_the_inviter_typed_and_a_fresh_outstanding_id; outstanding_invite_ids_survive_a_reopen_and_settle_one_at_a_time; an_outstanding_invites_file_that_is_not_a_hex_list_is_refused; an_invite_and_a_reply_that_parse_encode_back_to_the_same_blob; an_invite_or_reply_that_does_not_parse_is_refused_naming_the_field; a_reply_that_does_not_parse_acts_on_nothing; a_confirmed_reply_carries_the_persona_and_declares_the_expected_admission; no_reply_is_produced_and_nothing_declared_without_confirmation; a_reply_whose_admission_fails_on_chain_stays_outstanding; a_reply_for_another_organisation_is_refused
- a_reply_is_acted_on_once_and_only_if_its_invite_is_outstanding — runtime-red against the stubs; first run against the implementation also red from a test fault (sink endpoint dropped too early: "connection lost"), fixed in the test, which now also asserts the sent message carries the invite id; green
- no_invite_is_issued_for_an_organisation_no_persona_here_belongs_to — passed against the stubs; runtime-red under a mutant dropping the `keys.is_empty()` refusal; green
- ipc.rs rewrites, runtime-red against the stub handler then green: export_invite_rejects_a_short_org_id; export_invite_rejects_a_non_hex_org_id; export_invite_does_not_refuse_a_well_formed_org_id (also killed by the no-empty-keys mutant); admit_member_refuses_a_malformed_org_id_with_the_parsers_message; admit_member_refuses_an_org_secret_that_is_not_32_bytes; admit_member_refuses_an_org_secret_that_is_not_hex; admit_member_refuses_a_peer_addr_that_is_not_hex (new); admit_member_refuses_a_peer_addr_that_is_not_an_endpoint_addr; import_invite_reply_reports_the_reply_it_parses; import_invite_reply_refuses_a_malformed_blob_or_an_unknown_invite; produce_invite_reply_refuses_without_confirmation_over_ipc (new)
- admit_member_does_not_refuse_a_well_formed_org_id, admit_member_does_not_refuse_a_32_byte_or_absent_org_secret, admit_member_does_not_refuse_a_blank_or_decodable_peer_addr — passed against the stubs (stub messages coincided); runtime-red under a mutant returning `Err("stub")` after the org-id parse with the real `InviteReply::parse`; green
- product_registers_the_same_commands_as_the_harness (7 → 12) — passed with the stubs; runtime-red under a mutant dropping `commands::admit_member` from lib.rs; green

Left for T17: LLR-b7wgpf names `P2pDeviceKey::parse`/`P2pMemberKey::parse`;
the code uses person's `DevicePublicKey::parse`/`PersonPublicKey::parse`.

---

### T16 — app: the invitation flow in the UI (REQ-ab2mfz)

**Files touched:** `app/src/lib/api.ts`, `app/src/lib/invite.ts` (new),
`app/tests/invite.confirm.test.ts` (new),
`app/src/lib/components/Invite.svelte`, `app/src/lib/components/Admit.svelte`
**Parallel:** no (serial, after T15)
**IDs verified:** REQ-ab2mfz (RC-wzb48r), LLR-n2u4uf.
**Size:** ~300 lines.

**Step 1 — failing test.** `app/tests/invite.confirm.test.ts`:

```ts
/**
 * REQ-ab2mfz (RC-wzb48r), LLR-n2u4uf: before an Invite reply is produced the
 * user is told that nothing verified the Invite's sender or the Organisation
 * name it states, and that the reply reveals the chosen Persona's handle, name
 * and surname to its sender; the reply is produced only once the user
 * confirms. Tested on the extracted decision, never a component (as
 * revoke.validate).
 */
import { describe, it, expect } from 'vitest';
import { REPLY_WARNING, replyGate } from '../src/lib/invite';

describe('the Invite reply warning and gate', () => {
	// verifies: LLR-n2u4uf
	it('states each thing the user must be told', () => {
		expect(REPLY_WARNING).toMatch(/nothing has verified who sent this invite/i);
		expect(REPLY_WARNING).toMatch(/organisation name/i);
		expect(REPLY_WARNING).toMatch(/handle, name and surname/i);
	});

	// verifies: LLR-n2u4uf
	it('refuses until the user has confirmed', () => {
		expect(replyGate({ personaId: 'p-1', confirmed: false })).toEqual({
			ok: false,
			message: 'Read the warning and confirm before replying.'
		});
	});

	// verifies: LLR-n2u4uf
	it('refuses without a chosen Persona even when confirmed', () => {
		expect(replyGate({ personaId: '  ', confirmed: true }).ok).toBe(false);
	});

	// verifies: LLR-n2u4uf
	it('allows the reply once a Persona is chosen and the user confirmed', () => {
		expect(replyGate({ personaId: 'p-1', confirmed: true })).toEqual({ ok: true });
	});
});
```

Run `npm --prefix app run test` → fails: cannot resolve `../src/lib/invite`.
Record.

**Step 2 — implement.** `app/src/lib/invite.ts`:

```ts
/**
 * REQ-ab2mfz / RC-wzb48r (LLR-n2u4uf): what the user is told before an
 * Invite reply is produced, and the decision that gates producing it.
 */
export const REPLY_WARNING =
	'Nothing has verified who sent this Invite, or the Organisation name it states. ' +
	'Your reply will reveal the handle, name and surname of the Persona you choose to whoever sent it.';

export type ReplyCheck = { ok: true } | { ok: false; message: string };

export function replyGate(input: { personaId: string; confirmed: boolean }): ReplyCheck {
	if (input.personaId.trim() === '') {
		return { ok: false, message: 'Choose the Persona to reply as.' };
	}
	if (!input.confirmed) {
		return { ok: false, message: 'Read the warning and confirm before replying.' };
	}
	return { ok: true };
}
```

`app/src/lib/api.ts`: delete `JoinRequestDto`, `exportJoinRequest`,
`importJoinRequest`; replace `exportInvite`, `importInvite`, `admitMember`
and add the two new commands:

```ts
export interface InviteDto {
	org_name: string;
	org_id: string;
	invitee_name: string;
	inviter_device_keys: string[];
	invite_id: string;
}

export interface InviteReplyDto {
	org_id: string;
	handle: string;
	name: string;
	surname: string;
	member_key: string;
	device_key: string;
}

/** REQ-prjja8: an Invite Blob for an Organisation this device belongs to. */
export function exportInvite(orgId: string, orgName: string, inviteeName: string): Promise<string> {
	return invoke<string>('export_invite', { orgId, orgName, inviteeName });
}

/** REQ-yazum3: parse an Invite Blob; a field that does not parse is named. */
export function importInvite(blob: string): Promise<InviteDto> {
	return invoke<InviteDto>('import_invite', { blob });
}

/** REQ-tcutr6 / REQ-ab2mfz: the reply, only with `confirmed` true. */
export function produceInviteReply(inviteBlob: string, personaId: string, confirmed: boolean): Promise<string> {
	return invoke<string>('produce_invite_reply', { inviteBlob, personaId, confirmed });
}

/** REQ-65xqp8: parse a reply; refused unless it names an outstanding Invite. */
export function importInviteReply(blob: string): Promise<InviteReplyDto> {
	return invoke<InviteReplyDto>('import_invite_reply', { blob });
}

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

and the header comment's "all 12 ODS commands" → "the ODS commands".

`Invite.svelte`: side A — organisation select (as today), two text inputs
"Organisation name (as you want the invitee to see it)" and "Invitee's full
name (your guess)", button calling `exportInvite(orgId, orgName,
inviteeName)`, the Blob shown with a copy button. Side B — paste an Invite →
`importInvite` → show `org_name` with the label "stated by the sender,
unverified", the org id and the invitee name; a Persona select over
`listPersonas()` (with the existing create-Persona form beside it); a
warning box rendering `REPLY_WARNING` and a checkbox "I understand" bound to
`confirmed`; the "Produce reply" button is disabled unless
`replyGate({ personaId, confirmed }).ok`, and calls
`produceInviteReply(blob, personaId, confirmed)`; the reply Blob is shown
with a copy button. Delete the join-request code.

`Admit.svelte`: paste an Invite reply → `importInviteReply` → review the
parsed fields; a peer-address input labelled "required in Loopback only"
(reuse `validateRevokeInput`'s rule: required when the transport mode is
`loopback`); the optional Organisation secret input as today; "Admit" calls
`admitMember(orgId, replyBlob, peerAddrBlob, secretOrNull)` and shows the
returned member id or the error (a chain-write failure or timeout is reported
as the backend phrased it). Delete the join-request code.

**Step 3 — green.** `npm --prefix app run test` (invite.confirm 4 passed, the
other files unchanged) and `npm --prefix app run check` (0 errors). The
app's cargo line still passes. Commit:
`T16: app invitation flow with the reply warning`.

**Red→green attestations:** (DONE 2026-10-06, merged. App cargo line: 154
passed, 0 failed; `npm run check`: 0 errors; `npm run test`: 5 files, 40
passed. app/src-tauri/Cargo.lock unchanged.) The suite first failed with
"Cannot find module '../src/lib/invite'" (0 tests ran); then, against a stub
module (`REPLY_WARNING = ''`, `replyGate` ok only for a blank Persona):

- states each thing the user must be told — runtime-red ("expected '' to match /nothing has verified who sent this i…/i"); green
- refuses until the user has confirmed — runtime-red (`{ok:false, message:'stub'}` ≠ the expected message); green
- refuses without a chosen Persona even when confirmed — runtime-red ("expected true to be false"); green
- allows the reply once a Persona is chosen and the user confirmed — runtime-red (`{ok:false, message:'stub'}` ≠ `{ok:true}`); green

A second mutant (dropping the confirmation check) was refused by the
sandbox's classifier as security-weakening and not pursued; the stub run is
the runtime red for all four. The warning follows the owner's narrowing
(handle, name, surname; nothing about keys). The UI components were checked by
svelte-check only (no component test, by the plan's design).

**Dispatcher ruling for T17:** T16 added an optional Organisation-secret hex
input to Admit.svelte, reading the plan's "as today" as an input that never
existed (the old component always sent `null`). T17 removes that input and
keeps sending `null`: no requirement asks for typing a secret into the UI, and
the key-pair change replaces how the secret travels.

---

### T17 — docs, deslop and the full gate

**Files touched:** `on-chain-client/docs/architecture/soup.md`,
`org-node/docs/architecture/soup.md`,
`org-node/docs/architecture/2026-10-03-decomposition.md`,
`org-node/docs/architecture/2026-10-05-unsigned-envelope.md`,
`app/docs/architecture/soup.md`, `org-node/.guardrails/config.yaml`,
`on-chain-client/.guardrails/config.yaml`, `app/.guardrails/config.yaml`,
`org-node/src/service.rs`, `org-node/src/lib.rs`, `org-node/src/store.rs`,
`org-node/src/verify.rs`, `org-node/src/transport/endpoint.rs`
**Parallel:** no (serial, after T16)
**IDs verified:** none new; this task makes the whole change verifiable.
**Size:** ~170 lines.

**Step 1 — deslop org-node's source.** Run each and fix every hit that
describes removed behaviour (an Invite, a Join request, an administrator, a
chain write, an old line number), rewording rather than deleting where a
comment still says something true:

```
grep -rn -i "invite\|join request\|join_request" org-node/src
grep -rn -i "admin" org-node/src
grep -rn -i "submit\|ceremony\|multisig\|extrinsic" org-node/src
grep -rn -i "signature\|signed\|signs " org-node/src
```

Expected after the fix: the only `invite` hits are `InviteId`,
`invite_id` and `ExpectedAdmission`'s docs; no `admin` hit except iroh's or
the transport's own vocabulary; `preflight.rs`'s chain-health words stay.
Each kept hit is listed in the commit message. Rerun `--test absences` after
editing (it scans the same text).

**Step 2 — docs.**
- `on-chain-client/docs/architecture/soup.md`, "Behind the `write`
  feature" → "Versions": replace "The two rows below give the versions in the
  repository-root `Cargo.lock`, which governs the build in which the writer
  ships — org-node and the app are root-workspace members and build this
  crate as a path dependency." by "The two rows give the versions in
  `app/src-tauri/Cargo.lock`, which governs the build the writer ships in:
  the app is its own workspace (its `Cargo.toml` opens with `[workspace]`),
  and only the app enables `write`; org-node, a root-workspace member, no
  longer uses the writer." Keep the 0.50.1 sentence and replace "which one
  the writer's evidence is taken from is to be settled when the feature
  lands" by "the writer's gated evidence is taken from this unit's lock
  (0.50.1), the shipped build from the app's (0.50.3); aligning them is an
  owner decision (docs/plans/2026-10-05-chain-authority.md, Open questions)".
  Re-measure the closure with `write` on:
  `cargo tree --manifest-path on-chain-client/Cargo.toml --features write --edges normal --prefix none --no-dedupe | sort -u | wc -l`
  and add the figure beside the five-dependency one, dated.
- `org-node/docs/architecture/soup.md`: re-measure the `normal` and
  `normal,build,dev` closure rows with the command the file states, and add a
  dated note under them; the rows for `base64`, `subxt-signer`,
  `parity-scale-codec`, `blake2` already say they leave — amend each
  "Leaves" note to "Left … (T7/T13)".
- `org-node/docs/architecture/2026-10-03-decomposition.md`, SDD-z85ux9's
  section "The item with no low-level requirements": add a dated
  re-measurement paragraph (same rule as the 2026-10-05 one: whole files plus
  the named parts of `service.rs`) and note that `chain_genesis_e2e` and
  `finality_polling` are deleted (`preflight` is the one chopsticks target
  left). Its "Robustness" section and
  `2026-10-05-unsigned-envelope.md`'s robustness table: rename every test
  this plan renamed, and replace every test it deleted by the replacement the
  tables below name.
- `app/docs/architecture/soup.md`: add `base64` 0.22.x (version from
  `app/src-tauri/Cargo.lock`) — role: the Invite and Invite-reply armour
  (`src-tauri/src/invitation.rs`), REQ-yazum3, decode failure is a named
  refusal; and amend the on-chain-client row (or add one) to say the app
  enables its `write` feature (REQ-nfr3n2) and holds the signatory key it
  passes to it.
- `org-node/docs/problems/2026-09-09-org-node-problems.md`: check that
  PR-u4c2vp's resolution names a test that still exists
  (`pr_u4c2vp_an_update_relayed_by_a_non_member_is_committed_on_the_self_delete_path`);
  change nothing else.

**Step 3 — config comments.** Each of the three configs gets one dated
comment block naming what this change did to its `verify_commands` and the
measured pass counts of the final run (Step 4), as earlier changes recorded
theirs (org-node's block above `verify_commands`).

**Step 4 — the full gate.** Run every command of this plan's Verification
header, from a clean `target` for the three crates
(`QUINT_HOME` per Environment), and the gates:

```
GR_CONFIG=org-node/.guardrails/config.yaml .guardrails/scripts/check-trace.sh
GR_CONFIG=on-chain-client/.guardrails/config.yaml .guardrails/scripts/check-trace.sh
GR_CONFIG=app/.guardrails/config.yaml .guardrails/scripts/check-trace.sh
GR_CONFIG=org-node/.guardrails/config.yaml .guardrails/scripts/check-ids.sh --allow-draft-files
GR_CONFIG=on-chain-client/.guardrails/config.yaml .guardrails/scripts/check-ids.sh --allow-draft-files
GR_CONFIG=app/.guardrails/config.yaml .guardrails/scripts/check-ids.sh --allow-draft-files
.guardrails/scripts/check-units.sh
make coverage-on-chain-client
```

Expected: every cargo line `test result: ok` with `0 failed`; quint lines
pass; `check-trace` reports no MISSING-TEST, DANGLING-REF, UNIMPLEMENTED-
CONTROL or UNTRACED-DESIGN (UNRESOLVED-PR warnings list the open reports,
which no longer include PR-b9wab3 or PR-322qst; UNMET-EXPECTATION lines are
the other units' standing ones); `check-ids` and `check-units` exit 0;
coverage at or above its floors. Paste the outputs into the commit message's
body (counts per target). Clippy for the three crates with `-D warnings` is
clean. Commit: `T17: docs, deslop and the full gate for chain authority`.

**Red→green attestations:** (no new test) DONE 2026-10-06 for the docs,
source-comment and leftover steps (merged); the whole-change deslop pass and
the gate are separate dispatches, and Step 3's config comments wait for the
final run. Suites at the end of T17: org-node 223 passed (23 targets) and quint
clean; on-chain-client line 1 73 passed (lib + 9 targets; the merge run's "78"
was a miscount), line 2 23 passed; app 154 passed; `npm run check` 0 errors,
`npm run test` 40 passed. check-trace: org-node, on-chain-client and app
report no MISSING-TEST — only UNRESOLVED-PR and UNMET-EXPECTATION.
check-ids and check-units clean. Leftovers fixed: ledger test citations,
service_stories comment, unused `hex` dev-dependency removed (root lock
byte-identical), LLR-b7wgpf names person's parse functions, api.ts comments,
Admit.svelte's Organisation-secret input removed (sends `null` again).

---

## Implements → tasks

Every ID of **Implements** and the task whose test verifies it. RCs and SDDs
carry no `verifies:` of their own: an RC is implemented by the REQ named
beside it, an SDD is evidenced through its LLRs.

| ID | Task(s) | Evidence (target) |
|---|---|---|
| REQ-xs4ab8 | T8, T9, T10, T11, T13 | provisional_store, commit_paths, absences |
| REQ-uv3v5w | T9, T10 | commit_paths |
| REQ-fwfku9 | T8, T10 | provisional_store, value_types, commit_paths |
| REQ-tqap3r | T9, T10, T11 | commit_paths |
| REQ-f2k4tr (RC-mj6gjq) | T5 | verify_against_chain, receive_chain_reads |
| REQ-8amu2a (RC-2ferct) | T6, T10 | expected_admission, wire_frame_bound, value_types, commit_paths |
| REQ-kt877x | T6 | expected_admission, value_types |
| REQ-nhe2zu, REQ-txvtm9 | T7, T10, T11 (rewritten tests) | admission_sender, commit_paths |
| REQ-xa6smf | T6, T7 | admission_sender |
| REQ-ech45n | T8, T9 | provisional_store, commit_paths, organisation_key |
| REQ-d9g6nt | T9 (+ unchanged `fuzz_first_admission_base`, T6 `an_expected_first_admission_decodes…`) | commit_paths, admission_sender |
| REQ-qn2erx | T7 | absences, admission_sender |
| REQ-uxv2x2 | T12 | admission_sender, commit_paths |
| LLR-mxskg9 | T6, T8, T9 | value_types, expected_admission, provisional_store, commit_paths |
| LLR-fuq379 | T5 | verify_against_chain |
| LLR-ms8njy | T6 | wire_frame_bound, encoding_golden, expected_admission |
| LLR-95753m | T6, T8 | expected_admission, provisional_store |
| LLR-jq7qh7 | T8, T10 | provisional_store, commit_paths |
| LLR-nvn3wk | T9, T10 | commit_paths |
| LLR-qjz3q4 | T8, T9 | provisional_store, commit_paths, organisation_key |
| LLR-s6qnht, LLR-wzqqg9 | T9 | commit_paths |
| LLR-2xzys9, LLR-48jakr | T10 | commit_paths |
| LLR-9ew26y | T5 | receive_chain_reads |
| LLR-s8xp7m | T6 | expected_admission |
| LLR-3f5h7b | T6 | expected_admission |
| LLR-cmdrp9, LLR-ewkg85, LLR-4tcxsu, LLR-mkj4bz | T9, T10 | commit_paths |
| LLR-b27jr6 | T12 | admission_sender, commit_paths |
| LLR-3wb7th, LLR-379hnv | T5 | receive_chain_reads, admission_sender |
| LLR-9zfnmb, LLR-q8emds, LLR-mbjfq8, LLR-y2v8v2, LLR-j83kc8 | T6 | expected_admission, admission_sender |
| LLR-g9vmbx, LLR-zj88e6, LLR-qezw3n, LLR-836z24, LLR-8qxwst | T7 | absences |
| LLR-437fvx, LLR-kkj64b, LLR-ctzkv7 | T7 | admission_sender |
| LLR-tcft2r | T7 | persona_records |
| LLR-xq9nrq, LLR-rys5nx, LLR-e5c9ud | T8 | absences, admission_sender |
| LLR-g76zqd, LLR-8bum44 | T7, T8 | persona_records, provisional_store, absences |
| LLR-sj7cd5, LLR-3fwykc, LLR-2dvhz8 | T9 | organisation_key, commit_paths |
| LLR-68yd3j | T9 | commit_paths, service_lifecycle |
| LLR-rjg3m2, LLR-w3fhhg, LLR-q3aj8z, LLR-dzte8x | T9 | commit_paths, admission_sender |
| LLR-3v5nu9 | T9, T11 | commit_paths, absences |
| LLR-ryzr8m | T9, T11 | service_lifecycle |
| LLR-rb8r65, LLR-ghja3x, LLR-bg3vsw, LLR-jn5jeh, LLR-t4znbk, LLR-vdyu65, LLR-8hdu9x, LLR-ckk5nz | T10 | commit_paths, admission_sender |
| LLR-pw369n | T10, T11 | commit_paths, admission_sender |
| LLR-cja9zv | T10 (re-annotated), existing | admission_sender, commit_paths |
| LLR-6dc598, LLR-tax3pm, LLR-qg9utu, LLR-drgdy8 | T11 | commit_paths, admission_sender |
| LLR-65py3d | T11 | service_lifecycle, absences |
| LLR-6p4pj2, LLR-jsx922 | T12 (and existing) | commit_paths, admission_sender |
| LLR-rv4vux, LLR-rc74nq, LLR-txqmz4, LLR-66h529, LLR-463d89, LLR-8m3bwj, LLR-f74xwb | T13 | absences |
| LLR-s7whrn | T13 (+ unchanged node_value_types, chain_read_state) | absences |
| LLR-ayrdr8 | T6, T7, T8 | encoding_golden |
| REQ-6jefu2, REQ-aat4yt (SDD-yg7n55) | T1, T2 (done) | write_manifest, write_pure, write_compose |
| LLR-rxs5ec | T1 (done) | write_manifest |
| LLR-yvq33e, LLR-5varjf, LLR-d4ftc8, LLR-m7wmmx, LLR-ywhd23, LLR-gjx3jn, LLR-kqxw9t, LLR-qs8jfw | T1 (done) | write_pure |
| LLR-a2acvh, LLR-z9vugt, LLR-kv27gp, LLR-hun4wf | T2 (done) | write_compose |
| REQ-nfr3n2: LLR-qhjp6g, LLR-be3zv9 | T14 | submit_flow |
| REQ-nfr3n2, REQ-65xqp8: LLR-gha5f6 | T15 | invitation |
| REQ-prjja8: LLR-9sraks, LLR-f35pda | T15 | invitation |
| REQ-tcutr6: LLR-w4mhd4 | T15 | invitation, ipc |
| REQ-yazum3: LLR-b7wgpf | T15 | invitation |
| REQ-ab2mfz (RC-wzb48r): LLR-w4mhd4, LLR-n2u4uf | T15, T16 | invitation, ipc, invite.confirm.test.ts |
| LLR-7bk6qh | T14 | receiver_events |
| LLR-vzf8j2, LLR-8krgzj, LLR-ctrfz4, LLR-pmus9f, LLR-4wcyqy | T14, T15 | ipc |
| PR-b9wab3 / PR-322qst | T11 / T12 | admission_sender |
| PR-u4c2vp | master (switch-trim), checked in T17 | admission_sender |

No ID of **Implements** is without a task. The SDD items amended in place
are realised by the tasks that change the code they own: SDD-na9nc3 (T5);
SDD-kwncn7, SDD-swtd3w (T6, T8, T9); SDD-af5vnt (T6–T9); SDD-vee2fq (T7);
SDD-89es4z (T9); SDD-rx2yvy, SDD-8cpyfa (T6, T10, T12); SDD-72ddm6 (T11,
T12); SDD-ueh4tm (T9–T11); SDD-msb6xh, SDD-rq6nv4, SDD-z85ux9 (T13);
SDD-yg7n55 (T1, T2); the app's SDD-2pa6h6, SDD-rmbr3t (T14, T15),
SDD-jx363y (T16), SDD-6g3wnh (T14–T16, untested by design).

## Tests deleted, and what verifies their IDs instead

| Task | Deleted test(s) | ID(s) | Replacing evidence |
|---|---|---|---|
| T7 | `blob_exchange` (5 tests) | REQ-9g6as6, LLR-g9vmbx, LLR-kkj64b, LLR-8qxwst, LLR-tcft2r | `absences::org_node_holds_no_invitation_exchange`, `admission_sender::an_admitted_member_carries_exactly_the_joiners_values`, `persona_records::bytes_that_are_not_a_record_snapshot_are_refused` |
| T7 | `encoding_golden::invite_and_join_request_text_is_pinned` | LLR-ayrdr8 | the store, wire and truncation golden tests |
| T7 | `persona_records`: three Join-request and two Invite import tests | LLR-8bum44 | the store-open and record-snapshot tests in the same file, and T8's `a_store_with_an_invalid_provisional_update_is_refused_naming_the_field` |
| T7 | `admission_sender::an_imported_invite_reaches_the_joiners_disk` | LLR-9zfnmb | `expected_admission::expect_admission_records_a_pair_once_and_reaches_the_disk` |
| T7 | `admission_sender::pr_8qsnhx_a_join_request_advertises…`, `…an_invite_advertises…` | LLR-437fvx, LLR-qezw3n | `persona_public_keys_returns_the_personas_two_keys`, `org_node_holds_no_invitation_exchange` |
| T7 | `admission_sender::a_first_admission_whose_invite_names_another_organisation_key_is_committed` | REQ-xa6smf, LLR-j83kc8, LLR-rys5nx | `a_first_admission_relayed_by_another_device_is_committed` (REQ-xa6smf, LLR-j83kc8); T8's `a_first_admission_records_the_chains_organisation_public_key` and `org_node_has_no_administrator_key` (LLR-rys5nx) |
| T13 | `chain_write_pure` | LLR-rv4vux, LLR-txqmz4, LLR-66h529, LLR-463d89, LLR-8m3bwj, LLR-f74xwb, LLR-rc74nq | `absences::org_node_writes_nothing_to_the_chain`; the behaviour's evidence is on-chain-client's `write_pure` |
| T13 | `calldata_typed`, `encoding_golden::genesis_and_update_calldata_is_pinned` | LLR-ayrdr8 | the remaining golden tests; calldata pinned by `write_pure::update_calldata_is_the_pinned_hundred_bytes` |
| T13 | `chain_genesis_e2e`, `finality_polling` | none (chopsticks, no annotation) | `on-chain-client/tests/write_genesis_e2e.rs` (T2); the polling loop is removed |
| T14 (re-written T15) | ipc: the `export_invite`, `admit_member` and `import_join_request` tests | LLR-vzf8j2, LLR-8krgzj, LLR-ctrfz4, LLR-pmus9f | the same names (or the renamed ones in T15's list) against the new arguments |

*Withdrawn with T4 (master did the work):* the deletions of
`key_custody::a_signature_verifies_…`, `…does_not_verify_over_a_different_message`
and `verify_against_chain::rejects_bad_signature_before_decoding_delta`.

About 70 annotated tests are rewritten — renamed, re-asserted or
re-annotated — each listed in its task's tables and steps; the rest change
only mechanically (the Wire message's new field, story steps through
`support`).

## Open questions (decisions the inputs do not settle; this plan's default in brackets)

1. **Where the new refusals show in the app** [receiver errors, not
   verification failures — Decision 14, LLR-7bk6qh as amended]. Neither
   `AdmissionNotExpected` nor `AdmissionNotOurs` is a verdict on a change the
   chain does or does not carry; REQ-kn5rtx's wording allows either reading.
2. **No timer in on-chain-client's writer; the app bounds each call at
   90 s** [yes — Decision 9, LLR-be3zv9]. A timer in on-chain-client would
   need tokio as a normal dependency there, which its design forbids.
3. **subxt 0.50.1 (writer evidence, on-chain-client lock) vs 0.50.3 (shipped,
   app lock)** [left as is, recorded in SOUP]. Aligning means bumping
   on-chain-client's lock, which re-baselines its whole gate.
4. **Coverage of the `write` feature** [not measured: `make
   coverage-on-chain-client` runs without `write`, so the writer is outside
   the coverage floors]. Measuring it needs a second llvm-cov run and new
   floors, an owner decision under the Makefile's ratchet rule.
5. **The app never commits a first admission** — booked as PR-kwwap5
   (`app/docs/problems/2026-10-06-first-admission.md`),
   fixed by owner ruling in the transport change that follows [not changed
   here]. The expectation REQ-tcutr6 declares is consulted only by
   `receive_and_verify`, which the shipped app's loop does not call.
6. **Only the founding device can submit.** The proxy account is kept only by
   `commit_genesis` (LLR-3v5nu9: `None` on a first-admission record), so a
   co-administrator on another device has nothing to submit through
   [reported as a refusal by `submit_commit_send`].
7. **No operation abandons a provisional update.** A genesis update that is
   never committed keeps its Organisation private key in the store until a
   later change offers a discard; an Organisation's updates are discarded only
   by a commit that orphans them (REQ-uv3v5w) [no discard operation in this
   change; the owner's "discarded with the provisional update" holds wherever
   an update leaves the store, LLR-qjz3q4]. Does the app need "abandon this
   update"?
8. **A genesis whose epoch is not 1** [refused with `SeqNotEpoch`, Decision
   16]. The contract never produces one, so the refusal is defensive; a root
   republished later at a higher epoch cannot be the genesis the node built.
9. **Outstanding invite identifiers on disk in plain JSON** [yes —
   Decision 13; they are not secret].

## Self-review (plan-change step 9)

1. Every ID of **Implements** has a task whose test verifies it (table above),
   or is done (T1, T2) or already verified on master (listed separately in
   the header).
2. Every remaining task shows real code against the merged tree, real
   commands and the expected red and green. Where a step migrates many call
   sites it gives the exact rewrite rule and names the tests it does not
   apply to. Golden values are derived by the format rule from master's
   pinned bytes, with offsets stated.
3. Names are used consistently across tasks: `check_chain_free`, `InviteId`,
   `ExpectedAdmission`, `expect_admission(org_id, invite_id)` /
   `expected_admissions`, `AdmissionNotExpected`, `AdmissionNotOurs`,
   `Joiner`, `persona_public_keys`, `ProvisionalUpdate`/`ProvisionalChange`
   (`Genesis { members, org_private_key }`), `insert_provisional`,
   `MAX_PROVISIONAL_BYTES`, `create_organisation`/`commit_genesis`,
   `admit_member`/`revoke_member` (synchronous, returning
   `ProvisionalUpdate`), `commit_update` → `CommitOutcome { outgoing:
   OutgoingUpdate }`, `send_update(outgoing, recipient, peer_addr,
   org_secret, invite_id)`, `MockChainOps::apply_genesis`/`apply_update`,
   `first_persona_bound_to`, `commit_held`, `discard_orphans`,
   `still_member`/`forget_organisation`; support's `invite_for`,
   `setup_over`, `setup_counted`, `CountingChain`, `deliver`; app
   `submit::{ChainWriter, OnChainWriter, WriterNotConfigured, WRITE_TIMEOUT,
   found_organisation, submit_commit_send}`, `invitation::{Invite,
   InviteReply, OutstandingInvites, issue_invite, produce_reply, check_reply,
   admit_reply}`.
4. Every task states **Files touched** and **Parallel**; no remaining task is
   parallel. Every `org-node/Cargo.toml`, `org-node/.guardrails/config.yaml`,
   `org-node/src/service.rs` and `tests/support/mod.rs` edit sits in the
   serial T5–T13 chain; each lockfile has its owners, serialised.

## Hand-off

Execute with `develop-change`, task by task in the order above (T5 first;
T1–T3 are done and T4 is withdrawn), each in its own task worktree
(`.guardrails/scripts/task-worktree.sh start <tag>`), filling that task's
**Red→green attestations** with the red output, the green output and the
commit. Then `check-traceability`, `verify-before-merge` and `merge-change`.

## Gate follow-up (2026-10-06)

The first gate run (tree `63a5b37`) was not green:

1. org-node `UNDECLARED-DEPENDENCY REQ-prjja8`: a correction note cited the
   app's requirement by ID; now cited by path. Fixed.
2. on-chain-client coverage short of class C (about 42% lines and regions,
   decisions unmeasured, the `write` feature not in the coverage target).
   **Owner accepted the gap for this merge (2026-10-06)**; the writer's
   absence from the coverage run is filed as PR-b795an. org-node and app have
   no coverage command.
3. 26 Implements IDs with no direct `verifies:` test: all are SDDs, RCs
   (RC-mj6gjq, RC-2ferct, RC-wzb48r) or app REQs whose LLRs carry tests;
   covered transitively, which check-trace accepts (no MISSING-TEST in any
   unit). *Corrected after the second gate run:* 24 of the 26 are carried by
   tested LLRs (the run lists each); SDD-z85ux9 (org-node's chain-facing I/O
   shell) and SDD-6g3wnh (the app's shell) carry no LLR and no gated test by a
   deviation already recorded in their decomposition files on master. This
   change amends their text and nothing about that deviation, which stands.
4. Robustness, judged by the gate from test names only; re-judged from the
   bodies by a dispatched review:
   - both kinds already present: LLR-nvn3wk, LLR-48jakr, LLR-b27jr6,
     LLR-q3aj8z, LLR-9zfnmb, LLR-437fvx, REQ-kt877x, LLR-3f5h7b;
   - the other kind present under a sibling ID covering the same behaviour:
     REQ-uv3v5w, REQ-qn2erx, LLR-s6qnht, LLR-4tcxsu, LLR-mkj4bz, LLR-68yd3j,
     LLR-dzte8x, LLR-rb8r65, LLR-bg3vsw, LLR-j83kc8, LLR-cja9zv, LLR-tax3pm,
     LLR-ewkg85, LLR-3wb7th, LLR-379hnv, LLR-pw369n, on-chain-client
     LLR-a2acvh, LLR-kv27gp, LLR-z9vugt, LLR-hun4wf (each pair named in the
     review report, copied to the verification record);
   - absence or type-shape statements with no abnormal input: LLR-65py3d,
     LLR-rys5nx, LLR-g9vmbx, LLR-8qxwst, LLR-zj88e6, LLR-qezw3n, LLR-836z24,
     LLR-rv4vux, LLR-rc74nq, LLR-txqmz4, LLR-66h529, LLR-463d89, LLR-8m3bwj,
     LLR-f74xwb;
   - genuinely missing, now added:
     - admitting_into_an_organisation_not_held_touches_no_other (LLR-vdyu65)
       — red under mutant "admit-fallback" (`find_org` falls back to the first
       record): `Chain("no persona bound …")` ≠ `OrgNotOnChain`; restored, green
     - degenerate_seeds_still_give_valid_keys (LLR-ctzkv7) — red under mutant
       "unclamped" (`mul_base(from_bytes_mod_order)` in place of
       `mul_base_clamped`): the all-zero seed's member key refused; restored,
       green
   org-node now 225 passed.
5. Commands without counts: quint typecheck and check-ids print nothing on a
   pass (exit status is their only signal); on-chain-client's bolero fuzz
   targets printed nothing in the combined run and ran normally alone
   (153026 iterations/s); svelte-check's one warning ("Cannot find type
   definition file for 'node'") is environmental — `app/tsconfig.json` is
   unchanged from master and the warning was seen before any app edit (T14).

## Independent review round 1 (2026-10-06, tree 8920b33)

Reviewer: fresh subagent, own worktree, suites re-run (org-node 225,
on-chain-client 73 + 23, app 154, vitest 40, quint clean). Verdict: "The
safety replacements for the removed signatures and sender checks hold on every
receive and commit path … Not mergeable as is: finding-1 (high) … finding-2
(medium)". Findings verbatim are copied into the verification record.

Planned dispositions:

- finding-1 (code, high), Persona rebinding. Owner ruling 2026-10-06: one
  Persona, one Organisation. New REQ-yp75u9 (org-node), LLR-6z5xya
  (`create_organisation` refuses a bound Persona) and LLR-eyc4ud (no first
  admission or commit rebinds a Persona bound elsewhere), app LLR-rt8gdz
  (founding and replying offer unbound Personas only; the backend refuses a
  bound one). Resolves PR-mdv38y. Tasks R1a (org-node), then R1c (app).
- finding-2 (requirement, medium), proxied inner failure reported as success.
  REQ-6jefu2 and REQ-aat4yt amended to name it; LLR-24mwew (on-chain-client):
  the writer reads pallet-proxy's `ProxyExecuted` result and turns an inner
  failure into a typed error. Task R1b.
- finding-3 (code, low): recorded as PR-924ftr (app).
- finding-4 (code, low): mechanical — `a_reply_for_another_organisation_is_refused`
  uses a second Organisation the inviter holds; a test for LLR-gha5f6's
  "settles even when the send fails" clause. Task R1c.
- finding-5 (record): REQ-6jefu2/REQ-aat4yt wording and the risk file's
  "device key" fixed by the dispatcher.
- finding-6 (record): org-node/docs/CONTEXT.md's Organisation secret, Wire
  message and Persona store entries brought up to date. Task R1d.
- finding-7 (record): this plan's Implements citations of DRAFT names updated
  to the dated files; REQ-xs4ab8's abnormal case recorded (verdict b: the
  refusals under LLR-rb8r65 — `a_joiner_with_a_member_key_already_held_is_refused`,
  `an_admission_refused_by_the_bound_writes_nothing` — keep nothing). Task R1d.

### R1b — attestations (DONE 2026-10-06, merged)

on-chain-client: reader line 73 passed; write line 29 passed (write_events 6
new). LLR-24mwew added under SDD-yg7n55: `outcome_of(proxied, events)` turns a
`ProxyExecuted` error, or a missing `ProxyExecuted` for a proxied call, into a
typed failure (`WriteError::InnerCallFailed` / `EventNotFound`).

- a_proxied_call_whose_inner_result_is_ok_executed — compile-red; passes against an `Executed` stub (correct answer); runtime-red against an `Err` stub; green
- a_proxied_call_whose_inner_result_is_an_error_is_a_typed_failure — compile-red; runtime-red against the `Executed` stub (the defect); green
- any_inner_error_among_several_proxy_executed_events_is_a_failure — compile-red; runtime-red against the `Executed` stub; green
- a_proxied_call_without_proxy_executed_is_a_failure — compile-red; runtime-red against the `Executed` stub; green
- a_call_not_made_through_the_proxy_executed_on_a_successful_extrinsic — compile-red; runtime-red against the `Err` stub; green
- is_proxied_is_true_only_for_proxy_proxy — compile-red; runtime-red against an `is_proxied → false` stub; green

Finding: `SubxtWriteOps` never yields `DispatchOutcome::ApprovalRecorded`
(threshold-1 `as_multi` dispatches at once); LLR-hun4wf's PendingApproval
branch is reachable only through the test substitute — recorded in LLR-24mwew's
note. The decoded error text has not been checked against a live chain
(chopsticks e2e outside the gate).

### R1a — attestations (DONE 2026-10-06, merged)

org-node: 228 passed (admission_sender 46, commit_paths 29). LLR-6z5xya and
LLR-eyc4ud added (REQ-yp75u9); LLR-mxskg9 gains `PersonaAlreadyBound`;
LLR-3f5h7b, LLR-w3fhhg, LLR-q3aj8z, LLR-e5c9ud, LLR-rjg3m2 amended in place.
PR-mdv38y resolved. Each test below was compile-red first (the variant did
not exist); then, with only the variant added, runtime-red as stated; green
after the fix.

- commit_paths::a_persona_bound_to_an_organisation_cannot_found_another (the reviewer's scenario) — the second `create_organisation` returned Ok
- commit_paths::commit_genesis_refuses_a_persona_bound_to_another_organisation — the second `commit_genesis` succeeded and rebound the Persona
- admission_sender::a_first_admission_binds_the_unbound_persona_not_one_bound_elsewhere — b1 was rebound to org 2 with a new member id
- admission_sender::a_first_admission_listing_only_a_persona_bound_elsewhere_is_refused — B committed org 2 and rebound its only Persona
- admission_sender::pr_mdv38y_an_update_enrolling_another_organisations_persona_does_not_rebind_it (rewritten) — b1 moved to org 2
- admission_sender::pr_mdv38y_a_member_persona_cannot_found_another_organisation (rewritten) — `create_organisation` on a bound member Persona returned Ok

Deleted: admission_sender::same_persona_founding_two_organisations_gets_two_member_ids
(REQ-d9g6nt, LLR-rjg3m2) → commit_paths::a_persona_bound_to_an_organisation_cannot_found_another
(the refusal) and commit_paths::the_founding_member_id_and_the_organisation_key_are_drawn_not_derived
(two drawn ids for two genesis updates).

### R1c — attestations (DONE 2026-10-06, merged)

app: cargo 158 passed (invitation 16); vitest 43 passed; `npm run check` 0
errors. LLR-rt8gdz added (derived, assessed in the app risk file); LLR-w4mhd4
and LLR-gha5f6 amended in place; `PersonaAlreadyBound` classified as a
receiver error (LLR-7bk6qh).

- locally_reachable_variants_are_classified_as_receiver_errors (rewritten) — compile-red (E0004); runtime-red with the variant in the verdict arm; green
- a_reply_for_another_organisation_is_refused (rewritten, finding-4) — green on the current code; red under a mutant removing the `reply.org_id != org_id` check; restored
- a_reply_is_settled_once_committed_even_when_the_send_fails (finding-4) — green on the current code; red under a mutant returning before `settle`; restored
- no_reply_is_produced_and_nothing_declared_for_a_bound_persona — red before the fix (a bound Persona produced a reply); green
- a_bound_persona_cannot_reply_and_an_unbound_one_on_the_same_device_can — red before the fix; green
- a_bound_persona_founds_nothing_and_the_chain_is_not_written — passes on the current code (org-node refuses); red under a temporary org-node mutant skipping `ensure_unbound` (geneses 2); reverted, org-node untouched
- vitest ×3 for `unboundPersonas` — red against a throwing stub (all three; two also against an identity stub); green

### R1d — DONE 2026-10-06 (dispatcher, commit fe4fed1)

No test. org-node/docs/CONTEXT.md: Wire message (invite identifier, snapshot
on every update), Persona store (provisional updates, expected admissions), new
*Expected admission*, Organisation secret (no administrator); this plan's
citations moved to the dated files. **Correction (review round 2,
finding-5):** round 1's planned disposition for finding-7 named the wrong
tests for REQ-xs4ab8's abnormal case. REQ-xs4ab8's abnormal case is verified
at its own LLR: LLR-6dc598 (`satisfies: REQ-xs4ab8`) by
`commit_paths::revoking_a_member_the_record_does_not_hold_keeps_nothing` —
a revocation the record cannot build keeps nothing; and LLR-6z5xya by
`commit_paths::a_persona_bound_to_an_organisation_cannot_found_another`.

## Independent review round 2 (2026-10-06, tree 85eda64)

Reviewer: fresh subagent, own worktree; suites re-run (org-node 228,
on-chain-client 73 + 29, app 158, vitest 43, quint clean). The permission
classifier refused its mutation probes and one test-file read, so for R1c it
relied on the plan's attestations (recorded in the verification record).
Verdict: "Not mergeable as is. The safety replacements and the round-1
dispositions hold on every receive and commit path, but finding-1 (medium) is
open". Findings verbatim go to the verification record. Planned dispositions:

- finding-1 (code, medium), outstanding invites not tied to their
  Organisation: REQ-65xqp8, LLR-f35pda, LLR-gha5f6 amended; outstanding
  entries keyed by (Organisation, invite id). Task R2a (app); finding-8 (app
  risk note → PR-924ftr) in the same task.
- finding-2 (code, low), event decoding untested: pure decoder with tests,
  LLR-24mwew amended. Task R2b, with findings 6 and 7 (on-chain-client docs).
- findings 3, 9, 10, 11 (record): org-node docs. Task R2c.
- findings 4 and 5 (record): this plan — fixed by the dispatcher (this
  section, the R1d block above, the Implements list, the open/resolved PR
  list, the risk-file path, the write-line note).

A medium finding means a further review round (round 3) after these fixes.

### R2b — attestations (DONE 2026-10-06, merged)

on-chain-client: reader 73 passed; write 34 passed (write_events 11).
LLR-24mwew amended (round 2, finding-2): `dispatch_result` and
`observed_event` are pure; the subxt shell only fetches events. PR-b795an
names write_events (finding-6); the SDD-yg7n55 file list puts `BlockSink`
under subxt_ops.rs (finding-7). Each test compile-red first (E0432), then:

- a_proxy_executed_result_of_ok_decodes_to_ok — runtime-red against an `Err(MalformedEvent("stub"))` stub; green
- a_proxy_executed_result_of_err_decodes_to_err_carrying_the_error_text — runtime-red against an all-Ok stub; also red under an Err→Ok mutant; green
- a_proxy_executed_without_a_result_field_is_a_typed_decode_failure — runtime-red against the all-Ok stub; green
- a_proxy_executed_result_that_is_not_ok_or_err_is_a_typed_decode_failure — runtime-red against the all-Ok stub; green
- observed_event_decodes_proxy_executed_and_ignores_the_fields_of_other_events — runtime-red against a stub; also red under the Err→Ok mutant; green

### R2c — DONE 2026-10-06 (merged; no test)

org-node docs: findings 3 (deleted/renamed test citations, history notes
kept with dated follow-ups), 9 (risk-file path, PR-szkat6 wording, postcard
SOUP leaving note), 10 (DevicePublicKey; Organisation secret on any update),
11 (the 2026-10-05 note restored word for word, a dated 2026-10-06 note added).

### R2a — attestations (DONE 2026-10-06, merged)

app: cargo 159 passed (invitation 17); vitest 43; `npm run check` 0 errors.
REQ-65xqp8, LLR-f35pda, LLR-gha5f6 and LLR-pmus9f amended in place (round 2,
finding-1): outstanding invites are (Organisation, invite id) pairs; a reply
is acted on only for the Organisation its outstanding pair names, and the
caller's selected Organisation is refused unless it is that one. An old
bare-id file is refused, naming the file (not migrated; nothing is deployed).
Finding-8 fixed in the app risk file (qualified; cross-reference PR-924ftr).

- a_reply_naming_another_held_organisation_with_an_outstanding_invite_id_is_refused — runtime-red on the old code (`check_reply` accepted the cross-Organisation reply; with that assertion removed, `admit_reply` admitted the person into the second Organisation — finding reproduced); green
- import_invite_reply_refuses_a_malformed_blob_or_an_unknown_invite and import_invite_reply_reports_the_reply_it_parses (ipc, harness file now in pair format) — runtime-red on the old code (startup refused the pair file); green
- an_outstanding_invites_file_that_is_not_a_list_of_pairs_is_refused, outstanding_invite_ids_survive_a_reopen_and_settle_one_at_a_time, a_reply_admitted_under_another_selected_organisation_is_refused (rewritten) — compile-red only (written against `holds`/`settle(org, id)`); the old code would fail each on behaviour (it accepted the bare-id file, rejected the pair file, returned another message)

## Independent review round 3 (2026-10-06, tree bd9c329) — last round

Reviewer: fresh subagent, own worktree; suites re-run (org-node 228,
on-chain-client 73 + 34, app 159, vitest 43, quint clean); mutation probes
allowed this round, each turning tests red (app `holds` ignoring the
Organisation; expected-admission check skipped; `check_chain_free` removed;
own-Persona rule skipped; every expectation cleared; `commit_update` never
forgetting; `commit_genesis` epoch rule removed; a bound Persona rebound).
Verdict: "Mergeable. The only findings are record findings … this is the last
round." Four record findings, all fixed by the dispatcher:

- finding-1: dated 2026-10-06 notes on the PR-mdv38y history in
  org-node/docs/problems/2026-10-04-receive-persona-rebind.md and
  org-node/docs/risk/2026-10-03-architecture-derived.md (two places), and on
  org-node/docs/architecture/README.md's pinned-defects paragraph.
- finding-2: app/docs/CONTEXT.md *Invite* — outstanding pair, released only
  when a reply naming both has been acted on.
- finding-3: org-node/docs/CONTEXT.md *Expected admission* — declared when the
  user confirms the reply.
- finding-4: this plan's on-chain-client write line gains `--test write_events`.

No code or requirement finding: per merge-change 6a the sequence reruns from
step 1 and ends at 6b; no further reviewer is dispatched.

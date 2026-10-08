# S2: create org-io — design and implementation plan

**Goal:** stage S2 of `docs/plans/2026-10-06-org-io-roadmap.md`. It creates
the org-io unit and moves into it the chain read, the app's chain-write
submission wiring, custody of the user's own signatory key (the development `ODS_ADMIN_SEED`
path), and the new
own-admin signatory check. After S2 the app depends only on org-io, and
org-node meets org-io by values in, values out. Behaviour does not change.
S2 also resolves PR-b795an.
**Status:** design draft, change `worktree-org-io-create`, branched from
master `c036fab`. Revised 2026-10-06 to the owner's rulings on the S2–S4
design drafts (the roadmap's section of that name, rulings A to D and the
accepted recommendations); §7 lists how each was applied. The org-io
guardrails skeleton exists on this branch (§3); its first requirements are
drafted in `org-io/docs/requirements/DRAFT-worktree-org-io-create-org-io.md`
and assessed in `org-io/docs/risk/DRAFT-worktree-org-io-create-org-io.md`.
No code has been written. The develop phase was gated on
`worktree-org-node-org-key-pair` merging (see the roadmap, "Overlap with the
key-pair change"). *Reconciled 2026-10-07:* that change merged to master as
`1f52c36`, and master was merged into this branch (`6a0f208`). §1 and §6 were
re-surveyed against the merged code and ledgers, and every line number below
is as of `1f52c36`. The plan's tasks are in "Tasks" at the end of this file.
*Owner rulings of 2026-10-07* on the two "Still open" items (§7): the chain
read stays coupled to the development seed in S2 and is split from the write
in S8; the own-admin check is org-io's function and tests only in S2, not
wired into the app, and its first caller is S3b-io (stage S3's org-io half).
*Reconciled again 2026-10-08 with S3a:* master, now carrying change S3a
(`60c7d58`, the commit workflow: `revocation.rs`, `reconcile.rs`, the
member-sender rule, the three Wire kinds), was merged into this branch
(`35e634b`). §1's line numbers were re-taken against that tree, and §2, §6,
the Conventions and tasks T6–T10 were rewritten so that each develops as
written against it (T6–T9 in full; T10 for the carried on-chain-client
item).
**ADR:** `docs/adr/2026-10-06-org-io-unit.md` (Accepted 2026-10-08,
Proposed when this plan was written; renamed from its
`DRAFT-` name because check-units convicts a DRAFT-named file under the
disclaimed `docs`, and finalize-docs does not rename ADRs).
**Safety class:** C in every unit touched (org-io, org-node, app,
on-chain-client). No per-item override.

## 1. What moves, and where it is today

Line numbers are as of `35e634b` (this branch with master `60c7d58`, S3a,
merged in), re-taken on 2026-10-08; before that they were as of `1f52c36`,
and the first draft's as of `c036fab`. Rows for code T5 already moved keep
the location it moved from and say so.

| Moving part | Today | After S2 |
|---|---|---|
| `ChainOps` trait, `MockChainInner`, `MockChainOps` (with its `apply_genesis`/`apply_update` stand-ins, LLR-ryzr8m), `ChainOpsReader` | `org-node/src/service.rs:34-131` (trait `:41-44`), `ChainOpsReader` `:1260-1275` (used by `verify_received` `:1035` and S3a's `commit_step` `:1318`); re-exported at `org-node/src/lib.rs:34-40` (`SubxtChainOps` at `:39-40`) | **Gone** (ruling B). `OrgService::new(store)` holds no chain; its chain-judging operations take an `Option<OrgState>` argument. A test-support value store, `org_node::test_fixtures::ChainSlots`, replaces `MockChainOps`' slots and keeps `apply_genesis`/`apply_update` (LLR-ryzr8m, amended) |
| `ChainReader` trait and `MockChain` (sync view for `verify.rs`) | `org-node/src/chain.rs:33-62` (and its unit test `:64-79`); `verify_envelope_against_chain<C: ChainReader>` at `org-node/src/verify.rs:69` | Replaced by the value: `verify_envelope_against_chain(local, envelope, ctx, chain_state: Option<OrgState>)`. `MockChain` and the `FailingChain` of `org-node/tests/verify_against_chain.rs:119-138` go; a failed read is org-io's to report (LLR-rm9x4z, moved in T5) |
| `OrgService` call sites that read the chain | `commit_genesis` (`service.rs:412`, read `:421`), `commit_update` (`:607`, read `:613`), S3a's `receive_revocation` (`:700`, read `:708`), `verify_received` (`:1027`, read `:1034`) under `receive_and_verify` (`:780`) and `receive_and_self_delete_if_revoked` (`:948`), S3a's interim `reconcile` (`:1062`, read `:1072`) | Take the state as an argument. `commit_genesis`, `commit_update` and `reconcile` become synchronous. Each receive path splits in three (T6 has the exact shape): `receive_message` (transport, async until S4; returns the authenticated sender with the message, as S3a's `receive_one` does), a synchronous chain-free phase that takes the sender and the message and either finishes (an acknowledgement, decided against the store alone) or returns a pending value naming the Organisation (every check before the read: the sender rule, `check_notice`, `AdmissionNotExpected`, the snapshot decode, `check_chain_free`), then org-io reads the chain once, then a synchronous apply that takes the state: `revocation::accept`, or `verify_envelope_against_chain`, the carried-key check `check_carried_key` (LLR-ba2ejp, RC-9cefcn) and the commit or the removal step |
| `SubxtChainOps` (production read) | `org-node/src/service.rs:137-175` (`mod subxt_impl`, `#[cfg(feature = "chain")]`) | org-io, `OnChainStateReader` behind org-io's own `StateReader` trait (org-io's test seam; ruling B binds org-node, not org-io); copied by T5, deleted from org-node by T6 |
| `connect_chain_client` (LegacyBackend + reconnecting RPC, builds `OrgRegistryClient`) | `org-node/src/service.rs:177-217` | org-io (`connect`); copied by T5, deleted by T6 |
| `org_state_from_chain` | `org-node/src/chain_read.rs:21-23` | org-io (it is the bridge from on-chain-client's type to org-node's parse); copied by T5 |
| `OrgStateCache`, `OnChainReader` | `org-node/src/chain_read.rs:25-98` (98 lines in all; `OnChainReader` unused by production, per its own module doc; module and re-export at `lib.rs:17-20`) | **Deleted**, which resolves PR-k2xxaq (its defect, `refresh` keeping a superseded state after a failed fetch, is in the deleted code) |
| `OrgState::from_chain` (the parse edge, REQ-8jb4ny) | `org-node/src/chain.rs:20-30` | Stays in org-node; org-io calls it |
| Operator preflight | `org-node/src/preflight.rs` (138; also checks the transport through `org_node::transport`), `org-node/src/bin/preflight.rs` (97; decodes `ODS_DEVICE_SEED` and the H160s with `hex`), test `org-node/tests/preflight.rs` (two transport tests and one chopsticks chain test over `tests/common/`, not in `verify_commands`) | The chain checks moved to org-io (T5). org-node keeps a transport-only preflight until S4 (T6 step 7, as amended at T5), because org-io's LLR-3zdw8v bars `SigningKeypair` and `DeviceSeed` from org-io's source |
| org-node's `chain` feature and its `subxt`, `on-chain-client`, `hex`, `async-trait` deps; `app = ["chain", …]`; the chopsticks dev dependencies (`jsonrpsee`, `libc`, `async-trait`, `on-chain-client`, `subxt`; `serde_json` too, but `tests/encoding_golden.rs` also uses it) | `org-node/Cargo.toml:14`, `:18`, `:45-49`, `:70-78`; `[[bin]] preflight` `:86-88`; `[[test]]` `preflight` `:125-128`, `chain_read_state` `:179-182` | Removed, except `serde_json`, which stays for `encoding_golden`; `app` keeps `transport` and the store's crypto until S4; `tokio` stays behind `transport` |
| Test `chain_read_state` (LLR-mmdu38's cache clause, the parse) | `org-node/tests/chain_read_state.rs`, `[[test]]` with `required-features = ["chain"]`, in org-node's `verify_commands` | The two parse cases move to org-io's `chain_read_rule` (they call `org_state_from_chain`, which moves); the cache cases are deleted with the cache; the target leaves org-node's manifest and `verify_commands` |
| `ChainWriter`, `OnChainWriter`, `WriterNotConfigured`, `NOT_CONFIGURED`, `WRITE_TIMEOUT`, `bounded`, `found_organisation`, `submit_commit_send` | `app/src-tauri/src/submit.rs` (142 lines: `:22`, `:24`, `:27-34`, `:37-41`, `:44-81`, `:84-95`, `:98-110`, `:116-142`) | org-io. Since the key-pair change, `submit_commit_send` takes no key, secret or invite identifier (LLR-q225ws): org-node picks the message kind from its committed record |
| Test `submit_flow` (seven tests: five for LLR-qhjp6g/LLR-be3zv9, paused tokio clock; two for LLR-q225ws, added by the key-pair change, `:167`, `:209`) and its fixture `FakeWriter` over `MockChainOps` (`app/src-tauri/tests/support/mod.rs`) | `app/src-tauri/tests/submit_flow.rs` | The five LLR-qhjp6g/LLR-be3zv9 tests move to org-io, `verifies:` lines unchanged (ruling A). The two LLR-q225ws tests stay in the app: they drive the app's own `revoke_and_send` and `admit_reply`, which choose the recipient. `FakeWriter` becomes org-io's test-support `FakeChain` |
| `ChainNotConfigured` | `app/src-tauri/src/state.rs:78-95` | org-io (one "not configured" value for read and write) |
| `build_chain_ops`: env read and parse of `ODS_CHAIN_WS`, `ODS_CONTRACT_H160`, `ODS_ADMIN_SEED`, `ODS_COSIGNER_PUB` | `app/src-tauri/src/state.rs:214-303` (seed parse `:253-264`, co-signer `:266-280`) | org-io reads `ODS_ADMIN_SEED` itself, only under its development-only `dev-seed` feature (REQ-8zuka3), and parses it and `ODS_COSIGNER_PUB` with total functions. The app keeps reading `ODS_CHAIN_WS` and `ODS_CONTRACT_H160` and hands them to org-io as values. Note: today's seed and co-signer errors format `hex::FromHexError`, whose text names the offending character and its position (`:262`, `:271`); REQ-qnh9pb's errors carry no part of the value |
| `connect_chain`: builds the sr25519 `Keypair` and the writer | `app/src-tauri/src/state.rs:305-341` | org-io; the key lives only inside org-io |
| `AppState.writer: Box<dyn ChainWriter>`, `AppState.service: Mutex<OrgService>`, `assemble`, `for_test` | `app/src-tauri/src/state.rs:54-76`, `:162-188`, `:196-211` | Replaced by one org-io handle, `AppState.org_io: Mutex<OrgIo>` |
| Call sites of the write | `app/src-tauri/src/commands.rs:122` (`create_organisation` → `found_organisation`), `:226-235` (`admit_member` → `invitation::admit_reply`), `:281` and `:292-312` (`revoke_member` → `revoke_and_send` → `submit_commit_send`), `app/src-tauri/src/invitation.rs:349-384` (`admit_reply`, the write at `:370`) | Call org-io's handle; the app still chooses the recipient (LLR-q225ws) |
| Call site of the read (receive) | `app/src-tauri/src/commands.rs:369-376` (`next_outcomes` → `receive_and_self_delete_if_revoked`) | Calls org-io's `receive_and_self_delete_if_revoked`, which returns the same `Result<SelfDeleteOutcome, OrgNodeError>`, so `events::outcomes_for_receive_error` is unchanged |
| Signatory read | Nowhere. Planned for org-node at `org-node/docs/requirements/2026-10-06-chain-authority.md:89-91` and sequencing item 4 (`:226`); `multi_account_id` exists at `on-chain-client/src/write/multisig.rs:15`; no reader of `Revive.OriginalAccount` or `Proxy.Proxies` exists | org-io, through two new on-chain-client reads (§4) |
| Writer coverage (PR-b795an) | `make coverage-on-chain-client` builds without `write` | The recipe adds `--features write` and the four `write_*` targets (`ON_CHAIN_CLIENT_COVERAGE_ARGS` once the branch-coverage change is on master) |

## 2. Architecture: org-io owns the node, values cross the boundary

org-io is a crate that owns the `OrgService`, the chain connection, the
reader, the writer and the user's signatory key. The app holds one org-io handle.

**Two keys, kept apart** (owner, 2026-10-06). The Organisation key pair is
Organisation-wide: the X25519 pair from the key-pair change, held in
org-node's record, for encryption and key agreement only. It never signs and
is outside S2's custody scope. There is no Organisation-wide signing key: the
key org-io holds is the device's user's own sr25519 account key, one
signatory of the multisig that controls each pure proxy the user administers.

```
app (UI; decides when)      ── depends only on ──▶  org-io
org-io (all IO; workflow)   ──▶ org-node  (OrgService; values in, values out; verify)
                            ──▶ on-chain-client [write]  (reader, writer, two new storage reads)
                            ──▶ person    (declared when first named; maybe S4)
org-node                    ──▶ org-members, person   (on-chain-client edge removed)
```

**Values in, values out (ruling B).** A publish in S2 runs, inside org-io:
org-node builds the provisional update and returns its values (root,
Organisation public key, expected epoch, proxy account); org-io submits them
through on-chain-client's writer under the 90 s bound; on success org-io
reads the chain state at the latest finalised block, parses it through
org-node's `OrgState::from_chain`, and calls org-node's synchronous commit
with that state; then org-io asks org-node to send (async until S4). A
receive runs: org-node receives a message and the Device that sent it
(async until S4); org-node's chain-free phase takes both as values and
either finishes (an acknowledgement is decided against the store alone) or
names the Organisation whose state it needs; only then org-io reads the
chain for it; org-node verifies and applies synchronously against that
value. Nothing in org-node awaits the chain and nothing in org-node names
subxt or on-chain-client.

*Since S3a (2026-10-08 reconciliation).* S3a's decisions are already values
in, values out: `revocation::accept` takes `chain: Option<OrgState>`,
`reconcile::reconcile` takes `chain: OrgState`, `check_notice` and
`check_acknowledgement` take no state, and all of them take the store as
`&StoreData`. What still reads the chain is `OrgService`'s wiring around
them, which T6 cuts as above. The device-seed sources S3a made lazy
(`impl FnOnce() -> Vec<DeviceSeed>`, called only on the path that signs)
are not IO in S2: they read the Persona seeds org-node's own store holds,
so they stay closures inside org-node's apply and commit operations. S4
moves the seed to the OS keychain and makes the source a value org-io
passes in; S2 cannot, because org-io's LLR-3zdw8v bars `DeviceSeed` from
org-io's source.

Software items for org-io. *Minted 2026-10-07 and drafted, with their
low-level requirements, in
`org-io/docs/architecture/DRAFT-worktree-org-io-create-org-io.md`:* the chain
read SDD-6qz9ms, submission SDD-erzj3m, key custody SDD-789u6d, the
own-admin check SDD-f4khqn, the handle SDD-z3ychz; ruling A brings
SDD-z85ux9 across as the fifth (task T5).

- **The chain connection** (SDD-z85ux9, moved). `connect` (moved
  `connect_chain_client`), the configured-or-not wiring
  (`ChainNotConfigured`), and the preflight. This is the untested subxt shell;
  its deviation (no LLRs) moves with the item and is argued here.
- **The chain read.** `read_org_state` and `org_state_from_chain`, with
  LLR-rm9x4z moved and re-parented to org-io's chain read requirement.
- **Submission.** `ChainWriter`, `found_organisation`, `submit_commit_send`,
  and the 90-second bound, with LLR-qhjp6g and LLR-be3zv9 moved.
- **Key custody** (the user's signatory key). Reading `ODS_ADMIN_SEED` under
  the `dev-seed` feature only (development builds), parsing it (64 hex characters
  after at most one `0x`, 32 bytes) and the co-signer list, building the
  sr25519 `Keypair`, and holding it. Nothing outside org-io gets the key or
  its bytes: no getter, and `Debug` is redacted (the pattern of org-node's
  `secret_redaction`).
- **The own-admin check** (§4).
- **The handle.** One `OrgIo` value owning the above plus `OrgService`, with
  the transitional re-export module for the org-node types the app's commands
  use.

## 3. The org-io skeleton

### Created on this branch (design phase)

```
org-io/
  AGENTS.md                         # terse: all IO; org-node IO-free; app talks only to org-io
  .guardrails/config.yaml           # class C; ledgers under org-io/docs; depends_on org-node, on-chain-client
  docs/requirements/README.md       # + DRAFT-worktree-org-io-create-org-io.md
  docs/risk/README.md               # the acceptability matrix (S3 unacceptable at every probability)
  docs/architecture/README.md
  docs/architecture/soup.md         # empty inventory until the crate lands
  docs/problems/README.md
  tests/README.md                   # placeholder so test_paths names a file; see below
```

Two placeholders are deliberate and documented in the config. `strict_paths`
names `org-io/docs` and `org-io/AGENTS.md` (the documentation-first form
`lib.sh` prescribes) because `org-io/src` does not exist yet, and
`test_paths` names `org-io/tests`, which holds only a README until the
first test. With no tests, every drafted requirement is reported
`MISSING-TEST` by check-trace: that is the correct conviction for
requirements with no code, and it clears in the develop phase. Neither
placeholder weakens a gate: `strict_paths` still scans everything org-io has,
and the `verifies:` search still runs over `test_paths`. `verify_commands`
and `coverage_command` are commented out until the crate exists.

### Added in the develop phase

```
org-io/
  Cargo.toml          # [package] org-io; GPL-3.0-only; publish = false; [lints] workspace = true
  src/
    lib.rs            # OrgIo handle; transitional re-exports of org-node types
    connect.rs        # connect (was org_node::service::connect_chain_client), ChainNotConfigured
    chain_read.rs     # read_org_state, org_state_from_chain
    submit.rs         # ChainWriter, OnChainWriter, WriterNotConfigured, WRITE_TIMEOUT, found_organisation, submit_commit_send
    custody.rs        # ODS_ADMIN_SEED read (dev-seed feature only); seed / co-signer parsing (total fns); SigningKey wrapper (redacted Debug, no getter)
    signatory.rs      # own-admin check
    preflight.rs      # moved from org-node
    bin/preflight.rs  # moved from org-node
  tests/
    chain_read_rule.rs    # org_state_from_chain on values: present, absent, refused key
    submit_flow.rs        # moved from app, annotations unchanged
    key_custody_config.rs # seed/co-signer parse: normal + abnormal
    signatory_rule.rs     # the pure decision of §4, no chain
    absences.rs           # no signatory key reachable from the public API; the only env read is behind dev-seed (source scan, as org-node's absences.rs)
    fuzz_seed_parse.rs    # bolero over both parsers (project rule: always fuzz)
    preflight.rs          # chopsticks; NOT in verify_commands (as today)
```

At that point the config switches `strict_paths` to `org-io/src` (keeping
the ledger directories if wanted), deletes `tests/README.md`, and enables:

```yaml
verify_commands:
  - cargo test -p org-io --features test-support --lib --test chain_read_rule --test submit_flow --test key_custody_config --test signatory_rule --test absences --test fuzz_seed_parse
  - cargo clippy -p org-io --all-targets -- -D warnings
coverage_command:
  - make coverage-org-io
  - make coverage-branch-org-io
```

Dependencies are taken from what moves: `org-node` (path, features `app`
until S4 splits it), `on-chain-client` (path, `dev-rpc`, `write`), `subxt` 0.50
(`native`, `jsonrpsee`, `reconnecting-rpc-client`), `subxt-signer` 0.50
(`sr25519`, `subxt`), `tokio` (`rt-multi-thread`, `macros`, `time`), `hex`,
`rand_core`, `zeroize`. `async-trait` only if `ChainWriter` stays a trait
object (it is org-io's own seam for the submit tests, so ruling B does not
apply to it). Dev dependencies: `tokio` `test-util`, `bolero` 0.13.
**Workspace:** `app/src-tauri` is its own workspace (`[workspace]` in its
manifest), and on-chain-client is its own too. org-io joins the root
workspace beside org-node, so `cargo test -p org-io` works. Check whether
the root workspace's subxt feature set unifies with on-chain-client's pins.
The org-node manifest comment already relies on this.

### Coverage targets (from the first merge)

- `coverage-org-io` in the Makefile, with `ORG_IO_COVERAGE_ARGS` shared
  between the stable recipe and the branch recipe. Line, region and branch
  floors are one point below the first measurement, rounded down. Expect a
  low line figure: `connect.rs`, `OnChainWriter` and the read's subxt call are
  the shell that no gate reaches, as with on-chain-client's `client.rs`.
- **Branch twin:** `worktree-guardrails-branch-coverage` (tip `4811cff` on
  2026-10-07) is not yet on master (`1f52c36`), and is about to merge. It
  brings `BRANCH_TOOLCHAIN := nightly-2026-10-03`, `BRANCH_TARGET_DIR`, the
  `*_COVERAGE_ARGS` variables shared by each stable recipe and its
  `coverage-branch-<unit>` twin, and the `judge_branches` macro (`jq -e` on
  the JSON summary). The owner ruled that org-io has branch floors from its
  first merge, so S2's develop phase starts by merging master again once that
  change is on it (task T0), and `coverage-branch-org-io` reuses that pattern
  rather than bringing its own.
- **PR-b795an:** add `--features test-support,write` and the four `write_*`
  targets to `coverage-on-chain-client`, re-measure, and move the floors
  (upward only; if the figure falls, record why as the 2026-09-10
  recalibration did). If the branch-coverage change has merged, the same
  applies to `ON_CHAIN_CLIENT_COVERAGE_ARGS`, which both recipes read.

## 4. The own-admin check (design note)

What the chain holds: the Organisation's slot is keyed by `OrgAdmin =
h160_of(proxy)` (`on-chain-client/src/write/ceremony.rs`, `genesis`). The pure
proxy is mapped in pallet-revive (`map_account_call`), so
`Revive.OriginalAccount(h160)` gives back the 32-byte proxy account.
`Proxy.Proxies(proxy)` lists the proxy's delegates. The controller is either
the signatory itself (no co-signatories) or `multi_account_id(sorted
signatories, 1)`. Every dispatch is `as_multi_threshold_1`
(`on-chain-client/src/write/multisig.rs`, `build_dispatch_tx`).

What it does not hold: pallet-multisig stores no member list, because a
multisig account is a hash. So a chain read can **confirm** a signatory set
that the caller supplies; it cannot list the set. By the accepted
recommendation, S2 checks only this node's own status:

- `is_own_admin(org_id) -> Result<AdminStatus, _>`, where org-io supplies its
  own account (from the key it holds) and its configured co-signers. It reads
  the proxy account and its delegates and answers admin when one delegate
  equals the own account (no co-signers) or `multi_account_id({own} ∪
  co_signers, 1)`. A slot with no mapped account or a proxy with no
  delegates is "not an admin", and a read that fails is an error, never "not
  an admin". The pure part (given the delegates and the set, decide) is the
  `signatory_rule` test's subject. The storage reads are new on-chain-client
  reader functions, each a REQ with LLRs in that unit, exported for org-io.
- Nothing in S2 acts on the answer: owner ruling of 2026-10-07, the check is
  org-io's function with its tests only, and the app is not wired to it in
  S2. Its first caller is S3b-io (stage S3's org-io half). RC-wzb48r stays
  information for safety. Publishing the set (in the record or the contract)
  is what would let it become a control by design; that is for a later stage.

## 5. Dependency assessments ("Declaring a dependency")

The org-node and on-chain-client edges are declared in org-io's config with
the skeleton (§3), on the strength of these assessments.

- **org-io → org-node.** Uses: `OrgService` and its operations, `OrgState`
  (and its `from_chain` parse), `OrgId`, `ChainAccount`, `Epoch`, `RootHash`,
  `OrgPublicKey`, `ProvisionalUpdate`, `CommitOutcome`, `OrgNodeError`,
  `PersonaStore`. **Export surface: empty.** org-node has no `exported: yes`
  requirement, so in the develop phase org-node exports the requirements
  org-io relies on: provisional updates (REQ-xs4ab8), commit after verifying
  against the chain, the Organisation-state parse (REQ-8jb4ny), the
  absence-versus-failure requirement LLR-rm9x4z used to satisfy
  (REQ-bvh8v6), and from the key-pair change REQ-stx9v3 and REQ-jy6ybw. Until
  then org-io's ledgers name no org-node ID. RMF: org-node's HAZ-vxabf9 and
  HAZ-ep6uzs (the key-pair change amends both) name the sending path that
  org-io now drives. Problems open against org-node that the workflow
  touches: PR-vt244s (S3), PR-qmvj83 (S3), PR-kwwap5 (S4). SOUP: org-node's
  inventory (iroh, chacha20poly1305, argon2) is inherited.
- **org-io → on-chain-client.** Uses: the reader (`OrgRegistryClient`,
  `get_org_state`, `OrgAdmin`, `OrgState`), the writer (`genesis`,
  `submit_update`, `SubxtWriteOps`, `FinalitySink`, `AccountId`), and the two
  new storage reads. **Export surface:** REQ-6jefu2 and REQ-aat4yt (the
  writer) are exported. The reader has no exported REQ. REQ-ysyu9g
  (finalised-block read) is org-node's `expects: on-chain-client` item today
  and moves to org-io with the read (§6). Problems: PR-b795an (resolved
  here). SOUP: subxt, jsonrpsee, subxt-signer, blake2. There is no gap beyond
  the two new reads.
- **org-io → person.** Not declared in S2 unless the code names a person type.
  person exports 18 requirements.
- **app → org-io** (replacing app → org-node and app → on-chain-client). The
  export surface is org-io's exported requirements: REQ-nfr3n2 once it moves,
  and the drafted org-io requirements marked `exported: yes`. Every app item
  that names an org-node or on-chain-client behaviour is re-pointed (§6).
- **Class floor and segregation:** all class C. No `segregated_from:`.

## 6. Existing items S2 must move or amend

*Re-surveyed 2026-10-08 against S3a (`60c7d58`).* T5 has already moved
LLR-rm9x4z, REQ-ysyu9g and SDD-z85ux9 and rewritten org-node's mentions of
them (`grep -rnw` finds none left in `org-node` or `app`). The line numbers
in this section's org-node bullets are re-taken where a task below uses
them; S3a's items are under "S3a's items" at the end of the org-node list.
No item S3a minted moves to org-io in S2: S3a minted its decisions as
org-node's values-in functions (SDD-uck4tz, SDD-8cpyfa's additions), and
the org-io half it proposed is unminted (the eleven proposed requirements
of `docs/plans/2026-10-06-org-io-commit-workflow.md`, minted by S3b-io).
What S2 does take into org-io from S3a is an ordering — the sender rule,
`check_notice` and the acknowledgement check all come before any chain
read — and that is stated by amending org-io's own LLR-7pj5af (T7), not by
moving an S3a ID; the S3a items that said "before the chain read" are
amended in place (T6).

**Ruling A:** a moved item keeps its ID. Its definition (the whole block,
including its amendments) moves to an org-io ledger file of the same kind,
under a dated note naming the file and line it came from. The source file
keeps a dated pointer in prose that names org-io's ledger file but not the
ID: org-node does not depend on org-io (a bare ID would be
UNDECLARED-DEPENDENCY), and the app may cite only what org-io exports (an
LLR would be NON-EXPORTED-REF). Every other mention of a moved ID in the
source unit (ledgers, source comments, tests that stay) is rewritten the
same way. The moves happen in the develop phase, with the code. Nothing is
re-minted.

### Moves to org-io (ID kept)

| ID | From | Into org-io as | Notes |
|---|---|---|---|
| REQ-nfr3n2 | `app/docs/requirements/2026-10-06-invitation.md:72` | requirements | Subject becomes org-io: it submits the provisional update org-node built, and only once the submission has executed asks org-node to commit and send it. `satisfies: derived` and its assessment (`app/docs/risk/2026-10-06-invitation.md:78-86`) move with it, re-assessed for key custody now in org-io (that `assesses:` line at `:86` also names five app requirements; split it). Marked `exported: yes` so the app and its UI (`app/src/lib/api.ts:156`, `app/src/lib/components/Admit.svelte:6`) may still cite it. Named by PR-3ue4va (`app/docs/problems/2026-10-07-update-fan-out.md:9`), PR-924ftr and the first-admission problem file, which stay app problems and cite an exported ID. Two app LLRs satisfy it today and must be re-parented, because an app LLR does not decompose another unit's requirement: LLR-q225ws (`app/docs/architecture/2026-10-07-org-key-pair.md:21`, `satisfies:` at `:32`) and LLR-gha5f6 (`app/docs/architecture/2026-10-06-invitation.md:146`, `satisfies:` at `:161`). Both go to the new app requirement REQ-m8sgjk (the residue: the app decides when to submit and which Device receives the committed update; minted 2026-10-07, written by task T9); SDD-rmbr3t's `traces:` (`app/docs/architecture/2026-10-05-decomposition.md:407`) swaps REQ-nfr3n2 for REQ-m8sgjk |
| LLR-qhjp6g | `app/docs/architecture/2026-10-06-invitation.md:192` | architecture, under the submission item SDD-erzj3m | Its five tests in `submit_flow.rs` move with it. Restated on arrival for org-io's signatures (`OrgIo::found_organisation`, `OrgIo::submit_commit_send`) and for the chain read org-io now makes between the write and the commit (LLR-9r2bxd) |
| LLR-be3zv9 | `app/docs/architecture/2026-10-06-invitation.md:207`, note `:218-221` | architecture, under the submission item SDD-erzj3m | The note's "the bound is the app's" becomes org-io's. `app/src-tauri/Cargo.toml:49` names it in a comment: rewrite |
| REQ-ysyu9g | `org-node/docs/requirements/2026-09-06-dependency-expectations.md:50` | requirements | `expects: on-chain-client`, `opened: 2026-09-06` kept, so its age and the due date 2026-12-05 are unchanged. on-chain-client's ledgers cite it as an expectation addressed to them (`on-chain-client/docs/requirements/2026-09-10-chain-reading.md:220`, `on-chain-client/docs/risk/2026-09-10-on-chain-client-hazards.md:468`, `:1578`, `:1722`, `:2210`), which stays valid because org-io depends on on-chain-client. Its "what org-node owes in return" paragraph is restated for org-io. Its assessment (`org-node/docs/risk/2026-09-06-dependency-expectations.md:31`) moves too. org-node's hazard file cites it as HAZ-tawvm2's decisive input eight times (`org-node/docs/risk/2026-09-09-org-node-hazards.md:69`, `:111`, `:115`, `:247`, `:274`, `:942`, `:998`, `:1045`, `:1125`, `:1325`): rewritten to prose naming org-io's ledger |
| LLR-rm9x4z | `org-node/docs/architecture/2026-10-03-decomposition.md:630` | architecture, under the chain-read item SDD-6qz9ms | Amended on arrival: re-parented from org-node's REQ-bvh8v6 to org-io's chain-read requirement REQ-tg9zrn, and its cache exception retired with the cache. It states that org-io's read returns `None` for an Organisation with no slot, `Some` for one with state, and an error, never `None`, for a failed read or a refused parse. `org-node/tests/verify_against_chain.rs:107` and `:139` annotate it: the absent-slot case stays in org-node annotated `REQ-bvh8v6, LLR-8m99q2`; the failed-read case (`FailingChain`) moves to org-io's `chain_read_rule` |
| SDD-z85ux9 | `org-node/docs/architecture/2026-10-03-decomposition.md:2476` (section from `:2451`), deviation `:2500-2560` | architecture, as the chain-connection item | Its `traces:` (today org-node's REQ-nhe2zu, REQ-txvtm9, REQ-bvh8v6) are re-pointed to org-io's REQ-tg9zrn (chain read), REQ-nfr3n2 (submission) and REQ-f3eu9n (own-admin check); its code list becomes org-io's `connect.rs`, `OnChainStateReader`, `OnChainWriter`, the two signatory-set fetches and the preflight; its deviation and size measurement move and are re-measured. org-node keeps an absence, stated by the new org-node LLR-mn5c2q ("org-node names no `on_chain_client` and no `subxt`", tested in `org-node/tests/absences.rs`; minted 2026-10-07, written by task T6). Mentions in `org-node/docs/architecture/README.md:85`, `:88`, `:105`, `:110`, `:144`, `:173`, `2026-10-04-type-safety.md:31`, `docs/risk/2026-10-03-architecture-derived.md:722` and the decomposition's `:20`, `:63`, `:606`, `:1012`, `:1017`, `:2667`, `:2786`, `:2810` are rewritten |

### Amended in place (stay in their unit)

**app**

- The residue of REQ-nfr3n2 is the new app requirement REQ-m8sgjk (minted
  2026-10-07, written by task T9): the app decides when to found an
  Organisation or submit a provisional update, and which Device receives the
  committed update, hands both to org-io, and reports the outcome org-io
  returns. The first draft hoped to need none; the key-pair change made one
  necessary, because LLR-q225ws (recipient choice) and LLR-gha5f6 (admitting a
  reply) both satisfy REQ-nfr3n2 today and stay in the app with the code they
  describe. It is `satisfies: derived` and is assessed in the app's risk draft
  of this change.
- LLR-q225ws, `app/docs/architecture/2026-10-07-org-key-pair.md:21`
  (`satisfies:` `:32`), and LLR-gha5f6,
  `app/docs/architecture/2026-10-06-invitation.md:146` (`satisfies:` `:161`,
  and its text "submits it as LLR-qhjp6g states" at `:156`). Re-parent to
  REQ-m8sgjk; LLR-gha5f6 keeps REQ-65xqp8. Rewrite the LLR-qhjp6g mention to
  prose naming org-io's ledger (an app LLR may cite only exported org-io
  items). The file intro at `app/docs/architecture/2026-10-07-org-key-pair.md:4`
  ("It refines REQ-nfr3n2") is restated.
- The intro of `app/docs/architecture/2026-10-06-invitation.md:3-6` (it lists
  REQ-nfr3n2 among the requirements the file refines) and the code paragraph
  at `:22-28` (says `submit.rs` and the production chain writer are
  SDD-6g3wnh's). Amend both.
- SDD-rmbr3t, `app/docs/architecture/2026-10-05-decomposition.md:403`
  (`traces:` `:407`), whose 2026-10-06 amendment at `:409-420` gives the item
  `submit.rs` and lists LLR-qhjp6g and LLR-be3zv9 among its LLRs. Amend:
  `submit.rs` leaves; the commands call org-io's handle; `traces:` swaps
  REQ-nfr3n2 for REQ-m8sgjk; the two moved LLRs are named in prose.
- SDD-6g3wnh, `app/docs/architecture/2026-10-05-decomposition.md:927`, whose
  code list at `:937-939`, setup at `:954`, environment reasons at `:985-987`,
  harness note at `:1001-1010` and inventory rows `:1023-1025` name
  `build_chain_ops`, `connect_chain`, `ChainNotConfigured` and `OnChainWriter`.
  Amend: these leave for org-io's chain-connection item SDD-z85ux9 (moved).
- SDD-aq7m6b, `app/docs/architecture/2026-10-05-decomposition.md:110`. Amend:
  it wires one org-io handle, not `OrgService` plus a chain implementation plus
  a writer.
- SDD-k95rmp, `app/docs/architecture/2026-10-05-decomposition.md:44`. Amend:
  the app is the only reader of the environment except for `ODS_ADMIN_SEED`,
  which org-io reads under `dev-seed` only (accepted recommendation). The
  tests-race reason for SDD-k95rmp's rule holds for org-io's reader too:
  org-io's tests exercise the parser on values; the one test that sets the
  variable runs alone in its own test binary.
- REQ-e4ah9h and REQ-bvx4nh, `app/docs/requirements/2026-09-14-tauri-shell.md:49`
  and `:54`. Check the text: the endpoint reported must still be the one the
  chain was built from: the app records the values it handed
  `OrgIo::connect`, and only when that call succeeded, as `build_chain_ops`
  returns them today. Under the owner ruling of 2026-10-07 on "Still open"
  item 1 (the read stays coupled to the seed in S2, split in S8) the rule
  "no endpoint ⇔ not configured" is unchanged.
- REQ-x3c8n2, `app/docs/requirements/2026-09-14-tauri-shell.md:391`
  (`expects: org-node`, opened 2026-09-14). The app no longer depends on
  org-node, so `expects: org-node` would be UNDECLARED-DEPENDENCY. Re-address
  it to org-io (`expects: org-io`), keeping its ID and `opened:` date (due
  2026-12-13), with a dated note; org-io answers it when S4 moves the transport
  and the receive loop. This settles the first draft's "decide when the edge
  changes".
- RC-wzb48r, `app/docs/risk/2026-10-06-invitation.md:27`. Its rationale at
  `:33-37` says "the multisig signatory read is planned for a later change".
  Amend: org-io now checks the node's own admin status only (§4); the
  inviter's status cannot be checked without its signatory set, so the
  control stays information for safety (accepted recommendation).
- The note "The admin signing key travels in the process environment",
  `app/docs/risk/2026-09-14-app-hazards.md:989-994`. Amend: the key is the
  user's own signatory key; the variable is development-only (owner,
  2026-10-06) and release builds of org-io contain no code that reads it
  (REQ-8zuka3); production custody is the roadmap's stage S8.
- PR-5mc4d8, `app/docs/problems/2026-09-14-app-problems.md:200` (its startup
  bullets `:258` `ODS_ADMIN_SEED` and `:262` `ODS_COSIGNER_PUB`; `:248`
  `ODS_CONTRACT_H160` stays the app's). Add a dated note that the seed and
  co-signer parsing moved to org-io under REQ-qnh9pb's single-strip rule,
  which fixes those two sites; the report stays open for the remaining sites
  (`ODS_CONTRACT_H160`, `peer_addr_blob`).
- PR-924ftr, `app/docs/problems/2026-10-06-provisional-housekeeping.md:3`
  (`submit.rs` at `:5`, `affects:` `:11` names REQ-nfr3n2, LLR-qhjp6g,
  LLR-be3zv9). Re-point the location of `submit_commit_send` to org-io; the
  two LLRs leave its `affects:` line and are named in prose (it is still
  resolved in S3; REQ-nfr3n2 stays, exported).
- `app/docs/problems/2026-10-06-first-admission.md:8` and
  `app/docs/problems/2026-10-07-update-fan-out.md:9` (`affects:` REQ-nfr3n2),
  `app/docs/architecture/soup.md:108` and `app/.guardrails/config.yaml:106`
  name REQ-nfr3n2: valid once org-io exports it; re-check after the move.
- Source mentions of moved IDs: `app/src-tauri/src/state.rs:56`,
  `app/src-tauri/src/invitation.rs:344` (LLR-qhjp6g, not exported: rewrite),
  `app/src-tauri/src/commands.rs:113`, `:207`, `:286`. `submit.rs` itself is
  deleted.
- SOUP, `app/docs/architecture/soup.md:98` (`subxt`), `:99` (`subxt-signer`),
  `:103-113` (`on-chain-client` with `write`, `org-node`, `org-members` dev),
  and the version-skew note `:154-158`. Remove these, add `org-io`, and add the
  chain crates to org-io's inventory. The app keeps its own lock
  (`app/src-tauri` is its own workspace), so org-io's dependencies resolve
  through the app lock when the app builds; record that as the skew note does
  for on-chain-client today.
- `app/.guardrails/config.yaml:187-189` `depends_on:`: org-node and
  on-chain-client become org-io. `app/src-tauri/Cargo.toml:25` (`async-trait`,
  only if no app code needs it after the move), `:29-33` (`subxt`,
  `subxt-signer`, `on-chain-client`, `org-node`) and `:52-57` (the dev
  dependency `org-members`, which org-io re-exports as `OrgMembersError`) go;
  `org-io` arrives.

**org-node**

- SDD-ueh4tm, `org-node/docs/architecture/2026-10-03-decomposition.md:1105`
  (and its 2026-10-05 amendment). Amend (ruling B): the boundary between the
  user stories and the chain is no longer a trait but the state value each
  chain-judging operation takes; the stories are exercised in full without a
  chain by passing values from `test_fixtures::ChainSlots`.
- LLR-65py3d (`:1120`). Amend (ruling B): org-node presents no operation
  that reads or writes the chain; every operation that judges against the
  chain takes the Organisation's state as an argument and is synchronous.
  Its tests (`service_lifecycle.rs`, `absences.rs`) are rewritten to match,
  with the annotation kept. The doc comment at `org-node/src/service.rs:38-39`
  goes with the trait.
- LLR-hg3xzf (`:1126`, clones of `MockChainOps` share one chain state) and
  LLR-ryzr8m (`:1130`, `MockChainOps::apply_update`'s compare-and-swap):
  amend to `test_fixtures::ChainSlots`, the value store that replaces the
  mock (its clones share one set of slots; `apply_update` keeps the
  compare-and-swap). Their tests are re-pointed.
- SDD-pa6p7w, `org-node/docs/architecture/2026-10-03-decomposition.md:602`
  (and its amendments that follow). Amend: the `ChainReader` view is replaced
  by the state value; `OrgState::from_chain` stays; `org_state_from_chain`
  leaves for org-io; `OrgStateCache` and `OnChainReader` are deleted.
  LLR-rm9x4z leaves (moves, above).
- LLR-8m99q2 (`:569`). Amend: "an Organisation for which the chain reports no
  state" becomes "verification given no chain state (`None`)"; the clause "a
  chain read that failed, which is refused with `Chain`" leaves org-node,
  because org-node no longer sees a read; that distinction is LLR-rm9x4z's in
  org-io. Its absent-state test stays (`verify_against_chain.rs:107`); its
  failed-read test (`:142`, moved in substance to org-io's `chain_read_rule`
  by T5) is deleted by T6.
- The receive-path LLRs that name `ChainOps::read_state`: LLR-9ew26y
  (`org-node/docs/architecture/2026-10-06-chain-authority.md:442`), LLR-s8xp7m
  (`:451`), LLR-379hnv (`org-node/docs/architecture/2026-10-03-decomposition.md:2309`),
  and the key-pair change's LLR-xn5pwc, LLR-ba2ejp, LLR-38e2kn and LLR-pt32fx
  (`org-node/docs/architecture/2026-10-07-org-key-pair.md:275`, `:284`,
  `:296`, `:311`; LLR-38e2kn and LLR-pt32fx as S3a amended them).
  LLR-xn5pwc's `receive_one` becomes `receive_message`. Amend each "with no `read_state` call" to "refused by the
  chain-free phase (`prepare_*`), which hands org-io no Organisation to read",
  and "the state read by their one `read_state` call" to "the state the apply
  phase is given". The ordering guarantee itself (no read before the
  chain-free checks pass, at most one read) moves up to org-io as LLR-7pj5af.
  The design notes at `2026-10-06-chain-authority.md:64-70` (the "No chain
  write" bullet: "`ChainOps` keeps `read_state` only") get a dated
  supersession note.
- LLR-mmdu38, `org-node/docs/architecture/2026-10-04-type-safety.md:185`.
  Amend: the parse and `Debug` clauses stay; the fail-closed cache clause is
  retired with a dated note, because `OrgStateCache` is deleted. Its mentions
  in `org-node/Cargo.toml` (the `chain_read_state` comment), and the two
  problem files are re-checked. Its two parse tests in
  `org-node/tests/chain_read_state.rs:36`, `:50` move to org-io's
  `chain_read_rule` with `org_state_from_chain` and are re-annotated there
  `verifies: LLR-rm9x4z` (org-io may not name org-node's unexported items).
  org-node keeps other evidence for every ID they carried: LLR-s7whrn
  (`node_value_types.rs:201`, `:214`, `:230`), LLR-3jjgtw and LLR-mmdu38
  (`node_value_types.rs:157`, `:174`, `:184`), REQ-8jb4ny
  (`organisation_key.rs:349`, `:374`; the receive against an invalid key at
  `:392` moves to org-io's `receive_order` in T7, because the refused parse
  is now org-io's read). The cache test (`:64`) is deleted with the cache.
- PR-k2xxaq, `org-node/docs/problems/2026-10-05-stale-cache-on-fetch-error.md:3`
  (affects LLR-mmdu38's cache clause). Resolved by deletion of
  `OnChainReader` and `OrgStateCache` (accepted recommendation, 2026-10-06);
  the absence is verified by org-node's `absences.rs` (LLR-mn5c2q
  below). PR-hvg2dy's text (`org-node/docs/problems/2026-09-09-org-node-problems.md:21`)
  and `:77`, `:105` mention the reader: re-check.
- The owner-context bullet at
  `org-node/docs/requirements/2026-10-06-chain-authority.md:93` ("keeps
  reading the chain through on-chain-client") and sequencing item 4 at `:238`
  ("the advisory signatory read"). Add a dated supersession note pointing to
  the roadmap and the ADR.
- Mentions of REQ-ysyu9g left in org-node (`README.md:63`,
  `docs/architecture/README.md:78`, `docs/architecture/soup.md`,
  `docs/requirements/2026-09-09-verify-and-commit.md:154`,
  `docs/requirements/2026-10-03-transport-binding-scope.md:166`,
  `docs/requirements/2026-09-06-dependency-expectations.md:32`, `:80`,
  `docs/architecture/2026-10-03-decomposition.md:16`, the hazards file (listed
  with the move, above), `docs/problems/2026-09-09-org-node-problems.md:105`,
  `src/chain_read.rs:75` (deleted), `.guardrails/config.yaml:283`) are
  rewritten to prose naming org-io's ledger.
- `org-node/.guardrails/config.yaml:244` `verify_commands`: drop
  `--test chain_read_state`. Rewrite the comment at `:48` (the excluded
  `preflight` target): it stays excluded, now because its networked check
  depends on relay reachability, not because it spawns chopsticks; the
  historical counts at `:127-236` stay as history. Drop `depends_on`
  on-chain-client (`:287`) and restate the comment above the list (`:279-284`).
- `org-node/Cargo.toml`: the `chain` feature (`:14`), `app`'s `"chain"`
  (`:18`), the optional `subxt`, `on-chain-client`, `async-trait` and `hex`
  dependencies (`:45`, `:47-49`; `tokio` `:46` stays for `transport`), the
  chopsticks-only dev dependencies (`:70-78`, except `serde_json` `:74`, which
  `tests/encoding_golden.rs` uses), and the `[[test]]` entry
  `chain_read_state`; the `preflight` `[[bin]]` (`:86-88`) and `[[test]]`
  (`:125-128`) stay, gated on `transport` instead of `app`.
- `org-node/docs/architecture/soup.md:91-98`, `:106`, `:140-145`, `:153`,
  `:173-181` (the subxt, subxt-signer, on-chain-client, async-trait and
  chopsticks rows and notes): remove them or restate them as org-io's.
  `org-node/AGENTS.md:14` ("chain reads (`chain_read.rs`)"): amend.
- `org-node/README.md:63` (finalised read) and the module doc of
  `org-node/src/service.rs:1-4` ("composes store + chain + transport"; "`app`
  implies `chain`"): amend.
- LLR-ewkg85, `org-node/docs/architecture/2026-10-06-chain-authority.md:511`
  ("a failed or empty chain read"). Amend: "no chain state given (`None`)";
  a failed read never reaches org-node and is org-io's LLR-9r2bxd. Its
  failed-read tests (`commit_paths.rs:235`, and the failing half of `:399`)
  are deleted by T6; org-io's LLR-9r2bxd tests (T7) take their place.
- SDD-8cpyfa (`2026-10-03-decomposition.md:1656`, "read the chain once"):
  amend to "take the state org-io read once".

*S3a's items (re-surveyed 2026-10-08).* None moves (see the note at the
head of this section). Amended in place by T6, each with a dated note in the
file that defines it, because each names a chain read or `receive_one` that
org-node no longer makes:

- LLR-2r2fha (`org-node/docs/architecture/2026-10-07-commit-workflow.md:481`)
  and LLR-kzgjz8 (`:497`): "the sender `receive_one` returned" becomes "the
  sender passed to `prepare_receive`/`prepare_self_delete`"; "before
  `check_chain_free`, the chain read …" / "before the chain read and
  `revocation::accept`" becomes "in the chain-free phase, so the phase
  returns no pending value and no state is asked for". LLR-3aysup (`:505`):
  the check is made in the chain-free phase, which returns the outcome
  itself (`Prepared::Done`). LLR-dc45ur (`:349`): `receive_one` →
  `receive_message`. The section intro at `:472-479` ("Until S4 … org-node
  takes the sender from its own endpoint") gains: the sender is a value the
  chain-free phase takes, so S4 changes only who supplies it.
- LLR-3q63zv (`org-node/docs/architecture/2026-10-03-decomposition.md:2271`):
  "before it reads the chain" → "in `prepare_self_delete`".
- REQ-ztdza4 (`org-node/docs/requirements/2026-09-09-verify-and-commit.md:115`)
  and REQ-ea4qs5 (`org-node/docs/requirements/2026-10-07-commit-workflow.md:100`):
  "before it reads the chain" → "before it takes any chain state, so that
  none is read for it" (the read is org-io's; its order is stated in
  org-io's architecture ledger, named in prose). RC-u7kdam
  (`org-node/docs/risk/2026-10-07-commit-workflow.md:27`) gets the same
  dated note; its risk reasoning is unchanged.
- LLR-xgefn8 (`2026-10-07-commit-workflow.md:261`) needs no amendment: it
  forbids `ChainOps` and `ChainReader` in `revocation.rs` and `reconcile.rs`,
  which stays true once they no longer exist.
- The interim `OrgService::reconcile` (no LLR of its own; its tests verify
  LLR-gr8x3r and LLR-fm38ww) becomes synchronous and takes the state; its
  "no chain read for an unheld Organisation" becomes "`OrgNotHeld` whatever
  state it is given". The no-read order for the startup reconcile is
  S3b-io's (its T15), in org-io.

**on-chain-client**

- PR-b795an, `on-chain-client/docs/problems/2026-10-06-writer-coverage.md:3`.
  Resolve it (Makefile recipe, `on-chain-client/.guardrails/config.yaml:135`
  coverage comment, re-measured floors).
- `on-chain-client/docs/requirements/2026-10-06-chain-write.md:8` and
  `on-chain-client/docs/architecture/README.md:78` ("the app calls them";
  "org-node keeps reading the chain through this"): change to org-io.
- SDD-yg7n55, `on-chain-client/docs/architecture/2026-10-06-chain-write.md:50`.
  Check the consumer wording.
- New: the two signatory-set reads behind the own-admin check
  (`Revive.OriginalAccount`, `Proxy.Proxies`): REQ-8p2veg and
  REQ-v8jczx (exported), under the new item SDD-rxfu6h, with
  LLR-m3tjvp and LLR-a9bb7b on their pure decoders (minted
  2026-10-07, written by task T3). The fetch calls themselves join the
  subxt shell SDD-3b8zef, which carries no LLRs by its recorded deviation.
- Carried from T5 (2026-10-07): `on-chain-client/docs/requirements/2026-09-10-chain-reading.md:220`
  ("is org-node's expectation REQ-ysyu9g") and
  `on-chain-client/docs/risk/2026-09-10-on-chain-client-hazards.md:467`
  ("One requirement of org-node's") and `:1578` ("org-node holds one
  requirement addressed to this unit") still name org-node as its holder.
  Restate as org-io's, with a dated note (task T10). `:1722` and `:2210`
  name no holder and stay.

**org-io (its own drafted items, amended for S3a)**

- LLR-7pj5af (`org-io/docs/architecture/DRAFT-worktree-org-io-create-org-io.md:41`)
  was drafted before S3a and knows one message kind. T7 restates it for the
  three: org-io passes org-node the message and its sender; when the
  chain-free phase finishes (an acknowledgement, decided against the store
  alone), org-io returns that outcome with no read; otherwise it reads
  once for the Organisation the pending value names. Its `Test:` paragraph
  gains the acknowledgement, the unlisted sender and the refused parse.
- LLR-9r2bxd (`:105`): unchanged in text; T7 tests both of its operations
  (`found_organisation` and `submit_commit_send`), not only the first.

**repository**

- `.guardrails/units.yaml`: org-io and its header paragraph were added on
  this branch (§3). The header's sentence "the app's edges are then reduced
  to org-io alone" becomes a dated statement of the edge change (task T9).
- `docs/CONTEXT.md`: add the term *org-io*. "Admin" there should mean a
  signatory of the multisig controlling the pure proxy. (The key-pair change
  amended this file; re-read it before editing.)
- `docs/plans/2026-09-05-ratchet-setup.md`: org-io is born with coverage.
- `.github/workflows/rust.yml`: its `test` job (`:49-81`) lists each crate's
  cargo entries and its `coverage` jobs call `make coverage` (`:170-`); add
  org-io's lines and drop org-node's `chain_read_state` target there too.
- `docs/adr/2026-10-06-org-io-unit.md`: its "Sequencing" consequence names the
  key-pair change as unmerged; restate it as merged (`1f52c36`).

## 7. Rulings applied (2026-10-06)

The open decisions of the first draft are settled by the owner's rulings on
the S2–S4 design drafts:

| First draft's decision | Ruling | Applied in |
|---|---|---|
| 1. What the app sees of org-node | Accepted: org-io re-exports, transitionally; S4 narrows | §2 (the handle), ADR |
| 2. Who reads `ODS_ADMIN_SEED` | Accepted: org-io reads it | §1, §6 (SDD-k95rmp), REQ-3pxa8f |
| 3. Where the signatory set comes from | Accepted: own status only; RC-wzb48r stays a warning | §4, REQ-f3eu9n |
| 4. `OnChainReader` | Accepted: deleted, and `OrgStateCache` too if unused | §1, §6 (LLR-mmdu38, SDD-pa6p7w) |
| 5. Items that change unit | **Overruled by ruling A**: moved items keep their IDs | §6 |
| 6. org-io's coverage | Accepted: coverage and branch floors from the first merge | §3 |
| 7. When the preflight moves | Accepted: S2, so org-node's `chain` feature goes | §1 |
| (seam) `ChainOps` kept in org-node | **Overruled by ruling B**: values in, values out | §1, §2, §6 (SDD-ueh4tm, LLR-65py3d) |
| PR-b795an | Accepted: fixed by measuring the `write` feature | §3 |
| org-io's class | Accepted: C | §3, config |
| Risk matrix | Ruling D: S3 is unacceptable at every probability | org-io's risk README and draft |

### org-io's first requirements and risk (drafted 2026-10-06)

`org-io/docs/requirements/DRAFT-worktree-org-io-create-org-io.md`:

- REQ-tg9zrn: read the chain (finalised), parse through org-node, hand
  org-node the state or its absence as a value; a failure is an error, never
  absence. Derived.
- REQ-f3eu9n: the own-admin check (`Revive.OriginalAccount` →
  `Proxy.Proxies` against the own account or `multi_account_id({own} ∪
  co-signers, 1)`). Derived, exported. In S2 org-io's function and its tests
  only; the app does not call it (owner ruling of 2026-10-07; first caller
  S3b-io).
- REQ-3pxa8f: org-io is the only holder of the user's signatory key (not
  the Organisation's X25519 key pair). Implements RC-2xufsr, exported.
- REQ-v4tfap: the user's sr25519 signatory private key is never exposed
  (no output, error, log, `Debug`/`Display`, IPC to the app, file or
  environment beyond the dev-seed read), and org-io never receives, holds or
  outputs a device ed25519 private key (the store is opaque sealed bytes).
  Owner ruling 2026-10-06. Implements RC-2xufsr, exported.
- REQ-8zuka3: `ODS_ADMIN_SEED` is read only under the `dev-seed` Cargo
  feature, which no release build enables; without it, the variable is
  ignored and the chain reported as not configured (reworded 2026-10-07: the
  read stays coupled to the seed in S2, owner ruling of 2026-10-07, and is
  split from the write in S8). Implements
  RC-2xufsr, derived. Chosen over `debug_assertions` because running the
  suite with and without the feature, plus a source scan, verifies it.
- REQ-qnh9pb: total seed and co-signer parse, single `0x` strip, errors carry
  no part of the value. Implements RC-2xufsr, derived.
- REQ-6fk4qj: the app reaches org-node, the chain and the key only through
  org-io. Derived, exported.

`org-io/docs/risk/DRAFT-worktree-org-io-create-org-io.md`: HAZ-gaqnc2 (the
user's signatory key exposed through an org-io output, letting an attacker
act alone, at threshold 1, as that signatory for every Organisation the user
administers; S3/P2) and RC-2xufsr (custody
by design), with an assessment for each derived requirement. The moved items
of §6 are not re-minted.

**Production custody** (prompt the user to keep the signatory key in the OS
keychain, or use a hardware signing device; never stored in plaintext) is not
an S2 item. It is the roadmap's stage S8, unminted. A requirement with no test
would be MISSING-TEST and block S2's merge, and a problem report is for a
defect and would go stale on planned work; S8 mints its requirements when it
starts. Until then a release build signs no chain write: for this PoC that is
the owner's ruling (the seed is for development only), not a behaviour
change to hide. Nor does a release build read the chain until S8, because
the read stays coupled to the seed (owner ruling of 2026-10-07, "Still open"
item 1 below); S8 splits the read from the write.

### Key non-exposure (owner ruling, 2026-10-06)

"Neither a user's on-chain private signing key (sr25519) nor any of their
device private signing keys (ed25519) should ever be exposed." For org-io
this is REQ-v4tfap. HAZ-gaqnc2's residual (S3/P1 after RC-2xufsr) is
therefore not put to acceptance: an exposure route found later is a defect,
recorded as a problem report. The same ruling binds org-node and person for
device ed25519 keys; their existing items (for example org-node's problem
report on a derived `Debug` over a secret, and its secret-redaction tests)
are where it is checked, and S2 does not change them.

### Still open (for the owner) — ruled 2026-10-07

*Both items below were ruled by the owner on 2026-10-07, each as
recommended; the tasks were already written to the recommendations, so no
task changes beyond the notes added to T7, T8 and T9. The heading is kept so
that references to "Still open" item 1 and item 2 still resolve.*

The first draft's two develop-phase choices are settled by the 2026-10-07
reconciliation: `ChainWriter` stays a trait object inside org-io, beside a
`StateReader` trait for the read (both org-io's own test seams; ruling B
binds org-node's interface, not org-io's), and REQ-x3c8n2 is re-addressed to
org-io (§6, app).

The reconciliation against the merged code found two questions the rulings
do not settle. The tasks are written to the recommendation of each, and the
step that changes if the owner rules otherwise is named.

1. **Does a build without `dev-seed` still read the chain?** Today the app
   builds the read and the write together and only when `ODS_CHAIN_WS`,
   `ODS_CONTRACT_H160` and `ODS_ADMIN_SEED` are all set; otherwise both are
   "not configured" and the connection status shows no endpoint (REQ-e4ah9h,
   REQ-bvx4nh). A release build (no `dev-seed`) can never hold the seed, so
   under today's rule it can never read the chain either, and so can never
   receive a membership update. The alternative, a read configured by the two
   non-secret values alone, is a behaviour change (a node with no seed starts
   reading the chain and receiving) and changes what the connection status
   means. **Recommendation: keep today's rule in S2** (no behaviour change:
   with `dev-seed` everything is as today; without it the whole chain is "not
   configured", with a message that signing and the chain are available only
   in development builds until S8), and split the read from the write in S8,
   when a release build first has a key. If the owner rules for the split now,
   T7's `OrgIo::connect` builds the reader from the two values alone and T9
   changes `connection_status` (and REQ-e4ah9h's wording) accordingly.
   **Owner ruling of 2026-10-07: accepted — keep the read coupled to the seed
   in S2, split it in S8.** In S2 the read and the write are built together,
   only under `dev-seed` with all three values set; a release build therefore
   cannot read the chain (and so receives no membership update) until S8
   splits the read from the write. This is an availability limit the owner
   accepted for the PoC, not a residual left open; the roadmap's S8 row
   carries the split. T7 and T9 stand as written.
2. **Does anything call the own-admin check in S2?** REQ-f3eu9n says org-io
   "shall report" the node's own admin status, and §4 said nothing acts on it
   "beyond what the app shows" — but the app shows nothing about admin status
   today, so displaying it would add an IPC command and a UI element: a
   behaviour change in a stage that promises none. **Recommendation: no app
   wiring in S2.** org-io exposes `OrgIo::is_own_admin` and tests its rule
   (T8); its first caller is S3's workflow. If the owner wants it shown now,
   T9 adds an `own_admin_status` command, its IPC test and a requirement in
   the app.
   **Owner ruling of 2026-10-07: accepted — implemented in org-io in S2, not
   wired into the app in S2.** S2 delivers `OrgIo::is_own_admin` and its
   tests (T8) only; T9 adds no `own_admin_status` command, no IPC test and no
   app requirement. The first caller is S3b-io (stage S3's org-io half, after
   S2 merges), which wires it; the roadmap's S3 row records that.

## Tasks

*Written 2026-10-07 with the `plan-change` skill, after the reconciliation
above. Execute with `develop-change`, one task per subagent, in the order
below; a task marked **Parallel: yes** may run in its own task worktree
(`.guardrails/scripts/task-worktree.sh start <tag>`) beside the tasks named
with it, because their file sets do not intersect.*

### Plan header

**Goal:** Create the org-io unit and move into it the chain read, the chain
write's submission, the custody of the user's signatory key and a new
own-admin check, with org-node taking and returning values only and the app
depending on org-io alone, and no change in behaviour.
**Implements:** REQ-tg9zrn, REQ-f3eu9n, REQ-3pxa8f, REQ-v4tfap, REQ-8zuka3,
REQ-qnh9pb, REQ-6fk4qj, RC-2xufsr (org-io, drafted; REQ-f3eu9n as org-io's
function and its tests only — the app is not wired to it in S2, owner ruling
of 2026-10-07, and its first caller is S3b-io); SDD-6qz9ms (LLR-7pj5af),
SDD-erzj3m (LLR-9r2bxd), SDD-789u6d (LLR-4ax2m6, LLR-9fy622, LLR-gc6kwy,
LLR-c4bktx, LLR-rgdx22, LLR-3zdw8v), SDD-f4khqn (LLR-qhyc3n, LLR-vyd5d3,
LLR-c9fyun), SDD-z3ychz (LLR-n3zmt6, LLR-u2pk5y) (org-io, drafted in
`org-io/docs/architecture/DRAFT-worktree-org-io-create-org-io.md`); moved
with their IDs (ruling A): REQ-nfr3n2, LLR-qhjp6g, LLR-be3zv9 (from the app),
REQ-ysyu9g, LLR-rm9x4z, SDD-z85ux9 (from org-node); new in other units:
REQ-8p2veg, REQ-v8jczx, SDD-rxfu6h, LLR-m3tjvp, LLR-a9bb7b
(on-chain-client, T3), REQ-m8sgjk (app, T9), LLR-mn5c2q
(org-node, T6), REQ-uk9rw7 (org-node, T10; supersedes REQ-ztdza4); amended
in place: the items of §6; resolves PR-b795an (on-chain-client, T4),
PR-k2xxaq (org-node, T6) and PR-zf924s (org-node, T10; merge message:
`Resolves: PR-zf924s`); narrows PR-5mc4d8 (app, T9).
**Safety class:** C in every unit touched (org-io, org-node, app,
on-chain-client); no per-item override.
**Verification:** each unit's `verify_commands` after this change:
- org-io (set by T1, extended by each org-io task, final form in T12):
  `cargo test -p org-io --features test-support --lib --test key_custody_config --test fuzz_seed_parse --test signatory_key --test absences --test chain_read_rule --test receive_order --test submit_flow --test signatory_rule --test app_boundary`;
  `cargo test -p org-io --features test-support,dev-seed --test key_custody_config --test signatory_key --test dev_seed_env`;
  `cargo build -p org-io`;
  `cargo clippy -p org-io --all-targets --features test-support,dev-seed -- -D warnings`.
- org-node: today's `cargo test -p org-node --features app,test-support …`
  line without `--test chain_read_state`, plus the five `quint` lines,
  unchanged.
- app: today's three lines, unchanged.
- on-chain-client: today's two lines, the second extended with
  `--test signatory_set_decode --test fuzz_signatory_set` (T3).
- Coverage: `make coverage-org-io`, `make coverage-branch-org-io` (T12),
  `make coverage-on-chain-client` (T4) and the branch twins already on master.
- Trace: `.guardrails/scripts/check-units.sh`; per unit
  `GR_CONFIG=<unit>/.guardrails/config.yaml .guardrails/scripts/check-trace.sh`
  and `… check-ids.sh --allow-draft-files`.

### Conventions for every task

- Work in this change worktree (or the task worktree the dispatcher gives
  you). Commit with plain `git commit`; **never** a `Co-Authored-By` line.
  One plain git command per shell call.
- Never edit a file with `sed`; use the editor. Rust test files especially
  (AGENTS.md, "Lessons learned").
- Red first: write the test, run it, paste the failure into the task's
  report, then implement, then paste the green run. A test that cannot be
  red at runtime (a new target that does not compile yet) is red at compile
  time; say so.
- Every test carries `// verifies: <IDs>` (or `/// verifies:`) on the line
  directly above `#[test]`/`#[tokio::test]`, naming the lowest level that
  exists (an LLR where one is listed).
- Clippy denies `unwrap_used`, `expect_used` and `panic` in library code
  (workspace lints). Tests open with
  `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.
- Names are concrete; no single-character names (AGENTS.md).
- The item texts this plan quotes for a ledger file (T3, T6, T7, T9) are
  indented two spaces, because `check-ids.sh` reads a `**ID**:` line at
  column one as a definition even inside a code fence, and the quote would
  duplicate the ledger's own. Drop the indent when copying the text into its
  ledger file (2026-10-07, found at T3).
- Trace checkpoints: `check-trace.sh` is expected to report the moved items
  as dangling or missing between T5 and T9 (a test and its item move in
  different tasks); it must be clean for org-node after T5 and after T6,
  for every unit after T9 — org-io included, since T8 gave the own-admin
  items their tests (T9 step 8) — and for every unit after T12. "Clean"
  means exit 0; advisories (UNMET-EXPECTATION, UNRESOLVED-PR) may remain.
  (Corrected 2026-10-08: this line excepted org-io's own-admin items after
  T9, which T9's own step 8 does not.) Each task's own gate is its unit's
  `verify_commands`.
- Amending an existing item (owner ruling on PR-zf924s, 2026-10-08,
  "hybrid"): a change that narrows or clarifies an item without changing
  its meaning is a dated in-place note quoting the old text; a change of
  meaning mints a NEW superseding ID (`supersedes:` / `superseded-by:`),
  and the tests carry both IDs (a dual-ID `verifies:`, since check-trace
  has no MISSING-TEST exemption for a superseded item; a test that is
  already green gains the new ID only through a new red-first test or a
  dated N/A note, per the robustness rule). Every task that amends items (T6, T7, T10) classifies each
  amendment and says which it is.
  (Added 2026-10-08, review round 1 finding-13.) This is how the guardrails
  checklist item "tests of a superseded item name both IDs" is met here:
  the older, already-green tests keep only the old ID, because the
  robustness rule forbids re-annotating them; each superseding item has one
  new red-first test that names both IDs (REQ-uk9rw7 with REQ-ztdza4,
  LLR-tajh9d with LLR-js9dsu, LLR-mn5c2q with LLR-65py3d), and each
  superseded item carries a dated note listing the older tests that still
  name only it. The owner has reported the conflict between the two rules
  upstream to the guardrails project.
- Between T6 and T9 the app does not compile against the new org-node API.
  That is expected; T9 restores it. Do not patch the app before T9.

### T0 — Base: master with the branch-coverage change

**Files touched:** none by hand (a merge commit)
**Parallel:** no (first; every task depends on it)

1. Confirm `worktree-guardrails-branch-coverage` is merged:
   `git log --oneline -1 master -- Makefile` must show that change's squash
   commit, and `grep -n "define judge_branches" Makefile` on master must
   print a line. If it is not merged yet, stop and report: T4 and T12 need
   its Makefile.
2. `git merge-base --is-ancestor origin/master master` (local master is
   often ahead of origin; merge the local one — owner standing instruction).
3. `git merge master` in this worktree. Expected: a clean merge (this branch
   touches `docs/`, `org-io/` and `.guardrails/units.yaml`; the
   branch-coverage change touches `Makefile`, unit configs' coverage lines
   and `.github/workflows/rust.yml`). If `org-io/.guardrails/config.yaml`
   conflicts, keep this branch's file and add the `coverage-branch-org-io`
   line only in T12.
4. Run the baseline: `zsh` loop of `check-units.sh` and per-unit
   `check-trace.sh` / `check-ids.sh --allow-draft-files`. Expected: exit 0
   everywhere except org-io's `check-trace.sh` (exit 1, MISSING-TEST for the
   seven drafted REQs and the thirteen drafted LLRs — the truth before code).

### T1 — The org-io crate and the seed and co-signer parse

**Files touched:** Cargo.toml, Cargo.lock, org-io/Cargo.toml, org-io/src/lib.rs, org-io/src/custody.rs, org-io/tests/key_custody_config.rs, org-io/tests/fuzz_seed_parse/fuzz_target.rs, org-io/tests/README.md, org-io/.guardrails/config.yaml, org-io/docs/architecture/soup.md
**Parallel:** yes, with T3 (on-chain-client only); serial after T0
**IDs verified:** LLR-4ax2m6, LLR-9fy622, LLR-gc6kwy

1. Root `Cargo.toml`: add `"org-io",` to `members` after `"org-members",`,
   and reword the `exclude` comment's "used as a path dependency by
   org-node (behind the `chain` feature)" to "used as a path dependency by
   org-io". (org-node still names it until T6; the comment states the
   destination.)
2. Create `org-io/Cargo.toml`:

```toml
[package]
name = "org-io"
version = "0.1.0"
edition = "2021"
rust-version = "1.85"
license = "GPL-3.0-only"
description = "ODS: all IO for an Organisation — the chain read and write, the user's signatory key, the own-admin check"
publish = false

[features]
default = []
# The development-only signing seed (REQ-8zuka3): the one read of
# ODS_ADMIN_SEED is compiled only with this feature. No release build enables it.
dev-seed = []
# Test-only constructors and fakes (`test_support`); never in production builds.
test-support = ["org-node/test-support"]

[dependencies]
# Features mirror on-chain-client's pins so the subxt types unify (the reason
# org-node's manifest gave for the same line).
on-chain-client = { path = "../on-chain-client", default-features = false, features = ["dev-rpc", "write"] }
zeroize = "1.9"

[dev-dependencies]
bolero = "0.13"

[lints]
workspace = true

# The seed and co-signer parse never panics (LLR-4ax2m6, LLR-9fy622) and its
# errors carry no part of the value (LLR-gc6kwy).
[[test]]
name = "fuzz_seed_parse"
path = "tests/fuzz_seed_parse/fuzz_target.rs"
harness = false
```

   `org-node`, `subxt`, `subxt-signer`, `tokio`, `async-trait` and `hex`
   arrive with the tasks that first use them (T2, T5, T7), each with the
   same feature list as the manifest it comes from.
3. Create `org-io/src/lib.rs`:

```rust
//! org-io: all IO for an Organisation (docs/adr/2026-10-06-org-io-unit.md).
//! org-node takes values and returns values; org-io reads and writes the
//! chain, holds the user's own signatory key, and is the app's one way in.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod custody;
```

4. Write the failing test `org-io/tests/key_custody_config.rs` (it does not
   compile yet: compile-red):

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The development seed and co-signer parse (SDD-789u6d): total, single
//! `0x` strip, errors that name the variable and the rule and carry no part
//! of the value.

use org_io::custody::{parse_co_signer, parse_seed, ConfigError, ConfigRule, ConfigVariable};

const SEED_HEX: &str = "e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a";

/// True when `rendered` contains any four-character window of `input`.
fn leaks(rendered: &str, input: &str) -> bool {
    let body: Vec<char> = input.strip_prefix("0x").unwrap_or(input).chars().collect();
    body.windows(4).any(|window| rendered.contains(&window.iter().collect::<String>()))
}

fn assert_refused(result: Result<impl std::fmt::Debug, ConfigError>, input: &str, variable: ConfigVariable) -> ConfigRule {
    let error = result.unwrap_err();
    assert_eq!(error.variable, variable);
    assert!(!leaks(&error.to_string(), input), "Display leaks the input: {error}");
    assert!(!leaks(&format!("{error:?}"), input), "Debug leaks the input: {error:?}");
    error.rule
}

// verifies: LLR-4ax2m6
#[test]
fn a_seed_of_64_hex_characters_parses_with_or_without_one_prefix_and_in_either_case() {
    let plain = parse_seed(SEED_HEX).unwrap();
    let prefixed = parse_seed(&format!("0x{SEED_HEX}")).unwrap();
    let upper = parse_seed(&SEED_HEX.to_uppercase()).unwrap();
    assert!(plain.same_bytes_as(&prefixed));
    assert!(plain.same_bytes_as(&upper));
}

// verifies: LLR-4ax2m6, LLR-gc6kwy
#[test]
fn a_malformed_seed_is_refused_by_rule_and_the_error_carries_none_of_it() {
    let doubled = format!("0x0x{SEED_HEX}");
    let short = &SEED_HEX[..63];
    let long = format!("{SEED_HEX}a");
    let not_hex = format!("{}g", &SEED_HEX[..63]);
    let non_ascii = format!("{}é", &SEED_HEX[..62]);
    assert_eq!(assert_refused(parse_seed(""), "", ConfigVariable::AdminSeed), ConfigRule::Empty);
    assert_eq!(assert_refused(parse_seed("0x"), "0x", ConfigVariable::AdminSeed), ConfigRule::Empty);
    assert_eq!(assert_refused(parse_seed(short), short, ConfigVariable::AdminSeed), ConfigRule::WrongLength);
    assert_eq!(assert_refused(parse_seed(&long), &long, ConfigVariable::AdminSeed), ConfigRule::WrongLength);
    assert_eq!(assert_refused(parse_seed(&doubled), &doubled, ConfigVariable::AdminSeed), ConfigRule::WrongLength);
    assert_eq!(assert_refused(parse_seed(&not_hex), &not_hex, ConfigVariable::AdminSeed), ConfigRule::NotHex);
    assert_eq!(assert_refused(parse_seed(&non_ascii), &non_ascii, ConfigVariable::AdminSeed), ConfigRule::NotHex);
}

// verifies: LLR-9fy622
#[test]
fn a_co_signer_of_64_hex_characters_parses_to_its_account() {
    let account = parse_co_signer(&format!("0x{SEED_HEX}")).unwrap();
    assert_eq!(hex_of(&account.0), SEED_HEX);
}

// verifies: LLR-9fy622, LLR-gc6kwy
#[test]
fn a_malformed_co_signer_is_refused_by_rule_and_the_error_carries_none_of_it() {
    let doubled = format!("0x0x{SEED_HEX}");
    let odd = format!("{SEED_HEX}0");
    let not_hex = format!("{}z", &SEED_HEX[..63]);
    assert_eq!(assert_refused(parse_co_signer(""), "", ConfigVariable::CoSigner), ConfigRule::Empty);
    assert_eq!(assert_refused(parse_co_signer(&doubled), &doubled, ConfigVariable::CoSigner), ConfigRule::WrongLength);
    assert_eq!(assert_refused(parse_co_signer(&odd), &odd, ConfigVariable::CoSigner), ConfigRule::WrongLength);
    assert_eq!(assert_refused(parse_co_signer(&not_hex), &not_hex, ConfigVariable::CoSigner), ConfigRule::NotHex);
}

// verifies: LLR-gc6kwy
#[test]
fn each_refusal_names_its_variable_and_rule() {
    let seed_error = parse_seed("abc").unwrap_err().to_string();
    assert!(seed_error.starts_with("ODS_ADMIN_SEED"), "{seed_error}");
    assert!(seed_error.contains("64 hexadecimal characters"), "{seed_error}");
    let co_signer_error = parse_co_signer("abc").unwrap_err().to_string();
    assert!(co_signer_error.starts_with("ODS_COSIGNER_PUB"), "{co_signer_error}");
}

fn hex_of(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
```

   Run `cargo test -p org-io --test key_custody_config`. Expected:
   compile-red (`unresolved import org_io::custody::parse_seed` …).
5. Write the fuzz target `org-io/tests/fuzz_seed_parse/fuzz_target.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! verifies: LLR-4ax2m6, LLR-9fy622, LLR-gc6kwy
//! Neither parse panics on any input, and every refusal renders as one of
//! the fixed texts its (variable, rule) pair gives — so no rendering can
//! carry any part of the input. (A substring check would raise false alarms:
//! an input such as "xadecx" shares "adec" with "hexadecimal".)

use org_io::custody::{parse_co_signer, parse_seed, ConfigError, ConfigRule, ConfigVariable};

fn fixed_renderings() -> Vec<(String, String)> {
    let mut renderings = vec![];
    for variable in [ConfigVariable::AdminSeed, ConfigVariable::CoSigner] {
        for rule in [ConfigRule::Empty, ConfigRule::NotHex, ConfigRule::WrongLength, ConfigRule::NotAKey] {
            let error = ConfigError { variable, rule };
            renderings.push((error.to_string(), format!("{error:?}")));
        }
    }
    renderings
}

fn main() {
    let fixed = fixed_renderings();
    bolero::check!().with_type::<String>().for_each(|input: &String| {
        for refusal in [parse_seed(input).err(), parse_co_signer(input).err()].into_iter().flatten() {
            let rendered = (refusal.to_string(), format!("{refusal:?}"));
            assert!(fixed.contains(&rendered), "a refusal rendered outside the fixed set: {rendered:?}");
        }
    });
}
```

   Check against `org-node/tests/fuzz_envelope_decode/fuzz_target.rs` that
   the `verifies:` line's placement there (a `//!` line) is what
   `check-trace.sh` reads; copy that file's exact placement if it differs.
6. Implement `org-io/src/custody.rs` (the first half of SDD-789u6d; T2 adds
   the key):

```rust
//! Custody of the user's own sr25519 signatory key (SDD-789u6d): the
//! development configuration's parse. Errors name the variable and the rule
//! and carry no part of the value (LLR-gc6kwy).

use core::fmt;

use on_chain_client::write::AccountId;
use zeroize::Zeroize;

/// The environment variable a configuration value came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigVariable {
    AdminSeed,
    CoSigner,
}

impl ConfigVariable {
    fn name(self) -> &'static str {
        match self {
            ConfigVariable::AdminSeed => "ODS_ADMIN_SEED",
            ConfigVariable::CoSigner => "ODS_COSIGNER_PUB",
        }
    }
}

/// The rule a configuration value broke.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigRule {
    Empty,
    NotHex,
    WrongLength,
    NotAKey,
}

/// A refused configuration value: which variable, which rule. Holds nothing
/// taken from the value (LLR-gc6kwy).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConfigError {
    pub variable: ConfigVariable,
    pub rule: ConfigRule,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rule = match self.rule {
            ConfigRule::Empty => "is empty",
            ConfigRule::NotHex | ConfigRule::WrongLength => {
                "must be 64 hexadecimal characters after at most one leading 0x"
            }
            ConfigRule::NotAKey => "is not a valid sr25519 secret seed",
        };
        write!(formatter, "{} {rule}", self.variable.name())
    }
}

impl std::error::Error for ConfigError {}

/// The 32 seed bytes, wiped on drop; no public accessor (LLR-c4bktx).
pub struct SeedBytes([u8; 32]);

impl SeedBytes {
    /// Whether two seeds hold the same bytes, for tests; reveals nothing else.
    pub fn same_bytes_as(&self, other: &SeedBytes) -> bool {
        self.0 == other.0
    }

    pub(crate) fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Drop for SeedBytes {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl fmt::Debug for SeedBytes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SeedBytes(..)")
    }
}

/// 32 bytes from `raw`: at most one leading `0x`, then exactly 64 hex
/// characters. Rules, in order: empty body → `Empty`; a body that is not
/// 64 bytes long → `WrongLength` (a doubled `0x` leaves 66); a 64-byte body
/// with any byte outside `0-9a-fA-F` (a non-ASCII character included) →
/// `NotHex`.
fn parse_32_hex(raw: &str, variable: ConfigVariable) -> Result<[u8; 32], ConfigError> {
    let refuse = |rule| ConfigError { variable, rule };
    let body = raw.strip_prefix("0x").unwrap_or(raw).as_bytes();
    if body.is_empty() {
        return Err(refuse(ConfigRule::Empty));
    }
    if body.len() != 64 {
        return Err(refuse(ConfigRule::WrongLength));
    }
    let mut out = [0u8; 32];
    for (slot, pair) in out.iter_mut().zip(body.chunks_exact(2)) {
        let (Some(&high), Some(&low)) = (pair.first(), pair.get(1)) else {
            return Err(refuse(ConfigRule::WrongLength));
        };
        let high = hex_value(high).ok_or(refuse(ConfigRule::NotHex))?;
        let low = hex_value(low).ok_or(refuse(ConfigRule::NotHex))?;
        *slot = high << 4 | low;
    }
    Ok(out)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// The development signing seed (LLR-4ax2m6).
pub fn parse_seed(raw: &str) -> Result<SeedBytes, ConfigError> {
    parse_32_hex(raw, ConfigVariable::AdminSeed).map(SeedBytes)
}

/// One co-signer account (LLR-9fy622).
pub fn parse_co_signer(raw: &str) -> Result<AccountId, ConfigError> {
    parse_32_hex(raw, ConfigVariable::CoSigner).map(AccountId)
}
```

   The test's table follows from that order: `0x0x` + 64 is 66 bytes
   (`WrongLength`); 63 hex + `g` and 62 hex + `é` (two bytes in UTF-8) are
   64 bytes with a non-hex byte (`NotHex`). Working on bytes cannot split a
   character wrongly, because every non-ASCII byte is refused by
   `hex_value`.
7. Run `cargo test -p org-io --test key_custody_config` and
   `cargo test -p org-io --test fuzz_seed_parse`: green. Paste both.
8. Delete `org-io/tests/README.md` (the placeholder; the config comment
   says to delete it with the first test).
9. `org-io/.guardrails/config.yaml`: replace the commented-out
   `verify_commands` block with

```yaml
verify_commands:
  - cargo test -p org-io --lib --test key_custody_config --test fuzz_seed_parse
  - cargo build -p org-io
  - cargo clippy -p org-io --all-targets -- -D warnings
```

   and add `org-io/src` to `strict_paths` (keep `org-io/docs` and
   `org-io/AGENTS.md`); update the comments above both keys to say the crate
   exists since this task. Later tasks extend the first line.
10. `org-io/docs/architecture/soup.md`: replace the "Empty at the unit's
    creation" paragraph's tense (the crate exists) and add rows for
    `zeroize` (1.9.x, the version in `Cargo.lock`: wipes the seed bytes;
    LLR-c4bktx) and `bolero` (dev, 0.13.x: the fuzz target). Take exact
    versions from `grep -A1 'name = "zeroize"' Cargo.lock`.
11. Run org-io's three verify lines; all exit 0. Commit:
    `feat(org-io): the crate and the development seed and co-signer parse (LLR-4ax2m6, LLR-9fy622, LLR-gc6kwy)`.

### T2 — The signatory key holder, the `dev-seed` read, and the source scan

**Files touched:** org-io/Cargo.toml, Cargo.lock, org-io/src/custody.rs, org-io/tests/signatory_key.rs, org-io/tests/dev_seed_env.rs, org-io/tests/absences.rs, org-io/.guardrails/config.yaml, org-io/docs/architecture/soup.md
**Parallel:** no (serial, after T1)
**IDs verified:** LLR-c4bktx, LLR-rgdx22 (the environment read; T7 adds the `OrgIo::connect` half), LLR-3zdw8v

1. `org-io/Cargo.toml`: add to `[dependencies]`
   `subxt-signer = { version = "0.50", default-features = false, features = ["sr25519", "subxt"] }`
   (the app's line, `app/src-tauri/Cargo.toml:31`), and the targets:

```toml
[[test]]
name = "dev_seed_env"
path = "tests/dev_seed_env.rs"
required-features = ["dev-seed"]
```

2. Write `org-io/tests/signatory_key.rs` (compile-red):

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The signatory key holder (LLR-c4bktx): no getter for the secret, a
//! hand-written `Debug` that shows only the start of the public key.

use org_io::custody::{parse_seed, SignatoryKey};

const SEED_HEX: &str = "e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a";

fn windows_of(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    chars.windows(4).map(|window| window.iter().collect()).collect()
}

// verifies: LLR-c4bktx
#[test]
fn the_key_answers_its_account_and_renders_only_its_start() {
    let key = SignatoryKey::from_seed(parse_seed(SEED_HEX).unwrap()).unwrap();
    let account = key.account_id();
    let expected = subxt_signer::sr25519::Keypair::from_secret_key(
        <[u8; 32]>::try_from(hex_bytes(SEED_HEX)).unwrap(),
    )
    .unwrap()
    .public_key()
    .0;
    assert_eq!(account.0, expected);
    let rendered = format!("{key:?}");
    let start: String = expected[..4].iter().map(|byte| format!("{byte:02x}")).collect();
    assert_eq!(rendered, format!("SignatoryKey(account {start}..)"));
}

// verifies: LLR-c4bktx
#[test]
fn neither_the_seed_nor_the_key_renders_any_part_of_the_secret() {
    let seed = parse_seed(SEED_HEX).unwrap();
    assert_eq!(format!("{seed:?}"), "SeedBytes(..)");
    let key = SignatoryKey::from_seed(seed).unwrap();
    let rendered = format!("{key:?}");
    for window in windows_of(SEED_HEX) {
        assert!(!rendered.contains(&window), "Debug renders part of the seed: {rendered}");
    }
}

fn hex_bytes(text: &str) -> Vec<u8> {
    (0..text.len()).step_by(2).map(|index| u8::from_str_radix(&text[index..index + 2], 16).unwrap()).collect()
}
```

   Add `subxt-signer` (same line) to `[dev-dependencies]` so the test can
   name it. Run: compile-red (`no SignatoryKey in custody`).
3. Write `org-io/tests/dev_seed_env.rs` (one test in its own binary, so no
   other test races the environment — SDD-k95rmp's reason):

```rust
#![cfg(feature = "dev-seed")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The development seed path (LLR-rgdx22): the one read of ODS_ADMIN_SEED,
//! compiled only with `dev-seed`. The only test in this binary.

use org_io::custody::signatory_from_environment;

const SEED_HEX: &str = "e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a";

// verifies: LLR-rgdx22
#[test]
fn the_development_build_reads_the_seed_once_and_none_when_unset() {
    // SAFETY of set_var/remove_var: this binary runs this one test only.
    unsafe { std::env::set_var("ODS_ADMIN_SEED", SEED_HEX) };
    let key = signatory_from_environment().unwrap().unwrap();
    assert!(format!("{key:?}").starts_with("SignatoryKey(account "));
    unsafe { std::env::set_var("ODS_ADMIN_SEED", "0x0x00") };
    assert!(signatory_from_environment().is_err(), "a malformed seed is refused");
    unsafe { std::env::remove_var("ODS_ADMIN_SEED") };
    assert!(signatory_from_environment().unwrap().is_none());
}
```

   (Edition 2021: `set_var` is not `unsafe` yet; drop the `unsafe` blocks if
   the compiler warns `unused_unsafe`, keeping the comment.)
4. Write `org-io/tests/absences.rs` (runtime-red once `custody.rs` gains the
   names it scans for; write it now, run it after step 5):

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! What org-io must not contain (LLR-3zdw8v, LLR-u2pk5y): read from its own
//! source, in the style of org-node/tests/absences.rs.

use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            out.push(path);
        }
    }
}

fn sources() -> Vec<(PathBuf, String)> {
    let mut files = vec![];
    rust_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut files);
    files.into_iter().map(|path| { let text = std::fs::read_to_string(&path).unwrap(); (path, text) }).collect()
}

// verifies: LLR-3zdw8v
#[test]
fn the_seed_is_read_once_and_only_behind_dev_seed() {
    let mut reads = vec![];
    for (path, text) in sources() {
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            if line.contains("ODS_ADMIN_SEED") && line.contains("env::var") {
                reads.push(format!("{}:{}", path.display(), index + 1));
                // The read must sit inside `signatory_from_environment`: the
                // nearest `fn` line above it is that function's, and one of
                // the three lines above that is the dev-seed gate.
                let function_line = (0..index).rev().find(|&earlier| lines[earlier].contains("fn ")).unwrap();
                assert!(lines[function_line].contains("fn signatory_from_environment"),
                    "{}:{}: the seed is read outside signatory_from_environment", path.display(), index + 1);
                let gate_window = &lines[function_line.saturating_sub(3)..function_line];
                assert!(gate_window.iter().any(|attribute| attribute.contains("#[cfg(feature = \"dev-seed\")]")),
                    "{}:{}: signatory_from_environment is not behind dev-seed", path.display(), function_line + 1);
            }
        }
    }
    assert_eq!(reads.len(), 1, "exactly one read of ODS_ADMIN_SEED: {reads:?}");
}

// verifies: LLR-3zdw8v
#[test]
fn the_key_holders_derive_no_debug_and_have_no_display() {
    for (path, text) in sources() {
        for holder in ["SignatoryKey", "SeedBytes"] {
            let declaration = format!("pub struct {holder}");
            if let Some(position) = text.find(&declaration) {
                let before = &text[..position];
                let attributes: Vec<&str> = before.lines().rev().take_while(|line| line.trim_start().starts_with("#[") || line.trim_start().starts_with("///")).collect();
                assert!(attributes.iter().all(|line| !line.contains("Debug")), "{}: {holder} derives Debug", path.display());
            }
            assert!(!text.contains(&format!("Display for {holder}")), "{}: {holder} implements Display", path.display());
        }
    }
}

// verifies: LLR-3zdw8v
#[test]
fn org_io_names_no_device_or_member_private_key() {
    for (path, text) in sources() {
        for needle in ["ed25519_dalek::SigningKey", "SigningKeypair", "DeviceSeed", "MemberSeed"] {
            assert!(!text.contains(needle), "{} names `{needle}`", path.display());
        }
    }
}
```

   The first test requires the read to be written on one line
   (`std::env::var("ODS_ADMIN_SEED")`), as step 5 writes it. Show it red
   after step 5 by deleting the `cfg` line, then revert.
5. Implement in `org-io/src/custody.rs`:

```rust
use subxt_signer::sr25519::Keypair;

/// The user's own sr25519 signatory key (LLR-c4bktx). The key pair is
/// private; the writer reaches it through `keypair`, crate-private.
pub struct SignatoryKey {
    keypair: Keypair,
}

impl SignatoryKey {
    /// Build the key from its seed; the seed is consumed and wiped.
    pub fn from_seed(seed: SeedBytes) -> Result<Self, ConfigError> {
        Keypair::from_secret_key(*seed.bytes())
            .map(|keypair| Self { keypair })
            .map_err(|_| ConfigError { variable: ConfigVariable::AdminSeed, rule: ConfigRule::NotAKey })
    }

    /// The public account: the key's sr25519 public key.
    pub fn account_id(&self) -> AccountId {
        AccountId(self.keypair.public_key().0)
    }

    pub(crate) fn keypair(&self) -> &Keypair {
        &self.keypair
    }
}

impl fmt::Debug for SignatoryKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let public = self.keypair.public_key().0;
        write!(formatter, "SignatoryKey(account {:02x}{:02x}{:02x}{:02x}..)", public[0], public[1], public[2], public[3])
    }
}

/// The one read of `ODS_ADMIN_SEED`, development builds only (LLR-rgdx22,
/// REQ-8zuka3). `Ok(None)` when the variable is unset.
#[cfg(feature = "dev-seed")]
pub fn signatory_from_environment() -> Result<Option<SignatoryKey>, ConfigError> {
    match std::env::var("ODS_ADMIN_SEED") {
        Ok(raw) => parse_seed(&raw).and_then(SignatoryKey::from_seed).map(Some),
        Err(_) => Ok(None),
    }
}
```

   `public[0..4]` indexing is on a `[u8; 32]`, so clippy's
   `indexing_slicing` (not denied here) does not apply; if the workspace
   denies it later, use `public.iter().take(4)`. Drop the `same_bytes_as`
   helper's visibility to what the test needs (it stays `pub`; it reveals
   only equality).
6. Run: `cargo test -p org-io --test signatory_key --test absences` and
   `cargo test -p org-io --features dev-seed --test dev_seed_env`: green.
   Show the absence tests red once: add `#[derive(Debug)]` above
   `pub struct SeedBytes` (and remove its hand-written impl), run, paste
   the failure, revert.
7. Config: extend the first verify line with `--test signatory_key --test absences`,
   and add `  - cargo test -p org-io --features dev-seed --test key_custody_config --test signatory_key --test dev_seed_env`
   after it. SOUP: add `subxt-signer` (0.50.x from `Cargo.lock`; the
   sr25519 signatory key; LLR-c4bktx; anomaly: its `Keypair` derives
   `Clone`, so custody rests on org-io never cloning it out — LLR-u2pk5y's
   scan).
8. Commit: `feat(org-io): the signatory key holder and the development-only seed read (LLR-c4bktx, LLR-rgdx22, LLR-3zdw8v)`.

### T3 — on-chain-client: the two signatory-set reads

**Files touched:** on-chain-client/src/write/mod.rs, on-chain-client/src/write/signatory_set.rs, on-chain-client/src/client.rs, on-chain-client/tests/signatory_set_decode.rs, on-chain-client/tests/fuzz_signatory_set/fuzz_target.rs, on-chain-client/Cargo.toml, on-chain-client/.guardrails/config.yaml, on-chain-client/docs/requirements/DRAFT-worktree-org-io-create-signatory-reads.md, on-chain-client/docs/architecture/DRAFT-worktree-org-io-create-signatory-reads.md, on-chain-client/docs/risk/DRAFT-worktree-org-io-create-signatory-reads.md
**Parallel:** yes, with T1 and T2 (no file in common); serial after T0
**IDs written and verified:** REQ-8p2veg, REQ-v8jczx, SDD-rxfu6h, LLR-m3tjvp, LLR-a9bb7b (minted 2026-10-07)

1. Write `on-chain-client/docs/requirements/DRAFT-worktree-org-io-create-signatory-reads.md`
   (heading, a two-paragraph intro naming the change, org-io's REQ-f3eu9n as
   the consumer in prose — on-chain-client does not depend on org-io, so no
   org-io ID — and these items):

```markdown
  **REQ-8p2veg**: The software shall provide a read of the account that
pallet-revive maps an Organisation's H160 to (`Revive.OriginalAccount`) at
the latest finalised block, returning the 32-byte account when the H160 is
mapped, nothing when it is not, and an error when the read fails or the
stored value is not a 32-byte account.
satisfies: derived
exported: yes

  **REQ-v8jczx**: The software shall provide a read of an account's proxy
delegates (`Proxy.Proxies`) at the latest finalised block, returning every
delegate account the stored definitions name, an empty list when the account
has no proxies, and an error when the read fails or the stored value does not
decode as the runtime's proxy definitions.
satisfies: derived
exported: yes
```

2. Architecture draft `on-chain-client/docs/architecture/DRAFT-worktree-org-io-create-signatory-reads.md`:

```markdown
  **SDD-rxfu6h**: the signatory-set reads: the pure decoders of the two
storage values that let a consumer confirm a multisig signatory set
(`write/signatory_set.rs`, compiled with the `write` feature beside
`multi_account_id`), and their two fetches in `client.rs`, which are the
subxt shell SDD-3b8zef's and carry no low-level requirement by its recorded
deviation.
traces: REQ-8p2veg, REQ-v8jczx

  **LLR-m3tjvp**: `decode_original_account(bytes: &[u8]) -> Result<AccountId, SignatorySetError>`
accepts exactly 32 bytes and returns them as the account; any other length is
`SignatorySetError::Malformed`.
satisfies: REQ-8p2veg

  **LLR-a9bb7b**: `decode_proxy_delegates(bytes: &[u8]) -> Result<Vec<AccountId>, SignatorySetError>`
decodes the SCALE value `(BoundedVec<ProxyDefinition<AccountId32, ProxyType, u32>>, u128)`
— a compact length `n`, then `n` definitions of a 32-byte delegate, a
one-byte proxy type and a four-byte little-endian delay, then a 16-byte
deposit — and returns the `n` delegates in stored order; a value with
trailing bytes, too few bytes, or a length prefix larger than the bytes
that follow is `Malformed`, and no input panics or allocates more than the
input's length allows.
satisfies: REQ-v8jczx
```

   Risk draft `on-chain-client/docs/risk/DRAFT-worktree-org-io-create-signatory-reads.md`:
   one assessment paragraph per REQ, each ending `assesses: REQ-8p2veg` /
   `assesses: REQ-v8jczx`: no hazard impact in S2 (the consumer acts on
   nothing beyond reporting; a wrong "admin" answer cannot authorise a write,
   because the chain refuses a dispatch from a non-delegate; a decode error
   is reported, never read as "no delegates").
3. Write the failing test `on-chain-client/tests/signatory_set_decode.rs`
   (compile-red):

```rust
#![cfg(feature = "write")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The pure decoders behind the signatory-set reads (SDD-rxfu6h).

use on_chain_client::write::signatory_set::{decode_original_account, decode_proxy_delegates, SignatorySetError};
use on_chain_client::write::AccountId;
use parity_scale_codec::{Compact, Encode};

fn definitions(delegates: &[[u8; 32]]) -> Vec<u8> {
    let mut bytes = Compact(delegates.len() as u32).encode();
    for delegate in delegates {
        bytes.extend_from_slice(delegate);
        bytes.push(0); // ProxyType::Any
        bytes.extend_from_slice(&0u32.to_le_bytes()); // delay
    }
    bytes.extend_from_slice(&1_000u128.to_le_bytes()); // deposit
    bytes
}

// verifies: LLR-m3tjvp
#[test]
fn an_original_account_is_its_32_bytes() {
    assert_eq!(decode_original_account(&[7u8; 32]).unwrap(), AccountId([7u8; 32]));
}

// verifies: LLR-m3tjvp
#[test]
fn an_original_account_of_another_length_is_malformed() {
    assert_eq!(decode_original_account(&[7u8; 31]), Err(SignatorySetError::Malformed));
    assert_eq!(decode_original_account(&[7u8; 33]), Err(SignatorySetError::Malformed));
    assert_eq!(decode_original_account(&[]), Err(SignatorySetError::Malformed));
}

// verifies: LLR-a9bb7b
#[test]
fn proxy_definitions_give_their_delegates_in_order() {
    let bytes = definitions(&[[1u8; 32], [2u8; 32]]);
    assert_eq!(decode_proxy_delegates(&bytes).unwrap(), vec![AccountId([1u8; 32]), AccountId([2u8; 32])]);
    assert_eq!(decode_proxy_delegates(&definitions(&[])).unwrap(), vec![]);
}

// verifies: LLR-a9bb7b
#[test]
fn truncated_padded_or_overlong_definitions_are_malformed() {
    let good = definitions(&[[1u8; 32]]);
    assert_eq!(decode_proxy_delegates(&good[..good.len() - 1]), Err(SignatorySetError::Malformed));
    let mut padded = good.clone();
    padded.push(0);
    assert_eq!(decode_proxy_delegates(&padded), Err(SignatorySetError::Malformed));
    let mut overlong = Compact(u32::MAX).encode();
    overlong.extend_from_slice(&[0u8; 16]);
    assert_eq!(decode_proxy_delegates(&overlong), Err(SignatorySetError::Malformed));
}
```

   Fuzz target `on-chain-client/tests/fuzz_signatory_set/fuzz_target.rs`,
   in the shape of `on-chain-client/tests/fuzz_decode_org_state/fuzz_target.rs`
   (copy its `verifies:` placement and harness), body:
   `bolero::check!().for_each(|bytes: &[u8]| { let _ = decode_original_account(bytes); let _ = decode_proxy_delegates(bytes); });`
   annotated `verifies: LLR-m3tjvp, LLR-a9bb7b`. Add both
   `[[test]]` entries to `on-chain-client/Cargo.toml` with
   `required-features = ["write"]` (the fuzz one with `harness = false`), as
   the four `write_*` targets are declared.
4. Implement `on-chain-client/src/write/signatory_set.rs`:

```rust
//! The signatory-set reads' pure half (SDD-rxfu6h): decoding the stored
//! values of `Revive.OriginalAccount` and `Proxy.Proxies`.

use parity_scale_codec::{Compact, Decode};

use super::AccountId;

/// A stored value that is not what the runtime stores there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignatorySetError {
    Malformed,
}

/// One proxy definition: delegate, proxy type, delay.
const DEFINITION_BYTES: usize = 32 + 1 + 4;
const DEPOSIT_BYTES: usize = 16;

/// `Revive.OriginalAccount`'s value: an AccountId32 (LLR-m3tjvp).
pub fn decode_original_account(bytes: &[u8]) -> Result<AccountId, SignatorySetError> {
    <[u8; 32]>::try_from(bytes).map(AccountId).map_err(|_| SignatorySetError::Malformed)
}

/// `Proxy.Proxies`' value: the delegates, in stored order (LLR-a9bb7b).
pub fn decode_proxy_delegates(bytes: &[u8]) -> Result<Vec<AccountId>, SignatorySetError> {
    let mut input = bytes;
    let count = Compact::<u32>::decode(&mut input).map_err(|_| SignatorySetError::Malformed)?.0 as usize;
    let needed = count
        .checked_mul(DEFINITION_BYTES)
        .and_then(|definitions| definitions.checked_add(DEPOSIT_BYTES))
        .ok_or(SignatorySetError::Malformed)?;
    if input.len() != needed {
        return Err(SignatorySetError::Malformed);
    }
    Ok(input
        .chunks_exact(DEFINITION_BYTES)
        .take(count)
        .filter_map(|definition| definition.get(..32).and_then(|delegate| <[u8; 32]>::try_from(delegate).ok()))
        .map(AccountId)
        .collect())
}
```

   Add `pub mod signatory_set;` to `on-chain-client/src/write/mod.rs`.
5. Add the two fetches to `OrgRegistryClient` in `on-chain-client/src/client.rs`
   (under the `write` feature, beside `get_org_state`; SDD-3b8zef's shell):
   `pub async fn original_account(&self, admin: OrgAdmin) -> Result<Option<write::AccountId>, ClientError>`
   and
   `pub async fn proxy_delegates(&self, account: write::AccountId) -> Result<Vec<write::AccountId>, ClientError>`.
   Each resolves the latest **finalised** block the way `get_org_state`
   resolves `at = None` (read that function and use the same block
   selection, so REQ-ysyu9g's premise holds for these reads too), fetches the
   raw storage value with subxt's dynamic storage API for
   `("Revive", "OriginalAccount", [h160])` and `("Proxy", "Proxies", [account])`
   (`fetch_raw` or the 0.50 equivalent; match how `read_contract_slot` calls
   subxt), maps an absent value to `None` / `vec![]`, and decodes a present
   one with the step-4 decoder, mapping `Malformed` to
   `ClientError::Decode` (or the closest existing variant; read
   `ClientError`). No gated test reaches these two; say so in their doc
   comments ("SDD-3b8zef's shell; exercised only against a chain").
6. Run `cargo test --manifest-path on-chain-client/Cargo.toml --features test-support,write --test signatory_set_decode --test fuzz_signatory_set`: green.
   Extend `on-chain-client/.guardrails/config.yaml`'s second
   `verify_commands` line with `--test signatory_set_decode --test fuzz_signatory_set`.
   Run both on-chain-client verify lines and
   `cargo clippy --manifest-path on-chain-client/Cargo.toml --features test-support,write --all-targets -- -D warnings`.
7. `GR_CONFIG=on-chain-client/.guardrails/config.yaml .guardrails/scripts/check-trace.sh`:
   exit 0 (the new REQs are covered through their tested LLRs; both are
   assessed). Commit: `feat(on-chain-client): the signatory-set reads (REQ-8p2veg, REQ-v8jczx)`.

### T4 — PR-b795an: on-chain-client's coverage measures the writer

**Files touched:** Makefile, on-chain-client/.guardrails/config.yaml, on-chain-client/docs/problems/2026-10-06-writer-coverage.md
**Parallel:** no (serial, after T3: both edit on-chain-client's config); may run beside T1/T2
**IDs:** resolves PR-b795an

1. Reproduce: `make coverage-on-chain-client` and note that its summary
   lists no file under `src/write/` (the defect: the recipe builds without
   `write`). Paste the summary lines for `src/write/*` (absent).
2. `Makefile`: in `ON_CHAIN_CLIENT_COVERAGE_ARGS` (from the branch-coverage
   change, merged in T0), change `--features test-support` to
   `--features test-support,write` and append
   `--test write_manifest --test write_pure --test write_compose --test write_events --test signatory_set_decode --test fuzz_signatory_set`,
   so both `coverage-on-chain-client` and `coverage-branch-on-chain-client`
   measure the writer.
3. Run `make coverage-on-chain-client` and `make coverage-branch-on-chain-client`.
   Record the new line, region and branch figures in the Makefile's
   on-chain-client comment block as a dated row ("2026-10-0x, +write: …"),
   in the style of the rows already there. Move each floor up to one point
   below the new figure, rounded down, **only if the figure rose**; if a
   figure fell (the writer's `subxt_ops.rs` is chain-only shell and enlarges
   the denominator), keep the floor, and write the reason beside it as the
   2026-09-10 recalibration note does — do not lower a floor.
4. `on-chain-client/.guardrails/config.yaml`: update the coverage comment
   above `coverage_command` (`:84-135`) with the new measurement.
5. `on-chain-client/docs/problems/2026-10-06-writer-coverage.md`: set
   `status: resolved`, add `resolved-by: worktree-org-io-create` (copy the
   exact resolution fields another resolved report in this repository
   uses, e.g. `org-node/docs/problems/2026-10-07-key-rotation.md`), and a
   dated paragraph with the before/after figures.
6. Run `check-trace.sh` for on-chain-client: PR-b795an leaves the open list.
   Commit: `fix(on-chain-client): coverage measures the write feature (resolves PR-b795an)`.

### T5 — org-io: the chain connection, the read, and the preflight

**Files touched:** org-io/Cargo.toml, Cargo.lock, org-io/src/lib.rs, org-io/src/connect.rs, org-io/src/chain_read.rs, org-io/src/preflight.rs, org-io/src/bin/preflight.rs, org-io/tests/chain_read_rule.rs, org-io/tests/preflight.rs, org-io/tests/common/mod.rs, org-io/.guardrails/config.yaml, org-io/docs/architecture/DRAFT-worktree-org-io-create-org-io.md, org-io/docs/requirements/DRAFT-worktree-org-io-create-org-io.md, org-io/docs/risk/DRAFT-worktree-org-io-create-org-io.md, org-io/docs/architecture/soup.md, org-node/tests/verify_against_chain.rs, org-node/docs/architecture/2026-10-03-decomposition.md, org-node/docs/requirements/2026-09-06-dependency-expectations.md, org-node/docs/risk/2026-09-06-dependency-expectations.md, org-node/docs/risk/2026-09-09-org-node-hazards.md
**Parallel:** no (serial, after T2; before T6)
**IDs verified:** LLR-rm9x4z (moved here, amended); moves REQ-ysyu9g and SDD-z85ux9

1. `org-io/Cargo.toml` `[dependencies]`: add
   `org-node = { path = "../org-node", features = ["app"] }`,
   `subxt = { version = "0.50", default-features = false, features = ["native", "jsonrpsee", "reconnecting-rpc-client"] }`,
   `tokio = { version = "1", features = ["rt-multi-thread", "macros", "time"] }`,
   `async-trait = "0.1"`, `hex = "0.4"` (org-node's lines `:45-49`). Copy
   org-node's chopsticks dev dependencies (`org-node/Cargo.toml:70-78`) and
   `org-node/tests/common/` (the chopsticks harness `preflight.rs` uses) into
   org-io, and add
   `[[bin]] name = "preflight"` and `[[test]] name = "preflight"` (no
   `required-features`: org-io always has the chain).
2. Write the failing `org-io/tests/chain_read_rule.rs` (compile-red):

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The chain read hands org-node a value: present, absent, or an error —
//! never absence for a failure (LLR-rm9x4z, moved from org-node 2026-10-0x).

use on_chain_client::{Epoch as ChainEpoch, OnChainRootHash, OrgPubKey, OrgState as ChainOrgState};
use org_io::chain_read::{org_state_from_chain, read_org_state, StateReader};
use org_io::node::{Epoch, OrgId, OrgNodeError, OrgPrivateKey, OrgState, RootHash};

fn x25519_key(seed: u8) -> [u8; 32] {
    *OrgPrivateKey::from([seed; 32]).x25519_keypair().org_public_key().unwrap().as_bytes()
}

fn chain_state(key: [u8; 32]) -> ChainOrgState {
    ChainOrgState { root_hash: OnChainRootHash([0x33; 32]), org_pub_key: OrgPubKey(key), epoch: ChainEpoch(7) }
}

struct ScriptedReader(Result<Option<ChainOrgState>, String>);

#[async_trait::async_trait]
impl org_io::chain_read::RawStateSource for ScriptedReader {
    async fn fetch(&self, _org_id: OrgId) -> Result<Option<ChainOrgState>, String> {
        self.0.clone()
    }
}

// Moved from org-node/tests/chain_read_state.rs (2026-10-0x).
// verifies: LLR-rm9x4z
#[test]
fn chain_state_is_read_into_typed_values() {
    let key = x25519_key(0x11);
    let state = org_state_from_chain(chain_state(key)).unwrap();
    assert_eq!(state.root_hash, RootHash::new([0x33; 32]));
    assert_eq!(state.org_pub_key.as_bytes(), &key);
    assert_eq!(state.epoch, Epoch::new(7));
}

// Moved from org-node/tests/chain_read_state.rs (2026-10-0x).
// verifies: LLR-rm9x4z
#[test]
fn a_chain_state_whose_key_org_node_refuses_is_an_error() {
    assert_eq!(org_state_from_chain(chain_state([0u8; 32])).unwrap_err(), OrgNodeError::InvalidOrgPublicKey);
}

// verifies: LLR-rm9x4z
#[tokio::test]
async fn present_absent_failed_and_refused_reads_are_four_different_answers() {
    let org = OrgId::new([1u8; 20]);
    let present = read_org_state(&ScriptedReader(Ok(Some(chain_state(x25519_key(0x11))))), org).await.unwrap();
    assert!(present.is_some());
    assert_eq!(read_org_state(&ScriptedReader(Ok(None)), org).await.unwrap(), None);
    let failed = read_org_state(&ScriptedReader(Err("registry read failed".into())), org).await.unwrap_err();
    assert_eq!(failed, OrgNodeError::Chain("get_org_state: registry read failed".into()));
    let refused = read_org_state(&ScriptedReader(Ok(Some(chain_state([0u8; 32])))), org).await.unwrap_err();
    assert_eq!(refused, OrgNodeError::InvalidOrgPublicKey);
}
```

   The failed-read case is org-node's deleted `FailingChain` test, moved:
   the distinction now lives where the read happens.
3. Implement `org-io/src/chain_read.rs`:

```rust
//! The chain read (SDD-6qz9ms): read, parse through org-node, hand over a value.

use async_trait::async_trait;
use on_chain_client::{OrgAdmin, OrgRegistryClient};
use org_node::{OrgId, OrgNodeError, OrgState};

/// org-io's read seam: the state for an Organisation, parsed.
#[async_trait]
pub trait StateReader: Send + Sync {
    async fn read_state(&self, org_id: OrgId) -> Result<Option<OrgState>, OrgNodeError>;
}

/// The unparsed read, so the parse rule is tested without a chain.
#[async_trait]
pub trait RawStateSource: Send + Sync {
    async fn fetch(&self, org_id: OrgId) -> Result<Option<on_chain_client::OrgState>, String>;
}

/// Through org-node's one parse edge (REQ-8jb4ny): moved from org-node's chain_read.rs.
pub fn org_state_from_chain(state: on_chain_client::OrgState) -> Result<OrgState, OrgNodeError> {
    OrgState::from_chain(state.root_hash.0, state.org_pub_key.0, state.epoch.0)
}

/// Present, absent, or an error — a failure is never absence (LLR-rm9x4z).
pub async fn read_org_state(source: &dyn RawStateSource, org_id: OrgId) -> Result<Option<OrgState>, OrgNodeError> {
    source
        .fetch(org_id)
        .await
        .map_err(|reason| OrgNodeError::Chain(format!("get_org_state: {reason}")))?
        .map(org_state_from_chain)
        .transpose()
}

/// The production reader (SDD-z85ux9's shell): today's `SubxtChainOps::read_state`.
pub struct OnChainStateReader {
    registry: OrgRegistryClient,
}

impl OnChainStateReader {
    pub fn new(registry: OrgRegistryClient) -> Self {
        Self { registry }
    }
}

#[async_trait]
impl RawStateSource for OnChainStateReader {
    async fn fetch(&self, org_id: OrgId) -> Result<Option<on_chain_client::OrgState>, String> {
        self.registry.get_org_state(OrgAdmin(*org_id.as_bytes()), None).await.map_err(|error| error.to_string())
    }
}

#[async_trait]
impl StateReader for OnChainStateReader {
    async fn read_state(&self, org_id: OrgId) -> Result<Option<OrgState>, OrgNodeError> {
        read_org_state(self, org_id).await
    }
}
```

   The error text matches today's `SubxtChainOps` (`get_org_state: {e}`),
   so the app's receive-error classification sees the same `Chain(_)`.
   `org-io/src/connect.rs`: copy `connect_chain_client`
   (`org-node/src/service.rs:176-216`) as `pub async fn connect(ws_url: &str, contract: [u8; 20])`
   with its comments, unchanged otherwise. `org-io/src/preflight.rs` and
   `org-io/src/bin/preflight.rs`: copy org-node's, replacing
   `crate::service::connect_chain_client` with `crate::connect::connect`,
   `crate::keys`/`crate::transport` with `org_node::…`. `org-io/tests/preflight.rs`:
   copy org-node's, adjusting paths. `lib.rs`: `pub mod chain_read; pub mod connect; pub mod preflight;`
   and `pub use org_node as node;` (transitional, accepted 2026-10-06; S4
   narrows it).
4. Run `cargo test -p org-io --test chain_read_rule`: green;
   `cargo build -p org-io --bin preflight`: builds. (`tests/preflight.rs`
   needs chopsticks; it is not in `verify_commands`, as in org-node.) Add
   `--test chain_read_rule` to org-io's first verify line, and the comment
   that `preflight` is excluded, copied from org-node's config `:48`.
5. Ledger moves (ruling A), each with a dated note at the source naming
   org-io's file, no ID:
   - LLR-rm9x4z: cut its whole block from
     `org-node/docs/architecture/2026-10-03-decomposition.md:630` (with its
     amendments) into `org-io/docs/architecture/DRAFT-worktree-org-io-create-org-io.md`
     under SDD-6qz9ms, headed `*Moved 2026-10-0x from org-node/docs/architecture/2026-10-03-decomposition.md:630 (ruling A).*`,
     then amend it: `satisfies: REQ-tg9zrn` (was REQ-bvh8v6), the subject is
     `read_org_state`, and the cache exception is retired. Text:
     "`read_org_state` returns `Ok(None)` for an Organisation with no
     on-chain slot, `Ok(Some(state))` for one whose state org-node's parse
     accepts, and `Err`, never `Ok(None)`, for a failed read
     (`OrgNodeError::Chain`) or a refused parse (the parse's error)."
     In `org-node/tests/verify_against_chain.rs:107` and `:139` remove
     `LLR-rm9x4z` from the `verifies:` lines (T6 then deletes the second test).
   - SDD-z85ux9: cut its section (`decomposition.md:2451-` through the end
     of "The item with no low-level requirements", including the size
     measurement) into the org-io architecture draft as a fifth item, after
     SDD-z3ychz; re-point `traces:` to `REQ-tg9zrn, REQ-f3eu9n` (T7 adds the
     moved REQ-nfr3n2); rewrite its code list to org-io's (`connect.rs`,
     `OnChainStateReader`, the preflight; T7 adds `OnChainWriter`, T8 the
     signatory-set reader); re-measure its size (`wc -l` of the shell files,
     by its own counting rule) and record the figure. Leave at the source a
     dated paragraph: "Moved to org-io's architecture ledger … (ruling A);
     org-node has no chain-facing shell after ruling B."
   - REQ-ysyu9g: cut its block from
     `org-node/docs/requirements/2026-09-06-dependency-expectations.md:50`
     into `org-io/docs/requirements/DRAFT-worktree-org-io-create-org-io.md`
     (keep `expects: on-chain-client`, `opened: 2026-09-06`), restating
     "what org-node owes in return" for org-io; cut its assessment
     (`org-node/docs/risk/2026-09-06-dependency-expectations.md:31` and its
     paragraph) into org-io's risk draft. Rewrite org-node's hazard-file
     mentions (`2026-09-09-org-node-hazards.md:69`, `:111`, `:115`, `:247`,
     `:274`, `:942`, `:998`, `:1045`, `:1125`, `:1325`) to "the
     finalised-block expectation now held by org-io
     (`org-io/docs/requirements/…`)", no ID.
   - org-io's requirements draft: remove the sentence "Until those
     definitions arrive, this file names none of their IDs" for the moved
     ones now present, and the architecture draft's matching paragraph.
6. SOUP (`org-io/docs/architecture/soup.md`): add `subxt`, `jsonrpsee`
   (through subxt), `tokio`, `async-trait`, `hex`, with versions from
   `Cargo.lock`, roles copied from `org-node/docs/architecture/soup.md:140-145`,
   and the chopsticks dev dependencies' row from its `:173`.
7. Run org-io's verify lines; `check-trace.sh` for org-node: exit 0 (no
   LLR-rm9x4z left there; its absence of tests is fine because it moved).
   Commit: `feat(org-io): the chain connection, the read and the preflight move in (LLR-rm9x4z, SDD-z85ux9, REQ-ysyu9g moved)`.

### T6 — org-node: values in, values out

**Files touched:** org-node/Cargo.toml, Cargo.lock, org-node/src/lib.rs, org-node/src/service.rs, org-node/src/chain.rs, org-node/src/verify.rs, org-node/src/chain_read.rs, org-node/src/preflight.rs, org-node/src/bin/preflight.rs, org-node/src/test_fixtures.rs, org-node/tests/support/mod.rs, org-node/tests/absences.rs, org-node/tests/admission_sender.rs, org-node/tests/commit_paths.rs, org-node/tests/expected_admission.rs, org-node/tests/persona_records.rs, org-node/tests/organisation_key.rs, org-node/tests/service_stories.rs, org-node/tests/service_lifecycle.rs, org-node/tests/receive_chain_reads.rs, org-node/tests/reconcile.rs, org-node/tests/transport_handshake.rs, org-node/tests/transport_networked.rs, org-node/tests/verify_against_chain.rs, org-node/tests/chain_read_state.rs, org-node/tests/preflight.rs, org-node/tests/common/ (deleted), org-node/tests/fuzz_verify_against_chain/fuzz_target.rs, org-node/.guardrails/config.yaml, org-node/docs/architecture/2026-10-03-decomposition.md, org-node/docs/architecture/2026-10-06-chain-authority.md, org-node/docs/architecture/2026-10-07-org-key-pair.md, org-node/docs/architecture/2026-10-07-commit-workflow.md, org-node/docs/architecture/2026-10-04-type-safety.md, org-node/docs/architecture/DRAFT-worktree-org-io-create-values.md, org-node/docs/requirements/2026-09-09-verify-and-commit.md, org-node/docs/requirements/2026-10-07-commit-workflow.md, org-node/docs/risk/2026-10-07-commit-workflow.md, org-node/docs/problems/2026-10-05-stale-cache-on-fetch-error.md
**Parallel:** no (serial, after T5: T5 must have copied the chain checks of the preflight, the connection and `org_state_from_chain` into org-io first)
**IDs verified:** LLR-mn5c2q (new), LLR-65py3d, LLR-hg3xzf, LLR-ryzr8m, LLR-8m99q2, LLR-ewkg85, LLR-9ew26y, LLR-s8xp7m, LLR-379hnv, LLR-xn5pwc, LLR-ba2ejp, LLR-38e2kn, LLR-pt32fx, and S3a's LLR-2r2fha, LLR-kzgjz8, LLR-3aysup, LLR-3q63zv, LLR-dc45ur, LLR-gr8x3r, LLR-fm38ww (amended or re-pointed, annotations kept); resolves PR-k2xxaq

*Rewritten 2026-10-08 for S3a (`60c7d58`).* The receive paths now carry
three message kinds and the sender `receive_one` returns; a revocation is
decided by `revocation::accept` and an acknowledgement by
`revocation::check_acknowledgement`; `reconcile` and the commit step take
lazy device-seed sources. The shape below keeps every S3a decision where it
is and cuts only the chain reads out of `OrgService`.

The new interface (ruling B). Write these signatures exactly; T7 and T9
call them.

```rust
// org-node/src/service.rs
impl OrgService {
    pub fn new(store: PersonaStore) -> Self;
    pub fn commit_genesis<R: RngCore + CryptoRng>(&mut self, rng: &mut R, persona_id: &PersonaId,
        org_id: OrgId, proxy_account: ChainAccount, chain_state: Option<OrgState>) -> Result<ReceiveOutcome, OrgNodeError>;
    pub fn commit_update<R: RngCore + CryptoRng>(&mut self, rng: &mut R, org_id: OrgId,
        chain_state: Option<OrgState>) -> Result<CommitOutcome, OrgNodeError>;
    /// S3a's interim caller of `reconcile::reconcile`, now given the state: `OrgNotHeld`
    /// before the state is looked at, then `None` → `OrgNotOnChain`, then the pure reconcile.
    pub fn reconcile<R: RngCore + CryptoRng>(&mut self, rng: &mut R, org_id: OrgId,
        chain_state: Option<OrgState>) -> Result<crate::reconcile::Reconciled, OrgNodeError>;
    /// Transport only: one message and the Device the transport authenticated for it,
    /// from the first Persona's endpoint (S3a's private `receive_one`, made public).
    pub async fn receive_message(&mut self) -> Result<(DevicePublicKey, WireMessage), OrgNodeError>;
    /// Every check of `receive_and_verify` that needs no chain, on values: the sender rule
    /// (LLR-2r2fha, LLR-kzgjz8), `check_notice` (LLR-r7zm39, LLR-38e2kn), the acknowledgement
    /// (LLR-5azhry, LLR-3aysup: finished here, `Prepared::Done`), `AdmissionNotExpected`
    /// (LLR-s8xp7m), the snapshot decode, `check_chain_free` (LLR-9ew26y).
    pub fn prepare_receive(&self, sender: DevicePublicKey, message: WireMessage)
        -> Result<Prepared<ReceiveOutcome>, OrgNodeError>;
    /// The rest of `receive_and_verify`, against `chain_state`.
    pub fn apply_receive<R: RngCore + CryptoRng>(&mut self, rng: &mut R, pending: PendingReceive<ReceiveOutcome>,
        chain_state: Option<OrgState>) -> Result<ReceiveOutcome, OrgNodeError>;
    /// The chain-free half of `receive_and_self_delete_if_revoked` (LLR-379hnv, LLR-3q63zv, LLR-pt32fx).
    pub fn prepare_self_delete(&self, sender: DevicePublicKey, message: WireMessage)
        -> Result<Prepared<SelfDeleteOutcome>, OrgNodeError>;
    pub fn apply_self_delete<R: RngCore + CryptoRng>(&mut self, rng: &mut R, pending: PendingReceive<SelfDeleteOutcome>,
        chain_state: Option<OrgState>) -> Result<SelfDeleteOutcome, OrgNodeError>;
}

/// What the chain-free phase decided: finished with no chain state, or a pending value
/// that needs the state of the Organisation it names.
pub enum Prepared<O> {
    Done(O),
    NeedsChain(PendingReceive<O>),
}

/// A message that passed the chain-free checks; opaque but for the Organisation it names.
/// `O` ties it to the path that prepared it: `apply_receive` takes only what
/// `prepare_receive` made, `apply_self_delete` only what `prepare_self_delete` made.
pub struct PendingReceive<O> { kind: PendingKind, path: std::marker::PhantomData<O> }
enum PendingKind {
    /// `check_notice` and the sender rule passed.
    Revocation { notice: RevocationNotice },
    /// The sender rule (held record) or the expectation gate (first admission) passed,
    /// the snapshot decoded, `check_chain_free` passed.
    OrgInformation { envelope: Envelope, carried_key: OrgPrivateKey, existing: Option<OrgRecord>,
        local_trie: Trie, ctx: VerifyContext, expectation: Option<ExpectedAdmission> },
}
impl<O> PendingReceive<O> { pub fn org_id(&self) -> OrgId; }

// org-node/src/verify.rs
pub fn verify_envelope_against_chain(local_trie: &Trie, envelope: &Envelope, ctx: &VerifyContext,
    chain_state: Option<OrgState>) -> Result<VerifiedUpdate, OrgNodeError>; // None → OrgNotOnChain

// org-node/src/test_fixtures.rs (test-support only)
#[derive(Clone, Default)]
pub struct ChainSlots { /* Arc<Mutex<HashMap<OrgId, OrgState>>> plus the id counter */ }
impl ChainSlots {
    pub fn new() -> Self;
    pub fn set(&self, org_id: OrgId, state: OrgState);
    pub fn get(&self, org_id: &OrgId) -> Option<OrgState>;
    pub fn apply_genesis(&self, root: RootHash, key: OrgPublicKey) -> OrgId;          // LLR-ryzr8m
    pub fn apply_update(&self, org_id: OrgId, root: RootHash, key: OrgPublicKey,
        expected_epoch: Epoch) -> Result<(), OrgNodeError>;                               // LLR-ryzr8m
}
```

`receive_and_verify` and `receive_and_self_delete_if_revoked` leave
`OrgService` (org-io composes them in T7). `OrgMembersError` is added to
org-node's root re-exports (`pub use org_members::{…, OrgMembersError}`), so
the app's `receiver_events.rs` can reach it through org-io in T9.

Where each S3a part goes, and how its chain or IO input becomes a value:

| S3a part (`service.rs` today) | After T6 | Its chain / IO input |
|---|---|---|
| `receive_one` `:1005` (returns sender and message) | `receive_message`, public, unchanged body; `TransportError::Malformed` → `MalformedMessage` stays here (LLR-xn5pwc, LLR-dc45ur) | Transport, org-node's until S4 |
| Sender rule `sender_listed` `:1364` (LLR-2r2fha, LLR-kzgjz8; R1) | Called in `prepare_*` with the `sender` argument: for Organisation information about a held record (after the held lookup, before `check_chain_free`) and for a revocation (after `check_notice`) | The sender is a value the caller passes; in S2 org-io passes what `receive_message` returned, from S4 what its own transport returned (S4's `check_received(sender, body)` then needs only the body decode added) |
| Revocation, `receive_revocation` `:700` | `prepare_*`: `check_notice` (`RevocationNotHeld`, `RevocationNotForThisDevice`), then `sender_listed` → `NeedsChain(Revocation)`. `apply_*`: `revocation::accept(data, &notice, chain_state, \|\| device_seeds_bound_to(data, org_id))`, adopt and save the successor; `apply_receive` returns the state's epoch and root, `apply_self_delete` `SelfDeleted` | `chain_state: Option<OrgState>` from org-io's one read; `accept` already took `Option<OrgState>`. The seed source stays the internal lazy closure over the store's Persona seeds |
| Acknowledgement (`:803`, `:959`) | `prepare_*` only: `check_acknowledgement(store, sender, ack)` — sender = the named Device, checked before the signature (LLR-5azhry as amended) — returns `Done(ReceiveOutcome { acknowledged: Some(_), epoch/root of the record, .. })` or `Done(SelfDeleteOutcome::Acknowledged(_))` | None: no chain state, so no `PendingReceive` and no read |
| First admission (`:823-898`) | `prepare_receive`: no record → no sender rule (any sender, REQ-xa6smf); `AdmissionNotExpected` unless expected; `first_admission_base`; `check_chain_free`. `apply_receive`: verify against the state, `check_carried_key`, the own-Persona rule (`AdmissionNotOurs`), the record with `kept_change_set`, clear that expectation. `prepare_self_delete` refuses it `OrgNotOnChain` (LLR-379hnv) | `chain_state` from org-io's read |
| Update to a held record | `prepare_*`: held lookup, sender rule, `check_chain_free`. `apply_*`: `verify_envelope_against_chain(…, chain_state)`, `check_carried_key`, then `commit_received` or `remove_self(org_id, &state)` (the removal step, `device_seeds_bound_to` as today) | `chain_state` from org-io's read |
| `verify_received` `:1027` | Deleted: `check_chain_free` runs in `prepare_*`, the rest in `apply_*` | — |
| `commit_step` `:1300` | Unchanged but for `&ChainOpsReader { state: *chain }` → `Some(*chain)`; keeps its lazy seed source | `chain: &OrgState`, already a value |
| `commit_update` `:607` | `chain_state` argument; `None` → `OrgNotOnChain`; then `held_update_for` and `commit_step` as today | org-io's read-back after its write (LLR-9r2bxd) |
| `reconcile` `:1062` (interim) | Synchronous, `chain_state` argument; `OrgNotHeld` first, `None` → `OrgNotOnChain`, then `reconcile::reconcile(data, org_id, state, \|\| device_seeds_bound_to(…))` unchanged | No production caller in S2 (the app has none); S3b-io's startup reconcile (its T15) reads the chain in org-io and calls it |
| `revocation.rs`, `reconcile.rs` | Unchanged (already values in, values out, LLR-xgefn8) | — |

`PendingReceive` holds values taken from the store at prepare time (the
record, the trie). It is safe because the only callers — org-io's
`receive_*` in T7 and the test helpers below — run prepare, read and apply
under one `&mut` borrow, so no store change falls between them. S4, which
plans to release the service lock during the read (its §2.4), must
re-check the record in its apply step; noted for S4's T0.

1. Red first, the absence: add to `org-node/tests/absences.rs`

```rust
// org-node does no chain IO (ruling B, change worktree-org-io-create): no
// chain library, no chain seam, no async chain read, no preflight.
// verifies: LLR-mn5c2q, LLR-65py3d
#[test]
fn org_node_names_no_chain_library_and_reads_no_chain() {
    for (needle, why) in [
        // Paths, not the bare words: `types.rs`'s doc comment names subxt's
        // account type in prose, as `org_node_writes_nothing_to_the_chain` notes.
        ("on_chain_client::", "the chain is org-io's"),
        ("subxt::", "the chain is org-io's"),
        ("ChainOps", "org-node takes the chain state as a value"),
        ("ChainReader", "verification takes the chain state as a value"),
        ("read_state", "org-node reads no chain"),
        ("OnChainReader", "deleted with its cache (PR-k2xxaq)"),
        ("OrgStateCache", "deleted (PR-k2xxaq)"),
        ("connect_chain_client", "the connection is org-io's"),
    ] {
        assert_absent(needle, why);
    }
    let dependencies = normal_dependencies();
    for crate_name in ["subxt", "on-chain-client", "async-trait", "hex"] {
        assert!(!dependencies.contains(&format!("{crate_name} =")), "org-node depends on {crate_name}");
    }
    assert!(!std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap().contains("chain = ["),
        "org-node has no `chain` feature");
}
```

   Run `cargo test -p org-node --test absences`: red on `ChainOps`
   (`service.rs contains ChainOps`). Paste it.
2. Write the org-node draft ledger
   `org-node/docs/architecture/DRAFT-worktree-org-io-create-values.md`:
   a short intro (stage S2, ruling B) and

```markdown
  **LLR-mn5c2q**: org-node's source names no `on_chain_client::` or
  `subxt::` path, and no `ChainOps`, `ChainReader`, `read_state`,
  `OnChainReader`, `OrgStateCache` or `connect_chain_client`; its manifest
  has no `chain` feature and no `subxt`, `on-chain-client`, `async-trait` or
  `hex` dependency; every operation that judges against the chain
  (`commit_genesis`, `commit_update`, `reconcile`, `apply_receive`,
  `apply_self_delete`, `verify_envelope_against_chain`) takes the
  Organisation's state as an `Option<OrgState>` argument and is
  synchronous.
  satisfies: REQ-bvh8v6
```

   (REQ-bvh8v6 is the org-node requirement SDD-ueh4tm and LLR-8m99q2
   already serve: the Organisation's state comes from a path the sender does
   not control. Place the item under SDD-ueh4tm by stating that in the intro;
   add LLR-mn5c2q to nothing else.)
3. `org-node/src/test_fixtures.rs`: add `ChainSlots` (above), moving the
   body of `MockChainOps` (`service.rs:50-131`: `set`, `get`,
   `apply_genesis`, `apply_update`, the shared `Arc<Mutex<…>>`) unchanged
   apart from the type name. Keep its doc comments' LLR-ryzr8m citations.
4. `org-node/src/verify.rs`: change `verify_envelope_against_chain` to take
   `chain_state: Option<OrgState>`; replace the `chain.get_org_state(…)`
   call with `let state = chain_state.ok_or(OrgNodeError::OrgNotOnChain)?;`
   (the `Err` arm, which mapped a reader failure to `OrgNodeError::Chain`,
   goes: no reader). Delete `ChainReader`, `MockChain` and its unit test
   from `org-node/src/chain.rs`; keep `OrgState` and `OrgState::from_chain`.
   Update the module docs of both files.
5. `org-node/src/service.rs`:
   - delete `ChainOps`, `MockChainInner`, `MockChainOps`, `mod subxt_impl`,
     `SubxtChainOps`, `connect_chain_client` (`:34-217`), the
     `ChainOpsReader` adapter (`:1260-1275`), the `chain` field and the
     `async_trait` import; `OrgService::new(store)`;
   - `commit_genesis` (`:412`) / `commit_update` (`:607`): drop `async`,
     add the `chain_state: Option<OrgState>` parameter, and replace each
     `self.chain.read_state(org_id).await?.ok_or(OrgNodeError::OrgNotOnChain)?`
     (`:421`, `:613`) with `chain_state.ok_or(OrgNodeError::OrgNotOnChain)?`,
     keeping `ensure_unbound` / `find_org` before it as today;
   - `commit_step` (`:1300`): `&ChainOpsReader { state: *chain }` (`:1318`)
     → `Some(*chain)`; nothing else changes (its lazy seed source stays);
   - `reconcile` (`:1062`): drop `async`, add `chain_state`, keep the
     `OrgNotHeld` check first (`:1069`), replace the read (`:1072`) with
     `chain_state.ok_or(OrgNodeError::OrgNotOnChain)?`;
   - split `receive_and_verify` (`:780-917`) by message kind, as the table
     above says. `prepare_receive(&self, sender, message)`: for
     `Revocation(notice)`, `check_notice` then `sender_listed` (today's
     `receive_revocation` `:706-707`) → `NeedsChain`; for
     `Acknowledgement(ack)`, `check_acknowledgement(store, sender, ack)` and
     the record's epoch and root (`:803-814`) → `Done`; for
     `OrgInformation`, everything from `:816` to the `verify_received` call
     (`:848`), with `check_chain_free(&local_trie, &envelope, &ctx)` (the
     first line of `verify_received`, `:1033`) as its last step →
     `NeedsChain`. `apply_receive(&mut self, rng, pending, chain_state)`:
     for a revocation, the rest of `receive_revocation` (`:708-715`, with
     `chain_state` for the read) and the outcome of `:795-801`; for
     Organisation information, `let state = chain_state.ok_or(OrgNodeError::OrgNotOnChain)?;`,
     `verify_envelope_against_chain(&local_trie, &envelope, &ctx, Some(state))`,
     then everything from `check_carried_key` (`:852`) down, unchanged;
   - split `receive_and_self_delete_if_revoked` (`:948-998`) the same way
     into `prepare_self_delete` (the revocation and acknowledgement arms as
     above, the latter returning `Done(SelfDeleteOutcome::Acknowledged(_))`;
     Organisation information: `:964-982` and `check_chain_free`) and
     `apply_self_delete` (`:983-997` against the state; a revocation returns
     `SelfDeleted { org_id, acknowledgements }`);
   - `receive_one` (`:1005`) becomes `pub async fn receive_message`, body
     unchanged (it already returns the sender);
   - delete `verify_received` and `receive_revocation` (their halves now
     live in `prepare_*`/`apply_*`); update the module doc (`:1-4`) — it
     composes the store and the transport; the chain state is an argument —
     and the doc comments that name `ChainOps`, `MockChainOps` or a chain
     read (`:280-283`, `:403-411`, `:693-699`, `:775-779`, `:942-947`,
     `:1000-1004`, `:1057-1061`).
6. `org-node/src/lib.rs`: drop `chain_read` and its re-export (`:17-20`),
   `ChainOps`, `MockChainOps` (`:35-38`), `SubxtChainOps` (`:39-40`),
   `ChainReader` (`:48`) from the re-exports; gate `pub mod preflight`
   (`:29-30`) on `transport` instead of `app`; add `Prepared` and
   `PendingReceive` to the service re-exports and `OrgMembersError` to the
   `org_members` re-export line (`:61`).
7. **As amended 2026-10-07 at T5 (dispatcher's decision), restated
   2026-10-08:** delete `org-node/src/chain_read.rs`,
   `org-node/tests/chain_read_state.rs` and `org-node/tests/common/` (the
   chopsticks harness; org-io has its copy since T5) with `git rm`; keep a
   transport-only preflight, as follows.
   - `org-node/src/preflight.rs`: delete `check_chain_live`,
     `check_contract` and the `subxt` imports; keep `CheckResult`,
     `check_transport` and `render`; `#![cfg(feature = "transport")]`.
   - `org-node/src/bin/preflight.rs`: reads only `ODS_DEVICE_SEED` and
     `ODS_TRANSPORT`; delete `h160_from_env`, the chain section and the
     `ODS_CHAIN_WS`, `ODS_CONTRACT_H160` and `ODS_ADMIN_H160` lines of its
     doc; `#![cfg(feature = "transport")]`. It decoded the seed with the
     `hex` crate, which step 8 removes: decode the 64 hex characters (after
     at most one `0x`) with a small local function over
     `u8::from_str_radix` on each pair, refusing any other length or a
     non-hex character with today's message "ODS_DEVICE_SEED must be
     32-byte hex".
   - `org-node/tests/preflight.rs`: keep `transport_loopback_check_passes`
     and `transport_networked_check_is_bounded`; delete
     `chain_live_check_passes_with_miner_and_fails_when_stalled` (org-io
     has it), `mod common` and its imports; `#![cfg(feature = "transport")]`.
   - S4, which moves the iroh binding into org-io with the seed from the OS
     keychain, moves this check with it.

   *The original text of this step, kept for the record:* "Delete
   `org-node/src/chain_read.rs`, `org-node/src/preflight.rs`,
   `org-node/src/bin/preflight.rs`, `org-node/tests/chain_read_state.rs`
   and `org-node/tests/preflight.rs`." **Amended 2026-10-07 at T5 (dispatcher's decision):** T5 moved only the
   chain checks, because org-io's LLR-3zdw8v forbids `SigningKeypair` and
   `DeviceSeed` in org-io's source, and the transport check needs both.
   So org-node keeps a transport-only preflight until S4: delete
   `org-node/tests/chain_read_state.rs` and `org-node/src/chain_read.rs`;
   cut `org-node/src/preflight.rs` and `org-node/src/bin/preflight.rs` down
   to the transport check (no chain checks, no `ODS_CHAIN_WS`,
   `ODS_CONTRACT_H160` or `ODS_ADMIN_H160`; the bin reads only
   `ODS_DEVICE_SEED`), gated on `transport`, not on the removed `chain`
   feature; keep the two transport tests in `org-node/tests/preflight.rs`
   and delete its chain test (org-io has it). S4, which moves the iroh
   binding into org-io with the seed from the OS keychain, moves this check
   with it.
8. `org-node/Cargo.toml`: delete the `chain` feature (`:14`), `"chain"` from
   `app` (`:18`; `app = ["transport", "dep:chacha20poly1305", "dep:argon2"]`),
   the optional `subxt`, `on-chain-client`, `async-trait`, `hex` lines
   (`:45`, `:47-49`; keep `tokio` `:46`, which `transport` names), the
   chopsticks dev dependencies (`:70-78`: `jsonrpsee`, `libc`,
   `async-trait`, `on-chain-client`, `subxt`) and their comment — but keep
   `serde_json` (`:74`), with its comment rewritten to name
   `tests/encoding_golden.rs`, which uses it — and the `[[test]]` block
   `chain_read_state` (`:178-182`, with its comment). The `[[bin]]
   preflight` block (`:83-88`) and the `[[test]] preflight` block
   (`:125-128`) stay, with `required-features = ["transport"]` (was
   `["app"]`) and the bin's comment restated. (`async-trait` as a dev
   dependency is used today only by the `ChainOps` impls in
   `tests/support/mod.rs`, `tests/organisation_key.rs:338` and
   `tests/service_lifecycle.rs:156`, all removed by step 9.) If `cargo test`
   then reports an unused-crate or missing-crate error in a remaining test,
   restore only that dev dependency, with a comment naming the test.
9. Tests. The mechanical rewrite, file by file; keep every `verifies:` line
   unless a step below says otherwise. S3a's `tests/support/mod.rs` already
   routes every receive through helpers (`spawn_receive`,
   `spawn_self_delete`, `deliver_to_*`, `deliver_from_to_*`) and counts
   reads with `CountingChain` (`:497-562`), so the rewrite is mostly there:
   - `tests/support/mod.rs`: `MockChainOps` → `org_node::test_fixtures::ChainSlots`;
     `OrgService::new(store, Box::new(…))` → `OrgService::new(store)`.
     `CountingChain` stays, with its name and its switches (`reads`,
     `hide`, `fail_reads`), but as a test-side state source over a
     `ChainSlots` rather than a `ChainOps`: its `read_state` becomes
     `pub fn read(&self, org_id: OrgId) -> Result<Option<OrgState>, OrgNodeError>`
     (count, then the failing / hidden / slot answer), and
     `CountingChain::over(slots: ChainSlots)`. `Setup` keeps
     `chain: ChainSlots` and gains `view_a: CountingChain` and
     `view_b: CountingChain` (over the same slots); `setup_over_both`
     (`:371-404`) goes; `setup_counted` makes `view_b` its own counter and
     returns it; `setup_counting` gives A and B one shared counter. The
     receive helpers take the view of the receiving node
     (`spawn_receive(svc_b, view_b.clone(), &kp)`,
     `deliver_to_receive(&mut svc, &view, msg)`, and so on) and run the
     sequence org-io runs:

```rust
/// What org-io does on a receive: transport, chain-free phase, the state, apply.
pub async fn receive_and_verify<R: rand_core::RngCore + rand_core::CryptoRng>(
    svc: &mut OrgService, chain: &CountingChain, rng: &mut R,
) -> Result<ReceiveOutcome, OrgNodeError> {
    let (sender, message) = svc.receive_message().await?;
    match svc.prepare_receive(sender, message)? {
        Prepared::Done(outcome) => Ok(outcome),
        Prepared::NeedsChain(pending) => {
            let state = chain.read(pending.org_id())?;
            svc.apply_receive(rng, pending, state)
        }
    }
}

pub async fn receive_and_self_delete_if_revoked<R: rand_core::RngCore + rand_core::CryptoRng>(
    svc: &mut OrgService, chain: &CountingChain, rng: &mut R,
) -> Result<SelfDeleteOutcome, OrgNodeError> {
    let (sender, message) = svc.receive_message().await?;
    match svc.prepare_self_delete(sender, message)? {
        Prepared::Done(outcome) => Ok(outcome),
        Prepared::NeedsChain(pending) => {
            let state = chain.read(pending.org_id())?;
            svc.apply_self_delete(rng, pending, state)
        }
    }
}
```

     A read is made only when `prepare_*` returned `NeedsChain`, so every
     existing "without a chain read" assertion (`reads() == 0`) keeps its
     meaning: the chain-free phase refused or finished. The story helpers
     `found`, `admit`, `revoke`, `revoke_and_tell` take a `&CountingChain`,
     write through its slots (`apply_genesis`, `apply_update`) and pass
     `chain.read(org_id)?` to the commits.
   - every other test file that drives `OrgService` (`admission_sender.rs`,
     `commit_paths.rs`, `expected_admission.rs`, `organisation_key.rs`,
     `persona_records.rs`, `receive_chain_reads.rs`, `reconcile.rs`,
     `service_lifecycle.rs`, `service_stories.rs`): `MockChainOps` →
     `ChainSlots`; `svc.commit_genesis(rng, pid, org, proxy).await` →
     `svc.commit_genesis(rng, pid, org, proxy, slots.get(&org))`;
     `svc.commit_update(rng, org).await` → `svc.commit_update(rng, org, slots.get(&org))`;
     the helpers gain the view argument; tests of a commit against a chain
     with no slot pass `None`.
   - Tests whose subject was the read itself leave org-node; each is
     replaced in org-io (T7), because a failed read or a refused parse
     never reaches org-node now:
     - `commit_paths.rs:235` `a_failed_chain_read_refuses_commit_genesis`
       (LLR-ewkg85): delete; org-io's
       `a_write_whose_read_back_fails_commits_nothing_and_says_the_write_executed`
       (LLR-9r2bxd) replaces it.
     - `commit_paths.rs:399` `commit_update_refuses_when_the_chain_fails_or_is_silent`:
       keep the silent half (`commit_update(…, None)` → `OrgNotOnChain`,
       renamed `commit_update_refuses_when_the_chain_holds_no_state`);
       delete the failing half; org-io's
       `an_update_whose_read_back_fails_commits_and_sends_nothing`
       (LLR-9r2bxd) replaces it.
     - `organisation_key.rs:392` `a_receive_against_a_state_with_an_invalid_key_commits_nothing`
       and its `InvalidKeyChain` (`:333-345`): delete; org-io's
       `a_refused_parse_applies_nothing` (LLR-7pj5af) replaces it; REQ-8jb4ny
       keeps `:349` and `:374`.
     - `verify_against_chain.rs` (below) and `service_lifecycle.rs` (below).
   - `tests/reconcile.rs`: `:216`, `:220`
     `s.svc_a.reconcile(&mut OsRng, s.org_id).await` →
     `s.svc_a.reconcile(&mut OsRng, s.org_id, s.chain.get(&s.org_id))`;
     `the_service_reconcile_refuses_an_unheld_organisation_without_a_chain_read`
     (`:228`) becomes `…_refuses_an_unheld_organisation_whatever_state_it_is_given`:
     `OrgNotHeld` for `Some(state)` and for `None`, annotation unchanged
     (LLR-gr8x3r). The pure `reconcile::reconcile` tests and
     `tests/revocation.rs` (pure `accept`, `check_notice`,
     `check_acknowledgement`, `notices_for`) do not change.
   - `tests/transport_handshake.rs:120-129`, `tests/transport_networked.rs:145-157`:
     `MockChain` → `Some(OrgState { … })`.
   - `tests/verify_against_chain.rs`: `MockChain` → an `Option<OrgState>`
     (`chain_at(org, root, epoch)` returns `Some(OrgState { … })`; an empty
     chain is `None`); delete `FailingChain` and
     `a_chain_read_that_fails_is_refused_as_chain_not_as_absence` (`:119-170`;
     moved in substance to org-io's `chain_read_rule` in T5);
     `rejects_when_org_absent_from_chain` (`:107`) already reads
     `// verifies: REQ-bvh8v6, LLR-8m99q2` since T5.
   - `tests/fuzz_verify_against_chain/fuzz_target.rs`: pass the state value.
   - `tests/service_lifecycle.rs`: the test that verifies LLR-65py3d
     (`:152`, `the_seam_is_a_trait_object_a_substitute_can_stand_in_for`,
     with its `ReadOnly` `ChainOps` impl) becomes: build two services over
     one `ChainSlots`, commit on one with `slots.get(&org)`, and assert the
     other sees the same slot (LLR-hg3xzf) — and LLR-65py3d's annotation
     moves to the absence test of step 1.
10. Run `cargo test -p org-node --features app,test-support --lib --test …`
    (the `verify_commands` line minus `--test chain_read_state`),
    `cargo build -p org-node --features transport --bin preflight`,
    `cargo test -p org-node --features transport --test preflight transport_loopback_check_passes`,
    and `cargo clippy -p org-node --lib --bins --features app,test-support -- -D warnings`
    (`--all-targets` stays red on four older targets, PR-6f3qku):
    green. Run the absence test: green. Paste.
11. Ledgers (dated amendments in the files that define the items, each
    beginning `*Amended 2026-10-0x (ruling B, change worktree-org-io-create).*`):
    SDD-ueh4tm (`decomposition.md:1105`), LLR-65py3d (`:1120`), LLR-hg3xzf
    (`:1126`), LLR-ryzr8m (`:1130`), LLR-8m99q2 (`:569`), SDD-pa6p7w
    (`:602`), SDD-8cpyfa (`:1656`), LLR-379hnv (`:2309`), LLR-3q63zv
    (`:2271`), LLR-9ew26y and LLR-s8xp7m (`2026-10-06-chain-authority.md:442`,
    `:451`; the design notes at `:64-70`), LLR-ewkg85 (`:511`), LLR-xn5pwc,
    LLR-ba2ejp, LLR-38e2kn, LLR-pt32fx (`2026-10-07-org-key-pair.md:275`,
    `:284`, `:296`, `:311`), LLR-mmdu38 (`2026-10-04-type-safety.md:185`,
    cache clause retired); S3a's LLR-dc45ur, LLR-2r2fha, LLR-kzgjz8,
    LLR-3aysup and the section intro (`2026-10-07-commit-workflow.md:349`,
    `:481`, `:497`, `:505`, `:472-479`), REQ-ztdza4
    (`requirements/2026-09-09-verify-and-commit.md:115`), REQ-ea4qs5
    (`requirements/2026-10-07-commit-workflow.md:100`) and RC-u7kdam
    (`risk/2026-10-07-commit-workflow.md:27`) — the wording for each is in
    §6 of this plan. Name org-io's items in prose only (org-node does not
    depend on org-io). PR-k2xxaq: `status: resolved` with the resolution
    fields and a dated paragraph ("deleted with `OnChainReader`; the absence
    is LLR-mn5c2q's test"). *Note (owner ruling on PR-zf924s, 2026-10-08,
    hybrid):* classify each amendment. Where the item's meaning is
    unchanged (its chain input now arrives as a value, but it says the same
    thing), use a dated in-place note. Where its meaning changes, mint a new
    superseding ID with a `supersedes:` / `superseded-by:` pair, and move the
    tests' `verifies:` to the new ID. List the classification in the task
    report.
12. `org-node/.guardrails/config.yaml`: drop `--test chain_read_state` from
    `:244`; rewrite the preflight comment at `:48` (still excluded: its
    networked check depends on relay reachability; no chopsticks any more);
    drop `on-chain-client` from `depends_on` (`:287`) and restate the
    comment above it (`:279-284`) with a dated note. (`.github/workflows/rust.yml`
    names no `chain_read_state`, `:75-79`: nothing to drop there.)
13. `GR_CONFIG=org-node/.guardrails/config.yaml .guardrails/scripts/check-trace.sh`:
    exit 0, PR-k2xxaq no longer open. Commit:
    `refactor(org-node): values in, values out — no chain seam, no chain feature (ruling B; resolves PR-k2xxaq)`.

### T7 — org-io: the handle, submission and the receive sequence

**Files touched:** org-io/Cargo.toml, Cargo.lock, org-io/src/lib.rs, org-io/src/submit.rs, org-io/src/custody.rs, org-io/src/test_support.rs, org-io/tests/submit_flow.rs, org-io/tests/receive_order.rs, org-io/tests/common/mod.rs, org-io/tests/common/receive.rs, org-io/tests/signatory_key.rs, org-io/tests/absences.rs, org-io/.guardrails/config.yaml, org-io/docs/architecture/DRAFT-worktree-org-io-create-org-io.md
**Parallel:** no (serial, after T6)
**IDs verified:** LLR-qhjp6g, LLR-be3zv9 (tests moved; definitions move in T9), LLR-9r2bxd, LLR-7pj5af (amended here for S3a), LLR-rgdx22 (the `connect` half), LLR-u2pk5y

*Reconciled 2026-10-08 with S3a and T6's shape:* the receive sequence passes
the sender, handles `Prepared::Done` with no read, and is tested for the
three message kinds; the read-back is tested on both submit operations;
the carried `#[allow(dead_code)]` on `SignatoryKey::keypair()` goes here.

0. Amend LLR-7pj5af in `org-io/docs/architecture/DRAFT-worktree-org-io-create-org-io.md:41`
   (a draft of this change, so rewrite it in place with a dated note, no
   new ID) to:

```markdown
  **LLR-7pj5af**: `OrgIo::receive_and_verify` and
  `OrgIo::receive_and_self_delete_if_revoked` take one message and the
  Device that sent it from org-node's `receive_message`, and hand both to
  org-node's chain-free phase (`prepare_receive`, `prepare_self_delete`).
  When that phase refuses, they return its error unchanged and make no
  `StateReader::read_state` call; when it finishes without a chain state
  (`Prepared::Done`: an acknowledgement, checked against the store alone),
  they return its outcome and make no call; only when it returns a pending
  value (`Prepared::NeedsChain`) do they call `read_state` exactly once,
  for the Organisation the pending value names, then hand the state read
  (or `None`) to org-node's apply phase (`apply_receive`,
  `apply_self_delete`). When `read_state` fails, or the parse refuses the
  state, they return that error and call no apply phase, so nothing is
  committed or written.
  satisfies: REQ-tg9zrn
```

   and its `Test:` paragraph to: normal: a message about a held
   Organisation is read once and committed; an acknowledgement is decided
   with zero reads; abnormal: an unheld revocation, a held Organisation's
   update from a Device its record does not list, and a first admission
   not expected each make zero reads; a failed read and a refused parse
   each leave the record unchanged. The reads are counted by
   `test_support::FakeChain`.

1. `org-io/src/test_support.rs` (`#[cfg(feature = "test-support")] pub mod test_support;`
   in `lib.rs`): `FakeChain`, the app's `FakeWriter`
   (`app/src-tauri/tests/support/mod.rs:13-71`) generalised: it holds a
   `ChainSlots`, the `failing`/`hanging` switches for writes, a
   `read_failing` switch, a `parse_refusing` switch and an `AtomicUsize`
   read counter (`pub fn reads(&self) -> usize`), signatory-set slots for T8
   (`original_accounts: Mutex<HashMap<OrgId, AccountId>>`,
   `delegates: Mutex<HashMap<[u8; 32], Vec<AccountId>>>` and a
   `signatory_read_failing` switch), and implements `StateReader` (counts,
   then `Err(OrgNodeError::Chain("node unreachable"))` when `read_failing`,
   `Err(OrgNodeError::InvalidOrgPublicKey)` — what `read_org_state` returns
   for a refused parse — when `parse_refusing`, else
   `Ok(slots.get(&org_id))`) and `ChainWriter` (today's
   `FakeWriter::genesis`/`update`). `Clone` (all state behind `Arc`). Also
   `pub fn org_io_over(store: PersonaStore, chain: &FakeChain) -> OrgIo`.
2. `org-io/src/submit.rs`: copy `app/src-tauri/src/submit.rs` with these
   changes only: `found_organisation` and `submit_commit_send` take
   `reader: &dyn StateReader` after `writer`, and after the bounded write
   they read the state (LLR-9r2bxd):

```rust
    let chain_state = reader
        .read_state(org_id)
        .await
        .map_err(|error| format!("the chain write executed, but reading it back failed; nothing committed: {error}"))?;
    svc.commit_genesis(rng, persona_id, org_id, proxy, chain_state).map_err(|error| error.to_string())?;
```

   (and `svc.commit_update(rng, org_id, chain_state)` in
   `submit_commit_send`, followed, as S3a left it, by
   `svc.send_update(&outcome, recipient, peer_addr)` — `send_update` takes
   the `CommitOutcome` since S3a, so a removed Device is sent its notice);
   `OnChainWriter.signatory` becomes a private
   `signatory: SignatoryKey` (`crate::custody`), used as
   `self.signatory.keypair()`; `OnChainWriter` gets a `pub(crate) fn new`.
   `NOT_CONFIGURED`'s text is unchanged. Remove the
   `#[allow(dead_code)] // first caller: the chain write's submission (T7)`
   on `SignatoryKey::keypair()` (`org-io/src/custody.rs:153`, carried from
   T2): this is its first caller.
3. `org-io/src/lib.rs`, the handle (SDD-z3ychz):

```rust
pub struct ChainSettings {
    pub ws_url: String,
    pub contract_h160: [u8; 20],
    /// `ODS_COSIGNER_PUB` as the app read it; parsed here (LLR-9fy622).
    pub co_signer: Option<String>,
}

pub struct OrgIo {
    service: OrgService,
    reader: Box<dyn StateReader>,
    writer: Box<dyn ChainWriter>,
    signatory_set: Box<dyn SignatorySetReader>, // T8
    own_account: Option<AccountId>,
    co_signers: Vec<AccountId>,
}

impl OrgIo {
    /// The chain is not configured: every read and write refused.
    pub fn not_configured(service: OrgService) -> Self;
    /// Connect: under `dev-seed`, the seed from the environment and the
    /// chain from `settings`; without it, `Err` with the development-only
    /// message (LLR-rgdx22). The read and the write are built together, as
    /// today ("Still open" 1; owner ruling of 2026-10-07: coupled in S2,
    /// split in S8).
    pub async fn connect(service: OrgService, settings: ChainSettings) -> Result<Self, String>;
    #[cfg(feature = "test-support")]
    pub fn for_test(service: OrgService, chain: &test_support::FakeChain) -> Self;
    /// Transitional (S4 narrows): org-node's queries and builders.
    pub fn node(&self) -> &OrgService;
    pub fn node_mut(&mut self) -> &mut OrgService;
    pub async fn found_organisation<R: RngCore + CryptoRng + Send>(&mut self, rng: &mut R, persona_id: &PersonaId) -> Result<OrgId, String>;
    pub async fn submit_commit_send<R: RngCore + CryptoRng + Send>(&mut self, rng: &mut R, update: &ProvisionalUpdate,
        recipient: DevicePublicKey, peer_addr: Option<iroh::EndpointAddr>) -> Result<CommitOutcome, String>;
    pub async fn receive_and_verify<R: RngCore + CryptoRng>(&mut self, rng: &mut R) -> Result<ReceiveOutcome, OrgNodeError>;
    pub async fn receive_and_self_delete_if_revoked<R: RngCore + CryptoRng>(&mut self, rng: &mut R) -> Result<SelfDeleteOutcome, OrgNodeError>;
}
```

   `connect` without `dev-seed` returns
   `Err("chain not configured: the signing seed (ODS_ADMIN_SEED) is read only by development builds (dev-seed)".into())`
   before any connection; with it, `signatory_from_environment()?` (`None`
   → `Err("chain not configured: set ODS_CHAIN_WS, ODS_CONTRACT_H160, ODS_ADMIN_SEED")`,
   today's text), `settings.co_signer.as_deref().map(parse_co_signer).transpose()`,
   then `connect::connect`, then `OnChainStateReader`, `OnChainWriter`,
   exactly as `app/src-tauri/src/state.rs:305-341` does. The receive pair is
   LLR-7pj5af:

```rust
    pub async fn receive_and_self_delete_if_revoked<R: RngCore + CryptoRng>(&mut self, rng: &mut R)
        -> Result<SelfDeleteOutcome, OrgNodeError> {
        // The sender is a value from here on (S3a's member-sender rule runs in prepare).
        let (sender, message) = self.service.receive_message().await?;
        match self.service.prepare_self_delete(sender, message)? {
            // An acknowledgement: decided against the store alone, no read.
            Prepared::Done(outcome) => Ok(outcome),
            Prepared::NeedsChain(pending) => {
                let chain_state = self.reader.read_state(pending.org_id()).await?;
                self.service.apply_self_delete(rng, pending, chain_state)
            }
        }
    }
```

   `receive_and_verify` is the same over `prepare_receive`/`apply_receive`.
   The order the S3a rules need is org-node's, inside `prepare_*` (T6's
   table): the sender rule before any read; for a notice, `check_notice`
   then the sender rule, and only then the read and `accept`; a first
   admission from any sender, gated by the expectation before the read and
   by the chain after it; an acknowledgement's sender equal to the Device it
   names, checked before the signature, and never a read. org-io's part is
   to read only on `NeedsChain`, once.

   `org-io/Cargo.toml`: add to `[dependencies]` `iroh = "0.98"` and
   `rand_core = "0.6"` (org-node's versions; `submit_commit_send` takes an
   `iroh::EndpointAddr`), and to `[dev-dependencies]`
   `tokio = { version = "1", features = ["rt-multi-thread", "macros", "time", "test-util"] }`
   (the paused clock of LLR-be3zv9's test) and `rand = "0.8"` (`OsRng`, as
   the app's tests use).
4. Move the five submit tests: create `org-io/tests/submit_flow.rs` from
   `app/src-tauri/tests/submit_flow.rs` lines 1-165 (the five tests
   annotated LLR-qhjp6g / LLR-be3zv9; not the two LLR-q225ws tests), with
   `verifies:` lines unchanged, rewritten onto the handle:
   `service(tag, &chain)` → `OrgIo::for_test(OrgService::new(store), &chain)`;
   `found_organisation(&mut a, &writer, &mut OsRng, &pid)` →
   `a.found_organisation(&mut OsRng, &pid)`; `submit_commit_send(&mut a, &writer, …)` →
   `a.submit_commit_send(&mut OsRng, …)`; `a.list_orgs()` → `a.node().list_orgs()`;
   `b.receive_and_verify(…)` → the handle's. Do not delete the app's copies
   yet (T9 does, with the app's switch). Then add the new case:

```rust
// LLR-9r2bxd: a write that executed, then a read-back that failed: nothing
// committed, the update kept, both facts reported.
// verifies: LLR-9r2bxd
#[tokio::test]
async fn a_write_whose_read_back_fails_commits_nothing_and_says_the_write_executed() {
    let chain = FakeChain::new();
    let mut a = handle("read-back-fails", &chain);
    let pid = persona(&mut a, "alice");
    chain.read_failing.store(true, Ordering::SeqCst);
    let err = a.found_organisation(&mut OsRng, &pid).await.unwrap_err();
    assert!(err.contains("write executed") && err.contains("nothing committed"), "{err}");
    assert!(a.node().list_orgs().is_empty(), "nothing committed");
    assert_eq!(a.node().genesis_provisional_updates(&pid).len(), 1, "the update is kept");
    assert_eq!(chain.geneses(), 1, "the write did execute");
}
```

   and extend `founding_writes_the_chain_then_commits` with
   `assert_eq!(chain.reads(), 1, "one read between the write and the commit");`,
   annotated `// verifies: LLR-qhjp6g, LLR-be3zv9, LLR-9r2bxd`. Add the same
   case for the update path, replacing org-node's deleted failing half of
   `commit_update_refuses_when_the_chain_fails_or_is_silent` (T6):

```rust
// LLR-9r2bxd on `submit_commit_send`: the update write executed, the
// read-back failed: nothing committed, nothing sent, the update kept.
// verifies: LLR-9r2bxd
#[tokio::test(flavor = "multi_thread")]
async fn an_update_whose_read_back_fails_commits_and_sends_nothing() {
    // A founds (read-back succeeds), builds an admission of joiner(0x41, "bob"),
    // then sets chain.read_failing and calls a.submit_commit_send(&mut OsRng,
    // &update, bob_device, Some(dead_addr)). Assert: Err containing
    // "write executed" and "nothing committed"; the record's epoch is still 1;
    // a.node().provisional_updates(org).len() == 1; chain slot at epoch 2
    // (the write did execute); nothing was sent (the dead address was never
    // dialled: the error is the read-back's, not a send's).
}
```

   The body follows `a_failed_update_write_neither_commits_nor_sends`
   (`app/src-tauri/tests/submit_flow.rs:118`), with the write succeeding
   and the read failing instead.
5. Write `org-io/tests/receive_order.rs` (LLR-7pj5af), built on the
   `admission` scenario of `submit_flow.rs` (A founds and admits B; B
   receives through its handle over the same `FakeChain`):

```rust
// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn a_held_organisations_message_is_read_once_then_applied() { /* B receives A's admission; assert B committed epoch 2 and chain.reads() rose by exactly 1 for B's receive */ }

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn a_message_the_chain_free_phase_refuses_causes_no_read() {
    // B does NOT call expect_admission, so A's admission is a first
    // admission B does not expect: AdmissionNotExpected, with zero reads.
}

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_read_applies_nothing() {
    // B expects the admission; chain.read_failing is set before B receives:
    // Err(Chain("node unreachable")), B holds no record, its expectation stays.
}

// Added 2026-10-08 (S3a's three kinds and the member-sender rule).

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn a_refused_parse_applies_nothing() {
    // As above with chain.parse_refusing: Err(InvalidOrgPublicKey), B holds no
    // record, its expectation stays. (Replaces org-node's
    // a_receive_against_a_state_with_an_invalid_key_commits_nothing, T6.)
}

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn an_update_from_a_device_the_record_does_not_list_causes_no_read() {
    // B holds the record (admitted as above). A builds and submits an
    // admission of C; its Organisation information is captured by a sink and
    // re-sent to B from a relay Device no record lists: Err(SenderNotListed),
    // B's counter unchanged.
}

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn an_unheld_revocation_causes_no_read() {
    // A revocation notice for an Organisation B does not hold, delivered to
    // B's receive_and_self_delete_if_revoked: Err(RevocationNotHeld), zero reads.
}

// verifies: LLR-7pj5af
#[tokio::test(flavor = "multi_thread")]
async fn an_acknowledgement_is_decided_with_no_read() {
    // A revokes B; B's removal produces B's signed acknowledgement; it is
    // delivered from B's Device to A's receive_and_verify:
    // Ok(ReceiveOutcome { acknowledged: Some(_), .. }), A's counter unchanged.
}
```

   The relay and sink helpers are org-node's test-support ones
   (`tests/support/mod.rs`'s `spawn_recv_one`, `deliver_from`), copied into
   `org-io/tests/common/receive.rs` with the endpoint calls going through
   `org_io::node::transport` (the transitional re-export; a `DeviceSeed` in
   a test file is allowed, since LLR-3zdw8v's scan reads `org-io/src` only),
   and `pub mod receive;` added to `org-io/tests/common/mod.rs`; the
   acknowledgement and notice values come from the outcomes the handles
   return (`CommitOutcome.revocations`, `SelfDeleteOutcome::SelfDeleted {
   acknowledgements }`), never built by hand.

   Write the bodies in full from `submit_flow.rs`'s
   `an_admission_is_written_then_committed_then_sent` (spawn B's receive,
   then A's `submit_commit_send` to `Some(addr_b)`); count reads as
   `let before = chain.reads();` immediately before B's receive is spawned
   and compare after the task joins; A's own read (its submission) happens
   before B's message arrives, so take the count after A's submit returns:
   use a second `FakeChain` handle for B whose counter is separate
   (`FakeChain::sharing_slots_with(&chain)` — add that constructor: same
   `ChainSlots`, own counter and switches).
6. LLR-rgdx22's `connect` half, in `org-io/tests/signatory_key.rs`:

```rust
// verifies: LLR-rgdx22
#[cfg(not(feature = "dev-seed"))]
#[tokio::test]
async fn a_build_without_dev_seed_reads_no_seed_and_reports_why() {
    let dir = std::env::temp_dir().join(format!("org-io-no-dev-seed-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let service = org_io::node::service::OrgService::new(org_io::node::store::PersonaStore::open(dir.join("store.bin"), "pw").unwrap());
    let settings = org_io::ChainSettings { ws_url: "ws://127.0.0.1:1".into(), contract_h160: [0u8; 20], co_signer: None };
    let refused = org_io::OrgIo::connect(service, settings).await.err().unwrap();
    assert_eq!(refused, "chain not configured: the signing seed (ODS_ADMIN_SEED) is read only by development builds (dev-seed)");
}
```

   (The address is unreachable on purpose: a build that tried to connect
   would fail differently.)
7. LLR-u2pk5y, in `org-io/tests/absences.rs`:

```rust
// verifies: LLR-u2pk5y
#[test]
fn the_handle_hands_out_no_key() {
    let lib = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs")).unwrap();
    let submit = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/submit.rs")).unwrap();
    for line in lib.lines().filter(|line| line.trim_start().starts_with("pub fn") || line.trim_start().starts_with("pub async fn")) {
        for needle in ["SignatoryKey", "SeedBytes", "subxt_signer"] {
            assert!(!line.contains(needle), "a public OrgIo method names {needle}: {line}");
        }
    }
    let org_io_struct = &lib[lib.find("pub struct OrgIo").unwrap()..];
    let org_io_body = &org_io_struct[..org_io_struct.find('}').unwrap()];
    assert!(!org_io_body.contains("pub "), "OrgIo has no public field");
    assert!(!submit.contains("pub signatory"), "OnChainWriter's key field is private");
    for (text, holder) in [(&lib, "OrgIo"), (&submit, "OnChainWriter")] {
        let position = text.find(&format!("pub struct {holder}")).unwrap();
        let attributes = text[..position].lines().rev().take_while(|line| line.trim_start().starts_with("#[") || line.trim_start().starts_with("///"));
        assert!(attributes.into_iter().all(|line| !line.contains("Debug")), "{holder} derives Debug");
    }
}
```

8. Run (red first for each new test, as the conventions say), then the org-io
   verify lines with the new targets added to the first line
   (`--features test-support` now, since `submit_flow`, `receive_order` need
   `FakeChain`: `cargo test -p org-io --features test-support --lib --test …`),
   and `cargo clippy -p org-io --all-targets --features test-support,dev-seed -- -D warnings`.
   Add `[[test]]` entries with `required-features = ["test-support"]` for
   `submit_flow` and `receive_order`.
9. Commit: `feat(org-io): the handle, submission with its read-back, and the receive sequence (LLR-9r2bxd, LLR-7pj5af, LLR-u2pk5y)`.

### T8 — org-io: the own-admin check

**Files touched:** org-io/src/signatory.rs, org-io/src/lib.rs, org-io/src/test_support.rs, org-io/tests/signatory_rule.rs, org-io/Cargo.toml, org-io/.guardrails/config.yaml
**Parallel:** no (serial, after T3 and T7)
**IDs verified:** LLR-qhyc3n, LLR-vyd5d3, LLR-c9fyun

*Checked 2026-10-08 against S3a:* unaffected (S3a touched no org-io or
on-chain-client file; `OrgRegistryClient::original_account` /
`proxy_delegates`, `on-chain-client/src/client.rs:220`, `:240`, and
`multi_account_id`, `write/multisig.rs:15`, are as T3 left them).

*Scope (owner ruling of 2026-10-07, "Still open" item 2):* this task delivers
the org-io function and its tests only. Nothing outside org-io's tests calls
`OrgIo::is_own_admin` in S2; the app is not wired to it (T9 adds no command
for it). Its first caller is S3b-io.

1. Write `org-io/tests/signatory_rule.rs` (compile-red):

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The own-admin check (SDD-f4khqn): the pure rule, then the handle over FakeChain.

use on_chain_client::write::multisig::multi_account_id;
use on_chain_client::write::AccountId;
use org_io::signatory::{admin_status, controller_of, AdminStatus};

const OWN: AccountId = AccountId([1u8; 32]);
const CO_SIGNER: AccountId = AccountId([2u8; 32]);
const STRANGER: AccountId = AccountId([3u8; 32]);
const PROXY: AccountId = AccountId([9u8; 32]);

// verifies: LLR-qhyc3n
#[test]
fn the_controller_is_the_own_account_alone_or_the_threshold_one_multisig() {
    assert_eq!(controller_of(OWN, &[]), OWN);
    let expected = multi_account_id(&[OWN, CO_SIGNER], 1);
    assert_eq!(controller_of(OWN, &[CO_SIGNER]), expected);
    assert_eq!(controller_of(OWN, &[CO_SIGNER, OWN]), expected, "the own account is not counted twice");
}

// verifies: LLR-vyd5d3
#[test]
fn admin_exactly_when_the_mapped_proxy_lists_the_controller() {
    let controller = controller_of(OWN, &[CO_SIGNER]);
    assert_eq!(admin_status(controller, Some(PROXY), &[STRANGER, controller]), AdminStatus::Admin);
    assert_eq!(admin_status(controller, None, &[controller]), AdminStatus::NotAdmin, "unmapped H160");
    assert_eq!(admin_status(controller, Some(PROXY), &[]), AdminStatus::NotAdmin, "no delegates");
    assert_eq!(admin_status(controller, Some(PROXY), &[STRANGER]), AdminStatus::NotAdmin);
    assert_eq!(admin_status(controller, Some(PROXY), &[OWN]), AdminStatus::NotAdmin,
        "with co-signers configured, the own account alone is not the controller");
}
```

   plus three `#[tokio::test]`s over `OrgIo::for_test` and `FakeChain`
   (LLR-c9fyun): no key configured → `Err(OwnAdminError::NotConfigured)`
   (give `for_test` an `own_account: None` default and a
   `with_own_account(AccountId, Vec<AccountId>)` builder); a failing
   signatory read → `Err(OwnAdminError::Read(_))`; a configured account
   listed as delegate → `Ok(AdminStatus::Admin)`, and the delegates are read
   only when the original account is `Some` (count FakeChain's signatory
   reads).
2. Implement `org-io/src/signatory.rs`: `AdminStatus { Admin, NotAdmin }`,
   `OwnAdminError { NotConfigured, Read(String) }` (Display: "this node holds
   no signatory key" / "the signatory-set read failed: {0}"),
   `controller_of` (dedup `own` out of `co_signers`, then
   `multi_account_id(&[own, co_signers…], 1)` when any remain, else `own`),
   `admin_status`, the trait

```rust
#[async_trait::async_trait]
pub trait SignatorySetReader: Send + Sync {
    async fn original_account(&self, org_id: OrgId) -> Result<Option<AccountId>, String>;
    async fn proxy_delegates(&self, account: AccountId) -> Result<Vec<AccountId>, String>;
}
```

   with the production impl over `OrgRegistryClient::original_account` /
   `proxy_delegates` (T3; SDD-z85ux9's shell — add it to that item's code
   list), `FakeChain`'s impl, and `OrgIo::is_own_admin(&self, org_id)`.
   `OrgIo::connect` builds the production reader from the same registry
   client; `not_configured` uses one that returns `Err("chain not configured")`.
3. Run, add `--test signatory_rule` (+ `[[test]]`, `test-support`) to the
   first verify line; clippy. Commit:
   `feat(org-io): the own-admin check (LLR-qhyc3n, LLR-vyd5d3, LLR-c9fyun)`.

### T9 — The app depends on org-io alone

**Files touched:** app/src-tauri/Cargo.toml, app/src-tauri/Cargo.lock, app/src-tauri/src/state.rs, app/src-tauri/src/commands.rs, app/src-tauri/src/invitation.rs, app/src-tauri/src/events.rs, app/src-tauri/src/parsing.rs, app/src-tauri/src/lib.rs, app/src-tauri/src/submit.rs, app/src-tauri/tests/support/mod.rs, app/src-tauri/tests/submit_flow.rs, app/src-tauri/tests/invitation.rs, app/src-tauri/tests/ipc.rs, app/src-tauri/tests/receiver_events.rs, app/src-tauri/tests/org_id_parsing.rs, app/src-tauri/tests/state_assembly.rs, app/.guardrails/config.yaml, app/docs/requirements/2026-10-06-invitation.md, app/docs/requirements/2026-09-14-tauri-shell.md, app/docs/requirements/DRAFT-worktree-org-io-create-submission.md, app/docs/risk/2026-10-06-invitation.md, app/docs/risk/2026-09-14-app-hazards.md, app/docs/risk/DRAFT-worktree-org-io-create-submission.md, app/docs/architecture/2026-10-06-invitation.md, app/docs/architecture/2026-10-07-org-key-pair.md, app/docs/architecture/2026-10-05-decomposition.md, app/docs/architecture/soup.md, app/docs/problems/2026-09-14-app-problems.md, app/docs/problems/2026-10-06-provisional-housekeeping.md, org-io/tests/app_boundary.rs, org-io/Cargo.toml, org-io/.guardrails/config.yaml, org-io/docs/requirements/DRAFT-worktree-org-io-create-org-io.md, org-io/docs/architecture/DRAFT-worktree-org-io-create-org-io.md, org-io/docs/risk/DRAFT-worktree-org-io-create-org-io.md, .guardrails/units.yaml
**Parallel:** no (serial, after T8)
**IDs verified:** LLR-n3zmt6; moves REQ-nfr3n2, LLR-qhjp6g, LLR-be3zv9; writes REQ-m8sgjk

*Out of scope (owner ruling of 2026-10-07, "Still open" item 2):* the
own-admin check. The app does not call `OrgIo::is_own_admin`; add no
`own_admin_status` command, IPC test or app requirement for it. S3b-io wires
it. The chain read stays built together with the write, as today ("Still
open" item 1, ruled the same day): `connection_status` and REQ-e4ah9h are
unchanged.

*Reconciled 2026-10-08 with S3a.* S3a changed the app's receive handling
but not its shape: `next_outcomes` (`app/src-tauri/src/commands.rs:369`)
still calls `receive_and_self_delete_if_revoked` and matches
`SelfDeleted { org_id, .. }`, `UpdatedNotRevoked` and S3a's `Acknowledged`;
`events.rs` (`:53`, `outcomes_for_receive_error`) classifies S3a's new
`OrgNodeError` variants (`SenderNotListed`,
`AcknowledgementNotFromItsDevice`, `StaleChainState`, …). org-io's
`receive_and_self_delete_if_revoked` returns the same outcome and error
types, so both change only their `use` lines (`org_node::` →
`org_io::node::`), and the call becomes `io.receive_and_self_delete_if_revoked(&mut OsRng)`.
`tests/receiver_events.rs` names `org_members::OrgMembersError`
(`:436`) and, since S3a, `org_node::OrgId` and `org_node::Epoch`
(`:749-755`): all through `org_io::node::` (T6 adds `OrgMembersError` to
org-node's root re-exports). `tests/invitation.rs` and `tests/submit_flow.rs`
build `MockChainOps` (gone since T6): use `FakeChain`. The receiver loop
still runs only the self-delete path; one receive path is S4's (PR-kwwap5).

1. Red: `org-io/tests/app_boundary.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The app reaches org-node, the chain and the key only through org-io (LLR-n3zmt6).

use std::path::Path;

/// The manifest's dependency tables, each as text.
fn dependency_tables(manifest: &str) -> Vec<(String, String)> {
    let mut tables = vec![];
    let mut current: Option<(String, String)> = None;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            if let Some(table) = current.take() {
                tables.push(table);
            }
            if trimmed.contains("dependencies") {
                current = Some((trimmed.to_string(), String::new()));
            }
        } else if let Some((_, body)) = current.as_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    tables.extend(current);
    tables
}

// verifies: LLR-n3zmt6
#[test]
fn the_app_depends_on_org_io_and_on_no_other_unit_or_chain_library() {
    let manifest = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../app/src-tauri/Cargo.toml")).unwrap();
    let tables = dependency_tables(&manifest);
    assert!(tables.iter().any(|(name, body)| name == "[dependencies]" && body.lines().any(|line| line.starts_with("org-io "))));
    for (name, body) in &tables {
        for forbidden in ["org-node", "org-members", "on-chain-client", "person", "subxt", "subxt-signer"] {
            assert!(!body.lines().any(|line| line.starts_with(&format!("{forbidden} ")) || line.starts_with(&format!("{forbidden}="))),
                "{name} names {forbidden}");
        }
    }
}
```

   Run `cargo test -p org-io --test app_boundary`: red (`[dependencies]
   names org-node`). Paste.
2. `app/src-tauri/Cargo.toml`: remove `subxt`, `subxt-signer`,
   `on-chain-client`, `org-node` (`:29-33`) and the `org-members` dev
   dependency with its comment (`:52-57`); add
   `org-io = { path = "../../org-io" }` to `[dependencies]` and
   `org-io = { path = "../../org-io", features = ["test-support"] }` to
   `[dev-dependencies]`; add `[features] dev-seed = ["org-io/dev-seed"]`
   so a development build of the app enables the seed path
   (`cargo tauri dev --features dev-seed`; record this in `app/README` if
   one documents the run command, else in `app/AGENTS.md`). Keep
   `async-trait` only if `cargo build` needs it.
3. `app/src-tauri/src/state.rs`: `AppState.service` + `writer` →
   `pub org_io: Mutex<org_io::OrgIo>`; delete `ChainNotConfigured`,
   `build_chain_ops`, `connect_chain`, `ChainWiring`; `init` reads
   `ODS_CHAIN_WS`, `ODS_CONTRACT_H160` and `ODS_COSIGNER_PUB` (absent
   either of the first two → not configured, as today), parses the H160
   exactly as `:240-252` does today (it stays the app's, PR-5mc4d8), and
   calls `tauri::async_runtime::block_on(OrgIo::connect(service, settings))`;
   on `Ok` it records `ChainEndpoint { ws_url, contract_h160 }` from the
   values it passed (REQ-bvx4nh: what was built); on `Err(e)` it prints
   `e` as today (`:145`) and uses `OrgIo::not_configured(service)` with
   `chain_endpoint: None`. `for_test` uses `OrgIo::not_configured`. Keep
   the module doc's account of the environment, minus the seed (org-io
   reads it, under `dev-seed`).
4. `commands.rs`, `invitation.rs`, `events.rs`, `parsing.rs`: replace
   `use org_node::…` with `use org_io::node::…`; `state.service.lock()` →
   `state.org_io.lock()`; `svc.<query>` → `io.node().<query>`, `svc.<builder>`
   → `io.node_mut().<builder>`; `crate::submit::found_organisation(&mut svc, &*state.writer, …)`
   → `io.found_organisation(&mut OsRng, …)`; `revoke_and_send` and
   `admit_reply` take `&mut OrgIo` instead of `(&mut OrgService, &dyn ChainWriter)`
   and call `io.submit_commit_send(rng, &update, recipient, peer_addr)`;
   `next_outcomes` calls `io.receive_and_self_delete_if_revoked(&mut OsRng)`.
   Delete `app/src-tauri/src/submit.rs` and its `mod` line in `lib.rs`.
   Rewrite the source comments that name moved, unexported IDs
   (`invitation.rs:344` LLR-qhjp6g) to prose; REQ-nfr3n2 mentions stay
   (exported).
5. App tests: `support/mod.rs` drops `FakeWriter` (use
   `org_io::test_support::FakeChain`); `submit_flow.rs` keeps only the two
   LLR-q225ws tests (delete the five that T7 copied to org-io, and its module
   doc's REQ-nfr3n2/LLR sentence becomes "the recipient each command
   chooses (LLR-q225ws); the submission itself is org-io's"), rewritten onto
   `OrgIo::for_test`; `invitation.rs`, `ipc.rs`, `receiver_events.rs`
   (`org_io::node::OrgMembersError`), `org_id_parsing.rs`,
   `state_assembly.rs`: imports only.
6. Run the app's three verify lines: green. Run `cargo test -p org-io --test app_boundary`: green.
   Add `--test app_boundary` to org-io's first verify line.
7. Ledgers (ruling A moves; dated notes at each source naming org-io's file,
   no ID):
   - REQ-nfr3n2: cut `app/docs/requirements/2026-10-06-invitation.md:72`
     (block) into org-io's requirements draft; restate its subject as
     org-io, add `exported: yes`, keep `satisfies: derived`; move its
     assessment out of `app/docs/risk/2026-10-06-invitation.md:78-86` into
     org-io's risk draft (split the `assesses:` line at `:86`: the app keeps
     its five, org-io's gets `assesses: REQ-nfr3n2`), re-assessed for
     custody in org-io (RC-2xufsr).
   - LLR-qhjp6g, LLR-be3zv9: cut `app/docs/architecture/2026-10-06-invitation.md:192`
     and `:207` (with the note `:218-221`) into the org-io architecture
     draft under SDD-erzj3m; restate for `OrgIo::found_organisation` /
     `OrgIo::submit_commit_send` and the read-back (LLR-9r2bxd); "the bound
     is the app's" → org-io's. Add REQ-nfr3n2 to SDD-erzj3m's and
     SDD-z85ux9's `traces:`; add `OnChainWriter` to SDD-z85ux9's code list.
   - REQ-m8sgjk, in `app/docs/requirements/DRAFT-worktree-org-io-create-submission.md`:

```markdown
  **REQ-m8sgjk**: The software shall decide when an Organisation is
  founded and when a provisional update is submitted, and which Device
  receives the committed update, and shall hand each to org-io and report
  the outcome org-io returns.
  satisfies: derived
```

     with its assessment in `app/docs/risk/DRAFT-worktree-org-io-create-submission.md`
     (no hazard impact in S2: the same decisions the app made before, now
     handed to org-io; the recipient rule is PR-3ue4va's, unchanged;
     `assesses: REQ-m8sgjk`).
   - LLR-q225ws (`app/docs/architecture/2026-10-07-org-key-pair.md:32`) and
     LLR-gha5f6 (`2026-10-06-invitation.md:161`): `satisfies:` REQ-nfr3n2 →
     REQ-m8sgjk (LLR-gha5f6 keeps REQ-65xqp8); the rest of the app list
     in §6 (SDD-rmbr3t, SDD-6g3wnh, SDD-aq7m6b, SDD-k95rmp, REQ-e4ah9h,
     REQ-bvx4nh check, REQ-x3c8n2 re-addressed `expects: org-io`, RC-wzb48r,
     the hazards note, PR-5mc4d8 note, PR-924ftr, the soup rows,
     `app/.guardrails/config.yaml:106` and `depends_on: org-io` at `:187-189`).
   - `.guardrails/units.yaml`: the org-io paragraph's last sentence becomes
     "Since 2026-10-0x (S2) the app depends on org-io alone; org-node no
     longer depends on on-chain-client."
8. Checks: `check-units.sh`; `check-trace.sh` and `check-ids.sh --allow-draft-files`
   for app, org-node, org-io, on-chain-client. Expected: exit 0 for app,
   org-node, on-chain-client; org-io exit 0 too (every org-io REQ and LLR now
   has its test or a tested LLR). Fix any NON-EXPORTED-REF by rewriting the
   mention to prose. Commit:
   `feat(app): the app depends on org-io alone (LLR-n3zmt6; REQ-nfr3n2, LLR-qhjp6g, LLR-be3zv9 move to org-io)`.

### T10 — org-node: the remaining ledger and prose sweep

**Files touched:** org-node/README.md, org-node/AGENTS.md, org-node/docs/architecture/README.md, org-node/docs/architecture/soup.md, org-node/docs/architecture/2026-10-03-decomposition.md, org-node/docs/architecture/2026-10-04-type-safety.md, org-node/docs/requirements/2026-09-09-verify-and-commit.md, org-node/docs/requirements/2026-10-03-transport-binding-scope.md, org-node/docs/requirements/2026-09-06-dependency-expectations.md, org-node/docs/requirements/2026-10-06-chain-authority.md, org-node/docs/risk/2026-10-03-architecture-derived.md, org-node/docs/problems/2026-09-09-org-node-problems.md, org-node/.guardrails/config.yaml, on-chain-client/docs/requirements/2026-09-10-chain-reading.md, on-chain-client/docs/risk/2026-09-10-on-chain-client-hazards.md
**Parallel:** yes, with T11 (no file in common: T11's on-chain-client files are `2026-10-06-chain-write.md` and `architecture/README.md`); serial after T9
**IDs:** none new; prose only

*Note 2026-10-08:* T5 already rewrote org-node's mentions of the moved IDs
(step 2's grep finds none today); what remains in step 1 is the prose about
the chain read, the reader and the preflight. Re-take each line number
with `grep -n` when the task runs; those below are as of `1f52c36`.

1. For each org-node bullet of §6 not already done in T6/T5 — the README
   (`:63`), `AGENTS.md:14`, `docs/architecture/README.md:78`, `:85`, `:88`,
   `:105`, `:110`, `:144`, `:173`, `soup.md` rows, the decomposition's
   remaining mentions (`:16`, `:20`, `:63`, `:606`, `:1012`, `:1017`,
   `:2667`, `:2786`, `:2810`), `2026-10-04-type-safety.md:31`, the
   verify-and-commit and transport-binding-scope sentences, the
   dependency-expectations file's `:32`, `:80`, the chain-authority
   requirements' owner-context bullet `:93` and sequencing item `:238`,
   `architecture-derived.md:722`, the problems file `:21`, `:77`, `:105`,
   and `.guardrails/config.yaml:283` — rewrite to prose naming org-io's
   ledger, with a dated note where an item's meaning changes.
1a. *Added 2026-10-08:* resolve PR-zf924s (org-node/docs/problems/
   2026-10-07-close-out.md) under the owner's hybrid ruling (see
   Conventions). Classify each item it lists. REQ-ztdza4, whose meaning
   was reversed, gets a new superseding REQ with the R1 sender-rule
   wording, and the tests now verifying REQ-ztdza4 for that rule move to
   it. Do the same for any other listed item whose meaning changed. Items
   that were only narrowed or clarified keep their dated notes. Then set
   PR-zf924s to resolved, with the classification as its resolution, and
   add `Resolves: PR-zf924s` to the change's merge-message notes.
2. Carried from T5: on-chain-client's ledgers still call REQ-ysyu9g
   org-node's. In `on-chain-client/docs/requirements/2026-09-10-chain-reading.md:220`
   ("is org-node's expectation REQ-ysyu9g") and
   `on-chain-client/docs/risk/2026-09-10-on-chain-client-hazards.md:467`
   ("One requirement of org-node's") and `:1578` ("org-node holds one
   requirement addressed to this unit"), name org-io as its holder, each
   with a dated note ("moved to org-io with the chain read, change
   `worktree-org-io-create`, ruling A; ID, `opened:` date and due date
   unchanged"). The ID stays named: on-chain-client may cite it, being the
   unit it is addressed to. `:1722` and `:2210` name no holder: leave them.
3. Run `grep -rnw -e REQ-ysyu9g -e LLR-rm9x4z -e SDD-z85ux9 org-node` —
   expected: no output; `grep -rn "org-node's expectation\|org-node holds one requirement\|requirement of org-node's" on-chain-client/docs` —
   expected: no output. `check-trace.sh` and `check-ids.sh --allow-draft-files`
   for org-node and on-chain-client: exit 0. Commit: `docs(org-node): point the moved chain items to org-io`.

### T11 — on-chain-client and repository prose

**Files touched:** on-chain-client/docs/requirements/2026-10-06-chain-write.md, on-chain-client/docs/architecture/README.md, on-chain-client/docs/architecture/2026-10-06-chain-write.md, docs/CONTEXT.md, docs/plans/2026-09-05-ratchet-setup.md, docs/adr/2026-10-06-org-io-unit.md
**Parallel:** yes, with T10; serial after T9
**IDs:** none new

1. `on-chain-client/docs/requirements/2026-10-06-chain-write.md:8` and
   `docs/architecture/README.md:78`: "the app calls them" / "org-node keeps
   reading the chain through this" → org-io, dated. SDD-yg7n55 (`:50`):
   check the consumer wording; amend if it names the app.
2. `docs/CONTEXT.md`: add *org-io* ("the unit that does all IO for an
   Organisation…", one sentence, the ADR's) and make *Admin* "a user whose
   own account is a signatory of the multisig that controls the
   Organisation's pure proxy". `docs/plans/2026-09-05-ratchet-setup.md`:
   record that org-io is born with statement and branch coverage.
   The ADR: status stays Proposed until merge-change accepts it. The
   "Still open" items were ruled on 2026-10-07 and are already recorded in
   it; check the wording still matches the code.
3. `check-trace.sh` for on-chain-client: exit 0. Commit:
   `docs: org-io is on-chain-client's consumer; glossary and setup record`.

### T12 — org-io's coverage, CI and the closing checks

**Files touched:** Makefile, org-io/.guardrails/config.yaml, .github/workflows/rust.yml, org-io/docs/architecture/soup.md, docs/plans/2026-10-06-org-io-create.md
**Parallel:** no (serial, after T4, T10 and T11; last)
**IDs:** none new

1. `Makefile`, following the branch-coverage pattern from T0:

```make
ORG_IO_COVERAGE_ARGS := -p org-io --features test-support,dev-seed \
	--lib --test key_custody_config --test fuzz_seed_parse --test signatory_key \
	--test dev_seed_env --test absences --test chain_read_rule --test receive_order \
	--test submit_flow --test signatory_rule --test app_boundary

coverage-org-io:
	cargo llvm-cov $(ORG_IO_COVERAGE_ARGS) \
		--summary-only \
		--fail-under-lines $(ORG_IO_LINES) \
		--fail-under-regions $(ORG_IO_REGIONS)

coverage-branch-org-io:
	mkdir -p $(BRANCH_REPORT_DIR)
	CARGO_TARGET_DIR=$(BRANCH_TARGET_DIR) cargo +$(BRANCH_TOOLCHAIN) llvm-cov \
		$(ORG_IO_COVERAGE_ARGS) \
		--branch --json --summary-only \
		--output-path $(BRANCH_REPORT_DIR)/org-io.json
	$(call judge_branches,org-io,$(ORG_IO_BRANCHES))
```

   Add both to `.PHONY`, `coverage-org-io` to `coverage`, and
   `coverage-branch-org-io` to `coverage-branch`. (`dev_seed_env` is in the
   measured set, so its environment test runs; it is its own binary, so it
   still races nothing.) The preflight binary and `connect.rs` are shell
   (SDD-z85ux9): expect them uncovered.
2. Measure: run `make coverage-org-io` with the floors set to 0 first, then
   `make coverage-branch-org-io` with `ORG_IO_BRANCHES := 0`. Record the
   figures in a dated comment block in the style of person's ("measured
   2026-10-0x: lines …, regions …, branches … of …", toolchain line), and
   set `ORG_IO_LINES`, `ORG_IO_REGIONS`, `ORG_IO_BRANCHES` one point below
   each, rounded down. Re-run both with the floors: exit 0. If the stable
   line figure is below class C expectations because of the shell, write
   the shortfall beside the floor as on-chain-client's note does; the owner
   accepts or not at merge.
3. `org-io/.guardrails/config.yaml`: final `verify_commands` (the four lines
   of the plan header) and

```yaml
coverage_command:
  - make coverage-org-io
  - make coverage-branch-org-io
```

   replacing the commented-out block and its comment.
4. `.github/workflows/rust.yml`: add org-io's two `cargo test` lines and its
   clippy line to the `test`/`clippy` jobs, in the form the other crates
   use; the coverage jobs already call `make coverage` and
   `make coverage-branch`, which now include org-io.
5. Closing checks (paste the output into the plan's Progress section):
   `.guardrails/scripts/check-units.sh`; for each of the six units
   `check-trace.sh` and `check-ids.sh --allow-draft-files`; every unit's
   `verify_commands`; `make coverage coverage-branch`. Expected: exit 0
   everywhere; only UNRESOLVED-PR and UNMET-EXPECTATION advisories. Then
   hand off to `verify-before-merge` and `merge-change`.
6. Commit: `build(org-io): statement and branch coverage floors from the first merge`.

## Self-review (plan-change step 9)

1. **Every ID in Implements has a task whose test verifies it.** REQ-tg9zrn
   through LLR-7pj5af (T7), LLR-9r2bxd (T7), LLR-rm9x4z (T5); REQ-f3eu9n
   through LLR-qhyc3n, LLR-vyd5d3, LLR-c9fyun (T8); REQ-3pxa8f through
   LLR-c4bktx (T2), LLR-rgdx22 (T2, T7), LLR-3zdw8v (T2), LLR-u2pk5y (T7);
   REQ-v4tfap through LLR-gc6kwy (T1), LLR-c4bktx, LLR-3zdw8v, LLR-u2pk5y;
   REQ-8zuka3 through LLR-rgdx22, LLR-3zdw8v; REQ-qnh9pb through LLR-4ax2m6,
   LLR-9fy622, LLR-gc6kwy (T1); REQ-6fk4qj through LLR-n3zmt6 (T9),
   LLR-u2pk5y; RC-2xufsr through the REQs that implement it; REQ-nfr3n2
   through LLR-qhjp6g, LLR-be3zv9 (T7 tests, T9 definitions); REQ-ysyu9g is
   an `expects:` item (UNMET-EXPECTATION, no test owed); SDD-z85ux9 carries
   no LLR by its moved deviation; REQ-8p2veg, REQ-v8jczx through
   LLR-m3tjvp, LLR-a9bb7b (T3); REQ-m8sgjk through
   LLR-q225ws and LLR-gha5f6 (existing app tests, T9 re-parents);
   LLR-mn5c2q (T6). PR-b795an (T4), PR-k2xxaq (T6).
2. **Real code, commands and expected output** are given for every new test
   and every new module whose shape a reader could not take from an
   existing file; moves name the source lines to copy and the exact edits.
   T7 step 5's three bodies are specified by scenario and assertion rather
   than written out, because they are the admission scenario of
   `submit_flow.rs` (shown) with B's receive spawned; their assertions are
   given.
3. **Names and signatures are consistent:** `OrgService::new(store)`,
   `commit_genesis(…, chain_state)`, `commit_update(…, chain_state)`,
   `reconcile(…, chain_state)`, `receive_message` (returns the sender and
   the message), `prepare_receive`/`apply_receive`,
   `prepare_self_delete`/`apply_self_delete` (the `prepare_*` taking the
   sender), `Prepared::{Done, NeedsChain}`, `PendingReceive<O>::org_id`
   (T6, used in T7; re-checked 2026-10-08 against S3a); `StateReader::read_state`, `RawStateSource`,
   `read_org_state`, `org_state_from_chain` (T5, used in T7); `ChainWriter`,
   `FakeChain`, `OrgIo::{connect, not_configured, for_test, node, node_mut,
   found_organisation, submit_commit_send, receive_and_verify,
   receive_and_self_delete_if_revoked, is_own_admin}` (T7/T8, used in T9,
   except `is_own_admin`, which only T8's tests call in S2; its first caller
   is S3b-io, owner ruling of 2026-10-07);
   `SignatoryKey::{from_seed, account_id}`, `parse_seed`,
   `parse_co_signer`, `signatory_from_environment` (T1/T2, used in T7).
4. **Files touched and Parallel** are stated for every task. Tasks marked
   parallel share no file: T1 ∥ T3 (T1 is root workspace and org-io; T3 is
   on-chain-client only); T2 ∥ T3 (likewise); T4 runs after T3 and may run
   beside T1/T2 (Makefile and on-chain-client files only); T10 ∥ T11
   (org-node files vs on-chain-client and `docs/` files). Every other pair
   is serial: the root `Cargo.lock` and org-io's config are touched by every
   root-workspace task.

## Progress

| Task | State | Commit | Notes |
|---|---|---|---|
| T0 | done | `c164adf` | master `a57d760` merged |
| T1 | done | `d724133` | red -> green: a_seed_of_64_hex_characters_parses_with_or_without_one_prefix_and_in_either_case; a_malformed_seed_is_refused_by_rule_and_the_error_carries_none_of_it; a_co_signer_of_64_hex_characters_parses_to_its_account; a_malformed_co_signer_is_refused_by_rule_and_the_error_carries_none_of_it; each_refusal_names_its_variable_and_rule; fuzz_seed_parse — each watched fail (E0432, the custody functions did not exist) before custody.rs, green after. Left for T2: `test-support = []` gains "org-node/test-support"; remove `#[allow(dead_code)]` on `SeedBytes::bytes()` |
| T2 | done | `5142495` | red -> green: the_key_answers_its_account_and_renders_only_its_start; neither_the_seed_nor_the_key_renders_any_part_of_the_secret; the_development_build_reads_the_seed_once_and_none_when_unset — compile-red (E0432) before the code existed, green after; the_seed_is_read_once_and_only_behind_dev_seed — runtime-red before the implementation (no read), and red again with the dev-seed cfg removed; the_key_holders_derive_no_debug_and_have_no_display and org_io_names_no_device_or_member_private_key — source scans, watched red against a planted Debug derive and a planted DeviceSeed name, green restored. Carried: `test-support` gains "org-node/test-support" at T5 (org-node becomes a dependency there); remove `#[allow(dead_code)]` on `SignatoryKey::keypair()` at T7 |
| T3 | done | merged from worktree-org-io-create-s2t3 | red -> green: an_original_account_is_its_32_bytes; an_original_account_of_another_length_is_malformed; proxy_definitions_give_their_delegates_in_order; truncated_padded_or_overlong_definitions_are_malformed; fuzz_signatory_set — each watched fail (E0432, no `signatory_set` module) before the implementation, green after. on-chain-client `--all-targets` clippy was already red in older test files before T3 (`--lib` clean); quoted item texts indented (see Conventions) |
| T4 | done | `e8dffae`, fix `6fd2250` | PR-b795an: the writer is measured — lines 47.88% (floors 46/47), branches 58 of 58 (floor 99 kept). No test for the recipe itself (its before/after is the coverage summary). The fix added red -> green: is_proxied_is_false_for_a_call_that_is_not_a_variant — red with proxy.rs's non-variant arm returning true, green restored; observed_event_decodes_proxy_executed_and_ignores_the_fields_of_other_events (restructured to one closure type so llvm-cov sees one instantiation) — red with the fields read in the other-event branch, green restored |
| T5 | done | `cd674bb` | red -> green: chain_state_is_read_into_typed_values; a_chain_state_whose_key_org_node_refuses_is_an_error — compile-red (E0432, no `org_io::chain_read`) before chain_read.rs, green after; present_absent_failed_and_refused_reads_are_four_different_answers — compile-red, and runtime-red with a failure mapped to absence, green restored. Only the chain checks of the preflight moved (LLR-3zdw8v); T6 step 7 amended: org-node keeps a transport-only preflight until S4. Moved IDs no longer named in org-node docs (wording instead). Carried to T10: on-chain-client's ledgers still call REQ-ysyu9g "org-node's" (requirements/2026-09-10-chain-reading.md:220 and its hazards file) |
| T6 | done | `2a1a15a`, `7fa61a2` | red -> green: absences::org_node_names_no_chain_library_and_reads_no_chain (LLR-mn5c2q, LLR-65py3d) — red: preflight bin named `on_chain_client::`; service_lifecycle::two_services_over_one_set_of_chain_slots_see_the_same_state (LLR-hg3xzf) — red under a mutation where ChainSlots clones did not share; reconcile::the_service_reconcile_refuses_an_unheld_organisation_whatever_state_it_is_given (LLR-gr8x3r) — red under a mutation reading state before the held check. The rewritten value tests kept their IDs and were only compile-red; the deleted tests are as the plan lists. Gate: 301 passed, 0 failed; Quint 5/5; transport preflight 1/1; check-trace org-node exit 0. Ledgers: LLR-65py3d superseded by LLR-mn5c2q (dual-ID); 25 other amendments in place. PR-k2xxaq resolved. Deviation: the tests drive a test-side `support::Node` that runs org-io's sequence; the production API matches the plan. Carried to T10: org-node/README.md :22/:36/:78 still names MockChain, ChainReader and ChainOps |
| T7 | done | `a224d26` | red -> green: the 5 moved submit tests and receive_order's 9 — compile-red (E0432, no OrgIo/test_support), then runtime-red under mutations (a read before the chain-free phase: all 9 receive_order tests; a failed read taken as absence: a_failed_read/a_refused_parse); each_submission_reads_the_chain_once (new, LLR-9r2bxd normal) — red under a double read; the two read-back-failure tests — red with the failure mapped to absence; the_handle_hands_out_no_key — red (no src/submit.rs), and with a planted `pub` field; a_build_without_dev_seed… — compile-red, then hung under a connect-first mutant. LLR-7pj5af amended in place (unmerged draft). Deviations: the moved founding test kept its annotation (LLR-9r2bxd's normal case is a new test instead); `OrgIo` and `FakeChain` carry no signatory-set fields yet (T8 adds them with `SignatorySetReader`); no `org_io_over` helper (`OrgIo::for_test` covers it); the plan's no-public-field scan sliced from `pub struct`, fixed to the braces. org-io trace exit 1 until T8/T9: LLR-qhjp6g/LLR-be3zv9 UNDECLARED-DEPENDENCY (definitions move in T9), T8's and T9's LLRs MISSING-TEST. Carried to T8: LLR-9r2bxd's "no state → OrgNotOnChain" clause has no test (FakeChain needs a hide-state switch) |
| T8 | done | `71b682c` | All 8 new tests compile-red first (E0432/E0599). Runtime red under mutations: a_co_signer_equal_to_the_own_account_is_not_counted_twice (duplicate check removed); not_admin_for_an_unmapped_h160… (mapped account ignored); a_node_holding_no_signatory_key_is_refused_with_not_configured (read before key check); a_failed_signatory_read_is_an_error_never_not_admin (failure → NotAdmin); a_configured_account_listed_as_delegate_is_admin… (delegates always read); submit_flow::a_read_back_that_finds_no_state_is_refused_with_org_not_on_chain (LLR-9r2bxd, carried from T7; founding returned Ok). the_controller_is_the_own_account_alone… and admin_when_the_mapped_proxy_lists_the_controller were compile-red only. Gate: org-io 40+8 passed, clippy clean under 3 feature sets, check-trace org-node exit 0, org-io left with only T9 items. Ledger edits clarify wording in place. Deviations: two FakeChain failure switches; normal/abnormal tests split; signatory::own_admin_status crate-private. Carried to T8b: LLR-9r2bxd "returned the same way" read as also saying the write executed and nothing was committed (dispatcher decision; the draft is this change's) |
| T8b | done | `5474c7f` | red -> green: submit_flow::a_read_back_that_finds_no_state_says_the_write_executed_and_nothing_was_committed — red: the founding error carried only the refusal's text. `refused_after_write` in submit.rs. LLR-9r2bxd clarified in place (dated; meaning unchanged). org-io 41+8 passed, clippy clean. Note for deslop/review: org-io's submission errors are `String`, and the test matches text; a typed error is a candidate PR |
| T9 | done | `8c3a30c` | red -> green: app_boundary::the_app_depends_on_org_io_and_on_no_other_unit_or_chain_library — red because `[dependencies]` lacked org-io, then red with "names org-node", green once the old dependencies were removed. App tests rewritten onto OrgIo::for_test with IDs unchanged (compile-red only). Gate: app cargo 157 passed, vitest 44, npm check clean, org-io 41+8, org-node 303, clippy clean. check-trace and check-ids exit 0 for all six units; check-units exit 0. Ledger: REQ-nfr3n2, LLR-qhjp6g and LLR-be3zv9 moved (ruling A, clarifications); LLR-q225ws and LLR-gha5f6 re-parented to the new REQ-m8sgjk; other dated notes in place; REQ-x3c8n2 `expects:` moved from org-node to org-io (clarification: the IO owner changed, the expectation did not). The lock is now format v4 (quinn-udp fetched via CARGO_HOME=/tmp/cargo_home_fuzz); async-trait dropped. Carried to T9b: the app's `init` reopens the store after a failed `OrgIo::connect`; connect should hand the service back on error. Carried to T12: ADR :15 still labels the moved IDs "(app)" |
| T9b | done | `907f3d2` | red -> green: signatory_key::a_refused_connect_hands_back_the_service_it_was_given_and_says_why — compile-red (E0432, no ConnectFailed); state_assembly::init_opens_the_store_once_even_when_the_connect_is_refused — runtime-red, "init opens the store 2 times". connect returns Err(Box<ConnectFailed{service, reason: ConnectFailure}>), boxed for clippy result_large_err. LLR-rgdx22 and LLR-85zque clarified in place. Gate: org-io 43+9, app 158 + vitest 44, clippy clean, trace and ids exit 0 |
| T10 | done | `6a572b9` | Ran early, in parallel with T7; touches no org-io file. Prose sweep done, including the T6 README carry and the README's stale "no sender check" note. PR-zf924s resolved under the hybrid rule: REQ-ztdza4 superseded by REQ-uk9rw7 (13 LLR/SDD pointers moved, each with a dated note); RC-b6mydy kept in place (its sender rule already has its own ID, RC-u7kdam); 7 narrowings and 3 clarifications in place; LLR-js9dsu kept in place, borderline, flagged for the owner. red -> green: receive_chain_reads::an_update_from_an_unlisted_device_is_refused_before_any_chain_state_is_asked_for (REQ-uk9rw7, REQ-ztdza4, LLR-2r2fha) — red under a mutation where `sender_listed` accepted all. org-node 302 passed, 0 failed; check-trace org-node and on-chain-client exit 0. The hazards note says the app's edge to on-chain-client goes "in the same change"; T9 must make that true |
| T10b | done | `5210b92` | Owner ruling 2026-10-08: LLR-js9dsu changed meaning, so it is superseded by LLR-tajh9d (exactly three wire kinds; leftover bytes and unknown kinds refused). red -> green: wire_frame_bound::exactly_three_kinds_round_trip_and_an_unknown_kind_or_leftover_bytes_are_refused — red with a decoder that ignored leftovers, and red with index 3 decoded as a revocation. No satisfies/traces pointers needed moving. org-node 303 passed, 0 failed; check-trace exit 0 |
| T11 | done | `2163ecb` | Run early, in parallel with T6 (no shared file). Prose only, no tests. All amendments are in-place notes; none changes a meaning. Additional note on on-chain-client architecture/README.md segregation paragraph. The ADR was checked against the T6–T9 plan text, and its preflight sentence was corrected with a dated revision note. **T12/deslop must re-check the ADR against the merged code.** check-trace on-chain-client exit 0 |
| T12 | done | (this commit) | Coverage (first measurement, 2026-10-08): lines 62.30% (342/549), regions 59.72% (507/849), branches 17 of 26, 65.38%; floors 61 / 58 / 64. Plan target (one point below the measurement) met. The class C shortfall is written beside the floors in the Makefile, for the owner at merge: the chain shell (SDD-z85ux9: preflight, connect, the OnChain* readers and writer) is uncovered, and so are the 9 branch outcomes (preflight 8, plus custody.rs:102's unreachable `let … else`). Outside the shell, uncovered code with no test: `OwnAdminError` Display, `WriterNotConfigured`, `refused_after_write`'s other arm, and lib.rs:207 (the self-delete `NeedsChain` read). No test was written, because none of these is plainly owed by an LLR. The ADR was re-checked against the code, with two dated notes (the moved IDs' units at :15; the whole-crate `pub use org_node as node`). SOUP: rows added for rand_core, iroh and the dev-only rand and tokio test-util. CI runs org-io's two test lines and its clippy line. Closing checks: every unit's verify_commands exit 0 (org-io 43+9, org-node 303 + Quint 8 lines, app 158 + vitest 44 + check clean, on-chain-client 73+39, org-members 271 + Quint 6 lines, person 123 + clippy); `make coverage coverage-branch` exit 0; check-trace and check-ids --allow-draft-files exit 0 for all six units (advisories: UNMET-EXPECTATION in org-io, org-node and app; UNRESOLVED-PR in org-node, app and on-chain-client); check-units exit 0. `merge-preflight.sh --before-review --local-base worktree-org-io-create-t12`: CLEAN-TREE ok and BASE-MERGED ok, then it stops at UNITS: on-chain-client has three DRAFT-FILE ledgers (signatory-reads), which merge-change step 3 renames |

T12b (`c8c401a`, missing tests after T12's first measurement). Each was red under a temporary mutation:
- red -> green: receive_order::a_revocation_whose_read_fails_deletes_nothing (LLR-7pj5af). Red with a failed self-delete read taken as absence.
- red -> green: receive_order::a_revocation_whose_read_is_refused_at_parse_deletes_nothing (LLR-7pj5af). Red under the same mutation.
- red -> green: receive_order::an_acknowledgement_on_the_self_delete_path_is_decided_with_no_read (LLR-7pj5af). Red with a read in the Done arm.
- red -> green: submit_flow::a_commit_refused_for_another_reason_after_a_write_returns_that_refusal_unchanged (LLR-9r2bxd). Red with the other arm wrapped.
- red -> green: submit_flow::a_handle_with_no_chain_refuses_every_submission_at_the_write (LLR-qhjp6g). Red with WriterNotConfigured returning Ok.

Coverage: lines 63.39%, regions 60.90%, branches 17/26. Floors are now 62/59/64. org-io 48+9 passed.

Carried to T12c: every refusal after a successful write must say that the write executed, not only OrgNotOnChain.

T12c (dispatcher decision, 2026-10-08): LLR-9r2bxd clarified in place (dated note quoting T12b's "returned as org-node's refusal, its text unchanged"); `refused_after_write` now wraps every refusal. Deviation: FakeChain::epoch_ahead was added to test_support.rs with a python script, not the editor tool (AGENTS.md rule); the content was checked at deslop.
- red -> green: submit_flow::a_commit_refused_for_another_reason_after_a_write_says_the_write_executed_and_nothing_was_committed (LLR-9r2bxd) — red: the founding error was the bare refusal "parent_seq 1 is not the on-chain epoch 2".
- Deleted: submit_flow::a_commit_refused_for_another_reason_after_a_write_returns_that_refusal_unchanged (T12b), because it asserted the refusal returned unchanged, which LLR-9r2bxd as clarified forbids.
- New `FakeChain::epoch_ahead` switch: the read-back answers the slot one epoch past the write, so the commit refuses (SeqNotEpoch / verify) with the update still held.

Coverage at T12c: lines 63.52%, regions 61.03%, branches 17/26; floors 62/59/64 unchanged. org-io 48+9 passed.

*Corrected 2026-10-08 (review round 1, finding-15).* The figures above were
measured before the deslop pass (`6ddbc76`), and do not reproduce on the
reviewed tree. The deslop pass removed code, which changed the totals (549
lines and 849 regions before). Measured after it, as the reviewer
reproduced: lines 62.78% (334 of 532), regions 60.41% (502 of 831),
branches 17 of 26. The floors stay at 62 / 59 / 64 and are still met.
*Superseded 2026-10-08 by the review round 2 note below.*

*Note 2026-10-08 (review round 2, finding-11).* The figures recorded above
are stale. Measured after the review round 1 and round 2 fixes, at commit
`929c828`: lines 65.57% (400 of 610), regions 63.44% (597 of 941), branches
18 of 28 (64.29%). The floors are raised to 64 / 62 / 64; the branch floor
is kept at 64, because its margin is one outcome.

*Note 2026-10-08 (review fixes; review round 3, finding-5).* The red-first
evidence for the tests added or rewritten by the review round 1 and round 2
code fixes, as the fix reports observed it.

Round 1 (`92c610e`, `3c62a84`, `20133c7`, `ac616f8`, `75f4f46`, `8777f8f`,
`2c54cad`; merged `ad3d79b`):
- red -> green: absences::org_node_hands_out_no_stored_device_private_key — red on the original tree: a public `device_seed` field, two production `Serialize` derives (`PersonaRecord`, `StoreData`) and `OrgEndpoint::inner()`.
- red -> green: app_boundary::no_ipc_facing_type_of_the_app_carries_key_material — red with a planted `key: DeviceSeed` field in `PersonaDto`.
- red -> green: absences::the_seed_path_keeps_no_unwiped_copy_of_the_seed — red because `size_of::<SeedBytes>()` was 32.
- red -> green: signatory_key::a_build_without_dev_seed_never_asks_for_the_seed_on_a_write_or_a_read — red because the refused write said "set … ODS_ADMIN_SEED".
- red -> green: absences::the_handle_offers_no_chain_judging_operation_outside_its_own_sequence — red on `node_mut() -> &mut OrgService`.
- red -> green: dev_seed_env::connect_refuses_a_co_signer_equal_to_the_own_account — red because connect ran past the 5 s bound.
- red -> green: dev_seed_env::a_seed_set_but_not_utf8_is_refused_as_malformed_not_reported_unset — red because it returned `Ok(None)`.
- red -> green: absences::the_handle_hands_out_no_key (rewritten to read whole multi-line signatures) — red with a planted multi-line method returning `Option<&custody::SignatoryKey>`.
- red -> green: submit_flow::a_submission_that_never_finishes_times_out_and_nothing_is_committed — red on "chain write failed" (the timed-out write did not say its outcome is unknown).
- red -> green: absences::seed_bytes_offers_no_comparison_outside_test_builds — red while `same_bytes_as` was available outside test builds.
- Follow-up: the `wire_frame_bound` clippy fix (`clippy::panic`) in `e6745f2`, with no change of behaviour.

Round 2 (`5351490`, `fc2fa16`), each watched red against the tree at `50d4c5a`:
- red -> green: absences::org_io_hands_out_no_bare_service — red on `connect.rs:55 pub service: OrgService`.
- red -> green: app_boundary::the_app_calls_no_chain_judging_operation_and_builds_no_service — red on `state.rs:212 OrgService::new(`.
- red -> green: signatory_key::a_build_without_dev_seed_says_why_the_own_admin_check_is_not_configured — red because the message said only "this node holds no signatory key".
- red -> green: absences::org_node_hands_out_no_stored_x25519_secret — red listing three public fields and two production `Serialize` derives (`OrgRecord`, `ProvisionalUpdate`).

Record follow-ups:
- r1follow (`e6745f2`, merged `0939935`): finding-5's clause on LLR-7pj5af and LLR-9r2bxd, finding-3's residual on LLR-c4bktx, a placeholder dropped, and the `wire_frame_bound` clippy fix above.
- r2record (`ded559f`, merged `a301aa8`): round 2's record findings 4, 5, 8, 9, 10 and 12, the LLR-c4bktx owner ruling, and PR-8geawr and PR-z3gg2b booked.
- r2tidy (`ece40c6`, merged `715d06a`): LLR-rgdx22's `ConnectFailed` note brought to `5351490`, and the coverage record at `929c828` (finding-11).
- r3record (this commit): round 3's record findings 5, 6 and 7 and PR-wk7zwq booked (finding-4).

*Note 2026-10-08 (review round 3 code fix; review round 4, finding-4).*
Round 3 (`3f0bfed`, record `c08515e`; merged `31f758f`): the allowlisted read
surface (`NodeView`), `BuiltUpdate` submission with the held-update check,
`OrgNotHeld`, and `OrgIo::open`'s tests. Each red was watched as follows:
- red -> green: absences::the_handle_returns_only_allowlisted_public_types — red at `36f9470`, listing `node() -> &OrgService`, `NodeBuilders`' `Deref`, and `admit_member`/`revoke_member` returning `ProvisionalUpdate`.
- red -> green: submit_flow::the_view_summarises_the_public_fields_org_node_holds — red against a planted `OrgSummary` that dropped its Members.
- red -> green: submit_flow::an_update_org_node_does_not_hold_on_the_records_root_is_refused_before_any_write — red at `36f9470`: the altered and the stale update were each written. (Review round 4, finding 2: the stale case is refused by the held half; the root comparison has no red of its own.)
- red -> green: submit_flow::an_update_for_an_organisation_the_store_does_not_hold_is_refused_as_not_held — red reporting "no on-chain state found for org".
- red -> green: open::open_creates_the_data_dir_and_opens_the_store_inside_it, open::open_refuses_a_data_dir_it_cannot_create_naming_it — red against a planted `create_dir` in place of `create_dir_all`, with the prefix "mkdir".
- red -> green: open::open_refuses_a_store_under_another_passphrase_and_opens_no_other — red against the planted prefix "store: ".

Round 3 coverage (`c08515e`): lines 71.26% (476 of 668), regions 69.16%
(711 of 1028), branches 25 of 38; floors 70 / 68 / 64.

*Note 2026-10-08 (review round 4 fix, r4fix).* Findings 1 and 2 fixed,
3 and 4 recorded (LLR-3zdw8v's sixth-scan N/A note; the round-3 entry
above). org-node gains `OrgService::holds_provisional` (LLR-gwk4nk, in
`org-node/docs/architecture/2026-10-08-values.md`) and a test-only
`OrgService::transport_mode()`; org-io's pre-write guard reads by reference
and clones no record or held update; `BuiltUpdate::names` is gone. LLR-qhjp6g
has a dated note: the root-comparison half of its third refusal is defensive
and unreachable today, and its "watched red" for the stale update is the held
half's. LLR-rgdx22 has a dated note for the transport-mode test.
- red -> green: open::open_sets_the_transport_mode_it_was_given — red with `set_transport_mode` deleted from `OrgIo::open` (left `Loopback`, right `Networked`).
- red -> green: commit_paths::holds_provisional_matches_all_four_public_fields_and_writes_nothing (org-node) — red with the method missing (E0599), and again with the base-root comparison removed ("another base root").
- red -> green: absences::the_pre_write_guard_copies_no_record_and_no_held_update — red with `.cloned()` replanted after the record's `find` (submit.rs:217).

Round 4 coverage: lines 70.91% (468 of 660), regions 69.07% (708 of 1025),
branches 21 of 32 (65.63%). The fall is `BuiltUpdate::names` leaving
org-io (covered code, six branch outcomes). One point below would be
69 / 68 / 64; floors are never lowered, so they stay 70 / 68 / 64, all met.

Execution order: T0 → (T1 → T2) ∥ (T3 → T4) → T5 → T6 → T7 → T8 → T9 →
(T10 ∥ T11) → T12.

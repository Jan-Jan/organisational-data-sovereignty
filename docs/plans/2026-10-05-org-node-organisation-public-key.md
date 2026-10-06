# org-node: Organisation public key, X25519 member keys, sender-authenticated Envelopes — Implementation Plan

**Goal:** Bring org-node (and the app over it) back to green on this branch by giving every persona an X25519 Member-as-a-group key, dropping the Envelope signature in favour of the connection-authenticated sender-device rule on every Receive operation, publishing a freshly generated X25519 Organisation public key, and refusing an Organisation state whose key is not a valid X25519 key.
**Implements:** REQ-7h7qp3, REQ-txvtm9, REQ-8jb4ny, REQ-ech45n (org-node SRS draft `org-node/docs/requirements/2026-10-05-envelope-authenticity.md`); RC-2e6k44 (org-node RMF draft `org-node/docs/risk/2026-10-05-envelope-authenticity.md`), through REQ-7h7qp3 and REQ-txvtm9. Superseded and kept on their tests beside the replacements: REQ-ag6kqm, REQ-nhe2zu (`org-node/docs/requirements/2026-09-09-verify-and-commit.md`), RC-pm9kmx (`org-node/docs/risk/2026-09-09-org-node-hazards.md`).
**Safety class:** C (org-node, app; no per-item overrides)
**Verification:** every `verify_commands` entry of `org-node/.guardrails/config.yaml` (the cargo line, with `--test organisation_key` added by T6, and the seven quint commands) and of `app/.guardrails/config.yaml` (the cargo line, `npm --prefix app run check`, `npm --prefix app run test`); `GR_CONFIG=org-node/.guardrails/config.yaml .guardrails/scripts/check-trace.sh` and `GR_CONFIG=app/.guardrails/config.yaml .guardrails/scripts/check-trace.sh`, each exit 0. Additionally the three chopsticks targets must compile: `cargo test -p org-node --features app,test-support --test chain_genesis_e2e --test finality_polling --test preflight --no-run`.

This is step 4 ("fix org-node") of `docs/plans/2026-10-04-person-shared-types.md`.
Its owner rulings, restated so the tasks below can be checked against them:

- `orgPubKey` is the Organisation public key, an X25519 key-agreement key
  (root `docs/CONTEXT.md`). It authenticates nothing org-node receives.
- The Envelope signature is dropped entirely. The Envelope keeps the
  Organisation identifier, the Sequence number and the Change set bytes, and
  loses its signature field, its signing and its verification. The wire
  format changes; there are no adopters.
- Sender authenticity is the ed25519 Device key the iroh connection
  authenticated, checked before the Change set is decoded on every Receive
  operation (`receive_and_verify` and `receive_and_self_delete_if_revoked`):
  on a first admission, the administrator's Device key named by the imported
  Invite; otherwise a Device key in the node's own current record.
- Authority is the chain root and epoch match (RC-6a2dke, RC-e5atck,
  unchanged). Admin authority over on-chain updates is the multisig proxy and
  out of org-node's scope. The `role` field is out of scope.
- `OrgPublicKey` is org-node's own newtype (not in `person`), constructed only
  through `parse`, which applies `person::x25519::is_valid_public_key`.
- A persona's member key is an X25519 key derived from `member_seed`
  (`MontgomeryPoint::mul_base_clamped`, curve25519-dalek 4.1.3 as a direct
  dependency). The Organisation secret is freshly generated in
  `create_organisation`, kept in the creator's encrypted store and distinct
  from the genesis member and device keys.
- The node's own persona and the admin persona are identified without
  comparing against `org_pub_key`. `admin_member_key` stays a local record,
  taken from the Invite on first admission, not from the chain.

*Note 2026-10-05 (review round 3, finding-16).* Two later rulings change this
list; the tasks below predate both and are kept as written.

- The Sequence number equals the epoch of the Organisation state the
  receiving node read itself and verifies against (owner, 2026-10-05, review
  round 1 finding-1; REQ-txvtm9, RC-95dgg8). Senders set it to the epoch
  their own chain update produced.
- Nothing about the sender is checked, and a first admission no longer needs
  an imported Invite (owner, 2026-10-05, change
  `worktree-org-node-chain-authority`). The sender-authenticity rule above
  was withdrawn: `docs/plans/2026-10-05-switch-trim.md`.

**Harness rules for whoever executes this plan.** One plain git command per
Bash call (no compound lines that mention git, no heredocs). Write files with
the Write/Edit tools. `~/.cargo` is read-only: every cargo command below runs
with `CARGO_HOME=/tmp/cargo_home_fuzz`. Logs go to
`/private/tmp/claude-501/-Users-jan-jan-Coding-2-tier-access-control/040b9a0d-b135-4b8c-ab0a-f48aa8a3106c/scratchpad/`
under names starting `plan4-` (written `$S/plan4-…` below; set
`S=/private/tmp/claude-501/-Users-jan-jan-Coding-2-tier-access-control/040b9a0d-b135-4b8c-ab0a-f48aa8a3106c/scratchpad`
at the start of each Bash call that uses it). The quint simulator commands
need `QUINT_HOME` set to a writable scratch directory seeded from
`~/.quint/rust-evaluator-v0.7.0` (never write `~/.quint`). Commits use
`git -c commit.gpgsign=false commit` with no Co-Authored-By line. The two
long cargo lines are written once here and referred to by name:

- **ORG-NODE-GATE**: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --lib --test service_stories --test transport_handshake --test transport_networked --test fuzz_envelope_decode --test fuzz_verify_against_chain --test verify_against_chain --test wire_frame_bound --test store_at_rest --test admission_sender --test fuzz_first_admission_base` (from T6 on, with ` --test organisation_key` appended)
- **APP-GATE**: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --manifest-path app/src-tauri/Cargo.toml --features test-support --test startup_policy --test connection_status --test org_id_parsing --test receiver_events --test receiver_guard --test ipc`

Commands run from the worktree root,
`/Users/jan-jan/Coding/2-tier-access-control/.claude/worktrees/worktree-person-shared-types`.

## Red baseline (measured 2026-10-05, before T1)

Measured while writing this plan, with `--no-fail-fast` added to both gates:

- **org-node** (ORG-NODE-GATE): does not compile, `exit 101`, 0 tests run.
  `error: could not compile org-node (lib) due to 11 previous errors`, all
  `E0599`, all from `person`'s constructor rename (b3bf9a4): eight
  `no associated function … named new found for struct DevicePublicKey`
  (`org-node/src/transport/endpoint.rs:303`, `org-node/src/keys.rs:38`,
  `org-node/src/service.rs:490, 821, 1015, 1048, 1058, 1327`) and three
  `no associated function … named from_bytes found for struct PersonPublicKey`
  (`org-node/src/keys.rs:33`, `org-node/src/service.rs:496, 818`).
- **app** (APP-GATE): does not compile, `exit 101`, 0 tests run: the same 11
  errors, reached through the org-node path dependency. The app's own code
  has none yet; it gains two when T4 renames `BadSignature` and T7 retypes
  `ChainOps`.
- **check-trace, org-node**: exit 1, from four findings only:
  `MISSING-TEST REQ-7h7qp3`, `MISSING-TEST REQ-8jb4ny`,
  `MISSING-TEST REQ-ech45n`, `MISSING-TEST REQ-txvtm9`. The rest of its
  output (2 UNMET-EXPECTATION, 5 UNRESOLVED-PR, all within limits) is not
  a failure.
- **check-trace, app**: exit 0.

Behind the compile errors sits a second layer of red, which T2 exposes and
records: `PersonPublicKey` is now validated as X25519, and org-node still
hands it ed25519 encodings. An ed25519 public key whose top bit is set is
never a canonical X25519 key. Of the fixture seeds, `[2u8; 32]` (bob's member
key in `test_fixtures::admit_member_delta` and the inline fixtures) is one.
The `OsRng` personas of the service tests fail at random.

## Order and parallelism

Every task is serial. `org-node/src/service.rs` is touched by T2, T3, T5, T6
and T7. The test files are touched by several tasks each. The app (T8) needs
the org-node API as it stands after T7. No two tasks run at once.

| Task | Title | Parallel |
|---|---|---|
| T1 | Record the red baseline | no |
| T2 | Compile again: call `person`'s `parse`/`try_from` constructors | no (after T1) |
| T3 | X25519 Member-as-a-group keys from `member_seed` | no (after T2) |
| T4 | Drop the Envelope signature; the sender-device check in verification | no (after T3) |
| T5 | The sender-device rule on both Receive operations; the admin persona without `org_pub_key` | no (after T4) |
| T6 | A fresh Organisation key at creation (REQ-ech45n) | no (after T5) |
| T7 | `OrgPublicKey`, parsed where chain state is read (REQ-8jb4ny) | no (after T6) |
| T8 | App: the renamed verdict, the new receiver error, the retyped stubs | no (after T7) |
| T9 | Full verification, measured counts, traceability | no (after T8) |

---

### T1 — Record the red baseline

**Files touched:** none in the tree (logs only: `$S/plan4-t1-org-node.log`, `$S/plan4-t1-app.log`, `$S/plan4-t1-trace-org-node.log`, `$S/plan4-t1-trace-app.log`)
**Parallel:** no (first task)
**verifies:** none (no test written)

1. Run ORG-NODE-GATE with `--no-fail-fast`, output to `$S/plan4-t1-org-node.log`:

   ```bash
   S=/private/tmp/claude-501/-Users-jan-jan-Coding-2-tier-access-control/040b9a0d-b135-4b8c-ab0a-f48aa8a3106c/scratchpad
   CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --lib --test service_stories --test transport_handshake --test transport_networked --test fuzz_envelope_decode --test fuzz_verify_against_chain --test verify_against_chain --test wire_frame_bound --test store_at_rest --test admission_sender --test fuzz_first_admission_base --no-fail-fast > $S/plan4-t1-org-node.log 2>&1; echo "exit $?" >> $S/plan4-t1-org-node.log
   grep -E "^error" $S/plan4-t1-org-node.log | sort | uniq -c; tail -1 $S/plan4-t1-org-node.log
   ```

   Expected:

   ```
      1 error: could not compile `org-node` (lib) due to 11 previous errors
      1 error: could not compile `org-node` (lib test) due to 11 previous errors
      3 error[E0599]: no associated function or constant named `from_bytes` found for struct `PersonPublicKey` in the current scope
      8 error[E0599]: no associated function or constant named `new` found for struct `DevicePublicKey` in the current scope
   exit 101
   ```

2. Run APP-GATE with `--no-fail-fast` into `$S/plan4-t1-app.log`, the same
   way. Expected: the same 3 + 8 `E0599` lines,
   `error: could not compile org-node (lib) due to 11 previous errors`, `exit 101`.

3. Run both traceability checks:

   ```bash
   S=/private/tmp/claude-501/-Users-jan-jan-Coding-2-tier-access-control/040b9a0d-b135-4b8c-ab0a-f48aa8a3106c/scratchpad
   GR_CONFIG=org-node/.guardrails/config.yaml .guardrails/scripts/check-trace.sh > $S/plan4-t1-trace-org-node.log 2>&1; echo "exit $?" >> $S/plan4-t1-trace-org-node.log
   GR_CONFIG=app/.guardrails/config.yaml .guardrails/scripts/check-trace.sh > $S/plan4-t1-trace-app.log 2>&1; echo "exit $?" >> $S/plan4-t1-trace-app.log
   grep -E "MISSING-TEST|^exit" $S/plan4-t1-trace-org-node.log; tail -1 $S/plan4-t1-trace-app.log
   ```

   Expected: four `MISSING-TEST` lines (REQ-7h7qp3, REQ-8jb4ny, REQ-ech45n,
   REQ-txvtm9), `exit 1`; then `exit 0`.

If any of the three differs from "Red baseline" above, stop and report. The
plan was written against this state.

---

### T2 — Compile again: call `person`'s `parse`/`try_from` constructors

**Files touched:** `org-node/src/keys.rs`, `org-node/src/service.rs`, `org-node/src/transport/endpoint.rs`, `org-node/src/test_fixtures.rs`, `org-node/tests/transport_handshake.rs`, `org-node/tests/transport_networked.rs`, `org-node/tests/fuzz_verify_against_chain/fuzz_target.rs`, `org-node/tests/service_stories.rs`, `org-node/tests/admission_sender.rs`
**Parallel:** no (serial, after T1)
**verifies:** none new. This task only renames call sites. Its "test" is that the crate compiles, and that the test run then shows the X25519 layer of red, which it records.

`DevicePublicKey::new` became `DevicePublicKey::try_from(VerifyingKey)` and
`DevicePublicKey::parse(&[u8; 32])`. `PersonPublicKey::from_bytes` became
`PersonPublicKey::parse`. Both are fallible, so
`SigningKeypair::device_key` now returns a `Result`. `chain_genesis_e2e.rs`
is not compiled by ORG-NODE-GATE and is brought in line once, in T7.

1. `org-node/src/keys.rs`: replace `member_key` and `device_key`:

   ```rust
       /// As a member-as-a-group key for the trie.
       pub fn member_key(&self) -> Result<PersonPublicKey, OrgMembersError> {
           Ok(PersonPublicKey::parse(self.verifying_key().as_bytes())?)
       }

       /// As a device key for the trie / iroh identity, through `person`'s parse.
       pub fn device_key(&self) -> Result<DevicePublicKey, OrgMembersError> {
           Ok(DevicePublicKey::try_from(self.verifying_key())?)
       }
   ```

   In its test `member_and_device_keys_wrap_the_verifying_key`, change the
   second assertion to
   `assert_eq!(kp.device_key().expect("valid key").as_bytes(), kp.verifying_key().as_bytes());`.

2. `org-node/src/transport/endpoint.rs`:
   - both `device_key: device.device_key(),` lines (in `bind_with_mode` and
     `bind_with_relay`) become
     `device_key: device.device_key().map_err(|e| TransportError::Bind(format!("device key: {e}")))?,`
   - in `recv_one`, `let remote_key = DevicePublicKey::new(verifying);` becomes
     `let remote_key = DevicePublicKey::try_from(verifying).map_err(|_| TransportError::Malformed)?;`
   - in the test `endpoint_id_equals_device_key`, both `device.device_key()`
     become `device.device_key().unwrap()`.

3. `org-node/src/service.rs`:
   - `trie_from_snapshots` becomes:

     ```rust
     fn trie_from_snapshots(snapshots: &[MemberSnapshot]) -> Result<Trie, OrgNodeError> {
         let leaves: Result<Vec<MemberLeaf>, OrgNodeError> = snapshots
             .iter()
             .map(|s| {
                 let device_keys: Result<Vec<org_members::DevicePublicKey>, OrgNodeError> = s
                     .device_keys
                     .iter()
                     .map(|dk| Ok(org_members::DevicePublicKey::parse(dk).map_err(OrgMembersError::from)?))
                     .collect();
                 let leaf = MemberLeaf::new(
                     MemberId::new(s.id),
                     Handle::parse(&s.handle)?,
                     org_members::PersonPublicKey::parse(&s.member_key).map_err(OrgMembersError::from)?,
                     Name::parse(&s.name).map_err(OrgMembersError::from)?,
                     Surname::parse(&s.surname).map_err(OrgMembersError::from)?,
                     device_keys?,
                 )
                 .map_err(OrgNodeError::Trie)?;
                 Ok(leaf)
             })
             .collect();
         Trie::genesis(leaves?).map_err(OrgNodeError::Trie)
     }
     ```

   - `create_organisation`: `vec![device_kp.device_key()],` becomes `vec![device_kp.device_key()?],`.
   - `admit_member`: delete the two lines binding `member_vk` and `device_vk`
     (`VerifyingKey::from_bytes(&join_request.member_key)` and
     `VerifyingKey::from_bytes(&join_request.device_key)` with their
     `.map_err`), and build the leaf with:

     ```rust
             org_members::PersonPublicKey::parse(&join_request.member_key).map_err(OrgMembersError::from)?,
             Name::parse(&join_request.name).map_err(OrgMembersError::from)?,
             Surname::parse(&join_request.surname).map_err(OrgMembersError::from)?,
             vec![org_members::DevicePublicKey::parse(&join_request.device_key).map_err(OrgMembersError::from)?],
     ```

   - `receive_and_verify`, the post-verification sender check: `.any(|m| m.has_p2p_device(&org_members::DevicePublicKey::new(*remote_device_key.verifying_key())));`
     becomes `.any(|m| m.has_p2p_device(&remote_device_key));`. The value
     `recv_one` returns is already a `DevicePublicKey`.
   - `receive_and_verify`, `my_persona_id`: the `find` closure becomes

     ```rust
             .find(|p| {
                 let Ok(dk) = SigningKeypair::from_seed(p.device_seed).device_key() else {
                     return false;
                 };
                 verified.trie.members().iter().any(|m| {
                     m.has_p2p_device(&dk)
                         && m.p2p_key().as_bytes() != chain_state.org_pub_key.as_ref()
                 })
             })
     ```

   - `receive_and_verify`, `my_member_id`: the two lines building
     `my_device_kp`/`my_device_key` become
     `let my_device_key = SigningKeypair::from_seed(persona.device_seed).device_key()?;`.
   - `receive_and_self_delete_if_revoked`, `my_still_present`: the `.any`
     closure becomes

     ```rust
             .any(|p| {
                 SigningKeypair::from_seed(p.device_seed)
                     .device_key()
                     .is_ok_and(|my_dk| verified.trie.members().iter().any(|m| m.has_p2p_device(&my_dk)))
             });
     ```

4. `org-node/src/test_fixtures.rs`: in `member`, `vec![fix.device.device_key()]` becomes `vec![fix.device.device_key().unwrap()]`.

5. Tests: `.device_key()` gains `.unwrap()` at
   `org-node/tests/transport_handshake.rs` (the two `vec![…device_key()]` in
   `genesis_and_admit` and `a_device.device_key().as_bytes()` in the first
   assertion), `org-node/tests/transport_networked.rs` (the same three),
   `org-node/tests/fuzz_verify_against_chain/fuzz_target.rs`
   (`vec![admin_device.device_key()]`), `org-node/tests/service_stories.rs`
   (`vec![device.device_key()]` in `leaf_of`) and
   `org-node/tests/admission_sender.rs` (the `device_kp(…).device_key()` in
   `first_admission_from_a_device_other_than_the_invites_admin_is_rejected`).

6. Run ORG-NODE-GATE with `--no-fail-fast` into `$S/plan4-t2-org-node.log`
   and summarise:

   ```bash
   grep -E "^test result|FAILED|panicked|InvalidPersonKey|^exit" $S/plan4-t2-org-node.log
   ```

   Expected: it compiles, with no `error[`. It does not pass: tests whose
   member keys are ed25519 encodings fail with an `InvalidPersonKey`
   (`OrgMembersError` from `PersonPublicKey::parse`) panic or error.
   Deterministically: every test using `test_fixtures::admit_member_delta`
   or the inline bob fixture (seed `[2u8; 32]`): `verify_against_chain`
   (all of `setup()`'s users), `wire_frame_bound`, the `envelope.rs` unit
   tests and `transport_handshake`/`transport_networked`'s
   delivers-and-verifies tests. At random: the `OsRng`-persona tests in
   `service_stories` and `admission_sender`. `exit 101`.

7. Run APP-GATE with `--no-fail-fast` into `$S/plan4-t2-app.log`. Expected:
   it compiles and the app's 78 tests pass, `exit 0`. The app's tests do not
   exercise member keys. If any app test fails, record it and carry on: T8
   re-runs the gate.

8. Commit. The message records the per-target pass/fail counts the step-6 run printed:

   ```bash
   git add org-node/src/keys.rs org-node/src/service.rs org-node/src/transport/endpoint.rs org-node/src/test_fixtures.rs org-node/tests/transport_handshake.rs org-node/tests/transport_networked.rs org-node/tests/fuzz_verify_against_chain/fuzz_target.rs org-node/tests/service_stories.rs org-node/tests/admission_sender.rs
   ```
   ```bash
   git -c commit.gpgsign=false commit -m "fix(org-node): call person's parse/try_from constructors; compiles again, member keys still ed25519 (red: <per-target counts from the T2 run>)"
   ```

**Status: done (2026-10-05).**

- red -> green: none. T2 writes no test. Its check is that the crate
  compiles, and it does: ORG-NODE-GATE shows no `error[`.
- ORG-NODE-GATE (`$S/step4-t2-org-node.log`): `exit 101`, 29 tests fail
  and all 29 panic with `InvalidPersonKey`. Per target: lib 17 passed, 6
  failed (the `envelope.rs` unit tests); `admission_sender` 1 passed, 6
  failed; `service_stories` 0 passed, 3 failed; `transport_handshake` 2
  passed, 1 failed; `transport_networked` 0 passed, 1 failed;
  `verify_against_chain` 3 passed, 10 failed; `wire_frame_bound` 1 passed,
  2 failed; `store_at_rest` 4 passed. The three fuzz targets each ran for
  1 s without a failure.
- APP-GATE (`$S/step4-t2-app.log`): `exit 0`, 78 passed (10 + 9 + 14 + 27
  + 6 + 12).
- As planned, `chain_genesis_e2e.rs` still does not compile: two `E0308`
  at its two `vec![….device_key()]` lines. T7 fixes it.

---

### T3 — X25519 Member-as-a-group keys from `member_seed`

**Files touched:** `Cargo.lock`, `app/src-tauri/Cargo.lock`, `org-node/Cargo.toml`, `org-node/docs/architecture/soup.md`, `org-node/src/keys.rs`, `org-node/src/service.rs`, `org-node/src/test_fixtures.rs`, `org-node/src/envelope.rs`, `org-node/tests/verify_against_chain.rs`, `org-node/tests/wire_frame_bound.rs`, `org-node/tests/transport_handshake.rs`, `org-node/tests/transport_networked.rs`, `org-node/tests/fuzz_verify_against_chain/fuzz_target.rs`, `org-node/tests/service_stories.rs`
**Parallel:** no (serial, after T2)
**verifies:** none new. The keys.rs unit tests below are lib tests. `test_paths` reads only `org-node/tests`, so they carry no annotation. Every existing annotated test must pass again.

The persona's member key becomes the X25519 public key of `member_seed`.
The Envelope is still signed in this task. Until T4/T5 drop the signature,
the admin signs with `SigningKeypair::from_seed(member_seed)` (what
`admit_member` and `revoke_member` already do), and `create_organisation`
still publishes that ed25519 key as `org_pub_key`. The seed thus serves two
roles for the length of T3–T5. T5 removes the signer and T6 replaces the
published key. `receive_and_verify` keeps its T2 form in this task.

1. `org-node/Cargo.toml`, in `[dependencies]` after the `ed25519-dalek` line:

   ```toml
   # X25519 public keys for Member-as-a-group keys and the Organisation key:
   # MontgomeryPoint::mul_base_clamped (RFC 7748). Same 4.1.3 the lockfile
   # already resolves through ed25519-dalek.
   curve25519-dalek = "4"
   ```

2. `org-node/src/keys.rs`: replace the module doc comment, the imports and
   add `X25519Keypair`; remove `SigningKeypair::member_key`. The file becomes:

   ```rust
   //! Key material held locally. A device's key is ed25519: its verifying key
   //! is its DevicePublicKey in the trie and its iroh EndpointId. A
   //! Member-as-a-group key (and, from REQ-ech45n, the Organisation key) is an
   //! X25519 key-agreement key, derived from a 32-byte secret by RFC 7748
   //! clamping and multiplication by the base point.
   use core::fmt;

   use curve25519_dalek::montgomery::MontgomeryPoint;
   use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
   use org_members::{DevicePublicKey, OrgMembersError, PersonPublicKey};
   use rand_core::{CryptoRng, RngCore};

   /// An ed25519 keypair held locally: a device's identity. Wraps a dalek SigningKey.
   #[derive(Clone, Debug)]
   pub struct SigningKeypair(SigningKey);

   impl SigningKeypair {
       /// Generate from a CSPRNG. (Tests use rand; production wires this to the OS RNG.)
       pub fn generate<R: CryptoRng + RngCore>(rng: &mut R) -> Self {
           Self(SigningKey::generate(rng))
       }

       /// Reconstruct from the 32-byte secret seed (for persisted keys).
       pub fn from_seed(seed: [u8; 32]) -> Self {
           Self(SigningKey::from_bytes(&seed))
       }

       /// The 32-byte secret seed, for at-rest persistence. Handle as a secret.
       pub fn to_seed(&self) -> [u8; 32] {
           self.0.to_bytes()
       }

       pub fn verifying_key(&self) -> VerifyingKey {
           self.0.verifying_key()
       }

       /// As a device key for the trie / iroh identity, through `person`'s parse.
       pub fn device_key(&self) -> Result<DevicePublicKey, OrgMembersError> {
           Ok(DevicePublicKey::try_from(self.verifying_key())?)
       }

       pub fn sign(&self, msg: &[u8]) -> Signature {
           self.0.sign(msg)
       }
   }

   /// Verify a signature against an already-known verifying key.
   pub fn verify(vk: &VerifyingKey, msg: &[u8], sig: &Signature) -> bool {
       vk.verify(msg, sig).is_ok()
   }

   /// An X25519 secret held locally, as the 32 bytes it is persisted as.
   #[derive(Clone)]
   pub struct X25519Keypair([u8; 32]);

   impl X25519Keypair {
       /// 32 bytes from the caller's cryptographic random source.
       pub fn generate<R: CryptoRng + RngCore>(rng: &mut R) -> Self {
           let mut seed = [0u8; 32];
           rng.fill_bytes(&mut seed);
           Self(seed)
       }

       /// Reconstruct from the persisted 32 bytes.
       pub fn from_seed(seed: [u8; 32]) -> Self {
           Self(seed)
       }

       /// The 32 secret bytes, for at-rest persistence. Handle as a secret.
       pub fn to_seed(&self) -> [u8; 32] {
           self.0
       }

       /// The X25519 public key: the clamped secret times the base point
       /// (RFC 7748 §5).
       pub fn public_bytes(&self) -> [u8; 32] {
           MontgomeryPoint::mul_base_clamped(self.0).to_bytes()
       }

       /// As a Member-as-a-group key for the trie, through `person`'s parse.
       pub fn member_key(&self) -> Result<PersonPublicKey, OrgMembersError> {
           Ok(PersonPublicKey::parse(&self.public_bytes())?)
       }
   }

   /// Redacted: the secret never reaches a log.
   impl fmt::Debug for X25519Keypair {
       fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
           f.write_str("X25519Keypair(..)")
       }
   }

   #[cfg(test)]
   mod tests {
       use super::*;
       use rand::rngs::OsRng;

       fn hex32(s: &str) -> [u8; 32] {
           let mut out = [0u8; 32];
           for (i, byte) in out.iter_mut().enumerate() {
               *byte = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap();
           }
           out
       }

       #[test]
       fn sign_verify_round_trip() {
           let kp = SigningKeypair::generate(&mut OsRng);
           let msg = b"hello org";
           let sig = kp.sign(msg);
           assert!(verify(&kp.verifying_key(), msg, &sig));
           assert!(!verify(&kp.verifying_key(), b"tampered", &sig));
       }

       #[test]
       fn seed_round_trip_preserves_key() {
           let kp = SigningKeypair::generate(&mut OsRng);
           let kp2 = SigningKeypair::from_seed(kp.to_seed());
           assert_eq!(kp.verifying_key(), kp2.verifying_key());
       }

       #[test]
       fn device_key_wraps_the_verifying_key() {
           let kp = SigningKeypair::generate(&mut OsRng);
           assert_eq!(kp.device_key().unwrap().as_bytes(), kp.verifying_key().as_bytes());
       }

       // RFC 7748 §6.1: Alice's and Bob's private keys and X25519(k, 9).
       #[test]
       fn x25519_public_key_matches_rfc_7748() {
           let alice = X25519Keypair::from_seed(hex32(
               "77076d0a7318a57d3c16c17251b26645df4c2f87ebc0992ab177fba51db92c2a",
           ));
           assert_eq!(
               alice.public_bytes(),
               hex32("8520f0098930a754748b7ddcb43ef75a0dbf3a0d26381af4eba4a98eaa9b4e6a")
           );
           let bob = X25519Keypair::from_seed(hex32(
               "5dab087e624a8a4b79e17f8b83800ee66f3bb1292618b6fd1c2f8b27ff88e0eb",
           ));
           assert_eq!(
               bob.public_bytes(),
               hex32("de9edb7d7b7dc1b4d35b61c2ece435373f8343c85b78674dadfc7e146f882b4f")
           );
       }

       #[test]
       fn member_key_is_the_x25519_public_key_and_always_valid() {
           for _ in 0..256 {
               let kp = X25519Keypair::generate(&mut OsRng);
               let key = kp.member_key().expect("an X25519 public key is a valid PersonPublicKey");
               assert_eq!(key.as_bytes(), &kp.public_bytes());
           }
       }

       #[test]
       fn x25519_seed_round_trip_preserves_key() {
           let kp = X25519Keypair::generate(&mut OsRng);
           assert_eq!(X25519Keypair::from_seed(kp.to_seed()).public_bytes(), kp.public_bytes());
       }

       #[test]
       fn x25519_debug_does_not_print_the_secret() {
           let kp = X25519Keypair::from_seed([0xabu8; 32]);
           assert_eq!(format!("{kp:?}"), "X25519Keypair(..)");
       }
   }
   ```

3. `org-node/src/test_fixtures.rs`: the fixtures' member keys become X25519.
   Replace from `use crate::keys::SigningKeypair;` to the end of the file with:

   ```rust
   use crate::keys::{SigningKeypair, X25519Keypair};

   pub type Trie = OrgTrie<Blake3Hasher>;

   /// Bundles a member's keys + a stable id for building leaves.
   pub struct NodeFixture {
       pub member: X25519Keypair,
       pub device: SigningKeypair,
       pub id: MemberId,
   }

   /// Build a MemberLeaf from a fixture with a fixed handle/name.
   pub fn member(fix: &NodeFixture, handle: &str) -> MemberLeaf {
       MemberLeaf::new(
           fix.id,
           Handle::parse(handle).unwrap(),
           fix.member.member_key().unwrap(),
           Name::parse("Test").unwrap(),
           Surname::parse("User").unwrap(),
           vec![fix.device.device_key().unwrap()],
       )
       .unwrap()
   }

   /// A genesis trie containing a single admin member (id = [1u8;32]).
   pub fn genesis_trie(admin: &X25519Keypair, admin_device: &SigningKeypair) -> Trie {
       let admin_fix = NodeFixture {
           member: admin.clone(),
           device: admin_device.clone(),
           id: MemberId::new([1u8; 32]),
       };
       let leaf = member(&admin_fix, "admin");
       Trie::genesis(vec![leaf]).unwrap()
   }

   /// Seed of the admin's device keypair. Distinct from every other seed the
   /// fixtures use ([1u8;32] admin member, [2u8;32] and [3u8;32] bob), because
   /// org-members refuses an organisation in which one key is held twice
   /// (`OrgMembersError::DuplicateKey`).
   pub const ADMIN_DEVICE_SEED: [u8; 32] = [4u8; 32];

   /// The admin's device keypair: a key of its own, never the admin's member key.
   pub fn admin_device() -> SigningKeypair {
       SigningKeypair::from_seed(ADMIN_DEVICE_SEED)
   }

   /// Seed of bob's device keypair in [`admit_member_delta`].
   pub const BOB_DEVICE_SEED: [u8; 32] = [3u8; 32];

   /// Bob's device keypair: the device [`admit_member_delta`] enrols.
   pub fn bob_device() -> SigningKeypair {
       SigningKeypair::from_seed(BOB_DEVICE_SEED)
   }

   /// Build the "admit member B (id=[2u8;32])" delta against a genesis trie
   /// whose admin member key is `admin` and whose admin device is
   /// [`admin_device`]. Returns (delta, new_trie).
   pub fn admit_member_delta(admin: &X25519Keypair) -> (Delta, Trie) {
       let base = genesis_trie(admin, &admin_device());
       let b_fix = NodeFixture {
           member: X25519Keypair::from_seed([2u8; 32]),
           device: bob_device(),
           id: MemberId::new([2u8; 32]),
       };
       let leaf = member(&b_fix, "bob");
       let (new_trie, delta) = base.add_member(leaf).unwrap().recalculate().unwrap();
       (delta, new_trie)
   }
   ```

4. `org-node/src/envelope.rs` tests: in each of the six tests, keep `admin`
   as the signer and give the delta an X25519 admin. Add
   `use crate::keys::X25519Keypair;` to the test module, and in each test
   replace `admit_member_delta(&admin)` with
   `admit_member_delta(&X25519Keypair::generate(&mut OsRng))`.

5. `org-node/src/service.rs`: add `X25519Keypair` to the import
   (`use crate::keys::{SigningKeypair, X25519Keypair};`) and change:
   - `create_persona`:

     ```rust
             let member_kp = X25519Keypair::generate(rng);
             let device_kp = SigningKeypair::generate(rng);
             // Derive a unique persona_id from the Member-as-a-group key.
             let persona_id = hex_id(&member_kp.public_bytes());
     ```
     (the record's `member_seed: member_kp.to_seed(),` is unchanged.)
   - `persona_keys` returns `(X25519Keypair, SigningKeypair, String, String, String)`
     and its first element is `X25519Keypair::from_seed(p.member_seed)`.
   - `create_organisation`, after `let genesis_root = *genesis_root_hash.as_bytes();`:

     ```rust
             // Interim (T3–T5): the Envelope is still signed with the ed25519
             // key of the member seed, and that key is still what is published.
             // REQ-ech45n replaces it with a fresh Organisation key.
             let signer = SigningKeypair::from_seed(member_kp.to_seed());
             let org_pub_key = *signer.verifying_key().as_bytes();
     ```
     and in `admin_snap` and `org_rec`,
     `member_key: member_kp.public_bytes(),` and
     `admin_member_key: member_kp.public_bytes(),`.
   - `export_join_request`: delete `let member_kp = SigningKeypair::from_seed(persona.member_seed);`
     and set `member_key: X25519Keypair::from_seed(persona.member_seed).public_bytes(),`.
   - `admin_persona_for_org`: the `find` closure becomes
     `.find(|p| X25519Keypair::from_seed(p.member_seed).public_bytes() == org_rec.admin_member_key)`.

6. Tests that build member leaves:
   - `org-node/tests/verify_against_chain.rs`, `setup()`: keep
     `let admin = SigningKeypair::from_seed([1u8; 32]);` (the signer) and add
     `let member = X25519Keypair::from_seed([1u8; 32]);`. Use
     `genesis_trie(&member, &admin_device())` and
     `admit_member_delta(&member)`. Import
     `use org_node::keys::{SigningKeypair, X25519Keypair};`.
   - `org-node/tests/wire_frame_bound.rs`, `sample_msg()`:
     `let (delta, _) = admit_member_delta(&X25519Keypair::from_seed([1u8; 32]));`
     with the same import change.
   - `org-node/tests/transport_handshake.rs` and
     `org-node/tests/transport_networked.rs`: `genesis_and_admit(admin: &X25519Keypair, admin_device: &SigningKeypair)`.
     Bob's member is `let b_member = X25519Keypair::from_seed([2u8; 32]);`
     (`member_key()` calls unchanged). In the test, after
     `let admin = SigningKeypair::from_seed([1u8; 32]);` add
     `let member = X25519Keypair::from_seed([1u8; 32]);` and call
     `genesis_and_admit(&member, &a_device)`. Import
     `use org_node::keys::{SigningKeypair, X25519Keypair};`. Delete the
     comment line
     `// admin.member_key().expect("valid key").as_bytes() == admin.verifying_key().as_bytes()`
     in transport_handshake.
   - `org-node/tests/fuzz_verify_against_chain/fuzz_target.rs`:
     `fn fixed_trie(admin: &X25519Keypair, admin_device: &SigningKeypair)`.
     In `main`, add `let member = X25519Keypair::from_seed([1u8; 32]);` and
     `let local = fixed_trie(&member, &admin_device);`, keeping
     `let admin = SigningKeypair::from_seed([1u8; 32]);` for `vk`. Import
     `use org_node::keys::{SigningKeypair, X25519Keypair};`.
   - `org-node/tests/service_stories.rs`,
     `unverified_revocation_leaves_the_record_in_place`: `keys_of` returns
     `(X25519Keypair, SigningKeypair)` built from
     `X25519Keypair::from_seed(p.member_seed)` and
     `SigningKeypair::from_seed(p.device_seed)`. The forger assertion becomes

     ```rust
         assert_ne!(
             *forger.verifying_key().as_bytes(),
             org_pub_key,
             "the forging key must not be the Organisation's published signing key"
         );
     ```
     Add `use org_node::keys::X25519Keypair;` inside that test.

7. `org-node/docs/architecture/soup.md`: replace the empty row `| | | | | |` with:

   ```markdown
   | `curve25519-dalek` | 4.1.3 | Direct dependency of org-node: `MontgomeryPoint::mul_base_clamped`, which `X25519Keypair::public_bytes` (`org-node/src/keys.rs`) calls to derive the X25519 public key of a member seed and of the Organisation secret (RFC 7748 clamping, then multiplication by the base point). Also reached through `ed25519-dalek` for every Device key. | REQ-ech45n | **Safety-relevant.** A wrong multiplication would place a Member-as-a-group key, or publish an Organisation public key, that does not belong to the secret the node holds: key agreement against that member key fails, and a member could not confirm an Organisation private key shared with it (not yet implemented in org-node). Every derived key is re-checked before use by `person`'s X25519 validity rule (`PersonPublicKey::parse`, and from REQ-8jb4ny `OrgPublicKey::parse`). The RFC 7748 §6.1 vectors are a unit test in `keys.rs`. The 5.0.0-pre.6 pre-release in the lockfile is not reached from org-node. |
   ```

8. Run ORG-NODE-GATE into `$S/plan4-t3-org-node.log`. Expected: every
   target `test result: ok`, `exit 0`. The lib's `keys` module goes from 3
   tests to 7, so the lib total rises by 4 over whatever T2's run listed for
   the lib. Record the per-target counts.
   `Cargo.lock` and `app/src-tauri/Cargo.lock` gain `curve25519-dalek` in
   org-node's dependency list. No new package is resolved.

9. Commit:

   ```bash
   git add Cargo.lock app/src-tauri/Cargo.lock org-node/Cargo.toml org-node/docs/architecture/soup.md org-node/src/keys.rs org-node/src/service.rs org-node/src/test_fixtures.rs org-node/src/envelope.rs org-node/tests/verify_against_chain.rs org-node/tests/wire_frame_bound.rs org-node/tests/transport_handshake.rs org-node/tests/transport_networked.rs org-node/tests/fuzz_verify_against_chain/fuzz_target.rs org-node/tests/service_stories.rs
   ```
   ```bash
   git -c commit.gpgsign=false commit -m "feat(org-node): X25519 Member-as-a-group keys from member_seed (mul_base_clamped); curve25519-dalek direct, SOUP row"
   ```

**Status: done (2026-10-05).**

- red -> green: `keys::tests::x25519_public_key_matches_rfc_7748`,
  `keys::tests::member_key_is_the_x25519_public_key_and_always_valid`,
  `keys::tests::x25519_seed_round_trip_preserves_key`,
  `keys::tests::x25519_debug_does_not_print_the_secret` (lib tests, no
  annotation): each watched fail before `X25519Keypair` existed, the lib
  test target refusing to compile with six
  `E0433 cannot find type X25519Keypair` (`$S/step4-t3-red.log`). Each
  passes after step 2. `device_key_wraps_the_verifying_key` replaces
  `member_and_device_keys_wrap_the_verifying_key`, whose member half went
  with `SigningKeypair::member_key`.
- ORG-NODE-GATE (`$S/step4-t3-org-node.log`): `exit 0`. Per target: lib 27
  passed (T2's 23 plus 4); `admission_sender` 7; `service_stories` 3;
  `store_at_rest` 4; `transport_handshake` 3; `transport_networked` 1;
  `verify_against_chain` 13; `wire_frame_bound` 3; the three fuzz targets
  each ran without a failure.
- APP-GATE (`$S/step4-t3-app.log`): `exit 0`, 78 passed (10 + 9 + 14 + 27
  + 6 + 12).
- Deviation: in `wire_frame_bound.rs`, `sample_msg()` keeps
  `let admin = SigningKeypair::from_seed([1u8; 32]);`, because the
  Envelope is still signed until T4. Step 6 dropped it.
- `chain_genesis_e2e.rs` now fails to compile with four errors: the two
  `E0308` from T2 and two `E0599` for the removed
  `SigningKeypair::member_key`. T7 rewrites that file.

---

### T4 — Drop the Envelope signature; the sender-device check in verification

**Files touched:** `org-node/src/envelope.rs`, `org-node/src/verify.rs`, `org-node/src/error.rs`, `org-node/src/keys.rs`, `org-node/src/lib.rs`, `org-node/src/transport/wire.rs`, `org-node/docs/CONTEXT.md`, `org-node/tests/verify_against_chain.rs`, `org-node/tests/wire_frame_bound.rs`, `org-node/tests/transport_handshake.rs`, `org-node/tests/transport_networked.rs`, `org-node/tests/fuzz_envelope_decode/fuzz_target.rs`, `org-node/tests/fuzz_verify_against_chain/fuzz_target.rs`
**Parallel:** no (serial, after T3)
**verifies:** `REQ-nhe2zu, REQ-txvtm9`; `REQ-ag6kqm, REQ-7h7qp3` (two rewritten tests); `REQ-7h7qp3` (one new test); `REQ-wp2nyc, REQ-nhe2zu, REQ-txvtm9, REQ-mr5abb`

The glossary already says "Envelope" and lists "delta envelope" under
_Avoid_, so `SignedDeltaEnvelope` becomes `Envelope`. `BadSignature` names a
check that no longer exists and becomes `UnknownSender`. The sender check
moves into `verify_envelope_against_chain` as step 2, where the signature
check was, so it precedes the replay check and the decode as RC-2e6k44
requires. The caller passes the authenticated sender and the accepted set.
`org-node/src/service.rs` does not compile after this task. T5 fixes it. The
gate here therefore omits `app`.

1. Write the failing tests first. Replace `org-node/tests/verify_against_chain.rs` with:

   ```rust
   #![cfg(feature = "test-support")]
   #![allow(clippy::unwrap_used, clippy::expect_used)]
   //! verify-against-chain: the receive-side commit rule (verify.rs) and the
   //! replay guard it leans on (sequence.rs), tested at the crate's public
   //! interface. Nine of these tests were relocated from the `#[cfg(test)]`
   //! modules of `src/verify.rs` and `src/sequence.rs` and annotated; the rest
   //! are the abnormal-input cases the org-node risk analysis added. What the
   //! fix rounds added to the relocated bodies is recorded in
   //! `docs/plans/2026-09-09-org-node-risk-analysis.md`.
   //!
   //! 2026-10-05: the Envelope carries no signature. The check that stood in
   //! its place — the sender is one of the Device keys the receiver accepts
   //! (REQ-7h7qp3, which supersedes REQ-ag6kqm) — is tested here at the same
   //! position in the order: after the org binding, before the replay check
   //! and the decode.

   use org_members::{DevicePublicKey, RootHash};
   use org_node::chain::{MockChain, OrgState};
   use org_node::ids::OrgId;
   use org_node::keys::{SigningKeypair, X25519Keypair};
   use org_node::sequence::SeqGuard;
   use org_node::test_fixtures::{admin_device, admit_member_delta, genesis_trie, Trie};
   use org_node::verify::{verify_envelope_against_chain, VerifyContext};
   use org_node::{Envelope, OrgNodeError};

   fn setup() -> (OrgId, Trie, Envelope, RootHash) {
       let admin = X25519Keypair::from_seed([1u8; 32]);
       let local = genesis_trie(&admin, &admin_device()); // receiver's mirror (epoch 1 state)
       // NOTE: admit_member_delta builds its own genesis internally from the same
       // admin and admin_device(); both genesis tries agree by construction
       // (deterministic fixtures).
       let (delta, new_trie) = admit_member_delta(&admin);
       let org = OrgId::new([5u8; 20]);
       let env = Envelope::build(org, 2, &delta).unwrap();
       let new_root = new_trie.root_hash().unwrap();
       (org, local, env, new_root)
   }

   /// The admin's Device key as the connection authenticates it, and the
   /// accepted set of a receiver whose record holds that device alone.
   fn admin_sender() -> (DevicePublicKey, [DevicePublicKey; 1]) {
       let key = admin_device().device_key().unwrap();
       (key, [key])
   }

   /// A device in no record: neither the admin's nor bob's.
   fn imposter() -> DevicePublicKey {
       SigningKeypair::from_seed([0xaa; 32]).device_key().unwrap()
   }

   /// The receiver's context: expects `org`, was sent the envelope by
   /// `sender`, accepts `accepted`, has committed `parent_seq` 1 at epoch 1.
   fn ctx<'a>(
       org: OrgId,
       sender: &'a DevicePublicKey,
       accepted: &'a [DevicePublicKey],
   ) -> VerifyContext<'a> {
       VerifyContext {
           expected_org_id: org,
           sender,
           accepted_senders: accepted,
           seq_guard: SeqGuard::from_last_seen(1),
           last_committed_epoch: 1,
       }
   }

   /// A chain whose state for `org` is `root` at `epoch`.
   fn chain_at(org: OrgId, root: RootHash, epoch: u64) -> MockChain {
       let mut chain = MockChain::new();
       chain.set(org, OrgState { root_hash: root, org_pub_key: [0u8; 32], epoch });
       chain
   }

   /// An envelope for `org` at `seq` whose Change set bytes do not decode.
   fn garbage(org: OrgId, seq: u64) -> Envelope {
       Envelope { org_id: org, parent_seq: seq, delta_bytes: vec![0xff; 16] }
   }

   // ---- relocated from src/verify.rs ------------------------------------------

   // REQ-txvtm9 is REQ-nhe2zu without the signature clause: the envelope names
   // the expected Organisation, arrives from an accepted Device key, carries a
   // newer Sequence number and reaches the chain's root at a newer epoch.
   // verifies: REQ-nhe2zu, REQ-txvtm9
   #[test]
   fn happy_path_commits_when_root_matches_chain() {
       let (org, local, env, new_root) = setup();
       let chain = chain_at(org, new_root, 2);
       let (sender, accepted) = admin_sender();
       let ctx = ctx(org, &sender, &accepted);
       let out = verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap();
       assert_eq!(out.epoch, 2);
       assert_eq!(out.seq_guard.last_seen(), 2);
       assert_eq!(out.trie.root_hash().unwrap(), new_root);
   }

   // verifies: REQ-gju89b
   #[test]
   fn rejects_wrong_org_id() {
       let (_org, local, env, _) = setup();
       let chain = MockChain::new();
       let (sender, accepted) = admin_sender();
       let ctx = ctx(OrgId::new([0xff; 20]), &sender, &accepted);
       assert_eq!(
           verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap_err(),
           OrgNodeError::OrgIdMismatch
       );
   }

   // Was `rejects_bad_signature`: the envelope a device outside the accepted
   // set delivers is rejected, whatever it carries. The same envelope from the
   // admin's device verifies (the happy path above), so the sender alone
   // decides.
   // verifies: REQ-ag6kqm, REQ-7h7qp3
   #[test]
   fn rejects_a_sender_outside_the_accepted_devices() {
       let (org, local, env, new_root) = setup();
       let chain = chain_at(org, new_root, 2);
       let (_admin, accepted) = admin_sender();
       let sender = imposter();
       let ctx = ctx(org, &sender, &accepted);
       assert_eq!(
           verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap_err(),
           OrgNodeError::UnknownSender
       );
   }

   // A receiver with nothing to accept a sender against (a first admission
   // with no imported Invite) accepts no sender, the admin's included.
   // verifies: REQ-7h7qp3
   #[test]
   fn an_empty_accepted_set_accepts_no_sender() {
       let (org, local, env, new_root) = setup();
       let chain = chain_at(org, new_root, 2);
       let (sender, _accepted) = admin_sender();
       let ctx = ctx(org, &sender, &[]);
       assert_eq!(
           verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap_err(),
           OrgNodeError::UnknownSender
       );
   }

   // verifies: REQ-6yu72z
   #[test]
   fn rejects_stale_seq() {
       let (org, local, env, new_root) = setup();
       let chain = chain_at(org, new_root, 2);
       let (sender, accepted) = admin_sender();
       let ctx = VerifyContext {
           seq_guard: SeqGuard::from_last_seen(2), // env.parent_seq == 2, not > 2
           ..ctx(org, &sender, &accepted)
       };
       assert_eq!(
           verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap_err(),
           OrgNodeError::StaleSeq { got: 2, last_seen: 2 }
       );
   }

   // verifies: REQ-bvh8v6
   #[test]
   fn rejects_when_org_absent_from_chain() {
       let (org, local, env, _) = setup();
       let chain = MockChain::new(); // empty
       let (sender, accepted) = admin_sender();
       let ctx = ctx(org, &sender, &accepted);
       assert_eq!(
           verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap_err(),
           OrgNodeError::OrgNotOnChain
       );
   }

   // REQ-nhe2zu's commit rule, and REQ-txvtm9's after it, is conditional, so
   // withholding the commit when the recomputed root does not match the
   // chain's is that requirement's abnormal-input case as well as REQ-wp2nyc's
   // rejection.
   //
   // It is also REQ-mr5abb's last step: the root match is the final check, so an
   // envelope rejected here has passed the sequence check and would have had its
   // Sequence number committed had the root matched. The mark must not move.
   // `verify_envelope_against_chain` takes the context by shared reference and
   // returns the advanced guard only inside `Ok`, so the guard to assert on is
   // the one this test handed in.
   // verifies: REQ-wp2nyc, REQ-nhe2zu, REQ-txvtm9, REQ-mr5abb
   #[test]
   fn rejects_root_mismatch_when_chain_root_differs() {
       let (org, local, env, _new_root) = setup();
       // Attacker-influenced delta but honest chain root that does NOT match.
       let chain = chain_at(org, RootHash::new([0xde; 32]), 2);
       let (sender, accepted) = admin_sender();
       let ctx = ctx(org, &sender, &accepted);
       assert_eq!(ctx.seq_guard.last_seen(), 1, "the mark this test hands in");
       assert_eq!(
           verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap_err(),
           OrgNodeError::RootMismatch
       );
       assert_eq!(
           ctx.seq_guard.last_seen(),
           1,
           "a rejection at the root match must leave the high-water mark where it was"
       );
   }

   // verifies: REQ-8gz8bu
   #[test]
   fn rejects_stale_epoch() {
       let (org, local, env, new_root) = setup();
       let chain = chain_at(org, new_root, 1); // chain epoch 1 is not newer than committed 1
       let (sender, accepted) = admin_sender();
       let ctx = ctx(org, &sender, &accepted);
       assert_eq!(
           verify_envelope_against_chain(&local, &env, &ctx, &chain).unwrap_err(),
           OrgNodeError::StaleEpoch { got: 1, last: 1 }
       );
   }

   // ---- relocated from src/sequence.rs ----------------------------------------

   // verifies: REQ-6yu72z
   #[test]
   fn rejects_equal_and_lower_seq() {
       let g = SeqGuard::from_last_seen(5);
       assert!(g.check(6).is_ok());
       assert_eq!(g.check(5), Err(OrgNodeError::StaleSeq { got: 5, last_seen: 5 }));
       assert_eq!(g.check(4), Err(OrgNodeError::StaleSeq { got: 4, last_seen: 5 }));
   }

   // verifies: REQ-mr5abb
   #[test]
   fn advance_moves_high_water_mark_forward_only() {
       let mut g = SeqGuard::new();
       g.advance(3);
       assert_eq!(g.last_seen(), 3);
       g.advance(2); // ignored
       assert_eq!(g.last_seen(), 3);
   }

   // ---- abnormal input: the cheap checks run before the delta is decoded ------

   // verifies: REQ-gju89b
   #[test]
   fn rejects_wrong_org_before_decoding_delta() {
       // Garbage delta bytes from an accepted sender: if the org check did not
       // come first, the error would be MalformedDelta.
       let (org, local, _env, _) = setup();
       let (sender, accepted) = admin_sender();
       let ctx = ctx(OrgId::new([0xee; 20]), &sender, &accepted);
       assert_eq!(
           verify_envelope_against_chain(&local, &garbage(org, 2), &ctx, &MockChain::new())
               .unwrap_err(),
           OrgNodeError::OrgIdMismatch
       );
   }

   // Was `rejects_bad_signature_before_decoding_delta`: garbage delta bytes
   // for the expected org at a fresh Sequence number, delivered by a device
   // outside the accepted set. If the sender check did not precede decoding,
   // the error would be MalformedDelta.
   // verifies: REQ-ag6kqm, REQ-7h7qp3
   #[test]
   fn rejects_unknown_sender_before_decoding_delta() {
       let (org, local, _env, _) = setup();
       let (_admin, accepted) = admin_sender();
       let sender = imposter();
       let ctx = ctx(org, &sender, &accepted);
       assert_eq!(
           verify_envelope_against_chain(&local, &garbage(org, 2), &ctx, &MockChain::new())
               .unwrap_err(),
           OrgNodeError::UnknownSender
       );
   }

   // verifies: REQ-6yu72z
   #[test]
   fn rejects_stale_seq_before_decoding_delta() {
       // Garbage delta bytes at parent_seq 1, from an accepted sender, against a
       // guard that has already seen 1: if the replay check did not precede
       // decoding, the error would be MalformedDelta.
       let (org, local, _env, _) = setup();
       let (sender, accepted) = admin_sender();
       let ctx = ctx(org, &sender, &accepted); // SeqGuard::from_last_seen(1)
       assert_eq!(
           verify_envelope_against_chain(&local, &garbage(org, 1), &ctx, &MockChain::new())
               .unwrap_err(),
           OrgNodeError::StaleSeq { got: 1, last_seen: 1 }
       );
   }

   // verifies: REQ-mr5abb
   #[test]
   fn check_does_not_advance_the_mark() {
       // `check` takes `&self`: a rejection (or an acceptance that is later
       // rejected downstream) can never move the high-water mark. The assertion
       // documents the contract the type system enforces.
       let g = SeqGuard::from_last_seen(5);
       g.check(6).unwrap();
       assert_eq!(g.last_seen(), 5);
   }
   ```

   Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features test-support --test verify_against_chain`.
   Expected: does not compile (`cannot find type Envelope`, `no field sender`,
   `no variant UnknownSender`). This is the failing test.

2. `org-node/src/envelope.rs` becomes:

   ```rust
   //! Envelope: the wire form for a trie change, bound to an Organisation and
   //! a Sequence number. It carries no signature (owner, 2026-10-05): the
   //! connection authenticates the sending device, and verification checks
   //! that device (REQ-7h7qp3) before it decodes the Change set.
   use org_members::delta::Delta;
   use serde::{Deserialize, Serialize};

   use crate::error::OrgNodeError;
   use crate::ids::OrgId;

   /// An org-bound, sequence-bound trie delta.
   #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
   pub struct Envelope {
       pub org_id: OrgId,
       pub parent_seq: u64,
       pub delta_bytes: Vec<u8>, // postcard(Delta)
   }

   impl Envelope {
       /// Author side: encode `delta` and bind it to (org, seq).
       pub fn build(org_id: OrgId, parent_seq: u64, delta: &Delta) -> Result<Self, OrgNodeError> {
           // to_allocvec on a valid Delta is infallible in practice; reuse MalformedDelta for the unreachable encode error.
           let delta_bytes = postcard::to_allocvec(delta).map_err(|_| OrgNodeError::MalformedDelta)?;
           Ok(Self { org_id, parent_seq, delta_bytes })
       }

       /// Decode the inner Delta from postcard bytes.
       pub fn decode_delta(&self) -> Result<Delta, OrgNodeError> {
           postcard::from_bytes(&self.delta_bytes).map_err(|_| OrgNodeError::MalformedDelta)
       }
   }

   #[cfg(test)]
   mod tests {
       use super::*;
       use crate::keys::X25519Keypair;
       use crate::test_fixtures::admit_member_delta;
       use rand::rngs::OsRng;

       #[test]
       fn build_binds_org_and_sequence_to_the_encoded_delta() {
           let (delta, _) = admit_member_delta(&X25519Keypair::generate(&mut OsRng));
           let env = Envelope::build(OrgId::new([5u8; 20]), 7, &delta).unwrap();
           assert_eq!(env.org_id, OrgId::new([5u8; 20]));
           assert_eq!(env.parent_seq, 7);
           assert_eq!(env.delta_bytes, postcard::to_allocvec(&delta).unwrap());
       }

       #[test]
       fn decode_delta_round_trips() {
           let (delta, _) = admit_member_delta(&X25519Keypair::generate(&mut OsRng));
           let env = Envelope::build(OrgId::new([5u8; 20]), 1, &delta).unwrap();
           // Delta doesn't implement PartialEq; verify round-trip via re-encoding.
           let decoded = env.decode_delta().unwrap();
           assert_eq!(env.delta_bytes, postcard::to_allocvec(&decoded).unwrap());
       }

       #[test]
       fn decode_delta_rejects_bytes_that_are_not_a_delta() {
           let env = Envelope { org_id: OrgId::new([5u8; 20]), parent_seq: 1, delta_bytes: vec![0xff; 16] };
           assert_eq!(env.decode_delta().unwrap_err(), OrgNodeError::MalformedDelta);
       }

       #[test]
       fn the_wire_form_has_no_signature_field() {
           // OrgId is `[u8; 20]` (serde tuple, no length prefix) ‖ varint
           // parent_seq ‖ varint len ‖ bytes: nothing after.
           let env = Envelope { org_id: OrgId::new([5u8; 20]), parent_seq: 1, delta_bytes: vec![9, 9] };
           let bytes = postcard::to_allocvec(&env).unwrap();
           assert_eq!(bytes.len(), 20 + 1 + 1 + 2);
       }
   }
   ```

3. `org-node/src/error.rs`: replace the `BadSignature` variant with

   ```rust
       /// The Device key the connection authenticated is not one this node
       /// accepts as a sender for the Organisation (REQ-7h7qp3, REQ-xa6smf,
       /// REQ-ztdza4).
       #[error("envelope sender is not a device this node accepts for the organisation")]
       UnknownSender,
   ```

4. `org-node/src/verify.rs` becomes:

   ```rust
   //! verify-against-chain: the single security property of the PoC. A received
   //! envelope is committed only if it came from a device the receiver accepts
   //! and applying its delta reproduces a root that independently matches the
   //! on-chain root at a newer epoch. See spec §5.2.
   use org_members::hasher::Blake3Hasher;
   use org_members::trie::OrgTrie;
   use org_members::DevicePublicKey;

   use crate::chain::ChainReader;
   use crate::envelope::Envelope;
   use crate::error::OrgNodeError;
   use crate::ids::OrgId;
   use crate::sequence::SeqGuard;

   pub type Trie = OrgTrie<Blake3Hasher>;

   /// Inputs that pin what the receiver already trusts about the org.
   pub struct VerifyContext<'a> {
       /// The org we expect this envelope to be for.
       pub expected_org_id: OrgId,
       /// The Device key the connection authenticated: who delivered the envelope.
       pub sender: &'a DevicePublicKey,
       /// The Device keys the sender must be one of (REQ-7h7qp3): on a first
       /// admission, the administrator's Device key named by the imported
       /// Invite; otherwise every Device key in the receiver's own current
       /// record. An empty slice accepts no sender.
       pub accepted_senders: &'a [DevicePublicKey],
       /// Replay guard for this org.
       pub seq_guard: SeqGuard,
       /// The last on-chain epoch this receiver has already committed (0 if none).
       pub last_committed_epoch: u64,
   }

   /// The result of a successful verification: the new committed trie and the
   /// advanced guards. Caller persists these atomically.
   ///
   /// `Debug` is derived for test convenience; `OrgTrie`'s `Debug` output includes
   /// the full node tree (member leaves redact PII, but it is still large). Avoid
   /// logging `VerifiedUpdate` at trace/debug level in production code.
   #[derive(Debug)]
   pub struct VerifiedUpdate {
       pub trie: Trie,
       pub seq_guard: SeqGuard,
       pub epoch: u64,
   }

   /// Verify an envelope against the local trie and an independent chain oracle.
   ///
   /// Order is security-critical: cheap checks on what the envelope claims and
   /// who sent it first, chain read and root match last. Returns the committed
   /// trie or a typed rejection; never panics, never mutates `local_trie`.
   pub fn verify_envelope_against_chain<C: ChainReader>(
       local_trie: &Trie,
       envelope: &Envelope,
       ctx: &VerifyContext<'_>,
       chain: &C,
   ) -> Result<VerifiedUpdate, OrgNodeError> {
       // 1. Org binding.
       if envelope.org_id != ctx.expected_org_id {
           return Err(OrgNodeError::OrgIdMismatch);
       }
       // 2. Sender — before touching delta bytes (REQ-7h7qp3, RC-2e6k44).
       if !ctx.accepted_senders.contains(ctx.sender) {
           return Err(OrgNodeError::UnknownSender);
       }
       // 3. Replay.
       ctx.seq_guard.check(envelope.parent_seq)?;
       // 4. Decode the delta (typed error on malformed/non-canonical wire form).
       let delta = envelope.decode_delta()?;
       // 5. Base-root must match the local trie (apply_delta also checks this, but
       //    we surface the specific error before doing work).
       if delta.base_root() != &local_trie.root_hash()? {
           return Err(OrgNodeError::DeltaBaseMismatch);
       }
       // 6. Apply → candidate.
       let candidate = local_trie.apply_delta(&delta)?;
       // 7. Independent trusted root + epoch from the chain.
       let on_chain = chain
           .get_org_state(&ctx.expected_org_id)
           .map_err(OrgNodeError::Chain)?
           .ok_or(OrgNodeError::OrgNotOnChain)?;
       if on_chain.epoch <= ctx.last_committed_epoch {
           return Err(OrgNodeError::StaleEpoch { got: on_chain.epoch, last: ctx.last_committed_epoch });
       }
       // 8. The decisive check: recomputed root must equal the on-chain root.
       let committed = candidate
           .verify_against(&on_chain.root_hash)
           .map_err(|_| OrgNodeError::RootMismatch)?;

       let mut seq_guard = ctx.seq_guard;
       seq_guard.advance(envelope.parent_seq);
       Ok(VerifiedUpdate { trie: committed, seq_guard, epoch: on_chain.epoch })
   }
   ```

5. `org-node/src/keys.rs`: delete `SigningKeypair::sign`, the free function
   `verify`, and the test `sign_verify_round_trip`. The `ed25519_dalek`
   import becomes `use ed25519_dalek::{SigningKey, VerifyingKey};`. Nothing
   in org-node signs any more.

6. `org-node/src/lib.rs`: `pub use envelope::SignedDeltaEnvelope;` becomes `pub use envelope::Envelope;`.

7. `org-node/src/transport/wire.rs`: `use crate::envelope::SignedDeltaEnvelope;`
   becomes `use crate::envelope::Envelope;`. The field becomes
   `pub envelope: Envelope,`. The doc comment's first line becomes
   `/// One message over the org-node channel: an Envelope carrying one delta,`.

8. The other tests:
   - `org-node/tests/wire_frame_bound.rs`: drop the `SigningKeypair` import
     and `let admin = …`. Import `use org_node::Envelope;` (not
     `SignedDeltaEnvelope`). `sample_msg` builds
     `let env = Envelope::build(OrgId::new([5u8; 20]), 2, &delta).unwrap();`.
   - `org-node/tests/transport_handshake.rs` and
     `org-node/tests/transport_networked.rs`: import `use org_node::Envelope;`.
     Delete `let admin = SigningKeypair::from_seed([1u8; 32]);` and its
     comment, and build `let env = Envelope::build(org, 2, &delta).unwrap();`.
     The verification context uses the sender the handshake authenticated:

     ```rust
         let accepted = [a_device.device_key().unwrap()];
         let ctx = VerifyContext {
             expected_org_id: org,
             // The sender is the Device key the QUIC handshake authenticated.
             sender: &remote_device,
             accepted_senders: &accepted,
             seq_guard: SeqGuard::from_last_seen(1),
             last_committed_epoch: 1,
         };
     ```
   - `org-node/tests/fuzz_envelope_decode/fuzz_target.rs`:
     `SignedDeltaEnvelope` → `Envelope` in the doc comment, the import and the
     `postcard::from_bytes::<Envelope>` call.
   - `org-node/tests/fuzz_verify_against_chain/fuzz_target.rs`: import
     `org_node::envelope::Envelope`. Delete
     `let admin = SigningKeypair::from_seed([1u8; 32]);` and
     `let vk = admin.verifying_key();`. Add, after `chain.set(…)`:

     ```rust
         // The fuzzed envelope arrives from the admin's device, which the
         // receiver accepts: every input reaches the decode.
         let sender = admin_device.device_key().unwrap();
         let accepted = [sender];
     ```
     and build the context as
     `VerifyContext { expected_org_id: org, sender: &sender, accepted_senders: &accepted, seq_guard: SeqGuard::from_last_seen(0), last_committed_epoch: 0 }`.
     In the unwind-safety comment, `None of local/chain/org/vk` becomes
     `None of local/chain/org/sender/accepted`.

9. `org-node/docs/CONTEXT.md`:
   - the **Envelope** definition becomes:
     "The carrier of one Change set between devices: the Organisation it is
     for, its Sequence number and the Change set bytes. It carries no
     signature: the connection authenticates the device that sends it."
   - in **Wire message**, the sentence
     "Wider than the Envelope, because neither the secret nor the snapshot is covered by the Envelope's signature."
     becomes "Wider than the Envelope, because neither the secret nor the snapshot is part of it."

10. Run:

    ```bash
    S=/private/tmp/claude-501/-Users-jan-jan-Coding-2-tier-access-control/040b9a0d-b135-4b8c-ab0a-f48aa8a3106c/scratchpad
    CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features test-support --lib --test verify_against_chain --test wire_frame_bound --test fuzz_envelope_decode --test fuzz_verify_against_chain --test transport_handshake --test transport_networked > $S/plan4-t4-core.log 2>&1; echo "exit $?" >> $S/plan4-t4-core.log
    grep -E "^test result|^exit|iterations" $S/plan4-t4-core.log
    ```

    Expected: every `test result: ok`, verify_against_chain 14 passed (was
    13, plus `an_empty_accepted_set_accepts_no_sender`; the two signature
    tests are rewritten in place), the two bolero targets exit 0, `exit 0`. Do not run
    ORG-NODE-GATE here. `service.rs` still calls `SignedDeltaEnvelope` and
    `author_member_key` and does not compile under `app` until T5.

11. Commit:

    ```bash
    git add org-node/src/envelope.rs org-node/src/verify.rs org-node/src/error.rs org-node/src/keys.rs org-node/src/lib.rs org-node/src/transport/wire.rs org-node/docs/CONTEXT.md org-node/tests/verify_against_chain.rs org-node/tests/wire_frame_bound.rs org-node/tests/transport_handshake.rs org-node/tests/transport_networked.rs org-node/tests/fuzz_envelope_decode/fuzz_target.rs org-node/tests/fuzz_verify_against_chain/fuzz_target.rs
    ```
    ```bash
    git -c commit.gpgsign=false commit -m "feat(org-node): Envelope signature dropped; verification checks the authenticated sender against the accepted devices before decoding (REQ-7h7qp3, REQ-txvtm9)"
    ```

**Status: done (2026-10-05).**

- red -> green: every test of the rewritten
  `org-node/tests/verify_against_chain.rs` — among them
  `rejects_a_sender_outside_the_accepted_devices` (REQ-ag6kqm, REQ-7h7qp3),
  `rejects_unknown_sender_before_decoding_delta` (REQ-ag6kqm, REQ-7h7qp3),
  `an_empty_accepted_set_accepts_no_sender` (REQ-7h7qp3, new),
  `happy_path_commits_when_root_matches_chain` (REQ-nhe2zu, REQ-txvtm9) and
  `rejects_root_mismatch_when_chain_root_differs` (REQ-wp2nyc, REQ-nhe2zu,
  REQ-txvtm9, REQ-mr5abb): watched fail before step 2, the target refusing
  to compile with six errors (`E0432 unresolved import org_node::Envelope`,
  `E0560 no field named sender`, `E0560 no field named accepted_senders`,
  three `E0599 no variant … UnknownSender`; `$S/step4-t4-red.log`). All 14
  pass after steps 2–8.
- The four `envelope::tests` (lib, no annotation) were written with the
  module in step 2 and not run red on their own; they name `Envelope` and
  the two-argument `build`, neither of which existed before.
- Core run (`$S/step4-t4-core.log`): `exit 0`. lib 14 passed (under
  `test-support` without `app`); `verify_against_chain` 14;
  `wire_frame_bound` 3; `transport_handshake` 3; `transport_networked` 1;
  both bolero targets ran their default second without a failure.
- ORG-NODE-GATE and APP-GATE were not run, as step 10 says: `service.rs`
  still names `SignedDeltaEnvelope`, `author_member_key` and `BadSignature`,
  and `service_stories.rs` and `admission_sender.rs` still match
  `BadSignature`. T5 fixes them.

---

### T5 — The sender-device rule on both Receive operations; the admin persona without `org_pub_key`

**Files touched:** `org-node/src/service.rs`, `org-node/tests/admission_sender.rs`, `org-node/tests/service_stories.rs`
**Parallel:** no (serial, after T4)
**verifies:** `REQ-xa6smf, REQ-7h7qp3`; `REQ-7h7qp3` (new, no Invite); `REQ-7h7qp3, REQ-mr5abb` (re-annotated); `REQ-ztdza4, REQ-mr5abb` (new); `REQ-nhe2zu, REQ-txvtm9, REQ-xa6smf, REQ-uxv2x2`; `REQ-uxv2x2, REQ-7h7qp3` (rewritten)

The service passes `verify_envelope_against_chain` the sender `recv_one`
authenticated and the accepted set. For a record the node holds, that is
every Device key in the record. On a first admission it is the administrator's
Device key from the imported Invite, and with no Invite the operation is
refused. Both Receive operations now do this. On the removal path the
authenticated sender used to be discarded (`let _ = remote_device_key;`).
REQ-ztdza4's check after verification, against the record the message was
verified into, stays as it is: it is not superseded, and it catches a sender
the Change set itself removes. `admin_member_key` on a first admission comes
from the Invite. The node's own persona is found by excluding the persona
whose member key is `admin_member_key`, not by comparing with `org_pub_key`.
The admin no longer signs. On a first admission the sender is checked against
the Invite's administrator Device key before `first_admission_base` decodes the
snapshot, as well as before the Change set (decision 5, as overturned).
`first_admission_without_a_record_snapshot_is_refused` (REQ-d9g6nt) therefore
sends its stripped message from A's own endpoint.

1. Failing tests first, in `org-node/tests/admission_sender.rs`:
   - Replace the module doc comment's second paragraph and bullets with:

     ```rust
     //! The admin (A) always produces a chain-valid admission envelope. The
     //! Envelope carries no signature; what varies is *who delivers it* to the
     //! member (B), as the QUIC connection authenticates it:
     //!
     //!   * first admission — B has imported A's invite, so the sender must
     //!     equal the invite's `admin_device_key`; a rogue endpoint R relaying
     //!     the identical message is rejected before the Change set is decoded
     //!     (REQ-xa6smf, REQ-7h7qp3), and without an imported invite no sender
     //!     is accepted (REQ-7h7qp3);
     //!   * after admission — the sender must be a Device key in B's current
     //!     record before decoding (REQ-7h7qp3), and in the record the message
     //!     was verified into after verification (REQ-ztdza4).
     ```
   - Split `setup` so a test can skip the invite import:

     ```rust
     async fn setup(tag: &str) -> Setup {
         setup_with(tag, true).await
     }

     async fn setup_with(tag: &str, import_invite: bool) -> Setup {
     ```
     The body is today's `setup`, except that these three lines

     ```rust
         let invite_blob = svc_a.export_invite(org_id).unwrap();
         let invite = svc_b.import_invite(&mut OsRng, &invite_blob).unwrap();
         assert_eq!(invite.org_id, org_id);
     ```
     sit inside `if import_invite { … }`.
   - In `admit_b_directly`, after `assert_eq!(svc_b.list_orgs()[0].trie_members.len(), 2, "admin + B");` add:

     ```rust
         assert_eq!(
             svc_b.list_orgs()[0].admin_member_key,
             s.svc_a.list_orgs()[0].admin_member_key,
             "B's record names the administrator the invite named, not a chain key"
         );
     ```
   - `first_admission_from_a_device_other_than_the_invites_admin_is_rejected`:
     the annotation becomes `// verifies: REQ-xa6smf, REQ-7h7qp3`. The
     comment's "chain-valid and correctly signed by the admin" becomes
     "chain-valid". The assertion becomes
     `matches!(result, Err(OrgNodeError::UnknownSender))`.
   - `update_relayed_by_a_non_member_after_admission_is_rejected`: replace
     its comment block and annotation with

     ```rust
     // Abnormal case of REQ-7h7qp3 after admission: the same chain-valid update,
     // relayed by a device that is in no member's slots of B's current record,
     // is rejected before its Change set is decoded, and B's record stays at
     // epoch 2. REQ-mr5abb's "never on a rejection at any step", over the whole
     // receive operation: the persisted high-water mark (`OrgRecord.last_seq`)
     // must be exactly where it was. (Until 2026-10-05 this rejection came after
     // verification, under REQ-ztdza4; `update_relayed_by_the_device_it_removes_is_rejected`
     // now exercises that later check.)
     // verifies: REQ-7h7qp3, REQ-mr5abb
     ```
     and its assertion becomes `matches!(result, Err(OrgNodeError::UnknownSender))`.
   - Add these two tests after it:

     ```rust
     // Abnormal case of REQ-7h7qp3's first-admission clause: B never imported an
     // Invite for the Organisation, so on its first admission no Device key is
     // accepted, not even the genuine administrator's delivering a chain-valid
     // admission. B rejects it and commits nothing.
     // verifies: REQ-7h7qp3
     #[tokio::test(flavor = "multi_thread")]
     async fn first_admission_without_an_imported_invite_is_rejected() {
         let mut s = setup_with("no-invite", false).await;

         let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
         tokio::time::timeout(
             NET,
             s.svc_a.admit_member(&mut OsRng, s.org_id, &s.join_request_b, b_addr, ORG_SECRET),
         )
         .await
         .expect("admit_member(B) timed out")
         .expect("admit_member(B) failed");
         assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, 2);

         let (svc_b, result) = b_task.await.unwrap();
         assert!(
             matches!(result, Err(OrgNodeError::UnknownSender)),
             "B must reject a first admission for which it imported no invite, got {result:?}"
         );
         assert!(svc_b.list_orgs().is_empty(), "B must not have committed an OrgRecord");
         let persona_b = svc_b.list_personas().iter().find(|p| p.persona_id == s.pid_b).unwrap();
         assert_ne!(persona_b.status, PersonaStatus::Active, "B's persona must not be Active");
         assert_eq!(persona_b.org_id, None);
     }

     // REQ-ztdza4: C's device is in B's current record, so the revocation of C
     // that C's own device relays passes the sender check before decoding
     // (REQ-7h7qp3) and verifies against the chain. But the record it was
     // verified into no longer holds C's device, so B rejects it after
     // verification and before touching its record. This is the deepest
     // rejection the Receive operation has: every verification check passed.
     // So it is also REQ-mr5abb's case over the whole operation: the persisted
     // high-water mark must not move.
     // verifies: REQ-ztdza4, REQ-mr5abb
     #[tokio::test(flavor = "multi_thread")]
     async fn update_relayed_by_the_device_it_removes_is_rejected() {
         let mut s = admit_b_directly(setup("update-removed").await).await;

         // A creates C in its own store and admits C, pushing the update to B.
         let pid_c = s.svc_a.create_persona(&mut OsRng, "carol", "Carol", "Coder").unwrap();
         let c_device_kp = device_kp(&s.svc_a, &pid_c);
         let jr_c = OrgService::import_join_request(&s.svc_a.export_join_request(&pid_c).unwrap())
             .unwrap();
         let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
         let c_member_id = tokio::time::timeout(
             NET,
             s.svc_a.admit_member(&mut OsRng, s.org_id, &jr_c, b_addr, ORG_SECRET),
         )
         .await
         .expect("admit_member(C) timed out")
         .expect("admit_member(C) failed");
         let (svc_b, result) = b_task.await.unwrap();
         assert_eq!(result.expect("B must accept C's admission from the admin").epoch, 3);
         s.svc_b = svc_b;
         let root_at_3 = s.svc_b.list_orgs()[0].root_hash;
         let last_seq_at_3 = s.svc_b.list_orgs()[0].last_seq;

         // A revokes C; a sink captures the genuine revocation (epoch 4).
         let (sink_addr, sink_task) = spawn_recv_one(rand::random()).await;
         tokio::time::timeout(
             NET,
             s.svc_a.revoke_member(&mut OsRng, s.org_id, c_member_id, Some(sink_addr)),
         )
         .await
         .expect("revoke_member(C) timed out")
         .expect("revoke_member(C) failed");
         assert_eq!(s.chain.get(&s.org_id).unwrap().epoch, 4);
         let (_sink_ep, _sender, msg) = sink_task.await.unwrap();

         // C's own device relays it to B.
         let ep_c = OrgEndpoint::bind(&c_device_kp).await.unwrap();
         let (b_addr, b_task) = spawn_b_receive(s.svc_b, &s.b_device_kp).await;
         tokio::time::timeout(NET, ep_c.send(b_addr, &msg))
             .await
             .expect("C relay send timed out")
             .expect("C relay send failed");

         let (svc_b, result) = b_task.await.unwrap();
         assert!(
             matches!(result, Err(OrgNodeError::UnknownSender)),
             "B must reject an update whose sender the update itself removes, got {result:?}"
         );
         let rec = &svc_b.list_orgs()[0];
         assert_eq!(rec.epoch, 3, "B's record must still be at epoch 3");
         assert_eq!(rec.root_hash, root_at_3, "B's root must be unchanged");
         assert_eq!(rec.last_seq, last_seq_at_3, "the high-water mark must not advance");
         assert_eq!(rec.trie_members.len(), 3, "admin + B + C");
     }
     ```

2. Failing tests first, in `org-node/tests/service_stories.rs`:
   - `five_stories_full_e2e`: the comment's
     "commits the applied Change set with the chain's epoch and root (REQ-nhe2zu)"
     becomes "commits the applied Change set with the chain's epoch and root (REQ-nhe2zu, and REQ-txvtm9 which supersedes it)".
     The annotation becomes
     `// verifies: REQ-nhe2zu, REQ-txvtm9, REQ-xa6smf, REQ-uxv2x2`.
   - Rename `unverified_revocation_leaves_the_record_in_place` to
     `revocation_from_an_unknown_device_leaves_the_record_in_place` and
     replace its leading comment and annotation with

     ```rust
     // Abnormal-input case of the self-delete rule, on the Receive operation that
     // acts on the node's own removal: a revocation Change set that removes this
     // node's own Device key, well formed, naming the right Organisation and the
     // next Sequence number, but delivered by a device in no member's slots of
     // the node's current record, must be rejected before it is decoded
     // (REQ-7h7qp3 covers this path; RC-b6mydy did not) — and the rejection must
     // leave the OrgRecord exactly as it was. The node must never delete its
     // record of the Organisation on a message it refused.
     // verifies: REQ-uxv2x2, REQ-7h7qp3
     ```
   - In its body, `use org_node::envelope::SignedDeltaEnvelope;` becomes
     `use org_node::envelope::Envelope;`. The `let (epoch_before, …, org_pub_key) = { … }`
     binding drops `org_pub_key` (and `rec.org_pub_key`). Replace everything
     from `// Signed by a keypair that is not the Organisation's published signing key.`
     down to and including the `let msg = WireMessage { … };` line with:

     ```rust
         // No signature to forge any more: only the sender gives it away. The
         // forging device is in no member's slots of B's record.
         let forger_device = SigningKeypair::from_seed([0x77u8; 32]);
         assert!(
             !svc_b.list_orgs()[0]
                 .trie_members
                 .iter()
                 .any(|m| m.device_keys.contains(forger_device.verifying_key().as_bytes())),
             "the forging device must not be in B's record"
         );
         let envelope = Envelope::build(org_id, seq_before + 1, &delta).unwrap();
         let msg = WireMessage { envelope, org_secret: None, genesis_snapshot: None };
     ```
     Further down, delete the now-duplicate
     `let forger_device = SigningKeypair::from_seed([0x77u8; 32]);` line
     before `let ep_forger = …`. Replace the error assertion with

     ```rust
         let err = r.expect_err("a revocation from an unknown device must be rejected");
         assert!(
             matches!(err, OrgNodeError::UnknownSender),
             "expected UnknownSender (the sender check, before the stale-epoch check \
              this forged message would also fail), got {err:?}"
         );
     ```
   - `revocation_of_another_member_is_committed_not_self_deleted`: no change.

   Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test admission_sender --test service_stories`.
   Expected: does not compile (service.rs).

3. `org-node/src/service.rs`:
   - Imports: delete `use ed25519_dalek::VerifyingKey;`. Change
     `use crate::envelope::SignedDeltaEnvelope;` to `use crate::envelope::Envelope;`.
   - Add these helpers after `encode_record_snapshot`:

     ```rust
     /// Every Device key in a record: the senders REQ-7h7qp3 accepts for an
     /// Organisation the node already holds a record of.
     fn record_devices(trie: &Trie) -> Vec<org_members::DevicePublicKey> {
         trie.members().iter().flat_map(|m| m.p2p_devices().to_vec()).collect()
     }

     /// The persisted form of one member of a committed record.
     fn snapshot_of(m: &MemberLeaf) -> MemberSnapshot {
         MemberSnapshot {
             id: *m.id().as_bytes(),
             handle: m.handle().to_string(),
             name: m.name().to_string(),
             surname: m.surname().to_string(),
             member_key: *m.p2p_key().as_bytes(),
             device_keys: m.p2p_devices().iter().map(|d| *d.as_bytes()).collect(),
         }
     }
     ```
   - `admit_member`: the doc comment's "build a `SignedDeltaEnvelope`" becomes
     "build an `Envelope`". The opening block becomes

     ```rust
             let (trie, org_epoch, org_pub_key, last_seq, pre_add_snapshots, proxy_account, admin_persona_id) = {
                 let org_rec = self.find_org(org_id)?;
                 let trie = trie_from_snapshots(&org_rec.trie_members)?;
                 // Capture pre-add snapshots so B can reconstruct the genesis trie.
                 let snapshots = org_rec.trie_members.clone();
                 // The admin persona's device is the endpoint the push leaves from;
                 // found before the chain write so a missing one changes nothing.
                 let admin_persona_id = self.admin_persona_for_org(org_id)?.persona_id.clone();
                 (trie, org_rec.epoch, org_rec.org_pub_key, org_rec.last_seq, snapshots, org_rec.proxy_account, admin_persona_id)
             };
     ```
     The envelope is built as

     ```rust
             // The Envelope carries no signature: the receiver authenticates the
             // admin's device through the connection (REQ-7h7qp3).
             let parent_seq = last_seq + 1;
             let envelope = Envelope::build(org_id, parent_seq, &delta)?;
     ```
     Delete the later line
     `let admin_persona_id = self.admin_persona_for_org(org_id)?.persona_id.clone();`.
   - `revoke_member`: the opening block becomes

     ```rust
             let (trie, org_epoch, org_pub_key, last_seq, proxy_account, admin_persona_id) = {
                 let org_rec = self.find_org(org_id)?;
                 let trie = trie_from_snapshots(&org_rec.trie_members)?;
                 let admin_persona_id = self.admin_persona_for_org(org_id)?.persona_id.clone();
                 (trie, org_rec.epoch, org_rec.org_pub_key, org_rec.last_seq, org_rec.proxy_account, admin_persona_id)
             };
     ```
     The envelope is `let envelope = Envelope::build(org_id, parent_seq, &delta)?;`
     with the comment `// The revocation Envelope; the receiver authenticates this device.`.
     The later block binds only `networked_peer_id`. Its first line inside
     becomes `let org_rec = self.find_org(org_id)?;`. Delete
     `let admin_persona_id = …` there, and it ends with `networked_peer_id`
     instead of the tuple:
     `let networked_peer_id = { … networked_peer_id };`. The `new_snaps` map
     becomes `new_trie.members().iter().map(snapshot_of).collect()`.
   - Replace the whole body of `receive_and_verify` after the `recv_one` call
     (from `let org_id = msg.envelope.org_id;` through the final `Ok(ReceiveOutcome { … })`) with:

     ```rust
             let org_id = msg.envelope.org_id;

             // The chain's Organisation state: what the Change set must reach
             // (RC-6a2dke, RC-e5atck).
             let chain_state = self
                 .chain
                 .read_state(org_id)
                 .await?
                 .ok_or(OrgNodeError::OrgNotOnChain)?;

             // The record this message extends and the senders REQ-7h7qp3 accepts
             // for it: for an Organisation the node holds a record of, every Device
             // key in that record; on a first admission, only the administrator's
             // Device key named by the Invite it imported, and no sender at all
             // without one.
             let existing = self.store.data().orgs.iter().find(|o| o.org_id == org_id).cloned();
             let (local_trie, last_seq, last_epoch, accepted_senders, admin_member_key, is_first_admission) =
                 match existing {
                     Some(existing) => {
                         let trie = trie_from_snapshots(&existing.trie_members)?;
                         let senders = record_devices(&trie);
                         (trie, existing.last_seq, existing.epoch, senders, existing.admin_member_key, false)
                     }
                     None => {
                         // The record a first admission extends (REQ-d9g6nt).
                         let trie = first_admission_base(msg.genesis_snapshot.as_deref())?;
                         let invite = self
                             .store
                             .data()
                             .pending_invites
                             .iter()
                             .find(|p| p.org_id == org_id)
                             .cloned()
                             .ok_or(OrgNodeError::UnknownSender)?;
                         let admin_device = org_members::DevicePublicKey::parse(&invite.admin_device_key)
                             .map_err(OrgMembersError::from)?;
                         (trie, 0, 0, vec![admin_device], invite.admin_member_key, true)
                     }
                 };

             let ctx = VerifyContext {
                 expected_org_id: org_id,
                 sender: &remote_device_key,
                 accepted_senders: &accepted_senders,
                 seq_guard: SeqGuard::from_last_seen(last_seq),
                 last_committed_epoch: last_epoch,
             };
             let chain_reader = ChainOpsReader { state: chain_state };
             let verified = verify_envelope_against_chain(&local_trie, &msg.envelope, &ctx, &chain_reader)?;
             let members = verified.trie.members();

             // REQ-ztdza4: an update to a record the node already holds must also
             // come from a Device key in the record it was verified into.
             if !is_first_admission && !members.iter().any(|m| m.has_p2p_device(&remote_device_key)) {
                 return Err(OrgNodeError::UnknownSender);
             }

             let new_snapshots: Vec<MemberSnapshot> = members.iter().map(snapshot_of).collect();
             let new_root = *verified.trie.root_hash().map_err(OrgNodeError::Trie)?.as_bytes();

             // This node's persona: one whose Device key is in the new record and
             // which is not the administrator the record names. The administrator
             // is `admin_member_key` (from the Invite on a first admission), never
             // the chain's Organisation public key.
             let my_member = self.store.data().personas.iter().find_map(|p| {
                 if X25519Keypair::from_seed(p.member_seed).public_bytes() == admin_member_key {
                     return None;
                 }
                 let dk = SigningKeypair::from_seed(p.device_seed).device_key().ok()?;
                 members
                     .iter()
                     .find(|m| m.has_p2p_device(&dk))
                     .map(|m| (p.persona_id.clone(), *m.id().as_bytes()))
             });

             {
                 let data = self.store.data_mut();
                 if let Some(existing) = data.orgs.iter_mut().find(|o| o.org_id == org_id) {
                     existing.root_hash = new_root;
                     existing.epoch = verified.epoch;
                     existing.last_seq = verified.seq_guard.last_seen();
                     existing.org_secret = msg.org_secret;
                     existing.trie_members = new_snapshots;
                 } else {
                     data.orgs.push(OrgRecord {
                         org_id,
                         root_hash: new_root,
                         org_pub_key: chain_state.org_pub_key,
                         epoch: verified.epoch,
                         org_secret: msg.org_secret,
                         last_seq: verified.seq_guard.last_seen(),
                         admin_member_key,
                         trie_members: new_snapshots,
                         // Member-side record: P is only known by the admin who created the org.
                         proxy_account: None,
                     });
                 }
                 // Consume the pending invite now that first admission has committed.
                 if is_first_admission {
                     data.pending_invites.retain(|p| p.org_id != org_id);
                 }
             }

             // Mark persona as Active + set member_id.
             if let Some((pid, member_id)) = my_member {
                 let personas = &mut self.store.data_mut().personas;
                 if let Some(p) = personas.iter_mut().find(|p| p.persona_id == pid) {
                     p.status = PersonaStatus::Active;
                     p.org_id = Some(org_id);
                     p.member_id = Some(member_id);
                 }
             }

             self.store.save(rng)?;

             Ok(ReceiveOutcome { org_id, epoch: verified.epoch, root: new_root })
     ```
     Replace the method's doc comment with:

     ```rust
         /// Accept one inbound `WireMessage`, check its sender against the
         /// devices this node accepts before the Change set is decoded
         /// (REQ-7h7qp3), verify the envelope against the chain, check the sender
         /// against the verified record (REQ-ztdza4), and commit the new state.
     ```
   - In `receive_and_self_delete_if_revoked`, replace everything from
     `let author_vk = VerifyingKey::from_bytes(&chain_state.org_pub_key)`
     through `let _ = remote_device_key; // authenticated but not cross-checked here (revocation path)`
     with:

     ```rust
             let (local_trie, last_seq, last_epoch) = {
                 let existing = self
                     .store
                     .data()
                     .orgs
                     .iter()
                     .find(|o| o.org_id == org_id)
                     .cloned()
                     .ok_or(OrgNodeError::OrgNotOnChain)?;
                 let trie = trie_from_snapshots(&existing.trie_members)?;
                 (trie, existing.last_seq, existing.epoch)
             };

             // REQ-7h7qp3 on the removal path: the sender must be a Device key in
             // this node's own current record, checked before the Change set is
             // decoded; the chain root still decides whether the removal is real.
             let accepted_senders = record_devices(&local_trie);
             let ctx = VerifyContext {
                 expected_org_id: org_id,
                 sender: &remote_device_key,
                 accepted_senders: &accepted_senders,
                 seq_guard: SeqGuard::from_last_seen(last_seq),
                 last_committed_epoch: last_epoch,
             };

             let chain_reader = ChainOpsReader { state: chain_state };
             let verified =
                 verify_envelope_against_chain(&local_trie, &msg.envelope, &ctx, &chain_reader)?;
             let members = verified.trie.members();

             // Check whether OUR device is still in the new trie.
             let my_still_present = self
                 .store
                 .data()
                 .personas
                 .iter()
                 .filter(|p| p.org_id == Some(org_id))
                 .any(|p| {
                     SigningKeypair::from_seed(p.device_seed)
                         .device_key()
                         .is_ok_and(|my_dk| members.iter().any(|m| m.has_p2p_device(&my_dk)))
                 });
     ```
     and in the `if my_still_present` branch the `new_snaps` map becomes
     `members.iter().map(snapshot_of).collect()`.

4. Run ORG-NODE-GATE into `$S/plan4-t5-org-node.log`. Expected: every target
   `test result: ok`, `exit 0`. admission_sender: 9 passed, the 7 it had
   plus `first_admission_without_an_imported_invite_is_rejected` and
   `update_relayed_by_the_device_it_removes_is_rejected`. Also
   `grep -n "VerifyingKey\|SignedDeltaEnvelope\|BadSignature\|author_member_key" org-node/src org-node/tests -r`
   must print only `org-node/tests/chain_genesis_e2e.rs` lines (T7 fixes
   that file) and `SigningKeypair::verifying_key` uses.

5. Commit:

   ```bash
   git add org-node/src/service.rs org-node/tests/admission_sender.rs org-node/tests/service_stories.rs
   ```
   ```bash
   git -c commit.gpgsign=false commit -m "feat(org-node): both Receive operations accept only known sender devices before decoding; admin_member_key from the Invite; no signing (REQ-7h7qp3, REQ-txvtm9)"
   ```


**Status: done (2026-10-05).**

- Added beside the plan, for decision 5 as overturned:
  `first_admission_from_a_wrong_sender_is_rejected_before_its_snapshot_is_decoded`
  (REQ-7h7qp3) in `admission_sender.rs`: R sends B a first admission whose
  snapshot is 64 bytes of `0xff`; B must answer `UnknownSender`. In
  `receive_and_verify`'s first-admission branch the Invite's administrator
  Device key is compared with the sender before `first_admission_base` runs.
  `first_admission_without_a_record_snapshot_is_refused` (REQ-d9g6nt) now
  sends the stripped message from A's own endpoint (`svc_a.endpoint()`), and
  asserts what it asserted before.
- red -> green: every test in the edited `admission_sender.rs` and
  `service_stories.rs` (among them
  `first_admission_without_an_imported_invite_is_rejected` (REQ-7h7qp3),
  `update_relayed_by_the_device_it_removes_is_rejected` (REQ-ztdza4,
  REQ-mr5abb), `first_admission_from_a_device_other_than_the_invites_admin_is_rejected`
  (REQ-xa6smf, REQ-7h7qp3), `update_relayed_by_a_non_member_after_admission_is_rejected`
  (REQ-7h7qp3, REQ-mr5abb), `revocation_from_an_unknown_device_leaves_the_record_in_place`
  (REQ-uxv2x2, REQ-7h7qp3) and `five_stories_full_e2e`): watched fail before
  step 3, the targets refusing to compile with five errors in `service.rs`
  (`E0432 unresolved import crate::envelope::SignedDeltaEnvelope`, two
  `E0599 no variant … BadSignature`, two `E0560 no field named author_member_key`;
  `$S/step4-t5-red.log`).
- red -> green: `first_admission_from_a_wrong_sender_is_rejected_before_its_snapshot_is_decoded`
  (REQ-7h7qp3): watched fail with step 3 applied as the plan first wrote it
  (snapshot decoded before the sender check):
  `got Err(Chain("genesis_snapshot decode: Found a varint that didn't terminate. …"))`
  (`$S/step4-t5-order-red.log`; the other 12 tests of the two targets passed).
  Passes once the sender check precedes `first_admission_base`.
- ORG-NODE-GATE (`$S/step4-t5-org-node.log`): `exit 0`. lib 24,
  admission_sender 10 (the 7 it had, plus the plan's two and the order test),
  service_stories 3, store_at_rest 4, transport_handshake 3,
  transport_networked 1, verify_against_chain 14, wire_frame_bound 3; the
  three bolero targets ran without a failure. No compiler warnings from
  org-node.
- The grep of step 4 prints, besides `chain_genesis_e2e.rs` (T7) and
  `SigningKeypair::verifying_key` in `keys.rs`, the transport's own
  `ed25519_dalek::VerifyingKey::from_bytes` in `transport/endpoint.rs:301`,
  which parses the connection's peer id and is not the Envelope signature.
- APP-GATE not run: the app still matches `BadSignature` until T8.
---

### T6 — A fresh Organisation key at creation (REQ-ech45n)

**Files touched:** `org-node/Cargo.toml`, `org-node/.guardrails/config.yaml`, `org-node/src/store.rs`, `org-node/src/service.rs`, `org-node/tests/store_at_rest.rs`, `org-node/tests/organisation_key.rs` (new)
**Parallel:** no (serial, after T5)
**verifies:** `REQ-ech45n` (four new tests)

1. `org-node/Cargo.toml`, after the `admission_sender` `[[test]]`:

   ```toml
   # The Organisation key: a fresh X25519 secret per created Organisation
   # (REQ-ech45n) and the validity of the key read from the chain (REQ-8jb4ny).
   [[test]]
   name = "organisation_key"
   path = "tests/organisation_key.rs"
   required-features = ["app", "test-support"]
   ```

   `org-node/.guardrails/config.yaml`: append ` --test organisation_key` to
   the end of the cargo `verify_commands` line. Edit nothing else in that
   file in this task.

2. Failing tests: create `org-node/tests/organisation_key.rs`:

   ```rust
   #![cfg(all(feature = "app", feature = "test-support"))]
   #![allow(clippy::unwrap_used, clippy::expect_used)]
   //! The Organisation key. A created Organisation publishes the X25519 public
   //! key of a secret generated for it alone, distinct from every key of its
   //! genesis record, and that secret is kept only in the creator's encrypted
   //! store (REQ-ech45n).
   //!
   //! The gate: `cargo test -p org-node --features app,test-support --test organisation_key`

   use std::path::PathBuf;

   use org_members::{OrgMembersError, PersonPublicKey};
   use org_node::error::OrgNodeError;
   use org_node::ids::OrgId;
   use org_node::keys::X25519Keypair;
   use org_node::service::{MockChainOps, OrgService};
   use org_node::store::{OrgRecord, PersonaStatus, PersonaStore};
   use rand::rngs::OsRng;
   use rand::{CryptoRng, RngCore};

   const PASSWORD: &str = "pw_org_key";

   /// Fresh encrypted store under `temp_dir()`, unique per test and party.
   fn open_store(tag: &str, party: &str) -> (PersonaStore, PathBuf) {
       let dir = std::env::temp_dir().join(format!(
           "ods-organisation-key-{tag}-{party}-{}",
           std::process::id()
       ));
       let _ = std::fs::remove_dir_all(&dir);
       std::fs::create_dir_all(&dir).unwrap();
       let path = dir.join("store.bin");
       (PersonaStore::open(path.clone(), PASSWORD).unwrap(), path)
   }

   /// The Organisation public key the chain holds for `org_id`, as bytes.
   fn published_key(chain: &MockChainOps, org_id: &OrgId) -> [u8; 32] {
       chain.get(org_id).unwrap().org_pub_key
   }

   /// The Organisation public key a record holds, as bytes.
   fn record_key(rec: &OrgRecord) -> [u8; 32] {
       rec.org_pub_key
   }

   fn contains(hay: &[u8], needle: &[u8]) -> bool {
       hay.windows(needle.len()).any(|w| w == needle)
   }

   /// A random source that returns the same byte every time: makes every
   /// secret a node draws identical, so the Organisation secret equals the
   /// persona's member seed.
   struct ConstRng(u8);

   impl RngCore for ConstRng {
       fn next_u32(&mut self) -> u32 {
           u32::from_le_bytes([self.0; 4])
       }
       fn next_u64(&mut self) -> u64 {
           u64::from_le_bytes([self.0; 8])
       }
       fn fill_bytes(&mut self, dest: &mut [u8]) {
           dest.fill(self.0);
       }
       fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
           dest.fill(self.0);
           Ok(())
       }
   }

   impl CryptoRng for ConstRng {}

   // Normal case: the published key is the X25519 public key of the secret the
   // creator holds, and no member or device of the genesis record holds it.
   // verifies: REQ-ech45n
   #[tokio::test(flavor = "multi_thread")]
   async fn a_created_organisation_publishes_a_fresh_key_no_genesis_key_equals() {
       let chain = MockChainOps::new();
       let (store, _) = open_store("fresh", "a");
       let mut svc = OrgService::new(store, Box::new(chain.clone()));
       let pid = svc.create_persona(&mut OsRng, "admin", "Admin", "User").unwrap();
       let org_id = svc.create_organisation(&mut OsRng, &pid).await.unwrap();

       let published = published_key(&chain, &org_id);
       let rec = svc.list_orgs()[0].clone();
       assert_eq!(record_key(&rec), published, "the record holds the key it published");
       assert!(PersonPublicKey::parse(&published).is_ok(), "a valid X25519 public key");

       let secret = rec.org_private_key.expect("the creator holds the Organisation secret");
       assert_eq!(
           X25519Keypair::from_seed(secret).public_bytes(),
           published,
           "the published key is the public key of the secret held"
       );
       for m in &rec.trie_members {
           assert_ne!(published, m.member_key, "a Member-as-a-group key of the genesis record");
           for d in &m.device_keys {
               assert_ne!(published, *d, "a Device key of the genesis record");
           }
       }
       let persona = &svc.list_personas()[0];
       assert_ne!(secret, persona.member_seed, "the Organisation secret is the member seed");
       assert_ne!(secret, persona.device_seed, "the Organisation secret is the device seed");
   }

   // Freshness: one persona founding two Organisations publishes two keys. A
   // key derived from the persona's own secrets would be the same twice.
   // verifies: REQ-ech45n
   #[tokio::test(flavor = "multi_thread")]
   async fn two_organisations_of_one_persona_publish_two_keys() {
       let chain = MockChainOps::new();
       let (store, _) = open_store("two-orgs", "a");
       let mut svc = OrgService::new(store, Box::new(chain.clone()));
       let pid = svc.create_persona(&mut OsRng, "admin", "Admin", "User").unwrap();
       let org_1 = svc.create_organisation(&mut OsRng, &pid).await.unwrap();
       let org_2 = svc.create_organisation(&mut OsRng, &pid).await.unwrap();
       assert_ne!(published_key(&chain, &org_1), published_key(&chain, &org_2));
   }

   // The secret is in the store, under the passphrase, and never in the
   // store file in clear.
   // verifies: REQ-ech45n
   #[tokio::test(flavor = "multi_thread")]
   async fn the_organisation_secret_is_kept_only_in_the_encrypted_store() {
       let chain = MockChainOps::new();
       let (store, path) = open_store("at-rest", "a");
       let mut svc = OrgService::new(store, Box::new(chain.clone()));
       let pid = svc.create_persona(&mut OsRng, "admin", "Admin", "User").unwrap();
       let org_id = svc.create_organisation(&mut OsRng, &pid).await.unwrap();
       let secret = svc.list_orgs()[0].org_private_key.unwrap();

       let reopened = PersonaStore::open(path.clone(), PASSWORD).unwrap();
       let rec = reopened.data().orgs.iter().find(|o| o.org_id == org_id).unwrap();
       assert_eq!(rec.org_private_key, Some(secret), "the store holds the secret");
       assert!(PersonaStore::open(path.clone(), "wrong").is_err(), "only under the passphrase");

       let bytes = std::fs::read(&path).unwrap();
       assert!(!contains(&bytes, &secret), "the Organisation secret is in the file in clear");
   }

   // Abnormal case: a random source that repeats itself makes the Organisation
   // secret equal to the member seed, so the published key would be the
   // admin's Member-as-a-group key. Creation is refused before the chain is
   // written, and nothing is recorded.
   // verifies: REQ-ech45n
   #[tokio::test(flavor = "multi_thread")]
   async fn an_organisation_key_equal_to_a_genesis_key_is_refused() {
       let chain = MockChainOps::new();
       let (store, _) = open_store("collision", "a");
       let mut svc = OrgService::new(store, Box::new(chain.clone()));
       let mut rng = ConstRng(0x5c);
       let pid = svc.create_persona(&mut rng, "admin", "Admin", "User").unwrap();
       let err = svc.create_organisation(&mut rng, &pid).await.unwrap_err();
       assert_eq!(err, OrgNodeError::Trie(OrgMembersError::DuplicateKey));
       assert!(svc.list_orgs().is_empty(), "no Organisation recorded");
       assert_eq!(svc.list_personas()[0].status, PersonaStatus::Proposed);
   }
   ```

   Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test organisation_key`.
   Expected: does not compile (`no field org_private_key on OrgRecord`).

3. `org-node/src/store.rs`, at the end of `OrgRecord` after `proxy_account`:

   ```rust
       /// The Organisation's X25519 secret (REQ-ech45n): generated by the node
       /// that created the Organisation and held only here, in its encrypted
       /// store. `None` on every other node's record.
       #[serde(default)]
       pub org_private_key: Option<[u8; 32]>,
   ```

4. `org-node/src/service.rs`:
   - `create_organisation` becomes:

     ```rust
         pub async fn create_organisation<R: RngCore + CryptoRng>(
             &mut self,
             rng: &mut R,
             persona_id: &str,
         ) -> Result<OrgId, OrgNodeError> {
             let (member_kp, device_kp, handle, name, surname) = self.persona_keys(persona_id)?;
             let member_key = member_kp.member_key()?;
             let device_key = device_kp.device_key()?;

             // Build genesis trie: admin = this persona.
             let admin_id = fresh_member_id(rng);
             let admin_leaf = MemberLeaf::new(
                 admin_id,
                 Handle::parse(&handle)?,
                 member_key,
                 Name::parse(&name).map_err(OrgMembersError::from)?,
                 Surname::parse(&surname).map_err(OrgMembersError::from)?,
                 vec![device_key],
             )
             .map_err(OrgNodeError::Trie)?;
             let trie = Trie::genesis(vec![admin_leaf]).map_err(OrgNodeError::Trie)?;

             let genesis_root_hash = trie.root_hash().map_err(OrgNodeError::Trie)?;
             let genesis_root = *genesis_root_hash.as_bytes();

             // REQ-ech45n: the Organisation key is a secret drawn here, for this
             // Organisation alone, and kept only in the encrypted store. Its public
             // key may equal no key the genesis record holds.
             let org_kp = X25519Keypair::generate(rng);
             let org_pub_key = org_kp.public_bytes();
             if &org_pub_key == member_key.as_bytes() || &org_pub_key == device_key.as_bytes() {
                 return Err(OrgNodeError::Trie(OrgMembersError::DuplicateKey));
             }

             // Submit genesis to chain (stub for headless test; real chain for production).
             // Returns the org_id AND the pure-proxy AccountId32 P (Some for SubxtChainOps,
             // None for MockChainOps).  P is persisted in OrgRecord so submit_update can
             // find it after a restart.
             let (org_id, proxy_account) = self.chain.submit_genesis(genesis_root, org_pub_key).await?;

             // Persist the OrgRecord.
             let admin_snap = MemberSnapshot {
                 id: *admin_id.as_bytes(),
                 handle: handle.clone(),
                 name: name.clone(),
                 surname: surname.clone(),
                 member_key: *member_key.as_bytes(),
                 device_keys: vec![*device_key.as_bytes()],
             };
             let org_rec = OrgRecord {
                 org_id,
                 root_hash: genesis_root,
                 org_pub_key,
                 epoch: 1,
                 org_secret: None,
                 last_seq: 0,
                 admin_member_key: *member_key.as_bytes(),
                 trie_members: vec![admin_snap],
                 // Persist the pure-proxy AccountId32 returned by submit_genesis so that
                 // submit_update can find P even after a restart (fixes Gap 2).
                 // None for MockChainOps; Some(p) for SubxtChainOps.
                 proxy_account,
                 org_private_key: Some(org_kp.to_seed()),
             };
             self.store.data_mut().orgs.push(org_rec);

             // Transition persona to Active.
             self.update_persona_status(persona_id, org_id, PersonaStatus::Active)?;

             self.store.save(rng)?;
             Ok(org_id)
         }
     ```
     The T3 interim `signer` lines are gone with this.
   - `receive_and_verify`, in the `OrgRecord { … }` it pushes on a first
     admission: add `org_private_key: None,` after `proxy_account: None,`.

5. `org-node/tests/store_at_rest.rs`, in `org_record`: add
   `org_private_key: None,` after `proxy_account: None,`.

6. Run ORG-NODE-GATE with ` --test organisation_key` appended, into
   `$S/plan4-t6-org-node.log`. Expected: every target `test result: ok`,
   organisation_key 4 passed, `exit 0`.

7. Commit:

   ```bash
   git add org-node/Cargo.toml org-node/.guardrails/config.yaml org-node/src/store.rs org-node/src/service.rs org-node/tests/store_at_rest.rs org-node/tests/organisation_key.rs
   ```
   ```bash
   git -c commit.gpgsign=false commit -m "feat(org-node): a created Organisation publishes the X25519 key of a fresh secret held only in the encrypted store (REQ-ech45n)"
   ```

**Status: done (2026-10-05).**

- Added beside the plan, for the class C robustness rule (the secret never
  in a log): `the_organisation_secret_is_not_in_the_record_debug_output`
  (REQ-ech45n) in `organisation_key.rs`. `OrgRecord`'s derived `Debug` would
  print `org_private_key` as its 32 bytes. `store.rs` now has a hand-written
  `Debug` for `OrgRecord` that prints every field as before and
  `org_private_key` only as `Some("..")` or `None`.
- red -> green: all five `organisation_key` tests: watched fail first with
  the test file alone, the target refusing to compile with four
  `E0609 no field org_private_key on type OrgRecord`
  (`$S/step4-t6-red1.log`). Then with step 3 applied and `create_organisation`
  still publishing the interim ed25519 key with `org_private_key: None`, all
  five ran and failed on their assertions (`$S/step4-t6-red2.log`):
  - `a_created_organisation_publishes_a_fresh_key_no_genesis_key_equals`
    (REQ-ech45n): `the creator holds the Organisation secret` (None).
  - `two_organisations_of_one_persona_publish_two_keys` (REQ-ech45n):
    `assertion left != right failed` (the same key published twice).
  - `the_organisation_secret_is_kept_only_in_the_encrypted_store`
    (REQ-ech45n): `Option::unwrap() on a None value`.
  - `an_organisation_key_equal_to_a_genesis_key_is_refused` (REQ-ech45n):
    `unwrap_err() on an Ok value` (creation went through).
  - `the_organisation_secret_is_not_in_the_record_debug_output`
    (REQ-ech45n): `Option::unwrap() on a None value`.
- red -> green: `the_organisation_secret_is_not_in_the_record_debug_output`
  (REQ-ech45n): with step 4 applied and `OrgRecord`'s `Debug` still derived,
  it failed with `the Organisation secret is printed`, and the other four
  passed (`$S/step4-t6-green1.log`). It passes with the redacting `Debug`
  (`$S/step4-t6-green2.log`, 5 passed).
- ORG-NODE-GATE with `--test organisation_key` (`$S/step4-t6-org-node.log`):
  `exit 0`. lib 24, admission_sender 10, organisation_key 5, service_stories
  3, store_at_rest 4, transport_handshake 3, transport_networked 1,
  verify_against_chain 14, wire_frame_bound 3; the three bolero targets ran
  without a failure. No compiler warnings from org-node.
- The chopsticks `--no-run` line still does not compile, and none of its
  errors come from T6. They are `chain_genesis_e2e.rs`'s
  `SignedDeltaEnvelope`, `member_key` on `SigningKeypair` and
  `author_member_key` (T7; `$S/step4-t6-chopsticks.log`).
- APP-GATE not run: the app still matches `BadSignature` until T8.

---

### T7 — `OrgPublicKey`, parsed where chain state is read (REQ-8jb4ny)

**Files touched:** `Cargo.lock`, `app/src-tauri/Cargo.lock`, `org-node/Cargo.toml`, `org-node/src/org_key.rs` (new), `org-node/src/lib.rs`, `org-node/src/error.rs`, `org-node/src/keys.rs`, `org-node/src/chain.rs`, `org-node/src/chain_read.rs`, `org-node/src/service.rs`, `org-node/src/store.rs`, `org-node/src/blobs.rs`, `org-node/src/test_fixtures.rs`, `org-node/docs/CONTEXT.md`, `org-node/tests/organisation_key.rs`, `org-node/tests/verify_against_chain.rs`, `org-node/tests/transport_handshake.rs`, `org-node/tests/transport_networked.rs`, `org-node/tests/fuzz_verify_against_chain/fuzz_target.rs`, `org-node/tests/store_at_rest.rs`, `org-node/tests/chain_genesis_e2e.rs`
**Parallel:** no (serial, after T6)
**verifies:** `REQ-8jb4ny` (three new tests)

`OrgState::from_chain` is the one edge where chain bytes become an
`OrgState`. Both chain readers (`SubxtChainOps::read_state`, the production
path, and `OnChainReader::refresh`) go through it. An `OrgState` cannot hold
an unparsed key, so no Envelope is verified against one (`verify.rs` reads
only `OrgState`). `OnChainReader::refresh` keeps its `String` error. It is off
the production receive path (its module doc says so), and its
`ChainReader` trait's error type is unchanged.

1. Failing tests: append to `org-node/tests/organisation_key.rs`. Add
   `use std::time::Duration;`, `use org_node::chain::OrgState;`,
   `use org_node::keys::SigningKeypair;`,
   `use org_node::service::ChainOps;`,
   `use org_node::transport::endpoint::OrgEndpoint;` and
   `use org_node::OrgPublicKey;` to the imports. Change the two helpers'
   bodies to `*chain.get(org_id).unwrap().org_pub_key.as_bytes()` and
   `*rec.org_pub_key.as_bytes()`. Then add:

   ```rust
   const NET: Duration = Duration::from_secs(30);

   /// A chain whose reads carry an Organisation public key that is not a
   /// valid X25519 key: the bytes a corrupted or hostile chain answer would
   /// carry, passed through the node's one edge for chain state.
   struct InvalidKeyChain(MockChainOps);

   #[async_trait::async_trait]
   impl ChainOps for InvalidKeyChain {
       async fn submit_genesis(
           &self,
           genesis_root: [u8; 32],
           org_pub_key: OrgPublicKey,
       ) -> Result<(OrgId, Option<[u8; 32]>), OrgNodeError> {
           self.0.submit_genesis(genesis_root, org_pub_key).await
       }

       async fn submit_update(
           &self,
           org_id: OrgId,
           new_root: [u8; 32],
           org_pub_key: OrgPublicKey,
           expected_epoch: u64,
           proxy_account: Option<[u8; 32]>,
       ) -> Result<(), OrgNodeError> {
           self.0.submit_update(org_id, new_root, org_pub_key, expected_epoch, proxy_account).await
       }

       async fn read_state(&self, org_id: OrgId) -> Result<Option<OrgState>, OrgNodeError> {
           match self.0.read_state(org_id).await? {
               Some(s) => OrgState::from_chain(*s.root_hash.as_bytes(), [0u8; 32], s.epoch).map(Some),
               None => Ok(None),
           }
       }
   }

   /// The device keypair of a persona, from its persisted `device_seed`.
   fn device_kp(svc: &OrgService, persona_id: &str) -> SigningKeypair {
       let seed = svc
           .list_personas()
           .iter()
           .find(|p| p.persona_id == persona_id)
           .map(|p| p.device_seed)
           .expect("persona not found");
       SigningKeypair::from_seed(seed)
   }

   // The parse: canonical, non-small-order X25519 keys only, by person's rule.
   // verifies: REQ-8jb4ny
   #[test]
   fn an_organisation_public_key_that_is_not_a_valid_x25519_key_is_refused() {
       let valid = X25519Keypair::from_seed([0x42u8; 32]).public_bytes();
       assert_eq!(OrgPublicKey::parse(&valid).unwrap().as_bytes(), &valid);

       let mut top_bit = valid;
       top_bit[31] |= 0x80; // at least 2^255: not canonical
       let mut one = [0u8; 32];
       one[0] = 1; // small order
       let mut prime = [0xffu8; 32]; // 2^255 - 19: not canonical
       prime[0] = 0xed;
       prime[31] = 0x7f;
       for bad in [[0u8; 32], one, top_bit, prime] {
           assert_eq!(
               OrgPublicKey::parse(&bad),
               Err(OrgNodeError::InvalidOrgPublicKey),
               "{bad:02x?} must be refused"
           );
       }
   }

   // The edge: an Organisation state read from the chain is refused, with the
   // typed error, when its key is not valid, and accepted with its fields
   // intact when it is.
   // verifies: REQ-8jb4ny
   #[test]
   fn an_organisation_state_read_with_an_invalid_key_is_refused() {
       assert_eq!(
           OrgState::from_chain([7u8; 32], [0u8; 32], 3),
           Err(OrgNodeError::InvalidOrgPublicKey)
       );
       let key = X25519Keypair::from_seed([0x42u8; 32]).public_bytes();
       let state = OrgState::from_chain([7u8; 32], key, 3).unwrap();
       assert_eq!(state.org_pub_key.as_bytes(), &key);
       assert_eq!(state.root_hash.as_bytes(), &[7u8; 32]);
       assert_eq!(state.epoch, 3);
   }

   // "Shall act on no Envelope verified against it": B, whose chain answers
   // with an invalid Organisation public key, receives A's genuine admission
   // from A's own device, the invite's administrator. The receive fails with
   // the typed error and B commits nothing.
   // verifies: REQ-8jb4ny
   #[tokio::test(flavor = "multi_thread")]
   async fn a_receive_against_a_state_with_an_invalid_key_commits_nothing() {
       let chain = MockChainOps::new();
       let (store_a, _) = open_store("invalid-key", "a");
       let (store_b, _) = open_store("invalid-key", "b");
       let mut svc_a = OrgService::new(store_a, Box::new(chain.clone()));
       let mut svc_b = OrgService::new(store_b, Box::new(InvalidKeyChain(chain.clone())));

       let pid_a = svc_a.create_persona(&mut OsRng, "admin", "Admin", "User").unwrap();
       let org_id = svc_a.create_organisation(&mut OsRng, &pid_a).await.unwrap();
       let a_device = device_kp(&svc_a, &pid_a);
       let mut svc_a = svc_a.with_endpoint(OrgEndpoint::bind(&a_device).await.unwrap());

       let pid_b = svc_b.create_persona(&mut OsRng, "bob", "Bob", "Builder").unwrap();
       svc_b.import_invite(&mut OsRng, &svc_a.export_invite(org_id).unwrap()).unwrap();
       let jr = OrgService::import_join_request(&svc_b.export_join_request(&pid_b).unwrap()).unwrap();

       let ep_b = OrgEndpoint::bind(&device_kp(&svc_b, &pid_b)).await.unwrap();
       let b_addr = ep_b.inner().addr();
       let mut svc_b = svc_b.with_endpoint(ep_b);
       let b_task = tokio::spawn(async move {
           let r = tokio::time::timeout(NET, svc_b.receive_and_verify(&mut OsRng))
               .await
               .expect("B receive_and_verify timed out");
           (svc_b, r)
       });
       tokio::time::sleep(Duration::from_millis(50)).await;

       tokio::time::timeout(
           NET,
           svc_a.admit_member(&mut OsRng, org_id, &jr, b_addr, Some([0xffu8; 32])),
       )
       .await
       .expect("admit_member(B) timed out")
       .expect("admit_member(B) failed");

       let (svc_b, r) = b_task.await.unwrap();
       assert_eq!(r.unwrap_err(), OrgNodeError::InvalidOrgPublicKey);
       assert!(svc_b.list_orgs().is_empty(), "B must not have committed an OrgRecord");
       let persona_b = svc_b.list_personas().iter().find(|p| p.persona_id == pid_b).unwrap();
       assert_ne!(persona_b.status, PersonaStatus::Active);
   }
   ```

   Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test organisation_key`.
   Expected: does not compile (`unresolved import org_node::OrgPublicKey`).

2. `org-node/Cargo.toml`, `[dependencies]` after the `org-members` line:

   ```toml
   # The X25519 validity rule for the Organisation public key (REQ-8jb4ny):
   # person::x25519::is_valid_public_key, the rule PersonPublicKey applies.
   person = { path = "../person", default-features = false }
   ```

3. Create `org-node/src/org_key.rs`:

   ```rust
   //! The Organisation public key: the X25519 key-agreement key the chain
   //! records for an Organisation (root `docs/CONTEXT.md`). org-node's own
   //! type, constructed only through `parse`, which applies `person`'s
   //! exported X25519 validity rule (REQ-8jb4ny). It authenticates nothing
   //! the node receives.
   use core::fmt;

   use crate::error::OrgNodeError;

   #[derive(Clone, Copy, PartialEq, Eq, Hash)]
   pub struct OrgPublicKey([u8; 32]);

   impl OrgPublicKey {
       /// Accepts a canonical, non-small-order X25519 public key (REQ-8jb4ny).
       pub fn parse(bytes: &[u8; 32]) -> Result<Self, OrgNodeError> {
           if person::x25519::is_valid_public_key(bytes) {
               Ok(Self(*bytes))
           } else {
               Err(OrgNodeError::InvalidOrgPublicKey)
           }
       }

       pub fn as_bytes(&self) -> &[u8; 32] {
           &self.0
       }
   }

   impl TryFrom<[u8; 32]> for OrgPublicKey {
       type Error = OrgNodeError;

       fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
           Self::parse(&bytes)
       }
   }

   impl fmt::Debug for OrgPublicKey {
       fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
           f.write_str("OrgPublicKey(")?;
           for byte in &self.0[..4] {
               write!(f, "{byte:02x}")?;
           }
           f.write_str("..)")
       }
   }

   impl serde::Serialize for OrgPublicKey {
       fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
           self.0.serialize(serializer)
       }
   }

   /// Decoding (the store, an Invite) goes through `parse` too.
   impl<'de> serde::Deserialize<'de> for OrgPublicKey {
       fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
           let bytes = <[u8; 32]>::deserialize(deserializer)?;
           Self::parse(&bytes).map_err(serde::de::Error::custom)
       }
   }
   ```

4. `org-node/src/lib.rs`: add `pub mod org_key;` after `pub mod keys;`, and
   `pub use org_key::OrgPublicKey;` after `pub use keys::SigningKeypair;`.

5. `org-node/src/error.rs`, after `UnknownSender`:

   ```rust
       /// The Organisation state read from the chain carries an Organisation
       /// public key that is not a valid X25519 public key (REQ-8jb4ny).
       #[error("organisation public key read from the chain is not a valid X25519 public key")]
       InvalidOrgPublicKey,
   ```

6. `org-node/src/keys.rs`: add `use crate::error::OrgNodeError;` and
   `use crate::org_key::OrgPublicKey;`, and to `impl X25519Keypair`:

   ```rust
       /// As an Organisation public key, through `OrgPublicKey::parse`.
       pub fn org_public_key(&self) -> Result<OrgPublicKey, OrgNodeError> {
           OrgPublicKey::parse(&self.public_bytes())
       }
   ```

7. `org-node/src/chain.rs`: import `use crate::error::OrgNodeError;` and
   `use crate::org_key::OrgPublicKey;`. The field becomes
   `pub org_pub_key: OrgPublicKey,`. Add after the struct:

   ```rust
   impl OrgState {
       /// The edge where an Organisation state read from the chain enters the
       /// node: its Organisation public key must be a valid X25519 public key,
       /// or the state is refused (REQ-8jb4ny).
       pub fn from_chain(root_hash: [u8; 32], org_pub_key: [u8; 32], epoch: u64) -> Result<Self, OrgNodeError> {
           Ok(Self { root_hash: RootHash::new(root_hash), org_pub_key: OrgPublicKey::parse(&org_pub_key)?, epoch })
       }
   }
   ```
   In its unit test, `org_pub_key: [3u8; 32]` becomes
   `org_pub_key: crate::keys::X25519Keypair::from_seed([3u8; 32]).org_public_key().unwrap()`.

8. `org-node/src/chain_read.rs`: delete `use org_members::RootHash;`.
   `map_state` becomes

   ```rust
   /// Maps on-chain-client's typed OrgState to org-node's, through the one edge
   /// that refuses an invalid Organisation public key (REQ-8jb4ny).
   fn map_state(s: on_chain_client::OrgState) -> Result<OrgState, crate::error::OrgNodeError> {
       OrgState::from_chain(s.root_hash.0, s.org_pub_key.0, s.epoch.0)
   }
   ```
   and in `refresh`, `.map(map_state);` becomes
   `.map(map_state).transpose().map_err(|e| e.to_string())?;`.

9. `org-node/src/service.rs`:
   - `use crate::org_key::OrgPublicKey;`.
   - `ChainOps`: in `submit_genesis` and `submit_update`, `org_pub_key: [u8; 32]`
     becomes `org_pub_key: OrgPublicKey`. Make the same change in both
     `MockChainOps` methods. Their bodies store `org_pub_key` in `OrgState`
     unchanged.
   - `subxt_impl`: delete `use org_members::RootHash;`. `map_state` becomes
     `fn map_state(s: on_chain_client::OrgState) -> Result<OrgState, OrgNodeError> { OrgState::from_chain(s.root_hash.0, s.org_pub_key.0, s.epoch.0) }`.
     The parameters become `OrgPublicKey`. `genesis_ceremony(…, genesis_root, *org_pub_key.as_bytes())`.
     `revive_update_runtime_call(self.contract_h160, new_root, *org_pub_key.as_bytes(), u128::from(expected_epoch))`.
     `read_state` ends:

     ```rust
                 .map_err(|e| OrgNodeError::Chain(format!("get_org_state: {e}")))?
                 .map(map_state)
                 .transpose()?;
             Ok(state)
     ```
   - `create_organisation`: `let org_pub_key = org_kp.org_public_key()?;` and
     the collision check compares
     `org_pub_key.as_bytes() == member_key.as_bytes() || org_pub_key.as_bytes() == device_key.as_bytes()`.

10. `org-node/src/store.rs`: `use crate::org_key::OrgPublicKey;`.
    `OrgRecord.org_pub_key` and `PendingInvite.org_pub_key` become
    `OrgPublicKey`.

11. `org-node/src/blobs.rs`: `use crate::org_key::OrgPublicKey;`.
    `Invite.org_pub_key` becomes `OrgPublicKey`. In `invite_round_trips`,
    `org_pub_key: [1u8; 32],` becomes
    `org_pub_key: crate::keys::X25519Keypair::from_seed([1u8; 32]).org_public_key().unwrap(),`.

12. `org-node/src/test_fixtures.rs`: add `use crate::org_key::OrgPublicKey;` and

    ```rust
    /// A valid Organisation public key, for chain states in tests.
    pub fn org_public_key() -> OrgPublicKey {
        X25519Keypair::from_seed([9u8; 32]).org_public_key().unwrap()
    }
    ```

13. Tests that build an `OrgState` or `OrgRecord`:
    - `org-node/tests/verify_against_chain.rs`: import `org_public_key` from
      `org_node::test_fixtures`. In `chain_at`,
      `org_pub_key: [0u8; 32]` becomes `org_pub_key: org_public_key()`.
    - `org-node/tests/transport_handshake.rs` and
      `org-node/tests/transport_networked.rs`: `org_pub_key: [0u8; 32]`
      becomes `org_pub_key: X25519Keypair::from_seed([9u8; 32]).org_public_key().unwrap()`.
    - `org-node/tests/fuzz_verify_against_chain/fuzz_target.rs`: the same
      replacement. The target builds without `test-support` and cannot use
      the fixture.
    - `org-node/tests/store_at_rest.rs`: `use org_node::keys::X25519Keypair;`.
      In `org_record`, `org_pub_key: [0x22u8; 32],` becomes
      `org_pub_key: X25519Keypair::from_seed([0x22u8; 32]).org_public_key().unwrap(),`.
    - `org-node/tests/chain_genesis_e2e.rs` (not compiled by
      ORG-NODE-GATE; every change since T2 lands here at once):
      the import line becomes
      `use org_node::{ChainReader, Envelope, OrgId, SeqGuard, SigningKeypair, verify_envelope_against_chain, VerifyContext};`
      plus `use org_node::keys::X25519Keypair;`. `admin_leaf(admin_kp: &X25519Keypair, admin_device: &SigningKeypair)`
      and `member_b_leaf(b_kp: &X25519Keypair, b_device: &SigningKeypair)`,
      each `vec![…device_key()]` becoming `vec![….device_key().expect("valid device key")]`.
      In both tests:
      `let admin_kp = X25519Keypair::from_seed([0xA1u8; 32]);`,
      `let org_pub_key: [u8; 32] = X25519Keypair::from_seed([0xA3u8; 32]).public_bytes();`,
      `let b_kp = X25519Keypair::from_seed([0xB1u8; 32]);`. The envelope is
      `let env = Envelope::build(org_id, 2, &admit_delta).expect("build envelope");`
      (drop the `&admin_kp` argument and its comment line about signing).
      The context is

      ```rust
          let sender = admin_device.device_key().expect("valid device key");
          let accepted = [sender];
          let ctx = VerifyContext {
              expected_org_id: org_id,
              sender: &sender,
              accepted_senders: &accepted,
              seq_guard: SeqGuard::from_last_seen(1), // last committed seq was 1
              last_committed_epoch: 1,                // last committed epoch was 1
          };
      ```

14. `org-node/docs/CONTEXT.md`, **Invite**: "the Organisation, its published
    signing key and the administrator's Device key." becomes "the
    Organisation, its Organisation public key, the administrator's
    Member-as-a-group key and the administrator's Device key."

15. Run:
    - ORG-NODE-GATE (with `--test organisation_key`) into
      `$S/plan4-t7-org-node.log`. Expected: every target `test result: ok`,
      organisation_key 7 passed, `exit 0`.
    - `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test chain_genesis_e2e --test finality_polling --test preflight --no-run`
      into `$S/plan4-t7-chopsticks-build.log`. Expected:
      `Finished`, three `Executable` lines, `exit 0`. Do not run them: they
      need a chopsticks fork.
    - `grep -rn "org_pub_key: \[u8; 32\]\|\[0u8; 32\], epoch" org-node/src org-node/tests`.
      Expected: only `chain_write/calldata.rs`, `chain_write/submit.rs` and
      `ceremony.rs`, which keep raw bytes as the contract's calldata, and
      `OrgState::from_chain`'s own signature.

16. Commit:

    ```bash
    git add Cargo.lock app/src-tauri/Cargo.lock org-node/Cargo.toml org-node/src/org_key.rs org-node/src/lib.rs org-node/src/error.rs org-node/src/keys.rs org-node/src/chain.rs org-node/src/chain_read.rs org-node/src/service.rs org-node/src/store.rs org-node/src/blobs.rs org-node/src/test_fixtures.rs org-node/docs/CONTEXT.md org-node/tests/organisation_key.rs org-node/tests/verify_against_chain.rs org-node/tests/transport_handshake.rs org-node/tests/transport_networked.rs org-node/tests/fuzz_verify_against_chain/fuzz_target.rs org-node/tests/store_at_rest.rs org-node/tests/chain_genesis_e2e.rs
    ```
    ```bash
    git -c commit.gpgsign=false commit -m "feat(org-node): OrgPublicKey parsed through person's X25519 rule wherever chain state is read; an invalid key is refused with InvalidOrgPublicKey (REQ-8jb4ny)"
    ```

**Status: done (2026-10-05).**

- The plan's code worked as written. The class C robustness inputs for
  REQ-8jb4ny are in the first test: all-zero, small order (`u = 1`),
  non-canonical at `u = 2^255 - 19` and with the top bit set. The third
  test covers "acts on no Envelope verified against such a state".
- red -> green: all three new `organisation_key` tests (REQ-8jb4ny):
  watched fail first with the test file alone, the target refusing to
  compile: `E0432 unresolved import org_node::OrgPublicKey`, three
  `E0599 no … from_chain found for struct OrgState`, three
  `E0599 no variant … InvalidOrgPublicKey` and two `E0599 no method
  as_bytes found for array [u8; 32]` (`$S/step4-t7-red1.log`). With steps
  2–12 applied they pass (`$S/step4-t7-green1.log`, 8 passed):
  - `an_organisation_public_key_that_is_not_a_valid_x25519_key_is_refused`
  - `an_organisation_state_read_with_an_invalid_key_is_refused`
  - `a_receive_against_a_state_with_an_invalid_key_commits_nothing`
- Mutation check: with `OrgPublicKey::parse` made to accept every input,
  all three fail and the five REQ-ech45n tests still pass
  (`$S/step4-t7-mutant.log`). So the receive test depends on the parse,
  not just on the error type.
- ORG-NODE-GATE with `--test organisation_key` (`$S/step4-t7-org-node.log`):
  `exit 0`. lib 24, admission_sender 10, organisation_key 8 (the plan's
  expected 7 was a miscount: 5 from T6 plus 3), service_stories 3,
  store_at_rest 4, transport_handshake 3, transport_networked 1,
  verify_against_chain 14, wire_frame_bound 3. The three bolero targets ran
  without a failure.
- Chopsticks `--no-run` (`$S/step4-t7-chopsticks-build.log`): `Finished`,
  three `Executable` lines, `exit 0`. Not run.
- The step 15 grep prints the expected lines, plus the two
  `let org_pub_key: [u8; 32]` lines in `chain_genesis_e2e.rs`. Those are
  raw calldata bytes passed to `genesis_ceremony` and
  `revive_update_runtime_call`.
- `app/src-tauri/Cargo.lock` is unchanged: the app was not built in T7.
  The root `Cargo.lock` gains `person` under org-node.

---

### T8 — App: the renamed verdict, the new receiver error, the retyped stubs

**Files touched:** `app/src-tauri/src/events.rs`, `app/src-tauri/src/state.rs`, `app/src-tauri/tests/receiver_events.rs`, `app/docs/requirements/2026-09-14-tauri-shell.md`
**Parallel:** no (serial, after T7)
**verifies:** `REQ-kn5rtx` (one new test; the existing REQ-kn5rtx and REQ-jfxah3 tests updated)

`UnknownSender` rejects the received update itself, as `BadSignature` did,
so it stays a verdict. `InvalidOrgPublicKey` is raised by the chain read,
before anything is verified. By `classify_receive_error`'s own rule (claim
no verdict that cannot be supported), it is a receiver error.

1. Failing test first, in `app/src-tauri/tests/receiver_events.rs`:
   - in `verification_verdicts()`, `OrgNodeError::BadSignature,` becomes `OrgNodeError::UnknownSender,`;
   - in the REQ-jfxah3 test that ends with the verdict contrast,
     `// …and neither is a verdict on an update: a bad signature is a reason to`
     becomes `// …and neither is a verdict on an update: an unknown sender is a reason to`,
     and `let verdict = OrgNodeError::BadSignature;` becomes `let verdict = OrgNodeError::UnknownSender;`;
   - add after `a_chain_failure_emits_no_verification_event_for_any_message`:

     ```rust
     // verifies: REQ-kn5rtx
     #[test]
     fn an_invalid_organisation_public_key_is_classified_as_a_receiver_error() {
         // org-node refuses an Organisation state whose key is not a valid X25519
         // key when it reads the chain, before verifying anything against it:
         // nothing failed to verify, so no verdict may be claimed.
         let e = OrgNodeError::InvalidOrgPublicKey;
         let outcome = events::classify_receive_error(&e);
         assert_eq!(
             outcome,
             ReceiverOutcome::ReceiveError {
                 message: e.to_string()
             }
         );
         let out = events::emissions_for(&outcome);
         assert_eq!(names(&out), vec!["receiver-error"]);
         assert!(
             !names(&out).contains(&"verification-failed"),
             "an unreadable Organisation state is not a verification failure"
         );
     }
     ```

   Run APP-GATE. Expected: does not compile (`no variant BadSignature` in
   events.rs, `expected OrgPublicKey, found [u8; 32]` in state.rs).

2. `app/src-tauri/src/events.rs`, `classify_receive_error`:

   ```rust
           // Verdicts on an incoming update: each one means "this did not verify",
           // and each rejects the received update itself. `UnknownSender` is the
           // sender-device check: inside `verify_envelope_against_chain` before the
           // Change set is decoded, and the receive path's own checks around it.
           OrgNodeError::OrgIdMismatch
           | OrgNodeError::UnknownSender
           | OrgNodeError::StaleSeq { .. }
           | OrgNodeError::MalformedDelta
           | OrgNodeError::DeltaBaseMismatch
           | OrgNodeError::RootMismatch
           | OrgNodeError::StaleEpoch { .. } => ReceiverOutcome::VerifyFailed {
               // REQ-affyf5: the organisation is absent rather than invented. The
               // service reports the failure without naming one.
               org_id: None,
               message: e.to_string(),
           },

           // Not a verdict: the chain could not be read or answered with an
           // Organisation state the node refuses (`InvalidOrgPublicKey`, before
           // anything is verified against it), the transport failed, or the local
           // store could not supply what the verification needed.
           OrgNodeError::Chain(_)
           | OrgNodeError::OrgNotOnChain
           | OrgNodeError::Trie(_)
           | OrgNodeError::InvalidOrgPublicKey => ReceiverOutcome::ReceiveError {
               message: e.to_string(),
           },
     ```

3. `app/src-tauri/src/state.rs`, `ChainNotConfigured`: both
   `_org_pub_key: [u8; 32],` become `_org_pub_key: org_node::OrgPublicKey,`.

4. `app/docs/requirements/2026-09-14-tauri-shell.md`: insert, as its own
   paragraph immediately before the line beginning
   `Both are now classified as receiver errors.`:

   ```markdown
   *(2026-10-05.)* org-node dropped the Envelope signature, and `BadSignature`
   became `UnknownSender`: the received update came from a device the node does
   not accept as a sender. It rejects that update, as `BadSignature` did, and
   stays among the seven verdicts. org-node also gained `InvalidOrgPublicKey`,
   raised when the Organisation state read from the chain carries a key that is
   not a valid X25519 key, before anything is verified against it. It is a
   receiver error, for the reason the next paragraph gives.
   ```

5. Run APP-GATE into `$S/plan4-t8-app-cargo.log`. Expected: every target
   `test result: ok`, receiver_events 28 passed (27 + 1), `exit 0`. Then
   `npm --prefix app ci` if `app/node_modules` is absent, then
   `npm --prefix app run check` (expected `0 errors`, the one pre-existing
   `@types/node` warning) and `npm --prefix app run test` (expected 30 tests
   passed across four files).

6. Commit:

   ```bash
   git add app/src-tauri/src/events.rs app/src-tauri/src/state.rs app/src-tauri/tests/receiver_events.rs app/docs/requirements/2026-09-14-tauri-shell.md
   ```
   ```bash
   git -c commit.gpgsign=false commit -m "fix(app): UnknownSender replaces BadSignature as a verdict; InvalidOrgPublicKey is a receiver error; ChainOps stubs take OrgPublicKey"
   ```

**Status: done (2026-10-05).**

- The plan's code worked as written.
- red -> green: `an_invalid_organisation_public_key_is_classified_as_a_receiver_error`
  (REQ-kn5rtx), with the two updated tests that now name `UnknownSender`
  (`every_verification_verdict_…`, `a_verification_verdict_carries_…`,
  REQ-kn5rtx; `a_non_terminal_failure_does_not_stop_the_loop`, REQ-jfxah3):
  watched fail first with the test edits alone, the crate refusing to
  compile: two `E0053` at `state.rs:77` and `:89` (`expected OrgPublicKey,
  found [u8; 32]`) and one `E0599 no variant … BadSignature` at
  `events.rs:72` (`$S/step4-t8-red.log`). With steps 2–3 applied they pass.
- APP-GATE (`$S/step4-t8-app-cargo.log`): `exit 0`, 79 passed:
  connection_status 10, ipc 9, org_id_parsing 14, receiver_events 28,
  receiver_guard 6, startup_policy 12.
- `npm --prefix app ci` (no `node_modules` in the task worktree), then
  `npm --prefix app run check`: 0 errors, the one `@types/node` warning;
  `npm --prefix app run test`: 4 files, 30 tests passed.
- `GR_CONFIG=app/.guardrails/config.yaml .guardrails/scripts/check-trace.sh`
  (`$S/step4-t8-trace-app.log`): `exit 0`.
- `app/src-tauri/Cargo.lock` gains `person` under org-node. Cargo rewrote it
  as `version = 4`; set back to 3, and `cargo metadata --locked` accepts it.

---

### T9 — Full verification, measured counts, traceability

**Files touched:** `org-node/.guardrails/config.yaml`
**Parallel:** no (serial, after T8)
**verifies:** none new (gate task)

1. Run every org-node `verify_commands` entry exactly as
   `org-node/.guardrails/config.yaml` lists it, with
   `CARGO_HOME=/tmp/cargo_home_fuzz` on the cargo line and
   `QUINT_HOME` set to a scratch directory seeded from
   `~/.quint/rust-evaluator-v0.7.0` for the quint lines. Output goes to
   `$S/plan4-t9-org-node.log`. Expected: the cargo line exits 0 with every
   target `test result: ok`. `quint --version` prints a version. Both
   typechecks exit 0. All five `quint run` invariants print
   `[ok] No violation found`. The quint models are untouched by this plan.

2. Run every app `verify_commands` entry into `$S/plan4-t9-app.log`.
   Expected: all three exit 0, with the counts given in T8 step 5.

3. Run the chopsticks build check:
   `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features app,test-support --test chain_genesis_e2e --test finality_polling --test preflight --no-run`.
   Expected: `exit 0`.

4. `org-node/.guardrails/config.yaml`: in the comment block above
   `verify_commands`, after the `2026-10-03, T2 of …fuzz_first_admission_base…`
   paragraph, add:

   ```yaml
   # 2026-10-05, docs/plans/2026-10-05-org-node-organisation-public-key.md: a
   # target organisation_key (REQ-ech45n, REQ-8jb4ny) is added to the line
   # below. The Envelope lost its signature; the sender-device rule
   # (REQ-7h7qp3) took its place in verify_against_chain and admission_sender.
   # Measured 2026-10-05 (agent sandbox, CARGO_HOME=/tmp/cargo_home_fuzz):
   ```
   and complete that last line with the per-target pass counts the step-1 run
   printed, in the form used by the 2026-09-10 note above it
   (`N passed, 0 failed — lib N, verify_against_chain N, …, organisation_key N`).
   Write the numbers the run printed, not the expectations in this plan.

5. Traceability:

   ```bash
   S=/private/tmp/claude-501/-Users-jan-jan-Coding-2-tier-access-control/040b9a0d-b135-4b8c-ab0a-f48aa8a3106c/scratchpad
   GR_CONFIG=org-node/.guardrails/config.yaml .guardrails/scripts/check-trace.sh > $S/plan4-t9-trace-org-node.log 2>&1; echo "exit $?" >> $S/plan4-t9-trace-org-node.log
   GR_CONFIG=app/.guardrails/config.yaml .guardrails/scripts/check-trace.sh > $S/plan4-t9-trace-app.log 2>&1; echo "exit $?" >> $S/plan4-t9-trace-app.log
   grep -E "MISSING-TEST|DANGLING|ORPHAN|^exit" $S/plan4-t9-trace-org-node.log $S/plan4-t9-trace-app.log
   ```

   Expected: no `MISSING-TEST` (the four from T1 closed:
   REQ-7h7qp3 by verify_against_chain, admission_sender and service_stories;
   REQ-txvtm9 by verify_against_chain and service_stories; REQ-8jb4ny and
   REQ-ech45n by organisation_key), no new finding, and `exit 0` for both
   units. The pre-existing UNMET-EXPECTATION and UNRESOLVED-PR lines remain,
   within their limits.

6. Commit:

   ```bash
   git add org-node/.guardrails/config.yaml
   ```
   ```bash
   git -c commit.gpgsign=false commit -m "docs(org-node): verify_commands note for the organisation_key target, measured counts"
   ```

Hand-off after T9: `check-traceability`, `verify-before-merge`, then
`merge-change` for the whole branch (steps 2–4 merge together, per
`docs/plans/2026-10-04-person-shared-types.md`).

---

## Trace summary

| ID | Tests that verify it after this plan |
|---|---|
| REQ-7h7qp3 | verify_against_chain: `rejects_a_sender_outside_the_accepted_devices`, `an_empty_accepted_set_accepts_no_sender`, `rejects_unknown_sender_before_decoding_delta`; admission_sender: `first_admission_from_a_device_other_than_the_invites_admin_is_rejected`, `first_admission_without_an_imported_invite_is_rejected`, `first_admission_from_a_wrong_sender_is_rejected_before_its_snapshot_is_decoded`, `update_relayed_by_a_non_member_after_admission_is_rejected`; service_stories: `revocation_from_an_unknown_device_leaves_the_record_in_place` |
| REQ-txvtm9 | verify_against_chain: `happy_path_commits_when_root_matches_chain`, `rejects_root_mismatch_when_chain_root_differs`; service_stories: `five_stories_full_e2e` |
| REQ-8jb4ny | organisation_key: `an_organisation_public_key_that_is_not_a_valid_x25519_key_is_refused`, `an_organisation_state_read_with_an_invalid_key_is_refused`, `a_receive_against_a_state_with_an_invalid_key_commits_nothing` |
| REQ-ech45n | organisation_key: `a_created_organisation_publishes_a_fresh_key_no_genesis_key_equals`, `two_organisations_of_one_persona_publish_two_keys`, `the_organisation_secret_is_kept_only_in_the_encrypted_store`, `an_organisation_key_equal_to_a_genesis_key_is_refused` |
| RC-2e6k44 | through REQ-7h7qp3 and REQ-txvtm9 (`implements: RC-2e6k44`) |
| REQ-ag6kqm (superseded) | kept beside REQ-7h7qp3 on the two rewritten verify_against_chain tests |
| REQ-nhe2zu (superseded) | kept beside REQ-txvtm9 on the happy path, the root mismatch and `five_stories_full_e2e` |
| REQ-ztdza4 | `update_from_the_admin_after_admission_is_committed` (unchanged), `update_relayed_by_the_device_it_removes_is_rejected` (new); moved off the rogue-relay test, which no longer reaches its check |

## Decisions this plan makes that the owner may overturn

These follow from the inputs but are not owner rulings. Each is cheap to
change before execution.

1. `SignedDeltaEnvelope` is renamed `Envelope` (the glossary's term; it lists
   "delta envelope" under _Avoid_). `OrgNodeError::BadSignature` is renamed
   `UnknownSender`.
2. The sender check sits inside `verify_envelope_against_chain` (step 2,
   where the signature check was), with the accepted set passed in
   `VerifyContext`, rather than in the service before the call.
3. REQ-ztdza4's post-verification check is kept beside REQ-7h7qp3's
   pre-decode check. The rogue-relay test is re-annotated from REQ-ztdza4 to
   REQ-7h7qp3, and a new test covers REQ-ztdza4.
4. A first admission with no imported Invite is refused. It used to fall
   through. This is how this plan reads REQ-7h7qp3's "otherwise a Device key
   present in its own current record" for a node that has no record.
5. ~~On a first admission the snapshot (`first_admission_base`) is still
   decoded before the sender check.~~ **Overturned by the dispatcher
   (2026-10-05):** the Invite names the administrator's Device key before
   anything is decoded, so on a first admission the sender check runs before
   the snapshot is decoded as well as before the Change set, as RC-2e6k44's
   rationale ("authenticity is settled before any work is done on
   attacker-chosen bytes") requires. T5 implements that order; a REQ-d9g6nt
   test that depended on the old order is adapted to send from the Invite's
   administrator device, keeping what it asserts, and if it cannot be, the
   task reports it instead of keeping the old order.

Settled by the dispatcher, beside the list above:

- REQ-gju89b (Organisation binding before decoding) now also names
  `implements: RC-2e6k44`, which contains the same check.
- The Organisation private key stays in the creator's store; giving it to
  members belongs to the future CGKA work, not this change.
- Encrypted stores written before this change do not decode after it
  (`OrgRecord` gains a field, the Envelope loses one). No store is in use
  outside development, as with the wire format the owner ruled on.
6. An Organisation key equal to a genesis key is refused with
   `Trie(OrgMembersError::DuplicateKey)`. The OrgRecord field is named
   `org_private_key`.
7. `InvalidOrgPublicKey` is an app receiver error, not a verdict.
8. `OnChainReader::refresh` reports an invalid key as a `String`, the
   `ChainReader` error type. It is off the production path.

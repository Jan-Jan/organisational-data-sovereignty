# Sparse Merkle Tree Implementation Plan

**Goal:** Build the `sparse-merkle-tree` crate (one immutable sparse Merkle tree of a compile-time depth, generic over its leaf and hasher, with absence and inclusion proofs), rebuild `person`'s device root and `org-members`' Membership record on it, and add the two-step absence proofs under a Membership root and under a Person hash — test-first, replacing every test that verifies an item this change took out of force.
**Implements:** sparse-merkle-tree REQ-vg9wq4, REQ-6cej5z, REQ-uy4euz, REQ-ymgg6b, REQ-s3unp5, REQ-dyjq33, REQ-3szczq, REQ-fmx9e7; SDD-x43d2z, SDD-rh9u6z, SDD-ur6uvu; LLR-jd4phg, LLR-znpbf5, LLR-t3e8pp, LLR-9z6dc9, LLR-ggu9bh, LLR-mm45ry, LLR-yuyn8f, LLR-kep2ju, LLR-8uem8c, LLR-ycf27m, LLR-3y5pq6, LLR-3zffc2, LLR-u2r4vj, LLR-byz55h, LLR-c4eunm, LLR-9qncuc, LLR-639mz9, LLR-m5c3vw, LLR-tz3nfz, LLR-5tww69, LLR-88rm99, LLR-77zy97, LLR-53ezgd, LLR-c7bdee, LLR-smf4zq. person REQ-5kvqcf, REQ-uh2bzg, REQ-9sh8qb, REQ-4e23xr; SDD-bx9mm6, SDD-7r833z and SDD-k5ee9x (amended); LLR-9mjfjn, LLR-8y8tqr, LLR-exwx4n, LLR-vraz7n, LLR-xsx6gg, LLR-pjf9qn, LLR-qwp8d7, LLR-985v9t, LLR-cve3d6, LLR-z8h2w8, LLR-3dvfjb. org-members REQ-sb2frw, REQ-65xrgh, REQ-2sz7j8; SDD-d9svdj and SDD-57vaj4 (amended); LLR-t465mb, LLR-4rdqyn, LLR-uudst2, LLR-utk6uy, LLR-5jh6t3, LLR-w5yahk, LLR-hbb4je, LLR-v6smww, LLR-bh53rr, LLR-9g2kjg, LLR-54gp2f, LLR-25nmwc, LLR-v3cysh. Through the REQs it realises RC-5zyczk, RC-gy5f5r (sparse-merkle-tree), RC-ja7gx3, RC-5d9qxe (person), RC-5nka8z, RC-j2znx8 (org-members).
**Safety class:** C for all three units (`safety_class: C` in each `.guardrails/config.yaml`); no per-item overrides — every SDD item in the three design files is class C.
**Verification:** each unit's `verify_commands`, run with `CARGO_HOME=/tmp/cargo_home_fuzz` (`~/.cargo` is read-only here) and, for org-members, `QUINT_HOME` set to a writable scratch directory:
- sparse-merkle-tree (added by T2): `cargo test -p sparse-merkle-tree`, `cargo test -p sparse-merkle-tree --no-default-features`, `cargo clippy -p sparse-merkle-tree --all-targets -- -D warnings`;
- person: `cargo test -p person`, `cargo clippy -p person --all-targets -- -D warnings`;
- org-members: `cargo test -p org-members` and its five `quint` entries;
- the downstream units `check-units.sh --impact` names (org-node, org-io, app), each by its own `verify_commands`;
- `make coverage-sparse-merkle-tree coverage-branch-sparse-merkle-tree` (added by T11), `make coverage-person coverage-branch-person`, `make coverage-org-members coverage-branch-org-members`;
- `GR_CONFIG=<unit>/.guardrails/config.yaml sh .guardrails/scripts/check-trace.sh` for the three units, exit 0, and `sh .guardrails/scripts/check-units.sh`, exit 0.

Context. Requirements, risk and design are this branch's
(`worktree-generic-smt`, 21 commits before this plan):
`sparse-merkle-tree/docs/{requirements,risk,architecture}/2026-10-2{3,4}-sparse-merkle-tree.md`;
`person/docs/requirements/2026-10-23-person-absence-proofs.md`,
`person/docs/risk/2026-10-23-person-absence-proofs.md`,
`person/docs/architecture/2026-10-24-device-root-tree.md`;
`org-members/docs/requirements/2026-10-23-two-level-absence-proofs.md`,
`org-members/docs/risk/2026-10-23-two-level-absence-proofs.md`,
`org-members/docs/architecture/2026-10-24-on-sparse-merkle-tree.md`;
the ADRs `docs/adr/2026-10-23-one-sparse-merkle-tree.md` and
`docs/adr/2026-10-23-unsalted-tree-leaves.md`; the edge assessment
`docs/plans/2026-10-24-sparse-merkle-tree-edges.md`. This change merges
before `jan-jan-person` (owner ruling 2026-10-24).

Every test this plan writes is annotated at the LLR level, so each REQ is
covered through the LLRs' `satisfies:` lines; a few tests also name the REQ
directly where the test is the REQ's end-to-end case.

## Decisions this plan makes

Each is within the design; the five that change an LLR's wording are made
in T1, in the file that defines the item, before any test names it.

- **The tree stores no empty nodes.** A child is `Option<Arc<Node<L>>>`;
  `None` is an empty subtree, whose hash is read from the level's
  empty-subtree hash when needed. A node is either a compact leaf
  (`Kind::Leaf(Arc<L>)`, at any level, LLR-kep2ju) or an internal node with
  two children, at least one of them `Some`. An empty tree is `root: None`
  and is calculated (it has no unset cell), so `calculate()` on it returns
  `HashesAlreadyCalculated` (LLR-3y5pq6 as written).
- **One form per leaf set.** `insert` places a leaf into an empty subtree
  as a compact leaf, and a second leaf into a compact leaf's subtree by
  splitting at the first bit where their positions differ, with a lone-child
  internal node on the shared side at every level above; `remove` raises a
  lone leaf beside an empty subtree to the node above, level by level. The
  root is the classic tree's either way (LLR-kep2ju); the single form is what
  makes `diff` and equality of structure line up with equality of leaf sets.
- **Leaves are held as `Arc<L>`,** so splitting, raising and diffing never
  clone a leaf. `prove_included` returns `(Arc<L>, InclusionProof<DEPTH>)`
  and `diff` returns `(Vec<L::Identity>, Vec<Arc<L>>)`.
- **Empty-subtree hashes are computed when an empty tree is made and shared
  by every tree derived from it, and once per proof verification.** A
  per-type static is not expressible in Rust for a generic `H`. T1 amends
  LLR-yuyn8f to say so.
- **`diff` accepts uncalculated trees.** `OrgTrie::pending_changes` diffs the
  last calculated tree against the current, uncalculated one, as the store
  does today. A pair of subtrees is skipped when they are one node
  (`Arc::ptr_eq`) or both have a hash set and the hashes are equal; a compact
  leaf facing an internal node is lowered one level and compared again. T1
  amends LLR-u2r4vj.
- **`from_leaves` and `root_of`** (new LLR-9qncuc) build a calculated tree
  from leaves in one call, refusing only with `Collision`; `get`, `contains`,
  `leaves` and `is_calculated` (new LLR-639mz9) read a tree. Both IDs were
  minted while writing this plan.
- **A proof's serde form carries its depth.** A depth-4 and a depth-8 map are
  both one byte, so without it a depth-4 proof would parse as a depth-8 one,
  against LLR-77zy97's last sentence. The form is the tuple
  `(depth: u16, map, siblings, ending)` for an absence proof and
  `(depth, map, siblings)` for an inclusion proof; `Deserialize` refuses a
  depth other than `DEPTH` before reading the map. T1 amends LLR-77zy97.
- **The map** is `DEPTH.div_ceil(8)` bytes; bit `level % 8` of byte
  `level / 8` set means the sibling at `level` (0 = the leaf's level) is not
  that level's empty-subtree hash and is among the siblings. Bits at levels
  `DEPTH` and above must be clear (`ProofMalformed` otherwise).
- **The crate implements `Display` and `core::error::Error` for `TreeError`
  by hand,** so its only dependencies stay `spin` and `serde` (LLR-c4eunm).
- **A depth outside 1..=256 fails to compile** through a constant
  `Depth::<D>::IN_RANGE` (an `assert!` evaluated at monomorphisation),
  named in `SparseMerkleTree::new` and `Path::parse`. Its refusing side is a
  `compile_fail` doctest beside a compiling twin, run by
  `cargo test -p sparse-merkle-tree` (doctests are part of `cargo test`);
  the `tests/` test annotated LLR-ggu9bh checks the accepted boundary.
- **The device root takes a slice.**
  `person::device_tree::<H>(&[DevicePublicKey])` and
  `compute_device_root::<H>(&[DevicePublicKey])` return
  `Result<_, IdentityError>`, refusing with `DevicePositionTaken` when two
  keys share a position. Callers pass `DeviceSlots::devices()`, which admits
  no such pair (LLR-xsx6gg), so the refusal never fires for them; taking a
  slice makes it reachable and testable instead of an uncovered branch.
  `person::hash::encode` becomes fallible for the same reason. T1 amends
  LLR-8y8tqr and LLR-qwp8d7.
- **org-members' tree leaf is `TreeMember`** (crate-private): a `MemberLeaf`
  and its device root, computed by `TreeMember::new::<H>` when the leaf
  enters the tree, as `LeafPayload` holds it today. `Leaf::content_hash` is
  infallible and cannot compute a fallible device root. T1 amends LLR-t465mb.
- **Two org-members hashers.** One type cannot implement `TreeHasher` twice,
  so the member domains are `Blake3Hasher` (`TreeHasher` + `TrieHasher`,
  whose associated `type Device: DeviceRootHasher` names the device domains)
  and the device domains are a new `Blake3DeviceHasher`. `OrgTrie<H>` keeps
  its one type parameter.
- **A Vacant proof whose step shows the MemberId held is refused with
  `AbsenceProofMalformed`,** and a Held proof whose MemberId is not the one
  asked about with `AbsenceProofRootMismatch` (it does not resolve to the
  root for that MemberId). T1 amends LLR-25nmwc and LLR-54gp2f.
- **`From<TreeError> for OrgMembersError`** is exhaustive, with no wildcard:
  `HashesNotCalculated` and `HashesAlreadyCalculated` keep their names,
  `ProofMalformed` → `AbsenceProofMalformed`, `ProofRootMismatch` →
  `AbsenceProofRootMismatch`, `ProofLeafHeld` → `DeviceStillHeld`, and
  `Collision`, `LeafNotHeld`, `LeafHeld`, `LevelOutOfRange` →
  `InvariantViolated` (LLR-w5yahk).
- **person's new `IdentityError` variants reach org-members** through
  LLR-hbb4je (minted for this plan), which supersedes LLR-28ekrv:
  `DevicePositionTaken` and `DeviceStillHeld` map to org-members' variants of
  the same names, `PersonHashMismatch`, `DeviceRootMismatch` and
  `ProofMalformed` to `InvariantViolated`. The two tests of LLR-28ekrv in
  `org-members/tests/person_error_mapping.rs` are pointed at LLR-hbb4je and
  listed as inherited.
- **Test fixtures choose device keys by position.** 16 positions and up to 8
  keys per set make collisions between arbitrary fixture keys likely (7/16
  for the eighth key). person's `tests/common::device(seed)` becomes the
  first key, over `blake3(seed, i)` signing seeds, whose 4-bit position is
  `seed % 16`, so seeds 1..=15 never collide; org-members gains
  `common::positioned_device_key(seed, position)`. Measured below.
- **The Quint model gets device positions.** `devicePosition(k) = k.gen`, and
  the conformance driver derives the real key for `{owner, gen}` as the
  first dual-valid key whose 4-bit position is `gen`, so the model's
  positions are the crate's. The model adds `DevicePositionTaken` where
  `DeviceSlots` checks it: after `DuplicateDevice` in `addDevice`, after
  `DeviceSlotsFull` in a record's construction. `GENS` widens to `0.to(8)`
  so a member can reach the bound of 8.

## Measurement: which fixtures collide

Run 2026-10-24 while writing this plan, in the scratch task worktree
`.worktrees/worktree-generic-smt-measure` (discarded, nothing committed):
`person/src/slots.rs` patched to refuse two keys of one 4-bit
`persona::device-position________` position in `parse`, `add_device` and
`Deserialize`, with `MAX_DEVICES` left at 4, then
`CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p <unit> --no-fail-fast` for
person, org-members, org-node (`--features app,test-support`) and org-io
(`--features test-support`).

- **org-node, org-io:** every target green (org-io's chopsticks-only
  `preflight` failed as it does without chopsticks, and is outside
  `verify_commands`). Their fixtures need no change.
- **person:** 14 tests red in `decode_errors`, `definition`,
  `definition_decode`, `device_trie`, `group_key`, `person_device_trie`,
  `person_hash`, `slots`. Cause: the signing seeds `[2; 32]` and `[4; 32]`
  share position 1. Positions of seeds 1..=40 (`[seed; 32]`):
  `1→7 2→1 3→6 4→1 5→15 6→4 7→10 8→9 9→15 10→9 11→3 12→13 13→12 14→11 15→12 16→8 17→7 18→1 19→1 20→8`.
- **org-members:** `fuzz_tests::device_removal_never_keeps_key` (the pool's
  `"<handle>-d-<i>"` keys), `integration_test::{add_p2p_device_rejects_when_full,
  deserialize_rejects_too_many_devices, every_device_slot_reaches_the_root}`,
  and `mbt_conformance::{membership_conformance, scenario_device_slots_full}`
  (the model has no positions).

With the bound at 8 every fixture that fills a set must choose its keys by
position; T7 lists each one.

## Golden values

LLR-377ddr pins the root of a fixed member set; person's tests pin the
device root and the V1 Person hash. This change alters both on purpose (the
device root is a new tree, LLR-8y8tqr, LLR-uudst2; the member leaf hash is
`H::leaf(identity_hash, content_hash)` under new domains, LLR-t465mb,
LLR-4rdqyn; V1 is redefined, LLR-qwp8d7). They are re-pinned, never to make a
red test pass but as the record of the new value, in this way:

1. The task first writes the structural tests that recompute the value from
   `blake3::keyed_hash` alone, with no code of the crate under test on the
   expected side, and brings them green.
2. It then sets the pinned string to the literal `RE-PIN`, runs the pinning
   test, copies the `left:` value of the failed assertion, pastes it, and
   reruns it green.
3. It records old and new value, with the commit, in this plan beside the
   task's `red -> green:` lines, and updates the test's doc comment with the
   date and the LLR that changed it.

The member-set root is re-pinned twice: once in T7 (new device roots under
the old member leaf hash) and once in T9 (new member leaf hash). There is no
second blake3 implementation on this machine, as in
`docs/plans/2026-10-06-person-definition.md`; the structural tests guard the
construction, the pins guard against change.

## Tasks

| Task | Title | After | Parallel with |
|---|---|---|---|
| T1 | Ledger: amendments and the three minted LLRs | — | — |
| T2 | Crate skeleton: `NodeHash`, `TreeError`, traits, discipline | T1 | — |
| T3 | Tree store | T2 | — |
| T4 | `diff` | T3 | — |
| T5 | Proofs | T4 | — |
| T6 | Proof decoding and no-panic properties | T5 | — |
| T7 | Device roots on the tree: positions, bound 8, V1, org-members adaptation, Quint | T6 | — |
| T8 | Absence proofs under a Person hash | T7 | T9 |
| T9 | The Membership record on the tree, and its two-step absence proof | T7 | T8 |
| T10 | Downstream units | T8, T9 | — |
| T11 | SOUP, coverage, CI, configuration; close | T10 | — |

T2–T6 share `sparse-merkle-tree/src/lib.rs` and run serially. T7 touches
person and org-members together because removing `DeviceTrieHasher` breaks
org-members' build in the same commit. T8 touches only `person/`, T9 only
`org-members/`, so they run in parallel.

Every task's commit is unsigned on its task branch
(`git -c commit.gpgsign=false commit`), with no Co-Authored-By line.

### T1 — Ledger: amendments and the three minted LLRs

**Files touched:** `sparse-merkle-tree/docs/architecture/2026-10-24-sparse-merkle-tree.md`, `sparse-merkle-tree/docs/risk/2026-10-23-sparse-merkle-tree.md`, `person/docs/architecture/2026-10-24-device-root-tree.md`, `org-members/docs/architecture/2026-10-24-on-sparse-merkle-tree.md`, `org-members/docs/architecture/2026-10-06-person-variants.md`, `org-members/docs/risk/2026-10-23-two-level-absence-proofs.md`
**Parallel:** no (first)

No code and no test: the items the later tasks' tests name must exist
first. The amended items were all written on this branch and have no test
yet, so they are amended in place. LLR-28ekrv is merged and tested, so it is
superseded.

**Step 1 — sparse-merkle-tree design.** In
`sparse-merkle-tree/docs/architecture/2026-10-24-sparse-merkle-tree.md`:

- LLR-yuyn8f: replace `is computed once per tree type:` with
  `is computed once when an empty tree is made, shared by every tree derived
  from it, and computed again once per proof verification:`.
- LLR-u2r4vj: replace the item text up to `satisfies:` with:
  `` `diff(before, after)` walks both trees, calculated or not, descending
  left before right, skipping every pair of subtrees that are one node or
  whose hashes are both set and equal, and lowering a compact leaf one level
  where it faces an internal node; it returns the identities removed and the
  leaves added or replaced, each in strictly increasing position order. ``
- LLR-77zy97: after `refuses a sibling beyond `DEPTH` before storing it.`
  insert `Its serde form is the tuple (depth, map, siblings, ending) for an
  absence proof and (depth, map, siblings) for an inclusion proof, and
  `Deserialize` refuses a depth other than `DEPTH` before reading the map.`
- Under the store item, after LLR-3zffc2, add the two items minted for this
  plan (`new-id.sh LLR 2` with
  `GR_CONFIG=sparse-merkle-tree/.guardrails/config.yaml`, run 2026-10-24):

  *(Code pruned at merge: 10 lines. Files touched: `sparse-merkle-tree/docs/architecture/2026-10-24-sparse-merkle-tree.md`, `sparse-merkle-tree/docs/risk/2026-10-23-sparse-merkle-tree.md`, `person/docs/architecture/2026-10-24-device-root-tree.md`, `org-members/docs/architecture/2026-10-24-on-sparse-merkle-tree.md`, `org-members/docs/architecture/2026-10-06-person-variants.md`, `org-members/docs/risk/2026-10-23-two-level-absence-proofs.md`.)*

**Step 2 — sparse-merkle-tree risk file.** In
`sparse-merkle-tree/docs/risk/2026-10-23-sparse-merkle-tree.md`, under the
derived-requirements assessment, append:

*(Code pruned at merge: 8 lines. Files touched: `sparse-merkle-tree/docs/architecture/2026-10-24-sparse-merkle-tree.md`, `sparse-merkle-tree/docs/risk/2026-10-23-sparse-merkle-tree.md`, `person/docs/architecture/2026-10-24-device-root-tree.md`, `org-members/docs/architecture/2026-10-24-on-sparse-merkle-tree.md`, `org-members/docs/architecture/2026-10-06-person-variants.md`, `org-members/docs/risk/2026-10-23-two-level-absence-proofs.md`.)*

**Step 3 — person design.** In
`person/docs/architecture/2026-10-24-device-root-tree.md`:

- LLR-8y8tqr: replace its text up to `supersedes:` with:
  `` `device_tree::<H>(&[DevicePublicKey])` builds, through `from_leaves`
  (`sparse-merkle-tree/docs/architecture/2026-10-24-sparse-merkle-tree.md`),
  a calculated `SparseMerkleTree` of depth `DEVICE_ROOT_DEPTH` (4) holding one
  `DeviceLeaf` per key, and refuses with `DevicePositionTaken` when two keys
  share a position; `compute_device_root::<H>(&[DevicePublicKey])` returns
  that tree's root under the same refusal. A `DeviceLeaf`'s identity is its
  DevicePublicKey, its identity hash `H::device_identity` of the key's 32
  bytes, its content hash `H::device_content()`, and its position
  `device_position(key)` (LLR-exwx4n). The keys of a `DeviceSlots`, which
  admits no two keys of one position (LLR-xsx6gg), are never refused. ``
- LLR-qwp8d7: replace
  `` `compute_device_root::<PersonDeviceHasher>(&devices)` (LLR-8y8tqr) `` with
  `` `compute_device_root::<PersonDeviceHasher>(devices.devices())` (LLR-8y8tqr),
  which makes `encode` return a `Result` ``.

**Step 4 — org-members design.** In
`org-members/docs/architecture/2026-10-24-on-sparse-merkle-tree.md`:

- LLR-t465mb: replace `` `MemberLeaf` is the Membership record's leaf of the
  `sparse-merkle-tree` tree at depth 256: its identity `` with `` The
  Membership record's leaf of the `sparse-merkle-tree` tree at depth 256 is a
  `MemberLeaf` held with its device root, which is computed when the leaf
  enters the tree: its identity ``.
- LLR-54gp2f: after `` followed by the absence of `device` under the proof's
  device root; `` insert `` a Held proof whose MemberId is not `id` does not
  resolve to `root` for `id`; ``.
- LLR-25nmwc: replace `and accepts otherwise.` with `` refuses with
  `AbsenceProofMalformed` when a Vacant proof's step shows `id` held, and
  accepts otherwise. ``
- After LLR-w5yahk add the item minted for this plan (`new-id.sh LLR 1` with
  `GR_CONFIG=org-members/.guardrails/config.yaml`, run 2026-10-24):

  *(Code pruned at merge: 15 lines. Files touched: `sparse-merkle-tree/docs/architecture/2026-10-24-sparse-merkle-tree.md`, `sparse-merkle-tree/docs/risk/2026-10-23-sparse-merkle-tree.md`, `person/docs/architecture/2026-10-24-device-root-tree.md`, `org-members/docs/architecture/2026-10-24-on-sparse-merkle-tree.md`, `org-members/docs/architecture/2026-10-06-person-variants.md`, `org-members/docs/risk/2026-10-23-two-level-absence-proofs.md`.)*

- In the opening paragraph's list of superseded items, add `LLR-28ekrv (the
  mapping of person's errors, for person's new variants)`.

**Step 5.** In `org-members/docs/architecture/2026-10-06-person-variants.md`
add, at column one, in LLR-28ekrv's block directly after its `satisfies:`
line: `superseded-by: LLR-hbb4je`.

**Step 6.** In `org-members/docs/risk/2026-10-23-two-level-absence-proofs.md`
append:

*(Code pruned at merge: 5 lines. Files touched: `sparse-merkle-tree/docs/architecture/2026-10-24-sparse-merkle-tree.md`, `sparse-merkle-tree/docs/risk/2026-10-23-sparse-merkle-tree.md`, `person/docs/architecture/2026-10-24-device-root-tree.md`, `org-members/docs/architecture/2026-10-24-on-sparse-merkle-tree.md`, `org-members/docs/architecture/2026-10-06-person-variants.md`, `org-members/docs/risk/2026-10-23-two-level-absence-proofs.md`.)*

**Step 7 — check.** Run, from the change worktree:

*(Code pruned at merge: 3 lines. Files touched: `sparse-merkle-tree/docs/architecture/2026-10-24-sparse-merkle-tree.md`, `sparse-merkle-tree/docs/risk/2026-10-23-sparse-merkle-tree.md`, `person/docs/architecture/2026-10-24-device-root-tree.md`, `org-members/docs/architecture/2026-10-24-on-sparse-merkle-tree.md`, `org-members/docs/architecture/2026-10-06-person-variants.md`, `org-members/docs/risk/2026-10-23-two-level-absence-proofs.md`.)*

Expected: no DANGLING-REF, UNTRACED, MALFORMED-ID, UNANALYZED-DERIVED or
NON-RECIPROCAL-SUPERSESSION. New: `MISSING-TEST LLR-9qncuc`,
`MISSING-TEST LLR-639mz9`, `MISSING-TEST LLR-hbb4je`, and the two
`person_error_mapping.rs` tests reported `OUT-OF-FORCE-VERIFIES` (LLR-28ekrv);
T7 repoints them.

**Step 8 — commit.** `docs: plan-time amendments; mint LLR-9qncuc, LLR-639mz9, LLR-hbb4je (supersedes LLR-28ekrv)`.

### T2 — Crate skeleton: `NodeHash`, `TreeError`, traits, discipline

**Files touched:** `Cargo.toml`, `sparse-merkle-tree/Cargo.toml`, `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/hash.rs`, `sparse-merkle-tree/src/error.rs`, `sparse-merkle-tree/src/leaf.rs`, `sparse-merkle-tree/tests/README.md` (deleted), `sparse-merkle-tree/tests/node_hash.rs`, `sparse-merkle-tree/tests/crate_discipline.rs`, `sparse-merkle-tree/.guardrails/config.yaml`
**Parallel:** no (serial, after T1)

**Step 1 — failing tests.** Write `sparse-merkle-tree/tests/node_hash.rs`:

*(Code pruned at merge: 28 lines. Files touched: `Cargo.toml`, `sparse-merkle-tree/Cargo.toml`, `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/hash.rs`, `sparse-merkle-tree/src/error.rs`, `sparse-merkle-tree/src/leaf.rs`, `sparse-merkle-tree/tests/README.md` (deleted), `sparse-merkle-tree/tests/node_hash.rs`, `sparse-merkle-tree/tests/crate_discipline.rs`, `sparse-merkle-tree/.guardrails/config.yaml`.)*

Write `sparse-merkle-tree/tests/crate_discipline.rs`:

*(Code pruned at merge: 75 lines. Files touched: `Cargo.toml`, `sparse-merkle-tree/Cargo.toml`, `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/hash.rs`, `sparse-merkle-tree/src/error.rs`, `sparse-merkle-tree/src/leaf.rs`, `sparse-merkle-tree/tests/README.md` (deleted), `sparse-merkle-tree/tests/node_hash.rs`, `sparse-merkle-tree/tests/crate_discipline.rs`, `sparse-merkle-tree/.guardrails/config.yaml`.)*

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p sparse-merkle-tree`.
Expected: `error: package ID specification `sparse-merkle-tree` did not
match any packages` (red: no crate).

**Step 2 — the crate.** Add `"sparse-merkle-tree",` to `members` in the
root `Cargo.toml`, after `"person",`. Write `sparse-merkle-tree/Cargo.toml`:

*(Code pruned at merge: 24 lines. Files touched: `Cargo.toml`, `sparse-merkle-tree/Cargo.toml`, `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/hash.rs`, `sparse-merkle-tree/src/error.rs`, `sparse-merkle-tree/src/leaf.rs`, `sparse-merkle-tree/tests/README.md` (deleted), `sparse-merkle-tree/tests/node_hash.rs`, `sparse-merkle-tree/tests/crate_discipline.rs`, `sparse-merkle-tree/.guardrails/config.yaml`.)*

Write `sparse-merkle-tree/src/lib.rs` (T3–T6 extend the module list and the
re-exports):

*(Code pruned at merge: 18 lines. Files touched: `Cargo.toml`, `sparse-merkle-tree/Cargo.toml`, `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/hash.rs`, `sparse-merkle-tree/src/error.rs`, `sparse-merkle-tree/src/leaf.rs`, `sparse-merkle-tree/tests/README.md` (deleted), `sparse-merkle-tree/tests/node_hash.rs`, `sparse-merkle-tree/tests/crate_discipline.rs`, `sparse-merkle-tree/.guardrails/config.yaml`.)*

Write `sparse-merkle-tree/src/hash.rs`:

*(Code pruned at merge: 33 lines. Files touched: `Cargo.toml`, `sparse-merkle-tree/Cargo.toml`, `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/hash.rs`, `sparse-merkle-tree/src/error.rs`, `sparse-merkle-tree/src/leaf.rs`, `sparse-merkle-tree/tests/README.md` (deleted), `sparse-merkle-tree/tests/node_hash.rs`, `sparse-merkle-tree/tests/crate_discipline.rs`, `sparse-merkle-tree/.guardrails/config.yaml`.)*

Write `sparse-merkle-tree/src/error.rs`:

*(Code pruned at merge: 43 lines. Files touched: `Cargo.toml`, `sparse-merkle-tree/Cargo.toml`, `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/hash.rs`, `sparse-merkle-tree/src/error.rs`, `sparse-merkle-tree/src/leaf.rs`, `sparse-merkle-tree/tests/README.md` (deleted), `sparse-merkle-tree/tests/node_hash.rs`, `sparse-merkle-tree/tests/crate_discipline.rs`, `sparse-merkle-tree/.guardrails/config.yaml`.)*

Write `sparse-merkle-tree/src/leaf.rs`:

*(Code pruned at merge: 34 lines. Files touched: `Cargo.toml`, `sparse-merkle-tree/Cargo.toml`, `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/hash.rs`, `sparse-merkle-tree/src/error.rs`, `sparse-merkle-tree/src/leaf.rs`, `sparse-merkle-tree/tests/README.md` (deleted), `sparse-merkle-tree/tests/node_hash.rs`, `sparse-merkle-tree/tests/crate_discipline.rs`, `sparse-merkle-tree/.guardrails/config.yaml`.)*

Delete `sparse-merkle-tree/tests/README.md`.

**Step 3 — the unit's configuration.** In
`sparse-merkle-tree/.guardrails/config.yaml`, replace the `strict_paths`
block and its comment with:

*(Code pruned at merge: 3 lines. Files touched: `Cargo.toml`, `sparse-merkle-tree/Cargo.toml`, `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/hash.rs`, `sparse-merkle-tree/src/error.rs`, `sparse-merkle-tree/src/leaf.rs`, `sparse-merkle-tree/tests/README.md` (deleted), `sparse-merkle-tree/tests/node_hash.rs`, `sparse-merkle-tree/tests/crate_discipline.rs`, `sparse-merkle-tree/.guardrails/config.yaml`.)*

replace the `test_paths` comment with `# The crate's integration tests.`, and
after `test_paths` add:

*(Code pruned at merge: 6 lines. Files touched: `Cargo.toml`, `sparse-merkle-tree/Cargo.toml`, `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/hash.rs`, `sparse-merkle-tree/src/error.rs`, `sparse-merkle-tree/src/leaf.rs`, `sparse-merkle-tree/tests/README.md` (deleted), `sparse-merkle-tree/tests/node_hash.rs`, `sparse-merkle-tree/tests/crate_discipline.rs`, `sparse-merkle-tree/.guardrails/config.yaml`.)*

**Step 4 — green.** Run:

*(Code pruned at merge: 3 lines. Files touched: `Cargo.toml`, `sparse-merkle-tree/Cargo.toml`, `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/hash.rs`, `sparse-merkle-tree/src/error.rs`, `sparse-merkle-tree/src/leaf.rs`, `sparse-merkle-tree/tests/README.md` (deleted), `sparse-merkle-tree/tests/node_hash.rs`, `sparse-merkle-tree/tests/crate_discipline.rs`, `sparse-merkle-tree/.guardrails/config.yaml`.)*

Expected: `crate_discipline` 3 passed, `node_hash` 2 passed (1 without
default features); clippy no warnings. `blake3`, `postcard`, `serde_json` and
`proptest` are already in `Cargo.lock` (person and org-members use them).

**Step 5 — commit.** `feat(sparse-merkle-tree): crate skeleton — NodeHash, TreeError, TreeHasher, Leaf (LLR-jd4phg, LLR-byz55h, LLR-c4eunm)`.

### T3 — Tree store

**Files touched:** `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/bits.rs`, `sparse-merkle-tree/src/defaults.rs`, `sparse-merkle-tree/src/node.rs`, `sparse-merkle-tree/src/tree.rs`, `sparse-merkle-tree/tests/common/mod.rs`, `sparse-merkle-tree/tests/store.rs`, `sparse-merkle-tree/tests/interfaces.rs`
**Parallel:** no (serial, after T2)

**Step 1 — test support.** Write `sparse-merkle-tree/tests/common/mod.rs`:

*(Code pruned at merge: 117 lines. Files touched: `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/bits.rs`, `sparse-merkle-tree/src/defaults.rs`, `sparse-merkle-tree/src/node.rs`, `sparse-merkle-tree/src/tree.rs`, `sparse-merkle-tree/tests/common/mod.rs`, `sparse-merkle-tree/tests/store.rs`, `sparse-merkle-tree/tests/interfaces.rs`.)*

**Step 2 — failing tests.** Write `sparse-merkle-tree/tests/interfaces.rs`:

*(Code pruned at merge: 65 lines. Files touched: `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/bits.rs`, `sparse-merkle-tree/src/defaults.rs`, `sparse-merkle-tree/src/node.rs`, `sparse-merkle-tree/src/tree.rs`, `sparse-merkle-tree/tests/common/mod.rs`, `sparse-merkle-tree/tests/store.rs`, `sparse-merkle-tree/tests/interfaces.rs`.)*

Write `sparse-merkle-tree/tests/store.rs`:

*(Code pruned at merge: 257 lines. Files touched: `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/bits.rs`, `sparse-merkle-tree/src/defaults.rs`, `sparse-merkle-tree/src/node.rs`, `sparse-merkle-tree/src/tree.rs`, `sparse-merkle-tree/tests/common/mod.rs`, `sparse-merkle-tree/tests/store.rs`, `sparse-merkle-tree/tests/interfaces.rs`.)*

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p sparse-merkle-tree --test store --test interfaces`.
Expected: compile errors, `cannot find type SparseMerkleTree in crate
sparse_merkle_tree` (red).

**Step 3 — implementation.** Replace the module list and re-exports of
`sparse-merkle-tree/src/lib.rs` with:

*(Code pruned at merge: 12 lines. Files touched: `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/bits.rs`, `sparse-merkle-tree/src/defaults.rs`, `sparse-merkle-tree/src/node.rs`, `sparse-merkle-tree/src/tree.rs`, `sparse-merkle-tree/tests/common/mod.rs`, `sparse-merkle-tree/tests/store.rs`, `sparse-merkle-tree/tests/interfaces.rs`.)*

Write `sparse-merkle-tree/src/bits.rs`:

*(Code pruned at merge: 19 lines. Files touched: `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/bits.rs`, `sparse-merkle-tree/src/defaults.rs`, `sparse-merkle-tree/src/node.rs`, `sparse-merkle-tree/src/tree.rs`, `sparse-merkle-tree/tests/common/mod.rs`, `sparse-merkle-tree/tests/store.rs`, `sparse-merkle-tree/tests/interfaces.rs`.)*

Write `sparse-merkle-tree/src/defaults.rs`:

*(Code pruned at merge: 41 lines. Files touched: `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/bits.rs`, `sparse-merkle-tree/src/defaults.rs`, `sparse-merkle-tree/src/node.rs`, `sparse-merkle-tree/src/tree.rs`, `sparse-merkle-tree/tests/common/mod.rs`, `sparse-merkle-tree/tests/store.rs`, `sparse-merkle-tree/tests/interfaces.rs`.)*

Write `sparse-merkle-tree/src/node.rs`:

*(Code pruned at merge: 72 lines. Files touched: `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/bits.rs`, `sparse-merkle-tree/src/defaults.rs`, `sparse-merkle-tree/src/node.rs`, `sparse-merkle-tree/src/tree.rs`, `sparse-merkle-tree/tests/common/mod.rs`, `sparse-merkle-tree/tests/store.rs`, `sparse-merkle-tree/tests/interfaces.rs`.)*

Write `sparse-merkle-tree/src/tree.rs`:

*(Code pruned at merge: 286 lines. Files touched: `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/bits.rs`, `sparse-merkle-tree/src/defaults.rs`, `sparse-merkle-tree/src/node.rs`, `sparse-merkle-tree/src/tree.rs`, `sparse-merkle-tree/tests/common/mod.rs`, `sparse-merkle-tree/tests/store.rs`, `sparse-merkle-tree/tests/interfaces.rs`.)*

**Step 4 — green.** Run:

*(Code pruned at merge: 2 lines. Files touched: `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/bits.rs`, `sparse-merkle-tree/src/defaults.rs`, `sparse-merkle-tree/src/node.rs`, `sparse-merkle-tree/src/tree.rs`, `sparse-merkle-tree/tests/common/mod.rs`, `sparse-merkle-tree/tests/store.rs`, `sparse-merkle-tree/tests/interfaces.rs`.)*

Expected: `store` 13 passed, `interfaces` 2 passed, doctests 2 passed (one
`compile_fail`), earlier targets unchanged; clippy clean. If the
`compile_fail` doctest passes for a reason other than the depth (run
`cargo test -p sparse-merkle-tree --doc -- --nocapture` and read the
compiler output it records), fix the doctest until the only error is the
`IN_RANGE` assertion: `a sparse Merkle tree's depth is 1 to 256`.

**Step 5 — commit.** `feat(sparse-merkle-tree): tree store — compact nodes, insert, remove, calculate, from_leaves, reads (SDD-rh9u6z)`.

### T4 — `diff`

**Files touched:** `sparse-merkle-tree/src/tree.rs`, `sparse-merkle-tree/tests/diff.rs`
**Parallel:** no (serial, after T3)

**Step 1 — failing tests.** Write `sparse-merkle-tree/tests/diff.rs`:

*(Code pruned at merge: 97 lines. Files touched: `sparse-merkle-tree/src/tree.rs`, `sparse-merkle-tree/tests/diff.rs`.)*

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p sparse-merkle-tree --test diff`.
Expected: `no function or associated item named diff` (red).

**Step 2 — implementation.** Append to `sparse-merkle-tree/src/tree.rs`,
before the `Default` impl:

*(Code pruned at merge: 80 lines. Files touched: `sparse-merkle-tree/src/tree.rs`, `sparse-merkle-tree/tests/diff.rs`.)*

**Step 3 — green.** Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p sparse-merkle-tree`
and the clippy entry. Expected: `diff` 4 passed; the rest unchanged; clippy clean.

**Step 4 — commit.** `feat(sparse-merkle-tree): diff walk over calculated and uncalculated trees (LLR-u2r4vj)`.

### T5 — Proofs

**Files touched:** `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/proof.rs`, `sparse-merkle-tree/src/tree.rs`, `sparse-merkle-tree/tests/proofs.rs`
**Parallel:** no (serial, after T4)

**Step 1 — failing tests.** Write `sparse-merkle-tree/tests/proofs.rs`:

*(Code pruned at merge: 219 lines. Files touched: `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/proof.rs`, `sparse-merkle-tree/src/tree.rs`, `sparse-merkle-tree/tests/proofs.rs`.)*

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p sparse-merkle-tree --test proofs`.
Expected: `unresolved imports sparse_merkle_tree::AbsenceProof` (red).

**Step 2 — implementation.** In `sparse-merkle-tree/src/lib.rs`, add
`pub mod proof;` after `mod node;` and
`pub use proof::{AbsenceProof, InclusionProof, ProofEnding};` after the
`leaf` re-export.

Write `sparse-merkle-tree/src/proof.rs`:

*(Code pruned at merge: 164 lines. Files touched: `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/proof.rs`, `sparse-merkle-tree/src/tree.rs`, `sparse-merkle-tree/tests/proofs.rs`.)*

In `sparse-merkle-tree/src/tree.rs`, add
`use crate::proof::{AbsenceProof, InclusionProof, Path, ProofEnding};` to the
imports, and add to the first `impl` block, after `leaves`:

*(Code pruned at merge: 80 lines. Files touched: `sparse-merkle-tree/src/lib.rs`, `sparse-merkle-tree/src/proof.rs`, `sparse-merkle-tree/src/tree.rs`, `sparse-merkle-tree/tests/proofs.rs`.)*

**Step 3 — green.** Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p sparse-merkle-tree`
and the clippy entry. Expected: `proofs` 10 passed; the rest unchanged.

**Step 4 — commit.** `feat(sparse-merkle-tree): absence and inclusion proofs (SDD-ur6uvu)`.

### T6 — Proof decoding and no-panic properties

**Files touched:** `sparse-merkle-tree/src/proof.rs`, `sparse-merkle-tree/tests/proof_decoding.rs`, `sparse-merkle-tree/tests/no_panic.rs`
**Parallel:** no (serial, after T5)

**Step 1 — failing tests.** Write `sparse-merkle-tree/tests/proof_decoding.rs`:

*(Code pruned at merge: 91 lines. Files touched: `sparse-merkle-tree/src/proof.rs`, `sparse-merkle-tree/tests/proof_decoding.rs`, `sparse-merkle-tree/tests/no_panic.rs`.)*

The depth is a `u16`; postcard writes `8` and `4` as one varint byte, which
the second test reads at `bytes[0]`.

Write `sparse-merkle-tree/tests/no_panic.rs`:

*(Code pruned at merge: 90 lines. Files touched: `sparse-merkle-tree/src/proof.rs`, `sparse-merkle-tree/tests/proof_decoding.rs`, `sparse-merkle-tree/tests/no_panic.rs`.)*

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p sparse-merkle-tree --test proof_decoding --test no_panic`.
Expected: `the trait bound AbsenceProof<8>: Serialize is not satisfied` (red).

**Step 2 — implementation.** Append to `sparse-merkle-tree/src/proof.rs`:

*(Code pruned at merge: 138 lines. Files touched: `sparse-merkle-tree/src/proof.rs`, `sparse-merkle-tree/tests/proof_decoding.rs`, `sparse-merkle-tree/tests/no_panic.rs`.)*

`depth::<DEPTH>()`'s `unwrap_or` cannot take its fallback: `Path::parse`
and `SparseMerkleTree::new`, the only ways to a proof, assert
`DEPTH <= 256` at compile time.

**Step 3 — green.** Run the three `verify_commands` of the unit. Expected:
`proof_decoding` 4 passed, `no_panic` 2 passed (the decoding property is
skipped without default features); every earlier target unchanged; clippy
clean.

**Step 4 — commit.** `feat(sparse-merkle-tree): proof serde form with depth, bounded decoding; no-panic properties (LLR-77zy97, LLR-smf4zq)`.

### T7 — Device roots on the tree: positions, bound 8, V1, org-members adaptation, Quint

**Files touched:** `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`
**Parallel:** no (serial, after T6)

One task because the pieces cannot be green apart: `DeviceTrieHasher`'s
removal breaks org-members' build, the position check breaks fixtures in
both units, and the bound of 8 breaks the Quint conformance until the model
has it too. The member leaf hash is untouched here (T9).

**Step 1 — person fixtures.** In `person/tests/common/mod.rs`, replace
`device` with:

*(Code pruned at merge: 25 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

In `person/tests/slots.rs`, `person/tests/slots_decode.rs`,
`person/tests/decode_errors.rs` and `person/tests/definition_decode.rs`,
add `mod common;` if absent, delete the file's own `device`, `device_bytes`
and `sorted_device_bytes` functions, and `use common::{...}` the ones it
called. `person/tests/device_key.rs` keeps its own `device`: it builds no
set of two keys.

**Step 2 — person failing tests.** Delete `person/tests/device_trie.rs` and
`person/tests/person_device_trie.rs`; every test in them verifies an item
superseded or retired by this change (LLR-6ezhw7, LLR-4vsm8d, LLR-edn55h,
LLR-z99qee). Their behaviours are re-tested below against the successors;
`NodeHash` is `sparse-merkle-tree`'s (T2, LLR-jd4phg).

Write `person/tests/device_root.rs`:

*(Code pruned at merge: 166 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

Write `person/tests/person_device_root.rs`:

*(Code pruned at merge: 95 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

Replace `person/tests/slots.rs` whole:

*(Code pruned at merge: 24 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

Continue the file:

*(Code pruned at merge: 105 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

In `person/tests/slots_decode.rs`:
- `decoding_rejects_a_set_over_the_bound` and
  `decoding_a_huge_declared_length_stops_at_the_bound`: use the nine seeds
  `1..=9` in place of five, `1 + 9 * 32` in place of `1 + 5 * 32`, and say
  "ninth" where the doc comment says "fifth".
- `decoding_a_non_sequence_names_the_expected_form`: expect
  `"at most 8 device keys in strictly increasing order"`.
- Add:

*(Code pruned at merge: 10 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

In `person/tests/decode_errors.rs`, `rejected_slots_name_the_bound_or_the_order`:
`five` becomes `nine` over seeds `[1, 2, 3, 4, 5, 6, 7, 8, 9]`; `four`
becomes `eight` over `[1, 2, 3, 4, 5, 6, 7, 8]` with
`assert_eq!(slots.device_count(), 8)`; and before the identity-point case
add:

*(Code pruned at merge: 3 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

and add `LLR-pjf9qn` to its `verifies:` line.

In `person/tests/definition.rs`: `a_person_holds_up_to_max_devices_sorted`
asserts `MAX_DEVICES == 8` and gives `(1..=8).rev().map(device).collect()`
(expect message `"eight"`); `over_the_bound_or_a_duplicate_is_refused_before_any_person_exists`
uses `(1..=9)` for the over-bound list and `slots(&[1, 2, 3, 4, 5, 6, 7, 8])`
with `add_device(device(9))` for the full set, and its doc comment says
"A ninth key".

In `person/tests/definition_decode.rs`, the over-bound case uses
`sorted_device_bytes(&[1, 2, 3, 4, 5, 6, 7, 8, 9])`.

In `person/tests/crate_discipline.rs`, `every_rejection_is_a_distinct_named_variant`
lists, after `IdentityError::UnsupportedEncodingVersion(2),`:
`IdentityError::DevicePositionTaken, IdentityError::DeviceStillHeld,
IdentityError::PersonHashMismatch, IdentityError::DeviceRootMismatch,
IdentityError::ProofMalformed,`.

In `person/tests/no_panic.rs`: `satisfies_the_person_rules` gains

*(Code pruned at merge: 3 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

and returns `devices.len() <= MAX_DEVICES && sorted && key_rule && positions_distinct`;
`decoding_any_bytes_does_not_panic` adds the same distinctness assertion on
a decoded `DeviceSlots`; the `seeds` ranges `0..6` become `0..10` so the
bound of 8 is reached. Add `LLR-xsx6gg` to the `verifies:` line of
`person_new_on_any_fields` and `LLR-pjf9qn` to the two decoding properties'.

In `person/tests/person_hash.rs`:
- every `compute_device_root::<PersonDeviceHasher>(&slots(X))` becomes
  `compute_device_root::<PersonDeviceHasher>(slots(X).devices()).expect("distinct positions")`;
- every `encode(&P, EncodingVersion::V1)` becomes
  `encode(&P, EncodingVersion::V1).expect("encodes")`;
- every `verifies:` naming `LLR-edn55h` names `LLR-qwp8d7` instead;
- `the_v1_hash_is_pinned`: both expected strings become `"RE-PIN"`, and its
  doc comment reads: `Pinned values: see "Golden values" in
  docs/plans/2026-10-24-sparse-merkle-tree.md (T7). Re-pinned 2026-10-24
  when V1 was redefined over the depth-4 device root (LLR-qwp8d7, owner
  ruling 2026-10-23); a later change here is a change of the V1 encoding,
  which needs a new version (REQ-wg7z4s).`

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p person`. Expected:
`unresolved import person::device_position` and the like (red).

**Step 3 — person implementation.** In `person/Cargo.toml` add, under
`[dependencies]`:

*(Code pruned at merge: 1 line. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

and make the feature `serde = ["dep:serde", "sparse-merkle-tree/serde"]`.

Delete `person/src/device_trie.rs`. Write `person/src/device_root.rs`:

*(Code pruned at merge: 92 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

Replace `person/src/device_hasher.rs` whole:

*(Code pruned at merge: 53 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

In `person/src/slots.rs`:
- `MAX_DEVICES` becomes `8`, its doc comment `Maximum number of devices,
  held below the device root's 16 positions (owner ruling 2026-10-23).
  LLR-xsx6gg.`; the type's doc names LLR-xsx6gg.
- add `use crate::device_root::same_device_position;`.
- in `parse`, after the duplicate check:

*(Code pruned at merge: 7 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

- add, after `device_count`:

*(Code pruned at merge: 5 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

- in `add_device`, after the duplicate check:

*(Code pruned at merge: 3 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

- delete `to_fixed_slots`.
- in the `Deserialize` visitor: `expecting` writes
  `"at most {MAX_DEVICES} device keys in strictly increasing order, at distinct device positions"`,
  and after the order check:

*(Code pruned at merge: 3 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

In `person/src/error.rs`: `DeviceSlotsFull`'s message becomes
`"device slots full (max 8)"`, and after `DeviceNotFound` add:

*(Code pruned at merge: 22 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

In `person/src/hash.rs`:
- imports: replace `use crate::device_trie::compute_device_root;` with
  `use crate::device_root::compute_device_root;`, and add
  `use crate::name::{Name, Surname};`, `use crate::person_key::PersonPublicKey;`,
  `use sparse_merkle_tree::NodeHash;` (keep `MAX_NAME_LEN`, `MAX_SURNAME_LEN`).
- replace `encode`, `encode_v1` and `person_hash` with:

*(Code pruned at merge: 51 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

In `person/src/encoding.rs`, the `V1` doc comment names LLR-qwp8d7 instead
of LLR-edn55h.

In `person/src/lib.rs`: the crate doc says "device slots and the device
root" for "device slots and the device sub-trie"; replace
`pub mod device_trie;` with `pub mod device_root;`; replace
`pub use device_trie::{compute_device_root, DeviceTrieHasher, NodeHash};` with:

*(Code pruned at merge: 5 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p person`. Expected: all
green except `person_device_root::the_person_device_root_is_pinned` and
`person_hash::the_v1_hash_is_pinned`, red on `RE-PIN`. Follow "Golden
values": paste each `left:` value, rerun green, and record the four
values (two device roots, two Person hashes) with their old values
(`db36a284…`, `6b8a4eea…`, `d6177f1c…`, `225bc8ac…`) in this plan.

**Step 4 — org-members fixtures and failing tests.** In
`org-members/tests/common/mod.rs` add:

*(Code pruned at merge: 30 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

In `org-members/tests/person_error_mapping.rs`: every `// verifies: LLR-28ekrv`
becomes `// verifies: LLR-hbb4je`; the module doc names LLR-hbb4je and
"the eight variants only person's Person operations and Person-hash proofs
produce"; `person_only_variants` adds `IdentityError::PersonHashMismatch`,
`IdentityError::DeviceRootMismatch`, `IdentityError::ProofMalformed`; the
`expected` list of the shared variants adds
`(IdentityError::DevicePositionTaken, OrgMembersError::DevicePositionTaken)`
and `(IdentityError::DeviceStillHeld, OrgMembersError::DeviceStillHeld)`.
Both tests are inherited (record:
`inherited: person_only_variants_map_to_invariant_violated — from LLR-28ekrv`,
`inherited: shared_variants_keep_their_names_and_fields — from LLR-28ekrv`).

In `org-members/tests/integration_test.rs`:
- imports: `use common::{device_key, free_position_keys, member_key, positioned_device_key};`,
  `use org_members::hasher::{Blake3DeviceHasher, Blake3Hasher, TrieHasher};`,
  and `use person::DeviceRootHasher;` in place of `use person::DeviceTrieHasher;`.
- `the_device_empty_sentinel_encodes_no_device_public_key`:
  `<Blake3DeviceHasher as DeviceRootHasher>::DEVICE_EMPTY_SENTINEL`.
- `hasher_domains_are_separated` (verified LLR-72p8bz, out of force):
  replace whole with

*(Code pruned at merge: 24 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

- `device_slot_order_does_not_change_the_root`: `d1` and `d2` become
  `positioned_device_key("dev-a", 1)` and `positioned_device_key("dev-b", 2)`;
  its annotation becomes `/// verifies: LLR-uudst2` (inherited: from
  LLR-kdhd2v).
- `every_device_slot_reaches_the_root` (verified LLR-kdhd2v, LLR-pys2ek):
  replace whole with

*(Code pruned at merge: 23 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

- `add_p2p_device_rejects_when_full`: annotation
  `/// verifies: REQ-xdx2c2, LLR-utk6uy, LLR-4phmjf`; doc comment "The bound
  itself (MAX_DEVICES, 8 today) is design data"; body:

*(Code pruned at merge: 10 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

- `member_leaf_too_many_devices`: annotation `/// verifies: LLR-utk6uy`;
  `devices` is `(0..9).map(|p| positioned_device_key(&format!("d{p}"), p)).collect()`.
- `deserialize_rejects_too_many_devices`: annotation
  `/// verifies: REQ-shk82j, REQ-xdx2c2, LLR-utk6uy, LLR-xyv6p9`; `many` is
  `(0..9).map(|p| positioned_device_key(&format!("d{p}"), p)).collect()`.
- `add_p2p_device_reports_earlier_refusals_before_shared_key`: the loop over
  `["alice-d2", "alice-d3", "alice-d4"]` becomes
  `for key in free_position_keys("alice-fill", &[device_key("alice-d1")], 7)`
  adding `key`.
- add, after `add_p2p_device_rejects_when_full`:

*(Code pruned at merge: 22 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

In `org-members/tests/newtypes.rs`, `slot_device` becomes
`common::positioned_device_key(&format!("slot-{seed}"), seed)`.

In `org-members/tests/fuzz_tests.rs`: `DEVICE_POOL` becomes `10`
(doc: "Wider than MAX_DEVICES so some ops name a device the member does not
hold, or overflow the slots"); `pool_device` becomes
`positioned_device_key(&format!("{}-d-{}", HANDLES[member_idx], device_idx), device_idx as u8)`;
import `positioned_device_key` from `common`.

In `org-members/tests/encoding_golden.rs`, `member_set_root_is_pinned`'s
expected string becomes `"RE-PIN"` (re-pinned in Step 7) and the module doc
gains: `Re-derived 2026-10-24 (plan docs/plans/2026-10-24-sparse-merkle-tree.md,
T7): the device root became a depth-4 sparse Merkle tree under four device
domains (LLR-uudst2, LLR-4rdqyn); the wire bytes are unchanged.`

**Step 5 — the Quint model and its driver.** In
`org-members/quint/membership.qnt`:
- `/// Crate constant: members cap at 8 devices.` and `pure val MAX_DEVICES = 8`.
- after `MAX_DEVICES` add:

*(Code pruned at merge: 12 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

- `addMember`: after the `DeviceSlotsFull` branch insert
  `else if (not(positionsDistinct(l.devices))) Err("DevicePositionTaken")`,
  and its doc's check order reads `(leaf construction: EmptyDeviceList,
  DeviceSlotsFull, DevicePositionTaken)`.
- `leafDeviceError`: after the `DeviceSlotsFull` branch insert
  `else if (not(positionsDistinct(l.devices))) "DevicePositionTaken"`.
- `addDevice`: after the `DuplicateDevice` branch insert
  `else if (positionTaken(s.get(id).devices, d)) Err("DevicePositionTaken")`;
  its doc's check order reads `IdNotFound, DeviceSlotsFull, DuplicateDevice,
  DevicePositionTaken, DuplicateKey`.
- `applyDelta`: after the `DeviceSlotsFull` branch insert
  `else if (afterUpsert.keys().exists(k => not(positionsDistinct(afterUpsert.get(k).devices)))) Err("DevicePositionTaken")`,
  and its doc says this branch, like the cap's, is unreachable from the
  crate, whose member records cannot hold two keys of one position by type.
- `deviceCapOk` stays; add `pure def positionsOk(s: Snapshot): bool = s.keys().forall(k => positionsDistinct(s.get(k).devices))`.
- `fullA` holds eight devices at distinct positions:
  `Set({ owner: "a", gen: 1 }, { owner: "a", gen: 2 }, { owner: "a", gen: 3 }, { owner: "a", gen: 4 }, { owner: "a", gen: 5 }, { owner: "a", gen: 6 }, { owner: "c", gen: 7 }, { owner: "c", gen: 8 })`.
- add, after `addDeviceFullBeforeDuplicateKeyTest`:

*(Code pruned at merge: 10 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

Before writing these two tests, read `twoMembers` (line 388): they assume
member `a` holds `{ owner: "a", gen: 1 }` and member `b` holds
`{ owner: "b", gen: 1 }`. If `twoMembers` differs, choose the generations
so the first key is unheld at a's held position and the second is b's held
device at a's held position, and say so in the test's doc.

In `org-members/quint/membership_mbt.qnt`:
- `/// Nine generations, so a member can reach the device cap (MAX_DEVICES = 8)
  with distinct positions.` and `pure val GENS = 0.to(8)`.
- in `init`, `flaw == 3` gives `devs: 9`.
- `scenarioDeviceSlotsFull`: `initWith(TWO).then(doAddDevice("a", key("a", 2)))`
  continued with `.then(doAddDevice("a", key("a", g)))` for `g` = 3, 4, 5,
  6, 7, then `.then(doAddDevice("a", key("c", 0)))` (eight devices,
  positions 0..7), then `.then(doAddDevice("a", key("a", 8)))`.
- `scenarioDeviceKeyMoves` moves `key("b", 2)` instead of `key("b", 1)`: a's
  built record holds `Set(key("a", 1), key("b", 2))`, b's
  `.with("pKey", key("b", 3)).with("devices", Set(key("b", 1)))`, and its
  comment reads "A device key moves from b to a: the record
  delete_p2p_device(b, (b,2), (b,3)) then add_p2p_device(a, (b,2))
  reaches" (positions 1 and 2 for a, distinct).
- add, after `scenarioAddDeviceDuplicate`:
  `run scenarioDevicePositionTaken = initWith(TWO).then(doAddDevice("a", key("c", 1)))`.
- `mbtInv` adds `positionsOk(trie),`.

In `org-members/tests/mbt_conformance.rs`:
- `GENS` becomes `[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]` with the doc "Key
  generations the model draws (0..=8) plus 9, which only a refused
  nine-device genesis record uses (devices 1..=9), so every key the driver
  builds inverts."
- `real_key_bytes` becomes

*(Code pruned at merge: 6 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

  with `positioned_key_bytes` imported from `common`.
- `err_tag` maps `OrgMembersError::DevicePositionTaken => "DevicePositionTaken"`.
- add:

*(Code pruned at merge: 7 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

- `scenario_device_slots_full` gains `/// verifies: LLR-utk6uy, REQ-xdx2c2`.

**Step 6 — org-members implementation.** In `org-members/Cargo.toml` add
`sparse-merkle-tree = { path = "../sparse-merkle-tree", default-features = false }`
under `[dependencies]` and `"sparse-merkle-tree/serde"` to the `serde`
feature.

Replace `org-members/src/hasher.rs` whole:

*(Code pruned at merge: 72 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

In `org-members/src/smt.rs`, `insert`'s device root line becomes
`let device_root = compute_device_root::<H::Device>(member.p2p_devices())?;`.
In `org-members/src/proof.rs`, `verify`'s becomes
`let device_root = compute_device_root::<H::Device>(member.p2p_devices())?;`.

In `org-members/src/error.rs`: `DeviceSlotsFull`'s message becomes
`"device slots full (max 8)"`; after `DeviceSlotsFull` add

*(Code pruned at merge: 4 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

and the `From<person::IdentityError>` impl (doc: `LLR-hbb4je.`) adds the arms
`I::DevicePositionTaken => Self::DevicePositionTaken,` and
`I::DeviceStillHeld => Self::DeviceStillHeld,` after `I::DeviceNotFound`, and
`| I::PersonHashMismatch | I::DeviceRootMismatch | I::ProofMalformed` to the
`InvariantViolated` arm, whose comment reads "Only person's Person
operations and Person-hash proofs produce these, and org-members calls none
of them (LLR-hbb4je)."

In `org-members/src/trie.rs`, `add_p2p_device`'s doc check order reads
`IdNotFound, DeviceSlotsFull, DuplicateDevice, DevicePositionTaken
(LLR-5jh6t3), DuplicateKey`.

**Step 7 — green and the first member-root re-pin.** Run:

*(Code pruned at merge: 7 lines. Files touched: `person/Cargo.toml`, `person/src/lib.rs`, `person/src/device_trie.rs` (deleted), `person/src/device_root.rs`, `person/src/device_hasher.rs`, `person/src/slots.rs`, `person/src/error.rs`, `person/src/hash.rs`, `person/src/encoding.rs`, `person/tests/common/mod.rs`, `person/tests/device_trie.rs` (deleted), `person/tests/person_device_trie.rs` (deleted), `person/tests/device_root.rs`, `person/tests/person_device_root.rs`, `person/tests/slots.rs`, `person/tests/slots_decode.rs`, `person/tests/decode_errors.rs`, `person/tests/definition.rs`, `person/tests/definition_decode.rs`, `person/tests/crate_discipline.rs`, `person/tests/person_hash.rs`, `person/tests/no_panic.rs`, `org-members/Cargo.toml`, `org-members/src/hasher.rs`, `org-members/src/error.rs`, `org-members/src/smt.rs`, `org-members/src/proof.rs`, `org-members/src/trie.rs`, `org-members/tests/common/mod.rs`, `org-members/tests/person_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/newtypes.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`, `org-members/tests/mbt_conformance.rs`, `org-members/quint/membership.qnt`, `org-members/quint/membership_mbt.qnt`.)*

Expected: person green (pins pasted in Step 3); org-members green except
`encoding_golden::member_set_root_is_pinned` on `RE-PIN`: paste its `left:`
value per "Golden values", record old (`4740435a…efd4`) and new, rerun
green. The structural evidence that only the device root moved is
`every_device_reaches_the_root` and `device_root.rs`'s slot oracle; the wire
bytes test `member_record_wire_bytes_are_pinned` stays green unchanged. If
`membership_conformance` reports a step whose crate error is
`DevicePositionTaken` and whose model error differs, the model's check
order is wrong at that step: compare with the crate's order in Step 5 and fix
the model, not the driver.

Check-trace: `person` and `org-members` report no OUT-OF-FORCE-VERIFIES
for `device_trie.rs`, `person_device_trie.rs`, `person_hash.rs`,
`integration_test.rs:{1308,2437,2460,2494}`, `person_error_mapping.rs`
(the remaining ones are T8's and T9's).

**Step 8 — commit.** `feat(person, org-members): device roots on sparse-merkle-tree at depth 4; device positions; bound 8; V1 over the new root; Quint positions`.

### T8 — Absence proofs under a Person hash

**Files touched:** `person/src/absence.rs`, `person/src/lib.rs`, `person/tests/person_absence_proofs.rs`
**Parallel:** yes (with T9; serial after T7)

**Step 1 — failing tests.** Write `person/tests/person_absence_proofs.rs`:

*(Code pruned at merge: 165 lines. Files touched: `person/src/absence.rs`, `person/src/lib.rs`, `person/tests/person_absence_proofs.rs`.)*

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p person --test person_absence_proofs`.
Expected: `unresolved imports person::prove_device_absent, person::PersonAbsenceProof` (red).

**Step 2 — implementation.** In `person/src/lib.rs` add `pub mod absence;`
(first in the module list) and
`pub use absence::{prove_device_absent, PersonAbsenceProof};` (first in the
re-exports); the crate doc's last sentence ends "…the Person hash, and the
proof that a device key is not held under one."

Write `person/src/absence.rs`:

*(Code pruned at merge: 158 lines. Files touched: `person/src/absence.rs`, `person/src/lib.rs`, `person/tests/person_absence_proofs.rs`.)*

**Step 3 — green.** Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p person`
and `cargo clippy -p person --all-targets -- -D warnings`. Expected: 8
passed in the new target; the rest unchanged; clippy clean. Then
`GR_CONFIG=person/.guardrails/config.yaml sh .guardrails/scripts/check-trace.sh`:
no MISSING-TEST and no OUT-OF-FORCE-VERIFIES left in person.

**Step 4 — commit.** `feat(person): absence proofs under a Person hash (SDD-bx9mm6)`.

### T9 — The Membership record on the tree, and its two-step absence proof

**Files touched:** `org-members/Cargo.toml`, `org-members/src/lib.rs`, `org-members/src/hasher.rs`, `org-members/src/member_tree.rs`, `org-members/src/smt.rs` (deleted), `org-members/src/node.rs` (deleted), `org-members/src/trie.rs`, `org-members/src/delta.rs`, `org-members/src/types.rs`, `org-members/src/proof.rs`, `org-members/src/error.rs`, `org-members/docs/architecture/soup.md`, `org-members/tests/common/mod.rs`, `org-members/tests/absence_proofs.rs`, `org-members/tests/tree_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`
**Parallel:** yes (with T8; serial after T7)

**Step 1 — test support.** In `org-members/tests/common/mod.rs`, delete
`default_level_count` and add:

*(Code pruned at merge: 27 lines. Files touched: `org-members/Cargo.toml`, `org-members/src/lib.rs`, `org-members/src/hasher.rs`, `org-members/src/member_tree.rs`, `org-members/src/smt.rs` (deleted), `org-members/src/node.rs` (deleted), `org-members/src/trie.rs`, `org-members/src/delta.rs`, `org-members/src/types.rs`, `org-members/src/proof.rs`, `org-members/src/error.rs`, `org-members/docs/architecture/soup.md`, `org-members/tests/common/mod.rs`, `org-members/tests/absence_proofs.rs`, `org-members/tests/tree_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`.)*

**Step 2 — failing tests.** Write `org-members/tests/tree_error_mapping.rs`:

*(Code pruned at merge: 24 lines. Files touched: `org-members/Cargo.toml`, `org-members/src/lib.rs`, `org-members/src/hasher.rs`, `org-members/src/member_tree.rs`, `org-members/src/smt.rs` (deleted), `org-members/src/node.rs` (deleted), `org-members/src/trie.rs`, `org-members/src/delta.rs`, `org-members/src/types.rs`, `org-members/src/proof.rs`, `org-members/src/error.rs`, `org-members/docs/architecture/soup.md`, `org-members/tests/common/mod.rs`, `org-members/tests/absence_proofs.rs`, `org-members/tests/tree_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`.)*

Replace `org-members/tests/absence_proofs.rs` whole (every test in it
verified an item this change supersedes; the in-force items it also
verified — REQ-yyuxh8, REQ-ds8ryr, RC-qa2758 — are verified again below):

*(Code pruned at merge: 376 lines. Files touched: `org-members/Cargo.toml`, `org-members/src/lib.rs`, `org-members/src/hasher.rs`, `org-members/src/member_tree.rs`, `org-members/src/smt.rs` (deleted), `org-members/src/node.rs` (deleted), `org-members/src/trie.rs`, `org-members/src/delta.rs`, `org-members/src/types.rs`, `org-members/src/proof.rs`, `org-members/src/error.rs`, `org-members/docs/architecture/soup.md`, `org-members/tests/common/mod.rs`, `org-members/tests/absence_proofs.rs`, `org-members/tests/tree_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`.)*

In `org-members/tests/integration_test.rs`:
- `hasher_domains_are_separated` (T7's) becomes the eight domains:

*(Code pruned at merge: 42 lines. Files touched: `org-members/Cargo.toml`, `org-members/src/lib.rs`, `org-members/src/hasher.rs`, `org-members/src/member_tree.rs`, `org-members/src/smt.rs` (deleted), `org-members/src/node.rs` (deleted), `org-members/src/trie.rs`, `org-members/src/delta.rs`, `org-members/src/types.rs`, `org-members/src/proof.rs`, `org-members/src/error.rs`, `org-members/docs/architecture/soup.md`, `org-members/tests/common/mod.rs`, `org-members/tests/absence_proofs.rs`, `org-members/tests/tree_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`.)*

- add, after it:

*(Code pruned at merge: 29 lines. Files touched: `org-members/Cargo.toml`, `org-members/src/lib.rs`, `org-members/src/hasher.rs`, `org-members/src/member_tree.rs`, `org-members/src/smt.rs` (deleted), `org-members/src/node.rs` (deleted), `org-members/src/trie.rs`, `org-members/src/delta.rs`, `org-members/src/types.rs`, `org-members/src/proof.rs`, `org-members/src/error.rs`, `org-members/docs/architecture/soup.md`, `org-members/tests/common/mod.rs`, `org-members/tests/absence_proofs.rs`, `org-members/tests/tree_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`.)*

- `member_id_bit_indexes_msb_first` (used `MemberId::bit`, removed with
  LLR-7jkcba's retirement) is replaced by:

*(Code pruned at merge: 19 lines. Files touched: `org-members/Cargo.toml`, `org-members/src/lib.rs`, `org-members/src/hasher.rs`, `org-members/src/member_tree.rs`, `org-members/src/smt.rs` (deleted), `org-members/src/node.rs` (deleted), `org-members/src/trie.rs`, `org-members/src/delta.rs`, `org-members/src/types.rs`, `org-members/src/proof.rs`, `org-members/src/error.rs`, `org-members/docs/architecture/soup.md`, `org-members/tests/common/mod.rs`, `org-members/tests/absence_proofs.rs`, `org-members/tests/tree_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`.)*

- `member_id_bit_out_of_range_does_not_panic` is replaced by:

*(Code pruned at merge: 20 lines. Files touched: `org-members/Cargo.toml`, `org-members/src/lib.rs`, `org-members/src/hasher.rs`, `org-members/src/member_tree.rs`, `org-members/src/smt.rs` (deleted), `org-members/src/node.rs` (deleted), `org-members/src/trie.rs`, `org-members/src/delta.rs`, `org-members/src/types.rs`, `org-members/src/proof.rs`, `org-members/src/error.rs`, `org-members/docs/architecture/soup.md`, `org-members/tests/common/mod.rs`, `org-members/tests/absence_proofs.rs`, `org-members/tests/tree_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`.)*

- delete `default_hashes_at_level_out_of_range_does_not_panic` and
  `out_of_range_path_index_is_index_out_of_range` (both verified
  LLR-7jkcba, retired; the bound on levels is `sparse-merkle-tree`'s,
  verified there by LLR-yuyn8f's test).
- imports: `use org_members::hasher::{Blake3DeviceHasher, Blake3Hasher, TrieHasher};`
  stays; delete any remaining `org_members::smt` path.

In `org-members/tests/fuzz_tests.rs`:
- `arb_wire_proof` is replaced by one building real proofs and corrupting
  their bytes:

*(Code pruned at merge: 22 lines. Files touched: `org-members/Cargo.toml`, `org-members/src/lib.rs`, `org-members/src/hasher.rs`, `org-members/src/member_tree.rs`, `org-members/src/smt.rs` (deleted), `org-members/src/node.rs` (deleted), `org-members/src/trie.rs`, `org-members/src/delta.rs`, `org-members/src/types.rs`, `org-members/src/proof.rs`, `org-members/src/error.rs`, `org-members/docs/architecture/soup.md`, `org-members/tests/common/mod.rs`, `org-members/tests/absence_proofs.rs`, `org-members/tests/tree_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`.)*

- `absence_proof_decoding_and_verify_never_panic`: `/// verifies: REQ-ds8ryr, LLR-9g2kjg, LLR-v3cysh`.
- `every_honest_absence_proof_verifies` (line 1108, verified only superseded
  items): annotation `/// verifies: LLR-bh53rr, LLR-54gp2f, REQ-sb2frw, REQ-65xrgh`
  (inherited: from LLR-utp6x4, LLR-dgzy7e, LLR-4xz255), and after the Vacant
  check add the Held case:

*(Code pruned at merge: 4 lines. Files touched: `org-members/Cargo.toml`, `org-members/src/lib.rs`, `org-members/src/hasher.rs`, `org-members/src/member_tree.rs`, `org-members/src/smt.rs` (deleted), `org-members/src/node.rs` (deleted), `org-members/src/trie.rs`, `org-members/src/delta.rs`, `org-members/src/types.rs`, `org-members/src/proof.rs`, `org-members/src/error.rs`, `org-members/docs/architecture/soup.md`, `org-members/tests/common/mod.rs`, `org-members/tests/absence_proofs.rs`, `org-members/tests/tree_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`.)*

  with `let members_ids: Vec<MemberId> = members.iter().map(|m| *m.id()).collect();`
  taken before `members` is moved into `genesis`.

In `org-members/tests/encoding_golden.rs`, `member_set_root_is_pinned`'s
expected string becomes `"RE-PIN"` again, and the module doc gains:
`Re-derived again 2026-10-24 (same plan, T9): the member leaf hash became
H::leaf(identity hash, content hash) under the member identity and content
domains (LLR-t465mb, LLR-4rdqyn); a_member_leaf_hashes_as_documented
(integration_test.rs) recomputes a one-member root from blake3 alone.`

Run `QUINT_HOME=<scratch>/quint_home CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-members`.
Expected: compile errors (`no method named member_identity`,
`no method named vacancy`, `unresolved import sparse_merkle_tree::ProofEnding`
in this crate's dev scope) — red.

**Step 3 — implementation.** In `org-members/Cargo.toml` delete the `spin`
dependency and its comment.

In `org-members/docs/architecture/soup.md`, delete the `spin` row and add,
below the table: `**2026-10-24:** `spin` removed: the store's write-once
cells are `sparse-merkle-tree`'s (its SOUP inventory lists `spin` 0.9.9),
plan `docs/plans/2026-10-24-sparse-merkle-tree.md`, T9.`

Replace `org-members/src/hasher.rs` whole:

*(Code pruned at merge: 89 lines. Files touched: `org-members/Cargo.toml`, `org-members/src/lib.rs`, `org-members/src/hasher.rs`, `org-members/src/member_tree.rs`, `org-members/src/smt.rs` (deleted), `org-members/src/node.rs` (deleted), `org-members/src/trie.rs`, `org-members/src/delta.rs`, `org-members/src/types.rs`, `org-members/src/proof.rs`, `org-members/src/error.rs`, `org-members/docs/architecture/soup.md`, `org-members/tests/common/mod.rs`, `org-members/tests/absence_proofs.rs`, `org-members/tests/tree_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`.)*

Write `org-members/src/member_tree.rs`:

*(Code pruned at merge: 112 lines. Files touched: `org-members/Cargo.toml`, `org-members/src/lib.rs`, `org-members/src/hasher.rs`, `org-members/src/member_tree.rs`, `org-members/src/smt.rs` (deleted), `org-members/src/node.rs` (deleted), `org-members/src/trie.rs`, `org-members/src/delta.rs`, `org-members/src/types.rs`, `org-members/src/proof.rs`, `org-members/src/error.rs`, `org-members/docs/architecture/soup.md`, `org-members/tests/common/mod.rs`, `org-members/tests/absence_proofs.rs`, `org-members/tests/tree_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`.)*

The `as u32` casts are lossless: `Handle`, `Name` and `Surname` are each
bounded at 128 bytes.

Delete `org-members/src/smt.rs` and `org-members/src/node.rs`. In
`org-members/src/lib.rs`, remove `pub mod node;` and `pub mod smt;`, add
`pub mod member_tree;`, and make the re-exports
`pub use member_tree::MEMBER_DEPTH;` and
`pub use proof::{AbsenceProof, ProvenMember};` in place of
`pub use proof::{AbsenceProof, ProofEnding};`.

In `org-members/src/types.rs`, delete `MemberId::bit` and
`MemberId::path_bits` (LLR-7jkcba retired; the tree reads positions) and
`MemberLeaf::canonical_bytes` (its bytes are `member_content_hash`'s, without
the MemberId); `MemberId`'s doc says "Used as the tree's position (its 256
bits, most significant first; LLR-zbe553)".

In `org-members/src/error.rs`: delete `IndexOutOfRange` (only `bit` and
`at_level` returned it); add, after the `From<person::IdentityError>` impl:

*(Code pruned at merge: 17 lines. Files touched: `org-members/Cargo.toml`, `org-members/src/lib.rs`, `org-members/src/hasher.rs`, `org-members/src/member_tree.rs`, `org-members/src/smt.rs` (deleted), `org-members/src/node.rs` (deleted), `org-members/src/trie.rs`, `org-members/src/delta.rs`, `org-members/src/types.rs`, `org-members/src/proof.rs`, `org-members/src/error.rs`, `org-members/docs/architecture/soup.md`, `org-members/tests/common/mod.rs`, `org-members/tests/absence_proofs.rs`, `org-members/tests/tree_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`.)*

and the docs of `DeviceStillHeld`, `AbsenceProofMalformed` and
`AbsenceProofRootMismatch` name LLR-bh53rr/LLR-25nmwc, LLR-9g2kjg and
LLR-54gp2f.

In `org-members/src/trie.rs` and `org-members/src/delta.rs`:
- imports: drop `crate::node::Node` and `crate::smt::{self, DefaultHashes}`;
  add `use crate::member_tree::{calculated, diff, member_of, MemberTree, TreeMember};`
  and, in trie.rs, `use person::device_tree;` and
  `use crate::proof::{AbsenceProof, ProvenMember};`.
- `OrgTrie` fields: `root: Arc<Node>` → `tree: MemberTree<H>`;
  `last_calculated_root: Arc<Node>` → `last_calculated: MemberTree<H>`;
  delete `defaults`. `CandidateTrie`: `root` → `tree: MemberTree<H>`, delete
  `defaults`. `from_candidate` takes `tree: MemberTree<H>` in place of `root`
  and `defaults`, and sets `last_calculated: tree.clone()`;
  `CandidateTrie::verify_against` (delta.rs) passes `self.tree` in their
  place.
- every `smt::get_member(&self.root, X)` → `member_of(&self.tree, X)`; every
  `smt::get_member(&root, X)` (the local in `apply_delta`) →
  `member_of(&root, X)`.
- `smt::empty_root(&defaults)?` → `MemberTree::<H>::new()`.
- every `smt::insert::<H>(&R, M, &D)?` → `R.insert(TreeMember::new::<H>(M)?)?`.
- every `smt::remove(&R, ID, &D)?` → `R.remove(ID)?`.
- `genesis`: after the loop, `let tree = calculated(tree)?; let root_hash = tree.root()?;`
  and the struct takes `tree: tree.clone()`, `last_calculated: tree`.
- `apply_delta`: `let root = calculated(root)?; let root_hash = root.root()?;`
  and `CandidateTrie { tree: root, … }`.
- `recalculate`: `let tree = calculated(self.tree.clone())?; let root_hash = tree.root()?;`
  then `tree: tree.clone(), last_calculated: tree`.
- `pending_changes`: `let (removed, upserted) = diff(&self.last_calculated, &self.tree);`
  and `base_root: self.last_calculated.root()?.into()`; delete the
  invariant comment about the base hash (`root()` refuses an unset one).
- `calculate_delta`: `diff(&old.tree, &self.tree)`.
- `members`: `self.tree.leaves().into_iter().map(|held| held.member().clone()).collect()`.
- `contains`, `get`, `get_by_handle`: through `member_of(&self.tree, …)`.
- every `defaults: self.defaults.clone(),` line is deleted; every
  `root: …` / `last_calculated_root: …` field initialiser is renamed.
- `prove_absent` becomes:

*(Code pruned at merge: 18 lines. Files touched: `org-members/Cargo.toml`, `org-members/src/lib.rs`, `org-members/src/hasher.rs`, `org-members/src/member_tree.rs`, `org-members/src/smt.rs` (deleted), `org-members/src/node.rs` (deleted), `org-members/src/trie.rs`, `org-members/src/delta.rs`, `org-members/src/types.rs`, `org-members/src/proof.rs`, `org-members/src/error.rs`, `org-members/docs/architecture/soup.md`, `org-members/tests/common/mod.rs`, `org-members/tests/absence_proofs.rs`, `org-members/tests/tree_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`.)*

- `Debug` for `CandidateTrie` and `OrgTrie` are unchanged.

Replace `org-members/src/proof.rs` whole:

*(Code pruned at merge: 216 lines. Files touched: `org-members/Cargo.toml`, `org-members/src/lib.rs`, `org-members/src/hasher.rs`, `org-members/src/member_tree.rs`, `org-members/src/smt.rs` (deleted), `org-members/src/node.rs` (deleted), `org-members/src/trie.rs`, `org-members/src/delta.rs`, `org-members/src/types.rs`, `org-members/src/proof.rs`, `org-members/src/error.rs`, `org-members/docs/architecture/soup.md`, `org-members/tests/common/mod.rs`, `org-members/tests/absence_proofs.rs`, `org-members/tests/tree_error_mapping.rs`, `org-members/tests/integration_test.rs`, `org-members/tests/fuzz_tests.rs`, `org-members/tests/encoding_golden.rs`.)*

**Step 4 — green and the second member-root re-pin.** Run the org-members
`verify_commands` (`QUINT_HOME` set). Expected: all green except
`encoding_golden::member_set_root_is_pinned` on `RE-PIN`; paste its `left:`
value per "Golden values", record the T7 value and the new one, rerun green.
`a_member_leaf_hashes_as_documented` is the structural evidence for the new
value. The tests of the store's in-force items — LLR-zbe553, LLR-wm5hpc,
LLR-tk4qxu, LLR-n7nya3, LLR-4n8zqx, LLR-8jttpb, LLR-j35sxz — pass unchanged
through the new store; any that does not is a defect of this task, not a
test to edit.

Then `GR_CONFIG=org-members/.guardrails/config.yaml sh .guardrails/scripts/check-trace.sh`:
no MISSING-TEST, no OUT-OF-FORCE-VERIFIES, no DANGLING-REF.

**Step 5 — commit.** `feat(org-members): Membership record on sparse-merkle-tree; two-step absence proofs (Vacant / Held); spin dropped`.

### T10 — Downstream units

**Files touched:** none expected. A red that needs a change is fixed in the
file it names, and that file is added to this list, with the reason, before
the task is marked done.
`org-node/tests/encoding_golden.rs` — `revocation_wire_message_is_pinned`
was red (`MalformedMessage` decoding `GOLDEN_REVOCATION`): the notice's
proof changed form, as Step 2 expects. Re-pinned 2026-10-25 by the format
rule, never captured from the code: the identity bytes are unchanged and the
34-byte proof `ff`x32 ‖ `00` ‖ `00` becomes the 38-byte `00` (Vacant) ‖
`8002` (depth 256) ‖ `20` ‖ `00`x32 (map: no non-empty sibling) ‖ `00` (no
siblings) ‖ `00` (Empty); the body length 119 becomes 123. Old and new value
are in the file's comment above `GOLDEN_REVOCATION`.
`app/src-tauri/Cargo.lock` — the app is its own Cargo workspace, and its
verify command rewrote the lock for T7–T9's path dependencies: org-members
and person now depend on `sparse-merkle-tree` (which brings `spin` 0.9.9 and
`serde`), and org-members no longer on `spin` directly. No version changed.
**Parallel:** no (serial, after T8 and T9)

Checked while writing this plan: no code in `org-node`, `org-io` or `app`
names `ProofEnding`, `DefaultHashes`, `org_members::smt`,
`org_members::node`, `MemberId::bit`, `IndexOutOfRange`, `DeviceTrieHasher`,
`compute_device_root` or `MAX_DEVICES`, and none matches `OrgMembersError`
exhaustively (two sites construct `DuplicateKey`). org-node calls
`OrgTrie::prove_absent(&id, &device)` and
`AbsenceProof::verify::<Blake3Hasher>(&root, &id, &device)`, whose
signatures T9 keeps. The measurement above found no position collision in
their fixtures. `org-node/quint/protocol.qnt` imports
`membership_types.qnt`, which this change does not touch.

**Step 1 — impact.** Run `sh .guardrails/scripts/check-units.sh --impact`
from the change worktree. Expected: org-members, person and
sparse-merkle-tree changed; org-node, org-io and app impacted.

**Step 2 — run each impacted unit's `verify_commands`, verbatim from its
config** (`org-node/.guardrails/config.yaml`, `org-io/.guardrails/config.yaml`,
`app/.guardrails/config.yaml`), with `CARGO_HOME=/tmp/cargo_home_fuzz`.
Expected: green, no file changed. A revocation notice's wire bytes change
(the proof's form changed), so a red in a test pinning a `RevocationNotice`
encoding is re-pinned per "Golden values", the reason recorded here. Any
other red is a defect of T7–T9: fix it there (in a task worktree for that
task's files), not here.

**Step 3 — check-trace for the downstream units.** Run `check-trace.sh` with
each of their configs. Expected: no finding that names an item of this
change; their pre-existing UNMET-EXPECTATION and UNRESOLVED-PR findings are
unchanged.

**Step 4 — commit** only if a file changed:
`test(<unit>): <what was re-pinned or fixed and why>`.

### T11 — SOUP, coverage, CI, configuration; close

**Files touched:** `sparse-merkle-tree/docs/architecture/soup.md`, `sparse-merkle-tree/.guardrails/config.yaml`, `Makefile`, `.github/workflows/rust.yml`, `docs/plans/2026-10-24-sparse-merkle-tree.md`
**Parallel:** no (serial, after T10)

**Step 1 — SOUP as built.** Diff the inventory against the manifest and the
lock file:

*(Code pruned at merge: 3 lines. Files touched: `sparse-merkle-tree/docs/architecture/soup.md`, `sparse-merkle-tree/.guardrails/config.yaml`, `Makefile`, `.github/workflows/rust.yml`, `docs/plans/2026-10-24-sparse-merkle-tree.md`.)*

Expected: `spin` 0.9.9 and `serde` 1.0.229, as the inventory says. In
`sparse-merkle-tree/docs/architecture/soup.md` replace the "As designed on
2026-10-24" paragraph with: `**As built on <date>** (plan
docs/plans/2026-10-24-sparse-merkle-tree.md, T2 and T6): diffed against
`sparse-merkle-tree/Cargo.toml` and the workspace `Cargo.lock`; the
versions below are the locked ones. The dev-dependencies (`blake3`,
`postcard`, `serde_json`, `proptest`) are test tooling and ship in no build.`
If either version differs, correct the row to the locked version and say so
in the same paragraph.

**Step 2 — coverage.** Run once to measure:

*(Code pruned at merge: 3 lines. Files touched: `sparse-merkle-tree/docs/architecture/soup.md`, `sparse-merkle-tree/.guardrails/config.yaml`, `Makefile`, `.github/workflows/rust.yml`, `docs/plans/2026-10-24-sparse-merkle-tree.md`.)*

(the `CARGO_TARGET_DIR` is the Makefile's `BRANCH_TARGET_DIR`; use its
value). Add to the `Makefile`, following person's targets:

- beside `PERSON_LINES`, the measured figures as a comment in the form of
  person's, then `SPARSE_MERKLE_TREE_LINES` and
  `SPARSE_MERKLE_TREE_REGIONS` at one point below the measurement, rounded
  down (the rule org-io's first floors used);
- beside `PERSON_BRANCHES`, the branch figure and
  `SPARSE_MERKLE_TREE_BRANCHES` by the same rule;
- `SPARSE_MERKLE_TREE_COVERAGE_ARGS := -p sparse-merkle-tree` with the
  comment "Every target, none excluded, as person's.";
- targets `coverage-sparse-merkle-tree` and
  `coverage-branch-sparse-merkle-tree`, copies of `coverage-person` and
  `coverage-branch-person` with the variable names and report name changed;
- both added to `coverage`, `coverage-branch` and `.PHONY`.

A region or branch the tests do not reach is either covered by a test added
here (annotated with the LLR whose behaviour it is), or named, with the
reason no test reaches it, in the floor's comment — as org-io's comment does
for its preflight branches. The candidates are the three by construction:
`Path::from_siblings`' `if let Some(byte)` else, `depth::<DEPTH>()`'s
fallback, and `DefaultHashes::at`'s refusal inside the store.

Run `make coverage-person coverage-branch-person coverage-org-members coverage-branch-org-members`.
Expected: each at or above its floor. A fall below a floor is closed by a
test, never by lowering the floor (the floors ratchet upward only).

**Step 3 — the unit's configuration.** In
`sparse-merkle-tree/.guardrails/config.yaml`, after `verify_commands` add:

*(Code pruned at merge: 6 lines. Files touched: `sparse-merkle-tree/docs/architecture/soup.md`, `sparse-merkle-tree/.guardrails/config.yaml`, `Makefile`, `.github/workflows/rust.yml`, `docs/plans/2026-10-24-sparse-merkle-tree.md`.)*

**Step 4 — CI.** In `.github/workflows/rust.yml`, beside each person line:
- the test job: `- run: cargo test -p sparse-merkle-tree` and
  `- run: cargo test -p sparse-merkle-tree --no-default-features`, with the
  comment "sparse-merkle-tree's verify_commands test entries.";
- the msrv job: `- run: cargo test -p sparse-merkle-tree`;
- the clippy job: `- run: cargo clippy -p sparse-merkle-tree --all-targets -- -D warnings`;
- the wasm32 job: `- run: cargo check -p sparse-merkle-tree --target wasm32-unknown-unknown`
  and the same with `--no-default-features`;
- the comment above the coverage job names `coverage-sparse-merkle-tree`.

**Step 5 — close.** Run, from the change worktree, and paste each output
into this plan under "Verification record" below:

*(Code pruned at merge: 4 lines. Files touched: `sparse-merkle-tree/docs/architecture/soup.md`, `sparse-merkle-tree/.guardrails/config.yaml`, `Makefile`, `.github/workflows/rust.yml`, `docs/plans/2026-10-24-sparse-merkle-tree.md`.)*

Expected: exit 0 for each; no MISSING-TEST, OUT-OF-FORCE-VERIFIES,
DANGLING-REF, UNTRACED, UNANALYZED-DERIVED or NON-RECIPROCAL finding in the
three units. Then run every `verify_commands` entry of the three units once
more, all green.

**Step 6 — commit.** `chore(sparse-merkle-tree): SOUP as built, coverage floors, CI lanes; close the plan`.

## Self-review (plan-change step 9)

1. **Every ID in Implements has a test.** sparse-merkle-tree: LLR-jd4phg
   (T2 node_hash), LLR-byz55h and LLR-c4eunm (T2 crate_discipline),
   LLR-znpbf5, LLR-t3e8pp, LLR-9z6dc9 (T3 interfaces), LLR-ggu9bh,
   LLR-mm45ry, LLR-yuyn8f, LLR-kep2ju, LLR-8uem8c, LLR-ycf27m, LLR-3y5pq6,
   LLR-3zffc2, LLR-9qncuc, LLR-639mz9 (T3 store), LLR-u2r4vj (T4),
   LLR-m5c3vw, LLR-tz3nfz, LLR-5tww69, LLR-88rm99, LLR-77zy97, LLR-53ezgd,
   LLR-c7bdee, LLR-smf4zq (T5, T6); the eight REQs through them and directly
   in T5/T6. person: LLR-exwx4n, LLR-8y8tqr, LLR-9mjfjn (T7 device_root),
   LLR-qwp8d7 (T7 person_device_root, person_hash), LLR-xsx6gg, LLR-vraz7n
   (T7 slots), LLR-pjf9qn (T7 slots_decode, decode_errors), LLR-985v9t,
   LLR-cve3d6, LLR-z8h2w8, LLR-3dvfjb (T8); REQ-9sh8qb, REQ-4e23xr,
   REQ-5kvqcf, REQ-uh2bzg through them. org-members: LLR-hbb4je (T7
   person_error_mapping), LLR-uudst2, LLR-utk6uy, LLR-5jh6t3 (T7
   integration, mbt), LLR-4rdqyn, LLR-t465mb, LLR-w5yahk (T9), LLR-v6smww,
   LLR-bh53rr, LLR-9g2kjg, LLR-54gp2f, LLR-25nmwc, LLR-v3cysh (T9
   absence_proofs, fuzz); REQ-sb2frw, REQ-65xrgh, REQ-2sz7j8 through them
   and directly. The SDD items are covered through their LLRs.
2. **Code steps show real code, commands and expected output.** Each task
   gives the full text of every new file and the exact edit of every
   changed one; the only values not written here are the re-pinned golden
   hashes and the coverage floors, both produced by a stated procedure and
   recorded in this file when produced.
3. **Names and signatures across tasks.** `SparseMerkleTree::{new,
   from_leaves, root_of, insert, remove, calculate, root, is_calculated,
   get, contains, leaves, empty_hash, prove_absent, prove_included, diff}`
   (T3–T5) are what T7–T9 call; `AbsenceProof<D>::{from_parts, map,
   siblings, ending, verify::<L, H>}` and `InclusionProof<D>::{from_parts,
   map, siblings, verify::<L, H>}` (T5) likewise; person's `device_tree`,
   `compute_device_root` (slices, `Result<_, IdentityError>`),
   `device_position`, `DeviceLeaf`, `DeviceRootHasher`,
   `DEVICE_ROOT_DEPTH` (T7) are what T8 and T9 call; org-members'
   `TrieHasher::{member_identity, member_content}` and `type Device` (T9)
   replace T7's `hash_member_leaf`/`hash_member_node`, and T9 updates
   every T7 test that named those.
4. **Files and parallelism.** T8's files are all under `person/`, T9's all
   under `org-members/`: disjoint, so they run in parallel. Every other
   pair runs serially. No two parallel tasks name the same file.

## Verification record

(Filled in as tasks complete: each task's `red -> green:` lines from its
dispatch report, the `inherited:` lines, the re-pinned golden values with
their old values, the coverage figures, and the close's outputs.)

Inherited, known at planning:

```text
inherited: person_only_variants_map_to_invariant_violated — from LLR-28ekrv
inherited: shared_variants_keep_their_names_and_fields — from LLR-28ekrv
inherited: device_slot_order_does_not_change_the_root — from LLR-kdhd2v
inherited: every_honest_absence_proof_verifies — from LLR-utp6x4, LLR-dgzy7e, LLR-4xz255
```

### Progress

- **T1 done** (task commit eef8c80, merged 2026-10-25). Docs only: no test,
  no `red -> green:` line. Step 7 check-trace: only the expected new
  findings (MISSING-TEST LLR-9qncuc, LLR-639mz9, LLR-hbb4je;
  OUT-OF-FORCE-VERIFIES person_error_mapping.rs:13, :34). The opening
  paragraph of the org-members design file lists its superseded items in
  semicolon-separated groups; the meaning is unchanged.
- **T2 done** (task commit a45f0fb5, merged 2026-10-25). 5 passed (4
  without default features); clippy clean. The plan's code was used as
  written. Cargo.lock gains only the sparse-merkle-tree entry; the
  workspace needed `cargo` without `--offline` once, to fetch crates absent
  from /tmp/cargo_home_fuzz.

```text
red -> green: a_node_hash_wraps_any_32_bytes — failed before the crate existed (package ID specification `sparse-merkle-tree` did not match any packages)
red -> green: a_node_hash_encodes_as_its_32_bytes — same run, no crate
red -> green: the_crate_is_no_std_and_denies_indexing — same run, no crate
red -> green: the_source_names_no_std_item_and_no_io — same run, no crate
red -> green: the_only_dependencies_are_spin_and_optional_serde — same run, no crate
```
- **T3 done** (task commit cdb9b41a, merged 2026-10-25). store 13, interfaces
  2, doctests 2 (the `compile_fail` one fails only on E0080 "a sparse Merkle
  tree's depth is 1 to 256"); clippy clean; plan code as written. Note:
  `cargo fmt --check` reports diffs in the new crate; formatting is not a
  gate in this repository.

```text
red -> green: a_depth_one_root_is_built_from_the_three_hasher_functions — compile red, E0432 unresolved import sparse_merkle_tree::SparseMerkleTree
red -> green: only_the_identity_hash_and_content_hash_enter_the_tree — same
red -> green: depths_one_and_256_build — same
red -> green: positions_are_the_first_depth_bits_most_significant_first — same
red -> green: the_empty_subtree_hashes_and_their_bound — same
red -> green: the_root_is_the_classic_trees_at_depth_8 — same
red -> green: the_root_is_the_classic_trees_at_depth_256 — same
red -> green: removal_gives_the_classic_tree_of_what_remains — same
red -> green: insert_and_remove_leave_the_receiver_unchanged — same
red -> green: insert_refuses_a_collision_and_replaces_the_same_identity — same
red -> green: remove_refuses_an_identity_not_held — same
red -> green: hashes_are_write_once_and_the_root_waits_for_them — same
red -> green: from_leaves_builds_a_calculated_tree_or_refuses_a_collision — same
red -> green: get_contains_and_leaves_read_without_changing_the_tree — same
red -> green: the_root_depends_on_the_leaf_set_alone — same
```
- **T4 done** (task commit f74a30e, merged 2026-10-25). diff 4 passed; full
  suite green with and without default features; clippy clean; plan code as
  written.

```text
red -> green: diff_reports_removed_added_and_replaced_in_position_order — compile red, E0599 no associated function `diff`
red -> green: equal_trees_differ_in_nothing — same
red -> green: diff_walks_uncalculated_trees_and_lowers_compact_leaves — same
red -> green: diff_matches_the_set_difference — same
```
- **T5 done** (task commit e325bd5, merged 2026-10-25). Crate 36 passed with
  and without default features; clippy clean; plan code as written.
  Deviation in the red-first record: `every_refusal_is_its_own_variant_and_message`
  checks `TreeError`'s `PartialEq` and `Display`, written in T2 ahead of
  this test (the plan placed the enum in the skeleton); it was red only
  because its file did not compile. LLR-smf4zq's no-panic half is shown
  red-first in T6.

```text
red -> green: an_absence_proof_holds_a_map_its_siblings_and_an_ending — compile red, unresolved AbsenceProof/ProofEnding, no method prove_absent
red -> green: prove_absent_ends_by_what_the_position_holds — same
red -> green: prove_absent_refuses_a_held_leaf_and_an_uncalculated_tree — compile red, no method prove_absent
red -> green: prove_included_returns_the_leaf_and_a_proof_that_verifies — compile red, no method prove_included
red -> green: prove_included_refuses_an_absent_leaf_and_an_uncalculated_tree — same
red -> green: an_inclusion_proof_refuses_any_other_leaf_or_root — same
red -> green: verify_absent_checks_the_root_first_then_the_ending — compile red, unresolved AbsenceProof
red -> green: from_parts_refuses_a_map_or_siblings_that_do_not_fit_the_depth — compile red, unresolved AbsenceProof/InclusionProof
red -> green: every_refusal_is_its_own_variant_and_message — compile red of its file only; the behaviour (T2's TreeError) existed before it
red -> green: proofs_verify_at_depth_256_at_the_edges — compile red, no method prove_absent/prove_included
```
- **T6 done** (task commit f2ced38, merged 2026-10-25). Crate 40 passed (36
  without default features); clippy clean in both. Deviation: no_panic.rs's
  serde-only imports are gated `#[cfg(feature = "serde")]` (unused-import
  warnings otherwise). `any_operation_sequence_does_not_panic` was not red:
  it exercises T3–T5 code already merged, and is recorded as a property over
  that code, not red-first evidence for T6.

```text
red -> green: proofs_round_trip_and_still_verify — compile red, Serialize/Deserialize not implemented for AbsenceProof<8>/InclusionProof<8>
red -> green: a_proof_parses_at_its_own_depth_only — same, for depth 4
red -> green: decoding_refuses_malformed_and_oversized_parts — same
red -> green: a_huge_declared_length_reserves_nothing — same
red -> green: decoding_and_verifying_any_bytes_does_not_panic — same, for AbsenceProof<4>/<256>, InclusionProof<256>
not red: any_operation_sequence_does_not_panic — passed on first run (T3–T5 code)
```
- **T7 done** (task commit 45183b21, merged 2026-10-25). person and
  org-members green (mbt_conformance 33 passed, 1 ignored as before;
  integration 167; fuzz 11); clippy clean; quint typecheck both files,
  `quint test` 65 passing, `quint run … --invariant=mbtInv` no violation.
  Deviations:
  1. Quint: two existing tests broke under positions = gen and were
     adjusted as the plan adjusted `scenarioDeviceKeyMoves`:
     `addDeviceOtherMemberDeviceTest` gives b the device (b,2) and adds that
     (still DuplicateKey); `applyDeltaDeviceKeyMovesTest` moves (b,2), b
     rotating to (b,3).
  2. person/tests/device_root.rs: a comment naming org-members' LLR-4rdqyn
     was UNDECLARED-DEPENDENCY (person cannot cite org-members); reworded to
     "as that unit's design states them".
  3. person no_panic.rs: the slot-decoding properties' ranges widened to
     reach the bound of 8 (`slot_count`, `length` and `keys` to 0..10), and
     LLR-pjf9qn plus the position-distinctness assertion added to
     `decoding_any_bytes_does_not_panic` and
     `decoding_slot_shaped_bytes_does_not_panic`.
  4. Speed: positioned key derivation took mbt_conformance to 1133 s and
     fuzz_tests to 90 s; a per-process memo (`OnceLock<Mutex<HashMap>>`) in
     `mbt_conformance::real_key_bytes` and `fuzz_tests::pool_device` brings
     them to about 13 s and 7–11 s.
  5. definition_decode.rs drops the now-unused `device` import.
     `MemberLeaf::p2p_device_slots` is no longer called by smt.rs or
     proof.rs (still by trie.rs); no dead-code warning.

Re-pinned golden values (T7, commit 45183b21):

| Value | Old | New |
|---|---|---|
| Person device root, no devices | `db36a2843de361cd8fa691c95f305e206fbcdc666e0f1d1a5e494610df4421a5` | `a01d79a58bbd2f5c628513a1c31a338d9f84a623a608dd2979be451f433fdac0` |
| Person device root, devices 1, 2 | `6b8a4eea74c7daf62a0e6b87112f24ad1037bbfdc5e8607eaa2e217bc8e9d4a5` | `1adf410eb24c2d605b0fff5c81928567158a9100611ce790c565e744b64f4f15` |
| V1 Person hash, Alice Example, key 9, devices 1, 2 | `d6177f1cbed18504b63666dd45b5f54834667606fa73de40c2dbaf28a704e292` | `0d8019c330b4279c02cf747b911a9cb6f1357dbb164bbd9183265e90ca951c8a` |
| V1 Person hash, Alice Example, no key, no device | `225bc8ac0117ae0b3c4aaaec7654b81b3aa45a8f27a58eab7f365dd6fc8183f0` | `943411327947df5cd3f96b581d5c466cb0c8202ff00d72516fda45e4bbdd8bc3` |
| member-set root (GOLDEN_ROOT), first re-pin | `4740435a055ef3825a1a96a9a72361c3bc22bd2c47a992cda21161048d17efd4` | `b0c01649b9f0c8169c5d47b9c0177d37d44f56e04ab5bfab4192cfbee70909d7` |

The device keys of the person fixtures changed too (T7 Step 1), so the
person pins move for two reasons at once: the new tree and the new keys.
`member_record_wire_bytes_are_pinned` stayed green unchanged.

```text
red -> green: person tests of T7 (device_root.rs, person_device_root.rs, slots.rs, decoding_refuses_two_keys_of_one_position, amended decode_errors/definition/no_panic/person_hash/crate_discipline) — compile red before the person implementation: E0432 unresolved person::device_position, device_tree, DeviceRootHasher, DEVICE_POSITION_KEY, DEVICE_ROOT_DEPTH; E0599 no method position_taken; no variant IdentityError::DevicePositionTaken; E0599 .expect on Vec<u8>/NodeHash
red -> green: the_person_device_root_is_pinned — red on "RE-PIN", then pasted
red -> green: the_v1_hash_is_pinned — red on "RE-PIN", then pasted
red -> green: org-members tests of T7 (hasher_domains_are_separated, every_device_reaches_the_root, a_device_at_a_taken_position_is_refused, scenario_device_position_taken, person_error_mapping additions) — red before Step 6 as a library compile failure (E0432 person::DeviceTrieHasher, src/hasher.rs:1), not on their own assertions
red -> green: member_set_root_is_pinned — red on "RE-PIN", then pasted
not red: Quint addDevicePositionTakenTest, addDevicePositionBeforeDuplicateKeyTest, scenarioDevicePositionTaken — written with the model change in one edit
inherited: person_only_variants_map_to_invariant_violated — from LLR-28ekrv
inherited: shared_variants_keep_their_names_and_fields — from LLR-28ekrv
inherited: device_slot_order_does_not_change_the_root — from LLR-kdhd2v
inherited: member_leaf_too_many_devices — from LLR-pys2ek
inherited: deserialize_rejects_too_many_devices — from LLR-pys2ek
inherited: add_p2p_device_rejects_when_full — from LLR-pys2ek
inherited: person/tests/slots.rs tests carried over (add_keeps_the_set_sorted_wherever_the_key_falls, has_device_is_membership_in_the_set, try_from_goes_through_parse, debug_is_the_count_and_names_no_key) — from LLR-9p34cv
```
- **T8 done** (task commit 15c9a3d, merged 2026-10-25). person_absence_proofs
  9 passed (7 tests + 2 properties; the plan's "8" was a miscount); person
  green; clippy clean with and without default features; person
  check-trace: no MISSING-TEST, no OUT-OF-FORCE-VERIFIES. Deviation: the
  test file is gated `#![cfg(feature = "serde")]`, as the other
  serde-reliant person test files are.

```text
red -> green: a_proof_holds_the_fields_and_no_device_public_key — compile red, unresolved person::prove_device_absent, person::PersonAbsenceProof
red -> green: a_proof_for_a_key_not_held_verifies_against_the_person_hash — same
red -> green: no_proof_is_made_for_a_key_the_person_holds — same
red -> green: verify_checks_the_person_hash_first — same
red -> green: verify_then_checks_the_device_root_and_the_ending — same
red -> green: a_proof_survives_the_wire_and_malformed_ones_are_refused — same
red -> green: decoding_and_verifying_any_bytes_does_not_panic — same
red -> green: every_proof_made_verifies — same
red -> green: the_proof_refusals_are_distinct_named_variants — compile red of its file only; the variants it checks were added in T7
```
- **T9 done** (task commit 7af855bd, merged 2026-10-25). org-members 264
  passed, 1 ignored (pre-existing); clippy clean; the five quint entries
  exit 0; `--no-default-features` builds; org-members check-trace exit 0,
  no findings. The in-force store items' tests (LLR-zbe553, wm5hpc,
  tk4qxu, n7nya3, 4n8zqx, 8jttpb, j35sxz) passed unchanged through the new
  store. Deviations:
  1. clippy `large_enum_variant` on `proof.rs`'s `Kind`: the Held variant
     holds `member: Box<ProvenMember>`; serde treats `Box` transparently,
     so the wire form is the plan's.
  2. clippy `manual_repeat_n` in `arb_wire_proof`: `std::iter::repeat_n`.
  3. absence_proofs.rs drops the unused `device_key` import.
  4. Root Cargo.lock: org-members' entry loses `spin 0.9.9`.

Re-pinned golden value (T9, commit 7af855bd):

| Value | Old (T7) | New |
|---|---|---|
| member-set root (GOLDEN_ROOT), second re-pin | `b0c01649b9f0c8169c5d47b9c0177d37d44f56e04ab5bfab4192cfbee70909d7` | `830f1f7341de3d5ff2ba752e76a25db2c3a8bc180e72a5af8e026b5d16e20c96` |

Structural evidence: `a_member_leaf_hashes_as_documented` recomputes a
one-member root from `blake3::keyed_hash` alone. The wire-bytes pin is
unchanged.

```text
red -> green: every_tree_error_maps_to_an_org_members_error — compile red, no From<TreeError> for OrgMembersError
red -> green: absence_proofs.rs, all 14 tests — compile red, no is_vacant/member/inclusion/device_absence/vacancy on AbsenceProof, Blake3Hasher not TreeHasher
red -> green: hasher_domains_are_separated — compile red, no member_identity/member_content
red -> green: a_member_leaf_hashes_as_documented — same
red -> green: a_member_is_addressed_by_its_id_bits_most_significant_first — compile red, no AbsenceProof::vacancy
red -> green: the_extreme_member_ids_are_handled — red only as its binary's compile failure (same missing API)
red -> green: member_set_root_is_pinned — red on "RE-PIN", then pasted
not red: fuzz every_honest_absence_proof_verifies (Held case) and absence_proof_decoding_and_verify_never_panic (new arb_wire_proof) — not run before the implementation
inherited: every_honest_absence_proof_verifies — from LLR-utp6x4, LLR-dgzy7e, LLR-4xz255
inherited: absence_proof_decoding_and_verify_never_panic — from LLR-2dcnbp, LLR-4rju5r
```
- **T10 done** (task commit f465809, merged 2026-10-25). org-node 375
  passed, quint ×7 exit 0, clippy clean; org-io 257 passed; app cargo 150
  passed; on-chain-client (reported impacted too) 128 passed; downstream
  check-trace: no finding names an item of this change.
  - Re-pinned: org-node `revocation_wire_message_is_pinned` — the notice's
    34-byte proof became 38 bytes (`00` Vacant ‖ `8002` depth 256 ‖ `20` ‖
    `00`×32 map ‖ `00` no siblings ‖ `00` Empty), derived by the format rule
    as the file's precedent does; body length 119 → 123; old value kept in
    the file's comment.
  - app/src-tauri/Cargo.lock (its own workspace) records the new path
    dependencies; no version changed.
  - Open: `npm --prefix app run check` and `run test` exit 127
    (`svelte-kit: command not found`): app/node_modules is absent in the
    worktrees. To be run with dependencies installed before the merge gate.
  - Note for merge: on-chain-client's four open PRs are 45 days old, at its
    `problem_age_days` limit.

```text
red -> green: revocation_wire_message_is_pinned — red (MalformedMessage decoding the old GOLDEN_REVOCATION), re-pinned to the derived value
```

### T11 close

Run 2026-10-25 in the task worktree (`.worktrees/worktree-generic-smt-t11`,
branch `worktree-generic-smt-t11`, off `worktree-generic-smt` at f44e313b).

**SOUP as built.** `sparse-merkle-tree/Cargo.toml` asks `spin = "0.9"`
(`once`) and `serde = "1"` (`derive`, `alloc`, optional); the workspace
`Cargo.lock` resolves the crate's `spin 0.9.9` and `serde 1.0.229`, the
inventory's versions. No row changed; the "As designed" paragraph is now
"As built on 2026-10-25".

**Coverage, sparse-merkle-tree** (stable rustc 1.99.0 (b940084d7
2026-09-28); nightly-2026-10-03 = rustc 1.101.0-nightly (0abfedbc7
2026-10-02); cargo-llvm-cov 0.9.0; aarch64-darwin).

First measurement, before T11's tests: lines 96.99% (483 of 498), regions
94.84% (1030 of 1086), branches 76 of 78 (97.44%). T11's three
sparse-merkle-tree tests closed the `Visitor::expecting` bodies, the
serializer-error `?`s, and the branch llvm-cov counted short because no
single `Path::parse` instantiation took every refusal (a generic function's
figure is its best single instantiation's). After them, the figures the
floors are set from:

*(Code pruned at merge: 2 lines.)*

What no test reaches is named, with its reason, in the Makefile comment
beside the floors: the `Default`/`Debug` impls, which no requirement states;
the `?`s after `DefaultHashes::at` and on `fold`, unreachable by
construction; the `if let Some(byte)` else (the one uncovered branch); the
`serialize_tuple` `?`, which postcard never fails.

**Coverage, person and org-members** (`make coverage-person
coverage-branch-person coverage-org-members coverage-branch-org-members`,
all exit 0):

*(Code pruned at merge: 2 lines.)*

person first measured below its floors: lines 98.69% (452 of 458), regions
98.55% (678 of 688), against 99/99. `DeviceLeaf::new` and `DeviceLeaf::key`
(T7) were never called. Closed by a test
(`the_tree_holds_one_device_leaf_per_key`, LLR-8y8tqr); no floor lowered.
The four regions left are the `?` on `compute_device_root`/`device_tree`
in person's own callers, which pass a `DeviceSlots`, admitting no shared
position (LLR-xsx6gg).

**Trace and units:**

*(Code pruned at merge: 30 lines.)*

No MISSING-TEST, OUT-OF-FORCE-VERIFIES, DANGLING-REF, UNTRACED,
UNANALYZED-DERIVED or NON-RECIPROCAL finding in the three units.

**verify_commands, all three units** (CARGO_HOME=/tmp/cargo_home_fuzz,
QUINT_HOME in the session scratchpad):

*(Code pruned at merge: 13 lines.)*

CI lanes added (`.github/workflows/rust.yml`): test (both feature sets),
msrv, clippy, wasm32 (both feature sets; both checked locally, exit 0);
`make coverage` and `make coverage-branch` now include sparse-merkle-tree.

The four tests T11 adds cover behaviour that existed before them, so no
red was possible on the real code; each was watched fail against a mutant
of the behaviour it verifies, then pass with the mutant reverted:

```text
red -> green: a_depth_four_map_is_refused_for_each_malformed_shape — failed with the map-length check disabled (proofs.rs:183)
red -> green: decoding_refuses_a_value_that_is_not_the_tuple — failed with AbsenceProof decoded through deserialize_any and a visit_str (proof_decoding.rs:89)
red -> green: encoding_into_a_short_buffer_fails — failed with the serialize_element `?`s swallowed (proof_decoding.rs:107)
red -> green: the_tree_holds_one_device_leaf_per_key — failed with device_tree keeping only the first key (device_root.rs:165)
```
- **T11 done** (task commit 0a8987ae, merged 2026-10-25). The close's
  outputs and coverage figures are under "T11 close". sparse-merkle-tree:
  lines 98.80%, regions 96.59%, branches 77 of 78; floors 97 / 95 / 97,
  the unreached code named in the Makefile. person fell below its floors
  (98.69% / 98.55% against 99) because T7's `DeviceLeaf::new` and
  `DeviceLeaf::key` were never called; closed with a test in
  person/tests/device_root.rs (outside T11's planned files), now 100% /
  99.42% / 46 of 46. org-members above its floors. SOUP unchanged
  (spin 0.9.9, serde 1.0.229 match the lock). The msrv lane was not run
  locally.

```text
red -> green: a_depth_four_map_is_refused_for_each_malformed_shape — existing behaviour; red against a mutant with the map-length check disabled
red -> green: decoding_refuses_a_value_that_is_not_the_tuple — red against a mutant decoding through deserialize_any with visit_str
red -> green: encoding_into_a_short_buffer_fails — red against a mutant swallowing the serialize_element `?`s
red -> green: the_tree_holds_one_device_leaf_per_key — red against a mutant of device_tree keeping only the first key
```

## Guideline reviews

### Round 1 (2026-10-25)

**TEST (.guardrails/templates/TEST_GUIDELINES.md): 2 findings (1 medium, 1 low).**
- T-1 medium, Existing suites 1 — person/tests/crate_discipline.rs:57
  `every_rejection_is_a_distinct_named_variant` edited in place with no
  `verifies:`. Disposition: fix — move the five new variants' check to
  `person_absence_proofs.rs::the_proof_refusals_are_distinct_named_variants`
  (LLR-3dvfjb) and revert the in-place edit.
- T-2 low, Seams 1 — sparse-merkle-tree/tests/store.rs:179-180 asserts
  `calculate()` fills the receiver's cells, which no LLR states.
  Disposition: fix — drop the assertion.
- Note: two Quint runs sit under `// verifies: LLR-v6gfc7`; fix — annotate
  them LLR-5jh6t3.

**CODE (.guardrails/templates/CODE_GUIDELINES.md): 17 findings (1 medium, 16 low).**
- C-5 medium, Names 4 — org-members/src/trie.rs apply_delta holds the
  MemberTree in `root`. Disposition: fix — `tree`.
- C-1, C-2, C-3, C-4 low, Names 1/3 — short or generic names in tests,
  src and Quint. Disposition: fix the src and test names; Quint lambda
  parameters (`k`, `d`, `e`) and Rust generic type parameters (`L`, `H`,
  `D`) kept: the surrounding model and the workspace use that convention
  throughout, and renaming only the new ones would split it.
- C-6 low — inline block expressions in proofs.rs. Fix.
- C-7..C-11 low, Simplicity 4 — duplicated helpers/checks. Disposition:
  fix C-8 (org-members `calculated` reuses the tree's rule via a public
  method), C-9 (one `same_position` helper in slots.rs), C-10 (shared
  MemberId leaf functions), C-11 (one memo in tests/common); C-7 kept:
  `keyed` is private to each crate's hasher adapter and person exports no
  hashing helper; the test-local hashers are the oracle and stay
  independent of the code under test.
- C-12, C-13 low — commented-out old value and change history in comments.
  Fix: remove; the history is in this plan.
- C-14, C-15, C-16, C-17 low, Errors — discarded errors without a reason.
  Fix: comments stating why the discarded variants cannot occur;
  `from_siblings` returns `LevelOutOfRange` instead of skipping.

**ARCHITECTURE (.guardrails/templates/ARCHITECTURE_GUIDELINES.md): 14 findings (9 medium, 5 low).**
- A-1 medium, Boundaries 2 — person exports `device_tree`/`DeviceTree`,
  whose methods return `TreeError`. Owner ruling 2026-10-27: narrow
  person's API — `prove_device_absent_in::<H>(devices, key)` returning
  `Result<AbsenceProof<4>, IdentityError>` replaces the export; LLR-8y8tqr
  amended; an architecture test checks no public person signature names
  `TreeError`. LLR-w5yahk's "with the tree's error as its cause" reworded
  to what the code does (a variant named for the check the tree's error
  reports).
- A-2 low — `map_err(|_| …)` discards the cause. Fix with C-14's comments.
- A-3 medium, Domain 3 — "position" names both the 32 bytes and the
  place. Owner ruling 2026-10-27: rename the bytes — `Leaf::position_bytes`,
  `device_position_bytes`; glossary gains "Position bytes".
- A-4, A-5 medium, Domain 2 — undefined public terms (ProofEnding, identity
  hash, content hash, empty-subtree hash; ProvenMember, Vacant/Held proof,
  device absence). Fix: glossary entries.
- A-6 medium, Domain 1 — "device key" in user-facing text. Fix.
- A-7 low — `TrieHasher` keeps "trie". Fix: rename `MemberHasher`
  (org-members only; no downstream user).
- A-8 medium, Core and edges 4 — `device_position` calls blake3 outside
  person's hasher adapter. Fix: move it into device_hasher.rs.
- A-9 low — serde in several modules. Kept: derives plus one wire module
  per crate is the workspace's existing pattern.
- A-10, A-11, A-12 medium; A-13 low, Boundaries 3 — SDD statements with
  two responsibilities or stale. Fix: one-sentence statements; SDD-d9svdj
  restated.
- A-14 medium, Constraints 1 — no architecture test for "no TreeError
  crosses the interface". Fix with A-1.

**ARCHITECTURE round 1 fixes merged** (fix-arch1, 9 commits, 2026-10-27).
855 tests passed; every verify entry of the three units and org-node's
cargo entry green; check-trace clean for the three units. A-1/A-14,
A-2, A-3, A-4/A-5, A-7, A-8, A-10..A-13 fixed as dispositioned.
LLRs amended: LLR-8y8tqr, LLR-w5yahk, LLR-t3e8pp, LLR-mm45ry, LLR-exwx4n,
LLR-vraz7n. SDDs restated: SDD-x43d2z (now "Consumer interfaces"),
SDD-rh9u6z, SDD-ur6uvu, SDD-bx9mm6, SDD-7r833z, SDD-d9svdj.
A-6 partly: the DeviceSlots `expecting` text still says "device keys",
and LLR-5za6mp (merged, already quoting the stale bound 4) quotes it —
passed to the CODE fix batch to supersede.

```text
red -> green: no_public_signature_names_the_tree_or_its_error — red with device_tree/DeviceTree made pub again ("device_root.rs exposes SparseMerkleTree: pub type DeviceTree<H> = …"), green restored
```

**CODE round 1 fixes merged** (fix-code1, 13 commits, 2026-10-27). 855
tests passed; every verify entry green; check-trace clean for the three
units. C-1, C-2, C-4, C-5, C-6, C-9, C-10, C-11, C-12/C-13, C-15, C-16,
C-17 fixed; C-3, C-7 kept; C-8 kept with a comment (no new tree API);
C-14 already met by the architecture fix, one comment added. Not fixed
(left as master wrote them): `for i/j` in hasher_domains_are_separated,
`e`/`as I` in `From<IdentityError>`. Coverage re-measured 2026-10-27:
sparse-merkle-tree 491/497 lines, 1051/1088 regions, 76 of 76 branches;
floors unchanged. A-6 remainder: LLR-cyx9a3 minted, supersedes
LLR-5za6mp (DeviceSlots `expecting` message without "device keys"),
derived, assessed (no hazard impact) in
person/docs/risk/2026-10-04-identity-types.md. person's SOUP serde row
still cites LLR-5za6mp — to the TEST fix batch.

```text
inherited: device_key.rs::a_key_encodes_as_its_32_bytes — from LLR-5za6mp
inherited: person_key.rs::a_key_encodes_as_its_32_bytes — from LLR-5za6mp
inherited: names.rs::names_encode_as_the_string_of_their_nfc_value — from LLR-5za6mp
inherited: slots_decode.rs::slots_encode_as_the_sequence_of_their_keys_in_sorted_order — from LLR-5za6mp
inherited: slots_decode.rs::decoding_a_non_sequence_names_the_expected_form — from LLR-5za6mp
```

**TEST round 1 fixes merged** (fix-test1, commit d08c1c7, 2026-10-27). The
three units' verify entries green; check-trace clean. T-1: the in-place
edit of `every_rejection_is_a_distinct_named_variant` reverted to
master's list; the five new variants' distinctness (pairwise and against
the 11 earlier variants) checked in `the_proof_refusals_are_distinct_named_variants`.
T-2: the over-specified assertion deleted. Note: the two Quint runs under
their own `// verifies: LLR-5jh6t3` block; `scenarioDevicePositionTaken`
annotated. person SOUP serde row cites LLR-cyx9a3.

```text
red -> green: the_proof_refusals_are_distinct_named_variants (extended) — existing behaviour; red against a mutant giving DeviceNotFound DeviceStillHeld's message (assert_ne! at person_absence_proofs.rs:154), green restored
```

### Round 2 (2026-10-27)

**TEST: 1 finding (1 medium).** T2-1 medium, Seams 1 —
person/tests/crate_discipline.rs:144-166: the signature scan sees only
`pub fn`/`pub type`/`pub use` lines, not enum variants, pub fields or
trait-impl methods. New place (not a recurrence).

**CODE: 12 findings, all low** — review done for this guidelines file.
C2-2 (generic parameter names), C2-7/C2-8 (keyed helper, test hashers),
C2-9 (`calculated`) repeat round-1 kept dispositions. The rest (Quint
lambda names; `value`/`how`; `ids()`; `device` field vs parameter in
org-members proof.rs; `leaf()`'s `name`/`index`; `Debug` impl; the
`unwrap_or` fallback in `depth()`; a "now" in a Makefile comment) to be
fixed or dispositioned with the round-2 fixes.

**ARCHITECTURE: 14 findings (6 medium, 8 low).**
- A2-1 medium, Boundaries 2 (cause not kept) — org-members/src/error.rs
  `From<TreeError>`, person/src/device_root.rs, absence.rs: **recurs**
  A-1/A-2 (same clause, same places) after their fix was merged.
- A2-2 medium, Boundaries 2 (dependency's error type reaches callers) —
  person `PersonAbsenceProof::absence()`, `prove_device_absent_in`;
  org-members `vacancy()`, `inclusion()`, `device_absence()` return the
  tree's proof types, whose `verify` returns `TreeError`: **recurs** A-1
  (same clause; person's device_root.rs is the same place).
- A2-3 medium, Domain 3 — "absence proof" means two things in org-members.
- A2-4 medium, Domain 2 — `PersonAbsenceProof` not in a glossary.
- A2-5 medium, Domain 2 — calculated, level, map, siblings, node hash,
  empty leaf not in the tree unit's glossary.
- A2-6 medium, Domain 1 — `ProvenMember::p2p_key()` uses an avoided term.
- A2-7..A2-14 low — "member key"/"trie"/"key" in new prose; blake3 in two
  person modules; serde spread; store builds proofs from pub(crate)
  fields; device_hasher.rs's two jobs; no org-members architecture test
  for LLR-w5yahk's structural half.

Derailment stop (develop-change): A2-1 and A2-2 recur after the round-1 fix.
Put to the owner.

**Owner ruling 2026-10-28 on A2-1/A2-2:** accepted as designed. The
sparse-merkle-tree unit is a declared dependency of both consumers and its
proofs are shared by design (docs/adr/2026-10-23-one-sparse-merkle-tree.md);
a mapped error names the check and keeps no cause. LLR-3dvfjb and
LLR-w5yahk are reworded to scope "no TreeError crosses" to each unit's own
functions and to drop "as its cause". No code change for these two.

**Round 2 fixes merged** (fix-r2, 7 commits, 2026-10-28). person 134,
org-members 266 (+ quint), sparse-merkle-tree, org-node 375 all green;
check-trace exit 0 for the four units. Fixed: T2-1, A2-3..A2-10, A2-13,
A2-14, and the CODE lows; kept: A2-11, A2-12, generic names, keyed
helpers/test hashers, `calculated`, the Debug impl. LLR-3dvfjb,
LLR-w5yahk (owner ruling), LLR-v6smww, LLR-bh53rr amended in place.
person: blake3 now only in the private `keyed_hash` module; position
bytes in a private `position` module; device_hasher.rs holds the Person
device hasher alone. `Depth::<D>::AS_U16` replaces the `unwrap_or`
fallback. Left: LLR-9g2kjg still says "the member key" (no finding named
it).

```text
red -> green: public_signatures_are_read_whole (extended) — red against the old helper (missed variants, pub fields, trait-impl methods)
red -> green: no_public_signature_names_the_tree_or_its_error (extended) — red against mutants: IdentityError::Tree(TreeError); impl From<TreeError> for IdentityError; pub field SparseMerkleTree; green restored
red -> green: no_public_signature_returns_or_holds_the_tree_error (new, LLR-w5yahk) — red against mutants: pub fn returning TreeError on OrgMembersError; pub enum variant Tree(TreeError); green restored
```

### Round 3 (2026-10-28)

**TEST: 2 medium (new).** T3-1, T3-2 Seams 1 — the scanner fixture tests
`public_signatures_are_read_whole` (person/tests/crate_discipline.rs) and
`only_a_parameter_may_name_the_tree_error` (org-members/tests/tree_error_mapping.rs)
test test-local helpers and carry no `verifies:`. Fix: fold their cases
into the annotated scan tests.

**CODE: 14 (2 medium, 12 low).** C3-1 medium, Comments 2 — the stale
comment above `GOLDEN_REVOCATION` (org-node/tests/encoding_golden.rs:126-134).
Fix. C3-2 medium, Simplicity 4 — the signature scanner copied into
org-members' tests. Fix with T3-1/T3-2: one copy reachable from both.
Lows: fix C3-3..C3-6 (test helpers into tests/common), C3-8 (`NotHeldProof`
alias), C3-13/C3-14 (comments); kept C3-7, C3-10, C3-11, C3-12 (as before);
C3-9 resolved by A3-5's rename.

**ARCHITECTURE: 12 (4 medium, 8 low).**
- A3-1, A3-2 (Boundaries 2, cause): owner ruling 2026-10-28 stands —
  accepted.
- A3-3 medium, Core and edges 4 (serde spread): recurs A-9/A2-11.
  **Owner ruling 2026-10-28:** accepted for this change (derives on core
  types plus one `wire` module per crate is the workspace pattern);
  follow-up: tailor the project's ARCHITECTURE guidelines to allow serde
  derives.
- A3-5 medium, Domain 1 — `ProvenMember::group_key()`: "group key" is on
  Member-as-a-group key's _Avoid_ list (the round-2 fix chose it in error).
  Fix: `person_key()`, the code name docs/CONTEXT.md gives a
  PersonPublicKey.
- Lows: fix A3-6 (`DeviceLeaf::key` → `public_key`), A3-7
  (`device_identity`/`member_identity` → `…_identity_hash`), A3-8, A3-9,
  A3-10 (avoided words in new prose), A3-11 (Leaf, Hasher glossary
  entries), A3-12 (an LLR + architecture test for "only keyed_hash calls
  blake3", or drop the claim from the module doc); A3-4 kept (spin in one
  place).

**Round 3 fixes merged** (fix-r3, commits 4f7fb0a7, 2c0c3ce1, 2026-10-28).
sparse-merkle-tree 45 (37 without default features), person 133,
org-members 265 (+ quint), org-node 375; clippy clean; check-trace exit 0
for the four units. Fixed: T3-1, T3-2, C3-1, C3-2, C3-3..C3-6, C3-8,
C3-13, C3-14, A3-5..A3-12. Kept as dispositioned: A3-1..A3-4, C3-7,
C3-10..C3-12. The signature scanner is one file,
person/tests/support/public_signatures.rs, reached by org-members' test
through `#[path]`. Amended in place: LLR-9mjfjn, LLR-8y8tqr, LLR-t465mb,
LLR-9g2kjg. The org-members wire form is unchanged by the `person_key`
rename (postcard encodes no field names).

```text
red -> green: no_public_signature_names_the_tree_or_its_error (fixture cases folded in) — red against a scanner mutant skipping enum variants; green restored
red -> green: no_public_signature_returns_or_holds_the_tree_error (fixture cases folded in) — red against the same mutant; green restored
```

### Round 4 (2026-10-28)

**CODE: 15, all low** — review done for this guidelines file. Repeats of
kept dispositions (generic parameter names, keyed helpers, `calculated`,
the Debug impl, the `org-node` golden history kept as that file's
convention, Makefile/rust.yml dated lines kept as those files'
convention). To fix with round 4: C4-3 (`device_content` /
`member_content` → `…_content_hash`), C4-8 (org-members domain keys as
named constants), C4-9 (the src walk into the shared scanner module),
C4-10 (absence_proofs `device` wrapper), C4-14 (comment for
LevelOutOfRange), C4-15 (cite LLR-mm45ry on `bit`). Kept: C4-4
(`p2p_key` on master's `MemberLeaf`), C4-12 (ruling dates are the
project's convention for bounds).

**TEST: 4 (1 medium, 3 low).** T4-1 medium, Doubles 2/3 — person's
`OrgMembersDomains` copy of org-members' device hasher has no contract
test; `no_set_shares_its_root_with_org_members` checks against the copy.
Fix: org-members asserts `Blake3DeviceHasher`'s four device domain keys
and sentinel (contract), and the "no set shares its root" check moves to
org-members against both real hashers. Lows: T4-2 (`DeviceLeaf::new`/
`public_key` untested by a stated interface) — kept with C-11-style
disposition: `DeviceLeaf` is the public leaf of LLR-8y8tqr; T4-3 (proptest
seeds persisted, not printed) — kept, proptest's default; T4-4 (against
the template's UI 6 clause) — for the guidelines tailoring follow-up.

**ARCHITECTURE: 9 (6 medium, 3 low).** A4-1..A4-3 (Boundaries 2, cause)
and A4-5..A4-7 (serde spread): owner rulings of 2026-10-28 stand —
accepted. A4-4 medium, Domain 3 — "absence proof" names two types in
org-members: **recurs** A2-3. **Owner ruling 2026-10-28:** fix the
glossary — "Absence proof" the general term; "Device absence proof"
org-members' Vacant/Held proof; "Tree absence proof" the tree crate's;
Vacant and Held defined under the former; no code change. Lows A4-8,
A4-9: glossary entries for Member hasher, Device root hasher, Empty-leaf
sentinel.

**Round 4 fixes merged** (fix-r4, 3 commits, 2026-10-28). The three units'
verify entries, org-node's cargo entry and check-trace for the four units
green. T4-1: org-members' `hasher_domains_are_separated` (now LLR-4rdqyn,
LLR-uudst2) checks `Blake3DeviceHasher`'s real outputs and sentinel; new
`no_device_set_shares_its_root_with_the_person_device_root` (LLR-4rdqyn)
compares the two real hashers; person's copy-based test replaced by
`no_domain_key_or_the_sentinel_is_org_members` (LLR-qwp8d7), a check of
person's constants against org-members' literal keys. A4-4: glossary per
the owner's ruling (Absence proof general; Tree / Device / Person absence
proof; Vacant and Held as the two Device kinds). A4-8, A4-9 glossary
entries. C4-3, C4-8, C4-9, C4-10, C4-14, C4-15 fixed. Amended in place:
LLR-9mjfjn, LLR-8y8tqr.

```text
red -> green: hasher_domains_are_separated (extended) — red against mutants changing the device-node key and the sentinel; green restored
red -> green: no_device_set_shares_its_root_with_the_person_device_root (new) — red against a mutant giving org-members person's device domains and sentinel; green restored
red -> green: no_domain_key_or_the_sentinel_is_org_members (new, replaces no_set_shares_its_root_with_org_members) — red against PERSON_DEVICE_NODE_KEY set to org-members' key; green restored
```

### Round 5 (2026-10-28)

**TEST: 1 low** (against the template's UI 6 clause, not the change) —
review done for this guidelines file. Note taken: restore an accepting
case in person `try_from_goes_through_parse`.

**ARCHITECTURE: 9 (3 medium, 6 low).**
- A5-1 medium, Domain 3 — `device_absence()` / `Held { device_absence }`
  hold a Tree absence proof while the glossary's Device absence proof is
  the whole proof: **recurs** A4-4 at code level. **Owner ruling
  2026-10-28:** rename to `device_root_absence`; type names unchanged.
- A5-2 medium, Domain 1 — `Blake3DeviceHasher` and `MemberHasher::Device`
  use avoided "device hasher" / bare "Device". Fix: `Blake3DeviceRootHasher`,
  `MemberHasher::DeviceRoot`.
- A5-3 medium, Boundaries 3 — SDD-d6x85b's statement (two
  responsibilities, both now false), its heading and README row; lesser:
  SDD-57vaj4. Fix: restate in one sentence each.
- Lows: A5-4 `empty_hash` → `empty_subtree_hash`; A5-5 glossary entries
  Device leaf, Device position; A5-9 move `same_device_position` and
  LLR-vraz7n to the DeviceSlots item, or make the helper part of the
  device root's stated interface. A5-6, A5-7 fall under rulings (1), (2);
  A5-8 (spin in node.rs) kept, as A3-4.

**Round 5 fixes merged** (fix-r5, 5 commits, 2026-10-28). The three units'
verify entries, org-node's cargo entry and check-trace green; no renamed
identifier is used downstream. A5-1 (`device_root_absence`), A5-2
(`Blake3DeviceRootHasher`, `MemberHasher::DeviceRoot`), A5-3 (SDD-d6x85b,
SDD-57vaj4 restated), A5-4 (`empty_subtree_hash`), A5-5, A5-9 fixed.
Amended in place: SDD-d6x85b, SDD-57vaj4, LLR-yuyn8f, LLR-exwx4n.

```text
red -> green: try_from_goes_through_parse (accepting case restored) — red against a mutant refusing one-element sets ("a valid set is accepted: DeviceSlotsFull" at slots.rs:121); green restored
```

### Round 6 (2026-10-28), ARCHITECTURE only

4 findings (1 medium, 3 low), plus findings under rulings (1)-(3), accepted.
- A6-1 medium, Domain 3 — "device root" names both the value and the tree
  (new; not a recurrence). Fix: keep "Device root" for the value; add
  "Device root tree" for the tree; reword the lines that mean the tree
  (glossaries, SDD-7r833z, LLR-uudst2, person/src/device_root.rs docs).
- Lows: A6-2 `MemberId` doc "p2p key" → fix; A6-3 "vacancy" — glossary
  ties it to the Vacant proof's Tree absence proof; A6-4 spin in node.rs —
  kept (A3-4). Notes to fix while there: stale `spin::Once` wording in
  org-members/src/trie.rs; the units.yaml comment on the edges.

**Round 6 fixes merged** (fix-r6, 3 commits, 2026-10-28). Wording only;
tests, clippy, check-trace and check-units green. A6-1: "Device root tree"
added to the glossaries; every changed line meaning the tree now says so —
including SDD-7r833z, LLR-uudst2, LLR-utk6uy, LLR-5jh6t3, LLR-exwx4n,
REQ-9sh8qb, REQ-4e23xr, REQ-2sz7j8, HAZ-vjm5fg (this branch's items,
amended in place), the ADR and the unit docs. A6-2, A6-3 and both notes
fixed.

### Round 7 (2026-10-28), ARCHITECTURE only

7 findings (2 medium, 5 low); both mediums new.
- A7-1 medium, Domain 1/2 — `SparseMerkleTree::diff` and "upserted":
  undefined, and "diff" is on Change set's _Avoid_. Fix: rename to
  `changes_between`, glossary "Tree changes" (removed identities,
  upserted leaves) and "Upserted leaf".
- A7-2 medium, Domain 1 — person's `PersonDeviceHasher` / module
  `device_hasher` use the avoided "device hasher". Fix:
  `PersonDeviceRootHasher`, module `device_root_hasher`.
- A7-3 low — `PersonAbsenceProof::absence()` → `device_root_absence()`,
  as org-members' Held accessor. Fix.
- A7-4..A7-7 low — under rulings (1)-(3); the serde-field part of A7-5 is
  covered by LLR-9g2kjg / LLR-985v9t (every decode refusal is the one
  malformed-proof variant): accepted.
- Notes to fix: org-members soup.md's `person` row (DeviceTrieHasher,
  smt.rs) and a "not SOUP, a unit dependency" row for sparse-merkle-tree;
  SDD-bx9mm6's title "Person-hash absence proof" → "Person absence proof".

**Round 7 fixes merged** (fix-r7, 2 commits, 2026-10-28). All verify
entries, org-node's cargo entry, check-trace and check-units green; no
renamed identifier used downstream. A7-1 (`changes_between`; glossary
"Tree changes", "Upserted leaf"), A7-2 (`PersonDeviceRootHasher`, module
`device_root_hasher`), A7-3 (`device_root_absence()`), org-members SOUP
rows, SDD-bx9mm6 title. Amended in place: LLR-u2r4vj, LLR-qwp8d7,
LLR-z8h2w8, SDD-bx9mm6.

### Round 8 (2026-10-28), ARCHITECTURE only

10 findings (1 medium, 9 low).
- A8-1 medium, Domain 1/3 — `MemberHasher::DeviceRoot` names a hasher
  with the term for the root value: **recurs** A5-2 at hasher.rs:11.
  **Owner ruling 2026-10-28:** rename to `DeviceRootHasher`, then end the
  architecture reviews.
- Lows dispositioned (owner ruling 2026-10-28: no further rounds; fold
  into the ARCHITECTURE guidelines tailoring): A8-2 `leaf_hash` term;
  A8-3 "depth" for one node's distance in the store's internals; A8-4
  SDD-7r833z understates its LLRs; A8-5 serde field refusals mapped to
  the malformed-proof variant (LLR-985v9t, LLR-9g2kjg require it); A8-6..
  A8-10 under rulings (1)-(3) and Existing code 1.

Final state of the conformance reviews: TEST — round 5 nothing above low;
CODE — round 4 nothing above low; ARCHITECTURE — ended by owner ruling
after round 8, A8-1 fixed.

**Follow-up (owner, 2026-10-28):** tailor the project's ARCHITECTURE
guidelines (serde derives; root vs unit glossary in a multi-unit repo;
internal names; the template's UI 6 clause in TEST is a separate
tailoring point).

**A8-1 fixed** (fix-r8, commit ce29182, merged 2026-10-28):
`type DeviceRootHasher: person::DeviceRootHasher`. org-members 266
passed, clippy clean, org-node 375 passed, check-trace exit 0.
Development (develop-change) complete; next: check-traceability, then
merge-change.

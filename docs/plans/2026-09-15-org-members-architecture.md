# org-members Architecture, SOUP and LLRs — Implementation Plan

**Goal:** Give the `org-members` unit its first architecture ledger — six
software items, thirty-five low-level requirements, a measured SOUP inventory
— and put the unit's tests behind the LLRs so the design layer is verified
rather than merely written.

**Implements:** SDD-4yr9ge, SDD-d6x85b, SDD-d9svdj, SDD-k5wa4n, SDD-55b2zj,
SDD-m9gs5g, LLR-xzqs9r, LLR-5w2jx8, LLR-pys2ek, LLR-w5nkbu, LLR-paxj7b,
LLR-4czn8t, LLR-xyv6p9, LLR-68tka5, LLR-72p8bz, LLR-kdhd2v, LLR-zbe553,
LLR-wm5hpc, LLR-tk4qxu, LLR-n7nya3, LLR-4n8zqx, LLR-8jttpb, LLR-ub6dw9,
LLR-fv75ec, LLR-ch2pkw, LLR-j4d38d, LLR-4phmjf, LLR-s97ywt, LLR-w92psx,
LLR-k89ahd, LLR-mmst86, LLR-g6arcs, LLR-v3jqau, LLR-au8het, LLR-y38jfk,
LLR-7tdqv9, LLR-xmpqn2, LLR-juxk9q, LLR-h7stq2, LLR-h9gs32, LLR-sa3ugj

**Safety class:** C (`org-members/.guardrails/config.yaml`; no per-item
override — every item below is class C, which is why every one of them carries
LLRs rather than the class B "optional per item").

**Verification:** `cargo test -p org-members`, `quint typecheck
quint/membership.qnt`, `quint typecheck quint/protocol.qnt`, and
`make coverage-org-members`.

This is tooth 4 of `docs/plans/2026-09-05-ratchet-gap-analysis.md`, scoped to
the first of four units. Teeth 1–3 are merged; all four units have per-unit
ISO 14971 risk registers as of 2026-09-15 (`5c0c710`).

**Filenames, after `finalize-docs.sh` ran on 2026-09-17.** Several places below
name the ledger files by their draft names, which no longer exist. The text is
left as written — this project dates its corrections rather than rewriting the
record — so read them as:

| written below | actual file |
|---|---|
| `…/architecture/DRAFT-worktree-guardrails-org-members-arch-decomposition.md` | `org-members/docs/architecture/2026-09-17-decomposition.md` |
| `…/risk/DRAFT-worktree-guardrails-org-members-arch-design-derived.md` | `org-members/docs/risk/2026-09-17-design-derived.md` |
| `…/problems/DRAFT-worktree-guardrails-org-members-arch-review-fixes.md` | `org-members/docs/problems/2026-09-17-review-fixes.md` |
| `…/problems/DRAFT-worktree-guardrails-org-members-arch-review2-fixes.md` | `org-members/docs/problems/2026-09-17-review2-fixes.md` |

(The third row was added on 2026-09-17 by T12: the table listed only the
architecture and risk files, while T11's own record names the problems draft in
its `Files touched:` list, so a reader of T11 had no mapping for it. The fourth
was added the same day when T12's own draft was finalised.)

**The same defect recurred twice, which makes it a process defect rather than an
oversight.** Review round 2's finding 6a was a ledger cross-reference left
pointing at a draft name after `finalize-docs.sh` renamed it. T12 fixed that one
— and then created the identical problem with its own draft, because a fix round
cannot fix a rename that has not happened yet. The renames land at
`merge-change` step 3, *after* every fix round that writes a draft.

So the rule this project already had — the 2026-09-08 record's "three prose
cross-references updated to the new names in the same commit" — is the right
one, and it belongs to the **finalize commit**, not to whoever wrote the draft.
`finalize-docs.sh` renames the file and says nothing about references to it, and
no gate reads prose links, so nothing mechanical will ever catch this. After
every run of that script, grep the tree for the draft name it just consumed.

---

## Baseline, measured in this worktree on 2026-09-15

Run before anything was edited (`worktree-discipline` step 4), branched off
local `master` at `5c0c710` — `origin/master` is 44 commits behind and is not
the base:

```
cargo test -p org-members    0 + 6 + 102 + 1 + 0 = 109 passed, 0 failed
quint typecheck quint/membership.qnt                          exit 0
quint typecheck quint/protocol.qnt                            exit 0
```

`check-trace.sh` with `GR_CONFIG=org-members/.guardrails/config.yaml`:

```
checked: REQ 12, HAZ 8, RC 10, SDD 0, LLR 0, PR 1
```

**`SDD 0, LLR 0` is the gap this change closes.** All four units' architecture
ledgers are the untouched template, and `soup.md` is an empty table row in each.

### The test capacity this rests on

| File | Tests | `verifies:` annotations | Unannotated |
|---|---|---|---|
| `org-members/tests/integration_test.rs` | 102 | 45 | 57 |
| `org-members/tests/fuzz_tests.rs` | 6 | 4 | 2 |
| `org-members/tests/mbt_conformance.rs` | 1 | 0 | 1 |

That is the fact that makes this tooth affordable: **sixty tests already exist
that verify design behaviour and trace to nothing.** Thirty of the thirty-five
LLRs below are carried by a test that is already written and already green. Only
five need a new test.

**A test that is already green has never been watched failing, so annotating it
proves nothing on its own.** Every LLR below is therefore discharged by *red by
mutation*, the practice this project used throughout tooth 3: mutate the source
so the named behaviour is wrong, watch the named test fail **for that reason**,
revert, verify the revert is byte-identical by SHA-256, and name the mutation in
the dispatch report. A mutation that reds nothing is a **measured negative** and
is recorded as one rather than quietly dropped — that is evidence about the
test, not an absence of evidence.

---

## The three decisions this plan was given

Put to the owner on 2026-09-15, each with a recommendation; all three
recommendations were taken.

| Decision | Answer |
|---|---|
| Scoping | **One change per unit, org-members first.** Four changes, each carried to its own signed squash, matching tooth 3's rhythm. |
| SOUP evidence basis | **Track the real lockfiles, delete the inert one.** SOUP tables cite the lockfile rather than restating versions. |
| Granularity | **One SDD per architectural responsibility.** Six items; LLRs per interface, algorithm, error behaviour and resource limit. |

### Why granularity is the test budget

`check-trace.sh:502` fires `MISSING-TEST` on every **defined** LLR with no
direct `verifies:` reference. The transitivity at `:506` runs **upward only** —
a tested LLR discharges its parent REQ; a tested REQ discharges nothing beneath
it. So each LLR written is one annotation owed, and the number of LLRs is the
number of mutations this change must perform. Thirty-five is the budget the
granularity decision bought.

The twelve REQs stay covered either way: all twelve already carry direct
`verifies:` annotations, so no REQ's coverage depends on this change landing.

---

## The lockfile finding

The gap analysis recorded this as "the root `Cargo.lock` is still gitignored —
either it becomes tracked or each unit's SOUP table carries versions". Measured
here, it is worse than that.

```
$ cargo locate-project --workspace        # run from org-members/
{"root":".../guardrails-org-members-arch/Cargo.toml"}
```

`org-members` declares no `[workspace]` of its own, so it is a member of the
**root** workspace and the lockfile that governs its build is `/Cargo.lock`.
That file is gitignored. Meanwhile `org-members/Cargo.lock` **is** tracked, and
cargo never reads it:

| | `/Cargo.lock` (governs) | `org-members/Cargo.lock` (tracked, inert) |
|---|---|---|
| `blake3` | **1.8.7** | **1.8.5** |
| `ed25519-dalek` | 2.2.0 **and** 3.0.0-pre.6 | 2.2.0 |
| packages | 797 | 121 |

So the one tracked lockfile for this unit records a version of the hash function
that nothing builds with. A SOUP inventory sourced from it would be wrong on its
first row. This is exactly the drift the "track the lockfiles" decision was made
to remove, and it was already real before the decision was taken.

The root lockfile also carries **two** versions of the signature library, one of
them a pre-release (`3.0.0-pre.6`). T1 must measure whether that pre-release is
in `org-members`' own dependency closure or belongs to another workspace member,
and record the answer rather than assuming it.

Measured closure for `org-members` with default features (`cargo tree -p
org-members --edges normal`): nine direct dependencies — `blake3` 1.8.7,
`ed25519-dalek` 2.2.0, `hashbrown` 0.15.5, `postcard` 1.1.3, `serde` 1.0.229,
`spin` 0.9.9, `thiserror` 2.0.20, `unicode-normalization` 0.1.25,
`unicode-security` 0.1.2 — over a transitive closure of roughly thirty crates.
`ed25519-dalek` resolves to **2.2.0** here, not the pre-release.

Repository-root files are implicitly outside every unit (`.guardrails/units.yaml`
header), so tracking `/Cargo.lock` raises no `UNCLAIMED-PATH`.

---

## The architecture to be written

The text below is the deliverable. T2 copies each item **verbatim** into
`org-members/docs/architecture/DRAFT-worktree-guardrails-org-members-arch-decomposition.md`;
it is written here so that the design decisions live in the plan and the tasks
copy rather than invent.

Every ID below was minted by `.guardrails/scripts/new-id.sh --unit org-members`
on 2026-09-15 and is final.

### Overview (for the ledger's Overview section)

`org-members` is the membership authority: an immutable, hash-addressed record
of who belongs to an organisation and which keys speak for them. It computes no
policy and performs no I/O. Six items divide it, along the line between *what a
member is*, *how the record is hashed*, *how the record is stored and changed*,
and *how a change crosses a process boundary*.

There is no segregation boundary inside the unit: every item is class C, so
IEC 62304 §5.3.5 has nothing to argue here. The boundary that matters is the
one at the crate's edge, and it is stated as a non-responsibility —
authentication, organisation binding, replay protection across time, authority,
and supply of an independent trusted root all lie **above** this crate
(`org-members/src/delta.rs`, "What this crate does NOT do"). SDD-55b2zj carries
that boundary.

---

### SDD-4yr9ge — Member record and validation

`org-members/src/types.rs`, `org-members/src/normalize.rs`

    **SDD-4yr9ge**: the validated, canonically serialisable member record — the
    immutable identifier, the handle, the member-as-a-group key, the device key
    set and the personal fields — together with the validation and
    normalisation every construction path applies, including the path from
    deserialised bytes. traces: REQ-crjxk8, REQ-h5ret5, REQ-m8aexh, REQ-xdx2c2,
    REQ-shk82j

| LLR | Behaviour | `satisfies:` |
|---|---|---|
| LLR-xzqs9r | `validate_handle` returns the NFC form and rejects a handle that is empty, exceeds 128 bytes after normalisation, contains an uppercase character, contains `.`, or mixes Unicode scripts — `-` being permitted alongside any script | REQ-h5ret5 |
| LLR-5w2jx8 | `handle_skeleton` returns the UTS#39 skeleton of a handle, so that two handles rendering alike share a skeleton | REQ-m8aexh |
| LLR-pys2ek | a member holds at most `MAX_DEVICES` device keys, and `MAX_DEVICES` is 4 because the device sub-trie is a fixed depth-2 binary tree of four slots | REQ-xdx2c2 |
| LLR-w5nkbu | `name` and `surname` are each bounded at 128 bytes after NFC normalisation, and a field exceeding its bound is rejected as `FieldTooLong { field, max }` naming which field and which limit | derived |
| LLR-paxj7b | `MemberLeaf::new` rejects a member constructed with no device key, so that the zero-device state is reachable only through the isolation operation | derived |
| LLR-4czn8t | the `Debug` rendering of a member record redacts the handle, the name and the surname | derived |
| LLR-xyv6p9 | deserialising a device key set accepts only a strictly increasing, duplicate-free list within `MAX_DEVICES`, and accepts the empty list | REQ-shk82j, REQ-xdx2c2 |
| LLR-68tka5 | deserialising a member record re-runs handle validation, so a handle the software would refuse cannot enter through the wire form | REQ-shk82j |

**LLR-pys2ek is owed.** `org-members/docs/requirements/2026-08-31-org-membership.md`
says so in as many words under REQ-xdx2c2: the bound "belongs in a low-level
requirement under the software item that owns the sub-trie, which does not exist
yet", and until it is written the number 4 "lives only in places no gate reads".
This is the LLR that closes that note. When T2 lands, the note's forward
reference is satisfied — but **do not edit REQ-xdx2c2's text or its note**: it
is a dated ledger entry and the project records corrections by date rather than
by rewrite. T9 records the discharge in this change's own risk draft instead.

---

### SDD-d6x85b — Domain-separated hashing and the device sub-trie

`org-members/src/hasher.rs`, `org-members/src/device_trie.rs`

    **SDD-d6x85b**: the hash interface the membership record is committed
    through — four separated domains, and the fixed-shape device sub-trie whose
    root enters the member leaf hash. traces: REQ-xdx2c2, REQ-avmu3j,
    REQ-4umsuz

| LLR | Behaviour | `satisfies:` |
|---|---|---|
| LLR-72p8bz | the hash interface provides four separated domains — member leaf, member node, device leaf, device node — such that the same input hashed in two domains yields two different values | derived |
| LLR-kdhd2v | the device sub-trie is a fixed depth-2 binary tree of four slots whose unoccupied slots hash a device empty sentinel, and device keys are held sorted so that construction order does not change the record's hash | REQ-xdx2c2 |

---

### SDD-d9svdj — Sparse Merkle trie store

`org-members/src/smt.rs`, `org-members/src/node.rs`

    **SDD-d9svdj**: the immutable 256-level sparse Merkle store — addressing,
    path-copying, lazy hashing and the diff walk — which holds member records
    without interpreting them. traces: REQ-d3prca, REQ-avmu3j, REQ-crjxk8

| LLR | Behaviour | `satisfies:` |
|---|---|---|
| LLR-zbe553 | a member is addressed by the 256 bits of its identifier, index 0 being the most significant bit of the first byte | derived |
| LLR-wm5hpc | the hash of an empty subtree at each of the 257 levels is computed once per hasher, so that an absent member contributes its level's default and a trie emptied of members returns to the empty root | derived |
| LLR-tk4qxu | a modification copies only the path from the changed leaf to the root, sharing every other node, and leaves the trie it was applied to observably unchanged | REQ-d3prca |
| LLR-n7nya3 | each node's hash is a write-once cell; `recalculate()` fills every unset cell bottom-up, and the root value is refused with `HashesNotCalculated` until it has | REQ-avmu3j |
| LLR-4n8zqx | the root of a trie is determined by its member set alone, not by the order the members were inserted in | derived |
| LLR-8jttpb | the diff walk descends the left subtree before the right, so removed identifiers and upserted records are produced in strictly increasing identifier order | derived |

> **Pointer added 2026-09-17 (independent review, finding-10): LLR-tk4qxu's row
> above is the PRE-NARROWING wording and is no longer what the item says.** The
> row is left as written, because this plan is a dated record of what was
> planned and this project corrects by dating rather than by rewriting. On
> 2026-09-15, before the ledger file was ever merged, T5 measured that the
> clause "copies only the path from the changed leaf to the root, sharing every
> other node" could not be discriminated by any test — it describes internal
> structure invisible at the item's interface — and the clause was moved out of
> the item into design rationale. The item now reads: *a modification leaves the
> trie it was applied to observably unchanged, producing the modified membership
> as a separate value.* The narrowing and its reasoning are in
> `org-members/docs/architecture/2026-09-17-decomposition.md` under SDD-d9svdj;
> the measurement is in T5's DONE block below and in
> `org-members/docs/risk/2026-09-17-design-derived.md`. **T2's instruction to
> copy this table "verbatim" therefore describes what T2 did on the day, not
> what the ledger should say now** — the ledger is the authority for item text,
> this table for what was planned.

**LLR-8jttpb is the load-bearing one.** It is the reason the canonical form in
SDD-55b2zj holds *by construction* for every delta this crate produces, rather
than only being checked on deltas it receives.

---

### SDD-k5wa4n — Membership operations

`org-members/src/trie.rs`

    **SDD-k5wa4n**: the organisation trie and its eight membership operations,
    together with the handle and skeleton indexes that make uniqueness and
    confusability decidable without walking the store. traces: REQ-crjxk8,
    REQ-kmvc96, REQ-m8aexh, REQ-xdx2c2, REQ-ewdg2q, REQ-r784fu, REQ-avmu3j,
    REQ-d3prca

| LLR | Behaviour | `satisfies:` |
|---|---|---|
| LLR-ub6dw9 | the trie carries a handle-to-identifier index and a skeleton-to-handle index, and every operation that adds, changes or removes a member updates both | derived |
| LLR-fv75ec | `add_member` rejects an identifier already present, a handle already held, and a handle whose skeleton matches one already held | REQ-crjxk8, REQ-kmvc96, REQ-m8aexh |
| LLR-ch2pkw | `genesis` applies the same identifier, handle and skeleton checks as `add_member` to its initial member set | REQ-crjxk8, REQ-kmvc96, REQ-m8aexh |
| LLR-j4d38d | `delete_member` removes the member's handle and skeleton from the indexes, so the handle becomes available to another member | REQ-kmvc96 |
| LLR-4phmjf | `add_p2p_device` rejects a device the member already holds and a member already holding `MAX_DEVICES`, and does not replace the member's key | REQ-xdx2c2 |
| LLR-s97ywt | `delete_p2p_device` removes the device key and replaces the member-as-a-group key in one operation, and isolates the member when the device removed was the last | REQ-ewdg2q |
| LLR-w92psx | `emergency_isolate_member` removes every device key and replaces the member-as-a-group key in one step, retains the member in the organisation, and leaves the member restorable by adding a device key | REQ-r784fu |
| LLR-k89ahd | `rotate_p2p_key` replaces the member-as-a-group key and changes no other field, the device set included | derived |
| LLR-mmst86 | `update_handle` revalidates the new handle and re-checks it for uniqueness and confusability against every other member | REQ-h5ret5, REQ-kmvc96, REQ-m8aexh |
| LLR-g6arcs | `update_name_surname` NFC-normalises both fields and applies the bounds of LLR-w5nkbu | derived |
| LLR-v3jqau | every operation naming a member that is not in the organisation is refused with `IdNotFound` | REQ-ds8ryr |

**LLR-s97ywt states what the software does, which is less than REQ-ewdg2q
requires.** The requirement's second clause — reject a replacement key equal to
the key being replaced — is **not implemented**, recorded as PR-zz4exm and open
since 2026-08-31. The LLR is deliberately worded to the implemented behaviour
rather than the required behaviour, because an LLR that restated the
requirement would be satisfied by a test that cannot exist, and the gap would
vanish into a green gate. T9 records this, and the problem report stays open.

---

### SDD-55b2zj — Delta exchange and the trust boundary

`org-members/src/delta.rs`, and `apply_delta` / `calculate_delta` /
the canonical-form check in `org-members/src/trie.rs`

    **SDD-55b2zj**: the change set that crosses a process boundary — anchored
    to the record it was computed against, canonical in its encoding, and
    usable only after its result is verified against a root supplied
    independently of it. traces: REQ-4umsuz, REQ-shk82j, REQ-d3prca

| LLR | Behaviour | `satisfies:` |
|---|---|---|
| LLR-au8het | a change set names the record root it was computed against, and applying it to a record with a different root is refused with `DeltaBaseMismatch` | REQ-4umsuz |
| LLR-y38jfk | applying a change set yields a candidate record that exposes no member query, and only verification against an expected root turns it into a usable record | REQ-4umsuz |
| LLR-7tdqv9 | verification refuses a candidate whose root differs from the expected root, and the candidate is consumed either way | REQ-4umsuz |
| LLR-xmpqn2 | a change set is accepted only in canonical form: removals strictly increasing and all present in the record, upserts strictly increasing and each observably changing the record, and the two sets disjoint | REQ-4umsuz, REQ-shk82j |
| LLR-juxk9q | an upserted member record is checked for handle uniqueness and confusability when the change set is applied, not when it is decoded | REQ-shk82j |
| LLR-h7stq2 | `calculate_delta(old)` produces the change set transforming `old` into the receiver, and is refused when either record has uncomputed hashes | derived |

**What LLR-xmpqn2 buys, and what it does not.** Together with LLR-8jttpb it
makes the postcard encoding of an accepted change set the *unique* byte string
for a transition between two roots. It does not authenticate the change set,
bind it to an organisation, protect against replay across a root the record
revisits, or decide whether the sender was permitted to make the change. Those
are the caller's, and LLR-y38jfk's `expected_root` must reach the caller by a
path the attacker does not control. This is the crate's stated trust boundary,
not an omission.

---

### SDD-m9gs5g — Error reporting and panic-freedom

`org-members/src/error.rs`, and the crate-level lint posture in
`org-members/src/lib.rs`

    **SDD-m9gs5g**: the single typed error by which every rejection leaves the
    crate, and the lint posture that keeps a rejection from becoming a panic.
    traces: REQ-ds8ryr

| LLR | Behaviour | `satisfies:` |
|---|---|---|
| LLR-h9gs32 | every rejected operation returns an `OrgMembersError` variant, and the crate denies `unwrap`, `expect` and `panic` at the lint level so that no input reaches a panicking path | REQ-ds8ryr |
| LLR-sa3ugj | `MalformedDelta` renders the rule that was broken, and `FieldTooLong` renders the field and the limit | REQ-ds8ryr |

---

## The twelve derived LLRs

`check-trace.sh` reports `UNANALYZED-DERIVED` for any item marked
`satisfies: derived` that the risk ledger never mentions. Twelve of the
thirty-five are derived and must each be assessed in T9:

LLR-w5nkbu, LLR-paxj7b, LLR-4czn8t, LLR-72p8bz, LLR-zbe553, LLR-wm5hpc,
LLR-4n8zqx, LLR-8jttpb, LLR-ub6dw9, LLR-k89ahd, LLR-g6arcs, LLR-h7stq2.

"No hazard impact because `<reason>`" is a valid assessment; silence is not.

---

## Tasks

Nine tasks. T1 and T2 touch disjoint files and may run alongside anything.
**T3–T8 are serial** — every one of them edits
`org-members/tests/integration_test.rs`, and `plan-change`'s rule is the file
sets, not the judgment that the edits are independent.

Each of T3–T8 runs the `develop-change` loop for one software item: write or
annotate the test, prove it discriminates, commit. Each returns the dispatch
report shape, with one `red -> green:` line per LLR naming the mutation.

### The mutation protocol, for T3–T8

For each LLR whose carrier test already exists and already passes:

1. Edit `org-members/src/<file>` so the LLR's behaviour is wrong in the
   narrowest way that expresses it (the table in each task names the mutation).
2. `cargo test -p org-members --test integration_test <test name>` — watch the
   named test fail, and read the failure to confirm it failed **for the
   mutated reason** and not for a compile error or an unrelated assertion.
3. Revert. Verify byte-identical:
   `shasum -a 256 org-members/src/<file>` before and after must match.
4. Record in the dispatch report:
   `red -> green: <test name> — <mutation>, watched fail for that reason, reverted (sha256 identical)`.

A mutation that leaves every test green is **not** a failure of the task. Record
it as `measured negative: <mutation> — no test reddened` and carry on; T9 and
the verification record both keep it. Do not invent a test to make a negative
disappear without saying that is what happened.

---

### T1 — Lockfiles and the SOUP inventory

**Files touched:** `.gitignore`, `Cargo.lock`, `app/src-tauri/Cargo.lock`,
`org-members/Cargo.lock`, `org-members/docs/architecture/soup.md`
**Parallel:** yes

1. Remove the `Cargo.lock` line from `.gitignore` (line 5) and replace it with a
   comment recording why lockfiles are tracked: an application-side workspace
   pins its dependencies, and the SOUP inventory of a class C unit cites the
   lockfile as its evidence.
2. `git add Cargo.lock app/src-tauri/Cargo.lock`.
3. `git rm --cached org-members/Cargo.lock` and delete the file. It is inert —
   `cargo locate-project --workspace` run from `org-members/` names the
   repository root — and it disagrees with what builds (`blake3` 1.8.5 against
   1.8.7). Removing a file cargo never reads changes no build; prove that by
   re-running the suite afterwards and reporting the count.
4. Measure the closure and write `org-members/docs/architecture/soup.md`:

   ```sh
   cargo tree -p org-members --edges normal --prefix depth | sort -u
   ```

   One row per **direct** dependency, with the transitive closure named by count
   rather than enumerated row by row. Each row: name, exact version from
   `/Cargo.lock`, role in the system, the requirements it supports, and risk
   considerations.
5. **Answer the pre-release question rather than assuming it.** `/Cargo.lock`
   carries both `ed25519-dalek` 2.2.0 and 3.0.0-pre.6. Determine which workspace
   member pulls the pre-release — `cargo tree -i ed25519-dalek@3.0.0-pre.6` — and
   record the answer in the SOUP file. If it is in `org-members`' own closure
   that is a finding about a pre-release cryptographic dependency in a class C
   unit and the task must say so plainly; the measurement in this plan says it
   is not, and the task confirms or corrects that.
6. The `blake3`, `ed25519-dalek`, `unicode-security` and `postcard` rows are the
   safety-relevant ones: the first two carry REQ-avmu3j and REQ-4umsuz, the
   third carries REQ-m8aexh, and the fourth carries the canonical encoding
   LLR-xmpqn2 depends on. Their "risk considerations" cells must be written, not
   left as a dash.

**Expected output:** `cargo test -p org-members` still reports 109 passed, 0
failed. `git status --porcelain` shows `Cargo.lock` and `app/src-tauri/Cargo.lock`
added, `org-members/Cargo.lock` deleted, `.gitignore` and `soup.md` modified.

#### T1 — DONE (task branch `…-t1` at `84932ee`, merged into the change branch)

```
cargo test -p org-members, after org-members/Cargo.lock was deleted
    0 + 6 + 102 + 1 + 0 = 109 passed, 0 failed  — the baseline, unchanged
```

That equality is the evidence, not a formality: deleting a lockfile cargo
actually read would change resolution. It did not, so the file was inert.

**The pre-release question, answered.** `cargo tree -i ed25519-dalek@3.0.0-pre.6`
resolves its only parents to `iroh` 0.98.2, `iroh-base` 0.98.0 and `iroh-gossip`
0.98.0 — reached from `p2panda-net` 0.6.0 into `spike-p2panda`, and from `iroh`
directly as a **dev-dependency** of `org-node`. No path reaches `org-members`,
which takes stable 2.2.0 on the normal-edge path. So it is not a class C
pre-release finding for this unit. **It is a live one for `org-node`**, whose
architecture tooth follows this one, and it belongs in that unit's SOUP
inventory rather than being forgotten because it was measured here.

Three things the plan did not anticipate:

1. **`app/src-tauri/Cargo.lock` did not exist at all** — not untracked, absent.
   That workspace had never had a lockfile generated, and the blanket ignore
   rule meant nothing ever noticed. T1 generated it (805 packages) before
   tracking it. Until this change, `app` — the shipped binary — had no pinned
   dependency set of any kind.
2. **`/Cargo.lock` re-resolves byte-identically** from scratch in a fresh
   worktree (`diff` clean, 797 packages). So tracking it changes nothing about
   what builds; it only makes what builds reproducible and citable.
3. **`check-trace.sh` was transiently red on 13 `DANGLING-REF`s** while T1 stood
   alone — the `SDD-*`/`LLR-*` IDs the new `soup.md` rows cite. T2 defines all
   thirteen, so the merge of both clears it. A task-boundary artifact, recorded
   because a reader of T1's branch in isolation would otherwise read it as a
   defect.

**An evidence gap, recorded in `soup.md` rather than glossed.** No advisory
database was consulted: neither `cargo-audit` nor `cargo-deny` is installed in
this toolchain and the `Makefile` has no audit target, so the risk column of
every row is written from the code's actual use of the crate rather than from an
advisory feed. That is a real limitation of this inventory and it is the
ratchet checklist's open `npm audit` / `cargo audit` decision surfacing on the
Rust side. Two substantive findings came out of writing those rows anyway:

- **`ed25519-dalek` is used for `VerifyingKey` point validation only.** There is
  no signing and no signature verification anywhere in `org-members/src`, so the
  historical dalek signing-API weaknesses sit outside the compiled paths.
- **`unicode-security` 0.1.2 is the weakest row in the table**, because
  REQ-m8aexh's homograph control *is* that crate's confusables table, and the
  table ages with every Unicode revision. The control is exactly as current as
  the dependency.

---

### T2 — The architecture ledger

**Files touched:**
`org-members/docs/architecture/DRAFT-worktree-guardrails-org-members-arch-decomposition.md`
**Parallel:** yes

> One item has since diverged from the table this task copies: LLR-tk4qxu was
> narrowed on 2026-09-15 after T5's measurement. See the pointer at the
> SDD-d9svdj table above. (Added 2026-09-17, independent review finding-10.)

Create the draft ledger file and copy in, **verbatim from this plan**: the
Overview, the six `**SDD-…**:` items with their `traces:` lines, and the
thirty-five `**LLR-…**:` items with their `satisfies:` lines. Each item's
definition line starts at column one; each `traces:` / `satisfies:` line sits on
the header line or directly beneath it, with nothing between — an SDD block ends
at the next definition line or markdown heading, so a bold aside or a fenced
block in between is `UNTRACED-DESIGN`.

Carry across the three notes this plan attaches to items — LLR-pys2ek closing
REQ-xdx2c2's forward reference, LLR-s97ywt's wording against PR-zz4exm, and
LLR-xmpqn2's trust boundary — as prose beneath the relevant table, **not**
between an item header and its annotation line.

Do not edit `org-members/docs/architecture/README.md` beyond its Overview
section, and do not touch any dated file in `org-members/docs/requirements/` or
`org-members/docs/risk/`.

**Expected output:**

```sh
GR_CONFIG=org-members/.guardrails/config.yaml sh .guardrails/scripts/check-ids.sh --allow-draft-files
```
exit 0, and

```sh
GR_CONFIG=org-members/.guardrails/config.yaml sh .guardrails/scripts/check-trace.sh
```
reporting `SDD 6, LLR 35` — and failing with thirty-five `MISSING-TEST` lines
and twelve `UNANALYZED-DERIVED` lines. **That failure is the expected result of
T2 in isolation**; T3–T9 clear it. Report the counts, not "it failed".

#### T2 — DONE (task branch `…-t2` at `f52cf64`, merged into the change branch)

```
check-ids.sh --allow-draft-files                                   exit 0
check-trace.sh   REQ 12, HAZ 8, RC 10, SDD 6, LLR 35, PR 1         exit 1
                 MISSING-TEST 35          (predicted 35 — exact)
                 UNANALYZED-DERIVED 12    (predicted 12 — exact, same twelve IDs)
                 UNRESOLVED-PR PR-zz4exm (open 15 days)  — pre-existing
no UNTRACED-DESIGN, no MALFORMED-ID, no MISPLACED-ITEM, no DANGLING-REF
```

The 41 defined IDs were diffed against this plan's `Implements:` list and are
set-identical. `UNRESOLVED-PR` was confirmed present at baseline too, with
`SDD 0, LLR 0`, so it is not this task's doing.

**One thing this plan got wrong, and the task was right to depart from it.**
The LLRs are presented above in tables, which is fine for reading a plan and
useless in a ledger: `gr_block_opens` anchors `**LLR-xxxxxx**:` at column one,
so a definition inside a table cell is invisible to every gate. T2 wrote each
item as a column-one block with its `satisfies:` in the same wrapped paragraph.
The item text is verbatim from the Behaviour cells. **T3–T8 must read the IDs
and wording from the ledger file, not from this plan's tables.**

---

### T3 — SDD-4yr9ge: member record and validation

**Files touched:** `org-members/tests/integration_test.rs`
**Parallel:** no (serial, first of T3–T8)

Eight LLRs, all carried by existing tests. Add the LLR to the `verifies:` line
where one exists; add a `/// verifies:` line where there is none.

| LLR | Carrier test(s) | Mutation to red it |
|---|---|---|
| LLR-xzqs9r | `handle_valid_ascii`, `handle_empty_rejected`, `handle_dot_rejected`, `handle_uppercase_rejected`, `handle_nfc_normalized`, `handle_mixed_script_rejected`, `handle_single_script_unicode_ok`, `handle_hyphen_allowed`, `handle_too_long_rejected`, `handle_digits_allowed` | in `types.rs::validate_handle`, accept an uppercase character |
| LLR-5w2jx8 | `genesis_rejects_confusables`, `insert_rejects_confusable_handle`, `update_rejects_confusable_handle` | make `handle_skeleton` return its input unchanged |
| LLR-pys2ek | `add_p2p_device_rejects_when_full`, `member_leaf_too_many_devices`, `deserialize_rejects_too_many_devices` | raise `MAX_DEVICES` to 5 |
| LLR-w5nkbu | `member_leaf_new_rejects_oversized_name`, `member_leaf_new_rejects_oversized_surname`, `member_leaf_new_accepts_max_length_name_and_surname`, `deserialize_rejects_oversized_name`, `deserialize_rejects_oversized_surname`, `update_name_surname_rejects_oversized_name`, `update_name_surname_rejects_oversized_surname` | raise `MAX_NAME_LEN` to 129 |
| LLR-paxj7b | `member_leaf_empty_devices`, `member_leaf_new_rejects_empty_device_list` | drop the ≥1-device check in `MemberLeaf::new` |
| LLR-4czn8t | `member_leaf_debug_redacts_pii` | print the handle in `MemberLeaf`'s `Debug` |
| LLR-xyv6p9 | `deserialize_rejects_unsorted_devices`, `deserialize_rejects_duplicate_devices`, `deserialize_rejects_too_many_devices`, `deserialize_accepts_sorted_unique_devices`, `deserialize_accepts_empty_device_list` | drop the strictly-increasing check in `P2pDeviceSlots`' `Deserialize` |
| LLR-68tka5 | `deserialize_rejects_invalid_handle` | skip `validate_handle` in `MemberLeaf`'s `Deserialize` |

`handle_digits_allowed` and the seven length tests currently carry no
annotation; they gain one here. `deserialize_rejects_too_many_devices` already
carries `REQ-shk82j, REQ-xdx2c2` and gains two LLRs.

**Expected output:** `cargo test -p org-members` reports 109 passed, 0 failed —
no test is added by this task.

#### T3 — DONE (task branch `…-t3` at `4c95d44`, merged into the change branch)

Eight LLRs annotated across 31 tests; **no measured negatives**; suite unchanged
at 109 passed, 0 failed; `MISSING-TEST` 35 → 27, the eight cleared being exactly
T3's eight. `shasum -a 256 -c` over all ten `org-members/src/*.rs` clean after
every mutation round.

**The task ran three supplementary mutations this plan did not ask for, and was
right to.** The plan named one mutation per LLR, which is wrong wherever an LLR
has several clauses and its carriers assert them separately: the named mutation
for LLR-w5nkbu reddened 2 of 7 carriers and for LLR-xyv6p9 2 of 5. Rather than
report a subset and move on, T3 mutated until every named carrier had been
individually reddened — both length bounds to 256 and to 127, the device-slot
comparison inverted, the empty list rejected.

One carrier set remains proven by a single clause: **LLR-xzqs9r's ten carriers
rest on the uppercase mutation alone.** The other nine assert disjoint clauses
(empty, dot, script-mixing, NFC, hyphen, length) that a narrow mutation does not
reach. Recorded as a known limit of this round's evidence, not as a pass.

A second thing worth keeping: **LLR-5w2jx8's three carriers red through
`find_confusable_pair()`'s setup probe, not through the `ConfusableHandle`
assertion.** The probe is itself an assertion over `handle_skeleton`, so the red
is for the mutated reason — but it is a setup panic, and the test does not fail
at the line a reader would expect. Worth knowing before someone reads that
failure in anger.

---

### T4 — SDD-d6x85b: hashing and the device sub-trie

**Files touched:** `org-members/tests/integration_test.rs`
**Parallel:** no (serial, after T3)

Two LLRs, **both needing a new test**. These are genuine red-first tests: write
them, watch them fail against a deliberately wrong expectation first, then
assert the real behaviour.

```rust
/// verifies: LLR-72p8bz
///
/// The four hash domains are separated: the same bytes hashed as a member leaf,
/// a member node, a device leaf and a device node yield four different values.
/// Without separation, a device key could be presented as a member leaf.
#[test]
fn hasher_domains_are_separated() {
    let input = [7u8; 32];
    let node = NodeHash::new(input);

    let member_leaf = Blake3Hasher::hash_member_leaf(&input);
    let device_leaf = Blake3Hasher::hash_device_leaf(&input);
    let member_node = Blake3Hasher::hash_member_node(&node, &node);
    let device_node = Blake3Hasher::hash_device_node(&node, &node);

    let all = [member_leaf, device_leaf, member_node, device_node];
    for i in 0..all.len() {
        for j in (i + 1)..all.len() {
            assert_ne!(all[i], all[j], "domains {i} and {j} collide");
        }
    }
}

/// verifies: LLR-kdhd2v
///
/// Device keys are held sorted, so the order they are supplied in does not
/// change the member record's contribution to the root.
#[test]
fn device_slot_order_does_not_change_the_root() {
    let d1 = device_key("dev-a");
    let d2 = device_key("dev-b");

    let forward = MemberLeaf::new(
        member_id("m"), "alice".into(), member_key("k"),
        "Alice".into(), "Anderson".into(), vec![d1, d2],
    ).unwrap();
    let reverse = MemberLeaf::new(
        member_id("m"), "alice".into(), member_key("k"),
        "Alice".into(), "Anderson".into(), vec![d2, d1],
    ).unwrap();

    let a = OrgTrie::<Blake3Hasher>::genesis(vec![forward]).unwrap()
        .recalculate().unwrap().0.root_hash().unwrap();
    let b = OrgTrie::<Blake3Hasher>::genesis(vec![reverse]).unwrap()
        .recalculate().unwrap().0.root_hash().unwrap();

    assert_eq!(a, b);
}
```

`MemberLeaf::new`'s exact signature and the `device_key` / `member_id` /
`member_key` helpers are already in `integration_test.rs` (lines 10–30); match
them rather than the sketch above if they differ. `NodeHash` and `Blake3Hasher`
need importing.

**Red:** run each before the behaviour it asserts is relied upon —
`hasher_domains_are_separated` reds by giving all four `TrieHasher` methods the
same body; `device_slot_order_does_not_change_the_root` reds by removing the
sort in `P2pDeviceSlots::new`.

**Expected output:** `cargo test -p org-members` reports **112 passed**, 0
failed — the two tests above plus the third below.

#### A third test, added after T3 measured the coupling is unguarded

T3 reported that raising `MAX_DEVICES` from 4 to 5 **compiles cleanly**. Read
the two functions together and that is a latent defect, not a curiosity:

```rust
// types.rs
pub(crate) fn to_fixed_slots(&self) -> [Option<P2pDeviceKey>; MAX_DEVICES]

// device_trie.rs
let slots = devices.to_fixed_slots();
let leaves: [_; 4] = core::array::from_fn(|i| match slots[i] { … });
```

`to_fixed_slots` is sized by the constant; `compute_device_root` is sized by the
literal `4`. They agree today by coincidence of the two numbers matching, and
nothing makes them agree. Raise the constant and the fifth device key is
**never hashed** — it would live in the member record, serialise, and come back
from `p2p_devices()`, while the membership root stayed unchanged. A device
invisible to the root is a device invisible to every verifier downstream of it.

LLR-kdhd2v is the item that asserts this coupling ("a fixed depth-2 binary tree
of four slots"), so T4 is where it gets proven. Add:

```rust
/// verifies: LLR-kdhd2v
///
/// Every one of the MAX_DEVICES slots reaches the root. `to_fixed_slots` is
/// sized by MAX_DEVICES while `compute_device_root` is sized by the literal 4;
/// nothing but this test couples them, and a device that does not reach the
/// root is invisible to every verifier.
#[test]
fn every_device_slot_reaches_the_root() {
    let base: Vec<P2pDeviceKey> = (0..MAX_DEVICES)
        .map(|i| device_key(&format!("dev-{i}")))
        .collect();

    let root_of = |devices: Vec<P2pDeviceKey>| {
        let leaf = MemberLeaf::new(
            member_id("m"), "alice".into(), member_key("k"),
            "Alice".into(), "Anderson".into(), devices,
        ).unwrap();
        OrgTrie::<Blake3Hasher>::genesis(vec![leaf]).unwrap()
            .recalculate().unwrap().0.root_hash().unwrap()
    };

    let all = root_of(base.clone());

    // Vary each slot in turn; every one must move the root.
    for i in 0..MAX_DEVICES {
        let mut varied = base.clone();
        varied[i] = device_key("replacement");
        assert_ne!(all, root_of(varied),
            "device slot {i} does not reach the root");
    }
}
```

Note the devices are sorted by `P2pDeviceSlots::new`, so "slot i" is a position
in the sorted set rather than in the supplied vector; the test does not depend
on which, only that varying any member of the set moves the root.

**Red it the way that matters:** raise `MAX_DEVICES` to 5 in `types.rs` and
watch this test fail on the fifth slot. That is the exact defect it guards, and
it is the reason the test is worth more than a comment. Revert and verify
byte-identical as usual.

**Corrected after T4 measured it.** This paragraph said "while the suite
otherwise stays green", and that is false: the mutation reds **four** tests.
`add_p2p_device_rejects_when_full`, `member_leaf_too_many_devices` and
`deserialize_rejects_too_many_devices` fail too, because each hardcodes "a fifth
device must be rejected" and the *cap* moved. The substance survives the
correction, and it is worth stating precisely because T9 quotes it: none of
those three observes that the fifth key goes unhashed. They test the bound.
`every_device_slot_reaches_the_root` is the only test that detects the defect
itself, and the only one whose failure message names it — `device slot 4 does
not reach the root`, with both roots identical.

This adds a test, not production code. **Do not "fix" the coupling** by editing
`compute_device_root` or adding a `const` assertion — a guard is a behaviour
change, it needs its own red-first cycle, and it does not belong in an
architecture change. T9 records the finding.

#### T4 — DONE (task branch `…-t4` at `b404118`, merged into the change branch)

Three tests added, **112 passed, 0 failed**, no warnings. All three red-first:

```
hasher_domains_are_separated          all four Blake3Hasher contexts set to the
                                      member-leaf context -> "domains 0 and 1
                                      collide"
device_slot_order_does_not_change…    devices.sort() removed from
                                      P2pDeviceSlots::new -> 0b30d9c3… != 6782b3bf…
every_device_slot_reaches_the_root    MAX_DEVICES 4 -> 5 -> "device slot 4 does
                                      not reach the root", both roots 72633fdc…
```

Every mutation reverted and verified byte-identical by SHA-256.

**This plan's test sketches did not compile.** `MemberLeaf::new` takes `&str`
for handle, name and surname, not `String`, so the sketch's `"alice".into()` was
wrong; the real signature is
`MemberLeaf::new(MemberId, &str, P2pMemberKey, &str, &str, Vec<P2pDeviceKey>)`.
T4 used it, and the file's existing `TestTrie` alias in place of the sketch's
spelled-out `OrgTrie::<Blake3Hasher>`. The helpers were as described.

**The coupling is left unfixed, verified rather than asserted.** `types.rs` and
`device_trie.rs` are untouched in the commit, both source hashes identical to
their pre-mutation values, no `const` assertion added. The defect stands as
T9's finding.

---

### T5 — SDD-d9svdj: the sparse Merkle store

**Files touched:** `org-members/tests/integration_test.rs`
**Parallel:** no (serial, after T4)

Six LLRs; two need a new test.

| LLR | Carrier test(s) | Mutation / red |
|---|---|---|
| LLR-zbe553 | **new**: `member_id_bit_indexes_msb_first` | new test — red first |
| LLR-wm5hpc | `genesis_empty_is_ok`, **new**: `add_then_delete_returns_to_the_empty_root` | new test — red first |
| LLR-tk4qxu | `insert_does_not_mutate_original`, `delete_does_not_mutate_original` | in `smt::insert`, mutate the node in place instead of path-copying |
| LLR-n7nya3 | `root_hash_errs_until_recalculated`, `batch_mutations_then_recalculate` | make `root_hash()` return the stale cached value rather than `HashesNotCalculated` |
| LLR-4n8zqx | `same_members_same_root_hash`, `different_insertion_order_same_root` | mix insertion order into the leaf hash |
| LLR-8jttpb | `calculate_delta_returns_removed_and_upserted_leaves`, `calculate_delta_empty_when_tries_identical` | in `smt::diff_tries`, descend right before left |

The two new tests:

```rust
/// verifies: LLR-zbe553
///
/// Index 0 is the most significant bit of byte 0, and index 255 the least
/// significant bit of byte 31. The SMT traverses by this, so the convention is
/// the addressing scheme, not a detail.
#[test]
fn member_id_bit_indexes_msb_first() {
    let mut bytes = [0u8; 32];
    bytes[0] = 0b1000_0000;
    bytes[31] = 0b0000_0001;
    let id = MemberId::new(bytes);

    assert!(id.bit(0), "index 0 must be the MSB of byte 0");
    assert!(!id.bit(1));
    assert!(id.bit(255), "index 255 must be the LSB of byte 31");
    assert!(!id.bit(254));
}

/// verifies: LLR-wm5hpc
///
/// Every level's empty-subtree hash is precomputed, so a trie emptied of its
/// members is indistinguishable from one that never held any.
#[test]
fn add_then_delete_returns_to_the_empty_root() {
    let empty = OrgTrie::<Blake3Hasher>::genesis(vec![]).unwrap()
        .recalculate().unwrap().0.root_hash().unwrap();

    let populated = OrgTrie::<Blake3Hasher>::genesis(vec![]).unwrap()
        .add_member(alice()).unwrap();
    let emptied = populated.delete_member(alice().id()).unwrap()
        .recalculate().unwrap().0.root_hash().unwrap();

    assert_eq!(empty, emptied);
}
```

`member_id_bit_indexes_msb_first` reds by changing `bit`'s `7 - (index % 8)` to
`index % 8`. `add_then_delete_returns_to_the_empty_root` reds by having
`smt::remove` leave a tombstone leaf rather than restoring the level default.

**Expected output:** `cargo test -p org-members` reports **114 passed**, 0
failed.

#### T5 — DONE (task branch `…-t5` at `648ad85`, merged into the change branch)

Two tests added, six LLRs annotated, **114 passed, 0 failed**, no warnings.
`MISSING-TEST` 27 → 19.

**This plan's arithmetic was one low from T5 onward, and T5 caught it.** T4
added three tests where this plan originally specified two, so the baseline
entering T5 was 112, not 111. The figures written for T5 (113) and for T6–T8
(114) were all one short. Corrected here and below: T5 lands on **114**, T6 on
**115**, T7 and T8 on **115** since neither adds a test.

Red → green, each reverted and verified byte-identical:

```
member_id_bit_indexes_msb_first        bit()'s 7-(index%8) -> index%8
add_then_delete_returns_to_the_empty…  smt::remove writes a tombstone
root_hash_errs_until_recalculated      root_hash() falls back to the stale root
batch_mutations_then_recalculate       recalculate_hashes stops calling set_hash
same_members_same_root_hash            an insertion counter XORed into device_root
different_insertion_order_same_root    (same mutation)
```

##### Two measured negatives, neither papered over

**LLR-tk4qxu could not be reddened, and the item was wrong rather than the
evidence missing.** See the narrowing note in the ledger: the structural-sharing
clause is invisible at the item's interface, and what remains is guaranteed by
the type system — `Node` has no interior mutability but its write-once hash
cell, and every operation takes `&self`. T5 ran the nearest compilable surrogate
anyway (return the shared node rather than path-copying when it already has a
hash): fifty-plus tests reddened, both carriers stayed green, and
`delete_does_not_mutate_original` reddened only for an unrelated setup reason.
The task did not invent a test to make the negative disappear. **This is the
right outcome**, and the row in the verification record should read as a
type-level guarantee, not as a gap.

**LLR-8jttpb's two named carriers are both blind to the order it fixes.**
Descending right before left in `diff_recursive` leaves
`calculate_delta_returns_removed_and_upserted_leaves` green (one removal, one
upsert — no order to observe) and `calculate_delta_empty_when_tries_identical`
green (empty). The mutation *is* caught, by two tests this plan never named:
`apply_delta_rejects_unsorted_removed` and `fuzz_tests::delta_canonicality_fuzz`.
T5 annotated the first and left a comment saying why. Two consequences:
**T7 edits that same annotation block** (the test is also LLR-xmpqn2's carrier),
and **T8 should add LLR-8jttpb to `delta_canonicality_fuzz`**, which lives in
the file only T8 touches.

##### Three things worth keeping

- **`genesis_empty_is_ok` is a weak carrier for LLR-wm5hpc.** It asserts
  `member_count() == 0` and `is_calculated()` — nothing about a hash. The
  tombstone mutation reddens six tests and that is not one of them. The real
  carrier is `add_then_delete_returns_to_the_empty_root`.
- **LLR-n7nya3's named mutation reds one of its two carriers.** The second
  clause needed a supplementary mutation, and that one is blunt: forty tests.
- **Mutation runs harvested seeds into
  `org-members/tests/fuzz_tests.proptest-regressions`.** T5 deleted it rather
  than commit regression seeds captured from deliberately broken code. Every
  later task must do the same — a committed seed file from a mutation round
  pins the suite to behaviour that never existed.

---

### T6 — SDD-k5wa4n: membership operations

**Files touched:** `org-members/tests/integration_test.rs`
**Parallel:** no (serial, after T5)

Eleven LLRs; one needs a new test.

| LLR | Carrier test(s) | Mutation / red |
|---|---|---|
| LLR-ub6dw9 | `get_by_handle`, `contains_handle` | stop updating `handle_index` in `update_leaf` |
| LLR-fv75ec | `insert_duplicate_id_fails`, `insert_duplicate_handle_different_id_fails`, `insert_rejects_confusable_handle` | drop the skeleton check in `insert_leaf` |
| LLR-ch2pkw | `genesis_duplicate_id_fails`, `genesis_duplicate_handle_different_id_fails`, `genesis_rejects_confusables`, `genesis_multiple_members` | drop the duplicate-id check in `genesis` |
| LLR-j4d38d | **new**: `delete_member_frees_the_handle_for_reuse` | new test — red first |
| LLR-4phmjf | `add_p2p_device_adds_a_device`, `add_p2p_device_rejects_duplicate`, `add_p2p_device_rejects_when_full` | have `add_p2p_device` also replace the member key |
| LLR-s97ywt | `delete_p2p_device_removes_and_rotates_key`, `delete_p2p_device_last_device_isolates` | have `delete_p2p_device` keep the old member key |
| LLR-w92psx | `emergency_isolate_member_removes_all_devices_and_rotates_key`, `_keeps_member_in_trie`, `_then_readd_device_unisolates`, `_nonexistent_fails` | have `emergency_isolate_member` delete the member instead of isolating |
| LLR-k89ahd | `rotate_p2p_key_changes_only_key`, `rotate_p2p_key_nonexistent_fails` | have `rotate_p2p_key` also clear the device set |
| LLR-mmst86 | `update_handle_rejects_invalid`, `update_handle_rejects_collision`, `update_rejects_confusable_handle`, `update_handle_renames_member` | skip `validate_handle` in `update_handle` |
| LLR-g6arcs | `update_name_surname_nfc_normalizes`, `update_name_surname_changes_pii`, `update_name_surname_rejects_oversized_name`, `_oversized_surname` | drop the NFC normalisation in `update_name_surname` |
| LLR-v3jqau | `update_handle_nonexistent_fails`, `update_name_surname_nonexistent_fails`, `rotate_p2p_key_nonexistent_fails`, `add_p2p_device_nonexistent_member_fails`, `delete_p2p_device_nonexistent_member_fails`, `emergency_isolate_member_nonexistent_fails`, `delete_nonexistent_fails` | have `update_leaf` insert a new member when the id is absent |

The new test:

```rust
/// verifies: LLR-j4d38d
///
/// Deleting a member removes their handle and skeleton from the indexes, so a
/// later member may take the handle. The doc comment on `delete_member` has
/// claimed this since the operation was written; nothing tested it.
#[test]
fn delete_member_frees_the_handle_for_reuse() {
    let trie = OrgTrie::<Blake3Hasher>::genesis(vec![alice()]).unwrap();
    let after = trie.delete_member(alice().id()).unwrap();

    // A different member, same handle as the departed one.
    let successor = MemberLeaf::new(
        member_id("successor"), "alice".into(), member_key("s"),
        "Alicia".into(), "Brown".into(), vec![device_key("s-dev")],
    ).unwrap();

    assert!(after.add_member(successor).is_ok(),
        "the departed member's handle must be available again");
}
```

It reds by having `delete_by_id` skip the two index removals — which is the
mutation, so this LLR is discharged by the new test's own red rather than by a
separate mutation round.

**Expected output:** `cargo test -p org-members` reports **115 passed**, 0
failed. (Was written as 114 before T5 corrected this plan's arithmetic.)

#### T6 — ANNOTATIONS DONE, ATTESTATIONS OWED (task branch `…-t6` at `239d4ee`, merged)

**The work is complete; the evidence for it is not.** The dispatched subagent
stalled twice — the harness watchdog killed it at 600s of no progress, both
times — and it had batched an entire run of mutation rounds into an uncommitted
working tree. It committed nothing before the first stall, and its context, which
was the only holder of the `red -> green:` attestations, died with it.

What is established, by inspection rather than by report:

```
source tree            byte-identical to the change branch (sha256, all ten
                       src/*.rs) — no mutation was left live by the kill
suite                  0 + 6 + 108 + 1 + 0 = 115 passed, 0 failed
MISSING-TEST           19 -> 8, the eleven cleared being exactly T6's eleven
new test               delete_member_frees_the_handle_for_reuse, using the real
                       signatures (TestTrie, &str) rather than this plan's sketch
```

The dispatcher committed that work on the task branch and merged it, after
verifying the source tree was unmutated. Losing correct annotations to a third
stall would have been the worse outcome.

**What is owed: eleven mutation attestations.** Annotating a test that was
already green proves nothing on its own — that is this change's whole method, and
it is the one thing a stalled subagent cannot hand over. The eleven LLRs are
LLR-ub6dw9, LLR-fv75ec, LLR-ch2pkw, LLR-j4d38d, LLR-4phmjf, LLR-s97ywt,
LLR-w92psx, LLR-k89ahd, LLR-mmst86, LLR-g6arcs, LLR-v3jqau.

The last thing the subagent said before the first stall was "Supplementary for
the third carrier of LLR-fv75ec", which suggests it had worked through most of
the eleven. **That is not evidence and is not treated as any.** An attestation
names a mutation, a test, and a failure read and understood; a fragment of a
progress message names none of them. These are re-established or they are
recorded as absent.

**Why it stalled, for whoever plans the next unit.** T6 was the largest task in
the change — eleven LLRs against thirty-one carrier tests, each needing a
mutation, a test run and a revert. The five tasks that committed as they went
lost nothing to interruption; the one that batched lost an hour. Cut a task this
size in two.

#### T6a — DONE (five of the eleven; branch `…-t6a`, zero commits, removed)

The owner's decision on 2026-09-16 was to re-establish the eleven in two smaller
dispatches. T6a took five. It added nothing and committed nothing — correct,
since the product of a mutation round is the attestation, not a diff.

```
LLR-ub6dw9  get_by_handle, contains_handle — drop handle_index.insert in genesis
LLR-fv75ec  insert_duplicate_handle_different_id_fails, insert_rejects_confusable_handle
              — drop the skeleton check in insert_leaf
            insert_duplicate_id_fails — drop the id check in insert_leaf
LLR-ch2pkw  genesis_duplicate_id_fails — drop genesis's duplicate-id check
            genesis_duplicate_handle_different_id_fails, genesis_rejects_confusables
              — drop genesis's skeleton check
            genesis_multiple_members — drop genesis's handle_index.insert
LLR-j4d38d  delete_member_frees_the_handle_for_reuse — delete_by_id skips both
              index removals
LLR-4phmjf  add_p2p_device_adds_a_device — add_p2p_device also replaces the key
            add_p2p_device_rejects_duplicate — drop has_device in add_device
            add_p2p_device_rejects_when_full — drop the MAX_DEVICES check
```

All reverted, every `shasum -a 256` identical, suite 115 passed at the end,
source tree verified pristine against the change branch.

##### This plan's mutation column is weaker than it looks, and T6a proved it

**LLR-ub6dw9's named mutation reds none of its carriers.** The plan said "stop
updating `handle_index` in `update_leaf`". Neither `get_by_handle` nor
`contains_handle` ever calls an update path — both build a trie with `genesis`
and read it. The mutation is unreachable from the carriers, so it discriminates
nothing, and a run that stopped at the named mutation would have recorded a
measured negative for an LLR that is in fact well covered. The supplementary
(drop the insert in `genesis`) reds both.

**LLR-4phmjf's named mutation reds one of three**, and LLR-fv75ec's and
LLR-ch2pkw's red 2-of-3 and 1-of-4 respectively. Across T3, T5 and T6a the same
defect keeps appearing: **this plan pairs one mutation with a whole carrier
list, when the carriers assert different clauses and are reached by different
paths.** The mutation column should name, per carrier, a mutation that carrier
actually executes. Written as it is, the table invites exactly the false
confidence the attestations exist to prevent — which is why every task has been
told to treat it as a suggestion and measure for itself.

##### One carrier reds through a crash, not through its assertion

`add_p2p_device_rejects_when_full`, with the bound removed, takes the fifth
device successfully and then panics inside the fixed four-slot encoding —
`index out of bounds: the len is 4 but the index is 4` at `types.rs:421` —
before its own `assert_eq!(err.unwrap_err(), DeviceSlotsFull)` runs. The red is
unambiguously caused by the mutation, so the attestation stands. But the test
detects a removed bound only as a crash, and that is the same
`MAX_DEVICES`-versus-literal-`4` coupling T3 found and T4 gated: the slot array
is sized by the constant, the encoding by the literal. Worth recording next to
that finding in T9.

#### T6b — DONE (the other six; branch `…-t6b`, zero commits, removed)

Fourteen mutation rounds, every one reverted and SHA-256 verified before the
next. Suite 115 passed at the end, source tree pristine, nothing committed.
**All eleven owed attestations are now re-established and no carrier was left
unproven.**

```
LLR-s97ywt  delete_p2p_device keeps the old key        (key clause, 2 carriers)
            delete_p2p_device keeps the original slots (removal/isolation clause)
LLR-w92psx  isolate calls delete_by_id                 (3 of 4 carriers)
            isolate keeps the device slots             (2 carriers)
            isolate keeps the old key                  (2 carriers)
LLR-k89ahd  rotate_p2p_key also clears the device set
            rotate_p2p_key keeps the old key
LLR-mmst86  skip validate_handle in update_handle      (1 of 4 carriers)
            drop update_leaf's rename skeleton check   (2 carriers)
            disable update_leaf's rename branch        (1 carrier)
LLR-g6arcs  drop NFC normalisation                     (1 of 4 carriers)
            disable both length bounds                 (2 carriers)
            write the name but keep the old surname    (1 carrier)
LLR-v3jqau  all seven presence guards return Ok        (all 7 carriers)
            all eight guard sites return DuplicateId   (all 7 carriers)
```

##### A second named mutation that reaches nothing — and this one is worse

**LLR-v3jqau's named mutation reds 0 of 7 carriers.** This plan said "have
`update_leaf` insert a new member when the id is absent". Every public operation
performs its own `smt::get_member(…).ok_or(IdNotFound)?` before `update_leaf` is
ever reached, and `delete_member` routes through `delete_by_id`, which guards
itself. **`update_leaf`'s own guard is dead code from the perspective of all
seven tests** — the mutation is unreachable, and a run that stopped there would
have recorded seven carriers as unproven when all seven discriminate perfectly
well.

T6b then split the LLR's two clauses and proved each across all seven: make the
guards return `Ok` (not refused at all), and make them return `DuplicateId`
(refused, wrong error). That is the shape this plan's mutation column should
have had throughout.

**Two of eleven named mutations in T6 reached nothing at all** (LLR-ub6dw9,
LLR-v3jqau) and three more reached one carrier of three or four (LLR-4phmjf,
LLR-mmst86, LLR-g6arcs). Recorded here as a plan defect, not a code one: the
code is well covered, and it took a supplementary mutation per clause to show
it.

##### Two more weak carriers, recorded

- **LLR-w92psx**: under the named mutation only `_keeps_member_in_trie` reds
  through its own assertion (`member_count` 1 vs 2). The other two red through
  an `unwrap()` on the now-absent member — a crash. The attestation stands, but
  those two carriers do not themselves assert member retention.
- **LLR-g6arcs**: `update_name_surname_changes_pii` exercises neither NFC nor
  the bounds, only the write-through of both fields. It is a carrier for this
  LLR in name more than in substance.

---

### T7 — SDD-55b2zj: delta exchange

**Files touched:** `org-members/tests/integration_test.rs`
**Parallel:** no (serial, after T6)

Six LLRs, all carried by existing tests.

| LLR | Carrier test(s) | Mutation to red it |
|---|---|---|
| LLR-au8het | `delta_base_mismatch_fails`, `apply_delta_stale_delta_fails` | drop the `base_root` comparison in `apply_delta` |
| LLR-y38jfk | `delta_apply_and_verify` | have `apply_delta` return the trie directly |
| LLR-7tdqv9 | `candidate_verify_wrong_root_fails` | have `verify_against` ignore `expected_root` |
| LLR-xmpqn2 | `apply_delta_rejects_stale_removal`, `_unsorted_removed`, `_duplicate_in_removed`, `_duplicate_in_upserted`, `_unsorted_upserted`, `_id_in_both_removed_and_upserted`, `_noop_upsert`, `_canonical_delta_still_works` | drop the strictly-increasing check on `removed` in the canonical-form check |
| LLR-juxk9q | `apply_delta_rejects_confusable_in_upsert` | skip the skeleton check on upserted leaves |
| LLR-h7stq2 | `calculate_delta_then_apply_roundtrips`, `calculate_delta_fails_when_hashes_not_calculated`, `calculate_delta_reversed_args_produces_inverse_delta`, `apply_delta_to_wrong_side_after_reversed_calc_fails` | drop the hashes-calculated check in `calculate_delta` |

LLR-xmpqn2 has eight carriers and one mutation only reds a subset. Run the whole
`integration_test` target for it and report **which** of the eight reddened — a
per-LLR count, not a per-test one. If any of the eight stays green under every
one of the four canonical-form mutations (increasing-removed,
increasing-upserted, disjointness, no-op upsert), say so: that is a measured
negative about that test, and it belongs in the report.

**Also carry LLR-8jttpb here.** T5 found that item's two named carriers are both
blind to the traversal order it fixes, and that
`apply_delta_rejects_unsorted_removed` — which is LLR-xmpqn2's carrier, so T7
edits its annotation block anyway — is one of only two tests that catch the
mutation. T5 has already annotated it. Do not remove that annotation while
adding LLR-xmpqn2 to the same line.

**Expected output:** `cargo test -p org-members` reports 115 passed, 0 failed —
no test is added.

#### T7 — DONE (task branch `…-t7` at `fea1866`, six commits, merged)

Thirteen mutations across six LLRs, committed per LLR rather than batched.
115 passed, source verified pristine, **`MISSING-TEST` down to two** — LLR-h9gs32
and LLR-sa3ugj, both T8's.

LLR-xmpqn2's eight carriers were proven per clause, which is what this plan
should have specified from the start:

```
strictly-increasing removals   _unsorted_removed, _duplicate_in_removed
removals present in the record _stale_removal
strictly-increasing upserts    _unsorted_upserted, _duplicate_in_upserted
disjointness                   _id_in_both_removed_and_upserted
no-op upserts                  _noop_upsert
acceptance                     _canonical_delta_still_works
```

##### Five measured negatives

Two are **type-level guarantees** and are recorded in the ledger beside the
items: LLR-y38jfk's "exposes no member query" and LLR-7tdqv9's "the candidate is
consumed either way". The plan's mutation for the first is a signature change
producing seven `E0599` compile errors, which is not a discriminating red. These
clauses stay in their items — unlike LLR-tk4qxu's withdrawn clause, they
describe the interface rather than internal structure, and a compiler-enforced
interface property is stronger than a test, just not *test* evidence.

**LLR-juxk9q's handle-uniqueness half has no carrier**, verified independently:
all three `DuplicateHandle` assertions in the suite reach the check through
`genesis`, `add_member` or `update_handle`, never through `apply_delta`. The
confusability clause is carried; the uniqueness clause is not. Recorded in the
ledger as a gap rather than closed — this change adds no test beyond the five
its plan names.

Two more, both about carriers rather than code: `_canonical_delta_still_works`
stays green under all five *relaxing* canonical-form mutations (relaxing a check
cannot break an honest delta — it reds only under an over-strict polarity flip),
and `apply_delta_to_wrong_side_after_reversed_calc_fails` is carried by the
`base_root` mutation rather than the diff-direction one, since it never inspects
delta contents.

`_duplicate_in_removed` and `_stale_removal` red because the error degrades to
`InvariantViolated` inside the apply loop, not because the canonical check
returned `MalformedDelta` — the mutation is the cause, but the discriminating
assertion is not what fails.

---

### T8 — SDD-m9gs5g: error reporting

**Files touched:** `org-members/tests/integration_test.rs`,
`org-members/tests/fuzz_tests.rs`, `org-members/tests/mbt_conformance.rs`
**Parallel:** no (serial, after T7)

Two LLRs.

| LLR | Carrier test(s) | Mutation to red it |
|---|---|---|
| LLR-h9gs32 | the two unannotated `proptest!` blocks in `fuzz_tests.rs` (lines 226, 374), plus the two already annotated `REQ-ds8ryr` | replace a `Result` return in `insert_leaf` with an `unwrap` on the duplicate path — note this also breaks the crate's own `deny(clippy::unwrap_used)`, so report both the test failure and the lint failure |
| LLR-sa3ugj | `malformed_delta_error_displays_reason`, `field_too_long_error_displays_field_and_max` | make `MalformedDelta`'s `#[error]` string drop its `{0}` |

**Also annotate `mbt_conformance.rs`.** Its single test runs the crate against
`quint/membership.qnt` and carries no `verifies:` line at all — 310 lines of
model-based conformance tracing to nothing. It verifies the membership
operations as a whole, so annotate it `verifies: LLR-fv75ec, LLR-ch2pkw,
LLR-j4d38d, LLR-v3jqau` **only for those it genuinely exercises**. Read the
model first and annotate what it covers; if it covers fewer, annotate fewer and
say which and why in the report. Do not annotate an LLR the model does not
reach in order to fill the table.

**And add LLR-8jttpb to `delta_canonicality_fuzz`.** T5 measured that the
left-then-right traversal order is caught by exactly two tests, and this is the
one that lives in a file only T8 touches. Confirm it reds under the mutation
(descend right before left in `diff_recursive`) before annotating it — T5
observed the red, but observing it yourself is the point of the protocol.

**Delete `org-members/tests/fuzz_tests.proptest-regressions` if a mutation run
creates one.** T5 had to. A seed harvested from deliberately broken code, left
committed, pins the suite to behaviour that never existed.

**Expected output:** `cargo test -p org-members` reports 115 passed, 0 failed.

#### T8 — DONE (task branch `…-t8` at `0d0855b`, four commits, merged)

Eleven mutations, 115 passed, source pristine, `proptest-regressions` created by
three runs and deleted each time. **`MISSING-TEST` 0** — every one of the
thirty-five LLRs is carried.

**LLR-8jttpb confirmed independently of T5, and the failure is sharper than
expected.** Reversing the traversal makes `delta_canonicality_fuzz` fail on
`mutator = ReverseUpserted` with "apply_delta accepted a non-canonical delta" —
because the honest delta now emerges in *decreasing* id order, so the fuzzer's
reversal restores canonical order. That is the increasing-by-construction clause
failing through the test's own path, not through a crash.

##### `mbt_conformance` is annotated with three of the four candidates, not four

**LLR-ch2pkw is not reached by the model, and the reason is structural rather
than a sampling accident.** Removing genesis's identifier, handle *and* skeleton
guards leaves `membership_conformance` green: the model's `init` is the empty
map and both driver call sites are `Trie::genesis(Vec::new())`, so genesis's
per-member loop body never executes. More samples cannot change that. The test
carries LLR-fv75ec, LLR-j4d38d and LLR-v3jqau, and the reason for the fourth's
absence is written into the annotation block in the file rather than only into a
commit message — which is where a later reader will look.

##### The `unwrap` mutation is two kinds of evidence, not one

The plan's mutation for LLR-h9gs32 replaces a `Result` return with an `unwrap`,
and the brief asked whether that discriminates the test or merely the lint. T8
measured both halves: `cargo test` does not run clippy, so the mutation compiles
and three carriers red on the real behaviour change — `called Result::unwrap()
on an Err value: DuplicateId`, a rejection reaching a panicking path. Separately
`cargo clippy -p org-members --lib` errors citing `lib.rs:2
#![deny(clippy::unwrap_used)]`. **The clippy failure is compile-time evidence for
the item's lint-posture clause and proves nothing about the tests; the test reds
are what discriminate the carriers.** They coincide here, so the mutation is not
"just the lint" — but they are separate claims and the record keeps them apart.

The same plan defect appeared once more: the mutation reaches 3 of 4 carriers.
`handle_validation_never_panics` only calls `validate_handle` and never reaches
`add_member`; a second mutation (panic in the uppercase branch) reds it on input
`"Ę"`.

##### Carriers that red through a crash

`calculate_delta_roundtrip` and `trie_ops_never_panic_and_count_consistent` red
via the panic propagating out of `trie.rs` rather than via their own
`prop_assert`s. For a panic-freedom clause that is the assertion in substance —
but it should be read as "any panic fails this", not as a targeted check, and
`calculate_delta_roundtrip`'s own assertions concern the diff roundtrip, not
error reporting. It carries LLR-h9gs32 on the panic-freedom clause alone.

`membership_conformance` under the handle-guard mutation reds through the
driver's own `expect(…)` at `mbt_conformance.rs:149` — a harness crash inside
`commit()`, before the model/implementation comparison runs. The other three mbt
mutations red properly with "Specification and implementation states diverge".

##### An observation T8 recorded and correctly did not act on

`mbt_conformance.rs:278–301` implements a root-hash **equality-class** check:
equal model state ⇔ equal real root, and distinct model states never share a
root. That is direct evidence for **LLR-4n8zqx**, in a form nothing else in the
suite provides. T8 did not annotate it, because the brief capped that test at
its four named candidates and a fifth unmeasured annotation is exactly the
unearned evidence it was told to avoid. **Nothing is owed here** — LLR-4n8zqx is
already carried and proven by T5 through `same_members_same_root_hash` and
`different_insertion_order_same_root`. This is a stronger carrier going unused,
not a gap. Worth a measured follow-up in a later change.

---

### T9 — Derived-LLR assessments and the discharged notes

**Files touched:**
`org-members/docs/risk/DRAFT-worktree-guardrails-org-members-arch-design-derived.md`,
`org-members/docs/architecture/README.md`
**Parallel:** no (serial, after T8)

Its file set is disjoint from every other task, but its content is not: it
records the measured negatives T3–T8 produce, so it cannot be written until they
have reported.

Create the draft risk file with a `## Derived requirements assessment` section
assessing each of the twelve derived LLRs by name: does it introduce a new
hazard, affect an existing hazardous situation, or change a risk control's
effectiveness? Name the HAZ or RC where it does.

Expected shape of the substantive ones, to be argued rather than asserted:

- **LLR-8jttpb** (left-then-right diff order) and **LLR-xmpqn2** together make
  the encoding unique. That bears directly on **HAZ-y8h835** and **HAZ-h58jn6**
  and on **RC-9z65hw**; the assessment must say whether the control's
  effectiveness depends on the traversal order being what this LLR fixes.
- **LLR-72p8bz** (domain separation) bears on whether a device key can be
  presented as a member record — assess against **HAZ-bmv7cy**.
- **LLR-4czn8t** (PII redaction in `Debug`) is a confidentiality control that no
  hazard in the register currently names. Either it maps to one, or the
  assessment says the register has no hazard for PII disclosure through
  diagnostics — which is itself a finding worth recording.
- **LLR-ub6dw9** (the indexes) is where the uniqueness and confusability
  controls actually execute, so **RC-n2taat**'s effectiveness rests on it.
- **LLR-w5nkbu**, **LLR-paxj7b**, **LLR-zbe553**, **LLR-wm5hpc**, **LLR-4n8zqx**,
  **LLR-k89ahd**, **LLR-g6arcs**, **LLR-h7stq2**: assess each; "no hazard impact
  because `<reason>`" is a valid outcome and several of these will reach it.

The same file also records, as prose rather than as items:

1. **LLR-pys2ek discharges REQ-xdx2c2's forward reference.** The requirement's
   note said the bound of 4 "lives only in places no gate reads" until an LLR
   gave it a controlled home. It now has one. REQ-xdx2c2's own text and note are
   **not** edited — corrections in this project are dated, not retrofitted.
2. **LLR-s97ywt is worded to the implemented behaviour, not to REQ-ewdg2q.**
   PR-zz4exm stays open, and the LLR's wording is the reason the gap does not
   disappear into a green gate.
3. **Every measured negative from T3–T8**, named, with the mutation that
   produced it.
4. **The unguarded `MAX_DEVICES` coupling** T3 measured and T4 gated.
   `to_fixed_slots` is sized by the constant, `compute_device_root` by the
   literal `4`, and nothing but `every_device_slot_reaches_the_root` couples
   them. Nothing is defective today — the two numbers agree — so this is not a
   problem report, and T9 must not mint one. It is a latent defect held shut by
   a test, and the assessment should say which hazard it would fall under if
   the coupling ever broke: a device key present in the record but absent from
   the root is a member key set the committed root does not describe, which is
   **HAZ-y8h835**'s territory rather than a new hazard. If the assessment
   concludes otherwise, that is a finding for the owner under `analyze-risks`,
   not something to mint here.

Mint no new HAZ or RC in this task. If the assessment turns up a hazard the
register does not hold — LLR-4czn8t is the candidate — that is a finding to put
to the owner under `analyze-risks`, **not** something to mint in passing at the
end of an architecture change.

#### And move the Overview to where the ledger says it lives

T2 wrote the Overview into the draft ledger, because that is what this plan
told it to do, and flagged the problem: `org-members/docs/architecture/README.md`
says in its own words "Use this README's Overview section for the system-wide
decomposition picture". The draft becomes a **dated, per-change** file at merge,
and the system-wide picture is not per-change — it is the standing description
that every later change amends. This plan put it in the wrong file.

Move the Overview text from the draft ledger into the `## Overview` section of
`org-members/docs/architecture/README.md`, replacing that section's HTML comment
placeholder, and leave in the draft a one-line pointer saying where the overview
lives. Do not otherwise edit the README: the rest of it is shipped grammar,
re-copied from the toolkit at each upgrade, and a local edit there is lost at
the next `/ratchet`.

---

### T10 — The class C robustness rule, for the eighteen one-sided LLRs

**Files touched:** `org-members/tests/integration_test.rs`,
`org-members/docs/architecture/DRAFT-worktree-guardrails-org-members-arch-decomposition.md`
**Parallel:** no (serial, after T9)

Added 2026-09-17, after `verify-before-merge` found that T3–T8 had closed
`MISSING-TEST` without closing the class C rule underneath it: every REQ/LLR
needs **both** a normal-case and an abnormal-input test, and eighteen of the
thirty-five LLRs had tests on one side only. Eight were carried by refusals
alone, ten by the happy path alone.

The owner's decision, put on 2026-09-17: **fix the annotations where the missing
side already exists in the suite, record a reason for the rest, add no new
tests.** A test already written and already green is capacity this change has
paid for; a test written now to satisfy a gate is a test written to satisfy a
gate. The rule applied to each of the eighteen in turn:

- if a test in the suite genuinely exercises the missing side, add the LLR to
  that test's `verifies:` line — never removing what is already there — and
  then prove it with a mutation that breaks *this* LLR's behaviour and reds
  *that* test;
- otherwise, annotate nothing and argue the reason in the ledger next to the
  item.

#### T10 — DONE (task branch `…-t10`, five commits, not merged)

Ten annotated with ten mutations, eight argued, **no test added**: suite
unchanged at 0 + 6 + 108 + 1 + 0 = **115 passed, 0 failed**. All ten
`org-members/src/*.rs` verified byte-identical by `shasum -a 256 -c` after the
last revert; no `proptest-regressions` file was produced (every mutation run
targeted `integration_test`). `check-trace.sh` and `check-ids.sh
--allow-draft-files` both exit 0 under `GR_CONFIG=org-members/.guardrails/config.yaml`,
with `SDD 6, LLR 35` and no finding but the standing `UNRESOLVED-PR PR-zz4exm`.

**The ten annotated.** Eight of them red at the target test's own assertion; the
two that do not are named.

| LLR | Missing side | Carrier gained | Mutation that reddened it |
|---|---|---|---|
| LLR-5w2jx8 | normal | `genesis_multiple_members` | `handle_skeleton` returns a constant — every handle becomes a confusable of every other, genesis refuses member two with `ConfusableHandle` |
| LLR-pys2ek | normal | `every_device_slot_reaches_the_root` | `MAX_DEVICES` 4 → 5 — reds at `device slot 4 does not reach the root` |
| LLR-paxj7b | normal | `member_leaf_has_id_handle_and_key` | `MemberLeaf::new`'s guard widened from `is_empty()` to `len() <= 1` — refuses the minimum valid record |
| LLR-au8het | normal | `delta_apply_and_verify` | `apply_delta`'s base-root comparison inverted — `DeltaBaseMismatch` on an honest delta |
| LLR-7tdqv9 | normal | `delta_apply_and_verify` | `verify_against`'s comparison inverted — `VerificationFailed` on a matching root |
| LLR-juxk9q | normal | `delta_apply_and_verify` | the apply-time skeleton insert moved above its own check — `DuplicateHandle`, the upsert colliding with the entry the check just wrote |
| LLR-y38jfk | abnormal | `candidate_verify_wrong_root_fails` | `verify_against`'s guard deleted — an `Ok(OrgTrie { .. })` comes back from a verification against a root that is not the candidate's |
| LLR-4n8zqx | abnormal | `add_then_delete_returns_to_the_empty_root` | `smt::remove` leaves a residue (`Node::empty` at the level-1 default) — the root of an empty organisation then depends on how it got there |
| LLR-j4d38d | abnormal | `delete_removes_member` | `new_handle_index.remove(..)` dropped from `delete_by_id` — the departed handle still resolves |
| LLR-s97ywt | abnormal | `delete_p2p_device_unknown_device_fails` | `remove_device`'s error swallowed — the member key is rotated with no device removed |

**Two carriers red through a setup panic rather than their own assertion**, and
a reader meeting those failures should know it. `member_leaf_has_id_handle_and_key`
reds inside the `alice()` helper's `unwrap()` at `integration_test.rs:37`, because
the record it reads back cannot be built at all under the mutation.
`genesis_multiple_members` reds at its own `unwrap()` of `genesis`, on line 101,
rather than at one of the `contains_handle` assertions below it.

**LLR-j4d38d's two halves were measured apart, and that is the finding worth
keeping.** The item says `delete_member` removes the handle *and* the skeleton
from the indexes. Its existing carrier,
`delete_member_frees_the_handle_for_reuse`, reaches only the skeleton half —
reuse is gated by the skeleton index, so under the handle-index mutation that
test was **measured green** while `delete_removes_member` reddened. Half an item
was uncarried and the annotation now says so.

**The eight argued, and where.** Each reason is prose in the decomposition
ledger next to its own item, not a line in this plan, because the ledger is what
the independent review at merge reads:

| LLR | Missing side | Why no annotation |
|---|---|---|
| LLR-68tka5 | normal | The valid-record round-trip exists — `deserialize_rejects_invalid_handle` opens with one — but it is *inside that same carrier*, and it is the only `MemberLeaf` deserialisation of a valid handle in the suite (`from_bytes` appears 13 times; every other one takes a `P2pDeviceSlots` or an adversarial payload). There is no second test to annotate. |
| LLR-4czn8t | abnormal | The `Debug` impl writes three fixed `[REDACTED]` literals and never reads the fields it redacts. No input can drive it down another path; the axis is empty by construction. |
| LLR-72p8bz | abnormal | The carrier *is* the adversarial case — the same 32 bytes hashed in all four domains. A hash over `&[u8]` has no invalid input. |
| LLR-kdhd2v | abnormal | The fifth device is refused by `P2pDeviceSlots` before `compute_device_root` is reached, so that evidence lives under LLR-pys2ek and LLR-xyv6p9. The degenerate case (zero or partly filled slots) is executed by several tests and asserted by none a `device_trie.rs` mutation can discriminate: each compares two roots and the mutation moves both sides. |
| LLR-zbe553 | abnormal | Both address boundaries — indices 0, 1, 254, 255 — are the carrier. Every 32-byte value is a valid id. An index ≥ 256 would panic, but no caller can supply one and a test for it would have to assert a panic, against LLR-h9gs32's posture. |
| LLR-wm5hpc | abnormal | The 257 defaults take no input; the only input axis is the member set, whose degenerate values *are* the two carriers. They read as normal-case only because what they assert is success. The `InvariantViolated` path is unreachable through the public interface. |
| LLR-tk4qxu | abnormal | The abnormal-input test is already written — `mutations_preserve_original` drives operations that may be refused — and the red is impossible for the reason T5 recorded: `&self` throughout, no interior mutability but a write-once hash cell, and a refused operation returns no trie. An annotation here could never be discharged. |
| LLR-sa3ugj | normal | Both carriers already *are* the normal case. A `Display` impl over a fixed variant has one class of input — a `&'static str` and a `usize` the crate chooses itself. The gate read them as abnormal because the values are errors. |

**T8's unused stronger carrier stays unused, deliberately.** T8 recorded that
`mbt_conformance.rs:278–301` implements a root-hash equality-class check that is
direct evidence for LLR-4n8zqx, in a form nothing else provides. T10 did not
reach for it: `add_then_delete_returns_to_the_empty_root` supplies the missing
side and was proved here, and annotating the model-based test would need its own
mutation round against a target this task had no mandate to open. The note in
T8's block still stands as the follow-up.

**What this task did not do.** It added no test, wrote no production code, minted
no ID, and did not touch PR-zz4exm. Ten items are now two-sided on evidence and
eight are two-sided on argument; the gate's finding is answered either way, and
the independent review at merge can disagree with any of the eight by reading
the paragraph that makes the claim.

---

### T11 — The fix round for the independent review's thirteen findings

**Files touched:** `org-members/tests/integration_test.rs`,
`org-members/src/delta.rs` (comment only),
`org-members/docs/architecture/2026-09-17-decomposition.md`,
`org-members/docs/architecture/soup.md`,
`org-members/docs/risk/2026-09-17-design-derived.md`,
`org-members/docs/problems/DRAFT-worktree-guardrails-org-members-arch-review-fixes.md`
(new), `.gitignore`, this plan
**Parallel:** no (serial, after the independent review at merge-change step 6a)

Added 2026-09-17. The independent review returned **thirteen findings**. The
owner's disposition the same day: **fix 1, 3, 4, 10, 12, 13; record 2, 5, 6, 7,
8, 11 as dated obligations; finding 9 raised no fault.** Mint nothing beyond one
problem report. Change no test count.

The shape of this task is the reason it exists separately from T1–T10: two of
the thirteen are **false claims in shipped documentation**, and one of those had
already been exported to another unit. A false claim is not a missing test —
nothing reddens when it is wrong, and no gate in the toolkit reads prose for
truth. The only thing that finds them is a reader, which is what the independent
review is for, and the only thing that fixes them is a correction that says what
was wrong rather than quietly replacing the sentence.

#### T11 — DONE (task branch `…-t11-review-fixes`, seven commits, not merged)

Suite unchanged at 0 + 6 + 108 + 1 + 0 = **115 passed, 0 failed** — one test's
body rewritten, none added or removed. Nine of the ten `org-members/src/*.rs`
verified byte-identical to the change branch by `shasum -a 256 -c`; `delta.rs`
differs in `///` doc-comment lines only, confirmed by reading the diff.
`check-trace.sh` exit 0 and `check-ids.sh --allow-draft-files` exit 0 under
`GR_CONFIG=org-members/.guardrails/config.yaml`, with `SDD 6, LLR 35, PR 2` and
the two expected advisories (`PR-zz4exm` 17 days, `PR-jq43gx` 0 days, against
the 30-day limit). One ID minted: **PR-jq43gx**, the one the brief allocated.
Working tree clean; no `proptest-regressions` file was produced.

**finding-1 (BLOCKING) — the postcard byte-uniqueness claim, corrected in four
places.** The claim was that canonical form makes the postcard encoding of an
accepted change set the unique byte string for a transition between two roots.
It is false, and the code says so plainly: `MemberLeaf`'s `Deserialize`
**normalises rather than rejects** — `let name = to_nfc(&raw.name);`, the same
for `surname`, and the handle stored as the NFC form `validate_handle` returns,
with the code's own comment noting "the stored handle is canonical even if the
wire payload wasn't". An NFD-encoded leaf and its NFC equivalent are two
distinct postcard byte strings decoding to one `MemberLeaf`, one `Delta` value
and one root. `P2pDeviceSlots`' `Deserialize` is the contrasting case and the
one the claim was presumably generalised from: it genuinely refuses.

| Where | What it said | What it says now |
|---|---|---|
| decomposition ledger, LLR-xmpqn2's note | "makes the postcard encoding … the *unique* byte string for a transition between two roots" | canonical **structure**; one accepted `Delta` **value**; the encoding is not injective, and why |
| `delta.rs:29–32` | "there is exactly one postcard byte string of a `Delta` that `apply_delta` will accept" | one `Delta` value; explicit "this is not byte-level uniqueness, and must not be relied on as if it were" |
| `soup.md`, postcard row | "`types.rs` deliberately *rejects* rather than normalises non-canonical wire forms so that the encoding stays injective" | true of `P2pDeviceSlots` only; `MemberLeaf` normalises; postcard trusted for determinism of encoding a given value, not for one-encoding-per-value |
| risk draft, LLR-8jttpb assessment | "**encoding uniqueness**… The properties that consume uniqueness are the signature over the encoded envelope, deduplication, and the monotonic sequence guard — and all three live in `org-node`" | the correction, plus what `org-node` must now establish for itself |

**LLR-xmpqn2's own item text was read first and is sound** — "a change set is
accepted only in canonical form: removals strictly increasing and all present in
the record, upserts strictly increasing and each observably changing the record,
and the two sets disjoint" asserts nothing about bytes. The item is untouched;
only the surrounding note was wrong.

**The fourth one is the consequential one and was treated as such.** That
sentence exported a property this crate does not have to a unit that has not
been built yet: a dedup-by-encoded-bytes or a signed-envelope replay guard in
`org-node` resting on it would rest on nothing. The corrected assessment now
names the three consumers individually and says what each must do — dedup keys
on the decoded `Delta` or on the root pair, never on the bytes; the replay guard
turns on the envelope sequence number and the base root, as RC-9z65hw's residual
already says; a signature over the encoded envelope still authenticates *those
bytes*, which is sound, but the signed bytes are not a canonical identifier for
the transition. It also withdraws the sub-argument that a retained superseded
change set is byte-identical to a recomputed one — the HAZ-h58jn6 conclusion
never depended on it and is unchanged.

**No code was changed to make the claim true, and that is recorded in all four
places as a decision.** Making the encoding injective — refusing non-NFC leaf
fields the way device slots are refused — is a behaviour change, it needs its
own red-first test, and putting it in a documentation-correction round would put
a product change where no reviewer is looking for one.

**finding-4 (BLOCKING) — the LLR-zbe553 waiver rested on a false premise, and
PR-jq43gx was filed.** `MemberId::bit(index: u16)` computes `byte_idx = index/8`
and indexes a `[u8; 32]`; `bit(256)` gives 32 and panics. `bit` is `pub` on `pub
MemberId`, re-exported from the crate root, so an **external** caller can supply
one. The waiver said "no caller can supply one (every call site is bounded by
`SMT_DEPTH`)" — true of internal call sites, false of external ones. The
paragraph now states the real situation, and the waiver's conclusion (no
abnormal-input test added) stands on the owner's deferral rather than on an
unreachability claim.

**PR-jq43gx** (opened 2026-09-17, open) is in this change's own draft problems
file. Its `affects:` line names REQ-ds8ryr, LLR-h9gs32 and LLR-zbe553, and the
report is careful that **whether `bit()` is a "membership operation" in
REQ-ds8ryr's sense is itself the open question** — an addressing primitive that
adds nothing and rejects nothing, or public input that "shall not panic, for any
input" governs. The two readings lead to different fixes (narrow the surface
versus return a `Result`), which is why the report defers to the owner rather
than prescribing one. Code untouched, same disposition as PR-zz4exm.

**finding-3 — `every_device_slot_reaches_the_root` guarded by seed coincidence,
fixed and proved.** The reviewer changed one string in that test —
`device_key("replacement")` → `device_key("zzz-other-seed")` — and the
`MAX_DEVICES` 4 → 5 mutation went **green**: the constant raised, the fifth key
unhashed, and the suite reporting 115 passed. `P2pDeviceSlots` stores devices
sorted, so the mutation reds only if the replacement key's digest happens to
sort after the fourth key of the base set. Detection was a property of two
BLAKE3 digests.

The rewrite removes the coincidence rather than picking a luckier seed: build a
pool of `MAX_DEVICES + 1` keys, **sort it**, take the first `MAX_DEVICES` as the
base — so `base[i]` is the key in slot `i` by construction — and substitute the
pool's largest key, which therefore lands in the last slot whatever the digests
are. That is the only substitution a root hashing fewer slots than
`MAX_DEVICES` can fail to see.

Proved, in this order: mutation applied → red at `device slot 4 does not reach
the root`, both roots printed identical (`RootHash(bf3b7b98..)`); reverted;
`shasum -a 256 -c` over all ten `org-members/src/*.rs` → all OK; suite 115
passed. **Then the seed-independence itself was measured**, which is the part
the finding actually asks for: with the mutation applied, the device-key seed
family was changed twice — `zzz-other-seed-{i}` (the reviewer's own seed, which
defeated the old test) and `q7-slot-seed-{i}` — and the test reds at the same
assertion both times; with the constant restored it is green under both. Three
unrelated seed families, one behaviour.

The risk draft's overstatement is corrected with it. The section heading was
"a latent defect held shut by one test" and the body said "held shut by exactly
one test"; it was held shut by that test *and by a digest coincidence*, and the
section now says what the guard did, what the reviewer demonstrated, and what
the guard does now. **Two further uncoupled literals the ledger had never
mentioned are named**: `error.rs:26` renders `"device slots full (max 4)"` — a 4
inside a format string that nothing compares against `MAX_DEVICES`, so raising
the constant makes the crate report a limit it no longer enforces — and
`trie.rs:213`'s doc comment restates "`MAX_DEVICES` (4)". Neither decides what
is hashed, so neither is safety-relevant the way `compute_device_root` is; they
are named because "nothing couples the two numbers" was an undercount and a
later fix should reach all four sites.

**finding-12 — two semantically wrong SOUP references, replaced not dropped.**
`soup.md`'s `blake3` row cited LLR-w5nkbu (the 128-byte `name`/`surname` bounds)
and LLR-pys2ek (`MAX_DEVICES` is 4) as requirements a hash function supports.
Both resolve, so no gate complained, but a field-length rule and a capacity
constant are not properties of a hash. **Replaced rather than dropped**, because
the row genuinely does support hashing items and naming none would be the worse
record: LLR-72p8bz (the domain separation the row's own prose describes),
LLR-kdhd2v (the device sub-trie's empty-slot sentinel and sorted keys),
LLR-wm5hpc (per-level empty-subtree hashes) and LLR-n7nya3 (the write-once hash
cell), which the row already cited. The substitution and its reason are in the
row.

**finding-13 — the proptest seed file is now ignored.** `*.proptest-regressions`
was added to `.gitignore` with the reason written out: this project's method is
to break production code deliberately to prove a test discriminates, a proptest
run under a mutation writes the failing input to that file, and such a seed is a
seed for behaviour the software never had. Committed, it would replay on every
later run and pin the suite to the broken version. Losing a genuine regression
seed costs one rediscovery; committing a mutation seed costs a permanently
misleading suite.

**finding-10 — plan/ledger divergence, pointed at rather than rewritten.** The
architecture table under SDD-d9svdj still carries LLR-tk4qxu's pre-narrowing
wording, under a T2 heading saying the text is copied "verbatim". The history is
not rewritten — this project dates its corrections — so a dated pointer sits at
the table and a one-line cross-reference at T2, naming the narrowing, its date
(2026-09-15, before the ledger was ever merged), the item's current text, and
where the measurement that forced it is written. The pointer also settles which
document is authoritative for item text: the ledger, not this plan.

#### The six recorded as obligations, and where each went

Written where their subject lives, not into a verification record — a
verification record is read once, at its own merge, and these outlive it. Each
carries `opened: 2026-09-17`. **Nothing was minted for any of them.**

| Finding | Recorded as | Where |
|---|---|---|
| 5 — three `to_nfc` sites whose removal leaves 115/115 green (`trie.rs:174` surname, `types.rs:530` surname, `types.rs:484` name on the `Deserialize` path) | Obligation A | risk draft |
| 6 — LLR-h9gs32's "no input reaches a panicking path" is stronger than `unwrap`/`expect`/`panic` denials support | Obligation B | risk draft |
| 7 — the SOUP advisory-database gap is disclosed but carried by no item and aged by no gate | Obligation C | risk draft |
| 2 — no test in the suite ever serialises or deserialises a `Delta` | Obligation D | decomposition ledger |
| 8 — `pending_changes()`, `has_pending_changes()`, `last_calculated_root` and `orgtrie_is_send_sync` are unmarked derived work | Obligation E | decomposition ledger |
| 11 — `org-members/docs/CONTEXT.md` is still the unfilled template | Obligation F | decomposition ledger |

Three of them are worth restating here because they say something about the
method rather than about one item.

**Obligation D explains finding-1.** All twenty-one postcard encode/decode call
sites in the three test files handle a `MemberLeaf` or a `P2pDeviceSlots`;
**none handles a `Delta`** (verified by inspection, not taken from the review).
SDD-55b2zj is *defined* as the change set that crosses a process boundary and
LLR-xmpqn2 is worded about "the postcard encoding" — so the item's defining
boundary is precisely what nothing in the suite crosses. Everything tested about
deltas builds a value in memory and hands it to `apply_delta` in the same
process. A round-trip test would not have proved injectivity, but writing one
would have forced the author to ask what the deserialisation path does to the
leaf fields, and the answer to that question is finding-1.

**Obligation E names a gap `check-trace.sh` structurally cannot find.** The
checker flags an LLR with no test and design with no requirement. It never flags
a *test* with no LLR or a public function with no item — there is no direction in
the tooling running from the code towards the ledger. So every gate in this
change was green while a public, documented, five-test administrative-review
capability sat outside the traceability graph entirely. A reviewer reading the
code found it; no script could have.

**Obligation C is about the shape of a gap, not the gap.** The missing advisory
scan is honestly disclosed in `soup.md` and repeated as finding 5 of the risk
draft's owner findings. What is missing is that prose in an inventory is
invisible to `check-trace.sh`: nothing prints it at a merge, nothing counts its
days, nothing fails when it is still open in six months — unlike PR-zz4exm,
which is the same kind of outstanding obligation and *is* surfaced with its age
at every merge because it is an item with an `opened:` date. A tracked item is
**recommended and not minted** (the brief permitted one ID and it went to
PR-jq43gx), with a note that the gap is the toolchain's rather than this unit's,
so four per-unit reports may be worse than one repository-level item.

#### What this task did not do

It wrote no production code — `delta.rs`'s change is `///` lines and nothing
else, and the other nine source files are byte-identical to the change branch.
It added and removed no test. It minted no HAZ, RC, SDD or LLR: the two places
where an item looks wanted (Obligation D's delta round-trip, Obligation E's
pending-changes capability) say so and stop, because minting an LLR would also
owe a derived assessment, and that is `analyze-risks`' business rather than a
fix round's. It did not touch PR-zz4exm, which stays open at 17 days.

Three behaviour changes are now explicitly owed and explicitly deferred, each
with its reason written where a later reader will meet it: making the postcard
encoding injective (finding-1), fixing or narrowing `MemberId::bit` (PR-jq43gx),
and coupling the four `MAX_DEVICES` sites (the pre-existing latent defect). Each
needs its own red-first change.

---

### T12 — The fix round for independent review round 2

**Files touched:** `org-members/src/trie.rs` (doc comments only),
`org-members/README.md`,
`docs/superpowers/specs/2026-05-28-org-members-hyperbridge-review.md`,
`org-members/docs/architecture/2026-09-17-decomposition.md`,
`org-members/docs/architecture/README.md`,
`org-members/docs/risk/2026-09-17-design-derived.md`,
`org-members/docs/problems/2026-09-17-review-fixes.md`,
`org-members/docs/problems/DRAFT-worktree-guardrails-org-members-arch-review2-fixes.md`
(new), this plan
**Parallel:** no (serial, after T11 and the second independent review)

Added 2026-09-17. A **second** independent review of the same change returned
seven findings. The owner's disposition the same day: **fix 1, 3, 4, 5, 6; file
a problem report for 2 without touching the code; record 7.** Mint nothing
beyond one problem report. Add no test and change no production code except doc
comments.

The reason this round exists at all is finding 1: T11 corrected the postcard
byte-uniqueness claim in four places and the claim lived in **eight**. A false
claim that has been partially corrected is worse than one that has not, because
the correction is now evidence the matter was looked at.

#### T12 — DONE (task branch `…-t12-review2-fixes`, three commits, not merged)

```
cargo test -p org-members    0 + 6 + 108 + 1 + 0 = 115 passed, 0 failed
                             — unchanged; no test added, removed or edited
check-trace.sh               REQ 12, HAZ 8, RC 10, SDD 6, LLR 35, PR 3   exit 0
                             UNRESOLVED-PR PR-zz4exm (17 days)
                             UNRESOLVED-PR PR-jq43gx (0 days)
                             UNRESOLVED-PR PR-vf5hdm (0 days)
                             problems: open 3, oldest 17; limits age 30, open 10
check-ids.sh --allow-draft-files                                         exit 0
```

Both gates run with `GR_CONFIG=org-members/.guardrails/config.yaml`. Three open
problem reports against a limit of ten, oldest 17 days against a limit of 30 —
the advisories are the roll-call, not a failure. One ID minted: **PR-vf5hdm**,
the one the brief allocated.

**Production code, and how the "doc comments only" claim was verified.**
`git diff worktree-guardrails-org-members-arch --stat -- org-members/src` names
**one** file, `trie.rs`, 29 insertions and 7 deletions. Every one of those 36
lines was then checked to be a `///` line:

```sh
git diff worktree-guardrails-org-members-arch -U0 -- org-members/src/trie.rs \
  | grep -E '^[+-]' | grep -v '^\(+++\|---\)' | grep -vE '^[+-][[:space:]]*///'
```

printed nothing. So no executable line changed in `trie.rs`, and the other nine
`org-members/src/*.rs` do not appear in the diff at all.

**finding-1 (BLOCKING) — the byte-uniqueness claim, corrected in its last four
places.** T11 fixed the decomposition ledger's LLR-xmpqn2 note, `delta.rs`,
`soup.md`'s postcard row and the risk file's LLR-8jttpb assessment. Still
standing, all four verified present before editing:

| Where | What it said | What it says now |
|---|---|---|
| `trie.rs`, `validate_canonical_delta`'s doc | "so that the postcard byte string accepted by `apply_delta` is the unique encoding of the transition" | the rules narrow the accepted `Delta` **value**; a dated paragraph saying the byte claim is false, why, and what to key on |
| `trie.rs`, `apply_delta`'s doc | "Together these guarantee that the postcard byte string of an accepted `Delta` is the unique encoding for the transition" | the same, with the measured counterexample, and a pointer that the threat-model spec it sends readers to now carries a disproof note |
| `org-members/README.md`, "What this crate guarantees" | "`d` is the unique postcard **byte string** …", followed by "**Status:** the invariant holds" | the invariant restated over the decoded `Delta` **value**; the status qualified; a block quote naming what was false, the counterexample, and what to key on |
| `org-members/README.md`, §2 "Authenticate the delta blob" | "once you've verified one byte string for a given `(base_root, target_root)`, no other byte string with the same effect exists, so signatures, hashes, and replay caches all key cleanly off the blob bytes" | "**Do not key replay caches, dedup or change identity off the blob bytes**", with the failure it causes, plus a second bullet keeping "sign the bytes" correct and separating signing from identity |

**The fourth was the one worth the round.** It is not a claim, it is an
**instruction to integrators**, written in the file `delta.rs` points readers to,
telling them to do the exact thing the risk file's corrected LLR-8jttpb
assessment tells `org-node` not to do. A reader who obeyed it would build a
replay cache keyed on bytes and it would silently fail to recognise a
re-encoding of a change already applied.

The counterexample is cited in the code and in the README rather than merely
asserted: with `name = "é"`, a `Delta` encodes to **141 bytes**; byte-patching
the NFC name to its NFD form gives a distinct **142-byte** string; both decode,
both are accepted by `apply_delta` on the same base, and both `verify_against`
the same target root. The cause is read straight off `types.rs`: `MemberLeaf`'s
`Deserialize` runs `to_nfc` over `name` and `surname` and stores the NFC form
`validate_handle` returns, where `P2pDeviceSlots`' `Deserialize` rejects. The
new wording was written against T11's existing text in `delta.rs` and in the
LLR-8jttpb assessment rather than inventing a third vocabulary — value versus
bytes, "normalises rather than rejects", "key on the decoded `Delta` or on the
`(base_root, target_root)` pair".

**The historical spec keeps its body and gains a head note.**
`docs/superpowers/specs/2026-05-28-org-members-hyperbridge-review.md` asserts the
byte-uniqueness invariant at §4 and §6, and both `delta.rs` and `trie.rs` send
readers there for the threat model. This project does not rewrite history, so
the spec now opens with a dated block quote saying the invariant was disproved
on 2026-09-17, why, which of its own conclusions do not follow (the
`postcard(Delta)` replay-cache key, "no semantically-equivalent alternative blob
exists"), where the six corrections live, and that H-1 through Info-4 and the
threat model stand. Nothing below it changed.

**finding-2 (BLOCKING) — PR-vf5hdm, filed, code untouched.** `apply_delta`
refuses a lawful membership change. Verified by reading the upsert loop: it runs
in a **single ascending pass** and releases a leaf's old handle only when it
reaches that leaf, so a handle in flight between two members is still in
`skeleton_index` when the acquirer is checked, and the branch returns
`DuplicateHandle`. A two-way swap fails in **both** identifier orders (symmetric,
and canonical form forbids reordering the upserts anyway); a one-way handover —
giver `bravo`→`charlie`, taker `alpha`→`bravo` — fails whenever the taker's
identifier sorts first. The producer performs both renames sequentially and each
succeeds; `calculate_delta` emits a canonical two-leaf delta; the receiver
holding the byte-identical base trie refuses it. **Producer and receiver
diverge, with no adversary and no malformed input**, and no retry helps: the
same bytes fail identically forever. Removals are drained before the first
upsert, so the removal path does not have this defect.

Filed as **PR-vf5hdm** (opened 2026-09-17, open) in this round's own draft,
`org-members/docs/problems/DRAFT-worktree-guardrails-org-members-arch-review2-fixes.md`.
Its `affects:` names SDD-55b2zj, LLR-juxk9q, LLR-xmpqn2, LLR-8jttpb and
LLR-h7stq2, each with the reason it is named — and it records that **no
requirement is contradicted**, which is itself part of the finding: nothing in
`org-members/docs/requirements/` says a change set the software produces shall
be accepted by a receiver in the state it was computed against, so every gate is
green over the hole.

Two documents asserted a property this defect falsifies, and both were corrected
rather than deleted: the decomposition ledger's SDD-55b2zj prose and the risk
file's LLR-8jttpb assessment both said "an honest producer never trips an honest
receiver's canonical-form check". Strictly the refusal happens in the
handle-uniqueness block **after** `validate_canonical_delta` returns `Ok`, so
the sentence is defensible on a narrow reading and false on the reading every
reader takes. Both now say the narrow thing explicitly, name where the refusal
actually happens, and cite PR-vf5hdm.

**No hazard in the register covers it, recorded as finding 6 for
`analyze-risks`.** HAZ-8suua9 is malformed, hostile or oversized input — there
is none here. HAZ-y8h835 is the nearest miss and still does not reach it: its
causes are all lag-shaped (including "a delta applied to the **wrong** base",
where this one is applied to the right base), and its probability and
acceptability argument turns on recoverability through the chain anchor, which
is exactly what is absent — the receiver has the change, refuses it, and will
refuse it on every retry. The harm is the register's own, a member wrongly
denied access; the **cause** is new: a correct component refusing a correct
input. **No HAZ and no RC minted** — rating it owes a severity, a probability
and a matrix evaluation, which is the interview's work.

**finding-3 — PR-jq43gx widened to both sites of its own shape.**
`DefaultHashes::at_level(level: u16)` is `pub` on a `pub` struct in `pub mod
smt`; `compute()` fills the vector with `SMT_DEPTH as usize + 1` = 257 elements,
so `at_level(300)` indexes 300 into 257 and panics at `smt.rs:34:21` — the same
shape as `MemberId::bit(256)` at `types.rs:66:10`. Public, `u16` parameter,
unguarded slice index, no mask, no `debug_assert`, no `Result`, every internal
call site bounded by `SMT_DEPTH`. The report presented `bit` as *the* instance,
so a reader acting on it would have fixed one site and left the other.

Amended **in place**, in `org-members/docs/problems/2026-09-17-review-fixes.md`
where the item is defined — not restated elsewhere, because a problem report is
edited in the file that defines it. The item sentence now names both sites, the
`affects:` line gains **LLR-wm5hpc** (the 257 per-level empty-subtree hashes are
what `at_level` reads), and a new section tabulates the two sites element by
element and states the load-bearing point: **whichever reading of REQ-ds8ryr the
owner takes for `bit()` applies unchanged to `at_level()`**. There is no reading
on which they are answered differently, which is why they are one report and one
decision rather than two. It is also the concrete second instance of Obligation
B, which already named `smt.rs:34` as a live indexing site the lint posture does
not reach — the obligation named it, and this report is now what ages it.

**finding-4 — Obligation A undercounted its own site list, and the fourth site
was measured before it was written down.** The obligation was headed "three
normalisation sites" and its table listed three, while its own prose said the
deserialisation half is uncarried "for `name` as well as for `surname`" — four.
The two halves disagreed and the prose was right. The fourth site,
`types.rs:490`, is `to_nfc(&raw.surname)` on the `Deserialize` path. **Verified
by mutation, not by inspection**, the same way the first three were:

```
let surname = raw.surname.clone();   in place of   to_nfc(&raw.surname)
  -> 0 + 6 + 108 + 1 + 0 = 115 passed, 0 failed   (no test reddened)
reverted; shasum -a 256 -c over all ten org-members/src/*.rs -> all OK
```

The table now has four rows, the heading says four, and the prose says what is
actually true: **both** halves of normalisation are uncarried at **both** of
their sites, and the only normalisation clause any test carries is `name` in
`update_name_surname`. "Three assertions, not three tests" became four.

**finding-5 — the SDD `traces:` lists disagreed with their own LLRs, four ways.**
Every SDD's `traces:` list was checked against the `satisfies:` lines of the LLRs
written beneath it before anything was edited, and each disagreement was decided
on its merits rather than by making the lists mechanically equal. The rule
applied: an LLR whose REQ the parent does not trace is **always** a defect,
because impact analysis from that REQ stops at the item; a `traces:` entry with
no refining LLR may still be right, if the item genuinely realises the
requirement without one LLR being where it is refined.

| Item | Finding | Decision |
|---|---|---|
| SDD-k5wa4n | omits REQ-h5ret5 though LLR-mmst86 under it declares `satisfies: REQ-h5ret5` | **added** |
| SDD-k5wa4n | omits REQ-ds8ryr though LLR-v3jqau under it declares `satisfies: REQ-ds8ryr` | **added** — this is the damaging one: impact analysis from REQ-ds8ryr reached only SDD-m9gs5g and silently missed LLR-v3jqau's seven carriers |
| SDD-d6x85b | traces REQ-avmu3j, no LLR refines it | **removed** — REQ-avmu3j is a refusal behaviour and no refusal lives in `hasher.rs` or `device_trie.rs`; hashing a value is not reporting a root |
| SDD-d6x85b | traces REQ-4umsuz, no LLR refines it | **removed** — base and result-root matching is SDD-55b2zj's entirely |
| SDD-k5wa4n | traces REQ-avmu3j, no LLR refines it | **kept** — `OrgTrie::root_hash()`, which returns `Err(HashesNotCalculated)` rather than a stale root, is defined in `trie.rs`, this item's file |
| SDD-k5wa4n | traces REQ-d3prca, no LLR refines it | **kept** — every one of the eight operations takes `&self` and returns a new `OrgTrie`, which *is* the requirement; realised by the item's shape after LLR-tk4qxu was narrowed on 2026-09-15 |
| SDD-55b2zj | traces REQ-d3prca, no LLR refines it | **kept** — `apply_delta` takes `&self` and yields a `CandidateTrie`; the same requirement at the process boundary, and the reason a candidate exists |

Neither removal orphans a requirement: REQ-avmu3j stays traced by SDD-d9svdj and
SDD-k5wa4n, REQ-4umsuz by SDD-55b2zj. The reasoning for all seven is written
into the ledger under "Trace corrections — 2026-09-17", not only here, because
the ledger is where the next reader will ask.

**Recorded as Obligation G: REQ-crjxk8's identifier-immutability clause is
refined by no LLR anywhere.** The requirement has two clauses and only the
duplicate-rejection one is refined (LLR-fv75ec and LLR-ch2pkw, both
`satisfies: REQ-crjxk8`). The clause "an identifier that does not change when
that member's handle changes or when any of that member's keys are replaced" has
no LLR at all — LLR-k89ahd, LLR-mmst86 and LLR-zbe553 come close and stop short.
The REQ passes the gate because it carries direct `verifies:` annotations and
the transitivity runs upward only, which is exactly why nothing reddens. It is
**trivially testable** — `with_handle`, `with_p2p_key` and
`with_p2p_device_slots` each preserve `id()`, three assertions in one test — and
that is recorded as the argument for closing it. **No LLR is minted**: the item
would owe a carrier under the mutation protocol, and this round adds no test.

**finding-6 — two dead cross-references, both fixed.** The decomposition
ledger cited `../problems/DRAFT-worktree-guardrails-org-members-arch-review-fixes.md`,
which `finalize-docs.sh` renamed to `2026-09-17-review-fixes.md`; the reference
now names the real file and records the old name and why it changed. And this
plan's draft→final mapping table listed only the architecture and risk files
while T11's own `Files touched:` names the problems draft, so a reader of T11
had no mapping for it — the table has a third row now. PR-jq43gx's self-citation
("finding-2 of that review") was corrected to **finding-4**, against this plan's
T11 record, in the item's own file.

**finding-7 — closed in place rather than recorded, and the judgement is
stated.** The per-change ledger's Overview delegates the decomposition picture to
`org-members/docs/architecture/README.md`, and that README's Overview did not
contain it: it named one of the six items (SDD-55b2zj) and described the split as
four conceptual lines, listing neither the six items nor the files they own. The
brief permitted either recording it as a dated obligation or writing it. **It was
written**, because it is additive prose about items that already exist, it mints
nothing, changes no `traces:` or `satisfies:` line, adds no test, and the README
is the document that survives the per-change files — deferring it would have left
the delegation false for the sake of procedural tidiness. The README Overview now
carries a six-row table: item, one line of responsibility, and the source files
each owns, plus how the four conceptual lines map onto the six (SDD-m9gs5g cuts
across all four). The decision and its reasoning are recorded in the ledger under
"Finding-7 of the round-2 review — closed in place, not deferred", so the record
exists either way.

#### What this task did not do

It wrote no production code: one source file differs from the change branch and
it differs in `///` lines only, verified by the filtered diff above rather than
by assertion. It added, removed and edited no test — the suite is 115 passed, 0
failed before and after. It minted no HAZ, RC, SDD or LLR: three places where an
item looks wanted (Obligation G's immutability LLR, a HAZ for PR-vf5hdm's
pathway, an RC for it) say so and stop. It did not touch PR-zz4exm, and it did
not resolve PR-jq43gx — widening a report is not resolving it. No
`proptest-regressions` file was produced by the one mutation round; the working
tree is clean.

**One behaviour change is now owed that was not owed before**: fixing
PR-vf5hdm's handle handover, whose plausible shape is a two-phase upsert loop
that releases every outgoing handle before checking any incoming one. Under
`resolve-problem` it starts with a test that reproduces the refusal, which this
round was not permitted to add. That joins the three T11 deferred and PR-zz4exm.

---

## Gate, review and merge

After T1–T9, `verify-before-merge`. Two things about the gate in this
repository that are easy to get wrong:

- **The impact set is three units, not one.** `.guardrails/units.yaml` records
  the four `depends_on:` edges, and a change to `org-members` runs `org-node`'s
  and `app`'s gates too. Compute it, do not assume it:
  `.guardrails/scripts/check-units.sh --impact "master..HEAD"`. `app`'s
  `verify_commands` need `npm --prefix app ci` first (`app/node_modules` is
  gitignored) and a Tauri-capable toolchain; `cp -Rc app/src-tauri/target` from
  the primary checkout takes about two seconds against a cold build's many
  minutes.
- **`check-review.sh` is not part of this gate.** The record it reads is written
  at `merge-change` step 6b, after the independent review at 6a.

`coverage_command` is configured for this unit (`make coverage-org-members`,
floors at 93.50% lines / 92.20% regions), so check 5 applies and the figures go
in the verification record. This change adds five tests and no production code,
so coverage should not fall; if it does, that is a finding, not a floor to
lower.

### A defect on master, found by this change's gate but not caused by it

**`on-chain-client`'s `check-ids` gate is red on `master` today**, and has been
since tooth 3. Measured from this worktree on 2026-09-17:

```
GR_CONFIG=on-chain-client/.guardrails/config.yaml check-ids.sh
DRAFT-FILE on-chain-client/docs/problems/DRAFT-…-on-chain-client-problems.md
DRAFT-FILE on-chain-client/docs/requirements/DRAFT-…-chain-reading.md
DRAFT-FILE on-chain-client/docs/risk/DRAFT-…-on-chain-client-hazards.md
exit 1
```

All three are **tracked files**, committed by `fbf17f0` — tooth 3's
on-chain-client hazard analysis. `finalize-docs.sh` was never run for that unit,
so its three ledger files were squashed onto `master` still carrying their
draft names, and they have no dated filenames at all.

How it survived that merge: `verify-before-merge` step 3 runs `check-ids.sh`
**with** `--allow-draft-files`, which is correct there — the change's own drafts
are legitimate at that point. `merge-change` step 4 runs it **without** the flag,
and that is the run that would have convicted. The flag is the whole difference
between the two, which is exactly why this change ran step 4 bare in every unit
of its impact set and reported the exit code each time.

**Not fixed here, deliberately.** `on-chain-client` is not in this change's
impact set (`org-members` touched, `org-node` dependent, `app` touched). Renaming
files in it would pull the unit into the impact set and oblige this change to run
its gates and own its ledger — scope this change has no business taking. It is
recorded here, and the fix is a one-commit change of its own: run
`finalize-docs.sh` with that unit's `GR_CONFIG`, commit the three renames, and
check that nothing references the old names.

It also means **tooth 4's on-chain-client architecture change cannot merge until
this is cleared**, because its step 4 will convict on files it did not create.
Worth doing before that tooth starts rather than inside it.

> **Correction, 2026-09-17 — the blocker described in this section is cleared.
> The text above is left standing because this project dates its corrections
> rather than rewriting the record.**
>
> The one-commit change this section calls for was made the same day:
> `worktree-guardrails-on-chain-client-finalize`. It renamed the three
> `DRAFT-…` ledger files to
> `on-chain-client/docs/requirements/2026-09-10-chain-reading.md`,
> `on-chain-client/docs/risk/2026-09-10-on-chain-client-hazards.md` and
> `on-chain-client/docs/problems/2026-09-10-on-chain-client-problems.md`, and
> rewrote every cross-reference **to those three files** — seven of them, all
> inside the on-chain-client ledgers. It did **not** repair every stale pointer
> `fbf17f0` left behind: `Makefile` line 47 and
> `on-chain-client/.guardrails/config.yaml` line 106 both still cite
> `docs/verification/2026-09-10-worktree-guardrails-on-chain-client-risk.md`,
> a path that does not exist — the record is dated 2026-09-11. Those two are
> left deliberately, because editing a Makefile or a unit config would cost that
> change its documentation-only property; they are filed as an open item in
> `docs/plans/2026-09-05-ratchet-setup.md` under "Added 2026-09-17". The date is
> 2026-09-10, not the 2026-09-11 merge date of `fbf17f0`, on the owner's
> decision — the reasoning is in
> `docs/plans/2026-09-10-on-chain-client-risk-analysis.md` under "Filenames,
> corrected on 2026-09-17".
>
> So `GR_CONFIG=on-chain-client/.guardrails/config.yaml check-ids.sh` bare is
> exit 0 on `master`, and tooth 4's on-chain-client architecture change is no
> longer blocked by it. The systemic open items this defect raised — how a
> skipped `finalize-docs.sh` gets caught locally, and whether this repository
> pushes at all — are filed in `docs/plans/2026-09-05-ratchet-setup.md` under
> "Added 2026-09-17" and remain open.

### Gate result — 2026-09-17

Run in the change worktree over the computed impact set (`check-units.sh
--impact "master..HEAD"` → `org-members touched`, `org-node dependent`,
`app touched`; `app` is touched because T1 adds its lockfile).

```
check-units.sh                  units 4, disclaimed 7; tracked paths 410   pass
org-members  trace / ids        1 advisory (UNRESOLVED-PR, 16d of 30)      pass
             cargo test         115 passed, 0 failed, 0 warnings
             quint ×2           clean
org-node     trace / ids        7 advisories, all within limits            pass
             cargo test         51 passed + 2 fuzz targets clean, 0 failed
             quint              clean
app          trace / ids        6 advisories, all within limits            pass
             cargo test         78 passed, 0 failed
             npm run check      0 errors, 1 pre-existing @types/node warning
             npm run test       30 passed, 0 failed
TOTAL                           274 passed + 2 fuzz targets, 0 failed, 0 skipped
git status                      clean
```

Nothing was skipped: `app`'s Tauri toolchain and `npm ci` both ran here, which
the config's own note warns is not guaranteed on every machine.

**Counts chased rather than assumed.** Two of `org-node`'s ten targets print no
`test result` line at all — they are libfuzzer harnesses reporting their own
format (`exit reason: max duration (1s - default) exceeded`, no failing input,
no panic). Counted as two targets passed, not as silent skips. That is the check
the skill's "exit 0 is not a pass" rule exists for.

#### Check 4 — both halves answered

The subagent's half: **all 35 LLRs carry a `verifies:` test**, and the six SDD
items carry none — correct, since `check-trace.sh` has no MISSING-TEST gate for
`SDD`; their gate is `UNTRACED-DESIGN`, which is clean. Traced, not tested.

The author's half, which no subagent can answer: every one of the 35 has a
`red -> green:` attestation in a dispatch report — T3 eight, T4 two, T5 six,
T6a five, T6b six, T7 six, T8 two. Three carry documented type-level exceptions
instead of a mutation red (LLR-tk4qxu, and one clause each of LLR-y38jfk and
LLR-7tdqv9), each named in the ledger beside its item so a reader who looks for
the row finds the reason instead.

#### Check 5 — accepted as a documented gap, by the owner, on 2026-09-17

```
Lines    1000 total,  65 missed   93.50%   floor 92   pass
Regions  1551 total, 121 missed   92.20%   floor 91   pass
Branches    0 total,   0 missed   "-"      UNMEASURED
```

Both figures are byte-identical to the 2026-08-26 baseline in the `Makefile`,
which is what a change adding five tests and no production code should do.

**The class C target is statement AND decision, and the decision half is not
measured at all** — llvm-cov reports `Branches 0 / 0 / "-"` for every file in
the unit. This is the pre-existing shortfall recorded in
`docs/plans/2026-09-05-ratchet-setup.md` and scheduled for tooth 5, where
coverage is set for `org-node` and `app` on one measurement basis rather than
piecemeal. This change neither closes it nor worsens it.

The owner accepted it as a documented gap. **The verification record must state
the acceptance, the figures, and that decision coverage is unmeasured rather
than passing** — an unmeasured number reported as a pass is the exact failure
mode the skill's "exit 0 is not a pass" rule names, one level up.

#### Check 6 — 18 of 35 single-sided, and what was decided

The gate classified every LLR by reading test bodies, not just names: 17 have
both sides, 8 are abnormal-only, 10 are normal-only. Its own diagnosis is the
useful part — **in several cases the missing side already exists in the suite
but is annotated to a sibling LLR**: LLR-5w2jx8's normal counterpart is
`genesis_multiple_members`, annotated to LLR-ch2pkw; LLR-s97ywt's failure path
is `delete_p2p_device_nonexistent_member_fails`, annotated to LLR-v3jqau.

Owner's decision, 2026-09-17: **fix the annotations where the test already
exists, record a reason for the rest, add no tests.** That is T10 below. Each
added annotation still earns its own mutation — an annotation added to satisfy
a table, without evidence that the test discriminates, is precisely what this
change's method exists to prevent.

---

## Self-review

1. **Every ID in Implements: has a task whose test verifies it.** Six SDD items
   — SDD items are traced, not tested, and `check-trace.sh` has no MISSING-TEST
   gate for the `SDD` prefix; their gate is `UNTRACED-DESIGN`, discharged by the
   `traces:` lines T2 writes. Thirty-five LLRs, each named in exactly one of
   T3–T8 with its carrier test.
2. **Real code, real commands, expected output.** The five new tests are written
   out; each task states the test count the suite must report.
3. **Names and signatures consistent across tasks.** The helpers `member_id`,
   `member_key`, `device_key`, `alice()` are the ones already at
   `integration_test.rs:10–40`; T4–T6 use them rather than defining new ones.
4. **Files touched and Parallel on every task, and no two parallel tasks share a
   file.** T1 (`.gitignore`, three lockfiles, `soup.md`), T2 (the architecture
   draft) and T9 (the risk draft) are mutually disjoint and disjoint from
   T3–T8. T3–T8 all touch `integration_test.rs` and are serial; T8 additionally
   touches `fuzz_tests.rs` and `mbt_conformance.rs`, which nothing else does.
   T9's file set is disjoint from all of them, but it is serial anyway — it
   records what T3–T8 measure, which is a content dependency a file-set test
   does not catch.

### What this plan does not do

It writes no production code. Every LLR describes behaviour `org-members`
already has, which is why red-by-mutation is the discharge and why PR-zz4exm
stays open rather than being fixed here: fixing it is a behaviour change, it
needs its own red-first test, and folding it into an architecture change would
put a product fix where no reviewer is looking for one.

It does not touch `org-node`, `on-chain-client` or `app` beyond tracking
`app/src-tauri/Cargo.lock`, and it does not write their architecture ledgers.
Those are three further changes under this same tooth.

# Problem reports — public `u16` index parameters panic on an out-of-range value

**PR-jq43gx**: two public methods taking a `u16` index panic on a value the type
admits and nothing rejects — `MemberId::bit(index)` for any `index` of 256 or
beyond (`index / 8` gives a byte offset of 32 or more into a `[u8; 32]`) and
`smt::DefaultHashes::at_level(level)` for any `level` of 257 or beyond (a direct
index into a 257-element `Vec`) — both reachable from outside the crate.
affects: REQ-ds8ryr, LLR-h9gs32, LLR-zbe553, LLR-wm5hpc.
opened: 2026-09-17
status: open

Found by the independent review of the architecture change (merge-change step
6a, finding-4 of that review; this file previously miscited it as finding-2,
corrected 2026-09-17 against the plan's T11 record). The second site,
`at_level`, was added to this item on 2026-09-17 by the round-2 review, which
found the report's scope too narrow — see "A second site of the same shape"
below. Neither was found by a failing test. `org-members/src/types.rs`:

    pub fn bit(&self, index: u16) -> bool {
        let byte_idx = (index / 8) as usize;
        let bit_idx = 7 - (index % 8);
        (self.0[byte_idx] >> bit_idx) & 1 == 1
    }

`index` is a `u16`, so 256..=65535 are representable and none of them is
rejected. The slice index is the panicking site. There is no `debug_assert`, no
mask, and no `Result`.

## What it puts in tension

**REQ-ds8ryr** says the software "shall report every rejected membership
operation as an error and shall **not panic, for any input**, including input
that is malformed, hostile, or exceeds a documented limit." **LLR-h9gs32**
carries that down as "every rejected operation returns an `OrgMembersError`
variant, and the crate denies `unwrap`, `expect` and `panic` at the lint level
so that no input reaches a panicking path" — and the lint denial is real and
CI-gated but does not reach slice indexing, so it does not establish the
conclusion here (that gap is recorded separately against LLR-h9gs32 in
`../risk/2026-09-17-design-derived.md`).

**Whether `bit()` is a "membership operation" in REQ-ds8ryr's sense is itself
the open question, and is the reason this is a problem report rather than a
defect with an obvious fix.** Two readings are available and the ledger should
not pretend otherwise:

- `bit()` is an *addressing primitive*, not a membership operation. It adds no
  member, removes none, and rejects nothing; it is the bit-extraction LLR-zbe553
  describes, exposed because the SMT traversal needs it. On this reading
  REQ-ds8ryr does not govern it and the panic is a normal Rust out-of-bounds
  contract violation, no different from indexing a slice.
- `bit()` is reachable from outside the crate, on a public type, with an
  argument type that admits the bad value. On this reading it is *input*, the
  requirement says "for any input", and a panic is a violation.

The owner decides which reading stands. If the first, the fix is to the
**surface** (make it `pub(crate)`, or narrow the parameter) rather than to the
behaviour, and REQ-ds8ryr's scope should be written down so the next reader does
not re-open this. If the second, the fix is a behaviour change: return a
`Result`, or mask the index, with a reproducing test first.

## A second site of the same shape — `DefaultHashes::at_level`

Added 2026-09-17 by the round-2 independent review, which found this report's
scope too narrow: it presented `MemberId::bit` as *the* instance when the crate
has two of identical shape, and a reader who acted only on the report would fix
one and leave the other. `org-members/src/smt.rs`:

    pub struct DefaultHashes {
        hashes: Vec<NodeHash>,
    }

    pub fn at_level(&self, level: u16) -> &NodeHash {
        &self.hashes[level as usize]
    }

`hashes` is built by `compute()` with `SMT_DEPTH as usize + 1` = **257**
elements (`SMT_DEPTH` is 256). `at_level(300)` therefore indexes 300 into a
257-element vector and panics at `smt.rs:34:21` — "len is 257 but the index is
300" — exactly as `MemberId::bit(256)` panics at `types.rs:66:10` with "len is
32 but the index is 32". Every element of the shape matches:

| | `MemberId::bit` | `DefaultHashes::at_level` |
|---|---|---|
| reachable externally | `pub fn` on `pub struct MemberId`, re-exported from the crate root | `pub fn` on `pub struct DefaultHashes` in `pub mod smt` |
| parameter | `index: u16` | `level: u16` |
| admitted-but-invalid values | 256..=65535 | 257..=65535 |
| panicking construct | unguarded slice index | unguarded slice index |
| guard | none — no mask, no `debug_assert`, no `Result` | none |
| the crate's own call sites | all bounded by `SMT_DEPTH` | all bounded by `SMT_DEPTH` |

`DefaultHashes::compute()` is `pub` too, so an external caller can build the
value it needs to make the call; nothing about this requires access to a trie.

**Whichever reading of REQ-ds8ryr the owner takes for `bit()` applies unchanged
to `at_level()`.** If `bit()` is an addressing primitive the requirement does not
govern, so is `at_level()` — a level lookup into precomputed defaults — and the
fix for both is to the *surface*: `pub(crate)`, or a narrower parameter type. If
`bit()` is public input that "shall not panic, for any input" governs, so is
`at_level()`, and both want the same behaviour change. There is no reading on
which the two sites are answered differently, which is why they are one report
rather than two: they will be fixed by one decision and one change.

The item's `affects:` line gains **LLR-wm5hpc** for this site — the item that
states the 257 per-level empty-subtree hashes, which is what `at_level` reads.

This is also the concrete second instance of Obligation B in
`../risk/2026-09-17-design-derived.md`, which already names `smt.rs:34` among
the live indexing sites the crate's lint posture does not reach. The obligation
names the site; this report is now what ages it.

## What was corrected without waiting for that decision

The abnormal-input waiver for LLR-zbe553 in
`../architecture/2026-09-17-decomposition.md` used to read "no caller can supply
one (every call site is bounded by `SMT_DEPTH`)". That parenthetical is true of
the crate's internal call sites and false of external ones, so the waiver rested
on a false premise. The paragraph now states the real situation and points here.
The waiver's *conclusion* — no test is added — is unchanged, but it now stands
on the owner's decision to defer, not on an unreachability claim.

## Not fixed in this change, deliberately

The change that filed this is the architecture change's review-fix round: it
corrects claims and one test's robustness and writes no production code. Fixing
`bit()` is a behaviour or surface change and gets its own red-first change, the
same disposition PR-zz4exm has carried since 2026-08-31. Folding a product fix
into a documentation-correction round would put it where no reviewer is looking
for one.

Per `resolve-problem`, the fix starts with a test that reproduces it — which for
this item means a test asserting a panic, or asserting the `Err`/masked result
the chosen reading produces. That choice is part of the decision above.

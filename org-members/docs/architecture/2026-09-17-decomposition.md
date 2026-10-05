# org-members — software decomposition, items and low-level requirements

The first architecture ledger for the `org-members` unit: six software items
and the thirty-five low-level requirements that refine the twelve high-level
requirements in `org-members/docs/requirements/`. The unit's SOUP inventory is
`org-members/docs/architecture/soup.md`.

Safety class C throughout (`org-members/.guardrails/config.yaml`, no per-item
override), so every item below carries LLRs rather than the class B "optional
per item".

## Overview

The unit's overview — the decomposition picture and the segregation rationale —
lives in the Overview section of `org-members/docs/architecture/README.md`,
which is the standing description every later change amends, rather than in this
per-change file.

## SDD-4yr9ge — Member record and validation

`org-members/src/types.rs`, `org-members/src/normalize.rs`

**SDD-4yr9ge**: the validated, canonically serialisable member record — the
immutable identifier, the handle, the member-as-a-group key, the device key
set and the personal fields — together with the validation and
normalisation every construction path applies, including the path from
deserialised bytes. traces: REQ-crjxk8, REQ-h5ret5, REQ-m8aexh, REQ-xdx2c2,
REQ-shk82j, REQ-t46uad

**LLR-xzqs9r**: `Handle::parse` (and `TryFrom<&str>`/`TryFrom<String>`,
which delegate to it) is the only constructor of a `Handle`; it stores the NFC
form and rejects, with `InvalidHandle`, a handle that is empty, exceeds 128
bytes after normalisation, contains an uppercase character, contains `.`,
contains a character not permitted in identifiers by UTS#39 (the Unicode
General Security Profile, `unicode_security` `identifier_allowed`) other than
`-`, or mixes Unicode scripts — `-` being permitted alongside any script.
satisfies: REQ-h5ret5, REQ-t46uad

(Amended 2026-10-04: the identifier-character rule, applied since 2026-05-12,
added to the rejection list with REQ-h5ret5's owner-ruled amendment.)

**LLR-5w2jx8**: a `HandleSkeleton` is built only from a `&Handle` and holds
its UTS#39 skeleton, so that two handles rendering alike share a skeleton.
satisfies: REQ-m8aexh

**LLR-pys2ek**: a member holds at most `MAX_DEVICES` device keys, and
`MAX_DEVICES` is 4 because the device sub-trie is a fixed depth-2 binary tree
of four slots. satisfies: REQ-xdx2c2

**LLR-w5nkbu**: `Name::parse` and `Surname::parse` (and their `TryFrom`
impls) are the only constructors of a `Name` and a `Surname`; each stores the
NFC form, bounded at 128 bytes after normalisation, and a field exceeding its
bound is rejected as `FieldTooLong { field, max }` naming which field and which
limit. satisfies: derived

**LLR-paxj7b**: `MemberLeaf::new` rejects a member constructed with no device
key, so that the zero-device state is reachable only through the isolation
operation. satisfies: derived

**LLR-4czn8t**: the `Debug` rendering of a member record redacts the handle,
the name and the surname, and the `Debug` rendering of a `Handle`, `Name` or
`Surname` on its own redacts its value. satisfies: derived

**LLR-xyv6p9**: deserialising a device key set accepts only a strictly
increasing, duplicate-free list within `MAX_DEVICES`, and accepts the empty
list. satisfies: REQ-shk82j, REQ-xdx2c2

**LLR-68tka5**: deserialising a member record decodes its handle, name and
surname through `Handle::parse`, `Name::parse` and `Surname::parse`, so a
value the software would refuse cannot enter through the wire form.
satisfies: REQ-shk82j

LLR-pys2ek is owed. `org-members/docs/requirements/2026-08-31-org-membership.md`
says so in as many words under REQ-xdx2c2: the bound "belongs in a low-level
requirement under the software item that owns the sub-trie, which does not exist
yet", and until it is written the number 4 "lives only in places no gate reads".
This is the LLR that closes that note. REQ-xdx2c2's own text and note are not
edited — this project records corrections by date rather than by rewrite; the
discharge is recorded in this change's risk draft instead.

Two items here are carried on one side only, and stay that way. Recorded
2026-09-17, after the verification gate found eighteen of the thirty-five items
tested on one side of the class C robustness rule. Ten of the eighteen gained
the missing side from a test already in the suite, each proved by its own
mutation; the other eight are argued in place, in this section and the three
below, because an annotation on a test that does not exercise the missing side
is the unearned evidence the mutation protocol exists to prevent.

The first is LLR-68tka5, and its missing side is real but has nowhere to go.
`deserialize_rejects_invalid_handle` opens with a valid record round-tripped to
establish the baseline — `let _: MemberLeaf = from_bytes(&bytes).unwrap();` —
before it builds the uppercase-handle payload it exists to refuse. Measured on
2026-09-17: that line is the only place in the whole suite where a `MemberLeaf`
carrying a valid handle is deserialised. `from_bytes` appears thirteen times
across the three test files and every other occurrence deserialises a
`P2pDeviceSlots` or an adversarial payload. So the normal case exists, inside
the one test that already holds the annotation, and there is no second test to
annotate.

The second is LLR-4czn8t, which has no abnormal input to give. Its `Debug` impl
writes three fixed `[REDACTED]` literals and never reads the fields it redacts,
so no member record — however long, however malformed before construction
refused it — can drive the formatter down a different path. The abnormal-input
axis is empty by construction, which is a stronger statement than "no test was
written for it".

## SDD-d6x85b — Domain-separated hashing and the device sub-trie

`org-members/src/hasher.rs`, `org-members/src/device_trie.rs`

**SDD-d6x85b**: the hash interface the membership record is committed
through — four separated domains, and the fixed-shape device sub-trie whose
root enters the member leaf hash. traces: REQ-xdx2c2

**LLR-72p8bz**: the hash interface provides four separated domains — member
leaf, member node, device leaf, device node — such that the same input hashed
in two domains yields two different values. satisfies: derived

**LLR-kdhd2v**: the device sub-trie is a fixed depth-2 binary tree of four
slots whose unoccupied slots hash a device empty sentinel, and device keys are
held sorted so that construction order does not change the record's hash.
satisfies: REQ-xdx2c2

Both items here are carried on one side only, and stay that way (2026-09-17).

For LLR-72p8bz the carrier is itself the adversarial case.
`hasher_domains_are_separated` hashes the same thirty-two bytes in all four
domains and asserts that all six pairs differ; feeding identical input across
domains is precisely the attack domain separation defeats. A hash over `&[u8]`
has no invalid input — every byte string is in the domain — so there is no
further abnormal input to supply.

For LLR-kdhd2v the out-of-range case never reaches the item. A fifth device is
refused by `P2pDeviceSlots`, in `new()` and in the `Deserialize` impl, before
`compute_device_root` is ever called, so the abnormal-input evidence for the
four-slot shape sits under LLR-pys2ek and LLR-xyv6p9 where those refusals and
their mutations already live. The degenerate case — a record with zero or only
some slots filled — is *executed* by several tests
(`device_slot_order_does_not_change_the_root` fills two of the four, the
isolation tests fill none) and asserted by none that a mutation of
`device_trie.rs` could discriminate: each of them compares two roots, and a
change to the sentinel or to the tree's shape moves both sides of that
comparison together. Annotating one would buy a name and no evidence.

## SDD-d9svdj — Sparse Merkle trie store

`org-members/src/smt.rs`, `org-members/src/node.rs`

**SDD-d9svdj**: the immutable 256-level sparse Merkle store — addressing,
path-copying, lazy hashing and the diff walk — which holds member records
without interpreting them. traces: REQ-d3prca, REQ-avmu3j, REQ-crjxk8

**LLR-zbe553**: a member is addressed by the 256 bits of its identifier, index
0 being the most significant bit of the first byte. satisfies: derived

**LLR-wm5hpc**: the hash of an empty subtree at each of the 257 levels is
computed once per hasher, so that an absent member contributes its level's
default and a trie emptied of members returns to the empty root.
satisfies: derived

**LLR-tk4qxu**: a modification leaves the trie it was applied to observably
unchanged, producing the modified membership as a separate value.
satisfies: REQ-d3prca

Narrowed on 2026-09-15, before this file was ever merged, after T5 measured
that the clause it used to carry could not be verified. As first written the
item also said a modification "copies only the path from the changed leaf to
the root, sharing every other node". That is true, and it is how the code
works — but it is **structural sharing, which is invisible at this item's
interface**. T5's nearest compilable surrogate (returning the shared node
instead of path-copying whenever it already has a hash) reddened more than
fifty tests without reddening either of this item's carriers, because no
assertion over the original trie can see the difference: the shared tree is
never written either. An LLR clause no test can discriminate is a defect in
the item, so the clause is now design rationale in this paragraph rather than
a requirement in the item.

What remains is verified twice over, and the second way is worth stating.
`insert_does_not_mutate_original` and `delete_does_not_mutate_original` assert
it. Underneath them it is **guaranteed by the type system rather than by a
test**: `Node` has no interior mutability except the write-once `spin::Once`
hash cell, and every `OrgTrie` operation takes `&self`, so there is no
in-place mutation for a test to catch. That is a stronger guarantee than a
passing test, not a weaker one — but it is also why no mutation reds these two
carriers, and a reader who expects a red→green row here should read this
paragraph instead of assuming the row was forgotten.

**LLR-n7nya3**: each node's hash is a write-once cell; `recalculate()` fills
every unset cell bottom-up, and the root value is refused with
`HashesNotCalculated` until it has. satisfies: REQ-avmu3j

**LLR-4n8zqx**: the root of a trie is determined by its member set alone, not
by the order the members were inserted in. satisfies: derived

**LLR-8jttpb**: the diff walk descends the left subtree before the right, so
removed identifiers and upserted records are produced in strictly increasing
identifier order. satisfies: derived

LLR-8jttpb is the load-bearing one. It is the reason the canonical form in
SDD-55b2zj holds *by construction* for every delta this crate produces, rather
than only being checked on deltas it receives.

Three items here are carried on one side only, and stay that way (2026-09-17).

For LLR-zbe553 the boundary values are the carrier.
`member_id_bit_indexes_msb_first` asserts indices 0, 1, 254 and 255 — both ends
of the 256-bit address. Every thirty-two-byte value is a valid identifier, so
there is no malformed member id to supply.

**Corrected 2026-09-17: the rest of this waiver used to rest on a false
premise.** It read "An index of 256 or beyond would panic in the byte lookup,
but no caller can supply one (every call site is bounded by `SMT_DEPTH`)". The
parenthetical is true of the crate's **internal** call sites and false of
external ones: `MemberId::bit` is a `pub` method on a `pub` type re-exported
from the crate root, its parameter is a `u16`, and nothing rejects 256..=65535 —
`index / 8` produces a byte offset of 32 or more into a `[u8; 32]` and the slice
index panics (`types.rs`). An external caller *can* supply one. The
abnormal-input waiver for LLR-zbe553 therefore does not stand on unreachability.

What is actually the case: the panic is real and reachable through the public
API, it is filed as **PR-jq43gx** (opened 2026-09-17, open) in
`../problems/2026-09-17-review-fixes.md` (written under its draft name
until `finalize-docs.sh` dated it; the reference was corrected 2026-09-17), and
the open question inside it is whether `bit()` is a "membership operation" in
REQ-ds8ryr's sense at all — an addressing primitive that adds nothing and
rejects nothing, or public input the requirement's "for any input" governs. The
owner's decision on 2026-09-17 was that the code is not touched here: fixing it
is a behaviour or surface change owed its own red-first change, the same
disposition PR-zz4exm carried (it was resolved by such a change on 2026-10-03).

So no abnormal-input test is added for LLR-zbe553 in this change — but the
reason is now *the deferral recorded in PR-jq43gx*, not a claim that the input
cannot arrive, and a test written later will be the one that discharges the
problem report rather than one that merely asserts a panic against LLR-h9gs32's
posture.

For LLR-wm5hpc the defaults take no input at all: the 257 empty-subtree hashes
are derived from the hasher when the trie is constructed. The item's only input
axis is the member set, and its degenerate values are exactly the two carriers —
an organisation that never held a member (`genesis_empty_is_ok`) and one emptied
by deletion (`add_then_delete_returns_to_the_empty_root`). Both are already
boundary-value tests; they read as normal-case only because what they assert is
success. The item's one error path, `InvariantViolated` when an `Empty` node is
met without a precomputed hash, is unreachable through the public interface and
is deliberately untested.

For LLR-tk4qxu the abnormal-input test is written and the red is impossible.
`mutations_preserve_original` in `fuzz_tests.rs` already drives operations that
may be refused — `let _ = original.add_member(..)`, `let _ = original
.delete_member(..)` — and asserts the original's root and member count
afterwards. What cannot be produced is a mutation that reds it, for the reason
the paragraph above already records: every `OrgTrie` operation takes `&self`,
`Node` has no interior mutability beyond its write-once hash cell, and a refused
operation returns `Err` without producing a trie at all. An annotation here
would be one the mutation protocol could never discharge.

## SDD-k5wa4n — Membership operations

`org-members/src/trie.rs`

**SDD-k5wa4n**: the organisation trie and its eight membership operations,
together with the handle and skeleton indexes that make uniqueness and
confusability decidable without walking the store. traces: REQ-crjxk8,
REQ-kmvc96, REQ-m8aexh, REQ-xdx2c2, REQ-ewdg2q, REQ-r784fu, REQ-avmu3j,
REQ-d3prca, REQ-h5ret5, REQ-ds8ryr, REQ-t46uad

**LLR-ub6dw9**: the trie carries a handle-to-identifier index keyed by
`Handle` and a skeleton-to-handle index keyed by `HandleSkeleton`, and every
operation that adds, changes or removes a member updates both.
satisfies: derived

**LLR-fv75ec**: `add_member` rejects an identifier already present, a handle
already held, and a handle whose skeleton matches one already held.
satisfies: REQ-crjxk8, REQ-kmvc96, REQ-m8aexh

**LLR-ch2pkw**: `genesis` applies the same identifier, handle and skeleton
checks as `add_member` to its initial member set. satisfies: REQ-crjxk8,
REQ-kmvc96, REQ-m8aexh

**LLR-j4d38d**: `delete_member` removes the member's handle and skeleton from
the indexes, so the handle becomes available to another member.
satisfies: REQ-kmvc96

**LLR-4phmjf**: `add_p2p_device` rejects a device the member already holds and
a member already holding `MAX_DEVICES`, and does not replace the member's key.
satisfies: REQ-xdx2c2

**LLR-s97ywt**: `delete_p2p_device` removes the device key and replaces the
member-as-a-group key in one operation, and isolates the member when the device
removed was the last; a replacement key equal to the member's current key is
refused with `P2pKeyNotReplaced`, checked after the member and device lookups,
and the refusal changes nothing — the device stays enrolled and the key stays
as it was. satisfies: REQ-ewdg2q

**LLR-w92psx**: `emergency_isolate_member` removes every device key and
replaces the member-as-a-group key in one step, retains the member in the
organisation, and leaves the member restorable by adding a device key; a
replacement key equal to the member's current key is refused with
`P2pKeyNotReplaced`, checked after the member lookup, and the refusal changes
nothing — every device stays enrolled and the key stays as it was.
satisfies: REQ-r784fu, REQ-ewdg2q

**LLR-k89ahd**: `rotate_p2p_key` replaces the member-as-a-group key and changes
no other field, the device set included; a replacement key equal to the
member's current key is refused with `P2pKeyNotReplaced`, checked after the
member lookup, and the refusal changes nothing. satisfies: derived

**LLR-mmst86**: `update_handle` takes the new handle as a `Handle` (valid by
construction, LLR-xzqs9r) and re-checks it for uniqueness and confusability
against every other member. satisfies: REQ-h5ret5, REQ-kmvc96, REQ-m8aexh

**LLR-g6arcs**: `update_name_surname` takes a `Name` and a `Surname`, so both
fields carry the NFC form and the bounds of LLR-w5nkbu by construction.
satisfies: derived

**LLR-v3jqau**: every operation naming a member that is not in the organisation
is refused with `IdNotFound`. satisfies: REQ-ds8ryr

### Amendments to LLR-s97ywt, LLR-w92psx and LLR-k89ahd

LLR-s97ywt as first written (2026-09-17) stated what the software did, which
was less than REQ-ewdg2q requires: the requirement's second clause — reject a
replacement key equal to the key being replaced — was not implemented, recorded
as PR-zz4exm, open since 2026-08-31. The LLR was deliberately worded to the
implemented behaviour, because an LLR restating the requirement would have been
satisfied by a test that could not exist. Amended 2026-10-03, together with
LLR-w92psx, in the change that fixes PR-zz4exm: the owner decided that an
unchanged replacement key is an error and the operation is atomic — refused
whole, never performed by halves. `emergency_isolate_member` removes device
keys too, so REQ-ewdg2q's second clause governs it as well, and LLR-w92psx now
says so. LLR-w92psx's refusal also covers a member who already has no device
keys, where nothing is removed and REQ-ewdg2q does not reach; there it rests on
REQ-r784fu, read as requiring that the operation *replace* the key — and
installing the key already held replaces nothing. `rotate_p2p_key`
(LLR-k89ahd) is not amended: it removes no device, so REQ-ewdg2q does not reach
it. (LLR-k89ahd amended after all, 2026-10-03, by owner ruling, in a later
change: `rotate_p2p_key` refuses the member's current key with the same error.
It stays derived — no requirement asks for it — and is assessed in
`../risk/2026-09-17-design-derived.md` under LLR-k89ahd. The same ruling settled
that a key no longer held — a member key used earlier, the key of a device
removed earlier — and a non-canonical encoding of a held key are not checked,
by any of the three operations; see PR-z463w5. A second owner decision the same
day, in the same change, refuses with `DuplicateKey` every replacement key still
held anywhere in the organisation, the key of the device being removed included:
LLR-fym7dy in `2026-10-03-key-uniqueness.md`.)

## SDD-55b2zj — Delta exchange and the trust boundary

`org-members/src/delta.rs`, and `apply_delta` / `calculate_delta` /
the canonical-form check in `org-members/src/trie.rs`

**SDD-55b2zj**: the change set that crosses a process boundary — anchored
to the record it was computed against, canonical in its encoding, and
usable only after its result is verified against a root supplied
independently of it. traces: REQ-4umsuz, REQ-shk82j, REQ-d3prca, REQ-wx3wpv

(Amended 2026-10-03: REQ-wx3wpv added to the trace, refined by LLR-n5t6bn in
`docs/architecture/2026-10-03-lawful-change-replicates.md`. The
item's purpose is a change set that crosses a process boundary; PR-vf5hdm
showed a class of lawful transition it could not carry.)

**LLR-au8het**: a change set names the record root it was computed against, and
applying it to a record with a different root is refused with
`DeltaBaseMismatch`. satisfies: REQ-4umsuz

**LLR-y38jfk**: applying a change set yields a candidate record that exposes no
member query, and only verification against an expected root turns it into a
usable record. satisfies: REQ-4umsuz

**LLR-7tdqv9**: verification refuses a candidate whose root differs from the
expected root, and the candidate is consumed either way. satisfies: REQ-4umsuz

**LLR-xmpqn2**: a change set is accepted only in canonical form: removals
strictly increasing and all present in the record, upserts strictly increasing
and each observably changing the record, and the two sets disjoint.
satisfies: REQ-4umsuz, REQ-shk82j

**LLR-juxk9q**: an upserted member record is checked for handle uniqueness and
confusability when the change set is applied, not when it is decoded.
satisfies: REQ-shk82j

**LLR-h7stq2**: `calculate_delta(old)` produces the change set transforming
`old` into the receiver, and is refused when either record has uncomputed
hashes. satisfies: derived

What LLR-xmpqn2 buys, and what it does not. Together with LLR-8jttpb it fixes
the *structure* of an accepted change set: removals and upserts each strictly
increasing, the two sets disjoint, no no-op upsert. The set of change sets
`apply_delta` will accept between two roots is therefore restricted to one
**value**, and every change set this crate produces is already in that form, so
an honest producer never trips an honest receiver's **canonical-form** check
specifically — `validate_canonical_delta` returns `Ok` for it. It
does not authenticate the change set, bind it to an organisation, protect
against replay across a root the record revisits, or decide whether the sender
was permitted to make the change. Those are the caller's, and LLR-y38jfk's
`expected_root` must reach the caller by a path the attacker does not control.
This is the crate's stated trust boundary, not an omission.

**Narrowed 2026-09-17, and the narrowing matters.** The sentence above used to
end "so an honest producer never trips an honest receiver's canonical-form
check", full stop, and every reader took it — correctly, as English — to mean
that a change set this crate produced is one an honest receiver will accept.
**That is false.** The round-2 independent review found a lawful membership
change the producer makes successfully, `calculate_delta` encodes canonically,
and `apply_delta` then refuses on the same base trie: a handle moving from one
member to another. The refusal does not come from `validate_canonical_delta`; it
comes from the handle-uniqueness block a few lines later in `apply_delta`, which
walks the upserts in one ascending pass and so still sees the outgoing handle in
the index when it checks the member acquiring it. So the sentence is defensible
on the narrow reading — the canonical-form check itself does pass — and false on
the reading it invites. It is now written to the narrow claim only. The defect
is **PR-vf5hdm** (opened 2026-09-17, open) in
`../problems/2026-09-17-review2-fixes.md`, which
has the mechanism, the two failing shapes, and the reason no hazard in the
register covers it.

**What it does NOT buy is a unique byte string, and until 2026-09-17 this
paragraph said it did.** The independent review at merge found the claim false
and it is corrected here rather than quietly dropped. Canonical form constrains
the decoded `Delta`; it says nothing about how many wire encodings decode to
it. `MemberLeaf` (`types.rs`) derives `Deserialize` over its `Handle`, `Name`
and `Surname` fields, and each of those decodes through its own `parse`, which
stores the NFC form — so decoding **normalises rather than rejects**. So an
NFD-encoded leaf and its NFC equivalent are two distinct postcard byte strings
that decode to the same `MemberLeaf`, yield the same `Delta` value and produce
the same root. The postcard encoding of an accepted change set is **not
injective**. `P2pDeviceSlots`' `Deserialize` is the contrasting case: it
genuinely refuses out-of-order, duplicate-bearing and oversized slot vectors
rather than repairing them, and says so in its own comment.

No code was changed to make this paragraph true. Correcting a false claim is
the fix; making the encoding injective — by rejecting non-NFC leaf fields the
way the device slots are rejected — is a **behaviour change**, it needs its own
red-first test, and folding it into the change that found the error would put a
product change where no reviewer is looking for one. Recorded here as the
reason the code is untouched, not as an oversight. Anything upstream that needs
byte-level identity must supply it itself; see the corrected assessment of
LLR-8jttpb in `../risk/2026-09-17-design-derived.md`, which had exported the
false property to `org-node`.

**Two clauses here are guaranteed by the type system, and no test can
discriminate them.** Recorded on 2026-09-16, after T7 measured it, so that a
reader who looks for a red→green row and finds none knows why:

- LLR-y38jfk's "exposes no member query" is enforced by `CandidateTrie`'s
  surface, which declares exactly `root_hash()` and `verify_against()`. The
  mutation that would test it — have `apply_delta` return the trie directly —
  is a signature change, and it produces seven `E0599` compile errors rather
  than a discriminating failure.
- LLR-7tdqv9's "the candidate is consumed either way" is enforced by
  `verify_against` taking `self` by value.

Unlike the clause withdrawn from LLR-tk4qxu, these two stay in their items.
That clause described *internal structure* — path-copying and node sharing —
which is invisible at the item's interface and so could not be a requirement on
it. These two describe **the interface itself**, and a compiler-enforced
interface property is stronger evidence than a passing test, not weaker. What
they are not is *test* evidence, which is why they are named here.

The handle-uniqueness half of **LLR-juxk9q** has no carrier. The item says an
upserted record is checked for "handle uniqueness and confusability" when the
change set is applied. `apply_delta` does check both — the block returns
`DuplicateHandle` as well as `ConfusableHandle` — but every `DuplicateHandle`
assertion in the suite reaches it through `genesis`, `add_member` or
`update_handle`, never through `apply_delta`. Removing the whole block reds one
test, `apply_delta_rejects_confusable_in_upsert`. So the confusability clause is
carried and the uniqueness clause is not: a delta upserting a member whose
handle duplicates an existing one would be rejected by the code, and no test
would notice if that stopped being true. A test for it is a small piece of work
and it is deliberately **not** done here — this change adds no test outside the
five its plan names, and an evidence gap that is written down is worth more than
one quietly closed at the end of an unrelated task.

## SDD-m9gs5g — Error reporting and panic-freedom

`org-members/src/error.rs`, and the crate-level lint posture in
`org-members/src/lib.rs`

**SDD-m9gs5g**: the single typed error by which every rejection leaves the
crate, and the lint posture that keeps a rejection from becoming a panic.
traces: REQ-ds8ryr

**LLR-h9gs32**: every rejected operation returns an `OrgMembersError` variant,
and the crate denies `unwrap`, `expect` and `panic` at the lint level so that
no input reaches a panicking path. satisfies: REQ-ds8ryr

**LLR-sa3ugj**: `MalformedDelta` renders the rule that was broken, and
`FieldTooLong` renders the field and the limit. satisfies: REQ-ds8ryr

LLR-sa3ugj's two carriers are both already its normal case (2026-09-17). The
item is about how two error variants render.
`malformed_delta_error_displays_reason` and
`field_too_long_error_displays_field_and_max` each construct the variant and
assert its `Display` output, and for a formatter that is the ordinary input,
not an abnormal one; the gate read them as abnormal because the values they
format happen to be errors. There is no second class of input to a `Display`
impl over a fixed variant — its fields are a `&'static str` and a `usize` the
crate chooses itself, never a caller.

## Trace corrections — 2026-09-17, round-2 independent review

The round-2 review checked every SDD's `traces:` list against the `satisfies:`
lines of the LLRs written under it, and found four disagreements. Each was
decided on its merits rather than by making the two lists mechanically equal: an
LLR whose REQ its parent does not trace is always a defect, because impact
analysis from that REQ stops at the item and never reaches the LLR; a `traces:`
entry with no refining LLR may still be correct, if the item genuinely realises
the requirement without any single LLR being the place it is refined.

**Two additions to SDD-k5wa4n — both defects, both added.**

- **REQ-h5ret5.** LLR-mmst86 (`update_handle` revalidates the new handle and
  re-checks it for uniqueness and confusability) sits under SDD-k5wa4n and
  declares `satisfies: REQ-h5ret5`. The parent did not trace it.
- **REQ-ds8ryr.** LLR-v3jqau (every operation naming a member that is not in the
  organisation is refused with `IdNotFound`) declares `satisfies: REQ-ds8ryr`.
  The parent did not trace it, and this is the more damaging of the two:
  impact analysis starting from REQ-ds8ryr reached only SDD-m9gs5g and silently
  missed LLR-v3jqau and the seven tests that carry it. A reader asking "what
  does changing REQ-ds8ryr touch?" got an answer short by seven carriers.

**Two removals from SDD-d6x85b — both strays, both removed.** The item is
`hasher.rs` and `device_trie.rs`: four separated hash domains and the
fixed-shape device sub-trie.

- **REQ-avmu3j** ("shall not report a membership root value for a record whose
  hashes have not been computed since its last modification, reporting an error
  instead") is a *refusal* behaviour. No refusal lives in either file; the
  refusal is `OrgTrie::root_hash`'s, and the store's write-once hash cells are
  LLR-n7nya3's under SDD-d9svdj. Hashing a value is not reporting a root.
- **REQ-4umsuz** (base mismatch and result-root mismatch on a change set) is
  SDD-55b2zj's entirely. Nothing in the hash interface or the device sub-trie
  realises any part of it.

Neither REQ is orphaned by the removal: REQ-avmu3j stays traced by SDD-d9svdj
and SDD-k5wa4n, REQ-4umsuz by SDD-55b2zj.

**Three entries with no refining LLR are kept, and here is why each is right.**

- **SDD-k5wa4n traces REQ-avmu3j.** `OrgTrie::root_hash()` — the function that
  returns `Err(HashesNotCalculated)` rather than a stale root — is defined in
  `trie.rs`, which is this item's file. The item realises the requirement at the
  interface the caller actually touches; LLR-n7nya3 refines the store mechanism
  that makes it possible. Both traces are correct and neither is redundant.
- **SDD-k5wa4n traces REQ-d3prca.** Every one of the eight membership operations
  takes `&self` and returns a new `OrgTrie`, which *is* the requirement ("leave
  an existing membership record unchanged when a modification is applied,
  producing the modified membership as a separate value"). It is realised by the
  shape of the whole item rather than by one LLR — and LLR-tk4qxu, which would
  have been that LLR, was deliberately narrowed on 2026-09-15 to what is
  observable at the interface.
- **SDD-55b2zj traces REQ-d3prca.** `apply_delta` takes `&self` and yields a
  `CandidateTrie`; the record the change set was applied to is untouched. That is
  the same requirement realised at the process boundary, and it is the reason
  `apply_delta` returns a candidate at all.

Tracing **REQ-crjxk8** on SDD-4yr9ge and on SDD-d9svdj is kept for the same
kind of reason — the member record is where the immutable identifier is defined, and
the store is what addresses members by it — but only one of the requirement's
two clauses is refined anywhere, which is Obligation G below.

## Obligations from the independent review — opened 2026-09-17

The independent review at this change's merge returned thirteen findings. Six
were fixed in the review-fix round; one raised no fault; six the owner disposed
of as **record, do not fix** on 2026-09-17. The three whose subject is this
ledger are below. The other three — uncarried normalisation sites, LLR-h9gs32's
lint-posture rationale, and the untracked SOUP advisory gap — are in
`../risk/2026-09-17-design-derived.md` under the same heading.

A **second** independent review, on 2026-09-17, added Obligation G below and
closed its own finding-7 in place — see the note after Obligation G. Its other
outputs were fixes rather than obligations: the trace corrections above, the
byte-uniqueness corrections in `org-members/README.md` and
`org-members/src/trie.rs`, the widening of PR-jq43gx to `DefaultHashes::at_level`,
the fourth site added to Obligation A in the risk file, and **PR-vf5hdm**.

**No SDD, LLR, HAZ or RC is minted for any of these.** Where an item looks
wanted, that is said and left to the owner.

### Obligation D (finding-2) — SDD-55b2zj's defining boundary has no end-to-end test

Opened 2026-09-17. **No test in the suite ever serialises or deserialises a
`Delta`.** All twenty-one postcard call sites across the three test files
encode or decode a `MemberLeaf` or a `P2pDeviceSlots`; none encodes the type
this design item exists for.

Why that is a gap in *this item* rather than a general wish for more tests:
SDD-55b2zj is defined as "the change set that crosses a process boundary", and
LLR-xmpqn2 — its canonical-form item — is worded about "the postcard encoding".
The boundary is what the item is, and the boundary is exactly what nothing
crosses in the tests. Everything that *is* tested about deltas —
`delta_apply_and_verify`, `delta_canonicality_fuzz`, `calculate_delta_roundtrip`
— builds a `Delta` value in memory, hands it to `apply_delta` in the same
process, and never lets it become bytes. The canonical-form checks are therefore
proved over values, and the encoding is proved over leaves.

**This is also why finding-1 went unnoticed.** The uniqueness claim corrected
above — that an accepted change set has exactly one postcard byte string — was
written in four places and survived six implementation tasks, a verification
gate and the change's own self-review, because nothing in the suite ever put a
`Delta` on the wire and brought it back. A round-trip test would not have
*proved* injectivity, but writing one would have forced the author to ask what
the deserialisation path does to the leaf fields, which is the question whose
answer is "it normalises them".

To act on later: a delta round-trip test — encode, decode, `apply_delta`, compare
roots — is the missing normal case, and its abnormal twin is a hand-built
non-canonical encoding refused on decode or on apply. Both are new tests, which
is why neither is written here: this round's brief was to change no test count.
An item may also be wanted, since LLR-xmpqn2's wording promises a property about
"the postcard encoding" that no LLR carries a test for; that is the owner's call.

### Obligation E (finding-8) — unmarked derived work under SDD-k5wa4n

Opened 2026-09-17. A public, documented capability of `OrgTrie` is named by no
clause of SDD-k5wa4n, no LLR, no REQ, and is not marked `derived`:

- `OrgTrie::pending_changes()` and `OrgTrie::has_pending_changes()`, together
  with the `last_calculated_root` field they read and the `InvariantViolated`
  guard on it. Five tests carry it. It is an **administrative review**
  capability — see what a trie has accumulated since its last calculated root
  before committing it — which is a reasonable thing for the unit to offer and a
  thing no requirement asked for.
- `orgtrie_is_send_sync` is a second, smaller case: a compile-time assertion that
  the trie crosses threads, verifying a property no item states.

Under this project's own rules a capability with no parent in the requirements is
`satisfies: derived` and must be assessed for hazard impact — the
`UNANALYZED-DERIVED` gate exists for exactly that. Here the capability is not
merely unassessed; it is **unmarked**, so the gate never sees it.

**`check-trace.sh` cannot find this class of gap, and that is the durable part of
the finding.** The checker flags an LLR with no test (`MISSING-TEST`) and design
with no requirement (`UNTRACED-DESIGN`). It never flags a *test* with no LLR, or
a public function with no item — there is no direction in the tooling that runs
from the code towards the ledger. Every gate in this change was therefore green
while a documented public capability sat outside the traceability graph entirely.
A reviewer reading the code found it; no script could have.

To act on later: either mint an LLR under SDD-k5wa4n for the pending-changes
capability, marked `satisfies: derived` and assessed in the risk file, or decide
the capability is internal and narrow its visibility. `orgtrie_is_send_sync`
likely wants the second treatment or an explicit note rather than an item. **No
LLR is minted here** — the brief for this round permitted one problem report and
nothing else, and minting an LLR would also owe a derived assessment, which is
`analyze-risks`' business and not a fix round's.

### Obligation F (finding-11) — `org-members/docs/CONTEXT.md` is still the template

Opened 2026-09-17. The unit's `CONTEXT.md` has never been filled in, and this
change is what makes that matter: it introduces the unit's internal vocabulary at
scale — thirty-five LLRs, six SDD items and two ledger files' worth of prose —
with no glossary of its own to introduce it against.

Worse than empty: **this change's own documents use terms the repository's root
`docs/CONTEXT.md` lists under _Avoid_** — "trie" throughout, and "SMT" /
"sparse Merkle store" for the same structure. The root file avoids them for a
reason (they are implementation vocabulary where the domain has plainer words),
and a unit that contradicts the root glossary without saying so leaves a reader
unable to tell a deliberate local usage from an oversight. The usage here *is*
deliberate — inside the unit that implements the structure, the structure's name
is the precise word, and "trie" is what the type is called in the code — but
deliberate-and-unwritten is indistinguishable from careless.

To act on later: fill `org-members/docs/CONTEXT.md` with the unit's vocabulary
and an explicit note that "trie" and "SMT" are used here, in this unit, against
the root glossary's advice, with the reason. That is a documentation change and
wants no item. It should happen before the unit's vocabulary is copied into
`org-node`'s architecture ledger, which is the next tooth.

### Obligation G (round-2 finding-5b) — REQ-crjxk8's immutability clause is refined by no LLR

Opened 2026-09-17. REQ-crjxk8 has **two** clauses:

> The software shall identify each member by **an identifier that does not
> change when that member's handle changes or when any of that member's keys
> are replaced**, and shall **reject an attempt to admit a member whose
> identifier is already present** in the organisation.

The second clause is refined twice — LLR-fv75ec and LLR-ch2pkw both declare
`satisfies: REQ-crjxk8` for exactly the duplicate-identifier rejection. **The
first clause is refined by no LLR anywhere in this ledger.** The three items
that touch it come close and stop short: LLR-k89ahd says `rotate_p2p_key`
"changes no other field", LLR-mmst86 is about revalidating the new handle, and
LLR-zbe553 is about how the identifier addresses the store. None of them says
the identifier survives.

The REQ still passes the gate — it carries direct `verifies:` annotations, and
`check-trace.sh`'s transitivity runs upward only, so a REQ covered directly owes
nothing beneath it. That is precisely why this is invisible: nothing reddens,
and the design layer simply has no statement of the property the requirement
exists for.

**It is trivially testable, which is the argument for closing it.**
`with_handle`, `with_p2p_key` and `with_p2p_device_slots` each produce a new
`MemberLeaf`; asserting `id()` is unchanged across all three is three assertions
in one test, with no new machinery. The obvious mutation — have one of them
write a different id — would red it immediately.

**No LLR is minted here.** An LLR for it would be `satisfies: REQ-crjxk8`, not
derived, so it would owe no derived assessment — but it would owe a carrier
under the mutation protocol, and this round adds no test. Minting an item whose
evidence cannot be produced in the same change is the failure mode this project
spent T3–T8 avoiding. The owner decides whether it lands as an LLR under
SDD-4yr9ge (where the member record is defined) together with its test, in one
change.

### Finding-7 of the round-2 review — closed in place, not deferred

2026-09-17. The review found that this ledger's Overview delegates the
decomposition picture to `README.md`'s Overview, and that `README.md`'s Overview
did not contain it: it named one of the six items (SDD-55b2zj, for the trust
boundary), described the split as four conceptual lines, and listed neither the
six items nor the files they own. A reader following the delegation had to come
back here anyway — and the README is the document that survives the per-change
files, so the six items and their responsibilities belong there.

This was **fixed rather than recorded**, because it is additive prose about items
that already exist, it mints nothing, changes no `traces:` or `satisfies:` line,
and adds no test: the cheapest safe correction available, where recording it
would have left a reader mis-directed for the sake of procedural tidiness.
`org-members/docs/architecture/README.md`'s Overview now carries the six items,
one line of responsibility each, and the source files each owns — so the
delegation this file makes is now true.

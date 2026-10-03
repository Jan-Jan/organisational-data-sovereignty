# Key uniqueness across the organisation

Low-level requirements added 2026-10-03 by owner decision, after the third
review round of the `rotate_p2p_key` change found that one member could be given
a key another member holds. The owner ruled that every key in the organisation
is held in exactly one place, enforced on every path that builds or changes a
record, the change-set path included, and for member keys and device keys alike.
They refine SDD-k5wa4n (membership operations) and SDD-55b2zj (delta exchange)
in `2026-09-17-decomposition.md`. All three are `satisfies: derived` — no
requirement states the invariant — and are assessed in
`../risk/2026-10-03-key-uniqueness.md`.

The *keys held in an organisation* are every member's member-as-a-group key and
every enrolled device key, compared by their 32 encoded bytes; a member key and
a device key with the same bytes are the same key.

**LLR-v6gfc7**: every record the software builds or accepts holds each key in
exactly one place — no two members hold the same member key, no device key is
enrolled under two members, and no member key equals an enrolled device key —
and `genesis`, `add_member` and `add_p2p_device` refuse an input that would
break this with `DuplicateKey`, after their existing checks. satisfies: derived

**LLR-fym7dy**: `rotate_p2p_key`, `delete_p2p_device` and
`emergency_isolate_member` refuse a replacement member key that is held anywhere
in the organisation before the operation — another member's key or any enrolled
device key, the key of a device being removed included — with `DuplicateKey`,
after `P2pKeyNotReplaced`, and the refusal changes nothing. satisfies: derived

**LLR-gjj6bx**: `apply_delta` refuses, with `DuplicateKey` and after its existing
checks, a change set whose resulting record breaks LLR-v6gfc7's invariant.
satisfies: derived

LLR-gjj6bx deliberately checks the resulting record only, not keys held in the
base record. A change set is the collapse of a sequence of direct operations,
each judged against the record it ran on: `delete_p2p_device(D, K1)` followed by
`rotate_p2p_key(K_D)` is accepted step by step, because K_D is no longer held
when the rotation runs. A base-record check would refuse the change set that
`calculate_delta` computes from those two accepted steps, so peers would reject
an administrator's valid edits. The resulting-record invariant is preserved by
every sequence of accepted operations, so the two paths agree; a key no longer
held is the caller's to avoid on both (`org-members/README.md`, "Security checks
the caller MUST perform", item 11). An earlier draft of this item (2026-10-03,
same change) carried the base-record clause; the first implementation exposed
the disagreement and it was removed before merge.

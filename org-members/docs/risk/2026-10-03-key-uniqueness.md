# Key uniqueness — assessment of the derived requirements

assesses: LLR-v6gfc7, LLR-fym7dy, LLR-gjj6bx

The three items are derived behaviour by owner decision (2026-10-03), assessed
against the existing register (`2026-09-02-membership-hazards.md`).

**What they reduce.** HAZ-s39gbh — a removed device keeps access — has routes
the device-removal controls did not reach: a member key shared with another
member (one member's removed device keeps the other member's live key), a
device's own key installed as a member key (the device controls it outright),
and a key-replacing operation that installs the key of the device it removes.
LLR-v6gfc7 and LLR-fym7dy close those routes on the direct API, and LLR-gjj6bx
keeps the change-set path to the same invariant, so a change set cannot leave
two places holding one key. It checks the resulting record only — a change set
that removes a device and gives its key to a member in the same step is
accepted, exactly as the two direct operations in sequence are (see LLR-gjj6bx's
rationale) — and it does **not** close RC-mqtks7's wire-path bypass: a change
set may still remove a device and keep the member key unchanged, which is
not-minted control 2's subject. No path reaches a key that is *no longer held* —
the software keeps no key history, and the owner's ruling leaves that to the caller
(`org-members/README.md`, "Security checks the caller MUST perform", item 11).
HAZ-s39gbh is not re-scored here.

**What they introduce.** An unavailability pathway of the kind the register
already accepts for the direct-API control: an administrator whose tooling
reuses a key — for example one keypair serving as both a member key and a device
key — is refused with `DuplicateKey` where the operation used to succeed. That
is the intended effect. One place in org-node constructs exactly such a leaf: a
fallback in `receive_and_verify` that rebuilds a single-admin trie when an
admission carries no genesis snapshot, with the same key as member key and
device key; its own comment says it "should not happen in production", and it
already fails the base-root check. It now fails earlier, at `genesis`, still
closed. A second org-node path now refuses: `admit_member` builds the
new leaf from the joiner's Join request, and gets `Trie(DuplicateKey)` when the
joiner's member key equals its own device key or either key is already held in
the organisation. That is the intended effect; personas created by org-node have
distinct keys, so an honest joiner does not trigger it. The review that found
this also found that org-node derives `MemberId` from the member key, so a
revoked member rejoining with the same keypair re-adds the deleted identifier —
filed in org-node's own ledger (`org-node/docs/problems/`, the member-id-from-key
report), since it is that unit's design decision. A record built before this change that already shares a key is refused
when rebuilt by `genesis` or reached by `apply_delta`; none is known to exist.

**Not a hazard of the check itself.** The refusal is atomic by construction
(every operation takes `&self` and returns `Result<Self, _>`), and the check
compares encoded bytes, so a non-canonical encoding of a held key is not caught —
the same scope the owner set for `P2pKeyNotReplaced`.

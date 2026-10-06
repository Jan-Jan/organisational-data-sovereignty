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

**After the switch to `person`'s key types (owner, 2026-10-05).** A device is
identified by `person`'s `DevicePublicKey`, which accepts only the canonical
encoding of a point of prime order, so one DevicePublicKey has one byte string
and the byte comparison is exact for DevicePublicKeys. Member-as-a-group keys are `person`'s
`PersonPublicKey`, an X25519 u-coordinate. That narrows the check in two ways,
both accepted by the owner:

- One keypair reused as a DevicePublicKey (its ed25519 encoding) and as a
  Member-as-a-group key (its X25519 image) has different bytes in each role,
  so the refusal of a device's own key installed as a member key no longer
  catches that reuse. Accepted: a group key is a fresh key-agreement key from
  the group's CGKA (TreeKEM or BeeKEM), never a device's signing key, and the
  owner's group-key rules (2026-10-04) require it to differ from every
  DevicePublicKey; only tooling that deliberately departs from those rules
  reuses one, and such tooling is outside every analysed situation in this
  register.

  (Amended 2026-10-05, independent review round 3: no item on this branch
  states the group-key rules. The owner ruled them on 2026-10-04: no group key
  with zero DevicePublicKeys; with one or more, a group key distinct from
  every DevicePublicKey; a change whenever a DevicePublicKey is added, removed
  or replaced. They are recorded in `docs/adr/2026-10-04-group-key-rules.md`
  on the unmerged branch `worktree-person-requirements`, the branch
  `docs/plans/2026-10-04-person-shared-types.md` names for every earlier Person
  decision, and are to be realised by the planned Member key rules change
  (change 3 of `docs/plans/2026-10-04-person-sequencing.md` there). Until that
  change merges, org-members enforces "distinct from every DevicePublicKey"
  only by the byte comparison above, within one encoding: it refuses a
  Member-as-a-group key whose bytes equal a held DevicePublicKey's, and does
  not see the same keypair under its other encoding. org-node today derives
  each persona's Member-as-a-group key from its own seed, separate from the
  device seed, so its honest keys differ; that is org-node's construction, not
  a check.)
- A `PersonPublicKey` accepts the u-coordinate of a mixed-order point
  (`person`'s rule, owner 2026-10-05, matching RFC 7748), so one X25519 secret
  has up to 8 accepted byte strings with the same key-agreement output, and the
  byte comparison sees them as different keys. Accepted: the holder of that
  secret gains nothing by holding it twice that one encoding does not already
  give them, and a group key that must change on a device change is still
  compared against its predecessor's bytes by the caller (README item 11).

HAZ-s39gbh is not re-scored: neither narrowing opens a route by which a removed
device keeps access that the rules above do not already exclude.

The assessments of LLR-a645bx and LLR-z954wj, the key-validity rules the
switch adds, are in `2026-10-05-device-and-member-key-validity.md`.

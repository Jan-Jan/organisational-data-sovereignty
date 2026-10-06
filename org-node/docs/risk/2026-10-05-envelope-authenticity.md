# Envelope authenticity without a published signing key

The owner ruled (2026-10-04 and 2026-10-05, recorded in
`docs/plans/2026-10-04-person-shared-types.md`):

- the on-chain `orgPubKey` is the Organisation public key, an X25519
  key-agreement key (root `docs/CONTEXT.md`), so no Envelope can verify under
  it;
- which accounts may publish an Organisation state is decided on-chain, by the
  Organisation's multisig proxy, under standard Polkadot practice, and is not
  org-node's to decide;
- the Envelope signature is dropped, and nothing about the sender is checked
  (owner ruling 2026-10-05, change `worktree-org-node-chain-authority`): what
  the sender delivers is verified against the chain, which is the sole
  authority.

*Amended 2026-10-05 by docs/plans/2026-10-05-switch-trim.md.* The third
ruling first read that the connection's DevicePublicKeys authenticate the
sender, and this file minted a control that checked the sender against the
node's Invite or record on every Receive operation. The owner ruled that
nothing about the sender is checked, so that control was withdrawn before
merge and RC-pm9kmx is amended in place instead.

**What changes in HAZ-tawvm2's controls.** RC-pm9kmx settled authenticity
before any work was done on attacker-chosen bytes by a signature under a key
the chain named. It is amended in place: it keeps the Organisation binding
before the decode and checks no signature and no key of the sender. What the
chain names is no longer an author key but the Membership root and epoch, and
RC-6a2dke and RC-e5atck, unchanged, still require the applied Change set to
reach the root of an Organisation state the node read itself with an epoch
newer than the last it committed. Authority to change the membership
therefore rests wholly on the chain: only an account the Organisation's proxy
accepts can publish the root a Change set must reach. Any peer, not only a
device in the node's record, now reaches the decoder, whose surface is
org-members' class C deserialisation (REQ-shk82j). It cannot make the node
commit anything the chain has not published: a Change set that does not
reach the on-chain root is rejected by RC-6a2dke whoever sent it. HAZ-tawvm2
is not re-scored: its severity is unchanged, and the route to a commit the
chain did not publish stays closed by RC-6a2dke.

**The Sequence number** (*added 2026-10-05 by review round 1, finding-1*).
The signature bound the Sequence number to the administrator who chose it.
Without it, the number is whatever the sending device writes, and RC-m4r75s
only requires it to exceed the receiver's mark. A device the receiver accepts
(any peer, since nothing about the sender is checked) could rebuild a genuine Envelope
with the number set to `u64::MAX`. The Change set reaches the chain's root, so
the receiver committed it, with `u64::MAX` as its mark. From then on every
genuine Envelope was refused as stale, forever. The receiver's record froze:
later admissions and removals never reached it, its own removal included. That
is HAZ-vxabf9's situation: a device that never acts on a revocation keeps its
record and its keys, and its record keeps listing removed devices. The removal
of the jamming member does not help, because the jammed node never commits it.
The reviewer reproduced it on both Receive operations.

The owner ruled (2026-10-05) that the Sequence number must equal the epoch of
the Organisation state the receiving node read itself and verifies against.
The chain decides that epoch, and only an account the Organisation's proxy
accepts can move it. So the mark a commit writes is one the chain has
reached, and no sender can choose it. The senders set the number to the epoch
their own chain update produced. The control:

**RC-95dgg8**: the node commits an Envelope only when its Sequence number
equals the epoch of the Organisation state the node itself read from the chain
in the same Receive operation and verified it against, and refuses any other
number with a typed error before committing, so that the Sequence-number
high-water mark is never a value the chain has not reached. mitigates:
HAZ-vxabf9

REQ-txvtm9 implements it, and LLR-9f5hmr states it at
`verify_envelope_against_chain`. RC-m4r75s is unchanged: the number must still
exceed the mark, and the check still runs before the decode. RC-95dgg8 runs
after the chain read, with the other chain checks, so it adds nothing a
sender can reach before the decode. HAZ-vxabf9 is not re-scored: the control
closes a route to the hazard this branch opened, and the hazard's other
routes are unchanged.

**The Organisation secret** (*added 2026-10-05 by review round 1, finding-4,
on the owner's ruling of that day*). `WireMessage.org_secret` was never covered
by the signature. The signature covered the Envelope, and the secret travels
beside it, so any relay on the path could swap it before this branch. What
this branch adds is the wider sender set: on an admission or an update, any
peer reaches the write that stores the secret
(`receive_and_verify`, LLR-ckk5nz), not only a relay between the administrator
and the receiver. Any peer can therefore hand a member a secret it chose. The chain cannot catch it, because the secret is not part of what
is verified against the chain. That bears on HAZ-ep6uzs: a member whose
secret a peer chose shares it with that peer. The owner ruled that
the Organisation secret is to be replaced by CGKA keys a member verifies
against the Organisation public key the chain records. No interim control is
minted, because it would authenticate a value that is being removed. The gap
is booked as PR-ve9zw8, open until that replacement.

*Amended 2026-10-05 (owner answer Q2 to docs/plans/2026-10-05-switch-trim.md).*
The trim merges on its own, with this gap recorded: until chain-authority's
change 2, any peer that relays a genuine admission or update can substitute
the Organisation secret. Change 2 makes the secret the Organisation private
key and refuses it on receipt unless its X25519 public half equals the
chain's `org_pub_key`; that receipt check is named in PR-ve9zw8 as its fix.
The CGKA wording above is kept beside it, unranked: whether the receipt check
supersedes it was not ruled. The residual is recorded under RC-b6mydy in
`org-node/docs/risk/2026-09-09-org-node-hazards.md`.

## Derived requirements assessment

REQ-txvtm9 (the epoch rule) realises RC-95dgg8, assessed above, and states
only the rule REQ-nhe2zu lacks. It refuses Envelopes; it admits none. No new
hazard.
assesses: REQ-txvtm9

REQ-8jb4ny (an Organisation state whose Organisation public key is not a
valid X25519 key is refused) and REQ-ech45n (a created Organisation publishes
a fresh X25519 Organisation public key): no hazard impact. They narrow what
the node accepts from the chain and publish a key no Envelope check reads; the
Organisation public key's use, letting members check the Organisation private
key shared with them, has no implementation in org-node yet, and the private
key is held only in the creating node's encrypted store.
assesses: REQ-8jb4ny, REQ-ech45n

## Derived low-level requirements

*Rewritten 2026-10-05 by docs/plans/2026-10-05-switch-trim.md.* The master
decomposition's items that described a signed Envelope are amended in place
(`org-node/docs/architecture/2026-10-03-decomposition.md` and
`2026-10-04-type-safety.md`), and the replacements this file assessed were
withdrawn before merge. Each passage below assesses the amended old item, or
an item new on this change, against what the old item's assessment in
`org-node/docs/risk/2026-10-03-architecture-derived.md` or
`org-node/docs/risk/2026-10-04-type-safety.md` relied on.

**The administrator's write path: LLR-rb8r65, LLR-ghja3x, LLR-6dc598,
LLR-tax3pm, LLR-8hdu9x.** Amended in place without their signing clauses.
Sections A and D of the architecture risk file judged these failures as
availability losses, not integrity losses, because the receiver re-derives
everything and commits only on a root match (RC-6a2dke, RC-e5atck). That
argument did not use the signature, so it holds. One mechanism changes. A
sender-side mistake that used to fail the receiver's signature check now
fails the chain checks, or is committed if it matches the chain. That is
still a change that does not take effect, or one that takes effect as the
chain says, not one that takes effect wrongly. No new hazard and no new
control. *Amended 2026-10-05 by review round 1:* LLR-ghja3x and LLR-tax3pm
now set the Sequence number to the epoch the sender's own chain update
produced, not one past the record's last, as RC-95dgg8 requires. A sender
that got it wrong is refused by the receiver's RC-95dgg8 check, which again
is a change that does not take effect.
assesses: LLR-rb8r65, LLR-ghja3x, LLR-6dc598, LLR-tax3pm, LLR-8hdu9x

**The update calldata: LLR-rv4vux.** Amended in place to name the
Organisation public key where it named the Published signing key. No
receiver uses that key for any check, because it authenticates nothing (see
the assessment of REQ-8jb4ny and REQ-ech45n above). A malformed key on chain
is refused at the edge by `OrgState::from_chain`, so no receiver acts on that
state. A wrong but valid key misleads only a member who later checks an
Organisation private key against it, and org-node does not implement that
yet. Calldata layout errors stay fail-closed, as section A says. No new
hazard and no new control.
assesses: LLR-rv4vux

**Which Organisation an admission lands in: LLR-vdyu65.** Amended in place
without its "signs with that Organisation's administrator Persona" clause.
Section F's confused-deputy argument holds. With nothing signed, the wrongly
admitted change would be accepted on the on-chain root, which would be
genuine. The conclusion, and the test that carries it, are unchanged. No new
hazard and no new control.
assesses: LLR-vdyu65

**The administrator's key on a member's record: LLR-rys5nx.** New on this
change, reworded by the owner's answer Q1. On a first admission for which an
Invite was imported, the key comes from the Invite's `admin_member_key`; on
one without an Invite, which now commits, it is the chain's Organisation
public key (LLR-xq9nrq, amended in place), which names no administrator. The
bad outcome is the one LLR-xq9nrq's assessment named: a member made to believe
it administers, or handing on an invite naming someone else. From the Invite,
that needs a forged Invite, imported by the member itself out of band. From
the chain, the key is equal to no member key (REQ-ech45n), so no Persona is
taken for the administrator and `admin_persona_for_org` finds none. The key is
never taken from the Wire message. The Organisation public key on the record
is the chain's, read in the same operation. The field leaves org-node with
chain-authority's change 1. No new hazard and no new control.
assesses: LLR-rys5nx

**Which Persona a receive marks Active: LLR-e5c9ud.** Amended in place. The
administrator is now excluded by comparing the Persona's Member-as-a-group key
with the record's, instead of the Persona's DevicePublicKey with it. The two
comparisons agreed only while both were one ed25519 key. Without an imported
Invite the record's key is the chain's, and the comparison excludes only a
Persona whose member key equals it. The weakening of RC-wqgm2p that
LLR-e5c9ud's assessment recorded (PR-mdv38y) is unchanged. No new hazard; the
existing finding stands.
assesses: LLR-e5c9ud

**The Persona identifier: LLR-s7yu4k.** Amended in place to state the
sixteen-byte prefix of the X25519 member key the code uses. The identifier is
local to one store, and no cross-node identity rests on it. Two Personas in
one store share an identifier only if 128 bits of independently drawn keys
agree. No new hazard and no new control.
assesses: LLR-s7yu4k

**A keypair rebuilt from its seed: LLR-e58j8m.** Amended in place to drop the
signature clause; it is derived now. A rebuilt keypair with a different
public key would make a Persona's key in the trie differ from the one its
node uses; the round-trip tests refuse that. No new hazard.
assesses: LLR-e58j8m

## Derived low-level requirements added by review round 2

**The X25519 secrets wiped on drop: LLR-98ufry.** *Added 2026-10-05 by
review round 2 (finding-22).* A member seed and an Organisation private key
held in an `X25519Keypair` are overwritten with zeros when it is dropped,
and the type cannot be cloned. That narrows the time those secrets spend in
the process's memory, which is part of HAZ-45ucqx's exposure, and restores
what the ed25519 `SigningKey` the member key used to be gave. It does not
reach the persisted copies (`PersonaRecord`'s seeds,
`OrgRecord.org_private_key`) or the copy `member_seed` / `org_private_key`
returns, which are held in redacted secret types since the merge of master
`1feb608` (PR-hqwpg9, resolved there) and are not wiped on drop (*amended at
that merge: this said "which PR-hqwpg9 covers, or the copy `to_seed`
returns"*). No new hazard; HAZ-45ucqx's residual is amended in
`org-node/docs/risk/2026-09-09-org-node-hazards.md`. *Added 2026-10-05 by
review round 3 (finding-1):* that the drop itself wipes the bytes is verified
by inspection only; no test observes memory after a drop, so a change that
removed the call would pass the gate.
assesses: LLR-98ufry

## Derived low-level requirements re-traced by review round 3

**The Organisation private key's secret type: LLR-322xfu.** *Added
2026-10-05 by review round 3 (finding-8).* The item claimed REQ-ech45n, which
states none of its clauses, and is derived now. It gives the Organisation
private key every property LLR-sz4xhc gives the member seed, the device seed
and the Organisation secret: no `Display`, a `Debug` that shows no byte,
`Clone` but not `Copy`, and its bytes out only through `expose_secret` or
serialisation. That is RC-8a4xjb's formatting control applied to one more
secret, which REQ-y7tsft does not list. It lowers HAZ-45ucqx's exposure for
that key and adds no route to it: serialisation writes the key only into the
encrypted Persona store (LLR-3fwykc). `Clone` makes copies possible, as it does
for the other secret types, and none of the copies is wiped on drop; that
residual is the one recorded under LLR-98ufry above. Its key pair's public half
is the X25519 public key of those bytes, which `OrgPublicKey::parse` accepts,
so the key a created Organisation publishes is always one a receiver accepts
(REQ-8jb4ny). No new hazard and no new control.
assesses: LLR-322xfu

**`OrgRecord`'s `Debug` hides the Organisation private key: LLR-2dvhz8.**
Derived, re-traced 2026-10-05 for the same reason as LLR-322xfu. It narrows
what a record's `Debug` output reveals: the field's presence is printed and
its bytes never are, so a log or panic message that renders a record does not
leak the key. No new hazard; it reduces exposure of the secret REQ-ech45n
keeps.
assesses: LLR-2dvhz8

## Master's type-safety items amended in place

*Added 2026-10-05 after the merge of master `1feb608` (org-node type-safety);
reworded the same day by docs/plans/2026-10-05-switch-trim.md (owner answer
Q4).* These items first superseded master's; they were withdrawn before merge
and their text folded into master's items in place. Each is assessed against
master's assessment of the old item in
`org-node/docs/risk/2026-10-04-type-safety.md`.

**The Organisation public key's parse, `Debug` and cache: LLR-mmdu38.**
Amended in place. The parse is LLR-3jjgtw's X25519 rule, assessed under
REQ-8jb4ny above, in place of the interim Edwards-point rule. Under the
Edwards rule about half of the valid X25519 keys this branch publishes would
be refused at the chain read, and the small-order keys the X25519 rule refuses
would be accepted. The amendment therefore removes a wrong refusal and a
wrong acceptance, and refuses no key a genuine Organisation can publish
(`OrgPrivateKey::x25519_keypair` always yields a valid key). Dropping
`From<&P2pMemberKey>` removes the one route by which a member key became the
Organisation public key, which was PR-szkat6. The `Debug` clause and the
fail-closed cache are unchanged, and master's assessment of both stands: the
key is not secret, and a refused state leaves no cached state, so nothing is
verified against a superseded root (HAZ-tawvm2). No new hazard and no new
control.
assesses: LLR-mmdu38

**A key pair only from its typed secret: LLR-56hc77.** Amended in place;
master's assessment stands: no secret passes through a plain byte array, and
each secret type keeps RC-8a4xjb's formatting control. The role residual
master recorded changes shape. A member key pair and a device key pair are
now two types, `X25519Keypair` and `SigningKeypair`, so the swap PR-4b2v6p
describes no longer compiles. `X25519Keypair` carries no role of its own,
though. It holds either a member seed or an Organisation private key, and both
`member_seed()` and `org_private_key()` are public on it, so the
Organisation's key pair can hand its secret back as a `MemberSeed`, and the
reverse. Nothing does that today: each call site builds the key pair from the
secret type of the role it needs. A swap would publish a member's key as the
Organisation public key, or the reverse, which REQ-ech45n forbids;
`ensure_distinct_from` (LLR-sj7cd5) refuses an Organisation public key equal
to a genesis member's key when the Organisation is created. This residual
stays with PR-4b2v6p. No new hazard.
assesses: LLR-56hc77

**The pinned encodings: LLR-ayrdr8.** Amended in place; the same verification
obligation, with two pinned values changed. Each change is one of this
branch's deliberate format changes: the Organisation private key field in the
store (REQ-ech45n) and the Envelope without a signature (REQ-ag6kqm). Both
changed values were derived by hand from master's bytes, and the test keeps
master's values beside them, so a reviewer can check that each differs by
exactly the stated bytes. A store or Wire message written under master's
format does not decode under this one. That is a break in compatibility, not a
hazard: a store this branch cannot decode fails to open and nothing is read
from it, and a Wire message it cannot decode is refused before anything is
committed. Master's limit stands: fixed values show each type serialises as
before for those values, not for every value. No new hazard.
assesses: LLR-ayrdr8

# Organisation membership requirements

First requirements in the ledger. Scope: the membership capability as the ODS
product exhibits it, limited to what the `org-members` trie realizes. Phrased
from outside the software, so that `org-node`, `org-acl` and the Tauri
application trace into these same items as they are brought under discipline;
`org-members`' internals (path-copying, lazy hash cells, `spin::Once`, the
depth-2 device sub-trie) are design data and belong in low-level requirements
under an SDD item, not here.

Every item is `satisfies: derived`: these rules came from design and threat
reasoning, not from a written system-needs document, and there is no such
document to cite. Each is assessed in the risk ledger
(`docs/risk/2026-08-31-membership-derived.md`) as
`check-trace.sh` requires.

Every item also carries `(implements: RC-...)`, added on 2026-09-02 when the
hazard analysis in `docs/risk/2026-09-02-membership-hazards.md` enumerated the
hazards these rules were reasoned toward and minted the risk controls they
realise. The items stay `satisfies: derived`. Finding the hazard a rule
addresses gives the rule a justification, not a parent in system needs, and the
derived assessments remain the assessment of record. No requirement text changed
with the annotation: each control was worded from the behaviour the requirement
already stated, and where the software falls short of that behaviour it is the
residual risk that says so (REQ-ewdg2q, PR-zz4exm). (Corrected 2026-10-03:
PR-zz4exm is resolved; REQ-ewdg2q's remaining shortfalls are the wire path,
PR-z463w5 and PR-fzu25w — see the note under REQ-ewdg2q; the owner has since
resolved both by ruling, which moves those checks to the caller (security
check 11 in `org-members/README.md`) rather than removing them, and leaves the
wire path as the one gap in the software.)

## Membership identity

**REQ-crjxk8**: The software shall identify each member by an identifier that
does not change when that member's handle changes or when any of that member's
keys are replaced, and shall reject an attempt to admit a member whose
identifier is already present in the organisation.
(implements: RC-4u22ba)
satisfies: derived
exported: yes

**REQ-kmvc96**: The software shall reject an attempt to admit or rename a
member to a handle already held by another member of the organisation.
(implements: RC-n2taat)
satisfies: derived
exported: yes

**REQ-h5ret5**: The software shall normalize every member handle to NFC and
shall reject a handle that, after normalization, is empty, exceeds 128 bytes,
contains an uppercase character, contains `.`, or mixes Unicode scripts (`-`
being permitted alongside any script).
(implements: RC-n2taat)
satisfies: derived
exported: yes

**REQ-m8aexh**: The software shall reject a member handle whose UTS#39
confusable skeleton matches that of a handle already held by another member,
so that two members cannot hold handles that render alike.
(implements: RC-n2taat)
satisfies: derived
exported: yes

## Member keys and devices

**REQ-xdx2c2**: The software shall bound the number of device keys a member may
hold, shall reject an attempt to add a device key the member already holds, and
shall reject an attempt to add a device key to a member already at that bound.
(implements: RC-3ppkf6)
satisfies: derived
exported: yes

The bound is 4 today, and that number is deliberately NOT stated in the
requirement: it follows from the fixed depth-2 device sub-trie, so it is design
data. It belongs in a low-level requirement under the software item that owns
the sub-trie, which does not exist yet — the architecture ledger is tooth 6 of
`docs/plans/2026-08-26-ratchet-gap-analysis.md`. Until that LLR is written the
number lives only in places no gate reads: `MAX_DEVICES` in
`org-members/src/types.rs`, a doc-comment in `org-members/src/trie.rs` that
restates it as "`MAX_DEVICES` (4)", the test that exercises it, the crate's own
`AGENTS.md`, and the 2026-05-07 design document. Re-shaping the sub-trie changes
the bound without touching this requirement, which is the point of putting it
this way — and writing the LLR is what gives the number a controlled home.

**REQ-ewdg2q**: The software shall replace a member's member-as-a-group key in
the same operation that removes a device key from that member, and shall reject
a removal whose replacement key is the key being replaced, so that a removed
device cannot derive access from the key it held while enrolled.
(implements: RC-mqtks7)
satisfies: derived
exported: yes

The second clause is **not met today**: the replacement key is stored without
being compared to the outgoing one, so a caller that passes the current key back
removes the device and leaves its access intact. Recorded as PR-zz4exm rather
than softened away, because the intent is not in doubt — the operation exists to
cut off the removed device. The requirement states the behaviour the software is
supposed to have; the problem report tracks the distance to it.

Met on the direct API as of 2026-10-03: `delete_p2p_device` and
`emergency_isolate_member` refuse a replacement key equal to the current key
with `P2pKeyNotReplaced` and change nothing (LLR-s97ywt, LLR-w92psx; PR-zz4exm
resolved). Three gaps remain open and are not closed by this: the wire path,
where `apply_delta` still accepts a leaf with a device removed and the key
unchanged (the hazard register's not-minted control 2; org-node's REQ-q92yac);
a removal that installs a key the removed device held *earlier* (PR-z463w5);
and a device removed by `delete_member` followed by `add_member` with the same
key, which no device-removal operation sees (PR-fzu25w). "Met" here means met
by the two device-removal operations, not by every route to the same state.
(Ruled 2026-10-03 by the owner: a key no longer held is not refused, and a
deleted `MemberId` is never legitimately re-added; PR-z463w5 and PR-fzu25w are
resolved by ruling. Every key still held anywhere in the organisation is
refused, the removed device's own included — `DuplicateKey`, LLR-fym7dy, a
later decision the same day. The requirement's purpose — a removed device
cannot derive access from a key it held — therefore holds only if the caller
never supplies a key no longer held that a removed device holds the secret of,
nor gives a re-admitted member (under a fresh `MemberId`) such a key; that duty
is security check 11 in `org-members/README.md`. Only re-adding a deleted `MemberId` is outside the
contract. The wire path is the one gap in the software that stands.) The paragraph
above is kept as the record of the gap. (Narrowed 2026-10-03 by owner ruling: a
re-admitted member, under a fresh `MemberId`, is a new member and may bring the
keys their previous membership held when it was deleted; nothing granted to the
old `MemberId` carries over. Where a device removed from that membership was
compromised, the device still holds those keys' secrets and reaches what is
granted to the new member; fresh keys are then the joiner's choice, not a
check the software makes. This is residual risk accepted by the owner. The duty
not to supply any other key no longer held stands — a Member-as-a-group key
replaced earlier, or the Device key of a device removed earlier while the
member stayed — and a deleted member's keys are never given to anyone else.
The software keeps no key history: neither duty is enforced by org-members or
by org-node; the administrator who admits or rotates carries them.)

**REQ-r784fu**: The software shall provide an operation that removes every
device key from a member and replaces that member's member-as-a-group key in one step,
shall retain the member in the organisation when it does so, and shall restore
the member's access when a device key is next added.
(implements: RC-sq3yhp)
satisfies: derived
exported: yes

## Integrity of the membership record

**REQ-avmu3j**: The software shall not report a membership root value for a
membership record whose hashes have not been computed since its last
modification, reporting an error instead.
(implements: RC-ty8qdw)
satisfies: derived
exported: yes

**REQ-d3prca**: The software shall leave an existing membership record
unchanged when a modification is applied, producing the modified membership as
a separate value, so that a record already published cannot be altered in
place.
(implements: RC-3qn5xg)
satisfies: derived
exported: yes

**REQ-4umsuz**: The software shall reject a set of membership changes whose
declared base does not match the membership record it is applied to, and shall
reject the result of applying a change set whose membership root does not
match the root the change set was expected to produce.
(implements: RC-9z65hw)
satisfies: derived
exported: yes

**REQ-shk82j**: The software shall re-validate every member handle and device
key set that it receives from outside the process — handle validity per
REQ-h5ret5, and a device key set that is sorted, free of duplicates, and within
the bound of REQ-xdx2c2 — so that a serialized membership record cannot
introduce a member whose handle or device set the software would refuse.
(implements: RC-4apk6w)

Deliberately NOT "the same rules as a member admitted directly": the wire path
accepts an empty device key set where direct admission requires at least one,
and a member arriving this way is checked for handle uniqueness and confusables
when the change set is applied rather than when it is decoded. Both are existing
behaviour with tests; naming them here keeps the requirement from asserting an
equivalence the software does not implement.
satisfies: derived
exported: yes

## Robustness

**REQ-ds8ryr**: The software shall report every rejected membership operation
as an error and shall not panic, for any input, including input that is
malformed, hostile, or exceeds a documented limit.
(implements: RC-c4truv)
satisfies: derived
exported: yes

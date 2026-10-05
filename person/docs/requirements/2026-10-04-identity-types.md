# Shared identity types requirements

Scope: the validated types `person` owns and org-members is to use — names,
device keys, the PersonPublicKey, device slots and the device sub-trie root. This
change builds them in `person`, taken from org-members' code; org-members
switches to them in a following change (branch
`worktree-worktree-person-shared-types`), after which its own copies are
deleted, because `docs/adr/2026-10-04-parse-at-the-system-edge.md` forbids a
second, parallel copy and `person` must not depend on org-members (owner,
2026-10-04). Until that change merges, org-members keeps its originals, and its
code and requirements are untouched here; this change adds only a problem
report in org-members' ledger. The Person definition built on these types is a later change
again (branch `worktree-person-requirements`,
`docs/plans/2026-10-04-person-sequencing.md` there).

These items state what `person` guarantees to any unit that uses the types.
org-members' own requirements (the device bound, re-validation of records from
outside the process, and the rest) are unchanged by this change and by the
switch: they describe org-members' observable behaviour, and org-members'
tests keep verifying them.

The PersonPublicKey is the key a grant to a Person or to a Member is encoded
against (owner, 2026-10-04; it replaces org-members' `P2pMemberKey`). It is an
X25519 public key, the key-agreement root key.

Every item is `satisfies: derived`: the rules come from the existing
org-members design and owner decisions, not from a written system-needs
document.

**REQ-r7mytp**: The software shall normalise every name and surname to Unicode
NFC and shall reject one whose NFC form is longer than the documented bound in
bytes.
satisfies: derived
exported: yes

**REQ-q6xkna**: The software shall construct a DevicePublicKey only from the
canonical encoding of an ed25519 public key whose point is of prime order
(torsion-free and not the identity), and shall reject any other input.
satisfies: derived
exported: yes

**REQ-3vqs9b**: The software shall construct a PersonPublicKey only from a
canonical encoding of an X25519 public key that is not of small order, and
shall reject any other input.
satisfies: derived
exported: yes

REQ-3vqs9b accepts a canonical u-coordinate of a point on the quadratic twist
of Curve25519 (owner, 2026-10-05): RFC 7748, libsodium and the key-agreement
libraries the key interoperates with accept one, and the risk file records
why it is safe.

**REQ-4szc22**: The software shall keep a set of DevicePublicKeys in sorted
order with no key twice and no more than the documented bound, whatever order
they are given in, and shall reject a set over the bound or containing a key
twice.
satisfies: derived
exported: yes

**REQ-tq4ms4**: The software shall reject a set of DevicePublicKeys read from
outside the process unless it is already in strictly increasing order.
satisfies: derived
exported: yes

**REQ-mu3qgz**: The software shall compute, under a caller-chosen domain, a
device root that is the same for equal sets of DevicePublicKeys and differs
for different sets.
satisfies: derived
exported: yes

**REQ-aj6x3n**: The software shall compute a different device root for the
same set of DevicePublicKeys under a different domain.
satisfies: derived
exported: yes

**REQ-vxx8k3**: The software shall not panic for any input, including input
that is malformed, hostile, or exceeds a documented limit.
satisfies: derived
exported: yes

**REQ-bczz87**: The software shall report every rejected construction of these
types as an error that identifies the rule broken.
satisfies: derived
exported: yes

**REQ-7gz72r**: The software shall report, for any 32 bytes, whether they are a
canonical encoding of an X25519 public key that is not of small order, by the
same rule REQ-3vqs9b applies, so that a unit defining its own X25519 key type
applies that rule without restating it.
satisfies: derived
exported: yes

REQ-7gz72r exists because the organisation's public key is an X25519 key that
belongs to org-node, not to `person` (owner, 2026-10-04), while the parse-at-
the-system-edge ADR forbids a second copy of the validation rule.

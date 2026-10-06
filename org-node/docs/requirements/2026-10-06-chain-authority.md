# org-node chain-authority requirements

Decided 2026-10-05 in the `grill-requirements` interview that PR-szkat6
(`org-node/docs/problems/2026-10-04-org-key-conflation.md`) opened; PR-szkat6
stays open until change 3 (the Organisation key pair, below). The
node verified every Envelope under the Organisation state's `org_pub_key`, which
genesis set to the founding Member's key, while the design intends
`org_pub_key` to be the public half of the Organisation key pair whose secret
every Member holds.

*Amended 2026-10-06 (independent review round 2, finding-9).* This said the
interview "resolves PR-szkat6", contradicting the sequencing below, which
defers PR-szkat6 to change 3.

Owner statement (2026-10-05): the right to change Organisation data lies in the
on-chain multisig proxy and is, for now, wholly independent of org-members and
org-node. A Member who receives updated Organisation information, including
the Organisation private key, verifies it against the on-chain state. org-node
and org-members have no concept of an administrator: an update is checked
against the chain, never against an individual Member's keys.

Owner rulings (2026-10-05):

- An Envelope carries no authority signature. The on-chain root match at a
  newer epoch is the sole authority over a received Change set. REQ-ag6kqm and
  RC-pm9kmx are amended in place to that rule, with a dated note; retiring them
  is not expressible in the gate, and a supersession would ask a test to verify
  behaviour that no longer exists.
- Nothing about the sender of an Envelope is checked, on first admission or
  after: the check that a first admission's sender is the invite's device and
  the check that an update's sender is a Member's device are both removed.
- Every administrator field leaves org-node: the Organisation record's and the
  Invite's administrator Member-as-a-group key and the Invite's administrator DevicePublicKey.
- Genesis creates an X25519 Organisation key pair. `OrgPublicKey` is checked by
  `person`'s exported X25519 rule
  (`person/docs/requirements/2026-10-04-identity-types.md`); org-node declares a dependency on
  `person`.
- The Organisation secret becomes the Organisation private key. An Envelope
  carrying one is refused, record unchanged, unless its X25519 public half
  equals the on-chain `org_pub_key`. With none, the stored key is kept only
  while its public half equals the on-chain `org_pub_key`, and is cleared
  otherwise. This rules on PR-xwek5e. Generating a new key pair on removal
  (rotation) is out of scope and recorded as a problem report.
  *Superseded 2026-10-06 by owner ruling* in
  `org-node/docs/requirements/2026-10-07-org-key-pair.md`:
  a message carrying Organisation information always carries the key (its
  absence is a parse error), and a revocation carries none and leaves the
  stored key in place.
- Stores written before this change are not supported: the code is not
  deployed, and no migration or requirement covers them.
- `admit_member` always sends the Organisation private key from the sender's
  record; the caller supplies none (the `org_secret` argument and the app's
  `org_secret_hex` go).
- Every item whose behaviour these rulings remove is amended in place to a
  testable statement of the new behaviour, with a dated note naming the ruling
  (about four REQs, two RCs and twenty-five LLRs; the inventory is in the
  plan).
- There is no `admin_member_key` and no `admin_device_key` anywhere in
  org-node: they serve no purpose and are removed.
- org-node does not decide who acts for an Organisation: knowing who the
  administrators are lies outside org-node. org-node exposes provisional
  operations (admit, revoke, export an Invite, build a Change set); a
  provisional update becomes the Membership record's ground truth only after
  the administrators have updated the chain and the update has verified
  against it, on the sender's node as on every receiver.
- org-node does not write to the chain. Genesis, update submission and the
  proxy ceremony move to on-chain-client; the app submits, then hands the
  result to org-node. org-node drops its sr25519 multisig key.

- The invitation exchange is outside org-node and happens in the app: an
  administrator sends a prospective Member an Invite (Organisation name, the
  Organisation's on-chain key, a guess of the Member's full name) over a third-
  party channel; the Member replies with their device and Member public keys,
  handle, name and surname. Invites, Join requests and pending Invites leave
  org-node, and with them the Invite's dialling address, which nothing read.

  *Correction 2026-10-06 (found by the change's review pass).* The two bullets
  above were written before the owner's invitation journey and are kept as
  stated then. Since then: exporting an Invite is the app's, not one of
  org-node's provisional operations (org-node's are create, admit, revoke and
  discard); the Invite carries the Organisation name, the Organisation
  identifier, a guess of the invitee's full name, the inviter's
  DevicePublicKeys and an invite identifier, not the Organisation public key
  (`app/docs/requirements/2026-10-06-invitation.md`,
  the requirement that produces an Invite); and "administrators" there means the signatories of the
  Organisation's on-chain multisig, a role org-node has no notion of.
- org-node's job is to verify the latest Organisation information against the
  chain and move from the last verified Organisation information to the new
  verified one. When the update removes this Member, the Organisation
  information this Member holds becomes empty.
- Verification stays in org-node (owner ruling, after weighing a move of the
  Membership-record step into org-members). org-node keeps sending and
  receiving updates and keeps reading the chain through on-chain-client.
- org-node may learn, through on-chain-client, whether a user is a signatory of
  an Organisation's multisig. That is advisory, for the app to decide what to
  offer, and is never an input to verification.

## Requirements

**REQ-xs4ab8**: The software shall build, for creating an Organisation,
admitting a Member and revoking a Member, a provisional update — the Change
set and the Membership root it produces — and keep it in the Persona store,
alongside any other provisional updates for that Organisation, without writing
to the chain and without changing its record of the Organisation.
satisfies: derived

**REQ-uv3v5w**: The software shall, when an update for an Organisation commits
— one it built or one it received — discard its previous record of that
Organisation and every provisional update it holds for it whose base is no
longer its record's Membership root.
satisfies: derived

**REQ-fwfku9**: The software shall refuse, with a typed error naming the limit,
to keep a provisional update that would bring the provisional updates it holds
for one Organisation above 1 MiB in their stored form, and shall keep nothing
in that case.
satisfies: derived

REQ-xs4ab8, REQ-uv3v5w and REQ-fwfku9 follow the owner's ruling (2026-10-05)
that provisional updates are persisted: agreeing a change between
administrators and writing it to the chain can take days and restarts. A
Membership record is immutable, so several roots can be held at once, and the
old one is discarded when a provisional one verifies against the chain. The
bound is by size, not count (owner ruling), at the wire frame's 1 MiB.

**REQ-tqap3r**: The software shall commit a provisional update it built to its
own record only after that update verifies against the Organisation state on
the chain by the checks a received update must pass, and shall leave its record
unchanged when it does not.
satisfies: derived

**REQ-f2k4tr**: The software shall, for a received Envelope about an
Organisation it holds a record of, reject it with a typed error and without
reading the chain when it names another Organisation, when its Sequence
number is not greater than the highest committed, when it or its Change set
does not decode, or when the Change set's base is not its record's Membership
root. (implements: RC-mj6gjq)
satisfies: derived

**REQ-8amu2a**: The software shall read the chain for a first admission only
when the Organisation it names matches an admission the app has declared it
expects, and shall reject
any other first admission with a typed error and without reading the chain,
leaving every declared expectation in place. (implements: RC-2ferct)
satisfies: derived

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
The item also required the Wire message's invite identifier to match the
expectation's. The owner ruled that the invite identifier never travels
between peers (only in the Invite and its reply), so an expectation names the
Organisation alone. Pre-emption stays controlled by REQ-kt877x: the verified
record must list one of the node's own unbound Personas.

**REQ-yp75u9**: The software shall bind each Persona to at most one
Organisation: it shall refuse, with a typed error and changing nothing, to
create an Organisation with a Persona already bound to one, and it shall never
bind a Persona already bound to one Organisation to another, on a first
admission or on any commit.
satisfies: derived

REQ-yp75u9 follows the owner's ruling (2026-10-06, independent review round 1,
finding-1): a Persona's keys are generated for one Organisation and link to no
other identity, so a Persona is never reused across Organisations. Without
this rule a second founding, or a reply to an Invite made with a Persona
already in use, moved the Persona's binding: the first Organisation could no
longer be managed, and the next update received for it read as this node's
removal and deleted its record, Organisation private key and proxy account.
It also closes PR-mdv38y.

**REQ-hhva9d**: The software shall, on request, discard a provisional update
it holds, together with any Organisation private key that update holds, and
shall refuse with a typed error, changing nothing, a request naming no
provisional update it holds.
satisfies: derived

REQ-hhva9d follows the owner's ruling (2026-10-06): administrators may never
agree a drafted change, and a genesis that is never written to the chain would
otherwise keep its Organisation private key in the store indefinitely.

**REQ-kt877x**: The software shall commit a first admission only when the
Membership record it verified against the chain lists a DevicePublicKey of one
of its own Personas, and shall otherwise refuse it with a typed error, leaving
its records and its declared expectations unchanged.
satisfies: derived

REQ-8amu2a's invite identifier and REQ-kt877x follow the owner's ruling
(2026-10-06) on pre-emption, handed over by the person-types switch (master
`5f7c177`): without them any peer could relay someone else's genuine admission
to a node that is waiting to be admitted, which would commit a record it is not
in, use up its expectation, and then refuse its own admission as a base
mismatch. The invite identifier is the one the app's Invite carries
(`app/docs/requirements/2026-10-06-invitation.md`).

*Note 2026-10-06 (merge of master `5f7c177`):* REQ-f2k4tr's "not greater than
the highest committed" and master's epoch rule REQ-txvtm9 agree: a committed
Sequence number equals the epoch committed with it, so the chain-free check
refuses what cannot be newer than the last commit, and the epoch rule, checked
after the one chain read, refuses the rest.

REQ-xs4ab8 and REQ-tqap3r exist because the owner moved the chain write out of
org-node: the administrators update the chain outside it, so what org-node
builds is provisional until the chain agrees. The sender's own node is held to
the receivers' checks, so no node's record runs ahead of the chain.

REQ-f2k4tr and REQ-8amu2a exist because removing the Envelope signature
removed the cheap refusal of a stranger's Envelope: without them any reachable
peer forces a chain read per message (HAZ-bedm57,
`org-node/docs/risk/2026-10-06-chain-authority.md`).

*Amended 2026-10-06 (independent review round 2, finding-9).* This cited
HAZ-bedm57 "in this change's risk draft" without a path.

- Owner journey (risk analysis, 2026-10-05): Alice sends Bob an Invite over a
  third-party channel carrying the Organisation's name, its identifier, a guess
  of Bob's full name, Alice's devices' public keys and a unique invite id. Bob
  enters it by hand, so his node expects a first admission only for that
  Organisation; his reply carries the invite id, and Alice discards any reply
  whose invite id she did not issue. In this change the reply is a Blob; it
  moves onto iroh with the transport change.
- All iroh communication moves to the app, so users may use other channels;
  org-node serialises, deserialises and verifies the messages the app sends
  and receives. This is its own change, after this one.
- Between this change and the key-pair change, the Organisation secret in an
  admission is protected by nothing: the owner accepted that window
  (HAZ-ep6uzs's residual, not acceptable until the key-pair change).

Sequencing (owner rulings 2026-10-05): four changes, one at a time.

1. Chain authority (this change): no Envelope signature, no sender checks, no
   administrator fields, provisional operations, the chain write moved to
   on-chain-client, the invitation exchange moved to the app, an empty record
   on removal.
2. Transport: iroh and all peer communication move to the app; org-node keeps
   serialisation, deserialisation and verification.
3. The Organisation key pair: X25519 at genesis, `person`'s rule, the receipt
   check, keep-or-clear, admission sends the key. Resolves PR-szkat6 and
   PR-xwek5e.
4. Whatever the first three leave, including the advisory signatory read.

The rulings above on the key pair are recorded here so change 3 starts from
them; this change implements only those that belong to change 1.

Open:

- (change 3) The self-delete path's update branch and the keep-or-clear rule.
- (change 3) SOUP for the X25519 arithmetic: `x25519-dalek` 2.0.1 or
  `curve25519-dalek` 4.1.3.

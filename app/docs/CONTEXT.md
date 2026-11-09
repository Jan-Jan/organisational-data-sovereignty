# Context

app is the desktop shell through which a person acts on an Organisation and
learns what has become of their membership: it carries an administrator's
intent to admit or revoke a Member into calls on org-node, and it renders back
what org-node reports about verification, epochs and revocation. It computes
almost nothing of its own. It exists because every intent and every belief in
this system passes through one surface, and that surface is the last place a
human can catch what the software got wrong.

## Language

Glossary of app's internal vocabulary. Definitions only — no implementation
details, no specs. Terms that cross a unit boundary (Organisation, Organisation
state, Organisation public key, Membership root, Change set, Finalised block,
Member, DevicePublicKey) live in the root `docs/CONTEXT.md` and are not
repeated here. org-node's terms that this unit's text uses (Persona, Invite,
Envelope, Sequence number) are defined in `org-node/docs/CONTEXT.md`, and Epoch in
`on-chain-client/docs/CONTEXT.md`.

*Corrected 2026-10-05 by review round 3 (finding-14), change
`worktree-person-shared-types`.* The list named Device key, which the root
glossary no longer defines (it says DevicePublicKey), and Persona and Epoch,
which it never defined.

**Device key**:
The name text written before 2026-10-05 uses for a DevicePublicKey (root
`docs/CONTEXT.md`). Text written from 2026-10-05 on says DevicePublicKey.
_Avoid_: using it in new text; DevicePublicKey is the term

**Operator**:
The person using this application, in whichever of the two roles the moment
calls for: an Admin (root `docs/CONTEXT.md`) admitting or revoking, or a
Member watching their own membership. One word for both, deliberately — the
application checks neither role itself. Where text names the role, it says
Admin, as the panels an Admin uses do ("Issue Invite (Admin)"); whether a
user is one is a fact on the chain, which a write reports.
*Amended 2026-10-28 (change `worktree-org-io-domain-api`, ARCHITECTURE review round 3, AR3-7):* "admin" left _Avoid_: the
root glossary's Admin is the term for the role, and the changed panels use it.
_Avoid_: user, administrator, end user

**Command**:
One named operation the frontend may ask the backend to perform, registered in
the Tauri invoke handler and reachable from the frontend by that name and no
other. Eighteen exist (the app's design draft lists them, LLR-ce4hxz). A
command is the only way the frontend can cause anything to happen.
*Amended 2026-10-28 (change `worktree-org-io-domain-api`, ARCHITECTURE review round 3, AR3-7):* "Twelve" became seventeen with
the domain surface.
*Amended 2026-11-09 (change `worktree-org-io-publish-2`, merge review round 1, MR1-A2):* "Seventeen" became eighteen
with `list_removed_devices`, and the list's item is LLR-ce4hxz, which supersedes LLR-kk8gc7.
_Avoid_: handler, endpoint, IPC call, action

**Event**:
One named message the backend emits to the frontend without being asked,
carrying a payload. Events are how everything that originates outside an
operator's click — an inbound update, a revocation, a receiver failure — reaches
the screen. Unlike a Command, an Event has no reply and its delivery is not
confirmed to the sender.
_Avoid_: notification, signal, message, emit

**Receiver loop**:
The single background task that awaits inbound messages for this node, one at a
time, and turns each outcome into an Event. There is at most one per process,
and while it is running no second one may start.
_Avoid_: listener, watcher, subscriber, background task

**Start guard**:
The flag that decides whether a Receiver loop may be spawned. It answers "may
one start", and — this is the distinction the register turns on — it does not
answer "is one running", although until this change the interface read it as
though it did.
_Avoid_: receiver flag, running flag, started

**Transport mode**:
Which of the two ways this installation reaches other nodes: Networked, where a
peer is found by discovery from its DevicePublicKey, or Loopback, where a peer
is dialled at a Peer address org-io holds for it. It is fixed at startup and
reported to the frontend rather than assumed there.
*Amended 2026-10-28 (change `worktree-org-io-domain-api`, ARCHITECTURE review round 3, AR3-7):* it decided whether the
operator had to supply a peer address; none is supplied any more (see Peer
address).
_Avoid_: network mode, iroh mode, connectivity

**Peer address**:
The dialling information for one node, as distinct from its DevicePublicKey.
The operator never supplies one: an Invite carries the issuing Device's, an
Admission request and an Organisation request carry the sender's, and org-io
keeps each for that Device's DevicePublicKey and dials only those.
*Amended 2026-10-28 (change `worktree-org-io-domain-api`, ARCHITECTURE review round 3, AR3-7):* it was "required in Loopback,
absent by design in Networked, and optional in every request that takes
one"; no command takes one since the change retired the pasted address
(REQ-vgr7s2 retired 2026-10-20).
_Avoid_: node addr, endpoint addr, peer blob

**Blob**:
An opaque encoded string an operator moves between two installations by hand:
an Invite. A Blob carries no proof of who produced it; what
it decodes to is what its author chose.
_Avoid_: payload, token, code, invite string

**Invite**:
What an Admin's Device sends a prospective Member over a channel of their
choosing: the Organisation's name as the inviter typed it, the Organisation
identifier, the inviter's guess of the invitee's full name, the issuing
Device's DevicePublicKey and Peer addresses, and an invite identifier the
issuing Device keeps outstanding, paired with that Organisation, until an
adopted epoch admits the Member whose Admission request (root
`docs/CONTEXT.md`) answered it, or the Invite is cancelled. The name is for
display and is verified by nothing.
*Amended 2026-10-28 (change `worktree-org-io-domain-api`, ARCHITECTURE review round 3, AR3-7):* it named the inviter's
DevicePublicKeys and stayed outstanding "until a reply naming both has been
acted on"; it now names the one issuing Device and is answered by an
Admission request.
_Avoid_: invitation code, link, ticket

*Invite reply* was defined here until 2026-10-28, when the change
`worktree-org-io-domain-api` replaced it with the root glossary's Admission
request, sent over iroh rather than moved by hand (round 3 review, AR3-7).
The root glossary lists "Invite reply" under that term's _Avoid_.

**Connection status**:
The report the backend returns describing how this installation is configured:
whether chain operations are available, the endpoint and contract the running
configuration was built from, the Transport mode, and the data directory. It
describes configuration, not liveness.
_Avoid_: health, chain status, connectivity report

**Development defaults**:
The startup values the application will substitute only when explicitly
permitted to: the built-in store passphrase and the fallback data directory.
Named as one concept because they are permitted by one act, and because the
thing that makes them safe is that permitting them is deliberate.
_Avoid_: dev mode, debug defaults, fallbacks

**Verification log**:
The table in which each inbound update appears with its Organisation, Epoch,
Membership root and its outcome. Its outcome column is a report of what
org-node concluded, never a conclusion this unit reaches.
_Avoid_: update list, verify table, history

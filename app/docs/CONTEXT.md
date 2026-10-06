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
calls for: an administrator admitting or revoking, or a member watching their
own membership. One word for both, deliberately — the application does not
distinguish them, and a glossary that did would imply a separation the software
does not provide.
_Avoid_: user, admin, end user

**Command**:
One named operation the frontend may ask the backend to perform, registered in
the Tauri invoke handler and reachable from the frontend by that name and no
other. Twelve exist. A command is the only way the frontend can cause anything
to happen.
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
peer is found by discovery from its DevicePublicKey, or Loopback, where a peer must
be dialled at an address supplied alongside the request. It is fixed at startup
and it decides whether a peer address is required — which is why it has to be
reported to the frontend rather than assumed there.
_Avoid_: network mode, iroh mode, connectivity

**Peer address**:
The dialling information for one node, as distinct from its DevicePublicKey. Required
in Loopback, absent by design in Networked, and optional in every request that
takes one.
_Avoid_: node addr, endpoint addr, peer blob

**Blob**:
An opaque encoded string an operator moves between two installations by hand —
an Invite, or an Invite reply. A Blob carries no proof of who produced it; what
it decodes to is what its author chose.
_Avoid_: payload, token, code, invite string

**Invite**:
What a Member sends a prospective Member over a channel of their choosing: the
Organisation's name as the inviter typed it, the Organisation identifier, the
inviter's guess of the invitee's full name, the inviter's DevicePublicKeys and an
invite identifier the inviter keeps, paired with that Organisation, until a
reply naming both has been acted on; a refused reply, or one whose chain write
fails, leaves it outstanding. The name is for display and is verified by
nothing.
_Avoid_: invitation code, link, ticket

**Invite reply**:
What the invitee sends back so that the inviter can admit them: the
Organisation identifier and invite identifier from the Invite, and the chosen
Persona's Member-as-a-group key,
DevicePublicKey, handle, name and surname.
_Avoid_: join request, application, enrolment request

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

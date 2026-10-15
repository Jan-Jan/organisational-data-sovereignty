# Loopback delivers to an address the recipient did not prove

*Moved 2026-10-13 from org-node's problems ledger
(`org-node/docs/problems/2026-09-09-org-node-problems.md`, which keeps a
prose pointer here), plan task T12 of
`docs/plans/2026-10-06-org-io-transport.md` (ruling A): the send moved to
org-io, and the fix landed here. Its `affects:` line named, besides the
Loopback dial item that moved with it, org-node's control on the sender of an
admission; that control is named in prose below.*

Opened by org-node's assessment of 2026-09-09.

**PR-2dmjzj**: in loopback transport mode `admit_member` sends the admission
Envelope and the Organisation secret to the `EndpointAddr` the caller passes
from the Join request, without checking that the address's identity is the
Device key the Join request carries, so an altered Join request hands the
secret to whoever the altered address names while the record admits the
joiner's genuine keys.
affects: LLR-jn5jeh, REQ-8trkps
opened: 2026-09-09
status: resolved
resolution: fixed 2026-10-13 (change `worktree-org-io-transport`, plan task
T12): in Loopback mode org-io's `send_committed` sends to the one address it
is given only when that address's endpoint identity is the Device key of a
recipient the commit names, and then to that recipient only; an address of
no recipient is refused with `SendError::AddressNotARecipient`, nothing sent
to anyone, no endpoint bound (LLR-r9sh6k, REQ-8trkps). The reproduction:
`loopback_to_an_address_of_no_recipient_is_refused_and_sends_nothing`
(`org-io/tests/send_fanout.rs`), an admission sent at the address of a
listening endpoint that is not the joiner's: red before (no address check
existed; compile-red at the new API), green after, the address's endpoint
receiving nothing.

*Noted 2026-10-13 (S4, task T12):* the defect outlived the Join request and
the Organisation secret it names. Since 2026-10-06 the address comes from
the operator beside an Invite reply, and every Organisation-information
message carries the Organisation private key; the send moved from
`admit_member` to org-node's `send_update` and then to org-io. The defect
was the same throughout: the Loopback dial took the address on trust.

## As found (2026-09-09)

Where: `org-node/src/service.rs`, the transport send in `admit_member`: the
Loopback branch dialled `peer_addr` as given, against the Networked branch,
which derives the peer identity from the joiner's Device key and so binds
delivery to the key. The Join request was an unsigned blob by design,
travelling out of band by copy and paste, so nothing upstream detected the
alteration.

Observable symptom: with a Join request carrying B's keys and C's address,
`admit_member` succeeded, the chain's root admitted B, and C's endpoint
received the message carrying the Organisation's key.

Why it is a hazard: org-node's hazard of the Organisation's key reaching a
device the record does not name (S3/P2 in org-node's register; S3/P1 for the
altered admission address). In org-io it bears on HAZ-ztkex9, the misdirected
Loopback send, which REQ-8trkps (RC-d6f25u) controls.

The fix, in outline then: refuse a `peer_addr` whose `EndpointId` differs
from the Device key, or dial by the key in both modes. The first is the one
taken.

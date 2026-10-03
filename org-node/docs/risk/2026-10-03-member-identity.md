# Risk — random member identifiers, assessed

## Derived requirements

assesses: REQ-d9g6nt

**What it changes.** Before it, a member's `MemberId` was the bytes of its
Member-as-a-group key. Every grant an Organisation makes is to a `MemberId`, never to a handle,
so an identifier tied to a key tied grants to that key: a revoked person who
rejoined with the same keypair came back under the deleted identifier, and
whatever had been granted to it — by any party that had not yet processed the
revocation, or that keys grants by identifier — attached to the new membership.
Drawing the identifier at random removes that route by construction. It adds no
hazard: the identifier is chosen by the administrator's node, which already
chooses every other field of the leaf it signs, and it is carried to every
other device inside the Change set and the Membership record snapshot exactly
as before.
A collision between two random 32-byte identifiers is not a credible event, and
org-members refuses one anyway (`DuplicateId`).

**What it does not change, stated so the requirement is not read wider.**
By owner ruling (2026-10-03), a re-admitted person is a new member under a
fresh `MemberId` and may bring the keys their previous membership held when it
was deleted — the one exception; nothing granted to the old id carries over.
The software keeps no history of revoked keys and does not refuse them, so a
removed device that still holds those keys' secrets gets the new membership's
access. If a removed device was compromised, fresh keys are the joiner's
choice, not a software check. This is residual risk on HAZ-vxabf9 (a device
removed from the Membership record that keeps acting as a member), accepted by
the owner's ruling and not re-scored here. The duty not to supply any other key
no longer held stands (a Member-as-a-group key used earlier and since replaced;
the Device key of a device removed earlier while the member stayed), and a
deleted member's keys must never be given to anyone else. Neither duty is
enforced by org-members or by org-node, which passes the joiner's keys from the
Join request unchanged; the administrator who admits or rotates carries them.

**The refused first admission.** A first admission with no Membership record
snapshot was attempted against a reconstructed single-administrator Membership
record that could never match the chain's root, so it never succeeded; it is now refused explicitly,
with an error before any verification. No
sender in the software omits the snapshot, so no working exchange is lost; a
peer that did would previously have failed the base-root check, closed, and
now fails earlier, still closed.

**Existing Membership records.** Identifiers already stored are the key bytes
they were made from. Nothing re-derives an identifier from a key any more, so
they remain valid opaque identifiers; no migration is needed and old and new
identifiers coexist in one Membership record.

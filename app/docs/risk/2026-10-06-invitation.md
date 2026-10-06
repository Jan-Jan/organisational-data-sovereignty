# Risk analysis — the invitation exchange and submitting updates (app)

Run 2026-10-05 for change `worktree-org-node-chain-authority`, under IEC 62304
Class C and the acceptability matrix in this ledger's README. The owner moved
the invitation exchange out of org-node into this unit and made this unit
submit each update org-node builds
(`app/docs/requirements/2026-10-06-invitation.md`).

## A new hazard: an Invite from someone who is not a Member

**HAZ-qfb95k**: an Invite's sender, its DevicePublicKeys and the Organisation name
it states are verified by nothing, and the invitee has no record to check them
against; someone who is not a Member of the Organisation sends an Invite
naming that Organisation's real identifier and a familiar name, and the
invitee replies; the handle, name and surname the invitee entered for that
Persona reach a party outside the Organisation. Severity: S2. Probability: P2.

P2: impersonation over the third-party channel an Invite travels on is
ordinary, and nothing in an Invite distinguishes a Member from anyone else.

S2, by owner ruling (2026-10-05): the keys in the reply expose nothing, because
a Persona's keys are generated for that one Organisation and link to no other
identity; what reaches the impostor is the name details the invitee chose to
enter, which may be a pseudonym. The first draft of this item rated it S3 on
the keys and the name together.

**RC-wzb48r**: before producing an Invite reply, the app tells the user that
nothing has verified who sent the Invite or the Organisation name it states,
and that the reply reveals the handle, name and surname entered for the chosen
Persona to that sender, and produces the reply only once the user confirms.
mitigates: HAZ-qfb95k

This is information for safety, the lowest rank in ISO 14971's order, and the
owner chose it knowingly (2026-10-05). A design control would need to show the
invitee that the Invite's sender is a Member or a signatory of the
Organisation's multisig, which nothing in this change can establish; the
multisig signatory read is planned for a later change of the sequence.

What this control breaks: nothing a user relies on; it adds one confirmation
to joining.

Residual: Severity S2, probability P2, **accepted by owner ruling
(2026-10-05)** on the grounds that the invitee chose the channel, chose the
name details, and is told before replying what the reply reveals. The matrix
places S2/P2 in the unacceptable region; this is a deliberate acceptance,
recorded as such.

## Derived requirements assessment

**REQ-prjja8** (produce an Invite with a random invite identifier) and
**REQ-65xqp8** (refuse a reply naming no outstanding invite identifier): no new
hazard. The identifier's 32 random bytes make a reply that names one a reply
to that Invite; refusing the rest keeps an unsolicited reply from reaching an
admission.

*Amended 2026-10-06 (independent review round 2, finding-1).* The paragraph
above held only while an identifier was tied to its Organisation, and it was
not: an identifier was outstanding for every Organisation, and the admit
panel took the target Organisation from the reply, so a reply echoing one
Invite's identifier could name any other Organisation this device holds and
reach an admission there — a person the operator invited to one Organisation
admitted to another, which the operator neither intended nor reviewed. REQ-65xqp8 now refuses a
reply unless its identifier is outstanding for the Organisation it names, and
LLR-gha5f6 admits only to that Organisation, whatever the panel selects; with
that, the assessment above stands again, and no new hazard is recorded.

**REQ-tcutr6** (produce an Invite reply, and declare the expected first
admission to org-node): the reply is the disclosure HAZ-qfb95k analyses. The
declaration realises org-node's control on first admissions; a stale
declaration only lets a chain-valid admission for that Organisation be read.

**REQ-ab2mfz** realises RC-wzb48r; assessed above.

**REQ-yazum3** (refuse an Invite or reply that does not parse, naming the
field): no hazard impact; it is the parse-at-the-system-edge rule at this
unit's new edge, and refusing acts on nothing.

**REQ-nfr3n2** (submit through on-chain-client, then ask org-node to commit and
send): no new hazard. The multisig signatory key the app held for org-node it
now hands to on-chain-client instead; its custody is unchanged and remains as
HAZ-8ghmhn analyses. A submission that fails leaves org-node's record and
every peer's untouched; one that succeeds and is then not sent is org-node's
publish-before-persist hazard, which org-node's requirement that the sender
commit after verifying against the chain narrows.

assesses: REQ-prjja8, REQ-65xqp8, REQ-tcutr6, REQ-ab2mfz, REQ-yazum3, REQ-nfr3n2

## Derived low-level requirements amended by the design (2026-10-06)

The design step wrote eight low-level requirements for the requirements above
(`app/docs/architecture/2026-10-06-invitation.md`);
each satisfies one of them, and none is derived. LLR-be3zv9 bounds each chain
write at 90 seconds: a write that is still pending when the bound elapses is
reported as failed and nothing is committed or sent, which is REQ-nfr3n2's
failure case. A write that executes on the chain after the app gave up leaves
the chain ahead of the node. org-node's `commit_update` could reach it from
the provisional update the failure kept, but no app path retries that commit
today: the only caller is the next submission's `submit_commit_send`, which
can then commit the earlier, timed-out update in place of its own and send it
to the new recipient — PR-924ftr
(`app/docs/problems/2026-10-06-provisional-housekeeping.md`) records this.
Either way nothing is committed that the chain does not carry.
Two derived requirements of the 2026-10-05 decomposition are amended in place.

*Amended 2026-10-06 (independent review round 2, finding-8).* This paragraph
said the kept update is reached by `commit_update`, as though the app did so;
it is qualified to what the app does today, with PR-924ftr cross-referenced.

**LLR-ctrfz4** (`admit_member` reads the joiner's peer address from its own
argument, blank meaning absent): no new hazard. It replaces reading the
address from a join request; where the admission is sent is still the
operator's input, and in Networked transport the address is not used.

**LLR-pmus9f** (`import_invite_reply` shows a parsed reply and stores
nothing, and refuses one naming no outstanding invite): no new hazard. It
replaces `import_join_request`; refusing a reply that names no outstanding
invite is REQ-65xqp8's control against an unsolicited reply, assessed above.

assesses: LLR-ctrfz4, LLR-pmus9f

## Derived low-level requirement of review round 1 (2026-10-06)

**LLR-rt8gdz** (founding and replying offer only Personas bound to no
Organisation, and the backend refuses a bound one): no new hazard; it removes
one. Before it, a Persona bound to one Organisation could found a second or
reply to an Invite to one, and a first admission or commit could then rebind
it, so the device's record of which Organisation a Persona belongs to — what
every later send and verification of that Persona reads — could change
silently. Refusing before the chain is written and before an expectation is
declared acts on nothing; what it breaks is joining a second Organisation with
an existing Persona, for which the user creates another Persona.

assesses: LLR-rt8gdz

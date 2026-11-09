# Risk analysis — the invitation exchange and submitting updates (app)

Run 2026-10-05 for change `worktree-org-node-chain-authority`, under IEC 62304
Class C and the acceptability matrix in this ledger's README. The owner moved
the invitation exchange out of org-node into this unit and made this unit
submit each update org-node builds.

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

Its control is RC-dfzb5q, the warning before the first admission request
(`2026-10-15-domain-api.md`).

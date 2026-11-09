# app — the demo journey's domain surface: items and low-level requirements

Change `worktree-org-io-domain-api`, written 2026-10-15 after S4
(`worktree-org-io-transport`) merged into the branch. It refines the app
requirements of `app/docs/requirements/2026-10-14-domain-api.md`
(REQ-akeah6, REQ-88npkt, REQ-s3vnmh, REQ-pe2c7u, REQ-qv4d2z, REQ-2yh3c8,
REQ-vb5pag, REQ-k7ay5b, and since 2026-10-20 REQ-5q7nyg, REQ-vgn96n,
REQ-ergn9z and REQ-y53dqn) and keeps the unit's earlier requirements that are
still in force (REQ-sjkp8z, REQ-he8ejb, REQ-tw4cb5, REQ-2k7ys4,
REQ-dp95pv). The decisions it carries out are the two accepted records of
this change in `docs/adr/` — `*-batched-epochs.md` (provisional changes
batched into one proposed epoch) and `*-write-verify-send.md` (writing,
verifying and sending are three steps the app calls separately; only
verification adopts); their IDs are not written here because the checker
reads `docs/` as disclaimed and would report them dangling. Safety class C, no per-item override, as in
`2026-10-05-decomposition.md`, whose conventions this file keeps: a low-level
requirement states one behaviour a gated test reddens on; tests are named by
the plan of this change and carry `verifies:` with the item they test.

**The rule this change applies (owner, 2026-10-14):** the app decides when;
org-io does the IO and the judging. The app calls org-io's domain surface and
names no org-node item: `propose_person`, `propose_founding_organisation`,
`issue_invite`, `read_invite`, `propose_member`, `request_admission`,
`propose_admission`, `propose_revocation`, `cancel_invite`,
`propose_updated_organisation`, `write_organisation_to_chain`,
`verify_organisation`, `send_organisation` and `request_organisation`, plus
`start_receiving` and `next_received` (and, since 2026-10-20, the list of
kept admission requests, owner answer A3). org-io holds its own CSPRNG, so no
app command passes randomness (the app's `rand` leaves the SOUP, see
`soup.md`). Every IPC command of the surface carries the org-io method's
name.

**Replaced here:** the earlier command surface, reconcile view and
revocation input decision, whose responsibility text named the old commands,
the reconcile and the reply, and their low-level requirements for the old
commands, the reconcile, the Invite reply and the Persona offer of a reply.
Since 2026-10-28 this change's own verification view and pre-command
decisions are split (round 3 review AR3-5), into SDD-swt9fs and SDD-ua2d9b,
and into SDD-czjf2c and SDD-7aq69w.

**The peer address.** The Invite carries the issuing Device's endpoint
addresses (owner ruling R4), so the invitee needs no pasted address to reach
the inviter, and the admission takes none. An admission request and an
organisation request carry the sender's own endpoint addresses, which org-io
keeps for that Device's key (owner answer A2 of 2026-10-16), so org-io
reaches every Device it sends to at addresses it holds, and the app takes,
parses and passes no peer address anywhere: `send_organisation` and
`request_organisation` are asked with the Organisation alone. The send is
made once per own adoption and never repeated by the app (Q1), and an
Organisation reported behind is requested with no user step (Q2), so
neither `send_organisation` nor `request_organisation` is an IPC command:
both are asked only by `verify_and_send`. The surface gains
`list_admission_requests` (owner answer A3) and the Rust half gains a
periodic verification loop (SDD-tgu7dp). A revocation is accepted with no
peer address in every transport mode. The unit's requirement for a peer
address in Loopback was retired on 2026-10-20 (owner decision, answer A2);
the earlier requirements this ledger keeps in force are REQ-sjkp8z,
REQ-he8ejb, REQ-tw4cb5, REQ-2k7ys4 and REQ-dp95pv.

## SDD-6pjvhc — The command surface

`app/src-tauri/src/commands.rs`: the eighteen command handlers (eighteen
until 2026-10-20, seventeen from then until 2026-11-03), their DTOs (the verification entry's in
`app/src-tauri/src/entry.rs` since 2026-10-28),
`verify_and_send` (the body `verify_organisation` and the receiver loop
share) and the handler list the IPC suite registers;
`app/src-tauri/src/lib.rs` registers its own copy of that list in `run`.
What of them needs a chain to succeed is SDD-6g3wnh's, as before; the
low-level requirements below are the parts the IPC harness and org-io's
substitute chain (`AppState::for_test_over`) reach.

**SDD-6pjvhc**: the IPC command surface, the one way the webview reaches
org-io. Its eighteen commands are the fourteen that call org-io's domain
surface under its names, then `list_personas`, `list_orgs`,
`connection_status`, `start_receiver`. Each calls the shared org-io handle
with values parsed at the IPC edge, passes no randomness, maps org-io's
outcome to a DTO that carries no secret key material, returning org-io's
refusal as its text; a follow-up SDD-6zqqm8 decides after a verification or
a write is one more call made here.
traces: REQ-vb5pag, REQ-akeah6, REQ-qv4d2z, REQ-pe2c7u, REQ-88npkt, REQ-2yh3c8, REQ-s3vnmh, REQ-sjkp8z, REQ-645jq9, REQ-vgn96n, REQ-y53dqn, REQ-yvzef9

*Amended 2026-11-03 (change `worktree-org-io-publish-2`, task T13b; plan §0,
T16g; an extension of the surface, no existing command changes meaning):*
eighteen commands: `list_removed_devices` joins, calling org-io's
`OrgIo::removed_devices` (LLR-dmfvv6, LLR-ce4hxz in
`2026-11-02-publish-2.md`), and `traces:` gains REQ-yvzef9. The webview's
view of it is SDD-g9ugu7's.

*Amended in place 2026-11-09 (change `worktree-org-io-publish-2`, merge
review round 1, MR1-A2; meaning unchanged):* the item and the section
preamble above still read "seventeen commands are the thirteen that call
org-io's domain surface" and "the seventeen command handlers (eighteen until
2026-10-20)" after `list_removed_devices` joined on 2026-11-03; they now
give eighteen and fourteen, as the 2026-11-03 note above states.

*Amended in place 2026-10-28 (unmerged draft; round 3 review AR3-5, sweep
for ARCHITECTURE Boundaries 3):* reworded so its responsibility is stated in
one sentence without "and"; the responsibility, its interfaces and its LLRs
are unchanged.

*Amended in place 2026-10-28 (unmerged draft; round 2 review,
ARCHITECTURE findings 1 and 2):* the item held two responsibilities, the IPC
surface and the follow-up policy ("verification follows an included write,
an adopted epoch this node built is sent once, at once, and an Organisation
reported behind is requested from its Members at once"), and the policy was
written inside the adapter code making the networked calls. The policy
leaves for SDD-6zqqm8, a pure function in `follow_up.rs`; this item makes
the calls it decides. LLR-es49wp, LLR-nkg8sb and LLR-k4rhbs, stated at
`verify_and_send` and `write_organisation_to_chain`, stay here: they state
what this item's interfaces do, now as SDD-6zqqm8 decides. The verification
entry's DTOs, `entry_for` and `device_reports` move from `commands.rs` to
`app/src-tauri/src/entry.rs` (re-exported by `commands`), so the receiver
item reads the entry by its type rather than its JSON (finding 3); they are
still this item's, under LLR-jf63e4, LLR-73by2q and LLR-cugcu6.

*Amended in place 2026-10-20 (unmerged draft; consistency pass):*
`traces:` drops the peer-address requirement, retired 2026-10-20 (owner
answer A2): no
command takes a peer address.
*Amended in place 2026-10-20 (unmerged draft; owner rulings Q1, Q2 and
answers A2, A3 of 2026-10-16):* seventeen commands, not eighteen —
`send_organisation` and `request_organisation` leave the IPC surface (the
send is made once by `verify_and_send` and offers no retry; the request is
made by it with no user step), and `list_admission_requests` joins.
`verify_and_send` now also requests an Organisation reported behind
(LLR-nkg8sb), so every verification — at start, on request, after an
included write, on a received provisional organisation and each interval
(SDD-tgu7dp) — sends and requests alike. `traces:` gains REQ-vgn96n and
REQ-y53dqn.

The low-level requirements the earlier command surface carried that still hold — LLR-5wb4vd
(the shared handle, no app lock), LLR-yrb6th (the receiver loop's terminal
stop) and LLR-pguhw5 (the Persona fields) — are unchanged and now sit under
this item. The command-boundary parses stay SDD-2pa6h6's; the ones this
change adds are listed under that item below.

**LLR-k57uqv**: `propose_person` takes `name` and `surname` and no Handle,
calls org-io's `propose_person` once, and returns the Person's
`device_public_key` as 64 hexadecimal characters, its `name` and `surname`
as org-io stored them, and `status` `provisional`; a refusal names the field
it is about (`name:` or `surname:`) and creates nothing (no Person keychain
item, no Person file).
satisfies: REQ-vb5pag

*Amended in place 2026-10-28 (unmerged draft; round 2 review, ARCHITECTURE
finding-4):* a refusal crosses IPC as an object, `kind` and `message`:
`kind` `person_already_held` for org-io's refusal of a second Person (one
Person per device) and `refused` for every other, `message` org-io's text
with the refused field named first. The creation form passes over a held
Person by its `kind`; it branched on the text's first words, so a rewording
in org-io would have broken the founding with no type error.

**LLR-6ff735**: `propose_founding_organisation` takes `name`, `surname`,
`handle` and `org_name`, calls org-io's `propose_founding_organisation`
once, and returns the new Persona's `persona_id`, `status`
`provisional_founding` and `founder`, the founder's Device key as org-io's
`ProposedFounding` names it, as 64 hexadecimal characters; a refusal names
the field it is about and leaves no new Persona: `list_personas` returns
what it returned before the call.
satisfies: REQ-vb5pag

*Amended in place 2026-10-22 (unmerged draft; dispatcher decision, plan
assembly):* the command returned no Device key, and `list_personas` carries
none, so the webview had no `founder` to pass to
`write_organisation_to_chain` (LLR-rhc555) for the founding it had just
proposed; the result now carries org-io's `founder`.

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* org-io's
`ProposedFounding` names the new Persona by its Device key (`founder`), not
by a Persona id; the command returns the `persona_id` of the Persona
org-io's `personas()` lists with that Device key.

**LLR-y3zykm**: `issue_invite` takes `org_id` and `invitee_name`, calls
org-io's `issue_invite` once, and returns the Invite Blob org-io built as
text, unchanged; it takes no Organisation name (org-io's Invite carries the
name stored for the Organisation) and no address (org-io supplies the
issuing Device's).
satisfies: REQ-vb5pag

**LLR-b7ywy4**: `read_invite` takes a Blob, calls org-io's `read_invite`,
stores nothing, and returns the Organisation name, the Organisation
identifier as 40 hexadecimal characters, the invitee name, and the issuing
Device's DevicePublicKey as 64 hexadecimal characters; a Blob that does not
parse is refused with org-io's message, which names the field.
satisfies: derived

**LLR-rg9vc7**: `propose_member` takes an Invite Blob, `name`, `surname` and
`preferred_handles`, a list, and hands the list to org-io's `propose_member`
in the order given, unchanged; it returns the new Persona's `persona_id` and
`status` `provisional_member`; a refusal (a list of no Handle, of more than
three, or with a Handle twice, or an Invite that does not parse) is org-io's
message and leaves no new Persona. It sends nothing.
satisfies: REQ-vb5pag

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* org-io's
`ProposedMember` names the new Persona by its Device key (`member_device`);
the command returns the `persona_id` of the Persona org-io's `personas()`
lists with that key.

**LLR-q2k9z8**: `request_admission` takes `persona_id` and `confirmed`; with
`confirmed` false it refuses with `the user has not confirmed the admission
request` and asks org-io to send nothing; with `confirmed` true it calls
org-io's `request_admission` once for that Persona.
satisfies: REQ-2yh3c8

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* org-io's
`request_admission` takes the Organisation identifier, not a Persona: the
command passes the `org_id` org-io's `personas()` lists for that Persona (a
provisional Member's is the Invite's Organisation), and refuses, asking
org-io for nothing, a `persona_id` org-io lists no such Persona for.

This is the backend's half of the warning's gate, as the IPC refusal of an
unconfirmed reply was before: the webview decides what the user is asked
(LLR-qcg6bz, LLR-prh433); this refusal holds when the webview is bypassed.

**LLR-8cz828**: `request_admission` returns org-io's outcome tagged by
`result`: `delivered`, `already_received`, `invite_used_by_another_device`,
`refused`, or `not_delivered` with `reason`, each reason
org-io's text unchanged; an invite another Device used is never returned as
`refused` or `not_delivered`.
satisfies: REQ-s3vnmh

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* `refused`
carries no `reason`: org-io's `AdmissionOutcome::Refused` is the receiving
Device's one-byte verdict, which carries none.

**LLR-cr9nmw**: `propose_admission` takes `org_id`, `invite_id`, `handle`
and `confirmed`; with `confirmed` false it refuses with `the Admin has not
confirmed the admission` and records nothing; with `confirmed` true it calls
org-io's `propose_admission` once.
satisfies: REQ-88npkt

**LLR-gmb4p8**: `propose_admission`'s arguments are those four alone: it
takes and parses no Organisation secret, Organisation key, Invite reply or
peer address. A `handle` that is absent, empty or only whitespace is handed
on as none (org-io chooses the first preferred Handle free); any other is
handed on unchanged as the Admin's choice, and org-io's refusal when none is
free and none was chosen is returned as its text. The webview's
`proposeAdmission(orgId, inviteId, handle, confirmed)` invokes it with those
four values alone.
satisfies: derived

**LLR-7n4hgr**: `propose_revocation` takes `org_id` and `member_id`, strips
at most one leading `0x` from the member identifier, refuses a remainder that
is not hexadecimal with a message beginning `member_id hex` and one that
does not decode to exactly thirty-two bytes with `member_id must be 32 bytes
(64 hex chars)`, and otherwise calls org-io's `propose_revocation` once; it
takes no peer address and sends nothing.
satisfies: derived

**LLR-vqz4s6**: `cancel_invite` takes `org_id` and `invite_id`, calls
org-io's `cancel_invite` once, and returns nothing on success; a refusal is
org-io's text.
satisfies: derived

**LLR-2u3cga**: `propose_updated_organisation` takes `org_id`, calls
org-io's `propose_updated_organisation` once, writes nothing to the chain,
and returns the proposed `epoch`, its Membership `root` and its
`org_public_key` as org-io's `ProposedOrganisation` reports them (each key
and root as 64 hexadecimal characters); org-io's refusal (no provisional
change, or the Organisation reported behind) is returned as its text.
satisfies: REQ-vb5pag

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* it
returned "the number of provisional changes it applied", which org-io's
`ProposedOrganisation { org_id, epoch, root, org_public_key }` does not
report; it now returns what org-io reports.

**LLR-rhc555**: `write_organisation_to_chain` takes either `org_id` or, for
a provisional founding, `founder` (its Device key, 64 hexadecimal
characters), exactly one of them, calls org-io's
`write_organisation_to_chain` once with `HeldTarget::Organisation(org_id)`
or `HeldTarget::Founding(founder)`, and returns its outcome tagged by
`result`: `included` with `epoch`, `refused` with `reason`, or `unknown`
with `reason`, each reason org-io's text unchanged; it asks org-io to send
nothing. Both or neither given is refused before org-io is called. The
webview's `writeOrganisationToChain({ founder })` writes a founding with the
founder's Device key from one of two results it already has: the `founder`
`propose_founding_organisation` returned (LLR-6ff735), or the `founder` of a
founding's verification entry (LLR-jf63e4), so a restarted app finds every
held founding and its key in its first verification.
satisfies: REQ-vb5pag

*Amended in place 2026-10-30 (unmerged draft; merge review round 1,
finding-6):* which foundings the view offers to write is stated: each
founder from those two results once, those proposed first, then those of
the filed results newest first; a founder whose newest filed entry is
`adopted` or `in_step` is written and is no longer offered (`foundingWrites`).
A founder whose newest entry is any other result stays offered. An
Organisation update is written by `orgId` (LLR-tbtrb3).

*Amended in place 2026-10-22 (unmerged draft; dispatcher decision, plan
assembly):* no item said where the webview gets a founding's `founder`; it
takes it from those two results, and no command is added for it.

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* org-io's
write takes a `HeldTarget`, and a provisional founding has no Organisation
identifier until its proxy exists, so `org_id` alone could not write a
founding; the founding is named by its Device key, as org-io names it.

**LLR-k4rhbs**: when `write_organisation_to_chain`'s outcome is `included`,
the command then asks `verify_and_send` once for that Organisation and
returns its entry as `verification` beside the write's outcome; for
`refused` and `unknown` it asks for no verification and `verification` is
absent. The verification's target is the write's: for a write given `org_id`,
`HeldTarget::Organisation(org_id)`; for a founding's write given `founder`,
`HeldTarget::Founding(founder)`, the target org-io reads a founding by at
its stored proxy (org-io's founding verification). Once a verification has
adopted the founding, the Organisation is addressed by
`HeldTarget::Organisation(org_id)` from then on, by every command.
satisfies: derived

*Amended in place 2026-10-21 (unmerged draft; design gap G3 of the plan,
dispatcher's decision):* "for that Organisation" did not say which target a
founding is verified by after its write. org-io states the founding's read
for `HeldTarget::Founding` only, and its `HeldTarget::Organisation(org_id)`
names a held Organisation, not a founding store whose proxy is recorded and
which holds no record yet; so the app verifies a founding by its founder's
Device key, as it wrote it (LLR-rhc555).

**LLR-jf63e4**: `verify_organisation` takes an optional `org_id`; with one it
asks org-io to verify that Organisation or provisional founding, with none
every one org-io holds, and returns one entry per result in org-io's order,
tagged by `result` — `in_step` (`epoch`), `adopted` (`epoch`, `own`,
`discarded`), `removed`, `behind` (`chain_epoch`), `inconsistent`
(`reason`) or `unverified` (`reason`) — each with `org_id` as 40
hexadecimal characters, or, for a provisional founding whose proxy is not
yet recorded and which has no Organisation identifier, `org_id` null (JSON
`null`, never an empty string); every entry for a provisional founding
(org-io's `HeldTarget::Founding`) carries `founder`, its founder's Device
key as 64 hexadecimal characters, beside `org_id` null before its proxy is
recorded and beside the Organisation identifier the proxy gave after — and
each reason org-io's text unchanged. With
the chain not configured every held Organisation is `unverified`; with none
held the list is empty. org-io's `Verified::Store` and `Verified::Keychain`
(the store write or the keychain read a verification needed failed) are
returned as `unverified` with org-io's text as `reason`, never as another
result.
satisfies: REQ-akeah6

*Amended in place 2026-10-28 (unmerged draft; round 3 review AR3-2
follow-on):* org-io's refusal variant is now `Verified::Refused { error }`,
its field a typed `Cause<OrgNodeError>` (no longer `reason`); the note of
2026-10-22 below names it so. The entry is unchanged: `unverified` with
that cause's text as `reason`. No behaviour changes.

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* org-io's `verify_all` returns each result beside its
target (org-io's `(HeldTarget, Verified)`), and a provisional founding with
no proxy recorded has no `org_id`; such an entry carries `org_id` null and
its `founder`, never an empty string or a placeholder.

*Amended in place 2026-10-22 (unmerged draft; dispatcher decision, plan
assembly):* a founding entry carried `founder` only beside `org_id` null,
so once a founding's proxy was recorded nothing gave the webview the
founder's Device key its write needs (LLR-rhc555); every founding entry now
carries `founder`, so a restarted app finds every held founding and its
founder's key in its first verification (at mount, on request or periodic).

*Amended in place 2026-10-22 (unmerged draft; dispatcher decision, plan
assembly):* org-io's `Verified::Settled { outcome, write }` (a verification
that also settled an unknown chain write) is returned as the inner
`outcome`'s entry, tagged by that outcome's `result`, with one more field,
`write`: `adopted`, `not_executed`, `deprecated` or `still_unknown`, as
org-io's `WriteOutcome` says; no other entry carries `write`. org-io's
`Verified::Refused { error }` (an org-node refusal no other result names)
is returned as `unverified` with org-io's text as `reason`.

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* org-io's
`Verified` has two outcomes beyond the six this item listed, `Store` and
`Keychain`; each is a verification that changed nothing and reached no
result, which REQ-akeah6's results name "unverified with its reason".

*Amended in place 2026-10-21 (unmerged draft; design gap G3 of the plan,
dispatcher's decision):* "that Organisation or provisional founding" is
narrowed. With an `org_id` the command asks org-io to verify
`HeldTarget::Organisation(org_id)`, a held Organisation (a founding that
verification has adopted included); a provisional founding not yet adopted
is not reached by its `org_id`. It is verified by
`HeldTarget::Founding(founder)` after its included write (LLR-k4rhbs), and
among every target org-io holds when no `org_id` is given (org-io's
`verify_all`).

**LLR-es49wp**: `verify_and_send` asks org-io's `send_organisation` exactly
once for an Organisation whose result is `adopted` with `own` true, with no
peer address and no further input, and attaches org-io's per-Device report
to that entry as `sends`; for every other result, `adopted` with `own` false
included, it asks for no send and `sends` is absent. When org-io refuses the
send as a whole (no per-Device report), `sends` is empty and
`sends_refused` carries org-io's refusal text; otherwise `sends_refused` is
absent.
satisfies: REQ-qv4d2z

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* org-io's `send_organisation` can refuse as a whole
(the Organisation not held, or its endpoint not bound), which the
per-Device report cannot carry; `sends_refused` reports it.

**LLR-n64wvr**: an `adopted` entry's `discarded` lists every provisional
change org-io reports discarded, in org-io's order, each tagged by `kind` —
`admission` (`invite_id` as 64 hexadecimal characters, the request's `name`,
`surname` and the `handle` it was to receive) or `revocation` (`member_id`
as 64 hexadecimal characters, with the Member's `handle`) — and is empty, not
absent, when nothing was discarded. Every field an entry's kind lists is
always present: one that does not apply or that org-io does not know is
JSON `null`, never absent and never an empty string — an admission's `name`
and `surname` when no kept request gives them, a revocation's `handle` when
the adopted record no longer lists the Member.
satisfies: REQ-pe2c7u

*Amended in place 2026-10-22 (unmerged draft; dispatcher decision O2, plan
assembly):* org-io reports an admission's name and surname and a revoked
Member's Handle as optional (it finds them in the still-kept request and in
the adopted record), and this item stated the fields present and was silent
on a value org-io does not have; the fields are always present, `null` when
they do not apply or are unknown.

**LLR-rywe8f**: the `sends` an `adopted` entry carries holds one entry per
Device org-io attempted, in org-io's order, each with `device` as 64
hexadecimal characters and `result` `sent` or `failed` with `reason`
(org-io's text); a failure for one Device is reported beside the others'
results, not as a refusal of the verification, and the entry's other fields
are those of LLR-jf63e4 unchanged.
satisfies: REQ-qv4d2z

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q1 and answer A2
of 2026-10-16, dispatcher's resolution C8):* report only. There is no
`send_organisation` command and no peer address: the per-Device report is
what `verify_and_send`'s single send (LLR-es49wp) attaches, and this item
states its shape.

**LLR-nkg8sb**: `verify_and_send` asks org-io's `request_organisation`
exactly once for an Organisation whose result is `behind`, with the
Organisation alone and no user step, and attaches org-io's report to that
entry as `requests`, one entry per Device asked, each with `device` as 64
hexadecimal characters and `result` `sent` or `failed` with `reason`
(org-io's text); for every other result it asks for no request and
`requests` is absent. When org-io refuses the request as a whole,
`requests` is empty and `requests_refused` carries org-io's refusal text;
otherwise `requests_refused` is absent.
satisfies: REQ-vgn96n

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* org-io's `request_organisation` can refuse as a
whole, which the per-Device report cannot carry; `requests_refused`
reports it.

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q2 of
2026-10-16):* the request is automatic. It was an IPC command the user's
"Request from its Members" action invoked; it is now made by the body every
verification runs, so a behind result is requested at once wherever the
verification came from. It satisfied REQ-akeah6; it now satisfies
REQ-vgn96n, the requirement that states the automatic request (REQ-akeah6
refers to it).

**LLR-g8wfy9**: each Organisation `list_orgs` returns carries
`verification`, an object tagged by `status`: `in_step` or `adopted` (each
with `epoch`), `removed`, `behind` (`chain_epoch`), `inconsistent` or
`unverified` (each with `reason`), or `not_checked` when no verification has
run for it on this handle. The other fields of the entry are unchanged.
satisfies: REQ-akeah6

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* `removed` is added. org-io's last verification of an
Organisation can be `Removed` while its Persona's acknowledgements are
still being delivered, and REQ-akeah6 lists removed among the results the
user is shown, so it is shown as `removed`, not as `unverified`.

**LLR-73by2q**: `entry_for(named, verified)` maps one org-io `Verified` to
the entry LLR-jf63e4 describes, for the Organisation or founding `named`
names: `EntryOrg::Id` gives `org_id` (40 hexadecimal characters) and no
`founder`; `EntryOrg::Founding` gives `founder` (64 hexadecimal
characters) beside `org_id` or `null`. `InStep`, `Adopted` (with
`discarded` as LLR-n64wvr states, and no `sends`), `Removed`, `Behind` (no
`requests`), `Inconsistent` and `Unverified` give the result of the same
name with its fields; `Refused`, `Store` and `Keychain` give `unverified`
with org-io's text as `reason`; `Settled { outcome, write }` gives
`outcome`'s entry with `write` (`adopted`, `not_executed`, `deprecated` or
`still_unknown`); no other entry carries `write`.
satisfies: REQ-akeah6

*Added 2026-10-25 (unmerged draft; review round 1 T-A5, TEST finding-5):*
the mapping is tested at `entry_for` itself, under TEST_GUIDELINES Seams 2
criteria 1 and 2: `Refused` (an org-node refusal no other result names),
`Store`, `Keychain` and the four `Settled` labels cannot be produced
through `verify_organisation` over the substitute chain without faking
org-io or org-node, code this project owns, and the ten outcomes by three
ways of naming a target would take a separate chain, store or keychain set-up per
case at the command. The command's own tests (LLR-jf63e4) cover the
outcomes the substitute chain does produce.

**LLR-cugcu6**: `device_reports(report)` returns one entry per Device of
org-io's `SendReport`, in its order, each with `device` as 64 hexadecimal
characters: `Delivered` is `result` `sent`; every other delivery is
`failed` with `reason` — `NotDelivered` org-io's text unchanged,
`Refused` "refused by the recipient", `AlreadyReceived` "already
received", `InviteUsedByAnotherDevice` "invite used by another device".
satisfies: REQ-qv4d2z

*Added 2026-10-25 (unmerged draft; review round 1 T-A5, TEST finding-5):*
tested at `device_reports` itself under TEST_GUIDELINES Seams 2 criterion
1: a recipient's `Refused`, `AlreadyReceived` or
`InviteUsedByAnotherDevice` answer to a send cannot be produced through
`verify_and_send` over Loopback without faking org-io. `verify_and_send`'s
own tests (LLR-es49wp, LLR-nkg8sb) cover a delivered send.

**LLR-k4rxyt**: `attach_report(entry, follow_up, report)` attaches org-io's
answer to the send or the request `follow_up` names: for a send beside an
`adopted` entry, a report is `sends` (one entry per Device, as
`device_reports` gives it) and no `sends_refused`, and a refusal of the
whole is `sends` empty beside `sends_refused` with its text; for a request
beside a `behind` entry, the same with `requests` and `requests_refused`;
a follow-up the entry's result does not ask leaves the entry unchanged.
satisfies: REQ-qv4d2z, REQ-vgn96n

*Added 2026-10-28 (unmerged draft; round 2 review, TEST finding-5):*
`app/src-tauri/src/entry.rs`, one of this item's DTO mappings. org-io's
refusal of a send or a request as a whole (the Organisation not held, its
endpoint not bound) cannot be produced through `verify_and_send` over the
substitute chain without faking org-io: the verification that asks the
send has just found the Organisation held, and a refused bind needs the
keychain the verification itself reads. The clause of LLR-es49wp and
LLR-nkg8sb on a refusal of the whole is therefore tested here
(TEST_GUIDELINES Seams 2 criteria 1 and 4).

**LLR-q96vdr**: `OrgDto::new(summary, last)` gives the `verification`
LLR-g8wfy9 describes for org-io's last `Verified` of that Organisation:
`not_checked` for `Unverified` `NotYetVerified`; `in_step`, `adopted`,
`removed`, `behind`, `inconsistent` or `unverified` for the outcome of that
name, with its `epoch`, `chain_epoch` or `reason`; a `Settled` one its
`outcome`'s status; `Refused`, `Store` and `Keychain` `unverified` with
org-io's text as `reason`.
satisfies: REQ-akeah6

*Added 2026-10-25 (unmerged draft; review round 1 T-A5, TEST finding-5):*
tested at `OrgDto::new` itself under TEST_GUIDELINES Seams 2 criterion 1,
for the reason LLR-73by2q gives: `Removed`, `Settled` and `Refused` as a
last verification cannot be produced through `list_orgs` over the
substitute chain without faking org-io. `list_orgs`'s own tests
(LLR-g8wfy9) cover `not_checked`, `in_step`, `behind` and `unverified`.

**LLR-y7nc6x**: `list_admission_requests` takes no argument, calls org-io's
`admission_requests()` once, changes nothing (`list_personas` and
`list_orgs` return what they returned before the call), and returns one
entry per kept request, grouped by Organisation in org-io's order, each
carrying exactly the fields and encodings of the `admission-requested` event
(LLR-dp45mm): `org_id` (40 hexadecimal characters), `invite_id` (64), the
`invitee_name` typed for that Invite, the request's `name`, `surname` and
`preferred_handles` in order, and its `device_public_key` (64), verbatim;
with none kept the list is empty.
satisfies: REQ-y53dqn

Added 2026-10-20 (owner answer A3 of 2026-10-16). The command carries the
org-io method's name, as every command of the surface does; org-io's list
is its exported requirement on kept admission requests.

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* org-io's
design names the method `admission_requests` (returning one
`KeptAdmissionRequestSummary { org_id, invite_id, name, surname,
preferred_handles, device_key, invitee_name }` per kept request); the IPC
command keeps the name `list_admission_requests`, as the app's other
queries are named (`list_personas`, `list_orgs`), and is the one command of
the surface whose name is not the org-io method's. `device_key` is the
entry's `device_public_key`.

## SDD-6zqqm8 — The follow-up decision

`app/src-tauri/src/follow_up.rs`: `after_verification` and
`verifies_after_write`. Added 2026-10-28 (round 2 review, ARCHITECTURE
findings 1 and 2), split from SDD-6pjvhc, which calls it from
`verify_and_send` and `write_organisation_to_chain`. It reads org-io's
outcome and performs no I/O; its interface is tested directly
(TEST_GUIDELINES Seams 2 criterion 4, an SDD item's own interface).

**SDD-6zqqm8**: the decision of what the app asks org-io for next after a
verification or a write it ran, from org-io's outcome alone.
traces: REQ-qv4d2z, REQ-vgn96n

**LLR-292r26**: `after_verification(verified, org_id)` returns "send" for
the Organisation `org_id` names when `verified` is `Adopted` with `own`
true, "request" for it when `verified` is `Behind`, the decision of the
inner outcome for a `Settled` one, and nothing for every other outcome
(`Adopted` with `own` false, `InStep`, `Removed`, `Inconsistent`,
`Unverified`, `Refused`, `Store`, `Keychain`), and nothing for any outcome
when `org_id` is none (a provisional founding whose proxy is not recorded).
satisfies: REQ-qv4d2z, REQ-vgn96n

**LLR-bbfgk7**: `verifies_after_write(report)` is true for org-io's
`WriteReport::Included` and false for `Refused` and `Unknown`.
satisfies: REQ-qv4d2z

The verification an included write asks is what sends an own adoption at
once after the write, with no user step (the choice "An included write is
verified at once" below); LLR-k4rhbs states what the command returns.

## Under SDD-2pa6h6 — Command-boundary parsing

SDD-2pa6h6's responsibility (refusal of malformed operator-supplied values
at the IPC edge) is unchanged; these items replace the parses that named the
old commands. LLR-ecaw34 (`parse_org_id`) is unchanged.

**LLR-97hujy**: every command that takes an `org_id` — `issue_invite`,
`propose_admission`, `propose_revocation`, `cancel_invite`,
`propose_updated_organisation`, `write_organisation_to_chain` and
`verify_organisation` (when one is given) — parses it with `parse_org_id`
before any other argument and before it calls org-io; on a refusal it
returns the parser's message, and none refuses a well-formed identifier.
satisfies: REQ-sjkp8z

*Amended in place 2026-10-20 (unmerged draft; owner rulings Q1, Q2 of
2026-10-16):* `send_organisation` and `request_organisation` leave the list
with the IPC surface; `list_admission_requests` takes no `org_id`.

**LLR-r8jeu8**: `propose_admission` and `cancel_invite` strip at most one
leading `0x` from `invite_id` and refuse a remainder that is not hexadecimal
with a message beginning `invite_id hex`, and one that does not decode to
exactly thirty-two bytes with `invite_id must be 32 bytes (64 hex chars)`,
before they call org-io.
satisfies: derived

## Under SDD-k95rmp — The verification interval

Added 2026-10-20 (owner ruling Q2 and answer A1 of 2026-10-16; SDD-k95rmp
extended in `2026-10-05-decomposition.md`). The interval is configured as
every other startup value is, by an `ODS_*` environment variable that only
`init` reads (SDD-6g3wnh) and a total function in `policy.rs` decides:
`ODS_VERIFICATION_INTERVAL_SECS`, a whole number of seconds. The parse is at
the system edge (the type-safety rule): what leaves `policy.rs` is a
`Duration` that is never zero, and `init` returns the refusal as the startup
failure, as it does every `StartupError`.

**LLR-2dpn52**: `resolve_verification_interval` returns 600 seconds (10
minutes) when the configured value is absent or empty, and otherwise the
configured number of seconds when the value is a decimal integer from 1 to
4294967295 with no sign, space or other character.
satisfies: REQ-ergn9z

**LLR-rvrc25**: `resolve_verification_interval` refuses a value of zero, a
value that is not a decimal integer (a sign, a space, a unit suffix, a
fraction or any other character included) and one above 4294967295 with
`StartupError::InvalidVerificationInterval` carrying the value as given,
whose message names `ODS_VERIFICATION_INTERVAL_SECS`, states that the
verification interval must be a whole number of seconds of at least 1, and
quotes the value; it returns no interval for such a value.
satisfies: REQ-ergn9z

*Amended in place 2026-10-30 (unmerged draft; merge review round 1,
finding-5):* a value that is not UTF-8 is refused too. `init` reads the
variable as the OS holds it and hands it to `verification_interval_from`,
which refuses a value that is not UTF-8 with the same error, carrying the
value's lossy form, and decides every other value as
`resolve_verification_interval` does; `std::env::var(..).ok()` had read such
a value as absent and given the 10-minute default.

**LLR-hgrez9**: the ahead limit the app gives org-io — how long received
Organisation information one epoch ahead of the chain is kept unadopted —
is `policy::verification_ahead_limit(interval)`, twice the verification
interval `init` decided (LLR-2dpn52, LLR-rvrc25): 1200 seconds for the
default 600, and twice any configured interval, the greatest the resolver
accepts (4294967295 seconds) included. `init` hands it to `OrgIo::connect`
as `ChainSettings::ahead_limit`, so the one org-io handle the app holds
gives it to every verification: the command's (at mount and on the
button), the write's, the receiver loop's and the periodic run's. The app
holds no other limit and passes none per verification. `init` builds the
settings with the one function the test constructors take their handle's
limit from, and gives it the interval undoubled; that function doubles it.
satisfies: derived

*Added 2026-10-30 (unmerged draft; merge review MR3-N1, owner decision on
rule 4 of 2026-10-30):* the C+1 time limit is a parameter of org-node's
verification, and the app supplies twice its verification interval so
that at least two chain reads, one per periodic run, fall inside it
whatever the interval; org-io takes it at `connect` and gives it to every
verification (org-io's item on the ahead limit). No app requirement names
the limit: it refines how the configured interval (REQ-ergn9z) is used by
the verification the app asks for (REQ-5q7nyg), as the app's share of
org-node's discard rule for received information one epoch ahead of the
chain.

*Amended in place 2026-10-30 (unmerged draft; merge review round 4,
findings 3 and 8):* it read `satisfies: REQ-ergn9z, REQ-5q7nyg`, neither of
which names a limit; as its note above says, no app requirement does, so it
is derived, and is assessed in `app/docs/risk/2026-10-15-domain-api.md`
beside the other derived LLRs. The text's last sentence is added: the test
constructors doubled the interval themselves, so no test reached `init`'s
wiring; they now build the limit with `init`'s own function, which a test
also calls directly.

*Amended in place 2026-10-31 (unmerged draft; merge review round 5,
finding-4):* the note above said a test reached `init`'s wiring; none
did. A test called the function `init` builds the settings with, but not
`init`'s own call of it, so a regression there (passing the doubled limit
in place of the interval) stayed green. `init` now reads its four
variables (`ODS_CHAIN_WS`, `ODS_CONTRACT_H160`, `ODS_COSIGNER_PUB`,
`ODS_VERIFICATION_INTERVAL_SECS`) and hands what it read to one step,
`startup_chain`, which reads no environment, decides the interval and
builds the settings from it; `init` connects with those settings and
assembles the state with that interval. The test
`startup_chain_builds_the_settings_with_twice_the_interval_it_decides`
calls that step with environment-shaped values. What no test reaches is
`init`'s reading of the variables and its call of `OrgIo::connect`, which
needs a live chain.

*Amended in place 2026-10-31 (unmerged draft; merge review round 6,
finding-5):* the test the note above names was called
`init_builds_the_settings_it_connects_with_from_its_environment_with_twice_the_interval_decided`,
which said it covered `init` and the environment; it calls only
`startup_chain`. It is renamed, its test unchanged.

## Under SDD-aq7m6b — Application state assembly

**LLR-vj5s28**: the app holds and passes no randomness: the app's manifest
(`app/src-tauri/Cargo.toml`) names no `rand`, `rand_core` or `getrandom`
dependency, no source under `app/src-tauri/src` names `OsRng`, `RngCore` or
`CryptoRng`, and no command or receiver call into org-io takes an rng.
satisfies: derived

## Under SDD-mwqf6x — Receiver outcome classification and event vocabulary

SDD-mwqf6x's responsibility is unchanged: it maps what org-io's receive step
returns, and now what a verification it triggers returns, to outcomes and
events, and is still the only place the event names and payloads are
written. Received Organisation information is no longer adopted on receipt
(owner ruling R2), so a received message no longer gives an `Updated`
outcome itself; the verification the app asks at once does (LLR-acb5g4).
LLR-p2nm5a, LLR-hgsdm8, LLR-2vg79y and the error items are unchanged.

*Amended 2026-10-15 (this change; an extension, the ID is kept):* the item
also maps a verification result to outcomes and names the one follow-up a
received message asks for. `traces:` gains REQ-k7ay5b and REQ-88npkt
(amended in place in `2026-10-05-decomposition.md`).

**LLR-4bvdn7**: `outcome_for_arrival` maps org-io's "provisional
organisation received" to one `ProvisionalReceived` outcome naming the
Organisation, and never to `Updated`, `RecordUnreadable` or any other
outcome that states an epoch or root. A `ProvisionalReceived` outcome is
announced by no event of its own: the at-once verification it triggers
(LLR-w7m3d4) announces its result (LLR-8grnbb).
satisfies: REQ-k7ay5b

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* the last sentence is added; the item was silent on
whether a received provisional organisation is announced.

**LLR-ygr6gd**: `outcomes_for_verification` maps a `removed` result to
`SelfDeleted` naming the Organisation, with no epoch, whatever record is
read back, then to one `VerificationResult`; `outcome_for_arrival` maps
org-io's "revocation notice kept" to a `RevocationNoticeKept` outcome naming
the Organisation, announced by no event and stating no epoch, and an
acknowledgement to `AcknowledgementReceived`.
satisfies: REQ-tw4cb5, REQ-k7ay5b

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* it mapped
"an accepted revocation notice, or a received message that removed this
node" arriving to `SelfDeleted`. org-io reports no such arrival: receipt
keeps a revocation notice and acts on nothing (`RevocationNoticeKept`), and
this node's removal is a verification's `removed` result (org-io's
verification requirement, owner ruling Q4'), at which the Persona's data is
deleted. `SelfDeleted` therefore moves to the verification's mapping, still
with no epoch. The notice kept is acted on by the next verification — the
one at start, on request or each interval — as org-io's design states.

*Amended in place 2026-10-21 (unmerged draft; owner decision of
2026-10-20; REQ-k7ay5b as amended):* the sentence above is replaced: a
`RevocationNoticeKept` outcome is still announced by no event of its own,
but it triggers the same at-once verification as a `ProvisionalReceived`
one (LLR-w7m3d4), so the notice is acted on before the receiver loop waits
for the next message, and that verification's `removed` result is mapped as
above (`SelfDeleted`, then the result).

*Amended in place 2026-10-28 (unmerged draft; round 2 review, CODE
finding-1):* "naming the Organisation" is the entry's `org_id`: its 40
hexadecimal characters, or, for an entry that has none (a provisional
founding whose proxy is not yet recorded, LLR-jf63e4), none, carried as
`org_id` null in the `revoked` event's payload, never an empty string; the
`verification-result` that follows names that founding by its `founder`.

**LLR-dp45mm**: `outcome_for_arrival` maps org-io's "admission request kept"
to an `AdmissionRequested` outcome, announced as exactly one
`admission-requested` event carrying `org_id` (40 hexadecimal characters),
`invite_id` (64), the `invitee_name` typed for that Invite, the request's
`name`, `surname` and `preferred_handles` in order, and its
`device_public_key` (64), verbatim.
satisfies: REQ-88npkt

**LLR-t9rdf8**: `outcome_for_arrival` maps org-io's "organisation request
answered" to an `OrganisationRequestAnswered` outcome naming the
Organisation and the requesting Device, which is announced by no event.
satisfies: derived

**LLR-wgr46u**: `classify_receive_error` takes the refusal org-io's
`ReceiveError::Refused` carries, of type `org_io::node::OrgNodeError` (the
only error type of org-io's receive surface the app matches by variant),
and classifies the five refusals of a delivered update's own content —
`OrgIdMismatch`, `StaleSeq`, `MalformedDelta`, `DeltaBaseMismatch` and
`SeqNotEpoch` — together with `RootMismatch` and `StaleEpoch`, as
`VerifyFailed`, carrying the error's message and no organisation; it
classifies every other variant of that type as `ReceiveError`, carrying the
error's message, with no wildcard arm, so a variant org-io's type gains is
a compile error in the app until it is classified.
satisfies: REQ-kn5rtx

*Added 2026-10-21 (owner decision of 2026-10-20; consistency pass).*
The item it replaced named variants org-io's re-exported type no longer has
(`NoProvisionalUpdate`, `PersonaAlreadyBound`, `OrgKeyMismatch`) and
omitted those it gained, through which every refusal of org-io's
chain-free receive step now reaches the app (a sender not listed, a message
too large, an admission request whose key is not the connection's, the
acknowledgement and revocation refusals, the provisional limit, an
Organisation not held). The class rule is unchanged: org-io's receive step
reads no chain (owner ruling R2), so `RootMismatch` and `StaleEpoch` no
longer arrive through it and stay in the verdict class only because the
type still has them; a chain judgement reaches the app as a verification
result (LLR-acb5g4), never through this function. org-io's other
`ReceiveError` variants are LLR-zw6jqy's, unchanged.

*Noted 2026-10-28 (unmerged draft; owner ruling A2 of 2026-10-28):* the
match over `org_io::node::OrgNodeError` with no wildcard arm is a
deliberate tripwire: a variant org-node gains breaks the app's build until
this function classifies it. The owner accepts that org-io's error enums
carry `OrgNodeError` and that the app names it through `org_io::node`; the
exception to the ARCHITECTURE guidelines' Boundaries 2 is recorded under
org-io's handle (the app's one way in) in org-io's design draft
(`org-io/docs/architecture/2026-10-16-domain-api.md`).
`commands.rs`'s match on `OrgNodeError::InvalidField` rests on the same
ruling. No behaviour changes.

**LLR-w7m3d4**: `follow_up_for` returns "verify this Organisation" for a
`ProvisionalReceived` or a `RevocationNoticeKept` outcome and nothing for
every other outcome; the receiver loop runs that verification through
`verify_and_send` once, before it waits for the next message, with no user
step.
satisfies: REQ-k7ay5b

*Amended in place 2026-10-21 (unmerged draft; owner decision of
2026-10-20; REQ-k7ay5b as amended):* adds `RevocationNoticeKept`, which
until now got no follow-up and waited for the next verification at start,
on request or at the interval.

**LLR-j9m88u**: `receive_iteration(next, verify, emit)`, one iteration of
the receiver loop, calls `next` once, emits each outcome's events in order,
and right after an outcome for which `follow_up_for` asks a verification,
calls `verify` once with that Organisation's id and emits every outcome it
returns, all before it returns; it returns `true` (the loop stops) when an
outcome is `Stopped`, otherwise `false`.
satisfies: REQ-k7ay5b

*Added 2026-10-25 (unmerged draft; review round 1 T-A5, TEST finding-5):*
tested at `receive_iteration` itself under TEST_GUIDELINES Seams 2
criterion 1: that the follow-up verification runs before the loop asks
org-io for the next message is an order inside the spawned receiver task.
The real loop shows it only through when org-io's `next_received` is
called, and that cannot be observed without faking org-io, code this
project owns.

**LLR-acb5g4**: `outcomes_for_verification` maps an `adopted` result to
`Updated` with the epoch and root of the record read back after adoption, or
to `RecordUnreadable` naming the Organisation when none can be read, then to
one `VerificationResult`; a `removed` result as LLR-ygr6gd states; every
other result maps to one `VerificationResult` alone, never to `Updated`.
satisfies: REQ-k7ay5b, REQ-2k7ys4, REQ-dp95pv

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* a
`removed` result is LLR-ygr6gd's (`SelfDeleted`, then the result).

*Amended in place 2026-10-21 (unmerged draft; owner decision of
2026-10-20):* the mapping is unchanged; it now also maps the at-once
verification a kept revocation notice triggers (LLR-w7m3d4), whose result
is `removed` when org-io accepts the notice against the chain's state and
otherwise whatever that verification reports.

*Amended in place 2026-10-28 (unmerged draft; round 2 review, CODE
finding-1):* "naming the Organisation" is the entry's `org_id`: its 40
hexadecimal characters, or, for an entry that has none (a provisional
founding whose proxy is not yet recorded, LLR-jf63e4), none, carried as
`org_id` null in the `record-unreadable` event's payload, never an empty
string; with no identifier there is no record to read back, so such an
`adopted` entry is never `Updated`. The `verification-result` that follows
names that founding by its `founder`.

**LLR-8grnbb**: a `VerificationResult` outcome is announced as exactly one
`verification-result` event whose payload is the entry `verify_organisation`
returns for that result (LLR-jf63e4), with its `discarded`, `sends` and
`requests`.
satisfies: REQ-k7ay5b

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q2 of
2026-10-16):* the entry also carries the `requests` report of a `behind`
result (LLR-nkg8sb). The event is the same whichever verification produced
it — the receiver loop's or a periodic run's (LLR-2267hj).

## SDD-tgu7dp — Periodic verification

`app/src-tauri/src/periodic.rs`: `run_periodic_verification`, which takes
the shared org-io handle, the interval `init` decided (LLR-2dpn52) and the
event emitter the receiver loop uses. `run`'s setup spawns it once, with
`tauri::async_runtime::spawn`, after `AppState::init` succeeded (that call
site is SDD-6g3wnh's). Added 2026-10-20 (owner ruling Q2 and answer A1 of
2026-10-16). *Amended in place 2026-10-22 (unmerged draft; plan detail pass
— dispatcher decision):* it named `tokio::spawn`, which panics in Tauri's
setup hook, where no Tokio runtime is entered; Tauri's own async runtime
spawns it.

**SDD-tgu7dp**: the app's periodic verification: while the app runs, once
every verification interval, it asks org-io to verify every Organisation or
provisional founding it holds. Each run goes through the same
`verify_and_send` every other verification runs, so an own adoption is sent
once, a behind Organisation is requested at once, again at each interval
while it stays behind; each result is announced as the receiver loop's
verifications are announced. It decides nothing of the result: org-io
judges, as for every verification.
traces: REQ-5q7nyg, REQ-vgn96n

*Amended in place 2026-10-28 (unmerged draft; round 3 review AR3-5, sweep
for ARCHITECTURE Boundaries 3):* reworded so its responsibility, the
periodic run, is stated in one sentence without "and"; announcing a run's
results is how that run reports, as LLR-2267hj states, not a second
responsibility. Its interfaces and LLRs are unchanged.

**LLR-3ampqm**: `run_periodic_verification` runs no verification before one
interval has elapsed from its start; it then runs `verify_and_send` for
every Organisation and provisional founding org-io holds (no `org_id`) once
per interval, never two runs at once, and when a run takes longer than an
interval the next one starts when it ends, not in a burst of the runs
missed.
satisfies: REQ-5q7nyg

**LLR-2267hj**: each result of a periodic run is mapped by
`outcomes_for_verification` (LLR-acb5g4) and announced through the emitter
it was given exactly as the receiver loop announces a verification it ran:
one `verification-result` event per result (LLR-8grnbb), after the
`Updated` or `RecordUnreadable` event of an `adopted` one.
satisfies: REQ-5q7nyg

**LLR-fe7umz**: over consecutive periodic runs, an Organisation the chain
holds ahead of every provisional organisation org-io holds for it is asked
for with org-io's `request_organisation` exactly once at each run that
reports it `behind`, and not at any run after one that reports it
`in_step` or `adopted`.
satisfies: REQ-vgn96n

## Verification view

`app/src/lib/verification.ts` (`verificationLine`, `verificationLabel`),
the wrapper in `app/src/lib/api.ts` (`verifyOrganisations`), the call at
start in `app/src/routes/+page.svelte` and the rendering in
`app/src/lib/components/PersonaList.svelte` (markup SDD-6g3wnh's).
`app/src/lib/reconcile.ts` is renamed into it.

### When the webview asks for a verification

The wrapper in `app/src/lib/api.ts` (`verifyOrganisations`), the call at
start in `app/src/routes/+page.svelte` and the "Verify with chain" button in
`app/src/lib/components/PersonaList.svelte` (markup SDD-6g3wnh's). Added
2026-10-28 (round 3 review AR3-5), the asking half of the verification
view's earlier single item.

**SDD-ua2d9b**: when the webview asks org-io for a verification: once when
the main page mounts, again each time the user asks, never on a timer of
its own.
traces: REQ-akeah6, REQ-5q7nyg

**LLR-j2x9jb**: the main page calls `verifyOrganisations()` once when it
mounts, and the Organisation list calls it again when the user presses its
"Verify with chain" button; the wrapper invokes `verify_organisation` with
no `org_id`. The webview runs no timer of its own that calls it.
satisfies: REQ-akeah6

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q2 of
2026-10-16):* the at-start and on-request verifications stay here; the
periodic one is LLR-3ampqm's, in the Rust half, and the webview adds no
second timer (one would double the requests each interval).

### The verification display

`app/src/lib/verification.ts` (`verificationLine`, `verificationLabel`,
`fileVerificationResults`) and the rendering in
`app/src/lib/components/PersonaList.svelte` (markup SDD-6g3wnh's). Added
2026-10-28 (round 3 review AR3-5), the display half of the verification
view's earlier single item.

**SDD-swt9fs**: what the webview shows of a verification, whichever
verification produced it (one the user asked for, one a received provisional
organisation triggered, a periodic run); it offers no action to request or
send an Organisation, which the Rust half does with no user step.
traces: REQ-akeah6, REQ-k7ay5b, REQ-5q7nyg, REQ-vgn96n

**LLR-mfzr6z**: `verificationLine` renders each result as one line naming
the Organisation and the result, with the epoch for `in_step` and
`adopted`, the chain's epoch for `behind`, and the reason for
`inconsistent` and `unverified`; a `removed`, `behind`, `inconsistent` or
`unverified` result is never rendered as in step or adopted.
satisfies: REQ-akeah6

*Amended in place 2026-10-22 (unmerged draft; dispatcher decision, plan
assembly):* a result carrying a `write` label (LLR-jf63e4's settled
verification) is rendered as its result's line followed by the write's
settlement — `write adopted`, `write not executed`, `write deprecated` or
`write still unknown`; the label never changes how the result itself is
rendered.

**LLR-njb6mr**: `verificationLabel` renders a `list_orgs` verification as
`in step`, `adopted`, `removed`, `behind (chain at epoch N)`,
`inconsistent: <reason>`, `unverified: <reason>` or `not checked`.
satisfies: REQ-akeah6

*Amended in place 2026-10-22 (unmerged draft; dispatcher decision O3, plan
assembly):* `removed` is added to the labels, as LLR-g8wfy9 (amended
2026-10-22) adds the status to `list_orgs`; a removed Organisation is never
labelled not checked or in step.

*Amended in place 2026-10-28 (unmerged draft; round 3 review, CODE
finding-7 and TEST finding-5):* a status the view does not know is labelled
`unverified: unknown status <status>`, never `not checked`, `in step` or
`adopted`, as `verificationLine` shows an unknown result (LLR-mfzr6z,
HAZ-9fmhm4); `not checked` is the label of `not_checked` alone.

**LLR-zkpsw5**: the view shows no action to request an Organisation for any
result, `behind` included: no source under `app/src` defines
`offersRequest` or `requestOrganisation` or invokes `request_organisation`,
and a `behind` result's line (LLR-mfzr6z) is the same whether or not its
`requests` report names a failed Device.
satisfies: REQ-vgn96n

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q2 of
2026-10-16):* automatic. It stated the user's "Request from its Members"
action; the request is now made by `verify_and_send` with no user step
(LLR-nkg8sb), so this item states the action's absence and moves from
REQ-akeah6 to REQ-vgn96n, whose "no user step" it carries in the webview.

**LLR-84cxpt**: the view files a `verification-result` event into the same
list of results, through the same `verificationLine`, as an entry
`verifyOrganisations` returned, newest first.
satisfies: REQ-k7ay5b

## SDD-xg4nj7 — The Admin's admission and notices view

`app/src/lib/notices.ts` (`admissionConfirmation`, `admissionGate`,
`applyDiscarded`, `dismissNotice`, `reproposal`, `sendOutcomes`,
`fileAdmissionRequests`, `admissionRequestMessage`), their wrappers in
`app/src/lib/api.ts` (`listAdmissionRequests` among them), the call at start
in `app/src/routes/+page.svelte`, and their use in
`app/src/lib/components/Admit.svelte` and `Membership.svelte` (markup
SDD-6g3wnh's).

**SDD-xg4nj7**: the notices the webview shows of a membership change in
flight, from the admission request to the send of the epoch that settles it.
They are the admission requests kept before the start or announced on
arrival, with the confirmation beside each; the notices for discarded
changes, with their re-proposal; each Device's outcome of the single send;
the error an invitee is shown when an invite was used by another Device.
traces: REQ-88npkt, REQ-pe2c7u, REQ-qv4d2z, REQ-s3vnmh, REQ-y53dqn, REQ-vgn96n

*Amended in place 2026-10-28 (unmerged draft; round 3 review AR3-5, sweep
for ARCHITECTURE Boundaries 3):* it read "what the Admin and the invitee are
shown about admissions and sends", which names two subjects; it is reworded
to the one responsibility its functions share, the notices of a membership
change in flight. Its files and LLRs are unchanged.

*Amended in place 2026-10-30 (unmerged draft; merge review round 1,
MR1-A2 and finding-4):* the notices and the send reports are held by the
main page, not the Membership panel, and are filed from every verification
entry (LLR-cgtav9); `applySendReports` and `dismissSendReport` join the
file list, and a refusal of a whole send or request is shown with its
reason (LLR-9r6y7p). `traces:` gains REQ-vgn96n.

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q1 and answer A3
of 2026-10-16):* `failedSends` and its retry give way to `sendOutcomes`
(every Device's outcome, no retry, LLR-fsk4g2), and the kept admission
requests are listed at start (`listAdmissionRequests`,
`fileAdmissionRequests`; LLR-5mw2dj, LLR-dhtec7). `traces:` gains
REQ-y53dqn.

**LLR-47zvjn**: `admissionConfirmation` builds, from an
`admission-requested` event, a confirmation showing the `invitee_name` typed
for that Invite beside the request's `name`, `surname` and
`device_public_key` in full, and marks it a mismatch when the trimmed,
case-folded invitee name differs from the request's name and surname joined
by one space.
satisfies: REQ-88npkt

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* checked
against org-io and org-node: the `invitee_name` is the one the Admin typed
when issuing that Invite, which org-node keeps with the outstanding invite
and returns with each kept admission request, and org-io carries in both
`AdmissionRequestKept` and its list of kept requests; the app keeps none of
its own.

**LLR-e63yrs**: `admissionGate` refuses, naming what is missing, until the
Admin has confirmed that confirmation, and allows the admission only then;
`proposeAdmission` is invoked from the panel only when the gate allows, with
`confirmed` true.
satisfies: REQ-88npkt

**LLR-k5rf3r**: `applyDiscarded` adds one notice per `discarded` change of an
`adopted` result, naming the Member and the kind of change, and keeps every
notice until `dismissNotice` removes that one; no count limit drops a
notice, and a later result does not remove one.
satisfies: REQ-pe2c7u

**LLR-md6hhz**: `reproposal` returns, for an admission notice,
`propose_admission` with its `org_id`, `invite_id` and `handle`, and for a
revocation notice `propose_revocation` with its `org_id` and `member_id`;
the notice stays after its change is proposed again, until it is dismissed.
satisfies: REQ-pe2c7u

*Amended in place 2026-10-22 (unmerged draft; dispatcher decision, plan
assembly):* "Propose again" for an admission notice does not invoke
`propose_admission` with `confirmed` true. It goes back through the
REQ-88npkt confirmation dialog (LLR-e63yrs): the Admin is shown the typed
invitee name beside the kept request's name, surname and DevicePublicKey,
and `propose_admission` is invoked with `confirmed` true only on the
Admin's confirmation there; dismissing the dialog proposes nothing and the
notice stays. This replaces the earlier note of the same day, which
re-proposed with `confirmed` true on the strength of the first
confirmation. A revocation has no confirmation to repeat.

*Amended in place 2026-10-30 (unmerged draft; merge review round 1,
finding-6):* when no kept admission request matches the notice's `org_id`
and `invite_id`, "Propose again" shows "The admission request for this
invite is no longer kept, so it cannot be confirmed again." and proposes
nothing (`reproposalStep`'s `unavailable`); the notice stays. Checked
against org-node 2026-10-30: recording an admission does not remove its
kept request (`record_admission` appends an `Admit` and changes nothing
else); a kept request is removed only when an adopted record lists its keys
(`settle_admitted`) or its invite is cancelled (`cancel_invite`). A
discarded admission's record does not list its keys, so its request is
still kept and the confirmation path is the normal one; `unavailable` is
reached only once the invite was cancelled, or a later adopted epoch
admitted the request after all.

**LLR-fsk4g2**: `sendOutcomes` returns every entry of a `sends` report, in
the report's order, each with its `device` and its result — `sent`, or
`failed` with its `reason` — and offers no action with any of them: no
source under `app/src` defines `sendOrganisation` or invokes
`send_organisation`.
satisfies: REQ-qv4d2z

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q1 of
2026-10-16, dispatcher's resolution C8):* report only. It returned the
failed sends with a retry action calling `sendOrganisation(orgId,
peerAddr)`; the send is made once and the Admin is shown each Device's
outcome with no retry, so the function is renamed `sendOutcomes`, returns
every Device's outcome (REQ-qv4d2z: the outcome for each Device), and the
retry and its address are gone.

**LLR-5mw2dj**: the main page calls `listAdmissionRequests()` once when it
mounts, after it has started listening for `admission-requested` events, and
files the entries it returns with `fileAdmissionRequests`; the wrapper
invokes `list_admission_requests` with no argument. A refused listing is
shown on the page with its text under its own label, "listing the
admission requests at start failed:", and a refused subscription (the page
could not start listening for `verification-result` and
`admission-requested` events) under its own label, "listening for
verification results and admission requests at start failed:"; neither is
shown under the label of the verification at start, nor left unhandled.
satisfies: REQ-y53dqn

*Amended in place 2026-10-25 (unmerged draft; review round 1 T-A4):* the
item stated the listing and not what the page shows when the listing or the
subscription before it is refused; the tests of those two labels verified
behaviour no item described (TEST review round 1, finding-4).

**LLR-dhtec7**: `fileAdmissionRequests` adds each listed kept request to the
Admin's admission requests as one confirmation built by
`admissionConfirmation` (LLR-47zvjn), exactly as for an `admission-requested`
event with the same fields, and holds one confirmation per `org_id` and
`invite_id`: a request both listed and announced, in either order, is shown
once, and a later one with the same pair replaces the earlier.
satisfies: REQ-y53dqn

Added 2026-10-20 (owner answer A3 of 2026-10-16). Listening first, then
listing, and filing both by the pair is what keeps a request that arrives
during the start from being missed or shown twice.

**LLR-d8998k**: `admissionRequestMessage` returns, for an
`invite_used_by_another_device` outcome, a message of severity `error`
stating that another Device already used this invite and telling the user
to contact the person who invited them; it returns no such message for any
other outcome, and the panel shows an `error` message prominently, above
the form.
satisfies: REQ-s3vnmh

**LLR-cgtav9**: the main page holds the notices for discarded changes
(LLR-k5rf3r) and the send reports (LLR-9r6y7p), and files into them every
verification entry the webview receives: the entries `verifyOrganisations`
returns at mount and on "Verify with chain", the `verification` of an
`included` write (a founding's or an update's), and the entry of each
`verification-result` event, whichever tab is open when it arrives. The
Membership panel shows them from the page, so switching tabs neither clears
nor repeats one; each stays until the Admin dismisses it.
satisfies: REQ-pe2c7u, REQ-qv4d2z

Added 2026-10-30 (merge review round 1, MR1-A2): the notices and send
outcomes were the Membership panel's own state, fed only by events that
arrived while it was open, so a tab switch lost every undismissed notice and
the verifications the Admin asked for produced none.

**LLR-9r6y7p**: the Membership panel shows one send report per verification
entry that carries the outcome of a send or a request: an `adopted` entry's
`sends`, each Device's outcome as `sendOutcomes` returns it (LLR-fsk4g2),
or its `sends_refused`, shown as the send refused as a whole with org-io's
reason; and a `behind` entry's `requests_refused`, shown as the request
refused as a whole with org-io's reason. An entry that carries none of
these adds no report. Each report names its Organisation and epoch, offers
no action but "Dismiss", and stays until dismissed; a later report does not
replace an earlier one.
satisfies: REQ-qv4d2z, REQ-vgn96n

Added 2026-10-30 (merge review round 1, finding-4): a refusal of the whole
send or request was carried by the entry and shown nowhere; the Admin saw
an empty list of Devices and no reason.

## SDD-p7sdy3 — The Admin's epoch commands in the webview

`app/src/lib/epoch.ts` (`fileProposal`, `afterWrite`, `proposalMessage`,
`writeMessage`), the wrappers `proposeUpdatedOrganisation` and
`cancelInvite` in `app/src/lib/api.ts`, the proposed updates the main page
holds in `app/src/routes/+page.svelte`, and their use in
`app/src/lib/components/Membership.svelte`, `Admit.svelte` and
`Revoke.svelte` (markup SDD-6g3wnh's). Added 2026-10-30 (merge review round
1, MR1-A1, owner ruling of 2026-10-29: add the screens now).

*Amended in place 2026-10-30 (unmerged draft; merge review round 2,
findings 3 and 6):* `withdrawAdmissionRequest` (`app/src/lib/notices.ts`),
which drops a cancelled Invite's request from the page's list
(LLR-e7bbm5), joins the file list. The proposal, the write and the cancel
now run in `app/src/routes/+page.svelte`, which also holds what the Admin
was last told of each and whether one is in flight; the Membership and
Admit panels only offer the buttons and show what the page holds. So an
outcome that lands after a tab switch is shown on return, and the in-flight
guard outlives the panel (LLR-5vgpkd, LLR-tbtrb3, LLR-e7bbm5, unchanged in
substance).

*Corrected 2026-10-30 (unmerged draft; merge review round 3, finding-12):*
those two behaviours were new, not "unchanged in substance": no LLR stated
them. LLR-5vgpkd, LLR-tbtrb3 and LLR-e7bbm5 now state both.

**SDD-p7sdy3**: the Admin's commands that take a recorded change to the
chain or withdraw it: proposing the updated Organisation from the
provisional changes held, writing a proposed update to the chain, and
cancelling an outstanding Invite; and what the Admin is told of each step,
so that a change proposed is never shown as done.
traces: REQ-vb5pag

**LLR-dt8ak9**: `proposeUpdatedOrganisation(orgId)` invokes
`propose_updated_organisation` with `{ orgId }` alone and resolves to its
answer, `epoch`, `root` and `org_public_key`; `cancelInvite(orgId,
inviteId)` invokes `cancel_invite` with `{ orgId, inviteId }` alone. A
refusal rejects with org-io's text unchanged.
satisfies: REQ-vb5pag

**LLR-5vgpkd**: the Membership panel offers "Propose next epoch" for each
Organisation `list_orgs` lists; pressing it invokes
`proposeUpdatedOrganisation` once for that Organisation and shows "Epoch N
proposed for <Organisation>. It is not on the chain until you press "Write
to chain"." On a refusal (nothing to propose included, which org-io decides
from the provisional changes it holds) it shows "Nothing proposed for
<Organisation>: " and org-io's text, and offers no write for it. While
the proposal is in flight the button is disabled, on the panel shown on
return from another tab too, and a further press invokes nothing. An
outcome that lands while another tab is shown is shown on return.
satisfies: REQ-vb5pag

*Amended in place 2026-10-30 (unmerged draft; merge review round 3,
finding-12):* the last two sentences are added. Task a6 moved the proposal
to the main page, so its in-flight guard outlives the panel and an outcome
that lands after a tab switch is shown on return; no LLR stated either.
The a6 test of a proposal in flight across a tab switch already cites this
item; task a7 added its check that the button is disabled on return, and
a test that a second press while in flight invokes nothing (finding-11).

**LLR-tbtrb3**: the main page holds each proposed update (Organisation and
epoch), the latest proposal of an Organisation replacing an earlier one,
across tab switches; the Membership panel offers "Write to chain" for each.
Pressing it invokes `writeOrganisationToChain({ orgId })` once and shows
the outcome: `included` as "Epoch N written to the chain." with its
`verification` filed as every verification is (LLR-cgtav9), and the offer
withdrawn; `refused` as "Write refused: " and `unknown` as "Write outcome
unknown: ", each with org-io's reason, and a rejected invoke with its text,
each keeping the offer. While the write is in flight the button is
disabled, on the panel shown on return from another tab too. An outcome
that lands while another tab is shown is shown on return.
satisfies: REQ-vb5pag

*Amended in place 2026-10-30 (unmerged draft; merge review round 3,
finding-12):* the last two sentences are added. Task a6 moved the write to
the main page, so its in-flight guard outlives the panel and an outcome
that lands after a tab switch is shown on return; no LLR stated either.
The a6 test that pins them already cites this item.

**LLR-e7bbm5**: the Admit panel offers "Cancel invite" beside each kept
admission request; pressing it invokes `cancelInvite` once with that
request's `org_id` and `invite_id`. On success the request leaves the
page's admission requests (no panel lists it again) and the panel shows
"Invite cancelled: the admission request of <name> <surname> is removed.";
on a refusal it shows org-io's text and the request stays. While the
cancel is in flight the button is disabled, on the panel shown on return
from another tab too. An outcome that lands while another tab is shown is
shown on return.
satisfies: derived

*Amended in place 2026-10-30 (unmerged draft; merge review round 2,
finding-5):* it read `satisfies: REQ-vb5pag`, which states when to propose
and when to write but not cancelling an Invite; like its backend half
LLR-vqz4s6, it is derived, and is assessed beside it in
`app/docs/risk/2026-10-15-domain-api.md`.

*Amended in place 2026-10-30 (unmerged draft; merge review round 3,
finding-12):* the last two sentences of its text are added. Task a6 moved
the cancel to the main page, so its in-flight guard outlives the panel and
an outcome that lands after a tab switch is shown on return; no LLR stated
either. The a6 test that pins them already cites this item.

**LLR-b59vcx**: once an admission or a revocation is proposed, the Admit or
Revoke panel says that it is proposed and is written to the chain only with
the next epoch, by "Propose next epoch" and then "Write to chain" on the
Membership panel; neither panel says that a change was submitted, sent or
is awaited from a peer.
satisfies: REQ-vb5pag

## Pre-command input decisions

`app/src/lib/revoke.ts` (`validateRevokeInput`) and `app/src/lib/invite.ts`
(`ADMISSION_REQUEST_WARNING`, `admissionRequestGate`).

### The admission-request confirmation

`app/src/lib/invite.ts` (`ADMISSION_REQUEST_WARNING`,
`admissionRequestGate`), used by `app/src/lib/components/Invite.svelte`
(markup SDD-6g3wnh's). Added 2026-10-28 (round 3 review AR3-5), the
admission-request half of the pre-command decisions' earlier single item.

**SDD-7aq69w**: the invitee's confirmation before the first admission
request: the warning shown, on whose confirmation alone `request_admission`
may be invoked.
traces: REQ-2yh3c8

**LLR-qcg6bz**: `ADMISSION_REQUEST_WARNING` states that nothing has verified
who sent the Invite, that nothing has verified the Organisation name it
states, and that the admission request reveals the name, surname and
preferred Handles entered to the Device the Invite names; for a provisional
Member whose request the user has not yet confirmed, `admissionRequestGate`
refuses, naming the missing confirmation, until the user confirms, and
allows the request only then.
satisfies: REQ-2yh3c8

**LLR-prh433**: `admissionRequestGate` allows, with no confirmation asked, a
request for a provisional Member whose earlier request the user confirmed;
the retry then invokes `request_admission` with `confirmed` true and no
warning is shown again.
satisfies: REQ-2yh3c8

### The revocation input decision

`app/src/lib/revoke.ts` (`validateRevokeInput`), used by
`app/src/lib/components/Revoke.svelte` (markup SDD-6g3wnh's). Added
2026-10-28 (round 3 review AR3-5), the revocation half of the pre-command
decisions' earlier single item.

**SDD-czjf2c**: whether a revocation may be proposed for the member
identifier the Admin typed, decided before `propose_revocation` is invoked;
it asks for no peer address.
traces: REQ-he8ejb

LLR-csbs5v (`2026-10-05-decomposition.md`) is its LLR.

## SDD-g5wmxt — The creation form's refused-Person decision

`app/src/lib/founding.ts` (`personRefusalStep`), called by
`app/src/lib/components/CreateOrg.svelte` (markup SDD-6g3wnh's), which makes
the `propose_person` and `propose_founding_organisation` calls. Added
2026-10-28 (round 3 review, ARCHITECTURE finding-6, Core and edges 1 and 3):
the decision sat inside `proposeFounding`, the function making the two IPC
calls, in a module importing the IPC adapter; no SDD item named
`founding.ts`. The decision is now a pure function the form calls, and its
interface is tested directly (TEST_GUIDELINES Seams 2 criterion 4, an SDD
item's own interface).

**SDD-g5wmxt**: the decision, when `propose_person` rejects the creation
form's Person, whether the founding is still proposed or what is shown in
its place.
traces: REQ-vb5pag

**LLR-d8u93q**: `personRefusalStep(refusal)` returns "proceed" for a
`PersonRefusalDto` whose `kind` is `person_already_held`, whatever its
`message`; "stop" with the refusal's `message` for one of any other `kind`;
and "stop" with the rejection's own text for any value that is not a
`PersonRefusalDto` (an object with a string `kind` and a string
`message`): a string as it is, any other value as `String` gives it, so the
message shown is never missing.
satisfies: REQ-vb5pag

**LLR-5sxa6c**: `app/src/lib/founding.ts` imports types alone: each of its
`import` statements is an `import type`, and it names no `@tauri-apps/api`
module, so the decision calls no IPC adapter function.
satisfies: REQ-vb5pag

## SDD-23vjd8 — The Persona list's Handle

`app/src/lib/personas.ts` (`handleLabel`, `NO_HANDLE_YET`), called by the
Persona rows of `app/src/lib/components/PersonaList.svelte` (markup
SDD-6g3wnh's). Added 2026-10-28 (round 3 review, CODE app note 1): the
webview typed `PersonaDto.handle` as always present, while `list_personas`
reports it null for a Persona that holds no Handle yet (LLR-pguhw5), so a
provisional Member's row showed nothing or "null" where its Handle stands.

**SDD-23vjd8**: what the Persona list shows as each Persona's Handle.
traces: REQ-vb5pag

**LLR-q735zx**: `handleLabel(handle)` returns the Handle `list_personas`
reported, unchanged, and for a null Handle (a Persona that holds none yet,
such as a provisional Member) the stated label `NO_HANDLE_YET`, "(no Handle
yet)", which names neither null nor undefined; the Persona list shows each
Persona's `handleLabel` in its Handle column, so no row shows "null" there.
satisfies: REQ-vb5pag

## SDD-6g3wnh — what this change adds to the untested shell

*Amended 2026-10-15 in `2026-10-05-decomposition.md`:* its `traces:` loses
the requirements now out of force (the Invite's and the Invite reply's
production, the Invite
reply's warning and the submission decision) and gains REQ-vb5pag, REQ-2yh3c8, REQ-akeah6 and REQ-k7ay5b,
realised at call sites no gated test reaches: the Svelte panels for the
Person, the founding, the Invite, the Member proposal, the admission
request, the confirmation, the proposal and the write; the receiver loop's
call of `verify_and_send`; and the success paths of every command that needs
a chain or a peer.

*Amended 2026-10-20 in `2026-10-05-decomposition.md`:* `traces:` gains
REQ-5q7nyg, REQ-ergn9z and REQ-y53dqn for `init`'s read of
`ODS_VERIFICATION_INTERVAL_SECS` and its propagation of
`StartupError::InvalidVerificationInterval` as the startup failure, `run`'s
setup spawning `run_periodic_verification` (SDD-tgu7dp) with the decided
interval, and the `Admit.svelte` markup that renders the kept admission
requests. `AppState` carries the decided interval from `init` to `run`
(SDD-aq7m6b's assembly takes it as one more decided value; `for_test` and
`for_test_over` pass the default).

## Choices made where the design is silent

- The automatic verification on a received provisional organisation, and the
  send after an own adoption, run in the Rust half (`verify_and_send`, shared
  by `verify_organisation` and the receiver loop), not in the webview, so
  they happen whether or not a view is listening.
- An included write is verified at once (LLR-k4rhbs), so the send after
  adoption follows a write with no user step; a refused or unknown write is
  not, and waits for the next verification at start or on request.
- *Withdrawn 2026-10-20 (owner ruling Q1):* a repeated send asked org-io to
  send the whole epoch again. There is no repeated send.
- *Withdrawn 2026-10-20 (owner answer A3):* a kept admission request reached
  the Admin only as the `admission-requested` event. The kept requests are
  now listed at start (LLR-y7nc6x, LLR-5mw2dj, LLR-dhtec7).

Added 2026-10-20 (owner rulings Q1, Q2, answers A1–A3 of 2026-10-16):

- **Where the period runs.** In the Rust half (SDD-tgu7dp), like the
  automatic verification on receipt, so the pull happens whether or not a
  view listens; the webview keeps only the at-start and on-request calls.
  The first periodic run is one interval after start, because the at-start
  verification is the main page's (LLR-j2x9jb); a late run delays the next
  rather than bursting (tokio's delay policy on missed ticks).
- **One body for every verification.** The request of a behind
  Organisation is made by `verify_and_send`, so a verification at start, on
  request, after a write, on receipt or periodic all request at once; "again
  each interval until in step or adopted" falls out of the periodic run
  re-deriving `behind` (org-io keeps no persisted behind marker), with no
  per-Organisation retry state in the app.
- **No IPC send or request.** Both left the surface rather than staying
  unused: an IPC `send_organisation` would be a retry by another name.
- **The interval's form.** `ODS_VERIFICATION_INTERVAL_SECS`, whole seconds,
  1 to 4294967295 (a `u32`, so no `Instant` arithmetic can overflow); an
  empty value counts as absent, as `ODS_DATA_DIR`'s empty override does
  (LLR-kfmng5), and gives the 10-minute default.
- **The list's org-io name.** `list_admission_requests` is the name assumed
  for org-io's list of kept admission requests; the IPC command follows
  whatever name org-io's design gives it. *Amended in place 2026-10-20
  (consistency pass):* org-io's design names it `admission_requests`; the
  IPC command stays `list_admission_requests` and calls it (LLR-y7nc6x).

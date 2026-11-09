# org-io: the demo journey's domain surface (design)

Change `worktree-org-io-domain-api`. Software items and low-level
requirements for org-io's requirements draft
`org-io/docs/requirements/2026-10-14-domain-api.md`
(REQ-ggjuu6, REQ-3pjmub, REQ-36gxmg, REQ-nactq6, REQ-bm4ppg, REQ-sjqxg2,
REQ-z73ctw, REQ-wpq4fp, REQ-7jh4rh, REQ-kh2uth, REQ-qfnx94, REQ-4bvmwc,
REQ-ek652a, REQ-xd5a2d, REQ-cj9u3y and the expectations REQ-2yw5y3,
REQ-u894ty, REQ-2xgsbn) and its risk draft
(`org-io/docs/risk/2026-10-15-domain-api.md`, RC-6m89tr).

*Revised 2026-10-20 (unmerged draft; owner rulings Q1–Q3, Q4' (replacing
Q4), Q6 and answers A1–A4 of 2026-10-16, the rulings of 2026-10-20, and the
dispatcher's resolutions C1, C3, C7–C9):* the requirements draft gained
REQ-wb93ch (the list of kept admission requests), REQ-wd683g (on-chain-client's
genesis in two steps), REQ-k6kcgd, REQ-sd7ayv, REQ-98a3ty, REQ-jqmpz5 (acting
on this node's removal) and the expectation REQ-bg5whd, and amended
REQ-36gxmg, REQ-nactq6, REQ-sjqxg2, REQ-z73ctw, REQ-wpq4fp, REQ-xd5a2d,
REQ-kh2uth, REQ-qfnx94 and REQ-2xgsbn. Draft items are amended in place,
each with a dated note; dated items are superseded. No operation takes a
peer address any more: every send goes to the endpoint addresses org-node
keeps for the recipient's Device key, which the recipient's own request
carried (org-node's exported Invite and request building, REQ-ky5tyd), or,
for an admission request and a provisional Member's organisation request, to
the Invite's. Beside org-node's three exported items above, this unit now
relies on two more it exports: deleting an accepted acknowledgement
(REQ-ayyt5f) and listing the kept acknowledgements (REQ-35s4eb); and on
on-chain-client's two exported genesis steps (REQ-v86pyy) and their resume
from chain reads (REQ-6vh3ht). The new section "Acting on this node's
removal" (SDD-83un8g, split on 2026-10-28 into SDD-3my66m and SDD-wyn7b6) holds the acknowledgement delivery and the
deletions that follow it.
The decisions are the two decision records of 2026-10-15 in `docs/adr/`,
named by description because the repository's ADRs sit outside this unit's
compliance scope: the batched-epochs record (provisional changes batched
into one provisional organisation per epoch) and the write-verify-send
record (write, verify and send are three steps the app calls). Drafted 2026-10-15, after S4 merged, from the
owner-confirmed design of that day. Every item is class C, the unit's class;
no per-item override and no segregation is argued. Each low-level
requirement is MISSING-TEST until its plan task writes the test, which is
the truth.

org-io stays IO only: the chain (write, read at a finalised block), the
transport, the store files and the keychain. Every judgement is org-node's,
reached through org-node's sans-IO operations, which take values and return
a store change and an outcome. org-node's items are named by description,
except the three it exports to this unit: its one receive path driven by
values (REQ-ncv4sc), its messages for the adopted current epoch
(REQ-ttv8k7) and its Invite and admission-request building and checking
(REQ-ky5tyd). No new `depends_on` edge: the Person reaches org-io only as an
org-node type (SDD-j7k844).

## What changes, for an implementer

```
org-io/src/
  rng.rs         # new: the handle's random source (OsRng; seeded under test-support)
  person.rs      # new: propose_person, person-<hex>.store, its keychain item
  persona.rs     # new: a new Persona per proposal (founding, Member): keychain, file, endpoint
  exchange.rs    # issue_invite, read_invite, cancel_invite (reply Blob operations removed)
  propose.rs     # new: propose_admission, propose_revocation, propose_updated_organisation
  write.rs       # replaces submit.rs's sequence: write_organisation_to_chain (chain IO only)
  verify.rs      # replaces publish.rs's settle and reconcile: verify_organisation
  send.rs        # send_organisation, request_admission, request_organisation, answers
  receiver.rs    # next_received: check_received only, no chain read, four-value verdict
  transport/mod.rs  # Verdict gains AlreadyReceived (0x02) and InviteUsedByAnotherDevice (0x03)
  clock.rs       # new (2026-10-20): Clock, SystemClock; replaced under test-support only
  acknowledge.rs # new (2026-10-20): removal acknowledgements, backoff (SDD-3my66m)
  retention.rs   # new (2026-10-28): the removed Persona's deletion, split out of acknowledge.rs (SDD-wyn7b6)
```

*Amended in place 2026-10-20 (unmerged draft):* `send.rs` takes no peer
address any more; `exchange.rs` gains the list of kept admission requests;
`keychain.rs` gains `delete_device_secret` and `endpoints.rs` a direct-only
bind for a removed Persona.

The public methods removed from `OrgIo`: `create_persona`,
`found_organisation`, `admit_member`, `revoke_member`, `expect_admission`,
`produce_reply`, `import_reply`, `admit_reply`, `submit_commit_send`,
`settle_unknown`, `reconcile`, `reconcile_held`. None of the remaining or new
public methods takes a random source.

## The handle's random source

**SDD-3qwjs2**: the handle's random source: the one cryptographic
random-number generator `OrgIo` owns, from which every random value of its
operations is drawn, so that no caller supplies or sees one. Those values
are the keys org-node draws for a Person, a Persona, a provisional
organisation and an Organisation key pair, the invite identifier, and the
store file's temporary suffix.
Code: `org-io/src/rng.rs`, `OrgIo`'s constructor. SOUP: `rand_core` 0.6.4
over `getrandom` 0.2.17.
traces: REQ-ek652a, REQ-ggjuu6, REQ-6fk4qj

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-4 sweep, ARCHITECTURE Boundaries 3):* the responsibility is restated in one sentence without "and", the detail it listed moved to the sentences after it. No behaviour change.

**LLR-4qd5fx**: `OrgIo` holds one `HandleRng`, built by `OrgIo::open` as
`rand_core::OsRng` (rand_core 0.6.4, `getrandom` feature), behind its own
`std::sync::Mutex`; every org-node operation an `OrgIo` method calls with a
random source is passed this one (as `&mut impl RngCore + CryptoRng`), and
`store_file::write_store` draws the temporary file's 16-hex suffix from it;
no file of `org-io/src` outside `rng.rs` and `test_support.rs` names
`OsRng`, `thread_rng`, `StdRng`, `SmallRng` or `from_entropy`.
satisfies: REQ-ek652a, REQ-ggjuu6

**LLR-ctm86b**: no public method of `OrgIo` takes a parameter whose type is,
or is bounded by, `RngCore` or `CryptoRng`, and no public type of org-io
holds or returns the handle's random source; the scan in
`org-io/tests/absences.rs` reads every `pub fn` signature in `org-io/src`,
joined across lines (as LLR-u2pk5y's scan does), and fails on `RngCore`,
`CryptoRng` or `HandleRng` in one, `test_support.rs` excepted.
satisfies: derived

**LLR-5acgfg**: a build with the `test-support` feature offers
`OrgIo::with_rng_for_test(self, rng: Box<dyn CryptoRngCore + Send>) -> Self`,
which replaces the handle's random source with the given one (a seeded
`rand_chacha`-style generator in tests), so a test can reproduce an invite
identifier or a key pair; a build without `test-support` compiles no
function that replaces or seeds the source, and `OrgIo::open` and
`OrgIo::connect` take none.
satisfies: derived

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* the argument's type is `Box<dyn CryptoRngCore +
Send>` (rand_core's combined trait, which a boxed trait object needs, since
`RngCore + CryptoRng` names two non-auto traits); the behaviour is
unchanged.

**LLR-94nn6q**: when the operating system's entropy source fails, org-io's
own draws (`try_fill_bytes` on the handle's source, for the store file
suffix) return `StoreFileError { step: CreateTemporary, .. }` naming the
random source, write nothing and fall back to no other source; a draw that
org-node makes through `RngCore::fill_bytes` on `OsRng` panics (rand_core
0.6.4's documented behaviour for a `getrandom` error) inside the org-node
call, before the operation returns a store change, so nothing is written
for it and no value drawn from anything but the OS source is used. No code
in `org-io/src` catches that panic or substitutes a generator.
satisfies: derived

## The Person

**SDD-j7k844**: the Person: the user's Person, proposed apart from any
Organisation, kept as a small sealed Person file under its own keychain
item, reported as provisional. Code: `org-io/src/person.rs`,
`store_file.rs` (the Person file's name and listing), `keychain.rs` (the
item). The Person value, its DevicePublicKey and PersonPublicKey are
org-node's Person store type, re-exported through `org_io::node`; org-io's
manifest names no `person` crate (owner ruling 2026-10-15: no org-io →
person edge for now), so `depends_on` gains no edge.
traces: REQ-ggjuu6, REQ-rz93uz, REQ-hw82ue, REQ-fh5gsq

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-4 sweep, ARCHITECTURE Boundaries 3):* the responsibility is restated in one sentence without "and", the detail it listed moved to the sentences after it. No behaviour change.

**LLR-3cysjk**: `OrgIo::propose_person(name: &str, surname: &str) ->
Result<PersonSummary, CreateError>` refuses, when the handle already holds
a Person, with `CreateError::PersonAlreadyHeld { device_key }` naming that
Person's DevicePublicKey, before org-node is called or anything is drawn,
writing no keychain item and no file and leaving the Person held as it was.
Otherwise it has org-node build a new Person store
from the name, the surname and keys drawn from the handle's random source
(no Handle; the Person key pair's secret half is kept in the sealed Person
file, under the Person's Device seed, as a Persona's member seed is in its
store), writes the Person's Device seed to its keychain item (service
`"ods-device-seed"`, user the 64 lower-case hex of the Person's
DevicePublicKey, through `write_device_secret`, LLR-677fqq's scoped write)
first, then writes the sealed bytes to
`<data_dir>/person-<64 lower-case hex of that key>.store` with
`write_store`'s steps (LLR-b9bm36), and only then returns. A name or
surname org-node refuses is returned as `CreateError::Node` with no item
and no file written; a keychain refusal is `CreateError::Keychain`, with no
file written and the Person dropped; a failed file write is
`CreateError::Store`, the Person dropped and its item left in place, as for
a Persona (LLR-n9dnm4's note).
satisfies: REQ-ggjuu6, REQ-rz93uz

*Amended in place 2026-10-22 (unmerged draft; plan detail pass — owner
ruling of 2026-10-22, and dispatcher decision on the Person key):* a second
Person is refused (REQ-ggjuu6 as amended: one Person per device), and the
sealed Person file keeps the Person key pair's secret half, as org-node's
Person store now holds it.

**LLR-mas8vc**: every `PersonSummary` org-io returns (`propose_person`,
`OrgIo::person()`) carries the Person's name, surname, DevicePublicKey and
PersonPublicKey and `status: PersonStatus::Provisional`; org-io has no
operation that writes a Person hash to the chain or verifies one, and
returns no other `PersonStatus` value in this change.
satisfies: REQ-ggjuu6

**LLR-z7kbf9**: `OrgIo::open` lists the Person files of the data directory
(`person-<hex>.store`, parsed to a DevicePublicKey as LLR-nt8dja parses a
Persona's) apart from the Persona store files, and unseals each through its
keychain item; a Person file whose item is missing is refused with
`OpenError::KeychainItemMissing` naming the key, the file left untouched and
nothing created in its place; the Persona listing ignores Person files and
the Person listing ignores Persona files. `OrgIo::person()` returns the one
Person the handle holds, or `None` when it holds none.
satisfies: REQ-ggjuu6, REQ-fh5gsq

*Amended in place 2026-10-22 (unmerged draft; plan detail pass — owner
ruling of 2026-10-22):* `person()` returns the one Person, since a second
is refused (LLR-3cysjk); it no longer has a choice among Persons to make.

*Amended in place 2026-10-22 (unmerged draft; dispatcher decision, plan
assembly):* a data directory with two or more Person files (a corruption:
`propose_person` refuses a second Person) is refused by `OrgIo::open` with
`OpenError::MoreThanOnePerson` naming the Device keys of every Person file
found; no file is unsealed, changed or removed and no handle is returned.

**LLR-b97usg**: `org-io/Cargo.toml` names no `person` crate as a
dependency, dev-dependency or build-dependency, and no file of `org-io/src`
has a `use person` or `person::` path: org-io names the Person only through
org-node's types (`org_io::node`).
satisfies: derived

**LLR-g6kaa8**: no endpoint is bound for the Person's Device key:
`start_receiving` and every send bind endpoints only for Persona store
files, and a message addressed to the Person's key is never accepted by
this node.
satisfies: REQ-ggjuu6

## A new Persona per proposal

**SDD-bzu2jg**: the Persona's creation: each provisional founding or
provisional Member is a new Persona, created by the operation that proposes
it. Each has its own Device seed keychain item, sealed store file and
endpoint, with the Organisation name kept in that store. Code:
`org-io/src/persona.rs`. Replaces `OrgIo::create_persona` and the choice of
an existing Persona to found or reply with.
traces: REQ-rz93uz, REQ-g9cu2c, REQ-bupq5y, REQ-4bvmwc, REQ-2xgsbn

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-4 sweep, ARCHITECTURE Boundaries 3):* the responsibility is restated in one sentence without "and", the detail it listed moved to the sentences after it. No behaviour change.

**LLR-kna4zj**: `propose_founding_organisation` and `propose_member` each
create a new Persona and never reuse a held one: org-node returns a new
Persona store and its Device secret, org-io writes the Device seed to the
Persona's keychain item (`write_device_secret`) first, then the store file
`persona-<hex>.store` (LLR-nt8dja, LLR-b9bm36), and only then reports
success. When org-node refuses (a name, surname, Handle or Invite it
refuses), no item and no file is written; when the keychain write fails the
new store is dropped, no file is written and the refusal is
`CreateError::Keychain`; when the file write fails the Persona is dropped,
its item stays, and the refusal is `CreateError::Store`.
satisfies: REQ-rz93uz, REQ-g9cu2c
supersedes: LLR-n9dnm4, LLR-rt8gdz

**LLR-6sfub6**: `keychain::write_device_secret(device_key, secret:
&DeviceSecret)` hands the Device seed's bytes to the keychain only inside
org-node's scoped write: its backend's one `set_secret` call is the closure
it passes to `secret.lend_to_keychain(|bytes| entry.set_secret(bytes))`,
and nothing else in `keychain.rs` calls `set_secret` or receives the lent
bytes, so a new Persona's (and the Person's) Device secret is written to the
keychain item and nowhere else (REQ-rz93uz, RC-jgyd6m).
satisfies: REQ-rz93uz

*Added 2026-10-28 (unmerged draft; review round 1 T-O7):* the rule was
stated only in LLR-n9dnm4, which LLR-kna4zj superseded without carrying it
over (LLR-kna4zj states the keychain-before-file order of the proposals, not
how the bytes reach the keychain). This item restates it unchanged.

*Note 2026-10-28 (robustness, N/A):* a source scan of `keychain.rs`, which
takes no input, so it has no abnormal-input case; it was watched red
against a planted second `set_secret` call outside `lend_to_keychain`.

**LLR-4yte89**: `OrgIo::propose_founding_organisation(name, surname,
handle, org_name: &str) -> Result<ProposedFounding, CreateError>` has
org-node build the new Persona's provisional founding (epoch 1, its
Membership root and Organisation public key) with the Organisation name
given, writes it as LLR-kna4zj states, and returns `ProposedFounding {
founder: DevicePublicKey, epoch, root, org_public_key }`; it touches no
chain. A written Persona whose view org-node then gives with no founding is
org-io's own refusal, `CreateError::FoundingNotListed { founder }`.
satisfies: REQ-4bvmwc, REQ-g9cu2c

*Amended in place 2026-10-28 (unmerged draft; round 3 review, T3-O8):*
`CreateError::FoundingNotListed` is reached by no test (owner ruling of
2026-10-28: unreachable): org-node's `propose_founding` returns a store
holding the founding it built, and org-io reads the view only after writing
that store, so the view lacks the founding only under a double of org-node,
which the project owns and TEST_GUIDELINES Doubles 1 rules out. The
abnormal case tested is the chain's failure: with every chain read and
write failing, the founding is proposed and no chain call is made.
*Corrected 2026-10-28 (round 4 TEST review, low 9):* the note cited Seams 2
criterion 1 as the reason for no test; that criterion is what earns a
deeper seam a direct test, not what excuses one. The reason is the owner's
ruling with Doubles 1, as the notes on LLR-rav7dr, LLR-bv6wkz and
LLR-3fxxpp give.

*Amended in place 2026-10-28 (unmerged draft; round 2 review, AR2-5):* org-io's own
post-check was returned as an `OrgNodeError::NoProvisionalFounding` it
built, which org-node never returned; it is
`CreateError::FoundingNotListed`.

**LLR-6d6wfw**: `OrgIo::propose_member(invite_blob: &str, name, surname,
handles: &[&str]) -> Result<ProposedMember, CreateError>` parses the Invite
through org-node (a Blob that does not parse is `CreateError::Node` naming
the field, with nothing written), has org-node build the new Persona's
provisional Member with its expected first admission and the Organisation
name the Invite carries, writes it as LLR-kna4zj states, and returns
`ProposedMember { member_device: DevicePublicKey, org_id, invite_id,
org_name }`; fewer than one, more than three or repeated Handles are
org-node's refusal, returned with nothing written; it sends nothing.
satisfies: REQ-4bvmwc, REQ-2xgsbn

**LLR-jvx6j9**: once `start_receiving` has been called, `propose_founding_organisation`
and `propose_member` bind the new Persona's endpoint and give it an accept
task (`Endpoints::accept_if_receiving`, as LLR-z35vbh's note of 2026-10-14
states for `create_persona`) after the store file is written; a bind that
fails there does not undo the creation and is reported in the result
(`bind: Option<TransportError>`).
satisfies: REQ-bupq5y

**LLR-pn3hgc**: `OrgIo::organisations()` and `OrgIo::organisation(org_id)`
report each held Organisation's stored name in `OrgSummary::name`; no
`OrgIo` method but `propose_founding_organisation` takes an Organisation
name, and org-io hands the name to no send, chain write or chain read (it
reaches a peer only inside the Invite Blob org-node builds).
satisfies: REQ-4bvmwc

*Noted 2026-10-28 (unmerged draft; round 3 review, T3-O8):* org-node
refuses no Organisation name (`propose_founding` keeps it as given), so the
item's abnormal case is the name's edges: an empty name, and one with
surrounding spaces around text past ASCII, are each reported unchanged.

## The invitation operations

**SDD-6gb59p**: the invitation operations: issuing an Invite from the
Organisation's Persona with that Device's endpoint addresses, reading an
imported Invite, and cancelling an outstanding invite, each through
org-node's Invite operations (REQ-ky5tyd) and each writing the store
org-node returns before it reports; and listing the admission requests the
held Organisations keep, which writes nothing. Code: `org-io/src/exchange.rs`.
traces: REQ-ek652a, REQ-pnr6kh, REQ-2xgsbn, REQ-4bvmwc, REQ-wb93ch
supersedes: SDD-rr7dtw
superseded-by: SDD-zqv3v6, SDD-9rbm8t, SDD-cxh9d8

*Amended in place 2026-10-20 (unmerged draft; owner answer A3 of
2026-10-16):* the item also lists the kept admission requests (REQ-wb93ch,
LLR-dm2h8u), so that the app can show them at start.

*Superseded 2026-10-28 (unmerged draft; round 2 review AR2-6, ARCHITECTURE
Boundaries 3):* the item stated four operations of three responsibilities —
the Organisation's outstanding invites (issue, cancel), an imported
Invite's reading, and the listing of the kept admission requests — so a
change to any one edited it. It is split, as SDD-83un8g was, into
SDD-zqv3v6 (the outstanding invites: LLR-95e3jx, LLR-v9dkrv, LLR-8ze6cc),
SDD-9rbm8t (an imported Invite read: LLR-npw5zg) and SDD-cxh9d8 (the kept
admission requests listed: LLR-dm2h8u), whose code stays in
`org-io/src/exchange.rs`. Its LLRs are re-homed below under those items. No
behaviour changes.

### The outstanding invites (successor of SDD-6gb59p)

**SDD-zqv3v6**: the Organisation's outstanding invites as org-node keeps
them: each issued from the Persona that holds the Organisation, or cancelled
with its kept admission request, through org-node's Invite operations
(REQ-ky5tyd), the store org-node returns written before it reports. An
issued Invite carries that Device's endpoint addresses and the stored
Organisation name. Code:
`org-io/src/exchange.rs` (`issue_invite`, `cancel_invite`).
traces: REQ-ek652a, REQ-pnr6kh, REQ-2xgsbn, REQ-4bvmwc
supersedes: SDD-6gb59p

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-4 sweep, ARCHITECTURE Boundaries 3):* the responsibility is restated in one sentence without "and", the detail it listed moved to the sentences after it. No behaviour change.

*Added 2026-10-28 (unmerged draft; round 2 review AR2-6):* SDD-6gb59p's
outstanding-invite half. LLR-95e3jx, LLR-v9dkrv and LLR-8ze6cc are its
LLRs.

**LLR-95e3jx**: `OrgIo::issue_invite(org_id, invitee_name: &str) ->
Result<String, ExchangeError>` has org-node build the Invite in the store
of the Persona that holds `org_id`, with an invite identifier drawn from
the handle's random source, the stored Organisation name, the invitee name
guess, that Persona's DevicePublicKey and the addresses of LLR-v9dkrv;
org-node records the (Organisation, invite identifier) pair as outstanding
in that store, and org-io writes the store before it returns the Blob; a
failed write returns `ExchangeError::Store` and no Blob, and an
Organisation no Persona here holds is org-io's own refusal,
`ExchangeError::NotHeld { org_id }`, with nothing built or written. The
pairs survive a reopen of the data directory and leave the store only by
org-node's settle on adoption or by `cancel_invite`.
satisfies: REQ-ek652a, REQ-pnr6kh, REQ-2xgsbn
supersedes: LLR-h9qf3s, LLR-9sraks, LLR-f35pda

*Amended in place 2026-10-28 (unmerged draft; round 2 review, AR2-5):* an Organisation no
Persona here holds is found by org-io's own lookup of the Persona holding
the current record, before org-node is asked; org-io returned it as an
`OrgNodeError::OrgNotHeld` it built itself, which org-node never returned.
It is now org-io's own `ExchangeError::NotHeld`.

**LLR-v9dkrv**: the addresses `issue_invite` gives org-node are those the
issuing Persona's endpoint holds in the current mode: its bound loopback
sockets in Loopback mode, its direct addresses in Networked mode (possibly
none), each given as the text `<64 lower-case hex endpoint id>@<socket>`
(the endpoint identity, which is the Persona's Device key, then `@`, then
the socket address); the endpoint is bound first if it is not (LLR-d2pzzk),
and a bind that fails refuses with `ExchangeError::Bind`, building and
writing nothing.
satisfies: REQ-ek652a

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* the text form of every endpoint address org-io gives
org-node, in an Invite or a request, and reads back from what org-node keeps
(LLR-34tfm8, LLR-qa4nrt), is `<64 lower-case hex endpoint id>@<socket>`.
org-node holds it as opaque text within its 256-byte entry bound and reads
nothing of it.

**LLR-8ze6cc**: `OrgIo::cancel_invite(org_id, invite_id) -> Result<(),
ExchangeError>` has org-node remove the admission request kept for that
invite and the outstanding invite, and writes the store before it
returns `Ok`; a request naming an invite of `org_id` for which org-node
keeps no admission request is org-node's refusal, returned as
`ExchangeError::Node` with nothing written;
a failed write is `ExchangeError::Store`, the Persona reloaded from its
file (LLR-murjr6); an Organisation no Persona here holds is org-io's own
refusal, `ExchangeError::NotHeld { org_id }`, with nothing written.
satisfies: REQ-2xgsbn, REQ-pnr6kh

*Amended in place 2026-10-28 (unmerged draft; round 2 review, AR2-5):* the Organisation
no Persona here holds is org-io's own `ExchangeError::NotHeld`, not an
`OrgNodeError::OrgNotHeld` org-io built itself.

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* it read
"the admission request kept for that invite (if any)" and refused only "a
request naming no outstanding invite". org-node's cancel removes a kept
admission request and its invite and refuses an invite id with no kept
request (`NoAdmissionRequestKept`), as its requirement on removing a kept
request states; this item now says the same.

### An imported Invite read (successor of SDD-6gb59p)

**SDD-9rbm8t**: reading an imported Invite: its public fields, parsed
through org-node's Invite parse (REQ-ky5tyd), with nothing written or
created. Code: `org-io/src/exchange.rs` (`read_invite`).
traces: REQ-2xgsbn
supersedes: SDD-6gb59p

*Added 2026-10-28 (unmerged draft; round 2 review AR2-6):* SDD-6gb59p's
reading of an imported Invite. LLR-npw5zg is its LLR.

**LLR-npw5zg**: `OrgIo::read_invite(invite_blob: &str) ->
Result<InviteSummary, ExchangeError>` parses the Invite through org-node
and returns its public fields (Organisation name, Organisation identifier,
invitee name guess, issuing DevicePublicKey and its addresses as text),
writes nothing and creates nothing; a Blob that does not parse is
`ExchangeError::Node` naming the field that failed.
satisfies: REQ-2xgsbn

### The kept admission requests listed (successor of SDD-6gb59p)

**SDD-cxh9d8**: the listing of the admission requests the held
Organisations keep, for the app to show at start, read from org-node with
nothing changed. Code: `org-io/src/exchange.rs` (`admission_requests`).
traces: REQ-wb93ch
supersedes: SDD-6gb59p

*Added 2026-10-28 (unmerged draft; round 2 review AR2-6):* SDD-6gb59p's
listing of the kept admission requests. LLR-dm2h8u is its LLR.

**LLR-dm2h8u**: `OrgIo::admission_requests() -> Vec<KeptAdmissionRequestSummary>`
takes, under the node's lock, from every Persona store holding a current
record, each admission request org-node keeps for it (REQ-ky5tyd), and
returns one `KeptAdmissionRequestSummary { org_id, invite_id, name,
surname, preferred_handles, device_key, invitee_name }` per kept request —
`invitee_name` being the name the Admin typed when issuing that invite —
in store-file order and, within a store, in org-node's order; it writes no
store, reads no chain, sends nothing and changes no state, so two calls with
nothing received between them return equal lists; a handle holding no kept
request returns an empty list; the summary holds no endpoint address and
no key but the request's DevicePublicKey.
satisfies: REQ-wb93ch

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* the
fields come from org-node's `admission_requests(device_key)` alone, which
returns each kept request with the invitee name its outstanding invite
keeps (org-node's exported Invite and request item, REQ-ky5tyd, as amended
that day); `device_key` is the request's DevicePublicKey, which org-node
holds equal to the Device the transport authenticated.

## Provisional changes and the proposed Organisation

**SDD-frsa73**: the Admin's provisional changes and the proposal of the
updated Organisation: recording an admission or a revocation, and having
org-node calculate the provisional organisation from all of them, each
writing the store org-node returns and touching no chain or transport; and
no command that discards. Code: `org-io/src/propose.rs`.
traces: REQ-g9cu2c, REQ-3pjmub, REQ-7jh4rh, REQ-kh2uth
superseded-by: SDD-7mhqaz, SDD-4drrpr

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q4' of
2026-10-16):* its responsibility is unchanged; LLR-vvd8dn, the unit-wide
absence of deletions it carries, now exempts the deletions that follow this
node's removal (REQ-kh2uth as amended), which SDD-83un8g makes.

*Superseded 2026-10-28 (unmerged draft; round 3 review AR3-4, ARCHITECTURE
Boundaries 3):* the item stated two responsibilities, recording the Admin's
provisional changes and having org-node calculate the updated Organisation
from them, which change for different reasons. It is split, as SDD-83un8g
and SDD-6gb59p were, into SDD-7mhqaz (the provisional changes recorded:
LLR-un47j8, LLR-6kb33y, LLR-vvd8dn) and SDD-4drrpr (the updated
Organisation proposed: LLR-5fchwt), whose code stays in
`org-io/src/propose.rs`. Its LLRs are re-homed below under those items. No
behaviour changes.

### The provisional changes recorded (successor of SDD-frsa73)

**SDD-7mhqaz**: the Admin's provisional changes: each admission or
revocation the Admin records, kept by org-node in the Organisation's store,
which org-io writes before it reports, touching no chain or transport, with
no command that discards one. Code: `org-io/src/propose.rs`
(`propose_admission`, `propose_revocation`).
traces: REQ-g9cu2c, REQ-7jh4rh, REQ-kh2uth
supersedes: SDD-frsa73

*Added 2026-10-28 (unmerged draft; round 3 review AR3-4):* SDD-frsa73's
recording half. LLR-un47j8, LLR-6kb33y and LLR-vvd8dn are its LLRs.

**LLR-un47j8**: `OrgIo::propose_admission(org_id, invite_id,
chosen_handle: Option<&str>) -> Result<ProvisionalChangeSummary,
OperationError>` has org-node record an admission provisional change from
the admission request kept for that invite (with the Admin's chosen Handle
when one is given), writes the store before it returns, and returns the
change with the Handle given; every org-node refusal (no kept request, no
free Handle and none chosen, a behind Organisation, the 1 MiB limit) is
`OperationError::Node` with nothing written. org-io's own refusals are its
own variants: an Organisation no Persona here holds is
`OperationError::NotHeld { org_id }`, nothing written; a recorded and
written admission that org-node's provisional changes then list no Handle
for is `OperationError::AdmissionNotListed { invite_id }`.
satisfies: REQ-g9cu2c
supersedes: LLR-y4dg74

*Amended in place 2026-10-28 (unmerged draft; round 5 TEST review, Notes):*
untested clause: `OperationError::AdmissionNotListed`. org-node's
`record_admission` returns a store holding the admission it recorded, and
org-io lists the provisional changes only after writing that store under
the same lock, so the list lacks the admission's Handle only under a double
of org-node, which the project owns (TEST_GUIDELINES Doubles 1), as the
notes on LLR-rav7dr, LLR-bv6wkz, LLR-3fxxpp and LLR-4yte89 give. The
abnormal cases tested are org-node's refusals and `NotHeld`.

*Amended in place 2026-10-28 (unmerged draft; round 2 review, AR2-5):* org-io returned
its own lookup's refusal and its own post-check as `OrgNodeError`s it
built (`OrgNotHeld`, `NoAdmissionRequestKept`), which org-node never
returned; they are org-io's `OperationError::NotHeld` and
`OperationError::AdmissionNotListed`.

**LLR-6kb33y**: `OrgIo::propose_revocation(org_id, member_id) ->
Result<ProvisionalChangeSummary, OperationError>` has org-node record a
revocation provisional change, writes the store before it returns; when
org-node refuses (a Member id the record does not hold, a behind
Organisation, the 1 MiB limit) it returns `OperationError::Node`, writes
nothing, writes nothing to the chain and sends nothing; an Organisation no
Persona here holds is org-io's own refusal, `OperationError::NotHeld {
org_id }`, with the same nothing.
satisfies: REQ-g9cu2c
supersedes: LLR-nqdr97

*Amended in place 2026-10-28 (unmerged draft; round 2 review, AR2-5):* the Organisation
no Persona here holds is org-io's own `OperationError::NotHeld`, not an
`OrgNodeError::OrgNotHeld` org-io built itself.

**LLR-vvd8dn**: no public method of `OrgIo` discards a provisional
organisation or a provisional change, and no file of `org-io/src` calls
`std::fs::remove_file`, `std::fs::rename`, `remove_dir` or `remove_dir_all`,
or `.retain(` / `.remove(` / `.clear(` on a store's
records, but `store_file::write_store` (its own temporary file),
`store_file::mark_removed` and `store_file::remove_removed_file` (each called only
from `retention.rs`'s Persona deletion, LLR-vj2nkb, which `OrgIo::open`
also runs, LLR-upqj5g); no file but `retention.rs` calls
`keychain::delete_device_secret`; every other deletion is inside a store
change org-node returns from verification (LLR-pbng7d), from its deletion of
an accepted acknowledgement (LLR-6ux6yw) or from `cancel_invite`, which
org-io writes unread.
satisfies: REQ-7jh4rh, REQ-kh2uth
supersedes: LLR-cc59ys

*Amended in place 2026-10-28 (unmerged draft; review round 1 C-O12):*
`store_file::remove_store` is renamed `remove_removed_file`, for what it
removes: only a `removed-<hex>.store` file, never a live store file. The
notes below keep the old name as it was then. No behaviour change.

*Amended in place 2026-10-28 (unmerged draft; review round 1 A8):* the
Persona deletion moved from `acknowledge.rs` to `retention.rs`
(SDD-wyn7b6), so the one file calling `mark_removed`,
`remove_removed_file` and `keychain::delete_device_secret` is
`retention.rs`. No behaviour change.

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q4' of
2026-10-16):* org-node no longer returns `StoreChange::Forgotten` on this
node's removal — it returns a sealed store keeping the acknowledgements —
so `remove_store` moves from that store change to the Persona deletion of
REQ-98a3ty (LLR-vj2nkb), which also deletes the keychain item; the scan
names both exempt places.

*Amended in place 2026-10-20 (unmerged draft; consistency pass):*
`store_file::mark_removed`, the rename that now comes before the keychain
deletion (LLR-vj2nkb), joins the exempt functions, and `std::fs::rename`
the scanned names (`write_store`'s own rename of its temporary file is
inside `write_store`); "org-node's forget operation" leaves the list, since
org-node has none (its removal is a sealed store, LLR-zcw3xv).

### The updated Organisation proposed (successor of SDD-frsa73)

**SDD-4drrpr**: the proposal of the updated Organisation: org-node's
calculation of one provisional organisation from every provisional change
held, with an Organisation key pair drawn from the handle's random source,
written to the store before it is reported, with no chain read or write.
Code: `org-io/src/propose.rs` (`propose_updated_organisation`).
traces: REQ-g9cu2c, REQ-3pjmub
supersedes: SDD-frsa73

*Added 2026-10-28 (unmerged draft; round 3 review AR3-4):* SDD-frsa73's
proposal half. LLR-5fchwt is its LLR.

**LLR-5fchwt**: `OrgIo::propose_updated_organisation(org_id) ->
Result<ProposedOrganisation, OperationError>` has org-node apply every held
provisional change to the current record with one Organisation key pair
drawn from the handle's random source, writes the store before it returns
`ProposedOrganisation { org_id, epoch, root, org_public_key }`, and makes
no chain read or write; org-node's refusals (no provisional change, a behind
Organisation) are returned with nothing written. An Organisation no Persona
here holds is org-io's own refusal, `OperationError::NotHeld { org_id }`,
with nothing written; a written proposal that org-node's provisional
organisations then list as no current proposal of this node's is
`OperationError::ProposalNotListed { org_id }`.
satisfies: REQ-g9cu2c, REQ-3pjmub

*Amended in place 2026-10-28 (unmerged draft; round 5 TEST review, Notes):*
untested clause: `OperationError::ProposalNotListed`. org-node's proposal
returns a store holding the provisional organisation it built, and org-io
lists the provisional organisations only after writing that store under the
same lock, so the list lacks it only under a double of org-node, which the
project owns (TEST_GUIDELINES Doubles 1), as the notes on LLR-rav7dr,
LLR-bv6wkz, LLR-3fxxpp and LLR-4yte89 give. The abnormal cases tested are
org-node's refusals and `NotHeld`.

*Amended in place 2026-10-28 (unmerged draft; round 2 review, AR2-5):* org-io returned
its own lookup's refusal and its own post-check as `OrgNodeError`s it
built (`OrgNotHeld`, `NoProvisionalChange`); they are org-io's
`OperationError::NotHeld` and `OperationError::ProposalNotListed`.
org-node's own `NoProvisionalChange` (no change held) is still returned as
`OperationError::Node`.

## The chain write

**SDD-mk39e5**: writing one provisional organisation to the chain, a
founding or the next epoch of a held Organisation, through on-chain-client.
The write is bounded at 90 seconds, mortal from a write window, refused when
stale, and changes nothing in the store but a founding's proxy address; its
outcome is reported included, refused or unknown, an unknown one held for
verification to settle. Code: `org-io/src/write.rs` (replacing
the sequence of the former submit module, removed by T28; `ChainWriter`,
`WRITE_TIMEOUT` and `bounded` stay, in `write.rs`).
The production writer is the chain connection item's (SDD-z85ux9).
traces: REQ-3pjmub, REQ-qfnx94, REQ-bm4ppg, REQ-3pxa8f, REQ-tg9zrn, REQ-wd683g
supersedes: SDD-erzj3m

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-4 sweep, ARCHITECTURE Boundaries 3):* the responsibility is restated in one sentence without "and", the detail it listed moved to the sentences after it. No behaviour change.

*Amended in place 2026-10-20 (unmerged draft; owner answer A4 of
2026-10-16):* the founding's two steps are on-chain-client's exported
genesis steps (REQ-v86pyy), which resume from chain reads (REQ-6vh3ht);
REQ-wd683g, the expectation they meet, joins the line.

**LLR-s7rq5x**: `OrgIo::write_organisation_to_chain(target: HeldTarget) ->
Result<WriteReport, WriteRefused>` (`HeldTarget::Organisation(OrgId)` for
the next epoch, `HeldTarget::Founding(DevicePublicKey)` for a provisional
founding) writes the Organisation's latest own provisional organisation
through `ChainWriter` and returns `WriteReport::Included`, `Refused {
failure }` or `Unknown`; it calls no `StateReader` method and no org-node
operation that adopts, commits or discards, and writes no store change but
LLR-5tpdxw's. On `Refused` the provisional organisation stays held.
satisfies: REQ-3pjmub
supersedes: LLR-qhjp6g, LLR-9r2bxd

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-2,
ARCHITECTURE Boundaries 2):* `Refused { reason: String }` is `Refused {
failure: WriteFailure }`: the step that failed (`CreateProxy`,
`CreateProxyUnanswered`, `Founding { proxy, cause }`, `WindowRead`,
`WindowReadTimedOut`, `Update`) with the `ChainWriter` error it received,
kept as its `std::error::Error::source()`. Its `Display` is the text the
reason held, so the app's text is unchanged.

**LLR-48me7m**: before any chain call `write_organisation_to_chain`
refuses, writing nothing to the chain or the store, with
`WriteRefused::NotHeld` when the store holds no own provisional organisation
for the target, `WriteRefused::NoProxyAccount` when an update's record
holds no proxy account, and `WriteRefused::Stale` when org-node reports the
provisional organisation no longer the proposal of the current record and
the current provisional changes (its base epoch is not the current record's
epoch, or a provisional change was recorded or discarded after it was
proposed). A `WriteRefused` that has a cause keeps it, typed, as its
`std::error::Error::source()`: org-node's refusal for `Node`, the store
failure for `Store` (LLR-5tpdxw); the others have none.
satisfies: REQ-3pjmub

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-2,
ARCHITECTURE Boundaries 2):* the chain-write seam's errors are typed now
(`ChainWriteError`, LLR-a66ufr's note of this day); the review round 1 A3
note's "unchanged" is withdrawn.

*Amended in place 2026-10-28 (unmerged draft; review round 1 A3,
ARCHITECTURE Boundaries 2):* the refusals named their cause only in their
text; `source()` now returns it. The chain-write seam's `String` errors
(`ChainWriter`) are unchanged: typing them is not this item's change.

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* org-node's
report is the `current_proposal` flag of its view of the provisional
organisation (`provisional_organisations(device_key)`): `Stale` exactly when
it is false. No operation of org-node's design reported staleness before
this pass; its view now does.

**LLR-mhn4n5**: while an unknown outcome is recorded for an Organisation
(or a provisional founding), `write_organisation_to_chain` for it refuses
with `WriteRefused::OutcomeUnknown`, writing nothing, until verification
settles it (LLR-zey4mg); an unknown outcome for one target never refuses a
write for another.
satisfies: REQ-bm4ppg
supersedes: LLR-nqvbb6

**LLR-5tpdxw**: for a founding the write runs in two `ChainWriter` steps,
`create_proxy()` and `found_at(proxy, root, key)`; as soon as
`create_proxy` returns the pure proxy's address, org-io has org-node record
it with the provisional founding and writes the store, and only then calls
`found_at`; a failed store write stops the write there and returns
`WriteRefused::Store` saying the proxy exists and naming it, the Persona
reloaded (LLR-murjr6). A founding whose proxy is already stored skips
`create_proxy` and calls `found_at` with the stored address, so one founding
never creates a second proxy.
satisfies: REQ-qfnx94, REQ-wd683g

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-2,
ARCHITECTURE Boundaries 2):* the two steps return `ChainWriteError`, not
`String`: `create_proxy() -> Result<(OrgId, ChainAccount), ChainWriteError>`
and `found_at(proxy, root, key) -> Result<(), ChainWriteError>` keep
on-chain-client's `WriteError` as `ChainWriteError::Write`, whose `Display`
names a genesis step's failure as `genesis step <step> did not execute:
<reason>`, the text the 2026-10-20 note's mapping gave; the writer of a
handle with no chain refuses each with `ChainWriteError::NotConfigured`.

*Amended in place 2026-10-20 (unmerged draft; owner answer A4 of
2026-10-16):* `ChainWriter::create_proxy` and `ChainWriter::found_at` are,
in the production writer, on-chain-client's two exported genesis steps
(REQ-v86pyy) — creating the pure proxy under the controller, then funding,
mapping and recording the genesis root and key at expected epoch zero — and
not a split org-io makes of the one-call genesis. org-io keeps no memory of
which part of `found_at` ran: a retry calls `found_at` again with the stored
proxy, and on-chain-client skips the funding or the mapping the chain shows
done (REQ-6vh3ht). A step's failure is LLR-sq5pbq's.

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* the seam's
signatures against on-chain-client's. `ChainWriter::create_proxy() ->
Result<(OrgId, ChainAccount), String>` calls on-chain-client's
`create_proxy(ops, signatory, co_signatories)` with the writer's signatory
and co-signers and returns the proxy account it answers with the
Organisation identifier on-chain-client's exported `h160_of` gives for it;
org-io records both with org-node's `record_founding_proxy(rng, device_key,
org_id, proxy)`. `ChainWriter::found_at(proxy, root, key) -> Result<(),
String>` calls on-chain-client's `found_at(ops, signatory, co_signatories,
contract, proxy, root, key)` with the writer's signatory, co-signers and
contract, and maps its `WriteError::Step { step, reason }` to the reason
LLR-sq5pbq reports.

**LLR-sq5pbq**: when `found_at` returns on-chain-client's typed error naming
the step that did not execute (funding, mapping, or recording the genesis),
`write_organisation_to_chain` returns `WriteReport::Refused { reason }` whose
reason names that step and the stored proxy, records no unknown write,
keeps the provisional founding with its stored proxy address unchanged, and
writes nothing else; the next `write_organisation_to_chain` for that founding
calls `found_at` with the same proxy and no `create_proxy`. When
`create_proxy` itself fails, no proxy is stored and the report is `Refused`
naming the proxy creation. A `found_at` that does not return within
`WRITE_TIMEOUT` is LLR-d3farr's unknown outcome, not a refusal. A
recording refused because the slot already holds a genesis (an earlier
`found_at` that executed unreported) is reported the same way; verification
of the founding at its stored proxy (LLR-4qyeut) then adopts it when the
chain's genesis matches.
satisfies: REQ-wd683g, REQ-3pjmub, REQ-qfnx94

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-2,
ARCHITECTURE Boundaries 2):* the reports are typed: a step `found_at`
refused is `Refused { failure: WriteFailure::Founding { proxy, cause } }`,
`cause` on-chain-client's `WriteError::Step` (as `ChainWriteError::Write`)
and the text naming the step and the proxy as before; a failed
`create_proxy` is `WriteFailure::CreateProxy(cause)`, one past the bound
`WriteFailure::CreateProxyUnanswered`.

*Amended in place 2026-10-28 (unmerged draft; round 4 ARCHITECTURE review,
finding-4, Domain 1 and 2):* a `create_proxy` past the bound is an
Unanswered proxy creation (root glossary, `docs/CONTEXT.md`), not a
timed-out write: the variant is `WriteFailure::CreateProxyUnanswered`
(earlier `CreateProxyTimedOut`) and its text reads "the pure proxy creation
went unanswered for 90 s; nothing stored, and the next write creates the
proxy again". Behaviour is unchanged.

*Amended in place 2026-10-24 (unmerged draft; owner decision on PR-8ghk9s):*
a `create_proxy` that does not return within `WRITE_TIMEOUT` is reported as
its failure is: `WriteReport::Refused { reason }` naming the proxy creation
and the 90 s bound, no proxy stored and no unknown write recorded, so the
next `write_organisation_to_chain` for that founding calls `create_proxy`
again. It is not LLR-d3farr's unknown outcome: with no proxy stored,
verification reports `NoProxyYet` (LLR-4qyeut) and could never settle it
(LLR-zey4mg). A proxy the timed-out call did create is left orphaned with
its proxy deposit only. A `found_at` timeout is unchanged.

**LLR-d3farr**: each `ChainWriter` call the write makes is bounded by
`WRITE_TIMEOUT` (90 s, `tokio::time::timeout`); when it elapses the call is
treated as an unknown outcome: org-io records the unknown write
(LLR-7y46be), keeps the provisional organisation, changes nothing else,
and returns `WriteReport::Unknown` saying the write timed out after 90 s and
may yet execute on chain.
satisfies: REQ-bm4ppg
supersedes: LLR-be3zv9

*Amended in place 2026-10-24 (unmerged draft; owner decision on PR-8ghk9s):*
every call stays bounded, but a founding's `create_proxy` past the bound is
not treated as an unknown outcome: it is refused and records nothing
(LLR-sq5pbq). The unknown outcome is recorded for an update's write and for
a founding's `found_at` at its stored proxy.

**LLR-a66ufr**: for an update, `write_organisation_to_chain`, after its
refusals and before the write, calls `ChainWriter::write_window()` once and
passes that window to `ChainWriter::update`; when `write_window` fails it
writes nothing and returns `WriteReport::Refused` saying no write was made.
satisfies: REQ-bm4ppg
supersedes: LLR-2yjuaj

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-2,
ARCHITECTURE Boundaries 2):* `ChainWriter`'s calls return `ChainWriteError`
(`Write(WriteError)` for on-chain-client's writes, `Window(ClientError)` for
the window's read, `NotConfigured` with no chain), not text. A failed window
read is `Refused { failure: WriteFailure::WindowRead(cause) }` (one past the
bound `WindowReadTimedOut`), and an update the chain did not execute
`WriteFailure::Update(cause)`, each saying what its text said.

**LLR-7y46be**: the unknown write org-io records for a target holds the
provisional organisation's Membership root and Organisation public key, the
epoch the write expected, and, for an update, the window's
`last_valid_block()`; it is kept in the handle's memory
(`PublishState`), one per target, and is cleared only by LLR-zey4mg.
satisfies: REQ-bm4ppg

**LLR-j4nkx5**: the `ChainWriter` seam answers as the chain does:
`write_window()` returns a window born at or after the finalised block read
before it; `create_proxy()` then `found_at(proxy, root, key)` found the
Organisation `create_proxy` named at epoch 1 with that root and key;
`update(proxy, root, key, expected_epoch, window)` at the state's epoch,
mortal from a window just read, moves the state one epoch on to its root and
key, and one from an epoch the chain has left is refused as the call made
through the proxy failed (`ChainWriteError::Write(WriteError::InnerCallFailed(_))`),
the state unchanged. Every writer org-io holds, the production writer and
the test-support fake, meets it.
satisfies: REQ-3pjmub

*Added 2026-10-28 (unmerged draft; round 4 TEST review, finding-1):* see
LLR-js7wgk's note.

## The chain read (successor of SDD-6qz9ms)

**SDD-4bbt47**: the read of an Organisation's state from the chain at a
finalised block, handed to org-node as a value, made only by
verification: no receive, send or write path reads the chain. Code:
`org-io/src/chain_read.rs` (the `StateReader` seam and the parse,
unchanged). LLR-rm9x4z, defined under SDD-6qz9ms, stays in force beside it.
traces: REQ-tg9zrn, REQ-36gxmg, REQ-cj9u3y
supersedes: SDD-6qz9ms

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-4 sweep, ARCHITECTURE Boundaries 3):* the responsibility is restated in one sentence without "and", the detail it listed moved to the sentences after it. No behaviour change.

**LLR-k3zmqb**: `StateReader::read_state` and
`StateReader::finalised_block` are called in `org-io/src` only from
`verify.rs` (and `preflight.rs`'s operator checks); `receiver.rs`,
`send.rs`, `write.rs`, `propose.rs`, `exchange.rs`, `person.rs`,
`persona.rs`, `acknowledge.rs` and `retention.rs` name no `StateReader`.
satisfies: REQ-tg9zrn, REQ-cj9u3y

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q4'):*
`acknowledge.rs` joins the list: the acknowledgement delivery reads no chain.

*Amended in place 2026-10-28 (unmerged draft; review round 1 A8):*
`retention.rs`, the deletion split out of `acknowledge.rs` (SDD-wyn7b6),
joins the list.

**LLR-js7wgk**: the `StateReader` seam answers as the chain does:
`finalised_block()` returns the latest finalised block's number, at or
before the birth of a write window read after it and later once the chain
finalises another block; `read_state(org_id)` returns `Ok(None)` for an
Organisation the chain holds no state for, and, for one founded or updated
through `ChainWriter`, its root and key at the epoch written; a state whose
Organisation public key org-node's parse refuses is an `Err`, the error
`chain_read::org_state_from_chain` gives. Every reader org-io holds, the
production `OnChainStateReader` and the test-support fake, meets it.
satisfies: REQ-36gxmg, REQ-tg9zrn

*Added 2026-10-28 (unmerged draft; round 4 TEST review, finding-1, the
floor "every test fails when the behaviour it verifies breaks"):* the
contract tests of the chain seams (`tests/common/chain_contract.rs`, run over
`FakeChain` in `tests/chain_write.rs` and over the production halves in
`tests/chain_writer_contract.rs`) cited LLRs of `verify_organisation`,
`write_organisation_to_chain` and `is_own_admin`, which they do not call.
This item, LLR-j4nkx5 and LLR-bwerd2 state the seams' own contract, which
those tests verify. No behaviour changes.

## Verification

**SDD-t4z8sz**: verification: org-node's judgement of one held
Organisation, provisional founding or provisional Member against the
Organisation's state read from the chain at a finalised block, reported as
one outcome. org-io reads the state, hands it to org-node's verification,
writes the store change org-node returns, and reports the outcome with the
provisional changes an adoption left out and what the same read settles of
an unknown chain write. Code:
`org-io/src/verify.rs` (replacing the settle and reconcile of the former
publish module, removed by T28).
traces: REQ-36gxmg, REQ-bm4ppg, REQ-7jh4rh, REQ-kh2uth, REQ-tg9zrn
supersedes: SDD-6mpmsw

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-4 sweep, ARCHITECTURE Boundaries 3):* the responsibility is restated in one sentence without "and", the detail it listed moved to the sentences after it. No behaviour change.

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q4' of
2026-10-16 and the dispatcher's resolution C1):* on this node's removal
verification gives org-node the time read from the handle's clock and,
through a closure, the Persona's Device secret key for the signing; it
writes the sealed store org-node returns, which keeps only the
acknowledgements, and keeps the Persona's store file; and it then hands the
Persona to the acknowledgement delivery (SDD-83un8g), with no app
confirmation. On `Behind` nothing is written: the behind marker is not
persisted and is derived again by each verification.

**LLR-t2pve7**: `OrgIo::verify_organisation(target: HeldTarget) ->
Verified` calls `StateReader::finalised_block()` then
`StateReader::read_state(org_id)` once each, hands the parsed state (or its
absence) to org-node's verification, and maps its outcome to `Verified::InStep
{ epoch }`, `Adopted { epoch, discarded, own }`, `Removed`, `Behind {
chain_epoch }` or `Inconsistent { reason }`; it sends nothing.
satisfies: REQ-36gxmg
supersedes: LLR-k5pzsz

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-2,
ARCHITECTURE Boundaries 2):* `Verified::Refused { reason }`, org-node's
error text, is `Verified::Refused { error }`, org-node's `OrgNodeError`
itself, kept shared (`Cause<OrgNodeError>`) so that the outcome stays
`Clone`; its `Display` is the text the reason held.

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q4' of
2026-10-16):* "it sends nothing" stands for `verify_organisation`'s own
call: on `Removed` it starts the acknowledgement delivery (LLR-zzsqj4) and
returns without awaiting any send, so the outcome it returns is the same
whatever the delivery later reports.

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* the call
against org-node's design: `verify(rng, device_key, chain, verified_at,
device_secrets)`, with `verified_at` the `VerificationTime` of LLR-fzf2n3
and `device_secrets` the closure of LLR-8w2z64. org-node's `Removed {
org_id, acknowledgements_kept }` maps to `Verified::Removed`; the count is
not reported, since LLR-zzsqj4 reads the kept acknowledgements themselves.
org-node's refusals `OrgNotOnChain` (no state on the chain) and
`FoundingNotWritten` map to `Unverified` with the reason, and
`DeviceSecretNotSupplied` to `Verified::Keychain` (LLR-8w2z64).

*Amended in place 2026-10-22 (unmerged draft; dispatcher decision, plan
assembly):* every other org-node refusal of the verification (one this item
and LLR-8w2z64 do not map, `AdmissionNotOurs` or `OrganisationRemoved` for
instance, and `DeviceSecretNotSupplied` with no keychain refusal noted) is
returned as `Verified::Refused { reason }`, `reason` org-node's error text,
and nothing is written. `Verified` carries no target: `verify_all` returns
each outcome beside its `HeldTarget` (LLR-kq7z58).

*Amended in place 2026-10-30 (unmerged draft; merge review MR3-N1, owner
decision on rule 4):* the call against org-node's design is `verify(rng,
device_key, chain, verified_at, ahead_limit, device_secrets)`, with
`ahead_limit` the handle's (LLR-22se2q); the stand-in constant org-io gave
before is removed.

**LLR-zcw3xv**: `verify_organisation` writes the store change org-node's
verification returns before it reports: on `Adopted` the new record
(`write_store`), and on `Removed` the sealed store org-node returns, which
keeps only that Organisation's acknowledgements, their recipients' keys and
addresses and the verification time (`write_store`; the Persona's store file
stays and `remove_removed_file` is not called); on `Behind`, `InStep` and
`Inconsistent` it writes only the sealed change org-node returns when its
discard rules discarded or stamped a received provisional organisation
(org-node's discard of what it cannot adopt, LLR-22se2q's ahead limit), and
otherwise nothing, so the store file's bytes
are unchanged. A failed write reports `Verified::Store` with the failure, not the outcome,
the Persona reloaded (LLR-murjr6) with the behind marker the store it
replaces held (org-node's `restore_store`; none when that store is in the
removed form, which holds no marker), and starts no acknowledgement delivery.
satisfies: REQ-36gxmg, REQ-g9cu2c

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-2,
ARCHITECTURE Boundaries 2):* `Verified::Store { failure }` holds the
`StoreFailure` itself (`Cause<StoreFailure>`), not its text.

*Amended in place 2026-10-30 (unmerged draft; merge review round 4,
MR4-O1):* org-node's verification returns a sealed store change on
`Behind`, `InStep` and `Inconsistent` when its discard rules discard or
stamp a received provisional organisation, and
`verify_organisation` writes it (`write_store`), as on `Adopted`; with no
such change it writes nothing. A failed write of that change is reported as
a failed write of an adoption is: `Verified::Store` with the failure, not
`Behind`, `InStep` or `Inconsistent`, the Persona reloaded from its file,
which is as it was (the received provisional organisation still held, no
stamp). The outcome is not reported beside the failure, so for a behind
Organisation the app makes no organisation request that round (its
follow-up requests on `Behind` only); the next verification discards or
stamps again and reports the outcome. The note of 2026-10-20 ("on `Behind`
nothing is written") stands for the behind marker, which is still not
persisted.

*Amended in place 2026-10-31 (unmerged draft; merge review round 5
finding-2):* the Persona reloaded after a failed write of that change on a
`Behind` verification keeps the behind marker the verification set:
org-node's `restore_store` carries the replaced store's marker over
(org-node's behind-marker item, amended 2026-10-31). Until then the reload cleared it, so
until the next verification the Admin could record or propose against a
record the chain had moved past, and an organisation answer was refused as
not requested. Test:
`a_failed_write_of_a_discard_on_a_behind_verification_keeps_the_organisation_behind`.

*Amended in place 2026-10-31 (unmerged draft; merge review round 6
finding-3):* the restore rule of the note above now stands in the
statement, with the case org-node's behind-marker item states for a
replaced store in the removed form: a failed write of a removal reloads the
Persona with no marker.

*Amended in place 2026-10-20 (unmerged draft; dispatcher's resolution C1 and
owner ruling Q4' of 2026-10-16):* on `Behind` nothing is written — the
behind marker is not persisted (org-node keeps it out of the sealed store)
and the app's periodic verification derives it again each interval; on
`Removed` org-node's sealed change is written and the file kept, replacing
the removal of the file through `StoreChange::Forgotten` (LLR-amv4mb,
superseded by LLR-vj2nkb, which deletes the file and the keychain item when
REQ-98a3ty says).

*Amended in place 2026-10-28 (unmerged draft; review round 1 C-O12):*
`remove_store` is renamed `remove_removed_file` (LLR-vj2nkb). No behaviour
change.

**LLR-ce42ce**: when `read_state` or `finalised_block` fails, or the parse
refuses the state, `verify_organisation` returns `Verified::Unverified {
reason }`, calls no org-node operation and writes nothing;
`OrgIo::verification(target)` returns the last outcome of each target, and
`Unverified` until a later verification reads a state. A handle with the
chain not configured returns `Unverified` for every target.
satisfies: REQ-36gxmg, REQ-kh2uth
supersedes: LLR-kp95ty

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-2,
ARCHITECTURE Boundaries 2):* the reason of a failed read or refused parse,
`UnverifiedReason::Read`, holds the reader's `OrgNodeError`
(`Cause<OrgNodeError>`), not its text.

*Amended in place 2026-10-28 (unmerged draft; round 4 review, ARCHITECTURE
finding-1, Boundaries 2):* `UnverifiedReason::Read` holds org-io's own
`verify::ChainReadError`, not `OrgNodeError`: a failed read is org-io's I/O,
not an org-node refusal. Its text is the reader's error, and its `source()`
is the reader's `OrgNodeError`, typed.

*Checked 2026-10-20 (unmerged draft; owner ruling Q4' of 2026-10-16):*
unchanged. An unverified read reaches no org-node operation, so it neither
reports this node removed nor starts or stops an acknowledgement delivery,
and deletes nothing (REQ-kh2uth as amended).

**LLR-fzf2n3**: the time `verify_organisation` gives org-node's verification
as that of the verification is one reading of the handle's clock
(`OrgIo`'s `Clock`, whose production `now()` is `std::time::SystemTime::now()`
as whole seconds since the Unix epoch), taken once per call after
`read_state` returns and before org-node is called; org-io gives no other
time and keeps none of its own — the time that survives a restart is the one
org-node keeps in the sealed store (REQ-35s4eb). A clock reading before the
Unix epoch is `Verified::Unverified { reason: ClockBeforeEpoch }`, calling
no org-node operation and writing nothing.
satisfies: REQ-36gxmg, REQ-98a3ty, REQ-jqmpz5

**LLR-6vtbn6**: a build with the `test-support` feature offers
`OrgIo::with_clock_for_test(self, clock: Arc<dyn Clock + Send + Sync>) -> Self`,
which replaces the handle's clock (a settable test clock), and
`OrgIo::open_with_clock_for_test(data_dir, transport_mode, clock)`, which
opens as `open` does with that clock, so the deletions at open (LLR-upqj5g)
are judged against it; the acknowledgement delivery's waits (LLR-bv6wkz)
are measured against the handle's clock; a build without `test-support`
compiles no function that replaces the clock, and `OrgIo::open` and
`OrgIo::connect` take none.
satisfies: derived

**LLR-22se2q**: the ahead limit `verify_organisation` (and `verify_all`,
through it) gives org-node's verification — how long a received
provisional organisation one epoch ahead of the chain is kept unadopted
(the `ahead_limit: AheadLimit` of org-node's `OrgService::verify`) — is the one the handle
holds, the same for every verification of the handle: `OrgIo::connect`
sets it from `ChainSettings::ahead_limit: std::time::Duration`, given by
its caller (the app gives twice its verification interval), in whole
seconds, a fraction dropped. org-io holds no limit of its own and no
default: a handle with no chain (`open`, `not_configured`, a refused
connect's `into_not_configured`) holds none, and its verification calls
no org-node operation, writes nothing and is `Verified::Unverified {
reason: Read(_) }` with the not-configured message its chain read would
refuse with. A build with the `test-support` feature offers
`OrgIo::with_ahead_limit_for_test(self, Duration) -> Self`, which replaces
the handle's limit, and `ahead_limit_for_test(&self) -> Option<Duration>`,
which reads it; `OrgIo::for_test` and `open_for_test`, over a substitute
chain, hold `test_support::TEST_AHEAD_LIMIT` (1200 s) until replaced.
satisfies: REQ-36gxmg

*Added 2026-10-30 (unmerged draft; merge review MR3-N1, owner decision on
rule 4 of 2026-10-30):* the C+1 time limit is a parameter of org-node's
`OrgService::verify`, supplied by the app as twice its verification
interval so that at least two chain reads fall inside it whatever the
interval. org-io passed the stand-in `DEFAULT_AHEAD_LIMIT_SECS` (1200 s)
until this item; the stand-in is removed. The limit is a field of
`ChainSettings`, not a parameter of `verify_organisation` or `verify_all`,
so that every verification path (at mount, on the button, after a write,
on receipt, periodically) gives the one value the handle was connected
with, and the domain surface of LLR-4usfu2 is unchanged. It is a
`Duration`, not org-node's `AheadLimit`, which `org_io::node` does not
re-export (LLR-3zdw8v).

*Amended in place 2026-10-30 (unmerged draft; merge review round 4,
finding 3):* a `Duration` becomes the handle's limit through one
conversion, `ahead_limit_from(Duration) -> AheadLimit`: its whole seconds
(`Duration::as_secs`), a fraction dropped, not rounded, so a limit under a
second is `AheadLimit(0)` and `Duration::MAX` is `AheadLimit(u64::MAX)`.
`connect`, `for_test`, `open_for_test` and `with_ahead_limit_for_test` all
give their limit through it, and `test_support::TEST_AHEAD_LIMIT` is the
`Duration` of 1200 s the two substitute-chain constructors give it. The
conversion is crate-private: since `connect` needs a live chain, it is
tested through `with_ahead_limit_for_test` and `ahead_limit_for_test`,
which give and read back a limit through it.

**LLR-8w2z64**: `verify_organisation` gives org-node's verification a
`device_secrets` closure that reads, through `Keychain::for_signing_noting_refusal`
(LLR-677fqq), the keychain item of each Device key org-node names, and that
org-node calls only to sign the acknowledgements of this node's removal;
the secret read is moved into org-node's transient `DeviceSecret` and is
not kept by org-io after the call. When the keychain refuses, org-node's
refusal for want of the secret is reported as `Verified::Keychain(KeychainError)`
naming the Device key, nothing is written, the Persona is unchanged in
memory and on disk, and no acknowledgement delivery starts; the next
verification tries again.
satisfies: REQ-36gxmg

**LLR-4qyeut**: for `HeldTarget::Founding`, `verify_organisation` reads the
state at the proxy address stored with the provisional founding
(LLR-5tpdxw); a founding with no proxy stored is `Verified::Unverified {
reason: NoProxyYet }` with no read.
satisfies: REQ-36gxmg, REQ-qfnx94

*Checked 2026-10-21 (unmerged draft; design gap G3 of the plan,
dispatcher's decision):* unchanged. After a founding's included write the
app verifies it with `HeldTarget::Founding(founder)` (the app's
write-then-verify item), the target this item reads; `HeldTarget::Organisation(org_id)`
names a held Organisation, which a founding becomes only once a
verification adopts it, and from then on it is addressed so.

**LLR-kq7z58**: `OrgIo::verify_all() -> Vec<(HeldTarget, Verified)>`
verifies every Organisation, provisional founding and provisional Member the
node holds, one target at a time in store-file order, each as
`verify_organisation` does, and returns each outcome beside the target it
verified (`HeldTarget::Founding(founder)` for a provisional founding,
`HeldTarget::Organisation(org_id)` for a held Organisation and for a
provisional Member, by its Invite's Organisation); `Verified` itself names
no target. One target's failure or refusal does not stop the others.
satisfies: REQ-36gxmg

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* it returned `Vec<Verified>`, which says nothing of
which target each outcome is for, though the app must report each with its
Organisation and send or request for it; the outcome is now paired with its
`HeldTarget`, and `Verified` stays target-free.

**LLR-fp2cat**: for a provisional Member's Persona, `verify_organisation`
reads the state of the Invite's Organisation and hands it to org-node with
the Received provisional organisations of that store; org-io adopts nothing
itself, so an epoch whose record does not list the provisional Member's
keys is adopted by nothing and reported as org-node reports it.
satisfies: REQ-36gxmg

**LLR-zey4mg**: when an unknown write is recorded for the target,
`verify_organisation` reports and clears it as follows, after the outcome
above: `WriteOutcome::Adopted` when the verification adopted that
provisional organisation; `NotExecuted` only when the finalised block read
is later than the recorded `last_valid_block` and the state is at the
epoch the write was based on; `Deprecated`, with the Organisation reported
`Behind` (or `Adopted` for another of its provisional organisations), when
the chain's epoch is at or past the epoch the write expected with another
Membership root; otherwise `StillUnknown`, which keeps it. A failed read is
`StillUnknown`. A founding's unknown write has no window: it is settled
only as `Adopted` or `StillUnknown`. When an unknown write was recorded for
the target, the verification returns `Verified::Settled { outcome:
Box<Verified>, write: WriteOutcome }`, `outcome` the verification's own
outcome as above and `write` this settlement; with none recorded it returns
the outcome alone, never `Settled`.
satisfies: REQ-bm4ppg
supersedes: LLR-t98pk9

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* the report of the settlement is named: the
`Settled` variant carries the outcome and the `WriteOutcome` together, so
the app reads both from the one `Verified` (and handles `Settled` wherever
it matches one).

*Noted 2026-10-28 (unmerged draft; owner ruling A5 of 2026-10-28):*
`NotExecuted` is the glossary's term **Not executed** (`docs/CONTEXT.md`):
an Unknown write whose window has passed while the finalised chain is still
at its base epoch, so it may be proposed again against the same base; it is
distinct from **Deprecated**, where another Admin's epoch won. No item's
meaning changes.

**LLR-pbng7d**: org-io makes no discard call of its own: the provisional
organisations and provisional changes that leave the store are only those
org-node's verification returns as dropped in its store change (at or below
an adopted epoch, or under another Membership root at their epoch or later,
which no chain write can still execute), and org-io writes that change
unread; nothing is discarded on `Behind`, `Inconsistent`, `Unverified`, a
refused or unknown write, or a refused message.
satisfies: REQ-7jh4rh, REQ-kh2uth
supersedes: LLR-fptp57, LLR-36zp69

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q4' of
2026-10-16):* the same holds for what this node's removal deletes: the
deletion of everything but the acknowledgements is inside the sealed store
org-node's verification returns on `Removed`, which org-io writes unread
(LLR-zcw3xv); org-io deletes nothing of that Organisation itself but the
Persona's store file and keychain item when REQ-98a3ty says (LLR-vj2nkb).
On `Behind` nothing at all is written (C1).

**LLR-gaanm2**: `Verified::Adopted::discarded` lists, unchanged and in
org-node's order, every provisional change org-node reports the adopted
epoch left out, each with its kind (admission with its invite id and the
joiner's name, surname and Handle, or revocation with the Member id and
that Member's Handle in the adopted record).
satisfies: REQ-36gxmg, REQ-kh2uth

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* the app
re-proposes a discarded admission by its invite id and names the Member of
a discarded revocation by Handle (its notice and re-proposal items), so the
entry carries them: org-node's discarded change gives the invite id (or
Member id) and the Handle, the admission request org-node still keeps for
that invite gives the name and surname, and the adopted record, which still
lists a Member whose revocation was left out, gives that Member's Handle.

*Checked 2026-10-20 (unmerged draft; owner ruling Q4' of 2026-10-16):*
unchanged. An adopted epoch that removes this node is reported `Removed`,
not `Adopted`, so it lists no discarded change: the provisional changes go
with the rest of that Organisation's data in org-node's removal.

**LLR-3ame3g**: org-node's verification is called in `org-io/src` only from
`verify.rs`; no receive, write or send path adopts an epoch.
satisfies: REQ-36gxmg, REQ-cj9u3y

## The send path

**SDD-2bzgtp**: the send path: every message org-io sends — an
Organisation's current epoch, an admission request, an organisation
request, an answer to one — goes from the sending Persona's endpoint on a
new connection to a Device key, one recipient after another, each Device's
outcome reported from the one verdict byte it writes back, with no store
written or chain read. Code: `org-io/src/send.rs`.
traces: REQ-nactq6, REQ-8trkps, REQ-sjqxg2, REQ-z73ctw, REQ-wpq4fp, REQ-u894ty
supersedes: SDD-v2bq89 SDD-wr8m5n

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-4 sweep, ARCHITECTURE Boundaries 3):* the responsibility is restated in one sentence without "and", the detail it listed moved to the sentences after it. No behaviour change.

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* also
supersedes SDD-wr8m5n, the peer-address parse, whose requirement (REQ-n82q47)
and low-level requirement (LLR-2b2ghe) are retired (owner answer A2): no
address reaches org-io as text, and where each send's addresses come from is
this item's.

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q1 and answer A2
of 2026-10-16, dispatcher's resolution C8):* no operation takes a peer
address: each recipient is dialled at the endpoint addresses org-node keeps
for its Device key (REQ-ky5tyd), which that Device's own admission request
or organisation request carried, or, for a provisional Member's requests, at
the Invite's addresses for its issuer; no address is taken from an inbound
connection's source. The app asks for each adopted epoch's send once and
org-io offers no retry trigger of its own; a repeated call is still
harmless. The removed node's acknowledgements are sent by SDD-83un8g, over
this item's verdict read but from a direct-only endpoint.

**LLR-pxqdy4**: `OrgIo::send_organisation(org_id) -> Result<SendReport,
SendRefused>` (no address parameter) takes, under the node's lock,
org-node's messages for the adopted current epoch (every recipient Device
and its encoded message, REQ-ttv8k7) and, for each recipient, the endpoint
addresses org-node keeps for its Device key (REQ-ky5tyd), releases the
lock, and sends each in org-node's order to those addresses (LLR-34tfm8),
attempting every recipient whatever happened to the others;
`SendReport::per_device` holds each Device's outcome; the call writes no
store, starts no later send of its own, and repeated calls send the same
messages.
satisfies: REQ-nactq6, REQ-u894ty
supersedes: LLR-znhmh2

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q1 and answer A2
of 2026-10-16, dispatcher's resolution C8):* the `peer_addr` parameter is
gone; the addresses are those kept per Device key. Send once: org-io
schedules no retry and the app asks once per adopted epoch this node wrote;
a Device the send missed pulls the epoch (REQ-z73ctw).

**LLR-34tfm8**: in Networked mode each send dials the recipient by the
endpoint id equal to its Device key, with the addresses kept for that key
(or the Invite's, LLR-qa4nrt) as a hint ahead of discovery, and with none
when none is kept; in Loopback mode a send to a recipient refuses, sending
nothing to it, with `SendError::NoLoopbackAddress` when no address is kept
for that recipient and `SendError::AddressNotARecipient` when no kept
address's endpoint identity is the recipient's Device key, and otherwise
dials only the kept addresses whose identity is that key; each recipient is
judged on its own addresses, so one recipient's refusal leaves the others
attempted. A kept address is read in the text form of LLR-v9dkrv, its
endpoint identity the 64 hex characters before `@` and its socket the rest;
an entry of any other form names no identity and is not dialled.
satisfies: REQ-8trkps
supersedes: LLR-r9sh6k, LLR-jn5jeh, LLR-pw369n

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* the identity a kept address names is read from its
text form, `<64 lower-case hex endpoint id>@<socket>`.

*Amended in place 2026-10-20 (unmerged draft; owner answer A2 of
2026-10-16):* the address is no longer one the caller passes for one
addressed recipient, with every other recipient reported
`NotAttemptedInLoopback`; each recipient has the addresses kept for its key,
and the identity check applies to each. The variant `NotAttemptedInLoopback`
is removed. This item now also supersedes S4's Loopback dial at the
caller's full address (LLR-jn5jeh) and its refusal when the caller passes
none (LLR-pw369n), whose `send_committed` and caller-passed address are
gone.

**LLR-hx5c2y**: every send is made from the endpoint of the Persona whose
store the message came from (for `send_organisation` and
`request_organisation` the Persona holding `org_id`, for
`request_admission` the provisional Member's, for an answer the Persona
the request reached); when that endpoint cannot be had every recipient
reports `SendError::SenderEndpoint` and nothing is sent.
satisfies: REQ-nactq6, REQ-bupq5y

**LLR-zrgj8z**: `send_organisation` and `request_organisation` refuse with
`SendRefused::NotHeld`, sending nothing, for an Organisation no Persona
here holds, and return an empty `SendReport` when org-node returns no
recipient.
satisfies: REQ-nactq6, REQ-z73ctw

*Amended in place 2026-10-20 (unmerged draft; dispatcher's resolution C3 of
2026-10-16):* for `request_organisation`, a Persona holding a provisional
Member for `org_id` holds it (LLR-uf5cyy); `NotHeld` is for an
Organisation no Persona here holds as a current record or a provisional
Member. `send_organisation` still needs a current record.

**LLR-fwz3tb**: after writing a body and finishing its stream, each send
reads one byte back within `VERDICT_TIME_LIMIT` (10 s) and reports
`Delivered` for `0x00`, `Refused` for `0x01`, `AlreadyReceived` for `0x02`,
`InviteUsedByAnotherDevice` for `0x03`, and `NotDelivered { reason }` for a
failed dial or stream, no byte in time, or any other byte.
satisfies: REQ-sjqxg2, REQ-nactq6

## The requests a Device sends

**SDD-dhh3pw**: the requests a Device sends, each built by org-node from
the sending Persona's store and sent through the send path, changing nothing
in the store. They are a provisional Member's admission request, to the
Device its Invite names at the addresses the Invite gives, and an
organisation request, to each other Device of the current record (or, for a
provisional Member, to its Invite's issuer). Code: `org-io/src/send.rs`.
traces: REQ-sjqxg2, REQ-z73ctw, REQ-8trkps, REQ-2xgsbn

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-4 sweep, ARCHITECTURE Boundaries 3):* the responsibility is restated in one sentence without "and", the detail it listed moved to the sentences after it. No behaviour change.

*Amended in place 2026-10-20 (unmerged draft; owner answer A2 and the
dispatcher's resolution C3 of 2026-10-16):* every request carries the
sending Persona's own endpoint addresses in the current mode, as the Invite
does, so that the answer or the admitting epoch can be sent to them; and a
provisional Member, which holds no record, sends its organisation request
to the Device its Invite names as issuer, at the Invite's addresses.

**LLR-adgk9z**: `OrgIo::request_admission(org_id) ->
Result<Delivery, RequestError>` binds the provisional Member's
endpoint if it is not bound (LLR-d2pzzk), has org-node build the admission
request carrying the addresses that endpoint holds in the current mode (its
loopback sockets in Loopback mode, its direct addresses, possibly none, in
Networked mode, as LLR-v9dkrv takes them for an Invite), takes the Invite's
issuing DevicePublicKey and addresses from the provisional Member's store
for `org_id`, sends the request to that one Device (LLR-hx5c2y,
LLR-qa4nrt) and returns `Delivered`, `AlreadyReceived`,
`InviteUsedByAnotherDevice`, `Refused` or `NotDelivered { reason }`
(LLR-fwz3tb); it
writes nothing, and while the endpoint's addresses are unchanged it sends
the same body on every call. No provisional Member for `org_id` is
`RequestError::NoProvisionalMember`, and a bind that fails is
`RequestError::Bind`, each sending nothing.
satisfies: REQ-sjqxg2, REQ-2xgsbn
supersedes: LLR-9cb443, LLR-w4mhd4

*Amended in place 2026-10-20 (unmerged draft; owner answer A2 of
2026-10-16):* the request carries the sender's own endpoint addresses, so
the issuing Device keeps them for the new Device's key and sends the
admitting epoch there, with no pasted address; hence the bind before the
build and the `Bind` refusal.

*Amended in place 2026-10-28 (unmerged draft; round 2 review, C2-O5):*
the return type is `Delivery`, the one name of a recipient's outcome every
send path returns; the alias `AdmissionOutcome` is gone. No behaviour
change.

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* `Refused`
joins the outcomes: the receiving Device writes `0x01` for every refusal of
org-node's (an invite not outstanding, or a request whose DevicePublicKey is
not the connection's), and LLR-fwz3tb reports that byte as `Refused`; the
list had left it out. The verdict carries no reason. The Invite's issuer and
addresses are those org-node's `addresses_for(issuer)` returns in the
provisional Member's store.

**LLR-qa4nrt**: in Loopback mode `request_admission` dials only an Invite
address whose endpoint identity is the issuing Device's key, reporting
`NotDelivered { reason: AddressNotTheIssuer }` and sending nothing when
none is, and `NoLoopbackAddress` when the Invite gives none; in Networked
mode it dials the issuing Device's key with the Invite's direct addresses
as a hint. Each Invite address is read in the text form of LLR-v9dkrv; one
of any other form names no identity and is not dialled.
satisfies: REQ-8trkps, REQ-sjqxg2

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* the identity an Invite address names is read from
its text form, `<64 lower-case hex endpoint id>@<socket>`.

**LLR-uf5cyy**: `OrgIo::request_organisation(org_id) -> Result<SendReport,
SendRefused>` (no address parameter) binds the sending Persona's endpoint
if it is not bound, has org-node build an organisation request carrying the
addresses that endpoint holds in the current mode (as LLR-adgk9z), and
sends it — for a Persona holding a current record, to each Device that
record lists other than this node's own, at the addresses org-node keeps
for each key (LLR-34tfm8); for a Persona holding a provisional Member, to
the one Device its Invite names as issuer, at the Invite's addresses for it
(LLR-qa4nrt) — one after another whatever happened to the others, reports
each Device's outcome, writes nothing, and sends again on every call; a
bind that fails is `SendRefused::Bind`, sending nothing.
satisfies: REQ-z73ctw

*Amended in place 2026-10-20 (unmerged draft; owner answer A2 and the
dispatcher's resolution C3 of 2026-10-16):* the `peer_addr` parameter is
gone; the request carries the sender's addresses (REQ-wpq4fp sends the
answer to them); and a provisional Member, whose store has no record, asks
its Invite's issuer, which answers once its current record lists the
requester. The app calls this with no user step when a verification reports
the Organisation behind (the app's ledger, owner ruling Q2).

## The receive step (successor of SDD-aw3nka)

**SDD-2fc2cq**: the receive step: org-node's one receive path
(REQ-ncv4sc) applied to each body of every inbound connection whose
handshake completes, with the connection's authenticated DevicePublicKey,
with no chain read or adoption. For each body org-io writes the store change
org-node returns, writes one verdict byte back, sends an answer to an
organisation request through the send path, and reports the result to the
app. Code: `org-io/src/receiver.rs`,
`org-io/src/transport/` (the verdict and the framing, unchanged).
LLR-szhe7j and the framing items, defined under SDD-aw3nka, stay in force
beside it.
traces: REQ-xd5a2d, REQ-cj9u3y, REQ-wpq4fp, REQ-2yw5y3, REQ-qkrj55, REQ-8e57wm, REQ-szt6ea
supersedes: SDD-aw3nka

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-4 sweep, ARCHITECTURE Boundaries 3):* the responsibility is restated in one sentence without "and", the detail it listed moved to the sentences after it. No behaviour change.

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q3 and answer A2
of 2026-10-16):* an organisation request from a Device this node holds as
revoked is answered, as org-node decides, with that Device's revocation
notice alone; one from a Device no record has ever listed is still refused.
Every answer goes through the send path to the addresses the request
carried, which org-node keeps for the requester's key, never to the inbound
connection's source.

**LLR-kz9hj4**: `OrgIo::next_received(&self) -> Result<Received,
ReceiveError>` (no random source parameter) hands each body and its
authenticated sender to org-node's check of a received body, writes the
store change it returns (`write_store`, or nothing) before it reports, and
reads no chain and calls no verification; a refusal is
`ReceiveError::Refused` with nothing written, a failed write
`ReceiveError::Store`, the Persona reloaded (LLR-murjr6). A body org-node
kept in a Persona's store that is bound to no Organisation yet (a founding
before its proxy) is org-io's own refusal,
`ReceiveError::NoOrganisation { receiver }`, with `0x01` written: there is
no Organisation to report it under.
satisfies: REQ-cj9u3y, REQ-2yw5y3, REQ-tg9zrn
supersedes: LLR-cvyyr8, LLR-7pj5af

*Amended in place 2026-10-28 (unmerged draft; round 2 review, AR2-5):* that case was
returned as an `OrgNodeError::PersonaNotHeld` org-io built itself, which
both named org-node as the refuser and said the Persona was not held,
though it is held and only bound to no Organisation; it is org-io's own
`ReceiveError::NoOrganisation`.

**LLR-x36szv**: `Received` is one of `ProvisionalOrganisationReceived {
org_id, epoch, root, from }`, `AdmissionRequestKept { org_id, invite_id,
name, surname, preferred_handles, from, invitee_name }`, `OrganisationRequestAnswered {
org_id, requester, delivery }`, `RevocationNoticeKept { org_id, from }` and
`Acknowledged(..)`, each holding public
data only and joining the handle's return allowlist.
satisfies: REQ-cj9u3y, REQ-wpq4fp

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q3 of
2026-10-16):* `Received` gains `RevocationNoticeKept { org_id, from }`, the
report of a revocation notice org-node keeps for this node — pushed by a
Member's send or received in answer to this node's request — which only a
later verification acts on (REQ-36gxmg); a full record received in answer
is reported as `ProvisionalOrganisationReceived`. `OrganisationRequestAnswered`
gains `answer: AnswerKind` (`Record` or `RevocationNotice`), so the app is
told which answer went to the requester.

*Amended in place 2026-10-20 (unmerged draft; consistency pass):*
`AdmissionRequestKept` gains `invitee_name` and names `from` as the
request's DevicePublicKey, the fields the app's event carries; org-io takes
them from org-node's `admission_requests` for that invite (LLR-zkybw7),
since org-node's check returns the outcome alone. `answer: AnswerKind` is
withdrawn: org-node's check returns the answer as an encoded message with no
kind (its `OrganisationRequest { answer }`), org-io decodes no message body,
and the app announces no event for an answered request, so nothing needs it.

*Amended in place 2026-10-21 (unmerged draft; owner decision of
2026-10-20):* org-node's `Checked::AlreadyHeld` — a provisional organisation
whose identity this Persona already keeps, so a duplicate of one already
reported as `ProvisionalOrganisationReceived` — maps to no `Received`:
org-node returns no store change, so `next_received` writes nothing, writes
`Verdict::Accepted` (`0x00`, the message was not refused, LLR-hhgdk7),
reports nothing to the app and goes on to the next body. REQ-cj9u3y reports
each thing kept, and a duplicate keeps nothing new; the verification the
first report triggered already covers it, so silently accepting it is the
reading the in-force REQs allow. `RevocationNoticeKept` is reported as
REQ-cj9u3y (as amended 2026-10-21) states, and the app verifies at once on
it (the app's ledger).

**LLR-j467e9**: the accept path takes every inbound connection whose
handshake completes, direct or relayed, applies no allow-list of remote ids
at accept time, and passes each body with the connection's authenticated
DevicePublicKey to LLR-kz9hj4's check, so that from a Device no held record
lists only an acknowledgement or an organisation request from a Device held
as revoked and an admission request for an invite this store holds as
outstanding can be acted on; every other message from it is refused by that
check before any store write, and no chain is read on this path at all.
satisfies: REQ-xd5a2d
supersedes: LLR-wvgec4

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q3 of
2026-10-16):* an organisation request from a Device held as revoked joins
the messages such a Device may send; org-node answers it with the notice
alone (LLR-898z24, LLR-236rf9).

**LLR-hhgdk7**: for each inbound message the accept task keeps the stream's
send half until the receive step answers through the message's responder,
then writes exactly one byte and finishes the stream: `Verdict::Accepted`
(`0x00`) when org-node accepted the message and its store change (if any)
is written, `Verdict::AlreadyReceived` (`0x02`) and
`Verdict::InviteUsedByAnotherDevice` (`0x03`) for those two verdicts of
org-node's admission-request rule, and `Verdict::Refused` (`0x01`) for every
other refusal or failure; a dropped responder writes nothing, and no other
byte is written on an inbound connection by any code in `org-io/src`.
satisfies: REQ-xd5a2d
supersedes: LLR-unqfj5

**LLR-zkybw7**: for an admission request org-node's check returns its
outcome (`AdmissionRequestOutcome`) and, for the first request kept, the
store change; org-io writes that change before it writes `0x00` and reports `AdmissionRequestKept`; for
`AlreadyReceived` and `InviteUsedByAnotherDevice` it writes nothing,
reports nothing to the app, and writes `0x02` or `0x03`.
satisfies: REQ-xd5a2d, REQ-2yw5y3
supersedes: LLR-gha5f6

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* org-node's
check returns `AdmissionRequest { outcome }` and nothing else, so the
`AdmissionRequestKept` report is filled, after the write, from the entry
org-node's `admission_requests` returns for that invite id, with its invitee
name (LLR-x36szv, LLR-dm2h8u). A refusal of org-node's — an invite not
outstanding, or a request whose DevicePublicKey is not the connection's
(`AdmissionKeyNotSender`) — is `0x01`, nothing written, as LLR-hhgdk7
states.

*Amended in place 2026-10-28 (unmerged draft; round 2 review, C2-N1):* the
admission request's result is named `AdmissionRequestOutcome` (field
`outcome`), not a verdict: the glossary's Verdict is the byte org-io writes
back, which this outcome only chooses.

**LLR-898z24**: when org-node's check answers an organisation request (a
recipient Device key, an encoded answer — the full record for a listed
Device, the revocation notice alone for a Device held as revoked — and the
store change keeping the addresses the request carried for that key), the
receive step writes that store change, then the verdict `0x00` on the
inbound stream, releases the node's lock and sends the answer through the
send path on a new connection to that Device key, at the addresses org-node
keeps for it (LLR-34tfm8: in Loopback mode only an address whose identity
is that key), and reports `OrganisationRequestAnswered` with the send's
outcome; the answer is never written on the inbound
connection and no address is taken from the inbound connection's source. A
failed store write is `ReceiveError::Store` with `0x01` written and no
answer sent.
satisfies: REQ-wpq4fp

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q3 and answer A2
of 2026-10-16):* the answer goes to the addresses the request carried, kept
by org-node for the requester's key, not to the inbound connection's source
address; and a revoked Device's request is answered with its notice.

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* "with the
answer's kind" is withdrawn (LLR-x36szv): org-node returns the answer
without one. The addresses are those org-node's `addresses_for(peer)`
returns for the requester's key, which its check has just set from the
request.

**LLR-236rf9**: an organisation request from a Device no record held here
has ever listed — neither a current Member's Device nor a Device held as
revoked — is refused by org-node's check; org-io writes `0x01`, writes no
store, sends no answer and reports nothing to the app.
satisfies: REQ-xd5a2d

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q3 of
2026-10-16):* a Device held as revoked is no longer refused: org-node
answers it with its revocation notice (LLR-898z24); the refusal is now for
a Device never listed.

**LLR-eeh4pq**: `next_received` waits for the next body on the accept
channel holding neither the node's lock nor any store file, takes the lock
only around org-node's check and the store write that follows, and holds
none while it sends an answer; every other `OrgIo` method that takes the
lock completes while `next_received` waits.
satisfies: REQ-qkrj55
supersedes: LLR-9arvqj

## Acting on this node's removal

*Added 2026-10-20 (unmerged draft; owner ruling Q4' of 2026-10-16 and the
rulings of 2026-10-20).* S3b-io part 2's acknowledgement delivery, which
no item in force stated, is stated here for the store org-node now keeps on
this node's removal.

**SDD-83un8g**: the removal acknowledgements: after verification reports
this node removed from an Organisation, sending each acknowledgement
org-node keeps for it directly to its recipient from the removed Persona's
own endpoint identity, on a backoff schedule measured by a clock org-io
reads, having org-node delete each one its recipient accepts, and deleting
the Persona's store file and keychain item as soon as none is left, or 7
days after the kept verification time, or at start when either already
holds; with no command and no app confirmation. Code:
`org-io/src/acknowledge.rs` (the delivery task, the schedule, the deletion),
`org-io/src/clock.rs` (`Clock`, `SystemClock`), `store_file::mark_removed`
and `store_file::remove_store`, `keychain::delete_device_secret`, and the
direct-only bind in
`endpoints.rs`. It reaches org-node only through its exported listing
(REQ-35s4eb) and deletion (REQ-ayyt5f) of kept acknowledgements. SOUP:
`tokio` (`time`, `spawn`), `keyring` (`delete_credential`).
traces: REQ-k6kcgd, REQ-sd7ayv, REQ-98a3ty, REQ-jqmpz5, REQ-bg5whd, REQ-kh2uth, REQ-36gxmg
superseded-by: SDD-3my66m, SDD-wyn7b6

*Superseded 2026-10-28 (unmerged draft; review round 1 A8, ARCHITECTURE
Boundaries 3):* the item stated two responsibilities, the acknowledgements'
delivery and the removed data's retention and deletion, so a change to the
delivery's schedule and one to the retention deadline both edited it and
its one module. It is split into SDD-3my66m (the delivery,
`acknowledge.rs`) and SDD-wyn7b6 (the retention and deletion, the new
`retention.rs`), which share the handle's delivery-task registry; the
deletion rule is one pure function (`retention::deletion_due`) used by open,
resume and the delivery task. Its LLRs are re-pointed below: LLR-zzsqj4,
LLR-sn5ded, LLR-cqr26z, LLR-3w22wc, LLR-rav7dr, LLR-6ux6yw, LLR-xwz3nu,
LLR-bv6wkz and LLR-5rxqr8 are SDD-3my66m's; LLR-vj2nkb, LLR-n649th,
LLR-upqj5g and LLR-3fxxpp are SDD-wyn7b6's. No behaviour changes.

*Amended in place 2026-10-20 (unmerged draft; consistency pass, dispatcher's
decision on the deletion order):* the code gains `store_file::mark_removed`;
the Persona's keychain item is deleted before its store file (LLR-vj2nkb).

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* the handle's `OrgService` is held as
`Arc<tokio::sync::Mutex<OrgService>>`, so that each delivery task, which
outlives the call that started it, shares the one service with the handle
and reaches org-node's listing and deletion of kept acknowledgements under
the same lock as every other operation; the handle's operations are
otherwise unchanged.

### The acknowledgements' delivery (successor of SDD-83un8g)

**SDD-3my66m**: the removal acknowledgements' delivery: after verification
reports this node removed from an Organisation, sending each acknowledgement
org-node keeps for it directly to its recipient from the removed Persona's
own endpoint identity, on a backoff schedule measured by a clock org-io
reads, having org-node delete each one its recipient accepts, until the
removed data is due for deletion (SDD-wyn7b6); with neither a command nor an
app confirmation. Code: `org-io/src/acknowledge.rs` (the delivery task, the
schedule, the report, the resume at start, the registry of running
tasks the deletion shares, and the Device keys of the removed stores the
handle holds — those `OrgIo::open` found, and each a delivery's start
records — which the report and the resume read instead of listing the data
directory), `org-io/src/clock.rs` (`Clock`,
`SystemClock`), and the direct-only bind in `endpoints.rs`. It reaches
org-node only through its exported listing (REQ-35s4eb) and deletion
(REQ-ayyt5f) of kept acknowledgements. SOUP: `tokio` (`time`, `spawn`).
traces: REQ-k6kcgd, REQ-sd7ayv, REQ-jqmpz5, REQ-bg5whd, REQ-kh2uth, REQ-36gxmg, REQ-kn8zg6
supersedes: SDD-83un8g

*Traces amended 2026-11-03 (change `worktree-org-io-publish-2`, task T13b;
owner ruling Q1 (c) of 2026-11-03, as recommended):* REQ-kn8zg6 joins the line, the dial by key of
LLR-u2j6u8.

*Note 2026-11-04 (change `worktree-org-io-publish-2`, task q1r; owner
ruling Q1' of 2026-11-04):* "directly" and "the direct-only bind" hold for
every dial that has an address; the one dial by key alone in Networked mode
goes from an endpoint of the Persona's identity bound for it with iroh's
relay server and address lookup, closed after the dial (LLR-spckm6, which
supersedes LLR-cqr26z). No other Device forwards it (ruling D7). The code is
`acknowledge.rs`'s `dial_by_key`, binding the handle's bind plan (`Endpoints::bind_plan`).

*Amended in place 2026-11-09 (change `worktree-org-io-publish-2`, merge
review round 1, MR1-O4; a wording change, the ID is kept):* the note of
2026-11-04 above named `Endpoints::plan`, which task o2 renamed
`Endpoints::bind_plan` (LLR-spckm6's o2 note); the note now names
`Endpoints::bind_plan`.

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-4 sweep, ARCHITECTURE Boundaries 3):* the responsibility is restated in one sentence without "and", the detail it listed moved to the sentences after it. No behaviour change.

*Added 2026-10-28 (unmerged draft; review round 1 A8):* SDD-83un8g's
delivery half. LLR-zzsqj4, LLR-sn5ded, LLR-cqr26z, LLR-3w22wc, LLR-rav7dr,
LLR-6ux6yw, LLR-xwz3nu, LLR-bv6wkz and LLR-5rxqr8 are its LLRs. With owner
ruling C-O6 of 2026-10-27, the handle remembers the removed stores' keys:
the resume at start and the report used to list the data directory again,
and a listing that failed made both an empty list, so a kept removal was
neither resumed nor reported.

**LLR-zzsqj4**: once `verify_organisation` has written the sealed store of
a `Removed` outcome (LLR-zcw3xv), it takes org-node's list of the
acknowledgements kept for that Organisation (REQ-35s4eb); when the list is
empty, which happens only on a resume at start (see below), it deletes the
Persona at once (LLR-n649th); otherwise it starts the
Persona's delivery task (`tokio::spawn`), whose first round sends every
kept acknowledgement at once, and returns `Removed` without awaiting it. At
most one delivery task runs per Persona: a start for a Persona whose task
runs starts none.
satisfies: REQ-k6kcgd

*Amended in place 2026-10-28 (unmerged draft; round 3 review, T3-O8; owner
ruling of 2026-10-28):* the list cannot be empty at the verification that
writes the `Removed` outcome: the Admin whose epoch removes this Device is
always another Device the record lists, so org-node always keeps at least
one acknowledgement, the one for that Admin. The empty-list branch is
reached only on a resume at start (LLR-5rxqr8), after the last
acknowledgement was accepted and the deletion that followed failed; that
resume's test (`a_delivery_that_starts_with_none_kept_deletes_the_persona_at_once`)
is this item's abnormal-case test. "At most one delivery task runs per
Persona" is seen at the recipient: a start while the task runs sends no
second acknowledgement.

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* org-node's
list is `kept_acknowledgements(device_key)`, which returns `Some(
KeptAcknowledgements { org_id, verified_at, acknowledgements })` for a
removed store, each acknowledgement an `OutgoingMessage` with the
recipient's kept addresses, and `None` for a store not removed; its deletion
of an accepted one (LLR-6ux6yw) is `acknowledgement_accepted(rng,
device_key, recipient)`, which returns the number still kept. A removed
store refuses every other org-node operation but `drop_store`
(`OrganisationRemoved`): a message received for it is refused (`0x01`), and
a method naming its Organisation returns that refusal; `verify_all`
(LLR-kq7z58) holds it as no Organisation, founding or provisional Member, so
it does not verify it.

**LLR-sn5ded**: org-io learns the kept acknowledgements, their recipients'
DevicePublicKeys and endpoint addresses and the verification time only from
org-node's list (REQ-35s4eb), which changes nothing, and removes an
acknowledgement only through org-node's deletion of an accepted one
(REQ-ayyt5f); it decodes no acknowledgement, holds its encoded bytes only
for the round that sends them, and writes them to no file but through the
sealed store org-node returns.
satisfies: REQ-bg5whd, REQ-k6kcgd

*Note 2026-10-28 (unmerged draft; round 4 TEST review, finding-3;
robustness, N/A):* the item states where org-io's knowledge of an
acknowledgement comes from and who removes one, a property of the source
that takes no input of its own. Its test is a scan of `org-io/src` with
one delivery showing org-node's bytes unchanged at the recipient (whose
answer is already a refusal). The inputs a delivery can meet — a refused
or other verdict, no answer, a failed write of the acceptance, a list that
holds none — are the inputs of the items it constrains and are tested
there, abnormal cases included: LLR-xwz3nu, LLR-6ux6yw, LLR-3w22wc and
LLR-n649th.

**LLR-cqr26z**: every acknowledgement is sent from the endpoint whose
identity is the removed Persona's own Device key, bound from that Persona's
keychain item (LLR-cns6q6), never from a fresh identity; before its first
round the delivery closes that Persona's endpoint if it is bound with a
relay and binds it again with the relay disabled and no address lookup
(`BindPlan::Direct`: Loopback mode's loopback sockets, or Networked mode's
direct sockets), and each dial is to an address holding the recipient's
Device key and only one of its kept direct addresses, with no relay URL. A
bind that fails is the round's outcome for every recipient
(`NotDelivered { reason: SenderEndpoint }`), and the next round binds
again.
satisfies: REQ-k6kcgd
superseded-by: LLR-spckm6

*Superseded 2026-11-04 (change `worktree-org-io-publish-2`, task q1r;
owner ruling Q1' of 2026-11-04) by LLR-spckm6 (`2026-11-02-publish-2.md`):*
every clause stands; in Networked mode the one dial by a recipient's key
alone (REQ-kn8zg6, LLR-u2j6u8) goes from an endpoint of the same identity
bound with iroh's relay server and address lookup, since the direct-only
endpoint can never find a key; every dial that has an address keeps the
direct-only bind.

**LLR-3w22wc**: in a round, the delivery takes the kept acknowledgements in
org-node's order and, for each recipient, dials its kept addresses one at a
time in the order kept — in Loopback mode only those whose endpoint
identity is the recipient's key (LLR-34tfm8) — writing the acknowledgement
and reading the verdict (LLR-fwz3tb); it stops at the first `Accepted`, and
moves to the next address on `Refused`, on any other verdict byte and on
`NotDelivered` (a failed dial, a failed stream, no byte within 10 s); the
recipient's outcome for the round is `Accepted` or the last address's
result, `NotDelivered { reason: NoAddress }` when no address may be dialled;
one recipient's outcome never stops the others. The node's lock is not held
while a dial or a verdict read is in progress.
satisfies: REQ-k6kcgd
superseded-by: LLR-u2j6u8

*Superseded 2026-11-03 (change `worktree-org-io-publish-2`, task T13b;
owner ruling Q1 (c) of 2026-11-03, as recommended) by LLR-u2j6u8 (`2026-11-02-publish-2.md`):* in
Networked mode a recipient with no kept address that may be dialled is
dialled once by its key alone instead of reported `NoAddress`
(REQ-kn8zg6); Loopback mode is unchanged.

**LLR-rav7dr**: `OrgIo::acknowledgement_report(org_id) ->
Option<AcknowledgementReport>` returns, for the removed Persona bound to
`org_id`, each recipient DevicePublicKey whose acknowledgement is still
kept with its last round's outcome (`NotYetSent`, `Refused`,
`NotDelivered { reason }`), the time of the next round, the deletion
deadline (the kept verification time plus 7 days), the last deletion
failure if any (LLR-3fxxpp), and the last round's failure if that round's
read of org-node's list of the kept acknowledgements failed, or the
handle's clock read before the Unix epoch (`last_round_failure`, naming the
cause; LLR-bv6wkz), cleared by the next round that reads it. It finds the removed Persona among the removed stores
the handle remembers (LLR-5rxqr8), listing no directory; it returns `None` for an Organisation no
removed Persona holds, and after the Persona is deleted, except while a
failed keychain deletion or file removal of LLR-3fxxpp is still being
retried, when it returns a report with no recipient and that failure, and
except for a `removed-` file kept beside a live store of the
Organisation's Persona (LLR-upqj5g), when it returns a report with no
recipient, no deadline and `DeletionFailure::LeftoverKept`; it
writes nothing and sends nothing.
satisfies: REQ-k6kcgd

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-2,
ARCHITECTURE Boundaries 2):* the outcomes and failures are typed, their
texts unchanged: `NotDelivered { reason }` holds the send's `SendError`
(`Cause<SendError>`); `last_deletion_failure` is a `DeletionFailure` (the
store file's or the keychain item's step with its error, org-node's refused
list, a clock before the Unix epoch at open, or a `removed-` file kept
beside a live store); `last_round_failure` a `RoundFailure` (org-node's
refused list, or a clock before the Unix epoch).

*Amended in place 2026-10-28 (unmerged draft; round 3 review, T3-O2; owner
ruling of 2026-10-28):* untested clause: `last_round_failure` for a round
whose read of org-node's list of the kept acknowledgements failed. No store
org-node opens makes `kept_acknowledgements` refuse with anything but
`PersonaNotHeld`, so the failure can be produced only by a double of
org-node, which the project owns (TEST_GUIDELINES Doubles 1); its test and
the test-build fault at org-io's read (`fail_acknowledgement_listing_for_test`)
are removed. The report-and-retry code stays as it is. The clause's other
half, the clock reading before the Unix epoch, is tested.

*Amended in place 2026-10-28 (unmerged draft; round 2 review, C2-O1):*
`last_round_failure` also names a clock reading before the Unix epoch,
which LLR-bv6wkz now makes a round's failure; and `OrgIo::open` reports
such a reading as each kept removal's `last_deletion_failure`
(LLR-upqj5g).

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* "a failed
keychain deletion" becomes "a failed keychain deletion or file removal", the
two steps LLR-3fxxpp retries after the Persona is dropped now that the item
goes before the file.

*Amended in place 2026-10-28 (unmerged draft; review round 1, owner ruling
C-O3 of 2026-10-27):* the report gains `last_round_failure`. A refusal of
org-node's list other than `PersonaNotHeld` used to end the delivery
silently (the round counted it as none kept, and the task went on to the
deletion); it is now that round's failure, shown here and retried at the
next due time (LLR-bv6wkz).

*Amended in place 2026-10-28 (unmerged draft; review round 1, owner ruling
C-O6 of 2026-10-27):* the report listed the data directory to find a
removal whose delivery had not started, and a listing that failed made it
`None`; it now reads the removed stores' keys the handle remembers.

*Amended in place 2026-10-30 (unmerged draft; merge review round 1,
finding-1):* the deadline is `Option<u64>`. A report given while only a
failed keychain deletion or file removal is retried, its store dropped,
still gives the removal's deadline (the kept verification time plus 7
days), kept from when the store was held: by the delivery task, or by
`OrgIo::open` with the failure it found. It used to give 0, a deadline at
the Unix epoch. The deadline is `None` only when no removal is kept to have
one: a `removed-` file kept beside a live store that keeps no removal
(`DeletionFailure::LeftoverKept`, LLR-upqj5g).

*Amended in place 2026-10-30 (unmerged draft; merge review round 1,
finding-2):* the outcomes gain `AcceptedNotRecorded { reason }`: the
recipient answered `Accepted` but the acceptance was not recorded, because
org-node refused to delete the acknowledgement or the store write failed
(LLR-6ux6yw); `reason` is an `AcceptanceFailure` naming which, with its
error. The acknowledgement stays kept. Such a round used to record no
outcome, so the report kept giving the recipient's outcome from an earlier
round, or `NotYetSent`, though the last round reached it.

*Amended in place 2026-10-30 (unmerged draft; merge review round 2,
finding-4):* the `None` clause names its second exception, which the
amendment on the deadline above and LLR-upqj5g already described: a
`removed-` file kept beside a live store of the Organisation's Persona,
which keeps no removal, gives `Some` report with no recipient, no deadline
and `DeletionFailure::LeftoverKept`, not `None`. The code is unchanged. And
(merge review round 2, finding-2) the deadline is also kept when
`start_receiving` finds a removal whose deadline passed after `OrgIo::open`
and retries its deletion with no round: a failure after the rename still
reports it.

**LLR-6ux6yw**: when a recipient answers an acknowledgement `Accepted`, the
delivery, under the node's lock, has org-node delete that acknowledgement
with its recipient's key and addresses (REQ-ayyt5f) and writes the sealed
store it returns (`write_store`) before the next recipient is sent to; when
org-node reports no acknowledgement left for that Organisation it deletes
the Persona at once (LLR-n649th). A failed write reloads the Persona from
its file (LLR-murjr6), so the acknowledgement stays kept and is sent again
at the next round; a refusal of org-node's (no such acknowledgement) writes
nothing.
satisfies: REQ-sd7ayv

*Amended in place 2026-10-28 (unmerged draft; round 5 TEST review, Notes):*
untested clause: "a refusal of org-node's (no such acknowledgement) writes
nothing". The delivery asks org-node to delete only an acknowledgement it
has just read from org-node's list under the same Persona, and nothing else
deletes one while the task runs, so org-node's `AcknowledgementNotKept`
can be produced only by a double of org-node, which the project owns
(TEST_GUIDELINES Doubles 1), as the notes on LLR-rav7dr, LLR-bv6wkz,
LLR-3fxxpp and LLR-4yte89 give. The abnormal case tested is the failed
write and its reload.

*Amended in place 2026-10-30 (unmerged draft; merge review round 1,
finding-2):* a refusal or a failed write is that recipient's outcome for
the round, `AcceptedNotRecorded { reason }` in the report (LLR-rav7dr).

**LLR-xwz3nu**: on any answer to an acknowledgement but `Accepted` —
`Refused`, `AlreadyReceived`, `InviteUsedByAnotherDevice`, another byte —
and on `NotDelivered`, the delivery calls no org-node operation, writes no
store and deletes nothing: the acknowledgement, its recipient's key and its
addresses stay kept for the next round.
satisfies: REQ-sd7ayv, REQ-kh2uth

**LLR-bv6wkz**: with T the verification time org-node keeps, the delivery's
rounds are due at T, T + 1 min, T + 5 min, T + 30 min, and then T + 30 min +
k hours for k = 1, 2, …, until no acknowledgement is kept or the deadline
T + 7 days is reached, whichever is first; a round sends only the
acknowledgements still kept; the task waits with `tokio::time::sleep` for the
time between the handle's clock's `now()` and the next due time, reads the
clock again on waking, and runs a round only when `now()` is at or past it;
when one waking finds several due times passed it runs one round and then
waits for the first due time after `now()`, so missed rounds are not caught
up. Each round reads org-node's list of the kept acknowledgements first:
when org-node refuses it with `PersonaNotHeld` (the Persona's store is gone)
the task ends; when the read fails with anything else, the round sends
nothing and deletes nothing, the failure is that round's failure in the
report (LLR-rav7dr), and the next round runs at the next due time. A clock
reading before the Unix epoch is never taken as time 0: on it the task
judges no deletion and runs no round, the condition is that round's failure
in the report (naming `ClockBeforeEpoch`'s cause, with no next round), and
the round is tried again one retry period (an hour) later by the runtime's
timer, or when the task is woken, reading the clock again; a wait that
reads it before the epoch ends at once, and the task reports it so.
satisfies: REQ-jqmpz5

*Amended in place 2026-10-28 (unmerged draft; round 3 review, T3-O2; owner
ruling of 2026-10-28):* untested clause: "when the read fails with anything
else, the round sends nothing and deletes nothing, the failure is that
round's failure in the report (LLR-rav7dr), and the next round runs at the
next due time". No store org-node opens makes its list refuse with anything
but `PersonaNotHeld`, so that failure can be produced only by a double of
org-node, which the project owns (TEST_GUIDELINES Doubles 1); the test-build
fault at org-io's read that the note above names, and its test, are
removed. The code stays as it is.

*Amended in place 2026-10-28 (unmerged draft; round 2 review, C2-O1 and
T2-O2; dispatcher's ruling of 2026-10-28, consistent with the owner's
report-and-retry ruling of 2026-10-27):* a clock reading before the Unix
epoch was taken as time 0, so no deletion was ever due and a wait slept
about 1.7e9 s with nothing reported; it is now that round's failure,
reported and retried. The list's read "fails" rather than "is refused":
besides org-node's own refusals, a test build injects a content-free fault
at org-io's read (`fail_acknowledgement_listing_for_test`), which imitates
no org-node refusal, since no store org-node opens makes its list refuse.
(Withdrawn by the amendment of round 3 below: the fault and its test are
gone.)

*Amended in place 2026-10-28 (unmerged draft; review round 1, owner ruling
C-O3 of 2026-10-27):* the item said nothing of a refused list, and the code
treated any refusal as no acknowledgement kept, which ended the delivery
and went on to the deletion. Only `PersonaNotHeld` ends it now; any other
refusal is reported and retried at the next due time.

**LLR-5rxqr8**: `OrgIo::start_receiving` starts, for every Persona whose
store org-node reports keeping acknowledgements of a removal (REQ-35s4eb)
whose deadline the handle's clock has not reached, that Persona's delivery
task: one round at once, then the rounds of LLR-bv6wkz due after `now()`,
anchored at the kept verification time and not at the start; it starts
none for a Persona with no kept removal, and none twice (LLR-zzsqj4). The
Personas it considers are those whose removed stores the handle remembers:
each `OrgIo::open` found keeping a removal after its deletions
(LLR-upqj5g; `open` refuses when its listing fails), and each a delivery's
start recorded before it returned; `start_receiving` lists no directory for
them, so a listing that fails then resumes none fewer. With the clock
reading before the Unix epoch, no deadline is judged: each kept removal's
task is started, and it reports the condition as its round's failure
(LLR-bv6wkz).
satisfies: REQ-jqmpz5, REQ-k6kcgd

*Amended in place 2026-10-28 (unmerged draft; round 2 review, C2-O1):* the
resume judged the deadline on a reading before the epoch taken as time 0;
it now judges none and the started task reports the condition.

*Amended in place 2026-10-28 (unmerged draft; review round 1, owner ruling
C-O6 of 2026-10-27):* `start_receiving` listed the data directory again
and resumed none when that listing failed; it now reads the removed stores'
keys the handle remembers.

### The removed data's retention and deletion (successor of SDD-83un8g)

**SDD-wyn7b6**: the removed Persona's data lifetime: the store file of a
Persona this node was removed with, with its keychain item, kept while its
acknowledgements are delivered, then deleted by one rule (7 days after the
kept verification time, or at once when no acknowledgement is left), from
the delivery task, at open or at start, a failed step retried hourly. Code: `org-io/src/retention.rs` (`deletion_due`, the deletion,
the leftover `removed-` files at open and their hourly retry),
`store_file::mark_removed` and `store_file::remove_removed_file`,
`keychain::delete_device_secret`. It reaches org-node only through its
listing of kept acknowledgements (REQ-35s4eb), read for the verification
time and the count, and `drop_store`. SOUP: `tokio` (`spawn`), `keyring`
(`delete_credential`).
traces: REQ-98a3ty, REQ-kh2uth
supersedes: SDD-83un8g

*Amended in place 2026-10-28 (unmerged draft; round 3 review, AR3-4 sweep, ARCHITECTURE Boundaries 3):* the responsibility is restated in one sentence without "and", the detail it listed moved to the sentences after it. No behaviour change.

*Added 2026-10-28 (unmerged draft; review round 1 A7, A8):* SDD-83un8g's
deletion half. The rule, written inline three times there, is one pure
function, `deletion_due(left, verified_at, now)`: `Expired` once `now`
reaches `verified_at` + 604 800 s, else `NoneKept` when `left` is 0, else
none. Open deletes on either; the delivery task deletes on either; the
resume at start retries an `Expired` one's deletion hourly and starts the
delivery of a `NoneKept` one, whose task deletes it at once. LLR-vj2nkb,
LLR-n649th, LLR-upqj5g and LLR-3fxxpp are its LLRs.

**LLR-vj2nkb**: when the handle's clock reaches the kept verification time
plus 7 days (604 800 s), whatever was delivered, the delivery task, under
the node's lock, stops its rounds, closes the Persona's endpoint, and
deletes the Persona in this order: it renames the store file
`persona-<hex>.store` to `removed-<hex>.store` (`store_file::mark_removed`,
which on Unix syncs the directory), a name the Persona listing (LLR-nt8dja)
does not list; then deletes the Persona's keychain item
(`keychain::delete_device_secret`, keyring's `delete_credential`; an item
already missing counts as deleted); then removes the renamed file
(`remove_removed_file`, which on Unix syncs the directory); then drops the Persona
from the service (`drop_store`). The keychain item goes before the file, so
a file left behind is sealed under a Device secret that no longer exists and
cannot be opened, and its name records whose it was; the next listing holds
no Persona file for that key, and no other Persona's file or item is
touched.
satisfies: REQ-98a3ty, REQ-kh2uth
supersedes: LLR-amv4mb

*Amended in place 2026-10-28 (unmerged draft; review round 1 C-O12):* the
renamed file's removal is `store_file::remove_removed_file` (was
`remove_store`), named for the only file it removes. No behaviour change.

*Amended in place 2026-10-20 (unmerged draft; consistency pass, dispatcher's
decision on the deletion order):* the keychain item is deleted first, then
the store file; it read "removes the Persona's store file, then deletes the
Persona's keychain item … so that no store file is left whose item is
gone", which left a keychain item no file named when the keychain deletion
failed (the limit LLR-3fxxpp reported). The rename to `removed-<hex>.store`
comes first so that a leftover file is never a Persona store file whose
item is missing: opening still refuses such a file and leaves it untouched
(LLR-uzax55, RC-hd5des), while a `removed-` file is deleted at start
(LLR-upqj5g).

*Amended in place 2026-10-21 (unmerged draft; owner decision of
2026-10-20):* the owner confirmed this design on 2026-10-20 — the rename to
`removed-<hex>.store`, then the keychain item, then the file; leftover
`removed-` files finished at start; a `persona-*.store` file without its
item still refused (RC-hd5des). No behaviour change.

**LLR-n649th**: the Persona is deleted as LLR-vj2nkb states, in its order
(the store file marked removed, then the keychain item, then the file), at
once and without waiting for the deadline, when org-node's list holds no
acknowledgement for that Organisation at the start of a delivery
(LLR-zzsqj4) or at start (LLR-upqj5g), or when its deletion of an accepted
acknowledgement reports none left (LLR-6ux6yw).
satisfies: REQ-98a3ty

*Amended in place 2026-10-20 (unmerged draft; consistency pass, dispatcher's
decision on the deletion order):* names the order, keychain item before
store file, which every early deletion follows as the deadline's does.

**LLR-upqj5g**: `OrgIo::open`, before it lists the Persona store files,
finishes every deletion a `removed-<hex>.store` file in the data directory
records: it deletes the keychain item of the Device key `<hex>` names (an
item already missing counts as deleted), then removes the file, unsealing
nothing — except that a `removed-<hex>.store` beside a live
`persona-<hex>.store` of the same key is left untouched, with that key's
keychain item, and reported against the Persona's Organisation; and, after unsealing every store, it deletes as LLR-vj2nkb states,
before it returns, each Persona whose store org-node reports keeping a
removal whose verification time plus 7 days the handle's clock has reached,
or keeping no acknowledgement of it. The handle it returns holds none of
them, and a Persona holding no kept removal is not touched; a
`persona-<hex>.store` file whose keychain item is missing is still refused
and left untouched (LLR-uzax55). A deletion that fails there does not refuse
the open (LLR-3fxxpp). When the handle's clock reads before the Unix epoch
at open, that reading is never taken as time 0: no deletion is judged on
it, every kept removal is left with its file and item, and the condition
is reported as each one's deletion failure (LLR-rav7dr); the delivery
resumed at start then reports it as its round's failure (LLR-5rxqr8,
LLR-bv6wkz).
satisfies: REQ-98a3ty

*Amended in place 2026-10-28 (unmerged draft; round 2 review, C2-O1;
dispatcher's ruling of 2026-10-28):* a reading before the epoch was taken
as time 0 at open, so nothing was found due and nothing was reported; open
now judges no deletion on it and reports it.

*Amended in place 2026-10-25 (unmerged draft; PR-wch3zu, plan task T60):* a
`removed-` leftover whose key still has a live `persona-` store file — which
no real rename leaves, but a restored backup or copy can — is skipped, so the
live Persona never loses its keychain item (RC-hd5des's rule); it stays,
reported, until one of the two files goes.

*Amended in place 2026-10-20 (unmerged draft; consistency pass, dispatcher's
decision on the deletion order):* adds the first step. A file the deletion
marked removed but could not finish — its keychain item not yet deleted, or
deleted and the file not yet removed — is found by its name at the next
start and its deletion finished there, in the same order.

*Amended in place 2026-10-21 (unmerged draft; owner decision of
2026-10-20):* finishing leftover `removed-` files at start, while a
`persona-<hex>.store` file without its item is still refused (RC-hd5des),
is owner-confirmed 2026-10-20, with LLR-vj2nkb's order. No behaviour
change.

**LLR-3fxxpp**: when `mark_removed` fails, the deletion deletes nothing
else, the Persona stays in the service with its file and item, the failure
is reported in `acknowledgement_report` (LLR-rav7dr), and the deletion is
tried again one hour later while the handle runs and at the next open
(LLR-upqj5g); when the file is marked removed but the keychain deletion
fails, or the item is deleted but `remove_removed_file` fails, the Persona is
dropped from the service, the failure is reported, and the rest of the
deletion — the item, then the file, or the file alone — is tried again each
hour while the handle runs and, because the `removed-<hex>.store` file still
names the key, at the next open (LLR-upqj5g). No step leaves a keychain item
that no file names, or a store file that opens. When the deletion reads
org-node's list to learn whether the Persona is still held, only
`PersonaNotHeld` (or no removal kept) means it is not; any other failure of
that read is a failed step, reported and retried hourly, with nothing
deleted. Each retry is due an hour after the clock's reading; when the
clock reads before the Unix epoch, never taken as time 0, the retry waits
one hour by the runtime's timer instead.
satisfies: REQ-98a3ty

*Amended in place 2026-10-28 (unmerged draft; round 3 review, T3-O2; owner
ruling of 2026-10-28):* untested clause: "any other failure of that read is
a failed step, reported and retried hourly, with nothing deleted". No store
org-node opens makes its list refuse with anything but `PersonaNotHeld`, so
that failure can be produced only by a double of org-node, which the project
owns (TEST_GUIDELINES Doubles 1); its test and the test-build fault are
removed. The code stays as it is.

*Amended in place 2026-10-28 (unmerged draft; round 2 review, C2-O2 and
C2-O1; dispatcher's ruling of 2026-10-28):* the deletion read any refusal
of org-node's list as "not held", so with no `removed-` file it forgot the
removal and ended, the Persona still held and nothing reported; only
`PersonaNotHeld` means not held now, and any other failure is that
attempt's failure, reported and retried. A retry due time was the clock's
reading, a reading before the epoch being taken as 0; such a reading now
gives an hour by the runtime's timer.

*Amended in place 2026-10-28 (unmerged draft; review round 1 C-O12):*
`remove_store` is renamed `store_file::remove_removed_file`, for the only
file it removes. No behaviour change.

*Amended in place 2026-10-20 (unmerged draft; consistency pass, dispatcher's
decision on the deletion order):* with the keychain item deleted before the
file (LLR-vj2nkb), the limit this item recorded — an item left when the
handle closes, named by no file and so found by no later start — is closed:
a failure after the rename leaves a `removed-` file that names the key, and
a store file left after the item is gone is sealed under a secret that no
longer exists, so it cannot be opened, and is deleted at start.

*Amended in place 2026-10-21 (unmerged draft; owner decision of
2026-10-20):* the deletion order and the finishing at start this item
relies on (LLR-vj2nkb, LLR-upqj5g) are owner-confirmed 2026-10-20. No
behaviour change.

## Under SDD-8uyg4s (the authenticated endpoint)

SDD-8uyg4s (`2026-10-07-transport.md`) stays the owner of the iroh
endpoint; this item restates its confinement rule over the send path's new
addressing.

**LLR-4k43dm**: iroh is named only in `org-io/src/transport/` (and, under
`test-support`, in `org-io/src/test_support.rs`): no other file of
`org-io/src` names `iroh::` in code, and no function outside the transport
module returns or takes an iroh type. The rest of org-io reaches the
transport through org-io's and org-node's own types: it dials a recipient by
its `DevicePublicKey` with the endpoint addresses org-node keeps for that
key, or the Invite's (org-node's `EndpointAddresses`; LLR-34tfm8,
LLR-qa4nrt), binds an endpoint by org-io's own bind plan (relayed, or direct
only, LLR-cqr26z), and asks an endpoint for the addresses it holds in the
current mode as strings (LLR-v9dkrv) and whether it has a relay home, not
for its iroh address; no transport function takes an address parsed from
text a user gave.
satisfies: REQ-xuzw32
supersedes: LLR-3vb4k9

*Note 2026-11-04 (change `worktree-org-io-publish-2`, task q1r):* the
bind plan's rule is now LLR-spckm6's (successor of LLR-cqr26z): direct only,
or relayed for a dial by key alone in Networked mode. No change to this
item's rule.

*Added 2026-10-21 (unmerged draft; design gap G2 of the plan):* LLR-3vb4k9
named the dial at a transport-owned `PeerAddr` (`OrgEndpoint::send_body_at`,
SDD-wr8m5n's parse), which this change removes with the peer-address parse
(SDD-2bzgtp supersedes SDD-wr8m5n; LLR-2b2ghe retired). Its rule — iroh one
adapter's dependency, confined to the transport module — holds unchanged
over the addresses kept per Device key.

## Under SDD-z3ychz (the handle)

SDD-z3ychz (`2026-10-08-org-io.md`) stays the owner of the app's one way in;
these two items state its new surface.

*Amended in place 2026-10-28 (unmerged draft; round 2 review, T2-O1):* the
test-only `node_for_test()` (org-node's whole service under the handle's
lock) and `store_for_test` (a store change a test had org-node make,
written as the handle writes) are gone: no test reaches org-node's service
through the handle or writes a store change itself. Tests reach every state
through the handle's operations, over `FakeChain` and Loopback, and read
what the handle persisted from its files (org-node's service over them,
apart from the handle). The notes of 2026-10-08 and 2026-10-13 in
`2026-10-08-org-io.md` and `2026-10-07-transport.md` that name
`node_for_test()` record the surface as it then was.

*Recorded exception 2026-10-28 (unmerged draft; owner ruling A2 of
2026-10-28, review round 1 of the ARCHITECTURE review; guidelines file
`.guardrails/templates/ARCHITECTURE_GUIDELINES.md`, Boundaries 2, "An
error crosses a boundary as the interface's own error … it does not pass a
dependency's error type through to its caller"):* org-io's public error
enums carry org-node's `OrgNodeError` as a variant: `WriteRefused`,
`SendRefused`, `RequestError`, `CreateError` and `OperationError` in
`Node(OrgNodeError)`, the error of `exchange.rs` in `Node(OrgNodeError)`,
`ReceiveError::Refused`, `OpenError::Unseal` and `OpenError::Service`, and
`KeychainError::Store`. `OrgNodeError` is re-exported through `org_io::node`
(LLR-3zdw8v's clarification of 2026-10-14 lists it among the value types
that module re-exports). The owner accepts this coupling as a design
decision and an exception to Boundaries 2 for SDD-z3ychz's surface:
`OrgNodeError` is part of org-io's public interface through
`org_io::node`, not a dependency's type leaking past it. The exception
covers only the refusals org-node returned: a refusal org-io decides
itself — its lookup of the Persona holding an Organisation, a post-check
after an org-node call succeeded, a body kept by a Persona bound to no
Organisation — is its own variant (`OperationError::NotHeld`,
`AdmissionNotListed`, `ProposalNotListed`, `ExchangeError::NotHeld`,
`CreateError::FoundingNotListed`, `ReceiveError::NoOrganisation`; round 2
review, AR2-5, 2026-10-28). The app matches it
on purpose — `app/src-tauri/src/events.rs::classify_receive_error` matches
every variant with no wildcard arm (the app's design item for that
function), and
`app/src-tauri/src/commands.rs` matches `OrgNodeError::InvalidField` — so a
variant org-node gains breaks the app's build until the app classifies it.
That tripwire is the intended effect. The wrapping keeps the cause: each
such variant holds the `OrgNodeError` it received (LLR-48me7m for
`WriteRefused`'s `source()`). No item's meaning changes; no code changes.

*Exception amended 2026-10-28 (unmerged draft; round 4 ARCHITECTURE review,
findings 1 and 2):* three more public types carry a refusal org-node
returned and are within the exception: `verify::Verified::Refused { error:
Cause<OrgNodeError> }` (org-node refused the verification for a reason with
no outcome of its own), and `view::DeletionFailure::Listing` and
`view::RoundFailure::Listing` (each `Cause<OrgNodeError>`, org-node's
refusal of the listing the deletion or the delivery round read). A failed
chain read is not an org-node refusal and is outside it:
`UnverifiedReason::Read` holds org-io's own `verify::ChainReadError`, whose
`source()` is the reader's error (LLR-ce42ce as amended this day).

*Exception amended 2026-10-30 (unmerged draft; merge review round 2,
finding-1):* a fourth public type carries a refusal org-node returned and
is within the exception: `view::AcceptanceFailure::Refused`
(`Cause<OrgNodeError>`, org-node's refusal to delete an acknowledgement its
recipient accepted), reachable in the public report as
`AcknowledgementOutcome::AcceptedNotRecorded { reason }` (LLR-6ux6yw and
LLR-rav7dr as amended 2026-10-30). Its sibling `AcceptanceFailure::Store`
holds org-io's own store-write error and is outside it.

**LLR-4usfu2**: `OrgIo`'s production public methods for the domain are
exactly `propose_person`, `person`, `propose_founding_organisation`,
`issue_invite`, `read_invite`, `propose_member`, `request_admission`,
`propose_admission`, `propose_revocation`, `cancel_invite`,
`propose_updated_organisation`, `write_organisation_to_chain`,
`verify_organisation`, `verify_all`, `verification`, `send_organisation`,
`request_organisation`, `start_receiving` and `next_received`, beside the
queries (`personas`, `organisations`, `organisation`,
`admission_requests`, `acknowledgement_report`) and `open`,
`connect`, `not_configured`, `is_own_admin`; none of `create_persona`,
`found_organisation`, `admit_member`, `revoke_member`, `expect_admission`,
`produce_reply`, `import_reply`, `admit_reply`, `submit_commit_send`,
`settle_unknown`, `reconcile` or `reconcile_held` exists, and no public
method takes a `PeerAddr` or sends, deletes or resends an acknowledgement.
satisfies: REQ-6fk4qj

*Amended in place 2026-10-20 (unmerged draft; owner answers A2, A3 and
ruling Q4' of 2026-10-16):* the queries gain `admission_requests`
(LLR-dm2h8u) and `acknowledgement_report` (LLR-rav7dr); no method takes a
peer address; the acknowledgement delivery is org-io's own, with no command.
superseded-by: LLR-zhdc9f

*Superseded 2026-11-03 (change `worktree-org-io-publish-2`, task T13b; owner
ruling 6 and plan §0's T16g) by LLR-zhdc9f (`2026-11-02-publish-2.md`):*
`is_own_admin` leaves the list and `removed_devices` joins the queries.

**LLR-n2srhc**: no public field of a type `org-io/src` defines is declared
with the type `OrgService`, and no public function of `org-io/src` outside
test-only (`test-support`) code returns an `OrgService` by value: the app
reaches org-node's whole service only through `OrgIo`'s methods, and a
refused `connect` hands the service back only as an unconfigured handle
(`ConnectFailed::into_not_configured`). An architecture test reads org-io's
sources.
satisfies: REQ-6fk4qj

*Added 2026-10-28 (unmerged draft; round 5 TEST review, low 11):* the scan
`org_io_hands_out_no_bare_service` cited LLR-4usfu2, which states the exact
method list, not this rule; since LLR-7pj5af and LLR-9r2bxd were superseded
no in-force item stated it. *Note 2026-10-28 (robustness, N/A):* a scan of
org-io's sources, which takes no input, so it has no abnormal-input case; it
was watched red against a planted public `OrgService` field.

**LLR-7qdjqn**: `OrgIo::is_own_admin(org_id)` returns
`Err(OwnAdminError::NotConfigured)` when org-io holds no signatory key;
otherwise it makes one signatory-set read,
`SignatorySetReader::signatory_set(org_id)` (on-chain-client's pinned
read, which reads the mapped account and, only for a mapped H160, its proxy
definitions at one finalised block), returns `Err(OwnAdminError::Read(_))`
when that read fails at either value, never `Ok(NotAdmin)` for a failed
read, and otherwise returns `Ok(admin_status(controller_of(own,
co_signers), &set))` — `Admin`, `DelegateCannotDispatch` or `NotAdmin` by
LLR-2hkqv2's rule.
satisfies: REQ-ntkjk6
supersedes: LLR-c9fyun
retired: 2026-11-03 — owner ruling 6 of 2026-10-16: OrgIo::is_own_admin is removed in plan task T16h of change worktree-org-io-publish-2 (REQ-ntkjk6 retired the same day)

*Deleted 2026-11-03 (change `worktree-org-io-publish-2`, task T13b; owner
ruling 6; plan task T16h, which also drops `is_own_admin` from
`org-io/tests/absences.rs`'s allowlist):* the method, its error type and
its reader go; the absence is pinned by `absences.rs`
`the_handle_offers_none_of_the_removed_methods` and
`the_handles_production_methods_are_exactly_the_domain_surface`.

*Amended in place 2026-11-09 (change `worktree-org-io-publish-2`, merge
review round 1, MR1-O5; a wording change):* the deletion note above named
`org_io_offers_no_own_admin_check`, which task o2 replaced; it now names
the two `absences.rs` tests that pin the absence.

*Added 2026-10-21 (owner decision of 2026-10-20; consistency pass).*
LLR-c9fyun (`2026-10-08-org-io.md`) still read "either signatory-set read"
and `admin_status(…, original_account, delegates)`, which disagree with
LLR-2hkqv2 (`2026-10-13-publish.md`): `admin_status` takes the signatory
set from one pinned read. LLR-2hkqv2 is right — it is the in-force rule
the owner ruled (D4, D5 of 2026-10-13), the one read is on-chain-client's
pinned read the code makes, and LLR-c9fyun's own 2026-10-13 notes already
restated it so. This item states LLR-c9fyun's claims over that one read,
with the result set LLR-2hkqv2's rule gives, and supersedes it; no
behaviour changes.

**LLR-bwerd2**: the `SignatorySetReader` seam (SDD-f4khqn) answers as the
chain does: `signatory_set(org_id)` returns no mapped account and no
definition for an Organisation the chain holds no state for, and, once the
Organisation's account is mapped to its proxy and its proxy definitions are
the controller's founding one, the proxy as the mapped account and that one
definition, `(controller, Any, 0)`, with the runtime's `Any` index. Every
reader org-io holds, the production `OnChainSignatorySetReader` and the
test-support fake, meets it.
satisfies: REQ-ntkjk6
retired: 2026-11-03 — owner ruling 6 of 2026-10-16: the SignatorySetReader seam exists only for the own-admin check, removed in plan task T16h of change worktree-org-io-publish-2

*Deleted 2026-11-03 (change `worktree-org-io-publish-2`, task T13b; owner
ruling 6; plan task T16h):* the seam, the production reader and the
test-support fake's signatory slots go; their tests go with them.

*Added 2026-10-28 (unmerged draft; round 4 TEST review, finding-1):* see
LLR-js7wgk's note.

**LLR-gh4rw3**: every type the methods of LLR-4usfu2 return
(`PersonSummary`, `ProposedFounding`, `ProposedMember`, `InviteSummary`,
`ProvisionalChangeSummary`, `ProposedOrganisation`, `WriteReport`,
`WriteRefused`, `Verified`, `WriteOutcome`, `SendReport`, `SendRefused`,
`Delivery`, `RequestError`, `Received`) holds public data only and
is in `HANDLE_RETURN_ALLOWLIST`; none holds an Organisation private key, a
Device secret or a store key.
satisfies: REQ-6fk4qj, REQ-3pxa8f

*Amended in place 2026-10-28 (unmerged draft; round 2 review, C2-O5):*
`AdmissionOutcome`, an alias of `Delivery`, is gone; `request_admission`
returns `Delivery`, which the list and the allowlist name instead.

*Amended in place 2026-10-20 (unmerged draft; owner answer A3 and ruling
Q4' of 2026-10-16):* `KeptAdmissionRequestSummary` (LLR-dm2h8u) and
`AcknowledgementReport` (LLR-rav7dr) join the list and the allowlist; the
report holds recipients' DevicePublicKeys and outcomes, never an
acknowledgement's signed bytes.

*Note 2026-11-03 (change `worktree-org-io-publish-2`, task T13b; a
clarification, the ID is kept; the rule is unchanged):* the methods are now
LLR-zhdc9f's. `RemovedDevice` and `RemovedDevicesRefused` (LLR-3ydu9e)
join the list and `HANDLE_RETURN_ALLOWLIST`: MemberIds, DevicePublicKeys,
an epoch, `NotHeld { org_id }` and org-node's typed refusal, all public;
`AdminStatus`, `OwnAdminError` and the `signatory` path segment leave the
allowlist with the own-admin check (plan task T16h).

**LLR-jqb5z7**: org-io's production code reads org-node's stores only
through the key-free views org-node offers for it (org-node's views:
`persona_view`, `persona_records`, `provisional_changes`,
`provisional_organisations`, `admission_requests`, `kept_acknowledgements`
and `PersonStore::view`): no line of `org-io/src` outside `test_support.rs`
and items gated on `test-support` calls `OrgService::list_orgs`
(`OrgRecord`, holding the Organisation private key),
`OrgService::list_personas` (`PersonaRecord`, holding the member seed),
`PersonStore::data` (`PersonStoreContent`, holding the Person seed) or
`OrgService::store_data`, so org-io holds no value carrying a key field that
it could clone or keep. An architecture test (`org-io/tests/absences.rs`)
checks it.
satisfies: REQ-v4tfap

*Note 2026-11-03 (change `worktree-org-io-publish-2`, task T13b; a
clarification, the ID is kept; the rule is unchanged):* org-node's
`removed_devices` (a key-free view: MemberIds, DevicePublicKeys and signed
acknowledgements) joins the views org-io reads through (LLR-3ydu9e).

*Added 2026-10-28 (unmerged draft; review round 1 A1 and A9, ARCHITECTURE
Boundaries 1 and Constraints 1):* org-io's verification, chain write and
Person reads went through `list_orgs()` and `PersonStore::data()`, and its
listings through `list_personas()`; the key fields were crate-private, but
the values carrying them were org-io's to clone. They now go through
`persona_view` (of the Persona `record_holder_of` finds), the new
`persona_records` and the new `PersonStore::view`. No behaviour changes.

*Note 2026-10-28 (robustness, N/A):* an absence in the source, which takes
no input, so it has no abnormal-input case; the scan was watched red against
a planted `service.list_orgs()` in `verify.rs`.

*Amended in place 2026-10-28 (unmerged draft; round 4 ARCHITECTURE review,
finding-3, Domain 1):* org-node's `PersonData`, which `PersonStore::data`
returns, is renamed `PersonStoreContent` (the root glossary lists "person
data" under _Avoid_); the item names the new type. No behaviour change.

**LLR-ntag3g**: no file of `app/src-tauri/src` calls org-node's
`check_received`, `verify`, `messages_for_current_epoch` or
`acknowledgement_accepted`, builds an `OrgService` (`OrgService::new`) or
names `org_node::service`: the app reaches org-node's receive, verification
and sends only through `OrgIo`'s methods (`next_received`,
`verify_organisation`, `send_organisation` and the delivery org-io runs),
never around them. An architecture test (`org-io/tests/app_boundary.rs`)
reads the app's sources.
satisfies: REQ-6fk4qj

*Added 2026-10-28 (unmerged draft; round 3 review, T3-O6):* the scan cited
LLR-kz9hj4, which states `next_received`'s behaviour, not the app's code;
no item stated the scan's rule since LLR-kz9hj4 superseded LLR-7pj5af,
whose test it was.

*Note 2026-10-28 (robustness, N/A):* a scan of the app's sources, which
takes no input, so it has no abnormal-input case; it was watched red
against a planted `OrgService::new(` in the app's sources.

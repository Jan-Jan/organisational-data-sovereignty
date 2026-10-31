# Design — batched epochs, verification and admission requests (org-node)

Software items and low-level requirements for change
`worktree-org-io-domain-api`, drafted 2026-10-15 before any code. They refine
org-node's requirements of this change
(`org-node/docs/requirements/2026-10-14-batched-epoch.md`)
under the two root decisions it rests on, in `docs/adr/` (cited by file
name, since a root ADR is not an item of this unit): the batched-epochs
decision, `…-batched-epochs.md` (an Admin's admissions and revocations are provisional changes, calculated
once per epoch into one provisional organisation with one fresh Organisation
key pair) and the write-verify-send decision, `…-write-verify-send.md`
(write, then verify,
then send: nothing is adopted on receipt, verification alone adopts). Every
item is class C, the unit's class, with no per-item override.

## The shape of the change, for an implementer

- **One Persona per Organisation membership** (owner ruling R1). A Persona
  store holds exactly one Organisation's state: its current record (the
  existing `OrgRecord`: members, Membership root, Organisation key pair) and,
  beside it, the epoch state — provisional changes, provisional
  organisations, a provisional founding or a provisional Member, kept
  admission requests, outstanding invites, expected admissions, the
  Organisation name and the behind marker. `provisional_updates` is removed.
- **Values in, values out.** Every operation below takes values, returns
  values and at most one `StoreChange`; org-node holds no chain client, no
  endpoint, no file and no keychain, as since S4.
- **Proposals are not commitments.** `record_admission` and
  `record_revocation` keep provisional changes and calculate nothing;
  `propose_updated_organisation` calculates one next epoch from all of them;
  `verify` alone replaces the current record.
- **Receipt keeps, verification adopts** (owner rulings R2, R3).
  `check_received` is chain-free and never adopts.
- **Six Wire message kinds**, the frame bound unchanged.

## Segregation and dependencies

All units are class C. This change adds no cross-unit edge: org-node still
depends on `org-members` and `person` alone (its `depends_on` is unchanged),
and names no org-io or app item. org-io meets the new operations by values,
and names no `person` type because org-node exposes the Person store under
its own type (LLR-5uf4u2). The SOUP inventory (`soup.md`) is unchanged: no
dependency is added or changed.

## The epoch state in the Persona store

**SDD-undd7q**: the epoch state of one Organisation in its Persona store. Code: `org-node/src/store.rs` (`StoreData`, `ProvisionalChange`, `ProvisionalOrganisation`, `ProvisionalFounding`, `ProvisionalMember`, `KeptAdmissionRequest`, `MAX_PROVISIONAL_BYTES`) and the routing in `org-node/src/service.rs`. It holds what one Persona knows of its one Organisation beyond the current record, within a bound, sealed with the rest of the store; it decides nothing about the chain. This item adds no cross-unit edge (org-node depends on org-members and person alone, unchanged).
traces: REQ-dhr58q, REQ-3pj7rd, REQ-9ktgwx, REQ-g2537c, REQ-hx4r8y, REQ-6qxesy, REQ-akfwf3, REQ-4qh2h8, REQ-ky5tyd, REQ-kd34f5, REQ-x2dnee, REQ-x3jrh8, REQ-qthum6, REQ-yp75u9, REQ-ech45n, REQ-ttv8k7, REQ-p4hwuv

*Amended in place 2026-10-28 (unmerged draft; round 3 review AR3-5, sweep
for ARCHITECTURE Boundaries 3):* a sentence reworded so no clause is joined by "and"; the responsibility, the epoch state, its code and its LLRs are unchanged.

*Amended in place 2026-10-20 (unmerged draft; owner answer A2 and owner
ruling Q3 of 2026-10-16):* the epoch state also holds the per-Device address
book (LLR-h9d65p) and the Devices this node holds as revoked (LLR-bgj2ff),
and so traces REQ-p4hwuv. The address book is not provisional data: it is
outside the 1 MiB bound of REQ-g2537c, which names provisional changes and
provisional organisations only, and is bounded instead by one entry per
DevicePublicKey the record lists or holds as revoked, each entry at most
sixteen addresses of 256 bytes (LLR-h4gwns). The store of a Persona removed
from its Organisation is SDD-z4m57w's.

**LLR-d925uu**: `StoreData` holds, in this encoding order: `persona:
PersonaRecord`; `record: Option<OrgRecord>` (the current record, at most one,
in place of `orgs: Vec<OrgRecord>`); `org_name: String` (the Organisation
name, free text); `provisional_changes: Vec<ProvisionalChange>` in the order
recorded; `provisional_organisations: Vec<ProvisionalOrganisation>`;
`founding: Option<ProvisionalFounding>`; `member: Option<ProvisionalMember>`;
`kept_admission_requests: Vec<KeptAdmissionRequest>`;
`kept_revocation_notices: Vec<RevocationNotice>`; `expected_admissions:
Vec<ExpectedAdmission>`; `outstanding_invites: Vec<OutstandingInvite>`,
each `OutstandingInvite { org_id: OrgId, invite_id: InviteId, invitee_name:
String }` in that order, `invitee_name` the name the inviter typed for that
Invite (LLR-h9f5c9);
`revoked_devices: Vec<RevokedDevice>` (LLR-bgj2ff); `address_book:
Vec<(DevicePublicKey, EndpointAddresses)>` (LLR-h9d65p). It
holds no `provisional_updates` field and no `ProvisionalUpdate` type exists.
Every field reaches the sealed store and is read back unchanged by
`PersonaStore::unseal`, except the behind marker (LLR-zk7fx2). A
`StoreData` is sealed as the `Active` form of the store plaintext
(LLR-6zaeb6).
satisfies: REQ-dhr58q, REQ-3pj7rd, REQ-9ktgwx, REQ-hx4r8y, REQ-6qxesy, REQ-ky5tyd
supersedes: LLR-95753m

*Amended in place 2026-10-20 (unmerged draft; owner answer A2 and owner
rulings Q3 and Q4' of 2026-10-16):* adds the last two fields, and the
`Active` form.

*Amended in place 2026-10-20 (unmerged draft; consistency pass, REQ-ky5tyd
as amended that day):* an outstanding invite keeps the invitee name the
inviter typed, beside its `(OrgId, InviteId)` pair, so that each kept
admission request can be returned with it (LLR-8ru6mv).

**LLR-b2hgd4**: `ProvisionalChange` is `Admit { invite_id: InviteId, handle:
Handle }` (naming the kept admission request it admits and the Handle chosen
for it, LLR-bwtx3h) or `Revoke { member_id: MemberId }`; it holds no
Membership root, no Change set, no Sequence number and no key. Recording one
appends it to `provisional_changes` and changes no other field: the current
record, its Membership root and its Organisation key pair are byte-for-byte
as they were.
satisfies: REQ-dhr58q

**LLR-s3vq6d**: a `ProvisionalOrganisation` holds `epoch: Epoch`,
`base_root: RootHash` (the root of the record it extends), `members:
Vec<MemberSnapshot>` (the next record), `root: RootHash`, `org_pub_key:
OrgPublicKey`, `org_private_key: OrgPrivateKey`, `change_set: Option<Vec<u8>>`,
`base_snapshot: Option<Vec<u8>>`, `origin: Origin`, where `Origin` is
`Own`, `Received { from: DevicePublicKey }` or `Answer { from:
DevicePublicKey }`, and `applied: Vec<ProvisionalChange>` (for an `Own` one,
the provisional changes it applied, in recorded order, LLR-va2yqz; empty
for any other). Its identity is `(root, org_pub_key)`, and `org_pub_key`
is always the X25519 public half of `org_private_key` (it is derived from it
when one is received, never taken from a message). Keeping one whose identity
is already held keeps no second copy and reports it as already held; several
with different identities may be held for one epoch.
satisfies: REQ-3pj7rd, REQ-9ktgwx, REQ-2yduet

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR3-N1):* a `ProvisionalOrganisation` also holds, last, after
`applied`, `first_ahead_at: Option<VerificationTime>`: the time of the
first verification that found it one epoch ahead of the chain
(LLR-4tsgfh); `None` when kept, and always for an `Own` one. It is part of
the stored form (and so of REQ-g2537c's bound), encoded as a postcard
option; a store sealed before this field is not read (LLR-78jfrd).

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* adds
`applied`, so that org-node can tell org-io whether its own provisional
organisation is still the proposal of the current record and the current
provisional changes (LLR-8ru6mv), which org-io's write must check and no
operation reported.

**LLR-qw8tyu**: the Organisation private key of every provisional
organisation and of the provisional founding exists only inside the
`StoreData` it was kept in, reaches disk only inside the sealed Persona store,
is moved into the current record's `org_private_key` by the adoption that
selects it (LLR-34c5y3, LLR-gcm7bq), and leaves the store with its
provisional organisation when that is dropped (LLR-x8drb8); no operation
returns it to a caller except inside an encoded Wire message to a Device the
current record lists (LLR-degg8t, LLR-7kdhst). The current record holds
the key of the epoch it adopted and no earlier one (LLR-78jfrd); a fresh key
pair is drawn only by `propose_founding` (LLR-9qavm7) and
`propose_updated_organisation` (LLR-qghs95), and a received provisional
organisation holds the key its message carried.
satisfies: REQ-3pj7rd, REQ-ech45n
supersedes: LLR-qjz3q4 LLR-3fwykc

*Amended in place 2026-10-21 (unmerged draft; design gap G2 of the plan):*
supersedes LLR-3fwykc (the Organisation private key held only in the
record and, until `commit_genesis`, in the genesis provisional update; both
on disk only in the encrypted store; the record's key replaced by each
commit; a fresh one drawn only when a provisional update is built), whose
operations are removed: this item states the same custody over the
provisional organisations, the provisional founding and adoption, and the
added sentence its replacement and draw rule.

**LLR-m76r4k**: every operation that keeps a provisional change, a
provisional organisation or a revocation notice measures the postcard
encoding of the tuple `(provisional_changes, provisional_organisations,
kept_revocation_notices)` the store would hold after the keep, and when that
length exceeds `MAX_PROVISIONAL_BYTES` (1 MiB, `1 << 20`) refuses with
`ProvisionalLimit { limit: MAX_PROVISIONAL_BYTES }`, keeping nothing,
returning no `StoreChange` and leaving `StoreData` exactly as it was. The
operations are `record_admission`, `record_revocation`,
`propose_updated_organisation` and `check_received`.
satisfies: REQ-g2537c
supersedes: LLR-jq7qh7

**LLR-p8vpgz**: `OrgRecord` keeps `kept_change_set: Option<Vec<u8>>` and
gains `kept_base_snapshot: Option<Vec<u8>>` (the postcard `Vec<MemberSnapshot>`
of the record the kept Change set extends), both encoded with the record. An
adoption sets both to the adopted provisional organisation's `change_set` and
`base_snapshot`, replacing any earlier ones; a record created by a founding,
or adopted from an organisation answer (LLR-kd7u2r), keeps `None` for both.
Each is bounded by `MAX_MESSAGE`, being part of a body that was built or
received within it.
satisfies: REQ-x2dnee, REQ-ttv8k7
supersedes: LLR-d9778a

**LLR-78jfrd**: `OrgRecord` (and its decode mirror `RawOrgRecord`) has no
`org_secret` field, and its `org_private_key` is an `OrgPrivateKey`, not an
`Option`, encoded as its plain 32 bytes with no option tag: every current
record a store holds carries the Organisation's private key, the one value
the whole Organisation shares for that epoch. A record is created only by a
first adoption — a founding's, with the provisional founding's key
(LLR-gcm7bq), or a first admission's, with the matched provisional
organisation's (LLR-q4puca) — and its key is replaced only by an adoption,
with the match's (LLR-34c5y3); the record holds only the adopted epoch's key
and keeps no earlier one. Stores written before this change are not
supported: nothing reads or migrates their layout.
satisfies: REQ-ju6vn2, REQ-ech45n
supersedes: LLR-byjvd9

*Added 2026-10-21 (unmerged draft; design gap G2 of the plan):* LLR-byjvd9
named `commit_genesis`, `commit_update` and the committed
Organisation-information message as the record's creators and replacers,
which this change removes; its rule — a record always holds the
Organisation private key, the current one alone, in that encoding — still
holds, and this item states it over the adoptions that now create and
replace a record.

**LLR-zk7fx2**: `StoreData` holds `behind: Option<Epoch>`, the chain epoch
the last verification reported behind, as a field that is not encoded
(`#[serde(skip)]`): it lives in the open store and is `None` after
`unseal`. `verify` sets it on `Behind { chain_epoch }` and clears it on
`InStep` and `Adopted`; while it is `Some`, `record_admission`,
`record_revocation` and `propose_updated_organisation` refuse with
`OrganisationBehind { org_id, chain_epoch }`, changing nothing. Because it is
not encoded, setting it changes no sealed byte, so `Behind` still returns no
`StoreChange` (REQ-dwb9bq); org-io verifies at start (its own requirement),
which sets it again. `restore_store`, which puts back a store as its file
holds it, sets the replacement's `behind` to the replaced store's when the
replaced store is Active; a store added where none was held, or one
replacing a store with no marker or a store in the removed form (which holds
no `StoreData`, so no marker), has none; a replacement that is itself in the
removed form holds no `StoreData` and so no marker, whatever it replaces.
satisfies: REQ-akfwf3, REQ-dwb9bq

*Amended in place 2026-10-31 (change `worktree-org-io-domain-api`, merge
review round 7 finding-2):* the statement names the case of a replacement
in the removed form, which the code (`restore_store`) already handles; no
behaviour changes. A failed write of a removal leaves the Persona Active
with no marker until the next verification; that window is recorded as an
open problem in this unit's problems file, not stated as required here.

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR3-N1):* the marker itself still changes no sealed byte, but a
`Behind` verification now returns a `StoreChange` when LLR-4tsgfh's discard
or stamp changed the store (LLR-qvp7mq's note); "`Behind` still returns no
`StoreChange`" holds only when it did not.

*Amended in place 2026-10-31 (change `worktree-org-io-domain-api`, merge
review round 5 finding-2):* `OrgService::restore_store`, which puts back a
store as its file holds it (org-io's reload after a failed write),
keeps the behind marker of the store it replaces: the
replacement's `behind` is set to the replaced store's, since no file holds
it. A store added where none was held, or one replacing a store with no
marker, has none. Before this, a failed write of the change a `Behind`
verification returned reloaded the store with no marker, so until the next
verification `record_admission`, `record_revocation` and
`propose_updated_organisation` were not refused, and an organisation answer
was refused as not requested. Tests:
`a_store_put_back_from_its_file_keeps_the_behind_marker_the_replaced_store_held`
(proposals); org-io's
`a_failed_write_of_a_discard_on_a_behind_verification_keeps_the_organisation_behind`.

*Amended in place 2026-10-31 (change `worktree-org-io-domain-api`, merge
review round 6 finding-2 and finding-3):* the restore rule of the note above
now stands in the statement, which also states the case of a replaced store
in the removed form. A removed store holds no `StoreData` and so no marker:
the marker the Active store held went when the removal replaced its content,
before any write. So a failed write of a removal, which org-io follows by
putting back the Active store from its file, gives a store with no marker,
as `unseal` does; nothing held is lost by the restore. Test:
`a_store_put_back_over_a_removed_store_replaces_it_with_no_behind_marker`
(own_removal).

**LLR-8ru6mv**: `OrgService::provisional_changes(device_key)` and
`OrgService::provisional_organisations(device_key)` return, by reference, the
store's provisional changes in recorded order and, for each provisional
organisation, a `ProvisionalOrganisationView { epoch, base_epoch, root,
org_pub_key, origin, current_proposal }` that holds no private key,
`current_proposal` being true exactly for an `Own` one whose `base_epoch` is
the current record's epoch and whose `applied` equals the store's
`provisional_changes`; neither clones an
`OrgPrivateKey` nor changes the store. `OrgService::admission_requests(device_key)`
returns the kept admission requests (LLR-v3av9t) in the same way, in kept
order, each paired with the `invitee_name` of the outstanding invite whose
invite id it carries (LLR-d925uu). `OrgService::persona_view(device_key) ->
Result<PersonaView, OrgNodeError>` returns, by value, a copy of the store's
public values: `PersonaView { device_key, org_id, org_name, record:
Option<RecordView { org_id, root_hash, org_pub_key, epoch, members }>,
founding: Option<FoundingView { epoch, root, org_pub_key, proxy }>,
member_of: Option<OrgId> }` — the Persona's DevicePublicKey and the
Organisation it is bound to, the Organisation name, the current record's
identifier, Membership root, Organisation public key, epoch and member
snapshots, the provisional founding's epoch, root, Organisation public key
and recorded proxy account, and the Organisation a provisional Member asked
to join. None of its fields is a private key, a seed or a secret (no member
seed, no Device seed, no `OrgPrivateKey`), and reading it changes nothing.
`OrgService::persona_records() -> Vec<PersonaRecordView>` returns, by value
and in `list_personas`'s order, each listed Persona record's public fields
(`PersonaRecordView { persona_id, org_id, name, surname, handle,
device_key, member_id, status }`, no member seed), and
`PersonStore::view() -> PersonView { person, device_key, person_key }` the
Person store's (no Person seed); neither changes anything.
Every per-store view refuses an unknown Device key with `PersonaNotHeld`, as every
per-store operation does (LLR-f9x3vf). These views, not the
`test-support` accessor `store_data` nor `list_orgs`, `list_personas` or
`PersonStore::data`, whose values hold a key field, are how a caller
outside org-node reads a store.
satisfies: REQ-3pj7rd, REQ-dhr58q, REQ-6qxesy, REQ-ky5tyd
supersedes: LLR-nvn3wk LLR-gwk4nk

*Amended in place 2026-10-28 (unmerged draft; review round 1 A1):*
`persona_records` and `PersonStore::view` are added. org-io's listings of
Personas read `list_personas()`, whose `PersonaRecord` holds the member
seed, its record reads `list_orgs()`, whose `OrgRecord` holds the
Organisation private key, and its Person reads `PersonStore::data()`, whose
`PersonStoreContent` holds the Person seed; the key fields are crate-private, but
the values were org-io's to clone. The two views copy only the public
fields, and org-io's rule that it reads stores only through them is
an org-io LLR. `list_orgs`, `list_personas` and `PersonStore::data`
stay for org-node's own code and tests.

*Amended in place 2026-10-22 (unmerged draft; dispatcher decision O1, plan
assembly):* `persona_view` is added. org-io's production code read a store
through `store_data`, which org-node offers only under `test-support` and
which hands out the whole `StoreData`, member seed and Organisation private
keys included (crate-private fields, but the same value); org-io's
proposals, its lookup of the Persona holding an Organisation, its list of
Organisations, its founding write and its verification targets read only the
public values listed, so org-node offers them in every build as a named view
that copies no key, and `store_data` stays a `test-support` accessor.

*Amended in place 2026-10-20 (unmerged draft; consistency pass, REQ-ky5tyd
as amended that day):* each kept admission request is returned with the
invitee name the inviter typed for its Invite, which org-io's list of kept
admission requests and its report of an arriving one pass to the app. The
view also gains `current_proposal`: org-io refuses to write its own
provisional organisation as stale when a provisional change was recorded or
discarded after it was proposed, or its base is no longer the current
record (org-io's write requirement), and no operation of this ledger told
it so.

**LLR-7yu2xy**: `PersonaStore::unseal` refuses as a whole, with
`OrgNodeError::InvalidField` naming the field (`persona.*`, `org.*`,
`member.*`, `provisional.*`, `founding.*`, `provisional_member.*`,
`admission.*`), a decoded store any of whose Handles, names, surnames or keys
its type's parse refuses, or whose provisional Member holds fewer than one,
more than three or repeated preferred Handles; nothing is opened. Each key is
parsed by its own type (Member-as-a-group key as `PersonPublicKey`,
DevicePublicKey as `DevicePublicKey`, Organisation public key as
`OrgPublicKey`). Decoding a record snapshot (`first_admission_base`) refuses
in the same way with `member.*`.
satisfies: REQ-kd34f5
supersedes: LLR-8bum44

**LLR-mkvx2k**: the postcard encoding of a fixed Persona store plaintext (one
Persona; one current record with its Organisation private key, chain account,
members, kept Change set and kept base snapshot; an Organisation name; one
`Admit` and one `Revoke` provisional change; one provisional organisation of
each origin; one kept admission request with its sender's addresses; one
kept revocation notice; one expected admission; one outstanding invite with
its invitee name; one revoked Device; one address-book entry), in its
`Active` form, and of a fixed
removed store (an Organisation identifier, a verification time and two kept
acknowledgements, each with its recipient's key and addresses), in its
`Removed` form (LLR-6zaeb6), each equal the bytes pinned in
`org-node/tests/encoding_golden.rs`, field order as LLR-d925uu and
LLR-6zaeb6 state; the pinned Organisation-information and revocation Wire
messages of LLR-ayrdr8 are unchanged.
satisfies: derived
supersedes: LLR-ayrdr8

*Amended in place 2026-10-20 (unmerged draft; owner answer A2 and owner
rulings Q3 and Q4' of 2026-10-16):* the pinned plaintext gains the new
fields, and a removed store is pinned beside it.

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* the pinned
outstanding invite carries its invitee name (LLR-d925uu), and the pinned
provisional organisations their applied changes (LLR-s3vq6d).

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR3-N1):* each pinned provisional organisation ends with
`first_ahead_at` `None` (LLR-s3vq6d), one `00` option tag after its
`applied`; the pinned active store is re-derived by the format rule, 2105
bytes to 2108.

**LLR-pv2rdy**: `PersonaRecord` holds `persona_id`, `org_id: Option<OrgId>`,
`name: Name`, `surname: Surname`, `handle: Option<Handle>` (set for a
founder at founding, and on a first admission to the Handle the adopted
record gives the Persona's MemberId, LLR-q4puca; `None` for a provisional
Member), the member seed, `device_key: DevicePublicKey`, `member_id:
Option<MemberId>` and `status`; every value is constructed only from parsed
types, and `MemberSnapshot` and `ProvisionalMember` hold the handle, name and
surname typed. No `Joiner` type remains.
satisfies: REQ-kd34f5
supersedes: LLR-g76zqd

**LLR-qqqg4b**: a Persona's Organisation is fixed for the life of its store:
`propose_member` sets `persona.org_id` to the Invite's Organisation
identifier, `record_founding_proxy` sets it on a founding Persona from `None`
to the identifier it is given, and no other operation writes `org_id`.
`record_founding_proxy` refuses with `FoundingProxyAlreadyRecorded` a store
whose `org_id` is already set to another identifier (LLR-nx87k5).
satisfies: REQ-yp75u9
supersedes: LLR-6z5xya LLR-eyc4ud

**LLR-ay8nvu**: every `OrgService` operation that changes a store
(`propose_founding`, `record_founding_proxy`, `propose_member`,
`record_admission`, `record_revocation`, `propose_updated_organisation`,
`verify`, `check_received`, `build_invite`, `cancel_invite`,
`acknowledgement_accepted`) returns, beside
its result, at most one `StoreChange` for the one store it changed:
`Sealed(StoreWrite)` sealed with a fresh 24-byte nonce under the open store's
key (LLR-2qqqx6), including the removed store `verify` leaves on the
Persona's own removal (LLR-2d5a7m); `StoreChange` is `#[must_use]` and has
no `Forgotten` variant: no org-node operation removes a store, whose file
and keychain item org-io deletes. A refusal, and every
outcome that changes nothing, returns none and leaves the in-memory store as
it was before the call. Only `propose_founding`, `propose_member` and
`PersonaStore::unseal` take or return a `DeviceSecret`; `verify` takes the
`device_secrets` closure, called only when it signs acknowledgements
(LLR-5tvn7g).
satisfies: REQ-x3jrh8, REQ-qthum6
supersedes: LLR-bvt5up

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q4' of
2026-10-16):* `Forgotten` is removed. On its own removal a Persona's store
keeps its acknowledgements, so `verify` seals a removed store instead
(REQ-48zc3e), and `acknowledgement_accepted` joins the list (REQ-ayyt5f).

**LLR-f9x3vf**: `OrgService::new(stores: Vec<PersonaStore>)` holds one store
per Persona and refuses (`DuplicatePersonaStore`) two with the same Device
key; each per-store operation names its store by the Persona's Device key and
refuses an unknown one (`PersonaNotHeld`), reading and changing that store
alone; `propose_founding` and `propose_member` add one new store and change
no other.
satisfies: REQ-x3jrh8
supersedes: LLR-rdda86 LLR-vdyu65 LLR-y2v8v2

*Amended in place 2026-10-22 (unmerged draft; dispatcher decision O4, plan
assembly):* supersedes LLR-y2v8v2 (every record the receive path reads or
writes is the one the received change's Organisation identifier names, so a
node holding more than one Organisation leaves every other Organisation's
record, Organisation private key, expectation and provisional updates
exactly as they were), whose receive path is removed. A store holds one
Persona's view of one Organisation, and each per-store operation reads and
changes the store its Device key names alone, which is that rule over
today's operations: no other store's record, Organisation private key,
expected admission or provisional changes is read or changed.

*Amended in place 2026-10-21 (unmerged draft; design gap G2 of the plan):*
supersedes LLR-vdyu65 (`admit_member` builds on the record its `org_id`
names and leaves every other Organisation's record and provisional updates
where they were), whose operation is removed: a store now holds one
Organisation, and an admission is recorded in the store its Device key
names (`record_admission`, LLR-bwtx3h), changing no other store, which is
this item's rule.

*Amended in place 2026-10-28 (unmerged draft; round 2 review, C2-N2):* a
refusal names the input it refused: `InviteNotOutstanding { org_id,
invite_id }`, `NoAdmissionRequestKept { invite_id }`,
`AdmissionAlreadyRecorded { invite_id }`, `HandleTaken { handle }` and
`PersonaNotHeld { device_key }`. `HandleTaken` carries the Handle but its
`Display` leaves it out, a Handle being personal data. A lookup by Persona
identifier (`persona_public_keys`, LLR-437fvx) has no Device key to name,
so it refuses with the new `PersonaIdNotHeld { persona_id }`; every
lookup by Device key stays `PersonaNotHeld` (LLR-f9x3vf).

## Proposing a founding, a Member, changes and the updated Organisation

**SDD-hwax8v**: the proposal operations. Code: `org-node/src/service.rs` (`propose_founding`, `record_founding_proxy`, `propose_member`, `record_admission`, `record_revocation`, `propose_updated_organisation`). They turn the user's intentions into epoch state — a new Persona with a provisional founding or a provisional Member, a provisional change, or one provisional organisation for the next epoch; they write nothing to the chain.
traces: REQ-4qh2h8, REQ-hx4r8y, REQ-kd34f5, REQ-2nnsmv, REQ-dhr58q, REQ-3pj7rd, REQ-akfwf3, REQ-g2537c, REQ-ech45n, REQ-ky5tyd, REQ-uy7kfq

*Amended in place 2026-10-28 (unmerged draft; round 3 review AR3-5, sweep
for ARCHITECTURE Boundaries 3):* a sentence reworded so no clause is joined by "and"; the responsibility, its code and its LLRs are unchanged.

**LLR-d4gpxm**: `OrgService::propose_founding(rng, name: &str, surname:
&str, handle: &str, org_name: String) -> Result<(DevicePublicKey,
DeviceSecret, StoreChange), OrgNodeError>` parses the three details
(LLR-rjphn4), draws a fresh Device seed as a `DeviceSecret` and a member
seed, builds the founding record listing the founder alone under a MemberId
drawn from `rng` with that Member-as-a-group key and DevicePublicKey,
calculates its Membership root, then draws the Organisation key pair
(LLR-9qavm7), and adds a new store holding the Persona (`org_id` `None`,
`handle` set), `org_name`, and `founding: Some(ProvisionalFounding { epoch:
1, members, root, org_pub_key, org_private_key, proxy: None })`; it keeps no
current record and calls no chain operation. The secret is returned by value
for org-io's keychain; org-node keeps no copy, and the Persona records the
Device public key and the member seed, never the Device seed.
satisfies: REQ-4qh2h8, REQ-kd34f5, REQ-ky5tyd
supersedes: LLR-e5kqhg LLR-s6qnht LLR-68yd3j LLR-tev8h8

*Amended in place 2026-10-21 (unmerged draft; design gap G2 of the plan):*
supersedes LLR-tev8h8 (`create_persona` draws an independent member keypair
and Device seed, keeps the member seed and, since S4, the Device public key
and no Device seed, and returns the Device secret by value), whose operation
is removed; with LLR-6acxu9 this item states the same draws for
each new Persona, and the last clause makes its "records the Device public
key and no seed" explicit.

**LLR-9qavm7**: `propose_founding` draws the Organisation key pair after the
founding root is calculated and refuses with
`DuplicateOrgPublicKey` a public key equal to the founder's
Member-as-a-group key or DevicePublicKey, adding no store and returning no
secret.
satisfies: REQ-4qh2h8, REQ-ech45n
supersedes: LLR-sj7cd5

*Amended in place 2026-10-28 (unmerged draft; round 2 review, AR2-8):* a
drawn Organisation public key equal to a key already held is org-node's own
refusal, `DuplicateOrgPublicKey`, not org-members' `Trie(DuplicateKey)`
passed through (ARCHITECTURE_GUIDELINES Boundaries 2); `OrgPublicKey::
ensure_distinct_from` refuses with it too. `Trie(..)` stays for failures
org-members itself detects, as its cause.

*Noted 2026-10-28 (unmerged draft; round 5 TEST review, finding-2, owner
ruling of that day):* this item's DevicePublicKey clause is checked at
LLR-c759yd, the comparison it refuses through: `propose_founding` cannot
reach it, as a drawn X25519 key equals the founder's ed25519
DevicePublicKey only by a 2^-128 accident no boundary fake can arrange. Its
Member-as-a-group clause is checked through `propose_founding`.

**LLR-nx87k5**: `OrgService::record_founding_proxy(rng, device_key, org_id:
OrgId, proxy: ChainAccount) -> Result<StoreChange, OrgNodeError>` sets the
founding's `proxy` to `Some(proxy)` and `persona.org_id` to `Some(org_id)`;
it refuses with `NoProvisionalFounding` a store holding no provisional
founding, and with `FoundingProxyAlreadyRecorded` one whose founding already
holds another proxy or whose `org_id` is another identifier; the same proxy
and identifier again is accepted and returns the store sealed unchanged in
content, so a retried call after a lost write succeeds.
satisfies: derived

**LLR-6acxu9**: `OrgService::propose_member(rng, invite: &Invite, name:
&str, surname: &str, handles: &[&str]) -> Result<(DevicePublicKey,
DeviceSecret, StoreChange), OrgNodeError>` parses the details and Handles
(LLR-rjphn4), draws a fresh Device seed and member seed for this
Organisation alone, and adds a new store holding the Persona (`org_id`
`Some(invite.org_id)`, `handle` `None`), `org_name` from the Invite, `member:
Some(ProvisionalMember { org_id, invite_id, issuer: invite.issuer,
issuer_addresses, name, surname, preferred_handles, member_key, device_key })`
and one `ExpectedAdmission { org_id }`; it calls no chain operation. As
for `propose_founding` (LLR-d4gpxm), the secret is returned by value for
org-io's keychain, org-node keeps no copy, and the Persona records the
Device public key and the member seed, never the Device seed.
satisfies: REQ-hx4r8y, REQ-kd34f5, REQ-ky5tyd
supersedes: LLR-7q9g9z LLR-9zfnmb LLR-tev8h8

*Amended in place 2026-10-21 (unmerged draft; design gap G2 of the plan):*
supersedes LLR-tev8h8 together with LLR-d4gpxm: a provisional Member's
Persona is the other Persona `create_persona` made, and the last sentence
states for it what LLR-tev8h8 stated of the keys.

**LLR-rjphn4**: `propose_founding` and `propose_member` parse the name, the
surname and each Handle with `Name::parse`, `Surname::parse` and
`Handle::parse` (NFC, as stored), in that order, and `propose_member` then
checks the preferred Handles are one to three and pairwise distinct; the
first failure is refused with `InvalidField` whose `field` is
`member.name`, `member.surname`, `member.handle` (founding) or
`member.preferred_handles[i]` / `member.preferred_handles` (count or
repeat), before any draw from `rng`, adding no store.
satisfies: REQ-kd34f5
supersedes: LLR-q6n25z

**LLR-w2uyyh**: `propose_member` refuses with
`OrganisationHeldByAnotherPersona { org_id }`, before any draw and adding no
store, an Invite whose Organisation a held store is already bound to
(`persona.org_id`) or expects admission to; the variant's `Display` names
the Organisation and no Persona.
satisfies: REQ-uy7kfq
supersedes: LLR-yexf5m

**LLR-bwtx3h**: `OrgService::record_admission(rng, device_key, invite_id,
chosen: Option<&str>) -> Result<StoreChange, OrgNodeError>` takes the kept
admission request for `invite_id` and gives it, in their ranked order, the
first of its preferred Handles held neither by a member of the current
record nor by the `handle` of another `Admit` change in
`provisional_changes`; when none is free it parses `chosen` and gives that,
refused as LLR-mq7n6u states; it then appends `Admit { invite_id, handle }`.
satisfies: REQ-2nnsmv, REQ-dhr58q
supersedes: LLR-rb8r65 LLR-kkj64b

**LLR-mq7n6u**: `record_admission` refuses, recording nothing: an
`invite_id` with no kept admission request (`NoAdmissionRequestKept`); an
`invite_id` an `Admit` change already names (`AdmissionAlreadyRecorded`); no
free preferred Handle and `chosen` `None` (`NoFreeHandle`); a `chosen` that
does not parse (`InvalidField { field: "admission.handle" }`) or that is held
as LLR-bwtx3h defines (`HandleTaken`); a store with no current record
(`OrgNotHeld`); the behind marker set (LLR-zk7fx2); and the bound
(LLR-m76r4k).
satisfies: REQ-2nnsmv, REQ-akfwf3, REQ-g2537c

**LLR-f3hru6**: `OrgService::record_revocation(rng, device_key, member_id)
-> Result<StoreChange, OrgNodeError>` appends `Revoke { member_id }`; it
refuses, recording nothing, a MemberId the current record does not hold
(`MemberNotHeld`), one a `Revoke` change already names
(`RevocationAlreadyRecorded`), a store with no current record (`OrgNotHeld`),
the behind marker set and the bound; it builds no trie and draws nothing.
satisfies: REQ-dhr58q, REQ-akfwf3
supersedes: LLR-6dc598

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, the
plan's open low, confirmed by merge review MR1):* "draws nothing" reads
"draws nothing from `rng` beyond its store seal's nonce": the `StoreChange`
it returns is the store sealed again, and the seal draws its nonce from
`rng`. It draws no MemberId and no key.

**LLR-va2yqz**: `OrgService::propose_updated_organisation(rng, device_key)
-> Result<StoreChange, OrgNodeError>` rebuilds the trie of the current
record and applies every provisional change in recorded order — an `Admit`
adds a member under a MemberId drawn from `rng` with the kept request's name,
surname, Member-as-a-group key and DevicePublicKey and the change's Handle; a
`Revoke` deletes that member — calculates the resulting Membership root once,
encodes the Change set from the current record to the result, and keeps an
`Own` provisional organisation with `epoch` the record's epoch plus one,
`base_root` the record's root, `base_snapshot` the record's members,
`applied` a copy of `provisional_changes` as they were applied, and the
key pair of LLR-qghs95. The provisional changes stay held; the current
record is unchanged.
satisfies: REQ-3pj7rd
supersedes: LLR-ghja3x LLR-tax3pm

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* keeps the
applied changes with the provisional organisation (LLR-s3vq6d).

**LLR-qghs95**: `propose_updated_organisation` draws one Organisation key
pair from `rng` after the resulting root is calculated and refuses with
`DuplicateOrgPublicKey`, keeping nothing, a public key equal to
the record's current Organisation public key or to any Member-as-a-group key
or DevicePublicKey of the resulting record (`OrgPublicKey::ensure_distinct_from`).
satisfies: REQ-3pj7rd
supersedes: LLR-e2b7gv

*Amended in place 2026-10-28 (unmerged draft; round 2 review, AR2-8):* a
drawn Organisation public key equal to a key already held is org-node's own
refusal, `DuplicateOrgPublicKey`, not org-members' `Trie(DuplicateKey)`
passed through (ARCHITECTURE_GUIDELINES Boundaries 2); `OrgPublicKey::
ensure_distinct_from` refuses with it too. `Trie(..)` stays for failures
org-members itself detects, as its cause.

**LLR-4rv4kr**: every `MemberId` org-node gives a Member is drawn from the
caller's random source before that Member's leaf is built — the founder's
by `propose_founding` (LLR-d4gpxm), each admitted Member's by
`propose_updated_organisation` when it applies an `Admit` change
(LLR-va2yqz) — and is never computed from a key, so the same
Member-as-a-group key and DevicePublicKey admitted twice, in two proposals,
get two different identifiers, and a deleted identifier is never issued
again.
satisfies: REQ-d9g6nt
supersedes: LLR-ag9mgm

*Added 2026-10-21 (unmerged draft; design gap G2 of the plan):* LLR-ag9mgm
stated this of `admit_member`, which this change removes; a Member is now
given its identifier when the provisional organisation that admits it is
calculated, and the founder when the founding is proposed, and the rule is
unchanged.

**LLR-n6ssue**: `propose_updated_organisation` refuses, keeping nothing: a
store holding no provisional change (`NoProvisionalChange`), no current
record (`OrgNotHeld`), the behind marker set (`OrganisationBehind`), a change
org-members refuses to apply (`Trie`), and the bound (LLR-m76r4k).
satisfies: REQ-3pj7rd, REQ-akfwf3, REQ-g2537c

## Verification

**SDD-kegag2**: verification of one Organisation against the chain. Code: `org-node/src/verify_org.rs` (`judge`, `adopt`) and `OrgService::verify` in `org-node/src/service.rs`. Given the Organisation state org-io read at a finalised block, it decides exactly one outcome, carried out on the store: on adoption the current record is replaced; on the Persona's own removal the store goes to SDD-z4m57w's removal. It takes over the commit responsibility SDD-8cpyfa describes for `commit_update`, `commit_genesis`, `commit_checked`, `reconcile`, all removed.
traces: REQ-dwb9bq, REQ-x2dnee, REQ-eqt23j, REQ-nbx4nn, REQ-2yduet, REQ-akfwf3, REQ-9ktgwx, REQ-tqap3r, REQ-jy6ybw, REQ-ju6vn2, REQ-kt877x, REQ-m2xh8q, REQ-4qh2h8, REQ-48zc3e, REQ-y9p27b
supersedes: SDD-8cpyfa

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR3-N1):* the code also holds `verify_org::discard_unadoptable` and
`AheadLimit`: before the outcome is decided, what can never be adopted is
discarded (LLR-4tsgfh). It is part of the same responsibility, the store a
verification leaves; no edge is added.

*Amended in place 2026-10-28 (unmerged draft; round 3 review AR3-5, sweep
for ARCHITECTURE Boundaries 3):* reworded so deciding the outcome and carrying it out read as the one responsibility they are, a verification's outcome (adoption is the outcome carried out, not a second job); its code and LLRs are unchanged.

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q4' and
dispatcher's resolution C6 of 2026-10-16):* REQ-uxv2x2, REQ-y99c9w and
REQ-em28bq are superseded; the item traces their successors REQ-48zc3e and
REQ-y9p27b, and supersedes SDD-8cpyfa together with SDD-busuj3, whose
chain-free checks that item also held.

**LLR-rr558z**: `OrgService::verify(rng, device_key, chain: Option<OrgState>,
verified_at: VerificationTime, device_secrets: impl FnOnce(&[DevicePublicKey])
-> Vec<DeviceSecret>) -> Result<(Verified, Option<StoreChange>),
OrgNodeError>` refuses a founding store with no proxy recorded with
`FoundingNotWritten`, checked first, then `None` with `OrgNotOnChain`, and
otherwise returns exactly one `Verified`: with a
current record, `chain.epoch` lower than its epoch → `Inconsistent {
reason: ChainBehindRecord { chain_epoch, record_epoch } }`; equal epoch and
another root → `Inconsistent { reason: OtherRoot { epoch } }`; then a kept
revocation notice accepted against `chain` (LLR-jhy6t6) → `Removed { org_id,
acknowledgements_kept }`; then equal epoch and equal root → `InStep`;
greater epoch → the match of LLR-8528ud,
giving `Adopted { epoch, discarded, own }`, or `Removed { org_id,
acknowledgements_kept }` when the matched record lists the Persona's Device
not (LLR-jhy6t6), else `Behind { chain_epoch }`. With no current record (a
founding or provisional Member store) only the match or `Behind` is
possible. `VerificationTime(u64)` is the caller's time of the verification
in seconds since the Unix epoch; `verify` reads no clock and keeps it only
on a removal (LLR-2d5a7m).
satisfies: REQ-dwb9bq, REQ-48zc3e
supersedes: LLR-gr8x3r LLR-3wb7th REQ-bvh8v6 LLR-8m99q2 LLR-vf5mjx

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR3-N1, owner decision on rule 4 of 2026-10-30):* `verify` also takes
`ahead_limit: AheadLimit` after `verified_at`, the caller's limit on how
long a received provisional organisation one epoch ahead of the chain is
kept unadopted; with a chain state, LLR-4tsgfh's discard runs before the
judgement above, which is made on what it leaves.

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR4 finding-2, owner ruling and clarification 2026-10-30):* the
`ChainBehindRecord` case above (C below the record's epoch R) cannot come
from a sound chain source — a current record is never ahead of the on-chain
epoch, which only increases — so it is a stale or faulty read. It is still
reported `Inconsistent`, and LLR-4tsgfh's discard does not run on it:
nothing is discarded or stamped.

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR1-N1, owner ruling 2026-10-29):* supersedes REQ-bvh8v6, LLR-8m99q2,
LLR-vf5mjx, which `verify_envelope_against_chain` alone did; it is deleted,
and this item states what each became (see each one's note).

*Amended in place 2026-10-22 (unmerged draft; dispatcher decision O4, plan
assembly):* supersedes LLR-3wb7th (an Organisation the chain holds no state
for is refused with `OrgNotOnChain`, record unchanged — for a held
Organisation only after the chain-free checks, for a first admission only
after the expectation check, the snapshot decode and the chain-free
checks), whose Receive operations and first-admission path are removed.
This item's `None` → `OrgNotOnChain` is that refusal over `verify`, for a
store with a current record and for a founding or provisional Member store
alike, and LLR-qvp7mq states that it changes nothing; the chain-free checks
it waited for are now made on receipt, before anything is kept
(LLR-76bhw9), so they still come before the chain is consulted.

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* the order of the two refusals is stated:
`FoundingNotWritten` (the store refusal of LLR-3sddey's note, made by
`service::current_record`) comes before `OrgNotOnChain`, so a founding with
no proxy is refused as not written whatever the caller read. It was
silent.

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q4' of
2026-10-16):* `verify` takes the time of the verification, which the removed
store keeps for org-io's 7-day deadline, and `Removed` reports the
Organisation and how many acknowledgements were kept instead of returning
them. A kept notice is tried after the Inconsistent checks, so a state the
record refuses never deletes anything.

**LLR-qvp7mq**: `InStep`, `Behind`, `Inconsistent` and every refusal of
`verify` return no `StoreChange`, leave every sealed byte of the store as it
was and call `device_secrets` never; only the behind marker of LLR-zk7fx2
changes.
satisfies: REQ-dwb9bq, REQ-tqap3r, REQ-y9p27b
supersedes: LLR-ewkg85 LLR-6qjmhf LLR-8hwqru

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR3-N1, owner ruling 2026-10-30):* the one exception to this item is
LLR-4tsgfh's discard and stamp. When a chain state lets it discard a
received or answer provisional organisation, or stamp one one epoch ahead,
`InStep`, `Behind` and `Inconsistent` return one `StoreChange::Sealed`
holding exactly that, and those bytes change; the record, the Persona and
every other field are as they were, and `device_secrets` is still never
called. Every refusal (no chain state among them) still changes nothing.

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR1-N1, owner ruling 2026-10-29):* supersedes LLR-8hwqru, which
`verify_envelope_against_chain` alone did; it is deleted, and this item states
what each became (see each one's note).

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q4' of
2026-10-16):* also satisfies REQ-y9p27b: a state reported behind or
inconsistent, and a call with no chain state, delete nothing.

**LLR-8528ud**: the match for a chain state at a greater epoch is the one
held provisional organisation whose `epoch`, `root` and `org_pub_key` equal
`chain.epoch`, `chain.root_hash` and `chain.org_pub_key` — own, received or
answer alike — or, in a founding store, the provisional founding when
`chain.epoch` is 1, its root and key equal the chain's and its members,
calculated again, give that root; identity
`(root, org_pub_key)` makes at most one match, and because `org_pub_key` is
derived from the held private key (LLR-s3vq6d), a match proves the private
key is the chain key's private half. A provisional organisation with the
chain's epoch and root but another `org_pub_key` is no match: it is not
adopted, and the store is left as the outcome (`Behind`, or the adoption of
the one that matches) states. Which record is adopted is decided by the
chain state and the held provisional organisations alone: nothing a
message carried but the Change set, record snapshot and Organisation
private key it was kept from, and never the Device that delivered it
(origin plays no part in the match).
satisfies: REQ-dwb9bq, REQ-9ktgwx, REQ-2yduet
supersedes: LLR-ba2ejp LLR-37cj3n REQ-ag6kqm LLR-na7p4w LLR-8n95rf

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR1-N1, owner ruling 2026-10-29):* supersedes REQ-ag6kqm, LLR-na7p4w,
LLR-8n95rf, which `verify_envelope_against_chain` alone did; it is deleted,
and this item states what each became (see each one's note).

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* a provisional founding matches only when its members
rebuild its root. The root it holds is not trusted on its own, so a
founding whose members do not give its root is no match and adopts
nothing. It was silent.

*Amended in place 2026-10-20 (unmerged draft; dispatcher's resolution C6 of
2026-10-16):* supersedes LLR-ba2ejp, whose functions are removed: the key
check it made on receipt is this match, at verification.

*Amended in place 2026-10-21 (unmerged draft; design gap G2 of the plan):*
supersedes LLR-37cj3n (which Change set `receive_and_verify` commits is
decided by the chain state and the node's own record, never by a carried
value but the Envelope, the key and a first admission's snapshot, and never
by the delivering Device), whose operation is removed: the added sentence
states the same over the match.

**LLR-34c5y3**: adopting a matched provisional organisation replaces the
current record's members, root, Organisation public and private key, epoch
and Sequence number (= epoch) with the match's, sets the kept Change set and
base snapshot (LLR-p8vpgz), clears the behind marker, and returns
`Adopted { epoch, discarded, own }` with `own` true exactly when the match's
origin is `Own`, and one `StoreChange::Sealed` holding all of the adoption's
effects (LLR-d9x6xf, LLR-x8drb8, LLR-rzqs2u, LLR-bgj2ff), sealed before the
outcome is returned. The current record takes a recorded admission or
revocation only through such an adoption: `record_admission`,
`record_revocation` and `propose_updated_organisation` leave it unchanged
(LLR-b2hgd4, LLR-va2yqz), and `messages_for_current_epoch` changes nothing
(LLR-degg8t).
satisfies: REQ-x2dnee, REQ-tqap3r, REQ-jy6ybw, REQ-ju6vn2
supersedes: LLR-mkj4bz LLR-cmdrp9 LLR-6s785x LLR-fm38ww LLR-4kh9w9 LLR-ckk5nz LLR-sxd3tg LLR-qg9utu LLR-d6kvbx LLR-uc7cej

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR1-N1, owner ruling 2026-10-29):* supersedes LLR-d6kvbx, LLR-uc7cej,
which `verify_envelope_against_chain` alone did; it is deleted, and this item
states what each became (see each one's note).

*Amended in place 2026-10-20 (unmerged draft; dispatcher's resolution C6 and
owner ruling Q3 of 2026-10-16):* supersedes LLR-sxd3tg (the update branch's
record fields, saved before the outcome), which this item states over
`verify`; the sealed image also holds the revoked Devices the adoption adds.

*Amended in place 2026-10-21 (unmerged draft; design gap G2 of the plan):*
supersedes LLR-qg9utu (the revoking node's record takes the removal only
through `commit_update`, once it verified against the chain, and never
through `revoke_member` or `send_update`), whose operations are removed: the
added sentence states the same over adoption.

**LLR-d9x6xf**: on adoption, every provisional change the adopted record
did not include is removed and reported, in recorded order, in `discarded:
Vec<DiscardedChange>` (`Admit { invite_id, handle }` or `Revoke { member_id
}`); an `Admit` is included exactly when the adopted record lists a member
with the kept request's Member-as-a-group key and DevicePublicKey, and a
`Revoke` exactly when the adopted record holds no member with that MemberId;
included changes are removed and not reported. No command discards a change
on request.
satisfies: REQ-x2dnee
supersedes: LLR-7cmp38

**LLR-x8drb8**: on adoption at epoch `e`, every provisional organisation
whose `epoch` is at most `e` is removed with its Organisation private key, the
match included once moved, and every kept revocation notice is removed;
provisional organisations for a later epoch are kept.
satisfies: REQ-x2dnee, REQ-9ktgwx

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR3-N1, owner ruling 2026-10-30):* an adoption is no longer the only
point at which provisional organisations are discarded. Every verification
against a chain state, adopting or not, first discards the received and
answer ones that can never be adopted and ages those one epoch ahead, as
LLR-4tsgfh states; this item's discard at an adoption is unchanged and
applies to what that leaves, `Own` ones included.

**LLR-4tsgfh**: `OrgService::verify`, given a chain state at epoch C, before
judging it (LLR-rr558z) and unless C is below the epoch R of the store's
own record, discards with its
Organisation private key each provisional organisation of origin `Received`
or `Answer` whose `epoch` E is (1) below C; (2) above C + 1; or (3) C with a
root or Organisation public key that is not the chain's (the match rule of
LLR-8528ud); and (4) keeps one at C + 1 for a time: the first verification
that finds it at C + 1 sets its `first_ahead_at: Option<VerificationTime>`
to that verification's `verified_at`, no later verification moves it, and
a verification whose `verified_at` is more than `ahead_limit` seconds after
it discards it while it is still at C + 1 (exactly `ahead_limit` seconds
after, it is kept). A verification whose `verified_at` is earlier than the
stamp (the wall clock stepped back) keeps it and leaves the stamp, which
still ages from its own time. With `AheadLimit(0)` one at C + 1 is kept by
its stamping verification and any other in that same second, and discarded
by the first verification a second or more after the stamp. A chain at
epoch `u64::MAX` has no C + 1: rule 4 keeps nothing and no epoch
arithmetic overflows. Once the chain reaches its epoch it is judged by rule 3
or adopted by the match. `ahead_limit: AheadLimit(u64)`, in the seconds of
`VerificationTime`, is the caller's: org-node reads no clock and holds no
default; the app supplies twice its verification interval, so at least two
chain reads fall inside the limit (org-io passes on the limit the app
handed it when it connected to the chain; 1200 s at the default
interval).
Provisional organisations of origin `Own` (this node's proposals) and the
provisional changes are touched by none of these rules; a provisional
founding is not a provisional organisation and is untouched. With no chain
state (`OrgNotOnChain`), on `FoundingNotWritten`, and when C is below the
epoch R of the store's own record (`Inconsistent { reason:
ChainBehindRecord }`), nothing is discarded or stamped and these rules
change nothing; on an `Inconsistent` read at R itself with another root
(`OtherRoot`, C = R) the rules run as above. When a discard or a stamp changes the store, the outcome
carries one `StoreChange::Sealed` holding it, sealed before the outcome is
returned (for an adoption, in the adoption's one sealed change); otherwise
no store change is returned. These rules add no restriction on the sender: Organisation
information and organisation answers are still kept from any Device by a
store with no record (LLR-e6w3nr), and from a Device the record lists by a
store with one (LLR-283x6f), and a Device whose information is discarded keeps waiting,
its periodic verification and organisation request unchanged.
satisfies: REQ-dwb9bq, REQ-9ktgwx

*Added 2026-10-30 (change `worktree-org-io-domain-api`, merge review
MR3-N1, owner ruling and decision on rule 4 of 2026-10-30):* RC-d4fc4s
promises that a provisional organisation that can never be adopted is
discarded after verification, but a store with no record kept Organisation
information from any Device at any epoch and discarded only at an
adoption, so any Device could fill REQ-g2537c's bound and block a new
Member's admission for good. Rules 1–3 discard what no later chain state
can adopt (the chain's epoch only grows); rule 4 bounds how long a body
for the next epoch can hold space. The pure step is
`verify_org::discard_unadoptable`, taking the store by value (LLR-g2jeqd).

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR4 finding-2, owner ruling and clarification 2026-10-30):* a chain
read behind the store's own record (C < R) runs none of the rules. A
Device's current record is never ahead of the on-chain epoch: that epoch
only increases, and a record becomes current only by verification against
the chain, so the record is at the chain's epoch or behind it. A read with
C < R therefore cannot come from a sound chain source; it is a stale or
faulty read. It is reported `Inconsistent { reason: ChainBehindRecord }`
as before, and because it cannot be true, nothing is discarded or stamped
on it. Before, "whatever the outcome" ran the rules on it, and rule 2
discarded genuine information for R + 1 that a listed Admin sent. At C = R
with another root the rules still run.

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR4 finding-4):* the abnormal cases are stated — a verification
earlier than the stamp, `AheadLimit(0)` and a chain at epoch `u64::MAX`.
Each was the code's behaviour already; the item was silent on them.

*Amended in place 2026-10-31 (change `worktree-org-io-domain-api`, merge
review round 5 finding-3):* the opening read "before judging it and
whatever the outcome, discards", which contradicted the C < R exclusion
the MR4 finding-2 note added later in the item. It now reads "unless C is
below the epoch R of the store's own record". No behaviour change: the
code already ran no rule at C < R. With finding-5, its last sentence's
"kept from any Device" is qualified by store state, as HAZ-cuj9gn now is.

*Amended in place 2026-10-31 (change `worktree-org-io-domain-api`, merge
review round 6 finding-4):* the last sentence opened "Nothing restricts the
sender:" and then stated a restriction, the record's listing. It now opens
"These rules add no restriction on the sender:", which is what it meant;
the store-state qualification follows unchanged. No behaviour change.

**LLR-rzqs2u**: on adoption, for each kept admission request whose
Member-as-a-group key and DevicePublicKey the adopted record lists, the
request and its `(org_id, invite_id)` outstanding pair are removed, so a
later admission request carrying that invite id is refused as not
outstanding (LLR-6gkxpg), and the endpoint addresses the request kept are
moved into the address book under its DevicePublicKey (LLR-h9d65p),
replacing any entry for that key.
satisfies: REQ-nbx4nn, REQ-ky5tyd
supersedes: LLR-d55brn

*Amended in place 2026-10-20 (unmerged draft; owner answer A2 of
2026-10-16):* the admitted Device's addresses outlive its settled request, so
the send path still reaches it.

**LLR-q4puca**: in a provisional Member's store, a matched provisional
organisation is adopted only when its members list one member whose
Member-as-a-group key and DevicePublicKey are the provisional Member's; the
adoption creates the current record, sets `persona.member_id` and
`persona.handle` from that member, and removes the provisional Member and
the expected admission, and moves the Invite issuer's endpoint addresses the
provisional Member kept into the address book under the issuer's
DevicePublicKey (LLR-h9d65p), replacing any entry for that key, in the same
sealed store change. A match that lists them not is refused with
`AdmissionNotOurs { org_id }`, changing nothing. No Invite import is needed.
The created record's Organisation public key is the chain's `org_pub_key`
(equal to the match's, LLR-8528ud), never a value a message carried, and
`OrgRecord` holds no administrator key.
satisfies: REQ-eqt23j, REQ-kt877x
supersedes: LLR-3f5h7b LLR-j83kc8 LLR-mbjfq8 LLR-xq9nrq

*Amended in place 2026-10-20 (unmerged draft; dispatcher's resolution C6 of
2026-10-16):* states LLR-xq9nrq's rule over the first adoption and
supersedes it.

*Amended in place 2026-10-25 (unmerged draft; PR-8sgsb5):* the first
adoption moves the Invite issuer's addresses into the address book. Without
it `addresses_for` (LLR-pgy2rb) lost the issuer once the provisional Member
was removed, and an admitted Member could not reach the Admin to request the
Organisation (owner answer A2 of 2026-10-16: the receiver keeps addresses
per Device key).

**LLR-gcm7bq**: in a founding store, adopting the matched provisional
founding creates the current record with `org_id` and the founding's proxy
as `proxy_account`, epoch and Sequence number 1, its members, root and key
pair, `kept_change_set` and `kept_base_snapshot` `None`, sets
`persona.member_id` to the founder's MemberId, removes the founding, and
reports `Adopted { epoch: 1, discarded: [], own: true }`.
satisfies: REQ-4qh2h8, REQ-tqap3r, REQ-x2dnee
supersedes: LLR-wzqqg9 LLR-pk6mhv LLR-q3aj8z

*Amended in place 2026-10-21 (unmerged draft; design gap G2 of the plan):*
supersedes LLR-q3aj8z, which said `commit_genesis` leaves the founding
Persona's member id `None`, the founder's MemberId held only in the record's
snapshots. That operation is removed, and this item states what the
founding's adoption now does with the Persona's member id: it sets it to the
founder's MemberId, as a first admission sets a joiner's (LLR-q4puca). A
change of meaning, stated here and not by amending the dated item.

**LLR-exaa9u**: a Persona's `status` is `Proposed` from `propose_founding`
or `propose_member` until the first adoption in its store — of the
provisional founding (LLR-gcm7bq) or of a first admission (LLR-q4puca) —
which sets it `Active` and gives it the member id of the one member whose
`device_keys` hold its DevicePublicKey (the founder's, LLR-gcm7bq; the
admitted Member's, LLR-q4puca); no other operation writes `status`, and, a
store holding one Persona (LLR-f9x3vf), no other Persona is changed. The
Organisation the Persona is bound to is fixed as LLR-qqqg4b states and is
never rebound.
satisfies: derived
supersedes: LLR-w3fhhg LLR-e5c9ud

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* supersedes LLR-e5c9ud (the Persona marked Active is
the first in the store whose DevicePublicKey is in the verified trie, among
those the commit may bind, and is given the member id of the member holding
that key and the change's Organisation), whose commit is removed. A store
holds one Persona, so there is no choice among Personas left to state; the
member id clause is added, as the two first adoptions already set it, and
the Organisation is fixed before the adoption (LLR-qqqg4b). Recorded at the
same pass (dispatcher's design call): the plan builds the first adoptions
(T19) with no `status` write and adds the write this item states later
(T60); the item states the change's end state and is unchanged by that
order.

*Added 2026-10-21 (unmerged draft; design gap G2 of the plan):*
LLR-w3fhhg stated this of `commit_genesis` (the named Persona marked Active
and bound, every other Persona as it was, a bound Persona never rebound),
which this change removes; the rule holds over the two first adoptions.

**LLR-t2adzg**: a Persona `propose_founding` creates is recorded `Proposed`
with `org_id` `None`, which only `record_founding_proxy` sets (LLR-qqqg4b),
and no member id; a Persona `propose_member` creates is recorded `Proposed`
with `org_id` `Some(invite.org_id)` and no member id. A Persona's member id
is set only by the first adoption in its store (LLR-exaa9u), and by no
other operation.
satisfies: derived
supersedes: LLR-v82xds

*Added 2026-10-22 (unmerged draft; plan detail pass — dispatcher
decision):* LLR-v82xds said a newly created Persona is recorded Proposed
with no Organisation and no member identifier. That holds of a founding's
Persona, but a provisional Member's Persona is created bound to its
Invite's Organisation (LLR-6acxu9, LLR-qqqg4b), which contradicts it; this
item states the rule for each of the two creating operations.

**LLR-x5asbx**: the proxy account `record_founding_proxy` is given
(LLR-nx87k5) is held as opaque data, in the provisional founding and, from
the founding's adoption, as the record's `proxy_account` (LLR-gcm7bq); it
reaches the disk only inside the sealed store; `OrgService::proxy_account(org_id)`
returns it unchanged — `Some` on a record a founding's adoption created,
`None` on one a first admission created — and org-node reads it only to
hand it back: it derives nothing from it, passes it to no operation, and no
other operation — `record_admission`, `record_revocation`,
`propose_updated_organisation`, an adoption that replaces the record
(LLR-34c5y3) — reads or changes it.
satisfies: derived
supersedes: LLR-dzte8x LLR-3v5nu9 LLR-drgdy8

*Added 2026-10-21 (unmerged draft; design gap G2 of the plan):* LLR-dzte8x
(stored opaquely by `commit_genesis`), LLR-3v5nu9 (handed back by
`proxy_account`) and LLR-drgdy8 (neither read nor changed by
`revoke_member`) state one rule over removed operations; it holds over the
founding's proxy recorded before its write, which org-io reads back to
write each later epoch.

**LLR-jhy6t6**: `verify` decides the Persona's own removal in two cases
only: the match of LLR-8528ud lists no member snapshot holding the Persona's
DevicePublicKey; or, tried in kept order after the Inconsistent checks of
LLR-rr558z, a kept revocation notice is accepted by `revocation::accept`
against `chain` (REQ-m2xh8q, whether the send step pushed the notice or it
came in answer to this Device's organisation request). It then signs the
acknowledgements (LLR-5tvn7g) and seals the removed store (LLR-2d5a7m),
returning `Removed { org_id, acknowledgements_kept }` with that one
`StoreChange::Sealed`. A notice `accept` refuses stays kept and the
judgement continues as LLR-rr558z states (LLR-f9mcsq).
satisfies: REQ-48zc3e, REQ-2yduet, REQ-m2xh8q
supersedes: LLR-b27jr6 LLR-pt32fx

*Amended in place 2026-10-20 (unmerged draft; owner rulings Q3 and Q4' of
2026-10-16):* the removal no longer forgets the store
(`StoreChange::Forgotten`); it seals one that keeps the acknowledgements.
REQ-uxv2x2 and REQ-y99c9w are superseded by REQ-48zc3e and REQ-h29c8e. A
refused notice is no longer dropped, so `Behind` and `InStep` still change no
sealed byte (LLR-qvp7mq); kept notices go on adoption (LLR-x8drb8).

**LLR-g2jeqd**: `verify` and every public function of `revocation.rs` and
`verify_org.rs` take the store as `&StoreData` (or the service's held
store) and the chain state as an `Option<OrgState>` or `OrgState` value, are
not `async`, are generic over no IO trait and call no chain, transport,
endpoint or file-system function; the `absences` test reads their source for
those names. The same holds of `removal.rs` (SDD-z4m57w), which also reads
no clock (no `SystemTime` or `Instant`). So an adoption is sealed and
returned with no send made or awaited, and whether its epoch's messages
(LLR-degg8t, built afterwards from the adopted record) are ever sent changes
neither the outcome nor the store.
satisfies: REQ-dwb9bq, REQ-m2xh8q, REQ-y9p27b, REQ-b462sh, REQ-qrtsc9, REQ-tqap3r
supersedes: LLR-xgefn8 LLR-4tcxsu LLR-t4znbk

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR1, finding-6):* `verify_org::accepted_notice` is now `pub(crate)`
like the rest of `verify_org.rs`, which so has no public function; for
that file the rule holds of every `pub(crate)` function, the ones `verify`
calls, and the `absences` test reads those.

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* supersedes LLR-t4znbk (`send_update` writes nothing
to the store and changes no record, the record already updated by the
commit the send came from, and a failed send leaves that record in place),
whose operation is removed: org-node sends nothing now, and the last
sentence states the same of the adopted record — the store change is the
adoption's alone, sealed before any send, and no send or its failure
changes it. The typed error of a failed send is org-io's.

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q4' of
2026-10-16):* REQ-em28bq is superseded by REQ-y9p27b, which this item
satisfies in its place; `removal.rs` is added, with the clock names.

*Amended in place 2026-10-21 (unmerged draft; design gap G2 of the plan):*
supersedes LLR-4tcxsu (`commit_update` and `commit_genesis` bind no
endpoint and send nothing, so a commit never waits on a send and its
outcome does not depend on one), whose operations are removed: the added
sentence states the same of `verify`'s adoption, and the item now also
satisfies REQ-tqap3r, the requirement LLR-4tcxsu served.

**LLR-7en38k**: org-node's source names no `on_chain_client::` or
`subxt::` path, and no `ChainOps`, `ChainReader`, `read_state`,
`OnChainReader`, `OrgStateCache` or `connect_chain_client`; its manifest has
no `chain` feature and no `subxt`, `on-chain-client`, `async-trait` or `hex`
dependency; org-node presents no operation that writes to the chain; and
the one operation that judges against the chain, `verify`
(with the `revocation::accept` and `verify_org.rs` functions it calls),
takes the Organisation's state as an `Option<OrgState>` or `OrgState` value
and is synchronous (LLR-g2jeqd).
satisfies: REQ-dwb9bq
supersedes: LLR-mn5c2q

*Re-pointed 2026-10-30 (change `worktree-org-io-domain-api`, merge review
round 2, finding-4).* This item satisfied REQ-bvh8v6 alone, superseded by
LLR-rr558z. It now satisfies REQ-dwb9bq, which takes the Organisation state
the caller read as given and, as amended that day, refuses with none.

*Added 2026-10-21 (unmerged draft; design gap G2 of the plan):* LLR-mn5c2q
listed `commit_genesis`, `commit_update`, `reconcile` and `commit_checked`
as the operations that judge against the chain, all removed by this change;
its absences hold unchanged, and the list is now `verify` alone. The
no-chain-write clause is the one LLR-65py3d also stated, which LLR-mn5c2q's
note of 2026-10-08 kept in force through its test.

**LLR-9jn3z8**: `StoreData::into_removed` (LLR-2d5a7m) is called from exactly
one place, `verify`'s removal (LLR-jhy6t6), and a kept acknowledgement is
removed in exactly one, `acknowledgement_accepted` (LLR-85kves);
`OrgService::drop_store` stays org-io's local-IO exception; no other
function removes a current record other than by replacing it with an adopted
one (LLR-34c5y3), or a Persona's keys or store. A source scan in the `absences` test lists the
callers of each.
satisfies: REQ-y9p27b
supersedes: LLR-jd82t6 LLR-vw2jn6

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q4' of
2026-10-16):* `forget_organisation` becomes `into_removed`, which keeps the
acknowledgements; the acknowledgement deletion is the one other deleting
path REQ-y9p27b allows. Supersedes LLR-vw2jn6 (verification before anything
is decided about the node's own membership), which this states over
`verify`.

## The node's own removal

*Added 2026-10-20 (unmerged draft; owner ruling Q4' of 2026-10-16, the
owner's rulings of 2026-10-20).* A Persona that verifies its own removal
deletes at once everything it holds for its Organisation except the signed
acknowledgements it must send, their recipients' keys and addresses and the
time of the verification. The store file and the keychain item stay, so
org-io can send the acknowledgements under the revoked Device's identity;
org-io deletes them once none is left, and at the latest after 7 days.

**SDD-z4m57w**: the removed store, from the node's own removal until its last acknowledgement is accepted. Code: `org-node/src/removal.rs` (new: `StoreContent`, `RemovedStore`, `KeptAcknowledgement`, `StoreData::into_removed`, the recipient choice) and `OrgService::acknowledgement_accepted` and `OrgService::kept_acknowledgements` in `org-node/src/service.rs`. When verification finds the Persona removed, the store is replaced, in one sealed image, by a removed store holding only the signed acknowledgements, each with its recipient's DevicePublicKey and addresses, with the verification time; afterwards an acknowledgement its recipient accepted is deleted, what is still kept is listed. It reads no clock; it removes no file or keychain item, which are org-io's.
traces: REQ-48zc3e, REQ-y9p27b, REQ-h29c8e, REQ-6tepxq, REQ-ayyt5f, REQ-35s4eb, REQ-m2xh8q, REQ-2yduet
supersedes: SDD-72ddm6

*Amended in place 2026-10-28 (unmerged draft; round 3 review AR3-5, sweep
for ARCHITECTURE Boundaries 3):* it read "the node's own removal and what it keeps", which names two subjects; it is reworded to the one responsibility its code holds, the removed store over its life. Its code and LLRs are unchanged.

**LLR-2d5a7m**: `StoreData::into_removed(self, acknowledgements:
Vec<KeptAcknowledgement>, verified_at: VerificationTime) -> Result<RemovedStore,
OrgNodeError>` consumes the store and returns `RemovedStore { org_id,
verified_at, acknowledgements }`, holding nothing else: the current record (members,
root, kept Change set and base snapshot, Organisation key pair, proxy
account), the Organisation name, every provisional change, provisional
organisation and the provisional founding, the provisional Member, every
expected admission, outstanding invite, kept admission request and kept
revocation notice, the revoked Devices, every address-book entry (those of
the chosen recipients copied into their acknowledgements first,
LLR-vq5d5n), and the Persona record with its member seed and keys are
dropped with it. `verify`
replaces the held store's content with the `Removed` form (LLR-6zaeb6) and
returns it as its one `StoreChange::Sealed`; no other store the service
holds is read or changed (LLR-f9x3vf).
satisfies: REQ-48zc3e
supersedes: LLR-pba7yu LLR-6p4pj2 LLR-jwhzh3

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* "each Organisation private key zeroised on drop" is
dropped. The owner ruling of 2026-10-04 (org-node's type-safety
requirements, `2026-10-04-type-safety.md`, and the `secret_type!` comment
in `org-node/src/types.rs`: "Wiping on drop is not done (owner ruling
2026-10-04, RC-jjsz97 residual)") leaves wiping secret memory on drop with
RC-jjsz97's store-hardening residual, because the plaintext also passes
through buffers no newtype owns; this item does not reverse it. The removed
data are dropped, not wiped; the deletion REQ-48zc3e asks for is of the
sealed store's content, which the `Removed` form no longer holds.

*Amended in place 2026-10-30 (merge review round 3, finding-7):* the
signature returns `Result`, and a store holding no record is refused with
nothing returned: `OrgNotHeld { org_id }` when its Persona is bound to an
Organisation, `OrgNotOnChain` when it is bound to none. `verify`, the only
caller, reaches `into_removed` only for a store holding a record, so neither
refusal is reachable through it; they are stated because the function is
`pub`.

**LLR-6zaeb6**: a Persona store's plaintext is `StoreContent`, encoded by
postcard as its variant index followed by its fields: `Active(StoreData)`
(0) or `Removed(RemovedStore)` (1). `RemovedStore` holds, in this order,
`org_id: OrgId`, `verified_at: VerificationTime` and `acknowledgements:
Vec<KeptAcknowledgement>`; `KeptAcknowledgement` holds `ack:
Acknowledgement`, `recipient: DevicePublicKey` and `recipient_addresses:
Option<EndpointAddresses>`. `PersonaStore::unseal` re-parses a `Removed`
form's keys and addresses with their types, refusing a value the parse
refuses with `InvalidField` naming `removed.*`, and an unknown variant index
with `InvalidField { field: "store" }`, opening nothing.
satisfies: REQ-48zc3e, REQ-35s4eb

**LLR-myfk8w**: on a store in the `Removed` form every `OrgService`
operation other than `kept_acknowledgements`, `acknowledgement_accepted`,
`drop_store` and sealing refuses with `OrganisationRemoved { org_id }`,
changing nothing and returning no `StoreChange`; `verify` so refused calls
`device_secrets` never. A removed store therefore never again holds a
record, a key or any other data of the Organisation.
satisfies: REQ-48zc3e, REQ-y9p27b

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR1, finding-5):* two operations answer a removed store otherwise,
and are exceptions, each changing nothing and returning no `StoreChange` as
the rule requires. `check_received` bounds the body before it reads the
store's form, so an oversized body to a removed store is `MessageTooLarge
{ length }` (the bound comes first for every store, LLR-94rzrk).
`proxy_account(org_id)` finds the store by the record it holds, which a
removed store does not, so a removed Organisation is `OrgNotHeld { org_id }`.
`persona_public_keys` is the third exception, documented at the function
(`PersonaIdNotHeld`). The code is unchanged: this states what it does.

**LLR-f9mcsq**: on a store in the `Active` form, nothing but the removal of
LLR-jhy6t6 deletes the current record, its Organisation private key, the
Persona's keys or any data the other operations do not name: every refusal
of every `OrgService` operation leaves the store as it was; a kept
revocation notice `revocation::accept` refuses stays kept, `device_secrets`
is not called for it, and the judgement continues; an adoption whose record
lists the Persona's Device removes only what LLR-d9x6xf, LLR-x8drb8 and
LLR-rzqs2u state; `InStep`, `Behind` and `Inconsistent` remove nothing
(LLR-qvp7mq); `check_received` removes nothing on any outcome; and
`cancel_invite` removes only what LLR-ag8hjd states. The test drives each
against a store holding a record, a provisional change and organisation, a
kept admission request and revocation notice, an outstanding invite, an
address-book entry and a revoked Device, and finds every other field
unchanged.
satisfies: REQ-y9p27b, REQ-m2xh8q
supersedes: LLR-jsx922 LLR-6qmq2g LLR-uw7nmv

**LLR-5tvn7g**: before `into_removed`, `verify` signs the acknowledgement:
when the count of LLR-vq5d5n is not zero it calls `device_secrets` once with
`[persona.device_key]`, takes the returned secret whose `device_key()` is
the Persona's, and signs one `Acknowledgement { org_id, member_id, device,
epoch: chain.epoch, root: chain.root_hash }` naming the Persona's MemberId
and DevicePublicKey (LLR-bs6yxf; Ed25519, deterministic, drawing nothing
from `rng`), kept once for each chosen recipient. When no such secret is
returned it refuses with `DeviceSecretNotSupplied { org_id }`, returning no
`StoreChange` and leaving the store as it was. When the count is zero it
calls `device_secrets` never and signs nothing. Every secret is moved in and
dropped, zeroised, before `verify` returns, on success and on refusal; none
is cloned, put in the store or returned.
satisfies: REQ-h29c8e
supersedes: LLR-hby4jr LLR-r8qhky

**LLR-vq5d5n**: the candidates for a removal are the distinct
DevicePublicKeys the member snapshots of the current record held before the
removal list, other than the Persona's own; with `n` of them, `k` is 3 when
`n` is five or more, 2 when three or four, 1 when one or two, and 0 when
none. The recipients are the first `k` candidates in this order: on a
removal by an adopted match, first the candidates the matched record still
lists, then the others; on a removal by a notice, all of them; each group in
ascending order of the key's 32 bytes. Each recipient is distinct, and its
`recipient_addresses` are its address-book entry (LLR-h9d65p), or `None`
when the store keeps none for it.
satisfies: REQ-6tepxq

**LLR-85kves**: `OrgService::acknowledgement_accepted(rng, device_key,
recipient: DevicePublicKey) -> Result<(usize, StoreChange), OrgNodeError>`,
on a store in the `Removed` form, removes the kept acknowledgement whose
`recipient` is `recipient`, with that key and its addresses, and returns
the number still kept and the store sealed; it refuses with
`AcknowledgementNotKept { recipient }`, changing nothing and returning no
`StoreChange`, a recipient no kept acknowledgement names and a store in the
`Active` form. It does not judge the recipient's answer, which org-io
received.
satisfies: REQ-ayyt5f

**LLR-75sn9g**: `OrgService::kept_acknowledgements(&self, device_key) ->
Result<Option<KeptAcknowledgements>, OrgNodeError>` returns `None` for a
store in the `Active` form and, for one in the `Removed` form,
`KeptAcknowledgements { org_id, verified_at, acknowledgements:
Vec<(OutgoingMessage, Option<EndpointAddresses>)> }`, one per kept
acknowledgement in kept order, each message's `recipient` the
acknowledgement's recipient and its `body` the encoded
`WireMessage::Acknowledgement`; it refuses an unknown store
(`PersonaNotHeld`), changes nothing, and equal stores give equal results.
satisfies: REQ-35s4eb

**SDD-73emgx**: revocation notices and acknowledgements. Code: `org-node/src/revocation.rs`. It builds a revocation notice — for each Device an adopted epoch removed, for the send step, and for a Device held as revoked that asks for the Organisation, each proving the Device's absence from the current record's trie; checks a received notice with no chain state and decides a kept one against the chain state the caller read; signs this Device's acknowledgement of its own removal under a domain of its own; and checks a received acknowledgement with no chain state. Pure functions over values: no chain read, no transport, no file, no clock.
traces: REQ-ps2gy2, REQ-qrtsc9, REQ-m2xh8q, REQ-b462sh, REQ-h29c8e, REQ-p4hwuv, REQ-ea4qs5
supersedes: SDD-uck4tz
superseded-by: SDD-gatf4h, SDD-8zzmav

*Superseded 2026-10-28 (unmerged draft; round 2 review AR2-11,
ARCHITECTURE Boundaries 3):* the item stated two responsibilities, the
revocation notice and the removal acknowledgement's signature, each with its
own reason to change (what a notice proves and when it is acted on; what an
acknowledgement signs). It is split into SDD-gatf4h (the notices) and
SDD-8zzmav (the acknowledgement's signature), both still in
`org-node/src/revocation.rs`, each owning the functions and types named in
its Code line. Its LLRs are re-pointed: LLR-rhsfw9 (below), LLR-kr5t6f and
LLR-r7zm39 (2026-10-07-commit-workflow.md) are SDD-gatf4h's; LLR-bs6yxf
(below) and LLR-5azhry (2026-10-07-commit-workflow.md) are SDD-8zzmav's. No
behaviour changes.

### The removal acknowledgement's signature (successor of SDD-73emgx)

**SDD-8zzmav**: the removal acknowledgement's signature: a removed Device's
signed statement of its own removal, signed on this Device under a domain of
its own or checked on receipt with no chain state. Code:
`org-node/src/revocation.rs` (`Acknowledgement`, `ACK_DOMAIN`,
`Acknowledgement::signed_bytes`, `Signature64`, `sign_acknowledgement`,
`check_acknowledgement`, `VerifiedAcknowledgement`). Pure functions over
values: no chain read, no transport, no file, no clock.
traces: REQ-h29c8e, REQ-b462sh
supersedes: SDD-73emgx

*Added 2026-10-28 (unmerged draft; round 2 review AR2-11):* SDD-73emgx's
acknowledgement half. LLR-bs6yxf and LLR-5azhry are its LLRs; the signing
`verify` does on a removal is LLR-5tvn7g's (SDD-z4m57w), through
`sign_acknowledgement`.

**LLR-bs6yxf**: the bytes an `Acknowledgement` signs are `ACK_DOMAIN`
(`b"ods/org-node/revocation-acknowledgement/v1"`) followed by the postcard
encoding of the tuple `(org_id, member_id, device, epoch, root)`;
`Acknowledgement::signed_bytes(&self) -> Vec<u8>` returns them, and
`signature` is the 64-byte Ed25519 signature over them. No other message
org-node signs or verifies begins with `ACK_DOMAIN`.
satisfies: REQ-h29c8e, REQ-b462sh
supersedes: LLR-gbe9bt

### Revocation notices (successor of SDD-73emgx)

**SDD-gatf4h**: revocation notices: proving to a Device an adopted epoch
removed that it is absent from the current record, from building its notice
to deciding a kept one against the chain state the caller read. Code:
`org-node/src/revocation.rs` (`RevocationNotice`, `notices_for`,
`check_notice`, `accept`). A notice is built for each Device an adopted
epoch removed, for the send step, and for a Device held as revoked that asks
for the Organisation; a received one is checked with no chain state. Pure
functions over values: no chain read, no transport, no file, no clock.
traces: REQ-ps2gy2, REQ-qrtsc9, REQ-m2xh8q, REQ-p4hwuv, REQ-ea4qs5
supersedes: SDD-73emgx

*Added 2026-10-28 (unmerged draft; round 2 review AR2-11):* SDD-73emgx's
notice half. LLR-rhsfw9, LLR-kr5t6f and LLR-r7zm39 are its LLRs.

**LLR-rhsfw9**: `revocation::accept(store: &StoreData, notice:
&RevocationNotice, chain: Option<OrgState>) -> Result<(), OrgNodeError>`
only decides: it runs `check_notice` first and returns its refusal
unchanged; then refuses with `OrgNotOnChain` when `chain` is `None`; with
`StaleChainState { org_id, chain_epoch, record_epoch }` when `chain.epoch`
is lower than the record's epoch; with `ChainStateConflict { org_id }` when
`chain.epoch` equals the record's and `chain.root_hash` is not the record's
root; and with `RevocationProofRefused { org_id, cause }`, `cause` the
org-members error, when `notice.proof.verify(&chain.root_hash,
&notice.member_id, &notice.device)` (the hasher the node's tries use)
refuses — root mismatch, or the Device still held — and returns `Ok(())`
otherwise. It takes no seed or secret source, signs nothing and changes
nothing; the signing that follows an accepted notice is `verify`'s
(LLR-jhy6t6, LLR-5tvn7g). So a notice is acted on only against a chain
state of a later epoch than the record's, under whose root the Device is
absent.
satisfies: REQ-m2xh8q
supersedes: LLR-tx8ruv

*Added 2026-10-22 (unmerged draft; plan detail pass — dispatcher
decision):* LLR-tx8ruv stated `accept` with a `device_seeds` source called
just before signing and returning `Accepted`; under this change `accept`
only decides and `verify` signs (LLR-jhy6t6, LLR-5tvn7g), so the signature
loses the seed source and returns `()`. Every refusal of LLR-tx8ruv, as
amended on 2026-10-07 (`ChainStateConflict`), is kept, in the same order.

## The receive path

**SDD-busuj3**: one chain-free receive path. Code: `OrgService::check_received` and `Checked` in `org-node/src/service.rs`. It decides from a received body, the receiving Persona's store and the DevicePublicKey the transport authenticated what is kept, answered, settled or refused; it never adopts and takes no chain state.
traces: REQ-ncv4sc, REQ-9ktgwx, REQ-6qxesy, REQ-p4hwuv, REQ-eqt23j, REQ-8amu2a, REQ-uk9rw7, REQ-ea4qs5, REQ-zxgd46, REQ-b462sh, REQ-g2537c, REQ-2yduet, REQ-f2k4tr, REQ-ky5tyd, REQ-y9p27b, REQ-rv4dwb
supersedes: SDD-esg2kc SDD-8cpyfa

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review round 2, MR2-N1):* traces REQ-rv4dwb, the bound on receipt of any
body (LLR-7u3dg7), which supersedes REQ-bcxz96.

*Amended in place 2026-10-20 (unmerged draft; owner rulings Q3 and A2,
dispatcher's resolution C6 of 2026-10-16):* it also answers an organisation
request from a Device held as revoked with its notice, keeps the senders'
addresses, and holds the chain-free checks of SDD-8cpyfa, which it
supersedes with SDD-kegag2; it deletes nothing (REQ-y9p27b).

*Amended in place 2026-10-21 (unmerged draft; owner decision of
2026-10-20):* traces REQ-zxgd46, which supersedes REQ-c29s93 (RC-5gbgwb's
receipt half), in its place.

**LLR-94rzrk**: `OrgService::check_received(rng, device_key, sender:
DevicePublicKey, body: &[u8]) -> Result<(Checked, Option<StoreChange>),
OrgNodeError>` takes no `OrgState`; it refuses a body over `MAX_MESSAGE` or
that does not decode (`MessageTooLarge`, `MalformedMessage`), then applies
the sender rule (LLR-283x6f, LLR-w7b9kd) and returns one of `Checked::
ProvisionalOrganisationKept { epoch } | AlreadyHeld | RevocationNoticeKept |
AdmissionRequest { outcome } | OrganisationRequest { answer:
OutgoingMessage } | OrganisationAnswerKept { epoch } | Acknowledged(..)`; an
acknowledgement is decided as LLR-5wg8cu states. Every refusal returns no
`StoreChange`; an `OrganisationRequest` returns one only when it changed the
sender's address-book entry (LLR-h9d65p).
satisfies: REQ-ncv4sc, REQ-zxgd46
supersedes: LLR-zetn7a LLR-xn5pwc

*Amended in place 2026-10-20 (unmerged draft; owner answer A2 and
dispatcher's resolution C6 of 2026-10-16):* the acknowledgement arm is
LLR-5wg8cu, which supersedes LLR-3aysup; an organisation request may now
change the store, by its sender's addresses.

*Amended in place 2026-10-21 (unmerged draft; owner decision of
2026-10-20):* satisfies REQ-zxgd46, which supersedes REQ-c29s93, in its
place: the decode refusal (`MalformedMessage`) of a message with no key or a
key that is not 32 bytes is made here, before anything is kept.

*Amended in place 2026-10-21 (unmerged draft; design gap G2 of the plan):*
supersedes LLR-xn5pwc (an Organisation-information message with no
Organisation private key, or a key shorter than 32 bytes, refused as
`MalformedMessage` before any record or expectation is consulted and with no
chain read), whose `receive_one`, `receive_and_verify` and
`receive_and_self_delete_if_revoked` are removed and whose requirement,
REQ-c29s93, is superseded by REQ-zxgd46, which this item satisfies; the
refusal is this item's decode step, which consults no record and takes no
chain state.

*Amended in place 2026-10-28 (unmerged draft; round 2 review, C2-N1):* the
admission request's result is named `AdmissionRequestOutcome` (field
`outcome`), not a verdict: the glossary's Verdict is the byte org-io writes
back, which this outcome only chooses.

**LLR-283x6f**: in a store with a current record, Organisation information,
a revocation notice and an organisation answer are acted on only when
`sender` is in a member snapshot's `device_keys` of the current record, and
an organisation request only when it is there or is held as revoked
(LLR-bgj2ff, answered as LLR-j338wq states); any of them from another
Device is refused with `SenderNotListed { org_id }` before its Change set,
snapshot, record or addresses are used, keeping nothing, and an
organisation request so refused yields no answer. The sender is judged
against the current record alone, on receipt: no record a kept Change set
or answer would lead to is consulted about it, whether or not that record
lists the sender, and verification, which adopts, never consults the Device
that delivered what it adopts (LLR-8528ud).
satisfies: REQ-ncv4sc, REQ-uk9rw7, REQ-ea4qs5, REQ-9ktgwx, REQ-p4hwuv
supersedes: LLR-2r2fha LLR-kzgjz8 LLR-3q63zv LLR-u6rq4s LLR-9fvb3y LLR-mcdh85

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR1-N1, owner ruling 2026-10-29):* supersedes LLR-mcdh85, which
`verify_envelope_against_chain` alone did; it is deleted, and this item states
what each became (see each one's note).

*Amended in place 2026-10-22 (unmerged draft; dispatcher decision O4, plan
assembly):* supersedes LLR-9fvb3y (a genuine Envelope relayed by a device
other than the one that built it is committed on both Receive operations
when it verifies against the chain; the delivering device is not an input
to verification), whose Receive operations are removed. Its relay rule is
replaced by this item's sender rule: what a Device the current record does
not list delivers is refused on receipt with `SenderNotListed`, whoever
built it, so such a relay is no longer acted on. Its last clause holds as
this item's last sentence: verification, which adopts, never consults the
Device that delivered what it adopts.

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* supersedes LLR-u6rq4s (the authenticated Device key
of a message about a held Organisation is compared only with the current
committed record, before the update is verified; the record after the
update is never consulted about the sender), whose commit path is removed.
The added last sentence states its rule over receipt and verification; its
acceptance of a revocation by the receiving node's own Device is
LLR-rhsfw9's and LLR-jhy6t6's.

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q3 of
2026-10-16):* an organisation request from a Device held as revoked is
answered with its notice; a Device no record has listed is still refused.

*Amended in place 2026-10-21 (unmerged draft; design gap G2 of the plan):*
supersedes LLR-3q63zv (the sender's authenticated Device key is checked
against the current record, and only it, on the deleting and the updating
branch alike, a message from any other Device refused as `SenderNotListed`),
whose `receive_and_self_delete_if_revoked`, `commit_checked` and
`ReceiveKind` branches are removed: this item applies the same check to
every kind that can lead to an update or a removal (Organisation
information, a revocation notice, an organisation answer), on receipt, with
nothing kept.

**LLR-w7b9kd**: from a Device the current record does not list,
`check_received` acts on three kinds alone: an acknowledgement (S3's
revoked-Device path, LLR-5wg8cu), an admission request (LLR-6gkxpg), and an
organisation request from a Device held as revoked (LLR-j338wq); every
other kind, and an organisation request from any other Device, is refused
with `SenderNotListed { org_id }` as LLR-283x6f states, or, in a store with
no current record, as LLR-e6w3nr states.
satisfies: REQ-ncv4sc, REQ-6qxesy, REQ-b462sh, REQ-p4hwuv

*Amended in place 2026-10-20 (unmerged draft; owner ruling Q3 of
2026-10-16):* adds the third kind.

**LLR-5wg8cu**: on an `Acknowledgement(ack)`, `check_received` calls
`revocation::check_acknowledgement(store, sender, ack)` (LLR-5azhry as
amended) with its `sender` argument and no other Device key, applies none of
LLR-283x6f's sender check to it (its sender is by design a Device the record
no longer lists), takes no chain state, and returns `Checked::Acknowledged`
with no `StoreChange`, or that function's refusal unchanged with none.
satisfies: REQ-b462sh, REQ-ncv4sc
supersedes: LLR-3aysup

**LLR-76bhw9**: Organisation information from a listed Device is checked
against the current record with `check_chain_free` (`OrgIdMismatch`,
`StaleSeq`, `MalformedDelta`, `DeltaBaseMismatch`), refused with
`SeqNotEpoch` when its Sequence number is not the record's epoch plus one,
its Change set is applied to the record's trie to give the next record and
root, and it is kept as a `Received { from: sender }` provisional
organisation with `org_pub_key` derived from the carried private key and
`base_snapshot` the record snapshot it carried; an identity already held is
`AlreadyHeld`, no change. Every one of these refusals is made with no chain
state, which `check_received` never takes.
satisfies: REQ-9ktgwx, REQ-ncv4sc, REQ-zxgd46, REQ-f2k4tr
supersedes: LLR-9ew26y LLR-cs4mpb LLR-9sknpa LLR-992mbf LLR-9f5hmr

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR1-N1, owner ruling 2026-10-29):* supersedes LLR-cs4mpb, LLR-9sknpa,
LLR-992mbf, LLR-9f5hmr, which `verify_envelope_against_chain` alone did; it is
deleted, and this item states what each became (see each one's note).

*Amended in place 2026-10-20 (unmerged draft; dispatcher's resolution C6 of
2026-10-16):* supersedes LLR-9ew26y, whose functions are removed; its rule —
the chain-free checks refuse before any chain state is used — is this
item's, over `check_received`.

*Amended in place 2026-10-21 (unmerged draft; owner decision of
2026-10-20):* satisfies REQ-zxgd46, which supersedes REQ-c29s93, in its
place. REQ-zxgd46 (narrowed 2026-10-21, owner decision) refuses a missing
or non-32-byte key and keeps the public key derived from the carried key
(LLR-s3vq6d); the message carries no public key (LLR-xr88a8), so there is
no comparison to make at receipt.

**LLR-e6w3nr**: in a store with no current record, a body is decoded no
further than its kind and Organisation identifier unless it is Organisation
information, or an organisation answer, for an Organisation an expected
admission names; anything else is refused (`AdmissionNotExpected { org_id }`
for Organisation information, `OrgNotHeld { org_id }` otherwise). Expected
Organisation information, from whichever Device the connection
authenticated, has its snapshot decoded (`first_admission_base`), passes
`check_chain_free` against it and is kept as LLR-76bhw9 keeps one,
expectations untouched. An expected organisation answer — the answer to the
organisation request a provisional Member sends its Invite's issuer
(LLR-vyank2) — from whichever Device the connection authenticated, is kept
as LLR-kd7u2r keeps one (only while the behind marker is set, its record
snapshot decoded, its root calculated, an `Answer { from }` provisional
organisation with `org_pub_key` derived from the carried key and no Change
set), without LLR-kd7u2r's listed-sender and record-epoch conditions, which
a store with no record cannot judge; expectations untouched. Either is
adopted only by verification (LLR-8528ud), and in this store only when its
members list the provisional Member (LLR-q4puca).
satisfies: REQ-eqt23j, REQ-8amu2a, REQ-ncv4sc, REQ-2yduet
supersedes: LLR-s8xp7m

*Amended in place 2026-10-30 (change `worktree-org-io-domain-api`, merge
review MR3-N1, owner ruling 2026-10-30):* what this item keeps from
whichever Device is not held for good: every verification discards the
kept information and answers that can never be adopted and ages those one
epoch ahead of the chain (LLR-4tsgfh), so a flood from any Device fills the
bound at most until the next verification, or, for bodies one epoch ahead,
until `ahead_limit` after the first verification that saw them. No sender
restriction is added.

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision, on the dispatcher's resolution C3 of 2026-10-16):* a
provisional Member asks its Invite's issuer for the Organisation
(LLR-vyank2), and LLR-p8vpgz speaks of a record adopted from an
organisation answer, but this item refused the answer in a store with no
record, so the Member could ask and never keep what it was told. The answer
for the Organisation its expected admission names is now kept. A founding
store whose proxy is not yet recorded holds no expected admission, so it
keeps neither; its own organisation request is refused at the building
(`FoundingNotWritten`, LLR-vyank2). The item now also satisfies
REQ-2yduet, whose answer it keeps for verification.

**LLR-7u3dg7**: `check_received` returns `Ok` or a typed `OrgNodeError` for
every `body` and every `sender`, in a store with a current record and in one
with none, and panics on none. This holds past the decode: Organisation
information whose Change set decodes and names the base root is applied
(`apply_delta`), its root calculated and the result checked
(`verify_against`) without a panic, whatever Change set and record snapshot
bytes it carries. A refusal leaves the store's encoded bytes as they were.
satisfies: REQ-rv4dwb
supersedes: LLR-hs7g6j

*Added 2026-10-30 (change `worktree-org-io-domain-api`, merge review round
2, MR2-N1).* LLR-hs7g6j bounded `verify_envelope_against_chain`, which is
deleted; the apply its fuzz target reached now runs here (LLR-76bhw9,
LLR-e6w3nr). Verified by the bolero target `fuzz_check_received`, which
drives arbitrary bodies, arbitrary Change sets in an honest body, and honest
Change sets and bodies with one byte perturbed, through both stores.

**LLR-r4qtqe**: a revocation notice from a listed Device that
`revocation::check_notice` passes (LLR-r7zm39) is appended to
`kept_revocation_notices` (the bound applies) and returned as
`RevocationNoticeKept`; nothing is deleted and no secret is asked for on
receipt. A notice that comes in answer to this Device's organisation request
is a `Revocation` body like any other and is kept the same way; only
`verify` acts on it (LLR-jhy6t6).
satisfies: REQ-ncv4sc, REQ-ea4qs5, REQ-2yduet, REQ-y9p27b

*Amended in place 2026-10-20 (unmerged draft; owner rulings Q3 and Q4' of
2026-10-16):* checked against the new rulings — unchanged in rule; states
the answer case and satisfies REQ-2yduet and REQ-y9p27b.

**LLR-95av3e**: a `Revocation` body about an Organisation the receiving
store holds no record of is refused by `check_received` with a typed error,
keeping nothing and returning no `StoreChange`: in a store holding a record
of another Organisation, by `revocation::check_notice` with
`RevocationNotHeld { org_id }` (LLR-r7zm39); in a store holding no record (a
provisional founding or a provisional Member), with `OrgNotHeld { org_id }`
before any field past its kind and Organisation identifier is decoded
(LLR-e6w3nr). Either way no record is created, an expected admission stays
in place, and no chain state is taken.
satisfies: REQ-vxqc5g
supersedes: LLR-38e2kn

*Added 2026-10-21 (unmerged draft; design gap G2 of the plan):* LLR-38e2kn
stated this of `receive_and_verify` and `receive_and_self_delete_if_revoked`
(S4's note: of `check_received`, the store chosen by
`revocation::notice_store`), with `RevocationNotHeld` in every case. With
one Organisation per store, a store with no record refuses every body but
expected Organisation information first (LLR-e6w3nr), so its refusal of a
revocation is `OrgNotHeld`; REQ-vxqc5g asks for a typed refusal that leaves
the store unchanged, which both are.

**LLR-6gkxpg**: an admission request is first read with
`peek_admission_request` (LLR-g9n9w9); a pair `(org_id, invite_id)` not in
this store's `outstanding_invites` is refused with `InviteNotOutstanding {
org_id, invite_id }`
before any later field is decoded. For an outstanding pair the body is fully
decoded; a request whose `device_key` is not `sender` is refused with
`AdmissionKeyNotSender { org_id }`, no change (org-io's verdict for it is
"refused", as for every refusal); a request whose preferred Handles are
fewer than one, more than three or not all distinct is refused with
`InvalidField { field: "request.preferred_handles" }`, keeping nothing; its
addresses are parsed (LLR-h4gwns); and then: no request kept for that invite → kept with `from:
sender` and the addresses it carries, outcome `Accepted`; one kept from
`sender` → `AlreadyReceived`, no change; one kept from another Device →
`InviteUsedByAnotherDevice`, no change. Every outcome but `Accepted`, and
every refusal, returns no `StoreChange`.
satisfies: REQ-6qxesy, REQ-ncv4sc

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* the preferred-Handle refusal is added, mirroring for
a received request the rule REQ-kd34f5 sets for a provisional Member
(LLR-rjphn4), so that no kept request holds a preferred-Handle list the
joining Device's own provisional Member could not hold; it is refused at
receipt, after the sender check and before the addresses, as LLR-h4gwns
refuses addresses.

*Amended in place 2026-10-20 (unmerged draft; dispatcher's resolution C4
and owner answer A2 of 2026-10-16):* the request's DevicePublicKey must be
the key the connection authenticated, so one Device cannot request admission
for another key; the kept request keeps the sender's addresses.

*Amended in place 2026-10-28 (unmerged draft; round 2 review, C2-N1):* the
admission request's result is named `AdmissionRequestOutcome` (field
`outcome`), not a verdict: the glossary's Verdict is the byte org-io writes
back, which this outcome only chooses.

*Amended in place 2026-10-28 (unmerged draft; round 2 review, C2-N2):* a
refusal names the input it refused: `InviteNotOutstanding { org_id,
invite_id }`, `NoAdmissionRequestKept { invite_id }`,
`AdmissionAlreadyRecorded { invite_id }`, `HandleTaken { handle }` and
`PersonaNotHeld { device_key }`. `HandleTaken` carries the Handle but its
`Display` leaves it out, a Handle being personal data. A lookup by Persona
identifier (`persona_public_keys`, LLR-437fvx) has no Device key to name,
so it refuses with the new `PersonaIdNotHeld { persona_id }`; every
lookup by Device key stays `PersonaNotHeld` (LLR-f9x3vf).

## Messages for the current epoch

**SDD-qzv5mn**: the messages of the adopted current epoch. Code: `OrgService::messages_for_current_epoch` and `OutgoingMessage` in `org-node/src/service.rs`. Who receives what for the epoch a store holds as current; it neither writes nor sends.
traces: REQ-ttv8k7, REQ-3dsweu, REQ-szq3ud, REQ-ps2gy2, REQ-y7tsft
supersedes: SDD-bm27xt

*Amended in place 2026-10-28 (unmerged draft; round 3 review AR3-5, sweep
for ARCHITECTURE Boundaries 3):* "it writes nothing and sends nothing" is reworded "it neither writes nor sends"; nothing else changes.

**LLR-degg8t**: `OrgService::messages_for_current_epoch(&self, device_key)
-> Result<Vec<OutgoingMessage>, OrgNodeError>` refuses a store with no
current record (`OrgNotHeld`); when the record keeps no Change set it returns
none; otherwise it returns one `OrgInformation { envelope: (org_id, epoch,
kept_change_set), record_snapshot: kept_base_snapshot, org_private_key }`
body for each Device the record lists other than the Persona's own, in record
order, then one `Revocation` body (`notices_for`) for each Device the base
snapshot lists and the record does not, holding that Device's notice and
nothing else: no Envelope, Change set, member snapshot or Organisation
private key. Equal stores give equal results on every call, the store is
not changed, and no body carries an invite identifier. A receiver acts on
such a notice only when its absence proof verifies against the chain's
root (LLR-rhsfw9), which a Device the chain's record still lists has none
for.
satisfies: REQ-ttv8k7, REQ-3dsweu, REQ-szq3ud, REQ-ps2gy2
supersedes: LLR-gdfqn9 LLR-bg3vsw LLR-6ymd6d LLR-a8z7r5 LLR-2xzys9 LLR-48jakr LLR-8hdu9x

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* supersedes LLR-8hdu9x (what `send_update` sends: a
removed Device a `Revocation` holding its notice and nothing else, every
listed Device `OrgInformation` with the Envelope, the snapshots before the
update and the epoch's Organisation private key; a receiver accepts a
revocation only on an absence proof against the chain's root), whose
operation is removed. The "nothing else" clause and the receiver's
sentence are added so this item states the whole of its rule.

**LLR-rsjs8e**: `OutgoingMessage { recipient: DevicePublicKey, body:
Vec<u8> }` is not `Clone`, and its `Debug` renders the recipient and the
body's length only, never a byte of the body.
satisfies: REQ-ttv8k7, REQ-y7tsft

## The Invite and the admission request

**SDD-xexu3k**: the Invite and the admission request. Code: `org-node/src/invitation.rs` (`InviteId`, `Invite`, the Blob armour), `StoreData.outstanding_invites` and `kept_admission_requests`, and the service operations `build_invite`, `build_admission_request`, `cancel_invite`. The Invite names the one issuing Device and its addresses; the admission request carries the Invite's identifiers first and the sending Device's own addresses last; the endpoint addresses one Device keeps for another (`EndpointAddresses`, the address book, `addresses_for`) are this item's too.
traces: REQ-ky5tyd, REQ-9xf99q, REQ-6qxesy, REQ-f8p36u, REQ-nbx4nn
supersedes: SDD-xshv5b
superseded-by: SDD-n2x3gb, SDD-42cjqk

*Amended in place 2026-10-20 (unmerged draft; owner answer A2 of
2026-10-16):* requests carry the sender's addresses, and the receiver keeps
them per Device key for org-io's send path.

*Superseded 2026-10-28 (unmerged draft; round 2 review AR2-10,
ARCHITECTURE Boundaries 3):* the item stated two responsibilities, the
invitation of a joining Device and the endpoint addresses one Device keeps
for another, each with its own reason to change. It is split into
SDD-n2x3gb (the invitation) and SDD-42cjqk (the address book). Its LLRs are
re-pointed: LLR-h4gwns, LLR-h9d65p, LLR-pgy2rb and LLR-ds228z are
SDD-42cjqk's; LLR-h9f5c9, LLR-9jxwq4, LLR-g32hjn, LLR-v3av9t, LLR-96z2k5
and LLR-ag8hjd are SDD-n2x3gb's. No behaviour changes.

### The address book (successor of SDD-xexu3k)

**SDD-42cjqk**: the address book: the endpoint addresses one Device keeps
for another, held as opaque text it parses but never dials. Code:
`EndpointAddresses` (`org-node/src/ids.rs`, re-exported by
`invitation.rs`), `StoreData.address_book`, and `OrgService::addresses_for`;
the receive path (SDD-busuj3) and verification (SDD-kegag2) write the book,
as LLR-h9d65p states.
traces: REQ-ky5tyd, REQ-6qxesy
supersedes: SDD-xexu3k

*Added 2026-10-28 (unmerged draft; round 2 review AR2-10):* SDD-xexu3k's
address half. LLR-h4gwns, LLR-h9d65p, LLR-pgy2rb and LLR-ds228z are its
LLRs.

**LLR-h4gwns**: `EndpointAddresses::parse(Vec<String>)` accepts zero to
sixteen entries, each at most 256 bytes of UTF-8 (the bound of the Invite's
`issuer_addresses`, LLR-9jxwq4, which uses this type), and refuses any other
list. A received admission or organisation request whose addresses it
refuses is refused with `InvalidField { field: "request.addresses" }`,
keeping nothing and answering nothing; `build_admission_request` and
`build_organisation_request` refuse caller-given addresses it refuses with
the same field, building nothing.
satisfies: REQ-ky5tyd, REQ-6qxesy

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* "one to
sixteen" becomes "zero to sixteen". In Networked mode an endpoint may hold
no direct address (discovery finds it by its key), and org-io gives the
addresses its endpoint holds in the current mode, possibly none; refusing an
empty list would have refused every Invite and request such an endpoint
builds, against the requirement that they carry the addresses the caller
gives (REQ-ky5tyd).

**LLR-h9d65p**: `StoreData.address_book` holds at most one entry per
DevicePublicKey. An organisation request `check_received` acts on, from a
Device the current record lists or holds as revoked, sets the entry for
`sender` to the addresses it carries, replacing an earlier one, and returns
`StoreChange::Sealed` only when the entry changed; an admitted Device's
entry is set at adoption from its kept admission request (LLR-rzqs2u), and
the Invite issuer's entry is set at a provisional Member's first adoption
from the addresses its Invite carried (LLR-q4puca). No other operation
writes it, a refused request keeps nothing, and it is
dropped whole only by `into_removed` (LLR-2d5a7m). It is not measured by
the bound of LLR-m76r4k.
satisfies: REQ-ky5tyd

*Amended in place 2026-10-25 (unmerged draft; PR-8sgsb5):* the first
adoption is a third writer of the address book (LLR-q4puca): it keeps the
Invite issuer's addresses, which a provisional Member held outside the book.

**LLR-pgy2rb**: `OrgService::addresses_for(&self, device_key, peer:
DevicePublicKey) -> Result<Option<&EndpointAddresses>, OrgNodeError>`
returns, in this order: the address-book entry for `peer`; else the
addresses of a kept admission request whose `from` is `peer`; else, in a
provisional Member's store whose `member.issuer` is `peer`, the Invite's
issuer addresses; else `None`. It changes nothing and refuses an unknown
store (`PersonaNotHeld`).
satisfies: REQ-ky5tyd

**LLR-ds228z**: org-node holds endpoint addresses only as opaque text
(`EndpointAddresses`, LLR-h4gwns) and never dials, resolves or builds one:
it names no transport address type, no peer address and no function that
builds an address to dial. The values it returns that carry addresses
another Device gave are `addresses_for`'s (LLR-pgy2rb), the kept admission
requests `admission_requests` returns (LLR-8ru6mv, LLR-v3av9t), and the
recipients' addresses `kept_acknowledgements` returns with each kept
acknowledgement (LLR-75sn9g); the Invite and the admission and
organisation requests it builds carry only the addresses their caller gave
(LLR-h9f5c9, LLR-g32hjn, LLR-vyank2). No other value it returns carries an
endpoint address.
satisfies: derived
supersedes: LLR-qezw3n

*Added 2026-10-22 (unmerged draft; plan detail pass — dispatcher
decision):* LLR-qezw3n said no value org-node returns carries a dialling
address for another Device to reach it by. Under owner answer A2 of
2026-10-16 the Invite and both requests carry the sender's addresses and
the receiver keeps them per Device key, so that rule no longer holds; this
item states which returned values carry an address and that org-node still
dials none. The three named for addresses another Device gave are all the
operations of this design that return them: the dispatcher's decision named
`addresses_for` and the kept admission requests, and `kept_acknowledgements`
(LLR-75sn9g) returns its recipients' addresses by design too.

### Inviting a Device (successor of SDD-xexu3k)

**SDD-n2x3gb**: the invitation of a joining Device, from the Invite this
Device issues to the admission request kept for it until the invite is
settled or cancelled. Code: `org-node/src/invitation.rs` (`InviteId`,
`Invite`, the Blob armour), `StoreData.outstanding_invites` and
`kept_admission_requests`, and the service operations `build_invite`,
`build_admission_request`, `cancel_invite`. The Invite names the one issuing
Device and its addresses; the admission request carries the Invite's
identifiers first and the sending Device's own addresses last.
traces: REQ-ky5tyd, REQ-9xf99q, REQ-6qxesy, REQ-f8p36u, REQ-nbx4nn
supersedes: SDD-xexu3k

*Added 2026-10-28 (unmerged draft; round 2 review AR2-10):* SDD-xexu3k's
invitation half. LLR-h9f5c9, LLR-9jxwq4, LLR-g32hjn, LLR-v3av9t, LLR-96z2k5
and LLR-ag8hjd are its LLRs.

**LLR-h9f5c9**: `OrgService::build_invite(rng, device_key, invitee_name:
String, issuer_addresses: Vec<String>) -> Result<(Invite, StoreChange),
OrgNodeError>` refuses a store with no current record (`OrgNotHeld`, nothing
written); otherwise draws a fresh 32-byte `InviteId`, appends
`OutstandingInvite { org_id, invite_id, invitee_name }` to
`outstanding_invites`, and returns an `Invite { org_name, org_id,
invitee_name, issuer: DevicePublicKey, issuer_addresses, invite_id }`
carrying the stored Organisation name and the store's Persona's
DevicePublicKey; the invitee name kept and the one the Invite carries are
the one it was given.
satisfies: REQ-ky5tyd
supersedes: LLR-nr8df3

*Amended in place 2026-10-20 (unmerged draft; consistency pass, REQ-ky5tyd
as amended that day):* the outstanding invite keeps the invitee name the
inviter typed.

**LLR-9jxwq4**: `Invite::parse` takes a Blob (Base64 of the postcard
encoding) and parses each field with its type; it refuses the first that
fails with `InvalidField` whose `field` is `invite.org_name`,
`invite.org_id` (twenty bytes), `invite.invitee_name`, `invite.issuer`
(`DevicePublicKey::parse`), `invite.issuer_addresses` (zero to sixteen
entries, each at most 256 bytes of UTF-8), or `invite.invite_id` (thirty-two
bytes), and text that is not a Blob with `field` `invite`; the parse is pure,
so a refused Invite acts on nothing. A Blob `Invite::encode` wrote parses
back to an equal Invite.
satisfies: REQ-9xf99q, REQ-ky5tyd
supersedes: LLR-b7wgpf

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* "one to
sixteen" becomes "zero to sixteen", as LLR-h4gwns: a Networked endpoint may
hold no direct address.

**LLR-g32hjn**: `OrgService::build_admission_request(&self, device_key,
sender_addresses: Vec<String>) -> Result<OutgoingMessage, OrgNodeError>`
refuses a store holding no provisional Member (`NoProvisionalMember`) and
addresses the parse of LLR-h4gwns refuses, and otherwise returns, for the
recipient `member.issuer`, the encoded `WireMessage::AdmissionRequest` of the
provisional Member's `org_id`, `invite_id`, name, surname, preferred Handles,
Member-as-a-group key and DevicePublicKey, and the parsed
`sender_addresses` (LLR-xr88a8); it changes nothing, so a retry with the
same addresses sends the same bytes.
satisfies: REQ-ky5tyd, REQ-6qxesy

*Amended in place 2026-10-20 (unmerged draft; owner answer A2 of
2026-10-16):* the request carries the sending Device's own endpoint
addresses, which org-io gives.

**LLR-v3av9t**: a `KeptAdmissionRequest` holds `org_id`, `invite_id`,
`from: DevicePublicKey` (the Device the transport authenticated, equal to
the request's DevicePublicKey, LLR-6gkxpg), name, surname, preferred
Handles, Member-as-a-group key, DevicePublicKey and `addresses:
EndpointAddresses` (those the request carried); `kept_admission_requests`
holds at most one per invite id, the first kept.
satisfies: REQ-6qxesy, REQ-ky5tyd

*Amended in place 2026-10-20 (unmerged draft; owner answer A2 and
dispatcher's resolution C4 of 2026-10-16):* adds the sender's addresses.

**LLR-96z2k5**: nothing a joining Device composes reaches the inviting
store except a received `AdmissionRequest` that `check_received` decodes and
parses field by field (LLR-6gkxpg, LLR-h4gwns) into a
`KeptAdmissionRequest`; a Member it admits is built from that kept request
alone (LLR-va2yqz). org-node has no `import_join_request`, no Join-request
or Invite-reply type and no other operation that takes bytes a joining
Device composed.
satisfies: derived
supersedes: LLR-836z24

*Added 2026-10-21 (unmerged draft; design gap G2 of the plan):* LLR-836z24
said what a joiner composes reaches the inviter's store only through
`admit_member`'s parsed `Joiner`; both are removed. The admission request,
kept parsed, takes the `Joiner`'s place, and the absence of a Join-request
import holds unchanged.

**LLR-ag8hjd**: `OrgService::cancel_invite(rng, device_key, invite_id) ->
Result<StoreChange, OrgNodeError>` removes the kept admission request for
`invite_id`, its outstanding pair and any `Admit` change naming it, so a
later admission request carrying it is refused as not outstanding; it
refuses with `NoAdmissionRequestKept`, changing nothing, an invite id with
no kept request.
satisfies: REQ-f8p36u

## Requesting the Organisation

**SDD-pkqc2j**: the organisation request exchange, by which a Device behind obtains the Organisation. Code: `OrgService::build_organisation_request` and the organisation-request and organisation-answer arms of `check_received`. A Device behind asks the Devices of its current record for the Organisation (a provisional Member, holding no record, asks its Invite's issuer); a listed requester is answered with the full current record and its Organisation private key, a requester held as revoked with its revocation notice and nothing else; an answer is kept, never adopted, until verification.
traces: REQ-p4hwuv, REQ-2yduet, REQ-ky5tyd, REQ-ncv4sc

*Amended in place 2026-10-28 (unmerged draft; round 3 review AR3-5, sweep
for ARCHITECTURE Boundaries 3):* it read "the organisation request and its answer", which names two subjects; it is reworded to the one exchange both halves belong to. Its code and LLRs are unchanged.

*Amended in place 2026-10-20 (unmerged draft; owner rulings Q3 and A2,
dispatcher's resolution C3 of 2026-10-16):* the revoked requester's answer,
the requests' addresses and the provisional Member's request to the issuer.

**LLR-vyank2**: `OrgService::build_organisation_request(&self, device_key,
sender_addresses: Vec<String>) -> Result<Vec<OutgoingMessage>,
OrgNodeError>` returns one encoded `WireMessage::OrganisationRequest {
org_id, addresses }`, `addresses` the parsed `sender_addresses`
(LLR-h4gwns), for each Device the current record lists other than the
Persona's own, in record order; in a provisional Member's store, which holds
no record, one for `member.issuer`; it refuses a store with neither
(`OrgNotHeld`, or `FoundingNotWritten` for a founding store whose proxy is
not yet recorded, LLR-3sddey) and changes nothing.
satisfies: derived

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* a founding store whose proxy is not yet recorded has
no Organisation identifier for `OrgNotHeld { org_id }` to name, and is
refused with `FoundingNotWritten`, the existing variant for a founding with
no chain identity.

*Amended in place 2026-10-20 (unmerged draft; owner answer A2 and
dispatcher's resolution C3 of 2026-10-16):* the request carries the sending
Device's addresses, and a provisional Member asks the Invite's issuer, who
answers once its current record lists the requester.

**LLR-7kdhst**: an organisation request from a listed Device (LLR-283x6f) is
answered, without asking the user, with
`Checked::OrganisationRequest { answer }`, `answer.recipient` the sender and
`answer.body` the encoded `WireMessage::Organisation { org_id, epoch,
record_snapshot, org_private_key }` of the current record, the key carried
as the same `OrgPrivateKey` field an Organisation-information body carries
to a listed Device (`Debug` redacted, LLR-4ces44); the one change it makes
is the sender's address-book entry (LLR-h9d65p). org-io sends the answer
through its send path to the addresses kept for that key, never on the
connection the request arrived on.
satisfies: REQ-p4hwuv, REQ-ky5tyd

*Amended in place 2026-10-20 (unmerged draft; owner answer A2 of
2026-10-16):* the request's addresses are kept; the answer itself is
unchanged.

**LLR-bgj2ff**: `StoreData.revoked_devices` holds `RevokedDevice { member_id:
MemberId, device_key: DevicePublicKey }` values. Each adoption that replaces
a current record (LLR-34c5y3) appends, in the previous record's snapshot and
slot order, one for each pair the previous record lists whose DevicePublicKey
the adopted record does not list and `revoked_devices` does not already
hold; a first adoption (founding, first admission) appends none, no other
operation writes it, and only `into_removed` drops it. A Device is held as
revoked exactly when `revoked_devices` holds its DevicePublicKey and the
current record does not list it.
satisfies: REQ-p4hwuv

**LLR-j338wq**: an organisation request from a Device held as revoked
(LLR-bgj2ff) is answered, without asking the user, with
`Checked::OrganisationRequest { answer }`, `answer.recipient` the sender and
`answer.body` the encoded `WireMessage::Revocation(notice)` with `notice {
org_id, member_id, device: sender, proof }`, `member_id` the held
`RevokedDevice`'s and `proof` the current record's calculated trie's
`prove_absent(member_id, sender)`, and nothing else: no record, snapshot or
key. A `prove_absent` refusal is returned as `Trie(cause)` with no answer
and no `StoreChange`. The one change it makes is the sender's address-book
entry (LLR-h9d65p).
satisfies: REQ-p4hwuv, REQ-ncv4sc

**LLR-xc9c6w**: an answer whose encoding exceeds `MAX_MESSAGE` is refused
with `MessageTooLarge { length }` and no answer is returned; the request is
otherwise answered whole or not at all.
satisfies: REQ-p4hwuv

**LLR-kd7u2r**: an organisation answer is kept only from a listed Device and
only while the behind marker is set (else `SenderNotListed` or
`OrganisationNotRequested { org_id }`), for an epoch greater than the
current record's (else `StaleChainState`); its record snapshot is decoded
(`member.*` refusals), its root calculated, and it is kept as an `Answer {
from }` provisional organisation with `org_pub_key` derived from the carried
key and no Change set, adopted only by verification (LLR-8528ud).
satisfies: REQ-2yduet, REQ-ncv4sc

## SDD-kwncn7 — the Wire message of six kinds (additions)

These sit under SDD-kwncn7 (the framing of one Wire message,
`org-node/docs/architecture/2026-10-03-decomposition.md`), whose
responsibility is unchanged; `MAX_MESSAGE` stays 1 MiB.

**LLR-xr88a8**: `WireMessage` is an enum of exactly six kinds, encoded by
postcard as the variant index followed by its fields in order:
`OrgInformation { envelope, record_snapshot, org_private_key }` (0),
`Revocation(RevocationNotice)` (1), `Acknowledgement(Acknowledgement)` (2),
`AdmissionRequest { org_id, invite_id, name, surname, preferred_handles,
member_key, device_key, addresses }` (3), `OrganisationRequest { org_id,
addresses }` (4) and
`Organisation { org_id, epoch, record_snapshot, org_private_key }` (5). No
field is optional; `org_id()` returns any kind's Organisation;
`decode_message` refuses with `MalformedMessage`, without panicking, an
unknown index, a body that ends early and any bytes left over.
satisfies: REQ-ncv4sc, REQ-zxgd46, REQ-6qxesy, REQ-p4hwuv, REQ-ky5tyd
supersedes: LLR-tajh9d

*Amended in place 2026-10-20 (unmerged draft; owner answer A2 of
2026-10-16):* both requests carry the sending Device's addresses as their
last field, so `peek_admission_request` (LLR-g9n9w9) is unchanged.

*Amended in place 2026-10-21 (unmerged draft; owner decision of
2026-10-20):* satisfies REQ-zxgd46, which supersedes REQ-c29s93, in its
place (a key field that is not 32 bytes does not decode).

**LLR-g3wmen**: `AdmissionRequest` is the only kind that carries an invite
identifier; no Organisation-information, revocation, acknowledgement,
organisation-request or organisation body, and no expected admission, holds
one.
satisfies: REQ-6qxesy, REQ-8amu2a
supersedes: LLR-ms8njy

**LLR-g9n9w9**: `peek_admission_request(body: &[u8]) -> Option<(OrgId,
InviteId)>` decodes only the variant index and, for index 3, the next twenty
and thirty-two bytes, returning `None` for any other index or a shorter body;
it reads no byte after the invite id.
satisfies: REQ-6qxesy

**LLR-4ces44**: the `Debug` rendering of a `WireMessage` of any of the six
kinds contains no byte of an Organisation private key it holds
(`OrgPrivateKey([REDACTED])`).
satisfies: REQ-y7tsft

## SDD-swtd3w — value types and the rejection vocabulary (additions)

**LLR-3sddey**: `OrgNodeError` gains, each distinct from every other variant
with a `Display` naming the Organisation or value where it has one:
`NoProvisionalChange`, `OrganisationBehind { org_id, chain_epoch }`,
`NoAdmissionRequestKept { invite_id }`, `AdmissionAlreadyRecorded {
invite_id }`, `NoFreeHandle`, `HandleTaken { handle }` (whose `Display`
leaves the Handle out), `MemberNotHeld { member_id }`,
`RevocationAlreadyRecorded { member_id }`, `NoProvisionalFounding`,
`FoundingProxyAlreadyRecorded`, `FoundingNotWritten`, `NoProvisionalMember`,
`OrganisationNotRequested { org_id }`, `PersonaNotHeld { device_key }` and
`PersonaIdNotHeld { persona_id }` and `DuplicateOrgPublicKey`; `ProvisionalLimit { limit }`'s `Display`
reads "provisional changes and organisations for one Organisation would
exceed {limit} bytes". `NoProvisionalUpdate`, `ReplyForAnotherOrganisation`,
`PersonaAlreadyBound` and `RecordMovedSinceCheck` are removed with the
operations that raised them. It also gains `AdmissionKeyNotSender { org_id
}`, `OrganisationRemoved { org_id }` and `AcknowledgementNotKept { recipient
}`, and keeps the nine variants of LLR-n67aw8 unchanged —
`DeviceSecretNotSupplied { org_id }` (now raised by LLR-5tvn7g), `OrgNotHeld
{ org_id }`, `StaleChainState { org_id, chain_epoch, record_epoch }`,
`ChainStateConflict { org_id }`, `RevocationProofRefused { org_id, cause }`,
`AcknowledgementNotHeld`, `AcknowledgementFromFuture`,
`AcknowledgementForListedDevice` and `AcknowledgementSignatureInvalid`, each
`{ org_id }`. It keeps three of LLR-j5vbqj's four variants unchanged —
`MalformedMessage` ("received wire message is malformed", LLR-94rzrk,
LLR-xr88a8), `RevocationNotHeld { org_id }` (LLR-38e2kn) and
`RevocationNotForThisDevice { org_id }` (LLR-r7zm39) — and removes the
fourth, `OrgKeyMismatch`, which no operation raises any more.
satisfies: REQ-g2537c, REQ-2nnsmv, REQ-dhr58q, REQ-6qxesy, REQ-f8p36u, REQ-akfwf3, REQ-kt877x, REQ-8amu2a, REQ-yp75u9, REQ-m2xh8q, REQ-b462sh, REQ-h29c8e, REQ-ayyt5f, REQ-48zc3e, REQ-zxgd46, REQ-vxqc5g, REQ-3dsweu, REQ-qrtsc9
supersedes: LLR-mxskg9 LLR-n67aw8 LLR-j5vbqj

*Amended in place 2026-10-20 (unmerged draft; dispatcher's resolution C4,
owner ruling Q4' of 2026-10-16):* the three new variants, and LLR-n67aw8's
list, whose `DeviceSecretNotSupplied` traced the superseded REQ-y99c9w, is
restated here.

*Amended in place 2026-10-20 (unmerged draft; consistency pass):* also
supersedes LLR-j5vbqj, whose `OrgKeyMismatch` served the superseded
REQ-bwx7eg; its other three variants, and their requirements (REQ-c29s93,
REQ-vxqc5g, REQ-3dsweu, REQ-qrtsc9), are carried here.

*Amended in place 2026-10-21 (unmerged draft; owner decision of
2026-10-20):* REQ-c29s93 is superseded by REQ-zxgd46, which this item
satisfies in its place (`MalformedMessage` unchanged).

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* wherever an item of this design refuses a store with
no current record as `OrgNotHeld { org_id }`, a founding store whose proxy
is not yet recorded (`persona.org_id` `None`), which has no identifier to
name, is refused with `FoundingNotWritten` instead; one refusal of a store
with no current record (`service::current_record`) makes both. No variant
is added and no behaviour beyond the variant's choice.

*Amended in place 2026-10-28 (unmerged draft; round 2 review, C2-N2):* a
refusal names the input it refused: `InviteNotOutstanding { org_id,
invite_id }`, `NoAdmissionRequestKept { invite_id }`,
`AdmissionAlreadyRecorded { invite_id }`, `HandleTaken { handle }` and
`PersonaNotHeld { device_key }`. `HandleTaken` carries the Handle but its
`Display` leaves it out, a Handle being personal data. A lookup by Persona
identifier (`persona_public_keys`, LLR-437fvx) has no Device key to name,
so it refuses with the new `PersonaIdNotHeld { persona_id }`; every
lookup by Device key stays `PersonaNotHeld` (LLR-f9x3vf).

*Amended in place 2026-10-28 (unmerged draft; round 2 review, AR2-8):* a
drawn Organisation public key equal to a key already held is org-node's own
refusal, `DuplicateOrgPublicKey`, not org-members' `Trie(DuplicateKey)`
passed through (ARCHITECTURE_GUIDELINES Boundaries 2); `OrgPublicKey::
ensure_distinct_from` refuses with it too. `Trie(..)` stays for failures
org-members itself detects, as its cause.

**LLR-c759yd**: `OrgPublicKey::ensure_distinct_from(members)` refuses with
`DuplicateOrgPublicKey` an Organisation public key equal to the
Member-as-a-group key or to any DevicePublicKey of any of `members`, and
accepts every other key. It is the one comparison by which
`propose_founding` (LLR-9qavm7) and `propose_updated_organisation`
(LLR-qghs95) refuse a drawn Organisation public key equal to a key already
held.
satisfies: REQ-ech45n, REQ-3pj7rd

*Written 2026-10-28 (unmerged draft; round 5 TEST review of
worktree-org-io-domain-api, finding-2, owner ruling of that day):* the
DevicePublicKey half of the comparison cannot be reached through
`propose_founding`, since a drawn X25519 key equals an ed25519
DevicePublicKey only by a 2^-128 accident no boundary fake can arrange, so
the comparison's contract is stated here, at its own interface, where a
direct test checks it (TEST_GUIDELINES Seams 2, criterion 4).

## The Person store

**SDD-27p679**: the Person store. Code: `org-node/src/person_store.rs` (`PersonStore`, `PersonStoreContent`, `PersonView`). The user's own Person, kept apart from every Persona: a `person::Person`, the Person's own DevicePublicKey and PersonPublicKey, and the secret half of the Person key pair, sealed with the Person's own Device seed exactly as a Persona store is, under an org-node type so org-io names no `person` type.
traces: REQ-kd34f5

*Amended in place 2026-10-28 (unmerged draft; ARCHITECTURE review round 4
AR4-3, Domain 1):* the public type the store holds was named `PersonData`,
a synonym the root glossary lists under Person definition's _Avoid_; it is
`PersonStoreContent`, the content of the glossary's Person store. Nothing
else changes.

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* the Person key pair's secret half is kept in the
sealed Person file, under the Person's Device seed, as a Persona's
Member-as-a-group key is kept in its store (its member seed); neither item
held it anywhere before.

**LLR-wqu26s**: `PersonStore::create(rng, name: &str, surname: &str) ->
Result<(PersonStore, DeviceSecret, SealedStore), OrgNodeError>` parses the
name and surname (`InvalidField` `person.name` / `person.surname`, creating
nothing), draws a fresh Device seed and a Person key pair, and holds
`PersonStoreContent { person: Person, device_key, person_key, person_seed }`,
`person_seed` the key pair's secret seed under a redacting secret type, as
a Persona record holds its member seed (`MemberSeed`); it holds no Handle.
The Device seed is returned by value for org-io's keychain and not kept;
the Person seed is kept only in `PersonStoreContent` and the sealed Person file.
satisfies: REQ-kd34f5

*Amended in place 2026-10-28 (unmerged draft; ARCHITECTURE review round 4
AR4-3):* `PersonData` renamed `PersonStoreContent` (SDD-27p679's note).

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* the secret half of the Person key pair was held
nowhere, as neither this item nor SDD-27p679 asked for it; it is now kept
in the Person store as a Persona keeps its member seed.

**LLR-73sgz2**: `PersonStore::seal(rng)` and `PersonStore::unseal(sealed,
device_key, secret)` use the AEAD, nonce layout and refusals of the Persona
store (LLR-bg538c) with the store key derived as LLR-2qqqx6 states but with
the info string `b"ods/person-store/v1"`, so a Persona store's bytes never
open as a Person store or the reverse; `unseal` re-parses every field
(`person.*`), and the Person seed is sealed with the rest and read back
unchanged.
satisfies: derived

*Amended in place 2026-10-22 (unmerged draft; plan detail pass —
dispatcher decision):* the sealed Person file also holds the Person key
pair's seed (LLR-wqu26s).

**LLR-5uf4u2**: org-node's public interface names every type org-io needs
for Persons and Members (`PersonStore`, `PersonStoreContent`, `PersonView`, `Name`, `Surname`,
`Handle`, `DevicePublicKey`, `PersonPublicKey`) under `org_node::` paths, so
org-io's source names no `person::` path.
satisfies: derived

*Amended in place 2026-10-28 (unmerged draft; review round 1 A1):*
`PersonView`, the key-free view org-io now reads the Person through
(LLR-8ru6mv), joins the list.

*Amended in place 2026-10-28 (unmerged draft; ARCHITECTURE review round 4
AR4-3):* `PersonData` renamed `PersonStoreContent` (SDD-27p679's note).

# Requirements — verify-and-commit, the wire, and the store

org-node's first requirements of its own behaviour. Each realises a risk
control from
`org-node/docs/risk/2026-09-09-org-node-hazards.md`
and is worded from behaviour the crate already has; no source behaviour was
changed to fit a requirement. Each is `satisfies: derived` — it exists because
of how the node was built, not because a system-needs document asked for it —
and is assessed in that risk file.

Terms: *Envelope*, *Wire message*, *Sequence number*, *Published signing
key*, *Persona*, *Organisation secret*, *Invite*, *Persona store*,
*Receive operation* and *Device key* are defined in `org-node/docs/CONTEXT.md`;
*Organisation*, *Change set*, *Membership record*, *Membership root*,
*Organisation state* and *Member-as-a-group key* in the root
`docs/CONTEXT.md`. (*Amended 2026-10-05:* *Device key* is the name the items
below use for what the root glossary calls a DevicePublicKey; org-node's
glossary says so.)

Every requirement below is verified by a test in `org-node/tests` carrying
`verifies:` with its ID. The verify-path tests were relocated there from unit
tests inside `org-node/src` by this change; the abnormal-input tests are new.

## Verifying a received Change set against the chain

**REQ-gju89b**: The software shall reject an Envelope that names an
Organisation other than the one it expected, before decoding the Change set
the Envelope carries.
implements: RC-pm9kmx
satisfies: derived

## Replay

**REQ-6yu72z**: The software shall reject an Envelope whose Sequence number is
not greater than the highest Sequence number it has committed for that
Organisation, before decoding the Change set.
implements: RC-m4r75s
satisfies: derived

**REQ-mr5abb**: The software shall advance the Sequence-number high-water mark
for an Organisation only when an Envelope has been committed, never on a
rejection at any step, and never to a value lower than the mark it holds.
implements: RC-m4r75s
satisfies: derived

*Note 2026-10-30 (change `worktree-org-io-domain-api`, merge review MR1-N1,
owner ruling 2026-10-29):* the item stays in force. The mark is the current
record's Sequence number. The receive path asks it through
`SeqGuard::check`, which never moves it (LLR-f5kq88), and only an adoption,
at an epoch greater than the record's, sets it (LLR-34c5y3), so "committed"
reads "adopted". `SeqGuard::advance` is deleted with
`verify_envelope_against_chain`, its last caller.

## Hostile input

**REQ-9g6as6**: The software shall, for any sequence of bytes offered as an
Envelope, either decode it and the Change set it carries or return a typed
error, and shall not panic.
implements: RC-e2uvje
satisfies: derived

**REQ-eg5j8u**: The software shall reject with a typed error, before decoding
any of its bytes, a wire frame whose body exceeds 1 MiB, on send and on
receive alike, and shall deliver a frame within that bound unchanged.
implements: RC-gfn6kr
satisfies: derived

*Amended 2026-10-08 (S4, task T2), a clarification:* "wire frame" reads "Wire
message body": org-node bounds the Wire message's encoding (`encode_message`,
`decode_message`), with no length prefix. The bound on the stream a peer
sends is org-io's (its stream requirement, in
`org-io/docs/requirements/`).

## Who the Wire message came from

## Acting on one's own removal

## The store at rest

**REQ-hzm4kt**: The software shall write the Persona store only as ciphertext
under a key derived from a passphrase, such that opening the store file with a
different passphrase yields an error and not data, and no member seed, device
seed or Organisation private key appears in the file in clear.
implements: RC-jjsz97
satisfies: derived

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
The item named the Organisation secret, which that change removes in favour of
the Organisation private key (REQ-szq3ud).

*Amended 2026-10-08 (S4, task T3):* "write the Persona store only as
ciphertext" becomes "give the Persona store out only as ciphertext": org-node
seals each Persona's store under a key derived from that Persona's device
secret and returns the sealed bytes for org-io to write (REQ-x3jrh8,
REQ-qthum6); no passphrase.

## What these requirements do not say

They do not state the base-root rule (REQ-4umsuz, org-members, exported): the
node relies on it at step 5 of verification and it is org-members' behaviour.
They do not state what the chain read returns (org-io's finalised-block expectation (moved 2026-10-07), an expectation on
on-chain-client) nor what a Change set that removes a Device key must do to
the Member-as-a-group key (REQ-q92yac, an expectation on org-members); both
remain expectations, due 2026-12-05, and neither carries `(implements:)` in
this change for the reason the risk file gives. And they do not state anything
about the publish path — ordering the chain write and the local record, or
delivery to the peer — because that path's behaviour is a defect under a
problem report (the resolved problem report on the publish path writing the chain before the record) and a requirement would be written against the fix,
not the defect.

And they do not cover every module of the unit. The hazard analysis was run
over the whole of `org-node/src` — `verify.rs`, `sequence.rs`, `envelope.rs`,
`service.rs`, `chain.rs`, `chain_read.rs`, `chain_write/`, `ceremony.rs`,
`transport/`, `blobs.rs`, `store.rs`, `keys.rs`, `ids.rs`, `error.rs` and
`preflight.rs` — while the requirements above state only the behaviour the
nine controls that analysis chose call for, which is the verify-and-commit
path, the frame bound, the sender cross-check, the self-delete and the store
at rest. `ceremony.rs`, `keys.rs`, the blob encoding in `blobs.rs`,
`chain_write/`, `chain_read.rs` — the module the resolved problem report on the chain reader's finality doc-comment is filed against —
`preflight.rs`, the transport handshake and the two small modules `ids.rs`
and `error.rs` have no requirement here; `chain.rs`'s `ChainReader` is
exercised by the verify requirements as the seam the chain read arrives
through, without a requirement of its own. Their requirements,
and the low-level requirements of the whole unit, are tooth 4.

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`).* "The
sender cross-check" in the list above is no longer a behaviour any requirement
here states: earlier forms of REQ-eqt23j and REQ-uk9rw7 are amended in place to say that
nothing about the sender is checked.

*Amended 2026-10-07 (owner ruling R1, change
`worktree-org-io-commit-workflow`).* The membership cross-check is back for
an Organisation the node holds: an earlier form of REQ-uk9rw7 as amended that day. A first
admission still checks nothing about the sender (an earlier form of REQ-eqt23j).

*Amended 2026-10-08 (change `worktree-org-io-create`).* The membership
cross-check is now stated by REQ-uk9rw7, which supersedes an earlier form of REQ-uk9rw7 (owner's
hybrid ruling on the resolved problem report on amendment in place instead of supersession). And by ruling B org-node reads no chain:
`chain_read.rs` is deleted, `chain.rs` holds `OrgState` and its parse edge
but no `ChainReader`, and the verify requirements take the chain's state as
a value that org-io read (org-io's ledgers, `org-io/docs/`). The module list
above is the one the hazard analysis ran over.

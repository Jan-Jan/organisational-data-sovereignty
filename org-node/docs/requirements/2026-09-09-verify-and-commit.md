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

**REQ-wp2nyc**: The software shall reject a received Change set, leaving its
record unchanged, when the Membership root recomputed by applying the Change
set to its own record differs from the Membership root of the Organisation
state it read from the chain. (implements: RC-6a2dke)
satisfies: derived

**REQ-bvh8v6**: The software shall reject a received Change set, leaving its
record unchanged, when the chain holds no Organisation state for the
Organisation the Envelope names. (implements: RC-6a2dke)
satisfies: derived

**REQ-gju89b**: The software shall reject an Envelope that names an
Organisation other than the one it expected, before decoding the Change set
the Envelope carries. (implements: RC-pm9kmx)
satisfies: derived

**REQ-ag6kqm**: The software shall decide whether to commit a received
Envelope from the Organisation it names, its Sequence number, its Change set
and the Organisation state on the chain alone, and shall check no signature
and no Member's or device's key in doing so. (implements: RC-pm9kmx)
satisfies: derived

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
This item first required the Envelope's signature to verify under the
Published signing key before decoding. The right to change an Organisation's
data lies in its on-chain multisig proxy, and org-node has no concept of an
administrator, so an Envelope carries no signature and the on-chain Membership
root at a newer epoch is the sole authority. The item keeps its ID and states
the rule that replaced it; retiring it is not expressible in the trace gate.

**REQ-8gz8bu**: The software shall reject a received Change set, leaving its
record unchanged, when the epoch of the Organisation state it verified against
is not greater than the epoch of its last commit for that Organisation.
(implements: RC-e5atck)
satisfies: derived

**REQ-nhe2zu**: The software shall, when an Envelope names the expected
Organisation, carries a Sequence number greater than the highest committed,
and carries a Change set whose recomputed Membership root equals the root of
an Organisation state with an epoch greater than the last committed, commit
the applied Change set as its record together with that epoch and that
Sequence number as the new high-water mark.
(implements: RC-6a2dke, RC-e5atck, RC-m4r75s)
satisfies: derived

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`):*
the clause "carries a valid signature under the published signing key" is
removed, with REQ-ag6kqm's amendment.

## Replay

**REQ-6yu72z**: The software shall reject an Envelope whose Sequence number is
not greater than the highest Sequence number it has committed for that
Organisation, before decoding the Change set. (implements: RC-m4r75s)
satisfies: derived

**REQ-mr5abb**: The software shall advance the Sequence-number high-water mark
for an Organisation only when an Envelope has been committed, never on a
rejection at any step, and never to a value lower than the mark it holds.
(implements: RC-m4r75s)
satisfies: derived

## Hostile input

**REQ-9g6as6**: The software shall, for any sequence of bytes offered as an
Envelope, either decode it and the Change set it carries or return a typed
error, and shall not panic. (implements: RC-e2uvje)
satisfies: derived

**REQ-bcxz96**: The software shall, for any sequence of bytes offered as an
Envelope to verification against a record and an Organisation state, either
return a typed error or return a verified update, and shall not panic.
(implements: RC-e2uvje)
satisfies: derived

**REQ-eg5j8u**: The software shall reject with a typed error, before decoding
any of its bytes, a wire frame whose body exceeds 1 MiB, on send and on
receive alike, and shall deliver a frame within that bound unchanged.
(implements: RC-gfn6kr)
satisfies: derived

## Who the Wire message came from

**REQ-xa6smf**: The software shall commit a first admission to an Organisation
that verifies against the chain whichever Device key the connection
authenticated, and shall require no Invite to have been imported for it.
(implements: RC-b6mydy)
satisfies: derived

**REQ-ztdza4**: The software shall commit an update to an Organisation it holds
a record of that verifies against the chain whichever Device key the
connection authenticated, whether or not that key is in the Membership record
before or after the update. (implements: RC-b6mydy)
satisfies: derived

*Amended 2026-10-05 (owner ruling, change `worktree-org-node-chain-authority`).*
REQ-xa6smf first required a first admission's sender to be the administrator's
Device key named by an imported Invite; REQ-ztdza4 required an update's sender
to be in the verified Membership record. The owner ruled that nothing about the
sender is checked: authority is the chain's, Invites leave org-node for the
app, and org-node has no administrator. A chain-valid update delivered by any
peer is harmless because it matches the chain. Both items keep their IDs and
state the rule that replaced them.

## Acting on one's own removal

**REQ-uxv2x2**: The software shall, on committing a Change set that removes
its own Device key from an Organisation's record, delete its record of that
Organisation and mark the Persona revoked. (implements: RC-wqgm2p)
satisfies: derived

## The store at rest

**REQ-hzm4kt**: The software shall write the Persona store only as ciphertext
under a key derived from a passphrase, such that opening the store file with a
different passphrase yields an error and not data, and no member seed, device
seed or Organisation private key appears in the file in clear.
(implements: RC-jjsz97)
satisfies: derived

*Amended 2026-10-06 (owner ruling, change `worktree-org-node-org-key-pair`).*
The item named the Organisation secret, which that change removes in favour of
the Organisation private key (REQ-szq3ud).

## What these requirements do not say

They do not state the base-root rule (REQ-4umsuz, org-members, exported): the
node relies on it at step 5 of verification and it is org-members' behaviour.
They do not state what the chain read returns (REQ-ysyu9g, an expectation on
on-chain-client) nor what a Change set that removes a Device key must do to
the Member-as-a-group key (REQ-q92yac, an expectation on org-members); both
remain expectations, due 2026-12-05, and neither carries `(implements:)` in
this change for the reason the risk file gives. And they do not state anything
about the publish path — ordering the chain write and the local record, or
delivery to the peer — because that path's behaviour is a defect under a
problem report (PR-vt244s) and a requirement would be written against the fix,
not the defect.

And they do not cover every module of the unit. The hazard analysis was run
over the whole of `org-node/src` — `verify.rs`, `sequence.rs`, `envelope.rs`,
`service.rs`, `chain.rs`, `chain_read.rs`, `chain_write/`, `ceremony.rs`,
`transport/`, `blobs.rs`, `store.rs`, `keys.rs`, `ids.rs`, `error.rs` and
`preflight.rs` — while the requirements above state only the behaviour the
nine controls that analysis chose call for, which is the verify-and-commit
path, the frame bound, the sender cross-check, the self-delete and the store
at rest. `ceremony.rs`, `keys.rs`, the blob encoding in `blobs.rs`,
`chain_write/`, `chain_read.rs` — the module PR-hvg2dy is filed against —
`preflight.rs`, the transport handshake and the two small modules `ids.rs`
and `error.rs` have no requirement here; `chain.rs`'s `ChainReader` is
exercised by the verify requirements as the seam the chain read arrives
through, without a requirement of its own. Their requirements,
and the low-level requirements of the whole unit, are tooth 4 of
`docs/plans/2026-09-05-ratchet-gap-analysis.md`.

*Amended 2026-10-05 (owner ruling of that day, change
`worktree-org-node-chain-authority`; written by change
`worktree-person-shared-types`, docs/plans/2026-10-05-switch-trim.md).* "The
sender cross-check" in the list above is no longer a behaviour any requirement
here states: REQ-xa6smf and REQ-ztdza4 are amended in place to say that
nothing about the sender is checked.

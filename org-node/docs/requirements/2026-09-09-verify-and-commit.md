# Requirements — verify-and-commit, the wire, and the store

org-node's first requirements of its own behaviour. Each realises a risk
control from
`org-node/docs/risk/2026-09-09-org-node-hazards.md`
and is worded from behaviour the crate already has; no source behaviour was
changed to fit a requirement. Each is `satisfies: derived` — it exists because
of how the node was built, not because a system-needs document asked for it —
and is assessed in that risk file.

Terms: *Envelope*, *Wire message*, *Sequence number*, *Published signing
key*, *Persona*, *Organisation secret*, *Invite*, *Persona store* and
*Receive operation* are defined in `org-node/docs/CONTEXT.md`;
*Organisation*, *Change set*, *Membership record*, *Membership root*,
*Organisation state*, *Device key* and *Member-as-a-group key* in the root
`docs/CONTEXT.md`.

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

**REQ-ag6kqm**: The software shall reject an Envelope whose signature over the
Organisation identifier, the Sequence number and the Change set bytes does not
verify under the published signing key, before decoding the Change set.
(implements: RC-pm9kmx)
satisfies: derived

**REQ-8gz8bu**: The software shall reject a received Change set, leaving its
record unchanged, when the epoch of the Organisation state it verified against
is not greater than the epoch of its last commit for that Organisation.
(implements: RC-e5atck)
satisfies: derived

**REQ-nhe2zu**: The software shall, when an Envelope names the expected
Organisation, carries a valid signature under the published signing key,
carries a Sequence number greater than the highest committed, and carries a
Change set whose recomputed Membership root equals the root of an Organisation
state with an epoch greater than the last committed, commit the applied Change
set as its record together with that epoch and that Sequence number as the new
high-water mark. (implements: RC-6a2dke, RC-e5atck, RC-m4r75s)
satisfies: derived

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

**REQ-xa6smf**: The software shall, on its first admission to an Organisation
for which it has imported an Invite, reject a received Wire message, leaving
its record unchanged, when the Device key authenticated by the connection
differs from the administrator's Device key that Invite names.
(implements: RC-b6mydy)
satisfies: derived

**REQ-ztdza4**: The software shall, for a Wire message accepted through the
Receive operation that admits it to an Organisation or updates its record of
one — as distinct from the Receive operation that acts on its own removal —
and about an Organisation it already holds a record of, reject the Wire
message after verification and before touching its record when the Device key
authenticated by the connection is not present in the Membership record the
Wire message was verified into. (implements: RC-b6mydy)
satisfies: derived

## Acting on one's own removal

**REQ-uxv2x2**: The software shall, on committing a Change set that removes
its own Device key from an Organisation's record, delete its record of that
Organisation and mark the Persona revoked. (implements: RC-wqgm2p)
satisfies: derived

## The store at rest

**REQ-hzm4kt**: The software shall write the Persona store only as ciphertext
under a key derived from a passphrase, such that opening the store file with a
different passphrase yields an error and not data, and no member seed, device
seed or Organisation secret appears in the file in clear.
(implements: RC-jjsz97)
satisfies: derived

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

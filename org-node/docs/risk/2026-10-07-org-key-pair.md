# Risk analysis — the Organisation key pair (org-node)

Run 2026-10-06 for change `worktree-org-node-org-key-pair`, under IEC 62304
Class C and the acceptability matrix in this ledger's README. The owner's
rulings are in
`org-node/docs/requirements/2026-10-07-org-key-pair.md`:
the Organisation secret is replaced by the Organisation private key, every
Organisation-information Wire message carries it, a receiver keeps it only if
its public half is the chain's `org_pub_key`, and a revocation carries none.
HAZ-ep6uzs and HAZ-vxabf9 are amended in place with dated notes in
`org-node/docs/risk/2026-09-09-org-node-hazards.md`.

## The receipt check

**RC-9cefcn**: the node stores a received Organisation private key only if
the Wire message carries one, it decodes as 32 bytes, and its X25519 public
half equals the `org_pub_key` the chain holds for that Organisation;
otherwise it refuses the whole message, leaving the store unchanged.
mitigates: HAZ-ep6uzs

What it breaks: a genuine update relayed by a peer that strips or alters the
key is refused instead of committed. A relay can drop any message, so the
refusal gives an attacker nothing it lacked; the update arrives again from the
next correct sender. A key from an earlier epoch is refused: the
key pair changes with every update, and each receiver checks the key it is
sent against the `org_pub_key` the chain holds at the epoch it verifies, which
is the update's own (REQ-txvtm9). A receiver that missed updates receives the
current key with the next Organisation information. No new hazard.

## What the change does to the existing hazards

**HAZ-ep6uzs** (a relay substitutes the secret on an admission). With
RC-9cefcn a substituted key is refused: only the holder of the genuine private
key can produce one whose public half is the chain's key, and a relay that
holds the genuine key gains nothing by sending it. The owner-accepted window
recorded on 2026-10-05 closes. The hazard's second half, a non-member's
message shaping a member's record, stays controlled by the root match
(RC-6a2dke, RC-b6mydy). Residual: S3, P1, acceptable under the matrix.

*Restated 2026-10-07 (change `worktree-org-io-commit-workflow`, at its merge
of master `1f52c36`; owner ruling D of 2026-10-06,
`docs/plans/2026-10-06-org-io-roadmap.md`, section "Owner's rulings on the
S2–S4 design drafts").* Ruling D: the README's matrix wins, and S3 is
unacceptable at every probability. This residual is S3/P1 — unacceptable
under the matrix (ruling D); owner ruling pending (flagged 2026-10-07). The
owner reviews it individually (ruling of 2026-10-07), and has not accepted
it. The benefit-risk case put to the owner: what remains is a non-member's message shaping a member's
record, which needs that message's root to match the chain's — that is, a
forged chain state that passes on-chain-client's verifier, the boundary every
trust decision in this unit rests on; refusing every relayed update instead
would stop updates reaching Devices the sender cannot reach directly, which
leaves Devices on stale records (HAZ-vxabf9's ordinary case).

*Owner ruling of 2026-10-07:* residual accepted. The residual stated in the
note above — S3/P1, unacceptable under the matrix (ruling D) — is risk the
owner accepted on 2026-10-07, on the benefit-risk case given there. The
"owner ruling pending" of the note above is settled by this ruling.

*Restated 2026-10-07 after owner ruling R1 (S3a close-out residual review,
decision 16 of `docs/plans/2026-10-06-org-io-commit-workflow.md`; review
round 1 finding-4 of change `worktree-org-io-commit-workflow`).* The two
notes above say the hazard's second half — a non-member's message shaping a
member's record — "stays controlled by the root match (RC-6a2dke,
RC-b6mydy)", and the benefit-risk case argues that "refusing every relayed
update instead" would leave Devices stale. R1 reversed that for an
Organisation the receiver holds: RC-u7kdam
(`org-node/docs/risk/2026-10-07-commit-workflow.md`) acts on an
Organisation-information message about a held Organisation only when the
DevicePublicKey the transport authenticated for the connection (the iroh
endpoint id) is listed in the receiver's current committed record, checked
before the Change set is decoded or the chain is read, refusing without
writing (`sender_listed` in `receive_and_verify`, `org-node/src/service.rs`;
RC-b6mydy as amended keeps the chain checks after it). For a held
Organisation a non-member's message is therefore refused outright; shaping a
member's record now takes a listed Member Device, or its key, and a root that
matches the chain's. The stale-record cost the benefit-risk case weighed is
now RC-u7kdam's "what it breaks": a genuine update from a Device the
receiver's record does not list waits for a Device both records list — a
delay, not a loss.

What remains of the residual is first admission only. A new joiner holds no
record to check a sender against, and accepts the admitting update from any
sender; it commits only what the chain carries (the root match, RC-6a2dke)
and stores the carried key only if its public half is the chain's
`org_pub_key` (RC-9cefcn), so a stranger can deliver the genuine admission or
be refused. The owner ruled on that remainder at R1 (decision 16): a new
joiner accepts the admitting update from any sender "to avoid scenarios
where something happens to the admin's device during this window"; "The
invite id plays no role in the update"; "the new joiner has no org
information to disclose, and they verify the org information they receive
on-chain so the risk here is only a new joiner being DoS'ed which is
acceptable" — that denial-of-service residual is owner-accepted. Beyond it, a
wrong first admission still needs a forged chain state that passes
on-chain-client's verifier, as before. Severity S3, probability P1,
unchanged as a rating. The owner's acceptance of this residual on 2026-10-07
(the ruling above) stands; it now rests on this smaller residual — first
admission alone — rather than on every relayed update. RC-u7kdam is added to
the hazard's controls (its `mitigates:` line names HAZ-ep6uzs from this
date).

**HAZ-vxabf9** (a revoked Device keeps what it held). Before this change it
kept the opaque Organisation secret, which nobody checked. Now it keeps the
Organisation private keys of the epochs up to its removal. The update that
removes it draws a fresh key pair (REQ-stx9v3), and that key and every later
one go only to the Devices of the new record (REQ-3dsweu), so the removed
Device holds no key the Organisation uses after its removal. Severity
unchanged (S3); probability of exposure of post-removal material P1; residual
acceptable. A Device that is removed while offline still holds what it held,
as before. (Owner ruling 2026-10-06: rotation is part of calculating a new
root; the earlier acceptance of a never-rotated key is withdrawn and PR-g9u3xq
is resolved in this change.)

*Restated 2026-10-07 (change `worktree-org-io-commit-workflow`, at its merge
of master `1f52c36`; owner ruling D of 2026-10-06, as above).* "Residual
acceptable" is withdrawn: the residual — a removed Device keeps the
Organisation private keys of the epochs up to its removal, and a Device
removed while offline keeps what it held — is S3/P1 — unacceptable under
the matrix (ruling D); owner ruling pending (flagged 2026-10-07). The owner
reviews it individually (ruling of 2026-10-07), and has not accepted it. The
benefit-risk case put to the owner: every
Member's Device must hold the current private key for the key pair to serve
at all, and a key already delivered cannot be taken back; rotation on every
update (REQ-stx9v3) is the strongest control available at this layer, and it
bounds the exposure to material from before the removal. Refusing the
function would mean no Organisation key pair, which leaves the
unchecked-secret exposure this change removed. Stage S3 narrows the residual
further: the removed Device receives only an absence proof, not the Change
set (RC-r8bp43), and a cooperating Device deletes the keys it holds
(RC-wqgm2p as amended).

*Owner ruling of 2026-10-07:* residual accepted for the key itself. The
residual stated in the note above — a removed Device keeps the Organisation
private keys of the epochs up to its removal, S3/P1, unacceptable under the
matrix (ruling D) — is risk the owner accepted on 2026-10-07, on the
benefit-risk case given there. The continuous group key agreement (CGKA)
boundary — forward and post-compromise secrecy across rotations, so that a
key held by a removed or compromised Device gives no access to material
under other epochs' keys — is recorded as ODS Phase 3's to address, not this
stage's. The "owner ruling pending" of the note above is settled by this
ruling.

## Derived requirements assessment

**REQ-szq3ud** (every Organisation-information message carries the sender's
key; the caller supplies none): widens who holds the key from the founder to
every Member's Devices, which is the design's intent (design point 5). The
exposure of a removed Device is HAZ-vxabf9, assessed above. Taking no key from
the caller removes a path by which software above org-node could send a wrong
or foreign key. No new hazard.

**REQ-stx9v3, REQ-jy6ybw** (a fresh key pair with every provisional update's
root; the sender's record takes it on commit): realise rotation, which narrows
HAZ-vxabf9 as assessed above. What it breaks: a provisional update that is
discarded or never verifies leaves its private key unused, and it is dropped
with the update (REQ-hhva9d, REQ-uv3v5w). A key that was published on the
chain but whose update the sender never commits (crash after the chain write)
is still in the provisional update in the encrypted store, and the commit
retried later uses it (REQ-tqap3r). The record keeps only the current key;
nothing encrypts under the key pair yet, so no earlier key is needed. No new
hazard.

**REQ-c29s93, REQ-bwx7eg** realise RC-9cefcn; assessed above. The parse check
(REQ-c29s93) runs before the chain read, so a stranger's malformed message
costs no chain read (RC-mj6gjq unchanged).

**REQ-ju6vn2** (the received key is stored on commit): the key reaches the
encrypted store only (RC-jjsz97) and is redacted in diagnostics (RC-8a4xjb,
REQ-y7tsft as amended). No new hazard.

**REQ-3dsweu** (Organisation information only to a Device the committed
record lists, a revocation with no key to any other; a revocation is refused
by a Device the verified record still lists): narrows where the key goes, from
"whoever the caller names" to the Devices of the committed record, so the key
never leaves for a Device the chain-valid record does not list (a wrong peer
address in Loopback mode, PR-2dmjzj, still reaches whoever holds that
address). Resolves PR-xwek5e, where a revocation cleared a remaining Member's
secret: a remaining Member never commits a revocation now. A relay that
relabels Organisation information as a revocation, to hold a listed Device on
the previous epoch's key, is refused by that Device, which takes the update
from the next Organisation information; a first admission so relabelled is
refused too (REQ-vxqc5g). What it breaks: nothing a correct sender does, since
a correct sender sends a revocation only to an unlisted Device. No new hazard.

**REQ-8amu2a** (amended: an expectation names the Organisation, not the invite
identifier): RC-2ferct already named only the Organisation, so its text and
residual are unchanged. A stranger who knows an Organisation this node expects
to join can now cost it one chain read without knowing the invite identifier;
the inviter knew both before, and every Member knows the Organisation. The
first admission still commits only if the verified record lists one of the
node's own unbound Personas (REQ-kt877x), so nothing the stranger sends is
committed unless the chain published it. Residual S3, P1, as RC-2ferct states.

**REQ-vxqc5g** (a revocation for an Organisation the node holds no record of
is refused): narrows what can create a record. No new hazard.

**REQ-hzm4kt, REQ-y7tsft** (amended: "Organisation secret" becomes
"Organisation private key"): a rename of the secret they protect; their
controls are unchanged.

assesses: REQ-8amu2a, REQ-stx9v3, REQ-jy6ybw, REQ-szq3ud, REQ-c29s93, REQ-bwx7eg, REQ-ju6vn2, REQ-3dsweu, REQ-vxqc5g, REQ-hzm4kt, REQ-y7tsft

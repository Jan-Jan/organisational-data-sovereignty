# S3 — the commit workflow

**Goal:** A node commits only what the chain holds, keeps the Change set that produced its record, reconciles with the chain at startup, tells a revoked Device only by an absence proof, and deletes an Organisation's data only on a verified revocation, after signing an acknowledgement.
**Implements:** S3a (org-node): REQ-ps2gy2, REQ-qrtsc9, REQ-m2xh8q, REQ-em28bq, REQ-y99c9w, REQ-b462sh, REQ-tb4f8p; REQ-uv3v5w, REQ-uxv2x2, REQ-3dsweu as amended; RC-r8bp43, RC-ub82my, RC-eydn8t, RC-44vvjp, RC-wqgm2p as amended; SDD-uck4tz (LLR-kr5t6f, LLR-r7zm39, LLR-tx8ruv, LLR-r8qhky, LLR-uw7nmv, LLR-xgefn8, LLR-gbe9bt, LLR-hby4jr, LLR-5azhry as amended 2026-10-07), SDD-kwncn7 (LLR-dc45ur, LLR-378cj4, LLR-js9dsu), SDD-af5vnt (LLR-pba7yu, LLR-d9778a), SDD-swtd3w (LLR-n67aw8), SDD-8cpyfa (LLR-gr8x3r, LLR-fm38ww, LLR-a8z7r5, LLR-pt32fx, LLR-38e2kn, LLR-mkj4bz; from the owner rulings of 2026-10-07: LLR-2r2fha, LLR-kzgjz8, LLR-3aysup, and LLR-u6rq4s, LLR-3q63zv as amended), SDD-rx2yvy (LLR-6ymd6d, LLR-8hdu9x), SDD-72ddm6 (LLR-23sfdh, LLR-6p4pj2, LLR-b27jr6, LLR-jsx922); resolves PR-qmvj83 and the org-node half of PR-vt244s. From the owner rulings at the S3a close-out residual review (2026-10-07, decision 16): REQ-ea4qs5, REQ-ztdza4 and REQ-b462sh as amended, RC-u7kdam, RC-b6mydy and RC-eydn8t as amended; PR-u4c2vp's fix. S3b-io (org-io, after S2 merges): the eleven proposed org-io requirements below; resolves PR-vt244s and PR-924ftr.
**Safety class:** C (org-node; org-io is class C by owner ruling). No per-item override.
**Merge order** (owner ruling of 2026-10-07): S3a (org-node, T0–T12) merges early, before S2; S3b-io (org-io, T13–T17) follows after S2 merges, as a follow-up change.
**Verification:** org-node's `verify_commands` (`org-node/.guardrails/config.yaml`): the `cargo test -p org-node --features app,test-support --lib --test …` line (T0 adds `revocation`, `reconcile` and `commit_workflow_errors` to it), `quint --version`, the two `quint typecheck` lines and the five `quint run org-node/quint/protocol.qnt --invariant=… --max-steps=16 --max-samples=5000` lines. Set `QUINT_HOME` to a writable directory such as `$TMPDIR/quint_home` (`~/.quint` is read-only by design); on a fresh `QUINT_HOME` the first run may fail while the evaluator downloads — run it again. Fetch new crates with `CARGO_HOME=/tmp/cargo_home_fuzz` (`~/.cargo` is read-only here). Every task also runs `cargo clippy -p org-node --features app,test-support --lib --test <the task's targets> -- -D warnings` (narrowed 2026-10-07 at T0: `--all-targets` is red before S3 in six existing test targets — clippy 1.99 reports unwrap_used, expect_used and panic in fuzz_verify_against_chain, key_custody, value_types, store_at_rest, transport_handshake and service_stories; no gate runs it; T12 books it as a problem report); T8 and T9 also run the app's `verify_commands` (`app/.guardrails/config.yaml`: `cargo test --manifest-path app/src-tauri/Cargo.toml --features test-support --test startup_policy … --test invitation`, `npm --prefix app run check`, `npm --prefix app run test`). S3b-io adds org-io's `verify_commands` once S2 has written them.

Stage S3 of `docs/plans/2026-10-06-org-io-roadmap.md`, change
`worktree-org-io-commit-workflow`. Requirements, risk and org-node's design
(`org-node/docs/architecture/2026-10-07-commit-workflow.md`:
SDD-uck4tz, one combined module by owner ruling, and eighteen LLRs); no code. It
resolves PR-vt244s, PR-qmvj83 and PR-924ftr, and builds on two changes:
`worktree-org-node-org-key-pair` (key pair, two Wire message kinds, rotation
per update), merged to master as `1f52c36` and into this branch on
2026-10-07; and stage S2 (creates org-io, moves the chain read and the write
into it), not yet merged.

Drafted in org-node, with minted IDs:

- `org-node/docs/requirements/2026-10-07-commit-workflow.md`:
  REQ-ps2gy2, REQ-qrtsc9, REQ-m2xh8q, REQ-em28bq, REQ-y99c9w, REQ-b462sh,
  REQ-tb4f8p.
- `org-node/docs/risk/2026-10-07-commit-workflow.md`:
  HAZ-p6xkuz, HAZ-7ubhwz, HAZ-h6b34a; RC-r8bp43, RC-ub82my, RC-eydn8t,
  RC-44vvjp.
- Amended in place: REQ-uv3v5w (keeps the Change set), REQ-uxv2x2 and
  RC-wqgm2p (delete everything, the Persona included, on either verified
  path).

## What the key-pair change already covers

*Re-checked 2026-10-07 after merging master `1f52c36`; the text below holds
of the merged code.* `worktree-org-node-org-key-pair` splits `WireMessage` into
`OrgInformation { envelope, record_snapshot, org_private_key }` and
`Revocation { envelope }`, and chooses the kind by recipient: a Device the
committed record no longer lists gets `Revocation` (REQ-3dsweu, LLR-6ymd6d,
LLR-8hdu9x as amended there). So the literal defect of PR-qmvj83 — every
member's snapshot sent to the revoked Device — is gone, and so is the key.
It also rotates the key pair on every update (REQ-stx9v3, REQ-jy6ybw), which
is most of what the roadmap calls S3b. At its merge it added a note to
PR-qmvj83: the disclosure of every Member's record is gone, and the item
stays open for S3's absence-proof fix (this change adds a note of what
remains).

What remains for S3:

1. `Revocation` still carries the Envelope, i.e. the Change set. With a
   batch of one it holds only the revoked Member's own leaf or MemberId, but
   the key-pair change defines a provisional update as a batch, and then the
   Change set carries every other leaf the batch upserts. The owner's ruling
   is "only an absence proof". REQ-ps2gy2 replaces the Envelope with the
   proof.
2. The revoked Device verifies `Revocation` "as any update": it needs the
   base record of the Change set, and the chain must still hold that
   update's root (REQ-txvtm9). A Device one epoch behind, or one that reads
   the chain after a later update, can never verify its own revocation.
   REQ-m2xh8q verifies the proof against the current root instead, with the
   epoch floor of RC-ub82my.
3. The self-delete marks Personas Revoked and keeps their keys; S3 deletes
   them (REQ-uxv2x2 amended) after signing the acknowledgement (REQ-y99c9w).
4. No acknowledgement exists; REQ-y99c9w and REQ-b462sh add it.
5. The fan-out to every Device (the app's PR-3ue4va, opened by the key-pair
   change) is **stage S4's**, in Networked mode (roadmap, "Owner's rulings
   on the S2–S4 design drafts", accepted recommendations). S3's part is that
   what is sent is exactly the committed update and its revocation notices
   (proposed 5 below); S4 sends it to every listed Device.

Amended in place on 2026-10-07, after the merge: REQ-3dsweu (the revocation
kind carries the notice; its "still listed" refusal is REQ-m2xh8q's),
LLR-js9dsu, LLR-j5vbqj, LLR-6ymd6d, LLR-4kh9w9, LLR-38e2kn, LLR-pt32fx
(`org-node/docs/*/2026-10-07-org-key-pair.md`), LLR-8hdu9x and LLR-jsx922
(decomposition), and PR-qmvj83's note. LLR-mkj4bz follows REQ-uv3v5w in this
change's design. S3b is retired: the roadmap already marks its row as
absorbed by the key-pair change (decision 6 below).

The merge also restated master's key-pair residuals under ruling D: HAZ-vxabf9
(the keys a removed Device keeps) and HAZ-ep6uzs, which read "S3, P1,
acceptable", are now stated as "S3/P1 — unacceptable under the matrix
(ruling D); owner ruling pending (flagged 2026-10-07)", with the
benefit-risk case put to the owner
(`org-node/docs/risk/2026-10-07-org-key-pair.md`,
`org-node/docs/risk/2026-09-09-org-node-hazards.md`). The owner ruled on
2026-10-07 not to accept them wholesale but to review each one, and then,
the same day, accepted both: HAZ-ep6uzs's residual, and HAZ-vxabf9's for the
key itself, with the CGKA boundary (forward and post-compromise secrecy
across rotations) recorded as ODS Phase 3's to address, not this stage's
(decision 14; dated notes in both files).

The device secret key (owner rulings recorded by S4,
`docs/plans/2026-10-06-org-io-transport.md` §13–§15 on branch
`worktree-org-io-transport`): the device ed25519 seed lives only in the OS
keychain, the store holds only the Device public key, and org-node receives
the seed transiently for store-key derivation and signing. S3's signing
(REQ-y99c9w, LLR-hby4jr, RC-44vvjp) is restated as "given the seed as a
transient value"; S3 assumes neither store layout, since S4 makes that change.

## Proposed org-io requirements, to be minted once S2 creates the unit

Numbered for reference here only; these are not IDs. Each becomes a REQ in
org-io's requirements ledger, `satisfies: derived` unless noted, with an
`assesses:` passage in org-io's risk file.

1. **Order of the publish path.** org-io shall submit a chain write for an
   Organisation only for a provisional update org-node holds in the store,
   shall commit it through org-node only after reading the chain and finding
   the update's root and Organisation public key there, and shall send it to
   any peer only after it is committed. (PR-vt244s)
2. **A write whose outcome is unknown.** When a chain write times out or
   its outcome is otherwise unknown, org-io shall keep the provisional
   update and re-read the chain until it holds that update's root (then
   commit, 1), holds another root at a newer epoch (then reconcile, 3), or
   the app discards the update; it shall not submit another write for that
   Organisation meanwhile. (PR-vt244s, PR-924ftr first half)
3. **Startup reconcile.** At start, and whenever a read shows the chain at a
   newer epoch than a held record, org-io shall read the latest finalised
   Organisation state for each held Organisation and hand it to org-node's
   reconcile (REQ-tb4f8p): on "committed" it fans the update out (5); on
   "behind" it reports the Organisation as behind to the app (and, from S6,
   asks a peer in good standing); on a refusal it changes nothing and
   reports it. When it cannot reach the chain it keeps working on the last
   verified record, reports the Organisation to the app as unverified,
   retries the reconcile, and deletes and commits nothing until a read
   succeeds (REQ-em28bq; owner ruling of 2026-10-06, decision 5 below).
4. **Discard.** org-io shall offer the app a command to discard a named
   provisional update (org-node's REQ-hhva9d), refused while a write of that
   update is in flight. (PR-924ftr second half)
5. **Fan-out of exactly the committed update.** After a commit, org-io shall
   send the committed update — never one the caller names — as Organisation
   information to every Device the committed record lists other than its
   own, and as a revocation (REQ-ps2gy2) to every Device the previous record
   listed and the committed one does not. (PR-924ftr third half) *The
   sending to every listed Device is S4's fan-out (PR-3ue4va, Networked
   mode); whichever of S3b-io and S4 merges second mints this as one
   requirement with S4's.*
6. **Delivery of revocations.** org-io shall retry sending a revocation to a
   revoked Device, with backoff, until it receives that Device's valid
   acknowledgement or the Organisation's revocation resend limit (11) has
   passed since the commit, and then stop and leave the entry
   unacknowledged. (PR-vt244s: "a removal no device is ever told about";
   owner ruling C, decision 2 below)
7. **Receiving a revocation.** On a received revocation org-io shall read the
   latest finalised Organisation state at that moment and hand both, with
   the device seed of each Persona bound to that Organisation moved in from
   the OS keychain as a transient value (REQ-y99c9w), to org-node
   (REQ-m2xh8q). When org-node accepts it, org-io shall write the sealed
   store bytes org-node returned — or, with one store per Persona (S4),
   delete that Persona's store — and delete every piece of data it holds for
   that Organisation (peer addresses, queued messages, caches) in the same
   step, except what proposed 8 keeps for delivery (the Persona's keychain
   item among it, revised 2026-10-07), tell the app to delete its own Organisation data, and send the
   acknowledgements org-node returned (REQ-y99c9w) only after the app
   confirms that deletion, so "acknowledged" means the whole device was
   cleared (owner ruling of 2026-10-06, decision 4 below). Before deleting,
   it reads the Organisation's acknowledgement retry limit (11) and up to
   three distinct Member DevicePublicKeys from the record — the Device that
   delivered the revocation first, then up to two others the record lists,
   other than its own — which it keeps with the acknowledgement for as long
   as proposed 8 allows and no longer. (org-node holds the record and
   deletes it; how it hands those keys out with the acknowledgements is
   designed in T13: a values-out addition to `Accepted` and to the removal
   step.)
8. **Sending the acknowledgement.** org-io shall deliver each
   acknowledgement to up to three distinct Member Devices — the one that
   delivered the revocation first, then the others it kept (7) — fewer when
   the record listed fewer. It connects directly to each, under the revoked
   Device's own endpoint identity (its Device secret key, the Persona's
   keychain item), never through a relay and never from a fresh endpoint,
   so each receiver sees the Device the acknowledgement names as the sender
   (REQ-b462sh as amended). A Device that refuses — one that has not yet
   committed the revocation still lists the sender and refuses
   (`AcknowledgementForListedDevice`) — or cannot be reached is retried with
   backoff while the next kept Device is tried. It shall keep for this only
   the acknowledgement, the Device secret key, the kept DevicePublicKeys and
   the retry limit, and only until three Devices have accepted, or every kept
   Device has been tried and none remains to retry, or the Organisation's
   acknowledgement retry limit (11) has passed since the deletion, whichever
   comes first; at that moment it deletes all of them, the keychain item
   included. After it, the device holds nothing about that Organisation: no
   key, no copy of the acknowledgement, no peer address, no tombstone and no
   marker that it was ever a member. (owner ruling C, the owner's ruling on
   retention, decision 1, and rulings R5 and the acknowledgement sender rule
   of 2026-10-07, decision 16: "After delivering the acknowledgement, the
   local device should delete all the information. To be sure of delivery,
   the device delivers the acknowledgement to 2 more peers (if they have as
   many peers)"; "Acknowledgements of revocations will be sent from devices
   on the revocation list, so their ids can be checked before accepting.")
9. **Receiving an acknowledgement.** org-io shall accept an inbound
   connection from a Device the Organisation's record no longer lists for
   an acknowledgement only (a transport requirement shared with S4: every
   other message from such a Device is refused, RC-u7kdam), hand each
   acknowledgement with the connection's authenticated DevicePublicKey to
   org-node (REQ-b462sh as amended 2026-10-07: accepted only from the
   Device it names), and append each verified one to an append-only list in
   its storage, which stage S5 turns into the CRDT revoked-device list; it
   drops none (owner ruling of 2026-10-06, decision 3 below). S5's gossip of
   that list verifies each carried acknowledgement's signature before
   accepting an update (owner ruling R4), and prunes entries by age alone,
   acknowledged or not.
10. **No deletion on any other trigger.** org-io shall delete no Organisation
    data except under 7. (implements, once minted: RC-ub82my's org-io half)
11. **Retry limits set by the Organisation.** org-io shall hold, per
    Organisation, an acknowledgement retry limit (how long a revoked Device
    retries its acknowledgement) and a revocation resend limit (how long a
    node resends a revocation to an unacknowledged revoked Device), each a
    duration that defaults to 2 months when the Organisation sets none, and
    shall refuse a zero duration. Until stage S5, both live in org-io's own
    storage, per Organisation, set through the app by an admin; the revoked
    Device uses the value it holds when it deletes, and keeps it only as
    proposed 8 allows. org-node needs neither value. (owner rulings C and of
    2026-10-06 on where the limits live, decision 9 below)

## Reproducing tests (resolve-problem, red first)

- PR-vt244s: write the chain, crash before commit (drop the service), start
  again; assert the reconcile commits the kept update and the record's epoch
  equals the chain's (REQ-tb4f8p, proposed 3).
- PR-qmvj83: revoke one Device in a two-Member Organisation; assert the
  message the revoked Device receives decodes to a revocation holding no
  Change set, no snapshot, no key, and no leaf of the other Member
  (REQ-ps2gy2).
- PR-924ftr: time out a write that then executes, start a second admission;
  assert the update sent is the one the chain holds, to every listed Device
  (proposed 2, 5), and that the discard command removes a named update and
  its key (proposed 4).

## Decisions — ruled by the owner on 2026-10-06

The owner ruled on this draft's open decisions in
`docs/plans/2026-10-06-org-io-roadmap.md`, section "Owner's rulings on the
S2–S4 design drafts (2026-10-06)", and, later the same day, on the three
points the design pass raised (9 to 11 below, and the correction to 1). Each
is recorded here with where it landed.

1. **What the revoked Device keeps to send its acknowledgement, and for how
   long.** Ruled (C), and the rest of the recommendation **confirmed with a
   correction** by the owner's ruling on retention: "Once the device has
   deleted all the local org related data and sent back the signed
   acknowledgement, it should keep nothing with regards to that org
   anymore." It signs the acknowledgement, then deletes everything including
   the Device secret key at once; until delivery it keeps only what is needed
   to deliver it — the acknowledgement, the address of the peer that
   delivered the revocation, and the retry limit; it sends first on that
   same connection, then retries from a fresh endpoint; at delivery, or when
   the Organisation's acknowledgement retry limit has passed (default 2
   months, set by the Organisation), whichever comes first, it deletes those
   too, and from then on holds nothing about the Organisation — no copy of
   the acknowledgement, no peer address, no tombstone, no "was a member"
   marker. An undelivered acknowledgement leaves the entry unacknowledged in
   the Organisation's list. The correction: the draft deleted the retained
   items at the limit only, "whether or not delivered"; they now go at
   delivery too. Proposed 8 and 11; org-node's half is LLR-pba7yu; the
   residual is assessed under HAZ-h6b34a in the risk draft.
2. **How long a node resends a revocation to an unreachable revoked
   Device.** Ruled (C): with backoff until acknowledged, until the
   Organisation's revocation resend limit has passed (default 2 months, set
   by the Organisation); then it stops and leaves the entry unacknowledged.
   From S5 every node in good standing can answer the Device with a proof,
   so the limit only bounds one node's effort. Proposed 6 and 11.
3. **A verified acknowledgement before S5.** Ruled as recommended: org-io
   keeps it in an append-only list in its storage, which S5 turns into the
   CRDT list. Proposed 9. (This is the receiving node's list, not the
   revoked Device's; 1 governs the revoked Device.)
4. **"Delete everything" and the app.** Ruled as recommended: it includes the
   app's Organisation data, and the acknowledgement is sent only after the
   app confirms the deletion. Proposed 7.
5. **A device that cannot reach the chain.** Ruled as recommended: it keeps
   its last verified record, marked unverified, and never deletes or commits
   until a read succeeds. Proposed 3; org-node's half is REQ-em28bq.
6. **S3b.** Settled by the roadmap: the key-pair change absorbed it, and the
   roadmap's row is kept as history.
7. **Which reading of the acceptability matrix governs.** Ruled (D): the risk
   README's matrix wins, S3 is unacceptable at every probability, and each
   S3 residual is stated as unacceptable under the matrix, with its
   benefit-risk case. Whether the owner accepts a residual is ruled per
   residual, not by ruling D (owner ruling of 2026-10-07; decisions 14 and
   15). The risk draft is restated accordingly.
8. **Values in, values out (ruling B).** org-node's requirements in this
   change take every Organisation state as a value the caller supplies, and
   return the sealed store bytes, the revocation messages and the
   acknowledgements as values; org-node reads no chain and writes no file.
   The design draft
   (`org-node/docs/architecture/2026-10-07-commit-workflow.md`)
   follows that rule.
9. **Where the Organisation's two retry limits live.** Ruled as recommended:
   in org-io's storage, per Organisation, set through the app by an admin,
   default 2 months; org-node needs neither value. Proposed 11.
10. **One module or two.** Ruled: one combined pure module,
    `org-node/src/revocation.rs` (SDD-uck4tz), covering the notices, their
    acceptance, and signing and checking acknowledgements.
11. **Retention after delivery.** Ruled as quoted under 1.
12. **The store layout change** (`kept_change_set` on every Organisation
    record). Ruled 2026-10-07: accepted, with a new golden pin and no
    migration; stores written before S3 do not load. Task T2.
13. **Merge order.** Ruled 2026-10-07: S3a (org-node, T0–T12) merges early,
    before S2; S3b-io (T13–T17) follows after S2 merges, as its own change.
14. **The key-pair change's residuals.** Ruled 2026-10-07: not accepted
    wholesale under ruling D; the owner reviews each one. They are stated as
    "S3/P1 — unacceptable under the matrix (ruling D); owner ruling pending
    (flagged 2026-10-07)" and listed below. The chain-authority residual at
    `org-node/docs/risk/2026-10-06-chain-authority.md:70` is restated the
    same way in T12, not pre-accepted. *Ruled later on 2026-10-07:*
    HAZ-ep6uzs's residual accepted; HAZ-vxabf9's residual accepted for the
    key itself, with the CGKA boundary (forward and post-compromise secrecy
    across rotations) recorded as ODS Phase 3's to address, not this
    stage's. Recorded as dated notes in
    `org-node/docs/risk/2026-10-07-org-key-pair.md` and
    `org-node/docs/risk/2026-09-09-org-node-hazards.md`.
15. **S3's own residuals.** Ruled 2026-10-07: the residuals of HAZ-p6xkuz,
    HAZ-7ubhwz and HAZ-h6b34a are not owner-accepted; their status is
    "owner ruling pending", and the owner reviews each individually at the
    S3a close-out (T12), together with the chain-authority residual at
    `org-node/docs/risk/2026-10-06-chain-authority.md:70`. The risk draft
    states them so. *Ruled 2026-10-07 at that review: decision 16.*
16. **The S3a close-out residual review.** Ruled 2026-10-07, one ruling per
    item, recorded where each item lives:
    - **R1, the member-sender rule** (reverses, for an Organisation the node
      holds, the chain-authority change's "commit whoever sent it"). A
      membership update (Organisation information) for an Organisation the
      receiver holds, and a revocation, are acted on only when the sending
      Device — authenticated by the transport as its DevicePublicKey, the
      iroh endpoint id — is listed in the receiver's current committed
      record; the check comes before any chain read and refuses without
      writing. "Using iroh a connection can only be established via
      mutually known public keys, but I agree with only accepting updates
      and revocations from members. Furthermore, these are checked to be
      well formed before acting on them, which gives us another layer of
      protection." On revocations: "the receiver still thinks of the other
      party as a member (based on the information they have) so nothing is
      different here." *First admission, amended by the owner the same
      day:* a new joiner accepts the admitting update from any sender, "to
      avoid scenarios where something happens to the admin's device during
      this window"; "The invite id plays no role in the update"; "the new
      joiner has no org information to disclose, and they verify the org
      information they receive on-chain so the risk here is only a new
      joiner being DoS'ed which is acceptable" — that residual is
      owner-accepted. *Acknowledgements, ruled the same day:*
      "Acknowledgements of revocations will be sent from devices on the
      revocation list, so their ids can be checked before accepting" — an
      acknowledgement is accepted only when the sending Device is the Device
      it names and the receiver's record no longer lists it, checked before
      the signature. Landed: REQ-ea4qs5, RC-u7kdam, LLR-2r2fha, LLR-kzgjz8,
      LLR-3aysup (new); REQ-ztdza4, REQ-b462sh, RC-b6mydy, RC-eydn8t,
      LLR-5azhry, LLR-u6rq4s, LLR-3q63zv, SDD-8cpyfa, SDD-72ddm6 (amended);
      PR-u4c2vp (note). Code and tests: task T12a.
    - **R2, the epoch rule:** keep it — the proof against the chain's
      current root, `StaleChainState` when the chain is behind the record.
      No change (note at REQ-m2xh8q and in the risk draft).
    - **R3, HAZ-p6xkuz:** "a device only deletes everything after they
      verified the revocation proof against the on-chain information (from
      a peer that is as far as they know still a member of the org). If
      forced to choose between failure modes it is better for a device to
      delete everything about and related to an org, than it is for it to
      retain any information after it has been revoked, BUT the code should
      ideally make this failure if not impossible then extremely unlikely."
      R1 is a new control for it (RC-u7kdam); residual restated with it and
      accepted on the owner's ordering of the failure modes.
    - **R4, HAZ-7ubhwz:** "Upon gossiping a CRDT update to the revocation
      list containing an acknowledgement, peers should verify the signature
      before accepting. Change the risk file accordingly. Pruning happens a
      certain amount of time regardless of acknowledgement." Recorded as a
      constraint on S5's list (signature checked on every gossiped
      acknowledgement; pruning by age alone); the "shorter retention for
      acknowledged entries" text is removed; residual restated and accepted.
    - **R5, HAZ-h6b34a:** "After delivering the acknowledgement, the local
      device should delete all the information. To be sure of delivery, the
      device delivers the acknowledgement to 2 more peers (if they have as
      many peers)" — up to three distinct Member Devices in all, connected
      to directly under the revoked Device's own identity (compatible with
      the acknowledgement sender rule), then everything is deleted — the
      Device secret key, the acknowledgement, the addresses — bounded by
      the Organisation's limit (ruling C, default 2 months). Proposed 7, 8
      and 9 and T16 amended; residual restated and accepted.
    - **R6, chain-authority.md:70 (HAZ-bedm57):** "Strangers cannot make the
      node read the chain, because org updates come from other members (not
      strangers), even the new joiner knows the device key of the admin.
      Furthermore, member devices will be tracking the chain in any case for
      updates, so this hazard is irrelevant." R1 is the control; dated note
      at `org-node/docs/risk/2026-10-06-chain-authority.md:70` restates the
      residual and records the ruling, with the first-admission denial of
      service accepted under R1's amendment.

## Open for the owner

Items 1 and 2 were ruled on 2026-10-07 (decision 14), and items 3 to 6 at
the S3a close-out residual review the same day (decision 16); all are kept
as the case the owner ruled on. Nothing is open.

1. **HAZ-ep6uzs's residual** (key-pair change). *Ruled 2026-10-07: residual
   accepted.* *Hazard:* a relay
   substitutes the Organisation key on an admission, and a non-member's
   message shapes a member's record. *Control:* RC-9cefcn refuses a carried
   key whose X25519 public half is not the chain's `org_pub_key`, so the
   substitution half is closed; the second half stays controlled by the root
   match (RC-6a2dke, RC-b6mydy). *What remains:* a non-member's message is
   committed only if its root matches the chain's, which takes a forged
   chain state that passes on-chain-client's verifier — S3/P1, unacceptable
   under the matrix. *Recommendation:* accept. The remaining route sits on
   the chain-verifier boundary every trust decision in org-node already
   rests on; refusing relayed updates instead leaves Devices on stale
   records, HAZ-vxabf9's ordinary case. Case in
   `org-node/docs/risk/2026-10-07-org-key-pair.md`.
2. **HAZ-vxabf9's key-retention residual** (key-pair change). *Ruled
   2026-10-07: residual accepted for the key itself; the CGKA boundary
   (forward and post-compromise secrecy across rotations) is ODS Phase 3's
   to address, not this stage's.* *Hazard:* a
   removed Device keeps acting as a member with what it holds. *Control:*
   rotation on every update (REQ-stx9v3) and sending the new key only to
   the Devices of the new record (REQ-3dsweu); S3 adds the proof-only
   revocation (RC-r8bp43) and, on a cooperating Device, deletion of
   everything (RC-wqgm2p). *What remains:* a removed Device keeps the
   Organisation private keys of the epochs up to its removal, and a Device
   removed while offline, or one modified not to cooperate, keeps everything
   it held — S3/P1, unacceptable under the matrix. *Recommendation:* accept
   for the key itself, with the Phase 3 (CGKA) boundary recorded. A key
   already delivered cannot be taken back. Rotation bounds the exposure to
   material from before the removal, and nothing encrypts under the key
   pair yet. Case in the same file and in
   `org-node/docs/risk/2026-09-09-org-node-hazards.md`.
3. **The chain-authority residual** at
   `org-node/docs/risk/2026-10-06-chain-authority.md:70`. *Ruled
   2026-10-07 (R6): residual accepted with R1 (RC-u7kdam) as its control;
   the first-admission denial of service to a new joiner accepted on the
   owner's reasoning (decision 16).*
4. **HAZ-p6xkuz's residual** (S3). A forged chain state at least as new as
   the device's record, passing on-chain-client's verifier, triggers a
   deletion — S3/P1, unacceptable under the matrix. *Ruled 2026-10-07 (R3):
   residual accepted, restated with R1 (a listed sender is now needed too),
   on the owner's ordering: deleting wrongly is preferred to retaining
   after revocation, with the code making it extremely unlikely.* Case in
   `org-node/docs/risk/2026-10-07-commit-workflow.md`.
5. **HAZ-7ubhwz's residual** (S3). A revoked Device altered to sign the
   acknowledgement without deleting, whose entry a later retention rule
   prunes — S3/P1, unacceptable under the matrix. *Ruled 2026-10-07 (R4):
   residual accepted with the control changed — signatures checked on
   gossip (S5), pruning by age alone, and the acknowledgement sender rule.* Case in the same file.
6. **HAZ-h6b34a's residual** (S3). Until delivery or the retry limit, a lost
   or stolen revoked Device holds one peer's address and its own signed
   acknowledgement — S3/P1, unacceptable under the matrix. *Ruled
   2026-10-07 (R5): delivery to up to three Member Devices under the
   Device's own identity, then delete everything; residual (up to three
   endpoint ids, the acknowledgement and the Device secret key, within the
   limit) accepted.* Case in the same file.

## Implementation plan

Written 2026-10-07 with the plan-change skill, after this branch merged master
`1f52c36` (the key-pair change) and reconciled the design with it. Execute
with `develop-change`, task by task, red test first; then
`check-traceability`, `verify-before-merge` and `merge-change`.

The plan has two parts:

- **S3a — org-node (can be developed now).** Everything IO-free that S3 adds
  to org-node, and the interim wiring of `OrgService` and the app that keeps
  today's async, chain-reading service working until S2 and S4 reshape it.
  The new functions are values in, values out (ruling B), so S2 and S4 move
  only the wiring, not the decisions.
- **S3b-io — org-io (after S2 merges).** S2 creates the crate. At the start
  of S3b-io the eleven proposed org-io requirements above are minted with
  `.guardrails/scripts/new-id.sh REQ` in org-io's ledger (these are new
  items; ruling A concerns moved ones), with their `assesses:` passages and
  org-io's control for HAZ-h6b34a's retained state. Proposed 5's fan-out is
  merged with S4's PR-3ue4va requirement by whichever of S3b-io and S4 lands
  second; the transient device seed comes from S4's keychain path.

Conventions for every S3a task:

- Tests live in `org-node/tests/` (the only `test_paths`); each test's doc
  comment carries `verifies:` naming the LLRs it checks.
- No dependency changes are expected (the proof is org-members', the
  signature ed25519-dalek's). If a crate must be fetched, use
  `CARGO_HOME=/tmp/cargo_home_fuzz`, and stop: a new crate is a SOUP change
  this design did not assess.
- No `sed` on Rust files; use the Edit tool.
- Red evidence: each task records the failing run's output in the commit
  message body of its implementation commit.
- Per-task check: `cargo test -p org-node --features app,test-support --test <the task's targets>`
  and `cargo clippy -p org-node --features app,test-support --lib --test <the task's targets> -- -D warnings` (not `--all-targets`, which is red before S3; see Verification above).

### Task overview

| Task | Title | Parallel |
|---|---|---|
| T0 | Register the new test targets | no (first) |
| T1 | The ten new error variants (LLR-n67aw8) | yes, after T0, with T2 and T3 |
| T2 | `forget_organisation` and the `kept_change_set` field (LLR-pba7yu, LLR-d9778a) | yes, after T0, with T1 and T3 |
| T3 | `revocation.rs`: the notice and `notices_for`, the acknowledgement's signed bytes (LLR-kr5t6f, LLR-gbe9bt) | yes, after T0, with T1 and T2 |
| T4 | The Wire message of three kinds (LLR-js9dsu, LLR-dc45ur, LLR-378cj4) | no (serial, after T1–T3) |
| T5 | Deciding a notice and signing the acknowledgements (LLR-r7zm39, LLR-tx8ruv, LLR-r8qhky, LLR-uw7nmv, LLR-hby4jr) | no (serial, after T4) |
| T6 | Checking a received acknowledgement (LLR-5azhry) | no (serial, after T5) |
| T7 | Every commit keeps its Change set (LLR-d9778a, LLR-mkj4bz) | no (serial, after T6) |
| T8 | Commit returns its notices; `send_update` sends them (LLR-a8z7r5, LLR-6ymd6d, LLR-8hdu9x; PR-qmvj83) | no (serial, after T7) |
| T9 | Receive paths decide revocations and acknowledgements; one removal step (LLR-pt32fx, LLR-38e2kn, LLR-b27jr6, LLR-6p4pj2, LLR-jsx922, LLR-23sfdh) | no (serial, after T8) |
| T10 | `reconcile` (LLR-gr8x3r, LLR-fm38ww; PR-vt244s org-node half) | no (serial, after T9) |
| T11 | Source-scan absences (LLR-xgefn8, LLR-23sfdh) | no (serial, after T10) |
| T12a | The member-sender rule: updates and revocations only from listed Devices, acknowledgements only from their own Device (LLR-2r2fha, LLR-kzgjz8, LLR-3aysup, LLR-5azhry as amended; PR-u4c2vp) | no (serial, after T11) |
| T12 | S3a close-out: PR-qmvj83 resolved, PR-vt244s narrowed, PR-u4c2vp's resolution rewritten, the four residual rulings of 2026-10-07 recorded, verification record | no (last of S3a) |
| T13 | S3b-io: mint the org-io requirements and their risk passages | no (first of S3b-io, after S2 merges) |
| T14 | S3b-io: the publish path and the unknown-outcome write (proposed 1, 2, 4) | no (serial, after T13) |
| T15 | S3b-io: startup reconcile and the unverified state (proposed 3, 10) | yes, after T14, with T16 |
| T16 | S3b-io: revocation delivery, receipt, acknowledgement send and list (proposed 5–9, 11) | yes, after T14, with T15 |
| T17 | S3b-io: close-out — PR-vt244s and PR-924ftr resolved, verification record | no (last) |

T1, T2 and T3 touch disjoint files. Every later S3a task touches
`org-node/src/service.rs` or `org-node/src/revocation.rs`, so they are serial.

## S3a — org-node

### T0 — Register the new test targets

**Files touched:** org-node/Cargo.toml, org-node/.guardrails/config.yaml, org-node/tests/revocation.rs, org-node/tests/reconcile.rs, org-node/tests/commit_workflow_errors.rs
**Parallel:** no (first)

1. In `org-node/Cargo.toml`, after the `commit_paths` `[[test]]` entry, add:

```toml
[[test]]
name = "revocation"
path = "tests/revocation.rs"
required-features = ["app", "test-support"]

[[test]]
name = "reconcile"
path = "tests/reconcile.rs"
required-features = ["app", "test-support"]

[[test]]
name = "commit_workflow_errors"
path = "tests/commit_workflow_errors.rs"
required-features = ["app", "test-support"]
```

2. Create each of the three files with this header only:

```rust
#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Stage S3 (`docs/plans/2026-10-06-org-io-commit-workflow.md`).

mod support;
```

3. Append ` --test revocation --test reconcile --test commit_workflow_errors`
   to the `cargo test -p org-node …` line of `verify_commands` in
   `org-node/.guardrails/config.yaml`, with a dated comment above it:
   `# 2026-10-07: three targets added by S3 (docs/plans/2026-10-06-org-io-commit-workflow.md).`
4. Run `cargo test -p org-node --features app,test-support --test revocation --test reconcile --test commit_workflow_errors`.
   Expected: each target compiles and reports `0 passed; 0 failed`.
5. Commit: `test(org-node): S3 T0 — register the revocation, reconcile and error test targets`.

### T1 — The ten new error variants

**Files touched:** org-node/src/error.rs, org-node/tests/commit_workflow_errors.rs
**Parallel:** yes, after T0, with T2 and T3

1. Red, in `org-node/tests/commit_workflow_errors.rs`:

```rust
use org_members::OrgMembersError;
use org_node::error::OrgNodeError;
use org_node::ids::OrgId;
use org_node::Epoch;

fn all_new(org_id: OrgId) -> Vec<OrgNodeError> {
    vec![
        OrgNodeError::OrgNotHeld { org_id },
        OrgNodeError::StaleChainState { org_id, chain_epoch: Epoch::new(3), record_epoch: Epoch::new(4) },
        OrgNodeError::ChainStateConflict { org_id },
        OrgNodeError::RevocationProofRefused { org_id, cause: OrgMembersError::DeviceStillHeld },
        OrgNodeError::AcknowledgementNotHeld { org_id },
        OrgNodeError::AcknowledgementFromFuture { org_id },
        OrgNodeError::AcknowledgementForListedDevice { org_id },
        OrgNodeError::AcknowledgementSignatureInvalid { org_id },
        OrgNodeError::DeviceSecretNotSupplied { org_id },
        OrgNodeError::NoRevocationForRecipient { org_id },
    ]
}

/// verifies: LLR-n67aw8
///
/// Normal: each variant is distinct from every other and its `Display`
/// names the Organisation.
#[test]
fn the_ten_new_variants_are_distinct_and_name_the_organisation() {
    let org_id = OrgId::new([7; 20]);
    let errors = all_new(org_id);
    for (index, first) in errors.iter().enumerate() {
        for second in errors.iter().skip(index + 1) {
            assert_ne!(first.to_string(), second.to_string());
        }
        assert!(first.to_string().contains(&org_id.to_string()), "{first} names the Organisation");
    }
}

/// verifies: LLR-n67aw8
///
/// Abnormal: the stale-state refusal names both epochs; the proof refusal
/// names its cause.
#[test]
fn stale_state_names_both_epochs_and_the_proof_refusal_names_its_cause() {
    let org_id = OrgId::new([7; 20]);
    let stale = OrgNodeError::StaleChainState { org_id, chain_epoch: Epoch::new(3), record_epoch: Epoch::new(4) };
    let text = stale.to_string();
    assert!(text.contains("epoch 3") && text.contains("4"), "{text}");
    let proof = OrgNodeError::RevocationProofRefused { org_id, cause: OrgMembersError::DeviceStillHeld };
    assert!(proof.to_string().contains(&OrgMembersError::DeviceStillHeld.to_string()));
}
```

   Run `cargo test -p org-node --features app,test-support --test commit_workflow_errors`.
   Expected: does not compile (`no variant named OrgNotHeld`) — the red run.
2. Implement: add the ten variants to `OrgNodeError` in
   `org-node/src/error.rs`, each `#[error(...)]` printing `{org_id}`; for
   example
   `#[error("chain state for {org_id} is at epoch {chain_epoch}, older than the record's epoch {record_epoch}")]`
   and
   `#[error("revocation for {org_id} refused: its absence proof does not verify ({cause})")]`.
3. Green: the same command; expected `2 passed`.
4. Commit: `feat(org-node): S3 T1 — the commit workflow's refusals (LLR-n67aw8)`.

### T2 — `forget_organisation` and the `kept_change_set` field

**Files touched:** org-node/src/store.rs, org-node/src/service.rs, org-node/tests/store_at_rest.rs, org-node/tests/encoding_golden.rs, org-node/docs/architecture/2026-10-04-type-safety.md
**Parallel:** yes, after T0, with T1 and T3

(`service.rs` is touched only to add `kept_change_set: None` to the
`OrgRecord` literals it builds; T1 and T3 do not touch it.)

1. Red, in `org-node/tests/store_at_rest.rs`. Add a fixture at the top of the
   new tests:

```rust
const ORG_1: OrgId = OrgId::new([1; 20]);
const ORG_2: OrgId = OrgId::new([2; 20]);

/// Two Organisations, each with a bound Persona, a record, one ChangeSet
/// provisional update and one expectation; plus an unbound genesis update
/// built by org 1's Persona.
fn two_organisation_store() -> StoreData {
    let persona = |n: u8, org: OrgId| PersonaRecord {
        persona_id: PersonaId::new(format!("p{n}")),
        org_id: Some(org),
        handle: Handle::new(format!("h{n}")).unwrap(),
        name: Name::new("N").unwrap(),
        surname: Surname::new("S").unwrap(),
        member_seed: MemberSeed::from([n; 32]),
        device_seed: DeviceSeed::from([n + 10; 32]),
        member_id: Some(MemberId::new([n; 32])),
        status: PersonaStatus::Active,
    };
    let record = |org: OrgId| OrgRecord {
        org_id: org,
        root_hash: RootHash::new([org.as_bytes()[0]; 32]),
        org_pub_key: org_public_key(),
        epoch: Epoch::new(4),
        last_seq: SequenceNumber::new(4),
        trie_members: vec![],
        proxy_account: None,
        org_private_key: OrgPrivateKey::from([org.as_bytes()[0]; 32]),
        kept_change_set: Some(vec![org.as_bytes()[0]]),
    };
    let update = |org: Option<OrgId>, n: u8| ProvisionalUpdate {
        org_id: org,
        persona_id: PersonaId::new(format!("p{n}")),
        base_root: org.map(|o| RootHash::new([o.as_bytes()[0]; 32])),
        resulting_root: RootHash::new([n + 50; 32]),
        seq: SequenceNumber::new(5),
        org_pub_key: org_public_key(),
        change: ProvisionalChange::ChangeSet { change_set: vec![n], org_private_key: OrgPrivateKey::from([n + 60; 32]) },
    };
    StoreData {
        personas: vec![persona(1, ORG_1), persona(2, ORG_2)],
        orgs: vec![record(ORG_1), record(ORG_2)],
        provisional_updates: vec![update(Some(ORG_1), 1), update(None, 1), update(Some(ORG_2), 2)],
        expected_admissions: vec![ExpectedAdmission { org_id: ORG_1 }, ExpectedAdmission { org_id: ORG_2 }],
    }
}
```

   (Adjust each constructor to the type's actual parse function, e.g.
   `Handle::parse`; check with `grep -n 'pub fn' org-members/src/types.rs`.
   If `StoreData` has fields beyond these four, set them to their empty
   values and assert them unchanged below.) Then:

```rust
/// verifies: LLR-pba7yu
///
/// Normal: forgetting org 1 removes its record, provisional updates (the
/// genesis one its Persona built included), expectation and Persona with its
/// keys; org 2's data is unchanged and in order; nothing names org 1.
#[test]
fn forget_organisation_removes_everything_of_one_organisation_and_nothing_else() {
    let data = two_organisation_store();
    let input_bytes = postcard::to_allocvec(&data).unwrap();
    let after = data.forget_organisation(ORG_1);
    assert_eq!(after.personas, vec![data.personas[1].clone()]);
    assert_eq!(after.orgs, vec![data.orgs[1].clone()]);
    assert_eq!(after.provisional_updates, vec![data.provisional_updates[2].clone()]);
    assert_eq!(after.expected_admissions, vec![ExpectedAdmission { org_id: ORG_2 }]);
    let after_bytes = postcard::to_allocvec(&after).unwrap();
    assert!(!after_bytes.windows(20).any(|window| window == ORG_1.as_bytes()), "no tombstone, marker or copy names org 1");
    assert_eq!(postcard::to_allocvec(&data).unwrap(), input_bytes, "the input is unchanged");
}

/// verifies: LLR-pba7yu
///
/// Abnormal: forgetting an Organisation the store does not hold returns an
/// equal copy.
#[test]
fn forget_organisation_of_an_unheld_organisation_changes_nothing() {
    let data = two_organisation_store();
    assert_eq!(data.forget_organisation(OrgId::new([9; 20])), data);
}

/// verifies: LLR-d9778a
///
/// `kept_change_set` round-trips through the sealed store, `None` and `Some`.
#[test]
fn kept_change_set_round_trips_through_the_store() {
    for kept in [None, Some(vec![1u8, 2, 3])] {
        let mut data = two_organisation_store();
        data.orgs[0].kept_change_set = kept.clone();
        let dir = tempfile::tempdir().unwrap();
        let mut store = PersonaStore::create(dir.path(), "pw").unwrap();
        *store.data_mut() = data;
        store.save(&mut rand::rngs::OsRng).unwrap();
        let reopened = PersonaStore::open(dir.path(), "pw").unwrap();
        assert_eq!(reopened.data().orgs[0].kept_change_set, kept);
    }
}
```

   (Use this file's existing store-opening helpers if `PersonaStore::create`
   and `open` are named differently.) Run
   `cargo test -p org-node --features app,test-support --test store_at_rest`;
   expected: does not compile (`no method forget_organisation`, `no field kept_change_set`).
2. Implement in `org-node/src/store.rs`: `pub kept_change_set: Option<Vec<u8>>`
   as the last field of `OrgRecord` and of `RawOrgRecord`, carried through
   their conversion; every `OrgRecord` literal in `src` sets `None`; and

```rust
impl StoreData {
    /// The store without anything of `org_id` (LLR-pba7yu): its record, its
    /// provisional updates, its expectations, its Personas and the genesis
    /// updates they built. Adds nothing in their place.
    pub fn forget_organisation(&self, org_id: OrgId) -> StoreData {
        let forgotten: Vec<&PersonaId> = self
            .personas
            .iter()
            .filter(|persona| persona.org_id == Some(org_id))
            .map(|persona| &persona.persona_id)
            .collect();
        let keeps_update = |update: &&ProvisionalUpdate| match update.org_id {
            Some(update_org) => update_org != org_id,
            None => !forgotten.contains(&&update.persona_id),
        };
        StoreData {
            personas: self.personas.iter().filter(|persona| persona.org_id != Some(org_id)).cloned().collect(),
            orgs: self.orgs.iter().filter(|record| record.org_id != org_id).cloned().collect(),
            provisional_updates: self.provisional_updates.iter().filter(keeps_update).cloned().collect(),
            expected_admissions: self
                .expected_admissions
                .iter()
                .filter(|expectation| expectation.org_id != org_id)
                .cloned()
                .collect(),
        }
    }
}
```

   The store layout changes (one trailing option). **Owner ruling of
   2026-10-07 (decision 12 below): accepted, with a new golden pin and no
   migration; stores written before S3 do not load.** Follow the precedent
   LLR-ayrdr8's own notes set (the chain-authority and key-pair changes):
   amend LLR-ayrdr8 in place in `org-node/docs/architecture/2026-10-04-type-safety.md`
   with a dated note citing this ruling (the record now ends with the
   `kept_change_set` option after its Organisation private key), and in
   `org-node/tests/encoding_golden.rs` add the new store pin, derived from
   the old pin's bytes plus the trailing option, recording the old value
   beside it as the earlier changes did. Add both files to this task's
   files touched. Write no migration code. (T4 does the same for the
   revocation Wire message pin, which now encodes a notice.)
3. Green: `cargo test -p org-node --features app,test-support --test store_at_rest --test encoding_golden --test commit_paths`; expected all pass.
4. Commit: `feat(org-node): S3 T2 — forget an Organisation; the kept Change set field (LLR-pba7yu, LLR-d9778a)`.

### T3 — `revocation.rs`: the notice, `notices_for`, the acknowledgement's signed bytes

**Files touched:** org-node/src/revocation.rs, org-node/src/lib.rs, org-node/tests/revocation.rs
**Parallel:** yes, after T0, with T1 and T2

1. Red, in `org-node/tests/revocation.rs`. Fixtures in the same file:
   `record_pair_removing_one_device()` builds, with
   `org_node::test_fixtures::{genesis_trie, admin_device, bob_device, member}`
   and org-members' trie operations, a two-Member trie (A, and B with one
   Device), removes B's Device (`delete_member` of B, then `recalculate`),
   and returns the two `OrgRecord`s (snapshots built field by field:
   `MemberSnapshot { id, handle, name, surname, member_key, device_keys }`),
   the committed calculated trie, and B's `(MemberId, DevicePublicKey)`.
   `record_pair_removing_a_member_with_two_devices()` does the same with B
   holding two Devices. `sample_acknowledgement()` fills every field with
   fixed bytes.

```rust
use org_members::Blake3Hasher;
use org_node::revocation::{notices_for, Acknowledgement, ACK_DOMAIN};

/// verifies: LLR-kr5t6f
///
/// Normal: one removed Device → one notice holding the committed record's
/// Organisation, B's pair, and a proof that verifies against the committed
/// root.
#[test]
fn notices_for_names_each_removed_device_with_a_proof_from_the_committed_record() {
    let (previous, committed, committed_trie, (member_id, device)) = record_pair_removing_one_device();
    let notices = notices_for(&previous, &committed, &committed_trie).unwrap();
    assert_eq!(notices.len(), 1);
    let notice = &notices[0];
    assert_eq!((notice.org_id, notice.member_id, notice.device), (committed.org_id, member_id, device));
    notice.proof.verify::<Blake3Hasher>(&committed.root_hash, &notice.member_id, &notice.device).unwrap();
}

/// verifies: LLR-kr5t6f
///
/// Normal: a removed Member's Devices come in slot order; nothing removed →
/// no notice.
#[test]
fn notices_for_orders_by_member_then_slot_and_is_empty_when_nothing_was_removed() {
    let (previous, committed, trie, devices) = record_pair_removing_a_member_with_two_devices();
    let order: Vec<_> = notices_for(&previous, &committed, &trie).unwrap().iter().map(|notice| notice.device).collect();
    assert_eq!(order, devices);
    assert!(notices_for(&committed, &committed, &trie).unwrap().is_empty());
}

/// verifies: LLR-kr5t6f
///
/// Abnormal: an uncalculated trie is refused with the trie's error.
#[test]
fn notices_for_refuses_an_uncalculated_trie() {
    let (previous, committed, _, (member_id, _)) = record_pair_removing_one_device();
    let uncalculated = org_node::test_fixtures::genesis_trie(&org_node::keys::X25519Keypair::from_seed([1; 32]), &org_node::test_fixtures::admin_device())
        .delete_member(&member_id)
        .unwrap_or_else(|_| panic!("fixture"));
    assert!(matches!(
        notices_for(&previous, &committed, &uncalculated),
        Err(org_node::OrgNodeError::Trie(org_members::OrgMembersError::HashesNotCalculated))
    ));
}

/// verifies: LLR-gbe9bt
///
/// The signed bytes are the domain then the postcard tuple; the signature
/// is not part of them.
#[test]
fn acknowledgement_signed_bytes_are_the_domain_then_the_postcard_tuple() {
    let ack = sample_acknowledgement();
    let mut expected = ACK_DOMAIN.to_vec();
    expected.extend(postcard::to_allocvec(&(ack.org_id, ack.member_id, ack.device, ack.epoch, ack.root)).unwrap());
    assert_eq!(ack.signed_bytes(), expected);
    let other = Acknowledgement { signature: [0xAA; 64], ..ack.clone() };
    assert_eq!(other.signed_bytes(), expected);
    assert_eq!(ACK_DOMAIN, b"ods/org-node/revocation-acknowledgement/v1");
}
```

   (The uncalculated-trie fixture must be any trie on which `delete_member`
   has run without `recalculate`; adapt the constructor call to the
   fixture's actual signature.) Run
   `cargo test -p org-node --features app,test-support --test revocation`;
   expected: does not compile.
2. Implement `org-node/src/revocation.rs`, declared in `lib.rs` as
   `#[cfg(feature = "app")] pub mod revocation;` beside `store`:

```rust
//! SDD-uck4tz: the revocation a revoked Device receives, the decision it acts
//! on, and its acknowledgement. Pure functions over values (LLR-xgefn8).
use org_members::{AbsenceProof, DevicePublicKey, MemberId, RootHash};
use serde::{Deserialize, Serialize};

use crate::error::OrgNodeError;
use crate::ids::OrgId;
use crate::store::OrgRecord;
use crate::types::Epoch;

/// The signing domain of an acknowledgement (LLR-gbe9bt).
pub const ACK_DOMAIN: &[u8] = b"ods/org-node/revocation-acknowledgement/v1";

/// What a revoked Device receives (REQ-ps2gy2): its identity and an absence
/// proof from the committed record, nothing else.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevocationNotice {
    pub org_id: OrgId,
    pub member_id: MemberId,
    pub device: DevicePublicKey,
    pub proof: AbsenceProof,
}

/// A revoked Device's signed statement that it acted on its revocation
/// (REQ-y99c9w). Not evidence of erasure (RC-eydn8t).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Acknowledgement {
    pub org_id: OrgId,
    pub member_id: MemberId,
    pub device: DevicePublicKey,
    pub epoch: Epoch,
    pub root: RootHash,
    pub signature: Signature64,
}

impl Acknowledgement {
    /// LLR-gbe9bt.
    pub fn signed_bytes(&self) -> Vec<u8> {
        let mut bytes = ACK_DOMAIN.to_vec();
        if let Ok(tuple) = postcard::to_allocvec(&(self.org_id, self.member_id, self.device, self.epoch, self.root)) {
            bytes.extend(tuple);
        }
        bytes
    }
}

/// LLR-kr5t6f.
pub fn notices_for(
    previous: &OrgRecord,
    committed: &OrgRecord,
    committed_trie: &crate::Trie,
) -> Result<Vec<RevocationNotice>, OrgNodeError> {
    let still_listed =
        |device: &DevicePublicKey| committed.trie_members.iter().any(|member| member.device_keys.contains(device));
    let mut notices = Vec::new();
    for member in &previous.trie_members {
        for device in member.device_keys.iter().filter(|device| !still_listed(device)) {
            let proof = committed_trie.prove_absent(&member.id, device).map_err(OrgNodeError::Trie)?;
            notices.push(RevocationNotice { org_id: committed.org_id, member_id: member.id, device: *device, proof });
        }
    }
    Ok(notices)
}
```

   `Signature64` is a newtype over `[u8; 64]` defined in this file whose
   serde form is a byte sequence that refuses any length but 64 on decode
   (serde does not derive `[u8; 64]`); tests then write
   `signature: Signature64([0xAA; 64])`. Check that `org-members` exports
   `AbsenceProof` with its `serde` feature on in org-node's dependency line
   (`grep -n org-members org-node/Cargo.toml`). If `Trie` is not re-exported
   at the crate root, import it from where `verify.rs` does.
3. Green: `cargo test -p org-node --features app,test-support --test revocation`; expected `4 passed`.
4. Commit: `feat(org-node): S3 T3 — revocation notices and the acknowledgement's signed bytes (LLR-kr5t6f, LLR-gbe9bt)`.

### T4 — The Wire message of three kinds

**Files touched:** org-node/src/transport/wire.rs, org-node/src/service.rs, org-node/tests/wire_frame_bound.rs, org-node/tests/fuzz_wire_decode/fuzz_target.rs, org-node/tests/support/mod.rs, org-node/tests/service_stories.rs, org-node/tests/organisation_key.rs, org-node/tests/admission_sender.rs, org-node/tests/encoding_golden.rs, org-node/docs/architecture/2026-10-04-type-safety.md
**Parallel:** no (serial, after T1–T3)

1. Red, in `org-node/tests/wire_frame_bound.rs` (fixtures `org_information()`,
   `sample_notice()` — built with T3's fixture — and `sample_acknowledgement()`
   all for `const ORG: OrgId = OrgId::new([3; 20])`):

```rust
/// verifies: LLR-js9dsu, LLR-dc45ur, LLR-378cj4
///
/// Normal: each kind round-trips with variant index 0, 1 or 2, and
/// `org_id()` names its Organisation.
#[test]
fn the_three_kinds_round_trip_with_their_indices() {
    let kinds = [
        (org_information(), 0u8),
        (WireMessage::Revocation(sample_notice()), 1),
        (WireMessage::Acknowledgement(sample_acknowledgement()), 2),
    ];
    for (message, index) in kinds {
        let frame = encode_frame(&message).unwrap();
        assert_eq!(frame[4], index);
        assert_eq!(decode_body(&frame[4..]).unwrap(), message);
        assert_eq!(message.org_id(), ORG);
    }
}

/// verifies: LLR-js9dsu, LLR-dc45ur, LLR-378cj4
///
/// Abnormal: index 3, trailing bytes, a device that is not an Ed25519 point,
/// a proof over 256 siblings, or a signature of 63 bytes is `Malformed`,
/// without a panic.
#[test]
fn malformed_revocations_and_acknowledgements_are_refused() {
    let body = |message: &WireMessage| encode_frame(message).unwrap()[4..].to_vec();
    let mut bad_index = body(&WireMessage::Revocation(sample_notice()));
    bad_index[0] = 3;
    let mut trailing = body(&WireMessage::Acknowledgement(sample_acknowledgement()));
    trailing.push(0);
    let not_a_point = [0xFFu8; 32];
    let mut notice_bad_device = vec![1u8];
    notice_bad_device.extend(postcard::to_allocvec(&(ORG, sample_notice().member_id, not_a_point)).unwrap());
    notice_bad_device.extend(postcard::to_allocvec(&sample_notice().proof).unwrap());
    let mut ack_short_signature = vec![2u8];
    let ack = sample_acknowledgement();
    ack_short_signature.extend(postcard::to_allocvec(&(ack.org_id, ack.member_id, ack.device, ack.epoch, ack.root)).unwrap());
    ack_short_signature.extend(postcard::to_allocvec(&vec![0u8; 63]).unwrap());
    for refused in [bad_index, trailing, notice_bad_device, notice_with_siblings(257), ack_short_signature] {
        assert!(matches!(decode_body(&refused), Err(TransportError::Malformed)));
    }
}
```

   `notice_with_siblings(n)` encodes index 1, the notice's three identity
   fields and an org-members wire proof with `n` siblings (build the raw
   proof with postcard in the field order of `org_members::proof::wire::RawAbsenceProof`).
   Run `cargo test -p org-node --features app,test-support --test wire_frame_bound`; expected: does not compile.
2. Implement in `wire.rs`: `Revocation(RevocationNotice)` (index 1),
   `Acknowledgement(Acknowledgement)` (index 2), `pub fn org_id(&self) -> OrgId`;
   delete `envelope()`. `decode_body` uses `postcard::take_from_bytes` and
   refuses a non-empty remainder with `Malformed`. In `service.rs` replace
   `msg.envelope().org_id` with `msg.org_id()`; for this task only, both
   receive paths refuse `Revocation(_)` and `Acknowledgement(_)` with
   `MalformedMessage` (T9 replaces that) and `send_update`'s revocation arm
   refuses with `NoRevocationForRecipient { org_id }` (T8 replaces that).
   Update `tests/support/mod.rs`'s `with_envelope`, `with_snapshot` and
   `with_key` to the new variants (a `Revocation` has no Envelope:
   `with_envelope` panics on it). Keep `fuzz_wire_decode`'s property
   (decode never panics; a decoded body re-encodes to itself) over arbitrary
   bytes, now reaching indices 1 and 2.
   The revocation Wire message pin in `encoding_golden.rs` changes (index
   01 now precedes a notice, not an Envelope): amend LLR-ayrdr8 in place
   with a dated note, add the new pin derived from the notice's field order,
   and record the old value beside it, as in T2.
3. Tests that assert the old `Revocation { envelope }` behaviour now fail by
   design; mark each `#[ignore = "S3 T8"]` or `#[ignore = "S3 T9"]`
   (send side or receive side). T8 and T9 rewrite them; after T9,
   `grep -rn '"S3 T' org-node/tests` prints nothing.
4. Green: `cargo test -p org-node --features app,test-support --test wire_frame_bound --test fuzz_wire_decode --test service_stories --test organisation_key --test admission_sender`.
5. Commit: `feat(org-node): S3 T4 — the Wire message carries a notice or an acknowledgement (LLR-js9dsu, LLR-dc45ur, LLR-378cj4)`.

### T5 — Deciding a notice and signing the acknowledgements

**Files touched:** org-node/src/revocation.rs, org-node/tests/revocation.rs
**Parallel:** no (serial, after T4)

1. Red, in `org-node/tests/revocation.rs`. Fixture: `bound_store()` is a
   `StoreData` with one Persona bound to `ORG`, `member_id = Some(M)`, device
   seed `bound_device_seed()` (public key `D`), and a record of `ORG` at
   epoch 4 listing `M` with `D`; `removed_root()` is the root of that trie
   with `D` removed (calculated), `still_listing_root()` the record's own;
   `genuine_notice()` holds `(ORG, M, D, proof)` with the proof from the
   removed trie; `chain_at(epoch, root)` an `OrgState` with the record's key.

```rust
use org_node::revocation::{accept, check_notice};

/// verifies: LLR-r7zm39
///
/// Abnormal, in order: an unheld Organisation, then another Device; normal:
/// the bound Persona is returned.
#[test]
fn check_notice_refuses_an_unheld_organisation_then_another_device() {
    let store = bound_store();
    let unheld = RevocationNotice { org_id: OrgId::new([9; 20]), ..genuine_notice() };
    assert!(matches!(check_notice(&store, &unheld), Err(OrgNodeError::RevocationNotHeld { .. })));
    let other = RevocationNotice { device: device_key(77), ..genuine_notice() };
    assert!(matches!(check_notice(&store, &other), Err(OrgNodeError::RevocationNotForThisDevice { .. })));
    assert_eq!(check_notice(&store, &genuine_notice()).unwrap().member_id, Some(M));
}

/// verifies: LLR-tx8ruv, LLR-uw7nmv
///
/// Abnormal, in order: no chain state; a state older than the record; a
/// proof against another root; a Device the chain's record still lists.
/// Each returns the error alone; the store is unchanged.
#[test]
fn accept_refuses_in_order_and_changes_nothing() {
    let store = bound_store();
    let before = store.clone();
    let seeds = || vec![bound_device_seed()];
    assert!(matches!(accept(&store, &genuine_notice(), None, seeds()), Err(OrgNodeError::OrgNotOnChain)));
    assert!(matches!(
        accept(&store, &genuine_notice(), Some(chain_at(3, removed_root())), seeds()),
        Err(OrgNodeError::StaleChainState { .. })
    ));
    assert!(matches!(
        accept(&store, &genuine_notice(), Some(chain_at(5, RootHash::new([5; 32]))), seeds()),
        Err(OrgNodeError::RevocationProofRefused { .. })
    ));
    assert!(matches!(
        accept(&store, &genuine_notice(), Some(chain_at(5, still_listing_root())), seeds()),
        Err(OrgNodeError::RevocationProofRefused { .. })
    ));
    assert_eq!(store, before);
}

/// verifies: LLR-tx8ruv, LLR-r8qhky, LLR-hby4jr
///
/// Normal: at an equal and a newer epoch the notice is accepted; the
/// successor is `forget_organisation`'s, and one acknowledgement per bound
/// Persona names the chain's epoch and root and verifies under D.
#[test]
fn accept_signs_then_forgets() {
    let store = bound_store();
    for epoch in [4, 9] {
        let accepted = accept(&store, &genuine_notice(), Some(chain_at(epoch, removed_root())), vec![bound_device_seed()]).unwrap();
        assert_eq!(accepted.store, store.forget_organisation(ORG));
        assert_eq!(accepted.acknowledgements.len(), 1);
        let ack = &accepted.acknowledgements[0];
        assert_eq!((ack.org_id, ack.member_id, ack.device, ack.epoch, ack.root), (ORG, M, D, Epoch::new(epoch), removed_root()));
        let key = ed25519_dalek::VerifyingKey::from_bytes(D.as_bytes()).unwrap();
        key.verify_strict(&ack.signed_bytes(), &ed25519_dalek::Signature::from_bytes(&ack.signature.0)).unwrap();
    }
}

/// verifies: LLR-hby4jr, LLR-uw7nmv
///
/// Abnormal: no seed for the bound Persona (none, or only another Device's)
/// → `DeviceSecretNotSupplied`, nothing produced.
#[test]
fn a_missing_device_seed_is_refused_before_anything_is_deleted() {
    let store = bound_store();
    for seeds in [vec![], vec![DeviceSeed::from([99; 32])]] {
        assert!(matches!(
            accept(&store, &genuine_notice(), Some(chain_at(5, removed_root())), seeds),
            Err(OrgNodeError::DeviceSecretNotSupplied { .. })
        ));
    }
}
```

   Run `cargo test -p org-node --features app,test-support --test revocation`; expected: does not compile.
2. Implement in `revocation.rs`: `Accepted { pub store: StoreData, pub acknowledgements: Vec<Acknowledgement> }`,
   `check_notice`, `accept` and `sign_acknowledgements` as LLR-r7zm39,
   LLR-tx8ruv, LLR-r8qhky and LLR-hby4jr state. The proof is checked with
   `notice.proof.verify::<H>(&chain.root_hash, &notice.member_id, &notice.device)`,
   `H` the hasher `trie_from_snapshots` uses. A Persona's DevicePublicKey is
   computed by one private helper `persona_device(&PersonaRecord) -> Option<DevicePublicKey>`
   (today `persona.device_seed.signing_keypair().device_key().ok()`; S4
   replaces its body with the stored public key, the only line that changes).
   Signing: `seed.signing_keypair()` and `ed25519_dalek::Signer::sign` over
   `signed_bytes()`. `device_seeds` is taken by value and never cloned.
3. Green: same command; T3's and T5's tests pass.
4. Commit: `feat(org-node): S3 T5 — decide a revocation notice and sign the acknowledgements (LLR-r7zm39, LLR-tx8ruv, LLR-r8qhky, LLR-uw7nmv, LLR-hby4jr)`.

### T6 — Checking a received acknowledgement

**Files touched:** org-node/src/revocation.rs, org-node/tests/revocation.rs
**Parallel:** no (serial, after T5)

1. Red. Fixture: `admin_store(listing_d: bool)` is a store holding a record
   of `ORG` at epoch 5 that lists `D` or not; `signed_ack(seed, epoch)`
   signs `(ORG, M, public key of seed, epoch, removed_root())` with
   `seed.signing_keypair()`.

```rust
use org_node::revocation::check_acknowledgement;

/// verifies: LLR-5azhry
///
/// Normal: a genuine acknowledgement of a Device the record no longer lists,
/// at an epoch not above the record's, verifies; the store is untouched.
#[test]
fn a_genuine_acknowledgement_verifies() {
    let store = admin_store(false);
    let before = store.clone();
    let ack = signed_ack(&bound_device_seed(), Epoch::new(5));
    assert_eq!(check_acknowledgement(&store, ack.clone()).unwrap().into_inner(), ack);
    assert_eq!(store, before);
}

/// verifies: LLR-5azhry
///
/// Abnormal, cheapest first: unheld; from the future; for a listed Device;
/// a flipped signature byte; another key's signature.
#[test]
fn acknowledgements_are_refused_in_order() {
    let store = admin_store(false);
    let ack = signed_ack(&bound_device_seed(), Epoch::new(5));
    let unheld = Acknowledgement { org_id: OrgId::new([9; 20]), ..ack.clone() };
    assert!(matches!(check_acknowledgement(&store, unheld), Err(OrgNodeError::AcknowledgementNotHeld { .. })));
    let future = signed_ack(&bound_device_seed(), Epoch::new(6));
    assert!(matches!(check_acknowledgement(&store, future), Err(OrgNodeError::AcknowledgementFromFuture { .. })));
    assert!(matches!(check_acknowledgement(&admin_store(true), ack.clone()), Err(OrgNodeError::AcknowledgementForListedDevice { .. })));
    let mut flipped = ack.clone();
    flipped.signature.0[0] ^= 1;
    assert!(matches!(check_acknowledgement(&store, flipped), Err(OrgNodeError::AcknowledgementSignatureInvalid { .. })));
    let foreign = Acknowledgement { signature: signed_ack(&DeviceSeed::from([99; 32]), Epoch::new(5)).signature, ..ack };
    assert!(matches!(check_acknowledgement(&store, foreign), Err(OrgNodeError::AcknowledgementSignatureInvalid { .. })));
}
```

2. Implement `check_acknowledgement` and `VerifiedAcknowledgement` (private
   field; `into_inner(self)`; constructed only here), verifying with
   `VerifyingKey::from_bytes(ack.device.as_bytes())` and `verify_strict`.
3. Green: `cargo test -p org-node --features app,test-support --test revocation`.
4. Commit: `feat(org-node): S3 T6 — check a received acknowledgement (LLR-5azhry)`.

### T7 — Every commit keeps its Change set

**Files touched:** org-node/src/service.rs, org-node/tests/commit_paths.rs
**Parallel:** no (serial, after T6)

1. Red, in `org-node/tests/commit_paths.rs`:

```rust
/// verifies: LLR-d9778a, LLR-mkj4bz
///
/// Normal: genesis and a first admission keep none; each later commit keeps
/// exactly the committed Envelope's Change set, replacing the earlier one,
/// on disk too.
#[tokio::test]
async fn every_commit_keeps_the_change_set_that_produced_the_record() {
    let s = setup("kept-cs").await;
    assert_eq!(rec_of(&s.svc_a, s.org_id).kept_change_set, None, "genesis keeps none");
    let mut s = admit_b_directly(s).await;
    let first = rec_of(&s.svc_a, s.org_id).kept_change_set.expect("A keeps its admission's Change set");
    assert_eq!(rec_of(&s.svc_b, s.org_id).kept_change_set, None, "a first admission keeps none");
    assert_eq!(disk_rec_of(&reopen_store("kept-cs", "a", "pw_a"), s.org_id).kept_change_set, Some(first.clone()));
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let update = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_c).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    let outcome = s.svc_a.commit_update(&mut OsRng, s.org_id).await.unwrap();
    let kept = rec_of(&s.svc_a, s.org_id).kept_change_set.unwrap();
    assert_eq!(kept, outcome.outgoing.envelope.delta_bytes);
    assert_ne!(kept, first, "the earlier one is replaced");
}

/// verifies: LLR-mkj4bz
///
/// Abnormal: a refused commit keeps the earlier Change set.
#[tokio::test]
async fn a_refused_commit_keeps_the_earlier_change_set() {
    let mut s = admit_b_directly(setup("kept-cs-refused").await).await;
    let before = rec_of(&s.svc_a, s.org_id).kept_change_set;
    assert!(s.svc_a.commit_update(&mut OsRng, s.org_id).await.is_err(), "nothing new on the chain");
    assert_eq!(rec_of(&s.svc_a, s.org_id).kept_change_set, before);
}
```

   Also, in `service_stories.rs` or this file, a received update branch: B
   receives A's admission of C and keeps that Envelope's Change set (extend
   the existing three-party story that delivers C's admission to B with one
   assertion, `verifies: LLR-d9778a`).
2. Implement: `commit_held` takes the verified Envelope's `delta_bytes` and
   sets `kept_change_set = Some(...)`; the update branches of the two receive
   paths pass the received Envelope's; first admission and `commit_genesis`
   leave `None`.
3. Green: `cargo test -p org-node --features app,test-support --test commit_paths --test service_stories`.
4. Commit: `feat(org-node): S3 T7 — every commit keeps the Change set that produced the record (LLR-d9778a, LLR-mkj4bz)`.

### T8 — Commit returns its notices; `send_update` sends them

**Files touched:** org-node/src/service.rs, org-node/tests/support/mod.rs, org-node/tests/service_stories.rs, org-node/tests/organisation_key.rs, app/src-tauri/src/submit.rs, app/src-tauri/tests/submit_flow.rs, app/docs/architecture/2026-10-07-org-key-pair.md
**Parallel:** no (serial, after T7)

1. Red — PR-qmvj83's reproduction first, in `org-node/tests/service_stories.rs`:

```rust
/// verifies: LLR-8hdu9x, LLR-6ymd6d, LLR-a8z7r5
///
/// PR-qmvj83: revoking B in a two-Member Organisation sends B's Device a
/// revocation holding only B's identity and an absence proof that verifies
/// against the chain's root — no Envelope, Change set, snapshot, key, or any
/// leaf of A.
#[tokio::test]
async fn a_revoked_device_receives_only_its_notice() {
    let mut s = admit_b_directly(setup("pr-qmvj83").await).await;
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let (sink_addr, sink) = spawn_recv_one(*s.b_device_kp.device_seed().expose_secret()).await;
    revoke(&mut s.svc_a, &s.chain, s.org_id, b_id, Some(sink_addr)).await.unwrap();
    let (_endpoint, _sender, message) = sink.await.unwrap();
    let WireMessage::Revocation(notice) = message else { panic!("a revocation") };
    let b_device = s.b_device_kp.device_key().unwrap();
    assert_eq!((notice.org_id, notice.member_id, notice.device), (s.org_id, b_id, b_device));
    let root = s.chain.get(&s.org_id).unwrap().root_hash;
    notice.proof.verify::<org_members::Blake3Hasher>(&root, &b_id, &b_device).unwrap();
    let body = postcard::to_allocvec(&notice).unwrap();
    let a_leaf = &rec_of(&s.svc_a, s.org_id).trie_members[0];
    assert!(!body.windows(32).any(|window| window == a_leaf.member_key.as_bytes()), "no leaf of A");
}

/// verifies: LLR-6ymd6d, LLR-a8z7r5
///
/// An admission's outcome holds no notice; a revocation's holds exactly the
/// removed pair; a Device neither record lists is sent nothing.
#[tokio::test]
async fn send_update_chooses_by_record_and_outcome() {
    let mut s = admit_b_directly(setup("send-choice").await).await;
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let update = s.svc_a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    let outcome = s.svc_a.commit_update(&mut OsRng, s.org_id).await.unwrap();
    let b_device = s.b_device_kp.device_key().unwrap();
    assert_eq!(outcome.revocations.iter().map(|n| (n.member_id, n.device)).collect::<Vec<_>>(), vec![(b_id, b_device)]);
    let stranger = device_key(42);
    let refused = s.svc_a.send_update(&outcome, stranger, Some(dead_addr([42; 32]))).await;
    assert!(matches!(refused, Err(OrgNodeError::NoRevocationForRecipient { org_id }) if org_id == s.org_id));
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let admission = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_c).unwrap();
    s.chain.apply_update(s.org_id, admission.resulting_root, admission.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    assert!(s.svc_a.commit_update(&mut OsRng, s.org_id).await.unwrap().revocations.is_empty());
}
```

   Change `tests/support/mod.rs`'s `admit` and `revoke_and_send` to call
   `send_update(&outcome, …)`, and rewrite each test T4 marked
   `#[ignore = "S3 T8"]` (the send side: a listed Device gets
   `OrgInformation`, the removed one `Revocation(notice)` equal to
   `outcome.revocations[0]`), removing the marks. Run
   `cargo test -p org-node --features app,test-support --test service_stories`; expected: does not compile.
2. Implement: `CommitOutcome.revocations`, filled in `commit_update` with
   `revocation::notices_for(&record_before, &record_after, &verified.trie)`
   and empty on a received commit; `send_update(outcome: &CommitOutcome,
   recipient, peer_addr)` per LLR-6ymd6d as amended.
3. App: in `app/src-tauri/src/submit.rs`, `submit_commit_send` passes
   `&outcome` (one line); adjust `app/src-tauri/tests/submit_flow.rs` where
   it builds a `CommitOutcome` literal (add `revocations: vec![]`). Add a
   dated note to `app/docs/architecture/2026-10-07-org-key-pair.md` at the
   paragraph naming `send_update`: "*Amended 2026-10-07 (S3): `send_update`
   takes the `CommitOutcome`; a revoked Device is sent its notice, and a
   Device neither record lists nothing.*"
4. Green: `cargo test -p org-node --features app,test-support --test service_stories --test organisation_key --test admission_sender` and the app's `verify_commands`.
5. Commit: `fix(org-node): S3 T8 — a revoked Device receives only its notice (PR-qmvj83; LLR-a8z7r5, LLR-6ymd6d, LLR-8hdu9x)`.

### T9 — Receive paths decide revocations and acknowledgements; one removal step

**Files touched:** org-node/src/service.rs, org-node/tests/support/mod.rs, org-node/tests/service_stories.rs, org-node/tests/receive_chain_reads.rs, org-node/tests/admission_sender.rs, app/src-tauri/src/commands.rs, app/src-tauri/src/events.rs, app/src-tauri/tests/receiver_events.rs, app/docs/architecture/2026-10-05-decomposition.md
**Parallel:** no (serial, after T8)

1. Red, in `org-node/tests/service_stories.rs` (story 5 rewritten):

```rust
/// verifies: LLR-pt32fx, LLR-6p4pj2, LLR-23sfdh
///
/// Story 5: B receives its notice, accepts it against the chain, signs one
/// acknowledgement, and forgets the Organisation and its Persona, on disk.
#[tokio::test]
async fn a_revoked_node_acknowledges_then_forgets_everything() {
    let mut s = admit_b_directly(setup("story5-s3").await).await;
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let (b_addr, b_task) = spawn_self_delete(s.svc_b, &s.b_device_kp).await;
    revoke(&mut s.svc_a, &s.chain, s.org_id, b_id, Some(b_addr)).await.unwrap();
    let (svc_b, outcome) = b_task.await.unwrap();
    let SelfDeleteOutcome::SelfDeleted { org_id, acknowledgements } = outcome.unwrap() else { panic!("self-deleted") };
    assert_eq!((org_id, acknowledgements.len()), (s.org_id, 1));
    assert_eq!(acknowledgements[0].member_id, b_id);
    assert!(svc_b.list_orgs().is_empty() && svc_b.list_personas().is_empty());
    let disk = reopen_store("story5-s3", "b", "pw_b");
    assert!(disk.data().orgs.is_empty() && disk.data().personas.is_empty());
}
```

   and in `org-node/tests/receive_chain_reads.rs` (it already counts
   `read_state` calls through a wrapping `ChainOps`):

```rust
/// verifies: LLR-38e2kn, LLR-pt32fx
///
/// Abnormal: a notice about an unheld Organisation is refused with
/// `RevocationNotHeld` on both paths, with no chain read; a notice for B
/// whose proof does not verify against the chain (B still listed) is refused
/// with `RevocationProofRefused` after one read; nothing is written.
#[tokio::test]
async fn revocations_are_refused_without_writing() {
    let s = admit_b_directly(setup_counting("refuse-notice").await).await;
    let b_id = id_by_handle(&rec_of(&s.svc_b, s.org_id), "bob");
    let b_device = s.b_device_kp.device_key().unwrap();
    let proof_from_elsewhere = trie_without(b_id, b_device).prove_absent(&b_id, &b_device).unwrap();
    let unheld = RevocationNotice { org_id: OrgId::new([9; 20]), member_id: b_id, device: b_device, proof: proof_from_elsewhere.clone() };
    let still_listed = RevocationNotice { org_id: s.org_id, member_id: b_id, device: b_device, proof: proof_from_elsewhere };
    let before = store_bytes("refuse-notice", "b");
    for (notice, expected_reads) in [(unheld, 0), (still_listed, 1)] {
        let reads_before = s.reads();
        let result = deliver_to_self_delete(&mut s.svc_b, &s.b_device_kp, WireMessage::Revocation(notice.clone())).await;
        assert!(matches!(result, Err(OrgNodeError::RevocationNotHeld { .. }) | Err(OrgNodeError::RevocationProofRefused { .. })));
        assert_eq!(s.reads() - reads_before, expected_reads);
        let result = deliver_to_receive(&mut s.svc_b, &s.b_device_kp, WireMessage::Revocation(notice)).await;
        assert!(result.is_err());
    }
    assert_eq!(store_bytes("refuse-notice", "b"), before);
}

/// verifies: LLR-pt32fx, LLR-5azhry
///
/// A's receive path reports B's acknowledgement as `Acknowledged`, with no
/// chain read and no write.
#[tokio::test]
async fn an_acknowledgement_is_checked_and_reported() {
    let mut s = admit_b_directly(setup_counting("ack-report").await).await;
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let (b_addr, b_task) = spawn_self_delete(s.svc_b, &s.b_device_kp).await;
    revoke(&mut s.svc_a, &s.chain, s.org_id, b_id, Some(b_addr)).await.unwrap();
    let (_svc_b, outcome) = b_task.await.unwrap();
    let SelfDeleteOutcome::SelfDeleted { acknowledgements, .. } = outcome.unwrap() else { panic!() };
    let before = store_bytes("ack-report", "a");
    let reads_before = s.reads();
    let result = deliver_to_self_delete(&mut s.svc_a, &device_kp(&s.svc_a, &s.pid_a), WireMessage::Acknowledgement(acknowledgements[0].clone())).await;
    let SelfDeleteOutcome::Acknowledged(verified) = result.unwrap() else { panic!("acknowledged") };
    assert_eq!(verified.into_inner(), acknowledgements[0]);
    assert_eq!((s.reads() - reads_before, store_bytes("ack-report", "a")), (0, before));
}
```

   `setup_counting`, `reads()`, `deliver_to_self_delete` and
   `deliver_to_receive` go in `tests/support/mod.rs`: `setup_counting` is
   `setup_over` with a `ChainOps` wrapper that counts `read_state` (move this
   file's existing counter there if it has one); the two `deliver_to_*` bind
   the service's endpoint, send the message from a fresh endpoint with
   `OrgEndpoint::send`, and await the receive call under `NET`.
   `trie_without(member, device)` builds a calculated trie lacking that pair.
   A fourth test, `an_org_information_commit_that_removes_the_node_forgets_through_the_one_step`
   (`verifies: LLR-b27jr6, LLR-jsx922, LLR-23sfdh`): capture A's
   Organisation-information message for C's admission sent after B's
   revocation, relay it to B's `receive_and_verify` (B still holding the
   pre-revocation record), and assert `ReceiveOutcome.acknowledgements` has
   one entry and B's store holds no record and no Persona; and that the same
   kind of message that keeps B listed commits as an update with no
   acknowledgements. Rewrite each test T4 marked `#[ignore = "S3 T9"]`,
   removing the marks. Run
   `cargo test -p org-node --features app,test-support --test service_stories --test receive_chain_reads`; expected: does not compile.
2. Implement in `service.rs`: the `Revocation(notice)` and
   `Acknowledgement(ack)` arms per LLR-pt32fx and LLR-38e2kn as amended. The
   device seeds passed to `revocation::accept` are clones of the
   `device_seed`s of the Personas bound to that Organisation, taken at the
   call — the one interim place a seed is read from the store, commented
   `// S4: org-io moves the seed in from the OS keychain (REQ-y99c9w)`. One
   private `remove_self(&mut self, org_id, chain: &OrgState) -> Result<Vec<Acknowledgement>, OrgNodeError>`
   signs (`revocation::sign_acknowledgements`) and replaces the store data
   with `forget_organisation`; every commit path's removal calls it. Delete
   `OrgService::forget_organisation`. Outcomes:
   `SelfDeleteOutcome::{SelfDeleted { org_id, acknowledgements }, UpdatedNotRevoked { org_id }, Acknowledged(VerifiedAcknowledgement)}`;
   `ReceiveOutcome { org_id, epoch, root, acknowledgements, acknowledged }`.
3. App: `commands.rs`'s receiver loop maps `SelfDeleted { org_id, .. }` to
   `ReceiverOutcome::SelfDeleted { org_id }` as now — the acknowledgements
   are dropped until S3b-io sends them, which leaves the entry
   unacknowledged, an outcome ruling C already allows — and maps
   `Acknowledged(_)` to a new `ReceiverOutcome::AcknowledgementReceived { org_id }`
   whose `emissions_for` is empty. Add to `app/src-tauri/tests/receiver_events.rs`:

```rust
/// verifies: LLR-2vg79y
///
/// A received acknowledgement announces nothing to the UI (S3: org-io keeps
/// it, from S3b-io).
#[test]
fn a_received_acknowledgement_emits_nothing() {
    assert!(events::emissions_for(&ReceiverOutcome::AcknowledgementReceived { org_id: org() }).is_empty());
}
```

   and amend LLR-2vg79y in `app/docs/architecture/2026-10-05-decomposition.md`
   in place with a dated note adding the new outcome (confirm with
   `GR_CONFIG=app/.guardrails/config.yaml .guardrails/scripts/find-items.sh show LLR-2vg79y`
   that it is the item listing the receiver's emissions; if another item
   lists the outcomes, amend that one and annotate the test with it).
4. Green: org-node's full `verify_commands` cargo line and the app's.
5. Commit: `feat(org-node): S3 T9 — receive paths decide revocations by proof and report acknowledgements (LLR-pt32fx, LLR-38e2kn, LLR-b27jr6, LLR-6p4pj2, LLR-jsx922, LLR-23sfdh)`.

### T10 — `reconcile`

**Files touched:** org-node/src/reconcile.rs, org-node/src/lib.rs, org-node/src/service.rs, org-node/tests/reconcile.rs
**Parallel:** no (serial, after T9)

1. Red, in `org-node/tests/reconcile.rs`:

```rust
use org_node::reconcile::{reconcile, Reconciled};

/// verifies: LLR-gr8x3r
///
/// In step and behind, and the three refusals, each with no successor.
#[tokio::test]
async fn reconcile_reports_one_outcome_from_the_record_and_the_state() {
    let s = admit_b_directly(setup("reconcile").await).await;
    let store = s.svc_a.store_data().clone();
    let record = rec_of(&s.svc_a, s.org_id);
    let at = |epoch: u64, root: RootHash| OrgState { root_hash: root, org_pub_key: record.org_pub_key, epoch: Epoch::new(epoch) };
    let epoch = record.epoch.as_u64();
    let other_root = RootHash::new([5; 32]);
    assert!(matches!(reconcile(&store, s.org_id, at(epoch, record.root_hash), vec![]), Ok(Reconciled::InStep)));
    assert!(matches!(reconcile(&store, s.org_id, at(epoch + 1, other_root), vec![]), Ok(Reconciled::Behind { .. })));
    assert!(matches!(reconcile(&store, s.org_id, at(epoch - 1, record.root_hash), vec![]), Err(OrgNodeError::StaleChainState { .. })));
    assert!(matches!(reconcile(&store, s.org_id, at(epoch, other_root), vec![]), Err(OrgNodeError::ChainStateConflict { .. })));
    assert!(matches!(reconcile(&store, OrgId::new([9; 20]), at(epoch, record.root_hash), vec![]), Err(OrgNodeError::OrgNotHeld { .. })));
}

/// verifies: LLR-fm38ww, LLR-gr8x3r
///
/// PR-vt244s, org-node half: the chain was written, the node stopped before
/// committing, and on reopening, reconcile commits the kept update the chain
/// holds; the record's epoch equals the chain's.
#[tokio::test]
async fn reconcile_commits_the_update_the_chain_holds_after_a_crash() {
    let mut s = admit_b_directly(setup("reconcile-crash").await).await;
    let joiner_c = joiner_for_c(&mut s.svc_a);
    let update = s.svc_a.admit_member(&mut OsRng, s.org_id, &joiner_c).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    drop(s.svc_a);
    let reopened = reopen_store("reconcile-crash", "a", "pw_a");
    let chain_state = s.chain.get(&s.org_id).unwrap();
    let Reconciled::Committed { store, outcome } = reconcile(reopened.data(), s.org_id, chain_state, vec![]).unwrap() else { panic!("committed") };
    let record = store.orgs.iter().find(|r| r.org_id == s.org_id).unwrap();
    assert_eq!((record.epoch, record.root_hash, record.org_pub_key), (chain_state.epoch, chain_state.root_hash, chain_state.org_pub_key));
    let ProvisionalChange::ChangeSet { change_set, .. } = &update.change else { panic!() };
    assert_eq!(record.kept_change_set.as_ref(), Some(change_set));
    assert!(store.provisional_updates.iter().all(|u| u.resulting_root != update.resulting_root));
    assert!(outcome.revocations.is_empty());
}

/// verifies: LLR-fm38ww
///
/// A reconciled commit that removes this node returns `Removed`, with one
/// acknowledgement and `forget_organisation`'s successor.
#[tokio::test]
async fn reconcile_of_a_removal_signs_and_forgets() {
    let mut s = admit_b_directly(setup("reconcile-removal").await).await;
    let b_id = id_by_handle(&rec_of(&s.svc_a, s.org_id), "bob");
    let update = s.svc_a.revoke_member(&mut OsRng, s.org_id, b_id).unwrap();
    s.chain.apply_update(s.org_id, update.resulting_root, update.org_pub_key, rec_of(&s.svc_a, s.org_id).epoch).unwrap();
    // B holds A's revocation as a provisional update of its own: copy it into B's store data.
    let mut b_store = s.svc_b.store_data().clone();
    b_store.provisional_updates.push(ProvisionalUpdate { persona_id: s.pid_b.clone(), ..update });
    let seeds = vec![s.b_device_kp.device_seed()];
    let Reconciled::Removed { store, acknowledgements } = reconcile(&b_store, s.org_id, s.chain.get(&s.org_id).unwrap(), seeds).unwrap() else { panic!("removed") };
    assert_eq!(store, b_store.forget_organisation(s.org_id));
    assert_eq!(acknowledgements.len(), 1);
}
```

   (`store_data()` is a read-only accessor on `OrgService` added in this task
   under `#[cfg(feature = "test-support")]`.)
2. Implement `org-node/src/reconcile.rs` (`#[cfg(feature = "app")]`):
   `Reconciled` and `reconcile` per LLR-gr8x3r and LLR-fm38ww. Factor the
   pure commit step out of `commit_update` into
   `pub(crate) fn commit_step(store: &StoreData, org_id: OrgId, update: &ProvisionalUpdate, chain: &OrgState, device_seeds: Vec<DeviceSeed>) -> Result<Committed, OrgNodeError>`
   (where `Committed` is either the successor and outcome, or the removal's
   successor and acknowledgements), which `commit_update` and `reconcile`
   both call, so they cannot drift. Add `OrgService::reconcile(&mut self,
   rng, org_id) -> Result<Reconciled, OrgNodeError>` that reads the chain
   once, calls `reconcile`, and adopts and saves a successor — the interim
   caller until S3b-io's startup reconcile; the app does not call it in S3a.
3. Green: `cargo test -p org-node --features app,test-support --test reconcile --test commit_paths --test service_stories`.
4. Commit: `feat(org-node): S3 T10 — reconcile a record with the chain (LLR-gr8x3r, LLR-fm38ww; PR-vt244s org-node half)`.

### T11 — Source-scan absences

**Files touched:** org-node/tests/absences.rs
**Parallel:** no (serial, after T10)

1. Write, then mutate to see each fail: insert `use crate::service::ChainOps;`
   into `revocation.rs`, and `fn mutant(&mut self) { self.store.data_mut().orgs.retain(|_| true) }`
   into `service.rs`; run; record both failures; revert.

```rust
fn rust_files(dir: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

/// verifies: LLR-xgefn8
#[test]
fn revocation_and_reconcile_do_no_io() {
    for file in ["revocation.rs", "reconcile.rs"] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(file);
        let source = std::fs::read_to_string(&path).unwrap();
        for name in ["async ", "ChainOps", "ChainReader", "read_state", "OrgEndpoint", "iroh::", "std::fs", "tokio::"] {
            assert!(!source.contains(name), "{file} names {name}");
        }
    }
}

/// verifies: LLR-23sfdh
#[test]
fn only_forget_organisation_removes_records_and_personas() {
    let mut files = Vec::new();
    rust_files(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut files);
    for path in files {
        let source = std::fs::read_to_string(&path).unwrap();
        let is_store = path.ends_with("store.rs");
        for pattern in ["orgs.retain", "personas.retain", "orgs.remove", "personas.remove", "orgs.clear", "personas.clear"] {
            assert!(is_store || !source.contains(pattern), "{} uses {pattern}", path.display());
        }
        assert!(is_store || !source.contains("fn forget_organisation"), "{} defines its own forget_organisation", path.display());
    }
}
```

2. Run `cargo test -p org-node --features app,test-support --test absences`; expected all pass.
3. Commit: `test(org-node): S3 T11 — revocation and reconcile do no IO; one removal step (LLR-xgefn8, LLR-23sfdh)`.

### T12a — The member-sender rule

**Files touched:** org-node/src/error.rs, org-node/src/service.rs, org-node/src/revocation.rs, org-node/tests/support/mod.rs, org-node/tests/admission_sender.rs, org-node/tests/service_stories.rs, org-node/tests/receive_chain_reads.rs, org-node/tests/revocation.rs, org-node/tests/commit_workflow_errors.rs, app/src-tauri/src/events.rs, app/src-tauri/tests/receiver_events.rs, app/docs/architecture/2026-10-05-decomposition.md
**Parallel:** no (serial, after T11; before T12)

Owner rulings R1 and the acknowledgement sender rule of 2026-10-07
(decision 16): an update for an Organisation the node holds, and a
revocation, are acted on only when the sending Device is listed in the
node's current committed record — "Using iroh a connection can only be
established via mutually known public keys, but I agree with only accepting
updates and revocations from members"; on revocations, "the receiver still
thinks of the other party as a member (based on the information they have)
so nothing is different here". A first admission is **not** checked: "to
avoid scenarios where something happens to the admin's device during this
window"; "The invite id plays no role in the update"; "the new joiner has no
org information to disclose, and they verify the org information they
receive on-chain so the risk here is only a new joiner being DoS'ed which is
acceptable". An acknowledgement is accepted only from the Device it names:
"Acknowledgements of revocations will be sent from devices on the revocation
list, so their ids can be checked before accepting." Items: LLR-2r2fha
(REQ-ztdza4 as amended), LLR-kzgjz8 (REQ-ea4qs5), LLR-5azhry as amended and
LLR-3aysup (REQ-b462sh as amended); RC-u7kdam, RC-eydn8t.

1. Red, the two error variants, in `org-node/tests/commit_workflow_errors.rs`:

```rust
/// verifies: LLR-2r2fha, LLR-kzgjz8, LLR-5azhry
///
/// The two sender refusals are distinct from every S3 variant and name the
/// Organisation.
#[test]
fn the_sender_refusals_are_distinct_and_name_the_organisation() {
    let org_id = OrgId::new([7; 20]);
    let sender = OrgNodeError::SenderNotListed { org_id };
    let ack = OrgNodeError::AcknowledgementNotFromItsDevice { org_id };
    assert_ne!(sender, ack);
    for e in [&sender, &ack] {
        assert!(e.to_string().contains(&format!("{org_id:?}")), "{e}");
        assert_ne!(*e, OrgNodeError::AcknowledgementForListedDevice { org_id });
        assert_ne!(*e, OrgNodeError::RevocationNotForThisDevice { org_id });
    }
}
```

   Run `cargo test -p org-node --features app,test-support --test commit_workflow_errors`;
   expected: does not compile (E0599, no such variant). Add to `error.rs`:
   `SenderNotListed { org_id }` ("message for organisation {org_id:?} from a
   device its record does not list") and `AcknowledgementNotFromItsDevice {
   org_id }` ("acknowledgement for organisation {org_id:?} was not sent by
   the device it names"). Green. The app no longer builds (exhaustive match,
   step 7).

2. Red, the acknowledgement's sender check, in `org-node/tests/revocation.rs`
   (pure function; update every existing `check_acknowledgement(store, ack)`
   call to pass `ack.device` as the sender first, so the existing tests keep
   their meaning):

```rust
/// verifies: LLR-5azhry
///
/// An acknowledgement delivered by any Device but the one it names is
/// refused before its signature is checked: a forged signature from the
/// wrong sender reports the sender, not the signature.
#[test]
fn an_acknowledgement_from_another_device_is_refused_before_its_signature() {
    let (store, ack) = genuine_acknowledgement_fixture();
    let other = org_node::test_fixtures::device_key(0x61);
    assert_ne!(other, ack.device);
    let mut forged = ack.clone();
    forged.signature = Signature64([0; 64]);
    for candidate in [ack.clone(), forged] {
        assert_eq!(
            check_acknowledgement(&store, other, candidate),
            Err(OrgNodeError::AcknowledgementNotFromItsDevice { org_id: ack.org_id })
        );
    }
    assert!(check_acknowledgement(&store, ack.device, ack.clone()).is_ok());
}
```

   (`genuine_acknowledgement_fixture` is T6's setup in this file; factor it
   out if it is inline.) Expected: does not compile (arity). Implement
   LLR-5azhry as amended: the new `sender` argument, checked second, after
   `AcknowledgementNotHeld`. Green.

3. Support: in `org-node/tests/support/mod.rs` add
   `deliver_from_to_self_delete(svc, sender_seed: [u8; 32], msg)` and
   `deliver_from_to_receive(svc, sender_seed, msg)`, which send from an
   endpoint bound from `sender_seed`; keep `deliver_to_*` (random relay) for
   the refusal cases, and say so in their doc comments. `device_seed_of(svc,
   persona_id) -> [u8; 32]` reads a Persona's seed for tests (as
   `a_removal_relayed_by_the_member_it_removes_is_committed` already does).

4. Red, the receive paths, rewriting the tests that pin any-sender behaviour
   (each rename keeps a `*Rewritten 2026-10-07 (S3 T12a)*` line naming the
   old name), in `org-node/tests/admission_sender.rs`:

   - `update_relayed_by_a_non_member_after_admission_is_committed` →
     `update_relayed_by_a_non_member_after_admission_is_refused`
     (`verifies: LLR-2r2fha, LLR-u6rq4s`): R relays A's genuine C-admission
     to B's `receive_and_verify`; assert `Err(SenderNotListed { org_id })`,
     B's record (epoch, root, `last_seq`, members) unchanged and B's store
     bytes unchanged; then A's own device sends the same message and B
     commits epoch 3 (the normal case in the same test).
   - `a_revocation_relayed_by_a_non_member_is_still_acted_on` →
     `a_revocation_relayed_by_a_non_member_is_refused`
     (`verifies: LLR-kzgjz8`): R relays A's genuine notice for B to B's
     self-delete path; assert `Err(SenderNotListed { org_id })`, B keeps its
     record and Persona, and no chain read happened (`setup_counting`), which
     also shows the seed source was not reached (it is called only after the
     read and every check, LLR-tx8ruv). Then A's own device sends it and B
     self-deletes with one acknowledgement.
   - `pr_u4c2vp_an_update_relayed_by_a_non_member_is_committed_on_the_self_delete_path` →
     `pr_u4c2vp_an_update_relayed_by_a_non_member_is_refused_on_the_self_delete_path`
     (`verifies: LLR-2r2fha, LLR-3q63zv, PR-u4c2vp`): R relays A's genuine
     C-admission to B's self-delete path; assert `Err(SenderNotListed)`, B's
     epoch unchanged, no chain read.
   - `a_malformed_change_set_from_a_device_outside_the_record_is_refused_as_malformed`
     → `a_malformed_change_set_from_a_device_outside_the_record_is_refused_before_it_is_decoded`
     (`verifies: LLR-2r2fha, LLR-mcdh85, LLR-9sknpa`): from R →
     `SenderNotListed`; the same bytes from A's device → `MalformedDelta`.
     Both without a chain read.
   - `a_removal_relayed_by_the_member_it_removes_is_committed` stays green
     unchanged (C is listed in B's record before the update); add
     `LLR-2r2fha` to its `verifies:` — it is the rule's "listed before,
     absent after" normal case.
   - `a_first_admission_relayed_by_another_device_is_committed` stays green
     unchanged: add a comment citing the owner's first-admission amendment
     and `verifies: REQ-xa6smf, LLR-2r2fha` (the exemption clause).

   In `org-node/tests/service_stories.rs`: the forged-revocation story (the
   test that binds `forger_device`, `[0x77; 32]`) now sees
   `SenderNotListed` from the forger; split it — from the forger:
   `SenderNotListed`, nothing deleted; the same forged notice sent from A's
   listed device: `RevocationProofRefused`, nothing deleted (keeps LLR-tx8ruv's
   chain clause covered). `an_org_information_commit_that_removes_the_node_forgets_through_the_one_step`
   delivers both messages with `deliver_from_to_receive` from A's device seed.

   In `org-node/tests/receive_chain_reads.rs`:
   `revocations_are_refused_without_writing` sends `still_listed` from A's
   device (`deliver_from_to_*`), keeping one chain read and
   `RevocationProofRefused`; add a third case, the same notice from a random
   relay: `SenderNotListed`, **zero** chain reads, on both paths
   (`verifies:` add `LLR-kzgjz8`). `an_acknowledgement_is_checked_and_reported`
   and the forged-acknowledgement case send from B's device seed
   (`verifies:` add `LLR-3aysup`); add a case sending B's genuine
   acknowledgement from a random relay: `AcknowledgementNotFromItsDevice`
   on both paths, no read, no write.

   New, in `org-node/tests/receive_chain_reads.rs`:

```rust
/// verifies: LLR-2r2fha, LLR-kzgjz8, LLR-3aysup
///
/// The sender is checked before the chain on both paths, for both kinds a
/// held Organisation receives; an acknowledgement is held to its own rule,
/// not to the record's list.
#[tokio::test(flavor = "multi_thread")]
async fn a_held_organisations_messages_from_an_unlisted_device_cost_no_chain_read() {
    let (s, counting) = setup_counting("unlisted-sender").await;
    let mut s = admit_b_directly(s).await;
    let admission = captured_admission_of_c(&mut s).await;
    let before = store_bytes("unlisted-sender", "b");
    let reads_before = counting.reads();
    let on_receive = deliver_to_receive(&mut s.svc_b, admission.clone()).await;
    let on_self_delete = deliver_to_self_delete(&mut s.svc_b, admission).await;
    for result in [on_receive.map(|_| ()), on_self_delete.map(|_| ())] {
        assert_eq!(result, Err(OrgNodeError::SenderNotListed { org_id: s.org_id }));
    }
    assert_eq!((counting.reads() - reads_before, store_bytes("unlisted-sender", "b")), (0, before));
}
```

   Run `cargo test -p org-node --features app,test-support --test admission_sender --test service_stories --test receive_chain_reads --test revocation`;
   expected: the rewritten and new tests fail at runtime (the old code
   commits, self-deletes or reports the later refusal), each recorded.

5. Implement in `service.rs`: `receive_one` returns `(DevicePublicKey,
   WireMessage)`; one private `fn sender_listed(record: &OrgRecord, org_id:
   OrgId, sender: &DevicePublicKey) -> Result<(), OrgNodeError>`; call it on
   both paths per LLR-2r2fha (after the held lookup, before
   `check_chain_free`; not on a first admission) and LLR-kzgjz8 (after
   `check_notice`, before the chain read, inside `receive_revocation`, which
   gains a `sender` argument); pass `sender` to `check_acknowledgement`
   (LLR-3aysup). Replace the "Nothing about the sender is checked" and "The
   authenticated sender is not checked" doc comments with the rule and its
   LLR IDs. Update `org-node/src/transport/endpoint.rs`'s module comment
   ("compare the key with nothing") the same way.

6. Sweep: run org-node's full `verify_commands` cargo line. Any further test
   that fails only because its message now comes from an unlisted Device
   (a fresh relay or `ROGUE_SEED`) is rewritten the same way — the refusal
   asserted from the unlisted Device, the original assertion kept from a
   listed one — and named in the commit message. A first-admission test is
   never in that set; if one fails, the implementation checked a first
   admission, which LLR-2r2fha forbids.

7. App: `classify_receive_error` gains `SenderNotListed { .. }` and
   `AcknowledgementNotFromItsDevice { .. }` as `ReceiveError` (refused
   before anything is verified, so not a verdict). Red first, in
   `app/src-tauri/tests/receiver_events.rs`:

```rust
// verifies: LLR-7bk6qh
#[test]
fn the_sender_refusals_are_classified_as_receiver_errors() {
    // Owner rulings of 2026-10-07: a message from a Device the record does
    // not list, or an acknowledgement not sent by its own Device, is refused
    // before anything is verified, so neither is a verdict on an update.
    let org_id = org_node::OrgId::new([1; 20]);
    for e in [OrgNodeError::SenderNotListed { org_id }, OrgNodeError::AcknowledgementNotFromItsDevice { org_id }] {
        assert_eq!(events::classify_receive_error(&e), ReceiverOutcome::ReceiveError { message: e.to_string() }, "{e:?}");
    }
}
```

   (red: the app does not compile until both arms exist; record it). Amend
   LLR-7bk6qh in place with a dated note naming the two variants and this
   test. Green: the app's `verify_commands`.

8. Run `GR_CONFIG=org-node/.guardrails/config.yaml .guardrails/scripts/check-trace.sh`
   (no MISSING-TEST for LLR-2r2fha, LLR-kzgjz8, LLR-3aysup) and the same for
   the app; and clippy per task.

9. Commit: `feat(org-node, app): S3 T12a — act on updates and revocations only from listed Devices, and on acknowledgements only from their own Device (LLR-2r2fha, LLR-kzgjz8, LLR-3aysup, LLR-5azhry; PR-u4c2vp)`.

### T12 — S3a close-out

**Files touched:** org-node/docs/problems/2026-10-06-revoked-snapshot.md, org-node/docs/problems/2026-09-09-org-node-problems.md (PR-u4c2vp), docs/plans/2026-10-06-org-io-commit-workflow.md, docs/verification/2026-10-07-worktree-org-io-commit-workflow.md, plus the file that defines PR-vt244s (find it with `GR_CONFIG=org-node/.guardrails/config.yaml .guardrails/scripts/find-items.sh show PR-vt244s`)
**Parallel:** no (last of S3a)

1. PR-qmvj83: `status: resolved` with the resolving test
   (`a_revoked_device_receives_only_its_notice`) and the date.
2. PR-vt244s: a dated note that the org-node half (reconcile, kept updates,
   T10) is done and the item stays open for S3b-io; record its age against
   org-node's 45-day limit (reached around 2026-10-24). Do not mark it
   `accepted`; raise the limit deliberately if S3b-io cannot land in time.
3. Run org-node's full `verify_commands` (with `QUINT_HOME`),
   `GR_CONFIG=org-node/.guardrails/config.yaml .guardrails/scripts/check-ids.sh --allow-draft-files`,
   and `check-trace.sh` (no `MISSING-TEST` for any S3a LLR or REQ); and the
   app's. Paste outputs into the verification record (`verify-before-merge`).
4. Quint: S3 changes no modelled transition (revocation content and
   acknowledgements are not modelled); record that the five invariants pass
   unchanged. Whether to model "a revoked Device receives only a proof" is a
   later model change, not this one.
5. *Amended 2026-10-07.* The individual residual review this step called
   for took place on 2026-10-07 (decision 16): the chain-authority residual
   at `org-node/docs/risk/2026-10-06-chain-authority.md:70` is restated and
   ruled (R6), and S3's residuals of HAZ-p6xkuz, HAZ-7ubhwz and HAZ-h6b34a
   are restated and ruled (R3, R4, R5) in
   `org-node/docs/risk/2026-10-07-commit-workflow.md`
   — each accepted, the controls changed where the owner said so. No
   residual review is left: confirm each file states its ruling, dated, and
   no "owner ruling pending" remains for these four
   (`grep -n "owner ruling pending" org-node/docs/risk/`); name the four
   rulings, and R1 and R2, in the merge report.
5a. PR-u4c2vp: replace its `resolution:` with the fix T12a made (the
   member-sender rule on the self-delete path, REQ-ztdza4 as amended,
   LLR-2r2fha), its test
   `pr_u4c2vp_an_update_relayed_by_a_non_member_is_refused_on_the_self_delete_path`
   and the date, keeping the 2026-10-05 resolution and the 2026-10-07 note
   as history; `status: resolved`. Its affects line gains LLR-2r2fha.
6. Update **Progress** below.
7. Commit: `docs: S3a close-out — PR-qmvj83 resolved, PR-vt244s narrowed, PR-u4c2vp fixed, verification record`.

**S3a merges early, before S2** (owner ruling of 2026-10-07): after T12,
run `verify-before-merge` and `merge-change` for this branch. S3b-io
(T13–T17) then follows, after S2 merges, as its own change in a new
worktree that works from this plan.

## S3b-io — org-io (after S2 merges)

These tasks are named and scoped here; their `verifies:` lines name IDs
that T13 mints, so their test code is written in T13 step 4, when those
IDs exist.

### T13 — Mint the org-io requirements and their risk passages

**Files touched:** org-io/docs/requirements/DRAFT-worktree-org-io-commit-workflow-commit-workflow.md, org-io/docs/risk/DRAFT-worktree-org-io-commit-workflow-commit-workflow.md, org-io/docs/architecture/DRAFT-worktree-org-io-commit-workflow-commit-workflow.md, docs/plans/2026-10-06-org-io-commit-workflow.md
**Parallel:** no (first of S3b-io, after S2 merges and this branch merges master)

1. Merge master (with S2) into this branch, keeping both sides' amendments
   per item as on 2026-10-07.
2. For each proposed requirement 1–11, run
   `GR_CONFIG=org-io/.guardrails/config.yaml .guardrails/scripts/new-id.sh REQ`
   and write it with that ID and `satisfies: derived`; mint org-io's control
   for HAZ-h6b34a's retained state with `new-id.sh RC`; write the
   `assesses:` passage, stating each S3 residual as the owner ruled on it
   individually on 2026-10-07 (decision 16; R5 governs HAZ-h6b34a's
   retained state); ruling D alone accepts nothing. Proposed 5 cites S4's fan-out requirement if S4 has merged,
   otherwise records the merge rule. Proposed 7 takes the device seed from
   S4's keychain path if S4 has merged, otherwise from org-node's interim
   `OrgService` until S4 lands.
3. Design the org-io items with `design-architecture` (the workflow module,
   the retained acknowledgement's storage, the append-only list) and mint
   their SDD and LLR IDs.
4. Replace the proposed numbering in this plan by the minted IDs and write
   T14–T16's red tests in full, each with its `verifies:` line.
5. Run `check-ids.sh --allow-draft-files` and `check-trace.sh` for org-io.
6. Commit: `docs(org-io): S3b — mint the commit-workflow requirements`.

### T14 — The publish path and the unknown-outcome write (proposed 1, 2, 4)

**Files touched:** org-io/src/workflow.rs, org-io/src/lib.rs, org-io/tests/publish_path.rs
**Parallel:** no (serial, after T13)

Red tests, over S2's mock chain writer and reader:
`a_write_is_submitted_only_for_a_held_provisional_update`,
`a_commit_follows_only_a_read_that_holds_the_root_and_key`,
`nothing_is_sent_before_the_commit`,
`an_unknown_outcome_rereads_until_committed_reconciled_or_discarded`,
`no_second_write_while_one_is_unknown`,
`discard_is_refused_while_its_write_is_in_flight`, and PR-924ftr's
reproduction: time out a write that then executes, start a second
admission, and assert the update committed and sent is the one the chain
holds and that the discard command removes a named update and its key.
Implement `workflow.rs` over org-node's values (T10's `reconcile`, the
commit step).

### T15 — Startup reconcile and the unverified state (proposed 3, 10)

**Files touched:** org-io/src/startup.rs, org-io/tests/startup_reconcile.rs
**Parallel:** yes, after T14, with T16

Red tests: PR-vt244s's reproduction (write the chain, drop the service
before commit, start again; the reconcile commits the kept update and the
record's epoch equals the chain's);
`behind_is_reported_and_changes_nothing`;
`an_unreachable_chain_marks_unverified_and_deletes_and_commits_nothing`;
`no_deletion_on_any_trigger_but_an_accepted_revocation` (the org-io half of
RC-ub82my).

### T16 — Revocation delivery, receipt, acknowledgement send and list (proposed 5–9, 11)

**Files touched:** org-io/src/revocation_delivery.rs, org-io/src/acknowledgements.rs, org-io/src/retry_limits.rs, org-io/tests/revocation_delivery.rs
**Parallel:** yes, after T14, with T15

*Amended 2026-10-07 (owner ruling R5 and the acknowledgement sender rule,
decision 16; proposed 7, 8 and 9 as revised).* The revoked Device keeps,
until delivery ends, the acknowledgement, its Device secret key (the
keychain item), up to three distinct Member DevicePublicKeys taken from the
record before deletion (the delivering peer first) and the retry limit. It
connects directly to each under its own endpoint identity — never relayed,
never a fresh endpoint, so each receiver's sender check (REQ-b462sh) sees
the Device the acknowledgement names; a peer that has not committed the
revocation refuses, and the next kept peer is tried. After three accept, or
no kept peer remains to retry, or the limit passes, it deletes all of it.
Receivers accept an inbound connection from a Device they hold as removed
for an acknowledgement only (with S4's transport). T13 designs how org-node
hands out the kept DevicePublicKeys with the acknowledgements (values out of
`revocation::accept` and the removal step). These red tests replace
`the_acknowledgement_is_sent_after_the_app_confirms_first_on_the_delivering_connection`,
`only_the_acknowledgement_address_and_limit_are_kept_and_all_three_go_at_delivery_or_the_limit`
and `acknowledgements_from_any_endpoint_are_checked_and_appended_never_dropped`
below:
`the_acknowledgement_is_sent_after_the_app_confirms_first_to_the_delivering_device_under_the_devices_own_identity`;
`the_acknowledgement_goes_to_up_to_three_distinct_member_devices_fewer_when_fewer_exist`;
`a_peer_that_still_lists_the_device_refuses_and_the_next_kept_peer_is_tried`;
`only_the_acknowledgement_key_kept_peers_and_limit_are_kept_and_all_go_after_three_deliveries_or_the_limit`;
`an_acknowledgement_is_accepted_only_from_the_device_it_names_and_appended_never_dropped`;
`a_removed_device_may_connect_for_an_acknowledgement_and_nothing_else`.

Red tests: `the_committed_update_and_its_notices_are_what_is_sent`;
`a_revocation_is_resent_with_backoff_until_acknowledged_or_the_limit`;
`an_accepted_revocation_writes_the_image_deletes_the_keychain_item_and_organisation_data_then_waits_for_the_app`;
`the_acknowledgement_is_sent_after_the_app_confirms_first_on_the_delivering_connection`;
`only_the_acknowledgement_address_and_limit_are_kept_and_all_three_go_at_delivery_or_the_limit`;
`acknowledgements_from_any_endpoint_are_checked_and_appended_never_dropped`;
`retry_limits_default_to_two_months_and_refuse_zero`. Keychain access uses
S4's mock credential builder under `test-support` only.

### T17 — S3b-io close-out

**Files touched:** docs/verification/2026-10-07-worktree-org-io-commit-workflow.md, docs/plans/2026-10-06-org-io-commit-workflow.md, the files defining PR-vt244s and PR-924ftr (find each with `find-items.sh show`)
**Parallel:** no (last)

Resolve PR-vt244s and PR-924ftr, naming their reproducing tests; run every
unit's `verify_commands`, `check-units.sh`, `check-ids.sh` and
`check-trace.sh`; complete the verification record; hand to `merge-change`.

## Progress

| Step | Status | Commit |
|---|---|---|
| Requirements, risk and design (org-node); owner rulings | done | `a9df781` … `bda30d5` |
| Merge master `1f52c36` (the key-pair change), conflicts resolved | done | `12730dd` |
| Reconcile S3 with the merged code and S4's keychain rulings | done | `03e891b` |
| Implementation plan (this section) | done | the commit that adds it |
| Owner rulings of 2026-10-07 applied (residuals pending, layout change, early S3a merge) | done | `0a8ff95` |
| Owner rulings of 2026-10-07, later: key-pair residuals accepted (HAZ-vxabf9 for the key itself, CGKA to Phase 3); S3 residuals pending, reviewed at T12 | done | the commit that records them |
| Merge master `a57d760` (branch coverage) | done | `e6ad472` |
| T0 — register the new test targets (no tests, so no red -> green line; per-task clippy narrowed) | done | `4768bff` |
| T1 — the ten error variants. red -> green: the_ten_new_variants_are_distinct_and_name_the_organisation; stale_state_names_both_epochs_and_the_proof_refusal_names_its_cause — each watched fail (E0599, no such variant) before the variants existed, green after. Variants print `{org_id:?}` (OrgId and Epoch have no Display). Carried to T8: the app's exhaustive `OrgNodeError` match in app/src-tauri/src/events.rs needs arms for the ten variants before the app builds (T8 runs the app's verify_commands) | done | `602c809` |
| T3 — `revocation.rs`: notices and the acknowledgement's signed bytes. red -> green: notices_for_names_each_removed_device_with_a_proof_from_the_committed_record; notices_for_orders_by_member_then_slot_and_is_empty_when_nothing_was_removed; notices_for_refuses_an_uncalculated_trie; acknowledgement_signed_bytes_are_the_domain_then_the_postcard_tuple — each watched fail (E0432, no module; then the assert against a stub) before the implementation, green after. Carried to T4: a length-refusal test for `Signature64`'s decode | done | `9406523` |
| T2 — `forget_organisation` and `kept_change_set` (golden store re-pinned per ruling 12). red -> green: forget_organisation_removes_everything_of_one_organisation_and_nothing_else; forget_organisation_of_an_unheld_organisation_changes_nothing; forget_organisation_of_the_last_organisation_leaves_an_empty_store — each compile-red (E0599, no method) before the implementation; kept_change_set_round_trips_through_the_store — compile-red (no field); persona_store_plaintext_is_pinned — red against the new pin before the field existed; all green after. Also touched tests/secret_redaction.rs and tests/persona_records.rs (the new field); the merge added the field to tests/revocation.rs's literal | done | `8b9e085` |
| T4 — the Wire message of three kinds. red -> green: the_three_kinds_round_trip_with_their_indices; malformed_revocations_and_acknowledgements_are_refused; an_acknowledgement_signature_decodes_only_at_exactly_64_bytes — each compile-red (E0533/E0599, the variants did not exist) before wire.rs changed, green after (the last one was not watched red at runtime: the length check already existed from T3); revocation_wire_message_is_pinned — re-pinned to the new layout, no red recorded. INTERIM: 17 tests `#[ignore = "S3 T8"]` (14, send_update now refuses an unlisted recipient) and `#[ignore = "S3 T9"]` (3, relabelled or forged revocations); T8 and T9 must un-ignore every one, and the gate requires 0 ignored. `revocation` module now `transport`-gated, `notices_for` stays `app`-gated | done | `62307d9` |
| T5 — deciding a notice, signing the acknowledgements. red -> green: check_notice_refuses_an_unheld_organisation_then_another_device; accept_refuses_in_order_and_changes_nothing; accept_signs_then_forgets; a_missing_device_seed_is_refused_before_anything_is_deleted — each compile-red (E0432), then runtime-red against a stub, before the implementation; green after. `sign_acknowledgements` builds an `ed25519_dalek::SigningKey` from the seed (SigningKeypair has no `sign`); for review: that the key is zeroised on drop. Stub swap done with a scripted string replacement, not the editor (convention deviation, reviewed and tested) | done | `d82fef6` |
| T6 — checking a received acknowledgement. red -> green: a_genuine_acknowledgement_verifies; acknowledgements_are_refused_in_order; an_acknowledgement_with_an_edited_field_is_refused — each compile-red (E0432), then runtime-red against a stub, before the implementation; green after | done | `b100492` |
| T6b — the invalid-Device-key boundary. red -> green: an_acknowledgement_whose_device_is_no_curve_point_does_not_decode — red with a valid key spliced in its place (the acknowledgement decoded), green with the non-point bytes. The `VerifyingKey::from_bytes` arm in `check_acknowledgement` is unreachable (DevicePublicKey parses its bytes); for the deslop pass: verify through `ack.device` directly and drop the redundant parse | done | `89f9da1` |
| T7 — every commit keeps its Change set. red -> green: every_commit_keeps_the_change_set_that_produced_the_record; a_refused_commit_keeps_the_earlier_change_set; revocation_of_another_member_is_committed_not_self_deleted (extended) — each runtime-red before `commit_held` set the field, green after | done | `cc00353` |
| T8 — commit returns its notices; send_update sends them (PR-qmvj83). red -> green: a_revoked_device_receives_only_its_notice; send_update_chooses_by_record_and_outcome — compile-red, then runtime-red against a stub with no notices, green after; the_commit_workflow_refusals_are_classified_as_receiver_errors (app) — runtime-red with the ten variants sorted as verdicts, green after. 6 of the 14 T8-ignored tests un-ignored; 8 relabelled `S3 T9` (they fail only on the receiver's interim refusal), so T9 un-ignores 11. App builds again (ten variants classified as receiver errors) | done | `e862cb1` |
| T9 — receive paths decide revocations and acknowledgements; one removal step. red -> green: a_revoked_node_acknowledges_then_forgets_everything; an_org_information_commit_that_removes_the_node_forgets_through_the_one_step; revocations_are_refused_without_writing; an_acknowledgement_is_checked_and_reported; a_node_that_commits_its_own_removal_forgets_the_organisation (extended); a_received_acknowledgement_emits_nothing (app) — each compile-red, then runtime-red against a stub, green after; the 11 T9-ignored tests rewritten to the design, each runtime-red against the stub, green after. 0 ignored in org-node and app. Ten S3 refusals stay receiver errors (reasoning in LLR-7bk6qh's dated note). Carried to T12: (1) the design's `CommitOutcome` gains `acknowledgements` (LLR-b27jr6 returns them) — amend the SDD shape list; (2) a founding Persona has `member_id: None` (commit_genesis never sets it), so a founder committing its own removal signs no acknowledgement — investigate and book a problem report | done | `76e9548`, `39b23c9` |
| T10 — `reconcile`. red -> green: reconcile_reports_one_outcome_from_the_record_and_the_state; reconcile_is_behind_when_no_stored_update_matches_both_root_and_key; reconcile_commits_the_update_the_chain_holds_after_a_crash; reconcile_returns_a_refusal_of_the_commit_step_as_the_error (tightened after its first form passed the stub); reconcile_of_a_removal_signs_and_forgets; the_service_reconcile_adopts_and_saves_the_committed_record — each compile-red, then runtime-red against a stub, green after. One removal step (`removal_step`) shared by commit and receive; kept-update selection in `StoreData::held_update_for` | done | `9e64c0d` |
| T11 — source-scan absences. red -> green: revocation_and_reconcile_do_no_io; only_forget_organisation_removes_records_and_personas; forget_organisation_is_defined_once_and_called_only_from_accept_and_the_removal_step — each watched red against planted violations, green with them removed. Known limit: a removal through a renamed alias is not caught | done | `c77728e` |
| T10b — seeds only on the removal path; held check before the chain read. red -> green: a_commit_that_does_not_remove_this_node_never_obtains_the_device_seeds — red with the seed source called eagerly, green after; the_service_reconcile_refuses_an_unheld_organisation_without_a_chain_read — red on T10's code (OrgNotOnChain after a read), green after. LLR-gr8x3r's signature now a seed source (`impl FnOnce() -> Vec<DeviceSeed>`) | done | `d2a70b1` |
| T10c — `accept` takes a lazy seed source; a refused notice never reads the seeds. red -> green: a_refused_notice_never_calls_the_seed_source — red with the source called first in `accept`, green with the call just before signing | done | `316f43f` |
| Owner rulings at the S3a close-out residual review (2026-10-07, decision 16): R1 member-sender rule (first admission exempt, by the owner's amendment), acknowledgement sender rule, R2 epoch rule kept, R3–R6 residuals accepted with restated controls (R4 S5 constraints, R5 three-peer delivery); REQ-ea4qs5, RC-u7kdam, LLR-2r2fha, LLR-kzgjz8, LLR-3aysup minted; T12a added, T12 and T16 amended | done | the commit that records them (task branch `worktree-org-io-commit-workflow-s3r1`) |
| T12a — the member-sender rule (PR-u4c2vp fixed). red -> green: the_sender_refusals_are_distinct_and_name_the_organisation — compile-red; an_acknowledgement_from_another_device_is_refused_before_its_signature, the_sender_check_comes_after_the_held_lookup_and_before_the_rest — runtime-red against a stub ignoring the sender; update_relayed_by_a_non_member_after_admission_is_refused, a_revocation_relayed_by_a_non_member_is_refused, pr_u4c2vp_an_update_relayed_by_a_non_member_is_refused_on_the_self_delete_path, a_malformed_change_set_from_a_device_outside_the_record_is_refused_before_it_is_decoded, revocation_from_an_unknown_device_leaves_the_record_in_place, revocations_are_refused_without_writing, a_held_organisations_messages_from_an_unlisted_device_cost_no_chain_read — each runtime-red against the pre-change code; an_acknowledgement_is_checked_and_reported — runtime-red against stubbed wiring; the_sender_refusals_are_classified_as_receiver_errors (app) — compile-red (E0004); all green after. Nine sweep rewrites gained a relay-refusal assertion that was not watched red on its own (the classifier refused the no-check stub); that behaviour is red-proven by the tests above | done | `c2bc250` |
| T12 — S3a close-out (no tests, so no red -> green line). PR-qmvj83 resolved (`a_revoked_device_receives_only_its_notice`); PR-u4c2vp's resolution replaced with T12a's fix; PR-vt244s noted (org-node half done, open for S3b-io, 28 of 45 days, limit reached 2026-10-24). Owner rulings of 2026-10-07 recorded: key window (HAZ-h6b34a, risk draft) and join gate kept (RC-2ferct/REQ-8amu2a, chain-authority risk). Design: `CommitOutcome.acknowledgements` and the lazy seed sources added to the shape list (and a note at the chain-authority shape list). Booked PR-6f3qku (clippy --all-targets red on four pre-S3 targets) and PR-eqs4fs (a founding Persona has no `member_id`, so a removed founder signs no acknowledgement; confirmed by a throw-away probe). Gate: org-node 291 passed, 0 failed, 0 ignored, four fuzz targets ran; quint typechecks and five invariants pass unchanged; app 162 + 44 passed, svelte-check clean; check-ids and check-trace exit 0 for org-node and app. Verification record drafted | done | the commit that records it (task branch `worktree-org-io-commit-workflow-s3t12`) |
| T12b — PR-eqs4fs fixed: `commit_genesis` binds the founder's MemberId. red -> green: a_founder_that_commits_its_own_removal_signs_its_acknowledgement — red on the unfixed code (no acknowledgement signed), green after; commit_genesis_creates_the_record_once_the_chain_carries_the_root re-pinned from `member_id: None` to the genesis MemberId. LLR-hby4jr amended. org-node 292 passed, 0 failed, 0 ignored | done | `2b7f920` |
| Robustness fix (gate finding) — ten LLRs gain a new test of the missing kind. red -> green, each watched fail against a temporary mutation of the code under test, green restored: check_notice_returns_the_persona_the_notice_is_for (LLR-r7zm39); an_accepted_notice_produces_a_successor_and_leaves_the_store_passed_in_unchanged (LLR-uw7nmv); a_signing_refusal_is_returned_unchanged_with_nothing_forgotten (LLR-r8qhky); a_signature_without_the_acknowledgement_domain_is_refused (LLR-gbe9bt); a_change_set_that_decodes_and_extends_the_record_is_taken (LLR-9sknpa); an_envelope_past_the_mark_is_decoded_and_verified_with_no_signature_or_sender (LLR-mcdh85); after_an_unheld_revocation_the_expected_admission_is_still_taken (LLR-38e2kn); a_revocation_from_a_listed_device_reaches_accept_on_both_paths (LLR-kzgjz8); the_self_delete_path_acts_on_an_update_and_a_removal_from_a_listed_device (LLR-3q63zv); a_commit_that_removes_another_member_forgets_nothing (LLR-23sfdh) | done | `1ac9991` |
| Robustness fix 2 (gate round 2) — red -> green: a_removal_that_cannot_sign_forgets_nothing (LLR-b27jr6) — red with `removal_step` ignoring the signing error, green restored. LLR-6p4pj2: abnormal seed case unreachable through the service (every bound Persona supplies its own seed), dated N/A note in its LLR. Coverage measured ad hoc on tree 17888a5 (org-node stable 86.61% lines, 83.00% regions; nightly branches 120/142, 84.51%); owner accepted the gap with the figures recorded, floors at tooth 5 (2026-10-07) | done | `a1f5b66` |
| Review round 1 fixes — code (findings 1, 2, 3, 7). red -> green: every_commit_keeps_the_change_set_that_produced_the_record (rewritten: a first admission keeps the admitting Change set) — red on the old code, green after; a_genesis_listing_no_leaf_with_the_personas_device_is_refused (LLR-wzqqg9) — red with the lookup falling back to the first member; an_org_information_body_with_trailing_bytes_is_refused (LLR-js9dsu) — red with leftover bytes accepted; reconcile_of_a_crash_after_removing_another_device_returns_its_notice (LLR-a8z7r5) — red with revocations emptied; all green restored. Docs (findings 4, 5, 6, 8, 9): HAZ-ep6uzs and HAZ-p6xkuz residuals restated, PR-q8r32t resolved, interim key exposure stated under HAZ-45ucqx, PR-zf924s booked | done | `2324cbd`, `c454ed5` |
| Review round 2 fixes (last round; findings 1, 3; 2 booked) — red -> green: a_chain_state_at_the_records_epoch_with_another_root_is_a_conflict (LLR-tx8ruv) — red on the code before the fix (the seed source was called), green after; accept_signs_then_forgets rewritten to later epochs (the equal-epoch case is now a conflict by design). Stale docs fixed; PR-m9betm booked (a bound Persona with no MemberId is skipped, unreachable today) | done | `d15ebf4` |
| T1–T12 (S3a, org-node), then merge early, before S2 | T1–T12 done; verify-before-merge and merge-change next | |
| T13–T17 (S3b-io, org-io), a follow-up change after S2 merges | blocked on S2 | |

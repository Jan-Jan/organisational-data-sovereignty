# org-io — the membership publish workflow, in stages

Roadmap for resolving PR-vt244s (`org-node/docs/problems/2026-09-09-org-node-problems.md`:
the publish path writes the chain before the local record) by redesigning
the workflow around it. Agreed with the owner on 2026-10-06, during the
`resolve-problem` and `grill-requirements` sessions that opened
`worktree-org-members-absence-proofs`.

The stages are numbered S0 to S8, so they can't be confused with the
project's phases (ODS Phase 1 to Phase 3). Each stage is its own change
worktree and its own signed squash merge, and each leaves the base branch
green. Nothing waits to land as one big change at the end.

## The required behaviour

1. Admins make provisional changes to the members trie (org-members' pending
   changes).
2. Someone decides to calculate a new root covering all the provisional
   changes, and an admin writes the new Organisation state on-chain.
3. The writing device shares the change: the full Organisation state with new
   and existing members, and a revocation proof with revoked devices.
4. Recipients hold the received state as provisional.
5. A recipient reads the chain and verifies the provisional state. If it is
   still a member, the verified trie replaces the current one and the device
   keeps the delta that produced it. If it is no longer a member, it deletes
   everything it holds for the Organisation, and only on a verified
   revocation.
6. A device that sees the chain move asks any device in good standing for
   the update. The answer is the kept delta if the device is one change
   behind, otherwise the full trie.

There is no legacy trie. A requester is in good standing if its Device key
is in the responder's current trie; the iroh endpoint id is the Device key,
so the connection authenticates it.

## Unit responsibilities

| Unit | Responsibility |
|---|---|
| org-members | The trie: provisional changes, recalculation, deltas, candidate tries, absence proofs |
| person | Identity types |
| on-chain-client | Chain library: decoders, verifier, building and submitting calls |
| org-node | No IO: builds, serialises, deserialises, encrypts and verifies Organisation records, Wire messages and provisional updates |
| **org-io** (new) | All IO: the chain (through on-chain-client), other nodes (iroh: point-to-point queries and gossip), local storage (opaque bytes only); runs the workflow |
| app | UI, and deciding when things happen: when to read the chain, publish, or ask peers |

## Rulings that shape later stages

- **Revoked-device list:** held by every node, not only admins. Each node
  builds it from the deltas it verifies and merges it with peers' lists as a
  CRDT. Entries are (Device key, MemberId) pairs. A requester supplies its
  MemberId, and a node answers only when the pair matches an entry, so a
  revoked device cannot collect other members' leaves. Nodes need no concept
  of an admin for this. **The list is never pruned for now** (owner ruling,
  2026-10-06). Later, each Organisation sets its own retention limit, for
  example five years, separately for entries the revoked device has
  acknowledged (an acknowledgement is signed by the revoked Device key) and
  for entries it has not. A key pruned past that limit falls outside the
  never-again rule below, and that is the Organisation's accepted risk.
- **Good standing is judged against the responder's own trie** (owner
  ruling, 2026-10-06). A request names the epoch the requester wants. A
  responder answers if it holds that epoch's trie or delta and the
  requester's Device key is in it, and declines otherwise. It does not answer
  a new member's device until it holds the trie that contains that device.
  A revoked device therefore gets nothing it did not already hold, and learns
  of its revocation only through an absence proof. No chain read happens per
  request, which avoids the request-driven chain reads that master's
  HAZ-bedm57 warns about.
- **Device keys are never admitted again.** A Device key must be unique
  across the members trie and the revoked list together. Handing a device to
  another member means revoking it and enrolling it again with new keys. The
  unit that holds the list enforces this, including a move inside a single
  delta. org-members leaves a key that is no longer held to its caller on
  purpose (`org-members/docs/architecture/2026-10-03-key-uniqueness.md`,
  LLR-gjj6bx). The rule does not cover member keys; a person who leaves and
  returns gets a new identity (owner ruling 5 below), so an old member key
  does not come back either.
- **Revoking a device forces a rotation** of the Organisation key pair.
- **Admin node:** a node is an admin if its account is a signatory of the
  multisig that controls the Organisation's pure proxy, checked on-chain by
  org-io.
- **Two keys, and no private signing key is ever exposed** (owner rulings,
  2026-10-06). The Organisation key pair is X25519, Organisation-wide, for
  encryption and key agreement only; it never signs. There is no
  Organisation-wide signing key: on-chain authority is each admin user's own
  sr25519 signatory key, which org-io holds. Neither that key nor any device
  ed25519 private key may ever be exposed. org-io states this as REQ-v4tfap
  (S2); the same ruling binds org-node and person for device keys, checked by
  their existing items. `ODS_ADMIN_SEED` is development-only; production
  custody is S8. Until S8 the chain read stays coupled to that seed, so
  release builds read no chain either (owner ruling, 2026-10-07).

## Stages

| Stage | Content | Units | Depends on |
|---|---|---|---|
| S0 | Merge `worktree-org-node-chain-authority` (chain write in on-chain-client, provisional updates persisted, no admin concept in org-node) | org-node, on-chain-client, app | none |
| S1 | Absence proofs (REQ-535jcd, REQ-tk2qqj, REQ-yyuxh8) and PR-jq43gx (no panics in `MemberId::bit` and `DefaultHashes::at_level`) | org-members | none; runs alongside S0 |
| S2 | Create org-io: ADR, dependency-edge assessment, move chain read, the app's write wiring and the signing key's custody into it, plus the signatory read; app depends only on org-io. No behaviour change. Resolves PR-b795an (writer coverage). *Design now; code after the key-pair change merges (see "Overlap with the key-pair change")*. Owner rulings of 2026-10-07: the chain read stays coupled to the development seed (release builds read no chain until S8); the own-admin check is org-io's function and tests only, not wired into the app (first caller S3b-io). See "Owner's rulings on S2 (2026-10-07)" | org-io, org-node, app | S0; code also the key-pair change |
| S3 | Commit workflow: persist provisional → write → read and verify → promote, keeping the delta; reconcile with the chain at startup; delete only on a verified revocation; revoked devices get only an absence proof and answer with a signed acknowledgement. Resolves PR-vt244s, PR-qmvj83 (re-check it first: the key-pair change may already resolve or narrow it) and PR-924ftr. ~~Key rotation is S3b~~ Key rotation comes with the key-pair change. Split on 2026-10-07 (owner ruling, recorded in `docs/plans/2026-10-06-org-io-commit-workflow.md`) into S3a (org-node, may merge before S2) and S3b-io (org-io, after S2). **S3b-io wires the own-admin check:** it is the first caller of `OrgIo::is_own_admin`, which S2 implements in org-io but does not wire into the app (owner ruling, 2026-10-07) | org-io, org-node | S2 |
| S3b | *Absorbed by `worktree-org-node-org-key-pair` (2026-10-06); no separate stage. The row is kept as history.* The X25519 Organisation key pair, rotated on every device revocation. Resolves PR-xwek5e ("rotate"), takes in PR-szkat6 and HAZ-ep6uzs | org-io, org-node | S3 |
| S4 | iroh, the invitation exchange, and storage file IO into org-io; org-node becomes IO-free. Re-targets PR-kwwap5. Taken over from the `improved-type-safety` session on 2026-10-06 | org-io, org-node, app | S2; runs alongside S3 |
| S5 | Revoked-device list (CRDT, pairs), answering revoked devices with absence proofs, the never-again rule for Device keys | org-io, org-node | S1, S3, S4 |
| S6 | Pull queries between devices in good standing (delta, or full trie) | org-io, org-node | S3, S4 |
| S7 | Gossip spreading of updates; new SOUP | org-io, app | S6 |
| S8 | Production custody of the user's signatory key: the user is prompted to store it in the OS keychain, or uses a hardware signing device; the key is never stored in plaintext. Until S8, `ODS_ADMIN_SEED` is a development-only path (org-io's `dev-seed` feature) and release builds sign nothing. **S8 also splits the chain read from the write:** until then the read stays coupled to the development seed, so release builds read no chain either (owner ruling, 2026-10-07: keep coupled in S2, split in S8). Added 2026-10-06 by owner ruling; requirements minted when the stage starts | org-io, app | S2 |

## Overlap with the key-pair change

Checked on 2026-10-06 against the unmerged branch
`worktree-org-node-org-key-pair`: change 3 of the chain-authority sequence.
Its plan is `docs/plans/2026-10-06-org-key-pair.md`, and its requirements and
design are in `org-node/docs/*/DRAFT-worktree-org-node-org-key-pair-*.md`. On
that date the branch holds requirements, risk, design and the plan. No code has
been written yet.

- **It implements the X25519 Organisation key pair** and removes the
  Organisation secret (REQ-szq3ud, REQ-c29s93, REQ-bwx7eg, REQ-ju6vn2, control
  RC-9cefcn).
- **It rotates the key pair on every calculated provisional update**, not only
  on a device revocation. A fresh pair is drawn when the root of a provisional
  update is calculated (REQ-stx9v3) and taken into the record on commit
  (REQ-jy6ybw). That covers the ruling above that revoking a device forces a
  rotation. It resolves PR-g9u3xq (opened and resolved on that branch),
  PR-szkat6, PR-ve9zw8 and PR-xwek5e.
- **It splits the Wire message by recipient** (REQ-3dsweu; LLR-8hdu9x amended):
  `WireMessage::OrgInformation { envelope, record_snapshot, org_private_key }`
  goes to every Device the committed record lists, and
  `WireMessage::Revocation { envelope }` goes to a Device it no longer lists,
  with no snapshot and no key. A Device that is still listed refuses a
  revocation (LLR-pt32fx).
- **It opens app PR-3ue4va** (a committed update reaches one Device only) and
  hands it to the org-io work. The report affects REQ-nfr3n2, which S2 moves.

What follows for this roadmap:

1. **The key-pair change absorbs S3b.** Its row stays in the table above,
   marked, as history. That change carries out owner ruling 6 below, and no
   stage of this roadmap does.
2. **PR-qmvj83 is probably resolved, or at least narrowed.** With
   `Revocation { envelope }`, the revoked device no longer receives the member
   snapshots, which is the disclosure the report describes. What remains of the
   owner's model is that the revoked device should receive an absence proof, not
   a bare Envelope. S3 must check PR-qmvj83 after the key-pair change merges. If
   the disclosure was the whole defect, S3 resolves the report. Otherwise S3
   narrows it to the missing absence proof.
3. **S2's code waits for that change.** The key-pair change edits the app and
   org-node wiring that S2 moves: `app/src-tauri/src/commands.rs`, `events.rs`,
   `invitation.rs` and `submit.rs`, and `org-node/src/service.rs`, `store.rs`
   and `transport/wire.rs`. S2's design can go ahead now. Its develop phase
   starts after the key-pair change merges, from the merged master.
4. **PR-3ue4va (sending each update to every Device) belongs to S4**, with the
   transport. S2 moves REQ-nfr3n2 without changing what is sent.

PR-vt244s's age limit after S0 merges is 45 days (org-node's
`problem_age_days` on that branch), which falls around 2026-10-24. If S2 and
S3 cannot land by then, raise the limit deliberately; do not mark the item
`accepted`.

## What master's chain-authority change (`fdf4e77`) leaves for these stages

A sweep on 2026-10-06 compared the merged chain-authority change with these
rulings. Each item it found is listed here with the stage that settles it.

**Planned supersessions** (the roadmap already decides these):

- The planned "change 2" that moves iroh and peer communication to the app
  (`org-node/docs/requirements/2026-10-06-chain-authority.md`, the sequencing
  list), and the text that says org-node keeps sending, receiving and reading
  the chain. Superseded by S2 (chain read) and S4 (transport).
- The app submitting the chain write and holding the `ODS_ADMIN_SEED` signing
  key: REQ-nfr3n2, LLR-qhjp6g, LLR-be3zv9, the app's SOUP notes, and
  on-chain-client's "the app calls them" wording. In S2 the submission and
  the key custody move to org-io, and the app keeps the decision of when to
  publish.
- The planned "advisory signatory read" in org-node moves to org-io (S2).
  The app's warning-only RC-wzb48r can then become a control by design.
  (2026-10-07: S2 implements the own-status check in org-io; S3b-io is its
  first caller.)
- PR-kwwap5 was tied to the transport moving to the app. It is re-targeted
  to S4.
- REQ-uv3v5w and LLR-mkj4bz discard the update that was just committed, so
  its delta is lost. Amended in S3 to keep the delta that produced the
  current trie.
- There is no reconcile step at startup; that belongs to S3.
- The "no sender checks" rulings (REQ-xa6smf, REQ-ztdza4) cover receiving
  updates only. S5's requester check must say so.

**Owner's rulings on the sweep, 2026-10-06:**

1. **The invitation exchange moves in S4,** with the transport. org-node
   builds and serialises the messages, org-io holds the store and runs the
   exchange, and the app keeps the UI.
2. **The snapshot sent to a revoked device is a defect:** PR-qmvj83, filed
   against LLR-8hdu9x and HAZ-vxabf9. It is fixed in S3, after which a revoked
   device receives only an absence proof.
3. **Deletion may be triggered by either verified path,** a committed record
   or an absence proof, always checked against the current on-chain root.
4. **"Delete everything":** the revoked device deletes all data related to
   the Organisation. As its last step it sends, or answers with, a **signed
   acknowledgement**, and the device that receives it adds the
   acknowledgement to its CRDT revoked-device list. This amends LLR-eyc4ud in
   S3 or S5.
5. **A returning person gets a new identity.** Master's REQ-yp75u9 (one
   Persona per Organisation, never rebound) stays. This closes the rejoin
   case recorded below.
6. **New stage S3b** after S3: the X25519 key pair, and rotation of it on
   every device revocation. It resolves PR-xwek5e as "rotate" and takes in
   PR-szkat6 and HAZ-ep6uzs's residual. *(Later on 2026-10-06, the key-pair
   change absorbed S3b. It rotates the key pair on every calculated provisional
   update. See "Overlap with the key-pair change".)*
7. **PR-924ftr goes to S3, and PR-b795an to S2.**

## Owner's rulings on the S2–S4 design drafts (2026-10-06)

The S2, S3 and S4 design drafts (`docs/plans/2026-10-06-org-io-create.md`,
`docs/plans/2026-10-06-org-io-commit-workflow.md`,
`docs/plans/2026-10-06-org-io-transport.md`) ran in parallel. The owner
ruled on their open decisions:

- **A. An item that moves into org-io keeps its ID.** Its definition moves
  to org-io's ledger with a dated note, so existing `verifies:` lines keep
  resolving. This replaces the chain-authority precedent of restating the
  source item as an absence and minting a new ID.
- **B. org-node meets org-io by values in, values out.** org-node takes
  values and returns values, including the sealed store bytes org-io must
  write. It has no IO traits and no async. The chain read follows the same
  rule, so S2's draft, which keeps `ChainOps` in org-node, is revised to
  match.
- **C. Retry limits are set by the Organisation.** That covers how long a
  revoked device retries its signed acknowledgement, and how long the
  admin's node resends a revocation. Both default to 2 months.
- **D. The risk README's matrix wins.** S3 is unacceptable at every
  probability. Any "S3/P1 acceptable under the matrix" residual is restated
  as risk the owner accepted, with its benefit-risk reasoning.
- **Also accepted, as the drafts recommended:**
  - org-io re-exports the org-node types the app uses, transitionally, and
    narrows them in S4.
  - org-io reads `ODS_ADMIN_SEED`.
  - S2 checks only the node's own admin status, so RC-wzb48r stays a
    warning.
  - `OnChainReader` is deleted, and so is `OrgStateCache` if nothing uses
    it.
  - org-io has coverage and branch floors from its first merge.
  - The preflight moves in S2, so org-node's `chain` feature goes.
  - PR-b795an is fixed by measuring the `write` feature.
  - "Delete everything" includes the app's data, and the acknowledgement is
    sent only after the app confirms the deletion.
  - A device that can't reach the chain keeps its last verified record,
    marked unverified, and never deletes or commits.
  - Until S5, org-io keeps acknowledgements in an append-only list in its
    storage.
  - One receive path (fixes PR-kwwap5).
  - Outstanding invites move into the encrypted store, with no migration.
  - The PR-3ue4va fan-out lands in S4, in Networked mode.
  - One endpoint per Persona (PR-8qsnhx).
  - S4 makes store writes atomic and owner-only (PR-n696wv).
  - The receive loop holds no lock while it waits (PR-7v8anh).
  - org-io is class C.
  - The code for S2 and S4 starts after the key-pair change merges.

## Owner's rulings on S2 (2026-10-07)

Two questions the S2 reconciliation left open (the S2 plan's "Still open"
items 1 and 2) were ruled on 2026-10-07, each as the plan recommended:

1. **The chain read stays coupled to the development seed in S2.** As
   today, org-io builds the chain read and the chain write together, only
   when `ODS_CHAIN_WS`, `ODS_CONTRACT_H160` and `ODS_ADMIN_SEED` are set and
   only under `dev-seed` (REQ-8zuka3). A release build therefore cannot read
   the chain until S8. Accepted; S8 splits the read from the write.
2. **The own-admin check is implemented in org-io in S2 but not wired into
   the app.** S2 delivers `OrgIo::is_own_admin` with its tests (REQ-f3eu9n);
   the app gets no command for it. Its first caller is S3b-io.

## Recorded for ODS Phase 3 (Keyhive / CGKA)

*Closed 2026-10-06 by owner ruling: a returning person gets a new identity,
and master's REQ-yp75u9 (one Persona per Organisation, never rebound) keeps
the old key from coming back. The text below stays as the reasoning.*

**Rejoin with the same member key.** A member who leaves and is later
admitted again with the same member key makes new CGKA material decryptable
by any old device that still holds that private key, such as a laptop sold
without being wiped. Today that device cannot get ciphertext, because its
Device key is revoked and no peer will connect to it. The risk exists only
if document traffic can reach a device by a route that is not gated by
Device keys: a relay, a shared storage backend or a leaked sync log. The
owner ruled on 2026-10-06 that the never-again rule stays limited to Device
keys. Phase 3's design must check this route, and must not assume that
revocation covers it.

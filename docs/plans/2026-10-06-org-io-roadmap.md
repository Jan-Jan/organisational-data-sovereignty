# org-io — the membership publish workflow, in stages

Roadmap for resolving PR-vt244s (`org-node/docs/problems/2026-09-09-org-node-problems.md`:
the publish path writes the chain before the local record) by redesigning
the workflow around it. Agreed with the owner on 2026-10-06, during the
`resolve-problem` and `grill-requirements` sessions that opened
`worktree-org-members-absence-proofs`.

The stages are numbered S0 to S7, so they can't be confused with the
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

## Stages

| Stage | Content | Units | Depends on |
|---|---|---|---|
| S0 | Merge `worktree-org-node-chain-authority` (chain write in on-chain-client, provisional updates persisted, no admin concept in org-node) | org-node, on-chain-client, app | none |
| S1 | Absence proofs (REQ-535jcd, REQ-tk2qqj, REQ-yyuxh8) and PR-jq43gx (no panics in `MemberId::bit` and `DefaultHashes::at_level`) | org-members | none; runs alongside S0 |
| S2 | Create org-io: ADR, dependency-edge assessment, move chain read, the app's write wiring and the signing key's custody into it, plus the signatory read; app depends only on org-io. No behaviour change. Resolves PR-b795an (writer coverage) | org-io, org-node, app | S0 |
| S3 | Commit workflow: persist provisional → write → read and verify → promote, keeping the delta; reconcile with the chain at startup; delete only on a verified revocation; revoked devices get only an absence proof and answer with a signed acknowledgement. Resolves PR-vt244s, PR-qmvj83 and PR-924ftr. Key rotation is S3b | org-io, org-node | S2 |
| S3b | The X25519 Organisation key pair, rotated on every device revocation. Resolves PR-xwek5e ("rotate"), takes in PR-szkat6 and HAZ-ep6uzs | org-io, org-node | S3 |
| S4 | iroh, the invitation exchange, and storage file IO into org-io; org-node becomes IO-free. Re-targets PR-kwwap5. Taken over from the `improved-type-safety` session on 2026-10-06 | org-io, org-node, app | S2; runs alongside S3 |
| S5 | Revoked-device list (CRDT, pairs), answering revoked devices with absence proofs, the never-again rule for Device keys | org-io, org-node | S1, S3, S4 |
| S6 | Pull queries between devices in good standing (delta, or full trie) | org-io, org-node | S3, S4 |
| S7 | Gossip spreading of updates; new SOUP | org-io, app | S6 |

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
   PR-szkat6 and HAZ-ep6uzs's residual.
7. **PR-924ftr goes to S3, and PR-b795an to S2.**

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

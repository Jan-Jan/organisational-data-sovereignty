# ADR: org-io, the one unit that does IO for an Organisation

- **Date:** 2026-10-06
- **Status:** Accepted (2026-10-08). Change `worktree-org-io-create` (stage
  S2 of `docs/plans/2026-10-06-org-io-roadmap.md`) implements it; set at
  that change's review round 1 (finding-14), which found the ADR still
  "Proposed" while its implementing change was going to merge. History: it
  was Proposed (draft, change `worktree-org-io-create`). Revised 2026-10-06 to the
  owner's rulings on the S2–S4 design drafts (the roadmap's section of that
  name): ruling A (moved items keep their IDs), ruling B (values in, values
  out) and the recommendations accepted with them. Revised 2026-10-07 to two
  further owner rulings on S2: the chain read stays coupled to the
  development seed in S2 and is split from the write in S8; the own-admin
  check is implemented in org-io in S2 but not wired into the app, and its
  first caller is S3b-io (stage S3's org-io half). Checked 2026-10-08
  (task T11) against the plan's T6–T9 text: one sentence corrected (the
  preflight, under "What moves in S2"); the rest matches. Re-checked
  2026-10-08 (task T12) against the merged code of S2's branch: two notes
  added (the units in "Relates to", and the re-export under "What the app
  sees"); every other statement matches the code.
- **Relates to:** REQ-nfr3n2, LLR-qhjp6g, LLR-be3zv9, RC-wzb48r (app);
  SDD-z85ux9, SDD-pa6p7w, SDD-ueh4tm, LLR-65py3d, LLR-rm9x4z, LLR-mmdu38,
  REQ-ysyu9g (org-node); REQ-6jefu2, REQ-aat4yt, SDD-yg7n55, PR-b795an
  (on-chain-client). *Note 2026-10-08 (task T12):* the unit labels above are
  where each item was defined when this ADR was written. S2 moved six of them
  to org-io with their IDs (ruling A), so they are org-io's now:
  REQ-nfr3n2, LLR-qhjp6g and LLR-be3zv9 (from the app), and SDD-z85ux9,
  LLR-rm9x4z and REQ-ysyu9g (from org-node), all in org-io's ledgers.
  RC-wzb48r stays the app's; SDD-pa6p7w, SDD-ueh4tm and LLR-mmdu38 stay
  org-node's, amended in place; LLR-65py3d stays in org-node's ledger,
  superseded by LLR-mn5c2q. No ADR ID is minted. The existing ADRs in `docs/adr/`
  carry none; one is minted with `.guardrails/scripts/new-id.sh ADR` if the
  owner wants `(ADR-<token>)` citations.
- **Decision maker:** Jan-Jan (project owner). The rulings below were given
  on 2026-10-06. This ADR records them and the structure that follows from
  them.

A new unit, `org-io`, owns all IO for an Organisation. That covers the chain
(reading, writing and the signatory check, all through on-chain-client),
peer communication (iroh point-to-point and gossip), and local storage, which
holds opaque bytes only. org-io also runs the workflow that joins these
together. org-node becomes free of IO: it takes values and returns values. It
builds, serialises, deserialises, encrypts and verifies, and nothing else. It
has no IO traits and no async. The app keeps the UI and the decision of *when*
things happen, and depends on org-io alone. Stage S2 creates the unit and
moves the chain IO and the custody of the user's signatory key into it, with
no change in behaviour. Stage S4 moves the transport, the invitation exchange
and storage file IO.

**Two keys, kept apart** (owner, 2026-10-06). The *Organisation key pair* is
Organisation-wide: the X25519 pair from the key-pair change (merged as
`1f52c36`; `org-node/docs/requirements/2026-10-07-org-key-pair.md`), held in
org-node's Organisation record, for encryption and key agreement only. It never signs and is outside this
ADR's custody decision. There is *no Organisation-wide signing key*. On-chain
authority comes from each admin user's own sr25519 account, one signatory of
the multisig that controls the Organisation's pure proxy. A device is an
admin's device when its user's account is one of those signatories. That
*signatory key* is the one org-io holds.

## Context

After the chain-authority change (`fdf4e77`), IO is spread across three
places:

- **org-node** reads the chain. `SubxtChainOps` and `connect_chain_client` are
  in `org-node/src/service.rs`, and `OnChainReader` and `org_state_from_chain`
  in `org-node/src/chain_read.rs`. `OrgService` holds a `Box<dyn ChainOps>`
  and awaits `read_state` inside `commit_genesis`, `commit_update` and the
  receive path. It also runs the operator preflight
  (`org-node/src/preflight.rs`, `org-node/src/bin/preflight.rs`). It runs the
  iroh transport (`org-node/src/transport/`) and writes the encrypted Persona
  store to disk (`org-node/src/store.rs`). The chain code sits behind the
  `chain` Cargo feature, which brings in `subxt`, `on-chain-client`, `tokio`,
  `async-trait` and `hex`.
- **The app** makes the chain write. `ChainWriter`, `OnChainWriter`,
  `found_organisation` and `submit_commit_send` are in
  `app/src-tauri/src/submit.rs`. The app also holds the user's sr25519 signatory key:
  `build_chain_ops` and `connect_chain` in `app/src-tauri/src/state.rs` parse
  `ODS_ADMIN_SEED` into a `subxt_signer::sr25519::Keypair` and keep it in
  `OnChainWriter.signatory` for the life of the process. The app depends on
  org-node, on on-chain-client (with its `write` feature), on subxt and on
  subxt-signer.
- **on-chain-client** is the chain library. It has no consumer that it is
  written for: the app calls its writer and org-node calls its reader.

The roadmap's unit table puts all of this in org-io. The publish workflow in
S3 (persist provisional, write, read and verify, promote) has to sequence a
chain write, a chain read, a store write and a send. No unit owns that
sequence today, so it would otherwise end up split across the app and
org-node.

Neither the app nor org-node reads anything about the Organisation's multisig.
The "advisory signatory read" was planned for org-node
(`org-node/docs/requirements/2026-10-06-chain-authority.md:89-91`, sequencing
item 4), and the owner has since ruled that admin status is checked on-chain by
org-io.

## Options considered

1. **A new crate and unit, org-io, that owns the node.** org-io holds
   org-node's `OrgService` together with its own chain reader, chain writer and
   the user's signatory key. The app calls org-io and nothing else. *Decided.*
2. **IO kept in org-node, behind its Cargo features.** No new unit. This
   contradicts the ruling that org-node is IO-free. It would also keep the
   class C verification evidence for the pure logic in the same unit as the
   untested subxt shell (SDD-z85ux9 and its "no low-level requirements"
   deviation). Rejected.
3. **org-io as an adapter library that the app composes.** org-io provides
   the chain reader, the writer and the key, and the app still builds
   `OrgService` and joins the pieces together. This contradicts the ruling
   that the app depends only on org-io, and it leaves the S3 workflow in the
   app. Rejected.

How org-io meets org-node was a second choice:

- **(i) An IO trait in org-node that org-io implements.** org-node keeps
  `ChainOps` (async, a trait object), and org-io injects a subxt
  implementation. The earlier draft of this ADR chose it. org-node would keep
  `async-trait` and an awaiting `OrgService`, and every later IO move (S4's
  transport and store) would add another trait. **Rejected by ruling B.**
- **(ii) Values in, values out.** org-node's operations take the values they
  judge against and return the values to be written. org-io does every read
  and every write. *Decided (ruling B).*

## Decision

- **org-io is a new Cargo crate at `org-io/`** and a guardrails unit of safety
  class C. It has its own ledgers under `org-io/docs/` and its own
  `.guardrails/config.yaml`, and it is listed in `.guardrails/units.yaml`. The
  guardrails skeleton (manifest entry, config, ledger READMEs, SOUP file) is
  created in S2's design phase. The crate lands in S2's develop phase, after
  the key-pair change merges (it did, as `1f52c36`, on 2026-10-07).
- **Values in, values out (ruling B).** org-node has no IO traits and no
  async on the paths org-io drives:
  - `ChainOps`, `MockChainOps`, `ChainOpsReader` and `SubxtChainOps` leave
    org-node. `OrgService` no longer holds a chain. Each operation that judges
    against the chain (`commit_genesis`, `commit_update`, verifying a received
    message, the revocation check) takes the Organisation's state as an
    argument, an `Option<OrgState>` that org-io has just read and parsed, and
    is synchronous where it does no transport.
  - org-node keeps `OrgState::from_chain` as its parse edge (REQ-8jb4ny).
    org-io reads on-chain-client's state and passes its bytes through that
    parse, so an invalid Organisation public key is still refused by
    org-node's rule.
  - org-node's synchronous `ChainReader` view, used by `verify.rs`, is
    replaced by the same value: verification takes the state it compares
    against. The in-memory `MockChain` for tests becomes a test-support store
    of `OrgState` values with no trait.
  - Out: the values a write needs (the provisional update's root, Organisation
    public key, expected epoch and proxy account) are returned by org-node
    already, and org-io submits them. From S4, org-node also returns the
    sealed store bytes for org-io to write.
  - The transport stays async inside org-node until S4 moves it, so
    `send_update` and the receive wait are the async remainder in S2. The
    receive path is split in S2 so that the chain read happens in org-io
    between receiving a message and verifying it.
- **Moved items keep their IDs (ruling A).** An item whose subject moves into
  org-io keeps its ID. Its definition moves to org-io's ledger with a dated
  note stating where it came from, so existing `verifies:` lines keep
  resolving. The source ledger keeps a dated pointer in prose that names
  org-io's ledger file, not the ID: org-node does not depend on org-io, and
  the app may cite only what org-io exports, so a bare ID there would be
  UNDECLARED-DEPENDENCY or NON-EXPORTED-REF. An item that only partly moves is amended in place
  for the part that stays, and the part that moves becomes an org-io item. An
  item whose subject disappears (for example an IO trait under ruling B) is
  amended in place to state what replaces it. This replaces the
  chain-authority precedent, which restated the source item as an absence and
  minted a new ID. The definitions move in S2's develop phase, with the code.
- **Its dependency edges** are org-io → org-node, org-io → on-chain-client
  (with its `write` feature), and org-io → person. The org-node and
  on-chain-client edges are declared with the skeleton; the person edge is
  declared when org-io's code first names a person type, which may not happen
  until S4. The app's edges become app → org-io only: the app's `depends_on:`
  drops org-node and on-chain-client, and its `Cargo.toml` drops `org-node`,
  `on-chain-client`, `subxt` and `subxt-signer`. org-node's edge to
  on-chain-client is removed in S2, once org-node no longer calls it. The
  dependency assessment for each edge is in the S2 plan.
- **What the app sees.** org-io re-exports the org-node types the app's
  commands use, in one module marked transitional. S4 narrows them as the
  transport and the invitation commands move. *Note 2026-10-08 (task T12):*
  in the code the re-export is the whole crate, `pub use org_node as node;`
  in `org-io/src/lib.rs`, marked transitional there, rather than a list of
  the types the commands use. Narrowing it to such a list is S4's work.
- **Every unit stays class C.** All five are C, so `MISCLASSED-DEPENDENCY`
  has nothing to flag, and no `segregated_from:` is written.
- **What moves in S2** (chain IO and key custody, with no behaviour change):
  - The chain read. `SubxtChainOps`'s read and `connect_chain_client` move
    from org-node's `service.rs`, and `org_state_from_chain` from
    `chain_read.rs`. `OnChainReader` is deleted, and so is `OrgStateCache` if
    no org-io path caches. The operator preflight's chain checks
    (`preflight.rs`, `bin/preflight.rs`) move too, so org-node's `chain`
    feature and its `subxt`, `on-chain-client`, `hex` and `async-trait`
    dependencies go. *Revised 2026-10-08 (S2's T5 and T6):* this read "The
    operator preflight (`preflight.rs`, `bin/preflight.rs`) moves too";
    only its chain checks move. org-node keeps a transport-only preflight,
    gated on its `transport` feature, until S4 moves the transport.
  - The chain write. `ChainWriter`, `OnChainWriter`, `WriterNotConfigured`,
    `found_organisation`, `submit_commit_send` and the 90-second bound move
    from the app's `submit.rs`.
  - The custody of the user's signatory key (not the Organisation's X25519
    key pair, which stays in org-node's record and never signs). org-io
    reads `ODS_ADMIN_SEED` from the environment itself, in one function, and
    parses it with a total parser into the sr25519 key that only org-io
    holds. The co-signer list (`ODS_COSIGNER_PUB`) is parsed there too. The
    seed path is **development-only** (owner, 2026-10-06): it is compiled
    only under org-io's `dev-seed` Cargo feature, which no release build
    enables. Production custody, where the user is prompted to keep the key
    in the OS keychain or uses a hardware signing device and the key is never
    stored in plaintext, is a later roadmap stage. The app keeps reading the
    non-secret configuration. After S2 the app never holds a signatory key.
    The chain read is built together with the write, as today, so it too
    needs the development seed: a release build reads no chain until S8.
    Owner ruling of 2026-10-07: keep the read coupled to the seed in S2, and
    split it from the write in S8 (production custody).
  - The own-admin check. A new read: "is this node's chain account a
    signatory of the multisig that controls this Organisation's pure proxy?"
    org-io reads `Revive.OriginalAccount` for the Organisation's H160 to get
    the proxy account, reads `Proxy.Proxies` for that account, and answers yes
    when a delegate equals the controller derived from this node's own
    account and its configured co-signers (`multi_account_id` of the sorted
    signatories, threshold 1, or the account itself with no co-signers). The
    two storage reads are new on-chain-client reader functions, exported for
    org-io. The chain can confirm a signatory set but not list it, so S2
    checks only the node's own status. RC-wzb48r stays information for safety.
    Owner ruling of 2026-10-07: S2 implements the check in org-io
    (`OrgIo::is_own_admin`, with its tests) and does not wire it into the
    app; its first caller is S3b-io (stage S3's org-io half).
  - Coverage of the chain writer: PR-b795an's fix (the `write` feature and its
    four targets measured by `coverage-on-chain-client`) lands with S2,
    because org-io becomes the writer's consumer.
  - org-io has statement and branch coverage floors from its first merge.
- **What moves in S4:** the iroh transport (`org-node/src/transport/`), the
  invitation exchange (`app/src-tauri/src/invitation.rs`), the receiver loop,
  sending each update to every Device (PR-3ue4va), storage file IO (org-node's
  `PersonaStore` opening and writing files becomes org-io's, holding opaque
  encrypted bytes that org-node seals and opens), and PR-kwwap5. After S4,
  org-node has no `tokio`, no `iroh`, no async and no file IO.
- **The app** keeps the UI, the decision of when to publish, read the chain or
  ask peers, and the reading of non-secret configuration. It reports what
  org-io returns.

## Consequences

- **Easier.** S3's workflow has one owner, and every IO step of it is visible
  in org-io's code rather than behind a trait call. The subxt shell that no
  gate tests (org-node's SDD-z85ux9, part of the app's SDD-6g3wnh) moves into
  one unit, where its deviation is argued once. org-node's pure logic is
  tested on values without a mock chain. The app loses its whole chain SOUP
  surface (`subxt`, `subxt-signer`, `on-chain-client`).
- **Harder.** One more unit to ratchet: ledgers, SOUP inventory, coverage
  commands, and a dependency assessment for each edge. org-node has exported
  nothing so far, so the org-io → org-node edge needs org-node to export the
  requirements org-io relies on. Otherwise every reference org-io makes is a
  NON-EXPORTED-REF. `OrgService`'s chain-judging operations change signature,
  so every org-node test that drives them through `MockChainOps` is
  rewritten to pass the state value.
- **Items change home with their IDs.** Tests move with the behaviour they
  verify (for example `app/src-tauri/tests/submit_flow.rs` becomes org-io's)
  and their `verifies:` lines are unchanged. Every mention of a moved ID left
  in the source unit (ledgers, source comments, tests that stay) has to be
  rewritten to prose or removed, or the source unit's run fails.
- **Sequencing.** The key-pair change (`worktree-org-node-org-key-pair`)
  edited `submit.rs`, `commands.rs`, `invitation.rs`, `events.rs` and
  org-node's `service.rs`, so S2's code waited for it; it merged as `1f52c36`
  on 2026-10-07 and is merged into S2's branch. The branch-coverage change
  (`worktree-guardrails-branch-coverage`) must merge before S2's develop
  phase starts (the plan's task T0), so org-io's branch recipe reuses its
  pattern.
- **Reversibility.** Merging org-io back into org-node or the app later would
  undo the IO-free rule. That is the reason for recording this as an ADR.

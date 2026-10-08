# AGENTS.md (org-io)

Guidance specific to the `org-io` unit. See `../AGENTS.md` for project-wide
guidance.

- **org-io does all IO for an Organisation**: the chain (read, write, the
  own-admin check, through on-chain-client), and from stage S4 the transport
  and storage file IO. It runs the workflow that joins them.
- **org-node is IO-free: values in, values out.** org-io reads and parses,
  then hands org-node values; org-node returns values (and, from S4, sealed
  store bytes) for org-io to write. Never add an IO trait or async to
  org-node for org-io's sake.
- **The app depends on org-io only.** Re-exports of org-node types for the
  app are transitional, in one module; S4 narrows them.
- **Two keys, never confused.** The Organisation key pair is X25519,
  Organisation-wide, held in org-node's record, for encryption only; it never
  signs. There is no Organisation-wide signing key. org-io holds the user's
  own sr25519 signatory key (one signatory of the proxy's multisig).
- **The signatory key never leaves org-io**: no getter, redacted `Debug`,
  never in an error or a log. `ODS_ADMIN_SEED` is development-only, read only
  under the `dev-seed` feature; production custody (OS keychain or hardware
  signer) is roadmap stage S8.
- **Status (2026-10-08):** S2 built the crate: the chain read and write, key
  custody, the own-admin check, the handle the app uses. Plan:
  `../docs/plans/2026-10-06-org-io-create.md`; roadmap:
  `../docs/plans/2026-10-06-org-io-roadmap.md`.
- Type safety is the project's hard rule: parse at the system's edge
  (`../docs/adr/2026-10-04-parse-at-the-system-edge.md`). org-io's edges are
  the environment, the chain and, from S4, the network and the disk.

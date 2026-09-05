# ADR: four compliance units, each IEC 62304 class C

- **Date:** 2026-09-05
- **Status:** accepted
- **Relates to:** `docs/adr/2026-09-01-safety-class-c.md` (the pathway to
  serious injury this decision inherits; not superseded)
- **Decision maker:** Jan-Jan (project owner), interviewed by `/ratchet`
- **Recorded in:** `.guardrails/units.yaml`; `safety_class: C` in each of
  `org-members/`, `on-chain-client/`, `org-node/`, `app/` `.guardrails/config.yaml`

## Context

Guardrails assumed one compliance unit per repository. This repository holds
several things that version, release and take risk separately: a `no_std`
membership library (`org-members`) intended for consumers outside this tree, a
chain-reading client crate (`on-chain-client`), the headless node that holds
keys and makes trust decisions (`org-node`), a Tauri desktop shell (`app`), a
Foundry contract package (`on-chain`), formal models (`quint`) and three
qualification spikes. One root config gave all of them one class, one ledger
set and one problem budget, and could not express that `org-node` consumes
`org-members`' API.

Upstream guardrails `e2eac86` adds multi-unit support: a root manifest
enumerates units; each unit is a full guardrails project inside its own
directory; scans are scoped per unit; cross-unit references are legal only
across a declared `depends_on:` edge and only to `exported: yes` requirements;
a provider's class may not sit below its consumers' without a recorded
segregation argument (D1–D14 in the guardrails plan
`docs/plans/2026-08-26-monorepo-support.md`).

## Decision

1. **The repository is many systems, not one.** A manifest is installed and
   the root config removed.
2. **Four units:** `org-members`, `on-chain-client`, `org-node`, `app`. The
   recommendation was three; the owner added `app`, because the shipped
   product should be under discipline from the first tooth rather than
   disclaimed as "pending".
3. **Seven disclaimed directories**, each with its reason beside it in the
   manifest: `on-chain`, `quint`, `spike-common`, `spike-keyhive`,
   `spike-p2panda`, `docs`, `.github`.
4. **Every unit is class C.** Rationale per unit:
   - `org-members` — the membership authority; every access decision is
     derived from its record. A member retaining access they should have lost
     is the S3 pathway of the 2026-09-01 ADR, and it begins here.
   - `on-chain-client` — decides what the chain says the membership is
     (decoding org state and revive events). A wrong or stale reading is
     indistinguishable, to its consumers, from a wrong membership.
   - `org-node` — holds the keys, makes the trust decisions, performs the chain
     and p2p I/O through which access is actually granted or withdrawn. The
     hazard register already names it eight times.
   - `app` — the surface through which an administrator admits, verifies and
     revokes. It is deliberately thin, but it is where intent becomes a
     membership change, and a mis-wired command or wrong display reaches the
     same pathway.
   With every unit at C the class-floor gate (D5) has nothing to convict and no
   segregation argument is needed. Should a unit ever be argued down to B, the
   argument is a `segregated_from:` entry citing an RC or ADR, per edge.
5. **No dependency edge is declared in this change.** The four edges the code
   has (org-node → org-members, on-chain-client; app → org-node,
   on-chain-client) are each declared in a later change, after the dependency
   assessment `grill-requirements` prescribes. Until then each unit is a
   freestanding guardrails project that shares a repository — and the manifest
   says so.

## Alternatives considered

- **Stay single-unit and call "monorepo" a packaging fact.** Zero migration,
  and the scripts upgrade alone. Rejected: `org-members` is meant to be
  consumed outside this tree, and one class and one ledger set for a library,
  a node and a UI is the shape D3 rejects (the highest class governs all code,
  and a consumer can trace to a provider's internals unnoticed).
- **Two units (`org-members`, `on-chain-client`) with everything else
  disclaimed.** The minimal valid manifest. Rejected by the owner: org-node
  owns an open problem report and most of the hazard register's mentions, and
  the app is the product.
- **Five units, adding `on-chain`.** Deferred: Foundry is a separate toolchain
  and nobody in the session could run `forge test` to set an honest
  `verify_commands`. It is tooth 7 of the new gap analysis, with a dated
  disclaimer meanwhile.
- **Declare the four edges now.** Rejected for this change: each edge is a
  decision with an interview behind it (exports, RMF, ADRs, open problems,
  SOUP), and org-members has not yet decided what it exports. Declaring edges
  without the assessment would be config lines no gate can yet read
  meaningfully.
- **Class B for `on-chain-client`.** Considered and offered with its cost: a
  segregation argument showing a wrong chain read cannot grant access the
  membership record denies. Rejected: no such argument exists today, and the
  hazard register's two mentions of the crate sit on the S3 pathway.

## Consequences

- **The ledgers moved.** `docs/requirements`, `docs/risk`, `docs/problems`
  and `docs/architecture` are now `org-members/docs/…`, with history. IDs and
  item text are unchanged. Historical records that cite the old paths are
  left as written.
- **There is no root config.** A bare `check-trace.sh`, `check-ids.sh` or
  `new-id.sh` at the root is exit 2; each takes `GR_CONFIG=<unit>/.guardrails/
  config.yaml` (or, for `new-id.sh`, `--unit <path>` or a cwd inside the unit).
  `check-units.sh` is the repository-level entry point. CI reflects this.
- **The verification record stays repository-level**, one per change, and now
  names the units touched and the impact set (`merge-change`, D6).
- **`merge-change` computes which units run** from `check-units.sh --impact`.
  With no edges declared, a change to `org-members` runs org-members' gates and
  suite only — org-node's do not run until the edge exists. That is the same
  coverage the root config gave (org-node's tests never ran at merge), stated
  in the manifest instead of assumed.
- **Hazards about org-node live in org-members' RMF** until org-node's own
  risk analysis (tooth 3). Measured on this change: moving even one problem
  report across the boundary without an edge drew `UNDECLARED-DEPENDENCY` in
  both units. The boundary is real from the first tooth.
- **Hard to reverse.** The manifest and scope rules shape every config and
  every ledger path; going back to single-unit means deleting the manifest and
  merging four ledger sets. That, the fact that the scripts *look* as if they
  would work per-directory without any of this, and the real trade-off between
  per-unit autonomy and checked coupling are why this is an ADR.

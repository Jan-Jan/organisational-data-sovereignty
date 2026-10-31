**ADR-bbgfn2**: An Admin's admissions and revocations are provisional changes, calculated once per epoch into one provisional organisation with one fresh Organisation key pair, and the changes an adopted epoch left out are discarded.

- **Date:** 2026-10-15
- **Status:** Accepted (owner, 2026-10-14/15, `grill-requirements` interview
  of change `worktree-org-io-domain-api`).
- **Units:** org-node, org-io, app.

Until now each admission or revocation was its own complete provisional
update — Change set, Membership root and its own Organisation key pair — so
two pending changes excluded each other and the second was dropped at
commit (the chain-authority and key-pair changes). The demo journey has an
Admin make several changes and finalise them together, so a change is now an
intent against the current record, and the root and key pair are calculated
once, when the Admin proposes the updated Organisation (org-node's
batched-epoch requirements). The trade-off: one chain write and one key
rotation per epoch instead of per change, at the price of a stale
provisional organisation whenever a change is added after it was proposed,
and of an Admin re-proposing changes another Admin's epoch left out.
Handling Admins who disagree is future scope.

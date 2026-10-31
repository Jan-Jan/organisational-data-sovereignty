**ADR-be7uph**: Writing an Organisation to the chain, verifying it against the chain, and sending it to Devices are three steps the app calls separately; the chain write changes no local state except a founding's proxy address, and only verification adopts.

- **Date:** 2026-10-15
- **Status:** Accepted (owner, 2026-10-14/15, `grill-requirements` interview
  of change `worktree-org-io-domain-api`).
- **Units:** org-io, org-node, app.

S2 and S3b-io built one submission flow — settle, write, read back, commit,
send — so the commit sat inside the chain write and a delivery could only be
retried by settling a late write. Splitting it gives one verification path
for every case (own write, unknown outcome, start-up, another Admin's epoch,
a received provisional organisation), and lets the app retry a send without
touching the store. The trade-off: the app must call the three steps in
order, and a founding needs the single exception — its proxy address is
stored as soon as the proxy exists, because the Organisation identifier is
known only then and verification cannot find the Organisation without it.

# Problem reports — the chain reader documents the wrong finality

**Moved 2026-09-09.** The one report this file defined — that `OnChainReader`
in `org-node/src/chain_read.rs` documents a best-block read while the client
it calls performs a finalised one — is an org-node defect, and under
guardrails D14 a provider's defect is filed in the provider's ledger. It was
filed here on 2026-09-02 because this was the only ledger; it now lives, with
its original text and opened date and still open, in
`org-node/docs/problems/2026-09-09-org-node-problems.md` (the dated name
`finalize-docs.sh` gives that change's draft file at merge). Its `affects:`
line was retargeted at the same time from this unit's stale-record control to
org-node's own independent-root control, because a consumer may not cite a
provider's controls. This file is kept so that the ledger's history reads
chronologically; it defines no item. (Corrected 2026-09-10: an earlier
wording also claimed the 2026-09-02 verification record's reference to this
file still resolves through it. That record
(`docs/verification/2026-09-02-worktree-risk-membership.md`) names the report
by its identifier, never by this file, so the claim supported nothing. The chronology is the whole of the reason.)

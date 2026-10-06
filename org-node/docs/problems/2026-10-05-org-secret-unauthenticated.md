# Problem reports — org-node, the Organisation secret is not authenticated

Opened 2026-10-05 by review round 1 of `worktree-worktree-person-shared-types`
(finding-4), on the owner's ruling of the same day.

**PR-ve9zw8**: the Organisation secret a node stores (`WireMessage.org_secret`)
is authenticated by nothing. The Envelope never covered it, and the Wire
message has no signature of its own. A node takes whatever secret the Wire
message carries, from any sender: nothing about the sender is checked
(REQ-xa6smf, REQ-ztdza4). Any peer that relays a genuine admission or update
can therefore give a member a secret of its choosing, and the chain cannot
catch it, because the secret is not part of what is checked against the
chain.
affects: SDD-8cpyfa, LLR-ckk5nz, RC-b6mydy
opened: 2026-10-05
status: resolved
resolution: the Organisation secret is replaced by the Organisation private key, which a receiver stores only if its X25519 public half is the chain's `org_pub_key` (RC-9cefcn, REQ-bwx7eg, LLR-ba2ejp); reproduced by `pr_ve9zw8_a_first_admission_carrying_a_substituted_key_is_refused` and `pr_ve9zw8_an_update_carrying_a_substituted_key_is_refused_on_both_paths` (org-node/tests/admission_sender.rs), red before (the substituted key was committed), green after.

## What was observed

`receive_and_verify` writes `msg.org_secret` into the record it commits, on
first admission and on every update (`org-node/src/service.rs`, the commit
block). Only the Change set is verified against the chain. `org_secret` sits
in the Wire message beside the Envelope, so the signature the Envelope carried
until 2026-10-05 never covered it either: any relay on the path could swap it
before. What the unsigned Envelope and the owner's ruling that nothing about the
sender is checked add is the wider set of senders: any peer now reaches this
write, not only a relay between the administrator and the receiver.

## Why it matters

A member that holds a secret chosen by another peer encrypts or decrypts
under a key that peer knows. That is HAZ-ep6uzs's exposure, reached through
any peer rather than a leak. The assessment is in
`org-node/docs/risk/2026-10-05-envelope-authenticity.md` ("The Organisation
secret") and the dated note on LLR-ckk5nz in
`org-node/docs/risk/2026-10-03-architecture-derived.md`.

## What closing it looks like

The owner ruled (2026-10-05) that the Organisation secret is to be replaced by
CGKA keys that a member verifies against the Organisation public key the chain
records. That replacement closes this report. Until then no interim control
is minted: an interim check would authenticate a value that is being removed.
PR-xwek5e, the overwrite with nothing, belongs to the same replacement.

*Noted 2026-10-05.* The owner's chain-authority rulings make the
Organisation secret the Organisation private key, refused on receipt unless
its X25519 public half equals the on-chain `org_pub_key`. That is
chain-authority's change 2, and it closes this report. Whether that
supersedes the CGKA wording above was put to the owner (Q2 of
docs/plans/2026-10-05-switch-trim.md) and not answered; the risk file names
both, unranked.

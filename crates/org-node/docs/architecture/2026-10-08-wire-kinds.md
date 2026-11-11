# Design — the Wire message of three kinds, under its own ID

Low-level requirement for change `worktree-org-io-create` (stage S2), written under the owner's ruling
of 2026-10-08 on the resolved problem report on amendment in place instead of supersession (`crates/org-node/docs/problems/2026-10-07-close-out.md`):
An earlier form of LLR-xr88a8 (`2026-10-07-org-key-pair.md`) changed meaning in S3a, from two
message kinds to three (update, revocation notice, acknowledgement), and its
revocation payload changed from a members trie carried in an Envelope to an
absence proof. Under the hybrid amendment rule a change of meaning mints a
new ID that supersedes the old. The item below sits under SDD-kwncn7 (the
wire frame and its bound, `crates/org-node/src/transport/wire.rs`), as that earlier form
did, and states the rule that holds now.

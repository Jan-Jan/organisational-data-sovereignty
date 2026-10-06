# Problem reports — Envelope signatures are checked with the non-strict verify

**PR-vkw22m**: org-node checks every Envelope signature with
`ed25519-dalek`'s non-strict `verify` (`org-node/src/keys.rs`), which, unlike
`verify_strict`, accepts a signature under a small-order (weak) public key and
a signature whose `R` is a small-order point, so a signature can verify for
more than one message and its signer can later deny which one they signed.
affects: REQ-ag6kqm, RC-pm9kmx
opened: 2026-10-04
status: resolved
resolution: root cause — `keys::verify` checked Envelope signatures with ed25519-dalek's non-strict `VerifyingKey::verify`; fixed by change `worktree-person-shared-types`, which removes the Envelope signature (REQ-ag6kqm, LLR-na7p4w and LLR-9fvb3y amended in place), so org-node checks no signature and `keys::verify` is gone; test `the_wire_form_has_no_signature_field` (`org-node/tests/envelope_binding.rs`).

Found 2026-10-04 in the `grill-requirements` interview for the org-node
type-safety change, first recorded as "weak and non-canonical public keys are
accepted". That key-acceptance half is the same anomaly as the problem
report in org-members' ledger `2026-10-04-ed25519-small-order.md`
(small-order, non-canonical and mixed-order keys accepted), which reached
master first and is resolved by the change that
switches org-members to the `person` unit's key types. By owner ruling
(2026-10-05) this report was narrowed to the part that report does not cover —
the signature check — and moved to org-node's ledger, where that check is;
it had not been merged, so no definition moved on the base branch. The
type-safety change's key constructors accept exactly what deserialisation
accepted before (owner ruling 2026-10-04). Moving to `verify_strict` needs its
own requirement and risk analysis.

(Amended 2026-10-05 after independent review round 5: this report first said
the non-strict check also accepts a non-canonical signature encoding. In
`ed25519-dalek` 2.2.0 as built here (no `legacy_compatibility`), both checks
refuse a non-canonical `s` and a non-canonical `R`; the difference is the
small-order public key and the small-order `R` only.)

**Resolved 2026-10-05 by change `worktree-person-shared-types`.** The report is
moot rather than fixed in place: there is no signature left to check strictly.
The soup.md row for `ed25519-dalek` records that no signature path of the
library is reached from org-node. If a later change signs anything again, the
choice between `verify` and `verify_strict` comes back with it and needs its
own requirement.

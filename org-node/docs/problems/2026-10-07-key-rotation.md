# Problems — the Organisation key pair is never rotated

**PR-g9u3xq**: removing a Member does not replace the Organisation key pair,
so a removed or lost Device keeps the Organisation private key every remaining
Member still uses, for as long as the Organisation exists.
affects: HAZ-vxabf9, REQ-szq3ud, REQ-ech45n
opened: 2026-10-06
status: resolved
resolution: a fresh key pair is drawn with every provisional update's root
(REQ-stx9v3) and replaces the record's key on commit (REQ-jy6ybw); only the
Devices of the new record receive it (REQ-3dsweu). Owner ruling 2026-10-06.
Reproduced by `pr_g9u3xq_a_removed_device_holds_no_key_used_after_its_removal`
(org-node/tests/admission_sender.rs): red before, green after.

Recorded 2026-10-06 by the risk analysis of change
`worktree-org-node-org-key-pair`, which gives the Organisation private key to
every Member's Devices. Rotation (a fresh key pair on removal, its public half
written to the chain, its secret sent to the remaining Members) was ruled out
of scope on 2026-10-05. Later on 2026-10-06 the owner ruled that a fresh key
pair is part of calculating every new Membership root, which brings rotation
into this change; the item is resolved by it.

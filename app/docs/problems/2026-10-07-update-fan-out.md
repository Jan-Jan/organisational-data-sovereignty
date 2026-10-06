# Problems — a committed update reaches one Device

**PR-3ue4va**: the app sends each committed update to one Device only: an
admission to the joiner's Device (`app/src-tauri/src/invitation.rs`,
`admit_reply`) and a revocation to the revoked Member's first Device
(`app/src-tauri/src/commands.rs`, the revoke command); no other Member's
Device receives the update or its new Organisation private key, and a revoked
Member's other Devices receive no revocation.
affects: REQ-nfr3n2, LLR-q225ws
opened: 2026-10-06
status: open

Recorded 2026-10-06 by the design of change `worktree-org-node-org-key-pair`.
The owner's model: a change in Organisation information goes to every Device
of every new and current Member, and a revocation, with its proof, goes only
to the revoked Devices, every Device of a revoked Member. With the key pair
rotated at every update, a Member that receives nothing stays on an earlier
epoch's key. By owner ruling (2026-10-06) the fan-out is the `org-io`
session's work, which moves peer communication into the app; org-node already
chooses the right kind for each recipient.

# The epoch rule and the Organisation public key

The on-chain `orgPubKey` is the Organisation public key, an X25519
key-agreement key (root `docs/CONTEXT.md`), and the Envelope carries no
signature (owner, 2026-10-05). The reasons and the risk assessment are in
`org-node/docs/risk/2026-10-05-envelope-authenticity.md`.

**REQ-txvtm9**: The software shall commit a received Envelope only when its
Sequence number equals the epoch of the Organisation state on the chain it is
verified against, and shall otherwise refuse it, leaving its record unchanged.
(implements: RC-95dgg8)
satisfies: derived

*Amended 2026-10-05 (docs/plans/2026-10-05-switch-trim.md).* This item first
restated REQ-nhe2zu's commit conditions with a sender clause and the epoch
rule, and superseded it. By owner ruling REQ-nhe2zu is amended in place
instead, and this item states only the rule REQ-nhe2zu lacks.

**REQ-8jb4ny**: The software shall refuse an Organisation state read from the
chain whose Organisation public key is not a valid X25519 public key, as
`person`'s exported check decides, with a typed error, and shall act on no
Envelope verified against it.
satisfies: derived

**REQ-ech45n**: The software shall, when it creates an Organisation, publish
as its Organisation public key the X25519 public key of a freshly generated
secret, distinct from every Member-as-a-group key and DevicePublicKey in the
genesis record, and keep that secret in its encrypted store.
satisfies: derived

# The epoch rule and the Organisation public key

The on-chain `orgPubKey` is the Organisation public key, an X25519
key-agreement key (root `docs/CONTEXT.md`), and the Envelope carries no
signature (owner, 2026-10-05). The reasons and the risk assessment are in
`crates/org-node/docs/risk/2026-10-05-envelope-authenticity.md`.

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

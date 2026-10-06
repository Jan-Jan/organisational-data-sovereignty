# The Organisation public key, by its name

The owner ruled (2026-10-04, recorded in
`docs/plans/2026-10-04-person-shared-types.md`) that the on-chain `orgPubKey`
is the Organisation's X25519 key, published so that members can check the
Organisation private key shared with them, and that it never was a signing key.
The root `docs/CONTEXT.md` defines *Organisation public key* for it. This item
restates REQ-2qa5r5 with that term; the widths, and so the behaviour, are
unchanged. The software keeps the field as opaque bytes and never checks
that they are a valid X25519 key. org-node checks them when it reads
Organisation state (`OrgState::from_chain`, in `org-node/src/chain.rs`). The
`GenesisInitialized` and `RootUpdated` events also carry the key, and those
bytes reach their reader unchecked. Nothing reads them today; a consumer that
starts to must parse them first. This item carries
REQ-2qa5r5's abnormal-input exemption (`2026-09-10-chain-reading.md`, the
paragraph on the two requirements with no abnormal-input case) for the same
reason: it constrains a declaration and has no runtime input domain.

Terms: *Organisation slot*, *Organisation admin* and *Epoch* are defined in
`on-chain-client/docs/CONTEXT.md`; *Organisation state*, *Membership root* and
*Organisation public key* in the root `docs/CONTEXT.md`.

**REQ-54txzh**: The software shall represent each field of an Organisation
state, and the Organisation admin whose Organisation slot holds it, as a public
type whose width is the width the contract's ABI gives that field: thirty-two
bytes for a Membership root, thirty-two for the Organisation public key, eight
for an Epoch, and twenty for an Organisation admin. (implements: RC-sxjnx9)
satisfies: derived
supersedes: REQ-2qa5r5

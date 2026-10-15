# Open problems found by S4's code conformance review

Opened 2026-10-14 by the fix of S4's CODE conformance review (against
`.guardrails/templates/CODE_GUIDELINES.md`). Each would change behaviour to
fix, which a conformance fix may not do, so each is recorded here and left
open.

**PR-b8kmk2**: a reconcile that commits an unknown write reports its
recipients but not their send failures: `reconcile` sends the late commit's
bodies and returns `Reconciliation::Committed { recipients }`, whose
recipients are the Devices sent to, with no word of a send that failed.
affects: LLR-k5pzsz, REQ-by9k5q
opened: 2026-10-14
status: open

Where: `org-io/src/publish.rs`, `reconcile`, the arm that commits the
executed unknown write (`send_late_commit`'s result is not returned).
Finding-2 of the review (Errors 2). `Reconciliation` and `Settled` are
stage S3b-io's public types, and S3b-io's part 2 adds a per-Device
`SendReport` to them; until then the reconcile's caller cannot learn which
Device the late commit did not reach. `settle_unknown` already returns the
failures (`Settled::Committed { sent }`).

**PR-43t2zn**: on the submit and reconcile removal paths a locked or
refusing keychain is reported as org-node's "device secret … was not
supplied", not as the keychain's refusal: `Keychain::for_signing` leaves out
every secret the keychain refuses and drops each `KeychainError`.
affects: REQ-6d9xv6, SDD-4fvypt
opened: 2026-10-14
status: open

Where: `org-io/src/keychain.rs`, `Keychain::for_signing`, which passes
`&mut None` to `for_signing_noting_refusal`; its callers are
`LocalStore::device_secrets_for_signing` in `publish.rs` (`run_reconcile`)
and `submit.rs`. Finding-3 of the review (Errors 1). The receive path
already uses `for_signing_noting_refusal` and reports the keychain's
refusal.

**PR-vy83sd**: an endpoint identity that does not parse as a Device key is
reported as `TransportError::Malformed` ("malformed wire message"), which
names neither the identity nor that the identity, not a message, was
refused.
affects: SDD-8uyg4s, SDD-v2bq89
opened: 2026-10-14
status: open

Where: `org-io/src/transport/endpoint.rs`, `recv_answerable` (the remote
endpoint id parsed as a `DevicePublicKey`), and `org-io/src/send.rs`,
`send_committed` (a recipient's Device key parsed as an `EndpointId`).
Finding-5 of the review (Errors 1).

**PR-wayqj3**: `PeerAddr::to_hex` renders an address whose encoding fails
as the empty string, which `parse_peer_addr` reads back as "no address"
instead of a refusal.
affects: REQ-n82q47, LLR-2b2ghe
opened: 2026-10-14
status: open

Where: `org-io/src/peer_addr.rs`, `PeerAddr::to_hex`
(`postcard::to_allocvec(..).unwrap_or_default()`). Finding-6 of the review
(Errors 3). The comment there argues the encode cannot fail for an
`EndpointAddr`; the fallback still turns a failure into a different,
valid-looking value.

**PR-a63sk3**: the preflight CLI takes any value of `ODS_TRANSPORT` other
than `loopback`, a typo included, as Networked, with no refusal naming the
variable.
affects: LLR-h9tcat
opened: 2026-10-14
status: open

Where: `org-io/src/bin/preflight.rs`, `main`, the `ODS_TRANSPORT` match
(`_ => TransportMode::Networked`). Finding-8 of the review (Errors 3). Unset
is the documented default; an unrecognised value is not.

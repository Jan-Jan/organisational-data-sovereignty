# Problems — a committed update reaches one Device

*Moved 2026-10-13 (S4, task T12 of `docs/plans/2026-10-06-org-io-transport.md`,
ruling A):* the report this file held — the app sent each committed update to
one Device only (an admission to the joiner's Device, a revocation to the
revoked Member's first Device), so no other Member's Device received the
update or its new Organisation private key, and a revoked Member's other
Devices received no revocation — moved to org-io with the send, to
`org-io/docs/problems/2026-10-13-update-fan-out.md`,
where it is resolved by the fan-out to every recipient the commit names. Its
wording that the fan-out "moves peer communication into the app" is
corrected there (peer communication moved into org-io). The app's
one-send-per-command item it named is retired.

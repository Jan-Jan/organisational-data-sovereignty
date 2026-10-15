# The receiver holds the service lock while it waits for a peer

*Moved 2026-10-13 (S4, task T11 of `docs/plans/2026-10-06-org-io-transport.md`,
ruling A):* the report this file held — while the app's receiver ran, every
other command that needed the service waited until a peer connected, because
the receive loop took the service lock and then waited for an incoming
connection with it held — moved to org-io with the receive loop, to
`org-io/docs/problems/2026-10-06-receiver-holds-lock.md`,
where it is resolved by org-io's lock-free receive step. The app's receiver
still locks its `Mutex<OrgIo>` until task T16 moves it onto org-io's step on a
shared `Arc<OrgIo>`.

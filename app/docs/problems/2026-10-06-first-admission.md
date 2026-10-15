# Problem reports — the app never commits its own first admission

*Moved 2026-10-13 (S4, task T11 of `docs/plans/2026-10-06-org-io-transport.md`,
ruling A):* the report this file held — the app's receiver loop called only
org-node's self-delete entry point, which refuses an Envelope for an
Organisation the node holds no record of, so an invitee's app never committed
its own first admission — moved to org-io with the receive loop, to
`org-io/docs/problems/2026-10-13-first-admission.md`,
where it is resolved by org-io's one receive step. The app's receiver still
calls the retired entry point until task T16 moves it onto org-io's step
(the app does not compile against the new org-io until task T15).

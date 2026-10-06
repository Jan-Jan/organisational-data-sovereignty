# Problem reports — the app never commits its own first admission

**PR-kwwap5**: the app's receiver loop calls only org-node's
`receive_and_self_delete_if_revoked` (`app/src-tauri/src/commands.rs`,
`next_outcomes`), which refuses an Envelope for an Organisation the node holds
no record of, so an invitee's app never commits its own first admission and
never joins the Organisation it was admitted to.
affects: REQ-tcutr6, REQ-nfr3n2
opened: 2026-10-06
status: open

Found 2026-10-05 while planning change `worktree-org-node-chain-authority`. The
behaviour predates that change. It matters now because the owner's invitation
journey (an invitee enters an Invite, replies, and is admitted) ends in exactly
this first admission, and the expectation REQ-tcutr6 declares to org-node is
never consulted by the shipped app. By owner ruling (2026-10-06) it is fixed in
the transport change that follows, which moves the receive loop into the app
and rewrites it.

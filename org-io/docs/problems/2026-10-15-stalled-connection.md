# A stalled inbound connection stops a Persona's endpoint receiving

Opened 2026-10-15 by the fix of finding 1 of the S4 review, round 2
(`docs/plans/2026-10-06-org-io-transport.md`, merge-change step 6a; owner
ruling of 2026-10-15: fix the mediums).

**PR-d2p5y2**: one peer that completes the QUIC handshake and then opens no stream, or never finishes its body, stops the Persona's endpoint receiving anything from anyone, for as long as the peer keeps the connection open.
affects: LLR-z35vbh, LLR-wvgec4, SDD-xwft92, SDD-8uyg4s, HAZ-4n7pss
opened: 2026-10-15
status: resolved
resolution: fixed 2026-10-15 (change `worktree-org-io-transport`, review round 2, finding 1): root cause, the accept task awaited each connection's handshake, stream and body in turn with no limit; now each connection is read in a task of its own, bounded by `CONNECTION_TIME_LIMIT` and by `CONNECTIONS_AT_ONCE` slots per endpoint (LLR-8uf9a5, REQ-mnd9rj, RC-96w7nf for HAZ-4hj5m9). Reproduced by `a_stalled_connection_does_not_stop_another_peers_message`, with `a_stalled_connection_is_dropped_after_the_limit` and `a_seventeenth_connection_waits_for_a_slot_while_sixteen_stall` (`org-io/tests/endpoints.rs`).

## What was observed

The reviewer read it from the code (review round 2, finding 1; the
reviewer's sub-review rated it high, A1). Each endpoint has one accept task
(`org-io/src/endpoints.rs`, `accept`), and it handles one connection at a
time: `OrgEndpoint::recv_answerable` awaits the incoming connection, its
handshake, `accept_bi` and `read_to_end` in turn, with no time limit. The
next connection is not accepted until that one has given a body or failed.
LLR-wvgec4 takes every connection whose handshake completes, so any party
that can reach the endpoint can hold it. HAZ-4n7pss and RC-5wz559 cover
memory and floods, not one connection held open. The same shape was in
org-node's transport on master.

## Why it matters

A Member whose node cannot receive does not learn of a membership change,
a revocation among them (REQ-bupq5y, LLR-z35vbh: a message to any Persona's
Device reaches the one receive path).

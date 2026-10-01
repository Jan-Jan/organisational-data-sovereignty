# Problem report — org-node's loopback transport times out where it did not

One report, filed from another unit's change. `worktree-guardrails-on-chain-client-arch`
is on-chain-client's architecture tooth and touches no org-node source or test
file — the only file it adds under `org-node/` is this report;
org-node is in its impact set as a **dependent**, and running that unit's gate
is how this surfaced. It is recorded here, in the unit that owns the defect,
rather than in the change's own ledger.

The owner decided on 2026-09-28 to record it and proceed rather than block the
architecture tooth behind it: the evidence that the change did not cause it is
mechanical, and burying a real transport regression inside an unrelated
documentation change would be worse than filing it.

## Seven loopback-transport tests time out at ten and thirty seconds

**PR-d4nye8**: the three org-node test targets that exercise the iroh transport
over `TransportMode::Loopback` — `admission_sender`, `service_stories` and
`transport_handshake`, seven tests in all — now fail by timeout rather than
passing, on a source tree byte-identical to the one that measured 51 passed and
0 failed ten days earlier, so the unit's gate is red without any change to the
unit.
opened: 2026-09-28
status: open

Where: `org-node/tests/transport_handshake.rs:91` panics with
`send timed out after 10 s: Elapsed(())`;
`org-node/tests/admission_sender.rs:121` and
`org-node/tests/service_stories.rs:124` panic with
`B receive_and_verify timed out: Elapsed(())` after thirty seconds. The
endpoints under test are built by `OrgEndpoint::bind`, which is
`org-node/src/transport/endpoint.rs:44-46` —
`bind_with_mode(device, TransportMode::Loopback)` — and that mode sets
`RelayMode::Disabled` at `org-node/src/transport/endpoint.rs:62-68`, where the
builder is `presets::Minimal` and `.bind()` is called with no address. The
"binds on `127.0.0.1`" half of this sentence, as first written, cited those same
lines; it is not in them. It is the documented behaviour of the mode, stated in
the doc comment at `:50`. Corrected 2026-09-29 by review round 4; the argument
is unaffected, because what matters here is that the relay is disabled.

Observable symptom: `cargo test -p org-node --features app,test-support …`
reports **44 passed, 7 failed** where the unit's recorded baseline is **51
passed, 0 failed**. The declared `verify_commands` line fails fast on the first
failing target; the 44/7 figure comes from a `--no-fail-fast` diagnostic run.
The seven are exactly the three targets above (3 + 3 + 1); everything that does
not cross the loopback transport still passes — lib 23, `verify_against_chain`
13, `store_at_rest` 4, `wire_frame_bound` 3, `transport_networked` 1 — 23 + 13 +
4 + 3 + 1 = 44 — plus two bolero targets on budget.

## What has been established, and what has not

Established:

- **No change to this unit's source is involved.**
  `git diff master...HEAD -- org-node/src org-node/tests` is empty for the
  change that found it, as are the equivalent diffs for `org-members/` and
  `on-chain-client/src/`. The only file that change adds anywhere under
  `org-node/` is this report; `git diff --stat master...HEAD -- org-node/` is
  therefore *not* empty, it is this one file. That change is documentation and
  on-chain-client test annotations only — no production source under any unit
  is touched, which the diffs named above show directly. A file count is
  deliberately not given here: it went stale twice while this report was open.
- **It is not new code meeting old tests.** The same tree measured 51 passed, 0
  failed on 2026-09-18 (`docs/verification/2026-09-18-worktree-guardrails-on-chain-client-finalize.md`),
  on 2026-09-17 (`…-org-members-arch.md`) and on 2026-09-11
  (`…-on-chain-client-risk.md`). The repository's lockfiles are tracked and
  unchanged across that span.
- **It is not the harness sandbox.** `transport_handshake` was re-run with the
  sandbox explicitly disabled and failed identically, at the same line, with
  the same ten-second timeout.
- **It is not external relay or discovery infrastructure.** This was the first
  hypothesis and it is wrong. All three failing targets bind through
  `OrgEndpoint::bind`, which disables the relay entirely and binds loopback;
  they reach no n0 relay, no Pkarr publisher and no DNS lookup. The one
  transport target that still passes, `transport_networked`, is the one that
  runs an **in-process** relay with in-memory address lookup — so the passing
  and failing sets do not divide along an internet-access line.
- Ordinary outbound networking from this environment works: `git fetch origin`
  succeeds.

Not established — and deliberately not guessed at:

- Whether the cause is in this machine's environment (a host or OS change in
  the intervening ten days; macOS local-network permissions are a candidate
  worth eliminating first, since this is Darwin 27) or a latent nondeterminism
  in the loopback datapath that the earlier runs happened not to hit.
- Whether it reproduces on any other machine. **That is the single most
  valuable next measurement**, because it separates the two hypotheses above
  and nothing else in this report does.

## Why it is filed rather than fixed

Fixing it means diagnosing a transport timeout of unknown depth, which is not
the work of an architecture-documentation tooth for a different unit, and would
have to be done under `resolve-problem` with a reproducing test of its own. The
report exists so the red is carried in the unit's own ledger with its evidence,
rather than living in one verification record for a change that had nothing to
do with it.

Until it is resolved, **org-node's gate is red**, and every change whose impact
set includes org-node will meet it. That is the cost of recording rather than
fixing, and it is stated here so the next change is not surprised by it.

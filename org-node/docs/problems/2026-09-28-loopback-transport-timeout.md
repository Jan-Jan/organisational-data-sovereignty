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
status: resolved
resolution: `TransportMode::Loopback` named no bind address, so iroh bound its
pre-configured wildcard sockets and the endpoint advertised the host's LAN
address; every "loopback" exchange left the machine. Fixed by clearing iroh's
default IP transports and binding `127.0.0.1:0` and `[::1]:0` explicitly in
`org-node/src/transport/endpoint.rs`. Verified by
`loopback_mode_binds_and_advertises_loopback_only` in
`org-node/tests/transport_handshake.rs` (REQ-db6s7q), which asserts the bound
sockets and the advertised address set rather than a delivery timeout.

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

**Four of the six line numbers in the paragraph above no longer resolve; two
still do.** The fix lengthened `endpoint.rs` and `transport_handshake.rs`, so
the citations into those two moved: `transport_handshake.rs:91` is now `:94`,
`endpoint.rs:44-46` is now `:53-55`, `endpoint.rs:62-68` is now `:100-111`, and
the mode's doc comment at `:50` is now `:59`. The other two are **unmoved**,
because this change touches neither file: `admission_sender.rs:121` and
`service_stories.rs:124` are still the exact `expect("B receive_and_verify
timed out")` sites the paragraph cites.

The paragraph itself is kept as written, because it is the record of what was
seen on the day; the present positions are given here so that following it does
not land a reader in unrelated text.

**This note was itself wrong twice, and that is the point of keeping it.** Its
first version said "none of them resolves today", which condemned two good
references and offered no replacement for them — review round 4. And its
`endpoint.rs:62-68 → :93-104` figure was correct when round 2 wrote it and
stale by the time it was committed, because a later commit in the same change
added seven lines above that arm and nobody re-read the citations that had just
been fixed. Round 4 measured the real offset. So this repository has now broken
the rule this paragraph's closing sentence states — re-resolve citations after
the *last* edit to the files they point into, not after the edit that prompted
them — inside the very note that states it (`docs/verification/2026-09-30-…`,
finding-43, is the earlier instance).

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

## Resolved 2026-10-03 — the mode never bound loopback

Both hypotheses left open above are wrong, and so is the framing that produced
them. This was never a transport *regression*: it is a defect in
`bind_with_mode` that has been present since `2bb1c21` created the file, which
`git log -L '/pub async fn bind_with_mode/,/^    }/'` shows directly — the
Loopback arm has never named a bind address.

**Measured before the fix**, by binding one endpoint and printing what it
reports:

```
bound_sockets()      = [0.0.0.0:62868, [::]:57839]
node_addr_for_dial() = {Ip(0.0.0.0:62868), Ip([::]:57839)}
addr()               = {Ip(192.168.4.240:62868)}
```

`presets::Minimal` pre-configures a wildcard socket per address family, and the
builder never replaced them. So the endpoint listened on every interface, and
the only address it offered a dialling peer was the host's **LAN address**. The
three failing targets exchange that address and dial it; the traffic left the
machine and came back in over the local network, or — once the host stopped
obliging — did not come back at all.

That explains every fact this report had already established without being able
to join them up:

- **Not this change's source, and not new code meeting old tests.** Correct,
  and now unsurprising: the code was always wrong and the network was always
  what decided the outcome.
- **Not the harness sandbox.** Correct — a sandbox does not change which
  interface a socket is bound to.
- **Not external relay or discovery.** Correct, and this is why the hypothesis
  was so hard to kill: the relay genuinely is disabled. The traffic was on the
  LAN, not on the internet, and "offline" was never the property under test.
- **`transport_networked` keeps passing.** Not because it is luckier, but
  because `bind_with_relay` forces every path through an in-process relay
  reached on the local host, so it never depends on the wildcard bind at all.
- **`node_addr_for_dial()` "does not complete the dial in this setup"** — a
  note that sat in `transport_handshake.rs` as an empirical curiosity. It was
  the defect, stated plainly: `bound_sockets()` returned `0.0.0.0`, and no peer
  can dial the wildcard.

**The root cause was sighted once and dismissed.** Review round 4 corrected the
paragraph above to say that `.bind()` is "called with no address", and ruled
the argument unaffected "because what matters here is that the relay is
disabled". The observation was exactly right and the inference from it was
wrong: what mattered was not that the relay was off but that nothing had turned
the wildcard off. It is recorded here because the correction and the diagnosis
were the same sentence, five days apart.

**The fix** clears iroh's default IP transports and binds both loopback
addresses explicitly. Naming one is not enough — a bind address replaces the
pre-configured wildcard only for its own address family, so binding
`127.0.0.1` alone leaves `[::]` listening on every interface.

**Measured after:**

```
bound_sockets()      = [127.0.0.1:64601, [::1]:55937]
node_addr_for_dial() = {Ip(127.0.0.1:64601), Ip([::1]:55937)}
addr()               = {Ip(127.0.0.1:64601), Ip([::1]:55937)}
```

`addr()` is given here deliberately. It is the accessor whose LAN value in the
"before" block is what exposed the defect, and the first version of this block
omitted it — dropping the most direct refutation for no reason. It is also the
accessor the three failing targets dial.

org-node reports **53 passed, 0 failed**, against the 51/0 baseline this report
cites plus the **two** tests the fix adds — the contract test and the
robustness case. (This paragraph said 52 and "the one test" until 2026-10-03:
the robustness test arrived in a later commit of the same change and the figure
was not re-read. A count in prose goes stale against the tree that produced it,
which is the whole reason the merge gate's figures live in the verification
record and not here.) The suite completes in under four seconds where it
previously spent about seventy-two in timeouts;
`delivers_and_verifies_admit_over_iroh` passes in 0.05s.

**What the fix did NOT make honest, corrected 2026-10-03 before merge.** An
earlier draft of this section claimed that `check_transport`'s Loopback branch
in `org-node/src/preflight.rs` "now confirms what it claims". It does not, and
the claim was the same species of error this report is about — crediting code
with a property it does not have.

That branch passes when `bound_sockets()` is non-empty, under a comment reading
"bound_sockets() is exactly what node_addr_for_dial() builds from". It is
**unchanged by this fix**, it still tests only non-emptiness, and it would pass
just as vacuously if the bind regressed to the wildcard tomorrow. Its verdict
is true today because of code elsewhere, not because the check discriminates.
A preflight that asserted what REQ-db6s7q states — that every bound socket is
loopback — would catch the regression; this one cannot, and that is left as
found rather than fixed, because widening a preflight check is not this
change's business and would arrive untested.

**The measurement this report called most valuable is no longer needed.**
"Whether it reproduces on any other machine" was the question that separated
the two hypotheses; both are superseded, and the answer is now known without
running it — the bind is wrong on every machine, and whether the *symptom*
appears depends only on whether that host routes datagrams addressed to its own
LAN address back to itself. A reproduction that depends on the host's network
is exactly what the new test replaces.

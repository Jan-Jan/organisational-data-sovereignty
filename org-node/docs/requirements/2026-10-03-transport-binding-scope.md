# Requirements — the scope of the transport bind

Two requirements, minted while resolving PR-d4nye8: REQ-db6s7q, which states
which addresses Loopback mode may bind and advertise, and REQ-2wzfzv, which
states which of those sockets must come up for the mode to be provided at all.
The node's two transport modes differ in exactly one thing a reader cares
about: which interfaces the endpoint listens on and which addresses it hands a
peer to dial. That difference was written down only in a rustdoc comment on
`OrgEndpoint::bind_with_mode`, and the comment was false for as long as the
function existed — which is why nothing caught it.

Terms: *Device key* and *Wire message* are defined in the root
`docs/CONTEXT.md` and `org-node/docs/CONTEXT.md` respectively, and are used in
the assessment section below rather than in the requirement.

**Both requirements are written in vocabulary neither glossary defines** —
*Loopback mode*, *endpoint*, *dialling*. That is a gap, not a style choice:
`docs/CONTEXT.md` lists "endpoint key" only as a term to avoid, and neither
glossary mentions the transport at all. The terms are iroh's and this crate's,
used here because there is no project word for them yet; filling that in is
tooth 9's glossary work and this file is one of its inputs.

## Binding scope

**REQ-db6s7q**: The software shall, when its transport is configured in
Loopback mode, bind its endpoint only to loopback addresses, and shall offer a
peer for dialling only loopback addresses.
satisfies: derived

Verified by two tests in `org-node/tests/transport_handshake.rs`, both of which
assert the bound sockets and the advertised address set directly rather than
inferring the binding from a delivery timeout:

- `loopback_mode_binds_and_advertises_loopback_only` — the normal path.
- `loopback_mode_holds_under_repeated_and_colliding_binds` — the robustness
  case class C requires. `bind`'s whole input surface is one keypair, so the
  abnormal conditions that exist are repetition and collision rather than
  malformed arguments: eight endpoints alive at once in one process, two of
  them claiming the same Device key. It also pins the asymmetry between the
  address families — exactly one IPv4 loopback socket, required; whatever IPv6
  socket exists, loopback.

**One clause of this requirement cannot fail independently of the other.**
`node_addr_for_dial()` is built directly from `bound_sockets()`, so an
advertised address is non-loopback only if a bound socket already was. The
second assertion is still worth making — it is the accessor production hands a
peer (`org-node/src/service.rs:670`, `:720`) — but it adds no discrimination
the first does not already have, and a reader should not count it as two
independent checks.

**Binding loopback is not the whole of "offline", and this requirement is not
the whole of the mode.** An endpoint can be bound entirely to loopback and
still reach the network, by a relay or by publishing its address to a discovery
service. Those are properties of the relay mode and of the address-lookup
configuration, not of the bind addresses, and REQ-db6s7q does not state them.
Review round 2 established this by measurement rather than by argument: it
replaced `presets::Minimal` + `RelayMode::Disabled` with `presets::N0` — n0
relay servers, Pkarr publishing, DNS lookup — and both tests stayed green.

**Of those two escape routes the tests close one.** Address publication is
asserted: `address_lookup()` must be empty, which is synchronous, needs no
network, and reddens on the mutation above. **The relay is not asserted.**
The tests also read `addr().relay_urls()`, but that runs at bind time, before a
home relay can have been acquired, so it is vacuous — review round 3 measured
it by replacing `RelayMode::Disabled` with a real n0 relay map and watching all
three tests stay green while the endpoint went on to acquire a relay home, and
round 4 reproduced it with `RelayMode::Default`.

An earlier version of this paragraph claimed both were asserted. It was the
last record in the change still saying so after the correction landed in the
register, the rustdoc and the test's own doc comment — a reminder that
correcting a claim means finding every place it was made, and that the
normative statement of a requirement is the worst place to leave the stale
copy.

So `RelayMode::Disabled` on that arm is load-bearing and unverified: changing
it silently un-confines the mode, and no gated test will notice. Closing it
needs an assertion after `online()` resolves, which measures whether this host
can reach a relay rather than how the endpoint is configured, and iroh exposes
no relay map on `Endpoint` for a synchronous alternative. Recorded as a gap in
this change's verification record,
`docs/verification/2026-10-03-worktree-org-node-loopback-timeout.md`.

## Which sockets must come up

**REQ-2wzfzv**: The software shall, when its transport is configured in
Loopback mode, fail to provide an endpoint when it cannot bind an IPv4
loopback socket, and shall provide one when it cannot bind an IPv6 loopback
socket.
satisfies: derived

This is the policy the fix for PR-d4nye8 chose and did not, at first, write
down anywhere a gate could see — which is the defect that report is about,
repeated inside its own fix and caught by review round 2.

The asymmetry is deliberate. `BindOpts::is_required` defaults to true, so
naming `[::1]:0` without overriding it would abort the whole endpoint on a host
that cannot bind IPv6 — an IPv6-disabled kernel, a minimal container, a
locked-down CI runner — where iroh's own pre-configured `[::]` bind, which this
replaces, is documented as allowed to fail. Taking the default would have
narrowed the set of hosts this crate runs on as a side effect of fixing a
defect about depending on the host silently. IPv4 loopback stays required
because without it there is no usable Loopback transport at all, and failing
closed is the right failure.

**Neither clause of this requirement is verified by any gated test**, and an
earlier draft of this paragraph claimed one of them was. Review round 3
measured it: changing `bind_addr("127.0.0.1:0")` to
`bind_addr_with_opts("127.0.0.1:0", BindOpts::default().set_is_required(false))`
— the exact negation of the first clause — left both tests green.

The reason is that both clauses are about **bind failure**, and neither
antecedent can be produced from a test on a host where both loopback sockets
bind. `loopback_mode_holds_under_repeated_and_colliding_binds` asserts that
exactly one IPv4 loopback socket is present and that any IPv6 socket is
loopback, which is a fact about the normal case and says nothing about what
happens when either bind fails. Nothing in a test can take `127.0.0.1` or
`[::1]` away from the machine it runs on, and `bind` names its own addresses
with port 0, so there is no occupied-port route either.

What the requirement rests on instead is a reading of iroh's source:
`BindOpts::is_required` defaults to true and is honoured in
`socket/transports/ip.rs`, where a bind error is returned only for a required
transport. That is a correct reading, and it is not a measurement.

**So REQ-2wzfzv is an unverified requirement in a class C unit.**
`check-trace.sh` does not report it, because MISSING-TEST is satisfied by a
`verifies:` reference in a file under `test_paths` and the annotation is there
— the same property this repository recorded on 2026-09-28 against the
chain-facing transport item in on-chain-client's architecture ledger, named by
file rather than by ID here because a consumer may not cite a provider's
non-exported items (`check-trace.sh` reports NON-EXPORTED-REF, as it did for
the first draft of this sentence):
`on-chain-client/docs/architecture/2026-09-28-decomposition.md`. The gate is
green over a requirement nothing exercises, which is stated here, in this
change's verification record, and in the owner register, rather than left for a
reader to find. Making it testable means a `test-support`-gated constructor
that accepts bind addresses, so an occupied port can produce the failure; that
is new production surface and is booked rather than added late in a defect fix.

## Why this is worded about addresses and not about delivery

Seven tests already exercised this path end to end, and all seven were green
for weeks while the property was false. They were green because the host
happened to deliver datagrams addressed to its own LAN address back to itself,
and they went red — all seven, at ten and thirty seconds — when it stopped
doing so. A requirement worded about successful delivery would have been
satisfied by that accident and would be satisfied by it again.

The requirement is therefore about the observable the mode actually promises.
Its test reads `bound_sockets()` and `node_addr_for_dial()` and fails on the
address itself, by name, in hundredths of a second, on a machine with no
network at all.

## What the RMF was asked

**Both** requirements here are `satisfies: derived` and **no risk control cites
either**.

Two earlier drafts of this paragraph got the arithmetic wrong, in opposite
directions, and the figures are worth stating exactly because they are the
premise of this whole section. Measured on this tree: this unit has
**nineteen** requirements and **every one** of them is `satisfies: derived`.
Fifteen — all of them in `2026-09-09-verify-and-commit.md` — carry
`(implements: RC-…)`. **Four do not**: the two expectations in
`2026-09-06-dependency-expectations.md`, REQ-ysyu9g and REQ-q92yac, where that
file says at `:26` and `:35` that the withholding is deliberate pending the
analysis; and the two minted here. So REQ-db6s7q is the third derived
requirement in this unit citing no control and REQ-2wzfzv is the fourth.

What is unprecedented is the reason. The expectations cite no control because
they are owed by other units and their controls are not this unit's to name.
These two cite none because the behaviour they require is not a mitigation of
anything in this register — which is the fact the assessment below has to
handle, and it handles both of them.

A requirement in that position is not left unassessed — `check-trace.sh`
reports `UNANALYZED-DERIVED` for a derived item the RMF never mentions, and it
reported it for this one until the assessment was written. The assessment is
the section dated 2026-10-03 in
`org-node/docs/risk/2026-09-09-org-node-hazards.md`. It concludes that the
wildcard bind mints no new hazard, and it is the author's, made in the change
that fixed the defect rather than in a full `analyze-risks` pass. It names its
own weakest steps rather than being summarised here, because a summary of a
risk assessment in a requirements file is a second copy to keep true.

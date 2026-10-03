// -*- mode: Bluespec; -*-
/// Vacuity witnesses and scenario checks for the protocol layer. Reachability
/// is demonstrated via witness invariants: each `...Reachable` value is FALSE
/// exactly in a target state, so `quint run --invariant=<it>` finding a
/// VIOLATION proves that state is reachable (the counterexample IS the witness).
module ods_instances {
  import protocol.* from "./protocol"

  // WITNESS 1 — revocationSafety is non-vacuous: a SETTLED object coexists with
  // a non-empty `revoked` set. (Violation => such a state is reachable.)
  val settledWithRevocationReachable =
    not(OBJECTS.exists(o => isSettled(o, chain, local, objToken, rootKeys)) and revoked != Set())

  // WITNESS 2 — forkSafety is non-trivial: two devices can sit at DIFFERENT
  // epochs (so the implication's antecedent is exercised, not vacuous).
  val membersCanDifferReachable =
    not(DEVICES.exists(d1 => DEVICES.exists(d2 => local.get(d1).epoch != local.get(d2).epoch)))

  // WITNESS 3 — a revoked member is fully excluded post-rotation: there is a
  // reachable state where some principal is revoked AND holds neither the
  // current org secret nor any object's current token.
  val revokedFullyExcludedReachable =
    not(revoked.exists(p =>
      not(getOrEmpty(orgKnows, chain.orgGen).contains(p))
      and OBJECTS.forall(o => not(tokenKnows.get(objToken.get(o).id).contains(p)))
      and chain.orgGen > 0))

  // WITNESS 4 — a stale device accepts a chain-invalid write (tauWindow non-vacuous)
  val taintAcceptedReachable =
    not(acceptedWrites.exists(w => chainInvalid(w, chain, rootKeys)))

  // WITNESS 5 — the compromised-key (gen-mismatch) sub-case is reachable: a still-current
  // member's write was accepted with a gen != the current chain gen
  val keyGenMismatchAcceptedReachable =
    not(acceptedWrites.exists(w =>
      membersOf(rootKeys, chain.root).contains(w.owner)
        and genOf(rootKeys, chain.root, w.owner) != w.gen))

  // WITNESS 6 — a quiescent, fully-converged, post-removal state is reachable
  val convergedReachable =
    not(quiescent and chain.epoch > 0
        and currentDevices(rootKeys, chain.root).forall(d => local.get(d).root == chain.root))
}

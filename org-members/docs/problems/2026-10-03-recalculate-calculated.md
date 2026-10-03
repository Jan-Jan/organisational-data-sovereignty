# Problem reports — recalculating an already-calculated trie succeeds

**PR-zqvs7t**: `OrgTrie::recalculate` on a trie with no pending changes
succeeds, returning the same root and an empty change set, where the owner's
intent is that it is refused with an error.
affects: REQ-avmu3j, LLR-n7nya3.
opened: 2026-10-03
status: open

Found during review of the change that resolves PR-499dzp. A review round
proposed writing the current behaviour down as a clause of LLR-n7nya3. The owner
rejected it: a node's hash is a write-once cell, so asking to calculate a
trie that has already been calculated is a caller error and should be reported.
The owner decided on a new variant, `OrgMembersError::HashesAlreadyCalculated`,
which mirrors `HashesNotCalculated`, so that the refusal is distinct from a
broken invariant.

No item states the current behaviour, but code depends on it. `genesis()`
returns a calculated trie, and these sites call `recalculate()` directly on a
calculated trie:

- org-node production code: `service.rs:612` (in `create_organisation`) and
  `service.rs:969-971` (the fallback in `receive_and_verify` that, when there is
  no snapshot, rebuilds a single-admin trie on the receiving side);
- org-node test support and tests: `src/test_fixtures.rs:48`,
  `tests/chain_genesis_e2e.rs:205` and `:381`, `tests/transport_handshake.rs:39`,
  `tests/transport_networked.rs:47`, and
  `tests/fuzz_verify_against_chain/fuzz_target.rs:39`;
- four org-members integration tests: `root_hash_errs_until_recalculated`,
  `device_slot_order_does_not_change_the_root`,
  `every_device_slot_reaches_the_root` and
  `add_then_delete_returns_to_the_empty_root`;
- org-members `fuzz_tests.rs`: `trie_ops_never_panic_and_count_consistent`
  (verifies REQ-ds8ryr, LLR-h9gs32) applies `Op::Recalculate` to the genesis
  trie and to tries it has just recalculated, and discards an `Err` with
  `if let Ok`. Once the refusal lands, those steps would silently skip the
  `member_count` check, which is the same kind of drop PR-499dzp removes.
  `delta_roundtrip`, the `calculate_delta` round trip and
  `delta_canonicality_fuzz` return early on an `Err` when no op was accepted.
  The outcome is the same there, because the delta would be empty.

Some tests do not call `recalculate()` on a calculated trie themselves, yet
fail through `service.rs` (`create_organisation`) or `test_fixtures.rs`:
`service_stories`, `admission_sender`, `verify_against_chain`,
`service::tests::create_organisation_advances_mock_chain` and six
`envelope::tests`.

The independent review measured this on 2026-10-03 with a temporary mutation
that refuses the call. Exactly the four org-members integration tests went red,
and `fuzz_tests` stayed green. In org-node's gated command, the tests named
above went red, along with one `transport_handshake` test, `transport_networked`
and the `fuzz_verify_against_chain` harness. `store_at_rest` stayed green. So
the fix spans two units, and it is a change of its own. Its scope:

- amend LLR-n7nya3 to state the refusal;
- add the variant, and make `recalculate` return it when
  `!has_pending_changes()`;
- drop the redundant `recalculate()` at each direct site above;
- make the fuzz op stop discarding the refusal;
- add a test that the refusal is reported.

The conformance driver already recalculates only a pending trie, so it needs no
change.

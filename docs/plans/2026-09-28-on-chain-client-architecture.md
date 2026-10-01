# on-chain-client Architecture Implementation Plan

**Goal:** Give `on-chain-client` its first architecture ledger — nine software
items and thirty-two low-level requirements refining the twenty-one high-level
requirements — a measured SOUP inventory, and a recorded class C deviation for
the one item nothing at this unit's gate can verify.

**Implements:** SDD-5wamsz, SDD-5b8wxs, SDD-bw7v5x, SDD-v2rtka, SDD-d5jh6t,
SDD-f2s7bx, SDD-4z3k2u, SDD-m59zrg, SDD-3b8zef, LLR-xv7auy, LLR-z8rrkr,
LLR-b4p32h, LLR-2yhra8, LLR-3bkhuc, LLR-7pjzjn, LLR-vktf8w, LLR-62tqnv,
LLR-bhwsn6, LLR-2y9qdc, LLR-v62yjq, LLR-b3s7st, LLR-u8ajby, LLR-e6skvu,
LLR-u2e389, LLR-rjcqg3, LLR-n6gghu, LLR-89pdz9, LLR-8242kq, LLR-6tjhgk,
LLR-2v5u4d, LLR-mzh8df, LLR-sq76u3, LLR-nq7nhg, LLR-emp3g9, LLR-yhw34z,
LLR-2znra8, LLR-kfr75c, LLR-9qp3k7, LLR-d3ef7s, LLR-56bzcj, LLR-48ygak

**Safety class:** C throughout (`on-chain-client/.guardrails/config.yaml`, no
per-item override). SDD-3b8zef carries a **recorded deviation**, not an
override: see T11.

**Verification:** `on-chain-client`'s `verify_commands` — one `cargo test`
invocation over `--lib` plus the three bolero targets and the nine annotated
integration targets. Baseline measured on this branch at `2cc25c0`:
**72 passed, 0 failed, 0 ignored**, three bolero targets on budget without
panic.

## What makes this tooth affordable, and what it must not become

Every one of the thirty-two LLRs below is carried by a test that **already
exists and already runs at this unit's gate**. The work is annotation, not
authorship — with one exception, T7's new `root_updated` epoch-overflow case,
which closes a real gap (the shared `decode_uint256_to_u64` bound is exercised
today only through the storage path, never through the event topic path).

**A test that is already green has never been watched failing.** So every LLR
is discharged by RED BY MUTATION, exactly as `ec66743` did for `org-members`:

1. Apply the named mutation to the named source file.
2. Run the named test. Watch it fail, and check the failure is the mutation's
   and not a compile error elsewhere.
3. Revert the mutation.
4. Prove the revert is byte-exact: `shasum -a 256` the file before and after,
   and report both digests.

Report the mutation, the failing test's name, the failure reason and the two
digests on the dispatch report's `red -> green:` lines. `merge-change` step 6b
copies them into the verification record; they are the only evidence the iron
law was honoured, and nothing re-derives them later.

**The trap this change must not fall into.** `check-trace.sh`'s MISSING-TEST is
satisfied by a `verifies:` reference *in a file under `test_paths`* — not by a
test executing. `on-chain-client/tests` holds nine integration targets
(`00_chopsticks_sanity`, `01_multisig_sanity`, `off_chain_genesis_ceremony`,
`p_address_is_orgid`, `reorg_cancels_proposed`, `scenario_a_full`,
`smoldot_smoke`, `two_orgs_one_watcher`, `regenerate_corpus`) that run in **no
gate**. Seven of the nine need a chopsticks fork; the other two do not.
`smoldot_smoke` is `#[ignore]`d and needs live Paseo, and `regenerate_corpus` is
an `#[ignore]`d corpus writer that needs no chain at all. Why each one runs
nowhere does not matter to the trap: MISSING-TEST is a text scan of
`test_paths`, so all nine are hiding places whatever their reason. Annotating an
LLR onto one of them turns the gate green while no assertion ever runs. **No
task below may add a `verifies:` annotation to any of those nine files.** That
is the same defect as a hazard register arguing from CI that never executed, and
this repository has already paid for that lesson once.

**Corrected 2026-09-28 by this change's review sweep.** This paragraph said all
nine "need chopsticks or anvil", which is false for two of them and contradicted
T11 far below (line 669), where the same change reports that five of the nine
exercise `OrgRegistryClient` at all. The count of **nine** is right here and is
kept — it counts hiding places, not chain-dependent targets — and only the
characterisation is corrected.

## The decomposition

Nine items. The seam is the chain of custody of a byte: what it is typed as,
how an address is derived from it, where it is stored, which decoder reads it,
what that decoder refuses, who is admitted to see it, and what the transport
does with it.

| Item | Responsibility | Source | LLRs |
|---|---|---|---|
| SDD-5wamsz | ABI value types, the observation vocabulary, the client surface's typed error and its stream | `src/types.rs`, `src/state.rs`, `src/client.rs` (`ClientError`, `SubscribedEventStream`) | 3 |
| SDD-5b8wxs | Account-to-address derivation | `src/h160.rs` | 3 |
| SDD-bw7v5x | Solidity storage-slot derivation | `src/client.rs` (`internals::solidity_mapping_slot`, `internals::increment_slot`) | 5 |
| SDD-v2rtka | Runtime-version decoder selection, decode error vocabulary | `src/decode/mod.rs`, `src/decode/dispatch.rs` | 2 |
| SDD-d5jh6t | Event log decoder | `src/decode/v_paseo_ah.rs` (event half) | 9 |
| SDD-f2s7bx | Organisation state decoder | `src/decode/v_paseo_ah.rs` (storage half) | 4 |
| SDD-4z3k2u | Subscriber admission | `src/client.rs` (`internals::log_is_ours`, `event_admin`) | 3 |
| SDD-m59zrg | Best-head scan and reorg rule | `src/client.rs` (`internals::scan_step`, `internals::ScanStep`) | 3 |
| SDD-3b8zef | The chain-facing transport shell | `src/client.rs` (remainder) | **0 — deviation** |

`v_paseo_ah.rs` carries two items, and `client.rs` carries five. An item is a
responsibility with an interface, not a file: `decode_org_state` and
`parse_revive_event` are two interfaces that happen to share a module, and
`client.rs`'s `internals` block exists precisely because three of its decisions
were extracted to be testable without a chain.

---

### T1 — The architecture ledger, SOUP inventory, risk assessment and problem report

**Files touched:**
`on-chain-client/docs/architecture/2026-09-28-decomposition.md`,
`on-chain-client/docs/architecture/soup.md`,
`on-chain-client/docs/architecture/README.md`,
`on-chain-client/docs/risk/2026-09-28-design-derived.md`,
`org-node/docs/problems/2026-09-28-loopback-transport-timeout.md`
**Parallel:** no (serial, first — every later task annotates against the item
text this task fixes)

**Filenames updated 2026-09-28 at `merge-change` step 3.** The three ledger
files above were written as `DRAFT-<branch>-<slug>.md` and renamed by
`finalize-docs.sh`; the names here are the ones on disk. The fourth path this
task originally listed, a problem-report draft for `verify.rs`, was never
created — PR-h4mb8y already covers it — and the org-node problem report filed
after the gate stands in its place. No gate reads a prose path, so a reference
to a consumed draft name is caught by nothing and is repaired by hand.

Not dispatched. The decomposition is the judgment this change exists to make;
it is written by the agent holding the plan and the trace.

Four deliverables:

1. **The decomposition file** — the nine SDD items and thirty-two LLRs, in the
   grammar `on-chain-client/docs/architecture/README.md` states, with each
   item's source files named and the evidence for each LLR cited by test name.
2. **`soup.md`** — currently the twelve-line template with an empty table. It
   is replaced by a measured inventory to the standard `ec66743` set for
   `org-members`: an evidence-and-provenance section naming the lockfile the
   versions were read from, a row per direct dependency with its role, the
   requirements it supports and its risk considerations, the transitive
   closure, and a test-only-dependencies section recording what is *not* SOUP.

   **Measure, do not copy.** `on-chain-client/Cargo.toml` declares its own
   empty `[workspace]`, so unlike `org-members` this unit's lockfile is
   `on-chain-client/Cargo.lock` and cargo genuinely reads it. Confirm that with
   `cargo locate-project --workspace` run from `on-chain-client/` before
   writing the provenance paragraph, and take every version from the lockfile
   it names. Direct dependencies are `parity-scale-codec`, `tiny-keccak`,
   `futures-core`, `futures-util` and `subxt`; the closure comes from
   `cargo tree --edges normal`.

   Two rows carry real weight and must not be written as boilerplate.
   `tiny-keccak` **is** the derivation in SDD-5b8wxs and SDD-bw7v5x — a defect
   in it silently reads another organisation's slot. `subxt` carries the
   wasm32-browser lane blockage already recorded in `Cargo.toml` (jsonrpsee
   0.24.11's `jsonrpsee-wasm-client` was never published to crates.io, and
   cross-target feature unification re-enables it), which is a known anomaly of
   a supplied component and belongs in the inventory rather than only in a
   manifest comment.
3. **The derived-requirements assessment** — `check-trace.sh` reports
   UNANALYZED-DERIVED for any item marked `satisfies: derived` the RMF never
   mentions. **One** LLR below is derived — LLR-2y9qdc — and it needs an
   assessment naming its ID in this unit's risk ledger. "No hazard impact
   because <reason>" is a valid assessment; silence is not. (This plan named two,
   adding LLR-b4p32h. Review round 2 measured that REQ-ntn4ss states LLR-b4p32h
   verbatim — "carrying both the hash and the number of the head that was
   discarded" — so it was marked derived in error and now satisfies REQ-ntn4ss.
   Its risk-ledger entry is kept as a hazard note.)
4. **`src/verify.rs` — no new problem report.** The owner's decision on
   2026-09-28 was to record it as a problem report and change no code. It turns
   out already to be recorded: **PR-h4mb8y**, opened 2026-09-10, `status: open`,
   states that the file "is seven lines of doc-comment and no code" while its
   header claims in the present tense to be the verifier that closes the loop
   with `org_members::CandidateTrie::verify_against`. A second report would be a
   duplicate of a better one. `PR-bvhpz2` was minted for this and is
   **discarded unused** — IDs are allocated against nothing, so an unused token
   costs nothing and is never reused. The decomposition file cites PR-h4mb8y
   from its "What is not an item" section instead.

Also amend the README's `## Overview` section, which is the standing
decomposition picture every later change edits, with the nine-item table above
and the segregation position (this unit declares no `depends_on:` and no
`segregated_from:`, so there is no cross-class boundary to argue).

**Expected output:** `.guardrails/scripts/check-ids.sh` with
`GR_CONFIG=on-chain-client/.guardrails/config.yaml` exits 0 under
`--allow-draft-files`. `check-trace.sh` will still report MISSING-TEST for all
thirty-two LLRs until T2–T11 land; that is expected at this point and is the
worklist.

---

### T2 — SDD-5wamsz: the value types

**Files touched:** `on-chain-client/tests/type_widths.rs`
**Parallel:** yes (after T1)

SDD-5wamsz's other two LLRs are elsewhere: `LLR-z8rrkr` is T7b (the file that
tests the emitting address) and `LLR-b4p32h` is T9 (the file that tests the
discarded head reference). An item's LLRs go to the task that owns the test
file, not to the task named after the item.

  **LLR-xv7auy**: each on-chain field is a public newtype whose width is the
  ABI's — `OnChainRootHash` and `OrgPubKey` thirty-two bytes, `Epoch` a `u64`,
  `OrgAdmin` twenty. satisfies: REQ-2qa5r5

  *(Text as it stands after the 2026-09-28 falsifiability sweep cut the
  distinctness clause; see the findings table. The mutation below is unchanged
  by that cut — it was always the width assertion it reds.)*

Add `LLR-xv7auy` to the existing `verifies: REQ-2qa5r5` annotation on
`newtypes_have_the_widths_the_abi_gives_them` (line ~124). Leave
`epoch_display_is_the_inner_value` unannotated — the file's header says
deliberately why, and that decision is not this task's to revisit.

**Mutation — corrected 2026-09-28 after T2 ran.** The mutation this plan first
named, changing `pub struct OrgAdmin(pub [u8; 20]);` to `(pub [u8; 32]);`,
**does not discharge this LLR** and must not be used. It reds the *library*,
not the test: `src/decode/v_paseo_ah.rs:157` and `:187` fail to compile with
E0308 (`admin: OrgAdmin(admin)`, expected size 32 found 20), so
`tests/type_widths.rs` never compiles and no assertion in it is ever reached. A
compile error somewhere else is not this test being watched fail.

The mutation that does discharge it, and the one T2 used, is
`#[repr(align(32))]` on `pub struct OrgAdmin(pub [u8; 20]);` in
`on-chain-client/src/types.rs`. **Expect:**
`newtypes_have_the_widths_the_abi_gives_them` fails at its own assertion —
"OrgAdmin must be exactly the ABI address, with nothing else in it", left 32,
right 20 — because `size_of::<OrgAdmin>()` widens while the inner `[u8; 20]`
stands. It is in the same file the original named, and it is the same widening
`tests/type_widths.rs`'s own header records as the red watched when the target
was created.

---

### T3 — SDD-5b8wxs: account-to-address derivation

**Files touched:** `on-chain-client/tests/h160_mapping.rs`
**Parallel:** yes

  **LLR-2yhra8**: `h160_of` returns the account identifier's first twenty bytes
  when all twelve marker positions hold the EVM-fallback marker byte.
  satisfies: REQ-tkhe3u

  **LLR-3bkhuc**: `h160_of` returns the last twenty bytes of the keccak-256 of
  the whole thirty-two-byte identifier whenever any one marker position does not
  hold the marker byte. satisfies: REQ-rz7fja

  **LLR-7pjzjn**: the marker positions are bytes twenty through thirty-one
  inclusive and the marker byte is `0xEE`, so the branch is decided by those
  twelve bytes and no others. satisfies: REQ-tkhe3u, REQ-rz7fja

Annotations:
- `reverse_path_returns_the_first_twenty_bytes` → add `LLR-2yhra8`
- `reverse_path_taken_when_the_whole_account_is_the_marker_byte` → add `LLR-2yhra8`, `LLR-7pjzjn`
- `forward_path_keccaks_then_truncates` → add `LLR-3bkhuc`
- `forward_path_taken_when_only_eleven_marker_bytes_are_present` → add `LLR-3bkhuc`, `LLR-7pjzjn`
- `forward_path_taken_for_a_single_non_marker_byte_at_each_marker_position` → add `LLR-3bkhuc`, `LLR-7pjzjn`

**Mutations** (three, one per LLR — each reverted and digest-checked before the
next):
- LLR-2yhra8: in `src/h160.rs`, change `h160.copy_from_slice(&account_id_32[..20]);`
  to `&account_id_32[1..21]`. **Expect:** `reverse_path_returns_the_first_twenty_bytes` fails.
- LLR-3bkhuc: change `h160.copy_from_slice(&hash[12..32]);` to `&hash[0..20]`.
  **Expect:** `forward_path_keccaks_then_truncates` fails.
- LLR-7pjzjn: change `account_id_32[20..32]` to `account_id_32[21..32]`.
  **Expect:** `forward_path_taken_when_only_eleven_marker_bytes_are_present`
  fails — an eleven-marker account now takes the reverse path.

---

### T4 — SDD-bw7v5x: Solidity storage-slot derivation

**Files touched:** `on-chain-client/tests/storage_slot_layout.rs`
**Parallel:** yes

  **LLR-vktf8w**: `solidity_mapping_slot` returns the keccak-256 of a sixty-four
  byte buffer holding the Organisation admin left-padded into bytes twelve
  through thirty-one and the map's slot index in the second word.
  satisfies: REQ-9m2rnd

  **LLR-62tqnv**: the map's slot index occupies the low eight bytes of that
  second word, big-endian, so a different declared map slot yields a different
  key for the same admin. satisfies: REQ-9m2rnd

  **LLR-bhwsn6**: `increment_slot` adds its offset to a thirty-two byte
  big-endian slot identifier, propagating carry from the least significant
  byte upward. satisfies: REQ-xudf25

  **LLR-2y9qdc**: an increment that carries out of the most significant byte
  wraps to zero rather than panicking or saturating. satisfies: derived

  **LLR-v62yjq**: the three fields of an Organisation slot are read at the base
  key and the two keys above it, consecutive and pairwise distinct.
  satisfies: REQ-9m2rnd, REQ-xudf25

Annotations:
- `mapping_slot_matches_the_known_vector` → `LLR-vktf8w`
- `mapping_slot_for_a_second_admin_is_its_own_key` → `LLR-vktf8w`
- `the_zero_admin_is_hashed_like_any_other_address` → `LLR-vktf8w`
- `a_non_zero_mapping_slot_index_is_part_of_the_key` → `LLR-62tqnv`
- `the_mapping_slot_index_is_big_endian_in_the_low_eight_bytes_of_the_second_word` → `LLR-62tqnv`
- `offset_zero_is_the_identity` → `LLR-bhwsn6`
- `offset_three_carries_out_of_the_low_byte` → `LLR-bhwsn6`
- `a_carry_propagates_through_four_bytes` → `LLR-bhwsn6`
- `offset_255_is_the_widest_a_u8_can_ask_for` → `LLR-bhwsn6`
- `an_all_ones_slot_wraps_to_zero` → `LLR-2y9qdc`
- `the_three_struct_field_slots_are_consecutive_and_distinct` → `LLR-v62yjq`

**Mutations** (in `src/client.rs`'s `internals`):
- LLR-vktf8w: `buf[12..32].copy_from_slice(&admin.0);` → `buf[0..20]`.
  **Expect:** `mapping_slot_matches_the_known_vector` fails.
- LLR-62tqnv: `buf[56..64].copy_from_slice(&map_slot.to_be_bytes());` →
  `map_slot.to_le_bytes()`. **Expect:**
  `the_mapping_slot_index_is_big_endian_in_the_low_eight_bytes_of_the_second_word`
  fails.
- LLR-bhwsn6: in `increment_slot`, change `if carry == 0 { break; }` to
  `break;` unconditionally. **Expect:** `a_carry_propagates_through_four_bytes` fails.
- LLR-2y9qdc: change `*byte = sum as u8;` to `*byte = sum.min(255) as u8;`.
  **Expect:** `an_all_ones_slot_wraps_to_zero` fails.
- LLR-v62yjq: change the `for offset in 0u8..3u8` loop bound in
  `get_org_state` — **no.** That path is not gate-reachable. Instead mutate
  `increment_slot`'s `carry = u16::from(offset)` to `carry = 0`. **Expect:**
  `the_three_struct_field_slots_are_consecutive_and_distinct` fails, because
  the three slots collapse onto one.

---

### T5 — SDD-v2rtka: decoder selection

**Files touched:** `on-chain-client/tests/runtime_version_dispatch.rs`
**Parallel:** yes

  **LLR-b3s7st**: `for_runtime` resolves a Runtime spec version equal to the
  pinned one to the decoder compiled for that version, and to no other.
  satisfies: REQ-hd6m9d

  **LLR-u8ajby**: `for_runtime` returns `UnsupportedRuntime` naming the version it
  was asked about for every other Runtime spec version, including the ones
  immediately below and above the pinned one and the extremes of the range —
  never a nearest match and never a default. satisfies: REQ-hd6m9d

Annotations: `pinned_version_resolves_to_a_decoder_that_decodes_the_pinned_layout`
→ `LLR-b3s7st`; the four refusal tests
(`version_one_below_the_pinned_one_is_refused`,
`version_one_above_the_pinned_one_is_refused`, `zero_version_is_refused`,
`max_version_is_refused`) → `LLR-u8ajby`.

**Mutations** (in `src/decode/dispatch.rs`):
- LLR-b3s7st: change the match arm `PASEO_AH_SPEC_VERSION => Ok(&v_paseo_ah::DECODER)`
  to return `Err(DecodeError::UnsupportedRuntime { spec_version })`.
  **Expect:** `pinned_version_resolves_to_a_decoder_that_decodes_the_pinned_layout` fails.
- LLR-u8ajby: change the `_` arm to `_ => Ok(&v_paseo_ah::DECODER)` — the
  fall-back-to-a-guess defect the requirement forbids. **Expect:** all four
  refusal tests fail.

---

### T6 — SDD-f2s7bx: the state decoder

**Files touched:** `on-chain-client/tests/decode_org_state.rs`
**Parallel:** yes

  **LLR-sq76u3**: `decode_org_state` accepts exactly ninety-six bytes and refuses
  every other length — shorter, longer, empty, or a whole number of slots other
  than three — with `StorageLengthMismatch` naming both the expected and the
  actual length. satisfies: REQ-4astjb

  **LLR-nq7nhg**: the ninety-six bytes are read as the Membership root in the
  first slot, the Organisation's signing key in the second and the Epoch in the
  third, each at its own thirty-two byte offset. satisfies: REQ-4astjb

  **LLR-emp3g9**: the Epoch slot is a big-endian `uint256` whose high twenty-four
  bytes must all be zero; a non-zero byte anywhere in them is `EpochOverflow`,
  never a truncation to the low eight. satisfies: REQ-9wwenn

Annotations:
- `exactly_ninety_six_bytes_decodes_every_field` → `LLR-nq7nhg`
- `ninety_five_bytes_is_rejected`, `ninety_seven_bytes_is_rejected`,
  `an_empty_blob_is_rejected_rather_than_read_out_of_bounds`,
  `a_whole_number_of_slots_other_than_three_is_rejected` → `LLR-sq76u3`
- `a_non_zero_byte_anywhere_in_the_epoch_slots_leading_twenty_four_bytes_is_refused`,
  `the_largest_u64_epoch_is_accepted_whole`, `a_zero_epoch_is_accepted`,
  `each_of_the_low_eight_bytes_carries_its_big_endian_weight` → `LLR-emp3g9`

**Mutations** (in `src/decode/v_paseo_ah.rs`):
- LLR-sq76u3: change `if bytes.len() != 96` to `if bytes.len() < 96`.
  **Expect:** `ninety_seven_bytes_is_rejected` fails.
- LLR-nq7nhg: swap the two copies — read `root_hash` from `&bytes[32..64]` and
  `org_pub_key` from `&bytes[0..32]`. **Expect:**
  `exactly_ninety_six_bytes_decodes_every_field` fails.
- LLR-emp3g9: in `decode_uint256_to_u64`, delete the
  `if bytes[..24].iter().any(...)` guard so the high half is ignored.
  **Expect:**
  `a_non_zero_byte_anywhere_in_the_epoch_slots_leading_twenty_four_bytes_is_refused`
  fails.

---

### T7 — SDD-d5jh6t: the event decoder (annotations plus one new test)

**Files touched:** `on-chain-client/tests/decode_revive_event.rs`
**Parallel:** yes

  **LLR-e6skvu**: exactly two Event signature values are recognised, each the
  keccak-256 of one of the two canonical Solidity signature strings the deployed
  contract declares. satisfies: REQ-52uc8f

  **LLR-rjcqg3**: a log whose first topic matches neither, and a log carrying no
  topics at all, yield no event and no error. satisfies: REQ-wnjz9j

  **LLR-n6gghu**: `GenesisInitialized` requires exactly two topics and sixty-four
  data bytes and `RootUpdated` exactly three and ninety-six; any other count or
  length is `InvalidTopicCount` or `InvalidDataLength` naming the event, the
  expected value and the actual. satisfies: REQ-88fp2h

  **LLR-89pdz9**: an indexed address topic is refused as `InvalidAddressTopic`
  when any of its twelve leading pad bytes is non-zero, and yields the topic's
  low twenty bytes when all twelve are zero. satisfies: REQ-twdu84

  **LLR-8242kq**: a payload carrying bytes beyond the three declared fields is
  refused naming the trailing count, and one ending inside a field or inside a
  field's own length prefix is refused. satisfies: REQ-axcxf7

  **LLR-6tjhgk**: a structurally valid event decodes back to every field it
  carried — both event shapes, the Emitting contract included — without loss,
  reordering or defaulting. satisfies: REQ-n6v896

  **LLR-u2e389**: the Emitting contract is read from the `ContractEmitted`
  payload and returned to the caller alongside the decoded event rather than
  dropped. satisfies: REQ-5upq6n

  **LLR-mzh8df**: `RootUpdated`'s indexed Epoch topic is bounded to the range
  this reader represents, so an on-chain Epoch above it is `EpochOverflow`
  rather than a truncation. satisfies: REQ-9wwenn

  *(Text as it stands after the 2026-09-28 falsifiability sweep cut the
  "same rule as the storage path" clause; see the findings table. The mutation
  below is unchanged by that cut — it was always the truncation it reds, never
  the sharing.)*

Annotations:
- `the_recognised_signature_hashes_are_the_keccak_of_the_canonical_solidity_strings` → `LLR-e6skvu`
- `an_unknown_first_topic_yields_nothing`, `no_topics_at_all_yields_nothing` → `LLR-rjcqg3`
- the six count/length refusals (`genesis_with_one_topic_is_rejected`,
  `genesis_with_three_topics_is_rejected`,
  `root_updated_with_two_topics_is_rejected`,
  `root_updated_with_four_topics_is_rejected`,
  `genesis_data_of_any_length_but_sixty_four_is_rejected`,
  `root_updated_data_of_any_length_but_ninety_six_is_rejected`) → `LLR-n6gghu`
- `a_non_zero_byte_at_each_padding_position_of_the_address_topic_is_rejected`,
  `all_twelve_padding_bytes_zero_is_accepted_and_the_address_is_the_low_twenty_bytes` → `LLR-89pdz9`
- the three truncation/trailing refusals → `LLR-8242kq`
- `genesis_initialized_round_trips_every_field`,
  `root_updated_round_trips_every_field` → `LLR-6tjhgk`, `LLR-u2e389`

**The one new test.** `root_updated_epoch_above_u64_is_refused_not_truncated`,
carrying `verifies: LLR-mzh8df`. Build a well-formed `RootUpdated` log whose
`topics[2]` is a `uint256` with a non-zero byte in its high twenty-four — the
file's existing `root_updated_data` and topic helpers give the rest — and
assert the result is `Err(DecodeError::EpochOverflow)`. It will pass on first
run, because the behaviour is already there: `parse_root_updated` calls the
same `decode_uint256_to_u64`. That is why it is discharged by mutation like
every other LLR here, and why it is worth adding — nothing at this gate
exercises that bound through the event path today.

**Mutations** (in `src/decode/v_paseo_ah.rs`):
- LLR-e6skvu: flip one byte of `SIG_ROOT_UPDATED`. **Expect:**
  `the_recognised_signature_hashes_are_the_keccak_of_the_canonical_solidity_strings` fails.
- LLR-rjcqg3: change the `_ => return Ok(None)` match arm to
  `_ => return Err(DecodeError::InvalidAddressTopic)`. **Expect:**
  `an_unknown_first_topic_yields_nothing` fails.
- LLR-n6gghu: in `parse_genesis`, change `if data.len() != 64` to
  `if data.len() < 64`. **Expect:**
  `genesis_data_of_any_length_but_sixty_four_is_rejected` fails.
- LLR-89pdz9: in `unpack_address_topic`, change `topic[..12]` to `topic[..11]`.
  **Expect:**
  `a_non_zero_byte_at_each_padding_position_of_the_address_topic_is_rejected` fails.
- LLR-8242kq: delete the `if !bytes.is_empty()` trailing-bytes guard.
  **Expect:** `any_trailing_bytes_after_a_well_formed_payload_are_rejected` fails.
- LLR-6tjhgk: in `parse_root_updated`, copy `prev_root_hash` from
  `&data[32..64]` instead of `&data[64..96]`. **Expect:**
  `root_updated_round_trips_every_field` fails.
- LLR-u2e389: in `parse_revive_event`, return `EmittedEvent { contract: [0u8; 20], event }`.
  **Expect:** `genesis_initialized_round_trips_every_field` fails.
- LLR-mzh8df: in `parse_root_updated`, replace `decode_uint256_to_u64(&topics[2])?`
  with a direct read of the low eight bytes of `topics[2]`. **Expect:** the new
  `root_updated_epoch_above_u64_is_refused_not_truncated` fails. This mutation
  is the reason the test is worth its keep: it is the truncation the
  requirement forbids, and nothing else in the suite catches it.

---

### T7b — SDD-5wamsz and SDD-d5jh6t: the emitting address reaches the caller

**Files touched:** `on-chain-client/tests/contract_address_filter.rs`
**Parallel:** yes

This task exists because `LLR-z8rrkr` and `LLR-u2e389` are two halves of one
property — the decoder must *report* the Emitting contract (SDD-d5jh6t) and the
value type must *carry* it as one value with the event (SDD-5wamsz) — and this
file is the one that tests it end to end. It was missing from the first cut of
this plan; `check-ids.sh` did not catch that, because a missing task is not a
malformed one. `check-trace.sh`'s MISSING-TEST is what would have caught it, at
the gate, after every other task had run.

  **LLR-z8rrkr**: a decoded event is carried together with the twenty-byte
  address of the contract that emitted it, as one value, so that the address
  cannot be dropped between decoding and the admission decision that needs it.
  satisfies: REQ-5upq6n

Annotations in this file (it carries `verifies: REQ-5upq6n` at the header and on
three tests):
- `log_from_the_configured_contract_reports_that_address` → add `LLR-z8rrkr`, `LLR-u2e389`
- `the_emitting_address_is_reported_byte_for_byte_for_both_event_shapes` → add `LLR-z8rrkr`, `LLR-u2e389`
- `log_from_another_contract_with_a_valid_signature_is_rejected` → add `LLR-z8rrkr`

**Mutation** (in `on-chain-client/src/state.rs`): this LLR is about a value's
shape, so mutate the shape. Change `EmittedEvent`'s `pub contract: [u8; 20]` to
be populated from a constant rather than the decoded value — concretely, in
`src/decode/v_paseo_ah.rs`'s `parse_revive_event`, construct
`EmittedEvent { contract: [0u8; 20], event }`. **Expect:**
`the_emitting_address_is_reported_byte_for_byte_for_both_event_shapes` fails.

Note this is the same mutation T7 uses for `LLR-u2e389`. That is correct and not
a duplication to resolve: one mutation can red two LLRs when the two state the
same defect from the decoder's side and from the value type's side. Run it once
per task, in that task's own worktree, and report it in both — the digests prove
each task reverted its own copy.

### T8 — SDD-4z3k2u: subscriber admission

**Files touched:** `on-chain-client/tests/log_ownership.rs`
**Parallel:** yes

  **LLR-2znra8**: a decoded log is refused unless the contract that emitted it is
  the one the reader was constructed for, and that comparison is made before and
  independently of any admin filter, so a matching admin never rescues a foreign
  log. satisfies: REQ-9vwcwc

  **LLR-kfr75c**: with no admin filter set every log from the configured contract
  is admitted; with one set only a log whose Organisation admin equals the named
  one is. satisfies: REQ-nygs7k

  **LLR-9qp3k7**: the Organisation admin compared against the filter is read from
  both event shapes alike. satisfies: REQ-nygs7k

Annotations:
- `a_log_from_the_configured_contract_with_no_filter_is_ours`,
  `the_spoof_a_valid_log_from_another_contract_is_not_ours`,
  `an_address_differing_in_one_byte_at_either_end_is_not_ours`,
  `the_contract_check_dominates_a_matching_admin_filter` → `LLR-2znra8`
- `with_a_filter_set_a_matching_admin_is_ours`,
  `with_a_filter_set_a_non_matching_admin_is_not_ours`,
  `with_no_filter_any_admin_from_the_configured_contract_is_ours` → `LLR-kfr75c`
- `with_a_filter_set_a_matching_admin_is_ours` also → `LLR-9qp3k7`

**Mutations** (in `src/client.rs`'s `internals::log_is_ours` and `event_admin`):
- LLR-2znra8: delete the `if emitted.contract != *configured_contract` guard.
  **Expect:** `the_spoof_a_valid_log_from_another_contract_is_not_ours` fails.
  This is HAZ-werm85's own mutation — the spoof the whole item exists to refuse.
- LLR-kfr75c: change `None => true` to `None => false`. **Expect:**
  `with_no_filter_any_admin_from_the_configured_contract_is_ours` fails.
- LLR-9qp3k7 — **corrected 2026-09-28 after T8 ran.** Both mutations offered
  here first are unwritable. Binding a different field is E0308: `admin` is the
  only `OrgAdmin`-typed field in `Event::Update` and `event_admin` returns
  `&OrgAdmin`, so this reds a compile error, which is the failure mode T2's
  correction forbids. Returning the `Genesis` arm's admin is unwritable too —
  the arms bind disjoint values of one borrowed enum and no `Genesis` admin is
  in scope. **The mutation T8 used, and the one to repeat:** introduce
  `const NOT_THE_EVENTS_ADMIN: OrgAdmin = OrgAdmin([0u8; 20]);` in
  `event_admin`'s body and change the `Update` arm to
  `Event::Update { .. } => &NOT_THE_EVENTS_ADMIN`, leaving `Genesis` untouched.
  **Expect:** `with_a_filter_set_a_matching_admin_is_ours` fails at its own
  assertion — a genuine `RootUpdated` for the filtered admin is refused while
  the `Genesis` half of the same test still passes, which is the asymmetry the
  LLR asserts against.

---

### T9 — SDD-m59zrg: the best-head scan

**Files touched:** `on-chain-client/tests/best_lane_reorg_rule.rs`
**Parallel:** yes

  **LLR-b4p32h**: a head is carried as a reference holding both its hash and its
  number, so a discarded head can be named by both. satisfies: REQ-ntn4ss

  **LLR-d3ef7s**: a notification whose hash equals the last processed head's
  yields nothing and leaves the last processed head unchanged, whatever number it
  carries. satisfies: REQ-gr2ver

  **LLR-56bzcj**: a reorg is reported when a head has already been processed and
  the new head is either at or below its height, or at the next height with a
  parent other than it — and not otherwise, including for the first notification.
  satisfies: REQ-ntn4ss

  **LLR-48ygak**: the heights read for a new head are the one after the last
  processed head up to and including the new head's, or the new head's height
  alone when it is at or below the last processed head's or none has been
  processed, and the span is always ascending. satisfies: REQ-5zux82

Annotations:
- `repeated_head_is_skipped_and_last_is_unchanged`,
  `repeated_hash_with_inconsistent_number_is_still_skipped`,
  `same_height_different_hash_is_not_a_repeat` → `LLR-d3ef7s`
- the six reorg tests → `LLR-56bzcj`
- `the_discarded_reference_carries_both_hash_and_number` → also `LLR-b4p32h`
- the five backfill tests → `LLR-48ygak`

**Mutations** (in `src/client.rs`'s `internals::scan_step`):
- LLR-d3ef7s: change the dedup arm `Some(prev) if prev.hash == h` to
  `Some(prev) if prev.hash == h && prev.number == n`. **Expect:**
  `repeated_hash_with_inconsistent_number_is_still_skipped` fails.
- LLR-56bzcj: drop the `n <= prev.number` disjunct from the reorg condition.
  **Expect:** `a_rewind_below_the_last_height_reports_the_discarded_head` fails.
- LLR-48ygak: change `prev.number + 1` in the `from` computation to
  `prev.number`. **Expect:** `a_jump_backfills_every_skipped_height` fails.
- LLR-b4p32h — **corrected 2026-09-28 after T9 ran, and this is the most
  dangerous of the three plan errors.** The mutation named here first —
  writing `number: 0` into `scan_step`'s `*last = Some(BlockRef { hash: h,
  number: n })` — **compiles, runs, and leaves the named test GREEN.**
  `reorged` is computed from the OLD value of `*last` before that write
  happens, and `the_discarded_reference_carries_both_hash_and_number` asserts
  only on the returned discarded reference, never on `last` afterwards. A
  mutation that does not red discharges nothing; had it been accepted on the
  strength of "the mutation was applied", the LLR would carry an attestation
  for evidence that was never produced. T9 applied it, watched the test pass,
  and reverted it — that non-result is itself part of the record.
  **The mutation T9 substituted, and the one to repeat:** change the reorg
  arm's body from `Some(prev)` to `Some(BlockRef { hash: prev.hash, number: 0 })`.
  **Expect:** `the_discarded_reference_carries_both_hash_and_number` fails at
  `assert_eq!(discarded.number, 7_654_321)`, left 0 — the discarded head named
  by hash alone with its number defaulted, which is the shape defect the LLR
  forbids.

---

### T10 — the fuzz targets

**Files touched:** `on-chain-client/tests/fuzz_decode_org_state/fuzz_target.rs`,
`on-chain-client/tests/fuzz_parse_revive_event/fuzz_target.rs`,
`on-chain-client/tests/fuzz_event_round_trip/fuzz_target.rs`
**Parallel:** yes

  **LLR-2v5u4d**: no sequence of bytes offered to the event decoder causes a
  panic or an abort; every rejection is a typed error. satisfies: REQ-sx5b6g

  **LLR-yhw34z**: no sequence of bytes offered to the state decoder causes a
  panic or an abort; every rejection is a typed error. satisfies: REQ-sx5b6g

Add `LLR-2v5u4d` to `fuzz_parse_revive_event`'s header, `LLR-yhw34z` to
`fuzz_decode_org_state`'s. `fuzz_event_round_trip` carries
`verifies: REQ-n6v896` and gains `LLR-6tjhgk` (T7's round-trip LLR) — coordinate
nothing with T7, the files are disjoint and the annotation text is fixed here.

**These targets are `harness = false`.** They print an iteration count and an
exit reason, never a `test result` line, and a panic is the only failure
signal. Read the verdict from the absence of a panic and from the iteration
count being non-zero — an exit 0 with zero iterations is not a pass.

**Mutations.** Two entries, not three: `LLR-6tjhgk` is annotated in this task
but its mutation is specified under T7 (`prev_root_hash` copied from
`&data[32..64]`). Use that one rather than inventing a second. An LLR annotated
in one task and mutated in another is the arrangement T7b describes and is fine.

- LLR-2v5u4d: in `parse_revive_event`, replace the `Decode::decode(&mut bytes)`
  for `topics` with an indexing read that can go out of bounds, or change
  `unpack_address_topic`'s slice to `topic[..40]`. **Expect:** the fuzz target
  panics within its budget.
- LLR-yhw34z: in `decode_org_state`, delete the length guard so
  `bytes[0..32]` is read unconditionally. **Expect:**
  `fuzz_decode_org_state` panics on a short input within its budget.

Report the iteration count at which each panicked, and the panic message.

---

### T11 — SDD-3b8zef: the item with no LLRs, and its deviation

**Files touched:**
`on-chain-client/docs/architecture/2026-09-28-decomposition.md`
**Parallel:** no (serial, last — it records what T2–T10 did and did not reach)

Not dispatched; written by the plan-holder, after the other tasks report.

**SDD-3b8zef** is the chain-facing transport shell: `OrgRegistryClient`'s
construction and decoder pinning, `get_org_state`'s three-slot read,
`read_contract_slot`'s runtime-API call, `subscribe` and both of its lanes,
`events_in_best_block` and `decode_contract_events`. The majority of
`client.rs`, of which llvm-cov analyses 336 lines out of the 491 it analyses
across this crate, at 16.37% line coverage. (The hand-computed "roughly 580 of
699" was dropped 2026-09-28 by this change's review sweep as unreproducible;
the decomposition file states why.) It traces to REQ-9vwcwc, REQ-5zux82 and
REQ-hd6m9d — it is what actually delivers to a subscriber, reads the span of
heights, and pins the decoder at construction.

**It carries no LLRs, and that is a deviation from class C, recorded as one.**
Write it plainly, in the item's own text and in a section of its own:

- The obligation: IEC 62304 class C requires low-level requirements per
  software item, verified at that item's own interface.
- The state: this item has none, because **nothing at this unit's gate can
  verify one.** Its coverage is 16.37% of lines; the five integration targets
  that do exercise it — `off_chain_genesis_ceremony`, `p_address_is_orgid`,
  `reorg_cancels_proposed`, `scenario_a_full`, `two_orgs_one_watcher` — need a
  chopsticks fork and run in no gate. The three decisions
  that *were* made gate-testable — `log_is_ours`, `scan_step`, the slot
  arithmetic — were extracted into `internals` by an
  earlier change and are SDD-4z3k2u, SDD-m59zrg and SDD-bw7v5x above. What
  remains is async subxt transport code.
- Why the alternative was refused: LLRs here could be annotated onto those nine
  ungated targets and `check-trace.sh` would go green, because MISSING-TEST
  reads a `verifies:` reference in a file under `test_paths` and not an
  executed assertion. That would be a green gate over evidence that was never
  produced. It was put to the owner as a decision on 2026-09-28 and refused.
- What closes it: either the extraction continues until the remaining decisions
  are chain-free and testable, or this unit's gate gains a chopsticks harness.
  Both are code changes owed their own red-first cycle, and neither belongs in
  a documentation tooth.

File the same thing as an owner item in `docs/plans/2026-09-05-ratchet-setup.md`
so it is carried in the register with the other open decisions, not only in a
ledger file. Cross-reference it from the coverage shortfall already tracked
there — they are the same gap measured two ways, and the register should say so
rather than list them as two unrelated items.

---

## Self-review

1. **Every Implements ID has a task whose test verifies it.** The nine SDD
   items are documentation items carrying `traces:`, verified by
   `check-trace.sh`'s UNTRACED-DESIGN gate, not by tests. All thirty-two LLRs
   are annotated in T2–T10; SDD-3b8zef deliberately has none and T11 records
   why. No problem report is minted — the one this change would have written
   already exists as PR-h4mb8y.
2. **Real code, real commands, real expected output** — each task names the
   mutation, the file, the test and the failure expected.
3. **Consistent names** — LLR text here is the text that goes in the ledger;
   annotations quote the IDs minted on 2026-09-28.
4. **Files touched and Parallel on every task, no two parallel tasks sharing a
   file.** T2–T10 touch nine disjoint test files (T10 owns all three fuzz
   targets). T1 and T11 touch ledger files only and are serial, first and last.
   **No task touches `on-chain-client/src`** — every mutation is applied and
   reverted inside the task, and the file is proved byte-identical afterwards
   by digest. A task whose digest does not match has not finished.

## Execution

T1, then T2–T10 (T7b included) fanned out at most five at a time, then T11. Then
`check-traceability`, `verify-before-merge`, `merge-change`.

## What the dispatches found in this plan

Four of the mutations written here were wrong, and every one was caught by the
task that ran it rather than by a gate. They are corrected in place above; they
are collected here because the pattern matters more than the four fixes. A
fifth row was added by review round 2, and two more by the 2026-09-28
falsifiability sweep; those three are not wrong mutations, and the paragraphs
under the table say what they are instead.

| LLR | What the plan named | Why it failed |
|---|---|---|
| LLR-xv7auy (T2) | widen `OrgAdmin`'s inner array to 32 | Reds the **library** — E0308 at `v_paseo_ah.rs:157`/`:187` — so the test never compiles and no assertion is reached |
| LLR-9qp3k7 (T8) | bind a different field in `event_admin`'s `Update` arm | E0308 again: `admin` is the only `OrgAdmin`-typed field, and the offered alternative is unwritable across disjoint match arms |
| LLR-b4p32h (T9) | write `number: 0` into `scan_step`'s `*last` | **Compiles, runs, and the named test stays GREEN** — `reorged` reads the old `*last` before that write |
| — (T10) | three mutations listed for two LLRs | `LLR-6tjhgk`'s mutation lives under T7; the list was miscounted |
| LLR-bhwsn6 (review round 2) | `if carry == 0 { break; }` → unconditional `break` | The mutation is sound and still stands, but it never tested the clause the LLR added to it. **The LLR's stopping clause was cut on 2026-09-28 as unfalsifiable**: deleting the early exit outright leaves `--test storage_slot_layout` at 11 passed, 0 failed, because once `carry == 0` every remaining byte is `byte + 0`. What the mutation reds is the *propagation*, which the LLR still claims, so its attestation is unaffected |
| LLR-xv7auy (sweep, 2026-09-28; **reversed by review round 5**) | *the clause* "distinct … so that two fields of equal width cannot be substituted for one another" | **Cut, then restored.** The sweep cut it because one probe — replacing `pub struct OrgPubKey(pub [u8; 32]);` in `src/types.rs` with `pub use crate::types::OnChainRootHash as OrgPubKey;` — leaves `verify_commands` green at 73 passed. But that probe also deletes the *name*, so the assertions judging it can no longer run. Round 5 measured a probe that performs the same forbidden substitution and leaves the tests able to observe it: retype the `org_pub_key` field from `OrgPubKey` to `OnChainRootHash` at three sites in `src/state.rs` and three in `src/decode/v_paseo_ah.rs`, touching neither `types.rs` nor `lib.rs`. The library builds clean; the gate exits **101** with eight `E0308` across five targets. **RED, and the clause is back in the LLR.** T2 attestation unaffected either way |
| LLR-mzh8df (sweep, 2026-09-28) | *the clause* "by the same `uint256`-to-`u64` rule as the storage path" | **Cut as unfalsifiable.** Replacing the `decode_uint256_to_u64(&topics[2])?` call in `parse_root_updated` with a byte-for-byte identical check written inline leaves `verify_commands` at **73 passed, 0 failed** — no gated test can observe whether the two paths share a function. T7's attestation is unaffected: the mutation it names is the direct low-eight-byte read, which reds `root_updated_epoch_above_u64_is_refused_not_truncated`, and "refused rather than truncated" is what the LLR still claims |

The LLR-bhwsn6 row is review round 2's rather than a dispatch's, and it is a
defect in a low-level requirement rather than in a mutation: the property was
written into the LLR and nothing at this gate — including the mutation named for
it — could ever have observed it. It was the second clause cut from this ledger
on that ground; round 1 cut the ordering clause from LLR-2znra8 for the same
reason, and the requirements ledger had already cut it from REQ-9vwcwc and
RC-5e3bdk.

The last two rows are the 2026-09-28 **falsifiability sweep**, which stopped
waiting for review round four to find a sixth such clause and instead put every
clause of all thirty-two LLRs through the same measurement. Its result changed
twice afterwards and the current figures are in the decomposition, under "The
falsifiability sweep of 2026-09-28": **one hundred** independently falsifiable
claims, all one hundred probed, **99 RED (96 at runtime, 3 at compile time) and
1 GREEN**. Review round 4 corrected the classification rule the sweep used, and
review round 5 found that the corrected rule invalidated one of the two cuts —
LLR-xv7auy's distinctness clause is falsifiable and has been restored. Only
LLR-mzh8df's code-sharing clause remains cut. **No mutation named in this plan
and no line of attestation below changed at any point.**

**Three of the four are the same mistake:** a mutation aimed at a property was
written against the *source line nearest the words of the LLR* rather than
against the line the named test actually observes. Widening a type reds whoever
constructs it. Changing a match arm's binding reds the type checker. Writing to
a variable reds nothing at all if the assertion was taken before the write.

The third is the one worth remembering, because it is the only one that fails
silently. A mutation that will not compile announces itself. A mutation that
compiles, runs, and leaves the test green looks exactly like work completed —
and had the discipline been "apply the mutation, then annotate", LLR-b4p32h
would now carry a red-by-mutation attestation for a red that never happened.
What caught it was the requirement to **watch the named test fail and report
why**, which cannot be satisfied by a test that passed.

That is the same defect this repository has now recorded three times in three
forms: a CI workflow asserted to run that has never executed, a hazard register
arguing from that CI, and here a mutation asserted to red that does not. Each
time the gate was green and the evidence absent. The remedy is the same each
time — state what was observed, not what was expected.

## Task completion and red → green attestations

All eleven tasks done. Every LLR below was discharged by applying the named
mutation, watching the named test fail for that reason, reverting, and proving
the source byte-identical by SHA-256. **Five source files were mutated by the
tasks and the first sweep** (rounds 4 and 5 later mutated three more, which the
sweep bullets below record), and each holds one digest across every cycle in
every task that touched it:

| File | SHA-256, before every mutation and after every revert |
|---|---|
| `src/types.rs` | `d951091a3f461264a85c71173571ccf7a1d83c7f5ef49ea004d5ed1a46ee5832` |
| `src/h160.rs` | `e46ff22701997c1003c86752d9c0ca141a438d07bc932ad0f5e800f9fc6755bc` |
| `src/decode/dispatch.rs` | `b60cceb69528f2a2b086344059b28d75d4371cfb04c1b4dc027777acc73e2000` |
| `src/decode/v_paseo_ah.rs` | `d9557564ffd373c6d464022a7360c29495c893f8cd2522eda747dfee6bb1ca99` |
| `src/client.rs` | `9527992d6e2a26e783132be301c6a74caa30090754b9dd9ca590892323f96e68` |

`src/client.rs`'s digest was reported independently by T4, T8 and T9, and
`src/decode/v_paseo_ah.rs`'s by T6, T7, T7b and T10 — three and four tasks
arriving at the same value is the cross-check that no task left drift behind.

**Seven of the thirty line citations below were corrected on 2026-09-28 by
review round 4, and the cause is worth recording because it will recur.** Each
attestation cites the failing assertion as `file.rs:NN`. Those numbers were
captured while the task ran — *before* this change's own header annotations were
written into the same test files. Annotating `best_lane_reorg_rule.rs` and
`log_ownership.rs` added four net lines to each, so every citation into those two
files below the header was left four lines short, still resolving correctly
against `master` and incorrectly against the tree the attestation ships in.

Two of the seven were worse than a stale number. `log_ownership.rs:143` landed on
a string-literal continuation of a *self-check* (`assert_eq!(spoof.event,
genuine.event)`) that the named mutation cannot fail, and `:234` landed on the
`Genesis` assert rather than the `RootUpdated` one the mutation reds. A reader
checking either would have concluded the attestation was false rather than the
line number wrong — the citation actively argued against its own evidence.

**And the fix committed the same defect, in the same commit that wrote the
lesson.** Review round 5 found it. The commit that corrected those seven also
rewrote `type_widths.rs`'s header — **+2 net lines** — and re-resolved nothing
in that file, so five further citations moved by two and were not updated:
`type_widths.rs:143` in this plan and in the decomposition, and `:166`, `:167`,
`:175` in the decomposition. Two of the five then landed on lines the named
mutation cannot fail — a string-literal continuation of a const-vs-const
self-check, and the closing brace of a `for` loop — which is exactly the "argues
against its own evidence" failure catalogued above. The remediation paragraph
had said the test headers were "corrected line-for-line so the attestation
citations into those files do not move again"; that was true of
`best_lane_reorg_rule.rs` and `log_ownership.rs`, which were 3-for-3 line
replacements, and simply not checked for the third file edited in the same
commit.

All thirty were then re-resolved against the tree **as it stands at the end of
review round 5**, after every prose edit was frozen — which is the only moment
at which the check is worth anything. The seven from round 4 are
`log_ownership.rs` `:147`, `:281`, `:238` and `best_lane_reorg_rule.rs` `:105`,
`:208`, `:287`, `:240`. The four surviving `type_widths.rs` citations — `:143`,
`:166`, `:167`, `:175` — were each re-checked by hand and each lands on the
`size_of` assertion that `#[repr(align(N))]` reds: `OrgAdmin`,
`OnChainRootHash`, `OrgPubKey`, `Epoch` in that order. Round 5's own edits to
that header moved them back; **that is a coincidence and not a fix**, and it is
said here so no one mistakes it for one.

The lesson stands, and is now stated as a rule rather than an observation: line
citations into files a change edits are taken **once, after the last edit to
those files**, and re-taken whenever any later commit touches them. Three
separate commits in this change broke this rule.

**T2 — SDD-5wamsz.** `LLR-xv7auy`: `#[repr(align(32))]` on `OrgAdmin` in
`src/types.rs` → `newtypes_have_the_widths_the_abi_gives_them` failed at
`type_widths.rs:143`, left 32 right 20. (Plan's original mutation rejected; see
the findings table.)

**T3 — SDD-5b8wxs**, all in `src/h160.rs`. `LLR-2yhra8`: `&account_id_32[..20]`
→ `[1..21]` → `reverse_path_returns_the_first_twenty_bytes` at `:80`.
`LLR-3bkhuc`: `&hash[12..32]` → `[0..20]` → `forward_path_keccaks_then_truncates`
at `:123`. `LLR-7pjzjn`: marker window `[20..32]` → `[21..32]` →
`forward_path_taken_when_only_eleven_marker_bytes_are_present` at `:145`.

**T4 — SDD-bw7v5x**, all in `src/client.rs`'s `internals`. `LLR-vktf8w`:
`buf[12..32]` → `buf[0..20]` → `mapping_slot_matches_the_known_vector` at `:150`.
`LLR-62tqnv`: `to_be_bytes()` → `to_le_bytes()` →
`the_mapping_slot_index_is_big_endian_in_the_low_eight_bytes_of_the_second_word`
at `:248`. `LLR-bhwsn6`: `if carry == 0 { break; }` → unconditional `break` →
`a_carry_propagates_through_four_bytes` at `:331`. `LLR-2y9qdc`: `sum as u8` →
`sum.min(255) as u8` → `an_all_ones_slot_wraps_to_zero` at `:375`. `LLR-v62yjq`:
`carry = u16::from(offset)` → `0` →
`the_three_struct_field_slots_are_consecutive_and_distinct` at `:404`.

**T5 — SDD-v2rtka**, both in `src/decode/dispatch.rs`. `LLR-b3s7st`: pinned arm
→ `Err(UnsupportedRuntime)` →
`pinned_version_resolves_to_a_decoder_that_decodes_the_pinned_layout` at `:64`.
`LLR-u8ajby`: wildcard arm → `Ok(&DECODER)`, the fall-back-to-a-guess the
requirement forbids → all four refusal tests at `:53`.

**T6 — SDD-f2s7bx**, all in `src/decode/v_paseo_ah.rs`. `LLR-sq76u3`:
`bytes.len() != 96` → `< 96` → `ninety_seven_bytes_is_rejected` at `:161`.
`LLR-nq7nhg`: the two slot copies swapped →
`exactly_ninety_six_bytes_decodes_every_field` at `:122`. `LLR-emp3g9`: the
high-24-bytes guard deleted from `decode_uint256_to_u64` →
`a_non_zero_byte_anywhere_in_the_epoch_slots_leading_twenty_four_bytes_is_refused`
at `:235`.

**T7 — SDD-d5jh6t**, all in `src/decode/v_paseo_ah.rs`; all eight red the named
test's own assertion, no substitution needed. `LLR-e6skvu`: `SIG_ROOT_UPDATED`
first byte `0x24` → `0x25` → `the_recognised_signature_hashes_are_the_keccak_of_the_canonical_solidity_strings`
at `:213`. `LLR-rjcqg3`: `_ => Ok(None)` → `Err(InvalidAddressTopic)` →
`an_unknown_first_topic_yields_nothing` at `:331`. `LLR-n6gghu`: `parse_genesis`
`data.len() != 64` → `< 64` → `genesis_data_of_any_length_but_sixty_four_is_rejected`
at `:463`. `LLR-89pdz9`: `topic[..12]` → `[..11]` →
`a_non_zero_byte_at_each_padding_position_of_the_address_topic_is_rejected` at
`:537`. `LLR-8242kq`: `if !bytes.is_empty()` guard deleted →
`any_trailing_bytes_after_a_well_formed_payload_are_rejected` at `:631`.
`LLR-6tjhgk`: `prev_root_hash` from `&data[32..64]` →
`root_updated_round_trips_every_field` at `:302`. `LLR-u2e389`:
`contract: [0u8; 20]` → `genesis_initialized_round_trips_every_field` at `:268`.
`LLR-mzh8df`: `decode_uint256_to_u64(&topics[2])?` replaced by a direct
low-eight-byte read → the **new** test
`root_updated_epoch_above_u64_is_refused_not_truncated` at `:775`, left
`Ok(.. Epoch(7) ..)`, right `Err(EpochOverflow)` — the truncation REQ-9wwenn
forbids, and nothing else in the suite catches it.

**T7b — SDD-5wamsz / SDD-d5jh6t.** `LLR-z8rrkr` and `LLR-u2e389`:
`EmittedEvent { contract: [0u8; 20], event }` in `parse_revive_event` →
`the_emitting_address_is_reported_byte_for_byte_for_both_event_shapes` at
`contract_address_filter.rs:179`, and both sibling tests beside it.

**T8 — SDD-4z3k2u**, all in `src/client.rs`. `LLR-2znra8`: the
`emitted.contract != *configured_contract` guard deleted from `log_is_ours` →
`the_spoof_a_valid_log_from_another_contract_is_not_ours` at
`log_ownership.rs:147` — HAZ-werm85's own spoof. `LLR-kfr75c`: `None => true` →
`None => false` → `with_no_filter_any_admin_from_the_configured_contract_is_ours`
at `:281`. `LLR-9qp3k7`: a const sentinel returned from `event_admin`'s `Update`
arm → `with_a_filter_set_a_matching_admin_is_ours` at `:238`. (Substituted; see
the findings table.)

**T9 — SDD-m59zrg**, all in `src/client.rs`'s `scan_step`. `LLR-d3ef7s`: dedup
arm gains `&& prev.number == n` → `repeated_hash_with_inconsistent_number_is_still_skipped`
at `best_lane_reorg_rule.rs:105`. `LLR-56bzcj`: the `n <= prev.number` disjunct
dropped → `a_rewind_below_the_last_height_reports_the_discarded_head` at `:208`.
`LLR-48ygak`: `prev.number + 1` → `prev.number` →
`a_jump_backfills_every_skipped_height` at `:287`. `LLR-b4p32h`: reorg arm body
`Some(prev)` → `Some(BlockRef { hash: prev.hash, number: 0 })` →
`the_discarded_reference_carries_both_hash_and_number` at `:240`, left 0 right
7654321. **Substituted after the plan's original was applied and watched
PASSING** — see the findings table; that non-result is part of this record.

**T10 — the fuzz targets.** All three panicked on the **first input**, two from
committed corpus seeds and one from the first RNG draw. `LLR-yhw34z`: the length
guard deleted from `decode_org_state` → `fuzz_decode_org_state` panicked
`range end index 96 out of range for slice of length 95` after 1 corpus input,
303µs. `LLR-2v5u4d`: `topic[..12]` → `topic[..40]` in `unpack_address_topic` →
`fuzz_parse_revive_event` panicked `range end index 40 out of range for slice of
length 32` after 1 corpus input, 345µs. `LLR-6tjhgk`: `prev_root_hash` from
`&data[32..64]` → `fuzz_event_round_trip` failed its inversion assertion after 1
rng input. That these are killed immediately says the defects are maximally
reachable rather than deep in the state space, and — unlike the green runs,
whose iteration totals vary run to run — this evidence reproduces in under a
second.

**T1 and T11** produced no tests: the decomposition, the SOUP inventory, the
derived-requirements assessment, SDD-3b8zef's deviation and the owner item in
`docs/plans/2026-09-05-ratchet-setup.md`. `check-trace.sh` exits 0 for this unit
with `SDD 9, LLR 32` and no MISSING-TEST.

**The falsifiability sweep, 2026-09-28.** Every attestation above names one
mutation per LLR. The sweep asked the next question — whether each *clause* of
each LLR, rather than each LLR, has a mutation that reds — and answered it for
all thirty-two at once: **one hundred** independently falsifiable claims, and
after review round 4 corrected the classification rule, **all one hundred probed
by their own source mutation** — 103 runs over 102 distinct mutations in the
first sweep, plus two further mutations in round 4 — giving **98 RED, 2 GREEN**
and no unmeasured category. The two GREEN clauses were cut from the
decomposition and are the last two rows of the findings table above. The full
record and the method are in the decomposition, under "The falsifiability sweep
of 2026-09-28"; the two claims the first sweep had filed
UNTESTABLE-BY-MUTATION are recorded there as RED, with the mutations that red
them.

Three things about it belong here rather than there:

* **No attestation above changed.** Each named mutation still reds its named
  test, and each cut clause is one no mutation in this plan ever reached. The
  attestations were re-run as part of the sweep and are the same.
* **The five digests above held across all 103 runs of the first sweep**, and
  again after review rounds 4 and 5 mutated the tree further. Every run restored
  its files from pristine copies and re-hashed them.
* **Rounds 4 and 5 mutated three files that carry no tabulated digest**, and
  that is a gap in this cordon rather than a detail. The first sweep touched
  only the five files above, so "five source files were ever mutated" was true
  when written and is not true now: round 4 also mutated `src/lib.rs` and
  `src/decode/mod.rs`, and round 5 also mutated `src/state.rs`. All three are
  byte-identical to `master` — `git diff master...HEAD` lists no file under any
  unit `src/` — but that is checked against git rather than against a digest
  recorded before the mutation, which is weaker evidence than the five rows
  above provide. The digest table was not extended retroactively, because a
  digest taken after the fact proves nothing the git comparison does not.
* `verify_commands` is back at **73 passed, 0 failed**.
* **The sweep wrote no production code and no tests.** It is a measurement, and
  its only durable output is documentation: the cuts, the findings rows,
  and the decomposition's sweep section.

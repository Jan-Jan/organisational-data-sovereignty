# org-node type safety — Implementation Plan

**Goal:** bring org-node under the parse-at-the-system-edge rule
(`docs/adr/2026-10-04-parse-at-the-system-edge.md`): every org-node signature
and record field holds a newtype, secrets are held in redacted secret types
(PR-hqwpg9), Persona details are parsed when created, loaded or imported, and
no stored, sent, blob or calldata byte changes.
**Implements:** REQ-y7tsft, REQ-qn2erx, RC-8a4xjb, RC-zutc67;
SDD-st9knt: LLR-sz4xhc, LLR-bwb9pu, LLR-56hc77, LLR-mmdu38, LLR-s7whrn,
LLR-ayrdr8; SDD-zs2uyt: LLR-scgk5j, LLR-g76zqd, LLR-8bum44, LLR-q6n25z
(*re-homed 2026-10-05 at the merge of master `05f6f04`, by owner ruling: the
two items are withdrawn and the ten requirements sit under SDD-swtd3w,
SDD-sxp8hb and SDD-af5vnt, with cross-references from the other items they
constrain (placement as corrected in review round 7) — see
`org-node/docs/architecture/2026-10-04-type-safety.md`; the tasks below keep
the names they were written with*)
(`org-node/docs/architecture/2026-10-04-type-safety.md`);
SDD-4yr9ge: LLR-k6dhz7, LLR-t3p9zk
(`org-members/docs/architecture/2026-10-04-key-parse.md`).
Requirements in `org-node/docs/requirements/2026-10-04-type-safety.md`,
hazards and controls (HAZ-uy8sxm/RC-8a4xjb, HAZ-vfjy32/RC-zutc67) in
`org-node/docs/risk/2026-10-04-type-safety.md`.
**Resolves:** PR-hqwpg9 (`org-node/docs/problems/2026-10-04-secret-debug.md`).
**Safety class:** C (org-members), C (org-node), C (app). No per-item overrides.
**Verification:** the `verify_commands` of the three units, as amended by this
plan —
org-members: `cargo test -p org-members` and the quint typecheck/test/run lines
of `org-members/.guardrails/config.yaml`;
org-node: `cargo test -p org-node --features app,test-support --lib --test …`
(the line in `org-node/.guardrails/config.yaml`, extended by T1, T3, T4a, T4b
and T4c with `--test encoding_golden --test value_types --test chain_read_state
--test persona_records --test secret_redaction`, and by the review round 1
fixes with `--test calldata_typed`) and its quint lines;
app: `cargo test --manifest-path app/src-tauri/Cargo.toml --features
test-support --test …`, `npm --prefix app run check`, `npm --prefix app run test`;
`GR_CONFIG=<unit>/.guardrails/config.yaml .guardrails/scripts/check-trace.sh` and
`.guardrails/scripts/check-ids.sh --allow-draft-files` for each of the three
units; `.guardrails/scripts/check-units.sh`; coverage: `make coverage-org-members` (org-node and app have no
`coverage_command` — a recorded gap in their configs, unchanged here).

## Environment (every task)

- One git command per shell call; no `&&`/`;`/loops on a line that mentions
  git. Write files containing the word "git" with the Write tool.
- `CARGO_HOME=/tmp/cargo_home_fuzz` on every cargo command (`~/.cargo` is
  read-only; this home already holds every crate the lockfile needs). Add
  `--offline` if the network is blocked.
- Quint (org-members and org-node verify lines): `QUINT_HOME=<task
  worktree>/target/quint_home`, populated once per task worktree with
  `mkdir -p <task worktree>/target/quint_home` then
  `cp -R ~/.quint/. <task worktree>/target/quint_home/` (`~/.quint` is
  read-only by design; never write to it). A fresh `QUINT_HOME` can fail some
  `mbt_conformance` cases while the evaluator unpacks; rerun once before
  treating it as red.
- Never use `sed` on a Rust file (AGENTS.md lesson); use the Edit tool.
- `test_paths` of org-node is `org-node/tests`: an annotation in `src/` is read
  by no gate, so every `verifies:` test of this plan lives under `tests/`.

## Order and parallelism

```
T1 ∥ T2
T3            serial after T1 (shares org-node/.guardrails/config.yaml and
              org-node/Cargo.toml); may run ∥ T2 (disjoint files)
T4a → T4b → T4c   serial, after T2 and T3 (all share org-node/src/service.rs)
T5            serial after T4c
T6            serial after T5
```

The order the brief set (golden bytes, org-members, value types, conversion,
app, docs) is kept. Two adjustments the code forces:

1. **T4 is three tasks** along layer boundaries, not one: T4a the chain values
   (`OrgPublicKey`, `Epoch`, `SequenceNumber`, `ChainAccount` through chain,
   chain_read, chain_write, ceremony, verify, `ChainOps`), T4b the Persona
   details, keys and identifiers (`Handle`/`Name`/`Surname`, member/device keys,
   `MemberId`, `PersonaId`, the Raw decode mirrors, store-open and Join-request
   refusal), T4c the secrets (`MemberSeed`, `DeviceSeed`, `OrgSecret`,
   `StoreKey`, removal of `from_seed`/`to_seed`, the redaction test). Each ends
   with the whole org-node gate green. Secrets go last so the redaction test is
   written once, against final record types.
2. **`SigningKeypair::from_seed`/`to_seed` are removed in T4c, not T3.** T3's
   file set is `types.rs`, `keys.rs`, `error.rs`; removing the two functions
   there breaks every caller (service, endpoint, fixtures, preflight, eleven
   test files) outside it. T3 adds the replacements; T4c moves the callers and
   deletes the old functions.

**The app (`app/src-tauri`, its own workspace) does not compile from T3 until
T5.** T3 adds `OrgNodeError::InvalidKey` and `InvalidField`, and
`events::classify_receive_error` matches `OrgNodeError` exhaustively by design
(no wildcard); T4a–T4c change `ChainOps`, `create_persona`, `admit_member`,
`revoke_member` and the record fields the app reads. T5 brings the app back in
one task and runs its verify line; T3–T4c verify org-node only.

## Decisions this plan takes that the spec did not settle

- **`OrgNodeError::InvalidField { field: &'static str, reason: String }`** is
  the refusal LLR-8bum44 asks for ("an error whose text names the field").
  postcard's `de::Error::custom` discards its message (`SerdeDeCustom`, checked
  2026-10-04 against postcard 1.1.3), so a parse failing inside a nested
  `Deserialize` cannot name its field. Each record with a fallible field
  therefore has a crate-private `Raw…` mirror (same fields, same order, the
  fallible ones as `String`/`[u8; 32]`) and `impl TryFrom<Raw…>` that parses
  each field through `parse_field("<record>.<field>", …)`. The typed record
  declares `#[serde(try_from = "Raw…")]`, so its `Deserialize` and the
  field-naming load share one parse; `PersonaStore::open`,
  `blobs::decode_join_request` and `first_admission_base` decode the Raw form
  and call `try_from` themselves to keep the field name.
- **Error variants keep `u64`.** `StaleSeq { got, last_seen }` and
  `StaleEpoch { got, last }` are diagnostic output; they are built with
  `.get()`, and every existing assertion on them stays as it is.
- **`VerifyContext::author_member_key` stays `&VerifyingKey`** — already a
  parsed curve point, not plain bytes.
- **`Invite` keys are typed with a derived (validating) `Deserialize`**, no
  Raw mirror (LLR-8bum44 names the store and the Join request only). An Invite
  carrying an off-curve key is now refused at import — an observable change
  the risk draft does not list; T6 records it.
- **`chain_read::org_state_from_chain` becomes `pub`** and replaces the two
  private `map_state` copies (chain_read.rs and service.rs), so the chain-read
  parse (LLR-mmdu38's earlier refusal) is testable at an interface.
- **`ChainAccount` stays raw only inside `chain_write`**: the public
  chain_write functions (`proxied`, `create_pure`, `rotate`,
  `multi_account_id`, `dispatch_org_call`, `fund`) take and return
  `ChainAccount`; `chain_write::proxy::org_id_of(ChainAccount) -> OrgId` is
  added so `ceremony.rs` never unwraps it. `build_update_calldata` keeps raw
  arguments (it is the pinned EVM encoder); `revive_update_runtime_call` and
  `submit::submit_update` take `RootHash`, `OrgPublicKey`, `Epoch`.
- **Test seams** (feature `test-support`, `#[doc(hidden)]`):
  `store::seal_for_test` writes arbitrary plaintext as a store file, so a test
  can present content this software never writes;
  `store::store_key_debug_for_test` renders the private `StoreKey`'s `Debug`.
  `test_fixtures::Probe` plus `implements_display!`/`implements_copy!` answer
  "does T implement Display / Copy" at run time (autoref specialisation,
  checked 2026-10-04), so "no Display, not Copy" is a gated assertion.
- **App tests for persona input carry no requirement ID.** REQ-qn2erx is
  org-node's and is not exported to the app unit (an app annotation would be
  NON-EXPORTED-REF); they are robustness tests as in `ipc.rs`. REQ-qn2erx is
  verified in org-node through LLR-g76zqd and LLR-8bum44.
- **Golden test is type-agnostic** (T1), so it compiles and passes unchanged
  through T4: each test decodes a pinned value with the then-current types,
  re-encodes it to the same bytes, and reads named fields through serde's data
  model (`serde_json::to_value`), which is identical for a newtype and the plain
  value it wraps. The byte round trip alone would miss a symmetric change such
  as two fields swapping places; the field reads catch it. Constants were
  captured on this branch at `813f85b` by the generator in T1 Step 1 and the
  test was run green against that commit before this plan was written.

---

### T1 — Golden bytes before the refactor (LLR-ayrdr8)

**Files touched:** `org-node/tests/encoding_golden.rs` (new),
`org-node/Cargo.toml`, `org-node/.guardrails/config.yaml`
**Parallel:** yes (with T2)
**Status:** DONE — merged from `63c2589`; encoding_golden 5 passed, org-node 62 passed.

A characterisation test: green on today's code by construction; red is shown by
mutating code (Step 4). It must then stay green, byte-for-byte unedited,
through T2–T6. Feature gate: `app` (store and blobs are `app`; `app` implies
`chain` for calldata and `transport` for the wire). Makefile: no edit —
org-node has no coverage target.

red -> green:
- Step 1: the temporary generator reproduced all six GOLDEN_* constants byte for byte (deleted, never committed).
- persona_store_plaintext_is_pinned — mutation (a), `PendingInvite` admin_device_key/admin_member_key swapped: round trip still passed, field read FAILED at encoding_golden.rs:86 (208 vs 32); reverted -> green.
- genesis_and_update_calldata_is_pinned — mutation (b), epoch `to_le_bytes`: FAILED at encoding_golden.rs:140 (update calldata, epoch 7); reverted -> green.
- admission_wire_message_is_pinned — (evidence run in T3) mutation swapping `org_secret`/`genesis_snapshot` in `WireMessage` FAILED with `Err(Malformed)`; reverted -> green.
- invite_and_join_request_text_is_pinned — (evidence run in T3) mutation swapping `name`/`surname` in `JoinRequest` round-tripped but FAILED at line 129 ("Jones" vs "Bob"); reverted -> green.
- truncated_pinned_values_are_refused — (evidence run in T3) mutation making `decode_body` pad and retry FAILED (`decode_body(&body[..len-1]).is_err()`); reverted -> green.

Step 1 — regenerate the constants as a check (the values below were produced by
exactly this code at `813f85b`). Create a TEMPORARY file
`org-node/tests/golden_generator.rs` (never committed):

```rust
#![cfg(feature = "app")]
#![allow(clippy::unwrap_used)]
use org_node::blobs::{self, Invite, JoinRequest};
use org_node::chain_write::calldata::build_update_calldata;
use org_node::ids::OrgId;
use org_node::keys::SigningKeypair;
use org_node::store::{MemberSnapshot, OrgRecord, PendingInvite, PersonaRecord, PersonaStatus, StoreData};
use org_node::test_fixtures::admit_member_delta;
use org_node::transport::wire::{encode_frame, WireMessage};
use org_node::SignedDeltaEnvelope;

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn key(seed: u8) -> [u8; 32] {
    *SigningKeypair::from_seed([seed; 32]).verifying_key().as_bytes()
}

fn snap(id: u8, handle: &str, name: &str, surname: &str, mk: u8, dk: u8) -> MemberSnapshot {
    MemberSnapshot {
        id: [id; 32],
        handle: handle.into(),
        name: name.into(),
        surname: surname.into(),
        member_key: key(mk),
        device_keys: vec![key(dk)],
    }
}

#[test]
fn print_golden() {
    let store = StoreData {
        personas: vec![PersonaRecord {
            persona_id: "p-alice".into(),
            org_id: Some(OrgId::new([0xaa; 20])),
            handle: "alice".into(),
            name: "Alice".into(),
            surname: "Smith".into(),
            member_seed: [0x11; 32],
            device_seed: [0x12; 32],
            member_id: Some([0x01; 32]),
            status: PersonaStatus::Active,
        }],
        orgs: vec![OrgRecord {
            org_id: OrgId::new([0xaa; 20]),
            root_hash: [0x33; 32],
            org_pub_key: key(0x11),
            epoch: 3,
            org_secret: Some([0x44; 32]),
            last_seq: 2,
            admin_member_key: key(0x11),
            trie_members: vec![
                snap(0x01, "alice", "Alice", "Smith", 0x11, 0x12),
                snap(0x02, "bob", "Bob", "Jones", 0x21, 0x22),
            ],
            proxy_account: Some([0x55; 32]),
        }],
        pending_invites: vec![PendingInvite {
            org_id: OrgId::new([0xbb; 20]),
            admin_device_key: key(0x12),
            admin_member_key: key(0x11),
            org_pub_key: key(0x11),
        }],
    };
    println!("GOLDEN_STORE {}", hex(&postcard::to_allocvec(&store).unwrap()));
    let admin = SigningKeypair::from_seed([0x11; 32]);
    let (delta, _) = admit_member_delta(&admin);
    let envelope = SignedDeltaEnvelope::build(OrgId::new([0xaa; 20]), 2, &delta, &admin).unwrap();
    let snapshot = postcard::to_allocvec(&vec![snap(0x01, "admin", "Test", "User", 0x11, 0x04)]).unwrap();
    let msg = WireMessage { envelope, org_secret: Some([0x44; 32]), genesis_snapshot: Some(snapshot) };
    println!("GOLDEN_WIRE {}", hex(&encode_frame(&msg).unwrap()[4..]));
    let invite = Invite {
        org_id: OrgId::new([0xaa; 20]),
        org_pub_key: key(0x11),
        admin_member_key: key(0x11),
        admin_device_key: key(0x12),
        admin_node_addr: vec![1, 2, 3],
    };
    println!("GOLDEN_INVITE {}", blobs::encode(&invite).unwrap());
    let jr = JoinRequest {
        handle: "bob".into(),
        name: "Bob".into(),
        surname: "Jones".into(),
        member_key: key(0x21),
        device_key: key(0x22),
        node_addr: vec![4, 5, 6],
    };
    println!("GOLDEN_JOIN_REQUEST {}", blobs::encode(&jr).unwrap());
    println!("GOLDEN_GENESIS_CALLDATA {}", hex(&build_update_calldata([0x33; 32], [0x22; 32], 0)));
    println!("GOLDEN_UPDATE_CALLDATA {}", hex(&build_update_calldata([0x66; 32], [0x22; 32], 7)));
}
```

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node --features
app,test-support --test golden_generator -- --nocapture` → six `GOLDEN_…` lines
identical to the constants in Step 2. If any differs, STOP and report: the base
moved and the constants must be recaptured by the dispatcher, not by you.
Delete `org-node/tests/golden_generator.rs`.

Step 2 — write `org-node/tests/encoding_golden.rs`:

```rust
#![cfg(feature = "app")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Pins the bytes org-node stores, sends and submits, so the type-level
//! refactor of the org-node type-safety change cannot silently change them
//! (LLR-ayrdr8): the postcard plaintext of a Persona store, the postcard body
//! of an admission Wire message and the record snapshot inside it, the Base64
//! text of an Invite and a Join request, and the EVM calldata of a genesis and
//! an update.
//!
//! Values captured on worktree-org-node-type-safety @ 813f85b (2026-10-04),
//! before the refactor, by the generator in
//! docs/plans/2026-10-04-org-node-type-safety.md (T1). Never edit this file to
//! make it pass. Each test decodes a pinned value with whatever the field
//! types are at the time, requires it to encode back to the same bytes, and
//! reads named fields through serde's data model (`serde_json::Value`), which
//! is the same for a newtype and the plain value it wraps. The first catches
//! an asymmetric encoding change; the second catches a symmetric one, such as
//! two fields swapping places, which a byte round trip alone would miss.

use org_node::blobs::{self, Invite, JoinRequest};
use org_node::chain_write::calldata::build_update_calldata;
use org_node::service::first_admission_base;
use org_node::store::{MemberSnapshot, StoreData};
use org_node::transport::wire::{decode_body, encode_frame};
use serde_json::{json, Value};

const GOLDEN_STORE: &str = "0107702d616c69636501aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa05616c69636505416c69636505536d697468111111111111111111111111111111111111111111111111111111111111111112121212121212121212121212121212121212121212121212121212121212120101010101010101010101010101010101010101010101010101010101010101010101aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa3333333333333333333333333333333333333333333333333333333333333333d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c97787370301444444444444444444444444444444444444444444444444444444444444444402d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873702010101010101010101010101010101010101010101010101010101010101010105616c69636505416c69636505536d697468d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21020202020202020202020202020202020202020202020202020202020202020203626f6203426f62054a6f6e6573884b8857f4eaa1613c61504db34d4beaf346517a0e31de3cddd4d9b4201d9d0b01a09aa5f47a6759802ff955f8dc2d2a14a5c99d23be97f864127ff9383455a4f001555555555555555555555555555555555555555555555555555555555555555501bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb204040e364c10f2bec9c1fe500a1cd4c247c89d650a01ed7e82caba867877c21d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737";
const GOLDEN_WIRE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa029101b642ec203b364f4807ea74b0a63ab680c1c94d38b23246b1042f520394477e860001020202020202020202020202020202020202020202020202020202020202020203626f628139770ea87d175f56a35466c34c7ecccb8d8a91b4ee37a25df60f5b8fc9b3940454657374045573657201ed4928c628d1c2c6eae90338905995612959273a5c63f93636c14614ac8737d140523943b8c0e60f3412a3d5e2972b519b6e73eb2cc0ad9ebd217ca11640ebc38fc478dc6c7a3cbe94da88916e7295f66c52dff6a5c9c31eddfca932bdc245c10301444444444444444444444444444444444444444444444444444444444444444401720101010101010101010101010101010101010101010101010101010101010101010561646d696e04546573740455736572d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c977873701ca93ac1705187071d67b83c7ff0efe8108e8ec4530575d7726879333dbdabe7c";
const GOLDEN_INVITE: &str = "qqqqqqqqqqqqqqqqqqqqqqqqqqrQSrIydCu0qzoTaL1GFeTm0CJKtxoBa6+FIKMyyXeHN9BKsjJ0K7SrOhNovUYV5ObQIkq3GgFrr4UgozLJd4c3IEBA42TBDyvsnB/lAKHNTCR8idZQoB7X6CyrqGeHfCEDAQID";
const GOLDEN_JOIN_REQUEST: &str = "A2JvYgNCb2IFSm9uZXOIS4hX9OqhYTxhUE2zTUvq80ZReg4x3jzd1Nm0IB2dC6CapfR6Z1mAL/lV+NwtKhSlyZ0jvpf4ZBJ/+Tg0VaTwAwQFBg==";
const GOLDEN_GENESIS_CALLDATA: &str = "f1bc537b333333333333333333333333333333333333333333333333333333333333333322222222222222222222222222222222222222222222222222222222222222220000000000000000000000000000000000000000000000000000000000000000";
const GOLDEN_UPDATE_CALLDATA: &str = "f1bc537b666666666666666666666666666666666666666666666666666666666666666622222222222222222222222222222222222222222222222222222222222222220000000000000000000000000000000000000000000000000000000000000007";

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

/// The value as serde's data model sees it, field names included.
fn model<T: serde::Serialize>(value: &T) -> Value {
    serde_json::to_value(value).unwrap()
}

/// verifies: LLR-ayrdr8
#[test]
fn persona_store_plaintext_is_pinned() {
    let bytes = unhex(GOLDEN_STORE);
    let data: StoreData = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(hex(&postcard::to_allocvec(&data).unwrap()), GOLDEN_STORE);

    let m = model(&data);
    let persona = &m["personas"][0];
    assert_eq!(persona["persona_id"], json!("p-alice"));
    assert_eq!(persona["org_id"][0], json!(0xaa));
    assert_eq!(persona["handle"], json!("alice"));
    assert_eq!(persona["name"], json!("Alice"));
    assert_eq!(persona["surname"], json!("Smith"));
    assert_eq!(persona["member_seed"][31], json!(0x11));
    assert_eq!(persona["device_seed"][31], json!(0x12));
    assert_eq!(persona["member_id"][0], json!(0x01));
    assert_eq!(persona["status"], json!("Active"));
    let org = &m["orgs"][0];
    assert_eq!(org["org_id"][0], json!(0xaa));
    assert_eq!(org["root_hash"][0], json!(0x33));
    assert_eq!(org["org_pub_key"][0], json!(0xd0));
    assert_eq!(org["epoch"], json!(3));
    assert_eq!(org["org_secret"][31], json!(0x44));
    assert_eq!(org["last_seq"], json!(2));
    assert_eq!(org["admin_member_key"][0], json!(0xd0));
    assert_eq!(org["proxy_account"][0], json!(0x55));
    let bob = &org["trie_members"][1];
    assert_eq!(bob["id"][0], json!(0x02));
    assert_eq!(bob["handle"], json!("bob"));
    assert_eq!(bob["name"], json!("Bob"));
    assert_eq!(bob["surname"], json!("Jones"));
    assert_eq!(bob["member_key"][0], json!(0x88));
    assert_eq!(bob["device_keys"][0][0], json!(0xa0));
    let invite = &m["pending_invites"][0];
    assert_eq!(invite["org_id"][0], json!(0xbb));
    assert_eq!(invite["admin_device_key"][0], json!(0x20));
    assert_eq!(invite["admin_member_key"][0], json!(0xd0));
    assert_eq!(invite["org_pub_key"][0], json!(0xd0));
}

/// verifies: LLR-ayrdr8
#[test]
fn admission_wire_message_is_pinned() {
    let body = unhex(GOLDEN_WIRE);
    let msg = decode_body(&body).unwrap();
    let framed = encode_frame(&msg).unwrap();
    assert_eq!(hex(&framed[4..]), GOLDEN_WIRE);

    let m = model(&msg);
    assert_eq!(m["envelope"]["org_id"][0], json!(0xaa));
    assert_eq!(m["envelope"]["parent_seq"], json!(2));
    assert_eq!(m["org_secret"][31], json!(0x44));

    let snapshot = msg.genesis_snapshot.as_deref().expect("an admission carries a record snapshot");
    let members: Vec<MemberSnapshot> = postcard::from_bytes(snapshot).unwrap();
    assert_eq!(postcard::to_allocvec(&members).unwrap(), snapshot);
    let admin = &model(&members)[0];
    assert_eq!(admin["handle"], json!("admin"));
    assert_eq!(admin["member_key"][0], json!(0xd0));
    assert!(first_admission_base(Some(snapshot)).is_ok(), "the pinned snapshot rebuilds a record");
}

/// verifies: LLR-ayrdr8
#[test]
fn invite_and_join_request_text_is_pinned() {
    let invite: Invite = blobs::decode(GOLDEN_INVITE).unwrap();
    assert_eq!(blobs::encode(&invite).unwrap(), GOLDEN_INVITE);
    let m = model(&invite);
    assert_eq!(m["org_id"][0], json!(0xaa));
    assert_eq!(m["org_pub_key"][0], json!(0xd0));
    assert_eq!(m["admin_member_key"][0], json!(0xd0));
    assert_eq!(m["admin_device_key"][0], json!(0x20));
    assert_eq!(m["admin_node_addr"], json!([1, 2, 3]));

    let join_request: JoinRequest = blobs::decode(GOLDEN_JOIN_REQUEST).unwrap();
    assert_eq!(blobs::encode(&join_request).unwrap(), GOLDEN_JOIN_REQUEST);
    let m = model(&join_request);
    assert_eq!(m["handle"], json!("bob"));
    assert_eq!(m["name"], json!("Bob"));
    assert_eq!(m["surname"], json!("Jones"));
    assert_eq!(m["member_key"][0], json!(0x88));
    assert_eq!(m["device_key"][0], json!(0xa0));
    assert_eq!(m["node_addr"], json!([4, 5, 6]));
}

/// verifies: LLR-ayrdr8
#[test]
fn genesis_and_update_calldata_is_pinned() {
    assert_eq!(hex(&build_update_calldata([0x33; 32], [0x22; 32], 0)), GOLDEN_GENESIS_CALLDATA);
    assert_eq!(hex(&build_update_calldata([0x66; 32], [0x22; 32], 7)), GOLDEN_UPDATE_CALLDATA);
}

/// Abnormal input: a pinned value cut short is refused, never decoded into
/// something else.
/// verifies: LLR-ayrdr8
#[test]
fn truncated_pinned_values_are_refused() {
    let store = unhex(GOLDEN_STORE);
    assert!(postcard::from_bytes::<StoreData>(&store[..store.len() - 1]).is_err());
    let body = unhex(GOLDEN_WIRE);
    assert!(decode_body(&body[..body.len() - 1]).is_err());
    assert!(blobs::decode::<Invite>(&GOLDEN_INVITE[..GOLDEN_INVITE.len() - 4]).is_err());
    assert!(blobs::decode::<JoinRequest>(&GOLDEN_JOIN_REQUEST[..GOLDEN_JOIN_REQUEST.len() - 4]).is_err());
}
```

Step 3 — `org-node/Cargo.toml`, after the `admission_sender` `[[test]]` block:

```toml
# Golden bytes pinned before the org-node type-safety refactor (LLR-ayrdr8).
# Store and blobs are `app`; `app` implies `chain` (calldata) and `transport`
# (wire).
[[test]]
name = "encoding_golden"
path = "tests/encoding_golden.rs"
required-features = ["app"]
```

`org-node/.guardrails/config.yaml`: append ` --test encoding_golden` to the end
of the `cargo test -p org-node …` verify line.

Step 4 — run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node
--features app,test-support --test encoding_golden` → `test result: ok. 5
passed`. Show red by mutation (record both in `red -> green:`), reverting each
before the next:
(a) in `org-node/src/store.rs` swap the declaration order of
`admin_device_key` and `admin_member_key` in `PendingInvite` → the round trip
still passes but `persona_store_plaintext_is_pinned` FAILS at
`invite["admin_device_key"][0]` (0xd0 vs 0x20) — the field read is what catches
a symmetric change; revert;
(b) in `org-node/src/chain_write/calldata.rs` write the epoch little-endian
(`to_le_bytes`) → `genesis_and_update_calldata_is_pinned` FAILS; revert.
`git diff --stat` must then show only the three files of this task.

Step 5 — baseline for later tasks: run the full org-node verify line (cargo
part) and `CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy -p org-node --features
app,test-support` and put the pass counts and the clippy warning count in the
dispatch report (T4a–T4c compare against them).

Step 6 — commit `test(org-node): pin store, wire, blob and calldata bytes before the type-safety refactor (LLR-ayrdr8)`.

---

### T2 — org-members key parse and `P2pDeviceSlots::parse` (LLR-k6dhz7, LLR-t3p9zk)

**Files touched:** `org-members/src/types.rs`, `org-members/src/error.rs`,
`org-members/src/trie.rs`, `org-members/tests/newtypes.rs`,
`org-members/tests/integration_test.rs`
**Parallel:** yes (with T1, and with T3)

No new test file: the tests go in `newtypes.rs`, already in
`make coverage-org-members` and in `cargo test -p org-members`, so neither the
Makefile nor the config changes. `P2pDeviceSlots::new` has no caller in
org-node or the app; its callers are `types.rs:595`, `trie.rs:370`,
`newtypes.rs:129`, `integration_test.rs:1648`, `:1797`, `:1827`.

**Status:** DONE — merged from `1e5421f`; org-members all green (newtypes 20, integration 161, mbt 32), quint clean, clippy 0.

red -> green:
- key_parse_accepts_exactly_what_decoding_accepts — failed to compile before the implementation (no `parse`, no `TryFrom<[u8; 32]>`); then mutation `parse` always `Err(InvalidKey)` FAILED at newtypes.rs:343; reverted -> green.
- key_parse_refuses_bytes_off_the_curve — failed to compile before the implementation (no `InvalidKey`, no `parse`); green after.
- device_slots_parse_holds_keys_sorted — failed to compile before the implementation (no `P2pDeviceSlots::parse`); green after.
- device_slots_parse_refuses_too_many_and_repeated_keys — failed to compile before the implementation; then mutation removing the `MAX_DEVICES` check FAILED at newtypes.rs:393; reverted -> green.

Step 1 — append to `org-members/tests/newtypes.rs` (imports are inside the test
functions so nothing collides with the file's existing `use` lines):

```rust
fn curve_key(seed: u8) -> [u8; 32] {
    *ed25519_dalek::SigningKey::from_bytes(&[seed; 32]).verifying_key().as_bytes()
}

/// y = 0: a point of small order (a weak key). Decompresses.
fn weak_key() -> [u8; 32] {
    [0u8; 32]
}

/// y = p + 1 (≡ 1 mod p): a non-canonical encoding of the identity point.
/// Decompresses.
fn non_canonical_key() -> [u8; 32] {
    let mut b = [0xffu8; 32];
    b[0] = 0xee;
    b[31] = 0x7f;
    b
}

/// y = 2: no x satisfies the curve equation, so these bytes decompress to
/// nothing.
fn off_curve_key() -> [u8; 32] {
    let mut b = [0u8; 32];
    b[0] = 2;
    b
}

/// verifies: LLR-k6dhz7
#[test]
fn key_parse_accepts_exactly_what_decoding_accepts() {
    use org_members::types::{P2pDeviceKey, P2pMemberKey};
    // A key from a signing key, a weak key and a non-canonical encoding all
    // decompress, so all three are accepted, bytes unchanged (owner ruling
    // 2026-10-04; refusing the last two is PR-vkw22m).
    for bytes in [curve_key(7), weak_key(), non_canonical_key()] {
        let encoded = postcard::to_allocvec(&bytes).unwrap();
        let member = P2pMemberKey::parse(&bytes).unwrap();
        assert_eq!(member.as_bytes(), &bytes, "parse keeps the bytes as given");
        assert_eq!(P2pMemberKey::try_from(bytes).unwrap(), member);
        assert_eq!(postcard::from_bytes::<P2pMemberKey>(&encoded).unwrap(), member);
        let device = P2pDeviceKey::parse(&bytes).unwrap();
        assert_eq!(device.as_bytes(), &bytes);
        assert_eq!(P2pDeviceKey::try_from(bytes).unwrap(), device);
        assert_eq!(postcard::from_bytes::<P2pDeviceKey>(&encoded).unwrap(), device);
    }
}

/// verifies: LLR-k6dhz7
#[test]
fn key_parse_refuses_bytes_off_the_curve() {
    use org_members::types::{P2pDeviceKey, P2pMemberKey};
    let bytes = off_curve_key();
    let encoded = postcard::to_allocvec(&bytes).unwrap();
    assert_eq!(P2pMemberKey::parse(&bytes), Err(OrgMembersError::InvalidKey));
    assert_eq!(P2pMemberKey::try_from(bytes), Err(OrgMembersError::InvalidKey));
    assert!(postcard::from_bytes::<P2pMemberKey>(&encoded).is_err());
    assert_eq!(P2pDeviceKey::parse(&bytes), Err(OrgMembersError::InvalidKey));
    assert_eq!(P2pDeviceKey::try_from(bytes), Err(OrgMembersError::InvalidKey));
    assert!(postcard::from_bytes::<P2pDeviceKey>(&encoded).is_err());
}

fn slot_device(seed: u8) -> org_members::types::P2pDeviceKey {
    org_members::types::P2pDeviceKey::parse(&curve_key(seed)).unwrap()
}

/// verifies: LLR-t3p9zk
#[test]
fn device_slots_parse_holds_keys_sorted() {
    use org_members::types::{P2pDeviceKey, P2pDeviceSlots, MAX_DEVICES};
    let keys: Vec<P2pDeviceKey> = (1..=MAX_DEVICES as u8).map(slot_device).collect();
    let mut sorted = keys.clone();
    sorted.sort();
    let mut reversed = keys;
    reversed.reverse();
    let slots = P2pDeviceSlots::parse(reversed.clone()).unwrap();
    assert_eq!(slots.devices(), sorted.as_slice(), "held sorted, at the MAX_DEVICES bound");
    assert_eq!(P2pDeviceSlots::try_from(reversed).unwrap(), slots);
    let empty = P2pDeviceSlots::parse(Vec::new()).unwrap();
    assert_eq!(empty.device_count(), 0, "the empty list is the isolated state");
}

/// verifies: LLR-t3p9zk
#[test]
fn device_slots_parse_refuses_too_many_and_repeated_keys() {
    use org_members::types::{P2pDeviceKey, P2pDeviceSlots, MAX_DEVICES};
    let over: Vec<P2pDeviceKey> = (1..=MAX_DEVICES as u8 + 1).map(slot_device).collect();
    assert_eq!(P2pDeviceSlots::parse(over.clone()), Err(OrgMembersError::DeviceSlotsFull));
    assert_eq!(P2pDeviceSlots::try_from(over), Err(OrgMembersError::DeviceSlotsFull));
    assert_eq!(
        P2pDeviceSlots::parse(vec![slot_device(1), slot_device(2), slot_device(1)]),
        Err(OrgMembersError::DuplicateDevice)
    );
}
```

Step 2 — `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-members --test
newtypes` → red: `error[E0599]: no function or associated item named parse
found for struct P2pMemberKey` (and for `P2pDeviceSlots`), `no variant
InvalidKey`.

Step 3 — `org-members/src/error.rs`, after `DuplicateKey`:

```rust
    /// 32 bytes offered as a member or device key that do not decompress to
    /// an Edwards point. LLR-k6dhz7.
    #[error("invalid key: the bytes are not a point on the curve")]
    InvalidKey,
```

`org-members/src/types.rs`:
- In `impl P2pMemberKey` add, and the same on `P2pDeviceKey` (doc names it a
  device key):

```rust
    /// Parses 32 bytes as a member key. Accepts exactly the bytes that
    /// decompress to an Edwards point — what `Deserialize` accepts, which now
    /// calls this — and does not refuse small-order or non-canonical
    /// encodings (owner ruling 2026-10-04; PR-vkw22m). LLR-k6dhz7.
    pub fn parse(bytes: &[u8; 32]) -> Result<Self, OrgMembersError> {
        VerifyingKey::from_bytes(bytes)
            .map(Self)
            .map_err(|_| OrgMembersError::InvalidKey)
    }
```

```rust
impl TryFrom<[u8; 32]> for P2pMemberKey {
    type Error = OrgMembersError;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        Self::parse(&bytes)
    }
}
```
  (and `TryFrom<[u8; 32]> for P2pDeviceKey`).
- Both `Deserialize` impls become:

```rust
        let bytes = <[u8; 32]>::deserialize(d)?;
        Self::parse(&bytes).map_err(serde::de::Error::custom)
```
- Rename `P2pDeviceSlots::new` to `parse` with this doc, body unchanged:

```rust
    /// Holds `devices` sorted. Accepts 0..=MAX_DEVICES keys: the empty list
    /// is the isolated state `emergency_isolate_member` produces;
    /// `MemberLeaf::new` requires ≥1 of its own. Refuses more than
    /// `MAX_DEVICES` keys with `DeviceSlotsFull` and a repeated key with
    /// `DuplicateDevice`. LLR-t3p9zk.
    pub fn parse(mut devices: Vec<P2pDeviceKey>) -> Result<Self, OrgMembersError> {
```
  and add after the impl block:

```rust
impl TryFrom<Vec<P2pDeviceKey>> for P2pDeviceSlots {
    type Error = OrgMembersError;

    fn try_from(devices: Vec<P2pDeviceKey>) -> Result<Self, Self::Error> {
        Self::parse(devices)
    }
}
```
- `types.rs:595` `P2pDeviceSlots::new(p2p_devices)?` → `P2pDeviceSlots::parse(p2p_devices)?`.

`org-members/src/trie.rs:370` → `P2pDeviceSlots::parse(Vec::new())?`.
`newtypes.rs:129`, `integration_test.rs:1648`, `:1797`, `:1827`:
`P2pDeviceSlots::new(` → `P2pDeviceSlots::parse(` (Edit tool).
`grep -rn "P2pDeviceSlots::new" org-members org-node app --include='*.rs'` → no output.

Step 4 — `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-members` → all
suites green (newtypes +4, `encoding_golden` unchanged, mbt 32 passed /
1 ignored); the quint lines of org-members' verify_commands under `QUINT_HOME`
→ no violation; `cargo clippy -p org-members --all-targets` → no warnings;
`cargo check -p org-members --no-default-features`, `--no-default-features
--features serde`, `--no-default-features --features serde --target
wasm32-unknown-unknown` → clean. `cargo test -p org-node --features
app,test-support --lib` → green (org-node's `Trie(_)` arms absorb the new
variant). Show red by mutation as well, each reverted: (a)
`P2pMemberKey::parse` returning `Err(OrgMembersError::InvalidKey)` for every
input → `key_parse_accepts_exactly_what_decoding_accepts` FAILS; (b)
`P2pDeviceSlots::parse` without its `MAX_DEVICES` check →
`device_slots_parse_refuses_too_many_and_repeated_keys` FAILS.

Step 5 — commit `feat(org-members): key types parse from bytes; P2pDeviceSlots::new renamed parse (LLR-k6dhz7, LLR-t3p9zk)`.

---

### T3 — org-node value types (LLR-sz4xhc, LLR-56hc77, LLR-mmdu38, LLR-s7whrn)

**Files touched:** `org-node/src/types.rs` (new), `org-node/src/lib.rs`,
`org-node/src/keys.rs`, `org-node/src/error.rs`,
`org-node/src/test_fixtures.rs`, `org-node/tests/value_types.rs` (new),
`org-node/Cargo.toml`, `org-node/.guardrails/config.yaml`
**Parallel:** no for T1 (serial after T1: shares `Cargo.toml` and
`config.yaml`); yes with T2.

The types are added beside the existing code; nothing uses them yet, and
`from_seed`/`to_seed` stay until T4c. After this task the app does not compile
(new `OrgNodeError` variants); T5 restores it.

**Status:** DONE — merged from `46b7554`; org-node 71 passed (value_types 9, encoding_golden 5 unchanged), clippy lib clean.

red -> green:
- All 9 value_types tests failed to compile before the implementation (E0432 unresolved MemberSeed, DeviceSeed, OrgSecret, OrgPublicKey, ChainAccount, PersonaId, Epoch, SequenceNumber, implements_copy/display; E0599 member_seed, device_seed, InvalidKey).
- secret_types_redact_debug_and_give_bytes_only_through_the_accessor — FAILED under (a) Debug printing the bytes and (b) an added `Display for OrgSecret`; reverted -> green.
- secret_debug_is_the_same_whatever_the_bytes — FAILED under (a); reverted -> green.
- secret_types_serialise_as_the_plain_bytes — compile-red only; no runtime mutation tried (derived transparent serde).
- org_public_key_accepts_every_curve_point_unchanged, org_public_key_refuses_bytes_off_the_curve — FAILED under (c) `parse` decompressing a fixed array; reverted -> green.
- a_seed_yields_its_key_pair_and_a_key_pair_its_seed, seeds_at_the_byte_bounds_yield_working_key_pairs — FAILED under (d) `signing_keypair` flipping a seed bit; reverted -> green.
- tag_types_hold_their_value_unchanged, tag_types_accept_every_boundary_value_and_serialise_as_it — FAILED under (d') `Epoch::new` storing value+1; reverted -> green.

Step 1 — write `org-node/tests/value_types.rs`:

```rust
#![cfg(feature = "test-support")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! org-node's value types (SDD-st9knt) at their own interface: the secret
//! types (LLR-sz4xhc), seed to key pair (LLR-56hc77), the Organisation public
//! key (LLR-mmdu38) and the tag types (LLR-s7whrn).

use ed25519_dalek::SigningKey;
use org_members::P2pMemberKey;
use org_node::keys::{verify, SigningKeypair};
use org_node::{implements_copy, implements_display};
use org_node::{
    ChainAccount, DeviceSeed, Epoch, MemberSeed, OrgNodeError, OrgPublicKey, OrgSecret,
    PersonaId, SequenceNumber,
};
use rand::rngs::OsRng;

fn curve_key(seed: u8) -> [u8; 32] {
    *SigningKey::from_bytes(&[seed; 32]).verifying_key().as_bytes()
}

/// y = 0: a point of small order. Decompresses.
fn weak_key() -> [u8; 32] {
    [0u8; 32]
}

/// y = p + 1: a non-canonical encoding of the identity. Decompresses.
fn non_canonical_key() -> [u8; 32] {
    let mut b = [0xffu8; 32];
    b[0] = 0xee;
    b[31] = 0x7f;
    b
}

/// y = 2: not on the curve.
fn off_curve_key() -> [u8; 32] {
    let mut b = [0u8; 32];
    b[0] = 2;
    b
}

fn plain<T: serde::Serialize>(v: &T) -> Vec<u8> {
    postcard::to_allocvec(v).unwrap()
}

/// verifies: LLR-sz4xhc
#[test]
fn secret_types_redact_debug_and_give_bytes_only_through_the_accessor() {
    let bytes = [0x5au8; 32];
    let member = MemberSeed::from(bytes);
    let device = DeviceSeed::from(bytes);
    let org = OrgSecret::from(bytes);
    assert_eq!(member.expose_secret(), &bytes);
    assert_eq!(device.expose_secret(), &bytes);
    assert_eq!(org.expose_secret(), &bytes);
    assert_eq!(format!("{member:?}"), "MemberSeed([REDACTED])");
    assert_eq!(format!("{device:?}"), "DeviceSeed([REDACTED])");
    assert_eq!(format!("{org:?}"), "OrgSecret([REDACTED])");
    assert_eq!(member.clone(), member);
    assert_ne!(OrgSecret::from([1u8; 32]), org);
    assert!(implements_display!(String) && implements_copy!(u8), "the probe itself works");
    assert!(!implements_display!(MemberSeed) && !implements_copy!(MemberSeed));
    assert!(!implements_display!(DeviceSeed) && !implements_copy!(DeviceSeed));
    assert!(!implements_display!(OrgSecret) && !implements_copy!(OrgSecret));
}

/// verifies: LLR-sz4xhc
#[test]
fn secret_debug_is_the_same_whatever_the_bytes() {
    // Boundary bytes, and bytes that spell the marker itself.
    for bytes in [[0u8; 32], [0xffu8; 32], *b"MemberSeed([REDACTED])0123456789"] {
        assert_eq!(format!("{:?}", MemberSeed::from(bytes)), "MemberSeed([REDACTED])");
        assert_eq!(format!("{:#?}", DeviceSeed::from(bytes)), "DeviceSeed([REDACTED])");
        assert_eq!(format!("{:?}", Some(OrgSecret::from(bytes))), "Some(OrgSecret([REDACTED]))");
    }
}

/// verifies: LLR-sz4xhc
#[test]
fn secret_types_serialise_as_the_plain_bytes() {
    let bytes = [0xa5u8; 32];
    assert_eq!(plain(&MemberSeed::from(bytes)), plain(&bytes));
    assert_eq!(plain(&DeviceSeed::from(bytes)), plain(&bytes));
    assert_eq!(plain(&OrgSecret::from(bytes)), plain(&bytes));
    let back: OrgSecret = postcard::from_bytes(&plain(&bytes)).unwrap();
    assert_eq!(back, OrgSecret::from(bytes));
    // Abnormal: 31 bytes are not a secret.
    assert!(postcard::from_bytes::<OrgSecret>(&plain(&bytes)[..31]).is_err());
}

/// verifies: LLR-56hc77
#[test]
fn a_seed_yields_its_key_pair_and_a_key_pair_its_seed() {
    let member_kp = SigningKeypair::generate(&mut OsRng);
    assert_eq!(member_kp.member_seed().signing_keypair().verifying_key(), member_kp.verifying_key());
    let device_kp = SigningKeypair::generate(&mut OsRng);
    assert_eq!(device_kp.device_seed().signing_keypair().verifying_key(), device_kp.verifying_key());
    // The seed is the RFC 8032 secret key: any ed25519 implementation derives
    // the same key pair from it.
    assert_eq!(
        MemberSeed::from([0x11; 32]).signing_keypair().verifying_key(),
        SigningKey::from_bytes(&[0x11; 32]).verifying_key()
    );
}

/// verifies: LLR-56hc77
#[test]
fn seeds_at_the_byte_bounds_yield_working_key_pairs() {
    for bytes in [[0u8; 32], [0xffu8; 32]] {
        let member = MemberSeed::from(bytes).signing_keypair();
        let device = DeviceSeed::from(bytes).signing_keypair();
        // The seed type names the role; it does not change the derivation.
        assert_eq!(member.verifying_key(), device.verifying_key());
        let sig = member.sign(b"bound");
        assert!(verify(&member.verifying_key(), b"bound", &sig));
        assert!(!verify(&member.verifying_key(), b"other", &sig));
    }
}

/// verifies: LLR-mmdu38
#[test]
fn org_public_key_accepts_every_curve_point_unchanged() {
    for bytes in [curve_key(0x11), weak_key(), non_canonical_key()] {
        let key = OrgPublicKey::parse(&bytes).unwrap();
        assert_eq!(key.as_bytes(), &bytes);
        assert_eq!(OrgPublicKey::try_from(bytes).unwrap(), key);
        assert_eq!(plain(&key), plain(&bytes), "serialises as the plain bytes");
        assert_eq!(postcard::from_bytes::<OrgPublicKey>(&plain(&bytes)).unwrap(), key);
    }
    let member = P2pMemberKey::new(SigningKey::from_bytes(&[0x11; 32]).verifying_key());
    assert_eq!(OrgPublicKey::from(&member).as_bytes(), member.as_bytes());
}

/// verifies: LLR-mmdu38
#[test]
fn org_public_key_refuses_bytes_off_the_curve() {
    let bytes = off_curve_key();
    assert_eq!(OrgPublicKey::parse(&bytes), Err(OrgNodeError::InvalidKey));
    assert_eq!(OrgPublicKey::try_from(bytes), Err(OrgNodeError::InvalidKey));
    assert!(postcard::from_bytes::<OrgPublicKey>(&plain(&bytes)).is_err());
}

/// verifies: LLR-s7whrn
#[test]
fn tag_types_hold_their_value_unchanged() {
    let account = [0x5au8; 32];
    assert_eq!(ChainAccount::new(account).as_bytes(), &account);
    assert_eq!(ChainAccount::from(account), ChainAccount::new(account));
    assert_eq!(PersonaId::new("p-alice".to_string()).as_str(), "p-alice");
    assert_eq!(PersonaId::from("p-alice".to_string()), PersonaId::new("p-alice".to_string()));
    assert_eq!(Epoch::new(3).get(), 3);
    assert_eq!(Epoch::from(3), Epoch::new(3));
    assert_eq!(SequenceNumber::new(2).get(), 2);
    assert_eq!(SequenceNumber::from(2), SequenceNumber::new(2));
    assert!(Epoch::new(2) < Epoch::new(3), "epochs order as their numbers");
    assert!(SequenceNumber::new(1) < SequenceNumber::new(2));
}

/// verifies: LLR-s7whrn
#[test]
fn tag_types_accept_every_boundary_value_and_serialise_as_it() {
    for bytes in [[0u8; 32], [0xffu8; 32]] {
        assert_eq!(ChainAccount::new(bytes).as_bytes(), &bytes);
        assert_eq!(plain(&ChainAccount::new(bytes)), plain(&bytes));
    }
    for id in [String::new(), "ünïcødé-persona".to_string(), "p".repeat(1024)] {
        assert_eq!(PersonaId::new(id.clone()).as_str(), id);
        assert_eq!(plain(&PersonaId::new(id.clone())), plain(&id));
    }
    for n in [0u64, 1, u64::MAX] {
        assert_eq!(Epoch::new(n).get(), n);
        assert_eq!(SequenceNumber::new(n).get(), n);
        assert_eq!(plain(&Epoch::new(n)), plain(&n));
        assert_eq!(plain(&SequenceNumber::new(n)), plain(&n));
    }
}
```

Step 2 — `org-node/Cargo.toml`, after the T1 block:

```toml
# org-node's value types (SDD-st9knt); uses the Display/Copy probe in
# `test_fixtures`, hence `test-support`.
[[test]]
name = "value_types"
path = "tests/value_types.rs"
required-features = ["test-support"]
```

and append ` --test value_types` to the org-node cargo verify line in
`config.yaml`. Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node
--features app,test-support --test value_types` → red: `error[E0432]:
unresolved imports org_node::ChainAccount, …`, `cannot find macro
implements_display`.

Step 3 — `org-node/src/types.rs`:

```rust
//! The value types org-node defines for what it holds that org-members does
//! not (SDD-st9knt): the secrets (LLR-sz4xhc), the Organisation public key
//! (LLR-mmdu38), and the tag types for the chain account, the Persona
//! identifier, the epoch and the Sequence number (LLR-s7whrn). Seed to key
//! pair conversion lives in `keys.rs` (LLR-56hc77).
use core::fmt;

use ed25519_dalek::VerifyingKey;
use org_members::P2pMemberKey;
use serde::{Deserialize, Serialize};

use crate::error::OrgNodeError;

/// A secret type (LLR-sz4xhc): built infallibly from 32 bytes, gives them up
/// only through `expose_secret`, renders under `Debug` as its name and
/// `([REDACTED])` whatever it holds, has no `Display` and no `Copy`, and
/// serialises as the plain bytes it wraps. Wiping on drop is not done (owner
/// ruling 2026-10-04, RC-jjsz97 residual).
macro_rules! secret_type {
    ($(#[$meta:meta])* $ty:ident) => {
        $(#[$meta])*
        #[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $ty([u8; 32]);

        impl $ty {
            /// The secret bytes. The only way out: each call is a deliberate,
            /// searchable use (HAZ-uy8sxm residual).
            pub fn expose_secret(&self) -> &[u8; 32] {
                &self.0
            }
        }

        impl From<[u8; 32]> for $ty {
            fn from(bytes: [u8; 32]) -> Self {
                Self(bytes)
            }
        }

        impl fmt::Debug for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($ty), "([REDACTED])"))
            }
        }
    };
}

secret_type!(
    /// The seed of a Member's signing key pair: the secret half of the
    /// Member-as-a-group key.
    MemberSeed
);
secret_type!(
    /// The seed of a device's signing key pair: the secret half of its Device
    /// key and of its iroh identity.
    DeviceSeed
);
secret_type!(
    /// The Organisation secret handed to a Member at admission.
    OrgSecret
);

/// The Organisation public key, parsed as a curve point (LLR-mmdu38). The
/// published signing key today (PR-szkat6).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrgPublicKey(VerifyingKey);

impl OrgPublicKey {
    /// Accepts exactly the 32 bytes that decompress to an Edwards point;
    /// refuses any other with `InvalidKey`.
    pub fn parse(bytes: &[u8; 32]) -> Result<Self, OrgNodeError> {
        VerifyingKey::from_bytes(bytes)
            .map(Self)
            .map_err(|_| OrgNodeError::InvalidKey)
    }

    /// The 32 bytes as given to `parse`.
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }

    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.0
    }
}

impl TryFrom<[u8; 32]> for OrgPublicKey {
    type Error = OrgNodeError;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        Self::parse(&bytes)
    }
}

/// The genesis value today: the founding administrator's Member key
/// (PR-szkat6).
impl From<&P2pMemberKey> for OrgPublicKey {
    fn from(key: &P2pMemberKey) -> Self {
        Self(*key.verifying_key())
    }
}

impl fmt::Debug for OrgPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let b = self.as_bytes();
        write!(f, "OrgPublicKey({:02x}{:02x}{:02x}{:02x}..)", b[0], b[1], b[2], b[3])
    }
}

impl Serialize for OrgPublicKey {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.as_bytes().serialize(s)
    }
}

impl<'de> Deserialize<'de> for OrgPublicKey {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let bytes = <[u8; 32]>::deserialize(d)?;
        Self::parse(&bytes).map_err(serde::de::Error::custom)
    }
}

/// A chain account (`AccountId32`): an Organisation's pure proxy or a
/// co-signatory. Tag type; converted to subxt's account type or raw bytes only
/// inside `chain_write`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ChainAccount([u8; 32]);

impl ChainAccount {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for ChainAccount {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl fmt::Debug for ChainAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChainAccount(0x")?;
        for b in self.0 {
            write!(f, "{b:02x}")?;
        }
        write!(f, ")")
    }
}

/// The identifier of a Persona in the Persona store. Tag type.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PersonaId(String);

impl PersonaId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for PersonaId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

/// An Organisation state's epoch. Tag type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Epoch(u64);

impl Epoch {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for Epoch {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

/// An Envelope's Sequence number. Tag type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SequenceNumber(u64);

impl SequenceNumber {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for SequenceNumber {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
```

`org-node/src/error.rs`, after `Trie(..)`:

```rust
    /// 32 bytes offered as an Organisation public key that do not decompress
    /// to an Edwards point. LLR-mmdu38.
    #[error("invalid key: the bytes are not a point on the curve")]
    InvalidKey,

    /// A decoded Persona store, record snapshot or Join request holds a value
    /// its type's parse refuses; `field` names it. LLR-8bum44.
    #[error("invalid {field}: {reason}")]
    InvalidField { field: &'static str, reason: String },
```

`org-node/src/keys.rs`: add `use crate::types::{DeviceSeed, MemberSeed};` and

```rust
impl SigningKeypair {
    /// This key pair's seed, held as a Member seed (LLR-56hc77).
    pub fn member_seed(&self) -> MemberSeed {
        MemberSeed::from(self.0.to_bytes())
    }

    /// This key pair's seed, held as a device seed (LLR-56hc77).
    pub fn device_seed(&self) -> DeviceSeed {
        DeviceSeed::from(self.0.to_bytes())
    }
}

impl MemberSeed {
    /// The Member's signing key pair (LLR-56hc77).
    pub fn signing_keypair(&self) -> SigningKeypair {
        SigningKeypair(SigningKey::from_bytes(self.expose_secret()))
    }
}

impl DeviceSeed {
    /// The device's signing key pair (LLR-56hc77).
    pub fn signing_keypair(&self) -> SigningKeypair {
        SigningKeypair(SigningKey::from_bytes(self.expose_secret()))
    }
}
```
(put the two `SigningKeypair` methods inside the existing `impl SigningKeypair`
block; `from_seed`/`to_seed` stay until T4c).

`org-node/src/lib.rs`: add `pub mod types;` after `pub mod sequence;` and
`pub use types::{ChainAccount, DeviceSeed, Epoch, MemberSeed, OrgPublicKey, OrgSecret, PersonaId, SequenceNumber};`
after the `pub use sequence::SeqGuard;` line.

`org-node/src/test_fixtures.rs`, at the end:

```rust
/// Answers at run time whether a type implements `Display` or `Copy`, so a
/// test can assert that a secret type implements neither (LLR-sz4xhc,
/// LLR-scgk5j). Autoref specialisation: the impl on `Probe<T>` applies when
/// its bound holds, otherwise method lookup falls through to the impl on
/// `&Probe<T>`. Use through `implements_display!` / `implements_copy!`.
pub struct Probe<T>(core::marker::PhantomData<T>);

impl<T> Probe<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

pub trait ImplementsDisplay {
    fn implements_display(&self) -> bool {
        true
    }
}
impl<T: core::fmt::Display> ImplementsDisplay for Probe<T> {}

pub trait LacksDisplay {
    fn implements_display(&self) -> bool {
        false
    }
}
impl<T> LacksDisplay for &Probe<T> {}

pub trait ImplementsCopy {
    fn implements_copy(&self) -> bool {
        true
    }
}
impl<T: Copy> ImplementsCopy for Probe<T> {}

pub trait LacksCopy {
    fn implements_copy(&self) -> bool {
        false
    }
}
impl<T> LacksCopy for &Probe<T> {}

/// `implements_display!(T)`: does `T` implement `Display`?
#[macro_export]
macro_rules! implements_display {
    ($t:ty) => {{
        #[allow(unused_imports)]
        use $crate::test_fixtures::{ImplementsDisplay as _, LacksDisplay as _};
        (&$crate::test_fixtures::Probe::<$t>::new()).implements_display()
    }};
}

/// `implements_copy!(T)`: does `T` implement `Copy`?
#[macro_export]
macro_rules! implements_copy {
    ($t:ty) => {{
        #[allow(unused_imports)]
        use $crate::test_fixtures::{ImplementsCopy as _, LacksCopy as _};
        (&$crate::test_fixtures::Probe::<$t>::new()).implements_copy()
    }};
}
```

Step 4 — `cargo test -p org-node --features app,test-support --test
value_types` → 9 passed; the whole org-node verify line (cargo part) → green,
`encoding_golden` 5 passed unchanged. Red by mutation, each reverted: (a)
`secret_type!` `Debug` writing `{:?}` of `self.0` → the two Debug tests FAIL;
(b) add `impl fmt::Display for OrgSecret` → the probe assertion FAILS;
(c) `OrgPublicKey::parse` decompressing `&[0u8; 32]` instead of its argument →
`org_public_key_refuses_bytes_off_the_curve` FAILS.

Step 5 — commit `feat(org-node): secret, Organisation public key and tag value types (LLR-sz4xhc, LLR-56hc77, LLR-mmdu38, LLR-s7whrn)`.

---

### T4a — Chain values typed: `OrgPublicKey`, `Epoch`, `SequenceNumber`, `ChainAccount`

**Files touched:** `org-node/src/chain.rs`, `org-node/src/sequence.rs`,
`org-node/src/envelope.rs`, `org-node/src/verify.rs`,
`org-node/src/chain_read.rs`, `org-node/src/chain_write/calldata.rs`,
`org-node/src/chain_write/proxy.rs`, `org-node/src/chain_write/multisig.rs`,
`org-node/src/chain_write/submit.rs`, `org-node/src/ceremony.rs`,
`org-node/src/service.rs`, `org-node/src/store.rs`,
`org-node/tests/chain_read_state.rs` (new),
`org-node/tests/verify_against_chain.rs`,
`org-node/tests/transport_handshake.rs`,
`org-node/tests/transport_networked.rs`,
`org-node/tests/fuzz_verify_against_chain/fuzz_target.rs`,
`org-node/tests/chain_genesis_e2e.rs`, `org-node/tests/service_stories.rs`,
`org-node/tests/admission_sender.rs`, `org-node/tests/store_at_rest.rs`,
`org-node/Cargo.toml`, `org-node/.guardrails/config.yaml`
**Parallel:** no (serial, after T2 and T3)

**Status:** DONE — merged from `5b2f729`; org-node 73 passed (chain_read_state 2 new, encoding_golden 5 unchanged), clippy lib clean. Also touched `org-node/tests/wire_frame_bound.rs` (one `SequenceNumber::new(2)`), not in the file list above.

red -> green:
- chain_state_is_read_into_typed_values — failed to compile before the implementation (no `chain_read::org_state_from_chain`); then mutation parsing `[0u8; 32]` instead of the read key FAILED at chain_read_state.rs:27; reverted -> green.
- chain_state_with_an_off_curve_key_is_refused_as_invalid_key — failed to compile before the implementation; then the same mutation FAILED at chain_read_state.rs:38 (`unwrap_err` on `Ok`); reverted -> green.

Step 1 — write `org-node/tests/chain_read_state.rs`:

```rust
#![cfg(feature = "chain")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The chain read parses the Organisation state into org-node's types
//! (LLR-mmdu38, LLR-s7whrn): an Organisation public key that is not a curve
//! point is refused at the read as `InvalidKey`, before any Envelope is
//! verified under it.

use on_chain_client::{Epoch as ChainEpoch, OnChainRootHash, OrgPubKey, OrgState as ChainOrgState};
use org_members::RootHash;
use org_node::chain_read::org_state_from_chain;
use org_node::{Epoch, OrgNodeError};

fn chain_state(key: [u8; 32]) -> ChainOrgState {
    ChainOrgState {
        root_hash: OnChainRootHash([0x33; 32]),
        org_pub_key: OrgPubKey(key),
        epoch: ChainEpoch(7),
    }
}

/// verifies: LLR-mmdu38, LLR-s7whrn
#[test]
fn chain_state_is_read_into_typed_values() {
    let key = *ed25519_dalek::SigningKey::from_bytes(&[0x11; 32]).verifying_key().as_bytes();
    let state = org_state_from_chain(chain_state(key)).unwrap();
    assert_eq!(state.root_hash, RootHash::new([0x33; 32]));
    assert_eq!(state.org_pub_key.as_bytes(), &key);
    assert_eq!(state.epoch, Epoch::new(7));
    // A weak key decompresses, so it is read as before (owner ruling).
    assert!(org_state_from_chain(chain_state([0u8; 32])).is_ok());
}

/// verifies: LLR-mmdu38
#[test]
fn chain_state_with_an_off_curve_key_is_refused_as_invalid_key() {
    let mut off_curve = [0u8; 32];
    off_curve[0] = 2;
    assert_eq!(org_state_from_chain(chain_state(off_curve)).unwrap_err(), OrgNodeError::InvalidKey);
}
```

`Cargo.toml` (after the T3 block):

```toml
# The chain read parses the Organisation state (LLR-mmdu38).
[[test]]
name = "chain_read_state"
path = "tests/chain_read_state.rs"
required-features = ["chain"]
```

and append ` --test chain_read_state` to the org-node cargo verify line.
`cargo test -p org-node --features app,test-support --test chain_read_state` →
red: `unresolved import org_node::chain_read::org_state_from_chain`.

Step 2 — library changes.

`chain.rs`: `OrgState { pub root_hash: RootHash, pub org_pub_key: OrgPublicKey, pub epoch: Epoch }`
(import from `crate::types`); unit test: `OrgState { root_hash: RootHash::new([9u8; 32]), org_pub_key: OrgPublicKey::parse(&[0u8; 32]).unwrap(), epoch: Epoch::new(1) }`
(add `#[allow(clippy::unwrap_used)]` on that test fn if clippy asks).

`chain_read.rs`: replace `map_state` with

```rust
/// Parses an Organisation state read from the chain into org-node's types.
/// The Organisation public key is parsed here, at the read: bytes that are
/// not a curve point are refused with `InvalidKey` (LLR-mmdu38).
pub fn org_state_from_chain(s: on_chain_client::OrgState) -> Result<OrgState, OrgNodeError> {
    Ok(OrgState {
        root_hash: RootHash::new(s.root_hash.0),
        org_pub_key: OrgPublicKey::parse(&s.org_pub_key.0)?,
        epoch: Epoch::new(s.epoch.0),
    })
}
```
  and in `refresh`: `.map(org_state_from_chain).transpose().map_err(|e| e.to_string())?;`
  (imports `crate::error::OrgNodeError`, `crate::types::{Epoch, OrgPublicKey}`).

`sequence.rs`: `last_seen: SequenceNumber`; `from_last_seen(last_seen: SequenceNumber)`;
`last_seen(&self) -> SequenceNumber`; `check(&self, seq: SequenceNumber)` with
`Err(OrgNodeError::StaleSeq { got: seq.get(), last_seen: self.last_seen.get() })`;
`advance(&mut self, seq: SequenceNumber)`; `new()` → `Self { last_seen: SequenceNumber::new(0) }`.

`envelope.rs`: field `pub parent_seq: SequenceNumber`; `build(org_id, parent_seq: SequenceNumber, delta, author)`;
`transcript(org_id: &OrgId, parent_seq: SequenceNumber, delta_bytes)` writes
`parent_seq.get().to_le_bytes()`; unit tests: `build(org, SequenceNumber::new(1), …)`
and `env.parent_seq = SequenceNumber::new(env.parent_seq.get() + 1);`.

`verify.rs`: `VerifyContext::last_committed_epoch: Epoch`;
`VerifiedUpdate::epoch: Epoch`; `StaleEpoch { got: on_chain.epoch.get(), last: ctx.last_committed_epoch.get() }`.

`chain_write/calldata.rs`: `build_update_calldata` unchanged. 

```rust
pub fn revive_update_runtime_call(
    contract_h160: [u8; 20],
    new_root_hash: RootHash,
    new_org_pub_key: OrgPublicKey,
    expected_epoch: Epoch,
) -> Value {
    let calldata = build_update_calldata(
        *new_root_hash.as_bytes(),
        *new_org_pub_key.as_bytes(),
        u128::from(expected_epoch.get()),
    );
```
(rest unchanged).

`chain_write/submit.rs`: `submit_update(api, signer, contract_h160, new_root_hash: RootHash, new_org_pub_key: OrgPublicKey, expected_epoch: Epoch)`,
calldata built as above.

`chain_write/proxy.rs`: `proxied(pure_proxy: ChainAccount, call)` using
`pure_proxy.as_bytes().as_slice()`; `add_proxy_call`/`remove_proxy_call(delegate: ChainAccount)`
likewise; `create_pure(sink, api, signer, others: &[ChainAccount]) -> Result<ChainAccount, WriteError>`;
`rotate(sink, api, pure_proxy: ChainAccount, signer_old, others_old: &[ChainAccount], old_multi: ChainAccount, new_multi: ChainAccount)`;
`account32_from_named_field(..) -> Result<ChainAccount, WriteError>` (wrap with
`.map(ChainAccount::new)` before `ok_or`); add

```rust
/// The OrgId of the Organisation whose pure proxy is `p`: `h160_of(P)`.
pub fn org_id_of(p: ChainAccount) -> OrgId {
    OrgId::new(on_chain_client::h160_of(*p.as_bytes()))
}
```

`chain_write/multisig.rs`:

```rust
pub fn multi_account_id(signers: &[ChainAccount], threshold: u16) -> ChainAccount {
    let mut sorted: Vec<[u8; 32]> = signers.iter().map(|a| *a.as_bytes()).collect();
    sorted.sort();
    let entropy = (b"modlpy/utilisuba", sorted, threshold).encode();
    ChainAccount::new(blake2_256(&entropy))
}
```
  `build_dispatch_tx(other_signatories: &[ChainAccount], call)` (sort a
  `Vec<ChainAccount>` — its `Ord` is the byte order — and map each to
  `Value::from_bytes(id.as_bytes().as_slice())`); `dispatch_org_call(.., other_signatories: &[ChainAccount], ..)`;
  `fund(.., dest: ChainAccount, amount)` with `dest.as_bytes().as_slice()`;
  unit tests: `ChainAccount::new([9u8; 32])`, `ChainAccount::new([1u8; 32])`, `ChainAccount::new([2u8; 32])`.

`ceremony.rs`: `others: &[ChainAccount]`, `genesis_root: RootHash`,
`org_pub_key: OrgPublicKey`; `GenesisOutcome { pub p: ChainAccount, pub org_id: OrgId }`;
step 4 `revive_update_runtime_call(contract_h160, genesis_root, org_pub_key, Epoch::new(0))`;
`let org_id = org_id_of(p);` (import from `chain_write::proxy`).

`store.rs` — `OrgRecord` chain fields:
`root_hash: RootHash`, `org_pub_key: OrgPublicKey`, `epoch: Epoch`,
`last_seq: SequenceNumber`, `proxy_account: Option<ChainAccount>` (keep
`#[serde(default)]` and its comment; `admin_member_key`, `org_secret`,
`trie_members` unchanged until T4b/T4c).

`service.rs`:
- `ChainOps`:

```rust
    async fn submit_genesis(
        &self,
        genesis_root: RootHash,
        org_pub_key: OrgPublicKey,
    ) -> Result<(OrgId, Option<ChainAccount>), OrgNodeError>;

    async fn submit_update(
        &self,
        org_id: OrgId,
        new_root: RootHash,
        org_pub_key: OrgPublicKey,
        expected_epoch: Epoch,
        proxy_account: Option<ChainAccount>,
    ) -> Result<(), OrgNodeError>;
```
  (doc comments: "raw 32-byte AccountId32" → "the pure proxy's `ChainAccount`").
- `MockChainOps::submit_genesis`: `id_bytes.copy_from_slice(&genesis_root.as_bytes()[..20])`;
  `OrgState { root_hash: genesis_root, org_pub_key, epoch: Epoch::new(1) }`.
  `submit_update`: the mismatch message formats `expected_epoch.get()` and
  `state.epoch.get()`; new state `epoch: Epoch::new(expected_epoch.get() + 1)`,
  `root_hash: new_root`.
- `subxt_impl`: delete `map_state`; `read_state` uses
  `.map(crate::chain_read::org_state_from_chain).transpose()?` (the `?` turns
  an off-curve key into `OrgNodeError::InvalidKey`); `SubxtChainOps.others: Vec<ChainAccount>`,
  `proxy_map: Arc<Mutex<HashMap<OrgId, ChainAccount>>>`, `new(.., others: Vec<ChainAccount>)`;
  `submit_genesis`/`submit_update` signatures as the trait;
  `revive_update_runtime_call(self.contract_h160, new_root, org_pub_key, expected_epoch)`.
  `FinalitySink::settle` keeps `[u8; 32]` (block hash; `BlockSink` edge).
- `create_organisation`: `let genesis_root = trie.root_hash().map_err(OrgNodeError::Trie)?;`
  `let org_pub_key = OrgPublicKey::from(&member_kp.member_key());`
  `OrgRecord { root_hash: genesis_root, org_pub_key, epoch: Epoch::new(1), last_seq: SequenceNumber::new(0), admin_member_key: *member_kp.verifying_key().as_bytes(), .. }`.
- `export_invite`: `org_pub_key: *org_rec.org_pub_key.as_bytes()` (Invite is
  typed in T4b).
- `admit_member`: `let new_root = new_trie.root_hash().map_err(OrgNodeError::Trie)?;`
  `let new_epoch = Epoch::new(org_epoch.get() + 1);`
  `let parent_seq = SequenceNumber::new(last_seq.get() + 1);`
  `org_rec.root_hash = new_root;`. Same in `revoke_member`.
- `receive_and_verify` and `receive_and_self_delete_if_revoked`:
  `let author_vk = *chain_state.org_pub_key.verifying_key();` (the
  `VerifyingKey::from_bytes` and its `Chain("bad org_pub_key")` error go — the
  key was parsed at the read); first admission `(trie, SequenceNumber::new(0), Epoch::new(0))`;
  `m.p2p_key().as_bytes() != chain_state.org_pub_key.as_bytes()`;
  `let new_root = verified.trie.root_hash().map_err(OrgNodeError::Trie)?;`
  new `OrgRecord { org_pub_key: chain_state.org_pub_key, admin_member_key: *chain_state.org_pub_key.as_bytes(), .. }`.
- `ReceiveOutcome { pub org_id: OrgId, pub epoch: Epoch, pub root: RootHash }`.
- unit test `create_organisation_advances_mock_chain`: `assert_eq!(state.epoch, Epoch::new(1));`.

Step 3 — tests (Edit tool). Rewrite table, applied in the listed files; the
compiler lists every remaining site:

| Before | After |
|---|---|
| `OrgState { root_hash: r, org_pub_key: [0u8; 32], epoch: n }` | `OrgState { root_hash: r, org_pub_key: OrgPublicKey::parse(&[0u8; 32]).unwrap(), epoch: Epoch::new(n) }` |
| `SignedDeltaEnvelope::build(org, n, …)` / struct literal `parent_seq: n` | `SequenceNumber::new(n)` |
| `SeqGuard::from_last_seen(n)`, `g.check(n)`, `g.advance(n)` | the same with `SequenceNumber::new(n)` |
| `g.last_seen(), n` / `seq_guard.last_seen(), n` in `assert_eq!` | `SequenceNumber::new(n)` |
| `last_committed_epoch: n` | `Epoch::new(n)` |
| `out.epoch, n` / `verified.epoch, n` / `.get(&org_id).unwrap().epoch, n` / `outcome.epoch, n` / `list_orgs()[0].epoch, n` / `rec_b.epoch, n` in `assert_eq!` | right side `Epoch::new(n)` |
| `fn chain_at(org, root, epoch: u64)` (verify_against_chain) | `epoch: Epoch`, callers pass `Epoch::new(n)` |
| `outcome.root, *chain.get(&org_id).unwrap().root_hash.as_bytes()` (and `rec_b.root_hash, *…as_bytes()`) | `outcome.root, chain.get(&org_id).unwrap().root_hash` |
| `b_trie.root_hash().unwrap().as_bytes(), &root_before` | `b_trie.root_hash().unwrap(), root_before` |
| `*forger.member_key().as_bytes(), org_pub_key` (`assert_ne!`) | `forger.member_key().as_bytes(), org_pub_key.as_bytes()` |
| `SignedDeltaEnvelope::build(org_id, seq_before + 1, …)` | `SequenceNumber::new(seq_before.get() + 1)` |
| `eprintln!("… {}", verified.epoch)` / `{}` of a `SequenceNumber` | `{:?}` |
| store_at_rest `org_record`: `root_hash: [0x11u8; 32]`, `org_pub_key: [0x22u8; 32]`, `epoch: 3`, `last_seq: 2` | `RootHash::new([0x11u8; 32])`, `OrgPublicKey::parse(&[0u8; 32]).unwrap()`, `Epoch::new(3)`, `SequenceNumber::new(2)` |
| chain_genesis_e2e: `let bob_pub: [u8; 32] = dev::bob().public_key().0;` (and `alice_pub`) | `let bob_pub = ChainAccount::new(dev::bob().public_key().0);` |
| chain_genesis_e2e: `hex::encode(alice_bob_multi)`, `hex::encode(p)` | `hex::encode(alice_bob_multi.as_bytes())`, `hex::encode(p.as_bytes())` |
| chain_genesis_e2e: `let org_pub_key: [u8; 32] = admin_kp.verifying_key().to_bytes();` | `let org_pub_key = OrgPublicKey::from(&admin_kp.member_key());` |
| chain_genesis_e2e: `genesis_ceremony(…, *genesis_root.as_bytes(), org_pub_key)` | `genesis_ceremony(…, genesis_root, org_pub_key)` |
| chain_genesis_e2e: `revive_update_runtime_call(contract, *new_root.as_bytes(), org_pub_key, 1)` | `revive_update_runtime_call(contract, new_root, org_pub_key, Epoch::new(1))` |

Imports: `org_node::{Epoch, OrgPublicKey, SequenceNumber, ChainAccount}` and
`org_members::RootHash` where used. Both ceremony runs in
`chain_genesis_e2e.rs` (multisig, lines ~180–335, and single-admin, ~370–500)
get the same rewrites.

Step 4 — run: `CARGO_HOME=/tmp/cargo_home_fuzz cargo test -p org-node
--features app,test-support --no-run` (compiles every target, including
`chain_genesis_e2e`, `finality_polling` and `preflight`, which the gate does
not run) → clean; the org-node cargo verify line → green, `chain_read_state` 2
passed, `encoding_golden` 5 passed with the file unchanged (`git diff --stat
org-node/tests/encoding_golden.rs` empty); `cargo clippy -p org-node --features
app,test-support` → no more warnings than the T1 baseline. Red by mutation:
`org_state_from_chain` using `OrgPublicKey::parse(&[0u8; 32])` instead of the
read bytes → `chain_state_with_an_off_curve_key_is_refused_as_invalid_key` FAILS; revert.
`grep -n '\[u8; 32\]' org-node/src/chain.rs org-node/src/chain_read.rs org-node/src/ceremony.rs org-node/src/verify.rs org-node/src/sequence.rs org-node/src/envelope.rs` → no output.

Step 5 — commit `refactor(org-node): chain values typed — OrgPublicKey parsed at the chain read, Epoch, SequenceNumber, ChainAccount (LLR-mmdu38, LLR-s7whrn)`.

---

### T4b — Persona details, keys and identifiers typed; refusal names the field (LLR-g76zqd, LLR-8bum44)

**Files touched:** `org-node/src/store.rs`, `org-node/src/blobs.rs`,
`org-node/src/service.rs`, `org-node/tests/persona_records.rs` (new),
`org-node/tests/store_at_rest.rs`, `org-node/tests/service_stories.rs`,
`org-node/tests/admission_sender.rs`, `org-node/Cargo.toml`,
`org-node/.guardrails/config.yaml`
**Parallel:** no (serial, after T4a)

**Status:** DONE — merged from `cf7df46`; org-node 79 passed (persona_records 6 new, encoding_golden 5 unchanged). Also touched `org-node/tests/admission_sender.rs` (six `assert_ne!` now compare `.as_bytes()`; `pid_b: PersonaId`). persona_records has 6 tests, not 7.

red -> green:
- All six persona_records tests failed to compile before the implementation (17 errors: `create_persona` took `&str`, untyped record fields, no `store::seal_for_test`); green after.
- a_store_holding_an_invalid_value_is_refused_naming_the_field — also FAILED under (a) `parse_field` ignoring its field (`field: "?"`) and (b) `open` decoding `StoreData` directly (`Chain("store decode: …")`); reverted -> green.
- a_join_request_holding_an_invalid_value_is_refused_naming_the_field — also FAILED under (a); reverted -> green.

Step 1 — write `org-node/tests/persona_records.rs`:

```rust
#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The Persona's records (SDD-zs2uyt): Persona details are held parsed and
//! `create_persona` takes them typed (LLR-g76zqd); a Persona store or Join
//! request holding a value its type's parse refuses is refused whole, naming
//! the field (LLR-8bum44).

use std::path::PathBuf;

use ed25519_dalek::SigningKey;
use org_members::{Handle, MemberId, Name, OrgMembersError, P2pDeviceKey, P2pMemberKey, RootHash, Surname};
use org_node::blobs;
use org_node::ids::OrgId;
use org_node::store::{self, MemberSnapshot, OrgRecord, PersonaRecord, PersonaStatus, PersonaStore, StoreData};
use org_node::{Epoch, MockChainOps, OrgNodeError, OrgPublicKey, OrgService, PersonaId, SequenceNumber};
use rand::rngs::OsRng;

fn tmp_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ods-persona-records-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.bin"));
    let _ = std::fs::remove_file(&path);
    path
}

fn member_key(seed: u8) -> P2pMemberKey {
    P2pMemberKey::new(SigningKey::from_bytes(&[seed; 32]).verifying_key())
}

fn device_key(seed: u8) -> P2pDeviceKey {
    P2pDeviceKey::new(SigningKey::from_bytes(&[seed; 32]).verifying_key())
}

fn off_curve_key() -> [u8; 32] {
    let mut b = [0u8; 32];
    b[0] = 2;
    b
}

/// The only construction site of a `PersonaRecord` in this file.
fn persona(handle: &str) -> PersonaRecord {
    PersonaRecord {
        persona_id: PersonaId::new("p1".to_string()),
        org_id: None,
        handle: Handle::parse(handle).unwrap(),
        name: Name::parse("Alice").unwrap(),
        surname: Surname::parse("Smith").unwrap(),
        member_seed: [0x11; 32],
        device_seed: [0x12; 32],
        member_id: None,
        status: PersonaStatus::Proposed,
    }
}

/// An Organisation record with one member, whose Member key (seed 0x21) is
/// held nowhere else in the record.
fn org_with_member() -> OrgRecord {
    OrgRecord {
        org_id: OrgId::new([5u8; 20]),
        root_hash: RootHash::new([0x11u8; 32]),
        org_pub_key: OrgPublicKey::from(&member_key(0x31)),
        epoch: Epoch::new(1),
        org_secret: None,
        last_seq: SequenceNumber::new(0),
        admin_member_key: member_key(0x31),
        trie_members: vec![MemberSnapshot {
            id: MemberId::new([1u8; 32]),
            handle: Handle::parse("bob").unwrap(),
            name: Name::parse("Bob").unwrap(),
            surname: Surname::parse("Jones").unwrap(),
            member_key: member_key(0x21),
            device_keys: vec![device_key(0x22)],
        }],
        proxy_account: None,
    }
}

/// Replaces the one occurrence of `from` in `bytes` with `to` (same length).
fn replace_once(bytes: &mut [u8], from: &[u8], to: &[u8]) {
    let at: Vec<usize> = bytes.windows(from.len()).enumerate().filter(|(_, w)| *w == from).map(|(i, _)| i).collect();
    assert_eq!(at.len(), 1, "the pattern must occur exactly once");
    bytes[at[0]..at[0] + to.len()].copy_from_slice(to);
}

/// A store file holding `data` with `from` replaced by `to` in its plaintext.
fn sealed_store(name: &str, data: &StoreData, from: &[u8], to: &[u8]) -> PathBuf {
    let path = tmp_path(name);
    let mut plaintext = postcard::to_allocvec(data).unwrap();
    replace_once(&mut plaintext, from, to);
    store::seal_for_test(&path, "pw", &plaintext, &mut OsRng).unwrap();
    path
}

/// verifies: LLR-g76zqd
#[test]
fn create_persona_holds_the_parsed_details_across_a_reopen() {
    let path = tmp_path("create");
    let mut svc = OrgService::new(PersonaStore::open(path.clone(), "pw").unwrap(), Box::new(MockChainOps::new()));
    let pid = svc
        .create_persona(
            &mut OsRng,
            Handle::parse("jose\u{0301}").unwrap(),
            Name::parse("Jose\u{0301}").unwrap(),
            Surname::parse("Smith").unwrap(),
        )
        .unwrap();
    let reopened = PersonaStore::open(path, "pw").unwrap();
    let p = &reopened.data().personas[0];
    assert_eq!(p.persona_id, pid);
    assert_eq!(p.handle.as_str(), "jos\u{e9}", "stored in the NFC form Handle::parse produces");
    assert_eq!(p.name.as_str(), "Jos\u{e9}");
    let jr = OrgService::import_join_request(&svc.export_join_request(&pid).unwrap()).unwrap();
    assert_eq!(jr.handle, p.handle);
    assert_eq!(jr.name, p.name);
    assert_eq!(jr.surname, p.surname);
}

/// verifies: LLR-g76zqd
#[test]
fn persona_details_are_refused_before_a_persona_can_be_created() {
    let path = tmp_path("refuse-create");
    let mut svc = OrgService::new(PersonaStore::open(path.clone(), "pw").unwrap(), Box::new(MockChainOps::new()));
    let over = "a".repeat(129);
    let attempts = [("Alice", "Alice", "Smith"), ("alice", over.as_str(), "Smith"), ("alice", "Alice", over.as_str())];
    for (handle, name, surname) in attempts {
        let parsed = Handle::parse(handle)
            .and_then(|h| Ok((h, Name::parse(name)?, Surname::parse(surname)?)));
        assert!(
            matches!(parsed, Err(OrgMembersError::InvalidHandle(_)) | Err(OrgMembersError::FieldTooLong { .. })),
            "{handle:?}/{name:?}/{surname:?} must not parse"
        );
        if let Ok((h, n, s)) = parsed {
            svc.create_persona(&mut OsRng, h, n, s).unwrap();
        }
    }
    assert!(svc.list_personas().is_empty(), "nothing was created");
    assert!(!path.exists(), "nothing was saved");
    // The typed record's own decoding refuses an invalid handle too.
    let mut bytes = postcard::to_allocvec(&persona("alice")).unwrap();
    replace_once(&mut bytes, b"alice", b"Alice");
    assert!(postcard::from_bytes::<PersonaRecord>(&bytes).is_err());
}

/// verifies: LLR-8bum44
#[test]
fn a_store_of_valid_records_opens_with_every_field_parsed() {
    let data = StoreData { personas: vec![persona("alice")], orgs: vec![org_with_member()], pending_invites: vec![] };
    let path = sealed_store("valid", &data, b"alice", b"alice");
    let opened = PersonaStore::open(path, "pw").unwrap();
    assert_eq!(opened.data().personas[0].handle.as_str(), "alice");
    assert_eq!(opened.data().orgs[0].trie_members[0].member_key, member_key(0x21));
}

/// verifies: LLR-8bum44
#[test]
fn a_store_holding_an_invalid_value_is_refused_naming_the_field() {
    let data = StoreData { personas: vec![persona("alice")], orgs: vec![org_with_member()], pending_invites: vec![] };
    let held = member_key(0x21);
    let cases: [(&str, &[u8], Vec<u8>, &str); 2] = [
        ("handle", b"alice", b"Alice".to_vec(), "persona.handle"),
        ("key", held.as_bytes(), off_curve_key().to_vec(), "member.member_key"),
    ];
    for (name, from, to, field) in cases {
        let path = sealed_store(name, &data, from, &to);
        let before = std::fs::read(&path).unwrap();
        let err = PersonaStore::open(path.clone(), "pw").err().expect("must be refused");
        assert!(matches!(&err, OrgNodeError::InvalidField { field: f, .. } if *f == field), "got {err:?}");
        assert!(err.to_string().contains(field), "the message names the field: {err}");
        assert_eq!(std::fs::read(&path).unwrap(), before, "a refused store is left as it was");
    }
}

/// A Join request as its wire form, field by field (postcard encodes a struct
/// as the tuple of its fields).
fn join_request_blob(handle: &str, name: &str, surname: &str, member_key: [u8; 32], device_key: [u8; 32]) -> String {
    blobs::encode(&(handle, name, surname, member_key, device_key, vec![9u8, 10])).unwrap()
}

/// verifies: LLR-8bum44
#[test]
fn a_valid_join_request_imports_with_every_field_parsed() {
    let jr = OrgService::import_join_request(&join_request_blob(
        "bob", "Bob", "Jones", *member_key(0x21).as_bytes(), *device_key(0x22).as_bytes(),
    ))
    .unwrap();
    assert_eq!(jr.handle.as_str(), "bob");
    assert_eq!(jr.member_key, member_key(0x21));
    assert_eq!(jr.device_key, device_key(0x22));
    assert_eq!(jr.node_addr, vec![9, 10]);
}

/// verifies: LLR-8bum44
#[test]
fn a_join_request_holding_an_invalid_value_is_refused_naming_the_field() {
    let mk = *member_key(0x21).as_bytes();
    let dk = *device_key(0x22).as_bytes();
    let over = "a".repeat(129);
    let cases = [
        (join_request_blob("Bob", "Bob", "Jones", mk, dk), "join_request.handle"),
        (join_request_blob("bob", &over, "Jones", mk, dk), "join_request.name"),
        (join_request_blob("bob", "Bob", &over, mk, dk), "join_request.surname"),
        (join_request_blob("bob", "Bob", "Jones", off_curve_key(), dk), "join_request.member_key"),
        (join_request_blob("bob", "Bob", "Jones", mk, off_curve_key()), "join_request.device_key"),
    ];
    for (blob, field) in cases {
        let err = OrgService::import_join_request(&blob).unwrap_err();
        assert!(matches!(&err, OrgNodeError::InvalidField { field: f, .. } if *f == field), "got {err:?}");
        assert!(err.to_string().contains(field), "the message names the field: {err}");
    }
}
```

`Cargo.toml` (after the T4a block):

```toml
# The Persona's records: parsed details and field-named refusal (SDD-zs2uyt).
[[test]]
name = "persona_records"
path = "tests/persona_records.rs"
required-features = ["app", "test-support"]
```

and append ` --test persona_records` to the org-node cargo verify line. Run it
→ red: `create_persona` expects `&str`, no `seal_for_test`, no `InvalidField`
match on `import_join_request`'s result, record fields of the wrong type.

Step 2 — `store.rs`. Imports: `org_members::{Handle, MemberId, Name, P2pDeviceKey, P2pMemberKey, RootHash, Surname}`
and `crate::types::{ChainAccount, Epoch, OrgPublicKey, PersonaId, SequenceNumber}`.

```rust
/// Names the field whose parse refused a decoded value (LLR-8bum44).
/// postcard drops the message of an error raised inside `Deserialize`, so a
/// record with a fallible field is decoded as its `Raw…` mirror and parsed
/// here, field by field, instead.
pub(crate) fn parse_field<T, E: core::fmt::Display>(
    field: &'static str,
    parsed: Result<T, E>,
) -> Result<T, OrgNodeError> {
    parsed.map_err(|e| OrgNodeError::InvalidField { field, reason: e.to_string() })
}
```

Records (the typed struct keeps `#[derive(Clone, Debug, Serialize, Deserialize)]`
and gains `#[serde(try_from = "Raw…")]`; the Raw struct is
`#[derive(Deserialize)] pub(crate)`, private fields, same order):

| Typed record | Typed fields that change | Raw mirror field types | `parse_field` names |
|---|---|---|---|
| `PersonaRecord` (`try_from = "RawPersonaRecord"`) | `persona_id: PersonaId`, `handle: Handle`, `name: Name`, `surname: Surname`, `member_id: Option<MemberId>` (seeds stay `[u8; 32]` until T4c) | `persona_id: PersonaId, org_id: Option<OrgId>, handle: String, name: String, surname: String, member_seed: [u8; 32], device_seed: [u8; 32], member_id: Option<MemberId>, status: PersonaStatus` | `persona.handle`, `persona.name`, `persona.surname` |
| `MemberSnapshot` (`try_from = "RawMemberSnapshot"`) | `id: MemberId`, `handle: Handle`, `name: Name`, `surname: Surname`, `member_key: P2pMemberKey`, `device_keys: Vec<P2pDeviceKey>` | `id: MemberId, handle: String, name: String, surname: String, member_key: [u8; 32], device_keys: Vec<[u8; 32]>` | `member.handle`, `member.name`, `member.surname`, `member.member_key`, `member.device_keys` |
| `OrgRecord` (`try_from = "RawOrgRecord"`) | `admin_member_key: P2pMemberKey` (`org_secret` stays `Option<[u8; 32]>` until T4c) | `org_id: OrgId, root_hash: RootHash, org_pub_key: [u8; 32], epoch: Epoch, org_secret: Option<[u8; 32]>, last_seq: SequenceNumber, admin_member_key: [u8; 32], trie_members: Vec<RawMemberSnapshot>, #[serde(default)] proxy_account: Option<ChainAccount>` | `org.org_pub_key`, `org.admin_member_key`, and each snapshot through `MemberSnapshot::try_from` |
| `PendingInvite` (`try_from = "RawPendingInvite"`) | `admin_device_key: P2pDeviceKey`, `admin_member_key: P2pMemberKey`, `org_pub_key: OrgPublicKey` | `org_id: OrgId, admin_device_key: [u8; 32], admin_member_key: [u8; 32], org_pub_key: [u8; 32]` | `pending_invite.admin_device_key`, `pending_invite.admin_member_key`, `pending_invite.org_pub_key` |

Move `#[serde(default)]` from the typed `OrgRecord::proxy_account` to the Raw
field (the typed struct deserialises through the Raw one). The conversions, in
full for one record — the others follow the same shape:

```rust
impl TryFrom<RawPersonaRecord> for PersonaRecord {
    type Error = OrgNodeError;

    fn try_from(raw: RawPersonaRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            persona_id: raw.persona_id,
            org_id: raw.org_id,
            handle: parse_field("persona.handle", Handle::parse(&raw.handle))?,
            name: parse_field("persona.name", Name::parse(&raw.name))?,
            surname: parse_field("persona.surname", Surname::parse(&raw.surname))?,
            member_seed: raw.member_seed,
            device_seed: raw.device_seed,
            member_id: raw.member_id,
            status: raw.status,
        })
    }
}

impl TryFrom<RawMemberSnapshot> for MemberSnapshot {
    type Error = OrgNodeError;

    fn try_from(raw: RawMemberSnapshot) -> Result<Self, Self::Error> {
        Ok(Self {
            id: raw.id,
            handle: parse_field("member.handle", Handle::parse(&raw.handle))?,
            name: parse_field("member.name", Name::parse(&raw.name))?,
            surname: parse_field("member.surname", Surname::parse(&raw.surname))?,
            member_key: parse_field("member.member_key", P2pMemberKey::parse(&raw.member_key))?,
            device_keys: raw
                .device_keys
                .iter()
                .map(|k| parse_field("member.device_keys", P2pDeviceKey::parse(k)))
                .collect::<Result<_, _>>()?,
        })
    }
}
```

`OrgRecord`'s conversion maps `trie_members` with
`raw.trie_members.into_iter().map(MemberSnapshot::try_from).collect::<Result<_, _>>()?`.
`StoreData` keeps its derives (its fields now decode through the conversions)
and gains a Raw mirror for `open`:

```rust
/// The store plaintext as decoded, before parsing (LLR-8bum44).
#[derive(Deserialize)]
pub(crate) struct RawStoreData {
    personas: Vec<RawPersonaRecord>,
    orgs: Vec<RawOrgRecord>,
    pending_invites: Vec<RawPendingInvite>,
}

impl TryFrom<RawStoreData> for StoreData {
    type Error = OrgNodeError;

    fn try_from(raw: RawStoreData) -> Result<Self, Self::Error> {
        Ok(Self {
            personas: raw.personas.into_iter().map(PersonaRecord::try_from).collect::<Result<_, _>>()?,
            orgs: raw.orgs.into_iter().map(OrgRecord::try_from).collect::<Result<_, _>>()?,
            pending_invites: raw
                .pending_invites
                .into_iter()
                .map(PendingInvite::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
```

`PersonaStore::open` — move `APP_SALT` to module level
(`const APP_SALT: &[u8] = b"ods-phase2-personastore-v1______";`), factor the
cipher and the sealing so `save` and the seam share them, and decode the Raw
form:

```rust
fn cipher(passphrase: &str) -> Result<XChaCha20Poly1305, OrgNodeError> {
    let key_bytes = derive_key(passphrase, APP_SALT)?;
    XChaCha20Poly1305::new_from_slice(&key_bytes)
        .map_err(|e| OrgNodeError::Chain(format!("bad key length: {e}")))
}

/// `nonce(24) ‖ ciphertext` of `plaintext`, with a fresh random nonce.
fn seal<R: RngCore + CryptoRng>(key: &XChaCha20Poly1305, plaintext: &[u8], rng: &mut R) -> Result<Vec<u8>, OrgNodeError> {
    let mut nonce = [0u8; 24];
    rng.fill_bytes(&mut nonce);
    let ct = key
        .encrypt(XNonce::from_slice(&nonce), plaintext)
        .map_err(|e| OrgNodeError::Chain(format!("encrypt failed: {e}")))?;
    let mut out = Vec::with_capacity(24 + ct.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ct);
    Ok(out)
}

/// Writes `plaintext` to `path` encrypted exactly as `save` writes a store,
/// so a test can present content this software never writes (LLR-8bum44).
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub fn seal_for_test<R: RngCore + CryptoRng>(
    path: &std::path::Path,
    passphrase: &str,
    plaintext: &[u8],
    rng: &mut R,
) -> Result<(), OrgNodeError> {
    let sealed = seal(&cipher(passphrase)?, plaintext, rng)?;
    std::fs::write(path, sealed).map_err(|e| OrgNodeError::Chain(e.to_string()))
}
```

In `open`: `let key = cipher(passphrase)?;` and the decode becomes

```rust
            let raw: RawStoreData = postcard::from_bytes(&pt)
                .map_err(|e| OrgNodeError::Chain(format!("store decode: {e}")))?;
            StoreData::try_from(raw)?
```
`save` uses `seal(&self.key, &pt, rng)?` then writes.

Step 3 — `blobs.rs`. Imports `org_members::{Handle, Name, P2pDeviceKey, P2pMemberKey, Surname}`,
`crate::store::parse_field`, `crate::types::OrgPublicKey`.
`Invite { pub org_id: OrgId, pub org_pub_key: OrgPublicKey, pub admin_member_key: P2pMemberKey, pub admin_device_key: P2pDeviceKey, pub admin_node_addr: Vec<u8> }`
(derives unchanged; its `Deserialize` validates each key).

```rust
/// B → A: B's proposed persona, so A can mint a member_id and add B.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawJoinRequest")]
pub struct JoinRequest {
    pub handle: Handle,
    pub name: Name,
    pub surname: Surname,
    pub member_key: P2pMemberKey,
    pub device_key: P2pDeviceKey,
    /// postcard-encoded iroh EndpointAddr for dialing B.
    pub node_addr: Vec<u8>,
}

/// A Join request as decoded, before parsing (LLR-8bum44).
#[derive(Deserialize)]
pub(crate) struct RawJoinRequest {
    handle: String,
    name: String,
    surname: String,
    member_key: [u8; 32],
    device_key: [u8; 32],
    node_addr: Vec<u8>,
}

impl TryFrom<RawJoinRequest> for JoinRequest {
    type Error = OrgNodeError;

    fn try_from(raw: RawJoinRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            handle: parse_field("join_request.handle", Handle::parse(&raw.handle))?,
            name: parse_field("join_request.name", Name::parse(&raw.name))?,
            surname: parse_field("join_request.surname", Surname::parse(&raw.surname))?,
            member_key: parse_field("join_request.member_key", P2pMemberKey::parse(&raw.member_key))?,
            device_key: parse_field("join_request.device_key", P2pDeviceKey::parse(&raw.device_key))?,
            node_addr: raw.node_addr,
        })
    }
}

/// Decode a Join request blob; a value its type's parse refuses is reported
/// with the field's name (LLR-8bum44).
pub fn decode_join_request(s: &str) -> Result<JoinRequest, OrgNodeError> {
    let raw: RawJoinRequest = decode(s)?;
    JoinRequest::try_from(raw)
}
```
Unit tests: build keys with
`P2pMemberKey::new(ed25519_dalek::SigningKey::from_bytes(&[n; 32]).verifying_key())`
(and `P2pDeviceKey::new(..)`, `OrgPublicKey::from(&member)`); the Join request
fixture uses `Handle::parse("bob").unwrap()` etc. (`#[allow(clippy::unwrap_used)]`
on the module if clippy asks).

Step 4 — `service.rs`:
- `create_persona(&mut self, rng, handle: Handle, name: Name, surname: Surname) -> Result<PersonaId, OrgNodeError>`;
  `let persona_id = persona_id_for(&member_kp.member_key());` — replace `hex_id` with

```rust
/// A Persona identifier from its Member key: the first 16 bytes, in hex.
fn persona_id_for(member_key: &P2pMemberKey) -> PersonaId {
    use std::fmt::Write as _;
    PersonaId::new(member_key.as_bytes().iter().take(16).fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    }))
}
```
  `PersonaRecord { persona_id: persona_id.clone(), handle, name, surname, member_id: None, .. }`.
- `&str` persona ids become `&PersonaId` in `create_organisation`,
  `export_join_request`, `ensure_endpoint`, `find_persona`, `persona_keys`,
  `update_persona_status`; messages format `persona_id.as_str()`.
  `persona_keys` returns `(SigningKeypair, SigningKeypair, Handle, Name, Surname)`.
- `create_organisation`: `MemberLeaf::new(admin_id, handle.clone(), member_kp.member_key(), name.clone(), surname.clone(), vec![device_kp.device_key()])`;
  `MemberSnapshot { id: admin_id, handle, name, surname, member_key: member_kp.member_key(), device_keys: vec![device_kp.device_key()] }`;
  `admin_member_key: member_kp.member_key()`.
- `export_invite`: `org_pub_key: org_rec.org_pub_key, admin_member_key: org_rec.admin_member_key, admin_device_key: device_kp.device_key()`.
- `import_invite`: `PendingInvite` fields copied as typed values.
- `export_join_request`: `member_key: member_kp.member_key(), device_key: device_kp.device_key()`.
- `import_join_request(blob)` → `crate::blobs::decode_join_request(blob)`.
- `trie_from_snapshots`: no parsing left —

```rust
fn trie_from_snapshots(snapshots: &[MemberSnapshot]) -> Result<Trie, OrgNodeError> {
    let leaves = snapshots
        .iter()
        .map(|s| {
            MemberLeaf::new(s.id, s.handle.clone(), s.member_key, s.name.clone(), s.surname.clone(), s.device_keys.clone())
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(OrgNodeError::Trie)?;
    Trie::genesis(leaves).map_err(OrgNodeError::Trie)
}
```
- `first_admission_base`: decode `Vec<crate::store::RawMemberSnapshot>` and
  convert, so a refusal names its field:

```rust
    let raw: Vec<RawMemberSnapshot> = postcard::from_bytes(snap_bytes)
        .map_err(|e| OrgNodeError::Chain(format!("genesis_snapshot decode: {e}")))?;
    let snaps = raw.into_iter().map(MemberSnapshot::try_from).collect::<Result<Vec<_>, _>>()?;
    trie_from_snapshots(&snaps)
```
- add `fn snapshot_of(m: &MemberLeaf) -> MemberSnapshot { MemberSnapshot { id: *m.id(), handle: m.handle().clone(), name: m.name().clone(), surname: m.surname().clone(), member_key: *m.p2p_key(), device_keys: m.p2p_devices().to_vec() } }`
  and use it at the three `MemberSnapshot { … }` literals built from
  `members()` (receive, revoke, self-delete).
- `admit_member(.., join_request, ..) -> Result<MemberId, OrgNodeError>`: the
  two `VerifyingKey::from_bytes` go;
  `MemberLeaf::new(new_member_id, join_request.handle.clone(), join_request.member_key, join_request.name.clone(), join_request.surname.clone(), vec![join_request.device_key])`;
  Networked `iroh::EndpointId::from_bytes(join_request.device_key.as_bytes())`;
  `new_snap = MemberSnapshot { id: new_member_id, handle: join_request.handle.clone(), name: join_request.name.clone(), surname: join_request.surname.clone(), member_key: join_request.member_key, device_keys: vec![join_request.device_key] }`;
  `Ok(new_member_id)`.
- `receive_and_verify`: `if remote_device_key != inv.admin_device_key`;
  `m.has_p2p_device(&remote_device_key)`; `my_member_id … .map(|m| *m.id())`;
  first admission `admin_member_key: P2pMemberKey::new(*chain_state.org_pub_key.verifying_key())`
  with the comment `// The published signing key is the admin's Member key today (PR-szkat6).`;
  persona lookups compare `PersonaId`s.
- `revoke_member(.., member_id: MemberId, ..)`: `trie.delete_member(&member_id)`;
  `.find(|s| s.id == member_id)`; `.and_then(|s| s.device_keys.first().copied())`;
  `iroh::EndpointId::from_bytes(dk.as_bytes())`.
- `admin_persona_for_org`: `SigningKeypair::from_seed(p.member_seed).member_key() == org_rec.admin_member_key`.
- unit tests: `create_persona(&mut OsRng, Handle::parse("alice").unwrap(), Name::parse("Alice").unwrap(), Surname::parse("Smith").unwrap())`;
  `svc2.list_personas()[0].handle.as_str()`.
  `VerifyingKey` import goes if unused.

Step 5 — tests (Edit tool). In each file add next to its helpers:

```rust
fn h(s: &str) -> Handle { Handle::parse(s).unwrap() }
fn nm(s: &str) -> Name { Name::parse(s).unwrap() }
fn sn(s: &str) -> Surname { Surname::parse(s).unwrap() }
```

| Before | After |
|---|---|
| `svc.create_persona(&mut OsRng, "admin", "Admin", "User")` | `svc.create_persona(&mut OsRng, h("admin"), nm("Admin"), sn("User"))` |
| `join_request.handle, "bob"` / `jr.handle, "carol"` / `jr_c.handle, "carol"` in `assert_eq!` | `….handle.as_str(), "bob"` |
| `fn device_kp(svc: &OrgService, persona_id: &str)` / `fn keys_of(…, persona_id: &str)` | `persona_id: &PersonaId` |
| `let mut ids: Vec<[u8; 32]>` (admission_sender) | `Vec<MemberId>` |
| `MemberId::new(b_member_id)` (service_stories) | `b_member_id` (already a `MemberId`; pass `&b_member_id`) |
| `leaf_of`: `s.member_key == *a_member_kp.member_key().as_bytes()` | `s.member_key == a_member_kp.member_key()` |
| `leaf_of`: `MemberLeaf::new(MemberId::new(s.id), Handle::parse(&s.handle).unwrap(), member.member_key(), Name::parse(&s.name).unwrap(), Surname::parse(&s.surname).unwrap(), …)` | `MemberLeaf::new(s.id, s.handle.clone(), member.member_key(), s.name.clone(), s.surname.clone(), …)` |
| store_at_rest `persona`: `persona_id: "p1".into(), handle: "alice".into(), name: "A".into(), surname: "U".into()` | `persona_id: PersonaId::new("p1".to_string()), handle: Handle::parse("alice").unwrap(), name: Name::parse("A").unwrap(), surname: Surname::parse("U").unwrap()` |
| store_at_rest `org_record`: `admin_member_key: [0x33u8; 32]` | `admin_member_key: P2pMemberKey::new(ed25519_dalek::SigningKey::from_bytes(&[0x33u8; 32]).verifying_key())` |
| store_at_rest `s2.data().personas[0].handle, "alice"` | `….handle.as_str(), "alice"` |

Step 6 — run: `--no-run` compile of all targets → clean; the org-node cargo
verify line → green (`persona_records` 7, `encoding_golden` 5 unchanged,
`fuzz_first_admission_base` exit 0); clippy ≤ baseline. Red by mutation, each
reverted: (a) `parse_field` ignoring its `field` argument (`field: "?"`) → the
two refusal tests FAIL on the field name; (b) `PersonaStore::open` decoding
`StoreData` directly instead of `RawStoreData` → the store refusal test FAILS
(`Chain("store decode: SerdeDeCustom")`, not `InvalidField`).
`grep -n '\[u8; 32\]' org-node/src/service.rs org-node/src/blobs.rs org-node/src/store.rs`
→ only the Raw mirrors, the seed/secret fields T4c converts,
`FinalitySink::settle`, `fresh_member_id`'s buffer.

Step 7 — commit `refactor(org-node): Persona details, keys and identifiers typed; store and Join request refusal names the field (LLR-g76zqd, LLR-8bum44)`.

---

### T4c — Secrets in secret types; seeds become key pairs only through them (LLR-bwb9pu, LLR-scgk5j, LLR-56hc77, PR-hqwpg9)

**Files touched:** `org-node/src/keys.rs`, `org-node/src/store.rs`,
`org-node/src/transport/wire.rs`, `org-node/src/transport/endpoint.rs`,
`org-node/src/service.rs`, `org-node/src/test_fixtures.rs`,
`org-node/src/bin/preflight.rs`, `org-node/tests/secret_redaction.rs` (new),
`org-node/tests/persona_records.rs`, `org-node/tests/store_at_rest.rs`,
`org-node/tests/wire_frame_bound.rs`, `org-node/tests/service_stories.rs`,
`org-node/tests/admission_sender.rs`, `org-node/tests/transport_handshake.rs`,
`org-node/tests/transport_networked.rs`,
`org-node/tests/verify_against_chain.rs`, `org-node/tests/chain_genesis_e2e.rs`,
`org-node/tests/preflight.rs`,
`org-node/tests/fuzz_verify_against_chain/fuzz_target.rs`,
`org-node/Cargo.toml`, `org-node/.guardrails/config.yaml`
**Parallel:** no (serial, after T4b)

**Status:** DONE — merged from `7b87cd9`; org-node 84 passed (secret_redaction 4 new, encoding_golden 5 unchanged), quint clean, clippy lib clean. Also touched `transport_handshake.rs`, `transport_networked.rs` (`Some(OrgSecret::from(..))`), not in the file list above. Step 5 mutations were not run: the harness classified editing the redaction to print the bytes as security-weakening and refused it; the runtime red for LLR-bwb9pu comes from the unmodified pre-change code instead.

red -> green:
- records_and_wire_messages_never_render_secret_bytes — PR-hqwpg9 reproduction: run against the pre-change records (`[u8; 32]` fields) it FAILED at runtime ("member seed: debug output renders the secret as \"208,209,210\""; a probe showed OrgRecord and WireMessage leaking the Organisation secret too); green after.
- secrets_at_the_high_byte_bound_and_many_records_stay_unrendered — FAILED at runtime against the pre-change code ("255,254,253"); green after.
- the_store_key_renders_as_its_marker — failed to compile before the implementation (E0425, no `store_key_debug_for_test`); green after. No runtime red (mutation refused, see Status).
- the_store_key_renders_the_same_for_every_passphrase — failed to compile before the implementation (E0425); green after. No runtime red.

Step 1 — write `org-node/tests/secret_redaction.rs`:

```rust
#![cfg(all(feature = "app", feature = "test-support"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! No secret reaches debug output (REQ-y7tsft, PR-hqwpg9): a Persona record,
//! an Organisation record, the store plaintext and a Wire message holding a
//! member seed, a device seed or an Organisation secret render none of its
//! bytes in any form (LLR-bwb9pu), and the store encryption key renders as
//! its redaction marker (LLR-scgk5j).

use ed25519_dalek::SigningKey;
use org_members::{Handle, Name, P2pMemberKey, RootHash, Surname};
use org_node::ids::OrgId;
use org_node::store::{self, OrgRecord, PersonaRecord, PersonaStatus, StoreData};
use org_node::test_fixtures::admit_member_delta;
use org_node::transport::wire::WireMessage;
use org_node::{
    DeviceSeed, Epoch, MemberSeed, OrgPublicKey, OrgSecret, PersonaId, SequenceNumber,
    SignedDeltaEnvelope,
};

/// 32 bytes counting up from `start`, so each secret is distinctive.
fn sentinel(start: u8) -> [u8; 32] {
    core::array::from_fn(|i| start.wrapping_add(i as u8))
}

/// 32 bytes counting down from `start`.
fn sentinel_down(start: u8) -> [u8; 32] {
    core::array::from_fn(|i| start.wrapping_sub(i as u8))
}

/// Fails if `rendered` holds `secret` as hex (either case, from any of its
/// first four bytes), as a decimal list (`[208, 209, …]`, compact or pretty)
/// or as a hex byte list (`[0xd0, 0xd1, …]`).
fn assert_not_rendered(rendered: &str, secret: &[u8; 32], what: &str) {
    let squashed: String = rendered.chars().filter(|c| !c.is_whitespace()).collect();
    let lower: String = secret[..4].iter().map(|b| format!("{b:02x}")).collect();
    let decimal = format!("{},{},{}", secret[0], secret[1], secret[2]);
    let hex_list = format!("{:#04x},{:#04x},{:#04x}", secret[0], secret[1], secret[2]);
    for form in [lower.clone(), lower.to_uppercase(), decimal, hex_list] {
        assert!(!squashed.contains(&form), "{what}: debug output renders the secret as {form:?}");
    }
}

fn persona(member: [u8; 32], device: [u8; 32]) -> PersonaRecord {
    PersonaRecord {
        persona_id: PersonaId::new("p1".to_string()),
        org_id: None,
        handle: Handle::parse("alice").unwrap(),
        name: Name::parse("Alice").unwrap(),
        surname: Surname::parse("Smith").unwrap(),
        member_seed: MemberSeed::from(member),
        device_seed: DeviceSeed::from(device),
        member_id: None,
        status: PersonaStatus::Active,
    }
}

fn org(secret: Option<[u8; 32]>) -> OrgRecord {
    let admin = P2pMemberKey::new(SigningKey::from_bytes(&[0x31; 32]).verifying_key());
    OrgRecord {
        org_id: OrgId::new([5u8; 20]),
        root_hash: RootHash::new([0x11u8; 32]),
        org_pub_key: OrgPublicKey::from(&admin),
        epoch: Epoch::new(1),
        org_secret: secret.map(OrgSecret::from),
        last_seq: SequenceNumber::new(0),
        admin_member_key: admin,
        trie_members: vec![],
        proxy_account: None,
    }
}

fn wire(secret: [u8; 32]) -> WireMessage {
    let admin = MemberSeed::from([1u8; 32]).signing_keypair();
    let (delta, _) = admit_member_delta(&admin);
    let envelope = SignedDeltaEnvelope::build(OrgId::new([5u8; 20]), SequenceNumber::new(1), &delta, &admin).unwrap();
    WireMessage { envelope, org_secret: Some(OrgSecret::from(secret)), genesis_snapshot: None }
}

/// verifies: LLR-bwb9pu
#[test]
fn records_and_wire_messages_never_render_secret_bytes() {
    let (member, device, secret) = (sentinel(0xd0), sentinel(0x10), sentinel(0x40));
    let p = persona(member, device);
    let o = org(Some(secret));
    let data = StoreData { personas: vec![p.clone()], orgs: vec![o.clone()], pending_invites: vec![] };
    let w = wire(secret);
    for rendered in [format!("{p:?}"), format!("{p:#?}"), format!("{data:?}"), format!("{data:#?}")] {
        assert_not_rendered(&rendered, &member, "member seed");
        assert_not_rendered(&rendered, &device, "device seed");
        assert!(rendered.contains("MemberSeed([REDACTED])") && rendered.contains("DeviceSeed([REDACTED])"));
    }
    for rendered in [format!("{o:?}"), format!("{o:#?}"), format!("{data:?}"), format!("{w:?}"), format!("{w:#?}")] {
        assert_not_rendered(&rendered, &secret, "Organisation secret");
        assert!(rendered.contains("OrgSecret([REDACTED])"));
    }
}

/// verifies: LLR-bwb9pu
#[test]
fn secrets_at_the_high_byte_bound_and_many_records_stay_unrendered() {
    let personas: Vec<PersonaRecord> = (0..3u8)
        .map(|i| persona(sentinel_down(0xff - i), sentinel_down(0xef - i)))
        .collect();
    let data = StoreData { personas: personas.clone(), orgs: vec![org(Some(sentinel_down(0xdf))), org(None)], pending_invites: vec![] };
    let rendered = format!("{data:#?}");
    for (i, _) in personas.iter().enumerate() {
        let i = i as u8;
        assert_not_rendered(&rendered, &sentinel_down(0xff - i), "member seed");
        assert_not_rendered(&rendered, &sentinel_down(0xef - i), "device seed");
    }
    assert_not_rendered(&rendered, &sentinel_down(0xdf), "Organisation secret");
    assert!(rendered.contains("org_secret: None"), "an absent secret still renders as None");
}

/// verifies: LLR-scgk5j
#[test]
fn the_store_key_renders_as_its_marker() {
    assert_eq!(store::store_key_debug_for_test("hunter2").unwrap(), "StoreKey([REDACTED])");
}

/// verifies: LLR-scgk5j
#[test]
fn the_store_key_renders_the_same_for_every_passphrase() {
    let long = "p".repeat(4096);
    for passphrase in ["", "\u{1F511} ünïcødé", long.as_str()] {
        assert_eq!(store::store_key_debug_for_test(passphrase).unwrap(), "StoreKey([REDACTED])");
    }
}
```

`Cargo.toml` (after the T4b block):

```toml
# No secret reaches debug output (REQ-y7tsft, PR-hqwpg9).
[[test]]
name = "secret_redaction"
path = "tests/secret_redaction.rs"
required-features = ["app", "test-support"]
```

and append ` --test secret_redaction` to the org-node cargo verify line. Run →
red: `expected [u8; 32], found MemberSeed` at the record fields, no
`store_key_debug_for_test`. (The run-time red is shown by mutation in Step 5.)

Step 2 — library.
- `store.rs`: `PersonaRecord.member_seed: MemberSeed`, `device_seed: DeviceSeed`,
  `OrgRecord.org_secret: Option<OrgSecret>`, and the same types in
  `RawPersonaRecord`/`RawOrgRecord` (infallible, so no `parse_field`). Replace
  the doc "Keys stored as 32-byte seeds" with "Seeds held in their secret
  types; `Debug` redacts them (PR-hqwpg9)". `StoreKey`:

```rust
/// The store encryption key (LLR-scgk5j): private to this file, no
/// `Display`, not `Copy`, redacted `Debug`.
struct StoreKey([u8; 32]);

impl StoreKey {
    fn expose_secret(&self) -> &[u8; 32] {
        &self.0
    }
}

impl core::fmt::Debug for StoreKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("StoreKey([REDACTED])")
    }
}

fn derive_key(passphrase: &str, salt: &[u8]) -> Result<StoreKey, OrgNodeError> {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|e| OrgNodeError::Chain(format!("kdf failed: {e}")))?;
    Ok(StoreKey(key))
}

/// The `Debug` rendering of the key `passphrase` derives (LLR-scgk5j).
#[cfg(feature = "test-support")]
#[doc(hidden)]
pub fn store_key_debug_for_test(passphrase: &str) -> Result<String, OrgNodeError> {
    Ok(format!("{:?}", derive_key(passphrase, APP_SALT)?))
}
```
  `cipher` uses `new_from_slice(derive_key(passphrase, APP_SALT)?.expose_secret())`.
  Add a unit test in `store.rs` (not gate-read; documents the type's traits):

```rust
#[cfg(test)]
mod tests {
    use super::StoreKey;

    #[test]
    fn store_key_has_no_display_and_is_not_copy() {
        assert!(!crate::implements_display!(StoreKey));
        assert!(!crate::implements_copy!(StoreKey));
    }
}
```
- `transport/wire.rs`: `pub org_secret: Option<OrgSecret>` (doc: "the
  Organisation secret, redacted in `Debug`").
- `keys.rs`: delete `from_seed` and `to_seed`; replace the unit test
  `seed_round_trip_preserves_key` with
  `assert_eq!(kp.member_seed().signing_keypair().verifying_key(), kp.verifying_key());`.
  Module doc gains: "A seed becomes a key pair only through `MemberSeed` or
  `DeviceSeed` (LLR-56hc77)."
- `transport/endpoint.rs`: both `iroh::SecretKey::from_bytes(&device.to_seed())`
  → `iroh::SecretKey::from_bytes(device.device_seed().expose_secret())`; unit
  test `DeviceSeed::from([7u8; 32]).signing_keypair()`.
- `test_fixtures.rs`: delete `ADMIN_DEVICE_SEED`; keep its doc on
  `admin_device()`, now `DeviceSeed::from([4u8; 32]).signing_keypair()`;
  `b_member = MemberSeed::from([2u8; 32]).signing_keypair()`,
  `b_device = DeviceSeed::from([3u8; 32]).signing_keypair()`.
- `bin/preflight.rs`: `let device = DeviceSeed::from(seed).signing_keypair();`
  (env parsing to `[u8; 32]` stays: input edge).
- `service.rs`: `create_persona` stores `member_seed: member_kp.member_seed(), device_seed: device_kp.device_seed()`;
  every `SigningKeypair::from_seed(p.member_seed)` → `p.member_seed.signing_keypair()`,
  every `SigningKeypair::from_seed(p.device_seed)` / `(persona.device_seed)` →
  `….device_seed.signing_keypair()`; `ensure_endpoint` binds from
  `self.find_persona(persona_id)?.device_seed.signing_keypair()`;
  `admit_member(.., org_secret: Option<OrgSecret>)`; `receive_and_verify`
  stores `msg.org_secret.clone()` in both branches. Docs naming
  `SigningKeypair::from_seed` → `DeviceSeed::signing_keypair`.

Step 3 — tests (Edit tool):

| Before | After |
|---|---|
| `SigningKeypair::from_seed(X)` where the key pair is a device or endpoint identity (name contains `device`, passed to `OrgEndpoint::bind`, `check_transport`, or `ROGUE_SEED`) | `DeviceSeed::from(X).signing_keypair()` |
| any other `SigningKeypair::from_seed(X)` | `MemberSeed::from(X).signing_keypair()` |
| `.map(\|p\| p.device_seed)` then `SigningKeypair::from_seed(seed)` | `.map(\|p\| p.device_seed.signing_keypair())` |
| `keys_of`: `SigningKeypair::from_seed(p.member_seed)`, `(p.device_seed)` | `p.member_seed.signing_keypair()`, `p.device_seed.signing_keypair()` |
| `const ORG_SECRET: Option<[u8; 32]> = Some([0xffu8; 32]);` | `fn org_secret() -> Option<OrgSecret> { Some(OrgSecret::from([0xffu8; 32])) }`, uses `org_secret()` |
| `admit_member(…, Some([0xffu8; 32]))` | `admit_member(…, Some(OrgSecret::from([0xffu8; 32])))` |
| `list_orgs()[0].org_secret, Some([0xffu8; 32])` | `…, Some(OrgSecret::from([0xffu8; 32]))` |
| wire_frame_bound `org_secret: Some([9u8; 32])` | `Some(OrgSecret::from([9u8; 32]))` |
| store_at_rest `persona(member_seed, device_seed)` fields | `member_seed: MemberSeed::from(member_seed), device_seed: DeviceSeed::from(device_seed)` (the file searches stay on the raw arrays) |
| store_at_rest `org_record(org_secret: Option<[u8; 32]>)` | field `org_secret: org_secret.map(OrgSecret::from)` |
| persona_records `persona`: `member_seed: [0x11; 32], device_seed: [0x12; 32]` | `MemberSeed::from([0x11; 32])`, `DeviceSeed::from([0x12; 32])` |
| fuzz_verify comment "Same seed as `test_fixtures::ADMIN_DEVICE_SEED`" | "Same seed as `test_fixtures::admin_device`" |

Step 4 — `grep -rn "from_seed\|to_seed\|ADMIN_DEVICE_SEED" org-node app/src-tauri/src --include='*.rs'`
→ no output except `app/src-tauri/src` (T5).

Step 5 — run: `--no-run` → clean; the org-node verify line (cargo and quint
under `QUINT_HOME`) → green, `secret_redaction` 4, `encoding_golden` 5
unchanged; clippy ≤ baseline. Red by mutation, each reverted: (a)
`secret_type!`'s `Debug` writing `{:?}` of `self.0` →
`records_and_wire_messages_never_render_secret_bytes` FAILS ("renders the
secret as \"208,209,210\""); (b) `StoreKey`'s `Debug` derived instead of
hand-written → both `StoreKey` tests FAIL. If a redaction test fails on the
unmutated code because a sentinel's rendering also occurs in non-secret content
(a key's hex prefix, the envelope's signature or delta bytes), that is a
fixture collision, not a leak: confirm by reading the output, move that
sentinel's start value, and say so in the dispatch report.

Step 6 — commit `fix(org-node): secrets held in redacted secret types; seeds become key pairs only through them (LLR-bwb9pu, LLR-scgk5j, LLR-56hc77, PR-hqwpg9)`.

---

### T5 — The app parses user input at its edge

**Files touched:** `app/src-tauri/src/commands.rs`,
`app/src-tauri/src/state.rs`, `app/src-tauri/src/events.rs`,
`app/src-tauri/Cargo.toml`, `app/src-tauri/Cargo.lock` (only if cargo rewrites
it), `app/src-tauri/tests/ipc.rs`, `app/src-tauri/tests/receiver_events.rs`
**Parallel:** no (serial, after T4c)

org-node does not re-export `Handle`, `Name`, `Surname` or `MemberId`, and the
app has org-members only as a dev-dependency, so `org-members` moves into the
app's `[dependencies]` (same path and features as its dev entry; drop the dev
entry and its comment, or keep the comment pointing at the normal entry).

**Status:** DONE — merged from `496f7ca`; app Rust 82 passed (ipc 12, receiver_events 28), vitest 30/30, svelte-check 0 errors (1 pre-existing warning), clippy lib clean.

red -> green:
- All four tests failed to compile before the implementation (25 errors: non-exhaustive `InvalidKey`/`InvalidField` in events.rs, changed `ChainOps` signatures, typed `create_persona`/`admit_member`/`revoke_member`).
- create_persona_refuses_an_invalid_field_naming_it_and_creates_nothing — also FAILED under (a), the refusal without its `handle: ` prefix (ipc.rs:409); reverted -> green.
- a_refused_key_or_field_is_classified_as_a_receiver_error — also FAILED under (b), `InvalidKey`/`InvalidField` moved to the verdict arm (receiver_events.rs:705); reverted -> green.
- create_persona_stores_the_parsed_details, admit_member_refuses_an_org_secret_that_is_not_32_bytes — compile-red only.

Step 1 — append to `app/src-tauri/tests/receiver_events.rs` (add
`use org_node::OrgNodeError;` if absent):

```rust
// verifies: REQ-kn5rtx
#[test]
fn a_refused_key_or_field_is_classified_as_a_receiver_error() {
    // Neither is a verdict on an update: an Organisation public key the chain
    // holds that is not a curve point, or a received or stored record holding
    // a value its type refuses. Before org-node parsed these they surfaced as
    // `Chain(..)` or `Trie(..)`, both receiver errors; the class is unchanged.
    let cases = [
        OrgNodeError::InvalidKey,
        OrgNodeError::InvalidField {
            field: "member.handle",
            reason: "invalid handle: handle must be lowercase".to_string(),
        },
    ];
    for e in cases {
        assert_eq!(
            events::classify_receive_error(&e),
            ReceiverOutcome::ReceiveError { message: e.to_string() },
            "{e:?} must be a receiver error"
        );
    }
}
```

Append to `app/src-tauri/tests/ipc.rs`:

```rust
// ---------------------------------------------------------------------------
// Persona details are parsed at this boundary. REQ-qn2erx is org-node's and
// is verified there (org-node/tests/persona_records.rs); these are this
// handler's robustness tests and carry no requirement of their own.
// ---------------------------------------------------------------------------

// robustness: the handler parses the handle, name and surname before
// org-node sees them, and stores the parsed (NFC) form.
#[test]
fn create_persona_stores_the_parsed_details() {
    let h = harness();
    invoke(
        &h,
        "create_persona",
        serde_json::json!({ "handle": "jose\u{0301}", "name": "Jose\u{0301}", "surname": "Smith" }),
    )
    .expect("create_persona");
    let personas = invoke(&h, "list_personas", serde_json::json!({})).expect("list_personas");
    assert_eq!(personas[0]["handle"], "jos\u{e9}");
    assert_eq!(personas[0]["name"], "Jos\u{e9}");
}

// robustness: an invalid field is refused at the boundary, named, and
// nothing is created.
#[test]
fn create_persona_refuses_an_invalid_field_naming_it_and_creates_nothing() {
    let h = harness();
    let long = "a".repeat(129);
    let cases = [
        ("Alice", "Alice", "Smith", "handle:"),
        ("alice", long.as_str(), "Smith", "name:"),
        ("alice", "Alice", long.as_str(), "surname:"),
    ];
    for (handle, name, surname, field) in cases {
        let err = invoke_err(
            &h,
            "create_persona",
            serde_json::json!({ "handle": handle, "name": name, "surname": surname }),
        );
        assert!(err.starts_with(field), "the refusal must name the field ({field}): {err}");
    }
    let personas = invoke(&h, "list_personas", serde_json::json!({})).expect("list_personas");
    assert_eq!(personas.as_array().map(Vec::len), Some(0), "a refused persona is not created");
}

// robustness: the Organisation secret is parsed from hex at this boundary.
#[test]
fn admit_member_refuses_an_org_secret_that_is_not_32_bytes() {
    let h = harness();
    let pid = invoke(
        &h,
        "create_persona",
        serde_json::json!({ "handle": "bob", "name": "Bob", "surname": "Jones" }),
    )
    .expect("create_persona");
    let blob = invoke(&h, "export_join_request", serde_json::json!({ "personaId": pid }))
        .expect("export_join_request");
    let err = invoke_err(
        &h,
        "admit_member",
        serde_json::json!({ "orgId": org_id_40(), "joinRequestBlob": blob, "orgSecretHex": "aa".repeat(31) }),
    );
    assert!(err.contains("org_secret must be 32 bytes"), "got {err}");
}
```

Run `CARGO_HOME=/tmp/cargo_home_fuzz cargo test --manifest-path
app/src-tauri/Cargo.toml --features test-support --test ipc --test
receiver_events` → red: the crate does not compile (E0004 non-exhaustive
`OrgNodeError` in `events.rs`; `ChainOps`, `create_persona`, `admit_member`,
`revoke_member` and record-field mismatches in `state.rs`/`commands.rs`).

Step 2 — `events.rs`, the non-verdict arm of `classify_receive_error`:

```rust
        // Not a verdict: the chain could not be read, the transport failed,
        // the local store could not supply what the verification needed, or a
        // value read from the chain, the store or a received snapshot is not
        // one its type admits (InvalidKey, InvalidField — before org-node
        // parsed these they surfaced as Chain or Trie, so the class is
        // unchanged).
        OrgNodeError::Chain(_)
        | OrgNodeError::OrgNotOnChain
        | OrgNodeError::Trie(_)
        | OrgNodeError::InvalidKey
        | OrgNodeError::InvalidField { .. } => ReceiverOutcome::ReceiveError {
            message: e.to_string(),
        },
```

`state.rs`: `ChainNotConfigured` signatures as `ChainOps` (T4a:
`RootHash`, `OrgPublicKey`, `Epoch`, `Option<ChainAccount>`); the
`ODS_COSIGNER_PUB` parse yields `Vec<ChainAccount>` (`vec![ChainAccount::new(arr)]`
— the env value is app input, parsed here); `connect_chain(.., others: Vec<ChainAccount>)`.
`ODS_ADMIN_SEED` stays `[u8; 32]` (an sr25519 chain-signer seed handed to
`subxt_signer`, not an org-node secret).

`commands.rs`:
- `create_persona`:

```rust
    let handle = Handle::parse(&handle).map_err(|e| format!("handle: {e}"))?;
    let name = Name::parse(&name).map_err(|e| format!("name: {e}"))?;
    let surname = Surname::parse(&surname).map_err(|e| format!("surname: {e}"))?;
    let mut svc = state.service.lock().await;
    svc.create_persona(&mut OsRng, handle, name, surname)
        .map(|id| id.as_str().to_string())
        .map_err(|e| e.to_string())
```
  (imports `org_members::{Handle, MemberId, Name, Surname}`, with
  `org-members = { path = "../../org-members", default-features = false, features = ["std", "serde"] }`
  in `[dependencies]` of `app/src-tauri/Cargo.toml`).
- `PersonaDto::from`: `persona_id: p.persona_id.as_str().to_string()`,
  `handle: p.handle.to_string()`, `name: p.name.to_string()`, `surname: p.surname.to_string()`.
- `OrgDto::from`: `epoch: o.epoch.get()`, `root_hash: hex::encode(o.root_hash.as_bytes())`.
- `create_organisation`, `export_join_request`: `&PersonaId::new(persona_id)`.
- `import_join_request` DTO: `handle: jr.handle.to_string()` (likewise name,
  surname), `member_key: hex::encode(jr.member_key.as_bytes())`,
  `device_key: hex::encode(jr.device_key.as_bytes())`.
- `admit_member`: `iroh::EndpointId::from_bytes(jr.device_key.as_bytes())`;
  the hex parse ends in `Some(OrgSecret::from(arr))` (type
  `Option<OrgSecret>`); returns `hex::encode(member_id.as_bytes())`.
- `revoke_member`: `svc.revoke_member(&mut OsRng, oid, MemberId::new(member_id), peer_addr)`.
- `next_outcomes`: `.map(|o| (o.epoch.get(), hex::encode(o.root_hash.as_bytes())))`.

Step 3 — the app verify line `cargo test --manifest-path
app/src-tauri/Cargo.toml --features test-support --test startup_policy --test
connection_status --test org_id_parsing --test receiver_events --test
receiver_guard --test ipc` → green (+1 receiver_events, +3 ipc);
`npm --prefix app run check` and `npm --prefix app run test` → green (DTO JSON
unchanged). Red by mutation, each reverted: (a) `create_persona` passing
`format!("{e}")` without the field prefix → the refusal test FAILS;
(b) the two new arms moved to the verdict arm → the classification test FAILS.
`grep -rn "\[u8; 32\]" app/src-tauri/src` → only the env and command-input
parsing in `state.rs` and `commands.rs`.

Step 4 — commit `feat(app): parse persona details, Organisation secret and identifiers at the command boundary`.

---

### T6 — Documentation follow-through

**Files touched:** `org-node/docs/problems/2026-10-04-secret-debug.md`,
`org-node/AGENTS.md`, `org-members/AGENTS.md`,
`docs/adr/2026-10-04-parse-at-the-system-edge.md`, `org-node/docs/CONTEXT.md`,
`org-node/docs/architecture/2026-10-04-type-safety.md`,
`org-node/docs/risk/2026-10-04-type-safety.md`,
`org-node/docs/architecture/soup.md`
**Parallel:** no (serial, after T5)

Documentation only; no IDs minted.

**Status:** DONE — merged from `77f2ac2`; check-trace/check-ids clean for org-members, org-node and app, check-units clean. Also reworded a comment in `app/src-tauri/tests/ipc.rs` that named org-node's non-exported REQ-qn2erx.

red -> green: n/a (documentation).

Step 1 — PR-hqwpg9: `status: open` → `status: resolved`, and add directly
under it:

```
resolution: the member seed, device seed and Organisation secret are held in
`MemberSeed`, `DeviceSeed` and `OrgSecret`, and the store key in `StoreKey`,
each with a redacted `Debug` and no `Display` (REQ-y7tsft, RC-8a4xjb);
reproduced and gated by `records_and_wire_messages_never_render_secret_bytes`
in org-node/tests/secret_redaction.rs.
```
(keep `opened: 2026-10-04`).

Step 2 — `org-node/AGENTS.md`: replace the "**Status (2026-10-04):** org-node
does not yet follow this rule; …" paragraph with a status that org-node
follows the rule since this change, and the remaining plain `[u8; 32]` sites,
counted with `grep -c '\[u8; 32\]'` over `org-node/src` (list each file and
count as the command prints them; expected: `chain_write/*` (encoding, event
decoding), `service.rs` (`FinalitySink::settle`, `fresh_member_id`'s buffer),
`store.rs` and `blobs.rs` (the Raw decode mirrors), `types.rs` (constructors),
`bin/preflight.rs` (env parsing), `chain_write/proxy.rs` (`BlockSink`)).
Replace the code example: the edge parse becomes

```rust
// Edge: a decoded JoinRequest is parsed once, each field named on refusal...
let jr = blobs::decode_join_request(blob)?; // OrgNodeError::InvalidField { field: "join_request.handle", .. }

// ...and everything past the edge takes the newtype, never &str / [u8; 32].
fn admit_member(&mut self, rng: &mut R, org_id: OrgId, join_request: &JoinRequest,
                peer_addr: EndpointAddr, org_secret: Option<OrgSecret>) -> Result<MemberId, OrgNodeError>;

// Secret: no Display, redacted Debug, bytes only through expose_secret.
let kp = persona.member_seed.signing_keypair();
```
and add one line: "A record with a fallible field decodes through a `Raw…`
mirror and `TryFrom` (`#[serde(try_from)]`), because postcard drops the message
of an error raised inside `Deserialize`."

Step 3 — `org-members/AGENTS.md`: replace the validated-type **Status
(2026-10-04)** note with: "`P2pDeviceSlots::parse` and `P2pMemberKey::parse` /
`P2pDeviceKey::parse` (with `TryFrom`) bring the device set and the key types
under the rule (LLR-t3p9zk, LLR-k6dhz7). Key parse accepts exactly what
deserialisation accepts; refusing weak and non-canonical keys is PR-vkw22m."

Step 4 — ADR: replace the *Status 2026-10-04* note under Decision 2 with
"*Status 2026-10-04: `P2pDeviceSlots` and the key types are under this
decision (`parse` + `TryFrom`), and org-node follows it, by the org-node
type-safety change.*"

Step 5 — `org-node/docs/CONTEXT.md`, after **Organisation secret**:

```markdown
**Secret**:
A value whose holder can act as someone else: a member seed, a device seed,
the Organisation secret, or the key the Persona store is encrypted under.
Never shown in diagnostic output; given up only where it is deliberately used.
_Avoid_: key material, private key (ambiguous with the public half)
```

Step 6 — observable changes, recorded where the previous change recorded its
five (design ledger and risk file). In `org-node/docs/architecture/2026-10-04-type-safety.md`, after
LLR-8bum44, add:

```markdown
### Observable changes

Recorded 2026-10-04. Every value that was accepted and used before is still
accepted; these refusals and error values change:

- **Persona creation** takes a parsed `Handle`, `Name` and `Surname`; an
  invalid one is refused where the Persona is created (the app's
  `create_persona` command), not on the administrator's device at admission.
- **Persona store open** refuses a store holding an invalid handle, name,
  surname or key with `InvalidField { field, .. }` naming it; such a store
  used to open and fail later, at the trie rebuild (`Trie(..)` or
  `Chain("bad member key")`).
- **Join request import** refuses an invalid handle, name, surname or key with
  `InvalidField` naming it; it used to import and be refused at admission.
- **Record snapshot decode** (`first_admission_base`): an invalid field now
  reports `InvalidField` instead of `Trie(InvalidHandle)` or
  `Chain("bad member key")`. Still refused.
- **Chain read**: an Organisation public key that is not a curve point is
  refused at the read (`InvalidKey` from `SubxtChainOps::read_state`; its text
  from `OnChainReader::refresh`), instead of `Chain("bad org_pub_key")` at the
  Receive operation.
- **Invite import** refuses an Invite whose Organisation public key, Member key
  or Device key is not a curve point (`Chain("blob decode: …")`); it used to be
  stored as a pending Invite.

Design note: a record with a fallible field decodes through a crate-private
`Raw…` mirror and `TryFrom`, declared as `#[serde(try_from)]`, because postcard
discards the message of an error raised inside `Deserialize`; one parse serves
the typed decode and the field-naming load.
```

In `org-node/docs/risk/2026-10-04-type-safety.md`, under "The refactor itself", after the "A key
parsed earlier than before" bullet, add a bullet: "**Refusals that move
earlier.** The observable changes are listed in the design ledger
(`…/architecture/2026-10-04-type-safety.md`,
"Observable changes"). Beyond those assessed here, an Invite holding a key
that is not a curve point is refused at import rather than stored; such an
Invite named a key no Envelope or device could match, so nothing that worked
before is refused." — and under "Residual risk" of HAZ-uy8sxm nothing changes.

Step 7 — `org-node/docs/architecture/soup.md`: `postcard` row, append to the
last cell: "`de::Error::custom` discards its message (`SerdeDeCustom`), so a
refusal inside a nested `Deserialize` cannot name its field; `store.rs`,
`blobs.rs` and `service.rs` decode `Raw…` mirrors and parse them
(LLR-8bum44)." `serde` row, append: "Secret and tag types use
`serde(transparent)`; records with a fallible field use
`serde(try_from = \"Raw…\")`."

Step 8 — traceability, from the change worktree root:
`GR_CONFIG=org-node/.guardrails/config.yaml .guardrails/scripts/check-trace.sh`,
`GR_CONFIG=org-members/.guardrails/config.yaml .guardrails/scripts/check-trace.sh`,
`GR_CONFIG=app/.guardrails/config.yaml .guardrails/scripts/check-trace.sh` → no
MISSING-TEST for any Implements ID, no UNRESOLVED-PR for PR-hqwpg9 (other
pre-existing findings unchanged); `.guardrails/scripts/check-ids.sh
--allow-draft-files` per unit → clean; `.guardrails/scripts/check-units.sh` →
no findings. Then the three units' full verify lines and
`make coverage-org-members` → green.

Step 9 — commit `docs: resolve PR-hqwpg9; org-node type-safety status, observable changes, Secret term and SOUP notes`.

---

## Self-review

1. **Implements → tests.**
   LLR-ayrdr8: T1 `persona_store_plaintext_is_pinned`,
   `admission_wire_message_is_pinned`, `invite_and_join_request_text_is_pinned`,
   `genesis_and_update_calldata_is_pinned` (normal),
   `truncated_pinned_values_are_refused` (abnormal).
   LLR-k6dhz7: T2 `key_parse_accepts_exactly_what_decoding_accepts` (normal,
   incl. weak and non-canonical accepted), `key_parse_refuses_bytes_off_the_curve`
   (abnormal). LLR-t3p9zk: T2 `device_slots_parse_holds_keys_sorted` (normal,
   empty and MAX bounds), `device_slots_parse_refuses_too_many_and_repeated_keys`
   (abnormal).
   LLR-sz4xhc: T3 `secret_types_redact_debug_and_give_bytes_only_through_the_accessor`,
   `secret_types_serialise_as_the_plain_bytes` (normal; 31-byte abnormal),
   `secret_debug_is_the_same_whatever_the_bytes` (abnormal bounds).
   LLR-56hc77: T3 `a_seed_yields_its_key_pair_and_a_key_pair_its_seed`
   (normal), `seeds_at_the_byte_bounds_yield_working_key_pairs` (abnormal);
   removal of `from_seed`/`to_seed` in T4c (grep step).
   LLR-mmdu38: T3 `org_public_key_accepts_every_curve_point_unchanged`,
   `org_public_key_refuses_bytes_off_the_curve`; T4a
   `chain_state_is_read_into_typed_values`,
   `chain_state_with_an_off_curve_key_is_refused_as_invalid_key`.
   LLR-s7whrn: T3 `tag_types_hold_their_value_unchanged`,
   `tag_types_accept_every_boundary_value_and_serialise_as_it`; T4a
   `chain_state_is_read_into_typed_values`.
   LLR-g76zqd: T4b `create_persona_holds_the_parsed_details_across_a_reopen`
   (normal), `persona_details_are_refused_before_a_persona_can_be_created`
   (abnormal).
   LLR-8bum44: T4b `a_store_of_valid_records_opens_with_every_field_parsed`,
   `a_valid_join_request_imports_with_every_field_parsed` (normal),
   `a_store_holding_an_invalid_value_is_refused_naming_the_field`,
   `a_join_request_holding_an_invalid_value_is_refused_naming_the_field`
   (abnormal).
   LLR-bwb9pu: T4c `records_and_wire_messages_never_render_secret_bytes`
   (normal), `secrets_at_the_high_byte_bound_and_many_records_stay_unrendered`
   (abnormal). LLR-scgk5j: T4c `the_store_key_renders_as_its_marker`,
   `the_store_key_renders_the_same_for_every_passphrase`.
   REQ-y7tsft / RC-8a4xjb: transitively through LLR-sz4xhc, LLR-bwb9pu,
   LLR-scgk5j. REQ-qn2erx / RC-zutc67: through LLR-g76zqd, LLR-8bum44.
   PR-hqwpg9: reproduced by `records_and_wire_messages_never_render_secret_bytes`
   (red by mutation in T4c Step 5), resolved in T6.
2. Every task carries real test code, real implementation code or a closed
   rewrite table with the non-mechanical parts written out, exact commands and
   expected results.
3. Names consistent across tasks: `MemberSeed`, `DeviceSeed`, `OrgSecret`,
   `expose_secret`, `signing_keypair`, `member_seed()`, `device_seed()`,
   `OrgPublicKey::{parse, as_bytes, verifying_key}`, `ChainAccount::{new, as_bytes}`,
   `PersonaId::{new, as_str}`, `Epoch::{new, get}`, `SequenceNumber::{new, get}`,
   `OrgNodeError::{InvalidKey, InvalidField { field, reason }}`,
   `OrgMembersError::InvalidKey`, `P2pMemberKey::parse`, `P2pDeviceKey::parse`,
   `P2pDeviceSlots::parse`, `org_state_from_chain`, `org_id_of`, `parse_field`,
   `Raw{PersonaRecord, MemberSnapshot, OrgRecord, PendingInvite, StoreData, JoinRequest}`,
   `decode_join_request`, `seal_for_test`, `store_key_debug_for_test`,
   `implements_display!`, `implements_copy!`, `snapshot_of`, `persona_id_for`.
4. Parallel only T1 ∥ T2 and T2 ∥ T3: T1 `{encoding_golden.rs, Cargo.toml,
   config.yaml}`, T2 `{org-members/src/types.rs, error.rs, trie.rs,
   tests/newtypes.rs, tests/integration_test.rs}`, T3 `{org-node/src/types.rs,
   lib.rs, keys.rs, error.rs, test_fixtures.rs, tests/value_types.rs,
   Cargo.toml, config.yaml}` — T2 is disjoint from both; T1 and T3 share
   `Cargo.toml` and `config.yaml`, so T3 follows T1. All other tasks serial.

## Risks to watch while executing

- **The golden file is never edited.** If `encoding_golden` goes red in T2–T6,
  the refactor changed bytes or a field's meaning: stop and report; do not
  touch the constants or the assertions.
- **`#[serde(try_from)]` and Raw-mirror field order.** A Raw mirror whose
  field order differs from the typed record decodes stored bytes into the
  wrong fields; `encoding_golden`'s field reads catch it for the pinned values,
  `store_at_rest` for a round trip.
- **The app stays broken T3–T4c.** Do not "fix" the app inside an org-node
  task; T5 owns it.
- **Error-variant changes in existing tests.** `fuzz_first_admission_base`
  matches only the no-snapshot error (unchanged). If any other test asserted
  `Chain("bad member key …")`, `Chain("bad org_pub_key …")` or
  `Trie(InvalidHandle)` on a decode path, update it to the new variant and
  list it in the dispatch report as an observable change T6 must record.

## Independent review round 1 (2026-10-04) — findings and fixes

Reviewer re-ran all three units at `31263f0`: org-members 225 passed (1 ignored), org-node 84, app 82 + vitest 30. Verdict: code meets its claimed REQs/LLRs; seven trace/verification gaps. Owner ruling: fix all, run round 2; role-typed key pairs filed as PR-4b2v6p, not done here.

**finding-1**: requirement — REQ-qn2erx's "refuse to create a Persona … reporting the field" was carried out only by app code (commands.rs:96-98) with no traced test; org-node's create_persona takes parsed values; the annotated test exercised only org-members' parse.
**finding-2**: requirement — the record-snapshot refusal in first_admission_base (InvalidField member.*) and the Invite off-curve refusal had no LLR and no test.
**finding-3**: code — store-open robustness tested only 2 of the 13 field names LLR-8bum44's design promises.
**finding-4**: code — LLR-ayrdr8's calldata pin exercised only the untyped build_update_calldata, not the rewritten typed wrappers.
**finding-5**: record — the LLR-56hc77 assessment claimed it removes the member-seed-as-device-seed route; SigningKeypair is role-less, so a wrong-role key pair still passes.
**finding-6**: record — SigningKeypair's Debug relies on ed25519-dalek 2.2.0 omitting the secret, unrecorded and untested.
**finding-7**: record — the "Interface" list of observable changes omitted most public signature changes.

Fixes merged from `af066b1` (fix-r1): new LLR-q6n25z + `PersonaDetails::parse` (app uses it); LLR-8bum44 and LLR-bwb9pu amended; 13-field table test; new `update_calldata` + `calldata_typed.rs`; LLR-56hc77 assessment corrected with residual; PR-4b2v6p opened; SOUP dalek row; Interface list completed. org-node 94 passed, org-members 225, app 82 + 30.

red -> green (fix-r1):
- persona_details_parse_into_their_types_and_create_a_persona, persona_details_refuse_each_invalid_field_naming_it (LLR-q6n25z) — compile-red E0432 (`PersonaDetails` missing); green after.
- a_persona_record_decoded_directly_refuses_an_invalid_handle (LLR-g76zqd) — FAILED under a mutation lowercasing the handle before parse; reverted -> green.
- a_record_snapshot_holding_an_invalid_value_is_refused_naming_the_field (LLR-8bum44) — FAILED under decoding typed `Vec<MemberSnapshot>` directly; reverted -> green.
- a_valid_record_snapshot_decodes_with_every_field_parsed (LLR-8bum44) — FAILED under dropping device keys; reverted -> green.
- a_valid_invite_imports_as_a_pending_invite (LLR-8bum44) — FAILED under dropping the pending-invite push; reverted -> green.
- an_invite_holding_a_key_that_is_not_a_curve_point_is_refused_and_nothing_stored (LLR-8bum44) — FAILED under saving before decoding (the "nothing stored" clause); the refusal itself is structural (no off-curve typed key can be built), not mutable.
- a_store_with_any_one_field_invalid_is_refused_naming_exactly_that_field (LLR-8bum44) — FAILED under mislabelling admin_member_key; reverted -> green.
- a_store_with_every_record_kind_opens_with_every_field_parsed (LLR-8bum44) — FAILED under parsing admin_member_key from the org_pub_key bytes; reverted -> green.
- typed_update_calldata_is_the_pinned_calldata, the_revive_runtime_call_carries_the_pinned_calldata (LLR-ayrdr8) — compile-red E0432 then FAILED under a root/key swap inside the wrapper; reverted -> green.
- a_signing_key_pair_never_renders_its_seed (LLR-bwb9pu) — FAILED under a temporary Debug printing the seed ("96,97,98"); reverted -> green.

## Independent review round 2 (2026-10-04) — findings and fixes

Reviewer re-ran all three units at `8380475`: org-members 225 passed (1 ignored), org-node 94, app 82 + vitest 30; all seven round-1 findings confirmed fixed. Verdict: code meets every claimed REQ/LLR; four minor gaps.

**finding-1**: record — Persona details are canonicalised to NFC at creation (`PersonaDetails::parse`), changing what is stored, listed and exported in a Join request for non-NFC input; not recorded as an observable change and not traced.
**finding-2**: code — no test made two fields invalid at once, so the parse order LLR-q6n25z states and the record-order change recorded for snapshots were unchecked.
**finding-3**: code — LLR-scgk5j's "no Display / not Copy" clauses were checked only by an unannotated unit test in `store.rs`, outside the gate's test paths.
**finding-4**: record — the plan named draft ledger paths that no longer exist.

Fixes merged from `b839217` (fix-r2): LLR-q6n25z amended (values stored in NFC) and the NFC change recorded and assessed; four order/NFC tests; `store_key_traits_for_test` seam with an annotated test (the src unit test removed); plan paths repointed. (The T4c code block in this plan still quotes the removed src unit test as it was written at the time.)

red -> green (fix-r2):
- non_nfc_persona_details_are_stored_and_exported_in_nfc (LLR-q6n25z) — FAILED under `to_nfc` returning its input unchanged; reverted -> green.
- persona_details_with_two_invalid_fields_report_the_first (LLR-q6n25z) — FAILED under reversed parse order, and separately under a name/surname swap; reverted -> green.
- a_record_snapshot_with_an_invalid_handle_and_member_key_reports_the_handle (LLR-8bum44) — FAILED under member key parsed first; reverted -> green.
- a_store_with_two_invalid_fields_in_one_record_reports_the_first_in_record_order (LLR-8bum44) — FAILED under admin_member_key parsed first; reverted -> green.
- the_store_key_has_no_display_and_is_not_copy (LLR-scgk5j) — compile-red E0425 (seam missing); then FAILED under a temporary `#[derive(Clone, Copy)]` and under a temporary `Display` on StoreKey; reverted -> green.

## Independent review round 3 (2026-10-05) — findings and fixes

Reviewer re-ran all three units at `2a8d39c`: org-members 225 passed (1 ignored), org-node 98, app 82 + vitest 30; ran encoding_golden.rs against the pre-refactor source (`813f85b`): 5 passed. All round-1 and round-2 findings confirmed fixed.

**finding-1**: requirement — LLR-8bum44's key refusals (org, member, pending-invite, join-request keys; Invite import) satisfied no requirement: REQ-qn2erx named only handle, name and surname.
**finding-2**: code — `OnChainReader::refresh` returned early on an off-curve key and kept the previous state cached, so `get_org_state` served a superseded root after a refused refresh.
**finding-3**: record — Join-request import also canonicalises to NFC; not recorded.
**finding-4**: record — the verify-line comments in org-node and app config and the plan's Verification header did not record the new targets and counts.

Owner rulings (2026-10-05): widen REQ-qn2erx to keys; on a parse failure clear the cache (fail closed).

Fixes merged from `fec7b02` (fix-r3): REQ-qn2erx, HAZ-vfjy32, RC-zutc67 amended in place to cover keys (Invite refusal reports the Invite failed to decode, not the field); `chain_read::OrgStateCache` clears on a refused parse, LLR-mmdu38 amended; NFC Join-request import recorded and tested; config comments and the Verification header updated. org-node 100 passed.

red -> green (fix-r3):
- refresh_refused_at_parse_clears_the_cached_state (LLR-mmdu38) — FAILED against the old early-return behaviour (epoch-1 state still served, `Some(..)` vs `None`); fixed -> green.
- a_non_nfc_join_request_imports_with_its_details_in_nfc (LLR-8bum44) — FAILED under `to_nfc` made the identity; reverted -> green.

## Independent review round 4 (2026-10-05) — findings and fixes

Reviewer re-ran all three units at `9185851`: org-members 225 passed (1 ignored), org-node 100, app 82 + vitest 30; all round-1–3 findings confirmed fixed.

**finding-1**: requirement — REQ-qn2erx said a store or snapshot holding a key "the member-record rules … would refuse" is refused, but a member snapshot's device key set is not parsed as `P2pDeviceSlots`, so a repeated key or more than four opens and is refused later by org-members (`Trie(..)`).
**finding-2**: record — the plan's Implements header omitted LLR-q6n25z.

Owner ruling (2026-10-05): narrow the requirement to each key's curve-point parse. Fixed in the documents only: REQ-qn2erx, LLR-8bum44 and the REQ-qn2erx assessment amended in place; Implements header completed. No code or test changed.

## Base merged 2026-10-05: `f688a3c` (person unit)

Local master moved to `f688a3c` (feat(person)) during round 4 and was merged in, without conflicts. It brought PR-b7khyw (org-members: small-order, non-canonical and mixed-order keys accepted), the same key-acceptance anomaly as this change's PR-vkw22m. Owner ruling (2026-10-05): narrow PR-vkw22m to the part PR-b7khyw does not cover — org-node's non-strict signature check (`keys.rs`) — and move it, unmerged, to `org-node/docs/problems/2026-10-04-non-strict-verify.md`; org-members' references (LLR-k6dhz7, the key-parse risk note, AGENTS.md, two doc comments, one test comment) now point to PR-b7khyw.

## Independent review round 5 (2026-10-05) — findings and fixes

Reviewer re-ran all three units at `4f974df`: org-members 225 passed (1 ignored), org-node 100, app 82 + vitest 30; all round-1–4 findings confirmed fixed.

**finding-1**: requirement — the master merge brought the owner's ruling (recorded in `person`, REQ-7gz72r) that the Organisation public key is an X25519 key checked by `person`'s rule; this change names `OrgPublicKey` for that role but checks an ed25519 Edwards point, and recorded no conflict.
**finding-2**: record — PR-vkw22m and the SOUP row said the non-strict `verify` accepts non-canonical signatures; in ed25519-dalek 2.2.0 it differs from `verify_strict` only on a small-order public key and a small-order `R`.
**finding-3**: record — PR-4b2v6p listed `affects: REQ-y7tsft`, which a key-pair role swap does not affect.

Owner ruling (2026-10-05): record all, then round 6. Fixed in documents only: LLR-mmdu38, the *Organisation public key* glossary entry and PR-szkat6 record the Edwards rule as interim and the move to `person`'s X25519 rule as owed when PR-szkat6 is resolved (cited by path: org-node does not depend on `person`); PR-vkw22m and the SOUP row corrected; PR-4b2v6p's affects narrowed to LLR-56hc77.

## Independent review round 6 (2026-10-05) — findings and fixes; last round

Reviewer re-ran all three units at `d364a6b`: org-members 225 passed (1 ignored), org-node 100, app 82 + vitest 30; all round-1–5 findings confirmed fixed. Verdict: the code meets every claimed REQ/LLR; two record gaps. No `code` or `requirement` finding, so by the convergence rule this was the last review round.

**finding-1**: record — the risk file judged the S3/P1 residuals of HAZ-uy8sxm and HAZ-vfjy32 "acceptable", contradicting the unit's matrix and register (every S3 residual not acceptable; overall UNACCEPTABLE).
**finding-2**: record — the app moved org-members into its production dependencies for `MemberId` and `RootHash` without a declared `depends_on` edge.

Owner rulings (2026-10-05): correct both residuals to not acceptable; re-export via org-node. Fixes merged from `6f64b23` (fix-r6): both residuals restated S3/P1 not acceptable, rows added to the register's residual table; org-node re-exports `MemberId` and `RootHash`, the app imports them from `org_node`, org-members back to an app dev-dependency (`app/src-tauri/Cargo.toml` identical to master). Counts unchanged.

## Base merged 2026-10-05: `05f6f04` (org-node architecture, ratchet tooth 4)

Master moved again during the round-6 fixes: org-node's first architecture ledger (19 SDDs, 111 LLRs, measured SOUP, a derived-risk file, five new problem reports, test relocations). 12 files conflicted. Owner rulings (2026-10-05): merge and reconcile; raise org-node's `problem_open_max` to 15 (13 open after the merge).

Merged from `cd11920` (merge-master): both sides' behaviour and evidence kept, master's tests ported to the typed API (no `verifies:` annotation lost, checked by script); this change's `tests/value_types.rs` renamed `tests/node_value_types.rs`. This change's LLRs re-homed under master's SDDs — LLR-sz4xhc, LLR-s7whrn, LLR-ayrdr8 → SDD-swtd3w; LLR-56hc77 → SDD-sxp8hb; LLR-mmdu38 → SDD-pa6p7w; LLR-bwb9pu, LLR-scgk5j, LLR-g76zqd, LLR-q6n25z → SDD-af5vnt; LLR-8bum44 → SDD-vee2fq — and SDD-st9knt / SDD-zs2uyt withdrawn (never on master). Master items amended in place: LLR-e58j8m, LLR-rc74nq, SDD-swtd3w, SDD-sxp8hb, SDD-pa6p7w, SDD-af5vnt, SDD-vee2fq, SDD-z85ux9, robustness rows of SDD-89es4z and SDD-rx2yvy. SOUP tables merged. org-node 165 passed (122 master + 43 this change), org-members 225, app 82 + vitest 30; encoding_golden.rs unchanged.

## Independent review round 7 (2026-10-05) — findings and fixes

Reviewer re-ran all three units at `857777f`: org-members 225 passed (1 ignored), org-node 165, app 82 + vitest 30; master's ported tests kept their meaning; no definition on master moved; all round-1–6 fixes survive the merge.

**finding-1**: requirement — master's LLR-rm9x4z said `get_org_state` returns `None` only for an Organisation with no slot; under LLR-mmdu38 (fail closed) it returns `Ok(None)` after a refused parse.
**finding-2**: record — LLR-8bum44, LLR-g76zqd (and, found while fixing, LLR-bwb9pu) sat under items that do not own all the code they constrain; the SDD-pa6p7w section claimed `types.rs`.
**finding-3**: record — `chain_write::calldata::update_calldata` was owned by no item.
**finding-4**: requirement — LLR-sz4xhc said secret bytes leave only through `expose_secret`; they also serialise as the plain bytes (owner ruling), which a test verified without the LLR stating it.
**finding-5**: record — a measurement note misattributed the service.rs line drop to removing a test module master had already relocated.
**finding-6**: record — three stale SOUP statements (an envelope.rs cite, "the one hand-maintained decode invariant", "fifty Chain(String) sites").
**finding-7**: record — service.rs line cites in the decomposition and five open problem reports shifted by this change.
**finding-8**: record — the Interface list omitted `test_support::build_dispatch_tx`'s signature change.
**finding-9**: record — the change in Debug rendering of records (personal fields redacted, keys shortened) was not recorded.

Owner ruling (2026-10-05): fix, then run round 8. Fixed in documents only, merged from `e92fb1b` (fix-r7): LLR-rm9x4z and LLR-sz4xhc amended; LLR-mmdu38 moved to SDD-swtd3w and LLR-8bum44 to SDD-af5vnt, with "also constrained by" notes and `traces:` additions on every other owning item; `first_admission_base` assigned to SDD-8cpyfa; SDD-msb6xh names `update_calldata`; SOUP rows and the line-count note corrected; all stale line cites re-measured (decomposition, risk files, six problem reports); Interface list re-derived by script; Debug-rendering change recorded and assessed. org-node 165 passed.

## Independent review round 8 (2026-10-05) — findings and fixes; review closed

Reviewer re-ran all three units at `b43f9c3`: org-members 225 passed (1 ignored), org-node 165, app 82 + vitest 30; all round-1–7 fixes present; line cites and counts verified.

**finding-1**: requirement — the hand-written Debug of `OrgPublicKey` and `ChainAccount` was stated by no LLR and checked by no test.
**finding-2**: record — two stale comments (`membership.qnt` naming `P2pDeviceSlots::new`; `verify_against_chain.rs` naming the removed `ADMIN_DEVICE_SEED`).
**finding-3**: record — `node_value_types.rs`'s header placed LLR-mmdu38 under SDD-pa6p7w.

Owner ruling (2026-10-05): fix all, then close review (no round 9). Fixes merged from `16b6f08` (fix-r8): Debug clauses added to LLR-mmdu38 and LLR-s7whrn, assessments extended; comments and test headers corrected; org-node/AGENTS.md's placement sentence corrected. org-node 167 passed.

red -> green (fix-r8):
- org_public_key_debug_is_its_name_and_first_four_bytes (LLR-mmdu38) — FAILED with the impl replaced by `#[derive(Debug)]`; restored -> green. (Its "no full hex" assert is a backstop, never red on its own.)
- tag_types_debug_renders_their_value (LLR-s7whrn) — FAILED for ChainAccount under `derive(Debug)`, and for PersonaId, Epoch and SequenceNumber under a bare-value impl; restored -> green.

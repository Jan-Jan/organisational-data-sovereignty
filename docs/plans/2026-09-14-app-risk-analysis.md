# app Risk Analysis — Implementation Plan

**Goal:** Give the `app` unit its ISO 14971 hazard register, the requirements
realising its controls, and the first tests it has ever had — on both sides of
the IPC boundary — and fix the five defects the analysis found that are fixable
from inside the unit.

**Implements:** HAZ-n97v5g, HAZ-9fmhm4, HAZ-5ha5vv, HAZ-8ghmhn, HAZ-ny7yvt,
HAZ-cfp4jb, RC-8ygnjd, RC-jrkn7w, RC-6ajdv4, RC-h7mnfj, RC-2t9sgv, RC-9t3kpm,
RC-xt4qr3, RC-djzms3, RC-a7fenm, RC-nyy73d, RC-8abufw, REQ-645jq9, REQ-vgr7s2,
REQ-he8ejb, REQ-affyf5, REQ-a83vqr, REQ-akt4p7, REQ-rq8g2v, REQ-2k7ys4,
REQ-dp95pv, REQ-tw4cb5, REQ-7g3k9a, REQ-bmk2z2, REQ-rxc8sp, REQ-e4ah9h,
REQ-bvx4nh, REQ-3hfggn, REQ-6hgm8r, REQ-jfxah3, REQ-sjkp8z, PR-u34uqm,
RC-3rddh7, REQ-kn5rtx, REQ-wu6z9p

**Forty IDs, not the thirty-seven this list carried until 2026-09-15.** The last
three were minted by the first independent review, which found that the controls
for HAZ-9fmhm4 had created a second instance of that hazard pointing the other
way (the register's §5, fourth entry). They were implemented and gated at the
time and this list was not updated — so five successive gate rounds reported
"37/37 Implements IDs mapped" against an enumeration three short of the change,
and the shortfall was invisible precisely because the map was measured from the
tests rather than from the list. Found by gate round 8.

REQ-x3c8n2 is an expectation on `org-node` (`expects:`), not a behaviour this
change implements, and is deliberately absent from the list above.

**Safety class:** C (`app/.guardrails/config.yaml`). The robustness rule
applies to every ID above: normal-case *and* abnormal-input tests, both.

**Verification:** `app/.guardrails/config.yaml` `verify_commands`, as this
change leaves them:

    cargo test --manifest-path app/src-tauri/Cargo.toml --features test-support \
      --test startup_policy --test connection_status --test org_id_parsing \
      --test receiver_events --test receiver_guard --test ipc
    npm --prefix app run check
    npm --prefix app run test

---

## Two test names in this plan were superseded during the change — read this first

The task sections below are the plan **as written**, and two test names in them
no longer exist. Both were renamed by later fix rounds because the name claimed
more than the test's body established, which is the defect class the
verification gate kept finding. The fix sections at the end explain each; the
mapping is here so a reader working forward from the tasks is not sent looking
for a name that is gone.

| written in the tasks below | actual, after fix2 / fix3 |
|---|---|
| `unconfigured_chain_reports_false_even_with_env_set` (§T1 step 1.6, twice) | `absent_chain_reports_false_and_no_endpoint_fields_whatever_the_ambient_env` (`connection_status.rs`) |
| `revoke_member_accepts_an_empty_peer_addr` (§T2 step 2.1, its worked example, and T2's mutation table) | `revoke_member_is_not_refused_for_an_empty_peer_addr` (`ipc.rs`) |

The worked example in §T2 step 2.1 also carries the assertion message
`"an empty peer address must be accepted in Networked transport"`, which fix3
deliberately retracted — the code now reads `"…must not be the ground of
refusal in Networked transport…"`, because the test requires the call to fail
and asserts only that the address was not the reason. Read the example for its
shape, not for its strings.

Occurrences further down, inside the dated `T1 — done` / `T2 — done` blocks and
the fix sections, are left as written: those are attestations of what was
observed at the time, and editing them would falsify a record.

## The shape of the change, and why it is shaped that way

Every decision this unit makes sits in a place no test can reach: inside an
`async` Tauri command handler that needs a running application, or inside a
Svelte `$effect` that needs a browser. So the analysis found defects that a
test would have caught, in code no test could have been written against.

The fix is structural and it is the same move `on-chain-client` made for
`log_is_ours` on 2026-09-11: **extract the decision into a free function, gate
the function, and leave the handler a wrapper.** That is what T1 and T3 do, on
the Rust and frontend sides respectively. T2 and T4 then rewire the callers.

Two properties that fall out of this, and both are load-bearing:

- **No test may call `std::env::set_var`.** Cargo runs integration tests in
  threads of one process; a test that mutates the environment races every other
  test in the binary, and the failure is intermittent and blamed on the wrong
  test. Every policy function below therefore takes its inputs as parameters
  and the *caller* reads the environment. This is not a testability
  concession — it is what makes the functions total and the tests
  deterministic.
- **`AppState` gets a test-only constructor.** `AppState::init` reads the
  environment and opens a real store. The IPC suite in T2 builds its state
  directly instead, behind `#[cfg(feature = "test-support")]`.

### What is deliberately NOT in this change

- No `coverage_command` for this unit (owner's decision, 2026-09-14: tooth 5,
  one basis for `org-node` and `app` together).
- No component-rendering harness. Every frontend test below tests an extracted
  function, never a component. This is the change's largest gap and it is
  recorded as not-minted control 1 with a date.
- No fix for the four open problem reports (PR-w5dae4, PR-h6xpnh, PR-eecx3y,
  and PR-5mc4d8, which the verification gate opened after this list was
  written).
- No change to the chain fallback itself: the app still starts without chain
  operations. Only the *reporting* of that state changes.

---

## Preconditions for every dispatched task

1. **A task worktree, nested.** `.worktrees/worktree-guardrails-app-risk-t<N>`,
   on branch `worktree-guardrails-app-risk-t<N>`, off
   `worktree-guardrails-app-risk`. The path is named in the dispatch prompt;
   it is not the subagent's to choose.
2. **Copy `app/node_modules` in.** It is 92 MB, it is gitignored, and a fresh
   task worktree does not have it. Any task running `npm` must first:

       cp -R <change-worktree>/app/node_modules <task-worktree>/app/node_modules

   Without it `npm run check` and `npm run test` try to refetch and the task
   stalls or fails. It is gitignored, so it cannot dirty `git status`.
3. **Cargo in this sandbox:** `CARGO_HOME=/tmp/cargo_home_fuzz`, never
   `--offline`, always foreground, logs outside the repository tree.
4. Commit on the task branch, unsigned
   (`git -c commit.gpgsign=false commit`). Do not merge. Return the dispatch
   report.

---

## T1 — Rust: the extracted decisions, test-first

**Files touched:**
`app/src-tauri/Cargo.toml`,
`app/src-tauri/src/lib.rs`,
`app/src-tauri/src/policy.rs` (new),
`app/src-tauri/src/parsing.rs` (new),
`app/src-tauri/src/events.rs` (new),
`app/src-tauri/tests/startup_policy.rs` (new),
`app/src-tauri/tests/connection_status.rs` (new),
`app/src-tauri/tests/org_id_parsing.rs` (new),
`app/src-tauri/tests/receiver_events.rs` (new),
`app/src-tauri/tests/receiver_guard.rs` (new)

**Parallel:** yes (with T3)

**Implements:** REQ-7g3k9a, REQ-bmk2z2, REQ-rxc8sp, REQ-645jq9, REQ-e4ah9h,
REQ-bvx4nh, REQ-sjkp8z, REQ-2k7ys4, REQ-dp95pv, REQ-tw4cb5, REQ-affyf5,
REQ-jfxah3, REQ-3hfggn, REQ-6hgm8r

This task adds three new modules and five new test targets. **It changes no
existing behaviour** — `state.rs` and `commands.rs` are untouched here and are
rewired in T2. That split is what makes T1 pure TDD: every function below is
new, so every test can be written first and watched failing to compile or
failing its assertion before the function exists.

### Step 1.1 — `Cargo.toml`

Add, after `[dependencies]`:

```toml
[features]
# Gates the test-only constructors and the integration test targets below.
# Every `[[test]]` entry declares it via `required-features`, so naming a
# target with the feature off is a cargo ERROR rather than a silent skip —
# the property on-chain-client's nine targets rely on.
test-support = []

[dev-dependencies]
# `tauri/test` supplies mock_builder / mock_context / get_ipc_response, used by
# the IPC suite in T2. Feature unification means the dev build of the lib also
# sees `test`; that is harmless and is why this is a dev-dependency rather than
# an optional feature of the main entry.
tauri = { version = "2", features = ["test"] }
tempfile = "3"

[[test]]
name = "startup_policy"
required-features = ["test-support"]

[[test]]
name = "connection_status"
required-features = ["test-support"]

[[test]]
name = "org_id_parsing"
required-features = ["test-support"]

[[test]]
name = "receiver_events"
required-features = ["test-support"]

[[test]]
name = "receiver_guard"
required-features = ["test-support"]

[[test]]
name = "ipc"
required-features = ["test-support"]
```

The `ipc` entry is declared here even though T2 writes the file, so that T1 and
T2 do not both edit `Cargo.toml`. **T1 must therefore also create
`app/src-tauri/tests/ipc.rs` as a placeholder containing exactly one line:**

```rust
// Populated by T2. Declared in Cargo.toml by T1 so the two tasks do not share a file.
```

A `[[test]]` target whose file does not exist is a cargo error, so the
placeholder is required, not tidiness.

### Step 1.2 — `lib.rs`: declare the modules

Add to the existing `pub mod` block, and change nothing else in this file:

```rust
pub mod commands;
pub mod events;
pub mod parsing;
pub mod policy;
pub mod state;
```

### Step 1.3 — `policy.rs`

This module owns every startup decision and the connection-status projection.
`ConnectionStatus` and `connection_status_from_state` **move here from
`state.rs`**; T2 makes `state.rs` re-export them so no other caller changes.

```rust
//! Startup policy and the connection-status projection.
//!
//! Every function here is total and takes its inputs as parameters: none reads
//! the process environment. The caller (`AppState::init`) does the reading.
//! That is deliberate — cargo runs integration tests in threads of one
//! process, so a test that set an environment variable would race every other
//! test in the binary.

use std::path::PathBuf;

/// Which of the two ways this installation reaches other nodes. Mirrors
/// org_node::transport::TransportMode, but is this unit's own type because it
/// is serialised across the IPC boundary and org-node's is not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportModeName {
    Networked,
    Loopback,
}

/// The chain endpoint the running configuration was actually built from.
/// Present only when `build_chain_ops` succeeded (REQ-bvx4nh).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainEndpoint {
    pub ws_url: String,
    /// The contract address as the operator supplied it, 0x-prefixed hex.
    pub contract_h160: String,
}

/// A startup value that is missing and is not permitted to be defaulted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartupError {
    MissingPassphrase,
    UnresolvableDataDir,
}

impl core::fmt::Display for StartupError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // REQ-bmk2z2: the message names BOTH the variable that would supply the
        // value AND the variable that would waive the requirement, and names the
        // supplying variable FIRST — a refusal an operator reads as "set the
        // waiver" is a refusal that recreates the hazard it was added for.
        match self {
            StartupError::MissingPassphrase => write!(
                f,
                "ODS_PASSPHRASE is not set. Set it to the passphrase protecting \
                 this installation's persona store. To run with the built-in \
                 development passphrase instead, set ODS_ALLOW_DEV_DEFAULTS=1 — \
                 do not do this on an installation holding real key material."
            ),
            StartupError::UnresolvableDataDir => write!(
                f,
                "The application data directory could not be resolved and \
                 ODS_DATA_DIR is not set. Set ODS_DATA_DIR to the directory \
                 this installation should keep its persona store in. To fall \
                 back to a temporary directory instead, set \
                 ODS_ALLOW_DEV_DEFAULTS=1 — that directory is world-readable \
                 and is cleared by the operating system."
            ),
        }
    }
}

/// The built-in passphrase. Public so the test can assert on the exact value
/// rather than restating the literal, and so a future reader greps one place.
pub const DEV_PASSPHRASE: &str = "ods-dev-default";

/// The built-in data directory fallback.
pub const DEV_DATA_DIR: &str = "/tmp/ods-poc";

/// REQ-7g3k9a. `configured` is `ODS_PASSPHRASE`; `allow_dev_defaults` is
/// whether `ODS_ALLOW_DEV_DEFAULTS` was set.
pub fn resolve_passphrase(
    configured: Option<&str>,
    allow_dev_defaults: bool,
) -> Result<String, StartupError> {
    match configured {
        // An empty ODS_PASSPHRASE is treated as absent, not as an empty
        // passphrase: a variable set to "" is overwhelmingly a scripting
        // accident, and honouring it would be the original hazard with an
        // extra step.
        Some(p) if !p.is_empty() => Ok(p.to_string()),
        _ if allow_dev_defaults => Ok(DEV_PASSPHRASE.to_string()),
        _ => Err(StartupError::MissingPassphrase),
    }
}

/// REQ-rxc8sp. `override_dir` is `ODS_DATA_DIR`; `resolved` is what the
/// platform path resolver returned (`None` on error).
pub fn resolve_data_dir(
    override_dir: Option<&str>,
    resolved: Option<PathBuf>,
    allow_dev_defaults: bool,
) -> Result<PathBuf, StartupError> {
    if let Some(d) = override_dir.filter(|d| !d.is_empty()) {
        return Ok(PathBuf::from(d));
    }
    if let Some(p) = resolved {
        return Ok(p);
    }
    if allow_dev_defaults {
        return Ok(PathBuf::from(DEV_DATA_DIR));
    }
    Err(StartupError::UnresolvableDataDir)
}

/// The transport mode, from `ODS_TRANSPORT`. Anything other than the literal
/// "loopback" is Networked, preserving the existing behaviour exactly.
pub fn transport_mode_from(raw: Option<&str>) -> TransportModeName {
    match raw {
        Some("loopback") => TransportModeName::Loopback,
        _ => TransportModeName::Networked,
    }
}

/// Connection status reported by the `connection_status` command.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConnectionStatus {
    /// True only when a real chain client was constructed at startup.
    /// This is a startup fact, not a liveness fact — see PR-eecx3y.
    pub chain_configured: bool,
    /// REQ-e4ah9h / REQ-bvx4nh: the endpoint the running configuration was
    /// built from, or absent. Never re-read from the environment.
    pub chain_ws: Option<String>,
    pub contract_h160: Option<String>,
    /// REQ-645jq9.
    pub transport_mode: TransportModeName,
    pub data_dir: String,
}

/// REQ-e4ah9h, REQ-bvx4nh, REQ-645jq9.
pub fn connection_status_from_state(
    data_dir: &std::path::Path,
    chain: Option<&ChainEndpoint>,
    transport_mode: TransportModeName,
) -> ConnectionStatus {
    ConnectionStatus {
        chain_configured: chain.is_some(),
        chain_ws: chain.map(|c| c.ws_url.clone()),
        contract_h160: chain.map(|c| c.contract_h160.clone()),
        transport_mode,
        data_dir: data_dir.display().to_string(),
    }
}
```

**Note the shape of `connection_status_from_state`:** because the endpoint and
the verdict come from the same `Option`, REQ-e4ah9h is true *by construction* —
there is no state in which `chain_configured` is false and `chain_ws` is
`Some`. The test still asserts it, because the requirement is about observable
behaviour and a later edit could reintroduce two independent fields.

### Step 1.4 — `parsing.rs`

```rust
//! Input parsing at the command boundary.

use org_node::OrgId;

/// REQ-sjkp8z. Accepts 40 hex characters, with or without a `0x` prefix.
pub fn parse_org_id(s: &str) -> Result<OrgId, String> {
    let s = s.trim_start_matches("0x");
    if s.len() != 40 {
        return Err(format!("org_id must be 40 hex chars, got {}", s.len()));
    }
    let bytes = hex::decode(s).map_err(|e| format!("org_id hex: {e}"))?;
    let mut arr = [0u8; 20];
    arr.copy_from_slice(&bytes);
    Ok(OrgId::new(arr))
}
```

This is the existing private function in `commands.rs`, moved verbatim and made
public. T2 deletes the original and imports this one. The behaviour is
unchanged and the test is the first one it has ever had.

### Step 1.5 — `events.rs`

```rust
//! What the receiver loop announces, and the guard that decides whether a
//! receiver loop may start.
//!
//! `emissions_for` is a total function from an outcome to the events that
//! outcome produces. It exists so the receiver's announcements can be tested
//! without a Tauri application, an OrgService, a chain or a peer — none of
//! which any gate in this repository can stand up.

use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// What one iteration of the receiver loop concluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiverOutcome {
    /// This persona was revoked and deleted itself.
    SelfDeleted { org_id: String },
    /// An update verified and the organisation record was read back.
    Updated { org_id: String, epoch: u64, root: String },
    /// REQ-2k7ys4 / REQ-dp95pv: an update verified and the record could NOT be
    /// read back. This variant is what stops `(0, String::new())` existing.
    RecordUnreadable { org_id: String },
    /// REQ-affyf5: verification failed. `org_id` is present when the failing
    /// update named an organisation and absent when the failure happened
    /// before one was identified.
    VerifyFailed { org_id: Option<String>, message: String },
    /// REQ-jfxah3: the loop is about to exit.
    Stopped { reason: String },
}

/// One event to emit: a name and a payload.
#[derive(Debug, Clone, PartialEq)]
pub struct Emission {
    pub name: &'static str,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
struct MembershipUpdated<'a> { org_id: &'a str, epoch: u64, root: &'a str }
#[derive(Debug, Clone, Serialize)]
struct OrgOnly<'a> { org_id: &'a str }
#[derive(Debug, Clone, Serialize)]
struct EpochChanged<'a> { org_id: &'a str, epoch: u64 }
#[derive(Debug, Clone, Serialize)]
struct VerifyFailed<'a> { org_id: Option<&'a str>, message: &'a str }
#[derive(Debug, Clone, Serialize)]
struct Stopped<'a> { reason: &'a str }

fn json<T: Serialize>(v: &T) -> serde_json::Value {
    serde_json::to_value(v).expect("event payloads are plain structs of owned scalars")
}

/// The events one outcome produces, in emission order.
pub fn emissions_for(outcome: &ReceiverOutcome) -> Vec<Emission> {
    match outcome {
        // REQ-tw4cb5: NO epoch-changed event here. The record is gone; there is
        // no epoch to report; the previous code emitted a literal 0, which is
        // genesis and therefore a reachable, meaningful value (HAZ-5ha5vv).
        ReceiverOutcome::SelfDeleted { org_id } => vec![Emission {
            name: "revoked",
            payload: json(&OrgOnly { org_id }),
        }],

        ReceiverOutcome::Updated { org_id, epoch, root } => vec![
            Emission {
                name: "membership-updated",
                payload: json(&MembershipUpdated { org_id, epoch: *epoch, root }),
            },
            Emission {
                name: "incoming-verified",
                payload: json(&MembershipUpdated { org_id, epoch: *epoch, root }),
            },
            Emission {
                name: "epoch-changed",
                payload: json(&EpochChanged { org_id, epoch: *epoch }),
            },
        ],

        // REQ-2k7ys4: no membership-updated, no incoming-verified, no
        // epoch-changed. REQ-dp95pv: an event that says so, naming the org.
        ReceiverOutcome::RecordUnreadable { org_id } => vec![Emission {
            name: "record-unreadable",
            payload: json(&OrgOnly { org_id }),
        }],

        ReceiverOutcome::VerifyFailed { org_id, message } => vec![Emission {
            name: "verification-failed",
            payload: json(&VerifyFailed { org_id: org_id.as_deref(), message }),
        }],

        ReceiverOutcome::Stopped { reason } => vec![Emission {
            name: "receiver-stopped",
            payload: json(&Stopped { reason }),
        }],
    }
}

/// REQ-6hgm8r / REQ-3hfggn. Claims the right to run a receiver loop, and
/// releases it on drop — however the loop ends, including by panic.
///
/// The previous code set the flag before spawning and never cleared it, so an
/// exited loop kept the slot forever and no restart was possible (HAZ-cfp4jb).
pub struct StartGuard(Arc<AtomicBool>);

impl StartGuard {
    /// `Some` if no loop is running and this caller may start one; `None` if
    /// one is already running.
    pub fn try_claim(flag: &Arc<AtomicBool>) -> Option<StartGuard> {
        flag.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .ok()
            .map(|_| StartGuard(Arc::clone(flag)))
    }
}

impl Drop for StartGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
```

### Step 1.6 — the five test targets

Each target begins with a header comment naming the requirements it carries,
and **every test carries its own `verifies:` line**. The counts below are what
the task must produce; the bodies are ordinary assertions and are not written
out here except where the assertion is subtle.

`tests/startup_policy.rs` — **12 tests.**

| test | verifies |
|---|---|
| `configured_passphrase_is_used` | REQ-7g3k9a |
| `absent_passphrase_is_refused` | REQ-7g3k9a |
| `empty_passphrase_is_refused_not_honoured` | REQ-7g3k9a |
| `absent_passphrase_with_opt_in_uses_dev_default` | REQ-7g3k9a |
| `configured_passphrase_wins_over_opt_in` | REQ-7g3k9a |
| `override_data_dir_is_used` | REQ-rxc8sp |
| `resolved_data_dir_is_used_when_no_override` | REQ-rxc8sp |
| `unresolvable_data_dir_is_refused` | REQ-rxc8sp |
| `unresolvable_data_dir_with_opt_in_uses_temp` | REQ-rxc8sp |
| `empty_override_is_ignored_not_used_as_path` | REQ-rxc8sp |
| `passphrase_refusal_names_both_variables_supplier_first` | REQ-bmk2z2 |
| `data_dir_refusal_names_both_variables_supplier_first` | REQ-bmk2z2 |

The last two are the ones worth writing out, because "names both variables" is
the whole requirement and a test that only checks for a substring would pass on
a message that named them in the wrong order:

```rust
// verifies: REQ-bmk2z2
#[test]
fn passphrase_refusal_names_both_variables_supplier_first() {
    let msg = policy::resolve_passphrase(None, false).unwrap_err().to_string();
    let supplier = msg.find("ODS_PASSPHRASE").expect("names the supplying variable");
    let waiver = msg.find("ODS_ALLOW_DEV_DEFAULTS").expect("names the waiver");
    // Order matters: an operator who reads the waiver first sets the waiver,
    // which recreates HAZ-8ghmhn. REQ-bmk2z2 is satisfied by naming both; this
    // assertion is the register's reasoning made executable.
    assert!(supplier < waiver, "the supplying variable must be named first: {msg}");
}
```

`tests/connection_status.rs` — **8 tests as planned; 10 as built.** fix6 added
`data_dir_comes_from_the_built_configuration` and
`abnormal_data_dir_paths_are_reported_verbatim`, both `verifies: REQ-bvx4nh`,
after the second independent review found that requirement's data-directory
clause gated by no test annotated to it.

| test | verifies |
|---|---|
| `configured_chain_reports_its_endpoint_and_contract` | REQ-e4ah9h |
| `unconfigured_chain_reports_neither` | REQ-e4ah9h |
| `unconfigured_chain_reports_false_even_with_env_set` | REQ-bvx4nh |
| `endpoint_comes_from_the_built_configuration` | REQ-bvx4nh |
| `networked_transport_is_reported` | REQ-645jq9 |
| `loopback_transport_is_reported` | REQ-645jq9 |
| `transport_mode_serialises_lowercase` | REQ-645jq9 |
| `unknown_transport_value_is_networked` | REQ-645jq9 |

`unconfigured_chain_reports_false_even_with_env_set` is the abnormal-input case
that pins REQ-bvx4nh, and it is the one test in this file that must **not** set
an environment variable to make its point. It passes `None` for the chain while
the ambient environment may or may not have `ODS_CHAIN_WS` set, and asserts
`chain_ws == None`. Because the function no longer reads the environment at
all, the assertion holds either way — and it would fail against the old
`connection_status_from_state`, which is the point.

`tests/org_id_parsing.rs` — **11 tests as planned; 14 as built.** The three the
list below does not name are fix4's, added after the verification gate found
that `trim_start_matches("0x")` strips the prefix repeatedly:
`a_doubled_zero_x_prefix_is_refused`, `a_zero_x_in_the_middle_is_refused` and
`the_bare_prefix_alone_is_refused`, all `verifies: REQ-sjkp8z`. The note is here
for the same reason as `receiver_events`'s below — this list is the only index
of REQ-sjkp8z's evidence in the plan's task sections, and a reader using it
would otherwise find 11 tests where 14 exist. (`ipc.rs` likewise gained
`revoke_member_rejects_a_doubled_zero_x_prefix_on_the_member_id`, 8 → 9,
`verifies: REQ-he8ejb`.)

REQ-sjkp8z throughout: the 40-char
accept, the `0x`-prefixed accept, uppercase hex, empty, 39, 41, odd length,
non-hex at the first position, non-hex at the last position, a 40-char string
of non-hex, and a `0x` prefix with 40 characters after it (which is 42 total
and must be accepted, because the code strips the prefix before measuring).

`tests/receiver_events.rs` — **14 tests as planned; 27 as built.** Thirteen of
the twenty-seven are absent from the table below, added across three later
rounds: fix5's six for REQ-kn5rtx, fix7's four for REQ-jfxah3 and REQ-kn5rtx
(after the third independent review found the terminal-error detection had never
fired), and fix1's three, which are the ones named here — added after the
verification gate found REQ-tw4cb5 verified on a single fixture rather than as a
property:
`self_delete_payload_carries_no_epoch_key_at_any_depth`,
`self_delete_carries_no_epoch_for_any_organisation_id` and
`the_epoch_probe_finds_an_epoch_on_updated_and_none_on_self_delete`, all three
`verifies: REQ-tw4cb5`. This table is the plan as written and is left that way;
the note is here because it is the only index of REQ-tw4cb5's evidence anywhere
in the plan, and a reader using it would otherwise find two tests where five
exist.

| test | verifies |
|---|---|
| `updated_emits_membership_incoming_and_epoch` | REQ-2k7ys4 |
| `updated_payload_carries_the_measured_epoch_and_root` | REQ-2k7ys4 |
| `record_unreadable_emits_no_membership_event` | REQ-2k7ys4 |
| `record_unreadable_emits_no_epoch_event` | REQ-2k7ys4 |
| `record_unreadable_names_the_organisation` | REQ-dp95pv |
| `record_unreadable_emits_exactly_one_event` | REQ-dp95pv |
| `self_delete_emits_no_epoch_event` | REQ-tw4cb5 |
| `self_delete_emits_revoked_naming_the_organisation` | REQ-tw4cb5 |
| `verify_failure_carries_the_organisation_when_known` | REQ-affyf5 |
| `verify_failure_carries_null_organisation_when_unknown` | REQ-affyf5 |
| `verify_failure_carries_the_message` | REQ-affyf5 |
| `verify_failure_emits_no_membership_event` | REQ-affyf5 |
| `stopped_names_the_reason` | REQ-jfxah3 |
| `stopped_emits_exactly_one_event` | REQ-jfxah3 |

The negative assertions are the substance of this file and must be written
against the event *names*, not against the payloads — "no membership event" has
to mean no emission named `membership-updated`, `incoming-verified` or
`epoch-changed`:

```rust
// verifies: REQ-2k7ys4
#[test]
fn record_unreadable_emits_no_membership_event() {
    let out = events::emissions_for(&events::ReceiverOutcome::RecordUnreadable {
        org_id: "aa".repeat(20),
    });
    let names: Vec<&str> = out.iter().map(|e| e.name).collect();
    for forbidden in ["membership-updated", "incoming-verified", "epoch-changed"] {
        assert!(!names.contains(&forbidden), "emitted {forbidden}: {names:?}");
    }
}
```

`tests/receiver_guard.rs` — **6 tests.**

| test | verifies |
|---|---|
| `first_claim_succeeds` | REQ-6hgm8r |
| `second_claim_while_held_fails` | REQ-6hgm8r |
| `claim_succeeds_again_after_the_guard_drops` | REQ-3hfggn |
| `guard_releases_when_dropped_by_panic` | REQ-3hfggn |
| `concurrent_claims_yield_exactly_one_guard` | REQ-6hgm8r |
| `release_then_concurrent_claims_yield_exactly_one_guard` | REQ-3hfggn |

`guard_releases_when_dropped_by_panic` is the one that justifies `Drop` over an
explicit release call, and is why the loop cannot leave the slot claimed:

```rust
// verifies: REQ-3hfggn
#[test]
fn guard_releases_when_dropped_by_panic() {
    let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let f = std::sync::Arc::clone(&flag);
    let _ = std::panic::catch_unwind(move || {
        let _g = events::StartGuard::try_claim(&f).expect("first claim");
        panic!("the receiver loop panicked");
    });
    assert!(
        events::StartGuard::try_claim(&flag).is_some(),
        "a panicking loop must not keep the slot claimed for the life of the process"
    );
}
```

`concurrent_claims_yield_exactly_one_guard` spawns 16 threads on one flag and
asserts exactly one `Some`. It is the abnormal-input case for REQ-6hgm8r and it
is what stops T2's rewiring from reintroducing double-spawning (§5 of the
register).

### Step 1.7 — expected output

```
$ CARGO_HOME=/tmp/cargo_home_fuzz cargo test --manifest-path app/src-tauri/Cargo.toml \
    --features test-support --test startup_policy --test connection_status \
    --test org_id_parsing --test receiver_events --test receiver_guard
...
test result: ok. 12 passed; 0 failed  (startup_policy)
test result: ok. 8 passed; 0 failed   (connection_status)
test result: ok. 11 passed; 0 failed  (org_id_parsing)
test result: ok. 14 passed; 0 failed  (receiver_events)
test result: ok. 6 passed; 0 failed   (receiver_guard)
```

**51 tests.**

---

## T2 — Rust: rewire the callers, and drive the commands over the real IPC boundary

**Files touched:**
`app/src-tauri/src/state.rs`,
`app/src-tauri/src/commands.rs`,
`app/src-tauri/tests/ipc.rs`

**Parallel:** no — serial, after T1. (May run alongside T4.)

**Implements:** REQ-645jq9 and REQ-sjkp8z across the IPC boundary; rewires the
callers of everything T1 gated.

### Step 2.1 — write `tests/ipc.rs` FIRST

This is the task's red. The suite builds a real `tauri::test::mock_builder`
application with the **same `generate_handler!` list as `lib.rs`**, manages a
test-constructed `AppState`, and invokes commands across the actual IPC
boundary using the actual argument names.

Watch it fail before writing step 2.2: `connection_status` does not yet report
a transport mode, and `AppState` has no test constructor, so the file does not
compile. That is the right red — the feature is missing.

**8 tests**, chosen for what only this suite can catch:

| test | verifies | what it catches that an extracted-logic test cannot |
|---|---|---|
| `connection_status_reports_transport_mode_over_ipc` | REQ-645jq9 | the DTO field actually crosses the boundary and is named `transport_mode` on the wire |
| `connection_status_is_registered_under_its_name` | REQ-645jq9 | a command renamed in `lib.rs` but not in `api.ts` |
| `export_invite_rejects_a_short_org_id` | REQ-sjkp8z | the boundary check is reached by the real handler, with the real `orgId` argument name |
| `export_invite_rejects_a_non_hex_org_id` | REQ-sjkp8z | as above, abnormal alphabet |
| `revoke_member_rejects_a_short_member_id` | REQ-he8ejb | the Rust-side width check behind the frontend's |
| `revoke_member_accepts_an_empty_peer_addr` | REQ-vgr7s2 | **the backend half of HAZ-n97v5g** — that an empty blob is genuinely accepted, which is what makes the frontend fix correct |
| `unknown_command_is_rejected` | — (robustness) | the handler list is closed |
| `list_personas_returns_an_empty_list_on_a_fresh_store` | — (robustness) | the state wiring works at all |

`revoke_member_accepts_an_empty_peer_addr` is the most valuable test in the
change and deserves care: it must assert that the empty blob gets **past the
peer-address decode** and fails (if it fails) for a later reason — not that the
call returns `Ok`, which it will not, there being no such member. Assert on the
error *not* mentioning `peer_addr`:

```rust
// verifies: REQ-vgr7s2
#[test]
fn revoke_member_accepts_an_empty_peer_addr() {
    let (app, webview) = harness();
    let err = invoke_err(&webview, "revoke_member", serde_json::json!({
        "orgId": "aa".repeat(20),
        "memberIdHex": "bb".repeat(32),
        "peerAddrBlob": ""
    }));
    // The revocation will not succeed — there is no such org and no such
    // member. What HAZ-n97v5g needs is that it is not rejected FOR THE ADDRESS.
    assert!(
        !err.to_lowercase().contains("peer_addr"),
        "an empty peer address must be accepted in Networked transport, got: {err}"
    );
    let _ = app;
}
```

### Step 2.2 — `commands.rs`

1. Delete the private `parse_org_id` and import `crate::parsing::parse_org_id`.
2. Make `connection_status` and `start_receiver` **generic over the runtime**,
   which is what lets the mock runtime drive them:

```rust
#[tauri::command]
pub async fn connection_status<R: tauri::Runtime>(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle<R>,
) -> Result<ConnectionStatus, String> {
```

   `tauri::AppHandle` defaults to `AppHandle<Wry>`, so as written today these
   two commands cannot be invoked under `MockRuntime` at all. `generate_handler!`
   supports generic commands and `lib.rs` needs no change.

3. `connection_status` no longer resolves a data directory of its own. It reads
   the one `AppState` recorded at startup, and passes the recorded
   `ChainEndpoint` and transport mode:

```rust
    Ok(policy::connection_status_from_state(
        &state.data_dir,
        state.chain_endpoint.as_ref(),
        state.transport_mode,
    ))
```

   This also removes a latent inconsistency: the old handler re-derived the data
   directory from `ODS_DATA_DIR`/`app_data_dir()`, duplicating `init`'s logic,
   so the two could disagree.

4. `start_receiver` claims a `StartGuard`, moves it into the spawned task so it
   drops when the loop ends, builds a `ReceiverOutcome` per iteration, and emits
   via `emissions_for`:

```rust
    let Some(guard) = events::StartGuard::try_claim(&state.receiver_started) else {
        return Ok(()); // REQ-6hgm8r: a loop is already running.
    };

    let app_handle_clone = app_handle.clone();
    tokio::spawn(async move {
        // REQ-3hfggn: the guard is moved in, so the slot is released when this
        // task ends — by break, by return, or by panic.
        let _guard = guard;
        loop {
            let outcome = next_outcome(&app_handle_clone).await;
            let stopping = matches!(outcome, events::ReceiverOutcome::Stopped { .. });
            for e in events::emissions_for(&outcome) {
                let _ = app_handle_clone.emit(e.name, e.payload);
            }
            if stopping {
                break;
            }
        }
    });
```

   `next_outcome` is a private helper holding the logic that currently sits
   inline in the loop, with three changes, each carrying its requirement:

   - the `list_orgs` read-back returns `RecordUnreadable` instead of
     `.unwrap_or((0, String::new()))` (REQ-2k7ys4, REQ-dp95pv);
   - the self-delete arm returns `SelfDeleted` alone (REQ-tw4cb5);
   - the `Err` arm returns `VerifyFailed`, and — when the message matches the
     terminal substring — is followed by `Stopped` (REQ-affyf5, REQ-jfxah3).

   **The substring match stays.** It is claim 7 of the register's §3 and the
   subject of REQ-x3c8n2; this change makes its consequence visible and
   recoverable, and does not pretend to fix the detection. Leave the comment
   in the code saying so, naming REQ-x3c8n2.

### Step 2.3 — `state.rs`

1. Re-export the moved types so no third caller changes:
   `pub use crate::policy::{ConnectionStatus, connection_status_from_state};`
2. `AppState` gains three recorded fields and changes one:

```rust
pub struct AppState {
    pub service: Mutex<OrgService>,
    /// The directory the store was actually opened in.
    pub data_dir: PathBuf,
    /// REQ-bvx4nh: the endpoint the running configuration was built from.
    /// `None` is exactly "chain not configured" — the two are one fact, so
    /// they cannot disagree (REQ-e4ah9h).
    pub chain_endpoint: Option<policy::ChainEndpoint>,
    /// REQ-645jq9.
    pub transport_mode: policy::TransportModeName,
    /// Arc so a StartGuard can be moved into the spawned task (REQ-3hfggn).
    pub receiver_started: Arc<AtomicBool>,
}
```

   `chain_ready: bool` is **deleted**, not kept alongside — two fields that must
   agree is the defect REQ-e4ah9h is about.

3. `init` uses the policy functions, reading the environment itself:

```rust
    let allow_dev = std::env::var("ODS_ALLOW_DEV_DEFAULTS").is_ok();
    let data_dir = policy::resolve_data_dir(
        std::env::var("ODS_DATA_DIR").ok().as_deref(),
        tauri_data_dir,
        allow_dev,
    ).map_err(|e| e.to_string())?;
    let passphrase = policy::resolve_passphrase(
        std::env::var("ODS_PASSPHRASE").ok().as_deref(),
        allow_dev,
    ).map_err(|e| e.to_string())?;
```

   Note `tauri_data_dir` becomes `Option<PathBuf>`: `lib.rs` currently swallows
   the resolver error with `unwrap_or_else(|_| "/tmp/ods-poc")`, and that
   fallback moves into `resolve_data_dir` where it is conditional. **T2 must
   therefore also change the one line in `lib.rs`** that builds it — from
   `.unwrap_or_else(|_| PathBuf::from("/tmp/ods-poc"))` to `.ok()`. This is the
   one line of `lib.rs` T2 touches; T1 owns the rest of that file and the two
   edits are in different places, so the merge is clean. *(If it conflicts,
   T2 wins — T1's change to `lib.rs` is additive module declarations only.)*

4. `build_chain_ops` returns the endpoint alongside the ops, so it can be
   recorded:
   `fn build_chain_ops() -> Result<(Box<dyn ChainOps>, policy::ChainEndpoint), String>`

5. Add the test-only constructor:

```rust
    /// Build a state directly, for tests. Never reads the environment.
    #[cfg(feature = "test-support")]
    pub fn for_test(
        data_dir: PathBuf,
        passphrase: &str,
        chain_endpoint: Option<policy::ChainEndpoint>,
        transport_mode: policy::TransportModeName,
    ) -> Result<Self, String> { /* opens a real PersonaStore under data_dir */ }
```

### Step 2.4 — red → green

The commands' rewiring is behaviour that pre-dates its test in three places
(the loop's structure, the guard's placement, the status projection). Where a
test cannot be written red first, **use red by mutation**: change the source,
watch the named test fail for that reason, revert with `git checkout --`, and
name the mutation in the dispatch report. A mutation that reds nothing is a
**measured negative** and must be reported as such — not silently dropped.

Prescribed mutations:

| mutate | must redden |
|---|---|
| `connection_status` passes `TransportModeName::Networked` literally | `connection_status_reports_transport_mode_over_ipc` (run it with a Loopback state) |
| `start_receiver` drops the `let _guard = guard;` line | nothing in this suite — **expected measured negative**, because no test drives the real loop. Report it as one. |
| `revoke_member` restores an unconditional non-empty check on `peer_addr_blob` | `revoke_member_accepts_an_empty_peer_addr` |
| `parse_org_id` import removed, length check changed to `>= 40` | `export_invite_rejects_a_short_org_id` (as `!=` → `>=` admits 41) |

The second row is the important one and it must be reported honestly: the
guard's release is gated by `receiver_guard.rs` at the unit level (T1) and by
nothing at the integration level, because nothing in this repository can run the
receiver loop. Say so.

### Step 2.5 — expected output

```
test result: ok. 8 passed; 0 failed   (ipc)
```

plus T1's 51, unchanged.

---

## T3 — Frontend: the extracted decisions, and a test runner

**Files touched:**
`app/package.json`,
`app/vitest.config.ts` (new),
`app/src/lib/api.ts`,
`app/src/lib/revoke.ts` (new),
`app/src/lib/verify.ts` (new),
`app/src/lib/receiver.ts` (new),
`app/tests/revoke.validate.test.ts` (new),
`app/tests/verify.row.test.ts` (new),
`app/tests/receiver.subscribe.test.ts` (new)

**Parallel:** yes (with T1)

**Implements:** REQ-vgr7s2, REQ-he8ejb, REQ-a83vqr, REQ-akt4p7, REQ-rq8g2v,
PR-u34uqm

### Step 3.1 — `package.json` and `vitest.config.ts`

`vitest` is already in `devDependencies` (installed 2026-09-14 while
establishing that this task is buildable at all). Add the script:

```json
    "test": "vitest run"
```

Also add, to converge with the uncommitted edit on the base branch and to keep
`npm ci` quiet in CI:

```json
  "allowScripts": {
    "fsevents@2.3.3": true
  }
```

`app/vitest.config.ts` — deliberately **without** the `sveltekit()` plugin:

```ts
import { defineConfig } from 'vitest/config';

// No sveltekit() plugin and no $lib alias, by choice: these tests import the
// modules under test by relative path, so the runner needs no SvelteKit
// machinery and cannot fail for a reason belonging to the framework rather
// than to the code. When components are rendered here one day (not-minted
// control 1) this file gains the plugin and a DOM environment.
export default defineConfig({
	test: {
		environment: 'node',
		include: ['tests/**/*.test.ts']
	}
});
```

**Note for the record — CORRECTED 2026-09-14 by T3's measurement.** This step
originally predicted that `npm run check` does *not* type-check `app/tests`,
on the grounds that the generated `.svelte-kit/tsconfig.json` covers `../src/**`
only, and instructed T3 to record vitest as their only gate. **That prediction
was wrong.** The generated file also includes `../test/**/*.{js,ts,svelte}` and
`../tests/**/*.{js,ts,svelte}`, so these files *are* type-checked.

T3 measured it rather than re-reading the config: it appended
`const PROBE: number = "definitely not a number";` to
`app/tests/verify.row.test.ts`, ran `npm --prefix app run check`, and got

    ERROR "tests/verify.row.test.ts" 91:7 "Type 'string' is not assignable to type 'number'."
    COMPLETED 193 FILES 1 ERRORS 1 WARNINGS

then removed the probe and re-ran to 0 errors. The file count rising from 166
to 193 on adding the tests corroborates it.

Two consequences. The gap goes into the verification record as **closed by
measurement**, not as an open gap — the frontend tests are checked under
`strict`, `verbatimModuleSyntax` and `isolatedModules` as well as run. And T4
inherits a constraint the plan did not state: a type error anywhere in
`app/tests` now fails `npm run check`, so T4 cannot change a type in `api.ts`
without the test files agreeing.

### Step 3.2 — `api.ts`

Type-level changes only, mirroring T1/T2's DTOs:

```ts
export type TransportMode = 'networked' | 'loopback';

export interface ConnectionStatus {
	chain_configured: boolean;
	chain_ws: string | null;
	contract_h160: string | null;
	transport_mode: TransportMode;   // REQ-645jq9
	data_dir: string;
}

export interface VerificationFailedPayload {
	org_id: string | null;           // REQ-affyf5: null when unknown
	message: string;
}

export interface RecordUnreadablePayload { org_id: string }
export interface ReceiverStoppedPayload { reason: string }
```

and three new listener wrappers, `onVerificationFailed`, `onRecordUnreadable`,
`onReceiverStopped`, in the shape of the five that exist. `revokeMember`'s
signature keeps its three parameters; the empty string continues to mean
"no address", which is what T2's IPC test pins.

`EpochChangedPayload` and the `onEpochChanged` wrapper stay, but the payload is
no longer emitted on self-delete (REQ-tw4cb5) — note it in the doc-comment.

### Step 3.3 — `revoke.ts`

```ts
import type { TransportMode } from './api';

export interface RevokeInput {
	transportMode: TransportMode;
	memberIdHex: string;
	peerAddrBlob: string;
}

export type RevokeCheck = { ok: true } | { ok: false; message: string };

/**
 * REQ-vgr7s2, REQ-he8ejb. Whether a revocation may be submitted.
 *
 * The peer address is required ONLY in Loopback. In Networked the joiner is
 * reached by EndpointId via discovery and the join request carries no address
 * at all — so demanding one made revocation unreachable in the default
 * transport (HAZ-n97v5g).
 */
export function validateRevokeInput(input: RevokeInput): RevokeCheck {
	const memberId = input.memberIdHex.trim().replace(/^0x/, '');
	if (memberId.length !== 64) {
		return { ok: false, message: `Member ID must be 64 hex characters, got ${memberId.length}.` };
	}
	if (!/^[0-9a-fA-F]+$/.test(memberId)) {
		return { ok: false, message: 'Member ID must be hexadecimal.' };
	}
	if (input.transportMode === 'loopback' && input.peerAddrBlob.trim() === '') {
		return {
			ok: false,
			message: 'Peer address is required in Loopback transport (same-machine dialling).'
		};
	}
	return { ok: true };
}
```

`tests/revoke.validate.test.ts` — **10 tests**, all `verifies: REQ-vgr7s2` or
`verifies: REQ-he8ejb`. The one that matters most:

```ts
// verifies: REQ-vgr7s2
it('accepts an empty peer address in networked transport', () => {
	// This is HAZ-n97v5g. Before this change the form refused unconditionally,
	// and a Networked join request never carries an address to supply — so
	// revocation could not be submitted at all in the shipped configuration.
	expect(
		validateRevokeInput({ transportMode: 'networked', memberIdHex: 'bb'.repeat(32), peerAddrBlob: '' })
	).toEqual({ ok: true });
});
```

plus: accepts an address in networked; **rejects** an empty address in
loopback; accepts an address in loopback; rejects a 63-character member id;
rejects a 65-character one; rejects an empty one; rejects a non-hex one;
accepts an `0x`-prefixed one; and trims surrounding whitespace.

### Step 3.4 — `verify.ts`

```ts
export type VerifyEvent =
	| { kind: 'verified'; org_id: string; epoch: number; root: string }
	| { kind: 'failed'; org_id: string | null; message: string };

export interface VerifyRow {
	org_id: string;
	epoch: number | null;
	root: string | null;
	verified: boolean;
	detail: string | null;
	ts: string;
}

/**
 * REQ-a83vqr, REQ-akt4p7. The row's outcome comes from the event that produced
 * it and from nowhere else.
 *
 * Before this change the component assigned `verified: true` at both of its
 * assignment sites, so the ✗ state was unreachable by any input (HAZ-9fmhm4).
 */
export function verifyResultFrom(event: VerifyEvent, ts: string): VerifyRow {
	if (event.kind === 'verified') {
		return { org_id: event.org_id, epoch: event.epoch, root: event.root,
		         verified: true, detail: null, ts };
	}
	return { org_id: event.org_id ?? '(unknown organisation)', epoch: null, root: null,
	         verified: false, detail: event.message, ts };
}
```

`epoch` and `root` are `null` on a failure rather than `0` and `''` — the same
rule as REQ-tw4cb5 on the Rust side, applied on this one. A failure knows
neither.

`tests/verify.row.test.ts` — **8 tests**: a verified event yields
`verified: true`; a failed event yields `verified: false` (**the test that
would have caught HAZ-9fmhm4**); a failed event carries no epoch and no root; a
failed event with a null org renders a placeholder rather than "null"; a
failed event carries its message as detail; a verified event carries no detail;
epoch 0 on a verified event is preserved as 0 and not coerced to null; and the
timestamp is passed through unmodified.

The epoch-0 test is the abnormal-input case that ties this file to HAZ-5ha5vv:
0 is genesis, it is a real value, and a row must be able to show it while a
failure shows nothing.

### Step 3.5 — `receiver.ts`

```ts
export type Unlisten = () => void;
export type Subscribe = (cb: (payload: unknown) => void) => Promise<Unlisten>;

/**
 * REQ-rq8g2v. Registers every subscription, then resolves to one cleanup that
 * cancels exactly those registered — including any whose `listen` call resolved
 * after teardown began.
 *
 * PR-u34uqm: the component previously returned its cleanup synchronously while
 * the `listen` calls it was meant to cancel were still pending, so the cleanup
 * emptied an array the pending subscriptions then refilled, leaving listeners
 * attached to a destroyed component and reachable by nothing.
 */
export async function subscribeAll(
	subscriptions: Array<[Subscribe, (payload: never) => void]>
): Promise<Unlisten> {
	const unlisteners = await Promise.all(
		subscriptions.map(([subscribe, handler]) => subscribe(handler as (p: unknown) => void))
	);
	let done = false;
	return () => {
		if (done) return;
		done = true;
		for (const u of unlisteners) u();
	};
}
```

`tests/receiver.subscribe.test.ts` — **6 tests**:

| test | verifies |
|---|---|
| `registers every subscription it is given` | REQ-rq8g2v |
| `unsubscribes exactly the listeners it registered, including those whose listen call resolved late` | REQ-rq8g2v |
| `resolves only after every subscription has registered` | REQ-rq8g2v |
| `is idempotent — calling the cleanup twice unsubscribes once` | REQ-rq8g2v |
| `propagates a subscription failure rather than resolving a partial cleanup` | REQ-rq8g2v |
| `registers nothing and cleans up cleanly when given no subscriptions` | REQ-rq8g2v |

The second is PR-u34uqm's reproducing test and must construct the late
resolution explicitly:

```ts
// verifies: REQ-rq8g2v
it('unsubscribes exactly the listeners it registered, including those whose listen call resolved late', async () => {
	const cancelled: string[] = [];
	const fast: Subscribe = async () => () => cancelled.push('fast');
	const slow: Subscribe = async () => {
		await new Promise((r) => setTimeout(r, 20));
		return () => cancelled.push('slow');
	};
	const cleanup = await subscribeAll([[fast, () => {}], [slow, () => {}]]);
	cleanup();
	// Against the old shape the slow listener was registered into an array the
	// cleanup had already emptied, so it was never cancelled.
	expect(cancelled.sort()).toEqual(['fast', 'slow']);
});
```

### Step 3.6 — expected output

```
$ npm --prefix app run test
 Test Files  3 passed (3)
      Tests  24 passed (24)
```

---

## T4 — Frontend: rewire the components

**Files touched:**
`app/src/lib/components/Revoke.svelte`,
`app/src/lib/components/Membership.svelte`,
`app/src/lib/components/StatusBar.svelte`

**Parallel:** no — serial, after T3. (May run alongside T2.)

**Implements:** the component half of REQ-vgr7s2, REQ-he8ejb, REQ-a83vqr,
REQ-akt4p7, REQ-rq8g2v.

No new tests. **Nothing in this change renders a component**, so every claim in
this task is held by review, and the task's red → green is by mutation only.
That is the register's not-minted control 1 and the dispatch report must state
it plainly rather than implying the components are gated.

1. **`Revoke.svelte`** — delete the three hand-rolled precondition checks and
   call `validateRevokeInput`, with the transport mode read from
   `connectionStatus()`. The form's peer-address field gains a label that says
   when it is needed ("required in Loopback transport only") rather than
   marking it required outright.
2. **`Membership.svelte`** — build rows with `verifyResultFrom`; render the
   failure row (the `✗ MISMATCH` branch now reachable, with the detail column
   showing the message); subscribe through `subscribeAll` and store the awaited
   cleanup; add the three new listeners; on `receiver-stopped`, clear the
   "Receiver running" badge and show the reason.
3. **`StatusBar.svelte`** — display the transport mode, and show the endpoint
   and contract only when `chain_configured` (they are `null` otherwise after
   T2, so this is a rendering guard rather than a second decision).

Prescribed mutations for the report:

| mutate | must redden |
|---|---|
| `verifyResultFrom`'s failure branch returns `verified: true` | `verify.row.test.ts` — a failed event yields verified false |
| `validateRevokeInput` drops the `loopback` condition | `revoke.validate.test.ts` — rejects an empty address in loopback |
| `subscribeAll` returns its cleanup before awaiting | `receiver.subscribe.test.ts` — the late-resolution test |

Each reddens a T3 test, not a T4 one — which is exactly the gap: the extracted
functions are gated, and that the components call them is not.

---

## T5 — Config, and CI

**Files touched:**
`app/.guardrails/config.yaml`,
`app/package.json`,
`app/package-lock.json`,
`.github/workflows/rust.yml`

### Step 5.0a — make the `test` script self-contained

Added 2026-09-14 from T4's first finding. `npm run test` fails outright in a
fresh checkout — vitest reports `[TSCONFIG_ERROR] … Tsconfig not found`, because
`app/tsconfig.json` extends the generated, gitignored
`./.svelte-kit/tsconfig.json`. `npm run check` generates it as its first step
(`svelte-kit sync`); `npm run test` does not, so `test` only works if `check`
has been run since the last clean checkout.

Both the `verify_commands` list and the CI job below happen to run `check`
first, so both work — by luck, not by design, and a reordering or running
`npm test` alone would break it. Make the script carry its own precondition, in
`app/package.json`:

    "test": "svelte-kit sync && vitest run"

`svelte-kit sync` is idempotent and costs well under a second when the output is
already current, so running it twice in a `verify_commands` pass is not worth
avoiding.

**Parallel:** no — last.

### Step 5.0 — regenerate the lockfile, BEFORE the CI job that depends on it

Added 2026-09-14 from T3's third finding, which is a genuine blocker rather
than a note: **`app/package-lock.json` records no `vitest` entry.** The tracked
lockfile predates the dependency — vitest reached `app/node_modules` through an
`npm install -D` in the change worktree whose `package.json` edit was
uncommitted at the time, and T3 added the manifest entry from its own worktree
without touching the lock.

Step 5.2's CI job runs `npm --prefix app ci`, which installs **from the lock and
ignores the manifest**. Against the current lock it would install no vitest and
then fail at `npm run test` — a red CI job introduced by the change that adds
the job. So, in the change worktree, before editing the workflow:

    npm --prefix app install

and commit `app/package-lock.json`. Then confirm the lock and the manifest
agree, which is the thing actually being asserted:

    npm --prefix app ci && npm --prefix app run test

`npm ci` deletes and reinstalls `node_modules` from the lock alone, so a green
`npm run test` after it is proof that a clean CI checkout gets a working vitest.
Expect 24 passing tests. Note that this deletes the 92 MB tree copied in at
setup and rebuilds it — that is fine and is the point of running it.

### Step 5.1 — `app/.guardrails/config.yaml`

```yaml
strict_paths:
  - app/src-tauri/src
  - app/src

test_paths:
  - app/src-tauri/tests
  - app/tests

verify_commands:
  - cargo test --manifest-path app/src-tauri/Cargo.toml --features test-support --test startup_policy --test connection_status --test org_id_parsing --test receiver_events --test receiver_guard --test ipc
  - npm --prefix app run check
  - npm --prefix app run test
```

Replace the `strict_paths` comment, which currently says the frontend "is not
under trace discipline yet — widen here when its first item is written". It is
now, and the reason for widening is worth recording: four of this unit's
requirements are realised in the frontend and nowhere else.

Replace the `test_paths` comment, which says the crate has no tests and the
directory holds a `.gitkeep`. **Delete the `.gitkeep`** — the directory now
holds six test files.

Leave the `coverage_command` block commented out and its note intact. It is
still true and it is still tooth 5.

### Step 5.2 — `.github/workflows/rust.yml`

Add a job — the frontend half of app's `verify_commands` needs no Tauri system
packages, so unlike the cargo half it can run in CI today:

```yaml
  # app's npm verify_commands. The cargo entry still runs NOWHERE in CI: it
  # needs Tauri's Linux system packages (webkit2gtk, libsoup, …) this workflow
  # does not install, so for that entry the merge gate remains the only gate.
  # Recorded in docs/plans/2026-09-05-ratchet-setup.md.
  app-frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: "20" }
      - run: npm --prefix app ci
      - run: npm --prefix app run check
      - run: npm --prefix app run test
```

Update the workflow's header comment, which currently states that both of app's
`verify_commands` run nowhere in CI. After this change that is true of one of
the three, and the comment must say which.

---

## Self-review

1. **Every Implements ID has a task whose test verifies it.** REQ-645jq9 (T1,
   T2), REQ-vgr7s2 (T3, T2), REQ-he8ejb (T3, T2), REQ-affyf5 (T1), REQ-a83vqr
   (T3), REQ-akt4p7 (T3), REQ-rq8g2v (T3), REQ-2k7ys4 (T1), REQ-dp95pv (T1),
   REQ-tw4cb5 (T1), REQ-7g3k9a (T1), REQ-bmk2z2 (T1), REQ-rxc8sp (T1),
   REQ-e4ah9h (T1), REQ-bvx4nh (T1), REQ-3hfggn (T1), REQ-6hgm8r (T1),
   REQ-jfxah3 (T1), REQ-sjkp8z (T1, T2) — nineteen, the requirements this plan
   was written around — plus **REQ-kn5rtx (fix5, fix7) and REQ-wu6z9p (fix5)**,
   minted by the first independent review and gated after the tasks had run.
   Twenty-one in all. Each HAZ and RC is carried transitively by the REQs that
   implement it, **RC-3rddh7 included** — it is realised by those last two.
   PR-u34uqm's reproducing test is in T3.

   *Corrected 2026-09-15 by gate round 9. Round 8 had found the Implements list
   in the header three IDs short and the fix updated the header, its explanatory
   note, and nothing else — so this enumeration one screen lower still agreed
   with the old figure, and RC-3rddh7 was carried transitively by nothing in it.
   The same finding surviving its own fix, which is the pattern these rounds kept
   finding: a correction that reaches the place the finding pointed at and not
   the places that agreed with it.*
2. **Real code, real commands, real expected output** — yes, above.
3. **Names are consistent across tasks:** `TransportModeName` (Rust) serialises
   to `'networked' | 'loopback'` and is typed `TransportMode` in TypeScript;
   the event names `record-unreadable`, `verification-failed` and
   `receiver-stopped` are spelled identically in T1's `emissions_for`, T3's
   `api.ts` and T4's components.
4. **No two parallel tasks name the same file.** T1 ∥ T3: Rust plus
   `Cargo.toml` versus frontend plus `package.json` — disjoint. T2 ∥ T4:
   `state.rs`/`commands.rs`/`ipc.rs` versus three `.svelte` files — disjoint.
   T5 alone. The one shared file across the serial boundary is `lib.rs` (T1
   adds module declarations, T2 changes one line in the `setup` hook), and they
   are serial, so there is no conflict to have.

**Total new tests as planned: 51 (T1) + 8 (T2) + 24 (T3) = 83.** The unit had
none. **As built: 108** — the cargo half rose from 59 to 78 across two
verification-gate findings and three review rounds. The cargo arithmetic in full, because a prior round found this enumeration
two short of its own total: 59 at the end of T5, + 3 (fix1, `receiver_events`,
REQ-tw4cb5 verified on one fixture rather than as a property) + 3 (fix4,
`org_id_parsing`) + 1 (fix4, `ipc`; a doubled `0x` prefix silently normalised)
+ 6 (fix5, `receiver_events`, REQ-kn5rtx) + 2 (fix6, `connection_status`;
REQ-bvx4nh's data-directory clause was gated by no test annotated to it)
+ 4 (fix7, `receiver_events`; the terminal-error detection had never fired)
= **78**. The vitest half rose 24 -> 30: fix5's `receiver.log.test.ts` is a
fourth file, six tests for REQ-wu6z9p.

---

## Red → green attestations

*(filled in as each dispatch report lands — merge-change step 6b copies this
table into the verification record)*

| task | status |
|---|---|
| T1 | **done** — see below |
| T2 | **done** — see below |
| T3 | **done** — see below |
| T4 | **done** — see below |
| T5 | **done** — see below |

### T5 — done 2026-09-14, branch `worktree-guardrails-app-risk-t5` @ `01730bc`

All three `verify_commands`, run verbatim as the config now spells them:

- cargo, six targets: `startup_policy` 12, `connection_status` 8,
  `org_id_parsing` 11, `receiver_events` 14, `receiver_guard` 6, `ipc` 8 =
  **59 passed, 0 failed**, no warnings. (Cargo runs targets alphabetically, so
  the output order is 8/8/11/14/6/12.)
- `npm --prefix app run check`: 193 files, **0 errors**, 1 warning — the
  pre-existing `@types/node` one.
- `npm --prefix app run test`: 3 files, **24 passed**.

`check-trace.sh` exit 0, `check-ids.sh` exit 0, `check-units.sh` 4 units / 404
tracked paths.

No tests added. Red → green by mutations T5 devised, to show the config and CI
edits are load-bearing rather than decorative:

| mutation | reddened |
|---|---|
| the pre-change `test` script (`vitest run`) in a fresh worktree with no generated `.svelte-kit` | all three files, `[TSCONFIG_ERROR] … Tsconfig not found`, `Tests no tests`. Green after `svelte-kit sync && vitest run`. |
| restore the pre-change lockfile, run `npm ci` | `npm error code EUSAGE … Missing: vitest@5.0.0 from lock file`, plus ten further entries |
| drop `--features test-support` from the cargo entry | `error: target 'startup_policy' in package 'ods-poc' requires the features: 'test-support'` — a cargo **error**, not a silent skip |
| drop `app/tests` from `test_paths` | `check-trace.sh` exit 1: `MISSING-TEST REQ-a83vqr`, `REQ-akt4p7`, `REQ-rq8g2v` |
| **drop `app/src` from `strict_paths`** | **nothing — measured negative. See finding 1.** |

Five findings; two correct this plan.

1. **Measured negative on `strict_paths: app/src`.** Removing it reddens
   nothing — `check-trace.sh` exit 0 with identical findings, `check-ids.sh`
   exit 0. The reason is that this unit has **SDD 0 / LLR 0**: `UNTRACED-DESIGN`
   has no design items to convict, so `strict_paths` gates nothing for `app`
   today. The widening is correct and forward-looking — it is what makes the
   frontend a traced path the moment tooth 4 writes this unit's first design
   item — but it is **not load-bearing now**, and only the `test_paths` half of
   step 5.1 is. Recorded in the verification record's gaps alongside T2's
   guard-placement negative.
2. **The lockfile red arrives earlier than step 5.0 predicted.** The step
   expected the stale lock to "install no vitest and then fail at
   `npm run test`". In fact `npm ci` refuses outright with `EUSAGE` when lock
   and manifest disagree — it never installs anything. The conclusion is
   unchanged (the new CI job would have been red on arrival) but the failure
   mode is a first-step error, not a test failure.
3. **Three requirements are frontend-only, not four.** Step 5.1 told T5 to
   record that "four of this unit's requirements are realised in the frontend
   and nowhere else". The `test_paths` mutation *measured* which: REQ-a83vqr,
   REQ-akt4p7, REQ-rq8g2v. REQ-vgr7s2 and REQ-he8ejb are realised in the
   frontend **and** verified on the Rust side by T2's `ipc.rs`, so they survive
   the mutation. The config comment states the measurement rather than the
   plan's count — which is the right way round, and the count here was a guess
   written before `ipc.rs` existed.
4. `npm install` moved 21 packages beyond vitest's own tree (`obug`
   2.1.3→2.2.1, `picomatch` 4.0.4→4.0.7, and the transitive set): 275
   insertions / 11 deletions, confined to `app/package-lock.json`. No
   `Cargo.lock` or other stray file appeared.
5. **Two stale comments elsewhere in `app/.guardrails/config.yaml`**, which T5
   flagged and correctly left alone because step 5.1 did not name them: the
   `coverage_command` note still opened "nothing to measure until the crate has
   tests", and the `depends_on` note still said "the app has no requirements
   yet, so it records no expects: items". Both were false as of this change.
   Fixed by the dispatcher on the change branch after the merge, each rewritten
   to say what is now true and what part of the old note still holds.

### T4 — done 2026-09-14, branch `worktree-guardrails-app-risk-t4` @ `a431f58`

No tests added, by design. 24 frontend tests still passing, `npm run check` at
193 files, 0 errors, 1 warning.

Red → green **by mutation only**, and all three prescribed mutations reddened a
named test — **no measured negatives**:

| mutation | reddened |
|---|---|
| `verifyResultFrom`'s failure branch returns `verified: true` | `renders a failed event as not verified` — `expected true to be false`, verify.row.test.ts:39. 1 failed, 23 passed. |
| `validateRevokeInput` drops the `loopback` condition | `rejects an empty peer address in loopback transport` — revoke.validate.test.ts:49, **and** `trims surrounding whitespace before measuring` (:137). 2 failed, 22 passed. |
| `subscribeAll` returns its cleanup before awaiting | `unsubscribes exactly the listeners it registered, including those whose listen call resolved late` — `expected [ 'fast' ] to deeply equal [ 'fast', 'slow' ]`, receiver.subscribe.test.ts:46, **and** two others in the same file. 3 failed, 21 passed. |

Two reddened more tests than the table predicted. The plan's named test reddened
in every case, so the prediction was right about what it asserted and
conservative about the blast radius.

**This is still the weakest evidence in the change, and the mutations do not
change that.** Each reddens a test in T3's files — it demonstrates that the
*extracted functions* discriminate, not that the components call them. Nothing
renders a component. That is not-minted control 1 and it is what the residuals
of HAZ-n97v5g and HAZ-5ha5vv rest on.

Six findings; two of them correct this plan.

1. **`npm run test` cannot run before `npm run check` in a fresh worktree.**
   vitest failed all three files with `[TSCONFIG_ERROR] … Tsconfig not found`,
   because `app/tsconfig.json` extends the generated, gitignored
   `./.svelte-kit/tsconfig.json`. `npm run check` starts with `svelte-kit
   sync`, which generates it; `npm run test` does not. T5's job order happens
   to work. **Do not leave it depending on that** — step 5.2 now makes the
   `test` script self-contained, which also fixes the `verify_commands`
   ordering and anyone running `npm test` by hand.
2. **The plan miscounted, and T4 was right not to follow it.** Step T4.1 says
   "delete the three hand-rolled precondition checks". `Revoke.svelte` has three
   `if` guards, but only two are what `validateRevokeInput` replaces; the third
   is `if (!selectedOrgId)`, which guards the form's own selection and is
   covered by neither REQ-vgr7s2 nor REQ-he8ejb. Deleting it would submit an
   empty `orgId` to the backend. T4 kept it and replaced the other two. The
   "three" in the plan was a miscount of `validateRevokeInput`'s three
   *decisions* (width, alphabet, loopback address) as the component's three
   guards.
3. **`Revoke.svelte` had PR-u34uqm too**, and T4 routed its single `onRevoked`
   subscription through `subscribeAll` as well — a deviation from the letter of
   the plan, which names `subscribeAll` only under Membership. Accepted: same
   defect, a file T4 owns, and REQ-rq8g2v is a T4 deliverable. PR-u34uqm has
   been amended in its defining file to name both components, because a defect
   found twice in two independently written components is a property of the
   shape rather than a slip.
4. **A Detail column is new markup, not a rewire.** The plan asked for "the
   detail column showing the message", and Membership's table had no such
   column. T4 added one, and renders a failed row's null epoch and root as an
   em dash rather than `0` and `""` — which is HAZ-5ha5vv's rule applied at the
   point of display.
5. `Revoke.svelte` initialises `transportMode` to `'networked'` before
   `connection_status` resolves, matching the backend's own default. Defaulting
   to `'loopback'` would demand an address the operator cannot supply during
   that window — HAZ-n97v5g with a shorter fuse. The reasoning is in a component
   comment.

### T1 — done 2026-09-14, branch `worktree-guardrails-app-risk-t1` @ `f797b4e`

51 tests: `startup_policy` 12, `connection_status` 8, `org_id_parsing` 11,
`receiver_events` 14, `receiver_guard` 6 — exactly the counts step 1.7
predicts. `cargo check` with `test-support` **off** is also clean, so the
shipped build is unaffected by the feature. `--test ipc` compiles and reports 0
tests, as intended for T2.

Red → green, as reported: **five reds, one per target.** Each target was
written before the module it exercises existed and was watched failing to
compile on the named missing item — `E0432 could not find 'policy' in
'ods_poc_lib'` (startup_policy); `E0432`/`E0425` for `ChainEndpoint`,
`TransportModeName`, `connection_status_from_state` and `transport_mode_from`,
15 errors (connection_status); `E0432 could not find 'parsing'`
(org_id_parsing); `E0432 could not find 'events'` (receiver_events); `E0432 no
'StartGuard' in 'events'` (receiver_guard). Every test in a target was red at
that run and green only after the module was written.

Like T3's, this is a per-target red rather than a per-test one, and it carries
the same caveat and the same remedy: it proves the tests could not have passed
before the code existed, and T2's prescribed mutations are what show the
individual assertions discriminate.

Five findings:

1. **A declared `[[test]]` target with no file is a cargo manifest parse
   error — for all six entries, not just `ipc`.** Step 1.1 anticipated this for
   the placeholder only. In fact the first red could not run at all until every
   one of the five test files existed, so they were created empty and filled one
   per cycle. Any future task adding a `[[test]]` entry must create the file in
   the same commit.
2. **`cp -Rc` the gitignored `app/src-tauri/target` into a task worktree.** A
   cold build of tauri + subxt + iroh would have cost several 600-second calls;
   an APFS clonefile copy took 2 seconds and zero extra disk. Carried into T2's
   dispatch.
3. No test sets an environment variable and none needed to — the design
   intent of step 1.3 held. The one test that names `ODS_CHAIN_WS`
   (`unconfigured_chain_reports_false_even_with_env_set`) only reads it to
   quote in a failure message.
4. **REQ-bmk2z2 and REQ-dp95pv have both their tests on one path**, because
   that is the only path on which either requirement has behaviour: a refusal
   that names its variables exists only when there is a refusal, and a
   record-unreadable event exists only when the record is unreadable. Their
   "normal case" — a value was supplied, so no refusal is produced; the record
   was readable, so no such event is emitted — is held by the accept tests for
   REQ-7g3k9a and REQ-rxc8sp, and by
   `record_unreadable_emits_no_membership_event`'s complement
   `updated_emits_membership_incoming_and_epoch`. **Accepted as satisfying the
   class C robustness rule**, and recorded here rather than left for the
   reviewer to find: the rule asks that abnormal input be exercised, not that
   every requirement have two tests in two named categories, and a requirement
   whose entire subject is the abnormal path cannot have a normal-case test of
   its own that means anything.

   ~~Every other ID in T1's Implements list has a distinct normal-case and
   abnormal-input test.~~ **That closing sentence was wrong, and the
   verification gate caught it — see fix1 below.** REQ-tw4cb5's two tests drove
   the identical input. The argument above is sound for REQ-bmk2z2 and
   REQ-dp95pv, and the gate independently agreed with it after checking that
   the named complements exist and pass. What was not sound was extending a
   clean bill to every other ID on the strength of having reasoned carefully
   about two of them. A completeness claim has to be checked item by item; this
   one was asserted.
5. **`commands.rs` still holds its own private `parse_org_id`.** `parsing.rs`
   is a copy, not a move, until T2 deletes the original — the T1/T2 split
   working as designed, but live duplication in the interim, and T2 must not
   leave it that way.

### T2 — done 2026-09-14, branch `worktree-guardrails-app-risk-t2` @ `e71fb23`

**59 tests across all six targets**: `startup_policy` 12, `connection_status`
8, `org_id_parsing` 11, `receiver_events` 14, `receiver_guard` 6, `ipc` 8. T1's
five are unchanged. `cargo check` with `test-support` off is clean with zero
warnings, and so is the test build.

Red: `tests/ipc.rs` was written first and watched failing **to compile**, on
exactly the two features it needed and neither of which existed —
`E0599 no associated function named 'for_test' found for struct 'AppState'`,
and `E0277 the trait bound 'AppHandle: CommandArg<'_, MockRuntime>' is not
satisfied`, twice, from `__cmd__connection_status` and `__cmd__start_receiver`.
That second error is the plan's step 2.2 claim measured: the two commands were
pinned to `Wry` and could not be driven by the mock runtime at all. 3 errors, 0
tests collected.

Green → the four prescribed mutations, three of which reddened a named test:

| mutation | reddened |
|---|---|
| `connection_status` passes `TransportModeName::Networked` literally | `connection_status_reports_transport_mode_over_ipc` — `the transport mode must cross the IPC boundary as transport_mode: {…,"transport_mode":"networked"}` against a Loopback state. 7 passed, 1 failed. |
| **drop `let _guard = guard;` from the spawned task** | **nothing — measured negative, as predicted. See below.** |
| `revoke_member` restores an unconditional non-empty check on `peer_addr_blob` | `revoke_member_accepts_an_empty_peer_addr` — `an empty peer address must be accepted in Networked transport, got: peer_addr_blob is required`. 7 passed, 1 failed. |
| `parse_org_id` import removed, private copy re-inlined with `!=` → `>=` | `export_invite_rejects_a_short_org_id` — `a short org_id must be refused for its WIDTH … got: org_id hex: Odd number of digits`; collaterally `revoke_member_rejects_a_short_member_id`. 6 passed, 2 failed. |

**The measured negative, stated in full because it is evidence about the
change's limits rather than a footnote.** Removing the line that moves the
`StartGuard` into the spawned task — the line that makes REQ-3hfggn true in
production — reddened nothing. All six targets stayed green at 12/8/11/14/6/8.
The only signal the toolchain produced was `warning: unused variable: guard`.

The guard's release is gated by `receiver_guard.rs` at the unit level and by
nothing at the integration level, because **nothing in this repository can run
the receiver loop**: it needs a bound iroh endpoint, an `OrgService` holding a
real organisation, a chain and a peer. So REQ-3hfggn is verified as a property
of `StartGuard` and assumed at its only call site. This is the same shape as
`on-chain-client`'s 2026-09-11 finding, where the prescribed mutation for the
contract-address control also stayed green — and unlike that case, it is **not**
remedied here by further extraction, because the thing unreached is the
`tokio::spawn` itself. It is recorded in the verification record's gaps and is
why HAZ-cfp4jb's residual is P2 rather than lower.

Six findings.

1. The measured negative above.
2. **The dispatch note's ACL claim was half wrong, and it cost a cycle.** This
   plan's author told T2, as an established fact, that `mock_context`'s empty
   ACL would not matter because Tauri 2's capability system gates plugin
   commands rather than app commands. The first half is right and the
   conclusion is not: `webview/mod.rs:1821` gates on `plugin_command.is_some()
   || has_app_acl_manifest || !is_local`, and `is_local` is decided by
   `Webview::is_local_url`, which compares the request URL against **the
   platform's** tauri protocol URL. On macOS that is `tauri://localhost`. The
   Tauri docs' own first example uses the literal `http://tauri.localhost`,
   which is the Windows/Android spelling and is a *remote* origin here — with
   it, all eight tests failed `<cmd> not allowed. Plugin not found`, an ACL
   rejection that looks exactly like the fixture-config problem the note
   promised would not arise. The fix is the docs' own conditional
   (`if cfg!(any(windows, target_os = "android"))`), and it is in `ipc.rs` with
   a comment saying why. **Anyone adding a Tauri IPC suite to another unit
   needs this.**
3. `next_outcomes` returns `Vec<ReceiverOutcome>`, not the single outcome step
   2.2's snippet shows. A terminal error must produce **two** outcomes —
   `VerifyFailed` (REQ-affyf5) then `Stopped` (REQ-jfxah3) — which the
   single-outcome signature cannot express. The loop keeps the plan's shape
   otherwise, including the verbatim `let _guard = guard;` line.
4. `AppState::assemble` is a private shared build path that `init` and
   `for_test` both call, so the test constructor cannot drift from the real one
   in how the store is opened or the transport is set. `for_test` differs from
   `init` in exactly one way: it wires `ChainNotConfigured` instead of calling
   `build_chain_ops`.
5. **T1's finding 5 is closed.** The private `parse_org_id` is gone from
   `commands.rs`; `crate::parsing::parse_org_id` is the unit's only
   implementation. `chain_ready` survives only as a word in a `state.rs`
   doc-comment explaining what replaced it.
6. T1's finding 2 held: `cp -Rc` of the target directory took ~2 seconds and
   the first test build was 1.66s rather than a cold tauri + subxt + iroh
   build.

### T3 — done 2026-09-14, branch `worktree-guardrails-app-risk-t3` @ `ab63fe9`

24 tests, 3 files, all passing; `npm run check` 193 files, 0 errors, 1 warning
(the pre-existing `@types/node` one, unchanged from the 166-file baseline).

Red → green, as reported: **the red is per file, not per test.** Each of the
three modules was new, so the first assertion in each file could not run until
the module it names existed — vitest reported `Cannot find module
'../src/lib/<name>'` and collected 0 tests from that file while the other files
stayed green. Once the module was written, every test in that file went green on
the first run with no intervening edit.

- all 10 `revoke.validate` tests — watched failing with `Cannot find module
  '../src/lib/revoke'` before `revoke.ts` existed.
- all 8 `verify.row` tests — watched failing with `Cannot find module
  '../src/lib/verify'` before `verify.ts` existed, the other 10 still passing.
- all 6 `receiver.subscribe` tests — watched failing with `Cannot find module
  '../src/lib/receiver'` before `receiver.ts` existed, the other 18 still
  passing.

This is a weaker attestation than a per-test red and it is recorded as such: a
module-absent failure proves the test could not have passed before the code
existed, which is the iron law's substance, but it does not prove each
individual assertion discriminates. What supplies that for the three assertions
that matter is T4's prescribed mutations, each of which must redden a *named*
test in these files.

Three findings, all acted on:

1. The plan's step 3.1 prediction was wrong and T3 measured it — `npm run check`
   **does** type-check `app/tests`. Step 3.1 corrected above; the verification
   record states the gap as closed by measurement.
2. vitest was absent from the task worktree's `package.json` because the change
   worktree's copy was an uncommitted modification. The dispatcher discarded
   that stray edit; T3's committed version is authoritative.
3. `app/package-lock.json` records no vitest, which would have made T5's new CI
   job red on arrival. New step 5.0 above.

### fix1 — done 2026-09-14, branch `worktree-guardrails-app-risk-fix1` @ `f40738e`

Raised by the **verification gate**, round 1, as a check 6 (robustness
completeness) shortfall. Not a review finding — the gate found it before the
independent review ran.

**The finding.** REQ-tw4cb5's two tests both drove
`ReceiverOutcome::SelfDeleted { org_id: org() }` — one input, one fixture. Under
class C every requirement needs a normal-case *and* an abnormal-input test, and
a persona self-deleting is a legitimate action, so the two existing tests were
both on the normal path. The gate also noted this is a *weaker* case than the
two exceptions T1 recorded, not a stronger one, and that T1's closing
completeness claim was false for it.

**The fix: three tests, `receiver_events` 14 → 17, total 59 → 62.**

| test | what it adds |
|---|---|
| `self_delete_payload_carries_no_epoch_key_at_any_depth` | asserts on the emitted JSON payload's keys at any depth, not on the event name |
| `self_delete_carries_no_epoch_for_any_organisation_id` | drives an all-zero and an all-ff organisation id, making the absence a property across inputs rather than a fact about one fixture |
| `the_epoch_probe_finds_an_epoch_on_updated_and_none_on_self_delete` | the contrast case: the same probe finds an epoch on `Updated`, where one legitimately belongs, and none on the self-delete path — so a probe that always finds nothing cannot pass |

**Genuinely red, not red by mutation.** The red was produced by restoring
`epoch: 0` to the self-delete emission in `events.rs` — which is precisely the
regression HAZ-5ha5vv is about — and each test was watched failing against it:

- `` `revoked` payload carries an epoch key: ["epoch", "org_id"] ``
- `org_id "0000…0000": an emission carried an epoch / left: Some(Object {"epoch": Number(0), …}) / right: None`
- `and find none on the self-delete path / left: Some(Object {"epoch": Number(0), …}) / right: None` — **with the `Updated` half of the same test passing**, which is the discrimination the contrast case exists to show.

`events.rs` was then reverted and confirmed byte-identical (`git diff --stat`
printed nothing; `git status --porcelain` showed only the test file).

Two findings of its own:

1. **The gate's characterisation was slightly too strong, and so was mine when
   I relayed it.** `self_delete_emits_no_epoch_event` already had a shallow
   `payload.get("epoch")` check, so it is not true that *nothing* checked the
   payload. What was true — and what the three new tests fix — is that the check
   was top-level only, on one fixture, with no contrast case. The shortfall was
   real; its description was overstated by one word.
2. **`cargo fmt --check` is dirty across `app/src-tauri` and was already dirty
   before this change** — `commands.rs`, `state.rs`, `lib.rs`, `events.rs` and
   four test files. There is no `rustfmt.toml`, no fmt step in `verify_commands`
   and none in CI, so nothing converts this into a failure. fix1 left the
   formatting as written rather than reformatting files it did not own. This is
   the same open question `on-chain-client`'s register raised for its own crate
   and it now applies to two units; it belongs in the ratchet-setup checklist
   as a decision, not in a hazard analysis.

### fix2 — done 2026-09-14, branch `worktree-guardrails-app-risk-fix2` @ `67ce503`

Raised by the **verification gate, round 2**, which judged it "worth a sentence
in the verification record; not worth blocking on". Fixed rather than recorded,
because the defect is in the evidence itself and a misnamed test is exactly what
an independent reviewer should not have to find.

**The defect.** `unconfigured_chain_reports_false_even_with_env_set` never set
`ODS_CHAIN_WS`. It only *read* whatever the variable ambiently held, to quote in
its failure message — so wherever the variable is unset, which is CI and every
run to date, the "even with env set" clause in its name was vacuous. A test name
is a claim about what was verified.

REQ-bvx4nh has two clauses. The first — the status reports the endpoint the
running configuration was built from — is properly gated. The second — it is not
re-read from the environment — is **true by construction**:
`connection_status_from_state` takes all three inputs as parameters and no
`std::env::var` call is reachable from it. The misnamed test papered over the
difference.

**The fix, and what was deliberately not done.** The test is renamed to
`absent_chain_reports_false_and_no_endpoint_fields_whatever_the_ambient_env` and
carries a comment stating what it does and does not establish. No environment-
setting test was added — cargo runs integration tests as threads of one process,
so it would race every other test in the binary and fail intermittently, blaming
whichever test happened to read the variable. No source-text grep for
`std::env` was added either; both would be worse than saying so plainly. The
unreached clause is now claim 9 of the register's §3.

**Red → green by mutation.** `chain_configured: chain.is_some()` → `true` in
`policy.rs:138`:

    thread 'absent_chain_reports_false_and_no_endpoint_fields_whatever_the_ambient_env'
    panicked at tests/connection_status.rs:73:5:
    no chain was built, so none may be reported

`unconfigured_chain_reports_neither` failed alongside it, as it should — it
asserts the same clause under REQ-e4ah9h. 6 passed, 2 failed. `policy.rs` was
then reverted and verified byte-identical **by SHA-256**, not by `git diff`
alone: `5136e084…` before the mutation and after the revert, against `a2d63ae3…`
while mutated.

62 tests, unchanged — 12, 8, 11, 17, 6, 8.

One finding, not acted on: `rustfmt --check` reports diffs in
`connection_status.rs` at code fix2 did not touch. Pre-existing, consistent with
fix1's finding, and covered by the repository-wide rustfmt decision now in
`docs/plans/2026-09-05-ratchet-setup.md`.

### fix3 and the round-3 count corrections — done 2026-09-14

Two things, from the verification gate's round 3.

**fix3, branch `worktree-guardrails-app-risk-fix3` @ `4a3f0cc`.** The gate's
systematic sweep — every one of the 86 test names read against its body, which
neither earlier round had done — found a second instance of fix2's defect class.

`revoke_member_accepts_an_empty_peer_addr` in `app/src-tauri/tests/ipc.rs`
calls `invoke_err`, which **panics if the command succeeds**. The test therefore
requires the revocation to fail, and asserts only that the failure message does
not mention `peer_addr`. Nothing is accepted. It had borrowed the verb from its
correctly-named frontend twin in `revoke.validate.test.ts`, where
`validateRevokeInput` genuinely returns `{ ok: true }` and the verb is earned.

Renamed to `revoke_member_is_not_refused_for_an_empty_peer_addr`, with a comment
stating that the failure is *required* by the harness (a fresh store has no such
organisation and no such member; a real success would need an org, a member, a
chain and a reachable peer) and that REQ-vgr7s2's acceptance clause is
established on the frontend, where the decision lives. The assertion is
unchanged; its failure message was reworded from "must be accepted" to "must not
be the ground of refusal", so that it does not re-assert the claim the name just
gave up.

fix3 also corrected the section banner three lines above — `an empty peer address
is genuinely accepted` — which carried the identical overstatement in the same
file. Beyond the two edits specified, inside the one file it owned, and right:
leaving it would have left the defect half-fixed with the false wording adjacent.

Red → green by mutation: an unconditional non-empty check on `peer_addr_blob`
restored in `commands.rs` produced `an empty peer address must not be the ground
of refusal in Networked transport, got: peer_addr_blob must not be empty`
(ipc.rs:263). `commands.rs` verified byte-identical after revert by SHA-256,
`ab70d44c…` both sides. 62 tests, unchanged.

**Three stale test counts, corrected on the change branch.** fix1 raised
`receiver_events` from 14 to 17, and the dispatcher updated one figure and not
the others — leaving `app/.guardrails/config.yaml` saying **62** on line 63 and
**59** on line 88, twenty-five lines apart in one file, plus "83 tests … 59 of
them" in the ratchet checklist and "= 83" in this plan's self-review. **Three
figures were wrong and are corrected**; `config.yaml:63` was never among them —
it is the cargo-only subtotal, it correctly read `= 62`, and an earlier draft of
this paragraph counted it as a fourth correction, which the verification gate's
round 4 then caught. This plan's self-review line now distinguishes *as planned*
(83) from *as built* (86).

Worth naming for what it is: this is the same defect this change spent its
length finding in the product — **a claim that outlived what supported it** —
committed in the change's own documentation, and then committed again in the
very paragraph recording the first instance. The gate caught both because it
reads the numbers instead of trusting them. That is the argument for the gate
being a fresh subagent with no memory of the change: an author re-reading their
own correction does not re-count it.

**Two observations carried into the verification record rather than fixed:**

- **REQ-dp95pv's two tests share one fixture** (`org()` = `"aa"×20`), which is
  structurally the weakness fix1 removed from REQ-tw4cb5 — a payload equality
  that would pass against an implementation hard-coding that value. Milder,
  because dp95pv is a positive existence claim rather than a universal negative,
  so a fixture is less likely to be incidental; the gate did not block on it and
  neither does this change. It is the one place where fix1's lesson was applied
  to the requirement that prompted it and not to its neighbour on the same code
  path.
- **`npm audit` reports 4 vulnerabilities** (1 low, 1 moderate, 2 high) in the
  frontend tree. Not introduced here — the tree predates this change — but this
  change is what makes the frontend a gated, CI-installed artefact, and no
  `verify_command` in any unit will ever surface them. Raised as a decision in
  `docs/plans/2026-09-05-ratchet-setup.md`, alongside the note that app's SOUP
  table is still the empty template and is where advisories of this kind belong.

### fix4 and the round-4 accuracy corrections — done 2026-09-14

**Gate round 4's commands all passed** — 86 tests at that point, every script
exit 0, clean tree, all 37 Implements IDs mapped, check 6 holding for all
nineteen requirements. Its nineteen findings came from a documentation sweep
the dispatcher commissioned beyond checks 1–7: every count checked against
measurement, every path, function and test name in prose checked against what
exists, and the register's code quotations spot-checked for before/after tense.

**The nineteen break down as: sixteen wrong claims in prose, two smaller
documentation observations, and one code defect.** Spelled out because gate
round 5 pointed out that the total did not follow from the text as written. The
two not discussed below are round 4's findings 17 and 18: the plan's
`receiver_events` table omitted fix1's three tests (corrected, see §T1 step 1.6's
as-built note), and a discrepancy between "ten further entries" and "nine
siblings" in two accounts of the same `npm ci` lockfile failure, which is
**not corrected** — re-measuring it would mean restoring the pre-change lockfile,
and the number is incidental to a failure both accounts agree happened.

**Sixteen were wrong claims in the change's own prose.** The register's
quotations of pre-change code were all exact, and so were the Rust line counts,
the ID arithmetic and every §6 citation — but sixteen statements about the
*evidence* were not. The three that mattered:

- *"Twelve command handlers, driven end to end."* The IPC suite invokes **four**
  (`connection_status`, `export_invite`, `revoke_member`, `list_personas`) plus
  one deliberately unknown name. The other eight are registered and never
  called, and registration is not exercise — the renamed-argument and
  renamed-DTO-field defects the paragraph promises to catch would still pass for
  two-thirds of the boundary. This is `Membership.svelte`'s unreachable
  `✗ MISMATCH` restated in prose: a claim of verification that cannot fail.
- *"That single point is gated by REQ-645jq9's tests on both sides of the IPC
  boundary."* All six tests carrying that ID are Rust. No frontend test verifies
  it. The sentence was the acceptance argument for a hazard RC-jrkn7w
  *introduces*, so the gap between "both sides" and "one" is the gap between an
  accepted risk and an assumed one.
- *"The startup refusals are tested at `AppState::init`, which is where the
  decision lives."* Both halves false: no test calls `init`, and `state.rs`'s
  own module doc says the policy functions decide. A paragraph whose job was to
  state a gap precisely misstated which gap.

Also corrected: the dev passphrase is fifteen characters, not eleven; the
frontend is 1,384 lines, not the "~350" estimated and never re-measured — inside
a sentence claiming all of it had been read; `receiver_guard.rs` was named
nowhere in a 900-line register despite being the sole gate for two requirements;
"`StartGuard`'s release-on-drop is gated six ways" was three (six is the file's
size, not the property's evidence); a requirements header promised "two
exceptions" that never existed; §9's derived-item totals said four and sixteen
where the enumerations held five and fourteen; PR-eecx3y described
`AppState.chain_ready`, a field this change deletes; and two checklist items in
the ratchet plan were falsified by this very change.

**And one with a certain symmetry.** The paragraph recording fix3's stale-count
corrections said the two contradicting figures were "fourteen lines apart" (25)
and claimed four corrections where three were due. A claim outliving its support,
committed inside the paragraph about a claim outliving its support. That is the
argument for the gate being a fresh subagent with no memory of the change: an
author re-reading their own correction does not re-count it.

**fix4, branch `worktree-guardrails-app-risk-fix4` @ `a73705d`.** The one code
finding. `str::trim_start_matches("0x")` strips **every** leading occurrence, so
`"0x0x"` plus a well-formed body was accepted wherever one `0x` would be.

Fixed at the two boundaries a requirement governs — the organisation identifier
(REQ-sjkp8z) and the member identifier (REQ-he8ejb) — with
`strip_prefix("0x").unwrap_or(…)`, so the second prefix counts toward the width
check that follows. Four tests added; `org_id_parsing` 11 → 14, `ipc` 8 → 9,
total 62 → 66.

Red → green, reported honestly as **two of four**:

- `a_doubled_zero_x_prefix_is_refused` — red: `exactly one 0x prefix is
  permitted, not two: OrgId(0xefef…efef)`. The malformed input was accepted and
  yielded an `OrgId`.
- `revoke_member_rejects_a_doubled_zero_x_prefix_on_the_member_id` — red: `must
  be refused at the boundary rather than normalised into a valid id, got: no
  on-chain state found for org`. The handler's validation passed and the call
  reached the chain layer.
- `a_zero_x_in_the_middle_is_refused` and `the_bare_prefix_alone_is_refused` —
  **not red.** Both passed against the unfixed source: a mid-string `0x` is not
  at the start and was never stripped, and `"0x"` alone already failed the width
  check. They are boundary coverage, not reproductions, and the task said so
  rather than claiming a red it did not observe.

**What the defect is not.** Over-acceptance, never mis-parsing. `x` is outside
the hex alphabet, so `0x` cannot occur inside a well-formed body and no
identifier can be parsed as a *different* identifier — `"0x0x"` plus 40 hex names
the same organisation as `"0x"` plus 40 hex. Nobody revokes the wrong member
because of this. It is a class C boundary parser silently normalising malformed
input: a robustness failure, not a route to harm, which is why the remainder is a
problem report rather than a hazard.

**Five further sites, recorded as PR-5mc4d8 rather than fixed.** fix4 found the
same pattern at `org_secret_hex` and `peer_addr_blob` in `commands.rs` — which
are operator input across the IPC boundary, so the "startup env" argument does
not cover them — and at `ODS_CONTRACT_H160`, `ODS_ADMIN_SEED` and
`ODS_COSIGNER_PUB` in `state.rs`. It declined to fix them, correctly: no
requirement governs any of the five, so a fix would carry a test that could not
be annotated, and `check-trace.sh` would report it as behaviour with no
requirement behind it. The sharpest is `ODS_CONTRACT_H160`, where the *raw,
unstripped* string is what reaches `ChainEndpoint` and therefore the status
display — so the app would talk to `0x…` while showing the operator `0x0x…`,
which is HAZ-ny7yvt's family.

Closing PR-5mc4d8 means one requirement covering prefix **and whitespace**
handling at every operator-supplied identifier in the unit, which is why it is
one report and not six. (The report was widened on 2026-09-14, after this
section was written, when fix6 found the two parsers also disagree about
whitespace: `validateRevokeInput` trims and `parse_org_id` does not, so a padded
organisation identifier is refused for its length. Both item forms describe
their own parser accurately; no item says which policy the unit intends.)

### Gate round 5 — confirming round, seven accuracy findings

All seven checks passed: 66 cargo + 24 vitest = **90**, every script exit 0,
tree clean, 37/37 Implements IDs mapped, check 6 holding for all nineteen
requirements. **No new defect class.** Every finding was the fourth class —
documentation accuracy — which round 4 had already opened.

Round 5 was asked to re-measure round 4's corrections rather than trust them,
and **they all landed correctly**: the frontend is 1,384 lines across 12 files
on master (and the derived "742 lines of `<script>` plus `api.ts` and
`+layout.ts`" reproduces exactly); the passphrase is fifteen characters; the IPC
suite invokes four of twelve registered handlers and the eight named as
uninvoked are exactly the set difference; release-on-drop is gated three ways;
§9's 5 + 14 = 19 partitions the ordinary requirements with no overlap and no
omission. Of 106 backticked identifiers in the register and problems file, the
only ones absent from source are two config keys, `on-chain-client`'s
`log_is_ours`, the `receiver_status` command correctly described as not
existing, and the deliberately-quoted old test name.

**The finding that matters is not any individual error — it is that round 4's
sweep was itself incomplete.** Two of round 5's seven findings sit in material
round 4 read and passed, and a third was introduced by the commit that recorded
round 4's corrections. "Each round smaller than the last" is therefore not
evidence the class is closed, and this section should not be read as claiming it
is.

Corrected here:

1. **PR-5mc4d8 said "two of the six"; there are seven sites.** Two fixed plus
   the five enumerated in its own body, one screen below. Measured on master:
   `parse_org_id`, `org_secret_hex`, `member_id_hex`, `peer_addr_blob`, and three
   in `state.rs`. This is the count a future reader uses to know when the report
   is closable.
2. **PR-5mc4d8's `peer_addr_blob` bullet was wrong on the behaviour.** It said a
   doubled prefix "fails at decode either way". It does not:
   `peer_addr_blob.trim().trim_start_matches("0x")` removes both prefixes, so the
   hex decodes and `postcard` succeeds and the revocation proceeds; under the
   single-strip rule the surviving `"0x"` is refused on the `'x'`. The site was
   correctly listed and its justification was false — asserted from the shape of
   the code rather than traced through it. The `ODS_CONTRACT_H160` display claim
   in the same report was checked and **is** true: `state.rs` stores the raw
   unstripped variable into `ChainEndpoint` and `policy.rs` returns it verbatim.
3. **"Three open defects" in the ratchet plan's restriction paragraph; there are
   four** — 112 lines from the line the same commit updated to say four. The
   round-3 defect (62 vs 59, twenty-five lines apart) reintroduced by the commit
   that added PR-5mc4d8. The repository-wide figure in that file, "fourteen
   across four ledgers", was checked and is correct.
4. **The register pointed at the wrong deferred control** — "not-minted control
   4" for *prompt for the passphrase at first run*, which §8 numbers 2 (4 is the
   receiver liveness query). The only such error in the register; the ratchet
   plan had it right.
5. **Three post-change sentences still named `chain_ready`**, the field this
   change deletes — in the register's *What these controls do not fix*, in
   REQ-x3c8n2's rationale, and in the ratchet plan's PR-eecx3y checklist item,
   which is the one a future reader works from. Round 4 corrected exactly this in
   PR-eecx3y and nowhere else, with the rationale that "a problem report naming a
   field that does not exist is one a future reader cannot act on" — which
   applies identically to the other three. All now say "the chain verdict".
6. **"Nineteen findings" did not follow from the enumeration.** It does — 16
   prose + 2 smaller documentation observations + 1 code — but the section never
   said so and a reader could not check it. Now spelled out, including the one
   item deliberately left uncorrected.
7. **`org_id_parsing.rs` kept an as-planned count with no as-built note**, while
   `receiver_events.rs` three paragraphs later had one, for a situation
   identical in kind. Asymmetric treatment inside one document; both now carry
   the note.

Two observations round 5 recorded without blocking, carried to the verification
record: REQ-dp95pv's two tests still share one fixture (round 3's observation,
unchanged), and REQ-jfxah3's and REQ-bmk2z2's abnormal-input cases are bounds
and message assertions rather than malformed input — because the behaviour each
requires exists only on a failure path, which is the argument already recorded
for REQ-bmk2z2 and accepted by three successive gate rounds.

---

## Independent review, round 1 — eight findings, four blocking

`merge-change` step 6a. A fresh subagent, given the repository and the artefacts
and **no account of how the work went**, required to run the suite itself. It
reproduced 90/90 and then declined to merge on four findings. All eight were
accepted; none was disputed.

**The blocking four, and what they cost:**

**finding-2 — the change introduced a new instance of the hazard it exists to
fix.** Before this change, `start_receiver` emitted `receiver-error` for
recoverable failures and kept looping. The change routed *every* `Err` into
`VerifyFailed`, which `Membership.svelte` renders as `✗ MISMATCH` under the
heading "Verified Updates (chain root match)". So a transport fault or a chain
read failure was displayed to the operator as a root mismatch — and
`receiver-error` lost its only emitter in the same edit, leaving the panel's
"Receiver Errors (non-fatal)" block as dead markup and its API doc-comment
false.

HAZ-9fmhm4 is *the verification display misrepresenting the verification state*.
Before: it could not report a failure. After the first implementation of its
controls: it reported failures that were not verification verdicts. §5, whose
entire job is to ask what each control breaks, listed four introduced hazards
and not this one — because it was written assessing the controls **as designed**
rather than **as built**.

Fixed by fix5 under **RC-3rddh7**, minted for it, with REQ-kn5rtx (classify by
the error's type) and REQ-wu6z9p (the consumer half). The classification was
available all along and was being discarded: `OrgNodeError` is a typed enum, and
the code called `e.to_string()` before deciding anything. `classify_receive_error`
now matches on the variant with **no wildcard arm**, so a variant added upstream
is a compile error rather than a silent misfiling.

**finding-1 — a claimed reduction that no code path delivers.** REQ-affyf5 is
worded "where the failing update names one", and the honest reading of that
qualifier is *never*: no `OrgNodeError` variant carries an organisation, so the
only production site must hard-code `None`. The requirement is satisfied
vacuously; the register's "failures gain an identifier" was true of no run and is
withdrawn. Its gating test constructs a payload production code cannot produce,
so it could not fail if the behaviour broke — kept for its shape, recorded as
claim 11. The *unlocatable* half of HAZ-9fmhm4 is not reduced by this change at
all, and REQ-x3c8n2's scope now covers it.

**finding-3 — an assertion keyed to a diagnostic's spelling.**
`revoke_member_is_not_refused_for_an_empty_peer_addr` asserted
`!err.to_lowercase().contains("peer_addr")`. The original defect's message was
`Peer addr blob (hex) is required.` — which lowercased contains no such
substring, so reintroducing the exact regression left the test green. fix5
confirmed this **empirically**, not by reading: `printf 'Peer addr blob (hex) is
required.' | tr A-Z a-z | grep -c peer_addr` → 0. The assertion now derives its
expected text from `OrgNodeError::OrgNotOnChain.to_string()`.

**finding-5 — two annotations claiming a requirement their code cannot bear.**
REQ-he8ejb requires rejection *before any command is invoked*; two `ipc.rs` tests
carried it while asserting the handler's check, which runs after. Deleting the
frontend check — the behaviour REQ-he8ejb actually names — left both green. The
annotations are removed and the tests kept as handler-side defence in depth, with
the reason in the file.

**The four non-blocking**, all accepted and fixed: REQ-rq8g2v's "after teardown
began" clause is gated by nothing (claim 10); the data directory stopped being
re-derived from the environment with no requirement covering it (REQ-bvx4nh
widened); REQ-sjkp8z and REQ-he8ejb said "not exactly 40 / 64 hexadecimal
characters" while their tests assert a `0x`-prefixed input is accepted, and `x`
is not a hexadecimal character (both amended); §3 claim 3 and claim 9 were stale.

**fix5 — branch `worktree-guardrails-app-risk-fix5` @ `901cdb6`.** 72 cargo + 30
vitest = **102**. Twelve tests added, all watched red first:

- six for REQ-kn5rtx in `receiver_events.rs` — red as 8 compile errors, "cannot
  find function `classify_receive_error`" and "no variant named `ReceiveError`".
- six for REQ-wu6z9p in the new `app/tests/receiver.log.test.ts` — red as
  `TypeError: applyReceiverEvent is not a function`, 6 failed / 24 passed.
- the finding-3 mutation: the original wording restored in `commands.rs`,
  `revoke_member_is_not_refused_for_an_empty_peer_addr` failed with left `"Peer
  addr blob (hex) is required."` / right `"no on-chain state found for org"`;
  reverted, `commands.rs` byte-identical by SHA-256 `20b00ae7…`.

Two things fix5 reported that are worth keeping: the frontend was already
half-right — `Membership.svelte` already routed `onReceiverError` into its error
list and already pushed no verification row, so the block was dead purely because
nothing emitted the event, and the regression was entirely on the Rust side. And
`app/src-tauri/Cargo.toml` gained one dev-dependency, `org-members`, because
testing `OrgNodeError::Trie(_)` needs `org_members::OrgMembersError` and org-node
does not re-export it.

## Independent review, round 2 — thirteen findings

A second fresh reviewer, given no knowledge of round 1, so a mis-applied fix had
to be found as a defect rather than recognised as an answer. It reproduced
102/102 independently. **Two of its findings were caused by round 1's fixes** —
which is the reason the rounds are run blind.

Four were fixed (owner's decision, 2026-09-14: fix the blocking four, record the
rest):

- **§3's headline claim was false for three requirements.** "Every decision
  named in a requirement below was moved out … into a free function, and gated
  there" does not hold for REQ-2k7ys4, REQ-dp95pv or REQ-jfxah3: their
  *decisions* live in `next_outcomes`, and what is gated is `emissions_for`,
  which is handed an already-chosen outcome. The consequence is gated; the
  recognition is not. Corrected, and added as claims 12 and 13.
- **REQ-bvx4nh's new data-directory clause was gated by no test annotated to
  it** — a direct consequence of round 1's amendment. Tests added.
- **REQ-he8ejb's amended wording forbade what its own test requires**: it now
  reads "optionally preceded by a single `0x` prefix", and whitespace is neither
  hexadecimal nor that prefix, yet `trims surrounding whitespace before
  measuring` asserts whitespace is accepted. Also a consequence of round 1's
  amendment — sharpening the wording exposed a permission that lived only in the
  code. Amended a third time.
- **This section.** The plan carried no record of the fix5 round, so its latest
  measurement read "66 cargo + 24 vitest = 90 … nineteen requirements" while the
  tree held 102 and 21.

The other nine findings were recorded rather than fixed, and they collapse into
**six** distinct items because several were facets of one gap — which is why
PR-j9f6kk enumerates six: three frontend behaviours
gated under requirements that do not describe them (the `'(unknown
organisation)'` substitution, the `detail` column, the `receiver-stopped`
consumer), the glossary declaring Persona and Epoch as defined in the root
glossary where neither is defined, and the ledger using "user" nine times for a
term its own glossary lists under `_Avoid_`. Age limit 2026-10-14.

## Independent review, round 3 — seven findings, two of them false claims

A third fresh reviewer, again given no knowledge of its predecessors, and told
explicitly that recorded deferred work — a problem report, a not-minted control —
is *not* a finding, so that it would not re-report the project's own debt
mechanism as discovery. It reproduced 104/104 independently and returned seven
findings, of which two were not deferred work but **claims of truth that were
false**. Both were verified against the source before acting.

**finding-1 — the receiver's terminal-error detection had never worked.** The
loop matched the single substring `"endpoint not bound"`. That string occurs
nowhere in `org-node` or `iroh`. What the receive path actually produces is
`"endpoint bind: …"`, `"endpoint bind failed unexpectedly"` and `"endpoint
closed"` (the last arriving as `chain read failed: iroh recv: iroh accept error:
endpoint closed`). So the loop had never broken, `receiver-stopped` had never
been emitted in production, and the guard had never been released by task exit.

Three statements rested on it working: §3's claim 7 said the substring appears
"in exactly the situations the receiver loop treats as terminal"; HAZ-cfp4jb's
residual said "the state stops being absorbing — the exit is announced and a
restart works"; and REQ-x3c8n2's rationale warned that *rewording* the message
would break this, when it was already broken. **REQ-jfxah3 was satisfied
vacuously** — its tests asserted that a `Stopped` outcome produces one
`receiver-stopped` emission, which was true, and nothing produced a `Stopped`
outcome. That is the second requirement in this change with correct tests over a
behaviour the system never exhibited; REQ-affyf5 was the first.

And the real behaviour was worse than the register's description of it.
`ensure_endpoint` caches the endpoint and rebinds only when it is `None`, so
after `"endpoint closed"` the loop reused the dead endpoint indefinitely: not a
quiet absorbing state but a hot spin, emitting one error event per iteration.

**finding-2 — RC-3rddh7's classification was wrong for two of the nine variants
it called verification verdicts**, and wrong in the direction that reinstates the hazard it was minted
to remove. `OrgNotOnChain` and `Trie(_)` were classified as verification
verdicts. Both are *also* raised by purely local conditions in the receive path —
the local store having no record of the organisation
(`org-node/src/service.rs:1307`), and the local members snapshot failing to
reconstruct (`:1308`) — with no bearing on whether anything verified. So a
local-state problem rendered as `✗ MISMATCH` under "Verified Updates (chain root
match)", which is §5's fourth entry happening again, through the control that
fixed it.

Both are now receiver errors. The app cannot tell the origins apart from the
variant, and the rule is that a verdict must not be claimed where it cannot be
supported: a misfiled verdict is the hazard, a misfiled receiver error is only
less specific.

**fix7 — branch `worktree-guardrails-app-risk-fix7` @ `90603db`.** 78 cargo + 30
vitest = **108**. Four tests added, and the reds are the evidence the mechanism
was dead:

- `every_real_terminal_message_stops_the_loop` — red against the live constant:
  the real bind-failure message produced `[ReceiveError { … }]` and **no
  `Stopped` outcome at all**.
- `the_stop_is_announced_after_the_failure_for_the_same_error` — red:
  `left: ["receiver-error"]`, `right: ["receiver-error", "receiver-stopped"]`.
- `locally_reachable_variants_are_classified_as_receiver_errors` — red:
  `OrgNotOnChain is reachable from a local condition and must not be a verdict`.
- `a_non_terminal_failure_does_not_stop_the_loop` — **did not go red**, and fix7
  reported it as a measured negative rather than dressing it up: with a matcher
  that never fired, nothing was terminal, so "does not stop" held for free. Its
  job is to constrain the *new* markers against over-matching, and it only means
  anything now that its sibling is green.

The `Err` arm was extracted to `events::outcomes_for_receive_error` so the
failure-then-stop emission order is testable without an `AppHandle` — the same
extract-then-gate move as everywhere else in this change — and that extraction
was committed as behaviour-preserving, with the 23 existing tests still green,
*before* the constant changed.

Three things fix7 found that the review had not, all recorded:

- **`Trie(_)` has four origins, two of them local**, so the reclassification was
  under-argued by the finding rather than over-argued.
- **Every other locally-reachable variant is `Chain(_)`**, which was already in
  the receiver-error class — so the misclassification was exactly these two.
- **`DeltaBaseMismatch` and `RootMismatch` compare the envelope against the
  *local* trie**, so a corrupt local snapshot yields a verdict against a good
  envelope. Both stay classified as verdicts, defensibly — verification is a
  verdict on the pair — but whether the display should distinguish "your copy is
  wrong" from "their update is wrong" is a question no item answers. Recorded
  against PR-5mc4d8 rather than acted on.

## Gate rounds 6, 7 and 8

Recorded together because none found a code defect and all three found the same
class in the prose: counts and cross-references that a later round invalidated.

- **Round 6** (102 tests): thirteen stale figures after the suite grew 90 → 102,
  plus the observation that the Implements list had not been updated for the
  review-minted items. That observation was recorded and not acted on, which is
  why round 8 found it again.
- **Round 7** (104 tests): six, including a config note saying "three of the six
  rose" when four had, and the frontend-requirement count being wrong in three
  places with three different figures.
- **Round 8** (108 tests): nine. The sharpest was structural rather than
  numeric — the §3 claims list was numbered 1-10, 12, 13, 11, so markdown would
  renumber by position and the live cross-reference "claims 12 and 13" would
  point at the wrong items. Also: the Implements list still three short (now
  forty, with the reason stated in the header), two ordinals left behind by an
  earlier fix, and "the one hazard whose residual is unchanged" where §7's own
  table says two.

**What this sequence is evidence of.** Eight gate rounds and three reviews found
exactly one class of defect in the documentation, repeatedly, and it is the class
this change's subject matter is about: a claim that outlived what supported it.
Every instance was introduced by a correct fix to something else. The gates that
catch it are the ones that *measure* rather than read — and the reason they kept
finding more is that each fix round moved the numbers again.

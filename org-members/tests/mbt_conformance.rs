//! Conformance test: replays traces of the membership model
//! (`org-members/quint/membership_mbt.qnt`) against the real `OrgTrie` through
//! quint-connect, requiring after every model action the same result, the same
//! error and the same membership state, plus root-hash equality classes that
//! match the model's.
//!
//! What "maps 1:1" means here is decided in
//! `docs/adr/2026-10-03-quint-conformance-gate.md` (decisions 4, 8, 10-12):
//! every model action has exactly one driver arm, every trie-yielding public
//! operation has a model action or a declared-abstraction-boundary entry
//! (`TRIE_OPERATIONS`, checked against the source), every action is taken in
//! the random run, and every error tag maps to exactly one crate result.
//!
//! Requires the `quint` CLI on PATH and a writable `$HOME` (Quint's default
//! rust backend fetches its evaluator into `~/.quint` on first use). It does
//! NOT skip when either is missing: `quint_preflight` names the cause and the
//! test fails (decision 9).

use quint_connect::runner::{self, RunConfig, TestConfig};
use quint_connect::*;
use serde::Deserialize;

use anyhow::anyhow;
use ed25519_dalek::SigningKey;
use org_members::delta::test_support;
use org_members::hasher::Blake3Hasher;
use org_members::trie::OrgTrie;
use org_members::types::{Handle, MemberId, MemberLeaf, Name, P2pDeviceKey, P2pMemberKey, Surname};
use org_members::OrgMembersError;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

type Trie = OrgTrie<Blake3Hasher>;

const SPEC: &str = "quint/membership_mbt.qnt";
const IDS: [&str; 3] = ["a", "b", "c"];
/// Key generations the model draws (0..=3) plus 4 and 5, which only a refused
/// five-device genesis record uses (devices 1..=5), so every key the driver
/// builds inverts.
const GENS: [i64; 6] = [0, 1, 2, 3, 4, 5];
/// Random-run bounds (decision 7): explicit, per unit.
const MAX_SAMPLES: usize = 100;
const MAX_STEPS: usize = 15;

/// Every model action, by its `lastAction` tag.
const ACTIONS: [&str; 10] = [
    "Init",
    "AddMember",
    "DeleteMember",
    "UpdateHandle",
    "UpdateNameSurname",
    "RotateKey",
    "AddDevice",
    "DeleteDevice",
    "Isolate",
    "ApplyDelta",
];

/// Decision 8(i): every public operation that yields a trie or a candidate,
/// mapped to the model action that drives it or to its declared abstraction
/// boundary. `trie_operations_table_matches_source` fails when the source
/// gains one that is not listed here.
const TRIE_OPERATIONS: [(&str, &str); 11] = [
    ("genesis", "Init"),
    ("add_member", "AddMember"),
    ("delete_member", "DeleteMember"),
    ("update_name_surname", "UpdateNameSurname"),
    ("update_handle", "UpdateHandle"),
    ("rotate_p2p_key", "RotateKey"),
    ("add_p2p_device", "AddDevice"),
    ("delete_p2p_device", "DeleteDevice"),
    ("emergency_isolate_member", "Isolate"),
    ("apply_delta", "ApplyDelta"),
    ("verify_against", "ApplyDelta"),
];

/// Declared abstraction boundary: what the model deliberately does not
/// describe, and where it is carried instead.
const BOUNDARY: [(&str, &str); 1] = [(
    "recalculate",
    "hash lifecycle: called by the driver after every change that leaves hashes \
     pending and to produce honest deltas; pending/recalculated state is carried \
     by integration_test.rs",
)];
// Also outside the model, with no public trie-yielding operation of their own:
// UTS#39 skeletons (model skeleton == handle; carried by the handle tests in
// integration_test.rs), encoding-level delta non-canonicality — ordering and
// duplicate list entries (delta_canonicality_fuzz), wire encoding
// (fuzz_tests.rs), SMT internals and hashing (integration_test.rs).

/// Mirror of the Quint `Key` record.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Debug, serde::Serialize)]
struct Key {
    owner: String,
    gen: i64,
}

/// Mirror of the Quint `Leaf` record. `Ord` because `ApplyDelta` picks a
/// `Set[Leaf]`.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Debug, serde::Serialize)]
struct Leaf {
    id: String,
    handle: String,
    skeleton: String,
    name: String,
    surname: String,
    #[serde(rename = "pKey")]
    p_key: Key,
    devices: BTreeSet<Key>,
}

/// Mirror of the Quint `Seed` record (one requested genesis member).
#[derive(Clone, Deserialize, Debug)]
struct Seed {
    id: String,
    h: String,
    devs: i64,
}

/// Mirror of the Quint `Op` record (one producer-side operation). A rotation's
/// new key is `{ owner: ko, gen: g }`.
#[derive(Clone, Deserialize, Debug)]
struct Op {
    op: String,
    id: String,
    h: String,
    ko: String,
    g: i64,
}

/// The verifiable model state. `prev` and `lastAction` are model bookkeeping
/// and are not compared.
#[derive(Eq, PartialEq, Deserialize, Debug)]
struct MembershipState {
    #[serde(rename = "orgExists")]
    org_exists: bool,
    trie: BTreeMap<String, Leaf>,
    #[serde(rename = "lastError")]
    last_error: String,
}

fn real_id(model_id: &str) -> MemberId {
    MemberId::new(blake3::hash(format!("id:{model_id}").as_bytes()).into())
}
/// One derivation for member and device keys, so model key equality is real
/// key equality: the model `Key { owner, gen }` used as a member key and as a
/// device key is the same 32 bytes, exactly as the crate's key index sees it.
fn real_key_bytes(k: &Key) -> ed25519_dalek::VerifyingKey {
    let seed: [u8; 32] = blake3::hash(format!("k:{}:{}", k.owner, k.gen).as_bytes()).into();
    SigningKey::from_bytes(&seed).verifying_key()
}
fn real_member_key(k: &Key) -> P2pMemberKey {
    P2pMemberKey::new(real_key_bytes(k))
}
fn real_device_key(k: &Key) -> P2pDeviceKey {
    P2pDeviceKey::new(real_key_bytes(k))
}
fn key(owner: &str, gen: i64) -> Key {
    Key {
        owner: owner.to_string(),
        gen,
    }
}

/// The record a fresh member is admitted with (model `mkLeaf`): member key
/// generation 0, initial device generation 1 -- one derivation serves both,
/// so they must differ.
fn new_leaf(id: &str, h: &str) -> core::result::Result<MemberLeaf, OrgMembersError> {
    MemberLeaf::new(
        real_id(id),
        Handle::parse(h)?,
        real_member_key(&key(id, 0)),
        Name::parse("n")?,
        Surname::parse("s")?,
        vec![real_device_key(&key(id, 1))],
    )
}

/// A genesis seed's record (model `seedLeaf`): devices of generations 1..=devs.
fn seed_leaf(s: &Seed) -> core::result::Result<MemberLeaf, OrgMembersError> {
    MemberLeaf::new(
        real_id(&s.id),
        Handle::parse(&s.h)?,
        real_member_key(&key(&s.id, 0)),
        Name::parse("n")?,
        Surname::parse("s")?,
        (1..=s.devs)
            .map(|g| real_device_key(&key(&s.id, g)))
            .collect(),
    )
}

/// The real record of any model leaf (an `ApplyDelta` upsert built in the
/// model, keys of any owner included).
fn leaf_of_model(l: &Leaf) -> core::result::Result<MemberLeaf, OrgMembersError> {
    MemberLeaf::new(
        real_id(&l.id),
        Handle::parse(&l.handle)?,
        real_member_key(&l.p_key),
        Name::parse(&l.name)?,
        Surname::parse(&l.surname)?,
        l.devices.iter().map(real_device_key).collect(),
    )
}

// --- inverse maps (domains are tiny, so brute force) ---
fn model_id_of(id: &MemberId) -> Option<String> {
    IDS.iter()
        .find(|m| real_id(m) == *id)
        .map(|m| m.to_string())
}
/// Every model key, over all owners: a member may hold a key whose `owner`
/// is another id.
fn all_model_keys() -> impl Iterator<Item = Key> {
    IDS.iter()
        .flat_map(|o| GENS.iter().map(move |g| key(o, *g)))
}
fn model_key_of(bytes: &[u8; 32]) -> Option<Key> {
    all_model_keys().find(|k| real_key_bytes(k).as_bytes() == bytes)
}

fn model_leaf_of(m: &MemberLeaf) -> Result<Leaf> {
    let mid = model_id_of(m.id()).ok_or_else(|| anyhow!("unknown member id"))?;
    let p_key = model_key_of(m.p2p_key().as_bytes())
        .ok_or_else(|| anyhow!("unknown member key for {mid}"))?;
    let mut devices = BTreeSet::new();
    for d in m.p2p_devices() {
        devices.insert(
            model_key_of(d.as_bytes()).ok_or_else(|| anyhow!("unknown device for {mid}"))?,
        );
    }
    Ok(Leaf {
        id: mid.clone(),
        handle: m.handle().to_string(),
        skeleton: m.handle().to_string(), // model invariant: skeleton == handle
        name: m.name().to_string(),
        surname: m.surname().to_string(),
        p_key,
        devices,
    })
}

/// Map a crate error to the model's error tag — exactly one crate result per
/// tag (decision 10). The model's handles have distinct skeletons, so a model
/// "ConfusableHandle" is the crate's `DuplicateHandle`; the crate's own
/// `ConfusableHandle` (distinct handles, one skeleton) is outside the model
/// and maps to "Other", forcing a visible mismatch if it is ever reached.
fn err_tag(e: &OrgMembersError) -> String {
    match e {
        OrgMembersError::IdNotFound => "IdNotFound",
        OrgMembersError::DuplicateId => "DuplicateId",
        OrgMembersError::DuplicateHandle => "ConfusableHandle",
        OrgMembersError::DuplicateDevice => "DuplicateDevice",
        OrgMembersError::DeviceNotFound => "DeviceNotFound",
        OrgMembersError::DeviceSlotsFull => "DeviceSlotsFull",
        OrgMembersError::EmptyDeviceList => "EmptyDeviceList",
        OrgMembersError::DeltaBaseMismatch => "DeltaBaseMismatch",
        OrgMembersError::VerificationFailed => "VerificationFailed",
        OrgMembersError::MalformedDelta("removed id not present in trie") => "StaleRemoval",
        OrgMembersError::MalformedDelta("upserted leaf identical to existing trie state") => {
            "NoOpUpsert"
        }
        OrgMembersError::MalformedDelta("id appears in both removed and upserted") => {
            "RemoveUpsertOverlap"
        }
        OrgMembersError::P2pKeyNotReplaced => "P2pKeyNotReplaced",
        OrgMembersError::DuplicateKey => "DuplicateKey",
        other => return format!("Other:{other:?}"),
    }
    .to_string()
}

/// Apply one producer-side operation (model `applyOp`).
fn apply_op(t: &Trie, o: &Op) -> core::result::Result<Trie, OrgMembersError> {
    match o.op.as_str() {
        "add" => t.add_member(new_leaf(&o.id, &o.h)?),
        "delete" => t.delete_member(&real_id(&o.id)),
        "handle" => t.update_handle(&real_id(&o.id), Handle::parse(&o.h)?),
        "rotate" => t.rotate_p2p_key(&real_id(&o.id), real_member_key(&key(&o.ko, o.g))),
        _ => Err(OrgMembersError::InvariantViolated),
    }
}

/// Action and outcome counts across every trace of a run (decision 8(ii)).
type Coverage = Arc<Mutex<BTreeMap<(String, String), usize>>>;

#[derive(Default)]
struct MembershipDriver {
    trie: Option<Trie>,
    prev: Option<Trie>,
    last_error: String,
    // root-hash equality classes: canonical model-state bytes -> root hex
    root_classes: HashMap<Vec<u8>, String>,
    coverage: Option<Coverage>,
}

impl MembershipDriver {
    fn with_coverage(coverage: Coverage) -> Self {
        Self {
            coverage: Some(coverage),
            ..Self::default()
        }
    }

    fn commit(&mut self, res: core::result::Result<Trie, OrgMembersError>) {
        match res {
            Ok(t) => {
                // Mutations leave the trie with uncalculated hashes; recalculate
                // so `root_hash()` succeeds. This does not change observable
                // model state (member contents), only fills the hash cache.
                // An ApplyDelta result is already calculated and is kept as is:
                // recalculating it is to be refused (PR-zqvs7t). On a pending
                // trie recalculate has no lawful failure, so an Err is a crate
                // defect (PR-499dzp).
                let t = if t.has_pending_changes() {
                    t.recalculate()
                        .unwrap_or_else(|e| panic!("recalculate after an accepted mutation: {e:?}"))
                        .0
                } else {
                    t
                };
                // Delta-path conformance: for the real mutation old -> t, the crate's
                // calculate_delta + apply_delta + verify_against must reproduce t's
                // root (the model's round-trip law, against the real crate). Both
                // tries are calculated here, so a missing root fails the step
                // rather than skipping the check (PR-499dzp).
                if let Some(old) = &self.trie {
                    let old_root = old
                        .root_hash()
                        .unwrap_or_else(|e| panic!("root_hash of the previous trie: {e:?}"));
                    let new_root = t
                        .root_hash()
                        .unwrap_or_else(|e| panic!("root_hash of the accepted trie: {e:?}"));
                    if old_root != new_root {
                        let delta = t
                            .calculate_delta(old)
                            .expect("calculate_delta failed on a real mutation");
                        let verified = old
                            .apply_delta(&delta)
                            .expect("apply_delta rejected a canonical delta from calculate_delta")
                            .verify_against(&new_root)
                            .expect("verify_against failed for the calculated delta");
                        assert_eq!(
                            verified.root_hash().expect("root_hash of verified trie"),
                            new_root,
                            "delta round-trip produced a different root than the direct mutation"
                        );
                    }
                }
                self.prev = self.trie.take();
                self.trie = Some(t);
                self.last_error = String::new();
            }
            Err(e) => {
                self.last_error = err_tag(&e);
            }
        }
    }

    fn cur(&self) -> core::result::Result<Trie, OrgMembersError> {
        self.trie.clone().ok_or(OrgMembersError::IdNotFound)
    }

    /// The real counterpart of a model leaf in an `ApplyDelta` upsert: the
    /// current record itself when the model leaf equals it, else the record
    /// built from the model leaf's fields.
    fn real_leaf(&self, cur: &Trie, l: &Leaf) -> core::result::Result<MemberLeaf, OrgMembersError> {
        if let Some(existing) = cur.get(&real_id(&l.id)) {
            if model_leaf_of(&existing).ok().as_ref() == Some(l) {
                return Ok(existing);
            }
        }
        leaf_of_model(l)
    }

    fn apply_delta_step(
        &self,
        kind: &str,
        ops: &[Op],
        removed: &BTreeSet<String>,
        upserted: &BTreeSet<Leaf>,
        target: &str,
    ) -> core::result::Result<Trie, OrgMembersError> {
        let cur = self.cur()?;
        match kind {
            // The producer performs `ops` and computes the delta itself
            // (REQ-wx3wpv: a change set the software produced).
            "honest" => {
                let mut produced = cur.clone();
                for o in ops {
                    produced = apply_op(&produced, o)?;
                }
                let (produced, delta) = produced.recalculate()?;
                let expected = if target == "true" {
                    produced.root_hash()?
                } else {
                    cur.root_hash()?
                };
                cur.apply_delta(&delta)?.verify_against(&expected)
            }
            // Computed against the previous record, applied to the current.
            "stale" => {
                let prev = self.prev.clone().ok_or(OrgMembersError::IdNotFound)?;
                let delta = cur.calculate_delta(&prev)?;
                cur.apply_delta(&delta)?.verify_against(&cur.root_hash()?)
            }
            // Built from removals and upserts at the current base, in
            // canonical (strictly increasing) order.
            "built" => {
                let mut delta = cur.calculate_delta(&cur)?;
                let mut ids: Vec<MemberId> = removed.iter().map(|i| real_id(i)).collect();
                ids.sort();
                let mut leaves = upserted
                    .iter()
                    .map(|l| self.real_leaf(&cur, l))
                    .collect::<core::result::Result<Vec<_>, _>>()?;
                leaves.sort_by(|a, b| a.id().cmp(b.id()));
                test_support::delta_set_removed(&mut delta, ids);
                test_support::delta_set_upserted(&mut delta, leaves);
                let candidate = cur.apply_delta(&delta)?;
                let expected = if target == "true" {
                    candidate.root_hash()
                } else {
                    cur.root_hash()?
                };
                candidate.verify_against(&expected)
            }
            _ => Err(OrgMembersError::InvariantViolated),
        }
    }

    /// Reconstruct the model trie from the real OrgTrie via inverse maps.
    fn model_trie(&self) -> Result<BTreeMap<String, Leaf>> {
        let mut out = BTreeMap::new();
        if let Some(t) = &self.trie {
            for m in t.members() {
                let l = model_leaf_of(&m)?;
                out.insert(l.id.clone(), l);
            }
        }
        Ok(out)
    }
}

impl State<MembershipDriver> for MembershipState {
    fn from_driver(driver: &MembershipDriver) -> Result<Self> {
        Ok(MembershipState {
            org_exists: driver.trie.is_some(),
            trie: driver.model_trie()?,
            last_error: driver.last_error.clone(),
        })
    }
}

impl Driver for MembershipDriver {
    type State = MembershipState;

    /// The model records each action in `lastAction` (decision 12): `quint
    /// test` traces carry no `mbt::actionTaken`, and named scenarios must
    /// drive this same driver.
    fn config() -> Config {
        Config {
            state: &[],
            nondet: &["lastAction"],
        }
    }

    fn step(&mut self, step: &Step) -> Result {
        let switch_result: Result = (|| {
            switch!(step {
                Init(seeds: Vec<Seed>) => {
                    let res = seeds
                        .iter()
                        .map(seed_leaf)
                        .collect::<core::result::Result<Vec<_>, _>>()
                        .and_then(Trie::genesis);
                    self.trie = None;
                    self.prev = None;
                    match res {
                        Ok(t) => {
                            self.prev = Some(t.clone());
                            self.trie = Some(t);
                            self.last_error = String::new();
                        }
                        Err(e) => self.last_error = err_tag(&e),
                    }
                },
                AddMember(id: String, h: String) => {
                    let res = self.cur().and_then(|t| t.add_member(new_leaf(&id, &h)?));
                    self.commit(res);
                },
                DeleteMember(id: String) => {
                    let res = self.cur().and_then(|t| t.delete_member(&real_id(&id)));
                    self.commit(res);
                },
                UpdateHandle(id: String, h: String) => {
                    let res = self.cur().and_then(|t| t.update_handle(&real_id(&id), Handle::parse(&h)?));
                    self.commit(res);
                },
                UpdateNameSurname(id: String, nm: String, sn: String) => {
                    let res = self.cur().and_then(|t| {
                        t.update_name_surname(&real_id(&id), Name::parse(&nm)?, Surname::parse(&sn)?)
                    });
                    self.commit(res);
                },
                RotateKey(id: String, k: Key) => {
                    let nk = real_member_key(&k);
                    let res = self.cur().and_then(|t| t.rotate_p2p_key(&real_id(&id), nk));
                    self.commit(res);
                },
                AddDevice(id: String, k: Key) => {
                    let d = real_device_key(&k);
                    let res = self.cur().and_then(|t| t.add_p2p_device(&real_id(&id), d));
                    self.commit(res);
                },
                DeleteDevice(id: String, d: Key, k: Key) => {
                    let d = real_device_key(&d);
                    let nk = real_member_key(&k);
                    let res = self.cur().and_then(|t| t.delete_p2p_device(&real_id(&id), &d, nk));
                    self.commit(res);
                },
                Isolate(id: String, k: Key) => {
                    let nk = real_member_key(&k);
                    let res = self.cur().and_then(|t| t.emergency_isolate_member(&real_id(&id), nk));
                    self.commit(res);
                },
                ApplyDelta(kind: String, ops: Vec<Op>, removed: BTreeSet<String>, upserted: BTreeSet<Leaf>, target: String) => {
                    let res = self.apply_delta_step(&kind, &ops, &removed, &upserted, &target);
                    self.commit(res);
                }
            })
        })();
        switch_result?;

        if let Some(cov) = &self.coverage {
            let outcome = if self.last_error.is_empty() {
                "Ok".to_string()
            } else {
                self.last_error.clone()
            };
            *cov.lock()
                .expect("coverage lock")
                .entry((step.action_taken.clone(), outcome))
                .or_insert(0) += 1;
        }

        // Root-hash equality-class check: equal model state <=> equal real root.
        if self.last_error.is_empty() {
            if let Some(t) = &self.trie {
                let root = t.root_hash().map_err(|e| anyhow!("root_hash: {e:?}"))?;
                let root_hex = hex::encode(root.as_bytes());
                let model_state = serde_json::to_vec(&self.model_trie()?).unwrap_or_default();
                match self.root_classes.get(&model_state) {
                    Some(prev) if *prev != root_hex => {
                        return Err(anyhow!(
                            "abstraction violated: equal model state, different roots"
                        ))
                    }
                    None => {
                        if self.root_classes.values().any(|v| v == &root_hex) {
                            return Err(anyhow!(
                                "abstraction violated: distinct model states share a root"
                            ));
                        }
                        self.root_classes.insert(model_state, root_hex);
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}

/// Decision 9: name the cause when the Quint toolchain is unusable, rather
/// than letting quint-connect panic with "Failed to execute Quint command".
fn quint_preflight() -> core::result::Result<(), String> {
    let out = std::process::Command::new("quint")
        .arg("--version")
        .output()
        .map_err(|e| format!("`quint` is not runnable from PATH ({e}); install it (npm i -g @informalsystems/quint)"))?;
    if !out.status.success() {
        return Err(format!(
            "`quint --version` failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    // quint fetches its evaluator into `$QUINT_HOME`, else `~/.quint`
    // (quint's own `config.js`), so check the directory quint will use.
    let dir = match std::env::var_os("QUINT_HOME") {
        Some(q) => std::path::PathBuf::from(q),
        None => std::path::Path::new(
            &std::env::var_os("HOME").ok_or("HOME is not set; quint's rust backend needs ~/.quint")?,
        )
        .join(".quint"),
    };
    std::fs::create_dir_all(&dir)
        .and_then(|_| probe_writable(&dir))
        .map_err(|e| {
            format!(
                "{} is not writable ({e}); quint's rust backend fetches its evaluator there",
                dir.display()
            )
        })
}

/// Write and remove a probe file. The name is unique per process and thread:
/// the scenario tests run in parallel, and a shared name lets one test delete
/// another's probe mid-check (measured: a spurious preflight failure).
fn probe_writable(dir: &std::path::Path) -> std::io::Result<()> {
    let p = dir.join(format!(
        ".org-members-preflight-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::write(&p, b"")?;
    std::fs::remove_file(&p)
}

fn require_quint() {
    if let Err(cause) = quint_preflight() {
        panic!("conformance test cannot run: {cause}");
    }
}

fn run_conformance(driver: MembershipDriver, name: &str) {
    require_quint();
    let config = runner::Config {
        test_name: name.to_string(),
        gen_config: RunConfig {
            spec: SPEC.to_string(),
            main: None,
            init: None,
            step: None,
            max_samples: Some(MAX_SAMPLES),
            max_steps: Some(MAX_STEPS),
            seed: runner::gen_random_seed(),
        },
    };
    if let Err(err) = runner::run_test(driver, config) {
        panic!("{err}");
    }
}

fn run_scenario(test: &str) {
    require_quint();
    let config = runner::Config {
        test_name: test.to_string(),
        gen_config: TestConfig {
            spec: SPEC.to_string(),
            main: None,
            test: test.to_string(),
            max_samples: Some(1),
            seed: runner::gen_random_seed(),
        },
    };
    if let Err(err) = runner::run_test(MembershipDriver::default(), config) {
        panic!("{err}");
    }
}

/// verifies: LLR-fv75ec, LLR-j4d38d, LLR-v3jqau, LLR-s97ywt, LLR-w92psx
/// verifies: LLR-ch2pkw, LLR-4n8zqx, LLR-xmpqn2, LLR-juxk9q
/// verifies: LLR-k89ahd, LLR-v6gfc7, LLR-fym7dy
///
/// LLR-s97ywt and LLR-w92psx are carried for their unchanged-replacement-key
/// refusal (PR-zz4exm): removing the `P2pKeyNotReplaced` guard from either
/// `delete_p2p_device` or `emergency_isolate_member` turns this test red
/// (measured, 2026-10-03, three seeds each). Re-measured against the moved
/// model and this driver (2026-10-03, base merge): red 5 of 5 random seeds
/// under each deletion, "Specification and implementation states diverge" —
/// the random step draws the replacement key's generation from `GENS`
/// (0..3) and every member starts at generation 0, so the unchanged key is
/// reached without a named scenario. LLR-k89ahd likewise: removing the guard
/// from `rotate_p2p_key` turns it red (re-measured against the merged driver,
/// 2026-10-03: red 3 of 3 random seeds, "Specification and implementation
/// states diverge" — the rotation is then refused `DuplicateKey`, the
/// member's own key being held, where the model refuses `P2pKeyNotReplaced`).
///
/// LLR-v6gfc7 and LLR-fym7dy are carried for `DuplicateKey`, re-measured
/// against the merged driver (2026-10-03, three random seeds per mutation):
/// ignoring the key-index refusal in `add_member` — red 1 of 3 (killed by the
/// delta round-trip: `apply_delta` refuses the duplicate the direct path let
/// through; `scenario_add_member_duplicate` is red 3 of 3); in
/// `add_p2p_device` together with `update_leaf`'s re-index — red 3 of 3 (the
/// round-trip); in `rotate_p2p_key` together with `update_leaf`'s re-index —
/// red 3 of 3 (the round-trip; the op-level check alone stays green 3 of 3
/// across the whole suite, the re-index catching every rotation it does); in
/// `delete_p2p_device` alone — red 3 of 3 (divergence); in
/// `emergency_isolate_member` alone — red 2 of 3 (divergence). Not reached:
/// the `genesis` clause of LLR-v6gfc7 (ignoring its refusal leaves the whole
/// suite green 3 of 3; seed keys derive from distinct ids). Not LLR-gjj6bx:
/// ignoring `apply_delta`'s resulting-record refusal leaves this test green
/// 3 of 3; it is carried by `scenario_apply_delta_duplicate`,
/// `scenario_member_key_swap` and the crate's `apply_delta_*` tests in
/// integration_test.rs.
///
/// The random run, with action coverage (decision 8(ii)): every model action
/// must be taken at least once across the run; per-action outcome counts are
/// printed, not gated.
///
/// The second `verifies:` line is measured (2026-10-03, five random seeds per
/// mutation, each red 5 of 5 with "Specification and implementation states
/// diverge"): deleting `genesis`'s `DuplicateId` return, or its handle-collision
/// returns (LLR-ch2pkw — reached now that genesis is the model's `init`); an
/// order-dependent leaf hash in `smt::insert` (LLR-4n8zqx; 3 of the 5 reds were
/// the round-trip `expect` in `commit`, 2 the divergence); deleting the
/// stale-removal, no-op or overlap check (LLR-xmpqn2); deleting the
/// `DuplicateHandle` return in `apply_delta` (LLR-juxk9q's uniqueness half).
/// Not carried here: the skeleton (confusable) clauses of LLR-ch2pkw and
/// LLR-juxk9q — the model's skeleton is its handle (measured: deleting only
/// `genesis`'s `ConfusableHandle` return stays green) — and LLR-xmpqn2's two ordering
/// clauses (declared boundary, `delta_canonicality_fuzz`). Table:
/// `docs/risk/2026-10-03-lawful-change-replicates.md`.
#[test]
fn membership_conformance() {
    let coverage: Coverage = Arc::default();
    run_conformance(
        MembershipDriver::with_coverage(coverage.clone()),
        "membership_conformance",
    );
    let counts = coverage.lock().expect("coverage lock");
    for ((action, outcome), n) in counts.iter() {
        println!("coverage {action} -> {outcome}: {n}");
    }
    let missing: Vec<&str> = ACTIONS
        .iter()
        .copied()
        .filter(|a| !counts.keys().any(|(taken, _)| taken == a))
        .collect();
    assert!(
        missing.is_empty(),
        "model actions never taken in the random run: {missing:?}"
    );
}

/// Decision 8(i): the operation table matches the crate's public API.
#[test]
fn trie_operations_table_matches_source() {
    let mut found = BTreeSet::new();
    for file in ["src/trie.rs", "src/delta.rs"] {
        let src = std::fs::read_to_string(file).expect("read crate source");
        let mut rest = src.as_str();
        while let Some(i) = rest.find("pub fn ") {
            rest = &rest[i + "pub fn ".len()..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            let sig = &rest[..rest.find('{').unwrap_or(rest.len())];
            let ret = sig.rsplit("->").next().unwrap_or("");
            let ret: String = ret.split_whitespace().collect();
            if sig.contains("->")
                && [
                    "Result<Self",
                    "Result<(Self",
                    "Result<CandidateTrie",
                    "Result<OrgTrie",
                ]
                .iter()
                .any(|p| ret.starts_with(p))
            {
                found.insert(name);
            }
        }
    }
    let declared: BTreeSet<String> = TRIE_OPERATIONS
        .iter()
        .map(|(op, _)| op.to_string())
        .chain(BOUNDARY.iter().map(|(op, _)| op.to_string()))
        .collect();
    assert_eq!(
        found, declared,
        "trie-yielding public operations and the TRIE_OPERATIONS/BOUNDARY table disagree"
    );
    for (_, action) in TRIE_OPERATIONS {
        assert!(ACTIONS.contains(&action), "{action} is not a model action");
    }
}

/// Decision 9: the preflight names the cause when quint is missing.
#[test]
fn quint_preflight_names_a_missing_binary() {
    let out = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args([
            "--exact",
            "preflight_probe",
            "--nocapture",
            "--include-ignored",
        ])
        .env("PATH", "")
        .output()
        .expect("re-run test binary");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("PREFLIGHT: `quint` is not runnable from PATH"),
        "{text}"
    );
}

#[test]
#[ignore = "invoked by quint_preflight_names_a_missing_binary with PATH emptied"]
fn preflight_probe() {
    match quint_preflight() {
        Ok(()) => println!("PREFLIGHT: ok"),
        Err(cause) => println!("PREFLIGHT: {cause}"),
    }
}

// ---- Named scenarios (decision 12), one per `run scenario*` in the model.

#[test]
fn scenario_genesis() {
    run_scenario("scenarioGenesis")
}
#[test]
fn scenario_genesis_device_error_first() {
    run_scenario("scenarioGenesisDeviceErrorFirst")
}
#[test]
fn scenario_add_member() {
    run_scenario("scenarioAddMember")
}
#[test]
fn scenario_delete_member() {
    run_scenario("scenarioDeleteMember")
}
#[test]
fn scenario_update_handle() {
    run_scenario("scenarioUpdateHandle")
}
#[test]
fn scenario_update_name_surname() {
    run_scenario("scenarioUpdateNameSurname")
}
#[test]
fn scenario_rotate_key() {
    run_scenario("scenarioRotateKey")
}
#[test]
fn scenario_add_device() {
    run_scenario("scenarioAddDevice")
}
#[test]
fn scenario_delete_device() {
    run_scenario("scenarioDeleteDevice")
}
#[test]
fn scenario_isolate() {
    run_scenario("scenarioIsolate")
}
#[test]
fn scenario_apply_honest() {
    run_scenario("scenarioApplyHonest")
}
/// verifies: LLR-au8het
///
/// Measured: red with the `base_root != current_root` refusal deleted.
#[test]
fn scenario_apply_stale() {
    run_scenario("scenarioApplyStale")
}
/// verifies: LLR-7tdqv9
///
/// Measured: red with `verify_against`'s root comparison made to pass.
#[test]
fn scenario_apply_verify_fail() {
    run_scenario("scenarioApplyVerifyFail")
}
/// verifies: REQ-wx3wpv, LLR-n5t6bn
///
/// PR-vf5hdm: a handle moved between two members who are both still present,
/// in one honest delta produced by `recalculate()`.
#[test]
fn scenario_handover_a_to_b() {
    run_scenario("scenarioHandoverAtoB")
}
/// verifies: REQ-wx3wpv, LLR-n5t6bn
///
/// PR-vf5hdm: the same handover in the other identifier order.
#[test]
fn scenario_handover_b_to_a() {
    run_scenario("scenarioHandoverBtoA")
}
/// verifies: REQ-wx3wpv, LLR-n5t6bn
///
/// PR-vf5hdm: two present members swap handles in one honest delta.
#[test]
fn scenario_handle_swap() {
    run_scenario("scenarioHandleSwap")
}
/// verifies: REQ-wx3wpv, LLR-n5t6bn
///
/// PR-vf5hdm: a handle moved off a present member to a member the same honest
/// delta admits.
#[test]
fn scenario_handover_to_new_member() {
    run_scenario("scenarioHandoverToNewMember")
}
/// verifies: LLR-gjj6bx
///
/// Two present members swap member keys in one honest delta (rotate a to an
/// unheld key, b to a's old key, a to b's old key): the change set the
/// software produced is accepted, the key check judging the resulting record
/// (LLR-gjj6bx). Measured (2026-10-03): red 3 of 3 with `delta_key_index`
/// made one-phase (each upsert's old keys released and its new keys indexed
/// in one pass), "Specification and implementation states diverge".
#[test]
fn scenario_member_key_swap() {
    run_scenario("scenarioMemberKeySwap")
}
/// A device key moves from one present member to another in one change set
/// (LLR-gjj6bx). No `verifies:`: measured green 3 of 3 (2026-10-03) under the
/// one-phase `delta_key_index` that reddens `scenario_member_key_swap`
/// (not traced; presumably the giving member is upserted before the receiving
/// one in identifier order, so a one-pass check has released the key already).
#[test]
fn scenario_device_key_moves() {
    run_scenario("scenarioDeviceKeyMoves")
}
/// verifies: LLR-fym7dy
///
/// Measured (2026-10-03): red 3 of 3 with `rotate_p2p_key`'s held-key refusal
/// and `update_leaf`'s re-index refusal both ignored.
#[test]
fn scenario_rotate_key_duplicate() {
    run_scenario("scenarioRotateKeyDuplicate")
}
/// verifies: LLR-v6gfc7
///
/// Measured (2026-10-03): red 3 of 3 with `add_p2p_device`'s held-key refusal
/// and `update_leaf`'s re-index refusal both ignored.
#[test]
fn scenario_add_device_duplicate() {
    run_scenario("scenarioAddDeviceDuplicate")
}
/// verifies: LLR-fym7dy
///
/// Measured (2026-10-03): red 3 of 3 with `delete_p2p_device`'s held-key
/// refusal deleted.
#[test]
fn scenario_delete_device_removed_key() {
    run_scenario("scenarioDeleteDeviceRemovedKey")
}
/// verifies: LLR-fym7dy
///
/// Measured (2026-10-03): red 3 of 3 with `emergency_isolate_member`'s
/// held-key refusal deleted.
#[test]
fn scenario_isolate_duplicate() {
    run_scenario("scenarioIsolateDuplicate")
}
/// verifies: LLR-v6gfc7
///
/// Measured (2026-10-03): red 3 of 3 with `add_member`'s key-index refusal
/// ignored.
#[test]
fn scenario_add_member_duplicate() {
    run_scenario("scenarioAddMemberDuplicate")
}
/// verifies: LLR-gjj6bx
///
/// Measured (2026-10-03): red 3 of 3 with `apply_delta`'s resulting-record
/// key refusal ignored.
#[test]
fn scenario_apply_delta_duplicate() {
    run_scenario("scenarioApplyDeltaDuplicate")
}
#[test]
fn scenario_device_slots_full() {
    run_scenario("scenarioDeviceSlotsFull")
}
/// verifies: LLR-xmpqn2
///
/// Measured: red with the remove/upsert overlap check deleted (green under
/// the stale-removal and no-op deletions, which it does not reach).
#[test]
fn scenario_remove_upsert_overlap() {
    run_scenario("scenarioRemoveUpsertOverlap")
}
/// verifies: LLR-xmpqn2
///
/// Measured: red with the no-op upsert check deleted (green under the overlap
/// deletion, which the no-op check precedes).
#[test]
fn scenario_no_op_before_overlap() {
    run_scenario("scenarioNoOpBeforeOverlap")
}
/// verifies: LLR-7tdqv9
///
/// Measured: red with `verify_against`'s root comparison made to pass.
#[test]
fn scenario_built_verify_fail() {
    run_scenario("scenarioBuiltVerifyFail")
}

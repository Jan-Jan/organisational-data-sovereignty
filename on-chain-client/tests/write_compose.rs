#![cfg(all(feature = "test-support", feature = "write"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The writer's composition (LLR-a2acvh, LLR-z9vugt, LLR-kv27gp, LLR-hun4wf),
//! driven through a substitute `WriteOps` that records each step and can fail
//! or answer `ApprovalRecorded` at any of them.

use std::sync::Mutex;

use on_chain_client::write::calldata::revive_update_runtime_call;
use on_chain_client::write::proxy::{map_account_call, proxied};
use on_chain_client::write::{
    genesis, submit_update, AccountId, DispatchOutcome, Genesis, GenesisStep, WriteError, WriteOps, WriteWindow,
    FUND_AMOUNT,
};
use on_chain_client::{h160_of, Epoch, OnChainRootHash, OrgAdmin, OrgPubKey};
use subxt::dynamic::Value;

#[derive(Clone, Debug, PartialEq)]
enum Step {
    CreatePure(Vec<AccountId>),
    Fund(AccountId, u128),
    Dispatch(Vec<AccountId>, Value, Option<WriteWindow>),
}

/// What the substitute does at step `n` (0-based, in the order asked).
#[derive(Clone, Copy, PartialEq)]
enum Answer {
    Ok,
    Fail,
    Approval,
}

struct FakeOps {
    steps: Mutex<Vec<Step>>,
    answers: Vec<Answer>,
}

const PROXY: AccountId = AccountId([0x5a; 32]);

/// The pure proxy the substitute chain creates at its `n`th `create_pure`
/// (0-based): `PROXY`, then a distinct account each time.
fn nth_proxy(n: usize) -> AccountId {
    AccountId([0x5a + u8::try_from(n).unwrap(); 32])
}

impl FakeOps {
    fn answering(answers: &[Answer]) -> Self {
        Self { steps: Mutex::new(vec![]), answers: answers.to_vec() }
    }
    fn record(&self, s: Step) -> Answer {
        let mut steps = self.steps.lock().unwrap();
        steps.push(s);
        *self.answers.get(steps.len() - 1).unwrap_or(&Answer::Ok)
    }
    fn steps(&self) -> Vec<Step> {
        self.steps.lock().unwrap().clone()
    }
}

impl WriteOps for FakeOps {
    type Signer = ();
    /// The chain's answer: a new pure proxy per call, `PROXY` the first.
    async fn create_pure(&self, _: &(), co: &[AccountId]) -> Result<AccountId, WriteError> {
        match self.record(Step::CreatePure(co.to_vec())) {
            Answer::Fail => Err(WriteError::Subxt("create_pure refused".into())),
            _ => Ok(nth_proxy(self.steps().iter().filter(|s| matches!(s, Step::CreatePure(_))).count() - 1)),
        }
    }
    async fn fund(&self, _: &(), dest: AccountId, amount: u128) -> Result<(), WriteError> {
        match self.record(Step::Fund(dest, amount)) {
            Answer::Fail => Err(WriteError::Subxt("transfer refused".into())),
            _ => Ok(()),
        }
    }
    async fn dispatch(
        &self,
        _: &(),
        co: &[AccountId],
        call: Value,
        window: Option<WriteWindow>,
    ) -> Result<DispatchOutcome, WriteError> {
        match self.record(Step::Dispatch(co.to_vec(), call, window)) {
            Answer::Fail => Err(WriteError::Subxt("dispatch refused".into())),
            Answer::Approval => Ok(DispatchOutcome::ApprovalRecorded),
            Answer::Ok => Ok(DispatchOutcome::Executed),
        }
    }
}

const CONTRACT: [u8; 20] = [0xc0; 20];
fn co() -> Vec<AccountId> {
    vec![AccountId([2; 32]), AccountId([1; 32])]
}
fn root() -> OnChainRootHash {
    OnChainRootHash([0x33; 32])
}
fn key() -> OrgPubKey {
    OrgPubKey([0x22; 32])
}
fn window() -> WriteWindow {
    WriteWindow { birth_number: 4_242, birth_hash: [0x77; 32] }
}

// Normal: the four steps, in order, each with its exact call, and the result.
// verifies: LLR-a2acvh, REQ-6jefu2
#[tokio::test]
async fn genesis_creates_funds_maps_and_records_in_that_order() {
    let ops = FakeOps::answering(&[]);
    let out = genesis(&ops, &(), &co(), CONTRACT, root(), key()).await.unwrap();
    assert_eq!(out, Genesis { proxy: PROXY, admin: OrgAdmin(h160_of(PROXY.0)) });
    assert_eq!(
        ops.steps(),
        vec![
            Step::CreatePure(co()),
            Step::Fund(PROXY, FUND_AMOUNT),
            Step::Dispatch(co(), proxied(PROXY, map_account_call()), None),
            Step::Dispatch(co(), proxied(PROXY, revive_update_runtime_call(CONTRACT, root(), key(), Epoch(0))), None),
        ]
    );
}

// Each founding creates its own pure proxy: two ceremonies each start with
// `create_pure`, each funds, maps and records through the proxy it created,
// never an earlier founding's, and the two Organisations differ. Owner ruling
// of 2026-10-13 (review round 1, finding-2, of change worktree-org-io-publish):
// a timed-out founding that lands later founds a separate Organisation under
// its own new proxy, so it cannot collide with the committed one.
// verifies: LLR-a2acvh, REQ-6jefu2
#[tokio::test]
async fn each_genesis_creates_a_new_pure_proxy_and_writes_through_it() {
    let ops = FakeOps::answering(&[]);
    let first = genesis(&ops, &(), &co(), CONTRACT, root(), key()).await.unwrap();
    let second = genesis(&ops, &(), &co(), CONTRACT, root(), key()).await.unwrap();
    let (p1, p2) = (nth_proxy(0), nth_proxy(1));
    assert_eq!(first, Genesis { proxy: p1, admin: OrgAdmin(h160_of(p1.0)) });
    assert_eq!(second, Genesis { proxy: p2, admin: OrgAdmin(h160_of(p2.0)) });
    assert_ne!(first.admin, second.admin, "two Organisations");
    let ceremony = |proxy: AccountId| {
        vec![
            Step::CreatePure(co()),
            Step::Fund(proxy, FUND_AMOUNT),
            Step::Dispatch(co(), proxied(proxy, map_account_call()), None),
            Step::Dispatch(co(), proxied(proxy, revive_update_runtime_call(CONTRACT, root(), key(), Epoch(0))), None),
        ]
    };
    assert_eq!(ops.steps(), [ceremony(p1), ceremony(p2)].concat());
}

// Abnormal: a failure at each step is reported as that step, and no later
// step is attempted.
// verifies: LLR-z9vugt, REQ-6jefu2
#[tokio::test]
async fn a_failed_genesis_step_is_named_and_stops_the_ceremony() {
    let names = [GenesisStep::CreatePure, GenesisStep::Fund, GenesisStep::MapAccount, GenesisStep::RecordGenesis];
    for (i, step) in names.into_iter().enumerate() {
        let mut answers = vec![Answer::Ok; 4];
        answers[i] = Answer::Fail;
        let ops = FakeOps::answering(&answers);
        let err = genesis(&ops, &(), &co(), CONTRACT, root(), key()).await.unwrap_err();
        assert!(matches!(&err, WriteError::Step { step: s, .. } if *s == step), "step {i}: {err:?}");
        assert_eq!(ops.steps().len(), i + 1, "no step after {step:?} may run");
    }
}

// Abnormal: a dispatched genesis step that only records an approval is a
// failure of that step, not a success.
// verifies: LLR-z9vugt
#[tokio::test]
async fn an_approval_without_execution_fails_the_genesis_step() {
    for (i, step) in [(2usize, GenesisStep::MapAccount), (3, GenesisStep::RecordGenesis)] {
        let mut answers = vec![Answer::Ok; 4];
        answers[i] = Answer::Approval;
        let ops = FakeOps::answering(&answers);
        let err = genesis(&ops, &(), &co(), CONTRACT, root(), key()).await.unwrap_err();
        assert!(matches!(&err, WriteError::Step { step: s, .. } if *s == step), "{err:?}");
        assert_eq!(ops.steps().len(), i + 1);
    }
}

// Normal: exactly one dispatch, the proxied update at the expected epoch.
// verifies: LLR-kv27gp, REQ-aat4yt
#[tokio::test]
async fn submit_update_dispatches_one_proxied_update_and_succeeds_when_it_executes() {
    let ops = FakeOps::answering(&[]);
    submit_update(&ops, &(), &co(), CONTRACT, PROXY, root(), key(), Epoch(4), window()).await.unwrap();
    assert_eq!(
        ops.steps(),
        vec![Step::Dispatch(
            co(),
            proxied(PROXY, revive_update_runtime_call(CONTRACT, root(), key(), Epoch(4))),
            Some(window())
        )]
    );
}

// Abnormal: an approval that did not execute is PendingApproval; a failed
// dispatch returns the dispatch's own error.
// verifies: LLR-hun4wf, REQ-aat4yt
#[tokio::test]
async fn submit_update_reports_a_pending_approval_and_a_failed_dispatch() {
    let pending = FakeOps::answering(&[Answer::Approval]);
    assert_eq!(
        submit_update(&pending, &(), &co(), CONTRACT, PROXY, root(), key(), Epoch(4), window()).await.unwrap_err(),
        WriteError::PendingApproval
    );
    let failed = FakeOps::answering(&[Answer::Fail]);
    assert_eq!(
        submit_update(&failed, &(), &co(), CONTRACT, PROXY, root(), key(), Epoch(4), window()).await.unwrap_err(),
        WriteError::Subxt("dispatch refused".into())
    );
}

// Normal: the window the caller read reaches the one dispatch unchanged, so
// the update is signed mortal from that block (LLR-zj4jju); two windows give
// two dispatches that differ only in it.
// verifies: LLR-zj4jju
#[tokio::test]
async fn submit_update_dispatches_with_the_window_it_was_given() {
    let ops = FakeOps::answering(&[]);
    let later = WriteWindow { birth_number: u64::MAX, birth_hash: [0xee; 32] };
    submit_update(&ops, &(), &co(), CONTRACT, PROXY, root(), key(), Epoch(4), window()).await.unwrap();
    submit_update(&ops, &(), &co(), CONTRACT, PROXY, root(), key(), Epoch(4), later).await.unwrap();
    let windows: Vec<Option<WriteWindow>> = ops
        .steps()
        .into_iter()
        .map(|step| match step {
            Step::Dispatch(_, _, window) => window,
            other => panic!("not a dispatch: {other:?}"),
        })
        .collect();
    assert_eq!(windows, vec![Some(window()), Some(later)]);
}

// Abnormal: genesis is not settled by a window; each of its dispatches
// carries none, so they are signed with subxt's defaults (LLR-zj4jju).
// verifies: LLR-zj4jju
#[tokio::test]
async fn genesis_dispatches_with_no_window() {
    let ops = FakeOps::answering(&[]);
    genesis(&ops, &(), &co(), CONTRACT, root(), key()).await.unwrap();
    let dispatches: Vec<Option<WriteWindow>> = ops
        .steps()
        .into_iter()
        .filter_map(|step| match step {
            Step::Dispatch(_, _, window) => Some(window),
            _ => None,
        })
        .collect();
    assert_eq!(dispatches, vec![None, None]);
}

// Abnormal: a mortal dispatch that is refused, or only recorded as a pending
// approval, is reported as such and never retried — in particular not
// unwindowed, which would sign an immortal write the caller's window could
// not settle. One dispatch is made, carrying the window.
// verifies: LLR-zj4jju
#[tokio::test]
async fn a_refused_mortal_dispatch_is_reported_and_not_retried_without_its_window() {
    for (answer, expected) in [
        (Answer::Fail, WriteError::Subxt("dispatch refused".into())),
        (Answer::Approval, WriteError::PendingApproval),
    ] {
        let ops = FakeOps::answering(&[answer]);
        let err = submit_update(&ops, &(), &co(), CONTRACT, PROXY, root(), key(), Epoch(4), window()).await.unwrap_err();
        assert_eq!(err, expected);
        let windows: Vec<Option<WriteWindow>> = ops
            .steps()
            .into_iter()
            .map(|step| match step {
                Step::Dispatch(_, _, window) => window,
                other => panic!("not a dispatch: {other:?}"),
            })
            .collect();
        assert_eq!(windows, vec![Some(window())], "{expected:?}: one dispatch, with the window");
    }
}

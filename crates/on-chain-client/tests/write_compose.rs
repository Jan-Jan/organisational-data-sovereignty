#![cfg(all(feature = "test-support", feature = "write"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The writer's composition (LLR-tge9q8, LLR-q6buam, LLR-chy957, LLR-hea6y2,
//! LLR-kv27gp, LLR-hun4wf), driven through a substitute `WriteOps` that
//! records each step — reads beside writes — answers its two reads from
//! values the test sets, and can fail or answer `ApprovalRecorded` at any
//! step.

use std::sync::Mutex;

use on_chain_client::write::calldata::revive_update_runtime_call;
use on_chain_client::write::proxy::{map_account_call, proxied};
use on_chain_client::write::{
    create_proxy, found_at, genesis, submit_update, AccountId, DispatchOutcome, Genesis, GenesisStep, WriteError,
    WriteOps, WriteWindow, FUND_AMOUNT,
};
use on_chain_client::{h160_of, Epoch, OnChainRootHash, OrgAdmin, OrgPubKey};
use subxt::dynamic::Value;

#[path = "support/write_reads_contract.rs"]
mod contract;

#[derive(Clone, Debug, PartialEq)]
enum Step {
    CreatePure(Vec<AccountId>),
    Fund(AccountId, u128),
    Dispatch(Vec<AccountId>, Value, Option<WriteWindow>),
    /// A read of the account an H160 is mapped to.
    MappedAccount(OrgAdmin),
    /// A read of an account's free balance.
    FreeBalance(AccountId),
}

/// What the substitute does at step `n` (0-based, in the order asked).
#[derive(Clone, Copy, PartialEq)]
enum Answer {
    Ok,
    Fail,
    Approval,
}

/// The substitute chain: every call it is asked for, in order, in one step
/// log (reads beside writes), the answer for each step by its position, and
/// what its two reads answer when they do not fail — by default an unmapped
/// proxy with nothing in it.
struct FakeOps {
    steps: Mutex<Vec<Step>>,
    answers: Vec<Answer>,
    mapped: Option<AccountId>,
    balance: u128,
}

const PROXY: AccountId = AccountId([0x5a; 32]);

/// The pure proxy the substitute chain creates at its `n`th `create_pure`
/// (0-based): `PROXY`, then a distinct account each time.
fn nth_proxy(n: usize) -> AccountId {
    AccountId([0x5a + u8::try_from(n).unwrap(); 32])
}

impl FakeOps {
    fn answering(answers: &[Answer]) -> Self {
        Self { steps: Mutex::new(vec![]), answers: answers.to_vec(), mapped: None, balance: 0 }
    }
    /// The chain's state as the two reads show it: the account the proxy's
    /// H160 maps to, and the proxy's free balance.
    fn on_chain(self, mapped: Option<AccountId>, balance: u128) -> Self {
        Self { mapped, balance, ..self }
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
            _ => Ok(nth_proxy(self.steps().iter().filter(|step| matches!(step, Step::CreatePure(_))).count() - 1)),
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
    async fn free_balance(&self, account: AccountId) -> Result<u128, WriteError> {
        match self.record(Step::FreeBalance(account)) {
            Answer::Fail => Err(WriteError::Subxt("balance read refused".into())),
            _ => Ok(self.balance),
        }
    }
    async fn mapped_account(&self, admin: OrgAdmin) -> Result<Option<AccountId>, WriteError> {
        match self.record(Step::MappedAccount(admin)) {
            Answer::Fail => Err(WriteError::Subxt("mapping read refused".into())),
            _ => Ok(self.mapped),
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

// The admin, and the Organisation's on-chain id, of a proxy.
use contract::admin_of;

/// The record dispatch of a founding at `proxy`: the genesis root and key at
/// an expected epoch of zero, through the proxy, with no window.
fn record_step(proxy: AccountId) -> Step {
    Step::Dispatch(co(), proxied(proxy, revive_update_runtime_call(CONTRACT, root(), key(), Epoch(0))), None)
}

/// The seam calls of a founding at a proxy the reads show neither mapped nor
/// funded (LLR-q6buam): the two reads, then fund, map and record.
fn fresh_founding(proxy: AccountId) -> Vec<Step> {
    vec![
        Step::MappedAccount(admin_of(proxy)),
        Step::FreeBalance(proxy),
        Step::Fund(proxy, FUND_AMOUNT),
        Step::Dispatch(co(), proxied(proxy, map_account_call()), None),
        record_step(proxy),
    ]
}

// Normal: one `create_pure` call under the co-signatories given (none for a
// signatory alone), and the account it answers is returned.
// verifies: LLR-tge9q8
#[tokio::test]
async fn create_proxy_makes_one_create_pure_call_and_returns_its_account() {
    let ops = FakeOps::answering(&[]);
    assert_eq!(create_proxy(&ops, &(), &co()).await, Ok(PROXY));
    assert_eq!(ops.steps(), vec![Step::CreatePure(co())]);

    let alone = FakeOps::answering(&[]);
    assert_eq!(create_proxy(&alone, &(), &[]).await, Ok(PROXY));
    assert_eq!(alone.steps(), vec![Step::CreatePure(vec![])]);
}

// Abnormal: a refused `create_pure` is the `CreatePure` step, its reason the
// seam error's text, and nothing else is asked of the chain.
// verifies: LLR-tge9q8, LLR-chy957
#[tokio::test]
async fn a_failed_create_pure_is_the_create_pure_step() {
    let ops = FakeOps::answering(&[Answer::Fail]);
    assert_eq!(
        create_proxy(&ops, &(), &co()).await,
        Err(WriteError::Step {
            step: GenesisStep::CreatePure,
            reason: WriteError::Subxt("create_pure refused".into()).to_string()
        })
    );
    assert_eq!(ops.steps(), vec![Step::CreatePure(co())]);
}

// Normal: at a proxy the reads show neither mapped nor funded, the founding
// reads the mapping, then the balance, then funds, maps and records at
// expected epoch zero — at the proxy it was given, not one it made — and
// returns that proxy and its admin.
// verifies: LLR-q6buam
#[tokio::test]
async fn found_at_on_a_fresh_proxy_reads_then_funds_maps_and_records_in_order() {
    let given = AccountId([0x99; 32]);
    let ops = FakeOps::answering(&[]);
    let out = found_at(&ops, &(), &co(), CONTRACT, given, root(), key()).await.unwrap();
    assert_eq!(out, Genesis { proxy: given, admin: admin_of(given) });
    assert_eq!(ops.steps(), fresh_founding(given));
}

// Normal: whatever the chain shows and wherever a step fails, the founding
// step never creates a proxy.
// verifies: LLR-q6buam
#[tokio::test]
async fn found_at_never_calls_create_pure() {
    let states = [(None, 0), (None, FUND_AMOUNT), (Some(PROXY), 0), (Some(PROXY), FUND_AMOUNT)];
    for (mapped, balance) in states {
        for failing in 0..=5 {
            let mut answers = vec![Answer::Ok; 6];
            answers[failing] = Answer::Fail;
            let ops = FakeOps::answering(&answers).on_chain(mapped, balance);
            // The outcome is the other tests' to check, for each state and
            // failing step; this one asks only which calls were made.
            let _ = found_at(&ops, &(), &co(), CONTRACT, PROXY, root(), key()).await;
            assert!(
                !ops.steps().iter().any(|step| matches!(step, Step::CreatePure(_))),
                "{mapped:?}, {balance}, failing at {failing}: {:?}",
                ops.steps()
            );
        }
    }
}

// Abnormal: at a fresh proxy, a failure at each seam call is that call's
// step — the mapping read `MapAccount`, the balance read and the transfer
// `Fund`, the mapping dispatch `MapAccount`, the record `RecordGenesis` — and
// no later call is made. A dispatch that only records an approval fails its
// step the same way.
// verifies: LLR-chy957
#[tokio::test]
async fn each_failing_step_is_named_and_no_later_call_is_made() {
    let names = [
        GenesisStep::MapAccount,
        GenesisStep::Fund,
        GenesisStep::Fund,
        GenesisStep::MapAccount,
        GenesisStep::RecordGenesis,
    ];
    for (index, step) in names.into_iter().enumerate() {
        let mut answers = vec![Answer::Ok; 5];
        answers[index] = Answer::Fail;
        let ops = FakeOps::answering(&answers);
        let err = found_at(&ops, &(), &co(), CONTRACT, PROXY, root(), key()).await.unwrap_err();
        assert!(matches!(&err, WriteError::Step { step: failed_step, .. } if *failed_step == step), "call {index}: {err:?}");
        assert_eq!(ops.steps(), fresh_founding(PROXY)[..=index].to_vec(), "no call after call {index}");
    }
    for (index, step) in [(3usize, GenesisStep::MapAccount), (4, GenesisStep::RecordGenesis)] {
        let mut answers = vec![Answer::Ok; 5];
        answers[index] = Answer::Approval;
        let ops = FakeOps::answering(&answers);
        let err = found_at(&ops, &(), &co(), CONTRACT, PROXY, root(), key()).await.unwrap_err();
        assert!(matches!(&err, WriteError::Step { step: failed_step, .. } if *failed_step == step), "approval at {index}: {err:?}");
        assert_eq!(ops.steps().len(), index + 1);
    }
}

// Abnormal: a proxy whose H160 the chain maps to another account cannot be
// founded at: the mapping read's answer is the `MapAccount` step, and nothing
// is read, funded, mapped or recorded after it.
// verifies: LLR-chy957
#[tokio::test]
async fn a_mapping_read_holding_another_account_is_the_map_account_step() {
    let other = AccountId([0x0b; 32]);
    for balance in [0, FUND_AMOUNT] {
        let ops = FakeOps::answering(&[]).on_chain(Some(other), balance);
        let err = found_at(&ops, &(), &co(), CONTRACT, PROXY, root(), key()).await.unwrap_err();
        assert!(matches!(&err, WriteError::Step { step: GenesisStep::MapAccount, .. }), "{err:?}");
        assert_eq!(ops.steps(), vec![Step::MappedAccount(admin_of(PROXY))]);
    }
}

// Normal (resume): a proxy the chain shows mapped to itself was funded
// before it was mapped, and mapping took a deposit from its balance, so
// whatever the balance is — none, below or at the fund amount — it is
// neither read nor topped up.
// verifies: LLR-jk76wf
#[tokio::test]
async fn a_mapped_proxy_is_not_funded_and_its_balance_is_not_read() {
    for balance in [0, FUND_AMOUNT - 1, FUND_AMOUNT] {
        let ops = FakeOps::answering(&[]).on_chain(Some(PROXY), balance);
        found_at(&ops, &(), &co(), CONTRACT, PROXY, root(), key()).await.unwrap();
        let steps = ops.steps();
        assert!(!steps.iter().any(|step| matches!(step, Step::FreeBalance(_))), "{balance}: {steps:?}");
        assert!(!steps.iter().any(|step| matches!(step, Step::Fund(..))), "{balance}: {steps:?}");
    }
}

// Normal (resume): a founding interrupted between funding and mapping leaves
// an unmapped proxy holding at least the fund amount; the re-run reads both
// values, does not fund again, and goes on to map and record.
// verifies: LLR-jk76wf
#[tokio::test]
async fn an_unmapped_proxy_holding_the_fund_amount_is_not_funded_again() {
    for balance in [FUND_AMOUNT, FUND_AMOUNT + 1, u128::MAX] {
        let ops = FakeOps::answering(&[]).on_chain(None, balance);
        found_at(&ops, &(), &co(), CONTRACT, PROXY, root(), key()).await.unwrap();
        assert_eq!(
            ops.steps(),
            vec![
                Step::MappedAccount(admin_of(PROXY)),
                Step::FreeBalance(PROXY),
                Step::Dispatch(co(), proxied(PROXY, map_account_call()), None),
                record_step(PROXY),
            ],
            "{balance}"
        );
    }
}

// Abnormal (boundary): dust anyone may send to an unmapped proxy — one unit,
// or one short of the fund amount — is not taken as the funding: the proxy
// is funded with the whole fund amount, then mapped and recorded.
// verifies: LLR-jk76wf
#[tokio::test]
async fn dust_below_the_fund_amount_is_not_taken_as_funded() {
    for balance in [1, FUND_AMOUNT - 1] {
        let ops = FakeOps::answering(&[]).on_chain(None, balance);
        found_at(&ops, &(), &co(), CONTRACT, PROXY, root(), key()).await.unwrap();
        assert_eq!(ops.steps(), fresh_founding(PROXY), "{balance}");
    }
}

// Normal (resume): a proxy the chain shows mapped to itself is not mapped
// again; the founding goes straight from the mapping read to the record. An
// unmapped one is mapped once, after the funding step.
// verifies: LLR-95puh9
#[tokio::test]
async fn a_mapped_proxy_is_not_mapped_again() {
    let ops = FakeOps::answering(&[]).on_chain(Some(PROXY), FUND_AMOUNT);
    found_at(&ops, &(), &co(), CONTRACT, PROXY, root(), key()).await.unwrap();
    assert_eq!(ops.steps(), vec![Step::MappedAccount(admin_of(PROXY)), record_step(PROXY)]);

    let unmapped = FakeOps::answering(&[]).on_chain(None, FUND_AMOUNT);
    found_at(&unmapped, &(), &co(), CONTRACT, PROXY, root(), key()).await.unwrap();
    let mappings = unmapped
        .steps()
        .into_iter()
        .filter(|step| *step == Step::Dispatch(co(), proxied(PROXY, map_account_call()), None))
        .count();
    assert_eq!(mappings, 1);
}

// Normal: whatever the reads show, the record step is dispatched — once,
// last, with an expected epoch of zero — and the founding succeeds when it
// executes; no read of the Organisation's slot stands in for it.
// verifies: LLR-g8qevz
#[tokio::test]
async fn the_record_step_is_always_dispatched_at_expected_epoch_zero() {
    let states = [(None, 0), (None, 1), (None, FUND_AMOUNT), (Some(PROXY), 0), (Some(PROXY), FUND_AMOUNT)];
    for (mapped, balance) in states {
        let ops = FakeOps::answering(&[]).on_chain(mapped, balance);
        let out = found_at(&ops, &(), &co(), CONTRACT, PROXY, root(), key()).await.unwrap();
        assert_eq!(out, Genesis { proxy: PROXY, admin: admin_of(PROXY) });
        let steps = ops.steps();
        assert_eq!(steps.last(), Some(&record_step(PROXY)), "{mapped:?}, {balance}");
        assert_eq!(steps.iter().filter(|step| **step == record_step(PROXY)).count(), 1, "{mapped:?}, {balance}");
    }
}

// Abnormal (resume): on a slot that already holds a genesis, the contract's
// expected-epoch check fails the record's inner call; the founding returns
// the `RecordGenesis` step, never success — whether the proxy was mapped
// before this run or mapped by it. An approval without execution is refused
// the same way.
// verifies: LLR-g8qevz
#[tokio::test]
async fn a_record_refused_on_a_slot_holding_a_genesis_is_the_record_genesis_step() {
    // The record is the second call at a mapped proxy, the fourth at an
    // unmapped one that holds the fund amount.
    for (mapped, record_at) in [(Some(PROXY), 1usize), (None, 3)] {
        for answer in [Answer::Fail, Answer::Approval] {
            let mut answers = vec![Answer::Ok; record_at + 1];
            answers[record_at] = answer;
            let ops = FakeOps::answering(&answers).on_chain(mapped, FUND_AMOUNT);
            let err = found_at(&ops, &(), &co(), CONTRACT, PROXY, root(), key()).await.unwrap_err();
            assert!(matches!(&err, WriteError::Step { step: GenesisStep::RecordGenesis, .. }), "{mapped:?}: {err:?}");
            assert_eq!(ops.steps().last(), Some(&record_step(PROXY)), "{mapped:?}");
        }
    }
}

// Normal: `genesis` is `create_proxy`, then `found_at` at the proxy it
// returned — the same calls and the same result as the two steps made by
// hand — and a failed `create_proxy` is its error with no founding call.
// verifies: LLR-hea6y2
#[tokio::test]
async fn genesis_is_create_proxy_then_found_at_at_its_proxy() {
    let ops = FakeOps::answering(&[]);
    let out = genesis(&ops, &(), &co(), CONTRACT, root(), key()).await.unwrap();
    assert_eq!(ops.steps(), [vec![Step::CreatePure(co())], fresh_founding(PROXY)].concat());

    let by_hand = FakeOps::answering(&[]);
    let proxy = create_proxy(&by_hand, &(), &co()).await.unwrap();
    let founded = found_at(&by_hand, &(), &co(), CONTRACT, proxy, root(), key()).await.unwrap();
    assert_eq!(out, founded);
    assert_eq!(ops.steps(), by_hand.steps());

    let refused = FakeOps::answering(&[Answer::Fail]);
    let err = genesis(&refused, &(), &co(), CONTRACT, root(), key()).await.unwrap_err();
    assert_eq!(err, create_proxy(&FakeOps::answering(&[Answer::Fail]), &(), &co()).await.unwrap_err());
    assert_eq!(refused.steps(), vec![Step::CreatePure(co())]);
}

// Normal: the ceremony's calls, in order, each with its exact call, and the
// result: `create_pure`, the two reads, then fund, map and record.
// verifies: LLR-hea6y2, REQ-6jefu2
#[tokio::test]
async fn genesis_creates_funds_maps_and_records_in_that_order() {
    let ops = FakeOps::answering(&[]);
    let out = genesis(&ops, &(), &co(), CONTRACT, root(), key()).await.unwrap();
    assert_eq!(out, Genesis { proxy: PROXY, admin: OrgAdmin(h160_of(PROXY.0)) });
    assert_eq!(ops.steps(), [vec![Step::CreatePure(co())], fresh_founding(PROXY)].concat());
}

// Each founding creates its own pure proxy: two ceremonies each start with
// `create_pure`, each funds, maps and records through the proxy it created,
// never an earlier founding's, and the two Organisations differ. A timed-out
// founding that lands later founds a separate Organisation under
// its own new proxy, so it cannot collide with the committed one.
// verifies: LLR-hea6y2, REQ-6jefu2
#[tokio::test]
async fn each_genesis_creates_a_new_pure_proxy_and_writes_through_it() {
    let ops = FakeOps::answering(&[]);
    let first = genesis(&ops, &(), &co(), CONTRACT, root(), key()).await.unwrap();
    let second = genesis(&ops, &(), &co(), CONTRACT, root(), key()).await.unwrap();
    let (p1, p2) = (nth_proxy(0), nth_proxy(1));
    assert_eq!(first, Genesis { proxy: p1, admin: OrgAdmin(h160_of(p1.0)) });
    assert_eq!(second, Genesis { proxy: p2, admin: OrgAdmin(h160_of(p2.0)) });
    assert_ne!(first.admin, second.admin, "two Organisations");
    let ceremony = |proxy: AccountId| [vec![Step::CreatePure(co())], fresh_founding(proxy)].concat();
    assert_eq!(ops.steps(), [ceremony(p1), ceremony(p2)].concat());
}

// Abnormal: a failure at each step is reported as that step, and no later
// step is attempted: `create_pure`, the mapping read, the balance read, the
// transfer, the mapping and the record.
// verifies: LLR-chy957, REQ-6jefu2
#[tokio::test]
async fn a_failed_genesis_step_is_named_and_stops_the_ceremony() {
    let names = [
        GenesisStep::CreatePure,
        GenesisStep::MapAccount,
        GenesisStep::Fund,
        GenesisStep::Fund,
        GenesisStep::MapAccount,
        GenesisStep::RecordGenesis,
    ];
    for (index, step) in names.into_iter().enumerate() {
        let mut answers = vec![Answer::Ok; 6];
        answers[index] = Answer::Fail;
        let ops = FakeOps::answering(&answers);
        let err = genesis(&ops, &(), &co(), CONTRACT, root(), key()).await.unwrap_err();
        assert!(matches!(&err, WriteError::Step { step: failed_step, .. } if *failed_step == step), "step {index}: {err:?}");
        assert_eq!(ops.steps().len(), index + 1, "no step after {step:?} may run");
    }
}

// Abnormal: a dispatched genesis step that only records an approval is a
// failure of that step, not a success.
// verifies: LLR-chy957
#[tokio::test]
async fn an_approval_without_execution_fails_the_genesis_step() {
    for (index, step) in [(4usize, GenesisStep::MapAccount), (5, GenesisStep::RecordGenesis)] {
        let mut answers = vec![Answer::Ok; 6];
        answers[index] = Answer::Approval;
        let ops = FakeOps::answering(&answers);
        let err = genesis(&ops, &(), &co(), CONTRACT, root(), key()).await.unwrap_err();
        assert!(matches!(&err, WriteError::Step { step: failed_step, .. } if *failed_step == step), "{err:?}");
        assert_eq!(ops.steps().len(), index + 1);
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

// ---------------------------------------------------------------------------
// The contract of the substitute's two reads (TEST_GUIDELINES Doubles 3):
// the same assertions
// `write_genesis_e2e.rs` runs against `SubxtWriteOps` on a chopsticks fork.
// ---------------------------------------------------------------------------

// Each contract test also runs `found_at` over a substitute in the same
// chain state and asserts the steps it decides from that state, so the skip
// rules it cites (LLR-jk76wf, LLR-95puh9) redden it. It does not cite
// LLR-htp5tk: `FakeOps` never calls `free_balance_of` (write_pure.rs tests
// it; write_genesis_e2e.rs reaches it through `SubxtWriteOps::free_balance`).

// verifies: LLR-jk76wf, LLR-95puh9
#[tokio::test]
async fn the_substitute_reads_an_untouched_proxy_as_the_chain_does() {
    contract::untouched(&FakeOps::answering(&[]), PROXY).await;
    // Untouched: funded with the whole amount, then mapped and recorded.
    let ops = FakeOps::answering(&[]);
    found_at(&ops, &(), &co(), CONTRACT, PROXY, root(), key()).await.unwrap();
    assert_eq!(ops.steps(), fresh_founding(PROXY));
}

// verifies: LLR-jk76wf, LLR-95puh9
#[tokio::test]
async fn the_substitute_reads_a_funded_unmapped_proxy_as_the_chain_does() {
    contract::funded_unmapped(&FakeOps::answering(&[]).on_chain(None, FUND_AMOUNT), PROXY, FUND_AMOUNT).await;
    // Funded and not mapped: not funded again, mapped once, recorded.
    let ops = FakeOps::answering(&[]).on_chain(None, FUND_AMOUNT);
    found_at(&ops, &(), &co(), CONTRACT, PROXY, root(), key()).await.unwrap();
    assert_eq!(
        ops.steps(),
        vec![
            Step::MappedAccount(admin_of(PROXY)),
            Step::FreeBalance(PROXY),
            Step::Dispatch(co(), proxied(PROXY, map_account_call()), None),
            record_step(PROXY),
        ]
    );
}

// verifies: LLR-jk76wf, LLR-95puh9
#[tokio::test]
async fn the_substitute_reads_a_proxy_mapped_to_itself_as_the_chain_does() {
    let state = || FakeOps::answering(&[]).on_chain(Some(PROXY), FUND_AMOUNT - 1);
    contract::mapped_to_itself(&state(), PROXY, FUND_AMOUNT).await;
    // Mapped to itself: neither funded, nor its balance read, nor mapped again.
    let ops = state();
    found_at(&ops, &(), &co(), CONTRACT, PROXY, root(), key()).await.unwrap();
    assert_eq!(ops.steps(), vec![Step::MappedAccount(admin_of(PROXY)), record_step(PROXY)]);
}

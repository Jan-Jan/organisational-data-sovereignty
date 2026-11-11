//! The periodic verification (SDD-tgu7dp). While the app runs it asks
//! org-io, once every verification interval, to verify every Organisation
//! and provisional founding it holds, through the same body every other
//! verification runs (`commands::verification_outcomes`, over
//! `verify_and_send`), and announces each result as the receiver loop
//! announces one. It decides nothing of the result.

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use org_io::OrgIo;
use tokio::time::{interval_at, Instant, MissedTickBehavior};

use crate::commands;
use crate::events::{self, Emission, ReceiverOutcome};

/// LLR-3ampqm. `run` once per `interval`, the first time one interval
/// after this starts, never two at once (each run is awaited before the
/// next tick). When a run outlasts an interval the next starts as soon as
/// it ends and the missed ones are not made up (tokio's
/// `MissedTickBehavior::Delay`). LLR-2267hj: every outcome's events are
/// emitted in order, as the receiver loop emits them. Runs until the task
/// is dropped.
pub async fn periodic_runs<Run, RunFuture, Emit>(interval: Duration, mut run: Run, mut emit: Emit)
where
    Run: FnMut() -> RunFuture,
    RunFuture: Future<Output = Vec<ReceiverOutcome>>,
    Emit: FnMut(Emission),
{
    let mut ticker = interval_at(Instant::now() + interval, interval);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    loop {
        ticker.tick().await;
        for outcome in run().await {
            for emission in events::emissions_for(&outcome) {
                emit(emission);
            }
        }
    }
}

/// SDD-tgu7dp. Spawned once by `run`'s setup after `AppState::init`, with
/// the interval `init` decided (LLR-2dpn52) and the emitter the receiver
/// loop uses.
pub async fn run_periodic_verification<E>(org_io: Arc<OrgIo>, interval: Duration, emit: E)
where
    E: FnMut(Emission) + Send,
{
    periodic_runs(
        interval,
        || {
            // One run (LLR-3ampqm, LLR-fe7umz): every target org-io holds,
            // verified through `verify_and_send`, so an own adoption is sent
            // once and a behind Organisation is requested at this run; each
            // result mapped as the receiver loop maps one (LLR-2267hj).
            let org_io = Arc::clone(&org_io);
            async move { commands::verification_outcomes(&org_io, None).await }
        },
        emit,
    )
    .await
}

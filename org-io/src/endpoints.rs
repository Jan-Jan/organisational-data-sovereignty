//! One endpoint per Persona (SDD-xwft92): at most one endpoint
//! per Device public key, bound lazily from that Device's keychain item and
//! kept (LLR-d2pzzk); a send goes out from the endpoint of the sending
//! Persona's Device; and once the node starts receiving, every Persona's
//! endpoint has one accept task forwarding each received body, with the
//! sender the handshake authenticated, to the one receive channel
//! (LLR-z35vbh). Each inbound connection is read in a task of its own,
//! bounded in time and in number, so one that stalls stops no other
//! (LLR-8uf9a5). The accept tasks still running are counted, so the
//! receive step knows when every one has ended (LLR-szhe7j). An endpoint
//! that has closed is not reused (PR-tcd2ta): the next send or
//! `start_receiving` for its Persona binds it again, and `start_receiving`
//! gives a new accept task to an endpoint whose task has ended. Once
//! `start_receiving` has run, an endpoint bound by a send accepts too
//! (PR-47aycj), so a rebind while the receive loop runs is not left deaf.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use org_node::DevicePublicKey;
use tokio::sync::{mpsc, oneshot, Mutex, OwnedSemaphorePermit, Semaphore};
use tokio::task::{JoinHandle, JoinSet};

use crate::transport::{
    BindPlan, InboundReply, OrgEndpoint, PendingConnection, TransportError, TransportMode, Verdict, CONNECTION_TIME_LIMIT,
};

/// One received body, the Device the handshake authenticated as its
/// sender, and the responder its verdict goes back through (LLR-unqfj5):
/// dropped unanswered, nothing is written.
pub(crate) struct Inbound {
    pub(crate) sender: DevicePublicKey,
    pub(crate) body: Vec<u8>,
    pub(crate) verdict: oneshot::Sender<Verdict>,
}

/// What an accept task forwards: one body, or the reason a connection gave
/// none, or, last, `TransportError::EndpointClosed`.
type Accepted = Result<Inbound, TransportError>;

/// How many received bodies wait for the receive step before the accept
/// tasks stop accepting: a peer cannot queue without bound.
const INBOUND_CAPACITY: usize = 64;

/// How many of one endpoint's inbound connections are read at once, each in
/// a task of its own (LLR-8uf9a5): while every slot is taken the accept task
/// accepts no further connection, so the bodies being read stay bounded
/// (RC-5wz559's memory bound per stream, times this) and a flood waits
/// rather than growing without bound. Sixteen leaves room for the honest
/// senders of an Organisation's update fan-out while one or more peers
/// stall, each for at most `CONNECTION_TIME_LIMIT`.
const CONNECTIONS_AT_ONCE: usize = 16;

/// One bound endpoint and, once the node receives on it, its accept task.
struct Bound {
    endpoint: Arc<OrgEndpoint>,
    accept_task: Option<JoinHandle<()>>,
}

/// The node's endpoints, keyed by Device public key, and the receive
/// channel their accept tasks feed.
pub(crate) struct Endpoints {
    plan: BindPlan,
    bound: Mutex<HashMap<DevicePublicKey, Bound>>,
    inbound_sender: mpsc::Sender<Accepted>,
    inbound: Mutex<mpsc::Receiver<Accepted>>,
    /// How many accept tasks are running: each counts itself out before it
    /// forwards its endpoint's closure.
    accepting: Arc<AtomicUsize>,
    /// Whether `start_accepting` has run: from then on every endpoint bound,
    /// by a send too, gets an accept task (PR-47aycj). Written
    /// and read under `bound`'s lock.
    receiving: AtomicBool,
    /// How long one inbound connection may take, handshake to body
    /// (LLR-8uf9a5): `CONNECTION_TIME_LIMIT`, shorter only under a test.
    connection_time_limit: Duration,
}

impl Default for Endpoints {
    fn default() -> Self {
        Self::new(BindPlan::Loopback)
    }
}

impl Endpoints {
    /// No endpoint bound yet; each will be bound for `plan`.
    pub(crate) fn new(plan: BindPlan) -> Self {
        let (inbound_sender, inbound) = mpsc::channel(INBOUND_CAPACITY);
        Self {
            plan,
            bound: Mutex::new(HashMap::new()),
            inbound_sender,
            inbound: Mutex::new(inbound),
            accepting: Arc::new(AtomicUsize::new(0)),
            receiving: AtomicBool::new(false),
            connection_time_limit: CONNECTION_TIME_LIMIT,
        }
    }

    /// Bound each inbound connection by `limit` in place of
    /// `CONNECTION_TIME_LIMIT` (test builds only: a test of the limit need
    /// not wait half a minute). Applies to accept tasks started after it.
    #[cfg(feature = "test-support")]
    pub(crate) fn set_connection_time_limit(&mut self, limit: Duration) {
        self.connection_time_limit = limit;
    }

    /// The transport mode the endpoints are bound in (an in-process relay
    /// plan dials by id, as Networked does).
    pub(crate) fn transport_mode(&self) -> TransportMode {
        match self.plan {
            BindPlan::Loopback => TransportMode::Loopback,
            _ => TransportMode::Networked,
        }
    }

    /// The endpoint of `device_key`, bound from its keychain item on first
    /// use and the same one on every later call while it is open; once it
    /// has closed, bound again (LLR-d2pzzk). Once the node receives, an
    /// endpoint bound here accepts too (PR-47aycj; LLR-z35vbh), so a send that binds a closed endpoint again while the
    /// receive loop runs leaves it accepting.
    pub(crate) async fn endpoint(&self, device_key: &DevicePublicKey) -> Result<Arc<OrgEndpoint>, TransportError> {
        let mut bound = self.bound.lock().await;
        let entry = Self::bound_or_bind(&mut bound, &self.plan, device_key).await?;
        if self.receiving.load(Ordering::SeqCst) {
            self.keep_accepting(entry);
        }
        Ok(entry.endpoint.clone())
    }

    /// An endpoint that reports closed, or closing (`OrgEndpoint::is_closed`),
    /// is dropped and bound again (PR-tcd2ta). Its accept task, if any, is left to end on
    /// its own: it sees the closure and counts itself out of `accepting`,
    /// which an abort would skip. Binding happens under the map's lock, so
    /// one Device key never has two open endpoints.
    async fn bound_or_bind<'map>(
        bound: &'map mut HashMap<DevicePublicKey, Bound>,
        plan: &BindPlan,
        device_key: &DevicePublicKey,
    ) -> Result<&'map mut Bound, TransportError> {
        if bound.get(device_key).is_some_and(|entry| entry.endpoint.is_closed()) {
            bound.remove(device_key);
        }
        if !bound.contains_key(device_key) {
            let endpoint = OrgEndpoint::bind_from_keychain(device_key, plan.clone()).await?;
            bound.insert(*device_key, Bound { endpoint: Arc::new(endpoint), accept_task: None });
        }
        bound.get_mut(device_key).ok_or_else(|| TransportError::Bind("endpoint not kept".into()))
    }

    /// Mark the node receiving, under the map's lock, as `start_accepting`
    /// does, binding nothing: called before the Personas are listed, so a
    /// Persona created after the listing finds the flag set in
    /// `accept_if_receiving` (LLR-z35vbh as amended 2026-10-15).
    pub(crate) async fn mark_receiving(&self) {
        let _bound = self.bound.lock().await;
        self.receiving.store(true, Ordering::SeqCst);
    }

    /// Bind the endpoint of each of `device_keys` and give each one running
    /// accept task (LLR-z35vbh): a closed endpoint is bound again, and an
    /// endpoint whose accept task has ended gets a new one (PR-tcd2ta). A
    /// bind that fails is returned; the endpoints bound before it keep
    /// accepting. From the first call on, the node is receiving: every
    /// endpoint bound later, by a send too, accepts (PR-47aycj).
    pub(crate) async fn start_accepting(&self, device_keys: &[DevicePublicKey]) -> Result<(), TransportError> {
        let mut bound = self.bound.lock().await;
        self.receiving.store(true, Ordering::SeqCst);
        for device_key in device_keys {
            let entry = Self::bound_or_bind(&mut bound, &self.plan, device_key).await?;
            self.keep_accepting(entry);
        }
        Ok(())
    }

    /// When the node receives, bind the endpoint of `device_key` and give it
    /// a running accept task, as `start_accepting` does for one key; when it
    /// does not, nothing (PR-y6t5kt): a
    /// Persona created, or replying, while the receive loop runs is received
    /// on (LLR-z35vbh). The flag is read and the task counted under the
    /// map's lock.
    pub(crate) async fn accept_if_receiving(&self, device_key: &DevicePublicKey) -> Result<(), TransportError> {
        let mut bound = self.bound.lock().await;
        if self.receiving.load(Ordering::SeqCst) {
            let entry = Self::bound_or_bind(&mut bound, &self.plan, device_key).await?;
            self.keep_accepting(entry);
        }
        Ok(())
    }

    /// Give `entry` a running accept task unless it has one, counting the
    /// new task into `accepting` before it starts, so the count of running
    /// tasks stays exact. Called under `bound`'s lock.
    fn keep_accepting(&self, entry: &mut Bound) {
        if entry.accept_task.as_ref().is_none_or(JoinHandle::is_finished) {
            self.accepting.fetch_add(1, Ordering::SeqCst);
            let task =
                accept(entry.endpoint.clone(), self.inbound_sender.clone(), self.accepting.clone(), self.connection_time_limit);
            entry.accept_task = Some(tokio::spawn(task));
        }
    }

    /// The next received body and its sender, or the reason a connection
    /// gave none; `TransportError::EndpointClosed` once every accept task
    /// has ended and the bodies they forwarded are taken, or at once when
    /// none was started (LLR-szhe7j). Waits on the receive channel's own
    /// lock only.
    pub(crate) async fn next_inbound(&self) -> Accepted {
        let mut inbound = self.inbound.lock().await;
        loop {
            let next = if self.accepting.load(Ordering::SeqCst) == 0 {
                inbound.try_recv().map_err(|_| TransportError::EndpointClosed)?
            } else {
                inbound.recv().await.ok_or(TransportError::EndpointClosed)?
            };
            // One endpoint's closure; the others may still accept.
            if !matches!(next, Err(TransportError::EndpointClosed)) {
                return next;
            }
        }
    }

    /// How many endpoints are bound (test builds only).
    #[cfg(feature = "test-support")]
    pub(crate) async fn bound_count(&self) -> usize {
        self.bound.lock().await.len()
    }

    /// The endpoint bound for `device_key`, if any, binding nothing and
    /// starting no accept task (test builds only).
    #[cfg(feature = "test-support")]
    pub(crate) async fn bound_endpoint(&self, device_key: &DevicePublicKey) -> Option<Arc<OrgEndpoint>> {
        self.bound.lock().await.get(device_key).map(|entry| entry.endpoint.clone())
    }
}

impl Drop for Endpoints {
    /// Stop the accept tasks, so each endpoint closes with its last holder.
    fn drop(&mut self) {
        for entry in self.bound.get_mut().values_mut() {
            if let Some(task) = entry.accept_task.take() {
                task.abort();
            }
        }
    }
}

/// One endpoint's accept task (LLR-8uf9a5): it takes each inbound
/// connection as it arrives, holding one of `CONNECTIONS_AT_ONCE` slots for
/// it, and reads it in a task of its own, bounded by `limit`, so a
/// connection that stalls holds its own slot and nothing else. Each received
/// body and its authenticated sender, or a connection's failure (one past
/// the limit included), goes to the receive channel. When the endpoint
/// closes, the task waits for its connections' tasks (each ends within the
/// limit), counts itself out of `accepting`, then forwards the closure, so
/// the count stays exact (LLR-szhe7j): no body of this endpoint can arrive
/// after it. When the channel closes it counts itself out and ends, and its
/// connections' tasks end with it. Each body's stream send half waits,
/// apart from the loop, for the receive step's verdict (LLR-unqfj5).
async fn accept(endpoint: Arc<OrgEndpoint>, inbound: mpsc::Sender<Accepted>, accepting: Arc<AtomicUsize>, limit: Duration) {
    let slots = Arc::new(Semaphore::new(CONNECTIONS_AT_ONCE));
    let mut connections = JoinSet::new();
    loop {
        let next = tokio::select! {
            next = next_connection(&endpoint, &slots) => next,
            () = inbound.closed() => {
                accepting.fetch_sub(1, Ordering::SeqCst);
                return;
            }
        };
        while connections.try_join_next().is_some() {}
        let Some((slot, pending)) = next else {
            while connections.join_next().await.is_some() {}
            accepting.fetch_sub(1, Ordering::SeqCst);
            let _ = inbound.send(Err(TransportError::EndpointClosed)).await;
            return;
        };
        let inbound = inbound.clone();
        connections.spawn(async move {
            let received = pending.receive(limit).await;
            // The channel closed: the node is gone, and so is the body.
            let _ = inbound.send(received.map(|(sender, body, reply)| Inbound { sender, body, verdict: await_verdict(reply) })).await;
            drop(slot);
        });
    }
}

/// A free connection slot, then the endpoint's next inbound connection;
/// `None` once the endpoint has closed.
async fn next_connection(endpoint: &OrgEndpoint, slots: &Arc<Semaphore>) -> Option<(OwnedSemaphorePermit, PendingConnection)> {
    let slot = slots.clone().acquire_owned().await.ok()?;
    endpoint.next_connection().await.ok().map(|pending| (slot, pending))
}

/// The responder for one body: a task holds `reply` until the receive step
/// answers, then writes the verdict; a responder dropped unanswered (the
/// body never reached the step, or the step was cancelled) writes nothing.
fn await_verdict(reply: InboundReply) -> oneshot::Sender<Verdict> {
    let (responder, answer) = oneshot::channel();
    tokio::spawn(async move {
        if let Ok(verdict) = answer.await {
            reply.answer(verdict).await;
        }
    });
    responder
}

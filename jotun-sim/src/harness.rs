//! Per-node state: the live [`Engine`], its durable persisted state,
//! and the pending-write queue that models partial disk flushes.
//!
//! Real hosts write to disk at fsync boundaries they choose. Bugs in
//! that choice are the kind of thing simulation exists to surface, so
//! the harness does not automatically apply every `Persist` action to
//! the durable snapshot. It queues them. The scheduler picks when a
//! prefix of that queue gets flushed, and the rest is discarded on
//! crash — exactly mirroring the "we sent before fsync completed"
//! failure mode Raft's action-ordering contract exists to catch.

use jotun_core::{Action, Engine, LogEntry, LogIndex, NodeId, Term};

use crate::env::SharedRng;
use crate::env::SimEnv;

/// The durable snapshot the engine recovers from on restart.
///
/// Tracks exactly what Raft Figure 2 labels "persistent state on all
/// servers": `current_term`, `voted_for`, and the log. Volatile state
/// (commit_index, last_applied, role) is rebuilt by the replacement
/// engine on `Recover`.
#[derive(Debug, Clone)]
pub(crate) struct PersistedState<C> {
    pub(crate) current_term: Term,
    pub(crate) voted_for: Option<NodeId>,
    pub(crate) log: Vec<LogEntry<C>>,
}

impl<C> Default for PersistedState<C> {
    fn default() -> Self {
        Self {
            current_term: Term::ZERO,
            voted_for: None,
            log: Vec::new(),
        }
    }
}

/// A write that has been emitted by the engine but not yet fsynced.
#[derive(Debug, Clone)]
pub(crate) enum PendingWrite<C> {
    HardState {
        current_term: Term,
        voted_for: Option<NodeId>,
    },
    LogEntries(Vec<LogEntry<C>>),
}

/// One node in the simulated cluster: a live engine, its durable
/// snapshot, the still-pending writes, and the applied-entry log the
/// state-machine-safety invariant checks against.
#[derive(Debug)]
pub(crate) struct NodeHarness<C> {
    pub(crate) id: NodeId,
    pub(crate) peers: Vec<NodeId>,
    /// `None` while the node is crashed.
    pub(crate) engine: Option<Engine<C>>,
    pub(crate) persisted: PersistedState<C>,
    pub(crate) pending: Vec<PendingWrite<C>>,
    pub(crate) applied: Vec<LogEntry<C>>,
    pub(crate) heartbeat_interval_ticks: u64,
}

impl<C: Clone> NodeHarness<C> {
    pub(crate) fn new(
        id: NodeId,
        peers: Vec<NodeId>,
        heartbeat_interval_ticks: u64,
        rng: SharedRng,
    ) -> Self {
        let env = Box::new(SimEnv::new(rng));
        let engine = Engine::new(id, peers.iter().copied(), env, heartbeat_interval_ticks);
        Self {
            id,
            peers,
            engine: Some(engine),
            persisted: PersistedState::default(),
            pending: Vec::new(),
            applied: Vec::new(),
            heartbeat_interval_ticks,
        }
    }

    pub(crate) fn is_up(&self) -> bool {
        self.engine.is_some()
    }

    /// Scan `actions` for ordering correctness (§5.1: persist before
    /// send), then queue each persist into `pending` and append each
    /// applied entry to `applied`.
    ///
    /// Returns the index of any `Send` whose prerequisite persist was
    /// not ordered earlier in the vector, for the caller to turn into
    /// a safety violation.
    pub(crate) fn absorb(&mut self, actions: &[Action<C>]) -> Result<(), PersistOrderingError>
    where
        C: PartialEq,
    {
        let mut hard_state_persisted_this_step = false;
        let mut entries_persisted_this_step: Vec<LogEntry<C>> = Vec::new();
        let pre_step_term = self.persisted.current_term;
        let pre_step_voted_for = self.persisted.voted_for;
        let pre_step_log_end = self.persisted.log.len();

        for (i, action) in actions.iter().enumerate() {
            match action {
                Action::PersistHardState {
                    current_term,
                    voted_for,
                } => {
                    self.pending.push(PendingWrite::HardState {
                        current_term: *current_term,
                        voted_for: *voted_for,
                    });
                    hard_state_persisted_this_step = true;
                }
                Action::PersistLogEntries(entries) => {
                    self.pending
                        .push(PendingWrite::LogEntries(entries.clone()));
                    entries_persisted_this_step.extend(entries.iter().cloned());
                }
                Action::Send { message, .. } => {
                    check_send_ordering(
                        i,
                        message,
                        pre_step_term,
                        pre_step_voted_for,
                        pre_step_log_end,
                        hard_state_persisted_this_step,
                        &entries_persisted_this_step,
                    )?;
                }
                Action::Apply(entries) => {
                    self.applied.extend(entries.iter().cloned());
                }
                Action::Redirect { .. } => {}
            }
        }
        Ok(())
    }

    /// Flush up to `n` pending writes into the durable snapshot, or all
    /// of them when `n == usize::MAX`.
    pub(crate) fn flush(&mut self, n: usize) {
        let take = n.min(self.pending.len());
        let drained: Vec<_> = self.pending.drain(..take).collect();
        for write in drained {
            apply_write(&mut self.persisted, write);
        }
    }

    /// Drop the engine and any pending (unflushed) writes. Durable
    /// snapshot stays.
    pub(crate) fn crash(&mut self) {
        self.engine = None;
        self.pending.clear();
    }

    /// Rebuild the engine from the durable snapshot. Applied entries
    /// are *not* reset — the safety checker still needs to compare
    /// post-recovery applies against pre-crash applies.
    pub(crate) fn recover(&mut self, rng: SharedRng) {
        if self.engine.is_some() {
            return;
        }
        let env = Box::new(SimEnv::new(rng));
        let mut engine = Engine::new(
            self.id,
            self.peers.iter().copied(),
            env,
            self.heartbeat_interval_ticks,
        );
        hydrate_engine(&mut engine, &self.persisted);
        self.engine = Some(engine);
    }
}

/// Apply a pending write to the durable snapshot. Log entries
/// overwrite at their own indices — the engine always emits them
/// contiguous with the existing log, but a recovered engine re-sending
/// the same prefix would duplicate otherwise.
fn apply_write<C>(persisted: &mut PersistedState<C>, write: PendingWrite<C>) {
    match write {
        PendingWrite::HardState {
            current_term,
            voted_for,
        } => {
            persisted.current_term = current_term;
            persisted.voted_for = voted_for;
        }
        PendingWrite::LogEntries(entries) => {
            for entry in entries {
                let i = entry.id.index.get();
                if i == 0 {
                    continue;
                }
                let idx = (i - 1) as usize;
                match idx.cmp(&persisted.log.len()) {
                    std::cmp::Ordering::Less => persisted.log[idx] = entry,
                    // A gap (idx > log.len()) shouldn't happen given
                    // the engine emits entries contiguous with its
                    // in-memory log, but don't silently corrupt the
                    // snapshot — append anyway.
                    std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => {
                        persisted.log.push(entry);
                    }
                }
            }
        }
    }
}

/// Feed the durable snapshot back into a fresh engine so it resumes
/// with the same term / vote / log as before the crash. Volatile state
/// (role, commit_index, last_applied) stays at the engine's default
/// follower boot.
fn hydrate_engine<C: Clone>(engine: &mut Engine<C>, persisted: &PersistedState<C>) {
    use jotun_core::{Event, Incoming, Message, RequestAppendEntries};

    // The engine exposes no public setter for term / log. We drive it
    // through the only public mutator — `step` — using a synthetic
    // `AppendEntries` at the persisted term that carries the persisted
    // log. The engine appends entries, bumps term, and records
    // `voted_for = None`. If the persisted `voted_for` was `Some`, we
    // follow up with a `RequestVote` from that candidate at the same
    // term to restore the vote.
    //
    // If the persisted log is empty and term is zero, no hydration is
    // needed — the fresh engine already matches.
    if persisted.current_term == Term::ZERO && persisted.log.is_empty() {
        return;
    }

    // Pick any peer as the synthetic "leader" of the hydration message.
    // Safety: we only call this on a node with peers, which every
    // multi-node cluster has. Single-node simulation doesn't hit this.
    let Some(peer) = engine.peers().iter().copied().next() else {
        return;
    };

    let request = RequestAppendEntries {
        term: persisted.current_term,
        leader_id: peer,
        prev_log_id: None,
        entries: persisted.log.clone(),
        leader_commit: LogIndex::ZERO,
    };
    let _ = engine.step(Event::Incoming(Incoming {
        from: peer,
        message: Message::AppendEntriesRequest(request),
    }));

    if let Some(voted_for) = persisted.voted_for
        && voted_for != peer
    {
        // Forge a vote request from `voted_for` so the engine records
        // its vote for that candidate at the current term. The log
        // predicate requires the candidate's log to be at least as
        // up-to-date — we pass the persisted last log id, which the
        // engine itself holds, so the predicate passes.
        use jotun_core::RequestVote;
        let last_log_id = persisted.log.last().map(|e| e.id);
        let _ = engine.step(Event::Incoming(Incoming {
            from: voted_for,
            message: Message::VoteRequest(RequestVote {
                term: persisted.current_term,
                candidate_id: voted_for,
                last_log_id,
            }),
        }));
    }
}

/// Something a `Send` action needs to have been persisted before it,
/// but wasn't.
#[derive(Debug, Clone)]
pub(crate) struct PersistOrderingError {
    pub(crate) send_index_in_actions: usize,
    pub(crate) reason: &'static str,
}

/// Verify that any state change visible in `message` has a matching
/// Persist earlier in the same `step()` action vec. Raft Figure 2:
/// "respond to RPCs only after updating stable storage".
fn check_send_ordering<C: PartialEq>(
    i: usize,
    message: &jotun_core::Message<C>,
    pre_term: Term,
    pre_voted_for: Option<NodeId>,
    pre_log_end: usize,
    hard_state_persisted: bool,
    entries_persisted: &[LogEntry<C>],
) -> Result<(), PersistOrderingError> {
    use jotun_core::Message as M;

    // Determine the term visible in the outgoing message.
    let msg_term = match message {
        M::VoteRequest(r) => r.term,
        M::VoteResponse(r) => r.term,
        M::AppendEntriesRequest(r) => r.term,
        M::AppendEntriesResponse(r) => r.term,
    };

    // If the message term exceeds what was already durable *and* we
    // haven't seen a PersistHardState earlier this step, that's the
    // classic "sent before fsync" bug.
    if msg_term > pre_term && !hard_state_persisted {
        return Err(PersistOrderingError {
            send_index_in_actions: i,
            reason: "Send carries a term greater than last durable term with no prior PersistHardState",
        });
    }

    // VoteRequest => we became candidate => voted_for changed => same
    // rule. (VoteResponse granting a vote also mutates voted_for; the
    // engine emits persist_hard_state() on grant.)
    if matches!(message, M::VoteRequest(_)) && !hard_state_persisted {
        return Err(PersistOrderingError {
            send_index_in_actions: i,
            reason: "VoteRequest sent without prior PersistHardState in the same step",
        });
    }

    // AppendEntriesRequest carrying entries => those entries must have
    // been persisted earlier this step (we're the leader that just
    // appended them). Heartbeats — empty entries — don't need a log
    // persist, but the leader's no-op broadcast after an election does.
    if let M::AppendEntriesRequest(r) = message
        && !r.entries.is_empty()
    {
        let _ = pre_log_end; // reserved for future use
        for entry in &r.entries {
            if !entries_persisted.iter().any(|e| e.id == entry.id) {
                // Only fire if the entry is new this step. If the entry
                // was already durable (index <= pre_log_end and matches
                // persisted state), we didn't need to persist it again
                // this step — it was persisted in some prior step.
                if (entry.id.index.get() as usize) <= pre_log_end {
                    continue;
                }
                return Err(PersistOrderingError {
                    send_index_in_actions: i,
                    reason: "AppendEntriesRequest carries entries not persisted earlier this step",
                });
            }
        }
    }

    let _ = pre_voted_for;
    Ok(())
}

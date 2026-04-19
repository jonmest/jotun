use crate::engine::incoming::Incoming;
use crate::records::log_entry::ConfigChange;
use crate::types::{index::LogIndex, node::NodeId};

/// The single input type the engine accepts via
/// [`crate::engine::engine::Engine::step`].
///
/// The four sources of forward motion in Raft, funneled through one
/// dispatch: the abstract clock fires (`Tick`), a peer's RPC arrives
/// (`Incoming`), the application submits a command (`ClientProposal`),
/// or an operator asks for a membership change
/// (`ProposeConfigChange`) / leadership transfer (`TransferLeadership`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event<C> {
    /// One unit of abstract time has elapsed. Drives the election timer
    /// (followers/candidates) and the heartbeat interval (leaders).
    /// The caller decides what "one tick" means in wall-clock terms.
    Tick,
    /// A peer sent us an RPC.
    Incoming(Incoming<C>),
    /// The local application is asking us to replicate a command.
    ///
    /// Behaviour by role:
    ///  - **Leader**: appends at `(last+1, current_term)`, emits
    ///    [`crate::engine::action::Action::PersistLogEntries`] and
    ///    broadcasts `AppendEntries` to all peers immediately.
    ///  - **Follower** with a known leader (set by the most recent
    ///    accepted `AppendEntries`): emits
    ///    [`crate::engine::action::Action::Redirect`] so the host can
    ///    forward the client.
    ///  - **Follower** without a known leader, or **Candidate**: drops
    ///    silently. The host should retry on its own cadence.
    ClientProposal(C),
    /// Operator-initiated single-server membership change (§4.3).
    ///
    /// Same role-by-role behaviour as `ClientProposal`, with one extra
    /// rule: a leader refuses if it already has an uncommitted
    /// `ConfigChange` in its log, or if the change is a no-op (adding
    /// an existing member, removing a non-member). On accept, the
    /// active config mutates immediately (pre-commit) per §4.3.
    ProposeConfigChange(ConfigChange),
    /// Operator-initiated leadership transfer to `target`.
    ///
    /// Leaders replicate to `target` until it is caught up to the
    /// leader's current log tail, then send `TimeoutNow` so the target
    /// starts an election immediately. Followers with a known leader
    /// emit [`crate::engine::action::Action::Redirect`]; followers
    /// without a known leader and candidates drop silently.
    TransferLeadership {
        target: NodeId,
    },
    /// Linearizable read request (Raft §8 "`ReadIndex`").
    ///
    /// Leaders record `commit_index` as the read's `read_index`,
    /// confirm they are still leader via a heartbeat-quorum round,
    /// then emit [`crate::engine::action::Action::ReadReady`] once
    /// `last_applied >= read_index`. The id is opaque to the engine;
    /// the host uses it to match `ReadReady` back to a waiter.
    ///
    /// Leaders that have not yet committed an entry in their current
    /// term fail the read with
    /// [`crate::engine::action::Action::ReadFailed`] / `NotReady` —
    /// per §8 the leader must prove current-term authority before
    /// serving linearizable reads, which the §5.4.2 no-op provides
    /// once committed.
    ///
    /// Non-leaders redirect (if leader is known) or fail the read
    /// with `NoLeader`.
    ProposeRead {
        id: u64,
    },
    /// The host has just produced a snapshot of the application state
    /// machine that captures everything applied up to
    /// `last_included_index`. The engine truncates its in-memory log
    /// up to and including that index, records the snapshot floor,
    /// and emits an [`crate::engine::action::Action::PersistSnapshot`]
    /// for the host to flush.
    ///
    /// Rejected (silently) if `last_included_index > commit_index` —
    /// the host can only snapshot committed state.
    SnapshotTaken {
        last_included_index: LogIndex,
        bytes: Vec<u8>,
    },
}

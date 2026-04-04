use crate::{
    records::{log_entry::LogEntry, message::Message},
    types::node::NodeId,
};

/// The engine's only output, in vector form per `step()` call.
///
/// The engine never performs I/O directly. Instead, every effect it
/// wants the host to carry out becomes an `Action`. The host fulfils
/// the actions however it likes — sockets, async runtimes, in-memory
/// simulators. This is what makes the engine purely synchronous and
/// testable without a network or filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action<C> {
    /// Send `message` to peer `to`. The host owns the network
    /// transport; the engine just describes who and what.
    Send { to: NodeId, message: Message<C> },
    /// These entries (in index order, contiguous, all newly committed)
    /// are now safe to feed to the application state machine. The
    /// engine advances `last_applied` to the last index in the slice
    /// when emitting this; the host must not skip the action.
    Apply(Vec<LogEntry<C>>),
    /// A client proposal landed on a non-leader; the host should
    /// retarget the client at this peer (the leader for the current
    /// term, as last observed via `AppendEntries`).
    Redirect { leader_hint: NodeId },
}

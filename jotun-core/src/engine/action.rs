use crate::{
    records::{log_entry::LogEntry, message::Message},
    types::node::NodeId,
};

/// The engine's only output, in vector form per `step()` call.
///
/// The engine never performs I/O directly. Instead, every effect it
/// wants the host to carry out becomes an `Action`. The host fulfils
/// the actions however it likes — sockets, async runtimes, on-disk
/// persistence, in-memory simulators. This is what makes the engine
/// purely synchronous and testable without a network or filesystem.
///
/// Not every variant is wired up yet; some are placeholders for the
/// host integration we'll layer on later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action<C> {
    /// Send `message` to peer `to`. The host owns the network
    /// transport; the engine just describes who and what.
    Send { to: NodeId, message: Message<C> },
    /// Engine state needs to be flushed durably before continuing.
    /// Reserved for when persistence is wired in.
    PersistState,
    /// Engine has appended these entries to its in-memory log; the
    /// host should persist them. Reserved for when persistence is
    /// wired in.
    AppendLogEntries(Vec<LogEntry<C>>),
    /// These entries (in index order, contiguous, all newly committed)
    /// are now safe to feed to the application state machine. The
    /// engine advances `last_applied` to the last index in the slice
    /// when emitting this; the host must not skip the action.
    Apply(Vec<LogEntry<C>>),

    Redirect { leader_hint: NodeId },
}

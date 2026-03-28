use crate::types::log::LogId;

/// Payload carried by a [`LogEntry`].
///
/// Two flavors:
///  - [`LogPayload::Command`] wraps an application-defined command. The
///    state machine consumes it once the entry commits.
///  - [`LogPayload::Noop`] is a leader-emitted placeholder appended on
///    every leadership transition. §5.4.2 requires a leader to commit at
///    least one entry from its current term before counting prior-term
///    entries as committed; the no-op makes that possible without waiting
///    for a client proposal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogPayload<C> {
    /// Empty entry that exists solely to advance the current-term
    /// commit boundary (§5.4.2).
    Noop,
    /// Application-defined command. Opaque to the consensus layer.
    Command(C),
}

/// A single entry in the replicated log: a payload tagged with its
/// position and the term in which a leader assigned it.
///
/// Once committed, every node in the cluster will see the exact same
/// sequence of entries with the exact same `id`s — that's the durable
/// outcome the rest of Raft is engineered to deliver. The application
/// state machine consumes each [`LogPayload::Command`] in order and
/// ignores [`LogPayload::Noop`] entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry<C> {
    /// Position and term assigned by the leader that first appended this
    /// entry (§5.3 Log Matching).
    pub id: LogId,
    /// What this entry carries: either a no-op or an application command.
    pub payload: LogPayload<C>,
}

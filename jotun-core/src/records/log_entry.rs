use crate::types::log::LogId;

/// A single entry in the replicated log: an application command tagged
/// with its position and the term in which a leader assigned it.
///
/// Once committed, every node in the cluster will see the exact same
/// sequence of entries with the exact same `id`s — that's the durable
/// outcome the rest of Raft is engineered to deliver. The `command` is
/// opaque to the consensus layer; the application state machine is what
/// gives it meaning when it eventually consumes the entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry<C> {
    /// Position and term assigned by the leader that first appended this
    /// entry (§5.3 Log Matching).
    pub id: LogId,
    /// The application-defined command this entry carries.
    pub command: C,
}

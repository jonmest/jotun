use std::fmt;

/// Position of an entry in the replicated log.
///
/// Indices are 1-based per Raft convention. [`LogIndex::ZERO`] is a
/// pre-log sentinel meaning "before any entry" — it is never the index
/// of a real entry. The first appended entry is always at index 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[must_use]
pub struct LogIndex(u64);

impl LogIndex {
    /// The pre-log sentinel. Returned when "before any entry" is the
    /// honest answer (e.g., as `prev_log_index` for the very first
    /// `AppendEntries` against an empty log).
    pub const ZERO: Self = Self(0);

    /// Construct a `LogIndex` from a raw `u64`.
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// The raw `u64` underlying this index.
    #[must_use]
    pub fn get(self) -> u64 {
        self.0
    }

    /// The next index. Used when assigning a new entry's position.
    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl fmt::Display for LogIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "log_index:{}", self.0)
    }
}

/// Highest log index known to be committed (Figure 2).
///
/// "Committed" means an entry has been replicated on a majority of the
/// cluster and therefore can never be lost. The leader unilaterally
/// advances `commit_index` when its `matchIndex` set shows majority
/// replication; followers advance via `leader_commit` piggybacked on
/// `AppendEntries`.
///
/// A separate type from [`LogIndex`] to make accidental confusion
/// between "this entry's position" and "the highest committed position"
/// impossible at the type level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use]
pub struct CommitIndex(LogIndex);

impl CommitIndex {
    /// Wrap a `LogIndex` as a commit position.
    pub fn new(index: LogIndex) -> Self {
        Self(index)
    }

    /// The underlying `LogIndex`.
    pub fn get(self) -> LogIndex {
        self.0
    }
}

impl fmt::Display for CommitIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "commit_index:{}", self.0)
    }
}

/// Highest log index applied to the local state machine (Figure 2).
///
/// Always satisfies `last_applied <= commit_index` — you cannot apply
/// what you have not yet committed. The application layer drives this
/// forward as it consumes entries.
///
/// A separate type from [`LogIndex`] for the same reason as
/// [`CommitIndex`]: type-level safety against confusing the three
/// related-but-distinct positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use]
pub struct AppliedIndex(LogIndex);

impl AppliedIndex {
    /// Wrap a `LogIndex` as an applied position.
    pub fn new(index: LogIndex) -> Self {
        Self(index)
    }

    /// The underlying `LogIndex`.
    pub fn get(self) -> LogIndex {
        self.0
    }
}

impl fmt::Display for AppliedIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "applied_index:{}", self.0)
    }
}

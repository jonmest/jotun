// log indices are u64 but we back the log with a Vec; `u64 as usize` is safe
// on any 64-bit target and a log larger than 2^32 won't fit in RAM on 32-bit.
#![allow(clippy::cast_possible_truncation)]

use crate::records::log_entry::LogEntry;
use crate::types::{index::LogIndex, log::LogId, term::Term};

/// The replicated log — the linear, append-mostly history that consensus
/// is engineered to deliver to every node identically.
///
/// Indices are 1-based (Raft convention). [`LogIndex::ZERO`] is a
/// pre-log sentinel meaning "before any entry" and is never the index
/// of a real entry.
///
/// **Snapshot floor (§7).** After `Engine::install_snapshot` (or an
/// inbound `InstallSnapshot` RPC), entries below the floor are no
/// longer in `entries` — their state lives in the host-persisted
/// snapshot. Reading those indices via [`Log::entry_at`] returns
/// `None`. The single boundary entry's `(index, term)` is preserved
/// in `snapshot_last` so callers above the floor can still validate
/// `prev_log_id` checks against it.
///
/// **Invariants** (debug-checked internally):
///  - In-memory entries have contiguous indices starting at
///    `snapshot_last_index + 1` (or 1 if no snapshot).
///  - Entry terms are non-decreasing across the in-memory log (a
///    leader only appends at its current term, which is monotonic
///    across leadership).
///
/// Followers reconcile against incoming `AppendEntries` by truncating
/// conflicting tails and appending missing entries; leaders use
/// [`Log::entries_from`] to slice out what each peer needs next.
#[derive(Debug)]
pub struct Log<C> {
    entries: Vec<LogEntry<C>>,
    /// `(index, term)` of the entry conceptually at the snapshot's
    /// tail. Both `LogIndex::ZERO` / `Term::ZERO` when no snapshot
    /// has been installed.
    snapshot_last: LogId,
}

impl<C> Default for Log<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> Log<C> {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
            snapshot_last: LogId::new(LogIndex::ZERO, Term::ZERO),
        }
    }

    /// True iff the log holds no in-memory entries. The snapshot floor
    /// is independent — a log can be `is_empty()` while still having
    /// a non-zero snapshot. Callers reasoning about "any history at
    /// all" should use `last_log_id().is_none()` instead.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Number of in-memory entries. Excludes anything compacted into
    /// the snapshot.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// The smallest index callers can read via [`Log::entry_at`].
    /// Equals `snapshot_last_index + 1`, or 1 if no snapshot.
    pub fn first_index(&self) -> LogIndex {
        LogIndex::new(self.snapshot_last.index.get() + 1)
    }

    /// `(index, term)` at the snapshot's tail. Both fields are zero
    /// when no snapshot has been installed.
    #[must_use]
    pub fn snapshot_last(&self) -> LogId {
        self.snapshot_last
    }

    /// The id of the last entry, or `None` if the log has no history
    /// at all (no in-memory entries AND no snapshot).
    #[must_use]
    pub fn last_log_id(&self) -> Option<LogId> {
        if let Some(last) = self.entries.last() {
            Some(last.id)
        } else if self.snapshot_last.index == LogIndex::ZERO {
            None
        } else {
            Some(self.snapshot_last)
        }
    }

    /// The entry at a 1-based index, if it exists in memory. Returns
    /// `None` for indices ≤ snapshot floor (those are inside the
    /// snapshot and not individually addressable) and for indices
    /// past the in-memory tail.
    #[must_use]
    pub fn entry_at(&self, index: LogIndex) -> Option<&LogEntry<C>> {
        let floor = self.snapshot_last.index.get();
        let i = index.get().checked_sub(floor + 1)?;
        self.entries.get(i as usize)
    }

    /// The term of the entry at `index`, if known. Equals
    /// `snapshot_last.term` for the floor index itself; `None` for
    /// indices below the floor or past the in-memory tail.
    #[must_use]
    pub fn term_at(&self, index: LogIndex) -> Option<Term> {
        if index == LogIndex::ZERO {
            return None;
        }
        if index == self.snapshot_last.index {
            return Some(self.snapshot_last.term);
        }
        self.entry_at(index).map(|e| e.id.term)
    }

    /// All in-memory entries with index ≥ `index`. Returns an empty
    /// slice if `index` falls below the snapshot floor (caller should
    /// send an `InstallSnapshot` instead) or past the tail.
    #[must_use]
    pub fn entries_from(&self, index: LogIndex) -> &[LogEntry<C>] {
        let floor = self.snapshot_last.index.get();
        if index.get() <= floor {
            // Caller wants entries from below the floor — that's
            // snapshot territory, not addressable here.
            return &[];
        }
        let i = (index.get() - floor - 1) as usize;
        self.entries.get(i..).unwrap_or(&[])
    }

    /// Append a single entry. Caller is responsible for constructing
    /// entries with indices contiguous with whatever's already in the
    /// log (or `first_index()` if empty). Debug-checked.
    pub(crate) fn append(&mut self, entry: LogEntry<C>) {
        debug_assert!(
            match self.entries.last() {
                None => entry.id.index == self.first_index(),
                Some(last) => entry.id.index == last.id.index.next(),
            },
            "log entries must have contiguous indices starting at first_index()"
        );
        self.entries.push(entry);
    }

    /// Remove all entries with index ≥ `index`. No-op if `index` is
    /// past the end OR at/below the snapshot floor (snapshotted
    /// entries are immutable). Used by followers when an
    /// `AppendEntries` RPC conflicts with local state.
    pub(crate) fn truncate_from(&mut self, index: LogIndex) {
        let floor = self.snapshot_last.index.get();
        if index.get() <= floor {
            // Truncating into or below the snapshot is forbidden:
            // §5.4.1 guarantees a correct leader never asks us to.
            return;
        }
        let i = (index.get() - floor - 1) as usize;
        if i < self.entries.len() {
            self.entries.truncate(i);
        }
    }

    /// Install a fresh snapshot floor. Drops every in-memory entry
    /// with index ≤ `last_included_index`. Entries past the floor
    /// survive only when their index > `last_included_index` AND
    /// (for the boundary entry) their term agrees with
    /// `last_included_term` — otherwise the in-memory tail is wiped
    /// because Log Matching no longer holds across the floor.
    pub(crate) fn install_snapshot(
        &mut self,
        last_included_index: LogIndex,
        last_included_term: Term,
    ) {
        // If our existing log already has the snapshot's last entry
        // with the matching term, keep everything past it. Otherwise
        // we have no consistent prefix — wipe the in-memory log.
        let consistent = self
            .entry_at(last_included_index)
            .is_some_and(|e| e.id.term == last_included_term);
        let new_first = LogIndex::new(last_included_index.get() + 1);
        self.snapshot_last = LogId::new(last_included_index, last_included_term);
        if consistent {
            // Keep entries strictly past the snapshot floor.
            self.entries.retain(|e| e.id.index >= new_first);
        } else {
            self.entries.clear();
        }
    }

    /// True iff `candidate_last_log` is at least as up-to-date as
    /// ours per §5.4.1 (last term wins; ties broken by length).
    /// Snapshot floor is implicitly considered — `last_log_id` returns
    /// the snapshot's tail when in-memory log is empty.
    #[must_use]
    pub fn is_superseded_by(&self, candidate_last_log: Option<LogId>) -> bool {
        match (self.last_log_id(), candidate_last_log) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some(ours), Some(theirs)) => {
                theirs.term > ours.term || (theirs.term == ours.term && theirs.index >= ours.index)
            }
        }
    }

    /// Check structural invariants. Panics in debug builds when violated,
    /// no-op in release. Intended to run at the end of every state transition.
    ///
    /// §5.3 Log Matching Property requires:
    ///  - in-memory entries have contiguous indices starting at
    ///    `first_index()`,
    ///  - entry terms are non-decreasing across the log (a leader only appends
    ///    at its current term, which is monotonic across leadership).
    #[cfg(debug_assertions)]
    pub(crate) fn check_invariants(&self) {
        let floor = self.snapshot_last.index.get();
        let mut prev_term: Option<Term> = None;
        for (i, entry) in self.entries.iter().enumerate() {
            let expected = LogIndex::new(floor + (i as u64) + 1);
            debug_assert_eq!(
                entry.id.index, expected,
                "log entry at position {i} has non-contiguous index {:?} (expected {expected:?})",
                entry.id.index,
            );
            if let Some(pt) = prev_term {
                debug_assert!(
                    entry.id.term >= pt,
                    "log terms must be non-decreasing (§5.3): {pt:?} -> {:?}",
                    entry.id.term,
                );
            } else if floor > 0 {
                // First in-memory entry's term must be ≥ snapshot's term.
                debug_assert!(
                    entry.id.term >= self.snapshot_last.term,
                    "first in-memory entry's term {:?} must be ≥ snapshot term {:?}",
                    entry.id.term,
                    self.snapshot_last.term,
                );
            }
            prev_term = Some(entry.id.term);
        }
    }

    #[cfg(not(debug_assertions))]
    pub(crate) fn check_invariants(&self) {}
}

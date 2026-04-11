//! Pass-1 snapshot tests: host-driven `Event::SnapshotTaken` truncates
//! the log up to a committed index and emits `Action::PersistSnapshot`.
//! `InstallSnapshot` RPC + leader-side branching land in pass 2.

use super::fixtures::{
    append_entries_from, append_entries_request, follower, log_id, seed_log, snapshot_taken, term,
};
use crate::engine::action::Action;
use crate::types::index::LogIndex;
use crate::types::term::Term;

// ---------------------------------------------------------------------------
// Successful snapshot install
// ---------------------------------------------------------------------------

/// Drive a follower's `commit_index` up to `idx` by accepting an empty
/// `AppendEntries` from a fake leader after seeding.
fn commit_follower_log(engine: &mut crate::engine::engine::Engine<Vec<u8>>, idx: u64) {
    let last_term = engine.log().last_log_id().unwrap().term;
    engine.step(append_entries_from(
        2,
        append_entries_request(
            last_term.get(),
            2,
            engine.log().last_log_id(),
            vec![],
            idx,
        ),
    ));
}

#[test]
fn snapshot_taken_truncates_log_up_to_index_and_emits_persist_snapshot() {
    let mut engine = follower(1);
    seed_log(&mut engine, &[1, 1, 1, 1, 1]);
    commit_follower_log(&mut engine, 3);
    assert_eq!(engine.commit_index(), LogIndex::new(3));

    let actions = engine.step(snapshot_taken(3, b"snap-bytes".to_vec()));

    // Log floor advanced.
    assert_eq!(engine.log().snapshot_last().index, LogIndex::new(3));
    assert_eq!(engine.log().snapshot_last().term, term(1));
    // Pre-floor entries no longer addressable.
    assert!(engine.log().entry_at(LogIndex::new(1)).is_none());
    assert!(engine.log().entry_at(LogIndex::new(2)).is_none());
    assert!(engine.log().entry_at(LogIndex::new(3)).is_none());
    // Post-floor entries survive.
    assert!(engine.log().entry_at(LogIndex::new(4)).is_some());
    assert!(engine.log().entry_at(LogIndex::new(5)).is_some());
    // last_log_id still reflects the highest index we've ever held.
    assert_eq!(
        engine.log().last_log_id().map(|l| l.index.get()),
        Some(5),
    );

    // PersistSnapshot emitted with the right metadata.
    let persist = actions.iter().find_map(|a| match a {
        Action::PersistSnapshot {
            last_included_index,
            last_included_term,
            bytes,
        } => Some((*last_included_index, *last_included_term, bytes.clone())),
        _ => None,
    });
    assert_eq!(
        persist,
        Some((LogIndex::new(3), term(1), b"snap-bytes".to_vec())),
    );
}

#[test]
fn snapshot_at_last_committed_index_clears_in_memory_log() {
    // Snapshot covers everything: in-memory log becomes empty but
    // last_log_id still reports the snapshot's tail.
    let mut engine = follower(1);
    seed_log(&mut engine, &[1, 1, 1]);
    commit_follower_log(&mut engine, 3);

    engine.step(snapshot_taken(3, b"all".to_vec()));

    assert!(engine.log().is_empty());
    assert_eq!(engine.log().snapshot_last().index, LogIndex::new(3));
    assert_eq!(
        engine.log().last_log_id(),
        Some(log_id(3, 1)),
        "last_log_id falls through to the snapshot tail",
    );
}

#[test]
fn snapshot_advances_first_index() {
    let mut engine = follower(1);
    seed_log(&mut engine, &[1, 1, 1, 1]);
    commit_follower_log(&mut engine, 2);

    assert_eq!(engine.log().first_index(), LogIndex::new(1));
    engine.step(snapshot_taken(2, b"".to_vec()));
    assert_eq!(engine.log().first_index(), LogIndex::new(3));
}

// ---------------------------------------------------------------------------
// Refusal / no-op paths
// ---------------------------------------------------------------------------

#[test]
fn snapshot_past_commit_index_is_refused() {
    // Host can only snapshot what's committed. last_committed = 2,
    // request snapshot at 5 → refused, log unchanged.
    let mut engine = follower(1);
    seed_log(&mut engine, &[1, 1, 1, 1, 1]);
    commit_follower_log(&mut engine, 2);

    let actions = engine.step(snapshot_taken(5, b"too-far".to_vec()));

    assert!(
        actions.is_empty(),
        "snapshot past commit_index must yield no actions",
    );
    assert_eq!(engine.log().snapshot_last().index, LogIndex::ZERO);
    assert_eq!(engine.log().len(), 5);
}

#[test]
fn stale_snapshot_at_or_below_existing_floor_is_a_noop() {
    let mut engine = follower(1);
    seed_log(&mut engine, &[1, 1, 1, 1]);
    commit_follower_log(&mut engine, 4);
    engine.step(snapshot_taken(3, b"first".to_vec()));
    assert_eq!(engine.log().snapshot_last().index, LogIndex::new(3));

    // Re-snapshot at 3 (same point): no-op.
    let actions = engine.step(snapshot_taken(3, b"second".to_vec()));
    assert!(actions.is_empty());
    assert_eq!(engine.log().snapshot_last().index, LogIndex::new(3));

    // Re-snapshot at 2 (below existing): no-op.
    let actions = engine.step(snapshot_taken(2, b"third".to_vec()));
    assert!(actions.is_empty());
    assert_eq!(engine.log().snapshot_last().index, LogIndex::new(3));
}

// ---------------------------------------------------------------------------
// Term plumbing
// ---------------------------------------------------------------------------

#[test]
fn snapshot_records_correct_term_for_floor() {
    // Mixed-term log: snapshot the boundary, term must come from the
    // entry at last_included_index, not a synthesised value.
    let mut engine = follower(1);
    seed_log(&mut engine, &[1, 1, 2, 2, 3]); // entry 3 has term 2
    commit_follower_log(&mut engine, 3);

    engine.step(snapshot_taken(3, b"".to_vec()));
    assert_eq!(engine.log().snapshot_last().term, Term::new(2));
}

// ---------------------------------------------------------------------------
// Subsequent reads through entry_at and term_at
// ---------------------------------------------------------------------------

#[test]
fn term_at_floor_index_returns_snapshot_term() {
    let mut engine = follower(1);
    seed_log(&mut engine, &[1, 1, 1]);
    commit_follower_log(&mut engine, 3);
    engine.step(snapshot_taken(3, b"".to_vec()));

    // Entry at index 3 is in the snapshot, but term_at(3) must still
    // return the correct term — needed for prev_log_id checks above
    // the floor.
    assert_eq!(engine.log().term_at(LogIndex::new(3)), Some(term(1)));
    // Below the floor: still no answer.
    assert_eq!(engine.log().term_at(LogIndex::new(2)), None);
}

#[test]
fn entries_from_below_floor_returns_empty_slice() {
    let mut engine = follower(1);
    seed_log(&mut engine, &[1, 1, 1, 1, 1]);
    commit_follower_log(&mut engine, 3);
    engine.step(snapshot_taken(3, b"".to_vec()));

    // Asking for entries from 2 (below floor): empty — caller should
    // send InstallSnapshot instead. Asking from 4 returns the tail.
    assert!(engine.log().entries_from(LogIndex::new(2)).is_empty());
    assert_eq!(engine.log().entries_from(LogIndex::new(4)).len(), 2);
}

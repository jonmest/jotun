use std::collections::BTreeSet;

use crate::{engine::peer_progress::PeerProgress, types::node::NodeId};

/// Per-role state the follower carries while in the Follower role.
///
/// Currently empty — followers don't need any role-specific bookkeeping
/// beyond what's already on `RaftState` (term, log, `voted_for`). Reserved
/// for future fields like "current leader id" if observability needs it.
#[derive(Default, Copy, Clone, Debug)]
pub struct FollowerState {}

/// Per-role state the engine carries while in the Candidate role (§5.2).
///
/// Tracks which peers have granted us their vote this term. Self always
/// votes for self (inserted by `become_candidate`); set semantics make
/// duplicate grants from the same peer harmless.
#[derive(Default, Clone, Debug)]
pub struct CandidateState {
    /// Node ids that have granted us a vote this term — including self.
    /// Election wins when `votes_granted.len() >= cluster_majority()`.
    pub votes_granted: BTreeSet<NodeId>,
}

/// Per-role state the engine carries while in the Leader role
/// (§5.3 Figure 2).
///
/// All leader bookkeeping lives in [`PeerProgress`]: per-peer `nextIndex`
/// and `matchIndex`, plus the median calculation that drives commit
/// advancement. Membership changes (§6) eventually mutate this same
/// structure.
#[derive(Default, Clone, Debug)]
pub struct LeaderState {
    /// Per-peer replication state for every other node in the cluster.
    pub progress: PeerProgress,
}

/// The Raft role. Every node is exactly one of these at any given time.
///
/// Transitions are tightly constrained:
///  - Anyone → `Follower` on observing a higher term (§5.1).
///  - `Follower`/`Candidate` → `Candidate` on election timeout (§5.2).
///  - `Candidate` → `Follower` on receiving a current-term `AppendEntries`
///    (a peer won the election) or a higher-term message.
///  - `Candidate` → `Leader` on receiving votes from a majority.
///  - `Leader` → `Follower` only on observing a higher term.
#[derive(Debug, Clone)]
pub enum RoleState {
    Follower(FollowerState),
    Candidate(CandidateState),
    Leader(LeaderState),
}

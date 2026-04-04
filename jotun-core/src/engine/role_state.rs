use std::collections::BTreeSet;

use crate::{engine::peer_progress::PeerProgress, types::node::NodeId};

/// Per-role state the follower carries while in the Follower role.
///
/// Tracks the current leader — learned from any `AppendEntries` accepted
/// in the current term — so the host can redirect client proposals that
/// land on this node. `None` means "we haven't heard from a leader this
/// term yet" (just booted, or just stepped down from candidate/leader).
#[derive(Default, Copy, Clone, Debug)]
pub struct FollowerState {
    pub leader_id: Option<NodeId>,
}

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

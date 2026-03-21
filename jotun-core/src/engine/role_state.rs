use crate::engine::peer_progress::PeerProgress;

#[derive(Default, Copy, Clone, Debug)]
pub struct FollowerState {}

#[derive(Default, Copy, Clone, Debug)]
pub struct CandidateState {
    pub votes_granted: usize,
}

#[derive(Default, Clone, Debug)]
pub struct LeaderState {
    pub progress: PeerProgress,
}

#[derive(Debug, Clone)]
pub enum RoleState {
    Follower(FollowerState),
    Candidate(CandidateState),
    Leader(LeaderState),
}

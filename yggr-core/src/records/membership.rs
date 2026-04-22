use std::collections::BTreeSet;

use crate::types::node::NodeId;

/// Cluster membership excluding the local node.
///
/// Today the runtime only populates the voter set. Learners are kept as a
/// first-class part of the model so the engine and runtime can grow into
/// learner support without having to keep overloading "peer set" APIs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Membership {
    voters: BTreeSet<NodeId>,
    learners: BTreeSet<NodeId>,
}

impl Membership {
    #[must_use]
    pub fn new(
        voters: impl IntoIterator<Item = NodeId>,
        learners: impl IntoIterator<Item = NodeId>,
    ) -> Self {
        let voters: BTreeSet<NodeId> = voters.into_iter().collect();
        let mut learners: BTreeSet<NodeId> = learners.into_iter().collect();
        for voter in &voters {
            learners.remove(voter);
        }
        Self { voters, learners }
    }

    #[must_use]
    pub fn with_voters(voters: impl IntoIterator<Item = NodeId>) -> Self {
        Self::new(voters, std::iter::empty())
    }

    #[must_use]
    pub fn voters(&self) -> &BTreeSet<NodeId> {
        &self.voters
    }

    #[must_use]
    pub fn learners(&self) -> &BTreeSet<NodeId> {
        &self.learners
    }

    #[must_use]
    pub fn voter_count(&self) -> usize {
        self.voters.len()
    }

    #[must_use]
    pub fn contains_voter(&self, node: &NodeId) -> bool {
        self.voters.contains(node)
    }

    #[must_use]
    pub fn contains_learner(&self, node: &NodeId) -> bool {
        self.learners.contains(node)
    }

    #[must_use]
    pub fn contains_member(&self, node: &NodeId) -> bool {
        self.contains_voter(node) || self.contains_learner(node)
    }

    pub fn add_voter(&mut self, node: NodeId) {
        self.learners.remove(&node);
        self.voters.insert(node);
    }

    pub fn remove_voter(&mut self, node: NodeId) {
        self.voters.remove(&node);
    }

    pub fn add_learner(&mut self, node: NodeId) {
        if !self.voters.contains(&node) {
            self.learners.insert(node);
        }
    }

    pub fn remove_learner(&mut self, node: NodeId) {
        self.learners.remove(&node);
    }
}

#[cfg(test)]
mod tests {
    use super::Membership;
    use crate::types::node::NodeId;

    fn node(id: u64) -> NodeId {
        NodeId::new(id).unwrap()
    }

    #[test]
    fn voter_membership_starts_without_learners() {
        let membership = Membership::with_voters([node(2), node(3)]);
        assert!(membership.contains_voter(&node(2)));
        assert!(membership.contains_voter(&node(3)));
        assert!(membership.learners().is_empty());
    }

    #[test]
    fn node_cannot_be_voter_and_learner() {
        let membership = Membership::new([node(2), node(3)], [node(3), node(4)]);
        assert!(membership.contains_voter(&node(3)));
        assert!(!membership.contains_learner(&node(3)));
        assert!(membership.contains_learner(&node(4)));
    }
}

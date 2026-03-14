//! all tracing in the engine goes through this module so that field names
//! and targets stay consistent. Downstream consumers can filter by the
//! `jotun::engine` target and rely on stable field keys.

use crate::types::{node::NodeId, term::Term};

/// tracing target for all engine events. use this with
/// `target: telemetry::TARGET` so `RUST_LOG=jotun::engine=debug` works.
pub const TARGET: &str = "jotun::engine";

/// stable field keys. used with `Span::current().record(...)` where
/// the macro syntax doesn't accept constants.
pub mod fields {
    pub const NODE_ID: &str = "node_id";
    pub const CANDIDATE: &str = "candidate";
    pub const PEER: &str = "peer";
    pub const FROM_TERM: &str = "from_term";
    pub const TO_TERM: &str = "to_term";
    pub const DECISION: &str = "decision";
    pub const REASON: &str = "reason";
    pub const ROLE: &str = "role";
}

pub fn term_advanced(node_id: NodeId, from: Term, to: Term) {
    tracing::info!(
        target: TARGET,
        node_id = %node_id,
        from_term = %from,
        to_term = %to,
        "term advanced",
    );
}

pub fn became_follower(node_id: NodeId, term: Term) {
    tracing::info!(
        target: TARGET,
        node_id = %node_id,
        to_term = %term,
        role = "follower",
        "role changed",
    );
}

pub fn became_candidate(node_id: NodeId, term: Term) {
    tracing::info!(
        target: TARGET,
        node_id = %node_id,
        to_term = %term,
        role = "candidate",
        "role changed",
    );
}

pub fn became_leader(node_id: NodeId, term: Term) {
    tracing::info!(
        target: TARGET,
        node_id = %node_id,
        to_term = %term,
        role = "leader",
        "role changed",
    );
}

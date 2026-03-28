use crate::records::{
    append_entries::{AppendEntriesResponse, RequestAppendEntries},
    vote::{RequestVote, VoteResponse},
};

/// All inter-node messages the engine speaks. The wire format
/// ([`crate::transport::protobuf::Message`]) maps one-to-one onto these
/// variants — what comes off the wire is decoded into a `Message<C>`,
/// validated, and then handed to the engine wrapped in
/// [`crate::engine::incoming::Incoming`].
///
/// The four variants form two request/response pairs:
///  - `VoteRequest` / `VoteResponse` — leader election (§5.2).
///  - `AppendEntriesRequest` / `AppendEntriesResponse` — log replication,
///    heartbeats, and commit-index propagation (§5.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message<C> {
    /// A candidate is asking for our vote.
    VoteRequest(RequestVote),
    /// Reply to one of our outgoing `VoteRequest`s.
    VoteResponse(VoteResponse),
    /// A leader is replicating entries (or sending a heartbeat).
    AppendEntriesRequest(RequestAppendEntries<C>),
    /// Reply to one of our outgoing `AppendEntriesRequest`s.
    AppendEntriesResponse(AppendEntriesResponse),
}

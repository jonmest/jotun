# Leadership transfer

`Node::transfer_leadership_to(peer)` asks the current leader to hand leadership to `peer`. Uses:

- Graceful shutdown of the current leader.
- Pinning the leader to a specific node (e.g. for locality).
- Maintenance windows.

```rust
node.transfer_leadership_to(nid(2)).await?;
```

## Protocol

The leader replicates to `peer` until `matchIndex[peer] == last_log_index`, then sends `TimeoutNow`. The target immediately starts an election at `current_term + 1`. Followers with a known leader redirect; leaderless followers and candidates drop.

The call returns once the local driver has accepted the request and dispatched the engine actions. It does not wait for the new leader to be elected. The cluster may briefly have no leader if the target fails to win.

## Errors

- `NotLeader { leader_hint }` — we are a follower.
- `NoLeader` — no leader currently known.
- `InvalidTarget { target }` — `target` is self or not a member of the current peer set.

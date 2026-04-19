# Leadership transfer

`Node::transfer_leadership_to(peer)` asks the current leader to hand leadership off to `peer`. Useful for:

- Graceful shutdown of the leader
- Load balancing (leader pinned to a node you want to drain)
- Planned maintenance

```rust
node.transfer_leadership_to(nid(2)).await?;
```

## What happens

The leader replicates to `peer` until `matchIndex[peer] == last_log_index`, then sends `TimeoutNow`. The target starts an immediate election at `current_term + 1`. Followers with a known leader redirect; leaderless followers and candidates drop.

The call returns once the local driver has accepted the request and dispatched the engine actions — **it does not wait for the new leader to be elected**. The cluster may briefly have no leader if the target fails to win.

## Errors

- `NotLeader { leader_hint }` — we're a follower, not the leader.
- `NoLeader` — no leader currently known.
- `InvalidTarget { target }` — `target` is self or not in the current peer set.

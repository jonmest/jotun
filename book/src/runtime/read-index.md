# Linearizable reads

`Node::read_linearizable` runs a closure against the state machine at a point where the read is guaranteed to see every committed write. No log append, no fsync — just the §8 ReadIndex protocol.

```rust
let value = node.read_linearizable(|sm: &MyState| sm.value).await?;
```

## How it works

1. The leader records `commit_index` as the read's `read_index`.
2. The leader triggers a heartbeat round. Once a majority of peers ack, the leader knows it's still authoritative as of `read_index`.
3. When the state machine has caught up to `read_index`, the closure runs.

## Errors

- `NotLeader { leader_hint }` — retry against `leader_hint`.
- `NotReady` — the leader hasn't committed an entry in its current term yet (the §5.4.2 no-op takes a round to commit after election). Retry in a moment.
- `SteppedDown` — leader lost its role mid-read.

## Why a closure, not a generic read

The state machine lives on a dedicated apply task (see [Writing a state machine](../getting-started/state-machine.md)). The closure is shipped to that task and run in FIFO order with committed apply entries, so the read observes post-apply state, never mid-apply.

# The Node API

`Node<S>` is the user-facing handle to a running Raft node. It's cheap to clone — every clone shares the same underlying driver task.

## Starting

```rust
let node = Node::start(config, state_machine, storage, transport).await?;
```

`Node::start` recovers any persisted state, builds the engine, spawns the driver task, the ticker, and the apply task, then returns. See [Configuration](./config.md) and [Bootstrap modes](./bootstrap.md).

## Core methods

| Method | What it does |
|---|---|
| `propose(cmd)` | Replicate a command. Resolves once it commits and applies. |
| `add_peer(id)` / `remove_peer(id)` | §4.3 single-server membership change. |
| `read_linearizable(closure)` | Run `closure` against the state machine at a linearizable read point (§8 ReadIndex). |
| `transfer_leadership_to(peer)` | Hand off leadership to a specific follower. |
| `status()` | Current role, term, commit index, known leader. |
| `shutdown()` | Drain + stop every background task. |

See the [rustdoc for `Node`](../api/jotun/struct.Node.html) for full signatures and error semantics.

## Error types

- `ProposeError` — `NotLeader`, `NoLeader`, `Busy`, `Shutdown`, `DriverDead`, `Fatal`
- `ReadError` — `NotLeader`, `NotReady`, `SteppedDown`, `Shutdown`, `DriverDead`, `Fatal`
- `TransferLeadershipError` — `NotLeader`, `NoLeader`, `InvalidTarget`, `Shutdown`, `DriverDead`, `Fatal`
- `NodeStartError<E>` — `Config(ConfigError)`, `Storage(E)`

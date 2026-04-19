# Design decisions

## Engine as pure state machine

The engine has no I/O. Every effect is an `Action` the host dispatches. This makes the engine:

- **Deterministic** — same inputs, same outputs, always.
- **Fast to test** — the sim runs thousands of chaos steps per second.
- **Host-agnostic** — write your own host for gRPC, QUIC, or in-process testing.

## Single `step()` method

Every event becomes an `Event`, every effect becomes an `Action`. There is no "handle_request_vote", "handle_append_entries", etc. That design keeps the public surface small and the causal ordering of `Action`s obvious.

## Async apply as a separate task

User `apply()` calls can be slow. If they ran inline on the driver task, heartbeats would stall. Instead, a bounded mpsc channel decouples commit from apply; the state machine lives on its own task. Backpressure is natural: if apply falls behind, the channel fills, the driver blocks on send, and replication slows — which is correct behaviour when the state machine is the bottleneck.

## Snapshots opaque through the stack

Jotun never looks inside a snapshot. Compress, encrypt, or hash-chain inside `StateMachine::snapshot`; undo inside `restore`. The engine, disk, and wire format all treat the bytes as opaque. Keeps the library dependency-free in that axis and gives users the hooks they want.

## ReadIndex via a closure

`Node::read_linearizable` takes an `FnOnce(&S) -> R` that ships to the apply task and runs in FIFO order with applies. This makes the read observe post-apply state, not mid-apply, without requiring the `StateMachine` trait to grow a `read` method.

## `Engine::step` returns actions in causal order

Every `Action` that must reach stable storage before any subsequent `Send` appears earlier in the vector. This is Raft Figure 2's "respond to RPCs only after updating stable storage" — enforced at the contract level so hosts can't accidentally violate it.

## No OpenTelemetry dependency in the library

We emit `tracing` spans and events with stable targets and field names. Users bolt on OTel (or any other tracing backend) in their own `main`. See [Observability](../runtime/observability.md).

## Why membership changes are §4.3 single-server

We chose single-server over Joint Consensus (§6) for simplicity. Single-server has well-understood edge cases and fits our one-at-a-time `add_peer` / `remove_peer` API. Joint Consensus might land post-0.1 if operators need faster reconfiguration.

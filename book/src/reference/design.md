# Design decisions

## Engine as pure state machine

The engine has no I/O. Every effect is an `Action` the host dispatches. This makes the engine:

- Deterministic. Same inputs, same outputs.
- Fast to test. The sim runs thousands of chaos steps per second.
- Host-agnostic. Write your own host for gRPC, QUIC, or in-process testing.

## A single `step()` method

Every event becomes an `Event`, every effect becomes an `Action`. No `handle_request_vote`, `handle_append_entries`, etc. That keeps the public surface small and the causal ordering of `Action`s obvious.

## Async apply as a separate task

User `apply()` calls can be slow. If they ran on the driver task, heartbeats would stall. Instead, a bounded mpsc channel decouples commit from apply; the state machine lives on its own task. Backpressure is natural: if apply falls behind, the channel fills, the driver blocks on send, and replication slows. That's the correct behavior when the state machine is the bottleneck.

## Snapshots opaque through the stack

The library never looks inside a snapshot. Compress, encrypt, or hash-chain inside `StateMachine::snapshot`; undo inside `restore`. The engine, disk, and wire format all treat the bytes as opaque. Keeps the library dependency-free in that axis and gives users the hooks they want.

## ReadIndex via a closure

`Node::read_linearizable` takes an `FnOnce(&S) -> R` that ships to the apply task and runs in FIFO order with applies. The read observes post-apply state, not mid-apply, and the `StateMachine` trait doesn't have to grow a `read` method.

## Causal action ordering

Every `Action` that must reach stable storage before a subsequent `Send` appears earlier in the vector. That's Figure 2's "respond to RPCs only after updating stable storage", enforced at the contract level so hosts can't accidentally violate it.

## No OpenTelemetry dependency

The library emits `tracing` spans and events with stable targets and field names. OTel is bolted on by the user's `main`. See [Observability](../runtime/observability.md). OTel crates churn too fast to pin in a library.

## §4.3 single-server over Joint Consensus

Membership changes are §4.3 single-server. Simpler than Joint Consensus, well-understood edge cases, fits the one-at-a-time `add_peer` / `remove_peer` API. Joint Consensus may land post-0.1 if operators need faster reconfiguration.

# jotun

A Rust Raft library with a pure protocol engine, a deterministic simulator, and a tokio runtime.

[📖 Guide](https://joncm.github.io/logos/guide/) · [🔧 API docs](https://joncm.github.io/logos/api/jotun/) · [💬 Discussions](https://github.com/joncm/logos/discussions)

```rust
use jotun::{Config, Node, DiskStorage, TcpTransport};

let config    = Config::new(my_id, peer_ids);
let storage   = DiskStorage::open(&data_dir).await?;
let transport = TcpTransport::start(my_id, listen_addr, peer_addrs).await?;
let node      = Node::start(config, MyStateMachine::default(), storage, transport).await?;

// Replicate a command and wait for it to commit and apply.
let response = node.propose(MyCmd::Inc(5)).await?;

// Linearizable read (§8 ReadIndex) — no log append, no fsync.
let value = node.read_linearizable(|sm: &MyStateMachine| sm.value).await?;
```

## Why

Most Rust Raft libraries bundle consensus, transport, and storage into one crate. That makes them hard to test against adversarial schedules, hard to embed into a runtime you already have, and hard to audit because the protocol is tangled with I/O.

**jotun splits those apart:**

- [`jotun-core`](./jotun-core) — the pure Raft protocol. One type, `Engine<C>`, one method: `step(Event<C>) -> Vec<Action<C>>`. No sockets, no disk, no async. Deterministic, testable at memory speed.
- [`jotun-sim`](./jotun-sim) — a deterministic cluster simulator that drives drops, reorderings, partitions, crashes, and partial fsync against the engine, with safety invariants checked after every step. Finds bugs you can't find by running real nodes on localhost.
- [`jotun`](./jotun) — batteries-included tokio runtime. `Node`, `DiskStorage`, `TcpTransport`, length-prefixed protobuf wire format. This is what most users want.
- [`jotun-examples`](./jotun-examples) — a three-node replicated KV service, handy for poking at.

## What's in the box

- Leader election, log replication, membership changes (§4.3 single-server)
- Linearizable reads via [ReadIndex](https://joncm.github.io/logos/guide/runtime/read-index.html) (§8)
- Leadership transfer via `TimeoutNow`
- Snapshotting with chunked `InstallSnapshot` and auto-compaction hints
- Segmented on-disk log with crash-safe atomic file writes
- Async state machine apply — slow `apply()` doesn't stall heartbeats
- Opt-in leader proposal batching
- Structured `tracing` spans and events with stable field names (easy OTel wiring; see the [Observability guide](https://joncm.github.io/logos/guide/runtime/observability.html))

## Quick start

Prerequisites: Rust 1.85+, `protoc` installed.

```bash
# Run the three-node KV demo:
./jotun-examples/run-three-node.sh

# Or just run the test suite:
just test          # nextest + doc tests, full workspace
just clippy        # strict: -D warnings
just coverage      # llvm-cov, all-targets
just fuzz-check    # compile every fuzz target (stable)
```

## Correctness

Jotun is tested at four layers:

1. **Unit tests** — 280+ pure-engine tests covering every `Event`/`Action` path.
2. **Property tests** — 1024-case proptests on engine invariants (term monotonicity, commit monotonicity, single leader per term, ...). Mapping codec tests fuzz arbitrary bytes through the wire decoder.
3. **Sim chaos** — 128 cases × 1500 steps for 3-, 5-, and 7-node clusters under drops + reorder + partitions + crashes + partial fsync. Safety invariants (Election Safety, Log Matching, Leader Completeness, State Machine Safety) checked after every step.
4. **Runtime chaos** — real `Node` instances with driver, apply task, and storage, connected through an in-process chaos transport. Asserts the same invariants at the full-stack level.

Plus: [cargo-mutants](https://github.com/sourcefrog/cargo-mutants) sweeps on the correctness-critical modules, [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) targets for the wire codec, the engine, and disk recovery.

**Status.** Pre-1.0. Public API is stable enough to build on but may still change. Real-world feedback welcome via [discussions](https://github.com/joncm/logos/discussions) or issues.

## Docs

- [Introduction & crate layout](https://joncm.github.io/logos/guide/introduction.html)
- [Writing a state machine](https://joncm.github.io/logos/guide/getting-started/state-machine.html)
- [The `Node` API](https://joncm.github.io/logos/guide/runtime/node.html)
- [Writing a custom host](https://joncm.github.io/logos/guide/engine/custom-host.html) (use `jotun-core` without the runtime)
- [Safety invariants the sim checks](https://joncm.github.io/logos/guide/sim/invariants.html)
- [Design decisions](https://joncm.github.io/logos/guide/reference/design.html)

## License

MIT. See [LICENSE.md](./LICENSE.md).

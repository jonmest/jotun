# Introduction

**jotun** is a Rust Raft library split into four crates:

| crate | what it is | what it doesn't do |
|---|---|---|
| `jotun-core` | Pure Raft engine. One type, `Engine<C>`, one method: `step(Event<C>) -> Vec<Action<C>>`. | Touches no sockets, no disk, no async runtime. |
| `jotun-sim` | Deterministic cluster simulator that drives crashes, partitions, drops, reorderings, and partial flushes against `jotun-core` and checks safety invariants after every step. | Not a benchmark. Runs the engine synchronously. |
| `jotun` | Batteries-included tokio runtime. Wraps the engine with `Node`, `DiskStorage`, `TcpTransport`. | Doesn't mandate how you serialize commands — you pick. |
| `jotun-examples` | A replicated KV service built on `jotun`. | Not production-ready; it's a demo. |

## What's implemented

- §5.2 leader election with randomized election timeouts
- §5.3 log replication with `AppendEntries`
- §5.4.1 election restriction (up-to-date candidate check)
- §5.4.2 current-term commit rule (election no-op)
- §4.3 single-server membership changes
- §7 snapshotting with chunked `InstallSnapshot`
- §8 linearizable reads via `ReadIndex`
- Leadership transfer via `TimeoutNow`
- Async apply task (slow `apply()` does not stall heartbeats)
- Opt-in proposal batching on the leader
- Protobuf wire format, TCP transport
- Segmented on-disk log storage with crash-safe atomic file writes

## Who this is for

- **People building a replicated Rust service** and who want `Node`, storage, and transport in the box. Use `jotun`.
- **People who want Raft without runtime assumptions** — for a custom transport, a custom storage, or an embedded-in-something-else integration. Use `jotun-core` directly and write a thin driver; see [Writing a custom host](./engine/custom-host.md).
- **People who want to verify Raft changes against adversarial schedules** — use `jotun-sim` as a test library.

## Status

Pre-1.0. Public API is stable enough to build on but may still change. Expect refinement. All safety invariants are checked in both the simulator (hundreds of chaos seeds per CI run) and at runtime under real tokio tasks with an in-process chaos transport (see [The sim harness](./sim/overview.md)).

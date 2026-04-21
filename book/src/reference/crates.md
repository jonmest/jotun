# Crate layout

| Crate | Purpose |
|---|---|
| `jotun-core` | The engine. `Engine`, `Event`, `Action`, wire types, protobuf mapping. |
| `jotun-sim` | Sim harness. `Cluster`, `Network`, `SafetyChecker`, chaos proptests. |
| `jotun` | Runtime. `Node`, `Config`, `DiskStorage`, `TcpTransport`, `StateMachine` trait. |
| `jotun-examples` | Demos. `kv` three-node replicated KV server. |

## Dependency graph

```
jotun-examples → jotun → jotun-core
                            ↑
                       jotun-sim
```

`jotun-core` does not depend on `jotun`, `jotun-sim`, or any async runtime. `jotun-sim` pulls in `jotun-core` only. `jotun` adds tokio and prost on top of `jotun-core`.

## Why the split

Two reasons.

1. Testability. The sim relies on `jotun-core` being free of tokio, real sockets, and the filesystem. Otherwise we couldn't run thousands of deterministic chaos steps per second.
2. Embeddability. Users who want Raft inside something that's already async take `jotun-core` and write a small host. See [Writing a custom host](../engine/custom-host.md).

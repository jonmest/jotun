# Crate layout

| Crate | Lines of Rust | What lives here |
|---|---|---|
| `jotun-core` | pure engine | `Engine`, `Event`, `Action`, wire types, protobuf mapping |
| `jotun-sim` | sim harness | `Cluster`, `Network`, `SafetyChecker`, chaos proptests |
| `jotun` | runtime | `Node`, `Config`, `DiskStorage`, `TcpTransport`, `StateMachine` trait |
| `jotun-examples` | demos | `kv` 3-node replicated KV server |

## Dependency graph

```
jotun-examples → jotun → jotun-core
                            ↑
                       jotun-sim
```

`jotun-core` has no dependency on `jotun`, `jotun-sim`, or any async runtime. `jotun-sim` pulls in `jotun-core` only. `jotun` adds tokio and prost on top of `jotun-core`.

## Why the split

Two reasons:

1. **Testability.** The sim depends on `jotun-core` being free of tokio, real sockets, and the filesystem. Otherwise we couldn't run thousands of deterministic chaos steps per second.

2. **Embeddability.** Users who want Raft inside something that's already async (e.g. another runtime, or a non-tokio service) take `jotun-core` and write a small host — see [Writing a custom host](../engine/custom-host.md).

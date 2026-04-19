# Observability

Jotun emits structured `tracing` events and spans throughout the engine and runtime. We use stable targets and field names so you can filter and index reliably.

## Stable targets

| Target | What emits |
|---|---|
| `jotun::engine` | Role changes, term advances, vote decisions, AE accept/reject, commit advances. |
| `jotun::node` | Driver-level events: apply failures, transport errors, shutdown. |

## Stable fields

- `node_id` — always the emitting node's id
- `term` / `from_term` / `to_term` — term transitions
- `role` — `"follower" | "candidate" | "leader"`
- `decision` — on vote handling, `"granted" | "rejected"`

## OpenTelemetry

Jotun doesn't pull in the `opentelemetry` crates itself. Instead, wire your own subscriber in your service's `main`:

```rust
use tracing_subscriber::prelude::*;

let otlp = opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_exporter(opentelemetry_otlp::new_exporter().tonic())
    .install_batch(opentelemetry::runtime::Tokio)?;

tracing_subscriber::registry()
    .with(tracing_opentelemetry::layer().with_tracer(otlp))
    .with(tracing_subscriber::fmt::layer())
    .with(tracing_subscriber::EnvFilter::from_default_env())
    .init();
```

Every `jotun::engine` span now flows to your OTLP collector.

## Log filters

Quick-start filters for `RUST_LOG`:

- `RUST_LOG=jotun=debug` — everything jotun emits at debug or above
- `RUST_LOG=jotun::engine=info` — just protocol decisions
- `RUST_LOG=jotun::node=debug,jotun::engine=info` — runtime detail, protocol overview

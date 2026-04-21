# Observability

The engine and runtime emit structured `tracing` events and spans with stable targets and field names.

## Targets

| Target | What emits |
|---|---|
| `jotun::engine` | Role changes, term advances, vote decisions, AppendEntries accept/reject, commit advances. |
| `jotun::node` | Driver-level events: apply failures, transport errors, shutdown. |

## Fields

- `node_id` — the emitting node
- `term` / `from_term` / `to_term` — term transitions
- `role` — `"follower" | "candidate" | "leader"`
- `decision` — on vote handling, `"granted" | "rejected"`

## OpenTelemetry

The library doesn't depend on `opentelemetry` directly. The OTel crates churn fast, and pinning them in a library creates version conflicts for users. Wire your subscriber in your service's `main`:

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

`jotun::engine` spans and events flow to the collector.

## Log filters

Quick-start `RUST_LOG` values:

- `jotun=debug` — everything jotun emits at debug or above
- `jotun::engine=info` — just protocol decisions
- `jotun::node=debug,jotun::engine=info` — runtime detail, protocol overview

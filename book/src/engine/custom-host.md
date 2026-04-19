# Writing a custom host

The batteries-included `jotun` runtime assumes tokio, a tcp transport, and disk storage. If those don't fit — maybe you're embedding Raft into an existing async runtime, or replicating over QUIC, or persisting into an existing storage engine — you can drive `jotun-core` yourself.

A minimal host loop:

```rust
use jotun_core::{Engine, Event, Action};

let mut engine: Engine<MyCmd> = Engine::new(my_id, peers, env, heartbeat_ticks);

loop {
    let event = next_event().await?;  // your ticker / your transport / your app
    let actions = engine.step(event);

    for action in actions {
        match action {
            Action::PersistHardState { current_term, voted_for } => {
                my_storage.persist_hard_state(current_term, voted_for).await?;
            }
            Action::PersistLogEntries(entries) => {
                my_storage.append_log(entries).await?;
            }
            Action::PersistSnapshot { bytes, .. } => {
                my_storage.persist_snapshot(bytes).await?;
            }
            Action::Send { to, message } => {
                my_transport.send(to, message).await?;
            }
            Action::Apply(entries) => {
                for entry in entries { my_sm.apply(entry).await; }
            }
            Action::ApplySnapshot { bytes } => {
                my_sm.restore(bytes).await;
            }
            _ => {}
        }
    }
}
```

## Rules to follow

1. **Respect action order.** See [Events and actions](./events-and-actions.md). Fsync `Persist*` before any subsequent `Send`.
2. **Feed `Tick` at a steady cadence.** The engine's election and heartbeat timers are tick-based. Pick whatever wall-clock duration makes sense for your latency budget; the default runtime uses 50ms.
3. **Hydrate the engine on restart.** Read your persisted hard state, snapshot (if any), and post-snapshot log, then call `Engine::recover_from(RecoveredHardState { .. })`. Do this BEFORE feeding any real events.

See the source of `jotun/src/node.rs` and `jotun-sim/src/harness.rs` for two working reference hosts.

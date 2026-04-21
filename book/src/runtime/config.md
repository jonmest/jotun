# Configuration

`Config` is a plain struct with public fields. Defaults come from `Config::new(node_id, peers)`.

```rust
let mut config = Config::new(node_id, peers);
config.election_timeout_min_ticks = 10;
config.election_timeout_max_ticks = 20;
config.heartbeat_interval_ticks = 3;
config.tick_interval = Duration::from_millis(50);
```

## Timing

| Field | Default | Note |
|---|---|---|
| `election_timeout_min_ticks` | 10 | §5.2 minimum election timeout. |
| `election_timeout_max_ticks` | 20 | Exclusive. Actual timeout is uniform in `[min, max)`. |
| `heartbeat_interval_ticks` | 3 | Leader heartbeat interval. Must be `< min`. |
| `tick_interval` | 50ms | Wall-clock duration of one engine tick. |

The engine is tick-driven. A tick is whatever you say it is.

## Backpressure

| Field | Default | Note |
|---|---|---|
| `max_pending_proposals` | 1024 | `propose` / `add_peer` / `remove_peer` return `Busy` above this in-flight count. |
| `max_pending_applies` | 4096 | Capacity of the driver → apply-task channel. When full, the driver awaits space. |

## Batching

Off by default. Turn it on if you have many concurrent proposals.

| Field | Default | Note |
|---|---|---|
| `max_batch_delay_ticks` | 0 (off) | Hold proposals for up to this many ticks before flushing. |
| `max_batch_entries` | 64 | Flush immediately when the buffer reaches this size. |

With batching on, N concurrent `propose` calls can commit in a single broadcast and fsync.

## Snapshotting

| Field | Default | Note |
|---|---|---|
| `snapshot_hint_threshold_entries` | 1024 | The engine emits `Action::SnapshotHint` every time this many entries have been applied past the current floor. Set to `0` to disable. |
| `snapshot_chunk_size_bytes` | 64 KiB | Maximum bytes per `InstallSnapshot` chunk. |

See [rustdoc for `Config`](../api/jotun/struct.Config.html) for the full type.

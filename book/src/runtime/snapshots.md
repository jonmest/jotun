# Snapshots

Raft snapshots (§7) let a node discard the prefix of its log by capturing the state machine's state and the log position it reflects. Jotun handles the protocol end-to-end: fast followers catch up via `AppendEntries`, followers whose `nextIndex` has fallen below the snapshot floor catch up via `InstallSnapshot`.

## Two triggers

1. **Host-initiated.** Your application calls something that causes the driver to step the engine with `Event::SnapshotTaken { last_included_index, bytes }`. In the batteries-included runtime, this happens automatically via `Action::SnapshotHint` when enough entries have applied past the current floor.

2. **Follower catch-up.** A leader whose peer's `nextIndex <= snapshot_floor` sends an `InstallSnapshot` instead of `AppendEntries`. Jotun transfers the snapshot in chunks of `Config::snapshot_chunk_size_bytes`.

## Auto-compaction hints

The engine emits `Action::SnapshotHint { last_included_index }` every time the applied-entries count past the current floor crosses `Config::snapshot_hint_threshold_entries`. The default runtime reacts by calling `StateMachine::snapshot()` and feeding the bytes back via `Event::SnapshotTaken`. Set `snapshot_hint_threshold_entries = 0` to disable auto-snapshotting.

## Compression

Jotun treats snapshot bytes as opaque. If you want compression, do it inside `StateMachine::snapshot` and undo it inside `restore`:

```rust
fn snapshot(&self) -> Vec<u8> {
    zstd::encode_all(&self.serialized[..], 3).unwrap()
}
fn restore(&mut self, bytes: Vec<u8>) {
    let decoded = zstd::decode_all(&bytes[..]).unwrap();
    self.deserialize(&decoded);
}
```

## Membership and snapshots

Snapshots carry the cluster membership as of `last_included_index`. Without this, committed `AddPeer` / `RemovePeer` entries that get snapshotted would be lost on restart and the node would compute the wrong majority. The runtime stores the peer list alongside the snapshot bytes via `StoredSnapshot::peers`.

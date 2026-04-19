# Events and actions

See the [rustdoc for `Event`](../api/jotun_core/enum.Event.html) and [`Action`](../api/jotun_core/enum.Action.html) for the full definitions; this chapter is the shape-of-the-contract summary.

## Action ordering contract

`step()` returns `Vec<Action<C>>` in **causal order**. Every action that must reach stable storage before any subsequent network send appears earlier in the vector. Hosts MUST:

- Process actions in order
- Flush `PersistHardState` / `PersistLogEntries` / `PersistSnapshot` to disk before performing any `Send` that follows them

This is Raft Figure 2's "respond to RPCs only after updating stable storage" — the engine enforces the ordering on the host side of the contract.

## Actions that don't need fsync

- `Send { .. }` — best-effort; engine retries on its own cadence.
- `Apply(entries)` — feed to the state machine. The engine advances `last_applied` when emitting this; the host must not skip.
- `ApplySnapshot { bytes }` — restore the state machine with these bytes.
- `Redirect { leader_hint }` — wrap the user's call's pending reply with `NotLeader`.
- `SnapshotHint { .. }` — advisory. Ignore, debounce, or act.
- `ReadReady { id }` — the read with this id is now safe to serve against the applied state.
- `ReadFailed { id, reason }` — surface the reason to the caller.

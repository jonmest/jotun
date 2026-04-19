# The pure engine

`jotun-core` is the Raft protocol with nothing else. One type, one method:

```rust
pub struct Engine<C> { /* ... */ }

impl<C: Clone> Engine<C> {
    pub fn step(&mut self, event: Event<C>) -> Vec<Action<C>>;
}
```

Every forward motion in Raft becomes an `Event`:

- `Tick` — abstract time advanced one step
- `Incoming(msg)` — a peer sent us an RPC
- `ClientProposal(cmd)` — application wants to replicate something
- `ClientProposalBatch(cmds)` — same, many at once
- `ProposeConfigChange(change)` — §4.3 membership change
- `TransferLeadership { target }` — initiate leadership transfer
- `ProposeRead { id }` — §8 linearizable read
- `SnapshotTaken { last_included_index, bytes }` — host cut a snapshot

Every effect the engine wants the host to carry out becomes an `Action`:

- `PersistHardState`, `PersistLogEntries`, `PersistSnapshot`
- `Send { to, message }`
- `Apply(entries)` — commit these to the state machine
- `ApplySnapshot { bytes }` — restore the state machine
- `Redirect { leader_hint }` — we're not the leader
- `ReadReady { id }` / `ReadFailed { id, reason }`
- `SnapshotHint { last_included_index }` — advisory compaction

## No I/O

The engine never touches a socket, a file, or the clock. The host (either `jotun` or your own code) translates `Action`s into I/O.

## No async

`step` is synchronous. It returns `Vec<Action<C>>` and that's it. The host decides how to dispatch them.

## Testable

Because the engine has no I/O, it runs at memory speed under the deterministic chaos harness in `jotun-sim`. Hundreds of chaos seeds per CI run, each running thousands of steps, all in under a second.

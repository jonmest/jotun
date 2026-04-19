# The sim harness

`jotun-sim` drives `jotun-core` under adversarial schedules and checks safety after every step. It exists because Raft's subtle bugs live at the interleaving between events, and you can't unit-test your way to coverage of an N-node state space.

## What the sim can do

- Drive `Tick`, `ClientProposal`, and RPC delivery across N nodes
- Drop messages between any pair of nodes, probabilistically
- Reorder message delivery
- Partition the cluster (arbitrary subsets on each side)
- Crash a node and recover it from its durable state
- Flush only a prefix of the pending-write queue (mid-fsync crash)
- Do all of the above under a single deterministic seed, so failures shrink and reproduce

## Running it

Sim tests live under `jotun-sim/src/tests/`:

- `smoke.rs` — happy-path sanity
- `chaos.rs` — proptests with drops + reorder + partitions + crashes + partial fsync
- `membership.rs` — add/remove peers
- `snapshot.rs` — install-snapshot catch-up
- `proptests.rs` — happy-policy proptests asserting liveness (leader elected, majority commits)

Run everything:

```bash
cargo nextest run -p jotun-sim
```

## Budgets

The chaos proptests default to 128 cases × 1500 steps for 3-node and 5-node clusters, and 128 cases × 2000 steps for 7-node. Any regression seed the shrinker finds is persisted in `jotun-sim/proptest-regressions/` and replayed on every future run.

See [Safety invariants](./invariants.md) for what's actually checked.

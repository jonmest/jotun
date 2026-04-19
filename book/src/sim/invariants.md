# Safety invariants

The sim asserts these after every step. A violation fails the run immediately and prints the exact schedule that produced it so the bug is reproducible.

## Election Safety (§5.2)

At most one node can be `Leader` at a given term, across the whole history of the run. Implemented as a cumulative `{term -> leader_id}` map — second distinct leader in the same term panics.

## Log Matching (§5.3)

For any two nodes and any index present in both logs, if the entries at that index have the same term then all prior entries are identical. The sim checks this pairwise over every snapshot of node state.

## Leader Completeness (§5.4)

Every committed entry appears in the logs of every subsequent leader for higher terms. Checked by tracking `(term, last_committed_index)` pairs and asserting no later leader ever replaces them.

## State Machine Safety (§5.4.3)

Two nodes applying the same index must apply the same command. The sim's `TrackedCounter` harness records `(node, index, cmd)` tuples and rejects any mismatch.

## Persist-before-Send

Any `Send` action that references a term or log entry must have been preceded (at some earlier step) by a `PersistHardState` / `PersistLogEntries` action covering that reference. The sim's `check_send_ordering` enforces this.

## Single in-flight ConfigChange (§4.3)

A leader may have at most one uncommitted `ConfigChange` in its log at any time. The sim checks this every step.

## Snapshot within commit

A snapshot's `last_included_index` must be ≤ `commit_index` at the time of persist. Host-side protocol compliance.

---

All of these hold under the default chaos policy (drops + reorder + partition + crash + partial fsync). Liveness — "a leader eventually wins" / "a proposal eventually commits" — is not asserted under chaos (the scheduler is allowed to permanently partition); it gets its own narrower proptest under the `happy` policy.

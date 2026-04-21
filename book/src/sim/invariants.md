# Safety invariants

The sim asserts these after every step. A violation fails the run immediately and prints the schedule that produced it.

## Election Safety (§5.2)

At most one node can be `Leader` at a given term across the run. Implemented as a cumulative `{term -> leader_id}` map; a second distinct leader in the same term panics.

## Log Matching (§5.3)

For any two nodes and any index in both logs, if the entries have the same term, all prior entries are identical. Checked pairwise over every snapshot of node state.

## Leader Completeness (§5.4)

Every committed entry appears in the logs of every subsequent leader for higher terms. Tracked via `(term, last_committed_index)` pairs; no later leader may replace them.

## State Machine Safety (§5.4.3)

Two nodes applying the same index must apply the same command. The `TrackedCounter` harness records `(node, index, cmd)` tuples and rejects mismatches.

## Persist-before-Send

Any `Send` that references a term or log entry must have been preceded by a `PersistHardState` / `PersistLogEntries` covering that reference. `check_send_ordering` enforces this.

## Single in-flight ConfigChange (§4.3)

A leader may have at most one uncommitted `ConfigChange` in its log at any time. Checked every step.

## Snapshot within commit

A snapshot's `last_included_index` must be `<= commit_index` at the time of persist. Host-side protocol compliance.

---

All of these hold under the chaos policy (drops, reorder, partitions, crashes, partial fsync). Liveness — "a leader eventually wins" or "a proposal eventually commits" — is not asserted under chaos; the scheduler is allowed to permanently partition. It gets its own proptest under the happy policy.

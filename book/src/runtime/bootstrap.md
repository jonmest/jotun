# Bootstrap modes

`Bootstrap` controls how a node comes up. The point is to prevent operators from accidentally booting a would-be joiner as a fresh single-node cluster, which would let it elect itself, commit entries, then try to rejoin a real cluster it has diverged from.

| Variant | When to use |
|---|---|
| `Bootstrap::NewCluster { members }` | First boot of a brand-new cluster. Every founding node starts with the same `members` set. |
| `Bootstrap::Join` | Adding a node to a running cluster. Starts with an empty peer set; the existing leader calls `add_peer` to splice it in. |
| `Bootstrap::Recover` | Normal restart. Membership comes from the persisted snapshot and replayed `ConfigChange` entries. |

`Config::new(node_id, peers)` picks `NewCluster` when `peers` is non-empty and `Recover` otherwise. Override if you need `Join`.

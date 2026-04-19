# Bootstrap modes

`Bootstrap` controls how a node comes up at `Node::start` time. It stops operators accidentally booting a would-be joiner as a fresh single-node cluster — which would let it elect itself, commit entries, then try to rejoin a real cluster it's diverged from.

Three variants:

| Variant | When to use |
|---|---|
| `Bootstrap::NewCluster { members }` | First boot of a brand-new cluster. Every node must be started with the same `members` set. |
| `Bootstrap::Join` | Adding a new node to a running cluster. Starts with empty peers; the existing leader will splice this node in via `add_peer`. |
| `Bootstrap::Recover` | Normal restart. The node replays its persisted state; membership comes from the snapshot + replayed `ConfigChange` entries. |

`Config::new(node_id, peers)` picks `NewCluster` when `peers` is non-empty and `Recover` otherwise. Override when needed.

# A three-node cluster

The `jotun-examples` crate ships a replicated KV service. The quickest way to see jotun running is:

```bash
./jotun-examples/run-three-node.sh
```

That script brings up three `kv` processes on localhost, has them elect a leader, and exposes a simple text protocol you can drive with `nc`.

## What the script does

Each node is started with:

```rust
let config = Config::new(my_node_id, peer_ids);
let storage = DiskStorage::open(&data_dir).await?;
let transport = TcpTransport::start(my_node_id, listen_addr, peer_addrs).await?;
let node = Node::start(config, MyStateMachine::default(), storage, transport).await?;
```

From there, the leader accepts `propose(cmd)` calls from clients; followers redirect.

## Next steps

- [Writing a state machine](./state-machine.md) — how to implement the `StateMachine` trait for your own commands
- [The Node API](../runtime/node.md) — propose, read_linearizable, status, shutdown

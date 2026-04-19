# Writing a state machine

Your `StateMachine` is the application-level thing Raft replicates. You implement three required methods and (optionally) two more for snapshots.

```rust
use jotun::{DecodeError, StateMachine};

#[derive(Debug, Default)]
struct Counter { value: u64 }

#[derive(Debug, Clone)]
enum CountCmd { Inc(u64) }

impl StateMachine for Counter {
    type Command = CountCmd;
    type Response = u64;

    fn encode_command(c: &CountCmd) -> Vec<u8> {
        match c {
            CountCmd::Inc(n) => n.to_le_bytes().to_vec(),
        }
    }

    fn decode_command(bytes: &[u8]) -> Result<CountCmd, DecodeError> {
        let arr: [u8; 8] = bytes.try_into()
            .map_err(|_| DecodeError::new("expected 8 bytes"))?;
        Ok(CountCmd::Inc(u64::from_le_bytes(arr)))
    }

    fn apply(&mut self, cmd: CountCmd) -> u64 {
        match cmd {
            CountCmd::Inc(n) => { self.value += n; self.value }
        }
    }
}
```

## Rules

1. **`apply` must be deterministic.** The same `Command` on any node MUST produce the same `Response` and the same state mutation. Otherwise the cluster diverges.
2. **`encode_command` and `decode_command` must round-trip.** Jotun stores the bytes in the log and on the wire; your decoder must accept its own encoder's output.
3. **`apply` runs on its own tokio task.** Slow work won't stall heartbeats, but it does apply backpressure on replication if the driver → apply channel fills up (see [Configuration](../runtime/config.md)).

## Snapshots

Override `snapshot` and `restore` if your state machine has state that would take a long time to replay from the log:

```rust
fn snapshot(&self) -> Vec<u8> {
    bincode::serialize(&self.value).unwrap()
}

fn restore(&mut self, bytes: Vec<u8>) {
    self.value = bincode::deserialize(&bytes).unwrap();
}
```

Compression is your call — compress inside `snapshot`, decompress inside `restore`. Jotun treats snapshot bytes as opaque through the engine, disk, and wire.

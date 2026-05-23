# groot2_client

A minimal Rust client for [`BT::Groot2Publisher`](https://github.com/BehaviorTree/BehaviorTree.CPP/blob/master/include/behaviortree_cpp/loggers/groot2_publisher.h),
the ZMQ REP server that ships with BehaviorTree.CPP 4.x.

This crate intentionally implements only the read paths a visualizer needs:

- `T` — full tree XML
- `S` — node status snapshot

Hook / blackboard / recording opcodes are exposed as enum variants (so users
can extend without forking) but no high-level wrappers are provided.

## Wire format (verified against BT.CPP 4.9.0)

`groot2_protocol.h` is the source of truth. The byte layout is:

### `RequestHeader` — 6 bytes

```
offset 0: protocol  (u8)   == 2  (kProtocolID)
offset 1: type      (u8)   ASCII opcode ('T', 'S', ...)
offset 2: unique_id (u32 little-endian)
```

### `ReplyHeader` — 22 bytes

```
offset  0..6 : RequestHeader echoed back
offset  6..22: tree_uuid [u8; 16]
```

### Reply payloads

| Opcode | Payload (frame 2)                                                                |
|--------|----------------------------------------------------------------------------------|
| `T`    | UTF-8 BT XML string (the full factory tree definitions)                          |
| `S`    | Packed `(u16 uid LE, u8 status)` triples; total length must be a multiple of 3   |
| `t`    | Recording transitions: 9 bytes per entry — `[ts_usec u48, uid u16 LE, status u8]`|
| error  | First frame is the literal `b"error"`, second frame is a UTF-8 message           |

### Status byte encoding

| Code | Meaning                                |
|------|----------------------------------------|
| 0    | IDLE                                   |
| 1    | RUNNING                                |
| 2    | SUCCESS                                |
| 3    | FAILURE                                |
| 4    | SKIPPED                                |
| 10   | IDLE (was IDLE — rare)                 |
| 11   | IDLE (was RUNNING)                     |
| 12   | IDLE (was SUCCESS)                     |
| 13   | IDLE (was FAILURE)                     |
| 14   | IDLE (was SKIPPED)                     |

`NodeStatus::decode` collapses 10..=14 to `Idle`. Use
`NodeStatus::decode_with_prior` if the prior status matters (animation,
transition logging).

## Heartbeat

The publisher disconnects clients that fall silent for
`max_heartbeat_delay_ms` (default 5000). Any valid request resets the timer,
so a periodic STATUS poll is sufficient — no dedicated keepalive opcode.

## Example

```rust
use groot2_client::{Groot2Client, NodeStatus};
use std::time::Duration;

let mut client = Groot2Client::local(1667);
client.set_timeout(Duration::from_secs(2));

let xml = client.request_full_tree()?;
println!("tree definition: {} bytes", xml.len());

let map = client.request_status_map()?;
for (uid, status) in map {
    if status == NodeStatus::Running {
        println!("node #{uid} is RUNNING");
    }
}
# Ok::<_, groot2_client::ClientError>(())
```

## Live tests

```sh
# 1. Start a BT.CPP server (e.g. sentry_behavior). Send any goal so a tree
#    is instantiated and Groot2Publisher binds 1667.
# 2. From this crate:
cargo test --manifest-path crates/groot2_client/Cargo.toml -- --ignored
```

The two `#[ignore]`-d tests print the FULLTREE byte size and the STATUS map
breakdown. Both fail fast with a `zmq::Error::EAGAIN` if the publisher is
not listening.

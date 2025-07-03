# DriveShaft

**A minimal, high-performance thread pool for running synchronous tasks with per-thread context from async code.**

**NOTE: This repository is under heavy development**

---

## What is DriveShaft?

DriveShaft is a lightweight thread pool designed to **bridge the gap between async and sync Rust**. It lets you **spawn synchronous closures** that execute on dedicated worker threads — each thread holding a long-lived, mutable context of your choosing (e.g. a RocksDB instance, a database connection, or some custom state).

This is especially useful when integrating **blocking libraries** (like RocksDB, SQLite, image processing, etc.) into an async application without blocking the runtime.

---

## Key Features

- **Run sync closures from async code** with `.run_with(...)`
- **Custom per-thread context** (`&mut T`) — no `Arc<Mutex<_>>` needed
- **Bounded or unbounded channels** for backpressure or unbounded throughput
- **Round-robin load balancing** across threads
- **Returns results via async `await`**
- Foundation for higher-level wrappers (TBD)

---

## Example

```rust
use driveshaft::DriveShaftPool;

#[tokio::main]
async fn main() {
    // Create 4 contexts — one for each worker
    let ctxs: Vec<_> = (0..4)
        .map(|_| create_context())
        .collect();

    // Spawn a pool with those contexts
    let mut pool = DriveShaftPoolBuilder::new()
        .worker_type(driveshaft::WorkerType::Bound(32)) // 32-slot queue per worker
        .build(ctxs);

    // Run a blocking call from async context
    let result = pool
        .run_with(|ctx| ctx.do_something())
        .await;

    println!("Value: {:?}", result);
}
```

## Use Cases

Wrapping blocking libraries like:

  * RocksDB

  * SQLite

  * LMDB

  * Image/audio/video codecs

Maintaining one connection/context per thread

Avoiding lock contention from shared global state

Offloading CPU-heavy tasks while preserving &mut access

## How It Works

Each thread owns its own T context.

Tasks are FnOnce(&mut T) closures.

Results are sent back via a oneshot channel.

A DriveShaftPool distributes jobs across threads using round-robin.

## Crate Roadmap

 -[X] Per-thread context ownership

 -[X] run_with async API

 -[X] Bounded / unbounded channel modes

-[] Graceful shutdown

-[] Retry / backpressure handling

-[] Metrics + tracing

-[] Runtime-agnostic executor trait

## Contributing

Feedback and contributions are welcome.

## License

MIT or Apache-2.0 — your choice.

# DriveShaft

**A minimal, high-performance thread pool for running synchronous tasks with per-thread context from async code.**

**NOTE: This crate is under heavy development. Expect breaking API changes.**

---

## What is DriveShaft?

DriveShaft is a lightweight thread pool designed to **bridge the gap between async and sync Rust**. It lets you **spawn synchronous closures** that execute on dedicated worker threads — each thread holding a long-lived, mutable context of your choosing (e.g. a RocksDB instance, a database connection, or some custom state).

This is especially useful when integrating **blocking libraries** (like RocksDB, SQLite, image processing, etc.) into an async application without blocking the runtime or when you need a simple way to spawn cpu intense operations from your async runtime.

## Summary of Benchmarks

Generally, when the async runtime is under heavy load:
| Task Type | spawn blocking | driveshaft         | direct on runtime        |
| --------- | --------------- | ------------------ | ---------------- |
| CPU-heavy | ❌ a few times slower   | ✅ Fastest          | ⚠️ Poor scaling  |
| IO-heavy  | ✅ Best          | ⚠️ Slightly behind | ❌ Extremely slow |

As the load decreases, driveshaft and spawn_blocking tend to even out for CPU-heavy workloads.

As with any benchmarks, you should run them yourself using a workload similar to how you might use driveshaft. You can find some sample code to get you started [here](https://github.com/ctgallagher4/driveshaft-bench-examples). The machine you use to benchmark should have a few more cores available than the number of threads you plan to benchmark with for a meaningful assessment.

---

## Key Features

- **Run sync closures from async code** with `.run_with(...)`
- **Custom per-thread context** (`&mut T`) — no `Arc<Mutex<_>>` needed
- **Work Stealing** across threads
- **Returns results via async `await`**
- Foundation for higher-level wrappers (TBD)

---

## Example

```rust
use driveshaft::DriveShaftPool;

#[tokio::main]
async fn main() {
    // Create 4 contexts — one for each worker
    let ctxs: VecDeque<_> = (0..4)
        .map(|_| create_context())
        .collect();

    // Spawn a pool with those contexts
    let mut pool = DriveShaftPool::new(ctxs);

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

Results are sent back by implementing Future.

A DriveShaftPool distributes jobs across threads using `crossbeam::deque`.

## Crate Roadmap

 - [X] Per-thread context ownership

 - [X] run_with async API

 - [X] Bounded / unbounded channel modes

 - [ ] Graceful shutdown

 - [ ] Retry / backpressure handling

 - [ ] Metrics + tracing

 - [ ] Runtime-agnostic executor trait

## Contributing

Feedback and contributions are welcome.

## License

MIT or Apache-2.0 — your choice.

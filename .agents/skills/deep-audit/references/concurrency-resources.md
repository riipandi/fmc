# Concurrency and Resource Leaks

## Races and concurrency

Look for state read or written from more than one thread or task without clear synchronization.

- Shared mutable state: globals, statics, caches shared with worker threads, non-thread-safe collections, counters.
- Check-then-act on the file system: "path exists -> open it", "is directory -> read it", "stat -> rename". The file can change between the two calls; handle the error from the second call instead of trusting the first.
- Stale results: a background job (directory walk, `git` call, file load) that finishes after the user has moved on and writes into the current view. Results carry the request they answer, and a stale result is dropped.
- Unsafe publication: objects exposed to another thread before they are fully initialized; incorrect double-checked locking.
- Deadlock and starvation: nested locks, channels without a consumer, unbounded queues with no backpressure.
- Blocking on the input or render path: file reads of large files, a `git` subprocess, a full directory walk. Any of them can freeze the screen; they belong on a worker with results sent back.

Verification: `cargo miri test` where the code has `unsafe`; a thread sanitizer where the toolchain supports it; parallel stress runs with high iteration counts; tests that force an interleaving (a barrier or a channel handshake).

## Memory, handles, and resources

- Resources opened but not closed on every path, including error paths: files, directory handles, subprocess pipes and their children, temporary files.
- Threads and tasks with no termination condition: a loop with no shutdown signal, a join handle dropped.
- Caches and maps that grow without bound (no size cap, no eviction): per-directory caches, syntax trees, search results, the jump history.
- Reading whole files into memory where a window or the first megabyte is enough.
- Terminal state: raw mode, the alternate screen, and mouse capture restored on normal exit, on error, and on panic. A panic that leaves the terminal in raw mode is a defect.
- Graceful shutdown: background work stopped, the terminal restored, and nothing left running after the app exits.
- Timeouts: every subprocess call (`git`) has a timeout and a way to be abandoned when the user leaves the view.

Verification: measure resident memory and thread count before and after repeated navigation; count open file descriptors before and after; check for a child process left after exit; run a short soak test that opens and closes views in a loop.

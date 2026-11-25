# Rust

updated: 2026-11-26T03:19:42+07:00

Rust contracts and traps for the `fmc` crate. Gates and commands: `shape.md`. Cross-area rules, including parity with Scouter: `rules.md`.

## Toolchain facts

- Edition 2024 (`Cargo.toml:3`). Rules for new or changed code:
  - Every unsafe operation sits in an explicit `unsafe {}` block, even inside an `unsafe fn`; `extern` blocks are `unsafe extern`.
  - `gen` is a reserved keyword.
  - Tail-expression temporaries and `if let` scrutinee drop order changed in 2024: check them when a lock guard or a borrow is involved.
- `opentui_rust` 0.2.3 is the only dependency (`Cargo.lock:25-27`). Read its API from the locked source; docs.rs for 0.2.3 is the second source.

## Code rules

- No `.unwrap()` or `.expect()` in non-test code unless the panic is a true invariant, marked `// INVARIANT: ...`.
- `unsafe` only with a `// SAFETY: ...` comment; list each one in the change summary.
- A comment carries a why, a contract, or a trap; never the next line.

## Contracts and traps

None recorded yet. Record the first one here when the terminal loop lands (for example, terminal state restored on normal exit and on panic).

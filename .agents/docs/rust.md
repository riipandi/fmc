# Rust

updated: 2026-11-26T04:09:25+07:00

Rust contracts and traps for the `fmc` crate. Gates and commands: `shape.md`. Cross-area rules, including parity with Scouter: `rules.md`.

## Toolchain facts

- Edition 2024 (`Cargo.toml:3`). Rules for new or changed code:
  - Every unsafe operation sits in an explicit `unsafe {}` block, even inside an `unsafe fn`; `extern` blocks are `unsafe extern`.
  - `gen` is a reserved keyword.
  - Tail-expression temporaries and `if let` scrutinee drop order changed in 2024: check them when a lock guard or a borrow is involved.
- `opentui_rust` 0.2.3 is the only dependency today (`Cargo.lock:25-27`). `plan-00001` phase 3 replaces it with `opentui` and `opentui-sys` =0.9.9 (`architecture.md`). Read the API from the locked source; docs.rs for the locked version is the second source.

## Layout rules

The layout and its reasoning: `architecture.md` ("a pure core, I/O services, and the UI"). These rules bind once `plan-00001` phase 5 creates the modules.

- The core (`src/lib.rs` and its modules: `keys/`, `explorer/`, `reader/`, `find/`, `diff/`, `theme.rs`) has no I/O and does not import `opentui`. It takes values and returns values.
- A keymap returns `Action`s; `src/app/` carries them out.
- No file I/O and no process spawn on the UI thread. Workers in `src/services/` do it and send `Event`s back.
- Keep every file under 800 lines.
- Tests go in a sibling `tests.rs` or `*_tests.rs` file, not in an inline `mod tests` block of a large file.

## Code rules

- No `.unwrap()` or `.expect()` in non-test code unless the panic is a true invariant, marked `// INVARIANT: ...`.
- `unsafe` only with a `// SAFETY: ...` comment; list each one in the change summary.
- A comment carries a why, a contract, or a trap; never the next line.

## Contracts and traps

None recorded yet. Record the first one here when the terminal loop lands (for example, terminal state restored on normal exit and on panic).

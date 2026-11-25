# Rust

updated: 2026-11-26T04:47:14+07:00

Rust contracts and traps for the `fmc` crate. Gates and commands: `shape.md`. Cross-area rules, including parity with Scouter: `rules.md`.

## Toolchain facts

- Edition 2024 (`Cargo.toml:3`). Rules for new or changed code:
  - Every unsafe operation sits in an explicit `unsafe {}` block, even inside an `unsafe fn`; `extern` blocks are `unsafe extern`.
  - `gen` is a reserved keyword.
  - Tail-expression temporaries and `if let` scrutinee drop order changed in 2024: check them when a lock guard or a borrow is involved.
- The engine is `opentui` and `opentui-sys` =0.9.9 with the `tree-sitter` feature (`Cargo.toml`). Read the API from the locked source; docs.rs for the locked version is the second source.
- `rust-version = "1.90"`: the floor set by `tree-sitter` 0.27.2; `cargo +1.90 check` passed on 2026-11-26. The pinned toolchain (`rust-toolchain.toml`) is newer.

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

- **Terminal restore is OpenTUI's.** `App::run` enters the alternate screen and leaves it on return; the host installs a panic hook that restores the terminal before the panic message prints (`opentui-0.9.9/src/host.rs`, `install_panic_hook`). fmc adds no hook of its own. `tests/cli.rs` checks both paths; the panic path uses the debug-only `FMC_PANIC_AFTER_START` variable (`src/app/mod.rs`), compiled out of release builds.
- **The root keeps the terminal's background.** OpenTUI's themes fill the root with a color (`#12161c` dark); Scouter draws on the default background, so `app::run` sets both themes' `background` to `Color::DEFAULT_BACKGROUND`.
- **Keys are read in capture.** `on_key_capture` sees every key before widgets, as Scouter's `keyInput` handler does; `tab_focus` is off so OpenTUI does not move focus on Tab.
- **Widths come from the native measure.** Footer fitting measures with `Context::text_width` and the app's width method, so `↑`, `…`, and wide characters count as the terminal draws them; tests pass a character count.

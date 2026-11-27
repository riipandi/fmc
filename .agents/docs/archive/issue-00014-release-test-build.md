---
status: resolved
created: 2026-11-27T19:22:00+07:00
updated: 2026-11-27T21:30:00+07:00
summary: "`cargo test --release` does not compile: two tests in `src/app/overlay_tests.rs` call `open_dialog`, which `src/app/overlay.rs` defines only under `cfg(debug_assertions)`."
count: 1
---

# The release-profile test build fails

## Tests call the debug-only dialog opener

- status: resolved (2026-11-27: `open_dialog` and the `Dialog` import build under `cfg(any(debug_assertions, test))`; the panic test in `tests/cli.rs` runs in debug builds only, since its hook is debug-only and a release session waited forever; `cargo test --release` green, 649 tests)
  area: tests (`src/app/overlay_tests.rs`), overlay (`src/app/overlay.rs`)
  summary: `src/app/overlay_tests.rs:114` and `:130` call `open_dialog`, and `src/app/overlay.rs:194-195` defines it under `#[cfg(debug_assertions)]`. `cargo test --release` stops with E0425 (`cannot find function open_dialog`, three errors). The debug test run is green (650 tests at `plan-00013` phase 2 and the fix commit). The gates in `shape.md` run the debug profile, so no gate is red.
  plan: none yet
  trigger: the owner asks for a release-profile test run, or the next change to the dialog opener in `overlay.rs` touches it.
  note: found on 2026-11-27 while plan-00013 timed highlighting in a release build; the timing test was moved to the debug profile, and the release build was not fixed there.

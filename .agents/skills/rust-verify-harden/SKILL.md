---
name: rust-verify-harden
description: >-
    Run the Rust quality gates (cargo fmt --check, cargo clippy, cargo test, cargo build) and fix failures, then
    audit the changed and related Rust code for resource leaks, deadlock risk, data races, terminal-state and
    render-loop hazards, and hot-path waste, applying low-risk fixes. Use when the user asks to verify build
    quality gates, prep Rust code for merge or release, or audit concurrency and memory safety. Verify fixes
    against current docs (docs.rs, doc.rust-lang.org, crate changelogs), not memory.
---

# Rust Verify & Harden

Run the quality gates, fix what fails, then do a focused safety and performance pass on the code the change touched. The contracts in `.agents/docs/rust.md` and `.agents/docs/rules.md` win over anything here.

## Sources of truth

- The locked library source first: `opentui` and `opentui-sys` from `~/.cargo/registry/src/` (the version in `Cargo.lock`). Then docs.rs for the same version, doc.rust-lang.org for the language, std, and edition guide, the Clippy lint list for lint rationale, and the crate's changelog for deprecations.
- Use whatever fetch or search tool the runtime provides (web fetch, `cargo doc --open` locally). If none is available, say which claims rest on memory.
- When documentation conflicts with memory or with existing code, documentation wins; for `opentui`, the locked source wins over docs. Note the source in the summary.

## Phase 1 — Quality gates

Run the gates in `.agents/docs/shape.md` ("Commands"): `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build`. If a gate has no command there, say so and skip it: never invent a replacement, and do not install a tool unasked.

Fixing rules:

1. **Check or build fails**: find the root cause (type error, trait bound, API change). For a crate API change, read the locked version's source or docs before editing.
2. **Clippy fails**: fix the code. `#[allow(...)]` only when the lint is genuinely wrong for this case, with a one-line comment saying why. `-D warnings` turns `missing_docs` into an error too: document public items instead of silencing the lint.
3. **Test fails**: fix the bug, not the assertion.
4. **Format drift**: run `cargo fmt`, then re-check with `cargo fmt --check`.

Edition 2024 and error-handling rules for new or changed code: `.agents/docs/rust.md` ("Toolchain facts", "Code rules").

## Phase 2 — Deep analysis

Scope: files changed in Phase 1 plus their direct callers and callees.

1. **Resource leaks**: `Rc`/`Arc` cycles without `Weak`; `mem::forget` or `Box::leak` without reason; unbounded `Vec`, `HashMap`, or channel growth in long-lived loops; a spawned thread whose join handle is dropped. Every background worker stops on a shutdown signal and is joined.
2. **Deadlocks and blocking**: a `std::sync::Mutex` or `RwLock` guard held across a blocking call; nested locks taken in inconsistent order; blocking I/O on the input or render path (reading a large file, a `git` subprocess, a directory walk). Move it off that path and feed the result back.
3. **Races**: `unsafe impl Send` or `Sync` without proof; `static mut`; check-then-act on the file system (checking that a path exists, then opening it). Run `cargo miri test` only when the code has `unsafe` and miri is installed. Use `loom` only if it is already a dev-dependency.
4. **Terminal state** (once the code touches the terminal): raw mode, the alternate screen, and mouse capture are restored on normal exit, on error, and on panic.
5. **Untrusted input**: file names, file contents, and `git` output that reach a shell, a path join, or the terminal as raw control sequences. Fix locally when the fix is small; otherwise report it for `security-review`.

Apply a fix only when it is local and low-risk, with one line of reasoning. Report instead of fixing anything that changes behavior, ownership structure, or public API. A behavior change is also a parity question: check `rules.md` first.

## Phase 3 — Performance pass

Same scope. Look for needless `clone()` or `to_string()` on hot paths, `Vec` or `String` growth with a known size (`with_capacity`), allocation inside per-frame or per-line loops, `collect()` only to iterate again, and accidental O(n²) work. Do not claim a speedup without a measurement (a benchmark or a before/after timing); otherwise list it as an opportunity.

## Phase 4 — Close out

1. Re-run every gate from Phase 1; all must pass. A change to what is drawn or how input is handled also gets a live run (`run` skill or Rex): the unit suite never sees the rendered screen.
2. Summarize, in the user's language:
    - **Files changed**, each with its reason.
    - **Issues found** by category (leak, deadlock, race, terminal, boundary, perf), with `file:line`.
    - **Fixes applied**, one line each, with the documentation consulted.
    - **Remaining `unwrap` / `expect` / `unsafe`**, each justified or flagged.
    - **Not fixed**: risky or design-level findings and why.
    - **Gates skipped** and why.

## Rules

- Stay in scope: Phase 1 files plus direct neighbors. Anything else is a recommendation.
- Do not add dependencies or install tools unless the user approves.
- Do not commit or push unless asked.

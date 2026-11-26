# Rust

updated: 2026-11-26T08:12:18+07:00

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
- **Workers answer through events.** `services::spawn` runs a job on its own thread and sends the `services::Event` it returns through the app's `AppProxy`; `app/` applies each event on the UI thread. A reply may arrive after the tree changed, so `apply` checks that the path is still in the tree.
- **Git runs on a private index copy, one per repository.** `services/git.rs` `Repository` copies `.git/index` before a listing, keeps the copy's mtime (git's racy-index check), and copies again unless the index and the copy are unchanged and the index is older than 2 s (`src/diff/git.ts:156-198`). `Drop` removes the copy. Git's `diff` rewrites the index it reads even with `GIT_OPTIONAL_LOCKS=0` (Scouter decision 0007). A patch diffs against the base of the last listing and reads the copy that listing made, so call `changes()` before `patch` or `patches`.
- **Each git call is bounded and queued.** A call is killed after 15 s (it then reports `git exited with status 143.`, as Scouter's Bun timeout does), and a listing past 8 MiB has no text. A patch is cut at 1 MiB, the stacked diff at 4 MiB, and the files past the cut are not shown. One queue per repository runs one call at a time. A quit during a call leaves its copy in the temp directory, which Scouter removes at exit (`src/diff/git.ts:372-375`); fmc does not yet.
- **A conflicted file lists as modified.** `git diff <base> --name-status` reports it as `M`, so the `U` letter (`Conflict`) never comes from the change list; the `Unmerged:` notice is unreachable there, as in Scouter (`research/scouter-inventory.md` §21).
- **A text search stops, and a preview answers its latest request.** Each text search runs on a worker that checks an `AtomicBool` between files and reports progress every 120 ms (`services/find.rs`). A new query stops the running search and bumps the `Generation` (`find/text.rs`); an event for an older number is dropped. A preview read answers the latest request, and an older answer is dropped (`app/find.rs`).
- **The finder shows its matches in the tree's list.** `Browser::show` replaces the tree's rows with flat rows and `show_tree` brings the tree back (`explorer/browser.rs`); `row_count` is what the list and its scrollbar measure, and an empty list is valid. `Browser::rows` borrows the whole browser, so a method that also sets `selected` copies the new path out before it assigns.
- **The tree is drawn from a scroll position.** `ui/tree.rs` paints only the visible rows; the scrollbar column exists only while the list overflows (a hidden widget takes no space).
- **Overlapping captures cascade as Core's.** The vendored grammars register with `Grammar::core_captures()` (`src/ui/syntax/mod.rs`, `grammar`). Without it, a TOML key takes `(bare_key) @type`'s style where Scouter shows `(pair ...) @property`'s; with it, every vendored grammar's captures match Scouter's (plan-00001 phase 11). Rust, bundled with `opentui`, is registered by the binding and matched Scouter without it.
- **Core's queries use `#match?`, not `#lua-match?`.** The binding evaluates tree-sitter's regex predicates and ignores unknown ones, so a `#lua-match?` would colour every identifier. `src/ui/syntax/queries/{typescript,typescriptreact,zig}/highlights.scm` carry the translation (same patterns; the comment pattern takes `(?s)`); the test `typescript_names_a_property_as_core_does` pins one capture.
- **The Markdown injections are the binding's own.** `src/ui/syntax/queries/markdown/injections.scm` is the text of `MARKDOWN_INJECTIONS` (`opentui-0.9.9/src/syntax/highlight.rs`), not Core's: Core's `#set-lang-from-info-string!` and `#offset!` are not evaluated, and inline text needs `injection.include-unnamed-children` to keep its backticks in the string colour.
- **Rendered Markdown has rows only after a frame.** The viewer's scrollbar and the footer's position count `Markdown::block_rows`, which are empty until the frame lays the document out; `repaint_after_layout` (`src/app/mod.rs`) paints again 20 ms after each event and resize. The body and the stack clip with `Overflow::Hidden`, or a rendered block paints over the footer.
- **The reader scrolls in a `ScrollBox`, and unwrapped text is pane-wide.** `ui/viewer.rs` wraps the text, the rendered view, and the picture; the pan range is then the gutter's six columns, as in Scouter (`ui/reader.rs` `set_width`). The wrapped row count, which the bar and the footer use, comes from a second `TextView` on the code's buffer (`CodeView::rows`), because `Code` keeps its own view private.
- **Search marks name a style the buffer's table must hold.** A layer's group set names `opentui.<name>`, resolved against the buffer's style table, and the binding registers only the capture names a grammar uses. The code view installs its own table (`ui/syntax/mod.rs` `table`) with every capture name of the queries plus `opentui.search.match`; a new capture name in a query must join `CAPTURE_NAMES`, or it is drawn plain.
- **Widths come from the native measure.** Footer fitting measures with `Context::text_width` and the app's width method, so `↑`, `…`, and wide characters count as the terminal draws them; tests pass a character count.

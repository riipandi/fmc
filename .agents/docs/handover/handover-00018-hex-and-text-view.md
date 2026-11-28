---
status: done
created: 2026-11-28T19:20:00+07:00
plan: plan-00023-hex-and-text-view
---

# Handover: refused files open as text or in a hex view; plan-00023 closed

## Closed

- plan-00023 on `feat/hex-view` (the worktree branch `worktree-hex-view`, renamed). Commits, in order: `ab80511` (the research and the plan), `36fad22` (phase 1, the core: row layout, byte classes, `Blocks`, the byte match, `decode_lossy`), `cf1cb26` (phases 2 and 3, the block reads and the hex view with its keys, `:` offsets, and the notice buttons), `e17f9f6` (phase 4, the string search), `b3d2caf` (phase 5, `t` and the not-UTF-8 notice), and the docs commit that adds this handover (phase 6). Phase 7 is the validation below and the PR to `main`, not merged.
- The plan is archived as `archive/plan-00023-hex-and-text-view.md`. `handover-00017-markdown-image-boxes.md` stays in `handover/`: it belongs to the Markdown image work in the main checkout, and moving it from this branch would collide with that work.
- `just verify` is green (979 tests run, 1 skipped; licenses ok). Live runs in Rex tabs at 80x24 and 160x48 (then about 185 columns): a 2 GiB generated file and a 2 GiB sparse file open, `G` and `:50%` land at once; a 1 MiB file truncated to 4 KiB while shown did not crash; a search with no hit over 2 GiB answered in about 1.5 s while `j` kept scrolling; a hit is drawn with `match-bg` (checked in the HTML capture); a Latin-1 file shows the notice with `Only UTF-8 text files are supported.`, `t` shows `caf�` and `␛[0m`, and `X` shows 32 bytes a row at that width.

## Decisions taken as planned

- The owner chose every recommendation (O1 to O6) and added a string search; D4 to D10 in the plan record them, and `architecture.md` carries the deviation and the block-read decision.
- D1's long-lived worker became one `services::spawn` job per block (the plan was amended before phase 1); no worker needs stopping when the view closes.

## Choices the plan did not settle (for the owner)

- A not-UTF-8 file's header badge and footer still say `binary`, as a NUL file's do. A badge such as `not UTF-8` would be a small change in `reader::badges` and the footer facts.
- A lossy text's footer still says `UTF-8` as its encoding, which is what it was decoded as; the `lossy` badge says the bytes were changed.
- The footer hints of the hex view are the agent's pick: scroll, Tab, top/bottom, page, `/` search, `:` offset, `t` text, copy, close, help, quit. The notice's hints are the list's, unchanged.
- The wheel moves the hex view one row a step, as a wheel step on the bars does.

## Traps (paid for; the next agent should read these first)

- `window::scan` going back must not compute `high - CHUNK` below zero; the first version panicked on a worker, which left the footer at `searching…` with no answer. A panicking worker sends no event, so any worker fault looks like a hang.
- A worker function that no caller reaches fails `cargo clippy -D warnings` in the binary crate, so a service cannot land a commit before its caller.
- The worktree needs `scripts/opentui-native.sh` once, which links the shared native library cache into `.cache/`.
- The worktree guard refuses shell commands whose text contains `git` in a path it cannot resolve (such as `github.com` in an absolute path); run the binary from a copy outside the repository for Rex runs.

## Owner check

1. Run fmc in a real terminal on a large binary and press `X`, then scroll, `G`, `:50%`, and `/` with a word you know is inside: confirm the colours read well in both palettes.
2. For a cold-cache number, run `! sudo purge`, then open the 2 GiB file again and press `G`; the research measured only a warm cache.
3. Decide whether a not-UTF-8 file should say `not UTF-8` instead of `binary` in its badge and footer.
4. Review and merge the PR from `feat/hex-view` when ready; it touches `architecture.md`, `index.md`, `rust.md`, and `CHANGELOG.md`, which the Markdown image work may also touch.

---
status: partial
created: 2026-11-26T07:12:30+07:00
plan: plan-00001-scouter-parity
wave: W0–W3 done; W4 next
---

# Handover: workspace, plan, and parity waves W0 to W3 landed; W4 next

## What this session did

Set up the agent workspace (instructions, skills, `.claude/` settings, hooks, project agents), researched Scouter, the OpenTUI Rust binding, dependencies, sqltui, and bjarneo/editor (`research/`), and wrote `plan-00001-scouter-parity.md`. The owner made every stack decision on 2026-11-26; each is a record in `architecture.md` (engine `opentui`/`opentui-sys` =0.9.9 with `tree-sitter`, native library built from source with `-Dvideo=false`, `MIT OR Apache-2.0`, macOS aarch64 only, single crate with core/services/ui, clap, git subprocess with polling, `icu_collator`, Scouter v0.3.4 `434d000f` as baseline with `@opentui/core` bumped to 0.9.9 as the live reference, Scouter's 16 grammars plus Core's queries for JS/TS/TSX/Markdown/Zig).

Waves landed: W0 (licenses, toolchain pin 1.100.0, rustfmt, `deny.toml`/`about.toml`, `scripts/opentui-native.sh`, engine swap, `scripts/demo-repo.sh`), W1 (CLI, theme, key table and parser, app loop and chrome), W2 (tree model, fs and git services, tree view, sidebar resize), W3 (code view, Markdown, image and binary, motions/wrap/pan/copy, in-file search). Plan phases 1 to 15 are `done`.

Approved parity deviations (all in `architecture.md` and `docs/parity.md`, each naming the baseline): product name fmc; reveal expands the immediate parent; `Ctrl+q` quits and `q` closes the open document or screen; Enter on a tree file keeps list focus; a refused startup file still shows its folder; drag selects and `y` copies; no root row in the tree; `?` opens the key list as a table in the reader (supersedes the Mouse-section card record).

Ledger: `issue-00001` archived (resolved); `issue-00002-binary-footer-position.md` and `issue-00003-rendered-markdown-scroll-end.md` open for phase 23; `roadmap-00001-post-parity.md` holds RI-1 to RI-13 (Mermaid, Neovim grammars, fuzzy finder, file-manager writes, SQLite/DuckDB, platforms, Vim keymap, `notify`, settings.json, theme catalog, adaptive layout and collapsible sidebar, hidden-files toggle, configurable keymap).

## Commits (this slice)

`e0616ea..HEAD` on `main`, 42 commits; pushed up to `1a10d5c`, the rest is pushed with this handover. Wave commits: W0 `f4bf88f`, `b95be21`, `73c73ed`, `01e643e`; W1 `74c5816`, `68016a1`, `f5d8811`, `e250508`; W2 `49a7938`, `e73b26a`; W3 `3246799`, `aed05be`, `1697c5d`, `ceb054a`, `8e899cc`, `8b7ef30`, `191bce3`; deviations `be58013`, `a6d56f1`, `05f1618`.

## What stayed open / the trap

- W4 (phases 16 and 17: file finder, text search with preview) is next; `/` in `--help` and the key list is inert until it lands.
- W5 phase 19 owes `q` on the Changes screen (note in the plan frontmatter) and phase 18 owes Scouter's 15 s git timeout and 8 MiB listing cap.
- Owner checks pending (Rex cannot inject mouse or draw pixels): sidebar edge drag and clamp, click on rows, wheel step, drag-select with `Esc` and `y`, the image in a Kitty-capable terminal.
- The `q` footer hint does not fit at 80 columns; it shows when there is room and is always in the key list.
- Larger live captures than 80x24 are not available: Rex opens 80x24 tabs and `stty` cannot grow them.

Traps:

- Two writers on one branch: never `git commit --amend` (an amend once nearly rewrote another writer's commit) and stage explicit paths.
- Executors treat a relayed message as no approval: record the owner's decision in `architecture.md` first, then brief the executor.
- Editing the plan's YAML frontmatter by string replacement once merged two phase entries; check that 25 `- id:` entries remain after every edit.
- The sandbox may deny `rm`; run `git rm <path>` as a separate command.
- `opentui-sys` (not `opentui`) carries the LGPL declaration; the `deny.toml` exception is scoped to it.

Suggested for the next session: `executor` agent for W4 (Sonnet), `scout` for lookups (Haiku), `parity-audit` before claiming a screen, `agent-docs` for ledger moves.

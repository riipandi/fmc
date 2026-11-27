---
status: done
created: 2026-11-27T19:23:00+07:00
plan: plan-00013-injected-highlighting
---

# Handover: injected highlighting landed for the linked grammars (RI-18)

## What this session did

The owner's go-ahead of 2026-11-27 ("kerjakan berikutnya RI-18 dan RI-19") started RI-18 (`plan-00013-injected-highlighting.md`, now archived as done). The code now colours the code inside a file with its own grammar, for the pairs the linked grammars already support: Svelte and Astro `<script>` and `<style>` bodies (and the Astro frontmatter), HTML `<script>` and `<style>`, a HEEx template's markup and expressions, the markup and Elixir of an Elixir `~H` sigil, the bodies of Elixir's `~r`, `~SQL`, `~MD`, `~PY`, and `~JS` sigils, JavaScript, TypeScript, and TSX `/** */` comments (JSDoc) and regex literals, and the strings of Python's `re` functions. No dependency was added. Each injection query is the grammar crate's own file, or fmc's own for Python (D1, D2, D5). Two kinds of host rule had to change, because Core's cascade paints a 2-part host capture over a 1-part injected capture: the Elixir sigil rules and the JavaScript and TypeScript regex and doc-comment rules (D10, D11, D12). The deviations from Scouter are recorded in `architecture.md` (the HTML record and the JavaScript, TypeScript, and Python record) and `docs/parity.md` (the Reader section).

Ledger moves: `plan-00013-injected-highlighting.md` closed and moved to `archive/`; `handover-00008-write-operations.md` moved to `archive/` when this handover landed; `issue-00014-release-test-build.md` filed (open). RI-18 is `landed` in `roadmap-00001-post-parity.md`; RI-17 is parked with its own unlock.

## Commits (this slice)

- `2770e7b` `docs: plan-00013 for the injected highlighting (RI-18)` — the plan, the roadmap's RI-18 marked `planned`, and the index.
- `1c3857e` `feat(syntax): highlight the code inside HTML, Svelte, Astro, Elixir, and HEEx files` — the injection queries for those five filetypes, the `injected` helper in `src/ui/syntax/mod.rs`, the Elixir highlights' sigil split, the tests, and the records.
- `d619868` `feat(syntax): highlight JSDoc and regex literals in JavaScript and TypeScript, and regex strings in Python` — the JavaScript and Python injections, the JavaScript, TypeScript, and TSX host rules, the tests, and the record.
- `d68a7e7` `fix(syntax): colour the body of an Elixir ~r sigil as a regex` — the whole-sigil `string.regex` rule for `~r` dropped (found in the final live run).
- The docs close, this commit: the roadmap, the plan's archive move, the issue, this handover, and the index.

## What stayed open / the trap

- Open: `issue-00014-release-test-build.md`. `cargo test --release` does not compile (`src/app/overlay_tests.rs:114` and `:130` call the debug-only `open_dialog`). No gate runs the release tests, so it is not blocking.
- Open, owner's choice: the cost of JavaScript highlighting. On the first megabyte of Scouter's `node_modules` JavaScript, the highlight takes about 1.4 times as long with the injections (interleaved debug medians: 1.12 to 1.19 s without, 1.57 to 1.61 s with). TypeScript shows no measurable change. Options are in `architecture.md`, "JavaScript, TypeScript, and Python code inside a file is highlighted".
- Not done, listed in `plan-00013` "Explicitly out of scope": Svelte `{expression}` and `{#if}` blocks as JavaScript, JavaScript tagged templates, Elixir `~VUE` (inert: `vue` is not registered), and RI-19 (multi-select), which the owner also named and which is a separate task.
- Not pushed. Branch `feat/ri-18-injections` holds 5 commits on top of `b929627`; the pull request is the owner's to open.
- The trap: a host capture that covers injected text paints over it. Core's cascade orders captures by fewer dotted parts first, then by start, and the later one paints. A host `string.special`, `string.regex`, `string.regexp`, or `comment.documentation` over the body of an injection hides the injected grammar's 1-part captures (`tag`, `keyword`, `escape`, `operator`). The fix is a host rule that leaves the injected text out (`rust.md`, "Core's cascade paints a 2-part host capture over the injected text"). Check this first when an injected token shows the host's colour in a live capture, not the injection's.
- A scratch harness was used for the corpus digests and the timings (`/tmp/fmc-ri18-*`, outside the repository, not committed). The corpus baseline is `/tmp/fmc-ri18-corpus-before.txt`.

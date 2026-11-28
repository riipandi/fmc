---
status: done
created: 2026-11-28T18:00:00+07:00
plan: plan-00018-remaining-grammars
---

# Handover: the owner's grammar set mostly landed; plan-00018 closed

## Closed

- plan-00018 on `feat/remaining-grammars`, stacked on `feat/open-with-apps` (not `main`). Commits: `4acb036` (query and json5 from crates), `25e0f14` (vimdoc, luadoc, luap, and just vendored under `vendor/grammars/`), `21cde7b` (Lua's luap patterns), and the docs commit (this handover, the archive move, the roadmap and the spike).
- RI-17 is `landed in part` in `roadmap-00001-post-parity.md`. Parked: luadoc in Lua's `---` comments (OpenTUI has no offset directive) and eex (no licence).
- Filetypes now highlighted: `query` (`.scm`), `json5`, `just` (`justfile`, `.justfile`, `.just`), and `vimdoc` (only through `file_associations`, for example `"doc/*.txt": "vimdoc"`). Lua's pattern argument of `find`, `match`, `gmatch`, and `gsub` is injected as luap.
- The new dependency approvals are in `rules.md`, "Dependencies": `tree-sitter-tsquery` `=0.8.0`, `tree-sitter-json5-orchard` `=0.1.1`, and `cc` `=1.11.3` as a build dependency. The lock gains the two crates only.

## Live run

Run in a new tab of this session (`rex run -s "$REX_BLOCK" --focus=false`), at 120 by 40, on a sample folder under `target/fmc-p5-sample` with a temporary `XDG_CONFIG_HOME` holding `file_associations` (`target/fmc-p5-config`). The owner's settings were not touched. The tab was closed after the run.

- `doc/owls.txt` with `"doc/*.txt": "vimdoc"`: the tag `*owl-post*` takes the label colour (`rgb(229,184,128)`, the palette's constant), its asterisks the punctuation grey, and `|owl-post|` the underlined link colour (`rgb(130,170,255)`).
- `justfile`: `set` and `dotenv-load` take the keyword colour (`rgb(199,146,234)`), `"Hedwig"` the string colour, and the recipe `deliver` the function colour. The comment `# the owl post` inside the recipe body stays plain (the grammar does not tag it as a comment there).
- `vault.json5`: `// Gringotts vault 713` is the comment grey, the keys `owl`, `count`, `ok` and `true` take the constant colour, and `'Hedwig'` the string colour.
- `highlights.scm` (a byte-identical copy of `src/ui/syntax/queries/query/highlights.scm`; the browse root was the sample): the header lines take the comment colour.
- `lua/owls.lua`: `%d` in `"%d+"` takes the luap keyword colour (`rgb(199,146,234)`). `---@param key string …` is a plain comment, which is the luadoc gap.
- Earlier, a stale binary showed `file_associations "doc/*.txt" is skipped: vimdoc is not a filetype fmc shows`. The rebuilt binary accepts it. Nothing in the code changed between the two runs.
- Not run: no real Vim or Neovim session, and no colour check on the real terminal (the HTML capture gives the colours the terminal drew).

## Tests

- `just verify` on the final tree: 819 run, 819 passed, 1 skipped; licences ok. `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` clean. `rumdl fmt --check --quiet` clean on the changed Markdown.
- `cargo build --release` once (the build script runs in the release profile): finished, and the binary exports the four vendored symbols.
- `cargo about generate about.hbs -o target/THIRD_PARTY_LICENSES.html`: ran; the vendored section lists the luadoc and luap MIT texts in full, and the Apache-2.0 texts come through the crates' own Apache entries.
- Syntax tests: every copied query compiles with its grammar, every capture of the copied queries has a table entry, the new queries capture only names the palette styles, and a sample of each new filetype is painted (the tests in `src/ui/syntax/tests.rs`).

## Where the build differs from the plan

- The phase 3 commit `21cde7b` has only the luap rule. The luadoc rule was probed and dropped: the luadoc grammar fails on a body that starts with a dash, and OpenTUI does not evaluate `#offset!` (`architecture.md`, "Lua patterns are an injection"). The plan's test of `---@param` is not met.
- Phase 2 vendors `just` with the repository's own `queries/just/highlights.scm`, which is generated upstream. Its `@error` capture is dropped (an unstyled capture draws plain).
- vimdoc's `injections.scm` (code blocks in help files) is not vendored; the plan did not name it.
- The Cargo.toml comment lines the owner removed in the working tree are gone from the phase 2 commit, as the owner decided.
- `.gitignore` had `/vendor/`, which hid the new tree; that line is removed.
- Each phase carries its own docs, as AGENTS.md requires; the plan put them in phase 4.
- `CHANGELOG.md` and `docs/parity.md` changed in phases 1 to 3.

## Open threads

- `.agents/docs/rust.md` has one contract bullet (the vendored symbols, the `LanguageFn` transmute) that is not committed. The coordinator's uncommitted rule line is in the same file, and the brief said not to stage the file. The bullet must land with that rule's commit, or the coordinator tells the next run how to stage it.
- luadoc in Lua's `---` comments: needs OpenTUI to evaluate `#offset!`, or the owner's choice of another route (`architecture.md`).
- eex: needs a licence from the author (connorlay/tree-sitter-eex `f742f2f` has none).
- Possible follow-up, not planned: the just grammar leaves `# comments` in recipe bodies plain, which upstream's query does not cover.

---
status: done
created: 2026-11-28T19:47:10+07:00
plan: plan-00024-common-grammars
---

# Handover: Dockerfiles, config files, and seven common languages highlighted

## State

- plan-00024 on the worktree branch `worktree-common-grammars`, from `main` at `85256b9`. Commits, in order:
  - `08bbc80` `feat(syntax): highlight Dockerfile, Makefile, CMake, and common config files` (phase 1, with the research note and the plan). Ten crates: dockerfile, make, cmake, ini, properties, proto, graphql, nix, scss, vue. `reader/language.rs` gains `PREFIXES` (`Dockerfile.*`, `Containerfile.*`, `.env.*`); `.env` files are bash.
  - `e76cefb` `feat(syntax): highlight git and EditorConfig files` (phase 2). The four `konomanoasa-*` crates, under the owner's "avoid vendoring" (O3 (b)); SSH config dropped.
  - `f74c67e` `feat(syntax): highlight C++, C#, Ruby, PHP, Kotlin, Swift, and Dart` (phase 3).
  - Phase 4 (this commit) closes the ledger: the plan is archived, RI-21 parks wave 3, and this handover lands. The plan was renumbered from 00023 to 00024 and this handover from 00019 to 00020 when main's hex view took those numbers; `handover-00019-hex-and-text-view` moved to `archive/` at the merge.
- Docker Compose needed no change: `compose.yaml` and `docker-compose*.yml` were already YAML by extension, in fmc and in Scouter. A test pins it (`the_extensions_opentui_lacks_are_highlighted`).
- `just verify` is green on phase 3: 960 tests run, 960 passed, 1 skipped; licenses ok. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `rumdl fmt --check --quiet .`, and `cargo deny check licenses` are green after each phase.
- `cargo build --release`: 65,972,256 bytes (HEAD before the plan: 39,510,272). The research predicted 65,889,440 for the same set.
- Live run (Rex, a new tab in this session at 120x30, closed after): `.env.local`, `.gitignore`, `compose.yaml`, `Dockerfile`, `hat.cpp`, `House.vue`, `Makefile`, `registry.php`, captured as HTML and read per run. Each file shows the palette's keyword, string, comment, function, and constant colours. The Dockerfile's `RUN` line is coloured as bash (`echo` in the function colour, `--no-cache` as a constant). The Vue `<script lang="ts">` body is TypeScript (`const` keyword, `string` type colour), and its `<style>` is CSS. The PHP file's `<div>` outside `<?php` is HTML.

## Decisions taken as planned

- D1 to D8 hold, and O1 (a), O2, O4 (a) as the owner chose. Query sources follow the research's "Pick" column. C++ is nvim-treesitter's c and cpp queries at 04f1788; its ancestry predicates are structural patterns or dropped.

## Deviations from the plan as written

- O3 (b) replaced the vendoring of phase 2: four crates instead of five vendored parsers, so `build.rs`, `vendor/`, and `about.hbs` are unchanged, and SSH config is not highlighted. The plan was amended before the phase ran.
- The palette check in the tests allows `none`, `markup.strikethrough`, `markup.underline`, and `markup.link.label` (`LEFT_PLAIN` in `src/ui/syntax/tests.rs`): Vue's HTML query uses them, and Core's Markdown and TSX queries do too.
- Four older tests used `make` as the example of a filetype fmc does not show, and one a ` ```make ` fence as an undrawn language. They now use `perl`.

## Traps (paid for; the next agent should read these first)

- An nvim predicate OpenTUI does not know (`#has-ancestor?`, `#is-not?`, `#lua-match?`) does not fail: tree-sitter keeps it as a general predicate, which OpenTUI never reads, so the pattern matches with no condition (`tree-sitter-0.27.2/binding_rust/lib.rs:2987`). Translate or drop it; a compile test will not catch it.
- `#set!` with two captures (nvim's `html_tags`) fails the query compile on 0.27.2, and Vim's `\c` fails the regex. Both kill the whole grammar, and the file shows plain.
- Two crates that export the same C symbol (`tree_sitter_kotlin` in `tree-sitter-kotlin-ng` and `-sg`, `tree_sitter_groovy` in both Groovy crates) cannot both link: one silently replaces the other.
- Several crates hide `HIGHLIGHTS_QUERY` behind a cfg the build never sets (`tree-sitter-graphql`, `tree-sitter-vue-next`); copy the file from the archive.
- In this worktree the first build needs `scripts/opentui-native.sh` to link `.cache/opentui-native/`.

## Owner checks

1. Colours in a real terminal: open a `Dockerfile`, a `.vue`, a `.php`, and a `.cpp` file in Ghostty. The Rex capture read the colours as RGB values, not by eye.
2. A Makefile conditional after a recipe stays plain: in the live run, `ifeq` and `endif` after a rule's recipe were text-coloured, while the unit test's `ifeq` at the top of a file takes the keyword colour. This is the crate's grammar; reply "accept" or ask for a closer look.
3. The binary is now about 63 MiB (from 38). Reply "accept", or name the languages to take out.
4. The four `konomanoasa-*` crates are one author's, two days old, with an exact `tree-sitter-language` pin. If that pin ever blocks an upgrade, the fallback is the vendoring route in `research/common-grammars.md`, section 4.
5. Review and merge the pull request from `worktree-common-grammars` to `main` (not merged by the agent).

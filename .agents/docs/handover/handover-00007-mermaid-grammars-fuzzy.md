---
status: done
created: 2026-11-27T17:10:00+07:00
plan: plan-00006-mermaid-grammars-fuzzy
---

# Handover: Mermaid, the owner's grammars, and fuzzy finding landed; plan-00006 closed

## Closed

- plan-00006 phases 1 to 6: the Mermaid spike `821340d` (research only), Mermaid in rendered Markdown `1856099`, the grammar load spike `9de164e` (research only), the twelve grammars `363cbf3`, fuzzy finding `0637e8d`, and the close (this handover, the archive move, the roadmap).
- Dependencies: `mermaid-text` and its set (phase 2), twelve grammar crates (phase 4), and `neo_frizbee` 0.13.3 (phase 5). Each is pinned exactly and recorded in `rules.md`, "Dependencies".
- Tests: 491 at the start of phase 3, 509 at the close. `just verify` and `cargo build --release` pass. Licences: `cargo deny check licenses` passes.
- Decisions the owner made: O1 to O4 in the plan (Mermaid engine, grammars, fuzzy design, `mermaid-text`). The owner's 2026-11-27 message approved O2 (every grammar that loads) and O3 (fuzzy as `find_mode` and `Ctrl+F`), and D4 (`neo_frizbee`).

## Owner checks

These need a real terminal or a phone; the agent could not judge them.

1. **Mermaid at compact widths.** At 49 columns (the page stack) the sample flowchart is drawn compacted: the boxes touch, and an arrow runs into the next box. The engine's strict budget chose this; a diagram that does not fit falls back to the framed source. Is the 49-column drawing readable enough, or should a diagram below some width fall back to the frame?
2. **Fuzzy feel.** `Ctrl+F` flips the match; `fuzzy, N matches` leads the status. At 100 columns the sidebar prompt is 26 columns, so the count is cut (`fuzzy, 11 matc`). The mode survives; the count does not. Is that acceptable, or should the mode also go on the bottom bar?
3. **Terminal keys.** `Ctrl+F` is sent by every terminal as 0x06, but confirm it reaches fmc in the owner's terminal (Terminal.app was not checked).
4. **The flip's scope.** `Ctrl+F` lasts for the run; a restart returns to `find_mode`. The owner's O3 said "for the session"; this reads it as the run. Confirm, or ask for it to persist (a settings write, which needs its own decision).
5. **Languages on real files.** Check a few files of each: HCL (`.tf`, `.tfvars`), Elixir, SQL, Astro, Vim. A Svelte `<script>` body and other code inside a file stay plain: injections are RI-18.
6. **Priorities.** RI-17 (seven grammars, six of which pin another `tree-sitter`) and RI-18 (injections) are parked. Decide whether either comes next.

## Decisions the agent took (cheap to undo; each has its record)

- Captures renamed to Core's names in the Lua, SQL, and XML queries; XML's document text is `none` (plain). `architecture.md`, "the owner's extra grammars are highlighted".
- HCL's query is nvim-treesitter's (Apache-2.0) with `@spell` dropped; `.tf` and `.tfvars` map to HCL, `.heex` to heex (`reader/language.rs`, `EXTENSIONS`).
- Fuzzy: case ignored, no typos, the rank is the score then Scouter's order; the match colour is `accent_soft` in bold, the colour text hits use; the status puts the mode first (`architecture.md`, "the finder can match fuzzily").
- The test samples are inline strings, not files. The live samples live in `/tmp/fmc-live`, outside the repository.

## Live evidence (new tabs in the agent's session, each closed after its capture)

- Phase 4, debug build: `vault.tf`, `schema.sql`, and `owls.ex` in the reader at 100 columns. The footer reads `hcl`, `sql`, and `elixir`; keywords, comments, and identifiers take the palette's colours (`rex capture --format=vt`). A `sql` fence in rendered Markdown takes the same colours.
- Phase 5, debug build: `find_mode` `"fuzzy"` from a scratch settings file: `mermd` gives 11 matches, and its matched letters take `#c7d2fe`. With no settings file: `mermd` gives `no matches`, `Ctrl+F` gives `fuzzy, 11 matches`, a second `Ctrl+F` gives `no matches`, and Esc closes the finder. `find_mode` `"Fuzzy"` shows the notice in the footer and keeps substring.
- Phase 6, release build: `quest.md` (a Mermaid flowchart and a `sql` fence) at 100 columns (sidebar, flowchart drawn, SQL coloured), 70 columns (reader alone, flowchart drawn), and 49 columns (page stack, flowchart compacted; owner check 1).

## Where things live

- Grammars: `src/ui/syntax/mod.rs` (`register`, `CAPTURE_NAMES`, `language_for`); queries and LICENSE files in `src/ui/syntax/queries/<filetype>/`; `src/reader/language.rs` (`HIGHLIGHTED`, `EXTENSIONS`).
- Fuzzy: `src/find/mod.rs` (`Matching`, `fuzzy`, `char_positions`, `names_status`); `src/settings.rs` (`find_mode`); `src/app/find.rs` (`toggle_fuzzy`, `show_names`); `src/keys/parser.rs` (`Ctrl+F` while a prompt is open); `src/ui/tree.rs` (`marked_text`).
- Mermaid: `src/reader/mermaid.rs`, `src/ui/mermaid.rs` (phase 2).
- Records: `architecture.md` (Mermaid records; "the owner's extra grammars are highlighted"; "the finder can match fuzzily"); `docs/parity.md` (Reader, Finder); `docs/configuration.md` (`find_mode`, Finder); `docs/keymaps.md` (Find a file).

## Open and unverified

- Parked: RI-17 and RI-18 in `roadmap-00001-post-parity.md`.
- The MSRV 1.92 build (`cargo +1.92`) has not run: no 1.92 toolchain is installed here.
- Only macOS arm64 was built and run.
- The new grammar crates declare no `rust-version`.

## Scratch files (outside the repository unless named)

- `/tmp/fmc-spike/`, `/tmp/fmc-live/`, `/tmp/fmc-verify-*.log`, `/tmp/fmc-ref/` (read only).
- `src/ui/syntax/spike_tests.rs` is an untracked scratch test inside the repository. It is not compiled, and it is not committed. The agent could not delete it (the sandbox denied `rm`). Delete it before the next commit.

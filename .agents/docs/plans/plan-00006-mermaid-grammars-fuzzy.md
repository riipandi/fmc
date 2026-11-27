---
status: in-progress
created: 2026-11-27T11:07:49+07:00
updated: 2026-11-27T16:30:00+07:00
references: roadmap/roadmap-00001-post-parity.md (RI-1, RI-2, RI-3)
phases:
    - id: 1
      title: "Mermaid spike: a Grok port and mermaid-text on the same diagrams"
      status: done
      note: "Spike in a throwaway project (/tmp/fmc-spike, outside the repo; fmc's manifests and src untouched). Grok mermaid.rs ported to plain rows: 157 lines removed, 42 added, 3,968 non-test lines kept, unicode-width the only crate, 144 tests pass. mermaid-text 0.58.0 (MIT, MSRV 1.92, six new crates on macOS). Four Harry Potter and Dan Brown diagrams plus one broken, at 60 and 100 columns. Recommendation: mermaid-text with max_width_strict (Grok frames most flowcharts: no compaction, 104 columns natural). Width: the renderer is not called on resize (opentui markdown.rs:928-1426); a measured Custom widget gets the width in layout (custom.rs:155), to be proven in phase 2. Report: research/mermaid-spike.md. Stopped for the owner's pick."
    - id: 2
      title: "Mermaid in rendered Markdown, with the engine the owner picked"
      status: done
      note: "mermaid-text 0.58.0 with max_width_strict (src/reader/mermaid.rs), drawn by a measured Custom widget (src/ui/mermaid.rs). Width route: layout's measure, so a resize re-measures with no renderer call; proved by a TestApp test and live (100 to 60 columns: the frame's reason moved from 'the reader has 65' to 'the reader has 53'). 19 new tests. Crates beyond O4's list: none on macOS; the lock also gains target-only packages (rules.md). Records: architecture.md, 'Mermaid in rendered Markdown' (four records)."
    - id: 3
      title: "Grammar load spike and dependency approval list"
      status: done
      note: "All 12 candidates load on tree-sitter 0.27.2 and compile their query: diff, elixir, heex, hcl (no query shipped; D7 query in phase 4), jsdoc, lua, regex, sql, svelte-ng, xml, astro-next, vim. None dropped. Scratch test, not committed; the Cargo files reverted. Lock 131 to 143 packages (12 direct, 0 transitive); clean debug build about 4 s more CPU. Captures to rename or add in phase 4 are in the report. Report: research/grammar-load-spike.md. Dependency approvals land in rules.md with phase 4."
    - id: 4
      title: "Register the new grammars and their filetypes"
      status: done
      note: "Twelve grammars registered and =-pinned (src/ui/syntax/mod.rs), queries and LICENSE files under queries/<filetype>/. Captures renamed to the names the palette styles (lua, sql, xml markup to none); CAPTURE_NAMES +12; HIGHLIGHTED 18 to 30; reader/language.rs EXTENSIONS maps heex, hcl, tf, tfvars. Injections not registered (follow-up in the roadmap, RI-2). Tests: the load test covers all twelve, a capture-name check, and a styled token per language. Live: HCL, SQL, Elixir in the reader, and a sql fence in rendered Markdown, colours read from rex capture --format=vt. Records: architecture (Approved deviation: the owner's extra grammars are highlighted), parity, rules (Dependencies, Licenses), rust, shape, README, CHANGELOG, configuration, PRODUCT. Citation fix: Scouter supportedLanguages is code-view.ts:20-29."
    - id: 5
      title: "Fuzzy finding: matcher, `find_mode` setting, Ctrl+F in the finder"
      status: done
      note: "neo_frizbee =0.13.3 (rules.md, phase 5 approval). find::Matching (Substring default, Fuzzy): the whole query as one needle, no typos, case ignored, ranked by score then Scouter's order; find::matches dispatches on it. settings.rs find_mode (invalid: substring and a notice). Ctrl+F in the finder (Action::ToggleFuzzy, parser binds it while a prompt is open) flips it for the run and re-runs the query. Footer: `fuzzy, N matches`, the mode first (a 26-column sidebar cut the mode after the count in the first live run). Matched characters: accent_soft, bold. Tests: 6 find, the key, settings, tree row, and 3 TestApp finder tests. Live (new tabs, own session): fuzzy from settings (11 matches for mermd, match colour checked in VT), Ctrl+F flip from substring, the invalid-value notice. Records: architecture (Approved deviation: the finder can match fuzzily), parity (Finder), configuration, keymaps, README, CHANGELOG, rules, rust, shape, PRODUCT."
    - id: 6
      title: "Docs, roadmap close, handover, final validation"
      status: pending
---

## Mermaid diagrams, the owner's grammar set, and fuzzy file finding

### Objective

Land roadmap RI-1, RI-2, and RI-3. When this plan is done:

- A ` ```mermaid ` fence in rendered Markdown shows as a box-drawing diagram that fits the reader, and falls back to its source in a frame when the diagram is unsupported or does not parse.
- Files in the owner's extra languages (the compatible subset of RI-2) are highlighted.
- The finder can match fuzzily: a setting picks the start mode, and `Ctrl+F` in the finder flips it.

Each item is a deviation from Scouter v0.3.4 (`434d000f`) and is recorded in `architecture.md` and `docs/parity.md` in the phase that lands it.

### Current state (2026-11-27)

- **Markdown.** `src/ui/markdown.rs` wraps OpenTUI's `Markdown` widget. Scouter draws a `mermaid` fence as plain code (`research/scouter-inventory.md` §7). OpenTUI 0.9.9 has `Markdown::code_block_renderer(language, Fn(&CodeBlock) -> Option<AnyWidget>)` (`opentui-0.9.9/src/widget/markdown.rs:631`); returning `None` keeps the default code block. `CodeBlock` carries no width (`markdown.rs:157-175`), so the renderer needs the reader's width from somewhere else.
- **Grammars.** `src/ui/syntax/mod.rs` `register` lazily compiles 16 vendored grammars with `core_captures`; `src/reader/language.rs` `HIGHLIGHTED` allows 18 filetypes (its test pins the count). Filetypes come from OpenTUI's `filetype_for_path`, which already knows `lua`, `ex`/`exs` (elixir), `sql`, `xml`, `svelte`, `vim`, `diff`/`patch`, and `astro`, but not `heex`, `eex`, `hcl`/`tf`, `just`, or `json5`. Locked: `tree-sitter` 0.27.2, `tree-sitter-language` 0.1.9.
- **Finder.** `src/find/mod.rs` `matches` copies Scouter's lowercase substring AND match and its order (`compareRelative`). The finder prompt is in `src/app/find.rs`. `Ctrl+F` is unbound.
- **Settings.** `src/settings.rs` reads `auto_preview`, `sidebar_jump`, and `reader_jump` from `settings.json` once at start; fmc never writes it. `docs/configuration.md` lists the keys.
- **Policy.** `deny.toml` allows MIT, Apache-2.0, BSD, ISC, Zlib, and Unicode-3.0; MPL-2.0 (`nucleo-matcher`) is out. MSRV is 1.90 (`Cargo.toml:9`).

### Research behind the choices

- **Grok CLI** (`xai-org/grok-build`, Apache-2.0) draws Mermaid inline as Unicode box art with its own engine, `crates/codegen/xai-grok-markdown/src/mermaid.rs` (about 5.7k lines). It uses Ratatui only for styled lines; `plain_lines` is renderer-agnostic. It supports flowchart (with subgraphs), sequence, class, state, and ER. It lays out to the content width, falls back to the source in a frame when a diagram is too wide or unsupported, wraps labels at 4 lines of 24 columns, and re-renders on a width change. Its PNG path (resvg, writes cache files) is out: fmc is read-only.
- **`mermaid-text`** 0.58.0 (MIT, MSRV 1.92, deps `ascii-dag`, `chrono`, `unicode-width`): `render_with_width(input, Option<usize>) -> Result<String, Error>`; about 18 diagram types per its README table and `DiagramKind`; the width is a soft hint (sequence, pie, and ER ignore it; a strict mode exists).
- **Grammar crates** compatible with `tree-sitter-language` 0.1: `tree-sitter-diff` 0.1.0, `-elixir` 0.3.5, `-heex` 0.8.1, `-hcl` 1.1.0 (ships no query), `-jsdoc` 0.25.0, `-lua` 0.5.0, `-regex` 0.25.0, `-sequel` 0.3.13 (SQL), `-svelte-ng` 1.0.2, `-xml` 0.7.0 (queries under `queries/xml/`), `-astro-next` 0.1.1. `tree-sitter-vim` 0.4.0 depends on `tree-sitter >=0.21` directly (ABI unchecked). No compatible crate: json5, just, luadoc, luap, query (only an unreviewed fork), vimdoc (pins 0.25), eex (none published). A crate that pins another `tree-sitter` breaks the build on the `links` key.
- **Fuzzy crates.** `neo_frizbee` 0.13.3 (MIT, MSRV 1.89, no required deps; the matcher inside fff): `Matcher::new(needle, &Config)`, `match_one_indices(haystack, index) -> Option<MatchIndices>` with `score: u16` and `indices`. `frizbee` is the same code under its old name.

### Decisions

Owner, 2026-11-27 (do not reopen):

- **O1.** Mermaid: spike both engines first, then the owner picks.
- **O2.** Grammars: every compatible crate, each behind a load test.
- **O3.** Fuzzy: a `find_mode` setting (`"substring"` default, `"fuzzy"`) picks the start mode; `Ctrl+F` in the finder flips it for the session.
- **O4.** (After phase 1.) Mermaid engine: `mermaid-text` 0.58.0 with `max_width_strict`, over the Grok port (`research/mermaid-spike.md`). This approves the MSRV bump from 1.90 to 1.92 and the new crates `mermaid-text`, `ascii-dag`, `chrono`, `iana-time-zone`, `core-foundation-sys`, `num-traits`, and `unicode-width`. Its quirks are accepted: reordered class members, and some malformed lines rendered without an error.

Agent, within those:

- **D1.** Mermaid renders only in rendered Markdown (`m` preview). The raw code view keeps the fence as highlighted text, and `.mmd` files stay plain. Reason: the hook is the Markdown widget's; a code-view renderer is new surface nobody asked for.
- **D2.** A diagram wider than the reader falls back to the framed source rather than clipping or scrolling sideways (Grok's rule). Reason: rendered Markdown wraps and has no horizontal pan.
- **D3.** The diagram uses the palette: box lines in `border`, labels in `text`, edge labels in `muted`. No new palette keys.
- **D4.** Fuzzy is `neo_frizbee`, not `frizbee`. Reason: same code, newer release.
- **D5.** In fuzzy mode the query is one needle (spaces included), results sort by score, then by Scouter's order on ties, capped at `MAX_MATCHES`. Matched characters are drawn in the match colour. The finder's footer names the mode (`fuzzy`) when it is on. Text search (`text.rs`) is unchanged.
- **D6.** An invalid `find_mode` value falls back to `"substring"` with a notice, as the other keys do.
- **D7.** `hcl` gets a highlight query written for fmc's capture names, adapted from nvim-treesitter's (Apache-2.0), with a header comment naming the source.

### Phases

#### Phase 1: Mermaid spike

- In a scratch module behind `#[cfg(test)]` (no shipped code), render four diagrams through both engines at 60 and 100 columns: a flowchart with a subgraph, a sequence diagram, a class diagram, and a gantt chart. Diagram text uses Harry Potter or Dan Brown names (`rules.md`, "Sample data").
- For the Grok engine: copy `mermaid.rs` and what it needs into the scratch module, keep the Apache-2.0 header, and replace the Ratatui types with plain strings. Do not build or run the Grok repo; read it from `/tmp/fmc-ref/grok/src`.
- Find out how a code-block widget learns the reader's width (the `AnyWidget` measure path, or re-rendering from `MarkdownView::show` on resize). Cite the locked source.
- Report to the owner: side-by-side output, lines of code to keep, new crates and their licenses, the MSRV effect (mermaid-text needs 1.92), and render time on the largest sample. **Stop for the owner's pick.** Nothing is committed except the report in `.agents/docs/research/mermaid-spike.md`.

#### Phase 2: Mermaid in rendered Markdown

- Land the chosen engine: as `src/mermaid/` in the core crate (a pure `fn render(source, width) -> Result<Vec<Line>, Fallback>`), or as the approved crate behind the same function.
- Register a `mermaid` renderer on `MarkdownView` that returns a text widget with the lines, or `None` for an empty fence. Re-render on reader width changes (cache by source and width).
- Fallback: the source in a frame, with a one-line reason (`unsupported diagram: gantt`, `parse error at line 3`).
- Tests: render and fallback unit tests per diagram type; a TestApp test that opens a Markdown file with a fence and finds box characters. Live run at 60 and 100 columns.
- Docs: architecture record (deviation from Scouter §7), `docs/parity.md` (Reader), README feature list, CHANGELOG.

#### Phase 3: Grammar load spike

- Add each candidate crate to the working tree (not committed, reverted after), build, and compile its grammar with its highlight query through `Grammar::compile`. Record per crate: builds, compiles, query path, query errors against fmc's capture names.
- `tree-sitter-vim` and `tree-sitter-astro-next` are included as candidates; drop any that fail.
- Report the final list with licenses and the added dependency count. This list is the owner's dependency approval (O2 approves the set in principle; the report confirms what actually loads). Commit nothing.

#### Phase 4: Register the grammars

- Add the crates that passed, `=`-pinned as the existing grammars are, with the comment in `Cargo.toml` updated.
- Copy each highlight query into `src/ui/syntax/queries/<filetype>/highlights.scm` (keeping its license header), adjusted so every capture is in `CAPTURE_NAMES` or falls back by first part. Add missing names to `CAPTURE_NAMES` only when the theme styles them.
- Extend `HIGHLIGHTED` and its test. Map the extensions OpenTUI does not know (`heex`, `hcl`, `tf`, `tfvars`) in `language_for`, before `filetype_for_path`.
- Injections (heex in elixir, jsdoc in JavaScript, regex in JS/Python, SQL in Markdown fences) only where OpenTUI's injection support already handles them for Markdown; otherwise list them as a follow-up.
- Tests: the existing load test covers every new grammar; one sample file per language (Harry Potter/Dan Brown content) shows a styled token. `just verify` (licenses included). Live run on three sample files.
- Docs: architecture record, `docs/parity.md` (Reader), README language list, CHANGELOG.

#### Phase 5: Fuzzy finding

- Add `neo_frizbee`. A `Mode` (`Substring`, `Fuzzy`) on the finder; `find::matches` dispatches on it; the fuzzy path returns entries with match indices.
- `settings.rs`: `find_mode` (D6). `docs/configuration.md` documents it.
- `Ctrl+F` in the finder prompt flips the mode and re-runs the query; the footer shows the mode (D5). Keymaps doc and help text gain the key.
- Tests: fuzzy order and ties, indices, the setting (temporary `XDG_CONFIG_HOME`), the key in a TestApp finder. Live run on fmc's own tree.
- Docs: architecture record, `docs/parity.md` (Finder), `docs/keymaps.md`, CHANGELOG.

#### Phase 6: Close

- Mark RI-1, RI-2, and RI-3 landed in the roadmap; list dropped grammars (json5, just, luadoc, luap, query, vimdoc, eex) as a new parked entry.
- Archive this plan, write the handover with the owner checks (diagrams on the phone at compact width, a few languages, fuzzy feel), update `index.md`.
- `just verify`, `cargo build --release`, and live runs at 49, 70, and 100 columns.

### Risks

- The Grok port is large; if phase 1 shows it needs heavy rework for plain strings, mermaid-text wins on cost.
- mermaid-text's MSRV 1.92 raises fmc's MSRV; that is part of the owner's pick in phase 1.
- Old grammar crates may compile but carry queries written for other capture sets; phase 3 measures it.
- Each grammar adds C code to the build; phase 3 reports the build-time change.

### Validation

Every phase runs `just verify` (or `KACHE=false just verify`). Phases 2, 4, and 5 also need a live run in a new tab of the agent's own Rex session (AGENTS.md, "Rex live runs").

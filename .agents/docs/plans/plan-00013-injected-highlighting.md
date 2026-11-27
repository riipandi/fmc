---
status: in-progress
created: 2026-11-27T18:56:00+07:00
updated: 2026-11-27T18:56:00+07:00
phases:
    - id: 1
      title: Injections for HTML, Svelte, Astro, Elixir, and HEEx
      status: pending
      note: —
    - id: 2
      title: Injections for JavaScript and TypeScript (JSDoc, regex) and Python (regex)
      status: pending
      note: —
    - id: 3
      title: Docs close, ledger close, handover
      status: pending
      note: —
    - id: 4
      title: Final validation and live captures
      status: pending
      note: —
---

## Inject the code inside highlighted files — the pairs the grammars already link

> Owner go-ahead: "kerjakan berikutnya RI-18 dan RI-19" (2026-11-27). RI-19 is a separate task and is not in this plan.

### Objective

The owner's grammars (RI-2, `architecture.md`, "the owner's extra grammars are highlighted") and Scouter's HTML and Python colour a file's own tokens, but not the code a file holds: a Svelte `<script>` body, the HTML inside an Elixir `~H` sigil, the JSDoc comment or regex literal inside JavaScript, and the Astro frontmatter. This plan registers the injection queries those grammars already ship, so each pair is coloured by its inner grammar, with no new dependency. The tree after it has injections on eleven filetypes, and the Scouter-set filetypes that gain them (JavaScript, TypeScript, HTML, Python) are recorded as approved deviations. Read-only behaviour and every other filetype stay as they are.

### Current state (verified in the tree, 2026-11-27)

- **Registration has no injection for these filetypes.** `src/ui/syntax/mod.rs:21-29` registers `javascript`, `typescript`, and `typescriptreact` with the helper `grammar` (`mod.rs:69`), which calls `Grammar::new(..).core_captures().compile()` with no `.injections(..)`. The helper is used by `html` (`mod.rs:43`), `python` (`:45`), `elixir` (`:52`), `heex` (`:53`), `svelte` (`:59`), and `astro` (`:61`). Only `markdown` passes `.injections(include_str!("queries/markdown/injections.scm"))` (`mod.rs:32`).
- **The injected body is plain today.** A scratch dump of the current grammars (`src/ui/syntax/scratch.rs`, not committed) gives a Svelte `<script>` body (`let owl = 'Hedwig';`) only the capture `none`, and its `<style>` body the same. An Elixir `~H` sigil gives `string.special` only, with no `div` tag. JavaScript's JSDoc block is one `comment` capture, with no `@param` keyword.
- **The crates ship the queries.** `tree-sitter-svelte-ng-1.0.2/queries/injections.scm`, `tree-sitter-heex-0.8.1/queries/injections.scm`, `tree-sitter-elixir-0.3.5/queries/injections.scm`, `tree-sitter-javascript-0.25.0/queries/injections.scm`, `tree-sitter-html-0.23.2/queries/injections.scm`, and `tree-sitter-astro-next-0.1.1/queries/injections.scm` (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`). `tree-sitter-python-0.25.0` ships none, and `tree-sitter-regex-0.25.0` has only `highlights.scm`.
- **OpenTUI's own injection API.** `Grammar::injections` (`opentui-0.9.9/src/syntax/highlight.rs:65`) takes a query with `@injection.content` and `@injection.language` or the `injection.language` property. The language is resolved by name through the registry (`highlight.rs:189-195`: `resolve` takes the name or the info-string filetype), so `javascript`, `css`, `typescript`, `regex`, `jsdoc`, `heex`, `elixir`, `sql`, and `markdown` resolve to fmc's own grammars. A target the registry does not know is skipped (`highlight.rs:383-410`). The query compiles in `compile` (`highlight.rs:246-255`); a query that fails to compile fails the whole grammar, so `register_lazy` returns none and the file is plain.
- **Predicates the queries need are in the locked binding.** `tree-sitter-0.27.2/binding_rust/lib.rs:2712-2841` evaluates `eq?`, `not-eq?`, `match?`, `not-match?`, `any-of?`, and `not-any-of?`. The Svelte crate's own file uses `#eq?` and `#any-of?`.
- **Svelte's tree.** `tree-sitter-svelte-ng-1.0.2/src/node-types.json`: `script_element` and `style_element` each have `start_tag` and `raw_text` children. The crate's generic rule `((raw_text) @injection.content (#set! injection.language "javascript"))` matches style bodies too, so the crate's file alone would parse CSS as JavaScript. The crate's file also says `; inherits: html_tags`, which fmc does not resolve.
- **JavaScript's injection query in OpenTUI.** OpenTUI's bundled JavaScript, TypeScript, and TSX grammars take `js::INJECTIONS_QUERY` (`opentui-0.9.9/src/syntax/highlight.rs:206`). fmc's Core-query replacements (`mod.rs:21-29`) drop it. The JSDoc grammar on a line comment gives no captures (scratch: `// Sophie checks the key @param x` gives none; `/** @param {string} key */` gives `@param` as `keyword` and `string` as `type`).
- **Scouter does not inject these pairs.** Scouter registers the extra grammars with `highlights` only (`/tmp/fmc-ref/scouter/src/reader/grammars.ts:9-17`, the Scouter checkout at `434d000`, tag v0.3.4). Core's default parsers give JavaScript, TypeScript, and TSX `highlights` only, and Markdown the one `injections` (`/tmp/fmc-ref/scouter/node_modules/@opentui/core/chunk-bun-rgb7yx3r.js:4398-4438`). So JSDoc and regex in JavaScript and TypeScript, regex in Python, and HTML's `<script>` and `<style>` are differences from Scouter. Svelte, Astro, Elixir, and HEEx are not in Scouter.
- **Markdown fences already inject.** `MARKDOWN_INJECTIONS` (`opentui-0.9.9/src/syntax/highlight.rs:31-41`) resolves a fence's language by name. A fenced `js` block gets fmc's JavaScript grammar, so it gains the JavaScript injections too, which is a second Scouter difference, recorded with the first.
- **Baseline cost.** `TIMING` scratch runs (debug build, `highlights()`, median of 7, the sample files named in the phase 4 matrix): `large.js` (1,000,000 bytes, 193,566 captures) 848.8 ms; `large.ts` (221,796 bytes, 57,903 captures) 233.6 ms. A `--release` test build does not compile at HEAD: `src/app/overlay_tests.rs:114` and `:130` call `open_dialog`, which `src/app/overlay.rs:194-195` defines only under `cfg(debug_assertions)`. The timings are debug only, and the same debug build is used before and after.
- **Corpus baseline.** `/tmp/fmc-ri18-corpus-before.txt` holds a digest of every capture for 757 files highlighted at HEAD (typescript 500, html 385, javascript 270, rust 149, markdown 95, python 15, and others), taken from the repository, Scouter's `src`, `tests`, and `node_modules`, and OpenTUI's assets. Phase 4 diffs the after-digest against it.
- **Licences.** The vendored `highlights.scm` files sit beside a `LICENSE` (`rules.md`, "Licenses"). The crates are MIT (svelte, heex, javascript, html, python), Apache-2.0 (elixir), and MIT OR Apache-2.0 (astro).
- **Sample data.** The tests and live samples use Dan Brown and Harry Potter names (`rules.md`, "Sample data").

### Decisions

- **D1 (2026-11-27): one `injections.scm` per filetype, under `src/ui/syntax/queries/<filetype>/`, beside its `highlights.scm` and `LICENSE`.** Each is the crate's own file, copied verbatim or with the adaptations in D3 and D4, and its header names the crate and its version. Python has no crate file, so its file is fmc's own (MIT, the project licence). Rejected: inline strings in `mod.rs`, as `MARKDOWN_INJECTIONS` is. That fits one changed query; it does not fit seven vendored files that carry a licence. Cost: seven more files in the tree.
- **D2 (2026-11-27): the pairs in scope.** Svelte `<script>` to JavaScript, `<script lang="ts">` to TypeScript, `<style>` to CSS. Astro frontmatter, `<script>`, `<style>`, and its attribute and HTML interpolations to TypeScript or CSS (the crate's file, verbatim). HTML `<script>` to JavaScript and `<style>` to CSS (verbatim). HEEx `<%= %>` and `{ }` to Elixir (verbatim). Elixir sigils `~H`, `~LVN`, and `~HOLO` to HEEx, `~r` and `~R` to regex, `~SQL` to SQL, `~MD` to Markdown, `~PY` to Python, `~JS` to JavaScript (verbatim). JavaScript regex literals to regex, and JavaScript `/** */` comments to JSDoc (D3). TypeScript and TSX take the JavaScript file, as OpenTUI binds it (`highlight.rs:206`). Python `re.*` string arguments to regex (D5). Rejected: the whole crate-file set without a choice. The owner asked for these pairs and "any other cheap pair" already linked. The Elixir `~VUE` rule stays: `vue` is not registered, so its rule resolves nothing and is skipped (`highlight.rs:383-410`).
- **D3 (2026-11-27): JSDoc only for `/**` comments.** The crate's rule `((comment) @injection.content (#set! injection.language "jsdoc"))` sends every comment to the JSDoc grammar. The scratch run shows the grammar gives `//` comments no captures, so the rule is only wasted work there. The fmc file adds `(#match? @injection.content "^/[*][*]")`, so the rule means JSDoc's own syntax, and its comment says why.
- **D4 (2026-11-27): the crate's JavaScript tagged-template and Handlebars rules are left out.** A tagged template's tag names the language (`css`, `sql`, `html`), and the owner did not ask for that pair. The Handlebars rule uses `#offset!`, which the binding does not evaluate. Both are listed out of scope.
- **D5 (2026-11-27): Python regex is fmc's own rule.** `((call function: (attribute object: (identifier) @_module attribute: (identifier) @_function) arguments: (argument_list . (string (string_content) @injection.content))) (#eq? @_module "re") (#any-of? @_function "compile" "findall" "finditer" "fullmatch" "match" "search" "split" "sub") (#set! injection.language "regex") (#set! injection.include-children))`. The first argument of the `re` functions is the pattern. Rejected: every Python string as regex (a plain string is not a pattern); `#set! injection.include-children` is needed because a string's escapes are child nodes, and the pattern keeps them.
- **D6 (2026-11-27): Svelte's rules split by `lang`.** A `<script>` without a `lang` attribute is JavaScript: the crate's generic `raw_text` rule matches the style body too, so this file gives the script rule `#not-match? @_start "lang="` and the style rule its own CSS. `lang="ts"` or `"typescript"` is TypeScript (`#any-of?`). A `<style>` with a `lang` attribute is left plain, because scss and the other preprocessors are not registered (the crate's scss rule would resolve nothing).
- **D7 (2026-11-27): each phase records its own deviations in the phase's commit.** Phase 1 updates the record `architecture.md` "Approved deviation: the owner's extra grammars are highlighted" (its negative consequence, that Svelte's `<script>` is not coloured, is no longer true, so its text changes) and `docs/parity.md:46`. Phase 1 also adds a record for HTML, which Scouter colours without injections: "Approved deviation: HTML `<script>` and `<style>` bodies are highlighted". Phase 2 adds "Approved deviation: JavaScript, TypeScript, and Python code inside a file is highlighted", which covers JSDoc and regex in JavaScript and TypeScript, regex in Python, and the Markdown fences in those languages. Both name the Scouter baseline (`434d000f`, v0.3.4) and the owner's 2026-11-27 request as the approval, and each has its `docs/parity.md` line. The commits port no Scouter behaviour, so they carry no `Scouter-Base` trailer; the records cite the baseline.
- **D8 (2026-11-27): no new dependency.** Every grammar is pinned (`rules.md`, "Dependencies", the phase 4 and plan-00006 lists). `Cargo.toml` and `Cargo.lock` do not change.
- **D9 (2026-11-27): one test set per pair, in `src/ui/syntax/tests.rs`.** The file holds 172 lines, under the 800-line limit (`rust.md`, "Keep every file under 800 lines"). Each case asserts that a token of the inner language takes a capture of that language (the same style check as `the_owners_grammars_style_a_token_of_their_sample`), and a negative case asserts that the rule does not fire where it should not.

### Shape and enforcement points

| What | Where | The record |
| --- | --- | --- |
| The injection queries and their attribution | `src/ui/syntax/queries/<filetype>/injections.scm` and `LICENSE` | `rules.md`, "Licenses" (the vendored queries also carry an injections file, with the crate's licence) |
| The registration | `src/ui/syntax/mod.rs` (`injected` helper, the grammars that take a file) | `rust.md`, "Contracts and traps": a query that does not compile drops the whole grammar |
| The Scouter-set deviations | `architecture.md`, "Approved deviation: the owner's extra grammars are highlighted" (extended) and a new record for the JavaScript, TypeScript, and Python injections | `docs/parity.md` (the Reader section) |
| The files a path shows as | `shape.md`, the `ui/syntax/` row (`queries/<filetype>/` holds `highlights.scm` and `injections.scm`) | `shape.md` |
| The user-visible change | `docs/parity.md` and `CHANGELOG.md` `[Unreleased]` | `rules.md`, "Where a fact lives" |

### Execution rules

- **Atomic commit per phase** is authorized by the owner's go-ahead of 2026-11-27; each phase's heading names its commit message. Each commit runs `KACHE=false just verify` before it is made, and each commit names its paths explicitly.
- **No push.** The branch is `feat/ri-18-injections`, checked out from `main` at `b929627`. The work reaches `main` through a pull request the owner opens.
- **Scouter first.** Each port states its Scouter evidence (the file and line above). The injections that Scouter does not have are the deviations in D7, not ports.
- **The owner's parallel work is untouchable.** The Rex panes of the owner (the `REX_BLOCK` session's own panes, and the windows "Agent Console" and "File Manager") are never typed into. Live runs open a new tab in `$REX_BLOCK`'s session and close it.
- **Live runs.** Phases 1 and 2 change what is drawn, so each runs a live capture (Rex, new tab, `--focus=false`). Phase 4 holds the final captures.
- **Corpus check.** Phases 1 and 2 re-run the digest of `/tmp/fmc-ri18-corpus.txt` (scratch, not committed). A file whose digest changed must be a file with a pair in scope (a `.html` with a `<script>`, a `.js` or `.ts` with a comment or regex, a Markdown fence in one of those languages, a `.py` with a `re` call). Any other change is a regression, and the phase stops.

### Phases

1. **Injections for HTML, Svelte, Astro, Elixir, and HEEx.** Adds `queries/html/injections.scm` (the crate's file, verbatim, with its header), `queries/svelte/injections.scm` (D6), `queries/astro/injections.scm` (verbatim), `queries/elixir/injections.scm` (verbatim), and `queries/heex/injections.scm` (verbatim). `mod.rs` gains an `injected(language, highlights, injections)` helper that `grammar` calls with no injections, and the five registrations use it. The architecture record and `docs/parity.md:46` change as D7 says, `CHANGELOG.md` `[Unreleased]` gets the user-visible line, `rules.md` "Licenses" and `shape.md` name the new files, and `rust.md` gets the compile trap. Validation: `KACHE=false just verify` green; the tests in the matrix rows 1 to 8 pass; the corpus digest changes only in `.html` files with a `<script>` or `<style>`; a live Rex capture of the Svelte and Elixir samples in a new tab, with its colours checked against the palette. _Commit: `feat(syntax): highlight the code inside HTML, Svelte, Astro, Elixir, and HEEx files`. No `Scouter-Base` trailer: the commit ports no Scouter behaviour (D7)._
2. **Injections for JavaScript and TypeScript (JSDoc, regex) and Python (regex).** Adds `queries/javascript/injections.scm` (the crate's file adapted as D3 and D4 say) and `queries/python/injections.scm` (D5). `mod.rs` registers `javascript`, `typescript`, and `typescriptreact` with the JavaScript injections, and `python` with its own. The architecture record "JavaScript, TypeScript, and Python code inside a file is highlighted" is added, with its `docs/parity.md` line, and `CHANGELOG.md` `[Unreleased]` gets its line. Validation: `KACHE=false just verify` green; matrix rows 9 to 16 pass; the corpus digest changes only in files with a `/**` comment, a regex literal, a `re` call, or a Markdown fence in one of those languages; a timing run on `large.js` and `large.ts` (debug, the same harness, median of 7), with the numbers in the phase report; a live Rex capture of the JavaScript-with-JSDoc-and-regex sample and the Python sample. _Commit: `feat(syntax): highlight JSDoc and regex literals in JavaScript and TypeScript, and regex strings in Python`. No `Scouter-Base` trailer (D7)._
3. **Docs close, ledger close, handover.** `roadmap-00001-post-parity.md` RI-18 becomes `landed (commits)` in its status line and its table row; RI-17's text that names the injections as RI-18's work is checked. `plans/` loses `plan-00013` to `archive/` (status `done`, the phase table filled with the commits). The issue for the `cargo test --release` compile break (`src/app/overlay_tests.rs:114` and `:130`) goes to `.agents/docs/issues/` as `issue-00014-release-test-build.md` (status `open`, not fixed here). `index.md` rows move, and `handover-00009-injected-highlighting.md` is written (status `done`). Validation: the ledger's status lines agree with the table (`grep -c '^- status'`); no dangling references (`grep -rn "plan-00013"`); the docs' claims checked against the landed code. _Commit: `docs: close plan-00013 and RI-18, the injected highlighting`._
4. **Final validation and live captures.** The test matrix over the tree as it stands; the corpus digest after phase 2; the timings after phase 2, with the phase 0 numbers above; a final live Rex capture of the three samples (Svelte, Elixir and HEEx, JavaScript with JSDoc and regex), colours checked against `DESIGN.md` and the `DARK` palette in `src/theme.rs`. Validation: each matrix row green, the numbers in the report. _Commit: none; the evidence goes in the report._

### Test matrix

| # | What | How | Green |
| --- | --- | --- | --- |
| 1 | Svelte `<script>` body | `let` in `<script>\n  let owl = 'Hedwig';` takes `keyword` | `keyword` on `let` |
| 2 | Svelte `lang="ts"` | `let name: string` takes `type.builtin` (TypeScript only) | the capture exists |
| 3 | Svelte `<style>` body | `color` in `h1 { color: red; }` takes `property` (CSS) | `property` |
| 4 | Astro frontmatter and script | `const` in the frontmatter, and `let` in `<script>`, take `keyword` | `keyword` |
| 5 | HTML `<script>` and `<style>` | `const` takes `keyword`; `color` takes `property` | both |
| 6 | Elixir `~H` sigil | `div` in the sigil takes `tag` (HEEx) | `tag` |
| 7 | HEEx file `<%= %>` | `if` in `<%= if Hogwarts.open? do %>` takes `keyword` (Elixir) | `keyword` |
| 8 | Elixir `~r` and `~SQL` | a `[` in `~r/^[A-Z]+$/` takes a regex capture; `SELECT` in `~SQL` takes `keyword` | both |
| 9 | JSDoc in JavaScript | `@param` in `/** … */` takes `keyword`; `{string}` takes `type` | both |
| 10 | JSDoc in TypeScript and TSX | the same sample in `typescript` and `typescriptreact` | both |
| 11 | `//` comment is not JSDoc | `// Sophie checks the key @param x` has no capture beyond `comment` | none |
| 12 | Regex literal in JavaScript | a `\d` in `/^[A-Z]+\d{2}$/` takes a regex capture (`character.escape` or `string.regex`) | the capture exists |
| 13 | A string is not a regex | `"[A-Z]"` in JavaScript takes only `string` | only `string` |
| 14 | Python `re` call | `[A-Z]` in `re.compile(r"^[A-Z][a-z]+ Neveu$")` takes a regex capture | the capture exists |
| 15 | Python string, not a `re` call | `"[A-Z]"` in `print("[A-Z]")` takes only `string` | only `string` |
| 16 | Markdown fence | a `js` fence with `/** @param */` takes `keyword` for `@param` | `keyword` |
| 17 | Compile | `every_vendored_grammar_compiles_with_its_query` (existing) includes each new file | green |
| 18 | Existing tests | `cargo test` (the 647 tests at HEAD) | all green |
| 19 | Corpus regression | `/tmp/fmc-ri18-corpus.txt` digest, before (`/tmp/fmc-ri18-corpus-before.txt`, 757 files) and after each phase | only the pair files change |
| 20 | Timing | the `TIMING` scratch on `large.js` (javascript) and `large.ts` (typescript), debug, median of 7; before 848.8 ms and 233.6 ms | recorded |
| 21 | Live run | Rex, a new tab in `$REX_BLOCK`'s session, `rex capture --format=html`; the samples at 120x40 | each token's colour is its palette colour |
| 22 | Full gate | `KACHE=false just verify` | green |

E2E: recorded n/a (no E2E harness exists; `docs/development.md`).

### Explicitly out of scope

- **Svelte `{expression}` and `{#if}` blocks as JavaScript** — the crate's file does not inject them, and adding them is a rule of fmc's own. Follow-up if the owner wants it.
- **JavaScript tagged templates (`css\`…\``, `sql\`…\``, `html\`…\``)** — the tag names the language, and the owner did not ask for it (D4).
- **Elixir sigils that are not in the crate's file, and `re` aliasing in Python** (`import re as r`).
- **Markdown injections and the Markdown fence rules** — changed by nothing here.
- **New grammars** — RI-17, which needs crates (`roadmap-00001-post-parity.md`).
- **The `cargo test --release` compile break** — a test gate problem, not a highlight one: recorded as `issue-00014` (phase 3), not fixed here.
- **Settings for injections** — no key, no flag. Injections are part of the highlighting, which `file_associations` already decides.

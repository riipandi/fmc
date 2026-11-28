# Grammar load spike for plan-00006

updated: 2026-11-27T14:45:00+07:00

Point-in-time research, 2026-11-27. Plan: `.agents/docs/plans/plan-00006-mermaid-grammars-fuzzy.md` phase 3 (decision O2: every compatible crate that loads, each behind a load test). Agent audience; evidence as `file:line` or as the spike's output.

## 1. Result

All twelve candidate crates build, compile their grammar, and compile their highlight query against the locked `tree-sitter` 0.27.2 (`opentui` 0.9.9). None is dropped. Every query compiles; none reports an error. Three things remain for phase 4:

- `tree-sitter-hcl` ships no query. Its query is written for fmc (D7, adapted from nvim-treesitter).
- Some query captures are not in `CAPTURE_NAMES` (`src/ui/syntax/mod.rs`), and some of those are not styled by the palette. Phase 4 renames them to Core-style names the palette styles, or adds the names the palette styles (section 3).
- Injections are not wired by this spike. The Svelte sample gave 3 captures, so its `<script>` body is probably plain without the JavaScript injection; that cause is not checked.

## 2. Per-crate table

Method: each crate was added as an `=` pin to a scratch working tree (`Cargo.toml`, `Cargo.lock`), and a scratch test in `ui::syntax` called `Grammar::new(language, query).core_captures()` and `register` on each one, then `highlights` on a sample. The query was the crate's own `HIGHLIGHTS_QUERY` (or the file named below). "Not in CAPTURE_NAMES" lists the query's capture names that `CAPTURE_NAMES` lacks; "styled" is `theme(&DARK).get(name)`, which includes the first-part fallback (`src/ui/syntax/mod.rs`, `theme`).

| Crate (version) | Licence | Filetype (OpenTUI name or new) | Query file in the crate | Builds | Compiles | Query errors | Not in CAPTURE_NAMES (styled?) | Sample captures |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `tree-sitter-diff` 0.1.0 | MIT | `diff` (OpenTUI: `.diff`, `.patch`) | `queries/highlights.scm` | yes | yes | none | none | 5 |
| `tree-sitter-elixir` 0.3.5 | Apache-2.0 | `elixir` (OpenTUI: `.ex`, `.exs`) | `queries/highlights.scm` | yes | yes | none | `punctuation` yes, `comment.unused` yes, `string.special.symbol` yes, `string.regex` yes, `comment.doc` yes, `comment.doc.__attribute__` yes, `__name__` **no** | 30 |
| `tree-sitter-heex` 0.8.1 | MIT | `heex` (new: no OpenTUI mapping) | `queries/highlights.scm` | yes | yes | none | `error` **no** | 12 |
| `tree-sitter-hcl` 1.1.0 | Apache-2.0 | `hcl` (new: `.hcl`, `.tf`, `.tfvars`) | none shipped (`bindings`, `dialects`, `src`) | yes | yes, with an empty query | n/a | n/a | 0 (no query) |
| `tree-sitter-jsdoc` 0.25.0 | MIT | `jsdoc` (new) | `queries/highlights.scm` (2 lines) | yes | yes | none | none | 2 |
| `tree-sitter-lua` 0.5.0 | MIT | `lua` (OpenTUI: `.lua`) | `queries/highlights.scm` | yes | yes | none | `repeat`, `conditional`, `field`, `parameter`, `method`, `method.call`, `preproc`: all **no** | 14 |
| `tree-sitter-regex` 0.25.0 | MIT | `regex` (new) | `queries/highlights.scm` | yes | yes | none | `constant.character` yes | 29 |
| `tree-sitter-sequel` 0.3.13 | MIT | `sql` (OpenTUI: `.sql`) | `queries/highlights.scm` | yes | yes | none | `field`, `float`, `parameter`, `storageclass`, `conditional`: **no**; `type.qualifier` yes | 9 |
| `tree-sitter-svelte-ng` 1.0.2 | MIT | `svelte` (OpenTUI: `.svelte`) | `queries/highlights.scm` | yes | yes | none | `type.qualifier` yes, `keyword.debug` yes | 3 (`<script>` body plain; probably the JavaScript injection, not checked) |
| `tree-sitter-xml` 0.7.0 | MIT | `xml` (OpenTUI: `.xml`) | `queries/xml/highlights.scm` (`LANGUAGE_XML`; the DTD grammar is not registered) | yes | yes | none | `string.special.symbol` yes, `markup` **no**, `error` **no** | 16 |
| `tree-sitter-astro-next` 0.1.1 | MIT OR Apache-2.0 | `astro` (OpenTUI: `.astro`) | `queries/highlights.scm` | yes | yes | none | `tag.doctype` yes | 8 |
| `tree-sitter-vim` 0.4.0 | MIT | `vim` (OpenTUI: `.vim`, `.vimrc`) | `queries/vim/highlights.scm` (`language()`, a function) | yes | yes | none | `string.special.path` yes, `_scope` **no**, `_option` **no** | 16 |

Notes:

- The licence column is `[package].license` in each crate's `Cargo.toml`. Several archives ship no `LICENSE` file (`tree-sitter-diff`, `-heex`, `-lua`, `-svelte-ng`, `-vim`, `-xml`); phase 4 takes the upstream LICENSE text for each query it vendors (`rules.md`, "Licenses").
- `tree-sitter-vim` declares `tree-sitter >=0.21.0` as a direct dependency. It resolves to `tree-sitter` 0.27.2 (`cargo tree -i tree-sitter` lists 0.27.2 only), and the grammar loads.
- The other eleven crates depend only on `tree-sitter-language` (`0.1.x`); their `tree-sitter` line is a dev-dependency, which a consumer does not build.
- No crate declares `rust-version`. The pinned toolchain (1.100.0) builds them all; `cargo +1.92 check` has not run.
- Each crate's sample is a short Harry Potter or Dan Brown snippet (`rules.md`, "Sample data").

## 3. What phase 4 changes, from this table

- Captures to rename in the vendored query (Core-style names the palette styles, by first part): Lua `repeat` to `keyword.repeat`, `conditional` to `keyword.conditional`, `field` to `variable.member`, `parameter` to `variable.parameter`, `method` to `function.method`, `method.call` to `function.method.call`, `preproc` to be decided from the query's use; SQL `field` to `variable.member`, `float` to `number.float`, `parameter` to `variable.parameter`, `storageclass` to `keyword.modifier`, `conditional` to `keyword.conditional`. Each rename is checked against the query's use before it lands.
- Names to add to `CAPTURE_NAMES` because the palette styles them (the table registers only listed names): `punctuation`, `comment.unused`, `string.special.symbol`, `string.regex`, `comment.doc`, `comment.doc.__attribute__`, `constant.character`, `type.qualifier`, `keyword.debug`, `tag.doctype`, `string.special.path`.
- Names left plain: `error` (heex, xml), and `__name__` (elixir), `_scope`, `_option` (vim), unless the query's use shows a style is wanted. Their first part is not styled.
- `tree-sitter-hcl` gets a query written for fmc's capture names (D7), with a header that names nvim-treesitter (Apache-2.0).
- Injections are not registered in phase 4: elixir to heex, svelte to JavaScript and CSS, astro, JavaScript to jsdoc, JavaScript and Python to regex. The binding's injection support is per grammar (`Grammar::injections`, `opentui-0.9.9/src/syntax/highlight.rs:65`), and no existing vendored grammar uses it beyond Markdown. Markdown fences already reach the new filetypes by name, because the fence's info string resolves through the registry (`filetype_for_info_string`, `has_grammar`). The follow-up is listed in the roadmap.

## 4. Dependency and build effect

- Direct crates added: 12, all `=`-pinnable and all on the allowlist (`rules.md`, "Licenses"; `cargo deny check licenses` passed with them).
- `Cargo.lock`: 130 packages at HEAD, 142 with the twelve crates (a plain count of `version =` lines gives one more, the lockfile's own format line). The twelve are the only additions; `tree-sitter` 0.27.2 and `tree-sitter-language` 0.1.9 are shared.
- Clean debug build (`cargo build`, `RUSTC_WRAPPER` empty, a fresh target directory, one run each on this machine): HEAD 11.2 s wall, 46.9 s user; with the twelve crates 11.2 s wall, 50.8 s user. The C grammars add about 4 s of CPU in a debug build. Wall time was unchanged within the noise of one run.

## 5. Parked for the roadmap

Not candidates in this spike, because no crate loads: json5 (`tree-sitter` ~0.20), just (~0.25.5), luadoc (~0.20.10), luap (~0.20.10), query (0.20.9), vimdoc (0.25.10), and eex (no published crate).

Correction (plan-00018, 2026-11-28): the `links` clash on the locked `tree-sitter` is the 0.25 pins' (just and vimdoc). The 0.20 pins (json5, luadoc, luap, and query) link, with a second C runtime. The later outcome: query and json5 load from crates (`tree-sitter-tsquery`, `tree-sitter-json5-orchard`); just, vimdoc, luadoc, and luap are vendored C parsers, of which luap is injected into Lua's patterns; eex stays parked on its licence. See `architecture.md`, "the owner's remaining grammars are highlighted". Each version was read from the crate archive on 2026-11-27, after the spike; they are the reason the seven are parked (RI-17). The roadmap gets one parked entry for these and for any grammar that a later phase drops. Dropped in phase 3: none.

## 6. Unverified

- The highlights of each new filetype are not checked on screen. Phase 4 runs a live check on three sample files and the load test covers all twelve.
- MSRV of the twelve crates (none declares one) and their builds on targets other than macOS arm64.
- Query quality beyond compile: the renamed captures are checked by the phase 4 test, not by a visual comparison with Neovim.

## 7. Scratch files (outside the repository)

- `/tmp/fmc-spike/spike_tests.rs`: the scratch load test (not committed; it is also the untracked file `src/ui/syntax/spike_tests.rs` in the working tree, see the phase report).
- `/tmp/fmc-spike/crates/`: the twelve crate archives, extracted for reading the manifests and queries.
- `/tmp/fmc-spike/head/`: a `git archive` of HEAD, used for the baseline build time.
- `/tmp/fmc-spike/target-base/`, `/tmp/fmc-spike/target-new/`: the two target directories of the build-time measurement.

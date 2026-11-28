---
status: done
created: 2026-11-28T19:28:25+07:00
updated: 2026-11-28T19:47:10+07:00
phases:
    - id: 1
      title: Wave 1 crate grammars, file names, and .env as bash
      status: committed (08bbc80)
      note: —
    - id: 2
      title: Git-family and EditorConfig grammars from crates
      status: committed (e76cefb)
      note: O3 (b); SSH config dropped
    - id: 3
      title: Wave 2 language grammars
      status: committed (f74c67e)
      note: O1 (a)
    - id: 4
      title: Agent docs sync, roadmap, ledger close, handover
      status: committed (docs commit)
      note: —
    - id: 5
      title: Final validation, live run, push, pull request
      status: done
      note: just verify green, live run in the handover; push and PR follow the docs commit
---

## Highlight Dockerfile, build and config files, and the common languages

### Objective

fmc shows a `Dockerfile`, a `Makefile`, a `.gitignore`, a `.env`, and a C++ or Kotlin file as plain text today. After this plan, it highlights the filetypes of `research/common-grammars.md` that the owner picks below, each from a grammar that loads on the locked `tree-sitter` 0.27.2, with its query and `LICENSE` beside it, and resolves their file names. Docker Compose files are already highlighted as YAML and need no change (`research/common-grammars.md`, section 1).

### Current state (verified in the tree, 2026-11-28)

- **Registration** — `register_lazy` per filetype in `src/ui/syntax/mod.rs:20-126`, through `grammar` or `injected` (`:131-143`); queries under `src/ui/syntax/queries/<filetype>/`. Vendored parsers: `build.rs:5` lists `vimdoc`, `luadoc`, `luap`, `just`; `src/ui/syntax/vendored.rs:6-11` declares their C functions.
- **Mapping** — `HIGHLIGHTED` (34 filetypes, `src/reader/language.rs:9-44`), `EXTENSIONS` (`:49-59`), `NAMES` (`:63`, exact lowercased names only); `fallback_filetype` (`:72-79`) checks `NAMES`, then `EXTENSIONS`, before OpenTUI's `filetype_for_path`. No prefix rule exists, so `Dockerfile.dev` cannot be named today.
- **What OpenTUI already names** — `dockerfile`, `make`, `cpp`, `csharp`, `ruby`, `php`, `kotlin`, `swift`, `scala`, `dart`, `haskell`, `ocaml`, `r`, `perl`, `powershell`, `vue`, `scss`, `groovy`, `ini` (`opentui-0.9.9/src/generated.rs:136-169`). They stay plain because `is_highlighted` gates them (`src/ui/syntax/mod.rs:154`).
- **Tests** — `src/ui/syntax/tests.rs:11-22` (`every_vendored_grammar_compiles_with_its_query`), `COPIED` (`:25-44`, each query checked against `CAPTURE_NAMES`); `src/reader/language.rs:82-115` (filetype count 34, extensions, names).
- **Notices** — `about.hbs:18-19` lists the vendored grammars by hand.
- **Research** — `research/common-grammars.md`: each candidate's crate, licence, load result, query source, and measured cost.

### Decisions

- **D1 (2026-11-28): one grammar per filetype, the path plan-00006 and plan-00018 set.** Each filetype joins `HIGHLIGHTED`, gets one `register_lazy`, and a `queries/<filetype>/highlights.scm` with its `LICENSE`. Filetype names are OpenTUI's where it has one (`csharp`, `make`, `dockerfile`), else nvim-treesitter's (`gitignore`, `gitattributes`, `gitconfig`, `editorconfig`, `sshconfig`, `proto`, `graphql`, `nix`, `properties`, `cmake`).
- **D2 (2026-11-28): the query source per filetype is the research pick** (`research/common-grammars.md`, section 3, "Pick"): the crate's own where it compiles and styles most of the sample, nvim-treesitter's at `04f1788` (Apache-2.0) for graphql, cpp, scala, powershell, and objc; the git-family crates' own queries under O3 (b). Rejected: nvim's everywhere — it fails on eleven of the picked grammars, whose versions differ from nvim's pins.
- **D3 (2026-11-28): nvim predicates are translated, never left to over-match.** `#lua-match?` → `#match?` with regex classes; each `#has-ancestor?`, `#has-parent?`, `#not-has-ancestor?`, `#not-has-parent?` pattern is rewritten as a structural pattern or dropped; `#set!` on a capture is dropped; `#offset!` stays (ignored, the quotes take the colour); `@spell`, `@nospell`, `@none`, `@conceal` are dropped. Each copied query's header names its source, commit, and every change (`research/common-grammars.md`, section 5). A capture the palette does not style is renamed to one it does (`character.special` → `string.special`, `conditional` → `keyword.conditional`), as plan-00006 did.
- **D4 (2026-11-28): Dockerfile from `tree-sitter-containerfile` 0.9.2.** Rejected: `-orchard` 0.1.0 (ISC, 206 recent downloads, two weeks old), `-updated` 0.2.0 (a stale fork depending on `tree-sitter` directly), `tree-sitter-dockerfile` 0.2.0 (pins 0.20, a second C runtime).
- **D5 (2026-11-28): `.env` files are highlighted as bash.** No dotenv grammar is needed: `KEY=value`, `export`, and `#` comments are bash. Neovim names them `env`, which has no parser (`research/common-grammars.md`, section 8). Rejected: a dotenv grammar crate (none exists).
- **D6 (2026-11-28): Vue and PHP get their injections; Dockerfile's `RUN` gets bash.** From each crate's own `injections.scm`, adapted to fmc's filetype names, as plan-00013 did. Without them a `.vue` file's script and style and a PHP file's HTML are plain.
- **D7 (2026-11-28): dropped from this plan.** Perl (no crate loads; upstream commits no `parser.c`), git commit messages (WTFPL), Erlang, Clojure, Julia, LaTeX.
- **D8 (2026-11-28): a `Scouter-Base` trailer is not used.** Scouter has none of these grammars (`scouter:src/reader/code-view.ts:20-29`, `scouter:grammars/`); each filetype is one approved-deviation record and one `docs/parity.md` line.

### Owner decisions

- **O1 (owner, 2026-11-28: (a)): which languages land.** (a) Waves 1 and 2: the 16 config and build filetypes plus C++, Ruby, PHP, C#, Kotlin, Swift, Dart; the binary grows from 37.7 to about 63 MiB [recommended: covers the daily files of most developers; wave 3 waits for a request]. (b) Wave 1 only: about +1.2 MiB. (c) All three waves: about +46 MiB, to 84 MiB. (d) Wave 1 plus C++, Ruby, PHP: about +8.6 MiB.
- **O2 (owner, 2026-11-28: approved, with the four O3 (b) crates): the crates.** Each pinned exactly, none brings another package. Wave 1: `tree-sitter-containerfile` =0.9.2 (MIT), `tree-sitter-make` =1.1.1 (MIT), `tree-sitter-cmake` =0.7.6 (MIT), `tree-sitter-ini` =1.4.0 (Apache-2.0), `tree-sitter-properties` =0.3.0 (MIT), `tree-sitter-proto` =0.6.1 (MIT), `tree-sitter-graphql` =0.3.0 (MIT), `tree-sitter-nix` =0.3.0 (MIT), `tree-sitter-scss` =1.0.0 (MIT), `tree-sitter-vue-next` =0.1.0 (MIT). Wave 2: `tree-sitter-cpp` =0.23.4, `tree-sitter-ruby` =0.23.1, `tree-sitter-php` =0.25.2, `tree-sitter-c-sharp` =0.24.0, `tree-sitter-kotlin-sg` =0.4.1, `tree-sitter-swift` =0.7.4, `tree-sitter-dart` =0.3.0 (all MIT). Risk to note: `tree-sitter-kotlin-sg` and `tree-sitter-vue-next` are forks with no stars; they are the only Kotlin and Vue crates whose query compiles.
- **O3 (owner, 2026-11-28: (b), "avoid vendoring"): vendoring.** (a) Vendor five MIT parsers at nvim-treesitter's pins under `vendor/grammars/`, built by the existing `build.rs`: gitignore (shunsambongi `f4685bf`), gitattributes (tree-sitter-grammars `1b7af09`), gitconfig (the-mikedavis `3a61756`), editorconfig (ValdezFOmar `v2.0.0`, with `scanner.c`), sshconfig (tree-sitter-grammars `0c35b24`) [recommended: about 0.3 MiB, the grammars nvim uses, no one-author crate]. (b) Take the four `konomanoasa-*` crates for the git family (one author, published 2026-11-26, exact `tree-sitter-language` pin) and drop SSH config. (c) No git-family or SSH highlighting.
- **O4 (owner, 2026-11-28: (a), without the SSH config names, since O3 (b) drops SSH config): file-name rules.** (a) The full table of `research/common-grammars.md`, section 8: a new prefix rule for `Dockerfile.*`, `Containerfile.*`, and `.env.*`; extensions `dockerfile`, `containerfile`, `mk`, `mak`, `cmake`, `env`, `proto`, `graphql`, `gql`, `graphqls`, `nix`, `properties` (and `gradle`, `sbt`, `psd1` only for waves that land them); names `CMakeLists.txt`, `.env`, `.gitignore` and the eight other ignore files, `.gitattributes`, `.gitconfig`, `.gitmodules`, `.editorconfig`, `ssh_config`, `Jenkinsfile` (wave 3 only); `~/.ssh/config` through a documented `file_associations` example, not a default [recommended: Neovim's rules, and a bare `config` name is too broad]. (b) Exact names and extensions only, no prefix rule: `Dockerfile.dev` and `.env.local` stay plain unless the user adds an association.

### Shape and enforcement points

| What | Where | The record |
| --- | --- | --- |
| The approved crates | `rules.md`, "Dependencies", a table for plan-00023 | Crate, version, licence, purpose |
| The vendored sources | `rules.md`, "Licenses"; `vendor/grammars/<name>/LICENSE` and `SOURCE`; `about.hbs` | Upstream, commit, licence |
| The copied queries | `rules.md`, "Licenses"; each query's header and `LICENSE` | Source, commit, changes |
| The reasoning and the deviation | `architecture.md`, one record per phase; `docs/parity.md` | D1 to D8, O1 to O4, Scouter baseline v0.3.4 (`434d000f`) |
| The prefix rule | `rust.md` if it carries a trap; `shape.md` (`language.rs`) | Order: names, prefixes, extensions |
| User docs | `docs/usage.md` (highlighted list), `docs/configuration.md` (filetype names for `file_associations`, the `~/.ssh/config` example), `CHANGELOG.md` | New filetypes |
| Wave 3 and the dropped set | `roadmap-00001-post-parity.md`, a parked entry | Unlock: an owner request |

### Execution rules

- The owner chose O1 to O4 and gave the go-ahead on 2026-11-28 ("sesuai rekomendasi, usahakan tidak melakukan vendoring"); phases run in order. `Cargo.toml` edits ask for permission; versions are the ones above, re-checked to resolve.
- Branch: `worktree-common-grammars` (this worktree). Never touch `src/reader/document.rs`, `src/ui/binary.rs`, or the hex and binary view files.
- Every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Docs land in the same commit as the behaviour.
- Each phase passes `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo deny check licenses`.

### Phases

1. **Wave 1 crate grammars** — the ten wave-1 crates in `Cargo.toml`; queries and `LICENSE` per D2 and D3; `register_lazy` (vue with its injections, dockerfile with bash in `RUN`); `HIGHLIGHTED`; `fallback_filetype` gains the prefix step (O4 (a)); `NAMES` and `EXTENSIONS` per O4; `.env` names to `bash` (D5). Tests: each grammar in `COPIED` compiles and uses only palette-styled names; a sample per filetype highlights at least one styled span; `Dockerfile.dev`, `app.dockerfile`, `.env.local`, `CMakeLists.txt`, `x.mk`, `x.proto`, `x.gql` resolve; `compose.yaml` stays `yaml`; a Vue sample's `<script>` carries a JavaScript capture. Docs: `rules.md`, `architecture.md`, `docs/parity.md`, `docs/usage.md`, `docs/configuration.md`, `CHANGELOG.md`, `shape.md`. Validation: `cargo test syntax`, `cargo test language`, `cargo deny check licenses`. _Commit: `feat(syntax): highlight Dockerfile, Makefile, CMake, and common config files`._
2. **Git-family and EditorConfig grammars (O3 (b))** — `konomanoasa-tree-sitter-gitignore` =0.7.3, `konomanoasa-tree-sitter-gitattributes` =0.8.2, `konomanoasa-tree-sitter-gitconfig` =0.7.3, `konomanoasa-tree-sitter-editorconfig` =0.9.2 (all MIT, each pins `tree-sitter-language` =0.1.9, the locked version); each crate's own query (nvim's does not fit these grammars) with its `LICENSE`, per D3; names per O4. No vendoring, no `build.rs` change. SSH config is dropped: its only loading crate is CC0-1.0, not on the allowlist. Tests as phase 1, plus `.gitignore`, `.dockerignore`, `.gitmodules`, `.gitattributes`, `.editorconfig` resolve. Validation: as phase 1. _Commit: `feat(syntax): highlight git and EditorConfig files`._
3. **Wave 2 languages (per O1)** — the wave-2 crates; queries per D2 and D3 (cpp and objc inherit nvim's c query, prepended, with c's four ancestor patterns rewritten); PHP with its HTML injection (D6); `HIGHLIGHTED`. Tests as phase 1 for each; a PHP sample's HTML carries an HTML capture. Docs as phase 1. Validation: as phase 1, `cargo build --release` once, binary size recorded against the research's 65.9 MB. _Commit: `feat(syntax): highlight C++, C#, Ruby, PHP, Kotlin, Swift, and Dart`._
4. **Agent docs sync, roadmap, ledger close, handover** — `index.md` (research row, plan row), the roadmap's parked entry for wave 3 and the dropped set, the plan archived, a handover. Validation: `rumdl fmt --check --quiet .`; the roadmap status table's row count equals its entries. _Commit: `docs: close plan-00023, common grammars`._
5. **Final validation, live run, push, pull request** — the matrix below; a live run in a new tab of this Rex session (`rex run -s "$REX_BLOCK" --focus=false -- …`) opening a `Dockerfile`, a `compose.yaml`, a `Makefile`, a `.gitignore`, a `.env.local`, a `.vue`, and (phase 3) a `.cpp` and a `.php` file at 120 columns, then the tab closed; push the branch and open a pull request to `main`, not merged. _Commit: none when green._

### Test matrix

| What | How | Green |
| --- | --- | --- |
| Format | `cargo fmt --check`; `rumdl fmt --check --quiet .` | No drift |
| Lint | `cargo clippy --all-targets -- -D warnings` | 0 warnings |
| Unit | `cargo test syntax`, `cargo test language` | Every new grammar compiles with its query, uses styled names, highlights its sample; every name, prefix, and extension resolves |
| Full suite | `just verify` | All green |
| Licences | `cargo deny check licenses`; `cargo about generate …` | Pass; vendored licences listed |
| Build | `cargo build --release` | Builds; size recorded |
| Live run | Rex tab of this session, the files above at 120 columns | Coloured in the palette's roles, no plain block where a grammar applies |
| E2E | n/a: no command-line change | — |

### Out of scope

- Wave 3 (Scala, Haskell, OCaml, Objective-C, R, PowerShell, Groovy) unless O1 (c).
- Perl, git commit messages, Erlang, Clojure, Julia, LaTeX (D7).
- SSH config (O3 (b): the CC0 crate is not allowed, and vendoring is declined).
- Injections beyond D6.

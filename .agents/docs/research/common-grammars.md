# Common grammars research

updated: 2026-11-28T19:30:00+07:00

Point-in-time research, 2026-11-28, for plan-00023 (`.agents/docs/plans/plan-00023-common-grammars.md`). Agent audience; evidence as `file:line` in the tree, in the locked crates (`opentui` 0.9.9, `tree-sitter` 0.27.2, `tree-sitter-language` 0.1.9), or as the scratch runs' output. The scratch crates live in `/tmp/fmc-cg-PKSA/` (`load`, `load2`, `load4`, and the release trees `base`, `w1`, `w12`, `all`), outside the repository, and are not kept.

## 1. Result

- **Docker Compose needs nothing.** `compose.yaml`, `compose.yml`, `docker-compose.yml`, `docker-compose.override.yml`, and `docker-compose.prod.yaml` all resolve to `yaml` by extension (`opentui-0.9.9/src/syntax/filetype.rs:21-31`, the `yml`/`yaml` rows of `src/generated.rs:136-155`), and `yaml` is highlighted (`src/reader/language.rs:24`). Scouter does the same (yaml is in `scouter:grammars/`), so Compose is already at parity.
- **Every candidate except Perl has a grammar that loads on the locked runtime** with a highlight query that compiles. 26 load from a crate; six more (gitignore, gitattributes, git config, editorconfig, SSH config, and Perl's route) are vendored C, of which five load and Perl has no `parser.c` to vendor.
- **The cost is the parse tables, and it is uneven.** The 14 small config and build grammars add 0.87 MiB to the release binary (+2.3 %). The seven mainstream languages add 25.2 MiB more. All 28 crate grammars together add 45.8 MiB (+122 %, 37.7 to 83.5 MiB). Build time grows by 28 s of CPU at most; wall time is within one run's noise.
- **Dropped:** Perl (no crate loads, and upstream commits no `parser.c`), git commit messages (the only grammar, gbprod's, is WTFPL), and the dotenv grammar (none needed: `.env` reads well as bash).

## 2. How a candidate was tested

- A scratch crate in a fresh `mktemp -d`, with fmc's `Cargo.lock` copied in, `opentui` =0.9.9 (`tree-sitter`) and each candidate pinned `=`. `cargo tree -i tree-sitter` lists 0.27.2 only, through `opentui` and the three crates that depend on `tree-sitter` directly (`tree-sitter-scss`, `tree-sitter-dockerfile-updated`, `tree-sitter-ssh-client-config`, each `>=0.21` or `~0.27`).
- For each grammar: `Grammar::new(language, query).core_captures()` registered with `register` (which compiles it, `opentui-0.9.9/src/syntax/highlight.rs:260`), then `highlights` on a Harry Potter or Dan Brown sample (`highlight.rs:505`). Two queries per grammar: the crate's own, and nvim-treesitter's (`runtime/queries/<lang>/highlights.scm` at `04f17886f75416dfdc25374c950a52c63f0e98fe`, 2026-11-14, Apache-2.0, the pin plan-00018 used), with its `; inherits:` parents prepended and its predicates translated as section 5 says.
- "Styled" means fmc's palette styles the name, exactly or by its first dotted part (`src/ui/syntax/mod.rs:161-196`, `theme`).
- Crates that export the same C symbol (`tree_sitter_kotlin` in both Kotlin crates, `tree_sitter_groovy` in both Groovy crates) were tested in separate crates; linked together, one silently replaces the other (the trap plan-00018 recorded for `tree_sitter_query`).
- Vendored grammars: `parser.c` (and `scanner.c`) from the upstream commit nvim-treesitter pins, built with `cc` 1.11.3 (`-std=c11`, warnings off) as fmc's `build.rs` does, and wrapped as `vendored.rs` does.

## 3. Per-candidate table

Size is the grammar's linked code and data (`size` of the crate's static library, text plus data, release); the measured binary deltas in section 6 are about 17 % lower in total, because the linker drops unused objects (OCaml's interface and type grammars, for one). Licence is the crate's `license` and the `LICENSE` file it ships; "none shipped" means the archive has no `LICENSE` and the upstream repository's applies. Dates are the version's publish date and the repository's last push.

| Filetype | Crate (version) | tree-sitter dependency | Licence | Maintenance | Size | Crate query | nvim query | Pick |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| dockerfile | `tree-sitter-containerfile` 0.9.2 | `tree-sitter-language` ^0.1 | MIT (with camdencheek's MIT) | 2026-07-04; wharflab, pushed 2026-11-28; 369k recent downloads | 103 KiB | pass, 16/17 styled | fail (`heredoc_line`: newer grammar than nvim's) | crate + crate query |
| dockerfile (alt) | `tree-sitter-dockerfile-orchard` 0.1.0 | `tree-sitter-language` ^0.1 | ISC | 2026-11-11; Codeberg grammar-orchard (the JSON5 crate's org) | 59 KiB | pass, 13/13 | pass, 13/14 | alternative |
| dockerfile (alt) | `tree-sitter-dockerfile-updated` 0.2.0 | `tree-sitter` >=0.21 (resolves to 0.27.2) | MIT (none shipped) | 2025-09-11; a fork of camdencheek | 59 KiB | pass (file only, no const) | pass, 13/14 | rejected: stale fork |
| dockerfile | `tree-sitter-dockerfile` 0.2.0 | `tree-sitter` ^0.20 | MIT | 2024-05 | — | — | — | rejected: second C runtime |
| make | `tree-sitter-make` 1.1.1 | `tree-sitter-language` ^0.1 | MIT (none shipped) | 2024-12-21; tree-sitter-grammars, pushed 2026-09-13 | 145 KiB | pass, 28/31 | fail (`let`: nvim pins a newer grammar) | crate + crate query |
| cmake | `tree-sitter-cmake` 0.7.6 | `tree-sitter-language` ^0.1 | MIT | 2026-10-30; uyha, nvim's upstream | 71 KiB | pass, 41/43 | pass after `\c` → `(?i)`, 36/37 | crate + crate query |
| ini | `tree-sitter-ini` 1.4.0 | `tree-sitter-language` ^0.1 | Apache-2.0 | 2025-12-08; justinmk, nvim's upstream | 4 KiB | pass, 8/9 | pass, 10/11 | crate + crate query |
| properties | `tree-sitter-properties` 0.3.0 | `tree-sitter-language` ^0.1 | MIT (none shipped) | 2025-01-02; tree-sitter-grammars | 10 KiB | pass, 9/9 | pass, 10/11 | crate + crate query |
| proto | `tree-sitter-proto` 0.6.1 | `tree-sitter-language` ^0.1 | MIT | 2026-11-17; coder3101 | 84 KiB | pass, 42/44 | pass, 43/44 | crate + crate query |
| graphql | `tree-sitter-graphql` 0.3.0 | `tree-sitter-language` ^0.1.8 | MIT | 2026-10-02; joowani | 52 KiB | none shipped (its const is behind an unset cfg) | pass, 29/30 | crate + nvim query |
| nix | `tree-sitter-nix` 0.3.0 | `tree-sitter-language` ^0.1 | MIT | 2025-07-18; nix-community, pushed 2026-11-27 | 75 KiB | pass, 55/55 | pass, 41/42 | crate + crate query |
| scss | `tree-sitter-scss` 1.0.0 | `tree-sitter` >=0.21 (resolves to 0.27.2) | MIT (none shipped) | 2024-04-09; tree-sitter-grammars, pushed 2026-11-16 | 163 KiB | pass, 12/13 | fail (nvim pins serenadeai's grammar) | crate + crate query |
| vue | `tree-sitter-vue-next` 0.1.0 | `tree-sitter-language` ^0.1 | MIT | 2026-03-07; a fork of tree-sitter-grammars/tree-sitter-vue, 0 stars | 28 KiB | pass, 30/43 (html_tags + vue files; its const is behind an unset cfg) | pass after dropping one `#set!`, 30/43 | crate + crate query, with its injections |
| gitignore | `konomanoasa-tree-sitter-gitignore` 0.7.3 | `tree-sitter-language` =0.1.9 | MIT | published 2026-11-26, one author, 0 stars | 16 KiB | pass, 12/15 | fail (other grammar) | rejected: see section 4 |
| gitignore | vendored shunsambongi/tree-sitter-gitignore `f4685bf` | — | MIT | nvim's pin | 10 KiB | — | pass, 45/49 | vendored + nvim query |
| gitattributes | vendored tree-sitter-grammars/tree-sitter-gitattributes `1b7af09` | — | MIT | nvim's pin | 20 KiB | — | pass, 14/18 | vendored + nvim query |
| gitattributes | `tree-sitter-gitattributes` 0.1.6 | `tree-sitter` ~0.20.10 | MIT | 2024-02 | — | — | — | rejected: second C runtime |
| git config | vendored the-mikedavis/tree-sitter-git-config `3a61756` | — | MIT | nvim's pin | 11 KiB | — | pass, 22/23 | vendored + nvim query |
| editorconfig | vendored ValdezFOmar/tree-sitter-editorconfig `v2.0.0` | — | MIT | nvim's pin; has `scanner.c` | 10 KiB | — | pass, 13/15 | vendored + nvim query |
| ssh config | vendored tree-sitter-grammars/tree-sitter-ssh-config `0c35b24` | — | MIT | nvim's pin | 270 KiB | repo query pass, 13/14 | pass, 13/15 | vendored + repo query |
| ssh config | `tree-sitter-ssh-client-config` 2026.11.26 | `tree-sitter` ~0.27 | CC0-1.0 (not on the allowlist) | metio, pushed 2026-11-26 | 915 KiB | pass, 13/13 (file only) | fail (other grammar) | rejected: licence, size |
| gitcommit | `tree-sitter-gitcommit` 0.5.0 | `tree-sitter-language` ^0.1 | WTFPL (not on the allowlist) | gbprod, nvim's upstream | 1.6 MiB | pass | pass | dropped: licence |
| cpp | `tree-sitter-cpp` 0.23.4 | `tree-sitter-language` ^0.1 | MIT | 2024-11-11; tree-sitter org, pushed 2026-09-17 | 3.3 MiB | pass, 5/5 (thin: 5 captures on the sample) | pass, 64/68 (c + cpp) | crate + nvim query |
| csharp | `tree-sitter-c-sharp` 0.24.0 | `tree-sitter-language` ^0.1 | MIT | 2026-11-18; tree-sitter org | 6.9 MiB | pass, 57/58 | fail (impossible pattern on this grammar) | crate + crate query |
| php | `tree-sitter-php` 0.25.2 | `tree-sitter-language` ^0.1 | MIT | 2026-11-19; tree-sitter org | 2.0 MiB | pass, 28/30 | pass, 47/50 | crate + crate query, with its HTML injection |
| ruby | `tree-sitter-ruby` 0.23.1 | `tree-sitter-language` ^0.1 | MIT (none shipped) | 2024-11-11; tree-sitter org, pushed 2026-09-13 | 2.0 MiB | pass, 60/63 | pass, 58/59 | crate + crate query |
| kotlin | `tree-sitter-kotlin-sg` 0.4.1 | `tree-sitter-language` ^0.1 | MIT | 2026-05-09; ast-grep's fork of fwcd (nvim's upstream) | 5.5 MiB | pass, 67/71 | pass, 67/73 | crate + crate query |
| kotlin (alt) | `tree-sitter-kotlin-ng` 1.1.0 | `tree-sitter-language` ^0.1 | MIT | 2025-01-09; tree-sitter-grammars | 3.3 MiB | none shipped | fail (other grammar) | rejected: no query fits |
| swift | `tree-sitter-swift` 0.7.4 | `tree-sitter-language` ^0.1.0 | MIT | 2026-10-04; alex-pinkus, pushed 2026-11-28 | 4.0 MiB | pass, 67/68 | pass, 67/68 | crate + crate query |
| dart | `tree-sitter-dart` 0.3.0 | `tree-sitter-language` ^0.1 | MIT | 2026-10-25; nielsenko | 1.4 MiB | pass, 50/50 | fail (nvim pins UserNobody14's) | crate + crate query |
| dart (alt) | `tree-sitter-dart-orchard` 0.7.1 | `tree-sitter-language` ^0.1 | MIT | 2026-10-29; grammar-orchard | 1.0 MiB | pass, 56/57 | fail | alternative |
| scala | `tree-sitter-scala` 0.26.14 | `tree-sitter-language` ^0.1 | MIT | 2026-11-26; tree-sitter org | 4.4 MiB | pass, 45/53 (`method`, `parameter` unstyled) | pass, 57/59 | crate + nvim query |
| haskell | `tree-sitter-haskell` 0.24.1 | `tree-sitter-language` ^0.1 | MIT | 2026-09-30; tree-sitter org | 3.7 MiB | pass, 40/42 | pass, 35/38 | crate + crate query |
| ocaml | `tree-sitter-ocaml` 0.26.0 | `tree-sitter-language` ^0.1 | MIT | 2026-09-13; tree-sitter org | 12.2 MiB (three grammars; one is linked) | pass, 48/49 | pass, 49/51 | crate + crate query |
| r | `tree-sitter-r` 1.3.0 | `tree-sitter-language` ^0.1 | MIT | 2026-06-19; r-lib | 474 KiB | pass, 44/46 (`conditional` to rename) | pass, 46/47 | crate + crate query |
| powershell | `tree-sitter-powershell` 0.26.6 | `tree-sitter-language` ^0.1 | MIT | 2026-10-28; airbus-cert | 992 KiB | pass, 17/19 (thin) | pass, 52/53 | crate + nvim query |
| objc | `tree-sitter-objc` 3.0.2 | `tree-sitter-language` ^0.1 | MIT (none shipped) | 2024-12-16; tree-sitter-grammars | 5.1 MiB | pass, 27/34 | pass, 88/89 (c + objc) | crate + nvim query |
| groovy | `dekobon-tree-sitter-groovy` 0.6.0 | `tree-sitter-language` ^0.1.7 | Apache-2.0 OR MIT | 2026-11-13; dekobon | 1.4 MiB | pass, 23/23 | fail (nvim pins murtaza64's) | crate + crate query |
| groovy (alt) | `tree-sitter-groovy` 0.1.2 | `tree-sitter-language` ^0.1 | MIT | 2024-11-19; amaanq | 995 KiB | none shipped | fail | rejected: no query fits |
| perl | `tree-sitter-perl` 1.1.2 | `tree-sitter` ^0.26.3 | MIT | ganezdragon's grammar, not nvim's | — | — | — | fails: `links` clash with 0.27 (`cargo fetch`: "package `tree-sitter` links to the native library `tree-sitter`, but it conflicts") |
| perl | `tree-sitter-perl-next` 0.1.0, 0.1.1 | — | MIT OR Apache-2.0 | both versions yanked | — | — | — | fails: yanked |

Every grammar's ABI is 13 to 15, all loadable by 0.27.2. Each picked crate adds exactly one package to the lock (162 → 190 packages for all 28; no transitive additions), and `cargo deny check licenses` passes on the all-crates tree ("licenses ok").

## 4. Why the git files are vendored, not taken from crates

The only crates for gitignore, gitattributes, git config, and editorconfig are four `konomanoasa-*` crates published on 2026-11-26 by one author, with 0 stars and an exact pin on `tree-sitter-language` =0.1.9 (a pin that would hold back any later bump of that crate). nvim-treesitter's queries do not fit their grammars. The upstream grammars nvim-treesitter pins are MIT, commit their `parser.c`, are 10 to 20 KiB each, load with nvim's queries (section 3), and fit fmc's existing `build.rs` and `vendored.rs` path (`build.rs:5`, `src/ui/syntax/vendored.rs:6-11`). On the gitignore sample nvim's query on the upstream grammar gives 49 captures; the konomanoasa crate's gives 15.

## 5. Highlight queries and the predicates OpenTUI does not evaluate

- **What OpenTUI evaluates.** tree-sitter 0.27.2 checks the text predicates itself: `#eq?`, `#not-eq?`, `#match?`, `#not-match?`, `#any-of?`, `#not-any-of?` and their `any-` forms (`tree-sitter-0.27.2/binding_rust/lib.rs:2712-2841`). Any other predicate is kept as a general predicate (`lib.rs:2987`), which OpenTUI never reads (`opentui-0.9.9/src/syntax/highlight.rs` has no `general_predicates` call; it reads `property_settings` for injections only, `:381`). So an unknown predicate does not fail: its pattern matches with no condition.
- **`#lua-match?` and `#not-lua-match?`** become `#match?` and `#not-match?`, with Lua's classes written as regex (`%d` → `\\d`, `%u` → `[A-Z]`, `%a` → `[A-Za-z]`, `%w` → `\\w`, `%s` → `\\s`, `%.` → `\\.`), as plan-00018 did for the tsquery query (`src/ui/syntax/queries/query/highlights.scm:2`). Uses in the picked nvim queries: graphql 0, cpp 7 (+4 from c), scala 11, powershell 3, objc 0 (+4 from c), and the four git-family queries 2 (git config).
- **`#has-ancestor?`, `#has-parent?`, `#not-has-ancestor?`, `#not-has-parent?`** (nvim's own) are ignored, so their patterns over-match. In the picked queries: c (inherited by cpp and objc) 4, cpp 4, objc 1. Each is either rewritten as a structural pattern (`(field_declaration (field_identifier) @property)` for c's `has-ancestor? field_declaration`) or dropped; the plan decides per pattern, and the header of each copied query names the change.
- **`#offset!`** is not applied: powershell's two (`-regex` strings and here-strings) paint the quotes too. Accepted, as plan-00018 recorded for Lua.
- **`#set!` with a capture and a value** (`(#set! @string.special.url url @string.special.url)` in nvim's `html_tags`) does not compile on 0.27.2 ("Invalid arguments to set! predicate"); it is dropped. nvim's `(#set! @_template bo.commentstring …)` in vue is dropped too.
- **Vim's `\c`** (ignore case) in cmake's nvim query is not a Rust regex; it becomes `(?i)`. The pick uses cmake's crate query, so this applies only if the plan switches.
- **`@spell`, `@nospell`, `@none`, `@conceal`** are dropped, as plan-00018 did (fmc has no spell check).
- **Unstyled names to map** (plan-00006's method): `module` (cpp, csharp, php, proto, haskell, ocaml, ssh config) to `module` is already in `CAPTURE_NAMES` (`src/ui/syntax/mod.rs:274`) but unstyled, so it is mapped to `namespace` or left plain; `character.special` (git family, scala, haskell) to `string.special`; scala's crate `method`, `method.call`, `parameter` are moot with the nvim query; r's `conditional` to `keyword.conditional`; ruby's `embedded` is plain by design (interpolated code); powershell's crate `array`, `assignvalue` are moot with the nvim query.
- **Licences of the queries.** A crate query is the crate's licence (the `LICENSE` beside it, from the archive or the upstream repository where the archive has none). An nvim query is Apache-2.0 with nvim-treesitter's `LICENSE`, as hcl's is (`rules.md`, "Licenses"). The vendored repository's own query (ssh config) is MIT.
- **Injections.** Vue needs its `<script>` and `<style>` injected (JavaScript, TypeScript, CSS, SCSS), or a `.vue` file is mostly plain: the crate ships `queries/vue/injections.scm`. PHP needs its text outside `<?php` injected as HTML (`injections-text.scm` and `injections.scm` in the crate). Dockerfile's `RUN` lines as bash are in the containerfile crate's `injections.scm`. Markdown fences reach every new filetype by name with no change (`filetype_for_info_string("Dockerfile")` gives `dockerfile`, `filetype.rs:83`).

## 6. Cost, measured

Clean `cargo build --release` of fmc HEAD (`85256b9`) and of three scratch trees with crates added and registered, on this machine, one run each, no compiler cache (`RUSTC_WRAPPER` empty), a fresh target directory each time. The vendored grammars of section 4 were not in these builds; their objects total about 0.3 MiB (ssh config 270 KiB, the git family 51 KiB).

| Build | Grammars added | Binary | Delta | Wall | User CPU | Lock packages |
| --- | --- | --- | --- | --- | --- | --- |
| HEAD | — | 39,510,272 B (37.7 MiB) | — | 48.0 s | 197.7 s | 162 |
| Wave 1 | dockerfile, make, cmake, ini, properties, proto, graphql, nix, scss, vue, and the four konomanoasa crates (stand-ins for the vendored git family, same size class) | 40,422,000 B | +0.87 MiB (+2.3 %) | 48.9 s | 205.2 s | 176 |
| Waves 1 + 2 | + cpp, ruby, php, csharp, kotlin, swift, dart | 65,889,440 B | +25.2 MiB (+66.8 %) | 46.7 s | 217.9 s | 183 |
| All | + scala, haskell, ocaml, r, powershell, objc, groovy | 87,582,256 B | +45.8 MiB (+121.7 %) | 56.6 s | 225.7 s | 190 |

For scale: the release binary today is mostly the OpenTUI native library (30 MiB static), and the largest grammar already linked is SQL (`tree-sitter-sequel`, 12.7 MiB). The wall times move within one run's noise; the CPU time grows by at most 28 s (+14 %).

## 7. Scouter

Scouter v0.3.4 (`434d000f`) highlights `javascript`, `javascriptreact`, `typescript`, `typescriptreact`, `markdown`, `markdown_inline`, `zig` (`scouter:src/reader/code-view.ts:20-29`) and the extra languages of its manifest, which are the folders of `scouter:grammars/`: bash, c, css, go, html, java, json, python, rust, toml, yaml. None of the candidates is in that set, so each new filetype is an approved-deviation record (`architecture.md`, as "the owner's extra grammars" and plan-00018's are) and a `docs/parity.md` line. Compose stays at parity: both apps highlight it as YAML. A `Dockerfile` shows plain in Scouter (`pathToFiletype` gives `dockerfile`, which `supportedLanguages` lacks).

## 8. File names

OpenTUI already names `Dockerfile`, `Containerfile` (`dockerfile`), `Makefile`, `GNUmakefile`, `makefile` (`make`), `Gemfile`, `Rakefile`, `Podfile` (`ruby`), and the extensions `cpp cc cxx hpp hh c++ h++ cs rb rake gemspec php kt kts swift scala sc dart hs ml mli r ps1 psm1 vue scss groovy ini` (`opentui-0.9.9/src/generated.rs:136-169`). It returns nothing for these, measured with `filetype_for_path` in the scratch crate:

| Path | Proposed filetype | Neovim's rule (`/opt/homebrew/share/nvim/runtime/lua/vim/filetype.lua`, nvim 0.12.6) |
| --- | --- | --- |
| `Dockerfile.dev`, `Containerfile.prod` | dockerfile (prefix `dockerfile.`, `containerfile.`) | `^Dockerfile%.`, `^Containerfile%.` (`:2604-2605`) |
| `app.dockerfile`, `app.containerfile` | dockerfile (extension) | `dockerfile` extension (`:446`) |
| `x.mk`, `x.mak` | make | `mk`, `mak` (`:803-804`) |
| `CMakeLists.txt`, `x.cmake` | cmake | `CMakeLists.txt` (`:1596`) |
| `.env`, `.env.local`, `x.env` | bash | `.env` and `^%.env%.` are `env` (`:1670`, `:2697`), which has no tree-sitter parser; bash is fmc's choice, not Neovim's |
| `.gitignore`, `.dockerignore`, `.npmignore`, `.prettierignore`, `.ignore`, `.rgignore`, `.fdignore`, `.containerignore`, `.vscodeignore` | gitignore | `:569`, `:1700-1707` |
| `.gitattributes` | gitattributes | `:1699` |
| `.gitconfig`, `.gitmodules` | gitconfig | `:1697-1698` |
| `.editorconfig` | editorconfig | `:1668` |
| `ssh_config`, `~/.ssh/config` | sshconfig (`ssh_config` by name; `config` only under a `.ssh` folder) | `:1964`, `:2501` |
| `x.proto` | proto | `:1053` |
| `x.graphql`, `x.gql`, `x.graphqls` | graphql | `:591-593` |
| `x.nix` | nix | — (nix is not in the grep; the extension is universal) |
| `x.properties` | properties | `:700` (`jproperties`) |
| `x.gradle`, `Jenkinsfile` | groovy | `:597`, `:1723` |
| `x.sbt` | scala | — |
| `x.m`, `x.mm` | objc (`x.m` is also MATLAB) | — |
| `x.psd1` | powershell | — |

A path rule (`~/.ssh/config`) has no home in `NAMES` (`src/reader/language.rs:63`), which matches whole lowercased names; it needs a parent-folder check or a `file_associations` example instead.

## 9. Recommendation

Ranked by daily use against cost:

1. **Wave 1, config and build files (≈1.2 MiB with the vendored ones):** dockerfile, make, cmake, `.env` as bash, gitignore, gitattributes, git config, editorconfig, ini, properties, proto, graphql, nix, scss, vue, ssh config. These are in almost every repository a developer opens, and together they cost less than 3 % of the binary.
2. **Wave 2, mainstream languages (+25.2 MiB):** C++, Ruby, PHP, C#, Kotlin, Swift, Dart. Common, but each costs 1.4 to 6.9 MiB. By cost per daily user, C++, Ruby, and PHP (7.4 MiB together) come first; C# (6.9 MiB) and Kotlin (5.5 MiB) are the heaviest.
3. **Wave 3, niche and heavy (+20.7 MiB):** Scala, Haskell, OCaml, Objective-C, R, PowerShell, Groovy. Park in the roadmap with "on request" as the unlock.
4. **Dropped:** Perl, git commit messages, Erlang, Clojure, Julia, LaTeX (OpenTUI names them, but none is a daily file for most developers).

## 10. Unverified

- Highlights on screen: the samples prove compile and capture counts, not colours; the plan's live run checks them.
- The per-pattern rewrites of `#has-ancestor?` and friends (section 5) are not written yet.
- MSRV of the new crates (none was checked against 1.92), and builds on targets other than macOS arm64.
- The measured binary sizes are one run each; sizes are deterministic, times are not.

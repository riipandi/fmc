---
status: in-progress
created: 2026-11-28T14:53:33+07:00
updated: 2026-11-28T16:10:00+07:00
phases:
    - id: 1
      title: The crate grammars, query, json5, and just
      status: done
      note: query and json5; just is vendored in phase 2 (O3 (a))
    - id: 2
      title: The vendored grammars, vimdoc, luadoc, luap (and just under O3 (a))
      status: done
      note: runs, O2 (a)
    - id: 3
      title: Lua's injections for luadoc and luap
      status: pending
      note: runs after phase 2
    - id: 4
      title: Docs sync, agent docs sync, ledger close, handover
      status: pending
      note: —
    - id: 5
      title: Final validation and live run
      status: pending
      note: —
---

## Highlight the rest of the owner's grammar set — RI-17, on crates that load and, if approved, vendored parsers

### Objective

RI-17 parks the seven filetypes of the owner's Neovim set that had no crate loading on the locked `tree-sitter` 0.27 (`roadmap-00001-post-parity.md:210-216`). Research on 2026-11-28 found crates that load for three of them and a vendored-parser route for three more; eex has no licence. After this plan, fmc highlights every filetype in the set that has a licensed grammar, each with its upstream query and `LICENSE`, and the roadmap records eex's block. The plan adds dependencies only as the owner approves them below.

### Current state (verified in the tree, 2026-11-28)

- **Registration** — each grammar is `register_lazy("<filetype>", || grammar(<crate>::LANGUAGE, include_str!("queries/<filetype>/highlights.scm")))` in `src/ui/syntax/mod.rs` (lua at `:92`), with an optional `injections.scm`, and the query's `LICENSE` beside it (`rules.md`, "Licenses"). `HIGHLIGHTED` in `src/reader/language.rs` lists the filetypes (`:33` has `lua`); `EXTENSIONS` (`:44`) maps extensions OpenTUI lacks; nothing maps a whole file name (`justfile`).
- **The lock** — `tree-sitter` 0.27.2 and `tree-sitter-language` 0.1.9 (`Cargo.lock`), through `opentui` =0.9.9 with `tree-sitter`. `cc` 1.11.3 is in the lock only as a transitive dependency; there is no `build.rs`. `deny.toml` has no `[sources]` section.
- **Lua has no injections** — `src/ui/syntax/queries/lua/` holds `highlights.scm` and `LICENSE` only.
- **Measured on 2026-11-28 in a scratch crate on the same lock (research, not the tree):**
  - Loads from crates.io: `tree-sitter-tsquery` 0.8.0 (query; Apache-2.0; depends on `tree-sitter-language` ^0.1; its query is upstream's and nvim's), `tree-sitter-json5-orchard` 0.1.1 (json5; MIT; a fork of the grammar nvim uses), `codebook-tree-sitter-just` 0.1.3 (just; MIT; a derivative grammar, 870 lines from casey's `grammar.js`).
  - Does not load: `tree-sitter-json5` 0.1.0, `tree-sitter-luadoc` 1.1.0, `tree-sitter-luap` 1.0.0, `tree-sitter-query` 0.1.0 (pin `tree-sitter` 0.20, a second C runtime), `tree-sitter-just` 0.2.0 and `tree-sitter-vimdoc` 4.3.0 (pin 0.25, refused by the shared `links = "tree-sitter"` key).
  - Loads when vendored (`parser.c` built with `cc`): vimdoc (neovim pin `cf5f037`, Apache-2.0), luap (`c134aae`, MIT), just (casey `5685543`, Apache-2.0), json5 (Joakker `248b856`, MIT). luadoc loads as a git dependency at `4d04632` (MIT), the nvim pin.
  - eex (connorlay/tree-sitter-eex `f742f2f`) loads but has no licence (no `LICENSE`, GitHub reports none).
  - `tree-sitter-tsquery` and `konomanoasa-tree-sitter-query` both export `tree_sitter_query`; linked together, one silently replaces the other. Only one may be linked.
- **A wording error** — RI-17 and `research/grammar-load-spike.md` section 5 say the 0.20 pins clash on `links`; only the 0.25 pins do. A 0.20 pin links, with a second runtime.

### Decisions

- **D1 (2026-11-28): one grammar per filetype, registered as the twelve are.** Each new filetype joins `HIGHLIGHTED`, `register_lazy`, and gets `queries/<filetype>/highlights.scm` with its upstream `LICENSE`; capture names are mapped to fmc's palette as plan-00006 did (`research/grammar-load-spike.md`). Rejected: a second registration path for vendored grammars — a vendored parser exposes the same `LANGUAGE` const through a small `extern "C"` wrapper.
- **D2 (2026-11-28): file-name rules join the extension table.** `src/reader/language.rs` gains `NAMES` (`justfile`, `Justfile`, `.justfile` → `just`) beside `EXTENSIONS` (`json5` → `json5`, `just` → `just`, `scm` → `query`). vimdoc gets no default rule: a `.txt` file is not help text unless it lives in a `doc/` folder of a Vim plugin; the docs show `"doc/*.txt": "vimdoc"` for `file_associations`. luadoc and luap are injection-only (no file of their own). Rejected: a `doc/*.txt` default — wrong for every other `doc/` folder.
- **D3 (2026-11-28): `scm` maps to `query`.** fmc's own `queries/**/*.scm` files and any tree-sitter query become highlighted. Rejected: leaving `.scm` plain — Scheme is the other common `.scm`, and fmc has no Scheme grammar either, so the query grammar wins by use.
- **D4 (2026-11-28): the roadmap and the spike are corrected** in the docs phase: the `links` clash is the 0.25 pins'; the 0.20 pins would link a second C runtime.
- **O1 (owner, 2026-11-28: yes): approve three crates.** `tree-sitter-tsquery` =0.8.0 (Apache-2.0), `tree-sitter-json5-orchard` =0.1.1 (MIT), each pinned exactly as the other grammar crates [recommended: they load on the lock with no vendored C]. Per O3, `codebook-tree-sitter-just` =0.1.3 (MIT) too.
- **O2 (owner, 2026-11-28: (a)): vendored parsers.** (a) Approve `cc` (the 1.11.3 already in the lock) as a build-dependency and a `build.rs` that compiles vendored `parser.c` (and `scanner.c` where the grammar has one) for vimdoc, luadoc, and luap, each under `vendor/grammars/<name>/` with its `LICENSE` and the upstream commit in a `SOURCE` line [recommended: one mechanism for the three filetypes no crate can serve; luadoc vendored instead of a git dependency keeps every source on crates.io or in the tree]. (b) No vendoring: vimdoc, luadoc, and luap stay parked in RI-17 with their measured routes.
- **O3 (owner, 2026-11-28: (a)): which just grammar.** (a) Vendored casey `5685543` (Apache-2.0), the grammar nvim uses, built by O2 (a)'s `build.rs` [recommended when O2 (a): it matches the owner's editor]. (b) The `codebook-tree-sitter-just` crate (MIT, a derivative), when O2 is (b).
- **O4 (owner, 2026-11-28: (a)): eex.** (a) Leave eex parked in RI-17 with the unlock "a licence from the author", and ask upstream only if the owner wants [recommended: an unlicensed grammar cannot ship]. (b) Drop eex from the set.
- **O5 (owner, 2026-11-28: (a)): the new top-level directory.** Vendoring under O2 (a) creates `vendor/` at the repository root, which `AGENTS.md` reserves to the owner. (a) `vendor/grammars/` at the root [recommended: the conventional place, outside `src/`]. (b) `src/ui/syntax/vendor/`, no new top-level directory.

### Shape and enforcement points

| What | Where | The record |
| --- | --- | --- |
| The approved crates and `cc` | `rules.md`, "Dependencies", a table for plan-00018 | Crate, version, licence, purpose |
| The vendored sources | `rules.md`, "Licenses"; each `vendor/grammars/<name>/LICENSE` and `SOURCE` | Upstream, commit, licence |
| The reasoning | `architecture.md`, a record beside "the owner's extra grammars" | D1 to D4, O1 to O5, the rejected routes |
| The Rust contract | `rust.md` | A vendored grammar's `extern "C"` symbol is unique; two crates exporting one symbol cannot both link |
| The layout | `shape.md` | `build.rs`, `vendor/grammars/`, `NAMES` |
| The roadmap and the spike | `roadmap-00001-post-parity.md` RI-17; `research/grammar-load-spike.md` section 5 | Landed in part or landed; eex's block; the `links` wording |
| The user docs | `docs/usage.md` (the highlighted list), `docs/configuration.md` (the vimdoc association), `CHANGELOG.md` | New filetypes |

### Execution rules

- **The owner chose every recommendation and gave the go-ahead for all phases in order (2026-11-28).** `Cargo.toml` edits ask for permission (`.claude/settings.json`); the approved versions are the ones above, and each phase confirms they still resolve.
- **Every commit ends with** `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. No `Scouter-Base` trailer: Scouter has none of these grammars.
- **`cargo deny check licenses` passes after each phase**; vendored C is covered by its `LICENSE` and `about.toml` notices where `cargo about` cannot see it (the notices template lists it by hand).
- **Branch:** `feat/remaining-grammars`, from `main`.

### Phases

1. **The crate grammars** — `tree-sitter-tsquery`, `tree-sitter-json5-orchard` (and `codebook-tree-sitter-just` under O3 (b)) in `Cargo.toml`; queries and `LICENSE` copied from each crate; `register_lazy`; `HIGHLIGHTED`; `EXTENSIONS` and `NAMES` (D2, D3). Tests in `src/ui/syntax/tests.rs` and `src/reader/language_tests.rs` (or the module's tests): each grammar compiles with its query; each capture is one the palette styles; `justfile`, `x.json5`, `x.scm` resolve; a sample highlights at least one non-default span. Validation: `cargo test syntax`, `cargo test language`, `cargo deny check licenses`. _Commit: `feat(syntax): highlight query, json5, and just files`._
2. **The vendored grammars (O2 (a) only)** — `vendor/grammars/{vimdoc,luadoc,luap[,just]}/` (`parser.c`, `scanner.c` where present, `tree_sitter/parser.h`, `LICENSE`, `SOURCE`), `build.rs` compiling each with `cc` (warnings off for upstream C, `-std=c11`), an `extern "C"` wrapper per grammar giving `LanguageFn`; registration and queries as phase 1; vimdoc reachable through `file_associations` and `m`. Tests as phase 1; `cargo build --release` once, for the build script. Undo: revert the phase commit; nothing else depends on it until phase 3. Validation: `cargo test syntax`; `cargo deny check licenses`; `cargo about generate about.hbs -o target/THIRD_PARTY_LICENSES.html` lists the vendored licences. _Commit: `feat(syntax): highlight vimdoc, luadoc, and luap from vendored parsers`._
3. **Lua's injections (after phase 2)** — `queries/lua/injections.scm` from nvim-treesitter (Apache-2.0, its header names the source): luadoc in `---` comments, luap in the pattern argument of `string.match`, `gsub`, `find`, and `gmatch`; registered as the RI-18 injections are. Tests: a Lua sample's `---@param` and `"%d+"` carry the injected captures (the RI-18 test pattern). Validation: `cargo test syntax`. _Commit: `feat(syntax): luadoc and luap inside Lua`._
4. **Docs sync, agent docs sync, ledger close, handover** — per the table above; RI-17 to `landed` or `landed in part` with what stays (eex, and the three vendored ones under O2 (b)); the spike and roadmap wording (D4); the plan archived with its index row; a handover. Validation: `rumdl fmt --check --quiet .`; `grep -c` of the roadmap status rows equals the entries. _Commit: `docs: the rest of the owner's grammar set`._
5. **Final validation and live run** — the matrix; a live run in a new Rex tab of this session opening a `justfile`, a `.json5`, a `.scm` from `src/ui/syntax/queries/`, and (phase 2) a Vim help file with the association and a Lua file with `---@param`, each at 120 columns. _Commit: none when green._

### Test matrix

| What | How | Green |
| --- | --- | --- |
| Format | `cargo fmt --check`; `rumdl fmt --check --quiet .` | No drift |
| Unit | `cargo test syntax`, `cargo test language` | Each grammar compiles with its query and highlights its sample; names and extensions resolve |
| Lint | `cargo clippy --all-targets -- -D warnings` | 0 warnings |
| Build | `cargo build`; `cargo build --release` once (phase 2) | Builds; the build script runs in both profiles |
| Licences | `cargo deny check licenses`; `cargo about generate …` | Pass; vendored licences listed |
| Full suite | `just verify` | All green |
| Live run | Rex tab, the files above at 120 columns | Coloured as the palette's roles, no plain block where a grammar applies |
| E2E | n/a beyond the live run: no CLI change | — |

### Out of scope

- eex until it has a licence (O4).
- Scheme highlighting for `.scm` (D3).
- Grammars outside the owner's Neovim set.
- Injections other than Lua's (RI-18 covers the rest).

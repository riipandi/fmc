# Rules

updated: 2026-11-27T15:10:00+07:00

The cross-area rules behind `AGENTS.md`. Rust code contracts: `rust.md`. Reasoning and decisions: `architecture.md`. Read the section a change touches before touching it.

## Parity with Scouter

Scouter (<https://github.com/indrazm/scouter>, MIT) is the reference: a read-only terminal file browser with a Markdown and Mermaid preview and a git diff viewer, written in TypeScript on `@opentui`. Phase 1 goal: 100% visual and functional parity with it. Later phases grow fmc into a full file manager.

- **Parity-first.** A behavior difference is a defect unless the owner approved it and the approval is recorded in `architecture.md`.
- **Verify before porting.** Establish a Scouter behavior from its source or from a live run through Rex, never from memory or from this file. Scouter's `docs/decision/` records (0001 to 0037) and its `src/` tree (`app/`, `core/`, `explorer/`, `reader/`, `diff/`, `find/`, `ui/`) are the starting map. Record the evidence in the plan or issue that carries the port.
- **Read-only until parity is closed.** No file-writing feature (create, rename, move, delete, save, or any other write to the user's files) until the owner opens that phase.
- **Library source.** The UI crates `opentui` and `opentui-sys` are read from the locked version (`Cargo.lock`), in the copy under `~/.cargo/registry/src/`. Their source wins over docs, READMEs, and examples.
- **Baseline.** fmc ports Scouter v0.3.4, commit `434d000f0f934fb04fced8b1bc1d0e673c6ed4fe`. Every deviation record in `architecture.md` and the top of `docs/parity.md` names it once for the whole page. A commit that ports Scouter behavior ends with the trailer `Scouter-Base: v0.3.4 (434d000f)`. Moving the baseline is an owner decision; it re-checks every deviation and `research/scouter-inventory.md` against the new commit.
- **Every deviation is documented.** An owner-approved difference lands in the same commit as its `architecture.md` record (rationale) and its `docs/parity.md` line (what users see). An unrecorded difference is a defect.
- **Parity reference.** Scouter at commit `434d000` with `@opentui/core` bumped to 0.9.9, run from a scratch checkout outside this repository (`architecture.md`). Research snapshots of its behavior live in `research/`.

## Where a fact lives

| A change that lands… | updates |
| --- | --- |
| A Rust contract or trap | `rust.md` |
| A contract or trap across areas | this file |
| A decision, its rejected alternatives, or an approved parity deviation | `architecture.md`; a deviation also gets a line in `docs/parity.md` |
| A path, tool, or command | `shape.md` |
| A dependency approved, or a license rule | this file ("Dependencies", "Licenses") |
| A new or moved agent document | `index.md` |
| A user-visible behavior | the `docs/` page it belongs to (`usage.md` for a screen, `keymaps.md` for a key, `configuration.md` for a settings key, `cli.md` for an argument, `parity.md` for a difference from Scouter, `install.md` or `development.md` for setup and releases; `docs/index.md` lists them), `README.md` when the front page changes, and the `[Unreleased]` section of `CHANGELOG.md` (Keep a Changelog) |
| Product purpose, users, principles | `PRODUCT.md` |
| A palette token, layout metric, or component look | `DESIGN.md` (describes Scouter's look during parity; Scouter wins on disagreement) and `theme.rs` once it exists |

## Dependencies

- A new dependency needs the owner's approval. Declare it once in `[dependencies]` of `Cargo.toml`, at the version the owner approved.
- `cargo add`, `cargo remove`, `cargo install`, and edits to `Cargo.toml` are behind a permission prompt (`.claude/settings.json`).
- No backward compatibility (early-stage project): a replaced name, shape, or stored value is removed, not aliased or migrated.

### Approved for phase 1 (owner, 2026-11-26)

Versions are the crates.io snapshot of 2026-11-26 (`research/dependency-survey.md`); each lands in the plan phase that first uses it, and the phase confirms the version still resolves. Anything not listed needs a new approval.

| Crate | Version | Features | For |
| --- | --- | --- | --- |
| `opentui`, `opentui-sys` | `=0.9.9` | default (`static`), `tree-sitter` | UI engine and native library binding |
| `clap` | `4.6` | `default-features = false`, `std`, `derive`, `help`, `usage` | Command line |
| `anyhow` | `1.0` | default | Error plumbing |
| `imagesize` | `0.15` | default | Image width, height, and format from the header |
| `icu_collator` | `2.4` | default (`compiled_data`) | Tree sort order (`Intl.Collator` parity) |
| `tree-sitter-json` `=0.24.8`, `tree-sitter-css` `=0.25.0`, `tree-sitter-html` `=0.23.2`, `tree-sitter-go` `=0.25.0`, `tree-sitter-python` `=0.25.0`, `tree-sitter-bash` `=0.25.1`, `tree-sitter-yaml` `=0.7.2`, `tree-sitter-toml-ng` `=0.7.0`, `tree-sitter-c` `=0.24.2`, `tree-sitter-java` `=0.23.5` | exact | default | The grammars Scouter vendors beyond OpenTUI's bundled set |
| `tree-sitter-javascript` `=0.25.0`, `tree-sitter-typescript` `=0.23.2`, `tree-sitter-md` `=0.5.4`, `tree-sitter-zig` `=1.1.2` | exact (the versions `opentui` already links) | default | Language handles to register Core's highlight queries for JS, TS, TSX, JSX, Markdown, and Zig (owner, 2026-11-26) |
| dev: `insta` `1.49`, `assert_cmd` `2.2`, `tempfile` `3.27` | | default | Snapshots, CLI tests, fixture repositories |

Tools (installed, not crate dependencies): zig `0.17.1` (native library build), `cargo-deny` (license and advisory gate), `cargo-about` (third-party notices), kache `2.4.0` (Apache-2.0, compiler cache for the just tasks and CI; owner, 2026-11-27). Std covers git (`std::process::Command`), the filesystem, file-type checks, text search, and temp-file guards; clipboard and terminal theme come from `opentui`.

### Approved for plan-00006 phase 2 (owner, 2026-11-27)

Decision O4 of `plan-00006-mermaid-grammars-fuzzy.md`, with the MSRV raised from 1.90 to 1.92 (`Cargo.toml`, `rust.md`).

| Crate | Version | Features | For |
| --- | --- | --- | --- |
| `mermaid-text` | `=0.58.0` | default (it declares none) | Mermaid diagrams as box text in rendered Markdown (`src/reader/mermaid.rs`); MIT; MSRV 1.92 |
| `unicode-width` | `0.2.3` | default | Display width of the Mermaid frame's rows (`src/reader/mermaid.rs`); MIT OR Apache-2.0; `mermaid-text` uses it too |

Dependencies that come with `mermaid-text` and are not declared in `Cargo.toml`: `ascii-dag` 0.9.1, `chrono` 0.4.45 (with `std` and `clock`, `default-features = false`, as `mermaid-text` declares it), `iana-time-zone` 0.1.65, `core-foundation-sys` 0.8.7, `num-traits` 0.2.19, and `autocfg` 1.5.1 (a build dependency of `num-traits`). These are the crates `cargo tree` lists for `aarch64-apple-darwin`. `Cargo.lock` also holds packages that only other targets build (`wasm-bindgen`, `js-sys`, the `windows-*` crates, `android_system_properties`, `iana-time-zone-haiku`, `futures-*`, `log`, `slab`, `pin-project-lite`, `rustversion`, `bumpalo`). `cargo deny check licenses` passes with all of them.

### Approved for plan-00006 phase 4 (owner, 2026-11-27)

Decision O2 of `plan-00006-mermaid-grammars-fuzzy.md`: every grammar crate that loads on the locked `tree-sitter` 0.27.2 (`research/grammar-load-spike.md`, sections 2 and 4). Each is pinned exactly and is registered in `src/ui/syntax/mod.rs`. Each brings no other crate: the lock gains these twelve packages and no transitive ones.

| Crate | Version | Features | For |
| --- | --- | --- | --- |
| `tree-sitter-diff` | `=0.1.0` | default | Diff and patch files (`diff`); MIT |
| `tree-sitter-elixir` | `=0.3.5` | default | Elixir (`.ex`, `.exs`); Apache-2.0 |
| `tree-sitter-heex` | `=0.8.1` | default | HEEx templates (`.heex`); MIT |
| `tree-sitter-hcl` | `=1.1.0` | default | HCL and Terraform (`.hcl`, `.tf`, `.tfvars`); Apache-2.0; ships no query, so fmc's query is adapted from nvim-treesitter (Apache-2.0, D7) |
| `tree-sitter-jsdoc` | `=0.25.0` | default | JSDoc, reached by association or fence; MIT |
| `tree-sitter-lua` | `=0.5.0` | default | Lua (`.lua`); MIT |
| `tree-sitter-regex` | `=0.25.0` | default | Regular expressions, reached by association or fence; MIT |
| `tree-sitter-sequel` | `=0.3.13` | default | SQL (`.sql`); MIT |
| `tree-sitter-svelte-ng` | `=1.0.2` | default | Svelte (`.svelte`); MIT |
| `tree-sitter-xml` | `=0.7.0` | default | XML (`.xml`); MIT |
| `tree-sitter-astro-next` | `=0.1.1` | default | Astro (`.astro`); MIT or Apache-2.0 |
| `tree-sitter-vim` | `=0.4.0` | default | Vim script (`.vim`, `.vimrc`); MIT; depends on `tree-sitter` directly (`>=0.21`), which resolves to 0.27.2 |

### Approved for plan-00006 phase 5 (owner, 2026-11-27)

Decision D4 of `plan-00006-mermaid-grammars-fuzzy.md`: the fuzzy name match of the finder (`src/find/mod.rs`, `Matching::Fuzzy`). The owner approved `neo_frizbee` as a dependency on 2026-11-27. It has no required dependencies (its `serde` is optional and off), so the lock gains this one package.

| Crate | Version | Features | For |
| --- | --- | --- | --- |
| `neo_frizbee` | `=0.13.3` | default (`std`) | Fuzzy name matching in the finder; MIT; MSRV 1.89 |

### Approved for after parity (owner, 2026-11-26)

- `serde` (feature `derive`) and `serde_json`: persistent settings and themes (`roadmap-00001-post-parity.md`, RI-9 and RI-10). Not before plan-00001 closes.

## Licenses

- fmc is `MIT OR Apache-2.0` (`architecture.md`).
- Allowed dependency licenses: `MIT`, `Apache-2.0`, `Apache-2.0 WITH LLVM-exception`, `Unicode-3.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`. `cargo deny check licenses` enforces it.
- One documented exception: `opentui-sys` declares `MIT AND LGPL-2.1-or-later` (`opentui` itself is `MIT`) because the upstream release library embeds FFmpeg. fmc links a library built with `-Dvideo=false`, which contains no FFmpeg code; the exception is valid only while that holds.
- Never copy code from a GPL project (sqltui is GPL-3.0): ideas only.
- The vendored highlight queries (`src/ui/syntax/queries/<filetype>/highlights.scm`) are Scouter's copies, or `@opentui/core` 0.9.9's for JavaScript, TypeScript, TSX, Markdown, and Zig, each with its MIT `LICENSE` beside it; a copy keeps both files together.
- The owner's twelve queries (plan-00006 phase 4) are the grammar crates' own, each with its upstream `LICENSE` beside it: MIT for most, Apache-2.0 for elixir and astro (the `LICENSE` each crate ships), and nvim-treesitter's Apache-2.0 `LICENSE` for hcl, whose query is adapted from nvim-treesitter's. The archives of heex, lua, diff, svelte, vim, and xml ship no `LICENSE`, so theirs come from the upstream repository.

## Sample data

Every made-up value (test fixtures, fixture file and directory names, mock files, placeholder copy and labels) names people, places, and things from Dan Brown's novels (Robert Langdon, Sophie Neveu, Leigh Teabing, Katherine Solomon, Vittoria Vetra, the Louvre, CERN) or Harry Potter (Hermione Granger, Harry Potter, Albus Dumbledore, Minerva McGonagall, Hogwarts, Gringotts, Diagon Alley). Never `John Doe`, `Jane`, `Alice`/`Bob`, `foo`/`bar`, `test@`, `dummy`, or lorem ipsum. New copy follows this in every language.

- Addresses and hosts keep reserved names: `<first>.<last>@example.com`, `*.test`; never a real domain (`hogwarts.edu`).
- IPs come from the documentation ranges (`192.0.2.0/24`, `198.51.100.0/24`, `203.0.113.0/24`, `2001:db8::/32`).
- Codes and tokens may borrow spells (`EXPECTO-PATRONUM`), never something resembling a real secret.

## Secrets

- The project holds no secrets today. Never print a secret; never rewrite a `.env` file without consent.

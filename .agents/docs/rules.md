# Rules

updated: 2026-11-26T05:55:13+07:00

The cross-area rules behind `AGENTS.md`. Rust code contracts: `rust.md`. Reasoning and decisions: `architecture.md`. Read the section a change touches before touching it.

## Parity with Scouter

Scouter (<https://github.com/indrazm/scouter>, MIT) is the reference: a read-only terminal file browser with a Markdown and Mermaid preview and a git diff viewer, written in TypeScript on `@opentui`. Phase 1 goal: 100% visual and functional parity with it. Later phases grow fmc into a full file manager.

- **Parity-first.** A behavior difference is a defect unless the owner approved it and the approval is recorded in `architecture.md`.
- **Verify before porting.** Establish a Scouter behavior from its source or from a live run through Rex, never from memory or from this file. Scouter's `docs/decision/` records (0001 to 0037) and its `src/` tree (`app/`, `core/`, `explorer/`, `reader/`, `diff/`, `find/`, `ui/`) are the starting map. Record the evidence in the plan or issue that carries the port.
- **Read-only until parity is closed.** No file-writing feature (create, rename, move, delete, save, or any other write to the user's files) until the owner opens that phase.
- **Library source.** The UI crates `opentui` and `opentui-sys` are read from the locked version (`Cargo.lock`), in the copy under `~/.cargo/registry/src/`. Their source wins over docs, READMEs, and examples.
- **Parity reference.** Scouter at commit `434d000` with `@opentui/core` bumped to 0.9.9, run from a scratch checkout outside this repository (`architecture.md`). Research snapshots of its behavior live in `research/`.

## Where a fact lives

| A change that lands… | updates |
| --- | --- |
| A Rust contract or trap | `rust.md` |
| A contract or trap across areas | this file |
| A decision, its rejected alternatives, or an approved parity deviation | `architecture.md` |
| A path, tool, or command | `shape.md` |
| A dependency approved, or a license rule | this file ("Dependencies", "Licenses") |
| A new or moved agent document | `index.md` |
| A user-visible behavior | `docs/` (`cli.md`, `keymaps.md`, `development.md` as the plan lands them), `README.md`, and the `[Unreleased]` section of `CHANGELOG.md` (Keep a Changelog) |
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

Tools (installed, not crate dependencies): zig `0.17.1` (native library build), `cargo-deny` (license and advisory gate), `cargo-about` (third-party notices). Std covers git (`std::process::Command`), the filesystem, file-type checks, text search, and temp-file guards; clipboard and terminal theme come from `opentui`.

### Approved for after parity (owner, 2026-11-26)

- `serde` (feature `derive`) and `serde_json`: persistent settings and themes (`roadmap-00001-post-parity.md`, RI-9 and RI-10). Not before plan-00001 closes.

## Licenses

- fmc is `MIT OR Apache-2.0` (`architecture.md`).
- Allowed dependency licenses: `MIT`, `Apache-2.0`, `Apache-2.0 WITH LLVM-exception`, `Unicode-3.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`. `cargo deny check licenses` enforces it.
- One documented exception: `opentui-sys` declares `MIT AND LGPL-2.1-or-later` (`opentui` itself is `MIT`) because the upstream release library embeds FFmpeg. fmc links a library built with `-Dvideo=false`, which contains no FFmpeg code; the exception is valid only while that holds.
- Never copy code from a GPL project (sqltui is GPL-3.0): ideas only.
- The vendored highlight queries (`src/ui/syntax/queries/<filetype>/highlights.scm`) are Scouter's copies, or `@opentui/core` 0.9.9's for JavaScript, TypeScript, TSX, Markdown, and Zig, each with its MIT `LICENSE` beside it; a copy keeps both files together.

## Sample data

Every made-up value (test fixtures, fixture file and directory names, mock files, placeholder copy and labels) names people, places, and things from Dan Brown's novels (Robert Langdon, Sophie Neveu, Leigh Teabing, Katherine Solomon, Vittoria Vetra, the Louvre, CERN) or Harry Potter (Hermione Granger, Harry Potter, Albus Dumbledore, Minerva McGonagall, Hogwarts, Gringotts, Diagon Alley). Never `John Doe`, `Jane`, `Alice`/`Bob`, `foo`/`bar`, `test@`, `dummy`, or lorem ipsum. New copy follows this in every language.

- Addresses and hosts keep reserved names: `<first>.<last>@example.com`, `*.test`; never a real domain (`hogwarts.edu`).
- IPs come from the documentation ranges (`192.0.2.0/24`, `198.51.100.0/24`, `203.0.113.0/24`, `2001:db8::/32`).
- Codes and tokens may borrow spells (`EXPECTO-PATRONUM`), never something resembling a real secret.

## Secrets

- The project holds no secrets today. Never print a secret; never rewrite a `.env` file without consent.

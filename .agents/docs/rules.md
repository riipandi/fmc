# Rules

updated: 2026-11-26T03:19:42+07:00

The cross-area rules behind `AGENTS.md`. Rust code contracts: `rust.md`. Reasoning and decisions: `architecture.md`. Read the section a change touches before touching it.

## Parity with Scouter

Scouter (<https://github.com/indrazm/scouter>, MIT) is the reference: a read-only terminal file browser with a Markdown and Mermaid preview and a git diff viewer, written in TypeScript on `@opentui`. Phase 1 goal: 100% visual and functional parity with it. Later phases grow fmc into a full file manager.

- **Parity-first.** A behavior difference is a defect unless the owner approved it and the approval is recorded in `architecture.md`.
- **Verify before porting.** Establish a Scouter behavior from its source or from a live run through Rex, never from memory or from this file. Scouter's `docs/decision/` records (0001 to 0037) and its `src/` tree (`app/`, `core/`, `explorer/`, `reader/`, `diff/`, `find/`, `ui/`) are the starting map. Record the evidence in the plan or issue that carries the port.
- **Read-only until parity is closed.** No file-writing feature (create, rename, move, delete, save, or any other write to the user's files) until the owner opens that phase.
- **Library source.** `opentui_rust` is read from the locked version (`Cargo.lock`), in the copy under `~/.cargo/registry/src/`. Its source wins over docs, READMEs, and examples.

## Where a fact lives

| A change that lands… | updates |
| --- | --- |
| A Rust contract or trap | `rust.md` |
| A contract or trap across areas | this file |
| A decision, its rejected alternatives, or an approved parity deviation | `architecture.md` |
| A path, tool, or command | `shape.md` |
| A new or moved agent document | `index.md` |
| A user-visible behavior | `docs/` (human docs, once the owner asks for them) |

## Dependencies

- A new dependency needs the owner's approval. Declare it once in `[dependencies]` of `Cargo.toml`, at the version the owner approved.
- `cargo add`, `cargo remove`, `cargo install`, and edits to `Cargo.toml` are behind a permission prompt (`.claude/settings.json`).
- No backward compatibility (early-stage project): a replaced name, shape, or stored value is removed, not aliased or migrated.

## Sample data

Every made-up value (test fixtures, fixture file and directory names, mock files, placeholder copy and labels) names people, places, and things from Dan Brown's novels (Robert Langdon, Sophie Neveu, Leigh Teabing, Katherine Solomon, Vittoria Vetra, the Louvre, CERN) or Harry Potter (Hermione Granger, Harry Potter, Albus Dumbledore, Minerva McGonagall, Hogwarts, Gringotts, Diagon Alley). Never `John Doe`, `Jane`, `Alice`/`Bob`, `foo`/`bar`, `test@`, `dummy`, or lorem ipsum. New copy follows this in every language.

- Addresses and hosts keep reserved names: `<first>.<last>@example.com`, `*.test`; never a real domain (`hogwarts.edu`).
- IPs come from the documentation ranges (`192.0.2.0/24`, `198.51.100.0/24`, `203.0.113.0/24`, `2001:db8::/32`).
- Codes and tokens may borrow spells (`EXPECTO-PATRONUM`), never something resembling a real secret.

## Secrets

- The project holds no secrets today. Never print a secret; never rewrite a `.env` file without consent.

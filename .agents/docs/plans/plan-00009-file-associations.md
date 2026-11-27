---
status: draft
created: 2026-11-27T13:30:00+07:00
updated: 2026-11-27T13:30:00+07:00
references: settings.rs, ui/syntax/mod.rs (language_for), reader/language.rs, reader/markdown.rs
phases:
    - id: 1
      title: "Glob matcher and association table (core, pure)"
      status: pending
    - id: 2
      title: "`file_associations` setting wired into every filetype lookup"
      status: pending
    - id: 3
      title: "Docs, live checks, close"
      status: pending
---

## File associations in settings, as VS Code's `files.associations`

### Objective

A `file_associations` object in `settings.json` maps file patterns to a language, so `abc.rs.bak` is highlighted as Rust and `dev.env.example` as an env file:

```json
{
  "file_associations": {
    "*.rs.bak": "rust",
    "*.env.example": "dotenv",
    ".env.*": "dotenv",
    "docs/**/*.txt": "markdown"
  }
}
```

The association decides highlighting everywhere a filetype is looked up (the reader, the finder preview, and the changes screen), and whether `m` renders Markdown. This is a deviation from Scouter v0.3.4 (`434d000f`), whose `languageFor` reads only the extension.

### Current state (2026-11-27)

- `src/ui/syntax/mod.rs:61` `language_for(path)` calls OpenTUI's `filetype_for_path` and keeps the result only if `is_highlighted` (`src/reader/language.rs`). Its callers are `src/app/reader.rs:131`, `src/app/find.rs:449`, and `src/ui/diff.rs:354`.
- Markdown rendering is gated by `reader/markdown.rs` `is_markdown(filetype)`. That check needs tracing in phase 2, to confirm it reads the same filetype.
- `src/settings.rs` parses a JSON object with `serde_json` (no `preserve_order` feature, so object keys come back sorted, not in file order). Bad values fall back with a notice. The file is read once at start.
- OpenTUI has no `dotenv` filetype or grammar. The bash grammar reads `KEY=value` and `# comment` lines well.
- `globset` is not in `Cargo.lock`.

### Decisions (agent; the owner asked for "a setting like VS Code")

- **D1.** The key is `file_associations`, an object of pattern to language. It is snake_case, like the other keys, and follows VS Code's `files.associations` in shape.
- **D2.** The patterns follow VS Code's rule. A pattern without `/` matches the file name; a pattern with `/` matches the path relative to the folder fmc opened. Supported syntax: `*` (no `/`), `**` (any depth), `?`, and `{a,b}`. Matching is case-insensitive on macOS and Windows and case-sensitive elsewhere, like each filesystem. The matcher is hand-written (about 60 lines with tests), not `globset`. Reason: it avoids a new dependency for a small, fully tested function.
- **D3.** The values are fmc filetype names (`rust`, `bash`, `markdown`, `typescriptreact`, ...), plus VS Code language IDs where they differ: `shellscript` maps to `bash`, `jsonc` to `json`, and `dotenv`/`env` to the bash grammar. An unknown value is skipped, with a notice naming the pattern.
- **D4.** Precedence: a matching association beats OpenTUI's own lookup. When several patterns match, a path pattern beats a name pattern, and then the longer pattern wins. Reason: `serde_json` does not keep file order, so "last wins" would not be defined.
- **D5.** Mapping to a language fmc does not highlight (for example `"*.log": "plaintext"`) is allowed and shows plain text. `plaintext` also turns highlighting off for a file OpenTUI would otherwise highlight.
- **D6.** No built-in defaults for env files. The owner's file decides. `docs/configuration.md` shows the env example.
- **D7.** An association to `markdown` also enables `m` (rendered Markdown) for that file.

### Phases

#### Phase 1: Matcher and table (pure)

- `src/reader/associations.rs` (or beside `language.rs`): `Glob::parse`, `Glob::matches(path, name)`, and `Associations::language(relative_path) -> Option<&str>` per D2 to D5.
- Unit tests: `*.rs.bak`, `.env.*`, `*.env.example`, `docs/**/*.txt`, `{a,b}`, `?`, case rules, path versus name precedence, longest wins, an unknown language, and `plaintext`. File names from Harry Potter or Dan Brown (`gringotts.rs.bak`, `louvre.env.example`).

#### Phase 2: Setting and lookup

- `settings.rs`: parse `file_associations` (an object of strings; a bad entry is skipped with a notice, and a non-object value notices and falls back to empty). Tests use a temporary `XDG_CONFIG_HOME`.
- Make `language_for` take the associations (through the model or a once-set global, whichever matches how `settings` reaches the reader today) and pass the path relative to the opened folder. Update all three callers and `is_markdown`'s path. Register the `dotenv` alias to the bash grammar (`register_alias`).
- Tests: a TestApp test that opens `vault.rs.bak` with an association and finds a styled keyword; one for `m` on an associated Markdown file; one with no setting that keeps today's behaviour.
- Live run in a new tab of the agent's own Rex session, with a temporary `XDG_CONFIG_HOME` holding the example above: a `.rs.bak` file, a `.env.example` file, and a `.txt` under `docs/`.

#### Phase 3: Docs and close

- `docs/configuration.md` (the key, the pattern rules, the value names, precedence, the env example), `architecture.md` record (deviation, D1 to D7), `docs/parity.md` (Reader), CHANGELOG, README, `rust.md`.
- Archive the plan, update `index.md`, and list the owner checks.

### Risks

- The relative path depends on the folder fmc was opened on. A file opened by a path argument outside that folder matches by name only.
- Settings are read once at start, so editing associations needs a restart, as with every other key. This is documented.

### Validation

`just verify` (or `KACHE=false just verify`) per phase, plus the live run in phase 2.

### Order

Independent of plans 00007 and 00008. It runs when the owner gives the go-ahead.

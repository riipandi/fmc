---
status: draft
created: 2026-11-27T13:00:00+07:00
updated: 2026-11-27T13:00:00+07:00
references: services/git.rs, ui/reader.rs, settings.rs
phases:
    - id: 1
      title: "Line signs from a zero-context patch (core, pure)"
      status: done
    - id: 2
      title: "Signs in the reader gutter, the `git_gutter` setting, refresh"
      status: pending
    - id: 3
      title: "Docs, live checks, close"
      status: pending
---

## Git change markers in the reader gutter

### Objective

The reader shows, in a one-column sign strip left of the line numbers, which lines of the open file differ from the git base:

- a green `▎` for added lines;
- a yellow `▎` (the palette's `modified`) for changed lines;
- a red `▁` on the line above a deletion, or `▔` on line 1 when the deletion is at the top.

A `git_gutter` setting in `settings.json` turns the strip on or off; it defaults to on. This is a deviation from Scouter v0.3.4 (`434d000f`), whose code view has no signs (`src/reader/code-view.ts:99-108`).

### Current state (2026-11-27)

- **Git.** `src/services/git.rs` resolves a base (`resolve_base`), lists changes against it (`changes`, `listing`), and builds one file's patch against the last listing's base with a chosen context (`patch(change, context)`, line 163). The changes screen (`d`) uses the same base, with staged and unstaged changes together.
- **Gutter.** `src/ui/reader.rs` draws numbers with OpenTUI `LineNumbers`. Unwrapped text keeps a sticky gutter box outside the scroll; wrapped text keeps the numbers inside it. `LineNumbers` has `line_sign(line, LineSign::before(text, color))` and `clear_line_signs` (`opentui-0.9.9/src/widget/line_numbers.rs:26,417-429`).
- **Palette.** `added`, `modified`, and `removed` exist in both palettes (`src/theme.rs`).
- **Settings.** `src/settings.rs` reads `auto_preview`, `sidebar_jump`, and `reader_jump` once at start; a bad value falls back with a notice.
- **Polls.** `src/app/poll.rs` runs the browse poll for the app's life; Git work runs on a worker, not on the UI thread.

### Decisions

Owner, 2026-11-27 (do not reopen):

- **O1.** The setting is `git_gutter`, default `true`.
- **O2.** The style is a coloured bar: `▎` added (green), `▎` modified (yellow), and `▁`/`▔` deleted (red).
- **O3.** The base is the one the changes screen uses (HEAD, staged and unstaged together), so the gutter and `d` agree.

Agent, within those:

- **D1.** The sign strip shows for every text file inside a git repository while `git_gutter` is on, even a file with no changes. The gutter width stays steady as you edit; outside a repository, or with the setting off, the gutter is exactly as today.
- **D2.** Line classes come from `git diff -U0`. In a hunk with `old > 0` and `new > 0`, the first `min(old, new)` new lines are modified and the rest are added. In a hunk with `new == 0`, a deleted mark goes on the line above (on line 1 with `▔` when the deletion starts at the top). A hunk with `old == 0` is all added. An untracked file is all added.
- **D3.** Signs appear only in the text view: raw code, plain text, and wrapped mode. Rendered Markdown, images, binaries, and the changes screen are unchanged.
- **D4.** Signs refresh when the file reloads (the browse poll's content change) and when the change stamp moves. The diff runs on the worker. A stale answer for another file is dropped, keyed by path and reload generation.
- **D5.** A Git failure or timeout clears the signs with no notice. The gutter is a hint, not an error surface.
- **D6.** The sign takes one column inside the gutter's left padding, so the numbers do not move right by more than one column. Phase 2 measures this; if `LineSign::before` adds a column, that is accepted.
- **D7.** Every tier shows the strip, compact included.

### Phases

#### Phase 1: Signs from a patch (pure)

- `src/diff/signs.rs`: `fn signs(patch: &Patch) -> Vec<(u32, Sign)>` (or parse a `-U0` hunk header list), with `Sign { Added, Modified, DeletedBelow, DeletedTop }`, following D2.
- Unit tests: pure add, pure delete (middle and top), modify equal counts, modify with more new lines, modify with fewer new lines, several hunks, an untracked file, an empty patch. Sample lines from Harry Potter or Dan Brown.

#### Phase 2: Gutter, setting, refresh

- `settings.rs`: `git_gutter` (bool, default `true`); `docs/configuration.md` documents it. Tests use a temporary `XDG_CONFIG_HOME`.
- Service: a worker job builds the open file's `-U0` patch against the change base (reuse `patch` with context 0; an untracked file yields all-added) and returns the signs for that path and generation.
- Reader: apply signs with `line_sign` on both `LineNumbers` (sticky and wrapped) in the palette's `added`, `modified`, and `removed`; clear them on open, close, and setting off. Restyle on theme change.
- Refresh per D4, through the existing browse poll and reload path.
- Tests: a TestApp test in a temporary repo (`git init`, commit, edit) that opens the file and checks the sign cells; one with `git_gutter: false` shows none; one outside a repo shows no strip.
- Live run in a new tab of the agent's own Rex session: edit a file in a scratch repo under `/tmp` while fmc shows it, and watch the signs appear, change, and clear after `git commit`. Check at 49, 70, and 100 columns. 80x24 captures outside a repo stay byte-identical to HEAD.

#### Phase 3: Docs and close

- `architecture.md` record (the deviation, O1 to O3, D1 to D7); `docs/parity.md` (Reader); `docs/configuration.md`; README features; CHANGELOG (Unreleased); `DESIGN.md` (the sign strip and its colours); `rust.md` (the module).
- Archive the plan, update `index.md`, and hand over the owner checks.

### Risks

- A sign column inside a repository changes every capture taken inside one. The capture tests must run outside a repository, or be updated on purpose.
- Running `git diff` per reload on a large repository could be slow. The job runs on the worker and drops stale answers. Phase 2 times it on fmc's own repo.

### Validation

`just verify` (or `KACHE=false just verify`) per phase, plus the live run in phase 2.

### Order

Runs after plan-00006 phase 2 and plan-00007, unless the owner reorders.

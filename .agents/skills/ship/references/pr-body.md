# PR title and body

Written from the actual diff and commits (`git log origin/<default>..HEAD`, `git diff origin/<default>...HEAD --stat`), never from the plan's intent alone. English, plain words (`clarity` skill). At most about 20 lines before the trailer.

## Title

Imperative, under about 70 characters, conventional prefix when one change dominates: `fix(diff): fit short diffs to the pane`. A stack of several features: `feat: open with other apps, more grammars, footer root and branch`.

## Body

```markdown
## Summary

- <what changed and why, one to three bullets>

## Changes

- **<area>:** <the change that matters to a reviewer>
- **<area>:** <…>

## Testing

- `just verify`: <N> passed, <M> skipped.
- Live run: <what was checked, at which sizes>, or "none: <reason>".

## Heads-up

- <an action the merger must take: merge #18 first; set `editor` in settings to try `e`>

## Notes

- <context: an approved parity deviation and where it is recorded; a known limit>

Closes #<n>

🤖 Generated with [Claude Code](https://claude.com/claude-code)
```

Drop `## Heads-up` and `## Notes` when empty. The last line is the PR trailer from the session's attribution reminder.

## What goes where

- **Testing** says what ran and its result, and a pre-existing failure verified on the default branch: "`cargo deny` fails on `main` too (advisory RUSTSEC-…), unrelated".
- **Heads-up** is actions: merge order for stacked branches, a new dependency the owner must know, an owner check that needs a real terminal or app.
- **Notes** is context: a parity deviation (name its `architecture.md` record), a new rule, a limit left on purpose.

## Example (from PR #19)

```markdown
## Summary

- The diff body was always pane plus gutter wide, so the stat bar fell past the edge and panning hid the gutter even for short lines. The owner approved fitting the pane, a deviation from Scouter v0.3.4.

## Changes

- **Diff:** the stack is the pane, or the widest line's need when wider (`widest_line`, `stack_columns`).
- **Docs:** deviation record in `architecture.md`, line in `docs/parity.md`, `CHANGELOG.md`.

## Testing

- `just verify`: 786 passed, 1 skipped.
- Live run at 220x50 and 80x24, unified, split, and wrap.
```

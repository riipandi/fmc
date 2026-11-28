# Commits

How the `ship` skill splits work into atomic commits without interactive git.

## Grouping

One commit is one reason to change: a feature with its tests and its docs, a fix with its regression test, a refactor that changes no behaviour, a CI change. Each commit builds and passes `just verify` on its own; when two units depend on each other, the one depended on goes first.

Signals that changes belong apart:

- They would get different commit types (`feat` and `ci`).
- One could be reverted without breaking the other.
- The owner made one of them (an edit you did not make and were not asked to ship stays out; ask if unsure).

## Staging whole files

```sh
git add src/services/launch.rs src/services/launch_tests.rs docs/usage.md
git status --short
git commit -F /tmp/fmc-msg.txt
```

## Staging part of a file

`git add -p` is interactive and unavailable. Write the hunks you want as a patch and apply it to the index only:

```sh
git diff -- src/settings.rs > /tmp/fmc-settings.diff
# edit /tmp/fmc-settings.diff: keep the hunks of this unit, delete the others (whole @@ blocks)
git apply --cached --check /tmp/fmc-settings.diff
git apply --cached /tmp/fmc-settings.diff
git diff --cached --stat
```

The working tree keeps every change; only the index takes the chosen hunks. When a hunk mixes two units, split the edit in the file instead (commit one state, then make the second edit).

## Message

```text
feat(open): open the selected file with the default app

<why, only when the subject and the diff do not make it obvious>

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

Use the trailer from the session's attribution reminder; the line above is an example. A port of Scouter behaviour also ends with `Scouter-Base: v0.3.4 (434d000f)` (`rules.md`, "Parity with Scouter").

## Never

`git add -A`, `git add .`, `git commit --no-verify`, `git commit --amend` on a pushed commit, or a commit on the default branch.

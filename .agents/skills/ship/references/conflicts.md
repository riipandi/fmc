# Conflicts

Which conflicts the `ship` skill resolves on its own, and which go to the owner.

## Resolve: both sides added

Two branches each added entries under `## [Unreleased]` in `CHANGELOG.md`:

```text
<<<<<<< HEAD
### Changed

- The changes screen fits a diff to the pane when its lines do.
=======
### Added

- `o` opens the selected row with the macOS default app.

### Changed

- The footer shows the browsed root and the git branch.
>>>>>>> origin/main
```

Keep every entry, one heading per kind, in Keep a Changelog order (Added, Changed, Deprecated, Removed, Fixed, Security):

```text
### Added

- `o` opens the selected row with the macOS default app.

### Changed

- The footer shows the browsed root and the git branch.
- The changes screen fits a diff to the pane when its lines do.
```

The same rule covers rows added to a doc table, lines added to an index, and imports added to one `use` list: keep both, in the file's order.

## Resolve: one side edited a long line the other also edited

A one-line table row in `.agents/docs/shape.md` changed on both sides for different facts. Take the default branch's line, then reapply this branch's edit as a text replacement (find the phrase it changed in `git show <commit> -- <file>`), and check both facts are present.

## Ask: anything about behaviour

Stop and show both sides when the conflict is in code logic, a test's expected value, a decision record's wording of a rule, a settings default, or a key binding. Say what each side does and which you would keep; wait for the owner.

## After resolving

`git add <file>` for each, `git status --short` shows no `UU`, then continue (`git rebase --continue`, or `git commit --no-edit` for a merge). The test run in step 4 counts only after the last resolution.

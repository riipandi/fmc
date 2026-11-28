---
name: ship
description: >-
    Ship the current work end to end on GitHub: branch, atomic commits, sync with the default branch, the full gate,
    push, open or update the pull request, wait for CI, merge, then decide whether the merged work needs a release and,
    only if it does, cut it (version bump, changelog, release PR, tag, the release workflow) and distribute it
    (Homebrew tap). Use when the owner says ship, "commit push PR merge", "merge and release", or /ship. An optional
    argument narrows it: "no-merge" stops after the PR, "no-release" skips the release, "release" forces one,
    "patch|minor|major" overrides the bump.
argument-hint: "[no-merge | no-release | release | patch | minor | major]"
---

# Ship

Take the work in this checkout from wherever it is to merged, and released when a release is due. `AGENTS.md` and `.agents/docs/` win over this skill. Invoking the skill is the owner's request to commit, push, open the PR, and merge; the release half runs only when step 9 says it is due, or when the owner asked for it.

GitHub only (`gh`). The origin is `github.com/riipandi/fmc`; if `git remote get-url origin` is not a GitHub URL, stop and say so.

## House rules that shape every step

- Never `git stash`, `git reset --hard`, `git checkout .`, `git clean`, `git commit --no-verify`, `git push --force`, or `git push --force-with-lease` (the last is denied in `.claude/settings.json` too). Never `git branch -D`.
- No interactive flags: `git add -p` and `git rebase -i` are unavailable. Split hunks with a patch file and `git apply --cached` (`references/commits.md`).
- Stage explicit paths; never `git add -A` or `git add .`. Run `git status --short` before each commit.
- Every commit message ends with the attribution trailer the session's system reminder gives (for example `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`), and every PR body ends with its PR line. Comments in code never cite a plan, roadmap entry, decision ID, or doc file (`AGENTS.md`).
- `git rebase`, `gh pr merge`, `gh release *`, `gh workflow run`, and `gh api` ask for permission by design; expect the prompt, do not route around it.

## 1. Branch

1. Default branch: `git symbolic-ref --short refs/remotes/origin/HEAD` (strip `origin/`); if unset, `git remote set-head origin --auto` first. Never assume `main`.
2. Detached `HEAD`: stop and ask.
3. On the default branch: move nothing. Pick a name from the change (`feat/…`, `fix/…`, `docs/…`, `chore/…`, `refactor/…`, `ci/…`; `release/vX.Y.Z` only for step 10). If it exists locally (`git show-ref --verify refs/heads/<name>`) or on origin (`git ls-remote --heads origin <name>`), append `-2`, `-3`, … until free. `git switch -c <name>` from the current `HEAD`, so commits and uncommitted changes carry over. Say the name.
4. On another branch: use it.
5. In a worktree (`git rev-parse --git-common-dir` differs from `--git-dir`): work there; step 8 removes it.

## 2. Commit

Read `git status --short` and `git diff` (staged and unstaged) in full. Group the changes into logical units; one unit is one commit, each buildable on its own (`references/commits.md`). Messages follow `AGENTS.md`: `{feat,fix,docs,refactor,test,chore,ci}[(scope)]: <concise>`, a body only when the why is not obvious, then the trailer.

Leave out, and list in the final report: secrets (`.env*`, keys, tokens), build output (`target/`, `.cache/`), editor and OS files, `*.bak`, scratch logs, debug prints, and changes unrelated to the unit (an owner's in-progress edit to another file stays uncommitted). A docs change belongs in the commit of the behaviour it documents (`rules.md`, "Where a fact lives").

## 3. Sync with the default branch

`git fetch origin`. Then:

- **Never pushed** (`git ls-remote --heads origin <branch>` is empty): `git rebase origin/<default>`.
- **Already pushed:** `git merge --no-edit origin/<default>`. A rebase would need a force push, which this project forbids.

Conflicts: resolve the mechanical ones (both sides added lines to `CHANGELOG.md` `[Unreleased]`, adjacent rows in a doc table, an import list) by keeping both sides in order, as `references/conflicts.md` shows. A conflict in behaviour (both sides changed the same logic, a decision, a test's expectation) is the owner's: stop, show both sides, ask. If a rebase floods add/add conflicts in files the branch never touched, `git rebase --abort`, `git fetch --quiet --unshallow origin`, and rebase again.

## 4. Test

Run `just verify` once after a successful sync (it runs format, check, clippy, tests, and licences; `.agents/docs/shape.md`, "Commands"). Do not run it again after a later fetch or a rejected push unless conflict resolution or another edit changed files since. A failure is fixed, not shipped. Ship past a failing check only when the same check fails on `origin/<default>` without these changes (check it in a temporary worktree: `git worktree add /tmp/fmc-base-<n> origin/<default>`, run the one check, `git worktree remove`), and say so in the PR body's Testing section.

A change to what is drawn or to input handling also needs its live run (`AGENTS.md`, "Validation"); if none was done in this session, say so under Heads-up rather than claiming it.

## 5. Push

`git push -u origin <branch>`. Never to the default branch directly, never forced. A rejected push means origin moved: fetch, go back to step 3 (merge, since the branch is now pushed).

## 6. Open or update the PR

`gh pr view <branch> --json number,url,state` first. Open: `gh pr edit <n> --title … --body-file …`. None: `gh pr create --base <default> --head <branch> --title … --body-file …`. Write the body to a temporary file under `/tmp`, not inline. Title and body follow `references/pr-body.md`. If `gh` is missing or `gh auth status` fails, stop and give the title, body, and `https://github.com/<owner>/<repo>/compare/<default>...<branch>`.

Stacked work (this branch contains another open PR's commits): ship the stack as one PR from its top branch, or merge the lower PR first; say which under Heads-up.

## 7. Ticket

A GitHub issue tied to the work (in the branch name, a commit message, or this thread) gets `Closes #N` in the body. An agent-ledger item (`.agents/docs/issues/issue-<n>`, a plan, a roadmap entry) does not auto-close: its status flips in the docs commit of the change (`agent-docs` skill), and the final report names it.

## 8. CI, merge, cleanup

1. Wait for the checks: `gh pr checks <n> --watch --interval 30` in the background (not a foreground sleep loop). The `ci` job is the one status (`.github/workflows/ci.yml`); `rust` and `markdown` may be skipped by the path filter, which counts as passing.
2. Red CI: read the failing job's log (`gh run view <run> --log-failed`), fix, commit, push, wait again. Do not merge red.
3. Unless the owner said `no-merge`: `gh pr merge <n> --merge --delete-branch` (the repository merges with merge commits). For a stack shipped as separate PRs, merge bottom-up and re-sync each next branch (step 3) before its merge.
4. Cleanup, only after the merge:
   - In a worktree: leave it (`cd` to the main checkout) and `git worktree remove <path>`; never `--force`; if it refuses, say why.
   - `git switch <default> && git pull --ff-only`. Delete the local branch only with `git branch -d` (it refuses unmerged work). `git fetch --prune`.
   - If the local default branch had commits ahead of origin that are now in the PR, ask before `git reset --keep origin/<default>`; never reset on your own.

## 9. Decide whether to release

Run this after the merge, on the updated default branch. Unless the owner said `release` or `no-release`, a release is due only when all hold:

- Commits since the last tag (`git describe --tags --abbrev=0`, then `git log <tag>..HEAD --no-merges`) change what users get: any path among `src/`, `Cargo.toml`, `Cargo.lock`, `build.rs`, `vendor/`, `rust-toolchain.toml`, `.cargo/`, `scripts/opentui-native.sh`.
- `CHANGELOG.md` `## [Unreleased]` has at least one entry.

If only docs, CI, agent files (`.agents/`, `.claude/`, `AGENTS.md`), or tests changed, there is no release: say "no release: <reason>" and stop. If code changed but `[Unreleased]` is empty, stop and ask: the changelog is missing an entry, or the change is internal and needs none.

The bump (`references/release.md`, "Version"): before 1.0, any `### Added`, `### Changed`, or `### Removed` entry is a minor bump; only `### Fixed` is a patch. The owner's `patch|minor|major` argument wins.

## 10. Release and distribute

Follow `references/release.md` step by step: the `release/vX.Y.Z` branch with `Cargo.toml`, `Cargo.lock`, and `CHANGELOG.md`; its PR through steps 4 to 8; the tag on the merged default branch; the `release` workflow; then every distribution channel `docs/install.md` names (today the Homebrew tap and the release archive). Verify each channel installs or reports the new version before calling the release done.

## Report

End with: the branch (and whether it was created), the commits (hash and subject), the PR URL and its merge state, the linked issue or ledger item, the release (version and URL, or "no release" with the reason), each distribution channel's state, what was cleaned up, and what was left out of the commits. If something waits on the owner, close with the `Owner check` section `AGENTS.md` describes.

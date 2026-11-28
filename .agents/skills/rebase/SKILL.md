---
name: rebase
description: >-
    Fetch origin and rebase the current branch onto the latest default branch (origin/HEAD), keeping every uncommitted
    change (staged, unstaged, and untracked) and restoring it afterwards, without git stash. Handles a shallow clone that
    floods add/add conflicts by aborting, unshallowing, and rebasing again. Use when the owner says rebase, "sync with
    main", "update the branch", or /rebase. Stops before rewriting a branch that is already pushed.
---

# Rebase

Bring the current branch up to date with the default branch by rebasing, and leave the working tree exactly as it was. `AGENTS.md` and `.agents/docs/` win over this skill.

## House rules

- **No stash.** `git stash` and `git rebase --autostash` are not used (`AGENTS.md`, "Git"; `git stash*` is denied in `.claude/settings.json`). Uncommitted work is set aside as a temporary commit instead (step 3).
- **No force.** `git push --force` and `--force-with-lease` are denied, so a branch that is already on origin is not rebased without the owner's word (step 2).
- **No destructive resets.** Never `git reset --hard`, `git checkout .`, or `git clean`. The only resets here are `--soft` and the plain (mixed) one, which keep every file as it is.
- `git rebase` asks for permission by design; expect the prompt.

## 1. Where things stand

1. **The default branch.** Run `git symbolic-ref --short refs/remotes/origin/HEAD` (`origin/main` here). If it is unset, run `git remote set-head origin --auto` first.
2. **Stop on these states:**
   - a detached `HEAD`;
   - a rebase, merge, or cherry-pick already in progress (`.git/rebase-merge`, `.git/rebase-apply`, `.git/MERGE_HEAD`, or `.git/CHERRY_PICK_HEAD` in `git rev-parse --git-dir`);
   - the default branch itself. Rebasing `main` onto `origin/main` is a pull: use `git pull --ff-only`, and if it does not fast-forward, stop and ask.
3. **Fetch.** Run `git fetch origin`. If `git merge-base --is-ancestor origin/<default> HEAD` holds, the branch is already current: say so and stop.

## 2. Already pushed?

Run `git ls-remote --heads origin <branch>`. If the branch is on origin and its commits are not all in the default branch, a rebase rewrites published commits. Its next push would need a force push, which this project denies. Stop and ask:

```text
Unclear: <branch> is already on origin; rebasing rewrites its pushed commits, and pushing them needs a force push, which this project denies.

1. Merge origin/<default> into <branch> instead (no rewrite, a normal push) [recommended]
2. Rebase onto a new branch name (<branch>-2), push that, and close the old PR
3. Other: reply with what you want
```

If the owner already said to rebase this branch in the thread and knows about the push, go on: the rebased branch is pushed later under a new name, or the owner pushes it.

## 3. Set uncommitted work aside

Skip this step when `git status --porcelain` is empty.

1. Save the staged state so partial staging survives: `git diff --cached --binary > /tmp/fmc-rebase-staged-<branch>.patch`. Note whether it is empty.
2. List every changed and untracked path with `git status --porcelain --untracked-files=all`, minus ignored files. Stage those paths by name (`git add -- <path> …`; never `git add -A` or `.`). A deleted file is staged with `git add -- <path>` too.
3. Commit them as one temporary commit: `git commit -m "WIP: rebase autosave"` (no trailer; this commit never leaves the machine). Record its hash.

Leave secrets alone: a `.env` that is untracked stays untracked and is not part of the commit if it is ignored. If an untracked, non-ignored file looks like a secret, stop and ask before committing it even temporarily.

## 4. Rebase

Run `git rebase origin/<default>`.

- **Conflicts.** Resolve them as `.agents/skills/ship/references/conflicts.md` says: keep both sides of a mechanical conflict (changelog entries, table rows, lists); stop and ask for a conflict in behaviour. After each resolution: `git add <file>`, `git status --short` shows no `UU`, then `GIT_EDITOR=true git rebase --continue`.
- **A conflict inside the WIP commit** is a conflict in the owner's uncommitted work. Resolve it only if it is mechanical; otherwise stop and show it.
- **An add/add flood.** Many `AA` conflicts in files this branch never changed (check against `git diff --name-only $(git merge-base HEAD origin/<default>)..HEAD`) usually mean a shallow clone with no common history. If `git rev-parse --is-shallow-repository` prints `true`:

  ```sh
  git rebase --abort
  git fetch --quiet --unshallow origin HEAD
  git rebase origin/<default>
  ```

  If the clone is not shallow, abort and stop: the flood has another cause, and the owner decides.
- **Abort means restore.** On any `git rebase --abort`, the branch returns to its pre-rebase state, WIP commit included; go to step 5 to restore the work.

## 5. Restore uncommitted work

Only when step 3 made a WIP commit, and only if `HEAD`'s subject is `WIP: rebase autosave` (check with `git log -1 --format=%s`; never reset a commit that is not the WIP one):

1. `git reset --soft HEAD~1` puts the WIP changes back in the index.
2. `git reset` (mixed, no paths) unstages them all, so the working tree holds the changes and untracked files are untracked again.
3. If the staged patch from step 3 was not empty, run `git apply --cached /tmp/fmc-rebase-staged-<branch>.patch`. If it no longer applies (the rebase changed those lines), leave everything unstaged and say which files were staged before.
4. Compare `git status --short` with the state from step 3. The same paths should show the same states.

## 6. Check

- `git log --oneline origin/<default>..HEAD` shows the branch's own commits on top. No WIP commit is left.
- Run `just verify` if any conflict was resolved, or if code changed on the default branch since the branch's old base. A docs-only update needs no run.
- Do not push. Pushing is the owner's call, or the `ship` skill's.

## Report

Report these:

- the default branch and the new base commit;
- the commits replayed;
- each conflict and how it was resolved;
- whether the clone was unshallowed;
- the uncommitted work restored, and anything that could not be re-staged;
- the gate result.

If the branch is on origin, say that pushing it needs the owner's choice from step 2.

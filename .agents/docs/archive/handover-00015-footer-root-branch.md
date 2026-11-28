---
status: done
created: 2026-11-28T16:05:30+07:00
plan: plan-00019-footer-root-branch
---

# Handover: the footer shows the browsed root and the git branch; plan-00019 closed

## Closed

- plan-00019 on `feat/footer-root-branch`, stacked on `feat/remaining-grammars`. Commits: `55b31f8` (the branch read with the browse poll), `4383d53` (the footer's root and branch, with the deviation record, the parity line, the changelog, and the design row), `02ff5b7` (the branch clears when the root changes), and the docs commit (`docs: the footer shows the root and the branch`: this handover, the plan's archive move, the amendments, the index).
- The owner's decisions of 2026-11-28: the footer shows the root and the branch from 60 columns up; below 60 the compact bottom bar keeps its words (`architecture.md` D2 and "Decided"). Going up a folder clears the branch (`architecture.md` D7).
- `architecture.md` records the deviation (D1 to D7), `docs/parity.md` lists it under Layout, `CHANGELOG.md` has a `Changed` entry, `DESIGN.md` and `docs/usage.md` describe the footer.

## Accepted deviations from the plan (owner, 2026-11-28)

- The branch is read with `git symbolic-ref -q HEAD` (the name, without `refs/heads/`), not `--short`: with a tag of the same name `--short` prints `heads/main` (checked in a temporary repository).
- An unborn repository shows its branch name, following D3's wording. The plan's test matrix said "no commit: nothing"; that row was not met.
- The first browse poll is the first timer tick, about two seconds after start. The code does not poll at start.
- `OpenAt.shown` is removed as well as D6's list: its only reader was the footer's path.
- The branch takes the name's colour (`p.text`).

## Live run

Rex, in new windows of this session. Navigation keys only: `j`, `Enter`, `d`, `Ctrl+,`, `Esc`, and `-`. Each window was closed after its run; the owner's panes were not touched.

- 120 columns, fmc on this repository (`target/debug/fmc`): the Files, reader (`AGENTS.md`), changes (`d`), and Settings (`Ctrl+,`) screens each show `~/Developer/github.com/riipandi/fmc  ⎇ feat/footer-root-branch` at the left of the footer. The changes footer keeps its facts on the right (`1 file · All`), and the reader keeps `70 lines · markdown · UTF-8 · Top`.
- Branch switch: a temporary repository `/tmp/fmc-branch.*` on `hogwarts`, opened in a window. `git switch -c marauders/map` run from a shell (not in fmc) showed `⎇ marauders/map` on the next poll, within about four seconds. Then `-` (up one folder, to `/tmp`, which is not a repository) showed no branch.
- 50 columns: not repeated in phase 4. The window is owned by the owner's client at 187 by 60; a `resize` request from this session does not apply to a block the session does not own (`rex block call <block> size` shows the owner). In the phase 2 run, at 50 columns, the reader's compact bars showed `‹ AGENTS.md` in the top bar and `Back · Search · Keys` at the bottom, with no root or branch. The Files page's compact bar was not captured.
- A test in the phase 2 run typed `A G E N T S` into the tree, which includes `A` (create a folder) and `E` (reveal in Finder). No file or folder was created in the repository, and the Trash was empty. A Finder window may have opened; see the owner checks.

## Tests

- `just verify` on the fix commit `02ff5b7`: 828 run, 1 skipped; licences ok. `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` clean. `rumdl fmt --check --quiet .` runs inside `just verify`.
- New tests: `services::git` (`the_branch_is_its_name_with_any_slash_and_a_detached_head_is_its_hash`, `a_repository_without_a_commit_names_its_branch_and_a_folder_outside_one_has_none`), `ui::footer` (the root and branch on every screen, a message replaces them, the root cut before the branch at 40, 60, 80, and 120 columns, the branch cut from the right at 30, a detached hash), and `app::tree` (a poll names the branch; opening a file keeps the browsed root; going up clears the branch, and a poll of the old root does not set it).
- The footer's compact bar and its tests are unchanged.

## Owner checks

1. Check the Finder: a Finder window may have opened from the `E` in the phase 2 test. Close it if it is there.
2. Clean up the scratch files: `! rm -rf /tmp/fmc-dead-check /tmp/fmc-clippy.log /tmp/fmc-verify.log /tmp/fmc-branch.*`.
3. Confirm the 50-column Files page and the compact footer on a real phone or a resized window, since the live run could not resize the owner's window.

## Open threads

- None for the footer. The compact footer stays as record 7 has it, by the owner's decision.

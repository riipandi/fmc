---
status: done
created: 2026-11-28T15:21:18+07:00
updated: 2026-11-28T16:05:30+07:00
phases:
    - id: 1
      title: The branch, read with the browse poll
      status: done
      note: —
    - id: 2
      title: The footer shows the root and the branch on every screen
      status: done
      note: "the screens at 60 columns and up; compact unchanged (owner, 2026-11-28)"
    - id: 3
      title: Docs sync, agent docs sync, ledger close, handover
      status: done
      note: "the docs commit and the handover (handover-00015-footer-root-branch.md)"
    - id: 4
      title: Final validation and live run
      status: done
      note: "no commit; the live run is in handover-00015-footer-root-branch.md
---

## The footer shows the browsed root and the git branch — not the open file's path

### Objective

The owner found the footer's location redundant on 2026-11-28: it repeats the open file's path, which the reader header already names. After this plan, the footer's left side shows the browsed root (tildified) and the current git branch on every screen, and a message still replaces it while shown. This is an approved parity deviation from Scouter, whose footer shows the file's or folder's path.

### Current state (verified in the tree, 2026-11-28)

- **The footer's location** — `FooterState.location` (`src/ui/footer.rs:33`, drawn at `:162-166` with `fit_end`, which cuts from the left with `…`); in `Mode::Diff` the location is `DiffInfo.location` (`:163`). `Model.location` (`src/app/mod.rs:114-115`) starts as the startup path (`:268`) and is set to the revealed file (`src/app/reader.rs:111`, `location_of` `:122-127`) and to an opened folder (`src/app/tree.rs:157-161`). The footer tildifies it (`src/app/mod.rs:795`).
- **The changes screen's location** — the selected change's path, or the repository folder's name (`src/app/changes.rs:564-567`).
- **Scouter** — the footer's left side is the pill, then the location or a message; the location is `~`-tildified and cut from the left (`research/scouter-inventory.md:106-113`, Scouter `src/ui/footer.ts:198-228`).
- **Git** — fmc reads no branch today (`grep -n "symbolic-ref\|branch" src/services/git.rs` finds none). The browse poll (`src/services/poll.rs:12-30`) reads the marks with `git::marks(&root)` every 3 s and sends `Event::BrowsePolled`; git calls run through `capture` with the 15 s timeout (`src/services/git.rs:447-488`).

### Decisions

- **D1 (owner, 2026-11-28): the footer's location is the browsed root.** The root fmc browses (the startup folder, or the root after a move to a parent), tildified, never the selected file or folder. The reader header names the file. Rejected: the selection's folder, and the root with the relative folder (the owner's pick).
- **D2 (owner, 2026-11-28): every screen, at 60 columns and up.** Files, reader, Settings, and the changes screen show the root and the branch. The changes screen's selected path leaves the footer; its diff header names the file. Rejected: browse only. Amended by the owner on 2026-11-28: below 60 columns the compact bottom bar keeps its words and shows neither the root nor the branch (the second row and the compact top bar were the other options, not taken).
- **D3 (owner, 2026-11-28): the branch is `⎇ <name>`.** A detached `HEAD` shows `⎇ <short hash>`; outside a repository, or in a repository with no commit and no branch name, nothing is shown. Rejected: plain text with no glyph; a Nerd Font icon.
- **D4 (2026-11-28): the branch is read with the browse poll.** `git::branch(dir)` runs `git symbolic-ref --short -q HEAD`, and on no name, `git rev-parse --short HEAD`, through `capture`, and `Event::BrowsePolled` carries it; the first poll runs at start, so the branch shows within the first tick. A checkout in another terminal shows within 3 s. Rejected: a separate timer — the poll already runs git on the same folder.
- **D5 (2026-11-28): fitting.** The left side is `<root>  ⎇ <branch>`, the branch in the location's colour. When it does not fit, the root is cut from the left first (`fit_end`, as now), down to `…` plus its last segment; then the branch is cut from the right. A message replaces both, as now.
- **D6 (2026-11-28): `Model.location` goes.** No screen needs the selection's path in the footer any more; `location_of`, the two assignments, and `DiffInfo.location` are removed (no compatibility, `AGENTS.md`). The root comes from the browser's root.
- **D7 (owner, 2026-11-28): the deviation is recorded** in `architecture.md` and `docs/parity.md`, naming the baseline (v0.3.4, `434d000f`), in the docs phase.

### Execution rules

- **The owner gave the go-ahead for all phases on 2026-11-28; the plan runs after plan-00017 and plan-00018 finish**, on `feat/footer-root-branch` branched from the last of their branches.
- **Every commit ends with** `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. No `Scouter-Base` trailer: this departs from Scouter.
- **Tests run git in temporary repositories** (the pattern of `services/git_tests.rs` and `app/gutter_tests.rs`), never in this repository's state.

### Phases

1. **The branch** — `git::branch` (D4) and its field on `Event::BrowsePolled`; the model keeps the last value. Tests in temporary repositories: a branch name (`hogwarts`), a detached `HEAD` (short hash), no commit (nothing), outside a repository (nothing), a name with `/` (`feat/marauders-map`). Validation: `cargo test git`, `cargo test poll`. _Commit: `feat(git): read the current branch with the browse poll`._
2. **The footer** — `FooterState` takes the root and the branch; `Model.location`, `location_of`, and `DiffInfo.location` are removed (D6); the fitting of D5. Tests in `src/ui/footer_tests.rs` and the app tests: every mode shows the root and the branch; a message replaces them; at 40, 60, 80, and 120 columns the root is cut first, then the branch; opening a file or a folder leaves the footer's location unchanged. Validation: `cargo test`; clippy. _Commit: `feat(footer): show the browsed root and the git branch`._
3. **Docs sync, agent docs sync, ledger close, handover** — `docs/usage.md` (the footer), `docs/parity.md`, `CHANGELOG.md`, `DESIGN.md` (the footer's left side), `architecture.md` (D7), `shape.md`, `research/scouter-inventory.md` untouched (it records Scouter); plan archived with its index row; a handover. Validation: `rumdl fmt --check --quiet .`. _Commit: `docs: the footer shows the root and the branch`._
4. **Final validation and live run** — the matrix; a live run in a new Rex tab of this session at 120 and 50 columns on this repository: files, reader, changes (`d`), and Settings (`Ctrl+,`) each show `~/…/fmc  ⎇ <branch>`; `git switch` in another tab shows the new branch within 3 s. _Commit: none when green._

### Test matrix

| What | How | Green |
| --- | --- | --- |
| Format | `cargo fmt --check`; `rumdl fmt --check --quiet .` | No drift |
| Unit | `cargo test git`, `cargo test footer` | The branch cases; the fitting at four widths |
| App | `cargo test` | Every mode shows root and branch; a message replaces them |
| Lint | `cargo clippy --all-targets -- -D warnings` | 0 warnings |
| Full suite | `just verify` | All green |
| Live run | Rex tab, 120 and 50 columns, four screens, a branch switch | As D1 to D5 |
| E2E | n/a beyond the live run: no CLI change | — |

### Out of scope

- The header's content (it already names the file).
- Other git facts in the footer (ahead/behind, dirty count).

## Owner decisions and deviations taken during the run (2026-11-28)

- **Compact (below 60 columns):** option 1 of the owner's question: the one-row bottom bar stays as `architecture.md` record 7 has it. D2 is amended above and recorded in `architecture.md` (D2 and "Decided").
- **Root change (D7 in `architecture.md`):** going up a folder clears the branch until the next poll names the new root's branch. Commit `02ff5b7`, `fix(footer): clear the branch when the root changes`.
- **Accepted deviations** (owner, 2026-11-28; each is recorded in `architecture.md` D3 to D6 or in the commit):
  - the branch is read with `git symbolic-ref -q HEAD` (name, with `refs/heads/` removed), without `--short`, which prints `heads/main` when a tag has the same name;
  - an unborn repository shows its branch name (D3's wording; the test matrix's "no commit: nothing" was not met);
  - the first browse poll is the first timer tick, about two seconds after start, not at start;
  - `OpenAt.shown` is removed as well as D6's list, because its only reader was the footer's path;
  - the branch takes the name's colour (`p.text`), which is how this plan read "the location's colour".
- **Commits:** `55b31f8` (phase 1), `4383d53` (phase 2), `02ff5b7` (the root-change fix), and the phase 3 docs commit (`docs: the footer shows the root and the branch`).

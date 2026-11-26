# Agents Documentation Index

updated: 2026-11-27T04:02:00+07:00

Agent-only documentation. Start at the repo root `AGENTS.md` (workflow rules), then the core documents below. Everything else is reference, history, or contract; navigate from here.

## Core documents

| Path | Read it for |
| --- | --- |
| `/AGENTS.md` | Always-on rules: parity and read-only rules, ambiguity protocol, workflow, git, the skill map. |
| `.agents/docs/index.md` | This map. |
| `.agents/docs/shape.md` | Layout, toolchain, commands and gates. |
| `.agents/docs/rules.md` | Cross-area rules: parity with Scouter, where a fact lives, dependencies, sample data, secrets. |
| `.agents/docs/rust.md` | Rust code contracts and traps for the crate. |
| `.agents/docs/architecture.md` | Decision records, approved parity deviations, and the pending decisions. |
| `/PRODUCT.md`, `/DESIGN.md`, `/README.md` | Human docs: product purpose and principles; palettes, layout, and components (Scouter wins during parity); overview and usage. |
| `.agents/docs/research/` | Point-in-time research of 2026-11-26: `scouter-inventory.md` (the parity reference, cited by every port phase), `rendering-research.md`, `opentui-binding-risk.md`, `dependency-survey.md`, `editor-study.md`, `sqltui-study.md`. The tree and the locked crates win. |

## Terms

- **Agent docs**: `.agents/docs/` (this ledger and the core documents), `AGENTS.md`, and the project skills in `.agents/skills/`. Written for agents, English, with `file:line` evidence.
- **Human docs**: `README.md`, `PRODUCT.md`, `DESIGN.md`, and `docs/` (created by `plan-00001` phase 2).
- **Scouter**: the parity reference, <https://github.com/indrazm/scouter>. Its behavior is verified from its source or a live run (`rules.md`).

## Docs sync

A behavior change and its documentation land in the same commit; `rules.md` ("Where a fact lives") names the file, and this map gets the matching row.

## Project skills (`.agents/skills/`)

| Skill | Use it for |
| --- | --- |
| `agent-docs` | Every ledger or docs change: plans, issues, roadmap, handovers, index, archive. |
| `handoff` | Writing a handover on the owner's request. |
| `deep-audit` | Evidence-based audit of a feature or the whole system; report in `audit/`. |
| `parity-audit` | Screen-by-screen comparison of fmc with Scouter, using Scouter's source and a Rex live run. |
| `rust-verify-harden` | Rust quality gates and the leak, deadlock, race, and terminal-state pass. |

Global skills used here too: `security-review`, `clarity`, `run`, `rex`.

## Ledger

Each file keeps its name for life; an archived file moves unchanged to `archive/`.

| Path | Holds | Now |
| --- | --- | --- |
| `plans/` | Plan contracts, run phase by phase on the owner's go-ahead. | none open; `plan-00005-layout-tiers.md` is archived (done: the three layout tiers) |
| `issues/` | Findings that outlive a turn. | none open |
| `roadmap/` | Parked improvements with a named unlock. | `roadmap-00001-post-parity.md` (living): Mermaid, Neovim grammars, fuzzy finding, write operations, SQLite and DuckDB viewer, platforms, Vim keymap, event watching. |
| `handover/` | Session compactions. | `handover-00006-layout-tiers.md` (done: plan-00005 closed, with the owner checks on a phone); `handover-00005-phone-layout.md` (done: plan-00004 closed, with the owner checks); `handover-00001-parity-w0-w3.md` (partial, older: its open threads are closed or listed in 00002) |
| `audit/` | Point-in-time audit reports. | `audit-00002-mobile.md` (completed: the phone audit at 40 to 90 columns; verdicts for P1 to P8 landed in plan-00004; the owner's real-touch checks are in handover-00005); `audit-00001-scouter-parity.md` (completed: the plan-00001 phase 22 parity audit; F-01 to F-06 fixed in phase 23, F-07 and F-08 approved deviations in 6cb4373) |
| `archive/` | Completed ledger files. | `plan-00005-layout-tiers.md` (done: the three layout tiers, phases 1 to 6 in 166051c to the docs commit, closed by handover-00006); `plan-00004-phone-layout.md` (done: audit-00002 P1 to P8 landed, f46692d to 5ec24e4, closed by handover-00005); `handover-00004-browse-ux.md` (done: plan-00003 closed); `handover-00003-sidebar-and-hidden-files.md` (done: plan-00002 closed); `issue-00012-finder-gitignore-double-dot.md` (resolved: not reproduced by the owner); `plan-00003-browse-ux.md` (done: phases 1 to 7 landed in `cd1e413` to `8b8a88c`, closed by handover-00004); `plan-00002-sidebar-and-hidden-files.md` (done: RI-11 and RI-12 landed in `55f9b45` and `81a735f`); `handover-00002-scouter-parity-closed.md` (done: superseded by handover-00003); `issue-00001-w3-reader-parity-gaps.md` (resolved: the Core highlight queries and the reader focus and startup deviations); `issue-00003-rendered-markdown-scroll-end.md` (resolved by plan-00001 phase 23c); `issue-00004-welcome-clipped-at-80x24.md` (resolved by plan-00001 phase 23a); `issue-00005-markdown-table-widths.md` (resolved by plan-00001 phase 23b); `issue-00007-tree-wheel-scrolls-back.md` (resolved by plan-00001 phase 23d); `issue-00008-reader-pan-one-column-short.md` (resolved by plan-00001 phase 23e); `issue-00002-binary-footer-position.md` (resolved: owner decision, approved deviation in 6cb4373); `issue-00009-folded-header-stat-bar.md` (resolved by plan-00001 phase 23f); `issue-00010-upgrade-not-recorded.md` (resolved: owner decision, approved deviation in 6cb4373); `issue-00011-owner-hand-evaluation.md` (resolved: the owner's 2026-11-26 evaluation, four items fixed in 0b7eeb8, 91c39fd, 3aca636, 0796828, and 0f0400a); `issue-00006-scrollbar-not-smooth.md` (resolved: owner hand check, wheel and trackpad smooth; thumb drag fixed in 0b7eeb8); `plan-00001-scouter-parity.md` (done: W0 to W6, closed by plan-00001 phase 24) |

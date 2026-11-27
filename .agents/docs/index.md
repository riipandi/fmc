# Agents Documentation Index

updated: 2026-11-27T20:05:31+07:00

Agent-only documentation. Start at the repo root `AGENTS.md` (workflow rules), then the core documents below. Everything else is reference, history, or contract; navigate from here.

## Core documents

| Path | Read it for |
| --- | --- |
| `/AGENTS.md` | Always-on rules: parity and read-only rules, ambiguity protocol, workflow, git, the skill map. |
| `.agents/docs/index.md` | This map. |
| `.agents/docs/shape.md` | Layout, toolchain, commands and gates. |
| `.agents/docs/rules.md` | Cross-area rules: parity with Scouter, the bounded writes, where a fact lives, dependencies, sample data, secrets. |
| `.agents/docs/rust.md` | Rust code contracts and traps for the crate. |
| `.agents/docs/architecture.md` | Decision records, approved parity deviations, and the pending decisions. |
| `/PRODUCT.md`, `/DESIGN.md`, `/README.md` | Human docs: product purpose and principles; palettes, layout, and components (Scouter wins during parity); overview and usage. |
| `.agents/docs/research/` | Point-in-time research of 2026-11-26 and 2026-11-27: `scouter-inventory.md` (the parity reference, cited by every port phase), `rendering-research.md`, `opentui-binding-risk.md`, `dependency-survey.md`, `editor-study.md`, `sqltui-study.md`, `mermaid-spike.md` (2026-11-27, plan-00006 phase 1: the Mermaid engine comparison, its outputs and recommendation), `markdown-preview-mapping.md` (2026-11-27, plan-00010 phase 1: tree-sitter-md's blocks against OpenTUI's, the two rules that align them, the inline-mark and refresh measurements), `grammar-load-spike.md` (2026-11-27, plan-00006 phase 3: the twelve candidate grammar crates, each loaded and compiled on tree-sitter 0.27.2, with the captures to rename). The tree and the locked crates win. |

## Terms

- **Agent docs**: `.agents/docs/` (this ledger and the core documents), `AGENTS.md`, and the project skills in `.agents/skills/`. Written for agents, English, with `file:line` evidence.
- **Human docs**: `README.md`, `PRODUCT.md`, `DESIGN.md`, and `docs/` (the user pages, starting at `docs/index.md`).
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
| `plans/` | Plan contracts, run phase by phase on the owner's go-ahead. | `plan-00015-settings.md` (approved: the settings write path, theme mode and the theme catalog, the Settings page on `Ctrl+,`, and the keymap file; RI-9, RI-10, RI-13; sixteen owner decisions open); none in progress. `plan-00014-multi-select.md`, `plan-00013-injected-highlighting.md`, and `plan-00012-write-operations.md` are archived (done). |
| `issues/` | Findings that outlive a turn. | none open |
| `roadmap/` | Parked improvements with a named unlock. | `roadmap-00001-post-parity.md` (living): a status table at the top tracks every entry; RI-4, RI-18, and RI-19 landed (plan-00012, plan-00013, plan-00014); RI-17 parked (unlock: a crate that loads). |
| `handover/` | Session compactions. | `handover-00010-multi-select.md` (done: RI-19 landed, plan-00014 closed, with the owner checks); `handover-00009-injected-highlighting.md` (done: RI-18 landed, plan-00013 closed); `handover-00001-parity-w0-w3.md` (partial, older: its open threads are closed or listed in 00002) |
| `audit/` | Point-in-time audit reports. | `audit-00002-mobile.md` (completed: the phone audit at 40 to 90 columns; verdicts for P1 to P8 landed in plan-00004; the owner's real-touch checks are in handover-00005); `audit-00001-scouter-parity.md` (completed: the plan-00001 phase 22 parity audit; F-01 to F-06 fixed in phase 23, F-07 and F-08 approved deviations in 6cb4373) |
| `archive/` | Completed ledger files. | `plan-00014-multi-select.md` (done: the marks, the bulk copy, cut, paste, and Trash; phases 1 to 4 in `19d119d`, `b669168`, `4c7eac9`, and `df88716`, and the docs commit); `issue-00014-release-test-build.md` (resolved: the release-profile test build compiles and runs, 2026-11-27); `plan-00013-injected-highlighting.md` (done: RI-18, the injections for the linked grammars; phases 1 and 2 in `1c3857e` and `d619868`, the fix in `d68a7e7`, and the docs close); `handover-00008-write-operations.md` (done: superseded by handover-00009); `plan-00012-write-operations.md` (done: the file-manager phase, create, rename, copy, move, and delete to the Trash, with a context menu and dialogs; phases 1 to 4 in `047a58b`, `17aefe0`, `46f13a0`, and `738501b`, the fix in `872fd21`, and the docs commit); `handover-00007-mermaid-grammars-fuzzy.md` (done: plan-00006 closed); `handover-00006-layout-tiers.md` (done: plan-00005 closed); `handover-00005-phone-layout.md` (done: plan-00004 closed); `plan-00006-mermaid-grammars-fuzzy.md` (done: Mermaid in rendered Markdown, the owner's twelve grammars, and fuzzy finding; phases 1 to 5 in 821340d, 1856099, 9de164e, 363cbf3, and 0637e8d); `plan-00009-file-associations.md` (done: `file_associations` in the settings file, as VS Code's `files.associations`: glob patterns, the filetype lookup for the reader, the finder's preview, the changes screen, and `m`; phases 1 and 2 in 168e256 and edec283, closed by the phase 3 commit); `plan-00010-markdown-preview-search-signs.md` (done: search in the rendered Markdown view, `/` `n` `N` `p` with marks and the footer's count, and plan-00008's git signs beside the rendered blocks; phases 1 to 3 in a7050ec, dafc20b, and d8a96ae; the owner checks are in the run report); `plan-00011-heading-markers.md` (done: rendered headings keep their `#` marker, `reader/headings.rs`, a source rewrite; closed by its phase commit); `plan-00008-git-gutter.md` (done: git change signs in the reader's strip, phase 1 in 37207a1, phase 2 in 1049197, closed by the phase 3 commit; the owner checks are in the run report); `issue-00013-thumb-short-after-shrink.md` (resolved: thumbs stopped short after the terminal shrank; leftover cells now hidden); `plan-00007-compact-border-header.md` (done: the compact tier has the medium border and two-row header, phase 1 in 79c3826, closed with the docs commit); `plan-00005-layout-tiers.md` (done: the three layout tiers, phases 1 to 6 in 166051c to the docs commit, closed by handover-00006); `plan-00004-phone-layout.md` (done: audit-00002 P1 to P8 landed, f46692d to 5ec24e4, closed by handover-00005); `handover-00004-browse-ux.md` (done: plan-00003 closed); `handover-00003-sidebar-and-hidden-files.md` (done: plan-00002 closed); `issue-00012-finder-gitignore-double-dot.md` (resolved: not reproduced by the owner); `plan-00003-browse-ux.md` (done: phases 1 to 7 landed in `cd1e413` to `8b8a88c`, closed by handover-00004); `plan-00002-sidebar-and-hidden-files.md` (done: RI-11 and RI-12 landed in `55f9b45` and `81a735f`); `handover-00002-scouter-parity-closed.md` (done: superseded by handover-00003); `issue-00001-w3-reader-parity-gaps.md` (resolved: the Core highlight queries and the reader focus and startup deviations); `issue-00003-rendered-markdown-scroll-end.md` (resolved by plan-00001 phase 23c); `issue-00004-welcome-clipped-at-80x24.md` (resolved by plan-00001 phase 23a); `issue-00005-markdown-table-widths.md` (resolved by plan-00001 phase 23b); `issue-00007-tree-wheel-scrolls-back.md` (resolved by plan-00001 phase 23d); `issue-00008-reader-pan-one-column-short.md` (resolved by plan-00001 phase 23e); `issue-00002-binary-footer-position.md` (resolved: owner decision, approved deviation in 6cb4373); `issue-00009-folded-header-stat-bar.md` (resolved by plan-00001 phase 23f); `issue-00010-upgrade-not-recorded.md` (resolved: owner decision, approved deviation in 6cb4373); `issue-00011-owner-hand-evaluation.md` (resolved: the owner's 2026-11-26 evaluation, four items fixed in 0b7eeb8, 91c39fd, 3aca636, 0796828, and 0f0400a); `issue-00006-scrollbar-not-smooth.md` (resolved: owner hand check, wheel and trackpad smooth; thumb drag fixed in 0b7eeb8); `plan-00001-scouter-parity.md` (done: W0 to W6, closed by plan-00001 phase 24) |

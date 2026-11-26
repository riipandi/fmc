# Agents Documentation Index

updated: 2026-11-26T07:29:56+07:00

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
| `plans/` | Plan contracts, run phase by phase on the owner's go-ahead. | `plan-00001-scouter-parity.md` (in-progress): Scouter parity in 25 phases over 7 waves; phases 1 to 15 done (W3 reader). |
| `issues/` | Findings that outlive a turn. | `issue-00002-binary-footer-position.md`, `issue-00003-rendered-markdown-scroll-end.md` (open; both for plan-00001 phase 23), `issue-00004-welcome-clipped-at-80x24.md` (open; phase 23), `issue-00005-markdown-table-widths.md` (open; phase 22 records it, phase 23 fixes it) |
| `roadmap/` | Parked improvements with a named unlock. | `roadmap-00001-post-parity.md` (living): Mermaid, Neovim grammars, fuzzy finding, write operations, SQLite and DuckDB viewer, platforms, Vim keymap, event watching. |
| `handover/` | Session compactions. | `handover-00001-parity-w0-w3.md` (partial: W0–W3 landed, W4 next) |
| `audit/` | Point-in-time audit reports. | empty |
| `archive/` | Completed ledger files. | `issue-00001-w3-reader-parity-gaps.md` (resolved: the Core highlight queries and the reader focus and startup deviations) |

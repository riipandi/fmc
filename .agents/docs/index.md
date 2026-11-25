# Agents Documentation Index

updated: 2026-11-26T03:19:42+07:00

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

## Terms

- **Agent docs**: `.agents/docs/` (this ledger and the core documents), `AGENTS.md`, and the project skills in `.agents/skills/`. Written for agents, English, with `file:line` evidence.
- **Human docs**: `docs/`. Not created yet.
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

The ledger directories exist and are empty. Each file keeps its name for life; an archived file moves unchanged to `archive/`.

| Path | Holds | Now |
| --- | --- | --- |
| `plans/` | Plan contracts, run phase by phase on the owner's go-ahead. | empty |
| `issues/` | Findings that outlive a turn. | empty |
| `roadmap/` | Parked improvements with a named unlock. | empty |
| `handover/` | Session compactions. | empty |
| `audit/` | Point-in-time audit reports. | empty |
| `archive/` | Completed ledger files. | empty |

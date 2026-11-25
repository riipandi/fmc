---
name: deep-audit
description: >
    Deeply review and test code for correctness, races, leaks, defects, security
    issues, performance problems, and stale or unnecessary code. Use for audits,
    code or security reviews, hardening, pre-release checks, bug hunting,
    optimization, or cleanup, even when phrased indirectly or in another language.
    An optional feature name narrows the scope; otherwise, audit the whole system.
metadata:
    argument-hint: "[feature-name]"
---

# Deep Audit

An evidence-based, end-to-end audit: find real problems (not a generic list of suggestions), verify them with tests or code traces, then clean up the codebase after the user approves scope and key decisions.

This skill is stack-agnostic in method. The repository is a Rust crate: use `cargo` and the Rust ecosystem's tools, and the examples in the reference files are illustrative only.

## Input

**Optional argument: feature name** (e.g. `file-tree`, `diff-view`, `search`). Take it from the skill argument, the text after the skill name, or `$ARGUMENTS` if the runtime provides it.

- **Feature name given**: audit that feature and every flow touching it (entry points, state, file-system and `git` access, rendering, input handling). Follow dependencies; do not stop at one module.
- **No argument**: audit the whole system. Start with the feature inventory (Phase 1), then ask the user to pick priorities if the system is large.
- **Name matches nothing in the code**: do not guess. Offer the closest candidates as choices.
- **TUI parity** (a screen, a key, a view, or the whole app compared with Scouter): use the `parity-audit` skill, which writes the same report shape. A whole-system audit covers the Rust code here and hands the parity part to `parity-audit`.

Project facts that are not findings: decisions recorded in `.agents/docs/architecture.md`, and the read-only phase in `.agents/docs/rules.md` (a write feature that does not exist yet is the plan, not a defect). Read `.agents/docs/rust.md` for the contracts before judging code against them.

## Language rules

- **Skill-internal content, and the report file saved under `.agents/docs/audit/`**: English (including frontmatter keys and values).
- **Everything said in chat** (summaries, findings overview, questions, progress notes): the language the user is currently writing in. If the user switches language, follow.
- Technical identifiers, code, file paths, and tool output stay as-is regardless of language.

## Working principles

1. **Evidence before claims.** Every finding needs a location (`file:line`), a concrete failure scenario, and ideally a test, repro, or measurement. Findings without evidence are labeled "hypothesis", not "defect". Reports full of false positives destroy trust.
2. **Read before changing.** Phases 0-4 are read-only on production code (adding tests is fine). Code changes start only after the user approves the plan in Phase 6.
3. **Verify before deleting.** "Unused" code is often reached via configuration, dynamic dispatch, or external consumers. Delete only with evidence.
4. **Ask when unsure; never assume.** See the confirmation protocol below.
5. **Prioritize by impact.** Rank by severity x likelihood x blast radius, not by discovery order.
6. **Stay in scope.** Out-of-scope issues go into the recommendations list; do not fix them on the side.

## Confirmation protocol

Stop and ask when you hit: ambiguous scope, system design or architecture decisions, trade-offs that change behavior (a public API change, a persisted format), deletions with inconclusive evidence, or changes touching high-risk areas (anything that writes to the user's files, runs a subprocess, or renders untrusted content).

Question format: follow "Ambiguous decisions" in the root `AGENTS.md` (concrete options, exactly one `recommended`, a final free-input option). Use an interactive question tool when the runtime has one. Additionally:

- One decision per question; at most 3 questions per turn. Batch small matters instead of interrupting repeatedly.
- Do not ask what can be answered by reading the code or repo.
- Write the questions in the user's current chat language.

## Workflow

### Phase 0 - Scope and context

Determine scope from the argument. Skim `README` files, `.agents/docs/`, the layout (`src/`), `Cargo.toml`, `Cargo.lock`, and CI config if any. Summarize in 3-5 lines: stack, architecture, audited feature(s), assumptions. If a scope decision matters (whole system vs part, which environments may be exercised), ask now.

### Phase 1 - Feature and flow inventory

Map entry points (key handlers, commands, background work), data flow, shared state, trust boundaries (file names, file contents, `git` output, configuration), and what the app reads from or writes to the file system. Output a list of flows to audit, e.g. "open directory -> list entries -> select file -> preview". For whole-system audits, show the list and ask the user to prioritize if it is too large. Then create the report file with `status: draft` (see Phase 5) and record the flow inventory in it.

### Phase 2 - Automated baseline

Run what already exists before analyzing by hand: `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`; a dependency vulnerability scan (`cargo audit`) and a secret scan when those tools are installed; sanitizers or `cargo miri test` where they apply. Record results and flaky tests. If something cannot be run, record it as a limitation; never skip it silently.

### Phase 3 - Deep audit by dimension

Walk each flow through the dimensions below. Read only the reference files that are relevant.

| Dimension                    | Focus                                                                                                  | Reference                             |
| ---------------------------- | ------------------------------------------------------------------------------------------------------ | ------------------------------------- |
| Correctness                  | Behavior matches the parity reference and the decisions, edge cases, error paths, file-system edge cases | `references/correctness.md`           |
| Concurrency and resources    | Races, blocking work on the input or render path, leaked handles and threads, terminal restore on exit | `references/concurrency-resources.md` |
| Security                     | Subprocess arguments, escape injection from file content, path traversal, secrets, supply chain        | `references/security-owasp.md`        |
| Performance                  | Per-frame work, file-system walks, subprocess calls, memory, duplicated work, needless complexity      | `references/performance.md`           |
| Code hygiene                 | Dead code, shims, legacy leftovers, duplication, stale config, tests, and docs                         | `references/cleanup.md`               |
| Operability and test quality | Logging to a file, error messages on screen, configuration, test quality and gaps                      | `references/operability-tests.md`     |

For each finding record: location, description, failure or exploit scenario, impact, severity, confidence, proposed fix.

### Phase 4 - Verification through testing

Turn important findings into proof:

- Write a test that **fails** because of the bug (regression test), in the module's `#[cfg(test)]` block or in `tests/`, or a repro script for races, leaks, and escapes.
- Races: run the tests under miri or a thread sanitizer where the toolchain supports it, plus repeated parallel runs.
- Leaks: measure memory and thread count before and after repeated runs.
- Performance: measure first (benchmark, profiler, timing). Never claim "faster" without before/after numbers.
- Add tests for critical flows that lack coverage, including failure paths. New tests follow the crate's conventions. Never touch the user's files: use fixtures in a temporary directory.
  Findings that cannot be reproduced may still be reported, labeled "unverified".

### Phase 5 - Report

Write the report using `references/report-template.md`. It starts with YAML frontmatter (status, timestamps, scope, counts, and other metadata) followed by: executive summary, findings table sorted by severity, per-finding details, cleanup candidates (with evidence), optimization opportunities (with numbers where available), unaudited areas, and open questions.

**Output location and naming**

- Save the report to `.agents/docs/audit/audit-<nnnnn>-<topic>.md` relative to the repository root. `<nnnnn>` is the next five-digit number: the highest existing `audit-` number across `.agents/docs/audit/` and `.agents/docs/archive/`, plus one, never reused. `<topic>` is the audited scope in lowercase kebab-case (`file-tree`, `whole-system`). Read `created_at` from the system clock (`date -Iseconds`), never guess it.
- Create the file early (end of Phase 1) with `status: draft` and fill it in as phases complete, so progress survives an interrupted session.
- If a report for the same scope already exists and the user is continuing it, update that file (keep `created_at`, refresh `updated_at`) instead of creating a new one. If unsure whether to continue or start fresh, ask. Never overwrite another report.
- Do not commit the report unless asked.
- A completed report moves unchanged to `.agents/docs/archive/` and its `index.md` row moves with it (`agent-docs`).
- Keep frontmatter and body consistent: counts, `status`, and `updated_at` must be updated on every edit.

**Language**

- The report file is always **English**.
- In chat, give the summary in the user's language: the top findings, the file path, and the decisions needed. If the report is short, it may be shown in full in chat in the user's language.

### Phase 6 - Decisions and fixes

1. Present an ordered fix plan (critical to low), grouped into independently shippable changes. For every decision that needs the user's choice, use the confirmation protocol.
2. After approval, work in small steps. After each step run the related tests; run the full suite at the end. Use one logical change per commit (do not commit or push unless asked).
3. Keep cleanup in batches separate from bug fixes so they are easy to review and revert.
4. Keep the report file current as work proceeds: set `status` (`approved` once the plan is accepted, `in_progress` while fixing, `completed` when done), update each finding's `status`, refresh the counts and `updated_at`, and fill `completed_at` at the end.
5. Finish with a summary: what was fixed, what was deferred and why, test results before vs after, remaining risks.

## Safety rules

- Do not run exploits or penetration tests against systems the user does not own. Verify vulnerabilities only with local fixtures in a temporary directory.
- Never run a destructive command (delete, move, overwrite) against the user's files to prove a finding.
- Never leak discovered secrets: report location and type only, mask values, and recommend rotation.
- Changes outside the approved scope become recommendations, not edits.

## Style

Be concise and specific. Avoid generic advice like "add validation" without saying where and why.

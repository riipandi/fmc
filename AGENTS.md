# Agents Instructions

Always-on rules. Detail lives in `.agents/docs/` (map: `index.md`). Before a change, read `.agents/docs/shape.md` (layout, commands) and the section of the area file it touches: `rust.md` (Rust code), `rules.md` (cross-area, including parity with Scouter), `architecture.md` (decisions).

## Working rules

- Verify behavior in the tree before documenting, testing, or building on it; a declared module, a file, or a doc entry is not proof. A claim in a plan or doc is a hypothesis until the tree confirms it.
- Early-stage project: no backward compatibility, no legacy shims; a replaced name or shape is removed.
- Do not add, remove, or rename a top-level directory unless asked; extend an existing module.
- No new dependency without the owner's approval (`rules.md`, "Dependencies").
- A comment carries what the code cannot (a why, a contract, a trap); never restate the next line.
- Chat follows the user's language. Files, code, identifiers, and commit messages are English.
- Sample data comes from Dan Brown's novels and Harry Potter, never `John Doe`, `foo`, or lorem ipsum (`rules.md`, "Sample data").
- Never print a secret or rewrite a `.env` file without consent.

## Parity and read-only

- **Scouter is the reference.** Phase 1 goal: 100% visual and functional parity with Scouter (<https://github.com/indrazm/scouter>). A behavior difference is a defect unless the owner approved it and the approval is recorded in `architecture.md` and listed in `docs/parity.md`, both naming the Scouter baseline (v0.3.4, `434d000f`). Port commits carry the trailer `Scouter-Base: v0.3.4 (434d000f)` (`rules.md`, "Parity with Scouter").
- **Verify Scouter before porting.** Establish each Scouter behavior from its source or from a live run through Rex (`rex` skill), never from memory. Cite the evidence: a `file:line` in Scouter, or the captured terminal output.
- **Writes are bounded.** The owner opened the file-manager phase (2026-11-27): create, rename, copy, move, and delete from the tree, inside the browsed root (`rules.md`, "Writes"). fmc's own `settings.json` is written only by the Settings page (`Ctrl+,`), atomically (plan-00015, D1; `rules.md`, "Writes"). At start fmc also writes `meta/config.schema.json` in its config folder when it differs from the running build (plan-00016, D1; `rules.md`, "Writes"); no other config file is written. Any other write (saving a file, another config file, a write outside the root) waits for the owner to open its phase.
- **The `opentui` locked source wins over docs.** Read the API of `opentui` and `opentui-sys` from the version in `Cargo.lock`; a README, doc page, or example that disagrees with the source is the wrong one.

## Ambiguous decisions

Stop and ask before writing code, public APIs, or config when the request fits more than one module or screen; implies an unnamed keybinding, screen, config key, or flag; is hard to undo (a public API, a persisted format, a keybinding users rely on); or needs an invented product rule. A parity question that Scouter's source and live run do not answer is a question for the owner. A rule the docs already settle is not a question.

Ask in this shape, then wait; if the owner already chose in the thread, do not re-ask (confirm only a delta that new facts cause):

```text
Unclear: <decision> — wrong pick means <cost>.

1. <option> — <consequence>
2. <option> — <consequence>  [recommended: <one reason>]
3. Other — reply with the design you want
```

## Workflow

- **Docs sync.** A change lands its documentation in the same commit; `rules.md` ("Where a fact lives") says which file. Outdated prose is a defect of the change. The `agent-docs` skill carries the closing sweep.
- **Findings.** A finding that outlives the turn goes to `.agents/docs/issues/` when found (`agent-docs`).
- **Plans.** A plan in `.agents/docs/plans/` is a contract; it runs only on the owner's explicit go-ahead, phase by phase.
- **Validation.** Run the gates in `shape.md` ("Commands") before claiming a change works. A change to what is drawn or how input is handled also needs a live run (`run` skill or Rex): the unit suite never sees the rendered screen.
- **Rex live runs.** Never create a new Rex session to test. Open a new tab (window) in the session your own pane belongs to: `rex run -s "$REX_BLOCK" --focus=false -- <command>` (a block ID resolves to its session), resize its block with `rex block call resize`, and close the tab when done. Never type into the owner's own panes.
- **Owner check.** When finished work leaves something for the owner, end the chat reply with an `Owner check` section: a numbered list of actions, each saying what to do and why. It covers a check only the owner can do (a phone, a real terminal, colours), a decision or feedback the agent needs, a command the agent was denied (give it as `! <command>`), and a push or cleanup waiting for approval. Leave the section out when nothing is waiting. A subagent's report carries the same section, and the coordinator passes its items to the owner.
- **Library docs.** Check crates against the locked version (docs.rs, changelogs); the locked source wins.
- **Audience.** `.agents/docs/` is for agents (English, `file:line` evidence). Human docs are `README.md`, `PRODUCT.md` (purpose, users, principles), `DESIGN.md` (palettes, layout, components), and `docs/` (created by the plan phase that lands its first page): no paths or internals. During parity, Scouter wins over `DESIGN.md`; a disagreement is a defect in `DESIGN.md`.

## Subagents

Project agents live in `.claude/agents/`: `executor` (Sonnet) runs approved plan phases; `scout` (Haiku) answers narrow read-only lookups. Use them instead of a fork for that work; a fork always runs on the main model.

## Skills

A skill advises; this file and `.agents/docs/` win. Project skills live in `.agents/skills/` (`.claude/skills` links there).

- `agent-docs`: any ledger or docs work (plan, issue, roadmap, handover, audit, index).
- `handoff`: writes a handover on request.
- `rust-verify-harden`: before a Rust change claims done.
- `deep-audit`: evidence-based audit of a feature or the whole system; reports go to `.agents/docs/audit/`.
- `parity-audit`: screen-by-screen comparison of fmc with Scouter.
- `security-review`: before a change that reads paths, runs subprocesses, or renders file content claims done.
- `clarity`: human prose (commit and PR text, docs).

## Git

- New work never starts on `main` or another protected branch. Run `git branch --show-current` before the first edit; on `main` or a protected branch, create a branch for the work (`feat/…`, `fix/…`, `docs/…`, `release/vX.Y.Z`) and work there. Subagents stay on the branch the brief names. The work reaches `main` through a pull request.
- Stage explicit paths and check `git status`; never `git add -A` or `git add .`.
- Message: `{feat,fix,docs,refactor,test,chore}[(scope)]: <concise message>`. Never push unasked.
- Commit only when asked; a plan go-ahead covers its phases' commits. Otherwise recommend the message and paths.
- Never `git reset --hard`, `git checkout .`, `git clean -fd`, `git stash`, `git commit --no-verify`, or `git push --force`.

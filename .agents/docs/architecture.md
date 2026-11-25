# Architecture

updated: 2026-11-26T03:19:42+07:00

The architecture and the reasoning behind it. The layout lives in `shape.md`; the binding rules derived from these decisions live in `rules.md` and `rust.md`. This file keeps the why, so a settled decision is not re-litigated.

## Decision records

One record per decision. Format:

```text
### <The decision, stated as the state of the world>

Context: <the forces: the constraint, the problem, the Scouter behavior it touches. One short paragraph.>

Decision: <what was picked, and the one reason that survived the alternatives.>

Consequences:

- (positive) <the benefit, named concretely>
- (negative) <the cost, the trade-off, or the capability given up>

Alternatives considered:

- <alternative> — rejected: <the reason, specific to this tree>
```

An approved parity deviation is a record too. It names the Scouter behavior (with its `file:line` or its decision record), the fmc behavior, and the owner's approval with its date. A difference with no record is a defect.

No decision is recorded yet.

## Pending decisions

Open questions for the owner. None is decided; each one blocks the code that depends on it.

- Module layout of `src/`: `main.rs` is the only file today.
- Terminal loop and rendering: `main.rs` draws into a buffer and never enters a terminal. The event loop, the render path, and terminal restore on exit and on panic are not designed yet.
- Markdown and Mermaid preview: how Scouter renders them (verify first with `parity-audit`), and how fmc matches the result.
- Git diff viewer: whether fmc runs `git` as a subprocess the way Scouter does (`src/diff/git.ts`, to verify first), or uses a library. A library is a dependency decision.

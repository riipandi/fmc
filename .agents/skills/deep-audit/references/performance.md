# Performance and Optimization

Core rule: **measure first, optimize second.** Every recommendation carries evidence (profile, benchmark, timing) or is labeled "analysis-based, not measured". Optimizations that add complexity without measurable benefit are rejected.

## File system

- Directory walks that stat every entry when the screen shows only a window of rows; read only what is visible.
- Large files: Scouter's own rule for them is its decision record 0019. Verify it before reusing it; do not invent a limit.
- The same file read more than once per frame or per navigation; cache by path and modification time, and invalidate on change.

## Rendering

- Work per frame: no allocation or full re-layout per cell or per line; render only what is visible.
- Refresh cadence: Scouter polls `git` to refresh its diff (decision 0006) and refreshes its browser (decision 0017). Match its cadence after verifying it; a faster poll is a defect against parity, not an optimization.

## CPU and algorithms

- Quadratic work on large lists: repeated linear searches where a map or set fits; repeated sorts of the same data.
- Expensive regular expressions inside a per-line loop; compile once.
- Syntax highlighting: parse once per file version, not once per frame.

## Memory

- Whole-file reads where a window is enough; unbounded buffers for diffs and search results.
- Verification: heap profile or allocation counts before and after.

## Subprocesses

- One `git` call that returns the data for many files, not one call per file.
- Timeouts on every call (see `concurrency-resources.md`).

## Dedupe and unnecessary complexity

The same logic in more than one place; near-identical helpers; single-implementation abstractions without a reason; pass-through layers; configuration for cases that never occur. Consolidate when there is one clear source of truth, and protect the change with tests first.

## Recommendation format

Problem -> evidence or measurement -> proposed change -> expected benefit -> risk or trade-off -> effort (S/M/L).

# Correctness

Goal: confirm each flow behaves as expected, including with odd input or failing dependencies.

## Determining "expected"

Order of truth sources: Scouter's behavior (its source or a live run) for any parity question -> decisions in `.agents/docs/architecture.md` -> existing tests -> current code behavior. If sources conflict or are missing, ask the user what the correct behavior is. Do not treat current behavior as correct by default.

## Per-flow checklist

- Happy path works end to end, including what the screen shows.
- Input validation: empty input, very long input, non-UTF-8 and Unicode names, wide and combining characters, a zero-size or very small terminal, the largest supported size.
- File system: symbolic links (loops, links that point out of the tree), permission errors, entries removed or renamed while listed, very large files, special files (FIFOs, sockets, devices).
- Error paths: a failed read, a failed `git` call, a missing repository. Is the error shown? Is the state left consistent?
- Keys and focus: every key has one owner; repeated or held keys; the focus after an error or a resize.
- Refresh: the state after a file-system or `git` change matches what is on disk (no stale selection, no stale diff).
- Scrolling and selection: first and last item, an empty list, a selection beyond a list that shrank.
- Text handling: byte offsets are never used to slice a string that may hold multi-byte characters.
- Configuration: safe defaults, behavior when a config is missing or wrong.

## Commonly missed bug patterns

Wrong comparison (equality vs identity), shared mutation of arguments, null or `None` checks after use, errors wrapped and losing their type, cleanup blocks overwriting the original error, stale caches, silent integer truncation (`as` casts between widths and signs), off-by-one at the last row or column, partial-failure handling in batch operations, and string slicing at byte offsets inside a multi-byte character.

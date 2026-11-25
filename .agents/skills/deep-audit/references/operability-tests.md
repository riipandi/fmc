# Operability and Test Quality

## Operability

- Logging: a terminal app cannot print log lines onto the screen it draws. Log to a file, or not at all; levels set by configuration; no file contents, secrets, or personal paths at the default level.
- Error handling: errors are not swallowed; the screen shows a message the user can act on; internal detail stays out of it.
- Responsiveness: a slow file system or `git` call never freezes input; subprocesses have timeouts; work is cancelled when the user leaves the view.
- Configuration: validated at startup (fail fast), safe defaults, no environment-specific values in code.
- Shutdown: the terminal is restored and background work stops on exit (see `concurrency-resources.md`).
- Build: `Cargo.lock` is committed and the release build is reproducible from it; the dependency set stays small and approved (`rules.md`, "Dependencies").

## Test quality

- Coverage of critical flows and failure paths, not just a coverage percentage.
- Flaky tests (timing, ordering, file-system state, shared temporary directories) → stabilize or quarantine with a note.
- Screen tests assert the rendered cells of a fixed-size buffer, not the implementation. Where parity is the claim, the expected output comes from a Scouter capture (`parity-audit`).
- Over-mocking: prefer a real directory tree in a temporary directory and a real `git` repository over mocks at the file-system and `git` boundaries.
- Tests asserting implementation instead of behavior (brittle under refactor).
- Missing tests for symbolic link loops, non-UTF-8 names, very large files, and terminal sizes at the edges.

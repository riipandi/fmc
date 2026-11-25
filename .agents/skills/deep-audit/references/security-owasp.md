# Security

Scope: a local terminal app that reads the user's file system, runs `git`, and renders file content. There is no server, no account, and no network surface today. Re-scope this reference when a network feature arrives; the OWASP Top 10 and ASVS apply then.

## What to look for

- **Command injection (OWASP A03)**: every subprocess call passes its arguments as a list (`Command::new("git").args([...])`), never through a shell string. File names and branch names from the repository are data, not flags: check that a name starting with `-` cannot become an option.
- **Terminal escape injection**: file content and `git` output are untrusted. Control sequences (ESC, CSI, OSC) reaching the terminal can rewrite the screen, change the title, or write to the clipboard. They must be escaped or stripped before drawing.
- **Path traversal and symbolic links (OWASP A01)**: paths built from input or from the repository stay inside the intended root. Whether a symbolic link is followed or shown as a link is a parity question: verify Scouter first.
- **Secrets in the tree**: a secret scanner over the repository when one is installed. Report location and type only, never the value.
- **Logs (OWASP A09)**: no file contents or secrets in logs; logs go to a file, never to the terminal the app draws on.
- **Dependencies (OWASP A06)**: `cargo audit` when installed; report advisories against the locked versions of `opentui` and `opentui-sys` and against any new dependency.
- **Resource exhaustion from content**: very large files, deeply nested directories, and pathological regular expressions (ReDoS) in search. Bound the work and the memory each one can take.

## How to report

For each finding: category (the OWASP ID or the name above), location (`file:line`), a local reproduction (a temporary directory with a crafted file name or file content), impact, severity (scale in `report-template.md`), and the concrete fix. Mask secret values.

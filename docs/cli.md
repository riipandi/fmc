# Command line

```text
fmc [file-or-directory]
fmc --version
fmc --help
```

| You run | fmc does |
| --- | --- |
| `fmc` | Opens the current folder. |
| `fmc <folder>` | Opens the folder as the root of the file list. |
| `fmc <file>` | Opens the file in the reader. The file list shows the file's folder. |
| `fmc -h`, `fmc --help` | Prints the usage, the options, and where the keys, the settings, and these docs are, then exits. It wins over everything else on the line. |
| `fmc -v`, `fmc --version` | Prints `fmc` and its version, then exits. |

fmc accepts one file or folder. Any other word on the line, even one that starts with `-`, is read as a path.

## Errors

fmc prints the error on one line that starts with `fmc:` and exits with code 1.

| When | The message |
| --- | --- |
| More than one path | `Expected one file or directory. Use --help for usage.` |
| Input or output is not a terminal | `Run fmc in an interactive terminal.` fmc checks this before it looks at the path. |
| The path does not exist | `ENOENT: no such file or directory, stat '<path>'` |
| The path is not a file or a folder, such as a socket | `Only files and directories can be opened.` |

fmc has no `--upgrade`. Install a new version the way you installed fmc.

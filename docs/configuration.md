# Configuration

fmc reads one optional settings file when it starts. It never writes that file. Edit it by hand, then start fmc again.

## Location

`$XDG_CONFIG_HOME/fmc/settings.json`, or `~/.config/fmc/settings.json` when `XDG_CONFIG_HOME` is unset or empty.

## Keys

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `auto_preview` | `true` or `false` | `true` | Whether moving through the file list shows each file in the reader. |
| `sidebar_jump` | whole number, 1 or more | `2` | How many rows `[` and `]` move the file list. |
| `reader_jump` | whole number, 1 or more | `5` | How many lines `[` and `]` scroll the reader. |
| `git_gutter` | `true` or `false` | `true` | Whether the reader marks the lines of a file in a git repository that differ from `HEAD`: in a strip left of the line numbers, and beside the changed blocks of the rendered Markdown view. |
| `file_associations` | object of pattern to filetype name | none | Which filetype a file is highlighted as, and whether `m` renders it as Markdown. See [File associations](#file-associations). |
| `find_mode` | `"substring"` or `"fuzzy"` | `"substring"` | How the file finder matches names when it opens. `Ctrl+F` in the finder flips it for the session. See [Finder](#finder). |

A key that is not listed is ignored.

## Errors

A file that is not valid JSON, or whose top level is not an object, sets nothing: every key keeps its default. A key with the wrong type, or a jump below 1, keeps its own default. A `file_associations` entry that fails is skipped, and the others still apply. fmc shows the problems in the footer once, when it starts. A missing file means the defaults, with no message.

## File associations

`file_associations` works like VS Code's `files.associations`: each key is a pattern, and each value is the filetype a matching file is shown as.

```json
{
  "file_associations": {
    "*.rs.bak": "rust",
    "*.env.example": "dotenv",
    ".env.*": "dotenv",
    "docs/**/*.txt": "markdown"
  }
}
```

- A pattern with no `/` matches the file name, in any folder. A pattern with a `/` matches the path from the folder fmc opened. A file outside that folder matches by its name only.
- `*` matches any run of characters within one name or folder, `**` any number of folders (`docs/**/*.txt` matches `docs/owls.txt` too), `?` one character, and `{a,b}` either alternative.
- Case follows the filesystem: a pattern ignores case on macOS and Windows, and matches case exactly elsewhere.
- Values are fmc's filetype names, such as `rust`, `bash`, `json`, `markdown`, or `typescriptreact`. Besides the languages Scouter colours, fmc highlights `astro`, `diff`, `elixir`, `heex`, `hcl`, `jsdoc`, `lua`, `regex`, `sql`, `svelte`, `vim`, and `xml`. `shellscript` means `bash`, `jsonc` means `json`, and `dotenv` and `env` mean `bash`, which reads `KEY=value` and `#` comments. `plaintext` shows the file as plain text, with no highlighting. A name fmc does not know is skipped with a notice that names the pattern.
- When several patterns match a file, a path pattern wins over a name pattern, then the longer pattern wins, then the pattern that sorts first.
- An association decides the highlighting in the reader, in the finder's preview, and on the changes screen. A file with a `markdown` association opens rendered, and `m` shows its source.
- fmc has no built-in association for env files. The example above is only an example.

## Finder

`find_mode` sets how the file finder (`/`) matches a name when it opens. `"substring"` is Scouter's match: every word of the query must appear in the path, ignoring case. `"fuzzy"` takes the whole query as one pattern, spaces included, and matches its letters in order anywhere in the path, ignoring case. The best matches come first, and the letters that matched are highlighted.

`Ctrl+F` in the finder flips the match for the rest of the run, and the query runs again; the footer names `fuzzy` while it is on. A value that is not one of the two names is ignored with a notice, and `"substring"` is used.

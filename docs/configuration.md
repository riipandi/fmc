# Configuration

fmc reads one optional settings file when it starts. The [Settings page](usage.md#settings) (`Ctrl+,`) is the only thing that writes it: each change you make there is saved at once. You can also edit the file by hand. A hand edit made while fmc runs applies when fmc starts again.

## Location

`$XDG_CONFIG_HOME/fmc/settings.json`, or `~/.config/fmc/settings.json` when `XDG_CONFIG_HOME` is unset, empty, or relative.

## Keys

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `auto_preview` | `true` or `false` | `true` | Whether moving through the file list shows each file in the reader. |
| `sidebar_jump` | whole number, 1 or more | `2` | How many rows `[` and `]` move the file list. |
| `reader_jump` | whole number, 1 or more | `5` | How many lines `[` and `]` scroll the reader. |
| `git_gutter` | `true` or `false` | `true` | Whether the reader marks the lines of a file in a git repository that differ from `HEAD`: in a strip left of the line numbers, and beside the changed blocks of the rendered Markdown view. |
| `file_associations` | object of pattern to filetype name | none | Which filetype a file is highlighted as, and whether `m` renders it as Markdown. See [File associations](#file-associations). |
| `find_mode` | `"substring"` or `"fuzzy"` | `"substring"` | How the file finder matches names when it opens. `Ctrl+F` in the finder flips it for this run only. See [Finder](#finder). |
| `theme_mode` | `"auto"`, `"dark"`, or `"light"` | `"auto"` | Which palette to show. `auto` follows the terminal's background; `dark` and `light` force one. See [Themes](#themes). |
| `dark_theme` | a theme id | `"default"` | The palette for dark mode: `default` (Scouter's own) or a theme id that is dark. See [Themes](#themes). |
| `light_theme` | a theme id | `"default"` | The palette for light mode: `default` or a theme id that is light. See [Themes](#themes). |

A key that is not listed is ignored.

The file keeps no UI state. The sidebar's width and visibility, wrap, the changes screen's split and context lines, and dotfiles are not saved: a change to them lasts until fmc quits.

## Errors

A file that is not valid JSON, whose top level is not an object, or that is over 1 MiB sets nothing: every key keeps its default. A key with the wrong type, or a jump below 1, keeps its own default. A `file_associations` entry that fails is skipped, and the others still apply. fmc shows the problems in the footer once, when it starts. A missing file means the defaults, with no message.

## Editor completion

fmc writes a JSON Schema for the settings file to `$XDG_CONFIG_HOME/fmc/meta/config.schema.json`, or `~/.config/fmc/meta/config.schema.json`, each time it starts. An editor that reads a `$schema` key then completes the keys and their values, and flags a misspelt key or a value of the wrong type.

- When `settings.json` exists and has no `$schema`, fmc adds `"$schema": "./meta/config.schema.json"` as its first key at start. Nothing else in the file changes: your values, their order, and the layout stay. A `$schema` you wrote stays as it is, and a file that is not valid JSON is left alone. The Settings page adds the same line when it creates the file.
- The path is relative to the folder of `settings.json`, and VS Code and Zed resolve it there. If you move the file, change the path.
- fmc rewrites the schema only when it is missing or differs from the one this version writes. Do not edit it: a change is overwritten at the next start.
- A theme id in `dark_theme` or `light_theme` completes from the bundled themes of its mode. The schema accepts any text there, so a theme in your `themes` folder is valid too.
- A folder fmc cannot write, such as a read-only one, shows a footer notice at each start until the write works. The settings file is still read.
- `fmc --help` and `fmc --version` write nothing. Starting fmc writes the schema only when it is missing or out of date.
- VS Code and Zed both use the vscode-json-languageservice library, which reads JSON Schema draft-07.

## Saving from the Settings page

The Settings page edits the eight keys above except `file_associations`: `auto_preview`, `sidebar_jump`, `reader_jump`, `git_gutter`, `find_mode`, `theme_mode`, `dark_theme`, and `light_theme`. Edit `file_associations` by hand.

- Each change is saved when you make it. A save reads the file again and sets only the keys the page changed. Every other key stays as it is, including keys fmc does not know and hand edits made since fmc started.
- A save writes the keys in alphabetical order, so the first save of a file that was in another order reorders it. Values keep their types.
- A file that is not valid JSON, or whose top level is not an object, is refused. Nothing is written, the file keeps its bytes, the page shows the value the file has, and the footer says `Not saved: settings.json is not valid JSON (line N); nothing was saved. The value went back.`. Fix the file, then make the change again.
- A missing file is created with the one key and the `$schema` line (see [Editor completion](#editor-completion)). A missing folder is created too.
- A `settings.json` that is a symbolic link is written through to the file it points to, and the link stays. The target can lie anywhere the link reaches.
- Each save is written to a temporary file beside the target, which then replaces it, so a crash leaves the old file or the new one. Comments in the file are not kept: the parser refuses them, so a save cannot drop them silently.
- Without a usable home folder (no `HOME`, and no absolute `XDG_CONFIG_HOME`), the page cannot save, and it says so.

## Themes

Each mode has one palette. `dark_theme` picks the dark one and `light_theme` the light one, and `theme_mode` picks which one shows. `default` is Scouter's own palette, and it is the default for both. Other ids come from the bundled catalog or from your own files:

- The bundled themes are: dark, `catppuccin`, `tokyo-night`, `gruvbox`, `kanagawa`, `rose-pine`, `one-dark`, `everforest`, and `nord`; light, `light`, `catppuccin-latte`, `tokyo-night-day`, `gruvbox-light`, `rose-pine-dawn`, `one-light`, and `flexoki-light`. They are bjarneo/editor's themes (MIT).
- A file in `$XDG_CONFIG_HOME/fmc/themes/` (or `~/.config/fmc/themes/`) named `<id>.json` is a theme with that id. A file with the id of a bundled theme replaces it. Other files in the folder are ignored.

A theme file is strict JSON, with no comments:

```json
{
  "name": "Hogwarts Dusk",
  "mode": "dark",
  "bg": "#101418", "bg2": "#0c0f12", "bg3": "#1c232b", "bd": "#2b3440",
  "fg": "#dfe6ee", "fg2": "#9aa6b4", "fg3": "#6f7b89", "ac": "#e0b25a", "ins": "#8fd19e",
  "vis": "#7c9cd6", "cmd": "#d6a0d6", "kw": "#c792ea", "fn": "#82aaff", "str": "#a7c7a1",
  "num": "#f2a65a", "ty": "#5ec4c4", "mac": "#f07a8a", "com": "#7f8a99", "pun": "#b2b9c3",
  "prop": "#e5c07b", "warn": "#e5b880", "err": "#f2a1a1", "sel": "#2c365e", "on": "#101418"
}
```

- `mode` is `"dark"` or `"light"`, and the id must be used in the slot of its mode. `name` is optional; it defaults to the id.
- The 24 colour keys are required, each a `#rrggbb` value. Keys fmc does not use are checked, not shown: `bg2`, `vis`, `cmd`, `ty`, `mac`, and `prop`. Any other key is ignored.
- A file that cannot be read or parsed is skipped, and the footer names it. The id `default` cannot be a file: Scouter's palette is built in.
- A `dark_theme` or `light_theme` that is not a theme, or that names a theme of the other mode, keeps `default` for that slot and the footer says so.
- The page background is the terminal's background, tinted as Scouter tints it. When `theme_mode` forces a mode that is not the terminal's, the page uses the palette's own background instead.
- Themes are read once, when fmc starts, like the other keys. fmc only reads a theme file, and the Settings page does not write one either.

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

`Ctrl+F` in the finder flips the match for this run only, and the query runs again; the footer names `fuzzy` while it is on. A restart returns to `find_mode`: a flip is not saved. Change `find_mode` on the Settings page to keep a different start. A value that is not one of the two names is ignored with a notice, and `"substring"` is used.

## Keymap file

`$XDG_CONFIG_HOME/fmc/keymap.json`, or `~/.config/fmc/keymap.json`, sets key bindings. It is a JSON object of `"<context>.<key>": "<action>"` entries, or `null` to unbind a default key. fmc only reads it when it starts, so a change applies on the next start. A bad entry is skipped with a footer notice that names its line, and the other entries apply; a file that is not a JSON object keeps every default. The shape, the context and action names, and the notices are in [Keys](keymaps.md#changing-the-keys).

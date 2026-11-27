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
| `find_mode` | `"substring"` or `"fuzzy"` | `"substring"` | How the file finder matches names when it opens. `Ctrl+F` in the finder flips it for this run only. See [Finder](#finder). |
| `theme_mode` | `"auto"`, `"dark"`, or `"light"` | `"auto"` | Which palette to show. `auto` follows the terminal's background; `dark` and `light` force one. See [Themes](#themes). |
| `dark_theme` | a theme id | `"scouter"` | The palette for dark mode: `scouter` (Scouter's own) or a theme id that is dark. See [Themes](#themes). |
| `light_theme` | a theme id | `"scouter"` | The palette for light mode: `scouter` or a theme id that is light. See [Themes](#themes). |

A key that is not listed is ignored.

## Errors

A file that is not valid JSON, or whose top level is not an object, sets nothing: every key keeps its default. A key with the wrong type, or a jump below 1, keeps its own default. A `file_associations` entry that fails is skipped, and the others still apply. fmc shows the problems in the footer once, when it starts. A missing file means the defaults, with no message.

## Themes

Each mode has one palette. `dark_theme` picks the dark one and `light_theme` the light one, and `theme_mode` picks which one shows. `scouter` is Scouter's own palette, and it is the default for both. Other ids come from the bundled catalog or from your own files:

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
- A file that cannot be read or parsed is skipped, and the footer names it. The id `scouter` cannot be a file: Scouter's palette is built in.
- A `dark_theme` or `light_theme` that is not a theme, or that names a theme of the other mode, keeps `scouter` for that slot and the footer says so.
- The page background is the terminal's background, tinted as Scouter tints it. When `theme_mode` forces a mode that is not the terminal's, the page uses the palette's own background instead.
- Themes are read once, when fmc starts, like the other keys. fmc never writes a theme file.

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

`Ctrl+F` in the finder flips the match for this run only, and the query runs again; the footer names `fuzzy` while it is on. A restart returns to `find_mode`: fmc never writes `settings.json`, so a flip is not kept. A value that is not one of the two names is ignored with a notice, and `"substring"` is used.

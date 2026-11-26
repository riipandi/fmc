---
name: fmc
description: A quiet, read-only terminal viewer. The look is Scouter's, cell for cell.
colors:
  dark:
    text: "#d4d8de"
    muted: "#8b929c"
    dim: "#666d78"
    ignored: "#4a505a"
    border: "#2a2f3a"
    row-active-bg: "#222633"
    folder: "#8fa3bf"
    selection-bg: "#2c365e"
    accent: "#818cf8"
    accent-soft: "#c7d2fe"
    accent-ink: "#0f1117"
    error: "#f2a1a1"
    hover: "#343a42"
    added: "#8fd3a8"
    removed: "#f2a1a1"
    modified: "#e5b880"
    added-bg: "#12281c"
    removed-bg: "#2b1518"
    added-gutter: "#173a27"
    removed-gutter: "#3b1b20"
    match-fg: "#0f1117"
    match-bg: "#e5b880"
    target-bg: "#262b46"
    target-gutter: "#343b66"
    wordmark: ["#a5b4fc", "#818cf8", "#6366f1"]
    fallback-background: "#131519"
    syntax-keyword: "#c792ea"
    syntax-string: "#a7c7a1"
    syntax-comment: "#8b929c"
    syntax-constant: "#e5b880"
    syntax-function: "#82aaff"
    syntax-punctuation: "#b2b9c3"
  light:
    text: "#1f2328"
    muted: "#59636e"
    dim: "#7d8590"
    ignored: "#b1b8c0"
    border: "#d0d7de"
    row-active-bg: "#eaeef2"
    folder: "#4c5c75"
    selection-bg: "#d8e0ff"
    accent: "#4f46e5"
    accent-soft: "#4338ca"
    accent-ink: "#ffffff"
    error: "#cf222e"
    hover: "#d0d7de"
    added: "#1a7f37"
    removed: "#cf222e"
    modified: "#9a6700"
    added-bg: "#dafbe1"
    removed-bg: "#ffebe9"
    added-gutter: "#aceebb"
    removed-gutter: "#ffcecb"
    match-fg: "#1f2328"
    match-bg: "#ffdf5d"
    target-bg: "#e0e7ff"
    target-gutter: "#c7d2fe"
    wordmark: ["#6366f1", "#4f46e5", "#3730a3"]
    fallback-background: "#f6f8fa"
    syntax-keyword: "#8250df"
    syntax-string: "#116329"
    syntax-comment: "#646e78"
    syntax-constant: "#953800"
    syntax-function: "#0550ae"
    syntax-punctuation: "#57606a"
typography: none (the terminal font is the user's; a Nerd Font is needed for icons)
layout:
  sidebar-width: "min(40, max(28, floor(terminal width × 0.25))) columns"
  sidebar-min: 24
  reader-min: 28
  header-rows: 2
  footer-rows: 2
  find-bar-rows: 2
  split-diff-min-reader: 60
  resize-step: 2
  tint: 0.025
---

# Design System: fmc

## Overview

fmc looks like Scouter. Until parity is reached, Scouter's screen is the reference, and this file only describes it. If this file and Scouter disagree, Scouter wins and this file is wrong.

The interface is quiet: two bordered panels, a two-row footer, and colour used for meaning (focus, git state, matches), not decoration.

## Colors

Two complete palettes, dark and light, in the frontmatter. fmc picks one from the terminal background: brightness `0.299 r + 0.587 g + 0.114 b` below 0.5 is dark. It checks again when the terminal announces a change, and every 3 seconds while the terminal answers. With no answer, it uses dark.

### Named Rules

- Any colour outside the palette is a bug.
- The footer band and the diff file headers use the terminal background mixed 2.5% toward white (dark) or black (light). With an unknown background they use `fallback-background`. The sidebar is not tinted.
- Accent (indigo) marks focus: the focused panel border, pills, the selected row marker `▎`, prompt marks.
- Git states always pair a colour with a letter: `M` modified, `A` added, `D` deleted, `R` renamed, `T` type change, `U` conflict, `?` untracked.

## Typography

The terminal font is the user's. Emphasis is bold and colour; comments are italic. File icons are Nerd Font glyphs, not emoji.

## Layout

A column: a row of two panels that fills the screen, then the footer.

- **Sidebar** (left): the file tree, or the change list on the changes screen. Width `min(40, max(28, 25% of the terminal))`; at least 24 columns, and the reader keeps at least 28. A width the user sets is kept for the session. Below 80 columns it hides while a document is open, and `Ctrl+/` or the mark on its border shows it (`docs/parity.md`, Layout).
- **Reader** (right): the file, the rendered Markdown, the image, or the stacked diff.
- **Panels**: rounded borders; the focused panel border is accent, the other is `border`.
- **Header**: two rows at the top of each panel, with a hairline under it.
- **Footer**: two rows. Row one: a pill, the location or a message, and facts on the right separated by ` · `. Row two: key hints, dropped from the end to fit, with `^q quit` kept last.

## Components

### Tree row

`▎` on the selected row, depth guides `│ `, the folder chevron `▾` or `▸`, the icon, the name (folders end with `/`), and on the right a bold git letter or a `•` for a folder with changes. The selected row has the `row-active-bg` surface and an accent, bold name. Ignored files use `ignored`.

### Panel header

The sidebar shows the folder name with a quiet `files` badge (`no dotfiles` while dotfiles are hidden), or `Changes` with a count. The reader shows a dim parent path and a bold file name, then badges: quiet (`read-only`, `rendered`, `image`, `binary`, `all files`, `one file`) or pills (`first 1 MiB`, `wrap`, `split`, `wide context`, `preview`). An error replaces the badges in `error`, bold, until the next key.

### Footer pills and hints

Pills name the context: `FILES`, `READER`, `CHANGES`, `DIFF`, `FIND`, `TEXT`, `SEARCH`. A pill is `accent-ink` on `accent`, bold. Hints show the key in `accent-soft` and the action in `dim`. Position reads `All`, `Top`, `Bot`, or a percentage.

### Diff file band

One band per changed file on the tinted background: the marker, a fold chevron, the bold coloured letter, the bold name (`accent-soft` when active), `← old` for a rename, the dim folder, and on the right the counts and a five-cell stat bar of `▪` (green for added, red for removed, at least one cell each when both exist). Hunk header lines are not drawn.

### Help card

A centred, rounded card titled ` Keyboard Shortcuts `, with sections Browse, Reader, Changes, Everywhere, and Mouse. Any key closes it.

### Welcome

A three-row block wordmark in the three `wordmark` colours, a tagline with the version, a rounded hint card, and a line that says there is no editing.

## Do's and Don'ts

### Do

- Match Scouter's screen first; describe it here second.
- Use the palette tokens by name.

### Don't

- Add colour for decoration.
- Use emoji for icons.
- Let this file overrule Scouter during the parity phase.

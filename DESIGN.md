---
name: fmc
description: A quiet terminal viewer and file manager. The look is Scouter's, cell for cell; the menu and the dialog are fmc's own.
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
  compact-below: 60
  header-rows: 2
  footer-rows: 2
  find-bar-rows: 2
  compact-footer-rows: 1
  split-diff-min-reader: 60
  resize-step: 2
  tint: 0.025
---

# Design System: fmc

## Overview

fmc looks like Scouter. Until parity is reached, Scouter's screen is the reference, and this file only describes it. If this file and Scouter disagree, Scouter wins and this file is wrong.

The interface is quiet: two bordered panels, a two-row footer (one row below 60 columns), and colour used for meaning (focus, git state, matches), not decoration.

## Colors

Two complete palettes, dark and light, in the frontmatter. fmc picks one from the terminal background: brightness `0.299 r + 0.587 g + 0.114 b` below 0.5 is dark. It checks again when the terminal announces a change, and every 3 seconds while the terminal answers. With no answer, it uses dark.

### Named Rules

- Any colour outside the palette is a bug.
- The footer band and the diff file headers use the terminal background mixed 2.5% toward white (dark) or black (light). With an unknown background they use `fallback-background`. The sidebar is not tinted.
- Accent (indigo) marks focus: the focused panel border, pills, the selected row marker `▎`, prompt marks.
- Git states always pair a colour with a letter: `M` modified, `A` added, `D` deleted, `R` renamed, `T` type change, `U` conflict, `?` untracked.

### Catalog

Scouter's pair above is the default for both modes, and `theme_mode`, `dark_theme`, and `light_theme` in the settings file choose another (`docs/configuration.md`, "Themes"). The catalog holds 16 bundled themes, from bjarneo/editor (MIT): nine dark and seven light. Each has 24 colour roles, and fmc takes its tokens from them:

- `bg` to `fallback-background` and `match-fg`; `fg` to `text`; `fg2` to `muted`; `fg3` to `dim`; `bd` to `border`; `bg3` to `row-active-bg` and `hover`; `ac` to `accent`; `on` to `accent-ink`; `sel` to `selection-bg`; `err` to `error` and `removed`; `warn` to `modified` and `match-bg`; `ins` to `added`; and `kw`, `fn`, `str`, `num`, `com`, and `pun` to the six syntax tokens.
- The other tokens are mixes of two roles, with fmc's own weights: `accent-soft`, `ignored`, `folder`, `added-bg`, `removed-bg`, `added-gutter`, `removed-gutter`, `target-bg`, `target-gutter`, and the first and last `wordmark`. The middle `wordmark` is `accent`.

The page background is the terminal's, tinted as above. Not every bundled theme reaches the contrast Scouter's pair does on its selection, tints, and match colours; `src/theme/theme_tests.rs` lists which pairs fall short.

## Typography

The terminal font is the user's. Emphasis is bold and colour; comments are italic. File icons are Nerd Font glyphs, not emoji.

A heading in the rendered Markdown view keeps its marker (`# `, `## `, `### `, and so on) in the heading's own colour and weight, so the level reads without a larger font. Emphasis, code, and link markers stay hidden. Scouter hides the heading marker too; the difference is in `docs/parity.md`.

## Layout

A column: a row of two panels that fills the screen, then the footer.

- **Sidebar** (left): the file tree, or the change list on the changes screen. Width `min(40, max(28, 25% of the terminal))`; at least 24 columns, and the reader keeps at least 28. A width the user sets is kept for the session. From 60 to 79 columns it hides while a document is open: the `›` at the reader's title shows it and the `‹` at the list's title hides it. Below 60 columns the list is a page of its own. At 80 columns and wider `Ctrl+/` or the mark on its border shows it (`docs/parity.md`, Layout).
- **Reader** (right): the file, the rendered Markdown, the image, or the stacked diff.
- **Panels**: rounded borders in every tier; the focused panel border is accent, the other is `border`.
- **Header**: two rows at the top of each panel, with a hairline under it, in every tier.
- **Footer**: two rows from 60 columns up. Row one: a pill, the browsed root and its git branch (`⎇ name`) or a message, and facts on the right separated by ` · `. Row two: key hints, dropped from the end to fit, with `^q quit` kept last. Below 60 columns it is one row of words (see Tiers).

### Tiers

The width picks one of three layouts. The sketches are excerpts from live runs on the demo repository.

- **Compact, below 60 columns**: two pages in the medium tier's frame. The file list fills the screen; a file opens the reader full screen. The header's `‹` goes back, and so does a tap on the title beside it (`q` and `Esc` do too). The bottom bar is one row of equal words: `Find`, `Changes`, `Keys` on the list; `Back`, `Search`, `Keys` on the reader. A notice or an open prompt takes that row.

```text
╭──────────────────────────────────────╮
│ demo-fmc                       files │
│──────────────────────────────────────│
│▎▸  assets/                         •│
│ ▸  docs/                           •│
│    README.md                       M│
│                                      │
╰──────────────────────────────────────╯
    Find     ·   Changes   ·    Keys
```

```text
╭──────────────────────────────────────╮
│ ‹  README.md      read-only rendered │
│──────────────────────────────────────│
│  Marauder's Map                     ▐│
│                                     ▐│
│  A small ledger and symbology       ▐│
╰──────────────────────────────────────╯
    Back     ·   Search    ·    Keys
```

- **Medium, 60 to 79 columns**: the list beside the reader, with the two-row header and footer. The list hides while a document is open; `‹` at the start of the list's title hides it, and `›` at the start of the reader's title shows it. The panels keep their rounded borders, and the `«` and `»` marks are not drawn.

```text
╭──────────────────────────╮╭────────────────────────────────────────╮
│ ‹  demo-fmc        files ││ README.md           read-only rendered │
│──────────────────────────││────────────────────────────────────────│
│ ▸  assets/             •││  Marauder's Map                       ▐│
  FILES  …c-ref/demo-fmc/README.md  78 lines · markdown · UTF-8 · Top
 q close   ? help   / find   d changes   Enter open   ^q quit
```

- **Wide, 80 columns and wider**: Scouter's layout, with the `«` and `»` marks on the sidebar's border (`docs/parity.md`, Layout).

## Components

### Tree row

`▎` on the selected row, depth guides `│ `, the folder chevron `▾` or `▸`, the icon, the name (folders end with `/`), and on the right a bold git letter or a `•` for a folder with changes. The selected row has the `row-active-bg` surface and an accent, bold name. Ignored files use `ignored`. A marked row (`s`) shows `✓` in `accent-soft` in the bar's cell, in place of `▎`, and keeps its name; a marked row that is also selected keeps the surface. Scouter has no marks: the mark is fmc's own (plan-00014).

The footer's facts start with `N marked` in `accent-soft` while marks exist, and the compact bottom bar starts its Files page with the same text (plan-00014, D8).

### Panel header

The sidebar shows the folder name with a quiet `files` badge (`no dotfiles` while dotfiles are hidden), or `Changes` with a count. From 60 to 79 columns a chevron leads a header's title: `›` on the reader's header while the list is hidden, and `‹` on the list's header while the list shows and can hide. A tap on the chevron, its gap, or the title beside it does the same. Each is plain accent text. Below 60 columns a `‹` leads the header's title on every page but Files; a tap on it or on the title goes back, and the bottom bar is one row of words. The reader shows a dim parent path and a bold file name, then badges: quiet (`read-only`, `rendered`, `image`, `binary`, `all files`, `one file`) or pills (`first 1 MiB`, `wrap`, `split`, `wide context`, `preview`). An error replaces the badges in `error`, bold, until the next key.

### Reader sign strip

One column left of the line numbers in the text view, inside a git repository. `▎` in `added` marks an added line and `▎` in `modified` a changed one. `▁` in `removed` sits on the line above a deletion, and `▔` on line 1 when the deletion is above it. The column stays blank while nothing changed, so the numbers do not move. Rendered Markdown, images, and the changes screen have none. Scouter has no strip (`docs/parity.md`).

### Rendered search marks

In the rendered Markdown view, `/` marks each occurrence of the query in the text the view draws. Every match takes `match-fg` on `match-bg`, as the code view's match does. The current match takes the text colour on `target-bg`, so `n` and `N` show which one they reached. A fenced block in a highlighted language is marked the same way; a block of another language is counted and not marked. Scouter has no marks here (it refuses the search), so this is a recorded deviation (`docs/parity.md`).

In the rendered view, the same signs sit at the left edge beside each changed block: a bar in `added` or `modified` for the block's rows (the stronger one when a block has both), and `▁` in `removed` under the block above a deletion. The text does not move. The sign column is over the padding, so a block with a change keeps its width. Scouter has no signs here; this is a recorded deviation (`docs/parity.md`).

### Footer pills and hints

Pills name the context: `FILES`, `READER`, `CHANGES`, `DIFF`, `FIND`, `TEXT`, `SEARCH`. A pill is `accent-ink` on `accent`, bold. Hints show the key in `accent-soft` and the action in `dim`. Position reads `All`, `Top`, `Bot`, or a percentage.

### Mermaid diagram

A `mermaid` fence in the rendered view is drawn on `fallback-background`, across the reader's text width, the same span a code block takes, with the diagram at its left. Box lines take `border`, labels `text`, and a fallback's reason `muted`; a fallback's framed source sits on the same fill. Scouter draws the fence as plain text, so this is an approved deviation (`architecture.md`, "a Mermaid fence in rendered Markdown is drawn as a diagram").

### Diff file band

One band per changed file on the tinted background: the marker, a fold chevron, the bold coloured letter, the bold name (`accent-soft` when active), `← old` for a rename, the dim folder, and on the right the counts and a five-cell stat bar of `▪` (green for added, red for removed, at least one cell each when both exist). Hunk header lines are not drawn.

### Key list

`?` shows the key list in the reader, in place of the document, as tables: Browse, Reader, Changes, Everywhere, and Mouse. `?` or `Esc` brings the document back, with its scroll position and search.

### Context menu

A rounded border in `accent` on the `tint` background, over the page at the pointer or at the selected row (`Space`, or a right-click on a row). Each row is one line, padded two columns on each side: the label in `text` and the row's key at the right edge in `muted`. The selected row fills with `row-active-bg` and its label is bold; it has no `▎` marker (the owner, 2026-11-27). Delete's label is in `error`. A disabled row (Paste with nothing copied) is `muted`, its key `dim`, and the selection skips it. A rule in `border` separates the groups: creating (New file, New folder), changing an entry (Rename to Duplicate), and Delete. The box is the longest label, a three-column gap, and the longest key, plus six columns wide, and the rows and rules plus two tall. It sits at its anchor and flips above it, or moves left, to stay on the screen. In the compact tier it is a bottom sheet across the full width, above the bottom bar. Scouter has no menu (`architecture.md`, "The context menu and the dialog are fmc views").

A menu longer than its window scrolls. The window shows up to sixteen rows, or fewer on a short screen, and moves with the selection, so the Open With picker fits any terminal. The menu's look does not change while it scrolls.

### Dialog

A rounded border in `accent` on the `tint` background, centred and at most 56 columns wide. It has a bold title in `text`, a wrapped message, an optional one-line field (the terminal cursor shows in it while the field has the focus), an error line in `error`, and buttons on the right. The focused button is filled with `accent` and its label is `accent-ink` in bold. A pointer over a button tints its label in `accent` with no fill, and does not move the focus, so Return still picks the focused button. In the compact tier the dialog spans the width less a column on each side. A scrim dims the page under it with a colour gain of 0.4, and the page takes no clicks while a dialog or the Settings page is open.

### Settings page

A post-parity addition: Scouter has no settings page (`architecture.md`, "the Settings page edits settings.json"). It fills the content area as the changes screen does, and in the compact tier it is a page of the stack. The rows sit in two groups, Browsing and Appearance, each under an uppercase heading in bold `dim`, with a blank row above the page and between the groups. One row per setting, indented four columns: its name (`Auto preview`) in `muted` in an 18-column label column, then the value. An on-or-off value is `● on` in `accent` or `○ off` in `muted`; a number is in `accent-soft`; a choice is in `accent-soft` between `‹ ›`, which are `dim`, or `muted` on the selected row. The selected row fills with `row-active-bg` and its name is bold in `text`. Under the rows, after a blank row: `settings.json` and the selected row's key in `dim`, then its help line in `muted`. There are no boxes or Save button: Enter or Space changes a row, and the change saves at once.

### Welcome

A three-row block wordmark in the three `wordmark` colours, a tagline with the version, a rounded hint card, and a line that says there is no editing. Below 80 columns the closing line breaks at its ` · `. From 60 to 74 columns the card goes, since the reader beside the list is narrower than it; the wordmark, tagline, and prompt stay. Below 60 columns there is no welcome screen: the list is the first page.

## Do's and Don'ts

### Do

- Match Scouter's screen first; describe it here second.
- Use the palette tokens by name.

### Don't

- Add colour for decoration.
- Use emoji for icons.
- Let this file overrule Scouter during the parity phase.

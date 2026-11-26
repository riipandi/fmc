# Configuration

fmc reads one optional file when it starts, and never writes it.

## Location

`$XDG_CONFIG_HOME/fmc/settings.json`, or `~/.config/fmc/settings.json` when `XDG_CONFIG_HOME` is unset or empty.

## Keys

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `auto_preview` | `true` or `false` | `true` | Whether moving through the file list shows each file in the reader. |
| `sidebar_jump` | whole number, 1 or more | `2` | How many rows `[` and `]` move the file list. |
| `reader_jump` | whole number, 1 or more | `5` | How many lines `[` and `]` scroll the reader. |

A key that is not listed is ignored.

## Errors

A file that is not valid JSON, or whose top level is not an object, sets nothing: every key keeps its default. A key with the wrong type, or a jump below 1, keeps its own default. fmc shows the problems in the footer once, when it starts. A missing file is the defaults with no message.

## Changes

fmc reads the file only at start. Edit it by hand, then start fmc again.

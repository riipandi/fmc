//! Footer pills and key hints per context (Scouter `src/ui/footer-hints.ts`).

use super::styled::{Measure, Seg};
use fmc::theme::Palette;

pub const HINT_GAP: &str = "   ";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Browse,
    Diff,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pane {
    Files,
    Reader,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prompt {
    Files,
    Text,
    Reader,
    Command,
}

pub type Hint = (&'static str, &'static str);

pub fn prompt_label(prompt: Prompt) -> &'static str {
    match prompt {
        Prompt::Files => "FIND",
        Prompt::Text => "TEXT",
        Prompt::Reader => "SEARCH",
        Prompt::Command => "COMMAND",
    }
}

pub fn pill_label(mode: Mode, pane: Pane) -> &'static str {
    match (mode, pane) {
        (Mode::Browse, Pane::Files) => "FILES",
        (Mode::Browse, Pane::Reader) => "READER",
        (Mode::Diff, Pane::Files) => "CHANGES",
        (Mode::Diff, Pane::Reader) => "DIFF",
    }
}

pub fn prompt_hints(prompt: Prompt) -> Vec<Hint> {
    match prompt {
        Prompt::Files => {
            vec![("type", "filter"), ("↑ ↓", "move"), ("Enter", "open"), ("Tab", "text"), ("Esc", "cancel")]
        }
        Prompt::Text => {
            vec![("type", "search"), ("↑ ↓", "move"), ("Enter", "open"), ("Tab", "files"), ("Esc", "cancel")]
        }
        Prompt::Reader => vec![("type", "search"), ("Enter", "next"), ("↑ ↓", "prev / next"), ("Esc", "close")],
        Prompt::Command => vec![("type", "command"), ("Enter", "run"), ("Esc", "cancel")],
    }
}

/// What the reader shows, as far as the hints care.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Shown {
    /// `Some(rendered)` for a Markdown document, `None` for anything else.
    pub markdown: Option<bool>,
    pub image: bool,
    /// Something is open, so `q` closes it; the welcome screen has nothing to close.
    pub open: bool,
    /// The sidebar is hidden: `Tab` cannot reach it, and `Ctrl+/` shows it (plan-00002, D8).
    pub sidebar_hidden: bool,
}

/// The pane-switch hint: `Tab` moves to the sidebar while it shows. While it is hidden, `Tab` cannot reach it, so the
/// slot names the key that shows it. The slot sits early in the list, where it fits at 80 columns.
fn switch_hint(shown: Shown) -> Hint {
    if shown.sidebar_hidden { ("^/", "sidebar") } else { ("Tab", "files") }
}

pub fn hints_for(mode: Mode, pane: Pane, wrap: bool, all_files: bool, shown: Shown) -> Vec<Hint> {
    let pan: &[Hint] = if wrap { &[] } else { &[("h l", "pan")] };
    if mode == Mode::Diff {
        let other = ("a", if all_files { "one file" } else { "all files" });
        return if pane == Pane::Files {
            vec![
                ("j k", "move"),
                ("Enter", "view"),
                ("o", "open file"),
                ("Tab", "diff"),
                ("s", "split"),
                other,
                ("d", "back"),
                ("^←→", "resize"),
                ("?", "help"),
                ("^q", "quit"),
            ]
        } else {
            // A hidden sidebar's key comes second: at a phone width the footer keeps only the first few hints.
            let mut hints = vec![("j k", "scroll")];
            if shown.sidebar_hidden {
                hints.push(switch_hint(shown));
            }
            hints.extend([
                ("n N", "hunk"),
                ("o", "open file"),
                ("s", "split"),
                other,
                ("c", "fold"),
                ("x", "context"),
                ("y", "copy"),
                ("w", "wrap"),
            ]);
            if !shown.sidebar_hidden {
                hints.push(switch_hint(shown));
            }
            hints.extend_from_slice(pan);
            hints.extend([("gg G", "top/bottom"), ("^d ^u", "page"), ("d", "back"), ("?", "help"), ("^q", "quit")]);
            hints
        };
    }
    if shown.image && pane != Pane::Files {
        return vec![switch_hint(shown), ("d", "changes"), ("^q", "quit")];
    }
    if pane == Pane::Files {
        return vec![
            ("j k", "move"),
            ("Enter", "open"),
            ("/", "find"),
            ("-", "up"),
            ("h", "back"),
            ("Tab", "reader"),
            ("q", "close"),
            ("d", "changes"),
            ("w", "wrap"),
            ("^←→", "resize"),
            ("^o alt+i", "jump"),
            ("?", "help"),
            ("^q", "quit"),
        ]
        .into_iter()
        .filter(|(key, _)| *key != "q" || shown.open)
        .collect();
    }
    let rendered = shown.markdown == Some(true);
    let mut hints = vec![("j k", "scroll")];
    if shown.sidebar_hidden {
        hints.push(switch_hint(shown));
    }
    if !rendered {
        hints.extend_from_slice(pan);
    }
    hints.push(("gg G", "top/bottom"));
    if !rendered {
        hints.push(("{ }", "paragraph"));
    }
    hints.extend([("z z", "centre"), ("^d ^u", "page")]);
    if !shown.sidebar_hidden {
        hints.push(switch_hint(shown));
    }
    if let Some(rendered) = shown.markdown {
        hints.push(("m", if rendered { "source" } else { "preview" }));
    }
    if !rendered {
        hints.push(("/", "search"));
    }
    hints.extend([("y", "copy"), ("d", "changes"), ("^o alt+i", "jump")]);
    if !rendered {
        hints.push(("w", "wrap"));
    }
    if shown.open {
        hints.push(("q", "close"));
    }
    hints.extend([("?", "help"), ("^q", "quit")]);
    hints
}

/// The footer's hint row as text: each hint as `key action`, with the gap between hints.
pub fn row_text(hints: &[Hint]) -> String {
    hints.iter().map(|(key, action)| format!("{key} {action}")).collect::<Vec<_>>().join(HINT_GAP)
}

/// The hints that fit in `columns`, in order: the first that does not fit ends the row.
pub fn fit_hints(hints: &[Hint], columns: usize, measure: Measure<'_>) -> Vec<Hint> {
    let mut fitted: Vec<Hint> = Vec::new();
    for hint in hints {
        fitted.push(*hint);
        if measure(&row_text(&fitted)) > columns {
            fitted.pop();
            break;
        }
    }
    fitted
}

/// The segments of a hint row: each key in `accent-soft`, its action in `dim`, and the gap between hints.
pub fn hint_segments(hints: &[Hint], palette: &Palette) -> Vec<Seg> {
    let mut segments = Vec::new();
    for (i, (key, action)) in hints.iter().enumerate() {
        if i > 0 {
            segments.push(Seg::plain(HINT_GAP));
        }
        segments.push(Seg::new(*key, palette.accent_soft));
        segments.push(Seg::new(format!(" {action}"), palette.dim));
    }
    segments
}

/// Below 80 columns a short row keeps these hints first (plan-00004, P6): the one that leaves the current state, then
/// help, find, and the pane switch. A sort is stable, so the other hints keep their order after them.
pub fn phone_rank(hint: &Hint) -> u8 {
    match *hint {
        ("y", "Copy") => 0,
        ("Esc", _) => 1,
        ("^/", _) => 2,
        ("q", _) => 3,
        ("?", _) => 4,
        ("/", _) => 5,
        ("d", _) => 6,
        ("Enter", _) => 7,
        ("Tab", _) => 8,
        _ => 9,
    }
}

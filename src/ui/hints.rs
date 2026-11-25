//! Footer pills and key hints per context (Scouter `src/ui/footer-hints.ts`).

use super::styled::{Measure, Seg, width};
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
#[cfg_attr(not(test), expect(dead_code, reason = "the reader, finder, and changes phases of plan-00001 use the rest"))]
pub enum Prompt {
    Files,
    Text,
    Reader,
}

pub type Hint = (&'static str, &'static str);

pub fn prompt_label(prompt: Prompt) -> &'static str {
    match prompt {
        Prompt::Files => "FIND",
        Prompt::Text => "TEXT",
        Prompt::Reader => "SEARCH",
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
    }
}

/// What the reader shows, as far as the hints care.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Shown {
    /// `Some(rendered)` for a Markdown document, `None` for anything else.
    pub markdown: Option<bool>,
    pub image: bool,
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
            let mut hints = vec![
                ("j k", "scroll"),
                ("n N", "hunk"),
                ("o", "open file"),
                ("s", "split"),
                other,
                ("Tab", "files"),
                ("c", "fold"),
                ("x", "context"),
                ("y", "copy"),
                ("w", "wrap"),
            ];
            hints.extend_from_slice(pan);
            hints.extend([("gg G", "top/bottom"), ("^d ^u", "page"), ("d", "back"), ("?", "help"), ("^q", "quit")]);
            hints
        };
    }
    if shown.image && pane != Pane::Files {
        return vec![("Tab", "files"), ("d", "changes"), ("^q", "quit")];
    }
    if pane == Pane::Files {
        return vec![
            ("j k", "move"),
            ("Enter", "open"),
            ("/", "find"),
            ("-", "up"),
            ("h", "back"),
            ("Tab", "reader"),
            ("d", "changes"),
            ("w", "wrap"),
            ("^←→", "resize"),
            ("^o alt+i", "jump"),
            ("?", "help"),
            ("^q", "quit"),
        ];
    }
    let rendered = shown.markdown == Some(true);
    let mut hints = vec![("j k", "scroll")];
    if !rendered {
        hints.extend_from_slice(pan);
    }
    hints.push(("gg G", "top/bottom"));
    if !rendered {
        hints.push(("{ }", "paragraph"));
    }
    hints.extend([("z z", "centre"), ("^d ^u", "page"), ("Tab", "files")]);
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
    hints.extend([("?", "help"), ("^q", "quit")]);
    hints
}

/// The hints that fit in `columns`, in order, each a key in `accent-soft` and an action in `dim`.
pub fn fit_hint_list(hints: &[Hint], columns: usize, palette: &Palette, measure: Measure<'_>) -> Vec<Seg> {
    let mut segments: Vec<Seg> = Vec::new();
    for (key, action) in hints {
        let hint = [Seg::new(*key, palette.accent_soft), Seg::new(format!(" {action}"), palette.dim)];
        let mut next = segments.clone();
        if !next.is_empty() {
            next.push(Seg::plain(HINT_GAP));
        }
        next.extend(hint);
        if width(&next, measure) > columns {
            break;
        }
        segments = next;
    }
    segments
}

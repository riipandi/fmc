//! Keys: the parser that turns presses into actions, and the key list shown by `--help` and the `?` key page (Scouter
//! `src/core/keymap.ts`). Tests check that every key the list documents is one the parser binds.

mod parser;

pub use parser::{Focus, Parser, Screen, State, Step};

/// One key press, named as OpenTUI Core names it: `name` is the lowercase key (`g`, `return`, `pagedown`), `text`
/// the typed text (`G`, `?`, `{`), and Shift is set for capitals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key<'a> {
    pub name: &'a str,
    pub text: &'a str,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Key<'_> {
    /// No Ctrl and no Alt (Scouter `isPlain`); Shift may be set.
    pub fn plain(&self) -> bool {
        !self.ctrl && !self.alt
    }
}

/// What a key asks the app to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Quit,
    /// `?`: shows the key page, or puts the document back when it is shown.
    Help,
    /// `q`: closes the open document or the key page (architecture.md, approved deviation).
    Close,
    /// `Ctrl+/` (and `Ctrl+_`, the legacy `0x1F`): shows or hides the sidebar (plan-00002).
    ToggleSidebar,
    /// `Ctrl+.` and `.`: shows or hides dotfiles in the tree and the finder (plan-00002).
    ToggleDotfiles,
    SwitchPane,
    Escape,
    /// Narrower (-1) or wider (1) list, by two columns.
    Resize(i8),
    JumpBack,
    JumpForward,
    ToggleWrap,
    ToggleMarkdown,
    ToggleChanges,
    Copy,
    /// Move the list selection by rows.
    Move(i32),
    /// Move the list selection by a screen, down (1) or up (-1).
    MovePage(i8),
    Top,
    Bottom,
    /// Enter on a tree row: toggle a folder, open a file.
    Toggle,
    /// `l` or Right on a tree row: expand a folder or step into it; a file takes no key.
    Expand,
    Collapse,
    GoUp,
    Find,
    FocusDiff,
    Scroll(i32),
    Pan(i32),
    /// 0.9 of a page, down (1) or up (-1).
    Page(i8),
    HalfPage(i8),
    GoToLine(u32),
    Paragraph(i8),
    Centre,
    Search,
    NextMatch,
    PreviousMatch,
    ToggleSplit,
    ToggleScope,
    ToggleFold,
    ToggleContext,
    Reload,
    OpenChanged,
    Hunk(i8),
}

/// One help row: the keys, then what they do.
pub type KeyRow = (&'static str, &'static str);

pub const KEY_SECTIONS: &[(&str, &[KeyRow])] = &[
    (
        "Browse",
        &[
            ("j k ↑ ↓", "move"),
            ("Enter", "open file, toggle folder"),
            ("l →", "expand a folder"),
            ("h ← Backspace", "collapse a folder or go back"),
            ("/", "find a file by name; Tab: text in files"),
            ("-", "open the folder above"),
            ("gg G", "top / bottom"),
            ("Ctrl+← →", "resize the list, or drag its edge"),
        ],
    ),
    (
        "Reader",
        &[
            ("j k", "scroll"),
            ("/", "search the file; Enter next, Esc keeps it for n N"),
            ("n N p", "next / previous match"),
            ("space b ^d ^u", "page / half a page"),
            ("gg G", "top / bottom"),
            ("120G", "go to line 120"),
            ("{ }", "previous / next paragraph"),
            ("zz", "centre the match, or the middle line"),
            ("w", "wrap long lines"),
            ("m", "Markdown: rendered / source"),
        ],
    ),
    (
        "Changes",
        &[
            ("d", "show / hide the changes screen"),
            ("n N p", "next / previous hunk"),
            ("a", "all files / one file"),
            ("s", "split / unified"),
            ("c", "collapse / expand file diff"),
            ("x", "more / normal context lines"),
            ("r", "reload"),
            ("o", "open the file at the current hunk"),
        ],
    ),
    (
        "Everywhere",
        &[
            ("Tab", "switch panes"),
            ("y", "copy selection, or the current path"),
            ("Ctrl+o Alt+i", "jump back / forward through opened places"),
            ("Esc", "clear selection or search, or back to the file list"),
            ("q", "close the open document or key list; back from the changes"),
            ("?", "this key list; ? or Esc closes it"),
            ("Ctrl+/", "show / hide the sidebar"),
            ("Ctrl+. .", "show / hide dotfiles in the tree and the finder"),
            ("^q ^c", "quit"),
        ],
    ),
];

pub const MOUSE_ROWS: &[KeyRow] = &[
    ("Left click", "select file, or toggle folder"),
    ("Click", "focus the pane under the pointer"),
    ("Drag text", "select; y copies it"),
    ("Wheel", "scroll the pane under the pointer"),
    ("Drag edge", "resize the list"),
    ("Click «»", "show / hide the sidebar, from its mark"),
];

/// The Keys page (`?`): a Markdown table per section, Mouse last, from the same rows as `--help`.
pub fn key_page_markdown() -> String {
    let sections = KEY_SECTIONS.iter().copied().chain([("Mouse", MOUSE_ROWS)]).map(|(title, rows)| {
        let rows: Vec<String> = rows.iter().map(|(keys, action)| format!("| `{keys}` | {action} |")).collect();
        format!("## {title}\n\n| Key | Action |\n| --- | --- |\n{}", rows.join("\n"))
    });
    sections.collect::<Vec<_>>().join("\n\n") + "\n"
}

/// Width of the key column, in characters.
pub const KEY_COLUMN: usize = 15;

/// Every section, then Mouse, as plain text (`--help`).
pub fn key_list_text() -> String {
    KEY_SECTIONS
        .iter()
        .copied()
        .chain([("Mouse", MOUSE_ROWS)])
        .map(|(title, rows)| {
            let rows = rows.iter().map(|(keys, action)| format!("  {keys:<KEY_COLUMN$}{action}"));
            std::iter::once(title.to_string()).chain(rows).collect::<Vec<_>>().join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests;

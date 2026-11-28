//! Keys: the parser that turns presses into actions, and the key list shown by `--help` and the `?` key page (Scouter
//! `src/core/keymap.ts`). Tests check that every key the list documents is one the parser binds.

mod bindings;
pub mod keymap;
mod parser;

pub use keymap::{Keymap, Parsed};
pub use parser::{Focus, Parser, Screen, State, Step};

use bindings::{Context, display_sequence};

/// One key press, named as OpenTUI Core names it: `name` is the lowercase key (`g`, `return`, `pagedown`), `text`
/// the typed text (`G`, `?`, `{`), and Shift is set for capitals. `cmd` is the Super key (Cmd on a Mac), which a
/// terminal sends only with the kitty keyboard protocol or when it passes Cmd through.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key<'a> {
    pub name: &'a str,
    pub text: &'a str,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub cmd: bool,
}

impl Key<'_> {
    /// No Ctrl and no Alt (Scouter `isPlain`); Shift may be set. Cmd does not count, so a Cmd key takes the path of
    /// the same key without it unless a rule names `cmd`.
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
    /// `q`: closes the open document or the key page.
    Close,
    /// `Ctrl+/` (and `Ctrl+_`, the legacy `0x1F`): shows or hides the sidebar.
    ToggleSidebar,
    /// `Ctrl+.` and `.`: shows or hides dotfiles in the tree and the finder.
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
    /// `p` on the tree: turns auto-preview on or off for this session.
    TogglePreview,
    /// `]` (+1) and `[` (-1) while browsing: the list moves, or the reader scrolls, by the settings' jump.
    Jump(i8),
    /// `:` outside a prompt: opens the one-line command prompt in the footer (`:q` quits).
    OpenCommand,
    /// `Ctrl+,`: opens the Settings page. Only the kitty keyboard protocol sends it.
    OpenSettings,
    Find,
    /// `Space` on a tree row: the context menu for the row.
    ContextMenu,
    /// `a` on a tree row: a dialog for a new file, or a folder when the name ends in `/`.
    NewFile,
    /// `A` on a tree row: a dialog for a new folder.
    NewFolder,
    /// `R` on a tree row: a name dialog for a copy of the row in its folder.
    Duplicate,
    /// `r` on a tree row: a dialog that renames the row.
    Rename,
    /// `D` and Delete on a tree row: a confirm dialog that moves the row to the Trash.
    Delete,
    /// `c` on a tree row: the row goes on fmc's clipboard to copy.
    CopyFile,
    /// `x` on a tree row: the row goes on the clipboard to move, and is dimmed until a paste or Esc.
    CutFile,
    /// `v` in the tree: the clipboard goes into the selected folder, or the folder of the selected file.
    PasteFile,
    /// `s` in the tree: marks the selected row and moves the selection down one row.
    ToggleMark,
    /// `S` in the tree: clears every mark.
    ClearMarks,
    /// `o` on a tree row or in the reader: opens the row, or the shown file, with the default app.
    OpenDefault,
    /// `O` in the tree or the reader: lists the apps for the row or the shown file, and picks one.
    OpenWith,
    /// `e` in the tree or the reader: opens the browsed folder in the settings' editor, or in Finder.
    OpenInEditor,
    /// `E` in the tree or the reader: reveals the row, or the shown file, in Finder (`open -R`).
    RevealInFinder,
    /// `Ctrl+F` while a finder is open: flips the name matching between substring and fuzzy.
    ToggleFuzzy,
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

/// One help row: the keys, then what they do. The Keys page shows the action first.
pub type KeyRow = (&'static str, &'static str);

/// The sections, each in groups of related acts: moving, then opening, then finding and viewing; in Files, the menu,
/// creating, changing, the clipboard, deleting, then the marks.
pub const KEY_SECTIONS: &[(&str, &[KeyRow])] = &[
    (
        "Browse",
        &[
            ("j k ↑ ↓", "move"),
            ("gg G", "top / bottom"),
            ("[ ]", "jump the list or reader"),
            ("Enter", "open file, toggle folder"),
            ("l →", "expand a folder"),
            ("h ← Backspace", "collapse a folder or go back"),
            ("-", "open the folder above"),
            ("/", "find a file by name; Tab: text in files; Ctrl+F: fuzzy on or off"),
            ("p", "auto-preview on or off"),
            ("Ctrl+← →", "resize the list, or drag its edge"),
        ],
    ),
    (
        "Files",
        &[
            ("space", "context menu for the row"),
            ("a", "new file; a name ending in / makes a folder"),
            ("A", "new folder"),
            ("r", "rename"),
            ("R", "duplicate the row under a new name"),
            ("c", "copy the marked rows, or the row"),
            ("x", "cut the marked rows, or the row; a paste moves them"),
            ("v", "paste into the folder, or beside the file"),
            ("D Delete", "move the marked rows, or the row, to the Trash"),
            ("s", "mark the row and move down"),
            ("S", "clear the marks"),
            ("o", "open in the default app; a folder opens in Finder"),
            ("e", "open the folder in the editor from settings, or in Finder"),
            ("E", "show the row in Finder"),
            ("O", "pick an app to open the row with"),
        ],
    ),
    (
        "Reader",
        &[
            ("j k", "scroll"),
            ("space b ^d ^u", "page / half a page"),
            ("gg G", "top / bottom"),
            ("120G", "go to line 120"),
            ("{ }", "previous / next paragraph"),
            ("/", "search the file; Enter next, Esc keeps it for n N"),
            ("n N p", "next / previous match"),
            ("zz", "centre the match, or the middle line"),
            ("w", "wrap long lines"),
            ("m", "Markdown: rendered / source"),
            ("o", "open the file in its default app"),
            ("e", "open the folder in the editor from settings, or in Finder"),
            ("E", "show the file in Finder"),
            ("O", "pick an app to open the file with"),
        ],
    ),
    (
        "Changes",
        &[
            ("d", "show / hide the changes screen"),
            ("n N p", "next / previous hunk"),
            ("o", "open the file at the current hunk"),
            ("a", "all files / one file"),
            ("s", "split / unified"),
            ("c", "collapse / expand file diff"),
            ("x", "more / normal context lines"),
            ("r", "reload"),
        ],
    ),
    (
        "Everywhere",
        &[
            ("Tab", "switch panes"),
            ("Ctrl+o Alt+i", "jump back / forward through opened places"),
            ("y", "copy selection, or the current path"),
            (
                "Esc",
                "clear selection or search, or back to the file list; in the list, drop a copy or cut, then the marks",
            ),
            ("q", "close the open document or key list; back from the changes"),
            ("?", "this key list; ? or Esc closes it"),
            ("Ctrl+/", "show / hide the sidebar"),
            ("Ctrl+. .", "show / hide dotfiles in the tree and the finder"),
            ("Ctrl+,", "the Settings page; each change saves to settings.json"),
            (": q", "quit"),
            ("^q ^c", "quit"),
        ],
    ),
];

pub const MOUSE_ROWS: &[KeyRow] = &[
    ("Left click", "select file, or toggle folder"),
    ("Click", "focus the pane under the pointer"),
    ("Wheel", "scroll the pane under the pointer"),
    ("Drag text", "select; y copies it"),
    ("Right click", "context menu for the row"),
    ("Hover", "select a context-menu row; highlight a dialog button"),
    ("Drag edge", "resize the list"),
    ("Click «»", "show / hide the sidebar, from its mark"),
];

/// The actions each card row shows, by section and label: a row lists the keys its actions have in the merged
/// table, so a rebound key shows on its row. A row with no actions is fixed text.
const ROW_ACTIONS: &[(&str, &str, &[Action])] = &[
    ("Browse", "j k ↑ ↓", &[Action::Move(1), Action::Move(-1)]),
    ("Browse", "Enter", &[Action::Toggle]),
    ("Browse", "l →", &[Action::Expand]),
    ("Browse", "h ← Backspace", &[Action::Collapse]),
    ("Browse", "/", &[Action::Find]),
    ("Browse", "-", &[Action::GoUp]),
    ("Browse", "p", &[Action::TogglePreview]),
    ("Browse", "[ ]", &[Action::Jump(-1), Action::Jump(1)]),
    ("Browse", "gg G", &[Action::Top, Action::Bottom]),
    ("Browse", "Ctrl+← →", &[Action::Resize(-1), Action::Resize(1)]),
    ("Files", "space", &[Action::ContextMenu]),
    ("Files", "a", &[Action::NewFile]),
    ("Files", "A", &[Action::NewFolder]),
    ("Files", "R", &[Action::Duplicate]),
    ("Files", "r", &[Action::Rename]),
    ("Files", "D Delete", &[Action::Delete]),
    ("Files", "c", &[Action::CopyFile]),
    ("Files", "x", &[Action::CutFile]),
    ("Files", "v", &[Action::PasteFile]),
    ("Files", "s", &[Action::ToggleMark]),
    ("Files", "S", &[Action::ClearMarks]),
    ("Files", "o", &[Action::OpenDefault]),
    ("Files", "e", &[Action::OpenInEditor]),
    ("Files", "E", &[Action::RevealInFinder]),
    ("Files", "O", &[Action::OpenWith]),
    ("Reader", "j k", &[Action::Scroll(1), Action::Scroll(-1)]),
    ("Reader", "/", &[Action::Search]),
    ("Reader", "n N p", &[Action::NextMatch, Action::PreviousMatch]),
    ("Reader", "space b ^d ^u", &[Action::Page(1), Action::Page(-1), Action::HalfPage(1), Action::HalfPage(-1)]),
    ("Reader", "gg G", &[Action::Top, Action::Bottom]),
    ("Reader", "{ }", &[Action::Paragraph(-1), Action::Paragraph(1)]),
    ("Reader", "w", &[Action::ToggleWrap]),
    ("Reader", "m", &[Action::ToggleMarkdown]),
    ("Reader", "o", &[Action::OpenDefault]),
    ("Reader", "e", &[Action::OpenInEditor]),
    ("Reader", "E", &[Action::RevealInFinder]),
    ("Reader", "O", &[Action::OpenWith]),
    ("Changes", "d", &[Action::ToggleChanges]),
    ("Changes", "n N p", &[Action::Hunk(1), Action::Hunk(-1)]),
    ("Changes", "a", &[Action::ToggleScope]),
    ("Changes", "s", &[Action::ToggleSplit]),
    ("Changes", "c", &[Action::ToggleFold]),
    ("Changes", "x", &[Action::ToggleContext]),
    ("Changes", "r", &[Action::Reload]),
    ("Changes", "o", &[Action::OpenChanged]),
    ("Everywhere", "Tab", &[Action::SwitchPane]),
    ("Everywhere", "y", &[Action::Copy]),
    ("Everywhere", "Ctrl+o Alt+i", &[Action::JumpBack, Action::JumpForward]),
    ("Everywhere", "Esc", &[Action::Escape]),
    ("Everywhere", "q", &[Action::Close]),
    ("Everywhere", "?", &[Action::Help]),
    ("Everywhere", "Ctrl+/", &[Action::ToggleSidebar]),
    ("Everywhere", "Ctrl+. .", &[Action::ToggleDotfiles]),
    ("Everywhere", "Ctrl+,", &[Action::OpenSettings]),
    ("Everywhere", "^q ^c", &[Action::Quit]),
];

fn section_title(context: Context) -> &'static str {
    match context {
        Context::Everywhere => "Everywhere",
        Context::Browse => "Browse",
        Context::Reader => "Reader",
        Context::Changes | Context::ChangesList | Context::ChangesReader => "Changes",
    }
}

/// A row's keys: its fixed label while its actions keep their default keys, else the merged keys of its actions.
fn row_keys(title: &str, label: &str, keymap: &Keymap, defaults: &Keymap) -> String {
    let Some((_, _, actions)) = ROW_ACTIONS.iter().find(|(t, l, _)| *t == title && *l == label) else {
        return label.into();
    };
    if actions.is_empty() {
        return label.into();
    }
    let now = keymap.keys_for(actions);
    if now == defaults.keys_for(actions) {
        return label.into();
    }
    if now.is_empty() { "unbound".into() } else { now.join(" ") }
}

/// The card's sections, as `(title, (keys, action))` rows, from the merged keys. A binding of an
/// action with no row gets a row of its own, under the section of its context. Mouse comes last.
pub fn key_sections(keymap: &Keymap) -> Vec<(String, Vec<(String, String)>)> {
    let defaults = Keymap::defaults();
    let mut sections: Vec<(String, Vec<(String, String)>)> = KEY_SECTIONS
        .iter()
        .map(|(title, rows)| {
            let rows = rows
                .iter()
                .map(|(label, action)| (row_keys(title, label, keymap, &defaults), action.to_string()))
                .collect();
            (title.to_string(), rows)
        })
        .collect();
    let has_row = |action: Action| ROW_ACTIONS.iter().any(|(_, _, actions)| actions.contains(&action));
    for binding in keymap.bindings().iter().filter(|b| b.user && !has_row(b.action)) {
        let (Some(name), title) = (binding.action.name(), section_title(binding.context)) else { continue };
        if let Some((_, rows)) = sections.iter_mut().find(|(t, _)| t == title) {
            rows.push((display_sequence(&binding.keys), name.replace('_', " ")));
        }
    }
    sections.push((
        "Mouse".into(),
        MOUSE_ROWS.iter().map(|(keys, action)| (keys.to_string(), action.to_string())).collect(),
    ));
    sections
}

/// The Keys page (`?`): a Markdown table per section, Mouse last, from the merged keys.
pub fn key_page_markdown_with(keymap: &Keymap) -> String {
    let sections = key_sections(keymap).into_iter().map(|(title, rows)| {
        let rows: Vec<String> = rows.iter().map(|(keys, action)| format!("| {action} | `{keys}` |")).collect();
        format!("## {title}\n\n| Action | Key |\n| --- | --- |\n{}", rows.join("\n"))
    });
    sections.collect::<Vec<_>>().join("\n\n") + "\n"
}

/// The Keys page with the default keys.
pub fn key_page_markdown() -> String {
    key_page_markdown_with(&Keymap::defaults())
}

#[cfg(test)]
mod tests;

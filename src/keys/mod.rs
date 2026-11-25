//! The key list shown by `--help` and the `?` card (Scouter `src/core/keymap.ts`).

/// One help row: the keys, then what they do.
pub type KeyRow = (&'static str, &'static str);

pub const KEY_SECTIONS: &[(&str, &[KeyRow])] = &[
    (
        "Browse",
        &[
            ("j k ↑ ↓", "move"),
            ("l → Enter", "open a file or folder"),
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
            ("?", "this help (any key closes)"),
            ("q Ctrl+c", "quit"),
        ],
    ),
];

pub const MOUSE_ROWS: &[KeyRow] = &[
    ("Left click", "open a file or toggle a folder"),
    ("Drag text", "select and copy to clipboard"),
    ("Wheel", "scroll the pane under the pointer"),
    ("Drag edge", "resize the list"),
];

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

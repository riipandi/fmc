//! The default key table (plan-00015, A8): every key the parser binds is one row of (context, key, typing, action).
//! The parser walks it, the `?` card and `--help` read it, and `keymap.json` edits it (`keymap.rs`). Count digits and
//! `z` are not rows: the parser keeps them (`parser.rs`).

use super::Action::{self, *};
use super::Key;

/// Where a binding applies: the screen and focus the parser is in (`parser.rs` `active`). The names are the keymap
/// file's, so they are a public list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Context {
    Everywhere,
    /// The file tree (Browse screen, list focus).
    Browse,
    /// The reader of a file (Browse screen, reader focus).
    Reader,
    /// The changes screen, in either focus.
    Changes,
    /// The changes list (changes screen, list focus).
    ChangesList,
    /// The changes body (changes screen, reader focus).
    ChangesReader,
}

impl Context {
    pub fn name(self) -> &'static str {
        match self {
            Context::Everywhere => "everywhere",
            Context::Browse => "browse",
            Context::Reader => "reader",
            Context::Changes => "changes",
            Context::ChangesList => "changes_list",
            Context::ChangesReader => "changes_reader",
        }
    }

    pub fn from_name(name: &str) -> Option<Context> {
        use Context::*;
        [Everywhere, Browse, Reader, Changes, ChangesList, ChangesReader].into_iter().find(|c| c.name() == name)
    }
}

/// One key of a binding, as the parser matches it. `name` is OpenTUI Core's key name. A modifier set to `None` is not
/// checked: `ctrl-` and `alt-` bindings ignore Shift, and a symbol such as `?` ignores it because its text carries it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chord {
    name: String,
    ctrl: Option<bool>,
    alt: Option<bool>,
    shift: Option<bool>,
}

/// One key press, owned, so a sequence in progress can keep its keys.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Press {
    name: String,
    ctrl: bool,
    alt: bool,
    shift: bool,
}

impl Press {
    pub fn of(key: Key<'_>) -> Press {
        Press { name: key.name.to_string(), ctrl: key.ctrl, alt: key.alt, shift: key.shift }
    }
}

/// A binding's key: one or more chords. More than one is a sequence, such as `g g` in a keymap file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    pub context: Context,
    pub keys: Vec<Chord>,
    /// `Some(false)`: only while no prompt is open; `Some(true)`: only while one is. `None`: always.
    pub typing: Option<bool>,
    pub action: Action,
    /// Written by the keymap file (not a default row).
    pub user: bool,
}

impl Chord {
    /// A key from the keymap file or the table: `g`, `G` (Shift+g), `?`, `enter`, `down`, `shift-down`, `ctrl-q`,
    /// `alt-i`, `ctrl-alt-q`, or `*-tab` (any modifiers). Names are lowercase; `enter` is OpenTUI's `return`.
    pub fn parse(text: &str) -> Result<Chord, String> {
        let (mut rest, mut any, mut ctrl, mut alt, mut shift_prefix) = (text, false, false, false, false);
        loop {
            if let Some(tail) = rest.strip_prefix("*-") {
                (rest, any) = (tail, true);
            } else if let Some(tail) = rest.strip_prefix("ctrl-") {
                (rest, ctrl) = (tail, true);
            } else if let Some(tail) = rest.strip_prefix("alt-") {
                (rest, alt) = (tail, true);
            } else if let Some(tail) = rest.strip_prefix("shift-") {
                (rest, shift_prefix) = (tail, true);
            } else {
                break;
            }
        }
        let named = match rest {
            "enter" => Some("return"),
            "escape" | "space" | "tab" | "backspace" | "delete" | "home" | "end" | "pageup" | "pagedown" | "up"
            | "down" | "left" | "right" => Some(rest),
            _ => None,
        };
        let mut chars = rest.chars();
        let (name, letter, upper) = match (named, chars.next(), chars.next()) {
            (Some(name), _, _) => (name.to_string(), false, false),
            (None, Some(c), None) if c.is_ascii_graphic() => {
                (c.to_ascii_lowercase().to_string(), c.is_ascii_alphabetic(), c.is_ascii_uppercase())
            }
            _ => return Err(format!("\"{text}\" is not a key name")),
        };
        if shift_prefix && !(letter || named.is_some()) {
            return Err(format!("\"{text}\" puts shift on a symbol"));
        }
        let plain = !ctrl && !alt;
        let (ctrl, alt, shift) = if any {
            (None, None, None)
        } else if plain {
            // `.` is the one symbol whose arm checks Shift (Scouter's plain-key rule): a Shift+`.` is not dotfiles.
            let shift = if shift_prefix || upper {
                Some(true)
            } else if letter || rest == "." {
                Some(false)
            } else {
                None
            };
            (Some(false), Some(false), shift)
        } else {
            (Some(ctrl), Some(alt), None)
        };
        Ok(Chord { name, ctrl, alt, shift })
    }

    pub fn matches(&self, press: &Press) -> bool {
        let is = |want: Option<bool>, got: bool| want.is_none_or(|want| want == got);
        self.name == press.name && is(self.ctrl, press.ctrl) && is(self.alt, press.alt) && is(self.shift, press.shift)
    }

    /// A key whose Shift is not checked (a symbol, a named key, a `ctrl-` or `alt-` key); it sorts after a shifted rule.
    pub fn shift_unchecked(&self) -> bool {
        self.shift.is_none()
    }

    /// The key as the `?` card shows it: `Ctrl+q`, `Alt+i`, `Shift+↓`, `G`, `Enter`, `↑`.
    pub fn display(&self) -> String {
        let name = match self.name.as_str() {
            "return" => "Enter".to_string(),
            "escape" => "Esc".to_string(),
            "backspace" => "Backspace".to_string(),
            "delete" => "Delete".to_string(),
            "tab" => "Tab".to_string(),
            "pageup" => "PageUp".to_string(),
            "pagedown" => "PageDown".to_string(),
            "home" => "Home".to_string(),
            "end" => "End".to_string(),
            "up" => "↑".to_string(),
            "down" => "↓".to_string(),
            "left" => "←".to_string(),
            "right" => "→".to_string(),
            other => other.to_string(),
        };
        if self.ctrl == Some(true) {
            format!("Ctrl+{name}")
        } else if self.alt == Some(true) {
            format!("Alt+{name}")
        } else if self.shift == Some(true) && self.name.len() == 1 && self.name.chars().all(|c| c.is_ascii_alphabetic())
        {
            self.name.to_ascii_uppercase()
        } else if self.shift == Some(true) {
            format!("Shift+{name}")
        } else {
            name
        }
    }
}

/// A sequence's display: `gg` for plain characters, and the keys spaced when one of them has a modifier or a name.
pub fn display_sequence(keys: &[Chord]) -> String {
    let plain = keys.iter().all(|k| k.name.len() == 1 && k.ctrl != Some(true) && k.alt != Some(true));
    let parts: Vec<String> = keys.iter().map(Chord::display).collect();
    if plain { parts.concat() } else { parts.join(" ") }
}

impl Action {
    /// The name the keymap file uses for the action: the variant in snake_case. A parameter with its own row has its own
    /// name (`Move(1)` is `move_down`). `GoToLine` takes a count, not a key, so it has none.
    pub fn name(self) -> Option<&'static str> {
        ACTION_NAMES.iter().find(|(_, action)| *action == self).map(|(name, _)| *name)
    }

    pub fn from_name(name: &str) -> Option<Action> {
        ACTION_NAMES.iter().find(|(known, _)| *known == name).map(|(_, action)| *action)
    }
}

/// Every name the keymap file accepts, with its action.
pub const ACTION_NAMES: &[(&str, Action)] = &[
    ("quit", Quit),
    ("help", Help),
    ("close", Close),
    ("toggle_sidebar", ToggleSidebar),
    ("toggle_dotfiles", ToggleDotfiles),
    ("switch_pane", SwitchPane),
    ("escape", Escape),
    ("resize_narrower", Resize(-1)),
    ("resize_wider", Resize(1)),
    ("jump_back", JumpBack),
    ("jump_forward", JumpForward),
    ("toggle_wrap", ToggleWrap),
    ("toggle_markdown", ToggleMarkdown),
    ("toggle_changes", ToggleChanges),
    ("copy", Copy),
    ("move_down", Move(1)),
    ("move_up", Move(-1)),
    ("move_down_five", Move(5)),
    ("move_up_five", Move(-5)),
    ("move_page_down", MovePage(1)),
    ("move_page_up", MovePage(-1)),
    ("top", Top),
    ("bottom", Bottom),
    ("toggle", Toggle),
    ("expand", Expand),
    ("collapse", Collapse),
    ("go_up", GoUp),
    ("toggle_preview", TogglePreview),
    ("jump_next", Jump(1)),
    ("jump_previous", Jump(-1)),
    ("open_command", OpenCommand),
    ("open_settings", OpenSettings),
    ("find", Find),
    ("context_menu", ContextMenu),
    ("new_file", NewFile),
    ("new_folder", NewFolder),
    ("duplicate", Duplicate),
    ("rename", Rename),
    ("delete", Delete),
    ("copy_file", CopyFile),
    ("cut_file", CutFile),
    ("paste_file", PasteFile),
    ("toggle_mark", ToggleMark),
    ("clear_marks", ClearMarks),
    ("open_default", OpenDefault),
    ("open_in_editor", OpenInEditor),
    ("reveal_in_finder", RevealInFinder),
    ("open_with", OpenWith),
    ("toggle_fuzzy", ToggleFuzzy),
    ("focus_diff", FocusDiff),
    ("scroll_down", Scroll(1)),
    ("scroll_up", Scroll(-1)),
    ("pan_left", Pan(-4)),
    ("pan_right", Pan(4)),
    ("page_down", Page(1)),
    ("page_up", Page(-1)),
    ("half_page_down", HalfPage(1)),
    ("half_page_up", HalfPage(-1)),
    ("paragraph_previous", Paragraph(-1)),
    ("paragraph_next", Paragraph(1)),
    ("centre", Centre),
    ("search", Search),
    ("next_match", NextMatch),
    ("previous_match", PreviousMatch),
    ("toggle_split", ToggleSplit),
    ("toggle_scope", ToggleScope),
    ("toggle_fold", ToggleFold),
    ("toggle_context", ToggleContext),
    ("reload", Reload),
    ("open_changed", OpenChanged),
    ("next_hunk", Hunk(1)),
    ("previous_hunk", Hunk(-1)),
];

/// `None` means the key is always checked against the state; `Some(false)` is for a key a prompt keeps (`ctrl-o`), and
/// `Some(true)` for a key only a prompt takes (`ctrl-f`).
const ANY: Option<bool> = None;
const NO_PROMPT: Option<bool> = Some(false);
const PROMPT: Option<bool> = Some(true);

/// The default keys, in dispatch order: a row is tried before the rows after it, and a user binding comes before all of
/// them (`keymap.rs`). Every row copies a match arm of Scouter's keys, as `keys/tests.rs` pins them.
pub const DEFAULTS: &[(Context, &str, Option<bool>, Action)] = &[
    (Context::Everywhere, "ctrl-q", ANY, Quit),
    (Context::Everywhere, "ctrl-c", ANY, Quit),
    // Ctrl+Alt+q and Ctrl+Alt+c quit too: the quit arm checks Ctrl and ignores Alt.
    (Context::Everywhere, "ctrl-alt-q", ANY, Quit),
    (Context::Everywhere, "ctrl-alt-c", ANY, Quit),
    (Context::Everywhere, "*-tab", ANY, SwitchPane),
    (Context::Everywhere, "*-escape", ANY, Escape),
    (Context::Everywhere, "?", ANY, Help),
    (Context::Everywhere, ":", ANY, OpenCommand),
    (Context::Everywhere, "ctrl-left", ANY, Resize(-1)),
    (Context::Everywhere, "ctrl-right", ANY, Resize(1)),
    (Context::Everywhere, "ctrl-o", NO_PROMPT, JumpBack),
    (Context::Everywhere, "ctrl-/", NO_PROMPT, ToggleSidebar),
    (Context::Everywhere, "ctrl-_", NO_PROMPT, ToggleSidebar),
    (Context::Everywhere, "ctrl-f", PROMPT, ToggleFuzzy),
    (Context::Everywhere, "ctrl-.", ANY, ToggleDotfiles),
    (Context::Everywhere, "ctrl-,", NO_PROMPT, OpenSettings),
    (Context::Everywhere, "alt-i", NO_PROMPT, JumpForward),
    (Context::Everywhere, ".", NO_PROMPT, ToggleDotfiles),
    (Context::Everywhere, "w", ANY, ToggleWrap),
    (Context::Everywhere, "q", ANY, Close),
    (Context::Everywhere, "m", ANY, ToggleMarkdown),
    (Context::Everywhere, "d", ANY, ToggleChanges),
    (Context::Everywhere, "y", ANY, Copy),
    // The changes screen, in either focus. Rows with Shift are listed where the arm ignores it.
    (Context::Changes, "]", ANY, Hunk(1)),
    (Context::Changes, "[", ANY, Hunk(-1)),
    (Context::Changes, "n", ANY, Hunk(1)),
    (Context::Changes, "N", ANY, Hunk(-1)),
    (Context::Changes, "p", ANY, Hunk(-1)),
    (Context::Changes, "P", ANY, Hunk(-1)),
    (Context::Changes, "s", ANY, ToggleSplit),
    (Context::Changes, "S", ANY, ToggleSplit),
    (Context::Changes, "a", ANY, ToggleScope),
    (Context::Changes, "A", ANY, ToggleScope),
    (Context::Changes, "c", ANY, ToggleFold),
    (Context::Changes, "C", ANY, ToggleFold),
    (Context::Changes, "x", ANY, ToggleContext),
    (Context::Changes, "X", ANY, ToggleContext),
    (Context::Changes, "r", ANY, Reload),
    (Context::Changes, "R", ANY, Reload),
    (Context::Changes, "o", ANY, OpenChanged),
    (Context::Changes, "O", ANY, OpenChanged),
    // The file tree.
    (Context::Browse, "a", ANY, NewFile),
    (Context::Browse, "A", ANY, NewFolder),
    (Context::Browse, "R", ANY, Duplicate),
    (Context::Browse, "r", ANY, Rename),
    (Context::Browse, "D", ANY, Delete),
    (Context::Browse, "delete", ANY, Delete),
    (Context::Browse, "c", ANY, CopyFile),
    (Context::Browse, "x", ANY, CutFile),
    (Context::Browse, "v", ANY, PasteFile),
    (Context::Browse, "s", ANY, ToggleMark),
    (Context::Browse, "S", ANY, ClearMarks),
    (Context::Browse, "]", ANY, Jump(1)),
    (Context::Browse, "[", ANY, Jump(-1)),
    (Context::Browse, "enter", ANY, Toggle),
    (Context::Browse, "l", ANY, Expand),
    (Context::Browse, "L", ANY, Expand),
    (Context::Browse, "right", ANY, Expand),
    (Context::Browse, "h", ANY, Collapse),
    (Context::Browse, "H", ANY, Collapse),
    (Context::Browse, "left", ANY, Collapse),
    (Context::Browse, "backspace", ANY, Collapse),
    (Context::Browse, "-", ANY, GoUp),
    (Context::Browse, "p", ANY, TogglePreview),
    (Context::Browse, "P", ANY, TogglePreview),
    (Context::Browse, "/", ANY, Find),
    (Context::Browse, "space", ANY, ContextMenu),
    (Context::Browse, "G", ANY, Bottom),
    (Context::Browse, "g", ANY, Top),
    (Context::Browse, "home", ANY, Top),
    (Context::Browse, "end", ANY, Bottom),
    // A shifted key comes before its plain row: `shift-down` moves five, `down` one.
    (Context::Browse, "shift-down", ANY, Move(5)),
    (Context::Browse, "shift-up", ANY, Move(-5)),
    (Context::Browse, "J", ANY, Move(5)),
    (Context::Browse, "K", ANY, Move(-5)),
    (Context::Browse, "j", ANY, Move(1)),
    (Context::Browse, "down", ANY, Move(1)),
    (Context::Browse, "k", ANY, Move(-1)),
    (Context::Browse, "up", ANY, Move(-1)),
    (Context::Browse, "pagedown", ANY, MovePage(1)),
    (Context::Browse, "pageup", ANY, MovePage(-1)),
    // The reader of a file: its scroll keys, then the keys only a file the browser shows takes.
    (Context::Reader, "ctrl-d", ANY, HalfPage(1)),
    (Context::Reader, "ctrl-u", ANY, HalfPage(-1)),
    (Context::Reader, "H", ANY, Pan(-4)),
    (Context::Reader, "h", ANY, Pan(-4)),
    (Context::Reader, "left", ANY, Pan(-4)),
    (Context::Reader, "L", ANY, Pan(4)),
    (Context::Reader, "l", ANY, Pan(4)),
    (Context::Reader, "right", ANY, Pan(4)),
    (Context::Reader, "J", ANY, Scroll(1)),
    (Context::Reader, "j", ANY, Scroll(1)),
    (Context::Reader, "down", ANY, Scroll(1)),
    (Context::Reader, "K", ANY, Scroll(-1)),
    (Context::Reader, "k", ANY, Scroll(-1)),
    (Context::Reader, "up", ANY, Scroll(-1)),
    (Context::Reader, "space", ANY, Page(1)),
    (Context::Reader, "pagedown", ANY, Page(1)),
    (Context::Reader, "B", ANY, Page(-1)),
    (Context::Reader, "b", ANY, Page(-1)),
    (Context::Reader, "pageup", ANY, Page(-1)),
    (Context::Browse, "o", ANY, OpenDefault),
    (Context::Reader, "o", ANY, OpenDefault),
    (Context::Browse, "e", ANY, OpenInEditor),
    (Context::Reader, "e", ANY, OpenInEditor),
    (Context::Browse, "E", ANY, RevealInFinder),
    (Context::Reader, "E", ANY, RevealInFinder),
    (Context::Browse, "O", ANY, OpenWith),
    (Context::Reader, "O", ANY, OpenWith),
    (Context::Reader, "/", ANY, Search),
    (Context::Reader, "n", ANY, NextMatch),
    (Context::Reader, "N", ANY, PreviousMatch),
    (Context::Reader, "p", ANY, PreviousMatch),
    (Context::Reader, "P", ANY, PreviousMatch),
    (Context::Reader, "{", ANY, Paragraph(-1)),
    (Context::Reader, "}", ANY, Paragraph(1)),
    (Context::Reader, "]", ANY, Jump(1)),
    (Context::Reader, "[", ANY, Jump(-1)),
    (Context::Reader, "G", ANY, Bottom),
    (Context::Reader, "g", ANY, Top),
    (Context::Reader, "home", ANY, Top),
    (Context::Reader, "end", ANY, Bottom),
    // The changes list.
    (Context::ChangesList, "enter", ANY, FocusDiff),
    (Context::ChangesList, "l", ANY, FocusDiff),
    (Context::ChangesList, "L", ANY, FocusDiff),
    (Context::ChangesList, "right", ANY, FocusDiff),
    (Context::ChangesList, "G", ANY, Bottom),
    (Context::ChangesList, "g", ANY, Top),
    (Context::ChangesList, "home", ANY, Top),
    (Context::ChangesList, "end", ANY, Bottom),
    (Context::ChangesList, "shift-down", ANY, Move(5)),
    (Context::ChangesList, "shift-up", ANY, Move(-5)),
    (Context::ChangesList, "J", ANY, Move(5)),
    (Context::ChangesList, "K", ANY, Move(-5)),
    (Context::ChangesList, "j", ANY, Move(1)),
    (Context::ChangesList, "down", ANY, Move(1)),
    (Context::ChangesList, "k", ANY, Move(-1)),
    (Context::ChangesList, "up", ANY, Move(-1)),
    (Context::ChangesList, "pagedown", ANY, MovePage(1)),
    (Context::ChangesList, "pageup", ANY, MovePage(-1)),
    // The changes body: the reader's scroll keys without the file-only ones.
    (Context::ChangesReader, "ctrl-d", ANY, HalfPage(1)),
    (Context::ChangesReader, "ctrl-u", ANY, HalfPage(-1)),
    (Context::ChangesReader, "H", ANY, Pan(-4)),
    (Context::ChangesReader, "h", ANY, Pan(-4)),
    (Context::ChangesReader, "left", ANY, Pan(-4)),
    (Context::ChangesReader, "L", ANY, Pan(4)),
    (Context::ChangesReader, "l", ANY, Pan(4)),
    (Context::ChangesReader, "right", ANY, Pan(4)),
    (Context::ChangesReader, "J", ANY, Scroll(1)),
    (Context::ChangesReader, "j", ANY, Scroll(1)),
    (Context::ChangesReader, "down", ANY, Scroll(1)),
    (Context::ChangesReader, "K", ANY, Scroll(-1)),
    (Context::ChangesReader, "k", ANY, Scroll(-1)),
    (Context::ChangesReader, "up", ANY, Scroll(-1)),
    (Context::ChangesReader, "space", ANY, Page(1)),
    (Context::ChangesReader, "pagedown", ANY, Page(1)),
    (Context::ChangesReader, "B", ANY, Page(-1)),
    (Context::ChangesReader, "b", ANY, Page(-1)),
    (Context::ChangesReader, "pageup", ANY, Page(-1)),
    (Context::ChangesReader, "G", ANY, Bottom),
    (Context::ChangesReader, "g", ANY, Top),
    (Context::ChangesReader, "home", ANY, Top),
    (Context::ChangesReader, "end", ANY, Bottom),
];

/// The default rows as bindings, for a keymap with nothing from the file.
pub fn default_bindings() -> Vec<Binding> {
    DEFAULTS
        .iter()
        .map(|(context, keys, typing, action)| Binding {
            context: *context,
            keys: vec![Chord::parse(keys).expect("a default key parses")],
            typing: *typing,
            action: *action,
            user: false,
        })
        .collect()
}

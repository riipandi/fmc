//! Turns one key press into an action, in Scouter's dispatch order (`src/app/app.ts` `onKeyPress`): quit, count
//! digits, global commands, changes-screen keys, then the focused pane. The search prompt reads keys before this
//! parser does.

use super::{Action, Key};

/// The largest count a run of digits can build (Scouter `MAX_COUNT`).
const MAX_COUNT: u32 = 99_999_999;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    List,
    Reader,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Browse,
    Changes,
}

/// What the app looks like when the key arrives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub screen: Screen,
    pub focus: Focus,
    /// The reader has something open (Scouter `readerReady`); counts need it.
    pub reader_ready: bool,
    /// The reader shows rendered Markdown, which refuses counts and line motions.
    pub previewing: bool,
    /// A find or search prompt is open: `Ctrl+o` and `Alt+i` are left to it.
    pub typing: bool,
}

/// The parser's answer for one key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// The key started or extended a sequence (`z`, count digits); wait for the next one.
    More,
    /// No binding: the key does nothing.
    Bad,
    Done(Action),
}

/// Pending keys between presses: a count and a first `z`.
#[derive(Debug, Default)]
pub struct Parser {
    count: u32,
    z_pending: bool,
}

impl Parser {
    /// The count typed so far, for the footer.
    pub fn pending_count(&self) -> Option<u32> {
        (self.count > 0).then_some(self.count)
    }

    pub fn feed(&mut self, key: Key<'_>, state: State) -> Step {
        let after_z = std::mem::take(&mut self.z_pending);

        // Ctrl+q and Ctrl+c quit; a plain `q` does nothing (approved deviation, architecture.md).
        if (key.name == "q" || key.name == "c") && key.ctrl {
            return Step::Done(Action::Quit);
        }
        if self.absorb_count(key, state) {
            return Step::More;
        }
        let count = std::mem::take(&mut self.count);

        if let Some(action) = global(key, state) {
            return Step::Done(action);
        }
        if state.screen == Screen::Changes
            && let Some(action) = changes(key)
        {
            return Step::Done(action);
        }
        let action = match (state.focus, state.screen) {
            (Focus::Reader, _) => return self.reader(key, state, count, after_z),
            (Focus::List, Screen::Changes) => change_list(key),
            (Focus::List, Screen::Browse) => tree(key),
        };
        action.map_or(Step::Bad, Step::Done)
    }

    fn absorb_count(&mut self, key: Key<'_>, state: State) -> bool {
        let digit = match key.text.as_bytes() {
            [d @ b'0'..=b'9'] if key.plain() => u32::from(d - b'0'),
            _ => return false,
        };
        if !state.reader_ready
            || state.focus != Focus::Reader
            || state.screen != Screen::Browse
            || state.previewing
            || (self.count == 0 && digit == 0)
        {
            return false;
        }
        self.count = (self.count * 10 + digit).min(MAX_COUNT);
        true
    }

    fn reader(&mut self, key: Key<'_>, state: State, count: u32, after_z: bool) -> Step {
        if key.alt {
            return Step::Bad;
        }
        if let Some(action) = scroll(key) {
            return Step::Done(action);
        }
        if key.ctrl {
            return Step::Bad;
        }
        if state.screen == Screen::Browse {
            match (key.name, key.shift) {
                ("/", _) => return Step::Done(Action::Search),
                ("n", false) => return Step::Done(Action::NextMatch),
                ("n", true) | ("p", _) => return Step::Done(Action::PreviousMatch),
                _ => {}
            }
            if key.plain() {
                if after_z && key.name == "z" && !key.shift {
                    return Step::Done(Action::Centre);
                }
                if !after_z && key.name == "z" && !key.shift {
                    self.z_pending = true;
                    return Step::More;
                }
                match key.text {
                    "{" => return Step::Done(Action::Paragraph(-1)),
                    "}" => return Step::Done(Action::Paragraph(1)),
                    "]" => return Step::Done(Action::Jump(1)),
                    "[" => return Step::Done(Action::Jump(-1)),
                    _ => {}
                }
            }
        }
        match boundary(key) {
            Some(Action::Bottom) if count > 0 => Step::Done(Action::GoToLine(count)),
            Some(action) => Step::Done(action),
            None => Step::Bad,
        }
    }
}

fn global(key: Key<'_>, state: State) -> Option<Action> {
    match key.name {
        "tab" => return Some(Action::SwitchPane),
        "escape" => return Some(Action::Escape),
        _ => {}
    }
    if key.text == "?" && key.plain() {
        return Some(Action::Help);
    }
    if key.text == ":" && key.plain() {
        return Some(Action::OpenCommand);
    }
    if key.ctrl && !key.alt {
        match key.name {
            "left" => return Some(Action::Resize(-1)),
            "right" => return Some(Action::Resize(1)),
            "o" if !state.typing => return Some(Action::JumpBack),
            // `Ctrl+_` is the legacy `0x1F` (phase 1 note); a prompt takes the key as nothing (plan-00002).
            "/" | "_" if !state.typing => return Some(Action::ToggleSidebar),
            // `Ctrl+F` flips the finder's name matching (plan-00006 D5); a prompt that is not the finder has no use for it.
            "f" if state.typing => return Some(Action::ToggleFuzzy),
            // A prompt takes a plain `.` as text; `Ctrl+.` is not text, so the finder rebuilds its index with it.
            "." => return Some(Action::ToggleDotfiles),
            _ => {}
        }
    }
    if key.alt && !key.ctrl && key.name == "i" && !state.typing {
        return Some(Action::JumpForward);
    }
    if !key.plain() || key.shift {
        return None;
    }
    match key.name {
        "." if !state.typing => Some(Action::ToggleDotfiles),
        "w" => Some(Action::ToggleWrap),
        "q" => Some(Action::Close),
        "m" => Some(Action::ToggleMarkdown),
        "d" => Some(Action::ToggleChanges),
        "y" => Some(Action::Copy),
        _ => None,
    }
}

fn changes(key: Key<'_>) -> Option<Action> {
    if !key.plain() {
        return None;
    }
    Some(match (key.name, key.shift) {
        ("s", _) => Action::ToggleSplit,
        ("a", _) => Action::ToggleScope,
        ("c", _) => Action::ToggleFold,
        ("x", _) => Action::ToggleContext,
        ("r", _) => Action::Reload,
        ("o", _) => Action::OpenChanged,
        ("n", false) => Action::Hunk(1),
        ("n", true) | ("p", _) => Action::Hunk(-1),
        _ if key.text == "]" => Action::Hunk(1),
        _ if key.text == "[" => Action::Hunk(-1),
        _ => return None,
    })
}

/// `j k ↑ ↓` move one row, five with Shift; Page Up and Page Down move a screen (Scouter `handleListKey`).
fn list_move(key: Key<'_>) -> Option<Action> {
    if !key.plain() {
        return None;
    }
    let step = if key.shift { 5 } else { 1 };
    match key.name {
        "j" | "down" => Some(Action::Move(step)),
        "k" | "up" => Some(Action::Move(-step)),
        "pagedown" => Some(Action::MovePage(1)),
        "pageup" => Some(Action::MovePage(-1)),
        _ => None,
    }
}

fn tree(key: Key<'_>) -> Option<Action> {
    if !key.plain() {
        return None;
    }
    match key.text {
        "]" => return Some(Action::Jump(1)),
        "[" => return Some(Action::Jump(-1)),
        _ => {}
    }
    match (key.name, key.shift) {
        ("a", false) => return Some(Action::NewFile),
        ("a", true) => return Some(Action::NewFolder),
        ("r", false) => return Some(Action::Rename),
        ("d", true) | ("delete", _) => return Some(Action::Delete),
        _ => {}
    }
    match key.name {
        "return" => Some(Action::Toggle),
        "l" | "right" => Some(Action::Expand),
        "h" | "left" | "backspace" => Some(Action::Collapse),
        "-" => Some(Action::GoUp),
        "p" => Some(Action::TogglePreview),
        "/" => Some(Action::Find),
        "space" => Some(Action::ContextMenu),
        _ => boundary(key).or_else(|| list_move(key)),
    }
}

fn change_list(key: Key<'_>) -> Option<Action> {
    if !key.plain() {
        return None;
    }
    match key.name {
        "return" | "l" | "right" => Some(Action::FocusDiff),
        _ => boundary(key).or_else(|| list_move(key)),
    }
}

/// `g` and Home go to the top, `G` and End to the bottom (Scouter `boundaryOf`).
fn boundary(key: Key<'_>) -> Option<Action> {
    match key.name {
        "g" if key.shift => Some(Action::Bottom),
        "g" | "home" => Some(Action::Top),
        "end" => Some(Action::Bottom),
        _ => None,
    }
}

/// Reader scrolling (Scouter `scrollByKey`): lines, four-column pans, 0.9 pages, and half pages with Ctrl.
fn scroll(key: Key<'_>) -> Option<Action> {
    if key.ctrl {
        return match key.name {
            "d" => Some(Action::HalfPage(1)),
            "u" => Some(Action::HalfPage(-1)),
            _ => None,
        };
    }
    match key.name {
        "h" | "left" => Some(Action::Pan(-4)),
        "l" | "right" => Some(Action::Pan(4)),
        "j" | "down" => Some(Action::Scroll(1)),
        "k" | "up" => Some(Action::Scroll(-1)),
        "pagedown" | "space" => Some(Action::Page(1)),
        "pageup" | "b" => Some(Action::Page(-1)),
        _ => None,
    }
}

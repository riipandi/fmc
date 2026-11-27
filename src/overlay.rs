//! The overlays over the page (plan-00012, D1): a context menu and a dialog. While one is open the app gives it every key
//! (`app/capture.rs`), and these types decide what each key does. A key an overlay does not use is swallowed: `Stay`.
//! No widget and no I/O; the app draws the overlay and carries out what it answers.

use crate::keys::Key;

/// One row of a menu. A disabled row is drawn muted, and the selection skips it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Item {
    pub label: String,
    pub enabled: bool,
    /// The tree key that does the same act, shown at the row's right edge; empty for none.
    pub hint: &'static str,
    /// A destructive act: its label takes the error colour.
    pub danger: bool,
    /// A rule above the row starts a new group.
    pub rule: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Menu {
    items: Vec<Item>,
    selected: usize,
}

impl Menu {
    /// A menu with its first enabled row selected, or row 0 when no row is enabled.
    pub fn new(items: Vec<Item>) -> Self {
        let selected = items.iter().position(|item| item.enabled).unwrap_or(0);
        Self { items, selected }
    }

    pub fn items(&self) -> &[Item] {
        &self.items
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    /// The pointer moved over row `index`: the row is selected when it is enabled, as the keys select it. Returns whether
    /// the selection changed.
    pub fn point(&mut self, index: usize) -> bool {
        match self.items.get(index) {
            Some(item) if item.enabled && index != self.selected => {
                self.selected = index;
                true
            }
            _ => false,
        }
    }

    /// A click on row `index`: picks it when it is enabled.
    pub fn pick(&self, index: usize) -> Outcome {
        match self.items.get(index) {
            Some(item) if item.enabled => Outcome::Pick(index),
            _ => Outcome::Stay,
        }
    }

    pub fn key(&mut self, key: Key<'_>) -> Outcome {
        if !key.plain() {
            return Outcome::Stay;
        }
        match key.name {
            "escape" => Outcome::Close,
            "return" => self.pick(self.selected),
            "up" => self.move_by(false),
            "down" => self.move_by(true),
            "k" if !key.shift => self.move_by(false),
            "j" if !key.shift => self.move_by(true),
            _ => Outcome::Stay,
        }
    }

    /// The next enabled row in one direction; the selection stays when no enabled row lies that way.
    fn move_by(&mut self, forward: bool) -> Outcome {
        let next = if forward {
            (self.selected + 1..self.items.len()).find(|&i| self.items[i].enabled)
        } else {
            (0..self.selected).rev().find(|&i| self.items[i].enabled)
        };
        if let Some(index) = next {
            self.selected = index;
        }
        Outcome::Stay
    }
}

/// Where a dialog's focus is: its field, or one of its buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    Field,
    Button(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dialog {
    pub title: String,
    pub message: String,
    /// A one-line field (a name) takes the typed text.
    pub field: bool,
    /// At least one button. Return on the field picks `default`.
    pub buttons: Vec<String>,
    pub default: usize,
    pub focus: Stop,
    /// The button under the pointer: drawn highlighted, and not the focus, so Return still picks the focused button.
    pub hovered: Option<usize>,
    /// The line under the field; the caller sets it and clears it.
    pub error: Option<String>,
}

impl Dialog {
    /// A dialog whose focus starts on its field, or on its default button when it has none.
    pub fn new(title: &str, message: &str, field: bool, buttons: &[&str], default: usize) -> Self {
        let default = default.min(buttons.len().saturating_sub(1));
        let focus = if field { Stop::Field } else { Stop::Button(default) };
        Self {
            title: title.into(),
            message: message.into(),
            field,
            buttons: buttons.iter().map(|b| (*b).into()).collect(),
            default,
            focus,
            hovered: None,
            error: None,
        }
    }

    /// The focus stops in Tab order: the field first when there is one, then the buttons.
    fn stops(&self) -> Vec<Stop> {
        let field = self.field.then_some(Stop::Field);
        field.into_iter().chain((0..self.buttons.len()).map(Stop::Button)).collect()
    }

    /// The pointer entered (`on`) or left button `index`. A highlight only: the focus stays where the keys put it.
    /// Returns whether the highlight changed.
    pub fn hover(&mut self, index: usize, on: bool) -> bool {
        let next = if on { Some(index) } else { self.hovered.filter(|&hovered| hovered != index) };
        if next == self.hovered {
            return false;
        }
        self.hovered = next;
        true
    }

    /// A press on button `index`: picks it when the dialog has that button.
    pub fn pick(&self, index: usize) -> Outcome {
        if index < self.buttons.len() { Outcome::Pick(index) } else { Outcome::Stay }
    }

    pub fn key(&mut self, key: Key<'_>) -> Outcome {
        // Before the plain-key rule: an Option or Cmd key edits the field, where a plain key would be swallowed.
        if self.field
            && let Some(edit) = field_edit(key)
        {
            self.focus = Stop::Field;
            return Outcome::Edit(edit);
        }
        if !key.plain() {
            return Outcome::Stay;
        }
        match key.name {
            "escape" => return Outcome::Close,
            "tab" => {
                let stops = self.stops();
                let at = stops.iter().position(|stop| *stop == self.focus).unwrap_or(0);
                self.focus = stops[(at + 1) % stops.len()];
                return Outcome::Stay;
            }
            "left" | "right" => {
                let last = self.buttons.len().saturating_sub(1);
                self.focus = match self.focus {
                    Stop::Field => Stop::Button(self.default),
                    Stop::Button(i) if key.name == "right" => Stop::Button((i + 1).min(last)),
                    Stop::Button(i) => Stop::Button(i.saturating_sub(1)),
                };
                return Outcome::Stay;
            }
            "return" => {
                return match self.focus {
                    Stop::Field => self.pick(self.default),
                    Stop::Button(i) => self.pick(i),
                };
            }
            _ => {}
        }
        if self.field && is_text(key.text) {
            self.focus = Stop::Field;
            return Outcome::Insert(key.text.to_string());
        }
        Outcome::Stay
    }
}

/// An edit of a dialog's name field. The app runs each one as the OpenTUI `Input` action of the same meaning
/// (`app/overlay.rs` `editor_action`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldEdit {
    Backspace,
    DeleteWordBackward,
    DeleteWordForward,
    DeleteToLineStart,
    DeleteToLineEnd,
    WordBackward,
    WordForward,
    LineHome,
    LineEnd,
}

/// The field edit a key asks for. Option is Alt, and Cmd is Super, which arrives only with the kitty protocol or
/// from a terminal that passes Cmd through. The Ctrl letters are readline's. Shift is ignored: the field has no
/// selection, so Shift+Option+Left moves by word as Option+Left does.
fn field_edit(key: Key<'_>) -> Option<FieldEdit> {
    use FieldEdit::*;
    Some(match (key.name, key.ctrl, key.alt, key.cmd) {
        ("backspace", false, false, false) => Backspace,
        // ESC DEL is Option+Backspace without the kitty protocol; Ctrl+Backspace is OpenTUI's own binding.
        ("backspace", false, true, false) | ("backspace", true, false, false) | ("w", true, false, false) => {
            DeleteWordBackward
        }
        ("backspace", false, false, true) | ("u", true, false, false) => DeleteToLineStart,
        ("delete", false, true, false) | ("delete", true, false, false) | ("d", false, true, false) => {
            DeleteWordForward
        }
        ("k", true, false, false) => DeleteToLineEnd,
        ("left", false, true, false) | ("left", true, false, false) | ("b", false, true, false) => WordBackward,
        ("right", false, true, false) | ("right", true, false, false) | ("f", false, true, false) => WordForward,
        ("left", false, false, true) | ("a", true, false, false) => LineHome,
        ("right", false, false, true) | ("e", true, false, false) => LineEnd,
        _ => return None,
    })
}

/// Typed text: a key that carries printable characters, not a control key or a named one.
fn is_text(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| !c.is_control())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Overlay {
    Menu(Menu),
    Dialog(Dialog),
}

/// What a key, or a click, asks the app to carry out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing to carry out; the key is swallowed.
    Stay,
    /// The overlay closes, with no answer.
    Close,
    /// A row or a button was chosen; the overlay closes.
    Pick(usize),
    /// Text typed into the field.
    Insert(String),
    /// An edit of the field: its text or its caret moves.
    Edit(FieldEdit),
}

impl Overlay {
    pub fn key(&mut self, key: Key<'_>) -> Outcome {
        match self {
            Self::Menu(menu) => menu.key(key),
            Self::Dialog(dialog) => dialog.key(key),
        }
    }
}

#[cfg(test)]
#[path = "overlay_tests.rs"]
mod tests;

//! The overlays over the page (plan-00012, D1): a context menu and a dialog. While one is open the app gives it every key
//! (`app/capture.rs`), and these types decide what each key does. A key an overlay does not use is swallowed: `Stay`.
//! No widget and no I/O; the app draws the overlay and carries out what it answers.

use crate::keys::Key;

/// One row of a menu. A disabled row is drawn muted, and the selection skips it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub label: String,
    pub enabled: bool,
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
            error: None,
        }
    }

    /// The focus stops in Tab order: the field first when there is one, then the buttons.
    fn stops(&self) -> Vec<Stop> {
        let field = self.field.then_some(Stop::Field);
        field.into_iter().chain((0..self.buttons.len()).map(Stop::Button)).collect()
    }

    /// A press on button `index`: picks it when the dialog has that button.
    pub fn pick(&self, index: usize) -> Outcome {
        if index < self.buttons.len() { Outcome::Pick(index) } else { Outcome::Stay }
    }

    pub fn key(&mut self, key: Key<'_>) -> Outcome {
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
            "backspace" if self.field => {
                self.focus = Stop::Field;
                return Outcome::Backspace;
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
    /// The character before the caret of the field is removed.
    Backspace,
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

//! The binary notice (Scouter `src/reader/binary-view.ts`): a centred block with the file's name and size, what the
//! reader does show, and the choices it offers instead, as buttons a key or a click takes.

use fmc::keys::Action;
use fmc::theme::Palette;
use opentui::prelude::*;

use super::styled::{Seg, content};

/// The line under the title: what the reader shows instead (the product name is fmc).
pub const HINT: &str = "fmc only displays UTF-8 text and images.";

/// The choices under the hint: the key, what it does, and the action a click runs.
pub const CHOICES: &[(&str, &str, Action)] =
    &[("X", "hex view", Action::OpenAsHex), ("o", "open in the default app", Action::OpenDefault)];

pub struct BinaryView {
    root: View,
    title: Text,
    meta: Text,
    hint: Text,
    buttons: Vec<Text>,
}

impl BinaryView {
    pub fn new(app: &App) -> Self {
        let root = View::new(app)
            .grow(1.0)
            .min_height(0)
            .min_width(0)
            .column()
            .align_items(Align::Center)
            .justify(Justify::Center)
            .overflow(Overflow::Hidden)
            .visible(false);
        let line = |margin: u16| {
            Text::new(app, "")
                .height(1)
                .shrink(0.0)
                .margin_edge(Edge::Top, margin)
                .wrap(WrapMode::None)
                .selectable(false)
        };
        let (title, meta, hint) = (line(0), line(1), line(1));
        let row = View::new(app).row().height(1).shrink(0.0).margin_edge(Edge::Top, 1);
        let buttons: Vec<Text> = CHOICES
            .iter()
            .enumerate()
            .map(|(index, _)| {
                let button = Text::new(app, "")
                    .height(1)
                    .shrink(0.0)
                    .wrap(WrapMode::None)
                    .selectable(false)
                    .margin_edge(Edge::Left, if index == 0 { 0 } else { 3 });
                row.add(&button);
                button
            })
            .collect();
        root.add(&title).add(&meta).add(&hint).add(&row);
        Self { root, title, meta, hint, buttons }
    }

    pub fn root(&self) -> View {
        self.root.clone()
    }

    pub fn visible(&self, visible: bool) {
        self.root.visible(visible);
    }

    /// Each choice's button, with the action a click on it runs.
    pub fn buttons(&self) -> Vec<(Text, Action)> {
        self.buttons.iter().cloned().zip(CHOICES.iter().map(|(_, _, action)| *action)).collect()
    }

    /// Paints the notice for `name` (already display-safe) and `size` (already formatted).
    pub fn paint(&self, name: &str, size: &str, p: &Palette) {
        self.title.set_text(content(&[Seg::new("Binary files are not supported.", p.text).bold()], p.text));
        self.meta.set_text(content(
            &[Seg::new(name, p.accent_soft), Seg::new("  ·  ", p.ignored), Seg::new(size, p.muted)],
            p.text,
        ));
        self.hint.set_text(content(&[Seg::new(HINT, p.dim)], p.text));
        for (button, (key, label, _)) in self.buttons.iter().zip(CHOICES) {
            button.set_text(content(
                &[Seg::new(*key, p.accent_soft).bold(), Seg::new(format!(" {label}"), p.dim)],
                p.text,
            ));
        }
    }
}

#[cfg(test)]
#[path = "binary_tests.rs"]
mod tests;

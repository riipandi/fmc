//! The dialog: a box centred over the page, with the page behind it dimmed by a scrim (plan-00012, D1). A title, a message
//! that wraps, an optional one-line field, an inline error, and buttons. In the compact tier it spans the width less a
//! column on each side.

use fmc::layout::Tier;
use fmc::overlay::{Dialog, Stop};
use fmc::theme::Palette;
use opentui::prelude::*;

use super::menu::Rect;
use super::styled::{Seg, color, content};

/// The gain that dims the page under a dialog: OpenTUI's own example of the page behind a dialog.
const SCRIM_GAIN: f32 = 0.4;

/// The widest dialog, in the medium and wide tiers.
const MAX_WIDTH: u16 = 56;

/// The buttons the view holds. shortcut: a fourth button is not drawn; grow the pool when a dialog needs one.
pub const MAX_BUTTONS: usize = 3;

/// Dims what is painted under it. The catcher beneath a dialog takes its clicks (`app/overlay.rs`).
pub struct Scrim;

impl Component for Scrim {
    fn paint(&self, painter: &mut Painter<'_>) {
        let (width, height) = painter.size();
        painter.color_matrix(0, 0, width, height, &ColorMatrix::gain(SCRIM_GAIN));
    }
}

/// The box of a dialog on a screen of `tier`: its width by tier, its height from the message's wrapped rows, and centred.
pub fn frame(screen: (u16, u16), tier: Tier, message: &str, field: bool) -> Rect {
    let (columns, rows) = screen;
    let width = match tier {
        Tier::Compact => columns.saturating_sub(2),
        Tier::Medium | Tier::Wide => columns.saturating_sub(4).min(MAX_WIDTH),
    };
    // Two columns of border and one of padding on each side.
    let lines = wrap_rows(message, usize::from(width.saturating_sub(4))) as u16;
    let height = (7 + lines + u16::from(field)).min(rows);
    Rect { x: columns.saturating_sub(width) / 2, y: rows.saturating_sub(height) / 2, width, height }
}

/// The rows `text` takes when its words wrap at `width` columns. shortcut: a word longer than the width counts as one
/// row, not the rows it breaks into; the messages are short sentences.
pub fn wrap_rows(text: &str, width: usize) -> usize {
    let rows: usize = text
        .lines()
        .map(|line| {
            let (mut rows, mut used) = (1, 0);
            for word in line.split_whitespace() {
                let len = word.chars().count();
                if used == 0 {
                    used = len;
                } else if used + 1 + len <= width {
                    used += 1 + len;
                } else {
                    rows += 1;
                    used = len;
                }
            }
            rows
        })
        .sum();
    rows.max(1)
}

pub struct DialogView {
    pub root: View,
    title: Text,
    message: Text,
    pub field: Input,
    error: Text,
    /// The buttons, in a row pushed to the right; a button past the dialog's buttons is hidden.
    pub buttons: Vec<Text>,
}

impl DialogView {
    pub fn new(app: &App) -> Self {
        let root = View::new(app)
            .absolute()
            .z_index(i32::MAX)
            .column()
            .border(BorderStyle::Rounded)
            .padding_x(1)
            .visible(false);
        let title = Text::new(app, "").height(1).wrap(WrapMode::None);
        let message = Text::new(app, "").wrap(WrapMode::Word);
        let field = Input::new(app).height(1);
        let error = Text::new(app, "").height(1).wrap(WrapMode::None);
        let row = View::new(app).row().height(1).gap(2);
        row.add(&View::new(app).grow(1.0));
        let buttons: Vec<Text> = (0..MAX_BUTTONS)
            .map(|_| {
                let button = Text::new(app, "").height(1).wrap(WrapMode::None);
                row.add(&button);
                button
            })
            .collect();
        root.add(&title).add(&spacer(app)).add(&message).add(&spacer(app)).add(&field).add(&error).add(&row);
        Self { root, title, message, field, error, buttons }
    }

    /// Draws `dialog` in `rect` on the `background` tint, with its focus marked.
    pub fn show(&self, dialog: &Dialog, rect: Rect, p: &Palette, background: &str) {
        self.root
            .position(Edge::Left, Length::Cells(f32::from(rect.x)))
            .position(Edge::Top, Length::Cells(f32::from(rect.y)))
            .width(Length::Cells(f32::from(rect.width)))
            .height(Length::Cells(f32::from(rect.height)))
            .bg(color(background))
            .border_color(color(p.accent));
        let bold = Seg { text: dialog.title.clone(), fg: Some(p.text), bg: None, bold: true };
        self.title.set_text(content(&[bold], p.text));
        self.message.set_text(content(&[Seg::new(dialog.message.clone(), p.text)], p.text));
        self.field.visible(dialog.field).colors(color(p.text), color(background));
        let error = dialog.error.clone().unwrap_or_default();
        self.error.set_text(content(&[Seg::new(error, p.error)], p.error));
        for (index, button) in self.buttons.iter().enumerate() {
            let label = dialog.buttons.get(index);
            button.visible(label.is_some());
            let Some(label) = label else { continue };
            let focused = dialog.focus == Stop::Button(index);
            let seg = Seg {
                text: format!(" {label} "),
                fg: Some(if focused { p.accent_ink } else { p.text }),
                bg: focused.then_some(p.accent),
                bold: focused,
            };
            button.set_text(content(&[seg], p.text));
        }
    }
}

/// A blank row of the dialog.
fn spacer(app: &App) -> View {
    View::new(app).height(1)
}

#[cfg(test)]
#[path = "dialog_tests.rs"]
mod tests;

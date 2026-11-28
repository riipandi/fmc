//! A Markdown image's box in the rendered view (`reader/images.rs`): the name, the path with the size and format, and
//! the hint or the reason, in a frame of three text rows. It is a measured widget, so layout offers it the reader's
//! width and a resize measures it again.

use std::cell::RefCell;
use std::rc::Rc;

use fmc::reader::images::{self, Header, Image, Tone};
use fmc::theme::Palette;
use opentui::native::Constraint;
use opentui::prelude::*;

use super::styled::color;

/// A top border, three text rows, and a bottom border.
pub const ROWS: usize = 5;

/// The width a box takes when layout offers no width: the diagram's natural width is not a box's.
const UNBOUNDED_COLUMNS: usize = 60;

/// The palette a box draws in: its frame in `border`, its name and an open hint in `text`, the rest in `muted`.
#[derive(Clone, Copy)]
pub struct Colors {
    border: Color,
    text: Color,
    muted: Color,
}

impl Colors {
    pub fn new(p: &Palette) -> Self {
        Self { border: color(p.border), text: color(p.text), muted: color(p.muted) }
    }
}

pub struct ImageBox {
    card: images::Card,
    colors: Colors,
    /// The header a release picks, when the box has one to act on; the view takes it after the release.
    header: Option<Header>,
    pick: Rc<RefCell<Option<Header>>>,
}

impl ImageBox {
    /// A box for `image`, whose header is `None` until the worker has read it.
    pub fn new(image: &Image, header: Option<&Header>, colors: Colors, pick: Rc<RefCell<Option<Header>>>) -> Self {
        Self { card: images::card(image, header), colors, header: header.cloned(), pick }
    }
}

impl Component for ImageBox {
    fn paint(&self, painter: &mut Painter<'_>) {
        let width = painter.width() as usize;
        // Too narrow for a frame and its margins: nothing to draw.
        if width < 4 {
            return;
        }
        let right = painter.width() as i32 - 1;
        let bar = "─".repeat(width - 2);
        let border = Style::new().fg(self.colors.border);
        painter.text(0, 0, &format!("┌{bar}┐"), border);
        for (row, (tone, text)) in images::text_rows(&self.card, width - 4).into_iter().enumerate() {
            let y = row as i32 + 1;
            let fg = match tone {
                Tone::Text => self.colors.text,
                Tone::Muted => self.colors.muted,
            };
            painter.text(0, y, "│", border);
            painter.text(2, y, &text, Style::new().fg(fg));
            painter.text(right, y, "│", border);
        }
        painter.text(0, ROWS as i32 - 1, &format!("└{bar}┘"), border);
    }

    /// A release over the box picks its header, which `Enter` on the box would open (`app/picture.rs`).
    fn mouse(&mut self, _cx: &mut Cx<'_>, event: &MouseEvent, _local: (i32, i32)) -> bool {
        if event.kind != MouseEventKind::Up(MouseButton::Left) {
            return false;
        }
        // Not consumed: the release goes on to the pane, which takes the pick (`app/mouse.rs`).
        if let Some(header) = self.header.clone() {
            *self.pick.borrow_mut() = Some(header);
        }
        false
    }

    fn measure(&self, width: Constraint, _height: Constraint) -> (f32, f32) {
        let columns = match width {
            Constraint::Exactly(width) | Constraint::AtMost(width) => width.max(0.0) as usize,
            Constraint::Unbounded => UNBOUNDED_COLUMNS,
        };
        (columns as f32, ROWS as f32)
    }
}

#[cfg(test)]
#[path = "image_box_tests.rs"]
mod tests;

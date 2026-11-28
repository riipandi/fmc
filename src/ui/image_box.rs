//! A Markdown image's box in the rendered view (`reader/images.rs`): the name, the path with the size and format, and the
//! hint or the reason, in a frame. It is a measured widget, so layout offers it the pane's width and a resize measures it
//! again. Its rows follow the picture's size (`Size`), so the picture can take its place without the lines below moving.

use std::cell::RefCell;
use std::rc::Rc;

use fmc::reader::images::{self, Card, Header, Tone};
use fmc::theme::Palette;
use opentui::native::Constraint;
use opentui::prelude::*;

use super::styled::color;

/// A top border, three text rows, and a bottom border: the least a box takes.
pub const ROWS: usize = 5;

/// The width a box takes when layout offers no width: the diagram's natural width is not a box's.
const UNBOUNDED_COLUMNS: usize = 60;

/// How a box or a picture takes its size: `columns` wide (`None`: the pane's, which layout stretches it to), its height as its
/// width over `ratio` (`None`: `ROWS`), and the most rows it takes. Layout applies it (`apply`), and a box's measure follows
/// the same rule.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size {
    pub columns: Option<u32>,
    pub ratio: Option<f32>,
    pub cap: u32,
}

impl Size {
    /// Gives `widget` this size. The width is set, not a maximum: a widget that takes a maximum and no width is stretched to
    /// the pane's width instead, and a fixed width ignores a maximum (checked in `ui/markdown_pictures_tests.rs`).
    pub fn apply<W: Widget>(&self, widget: &W) {
        if let Some(columns) = self.columns {
            widget.width(columns);
        }
        match self.ratio {
            Some(ratio) => {
                widget.aspect_ratio(ratio).min_height(ROWS).max_height(self.cap);
            }
            None => {
                widget.height(ROWS);
            }
        }
    }

    /// The size a box measures at when layout offers `offered` columns.
    fn measure(&self, offered: usize) -> (f32, f32) {
        let columns = self.columns.map_or(offered, |columns| (columns as usize).min(offered));
        let rows = match self.ratio {
            Some(ratio) => (columns as f32 / ratio).clamp(ROWS as f32, self.cap.max(ROWS as u32) as f32),
            None => ROWS as f32,
        };
        (columns as f32, rows)
    }
}

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
    card: Card,
    size: Size,
    colors: Colors,
    /// The header a release picks, when the box has one to act on; the view takes it after the release.
    header: Option<Header>,
    pick: Rc<RefCell<Option<Header>>>,
}

impl ImageBox {
    pub fn new(
        card: Card,
        size: Size,
        colors: Colors,
        header: Option<Header>,
        pick: Rc<RefCell<Option<Header>>>,
    ) -> Self {
        Self { card, size, colors, header, pick }
    }
}

impl Component for ImageBox {
    fn paint(&self, painter: &mut Painter<'_>) {
        let width = painter.width() as usize;
        // Too narrow for a frame and its margins: nothing to draw.
        if width < 4 {
            return;
        }
        let bottom = painter.height() as i32 - 1;
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
        // A picture's box is taller than its three text rows: the frame runs on below them.
        for y in ROWS as i32 - 1..bottom {
            painter.text(0, y, "│", border);
            painter.text(right, y, "│", border);
        }
        painter.text(0, bottom, &format!("└{bar}┘"), border);
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
        self.size.measure(columns)
    }
}

#[cfg(test)]
#[path = "image_box_tests.rs"]
mod tests;

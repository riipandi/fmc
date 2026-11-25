//! The reader's scrolling pane (Scouter `src/reader/reader-pane.ts`, the `ScrollBoxRenderable` of the viewer): the
//! text, the rendered Markdown, and the picture scroll inside it, both ways. Its own bars are hidden; the viewer's
//! bar is `ui/scrollbar.rs`, which Scouter paints over the same position.

use opentui::prelude::*;
use opentui::widget::ScrollBox;

pub struct Viewer {
    root: ScrollBox,
}

impl Viewer {
    pub fn new(app: &App) -> Self {
        let root = ScrollBox::new(app).axes(true, true).scrollbars(false).grow(1.0).min_width(0).min_height(0);
        Self { root }
    }

    pub fn widget(&self) -> ScrollBox {
        self.root.clone()
    }

    /// Without wrapping, the content takes the width of its widest line, so the pane pans; with wrapping, it is the
    /// viewport's width.
    pub fn set_wrap(&self, wrap: bool) {
        let width = if wrap { Length::percent(100.0) } else { Length::Auto };
        self.root.content_view().width(width);
    }

    /// The scroll position as `(columns, rows)`, as the latest frame laid it out.
    pub fn position(&self) -> (f64, f64) {
        self.root.scroll_position()
    }

    pub fn scroll_by(&self, columns: f64, rows: f64) {
        self.root.scroll_by(columns, rows);
    }

    pub fn scroll_to(&self, columns: f64, rows: f64) {
        self.root.scroll_to(columns, rows);
    }

    pub fn scroll_to_bottom(&self) {
        self.root.scroll_to_bottom();
    }
}

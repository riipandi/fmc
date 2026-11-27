//! A Mermaid fence in rendered Markdown, drawn as its box diagram (`reader/mermaid.rs`) in the palette: box lines in
//! `border`, labels in `text`, the fallback's reason in `muted` (plan-00006, D3). It is a measured widget, so layout
//! offers it the reader's width and a resize measures it again; the renderer is not called on resize
//! (`architecture.md`, "Mermaid in rendered Markdown").

use std::cell::RefCell;
use std::rc::Rc;

use fmc::reader::mermaid;
use fmc::theme::Palette;
use opentui::native::Constraint;
use opentui::prelude::*;

use super::styled::color;

/// The three colours a diagram takes from the palette.
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

/// The rows for one width. A fallback's rows are its framed source, and its last row is the reason.
struct Drawn {
    rows: Vec<String>,
    fallback: bool,
}

pub struct Diagram {
    source: String,
    colors: Colors,
    /// The rows for the last width asked for. The source never changes: a changed fence builds a new widget.
    cache: RefCell<Option<(Option<usize>, Rc<Drawn>)>>,
}

impl Diagram {
    pub fn new(source: &str, colors: Colors) -> Self {
        Self { source: source.to_owned(), colors, cache: RefCell::new(None) }
    }

    /// The rows at `width`, from the cache when the last call asked for the same width.
    fn drawn(&self, width: Option<usize>) -> Rc<Drawn> {
        let mut cache = self.cache.borrow_mut();
        if let Some((cached, drawn)) = &*cache
            && *cached == width
        {
            return Rc::clone(drawn);
        }
        let drawn = Rc::new(match mermaid::render(&self.source, width) {
            Ok(rows) => Drawn { rows, fallback: false },
            Err(fallback) => Drawn { rows: mermaid::framed(&self.source, &fallback.reason, width), fallback: true },
        });
        *cache = Some((width, Rc::clone(&drawn)));
        drawn
    }

    /// The rows the last measure made, when they fit the box; otherwise the rows for the box's width. The engine is not
    /// idempotent at its own width (the gantt sample is 96 columns at 100 and falls back at 96), so paint must not
    /// render again at the width it was given.
    fn fitted(&self, width: usize) -> Rc<Drawn> {
        let last = self.cache.borrow().as_ref().map(|(_, drawn)| Rc::clone(drawn));
        match last {
            Some(drawn) if mermaid::columns(&drawn.rows) <= width => drawn,
            _ => self.drawn(Some(width)),
        }
    }
}

impl Component for Diagram {
    fn paint(&self, painter: &mut Painter<'_>) {
        let drawn = self.fitted(painter.width() as usize);
        let last = drawn.rows.len().saturating_sub(1);
        for (y, row) in drawn.rows.iter().enumerate() {
            let reason = drawn.fallback && y == last;
            let mut x = 0;
            for (glyphs, run) in runs(row) {
                let fg = match (reason, glyphs) {
                    (true, _) => self.colors.muted,
                    (false, true) => self.colors.border,
                    (false, false) => self.colors.text,
                };
                painter.text(x, y as i32, &run, Style::new().fg(fg));
                x += painter.text_width(&run) as i32;
            }
        }
    }

    fn measure(&self, width: Constraint, _height: Constraint) -> (f32, f32) {
        let width = match width {
            Constraint::Exactly(width) | Constraint::AtMost(width) => Some(width.max(0.0) as usize),
            Constraint::Unbounded => None,
        };
        let drawn = self.drawn(width);
        (mermaid::columns(&drawn.rows) as f32, drawn.rows.len() as f32)
    }
}

/// A row split into runs of box-drawing glyphs (`true`) and of everything else (`false`), so each run takes one colour.
fn runs(row: &str) -> Vec<(bool, String)> {
    let mut runs: Vec<(bool, String)> = Vec::new();
    for c in row.chars() {
        let glyph = is_glyph(c);
        match runs.last_mut() {
            Some((kind, text)) if *kind == glyph => text.push(c),
            _ => runs.push((glyph, c.to_string())),
        }
    }
    runs
}

/// Arrows, box drawing, block elements, and geometric shapes: the glyphs `mermaid-text` draws its lines and arrowheads
/// with. A label that uses one of them takes the line colour too; the engine's output has no other marker.
fn is_glyph(c: char) -> bool {
    matches!(c, '\u{2190}'..='\u{21ff}' | '\u{2500}'..='\u{259f}' | '\u{25a0}'..='\u{25ff}')
}

#[cfg(test)]
#[path = "mermaid_tests.rs"]
mod tests;

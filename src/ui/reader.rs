//! The text view (Scouter `src/reader/code-view.ts`): the document's lines in a line-number gutter, coloured by its
//! filetype. It scrolls inside the viewer (`ui/viewer.rs`); wrapping and the width it takes are set here. Unwrapped
//! text keeps its numbers in a gutter outside the scroll, so a pan moves only the text; wrapped text keeps them inside,
//! where the rows of a wrapped line stay with its number.

use std::cell::{Cell, RefCell};
use std::sync::Arc;

use fmc::diff::Sign;
use fmc::reader::document::Document;
use fmc::reader::search::match_ranges;
use fmc::theme::Palette;
use opentui::native::{LayerRun, TextView, Viewport};
use opentui::prelude::*;
use opentui::widget::{LineColor, LineSign};

use super::styled::color;
use super::syntax;

/// The signs a paint drew, and whether the gutters kept the strip's column.
type DrawnSigns = (Vec<(u32, Sign)>, bool);

/// Shown in place of an empty file's lines, with no gutter (`src/reader/code-view.ts:288`).
const EMPTY_FILE: &str = "(empty file)";

/// The layer the search marks go on, drawn over the syntax layer (1), as Scouter's `search.match` group is.
const SEARCH_LAYER: u32 = 2;

pub struct CodeView {
    code: Code,
    /// The numbers beside the text inside the scroll, shown while the text wraps.
    numbers: LineNumbers,
    /// The numbers outside the scroll, shown while the text does not wrap: a line-free copy of the text numbers it,
    /// one row per line, and the gutter moves up with the viewer's scroll (`follow`).
    line_rows: Code,
    gutter: LineNumbers,
    gutter_box: View,
    /// Whether the view is shown, and whether its document has numbers (an empty file has none).
    visible: Cell<bool>,
    numbered: Cell<bool>,
    /// A second view of the code's text, which counts the rows the text takes when it wraps.
    counter: Option<TextView>,
    /// The columns the text takes without wrapping: the pane's width. Scouter's text is pane-wide, so the viewer pans
    /// only past the gutter (six columns), and longer lines are cut at the pane's edge.
    width: Cell<u32>,
    wrap: Cell<bool>,
    /// The search marks last applied, as the query and the current line: skipped while they stay the same.
    applied: RefCell<Option<(String, Option<usize>)>>,
    /// The git signs and the strip flag last drawn (`set_signs`); `None` until the first paint after a show.
    signs: RefCell<Option<DrawnSigns>>,
}

impl CodeView {
    pub fn new(app: &App, p: &Palette) -> Self {
        let code = Code::new(app);
        // Scouter's gutter is a `LineNumberRenderable` with `minWidth: 3` and `paddingRight: 2` (`code-view.ts:99-108`).
        let numbers = LineNumbers::new(app, &code).gutter_min_width(3).gutter_padding(2).visible(false);
        let line_rows = Code::new(app).width(Length::Cells(0.0));
        let gutter = LineNumbers::new(app, &line_rows).gutter_min_width(3).gutter_padding(2);
        let gutter_box = View::new(app).column().overflow(Overflow::Hidden).shrink(0.0).visible(false);
        gutter_box.add(&gutter);
        let counter = TextView::new(code.buffer()).ok();
        let view = Self {
            code,
            numbers,
            line_rows,
            gutter,
            gutter_box,
            visible: Cell::new(false),
            numbered: Cell::new(false),
            counter,
            width: Cell::new(0),
            wrap: Cell::new(false),
            applied: RefCell::new(None),
            signs: RefCell::new(None),
        };
        view.restyle(p);
        view
    }

    pub fn widget(&self) -> LineNumbers {
        self.numbers.clone()
    }

    /// The gutter outside the scroll, for the viewer's gutter slot; it shows while the text does not wrap.
    pub fn gutter(&self) -> View {
        self.gutter_box.clone()
    }

    pub fn restyle(&self, p: &Palette) {
        self.code.fg(color(p.text)).syntax_theme(Arc::new(syntax::theme(p)));
        self.install_table(p);
        self.numbers.fg(color(p.dim));
        self.gutter.fg(color(p.dim));
        self.draw_signs(p);
    }

    /// Shows a document; `language` is its filetype when Scouter highlights it.
    pub fn show(&self, document: &Document, language: Option<&str>) {
        let empty = document.lines == 0;
        self.code.filetype(if empty { None } else { language });
        self.numbered.set(!empty);
        // One row per line, for the gutter outside the scroll; the text's own rows are not needed for its numbers.
        self.line_rows.set_code("\n".repeat(document.lines.saturating_sub(1)));
        self.sync_gutter();
        let content = if empty { EMPTY_FILE } else { document.content() };
        self.code.set_code(content);
        // Setting the text empties the buffer's layers, so the marks are applied again.
        *self.applied.borrow_mut() = None;
        // The next paint draws the signs of this text, so the last text's go now.
        *self.signs.borrow_mut() = None;
        self.apply_width();
    }

    /// The search's marks over the code, and its current line in the gutter (Scouter `markMatches`, `setActiveLine`).
    /// Nothing changes when the query and the line stay the same.
    pub fn sync_search(&self, p: &Palette, query: &str, current: Option<usize>, text: &str) {
        let key = (query.to_owned(), current);
        if self.applied.borrow().as_ref() == Some(&key) {
            return;
        }
        *self.applied.borrow_mut() = Some(key);
        let buffer = self.code.buffer();
        if let Ok(group) = buffer.context().group_set(&["opentui.search.match"]) {
            let runs: Vec<LayerRun> = match_ranges(text, query)
                .into_iter()
                .map(|range| LayerRun { start_byte: range.start as u32, end_byte: range.end as u32, group_set: group })
                .collect();
            let _ = buffer.set_layer(SEARCH_LAYER, 0..u32::MAX, &runs);
        }
        self.numbers.clear_line_colors();
        self.gutter.clear_line_colors();
        if let Some(line) = current {
            let colors = LineColor::new(color(p.target_gutter), color(p.target_bg));
            self.numbers.line_color(line as u32, colors);
            self.gutter.line_color(line as u32, colors);
        }
    }

    /// The git signs of the shown text, and whether the gutters keep the strip's column (`app/gutter.rs`). Each paint
    /// calls it; the gutters are drawn again only when the signs or the strip change.
    pub fn set_signs(&self, p: &Palette, signs: &[(u32, Sign)], strip: bool) {
        if self.signs.borrow().as_ref().is_some_and(|(drawn, on)| drawn.as_slice() == signs && *on == strip) {
            return;
        }
        *self.signs.borrow_mut() = Some((signs.to_vec(), strip));
        self.draw_signs(p);
    }

    /// Draws the stored signs in both gutters. A blank sign on line 1 holds the strip's column while no line has a sign,
    /// so the numbers do not move as signs come and go.
    fn draw_signs(&self, p: &Palette) {
        let stored = self.signs.borrow();
        let Some((signs, strip)) = stored.as_ref() else { return };
        for gutter in [&self.numbers, &self.gutter] {
            gutter.clear_line_signs();
            if *strip {
                gutter.line_sign(0, LineSign::before(" ", None::<Color>));
            }
            for (line, sign) in signs {
                gutter.line_sign(*line, LineSign::before(sign.glyph(), color(sign.color(p))));
            }
        }
    }

    pub fn visible(&self, visible: bool) {
        self.visible.set(visible);
        self.numbers.visible(visible);
        self.sync_gutter();
    }

    /// Moves the outside gutter up with the viewer's scroll, so its rows match the text's (the box clips the rest).
    pub fn follow(&self, row: f64) {
        self.gutter.margin_edge(Edge::Top, Length::Cells(-(row.floor() as f32)));
    }

    /// Which numbers show: beside the text while it wraps, outside it while it does not, and none for an empty file.
    fn sync_gutter(&self) {
        let (wrap, numbered) = (self.wrap.get(), self.numbered.get());
        self.numbers.show_line_numbers(wrap && numbered);
        self.gutter.show_line_numbers(!wrap && numbered);
        self.gutter_box.visible(self.visible.get() && !wrap && numbered);
    }

    /// Wrapping long lines; without it a line runs past the edge and the viewer pans (Scouter `setWrap`).
    pub fn set_wrap(&self, wrap: bool) {
        if self.wrap.replace(wrap) != wrap {
            self.code.wrap(if wrap { WrapMode::Word } else { WrapMode::None });
            self.apply_width();
            self.sync_gutter();
        }
    }

    /// The rows the text takes at the width it has: its lines, or its wrapped rows (Scouter's `scrollHeight`).
    pub fn rows(&self, lines: usize) -> usize {
        let Some(counter) = self.counter.as_ref().filter(|_| self.wrap.get()) else { return lines };
        let width = self.code.element().rect().width.max(1.0) as u32;
        let viewport = Viewport { x: 0, y: 0, width, height: u32::MAX / 2 };
        if counter.set_viewport(viewport).is_err() || counter.set_wrap(WrapMode::Word).is_err() {
            return lines;
        }
        counter.info().map_or(lines, |info| info.total_virtual_line_count as usize)
    }

    /// Sets the pane's text width in columns, the width unwrapped text takes.
    pub fn set_width(&self, columns: u32) {
        if self.width.replace(columns) != columns {
            self.apply_width();
        }
    }

    /// The buffer's style table: the widget's own names, and the search mark (see `syntax::table`).
    fn install_table(&self, p: &Palette) {
        let buffer = self.code.buffer();
        let search = Style::new().fg(color(p.match_fg)).bg(color(p.match_bg)).bold();
        if let Ok(table) = syntax::table(buffer.context(), &syntax::theme(p), search, search) {
            let _ = buffer.set_syntax_style(Some(&table));
        }
    }

    fn apply_width(&self) {
        let width = if self.wrap.get() { Length::percent(100.0) } else { Length::Cells(self.width.get() as f32) };
        self.code.width(width);
    }
}

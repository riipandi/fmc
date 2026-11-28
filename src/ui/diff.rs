//! The stacked diff of the changes screen (Scouter `src/diff/diff-blocks.ts`, and the empty state of
//! `src/diff/diff-screen.ts`): one block per changed file, a header over an OpenTUI `Diff` or a notice, in a scroll box.
//! The text and the layout facts are the core's (`fmc::diff`); this view builds the widgets and measures them after a
//! frame, since a wrapped row count and a block's top exist only once the frame laid them out.

use std::cell::{Cell, RefCell};
use std::sync::Arc;

use fmc::diff::{
    Change, FilePatch, Folds, Hunk, Patch, header_counts, header_title, hunk_starts, stack_columns, widest_line,
};
use fmc::theme::{Hex, Mode, Palette};
use opentui::native::{TextView, Viewport};
use opentui::prelude::*;
use opentui::widget::{DiffColors, DiffGutter, DiffView};

use super::styled::{Seg, color, content};
use super::syntax;
use super::viewer::Viewer;

/// The gutter's columns beside the line-number digits: a space each side, and the two-column sign (Scouter's gutter
/// `paddingLeft` and `paddingRight` of 1, with `SIGN_WIDTH`).
const GUTTER_OUTSIDE_DIGITS: usize = 4;

/// The key the blocks were built from: the listing's session and its generation, the line-number width, and the color
/// mode. The session is needed: a generation counts from zero on each visit, so a later visit could match an old one.
pub type Key = (u64, u64, usize, Mode);

/// A file's block: its header, then its diff or its notice.
struct Block {
    root: View,
    header: View,
    title: Text,
    counts: Text,
    body: Body,
    /// The active and folded state the header last painted (`None` until the first paint).
    painted: Cell<Option<(bool, bool)>>,
    /// The columns the block's diff needs unwrapped: its gutter and its widest line (0 for a notice).
    need: usize,
    /// Counts the wrapped rows of the diff's text, for the hunks' rows under wrap (built when first needed).
    counter: RefCell<Option<TextView>>,
}

enum Body {
    Diff(Diff),
    Notice(Text),
}

pub struct DiffBody {
    /// The body area: the empty state, or the stacked scroll box.
    pub root: View,
    message: View,
    title: Text,
    detail: Text,
    pub scroller: Viewer,
    blocks: RefCell<Vec<Block>>,
    built: Cell<Option<Key>>,
    wrapped: Cell<bool>,
    split: Cell<bool>,
    /// The pane's width in columns, which the stack fills (`set_pane`).
    pane: Cell<u32>,
}

impl DiffBody {
    pub fn new(app: &App) -> Self {
        let root = View::new(app).grow(1.0).min_width(0).min_height(0).column().overflow(Overflow::Hidden);
        let message =
            View::new(app).grow(1.0).min_height(0).column().justify(Justify::Center).align_items(Align::Center);
        let title = Text::new(app, "").height(1).shrink(0.0).wrap(WrapMode::None).selectable(false);
        let detail =
            Text::new(app, "").height(1).shrink(0.0).margin_edge(Edge::Top, 1).wrap(WrapMode::None).selectable(false);
        message.add(&title).add(&detail);
        message.visible(false);
        let scroller = Viewer::new(app);
        // As Scouter's stack: at least the pane's width, and wider when a line is (the pane pans then).
        scroller.widget().content_view().column().min_width(Length::percent(100.0));
        scroller.visible(false);
        root.add(&message).add(&scroller.widget());
        Self {
            root,
            message,
            title,
            detail,
            scroller,
            blocks: RefCell::new(Vec::new()),
            built: Cell::new(None),
            wrapped: Cell::new(false),
            split: Cell::new(false),
            pane: Cell::new(0),
        }
    }

    /// Sizes the blocks to the pane. Scouter's stack is pane-wide and its diff overflows by the gutter; this view
    /// fits the diff's lines instead, so the view pans only over a line that is wider than the pane.
    pub fn set_pane(&self, columns: u32) {
        if self.pane.replace(columns) == columns {
            return;
        }
        self.apply_widths();
    }

    /// Scouter's stack is as wide as its widest visible block, and each header is 100% of its block. Here the stack is
    /// as wide as the widest line of the unfolded diffs, or the pane (`stack_columns`), so a header's counts and stat
    /// bar sit at the pane's edge while every line fits, and past it while one does not. A folded block and a notice
    /// are not counted, and a wrapped body is pane-wide.
    fn apply_widths(&self) {
        let blocks = self.blocks.borrow();
        let pane = self.pane.get() as usize;
        let shown = blocks.iter().map(|block| {
            let folded = block.painted.get().is_none_or(|(_, folded)| folded);
            let diff = matches!(block.body, Body::Diff(_));
            (diff && !folded && !self.wrapped.get()).then_some(block.need)
        });
        let columns = stack_columns(pane, shown);
        let width = if columns > pane { Length::Cells(columns as f32) } else { Length::percent(100.0) };
        for block in blocks.iter() {
            block.root.min_width(width);
        }
    }

    /// Shows the empty state: a title and a detail, centred, with no blocks (Scouter `showEmpty`).
    pub fn empty(&self, title: &str, detail: &str, p: &Palette) {
        self.clear();
        self.title.set_text(content(&[Seg::new(title, p.text).bold()], p.text));
        self.detail.set_text(content(&[Seg::new(detail, p.muted)], p.text));
        self.message.visible(true);
        self.scroller.visible(false);
    }

    /// Builds the blocks of `files` when `key` differs from the blocks shown (Scouter `show`, which reuses them when
    /// the files stay the same).
    /// The key the blocks were last built for; a change of it means the blocks were rebuilt.
    pub fn built_key(&self) -> Option<Key> {
        self.built.get()
    }

    /// `filetype` names each file's highlighting: the `file_associations` setting decides it, as the reader's does.
    pub fn sync(
        &self,
        app: &App,
        key: Key,
        files: &[FilePatch],
        p: &Palette,
        filetype: &dyn Fn(&Change) -> Option<&'static str>,
    ) {
        if self.built.get() == Some(key) {
            return;
        }
        self.clear();
        self.built.set(Some(key));
        let (_, _, digits, _) = key;
        let layout = Layout { digits, wrap: self.wrapped.get(), split: self.split.get() };
        let blocks: Vec<Block> = files
            .iter()
            .enumerate()
            .map(|(index, file)| block(app, index, file, layout, filetype(&file.change), p))
            .collect();
        for block in &blocks {
            self.scroller.widget().content_view().add(&block.root);
        }
        *self.blocks.borrow_mut() = blocks;
        self.message.visible(false);
        self.scroller.visible(true);
    }

    fn clear(&self) {
        for block in std::mem::take(&mut *self.blocks.borrow_mut()) {
            block.root.detach();
        }
        self.built.set(None);
    }

    /// Paints each header's title for the active file and its fold (Scouter `paintHeader`), its tint, and each body's
    /// visibility. A title is set only when its state changed.
    pub fn paint_headers(&self, files: &[FilePatch], active: Option<&str>, folds: &Folds, tint: Hex, p: &Palette) {
        for (block, file) in self.blocks.borrow().iter().zip(files) {
            block.header.bg(color(tint));
            let is_active = active == Some(file.change.path.as_str());
            let folded = folds.contains(&file.change.path);
            if block.painted.get() == Some((is_active, folded)) {
                continue;
            }
            block.painted.set(Some((is_active, folded)));
            block.title.set_text(content(&header_title(&file.change, is_active, folded, p), p.text));
            block.counts.set_text(content(&header_counts(&file.change, p), p.text));
            match &block.body {
                Body::Diff(diff) => {
                    diff.visible(!folded);
                }
                Body::Notice(note) => {
                    note.visible(!folded);
                }
            }
        }
        self.apply_widths();
    }

    /// Sets the view of every diff: side by side when `split`, else one column (Scouter `toggleView`). The blocks built
    /// after it take it too.
    pub fn set_split(&self, split: bool) {
        if self.split.replace(split) == split {
            return;
        }
        let view = if split { DiffView::Split } else { DiffView::Unified };
        for block in self.blocks.borrow().iter() {
            if let Body::Diff(diff) = &block.body {
                diff.view(view);
            }
        }
    }

    /// The block whose header holds the screen row `y`, by the header's last frame (a click, Scouter `onMouseDown`).
    pub fn header_at(&self, y: f64) -> Option<usize> {
        self.blocks.borrow().iter().position(|block| {
            let rect = block.header.rect();
            let (top, height) = (rect.screen_y, f64::from(rect.height));
            height > 0.0 && y >= top && y < top + height
        })
    }

    /// The hunks of the shown blocks, in stack order, from the laid-out rows (Scouter `hunkEntries`): a folded file has
    /// none. Empty until the frame has laid the blocks out.
    pub fn hunks(&self, files: &[FilePatch], folds: &Folds) -> Vec<Hunk> {
        let mut hunks = Vec::new();
        if self.tops().is_none() {
            return hunks;
        }
        for (file, block) in self.blocks.borrow().iter().enumerate() {
            let (Some(patch), Body::Diff(diff)) = (files.get(file), &block.body) else { continue };
            let Patch::Text { text, .. } = &patch.patch else { continue };
            if folds.contains(&patch.change.path) {
                continue;
            }
            let header = block.header.rect().height.max(0.0) as usize;
            let diff_top = block.root.rect().top.max(0.0) as usize + diff.rect().top.max(0.0) as usize;
            let starts = hunk_starts(text);
            for (index, offset) in self.offsets(block, diff).into_iter().enumerate() {
                let row = (diff_top + offset).saturating_sub(if offset == 0 { header } else { 0 });
                hunks.push(Hunk { row, file, line: starts.get(index).copied().unwrap_or(1) });
            }
        }
        hunks
    }

    /// Each hunk's first row in a diff: its logical line, or under wrap its first wrapped row, as Scouter's
    /// `computeHunkRowOffsets` counts (`@opentui/core` `DiffRenderable`, the left code's `lineSources`).
    fn offsets(&self, block: &Block, diff: &Diff) -> Vec<usize> {
        let logical: Vec<usize> = diff.hunk_lines().into_iter().map(|line| line as usize).collect();
        if !self.wrapped.get() || logical.is_empty() {
            return logical;
        }
        let (left, _) = diff.code_views();
        let width = left.rect().width.max(1.0) as u32;
        let mut counter = block.counter.borrow_mut();
        if counter.is_none() {
            *counter = TextView::new(left.buffer()).ok();
        }
        let Some(view) = counter.as_ref() else { return logical };
        let viewport = Viewport { x: 0, y: 0, width, height: u32::MAX / 2 };
        if view.set_viewport(viewport).is_err() || view.set_wrap(WrapMode::Word).is_err() {
            return logical;
        }
        let total = view.info().map_or(0, |info| info.total_virtual_line_count);
        let Ok(rows) = view.lines(0, total) else { return logical };
        logical
            .iter()
            .map(|&line| rows.iter().position(|row| row.source_line as usize >= line).unwrap_or(line))
            .collect()
    }

    /// Sets the wrapping of every block (Scouter `applyWrap`): wrapped lines take the pane's width, unwrapped ones
    /// pan. The scroll position is kept.
    pub fn set_wrap(&self, wrap: bool) {
        if self.wrapped.replace(wrap) == wrap {
            return;
        }
        let shrink = if wrap { 1.0 } else { 0.0 };
        for block in self.blocks.borrow().iter() {
            block.root.shrink(shrink);
            if let Body::Diff(diff) = &block.body {
                diff.wrap(if wrap { WrapMode::Word } else { WrapMode::None }).shrink(shrink);
            }
        }
        self.apply_widths();
        self.scroller.set_wrap(wrap);
        self.scroller.scroll_to(0.0, self.scroller.position().1);
    }

    /// The row each block's header starts on, in the stack. `None` until the frame has laid the blocks out.
    pub fn tops(&self) -> Option<Vec<usize>> {
        let blocks = self.blocks.borrow();
        let first = blocks.first()?;
        if first.header.rect().height <= 0.0 {
            return None;
        }
        Some(blocks.iter().map(|block| block.root.rect().top.max(0.0) as usize).collect())
    }

    /// Whether the stack shows its last row, as the frame laid it out for a view of `viewport` rows. A stack the frame
    /// has not laid out yet, or one that fits, is not at its end: it has nothing to keep there.
    pub fn at_bottom(&self, viewport: usize) -> bool {
        let (_, top) = self.scroller.position();
        let height = self.height() as f64;
        height > viewport as f64 && top + viewport as f64 >= height - 0.5
    }

    /// The columns a block takes, as the frame laid it out: the pane, or wider while a line overflows it.
    #[cfg(test)]
    pub fn stack_width(&self) -> usize {
        self.blocks.borrow().first().map_or(0, |block| block.root.rect().width.max(0.0) as usize)
    }

    /// The rows the stack takes, as the frame laid them out.
    pub fn height(&self) -> usize {
        self.scroller.widget().content_view().rect().height.max(0.0) as usize
    }
}

/// One file's block. Blocks after the first have a blank row above them and a hairline on their header (Scouter
/// `createBlock`, `createHeader`).
/// How the stacked body lays out a file: the line-number digits, and whether lines wrap and the diff splits.
#[derive(Clone, Copy)]
struct Layout {
    digits: usize,
    wrap: bool,
    split: bool,
}

fn block(
    app: &App,
    index: usize,
    file: &FilePatch,
    layout: Layout,
    language: Option<&'static str>,
    p: &Palette,
) -> Block {
    let need = match &file.patch {
        Patch::Text { text, .. } => layout.digits + GUTTER_OUTSIDE_DIGITS + widest_line(text),
        Patch::Notice(_) => 0,
    };
    let root = View::new(app).min_width(Length::percent(100.0)).column().shrink(0.0);
    if index > 0 {
        root.margin_edge(Edge::Top, 1);
    }
    let header = View::new(app).width(Length::percent(100.0)).height(if index > 0 { 2 } else { 1 });
    let header = header.row().shrink(0.0).padding_edge(Edge::Right, 1);
    if index > 0 {
        header.border_sides(Sides::TOP).border_color(color(p.border));
    }
    let title = Text::new(app, "").height(1).grow(1.0).shrink(1.0).min_width(0).wrap(WrapMode::None).selectable(false);
    let counts =
        Text::new(app, "").height(1).shrink(0.0).margin_edge(Edge::Left, 1).wrap(WrapMode::None).selectable(false);
    header.add(&title).add(&counts);
    root.add(&header);
    let body = match &file.patch {
        Patch::Text { text, .. } => {
            let diff = diff(app, language, text, layout, p);
            root.add(&diff);
            Body::Diff(diff)
        }
        Patch::Notice(message) => {
            let notice = Text::new(app, "").height(1).shrink(0.0).margin_edge(Edge::Left, 2).wrap(WrapMode::None);
            notice.set_text(content(&[Seg::new(message.as_str(), p.muted)], p.text));
            root.add(&notice);
            Body::Notice(notice)
        }
    };
    Block { root, header, title, counts, body, painted: Cell::new(None), need, counter: RefCell::new(None) }
}

/// A file's patch as a unified diff, with the file's syntax and the palette's line colors (Scouter `createDiff`).
fn diff(app: &App, language: Option<&'static str>, text: &str, layout: Layout, p: &Palette) -> Diff {
    let Layout { digits, wrap, split } = layout;
    let gutter = DiffGutter { padding_left: 1, min_digits: Some(digits as u32) };
    let diff = Diff::new(app)
        .view(if split { DiffView::Split } else { DiffView::Unified })
        .filetype(language)
        .syntax_theme(Arc::new(syntax::theme(p)))
        .gutter(gutter)
        .colors(colors(p))
        .wrap(if wrap { WrapMode::Word } else { WrapMode::None })
        .shrink(if wrap { 1.0 } else { 0.0 });
    diff.set_diff(text);
    // Scouter's diff text is the palette's text color, not the syntax theme's default (`src/diff/diff-blocks.ts`).
    let (left, right) = diff.code_views();
    left.fg(color(p.text));
    right.fg(color(p.text));
    diff
}

fn colors(p: &Palette) -> DiffColors {
    DiffColors {
        line_number: color(p.dim),
        line_number_bg: Color::TRANSPARENT,
        added_bg: color(p.added_bg),
        removed_bg: color(p.removed_bg),
        context_bg: Color::TRANSPARENT,
        added_content_bg: None,
        removed_content_bg: None,
        context_content_bg: None,
        added_sign: color(p.added),
        removed_sign: color(p.removed),
        added_line_number_bg: color(p.added_gutter),
        removed_line_number_bg: color(p.removed_gutter),
    }
}

#[cfg(test)]
#[path = "diff_tests.rs"]
mod tests;

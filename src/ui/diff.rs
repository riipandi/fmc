//! The stacked diff of the changes screen (Scouter `src/diff/diff-blocks.ts`, and the empty state of
//! `src/diff/diff-screen.ts`): one block per changed file, a header over an OpenTUI `Diff` or a notice, in a scroll box.
//! The text and the layout facts are the core's (`fmc::diff`); this view builds the widgets and measures them after a
//! frame, since a wrapped row count and a block's top exist only once the frame laid them out.

use std::cell::{Cell, RefCell};
use std::path::Path;
use std::sync::Arc;

use fmc::diff::{Change, FilePatch, Patch, header_counts, header_title};
use fmc::theme::{Mode, Palette};
use opentui::prelude::*;
use opentui::widget::{DiffColors, DiffGutter};

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
    /// Whether the title last painted names the active file (`None` until the first paint).
    active: Cell<Option<bool>>,
    /// The line-number width the block was built with, which its gutter takes.
    digits: Cell<usize>,
}

enum Body {
    Diff(Diff),
    Notice,
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
            pane: Cell::new(0),
        }
    }

    /// Sizes the blocks to the pane. Unwrapped, a block is as wide as the pane plus its diff's gutter: Scouter's diff
    /// overflows the pane by that gutter, so the view pans by it (`src/diff/diff-blocks.ts` `gutter`).
    pub fn set_pane(&self, columns: u32) {
        if self.pane.replace(columns) == columns {
            return;
        }
        for block in self.blocks.borrow().iter() {
            block.root.min_width(Length::Cells(self.block_width(block.digits.get())));
        }
    }

    fn block_width(&self, digits: usize) -> f32 {
        let gutter = if self.wrapped.get() { 0 } else { digits + GUTTER_OUTSIDE_DIGITS };
        (self.pane.get() as usize + gutter) as f32
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
    pub fn sync(&self, app: &App, key: Key, files: &[FilePatch], p: &Palette) {
        if self.built.get() == Some(key) {
            return;
        }
        self.clear();
        self.built.set(Some(key));
        let (_, _, digits, _) = key;
        let wrap = self.wrapped.get();
        let blocks: Vec<Block> =
            files.iter().enumerate().map(|(index, file)| block(app, index, file, digits, wrap, p)).collect();
        for block in &blocks {
            block.root.min_width(Length::Cells(self.block_width(digits)));
        }
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

    /// Paints each header's title for the active file (Scouter `paintHeader`) and its tint. A title is set only when
    /// its active state changed.
    pub fn paint_headers(&self, files: &[FilePatch], active: Option<&str>, tint: &str, p: &Palette) {
        for (block, file) in self.blocks.borrow().iter().zip(files) {
            block.header.bg(color(tint));
            let is_active = active == Some(file.change.path.as_str());
            if block.active.get() == Some(is_active) {
                continue;
            }
            block.active.set(Some(is_active));
            block.title.set_text(content(&header_title(&file.change, is_active, p), p.text));
            block.counts.set_text(content(&header_counts(&file.change, p), p.text));
        }
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
            block.root.min_width(Length::Cells(self.block_width(block.digits.get())));
            if let Body::Diff(diff) = &block.body {
                diff.wrap(if wrap { WrapMode::Word } else { WrapMode::None }).shrink(shrink);
            }
        }
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

    /// The row each hunk starts on, in the stack, as Scouter's `hunkEntries` puts it: a hunk at the top of a diff starts
    /// on its header (`src/diff/diff-blocks.ts:552-570`).
    pub fn hunk_rows(&self) -> Vec<usize> {
        let mut rows = Vec::new();
        for block in self.blocks.borrow().iter() {
            let Body::Diff(diff) = &block.body else { continue };
            let header = block.header.rect().height.max(0.0) as usize;
            let block_top = block.root.rect().top.max(0.0) as usize;
            let diff_top = block_top + diff.rect().top.max(0.0) as usize;
            for offset in diff.hunk_lines() {
                let offset = offset as usize;
                rows.push((diff_top + offset).saturating_sub(if offset == 0 { header } else { 0 }));
            }
        }
        rows
    }

    /// The rows the stack takes, as the frame laid them out.
    pub fn height(&self) -> usize {
        self.scroller.widget().content_view().rect().height.max(0.0) as usize
    }
}

/// One file's block. Blocks after the first have a blank row above them and a hairline on their header (Scouter
/// `createBlock`, `createHeader`).
fn block(app: &App, index: usize, file: &FilePatch, digits: usize, wrap: bool, p: &Palette) -> Block {
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
            let diff = diff(app, &file.change, text, digits, wrap, p);
            root.add(&diff);
            Body::Diff(diff)
        }
        Patch::Notice(message) => {
            let notice = Text::new(app, "").height(1).shrink(0.0).margin_edge(Edge::Left, 2).wrap(WrapMode::None);
            notice.set_text(content(&[Seg::new(message.as_str(), p.muted)], p.text));
            root.add(&notice);
            Body::Notice
        }
    };
    Block { root, header, title, counts, body, active: Cell::new(None), digits: Cell::new(digits) }
}

/// A file's patch as a unified diff, with the file's syntax and the palette's line colors (Scouter `createDiff`).
fn diff(app: &App, change: &Change, text: &str, digits: usize, wrap: bool, p: &Palette) -> Diff {
    let gutter = DiffGutter { padding_left: 1, min_digits: Some(digits as u32) };
    let diff = Diff::new(app)
        .filetype(syntax::language_for(Path::new(&change.path)))
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

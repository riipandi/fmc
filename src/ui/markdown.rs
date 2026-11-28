//! The rendered Markdown view (Scouter `src/reader/code-view.ts`, the `MarkdownRenderable` of `preview`): OpenTUI's
//! Markdown widget with the syntax styles of the palette, the text colour, and two cells of padding on each side.
//! Mermaid fences are drawn as diagrams (`ui/mermaid.rs`); Scouter draws them as code. Search marks (`ui/markdown_search.rs`)
//! go through the inline hook and the code renderers, which the view registers once and rebuilds for each change.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::time::SystemTime;

use fmc::reader::headings;
use fmc::reader::images::{self, Header, Image};
use fmc::reader::language::HIGHLIGHTED;
use fmc::reader::rendered_search::Rendered;
use fmc::theme::{Hex, Palette};
use opentui::native::Context;
use opentui::prelude::*;
use opentui::widget::{Markdown, MarkdownColors};

use super::image_box;
use super::markdown_blocks;
use super::markdown_images;
use super::markdown_pictures::{self, Draw, Pictures, Slot};
use super::markdown_search::{
    Captured, Mark, Paint, Styles, capture_code, capture_prose, count_code, index, paint_code, paint_prose,
};
use super::mermaid::{Colors, Diagram};
use super::styled::color;
use super::syntax;

/// The source lines of each top-level block, shared with the view's cache.
type BlockLines = Rc<Vec<Range<usize>>>;

pub struct MarkdownView {
    doc: Markdown,
    /// Builds each block of a document on its own, for the search index; never shown.
    offscreen: Markdown,
    paint: Rc<Paint>,
    captured: Rc<RefCell<Vec<Captured>>>,
    /// The index of the last source the view indexed, so a search step does not index it again.
    index: RefCell<Option<(String, Rc<Rendered>)>>,
    /// The source lines of each top-level block of the last source, for the git signs.
    lines: RefCell<Option<(String, BlockLines)>>,
    /// The headers the boxes draw, by destination as written, from the worker (`app/picture.rs`).
    headers: Rc<RefCell<HashMap<String, Header>>>,
    /// The boxes of the shown source: each one's block index and image.
    boxes: RefCell<Vec<(usize, Image)>>,
    /// The header a box picked on the last release (`ui/image_box.rs`), until the app takes it.
    pick: Rc<RefCell<Option<Header>>>,
    /// The decoded pictures the boxes draw (`ui/markdown_pictures.rs`), kept across rebuilds.
    pictures: Rc<RefCell<Pictures>>,
    /// The context the native library decodes a picture in; the UI thread owns it.
    context: Context,
    /// Whether the dark theme is shown: a `<picture>` takes its dark source then (`reader/images.rs` `target_for`).
    dark: Rc<Cell<bool>>,
    /// The most rows a picture takes, from the viewer's height (`fit`).
    cap: Rc<Cell<u32>>,
    /// The columns the text takes, which a picture takes at most (`fit`).
    pane: Rc<Cell<u32>>,
}

impl MarkdownView {
    pub fn new(app: &App, p: &Palette, dark: bool) -> Self {
        let doc = Markdown::new(app).padding_x(2).visible(false);
        let offscreen = Markdown::new(app);
        let captured = Rc::new(RefCell::new(Vec::new()));
        register_capture(&offscreen, &captured);
        let view = Self {
            doc,
            offscreen,
            paint: Rc::new(Paint::default()),
            captured,
            index: RefCell::new(None),
            lines: RefCell::new(None),
            headers: Rc::new(RefCell::new(HashMap::new())),
            boxes: RefCell::new(Vec::new()),
            pick: Rc::new(RefCell::new(None)),
            pictures: Rc::new(RefCell::new(Pictures::default())),
            context: app.session().context().clone(),
            dark: Rc::new(Cell::new(dark)),
            cap: Rc::new(Cell::new(markdown_pictures::row_cap(0))),
            pane: Rc::new(Cell::new(0)),
        };
        view.restyle(p, dark);
        view
    }

    pub fn widget(&self) -> Markdown {
        self.doc.clone()
    }

    /// Scouter's prose takes the syntax styles (`scopeStyles`); the colours are the fallbacks for scopes the theme
    /// leaves out, which Core draws in the text colour. Inline and block code have no background there.
    pub fn restyle(&self, p: &Palette, dark: bool) {
        self.dark.set(dark);
        let text = color(p.text);
        let colors = MarkdownColors {
            text,
            heading: color(p.syntax.function),
            emphasis: text,
            code: color(p.syntax.string),
            code_bg: opentui::prelude::Color::TRANSPARENT,
            link: color(p.syntax.function),
            quote: text,
            marker: text,
            // Core draws table and rule borders in this grey when the theme has no `conceal` scope (`index.bun.js:11729`).
            rule: color(Hex::new("#888888")),
        };
        let theme = Arc::new(syntax::theme(p));
        self.doc.colors(colors).syntax_theme(Arc::clone(&theme)).scope_styles(true);
        *self.paint.styles.borrow_mut() = Some(Styles {
            text,
            theme,
            matched: Style::new().fg(color(p.match_fg)).bg(color(p.match_bg)).bold(),
            current: Style::new().fg(text).bg(color(p.target_bg)).bold(),
        });
        // An empty fence keeps the default code block; the renderer is registered again on each restyle, which replaces it.
        let diagram = Colors::new(p);
        let paint = Rc::clone(&self.paint);
        self.doc.code_block_renderer("mermaid", move |block| {
            count_code(&paint);
            (!block.code.trim().is_empty())
                .then(|| Custom::measured(block.app, Diagram::new(block.code, diagram)).as_any())
        });
        for language in HIGHLIGHTED {
            let paint = Rc::clone(&self.paint);
            self.doc.code_block_renderer(language, move |block| paint_code(block, &paint));
        }
        // A box fence (`reader/images.rs`): its picture or its box, from the header the worker read, with no I/O here. It
        // counts its fence like the search index does, so the slots of the marks hold (`markdown_search.rs`).
        let (paint, headers, colors) = (Rc::clone(&self.paint), Rc::clone(&self.headers), image_box::Colors::new(p));
        let (pick, pictures, cap, dark, pane) = (
            Rc::clone(&self.pick),
            Rc::clone(&self.pictures),
            Rc::clone(&self.cap),
            Rc::clone(&self.dark),
            Rc::clone(&self.pane),
        );
        self.doc.code_block_renderer(images::FENCE, move |block| {
            count_code(&paint);
            let image = images::from_info(block.info)?;
            let target = images::target_for(&image, dark.get());
            let header = headers.borrow().get(target).cloned();
            let draw = Draw {
                image: &image,
                target,
                header: header.as_ref(),
                pictures: pictures.as_ref(),
                colors,
                pane: pane.get(),
                cap: cap.get(),
                pick: Rc::clone(&pick),
            };
            Some(markdown_pictures::widget(block.app, &draw))
        });
        self.apply();
    }

    /// Installs the hook that marks prose, and rebuilds the document, so every block is built in document order from
    /// zero: the slots the mark names are the ones this build numbers.
    fn apply(&self) {
        self.paint.restart();
        let paint = Rc::clone(&self.paint);
        self.doc.inline_hook(move |spans, _| paint_prose(spans, &paint));
    }

    /// Shows `markdown` with no marks: the search re-applies its own (`search::sync`, at each paint), since the marks
    /// name the document they were counted in.
    pub fn show(&self, markdown: &str) {
        let marked = self.paint.mark.borrow_mut().take().is_some();
        self.paint.restart();
        let prepared = markdown_images::prepared(markdown);
        *self.boxes.borrow_mut() = markdown_images::boxes(&prepared, &markdown_blocks::split(&prepared));
        // The headers name the last document's folder, so they go with it; the worker answers for this one.
        self.headers.borrow_mut().clear();
        self.doc.set_markdown(headings::with_markers(&prepared));
        // A block the document keeps from the one shown keeps its marks, so the marked document is built again.
        if marked {
            self.apply();
        }
    }

    /// Each box's top row in the document and its header, in document order. Empty until a frame lays the document out.
    pub fn boxes(&self) -> Vec<(u32, Option<Header>)> {
        let tops = self.block_tops();
        let headers = self.headers.borrow();
        let dark = self.dark.get();
        self.boxes
            .borrow()
            .iter()
            .filter_map(|(block, image)| {
                let (top, _) = tops.get(*block)?;
                Some((*top, headers.get(images::target_for(image, dark)).cloned()))
            })
            .collect()
    }

    /// The pictures the boxes draw that the cache lacks, by path and modification time, each once: the app reads them.
    pub fn wanted(&self) -> Vec<(PathBuf, Option<SystemTime>)> {
        let (headers, pictures, boxes, dark) =
            (self.headers.borrow(), self.pictures.borrow(), self.boxes.borrow(), self.dark.get());
        let mut wanted: Vec<(PathBuf, Option<SystemTime>)> = Vec::new();
        for (_, image) in boxes.iter() {
            let Some(Header::Picture { path, modified, .. }) = headers.get(images::target_for(image, dark)) else {
                continue;
            };
            if !pictures.contains(path, *modified) && !wanted.iter().any(|(of, _)| of == path) {
                wanted.push((path.clone(), *modified));
            }
        }
        wanted
    }

    /// Decodes the bytes a worker read for `path` at `modified`, and draws the boxes again. A decode that fails keeps the box.
    pub fn decode(&self, path: &Path, modified: Option<SystemTime>, bytes: &[u8]) {
        let slot = markdown_pictures::decode(&self.context, bytes);
        self.store(path, modified, slot);
    }

    /// Records that the bytes of `path` could not be read, so its box keeps the reason.
    pub fn unreadable(&self, path: &Path, modified: Option<SystemTime>, reason: &'static str) {
        self.store(path, modified, Slot::Failed(reason));
    }

    fn store(&self, path: &Path, modified: Option<SystemTime>, slot: Slot) {
        self.pictures.borrow_mut().insert(path, modified, slot);
        self.apply();
    }

    /// Sets the text's columns and the viewer's rows, which bound a picture (`markdown_pictures::Draw`). A change draws the
    /// boxes again.
    pub fn fit(&self, columns: usize, viewer_rows: usize) {
        let (pane, cap) = (columns as u32, markdown_pictures::row_cap(viewer_rows as u32));
        if self.pane.get() != pane || self.cap.get() != cap {
            self.pane.set(pane);
            self.cap.set(cap);
            self.apply();
        }
    }

    /// The header the last release over a box picked, if any; taking it clears it.
    pub fn take_pick(&self) -> Option<Header> {
        self.pick.borrow_mut().take()
    }

    /// The headers the boxes draw, and draws them again: a box's rows follow its header.
    pub fn set_headers(&self, headers: Vec<(String, Header)>) {
        *self.headers.borrow_mut() = headers.into_iter().collect();
        self.apply();
    }

    /// Sets the search marks the view draws (`None` clears them). Each change rebuilds the document.
    pub fn mark(&self, mark: Option<Mark>) {
        if *self.paint.mark.borrow() == mark {
            return;
        }
        *self.paint.mark.borrow_mut() = mark;
        self.apply();
    }

    /// The rendered text of `source`, one piece per block leaf, as the marks count it. Cached for the last source.
    pub fn index(&self, source: &str) -> Rc<Rendered> {
        let cached =
            self.index.borrow().as_ref().filter(|(of, _)| of == source).map(|(_, rendered)| Rc::clone(rendered));
        if let Some(rendered) = cached {
            return rendered;
        }
        let rendered = Rc::new(index(&self.offscreen, &self.captured, source));
        *self.index.borrow_mut() = Some((source.to_owned(), Rc::clone(&rendered)));
        rendered
    }

    /// The source lines of each top-level block of `source`, cached for the last source (`ui/markdown_blocks.rs`).
    pub fn block_lines(&self, source: &str) -> BlockLines {
        let cached = self.lines.borrow().as_ref().filter(|(of, _)| of == source).map(|(_, lines)| Rc::clone(lines));
        if let Some(lines) = cached {
            return lines;
        }
        let lines = Rc::new(
            markdown_blocks::split(&markdown_images::prepared(source))
                .blocks
                .into_iter()
                .map(|block| block.lines)
                .collect::<Vec<_>>(),
        );
        *self.lines.borrow_mut() = Some((source.to_owned(), Rc::clone(&lines)));
        lines
    }

    /// Each top-level block's top row and rows in the document, in order. `block_rows` clamps a block scrolled above the
    /// view to row 0 but keeps its height, and the blocks stand one row apart, so the tops follow from the heights. Empty
    /// until a frame lays the document out.
    pub fn block_tops(&self) -> Vec<(u32, u32)> {
        // Before a frame lays the document out, every block is zero rows, so there are no tops yet.
        if self.rows() == 0 {
            return Vec::new();
        }
        let mut top = 0;
        self.doc
            .block_rows()
            .into_iter()
            .map(|rows| {
                let height = rows.end - rows.start;
                let out = (top, height);
                top += height + 1;
                out
            })
            .collect()
    }

    pub fn visible(&self, visible: bool) {
        self.doc.visible(visible);
    }

    /// The rows the rendered document takes in the latest frame: the footer's scroll position and the bar count them.
    /// The block rows are screen rows, which move up as the view scrolls, so the widget's own height is the total.
    pub fn rows(&self) -> usize {
        self.doc.rect().height.max(0.0) as usize
    }
}

/// Registers the offscreen index's hook and renderers: every language the shown view renders, the diagram, and the box.
fn register_capture(offscreen: &Markdown, captured: &Rc<RefCell<Vec<Captured>>>) {
    let hook = Rc::clone(captured);
    offscreen.inline_hook(move |spans, _| capture_prose(spans, &hook));
    for language in std::iter::once("mermaid").chain(std::iter::once(images::FENCE)).chain(HIGHLIGHTED.iter().copied())
    {
        let captured = Rc::clone(captured);
        offscreen.code_block_renderer(language, move |block| capture_code(block, &captured));
    }
}

#[cfg(test)]
#[path = "markdown_tests.rs"]
mod tests;

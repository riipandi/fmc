//! The pictures of the rendered view's box fences (`reader/images.rs`). A box draws its picture once the picture is decoded,
//! and its box until then. The native library decodes a picture only on the thread that owns its context, so the UI
//! thread decodes each file once, from the bytes a worker read (`app/picture.rs`). The decoded handles are kept per file
//! and modification time, so a rebuild of the document takes them again without a decode.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::SystemTime;

use fmc::reader::images::{self, Header, Image};
use opentui::native::{Context, Image as Decoded, ImageFit, ImageProtocol};
use opentui::prelude::*;
use opentui::widget::{ImageView, View};

use super::image_box::{self, ImageBox, ROWS, Size};

/// The most decoded pictures kept; the least recently drawn goes first.
pub const CAPACITY: usize = 32;

/// The most rows a picture takes: about three fifths of the viewer's rows, and never less than a box.
pub fn row_cap(viewer_rows: u32) -> u32 {
    (viewer_rows * 3 / 5).max(ROWS as u32)
}

/// What a picture came to: its decoded handle, or the reason it has none.
#[derive(Clone)]
pub enum Slot {
    Picture(Decoded),
    Failed(&'static str),
}

/// The pictures decoded so far, by path and modification time; the least recently drawn is first.
#[derive(Default)]
pub struct Pictures {
    entries: Vec<(Key, Slot)>,
}

#[derive(Clone, PartialEq, Eq)]
struct Key {
    path: PathBuf,
    modified: Option<SystemTime>,
}

impl Pictures {
    pub fn contains(&self, path: &Path, modified: Option<SystemTime>) -> bool {
        self.position(path, modified).is_some()
    }

    /// The slot of a picture, which becomes the most recently drawn.
    pub fn get(&mut self, path: &Path, modified: Option<SystemTime>) -> Option<Slot> {
        let entry = self.entries.remove(self.position(path, modified)?);
        let slot = entry.1.clone();
        self.entries.push(entry);
        Some(slot)
    }

    pub fn insert(&mut self, path: &Path, modified: Option<SystemTime>, slot: Slot) {
        if let Some(at) = self.position(path, modified) {
            self.entries.remove(at);
        }
        self.entries.push((Key { path: path.to_path_buf(), modified }, slot));
        if self.entries.len() > CAPACITY {
            self.entries.remove(0);
        }
    }

    fn position(&self, path: &Path, modified: Option<SystemTime>) -> Option<usize> {
        self.entries.iter().position(|(key, _)| key.path == path && key.modified == modified)
    }
}

/// The decoded picture in `bytes`, or the reason it has none.
pub fn decode(context: &Context, bytes: &[u8]) -> Slot {
    match Decoded::decode(context, bytes) {
        Ok(picture) => Slot::Picture(picture),
        Err(_) => Slot::Failed(images::DECODE_FAILED),
    }
}

/// What one box fence draws: its image as written, the target and header the theme chose, and the cache and the viewer's
/// row cap it reads.
pub struct Draw<'a> {
    pub image: &'a Image,
    pub target: &'a str,
    pub header: Option<&'a Header>,
    pub pictures: &'a RefCell<Pictures>,
    pub colors: image_box::Colors,
    /// The columns the text takes (`MarkdownView::fit`); a picture takes no more, and 0 means the pane is not known yet.
    pub pane: u32,
    pub cap: u32,
    pub pick: Rc<RefCell<Option<Header>>>,
}

/// The widget of one box fence: its picture when the cache holds it, else its box. The box keeps the picture's rows while the
/// picture is read, so the lines below do not move when it lands.
pub fn widget(app: &App, draw: &Draw<'_>) -> AnyWidget {
    let cell_aspect = app.session().cell_aspect() as f32;
    let info = match draw.header {
        Some(Header::Picture { info, .. }) => Some(info),
        _ => None,
    };
    let natural = draw.image.width.or(info.map(|info| info.width)).map(images::columns);
    let columns = match (natural, draw.pane) {
        (Some(natural), 0) => Some(natural),
        (Some(natural), pane) => Some(natural.min(pane)),
        (None, pane) => (pane > 0).then_some(pane),
    };
    let slot = match draw.header {
        Some(Header::Picture { path, modified, .. }) => draw.pictures.borrow_mut().get(path, *modified),
        _ => None,
    };
    match slot {
        Some(Slot::Picture(picture)) => match picture.info() {
            Ok(decoded) => {
                let ratio = decoded.width as f32 / decoded.height as f32 * cell_aspect;
                let size = Size { columns, ratio: Some(ratio), cap: draw.cap };
                picture_widget(app, draw, &picture, size)
            }
            Err(_) => boxed(app, draw, Some(Header::Refused(images::DECODE_FAILED)), columns, None),
        },
        Some(Slot::Failed(reason)) => boxed(app, draw, Some(Header::Refused(reason)), columns, None),
        None => {
            let ratio =
                info.filter(|info| info.height > 0).map(|info| info.width as f32 / info.height as f32 * cell_aspect);
            boxed(app, draw, draw.header.cloned(), columns, ratio)
        }
    }
}

/// A decoded picture, fitted into its rows and columns; a click on it picks its header, as on a box (`app/mouse.rs`).
fn picture_widget(app: &App, draw: &Draw<'_>, picture: &Decoded, size: Size) -> AnyWidget {
    let view = ImageView::new(app).fit(ImageFit::Contain).protocol(ImageProtocol::Auto);
    view.image(Some(picture.clone()));
    size.apply(&view);
    let (pick, header) = (Rc::clone(&draw.pick), draw.header.cloned());
    view.on_mouse(move |_, event| {
        if event.kind == MouseEventKind::Up(MouseButton::Left) {
            *pick.borrow_mut() = header.clone();
        }
    });
    in_block(app, &view, draw.image.center)
}

/// The box of `header`: its card names the target, and its rows follow `ratio` when a picture is expected.
fn boxed(app: &App, draw: &Draw<'_>, header: Option<Header>, columns: Option<u32>, ratio: Option<f32>) -> AnyWidget {
    let size = Size { columns, ratio, cap: draw.cap };
    let card = images::card(draw.image, draw.target, header.as_ref());
    let widget = Custom::measured(app, ImageBox::new(card, size, draw.colors, header, Rc::clone(&draw.pick)));
    size.apply(&widget);
    in_block(app, &widget, draw.image.center)
}

/// The widget in its block: an image written as centred sits in the middle of its row.
fn in_block(app: &App, widget: &impl Widget, center: bool) -> AnyWidget {
    if !center {
        return widget.as_any();
    }
    View::new(app).row().justify(Justify::Center).child(widget).as_any()
}

#[cfg(test)]
#[path = "markdown_pictures_tests.rs"]
mod tests;

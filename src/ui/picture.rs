//! The image view (Scouter `src/reader/image-view.ts`): the picture scaled to fit the reader, drawn by OpenTUI in the
//! best protocol the terminal supports (Kitty, Sixel, or blocks, as Scouter's `protocol: "auto"`).

use opentui::native::{ImageFit, ImageProtocol};
use opentui::prelude::*;
use opentui::widget::ImageView;

pub struct Picture {
    view: ImageView,
}

impl Picture {
    pub fn new(app: &App) -> Self {
        let view = ImageView::new(app).fit(ImageFit::Contain).protocol(ImageProtocol::Auto).visible(false);
        Self { view }
    }

    pub fn widget(&self) -> ImageView {
        self.view.clone()
    }

    /// Decodes the picture for display. A decode failure leaves the area blank, as Scouter's does.
    pub fn show(&self, bytes: &[u8]) {
        let _ = self.view.load(bytes);
    }

    /// The picture fills `rows` rows, the reader's viewport height (Scouter `image.height = viewport.height`).
    pub fn fill(&self, rows: u32) {
        self.view.height(rows);
    }

    pub fn visible(&self, visible: bool) {
        self.view.visible(visible);
    }
}

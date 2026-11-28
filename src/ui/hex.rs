//! The hex view: the rows the screen shows, painted as one grid of cells. It holds no file and no scroll box; the app
//! hands it the visible rows at each paint (`app/hex.rs`), so a file of any size costs one screen of cells.

use fmc::reader::hex::{Class, Ink};
use fmc::theme::Palette;
use opentui::prelude::*;

use super::styled::color;

#[derive(Default)]
pub struct Grid {
    width: u32,
    cells: Vec<PaintCell>,
}

impl Component for Grid {
    fn paint(&self, painter: &mut Painter<'_>) {
        painter.cells(0, 0, self.width, &self.cells);
    }
}

pub struct HexView {
    root: Custom<Grid>,
}

impl HexView {
    pub fn new(app: &App) -> Self {
        let root = Custom::new(app, Grid::default()).grow(1.0).min_width(0).min_height(0).visible(false);
        Self { root }
    }

    pub fn widget(&self) -> Custom<Grid> {
        self.root.clone()
    }

    pub fn visible(&self, visible: bool) {
        self.root.visible(visible);
    }

    /// Draws `rows`, each the same number of cells, in the palette's colours for each byte class.
    pub fn show(&self, rows: &[Vec<(char, Ink)>], p: &Palette) {
        let width = rows.first().map_or(0, Vec::len) as u32;
        let ink = Inks::new(p);
        let cells =
            rows.iter().flatten().map(|&(character, kind)| PaintCell::new(character, ink.style(kind))).collect();
        self.root.update(|grid| *grid = Grid { width, cells });
    }
}

/// The styles of one paint, parsed once from the palette.
struct Inks {
    offset: Style,
    blank: Style,
    classes: [Style; 5],
    hit: Style,
}

impl Inks {
    fn new(p: &Palette) -> Self {
        let fg = |hex| Style::new().fg(color(hex));
        Self {
            offset: fg(p.dim),
            blank: Style::new(),
            classes: [fg(p.ignored), fg(p.text), fg(p.added), fg(p.accent_soft), fg(p.modified)],
            hit: Style::new().fg(color(p.match_fg)).bg(color(p.match_bg)),
        }
    }

    fn style(&self, ink: Ink) -> Style {
        match ink {
            Ink::Blank | Ink::Gap { hit: false } => self.blank,
            Ink::Offset => self.offset,
            Ink::Byte { hit: true, .. } | Ink::Gap { hit: true } => self.hit,
            Ink::Byte { class, .. } => self.classes[class_index(class)],
        }
    }
}

fn class_index(class: Class) -> usize {
    match class {
        Class::Null => 0,
        Class::Printable => 1,
        Class::Whitespace => 2,
        Class::Control => 3,
        Class::High => 4,
    }
}

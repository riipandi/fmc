//! The empty reader (Scouter `src/ui/welcome.ts`): the wordmark, the tagline, a key card, and a closing line.

use fmc::theme::Palette;
use opentui::prelude::*;

use super::styled::{Seg, color, content};

/// `FMC` in the three-row block letters of Scouter's wordmark.
const WORDMARK: [&str; 3] = ["█▀▀ █▀▄▀█ █▀▀", "█▀▀ █ ▀ █ █  ", "▀   ▀   ▀ ▀▀▀"];

const HINTS: [(&str, &str); 5] = [
    ("j k ↑ ↓", "move"),
    ("Enter", "open file or folder"),
    ("/", "find a file"),
    ("d", "changes in a git repository"),
    ("^q", "quit"),
];

const KEY_COLUMN_WIDTH: usize = 7;

fn line(app: &App, segments: &[Seg], p: &Palette, margin_top: u16) -> Text {
    Text::new(app, content(segments, p.text))
        .height(1)
        .shrink(0.0)
        .margin_edge(Edge::Top, margin_top)
        .wrap(WrapMode::None)
        .selectable(false)
}

/// The welcome screen: its key card, and the three lines that wrap below `NARROW_WELCOME` columns.
pub struct Welcome {
    pub root: View,
    card: View,
    lines: Vec<Text>,
}

impl Welcome {
    /// Fits the welcome to `columns`. Below `NARROW_WELCOME` it drops the card and wraps its lines instead of cutting
    /// them (plan-00004, P5); at 50 columns and wider each line keeps one row, as Scouter draws it.
    pub fn fit(&self, columns: u16) {
        let narrow = columns < fmc::layout::NARROW_WELCOME;
        self.card.visible(!narrow);
        let (wrap, height) = if narrow { (WrapMode::Word, Length::Auto) } else { (WrapMode::None, Length::Cells(1.0)) };
        for line in &self.lines {
            line.wrap(wrap);
            line.height(height);
        }
    }
}

/// Builds the welcome view in the palette's colors; the app rebuilds it when the theme changes.
pub fn welcome(app: &App, p: &Palette) -> Welcome {
    let view = View::new(app)
        .grow(1.0)
        .min_height(0)
        .min_width(0)
        .column()
        .align_items(Align::Center)
        .justify(Justify::Center)
        .overflow(Overflow::Hidden);
    for (row, fg) in WORDMARK.iter().zip(p.wordmark) {
        view.add(&line(app, &[Seg::new(*row, fg)], p, 0));
    }
    let tagline = [
        Seg::new("a quiet, read-only file viewer", p.muted),
        Seg::new(format!("  v{}", env!("CARGO_PKG_VERSION")), p.dim),
    ];
    let tagline = line(app, &tagline, p, 1);
    let prompt = line(app, &[Seg::new("Select a file and press Enter.", p.accent_soft)], p, 1);
    view.add(&tagline).add(&prompt);
    let card = View::new(app)
        .column()
        .shrink(0.0)
        .margin_edge(Edge::Top, 1)
        .padding_x(2)
        .border(BorderStyle::Rounded)
        .border_color(color(p.border));
    for (key, action) in HINTS {
        let row = [
            Seg::new(format!("{key:>KEY_COLUMN_WIDTH$}"), p.accent_soft).bold(),
            Seg::new("  │  ", p.border),
            Seg::new(action, p.text),
        ];
        card.add(&line(app, &row, p, 0));
    }
    view.add(&card);
    let closing = [
        Seg::new("Press ? for every key.", p.accent_soft),
        Seg::new("  ·  ", p.ignored),
        Seg::new("No editing. Just reading.", p.dim),
    ];
    let closing = line(app, &closing, p, 1);
    view.add(&closing);
    Welcome { root: view, card, lines: vec![tagline, prompt, closing] }
}

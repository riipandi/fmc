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

const PROMPT: &str = "Select a file and press Enter.";
/// The prompt while the list is hidden below 80 columns: it names the menu button (plan-00004, amended).
const MENU_PROMPT: &str = "Tap ≡ to browse files.";

/// The welcome screen: its key card, and its lines, which wrap below `NARROW_WELCOME` columns.
pub struct Welcome {
    pub root: View,
    card: View,
    prompt: Text,
    /// The closing line in one row, and the same line broken at its ` · ` below 80 columns (plan-00004, P5, amended).
    closing: Text,
    closing_parts: [Text; 2],
    tagline: Text,
}

impl Welcome {
    /// Fits the welcome to `columns`. Below `NARROW_WELCOME` it drops the card and wraps its lines instead of cutting
    /// them (plan-00004, P5); at 50 columns and wider each line keeps one row, as Scouter draws it. `menu` is whether the
    /// list is hidden below 80 columns, which swaps the prompt for the menu's.
    pub fn fit(&self, columns: u16, menu: bool, p: &Palette) {
        let narrow = columns < fmc::layout::NARROW_WELCOME;
        let phone = fmc::layout::tier(columns) != fmc::layout::Tier::Wide;
        self.card.visible(!narrow);
        self.closing.visible(!phone);
        for part in &self.closing_parts {
            part.visible(phone);
        }
        let prompt = if menu { MENU_PROMPT } else { PROMPT };
        self.prompt.set_text(content(&[Seg::new(prompt, p.accent_soft)], p.text));
        let (wrap, height) = if narrow { (WrapMode::Word, Length::Auto) } else { (WrapMode::None, Length::Cells(1.0)) };
        let lines = [&self.tagline, &self.prompt, &self.closing].into_iter().chain(&self.closing_parts);
        for line in lines {
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
    let prompt = line(app, &[Seg::new(PROMPT, p.accent_soft)], p, 1);
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
    let parts = [
        line(app, &[Seg::new("Press ? for every key.", p.accent_soft)], p, 1),
        line(app, &[Seg::new("No editing. Just reading.", p.dim)], p, 0),
    ];
    view.add(&closing).add(&parts[0]).add(&parts[1]);
    Welcome { root: view, card, prompt, closing, closing_parts: parts, tagline }
}

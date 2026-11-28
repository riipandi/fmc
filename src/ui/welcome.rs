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
const SEPARATOR: &str = "  │  ";

/// The card's outer width: its border (one column a side), its padding (two a side), the key column, the separator, and
/// the longest action.
fn card_width() -> u16 {
    let action = HINTS.iter().map(|(_, action)| action.chars().count()).max().unwrap_or(0);
    u16::try_from(2 + 4 + KEY_COLUMN_WIDTH + SEPARATOR.chars().count() + action).unwrap_or(u16::MAX)
}

fn line(app: &App, segments: &[Seg], p: &Palette, margin_top: u16) -> Text {
    Text::new(app, content(segments, p.text))
        .height(1)
        .shrink(0.0)
        .margin_edge(Edge::Top, margin_top)
        .wrap(WrapMode::None)
        .selectable(false)
}

const PROMPT: &str = "Select a file and press Enter.";
/// The tagline, after the wordmark: fmc is a file manager as well as a viewer, where Scouter says
/// `a quiet, read-only file viewer` (`src/ui/welcome.ts:46`).
const TAGLINE: &str = "a quiet terminal file manager";
/// The closing line after `Press ? for every key.`; Scouter says `No editing. Just reading.` (`src/ui/welcome.ts:80`).
const MENU_HINT: &str = "Space opens the menu.";

/// The welcome screen: its key card, its prompt, and its closing lines. The compact tier has no welcome screen.
pub struct Welcome {
    pub root: View,
    prompt: Text,
    /// The closing line in one row, and the same line broken at its ` · ` below 80 columns.
    closing: Text,
    closing_parts: [Text; 2],
    card: View,
}

impl Welcome {
    /// Fits the welcome to `columns`, beside a sidebar `sidebar` columns wide: below 80 the closing line breaks at its
    /// separator, and in the medium tier the card goes when the reader is narrower than it.
    pub fn fit(&self, columns: u16, sidebar: u16, p: &Palette) {
        let tier = fmc::layout::tier(columns);
        let phone = tier != fmc::layout::Tier::Wide;
        self.closing.visible(!phone);
        for part in &self.closing_parts {
            part.visible(phone);
        }
        // The reader's inner width: its border takes a column on each side. The rule is the medium tier's; the wide tier
        // keeps the card as it was.
        let reader = columns.saturating_sub(sidebar).saturating_sub(2);
        self.card.visible(tier != fmc::layout::Tier::Medium || reader >= card_width());
        self.prompt.set_text(content(&[Seg::new(PROMPT, p.accent_soft)], p.text));
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
    let tagline = [Seg::new(TAGLINE, p.muted), Seg::new(format!("  v{}", env!("CARGO_PKG_VERSION")), p.dim)];
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
            Seg::new(SEPARATOR, p.border),
            Seg::new(action, p.text),
        ];
        card.add(&line(app, &row, p, 0));
    }
    view.add(&card);
    let closing =
        [Seg::new("Press ? for every key.", p.accent_soft), Seg::new("  ·  ", p.ignored), Seg::new(MENU_HINT, p.dim)];
    let closing = line(app, &closing, p, 1);
    let parts = [
        line(app, &[Seg::new("Press ? for every key.", p.accent_soft)], p, 1),
        line(app, &[Seg::new(MENU_HINT, p.dim)], p, 0),
    ];
    view.add(&closing).add(&parts[0]).add(&parts[1]);
    Welcome { root: view, prompt, closing, closing_parts: parts, card }
}

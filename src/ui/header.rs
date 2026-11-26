//! A pane header (Scouter `src/ui/header.ts`): the title (with a dim parent folder) and badges, or an error notice in
//! place of the badges, over a hairline.

use fmc::theme::Palette;
use opentui::prelude::*;

use super::styled::{Seg, color, content};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadgeKind {
    Pill,
    Quiet,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Badge {
    pub text: String,
    pub kind: BadgeKind,
}

impl Badge {
    pub fn quiet(text: &str) -> Self {
        Self { text: text.into(), kind: BadgeKind::Quiet }
    }
}

/// The glyph of a header's touch box: `MENU` shows the list, `CLOSE` hides it (plan-00004, amended). `☰` measures two
/// cells in opentui's Unicode width method, so the menu takes `≡`, one cell; `✕` measures one cell.
pub const MENU: &str = "≡";
pub const CLOSE: &str = "✕";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HeaderState {
    pub title: String,
    pub parent: Option<String>,
    pub badges: Vec<Badge>,
    pub focused: bool,
    pub notice: Option<String>,
    /// The touch box left of the title, when the header shows one.
    pub button: Option<&'static str>,
}

impl HeaderState {
    pub fn title_segments(&self, p: &Palette) -> Vec<Seg> {
        let mut parts = Vec::new();
        if let Some(parent) = &self.parent {
            parts.push(Seg::new(parent.clone(), p.dim));
        }
        let title = Seg::new(self.title.clone(), if self.focused { p.text } else { p.muted });
        parts.push(if self.focused { title.bold() } else { title });
        parts
    }

    pub fn badge_segments(&self, p: &Palette) -> Vec<Seg> {
        if let Some(notice) = &self.notice {
            return vec![Seg::new(notice.clone(), p.error).bold()];
        }
        let mut segments = Vec::new();
        for badge in &self.badges {
            if !segments.is_empty() {
                segments.push(Seg::plain(" "));
            }
            segments.push(match badge.kind {
                BadgeKind::Pill => Seg::new(format!(" {} ", badge.text), p.accent_ink).on(p.accent).bold(),
                BadgeKind::Quiet => Seg::new(badge.text.clone(), p.dim),
            });
        }
        segments
    }
}

pub struct PaneHeader {
    pub root: View,
    button: Text,
    title: Text,
    badges: Text,
}

impl PaneHeader {
    pub fn new(app: &App) -> Self {
        let root = View::new(app)
            .height(2)
            .width(Length::percent(100.0))
            .row()
            .shrink(0.0)
            .padding_x(1)
            .border_sides(Sides::BOTTOM);
        // A filled box like a web menu button: the palette's pill pair (`accent_ink` on `accent`), padded by a cell.
        let button = Text::new(app, "").height(1).shrink(0.0).wrap(WrapMode::None).selectable(false).visible(false);
        let title =
            Text::new(app, "").height(1).grow(1.0).shrink(1.0).min_width(0).wrap(WrapMode::None).selectable(false);
        let badges =
            Text::new(app, "").height(1).shrink(0.0).margin_edge(Edge::Left, 1).wrap(WrapMode::None).selectable(false);
        root.add(&button).add(&title).add(&badges);
        Self { root, button, title, badges }
    }

    pub fn paint(&self, state: &HeaderState, p: &Palette) {
        self.root.border_color(color(p.border));
        self.button.visible(state.button.is_some());
        if let Some(glyph) = state.button {
            self.button.set_text(content(&[Seg::new(format!(" {glyph} "), p.accent_ink).on(p.accent).bold()], p.text));
        }
        self.title.set_text(content(&state.title_segments(p), p.text));
        self.badges.set_text(content(&state.badge_segments(p), p.text));
    }
}

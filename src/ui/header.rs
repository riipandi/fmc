//! A pane header (Scouter `src/ui/header.ts`): the title (with a dim parent folder) and badges, or an error notice in
//! place of the badges, over a hairline. A lead text sits left of the title: the compact tier's back text, or a medium
//! tier chevron that shows or hides the list (plan-00005, D3 and record 10).

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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HeaderState {
    pub title: String,
    pub parent: Option<String>,
    pub badges: Vec<Badge>,
    pub focused: bool,
    pub notice: Option<String>,
    /// The text left of the title: `‹ Files` in compact, or a medium chevron (`‹` hides the list, `›` shows it).
    pub lead: Option<String>,
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
    lead: Text,
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
        let lead = Text::new(app, "").height(1).shrink(0.0).wrap(WrapMode::None).selectable(false).visible(false);
        let title =
            Text::new(app, "").height(1).grow(1.0).shrink(1.0).min_width(0).wrap(WrapMode::None).selectable(false);
        let badges =
            Text::new(app, "").height(1).shrink(0.0).margin_edge(Edge::Left, 1).wrap(WrapMode::None).selectable(false);
        root.add(&lead).add(&title).add(&badges);
        Self { root, lead, title, badges }
    }

    /// Runs `handler` on a press on the lead text: the compact back text, or a medium chevron.
    pub fn on_lead_press(&self, handler: impl FnMut(&mut Cx<'_>, &MouseEvent) + 'static) {
        self.lead.on_mouse(handler);
    }

    /// The compact tier's header is one row with no rule under it (plan-00005, architecture.md record 5).
    pub fn compact(&self, compact: bool) {
        self.root.height(if compact { 1 } else { 2 });
        self.root.border_sides(if compact { Sides::NONE } else { Sides::BOTTOM });
    }

    pub fn paint(&self, state: &HeaderState, p: &Palette) {
        self.root.border_color(color(p.border));
        self.lead.visible(state.lead.is_some());
        if let Some(lead) = &state.lead {
            // Plain accent text with two columns after it: a filled box read as a glitch beside the title (record 6), and
            // one column ran the text into the title.
            self.lead.set_text(content(&[Seg::new(format!("{lead}  "), p.accent)], p.text));
        }
        self.title.set_text(content(&state.title_segments(p), p.text));
        self.badges.set_text(content(&state.badge_segments(p), p.text));
    }
}

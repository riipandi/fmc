//! The two-row footer (Scouter `src/ui/footer.ts`): a band with the pill, the location or a message, and facts on
//! the right; then a row of key hints that fit. In the compact tier the footer is one row, the bottom bar (`ui/bars.rs`).
//! `FooterState::render` is pure; `Footer` holds the widgets.

use std::cell::RefCell;
use std::ops::Range;
use std::path::Path;

use fmc::layout::{Page, Tier, tier};
use fmc::theme::{Hex, Palette};
use opentui::prelude::*;

use super::bars;
use super::find::prompt_segments;
use super::hints::{
    HINT_GAP, Hint, Mode, Pane, Prompt, Shown, fit_hints, hint_segments, hints_for, phone_rank, pill_label,
    prompt_hints, prompt_label, row_text,
};
use super::styled::{Measure, Seg, content, fit_end, fit_start, width};

const MIN_LOCATION_COLUMNS: usize = 16;

/// What the reader shows, for the facts on the right.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DocumentInfo {
    Text { lines: usize, language: String, truncated: bool, rendered: Option<bool> },
    Image { width: u32, height: u32, format: String },
    Binary { size: String },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiffInfo {
    pub location: String,
    pub files: usize,
    pub all_files: bool,
    pub truncated: bool,
    pub counts: Option<(usize, usize)>,
    pub hunks: Option<usize>,
    pub hunk_index: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub text: String,
    pub error: bool,
}

#[derive(Clone, Debug)]
pub struct FooterState {
    pub mode: Mode,
    pub pane: Pane,
    pub prompt: Option<Prompt>,
    pub message: Option<Message>,
    /// Already shortened with `~` for the home folder (`tildify`).
    pub location: String,
    pub document: Option<DocumentInfo>,
    pub diff: Option<DiffInfo>,
    /// Scroll position per mode: browse, diff.
    pub positions: [Option<String>; 2],
    pub wrap: bool,
    pub pending: Option<u32>,
    /// The sidebar is hidden, which the hints name (`Shown::sidebar_hidden`).
    pub sidebar_hidden: bool,
    /// The `:` command line's text while it is open; it takes the footer's left side (`Prompt::Command`).
    pub command: Option<String>,
    /// A text selection exists: below 80 columns the hints lead with `Copy`, which does what `y` does (plan-00004, P8).
    pub selection: bool,
    /// The page on screen, which names the compact bottom bar's segments (`ui/bars.rs`).
    pub page: Page,
    /// The open prompt's own text, which the compact bottom bar shows after `Cancel`.
    pub prompt_text: Option<Vec<Seg>>,
    /// How many tree rows are marked; zero hides the count (plan-00014, D8).
    pub marked: usize,
}

impl Default for FooterState {
    fn default() -> Self {
        Self {
            mode: Mode::Browse,
            pane: Pane::Files,
            prompt: None,
            message: None,
            location: String::new(),
            document: None,
            diff: None,
            positions: [None, None],
            wrap: false,
            pending: None,
            sidebar_hidden: false,
            command: None,
            selection: false,
            page: Page::Files,
            prompt_text: None,
            marked: 0,
        }
    }
}

/// `~` for the home folder, as Scouter's `tildify`.
pub fn tildify(path: &str, home: Option<&Path>) -> String {
    let Some(home) = home.and_then(Path::to_str) else { return path.to_string() };
    match path.strip_prefix(home) {
        Some(rest) if rest.is_empty() || rest.starts_with('/') => format!("~{rest}"),
        _ => path.to_string(),
    }
}

struct Fact {
    key: &'static str,
    segments: Vec<Seg>,
}

impl FooterState {
    /// The band's left and right segments and the hint row, for a terminal `columns` wide.
    pub fn render(&self, columns: usize, p: &Palette, measure: Measure<'_>) -> (Vec<Seg>, Vec<Seg>, Vec<Seg>) {
        let inner = columns.saturating_sub(2);
        let label = self.prompt.map_or_else(|| pill_label(self.mode, self.pane), prompt_label);
        let pill = Seg::new(format!(" {label} "), p.accent_ink).on(p.accent).bold();
        let pill_width = measure(&pill.text);
        let facts = self.fit_facts(inner.saturating_sub(pill_width + 1 + MIN_LOCATION_COLUMNS), p, measure);
        let facts_width = width(&facts, measure);
        let room = inner.saturating_sub(pill_width + 1 + if facts_width > 0 { facts_width + 2 } else { 0 });
        let mut left = vec![pill, Seg::plain(" ")];
        left.extend(self.describe(room, p, measure));
        let mut hints = vec![Seg::plain(" ")];
        hints.extend(hint_segments(&self.hint_row(columns, measure), p));
        (left, facts, hints)
    }

    /// The compact bottom bar (plan-00005, D3): an open prompt replaces it, a notice shows in it for its time, and
    /// otherwise the page's segments. Returns each tap's cell and key with the text.
    pub fn bar(
        &self,
        columns: usize,
        p: &Palette,
        measure: Measure<'_>,
    ) -> (Vec<Seg>, Vec<(Range<usize>, &'static str)>) {
        if let Some(text) = &self.prompt_text {
            return bars::prompt(text, columns, p);
        }
        if let Some(message) = &self.message {
            let fg = if message.error { p.error } else { p.accent_soft };
            return (vec![Seg::new(fit_start(&message.text, columns, measure), fg)], Vec::new());
        }
        let all_files = self.diff.as_ref().is_none_or(|d| d.all_files);
        let mut segments = bars::page_segments(self.page, self.mode == Mode::Diff, all_files, self.selection);
        // Its key is empty, so a tap on the count does nothing: `S` clears the marks from the keyboard.
        if self.marked > 0 && self.page == Page::Files {
            segments.insert(0, bars::Segment { label: format!("{} marked", self.marked), key: "" });
        }
        bars::render(&segments, columns, p)
    }

    fn describe(&self, columns: usize, p: &Palette, measure: Measure<'_>) -> Vec<Seg> {
        if let Some(text) = &self.command {
            return prompt_segments(":", text, "", p);
        }
        if let Some(message) = &self.message {
            let fg = if message.error { p.error } else { p.accent_soft };
            return vec![Seg::new(fit_start(&message.text, columns, measure), fg)];
        }
        let location = match self.mode {
            Mode::Diff => self.diff.as_ref().map_or("", |d| d.location.as_str()),
            Mode::Browse | Mode::Settings => self.location.as_str(),
        };
        let shown = fit_end(location, columns, measure);
        // The folder part is dim and the last name in `text`; a trailing `/` stays with the name (JS
        // `lastIndexOf("/", length - 2)`, which still looks at index 0 of a one-character string).
        let end = shown.len().saturating_sub(1).max(shown.len().min(1));
        let cut = shown.as_bytes()[..end].iter().rposition(|&b| b == b'/').map_or(0, |i| i + 1);
        vec![Seg::new(&shown[..cut], p.dim), Seg::new(&shown[cut..], p.text)]
    }

    fn candidate_facts(&self, p: &Palette) -> Vec<Fact> {
        let mut facts = Vec::new();
        let muted = |text: String| vec![Seg::new(text, p.muted)];
        match (self.mode, &self.document, &self.diff) {
            (Mode::Diff, _, Some(diff)) => {
                let plus = if diff.truncated { "+" } else { "" };
                let noun = if diff.files == 1 && !diff.truncated { "file" } else { "files" };
                facts.push(Fact { key: "files", segments: muted(format!("{}{plus} {noun}", diff.files)) });
                if let Some(hunks) = diff.hunks {
                    let label = match diff.hunk_index {
                        Some(index) if hunks > 1 => format!("{index}/{hunks} hunks"),
                        _ => format!("{hunks} {}", if hunks == 1 { "hunk" } else { "hunks" }),
                    };
                    facts.push(Fact { key: "hunks", segments: muted(label) });
                }
                if let Some((added, removed)) = diff.counts {
                    let segments = vec![
                        Seg::new(format!("+{added}"), p.added),
                        Seg::plain(" "),
                        Seg::new(format!("−{removed}"), p.removed),
                    ];
                    facts.push(Fact { key: "counts", segments });
                }
            }
            (Mode::Diff, _, None) => {}
            (Mode::Browse, Some(DocumentInfo::Image { width, height, format }), _) => {
                facts.push(Fact { key: "size", segments: muted(format!("{width}×{height}")) });
                facts.push(Fact { key: "language", segments: muted(format.clone()) });
            }
            (Mode::Browse, Some(DocumentInfo::Binary { size }), _) => {
                facts.push(Fact { key: "size", segments: muted(size.clone()) });
                facts.push(Fact { key: "language", segments: muted("binary".into()) });
            }
            (Mode::Browse, Some(DocumentInfo::Text { lines, language, truncated, .. }), _) => {
                let plus = if *truncated { "+" } else { "" };
                facts.push(Fact { key: "lines", segments: muted(format!("{lines}{plus} lines")) });
                facts.push(Fact { key: "language", segments: muted(language.clone()) });
                facts.push(Fact { key: "encoding", segments: muted("UTF-8".into()) });
            }
            (Mode::Browse, None, _) | (Mode::Settings, ..) => {}
        }
        let index = usize::from(self.mode == Mode::Diff);
        if let Some(position) = &self.positions[index] {
            facts.push(Fact { key: "position", segments: vec![Seg::new(position.clone(), p.text)] });
        }
        if let Some(pending) = self.pending.filter(|&n| n > 0) {
            facts.push(Fact { key: "count", segments: vec![Seg::new(pending.to_string(), p.accent_soft)] });
        }
        if self.marked > 0 {
            facts.insert(
                0,
                Fact { key: "marked", segments: vec![Seg::new(format!("{} marked", self.marked), p.accent_soft)] },
            );
        }
        facts
    }

    fn fit_facts(&self, columns: usize, p: &Palette, measure: Measure<'_>) -> Vec<Seg> {
        let mut facts = self.candidate_facts(p);
        let total = |facts: &[Fact]| -> usize {
            facts
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    width(&f.segments, measure)
                        + if i == 0 {
                            0
                        } else if f.key == "count" {
                            2
                        } else {
                            3
                        }
                })
                .sum()
        };
        for key in ["encoding", "language", "files", "lines", "size", "hunks", "counts", "position", "count"] {
            if total(&facts) <= columns {
                break;
            }
            if let Some(i) = facts.iter().position(|f| f.key == key) {
                facts.remove(i);
            }
        }
        if total(&facts) > columns {
            return Vec::new();
        }
        let mut segments = Vec::new();
        for (i, fact) in facts.into_iter().enumerate() {
            if i > 0 {
                segments.push(if fact.key == "count" { Seg::plain("  ") } else { Seg::new(" · ", p.ignored) });
            }
            segments.extend(fact.segments);
        }
        segments
    }

    /// The hints on the second row, in order, for a terminal `columns` wide. `render` draws them; `hint_cells` taps them.
    pub fn hint_row(&self, columns: usize, measure: Measure<'_>) -> Vec<Hint> {
        let inner = columns.saturating_sub(2);
        let mut hints = match self.prompt {
            Some(prompt) => prompt_hints(prompt),
            None => {
                let all_files = self.diff.as_ref().is_none_or(|d| d.all_files);
                let open = self.document.is_some();
                let sidebar_hidden = self.sidebar_hidden;
                let shown = match &self.document {
                    Some(DocumentInfo::Text { rendered, .. }) => {
                        Shown { markdown: *rendered, image: false, open, sidebar_hidden }
                    }
                    Some(DocumentInfo::Image { .. }) => Shown { markdown: None, image: true, open, sidebar_hidden },
                    _ => Shown { open, sidebar_hidden, ..Shown::default() },
                };
                hints_for(self.mode, self.pane, self.wrap, all_files, shown)
            }
        };
        // Below 80 columns a short row keeps the hints that leave the state first, and `Copy` leads while a selection
        // exists (plan-00004, P6, P8, D3). At 80 columns and wider the order is Scouter's.
        if tier(u16::try_from(columns).unwrap_or(u16::MAX)) != Tier::Wide {
            if self.selection && self.prompt.is_none() {
                hints.retain(|hint| hint.0 != "y");
                hints.push(("y", "Copy"));
            }
            hints.sort_by_key(phone_rank);
        }
        let (quit, rest): (Vec<Hint>, Vec<Hint>) = hints.into_iter().partition(|(key, _)| *key == "^q");
        let tail = fit_hints(&quit, inner, measure);
        let room = inner.saturating_sub(if tail.is_empty() { 0 } else { measure(&row_text(&tail)) + HINT_GAP.len() });
        let mut shown = fit_hints(&rest, room, measure);
        shown.extend(tail);
        shown
    }

    /// Each hint's cells on the second row, from its left edge (`render` starts the row with a space), with its key.
    pub fn hint_cells(&self, columns: usize, measure: Measure<'_>) -> Vec<(Range<usize>, &'static str)> {
        let mut cells = Vec::new();
        let mut at = 1;
        for (i, (key, action)) in self.hint_row(columns, measure).into_iter().enumerate() {
            if i > 0 {
                at += HINT_GAP.len();
            }
            let end = at + measure(&format!("{key} {action}"));
            cells.push((at..end, key));
            at = end;
        }
        cells
    }
}

/// The footer's widgets.
pub struct Footer {
    pub root: View,
    band: View,
    left: Text,
    right: Text,
    hints: Text,
    /// Where the last paint put each hint, so a tap finds its key.
    targets: RefCell<Vec<(Range<usize>, &'static str)>>,
}

impl Footer {
    pub fn new(app: &App) -> Self {
        let root = View::new(app).width(Length::percent(100.0)).height(2).column().shrink(0.0);
        let band = View::new(app).width(Length::percent(100.0)).height(1).row().shrink(0.0).padding_x(1);
        let left =
            Text::new(app, "").height(1).grow(1.0).shrink(1.0).min_width(0).wrap(WrapMode::None).selectable(false);
        let right = Text::new(app, "").height(1).shrink(0.0).wrap(WrapMode::None).selectable(false);
        let hints = Text::new(app, "")
            .height(1)
            .width(Length::percent(100.0))
            .shrink(0.0)
            .wrap(WrapMode::None)
            .selectable(false);
        band.add(&left).add(&right);
        root.add(&band).add(&hints);
        Self { root, band, left, right, hints, targets: RefCell::new(Vec::new()) }
    }

    pub fn set_background(&self, hex: Hex) {
        self.band.bg(super::styled::color(hex));
    }

    pub fn paint(&self, state: &FooterState, columns: usize, p: &Palette, measure: Measure<'_>) {
        // The compact footer is the bottom bar alone, one row (architecture.md, record 7).
        if tier(u16::try_from(columns).unwrap_or(u16::MAX)) == Tier::Compact {
            self.root.height(1);
            self.band.visible(false);
            let (drawn, targets) = state.bar(columns, p, measure);
            self.hints.set_text(content(&drawn, p.text));
            *self.targets.borrow_mut() = targets;
            return;
        }
        self.root.height(2);
        self.band.visible(true);
        let (left, right, hints) = state.render(columns, p, measure);
        self.left.set_text(content(&left, p.text));
        self.right.set_text(content(&right, p.text));
        self.hints.set_text(content(&hints, p.text));
        *self.targets.borrow_mut() = state.hint_cells(columns, measure);
    }

    /// The key the last paint's hint at `column` names, for a tap on the hint row.
    pub fn key_at(&self, column: usize) -> Option<&'static str> {
        self.targets.borrow().iter().find(|(cells, _)| cells.contains(&column)).map(|(_, key)| *key)
    }

    /// Runs `handler` on a press in the hint row.
    pub fn on_hint_press(&self, handler: impl FnMut(&mut Cx<'_>, &MouseEvent) + 'static) {
        self.hints.on_mouse(handler);
    }
}

#[cfg(test)]
#[path = "footer_tests.rs"]
mod tests;

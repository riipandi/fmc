//! Whether the sidebar shows (plan-00002, D1, D2, D7) and the layout tiers (architecture.md, "three layout tiers").
//! With a document open, the sidebar hides below 80 columns; a manual choice holds until the width crosses 80 again, or
//! a document opens or closes. The welcome screen keeps the sidebar from 60 columns up, and below 60 starts it hidden.
//! An open finder always holds it, since the list is what it shows. Scouter always draws the sidebar.

/// The wide tier starts here: at 80 or more the layout is Scouter's, and a document hides the sidebar below it (D1).
pub const BREAKPOINT: u16 = 80;

/// The compact tier is below this many columns; the medium tier is from here up to `BREAKPOINT`.
pub const COMPACT: u16 = 60;

/// Below this many columns the welcome screen drops its key card and wraps its lines (plan-00004, P5).
pub const NARROW_WELCOME: u16 = 50;

/// The three layout tiers by width: each range has its own binding constraint (architecture.md, record 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    Compact,
    Medium,
    Wide,
}

/// The tier of a terminal `columns` wide. Every caller asks this rather than comparing widths (plan-00005, D1).
pub fn tier(columns: u16) -> Tier {
    if columns < COMPACT {
        Tier::Compact
    } else if columns < BREAKPOINT {
        Tier::Medium
    } else {
        Tier::Wide
    }
}

/// Whether a shown sidebar covers the whole terminal.
pub fn is_drawer(columns: u16, shown: bool) -> bool {
    shown && tier(columns) == Tier::Compact
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Visibility {
    /// The width and the document's presence last observed, to see a crossing or a document opening or closing.
    last: Option<(u16, bool)>,
    /// The user's choice (`Ctrl+/` or the mark); `None` keeps the automatic state.
    manual: Option<bool>,
}

impl Visibility {
    /// Records the width and whether a document is open. A crossing of the breakpoint, or a document opening or
    /// closing, returns a manual choice to automatic (D2, D7).
    pub fn observe(&mut self, columns: u16, document: bool) {
        if let Some((last_columns, last_document)) = self.last
            && (crosses(last_columns, columns) || last_document != document)
        {
            self.manual = None;
        }
        self.last = Some((columns, document));
    }

    /// Hides a shown sidebar until the width crosses the breakpoint or a document opens or closes.
    pub fn close(&mut self) {
        self.manual = Some(false);
    }

    pub fn shown(&self, columns: u16, document: bool, held: bool) -> bool {
        if held || (!document && tier(columns) != Tier::Compact) {
            return true;
        }
        // Below the compact tier the welcome screen starts with its list hidden (amended, architecture.md); a document
        // keeps the 80-column rule.
        self.manual.unwrap_or(document && tier(columns) == Tier::Wide)
    }

    /// Whether the drawer covers the reader, as of the last width observed.
    pub fn covers(&self, document: bool, held: bool) -> bool {
        self.last.is_some_and(|(columns, _)| is_drawer(columns, self.shown(columns, document, held)))
    }

    /// Flips what shows now.
    pub fn toggle(&mut self, columns: u16, document: bool, held: bool) {
        self.manual = Some(!self.shown(columns, document, held));
    }
}

fn crosses(last: u16, now: u16) -> bool {
    (tier(last) == Tier::Wide) != (tier(now) == Tier::Wide)
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod tests;

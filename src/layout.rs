//! Whether the sidebar shows (plan-00002, D1, D2, D7). With a document open, the sidebar hides below 80 columns; a
//! manual choice holds until the width crosses 80 again, or a document opens or closes. The welcome screen keeps the
//! sidebar from 60 columns up, and below 60 starts it hidden. An open finder always holds it, since the list is what it
//! shows. Scouter always draws the sidebar.

/// Below this many columns a document hides the sidebar (D1): at 80 or more the layout is Scouter's.
pub const BREAKPOINT: u16 = 80;

/// Below this many columns a shown sidebar is a drawer over the reader, and a focus that moves to the reader closes it
/// (approved phone rule, architecture.md, "a phone layout below 80 columns").
pub const DRAWER: u16 = 60;

/// Whether a shown sidebar covers the whole terminal.
pub fn is_drawer(columns: u16, shown: bool) -> bool {
    shown && columns < DRAWER
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
        if held || (!document && columns >= DRAWER) {
            return true;
        }
        // Below `DRAWER` the welcome screen starts with its list hidden (amended, architecture.md); a document keeps
        // the 80-column rule.
        self.manual.unwrap_or(document && columns >= BREAKPOINT)
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
    (last >= BREAKPOINT) != (now >= BREAKPOINT)
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod tests;

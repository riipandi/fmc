//! The layout tiers, the sidebar's visibility in the medium and wide tiers, and the compact tier's page stack. With a
//! document open, the sidebar hides below 80 columns; a manual choice holds until the width crosses 80 again, or a
//! document opens or closes. The welcome screen and an open finder hold the sidebar. Scouter always draws the sidebar.

/// The wide tier starts here: at 80 or more the layout is Scouter's, and a document hides the sidebar below it.
pub const BREAKPOINT: u16 = 80;

/// The compact tier is below this many columns; the medium tier is from here up to `BREAKPOINT`.
pub const COMPACT: u16 = 60;

/// The three layout tiers by width: each range has its own binding constraint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    Compact,
    Medium,
    Wide,
}

/// The tier of a terminal `columns` wide. Every caller asks this rather than comparing widths.
pub fn tier(columns: u16) -> Tier {
    if columns < COMPACT {
        Tier::Compact
    } else if columns < BREAKPOINT {
        Tier::Medium
    } else {
        Tier::Wide
    }
}

/// A screen of the compact tier. `Files` is the bottom page and is never removed; the top page is the
/// one on screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Files,
    Reader,
    Changes,
    Keys,
    Find,
    /// The Settings page: its own page in the compact tier, as Keys is.
    Settings,
}

/// The pages open, bottom first. Each page is kept while its screen is open, so a repaint never reorders the stack.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pages(Vec<Page>);

impl Default for Pages {
    fn default() -> Self {
        Self(vec![Page::Files])
    }
}

impl Pages {
    /// The page on screen.
    pub fn top(&self) -> Page {
        self.0.last().copied().unwrap_or(Page::Files)
    }

    /// The page under the one on screen: where Back goes.
    pub fn under(&self) -> Page {
        self.0.len().checked_sub(2).map_or(Page::Files, |at| self.0[at])
    }

    /// Opens `page` on top when `open` holds and it is not open yet, and removes it when `open` does not hold.
    pub fn sync(&mut self, page: Page, open: bool) {
        if page == Page::Files {
            return;
        }
        match (open, self.0.contains(&page)) {
            (true, false) => self.0.push(page),
            (false, true) => self.0.retain(|open| *open != page),
            _ => {}
        }
    }
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
    /// closing, returns a manual choice to automatic.
    pub fn observe(&mut self, columns: u16, document: bool) {
        if let Some((last_columns, last_document)) = self.last
            && (crosses(last_columns, columns) || last_document != document)
        {
            self.manual = None;
        }
        self.last = Some((columns, document));
    }

    /// Whether the sidebar shows in the medium and wide tiers. The welcome screen and an open finder keep it.
    pub fn shown(&self, columns: u16, document: bool, held: bool) -> bool {
        if held || !document {
            return true;
        }
        self.manual.unwrap_or(tier(columns) == Tier::Wide)
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

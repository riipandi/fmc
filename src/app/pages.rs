//! The compact tier's page stack (plan-00005, D2). The stack follows the screens: Find, Changes, and Keys are open while
//! their screen is, and Reader while the focus is in the reader on a document or on the changes body. The top page is
//! the one on screen in compact. Medium and wide keep the stack too, so a resize into compact shows the open page.

use fmc::keys::Focus;
use fmc::layout::{Page, Tier, tier};

use super::{Model, Ui, changes, find, reader};

impl Model {
    /// Whether the terminal is in the compact tier, where the page stack draws the screen.
    pub fn compact(&self) -> bool {
        tier(self.columns) == Tier::Compact
    }
}

/// Keeps the stack in step with the screens. In compact the top page sets the focus: the reader and the key page take
/// it, and the list pages leave it with the list. Runs at each paint, before the layout.
pub fn sync(m: &mut Model) {
    let reading = m.focus == Focus::Reader && !m.shown.shows_keys() && (m.shown.has_document() || m.changes.is_some());
    m.pages.sync(Page::Find, m.finder.is_some());
    m.pages.sync(Page::Changes, m.changes.is_some());
    m.pages.sync(Page::Keys, m.shown.shows_keys());
    m.pages.sync(Page::Reader, reading);
    if m.compact() {
        m.focus = if matches!(m.pages.top(), Page::Reader | Page::Keys) { Focus::Reader } else { Focus::List };
    }
}

/// The compact header's lead while a page is open over Files: a `‹` that Back runs (plan-00005, record 6; the page name
/// is dropped so the path has the columns, and the bottom bar still names Back).
pub fn back_lead(m: &Model) -> Option<String> {
    (m.compact() && m.pages.top() != Page::Files).then(|| "‹".into())
}

/// Back (compact): closes the top page the way its own key does, so `q` and `Esc` go back a page.
pub fn back(m: &mut Model, ui: &Ui) {
    match m.pages.top() {
        Page::Reader => m.focus = Focus::List,
        Page::Keys => reader::close_keys(m, ui),
        Page::Changes => changes::leave(m),
        Page::Find => find::end(m, None),
        Page::Files => {}
    }
}

/// `Ctrl+/` in compact: switches between the list and the reader, as `Tab` does (architecture.md, record 11).
pub fn switch(m: &mut Model) {
    if m.finder.is_some() {
        return;
    }
    if m.focus == Focus::Reader {
        m.focus = Focus::List;
    } else if m.shown.has_document() || m.changes.is_some() {
        m.focus = Focus::Reader;
    }
}

#[cfg(test)]
#[path = "pages_tests.rs"]
mod tests;

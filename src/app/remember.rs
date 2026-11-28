//! The remembered UI state (`meta/state.json`, `fmc::state`): the sidebar width, the reader's wrap, the changes view's
//! options, the dotfiles, and the finder's matching. Each paint compares what the model holds with the values the last
//! paint saw, so a toggle needs no code of its own. A change is written once the changes stop for `DELAY`, on a worker;
//! a write still waiting when the app quits is made before the process ends.

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use fmc::state::State;
use opentui::prelude::*;

use super::Model;
use crate::services::state::Loaded;
use crate::services::{self, Event};
use crate::ui::footer::Message;

/// How long after the last change the file is written: a burst of toggles is one write.
const DELAY: Duration = Duration::from_secs(1);

/// The save's bookkeeping: the file, the values the last paint saw, and the write waiting for its timer.
#[derive(Default)]
pub struct Saves {
    path: Option<PathBuf>,
    seen: State,
    /// The values the next write holds. The timer takes them when it fires, and a quit takes them too.
    due: Rc<RefCell<Option<State>>>,
    /// Counts the changes, so a timer set by an older change finds a newer count and does nothing.
    clock: Rc<Cell<u64>>,
    /// A failed write is a footer notice once per run, so a folder that stays unwritable does not repeat it.
    warned: bool,
}

/// What the model holds, in the file's terms. Every value is set, so a change to any one differs from the last paint.
fn snapshot(m: &Model) -> State {
    State {
        sidebar_width: m.sidebar,
        wrap: Some(m.wrap),
        diff_split: Some(m.diff_options.split),
        diff_wide: Some(m.diff_options.wide),
        diff_all_files: Some(!m.diff_options.one_file),
        dotfiles: Some(m.browser.dotfiles()),
        find_mode: Some(m.matching),
    }
}

/// Starts from the file: each value it sets replaces the default, and a value it does not set keeps the model's. The
/// finder's matching keeps the `settings.json` value when the file has none, so the setting seeds a fresh state.
/// Nothing is written at the start: the values read are the ones the first paint sees.
pub fn start(m: &mut Model, loaded: Loaded) {
    let state = loaded.state;
    m.sidebar = state.sidebar_width.or(m.sidebar);
    m.wrap = state.wrap.unwrap_or(m.wrap);
    m.diff_options.split = state.diff_split.unwrap_or(m.diff_options.split);
    m.diff_options.wide = state.diff_wide.unwrap_or(m.diff_options.wide);
    m.diff_options.one_file = state.diff_all_files.map_or(m.diff_options.one_file, |all| !all);
    m.browser.set_dotfiles(state.dotfiles.unwrap_or(m.browser.dotfiles()));
    m.matching = state.find_mode.unwrap_or(m.matching);
    m.saves.path = loaded.path;
    m.saves.seen = snapshot(m);
}

/// Runs at each paint: a remembered value that changed since the last paint is written `DELAY` after the last change.
pub fn sync(app: &App, m: &mut Model) {
    let now = snapshot(m);
    if now == m.saves.seen {
        return;
    }
    m.saves.seen = now;
    let Some(path) = m.saves.path.clone() else { return };
    *m.saves.due.borrow_mut() = Some(now);
    let turn = m.saves.clock.get() + 1;
    m.saves.clock.set(turn);
    let (due, clock, proxy) = (m.saves.due.clone(), m.saves.clock.clone(), m.proxy.clone());
    app.set_timeout(DELAY, move |_| {
        if clock.get() != turn {
            return;
        }
        let Some(state) = due.borrow_mut().take() else { return };
        services::spawn(&proxy, move || Event::StateSaved { result: services::state::save(&path, &state) });
    });
}

/// The worker's answer. A failure is the footer's notice, and only the first one of a run is shown.
pub fn saved(m: &mut Model, result: &Result<(), String>) {
    if let Err(text) = result
        && !m.saves.warned
    {
        m.saves.warned = true;
        m.message = Some(Message { text: text.clone(), error: true });
    }
}

/// Writes the change still waiting for its timer when the app quits. The loop has ended, so the write runs on a thread
/// that is joined here, and a write that fails has no footer left to say so.
pub fn flush(m: &Model) {
    let (Some(path), Some(state)) = (m.saves.path.clone(), m.saves.due.borrow_mut().take()) else { return };
    let _ = std::thread::spawn(move || services::state::save(&path, &state)).join();
}

#[cfg(test)]
#[path = "remember_tests.rs"]
mod tests;

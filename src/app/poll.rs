//! The polls (Scouter `BrowseWatcher` and `DiffScreen.watch`): the browse poll runs for the life of the app, and the
//! changes poll runs while the changes screen shows its session. A tick starts one job on a worker; the job's answer
//! schedules the next tick, after a back-off of three times the job's duration (`src/explorer/browse-watcher.ts`).

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use opentui::prelude::App;

use super::{Model, Ui, gutter, reader};
use crate::services;

/// Scouter's poll interval (`src/app/app.ts:43-44`), and its longest wait (`MAX_POLL_MS`).
pub const POLL: Duration = Duration::from_secs(2);
const MAX_POLL: Duration = Duration::from_secs(30);

/// The next wait after a poll that took `elapsed`: three times its duration, within [2 s, 30 s].
pub fn backoff(elapsed: Duration) -> Duration {
    (elapsed * 3).clamp(POLL, MAX_POLL)
}

/// Which poll a tick belongs to. A changes tick ends when its session has left the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chain {
    Browse,
    Diff(u64),
}

/// The next tick the model asks for; `schedule` runs it once, after `delay`.
#[derive(Clone, Copy, Debug)]
pub struct Tick {
    pub chain: Chain,
    pub delay: Duration,
}

/// Schedules the ticks the model asked for since the last call.
pub fn schedule(app: &App, model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>) {
    let ticks = std::mem::take(&mut model.borrow_mut().ticks);
    for tick in ticks {
        let (model, ui) = (model.clone(), ui.clone());
        app.set_timeout(tick.delay, move |cx| {
            fire(cx.app(), &model, &ui, tick.chain);
        });
    }
}

fn fire(app: &App, model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>, chain: Chain) {
    {
        let mut m = model.borrow_mut();
        match chain {
            Chain::Browse => browse_tick(&mut m),
            Chain::Diff(session) => diff_tick(&mut m, session),
        }
    }
    schedule(app, model, ui);
}

fn browse_tick(m: &mut Model) {
    // Scouter's refresh does nothing on the changes screen or while a file opens; the chain still goes on.
    if m.changes.is_some() || m.loading {
        m.ticks.push(Tick { chain: Chain::Browse, delay: POLL });
        return;
    }
    m.browse_started = Some(std::time::Instant::now());
    browse_now(m, true);
}

/// Starts a browse poll now: the expanded folders, the marks, and the open text document (Scouter `refreshBrowse`).
/// `chain` says whether its answer schedules the next timed poll.
pub fn browse_now(m: &Model, chain: bool) {
    let root = m.browser.tree.root().to_path_buf();
    let folders = m
        .browser
        .tree
        .expanded()
        .into_iter()
        .map(|path| {
            let parents = m.browser.tree.ancestors(&path);
            (path, parents)
        })
        .collect();
    let document = match &m.shown {
        reader::Shown::Text { path, stamp: Some(stamp), .. } => Some((path.clone(), stamp.clone())),
        _ => None,
    };
    services::poll::browse(&m.proxy, chain, root, folders, document);
    gutter::request(m);
}

fn diff_tick(m: &mut Model, session: u64) {
    let Some(repository) =
        m.changes.as_ref().filter(|changes| changes.session == session).map(|c| c.repository.clone())
    else {
        return;
    };
    m.diff_started = Some(std::time::Instant::now());
    let all = !m.diff_options.one_file;
    let selected = m.change_list.selected().map(|change| change.path.clone());
    services::changes::poll(&m.proxy, session, repository, all, selected);
}

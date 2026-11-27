//! The reader's git sign strip (plan-00008): the open text's signs against the change base. A job asks for them when a
//! text is shown and on each browse tick, and an answer is kept only for the showing it was asked for.

use std::path::Path;

use fmc::diff::Sign;

use super::Model;
use super::reader::Shown;
use crate::services::{self, Event};

/// Asks for the shown text's signs, when the text is in a repository and the git gutter is on.
pub fn request(m: &Model) {
    if !m.git_gutter || !m.in_repo {
        return;
    }
    let Shown::Text { path, serial, .. } = &m.shown else { return };
    let (work, serial) = (path.clone(), *serial);
    services::spawn(&m.proxy, move || Event::Signed {
        path: work.clone(),
        serial,
        result: services::git::signs(&work),
    });
}

/// Keeps a job's answer for the text shown as `serial`. A git failure leaves no signs, with no notice (D5).
pub fn apply(m: &mut Model, path: &Path, serial: u64, result: &Result<Vec<(u32, Sign)>, String>) {
    if let Shown::Text { path: shown, serial: shown_serial, signs, .. } = &mut m.shown
        && shown == path
        && *shown_serial == serial
    {
        *signs = result.clone().unwrap_or_default();
    }
}

/// The signs the reader draws, and whether its gutter keeps the strip's column: a text in a repository, with the setting
/// on. Other views draw no signs.
pub fn drawn(m: &Model) -> (&[(u32, Sign)], bool) {
    match &m.shown {
        Shown::Text { signs, .. } => (signs, m.git_gutter && m.in_repo),
        _ => (&[], false),
    }
}

#[cfg(test)]
#[path = "gutter_tests.rs"]
mod tests;

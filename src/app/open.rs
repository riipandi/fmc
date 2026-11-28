//! Opens the selected row or the shown file in another app (plan-00017): `o` and the context menu's Open row hand the
//! target to macOS's default app. The launch runs on a worker (`services/launch.rs`); this file picks the target and
//! words the answer.

use std::ffi::OsString;

use super::*;
use crate::services::launch::{self, How};

/// `o`, and the menu's Open row: the selected row in the tree, or the shown file in the reader (D5). The key page is
/// a reader with no file, so it has no target and `o` does nothing there.
pub fn default_app(m: &mut Model) {
    let target = match m.focus {
        Focus::List => m.browser.selected_entry().map(|entry| entry.path),
        Focus::Reader => m.shown.path().map(Path::to_path_buf),
    };
    let Some(target) = target else { return };
    let args = launch::open_args(&target, How::Default);
    let name = write::label(&target);
    crate::services::spawn(&m.proxy, move || Event::Launched { name, result: launch(&args) });
}

/// The answer of a launch (`Event::Launched`): a success names the target, and a failure is `open`'s first error line.
pub fn launched(m: &mut Model, name: &str, result: &Result<(), String>) {
    m.message = Some(match result {
        Ok(()) => Message { text: format!("Opened {name}."), error: false },
        Err(text) => Message { text: text.clone(), error: true },
    });
}

/// Hands the arguments to `open`. The tests stand in for it, so a test never launches an app (`write.rs`, `trash`).
#[cfg(not(test))]
fn launch(args: &[OsString]) -> Result<(), String> {
    launch::run_open(args)
}

/// The test stand-in: the target must exist, as `open` requires, and no app opens.
#[cfg(test)]
fn launch(args: &[OsString]) -> Result<(), String> {
    let target = args.last().map(std::path::PathBuf::from).unwrap_or_default();
    if target.exists() {
        Ok(())
    } else {
        Err(format!("The file {} couldn't be opened because there is no such file.", target.display()))
    }
}

#[cfg(test)]
#[path = "open_tests.rs"]
mod tests;

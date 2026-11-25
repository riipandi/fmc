//! The `opentui` app loop and key dispatch.

use std::path::Path;

use opentui::prelude::*;
use opentui::widget::Text;

/// Runs the app on `target` until the user quits.
// shortcut: placeholder screen until plan-00001 phase 8 lands the chrome.
pub fn run(target: &Path) -> opentui::Result<()> {
    let app = App::new(AppOptions::default())?;
    app.root().add(&Text::new(&app, format!("fmc: {} (q quits)", target.display())));
    app.on_key(|cx, key| {
        if key.matches("q") {
            cx.quit();
        }
    });
    app.run()
}

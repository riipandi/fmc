//! The `opentui` app loop and key dispatch.

mod theme;

use std::path::Path;

use fmc::theme::palette;
use opentui::prelude::*;
use opentui::widget::Text;

/// Runs the app on `target` until the user quits.
// shortcut: placeholder screen until plan-00001 phase 8 lands the chrome.
pub fn run(target: &Path) -> opentui::Result<()> {
    let app = App::new(AppOptions { palette_size: theme::PALETTE_SIZE, ..AppOptions::default() })?;
    let text = Text::new(&app, format!("fmc: {} (q quits)", target.display()));
    app.root().add(&text);
    theme::watch(&app, move |_app, system| {
        if let Some(color) = Color::parse(palette(system.mode).text) {
            text.fg(color);
        }
    });
    app.on_key(|cx, key| {
        if key.matches("q") {
            cx.quit();
        }
    });
    app.run()
}

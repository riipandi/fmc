//! Follows the terminal's background (Scouter `src/ui/system-theme.ts`): read at start, every 3 s while the terminal
//! answers and has not announced a change, and again on each announcement. The decisions live in `fmc::theme`.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use fmc::theme::{Mode, POLL_SECONDS, SystemTheme, ThemeWatch};
use opentui::prelude::*;
use opentui::term::TerminalPalette;

/// Palette entries to ask for: one, as Scouter's `getPalette({ size: 1 })`; only the background is read.
pub const PALETTE_SIZE: u16 = 1;

fn background_hex(palette: Option<&TerminalPalette>) -> Option<String> {
    let c = palette?.background?;
    Some(format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b))
}

fn mode_hint(app: &App) -> Option<Mode> {
    app.theme_mode().map(|mode| match mode {
        opentui::theme::ThemeMode::Light => Mode::Light,
        opentui::theme::ThemeMode::Dark => Mode::Dark,
    })
}

/// Calls `on_change` with the first theme at once, then whenever the terminal's theme changes.
pub fn watch(app: &App, on_change: impl Fn(&App, SystemTheme) + 'static) {
    let state = Rc::new(RefCell::new(ThemeWatch::default()));
    let on_change = Rc::new(on_change);
    let answered = Rc::new(Cell::new(false));

    let first_background = background_hex(app.palette().as_ref());
    let first = state.borrow_mut().reading(first_background.as_deref(), mode_hint(app));
    if let Some(theme) = first {
        on_change(app, theme);
    }

    {
        let (state, on_change, answered) = (state.clone(), on_change.clone(), answered.clone());
        app.on_palette(move |cx, palette| {
            answered.set(true);
            let hint = mode_hint(cx.app());
            let theme = state.borrow_mut().reading(background_hex(Some(palette)).as_deref(), hint);
            if let Some(theme) = theme {
                on_change(cx.app(), theme);
            }
        });
    }
    {
        let state = state.clone();
        app.on_theme_mode(move |cx, _mode| {
            state.borrow_mut().announced();
            cx.app().refresh_palette();
        });
    }

    if !state.borrow().keeps_polling(first_background.is_some()) {
        return;
    }
    answered.set(true);
    let timer = Rc::new(Cell::new(None));
    let id = app.set_interval(Duration::from_secs(POLL_SECONDS), {
        let timer = timer.clone();
        move |cx| {
            // A refresh waits at most a second, so by the next tick the last one has answered or never will.
            if !state.borrow().keeps_polling(answered.replace(false)) {
                if let Some(id) = timer.take() {
                    cx.app().cancel_timer(id);
                }
                return;
            }
            cx.app().refresh_palette();
        }
    });
    timer.set(Some(id));
}

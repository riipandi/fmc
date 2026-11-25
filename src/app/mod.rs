//! The `opentui` app loop: the two panes and the footer, key dispatch through `fmc::keys::Parser`, and theme
//! changes. Terminal restore on exit and on panic is OpenTUI's (`rust.md`, "Contracts and traps").

mod keys;
mod theme;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use fmc::keys::{Action, Focus, Parser, Screen, State, Step};
use fmc::theme::{Mode, Palette, SystemTheme, palette, tinted_background};
use opentui::prelude::*;

use crate::ui::footer::{Footer, FooterState, tildify};
use crate::ui::header::{Badge, HeaderState, PaneHeader};
use crate::ui::help::Help;
use crate::ui::hints::Pane;
use crate::ui::styled::color;
use crate::ui::{sidebar, welcome};

struct Ui {
    browser_panel: View,
    browser_header: PaneHeader,
    viewer_panel: View,
    viewer_header: PaneHeader,
    welcome: View,
    footer: Footer,
    layout: View,
    help: Option<Help>,
}

struct Model {
    root: PathBuf,
    target: PathBuf,
    focus: Focus,
    help_open: bool,
    mode: Mode,
    background: Option<String>,
    parser: Parser,
}

impl Model {
    fn palette(&self) -> &'static Palette {
        palette(self.mode)
    }

    fn tint(&self) -> String {
        tinted_background(self.background.as_deref(), self.mode)
    }

    fn key_state(&self) -> State {
        State {
            screen: Screen::Browse,
            focus: self.focus,
            help_open: self.help_open,
            reader_ready: false,
            previewing: false,
            typing: false,
        }
    }
}

/// Runs the app on `target` until the user quits.
pub fn run(target: &Path) -> opentui::Result<()> {
    // Scouter draws on the terminal's own background (`RGBA.defaultBackground()`), not a theme color.
    let [dark, light] = [Theme::dark(), Theme::light()].map(|t| Theme { background: Color::DEFAULT_BACKGROUND, ..t });
    let options = AppOptions {
        palette_size: theme::PALETTE_SIZE,
        tab_focus: false,
        theme: dark,
        light_theme: Some(light),
        ..AppOptions::default()
    };
    let app = App::new(options)?;
    let root = if target.is_dir() { target.to_path_buf() } else { target.parent().unwrap_or(target).to_path_buf() };
    let model = Rc::new(RefCell::new(Model {
        root,
        target: target.to_path_buf(),
        focus: Focus::List,
        help_open: false,
        mode: Mode::Dark,
        background: None,
        parser: Parser::default(),
    }));
    let ui = Rc::new(RefCell::new(build(&app, model.borrow().palette())));
    paint(&app, &model.borrow(), &mut ui.borrow_mut());

    theme::watch(&app, {
        let (model, ui) = (model.clone(), ui.clone());
        move |app, SystemTheme { mode, background }| {
            let mut m = model.borrow_mut();
            m.mode = mode;
            m.background = background;
            restyle(app, &m, &mut ui.borrow_mut());
        }
    });
    app.on_resize({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, _, _| paint(cx.app(), &model.borrow(), &mut ui.borrow_mut())
    });
    app.on_key_capture({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event| {
            let Some(named) = keys::name(event) else { return };
            let mut m = model.borrow_mut();
            let state = m.key_state();
            let Step::Done(action) = m.parser.feed(named.key(), state) else { return };
            match action {
                Action::Quit => return cx.quit(),
                Action::Help => m.help_open = true,
                Action::CloseHelp => m.help_open = false,
                // No reader content yet: Tab and Esc both leave the focus on the file list.
                Action::SwitchPane | Action::Escape => m.focus = Focus::List,
                _ => {}
            }
            paint(cx.app(), &m, &mut ui.borrow_mut());
        }
    });

    #[cfg(debug_assertions)]
    if std::env::var_os("FMC_PANIC_AFTER_START").is_some() {
        app.set_timeout(std::time::Duration::from_millis(200), |_| panic!("Peeves was here"));
    }
    app.run()
}

fn build(app: &App, p: &Palette) -> Ui {
    let layout = View::new(app).width(Length::percent(100.0)).height(Length::percent(100.0)).column();
    let panes = View::new(app).width(Length::percent(100.0)).row().grow(1.0).min_height(0);
    let browser_panel = View::new(app).shrink(0.0).column().border(BorderStyle::Rounded);
    let browser_header = PaneHeader::new(app);
    let viewer_panel = View::new(app).grow(1.0).min_width(0).column().border(BorderStyle::Rounded);
    let viewer_header = PaneHeader::new(app);
    let welcome = welcome::welcome(app, p);
    let footer = Footer::new(app);
    browser_panel.add(&browser_header.root);
    viewer_panel.add(&viewer_header.root).add(&welcome);
    panes.add(&browser_panel).add(&viewer_panel);
    layout.add(&panes).add(&footer.root);
    app.root().add(&layout);
    Ui { browser_panel, browser_header, viewer_panel, viewer_header, welcome, footer, layout, help: None }
}

/// Rebuilds the parts that bake colors in (welcome, help), then repaints.
fn restyle(app: &App, m: &Model, ui: &mut Ui) {
    ui.welcome.detach();
    ui.welcome = welcome::welcome(app, m.palette());
    ui.viewer_panel.add(&ui.welcome);
    if let Some(help) = ui.help.take() {
        help.overlay.detach();
    }
    paint(app, m, ui);
}

fn paint(app: &App, m: &Model, ui: &mut Ui) {
    let p = m.palette();
    let (columns, rows) = app.size();
    let measure = |text: &str| {
        app.session().context().text_width(text, app.width_method()).map_or(text.chars().count(), |w| w as usize)
    };
    let tint = m.tint();

    ui.browser_panel.width(sidebar::fit(columns, None));
    let focused = |pane: Focus| color(if m.focus == pane { p.accent } else { p.border });
    ui.browser_panel.border_color(focused(Focus::List));
    ui.viewer_panel.border_color(focused(Focus::Reader));

    let name = m.root.file_name().map_or(String::new(), |n| fmc::text::display_label(&n.to_string_lossy()));
    let sidebar = HeaderState {
        title: name,
        badges: vec![Badge::quiet("files")],
        focused: m.focus == Focus::List,
        ..Default::default()
    };
    ui.browser_header.paint(&sidebar, p);
    let reader = HeaderState {
        title: "Reader".into(),
        badges: vec![Badge::quiet("read-only")],
        focused: m.focus == Focus::Reader,
        ..Default::default()
    };
    ui.viewer_header.paint(&reader, p);

    let home = std::env::var_os("HOME").map(PathBuf::from);
    let location = tildify(&fmc::text::display_label(&m.target.to_string_lossy()), home.as_deref());
    let pane = if m.focus == Focus::List { Pane::Files } else { Pane::Reader };
    let footer = FooterState { location, pane, pending: m.parser.pending_count(), ..FooterState::default() };
    ui.footer.set_background(&tint);
    ui.footer.paint(&footer, usize::from(columns), p, &measure);

    if m.help_open && ui.help.is_none() {
        let help = Help::new(app, p, &tint);
        ui.layout.add(&help.overlay);
        ui.help = Some(help);
    }
    if let Some(help) = &ui.help {
        help.centre(usize::from(rows));
        help.overlay.visible(m.help_open);
    }
}

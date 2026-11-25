//! The `opentui` app loop: the two panes and the footer, key and mouse dispatch, the tree's worker events, and theme
//! changes. Terminal restore on exit and on panic is OpenTUI's (`rust.md`, "Contracts and traps").

mod keys;
mod reader;
mod theme;
mod tree;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use fmc::diff::Marks;
use fmc::explorer::{Browser, Tree};
use fmc::keys::{Action, Focus, Parser, Screen, State, Step};
use fmc::reader::markdown::{PARAGRAPH_REFUSED, SEARCH_REFUSED};
use fmc::theme::{Mode, Palette, SystemTheme, palette, tinted_background};
use opentui::prelude::*;

use crate::services::Event;
use crate::ui::footer::{Footer, FooterState, Message, tildify};
use crate::ui::header::{Badge, HeaderState, PaneHeader};
use crate::ui::help::Help;
use crate::ui::hints::Pane;
use crate::ui::markdown::MarkdownView;
use crate::ui::reader::CodeView;
use crate::ui::scrollbar::ScrollBar;
use crate::ui::styled::color;
use crate::ui::tree::{LIST_TOP, TreeView, line_at};
use crate::ui::{sidebar, syntax, welcome};

/// Screen rows the list does not use: the footer (2), the panel's borders (2), and the header (2).
const LIST_CHROME_ROWS: usize = 6;

struct Ui {
    browser_panel: View,
    browser_header: PaneHeader,
    tree: TreeView,
    handle: View,
    viewer_panel: View,
    viewer_header: PaneHeader,
    welcome: View,
    /// The viewer body: the welcome, the code view, and the rendered view stacked (one shows at a time), beside the bar.
    stack: View,
    reader: CodeView,
    markdown: MarkdownView,
    bar: ScrollBar,
    footer: Footer,
    layout: View,
    help: Option<Help>,
}

/// A reveal in progress: list the folders on the way to `target`, then select it. `expand` also opens the target
/// (going up keeps the old root open).
struct Pending {
    target: PathBuf,
    expand: bool,
    /// Read the target once it is revealed (a file on the command line).
    open: bool,
}

struct Model {
    /// The location the footer shows: the startup path, then each folder opened.
    location: String,
    focus: Focus,
    help_open: bool,
    mode: Mode,
    background: Option<String>,
    parser: Parser,
    proxy: AppProxy,
    browser: Browser,
    marks: Marks,
    pending: Option<Pending>,
    shown: reader::Shown,
    /// A file or folder open is in flight; another open is refused (Scouter `runLoading`).
    loading: bool,
    /// The next file shown is the startup file, which opens with the reader focused as in Scouter (`finishShowing`);
    /// a file opened from the tree keeps the list focused (approved deviation, architecture.md).
    focus_reader_on_open: bool,
    notice: Option<String>,
    message: Option<Message>,
    /// The width the user set with Ctrl+←/→ or a drag; `None` keeps the default (Scouter `preferredWidth`).
    sidebar: Option<i64>,
    /// An edge drag in progress: the pointer's start column and the sidebar's width then.
    drag: Option<(u16, u16)>,
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
            reader_ready: self.shown.is_some(),
            previewing: self.shown.previewing(),
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
    syntax::register();
    // A file opens its folder, with the file revealed (Scouter `reveal`, `src/app/app.ts:344-356`).
    let (root, pending) = if target.is_dir() {
        (target.to_path_buf(), None)
    } else {
        let parent = target.parent().unwrap_or(target).to_path_buf();
        (parent, Some(Pending { target: target.to_path_buf(), expand: false, open: true }))
    };
    let model = Rc::new(RefCell::new(Model {
        location: target.to_string_lossy().into_owned(),
        focus: Focus::List,
        help_open: false,
        mode: Mode::Dark,
        background: None,
        parser: Parser::default(),
        proxy: app.proxy(),
        browser: Browser::new(Tree::new(root.clone())),
        marks: Marks::default(),
        pending,
        shown: reader::Shown::None,
        loading: false,
        focus_reader_on_open: false,
        notice: None,
        message: None,
        sidebar: None,
        drag: None,
    }));
    let ui = Rc::new(RefCell::new(build(&app, model.borrow().palette())));
    paint(&app, &mut model.borrow_mut(), &mut ui.borrow_mut());
    tree::list(&mut model.borrow_mut(), root);
    tree::refresh_marks(&mut model.borrow_mut());

    theme::watch(&app, {
        let (model, ui) = (model.clone(), ui.clone());
        move |app, SystemTheme { mode, background }| {
            let mut m = model.borrow_mut();
            m.mode = mode;
            m.background = background;
            restyle(app, &mut m, &mut ui.borrow_mut());
        }
    });
    app.on_resize({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, _, _| {
            paint(cx.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
            repaint_after_layout(cx.app(), &model, &ui);
        }
    });
    app.on_message({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event: &Event| {
            let mut m = model.borrow_mut();
            let mut u = ui.borrow_mut();
            tree::apply(&mut m, &u, event);
            paint(cx.app(), &mut m, &mut u);
            repaint_after_layout(cx.app(), &model, &ui);
        }
    });
    app.on_key_capture({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event| {
            let Some(named) = keys::name(event) else { return };
            let mut m = model.borrow_mut();
            // Scouter clears the reader's error notice on every key (`src/app/app.ts:531-534`).
            m.notice = None;
            let state = m.key_state();
            let Step::Done(action) = m.parser.feed(named.key(), state) else {
                paint(cx.app(), &mut m, &mut ui.borrow_mut());
                return;
            };
            match action {
                Action::Quit => return cx.quit(),
                Action::Help => m.help_open = true,
                Action::CloseHelp => m.help_open = false,
                // Tab moves to the reader only when it shows a document (Scouter `switchPane`).
                Action::SwitchPane if m.focus == Focus::List && m.shown.is_some() => m.focus = Focus::Reader,
                Action::SwitchPane | Action::Escape => m.focus = Focus::List,
                Action::Resize(step) => resize_sidebar(&mut m, cx.app(), step),
                Action::ToggleMarkdown => reader::toggle_markdown(&mut m),
                // The rendered view has no search or paragraph motion: the footer says which key shows the source.
                Action::Search | Action::Paragraph(_) if m.focus == Focus::Reader && m.shown.previewing() => {
                    let text = if action == Action::Search { SEARCH_REFUSED } else { PARAGRAPH_REFUSED };
                    reader::refuse(&mut m, text);
                }
                Action::Move(_)
                | Action::MovePage(_)
                | Action::Top
                | Action::Bottom
                | Action::Toggle
                | Action::Expand
                | Action::Collapse
                | Action::GoUp
                    if m.focus == Focus::List =>
                {
                    tree::act(&mut m, action)
                }
                _ => {}
            }
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
        }
    });
    wire_mouse(&model, &ui);

    #[cfg(debug_assertions)]
    if std::env::var_os("FMC_PANIC_AFTER_START").is_some() {
        app.set_timeout(std::time::Duration::from_millis(200), |_| panic!("Peeves was here"));
    }
    app.run()
}

/// Mouse on the list (click to select and open, wheel to scroll) and on the sidebar's edge (drag to resize).
fn wire_mouse(model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>) {
    let list = ui.borrow().tree.root.clone();
    list.on_mouse({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event| {
            let mut m = model.borrow_mut();
            match event.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    m.focus = Focus::List;
                    let row = line_at(event.y, LIST_TOP, m.browser.height()).and_then(|line| m.browser.row_at(line));
                    let Some(index) = row else { return };
                    m.browser.select_index(index);
                    tree::activate(&mut m);
                }
                MouseEventKind::ScrollUp => m.browser.scroll(-1),
                MouseEventKind::ScrollDown => m.browser.scroll(1),
                _ => return,
            }
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
        }
    });

    let handle = ui.borrow().handle.clone();
    handle.on_hover({
        let (model, ui) = (model.clone(), ui.clone());
        move |_, hovered| {
            let p = model.borrow().palette();
            let edge = if hovered { color(p.accent_soft) } else { Color::TRANSPARENT };
            ui.borrow().handle.border_color(edge);
        }
    });
    handle.on_mouse({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event| {
            let mut m = model.borrow_mut();
            match event.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    let width = sidebar::fit(cx.app().size().0, m.sidebar);
                    m.drag = Some((event.x, width));
                }
                MouseEventKind::Drag(MouseButton::Left) => {
                    let Some((start_x, start_width)) = m.drag else { return };
                    m.sidebar = Some(sidebar::dragged(start_width, start_x, event.x));
                }
                MouseEventKind::Up(MouseButton::Left) => m.drag = None,
                _ => return,
            }
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
        }
    });
}

/// Paints again once the frame has laid out the new text: a rendered Markdown document's rows, which the scrollbar
/// and the footer count, exist only after that layout.
fn repaint_after_layout(app: &App, model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>) {
    let (model, ui) = (model.clone(), ui.clone());
    app.set_timeout(std::time::Duration::from_millis(20), move |cx| {
        paint(cx.app(), &mut model.borrow_mut(), &mut ui.borrow_mut())
    });
}

/// Ctrl+←/→: two columns narrower or wider, kept for the session (Scouter `nudge`).
fn resize_sidebar(m: &mut Model, app: &App, step: i8) {
    let columns = app.size().0;
    let width = i64::from(sidebar::fit(columns, m.sidebar)) + 2 * i64::from(step);
    m.sidebar = Some(i64::from(sidebar::constrain(columns, width)));
}

fn build(app: &App, p: &Palette) -> Ui {
    let layout = View::new(app).width(Length::percent(100.0)).height(Length::percent(100.0)).column();
    let panes = View::new(app).width(Length::percent(100.0)).row().grow(1.0).min_height(0);
    let browser_panel = View::new(app).shrink(0.0).column().border(BorderStyle::Rounded);
    let browser_header = PaneHeader::new(app);
    let tree = TreeView::new(app);
    // The sidebar's right edge: a one-column strip over the panel's border, between its top and bottom rows.
    let handle = View::new(app)
        .absolute()
        .width(1)
        .position(Edge::Top, 1)
        .position(Edge::Bottom, 1)
        .border_sides(Sides::LEFT)
        .border_color(Color::TRANSPARENT)
        .mouse_pointer(MousePointer::EW_RESIZE);
    let viewer_panel = View::new(app).grow(1.0).min_width(0).column().border(BorderStyle::Rounded);
    let viewer_header = PaneHeader::new(app);
    let welcome = welcome::welcome(app, p);
    let reader = CodeView::new(app, p);
    let markdown = MarkdownView::new(app, p);
    let footer = Footer::new(app);
    browser_panel.add(&browser_header.root).add(&tree.root);
    let stack = View::new(app).grow(1.0).min_width(0).min_height(0).column().overflow(Overflow::Hidden);
    let bar = ScrollBar::new(app);
    let body = View::new(app).grow(1.0).min_height(0).row().overflow(Overflow::Hidden);
    stack.add(&welcome).add(&reader.widget()).add(&markdown.widget());
    body.add(&stack).add(&bar.root());
    viewer_panel.add(&viewer_header.root).add(&body);
    panes.add(&browser_panel).add(&viewer_panel).add(&handle);
    layout.add(&panes).add(&footer.root);
    app.root().add(&layout);
    Ui {
        browser_panel,
        browser_header,
        tree,
        handle,
        viewer_panel,
        viewer_header,
        welcome,
        stack,
        reader,
        markdown,
        bar,
        footer,
        layout,
        help: None,
    }
}

/// Rebuilds the parts that bake colors in (welcome, help), then repaints.
fn restyle(app: &App, m: &mut Model, ui: &mut Ui) {
    ui.welcome.detach();
    ui.welcome = welcome::welcome(app, m.palette());
    ui.stack.add(&ui.welcome);
    ui.reader.restyle(m.palette());
    ui.markdown.restyle(m.palette());
    if let Some(help) = ui.help.take() {
        help.overlay.detach();
    }
    paint(app, m, ui);
}

fn paint(app: &App, m: &mut Model, ui: &mut Ui) {
    let p = m.palette();
    let (columns, rows) = app.size();
    m.browser.set_height(usize::from(rows).saturating_sub(LIST_CHROME_ROWS));
    let measure = |text: &str| {
        app.session().context().text_width(text, app.width_method()).map_or(text.chars().count(), |w| w as usize)
    };
    let tint = m.tint();

    let width = sidebar::fit(columns, m.sidebar);
    ui.browser_panel.width(width);
    ui.handle.position(Edge::Left, width.saturating_sub(1));
    let focused = |pane: Focus| color(if m.focus == pane { p.accent } else { p.border });
    ui.browser_panel.border_color(focused(Focus::List));
    ui.viewer_panel.border_color(focused(Focus::Reader));

    let root = m.browser.tree.root().to_path_buf();
    let name = root.file_name().map_or(String::new(), |n| fmc::text::display_label(&n.to_string_lossy()));
    let sidebar = HeaderState {
        title: name,
        badges: vec![Badge::quiet("files")],
        focused: m.focus == Focus::List,
        ..Default::default()
    };
    ui.browser_header.paint(&sidebar, p);
    ui.tree.paint(app, &m.browser, &m.marks, p, &root);
    let (title, parent) = match &m.shown {
        reader::Shown::None => ("Reader".into(), None),
        reader::Shown::Text { path, .. } => reader::title_parts(&root, path),
    };
    let viewer = HeaderState {
        title,
        parent,
        badges: reader::badges(&m.shown),
        focused: m.focus == Focus::Reader,
        notice: m.notice.clone(),
    };
    ui.viewer_header.paint(&viewer, p);
    ui.welcome.visible(!m.shown.is_some());
    let previewing = m.shown.previewing();
    ui.reader.visible(m.shown.is_some() && !previewing);
    ui.markdown.visible(previewing);

    let home = std::env::var_os("HOME").map(PathBuf::from);
    let location = tildify(&fmc::text::display_label(&m.location), home.as_deref());
    let pane = if m.focus == Focus::List { Pane::Files } else { Pane::Reader };
    let viewport = usize::from(rows).saturating_sub(LIST_CHROME_ROWS);
    ui.bar.paint(app, reader::content_rows(&m.shown, ui.markdown.rows()), viewport, 0, viewport, p);
    let (document, position) = reader::footer_facts(&m.shown, viewport, ui.markdown.rows())
        .map_or((None, None), |(info, label)| (Some(info), Some(label)));
    let footer = FooterState {
        location,
        pane,
        message: m.message.clone(),
        document,
        positions: [position, None],
        pending: m.parser.pending_count(),
        ..FooterState::default()
    };
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

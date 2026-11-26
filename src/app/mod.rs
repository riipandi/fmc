//! The `opentui` app loop: the two panes and the footer, key and mouse dispatch, the tree's worker events, and theme
//! changes. Terminal restore on exit and on panic is OpenTUI's (`rust.md`, "Contracts and traps").

mod capture;
mod changes;
mod find;
mod keys;
mod layout;
mod poll;
mod preview;
mod reader;
mod scroll;
mod search;
mod theme;
mod tree;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use fmc::diff::{ChangeList, Marks};
use fmc::explorer::{Browser, Tree};
use fmc::keys::{Action, Focus, Parser, Screen, State, Step};
use fmc::reader::markdown::{PARAGRAPH_REFUSED, SEARCH_REFUSED};
use fmc::reader::position::scroll_label;
use fmc::settings::Parsed;
use fmc::theme::{Mode, Palette, SystemTheme, palette, tinted_background};
use opentui::prelude::*;

use crate::services::Event;
use crate::ui::binary::BinaryView;
use crate::ui::changes::ChangesView;
use crate::ui::diff::DiffBody;
use crate::ui::find::{FindBar, prompt_segments};
use crate::ui::footer::{Footer, FooterState, Message, tildify};
use crate::ui::header::{Badge, BadgeKind, HeaderState, PaneHeader};
use crate::ui::hints::{self, Pane, Prompt};
use crate::ui::markdown::MarkdownView;
use crate::ui::picture::Picture;
use crate::ui::reader::CodeView;
use crate::ui::scrollbar::ScrollBar;
use crate::ui::sidebar::Mark;
use crate::ui::styled::color;
use crate::ui::tree::{LIST_TOP, TreeView, line_at};
use crate::ui::viewer::Viewer;
use crate::ui::{syntax, welcome};

/// Screen rows the list does not use: the footer (2), the panel's borders (2), and the header (2).
const LIST_CHROME_ROWS: usize = 6;

struct Ui {
    browser_panel: View,
    browser_header: PaneHeader,
    tree: TreeView,
    changes: ChangesView,
    find: FindBar,
    handle: View,
    mark: Mark,
    viewer_panel: View,
    viewer_header: PaneHeader,
    welcome: View,
    /// The viewer body: the welcome, the code view, and the rendered view stacked (one shows at a time), beside the bar.
    stack: View,
    viewer: Viewer,
    /// The changes screen's stacked body, in the same stack as the viewer.
    diff: DiffBody,
    reader: CodeView,
    markdown: MarkdownView,
    picture: Picture,
    /// The preview of a text hit, in its own view beside the reader while the finder shows (Scouter `HitPreview`).
    preview: CodeView,
    preview_viewer: Viewer,
    /// What the preview shows, as the paint last drew it.
    preview_at: Option<find::PreviewAt>,
    binary: BinaryView,
    /// Two bars: beside plain text the bar is the thumb's column alone; beside filled content it has two blanks before it.
    bar: ScrollBar,
    narrow_bar: ScrollBar,
    footer: Footer,
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
    /// Wrapping, kept across files (Scouter `wrap`).
    wrap: bool,
    /// The places opened, back and forward (Scouter `JumpHistory`).
    jumps: fmc::reader::jump::History,
    /// The reader's viewport rows, and the row it last painted, for the motions and the jump history.
    viewport: usize,
    top: f64,
    /// The place a jump opens at: applied when that file shows.
    restore: Option<(PathBuf, usize)>,
    /// The in-file search, while there is one (Scouter `DocumentSearch`).
    search: Option<fmc::reader::search::Search>,
    /// The file finder, while it is open (Scouter `FileFinder`).
    finder: Option<find::Finder>,
    /// The text hit Enter opened, to show at its line once its file is read (Scouter `openAt`).
    open_at: Option<find::OpenAt>,
    /// The row the reader scrolls to once its file shows, and the paints left to set it in (`find::OPEN_SCROLL_TRIES`).
    pending_top: Option<(usize, u8)>,
    /// The last finder's number; its index worker answers to it.
    find_session: u64,
    /// The width the user set with Ctrl+←/→ or a drag; `None` keeps the default (Scouter `preferredWidth`).
    sidebar: Option<i64>,
    /// Whether the sidebar shows: the 80-column rule and the user's choice (`fmc::layout`).
    visibility: fmc::layout::Visibility,
    /// The changes list, kept between visits (Scouter's `ChangeList`).
    change_list: ChangeList,
    /// The changes screen, while it shows.
    changes: Option<changes::Changes>,
    /// The stacked body's scope, view, and context, kept between visits (Scouter's `DiffScreen`).
    diff_options: changes::Options,
    /// The listing in flight, by its session (Scouter `loading === "diff"`).
    changes_load: Option<u64>,
    /// The listing of a reload in flight, by its session (`r`); a reply for it refreshes the open screen.
    changes_reload: Option<u64>,
    /// A rebuilt stacked body was at its end: it scrolls to its new end once the frame has measured it.
    bottom_pending: bool,
    /// The next polls to schedule, which `poll::schedule` runs after each event.
    ticks: Vec<poll::Tick>,
    /// When the browse poll in flight started, and the changes poll's, for the back-off.
    browse_started: Option<std::time::Instant>,
    diff_started: Option<std::time::Instant>,
    /// The last session of a listing; each `d` takes the next.
    changes_session: u64,
    /// The file the body scrolls to once it is laid out, and the paints left to lay it out.
    diff_jump: Option<(String, u8)>,
    /// An edge drag in progress: the pointer's start column and the sidebar's width then.
    drag: Option<(u16, u16)>,
    /// Whether moving the list selection previews the file it selects (`p` turns it off; `settings.json` sets the start).
    auto_preview: bool,
    /// Counts the list's moves, so a preview is read only for the newest move (`app/preview.rs`).
    preview_generation: u64,
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
            // The changes keys hold while the body shows, and while the list has focus over a key page.
            screen: if self.changes.is_some() && (self.focus == Focus::List || !self.shown.shows_keys()) {
                Screen::Changes
            } else {
                Screen::Browse
            },
            focus: self.focus,
            reader_ready: self.shown.has_document(),
            previewing: self.shown.previewing(),
            // Ctrl+o and Alt+i are left to a prompt (Scouter `commandFor`: the finder and the search).
            typing: self.finder.is_some() || self.search.as_ref().is_some_and(|search| search.editing),
        }
    }
}

/// The state a fresh app starts with: the browse root, the startup `pending` reveal, and the empty reader.
fn new_model(app: &App, target: &Path, root: PathBuf, pending: Option<Pending>) -> Model {
    Model {
        location: target.to_string_lossy().into_owned(),
        focus: Focus::List,
        mode: Mode::Dark,
        background: None,
        parser: Parser::default(),
        proxy: app.proxy(),
        browser: Browser::new(Tree::new(root.clone())),
        marks: Marks::default(),
        pending,
        shown: reader::Shown::None,
        wrap: false,
        jumps: Default::default(),
        viewport: 0,
        top: 0.0,
        restore: None,
        search: None,
        finder: None,
        open_at: None,
        pending_top: None,
        find_session: 0,
        loading: false,
        focus_reader_on_open: false,
        notice: None,
        message: None,
        sidebar: None,
        visibility: Default::default(),
        drag: None,
        change_list: ChangeList::default(),
        changes: None,
        changes_load: None,
        changes_reload: None,
        changes_session: 0,
        ticks: Vec::new(),
        bottom_pending: false,
        browse_started: None,
        diff_started: None,
        diff_options: changes::Options::default(),
        diff_jump: None,
        auto_preview: fmc::settings::AUTO_PREVIEW,
        preview_generation: 0,
    }
}

/// Runs the app on `target` until the user quits.
pub fn run(target: &Path, settings: Parsed) -> opentui::Result<()> {
    // Scouter draws on the terminal's own background (`RGBA.defaultBackground()`), not a theme color.
    let [dark, light] = [Theme::dark(), Theme::light()].map(|t| Theme { background: Color::DEFAULT_BACKGROUND, ..t });
    let options = AppOptions {
        palette_size: theme::PALETTE_SIZE,
        tab_focus: false,
        // A drag selects and keeps the selection; `y` copies it (architecture.md, approved deviation).
        copy_on_select: false,
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
    let model = Rc::new(RefCell::new(new_model(&app, target, root.clone(), pending)));
    {
        let mut m = model.borrow_mut();
        m.auto_preview = settings.settings.auto_preview;
        if !settings.notices.is_empty() {
            m.message = Some(Message { text: settings.notices.join(" "), error: true });
        }
    }
    let ui = Rc::new(RefCell::new(build(&app, model.borrow().palette())));
    paint(&app, &mut model.borrow_mut(), &mut ui.borrow_mut());
    tree::list(&mut model.borrow_mut(), root);
    tree::refresh_marks(&mut model.borrow_mut());
    model.borrow_mut().ticks.push(poll::Tick { chain: poll::Chain::Browse, delay: poll::POLL });
    poll::schedule(&app, &model, &ui);

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
            {
                let mut m = model.borrow_mut();
                let mut u = ui.borrow_mut();
                tree::apply(&mut m, &u, event);
                paint(cx.app(), &mut m, &mut u);
            }
            repaint_after_layout(cx.app(), &model, &ui);
            poll::schedule(cx.app(), &model, &ui);
        }
    });
    capture::wire(&app, &model, &ui);
    {
        let (model, ui) = (model.clone(), ui.clone());
        let viewer = ui.borrow().viewer.widget();
        viewer.on_scroll(move |cx, _| paint(cx.app(), &mut model.borrow_mut(), &mut ui.borrow_mut()));
    }
    {
        // The list follows the body only when the body scrolls, as Scouter's `layoutChanged` does; a repaint alone
        // (a focus change, say) leaves the selection where it is.
        let (model, ui) = (model.clone(), ui.clone());
        let stacked = ui.borrow().diff.scroller.widget();
        stacked.on_scroll(move |cx, _| {
            changes::follow(&mut model.borrow_mut(), &ui.borrow());
            paint(cx.app(), &mut model.borrow_mut(), &mut ui.borrow_mut())
        });
    }
    {
        // A left click on a file's header folds the file (Scouter `createHeader`); the rows are the last frame's.
        let (model, ui) = (model.clone(), ui.clone());
        let stacked = ui.borrow().diff.scroller.widget();
        stacked.on_mouse(move |cx, event| {
            if event.kind != MouseEventKind::Down(MouseButton::Left) {
                return;
            }
            let mut m = model.borrow_mut();
            if !changes::body_shows(&m) {
                return;
            }
            changes::click_header(&mut m, &ui.borrow(), f64::from(event.y));
            paint(cx.app(), &mut m, &mut ui.borrow_mut())
        });
    }
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
                    // A press from another pane only focuses the list; the next press selects the row.
                    let focused = m.focus == Focus::List;
                    m.focus = Focus::List;
                    let row = line_at(event.y, LIST_TOP, m.browser.height())
                        .and_then(|line| m.browser.row_at(line))
                        .filter(|_| focused);
                    if let Some(index) = row {
                        m.browser.select_index(index);
                        // A click on a finder row picks it, as Enter does (Scouter `onExplorerActivate`).
                        if m.finder.is_some() {
                            find::pick(&mut m);
                        } else {
                            tree::click(&mut m);
                        }
                    }
                }
                MouseEventKind::ScrollUp => m.browser.scroll(-1),
                MouseEventKind::ScrollDown => m.browser.scroll(1),
                _ => return,
            }
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
            drop(m);
            preview::moved(cx.app(), &model);
        }
    });

    // The changes list: a click selects its row and opens it (Scouter `ChangeList`), and the wheel scrolls it.
    let changes_list = ui.borrow().changes.root.clone();
    changes_list.on_mouse({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event| {
            let mut m = model.borrow_mut();
            match event.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    // The row's action may move the focus to the body, so the list's press stops here (the pane's own
                    // focus handler is further up the path).
                    let focused = m.focus == Focus::List;
                    m.focus = Focus::List;
                    let line = line_at(event.y, LIST_TOP, m.change_list.height());
                    if let Some(index) = line.filter(|_| focused).and_then(|line| m.change_list.row_at(line)) {
                        changes::click_row(&mut m, index);
                    }
                    cx.stop_propagation();
                }
                MouseEventKind::ScrollUp => m.change_list.scroll(-1),
                MouseEventKind::ScrollDown => m.change_list.scroll(1),
                _ => return,
            }
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
        }
    });

    // A press anywhere in a pane focuses it, its header and its bar included (approved deviation, architecture.md). A row
    // press has already set the focus and stopped here, so the row's own action is what the focus ends on.
    let (sidebar, viewer) = {
        let u = ui.borrow();
        (u.browser_panel.clone(), u.viewer_panel.clone())
    };
    focus_on_press(model, ui, &sidebar, Focus::List);
    focus_on_press(model, ui, &viewer, Focus::Reader);

    // The bars: a drag, a track click, and a wheel step move the pane each bar belongs to (Scouter's scroll box bars).
    let tree_track = ui.borrow().tree.track();
    wire_bar(
        model,
        ui,
        &tree_track,
        Focus::List,
        |m, _, rows| m.browser.scroll(rows),
        |m, _, row| m.browser.scroll(row as isize - m.browser.top() as isize),
    );
    let changes_track = ui.borrow().changes.track();
    wire_bar(
        model,
        ui,
        &changes_track,
        Focus::List,
        |m, _, rows| m.change_list.scroll(rows),
        |m, _, row| m.change_list.scroll(row as isize - m.change_list.top() as isize),
    );
    let (text_track, rendered_track) = {
        let u = ui.borrow();
        (u.bar.track(), u.narrow_bar.track())
    };
    for track in [&text_track, &rendered_track] {
        wire_bar(
            model,
            ui,
            track,
            Focus::Reader,
            |m, u, rows| {
                bar_box(m, u).scroll_by(0.0, rows as f64);
            },
            |m, u, row| {
                bar_box(m, u).scroll_to(0.0, row as f64);
            },
        );
    }

    layout::wire(model, ui);
}

/// Focuses `pane` on a left press anywhere in `panel`.
fn focus_on_press(model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>, panel: &View, pane: Focus) {
    panel.on_mouse({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event| {
            if event.kind != MouseEventKind::Down(MouseButton::Left) {
                return;
            }
            let mut m = model.borrow_mut();
            m.focus = pane;
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
        }
    });
}

/// Wires a bar's drag (the slider's press, drag, and track click), its wheel, and the press that focuses its pane. A
/// wheel step scrolls the pane by one row and stops there, as the wheel does over the text (Scouter's scroll box).
fn wire_bar(
    model: &Rc<RefCell<Model>>,
    ui: &Rc<RefCell<Ui>>,
    track: &Slider,
    pane: Focus,
    step: fn(&mut Model, &Ui, isize),
    to: fn(&mut Model, &Ui, usize),
) {
    track.on_mouse({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, event| {
            let mut m = model.borrow_mut();
            match event.kind {
                MouseEventKind::Down(MouseButton::Left) => m.focus = pane,
                MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                    let rows = if event.kind == MouseEventKind::ScrollUp { -1 } else { 1 };
                    step(&mut m, &ui.borrow(), rows);
                    cx.stop_propagation();
                }
                _ => return,
            }
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
        }
    });
    track.on_change({
        let (model, ui) = (model.clone(), ui.clone());
        move |cx, value| {
            let mut m = model.borrow_mut();
            to(&mut m, &ui.borrow(), value.round() as usize);
            paint(cx.app(), &mut m, &mut ui.borrow_mut());
        }
    });
}

/// The scroll box the reader's bar moves: the preview while the finder shows one, the stacked body on the changes
/// screen, and the viewer otherwise. Each is the box whose rows the bar's thumb counts.
fn bar_box(m: &Model, ui: &Ui) -> ScrollBox {
    let previewing = m.finder.as_ref().and_then(|finder| finder.preview.as_ref()).is_some_and(|p| p.shown.is_some());
    if previewing {
        ui.preview_viewer.widget()
    } else if changes::body_shows(m) {
        ui.diff.scroller.widget()
    } else {
        ui.viewer.widget()
    }
}

/// Paints again once the frame has laid out the new text: a rendered Markdown document's rows, which the scrollbar
/// and the footer count, exist only after that layout. The changes body's blocks are measured the same way, and a
/// frame can land after the first repaint, so that repaint keeps going until the blocks have their rows.
fn repaint_after_layout(app: &App, model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>) {
    repaint_later(app, model, ui, LAYOUT_TRIES);
}

/// How many more repaints a changes body may take to be measured after its blocks are built.
const LAYOUT_TRIES: u8 = 10;

fn repaint_later(app: &App, model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>, tries: u8) {
    let (model, ui) = (model.clone(), ui.clone());
    app.set_timeout(std::time::Duration::from_millis(20), move |cx| {
        let (mut m, mut u) = (model.borrow_mut(), ui.borrow_mut());
        paint(cx.app(), &mut m, &mut u);
        if m.bottom_pending && u.diff.tops().is_some() {
            m.bottom_pending = false;
            u.diff.scroller.scroll_to_bottom();
        }
        let unmeasured = changes::body_shows(&m) && !m.change_list.changes().is_empty() && u.diff.tops().is_none();
        drop((m, u));
        if tries > 0 && unmeasured {
            cx.app().request_redraw();
            repaint_later(cx.app(), &model, &ui, tries - 1);
        }
    });
}

/// Clears the footer's message two seconds on, as Scouter does after a copy; a later message is cleared too early
/// only if it lands in those two seconds, as in Scouter.
fn clear_message_later(app: &App, model: &Rc<RefCell<Model>>, ui: &Rc<RefCell<Ui>>) {
    let (model, ui) = (model.clone(), ui.clone());
    app.set_timeout(std::time::Duration::from_secs(2), move |cx| {
        model.borrow_mut().message = None;
        paint(cx.app(), &mut model.borrow_mut(), &mut ui.borrow_mut())
    });
}

/// The columns the reader's pane takes, which the stacked body fills and `s` measures (Scouter `this.body.width`).
fn pane_columns(app: &App, m: &Model) -> u32 {
    let columns = app.size().0;
    u32::from(columns.saturating_sub(layout::width(m, columns) + 3))
}

fn build(app: &App, p: &Palette) -> Ui {
    let layout = View::new(app).width(Length::percent(100.0)).height(Length::percent(100.0)).column();
    let panes = View::new(app).width(Length::percent(100.0)).row().grow(1.0).min_height(0);
    let browser_panel = View::new(app).shrink(0.0).column().border(BorderStyle::Rounded);
    let browser_header = PaneHeader::new(app);
    let tree = TreeView::new(app);
    let changes_list = ChangesView::new(app);
    let finder_bar = FindBar::new(app);
    let handle = layout::handle(app);
    let mark = Mark::new(app);
    let viewer_panel = View::new(app).grow(1.0).min_width(0).column().border(BorderStyle::Rounded);
    let viewer_header = PaneHeader::new(app);
    let welcome = welcome::welcome(app, p);
    let reader = CodeView::new(app, p);
    let markdown = MarkdownView::new(app, p);
    let viewer = Viewer::new(app);
    let picture = Picture::new(app);
    let binary = BinaryView::new(app);
    let preview = CodeView::new(app, p);
    let preview_viewer = Viewer::new(app);
    preview_viewer.visible(false);
    preview_viewer.widget().add(&preview.widget());
    let diff = DiffBody::new(app);
    let footer = Footer::new(app);
    browser_panel.add(&browser_header.root).add(&finder_bar.root).add(&tree.root).add(&changes_list.root);
    let stack = View::new(app).grow(1.0).min_width(0).min_height(0).column().overflow(Overflow::Hidden);
    let bar = ScrollBar::new(app, 2);
    let narrow_bar = ScrollBar::new(app, 0);
    let body = View::new(app).grow(1.0).min_height(0).row().overflow(Overflow::Hidden);
    viewer.widget().add(&reader.widget()).add(&markdown.widget()).add(&picture.widget());
    stack.add(&welcome).add(&binary.root()).add(&viewer.widget()).add(&preview_viewer.widget()).add(&diff.root);
    body.add(&stack).add(&narrow_bar.root()).add(&bar.root());
    viewer_panel.add(&viewer_header.root).add(&body);
    panes.add(&browser_panel).add(&viewer_panel).add(&handle).add(&mark.root);
    layout.add(&panes).add(&footer.root);
    app.root().add(&layout);
    Ui {
        browser_panel,
        browser_header,
        tree,
        changes: changes_list,
        find: finder_bar,
        handle,
        mark,
        viewer_panel,
        viewer_header,
        welcome,
        stack,
        viewer,
        diff,
        reader,
        markdown,
        picture,
        preview,
        preview_viewer,
        preview_at: None,
        binary,
        bar,
        narrow_bar,
        footer,
    }
}

/// Rebuilds the parts that bake colors in (welcome), then repaints.
fn restyle(app: &App, m: &mut Model, ui: &mut Ui) {
    ui.welcome.detach();
    ui.welcome = welcome::welcome(app, m.palette());
    ui.stack.add(&ui.welcome);
    ui.reader.restyle(m.palette());
    ui.markdown.restyle(m.palette());
    paint(app, m, ui);
}

fn paint(app: &App, m: &mut Model, ui: &mut Ui) {
    let p = m.palette();
    let (columns, rows) = app.size();
    let viewport = usize::from(rows).saturating_sub(LIST_CHROME_ROWS);
    m.browser.set_height(viewport);
    m.change_list.set_height(viewport);
    let measure = |text: &str| {
        app.session().context().text_width(text, app.width_method()).map_or(text.chars().count(), |w| w as usize)
    };
    let tint = m.tint();

    layout::sync(m, columns);
    let width = layout::lay_out(ui, m, columns, p);
    let focused = |pane: Focus| color(if m.focus == pane { p.accent } else { p.border });
    ui.browser_panel.border_color(focused(Focus::List));
    ui.viewer_panel.border_color(focused(Focus::Reader));

    // The changes screen replaces the tree in the sidebar and the reader in the viewer, unless a key page is open.
    let on_changes = m.changes.is_some();
    let body = changes::body_shows(m);
    let root = m.browser.tree.root().to_path_buf();
    let name = root.file_name().map_or(String::new(), |n| fmc::text::display_label(&n.to_string_lossy()));
    let sidebar = if on_changes {
        changes::sidebar_header(m)
    } else {
        HeaderState {
            title: name,
            badges: vec![Badge::quiet(if m.browser.dotfiles() { "files" } else { "no dotfiles" })],
            focused: m.focus == Focus::List,
            ..Default::default()
        }
    };
    // The finder's bar takes the header's place while it is open (Scouter `updateSidebarHeader`).
    ui.browser_header.root.visible(m.finder.is_none());
    ui.browser_header.paint(&sidebar, p);
    let bar = m.finder.as_ref().map(|finder| prompt_segments(find::prompt(finder).0, &finder.query, &finder.status, p));
    ui.find.paint(bar, p);
    ui.tree.root.visible(!on_changes);
    ui.changes.root.visible(on_changes);
    if on_changes {
        ui.changes.paint(app, &m.change_list, p);
    } else {
        ui.tree.paint(app, &m.browser, &m.marks, p, &root);
    }
    // A preview names its file, with a `preview` pill in place of the reader's badges (Scouter `viewerHeaderFor`).
    let previewed =
        m.finder.as_ref().and_then(|finder| finder.preview.as_ref()).and_then(|preview| preview.shown.as_ref());
    let diff_header = body.then(|| changes::viewer_header(m));
    let (title, parent) = match (previewed, &diff_header, &m.shown) {
        (Some(shown), _, _) => reader::title_parts(&root, &shown.path),
        (None, Some((title, parent, _)), _) => (title.clone(), parent.clone()),
        (None, None, reader::Shown::None) => ("Reader".into(), None),
        (None, None, reader::Shown::Keys { .. }) => ("Keys".into(), None),
        (
            None,
            None,
            reader::Shown::Text { path, .. } | reader::Shown::Image { path, .. } | reader::Shown::Binary { path, .. },
        ) => reader::title_parts(&root, path),
    };
    let badges = match (previewed, diff_header) {
        (Some(_), _) => vec![Badge { text: "preview".into(), kind: BadgeKind::Pill }],
        (None, Some((_, _, badges))) => badges,
        (None, None) => reader::badges(&m.shown, m.wrap),
    };
    let viewer = HeaderState { title, parent, badges, focused: m.focus == Focus::Reader, notice: m.notice.clone() };
    ui.viewer_header.paint(&viewer, p);
    // The preview takes the reader's place, and hides the welcome (Scouter `previewHit`).
    let shows_preview = previewed.is_some();
    // The preview's line count, copied out: the model is changed below, which the borrow of `previewed` forbids.
    let preview_lines = previewed.map(|shown| shown.document.lines);
    ui.welcome.visible(!m.shown.is_some() && !shows_preview && !on_changes);
    let previewing = m.shown.previewing();
    ui.reader.visible(m.shown.text().is_some() && !previewing && !shows_preview && !body);
    ui.markdown.visible(previewing && !shows_preview && !body);
    ui.picture.visible(matches!(m.shown, reader::Shown::Image { .. }) && !shows_preview && !body);
    ui.binary.visible(matches!(m.shown, reader::Shown::Binary { .. }) && !shows_preview && !body);
    // The reader's pane takes no space beside the preview, which is the pane the stack shows then. With no document it
    // takes none either: an empty viewer would halve the stack and push the welcome out of its half.
    ui.viewer.visible(m.shown.is_some() && !shows_preview && !body);
    ui.diff.root.visible(body);
    if on_changes {
        ui.diff.set_split(m.diff_options.split);
        // A body at its end stays at the end when a rebuild changes its rows (Scouter's stack does the same).
        let (at_bottom, built) = (ui.diff.at_bottom(viewport), ui.diff.built_key());
        changes::sync(app, m, ui, p, &tint);
        ui.diff.set_wrap(m.wrap);
        // The stack gives its bar's column back only while the text overflows (the bar takes no space when hidden), as
        // Scouter's diff scroller does; a fixed pane would leave that column empty.
        let overflow = ui.diff.height() > viewport;
        ui.diff.set_pane(u32::from(columns.saturating_sub(width + 3)) + u32::from(!overflow));
        // The end is held once the frame has measured the rebuilt blocks: a scroll before that picks the wrong file.
        if at_bottom && ui.diff.built_key() != built {
            m.bottom_pending = true;
        }
        changes::jump(m, ui);
    }
    if let Some((top, tries)) = m.pending_top.take() {
        ui.viewer.scroll_to(0.0, top as f64);
        if (ui.viewer.position().1 - top as f64).abs() >= 0.5 && tries > 0 {
            m.pending_top = Some((top, tries - 1));
        }
    }
    if let reader::Shown::Binary { path, size } = &m.shown {
        let name = fmc::text::display_label(&path.file_name().unwrap_or_default().to_string_lossy());
        ui.binary.paint(&name, &fmc::reader::binary::format_size(*size), p);
    }

    let home = std::env::var_os("HOME").map(PathBuf::from);
    let location = tildify(&fmc::text::display_label(&m.location), home.as_deref());
    let pane = if m.focus == Focus::List { Pane::Files } else { Pane::Reader };
    // The position is the one the last frame laid out; a move made since then lands in the next frame.
    m.viewport = viewport;
    m.top = if body { ui.diff.scroller.position().1 } else { ui.viewer.position().1 };
    find::paint_preview(m, ui, p);
    // A preview is unwrapped, as Scouter's is; it takes the same width as the reader. The stacked diff's bar overlays
    // its last column, as Scouter's diff scroller does (no blank before the thumb).
    let fill = !body && !shows_preview && (m.wrap || m.shown.text().is_none() || previewing);
    ui.viewer.set_wrap(fill);
    ui.reader.set_wrap(m.wrap);
    ui.preview.set_wrap(false);
    // Unwrapped text takes the pane's inner width, the bar's column included: the pan range is that width less the
    // viewport, and it stops where Scouter's does (issue-00008).
    ui.reader.set_width(u32::from(columns.saturating_sub(width + 2)));
    ui.preview.set_width(u32::from(columns.saturating_sub(width + 2)));
    ui.picture.fill(viewport as u32);
    let lines = m.shown.text().map_or(0, |(document, ..)| document.lines);
    let total = match preview_lines {
        _ if body => ui.diff.height(),
        Some(lines) => lines,
        None => reader::content_rows(&m.shown, ui.markdown.rows(), ui.reader.rows(lines)),
    };
    // A preview has its own position, as Scouter's preview scroll box does.
    let top = match preview_lines {
        Some(_) => ui.preview_viewer.position().1.floor() as usize,
        None => m.top.floor() as usize,
    };
    // The rendered view's bar takes one column, as Scouter's scroll box does: the blank before the thumb is wrapped
    // text's and a picture's, not the rendered view's (`scouter:src/app/reader-pane.ts:31-48`).
    let gapped = fill && !previewing;
    ui.bar.paint(app, total, viewport, top, gapped, p);
    ui.narrow_bar.paint(app, total, viewport, top, !gapped, p);
    let has_blocks = m.changes.as_ref().is_some_and(|changes| !changes.files.is_empty());
    let (document, position) = if body {
        (None, has_blocks.then(|| scroll_label(ui.diff.height(), viewport, m.top)))
    } else {
        match preview_lines {
            Some(_) => (None, None),
            None => reader::footer_facts(&m.shown, viewport, ui.markdown.rows(), ui.reader.rows(lines), m.top)
                .map_or((None, None), |(info, label)| (Some(info), label)),
        }
    };
    let search_text = m.shown.text().map_or_else(String::new, |(document, ..)| document.content().to_owned());
    ui.reader.sync_search(
        p,
        m.search.as_ref().map_or("", |search| search.query.as_str()),
        m.search.as_ref().and_then(|search| search.current_line()),
        &search_text,
    );
    let footer = FooterState {
        location,
        mode: if body { hints::Mode::Diff } else { hints::Mode::Browse },
        pane,
        prompt: match (&m.finder, &m.search) {
            (Some(finder), _) => Some(find::prompt(finder).1),
            (None, Some(search)) if search.editing => Some(Prompt::Reader),
            _ => None,
        },
        message: m.message.clone(),
        document,
        diff: if body { changes::footer(m, ui) } else { None },
        positions: if body { [None, position] } else { [position, None] },
        wrap: m.wrap,
        pending: m.parser.pending_count(),
        sidebar_hidden: !layout::shown(m, columns),
    };
    ui.footer.set_background(&tint);
    ui.footer.paint(&footer, usize::from(columns), p, &measure);
}

#[cfg(test)]
#[path = "focus_tests.rs"]
mod focus_tests;

#[cfg(test)]
#[path = "layout_tests.rs"]
mod layout_tests;

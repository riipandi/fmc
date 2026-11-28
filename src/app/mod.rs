//! The `opentui` app loop: the two panes and the footer, key and mouse dispatch, the tree's worker events, and theme
//! changes. Terminal restore on exit and on panic is OpenTUI's.

mod capture;
mod changes;
mod command;
mod find;
mod gutter;
mod hex;
mod keys;
mod layout;
mod mouse;
mod open;
mod overlay;
mod pages;
mod picture;
mod poll;
mod preview;
mod reader;
mod remember;
mod scroll;
mod search;
mod settings;
mod theme;
mod tree;
mod write;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use fmc::diff::{ChangeList, Marks};
use fmc::explorer::{Browser, Tree};
use fmc::keys::{Action, Focus, Parser, Screen, State, Step};
use fmc::layout::{Page, Tier, tier};
use fmc::line_edit::Line;
use fmc::reader::markdown::PARAGRAPH_REFUSED;
use fmc::reader::position::scroll_label;
use fmc::reader::rendered_signs::block_marks;
use fmc::settings::Parsed;
use fmc::theme::catalog::{self, Chosen};
use fmc::theme::{DARK, Hex, LIGHT, Mode, ModeChoice, Palette, SystemTheme, page_background};
use mouse::wire_mouse;
use opentui::prelude::*;

use crate::services::{self, Event};
use crate::ui::binary::BinaryView;
use crate::ui::changes::ChangesView;
use crate::ui::diff::DiffBody;
use crate::ui::find::{FindBar, PromptLine};
use crate::ui::footer::{Footer, FooterState, Message, tildify};
use crate::ui::header::{Badge, BadgeKind, HeaderState, PaneHeader};
use crate::ui::hex::HexView;
use crate::ui::hints::{self, Pane, Prompt};
use crate::ui::markdown::MarkdownView;
use crate::ui::markdown_signs::SignStrip;
use crate::ui::picture::Picture;
use crate::ui::reader::CodeView;
use crate::ui::scrollbar::ScrollBar;
use crate::ui::settings::SettingsView;
use crate::ui::sidebar::Mark;
use crate::ui::styled::color;
use crate::ui::tree::{LIST_TOP, TreeView, line_at};
use crate::ui::viewer::Viewer;
use crate::ui::{syntax, welcome};

/// Screen rows the list does not use: the footer (2), the panel's borders (2), and the header (2). Compact has a one-row
/// footer, the bottom bar, and the same borders and header.
const LIST_CHROME_ROWS: usize = 6;
const COMPACT_CHROME_ROWS: usize = 5;

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
    welcome: welcome::Welcome,
    /// The viewer body: the welcome, the code view, and the rendered view stacked (one shows at a time), beside the bar.
    stack: View,
    viewer: Viewer,
    /// The changes screen's stacked body, in the same stack as the viewer.
    diff: DiffBody,
    reader: CodeView,
    markdown: MarkdownView,
    /// The git signs beside the rendered view's blocks, over the viewer's left edge.
    markdown_signs: SignStrip,
    picture: Picture,
    /// The preview of a text hit, in its own view beside the reader while the finder shows (Scouter `HitPreview`).
    preview: CodeView,
    preview_viewer: Viewer,
    /// What the preview shows, as the paint last drew it.
    preview_at: Option<find::PreviewAt>,
    binary: BinaryView,
    hex: HexView,
    /// Two bars: beside plain text the bar is the thumb's column alone; beside filled content it has two blanks before it.
    bar: ScrollBar,
    narrow_bar: ScrollBar,
    footer: Footer,
    /// The context menu and the dialog, above the page (`app/overlay.rs`).
    overlay: overlay::Views,
    /// The Settings page's rows and its value column, in the content stack (`app/settings.rs`).
    settings: SettingsView,
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
    focus: Focus,
    /// The terminal's mode, from its background (`app/theme.rs`).
    mode: Mode,
    background: Option<String>,
    /// The palettes the settings chose for each mode, and whether `theme_mode` forces one.
    dark: Palette,
    light: Palette,
    choice: ModeChoice,
    parser: Parser,
    proxy: AppProxy,
    /// The image headers the rendered view's boxes read, shared by the workers (`app/picture.rs`).
    pictures: services::picture::Cache,
    browser: Browser,
    marks: Marks,
    pending: Option<Pending>,
    shown: reader::Shown,
    /// A file or folder open is in flight; another open is refused (Scouter `runLoading`).
    loading: bool,
    /// The next file shown is the startup file, which opens with the reader focused as in Scouter (`finishShowing`);
    /// a file opened from the tree keeps the list focused, where Scouter moves the focus to the reader.
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
    /// The `file_associations` table (`settings.json`), and the folder fmc opened: a path pattern matches from it.
    associations: fmc::reader::associations::Associations,
    opened: PathBuf,
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
    /// How the finder matches names: `find_mode` in `settings.json` sets the start, and `Ctrl+F` flips it for the run.
    matching: fmc::find::Matching,
    /// The rows `]` and `[` move the list by, and the lines they scroll the reader by (`settings.json`).
    sidebar_jump: u32,
    reader_jump: u32,
    /// Whether the reader marks git changes in its gutter (`settings.json`), and whether the list's root is a repository:
    /// the strip shows only when both hold (`app/gutter.rs`).
    git_gutter: bool,
    in_repo: bool,
    /// The branch of the browsed root, from the last browse poll (`None` outside a repository).
    branch: Option<String>,
    /// Counts each text shown, so a sign answer finds the showing it was asked for (`app/gutter.rs`).
    serial: u64,
    /// The `:` command line while it is open, with its caret (`app/command.rs`).
    command: Option<Line>,
    /// The terminal's width at the last paint: the tier reads it (`app/pages.rs`).
    columns: u16,
    /// The terminal's height at the last paint: a menu that scrolls sizes its window from it (`app/open.rs`).
    rows: u16,
    /// The page stack the compact tier draws.
    pages: fmc::layout::Pages,
    /// The open context menu or dialog, which takes every key and click until it closes (`app/overlay.rs`).
    overlay: Option<fmc::overlay::Overlay>,
    /// The cell a menu opened from the tree is anchored at.
    overlay_at: (u16, u16),
    /// What the open menu or dialog is for (`app/write.rs`); a dialog's task is kept while it reports an error.
    overlay_task: Option<write::Task>,
    /// The entries `c` or `x` put on the clipboard (`app/write.rs`); cut rows are drawn muted.
    clipboard: Option<write::Clip>,
    /// The run on the worker, while one is going: a second write is refused until it ends.
    bulk: Option<write::Bulk>,
    /// The Settings page's values, its page while it is open, and its saves (`app/settings.rs`).
    settings: settings::Store,
    /// The remembered UI state's file and its pending write (`app/remember.rs`).
    saves: remember::Saves,
}

impl Model {
    /// The mode in use: the one `theme_mode` forces, or the terminal's.
    fn effective(&self) -> Mode {
        match self.choice {
            ModeChoice::Auto => self.mode,
            ModeChoice::Dark => Mode::Dark,
            ModeChoice::Light => Mode::Light,
        }
    }

    fn palette(&self) -> &Palette {
        match self.effective() {
            Mode::Dark => &self.dark,
            Mode::Light => &self.light,
        }
    }

    /// The page background: the terminal's tinted, unless a forced mode differs from the terminal's.
    fn tint(&self) -> Hex {
        page_background(self.background.as_deref(), self.mode, self.effective(), self.palette().fallback_background)
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
            typing: self.finder.is_some()
                || self.search.as_ref().is_some_and(|search| search.editing)
                || self.command.is_some(),
        }
    }
}

/// The state a fresh app starts with: the browse root, the startup `pending` reveal, and the empty reader.
fn new_model(app: &App, root: PathBuf, pending: Option<Pending>) -> Model {
    Model {
        focus: Focus::List,
        mode: Mode::Dark,
        background: None,
        dark: DARK,
        light: LIGHT,
        choice: ModeChoice::Auto,
        parser: Parser::default(),
        proxy: app.proxy(),
        pictures: Default::default(),
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
        matching: fmc::settings::FIND_MODE,
        sidebar_jump: fmc::settings::SIDEBAR_JUMP,
        reader_jump: fmc::settings::READER_JUMP,
        git_gutter: fmc::settings::GIT_GUTTER,
        in_repo: false,
        branch: None,
        serial: 0,
        command: None,
        columns: app.size().0,
        rows: app.size().1,
        pages: Default::default(),
        overlay: None,
        overlay_at: (0, 0),
        overlay_task: None,
        settings: Default::default(),
        saves: Default::default(),
        clipboard: None,
        bulk: None,
        associations: Default::default(),
        opened: root,
    }
}

/// Runs the app on `target` until the user quits.
pub fn run(
    target: &Path,
    settings: Parsed,
    themes: crate::services::theme::Loaded,
    keys: fmc::keys::Parsed,
    settings_path: Option<PathBuf>,
    remembered: services::state::Loaded,
) -> opentui::Result<()> {
    // Scouter draws on the terminal's own background (`RGBA.defaultBackground()`), not a theme color.
    let [dark, light] = [Theme::dark(), Theme::light()].map(|t| Theme { background: Color::DEFAULT_BACKGROUND, ..t });
    let options = AppOptions {
        palette_size: theme::PALETTE_SIZE,
        tab_focus: false,
        // A drag selects and keeps the selection; `y` copies it, where Scouter copies on release.
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
    let model = Rc::new(RefCell::new(new_model(&app, root.clone(), pending)));
    {
        let mut m = model.borrow_mut();
        m.settings.values = settings.settings.clone();
        m.settings.themes = themes.themes.clone();
        m.settings.path = settings_path;
        m.parser = Parser::new(keys.keymap);
        m.auto_preview = settings.settings.auto_preview;
        m.sidebar_jump = settings.settings.sidebar_jump;
        m.reader_jump = settings.settings.reader_jump;
        m.git_gutter = settings.settings.git_gutter;
        m.associations = settings.settings.file_associations;
        m.matching = settings.settings.find_mode;
        remember::start(&mut m, remembered);
        m.choice = settings.settings.theme_mode;
        let Chosen { dark, light, notices } =
            catalog::choose(themes.themes, &settings.settings.dark_theme, &settings.settings.light_theme);
        (m.dark, m.light) = (dark, light);
        let notices: Vec<String> =
            settings.notices.into_iter().chain(themes.notices).chain(keys.notices).chain(notices).collect();
        if !notices.is_empty() {
            m.message = Some(Message { text: notices.join(" "), error: true });
        }
    }
    let ui = Rc::new(RefCell::new(build(&app, model.borrow().palette())));
    #[cfg(debug_assertions)]
    overlay::debug_open(&mut model.borrow_mut(), &ui.borrow());
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
                settings::event(cx.app(), &mut m, &mut u, event);
                paint(cx.app(), &mut m, &mut u);
            }
            // A failed launch stays in the footer, so the reason can be read; a launch that worked clears like a copy does.
            if matches!(event, Event::Launched { result: Ok(()), .. }) {
                clear_message_later(cx.app(), &model, &ui);
            }
            repaint_after_layout(cx.app(), &model, &ui);
            poll::schedule(cx.app(), &model, &ui);
        }
    });
    capture::wire(&app, &model, &ui);
    overlay::wire_views(&model, &ui);
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
        // A click on a file's header folds the file, on its release (`Click`); the rows are the last frame's.
        let (model, ui) = (model.clone(), ui.clone());
        let stacked = ui.borrow().diff.scroller.widget();
        let mut click = mouse::Click::default();
        stacked.on_mouse(move |cx, event| {
            if !click.feed(event.kind, event.x, event.y) {
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
    let result = app.run();
    remember::flush(&model.borrow());
    result
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
        // A run's status stays until its answer, whatever timer was set before it.
        if model.borrow().bulk.is_none() {
            model.borrow_mut().message = None;
        }
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
    let markdown_signs = SignStrip::new(app);
    let viewer = Viewer::new(app);
    let picture = Picture::new(app);
    let binary = BinaryView::new(app);
    let hex = HexView::new(app);
    let preview = CodeView::new(app, p);
    let preview_viewer = Viewer::new(app);
    preview_viewer.visible(false);
    preview_viewer.widget().add(&preview.widget());
    preview_viewer.gutter().add(&preview.gutter());
    let diff = DiffBody::new(app);
    let footer = Footer::new(app);
    browser_panel.add(&browser_header.root).add(&finder_bar.root).add(&tree.root).add(&changes_list.root);
    let stack = View::new(app).grow(1.0).min_width(0).min_height(0).column().overflow(Overflow::Hidden);
    let bar = ScrollBar::new(app, 2);
    let narrow_bar = ScrollBar::new(app, 0);
    let body = View::new(app).grow(1.0).min_height(0).row().overflow(Overflow::Hidden);
    viewer.widget().add(&reader.widget()).add(&markdown.widget()).add(&picture.widget());
    viewer.gutter().add(&reader.gutter());
    let settings = SettingsView::new(app);
    stack.add(&welcome.root).add(&binary.root()).add(&hex.widget()).add(&viewer.view()).add(&preview_viewer.view());
    stack.add(&diff.root);
    stack.add(&settings.root);
    viewer.view().add(&markdown_signs.view());
    body.add(&stack).add(&narrow_bar.root()).add(&bar.root());
    viewer_panel.add(&viewer_header.root).add(&body);
    panes.add(&browser_panel).add(&viewer_panel).add(&handle).add(&mark.root);
    layout.add(&panes).add(&footer.root);
    app.root().add(&layout);
    let overlays = overlay::build_views(app);
    overlay::mount(app, &overlays);
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
        markdown_signs,
        picture,
        preview,
        preview_viewer,
        preview_at: None,
        binary,
        hex,
        bar,
        narrow_bar,
        footer,
        overlay: overlays,
        settings,
    }
}

/// Rebuilds the parts that bake colors in (welcome), then repaints.
fn restyle(app: &App, m: &mut Model, ui: &mut Ui) {
    ui.welcome.root.detach();
    ui.welcome = welcome::welcome(app, m.palette());
    ui.stack.add(&ui.welcome.root);
    ui.reader.restyle(m.palette());
    ui.markdown.restyle(m.palette());
    paint(app, m, ui);
}

fn paint(app: &App, m: &mut Model, ui: &mut Ui) {
    // A copy: the palette borrows the model, which the frame writes to.
    let palette = m.palette().clone();
    let p = &palette;
    let (columns, rows) = app.size();
    m.columns = columns;
    m.rows = rows;
    // Every change passes a paint, so the remembered values are checked here (`app/remember.rs`).
    remember::sync(app, m);
    let chrome = if m.compact() { COMPACT_CHROME_ROWS } else { LIST_CHROME_ROWS };
    let viewport = usize::from(rows).saturating_sub(chrome);
    m.browser.set_height(viewport);
    m.change_list.set_height(viewport);
    let measure = |text: &str| {
        app.session().context().text_width(text, app.width_method()).map_or(text.chars().count(), |w| w as usize)
    };
    let tint = m.tint();

    layout::sync(m, columns);
    let width = layout::lay_out(ui, m, columns, p);
    ui.welcome.fit(columns, width, p);
    let focused = |pane: Focus| color(if m.focus == pane { p.accent } else { p.border });
    ui.browser_panel.border_color(focused(Focus::List));
    ui.viewer_panel.border_color(focused(Focus::Reader));

    // The changes screen replaces the tree in the sidebar and the reader in the viewer, unless a key page is open.
    let on_changes = m.changes.is_some();
    // The page fills the content area, so the changes body and the viewer's documents hide while it is open.
    let on_settings = m.settings.page.is_some();
    let body = changes::body_shows(m) && !on_settings;
    let root = m.browser.tree.root().to_path_buf();
    let name = root.file_name().map_or(String::new(), |n| fmc::text::display_label(&n.to_string_lossy()));
    let mut sidebar = if on_changes {
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
    sidebar.lead = pages::back_lead(m).or_else(|| layout::list_chevron(m, columns));
    ui.browser_header.root.visible(m.finder.is_none());
    ui.browser_header.paint(&sidebar, p);
    let bar = m.finder.as_ref().map(|finder| {
        let prompt = PromptLine::new(find::prompt(finder).0, &finder.query, &finder.status);
        prompt.segments(FindBar::room(usize::from(width)), p, &measure)
    });
    ui.find.paint(bar, p);
    ui.tree.root.visible(!on_changes);
    ui.changes.root.visible(on_changes);
    if on_changes {
        ui.changes.paint(app, &m.change_list, p);
    } else {
        let cut = m.clipboard.as_ref().filter(|clip| clip.cut).map(|clip| clip.paths.as_slice());
        ui.tree.paint(app, &m.browser, &m.marks, p, &root, cut);
    }
    // A preview names its file, with a `preview` pill in place of the reader's badges (Scouter `viewerHeaderFor`).
    let previewed =
        m.finder.as_ref().and_then(|finder| finder.preview.as_ref()).and_then(|preview| preview.shown.as_ref());
    let diff_header = body.then(|| changes::viewer_header(m));
    let (title, parent) = match (previewed, &diff_header, &m.shown) {
        _ if on_settings => ("Settings".into(), None),
        (Some(shown), _, _) => reader::title_parts(&root, &shown.path),
        (None, Some((title, parent, _)), _) => (title.clone(), parent.clone()),
        (None, None, reader::Shown::None) => ("Reader".into(), None),
        (None, None, reader::Shown::Keys { .. }) => ("Keys".into(), None),
        (
            None,
            None,
            reader::Shown::Text { path, .. } | reader::Shown::Image { path, .. } | reader::Shown::Binary { path, .. },
        ) => reader::title_parts(&root, path),
        (None, None, reader::Shown::Hex(hex)) => reader::title_parts(&root, &hex.path),
    };
    let badges = match (previewed, diff_header) {
        _ if on_settings => Vec::new(),
        (Some(_), _) => vec![Badge { text: "preview".into(), kind: BadgeKind::Pill }],
        (None, Some((_, _, badges))) => badges,
        (None, None) => reader::badges(&m.shown, m.wrap),
    };
    let viewer = HeaderState {
        title,
        parent,
        badges,
        focused: m.focus == Focus::Reader,
        notice: m.notice.clone(),
        lead: pages::back_lead(m).or_else(|| layout::reader_chevron(m, columns)),
    };
    ui.viewer_header.paint(&viewer, p);
    // The preview takes the reader's place, and hides the welcome (Scouter `previewHit`).
    let shows_preview = previewed.is_some();
    // The preview's line count, copied out: the model is changed below, which the borrow of `previewed` forbids.
    let preview_lines = previewed.map(|shown| shown.document.lines);
    // The compact tier has no welcome screen: its list is the first page.
    ui.welcome.root.visible(!m.compact() && !m.shown.is_some() && !shows_preview && !on_changes && !on_settings);
    let previewing = m.shown.previewing();
    ui.reader.visible(m.shown.text().is_some() && !previewing && !shows_preview && !body && !on_settings);
    ui.markdown.visible(previewing && !shows_preview && !body && !on_settings);
    ui.picture.visible(matches!(m.shown, reader::Shown::Image { .. }) && !shows_preview && !body && !on_settings);
    ui.binary.visible(matches!(m.shown, reader::Shown::Binary { .. }) && !shows_preview && !body && !on_settings);
    let hexing = matches!(m.shown, reader::Shown::Hex(_));
    ui.hex.visible(hexing && !shows_preview && !body && !on_settings);
    ui.settings.root.visible(on_settings);
    if let Some(page) = &m.settings.page {
        ui.settings.paint(&m.settings.values, page.selected, p);
    }
    // The reader's pane takes no space beside the preview, which is the pane the stack shows then. With no document it
    // takes none either: an empty viewer would halve the stack and push the welcome out of its half.
    ui.viewer.visible(m.shown.is_some() && !hexing && !shows_preview && !body && !on_settings);
    ui.diff.root.visible(body);
    if on_changes {
        ui.diff.set_split(m.diff_options.split);
        // A body at its end stays at the end when a rebuild changes its rows (Scouter's stack does the same).
        let (at_bottom, built) = (ui.diff.at_bottom(viewport), ui.diff.built_key());
        changes::sync(app, m, ui, p, tint);
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
    let root = tildify(&fmc::text::display_label(&m.browser.tree.root().to_string_lossy()), home.as_deref());
    let pane = if m.focus == Focus::List { Pane::Files } else { Pane::Reader };
    // The position is the one the last frame laid out; a move made since then lands in the next frame.
    m.viewport = viewport;
    m.top = if body { ui.diff.scroller.position().1 } else { ui.viewer.position().1 };
    // The outside gutters follow the scroll rows, so their numbers stay level with the text (`ui/reader.rs`).
    ui.reader.follow(ui.viewer.position().1);
    ui.preview.follow(ui.preview_viewer.position().1);
    // The rendered view's git signs sit beside its blocks at the scroll. Nothing shows outside it.
    let (signs, strip) = gutter::drawn(m);
    let marks = match m.shown.text() {
        Some((document, ..)) if strip && m.shown.previewing() && !signs.is_empty() => {
            block_marks(&ui.markdown.block_lines(&document.text), signs)
        }
        _ => Vec::new(),
    };
    ui.markdown_signs.show(app, p, &marks, &ui.markdown.block_tops(), ui.viewer.position().1);
    find::paint_preview(m, ui, p);
    // A preview is unwrapped, as Scouter's is; it takes the same width as the reader. The stacked diff's bar overlays
    // its last column, as Scouter's diff scroller does (no blank before the thumb).
    let fill = !body && !shows_preview && (m.wrap || m.shown.text().is_none() || previewing);
    ui.viewer.set_wrap(fill);
    ui.reader.set_wrap(m.wrap);
    {
        let (signs, strip) = gutter::drawn(m);
        ui.reader.set_signs(p, signs, strip);
    }
    ui.preview.set_wrap(false);
    // Unwrapped text takes the pane's inner width, the bar's column included: the pan range is that width less the
    // viewport, and it stops where Scouter's does.
    ui.reader.set_width(u32::from(columns.saturating_sub(width + 2)));
    ui.preview.set_width(u32::from(columns.saturating_sub(width + 2)));
    ui.picture.fill(viewport as u32);
    let lines = m.shown.text().map_or(0, |(document, ..)| document.lines);
    // The hex view draws its own rows, beside the narrow bar.
    let hex_rows = hex::sync(m, ui, usize::from(columns.saturating_sub(width + 3)), viewport, p);
    let total = match preview_lines {
        _ if body => ui.diff.height(),
        Some(lines) => lines,
        None if hex_rows.is_some() => hex_rows.map_or(0, |(rows, _)| rows),
        None => reader::content_rows(&m.shown, ui.markdown.rows(), ui.reader.rows(lines)),
    };
    // A preview has its own position, as Scouter's preview scroll box does.
    let top = match preview_lines {
        Some(_) => ui.preview_viewer.position().1.floor() as usize,
        None => hex_rows.map_or(m.top.floor() as usize, |(_, top)| top),
    };
    // The rendered view's bar takes one column, as Scouter's scroll box does: the blank before the thumb is wrapped
    // text's and a picture's, not the rendered view's (`scouter:src/app/reader-pane.ts:31-48`).
    let gapped = fill && !previewing && !hexing;
    ui.bar.paint(app, total, viewport, top, gapped, p);
    ui.narrow_bar.paint(app, total, viewport, top, !gapped, p);
    let has_blocks = m.changes.as_ref().is_some_and(|changes| !changes.files.is_empty());
    let (document, position) = if on_settings {
        (None, None)
    } else if body {
        (None, has_blocks.then(|| scroll_label(ui.diff.height(), viewport, m.top)))
    } else {
        match preview_lines {
            Some(_) => (None, None),
            None => reader::footer_facts(
                &m.shown,
                viewport,
                ui.markdown.rows(),
                hex_rows.map_or(ui.reader.rows(lines), |(rows, _)| rows),
                hex_rows.map_or(m.top, |(_, top)| top as f64),
            )
            .map_or((None, None), |(info, label)| (Some(info), label)),
        }
    };
    // The rendered view's marks follow the search; the code view's marks are for a code view that shows.
    search::sync(m, ui);
    let search_text = m.shown.text().map_or_else(String::new, |(document, ..)| document.content().to_owned());
    let code_search = m.search.as_ref().filter(|_| !m.shown.previewing());
    ui.reader.sync_search(
        p,
        code_search.map_or("", |search| search.query.text()),
        code_search.and_then(|search| search.current_line()),
        &search_text,
    );
    // An open prompt replaces the footer's left side, or the compact bottom bar's segments after `Cancel` (`ui/bars.rs`).
    let editing = match (&m.finder, &m.search, &m.command) {
        (Some(finder), _, _) => Some(PromptLine::new(find::prompt(finder).0, &finder.query, &finder.status)),
        (None, Some(search), _) if search.editing => Some(PromptLine::new("/", &search.query, &search.count_text())),
        (None, _, Some(line)) => Some(PromptLine::new(":", line, "")),
        _ => None,
    };
    let footer = FooterState {
        root,
        branch: m.branch.clone(),
        mode: if on_settings {
            hints::Mode::Settings
        } else if body {
            hints::Mode::Diff
        } else {
            hints::Mode::Browse
        },
        pane,
        prompt: match (&m.finder, &m.search) {
            (Some(finder), _) => Some(find::prompt(finder).1),
            (None, Some(search)) if search.editing => Some(Prompt::Reader),
            _ if m.command.is_some() => Some(Prompt::Command),
            _ => None,
        },
        message: m.message.clone(),
        document,
        diff: if body { changes::footer(m, ui) } else { None },
        positions: if body { [None, position] } else { [position, None] },
        wrap: m.wrap,
        pending: m.parser.pending_count(),
        selection: app.selected_text().is_some(),
        sidebar_hidden: !layout::shown(m, columns),
        page: m.pages.top(),
        editing,
        // The marks are the tree's: the finder and the changes screen show none.
        marked: if m.finder.is_none() && m.changes.is_none() { m.browser.marked().len() } else { 0 },
    };
    ui.footer.set_background(tint);
    ui.footer.paint(&footer, usize::from(columns), p, &measure);
    overlay::draw(app, m, ui);
}

#[cfg(test)]
#[path = "focus_tests.rs"]
mod focus_tests;

#[cfg(test)]
#[path = "layout_tests.rs"]
mod layout_tests;
#[cfg(test)]
mod mouse_tests;

#[cfg(test)]
mod associations_tests;

#[cfg(test)]
#[path = "mermaid_tests.rs"]
mod mermaid_tests;

#[cfg(test)]
#[path = "search_tests.rs"]
mod search_tests;

#[cfg(test)]
#[path = "find_tests.rs"]
mod find_tests;

#[cfg(test)]
#[path = "prompt_tests.rs"]
mod prompt_tests;

#[cfg(test)]
#[path = "rendered_signs_tests.rs"]
mod rendered_signs_tests;

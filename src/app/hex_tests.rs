//! The hex view through the model and the key path: `X` from the notice, the blocks a worker reads, the scroll keys,
//! the `:` offset jump, a click on the notice's button, and a late block of another file.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use opentui::testing::TestApp;
use tempfile::TempDir;

use super::super::*;
use super::block;
use crate::app::reader::Shown;

struct Viewing {
    app: TestApp,
    model: Rc<RefCell<Model>>,
    ui: Rc<RefCell<Ui>>,
    root: PathBuf,
    _dir: TempDir,
}

/// A folder holding `Pensieve.bin` (`bytes`), browsed on a 120 by 40 screen with its worker events wired, and the
/// file opened: the binary notice shows.
fn notice(bytes: &[u8]) -> Viewing {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    fs::write(root.join("Pensieve.bin"), bytes).unwrap();
    let app = TestApp::new(120, 40);
    let model = Rc::new(RefCell::new(new_model(app.app(), root.clone(), None)));
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette(), true)));
    wire_mouse(&model, &ui);
    capture::wire(app.app(), &model, &ui);
    {
        let (model, ui) = (model.clone(), ui.clone());
        app.app().on_message(move |cx, event: &Event| {
            tree::apply(&mut model.borrow_mut(), &ui.borrow(), event);
            paint(cx.app(), &mut model.borrow_mut(), &mut ui.borrow_mut());
        });
    }
    let entries = crate::services::fs::list(&root, &[]).unwrap();
    {
        let mut m = model.borrow_mut();
        m.browser.listed(&root, entries);
        m.browser.select(&root.join("Pensieve.bin"));
    }
    let v = Viewing { app, model, ui, root, _dir: dir };
    v.app.press("return");
    let shown = v.app.wait_until(Duration::from_secs(5), || matches!(v.model.borrow().shown, Shown::Binary { .. }));
    assert!(shown, "the binary notice is not shown");
    v.paint();
    v
}

impl Viewing {
    fn paint(&self) {
        paint(self.app.app(), &mut self.model.borrow_mut(), &mut self.ui.borrow_mut());
    }

    fn screen(&self) -> String {
        self.paint();
        let capture = self.app.render();
        (0..capture.height).map(|y| capture.row(y)).collect::<Vec<_>>().join("\n")
    }

    /// Waits until the screen shows `text`: the blocks come back from workers.
    fn await_screen(&self, text: &str) {
        let shown = self.app.wait_until(Duration::from_secs(5), || self.screen().contains(text));
        assert!(shown, "the screen lacks {text:?}:\n{}", self.screen());
    }

    fn hex_top(&self) -> u64 {
        match &self.model.borrow().shown {
            Shown::Hex(hex) => hex.top,
            _ => panic!("no hex view"),
        }
    }
}

/// A file of `size` bytes: a NUL first, so it is binary, then the low byte of each offset.
fn counting(size: usize) -> Vec<u8> {
    (0..size).map(|at| if at == 0 { 0 } else { at as u8 }).collect()
}

#[test]
fn x_on_the_notice_opens_the_hex_view_with_the_reader_focused() {
    let v = notice(b"\0Hogwarts\n");
    assert!(v.screen().contains("X hex view"), "the notice lacks its X button");
    v.app.type_text("X");
    assert!(v.model.borrow().shown.is_hex());
    assert_eq!(v.model.borrow().focus, Focus::Reader);
    v.await_screen("00000000  00 48 6f 67 77 61 72 74  73 0a");
    let screen = v.screen();
    assert!(screen.contains(".Hogwarts."), "{screen}");
    assert!(screen.contains("hex"), "the header names the view");
}

#[test]
fn the_scroll_keys_and_g_move_by_rows_and_reach_the_end() {
    let v = notice(&counting(300 * 1024));
    v.app.type_text("X");
    v.await_screen("00000010  10 11 12");
    v.app.type_text("j");
    assert_eq!(v.hex_top(), 16);
    v.app.type_text("G");
    v.paint();
    let viewport = v.model.borrow().viewport as u64;
    assert_eq!(v.hex_top(), (300 * 1024 / 16 - viewport) * 16);
    v.await_screen("0004aff0  f0 f1 f2");
    v.app.type_text("g");
    assert_eq!(v.hex_top(), 0);
    v.app.press("space");
    assert!(v.hex_top() > 16, "space pages down");
}

#[test]
fn a_colon_offset_jumps_and_a_bad_one_says_so() {
    let v = notice(&counting(300 * 1024));
    v.app.type_text("X");
    v.app.type_text(":0x20010");
    v.app.press("return");
    assert_eq!(v.hex_top(), 0x20010);
    v.await_screen("00020010  10 11 12");
    v.app.type_text(":Gringotts");
    v.app.press("return");
    let message = v.model.borrow().message.clone().map(|message| message.text);
    assert_eq!(message.as_deref(), Some("Not an offset: Gringotts"));
    assert_eq!(v.hex_top(), 0x20010, "a bad offset keeps the place");
}

#[test]
fn a_click_on_the_hex_button_opens_the_view() {
    let v = notice(b"\0Gringotts");
    let screen = v.screen();
    let (y, row) = screen.lines().enumerate().find(|(_, row)| row.contains("X hex view")).expect("the button shows");
    let chars: Vec<char> = row.chars().collect();
    let column = chars.windows(2).position(|pair| pair == ['X', ' ']).unwrap();
    v.app.mouse(opentui::input::MouseEventKind::Down(opentui::input::MouseButton::Left), column as u16, y as u16);
    assert!(v.model.borrow().shown.is_hex(), "the click opened the hex view");
}

#[test]
fn a_block_of_another_file_is_dropped() {
    let v = notice(b"\0Azkaban");
    v.app.type_text("X");
    v.await_screen("00 41 7a");
    let other = v.root.join("Nurmengard.bin");
    block(&mut v.model.borrow_mut(), &other, 0, &Ok(b"Grindelwald".to_vec()));
    assert!(!v.screen().contains("47 72 69"), "a block of another file must not show");
}

#[test]
fn a_short_block_moves_the_end_of_a_file_that_shrank() {
    let v = notice(&counting(200 * 1024));
    v.app.type_text("X");
    let path = v.root.join("Pensieve.bin");
    block(&mut v.model.borrow_mut(), &path, 1, &Ok(vec![7; 100]));
    match &v.model.borrow().shown {
        Shown::Hex(hex) => assert_eq!(hex.size, fmc::reader::hex::BLOCK + 100),
        _ => panic!("no hex view"),
    }
}

impl Viewing {
    fn message(&self) -> String {
        self.model.borrow().message.clone().map(|message| message.text).unwrap_or_default()
    }

    /// Waits until the search's answer puts `text` in the footer.
    fn await_message(&self, text: &str) {
        let shown = self.app.wait_until(Duration::from_secs(5), || self.message() == text);
        assert!(shown, "the footer shows {:?}, not {text:?}", self.message());
    }
}

/// A file with `Langdon` at 0x10 and `LANGDON` at 0x30000, past the blocks the first screen reads.
fn two_langdons() -> Vec<u8> {
    let mut bytes = counting(300 * 1024);
    bytes[0x10..0x17].copy_from_slice(b"Langdon");
    bytes[0x30000..0x30007].copy_from_slice(b"LANGDON");
    bytes
}

#[test]
fn a_search_finds_a_string_ignoring_case_steps_and_wraps() {
    let v = notice(&two_langdons());
    v.app.type_text("X");
    v.app.type_text("/langdon");
    v.app.press("return");
    v.await_message("/langdon  at 0x10");
    v.app.press("escape");
    assert!(v.model.borrow().search.as_ref().is_some_and(|search| !search.editing), "Esc keeps the search");
    v.app.type_text("n");
    v.await_message("/langdon  at 0x30000");
    assert!(v.hex_top() <= 0x30000 && 0x30000 < v.hex_top() + 16 * 40, "the hit is in view");
    v.await_screen("4c 41 4e 47 44 4f 4e");
    v.app.type_text("n");
    v.await_message("/langdon  at 0x10");
    v.app.type_text("N");
    v.await_message("/langdon  at 0x30000");
}

#[test]
fn a_search_with_no_hit_says_so_and_esc_drops_the_mark() {
    let v = notice(&two_langdons());
    v.app.type_text("X");
    v.app.type_text("/vittoria");
    v.app.press("return");
    v.await_message("/vittoria  no matches");
    v.app.press("escape");
    v.app.press("escape");
    assert!(v.model.borrow().search.is_none(), "a second Esc drops the search");
}

#[test]
fn closing_the_view_stops_its_search() {
    let v = notice(&two_langdons());
    v.app.type_text("X");
    v.app.type_text("/langdon");
    v.app.press("return");
    let flag = match &v.model.borrow().shown {
        Shown::Hex(hex) => hex.stop.as_ref().map(|stop| stop.0.clone()),
        _ => None,
    };
    v.app.press("escape");
    v.app.type_text("q");
    assert!(!v.model.borrow().shown.is_hex());
    if let Some(flag) = flag {
        assert!(flag.load(std::sync::atomic::Ordering::Relaxed), "the closed view's search is stopped");
    }
}

#[test]
fn t_reads_the_file_as_text_and_x_and_t_switch_views() {
    let v = notice(b"\0Robert\x1bLangdon\n\xe9\n");
    assert!(v.screen().contains("t as text"), "the notice lacks its t button");
    v.app.type_text("t");
    let lossy = || matches!(v.model.borrow().shown, Shown::Text { lossy: Some(_), .. });
    assert!(v.app.wait_until(Duration::from_secs(5), lossy), "t shows the text");
    assert_eq!(v.model.borrow().focus, Focus::Reader);
    let screen = v.screen();
    assert!(screen.contains("\u{2400}Robert\u{241b}Langdon"), "{screen}");
    assert!(screen.contains("lossy"), "the header says the text is lossy");
    v.app.type_text("X");
    assert!(v.model.borrow().shown.is_hex(), "X goes from the text to the hex view");
    v.app.type_text("t");
    assert!(v.app.wait_until(Duration::from_secs(5), lossy), "t goes from the hex view to the text");
}

#[test]
fn a_file_that_is_not_utf8_shows_the_notice_with_its_reason() {
    let v = notice(b"Sophie Neveu, caf\xe9 du Louvre\n");
    let screen = v.screen();
    assert!(screen.contains("Only UTF-8 text files are supported."), "{screen}");
    assert!(screen.contains("X hex view"), "{screen}");
}

#[test]
fn a_control_picture_is_one_cell_wide() {
    let v = notice(b"\0");
    let app = v.app.app();
    for picture in ["\u{2400}", "\u{241b}", "\u{2421}"] {
        assert_eq!(app.session().context().text_width(picture, app.width_method()).unwrap(), 1, "{picture}");
    }
}

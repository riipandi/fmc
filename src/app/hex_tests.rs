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
    let ui = Rc::new(RefCell::new(build(app.app(), model.borrow().palette())));
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

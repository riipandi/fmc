use super::*;

/// A key with the modifiers an Option, Cmd, or Ctrl chord sends; `text` is empty, as the field edits ignore it.
fn modified(name: &str, ctrl: bool, alt: bool, cmd: bool) -> Key<'_> {
    Key { name, text: "", ctrl, alt, shift: false, cmd }
}

fn key<'a>(name: &'a str, text: &'a str) -> Key<'a> {
    Key { name, text, ctrl: false, alt: false, shift: false, cmd: false }
}

fn ctrl(name: &str) -> Key<'_> {
    Key { name, text: "", ctrl: true, alt: false, shift: false, cmd: false }
}

fn row(label: &str, enabled: bool) -> Item {
    Item { label: label.into(), enabled, ..Item::default() }
}

/// The Harry Potter menu the app shows: `Paste` is disabled until a path is copied.
fn menu() -> Menu {
    Menu::new(vec![
        row("New file…", true),
        row("New folder…", true),
        row("Cut", true),
        row("Paste", false),
        row("Delete…", true),
    ])
}

#[test]
fn a_menu_selects_its_first_enabled_row() {
    let disabled_first = Menu::new(vec![row("Paste", false), row("Cut", true)]);
    assert_eq!(disabled_first.selected(), 1);
    assert_eq!(menu().selected(), 0);
}

#[test]
fn down_and_up_skip_a_disabled_row() {
    let mut menu = menu();
    assert_eq!(menu.key(key("down", "")), Outcome::Stay);
    assert_eq!(menu.key(key("j", "j")), Outcome::Stay);
    assert_eq!(menu.selected(), 2, "Cut is next after New folder");
    menu.key(key("down", ""));
    assert_eq!(menu.selected(), 4, "Paste is skipped");
    menu.key(key("down", ""));
    assert_eq!(menu.selected(), 4, "the last row holds the selection");
    menu.key(key("k", "k"));
    assert_eq!(menu.selected(), 2, "k moves up over the disabled row too");
}

#[test]
fn return_picks_the_selected_row_and_a_disabled_row_is_not_picked() {
    let mut menu = menu();
    assert_eq!(menu.key(key("return", "\r")), Outcome::Pick(0));
    assert_eq!(menu.pick(3), Outcome::Stay, "a click on a disabled row does nothing");
    assert_eq!(menu.pick(9), Outcome::Stay, "a click past the rows does nothing");
}

#[test]
fn escape_closes_a_menu_and_other_keys_are_swallowed() {
    let mut menu = menu();
    assert_eq!(menu.key(key("escape", "")), Outcome::Close);
    for other in [key("x", "x"), key("tab", ""), key("left", ""), key("space", " "), key("q", "q")] {
        assert_eq!(menu.key(other), Outcome::Stay);
    }
    assert_eq!(menu.key(ctrl("q")), Outcome::Stay, "a modified key is not a menu key");
    assert_eq!(menu.selected(), 0);
}

#[test]
fn a_dialog_without_a_field_starts_on_its_default_button() {
    let dialog = Dialog::new("Move to Trash?", "Hermione's essay goes to the Trash.", false, &["Cancel", "Delete"], 0);
    assert_eq!(dialog.focus, Stop::Button(0), "Cancel is focused for a destructive confirm");
}

#[test]
fn tab_cycles_the_field_and_the_buttons() {
    let mut dialog = Dialog::new("New file", "Name it.", true, &["Cancel", "OK"], 1);
    assert_eq!(dialog.focus, Stop::Field);
    dialog.key(key("tab", ""));
    assert_eq!(dialog.focus, Stop::Button(0));
    dialog.key(key("tab", ""));
    assert_eq!(dialog.focus, Stop::Button(1));
    dialog.key(key("tab", ""));
    assert_eq!(dialog.focus, Stop::Field);
}

#[test]
fn left_and_right_move_between_buttons_without_wrapping() {
    let mut dialog = Dialog::new("Move to Trash?", "Gone.", false, &["Cancel", "Delete"], 0);
    dialog.key(key("left", ""));
    assert_eq!(dialog.focus, Stop::Button(0));
    dialog.key(key("right", ""));
    assert_eq!(dialog.focus, Stop::Button(1));
    dialog.key(key("right", ""));
    assert_eq!(dialog.focus, Stop::Button(1));
}

#[test]
fn typing_goes_to_the_field_and_moves_the_focus_there() {
    let mut dialog = Dialog::new("New file", "Name it.", true, &["Cancel", "OK"], 1);
    dialog.key(key("tab", ""));
    assert_eq!(dialog.key(key("r", "r")), Outcome::Insert("r".into()));
    assert_eq!(dialog.focus, Stop::Field);
    assert_eq!(dialog.key(key("space", " ")), Outcome::Insert(" ".into()));
    assert_eq!(dialog.key(key("backspace", "")), Outcome::Edit(FieldEdit::Backspace));
    assert_eq!(dialog.key(ctrl("x")), Outcome::Stay, "a control key is not text");
}

#[test]
fn return_on_the_field_picks_the_default_and_on_a_button_picks_that_button() {
    let mut dialog = Dialog::new("New file", "Name it.", true, &["Cancel", "OK"], 1);
    assert_eq!(dialog.key(key("return", "\r")), Outcome::Pick(1));
    dialog.key(key("tab", ""));
    assert_eq!(dialog.key(key("return", "\r")), Outcome::Pick(0));
}

#[test]
fn a_dialog_swallows_typing_when_it_has_no_field_and_closes_on_escape() {
    let mut dialog = Dialog::new("Move to Trash?", "Gone.", false, &["Cancel", "Delete"], 0);
    assert_eq!(dialog.key(key("x", "x")), Outcome::Stay);
    assert_eq!(dialog.key(key("backspace", "")), Outcome::Stay);
    assert_eq!(dialog.key(key("escape", "")), Outcome::Close);
}

#[test]
fn the_overlay_hands_a_key_to_its_own_kind() {
    let mut overlay = Overlay::Menu(menu());
    assert_eq!(overlay.key(key("escape", "")), Outcome::Close);
    let mut overlay = Overlay::Dialog(Dialog::new("New file", "Name it.", true, &["Cancel", "OK"], 1));
    assert_eq!(overlay.key(key("a", "a")), Outcome::Insert("a".into()));
}

#[test]
fn a_pointer_over_an_enabled_row_selects_it_and_over_a_disabled_row_does_nothing() {
    let mut menu = menu();
    assert!(menu.point(2), "Cut is enabled, so the pointer selects it");
    assert_eq!(menu.selected(), 2);
    assert!(!menu.point(2), "a row already selected changes nothing");
    assert!(!menu.point(3), "Paste is disabled, so the selection stays");
    assert_eq!(menu.selected(), 2);
    assert!(!menu.point(9), "a row past the items is nothing");
}

#[test]
fn a_pointer_over_a_button_highlights_it_and_leaves_the_focus_alone() {
    let mut dialog = Dialog::new("Move to Trash?", "Gone.", false, &["Cancel", "Delete"], 0);
    assert!(dialog.hover(1, true));
    assert_eq!(dialog.hovered, Some(1));
    assert_eq!(dialog.focus, Stop::Button(0), "the focus stays on Cancel");
    assert_eq!(dialog.key(key("return", "")), Outcome::Pick(0), "Return still picks the focused button");
    assert!(!dialog.hover(1, true), "a button already highlighted changes nothing");
    assert!(dialog.hover(1, false));
    assert_eq!(dialog.hovered, None);
    assert!(!dialog.hover(0, false), "leaving a button the pointer is not on changes nothing");
    // Typing a name returns the focus to the field, so a pointer left on a button does not take the text.
    let mut named = Dialog::new("New file", "Name it.", true, &["Cancel", "OK"], 1);
    named.hover(0, true);
    assert_eq!(named.key(key("a", "a")), Outcome::Insert("a".into()));
    assert_eq!(named.focus, Stop::Field);
}

#[test]
fn each_option_cmd_and_ctrl_key_asks_for_its_field_edit() {
    let cases = [
        (modified("left", false, true, false), FieldEdit::WordBackward),
        (modified("b", false, true, false), FieldEdit::WordBackward),
        (modified("left", true, false, false), FieldEdit::WordBackward),
        (modified("right", false, true, false), FieldEdit::WordForward),
        (modified("f", false, true, false), FieldEdit::WordForward),
        (modified("right", true, false, false), FieldEdit::WordForward),
        (modified("backspace", false, true, false), FieldEdit::DeleteWordBackward),
        (modified("w", true, false, false), FieldEdit::DeleteWordBackward),
        (modified("backspace", true, false, false), FieldEdit::DeleteWordBackward),
        (modified("delete", false, true, false), FieldEdit::DeleteWordForward),
        (modified("d", false, true, false), FieldEdit::DeleteWordForward),
        (modified("delete", true, false, false), FieldEdit::DeleteWordForward),
        (modified("left", false, false, true), FieldEdit::LineHome),
        (modified("a", true, false, false), FieldEdit::LineHome),
        (modified("right", false, false, true), FieldEdit::LineEnd),
        (modified("e", true, false, false), FieldEdit::LineEnd),
        (modified("backspace", false, false, true), FieldEdit::DeleteToLineStart),
        (modified("u", true, false, false), FieldEdit::DeleteToLineStart),
        (modified("k", true, false, false), FieldEdit::DeleteToLineEnd),
        (modified("backspace", false, false, false), FieldEdit::Backspace),
    ];
    for (key, edit) in cases {
        let mut dialog = Dialog::new("New file", "Name it.", true, &["Cancel", "OK"], 1);
        assert_eq!(dialog.key(key), Outcome::Edit(edit), "{key:?}");
    }
}

#[test]
fn a_field_edit_takes_the_focus_to_the_field_from_a_button() {
    let mut dialog = Dialog::new("New file", "Name it.", true, &["Cancel", "OK"], 1);
    dialog.key(key("tab", ""));
    assert_eq!(dialog.focus, Stop::Button(0));
    dialog.key(modified("left", false, true, false));
    assert_eq!(dialog.focus, Stop::Field);
}

#[test]
fn other_option_cmd_and_ctrl_keys_do_nothing_in_the_field_and_a_confirm_has_no_field_edit() {
    let mut dialog = Dialog::new("New file", "Name it.", true, &["Cancel", "OK"], 1);
    for other in [modified("x", false, true, false), modified("up", false, false, true), ctrl("x")] {
        assert_eq!(dialog.key(other), Outcome::Stay, "{other:?}");
    }
    let mut confirm = Dialog::new("Move to Trash?", "Gone.", false, &["Cancel", "Delete"], 0);
    assert_eq!(confirm.key(modified("left", false, true, false)), Outcome::Stay);
    assert_eq!(confirm.key(modified("backspace", true, false, false)), Outcome::Stay);
    assert_eq!(confirm.focus, Stop::Button(0), "a confirm's focus does not move");
}

/// A scrolled menu shows its window and moves it with the selection, in both directions.
#[test]
fn a_scrolled_menu_moves_its_window_with_the_selection() {
    let items: Vec<Item> = (0..6).map(|n| row(&format!("Hogwarts {n}"), true)).collect();
    let mut menu = Menu::scrolled(items, 3);
    assert_eq!((menu.top(), menu.window().len()), (0, 3));
    for _ in 0..4 {
        menu.key(key("down", ""));
    }
    assert_eq!((menu.selected(), menu.top()), (4, 2), "the window follows the selection down");
    assert_eq!(menu.window()[2].label, "Hogwarts 4");
    for _ in 0..3 {
        menu.key(key("up", ""));
    }
    assert_eq!((menu.selected(), menu.top()), (1, 1), "the window follows the selection up");
}

/// The keys and the pointer name a row by its index in the whole menu: a scrolled window's row is `top` plus its place.
#[test]
fn a_scrolled_menu_picks_by_the_whole_menu_index() {
    let items: Vec<Item> = (0..6).map(|n| row(&format!("Hogwarts {n}"), true)).collect();
    let mut menu = Menu::scrolled(items, 3);
    for _ in 0..4 {
        menu.key(key("down", ""));
    }
    assert_eq!(menu.pick(menu.top() + 2), Outcome::Pick(4));
}

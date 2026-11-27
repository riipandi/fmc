//! The keymap file and the merged table (plan-00015, A8, A10, A11): the notices with their line, `null`, sequences, the
//! card and `--help` with a rebound key, and the default rows. `keys/tests.rs` pins the defaults; these pin the file.

use crate::keys::bindings::{ACTION_NAMES, Chord, DEFAULTS};
use crate::keys::keymap::parse;
use crate::keys::{
    Action, Focus, Key, Keymap, Parser, Screen, State, Step, key_page_markdown, key_page_markdown_with, key_sections,
};

const TREE: State =
    State { screen: Screen::Browse, focus: Focus::List, reader_ready: true, previewing: false, typing: false };
const READER: State = State { focus: Focus::Reader, ..TREE };

fn press(name: &str) -> Key<'_> {
    Key { name, text: name, ctrl: false, alt: false, shift: false, cmd: false }
}

fn ctrl(name: &str) -> Key<'_> {
    Key { ctrl: true, ..press(name) }
}

fn run(keymap: &Keymap, keys: &[Key<'_>], state: State) -> Vec<Step> {
    let mut parser = Parser::new(keymap.clone());
    keys.iter().map(|key| parser.feed(*key, state)).collect()
}

fn section<'a>(sections: &'a [(String, Vec<(String, String)>)], title: &str) -> &'a [(String, String)] {
    &sections.iter().find(|(name, _)| name == title).expect("the section exists").1
}

#[test]
fn an_unknown_action_skips_its_entry_and_the_notice_names_the_line() {
    let parsed = parse("{\n  \"reader.j\": \"scroll_sideways\",\n  \"reader.k\": \"scroll_up\"\n}");
    assert_eq!(parsed.notices.len(), 1, "{:?}", parsed.notices);
    assert!(parsed.notices[0].starts_with("keymap.json line 2:"), "{:?}", parsed.notices);
    assert!(parsed.notices[0].contains("scroll_sideways"));
    assert_eq!(run(&parsed.keymap, &[press("j")], READER), [Step::Done(Action::Scroll(1))], "the default j is kept");
}

#[test]
fn an_unknown_context_and_a_bad_key_name_each_skip_with_a_notice_naming_the_line() {
    let parsed = parse("{\n  \"nowhere.j\": \"top\",\n  \"reader.ctrl-zz\": \"top\"\n}");
    let notices = &parsed.notices;
    assert_eq!(notices.len(), 2, "{notices:?}");
    assert!(notices.iter().any(|n| n.starts_with("keymap.json line 2:") && n.contains("nowhere")), "{notices:?}");
    assert!(notices.iter().any(|n| n.starts_with("keymap.json line 3:") && n.contains("ctrl-zz")), "{notices:?}");
}

#[test]
fn a_key_with_no_context_is_skipped_with_its_line() {
    let parsed = parse("{\n\n  \"j\": \"top\"\n}");
    assert_eq!(parsed.notices.len(), 1, "{:?}", parsed.notices);
    assert!(parsed.notices[0].starts_with("keymap.json line 3:"), "{:?}", parsed.notices);
}

#[test]
fn null_unbinds_the_default_key_in_its_context_only() {
    let parsed = parse(r#"{"reader.j": null, "everywhere.q": null}"#);
    assert!(parsed.notices.is_empty(), "{:?}", parsed.notices);
    assert_eq!(run(&parsed.keymap, &[press("j")], READER), [Step::Bad], "the reader's j does nothing");
    assert_eq!(run(&parsed.keymap, &[press("j")], TREE), [Step::Done(Action::Move(1))], "the tree keeps its j");
    assert_eq!(run(&parsed.keymap, &[press("q")], TREE), [Step::Bad], "q is unbound everywhere");
}

#[test]
fn a_user_sequence_starting_with_g_makes_g_wait_in_the_more_state() {
    let parsed = parse(r#"{"browse.g x": "open_settings"}"#);
    let keymap = &parsed.keymap;
    assert!(parsed.notices.is_empty(), "{:?}", parsed.notices);
    assert_eq!(run(keymap, &[press("g")], TREE), [Step::More], "a longer binding makes g wait");
    assert_eq!(run(keymap, &[press("g"), press("x")], TREE), [Step::More, Step::Done(Action::OpenSettings)]);
    assert_eq!(
        run(keymap, &[press("g"), press("j")], TREE),
        [Step::More, Step::Done(Action::Move(1))],
        "a key that cannot follow cancels the wait and is read alone"
    );
    assert_eq!(
        run(keymap, &[press("g"), press("g")], TREE),
        [Step::More, Step::More],
        "the second g starts a new wait"
    );
    assert_eq!(run(keymap, &[press("g")], READER), [Step::Done(Action::Top)], "the reader's g is not the tree's");
}

#[test]
fn ctrl_comma_is_a_table_entry_that_a_user_can_rebind() {
    let comma = Chord::parse("ctrl-,").unwrap();
    assert!(
        Keymap::defaults().bindings().iter().any(|b| b.action == Action::OpenSettings && b.keys == [comma.clone()]),
        "Ctrl+, is a row of the table"
    );
    let parsed = parse(r#"{"everywhere.ctrl-,": null, "everywhere.ctrl-s": "open_settings"}"#);
    assert_eq!(run(&parsed.keymap, &[ctrl(",")], TREE), [Step::Bad]);
    assert_eq!(run(&parsed.keymap, &[ctrl("s")], READER), [Step::Done(Action::OpenSettings)]);
}

#[test]
fn the_card_shows_a_rebound_key_on_its_action_row() {
    let parsed = parse(r#"{"everywhere.ctrl-,": null, "everywhere.ctrl-s": "open_settings"}"#);
    let page = key_page_markdown_with(&parsed.keymap);
    assert!(page.contains("| the Settings page; each change saves to settings.json | `Ctrl+s` |"), "{page}");
    assert!(!page.contains("`Ctrl+,`"), "the old key is gone from the card");
}

#[test]
fn a_binding_of_an_action_with_no_row_gets_its_own_row_under_its_context() {
    let parsed = parse(r#"{"browse.z": "move_page_down"}"#);
    let sections = key_sections(&parsed.keymap);
    let browse = section(&sections, "Browse");
    assert!(browse.contains(&("z".to_string(), "move page down".to_string())), "{browse:?}");
    assert!(key_page_markdown_with(&parsed.keymap).contains("| move page down | `z` |"));
}

#[test]
fn the_default_card_is_the_default_keymap_card() {
    assert_eq!(key_page_markdown_with(&Keymap::defaults()), key_page_markdown());
}

#[test]
fn a_file_that_is_not_an_object_or_not_json_keeps_every_default() {
    let defaults = Keymap::defaults().bindings().len();
    let not_object = parse("[1, 2]");
    assert_eq!(not_object.notices.len(), 1, "{:?}", not_object.notices);
    assert!(not_object.notices[0].contains("JSON object"));
    assert_eq!(not_object.keymap.bindings().len(), defaults);

    let broken = parse("{\"reader.j\": ");
    assert_eq!(broken.notices.len(), 1, "{:?}", broken.notices);
    assert!(broken.notices[0].contains("not valid JSON (line 1)"), "{:?}", broken.notices);
    assert_eq!(broken.keymap.bindings().len(), defaults);
}

#[test]
fn an_empty_object_is_the_defaults_without_a_notice() {
    let parsed = parse("{}");
    assert!(parsed.notices.is_empty());
    assert_eq!(parsed.keymap.bindings(), Keymap::defaults().bindings());
}

#[test]
fn key_names_follow_the_file_rules() {
    assert_eq!(Chord::parse("G").unwrap(), Chord::parse("shift-g").unwrap(), "G is Shift+g");
    assert!(Chord::parse("?").is_ok());
    assert!(Chord::parse("enter").is_ok());
    assert!(Chord::parse("ctrl-alt-q").is_ok());
    assert!(Chord::parse("*-tab").is_ok());
    for bad in ["", "Enter", "ctrl-", "shift-?", "ctrl-zz", "f1"] {
        assert!(Chord::parse(bad).is_err(), "{bad:?} is not a key name");
    }
    let parsed = parse(r#"{"reader.g  g": "top"}"#);
    assert_eq!(parsed.notices.len(), 1, "a double space leaves an empty key: {:?}", parsed.notices);
}

#[test]
fn every_default_action_and_every_name_round_trip() {
    for (_, _, _, action) in DEFAULTS {
        let name = action.name().expect("a default action has a name");
        assert_eq!(Action::from_name(name), Some(*action), "{name}");
    }
    for (name, action) in ACTION_NAMES {
        assert_eq!(action.name(), Some(*name));
    }
    assert_eq!(Action::GoToLine(3).name(), None, "a count is not a key");
}

#[test]
fn the_card_lists_every_default_row_with_its_fixed_keys() {
    let sections = key_sections(&Keymap::defaults());
    assert_eq!(sections.last().map(|(title, _)| title.as_str()), Some("Mouse"));
    assert!(section(&sections, "Browse").contains(&("gg G".to_string(), "top / bottom".to_string())));
}

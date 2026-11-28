//! The line editor: each edit at both ends and in the middle, Unicode text, and the word boundaries. The word cases
//! are OpenTUI's own: `tests/edit-buffer_test.zig` (`word boundary and line end walks`) at the 0.9.9 tag, with the
//! CJK positions counted in characters rather than cells.

use super::*;

fn line(text: &str) -> Line {
    let mut line = Line::default();
    line.insert(text);
    line
}

/// The caret as a character count, which is how the word cases are written.
fn at(line: &Line) -> usize {
    line.before().chars().count()
}

/// The caret positions a word step visits from `start`, until it stops moving.
fn walk(text: &str, start: usize, step: fn(&mut Line, FieldEdit) -> bool, edit: FieldEdit) -> Vec<usize> {
    let mut line = line(text);
    line.edit(FieldEdit::LineHome);
    for _ in 0..start {
        line.right();
    }
    let mut seen = Vec::new();
    loop {
        let before = line.before().len();
        step(&mut line, edit);
        if line.before().len() == before {
            return seen;
        }
        seen.push(at(&line));
    }
}

fn forward(text: &str) -> Vec<usize> {
    walk(text, 0, Line::edit, FieldEdit::WordForward)
}

fn backward(text: &str) -> Vec<usize> {
    let mut line = line(text);
    let mut seen = Vec::new();
    loop {
        let before = line.before().len();
        line.edit(FieldEdit::WordBackward);
        if line.before().len() == before {
            return seen;
        }
        seen.push(at(&line));
    }
}

#[test]
fn text_is_inserted_at_the_caret_and_the_caret_follows_it() {
    let mut text = line("Hermione Granger");
    for _ in 0..7 {
        text.left();
    }
    text.insert("Jean ");
    assert_eq!(text.text(), "Hermione Jean Granger");
    assert_eq!(text.before(), "Hermione Jean ");
}

#[test]
fn the_caret_stops_at_both_ends() {
    let mut text = line("Luna");
    text.right();
    assert_eq!(text.after(), "");
    for _ in 0..10 {
        text.left();
    }
    assert_eq!(text.before(), "");
    text.left();
    assert_eq!(text.before(), "");
    assert!(!text.edit(FieldEdit::Backspace), "nothing before the caret");
    assert_eq!(text.text(), "Luna");
}

#[test]
fn arrows_move_by_one_character_never_into_a_utf8_sequence() {
    // The em dash is three bytes; the caret steps over it whole.
    let mut text = line("Hermione Granger — Gringotts");
    for _ in 0..11 {
        text.left();
    }
    assert_eq!(text.before(), "Hermione Granger ");
    text.right();
    assert_eq!(text.before(), "Hermione Granger —");
    text.left();
    assert_eq!(text.before(), "Hermione Granger ");
    text.left();
    assert_eq!(text.before(), "Hermione Granger");
}

#[test]
fn backspace_deletes_the_character_before_the_caret() {
    let mut text = line("Gringotts");
    text.left();
    assert!(text.edit(FieldEdit::Backspace));
    assert_eq!((text.text(), text.before()), ("Gringots", "Gringot"));
    let mut text = line("Hermione Granger — Gringotts");
    for _ in 0..10 {
        text.left();
    }
    assert!(text.edit(FieldEdit::Backspace), "the dash is one character of three bytes");
    assert_eq!((text.text(), text.before()), ("Hermione Granger  Gringotts", "Hermione Granger "));
}

#[test]
fn word_steps_follow_opentui_for_ascii_and_separators() {
    assert_eq!(forward("Hello World"), vec![6, 11]);
    assert_eq!(backward("Hello World"), vec![6, 0]);
    assert_eq!(forward("self-contained"), vec![5, 14]);
    assert_eq!(backward("self-contained"), vec![5, 0]);
    assert_eq!(forward("The quick brown fox"), vec![4, 10, 16, 19]);
    assert_eq!(backward("The quick brown fox"), vec![16, 10, 4, 0]);
    assert_eq!(forward("Hello\tWorld"), vec![6, 11]);
    assert_eq!(backward("Hello\tWorld"), vec![6, 0]);
}

#[test]
fn word_steps_stop_at_each_separator_and_not_at_an_em_dash() {
    // The spaces at 8, 16, and 18 end words; the dash at 17 is a word of its own, as OpenTUI treats it.
    let text = "Hermione Granger — Gringotts";
    assert_eq!(forward(text), vec![9, 17, 19, 28]);
    assert_eq!(backward(text), vec![19, 17, 9, 0]);
}

#[test]
fn word_steps_move_by_character_counts_across_scripts() {
    // A script change is a boundary: 日本語 is one word, and so is abc, with a boundary between them.
    assert_eq!(forward("日本語abc"), vec![3, 6]);
    assert_eq!(backward("日本語abc"), vec![3, 0]);
    // A CJK run of five is one word.
    assert_eq!(forward("日本語文字"), vec![5]);
    assert_eq!(backward("日本語文字"), vec![0]);
    // The ideographic full stop is a separator.
    assert_eq!(forward("日本語。abc"), vec![4, 7]);
    assert_eq!(backward("日本語。abc"), vec![4, 0]);
    assert_eq!(forward("丽abc"), vec![1, 4]);
    assert_eq!(forward("a日"), vec![1, 2]);
    assert_eq!(forward("日a"), vec![1, 2]);
    assert_eq!(forward("테스트"), vec![3]);
    assert_eq!(forward("테스트test"), vec![3, 7]);
    assert_eq!(backward("테스트test"), vec![3, 0]);
    // An emoji is one character and a word of its own; the space after it is the boundary.
    assert_eq!(forward("🌟 ok"), vec![2, 4]);
    assert_eq!(backward("🌟 ok"), vec![2, 0]);
}

#[test]
fn a_caret_on_a_space_steps_past_the_next_word() {
    // The caret at 8 sits on the space; the boundary at the caret does not count, so the step goes to the end.
    assert_eq!(walk("Hermione Granger", 8, Line::edit, FieldEdit::WordForward), vec![16]);
    // The caret at 9 is at the start of Granger; a step back goes to the start of the line, not onto the space.
    let mut text = line("Hermione Granger");
    for _ in 0..7 {
        text.left();
    }
    text.edit(FieldEdit::WordBackward);
    assert_eq!(at(&text), 0);
}

#[test]
fn option_delete_removes_the_word_before_and_after_the_caret() {
    let mut text = line("sophie neveu");
    assert!(text.edit(FieldEdit::DeleteWordBackward));
    assert_eq!((text.text(), at(&text)), ("sophie ", 7));
    assert!(text.edit(FieldEdit::DeleteWordBackward));
    assert_eq!((text.text(), at(&text)), ("", 0));

    let mut text = line("sophie neveu");
    text.edit(FieldEdit::LineHome);
    assert!(text.edit(FieldEdit::DeleteWordForward));
    assert_eq!((text.text(), at(&text)), ("neveu", 0));
    assert!(text.edit(FieldEdit::DeleteWordForward));
    assert_eq!(text.text(), "");
    assert!(!text.edit(FieldEdit::DeleteWordForward), "nothing after the caret");
}

#[test]
fn the_line_edits_cut_to_the_start_or_the_end_at_the_caret() {
    let mut text = line("Hermione Granger");
    for _ in 0..8 {
        text.left();
    }
    assert!(text.edit(FieldEdit::DeleteToLineEnd));
    assert_eq!((text.text(), at(&text)), ("Hermione", 8));
    assert!(text.edit(FieldEdit::DeleteToLineStart));
    assert_eq!((text.text(), at(&text)), ("", 0));

    let mut text = line("Hermione Granger");
    text.edit(FieldEdit::LineEnd);
    assert_eq!(at(&text), 16);
    text.edit(FieldEdit::LineHome);
    assert_eq!(at(&text), 0);
}

#[test]
fn edit_of_maps_the_modified_keys_and_leaves_plain_keys_alone() {
    use crate::keys::Key;
    let key = |name: &'static str, ctrl, alt, cmd| Key { name, text: "", ctrl, alt, shift: false, cmd };
    assert_eq!(edit_of(key("left", false, true, false)), Some(FieldEdit::WordBackward));
    assert_eq!(edit_of(key("left", true, false, false)), Some(FieldEdit::WordBackward));
    assert_eq!(edit_of(key("left", false, false, true)), Some(FieldEdit::LineHome));
    assert_eq!(edit_of(key("backspace", false, true, false)), Some(FieldEdit::DeleteWordBackward));
    assert_eq!(edit_of(key("a", true, false, false)), Some(FieldEdit::LineHome));
    assert_eq!(edit_of(key("left", false, false, false)), None, "a plain arrow moves by character");
    assert_eq!(edit_of(key("f", true, false, false)), None, "Ctrl+F is the finder's fuzzy toggle");
    assert_eq!(edit_of(key("q", true, false, false)), None, "Ctrl+Q quits");
}

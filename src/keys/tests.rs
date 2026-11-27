use super::*;

const TREE: State =
    State { screen: Screen::Browse, focus: Focus::List, reader_ready: true, previewing: false, typing: false };
const READER: State = State { focus: Focus::Reader, ..TREE };
const CHANGES: State = State { screen: Screen::Changes, ..TREE };
const DIFF_BODY: State = State { screen: Screen::Changes, focus: Focus::Reader, ..TREE };

/// A key as OpenTUI Core reports it: `G` is name `g` with Shift; `^d` is Ctrl+d; `M-i` is Alt+i.
fn key(spec: &str) -> Key<'_> {
    let plain = Key { name: spec, text: spec, ctrl: false, alt: false, shift: false, cmd: false };
    if let Some(rest) = spec.strip_prefix("^") {
        return Key { name: rest, text: rest, ctrl: true, ..plain };
    }
    if let Some(rest) = spec.strip_prefix("M-") {
        return Key { name: rest, text: rest, alt: true, ..plain };
    }
    match spec {
        "↑" => Key { name: "up", text: "", ..plain },
        "↓" => Key { name: "down", text: "", ..plain },
        "←" => Key { name: "left", text: "", ..plain },
        "→" => Key { name: "right", text: "", ..plain },
        "enter" => Key { name: "return", text: "\r", ..plain },
        "space" => Key { name: "space", text: " ", ..plain },
        "esc" => Key { name: "escape", text: "\x1b", ..plain },
        "tab" | "backspace" | "home" | "end" | "pageup" | "pagedown" => Key { text: "", ..plain },
        _ if spec.len() == 1 && spec.chars().all(|c| c.is_ascii_uppercase()) => {
            let lower = match spec {
                "G" => "g",
                "N" => "n",
                "J" => "j",
                "K" => "k",
                "Q" => "q",
                "A" => "a",
                "D" => "d",
                "R" => "r",
                "S" => "s",
                _ => spec,
            };
            Key { name: lower, shift: true, ..plain }
        }
        _ => plain,
    }
}

fn run(keys: &[&str], state: State) -> Vec<Step> {
    let mut parser = Parser::default();
    keys.iter().map(|k| parser.feed(key(k), state)).collect()
}

fn last(keys: &[&str], state: State) -> Step {
    *run(keys, state).last().unwrap()
}

#[test]
fn quit_and_the_key_page_keys() {
    assert_eq!(last(&["^q"], READER), Step::Done(Action::Quit));
    assert_eq!(last(&["^c"], TREE), Step::Done(Action::Quit));
    // A plain `q` quits nowhere: it closes the open document from either pane; `?` toggles the key page, and Esc leaves it.
    assert_ne!(last(&["q"], TREE), Step::Done(Action::Quit));
    assert_eq!(last(&["q"], TREE), Step::Done(Action::Close));
    assert_eq!(last(&["q"], READER), Step::Done(Action::Close));
    assert_eq!(last(&["?"], TREE), Step::Done(Action::Help));
    assert_eq!(last(&["?"], READER), Step::Done(Action::Help));
    assert_eq!(last(&["esc"], READER), Step::Done(Action::Escape));
    assert_eq!(last(&["^c"], CHANGES), Step::Done(Action::Quit));
    // Reader keys scroll the key page: it is a reader document, with focus on the reader.
    assert_eq!(last(&["j"], READER), Step::Done(Action::Scroll(1)));
}

#[test]
fn ctrl_slash_and_its_legacy_byte_show_the_sidebar_but_not_in_a_prompt() {
    assert_eq!(last(&["^/"], TREE), Step::Done(Action::ToggleSidebar));
    assert_eq!(last(&["^/"], READER), Step::Done(Action::ToggleSidebar));
    assert_eq!(last(&["^_"], TREE), Step::Done(Action::ToggleSidebar));
    assert_eq!(last(&["^/"], State { typing: true, ..TREE }), Step::Bad);
}

#[test]
fn a_period_and_ctrl_period_toggle_dotfiles_but_a_prompt_types_the_period() {
    assert_eq!(last(&["."], TREE), Step::Done(Action::ToggleDotfiles));
    assert_eq!(last(&["."], READER), Step::Done(Action::ToggleDotfiles));
    assert_eq!(last(&["^."], TREE), Step::Done(Action::ToggleDotfiles));
    assert_eq!(last(&["."], State { typing: true, ..TREE }), Step::Bad);
    assert_eq!(last(&["^."], State { typing: true, ..TREE }), Step::Done(Action::ToggleDotfiles));
}

#[test]
fn ctrl_f_flips_the_finder_match_and_is_unbound_in_the_tree() {
    let finding = State { typing: true, ..TREE };
    assert_eq!(last(&["^f"], finding), Step::Done(Action::ToggleFuzzy));
    assert_eq!(last(&["^f"], TREE), Step::Bad);
}

#[test]
fn g_and_gg_go_to_the_top() {
    assert_eq!(run(&["g", "g"], TREE), [Step::Done(Action::Top), Step::Done(Action::Top)]);
    assert_eq!(last(&["G"], READER), Step::Done(Action::Bottom));
    assert_eq!(last(&["home"], READER), Step::Done(Action::Top));
    assert_eq!(last(&["end"], TREE), Step::Done(Action::Bottom));
}

#[test]
fn a_count_goes_to_a_line_and_resets() {
    assert_eq!(
        run(&["1", "2", "0", "G"], READER),
        [Step::More, Step::More, Step::More, Step::Done(Action::GoToLine(120))]
    );
    let mut parser = Parser::default();
    parser.feed(key("4"), READER);
    assert_eq!(parser.pending_count(), Some(4));
    assert_eq!(parser.feed(key("j"), READER), Step::Done(Action::Scroll(1)));
    assert_eq!(parser.pending_count(), None);
    assert_eq!(parser.feed(key("G"), READER), Step::Done(Action::Bottom));
}

#[test]
fn counts_need_a_ready_reader_in_browse() {
    assert_eq!(last(&["0"], READER), Step::Bad);
    assert_eq!(last(&["7"], TREE), Step::Bad);
    assert_eq!(last(&["7"], State { previewing: true, ..READER }), Step::Bad);
    assert_eq!(last(&["7"], DIFF_BODY), Step::Bad);
    assert_eq!(last(&["7"], State { reader_ready: false, ..READER }), Step::Bad);
    assert_eq!(run(&["9"; 9], READER).last(), Some(&Step::More));
    let mut parser = Parser::default();
    for _ in 0..12 {
        parser.feed(key("9"), READER);
    }
    assert_eq!(parser.pending_count(), Some(99_999_999));
}

#[test]
fn zz_centres_and_a_lone_z_waits() {
    assert_eq!(run(&["z", "z"], READER), [Step::More, Step::Done(Action::Centre)]);
    assert_eq!(run(&["z", "j"], READER), [Step::More, Step::Done(Action::Scroll(1))]);
    assert_eq!(run(&["z", "j", "z"], READER), [Step::More, Step::Done(Action::Scroll(1)), Step::More]);
    assert_eq!(last(&["z"], DIFF_BODY), Step::Bad);
}

#[test]
fn alt_i_arrives_as_escape_i() {
    assert_eq!(last(&["M-i"], READER), Step::Done(Action::JumpForward));
    assert_eq!(last(&["^o"], TREE), Step::Done(Action::JumpBack));
    assert_eq!(last(&["^o"], State { typing: true, ..TREE }), Step::Bad);
}

#[test]
fn letter_commands_are_plain_and_lowercase() {
    assert_eq!(last(&["w"], TREE), Step::Done(Action::ToggleWrap));
    assert_eq!(last(&["p"], TREE), Step::Done(Action::TogglePreview));
    assert_eq!(last(&["]"], TREE), Step::Done(Action::Jump(1)));
    assert_eq!(last(&["["], TREE), Step::Done(Action::Jump(-1)));
    assert_eq!(last(&["]"], READER), Step::Done(Action::Jump(1)));
    assert_eq!(last(&["["], READER), Step::Done(Action::Jump(-1)));
    assert_eq!(last(&["]"], CHANGES), Step::Done(Action::Hunk(1)), "the changes screen keeps its hunks");
    assert_eq!(last(&[":"], TREE), Step::Done(Action::OpenCommand));
    assert_eq!(last(&[":"], READER), Step::Done(Action::OpenCommand));
    assert_eq!(last(&["d"], READER), Step::Done(Action::ToggleChanges));
    assert_eq!(last(&["y"], CHANGES), Step::Done(Action::Copy));
    assert_eq!(last(&["M-y"], TREE), Step::Bad);
}

#[test]
fn changes_keys_work_in_any_focus() {
    for state in [CHANGES, DIFF_BODY] {
        assert_eq!(last(&["n"], state), Step::Done(Action::Hunk(1)));
        assert_eq!(last(&["]"], state), Step::Done(Action::Hunk(1)));
        assert_eq!(last(&["N"], state), Step::Done(Action::Hunk(-1)));
        assert_eq!(last(&["["], state), Step::Done(Action::Hunk(-1)));
        assert_eq!(last(&["x"], state), Step::Done(Action::ToggleContext));
    }
    assert_eq!(last(&["enter"], CHANGES), Step::Done(Action::FocusDiff));
    assert_eq!(last(&["J"], CHANGES), Step::Done(Action::Move(5)));
    assert_eq!(last(&["n"], READER), Step::Done(Action::NextMatch));
}

#[test]
fn tree_keys() {
    assert_eq!(last(&["enter"], TREE), Step::Done(Action::Toggle));
    assert_eq!(last(&["→"], TREE), Step::Done(Action::Expand));
    assert_eq!(last(&["backspace"], TREE), Step::Done(Action::Collapse));
    assert_eq!(last(&["space"], TREE), Step::Done(Action::ContextMenu));
    assert_eq!(last(&["-"], TREE), Step::Done(Action::GoUp));
    assert_eq!(last(&["/"], TREE), Step::Done(Action::Find));
    assert_eq!(last(&["pagedown"], TREE), Step::Done(Action::MovePage(1)));
    assert_eq!(last(&["K"], TREE), Step::Done(Action::Move(-5)));
}

#[test]
fn file_keys_act_on_the_tree_only() {
    assert_eq!(last(&["a"], TREE), Step::Done(Action::NewFile));
    assert_eq!(last(&["A"], TREE), Step::Done(Action::NewFolder));
    assert_eq!(last(&["r"], TREE), Step::Done(Action::Rename));
    assert_eq!(last(&["R"], TREE), Step::Done(Action::Duplicate));
    assert_eq!(last(&["D"], TREE), Step::Done(Action::Delete));
    assert_eq!(last(&["delete"], TREE), Step::Done(Action::Delete));
    assert_eq!(last(&["c"], TREE), Step::Done(Action::CopyFile));
    assert_eq!(last(&["s"], TREE), Step::Done(Action::ToggleMark));
    assert_eq!(last(&["S"], TREE), Step::Done(Action::ClearMarks));
    assert_eq!(last(&["s"], READER), Step::Bad, "the reader has no marks");
    assert_eq!(last(&["s"], CHANGES), Step::Done(Action::ToggleSplit), "the changes screen keeps its split");
    assert_eq!(last(&["x"], TREE), Step::Done(Action::CutFile));
    assert_eq!(last(&["v"], TREE), Step::Done(Action::PasteFile));
    // The reader keeps its own keys: these do nothing there.
    for key in ["c", "x", "v"] {
        assert_eq!(last(&[key], READER), Step::Bad, "{key} is a tree key");
    }
    assert_eq!(last(&["a"], READER), Step::Bad);
    assert_eq!(last(&["D"], READER), Step::Bad);
    assert_eq!(last(&["R"], READER), Step::Bad);
}

#[test]
fn reader_keys() {
    assert_eq!(last(&["space"], READER), Step::Done(Action::Page(1)));
    assert_eq!(last(&["b"], READER), Step::Done(Action::Page(-1)));
    assert_eq!(last(&["^d"], READER), Step::Done(Action::HalfPage(1)));
    assert_eq!(last(&["^x"], READER), Step::Bad);
    assert_eq!(last(&["h"], READER), Step::Done(Action::Pan(-4)));
    assert_eq!(last(&["{"], READER), Step::Done(Action::Paragraph(-1)));
    assert_eq!(last(&["/"], READER), Step::Done(Action::Search));
    assert_eq!(last(&["p"], READER), Step::Done(Action::PreviousMatch));
    assert_eq!(last(&["/"], DIFF_BODY), Step::Bad);
}

/// Every key the `?` card and `--help` list does something in the section's context.
#[test]
fn every_documented_key_is_bound() {
    for (title, rows) in KEY_SECTIONS {
        let state = match *title {
            "Reader" => READER,
            "Changes" => CHANGES,
            _ => TREE,
        };
        for (keys, _) in *rows {
            let tokens: Vec<&str> = keys.split(' ').collect();
            for (i, token) in tokens.iter().enumerate() {
                let sequence: Vec<String> = match *token {
                    "Ctrl+←" => vec!["^left".into()],
                    "→" if i > 0 && tokens[0] == "Ctrl+←" => vec!["^right".into()],
                    "Ctrl+o" => vec!["^o".into()],
                    "Alt+i" => vec!["M-i".into()],
                    "Ctrl+/" => vec!["^/".into()],
                    "Ctrl+." => vec!["^.".into()],
                    "Enter" => vec!["enter".into()],
                    "Backspace" => vec!["backspace".into()],
                    "Delete" => vec!["delete".into()],
                    "Tab" => vec!["tab".into()],
                    "Esc" => vec!["esc".into()],
                    "gg" => vec!["g".into(), "g".into()],
                    "zz" => vec!["z".into(), "z".into()],
                    "120G" => vec!["1".into(), "2".into(), "0".into(), "G".into()],
                    _ => vec![token.to_string()],
                };
                let refs: Vec<&str> = sequence.iter().map(String::as_str).collect();
                let step = last(&refs, state);
                assert!(matches!(step, Step::Done(_)), "{title}: {token} does nothing ({step:?})");
            }
        }
    }
}

/// The Keys page and `--help` both come from the key table: every row is in each, one table per section.
#[test]
fn the_key_page_and_help_come_from_the_key_table() {
    let page = key_page_markdown();
    let help = key_list_text();
    for (title, rows) in KEY_SECTIONS.iter().copied().chain([("Mouse", MOUSE_ROWS)]) {
        assert!(page.contains(&format!("## {title}\n")), "the page lacks {title}");
        for (keys, action) in rows {
            assert!(page.contains(&format!("| {action} | `{keys}` |")), "the page lacks {keys}");
            assert!(help.contains(&format!("{keys:<KEY_COLUMN$}{action}")), "--help lacks {keys}");
        }
    }
    assert_eq!(page.matches("| --- | --- |").count(), KEY_SECTIONS.len() + 1);
}

#[test]
fn keymaps_doc_lists_every_row() {
    let doc = include_str!("../../docs/keymaps.md");
    for (title, rows) in KEY_SECTIONS.iter().copied().chain([("Mouse", MOUSE_ROWS)]) {
        assert!(doc.contains(&format!("## {title}")), "docs/keymaps.md lacks {title}");
        for (keys, action) in rows {
            let cell = keys.split(' ').map(|k| format!("`{k}`")).collect::<Vec<_>>().join(" ");
            assert!(doc.contains(&format!("| {action} | {cell} |")), "docs/keymaps.md lacks {keys}");
        }
    }
}

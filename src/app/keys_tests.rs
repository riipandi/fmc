use super::*;
use crate::ui::hints::{Mode, Pane, Prompt, Shown, hints_for, prompt_hints};

#[test]
fn a_tap_on_enter_presses_return() {
    let presses = hint_presses("Enter").expect("Enter has a tap");
    assert_eq!(presses.len(), 1);
    assert_eq!(presses[0].name, "return");
}

#[test]
fn a_tap_on_gg_presses_g_twice() {
    assert_eq!(hint_presses("gg G").map(|presses| presses.len()), Some(2));
}

#[test]
fn quit_has_no_tap() {
    assert!(hint_presses("^q").is_none());
}

#[test]
fn every_footer_key_taps_except_the_named_few() {
    // `^q` stays on the keyboard, `h l` and `{ }` are motions with two keys, `^←→` is a resize, and `type` is no key.
    let exempt = ["^q", "h l", "{ }", "^←→", "type"];
    let mut hints = Vec::new();
    for mode in [Mode::Browse, Mode::Diff] {
        for pane in [Pane::Files, Pane::Reader] {
            for wrap in [false, true] {
                for all_files in [false, true] {
                    for markdown in [None, Some(false), Some(true)] {
                        for image in [false, true] {
                            for open in [false, true] {
                                for sidebar_hidden in [false, true] {
                                    let shown = Shown { markdown, image, open, sidebar_hidden };
                                    hints.extend(hints_for(mode, pane, wrap, all_files, shown));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    for prompt in [Prompt::Files, Prompt::Text, Prompt::Reader, Prompt::Command] {
        hints.extend(prompt_hints(prompt));
    }
    for (key, action) in hints {
        if !exempt.contains(&key) {
            assert!(hint_presses(key).is_some(), "the hint `{key} {action}` has no tap");
        }
    }
}

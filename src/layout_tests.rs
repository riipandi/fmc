use super::*;

#[test]
fn a_document_hides_the_sidebar_below_the_breakpoint_and_shows_it_at_the_breakpoint() {
    let mut v = Visibility::default();
    v.observe(79, true);
    assert!(!v.shown(79, true, false));
    v.observe(80, true);
    assert!(v.shown(80, true, false));
}

#[test]
fn the_welcome_screen_and_an_open_finder_hold_the_sidebar_at_any_width() {
    let mut v = Visibility::default();
    v.observe(40, false);
    assert!(v.shown(40, false, false));
    assert!(v.shown(40, true, true), "a finder holds the sidebar with a document open");
}

#[test]
fn a_manual_choice_holds_until_the_width_crosses_the_breakpoint() {
    let mut v = Visibility::default();
    v.observe(100, true);
    v.toggle(100, true, false);
    assert!(!v.shown(100, true, false));
    v.observe(120, true);
    assert!(!v.shown(120, true, false), "a resize that does not cross keeps the choice");
}

#[test]
fn crossing_the_breakpoint_returns_to_automatic() {
    let mut v = Visibility::default();
    v.observe(100, true);
    v.toggle(100, true, false);
    v.observe(79, true);
    v.observe(80, true);
    assert!(v.shown(80, true, false), "back at 80 the automatic state shows the sidebar again");
}

#[test]
fn opening_and_closing_a_document_returns_to_automatic() {
    let mut v = Visibility::default();
    v.observe(60, false);
    v.toggle(60, false, false);
    v.observe(60, true);
    assert!(!v.shown(60, true, false), "a document opening applies the 80-column rule");
    v.toggle(60, true, false);
    assert!(v.shown(60, true, false));
    v.observe(60, false);
    assert!(v.shown(60, false, false), "closing the document brings the list back");
    v.observe(60, true);
    assert!(!v.shown(60, true, false));
}

#[test]
fn a_manual_show_below_the_breakpoint_holds_while_the_width_stays_under_it() {
    let mut v = Visibility::default();
    v.observe(70, true);
    v.toggle(70, true, false);
    v.observe(75, true);
    assert!(v.shown(75, true, false));
}

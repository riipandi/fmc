use super::*;

#[test]
fn the_tier_splits_at_60_and_80_columns() {
    assert_eq!(tier(39), Tier::Compact);
    assert_eq!(tier(59), Tier::Compact);
    assert_eq!(tier(60), Tier::Medium);
    assert_eq!(tier(79), Tier::Medium);
    assert_eq!(tier(80), Tier::Wide);
}

#[test]
fn a_tier_is_ordered_from_compact_to_wide() {
    assert!(Tier::Compact < Tier::Medium && Tier::Medium < Tier::Wide);
}

#[test]
fn a_document_hides_the_sidebar_below_the_breakpoint_and_shows_it_at_the_breakpoint() {
    let mut v = Visibility::default();
    v.observe(79, true);
    assert!(!v.shown(79, true, false));
    v.observe(80, true);
    assert!(v.shown(80, true, false));
}

#[test]
fn the_welcome_screen_and_an_open_finder_hold_the_sidebar() {
    let mut v = Visibility::default();
    v.observe(70, false);
    assert!(v.shown(70, false, false));
    assert!(v.shown(70, true, true), "a finder holds the sidebar with a document open");
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
    v.observe(70, false);
    v.toggle(70, false, false);
    v.observe(70, true);
    assert!(!v.shown(70, true, false), "a document opening applies the 80-column rule");
    v.toggle(70, true, false);
    assert!(v.shown(70, true, false));
    v.observe(70, false);
    assert!(v.shown(70, false, false), "closing the document brings the list back");
}

#[test]
fn a_manual_show_below_the_breakpoint_holds_while_the_width_stays_under_it() {
    let mut v = Visibility::default();
    v.observe(70, true);
    v.toggle(70, true, false);
    v.observe(75, true);
    assert!(v.shown(75, true, false));
}

#[test]
fn a_page_opens_on_top_and_closes_out_of_the_stack() {
    let mut pages = Pages::default();
    assert_eq!(pages.top(), Page::Files);
    pages.sync(Page::Changes, true);
    pages.sync(Page::Reader, true);
    assert_eq!(pages.top(), Page::Reader);
    pages.sync(Page::Reader, false);
    assert_eq!(pages.top(), Page::Changes, "Back returns to the page below");
}

#[test]
fn a_page_already_open_keeps_its_place() {
    let mut pages = Pages::default();
    pages.sync(Page::Find, true);
    pages.sync(Page::Keys, true);
    pages.sync(Page::Find, true);
    assert_eq!(pages.top(), Page::Keys, "a repaint does not reorder the stack");
}

#[test]
fn files_is_the_bottom_page_and_is_never_removed() {
    let mut pages = Pages::default();
    pages.sync(Page::Files, false);
    assert_eq!(pages.top(), Page::Files);
}

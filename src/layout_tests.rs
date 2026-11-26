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
fn the_welcome_screen_keeps_the_sidebar_from_60_columns_and_starts_it_hidden_below() {
    let mut v = Visibility::default();
    v.observe(60, false);
    assert!(v.shown(60, false, false));
    v.observe(40, false);
    assert!(!v.shown(40, false, false), "below 60 the welcome's list starts hidden");
    assert!(v.shown(40, true, true), "a finder holds the sidebar with a document open");
    assert!(v.shown(40, false, true), "and a finder holds it on the welcome screen");
}

#[test]
fn a_manual_choice_shows_the_welcome_list_below_60_until_the_width_crosses() {
    let mut v = Visibility::default();
    v.observe(40, false);
    v.toggle(40, false, false);
    assert!(v.shown(40, false, false));
    v.toggle(40, false, false);
    assert!(!v.shown(40, false, false), "a second toggle hides it again");
}

#[test]
fn the_drawer_covers_the_reader_only_while_the_list_shows_below_60() {
    let mut v = Visibility::default();
    v.observe(40, false);
    assert!(!v.covers(false, false), "the welcome's list starts hidden");
    v.toggle(40, false, false);
    assert!(v.covers(false, false));
    v.observe(60, false);
    assert!(!v.covers(false, false), "at 60 columns the list sits beside the reader");
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

#[test]
fn a_shown_sidebar_covers_the_reader_only_below_the_drawer_breakpoint() {
    assert!(is_drawer(59, true));
    assert!(!is_drawer(60, true));
    assert!(!is_drawer(40, false), "a hidden sidebar covers nothing");
}

#[test]
fn closing_a_drawer_holds_until_the_next_crossing() {
    let mut v = Visibility::default();
    v.observe(40, true);
    v.toggle(40, true, false);
    v.close();
    assert!(!v.shown(40, true, false));
    v.observe(41, true);
    assert!(!v.shown(41, true, false), "a resize that does not cross keeps it closed");
}

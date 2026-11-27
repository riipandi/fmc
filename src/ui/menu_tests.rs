use fmc::layout::tier;

use super::*;

fn items(labels: &[&str]) -> Vec<Item> {
    labels.iter().map(|label| Item { label: (*label).into(), enabled: true }).collect()
}

#[test]
fn a_box_is_the_longest_label_plus_its_marker_and_border() {
    // `Delete…` is seven characters: a marker and a gap on each side of it, and the border.
    assert_eq!(size(&items(&["Cut", "Delete…"])), (13, 4));
}

#[test]
fn a_menu_inside_the_screen_stays_at_its_anchor() {
    assert_eq!(place((10, 5), (20, 6), (120, 40)), (10, 5));
}

#[test]
fn a_menu_at_the_right_edge_moves_left_to_fit() {
    assert_eq!(place((119, 5), (20, 6), (120, 40)), (100, 5));
    assert_eq!(place((100, 5), (20, 6), (120, 40)), (100, 5), "the last column that fits is kept");
}

#[test]
fn a_menu_at_the_bottom_edge_flips_above_its_anchor() {
    assert_eq!(place((10, 39), (20, 6), (120, 40)), (10, 33), "the box ends on the row above the anchor");
    assert_eq!(place((10, 34), (20, 6), (120, 40)), (10, 34), "a box that fits below the anchor stays there");
    assert_eq!(place((10, 35), (20, 6), (120, 40)), (10, 29), "the anchor row is left uncovered");
}

#[test]
fn a_menu_in_the_bottom_right_corner_flips_and_moves_at_once() {
    assert_eq!(place((119, 39), (20, 6), (120, 40)), (100, 33));
}

#[test]
fn a_menu_at_the_top_left_corner_starts_there() {
    assert_eq!(place((0, 0), (20, 6), (120, 40)), (0, 0));
}

#[test]
fn a_menu_taller_than_the_screen_is_held_at_the_top() {
    assert_eq!(place((5, 30), (20, 50), (120, 40)), (5, 0));
    assert_eq!(place((5, 5), (200, 6), (120, 40)), (0, 5), "a box wider than the screen starts at column 0");
}

#[test]
fn every_anchor_on_every_tested_screen_keeps_the_box_on_screen() {
    for screen in [(120, 40), (70, 24), (45, 30), (20, 8)] {
        for y in 0..screen.1 {
            for x in 0..screen.0 {
                let (px, py) = place((x, y), (14, 6), screen);
                assert!(px + 14 <= screen.0 && py + 6 <= screen.1, "{screen:?} at {x},{y} gives {px},{py}");
            }
        }
    }
}

#[test]
fn the_compact_tier_is_a_bottom_sheet_across_the_width_above_the_bar() {
    let frame = frame(&items(&["New file…", "Cut"]), (30, 10), (45, 30), tier(45));
    assert_eq!(frame, Rect { x: 0, y: 30 - 4 - 1, width: 45, height: 4 });
}

#[test]
fn the_medium_and_wide_tiers_place_the_box_at_its_anchor() {
    let items = items(&["New file…", "Cut"]);
    let wide = frame(&items, (30, 10), (120, 40), tier(120));
    assert_eq!((wide.x, wide.y), (30, 10));
    let medium = frame(&items, (65, 10), (70, 24), tier(70));
    assert_eq!(medium.x, 70 - medium.width, "the medium tier moves a box off the right edge");
}

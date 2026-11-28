use fmc::layout::tier;

use super::*;

/// Sample copy from Harry Potter: the sample data uses the novels' names, never placeholders.
const MESSAGE: &str = "Hermione Granger's essay on house-elves goes to the Trash. Finder's Put Back restores it.";

#[test]
fn a_message_wraps_at_its_words() {
    assert_eq!(wrap_rows("Gringotts", 20), 1);
    assert_eq!(wrap_rows("Albus Dumbledore sends Hedwig", 16), 2, "Albus Dumbledore / sends Hedwig");
    assert_eq!(wrap_rows("Albus Dumbledore sends Hedwig", 14), 3, "Dumbledore does not fit beside Albus at 14");
    assert_eq!(wrap_rows("one two", 7), 1, "a line that fits exactly is one row");
    assert_eq!(wrap_rows("", 10), 1, "an empty message still takes a row");
    assert_eq!(wrap_rows("a\nb", 10), 2, "a line break starts a row");
}

#[test]
fn the_wide_and_medium_tiers_centre_a_box_of_at_most_56_columns() {
    let wide = frame((120, 40), tier(120), MESSAGE, false);
    assert_eq!(wide.width, 56);
    assert_eq!(wide.x, (120 - 56) / 2);
    let medium = frame((70, 24), tier(70), MESSAGE, false);
    assert_eq!(medium.width, 56, "70 columns still fit the 56-column box with a margin");
    assert_eq!(medium.x, (70 - 56) / 2);
}

#[test]
fn the_compact_tier_spans_the_width_less_a_column_each_side() {
    let compact = frame((45, 30), tier(45), MESSAGE, true);
    assert_eq!((compact.x, compact.width), (1, 43));
}

#[test]
fn the_height_is_the_frame_the_message_and_the_field_need() {
    let lines = wrap_rows(MESSAGE, 52) as u16;
    assert_eq!(frame((120, 40), tier(120), MESSAGE, false).height, 7 + lines);
    assert_eq!(frame((120, 40), tier(120), MESSAGE, true).height, 8 + lines, "a field takes one more row");
}

#[test]
fn a_dialog_is_centred_vertically_on_the_screen() {
    let dialog = frame((120, 40), tier(120), MESSAGE, true);
    assert_eq!(dialog.y, (40 - dialog.height) / 2);
}

#[test]
fn every_dialog_stays_on_every_tested_screen() {
    for columns in 1..=130 {
        for rows in 1..=50 {
            let dialog = frame((columns, rows), tier(columns), MESSAGE, true);
            assert!(
                dialog.x + dialog.width <= columns && dialog.y + dialog.height <= rows,
                "{columns}x{rows} gives {dialog:?}"
            );
        }
    }
}

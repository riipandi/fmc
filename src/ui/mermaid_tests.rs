use super::*;

#[test]
fn box_glyphs_and_labels_take_separate_runs() {
    assert_eq!(
        runs("│ Harry ▸│"),
        vec![(true, "│".to_owned()), (false, " Harry ".to_owned()), (true, "▸│".to_owned())]
    );
}

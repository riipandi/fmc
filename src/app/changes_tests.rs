use super::*;

#[test]
fn the_changes_screen_opens_on_one_file() {
    assert!(Options::default().one_file);
    assert!(!Options::default().split);
}

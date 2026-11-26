use fmc::theme::DARK;
use opentui::testing::TestApp;

use super::*;

#[test]
fn the_binary_notice_on_screen_names_fmc_and_never_scouter() {
    let app = TestApp::new(80, 24);
    let view = BinaryView::new(app.app());
    app.app().root().add(&view.root());
    view.visible(true);
    view.paint("gringotts-vault.bin", "1 KB", &DARK);
    let screen = app.text();
    assert!(screen.contains("fmc only displays UTF-8 text and images."), "{screen}");
    assert!(!screen.to_lowercase().contains("scouter"), "{screen}");
}

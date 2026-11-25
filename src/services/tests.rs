use std::fs;

use super::fs::{Opened, open};

#[test]
fn open_lists_a_folder_and_names_a_file() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("Gringotts")).unwrap();
    fs::write(dir.path().join("vault-713.md"), "Griphook\n").unwrap();
    match open(&dir.path().join("Gringotts"), &[]).unwrap() {
        Opened::Folder(entries) => assert!(entries.is_empty()),
        _ => panic!("a folder opened as something else"),
    }
    assert!(matches!(open(&dir.path().join("vault-713.md"), &[]).unwrap(), Opened::Text(_)));
}

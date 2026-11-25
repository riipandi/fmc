use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use fmc::explorer::Kind;
use tempfile::TempDir;

use super::list;

/// A vault with the Harry Potter and Dan Brown names Scouter's fixtures use, plus the noise files it drops.
fn vault() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for folder in ["hogwarts", "Gringotts", "assets", "node_modules", ".git"] {
        fs::create_dir(root.join(folder)).unwrap();
    }
    for file in ["README.md", "app.log", ".gitignore", ".DS_Store", "Thumbs.db"] {
        fs::write(root.join(file), "Hermione\n").unwrap();
    }
    dir
}

#[test]
fn lists_folders_first_without_noise() {
    let dir = vault();
    let entries = list(dir.path(), &[]).unwrap();
    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["assets", "Gringotts", "hogwarts", "node_modules", ".gitignore", "app.log", "README.md"]);
    assert_eq!(entries[0].kind, Kind::Directory);
    assert_eq!(entries[6].kind, Kind::File);
    assert_eq!(entries[0].path, dir.path().join("assets"));
}

#[test]
fn a_missing_folder_is_an_error() {
    let dir = vault();
    assert!(list(&dir.path().join("azkaban"), &[]).is_err());
}

#[test]
fn symlink_kinds_are_not_followed() {
    let dir = vault();
    symlink(dir.path().join("README.md"), dir.path().join("readme-link")).unwrap();
    symlink(dir.path().join("hogwarts"), dir.path().join("hogwarts-link")).unwrap();
    let entries = list(dir.path(), &[]).unwrap();
    let kind = |name: &str| entries.iter().find(|e| e.name == name).unwrap().kind;
    assert_eq!(kind("readme-link"), Kind::Link);
    assert_eq!(kind("hogwarts-link"), Kind::Link);
}

#[test]
fn a_link_back_into_the_tree_is_a_loop() {
    let dir = vault();
    let root = dir.path();
    let inner = root.join("hogwarts");
    symlink("..", inner.join("loop")).unwrap();
    let parents: Vec<PathBuf> = vec![inner.clone(), root.to_path_buf()];

    let refused = list(&inner.join("loop"), &parents);
    assert_eq!(refused.unwrap_err(), "Cannot expand a symbolic-link loop.");
    // Scouter only checks a link that is itself being expanded.
    assert!(list(&inner, &parents).is_ok());
}

#[test]
fn a_link_outside_the_tree_expands() {
    let dir = vault();
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("Azkaban.md"), "Sirius\n").unwrap();
    symlink(outside.path(), dir.path().join("outside")).unwrap();
    let parents = [dir.path().to_path_buf()];
    let entries = list(&dir.path().join("outside"), &parents).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "Azkaban.md");
}

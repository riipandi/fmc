use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use fmc::explorer::Kind;
use tempfile::TempDir;

use super::index;

/// Harry Potter names in a folder tree, with the noise Scouter drops and one link that must not be followed.
fn tree() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("hogwarts/gryffindor")).unwrap();
    fs::create_dir_all(root.join("azkaban")).unwrap();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::write(root.join("hogwarts/gryffindor/common-room.md"), "Hermione\n").unwrap();
    fs::write(root.join("hogwarts/owls.txt"), "Hedwig\n").unwrap();
    fs::write(root.join("azkaban/cell.txt"), "Sirius\n").unwrap();
    fs::write(root.join("Thumbs.db"), "noise\n").unwrap();
    fs::write(root.join("README.md"), "Dumbledore\n").unwrap();
    symlink(root.join("hogwarts"), root.join("hogwarts-link")).unwrap();
    dir
}

fn relatives(found: &fmc::find::Index) -> Vec<&str> {
    found.entries.iter().map(|entry| entry.entry.name.as_str()).collect()
}

#[test]
fn walks_one_level_at_a_time_with_folders_first_and_no_noise() {
    let dir = tree();
    let found = index(dir.path(), |_| false, 100);
    assert!(!found.truncated);
    assert_eq!(
        relatives(&found),
        [
            "azkaban",
            "hogwarts",
            "README.md",
            "hogwarts-link",
            "azkaban/cell.txt",
            "hogwarts/gryffindor",
            "hogwarts/owls.txt",
            "hogwarts/gryffindor/common-room.md",
        ]
    );
    let link = found.entries.iter().find(|entry| entry.entry.name == "hogwarts-link").unwrap();
    assert_eq!(link.entry.kind, Kind::Link);
}

#[test]
fn a_skipped_folder_is_left_out_with_everything_under_it() {
    let dir = tree();
    let skipped = dir.path().join("hogwarts");
    let found = index(dir.path(), |path: &Path| path == skipped, 100);
    assert!(!relatives(&found).iter().any(|name| name.starts_with("hogwarts/")));
    assert!(relatives(&found).contains(&"hogwarts-link"));
}

#[test]
fn the_walk_stops_at_the_limit_and_says_so() {
    let dir = tree();
    let found = index(dir.path(), |_| false, 3);
    assert!(found.truncated);
    assert_eq!(found.entries.len(), 3);
    let exact = index(dir.path(), |_| false, 8);
    assert!(!exact.truncated);
    assert_eq!(exact.entries.len(), 8);
}

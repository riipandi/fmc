//! Runs real `git` on temporary repositories: the marks, and the promise that the user's index is untouched.

use std::fs;
use std::path::Path;
use std::process::Command;

use fmc::diff::ChangeKind;
use tempfile::TempDir;

use super::marks;

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git").arg("-C").arg(dir).args(args).status().unwrap();
    assert!(status.success(), "git {args:?}");
}

/// A repository with one committed file, one staged, one modified, one untracked, and one ignored.
fn repository() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.email", "dumbledore@hogwarts.test"]);
    git(root, &["config", "user.name", "Albus Dumbledore"]);
    fs::write(root.join(".gitignore"), "target/\n").unwrap();
    fs::create_dir(root.join("spells")).unwrap();
    fs::write(root.join("spells/lumos.md"), "Lumos\n").unwrap();
    fs::write(root.join("spells/expecto.md"), "Expecto\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-q", "-m", "Hogwarts"]);
    fs::write(root.join("spells/lumos.md"), "Lumos maxima\n").unwrap();
    fs::write(root.join("spells/alohomora.md"), "Alohomora\n").unwrap();
    git(root, &["add", "spells/alohomora.md"]);
    fs::create_dir(root.join("target")).unwrap();
    fs::write(root.join("target/out.bin"), "0\n").unwrap();
    fs::write(root.join("notes.txt"), "Azkaban\n").unwrap();
    dir
}

#[test]
fn marks_list_every_kind_of_change_and_ignored_entry() {
    let dir = repository();
    let listing = marks(dir.path()).unwrap().unwrap();
    let mut changes = listing.changes.clone();
    changes.sort_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(
        changes,
        [
            ("notes.txt".to_string(), ChangeKind::Untracked),
            ("spells/alohomora.md".to_string(), ChangeKind::Added),
            ("spells/lumos.md".to_string(), ChangeKind::Modified),
        ]
    );
    assert_eq!(listing.ignored, ["target/"]);
    assert_eq!(listing.within, "");
}

#[test]
fn a_folder_inside_the_repository_is_marked_relative_to_it() {
    let dir = repository();
    let listing = marks(&dir.path().join("spells")).unwrap().unwrap();
    assert_eq!(listing.within, "spells");
    assert!(listing.changes.iter().any(|(path, _)| path == "spells/lumos.md"));
}

#[test]
fn a_directory_outside_any_repository_has_no_marks() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(marks(dir.path()).unwrap(), None);
}

#[test]
fn a_repository_without_commits_lists_everything_as_added() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    fs::write(dir.path().join("Hermione.md"), "Granger\n").unwrap();
    git(dir.path(), &["add", "Hermione.md"]);
    let listing = marks(dir.path()).unwrap().unwrap();
    assert_eq!(listing.changes, [("Hermione.md".to_string(), ChangeKind::Added)]);
}

#[test]
fn the_repository_index_is_byte_identical_after_marks() {
    let dir = repository();
    let index = dir.path().join(".git/index");
    let before = fs::read(&index).unwrap();
    let modified = fs::metadata(&index).unwrap().modified().unwrap();
    marks(dir.path()).unwrap();
    marks(dir.path()).unwrap();
    assert_eq!(fs::read(&index).unwrap(), before);
    assert_eq!(fs::metadata(&index).unwrap().modified().unwrap(), modified);
}

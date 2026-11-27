//! Runs real `git` on temporary repositories: each kind of change, the empty and non-repository cases, the limits and
//! the timeout, and the promise that the user's index is untouched.

use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use fmc::diff::{Change, ChangeKind, Patch, Sign, binary, not_shown, too_large};
use tempfile::TempDir;

use super::*;

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git").arg("-C").arg(dir).args(args).status().unwrap();
    assert!(status.success(), "git {args:?}");
}

/// A repository with an identity and one commit, which holds `Hogwarts` and nothing else.
fn initialized() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.email", "dumbledore@hogwarts.test"]);
    git(root, &["config", "user.name", "Albus Dumbledore"]);
    fs::write(root.join("Hogwarts.md"), "Hogwarts\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-q", "-m", "Hogwarts"]);
    dir
}

/// A repository with one committed file, one staged, one modified, one untracked, and one ignored.
fn repository() -> TempDir {
    let dir = initialized();
    let root = dir.path();
    fs::write(root.join(".gitignore"), "target/\n").unwrap();
    fs::create_dir(root.join("spells")).unwrap();
    fs::write(root.join("spells/lumos.md"), "Lumos\n").unwrap();
    fs::write(root.join("spells/expecto.md"), "Expecto\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-q", "-m", "Spells"]);
    fs::write(root.join("spells/lumos.md"), "Lumos maxima\n").unwrap();
    fs::write(root.join("spells/alohomora.md"), "Alohomora\n").unwrap();
    git(root, &["add", "spells/alohomora.md"]);
    fs::create_dir(root.join("target")).unwrap();
    fs::write(root.join("target/out.bin"), "0\n").unwrap();
    fs::write(root.join("notes.txt"), "Azkaban\n").unwrap();
    dir
}

/// A work tree with a change of every kind: M, D, R (staged), T, A (staged), a binary M, and an untracked file.
fn every_kind() -> TempDir {
    let dir = initialized();
    let root = dir.path();
    fs::create_dir(root.join("spells")).unwrap();
    for (path, text) in [
        ("spells/lumos.md", "Lumos\n"),
        ("spells/expecto.md", "Expecto\n"),
        // Different text from `alohomora.md`, so `-M` pairs only the rename with its old path.
        ("spells/old.md", "Sesame\n"),
        ("spells/link.md", "Accio\n"),
        ("seal.bin", "\0\u{1}\u{2}"),
    ] {
        fs::write(root.join(path), text).unwrap();
    }
    git(root, &["add", "."]);
    git(root, &["commit", "-q", "-m", "Spells"]);
    fs::write(root.join("spells/lumos.md"), "Lumos maxima\n").unwrap();
    fs::write(root.join("seal.bin"), "\0\u{9}").unwrap();
    fs::remove_file(root.join("spells/expecto.md")).unwrap();
    fs::remove_file(root.join("spells/link.md")).unwrap();
    symlink("Accio.md", root.join("spells/link.md")).unwrap();
    git(root, &["mv", "spells/old.md", "spells/new.md"]);
    fs::write(root.join("spells/alohomora.md"), "Alohomora\n").unwrap();
    git(root, &["add", "spells/alohomora.md"]);
    fs::write(root.join("notes.txt"), "Azkaban\n").unwrap();
    dir
}

/// A repository where `main` and `side` changed the same line, and the merge left the file unmerged.
fn conflicted() -> TempDir {
    let dir = initialized();
    let root = dir.path();
    fs::write(root.join("Hogwarts.md"), "one\n").unwrap();
    git(root, &["commit", "-q", "-am", "one"]);
    git(root, &["checkout", "-q", "-b", "side"]);
    fs::write(root.join("Hogwarts.md"), "two\n").unwrap();
    git(root, &["commit", "-q", "-am", "two"]);
    git(root, &["checkout", "-q", "-"]);
    fs::write(root.join("Hogwarts.md"), "three\n").unwrap();
    git(root, &["commit", "-q", "-am", "three"]);
    // The merge stops on the conflict; that is the state under test, so its status is not checked.
    let _ = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["merge", "-q", "side"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    dir
}

fn kinds(changes: &[Change]) -> Vec<(String, ChangeKind)> {
    let mut pairs: Vec<(String, ChangeKind)> =
        changes.iter().map(|change| (change.path.clone(), change.kind)).collect();
    pairs.sort_by(|left, right| left.0.cmp(&right.0));
    pairs
}

fn pair(path: &str, kind: ChangeKind) -> (String, ChangeKind) {
    (path.to_string(), kind)
}

#[test]
fn every_kind_of_change_is_listed() {
    let dir = every_kind();
    let set = Repository::open(dir.path()).unwrap().changes().unwrap();
    assert_eq!(
        kinds(&set.changes),
        [
            pair("notes.txt", ChangeKind::Untracked),
            pair("seal.bin", ChangeKind::Modified),
            pair("spells/alohomora.md", ChangeKind::Added),
            pair("spells/expecto.md", ChangeKind::Deleted),
            pair("spells/link.md", ChangeKind::Typechange),
            pair("spells/lumos.md", ChangeKind::Modified),
            pair("spells/new.md", ChangeKind::Renamed),
        ]
    );
    assert!(!set.truncated);
    let renamed = set.changes.iter().find(|change| change.kind == ChangeKind::Renamed).unwrap();
    assert_eq!(renamed.old_path.as_deref(), Some("spells/old.md"));
    let staged = |path: &str| set.changes.iter().find(|change| change.path == path).unwrap().staged;
    assert_eq!(staged("spells/alohomora.md"), Some(true));
    assert_eq!(staged("spells/lumos.md"), Some(false));
}

#[test]
fn counts_and_binary_come_from_numstat() {
    let dir = repository();
    let set = Repository::open(dir.path()).unwrap().changes().unwrap();
    let lumos = set.changes.iter().find(|change| change.path == "spells/lumos.md").unwrap();
    assert_eq!((lumos.added, lumos.removed, lumos.binary), (Some(1), Some(1), false));
    let added = set.changes.iter().find(|change| change.path == "spells/alohomora.md").unwrap();
    assert_eq!((added.added, added.removed), (Some(1), Some(0)));
    let untracked = set.changes.iter().find(|change| change.path == "notes.txt").unwrap();
    assert_eq!((untracked.added, untracked.staged, untracked.unstaged), (None, None, None));
    let binary = every_kind();
    let seal = Repository::open(binary.path()).unwrap().changes().unwrap();
    let seal = seal.changes.iter().find(|change| change.path == "seal.bin").unwrap();
    assert!(seal.binary);
    assert_eq!((seal.added, seal.removed), (None, None));
}

#[test]
fn both_marks_show_when_a_change_is_staged_and_then_edited() {
    let dir = repository();
    let root = dir.path();
    git(root, &["add", "spells/lumos.md"]);
    fs::write(root.join("spells/lumos.md"), "Lumos nox\n").unwrap();
    let set = Repository::open(root).unwrap().changes().unwrap();
    let lumos = set.changes.iter().find(|change| change.path == "spells/lumos.md").unwrap();
    assert_eq!((lumos.staged, lumos.unstaged), (Some(true), Some(true)));
}

#[test]
fn a_conflicted_file_is_listed_as_git_reports_it_against_the_base() {
    // `git diff <base> --name-status` reports a conflicted file as modified; the `U` letter does not come from this
    // command, so Scouter's and fmc's lists show it as `M`.
    let dir = conflicted();
    let set = Repository::open(dir.path()).unwrap().changes().unwrap();
    assert_eq!(kinds(&set.changes), [pair("Hogwarts.md", ChangeKind::Modified)]);
}

#[test]
fn a_repository_without_commits_lists_everything_as_added() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    fs::write(dir.path().join("Hermione.md"), "Granger\n").unwrap();
    git(dir.path(), &["add", "Hermione.md"]);
    fs::write(dir.path().join("Granger.txt"), "Hermione\n").unwrap();
    let repository = Repository::open(dir.path()).unwrap();
    let set = repository.changes().unwrap();
    assert_eq!(
        kinds(&set.changes),
        [pair("Granger.txt", ChangeKind::Untracked), pair("Hermione.md", ChangeKind::Added)]
    );
    let added = set.changes.iter().find(|change| change.path == "Hermione.md").unwrap();
    assert_eq!((added.staged, added.unstaged), (Some(true), Some(false)));
}

#[test]
fn marks_list_every_kind_of_change_and_ignored_entry() {
    let dir = repository();
    let listing = marks(dir.path()).unwrap().unwrap();
    assert_eq!(
        kinds(&listing.changes),
        [
            pair("notes.txt", ChangeKind::Untracked),
            pair("spells/alohomora.md", ChangeKind::Added),
            pair("spells/lumos.md", ChangeKind::Modified),
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
    assert!(listing.changes.iter().any(|change| change.path == "spells/lumos.md"));
}

#[test]
fn a_directory_outside_any_repository_has_no_marks() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(marks(dir.path()).unwrap(), None);
    assert_eq!(Repository::open(dir.path()).err().as_deref(), Some("Not a git repository."));
}

#[test]
fn the_repository_index_is_byte_identical_after_every_read() {
    let dir = every_kind();
    let index = dir.path().join(".git/index");
    let before = fs::read(&index).unwrap();
    let modified = fs::metadata(&index).unwrap().modified().unwrap();
    let repository = Repository::open(dir.path()).unwrap();
    let set = repository.changes().unwrap();
    repository.patches(&set.changes, 3).unwrap();
    marks(dir.path()).unwrap();
    assert_eq!(fs::read(&index).unwrap(), before);
    assert_eq!(fs::metadata(&index).unwrap().modified().unwrap(), modified);
}

#[test]
fn patches_cover_each_kind_of_change() {
    let dir = every_kind();
    let repository = Repository::open(dir.path()).unwrap();
    let set = repository.changes().unwrap();
    let patches = repository.patches(&set.changes, 3).unwrap();
    let patch = |path: &str| &patches.iter().find(|file| file.change.path == path).unwrap().patch;
    assert!(matches!(patch("spells/lumos.md"), Patch::Text { hunks: 1, added: 1, removed: 1, .. }));
    assert!(matches!(patch("spells/alohomora.md"), Patch::Text { hunks: 1, added: 1, removed: 0, .. }));
    assert!(matches!(patch("spells/expecto.md"), Patch::Text { hunks: 1, added: 0, removed: 1, .. }));
    assert!(matches!(patch("notes.txt"), Patch::Text { hunks: 1, added: 1, removed: 0, .. }));
    assert_eq!(patch("seal.bin"), &binary());
    assert_eq!(patch("spells/new.md"), &Patch::Notice("Renamed from spells/old.md with no content changes.".into()));
    // The one-file call agrees with the stacked call.
    let lumos = set.changes.iter().find(|change| change.path == "spells/lumos.md").unwrap();
    assert_eq!(&repository.patch(lumos, 3).unwrap(), patch("spells/lumos.md"));
}

#[test]
fn a_file_patch_past_one_mebibyte_is_too_large() {
    let dir = repository();
    let root = dir.path();
    fs::write(root.join("spells/lumos.md"), "Lumos\n".repeat(200_000)).unwrap();
    let repository = Repository::open(root).unwrap();
    let set = repository.changes().unwrap();
    let lumos = set.changes.iter().find(|change| change.path == "spells/lumos.md").unwrap();
    assert_eq!(repository.patch(lumos, 3).unwrap(), too_large());
}

#[test]
fn the_stacked_diff_stops_at_four_mebibytes_and_the_rest_is_not_shown() {
    let dir = initialized();
    let root = dir.path();
    for number in 0..8 {
        fs::write(root.join(format!("spell-{number}.md")), "Expecto Patronum\n".repeat(40_000)).unwrap();
    }
    let repository = Repository::open(root).unwrap();
    let set = repository.changes().unwrap();
    let patches = repository.patches(&set.changes, 3).unwrap();
    let shown = patches.iter().filter(|file| matches!(file.patch, Patch::Text { .. })).count();
    // Each file's patch is about 720 KB, so the sixth one crosses the 4 MiB budget and the last two are not shown.
    assert_eq!(shown, 6);
    assert!(patches[6..].iter().all(|file| file.patch == not_shown()));
}

#[test]
fn a_timed_out_command_ends_with_scouters_status() {
    let mut sleep = Command::new("sleep");
    sleep.arg("5");
    let error = capture(sleep, LIST_LIMIT, false, Duration::from_millis(200)).err();
    assert_eq!(error.as_deref(), Some("git exited with status 143."));
}

#[test]
fn a_failing_command_reports_its_first_stderr_line() {
    let mut fail = Command::new("sh");
    fail.args(["-c", "echo 'fatal: Dumbledore' >&2; echo 'second' >&2; exit 3"]);
    assert_eq!(capture(fail, LIST_LIMIT, false, TIMEOUT).err().as_deref(), Some("fatal: Dumbledore"));
    let mut silent = Command::new("sh");
    silent.args(["-c", "exit 3"]);
    assert_eq!(capture(silent, LIST_LIMIT, false, TIMEOUT).err().as_deref(), Some("git exited with status 3."));
}

#[test]
fn an_output_past_its_cap_is_cut_and_a_partial_one_keeps_its_bytes() {
    let cap = 64 * 1024;
    let cut = capture(Command::new("yes"), cap, false, TIMEOUT).unwrap();
    assert!(cut.truncated);
    assert!(cut.text.is_empty(), "a cut listing has no text");
    let partial = capture(Command::new("yes"), cap, true, TIMEOUT).unwrap();
    assert!(partial.truncated);
    assert_eq!(partial.text.len(), cap);
}

#[test]
fn the_racy_window_and_the_stamps_decide_reuse() {
    let now = SystemTime::now();
    let settled = now - Duration::from_secs(10);
    let stamp = Stamp { source: (1, 2, 3, 4, 5, 6), copy: (7, 8, 9, 10, 11, 12) };
    assert!(reusable(Some(stamp), stamp.source, settled, Some(stamp.copy), now));
    assert!(!reusable(Some(stamp), stamp.source, now - Duration::from_secs(1), Some(stamp.copy), now), "racy");
    assert!(!reusable(Some(stamp), (0, 0, 0, 0, 0, 0), settled, Some(stamp.copy), now), "the index changed");
    assert!(!reusable(Some(stamp), stamp.source, settled, None, now), "the copy is gone");
    assert!(!reusable(None, stamp.source, settled, Some(stamp.copy), now), "no earlier copy");
}

#[test]
fn a_copy_keeps_the_index_time_even_when_the_index_is_read_only() {
    let dir = initialized();
    let root = dir.path();
    let source = root.join("source-index");
    fs::write(&source, "DIRC").unwrap();
    let old = SystemTime::now() - Duration::from_secs(10);
    File::options().write(true).open(&source).unwrap().set_modified(old).unwrap();
    let mut permissions = fs::metadata(&source).unwrap().permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&source, permissions).unwrap();
    let copy = root.join("copy-index");
    let mut stamp = None;
    refresh_copy(&source, &copy, &mut stamp).unwrap();
    assert_eq!(fs::metadata(&copy).unwrap().modified().unwrap(), old);
    assert!(stamp.is_some());
}

#[test]
fn dropping_the_repository_removes_its_copy() {
    let dir = repository();
    let repository = Repository::open(dir.path()).unwrap();
    repository.changes().unwrap();
    let copy = repository.copy_dir.clone();
    assert!(copy.exists());
    drop(repository);
    assert!(!copy.exists());
}

#[test]
fn a_copy_is_listed_for_the_quit_until_its_repository_drops() {
    let dir = repository();
    let repository = Repository::open(dir.path()).unwrap();
    let copy = repository.copy_dir.clone();
    assert!(copies().contains(&copy));
    drop(repository);
    assert!(!copies().contains(&copy));
}

#[test]
fn calls_on_one_repository_from_two_threads_both_answer() {
    let dir = repository();
    let repository = Arc::new(Repository::open(dir.path()).unwrap());
    let threads: Vec<_> = (0..2)
        .map(|_| {
            let repository = Arc::clone(&repository);
            std::thread::spawn(move || repository.changes().map(|set| set.changes.len()))
        })
        .collect();
    for thread in threads {
        assert_eq!(thread.join().unwrap(), Ok(3));
    }
}

#[test]
fn the_stamp_changes_when_a_listed_file_or_the_index_changes() {
    let dir = repository();
    let repository = Repository::open(dir.path()).unwrap();
    let set = repository.changes().unwrap();
    let before = repository.stamp(&set.changes);
    assert_eq!(before, repository.stamp(&set.changes), "nothing changed, so the stamp holds");
    fs::write(dir.path().join("notes.txt"), "Azkaban and Hogsmeade\n").unwrap();
    assert_ne!(repository.stamp(&set.changes), before);
}

/// A repository whose `spells.txt` is committed with three lines, then left as `edit` makes it.
fn committed_spells(edit: Option<&str>) -> TempDir {
    let dir = initialized();
    let root = dir.path();
    fs::write(root.join("spells.txt"), "Hermione Granger\nRobert Langdon\nSophie Neveu\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-q", "-m", "Harry Potter"]);
    if let Some(text) = edit {
        fs::write(root.join("spells.txt"), text).unwrap();
    }
    dir
}

#[test]
fn an_edited_line_is_modified_and_the_others_have_no_sign() {
    let dir = committed_spells(Some("Hermione Granger\nLeigh Teabing\nSophie Neveu\n"));
    assert_eq!(signs(&dir.path().join("spells.txt")), Ok(vec![(1, Sign::Modified)]));
}

#[test]
fn a_line_added_at_the_end_is_added() {
    let dir = committed_spells(Some("Hermione Granger\nRobert Langdon\nSophie Neveu\nVittoria Vetra\n"));
    assert_eq!(signs(&dir.path().join("spells.txt")), Ok(vec![(3, Sign::Added)]));
}

#[test]
fn a_deleted_line_is_drawn_on_the_line_above() {
    let dir = committed_spells(Some("Hermione Granger\nSophie Neveu\n"));
    assert_eq!(signs(&dir.path().join("spells.txt")), Ok(vec![(0, Sign::DeletedBelow)]));
}

#[test]
fn a_committed_file_has_no_signs() {
    let dir = committed_spells(None);
    assert_eq!(signs(&dir.path().join("spells.txt")), Ok(Vec::new()));
}

#[test]
fn an_untracked_file_is_all_added() {
    let dir = initialized();
    fs::write(dir.path().join("letter.txt"), "Albus Dumbledore\nMinerva McGonagall\n").unwrap();
    assert_eq!(signs(&dir.path().join("letter.txt")), Ok(vec![(0, Sign::Added), (1, Sign::Added)]));
}

#[test]
fn a_file_outside_a_repository_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("spells.txt"), "Hogwarts\n").unwrap();
    assert_eq!(signs(&dir.path().join("spells.txt")), Err(NOT_A_REPOSITORY.to_string()));
}

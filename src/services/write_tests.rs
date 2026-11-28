//! Every operation and every refusal of the write layer, in temporary folders only. Nothing here touches the
//! repository, `$HOME`, or the real Trash: the Trash call is behind `trash_with`, and a test passes its own `delete`.

use std::cell::Cell;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

use super::{
    Error, copy, copy_entry, copy_seen, copy_then_remove, create_dir, create_file, free_name, move_to, move_with,
    open_seen, paste, rename, trash_with,
};

/// The browsed root, with Harry Potter and Dan Brown names, and a folder outside it that a link can point at.
struct Fixture {
    _dir: TempDir,
    _outside_dir: TempDir,
    root: PathBuf,
    outside: PathBuf,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let outside_dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let outside = outside_dir.path().to_path_buf();
    fs::create_dir_all(root.join("hogwarts/great-hall")).unwrap();
    fs::create_dir(root.join("Gringotts")).unwrap();
    fs::write(root.join("hogwarts/Harry Potter.md"), "Harry Potter\n").unwrap();
    fs::write(root.join("hogwarts/great-hall/Albus Dumbledore.md"), "Albus Dumbledore\n").unwrap();
    fs::write(root.join("README.md"), "Robert Langdon\n").unwrap();
    fs::write(outside.join("vault-713.txt"), "Griphook\n").unwrap();
    Fixture { _dir: dir, _outside_dir: outside_dir, root, outside }
}

/// An empty folder, for the tests that count what they write.
fn empty_root() -> TempDir {
    tempfile::tempdir().unwrap()
}

fn sorted_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> =
        fs::read_dir(dir).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    names.sort();
    names
}

/// Removes an entry without following it, as the test's stand-in for the Trash.
fn drop_entry(path: &Path) -> Result<(), Error> {
    if fs::symlink_metadata(path)?.is_dir() {
        fs::remove_dir_all(path)?
    } else {
        fs::remove_file(path)?
    }
    Ok(())
}

#[test]
fn create_file_makes_an_empty_file_in_the_folder() {
    let root = empty_root();
    let path = create_file(root.path(), root.path(), "robert-langdon.md").unwrap();
    assert_eq!(path, root.path().join("robert-langdon.md"));
    assert_eq!(fs::metadata(&path).unwrap().len(), 0);
}

#[test]
fn create_file_refuses_a_taken_name_and_keeps_its_content() {
    let f = fixture();
    let err = create_file(&f.root, &f.root, "README.md").unwrap_err();
    assert!(matches!(err, Error::Exists));
    assert_eq!(fs::read_to_string(f.root.join("README.md")).unwrap(), "Robert Langdon\n");
}

#[test]
fn create_dir_makes_a_folder_and_refuses_a_taken_name() {
    let f = fixture();
    let path = create_dir(&f.root, &f.root, "langdon").unwrap();
    assert!(path.is_dir());
    assert!(matches!(create_dir(&f.root, &f.root, "langdon"), Err(Error::Exists)));
    assert!(matches!(create_dir(&f.root, &f.root, "Gringotts"), Err(Error::Exists)));
}

#[test]
fn a_taken_name_includes_a_dangling_link() {
    let f = fixture();
    symlink(f.outside.join("missing"), f.root.join("sophie-neveu.md")).unwrap();
    assert!(matches!(create_file(&f.root, &f.root, "sophie-neveu.md"), Err(Error::Exists)));
}

#[test]
fn a_name_is_one_segment() {
    let f = fixture();
    for bad in ["", ".", "..", "langdon/teabing", "/etc", "vittoria\0vetra"] {
        assert!(matches!(create_file(&f.root, &f.root, bad), Err(Error::BadName)), "create_file {bad:?}");
        assert!(matches!(create_dir(&f.root, &f.root, bad), Err(Error::BadName)), "create_dir {bad:?}");
        let source = f.root.join("README.md");
        assert!(matches!(rename(&f.root, &source, bad), Err(Error::BadName)), "rename {bad:?}");
        assert!(matches!(copy(&f.root, &source, &f.root, bad), Err(Error::BadName)), "copy {bad:?}");
        assert!(matches!(move_to(&f.root, &source, &f.root, bad), Err(Error::BadName)), "move {bad:?}");
    }
    assert_eq!(sorted_names(&f.root), ["Gringotts", "README.md", "hogwarts"]);
}

#[test]
fn a_name_may_hold_spaces_and_dots() {
    let f = fixture();
    let path = create_file(&f.root, &f.root.join("hogwarts"), "Hermione Granger.notes.md").unwrap();
    assert!(path.is_file());
}

#[test]
fn a_parent_outside_the_root_is_refused() {
    let f = fixture();
    let parent = f.root.join("..");
    assert!(matches!(create_file(&f.root, &parent, "da-vinci.md"), Err(Error::Outside)));
    assert!(matches!(create_dir(&f.root, &f.outside, "da-vinci"), Err(Error::Outside)));
    assert!(!f.outside.join("da-vinci.md").exists());
    assert!(!f.outside.join("da-vinci").exists());
}

#[test]
fn a_link_out_of_the_root_is_not_written_through() {
    let f = fixture();
    symlink(&f.outside, f.root.join("louvre")).unwrap();
    let parent = f.root.join("louvre");
    assert!(matches!(create_file(&f.root, &parent, "mona-lisa.md"), Err(Error::Outside)));
    assert!(matches!(create_dir(&f.root, &parent, "salle-denon"), Err(Error::Outside)));
    assert_eq!(sorted_names(&f.outside), ["vault-713.txt"]);
}

#[test]
fn the_root_itself_cannot_be_renamed_copied_or_trashed() {
    let f = fixture();
    let name = "vitruvian";
    assert!(matches!(rename(&f.root, &f.root, name), Err(Error::Outside)));
    assert!(matches!(copy(&f.root, &f.root, &f.outside, name), Err(Error::Outside)));
    let mut called = false;
    assert!(matches!(
        trash_with(&f.root, &f.root, |_| {
            called = true;
            Ok(())
        }),
        Err(Error::Outside)
    ));
    assert!(!called);
}

#[test]
fn rename_moves_the_entry_to_the_new_name() {
    let f = fixture();
    let before = f.root.join("README.md");
    let after = rename(&f.root, &before, "sophie-neveu.md").unwrap();
    assert_eq!(after, f.root.join("sophie-neveu.md"));
    assert!(!before.exists());
    assert_eq!(fs::read_to_string(&after).unwrap(), "Robert Langdon\n");
}

#[test]
fn rename_refuses_a_taken_name_and_keeps_both() {
    let f = fixture();
    let err = rename(&f.root, &f.root.join("README.md"), "Gringotts").unwrap_err();
    assert!(matches!(err, Error::Exists));
    assert!(f.root.join("README.md").is_file());
    assert!(f.root.join("Gringotts").is_dir());
}

#[test]
fn rename_renames_a_link_and_leaves_its_target() {
    let f = fixture();
    let link = f.root.join("vault");
    symlink(f.outside.join("vault-713.txt"), &link).unwrap();
    let after = rename(&f.root, &link, "vault-door").unwrap();
    assert!(fs::symlink_metadata(&after).unwrap().file_type().is_symlink());
    assert!(fs::symlink_metadata(&link).is_err());
    assert_eq!(fs::read_to_string(f.outside.join("vault-713.txt")).unwrap(), "Griphook\n");
}

#[test]
fn copy_puts_a_file_in_another_folder_and_keeps_its_mode() {
    let f = fixture();
    let source = f.root.join("README.md");
    fs::set_permissions(&source, fs::Permissions::from_mode(0o4755)).unwrap();
    let dest = copy(&f.root, &source, &f.root.join("hogwarts"), "README.md").unwrap();
    assert_eq!(fs::read_to_string(&dest).unwrap(), "Robert Langdon\n");
    // The setuid bit of the source does not come along: the copy keeps the mode bits only.
    assert_eq!(fs::metadata(&dest).unwrap().permissions().mode() & 0o7777, 0o755);
    assert!(source.is_file());
}

#[test]
fn copy_takes_a_folder_with_everything_in_it() {
    let f = fixture();
    let dest = copy(&f.root, &f.root.join("hogwarts"), &f.root.join("Gringotts"), "hogwarts").unwrap();
    assert_eq!(fs::read_to_string(dest.join("great-hall/Albus Dumbledore.md")).unwrap(), "Albus Dumbledore\n");
    assert_eq!(fs::read_to_string(f.root.join("hogwarts/Harry Potter.md")).unwrap(), "Harry Potter\n");
}

#[test]
fn copy_keeps_a_link_inside_a_folder_as_a_link() {
    let f = fixture();
    symlink(f.outside.join("vault-713.txt"), f.root.join("hogwarts/owl-post")).unwrap();
    let dest = copy(&f.root, &f.root.join("hogwarts"), &f.root.join("Gringotts"), "hogwarts").unwrap();
    let copied = dest.join("owl-post");
    assert!(fs::symlink_metadata(&copied).unwrap().file_type().is_symlink());
    assert_eq!(fs::read_link(&copied).unwrap(), f.outside.join("vault-713.txt"));
    assert_eq!(fs::read_to_string(f.outside.join("vault-713.txt")).unwrap(), "Griphook\n");
}

#[test]
fn copy_does_not_follow_a_link_to_a_folder_outside_the_root() {
    let f = fixture();
    fs::write(f.outside.join("secret.md"), "Teabing\n").unwrap();
    symlink(&f.outside, f.root.join("hogwarts/forbidden")).unwrap();
    let dest = copy(&f.root, &f.root.join("hogwarts"), &f.root.join("Gringotts"), "hogwarts").unwrap();
    assert!(fs::symlink_metadata(dest.join("forbidden")).unwrap().file_type().is_symlink());
    assert_eq!(sorted_names(&f.outside), ["secret.md", "vault-713.txt"]);
}

#[test]
fn copy_refuses_a_folder_into_itself() {
    let f = fixture();
    let source = f.root.join("hogwarts");
    let err = copy(&f.root, &source, &source.join("great-hall"), "hogwarts").unwrap_err();
    assert!(matches!(err, Error::IntoItself));
    let err = copy(&f.root, &source, &source, "hogwarts").unwrap_err();
    assert!(matches!(err, Error::IntoItself));
    assert_eq!(sorted_names(&source), ["Harry Potter.md", "great-hall"]);
}

#[test]
fn copy_refuses_a_taken_name_and_keeps_the_original() {
    let f = fixture();
    fs::write(f.root.join("Gringotts/README.md"), "Leigh Teabing\n").unwrap();
    let err = copy(&f.root, &f.root.join("README.md"), &f.root.join("Gringotts"), "README.md").unwrap_err();
    assert!(matches!(err, Error::Exists));
    assert_eq!(fs::read_to_string(f.root.join("Gringotts/README.md")).unwrap(), "Leigh Teabing\n");
}

#[test]
fn copy_refuses_a_pipe_and_writes_nothing() {
    let f = fixture();
    let pipe = f.root.join("hogwarts/owl-post-pipe");
    assert!(Command::new("mkfifo").arg(&pipe).status().unwrap().success());
    let err = copy(&f.root, &pipe, &f.root.join("Gringotts"), "owl-post-pipe").unwrap_err();
    assert!(matches!(err, Error::Unsupported));
    assert!(!f.root.join("Gringotts/owl-post-pipe").exists());
}

#[test]
fn a_folder_copy_that_fails_halfway_leaves_no_partial_copy() {
    let f = fixture();
    let portraits = f.root.join("portraits");
    fs::create_dir(&portraits).unwrap();
    fs::write(portraits.join("Fat Lady.md"), "Sir Cadogan\n").unwrap();
    assert!(Command::new("mkfifo").arg(portraits.join("owl-post-pipe")).status().unwrap().success());
    let err = copy(&f.root, &portraits, &f.root.join("Gringotts"), "portraits").unwrap_err();
    assert!(matches!(err, Error::Unsupported));
    assert!(!f.root.join("Gringotts/portraits").exists());
    assert!(portraits.join("Fat Lady.md").is_file());
}

#[test]
fn move_puts_the_entry_under_the_new_name_on_the_same_volume() {
    let f = fixture();
    let dest = move_to(&f.root, &f.root.join("README.md"), &f.root.join("Gringotts"), "robert-langdon.md").unwrap();
    assert_eq!(dest, f.root.join("Gringotts/robert-langdon.md"));
    assert!(!f.root.join("README.md").exists());
    assert_eq!(fs::read_to_string(dest).unwrap(), "Robert Langdon\n");
}

#[test]
fn move_refuses_a_folder_into_itself_and_a_taken_name() {
    let f = fixture();
    let source = f.root.join("hogwarts");
    assert!(matches!(move_to(&f.root, &source, &source.join("great-hall"), "hogwarts"), Err(Error::IntoItself)));
    assert!(source.join("great-hall").is_dir());
    assert!(matches!(move_to(&f.root, &f.root.join("README.md"), &f.root, "Gringotts"), Err(Error::Exists)));
    assert!(f.root.join("README.md").is_file());
}

#[test]
fn the_cross_volume_fallback_copies_then_removes_the_source() {
    let f = fixture();
    symlink(f.outside.join("vault-713.txt"), f.root.join("hogwarts/owl-post")).unwrap();
    let source = f.root.join("hogwarts");
    let dest = f.root.join("Gringotts/hogwarts");
    copy_then_remove(&source, &dest, copy_entry, drop_entry).unwrap();
    assert_eq!(fs::read_to_string(dest.join("great-hall/Albus Dumbledore.md")).unwrap(), "Albus Dumbledore\n");
    assert!(fs::symlink_metadata(dest.join("owl-post")).unwrap().file_type().is_symlink());
    assert!(!source.exists());
}

#[test]
fn the_cross_volume_fallback_keeps_the_source_when_the_copy_fails() {
    let f = fixture();
    let source = f.root.join("README.md");
    let dest = f.root.join("Gringotts/README.md");
    fs::write(&dest, "Leigh Teabing\n").unwrap();
    let removed = Cell::new(false);
    let err = copy_then_remove(&source, &dest, copy_entry, |_| {
        removed.set(true);
        Ok(())
    })
    .unwrap_err();
    assert!(matches!(err, Error::Exists));
    assert!(!removed.get());
    assert!(source.is_file());
}

#[test]
fn the_cross_volume_fallback_keeps_the_source_when_it_is_replaced_during_the_copy() {
    let f = fixture();
    let source = f.root.join("README.md");
    let dest = f.root.join("Gringotts/README.md");
    let removed = Cell::new(false);
    let err = copy_then_remove(
        &source,
        &dest,
        |from, to| {
            copy_entry(from, to)?;
            // The old file is moved aside first, so its inode is not freed and reused for the new file.
            fs::rename(from, f.root.join("kept-aside.md"))?;
            fs::write(from, "Sirius Black\n")?;
            Ok(())
        },
        |_| {
            removed.set(true);
            Ok(())
        },
    )
    .unwrap_err();
    assert!(matches!(err, Error::Changed), "{err}");
    assert!(!removed.get(), "the replacement is not trashed");
    assert_eq!(fs::read_to_string(&source).unwrap(), "Sirius Black\n");
    assert_eq!(fs::read_to_string(&dest).unwrap(), "Robert Langdon\n", "the copy is left in place");
}

#[test]
fn a_link_swapped_in_after_the_lstat_is_refused_before_it_is_read() {
    let f = fixture();
    let link = f.root.join("hogwarts/owl-post");
    symlink(f.outside.join("vault-713.txt"), &link).unwrap();
    // The lstat saw the link; the open follows it to the file outside the root.
    let seen = fs::symlink_metadata(&link).unwrap();
    assert!(matches!(open_seen(&link, &seen), Err(Error::Changed)));
    // The file the lstat saw is opened as before.
    let readme = f.root.join("README.md");
    assert!(open_seen(&readme, &fs::symlink_metadata(&readme).unwrap()).is_ok());
}

#[test]
fn a_folder_swapped_for_a_link_after_its_lstat_is_refused_before_anything_is_copied() {
    let f = fixture();
    let hall = f.root.join("hogwarts/great-hall");
    // The lstat sees the folder; then the folder is replaced by a link to the folder outside the root, which holds
    // vault-713.txt. Copying through the link would copy that file.
    let seen = fs::symlink_metadata(&hall).unwrap();
    fs::remove_dir_all(&hall).unwrap();
    symlink(&f.outside, &hall).unwrap();
    let dest = f.root.join("Gringotts/great-hall");
    assert!(matches!(copy_seen(&hall, &dest, &seen), Err(Error::Changed)));
    assert!(fs::symlink_metadata(&dest).is_err(), "nothing is made, so the outside folder is not copied");
    // The same call on a folder the lstat still describes copies it, with its children.
    let portraits = f.root.join("portraits");
    fs::create_dir(&portraits).unwrap();
    fs::write(portraits.join("Fat Lady.md"), "Sir Cadogan\n").unwrap();
    let seen = fs::symlink_metadata(&portraits).unwrap();
    copy_seen(&portraits, &f.root.join("Gringotts/portraits"), &seen).unwrap();
    assert!(f.root.join("Gringotts/portraits/Fat Lady.md").is_file());
}
#[test]
fn the_cross_volume_fallback_keeps_both_when_the_removal_fails() {
    let f = fixture();
    let source = f.root.join("README.md");
    let dest = f.root.join("Gringotts/README.md");
    let err = copy_then_remove(&source, &dest, copy_entry, |_| Err(Error::Trash("Trash is full".into()))).unwrap_err();
    assert!(matches!(err, Error::Trash(_)));
    assert!(source.is_file() && dest.is_file());
}

#[test]
fn a_same_volume_move_never_calls_the_remover() {
    let f = fixture();
    let mut removed = false;
    let dest = move_with(&f.root, &f.root.join("README.md"), &f.root.join("Gringotts"), "README.md", |_| {
        removed = true;
        Ok(())
    })
    .unwrap();
    assert!(dest.is_file());
    assert!(!removed);
}

#[test]
fn trash_hands_the_entry_to_the_delete_seam() {
    let f = fixture();
    let path = f.root.join("hogwarts/Harry Potter.md");
    let seen = Cell::new(None);
    trash_with(&f.root, &path, |p| {
        seen.set(Some(p.to_path_buf()));
        drop_entry(p)
    })
    .unwrap();
    assert_eq!(seen.take(), Some(path.clone()));
    assert!(!path.exists());
}

#[test]
fn trash_refuses_what_the_root_does_not_hold_and_calls_no_delete() {
    let f = fixture();
    let mut called = false;
    let mut delete = |_: &Path| {
        called = true;
        Ok(())
    };
    assert!(matches!(trash_with(&f.root, &f.root.join("hogwarts/.."), &mut delete), Err(Error::BadName)));
    assert!(matches!(trash_with(&f.root, &f.root.join("hogwarts/great-hall/../.."), &mut delete), Err(Error::BadName)));
    assert!(matches!(trash_with(&f.root, &f.root.join("Robert Langdon.md"), &mut delete), Err(Error::Missing)));
    assert!(matches!(trash_with(&f.root, &f.outside.join("vault-713.txt"), &mut delete), Err(Error::Outside)));
    assert!(!called);
}

#[test]
fn trash_refuses_a_name_that_is_not_utf8_and_calls_no_delete() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let f = fixture();
    // A decoy with the percent-encoded spelling: the crate would move this one if the name went through.
    let decoy = f.root.join("caf%E9.md");
    fs::write(&decoy, "Griphook\n").unwrap();
    let path = f.root.join(OsStr::from_bytes(b"caf\xe9.md"));
    let mut called = false;
    let result = trash_with(&f.root, &path, |_| {
        called = true;
        Ok(())
    });
    assert!(matches!(result, Err(Error::NotUtf8)));
    assert!(!called);
    assert!(decoy.exists());
}

#[test]
fn trash_keeps_a_link_as_the_path_it_is_given() {
    let f = fixture();
    let link = f.root.join("vault");
    symlink(f.outside.join("vault-713.txt"), &link).unwrap();
    trash_with(&f.root, &link, drop_entry).unwrap();
    assert!(fs::symlink_metadata(&link).is_err());
    assert_eq!(fs::read_to_string(f.outside.join("vault-713.txt")).unwrap(), "Griphook\n");
}

#[test]
fn free_name_adds_copy_then_copy_two_before_the_extension() {
    let root = empty_root();
    let dir = root.path();
    assert_eq!(free_name(dir, "robert-langdon.md", false).unwrap(), "robert-langdon copy.md");
    fs::write(dir.join("robert-langdon copy.md"), "").unwrap();
    assert_eq!(free_name(dir, "robert-langdon.md", false).unwrap(), "robert-langdon copy 2.md");
    fs::write(dir.join("robert-langdon copy 2.md"), "").unwrap();
    assert_eq!(free_name(dir, "robert-langdon.md", false).unwrap(), "robert-langdon copy 3.md");
}

#[test]
fn free_name_puts_the_suffix_at_the_end_of_a_folder_a_dotfile_and_a_bare_name() {
    let root = empty_root();
    let dir = root.path();
    fs::create_dir(dir.join("Gringotts.vault")).unwrap();
    fs::write(dir.join(".gitignore"), "").unwrap();
    fs::write(dir.join("Hogwarts"), "").unwrap();
    assert_eq!(free_name(dir, "Gringotts.vault", true).unwrap(), "Gringotts.vault copy");
    assert_eq!(free_name(dir, ".gitignore", false).unwrap(), ".gitignore copy");
    assert_eq!(free_name(dir, "Hogwarts", false).unwrap(), "Hogwarts copy");
}

#[test]
fn free_name_splits_at_the_last_dot_and_keeps_a_name_with_a_trailing_dot_whole() {
    let root = empty_root();
    let dir = root.path();
    assert_eq!(free_name(dir, "archive.tar.gz", false).unwrap(), "archive.tar copy.gz");
    assert_eq!(free_name(dir, "Teabing.", false).unwrap(), "Teabing. copy");
}

#[test]
fn free_name_refuses_a_name_that_is_not_one_segment() {
    let root = empty_root();
    assert!(matches!(free_name(root.path(), "", false), Err(Error::BadName)));
    assert!(matches!(free_name(root.path(), "a/b", false), Err(Error::BadName)));
}

#[test]
fn paste_names_a_copy_in_the_same_folder_before_the_extension() {
    let f = fixture();
    let source = f.root.join("harry-potter.md");
    fs::write(&source, "Hedwig\n").unwrap();
    let copied = paste(&f.root, &source, &f.root, false).unwrap();
    assert_eq!(copied, f.root.join("harry-potter copy.md"));
    assert_eq!(fs::read_to_string(&source).unwrap(), "Hedwig\n", "the original stays");
}

#[test]
fn paste_moves_a_cut_entry_under_its_own_name_when_the_folder_has_no_such_name() {
    let f = fixture();
    let source = f.root.join("sophie-neveu.md");
    fs::write(&source, "Teabing\n").unwrap();
    let dest = f.root.join("Gringotts");
    let moved = paste(&f.root, &source, &dest, true).unwrap();
    assert_eq!(moved, dest.join("sophie-neveu.md"));
    assert!(fs::symlink_metadata(&source).is_err(), "the source is gone");
    assert_eq!(fs::read_to_string(&moved).unwrap(), "Teabing\n");
}

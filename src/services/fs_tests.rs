use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use fmc::explorer::Kind;
use tempfile::TempDir;

use super::{Opened, Reread, describe, list, open, reread};

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

#[test]
fn file_errors_read_like_node() {
    let dir = vault();
    let missing = dir.path().join("azkaban");
    assert_eq!(
        list(&missing, &[]).unwrap_err(),
        format!("ENOENT: no such file or directory, lstat '{}'", missing.display())
    );
    let file = dir.path().join("README.md");
    assert_eq!(list(&file, &[]).unwrap_err(), format!("ENOTDIR: not a directory, scandir '{}'", file.display()));
}

#[test]
fn a_read_error_has_no_path_like_node() {
    let error = io::Error::from(io::ErrorKind::IsADirectory);
    assert_eq!(describe(&error, "read", None), "EISDIR: illegal operation on a directory, read");
    let error = io::Error::from(io::ErrorKind::PermissionDenied);
    assert_eq!(
        describe(&error, "scandir", Some(Path::new("/var/root"))),
        "EACCES: permission denied, scandir '/var/root'"
    );
}

#[test]
fn a_text_file_opens_as_a_document() {
    let dir = vault();
    let path = dir.path().join("README.md");
    fs::write(&path, "Hermione\r\nGranger\n").unwrap();
    let Opened::Text { document, .. } = open(&path, &[]).unwrap() else { panic!("a text file") };
    assert_eq!(document.text, "Hermione\nGranger\n");
    assert_eq!(document.lines, 2);
}

#[test]
fn a_binary_file_opens_as_its_size_and_a_non_utf8_file_is_refused() {
    let dir = vault();
    fs::write(dir.path().join("owl.bin"), b"Hedwig\0").unwrap();
    fs::write(dir.path().join("owl.txt"), b"Hedwig \xff").unwrap();
    let Opened::Binary { size } = open(&dir.path().join("owl.bin"), &[]).unwrap() else { panic!("a binary") };
    assert_eq!(size, 7);
    assert_eq!(open(&dir.path().join("owl.txt"), &[]).unwrap_err(), "Only UTF-8 text files are supported.");
}

/// A 1x1 PNG, the smallest valid header the decoder reads.
const PNG_1X1: &[u8] = &[
    0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 0x0d, b'I', b'H', b'D', b'R', 0, 0, 0, 1, 0, 0, 0, 1, 8,
    6, 0, 0, 0, 0x1f, 0x15, 0xc4, 0x89, 0, 0, 0, 0x0d, b'I', b'D', b'A', b'T', 0x78, 0x9c, 0x63, 0x60, 0xf8, 0xff,
    0xff, 0x3f, 0, 0x05, 0xfe, 0x02, 0xfe, 0xa7, 0xd8, 0x9c, 0x0a, 0, 0, 0, 0, b'I', b'E', b'N', b'D', 0xae, 0x42,
    0x60, 0x82,
];

#[test]
fn an_image_opens_with_its_header_facts() {
    let dir = vault();
    let path = dir.path().join("marauder-map.PNG");
    fs::write(&path, PNG_1X1).unwrap();
    let Opened::Image { info, bytes } = open(&path, &[]).unwrap() else { panic!("an image") };
    assert_eq!((info.width, info.height, info.format), (1, 1, "png"));
    assert_eq!(bytes, PNG_1X1);
}

#[test]
fn an_image_that_will_not_decode_is_unreadable_and_a_big_one_is_refused_unread() {
    let dir = vault();
    let broken = dir.path().join("seal.png");
    fs::write(&broken, b"Hedwig").unwrap();
    assert_eq!(open(&broken, &[]).unwrap_err(), "This image could not be read.");
    let big = dir.path().join("map.jpg");
    fs::File::create(&big).unwrap().set_len(10 * 1024 * 1024 + 1).unwrap();
    assert_eq!(open(&big, &[]).unwrap_err(), "Image is too large (limit: 10 MiB).");
}

#[test]
fn a_missing_file_is_an_error_with_the_stat_call() {
    let dir = vault();
    let path = dir.path().join("azkaban.txt");
    assert_eq!(open(&path, &[]).unwrap_err(), format!("ENOENT: no such file or directory, stat '{}'", path.display()));
}

#[test]
fn a_poll_rereads_a_changed_file_and_reports_a_missing_one() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vault-713.md");
    fs::write(&path, "Gringotts\n").unwrap();
    let Opened::Text { stamp, .. } = open(&path, &[]).unwrap() else { panic!("a text file") };
    assert!(matches!(reread(&path, &stamp), Reread::Unchanged));
    fs::write(&path, "Gringotts and the Louvre\n").unwrap();
    let Reread::Changed { document, stamp: next } = reread(&path, &stamp) else { panic!("a changed file") };
    assert_eq!(document.text, "Gringotts and the Louvre\n");
    assert_ne!(next, stamp);
    fs::remove_file(&path).unwrap();
    assert!(matches!(reread(&path, &next), Reread::Missing));
}

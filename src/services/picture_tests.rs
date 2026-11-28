//! Each image's header is read in a temporary folder: the reasons a box can give, the cache by modification time, and a
//! symbolic link that leaves the root.

use std::fs;
use std::time::{Duration, SystemTime};

use fmc::reader::image::{self, Info};
use fmc::reader::images::{self, Header};

use super::*;

/// A PNG's signature and header, with the size written where the decoder reads it.
fn png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    bytes.extend_from_slice(&13u32.to_be_bytes());
    bytes.extend_from_slice(b"IHDR");
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.extend_from_slice(&[8, 2, 0, 0, 0, 0, 0, 0, 0]);
    bytes
}

/// The Hogwarts folder under a temporary root, its notes folder, and a folder outside the root.
struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    notes: PathBuf,
    outside: PathBuf,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let root = dir.path().join("hogwarts");
    let notes = root.join("notes");
    let outside = dir.path().join("outside");
    fs::create_dir_all(root.join("maps")).expect("maps");
    fs::create_dir_all(&notes).expect("notes");
    fs::create_dir_all(&outside).expect("outside");
    fs::write(root.join("maps/marauder.png"), png(480, 270)).expect("the marauder map");
    fs::write(outside.join("seal.png"), png(64, 64)).expect("a seal outside the root");
    Fixture { _dir: dir, root, notes, outside }
}

fn one(fixture: &Fixture, target: &str) -> Header {
    let cache = Cache::default();
    headers(&cache, &fixture.root, &fixture.notes, &[target.to_owned()]).remove(0).1
}

#[test]
fn a_local_picture_is_read_for_its_size_and_format() {
    let fixture = fixture();
    let header = one(&fixture, "../maps/marauder.png");
    let path = fs::canonicalize(fixture.root.join("maps/marauder.png")).expect("the real path");
    let Header::Picture { modified, .. } = &header else { panic!("a picture: {header:?}") };
    assert!(modified.is_some(), "the modification time keys the decoded picture");
    assert_eq!(
        header,
        Header::Picture { path, info: Info { width: 480, height: 270, format: "png" }, modified: *modified }
    );
}

#[test]
fn a_percent_escaped_name_is_found_under_its_decoded_name() {
    let fixture = fixture();
    fs::write(fixture.root.join("maps/Marauder map.png"), png(320, 200)).expect("a name with a space");
    match one(&fixture, "../maps/Marauder%20map.png") {
        Header::Picture { info, .. } => assert_eq!((info.width, info.height), (320, 200)),
        other => panic!("{other:?}"),
    }
}

#[test]
fn each_reason_a_box_can_give_names_its_cause() {
    let fixture = fixture();
    let cases = [
        ("https://hogwarts.test/seal.png", images::REMOTE),
        ("../crest.svg", images::SVG),
        ("../notes/vault.txt", images::NOT_SHOWN),
        ("../maps/missing.png", images::NOT_FOUND),
        ("../../outside/seal.png", images::OUTSIDE),
    ];
    for (target, reason) in cases {
        assert_eq!(one(&fixture, target), Header::Refused(reason), "{target}");
    }
    fs::write(fixture.root.join("maps/fake.png"), b"not a picture at all").expect("a file named like a picture");
    assert_eq!(one(&fixture, "../maps/fake.png"), Header::Refused(images::UNREADABLE));
}

#[test]
fn a_symbolic_link_that_leaves_the_root_is_outside_the_folder() {
    let fixture = fixture();
    std::os::unix::fs::symlink(fixture.outside.join("seal.png"), fixture.root.join("maps/link.png"))
        .expect("a link out of the root");
    assert_eq!(one(&fixture, "../maps/link.png"), Header::Refused(images::OUTSIDE));
}

#[test]
fn an_image_over_the_size_cap_is_too_large_and_is_not_read() {
    let fixture = fixture();
    let file = fs::File::create(fixture.root.join("maps/huge.png")).expect("a file");
    file.set_len(image::MAX_BYTES + 1).expect("a sparse file over the cap");
    assert_eq!(one(&fixture, "../maps/huge.png"), Header::Refused(images::TOO_LARGE));
}

#[test]
fn a_header_is_read_once_per_modification_time() {
    let fixture = fixture();
    let cache = Cache::default();
    let target = ["../maps/marauder.png".to_owned()];
    let first = headers(&cache, &fixture.root, &fixture.notes, &target);
    let path = fs::canonicalize(fixture.root.join("maps/marauder.png")).expect("the real path");
    // Same modification time: the cached answer stands, even though the bytes changed.
    let stamp = fs::metadata(&path).and_then(|m| m.modified()).expect("a stamp");
    fs::write(&path, png(10, 10)).expect("a new picture");
    fs::File::options().write(true).open(&path).and_then(|file| file.set_modified(stamp)).expect("the old stamp");
    assert_eq!(headers(&cache, &fixture.root, &fixture.notes, &target), first);
    // A later modification time reads the header again.
    let later = SystemTime::now() + Duration::from_secs(60);
    fs::File::options().write(true).open(&path).and_then(|file| file.set_modified(later)).expect("a later stamp");
    match headers(&cache, &fixture.root, &fixture.notes, &target).remove(0).1 {
        Header::Picture { info, .. } => assert_eq!((info.width, info.height), (10, 10)),
        other => panic!("{other:?}"),
    }
}

use super::*;
use crate::theme::DARK;

fn listing(within: &str, changes: &[(&str, ChangeKind)], ignored: &[&str]) -> Listing {
    Listing {
        within: within.into(),
        changes: changes.iter().map(|(path, kind)| (path.to_string(), *kind)).collect(),
        ignored: ignored.iter().map(|path| path.to_string()).collect(),
    }
}

#[test]
fn name_status_pairs_and_renames() {
    let text = "M\0src/hogwarts.rs\0A\0Gringotts/vault-713.ts\0R100\0lumos.md\0spells/lumos.md\0D\0azkaban.txt\0";
    assert_eq!(
        parse_name_status(text),
        [
            ("src/hogwarts.rs".to_string(), ChangeKind::Modified),
            ("Gringotts/vault-713.ts".to_string(), ChangeKind::Added),
            ("spells/lumos.md".to_string(), ChangeKind::Renamed),
            ("azkaban.txt".to_string(), ChangeKind::Deleted),
        ]
    );
}

#[test]
fn unknown_letters_count_as_modified() {
    assert_eq!(ChangeKind::from_status("C075"), ChangeKind::Modified);
    assert_eq!(ChangeKind::from_status("T"), ChangeKind::Typechange);
    assert_eq!(ChangeKind::from_status("U"), ChangeKind::Conflict);
}

#[test]
fn split_z_drops_only_the_trailing_empty_token() {
    assert_eq!(split_z("a\0b\0"), ["a", "b"]);
    assert_eq!(split_z("a\0\0"), ["a", ""]);
    assert!(split_z("").is_empty());
}

#[test]
fn marks_are_relative_to_the_browsed_folder() {
    let p = &DARK;
    let marks = Marks::new(listing(
        "app",
        &[("app/src/hogwarts.rs", ChangeKind::Modified), ("docs/notes.md", ChangeKind::Added)],
        &[],
    ));
    assert_eq!(marks.mark("src/hogwarts.rs", p).map(|m| m.letter), Some("M"));
    assert_eq!(marks.mark("src", p).map(|m| m.letter), Some("•"));
    assert!(marks.mark("docs", p).is_none(), "outside the browsed folder");
}

#[test]
fn a_change_at_the_root_has_no_folder_dot() {
    let marks = Marks::new(listing("", &[("src/lib/mod.rs", ChangeKind::Untracked)], &[]));
    assert_eq!(marks.mark("src/lib", &DARK).map(|m| m.letter), Some("•"));
    assert_eq!(marks.mark("src/lib/mod.rs", &DARK).map(|m| m.letter), Some("?"));
    assert!(marks.mark("src/lib/mod.rs/", &DARK).is_none());
}

#[test]
fn ignored_folders_cover_their_children() {
    let marks = Marks::new(listing("", &[], &["target/", "notes/secret.txt", "build.log"]));
    assert!(marks.is_ignored("target"));
    assert!(marks.is_ignored("target/debug/fmc"));
    assert!(marks.is_ignored("notes/secret.txt"));
    assert!(!marks.is_ignored("notes/public.txt"));
    assert!(marks.is_ignored("build.log"));
}

#[test]
fn an_ignored_folder_outside_the_browsed_one_does_not_mark_it() {
    let marks = Marks::new(listing("app", &[], &["target/", "app/dist/"]));
    assert!(!marks.is_ignored("target"));
    assert!(marks.is_ignored("dist/bundle.js"));
}

#[test]
fn the_cap_counts_changes_outside_the_folder() {
    // 1000 changes sort before "zz": the browsed folder's one change sits after the cut and is not marked.
    let mut changes: Vec<(String, ChangeKind)> =
        (0..1000).map(|n| (format!("aa/{n:04}.txt"), ChangeKind::Modified)).collect();
    changes.push(("zz/inside.txt".into(), ChangeKind::Added));
    let marks = Marks::new(Listing { within: "zz".into(), changes, ignored: vec![] });
    assert!(marks.mark("inside.txt", &DARK).is_none());
}

//! Change kinds, git's listings, the marks, and the patch text, on values: no git and no terminal.

use std::collections::HashMap;

use super::*;
use crate::reader::document::{self, Document, decode};
use crate::theme::DARK;

fn change(path: &str, kind: ChangeKind) -> Change {
    Change { kind, ..Change::untracked(path) }
}

fn listing(within: &str, changes: &[(&str, ChangeKind)], ignored: &[&str]) -> Listing {
    Listing {
        within: within.into(),
        changes: changes.iter().map(|(path, kind)| change(path, *kind)).collect(),
        ignored: ignored.iter().map(|path| path.to_string()).collect(),
    }
}

fn pairs(changes: &[Change]) -> Vec<(&str, ChangeKind)> {
    changes.iter().map(|change| (change.path.as_str(), change.kind)).collect()
}

/// The document a worker reads for `from_file`: the refusal's message on error, as `services/git.rs` passes it.
fn read(bytes: &[u8]) -> Result<Document, String> {
    decode(bytes).map_err(|refusal| refusal.message().to_string())
}

#[test]
fn name_status_pairs_and_renames() {
    let text = "M\0src/hogwarts.rs\0A\0Gringotts/vault-713.ts\0R100\0lumos.md\0spells/lumos.md\0D\0azkaban.txt\0";
    let changes = parse_name_status(text, &HashMap::new());
    assert_eq!(
        pairs(&changes),
        [
            ("src/hogwarts.rs", ChangeKind::Modified),
            ("Gringotts/vault-713.ts", ChangeKind::Added),
            ("spells/lumos.md", ChangeKind::Renamed),
            ("azkaban.txt", ChangeKind::Deleted),
        ]
    );
    assert_eq!(changes[2].old_path.as_deref(), Some("lumos.md"));
    assert_eq!(changes[0].old_path, None);
}

#[test]
fn a_rename_takes_its_counts_from_the_new_path() {
    let counts = parse_numstat("3\t0\t\0spells/lumos.md\0spells/lumen.md\0");
    let changes = parse_name_status("R075\0spells/lumos.md\0spells/lumen.md\0", &counts);
    assert_eq!(changes[0].path, "spells/lumen.md");
    assert_eq!((changes[0].added, changes[0].removed), (Some(3), Some(0)));
}

#[test]
fn numstat_gives_counts_and_marks_binary_files() {
    let counts = parse_numstat("2\t1\tspells/lumos.md\0-\t-\tseal.bin\0");
    assert_eq!(counts["spells/lumos.md"], LineCounts { added: Some(2), removed: Some(1), binary: false });
    assert_eq!(counts["seal.bin"], LineCounts { added: None, removed: None, binary: true });
}

#[test]
fn numstat_of_a_rename_is_keyed_by_the_new_path() {
    let counts = parse_numstat("0\t0\t\0spells/lumos.md\0spells/lumen.md\0");
    assert_eq!(counts.len(), 1);
    assert!(counts.contains_key("spells/lumen.md"));
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
fn the_list_sorts_by_utf16_units_as_scouter_does() {
    // U+1F600 is a surrogate pair starting at 0xD83D, which sorts before U+E000 in UTF-16 but after it in UTF-8.
    let mut changes = vec![change("\u{e000}", ChangeKind::Modified), change("\u{1f600}", ChangeKind::Modified)];
    order(&mut changes);
    assert_eq!(changes[0].path, "\u{1f600}");
}

#[test]
fn the_change_list_marks_staged_and_unstaged_and_caps_the_list() {
    let set = change_set("M\0zz.md\0A\0aa.md\0", "1\t2\tzz.md\0", "notes.txt\0", "aa.md\0", "zz.md\0", false);
    assert_eq!(
        pairs(&set.changes),
        [("aa.md", ChangeKind::Added), ("notes.txt", ChangeKind::Untracked), ("zz.md", ChangeKind::Modified)]
    );
    assert_eq!((set.changes[0].staged, set.changes[0].unstaged), (Some(true), Some(false)));
    assert_eq!((set.changes[2].added, set.changes[2].removed), (Some(1), Some(2)));
    assert_eq!((set.changes[1].staged, set.changes[1].unstaged), (None, None));
    assert!(!set.truncated);
    assert!(change_set("", "", "", "", "", true).truncated, "a cut output says so");
    let many: String = (0..1001).map(|n| format!("f{n:04}\0")).collect();
    let set = change_set("", "", &many, "", "", false);
    assert_eq!((set.changes.len(), set.truncated), (MAX_CHANGES, true));
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
    let mut changes: Vec<Change> = (0..1000).map(|n| change(&format!("aa/{n:04}.txt"), ChangeKind::Modified)).collect();
    changes.push(change("zz/inside.txt", ChangeKind::Added));
    let marks = Marks::new(Listing { within: "zz".into(), changes, ignored: vec![] });
    assert!(marks.mark("inside.txt", &DARK).is_none());
}

#[test]
fn a_text_diff_counts_its_hunks_and_lines() {
    let raw = "diff --git a/spells/lumos.md b/spells/lumos.md\n--- a/spells/lumos.md\n+++ b/spells/lumos.md\n@@ -1 +1,2 @@\n-Lumos\n+Lumos maxima\n+\tTabs\n";
    let patch = from_diff(&change("spells/lumos.md", ChangeKind::Modified), raw);
    assert_eq!(patch, Patch::Text { text: raw.replace('\t', "    "), hunks: 1, added: 2, removed: 1 });
}

#[test]
fn a_diff_without_hunks_says_why_it_is_empty() {
    let mode = "diff --git a/x b/x\nold mode 100644\nnew mode 100755\n";
    let notice = |message: &str| Patch::Notice(message.into());
    assert_eq!(
        from_diff(&change("x", ChangeKind::Modified), mode),
        notice("No line changes (permissions or file type only).")
    );
    let renamed = Change { old_path: Some("spells/lumos.md".into()), ..change("spells/lumen.md", ChangeKind::Renamed) };
    let pure = "diff --git a/spells/lumos.md b/spells/lumen.md\nsimilarity index 100%\n";
    assert_eq!(from_diff(&renamed, pure), notice("Renamed from spells/lumos.md with no content changes."));
    assert_eq!(
        from_diff(&change("x", ChangeKind::Conflict), ""),
        notice("Unmerged: resolve the conflict to see a diff.")
    );
}

#[test]
fn a_binary_diff_and_an_over_limit_diff_are_notices() {
    let seal = change("seal.bin", ChangeKind::Modified);
    let named = "diff --git a/seal.bin b/seal.bin\nBinary files a/seal.bin and b/seal.bin differ\n";
    assert_eq!(from_diff(&seal, named), binary());
    assert_eq!(from_diff(&seal, "Binary files  differ"), binary(), "the shortest line the pattern takes");
    assert_eq!(
        from_diff(&seal, "Binary files differ"),
        Patch::Notice("No line changes (permissions or file type only).".into()),
        "one space is not a binary line"
    );
    assert_eq!(from_diff(&seal, &"+".repeat(PATCH_LIMIT + 1)), too_large());
    assert_eq!(utf16_len("\u{1f600}"), 2, "limits count UTF-16 units");
}

#[test]
fn a_whole_tree_diff_splits_by_file() {
    let text = concat!(
        "diff --git a/spells/lumos.md b/spells/lumos.md\n",
        "--- a/spells/lumos.md\n",
        "+++ b/spells/lumos.md\n",
        "@@ -1 +1 @@\n",
        "-Lumos\n",
        "+Lumos maxima\n",
        "diff --git a/spells/old.md b/spells/new.md\n",
        "similarity index 100%\n",
        "rename from spells/old.md\n",
        "rename to spells/new.md\n",
        "diff --git a/azkaban.txt b/azkaban.txt\n",
        "deleted file mode 100644\n",
        "--- a/azkaban.txt\n",
        "+++ /dev/null\n",
    );
    let mut keys: Vec<String> = split(text, false).into_keys().collect();
    keys.sort_unstable();
    assert_eq!(keys, ["azkaban.txt", "spells/lumos.md", "spells/new.md"]);
    // A cut diff loses its last chunk, which may be partly read.
    assert_eq!(split(text, true).len(), 2);
    assert!(split("", true).is_empty());
}

#[test]
fn a_chunk_names_its_path_from_the_header_when_no_line_does() {
    let binary_file = "diff --git a/seal.bin b/seal.bin\nBinary files a/seal.bin and b/seal.bin differ\n";
    assert!(split(binary_file, false).contains_key("seal.bin"));
    let spaced = "diff --git a/my notes.md b/my notes.md\nsimilarity index 100%\n";
    assert!(split(spaced, false).contains_key("my notes.md"));
    let unnamed = "diff --git a/x b/y\nBinary files a/x and b/y differ\n";
    assert!(split(unnamed, false).is_empty(), "the two halves must name the same path");
    let utf16 = "diff --git a/Gringotts/\u{f1}.md b/Gringotts/\u{f1}.md\nsimilarity index 100%\n";
    assert!(split(utf16, false).contains_key("Gringotts/\u{f1}.md"));
    let tab = "--- a/vault.txt\n+++ b/vault.txt\t\n@@ -1 +1 @@\n";
    assert!(split(tab, false).contains_key("vault.txt"), "a tab that ends the line is not part of the path");
}

#[test]
fn an_untracked_file_is_one_hunk_of_additions() {
    let patch = from_file("Hermione.md", read(b"Granger\nHermione\n"));
    assert_eq!(
        patch,
        Patch::Text {
            text: "diff --git a/Hermione.md b/Hermione.md\nnew file mode 100644\n--- /dev/null\n+++ b/Hermione.md\n\
                   @@ -0,0 +1,2 @@\n+Granger\n+Hermione\n"
                .into(),
            hunks: 1,
            added: 2,
            removed: 0,
        }
    );
}

#[test]
fn an_untracked_file_that_cannot_be_read_is_a_notice() {
    let notice = |message: &str| Patch::Notice(message.into());
    assert_eq!(from_file("seal.bin", read(b"\0")), notice("New file: Binary files are not supported."));
    assert_eq!(
        from_file("note.txt", read(b"Katherine \xff")),
        notice("New file: Only UTF-8 text files are supported.")
    );
    assert_eq!(from_file("empty.md", read(b"")), notice("New empty file."));
    let missing = Err("ENOENT: no such file or directory, open 'vault.md'".to_string());
    assert_eq!(from_file("vault.md", missing), notice("New file: ENOENT: no such file or directory, open 'vault.md'"));
    let mut cut = read(b"Teabing\n").unwrap();
    cut.truncated = true;
    assert_eq!(from_file("long.md", Ok(cut)), notice("New file: File is too large (limit: 1 MiB)."));
    let line = "a".repeat(document::MAX_BYTES);
    assert_eq!(from_file("line.md", read(line.as_bytes())), too_large(), "the synthesized text is over the limit");
    assert_eq!(not_shown(), Patch::Notice("Too much to show at once. Press a to view this file alone.".into()));
}

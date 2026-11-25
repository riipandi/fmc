use std::path::{Path, PathBuf};

use super::*;

fn folder(path: &str) -> Entry {
    Entry::new(
        Path::new(path).parent().unwrap_or(Path::new("/")),
        Path::new(path).file_name().unwrap().to_str().unwrap(),
        Kind::Directory,
    )
}

fn file(folder: &str, name: &str) -> Entry {
    Entry::new(Path::new(folder), name, Kind::File)
}

fn sorted_names(names: &[&str]) -> Vec<String> {
    let mut entries: Vec<Entry> = names.iter().map(|name| file("/vault", name)).collect();
    sort_entries(&mut entries);
    entries.into_iter().map(|e| e.name).collect()
}

#[test]
fn sort_follows_intl_collator_order() {
    // Expected order from Bun's `Intl.Collator()` (en-US), the collator Scouter sorts with.
    let names = [
        "README.md",
        "app.log",
        ".gitignore",
        "assets",
        "node_modules",
        "notes",
        "scripts",
        "src",
        "target",
        "tests",
        "Zeta",
        "alpha",
        "a-b",
        "ab",
        "a_b",
        "a b",
        "Ärger",
        "Apple",
        "apple",
        "_hidden",
        "1file",
        "10file",
        "2file",
        "éclair",
        "eclair",
        "Desktop",
        "desktop.ini",
        "-x",
        "x.y",
        "x-y",
        "xy",
    ];
    assert_eq!(
        sorted_names(&names),
        [
            "_hidden",
            "-x",
            ".gitignore",
            "10file",
            "1file",
            "2file",
            "a b",
            "a_b",
            "a-b",
            "ab",
            "alpha",
            "app.log",
            "apple",
            "Apple",
            "Ärger",
            "assets",
            "Desktop",
            "desktop.ini",
            "eclair",
            "éclair",
            "node_modules",
            "notes",
            "README.md",
            "scripts",
            "src",
            "target",
            "tests",
            "x-y",
            "x.y",
            "xy",
            "Zeta",
        ]
    );
}

#[test]
fn folders_come_before_files() {
    let mut entries = vec![file("/vault", "Hogwarts.md"), folder("/vault/Hogwarts"), file("/vault", "Alpha.md")];
    sort_entries(&mut entries);
    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["Hogwarts", "Alpha.md", "Hogwarts.md"]);
}

#[test]
fn noise_list_is_fixed() {
    for name in [".git", ".hg", ".svn", ".DS_Store", ".AppleDouble", "Thumbs.db", "desktop.ini"] {
        assert!(is_noise(name), "{name}");
    }
    for name in [".gitignore", "Desktop", ".env", "git"] {
        assert!(!is_noise(name), "{name}");
    }
}

/// Lists `path` from a fake disk: `disk` maps each folder to its entries.
fn list(tree: &mut Tree, disk: &[(&str, Vec<Entry>)], path: &Path) {
    let entries = disk.iter().find(|(folder, _)| Path::new(folder) == path).map(|(_, e)| e.clone()).unwrap_or_default();
    assert!(tree.listed(path, entries));
}

/// Runs the reveal loop the way the app does, answering each `Need` with a listing from `disk`.
fn reveal_from_disk(tree: &mut Tree, disk: &[(&str, Vec<Entry>)], target: &Path) -> usize {
    let mut listings = 0;
    loop {
        match tree.reveal(target) {
            Reveal::Need(folder) => {
                list(tree, disk, &folder);
                listings += 1;
            }
            Reveal::Done => return listings,
            Reveal::Outside => panic!("{} is outside the tree", target.display()),
        }
    }
}

fn vault_disk() -> Vec<(&'static str, Vec<Entry>)> {
    vec![
        ("/vault", vec![folder("/vault/hogwarts"), file("/vault", "Notes.md")]),
        ("/vault/hogwarts", vec![folder("/vault/hogwarts/gryffindor")]),
        ("/vault/hogwarts/gryffindor", vec![file("/vault/hogwarts/gryffindor", "chamber13.ts")]),
    ]
}

#[test]
fn expand_asks_for_a_listing_first() {
    let mut tree = Tree::new(PathBuf::from("/vault"));
    assert_eq!(tree.expand(Path::new("/vault")), Expand::NeedsListing);
    assert_eq!(tree.expand(Path::new("/nowhere")), Expand::Missing);
    list(&mut tree, &vault_disk(), Path::new("/vault"));
    tree.collapse(Path::new("/vault"));
    assert_eq!(tree.expand(Path::new("/vault")), Expand::Ready);
    assert!(tree.is_expanded(Path::new("/vault")));
}

#[test]
fn rows_show_only_expanded_children() {
    let mut tree = Tree::new(PathBuf::from("/vault"));
    list(&mut tree, &vault_disk(), Path::new("/vault"));
    let names: Vec<(String, usize)> = tree.rows().iter().map(|row| (row.entry.name.clone(), row.depth)).collect();
    assert_eq!(names, [("vault".into(), 0), ("hogwarts".into(), 1), ("Notes.md".into(), 1)]);

    tree.collapse(Path::new("/vault/hogwarts"));
    assert_eq!(tree.rows().len(), 3);
    assert!(tree.rows()[1].folder && !tree.rows()[1].expanded);
}

#[test]
fn collapse_keeps_the_listing() {
    let mut tree = Tree::new(PathBuf::from("/vault"));
    list(&mut tree, &vault_disk(), Path::new("/vault"));
    list(&mut tree, &vault_disk(), Path::new("/vault/hogwarts"));
    tree.collapse(Path::new("/vault/hogwarts"));
    assert_eq!(tree.expand(Path::new("/vault/hogwarts")), Expand::Ready);
    assert_eq!(tree.rows().len(), 4);
}

#[test]
fn reveal_expands_every_folder_above_a_file_three_deep() {
    let disk = vault_disk();
    let mut tree = Tree::new(PathBuf::from("/vault"));
    let target = Path::new("/vault/hogwarts/gryffindor/chamber13.ts");

    assert_eq!(reveal_from_disk(&mut tree, &disk, target), 3);

    assert!(tree.is_expanded(Path::new("/vault")));
    assert!(tree.is_expanded(Path::new("/vault/hogwarts")));
    assert!(tree.is_expanded(Path::new("/vault/hogwarts/gryffindor")));
    assert!(tree.rows().iter().any(|row| row.entry.path == target));
}

#[test]
fn reveal_of_a_file_in_the_root_lists_only_the_root() {
    let disk = vault_disk();
    let mut tree = Tree::new(PathBuf::from("/vault"));
    assert_eq!(reveal_from_disk(&mut tree, &disk, Path::new("/vault/Notes.md")), 1);
    assert_eq!(tree.rows().len(), 3);
}

#[test]
fn reveal_outside_the_root_and_of_the_root_itself() {
    let mut tree = Tree::new(PathBuf::from("/vault"));
    assert_eq!(tree.reveal(Path::new("/elsewhere/a.md")), Reveal::Outside);
    assert_eq!(tree.reveal(Path::new("/vault")), Reveal::Done);
}

#[test]
fn ancestors_run_from_the_parent_up_to_the_root() {
    let disk = vault_disk();
    let mut tree = Tree::new(PathBuf::from("/vault"));
    reveal_from_disk(&mut tree, &disk, Path::new("/vault/hogwarts/gryffindor/chamber13.ts"));
    let ancestors = tree.ancestors(Path::new("/vault/hogwarts/gryffindor"));
    assert_eq!(ancestors, [PathBuf::from("/vault/hogwarts"), PathBuf::from("/vault")]);
}

#[test]
fn relisting_replaces_the_old_children() {
    let mut tree = Tree::new(PathBuf::from("/vault"));
    list(&mut tree, &vault_disk(), Path::new("/vault"));
    list(&mut tree, &vault_disk(), Path::new("/vault/hogwarts"));
    list(&mut tree, &[("/vault", vec![file("/vault", "Only.md")])], Path::new("/vault"));
    assert!(tree.entry(Path::new("/vault/hogwarts")).is_none());
    assert_eq!(tree.rows().len(), 2);
}

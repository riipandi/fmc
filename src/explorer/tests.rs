use std::path::{Path, PathBuf};

use super::*;
use crate::find::text::Snippet;

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
    assert_eq!(names, [("hogwarts".into(), 0), ("Notes.md".into(), 0)], "the root is not a row");

    list(&mut tree, &vault_disk(), Path::new("/vault/hogwarts"));
    let names: Vec<(String, usize)> = tree.rows().iter().map(|row| (row.entry.name.clone(), row.depth)).collect();
    assert_eq!(names, [("hogwarts".into(), 0), ("gryffindor".into(), 1), ("Notes.md".into(), 0)]);

    tree.collapse(Path::new("/vault/hogwarts"));
    assert_eq!(tree.rows().len(), 2);
    assert!(tree.rows()[0].folder && !tree.rows()[0].expanded);
}

#[test]
fn a_top_level_row_has_no_parent_row() {
    let disk = vault_disk();
    let mut tree = Tree::new(PathBuf::from("/vault"));
    reveal_from_disk(&mut tree, &disk, Path::new("/vault/hogwarts/gryffindor/chamber13.ts"));
    assert_eq!(tree.parent_row(Path::new("/vault/hogwarts")), None);
    assert_eq!(tree.parent_row(Path::new("/vault/Notes.md")), None);
    assert_eq!(tree.parent_row(Path::new("/vault/hogwarts/gryffindor")), Some(PathBuf::from("/vault/hogwarts")));
}

#[test]
fn collapse_keeps_the_listing() {
    let mut tree = Tree::new(PathBuf::from("/vault"));
    list(&mut tree, &vault_disk(), Path::new("/vault"));
    list(&mut tree, &vault_disk(), Path::new("/vault/hogwarts"));
    tree.collapse(Path::new("/vault/hogwarts"));
    assert_eq!(tree.expand(Path::new("/vault/hogwarts")), Expand::Ready);
    assert_eq!(tree.rows().len(), 3);
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
    assert_eq!(tree.rows().len(), 2);
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
    assert_eq!(tree.rows().len(), 1);
}

/// A browser over a root with `files` children, named `f00.md`, `f01.md`, ..., and a viewport of `height` rows.
/// The first file is selected, since the root has no row.
fn browser(files: usize, height: usize) -> Browser {
    let root = PathBuf::from("/vault");
    let entries = (0..files).map(|n| Entry::new(&root, &format!("f{n:02}.md"), Kind::File)).collect();
    let mut browser = Browser::new(Tree::new(root.clone()));
    browser.listed(&root, entries);
    browser.set_height(height);
    browser
}

fn selected_name(browser: &Browser) -> String {
    browser.selected().file_name().unwrap().to_string_lossy().into_owned()
}

#[test]
fn moving_past_the_bottom_scrolls_from_the_nearest_edge() {
    let mut browser = browser(10, 5);
    browser.move_by(3);
    assert_eq!((browser.selected_index(), browser.top()), (3, 0));
    browser.move_by(1);
    assert_eq!((browser.selected_index(), browser.top()), (4, 0));
    browser.move_by(1);
    assert_eq!((browser.selected_index(), browser.top()), (5, 1));
    browser.move_to_edge(true);
    assert_eq!((browser.selected_index(), browser.top()), (9, 5));
    browser.move_by(-20);
    assert_eq!((browser.selected_index(), browser.top()), (0, 0));
}

#[test]
fn moves_clamp_at_both_ends() {
    let mut browser = browser(2, 10);
    browser.move_by(-1);
    assert_eq!(selected_name(&browser), "f00.md");
    browser.move_by(99);
    assert_eq!(selected_name(&browser), "f01.md");
}

#[test]
fn a_page_is_one_row_less_than_the_viewport() {
    let mut browser = browser(30, 6);
    browser.page(true);
    assert_eq!(browser.selected_index(), 5);
    browser.page(true);
    assert_eq!(browser.selected_index(), 10);
    browser.page(false);
    assert_eq!(browser.selected_index(), 5);
}

#[test]
fn the_wheel_scrolls_without_moving_the_selection() {
    let mut browser = browser(10, 4);
    browser.scroll(3);
    assert_eq!((browser.top(), browser.selected_index()), (3, 0));
    browser.scroll(100);
    assert_eq!(browser.top(), 6, "rows minus the viewport: the last full page");
    browser.scroll(-100);
    assert_eq!(browser.top(), 0);
}

#[test]
fn a_repaint_at_the_same_height_keeps_the_wheel_scroll() {
    let mut browser = browser(10, 4);
    browser.scroll(3);
    browser.set_height(4);
    assert_eq!((browser.top(), browser.selected_index()), (3, 0));
}

#[test]
fn the_first_layout_brings_the_selection_into_view() {
    let root = PathBuf::from("/vault");
    let entries = (0..10).map(|n| Entry::new(&root, &format!("f{n:02}.md"), Kind::File)).collect();
    let mut browser = Browser::new(Tree::new(root.clone()));
    browser.listed(&root, entries);
    browser.select_index(8);
    browser.set_height(4);
    assert_eq!(browser.top(), 5);
}

#[test]
fn a_click_maps_a_line_to_its_row_through_the_scroll_position() {
    let mut browser = browser(10, 4);
    browser.scroll(2);
    assert_eq!(browser.row_at(0), Some(2));
    assert_eq!(browser.row_at(3), Some(5));
    assert_eq!(browser.row_at(4), None, "below the viewport");
    browser.select_index(5);
    assert_eq!(selected_name(&browser), "f05.md");
}

#[test]
fn a_click_past_the_last_row_selects_nothing() {
    let browser = browser(1, 6);
    assert_eq!(browser.row_at(0), Some(0));
    assert_eq!(browser.row_at(1), None);
}

#[test]
fn selecting_a_missing_path_falls_back_to_the_first_row() {
    let mut browser = browser(3, 5);
    browser.move_by(2);
    browser.select(Path::new("/vault/gone.md"));
    assert_eq!(selected_name(&browser), "f00.md");
    assert!(!browser.select_row(Path::new("/vault/gone.md")));
    assert!(browser.select_row(Path::new("/vault/f01.md")));
    assert_eq!(selected_name(&browser), "f01.md");
}

#[test]
fn relative_paths_use_slashes_and_keep_the_root_out() {
    assert_eq!(relative(Path::new("/vault"), Path::new("/vault/hogwarts/notes.md")), "hogwarts/notes.md");
    assert_eq!(relative(Path::new("/vault"), Path::new("/vault")), "");
}

#[test]
fn reflow_clamps_after_rows_disappear() {
    let mut browser = browser(10, 4);
    browser.move_to_edge(true);
    browser.listed(Path::new("/vault"), vec![Entry::new(Path::new("/vault"), "only.md", Kind::File)]);
    browser.reflow();
    assert_eq!(browser.top(), 0);
    assert_eq!(browser.visible_rows().len(), 1);
    assert_eq!(selected_name(&browser), "only.md", "a selection that lost its row moves to the first");
}

#[test]
fn an_empty_folder_takes_no_move_and_no_panic() {
    let mut browser = browser(0, 5);
    browser.move_by(1);
    browser.move_by(-1);
    browser.move_to_edge(true);
    assert_eq!(browser.row_count(), 0);
    assert!(!browser.is_flat());
}

#[test]
fn flat_rows_replace_the_tree_and_the_tree_comes_back_with_its_selection() {
    let mut browser = browser(4, 5);
    browser.move_by(2);
    let root = PathBuf::from("/vault");
    let hits = vec![
        Flat::name(Entry::new(&root, "notes", Kind::Directory)),
        Flat::name(Entry::new(&root, "ledger.md", Kind::File)),
    ];
    browser.show(hits, Path::new("/vault/notes"));
    assert!(browser.is_flat());
    assert_eq!(browser.row_count(), 2);
    assert_eq!(browser.selected(), Path::new("/vault/notes"));
    assert!(browser.visible_rows().iter().all(|row| row.depth == 0 && !row.expanded));
    assert!(browser.visible_rows()[0].folder);
    assert!(!browser.visible_rows()[1].folder);

    browser.move_by(1);
    assert_eq!(browser.selected(), Path::new("/vault/ledger.md"));
    browser.move_by(5);
    assert_eq!(browser.selected(), Path::new("/vault/ledger.md"));

    browser.show_tree(Path::new("/vault/f02.md"));
    assert!(!browser.is_flat());
    assert_eq!(browser.row_count(), 4);
    assert_eq!(selected_name(&browser), "f02.md");
}

#[test]
fn a_flat_list_without_a_match_selects_nothing_and_a_missing_path_takes_the_first_row() {
    let mut browser = browser(3, 5);
    browser.show(Vec::new(), Path::new("/vault/f01.md"));
    assert_eq!(browser.row_count(), 0);
    assert!(browser.selected_entry().is_none());
    browser.show(vec![Flat::name(Entry::new(Path::new("/vault"), "f02.md", Kind::File))], Path::new("/vault/gone.md"));
    assert_eq!(selected_name(&browser), "f02.md");
    assert_eq!(browser.selected_entry().map(|entry| entry.name), Some("f02.md".into()));
}

#[test]
fn a_listing_that_lands_while_the_flat_list_shows_keeps_its_selection() {
    let mut browser = browser(0, 5);
    browser.show(
        vec![Flat::name(Entry::new(Path::new("/vault"), "ledger.md", Kind::File))],
        Path::new("/vault/ledger.md"),
    );
    let root = PathBuf::from("/vault");
    browser.listed(&root, vec![Entry::new(&root, "a.md", Kind::File)]);
    assert_eq!(selected_name(&browser), "ledger.md");
}

#[test]
fn a_text_hit_list_selects_by_row_when_one_file_has_several_hits() {
    let mut browser = browser(0, 5);
    let a = Path::new("/vault/a.txt");
    let snippet = |text: &str| Snippet { before: String::new(), matched: text.into(), after: String::new() };
    let rows = vec![
        Flat {
            entry: Entry::new(Path::new("/vault"), "a.txt", Kind::File),
            hit: Some((3, snippet("one"))),
            matched: Vec::new(),
        },
        Flat {
            entry: Entry::new(Path::new("/vault"), "a.txt", Kind::File),
            hit: Some((9, snippet("two"))),
            matched: Vec::new(),
        },
        Flat {
            entry: Entry::new(Path::new("/vault"), "b.txt", Kind::File),
            hit: Some((1, snippet("three"))),
            matched: Vec::new(),
        },
    ];
    browser.show(rows, a);
    assert_eq!(browser.selected_index(), 0);
    browser.move_by(1);
    assert_eq!(browser.selected_index(), 1);
    assert_eq!(browser.selected_flat().and_then(|flat| flat.hit.map(|(line, _)| line)), Some(9));
    browser.move_by(1);
    assert_eq!(browser.selected(), Path::new("/vault/b.txt"));
    browser.move_by(-2);
    assert_eq!(browser.selected_index(), 0);
}

#[test]
fn a_refresh_keeps_a_surviving_child_open_and_forgets_a_gone_one() {
    let root = Path::new("/hogwarts");
    let mut tree = Tree::new(root.to_path_buf());
    let chamber = Entry::new(root, "chamber", Kind::Directory);
    let notes = Entry::new(root, "notes.md", Kind::File);
    tree.listed(root, vec![chamber.clone(), notes.clone()]);
    tree.listed(&chamber.path, vec![Entry::new(&chamber.path, "key.rs", Kind::File)]);
    let owl = Entry::new(root, "owl.txt", Kind::File);
    assert!(tree.refreshed(root, vec![chamber.clone(), owl.clone()]), "a file came and went");
    assert!(tree.is_expanded(&chamber.path) && tree.is_listed(&chamber.path), "a surviving folder keeps its children");
    assert!(tree.entry(&notes.path).is_none(), "a file that is gone leaves the tree");
    assert!(tree.entry(&owl.path).is_some());
    assert!(!tree.refreshed(root, vec![chamber, owl]), "the same listing changes no row");
    assert!(!tree.refreshed(Path::new("/azkaban"), Vec::new()), "a folder not in the tree is left alone");
}

/// A root with `.gitignore`, `Notes.md`, the folder `hogwarts`, and the dot folder `.owls` (expanded, with `owl.md`).
fn with_dotfiles() -> Tree {
    let root = PathBuf::from("/vault");
    let owls = root.join(".owls");
    let mut tree = Tree::new(root.clone());
    tree.listed(
        &root,
        vec![
            Entry::new(&root, ".gitignore", Kind::File),
            Entry::new(&root, ".owls", Kind::Directory),
            Entry::new(&root, "hogwarts", Kind::Directory),
            Entry::new(&root, "Notes.md", Kind::File),
        ],
    );
    tree.listed(&owls, vec![Entry::new(&owls, "owl.md", Kind::File)]);
    assert_eq!(tree.expand(&owls), Expand::Ready);
    tree
}

fn row_names(tree: &Tree) -> Vec<String> {
    tree.rows().iter().map(|row| row.entry.name.clone()).collect()
}

#[test]
fn a_dotfile_and_a_dot_folder_with_its_children_have_no_rows_while_hidden() {
    let mut tree = with_dotfiles();
    assert!(tree.dotfiles());
    assert!(row_names(&tree).contains(&".gitignore".to_string()));
    assert!(row_names(&tree).contains(&"owl.md".to_string()));

    tree.set_dotfiles(false);
    assert_eq!(row_names(&tree), ["hogwarts", "Notes.md"]);

    tree.set_dotfiles(true);
    assert!(row_names(&tree).contains(&"owl.md".to_string()), "the listing kept the dot folder's children");
}

#[test]
fn hiding_dotfiles_moves_a_selected_dotfile_to_the_first_row() {
    let root = PathBuf::from("/vault");
    let mut browser = Browser::new(Tree::new(root.clone()));
    browser.listed(&root, vec![Entry::new(&root, "Notes.md", Kind::File), Entry::new(&root, ".gitignore", Kind::File)]);
    browser.select(Path::new("/vault/.gitignore"));
    assert_eq!(selected_name(&browser), ".gitignore");

    browser.set_dotfiles(false);
    assert_eq!(selected_name(&browser), "Notes.md", "the selection falls back as for a removed row");
    assert!(!browser.dotfiles());
}

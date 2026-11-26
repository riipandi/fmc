use std::path::PathBuf;

use super::*;
use crate::explorer::Kind;

fn file(relative: &str) -> Indexed {
    Indexed::new(entry(relative, Kind::File))
}

fn entry(relative: &str, kind: Kind) -> Entry {
    Entry { name: relative.to_string(), path: PathBuf::from("/root").join(relative), kind }
}

fn names<'a>(matches: &Matches<'a>) -> Vec<&'a str> {
    matches.entries.iter().map(|&found| found.name.as_str()).collect()
}

#[test]
fn a_path_whose_name_holds_the_word_comes_first_then_the_shorter_path() {
    let index: Vec<Indexed> = ["docs/guide-to-search.md", "src/Search.ts", "src/search/index.ts", "src/other.ts"]
        .into_iter()
        .map(file)
        .collect();
    let all = matches(&index, "SEARCH");
    assert_eq!(all.total, 3);
    assert_eq!(names(&all), ["src/Search.ts", "docs/guide-to-search.md", "src/search/index.ts"]);
    assert_eq!(names(&matches(&index, "search ts")), ["src/Search.ts", "src/search/index.ts"]);
    assert_eq!(matches(&index, "   "), Matches { entries: Vec::new(), total: 0 });
}

#[test]
fn a_name_match_outranks_a_shorter_path_match() {
    let index = vec![file("a/b/c/search.rs"), file("search/x.rs")];
    assert_eq!(names(&matches(&index, "search")), ["a/b/c/search.rs", "search/x.rs"]);
}

#[test]
fn equal_lengths_compare_in_utf16_code_units() {
    // U+1F600 is a surrogate pair (0xD83D) and sorts before U+FF5E (0xFF5E) in UTF-16, though not in code points.
    let index = vec![file("a\u{FF5E}\u{FF5E}"), file("a\u{1F600}")];
    assert_eq!(names(&matches(&index, "a")), ["a\u{1F600}", "a\u{FF5E}\u{FF5E}"]);
}

#[test]
fn length_is_counted_in_utf16_units_not_characters() {
    // "a😀" has two characters but three UTF-16 units, the same as "abc": the tie goes to code units, where the
    // surrogate 0xD83D sorts after 'b', so "abc" comes first.
    let index = vec![file("a\u{1F600}"), file("abc")];
    assert_eq!(names(&matches(&index, "a")), ["abc", "a\u{1F600}"]);
}

#[test]
fn the_name_group_holds_at_most_the_limit_and_the_total_counts_every_match() {
    let index: Vec<Indexed> = (0..250).map(|n| file(&format!("notes/search-{n:03}.md"))).collect();
    let found = matches(&index, "search");
    assert_eq!(found.total, 250);
    assert_eq!(found.entries.len(), MAX_MATCHES);
    assert_eq!(found.entries[0].name, "notes/search-000.md");
    assert_eq!(found.entries[199].name, "notes/search-199.md");
}

#[test]
fn the_path_group_fills_the_rest_of_the_limit() {
    let mut index: Vec<Indexed> = (0..150).map(|n| file(&format!("hits/one-{n:03}.md"))).collect();
    index.extend((0..100).map(|n| file(&format!("deep/search-{n:03}.md"))));
    let found = matches(&index, "search");
    assert_eq!(found.total, 100);
    assert_eq!(found.entries.len(), 100);

    let index: Vec<Indexed> = (0..120)
        .map(|n| file(&format!("alpha/{n:03}-search.md")))
        .chain((0..120).map(|n| file(&format!("search/{n:03}.md"))))
        .collect();
    let found = matches(&index, "search");
    assert_eq!(found.total, 240);
    assert_eq!(found.entries.len(), MAX_MATCHES);
    assert_eq!(found.entries[119].name, "alpha/119-search.md");
    assert_eq!(found.entries[120].name, "search/000.md");
}

#[test]
fn folders_come_first_and_names_sort_by_code_unit() {
    let mut entries = vec![
        entry("alpha.md", Kind::File),
        entry("Zeta", Kind::Directory),
        entry("beta", Kind::Directory),
        entry("Zulu.md", Kind::File),
    ];
    sort_listing(&mut entries);
    let order: Vec<&str> = entries.iter().map(|found| found.name.as_str()).collect();
    assert_eq!(order, ["Zeta", "beta", "Zulu.md", "alpha.md"]);
}

#[test]
fn the_footer_count_groups_thousands_and_says_when_the_index_stopped() {
    assert_eq!(grouped(0), "0");
    assert_eq!(grouped(999), "999");
    assert_eq!(grouped(1_000), "1,000");
    assert_eq!(grouped(1_234_567), "1,234,567");
    assert_eq!(names_status(0, false), "no matches");
    assert_eq!(names_status(0, true), "no matches, first 20,000 files");
    assert_eq!(names_status(1, false), "1 match");
    assert_eq!(names_status(1_204, false), "1,204 matches");
    assert_eq!(names_status(3, true), "3 matches, first 20,000 files");
}

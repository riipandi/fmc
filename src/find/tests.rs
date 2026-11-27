use std::path::PathBuf;

use neo_frizbee::{CaseMatching, Config, Matcher};

use super::*;
use crate::explorer::Kind;

fn file(relative: &str) -> Indexed {
    Indexed::new(entry(relative, Kind::File))
}

fn entry(relative: &str, kind: Kind) -> Entry {
    Entry { name: relative.to_string(), path: PathBuf::from("/root").join(relative), kind }
}

fn names<'a>(matches: &Matches<'a>) -> Vec<&'a str> {
    matches.entries.iter().map(|found| found.entry.name.as_str()).collect()
}

fn substring<'a>(index: &'a [Indexed], query: &str) -> Matches<'a> {
    matches(index, query, Matching::Substring)
}

fn fuzzy<'a>(index: &'a [Indexed], query: &str) -> Matches<'a> {
    matches(index, query, Matching::Fuzzy)
}

#[test]
fn a_path_whose_name_holds_the_word_comes_first_then_the_shorter_path() {
    let index: Vec<Indexed> = ["docs/guide-to-search.md", "src/Search.ts", "src/search/index.ts", "src/other.ts"]
        .into_iter()
        .map(file)
        .collect();
    let all = substring(&index, "SEARCH");
    assert_eq!(all.total, 3);
    assert_eq!(names(&all), ["src/Search.ts", "docs/guide-to-search.md", "src/search/index.ts"]);
    assert_eq!(names(&substring(&index, "search ts")), ["src/Search.ts", "src/search/index.ts"]);
    assert_eq!(substring(&index, "   "), Matches { entries: Vec::new(), total: 0 });
}

#[test]
fn a_name_match_outranks_a_shorter_path_match() {
    let index = vec![file("a/b/c/search.rs"), file("search/x.rs")];
    assert_eq!(names(&substring(&index, "search")), ["a/b/c/search.rs", "search/x.rs"]);
}

#[test]
fn equal_lengths_compare_in_utf16_code_units() {
    // U+1F600 is a surrogate pair (0xD83D) and sorts before U+FF5E (0xFF5E) in UTF-16, though not in code points.
    let index = vec![file("a\u{FF5E}\u{FF5E}"), file("a\u{1F600}")];
    assert_eq!(names(&substring(&index, "a")), ["a\u{1F600}", "a\u{FF5E}\u{FF5E}"]);
}

#[test]
fn length_is_counted_in_utf16_units_not_characters() {
    // "a😀" has two characters but three UTF-16 units, the same as "abc": the tie goes to code units, where the
    // surrogate 0xD83D sorts after 'b', so "abc" comes first.
    let index = vec![file("a\u{1F600}"), file("abc")];
    assert_eq!(names(&substring(&index, "a")), ["abc", "a\u{1F600}"]);
}

#[test]
fn the_name_group_holds_at_most_the_limit_and_the_total_counts_every_match() {
    let index: Vec<Indexed> = (0..250).map(|n| file(&format!("notes/search-{n:03}.md"))).collect();
    let found = substring(&index, "search");
    assert_eq!(found.total, 250);
    assert_eq!(found.entries.len(), MAX_MATCHES);
    assert_eq!(found.entries[0].entry.name, "notes/search-000.md");
    assert_eq!(found.entries[199].entry.name, "notes/search-199.md");
}

#[test]
fn the_path_group_fills_the_rest_of_the_limit() {
    let mut index: Vec<Indexed> = (0..150).map(|n| file(&format!("hits/one-{n:03}.md"))).collect();
    index.extend((0..100).map(|n| file(&format!("deep/search-{n:03}.md"))));
    let found = substring(&index, "search");
    assert_eq!(found.total, 100);
    assert_eq!(found.entries.len(), 100);

    let index: Vec<Indexed> = (0..120)
        .map(|n| file(&format!("alpha/{n:03}-search.md")))
        .chain((0..120).map(|n| file(&format!("search/{n:03}.md"))))
        .collect();
    let found = substring(&index, "search");
    assert_eq!(found.total, 240);
    assert_eq!(found.entries.len(), MAX_MATCHES);
    assert_eq!(found.entries[119].entry.name, "alpha/119-search.md");
    assert_eq!(found.entries[120].entry.name, "search/000.md");
}

#[test]
fn a_substring_match_marks_no_characters() {
    let index = vec![file("hogwarts/owls.md")];
    assert!(substring(&index, "owl").entries.iter().all(|found| found.matched.is_empty()));
}

#[test]
fn the_mode_names_are_the_setting_values() {
    assert_eq!(Matching::named("substring"), Some(Matching::Substring));
    assert_eq!(Matching::named("fuzzy"), Some(Matching::Fuzzy));
    assert_eq!(Matching::named("Fuzzy"), None);
    assert_eq!(Matching::named(""), None);
    assert_eq!(Matching::default(), Matching::Substring);
    assert_eq!(Matching::Substring.flipped(), Matching::Fuzzy);
    assert_eq!(Matching::Fuzzy.flipped().name(), "substring");
}

#[test]
fn a_fuzzy_needle_matches_a_subsequence_the_substring_match_refuses() {
    let index = vec![file("hogwarts/owls.md"), file("gringotts/vault.md"), file("diagon/alley.txt")];
    assert_eq!(names(&fuzzy(&index, "hgw")), ["hogwarts/owls.md"]);
    assert_eq!(substring(&index, "hgw").total, 0);
}

#[test]
fn fuzzy_positions_are_the_character_positions_of_the_match() {
    let index = vec![file("hogwarts/owls.md")];
    assert_eq!(fuzzy(&index, "hgw").entries[0].matched, [0, 2, 3]);
    // "é" is two bytes, and the matcher reports both; the position is the character's, so 'o' is at 5, not 6.
    let index = vec![file("caf\u{E9}/owls.md")];
    assert_eq!(fuzzy(&index, "fo").entries[0].matched, [2, 5]);
}

#[test]
fn fuzzy_ranks_by_score_then_by_scouters_order_on_ties() {
    let index = vec![file("b/owl.md"), file("a/owl.md"), file("hogwarts/owls-and-other-owls.md"), file("owl.md")];
    let found = fuzzy(&index, "owl");
    let config = Config::default().casing(CaseMatching::Ignore);
    let mut matcher = Matcher::new("owl", &config);
    let key: Vec<(u16, usize)> = found
        .entries
        .iter()
        .map(|found| {
            let score = matcher.match_one(found.entry.name.as_str(), 0).map_or(0, |hit| hit.score);
            (score, found.entry.name.encode_utf16().count())
        })
        .collect();
    assert!(key.windows(2).all(|pair| pair[0].0 > pair[1].0 || (pair[0].0 == pair[1].0 && pair[0].1 <= pair[1].1)));
    // Equal scores and lengths tie, and the code-unit order decides.
    let tied: Vec<&str> = names(&found).into_iter().filter(|name| name.ends_with("/owl.md")).collect();
    assert_eq!(tied, ["a/owl.md", "b/owl.md"]);
}

#[test]
fn a_fuzzy_needle_keeps_its_spaces_and_ignores_case() {
    let index = vec![file("Hogwarts/Owl Post.md"), file("hogwarts/owlpost.md")];
    assert_eq!(names(&fuzzy(&index, "OWL post")), ["Hogwarts/Owl Post.md"]);
    assert_eq!(fuzzy(&index, "").total, 0);
}

#[test]
fn a_fuzzy_match_shows_the_limit_and_counts_every_match() {
    let index: Vec<Indexed> = (0..250).map(|n| file(&format!("notes/search-{n:03}.md"))).collect();
    let found = fuzzy(&index, "search");
    assert_eq!(found.total, 250);
    assert_eq!(found.entries.len(), MAX_MATCHES);
    assert_eq!(found.entries[0].entry.name, "notes/search-000.md");
}

#[test]
fn the_folders_come_first_and_names_sort_by_code_unit() {
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
    assert_eq!(names_status(0, false, Matching::Substring), "no matches");
    assert_eq!(names_status(0, true, Matching::Substring), "no matches, first 20,000 files");
    assert_eq!(names_status(1, false, Matching::Substring), "1 match");
    assert_eq!(names_status(1_204, false, Matching::Substring), "1,204 matches");
    assert_eq!(names_status(3, true, Matching::Substring), "3 matches, first 20,000 files");
}

#[test]
fn the_footer_names_fuzzy_when_it_is_on() {
    assert_eq!(names_status(1, false, Matching::Fuzzy), "fuzzy, 1 match");
    assert_eq!(names_status(0, false, Matching::Fuzzy), "fuzzy, no matches");
    assert_eq!(names_status(3, true, Matching::Fuzzy), "fuzzy, 3 matches, first 20,000 files");
}

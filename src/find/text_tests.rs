use std::path::Path;

use super::*;

fn hits(text: &str, needle: &str, limit: usize) -> (Vec<Hit>, bool) {
    hits_in(Path::new("/vault/ledger.txt"), "ledger.txt", text, needle, limit)
}

#[test]
fn one_hit_per_line_ignoring_case_with_its_line_number() {
    let text = "Hermione\nAUDIT trail\nno match here\naudit again, audit twice\n";
    let (found, more) = hits(text, "audit", 10);
    assert!(!more);
    let lines: Vec<usize> = found.iter().map(|hit| hit.line).collect();
    assert_eq!(lines, [2, 4]);
    assert_eq!(found[0].snippet, Snippet { before: "".into(), matched: "AUDIT".into(), after: " trail".into() });
    assert_eq!(found[1].snippet.matched, "audit");
    assert_eq!(found[1].relative, "ledger.txt");
}

#[test]
fn a_full_limit_reports_that_more_lines_matched() {
    let text = "audit\naudit\naudit\n";
    let (found, more) = hits(text, "audit", 2);
    assert_eq!(found.len(), 2);
    assert!(more);
    let (found, more) = hits(text, "audit", 3);
    assert_eq!(found.len(), 3);
    assert!(!more);
}

#[test]
fn a_snippet_keeps_forty_units_before_the_match_and_three_hundred_in_all() {
    let line = format!("{}needle{}", "x".repeat(60), "y".repeat(400));
    let (found, _) = hits(&line, "needle", 1);
    let snippet = &found[0].snippet;
    assert_eq!(snippet.before, "x".repeat(40));
    assert_eq!(snippet.matched, "needle");
    assert_eq!(snippet.after.len(), 300 - 40 - 6);
}

#[test]
fn a_match_near_the_start_of_an_indented_line_starts_at_its_text() {
    let (found, _) = hits("    return audit;", "audit", 1);
    // Scouter starts the snippet at the first non-blank column when the match is near the start (`snippetOf`).
    assert_eq!(found[0].snippet.before, "return ");
    assert_eq!(found[0].snippet.matched, "audit");
    assert_eq!(found[0].snippet.after, ";");
}

#[test]
fn units_are_utf16_and_a_letter_that_grows_when_lowercased_maps_back() {
    // "İ" lowercases to two characters, and the emoji is two UTF-16 units: the snippet must still cut at the match.
    let (found, _) = hits("İstanbul 😀 audit", "audit", 1);
    assert_eq!(found[0].snippet.before, "İstanbul 😀 ");
    assert_eq!(found[0].snippet.matched, "audit");
}

#[test]
fn a_scan_counts_a_file_once_and_stops_at_the_hit_limit() {
    let mut scan = Scan::new("Audit", 3);
    let text = "audit\n".repeat(12);
    scan.file(Path::new("/v/a.txt"), "a.txt", &text);
    let progress = scan.progress(false);
    assert_eq!(progress.hits.len(), MAX_HITS_PER_FILE, "ten hits from one file");
    assert!(progress.truncated);
    assert_eq!(progress.files, 1);

    let mut scan = Scan::new("audit", 30);
    for n in 0..25 {
        scan.file(Path::new(&format!("/v/{n}.txt")), &format!("{n}.txt"), &text);
        if scan.stopped() {
            break;
        }
    }
    assert!(scan.stopped());
    let progress = scan.progress(true);
    assert_eq!(progress.hits.len(), MAX_TEXT_HITS);
    assert_eq!(progress.scanned, 20);
    assert_eq!(text_status(&progress), "first 200 matches");
}

#[test]
fn a_file_after_the_stop_counts_as_scanned_but_is_not_read() {
    // Each file adds at most ten hits, so twenty files reach the limit.
    let mut scan = Scan::new("audit", 21);
    let text = "audit\n".repeat(12);
    for n in 0..20 {
        scan.file(Path::new(&format!("/v/{n}.txt")), &format!("{n}.txt"), &text);
    }
    assert!(scan.stopped());
    scan.file(Path::new("/v/b.txt"), "b.txt", "audit\n");
    let progress = scan.progress(true);
    assert_eq!(progress.scanned, 21);
    assert_eq!(progress.hits.len(), MAX_TEXT_HITS);
    assert_eq!(progress.files, 20);
}

#[test]
fn the_footer_says_what_the_scan_is_doing() {
    let mut progress = Progress { total: 7, scanned: 3, ..Progress::default() };
    assert_eq!(text_status(&progress), "searching 3/7");
    progress.done = true;
    assert_eq!(text_status(&progress), "no matches");
    progress.hits = vec![hit_at(1)];
    assert_eq!(text_status(&progress), "1 match");
    progress.hits = vec![hit_at(1), hit_at(2)];
    progress.truncated = true;
    assert_eq!(text_status(&progress), "2 matches+");
    progress.hits = (0..MAX_TEXT_HITS).map(hit_at).collect();
    assert_eq!(text_status(&progress), "first 200 matches");
    progress.total = 1_204;
    progress.scanned = 1_000;
    progress.done = false;
    assert_eq!(text_status(&progress), "searching 1,000/1,204");
}

fn hit_at(line: usize) -> Hit {
    Hit { path: Path::new("/v/a.txt").to_path_buf(), relative: "a.txt".into(), line, snippet: Snippet::default() }
}

#[test]
fn an_empty_query_finds_nothing() {
    assert_eq!(hits("anything", "", 5), (Vec::new(), false));
}

#[test]
fn a_query_needs_two_utf16_units_to_search() {
    assert!(!wants_scan(""));
    assert!(!wants_scan("a"));
    assert!(!wants_scan("é"));
    assert!(wants_scan("ab"));
    // One astral character is two UTF-16 units, the length Scouter counts.
    assert!(wants_scan("😀"));
    assert_eq!((MIN_QUERY, SCAN_DELAY_MS), (2, 150));
}

#[test]
fn only_the_latest_generation_is_current() {
    let mut generation = Generation::default();
    let first = generation.bump();
    assert!(generation.is_current(first));
    let second = generation.bump();
    assert!(!generation.is_current(first));
    assert!(generation.is_current(second));
}

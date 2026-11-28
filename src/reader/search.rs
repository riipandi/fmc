//! In-file search (Scouter `src/reader/document-search.ts`, `src/reader/match-style.ts`): a query matches the whole lines
//! that contain it, ignoring case; `n` and `N` step through those lines and wrap; every occurrence is marked, up to
//! 2000. Lines are zero-based.

use std::ops::Range;

use super::rendered_search::Hit;
use crate::line_edit::Line;

/// Scouter marks at most this many occurrences in a file (`src/reader/match-style.ts:11`).
pub const MAX_MARKS: usize = 2000;

/// The search in progress: the query and its caret, the hits it has in the view, the hit the view is on, and whether
/// the prompt is open. The code view's hits are lines; the rendered view's are occurrences (`marks`, `rendered`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Search {
    pub query: Line,
    pub lines: Vec<usize>,
    pub marks: Vec<Hit>,
    /// The search runs in the rendered Markdown view, so `marks` holds its hits.
    pub rendered: bool,
    pub at: usize,
    /// The prompt is open: typed keys edit the query.
    pub editing: bool,
}

impl Search {
    /// A search that opens its prompt with an empty query (Scouter `start`).
    pub fn start() -> Self {
        Self { editing: true, ..Self::default() }
    }

    /// A search in the rendered view, which opens its prompt with an empty query.
    pub fn start_rendered() -> Self {
        Self { editing: true, rendered: true, ..Self::default() }
    }

    /// The number of hits in the view the search runs in.
    pub fn hit_count(&self) -> usize {
        if self.rendered { self.marks.len() } else { self.lines.len() }
    }

    /// The line of the hit the view is on, if the query hits anything in the code view.
    pub fn current_line(&self) -> Option<usize> {
        if self.rendered { None } else { self.lines.get(self.at).copied() }
    }

    /// The occurrence the rendered view is on, if the query hits anything there.
    pub fn current_mark(&self) -> Option<&Hit> {
        if self.rendered { self.marks.get(self.at) } else { None }
    }

    /// Replaces the rendered hits for the edited query, and moves to the first hit at or below `top`, as `retype`
    /// does; `row` is a hit's row in the document. Returns the hit the view shows.
    pub fn retype_marks(&mut self, marks: Vec<Hit>, top: u32, row: impl Fn(&Hit) -> u32) -> Option<&Hit> {
        self.at = marks.iter().position(|hit| row(hit) >= top).unwrap_or(0);
        self.marks = marks;
        self.current_mark()
    }

    /// Replaces the rendered hits of a document that changed, keeping the hit in range.
    pub fn refresh_marks(&mut self, marks: Vec<Hit>) {
        self.marks = marks;
        self.at = self.at.min(self.marks.len().saturating_sub(1));
    }

    /// The footer's text for the search (Scouter `show`): the query and the hit count.
    pub fn message(&self) -> String {
        format!("/{}{}", self.query.text(), self.count_text())
    }

    /// The hit count after the query, or nothing for an empty query.
    pub fn count_text(&self) -> String {
        match (self.query.text().is_empty(), self.hit_count()) {
            (true, _) => String::new(),
            (false, 0) => "  no matches".into(),
            (false, n) => format!("  {} of {n}", self.at + 1),
        }
    }

    /// Finds the hits of the query again, and moves to the first hit at or below `top` (Scouter `type`). The caller has
    /// edited the query. Returns the line to show.
    pub fn retype(&mut self, text: &str, top: usize) -> Option<usize> {
        self.lines = matching_lines(text, self.query.text());
        self.at = self.lines.iter().position(|&line| line >= top).unwrap_or(0);
        self.current_line()
    }

    /// Steps to the next or previous hit, wrapping (Scouter `step`). Returns the line to show.
    pub fn step(&mut self, forward: bool) -> Option<usize> {
        let count = self.hit_count();
        if count > 0 {
            self.at = if forward { (self.at + 1) % count } else { (self.at + count - 1) % count };
        }
        self.current_line()
    }

    /// The search a kept hit starts with (Scouter `seed`): the query's hits in `text`, with the view on the hit at `line`.
    pub fn seed(text: &str, query: &str, line: usize) -> Self {
        let lines = matching_lines(text, query);
        let at = lines.iter().position(|&hit| hit == line).unwrap_or(0);
        Self { query: Line::with_text(query), lines, at, ..Self::default() }
    }

    /// Recomputes the hits for a text that changed or was shown (Scouter `refresh`), keeping the hit in range.
    pub fn refresh(&mut self, text: &str) {
        self.lines = matching_lines(text, self.query.text());
        self.at = self.at.min(self.lines.len().saturating_sub(1));
    }
}

/// The zero-based lines that contain `query`, ignoring case. An empty query hits nothing.
pub fn matching_lines(text: &str, query: &str) -> Vec<usize> {
    let needle = query.to_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }
    text.split('\n')
        .enumerate()
        .filter(|(_, line)| line.to_lowercase().contains(&needle))
        .map(|(index, _)| index)
        .collect()
}

/// The byte ranges of every occurrence of `query` in `text`, ignoring case, up to [`MAX_MARKS`]. Lowercasing that
/// changes the text's length marks nothing, as Scouter's does.
pub fn match_ranges(text: &str, query: &str) -> Vec<Range<usize>> {
    let lower = text.to_lowercase();
    let needle = query.to_lowercase();
    if needle.is_empty() || lower.len() != text.len() {
        return Vec::new();
    }
    lower.match_indices(&needle).take(MAX_MARKS).map(|(at, found)| at..at + found.len()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const VAULT: &str = "Gringotts\nHermione\n\nhermione again\nVault 713\n";

    #[test]
    fn matches_are_whole_lines_ignoring_case() {
        assert_eq!(matching_lines(VAULT, "HERMIONE"), vec![1, 3]);
        assert_eq!(matching_lines(VAULT, ""), Vec::<usize>::new());
        assert_eq!(matching_lines(VAULT, "azkaban"), Vec::<usize>::new());
    }

    #[test]
    fn every_occurrence_is_marked_and_capped() {
        assert_eq!(match_ranges("Hermione hermione", "hermione"), vec![0..8, 9..17]);
        assert!(match_ranges("Ünter", "ünter").len() <= 1);
        let many = "a".repeat(MAX_MARKS + 10);
        assert_eq!(match_ranges(&many, "a").len(), MAX_MARKS);
    }

    #[test]
    fn typing_moves_to_the_first_hit_at_or_below_the_top_line() {
        let mut search = Search::start();
        search.query = Line::with_text("hermione");
        assert_eq!(search.retype(VAULT, 2), Some(3));
        assert_eq!(search.message(), "/hermione  2 of 2");
        assert_eq!(search.retype(VAULT, 9), Some(1));
        search.query = Line::with_text("azkaban");
        assert_eq!(search.retype(VAULT, 0), None);
        assert_eq!(search.message(), "/azkaban  no matches");
    }

    #[test]
    fn stepping_wraps_both_ways() {
        let mut search = Search::start();
        search.query = Line::with_text("hermione");
        search.retype(VAULT, 0);
        assert_eq!(search.step(true), Some(3));
        assert_eq!(search.step(true), Some(1));
        assert_eq!(search.step(false), Some(3));
        assert_eq!(search.message(), "/hermione  2 of 2");
    }

    #[test]
    fn a_refresh_keeps_the_query_and_the_hit_in_range() {
        let mut search = Search::start();
        search.query = Line::with_text("hermione");
        search.retype(VAULT, 0);
        search.step(true);
        search.refresh("Hermione only\n");
        assert_eq!((search.lines.clone(), search.at), (vec![0], 0));
        assert_eq!(Search::start().message(), "/");
    }
}

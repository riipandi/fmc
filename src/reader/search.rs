//! In-file search (Scouter `src/reader/document-search.ts`, `src/reader/match-style.ts`): a query matches the whole lines
//! that contain it, ignoring case; `n` and `N` step through those lines and wrap; every occurrence is marked, up to
//! 2000. Lines are zero-based.

use std::ops::Range;

/// Scouter marks at most this many occurrences in a file (`src/reader/match-style.ts:11`).
pub const MAX_MARKS: usize = 2000;

/// The search in progress: the query, the lines it hits, the hit the view is on, and whether the prompt is open.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Search {
    pub query: String,
    pub lines: Vec<usize>,
    pub at: usize,
    /// The prompt is open: typed keys edit the query.
    pub editing: bool,
}

impl Search {
    /// A search that opens its prompt with an empty query (Scouter `start`).
    pub fn start() -> Self {
        Self { editing: true, ..Self::default() }
    }

    /// The line of the hit the view is on, if the query hits anything.
    pub fn current_line(&self) -> Option<usize> {
        self.lines.get(self.at).copied()
    }

    /// The footer's text for the search (Scouter `show`): the query and the hit count, or nothing for an empty query.
    pub fn message(&self) -> String {
        let count = match (self.query.is_empty(), self.lines.len()) {
            (true, _) => String::new(),
            (false, 0) => "  no matches".into(),
            (false, n) => format!("  {} of {n}", self.at + 1),
        };
        format!("/{}{count}", self.query)
    }

    /// Replaces the query, and moves to the first hit at or below `top` (Scouter `type`). Returns the line to show.
    pub fn retype(&mut self, text: &str, query: String, top: usize) -> Option<usize> {
        self.query = query;
        self.lines = matching_lines(text, &self.query);
        self.at = self.lines.iter().position(|&line| line >= top).unwrap_or(0);
        self.current_line()
    }

    /// Steps to the next or previous hit, wrapping (Scouter `step`). Returns the line to show.
    pub fn step(&mut self, forward: bool) -> Option<usize> {
        let count = self.lines.len();
        if count > 0 {
            self.at = if forward { (self.at + 1) % count } else { (self.at + count - 1) % count };
        }
        self.current_line()
    }

    /// Recomputes the hits for a text that changed or was shown (Scouter `refresh`), keeping the hit in range.
    pub fn refresh(&mut self, text: &str) {
        self.lines = matching_lines(text, &self.query);
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
        assert_eq!(search.retype(VAULT, "hermione".into(), 2), Some(3));
        assert_eq!(search.message(), "/hermione  2 of 2");
        assert_eq!(search.retype(VAULT, "hermione".into(), 9), Some(1));
        assert_eq!(search.retype(VAULT, "azkaban".into(), 0), None);
        assert_eq!(search.message(), "/azkaban  no matches");
    }

    #[test]
    fn stepping_wraps_both_ways() {
        let mut search = Search::start();
        search.retype(VAULT, "hermione".into(), 0);
        assert_eq!(search.step(true), Some(3));
        assert_eq!(search.step(true), Some(1));
        assert_eq!(search.step(false), Some(3));
        assert_eq!(search.message(), "/hermione  2 of 2");
    }

    #[test]
    fn a_refresh_keeps_the_query_and_the_hit_in_range() {
        let mut search = Search::start();
        search.retype(VAULT, "hermione".into(), 0);
        search.step(true);
        search.refresh("Hermione only\n");
        assert_eq!((search.lines.clone(), search.at), (vec![0], 0));
        assert_eq!(Search::start().message(), "/");
    }
}

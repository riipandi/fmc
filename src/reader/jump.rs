//! The jump history (Scouter `src/reader/jump-history.ts`): the places opened, back and forward, at most 100 each way.

use std::path::PathBuf;

const LIMIT: usize = 100;

/// A place in the reader: a file and the row it was scrolled to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Jump {
    pub path: PathBuf,
    pub top: usize,
}

#[derive(Debug, Default)]
pub struct History {
    older: Vec<Jump>,
    newer: Vec<Jump>,
}

impl History {
    /// Records the place left when another file opens. Leaving clears the forward places, and a place equal to the
    /// last one recorded is not added twice.
    pub fn leave(&mut self, here: Option<Jump>) {
        self.newer.clear();
        let Some(here) = here else { return };
        if self.older.last() == Some(&here) {
            return;
        }
        self.older.push(here);
        if self.older.len() > LIMIT {
            self.older.remove(0);
        }
    }

    /// The place to go back to; `here` becomes the first forward place.
    pub fn back(&mut self, here: Option<Jump>) -> Option<Jump> {
        let target = self.older.pop()?;
        self.newer.extend(here);
        Some(target)
    }

    pub fn forward(&mut self, here: Option<Jump>) -> Option<Jump> {
        let target = self.newer.pop()?;
        self.older.extend(here);
        Some(target)
    }

    pub fn can_go_back(&self) -> bool {
        !self.older.is_empty()
    }

    pub fn can_go_forward(&self) -> bool {
        !self.newer.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(name: &str, top: usize) -> Jump {
        Jump { path: name.into(), top }
    }

    #[test]
    fn back_and_forward_walk_the_places() {
        let mut history = History::default();
        history.leave(Some(at("Louvre/key.rs", 4)));
        history.leave(Some(at("CERN/vetra.json", 0)));
        assert_eq!(history.back(Some(at("Gringotts/vault.c", 9))), Some(at("CERN/vetra.json", 0)));
        assert_eq!(history.forward(Some(at("CERN/vetra.json", 0))), Some(at("Gringotts/vault.c", 9)));
        assert!(!history.can_go_forward() && history.can_go_back());
    }

    #[test]
    fn leaving_clears_the_forward_places_and_repeats_are_not_kept() {
        let mut history = History::default();
        history.leave(Some(at("a", 0)));
        history.back(Some(at("b", 0)));
        assert!(history.can_go_forward());
        history.leave(Some(at("c", 0)));
        assert!(!history.can_go_forward());
        history.leave(Some(at("c", 0)));
        assert_eq!(history.back(None), Some(at("c", 0)));
        assert_eq!(history.back(None), None);
    }

    #[test]
    fn the_history_keeps_a_hundred_places() {
        let mut history = History::default();
        for top in 0..150 {
            history.leave(Some(at("a", top)));
        }
        let mut count = 0;
        while history.back(None).is_some() {
            count += 1;
        }
        assert_eq!(count, LIMIT);
    }
}

//! The sidebar width (Scouter `src/explorer/sidebar-resizer.ts`): a quarter of the terminal within 28..=40
//! columns, never below 24 or so wide that the reader gets fewer than 28.

const MIN_SIDEBAR_COLUMNS: i64 = 24;
const MIN_READER_COLUMNS: i64 = 28;

pub fn constrain(terminal: u16, width: i64) -> u16 {
    let maximum = (i64::from(terminal) - MIN_READER_COLUMNS).max(1);
    width.clamp(MIN_SIDEBAR_COLUMNS.min(maximum), maximum) as u16
}

/// The width for a terminal `terminal` columns wide, or the user's own width when set.
pub fn fit(terminal: u16, preferred: Option<i64>) -> u16 {
    let default = (i64::from(terminal) / 4).clamp(28, 40);
    constrain(terminal, preferred.unwrap_or(default))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quarter_of_the_terminal_within_bounds() {
        assert_eq!(fit(80, None), 28);
        assert_eq!(fit(140, None), 35);
        assert_eq!(fit(187, None), 40);
        assert_eq!(fit(50, None), 22);
        assert_eq!(fit(20, None), 1);
        assert_eq!(fit(80, Some(10)), 24);
    }
}

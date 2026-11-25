//! The footer's scroll position (Scouter `formatScrollPosition`, `src/app/chrome.ts`).

/// `All` when the text fits the viewport, `Top` and `Bot` at the ends, else the percentage of the way down.
pub fn scroll_label(content_rows: usize, viewport_rows: usize, top: usize) -> String {
    let range = content_rows as i64 - viewport_rows as i64;
    if range <= 0 {
        "All".into()
    } else if top == 0 {
        "Top".into()
    } else if top as i64 >= range {
        "Bot".into()
    } else {
        format!("{}%", (top as f64 / range as f64 * 100.0).round() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_follow_scouter() {
        assert_eq!(scroll_label(10, 24, 0), "All");
        assert_eq!(scroll_label(50, 20, 0), "Top");
        assert_eq!(scroll_label(50, 20, 30), "Bot");
        assert_eq!(scroll_label(50, 20, 15), "50%");
        assert_eq!(scroll_label(50, 20, 1), "3%");
    }
}

//! Text made safe for one terminal line (Scouter `src/core/files.ts`, `displayText` and `displayLabel`).

/// Normalizes line ends and replaces control and bidi-override characters with U+FFFD.
pub fn display_text(text: &str) -> String {
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .chars()
        .map(|c| match c {
            '\u{0}'..='\u{8}' | '\u{b}' | '\u{c}' | '\u{e}'..='\u{1f}' | '\u{7f}'..='\u{9f}' => '\u{fffd}',
            '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}' => '\u{fffd}',
            c => c,
        })
        .collect()
}

/// [`display_text`] on one line: newlines and tabs become spaces.
pub fn display_label(text: &str) -> String {
    display_text(text).replace(['\n', '\t'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_controls_and_joins_lines() {
        assert_eq!(display_label("Gringotts\r\nvault\t713\u{1b}[31m"), "Gringotts vault 713\u{fffd}[31m");
        assert_eq!(display_text("Langdon\u{202e}"), "Langdon\u{fffd}");
    }
}

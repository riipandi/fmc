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

/// What `y` says after a copy (Scouter `copiedSummary`, `src/core/clipboard.ts`): the lines, or the characters, with
/// thousands separators as `en-US` writes them.
pub fn copied_summary(text: &str) -> String {
    let lines = text.split('\n').count();
    if lines > 1 {
        return format!("Copied {} lines.", grouped(lines));
    }
    let characters = text.chars().count();
    let noun = if characters == 1 { "character" } else { "characters" };
    format!("Copied {} {noun}.", grouped(characters))
}

fn grouped(number: usize) -> String {
    let digits = number.to_string();
    let mut out = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

#[cfg(test)]
mod copy_tests {
    use super::*;

    #[test]
    fn the_summary_counts_lines_or_characters_like_scouter() {
        assert_eq!(copied_summary("Hermione"), "Copied 8 characters.");
        assert_eq!(copied_summary("x"), "Copied 1 character.");
        assert_eq!(copied_summary("Hermione\nGranger"), "Copied 2 lines.");
        assert_eq!(copied_summary(&"ü".repeat(1234)), "Copied 1,234 characters.");
    }
}

//! Lone image paragraphs become box fences, line for line; everything else keeps its text.

use super::*;

const MARAUDER: &str = "![Marauder map](maps/marauder.png)";

/// The rewritten source, with the rule that every line keeps its number checked on the way.
fn boxed(source: &str) -> String {
    let out = as_boxes(source).into_owned();
    assert_eq!(out.matches('\n').count(), source.matches('\n').count(), "the line count holds:\n{out}");
    out
}

#[test]
fn a_lone_image_between_blank_lines_becomes_a_box_fence_on_its_own_lines() {
    let out = boxed(&format!("Before the map.\n\n{MARAUDER}\n\nAfter.\n"));
    assert_eq!(out, format!("Before the map.\n\n```fmc-image {MARAUDER}\n```\nAfter.\n"));
}

#[test]
fn a_lone_image_at_the_top_or_the_end_of_the_file_is_boxed() {
    assert_eq!(boxed(&format!("{MARAUDER}\n\nAfter.\n")), format!("```fmc-image {MARAUDER}\n```\nAfter.\n"));
    assert_eq!(boxed(&format!("Before.\n\n{MARAUDER}")), format!("Before.\n\n```fmc-image {MARAUDER}"));
    assert_eq!(boxed(&format!("Before.\n\n{MARAUDER}\n")), format!("Before.\n\n```fmc-image {MARAUDER}\n"));
}

#[test]
fn an_image_inside_a_sentence_a_list_or_a_quote_keeps_its_text() {
    for source in [
        format!("See the {MARAUDER} map.\n"),
        format!("- {MARAUDER}\n"),
        format!("> {MARAUDER}\n"),
        format!("Text.\n\n- item\n\n  {MARAUDER}\n"),
    ] {
        assert_eq!(boxed(&source), source, "{source}");
    }
}

#[test]
fn two_images_in_one_paragraph_keep_their_text_and_two_blank_separated_ones_are_boxes() {
    let joined = format!("{MARAUDER}\n![Seal](seal.png)\n");
    assert_eq!(boxed(&joined), joined);
    let apart = format!("{MARAUDER}\n\n![Seal](seal.png)\n");
    assert_eq!(boxed(&apart), "```fmc-image ![Marauder map](maps/marauder.png)\n```\n```fmc-image ![Seal](seal.png)\n");
}

#[test]
fn an_image_with_a_title_is_boxed_with_the_title_in_its_fence() {
    let line = "![Marauder map](maps/marauder.png \"The Marauder's Map\")";
    assert_eq!(boxed(&format!("{line}\n\nAfter.\n")), format!("```fmc-image {line}\n```\nAfter.\n"));
}

#[test]
fn html_and_a_line_with_no_blank_after_it_keep_their_text() {
    let html = "<img src=\"maps/marauder.png\">\n\nAfter.\n";
    assert_eq!(boxed(html), html);
    let joined = format!("{MARAUDER}\nAfter.\n");
    assert_eq!(boxed(&joined), joined);
}

#[test]
fn a_backtick_in_the_line_keeps_its_text_since_a_fence_cannot_carry_it() {
    let source = "![`code`](maps/marauder.png)\n\nAfter.\n";
    assert_eq!(boxed(source), source);
}

#[test]
fn an_image_inside_a_code_block_is_code() {
    let source = format!("```\n{MARAUDER}\n```\n");
    assert_eq!(boxed(&source), source);
}

#[test]
fn a_remote_and_an_svg_source_are_boxed_too_so_the_box_can_say_why() {
    let out = boxed("![Seal](https://hogwarts.test/seal.png)\n\n![Crest](crest.svg)\n");
    assert_eq!(out, "```fmc-image ![Seal](https://hogwarts.test/seal.png)\n```\n```fmc-image ![Crest](crest.svg)\n");
}

#[test]
fn front_matter_and_an_image_are_both_written_as_fences_and_the_lines_hold() {
    let source = "---\ntitle: Gryffindor\n---\n\n![Marauder map](maps/marauder.png)\n\nText.\n";
    let out = boxed(&prepared(source));
    assert_eq!(out, "```yaml\ntitle: Gryffindor\n```\n\n```fmc-image ![Marauder map](maps/marauder.png)\n```\nText.\n");
}

#[test]
fn a_crlf_file_keeps_its_line_endings() {
    let out = boxed(&format!("Before.\r\n\r\n{MARAUDER}\r\n\r\nAfter.\r\n"));
    assert_eq!(out, format!("Before.\r\n\r\n```fmc-image {MARAUDER}\r\n```\r\nAfter.\r\n"));
}

#[test]
fn a_source_without_an_image_is_borrowed_unchanged() {
    let source = "# Hogwarts\n\nPotions.\n";
    assert!(matches!(as_boxes(source), std::borrow::Cow::Borrowed(_)));
}

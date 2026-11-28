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
fn a_line_with_no_blank_after_it_keeps_its_text() {
    let joined = format!("{MARAUDER}\nAfter.\n");
    assert_eq!(boxed(&joined), joined);
    let html = "<img src=\"maps/marauder.png\">\nAfter.\n";
    assert_eq!(boxed(html), html);
}

const README_LOGO: &str = "<h1>\n  <picture>\n    <source media=\"(prefers-color-scheme: dark)\" srcset=\"assets/brand/png/logo-dark@2x.png\">\n    <img alt=\"fmc\" src=\"assets/brand/png/logo-light@2x.png\" width=\"392\">\n  </picture>\n</h1>\n";

#[test]
fn the_readme_logo_block_is_one_box_that_keeps_its_lines() {
    let out = boxed(&format!("{README_LOGO}\nAfter.\n"));
    assert_eq!(
        out,
        "```fmc-image <h1> <picture> <source media=\"(prefers-color-scheme: dark)\" srcset=\"assets/brand/png/logo-dark@2x.png\"> <img alt=\"fmc\" src=\"assets/brand/png/logo-light@2x.png\" width=\"392\"> </picture> </h1>\n  <picture>\n    <source media=\"(prefers-color-scheme: dark)\" srcset=\"assets/brand/png/logo-dark@2x.png\">\n    <img alt=\"fmc\" src=\"assets/brand/png/logo-light@2x.png\" width=\"392\">\n  </picture>\n```\n\nAfter.\n"
    );
}

#[test]
fn a_one_line_html_image_is_boxed_and_closes_on_the_blank_line() {
    let source = "<p align=\"center\"><img src=\"maps/marauder.png\" width=\"120\"></p>\n\nAfter.\n";
    let out = boxed(source);
    assert_eq!(
        out,
        "```fmc-image <p align=\"center\"><img src=\"maps/marauder.png\" width=\"120\"></p>\n```\nAfter.\n"
    );
}

#[test]
fn an_html_block_with_text_a_second_image_or_another_element_keeps_its_text() {
    for source in [
        "<p>See <img src=\"maps/marauder.png\"></p>\n\nAfter.\n",
        "<img src=\"a.png\"> <img src=\"b.png\">\n\nAfter.\n",
        "<span><img src=\"maps/marauder.png\"></span>\n\nAfter.\n",
        "<img src=\"`a`.png\">\n\nAfter.\n",
    ] {
        assert_eq!(boxed(source), source, "{source}");
    }
}

#[test]
fn an_html_image_inside_a_list_keeps_its_text() {
    let source = "- <img src=\"maps/marauder.png\">\n";
    assert_eq!(boxed(source), source);
}

#[test]
fn a_remote_html_image_is_boxed_so_the_box_can_say_why() {
    let out = boxed("<img src=\"https://hogwarts.test/seal.png\" alt=\"Seal\">\n\nAfter.\n");
    assert_eq!(out, "```fmc-image <img src=\"https://hogwarts.test/seal.png\" alt=\"Seal\">\n```\nAfter.\n");
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

#[test]
fn the_readme_head_is_one_box_and_the_file_keeps_its_line_count() {
    let head: Vec<&str> = include_str!("../../README.md").lines().take(6).collect();
    let source = format!("{}\n\nAfter.\n", head.join("\n"));
    let out = boxed(&source);
    assert!(out.starts_with("```fmc-image <h1> <picture> <source media=\"(prefers-color-scheme: dark)\" srcset=\"assets/brand/png/logo-dark@2x.png\">"), "{out}");
    assert!(
        out.contains("<img alt=\"fmc\" src=\"assets/brand/png/logo-light@2x.png\" width=\"392\"> </picture> </h1>\n"),
        "{out}"
    );
    assert!(out.ends_with("```\n\nAfter.\n"), "{out}");
}

use super::with_markers;

/// A line count that survives the rewrite: the source view's line numbers and the rows of a document depend on it.
fn assert_same_lines(source: &str, marked: &str) {
    assert_eq!(source.split('\n').count(), marked.split('\n').count(), "{marked:?}");
}

#[test]
fn each_atx_level_keeps_its_marker_and_escapes_it_into_the_text() {
    let source = "# Hogwarts\n## Gryffindor\n### Dumbledore\n#### Snape\n##### Sirius\n###### Lupin\n";
    assert_eq!(
        with_markers(source),
        "# &#35; Hogwarts\n## &#35;&#35; Gryffindor\n### &#35;&#35;&#35; Dumbledore\n#### &#35;&#35;&#35;&#35; Snape\n\
         ##### &#35;&#35;&#35;&#35;&#35; Sirius\n###### &#35;&#35;&#35;&#35;&#35;&#35; Lupin\n"
    );
}

#[test]
fn a_closing_run_stays_after_the_text_and_the_marker_is_still_stripped() {
    assert_eq!(with_markers("## Gringotts ##\n"), "## &#35;&#35; Gringotts ##\n");
}

#[test]
fn an_empty_heading_shows_its_marker_alone() {
    assert_eq!(with_markers("#\n##\n"), "# &#35;\n## &#35;&#35;\n");
}

#[test]
fn a_seven_hash_run_and_a_hash_without_a_space_are_text() {
    let source = "####### Too deep\n#hashtag\nC# is a language\n";
    assert_eq!(with_markers(source), source);
}

#[test]
fn a_setext_heading_takes_the_marker_of_its_level_and_its_underline_goes_blank() {
    let marked = with_markers("Title\n===\n\nSub\n---\n");
    assert_eq!(marked, "# &#35; Title\n\n\n## &#35;&#35; Sub\n\n");
    assert_same_lines("Title\n===\n\nSub\n---\n", &marked);
}

#[test]
fn a_setext_line_without_a_paragraph_above_is_text() {
    let source = "===\n\n---\n";
    assert_eq!(with_markers(source), source);
}

#[test]
fn a_multi_line_setext_heading_is_left_as_it_is() {
    let source = "Line one\nLine two\n===\n";
    assert_eq!(with_markers(source), source);
}

#[test]
fn a_hash_inside_a_backtick_or_tilde_fence_is_code() {
    let source = "```bash\n# comment\n```\n\n~~~\n## Not a heading\n~~~\n\n# Dumbledore\n";
    let marked = with_markers(source);
    assert_eq!(marked, "```bash\n# comment\n```\n\n~~~\n## Not a heading\n~~~\n\n# &#35; Dumbledore\n");
    assert_same_lines(source, &marked);
}

#[test]
fn a_fence_of_four_backticks_is_closed_only_by_a_run_as_long() {
    let source = "````\n```\n# still code\n```\n````\n# After\n";
    assert_eq!(with_markers(source), "````\n```\n# still code\n```\n````\n# &#35; After\n");
}

#[test]
fn a_fence_inside_a_quote_closes_on_the_quote_marked_line() {
    let source = "> ```\n> # code\n> ```\n\n# After\n";
    assert_eq!(with_markers(source), "> ```\n> # code\n> ```\n\n# &#35; After\n");
}

#[test]
fn an_unclosed_fence_runs_to_the_end_of_the_document() {
    let source = "```\n# never a heading\n";
    assert_eq!(with_markers(source), source);
}

#[test]
fn an_indented_code_block_is_code_but_a_paragraph_continuation_is_text() {
    let source = "    # indented code\n\nText\n    # continuation\n\n# Title\n";
    let marked = with_markers(source);
    assert_eq!(marked, "    # indented code\n\nText\n    # continuation\n\n# &#35; Title\n");
}

#[test]
fn an_html_block_is_left_until_its_blank_line() {
    let source = "<details>\n# inside\n</details>\n\n# After\n";
    let marked = with_markers(source);
    assert_eq!(marked, "<details>\n# inside\n</details>\n\n# &#35; After\n");
    assert_same_lines(source, &marked);
}

#[test]
fn a_comment_block_is_left_until_its_end_marker() {
    let source = "<!--\n# hidden\n-->\n# Shown\n";
    assert_eq!(with_markers(source), "<!--\n# hidden\n-->\n# &#35; Shown\n");
}

#[test]
fn an_escaped_or_inline_hash_is_not_a_heading() {
    let source = "\\# not a heading\n\nSome `# code` and a # sign.\n";
    assert_eq!(with_markers(source), source);
}

#[test]
fn a_heading_in_a_list_or_a_quote_takes_the_marker_after_its_prefix() {
    assert_eq!(
        with_markers("- ## In a list\n> ### In a quote\n1. # Numbered\n"),
        "- ## &#35;&#35; In a list\n> ### &#35;&#35;&#35; In a quote\n1. # &#35; Numbered\n"
    );
}

#[test]
fn a_list_item_with_an_indented_heading_is_still_marked() {
    assert_eq!(with_markers("- Item\n\n    ## Nested\n"), "- Item\n\n    ## &#35;&#35; Nested\n");
}

#[test]
fn a_thematic_break_before_a_setext_line_is_not_a_paragraph() {
    assert_eq!(with_markers("***\nTitle\n===\n"), "***\n# &#35; Title\n\n");
}

#[test]
fn a_mermaid_fence_beside_a_heading_is_left_as_its_source() {
    let source = "# Hogwarts\n\n```mermaid\nflowchart LR\n    A[Harry Potter] --> B[Albus Dumbledore]\n```\n";
    let marked = with_markers(source);
    assert_eq!(
        marked,
        "# &#35; Hogwarts\n\n```mermaid\nflowchart LR\n    A[Harry Potter] --> B[Albus Dumbledore]\n```\n"
    );
    assert_same_lines(source, &marked);
}

#[test]
fn the_line_count_holds_for_a_mixed_document() {
    let source = "Title\n===\n\n```\n# code\n```\n\n- ## item\n\n<div>\n# html\n</div>\n\n## End ##\n";
    assert_same_lines(source, &with_markers(source));
}

#[test]
fn an_empty_document_stays_empty() {
    assert_eq!(with_markers(""), "");
}

use std::borrow::Cow;

use super::*;

#[test]
fn yaml_front_matter_becomes_a_yaml_fence_with_the_same_lines() {
    let source = "---\ntitle: Hermione Granger\nhouse: Gryffindor\n---\n# Hogwarts\n";
    let fenced = as_fence(source);
    assert_eq!(fenced, "```yaml\ntitle: Hermione Granger\nhouse: Gryffindor\n```\n# Hogwarts\n");
    assert_eq!(fenced.matches('\n').count(), source.matches('\n').count());
}

#[test]
fn toml_front_matter_becomes_a_toml_fence() {
    assert_eq!(as_fence("+++\ntitle = \"Hogwarts\"\n+++\n\nBody.\n"), "```toml\ntitle = \"Hogwarts\"\n```\n\nBody.\n");
}

#[test]
fn a_block_at_the_end_of_the_file_keeps_its_missing_newline() {
    assert_eq!(as_fence("---\ntitle: Owls\n---"), "```yaml\ntitle: Owls\n```");
}

#[test]
fn crlf_line_endings_are_kept() {
    assert_eq!(as_fence("---\r\ntitle: Owls\r\n---\r\nBody\r\n"), "```yaml\r\ntitle: Owls\r\n```\r\nBody\r\n");
}

#[test]
fn trailing_spaces_on_a_marker_still_open_and_close_the_block() {
    assert_eq!(as_fence("---  \ntitle: Owls\n---\t\n"), "```yaml\ntitle: Owls\n```\n");
}

#[test]
fn a_source_without_front_matter_is_borrowed_unchanged() {
    let sources = [
        "# Hogwarts\n\n---\n\nA rule later in the file stays a rule.\n",
        "\n---\ntitle: Owls\n---\n",
        "----\ntitle: Owls\n----\n",
        "---\ntitle: Owls\n",
        "---\n---\nBody\n",
        "---",
        "",
        "--- # a comment\ntitle\n---\n",
        "---\ntitle: Owls\n```\n---\n",
        "---\ntitle: Owls\n~~~\n---\n",
    ];
    for source in sources {
        let fenced = as_fence(source);
        assert!(matches!(fenced, Cow::Borrowed(_)), "{source:?} is left alone");
        assert_eq!(fenced, source);
    }
}

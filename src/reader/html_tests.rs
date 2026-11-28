use super::*;

fn names(text: &str) -> Vec<(String, bool)> {
    tags(text).expect("tags").into_iter().map(|tag| (tag.name, tag.closing)).collect()
}

#[test]
fn tags_come_in_order_with_their_closing_marks() {
    assert_eq!(
        names("<h1>\n  <picture>\n</picture>\n</h1>"),
        [("h1", false), ("picture", false), ("picture", true), ("h1", true)].map(|(n, c)| (n.to_owned(), c))
    );
}

#[test]
fn attributes_read_quoted_unquoted_and_bare() {
    let tag = &tags(r#"<img alt="fmc" src='a b.png' width=392 hidden>"#).expect("tags")[0];
    assert_eq!(tag.attr("alt"), Some("fmc"));
    assert_eq!(tag.attr("src"), Some("a b.png"));
    assert_eq!(tag.attr("width"), Some("392"));
    assert_eq!(tag.attr("hidden"), Some(""));
    assert_eq!(tag.attr("missing"), None);
}

#[test]
fn names_and_attribute_keys_read_lowercase_and_self_closing_tags_parse() {
    let tag = &tags(r#"<IMG SRC="a.png"/>"#).expect("tags")[0];
    assert_eq!((tag.name.as_str(), tag.attr("src")), ("img", Some("a.png")));
}

#[test]
fn text_a_comment_or_a_broken_tag_is_not_tags() {
    for text in [
        "Caption <img src=\"a.png\">",
        "<img src=\"a.png\">Caption",
        "<!-- note --><img src=\"a.png\">",
        "<img src=\"a.png\"",
        "<img src=\"a.png",
        "<>",
        "< img>",
        "<img /x>",
    ] {
        assert_eq!(tags(text), None, "{text}");
    }
}

#[test]
fn an_empty_text_has_no_tags() {
    assert_eq!(tags("  \n "), Some(Vec::new()));
}

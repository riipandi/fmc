//! Front matter in the rendered Markdown view. OpenTUI has no front matter: an opening `---` is a rule and the fields read
//! as a paragraph, with the closing `---` turned into a heading. A YAML (`---`) or TOML (`+++`) block at the very start of
//! the file is therefore written as a code fence before the view reads it, so it draws as a highlighted block. Every line
//! stays a line, so the source line numbers, the search, and the git signs keep their rows. The source view shows the file
//! as it is.

use std::borrow::Cow;

/// The source with its front matter written as a code fence: the opening marker's line becomes a `yaml` or `toml` fence
/// opening, and the closing marker's line becomes a fence closing. A source without front matter at its first line, with
/// no closing marker, with an empty block, or with a line in the block that would close a fence, is returned unchanged.
pub fn as_fence(source: &str) -> Cow<'_, str> {
    let lines: Vec<&str> = source.split_inclusive('\n').collect();
    let Some(&first) = lines.first() else { return Cow::Borrowed(source) };
    let Some((marker, language)) = opening(first) else { return Cow::Borrowed(source) };
    // The first marker after the opening one closes the block; one right after it is an empty block.
    let Some(close) = (1..lines.len()).find(|&index| lines[index].trim_end() == marker) else {
        return Cow::Borrowed(source);
    };
    if close == 1 {
        return Cow::Borrowed(source);
    }
    // A fence inside the block would close it early, so the block stays as OpenTUI draws it.
    let block = &lines[1..close];
    if block.iter().any(|line| line.trim_start().starts_with("```") || line.trim_start().starts_with("~~~")) {
        return Cow::Borrowed(source);
    }
    let mut out = String::with_capacity(source.len() + 8);
    for (index, line) in lines.iter().enumerate() {
        let end = &line[line.trim_end_matches(['\r', '\n']).len()..];
        if index == 0 {
            out.push_str("```");
            out.push_str(language);
            out.push_str(end);
        } else if index == close {
            out.push_str("```");
            out.push_str(end);
        } else {
            out.push_str(line);
        }
    }
    Cow::Owned(out)
}

/// The marker and the fence language of a first line that opens front matter: `---` is YAML and `+++` is TOML.
fn opening(line: &str) -> Option<(&'static str, &'static str)> {
    match line.trim_end() {
        "---" => Some(("---", "yaml")),
        "+++" => Some(("+++", "toml")),
        _ => None,
    }
}

#[cfg(test)]
#[path = "frontmatter_tests.rs"]
mod tests;

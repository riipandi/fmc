//! The patterns of `file_associations`: `*`, `**`, `?`, and `{a,b}`, as VS Code's `files.associations` reads them. A
//! pattern without `/` matches a file name; a pattern with `/` matches the path relative to the opened folder. Hand
//! written, so no glob crate is needed (`rules.md`, "Dependencies").

/// Whether the filesystem ignores case: macOS and Windows do, so a pattern matches `Vault.RS.BAK` as it matches the file.
pub const CASE_INSENSITIVE: bool = cfg!(any(target_os = "macos", target_os = "windows"));

/// Brace expansions one pattern may make. A pattern past it is refused, so a bad settings file cannot make a match slow.
const MAX_ALTERNATIVES: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Token {
    Literal(char),
    /// `?`: one character.
    One,
    /// `*`: any run of characters within one path segment.
    Run,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Segment {
    /// `**` as a whole segment: any number of segments, zero included.
    Any,
    Part(Vec<Token>),
}

/// A parsed pattern. `{a,b}` is expanded when the pattern is parsed, so a match tries each alternative.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Glob {
    alternatives: Vec<Vec<Segment>>,
    /// Whether the pattern has a `/`: it then matches the relative path, not the name.
    pub is_path: bool,
}

impl Glob {
    pub fn parse(pattern: &str) -> Result<Self, String> {
        if pattern.is_empty() {
            return Err("the pattern is empty".into());
        }
        let expanded = expand(pattern)?;
        if expanded.len() > MAX_ALTERNATIVES {
            return Err(format!("the pattern expands to more than {MAX_ALTERNATIVES} alternatives"));
        }
        Ok(Self {
            alternatives: expanded.iter().map(|alternative| segments(alternative)).collect(),
            is_path: pattern.contains('/'),
        })
    }

    /// Whether the pattern matches a file: `relative` is its path under the opened folder, `/`-separated, and `None`
    /// when the file lies outside that folder, where only a name pattern can match.
    pub fn matches(&self, relative: Option<&str>, name: &str) -> bool {
        self.alternatives.iter().any(|segments| {
            if self.is_path {
                relative.is_some_and(|path| matches_segments(segments, &path.split('/').collect::<Vec<_>>()))
            } else {
                matches_segments(segments, &[name])
            }
        })
    }
}

/// Expands the first `{...}` group, then the rest, so nested groups come out in order.
fn expand(pattern: &str) -> Result<Vec<String>, String> {
    let Some(open) = pattern.find('{') else { return Ok(vec![pattern.to_string()]) };
    let mut depth = 0usize;
    let mut close = None;
    let mut commas = Vec::new();
    for (at, c) in pattern[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(open + at);
                    break;
                }
            }
            ',' if depth == 1 => commas.push(open + at),
            _ => {}
        }
    }
    let close = close.ok_or("a `{` has no matching `}`")?;
    let (prefix, suffix) = (&pattern[..open], &pattern[close + 1..]);
    let mut bounds = vec![open + 1];
    bounds.extend(commas.iter().map(|comma| comma + 1));
    let mut ends = commas;
    ends.push(close);
    let mut expanded = Vec::new();
    for (start, end) in bounds.into_iter().zip(ends) {
        for alternative in expand(&format!("{prefix}{}{suffix}", &pattern[start..end]))? {
            expanded.push(alternative);
            if expanded.len() > MAX_ALTERNATIVES {
                return Err(format!("the pattern expands to more than {MAX_ALTERNATIVES} alternatives"));
            }
        }
    }
    Ok(expanded)
}

fn segments(alternative: &str) -> Vec<Segment> {
    alternative
        .split('/')
        .map(|part| {
            if part == "**" {
                Segment::Any
            } else {
                Segment::Part(
                    part.chars()
                        .map(|c| match c {
                            '*' => Token::Run,
                            '?' => Token::One,
                            c => Token::Literal(c),
                        })
                        .collect(),
                )
            }
        })
        .collect()
}

fn fold(c: char) -> char {
    if CASE_INSENSITIVE { c.to_lowercase().next().unwrap_or(c) } else { c }
}

fn matches_segments(segments: &[Segment], parts: &[&str]) -> bool {
    match segments.split_first() {
        None => parts.is_empty(),
        Some((Segment::Any, rest)) => (0..=parts.len()).any(|skip| matches_segments(rest, &parts[skip..])),
        Some((Segment::Part(tokens), rest)) => match parts.split_first() {
            Some((part, others)) => {
                let text: Vec<char> = part.chars().collect();
                matches_tokens(tokens, &text) && matches_segments(rest, others)
            }
            None => false,
        },
    }
}

/// The wildcard match of one segment. A `*` takes the text back to its last restart point, which keeps the match
/// linear in the pattern times the text, where a recursive match is exponential in the number of `*`.
fn matches_tokens(tokens: &[Token], text: &[char]) -> bool {
    let (mut at, mut seen) = (0, 0);
    // The token after the last `*`, and the text it has taken so far.
    let mut restart: Option<(usize, usize)> = None;
    while seen < text.len() {
        match tokens.get(at) {
            Some(Token::Literal(c)) if fold(*c) == fold(text[seen]) => {
                at += 1;
                seen += 1;
            }
            Some(Token::One) => {
                at += 1;
                seen += 1;
            }
            Some(Token::Run) => {
                restart = Some((at + 1, seen));
                at += 1;
            }
            _ => match restart {
                Some((next, taken)) => {
                    at = next;
                    seen = taken + 1;
                    restart = Some((next, taken + 1));
                }
                None => return false,
            },
        }
    }
    tokens[at..].iter().all(|token| *token == Token::Run)
}

#[cfg(test)]
#[path = "glob_tests.rs"]
mod tests;

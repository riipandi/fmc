//! The `file_associations` table (VS Code's `files.associations`): the filetype a file is shown as, from a pattern that
//! matches its name or its path (`reader/glob.rs`). A file no pattern matches keeps OpenTUI's own lookup.

use std::path::Path;

use super::glob::Glob;
use super::language::HIGHLIGHTED;

/// VS Code language IDs that name a filetype fmc already highlights under another name, and `dotenv` and `env`, which
/// OpenTUI does not know: they show with the bash grammar, which reads `KEY=value` and `#` lines well.
const ALIASES: &[(&str, &str)] = &[("shellscript", "bash"), ("jsonc", "json"), ("dotenv", "bash"), ("env", "bash")];

/// What an association sets: the filetype to highlight the file as, or plain text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Filetype(&'static str),
    Plain,
}

impl Language {
    /// Every name `named` accepts: the highlighted filetypes, the aliases, and `plaintext`.
    pub fn names() -> Vec<&'static str> {
        let aliases = ALIASES.iter().map(|(alias, _)| *alias);
        HIGHLIGHTED.iter().copied().chain(aliases).chain(["plaintext"]).collect()
    }

    /// The language a value names, or `None` for a name fmc does not know.
    pub fn named(value: &str) -> Option<Self> {
        if value == "plaintext" {
            return Some(Self::Plain);
        }
        let filetype = ALIASES
            .iter()
            .find(|(alias, _)| *alias == value)
            .map(|(_, filetype)| *filetype)
            .or_else(|| HIGHLIGHTED.iter().copied().find(|filetype| *filetype == value))?;
        Some(Self::Filetype(filetype))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Rule {
    pattern: String,
    glob: Glob,
    language: Language,
}

/// The table, in no particular order: precedence is decided by the patterns, not by the order they were added.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Associations {
    rules: Vec<Rule>,
}

impl Associations {
    /// Adds an association. An error names the reason, and the table is unchanged.
    pub fn add(&mut self, pattern: &str, value: &str) -> Result<(), String> {
        let glob = Glob::parse(pattern)?;
        let language = Language::named(value).ok_or_else(|| format!("{value} is not a filetype fmc shows"))?;
        self.rules.push(Rule { pattern: pattern.to_string(), glob, language });
        Ok(())
    }

    /// The language of a file under `root` (the opened folder), or `None` when no pattern matches it. A file outside
    /// `root` matches by its name only.
    pub fn language_of(&self, path: &Path, root: &Path) -> Option<Language> {
        let name = path.file_name()?.to_string_lossy();
        let relative = path.strip_prefix(root).ok().map(|rest| {
            let parts: Vec<_> = rest.components().map(|part| part.as_os_str().to_string_lossy()).collect();
            parts.join("/")
        });
        self.language(relative.as_deref(), &name)
    }

    /// The language a file takes. A path pattern beats a name pattern, then the longer pattern wins, then the pattern
    /// that sorts first: `serde_json` keeps no file order, so "the last one wins" would not be defined.
    pub fn language(&self, relative: Option<&str>, name: &str) -> Option<Language> {
        self.rules
            .iter()
            .filter(|rule| rule.glob.matches(relative, name))
            .max_by_key(|rule| {
                (rule.glob.is_path, rule.pattern.chars().count(), std::cmp::Reverse(rule.pattern.as_str()))
            })
            .map(|rule| rule.language)
    }
}

#[cfg(test)]
#[path = "associations_tests.rs"]
mod tests;

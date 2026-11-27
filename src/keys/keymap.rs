//! The keymap: the default table merged with `keymap.json` (plan-00015, A10, A11). `parse` takes the file's text and
//! returns the merged keys and a notice for each entry it skips. The file itself is read by `services/keymap.rs`.

use serde_json::Value;

use super::Action;
use super::bindings::{Binding, Chord, Context, default_bindings, display_sequence};
use crate::text::display_label;

/// The merged table in dispatch order: the user's bindings first, then the defaults that were not replaced.
#[derive(Clone, Debug)]
pub struct Keymap {
    bindings: Vec<Binding>,
}

impl Keymap {
    pub fn defaults() -> Keymap {
        Keymap { bindings: default_bindings() }
    }

    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }

    /// The keys of `actions`, shown the way the card shows them, each once, in table order.
    pub fn keys_for(&self, actions: &[Action]) -> Vec<String> {
        let mut keys: Vec<String> = Vec::new();
        for binding in self.bindings.iter().filter(|b| actions.contains(&b.action)) {
            let key = display_sequence(&binding.keys);
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
        keys
    }
}

/// The merged keys, and a notice for each entry of the file that was skipped (plan-00015, A11).
#[derive(Clone, Debug)]
pub struct Parsed {
    pub keymap: Keymap,
    pub notices: Vec<String>,
}

impl Default for Parsed {
    fn default() -> Parsed {
        Parsed { keymap: Keymap::defaults(), notices: Vec::new() }
    }
}

/// One entry of the file: a binding, or `null`, which unbinds the default at the same context and key.
enum Entry {
    Bind { context: Context, keys: Vec<Chord>, action: Action },
    Unbind { context: Context, keys: Vec<Chord> },
}

impl Entry {
    fn context(&self) -> Context {
        match self {
            Entry::Bind { context, .. } | Entry::Unbind { context, .. } => *context,
        }
    }

    /// A binding whose last key has no Shift rule comes after one that has it, so `shift-down` wins over `down`.
    fn plain_last(&self) -> bool {
        let keys = match self {
            Entry::Bind { keys, .. } | Entry::Unbind { keys, .. } => keys,
        };
        keys.last().is_some_and(Chord::shift_unchecked)
    }
}

/// The text of a keymap file (at most 1 MiB, read by the caller). A file that is not a JSON object keeps every default,
/// and one notice says so. Each entry is `"<context>.<key>": "<action>"` or `"<context>.<key>": null`; a key is one
/// chord or several separated by spaces (`"browse.g x"`).
pub fn parse(text: &str) -> Parsed {
    let value: Value = match serde_json::from_str(text) {
        Ok(value) => value,
        Err(error) => {
            return whole_file(format!(
                "keymap.json is not valid JSON (line {}); every key keeps its default.",
                error.line()
            ));
        }
    };
    let Value::Object(map) = value else {
        return whole_file("keymap.json must hold a JSON object; every key keeps its default.".into());
    };
    let mut entries = Vec::new();
    let mut notices = Vec::new();
    for (order, (name, value)) in map.iter().enumerate() {
        match entry(name, value) {
            Ok(entry) => entries.push((order, entry)),
            // serde_json keeps no line numbers in a Value, so a skipped entry's line is found in the text. Only a
            // skipped entry searches: shortcut: a file of many bad entries searches the text once per entry, upgrade: a
            // line index built in one pass if a file of that size shows a slow start.
            Err(reason) => {
                let line = line_of(text, name);
                notices.push(display_label(&format!("keymap.json line {line}: \"{name}\": {reason}.")));
            }
        }
    }
    Parsed { keymap: merge(entries), notices }
}

fn whole_file(notice: String) -> Parsed {
    Parsed { keymap: Keymap::defaults(), notices: vec![notice] }
}

fn entry(name: &str, value: &Value) -> Result<Entry, String> {
    let Some((context, keys)) = name.split_once('.') else {
        return Err("needs a context before the key, as \"reader.j\"".into());
    };
    let Some(context) = Context::from_name(context) else {
        return Err(format!("names no context \"{}\"", display_label(context)));
    };
    let keys = keys.split(' ').map(Chord::parse).collect::<Result<Vec<Chord>, String>>()?;
    match value {
        Value::Null => Ok(Entry::Unbind { context, keys }),
        Value::String(action) => match Action::from_name(action) {
            Some(action) => Ok(Entry::Bind { context, keys, action }),
            None => Err(format!("names no action \"{}\"", display_label(action))),
        },
        _ => Err("needs an action name, or null to unbind".into()),
    }
}

/// The defaults with the file's entries applied: a bound key replaces the default at its context and key, and `null`
/// removes it. Entries sort so that a narrower context wins over `everywhere`, and a shifted key over its plain key.
fn merge(mut entries: Vec<(usize, Entry)>) -> Keymap {
    // `order` is the entry's place in serde_json's map, which is sorted by name (no `preserve_order`), so a tie is stable.
    entries.sort_by_key(|(order, entry)| {
        let rank = match entry.context() {
            Context::Everywhere => 2,
            Context::Changes => 1,
            _ => 0,
        };
        (rank, entry.plain_last(), *order)
    });
    let mut defaults = default_bindings();
    let mut bindings = Vec::new();
    for (_, entry) in entries {
        match entry {
            Entry::Bind { context, keys, action } => {
                defaults.retain(|d| !(d.context == context && d.keys == keys));
                bindings.push(Binding { context, keys, typing: None, action, user: true });
            }
            Entry::Unbind { context, keys } => defaults.retain(|d| !(d.context == context && d.keys == keys)),
        }
    }
    bindings.extend(defaults);
    Keymap { bindings }
}

/// The line of the first `"name":` in `text`, counted from 1. The key is searched in its JSON form so an escaped name
/// still matches; a name the text does not show gives line 1.
fn line_of(text: &str, name: &str) -> usize {
    let encoded = serde_json::to_string(name).unwrap_or_default();
    text.find(&encoded).map_or(1, |at| text[..at].matches('\n').count() + 1)
}

#[cfg(test)]
#[path = "keymap_tests.rs"]
mod tests;

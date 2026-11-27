//! The sign of each line of an open file, from its `-U0` patch against the change base (plan-00008, D2). Scouter's code
//! view has no signs (`src/reader/code-view.ts:99-108`), so the gutter is an approved deviation (`architecture.md`).
//! Pure: `services/git.rs` makes the patch.

use crate::theme::{Hex, Palette};

use super::Patch;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sign {
    Added,
    Modified,
    /// A deletion after line `n`, drawn on line `n`.
    DeletedBelow,
    /// A deletion before line 1, drawn on line 1.
    DeletedTop,
}

impl Sign {
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Added | Self::Modified => "▎",
            Self::DeletedBelow => "▁",
            Self::DeletedTop => "▔",
        }
    }

    pub fn color(self, p: &Palette) -> Hex {
        match self {
            Self::Added => p.added,
            Self::Modified => p.modified,
            Self::DeletedBelow | Self::DeletedTop => p.removed,
        }
    }
}

/// The signs of a patch's hunks, by zero-based line of the new file, in hunk order. A patch with no text (a notice:
/// binary, too large, or no line change) has none.
pub fn signs(patch: &Patch) -> Vec<(u32, Sign)> {
    let Patch::Text { text, .. } = patch else { return Vec::new() };
    text.lines().filter(|line| line.starts_with("@@")).flat_map(hunk_signs).collect()
}

/// One hunk header's signs. The first lines of the new side are modified, as many as the old side had, and the rest are
/// added. A hunk with no new lines is a deletion after its line, the line before the old side's start.
fn hunk_signs(header: &str) -> Vec<(u32, Sign)> {
    let mut fields = header.split_whitespace().skip(1);
    let (Some(old), Some(new)) = (fields.next().and_then(|f| range(f, '-')), fields.next().and_then(|f| range(f, '+')))
    else {
        return Vec::new();
    };
    let (old_count, (start, count)) = (old.1, new);
    if count == 0 {
        return vec![if start == 0 { (0, Sign::DeletedTop) } else { (start - 1, Sign::DeletedBelow) }];
    }
    let first = start.max(1) - 1;
    let modified = old_count.min(count);
    (0..count).map(|offset| (first + offset, if offset < modified { Sign::Modified } else { Sign::Added })).collect()
}

/// A hunk range field such as `-4,2` or `+7`: its start and its count, which is 1 when omitted.
fn range(field: &str, sign: char) -> Option<(u32, u32)> {
    let field = field.strip_prefix(sign)?;
    match field.split_once(',') {
        Some((start, count)) => Some((start.parse().ok()?, count.parse().ok()?)),
        None => Some((field.parse().ok()?, 1)),
    }
}

#[cfg(test)]
#[path = "signs_tests.rs"]
mod tests;

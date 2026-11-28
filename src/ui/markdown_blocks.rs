//! The source's top-level Markdown blocks with their source lines. OpenTUI's Markdown keeps its own block
//! list and shows no block's source lines, so the lines come from tree-sitter-md, which parses the same source. The
//! `tree_sitter` crate is reached through `opentui::syntax::tree_sitter`, as `ui/syntax` reaches it, so this module sits
//! in the UI layer rather than the core.

use std::ops::Range;

use opentui::syntax::tree_sitter::{Node, Parser};

/// A top-level block: its tree-sitter kind and its zero-based source lines, `lines.start..lines.end`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub kind: String,
    pub lines: Range<usize>,
}

/// The blocks OpenTUI draws, and the lines of the link reference definitions it draws nothing for.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Split {
    pub blocks: Vec<Block>,
    pub definitions: Vec<Range<usize>>,
}

/// Splits `source` as OpenTUI draws it. A section (a heading and the blocks under it) is not a block of its own, so its
/// children are. OpenTUI has no front matter: `---` at the top is a rule, so the leading blank line keeps the grammar
/// from reading one, and the rows are mapped back.
pub fn split(source: &str) -> Split {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_md::LANGUAGE.into()).is_err() {
        return Split::default();
    }
    let padded = format!("\n{source}");
    let Some(tree) = parser.parse(&padded, None) else { return Split::default() };
    let mut out = Split::default();
    collect(tree.root_node(), &mut out);
    for lines in out.blocks.iter_mut().map(|block| &mut block.lines).chain(out.definitions.iter_mut()) {
        *lines = lines.start - 1..lines.end - 1;
    }
    out
}

fn collect(node: Node<'_>, out: &mut Split) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "section" => collect(child, out),
            "link_reference_definition" => out.definitions.push(lines_of(child)),
            _ => out.blocks.push(Block { kind: child.kind().to_owned(), lines: lines_of(child) }),
        }
    }
}

fn lines_of(child: Node<'_>) -> Range<usize> {
    let (start, end) = (child.start_position(), child.end_position());
    // A node that ends at column 0 ends with its newline, so its last line is the one above.
    let last = if end.column == 0 && end.row > start.row { end.row - 1 } else { end.row };
    start.row..last + 1
}

#[cfg(test)]
#[path = "markdown_blocks_tests.rs"]
mod tests;

//! The source's top-level Markdown blocks with their source lines (plan-00010 phase 1). OpenTUI's Markdown keeps its own
//! block list and shows no block's source lines, so the lines come from tree-sitter-md, which parses the same source.
//! The `tree_sitter` crate is reached through `opentui::syntax::tree_sitter`, as `ui/syntax` reaches it, so this module
//! sits in the UI layer rather than the core.

use std::ops::Range;

use opentui::syntax::tree_sitter::{Node, Parser};

/// A top-level block: its tree-sitter kind and its zero-based source lines, `lines.start..lines.end`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub kind: String,
    pub lines: Range<usize>,
}

/// The top-level blocks of `source`, in order, as OpenTUI draws them. A section (a heading and the blocks under it) is
/// not a block of its own, so its children are. OpenTUI draws no block for a link reference definition, and it has no
/// front matter: `---` at the top is a rule, so the leading blank line keeps the grammar from reading one.
pub fn top_level(source: &str) -> Vec<Block> {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_md::LANGUAGE.into()).is_err() {
        return Vec::new();
    }
    let padded = format!("\n{source}");
    let Some(tree) = parser.parse(&padded, None) else { return Vec::new() };
    let mut blocks = Vec::new();
    collect(tree.root_node(), &mut blocks);
    for block in &mut blocks {
        block.lines = block.lines.start - 1..block.lines.end - 1;
    }
    blocks
}

fn collect(node: Node<'_>, out: &mut Vec<Block>) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "section" => collect(child, out),
            "link_reference_definition" => {}
            _ => push(child, out),
        }
    }
}

fn push(child: Node<'_>, out: &mut Vec<Block>) {
    let (start, end) = (child.start_position(), child.end_position());
    // A node that ends at column 0 ends with its newline, so its last line is the one above.
    let last = if end.column == 0 && end.row > start.row { end.row - 1 } else { end.row };
    out.push(Block { kind: child.kind().to_owned(), lines: start.row..last + 1 });
}

#[cfg(test)]
#[path = "markdown_blocks_tests.rs"]
mod tests;

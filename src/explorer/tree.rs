//! The tree of folders and their children (Scouter `src/explorer/file-tree.ts`). Every rule here is pure: a folder's
//! listing arrives as values, so a test builds the tree without a disk.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::{Entry, Kind};
use crate::find::text::Snippet;

struct Node {
    entry: Entry,
    parent: Option<PathBuf>,
    expanded: bool,
    /// `None` until the folder is listed; a listed folder keeps its children when collapsed.
    children: Option<Vec<PathBuf>>,
}

/// One visible row. `folder` is true for a directory and for any entry that has been listed (Scouter `paintRow`).
/// `hit` is a text hit's line and snippet, which only the finder's rows carry; `matched` is a fuzzy name match's
/// character positions in the path (empty for the tree and for a substring match).
pub struct Row<'a> {
    pub entry: &'a Entry,
    pub depth: usize,
    pub folder: bool,
    pub expanded: bool,
    pub hit: Option<(usize, &'a Snippet)>,
    pub matched: &'a [usize],
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expand {
    /// Expanded; the children were already listed.
    Ready,
    /// The folder must be listed first; call [`Tree::listed`] with its entries.
    NeedsListing,
    /// The path is not in the tree.
    Missing,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Reveal {
    /// List this folder, then call [`Tree::reveal`] again.
    Need(PathBuf),
    /// Every folder above the path is expanded, so the path has a row (if it exists).
    Done,
    /// The path is not under the root; the caller re-roots the tree at its parent.
    Outside,
}

pub struct Tree {
    root: PathBuf,
    nodes: HashMap<PathBuf, Node>,
    /// Whether dotfiles and the folders named with a dot show. Hidden, they and their children have no
    /// rows; the listing keeps them, so showing them again needs no read.
    dotfiles: bool,
}

/// A dotfile or a dot folder: a name that starts with `.`.
pub fn is_dotfile(name: &str) -> bool {
    name.starts_with('.')
}

impl Tree {
    pub fn new(root: PathBuf) -> Self {
        let name = root.file_name().map_or_else(|| root.display().to_string(), |n| n.to_string_lossy().into_owned());
        let entry = Entry { name, path: root.clone(), kind: Kind::Directory };
        let node = Node { entry, parent: None, expanded: false, children: None };
        Self { nodes: HashMap::from([(root.clone(), node)]), root, dotfiles: true }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Expands a folder whose children are listed, or says that it must be listed first.
    pub fn expand(&mut self, path: &Path) -> Expand {
        match self.nodes.get_mut(path) {
            None => Expand::Missing,
            Some(node) if node.children.is_none() => Expand::NeedsListing,
            Some(node) => {
                node.expanded = true;
                Expand::Ready
            }
        }
    }

    /// Stores a folder's sorted listing and expands it. Scouter's `load` and `expand` both end here. A folder that is
    /// listed again (a write in it, `app/write.rs`) keeps the children that stay, with their expansion.
    pub fn listed(&mut self, path: &Path, entries: Vec<Entry>) -> bool {
        if !self.nodes.contains_key(path) {
            return false;
        }
        self.replace_children(path, entries);
        if let Some(node) = self.nodes.get_mut(path) {
            node.expanded = true;
        }
        true
    }

    /// A fresh listing of an expanded folder (Scouter `FileTree.refresh`): a child that stays keeps its expansion and
    /// its own children, a child that is gone is forgotten, and a new one starts collapsed. Returns whether the rows
    /// changed; a folder that is not expanded and listed is left alone.
    pub fn refreshed(&mut self, path: &Path, entries: Vec<Entry>) -> bool {
        let Some(node) = self.nodes.get(path) else { return false };
        if !node.expanded || node.children.is_none() {
            return false;
        }
        let before = self.child_kinds(path);
        self.replace_children(path, entries);
        before != self.child_kinds(path)
    }

    /// The children of `path` with their kinds, in listing order.
    fn child_kinds(&self, path: &Path) -> Vec<(PathBuf, Kind)> {
        let children = self.nodes.get(path).and_then(|node| node.children.as_ref());
        children.into_iter().flatten().map(|child| (child.clone(), self.nodes[child].entry.kind)).collect()
    }

    /// Replaces the children of `path` with `entries`, as [`Tree::refreshed`] describes.
    fn replace_children(&mut self, path: &Path, entries: Vec<Entry>) {
        let old = self.nodes.get_mut(path).and_then(|node| node.children.take()).unwrap_or_default();
        let mut previous: HashMap<PathBuf, Node> =
            old.iter().filter_map(|child| self.nodes.remove(child).map(|node| (child.clone(), node))).collect();
        let mut children = Vec::with_capacity(entries.len());
        for entry in entries {
            let child = entry.path.clone();
            let node = match previous.remove(&child) {
                Some(existing) if existing.entry.kind == entry.kind => Node { entry, ..existing },
                Some(existing) => {
                    self.forget_children(&existing);
                    Node { entry, parent: Some(path.to_path_buf()), expanded: false, children: None }
                }
                None => Node { entry, parent: Some(path.to_path_buf()), expanded: false, children: None },
            };
            self.nodes.insert(child.clone(), node);
            children.push(child);
        }
        for gone in previous.values() {
            self.forget_children(gone);
        }
        if let Some(node) = self.nodes.get_mut(path) {
            node.children = Some(children);
        }
    }

    /// The folders that are expanded, root included (Scouter `expandedPaths`).
    pub fn expanded(&self) -> Vec<PathBuf> {
        self.nodes
            .values()
            .filter(|node| node.expanded && node.children.is_some())
            .map(|node| node.entry.path.clone())
            .collect()
    }

    pub fn collapse(&mut self, path: &Path) {
        if let Some(node) = self.nodes.get_mut(path) {
            node.expanded = false;
        }
    }

    /// Whether the folder's children are in the tree (a folder that was never expanded is not listed).
    pub fn is_listed(&self, path: &Path) -> bool {
        self.nodes.get(path).is_some_and(|node| node.children.is_some())
    }

    pub fn is_expanded(&self, path: &Path) -> bool {
        self.nodes.get(path).is_some_and(|node| node.expanded)
    }

    /// The entry at `path`, if it is in the tree.
    pub fn entry(&self, path: &Path) -> Option<&Entry> {
        self.nodes.get(path).map(|node| &node.entry)
    }

    pub fn dotfiles(&self) -> bool {
        self.dotfiles
    }

    pub fn set_dotfiles(&mut self, shown: bool) {
        self.dotfiles = shown;
    }

    /// The visible rows, depth first. The root is not a row: its children are the top level, at depth 0.
    pub fn rows(&self) -> Vec<Row<'_>> {
        let mut rows = Vec::new();
        if let Some(children) = self.nodes.get(&self.root).and_then(|node| node.children.as_ref()) {
            for child in children.iter().filter(|child| self.shows(child)) {
                self.visit(child, 0, &mut rows);
            }
        }
        rows
    }

    /// Whether a child has a row: a dotfile does only while dotfiles show.
    fn shows(&self, path: &Path) -> bool {
        self.dotfiles || !self.nodes.get(path).is_some_and(|node| is_dotfile(&node.entry.name))
    }

    fn visit<'a>(&'a self, path: &Path, depth: usize, rows: &mut Vec<Row<'a>>) {
        let Some(node) = self.nodes.get(path) else { return };
        rows.push(Row {
            entry: &node.entry,
            depth,
            folder: node.entry.kind == Kind::Directory || node.children.is_some(),
            expanded: node.expanded,
            hit: None,
            matched: &[],
        });
        if node.expanded
            && let Some(children) = &node.children
        {
            for child in children.iter().filter(|child| self.shows(child)) {
                self.visit(child, depth + 1, rows);
            }
        }
    }

    /// The folder above `path` when that folder has a row: `None` for a top-level row, whose parent is the root.
    pub fn parent_row(&self, path: &Path) -> Option<PathBuf> {
        self.nodes.get(path)?.parent.clone().filter(|parent| *parent != self.root)
    }

    /// The folders above `path` in the tree, nearest first, up to the root (Scouter's `rejectLinkLoop` walk).
    pub fn ancestors(&self, path: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        let mut at = self.nodes.get(path).and_then(|node| node.parent.clone());
        while let Some(parent) = at {
            at = self.nodes.get(&parent).and_then(|node| node.parent.clone());
            out.push(parent);
        }
        out
    }

    /// One step toward showing `path`: the first folder above it that is not listed yet, or `Done` once every
    /// folder above it is expanded. Scouter's `expandTo` (`file-tree.ts:76-81`) stops one level short and leaves
    /// the file's own parent collapsed; this expands that parent too, so the file has a row to select.
    pub fn reveal(&mut self, path: &Path) -> Reveal {
        let Ok(rest) = path.strip_prefix(&self.root) else { return Reveal::Outside };
        let parts: Vec<&OsStr> = rest.iter().collect();
        let mut folder = self.root.clone();
        for (index, part) in parts.iter().enumerate() {
            let Some(node) = self.nodes.get_mut(&folder) else { return Reveal::Done };
            if node.children.is_none() {
                return Reveal::Need(folder);
            }
            node.expanded = true;
            if index + 1 == parts.len() {
                break;
            }
            folder.push(part);
        }
        Reveal::Done
    }

    /// Drops a node that left the listing, with the nodes below it.
    fn forget_children(&mut self, node: &Node) {
        node.children.iter().flatten().for_each(|child| self.forget(child));
    }

    fn forget(&mut self, path: &Path) {
        if let Some(node) = self.nodes.remove(path) {
            node.children.iter().flatten().for_each(|child| self.forget(child));
        }
    }
}

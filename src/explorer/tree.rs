//! The tree of folders and their children (Scouter `src/explorer/file-tree.ts`). Every rule here is pure: a folder's
//! listing arrives as values, so a test builds the tree without a disk.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::{Entry, Kind};

struct Node {
    entry: Entry,
    depth: usize,
    parent: Option<PathBuf>,
    expanded: bool,
    /// `None` until the folder is listed; a listed folder keeps its children when collapsed.
    children: Option<Vec<PathBuf>>,
}

/// One visible row. `folder` is true for a directory and for any entry that has been listed (Scouter `paintRow`).
pub struct Row<'a> {
    pub entry: &'a Entry,
    pub depth: usize,
    pub folder: bool,
    pub expanded: bool,
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
}

impl Tree {
    pub fn new(root: PathBuf) -> Self {
        let name = root.file_name().map_or_else(|| root.display().to_string(), |n| n.to_string_lossy().into_owned());
        let entry = Entry { name, path: root.clone(), kind: Kind::Directory };
        let node = Node { entry, depth: 0, parent: None, expanded: false, children: None };
        Self { nodes: HashMap::from([(root.clone(), node)]), root }
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

    /// Stores a folder's sorted listing and expands it. Scouter's `load` and `expand` both end here.
    pub fn listed(&mut self, path: &Path, entries: Vec<Entry>) -> bool {
        let Some(depth) = self.nodes.get(path).map(|node| node.depth) else { return false };
        if let Some(old) = self.nodes.get_mut(path).and_then(|node| node.children.take()) {
            old.iter().for_each(|child| self.forget(child));
        }
        let children = entries
            .into_iter()
            .map(|entry| {
                let child = entry.path.clone();
                let node =
                    Node { entry, depth: depth + 1, parent: Some(path.to_path_buf()), expanded: false, children: None };
                self.nodes.insert(child.clone(), node);
                child
            })
            .collect();
        if let Some(node) = self.nodes.get_mut(path) {
            node.children = Some(children);
            node.expanded = true;
        }
        true
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

    /// The visible rows, depth first: the root, then the children of each expanded folder.
    pub fn rows(&self) -> Vec<Row<'_>> {
        let mut rows = Vec::new();
        self.visit(&self.root, &mut rows);
        rows
    }

    fn visit<'a>(&'a self, path: &Path, rows: &mut Vec<Row<'a>>) {
        let Some(node) = self.nodes.get(path) else { return };
        rows.push(Row {
            entry: &node.entry,
            depth: node.depth,
            folder: node.entry.kind == Kind::Directory || node.children.is_some(),
            expanded: node.expanded,
        });
        if node.expanded
            && let Some(children) = &node.children
        {
            for child in children {
                self.visit(child, rows);
            }
        }
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
    /// the file's own parent collapsed; this expands that parent too (the approved deviation in `architecture.md`).
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

    fn forget(&mut self, path: &Path) {
        if let Some(node) = self.nodes.remove(path) {
            node.children.iter().flatten().for_each(|child| self.forget(child));
        }
    }
}

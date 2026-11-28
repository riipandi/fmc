//! The grammars whose parser is vendored under `vendor/grammars/<name>/` and compiled by `build.rs`. Each C function
//! returns the grammar's language, and its symbol is unique in the binary: the crates linked here export other names.

use opentui::syntax::tree_sitter::Language;

unsafe extern "C" {
    fn tree_sitter_vimdoc() -> *const ();
    fn tree_sitter_luadoc() -> *const ();
    fn tree_sitter_luap() -> *const ();
    fn tree_sitter_just() -> *const ();
}

/// Wraps a grammar's language function. `tree-sitter-language` is not a direct dependency, so its `LanguageFn` cannot be
/// named here; the type is inferred from `Language::new`, which clippy's transmute lint cannot see.
#[allow(clippy::missing_transmute_annotations)]
fn language(function: unsafe extern "C" fn() -> *const ()) -> Language {
    // SAFETY: `LanguageFn` is `#[repr(transparent)]` over `unsafe extern "C" fn() -> *const ()` (tree-sitter-language
    // 0.1.9, `src/language.rs`), so the two types have the same layout and the same valid values. The function is a
    // grammar's language function, as `build.rs` compiles it from the grammar's `parser.c`.
    Language::new(unsafe { std::mem::transmute(function) })
}

pub fn vimdoc() -> Language {
    language(tree_sitter_vimdoc)
}

pub fn luadoc() -> Language {
    language(tree_sitter_luadoc)
}

pub fn luap() -> Language {
    language(tree_sitter_luap)
}

pub fn just() -> Language {
    language(tree_sitter_just)
}

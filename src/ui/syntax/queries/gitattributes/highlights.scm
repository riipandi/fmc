; From konomanoasa-tree-sitter-gitattributes 0.8.2 (MIT, LICENSE beside this file), the query its crate ships, with one change: the
; wildcards are string.special (character.special has no style).

[
  (comment_marker)
  (comment_text)
] @comment

[
  (glob_literal)
  (set_text)
  (range_character)
] @string.special.path

[
  (quote_open)
  (quote_close)
  (value_text)
] @string

[
  (escape)
  (quoted_escape)
] @string.escape

[
  (unset_marker)
  (unspecified_marker)
  (set_negation)
  (range_operator)
  (assignment_operator)
] @operator

[
  (wildcard)
  (recursive_wildcard)
  (single_character)
] @string.special

[
  (path_separator)
  ":"
] @punctuation.delimiter

[
  (set_open)
  (set_close)
  "["
  "]"
] @punctuation.bracket

(class_name) @type

(macro_marker) @keyword

(name_text) @attribute

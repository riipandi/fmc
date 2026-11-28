; From tree-sitter-luap (tree-sitter-grammars/tree-sitter-luap c134aae, MIT, LICENSE beside this file), queries/highlights.scm.
; adapted from https://github.com/vhyrro/tree-sitter-luap (the upstream query this one came from).
; Changes for OpenTUI: the wildcard dot is constant builtin (character is not a style the palette has), and the balanced
; match's parameter is variable parameter.

(".") @constant.builtin

[
  (anchor_begin)
  (anchor_end)
] @string.escape

[
  "[" "]"
  "(" ")"
] @punctuation.bracket

[
  (zero_or_more)
  (shortest_zero_or_more)
  (one_or_more)
  (zero_or_one)
] @operator

(range
  from: (character) @constant
  "-" @punctuation.delimiter
  to: (character) @constant)

(set
  (character) @constant)

(class) @keyword

(negated_set
  "^" @string.regex
  (character) @constant)

(balanced_match
  (character) @variable.parameter)

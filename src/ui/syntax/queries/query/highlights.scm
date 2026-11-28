; From tree-sitter-tsquery 0.8.0 (Apache-2.0, LICENSE beside this file), the query its crate ships, with three changes for OpenTUI.
; `#lua-match?` is `#match?` (the binding ignores the Lua form); the spell captures are dropped (fmc has no spell check);
; the wildcard `_` is `constant.builtin` (`character` is not a style the palette has).

(string) @string

(escape_sequence) @string.escape

(capture
  (identifier) @type)

(predicate
  name: (identifier) @function.call)

(named_node
  name: (identifier) @variable)

(missing_node
  name: (identifier) @variable)

(field_definition
  name: (identifier) @variable.member)

(negated_field
  "!" @operator
  (identifier) @property)

(comment) @comment

(quantifier) @operator

(predicate_type) @punctuation.special

"." @operator

[
  "["
  "]"
  "("
  ")"
] @punctuation.bracket

[
  ":"
  "/"
] @punctuation.delimiter

[
  "@"
  "#"
] @punctuation.special

(predicate
  "." @punctuation.special)

"_" @constant.builtin

"MISSING" @keyword

((parameters
  (identifier) @number)
  (#match? @number "^[-+]?[0-9]+(.[0-9]+)?$"))

((program
  .
  (comment)*
  .
  (comment) @keyword.import)
  (#match? @keyword.import "^;+ *inherits *:"))

((program
  .
  (comment)*
  .
  (comment) @keyword.directive)
  (#match? @keyword.directive "^;+ *extends *$"))

((comment) @keyword.directive
  (#match? @keyword.directive "^;+[ \t]*format-ignore[ \t]*$"))

((predicate
  name: (identifier) @_name
  parameters: (parameters
    .
    (capture)?
    .
    (identifier) @property))
  (#eq? @_name "set"))

((predicate
  name: (identifier) @_name
  parameters: (parameters
    (string
      "\"" @string
      "\"" @string) @string.regexp))
  (#any-of? @_name "match" "not-match" "vim-match" "not-vim-match" "lua-match" "not-lua-match"))

((predicate
  name: (identifier) @_name
  parameters: (parameters
    (string
      "\"" @string
      "\"" @string) @string.regexp
    .
    (string) .))
  (#any-of? @_name "gsub" "not-gsub"))

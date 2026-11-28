; From tree-sitter-properties 0.3.0 (MIT, LICENSE beside this file, from the upstream repository: the crate ships
; none), the query its crate ships, with one change: a substitution's secret is variable (embedded has no style).

(comment) @comment

(key) @property

(value) @string

(value (escape) @string.escape)

((index) @number
  (#match? @number "^[0-9]+$"))

((substitution (key) @constant)
  (#match? @constant "^[A-Z0-9_]+"))

(substitution
  (key) @function
  "::" @punctuation.special
  (secret) @variable)

(property [ "=" ":" ] @operator)

[ "${" "}" ] @punctuation.special

(substitution ":" @punctuation.special)

[ "[" "]" ] @punctuation.bracket

[ "." "\\" ] @punctuation.delimiter

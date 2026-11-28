; From tree-sitter-luadoc (tree-sitter-grammars/tree-sitter-luadoc 4d04632, MIT, LICENSE beside this file), queries/highlights.scm.
; Changes for OpenTUI: the include capture is keyword import, the uri capture is string special url, the parameter and
; field captures are variable parameter and variable member, and the spell captures are dropped (fmc has no spell check).

; Keywords
[
  "@module"
  "@package"
] @keyword.import

[
  "@class"
  "@type"
  "@param"
  "@alias"
  "@field"
  "@generic"
  "@vararg"
  "@diagnostic"
  "@deprecated"
  "@meta"
  "@source"
  "@version"
  "@operator"
  "@nodiscard"
  "@cast"
  "@overload"
  "@enum"
  "@language"
  "@see"
  "extends"
  (diagnostic_identifier)
] @keyword

"@async" @keyword.coroutine

(language_injection
  "@language"
  (identifier) @keyword)

(function_type
  [
    "fun"
    "function"
  ] @keyword.function)

(source_annotation
  filename: (identifier) @string.special.url
  extension: (identifier) @string.special.url)

(version_annotation
  version: _ @constant.builtin)

"@return" @keyword.return

; Qualifiers
[
  "public"
  "protected"
  "private"
  "@public"
  "@protected"
  "@private"
] @type.qualifier

; Variables
(identifier) @variable

[
  "..."
  "self"
] @variable.builtin

; Macros
(alias_annotation
  (identifier) @function.macro)

; Parameters
(param_annotation
  (identifier) @variable.parameter)

(parameter
  (identifier) @variable.parameter)

; Fields
(field_annotation
  (identifier) @variable.member)

(table_literal_type
  field: (identifier) @variable.member)

(member_type
  [
    "#"
    "."
  ]
  .
  (identifier) @variable.member)

; Types
(table_type
  "table" @type.builtin)

(builtin_type) @type.builtin

(class_annotation
  (identifier) @type)

(enum_annotation
  (identifier) @type)

((array_type
  [
    "["
    "]"
  ] @type)
  (#set! "priority" 105))

(type) @type

; Operators
"|" @operator

; Literals
(string) @namespace ; only used in @module

(literal_type) @string

(number) @number

; Punctuation
[
  "["
  "]"
] @punctuation.bracket

[
  "{"
  "}"
] @punctuation.bracket

[
  "("
  ")"
] @punctuation.bracket

[
  "<"
  ">"
] @punctuation.bracket

[
  ","
  "."
  "#"
  ":"
] @punctuation.delimiter

[
  "@"
  "?"
] @punctuation.special

; Comments
(comment) @comment

(at_comment
  (identifier) @type
  (_) @comment)

(class_at_comment
  (identifier) @type
  ("extends"?
    (identifier)? @type)
  (_) @comment)

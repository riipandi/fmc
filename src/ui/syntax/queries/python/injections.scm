; fmc's own (MIT, the project's licence): tree-sitter-python 0.25.0 ships no injections query. The first argument of
; `re`'s functions is a pattern, so a string there is regex. `include-children` keeps the escapes, which are child nodes
; of the string content and are part of the pattern.

((call
  function: (attribute
    object: (identifier) @_module
    attribute: (identifier) @_function)
  arguments: (argument_list
    .
    (string
      (string_content) @injection.content)))
  (#eq? @_module "re")
  (#any-of? @_function "compile" "findall" "finditer" "fullmatch" "match" "search" "split" "sub")
  (#set! injection.language "regex")
  (#set! injection.include-children))

; From nvim-treesitter (nvim-treesitter/nvim-treesitter 04f1788, Apache-2.0, LICENSE.nvim-treesitter beside this file), runtime/queries/lua/injections.scm.
; Kept: the rule for the pattern argument of find, match, gmatch, and gsub (luap). Changes for OpenTUI: the rule's
; directives are the ones OpenTUI evaluates. The other rules of the upstream file (C and Vim code, format strings, filetype
; patterns, and generic comments) name languages or cases outside this file, and are left out.
; shortcut: the upstream rule for `---` documentation comments (luadoc) is left out. Its text keeps the comment's leading
; dash, which the luadoc grammar does not parse, and OpenTUI does not evaluate the offset directive that would drop it.
; Add the rule when OpenTUI evaluates offset directives.

(function_call
  (dot_index_expression
    field: (identifier) @_method
    (#any-of? @_method "find" "match" "gmatch" "gsub"))
  arguments: (arguments
    .
    (_)
    .
    (string
      content: (string_content) @injection.content
      (#set! injection.language "luap")
      (#set! injection.include-children))))

(function_call
  (method_index_expression
    method: (identifier) @_method
    (#any-of? @_method "find" "match" "gmatch" "gsub"))
  arguments: (arguments
    .
    (string
      content: (string_content) @injection.content
      (#set! injection.language "luap")
      (#set! injection.include-children))))

; Adapted verbatim from tree-sitter-html 0.23.2, queries/injections.scm (MIT; the LICENSE beside this file).
; The script and style bodies are the only injections the crate ships.

((script_element
  (raw_text) @injection.content)
 (#set! injection.language "javascript"))

((style_element
  (raw_text) @injection.content)
 (#set! injection.language "css"))

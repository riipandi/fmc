; Adapted from tree-sitter-svelte-ng 1.0.2, queries/injections.scm (MIT; the LICENSE beside this file).
; Changes: the crate's `; inherits: html_tags` is not resolved here, so the HTML rules are written out; its generic
; `(raw_text)` rule matches a <style> body too, so the bodies have one rule each; its scss and pug rules are left out,
; because neither language is registered (a rule for an unknown language is skipped anyway).

((script_element
  (start_tag) @_start
  (raw_text) @injection.content)
  (#not-match? @_start "lang=")
  (#set! injection.language "javascript"))

((script_element
  (start_tag
    (attribute
      (attribute_name) @_attr
      (quoted_attribute_value
        (attribute_value) @_lang)))
  (raw_text) @injection.content)
  (#eq? @_attr "lang")
  (#any-of? @_lang "ts" "typescript")
  (#set! injection.language "typescript"))

((style_element
  (start_tag) @_start
  (raw_text) @injection.content)
  (#not-match? @_start "lang=")
  (#set! injection.language "css"))

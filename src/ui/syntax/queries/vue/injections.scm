; Adapted from tree-sitter-vue-next 0.1.0, queries/vue/injections.scm (MIT; the LICENSE beside this file).
; Changes: the crate's `; inherits: html_tags` is not resolved here, so the plain <script> and <style> rules are written
; out as the Svelte injections write them; `less`, `postcss`, and `sass` styles are read as SCSS, as upstream does; the
; pug rule is left out, because pug is not registered.

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
  (#any-of? @_lang "js" "jsx")
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

((script_element
  (start_tag
    (attribute
      (attribute_name) @_attr
      (quoted_attribute_value
        (attribute_value) @_lang)))
  (raw_text) @injection.content)
  (#eq? @_attr "lang")
  (#eq? @_lang "tsx")
  (#set! injection.language "typescriptreact"))

((style_element
  (start_tag) @_start
  (raw_text) @injection.content)
  (#not-match? @_start "lang=")
  (#set! injection.language "css"))

((style_element
  (start_tag
    (attribute
      (attribute_name) @_attr
      (quoted_attribute_value
        (attribute_value) @_lang)))
  (raw_text) @injection.content)
  (#eq? @_attr "lang")
  (#eq? @_lang "css")
  (#set! injection.language "css"))

((style_element
  (start_tag
    (attribute
      (attribute_name) @_attr
      (quoted_attribute_value
        (attribute_value) @_lang)))
  (raw_text) @injection.content)
  (#eq? @_attr "lang")
  (#any-of? @_lang "scss" "less" "postcss" "sass")
  (#set! injection.language "scss"))

((interpolation
  (raw_text) @injection.content)
  (#set! injection.language "typescript"))

(directive_attribute
  (quoted_attribute_value
    (attribute_value) @injection.content
    (#set! injection.language "typescript")))

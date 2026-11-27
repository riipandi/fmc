; Adapted from tree-sitter-javascript 0.25.0, queries/injections.scm (MIT; the LICENSE beside this file). The regex rule is the
; crate's. The JSDoc rule takes only `/**` comments: the crate's rule parses every comment as JSDoc, and JSDoc's own syntax
; is the `/**` form. The tagged-template rule is left out (its tag names the language, which is not asked for), and so is
; the Handlebars rule (its `#offset!` is not evaluated). TypeScript and TSX use this file too, as OpenTUI binds it.

((regex_pattern) @injection.content
  (#set! injection.language "regex"))

((comment) @injection.content
  (#match? @injection.content "^/[*][*]")
  (#set! injection.language "jsdoc"))

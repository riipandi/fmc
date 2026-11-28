; Adapted from tree-sitter-php 0.25.2, queries/injections.scm and queries/injections-text.scm (MIT; the LICENSE beside
; this file). Changes: the text outside `<?php` is HTML, the crate's injections-text rule, written here; the phpdoc rule
; is left out, because phpdoc is not registered. A heredoc's body takes the language its closing label names
; (`<<<SQL` … `SQL`).

((text) @injection.content
  (#set! injection.language "html")
  (#set! injection.combined))

(heredoc
  (heredoc_body) @injection.content
  (heredoc_end) @injection.language)

(nowdoc
  (nowdoc_body) @injection.content
  (heredoc_end) @injection.language)

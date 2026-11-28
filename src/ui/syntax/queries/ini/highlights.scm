; From tree-sitter-ini 1.4.0 (Apache-2.0, LICENSE beside this file), the query its crate ships, with one change: the
; spell capture is dropped (fmc has no spell check).

(section_name
  (text) @type) ; consistency with toml

(comment) @comment

[
  "["
  "]"
] @punctuation.bracket

"=" @operator

(setting
  (setting_name) @property)

; (setting_value) @none ; grammar does not support subtypes

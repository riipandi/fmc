; fmc: Core's injections (Scouter's markdown source) use #set-lang-from-info-string! and #offset!, which the binding does not
; evaluate, and inline text needs its punctuation kept (rust.md). This is the binding's own MARKDOWN_INJECTIONS (opentui-0.9.9,
; src/syntax/highlight.rs), which it applies to the same grammar.

(fenced_code_block
  (info_string (language) @injection.language)
  (code_fence_content) @injection.content
  (#set! injection.include-unnamed-children))
((html_block) @injection.content (#set! injection.language "html"))
((minus_metadata) @injection.content (#set! injection.language "yaml"))
((plus_metadata) @injection.content (#set! injection.language "toml"))
([(inline) (pipe_table_cell)] @injection.content
  (#set! injection.language "markdown_inline")
  (#set! injection.include-unnamed-children))

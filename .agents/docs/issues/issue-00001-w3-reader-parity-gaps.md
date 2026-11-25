---
status: open
created: 2026-11-26T05:45:28+07:00
updated: 2026-11-26T05:45:28+07:00
summary: "Two findings from the W3 phase 11 live runs: highlight colours for JavaScript, TypeScript, TSX, JSX, and Zig differ from Scouter (Core bundles its own queries), and a refused file at startup leaves Scouter's tree unloaded where fmc shows the folder. The count matches the items listed."
count: 2
---

# W3 reader parity gaps

Captures for both items are in `/tmp/fmc-ref/out/w3-11/` (outside the repository): `fmc-<name>-00.txt` and `.html` beside `sco-<name>-00.*`, 80x24, Scouter 0.9.9 as the reference.

## Highlight queries for JavaScript, TypeScript, TSX, JSX, and Zig differ from Scouter

- status: open (the owner's choice is in `architecture.md`, "Pending decisions")
  area: reader, syntax highlighting (`src/ui/syntax/mod.rs`)
  summary: Scouter highlights these five with the queries Core ships, `node_modules/@opentui/core/assets/{javascript,typescript,typescriptreact,zig}/highlights.scm` in the 0.9.9 reference (`typescript/highlights.scm:10` names a property `@variable.member`, which Scouter's table draws as plain text). OpenTUI's Rust binding bundles the grammar crates' own queries instead (`opentui-0.9.9/src/syntax/highlight.rs`, `bundled()`), and the property is `@property` there, drawn in the function colour. The binding can register a query only with the language handle, and the handles for these grammars come from crates fmc does not depend on directly. Differing cells: TypeScript 60, TSX 19, JSX 4, Zig 16 (`fmc-ts-00.html` against `sco-ts-00.html`, and the rest).
  plan: plan-00001 phase 11 (its colour validation) and phase 12 (Markdown uses Core's `markdown` and `markdown_inline` assets)
  trigger: the owner chooses an option in `architecture.md` "Pending decisions"; nothing else fires it.

## A refused file at startup leaves Scouter's tree unloaded; fmc shows the folder

- status: open
  area: reader, startup (`src/app/mod.rs` `run`, `src/app/tree.rs` `advance`)
  summary: Scouter reads a file before it reveals it (`scouter:src/app/app.ts:330-335`: `readDocumentWithStamp`, then `reveal`), so a file refused at startup (non-UTF-8, `latin.txt`) leaves the sidebar empty under the `Files` header. fmc lists the parent folder and selects the file before it reads, so the sidebar shows the folder and the file's row (`fmc-latin-00.txt`). The footer and the reader's notice match Scouter.
  plan: plan-00001 phase 23 (a difference found in a live run is fixed there, or recorded here)
  trigger: plan-00001 phase 23, or the owner's decision to accept the difference.

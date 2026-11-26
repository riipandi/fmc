---
status: open
created: 2026-11-26T08:57:16+07:00
updated: 2026-11-26T08:57:16+07:00
summary: "The file tree does not scroll with the mouse wheel: the next frame scrolls it back to the selected row, so each wheel notch undoes itself. Scouter's tree scrolls one row per notch. The HEAD build (before plan-00001 phase 19) does the same; the changes list had the same defect and phase 19 fixed it there."
count: 1
---

# The tree's wheel scroll is undone by the next frame

## The tree stays on its selected row under the wheel

- status: open
  area: explorer (`src/explorer/browser.rs` `set_height`, `keep_visible`), tree view (`src/app/mod.rs` `paint`)
  summary: `Browser::set_height` calls `keep_visible`, which moves the scroll back to the selected row on every paint. A wheel notch (`Browser::scroll`) moves the viewport, and the next frame, which calls `set_height` with the same height, brings it back. Live, 80x24, on a 30-row folder: three wheel-down reports left the tree on `hogwarts-01.txt` in the HEAD build (`/tmp/fmc-ref/out/w5-19/issues/head-tree-wheel-0.txt` and `head-tree-wheel-3down.txt`). Scouter 0.9.9 on the same folder scrolls by the wheel (`scouter-tree-wheel-0.txt` and `scouter-tree-wheel-3down.txt`, which show rows 3 and 4 at the top after three notches).
  plan: plan-00001 parity fix. The change list in phase 19 fixed the same defect by making `set_height` only clamp the scroll (`src/diff/list.rs`, with a test); the tree's `set_height` should do the same, with a test that a repaint keeps a wheel scroll.
  trigger: the next change to the tree's list, or the parity audit (phase 22) if it reaches the tree's mouse first

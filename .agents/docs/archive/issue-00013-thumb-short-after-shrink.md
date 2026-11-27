---
status: resolved
created: 2026-11-27T14:00:00+07:00
updated: 2026-11-27T15:00:00+07:00
summary: "After the terminal gets shorter, the scrollbar thumbs in the sidebar and the reader stop rows above the bottom, though the content is at its end. Cells left over from the taller frame stay visible and squeeze the bar's column."
count: 1
---

# Scrollbar thumbs stop short after the terminal shrinks

## Leftover thumb cells keep their rows

- status: resolved
  area: `src/ui/tree.rs` `TreeView::paint` (the thumb cells in `bar`), `src/ui/scrollbar.rs` `ScrollBar::paint` (the cells in `root`)
  summary: |
  Owner report, 2026-11-27, with a screenshot: in the wide tier at about 60 rows, both thumbs end about 4 rows above the panel's bottom while the tree shows its last entry and the reader shows `Bot`.

  Reproduced in a Rex tab at 150 columns. Opening `src/diff/row.rs`, pressing `G` (and `End` in the tree), and resizing from 63 to 61, 59, and 56 rows gives these last thumb rows:

  | Rows | Last content row | Reader thumb ends | Sidebar thumb ends |
  |---|---|---|---|
  | 63 | 60 | 60 | 60 |
  | 61 | 58 | 58 | 58 |
  | 59 | 56 | 55 | 55 |
  | 56 | 53 | 49 | 47 |

  Growing the terminal draws the thumbs correctly. Wheel bursts, key motions, and thumb drags at a fixed size draw correctly too.

  Cause: both bars keep one `Text` cell per row and only add cells when the frame grows (`while cells.len() < height`). When the frame shrinks:

  - The tree empties the extra thumb cells but leaves them visible.
  - `ScrollBar` never hides its extra cells.

    The column then holds more one-row cells than it has rows, so the layout shrinks them and the thumb glyphs are squeezed upward. `ui/changes.rs` already hides its extra cells (`cell.visible(index < lines.len())`) and is not affected.

    Fix:
  - In the tree, make the used cells visible and the extra cells hidden.
  - In `ScrollBar::paint`, set `cell.visible(index < height)`.
  - Add a TestApp test that paints at a tall size, then a shorter one, and checks that the last thumb row is the last list row.

    The fix waits for plan-00007 to land, because that plan is editing `src/ui/tree.rs`.

  resolution: Fixed in `fix(scrollbar)`. The tree shows only the thumb cells its lines use and hides the rest, and `ScrollBar::paint` hides the cells past `height`. The tests `ui::scrollbar::tests::the_thumb_reaches_the_bottom_after_the_bar_gets_shorter` and `app::mouse_tests::the_tree_thumb_reaches_the_bottom_after_the_terminal_shrinks` fail without the fix.

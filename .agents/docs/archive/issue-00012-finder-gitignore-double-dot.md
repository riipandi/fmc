---
status: resolved
created: 2026-11-27T03:30:00+07:00
updated: 2026-11-27T12:00:00+07:00
summary: "A finder row once read `..gitignore` in a wide tab. It does not reproduce at 80x24, and a headless render of the same row at 160 columns shows `.gitignore` with one dot."
count: 1
---

# The finder row that reads `..gitignore`

## The root-level dotfile row shows two dots in a wide tab

- status: resolved
  area: finder name rows (`src/explorer/browser.rs` `Browser::show`, `src/ui/tree.rs` `row_segments`), the index (`src/services/find.rs`, `src/explorer/browser.rs` `relative`)
  summary: A previous run saw a finder row reading `..gitignore` in a wide tab (owner report, plan-00003 brief, 2026-11-27). Checked in plan-00003 phase 1: the row does not reproduce at 80x24 on the demo repository. A headless render (`TestApp` at 160x40, `Browser::show` with the flat row `.gitignore`) paints `▎   .gitignore` with one dot. The index's relative name is `.gitignore` (`relative` joins the parts with `/`), and the row text is the icon, a space, and the name, so the source of the second dot is not in these paths.
  plan: none. Next step is a live capture in a tab wider than 80 columns, which Rex did not allow: the pane follows the window, and `stty` before `exec` garbled the capture.
  trigger: a reproduction in a wide tab (the finder with `/`, showing `.gitignore` at the demo repository's root), or a wide run from the owner. Then fix it in its own commit and archive this issue.

Resolution (2026-11-27): the owner checked the finder in a wide terminal and saw `.gitignore` with one dot. Not reproduced by the owner, the executor, or the headless render; closed without a change.

---
status: in-progress
created: 2026-11-27T18:00:00+07:00
updated: 2026-11-27T03:51:33+07:00
phases:
    - id: 1
      title: "Tier model and touch-safe row actions"
      status: done
      note: "`layout::tier` (Compact, Medium, Wide) with tests at 39, 59, 60, 79, 80; every width caller asks it (`DRAWER` renamed `COMPACT`). Row presses act on release through the shared `Click` tracker (renamed from `HeaderClick`); tests drive the list through TestApp (click, drag, swipe back to its cell, release off the row). 328 tests. Agent decisions in architecture.md (plan-00005 list)."
    - id: 2
      title: "Compact navigation stack: Files and Reader pages, no border, start on Files"
      status: done
      note: "`fmc::layout::Pages` (Files, Reader, Changes, Keys, Find) in the core, kept by `app/pages.rs` `sync` at each paint; compact draws the top page full width, has no panel border, one-row headers, no welcome, no preview. `q`, `Esc`, and Back (`pages::back`) pop; Tab and Ctrl+/ switch list and reader. Drawer code removed. Captures at 60, 70, and 80 columns are byte-identical to 166051c (10 steps each); 40 columns checked live (Files, Reader, q, Esc, Find, Keys, Changes, diff). 331 tests. Agent decisions in architecture.md (plan-00005 list)."
    - id: 3
      title: "Compact top bar and bottom bar"
      status: done
      note: "`ui/bars.rs`: the bottom bar's equal cells per page (Find, Changes, Keys; Back, Search, Keys; Back, All/One, Split; Back), a prompt bar with Cancel first, and the top bar's back text (`‹ Files`, the page under the top). Notices show in the bottom bar. Taps press the segment's key; the back text pops. The compact footer is one row, the finder bar one row. Live at 40 columns: Files, Reader, Find, Changes, diff. Captures at 60 and 80 columns still match 166051c. Tests at 40, 49, 59 columns for the cells and the hits; labels per page. Agent decisions in architecture.md (plan-00005 list)."
    - id: 4
      title: "Medium tier chevrons; remove the boxes, drawer, and border marks below 80"
      status: done
      note: "At 60 to 79 columns `‹` starts the list's title (it hides the list, when a document is open) and `›` starts the reader's title (it shows the list, when hidden); one press handler per header acts by tier. The `≡`/`✕` boxes, the header-row tap, and the drawer code are gone (the drawer went in phase 2); the `«`/`»` marks were already not drawn below 80. `rust.md`, `keymaps.md`, `parity.md`, `README.md`, `DESIGN.md`, and CHANGELOG describe the chevrons. 80x24 captures identical to 166051c (10 steps). Live at 70 columns: `›` showed the list and `‹` hid it, both by SGR press. 347 tests."
    - id: 5
      title: "Touch delivery from the phone probe"
      status: pending
      note: —
    - id: 6
      title: "Docs, ledger close, handover, final validation"
      status: pending
      note: —
---

## Rebuild the narrow layouts as tiers: a phone-native compact tier and a cleaner medium tier

### Objective

On a real phone, touch navigation does nothing, and the drawer and box buttons feel clunky (owner screenshots, 2026-11-27). The owner delegated the redesign. The decisions are in `architecture.md`, "three layout tiers, and a phone-native compact tier" (`ba7c66f`), items 1 to 12. When this plan is done:

- Below 60 columns, fmc is a two-page browser with a top bar and a bottom bar.
- From 60 to 79 columns, it keeps the split view, with text chevrons.
- At 80 columns and wider, nothing changes.

### Current state (2026-11-27)

- The phone layout, the drawer, the menu and close boxes, the header-row tap, and the tappable footer hints come from plan-00004 and `6817f0c`. They live mainly in `src/app/layout.rs`, `src/ui/header.rs`, `src/ui/welcome.rs`, and the footer modules. The breakpoints are listed in `rust.md`.
- Row actions in the tree, the finder, and the change list fire on press. Only the diff header acts on release (plan-00004 phase 1).
- A raw-input probe runs in a tab of the owner's Rex session. It writes to `/tmp/fmc-probe/touch.log` and shows what Rex's phone client sends. The owner taps it from the phone.

### Decisions

These are the architecture record items 1 to 12; do not reopen them. Within them, the agent chose the following:

- **D1.** The tier is one pure function of the width (`fmc::layout::tier`), and every caller asks it. No raw width comparisons remain in `app` or `ui`.
- **D2.** A page stack in the core model (`Page::Files`, `Page::Reader`, plus the existing screens) drives compact. Medium and wide ignore it.
- **D3.** Bottom bar labels:
  - Files: `Find`, `Changes`, `Keys`
  - Reader: `Back`, `Search`, `Keys`
  - Changes: `Back`, `All`/`One`, `Split`
  - Keys: `Back`
  - Prompts: `Cancel`, then the prompt

  Each label names the action that `/`, `d`, `?`, `q`, `s`, and `a` already run.

### Execution rules

- One atomic commit per phase. Code commits carry `Scouter-Base: v0.3.4 (434d000f)`.
- Run `just verify` before each commit. Never amend. Stage explicit paths. Do not push. Add no dependencies. Keep files under 800 lines; split modules as needed.
- Captures at 80x24 must stay unchanged in every phase, apart from the row action moving to release, which changes timing only.
- Live runs happen only in new tabs of your own Rex session (`rex run -s "$REX_BLOCK" --focus=false -- …`). Resize them with `rex block call resize` to 40x30, 49x40, 59x30, 60x24, 70x24, and 80x24. Create no sessions and write no session names. Close your tabs and reset the demo repo afterwards.
- Leave the untracked `src/ui/probe_tests.rs` alone. Do not stop the probe tab: the owner is using it.

### Phases

1. **Tier model and touch-safe rows.**
   - Add `layout::tier(width)` with tests at 39, 59, 60, 79, and 80.
   - Tree, finder, and change-list rows act on release when the pointer has not moved, as the fold rule already does. A drag or swipe does nothing to them.
   - Add tests for the click and drag cases.
   - _Commit: `feat(layout): width tiers and row actions on release`._
2. **Compact stack.**
   - Below 60 columns there is no panel border and no header rule.
   - The start page is Files.
   - Opening a file (Enter, or a tap) pushes Reader. `q`, `Esc`, and Back pop it, keeping the list's selection and scroll.
   - Auto-preview is off.
   - Changes, Keys, and Find are pages too.
   - Add tests for push and pop, the kept selection, and the welcome not drawn.
   - _Commit: `feat(layout): a two-page browser on narrow screens`._
3. **Bars.**
   - The top bar shows `‹ Files`, then the title, then the tag.
   - The bottom bar has equal-width segments, with `·` separators drawn in the muted colour.
   - A prompt replaces the bottom bar while it is open.
   - Notices show in the bottom bar row for their usual time.
   - Every segment runs its key's action through the key table, as plan-00004 D2 does.
   - Add tests for the segment hit-testing at 40, 49, and 59 columns, and for the labels on each page.
   - _Commit: `feat(layout): top and bottom bars on narrow screens`._
4. **Medium tier.**
   - At 60–79 columns, the list title starts with `‹` (hide) and the reader title with `›` (show), in the accent colour.
   - A tap on the chevron toggles.
   - Remove the `≡`/`✕` boxes, the drawer code, the header-row tap, and the `»`/`«` marks below 80, along with their tests. The marks stay at 80 and wider.
   - Update `rust.md` breakpoints.
   - _Commit: `refactor(layout): chevrons in the medium tier, drop the phone boxes and drawer`._
5. **Touch delivery.**
   - Read `/tmp/fmc-probe/touch.log` (do not clear it). If it is empty, say so in the report and skip this phase's code. The owner will tap the probe later, and the coordinator will brief this phase again.
   - If it has data, adapt the event handling to what Rex's phone client sends. For example: a press with no release, a release with no press, wheel codes for swipes, or motion-only events. Add tests built from the logged bytes.
   - _Commit: `fix(input): handle touch as the phone client sends it`._
6. **Docs and validation.**
   - Update `docs/keymaps.md` (touch and the bars), `README.md`, `DESIGN.md` (the three tiers, with ASCII sketches of each), and `CHANGELOG.md`.
   - In `rust.md`, record the tier function and the page stack.
   - Archive this plan, write the handover, and update the index.
   - Run the full matrix.
   - _Commit: `docs: close the layout tiers plan`._

### Explicitly out of scope

- Any change at 80 columns and wider, other than the release timing of row actions.
- New actions in the bottom bar beyond the ones that already have keys.

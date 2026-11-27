---
status: in-progress
created: 2026-11-27T12:30:00+07:00
updated: 2026-11-27T11:47:21+07:00
references: archive/plan-00005-layout-tiers.md, architecture.md ("Three layout tiers", record 5)
phases:
    - id: 1
      title: "Compact pages get the panel border and the two-row header"
      status: done
      note: "`Sides::ALL` and the two-row header in every tier; `Header::compact` and `Find::compact` removed. The compact back chevron is `‹` (`pages::back_lead`), and a press on the lead or the title goes Back (`layout.rs` `header_press`); `back_text`, `page_name`, and `list_top` removed. Compact chrome is 5 rows. Captures at 60 and 80 columns are byte-identical to HEAD (10 steps each); 40 and 49 change. Live at 40 and 59 columns: Files, reader, `‹` back, Find, Changes, diff header fold, Keys, bottom bar taps, and the first and last rows of each list. TestApp row tests at 40 and 59 columns. Agent decisions in architecture.md (plan-00007 list, to be written in phase 2)."
    - id: 2
      title: "Docs, records, live checks, close"
      status: pending
---

## Give the compact tier the medium tier's border and header

### Objective

On a phone the compact tier looks plain next to the medium tier: it has no panel border, and its header is one row with no rule (owner screenshots, 2026-11-27: compact at about 50 columns next to medium). The owner wants both tiers to look the same. When this plan is done, every compact page draws inside the same rounded panel border as medium, under the same two-row header: a lead glyph, the title, the badges, and a hairline below. The page stack, the bottom bar, and the touch rules from plan-00005 stay.

This supersedes `architecture.md` "Three layout tiers", record 5 ("compact draws no panel border and one-row headers"), and the plan-00005 phase 3 note that the finder bar is one row.

### Current state (2026-11-27)

- `src/app/layout.rs` `lay_out` sets `Sides::NONE` on both panels in compact and calls `compact(true)` on the two headers and the finder bar.
- `src/ui/header.rs:104-107` and `src/ui/find.rs:39-42` `compact` switch to one row with no bottom rule.
- The compact lead text is `‹ Files` (or the name of the page underneath), from `app/pages.rs` `back_label` and `ui/bars.rs` `back_text`. In medium the lead is a lone chevron, `‹` or `›`, and the whole title box is the target (`cd22086`).
- The compact list hit test assumes the list starts on the row after the one-row bar (`architecture.md`, plan-00005 phase 3 decision). The tree, finder, and change-list press handlers map a row from `event.y`.
- The bottom bar (`ui/bars.rs`) sits below the panel, as the footer does in medium.

### Decisions

Owner, 2026-11-27: compact keeps a border and a header like medium.

Agent, within that:

- **D1.** All compact pages (Files, Reader, Changes, Keys, Find) draw the panel border with the medium tier's rounded sides and focus colour. In compact the visible page always has the focus, so its border is always the accent colour.
- **D2.** The pane headers and the finder bar are two rows with the hairline in every tier. The `compact` switch on `Header` and `Find` is removed (early-stage rule: no dead shape).
- **D3.** The compact back lead is the lone `‹` chevron, styled and spaced as medium's, and the whole title box is the target. The page name (`Files`, `Changes`) is dropped. Reasons: the owner's second screenshot shows the lone chevron; at 40 to 59 columns the six freed columns go to the path; and Back in the bottom bar still names the action. On the Files page, with nothing underneath, there is no lead, the same as medium with the list showing.
- **D4.** The bottom bar is unchanged. The medium footer's key hints are keyboard text, and a phone needs the tap segments. The owner asked only for the border and the header.
- **D5.** Accepted cost: in compact the page loses 2 columns and 3 rows (top border, rule, bottom border).

### Phases

#### Phase 1: Border and header in compact

- `layout.rs` `lay_out`: `Sides::ALL` in every tier. Remove `Header::compact` and `Find::compact` and their calls.
- The lead: compact pages above Files show `‹` with medium's spacing and colours. A press on the lead or the title goes Back, as `‹ Files` did. `back_text` stays only if the bottom bar or a notice still uses it; otherwise remove it.
- Hit tests: every row mapping in compact (tree, finder results, change list, keys page, diff header, bottom bar) follows the new top offset. Add a TestApp test per page that taps the first and last visible row at 40 and 59 columns and checks which item opens.
- The welcome stays off in compact (record 5's other items stand).
- Gates: `just verify`. Captures at 60 and 80 columns must stay byte-identical to HEAD. Captures at 40 and 49 columns change on purpose; check them by eye against the owner's second screenshot.
- Live run at 40, 49, and 59 columns in a new tab of the agent's own Rex session: Files, open a file, `‹` back, Find, Changes, Keys, and a tap on the first and last rows of each list.

#### Phase 2: Docs and close

- `architecture.md`: a new record that supersedes record 5's border and header items and the finder-bar note, citing the owner's request.
- `DESIGN.md` (the compact sketch and tokens), `docs/keymaps.md` (the back tap is now `‹`), `docs/parity.md` (Layout), CHANGELOG (Unreleased), `rust.md` if it names `compact()`.
- Archive the plan and update `index.md`. Owner check: a phone at portrait width.

### Risks

- Off-by-one taps after the offset change. Phase 1's per-page row tests cover this.
- At 40 columns the title box loses 2 more columns. Phase 1 checks how a long path is cut in the header at 40 columns and reports it.

### Validation

`just verify` (or `KACHE=false just verify`) per phase, plus the live run in phase 1.

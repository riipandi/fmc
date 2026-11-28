---
status: done
created: 2026-11-28T19:00:00+07:00
plan: plan-00021-markdown-image-boxes
---

# Handover: image boxes in the rendered Markdown view; plan-00021 closed

## Closed

- plan-00021 on `feat/markdown-image-boxes` (branched from `main` at v0.7.0 per the owner's brief, not from the plan's stacked branch). Commits, in order: `d973b5d` (phase 1, the lone-image paragraphs found and rewritten as fences), `1cc7a58` (phase 2, the box drawn, the headers read on a worker, search and signs kept), `027452a` (phase 3, `Enter` and a click open the picture, `Ctrl+o` back at the same scroll), and the docs commit that adds this handover (phase 4). Phase 5 made no commit: its validation is below.
- The plan is archived as `archive/plan-00021-markdown-image-boxes.md`, and `handover-00016-ui-state-and-grammar.md` moved to `archive/` when this one landed.
- `just verify` is green at phase 3 (930 tests run, 1 skipped; licenses ok). The live runs are in the commit messages and the run reports, at 80 columns for every case, 120 and 50 columns for the boxes, and `Enter` then `Ctrl+o` at 35 percent of the document.

## Decisions taken as planned

- The owner's D1 to D3 and D5 to D7 hold as written: a box, not a picture; local files inside the browsed root only; no remote fetch and no SVG; the box's three rows and its reasons; `Enter` opens the first box whose top is in view, and a click opens the clicked one; search counts the name.
- D4 holds in substance. A lone image paragraph becomes a `fmc-image` fence with the image in its info string, and the blank line after it closes the fence, so no line is added. A fence cannot sit on one line, so the image cannot be the fence body while the line count holds; the plan's wording "the original line inside" is read as inside the info string.
- The header cache is keyed by the canonical path and the modification time (`services/picture.rs`), and the box reads it on the UI thread only from a value already on the view.

## Choices the plan did not settle (for the owner)

- The fence shape above (the owner check 1 asks for a yes).
- `Enter` with no box in view does nothing and shows nothing. The plan does not say what it shows.
- The reasons' copy: `Cannot open: <reason>.` in the footer. The plan asks for "why in the footer" and gives the reasons; the wording is the agent's, and 80 columns cuts the longer form.
- Percent-escapes in a destination are decoded before the file is looked up (`reader/images.rs` `decoded`), so `Marauder%20map.png` finds `Marauder map.png`. The plan says nothing about escapes.
- A jump back to a Markdown file now restores its scroll from the paint (`pending_top`), which also changes the existing jump-back and the find-open paths that share it. The live check was the Markdown round trip at 35 percent; the unit tests cover the box round trip.

## Traps (paid for; the next agent should read these first)

- A `Custom` widget's `mouse` that returns `true` consumes the release, so the pane's `on_mouse` never sees it. `ui/image_box.rs` records the pick and returns `false`; `app/mouse.rs` takes the pick on the viewer's release. Found by a test that failed until the return changed.
- A scroll set when a document is shown is clamped to the rows drawn before, because the rebuilt document is not laid out yet. `reset_scroll` sets `pending_top`, which the paint applies until it holds (the same as the find-open path). Before this fix a return from a box at 35 percent landed at the bottom of the document, found live.
- `m.top` is painted before the last key's scroll is laid out, so a jump recorded in the key handler takes the viewer's position at that moment (`app/picture.rs` `open`).
- The keymap rows: a `?` card row and the docs row must match exactly (`keys/tests.rs` ties them). Both read "open the image box in view".
- Rex's `block call -b <block> resize '{"columns":120,"rows":40}'` resizes a tab's pane; `rex block send-key` and `rex block capture` drive it. Rex has no click command, so the click path is tested through `TestApp` only.

## Owner checks

1. Read the fence shape in `architecture.md` ("an image that is a whole paragraph is drawn as a box") and say whether the image in the info string is acceptable. If the owner wants the image in a body, the rule also needs a blank line before the image, and an image at the top of a file keeps its text.
2. Open a Markdown file with a box in a real terminal (kitty or iTerm2) and press `Enter` on it. The Rex capture shows the picture area blank, so the picture itself has not been seen on screen; check that it draws, and that `Ctrl+o` returns to the same scroll.
3. Click a box in a real terminal. The release path was checked through `TestApp`, not a real pointer.
4. Attribution: the commits end with `Co-Authored-By: Claude Opus 5.5`, as the plan and the brief say. This session's harness reminder names Claude Haiku 5.5. Say which one the history should carry, and the commits can be reworded before the push if it is the other.
5. The push: `git push -u origin feat/markdown-image-boxes` is run at the end of the run, as the brief authorizes. No PR.

## Not in scope, left open

- Drawing the picture inside the preview, as a later plan behind the box (the plan's out-of-scope list). No roadmap entry: the owner rejected the inline picture in D1, so it is not a parked idea with an unlock.
- A header cache with no eviction: it grows with the distinct image files a session shows. Small in practice; a shortcut if a session browses thousands of images.
- The source view of a Markdown file shows the image as written, and the rewrite is skipped there, as the plan asks.

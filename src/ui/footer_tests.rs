use std::path::Path;

use fmc::theme::DARK;

use super::*;

fn chars(text: &str) -> usize {
    text.chars().count()
}

fn plain(segments: &[Seg]) -> String {
    segments.iter().map(|s| s.text.as_str()).collect()
}

#[test]
fn startup_band_and_hints_at_80_columns() {
    let state = FooterState { location: "/private/tmp/fmc-ref/demo-repo".into(), ..FooterState::default() };
    let (left, right, hints) = state.render(80, &DARK, &chars);
    assert_eq!(plain(&left), " FILES  /private/tmp/fmc-ref/demo-repo");
    assert!(right.is_empty());
    // Scouter's capture at 80 columns (screen 01).
    assert_eq!(plain(&hints), " j k move   Enter open   / find   - up   h back   Tab reader   ^q quit");
    assert_eq!(left[2], Seg::new("/private/tmp/fmc-ref/", DARK.dim));
    assert_eq!(left[3], Seg::new("demo-repo", DARK.text));
}

#[test]
fn hints_at_160_columns_match_scouter() {
    let state = FooterState::default();
    let (_, _, hints) = state.render(160, &DARK, &chars);
    assert_eq!(
        plain(&hints),
        " j k move   Enter open   / find   - up   h back   Tab reader   d changes   w wrap   ^←→ resize   ^o alt+i jump   ? help   ^q quit"
    );
}

#[test]
fn facts_drop_in_scouter_order() {
    let state = FooterState {
        location: "/hogwarts/library/restricted-section/spells.md".into(),
        document: Some(DocumentInfo::Text {
            lines: 120,
            language: "Markdown".into(),
            truncated: false,
            rendered: Some(false),
        }),
        positions: [Some("Top".into()), None],
        ..FooterState::default()
    };
    let (_, right, _) = state.render(160, &DARK, &chars);
    assert_eq!(plain(&right), "120 lines · Markdown · UTF-8 · Top");
    let (_, right, _) = state.render(50, &DARK, &chars);
    assert_eq!(plain(&right), "120 lines · Top");
}

#[test]
fn messages_cut_from_the_right_locations_from_the_left() {
    let state =
        FooterState { location: "/louvre/denon-wing/grande-galerie/mona-lisa.txt".into(), ..FooterState::default() };
    let (left, _, _) = state.render(30, &DARK, &chars);
    assert_eq!(plain(&left), " FILES  …lerie/mona-lisa.txt");
    let state = FooterState {
        message: Some(Message { text: "Still loading. Try again when it finishes.".into(), error: false }),
        ..state
    };
    let (left, _, _) = state.render(30, &DARK, &chars);
    assert_eq!(plain(&left), " FILES  Still loading. Try …");
}

#[test]
fn root_slash_is_dim() {
    let state = FooterState { location: "/".into(), ..FooterState::default() };
    let (left, _, _) = state.render(80, &DARK, &chars);
    assert_eq!(left[2], Seg::new("/", DARK.dim));
}

#[test]
fn tildify_only_whole_folders() {
    let home = Path::new("/Users/langdon");
    assert_eq!(tildify("/Users/langdon", Some(home)), "~");
    assert_eq!(tildify("/Users/langdon/notes", Some(home)), "~/notes");
    assert_eq!(tildify("/Users/langdonx", Some(home)), "/Users/langdonx");
}

#[test]
fn image_and_binary_facts() {
    let image = DocumentInfo::Image { width: 640, height: 480, format: "png".into() };
    let state = FooterState { document: Some(image), ..FooterState::default() };
    assert_eq!(plain(&state.render(120, &DARK, &chars).1), "640×480 · png");
    let state =
        FooterState { document: Some(DocumentInfo::Binary { size: "4.0 KB".into() }), ..FooterState::default() };
    assert_eq!(plain(&state.render(120, &DARK, &chars).1), "4.0 KB · binary");
}

#[test]
fn prompts_name_the_pill_and_the_hints() {
    for (prompt, pill, first) in [
        (Prompt::Files, " FIND ", " type filter"),
        (Prompt::Text, " TEXT ", " type search"),
        (Prompt::Reader, " SEARCH ", " type search"),
    ] {
        let state = FooterState { prompt: Some(prompt), ..FooterState::default() };
        let (left, _, hints) = state.render(120, &DARK, &chars);
        assert_eq!(left[0].text, pill);
        assert!(plain(&hints).starts_with(first));
    }
}

#[test]
fn header_pill_and_notice() {
    use crate::ui::header::{Badge, BadgeKind, HeaderState};
    let mut header = HeaderState {
        title: "vault.ts".into(),
        badges: vec![Badge::quiet("read-only"), Badge { text: "wrap".into(), kind: BadgeKind::Pill }],
        ..Default::default()
    };
    assert_eq!(plain(&header.badge_segments(&DARK)), "read-only  wrap ");
    header.notice = Some("Only UTF-8 text files are supported.".into());
    assert_eq!(header.badge_segments(&DARK), [Seg::new("Only UTF-8 text files are supported.", DARK.error).bold()]);
}

#[test]
fn q_is_offered_only_when_a_document_is_open() {
    use crate::ui::hints::{Mode, Pane, Shown, hints_for};
    let offers_q = |pane, open| {
        hints_for(Mode::Browse, pane, false, true, Shown { open, ..Shown::default() })
            .iter()
            .any(|(key, _)| *key == "q")
    };
    for pane in [Pane::Files, Pane::Reader] {
        assert!(!offers_q(pane, false), "{pane:?}: nothing to close on the welcome screen");
        assert!(offers_q(pane, true), "{pane:?}: q closes the open document");
    }
}

#[test]
fn the_diff_offers_the_other_scope_on_a() {
    use crate::ui::hints::{Mode, Pane, Shown, hints_for};
    let one_file = hints_for(Mode::Diff, Pane::Reader, false, false, Shown::default());
    assert!(one_file.contains(&("a", "all files")), "{one_file:?}");
    let all_files = hints_for(Mode::Diff, Pane::Reader, false, true, Shown::default());
    assert!(all_files.contains(&("a", "one file")), "{all_files:?}");
}

#[test]
fn a_hidden_sidebar_names_its_key_and_drops_tab_to_the_files() {
    use crate::ui::hints::{Mode, Pane, Shown, hints_for};
    let hidden = Shown { sidebar_hidden: true, open: true, ..Shown::default() };
    for mode in [Mode::Browse, Mode::Diff] {
        let hints = hints_for(mode, Pane::Reader, false, true, hidden);
        assert!(hints.contains(&("^/", "sidebar")), "{mode:?}: {hints:?}");
        assert!(!hints.contains(&("Tab", "files")), "{mode:?}: Tab cannot reach a hidden sidebar: {hints:?}");
        let shown = hints_for(mode, Pane::Reader, false, true, Shown { sidebar_hidden: false, ..hidden });
        assert!(!shown.contains(&("^/", "sidebar")), "{mode:?}: the footer at 80 columns keeps its hints: {shown:?}");
        assert!(shown.contains(&("Tab", "files")), "{mode:?}: {shown:?}");
    }
}

#[test]
fn a_hidden_sidebar_hint_survives_a_phone_width() {
    use crate::ui::hints::Pane;
    // At 40 columns only the first hints fit; the one that shows the sidebar must be among them.
    let state = FooterState {
        pane: Pane::Reader,
        sidebar_hidden: true,
        document: Some(DocumentInfo::Text { lines: 18, language: "rust".into(), truncated: false, rendered: None }),
        ..FooterState::default()
    };
    let (_, _, hints) = state.render(40, &DARK, &chars);
    assert!(plain(&hints).contains("^/ sidebar"), "{:?}", plain(&hints));
    assert!(plain(&hints).contains("^q quit"), "{:?}", plain(&hints));
}

#[test]
fn below_80_columns_the_hint_that_leaves_the_prompt_comes_first() {
    let state = FooterState { prompt: Some(Prompt::Reader), ..FooterState::default() };
    let hints = |columns| plain(&state.render(columns, &DARK, &chars).2);
    assert!(hints(40).starts_with(" Esc close"), "{}", hints(40));
    assert!(hints(80).starts_with(" type search"), "at 80 columns the order is Scouter's: {}", hints(80));
}

#[test]
fn a_selection_leads_the_hints_with_copy_below_80_columns() {
    let state = FooterState { pane: Pane::Reader, selection: true, ..FooterState::default() };
    assert_eq!(state.hint_row(40, &chars)[0], ("y", "Copy"));
    assert!(!state.hint_row(80, &chars).contains(&("y", "Copy")), "the copy hint is a phone rule");
}

#[test]
fn a_tap_finds_the_hint_under_its_cells() {
    let cells = FooterState::default().hint_cells(80, &chars);
    // " j k move   Enter open   / find": the `/` hint starts after " j k move   Enter open   ", at column 25.
    let find = cells.iter().find(|(_, key)| *key == "/").expect("find is shown");
    assert_eq!(find.0.start, 25);
    assert_eq!(cells.last().map(|(_, key)| *key), Some("^q"), "quit is the last cell");
}

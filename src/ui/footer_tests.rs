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

use super::*;

fn luminance(hex: &str) -> f64 {
    let linear = |c: f64| if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) };
    let [r, g, b] = channels(hex);
    0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
}

fn contrast(foreground: &str, background: &str) -> f64 {
    let (a, b) = (luminance(foreground), luminance(background));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

fn tokens(p: &Palette) -> Vec<(&'static str, String)> {
    let mut list = vec![
        ("text", p.text),
        ("muted", p.muted),
        ("dim", p.dim),
        ("ignored", p.ignored),
        ("border", p.border),
        ("row-active-bg", p.row_active_bg),
        ("folder", p.folder),
        ("selection-bg", p.selection_bg),
        ("accent", p.accent),
        ("accent-soft", p.accent_soft),
        ("accent-ink", p.accent_ink),
        ("error", p.error),
        ("hover", p.hover),
        ("added", p.added),
        ("removed", p.removed),
        ("modified", p.modified),
        ("added-bg", p.added_bg),
        ("removed-bg", p.removed_bg),
        ("added-gutter", p.added_gutter),
        ("removed-gutter", p.removed_gutter),
        ("match-fg", p.match_fg),
        ("match-bg", p.match_bg),
        ("target-bg", p.target_bg),
        ("target-gutter", p.target_gutter),
        ("fallback-background", p.fallback_background),
        ("syntax-keyword", p.syntax.keyword),
        ("syntax-string", p.syntax.string),
        ("syntax-comment", p.syntax.comment),
        ("syntax-constant", p.syntax.constant),
        ("syntax-function", p.syntax.function),
        ("syntax-punctuation", p.syntax.punctuation),
    ]
    .into_iter()
    .map(|(k, v)| (k, format!("\"{v}\"")))
    .collect::<Vec<_>>();
    list.push(("wordmark", format!("[{}]", p.wordmark.map(|w| format!("\"{w}\"")).join(", "))));
    list
}

#[test]
fn design_md_lists_the_same_colors() {
    let design = include_str!("../DESIGN.md");
    for (mode, palette) in [("dark", &DARK), ("light", &LIGHT)] {
        let section: Vec<&str> = design
            .lines()
            .skip_while(|l| l.trim() != format!("{mode}:"))
            .skip(1)
            .take_while(|l| l.starts_with("    "))
            .collect();
        let expected = tokens(palette);
        assert_eq!(section.len(), expected.len(), "{mode}: token count");
        for (name, value) in expected {
            assert!(section.contains(&format!("    {name}: {value}").as_str()), "{mode}.{name} differs from DESIGN.md");
        }
    }
}

#[test]
fn text_is_readable_in_both_modes() {
    for palette in [&DARK, &LIGHT] {
        let bg = palette.fallback_background;
        let s = &palette.syntax;
        for fg in [palette.text, palette.muted, palette.accent, palette.accent_soft, palette.added, palette.removed]
            .into_iter()
            .chain([palette.modified, palette.error, palette.folder])
            .chain([s.keyword, s.string, s.comment, s.constant, s.function, s.punctuation])
        {
            assert!(contrast(fg, bg) >= 4.5, "{fg} on {bg}");
        }
        assert!(contrast(palette.dim, bg) >= 3.0);
        assert!(contrast(palette.ignored, bg) >= 1.8);
        assert!(contrast(palette.ignored, bg) < contrast(palette.dim, bg));
        for tint in
            [palette.added_bg, palette.removed_bg, palette.target_bg, palette.row_active_bg, palette.selection_bg]
        {
            assert!(contrast(palette.text, tint) >= 7.0, "text on {tint}");
        }
        assert!(contrast(palette.accent, palette.row_active_bg) >= 4.5);
        assert!(contrast(palette.accent_ink, palette.accent) >= 4.5);
        assert!(contrast(palette.match_fg, palette.match_bg) >= 7.0);
    }
}

#[test]
fn background_brightness_picks_the_mode() {
    assert_eq!(mode_of_background("#000000"), Mode::Dark);
    assert_eq!(mode_of_background("#1e1e2e"), Mode::Dark);
    assert_eq!(mode_of_background("#ffffff"), Mode::Light);
    assert_eq!(mode_of_background("#eff1f5"), Mode::Light);
}

#[test]
fn tint_nudges_toward_white_on_dark_and_black_on_light() {
    let dark = tinted_background(Some("#101010"), Mode::Dark);
    assert_eq!(dark, "#161616");
    assert!(brightness(&dark) > brightness("#101010") && brightness(&dark) < brightness("#101010") + 0.05);
    let light = tinted_background(Some("#ffffff"), Mode::Light);
    assert_eq!(light, "#f9f9f9");
    assert_eq!(tinted_background(None, Mode::Dark), DARK.fallback_background);
    assert_eq!(tinted_background(None, Mode::Light), LIGHT.fallback_background);
}

#[test]
fn first_reading_is_dark_when_the_terminal_is_silent() {
    let mut watch = ThemeWatch::default();
    assert_eq!(watch.reading(None, None), Some(SystemTheme { mode: Mode::Dark, background: None }));
    assert_eq!(watch.reading(None, None), None);
    assert!(!watch.keeps_polling(false));
}

#[test]
fn only_a_different_reading_is_reported() {
    let mut watch = ThemeWatch::default();
    let light = SystemTheme { mode: Mode::Light, background: Some("#ffffff".into()) };
    assert_eq!(watch.reading(Some("#ffffff"), None), Some(light));
    assert_eq!(watch.reading(Some("#ffffff"), None), None);
    assert_eq!(watch.reading(Some("#101010"), None).map(|t| t.mode), Some(Mode::Dark));
    assert!(watch.keeps_polling(true));
    watch.announced();
    assert!(!watch.keeps_polling(true));
}

#[test]
fn tint_matches_scouter_output() {
    // Values printed by Scouter's tintedBackground for the same inputs.
    assert_eq!(tinted_background(Some("#1e1e2e"), Mode::Dark), "#242433");
    assert_eq!(tinted_background(Some("#eff1f5"), Mode::Light), "#e9ebef");
}

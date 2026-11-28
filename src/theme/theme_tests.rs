use super::catalog;
use super::*;

fn luminance(hex: Hex) -> f64 {
    let linear = |c: f64| if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) };
    let [r, g, b] = channels(&hex);
    0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
}

fn contrast(foreground: Hex, background: Hex) -> f64 {
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
    let design = include_str!("../../DESIGN.md");
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

/// Every palette the app can show: Scouter's pair, and each bundled theme.
fn shown_palettes() -> Vec<(String, Palette)> {
    let mut palettes = vec![("scouter dark".to_string(), DARK), ("scouter light".to_string(), LIGHT)];
    palettes.extend(catalog::bundled().into_iter().map(|theme| (theme.id, theme.palette)));
    palettes
}

/// The pairs a bundled theme is known to fall short of, found when the catalog was added. The thresholds
/// do not change: each listed pair is short of its threshold on purpose, until the owner changes the colour or the rule.
/// A pair that is short and not listed fails the test, and so does a listed pair that is no longer short.
const SHORT: &[(&str, &str)] = &[
    ("catppuccin", "text/selection-bg"),
    ("catppuccin-latte", "text/added-bg"),
    ("catppuccin-latte", "text/removed-bg"),
    ("catppuccin-latte", "text/target-bg"),
    ("catppuccin-latte", "text/row-active-bg"),
    ("catppuccin-latte", "text/selection-bg"),
    ("catppuccin-latte", "match-fg/match-bg"),
    ("everforest", "text/added-bg"),
    ("everforest", "text/removed-bg"),
    ("everforest", "text/target-bg"),
    ("everforest", "text/row-active-bg"),
    ("everforest", "text/selection-bg"),
    ("everforest", "match-fg/match-bg"),
    ("flexoki-light", "match-fg/match-bg"),
    ("gruvbox", "text/selection-bg"),
    ("gruvbox-light", "text/selection-bg"),
    ("gruvbox-light", "match-fg/match-bg"),
    ("kanagawa", "text/selection-bg"),
    ("light", "match-fg/match-bg"),
    ("one-dark", "text/added-bg"),
    ("one-dark", "text/removed-bg"),
    ("one-dark", "text/target-bg"),
    ("one-dark", "text/row-active-bg"),
    ("one-dark", "text/selection-bg"),
    ("one-light", "match-fg/match-bg"),
    ("rose-pine-dawn", "text/added-bg"),
    ("rose-pine-dawn", "text/removed-bg"),
    ("rose-pine-dawn", "text/target-bg"),
    ("rose-pine-dawn", "text/row-active-bg"),
    ("rose-pine-dawn", "text/selection-bg"),
    ("rose-pine-dawn", "match-fg/match-bg"),
    ("tokyo-night-day", "text/added-bg"),
    ("tokyo-night-day", "text/removed-bg"),
    ("tokyo-night-day", "text/target-bg"),
    ("tokyo-night-day", "text/row-active-bg"),
    ("tokyo-night-day", "text/selection-bg"),
    ("tokyo-night-day", "match-fg/match-bg"),
];

/// Every threshold the palette is held to, as the labels of the pairs that fall short.
fn shortfalls(palette: &Palette) -> Vec<&'static str> {
    let bg = palette.fallback_background;
    let s = &palette.syntax;
    let mut short = Vec::new();
    for fg in [palette.text, palette.muted, palette.accent, palette.accent_soft, palette.added, palette.removed]
        .into_iter()
        .chain([palette.modified, palette.error, palette.folder])
        .chain([s.keyword, s.string, s.comment, s.constant, s.function, s.punctuation])
    {
        if contrast(fg, bg) < 4.5 {
            short.push("text on the page");
        }
    }
    if contrast(palette.dim, bg) < 3.0 {
        short.push("dim");
    }
    if contrast(palette.ignored, bg) < 1.8 || contrast(palette.ignored, bg) >= contrast(palette.dim, bg) {
        short.push("ignored");
    }
    let tints = [
        ("text/added-bg", palette.added_bg),
        ("text/removed-bg", palette.removed_bg),
        ("text/target-bg", palette.target_bg),
        ("text/row-active-bg", palette.row_active_bg),
        ("text/selection-bg", palette.selection_bg),
    ];
    for (label, tint) in tints {
        if contrast(palette.text, tint) < 7.0 {
            short.push(label);
        }
    }
    if contrast(palette.accent, palette.row_active_bg) < 4.5 {
        short.push("accent/row-active-bg");
    }
    if contrast(palette.accent_ink, palette.accent) < 4.5 {
        short.push("accent-ink/accent");
    }
    if contrast(palette.match_fg, palette.match_bg) < 7.0 {
        short.push("match-fg/match-bg");
    }
    short
}

#[test]
fn text_is_readable_in_the_scouter_pair_and_every_bundled_theme() {
    for (name, palette) in shown_palettes() {
        let expected: Vec<&str> = SHORT.iter().filter(|(id, _)| *id == name).map(|(_, pair)| *pair).collect();
        assert_eq!(shortfalls(&palette), expected, "{name}");
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
    let dark = tinted_background("#101010");
    assert_eq!(&*dark, "#161616");
    assert!(brightness(&dark) > brightness("#101010") && brightness(&dark) < brightness("#101010") + 0.05);
    assert_eq!(&*tinted_background("#ffffff"), "#f9f9f9");
}

#[test]
fn the_page_is_tinted_only_in_the_terminals_own_mode() {
    // A forced mode that differs from the terminal paints the palette's fallback, not a tint of the terminal's.
    let dark_fallback = DARK.fallback_background;
    assert_eq!(&*page_background(Some("#101010"), Mode::Dark, Mode::Dark, dark_fallback), "#161616");
    assert_eq!(page_background(Some("#ffffff"), Mode::Light, Mode::Dark, dark_fallback), dark_fallback);
    assert_eq!(&*page_background(Some("#ffffff"), Mode::Light, Mode::Light, LIGHT.fallback_background), "#f9f9f9");
    assert_eq!(page_background(None, Mode::Dark, Mode::Dark, dark_fallback), dark_fallback);
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
    assert_eq!(&*tinted_background("#1e1e2e"), "#242433");
    assert_eq!(&*tinted_background("#eff1f5"), "#e9ebef");
}

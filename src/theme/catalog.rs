//! The theme catalog: the 16 bundled themes (bjarneo/editor, MIT, `catalog/LICENSE`), the user's themes, and the choice of
//! a dark and a light palette from the settings' ids. Pure: the files are read
//! by `services/theme.rs`.

use serde_json::Value;

use super::{DARK, Hex, LIGHT, Mode, Palette, Syntax, channels};
use crate::text::display_label;

/// The id of Scouter's own pair: `DARK` in a dark slot, `LIGHT` in a light slot.
pub const DEFAULT_THEME: &str = "default";

/// The roles fmc does not map: they are read and must be valid, and no palette field takes them.
const UNUSED: [&str; 6] = ["bg2", "vis", "cmd", "ty", "mac", "prop"];

/// The bundled themes, by id. The file names are the ids, so `include_str!` pins each one.
const BUNDLED: [(&str, &str); 16] = [
    ("dark", include_str!("catalog/dark.json")),
    ("catppuccin", include_str!("catalog/catppuccin.json")),
    ("tokyo-night", include_str!("catalog/tokyo-night.json")),
    ("gruvbox", include_str!("catalog/gruvbox.json")),
    ("kanagawa", include_str!("catalog/kanagawa.json")),
    ("rose-pine", include_str!("catalog/rose-pine.json")),
    ("one-dark", include_str!("catalog/one-dark.json")),
    ("everforest", include_str!("catalog/everforest.json")),
    ("nord", include_str!("catalog/nord.json")),
    ("light", include_str!("catalog/light.json")),
    ("catppuccin-latte", include_str!("catalog/catppuccin-latte.json")),
    ("tokyo-night-day", include_str!("catalog/tokyo-night-day.json")),
    ("gruvbox-light", include_str!("catalog/gruvbox-light.json")),
    ("rose-pine-dawn", include_str!("catalog/rose-pine-dawn.json")),
    ("one-light", include_str!("catalog/one-light.json")),
    ("flexoki-light", include_str!("catalog/flexoki-light.json")),
];

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub mode: Mode,
    pub palette: Palette,
}

/// The file's text as a theme named `id`. Strict JSON; unknown keys are ignored. A bad mode, a missing or bad role, or
/// the id `default` (Scouter's pair is built in) is an error, which the caller turns into a notice.
pub fn parse(id: &str, text: &str) -> Result<Theme, String> {
    if id == DEFAULT_THEME {
        return Err(format!("{DEFAULT_THEME} is fmc's built-in pair"));
    }
    let object = match serde_json::from_str::<Value>(text) {
        Ok(Value::Object(object)) => object,
        Ok(_) => return Err("it must hold a JSON object".into()),
        Err(error) => return Err(format!("it is not valid JSON (line {})", error.line())),
    };
    let mode = match object.get("mode").and_then(Value::as_str) {
        Some("dark") => Mode::Dark,
        Some("light") => Mode::Light,
        _ => return Err("mode must be \"dark\" or \"light\"".into()),
    };
    let name = match object.get("name") {
        None => id.to_string(),
        Some(Value::String(name)) => name.clone(),
        Some(_) => return Err("name must be a string".into()),
    };
    let role = |key: &str| match object.get(key) {
        None => Err(format!("role \"{key}\" is missing")),
        Some(Value::String(value)) => {
            Hex::parse(value).ok_or_else(|| format!("role \"{key}\" must be #rrggbb, not \"{value}\""))
        }
        Some(_) => Err(format!("role \"{key}\" must be #rrggbb")),
    };
    let palette = palette(role)?;
    Ok(Theme { id: id.to_string(), name, mode, palette })
}

/// Every bundled theme. `catalog_tests.rs` parses each file, so none is dropped here.
pub fn bundled() -> Vec<Theme> {
    BUNDLED.iter().filter_map(|(id, text)| parse(id, text).ok()).collect()
}

/// The palette from the 24 roles. The source value is the role the mapping names; a derived value is fmc's own
/// formula (`mix`). Every role is read, so a missing one fails the file.
fn palette(role: impl Fn(&str) -> Result<Hex, String>) -> Result<Palette, String> {
    let (bg, bg3, bd, fg, fg2, fg3) = (role("bg")?, role("bg3")?, role("bd")?, role("fg")?, role("fg2")?, role("fg3")?);
    let (ac, ins, warn, err, sel, on) =
        (role("ac")?, role("ins")?, role("warn")?, role("err")?, role("sel")?, role("on")?);
    let (kw, func, string, num, com, pun) =
        (role("kw")?, role("fn")?, role("str")?, role("num")?, role("com")?, role("pun")?);
    for key in UNUSED {
        role(key)?;
    }
    let black = Hex::new("#000000");
    Ok(Palette {
        text: fg,
        muted: fg2,
        dim: fg3,
        ignored: mix(fg3, bg, 0.4),
        border: bd,
        row_active_bg: bg3,
        folder: mix(fg, ac, 0.3),
        selection_bg: sel,
        accent: ac,
        accent_soft: mix(ac, fg, 0.35),
        accent_ink: on,
        error: err,
        hover: bg3,
        added: ins,
        removed: err,
        modified: warn,
        added_bg: mix(bg, ins, 0.15),
        removed_bg: mix(bg, err, 0.15),
        added_gutter: mix(bg, ins, 0.25),
        removed_gutter: mix(bg, err, 0.25),
        match_fg: bg,
        match_bg: warn,
        target_bg: mix(bg, ac, 0.2),
        target_gutter: mix(bg, ac, 0.3),
        wordmark: [mix(ac, fg, 0.3), ac, mix(ac, black, 0.25)],
        fallback_background: bg,
        syntax: Syntax { keyword: kw, string, comment: com, constant: num, function: func, punctuation: pun },
    })
}

/// `a` blended toward `b` by `t` (0 is `a`, 1 is `b`), per channel in RGB, rounded.
fn mix(a: Hex, b: Hex, t: f64) -> Hex {
    let (x, y) = (channels(&a), channels(&b));
    let c = |i: usize| ((x[i] + (y[i] - x[i]) * t) * 255.0).round() as u8;
    Hex::rgb(c(0), c(1), c(2))
}

/// The palettes the settings choose, with a notice for each id that is not usable in its slot.
pub struct Chosen {
    pub dark: Palette,
    pub light: Palette,
    pub notices: Vec<String>,
}

/// The dark and light palettes for the ids in settings. The user's themes replace a bundled theme with the same id. An
/// id names one mode and is used only in its own slot; `default` is Scouter's pair in either slot. A bad id keeps that
/// slot's Scouter palette and adds a notice.
pub fn choose(user: Vec<Theme>, dark_id: &str, light_id: &str) -> Chosen {
    let mut themes = merged(user);
    let mut notices = Vec::new();
    let dark = slot(&mut themes, "dark_theme", dark_id, Mode::Dark, &mut notices);
    let light = slot(&mut themes, "light_theme", light_id, Mode::Light, &mut notices);
    Chosen { dark, light, notices }
}

/// The bundled themes with the user's: a user theme replaces the bundled one with its id, and a new id is added.
fn merged(user: Vec<Theme>) -> Vec<Theme> {
    let mut themes = bundled();
    for theme in user {
        match themes.iter().position(|bundled| bundled.id == theme.id) {
            Some(index) => themes[index] = theme,
            None => themes.push(theme),
        }
    }
    themes
}

/// The ids a slot accepts for `mode`: `default` first, then each bundled or user theme of that mode, in the order
/// `merged` keeps them. The Settings page cycles through these.
pub fn slot_ids(user: &[Theme], mode: Mode) -> Vec<String> {
    let others = merged(user.to_vec()).into_iter().filter(|theme| theme.mode == mode).map(|theme| theme.id);
    std::iter::once(DEFAULT_THEME.to_string()).chain(others).collect()
}

fn slot(themes: &mut Vec<Theme>, key: &str, id: &str, mode: Mode, notices: &mut Vec<String>) -> Palette {
    let default = match mode {
        Mode::Dark => DARK,
        Mode::Light => LIGHT,
    };
    if id == DEFAULT_THEME {
        return default;
    }
    match themes.iter().position(|theme| theme.id == id) {
        Some(index) if themes[index].mode == mode => themes.swap_remove(index).palette,
        Some(_) => {
            let other = if mode == Mode::Dark { "light" } else { "dark" };
            notices.push(format!(
                "settings.json: {key} \"{}\" is a {other} theme; the default theme is used.",
                display_label(id)
            ));
            default
        }
        None => {
            notices.push(format!(
                "settings.json: {key} \"{}\" is not a theme; the default theme is used.",
                display_label(id)
            ));
            default
        }
    }
}

#[cfg(test)]
#[path = "catalog_tests.rs"]
mod tests;

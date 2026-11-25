//! The dark and light palettes, brightness, the tint, and the system-theme reading (Scouter `src/ui/theme.ts`,
//! `src/ui/system-theme.ts`). Colors are `#rrggbb` strings, as in Scouter; the views turn them into engine colors.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Dark,
    Light,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Syntax {
    pub keyword: &'static str,
    pub string: &'static str,
    pub comment: &'static str,
    pub constant: &'static str,
    pub function: &'static str,
    pub punctuation: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Palette {
    pub text: &'static str,
    pub muted: &'static str,
    pub dim: &'static str,
    pub ignored: &'static str,
    pub border: &'static str,
    pub row_active_bg: &'static str,
    pub folder: &'static str,
    pub selection_bg: &'static str,
    pub accent: &'static str,
    pub accent_soft: &'static str,
    pub accent_ink: &'static str,
    pub error: &'static str,
    pub hover: &'static str,
    pub added: &'static str,
    pub removed: &'static str,
    pub modified: &'static str,
    pub added_bg: &'static str,
    pub removed_bg: &'static str,
    pub added_gutter: &'static str,
    pub removed_gutter: &'static str,
    pub match_fg: &'static str,
    pub match_bg: &'static str,
    pub target_bg: &'static str,
    pub target_gutter: &'static str,
    pub wordmark: [&'static str; 3],
    pub fallback_background: &'static str,
    pub syntax: Syntax,
}

pub const DARK: Palette = Palette {
    text: "#d4d8de",
    muted: "#8b929c",
    dim: "#666d78",
    ignored: "#4a505a",
    border: "#2a2f3a",
    row_active_bg: "#222633",
    folder: "#8fa3bf",
    selection_bg: "#2c365e",
    accent: "#818cf8",
    accent_soft: "#c7d2fe",
    accent_ink: "#0f1117",
    error: "#f2a1a1",
    hover: "#343a42",
    added: "#8fd3a8",
    removed: "#f2a1a1",
    modified: "#e5b880",
    added_bg: "#12281c",
    removed_bg: "#2b1518",
    added_gutter: "#173a27",
    removed_gutter: "#3b1b20",
    match_fg: "#0f1117",
    match_bg: "#e5b880",
    target_bg: "#262b46",
    target_gutter: "#343b66",
    wordmark: ["#a5b4fc", "#818cf8", "#6366f1"],
    fallback_background: "#131519",
    syntax: Syntax {
        keyword: "#c792ea",
        string: "#a7c7a1",
        comment: "#8b929c",
        constant: "#e5b880",
        function: "#82aaff",
        punctuation: "#b2b9c3",
    },
};

pub const LIGHT: Palette = Palette {
    text: "#1f2328",
    muted: "#59636e",
    dim: "#7d8590",
    ignored: "#b1b8c0",
    border: "#d0d7de",
    row_active_bg: "#eaeef2",
    folder: "#4c5c75",
    selection_bg: "#d8e0ff",
    accent: "#4f46e5",
    accent_soft: "#4338ca",
    accent_ink: "#ffffff",
    error: "#cf222e",
    hover: "#d0d7de",
    added: "#1a7f37",
    removed: "#cf222e",
    modified: "#9a6700",
    added_bg: "#dafbe1",
    removed_bg: "#ffebe9",
    added_gutter: "#aceebb",
    removed_gutter: "#ffcecb",
    match_fg: "#1f2328",
    match_bg: "#ffdf5d",
    target_bg: "#e0e7ff",
    target_gutter: "#c7d2fe",
    wordmark: ["#6366f1", "#4f46e5", "#3730a3"],
    fallback_background: "#f6f8fa",
    syntax: Syntax {
        keyword: "#8250df",
        string: "#116329",
        comment: "#646e78",
        constant: "#953800",
        function: "#0550ae",
        punctuation: "#57606a",
    },
};

pub fn palette(mode: Mode) -> &'static Palette {
    match mode {
        Mode::Dark => &DARK,
        Mode::Light => &LIGHT,
    }
}

/// `#rrggbb` as channels in 0..=1, like OpenTUI's `RGBA.fromHex`. A malformed string reads as black.
pub fn channels(hex: &str) -> [f64; 3] {
    let byte = |i: usize| hex.get(1 + 2 * i..3 + 2 * i).and_then(|s| u8::from_str_radix(s, 16).ok()).unwrap_or(0);
    [0, 1, 2].map(|i| f64::from(byte(i)) / 255.0)
}

pub fn brightness(hex: &str) -> f64 {
    let [r, g, b] = channels(hex);
    r * 0.299 + g * 0.587 + b * 0.114
}

pub fn mode_of_background(hex: &str) -> Mode {
    if brightness(hex) < 0.5 { Mode::Dark } else { Mode::Light }
}

const TINT: f64 = 0.025;

/// The terminal background nudged 2.5% toward white (dark) or black (light); the palette's fallback when unknown.
pub fn tinted_background(background: Option<&str>, mode: Mode) -> String {
    let Some(background) = background else { return palette(mode).fallback_background.to_string() };
    let target = if mode_of_background(background) == Mode::Dark { 1.0 } else { 0.0 };
    let mix = channels(background).map(|c| ((c + (target - c) * TINT) * 255.0).round() as u8);
    format!("#{:02x}{:02x}{:02x}", mix[0], mix[1], mix[2])
}

/// What the terminal says about its theme.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemTheme {
    pub mode: Mode,
    pub background: Option<String>,
}

/// Decides when a terminal reading changes the theme (Scouter `SystemThemeWatcher`). The app asks the terminal for
/// its background at start and every 3 s, and again when the terminal announces a new mode.
#[derive(Debug, Default)]
pub struct ThemeWatch {
    last: Option<(Mode, Option<String>)>,
    announces: bool,
}

pub const POLL_SECONDS: u64 = 3;

impl ThemeWatch {
    /// A reading: the background the terminal reported, if it answered, and its mode hint. Returns the theme to
    /// apply when it differs from the last one. The first reading always yields a theme, dark when nothing is known.
    pub fn reading(&mut self, background: Option<&str>, hint: Option<Mode>) -> Option<SystemTheme> {
        let mode = match background {
            Some(bg) => mode_of_background(bg),
            None => hint.or(if self.last.is_none() { Some(Mode::Dark) } else { None })?,
        };
        let key = (mode, background.map(str::to_string));
        if self.last.as_ref() == Some(&key) {
            return None;
        }
        self.last = Some(key);
        Some(SystemTheme { mode, background: background.map(str::to_string) })
    }

    /// The terminal announced a mode: from now on it is trusted to announce, and polling stops.
    pub fn announced(&mut self) {
        self.announces = true;
    }

    /// Whether to keep polling after a reading: only while the terminal answers and has never announced.
    pub fn keeps_polling(&self, answered: bool) -> bool {
        answered && !self.announces
    }
}

#[cfg(test)]
#[path = "theme_tests.rs"]
mod tests;

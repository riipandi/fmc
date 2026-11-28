//! The dark and light palettes, brightness, the tint, and the system-theme reading (Scouter `src/ui/theme.ts`,
//! `src/ui/system-theme.ts`). Colors are `#rrggbb` values (`Hex`), as in Scouter; the views turn them into engine colors.
//! The catalog of themes and the choice from settings are in `catalog.rs`.

use std::fmt;
use std::ops::Deref;

pub mod catalog;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Dark,
    Light,
}

/// `theme_mode` in settings.json: follow the terminal, or force one mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModeChoice {
    Auto,
    Dark,
    Light,
}

impl ModeChoice {
    /// Every choice, in the order the Settings page and the schema list them.
    pub const ALL: [Self; 3] = [Self::Auto, Self::Dark, Self::Light];

    /// The word `theme_mode` holds for this choice.
    pub fn name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }

    pub fn named(text: &str) -> Option<Self> {
        match text {
            "auto" => Some(Self::Auto),
            "dark" => Some(Self::Dark),
            "light" => Some(Self::Light),
            _ => None,
        }
    }
}

/// A `#rrggbb` colour in lowercase: seven ASCII bytes, so a palette read is a copy and needs no borrow or allocation
/// (plan-00015 A15). Built-in values come from `Hex::new`, user and catalog values from `Hex::parse`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hex([u8; 7]);

impl Hex {
    /// A built-in value. A bad literal panics in const evaluation, so it fails the build, not a run.
    pub const fn new(text: &str) -> Self {
        match Self::from_bytes(text.as_bytes()) {
            Some(hex) => hex,
            None => panic!("a colour is #rrggbb"),
        }
    }

    /// A value from a file: `#` and six hex digits, in either case.
    pub fn parse(text: &str) -> Option<Self> {
        Self::from_bytes(text.as_bytes())
    }

    /// A derived value, from its channels.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        const DIGITS: &[u8; 16] = b"0123456789abcdef";
        Self([
            b'#',
            DIGITS[(r >> 4) as usize],
            DIGITS[(r & 15) as usize],
            DIGITS[(g >> 4) as usize],
            DIGITS[(g & 15) as usize],
            DIGITS[(b >> 4) as usize],
            DIGITS[(b & 15) as usize],
        ])
    }

    const fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 7 || bytes[0] != b'#' {
            return None;
        }
        let mut out = [b'#'; 7];
        let mut i = 1;
        while i < 7 {
            out[i] = match bytes[i] {
                c @ (b'0'..=b'9' | b'a'..=b'f') => c,
                c @ b'A'..=b'F' => c + (b'a' - b'A'),
                _ => return None,
            };
            i += 1;
        }
        Some(Self(out))
    }
}

impl Deref for Hex {
    type Target = str;

    fn deref(&self) -> &str {
        // INVARIANT: the bytes are ASCII (`from_bytes`), so they are UTF-8; the fallback is never taken.
        std::str::from_utf8(&self.0).unwrap_or("#000000")
    }
}

impl fmt::Display for Hex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}

impl fmt::Debug for Hex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &**self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Syntax {
    pub keyword: Hex,
    pub string: Hex,
    pub comment: Hex,
    pub constant: Hex,
    pub function: Hex,
    pub punctuation: Hex,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette {
    pub text: Hex,
    pub muted: Hex,
    pub dim: Hex,
    pub ignored: Hex,
    pub border: Hex,
    pub row_active_bg: Hex,
    pub folder: Hex,
    pub selection_bg: Hex,
    pub accent: Hex,
    pub accent_soft: Hex,
    pub accent_ink: Hex,
    pub error: Hex,
    pub hover: Hex,
    pub added: Hex,
    pub removed: Hex,
    pub modified: Hex,
    pub added_bg: Hex,
    pub removed_bg: Hex,
    pub added_gutter: Hex,
    pub removed_gutter: Hex,
    pub match_fg: Hex,
    pub match_bg: Hex,
    pub target_bg: Hex,
    pub target_gutter: Hex,
    pub wordmark: [Hex; 3],
    pub fallback_background: Hex,
    pub syntax: Syntax,
}

pub const DARK: Palette = Palette {
    text: Hex::new("#d4d8de"),
    muted: Hex::new("#8b929c"),
    dim: Hex::new("#666d78"),
    ignored: Hex::new("#4a505a"),
    border: Hex::new("#2a2f3a"),
    row_active_bg: Hex::new("#222633"),
    folder: Hex::new("#8fa3bf"),
    selection_bg: Hex::new("#2c365e"),
    accent: Hex::new("#818cf8"),
    accent_soft: Hex::new("#c7d2fe"),
    accent_ink: Hex::new("#0f1117"),
    error: Hex::new("#f2a1a1"),
    hover: Hex::new("#343a42"),
    added: Hex::new("#8fd3a8"),
    removed: Hex::new("#f2a1a1"),
    modified: Hex::new("#e5b880"),
    added_bg: Hex::new("#12281c"),
    removed_bg: Hex::new("#2b1518"),
    added_gutter: Hex::new("#173a27"),
    removed_gutter: Hex::new("#3b1b20"),
    match_fg: Hex::new("#0f1117"),
    match_bg: Hex::new("#e5b880"),
    target_bg: Hex::new("#262b46"),
    target_gutter: Hex::new("#343b66"),
    wordmark: [Hex::new("#a5b4fc"), Hex::new("#818cf8"), Hex::new("#6366f1")],
    fallback_background: Hex::new("#131519"),
    syntax: Syntax {
        keyword: Hex::new("#c792ea"),
        string: Hex::new("#a7c7a1"),
        comment: Hex::new("#8b929c"),
        constant: Hex::new("#e5b880"),
        function: Hex::new("#82aaff"),
        punctuation: Hex::new("#b2b9c3"),
    },
};

pub const LIGHT: Palette = Palette {
    text: Hex::new("#1f2328"),
    muted: Hex::new("#59636e"),
    dim: Hex::new("#7d8590"),
    ignored: Hex::new("#b1b8c0"),
    border: Hex::new("#d0d7de"),
    row_active_bg: Hex::new("#eaeef2"),
    folder: Hex::new("#4c5c75"),
    selection_bg: Hex::new("#d8e0ff"),
    accent: Hex::new("#4f46e5"),
    accent_soft: Hex::new("#4338ca"),
    accent_ink: Hex::new("#ffffff"),
    error: Hex::new("#cf222e"),
    hover: Hex::new("#d0d7de"),
    added: Hex::new("#1a7f37"),
    removed: Hex::new("#cf222e"),
    modified: Hex::new("#9a6700"),
    added_bg: Hex::new("#dafbe1"),
    removed_bg: Hex::new("#ffebe9"),
    added_gutter: Hex::new("#aceebb"),
    removed_gutter: Hex::new("#ffcecb"),
    match_fg: Hex::new("#1f2328"),
    match_bg: Hex::new("#ffdf5d"),
    target_bg: Hex::new("#e0e7ff"),
    target_gutter: Hex::new("#c7d2fe"),
    wordmark: [Hex::new("#6366f1"), Hex::new("#4f46e5"), Hex::new("#3730a3")],
    fallback_background: Hex::new("#f6f8fa"),
    syntax: Syntax {
        keyword: Hex::new("#8250df"),
        string: Hex::new("#116329"),
        comment: Hex::new("#646e78"),
        constant: Hex::new("#953800"),
        function: Hex::new("#0550ae"),
        punctuation: Hex::new("#57606a"),
    },
};

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

/// The terminal background nudged 2.5% toward white (dark) or black (light).
pub fn tinted_background(background: &str) -> Hex {
    let target = if mode_of_background(background) == Mode::Dark { 1.0 } else { 0.0 };
    let mix = channels(background).map(|c| ((c + (target - c) * TINT) * 255.0).round() as u8);
    Hex::rgb(mix[0], mix[1], mix[2])
}

/// The page's background (A14): the terminal's background tinted when the palette is the terminal's mode. Otherwise,
/// or when the terminal did not answer, the palette's own fallback, since a tint of the other mode's background is
/// not that mode's page.
pub fn page_background(background: Option<&str>, terminal: Mode, palette: Mode, fallback: Hex) -> Hex {
    match background {
        Some(background) if terminal == palette => tinted_background(background),
        _ => fallback,
    }
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

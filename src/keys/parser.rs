//! Turns one key press into an action by walking the merged key table (`bindings.rs`, `keymap.rs`), in Scouter's
//! dispatch order: quit, count digits, the rows of the state, then the focused pane. The search prompt reads keys before
//! this parser does. Count digits and `z` are not rows, so they stay here.

use super::bindings::{Binding, Context, Press};
use super::keymap::Keymap;
use super::{Action, Key};

/// The largest count a run of digits can build (Scouter `MAX_COUNT`).
const MAX_COUNT: u32 = 99_999_999;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    List,
    Reader,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Browse,
    Changes,
}

/// What the app looks like when the key arrives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub screen: Screen,
    pub focus: Focus,
    /// The reader has something open (Scouter `readerReady`); counts need it.
    pub reader_ready: bool,
    /// The reader shows rendered Markdown, which refuses counts and line motions.
    pub previewing: bool,
    /// A find or search prompt is open: `Ctrl+o` and `Alt+i` are left to it.
    pub typing: bool,
}

/// The parser's answer for one key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// The key started or extended a sequence (`z`, count digits, a multi-key binding); wait for the next one.
    More,
    /// No binding: the key does nothing.
    Bad,
    Done(Action),
}

/// What the table says about the keys pressed so far.
enum Found {
    /// A binding continues these keys.
    Prefix,
    Action(Action),
    Nothing,
}

/// The keys of the merged table, the count typed so far, a first `z`, and a sequence in progress.
#[derive(Debug)]
pub struct Parser {
    keymap: Keymap,
    count: u32,
    z_pending: bool,
    pending: Vec<Press>,
}

impl Default for Parser {
    fn default() -> Parser {
        Parser::new(Keymap::defaults())
    }
}

impl Parser {
    pub fn new(keymap: Keymap) -> Parser {
        Parser { keymap, count: 0, z_pending: false, pending: Vec::new() }
    }

    pub fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    /// The count typed so far, for the footer.
    pub fn pending_count(&self) -> Option<u32> {
        (self.count > 0).then_some(self.count)
    }

    pub fn feed(&mut self, key: Key<'_>, state: State) -> Step {
        let after_z = std::mem::take(&mut self.z_pending);
        let press = Press::of(key);

        // A sequence in progress takes this key. A key that cannot continue it is read on its own (below).
        if !self.pending.is_empty() {
            let mut keys = std::mem::take(&mut self.pending);
            keys.push(press.clone());
            match self.lookup(&keys, state) {
                Found::Prefix => {
                    self.pending = keys;
                    return Step::More;
                }
                Found::Action(action) => return self.done(action),
                Found::Nothing => {}
            }
        }
        if self.absorb_count(key, state) {
            return Step::More;
        }
        match self.lookup(std::slice::from_ref(&press), state) {
            Found::Prefix => {
                self.pending = vec![press];
                return Step::More;
            }
            Found::Action(action) => return self.done(action),
            Found::Nothing => {}
        }

        // `z` and `zz` (Scouter's centre keys) are not rows: the reader keeps them, after the table has had its turn.
        self.count = 0;
        if state.focus == Focus::Reader
            && state.screen == Screen::Browse
            && key.plain()
            && key.name == "z"
            && !key.shift
        {
            if after_z {
                return Step::Done(Action::Centre);
            }
            self.z_pending = true;
            return Step::More;
        }
        Step::Bad
    }

    /// A count takes the next `G` as its line (`120G`); every other action drops it.
    fn done(&mut self, action: Action) -> Step {
        let count = std::mem::take(&mut self.count);
        match action {
            Action::Bottom if count > 0 => Step::Done(Action::GoToLine(count)),
            action => Step::Done(action),
        }
    }

    fn absorb_count(&mut self, key: Key<'_>, state: State) -> bool {
        let digit = match key.text.as_bytes() {
            [d @ b'0'..=b'9'] if key.plain() => u32::from(d - b'0'),
            _ => return false,
        };
        if !state.reader_ready
            || state.focus != Focus::Reader
            || state.screen != Screen::Browse
            || state.previewing
            || (self.count == 0 && digit == 0)
        {
            return false;
        }
        self.count = (self.count * 10 + digit).min(MAX_COUNT);
        true
    }

    /// The rows of the state that bind `keys`. A row that continues them wins, so a longer binding (`g x` in a file)
    /// makes `g` wait for its next key, as the table says.
    fn lookup(&self, keys: &[Press], state: State) -> Found {
        let live: Vec<&Binding> = self
            .keymap
            .bindings()
            .iter()
            .filter(|b| active(b.context, state) && b.typing.is_none_or(|typing| typing == state.typing))
            .collect();
        let binds = |b: &Binding, n: usize| b.keys.iter().zip(keys).take(n).all(|(chord, press)| chord.matches(press));
        if live.iter().any(|b| b.keys.len() > keys.len() && binds(b, keys.len())) {
            return Found::Prefix;
        }
        match live.iter().find(|b| b.keys.len() == keys.len() && binds(b, keys.len())) {
            Some(binding) => Found::Action(binding.action),
            None => Found::Nothing,
        }
    }
}

/// Which rows apply in a state: `everywhere` in all, the others by screen and focus (`bindings.rs` `Context`).
fn active(context: Context, state: State) -> bool {
    let browse = state.screen == Screen::Browse;
    let reader = state.focus == Focus::Reader;
    match context {
        Context::Everywhere => true,
        Context::Browse => browse && !reader,
        Context::Reader => browse && reader,
        Context::Changes => !browse,
        Context::ChangesList => !browse && !reader,
        Context::ChangesReader => !browse && reader,
    }
}

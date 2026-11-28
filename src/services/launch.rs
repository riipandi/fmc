//! Hands a file or folder to macOS (plan-00017, D1 to D3). Every launch is `/usr/bin/open` with argv only, and the apps
//! that open a file come from one constant JXA script run by `/usr/bin/osascript`. No shell runs here: a path with
//! quotes or `$(…)` is only a path.
// The picker (`list_apps`, `parse_apps`, `App`) and the Reveal and App forms have no caller yet; plan-00017 phases 3 and 4 wire them and remove this line.
#![allow(dead_code)]

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::capture;

const OPEN: &str = "/usr/bin/open";
const OSASCRIPT: &str = "/usr/bin/osascript";

/// Lists the apps Launch Services offers for the file in `argv[0]`: one tab-separated line per app, with a `*` mark on
/// the default, the display name, the bundle id (`-` when there is none), and the app path. The path is read from argv
/// and never built into the script text, so no path reaches the script as code.
const APPS_SCRIPT: &str = "ObjC.import('AppKit');
function run(argv) {
  const ws = $.NSWorkspace.sharedWorkspace;
  const fm = $.NSFileManager.defaultManager;
  const url = $.NSURL.fileURLWithPath(argv[0]);
  const apps = ws.URLsForApplicationsToOpenURL(url);
  const def = ws.URLForApplicationToOpenURL(url);
  const defPath = (def && !def.isNil()) ? ObjC.unwrap(def.path) : '';
  const out = [];
  for (let i = 0; i < apps.count; i++) {
    const app = apps.objectAtIndex(i);
    const bid = $.NSBundle.bundleWithURL(app).bundleIdentifier;
    const mark = ObjC.unwrap(app.path) === defPath ? '*' : ' ';
    const name = ObjC.unwrap(fm.displayNameAtPath(app.path));
    const id = bid.isNil() ? '-' : ObjC.unwrap(bid);
    out.push([mark, name, id, ObjC.unwrap(app.path)].join('\\t'));
  }
  return out.length ? out.join('\\n') : '(none)';
}";

/// `open` prints nothing on success, so the cap only bounds a failure's text.
const OPEN_LIMIT: usize = 64 * 1024;
/// A machine with hundreds of apps prints tens of kilobytes.
const APPS_LIMIT: usize = 1024 * 1024;
/// A launch or a listing that runs longer is stopped, so a worker is never held forever.
const TIMEOUT: Duration = Duration::from_secs(15);

/// What `open` does with its target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum How<'a> {
    /// The app Launch Services picks for the target, as a Finder double-click does.
    Default,
    /// An app by its name or its path; `open -a` takes either.
    App(&'a str),
    /// Finder, with the target selected (`open -R`).
    Reveal,
}

/// The argv of one `open` call. The target comes after `--`, so a path that starts with `-` is never read as a flag.
pub fn open_args(target: &Path, how: How) -> Vec<OsString> {
    let mut args: Vec<OsString> = match how {
        How::Default => Vec::new(),
        How::App(app) => vec!["-a".into(), app.into()],
        How::Reveal => vec!["-R".into()],
    };
    args.push("--".into());
    args.push(target.into());
    args
}

/// Runs `open` with `args` and waits for it. A failure is `open`'s first stderr line, which names the cause.
pub fn run_open(args: &[OsString]) -> Result<(), String> {
    let mut command = Command::new(OPEN);
    command.args(args);
    capture(command, OPEN_LIMIT, false, TIMEOUT).map(|_| ())
}

/// An app that opens a file: its name as Finder shows it, its path, and whether Launch Services picks it for the file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct App {
    pub name: String,
    pub path: PathBuf,
    pub default: bool,
}

/// The apps that open `file`, from the application folders only (D3). A failure is the script's first stderr line.
pub fn list_apps(file: &Path, home: &Path) -> Result<Vec<App>, String> {
    let listed = capture(listing_command(file), APPS_LIMIT, false, TIMEOUT)?;
    Ok(parse_apps(&listed.text, home))
}

/// The `osascript` call that runs the listing script on `file`. The file is the only argument after `--`.
fn listing_command(file: &Path) -> Command {
    let mut command = Command::new(OSASCRIPT);
    command.args(["-l", "JavaScript", "-e", APPS_SCRIPT, "--"]).arg(file);
    command
}

/// Keeps the apps in the application folders (D3). A helper or a cache elsewhere, such as a Chrome for Testing under
/// `~/Library/Caches`, is dropped. The check is by path component, so `/ApplicationsX` does not match.
fn kept(path: &Path, home: &Path) -> bool {
    const FOLDERS: [&str; 3] = ["/Applications", "/System/Applications", "/System/Library/CoreServices/Applications"];
    FOLDERS.iter().any(|folder| path.starts_with(folder)) || path.starts_with(home.join("Applications"))
}

/// The script's lines as apps: the default first, the rest by name. A line that is not four tab-separated fields, such
/// as `(none)`, or that has no path, is skipped.
pub fn parse_apps(text: &str, home: &Path) -> Vec<App> {
    let mut apps: Vec<App> = text
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(4, '\t');
            let (mark, name, _bundle, path) = (fields.next()?, fields.next()?, fields.next()?, fields.next()?);
            let path = PathBuf::from(path);
            kept(&path, home).then(|| App { name: name.to_string(), path, default: mark == "*" })
        })
        .collect();
    apps.sort_by_key(|app| (!app.default, app.name.to_lowercase(), app.path.clone()));
    apps
}

#[cfg(test)]
#[path = "launch_tests.rs"]
mod tests;

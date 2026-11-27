//! The command line (Scouter `src/index.ts`): help, version, and one optional file or folder.

use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

use clap::Parser;

/// The positional arguments. `-h`/`--help` and `-v`/`--version` are matched as exact tokens before clap runs:
/// Scouter honors them anywhere in argv, and treats every other argument, even `-x` or `-hv`, as a path.
#[derive(Parser, Debug)]
#[command(name = "fmc", disable_help_flag = true, disable_version_flag = true)]
struct Args {
    #[arg(allow_hyphen_values = true)]
    paths: Vec<OsString>,
}

/// What the command line asks for.
#[derive(Debug, PartialEq)]
pub enum Command {
    Help,
    Version,
    /// Open this path, still unchecked: `open_target` checks it after the terminal check.
    Open(Option<OsString>),
}

pub fn help_text(keymap: &fmc::keys::Keymap) -> String {
    format!(
        "fmc — a minimal terminal file manager\n\n\
         Usage: fmc [file-or-directory]\n       fmc --version\n\n\
         Defaults to the current directory. UTF-8 text files show their first megabyte.\n\n{}\n",
        fmc::keys::key_list_text_with(keymap)
    )
}

pub fn version_text() -> String {
    format!("fmc {}", env!("CARGO_PKG_VERSION"))
}

/// Parses argv (without the program name), in Scouter's order: help, version, then one path.
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Command, String> {
    let args: Vec<OsString> = args.into_iter().collect();
    let has = |flags: [&str; 2]| args.iter().any(|a| flags.iter().any(|f| a == f));
    if has(["--help", "-h"]) {
        return Ok(Command::Help);
    }
    if has(["--version", "-v"]) {
        return Ok(Command::Version);
    }
    let argv = std::iter::once(OsString::from("fmc")).chain(["--".into()]).chain(args);
    let mut paths = Args::try_parse_from(argv).map_err(|e| e.to_string())?.paths;
    if paths.len() > 1 {
        return Err("Expected one file or directory. Use --help for usage.".into());
    }
    Ok(Command::Open(paths.pop()))
}

/// Resolves the path against `cwd` and checks that it is a file or a directory.
pub fn open_target(path: Option<OsString>, cwd: &Path) -> Result<PathBuf, String> {
    let path = resolve(cwd, path.as_deref().map(Path::new).unwrap_or(Path::new("")));
    let metadata = std::fs::metadata(&path).map_err(|e| stat_error(&e, &path))?;
    if !metadata.is_file() && !metadata.is_dir() {
        return Err("Only files and directories can be opened.".into());
    }
    Ok(path)
}

/// Lexical resolve like Node's `path.resolve`: absolute, with `.` and `..` folded, links untouched.
fn resolve(cwd: &Path, path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in cwd.join(path).components() {
        match part {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            part => out.push(part),
        }
    }
    out
}

/// The message Bun prints for a failed `stat`, e.g. `ENOENT: no such file or directory, stat '/a/b'`.
fn stat_error(error: &std::io::Error, path: &Path) -> String {
    let (code, text) = match error.raw_os_error() {
        Some(2) => ("ENOENT", "no such file or directory"),
        Some(13) => ("EACCES", "permission denied"),
        Some(20) => ("ENOTDIR", "not a directory"),
        Some(62) => ("ELOOP", "too many symbolic links encountered"),
        Some(63) => ("ENAMETOOLONG", "name too long"),
        _ => return error.to_string(),
    };
    format!("{code}: {text}, stat '{}'", path.display())
}

#[cfg(test)]
#[path = "args_tests.rs"]
mod tests;

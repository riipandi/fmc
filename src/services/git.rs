//! Git reads for the tree's marks (Scouter `src/diff/git.ts`). Git runs against a private copy of the index, so the
//! repository's own index is never rewritten: `git diff` refreshes the index it reads (`architecture.md`, Git).
//! Parsing is in `fmc::diff`.

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use fmc::diff::{ChangeKind, Listing, parse_name_status, split_z};

/// A copy of the repository's index for one call, removed when the call returns.
struct IndexCopy {
    dir: PathBuf,
    file: PathBuf,
}

impl IndexCopy {
    /// Copies `source` and keeps its modification time: git's racy-index check compares timestamps, so a copy
    /// stamped with the current time would hide a change made in the same second as the index (Scouter decision 0007).
    /// A repository without an index gets none, which git reads as empty.
    fn new(source: &Path) -> Result<Self, String> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "fmc-index-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).map_err(|e| e.to_string())?;
        let copy = Self { file: dir.join("index"), dir };
        if let Ok(metadata) = fs::metadata(source) {
            fs::copy(source, &copy.file).map_err(|e| e.to_string())?;
            let modified = metadata.modified().map_err(|e| e.to_string())?;
            File::options()
                .write(true)
                .open(&copy.file)
                .and_then(|file| file.set_modified(modified))
                .map_err(|e| e.to_string())?;
        }
        Ok(copy)
    }
}

impl Drop for IndexCopy {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

/// Git's marks for the repository holding `dir`, or `None` when `dir` is not inside one.
pub fn marks(dir: &Path) -> Result<Option<Listing>, String> {
    let Ok(root) = git(dir, None, &["rev-parse", "--show-toplevel"]) else { return Ok(None) };
    let root = fs::canonicalize(root.trim()).map_err(|e| e.to_string())?;
    let within = fs::canonicalize(dir).map_err(|e| e.to_string())?;
    let Ok(within) = within.strip_prefix(&root).map(|rest| rest.to_string_lossy().replace('\\', "/")) else {
        return Ok(None);
    };
    let index = git(&root, None, &["rev-parse", "--path-format=absolute", "--git-path", "index"])?;
    let base = match git(&root, None, &["rev-parse", "--verify", "--quiet", "HEAD"]) {
        Ok(_) => "HEAD".to_string(),
        Err(_) => git(&root, None, &["hash-object", "-t", "tree", "/dev/null"])?.trim().to_string(),
    };
    let copy = IndexCopy::new(Path::new(index.trim()))?;
    let index = Some(copy.file.as_path());
    let statuses = git(
        &root,
        index,
        &["diff", &base, "--name-status", "-z", "--no-ext-diff", "--no-textconv", "--no-color", "-M"],
    )?;
    let untracked = git(&root, index, &["ls-files", "--others", "--exclude-standard", "-z"])?;
    let ignored = git(&root, index, &["ls-files", "--others", "--ignored", "--exclude-standard", "--directory", "-z"])?;
    let changes = parse_name_status(&statuses)
        .into_iter()
        .chain(split_z(&untracked).into_iter().map(|path| (path.to_string(), ChangeKind::Untracked)))
        .collect();
    let ignored = split_z(&ignored).into_iter().map(str::to_string).collect();
    Ok(Some(Listing { within, changes, ignored }))
}

/// Runs one read-only git command in `dir`, with Scouter's options and environment (`src/diff/git.ts:24-45`).
/// `--no-optional-locks` keeps polling from taking the user's index lock.
fn git(dir: &Path, index: Option<&Path>, args: &[&str]) -> Result<String, String> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(dir)
        .args(["--no-pager", "--literal-pathspecs", "-c", "core.fsmonitor=false", "-c", "core.quotepath=false"])
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat")
        .env_remove("GIT_EXTERNAL_DIFF")
        .stdin(Stdio::null());
    if let Some(index) = index {
        command.env("GIT_INDEX_FILE", index);
    }
    let output = command.output().map_err(|_| "Git is not installed or could not be started.".to_string())?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr
            .lines()
            .next()
            .map_or_else(|| format!("git exited with status {}.", output.status), str::to_string));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
#[path = "git_tests.rs"]
mod tests;

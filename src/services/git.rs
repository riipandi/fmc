//! Git reads for the changes screen and the tree's marks (Scouter `src/diff/git.ts` and `src/diff/patch.ts`). Git never
//! reads the repository's own index: a `Repository` keeps a copy and runs every command against it, because `git diff`
//! rewrites the index it reads (`architecture.md`, "Git is read by running `git`"; Scouter decision 0007). The parsing
//! and the patch text are in `fmc::diff`.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use fmc::diff::{
    ALL_LIMIT, Change, ChangeKind, ChangeSet, FilePatch, Listing, PATCH_LIMIT, Patch, binary, change_set, from_diff,
    from_file, not_shown, order, parse_name_status, split, split_z, too_large, utf16_len,
};
use fmc::reader::document::{self, Document};
use fmc::text::display_text;

use super::fs::describe;

/// Scouter's cap on a git call's output (`src/diff/git.ts:8`).
const LIST_LIMIT: usize = 8 * 1024 * 1024;
/// Scouter's time limit for a git call (`src/diff/git.ts:9`).
const TIMEOUT: Duration = Duration::from_secs(15);
/// An index written this recently can hide a same-second change from git's stat check, so its copy is never reused
/// (`src/diff/git.ts:10`, Scouter decision 0007).
const RACY_WINDOW: Duration = Duration::from_secs(2);
/// How often a running command is checked for its exit, for an overflow, and for the timeout.
const POLL: Duration = Duration::from_millis(5);
/// The status Bun reports for a process ended by SIGTERM, which is how Scouter's timeout ends git (`src/diff/git.ts:59`).
const TERMINATED: i32 = 143;
/// Scouter's diff options (`src/diff/git.ts:32`).
const DIFF_OPTIONS: [&str; 4] = ["--no-ext-diff", "--no-textconv", "--no-color", "-M"];
const NOT_A_REPOSITORY: &str = "Not a git repository.";
const NO_OUTPUT: &str = "Git output could not be read.";

/// A repository a worker reads. Each git call holds its queue, so calls on one repository never overlap
/// (`src/diff/git.ts:141-148`). Dropping it removes the copy of the index.
pub struct Repository {
    root: PathBuf,
    /// The repository's own index, which only the copy in `copy_dir` stands in for.
    index: PathBuf,
    copy_dir: PathBuf,
    queue: Mutex<Queue>,
}

/// What the queue keeps between calls: the base the last listing diffed against (a patch diffs against it too), and the
/// stamps that decide whether the copy is made again.
struct Queue {
    base: String,
    stamp: Option<Stamp>,
}

/// The stats of the index a copy was made from, and of the copy as it was left.
#[derive(Clone, Copy)]
struct Stamp {
    source: Key,
    copy: Key,
}

/// The stat fields that tell two versions of a file apart: inode, size, mtime, and ctime.
type Key = (u64, u64, i64, i64, i64, i64);

fn key(metadata: &fs::Metadata) -> Key {
    (metadata.ino(), metadata.size(), metadata.mtime(), metadata.mtime_nsec(), metadata.ctime(), metadata.ctime_nsec())
}

/// What one git call printed: its text, and whether the cap cut it (`src/diff/git.ts:76`).
struct Output {
    text: String,
    truncated: bool,
}

impl Repository {
    /// The repository that holds `dir`, with a copy directory for its index. Outside one, the error is
    /// `Not a git repository.` (`src/diff/git.ts:109-127`).
    pub fn open(dir: &Path) -> Result<Self, String> {
        let root = match run(dir, None, &["rev-parse", "--show-toplevel"]) {
            Ok(output) => output.text.trim().to_string(),
            Err(message) if message.to_lowercase().contains("not a git repository") => String::new(),
            Err(message) => return Err(message),
        };
        if root.is_empty() {
            return Err(NOT_A_REPOSITORY.into());
        }
        let root = PathBuf::from(root);
        let index = run(&root, None, &["rev-parse", "--path-format=absolute", "--git-path", "index"])?;
        let base = resolve_base(&root)?;
        let copy_dir = create_copy_dir()?;
        Ok(Self {
            root,
            index: PathBuf::from(index.text.trim()),
            copy_dir,
            queue: Mutex::new(Queue { base, stamp: None }),
        })
    }

    /// The repository's root folder, which Scouter names in the empty state's footer.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Whether the base is `HEAD`, not the empty tree of a repository without commits (Scouter's `base`).
    pub fn on_head(&self) -> bool {
        self.queue().base == "HEAD"
    }

    /// The change list against the base of this call (`src/diff/git.ts:204-236`).
    pub fn changes(&self) -> Result<ChangeSet, String> {
        let mut queue = self.queue();
        queue.base = resolve_base(&self.root)?;
        self.sync(&mut queue);
        let base = queue.base.as_str();
        let statuses = self.copy_git(&diff_args(&["diff", base, "--name-status", "-z"]))?;
        let numstat = self.copy_git(&diff_args(&["diff", base, "--numstat", "-z"]))?;
        let untracked = self.copy_git(&["ls-files", "--others", "--exclude-standard", "-z"])?;
        let staged = self.copy_git(&diff_args(&["diff", "--cached", base, "--name-only", "-z"]))?;
        let unstaged = self.copy_git(&diff_args(&["diff", "--name-only", "-z"]))?;
        let cut = statuses.truncated || numstat.truncated || untracked.truncated;
        Ok(change_set(&statuses.text, &numstat.text, &untracked.text, &staged.text, &unstaged.text, cut))
    }

    /// The marks of the repository, with paths relative to `within` (`src/diff/git.ts:238-260`).
    fn listing(&self, within: String) -> Result<Listing, String> {
        let mut queue = self.queue();
        queue.base = resolve_base(&self.root)?;
        self.sync(&mut queue);
        let base = queue.base.as_str();
        let statuses = self.copy_git(&diff_args(&["diff", base, "--name-status", "-z"]))?;
        let untracked = self.copy_git(&["ls-files", "--others", "--exclude-standard", "-z"])?;
        let ignored =
            self.copy_git(&["ls-files", "--others", "--ignored", "--exclude-standard", "--directory", "-z"])?;
        let mut changes = parse_name_status(&statuses.text, &HashMap::new());
        changes.extend(split_z(&untracked.text).into_iter().map(Change::untracked));
        order(&mut changes);
        changes.truncate(fmc::diff::MAX_CHANGES);
        let ignored = split_z(&ignored.text).into_iter().map(str::to_string).collect();
        Ok(Listing { within, changes, ignored })
    }

    /// One change's patch against the base of the last listing (`src/diff/patch.ts:13-20`). The copy is the one that
    /// listing made; a patch does not refresh it.
    pub fn patch(&self, change: &Change, context: usize) -> Result<Patch, String> {
        if change.binary {
            return Ok(binary());
        }
        if change.kind == ChangeKind::Untracked {
            return Ok(self.new_file(change));
        }
        let queue = self.queue();
        let unified = format!("-U{context}");
        let mut args = diff_args(&["diff", unified.as_str(), queue.base.as_str()]);
        args.push("--");
        if let Some(old) = change.old_path.as_deref().filter(|path| !path.is_empty()) {
            args.push(old);
        }
        args.push(&change.path);
        let output = run_limited(&self.root, Some(&self.copy_index()), &args, PATCH_LIMIT, false)?;
        Ok(if output.truncated { too_large() } else { from_diff(change, &output.text) })
    }

    /// Every change's patch (`src/diff/patch.ts:22-73`). One whole-tree diff of at most 4 MiB is split by file; the
    /// untracked files share a budget of the same size. A file past the budget, or past the cut, is `not_shown`.
    pub fn patches(&self, changes: &[Change], context: usize) -> Result<Vec<FilePatch>, String> {
        let whole = if changes.iter().any(|change| change.kind != ChangeKind::Untracked && !change.binary) {
            let queue = self.queue();
            let unified = format!("-U{context}");
            let args = diff_args(&["diff", unified.as_str(), queue.base.as_str()]);
            Some(run_limited(&self.root, Some(&self.copy_index()), &args, ALL_LIMIT, true)?)
        } else {
            None
        };
        let (chunks, cut) = match &whole {
            Some(output) => (split(&output.text, output.truncated), output.truncated),
            None => (HashMap::new(), false),
        };
        let mut budget = ALL_LIMIT as i64;
        let mut new_files = HashMap::new();
        for change in changes.iter().filter(|change| change.kind == ChangeKind::Untracked && !change.binary) {
            if budget <= 0 {
                break;
            }
            let patch = self.new_file(change);
            if let Patch::Text { text, .. } = &patch {
                budget -= utf16_len(text) as i64;
            }
            new_files.insert(change.path.as_str(), patch);
        }
        let mut result = Vec::with_capacity(changes.len());
        for change in changes {
            let patch = if change.binary {
                binary()
            } else if change.kind == ChangeKind::Untracked {
                new_files.remove(change.path.as_str()).unwrap_or_else(not_shown)
            } else if let Some(chunk) = chunks.get(change.path.as_str()) {
                from_diff(change, chunk)
            } else if cut {
                not_shown()
            } else {
                self.patch(change, context)?
            };
            result.push(FilePatch { change: change.clone(), patch });
        }
        Ok(result)
    }

    fn new_file(&self, change: &Change) -> Patch {
        from_file(&change.path, read_new_file(&self.root.join(&change.path)))
    }

    /// Scouter's `syncIndex` (`src/diff/git.ts:156-198`). A failure leaves no copy, which git reads as an empty index,
    /// as in Scouter.
    fn sync(&self, queue: &mut Queue) {
        if refresh_copy(&self.index, &self.copy_index(), &mut queue.stamp).is_err() {
            queue.stamp = None;
            let _ = fs::remove_file(self.copy_index());
        }
    }

    /// Runs a command against this repository's copy of the index (Scouter's `runGit` with `indexFile`).
    fn copy_git(&self, args: &[&str]) -> Result<Output, String> {
        run(&self.root, Some(&self.copy_index()), args)
    }

    fn copy_index(&self) -> PathBuf {
        self.copy_dir.join("index")
    }

    /// The repository's queue. A poisoned lock still holds a base and a stamp that the next call can use: a stale stamp
    /// only copies the index again.
    fn queue(&self) -> MutexGuard<'_, Queue> {
        self.queue.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.copy_dir);
    }
}

/// Git's marks for the repository that holds `dir`, relative to `dir`. `None` when `dir` is outside a repository or the
/// repository cannot be opened (Scouter's `openRepository(root).catch`, `src/diff/marks.ts:38`).
pub fn marks(dir: &Path) -> Result<Option<Listing>, String> {
    let Ok(repository) = Repository::open(dir) else { return Ok(None) };
    let root = fs::canonicalize(&repository.root).map_err(|error| error.to_string())?;
    let within = fs::canonicalize(dir).map_err(|error| error.to_string())?;
    let Ok(within) = within.strip_prefix(&root).map(|rest| rest.to_string_lossy().replace('\\', "/")) else {
        return Ok(None);
    };
    repository.listing(within).map(Some)
}

/// `HEAD` when the repository has a commit, else the empty tree, so a repository with no commits lists everything as
/// added (`src/diff/git.ts:129-139`).
fn resolve_base(root: &Path) -> Result<String, String> {
    let head =
        run(root, None, &["rev-parse", "--verify", "--quiet", "HEAD"]).map(|output| output.text.trim().to_string());
    if head.is_ok_and(|head| !head.is_empty()) {
        return Ok("HEAD".into());
    }
    Ok(run(root, None, &["hash-object", "-t", "tree", "/dev/null"])?.text.trim().to_string())
}

fn create_copy_dir() -> Result<PathBuf, String> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let dir =
        std::env::temp_dir().join(format!("fmc-index-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::create_dir(&dir).map_err(|error| error.to_string())?;
    Ok(dir)
}

/// The arguments followed by Scouter's diff options.
fn diff_args<'a>(head: &[&'a str]) -> Vec<&'a str> {
    head.iter().copied().chain(DIFF_OPTIONS).collect()
}

/// Copies `source` to `copy` unless the last copy still stands: the index and the copy are unchanged since it was made,
/// and the index is past the racy window (Scouter `syncIndex`, `src/diff/git.ts:162-179`).
fn refresh_copy(source: &Path, copy: &Path, stamp: &mut Option<Stamp>) -> io::Result<()> {
    let metadata = fs::metadata(source)?;
    let source_key = key(&metadata);
    let modified = metadata.modified()?;
    let copied = fs::metadata(copy).ok().map(|copied| key(&copied));
    if reusable(*stamp, source_key, modified, copied, SystemTime::now()) {
        return Ok(());
    }
    fs::copy(source, copy)?;
    // The copy keeps the index's mtime, which git's stat check compares (`src/diff/git.ts:183`). Opened read-only, so an
    // index that is itself read-only still gets its time.
    File::open(copy)?.set_modified(modified)?;
    *stamp = Some(Stamp { source: source_key, copy: key(&fs::metadata(copy)?) });
    Ok(())
}

fn reusable(stamp: Option<Stamp>, source: Key, modified: SystemTime, copy: Option<Key>, now: SystemTime) -> bool {
    let Some(stamp) = stamp else { return false };
    let settled = now.duration_since(modified).is_ok_and(|age| age > RACY_WINDOW);
    stamp.source == source && settled && copy == Some(stamp.copy)
}

/// An untracked file as Scouter's `readDocument` reads it: a regular file, at most 1 MiB, and text only
/// (`src/core/files.ts:90-113`).
fn read_new_file(path: &Path) -> Result<Document, String> {
    let metadata = fs::metadata(path).map_err(|error| describe(&error, "open", Some(path)))?;
    if !metadata.is_file() {
        return Err("Only regular text files can be viewed.".into());
    }
    let file = File::open(path).map_err(|error| describe(&error, "open", Some(path)))?;
    let mut bytes = Vec::new();
    file.take(document::MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| describe(&error, "read", None))?;
    document::decode(&bytes).map_err(|refusal| refusal.message().to_string())
}

/// Runs git with Scouter's listing cap, and returns its text.
fn run(dir: &Path, index: Option<&Path>, args: &[&str]) -> Result<Output, String> {
    run_limited(dir, index, args, LIST_LIMIT, false)
}

fn run_limited(dir: &Path, index: Option<&Path>, args: &[&str], limit: usize, partial: bool) -> Result<Output, String> {
    capture(command(dir, index, args), limit, partial, TIMEOUT)
}

/// Scouter's read-only options and environment (`src/diff/git.ts:24-45`). `--no-optional-locks` keeps a poll from
/// taking the repository's index lock; the environment variable stays for git's subprocesses.
fn command(dir: &Path, index: Option<&Path>, args: &[&str]) -> Command {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(dir)
        .args([
            "--no-pager",
            "--literal-pathspecs",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.quotepath=false",
            "--no-optional-locks",
        ])
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat")
        .env_remove("GIT_EXTERNAL_DIFF");
    if let Some(index) = index {
        command.env("GIT_INDEX_FILE", index);
    }
    command
}

/// Runs `command` and collects its stdout up to `limit` bytes. The command is killed when it writes past the limit or
/// runs past `timeout`. Past the limit, a `partial` run keeps the bytes it read; any other run is cut and has no text
/// (`src/diff/git.ts:47-107`).
fn capture(mut command: Command, limit: usize, partial: bool, timeout: Duration) -> Result<Output, String> {
    command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|_| "Git is not installed or could not be started.".to_string())?;
    let stdout = child.stdout.take().ok_or(NO_OUTPUT)?;
    let mut stderr = child.stderr.take().ok_or(NO_OUTPUT)?;
    let overflow = Arc::new(AtomicBool::new(false));
    let reader = {
        let overflow = Arc::clone(&overflow);
        thread::spawn(move || read_capped(stdout, limit, &overflow))
    };
    // stderr is drained beside stdout, so a command that fills either pipe cannot wait on this loop.
    let errors = thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stderr.read_to_end(&mut bytes);
        bytes
    });
    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            break status;
        }
        if overflow.load(Ordering::SeqCst) || Instant::now() >= deadline {
            let _ = child.kill();
            break child.wait().map_err(|error| error.to_string())?;
        }
        thread::sleep(POLL);
    };
    let mut bytes = reader.join().map_err(|_| NO_OUTPUT.to_string())?.map_err(|error| error.to_string())?;
    let stderr = errors.join().unwrap_or_default();
    let truncated = overflow.load(Ordering::SeqCst);
    if !truncated && !status.success() {
        let code = status.code().unwrap_or(TERMINATED);
        let stderr = String::from_utf8_lossy(&stderr);
        let reason = stderr.trim().split('\n').next().unwrap_or_default();
        return Err(if reason.is_empty() { format!("git exited with status {code}.") } else { display_text(reason) });
    }
    if truncated {
        bytes.truncate(limit);
    }
    let text = if truncated && !partial { String::new() } else { String::from_utf8_lossy(&bytes).into_owned() };
    Ok(Output { text, truncated })
}

/// Reads stdout up to `limit` bytes, and one byte past it, so a cut is told from an output that ends exactly there.
/// A read past the limit sets `overflow`, and the waiting loop kills the command.
fn read_capped(stdout: ChildStdout, limit: usize, overflow: &AtomicBool) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    stdout.take(limit as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        overflow.store(true, Ordering::SeqCst);
    }
    Ok(bytes)
}

#[cfg(test)]
#[path = "git_tests.rs"]
mod tests;

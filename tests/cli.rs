//! The command line, end to end (Scouter `src/index.ts`).

use std::os::unix::net::UnixListener;
use std::process::Command;

use assert_cmd::cargo::cargo_bin;
use assert_cmd::prelude::*;

fn fmc() -> Command {
    Command::new(cargo_bin("fmc"))
}

/// Runs fmc on a pseudo-terminal, so the interactive-terminal check passes. Output arrives with `\r\n`.
fn fmc_on_tty(arg: &str) -> std::process::Output {
    Command::new("script").args(["-q", "/dev/null"]).arg(cargo_bin("fmc")).arg(arg).output().unwrap()
}

#[test]
fn help_prints_usage_and_every_key() {
    let out = fmc().args(["the-louvre", "--help"]).assert().success().get_output().stdout.clone();
    let text = String::from_utf8(out).unwrap();
    assert!(text.starts_with("fmc — a minimal read-only terminal file viewer\n\nUsage: fmc [file-or-directory]\n"));
    assert!(text.contains("\nBrowse\n  j k ↑ ↓        move\n"));
    assert!(text.ends_with("Mouse\n  Left click     open a file or toggle a folder\n  Drag text      select and copy to clipboard\n  Wheel          scroll the pane under the pointer\n  Drag edge      resize the list\n\n"));
}

#[test]
fn version() {
    fmc().arg("-v").assert().success().stdout(format!("fmc {}\n", env!("CARGO_PKG_VERSION")));
}

#[test]
fn two_paths_fail() {
    fmc()
        .args(["hogwarts", "gringotts"])
        .assert()
        .code(1)
        .stderr("fmc: Expected one file or directory. Use --help for usage.\n");
}

#[test]
fn not_a_terminal_fails_before_the_path_check() {
    fmc().arg("chamber-of-secrets").assert().code(1).stderr("fmc: Run fmc in an interactive terminal.\n");
}

#[test]
fn missing_path_fails() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().canonicalize().unwrap().join("chamber-of-secrets");
    let out = fmc_on_tty(missing.to_str().unwrap());
    let text = String::from_utf8_lossy(&out.stdout);
    let expected = format!("\x1b[31mfmc: ENOENT: no such file or directory, stat '{}'\x1b[0m", missing.display());
    assert!(text.contains(&expected), "{text:?}");
}

#[test]
fn socket_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("owl-post.sock");
    let _listener = UnixListener::bind(&socket).unwrap();
    let out = fmc_on_tty(socket.to_str().unwrap());
    assert!(String::from_utf8_lossy(&out.stdout).contains("fmc: Only files and directories can be opened."));
}

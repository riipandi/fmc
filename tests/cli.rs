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
fn help_prints_usage_and_points_to_the_keys() {
    let out = fmc().args(["the-louvre", "--help"]).assert().success().get_output().stdout.clone();
    let text = String::from_utf8(out).unwrap();
    // The keys are on the `?` page, not here (the owner, 2026-11-27).
    let expected = "fmc — a minimal terminal file manager\n\n\
     Usage: fmc [file-or-directory]\n\n\
     Arguments:\n  \
       [file-or-directory]  The folder to browse, or a file to open in the reader.\n                       \
       Defaults to the current folder.\n\n\
     Options:\n  \
       -h, --help           Print this help.\n  \
       -v, --version        Print the version.\n\n\
     Press ? inside fmc for the keys.\n\
     Settings: $XDG_CONFIG_HOME/fmc/settings.json, or ~/.config/fmc/settings.json.\n\
     Docs: https://github.com/riipandi/fmc/tree/main/docs\n";
    assert_eq!(text, expected);
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

/// Runs fmc on a pseudo-terminal with `input` typed once it has entered the alternate screen (the setup that
/// also puts the terminal in raw mode); returns everything fmc wrote.
fn session(env: &[(&str, &str)], input: &str) -> String {
    use std::io::{Read, Write};
    use std::process::Stdio;
    use std::sync::mpsc;
    const ALTERNATE_ON: &[u8] = b"\x1b[?1049h";
    let dir = tempfile::tempdir().unwrap();
    let mut child = Command::new("script")
        .args(["-q", "/dev/null"])
        .arg(cargo_bin("fmc"))
        .arg(dir.path())
        .envs(env.iter().copied())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let (entered, ready) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut out = Vec::new();
        let mut chunk = [0; 4096];
        let mut sent = false;
        while let Ok(n @ 1..) = stdout.read(&mut chunk) {
            out.extend_from_slice(&chunk[..n]);
            if !sent && out.windows(ALTERNATE_ON.len()).any(|w| w == ALTERNATE_ON) {
                sent = true;
                let _ = entered.send(());
            }
        }
        out
    });
    ready.recv_timeout(std::time::Duration::from_secs(20)).expect("fmc entered the alternate screen");
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(input.as_bytes()).unwrap();
    drop(stdin);
    child.wait().unwrap();
    String::from_utf8_lossy(&reader.join().unwrap()).into_owned()
}

#[test]
fn quit_restores_the_terminal() {
    let out = session(&[], "\x11");
    let entered = out.find("\x1b[?1049h").expect("alternate screen entered");
    assert!(out[entered..].contains("\x1b[?1049l"), "alternate screen left");
}

// The panic hook is debug-only (`FMC_PANIC_AFTER_START`); a release fmc never panics and the session waits forever.
#[cfg(debug_assertions)]
#[test]
fn a_panic_restores_the_terminal_before_its_message() {
    let out = session(&[("FMC_PANIC_AFTER_START", "1")], "");
    let left = out.rfind("\x1b[?1049l").expect("alternate screen left");
    let message = out.find("Peeves was here").expect("panic message printed");
    assert!(left < message, "the terminal is restored before the panic message");
}

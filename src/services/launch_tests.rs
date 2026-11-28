//! The argv of each launch, the listing's parser, and the script run on a temporary text file. No test launches an app:
//! `open` runs only on a missing file, which it refuses.

use std::fs;
use std::path::Path;

use super::*;

const HOME: &str = "/Users/langdon";
const TARGET: &str = "/Users/langdon/Notes/Symbologist.md";

fn argv(args: &[&str]) -> Vec<OsString> {
    args.iter().map(OsString::from).collect()
}

#[test]
fn each_how_builds_its_argv() {
    let target = Path::new(TARGET);
    assert_eq!(open_args(target, How::Default), argv(&["--", TARGET]));
    assert_eq!(open_args(target, How::App("Zed")), argv(&["-a", "Zed", "--", TARGET]));
    assert_eq!(
        open_args(target, How::App("/Applications/Zed.app")),
        argv(&["-a", "/Applications/Zed.app", "--", TARGET])
    );
    assert_eq!(open_args(target, How::Reveal), argv(&["-R", "--", TARGET]));
}

#[test]
fn a_path_that_starts_with_a_dash_comes_after_the_separator() {
    assert_eq!(open_args(Path::new("-Notes.md"), How::Reveal), argv(&["-R", "--", "-Notes.md"]));
    assert_eq!(open_args(Path::new("-Notes.md"), How::Default), argv(&["--", "-Notes.md"]));
}

#[test]
fn the_listing_keeps_the_application_folders_with_the_default_first() {
    let text = [
        " \tMarauder's Map\tcom.example.map\t/System/Applications/Marauders.app",
        "*\tPensieve\t-\t/Applications/Pensieve.app",
        " \tGringotts Vault\tcom.example.vault\t/Users/langdon/Applications/Gringotts.app",
        " \tChrome for Testing\tcom.example.chrome\t/Users/langdon/Library/Caches/ms-playwright/Chrome.app",
        " \tLookalike\t-\t/ApplicationsX/Lookalike.app",
        " \tNo Path\t-\t",
    ]
    .join("\n");
    let apps = parse_apps(&text, Path::new(HOME));
    let names: Vec<(&str, bool)> = apps.iter().map(|app| (app.name.as_str(), app.default)).collect();
    assert_eq!(names, [("Pensieve", true), ("Gringotts Vault", false), ("Marauder's Map", false)]);
    assert_eq!(apps[0].path, Path::new("/Applications/Pensieve.app"));
}

#[test]
fn a_listing_with_no_apps_is_empty() {
    assert!(parse_apps("(none)", Path::new(HOME)).is_empty());
    assert!(parse_apps("", Path::new(HOME)).is_empty());
}

#[test]
fn the_script_lists_well_formed_lines_for_a_text_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("Hogwarts.txt");
    fs::write(&file, "Hogwarts\n").unwrap();
    let text = capture(listing_command(&file), APPS_LIMIT, false, TIMEOUT).unwrap().text;
    assert!(text == "(none)" || text.lines().all(|line| line.split('\t').count() == 4), "{text}");
    assert!(text.lines().filter(|line| line.starts_with('*')).count() <= 1, "{text}");
}

#[test]
fn a_path_with_shell_syntax_is_only_a_path() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("Hogwarts \"$(touch PWNED)\".txt");
    fs::write(&file, "Hogwarts\n").unwrap();
    let mut command = listing_command(&file);
    command.current_dir(dir.path());
    let text = capture(command, APPS_LIMIT, false, TIMEOUT).unwrap().text;
    assert!(!dir.path().join("PWNED").exists(), "the path ran as a shell command");
    assert!(text == "(none)" || text.lines().all(|line| line.split('\t').count() == 4), "{text}");
}

#[test]
fn a_missing_file_is_refused_by_open() {
    let error = run_open(&open_args(Path::new("/nonexistent-fmc/Hogwarts.txt"), How::Default)).unwrap_err();
    assert!(!error.is_empty());
}

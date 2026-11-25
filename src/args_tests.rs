use super::*;

fn args(list: &[&str]) -> Vec<OsString> {
    list.iter().map(OsString::from).collect()
}

#[test]
fn help_wins_anywhere_then_version() {
    assert_eq!(parse(args(&["vault", "-h", "map"])), Ok(Command::Help));
    assert_eq!(parse(args(&["--version", "--help"])), Ok(Command::Help));
    assert_eq!(parse(args(&["-v", "vault", "map"])), Ok(Command::Version));
}

#[test]
fn one_path_at_most() {
    assert_eq!(parse(args(&[])), Ok(Command::Open(None)));
    assert_eq!(parse(args(&["-x"])), Ok(Command::Open(Some("-x".into()))));
    assert_eq!(parse(args(&["-hv"])), Ok(Command::Open(Some("-hv".into()))));
    assert_eq!(parse(args(&["--"])), Ok(Command::Open(Some("--".into()))));
    assert_eq!(parse(args(&["louvre", "cern"])), Err("Expected one file or directory. Use --help for usage.".into()));
}

#[test]
fn resolve_folds_dots() {
    assert_eq!(
        resolve(Path::new("/hogwarts/tower"), Path::new("../library/./restricted")),
        PathBuf::from("/hogwarts/library/restricted")
    );
    assert_eq!(resolve(Path::new("/hogwarts"), Path::new("")), PathBuf::from("/hogwarts"));
    assert_eq!(resolve(Path::new("/hogwarts"), Path::new("/louvre")), PathBuf::from("/louvre"));
}

#[test]
fn missing_path_reads_like_bun() {
    let err = open_target(Some("chamber-of-secrets".into()), Path::new("/hogwarts")).unwrap_err();
    assert_eq!(err, "ENOENT: no such file or directory, stat '/hogwarts/chamber-of-secrets'");
}

#[test]
fn only_files_and_directories() {
    assert_eq!(
        open_target(Some("/dev/null".into()), Path::new("/")),
        Err("Only files and directories can be opened.".into())
    );
}

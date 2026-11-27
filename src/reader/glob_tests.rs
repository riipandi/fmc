use super::*;

/// Whether the pattern matches a name at the top (no path), and a relative path.
fn name_matches(pattern: &str, name: &str) -> bool {
    Glob::parse(pattern).expect("a valid pattern").matches(None, name)
}

fn path_matches(pattern: &str, relative: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    Glob::parse(pattern).expect("a valid pattern").matches(Some(relative), name)
}

#[test]
fn a_star_matches_any_run_within_a_name() {
    assert!(name_matches("*.rs.bak", "gringotts.rs.bak"));
    assert!(name_matches("*.rs.bak", ".rs.bak"));
    assert!(!name_matches("*.rs.bak", "gringotts.rs"));
    assert!(name_matches(".env.*", ".env.hogwarts"));
    assert!(!name_matches(".env.*", "hogwarts.env"));
    assert!(name_matches("*a*b", "xaxab"));
    assert!(!name_matches("*a*b", "xaxa"));
}

#[test]
fn a_question_mark_matches_one_character() {
    assert!(name_matches("vault?.rs", "vault1.rs"));
    assert!(!name_matches("vault?.rs", "vault.rs"));
    assert!(!name_matches("vault?.rs", "vault12.rs"));
}

#[test]
fn braces_match_any_alternative() {
    assert!(name_matches("*.{env,env.example}", "louvre.env"));
    assert!(name_matches("*.{env,env.example}", "louvre.env.example"));
    assert!(!name_matches("*.{env,env.example}", "louvre.example"));
    assert!(name_matches("{Gringotts,Louvre}.txt", "Louvre.txt"));
    assert!(name_matches("a{b{c,d},e}", "abd"));
}

#[test]
fn a_name_pattern_matches_the_file_name_at_any_depth() {
    assert!(path_matches("*.rs.bak", "hogwarts/gringotts.rs.bak"));
    assert!(path_matches(".env.*", "a/b/.env.example"));
}

#[test]
fn a_path_pattern_matches_from_the_opened_folder() {
    assert!(path_matches("docs/*.txt", "docs/hogwarts.txt"));
    assert!(!path_matches("docs/*.txt", "src/docs/hogwarts.txt"));
    assert!(!path_matches("docs/*.txt", "docs/deep/hogwarts.txt"));
}

#[test]
fn a_double_star_matches_any_depth_and_zero_folders() {
    assert!(path_matches("docs/**/*.txt", "docs/hogwarts.txt"));
    assert!(path_matches("docs/**/*.txt", "docs/a/b/hogwarts.txt"));
    assert!(!path_matches("docs/**/*.txt", "notes/hogwarts.txt"));
    assert!(path_matches("**/*.env", "louvre.env"));
    assert!(path_matches("**/*.env", "a/b/louvre.env"));
}

#[test]
fn a_star_does_not_cross_a_slash() {
    assert!(!path_matches("docs/*", "docs/a/b.txt"));
    assert!(path_matches("*.txt", "docs/hogwarts.txt"));
}

#[test]
fn a_path_pattern_does_not_match_a_file_outside_the_folder() {
    let glob = Glob::parse("docs/*.txt").expect("a valid pattern");
    assert!(!glob.matches(None, "hogwarts.txt"));
}

#[test]
fn case_follows_the_filesystem() {
    let expected = CASE_INSENSITIVE;
    assert_eq!(name_matches("*.RS.BAK", "gringotts.rs.bak"), expected);
    assert_eq!(path_matches("DOCS/*.txt", "docs/hogwarts.txt"), expected);
    assert!(name_matches("*.rs.bak", "gringotts.rs.bak"));
}

#[test]
fn a_bad_pattern_is_refused_with_its_reason() {
    assert_eq!(Glob::parse("").unwrap_err(), "the pattern is empty");
    assert_eq!(Glob::parse("a{b,c").unwrap_err(), "a `{` has no matching `}`");
    let many = "{a,b}".repeat(7);
    assert!(Glob::parse(&many).unwrap_err().contains("more than 64 alternatives"));
}

#[test]
fn a_pattern_with_a_slash_is_a_path_pattern() {
    assert!(Glob::parse("docs/*.txt").expect("a valid pattern").is_path);
    assert!(!Glob::parse("*.txt").expect("a valid pattern").is_path);
}

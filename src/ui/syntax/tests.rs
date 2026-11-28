use std::path::Path;

use fmc::reader::associations::Associations;
use fmc::theme::DARK;
use opentui::prelude::{Attributes, Color};
use opentui::syntax::{has_grammar, highlight, highlights, prewarm};

use super::*;

#[test]
fn every_vendored_grammar_compiles_with_its_query() {
    register();
    for filetype in ["json", "css", "html", "go", "python", "bash", "yaml", "toml", "c", "java"] {
        assert!(prewarm(filetype), "{filetype} does not compile");
    }
    for filetype in ["javascript", "typescript", "typescriptreact", "markdown", "markdown_inline", "zig"] {
        assert!(prewarm(filetype), "{filetype} does not compile");
    }
    for (filetype, _) in COPIED {
        assert!(prewarm(filetype), "{filetype} does not compile");
    }
    assert!(has_grammar("javascriptreact") && has_grammar("rust"));
}

/// The grammars whose queries are copied into `queries/`, each with the query it is registered with: the owner's twelve
/// (RI-2), then the query and JSON5 grammars (plan-00018).
const COPIED: [(&str, &str); 14] = [
    ("diff", include_str!("queries/diff/highlights.scm")),
    ("elixir", include_str!("queries/elixir/highlights.scm")),
    ("heex", include_str!("queries/heex/highlights.scm")),
    ("hcl", include_str!("queries/hcl/highlights.scm")),
    ("jsdoc", include_str!("queries/jsdoc/highlights.scm")),
    ("lua", include_str!("queries/lua/highlights.scm")),
    ("regex", include_str!("queries/regex/highlights.scm")),
    ("sql", include_str!("queries/sql/highlights.scm")),
    ("svelte", include_str!("queries/svelte/highlights.scm")),
    ("xml", include_str!("queries/xml/highlights.scm")),
    ("astro", include_str!("queries/astro/highlights.scm")),
    ("vim", include_str!("queries/vim/highlights.scm")),
    ("query", include_str!("queries/query/highlights.scm")),
    ("json5", include_str!("queries/json5/highlights.scm")),
];

/// The names a query captures, from each `@name`. A name starting with `_` is a predicate's helper (`#eq? @_scope`),
/// which is never drawn.
fn capture_names(query: &str) -> Vec<&str> {
    query
        .split('@')
        .skip(1)
        .filter_map(|rest| {
            let end = rest.find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '.'))).unwrap_or(rest.len());
            let name = rest[..end].trim_end_matches('.');
            (!name.is_empty()).then_some(name)
        })
        .collect()
}

#[test]
fn every_capture_of_the_copied_queries_has_a_table_entry() {
    for (filetype, query) in COPIED {
        for name in capture_names(query) {
            assert!(
                name.starts_with('_') || CAPTURE_NAMES.contains(&name),
                "{filetype} captures {name}, not in the table"
            );
        }
    }
}

#[test]
fn the_plan_18_queries_capture_only_names_the_palette_styles() {
    let theme = theme(&DARK);
    for (filetype, query) in COPIED.into_iter().filter(|(filetype, _)| matches!(*filetype, "query" | "json5")) {
        for name in capture_names(query) {
            assert!(
                name.starts_with('_') || theme.get(name).is_some(),
                "{filetype} captures {name}, which the palette does not style"
            );
        }
    }
}

#[test]
fn the_owners_grammars_style_a_token_of_their_sample() {
    register();
    let theme = theme(&DARK);
    // Each token takes a capture the palette styles, and the capture is the one the query names for it.
    let cases = [
        (
            "elixir",
            "defmodule Hogwarts.Owl do\n  def deliver(letter), do: {:ok, letter}\nend\n",
            "defmodule",
            "keyword",
        ),
        ("lua", "local function open(vault) return vault.key end\n", "return", "keyword.return"),
        ("sql", "SELECT name FROM louvre_keys WHERE holder = 'Sophie Neveu';\n", "SELECT", "keyword"),
        ("xml", "<?xml version=\"1.0\"?>\n<owl name=\"Hedwig\"/>\n", "owl", "tag"),
        ("hcl", "resource \"aws_s3_bucket\" \"gringotts\" {\n  bucket = \"vault-713\"\n}\n", "resource", "keyword"),
        ("astro", "---\nconst name = 'Albus Dumbledore';\n---\n<h1>{name}</h1>\n", "h1", "tag"),
        ("vim", "\" Opens the Louvre\nfunction! Open(key)\n  let g:key = a:key\nendfunction\n", "function", "keyword"),
        ("diff", "--- a/owls.txt\n+++ b/owls.txt\n@@ -1 +1 @@\n-Hedwig\n+Crookshanks\n", "+Crookshanks", "string"),
        ("heex", "<div class=\"owl\">\n  <%= @name %>\n</div>\n", "div", "tag"),
        ("regex", "^[A-Z][a-z]+ (Langdon|Neveu)$", "(", "punctuation"),
        ("jsdoc", "/**\n * Opens the vault.\n * @param {string} key Robert Langdon's key\n */\n", "param", "keyword"),
        ("svelte", "<script>\n  let name = 'Hermione Granger';\n</script>\n<h1>{name}</h1>\n", "{", "punctuation"),
        ("query", "; inherits: javascript\n(named_node name: (identifier) @variable)\n", "inherits", "keyword"),
        ("query", "; format-ignore\n(named_node name: (identifier) @variable)\n", "format-ignore", "keyword"),
        ("json5", "{ owl: 'Hedwig', count: 3, ok: true }\n", "'Hedwig'", "string"),
        ("json5", "{ owl: 'Hedwig', count: 3, ok: true }\n", "3", "constant"),
    ];
    for (filetype, sample, token, expected) in cases {
        let start = sample.find(token).expect("the sample holds the token");
        let captures = highlights(sample, filetype).unwrap_or_else(|| panic!("{filetype} has no highlights"));
        let names: Vec<&str> = captures.iter().filter(|c| c.start <= start && start < c.end).map(|c| c.name).collect();
        assert!(
            names.iter().any(|name| name.starts_with(expected) && theme.get(name).is_some()),
            "{filetype}: {token} takes {names:?}, not {expected}"
        );
    }
}

#[test]
fn the_extensions_opentui_lacks_are_highlighted() {
    let none = Associations::default();
    assert_eq!(language_for(&none, Path::new(""), Path::new("Gringotts/vault.tf")), Some("hcl"));
    assert_eq!(language_for(&none, Path::new(""), Path::new("Hogwarts/owls.tfvars")), Some("hcl"));
    assert_eq!(language_for(&none, Path::new(""), Path::new("Hogwarts/live.heex")), Some("heex"));
    assert_eq!(language_for(&none, Path::new(""), Path::new("Hogwarts/owls.ex")), Some("elixir"));
    assert_eq!(language_for(&none, Path::new(""), Path::new("CERN/schema.sql")), Some("sql"));
    assert_eq!(language_for(&none, Path::new(""), Path::new("CERN/Makefile.mk")), None);
}

#[test]
fn typescript_names_a_property_as_core_does() {
    register();
    // Core's query names an interface's property `variable.member` (plain text in Scouter); the crate's says `property`.
    let source = "export interface Vault {\n  kind: string;\n}\n";
    let captures = highlights(source, "typescript").expect("typescript highlights");
    let property = source.find("kind").unwrap();
    let name = captures.iter().find(|c| c.start == property).map(|c| c.name);
    assert_eq!(name, Some("variable.member"));
}

#[test]
fn a_path_gets_the_filetype_scouter_names_or_none() {
    assert_eq!(language_for(&Associations::default(), Path::new(""), Path::new("hogwarts/Potter.rs")), Some("rust"));
    assert_eq!(
        language_for(&Associations::default(), Path::new(""), Path::new("Gringotts/vault.TSX")),
        Some("typescriptreact")
    );
    assert_eq!(
        language_for(&Associations::default(), Path::new(""), Path::new("Louvre/key.jsx")),
        Some("javascriptreact")
    );
    assert_eq!(language_for(&Associations::default(), Path::new(""), Path::new("CERN/config.toml")), Some("toml"));
    assert_eq!(language_for(&Associations::default(), Path::new(""), Path::new("CERN/Makefile")), None);
    assert_eq!(language_for(&Associations::default(), Path::new(""), Path::new("Diagon/owls.txt")), None);
}

#[test]
fn an_association_decides_before_the_filetype_lookup() {
    let mut associations = Associations::default();
    associations.add("*.rs.bak", "rust").unwrap();
    associations.add("*.env.example", "dotenv").unwrap();
    associations.add("*.txt", "plaintext").unwrap();
    associations.add("docs/**/*.txt", "markdown").unwrap();
    let root = Path::new("/Hogwarts/Gringotts");
    let at = |path: &str| language_for(&associations, root, &root.join(path));
    assert_eq!(at("vault.rs.bak"), Some("rust"));
    assert_eq!(at("louvre.env.example"), Some("bash"));
    assert_eq!(at("docs/owls.txt"), Some("markdown"));
    assert_eq!(at("owls.txt"), None);
    assert_eq!(at("CERN/config.toml"), Some("toml"));
    assert_eq!(language_for(&associations, root, Path::new("/Hogwarts/Diagon/vault.rs.bak")), Some("rust"));
}

#[test]
fn the_style_table_reads_the_palette() {
    let theme = theme(&DARK);
    let keyword = theme.get("keyword").unwrap();
    assert_eq!(keyword.fg, Some(color(DARK.syntax.keyword)));
    assert!(keyword.attributes.contains(Attributes::BOLD) && !keyword.attributes.contains(Attributes::ITALIC));
    // A capture with no style of its own takes its first part's, as Core does: `keyword.return` is a keyword.
    assert_eq!(theme.get("keyword.return"), theme.get("keyword"));
    assert_eq!(theme.get("markup.raw.block"), None);
    assert_eq!(theme.get("type.builtin"), theme.get("constant"));
}

/// The foreground the reader paints the first occurrence of `token` with: the span over it, after every capture's
/// style is layered in the order `highlight` gives.
fn painted(source: &str, filetype: &str, token: &str) -> Option<Color> {
    register();
    let start = source.find(token).unwrap_or_else(|| panic!("the sample holds {token}"));
    let mut at = 0;
    for span in highlight(source, filetype, &theme(&DARK))? {
        let end = at + span.content.len();
        if at <= start && start < end {
            return span.style.fg;
        }
        at = end;
    }
    None
}

#[test]
fn injected_code_is_painted_in_its_own_languages_colours() {
    let syntax = DARK.syntax;
    // Each token takes the palette colour of the capture its own language gives it, after the host's captures.
    let cases = [
        ("svelte", "<script>\n  let owl = 'Hedwig';\n</script>\n", "let", syntax.keyword),
        (
            "svelte",
            "<script lang=\"ts\">\n  let name: string = 'Hermione Granger';\n</script>\n",
            "string",
            syntax.constant,
        ),
        ("svelte", "<style>\n  h1 { color: red; }\n</style>\n", "color", syntax.function),
        ("astro", "---\nconst name = 'Albus Dumbledore';\n---\n<h1>{name}</h1>\n", "const", syntax.keyword),
        ("astro", "<script>\n  let owl = 'Hedwig';\n</script>\n", "let", syntax.keyword),
        ("html", "<script>\n  const owl = 'Hedwig';\n</script>\n", "const", syntax.keyword),
        ("html", "<style>\n  p { color: red; }\n</style>\n", "color", syntax.function),
        (
            "elixir",
            "def render(assigns) do\n  ~H\"\"\"\n  <div class=\"vault\"></div>\n  \"\"\"\nend\n",
            "div",
            syntax.keyword,
        ),
        ("heex", "<div class=\"vault\"><%= if Hogwarts.open?() do %>vault<% end %></div>\n", "if", syntax.keyword),
        ("elixir", "def query, do: ~SQL\"\"\"\nSELECT name FROM vaults;\n\"\"\"\n", "SELECT", syntax.keyword),
        ("elixir", "def pattern, do: ~r/^[A-Z]+$/\n", "[", syntax.punctuation),
        ("elixir", "def pattern, do: ~r/^[A-Z]+$/\n", "-", syntax.keyword),
    ];
    for (filetype, sample, token, expected) in cases {
        let got = painted(sample, filetype, token);
        assert_eq!(got, Some(color(expected)), "{filetype}: {token} is painted {got:?}");
    }
}

#[test]
fn injections_leave_what_no_grammar_is_registered_for_plain() {
    // scss is not registered, and neither is a script in an unknown language: the host's `none` clears the body.
    let style = "<style lang=\"scss\">\n  $owl: red;\n</style>\n";
    let script = "<script lang=\"js\">\n  let owl = 'Hedwig';\n</script>\n";
    for (sample, token) in [(style, "$owl"), (script, "let")] {
        assert_eq!(painted(sample, "svelte", token), Some(color(DARK.text)), "svelte: {token}");
    }
}

#[test]
fn jsdoc_and_regex_are_painted_in_their_own_languages_colours() {
    let syntax = DARK.syntax;
    let jsdoc = "/**\n * Opens the Louvre vault.\n * @param {string} key Robert Langdon's key\n */\n";
    // Each token takes the palette colour of the capture its own language gives it; a plain string or a line comment
    // takes the colour of its host capture, because it is not JSDoc or a pattern.
    let cases = [
        ("javascript", jsdoc, "param", syntax.keyword),
        ("typescript", jsdoc, "param", syntax.keyword),
        ("typescriptreact", jsdoc, "param", syntax.keyword),
        ("javascript", "// Sophie checks the key @param x\n", "param", syntax.comment),
        ("javascript", "function open(key) {\n  return /^[A-Z]+\\d{2}$/.test(key);\n}\n", "\\d", syntax.constant),
        ("javascript", "const pattern = \"[A-Z]+\";\n", "[", syntax.string),
        ("python", "import re\nPATTERN = re.compile(r\"^[A-Z][a-z]+ Neveu$\")\n", "[", syntax.punctuation),
        ("python", "import re\nPATTERN = re.compile(r\"^[A-Z][a-z]+ Neveu$\")\n", "A-Z", syntax.constant),
        ("python", "print(\"[A-Z]\")\n", "[", syntax.string),
        ("markdown", "```js\n/** @param {string} key */\n```\n", "param", syntax.keyword),
    ];
    for (filetype, sample, token, expected) in cases {
        let got = painted(sample, filetype, token);
        assert_eq!(got, Some(color(expected)), "{filetype}: {token} is painted {got:?}");
    }
}

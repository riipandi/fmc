use super::*;

// Sample diagrams with Harry Potter and Dan Brown names (rules.md, "Sample data").
const FLOWCHART: &str = "flowchart LR\n    subgraph Hogwarts\n        A[Harry Potter] --> B{Sorting Hat}\n        B -->|Gryffindor| C[Gryffindor Tower]\n        B -->|Slytherin| D[Slytherin Dungeon]\n    end\n    C --> E[Albus Dumbledore]\n    D --> F[Severus Snape]\n";
const SEQUENCE: &str = "sequenceDiagram\n    Robert->>Sophie: Where is the cryptex?\n    Sophie-->>Robert: Under the Louvre\n    Robert->>Leigh: Who holds the Grail?\n    Leigh-->>Robert: The Priory of Sion\n";
const CLASS: &str = "classDiagram\n    class Cryptex {\n        +String passcode\n        +open() bool\n    }\n    class Keystone {\n        +String location\n    }\n    Cryptex --> Keystone : holds\n";
const GANTT: &str = "gantt\n    title Hogwarts term plan\n    dateFormat YYYY-MM-DD\n    section Lessons\n    Potions          :a1, 2026-09-01, 30d\n    Charms           :a2, after a1, 20d\n";

fn rows(source: &str, width: Option<usize>) -> Vec<String> {
    render(source, width).unwrap_or_else(|fallback| panic!("expected a diagram, got: {}", fallback.reason))
}

fn reason(source: &str, width: Option<usize>) -> String {
    render(source, width).expect_err("expected a fallback").reason
}

#[test]
fn a_flowchart_with_a_subgraph_draws_its_boxes_at_100_columns() {
    let rows = rows(FLOWCHART, Some(100));
    assert!(columns(&rows) <= 100);
    assert!(rows.iter().any(|row| row.contains("Harry Potter")));
    assert!(rows.iter().any(|row| row.contains('┌') && row.contains('┐')));
}

#[test]
fn a_sequence_diagram_draws_its_participants_at_100_columns() {
    let rows = rows(SEQUENCE, Some(100));
    assert!(columns(&rows) <= 100);
    assert!(rows.iter().any(|row| row.contains("Robert")));
    assert!(rows.iter().any(|row| row.contains("Louvre") || row.contains("Grail")));
}

#[test]
fn a_class_diagram_fits_60_columns() {
    let rows = rows(CLASS, Some(60));
    assert!(columns(&rows) <= 60);
    assert!(rows.iter().any(|row| row.contains("Cryptex")));
    assert!(rows.iter().any(|row| row.contains("Keystone")));
}

#[test]
fn a_gantt_chart_draws_its_bars_at_60_columns() {
    let rows = rows(GANTT, Some(60));
    assert!(columns(&rows) <= 60);
    assert!(rows.iter().any(|row| row.contains("Potions")));
    assert!(rows.iter().any(|row| row.contains('█')));
}

#[test]
fn a_diagram_wider_than_the_budget_is_a_fallback_with_its_width() {
    assert_eq!(reason(FLOWCHART, Some(60)), "too wide: 70 columns, the reader has 60");
}

#[test]
fn an_unsupported_diagram_names_its_keyword() {
    assert!(reason("this is not a diagram\n", Some(60)).starts_with("unsupported diagram: "));
}

#[test]
fn a_bad_value_is_a_parse_error() {
    assert!(reason("pie title Hogwarts houses\n    \"Gryffindor\" : abc\n", None).starts_with("parse error: "));
}

#[test]
fn a_blank_source_is_an_empty_diagram() {
    assert_eq!(reason("  \n\n", Some(60)), "empty diagram");
}

#[test]
fn a_diagram_is_the_same_at_the_width_it_measures() {
    // The widget measures at a budget, then paints the rows it measured; this holds for the samples the widget draws.
    for source in [FLOWCHART, SEQUENCE, CLASS] {
        let drawn = rows(source, Some(100));
        assert_eq!(rows(source, Some(columns(&drawn))), drawn);
    }
}

#[test]
fn a_frame_wraps_its_source_inside_the_width() {
    let source = "flowchart LR\n    A[Harry Potter] --> B[Albus Dumbledore]";
    let frame = framed(source, "too wide", Some(20));
    assert!(columns(&frame) <= 20, "{frame:?}");
    assert_eq!(frame.first().map(String::as_str), Some(format!("┌{}┐", "─".repeat(18)).as_str()));
    assert_eq!(frame.last().map(String::as_str), Some("too wide"));
    assert!(frame.iter().any(|row| row.contains("Dumbledore")));
}

#[test]
fn a_frame_without_a_width_keeps_each_line_whole() {
    let frame = framed("flowchart LR\n    A[Harry Potter]", "too wide", None);
    assert!(frame.iter().any(|row| row.contains("flowchart LR")));
    assert_eq!(columns(&frame), "flowchart LR".len().max("    A[Harry Potter]".len()) + 4);
}

#[test]
fn a_frame_has_no_room_below_five_columns() {
    assert!(framed("x", "too wide", Some(4)).is_empty());
    assert!(framed("x", "too wide", Some(0)).is_empty());
}

#[test]
fn a_reason_longer_than_the_width_ends_in_an_ellipsis() {
    assert_eq!(fit("Expelliarmus", 6), "Expel…");
    assert_eq!(fit("Lumos", 6), "Lumos");
    assert_eq!(fit("Lumos", 0), "");
}

use super::*;

fn text(cells: &[(char, Ink)]) -> String {
    cells.iter().map(|(c, _)| c).collect()
}

#[test]
fn a_row_fits_the_widest_width_that_the_pane_holds() {
    assert_eq!(row_width(16, 8), 76);
    assert_eq!(row_width(8, 8), 43);
    assert_eq!(row_width(32, 8), 142);
    assert_eq!(per_row(30, 8), 4);
    assert_eq!(per_row(50, 8), 8);
    assert_eq!(per_row(80, 8), 16);
    assert_eq!(per_row(120, 8), 16);
    assert_eq!(per_row(200, 8), 32);
    // Narrower than the narrowest row: four bytes, cut at the edge.
    assert_eq!(per_row(10, 8), 4);
}

#[test]
fn the_offset_grows_past_four_gibibytes() {
    assert_eq!(offset_digits(0), 8);
    assert_eq!(offset_digits(1 << 32), 8);
    assert_eq!(offset_digits((1 << 32) + 1), 9);
    assert_eq!(offset_digits(u64::MAX), 16);
    let cells = row(0x1_0000_0010, b"A", 4, 9, None);
    assert!(text(&cells).starts_with("100000010  41"));
}

#[test]
fn a_row_lays_out_offset_hex_and_ascii() {
    let bytes = b"Hogwarts\0\x1b\xe9 Sophie";
    let cells = row(0x20, bytes, 16, 8, None);
    assert_eq!(text(&cells), "00000020  48 6f 67 77 61 72 74 73  00 1b e9 20 53 6f 70 68  Hogwarts... Soph");
    assert_eq!(cells.len(), row_width(16, 8));
}

#[test]
fn the_last_short_row_pads_with_blanks() {
    let cells = row(0x10, b"Lan", 8, 8, None);
    assert_eq!(text(&cells), "00000010  4c 61 6e                 Lan     ");
    assert_eq!(cells.len(), row_width(8, 8));
    assert_eq!(cells.last().unwrap().1, Ink::Blank);
}

#[test]
fn every_byte_class_is_told_apart() {
    assert_eq!(class(0), Class::Null);
    assert_eq!(class(b'V'), Class::Printable);
    assert_eq!(class(b'~'), Class::Printable);
    for byte in [b' ', b'\t', b'\n', b'\r', 0x0b, 0x0c] {
        assert_eq!(class(byte), Class::Whitespace, "{byte:#x}");
    }
    assert_eq!(class(0x1b), Class::Control);
    assert_eq!(class(0x7f), Class::Control);
    assert_eq!(class(0x80), Class::High);
    assert_eq!(class(0xff), Class::High);
    assert_eq!((ascii(b'a'), ascii(b' '), ascii(0), ascii(0xe9)), ('a', ' ', '.', '.'));
}

#[test]
fn a_hit_marks_its_bytes_and_the_gaps_between_them() {
    let hit = 0x02..0x05;
    let cells = row(0, b"Neveu", 8, 8, Some(&hit));
    let hits: Vec<bool> =
        cells.iter().map(|(_, ink)| matches!(ink, Ink::Byte { hit: true, .. } | Ink::Gap { hit: true })).collect();
    // Offset (8) and two spaces, then `4e 65 76 65 75`: bytes 2 to 4 and the two gaps between them.
    let marked: String = cells.iter().zip(&hits).filter(|(_, hit)| **hit).map(|((c, _), _)| c).collect();
    assert_eq!(marked, "76 65 75veu");
}

#[test]
fn an_offset_reads_as_hex_decimal_or_a_share() {
    assert_eq!(parse_offset("0x1f40", 1 << 20), Some(0x1f40));
    assert_eq!(parse_offset(" 8000 ", 1 << 20), Some(8000));
    assert_eq!(parse_offset("50%", 1000), Some(500));
    assert_eq!(parse_offset("100%", 1000), Some(999));
    assert_eq!(parse_offset("0xffffffff", 1000), Some(999));
    assert_eq!(parse_offset("150%", 1000), None);
    assert_eq!(parse_offset("Gringotts", 1000), None);
    assert_eq!(parse_offset("0", 0), Some(0));
}

#[test]
fn the_blocks_a_window_needs_are_asked_once() {
    let mut blocks = Blocks::default();
    assert_eq!(blocks.wanted(0..BLOCK + 1), vec![0, 1]);
    assert!(blocks.wanted(5..5).is_empty());
    blocks.ask(0);
    assert_eq!(blocks.wanted(0..BLOCK + 1), vec![1]);
    blocks.insert(1, vec![7; 10], 1);
    assert!(blocks.wanted(0..BLOCK + 1).is_empty());
    assert!(blocks.bytes(0, 16).is_none(), "block 0 is asked, not back");
}

#[test]
fn a_short_block_is_the_files_end() {
    let mut blocks = Blocks::default();
    blocks.insert(2, b"Teabing".to_vec(), 2);
    assert_eq!(blocks.bytes(2 * BLOCK, 16), Some(&b"Teabing"[..]));
    assert_eq!(blocks.bytes(2 * BLOCK + 4, 16), Some(&b"ing"[..]));
    assert_eq!(blocks.bytes(2 * BLOCK + 32, 16), Some(&b""[..]));
}

#[test]
fn the_farthest_block_goes_first() {
    let mut blocks = Blocks::default();
    for index in 0..MAX_BLOCKS as u64 {
        blocks.insert(index, vec![1], 0);
    }
    blocks.insert(100, vec![1], 100);
    assert!(blocks.bytes(0, 1).is_none(), "block 0 is farthest from 100");
    assert!(blocks.bytes(100 * BLOCK, 1).is_some());
    assert!(blocks.bytes(15 * BLOCK, 1).is_some());
}

#[test]
fn a_string_is_found_ignoring_ascii_case_both_ways() {
    let haystack = b"..Robert LANGDON met langdon..";
    assert_eq!(find(haystack, b"langdon", true), Some(9));
    assert_eq!(find(haystack, b"langdon", false), Some(21));
    assert_eq!(find(haystack, b"vittoria", true), None);
    assert_eq!(find(b"ab", b"abc", true), None);
    assert_eq!(find("Sophie Neveu é".as_bytes(), "é".as_bytes(), true), Some(13));
    assert_eq!(find(b"\0\x7fELF", b"\x7felf", true), Some(1));
}

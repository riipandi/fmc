use std::fs::{self, OpenOptions};

use super::*;

fn file_with(bytes: &[u8]) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Gringotts.vault");
    fs::write(&path, bytes).unwrap();
    (dir, path)
}

#[test]
fn blocks_read_whole_short_at_the_end_and_empty_past_it() {
    let mut bytes = vec![0u8; (BLOCK * 2 + 10) as usize];
    bytes[BLOCK as usize] = b'H';
    let (_dir, path) = file_with(&bytes);
    assert_eq!(read_block(&path, 0).unwrap().len(), BLOCK as usize);
    assert_eq!(read_block(&path, 1).unwrap()[0], b'H');
    assert_eq!(read_block(&path, 2).unwrap().len(), 10);
    assert!(read_block(&path, 3).unwrap().is_empty());
}

#[test]
fn a_file_that_shrank_reads_short() {
    let (_dir, path) = file_with(&vec![1u8; (BLOCK * 3) as usize]);
    OpenOptions::new().write(true).open(&path).unwrap().set_len(BLOCK + 5).unwrap();
    assert_eq!(read_block(&path, 1).unwrap().len(), 5);
    assert!(read_block(&path, 2).unwrap().is_empty());
}

#[test]
fn a_missing_file_is_an_error_in_nodes_words() {
    let error = read_block(Path::new("/nonexistent/Azkaban.bin"), 0).unwrap_err();
    assert!(error.starts_with("ENOENT"), "{error}");
}

#[test]
fn a_search_finds_a_hit_across_a_chunk_edge_and_wraps() {
    let mut bytes = vec![0u8; (CHUNK * 3) as usize];
    let edge = CHUNK as usize - 3;
    bytes[edge..edge + 7].copy_from_slice(b"Langdon");
    let late = 2 * CHUNK as usize + 100;
    bytes[late..late + 7].copy_from_slice(b"LANGDON");
    let (_dir, path) = file_with(&bytes);
    let stop = AtomicBool::new(false);
    assert_eq!(search(&path, b"langdon", 0, true, &stop).unwrap(), Some(edge as u64));
    assert_eq!(search(&path, b"langdon", edge as u64 + 1, true, &stop).unwrap(), Some(late as u64));
    // Past the last hit, forward wraps to the first.
    assert_eq!(search(&path, b"langdon", late as u64 + 1, true, &stop).unwrap(), Some(edge as u64));
    // Back finds the hit before, and wraps from the first to the last.
    assert_eq!(search(&path, b"langdon", late as u64, false, &stop).unwrap(), Some(edge as u64));
    assert_eq!(search(&path, b"langdon", edge as u64, false, &stop).unwrap(), Some(late as u64));
    assert_eq!(search(&path, b"vittoria", 0, true, &stop).unwrap(), None);
}

#[test]
fn a_stopped_search_finds_nothing() {
    let (_dir, path) = file_with(b"Teabing");
    assert_eq!(search(&path, b"teabing", 0, true, &AtomicBool::new(true)).unwrap(), None);
}

#[test]
fn a_hit_at_the_last_byte_is_found() {
    let (_dir, path) = file_with(b"...........Neveu");
    let stop = AtomicBool::new(false);
    assert_eq!(search(&path, b"neveu", 0, true, &stop).unwrap(), Some(11));
    assert_eq!(search(&path, b"u", 16, false, &stop).unwrap(), Some(15));
}

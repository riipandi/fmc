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

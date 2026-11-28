//! The hex view's reads: one block of a file at a time. Each read runs as one worker job (`services::spawn`), so a slow
//! disk never stalls a frame. Every file is opened read-only, and a file that shrank reads short instead of failing:
//! positioned reads, where a memory map would kill the process with SIGBUS.

use std::fs::File;
use std::io;
use std::os::unix::fs::FileExt;
use std::path::Path;

use fmc::reader::hex::BLOCK;

use super::fs::describe;

/// Block `index` of the file: [`BLOCK`] bytes, fewer at the end, none past it.
pub fn read_block(path: &Path, index: u64) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|error| describe(&error, "open", Some(path)))?;
    let mut bytes = vec![0; BLOCK as usize];
    let filled = read_full(&file, &mut bytes, index * BLOCK).map_err(|error| describe(&error, "read", None))?;
    bytes.truncate(filled);
    Ok(bytes)
}

/// Fills `buffer` from `offset` until it is full or the file ends; the bytes read.
fn read_full(file: &File, buffer: &mut [u8], offset: u64) -> io::Result<usize> {
    let mut filled = 0;
    while filled < buffer.len() {
        match file.read_at(&mut buffer[filled..], offset + filled as u64) {
            Ok(0) => break,
            Ok(read) => filled += read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    Ok(filled)
}

#[cfg(test)]
#[path = "window_tests.rs"]
mod tests;

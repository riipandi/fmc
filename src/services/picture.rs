//! The image reads of the rendered view's boxes (`reader/images.rs`): each image's header, its size and format, read
//! on a worker and cached by the file's path and modification time, so a document that is shown again reads nothing.
//! A destination is checked against the browsed root on the disk, after its symbolic links are resolved.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, ErrorKind, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::SystemTime;

use fmc::reader::image::{self, Info};
use fmc::reader::images::{self, Header, Source};
use imagesize::ImageType;

/// The headers read so far, by the canonical path. Shared by the workers, so it lives as long as the app.
pub type Cache = Arc<Mutex<HashMap<PathBuf, Entry>>>;

/// A read for one file at one modification time. A read that failed is kept too, so a broken image is not read again.
#[derive(Clone)]
pub struct Entry {
    modified: Option<SystemTime>,
    read: Result<Info, &'static str>,
}

/// Each image's header, by the destination as written. `folder` is the Markdown file's folder, and every local
/// destination must stay inside `root`, so a path that climbs out of it is refused before it is read.
pub fn headers(cache: &Cache, root: &Path, folder: &Path, targets: &[String]) -> Vec<(String, Header)> {
    targets.iter().map(|target| (target.clone(), header(cache, root, folder, target))).collect()
}

fn header(cache: &Cache, root: &Path, folder: &Path, target: &str) -> Header {
    let path = match images::classify(target) {
        Source::Remote => return Header::Refused(images::REMOTE),
        Source::Svg => return Header::Refused(images::SVG),
        Source::Other => return Header::Refused(images::NOT_SHOWN),
        Source::Local(_) => target,
    };
    // The lexical check first: a path that climbs out of the root never reaches the disk.
    let Some(lexical) = images::inside(root, folder, path) else { return Header::Refused(images::OUTSIDE) };
    let Ok(real) = fs::canonicalize(&lexical) else {
        return Header::Refused(images::NOT_FOUND);
    };
    // A symbolic link may lead out of the root, which only the resolved path shows.
    let Ok(real_root) = fs::canonicalize(root) else { return Header::Refused(images::UNREADABLE) };
    if !real.starts_with(&real_root) {
        return Header::Refused(images::OUTSIDE);
    }
    match read(cache, &real) {
        Ok(info) => Header::Picture { path: real, info },
        Err(reason) => Header::Refused(reason),
    }
}

/// The header of the file at `path`, from the cache when its modification time has not changed.
fn read(cache: &Cache, path: &Path) -> Result<Info, &'static str> {
    let metadata = fs::metadata(path).map_err(|error| match error.kind() {
        ErrorKind::NotFound => images::NOT_FOUND,
        _ => images::UNREADABLE,
    })?;
    if !metadata.is_file() {
        return Err(images::UNREADABLE);
    }
    if metadata.len() > image::MAX_BYTES {
        return Err(images::TOO_LARGE);
    }
    let modified = metadata.modified().ok();
    let cached = cache
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(path)
        .filter(|entry| entry.modified == modified)
        .map(|entry| entry.read.clone());
    if let Some(read) = cached {
        return read;
    }
    let read = size(path);
    cache
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(path.to_path_buf(), Entry { modified, read: read.clone() });
    read
}

/// The size and format from the header alone, which the decoder reads without the picture's pixels.
fn size(path: &Path) -> Result<Info, &'static str> {
    let mut reader = BufReader::new(File::open(path).map_err(|_| images::UNREADABLE)?);
    let format = match imagesize::reader_type(&mut reader) {
        Ok(ImageType::Png) => "png",
        Ok(ImageType::Jpeg) => "jpeg",
        Ok(ImageType::Webp) => "webp",
        Ok(ImageType::Gif) => "gif",
        _ => return Err(images::UNREADABLE),
    };
    reader.seek(SeekFrom::Start(0)).map_err(|_| images::UNREADABLE)?;
    let dimensions = imagesize::reader_size(reader).map_err(|_| images::UNREADABLE)?;
    Ok(Info { width: dimensions.width as u32, height: dimensions.height as u32, format })
}

#[cfg(test)]
#[path = "picture_tests.rs"]
mod tests;

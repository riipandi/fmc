//! Image files (Scouter `src/core/files.ts:7-15`, `readImage`, and `src/reader/image-view.ts`): the extensions that
//! open as images, the size cap, and the messages for a refused image.

use std::path::Path;

/// Scouter reads an image only by these extensions, case-insensitively (`src/core/files.ts:9`).
const EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp"];

/// Larger images are refused before they are read (`src/core/files.ts:7`).
pub const MAX_BYTES: u64 = 10 * 1024 * 1024;

pub const TOO_LARGE: &str = "Image is too large (limit: 10 MiB).";
pub const UNREADABLE: &str = "This image could not be read.";

/// The size and format of an image, from its header. The format names are the ones Scouter's decoder reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Info {
    pub width: u32,
    pub height: u32,
    pub format: &'static str,
}

pub fn is_image_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_five_extensions_open_as_images_in_any_case() {
        assert!(is_image_path(Path::new("assets/marauder-map.PNG")));
        assert!(is_image_path(Path::new("Louvre/mona.jpeg")) && is_image_path(Path::new("Louvre/mona.webp")));
        assert!(!is_image_path(Path::new("assets/seal.bin")) && !is_image_path(Path::new("assets/vector.svg")));
    }
}

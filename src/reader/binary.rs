//! The size a binary notice shows (Scouter `formatBytes`, `src/core/files.ts:50-56`).

/// Bytes, then KB or MB with one decimal. The decimal rounds half up, as JavaScript's `toFixed` does, so
/// `1280 B` reads `1.3 KB`.
pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let (unit, divisor) = if bytes < 1024 * 1024 { ("KB", 1024.0) } else { ("MB", 1024.0 * 1024.0) };
    let tenths = (bytes as f64 / divisor * 10.0 + 0.5).floor() as u64;
    format!("{}.{} {unit}", tenths / 10, tenths % 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_read_like_scouter() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1280), "1.3 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1536 * 1024), "1.5 MB");
    }
}

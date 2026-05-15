//! Shared utility helpers.

/// Filter helper for `WalkDir` iterators: logs a warning to stderr and drops
/// errored entries, passing successful entries through unchanged.
///
/// Use as `.filter_map(warn_on_walkdir_err)` on a `WalkDir` iterator so a
/// broken symlink or permission error on one entry does not abort the scan.
pub fn warn_on_walkdir_err(
    result: Result<walkdir::DirEntry, walkdir::Error>,
) -> Option<walkdir::DirEntry> {
    match result {
        Ok(entry) => Some(entry),
        Err(ref err) => {
            let path_str = err
                .path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<unknown path>".to_string());
            let reason = err
                .io_error()
                .map(|e| e.to_string())
                .unwrap_or_else(|| err.to_string());
            eprintln!("Warning: skipping {}: {}", path_str, reason);
            None
        }
    }
}

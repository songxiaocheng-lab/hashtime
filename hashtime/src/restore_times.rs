use crate::FileTimeResult;
use crate::file_time::set_times;
use std::path::PathBuf;

/// Restore file timestamps from a list of results.
///
/// This function takes a list of (path, timestamp) tuples and applies
/// the timestamps to the corresponding files.
///
/// # Arguments
///
/// * `input_results` - A vector of tuples containing:
///   - `PathBuf`: The file path to modify
///   - `FileTimeResult`: The timestamp result containing creation and modification times
///
/// # Example
///
/// ```rust
/// use hashtime::{FileTimeResult, restore_times};
/// use std::path::PathBuf;
///
/// let results = vec![
///     (PathBuf::from("/path/to/file.txt"), FileTimeResult {
///         created_ns: Some(1_700_000_000_000_000_000),
///         modified_ns: Some(1_700_000_000_000_000_000),
///     }),
/// ];
/// restore_times(results);
/// ```
pub fn restore_times(input_results: Vec<(PathBuf, FileTimeResult)>) {
    for (p, t) in input_results {
        set_times(p, t)
    }
}

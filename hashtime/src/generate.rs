//! Generation of file hashes and timestamps with parallel processing support.

use crate::file_hash::{self, HashField};
use crate::file_time::{self, TimeField};
use crate::{FileHashResult, FileTimeResult, utils};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

/// Result containing file path, hash values, and timestamps.
///
/// This struct is the primary output of the [`generate`] function.
/// It contains both the file path and optionally computed hash values
/// and timestamps depending on the requested fields.
///
/// # Fields
///
/// * `path` - The absolute path to the file
/// * `size` - File size in bytes
/// * `created_ns` - Creation/birth time in nanoseconds since epoch (Unix timestamp)
/// * `modified_ns` - Last modification time in nanoseconds since epoch (Unix timestamp)
/// * `md5` - MD5 hash (if requested)
/// * `sha1` - SHA1 hash (if requested)
/// * `sha256` - SHA256 hash (if requested)
/// * `sha512` - SHA512 hash (if requested)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHashTimeResult {
    pub path: PathBuf,
    pub size: Option<u64>,
    pub created_ns: Option<i64>,
    pub modified_ns: Option<i64>,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub sha512: Option<String>,
}

impl FileHashTimeResult {
    /// Creates a new FileHashTimeResult from separate hash and time results.
    #[inline]
    fn from_parts(path: &Path, h: Option<FileHashResult>, t: Option<FileTimeResult>) -> Self {
        let (size, md5, sha1, sha256, sha512) = h.map_or((None, None, None, None, None), |h| {
            (Some(h.size), h.md5, h.sha1, h.sha256, h.sha512)
        });
        let (created_ns, modified_ns) = t.map_or((None, None), |t| (t.created_ns, t.modified_ns));
        Self {
            path: path.to_path_buf(),
            size,
            created_ns,
            modified_ns,
            md5,
            sha1,
            sha256,
            sha512,
        }
    }
}

/// Process a single file and extract hashes and timestamps.
///
/// Returns None if the file doesn't exist or is not a file when no time fields requested.
#[inline]
fn get_hash_time(
    path: &Path,
    hash_field_set: &HashSet<HashField>,
    time_field_set: &HashSet<TimeField>,
) -> Option<FileHashTimeResult> {
    let path_is_file = path.is_file();
    if (hash_field_set.is_empty() || !path_is_file) && time_field_set.is_empty() {
        return None;
    }
    let fs_meta = std::fs::metadata(path).ok()?;
    let times = if !time_field_set.is_empty() {
        Some(file_time::get_times(&fs_meta, time_field_set))
    } else {
        None
    };
    let hashes = if !hash_field_set.is_empty() && path_is_file {
        Some(file_hash::get_hashes(path, &fs_meta, hash_field_set))
    } else {
        None
    };
    Some(FileHashTimeResult::from_parts(path, hashes, times))
}

/// Generate hashes and timestamps for files in parallel.
///
/// This is the main entry point for generating file metadata.
/// It processes all provided paths (files and directories) and computes
/// the requested hash algorithms and timestamp fields in parallel using rayon.
///
/// # Arguments
///
/// * `input_paths` - List of file or directory paths to process
/// * `hash_fields` - List of hash algorithms: "md5", "sha1", "sha256", "sha512"
/// * `time_fields` - List of time fields: "mtime", "birthtime"
///
/// # Returns
///
/// A vector of [`FileHashTimeResult`] containing the results for each file.
/// Directories are not included in the results (only files).
///
/// # Example
///
/// ```rust
/// use hashtime::generate;
/// use std::path::PathBuf;
///
/// let paths = [PathBuf::from("/path/to/file.txt")];
/// let results = generate(&paths, &["md5".to_string()], &["mtime".to_string()]);
///
/// for result in results {
///     println!("Path: {:?}", result.path);
///     println!("  MD5: {:?}", result.md5);
///     println!("  Modified: {:?}", result.modified_ns);
/// }
/// ```
pub fn generate(
    input_paths: &[PathBuf],
    hash_fields: &[String],
    time_fields: &[String],
) -> Vec<FileHashTimeResult> {
    let all_paths = utils::path_util::expand_path(input_paths, true);
    let hash_field_set: HashSet<HashField> = hash_fields
        .iter()
        .filter_map(|s| HashField::from_str(s).ok())
        .collect();
    let time_field_set: HashSet<TimeField> = time_fields
        .iter()
        .filter_map(|s| TimeField::from_str(s).ok())
        .collect();
    all_paths
        .into_par_iter()
        .filter_map(|p| get_hash_time(&p, &hash_field_set, &time_field_set))
        .collect()
}

/// Generate hashes and timestamps with a callback for progress reporting.
///
/// Similar to [`generate`] but calls the provided callback for each processed file.
/// This is useful for progress bars or logging during long-running operations.
///
/// # Arguments
///
/// * `input_paths` - List of file or directory paths to process
/// * `hash_fields` - List of hash algorithms: "md5", "sha1", "sha256", "sha512"
/// * `time_fields` - List of time fields: "mtime", "birthtime"
/// * `callback` - A function that receives each [`FileHashTimeResult`] as it's processed
///
/// # Example
///
/// ```rust
/// use hashtime::generate_with_callback;
/// use std::path::PathBuf;
///
/// let paths = [PathBuf::from("/path/to/files")];
/// generate_with_callback(&paths, &["md5".to_string()], &[], |result| {
///     println!("Processed: {:?}", result.path);
/// });
/// ```
pub fn generate_with_callback(
    input_paths: &[PathBuf],
    hash_fields: &[String],
    time_fields: &[String],
    callback: impl Fn(FileHashTimeResult) + Send + Sync,
) {
    let all_paths = utils::path_util::expand_path(input_paths, true);
    let hash_field_set: HashSet<HashField> = hash_fields
        .iter()
        .filter_map(|s| HashField::from_str(s).ok())
        .collect();
    let time_field_set: HashSet<TimeField> = time_fields
        .iter()
        .filter_map(|s| TimeField::from_str(s).ok())
        .collect();

    let callback = Arc::new(callback);

    all_paths
        .into_par_iter()
        .filter_map(|p| get_hash_time(&p, &hash_field_set, &time_field_set))
        .for_each(|r| {
            callback(r);
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::slice;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::tempdir;

    #[test]
    fn test_generate_single_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();

        let results = generate(slice::from_ref(&file_path), &["md5".to_string()], &[]);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, file_path);
        assert!(results[0].md5.is_some());
    }

    #[test]
    fn test_generate_directory() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");
        File::create(&file1)
            .unwrap()
            .write_all(b"content1")
            .unwrap();
        File::create(&file2)
            .unwrap()
            .write_all(b"content2")
            .unwrap();

        let results = generate(&[dir.path().to_path_buf()], &["md5".to_string()], &[]);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_generate_with_time_fields() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();

        let results = generate(
            slice::from_ref(&file_path),
            &["md5".to_string()],
            &["mtime".to_string()],
        );

        assert_eq!(results.len(), 1);
        assert!(results[0].md5.is_some());
        assert!(results[0].modified_ns.is_some());
    }

    #[test]
    fn test_generate_with_callback() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        generate_with_callback(
            slice::from_ref(&file_path),
            &["md5".to_string()],
            &[],
            move |_result| {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            },
        );

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_file_hash_time_result_serialize() {
        let result = FileHashTimeResult {
            path: PathBuf::from("/test/file.txt"),
            size: Some(100),
            created_ns: Some(1_700_000_000_000_000_000i64 + 123_456_789i64),
            modified_ns: Some(1_700_000_000_000_000_000i64 + 123_456_789i64),
            md5: Some("abc123".to_string()),
            sha1: None,
            sha256: Some("def456".to_string()),
            sha512: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("123456789"));
        let deserialized: FileHashTimeResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.created_ns, result.created_ns);
        assert_eq!(deserialized.modified_ns, result.modified_ns);
    }

    #[test]
    fn test_generate_multiple_hash_algorithms() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let results = generate(
            slice::from_ref(&file_path),
            &["md5".to_string(), "sha256".to_string()],
            &[],
        );

        assert_eq!(results.len(), 1);
        assert!(results[0].md5.is_some());
        assert!(results[0].sha256.is_some());
        assert!(results[0].sha1.is_none());
        assert!(results[0].sha512.is_none());
    }

    #[test]
    fn test_generate_empty_input() {
        let results = generate(&[], &["md5".to_string()], &[]);
        assert!(results.is_empty());
    }
}

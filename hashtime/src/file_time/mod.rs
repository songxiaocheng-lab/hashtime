//! Cross-platform file timestamp retrieval and modification.
//!
//! This module provides cross-platform support for getting and setting file timestamps.
//! On macOS and Windows, both birth time (creation time) and modification time are supported.
//! On Linux, only modification time is available (birth time is not stored by default).
//! The module provides best-effort fallbacks for unsupported operations.
//!
//! ## Linux Timestamp Behavior
//!
//! On Linux, birthtime (creation time) is set using debugfs, which requires sync and drop
//! caches operations. Running multiple birthtime modifications in parallel causes
//! interference between these low-level operations. Tests that set birthtime must run
//! serially (`#[cfg_attr(target_os = "linux", serial)]`) to ensure consistent results.

#[allow(clippy::module_inception)]
mod linux_impl;
#[allow(clippy::module_inception)]
mod macos_impl;
#[allow(clippy::module_inception)]
mod windows_impl;

use anyhow::Result;
use filetime::{FileTime, set_file_mtime};
use std::collections::HashSet;
use std::fs::Metadata;
use std::path::PathBuf;
use std::str::FromStr;

#[cfg(target_os = "macos")]
use macos_impl::set_birthtime as set_birthtime_platform;

#[cfg(target_os = "windows")]
use windows_impl::set_birthtime as set_birthtime_platform;

#[cfg(target_os = "linux")]
use linux_impl::set_birthtime as set_birthtime_platform;

use serde::{Deserialize, Serialize};

/// Timestamp field selection for file time operations.
///
/// # Variants
///
/// * `Birthtime` - Creation/birth time (not available on Linux)
/// * `Mtime` - Last modification time
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeField {
    Birthtime,
    Mtime,
}

impl FromStr for TimeField {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "birthtime" => Ok(TimeField::Birthtime),
            "mtime" => Ok(TimeField::Mtime),
            _ => Err(()),
        }
    }
}

/// Result containing file timestamps.
///
/// # Fields
///
/// * `created_ns` - Creation/birth time in nanoseconds since Unix epoch
/// * `modified_ns` - Last modification time in nanoseconds since Unix epoch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTimeResult {
    pub created_ns: Option<i64>,
    pub modified_ns: Option<i64>,
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn set_birthtime_platform(_path: &Path, _birth_ns: i64) -> Result<()> {
    use anyhow::bail;
    bail!("set_birthtime not implemented for this platform")
}

pub(crate) fn set_times(path: PathBuf, result: FileTimeResult) {
    if let Some(mtime_ns) = result.modified_ns {
        let m =
            FileTime::from_unix_time(mtime_ns / 1_000_000_000, (mtime_ns % 1_000_000_000) as u32);
        set_file_mtime(&path, m).unwrap_or_else(|err| eprintln!("{}", err))
    }
    if let Some(birth_ns) = result.created_ns {
        set_birthtime_platform(&path, birth_ns).unwrap_or_else(|err| eprintln!("{}", err))
    }
}

pub(crate) fn get_times(meta: &Metadata, field_set: &HashSet<TimeField>) -> FileTimeResult {
    let created_ns = if field_set.contains(&TimeField::Birthtime) {
        FileTime::from_creation_time(meta)
            .map(|b| b.unix_seconds() * 1_000_000_000 + (b.nanoseconds() as i64))
    } else {
        None
    };
    let modified_ns = if field_set.contains(&TimeField::Mtime) {
        let m = FileTime::from_last_modification_time(meta);
        Some(m.unix_seconds() * 1_000_000_000 + (m.nanoseconds() as i64))
    } else {
        None
    };
    FileTimeResult {
        created_ns,
        modified_ns,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "linux")]
    use serial_test::serial;
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn assert_timestamp_eq(actual: FileTime, expected_ns: i64, msg: &str) {
        let actual_sec = actual.unix_seconds();
        let actual_ns = actual.nanoseconds();
        let expected_sec = expected_ns / 1_000_000_000;
        let expected_ns_mod = (expected_ns % 1_000_000_000) as u32;

        assert_eq!(
            actual_sec, expected_sec,
            "{}: seconds should match: expected {}, got {}",
            msg, expected_sec, actual_sec
        );

        #[cfg(target_os = "windows")]
        {
            let diff = (expected_ns_mod as i32 - actual_ns as i32).abs();
            assert!(
                diff <= 100,
                "{}: Windows 100ns precision: expected ~{}, got {}",
                msg,
                expected_ns_mod,
                actual_ns
            );
        }

        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(
                expected_ns_mod, actual_ns,
                "{}: nanoseconds should match: expected {}, got {}",
                msg, expected_ns_mod, actual_ns
            );
        }
    }

    #[test]
    fn test_time_field_from_str() {
        assert_eq!(
            TimeField::from_str("birthtime").ok(),
            Some(TimeField::Birthtime)
        );
        assert_eq!(
            TimeField::from_str("BIRTHTIME").ok(),
            Some(TimeField::Birthtime)
        );
        assert_eq!(TimeField::from_str("mtime").ok(), Some(TimeField::Mtime));
        assert_eq!(TimeField::from_str("MTIME").ok(), Some(TimeField::Mtime));
        assert_eq!(TimeField::from_str("invalid").ok(), None);
        assert_eq!(TimeField::from_str("").ok(), None);
    }

    #[test]
    fn test_get_times_birthtime() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();

        let meta = std::fs::metadata(&file_path).unwrap();
        let mut field_set = HashSet::new();
        field_set.insert(TimeField::Birthtime);

        let result = get_times(&meta, &field_set);

        assert!(result.created_ns.is_some());
        assert!(result.modified_ns.is_none());
    }

    #[test]
    fn test_get_times_mtime() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();

        let meta = std::fs::metadata(&file_path).unwrap();
        let mut field_set = HashSet::new();
        field_set.insert(TimeField::Mtime);

        let result = get_times(&meta, &field_set);

        assert!(result.created_ns.is_none());
        assert!(result.modified_ns.is_some());
    }

    #[test]
    fn test_get_times_both() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();

        let meta = std::fs::metadata(&file_path).unwrap();
        let mut field_set = HashSet::new();
        field_set.insert(TimeField::Birthtime);
        field_set.insert(TimeField::Mtime);

        let result = get_times(&meta, &field_set);

        assert!(result.created_ns.is_some());
        assert!(result.modified_ns.is_some());
    }

    #[test]
    fn test_get_times_empty() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();

        let meta = std::fs::metadata(&file_path).unwrap();
        let field_set: HashSet<TimeField> = HashSet::new();

        let result = get_times(&meta, &field_set);

        assert!(result.created_ns.is_none());
        assert!(result.modified_ns.is_none());
    }

    #[test]
    fn test_set_times_mtime() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let original_meta = std::fs::metadata(&file_path).unwrap();
        let original_mtime = FileTime::from_last_modification_time(&original_meta);

        let new_mtime_ns = 1_700_000_000_000_000_000i64 + 123_456_789i64;
        let result = FileTimeResult {
            created_ns: None,
            modified_ns: Some(new_mtime_ns),
        };
        set_times(file_path.clone(), result);

        let new_meta = std::fs::metadata(&file_path).unwrap();
        let new_mtime = FileTime::from_last_modification_time(&new_meta);

        assert_ne!(new_mtime.unix_seconds(), original_mtime.unix_seconds());
        assert_timestamp_eq(new_mtime, new_mtime_ns, "mtime");
    }

    #[test]
    #[cfg_attr(
        target_os = "linux",
        serial,
        ignore = "requires root and debugfs on linux"
    )]
    fn test_set_times_birthtime() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let original_meta = std::fs::metadata(&file_path).unwrap();
        let original_birthtime = FileTime::from_creation_time(&original_meta).unwrap();

        // Use a timestamp with non-zero nanoseconds to verify precision
        // 1600000000 seconds + 567890123 nanoseconds
        let new_birthtime_ns = 1_600_000_000_000_000_000i64 + 567_890_123i64;
        let result = FileTimeResult {
            created_ns: Some(new_birthtime_ns),
            modified_ns: None,
        };
        set_times(file_path.clone(), result);

        let new_meta = std::fs::metadata(&file_path).unwrap();
        let new_birthtime = FileTime::from_creation_time(&new_meta).unwrap();

        assert_ne!(
            new_birthtime.unix_seconds(),
            original_birthtime.unix_seconds()
        );
        assert_timestamp_eq(new_birthtime, new_birthtime_ns, "birthtime");
    }

    #[test]
    fn test_set_times_none() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let original_meta = std::fs::metadata(&file_path).unwrap();
        let original_mtime = FileTime::from_last_modification_time(&original_meta);

        // Both None - should not change anything
        let result = FileTimeResult {
            created_ns: None,
            modified_ns: None,
        };
        set_times(file_path.clone(), result);

        let new_meta = std::fs::metadata(&file_path).unwrap();
        let new_mtime = FileTime::from_last_modification_time(&new_meta);

        assert_eq!(new_mtime.unix_seconds(), original_mtime.unix_seconds());
        assert_eq!(new_mtime.nanoseconds(), original_mtime.nanoseconds());
    }

    #[test]
    #[cfg_attr(
        target_os = "linux",
        serial,
        ignore = "requires root and debugfs on linux"
    )]
    fn test_set_times_both_birthtime_before_mtime() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let birthtime_ns = 1_000_000_000_000_000_000i64 + 100_000_000i64;
        let mtime_ns = 1_200_000_000_000_000_000i64 + 200_000_000i64;
        let result = FileTimeResult {
            created_ns: Some(birthtime_ns),
            modified_ns: Some(mtime_ns),
        };
        set_times(file_path.clone(), result);

        let new_meta = std::fs::metadata(&file_path).unwrap();
        let new_birthtime = FileTime::from_creation_time(&new_meta).unwrap();
        let new_mtime = FileTime::from_last_modification_time(&new_meta);

        assert_timestamp_eq(new_birthtime, birthtime_ns, "birthtime");
        assert_timestamp_eq(new_mtime, mtime_ns, "mtime");
    }

    #[test]
    #[cfg_attr(
        target_os = "linux",
        serial,
        ignore = "requires root and debugfs on linux"
    )]
    fn test_set_times_both_birthtime_after_mtime() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let birthtime_ns = 1_200_000_000_000_000_000i64 + 200_000_000i64;
        let mtime_ns = 1_000_000_000_000_000_000i64 + 100_000_000i64;
        let result = FileTimeResult {
            created_ns: Some(birthtime_ns),
            modified_ns: Some(mtime_ns),
        };
        set_times(file_path.clone(), result);

        let new_meta = std::fs::metadata(&file_path).unwrap();
        let new_birthtime = FileTime::from_creation_time(&new_meta).unwrap();
        let new_mtime = FileTime::from_last_modification_time(&new_meta);

        assert_timestamp_eq(new_birthtime, birthtime_ns, "birthtime");
        assert_timestamp_eq(new_mtime, mtime_ns, "mtime");
    }

    #[test]
    fn test_file_time_result_serialize() {
        let ts_with_ns = 1_700_000_000_000_000_000i64 + 123_456_789i64;
        let result = FileTimeResult {
            created_ns: Some(ts_with_ns),
            modified_ns: Some(ts_with_ns),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("created_ns"));
        assert!(json.contains("123456789"));

        let deserialized: FileTimeResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.created_ns, result.created_ns);
        assert_eq!(deserialized.modified_ns, result.modified_ns);
    }
}

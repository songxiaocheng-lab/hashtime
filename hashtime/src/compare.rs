//! Comparison of file states to detect changes.
//!
//! This module provides functionality to compare two sets of file metadata
//! (hashes and timestamps) and identify differences between them.

use crate::FileHashTimeResult;
use std::collections::HashSet;
use std::str::FromStr;

/// Field that can be compared in file metadata.
///
/// Used to specify which fields should be compared or ignored during
/// file state comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompareField {
    Md5,
    Sha1,
    Sha256,
    Sha512,
    Birthtime,
    Mtime,
    Size,
}

impl FromStr for CompareField {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "md5" => Ok(CompareField::Md5),
            "sha1" => Ok(CompareField::Sha1),
            "sha256" => Ok(CompareField::Sha256),
            "sha512" => Ok(CompareField::Sha512),
            "birthtime" => Ok(CompareField::Birthtime),
            "mtime" => Ok(CompareField::Mtime),
            "size" => Ok(CompareField::Size),
            _ => Err(()),
        }
    }
}

/// Type of difference detected between two file states.
///
/// # Variants
///
/// * `Modified` - File exists in both sets but has changed
/// * `Added` - File exists only in the target set (new file)
/// * `Removed` - File exists only in the base set (deleted file)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffType {
    Modified,
    Added,
    Removed,
}

/// Represents a difference between two file states.
///
/// # Fields
///
/// * `path` - The file path that has changed
/// * `diff_type` - The type of change ([`DiffType`])
/// * `field_diffs` - List of field-level differences
#[derive(Debug, Clone)]
pub struct Diff {
    pub path: String,
    pub diff_type: DiffType,
    pub field_diffs: Vec<FieldDiff>,
}

/// Represents a single field difference within a file diff.
///
/// # Fields
///
/// * `field` - The field that changed ([`CompareField`])
/// * `base` - The original value
/// * `target` - The new value
#[derive(Debug, Clone)]
pub struct FieldDiff {
    pub field: CompareField,
    pub base: String,
    pub target: String,
}

fn compare_option_field<T: PartialEq + Clone + ToString>(
    base: &Option<T>,
    target: &Option<T>,
    ignored: &HashSet<CompareField>,
    field: CompareField,
    field_diffs: &mut Vec<FieldDiff>,
) {
    if !ignored.contains(&field)
        && let Some(base_val) = base
    {
        if target.is_none() {
            field_diffs.push(FieldDiff {
                field,
                base: base_val.to_string(),
                target: String::new(),
            });
        } else if let Some(target_val) = target
            && base_val != target_val
        {
            field_diffs.push(FieldDiff {
                field,
                base: base_val.to_string(),
                target: target_val.to_string(),
            });
        }
    }
}

/// Compare two sets of file metadata and find differences.
///
/// This function compares a base set of file results with a target set
/// and identifies files that have been modified, added, or removed.
///
/// # Arguments
///
/// * `base_entries` - The baseline file metadata (from an earlier scan)
/// * `target_entries` - The target file metadata (from a later scan)
/// * `ignored_fields` - Fields to ignore during comparison
///
/// # Returns
///
/// A vector of [`Diff`] containing all detected differences.
///
/// # Example
///
/// ```rust
/// use hashtime::{compare, CompareField, DiffType, FileHashTimeResult};
/// use std::collections::HashSet;
/// use std::path::PathBuf;
///
/// let base = vec![FileHashTimeResult {
///     path: PathBuf::from("/test/file.txt"),
///     size: Some(100),
///     created_ns: Some(1000),
///     modified_ns: Some(1000),
///     md5: Some("abc123".to_string()),
///     sha1: None,
///     sha256: None,
///     sha512: None,
/// }];
///
/// let target = vec![FileHashTimeResult {
///     path: PathBuf::from("/test/file.txt"),
///     size: Some(200),
///     created_ns: Some(1000),
///     modified_ns: Some(2000),
///     md5: Some("xyz789".to_string()),
///     sha1: None,
///     sha256: None,
///     sha512: None,
/// }];
///
/// let ignored: HashSet<CompareField> = HashSet::new();
/// let diffs = compare(&base, &target, &ignored);
///
/// assert_eq!(diffs.len(), 1);
/// assert_eq!(diffs[0].diff_type, DiffType::Modified);
/// ```
pub fn compare(
    base_entries: &[FileHashTimeResult],
    target_entries: &[FileHashTimeResult],
    ignored_fields: &HashSet<CompareField>,
) -> Vec<Diff> {
    let base_meta_map: std::collections::HashMap<String, &FileHashTimeResult> = base_entries
        .iter()
        .map(|e| (e.path.to_string_lossy().to_string(), e))
        .collect();

    let target_meta_map: std::collections::HashMap<String, &FileHashTimeResult> = target_entries
        .iter()
        .map(|e| (e.path.to_string_lossy().to_string(), e))
        .collect();

    let mut diffs = Vec::new();

    for (path_str, base_meta) in &base_meta_map {
        let target_meta = target_meta_map.get(path_str);

        let mut field_diffs = Vec::new();

        compare_option_field(
            &base_meta.md5,
            &target_meta.and_then(|r| r.md5.clone()),
            ignored_fields,
            CompareField::Md5,
            &mut field_diffs,
        );
        compare_option_field(
            &base_meta.sha1,
            &target_meta.and_then(|r| r.sha1.clone()),
            ignored_fields,
            CompareField::Sha1,
            &mut field_diffs,
        );
        compare_option_field(
            &base_meta.sha256,
            &target_meta.and_then(|r| r.sha256.clone()),
            ignored_fields,
            CompareField::Sha256,
            &mut field_diffs,
        );
        compare_option_field(
            &base_meta.sha512,
            &target_meta.and_then(|r| r.sha512.clone()),
            ignored_fields,
            CompareField::Sha512,
            &mut field_diffs,
        );
        compare_option_field(
            &base_meta.created_ns,
            &target_meta.and_then(|r| r.created_ns),
            ignored_fields,
            CompareField::Birthtime,
            &mut field_diffs,
        );
        compare_option_field(
            &base_meta.modified_ns,
            &target_meta.and_then(|r| r.modified_ns),
            ignored_fields,
            CompareField::Mtime,
            &mut field_diffs,
        );
        compare_option_field(
            &base_meta.size,
            &target_meta.and_then(|r| r.size),
            ignored_fields,
            CompareField::Size,
            &mut field_diffs,
        );

        if !field_diffs.is_empty() {
            diffs.push(Diff {
                path: path_str.clone(),
                diff_type: if target_meta.is_some() {
                    DiffType::Modified
                } else {
                    DiffType::Removed
                },
                field_diffs,
            });
        }
    }

    for (path_str, target_meta) in &target_meta_map {
        if !base_meta_map.contains_key(path_str) {
            let mut field_diffs = Vec::new();

            if let Some(size) = &target_meta.size {
                field_diffs.push(FieldDiff {
                    field: CompareField::Size,
                    base: String::new(),
                    target: size.to_string(),
                });
            }
            if let Some(md5) = &target_meta.md5 {
                field_diffs.push(FieldDiff {
                    field: CompareField::Md5,
                    base: String::new(),
                    target: md5.clone(),
                });
            }
            if let Some(sha1) = &target_meta.sha1 {
                field_diffs.push(FieldDiff {
                    field: CompareField::Sha1,
                    base: String::new(),
                    target: sha1.clone(),
                });
            }
            if let Some(sha256) = &target_meta.sha256 {
                field_diffs.push(FieldDiff {
                    field: CompareField::Sha256,
                    base: String::new(),
                    target: sha256.clone(),
                });
            }
            if let Some(sha512) = &target_meta.sha512 {
                field_diffs.push(FieldDiff {
                    field: CompareField::Sha512,
                    base: String::new(),
                    target: sha512.clone(),
                });
            }
            if let Some(created) = &target_meta.created_ns {
                field_diffs.push(FieldDiff {
                    field: CompareField::Birthtime,
                    base: String::new(),
                    target: created.to_string(),
                });
            }
            if let Some(modified) = &target_meta.modified_ns {
                field_diffs.push(FieldDiff {
                    field: CompareField::Mtime,
                    base: String::new(),
                    target: modified.to_string(),
                });
            }

            diffs.push(Diff {
                path: path_str.clone(),
                diff_type: DiffType::Added,
                field_diffs,
            });
        }
    }

    diffs
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_compare_field_from_str() {
        assert_eq!(CompareField::from_str("md5").ok(), Some(CompareField::Md5));
        assert_eq!(CompareField::from_str("MD5").ok(), Some(CompareField::Md5));
        assert_eq!(
            CompareField::from_str("sha1").ok(),
            Some(CompareField::Sha1)
        );
        assert_eq!(
            CompareField::from_str("sha256").ok(),
            Some(CompareField::Sha256)
        );
        assert_eq!(
            CompareField::from_str("sha512").ok(),
            Some(CompareField::Sha512)
        );
        assert_eq!(
            CompareField::from_str("birthtime").ok(),
            Some(CompareField::Birthtime)
        );
        assert_eq!(
            CompareField::from_str("mtime").ok(),
            Some(CompareField::Mtime)
        );
        assert_eq!(
            CompareField::from_str("size").ok(),
            Some(CompareField::Size)
        );
        assert_eq!(CompareField::from_str("invalid").ok(), None);
    }

    #[test]
    fn test_compare_modified_file() {
        let base = vec![FileHashTimeResult {
            path: PathBuf::from("/test/file.txt"),
            size: Some(100),
            created_ns: Some(1000),
            modified_ns: Some(1000),
            md5: Some("abc123".to_string()),
            sha1: None,
            sha256: None,
            sha512: None,
        }];

        let target = vec![FileHashTimeResult {
            path: PathBuf::from("/test/file.txt"),
            size: Some(200),
            created_ns: Some(1000),
            modified_ns: Some(2000),
            md5: Some("xyz789".to_string()),
            sha1: None,
            sha256: None,
            sha512: None,
        }];

        let ignored: HashSet<CompareField> = HashSet::new();
        let diffs = compare(&base, &target, &ignored);

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DiffType::Modified);
        assert!(
            diffs[0]
                .field_diffs
                .iter()
                .any(|f| f.field == CompareField::Md5)
        );
    }

    #[test]
    fn test_compare_added_file() {
        let base: Vec<FileHashTimeResult> = vec![];

        let target = vec![FileHashTimeResult {
            path: PathBuf::from("/test/new.txt"),
            size: Some(100),
            created_ns: Some(1000),
            modified_ns: Some(1000),
            md5: Some("abc123".to_string()),
            sha1: None,
            sha256: None,
            sha512: None,
        }];

        let ignored: HashSet<CompareField> = HashSet::new();
        let diffs = compare(&base, &target, &ignored);

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DiffType::Added);
    }

    #[test]
    fn test_compare_removed_file() {
        let base = vec![FileHashTimeResult {
            path: PathBuf::from("/test/removed.txt"),
            size: Some(100),
            created_ns: Some(1000),
            modified_ns: Some(1000),
            md5: Some("abc123".to_string()),
            sha1: None,
            sha256: None,
            sha512: None,
        }];

        let target: Vec<FileHashTimeResult> = vec![];

        let ignored: HashSet<CompareField> = HashSet::new();
        let diffs = compare(&base, &target, &ignored);

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DiffType::Removed);
    }

    #[test]
    fn test_compare_no_changes() {
        let base = vec![FileHashTimeResult {
            path: PathBuf::from("/test/file.txt"),
            size: Some(100),
            created_ns: Some(1000),
            modified_ns: Some(1000),
            md5: Some("abc123".to_string()),
            sha1: None,
            sha256: None,
            sha512: None,
        }];

        let target = vec![FileHashTimeResult {
            path: PathBuf::from("/test/file.txt"),
            size: Some(100),
            created_ns: Some(1000),
            modified_ns: Some(1000),
            md5: Some("abc123".to_string()),
            sha1: None,
            sha256: None,
            sha512: None,
        }];

        let ignored: HashSet<CompareField> = HashSet::new();
        let diffs = compare(&base, &target, &ignored);

        assert!(diffs.is_empty());
    }

    #[test]
    fn test_compare_with_ignored_fields() {
        let base = vec![FileHashTimeResult {
            path: PathBuf::from("/test/file.txt"),
            size: Some(100),
            created_ns: Some(1000),
            modified_ns: Some(1000),
            md5: Some("abc123".to_string()),
            sha1: None,
            sha256: None,
            sha512: None,
        }];

        let target = vec![FileHashTimeResult {
            path: PathBuf::from("/test/file.txt"),
            size: Some(200),
            created_ns: Some(1000),
            modified_ns: Some(2000),
            md5: Some("xyz789".to_string()),
            sha1: None,
            sha256: None,
            sha512: None,
        }];

        let mut ignored: HashSet<CompareField> = HashSet::new();
        ignored.insert(CompareField::Md5);
        ignored.insert(CompareField::Size);
        ignored.insert(CompareField::Mtime);
        ignored.insert(CompareField::Sha256);
        let diffs = compare(&base, &target, &ignored);

        assert!(diffs.is_empty());
    }

    #[test]
    fn test_compare_multiple_files() {
        let base = vec![
            FileHashTimeResult {
                path: PathBuf::from("/test/file1.txt"),
                size: Some(100),
                created_ns: Some(1000),
                modified_ns: Some(1000),
                md5: Some("abc123".to_string()),
                sha1: None,
                sha256: None,
                sha512: None,
            },
            FileHashTimeResult {
                path: PathBuf::from("/test/file2.txt"),
                size: Some(100),
                created_ns: Some(1000),
                modified_ns: Some(1000),
                md5: Some("def456".to_string()),
                sha1: None,
                sha256: None,
                sha512: None,
            },
        ];

        let target = vec![
            FileHashTimeResult {
                path: PathBuf::from("/test/file1.txt"),
                size: Some(100),
                created_ns: Some(1000),
                modified_ns: Some(1000),
                md5: Some("abc123".to_string()),
                sha1: None,
                sha256: None,
                sha512: None,
            },
            FileHashTimeResult {
                path: PathBuf::from("/test/file3.txt"),
                size: Some(100),
                created_ns: Some(1000),
                modified_ns: Some(1000),
                md5: Some("ghi789".to_string()),
                sha1: None,
                sha256: None,
                sha512: None,
            },
        ];

        let ignored: HashSet<CompareField> = HashSet::new();
        let diffs = compare(&base, &target, &ignored);

        assert_eq!(diffs.len(), 2);
        let types: Vec<_> = diffs.iter().map(|d| d.diff_type).collect();
        assert!(types.contains(&DiffType::Removed));
        assert!(types.contains(&DiffType::Added));
    }
}

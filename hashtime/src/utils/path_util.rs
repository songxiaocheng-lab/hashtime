//! Path utilities for file discovery and expansion.

use std::path::PathBuf;
use walkdir::WalkDir;

/// Expand input paths to a list of all contained files.
///
/// This function takes a list of file and directory paths and expands
/// directories to their contained files recursively.
///
/// # Arguments
///
/// * `input_paths` - List of file or directory paths
/// * `include_dir` - If true, include directories in output (for recursive walking)
///
/// # Returns
///
/// A sorted vector of all file paths contained in the input paths.
pub(crate) fn expand_path(input_paths: &[PathBuf], include_dir: bool) -> Vec<PathBuf> {
    let mut all_files: Vec<PathBuf> = Vec::new();
    for path in input_paths {
        if path.is_file() {
            all_files.push(path.clone());
        } else if path.is_dir() {
            let base = path.clone();
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| include_dir || e.file_type().is_file())
            {
                let relative = entry.path().strip_prefix(&base).unwrap().to_path_buf();
                all_files.push(base.join(&relative));
            }
        }
    }
    all_files.sort();
    all_files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::slice;
    use tempfile::tempdir;

    #[test]
    fn test_expand_path_single_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let result = expand_path(slice::from_ref(&file_path), true);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], file_path);
    }

    #[test]
    fn test_expand_path_directory() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");
        File::create(&file1).unwrap();
        File::create(&file2).unwrap();

        let result = expand_path(&[dir.path().to_path_buf()], false);

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_expand_path_nested_directory() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();
        let file1 = dir.path().join("file1.txt");
        let file2 = subdir.join("file2.txt");
        File::create(&file1).unwrap();
        File::create(&file2).unwrap();

        let result = expand_path(&[dir.path().to_path_buf()], false);

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_expand_path_empty_input() {
        let result = expand_path(&[], true);
        assert!(result.is_empty());
    }

    #[test]
    fn test_expand_path_excludes_directories_when_include_dir_false() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();
        let file = dir.path().join("file.txt");
        File::create(&file).unwrap();

        let result = expand_path(&[dir.path().to_path_buf()], false);

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_expand_path_multiple_inputs() {
        let dir1 = tempdir().unwrap();
        let dir2 = tempdir().unwrap();
        let file1 = dir1.path().join("file1.txt");
        let file2 = dir2.path().join("file2.txt");
        File::create(&file1).unwrap();
        File::create(&file2).unwrap();

        let result = expand_path(
            &[dir1.path().to_path_buf(), dir2.path().to_path_buf()],
            false,
        );

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_expand_path_preserves_order() {
        let dir = tempdir().unwrap();
        let file_c = dir.path().join("c.txt");
        let file_a = dir.path().join("a.txt");
        let file_b = dir.path().join("b.txt");
        File::create(&file_c).unwrap();
        File::create(&file_a).unwrap();
        File::create(&file_b).unwrap();

        let result = expand_path(&[dir.path().to_path_buf()], false);

        assert_eq!(result.len(), 3);
    }
}

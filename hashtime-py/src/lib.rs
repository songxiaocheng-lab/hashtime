//! Python bindings for hashtime.
//!
//! This module provides Python bindings for the hashtime library,
//! allowing Python programs to generate, compare, and restore file
//! hashes and timestamps.

use pyo3::prelude::*;
use pyo3::types::PyAny;
use std::collections::HashSet;
use std::path::PathBuf;
use std::str::FromStr;

use hashtime::FileTimeResult;
use hashtime::restore_times as core_restore_times;
use hashtime::{CompareField, Diff, DiffType, FieldDiff, compare as core_compare};
use hashtime::{
    FileHashTimeResult, generate as core_generate,
    generate_with_callback as core_generate_with_callback,
};

/// Result containing file path, hash values, and timestamps.
///
/// This is the primary result type returned by [`generate`].
/// It contains the file path and optionally computed hash values
/// and timestamps depending on the requested fields.
///
/// # Attributes
///
/// * `path` - The file path
/// * `size` - File size in bytes
/// * `created_ns` - Creation/birth time in nanoseconds since epoch
/// * `modified_ns` - Modification time in nanoseconds since epoch
/// * `md5` - MD5 hash (if requested)
/// * `sha1` - SHA1 hash (if requested)
/// * `sha256` - SHA256 hash (if requested)
/// * `sha512` - SHA512 hash (if requested)
#[pyclass(from_py_object, name = "FileHashTimeResult")]
#[derive(Clone)]
pub struct FileHashTimeResultPy {
    path: String,
    size: Option<u64>,
    created_ns: Option<i64>,
    modified_ns: Option<i64>,
    md5: Option<String>,
    sha1: Option<String>,
    sha256: Option<String>,
    sha512: Option<String>,
}

#[pymethods]
impl FileHashTimeResultPy {
    #[getter]
    fn path(&self) -> String {
        self.path.clone()
    }
    #[getter]
    fn size(&self) -> Option<u64> {
        self.size
    }
    #[getter]
    fn created_ns(&self) -> Option<i64> {
        self.created_ns
    }
    #[getter]
    fn modified_ns(&self) -> Option<i64> {
        self.modified_ns
    }
    #[getter]
    fn md5(&self) -> Option<String> {
        self.md5.clone()
    }
    #[getter]
    fn sha1(&self) -> Option<String> {
        self.sha1.clone()
    }
    #[getter]
    fn sha256(&self) -> Option<String> {
        self.sha256.clone()
    }
    #[getter]
    fn sha512(&self) -> Option<String> {
        self.sha512.clone()
    }
}

impl From<FileHashTimeResult> for FileHashTimeResultPy {
    fn from(rust: FileHashTimeResult) -> Self {
        FileHashTimeResultPy {
            path: rust.path.to_string_lossy().to_string(),
            size: rust.size,
            created_ns: rust.created_ns,
            modified_ns: rust.modified_ns,
            md5: rust.md5,
            sha1: rust.sha1,
            sha256: rust.sha256,
            sha512: rust.sha512,
        }
    }
}

/// Difference between two file states.
///
/// # Attributes
///
/// * `path` - The file path that changed
/// * `diff_type` - The type of change ("modified", "added", or "removed")
/// * `field_diffs` - List of field-level differences
#[pyclass(from_py_object, name = "Diff")]
#[derive(Clone)]
pub struct DiffPy {
    path: String,
    diff_type: String,
    field_diffs: Vec<FieldDiffPy>,
}

#[pymethods]
impl DiffPy {
    #[getter]
    fn path(&self) -> String {
        self.path.clone()
    }
    #[getter]
    fn diff_type(&self) -> String {
        self.diff_type.clone()
    }
    #[getter]
    fn field_diffs(&self) -> Vec<FieldDiffPy> {
        self.field_diffs.clone()
    }
}

impl From<Diff> for DiffPy {
    fn from(rust: Diff) -> Self {
        let diff_type = match rust.diff_type {
            DiffType::Modified => "modified",
            DiffType::Added => "added",
            DiffType::Removed => "removed",
        };
        let field_diffs: Vec<FieldDiffPy> = rust
            .field_diffs
            .into_iter()
            .map(FieldDiffPy::from)
            .collect();
        DiffPy {
            path: rust.path,
            diff_type: diff_type.to_string(),
            field_diffs,
        }
    }
}

/// A single field difference within a file diff.
///
/// # Attributes
///
/// * `field` - The field that changed
/// * `base` - The original value
/// * `target` - The new value
#[pyclass(from_py_object, name = "FieldDiff")]
#[derive(Clone)]
pub struct FieldDiffPy {
    field: String,
    base: String,
    target: String,
}

#[pymethods]
impl FieldDiffPy {
    #[getter]
    fn field(&self) -> String {
        self.field.clone()
    }
    #[getter]
    fn base(&self) -> String {
        self.base.clone()
    }
    #[getter]
    fn target(&self) -> String {
        self.target.clone()
    }
}

impl From<FieldDiff> for FieldDiffPy {
    fn from(rust: FieldDiff) -> Self {
        let field = match rust.field {
            CompareField::Md5 => "md5",
            CompareField::Sha1 => "sha1",
            CompareField::Sha256 => "sha256",
            CompareField::Sha512 => "sha512",
            CompareField::Birthtime => "birthtime",
            CompareField::Mtime => "mtime",
            CompareField::Size => "size",
        };
        FieldDiffPy {
            field: field.to_string(),
            base: rust.base,
            target: rust.target,
        }
    }
}

/// Generate hashes and timestamps for files.
///
/// # Arguments
///
/// * `input_paths` - List of file or directory paths to process
/// * `hash_fields` - List of hash algorithms ("md5", "sha1", "sha256", "sha512")
/// * `time_fields` - List of time fields ("mtime", "birthtime")
///
/// # Returns
///
/// A list of [`FileHashTimeResultPy`] objects.
#[pyfunction]
pub fn generate(
    _py: Python,
    input_paths: Vec<String>,
    hash_fields: Vec<String>,
    time_fields: Vec<String>,
) -> PyResult<Vec<FileHashTimeResultPy>> {
    let paths: Vec<PathBuf> = input_paths.iter().map(PathBuf::from).collect();
    let result = core_generate(&paths, &hash_fields, &time_fields);
    Ok(result.into_iter().map(FileHashTimeResultPy::from).collect())
}

/// Generate hashes and timestamps with progress callback.
///
/// Similar to [`generate`] but calls the provided callback for each processed file.
/// This is useful for progress bars or logging.
///
/// # Arguments
///
/// * `input_paths` - List of file or directory paths to process
/// * `hash_fields` - List of hash algorithms
/// * `time_fields` - List of time fields
/// * `callback` - A callable that receives each result as it's processed
#[pyfunction]
pub fn generate_with_callback(
    _py: Python<'_>,
    input_paths: Vec<String>,
    hash_fields: Vec<String>,
    time_fields: Vec<String>,
    callback: Bound<'_, PyAny>,
) -> PyResult<()> {
    let paths: Vec<PathBuf> = input_paths.iter().map(PathBuf::from).collect();

    let callback: Py<PyAny> = callback.into();

    core_generate_with_callback(
        &paths,
        &hash_fields,
        &time_fields,
        move |result: FileHashTimeResult| {
            let py_result = FileHashTimeResultPy::from(result);
            unsafe {
                let py = Python::assume_attached();
                let _ = callback.call1(py, (py_result,));
            }
        },
    );

    Ok(())
}

/// Compare two sets of file metadata results.
///
/// # Arguments
///
/// * `base_results` - Results from the base state
/// * `target_results` - Results from the target state
/// * `ignored_fields` - Fields to ignore in comparison
///
/// # Returns
///
/// A list of [`DiffPy`] objects.
#[pyfunction]
pub fn compare(
    _py: Python,
    base_results: Vec<FileHashTimeResultPy>,
    target_results: Vec<FileHashTimeResultPy>,
    ignored_fields: Vec<String>,
) -> PyResult<Vec<DiffPy>> {
    let base: Vec<FileHashTimeResult> = base_results
        .iter()
        .map(|r| FileHashTimeResult {
            path: PathBuf::from(&r.path),
            size: r.size,
            created_ns: r.created_ns,
            modified_ns: r.modified_ns,
            md5: r.md5.clone(),
            sha1: r.sha1.clone(),
            sha256: r.sha256.clone(),
            sha512: r.sha512.clone(),
        })
        .collect();

    let target: Vec<FileHashTimeResult> = target_results
        .iter()
        .map(|r| FileHashTimeResult {
            path: PathBuf::from(&r.path),
            size: r.size,
            created_ns: r.created_ns,
            modified_ns: r.modified_ns,
            md5: r.md5.clone(),
            sha1: r.sha1.clone(),
            sha256: r.sha256.clone(),
            sha512: r.sha512.clone(),
        })
        .collect();

    let ignored: HashSet<CompareField> = ignored_fields
        .iter()
        .filter_map(|s| CompareField::from_str(s).ok())
        .collect();

    let diffs = core_compare(&base, &target, &ignored);
    Ok(diffs.into_iter().map(DiffPy::from).collect())
}

/// Restore file timestamps from results.
///
/// # Arguments
///
/// * `results` - List of tuples (path, FileTimeResultPy)
#[pyfunction]
pub fn restore_times(_py: Python, results: Vec<(String, FileTimeResultPy)>) {
    let input_results: Vec<(PathBuf, FileTimeResult)> = results
        .iter()
        .map(|(p, r)| {
            (
                PathBuf::from(p),
                FileTimeResult {
                    created_ns: r.created_ns,
                    modified_ns: r.modified_ns,
                },
            )
        })
        .collect();
    core_restore_times(input_results);
}

/// File timestamp result for restoration.
///
/// Used with [`restore_times`] to restore file timestamps.
///
/// # Attributes
///
/// * `created_ns` - Creation/birth time in nanoseconds since epoch
/// * `modified_ns` - Modification time in nanoseconds since epoch
#[pyclass(from_py_object, name = "FileTimeResult")]
#[derive(Clone)]
pub struct FileTimeResultPy {
    pub created_ns: Option<i64>,
    pub modified_ns: Option<i64>,
}

#[pymethods]
impl FileTimeResultPy {
    #[new]
    fn new(created_ns: Option<i64>, modified_ns: Option<i64>) -> Self {
        FileTimeResultPy {
            created_ns,
            modified_ns,
        }
    }

    #[getter]
    fn created_ns(&self) -> Option<i64> {
        self.created_ns
    }

    #[getter]
    fn modified_ns(&self) -> Option<i64> {
        self.modified_ns
    }
}

impl From<FileTimeResult> for FileTimeResultPy {
    fn from(rust: FileTimeResult) -> Self {
        FileTimeResultPy {
            created_ns: rust.created_ns,
            modified_ns: rust.modified_ns,
        }
    }
}

#[pymodule]
pub fn _hashtime(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate, m)?)?;
    m.add_function(wrap_pyfunction!(generate_with_callback, m)?)?;
    m.add_function(wrap_pyfunction!(compare, m)?)?;
    m.add_function(wrap_pyfunction!(restore_times, m)?)?;

    m.add_class::<FileHashTimeResultPy>()?;
    m.add_class::<FileTimeResultPy>()?;
    m.add_class::<DiffPy>()?;
    m.add_class::<FieldDiffPy>()?;

    Ok(())
}

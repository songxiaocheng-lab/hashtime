# hashtime

[![PyPI version](https://img.shields.io/pypi/v/hashtime.svg)](https://pypi.org/project/hashtime/)
[![CI](https://github.com/songxiaocheng/hashtime/actions/workflows/ci.yml/badge.svg)](https://github.com/songxiaocheng/hashtime/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-blue.svg)](https://www.rust-lang.org)
[![Python](https://img.shields.io/badge/python-3.8%2B-blue.svg)](https://www.python.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A high-performance Rust library for generating and comparing file hashes and timestamps, exposed as a Python package and CLI tool.

## Features

- **Parallel processing**: Uses Rust's rayon for efficient parallel file processing
- **Output formats**: text, JSON, JSONL, CSV
- **Multi-algorithm hashing**: MD5, SHA1, SHA256, SHA512
- **Cross-platform timestamps**: Birth time (macOS/Windows), modification time
- **Comparison**: Compare file states and detect changes
- **Time restoration**: Restore file timestamps from saved results
- **CLI**: Command-line interface for quick operations
- **Python bindings**: Use directly from Python code

## Installation

### Rust (binary)

Download pre-built binaries from the [GitHub Releases](https://github.com/songxiaocheng/hashtime/releases):

### Rust (cargo)

```bash
cargo install hashtime-cli
```

### Python (pip)

```bash
pip install hashtime
```

### From source

```bash
# Build Rust CLI
cargo build --release

# Build Python package
cd python
maturin build --release
pip install target/wheels/hashtime-*.whl
```

## CLI Usage

After installation, use the `hashtime` command:

### Generate

```bash
# Generate hashes and timestamps for all files in a directory
hashtime gen /path/to/directory

# Generate with specific hash algorithms and time fields
hashtime gen -s md5,sha256 -t mtime /path/to/file.txt

# Generate only hashes
hashtime gen -S /path/to/directory

# Generate only timestamps
hashtime gen -T /path/to/directory

# Output to JSON file
hashtime gen -o results.jsonl -f jsonl /path/to/directory
```

### Check

```bash
# Check current files against saved metadata
hashtime check metadata.jsonl /path/to/directory

# Ignore specific fields
hashtime check -i mtime metadata.jsonl /path/to/directory
```

### Compare

```bash
# Compare two metadata files
hashtime compare base.jsonl target.jsonl

# Ignore specific fields
hashtime compare -i mtime base.jsonl target.jsonl
```

### Restore

```bash
# Restore timestamps from metadata file
hashtime restore metadata.jsonl /base/path

# Restore with --unsafe-debugfs to attempt birthtime on Linux (requires root + debugfs)
sudo hashtime restore --unsafe-debugfs metadata.jsonl /base/path
```

**Note**: On Linux, birthtime restore requires the `--unsafe-debugfs` flag because it uses a low-level debugfs hack that requires root privileges. Without this flag, birthtime is silently skipped with a warning.

## CLI Options

- `-o, --output-file`: Output file path
- `-s, --hashes`: Comma-separated hash algorithms (default: md5,sha1,sha256,sha512)
- `-t, --times`: Comma-separated time fields (default: birthtime,mtime)
- `-S, --hashes-only`: Generate only hashes
- `-T, --times-only`: Generate only timestamps
- `-f, --format`: Output format (text, json, jsonl, csv)
- `-c, --color`: Enable/disable color output
- `-i, --ignore-fields`: Fields to ignore during comparison
- `--unsafe-debugfs`: Use debugfs to restore birthtime on Linux (requires root + debugfs binary)

## Python Usage

### Generate file hashes and timestamps

```python
import hashtime

# Generate hashes and timestamps for files
results = hashtime.generate(
    input_paths=["/path/to/file.txt", "/path/to/directory"],
    hash_fields=["md5", "sha256"],  # Algorithms to compute
    time_fields=["mtime", "birthtime"]  # Timestamps to retrieve
)

for r in results:
    print(f"Path: {r.path}")
    print(f"  Size: {r.size}")
    print(f"  MD5: {r.md5}")
    print(f"  SHA256: {r.sha256}")
    print(f"  Modified: {r.modified_ns}")
    print(f"  Created: {r.created_ns}")
```

### Using callback for progress reporting

```python
import hashtime

def progress_callback(result):
    print(f"Processed: {result.path}")

hashtime.generate_with_callback(
    input_paths=["/path/to/files"],
    hash_fields=["md5"],
    time_fields=[],
    callback=progress_callback
)
```

### Compare two states

```python
import hashtime

# Generate results for two different times
base_results = hashtime.generate(
    input_paths=["/path/to/files"],
    hash_fields=["md5", "sha256"],
    time_fields=["mtime"]
)

# ... some time passes and files change ...

target_results = hashtime.generate(
    input_paths=["/path/to/files"],
    hash_fields=["md5", "sha256"],
    time_fields=["mtime"]
)

# Compare the results
diffs = hashtime.compare(
    base_results=base_results,
    target_results=target_results,
    ignored_fields=[]  # Fields to ignore in comparison
)

for diff in diffs:
    print(f"{diff.diff_type}: {diff.path}")
    for field_diff in diff.field_diffs:
        print(f"  {field_diff.field}: {field_diff.base} -> {field_diff.target}")
```

### Restore file timestamps

```python
import hashtime

# First generate results with time fields
results = hashtime.generate(
    input_paths=["/path/to/files"],
    hash_fields=[],
    time_fields=["mtime", "birthtime"]
)

# Later, restore the timestamps
time_results = [(r.path, hashtime.FileTimeResultPy(
    created_ns=r.created_ns,
    modified_ns=r.modified_ns
)) for r in results if r.created_ns or r.modified_ns]

hashtime.restore_times(time_results)
```

## API Reference

### Python API

#### `generate(input_paths, hash_fields, time_fields)`

Generate hashes and timestamps for files.

- **input_paths**: List of file or directory paths to process
- **hash_fields**: List of hash algorithms ("md5", "sha1", "sha256", "sha512")
- **time_fields**: List of time fields ("mtime", "birthtime")

Returns a list of `FileHashTimeResultPy` objects.

#### `generate_with_callback(input_paths, hash_fields, time_fields, callback)`

Generate with progress callback.

- **callback**: A callable that receives each result as it's processed

#### `compare(base_results, target_results, ignored_fields)`

Compare two sets of results.

- **base_results**: Results from the base state
- **target_results**: Results from the target state
- **ignored_fields**: Fields to ignore in comparison

Returns a list of `DiffPy` objects.

#### `restore_times(results)`

Restore file timestamps from results.

- **results**: List of tuples (path, FileTimeResultPy)

### Data Types

#### FileHashTimeResultPy

- `path`: str - File path
- `size`: Optional[int] - File size in bytes
- `md5`: Optional[str] - MD5 hash
- `sha1`: Optional[str] - SHA1 hash
- `sha256`: Optional[str] - SHA256 hash
- `sha512`: Optional[str] - SHA512 hash
- `modified_ns`: Optional[int] - Modification time in nanoseconds since epoch
- `created_ns`: Optional[int] - Creation/birth time in nanoseconds since epoch

#### FileTimeResultPy

- `modified_ns`: Optional[int] - Modification time
- `created_ns`: Optional[int] - Creation time

#### DiffPy

- `path`: str - File path that changed
- `diff_type`: str - "modified", "added", or "removed"
- `field_diffs`: List[FieldDiffPy] - List of field differences

#### FieldDiffPy

- `field`: str - The field that changed
- `base`: str - Original value
- `target`: str - New value

## Performance

hashtime uses Rust's rayon for parallel processing, making it significantly faster than pure Python implementations for processing large directories.

## Building and Publishing

### Prerequisites

- Rust toolchain (1.80+)
- Python 3.8+
- maturin (`pip install maturin`)

### Build

```bash
cd python
maturin build --release
```

### Publish to PyPI

```bash
cd python
maturin upload
```

## License

MIT License - see [LICENSE](LICENSE) file for details.

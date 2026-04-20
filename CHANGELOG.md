# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-04-11

### Added

- Multi-algorithm hashing: MD5, SHA1, SHA256, SHA512
- Cross-platform timestamps: birth time (macOS/Windows), modification time
- Comparison: Check and compare file hashes and timestamps
- Restoration: Restore timestamps from saved results
- CLI: Command-line interface for quick operations
- Python bindings: Use directly from Python code
- Parallel processing: Uses Rust's rayon for efficient parallel file processing
- Output formats: text, JSON, JSONL, CSV

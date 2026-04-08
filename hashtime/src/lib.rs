//! # hashtime
//!
//! A high-performance Rust library for generating and comparing file hashes and timestamps.
//!
//! ## Features
//!
//! - **Multi-algorithm hashing**: MD5, SHA1, SHA256, SHA512
//! - **Cross-platform timestamps**: Birth time (macOS/Windows), modification time
//! - **Parallel processing**: Uses Rust's rayon for efficient parallel file processing
//! - **Comparison**: Compare file states and detect changes
//! - **Time restoration**: Restore file timestamps from saved results

mod compare;
mod file_hash;
mod file_time;
mod generate;
mod restore_times;
mod utils;

pub use crate::compare::{CompareField, Diff, DiffType, FieldDiff, compare};
pub use crate::file_hash::FileHashResult;
pub use crate::file_time::FileTimeResult;
pub use crate::generate::{FileHashTimeResult, generate, generate_with_callback};
pub use crate::restore_times::restore_times;

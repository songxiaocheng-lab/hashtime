//! CLI command implementations for hashtime.
//!
//! This module contains the command handlers for the CLI subcommands:
//! - `generate`: Generate file hashes and timestamps
//! - `check`: Check current files against a stored metadata file
//! - `compare`: Compare two metadata files
//! - `restore`: Restore file timestamps from a metadata file

pub(crate) mod check;
pub(crate) mod compare;
pub(crate) mod generate;
pub(crate) mod restore;

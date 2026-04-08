//! File hashing using multiple cryptographic algorithms.
//!
//! This module provides parallel computation of file hashes using OpenSSL.
//! Supported algorithms: MD5, SHA1, SHA256, SHA512.

use openssl::hash::{Hasher, MessageDigest};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{File, Metadata};
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

/// Chunk size for reading files during hash computation (1 MiB).
const CHUNK_SIZE: usize = 1024 * 1024;

/// Hash algorithm selection for file hashing.
///
/// # Variants
///
/// * `Md5` - MD5 hash (128-bit, not cryptographically secure)
/// * `Sha1` - SHA-1 hash (160-bit, deprecated for security)
/// * `Sha256` - SHA-256 hash (256-bit, recommended)
/// * `Sha512` - SHA-512 hash (512-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashField {
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

impl FromStr for HashField {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "md5" => Ok(HashField::Md5),
            "sha1" => Ok(HashField::Sha1),
            "sha256" => Ok(HashField::Sha256),
            "sha512" => Ok(HashField::Sha512),
            _ => Err(()),
        }
    }
}

impl HashField {
    fn to_openssl_digest(self) -> MessageDigest {
        match self {
            HashField::Md5 => MessageDigest::md5(),
            HashField::Sha1 => MessageDigest::sha1(),
            HashField::Sha256 => MessageDigest::sha256(),
            HashField::Sha512 => MessageDigest::sha512(),
        }
    }
}

/// Result of file hash computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHashResult {
    pub size: u64,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub sha512: Option<String>,
}

/// Convert bytes to lowercase hexadecimal string.
#[inline]
fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8] = b"0123456789abcdef";
    let mut hex = Vec::with_capacity(bytes.len() * 2);
    for &b in bytes {
        hex.push(HEX[(b >> 4) as usize]);
        hex.push(HEX[(b & 0x0f) as usize]);
    }
    unsafe { String::from_utf8_unchecked(hex) }
}

/// Compute hashes for a file using multiple algorithms in parallel.
///
/// This function reads the file once and computes all requested hash algorithms
/// simultaneously using rayon for parallel hash updates.
///
/// Returns a map of hash algorithm to hex-encoded hash string.
pub(crate) fn compute_hashes_parallel(
    path: &Path,
    algorithms: &HashSet<HashField>,
) -> std::collections::HashMap<HashField, String> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return std::collections::HashMap::new(),
    };

    let mut hashers: Vec<(HashField, Hasher)> = algorithms
        .iter()
        .filter_map(|algo| {
            Hasher::new((*algo).to_openssl_digest())
                .ok()
                .map(|h| (*algo, h))
        })
        .collect();

    if hashers.is_empty() {
        return std::collections::HashMap::new();
    }

    let mut buf = vec![0u8; CHUNK_SIZE];
    loop {
        let n = match file.read(&mut buf) {
            Ok(n) => n,
            Err(_) => return std::collections::HashMap::new(),
        };
        if n == 0 {
            break;
        }
        let data = &buf[..n];
        rayon::scope(|s| {
            for (_, h) in hashers.iter_mut() {
                s.spawn(move |_| {
                    let _ = h.update(data);
                });
            }
        });
    }

    hashers
        .into_iter()
        .filter_map(|(algo, mut h)| h.finish().ok().map(|r| (algo, to_hex(&r))))
        .collect()
}

/// Compute hashes for a file and return a structured result.
///
/// This is the main entry point for hash computation. It wraps [`compute_hashes_parallel`]
/// and combines the hash results with file size into a [`FileHashResult`] struct.
///
/// # Arguments
///
/// * `path` - Path to the file
/// * `meta` - File metadata (must be obtained from the same path)
/// * `algorithms` - Set of hash algorithms to compute
///
/// # Returns
///
/// A [`FileHashResult`] containing the file size and requested hash values.
pub(crate) fn get_hashes(
    path: &Path,
    meta: &Metadata,
    algorithms: &HashSet<HashField>,
) -> FileHashResult {
    let hashes = compute_hashes_parallel(path, algorithms);
    FileHashResult {
        size: meta.len(),
        md5: hashes.get(&HashField::Md5).cloned(),
        sha1: hashes.get(&HashField::Sha1).cloned(),
        sha256: hashes.get(&HashField::Sha256).cloned(),
        sha512: hashes.get(&HashField::Sha512).cloned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_hash_field_from_str() {
        assert_eq!(HashField::from_str("md5").ok(), Some(HashField::Md5));
        assert_eq!(HashField::from_str("MD5").ok(), Some(HashField::Md5));
        assert_eq!(HashField::from_str("sha1").ok(), Some(HashField::Sha1));
        assert_eq!(HashField::from_str("SHA1").ok(), Some(HashField::Sha1));
        assert_eq!(HashField::from_str("sha256").ok(), Some(HashField::Sha256));
        assert_eq!(HashField::from_str("sha512").ok(), Some(HashField::Sha512));
        assert_eq!(HashField::from_str("invalid").ok(), None);
        assert_eq!(HashField::from_str("").ok(), None);
    }

    #[test]
    fn test_compute_hashes_md5() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let mut algorithms = HashSet::new();
        algorithms.insert(HashField::Md5);

        let hashes = compute_hashes_parallel(&file_path, &algorithms);

        assert!(hashes.contains_key(&HashField::Md5));
        assert_eq!(
            hashes.get(&HashField::Md5).unwrap(),
            "5eb63bbbe01eeed093cb22bb8f5acdc3"
        );
    }

    #[test]
    fn test_compute_hashes_sha1() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let mut algorithms = HashSet::new();
        algorithms.insert(HashField::Sha1);

        let hashes = compute_hashes_parallel(&file_path, &algorithms);

        assert!(hashes.contains_key(&HashField::Sha1));
        assert_eq!(
            hashes.get(&HashField::Sha1).unwrap(),
            "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed"
        );
    }

    #[test]
    fn test_compute_hashes_sha256() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let mut algorithms = HashSet::new();
        algorithms.insert(HashField::Sha256);

        let hashes = compute_hashes_parallel(&file_path, &algorithms);

        assert!(hashes.contains_key(&HashField::Sha256));
        assert_eq!(
            hashes.get(&HashField::Sha256).unwrap(),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_compute_hashes_sha512() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let mut algorithms = HashSet::new();
        algorithms.insert(HashField::Sha512);

        let hashes = compute_hashes_parallel(&file_path, &algorithms);

        assert!(hashes.contains_key(&HashField::Sha512));
        assert_eq!(
            hashes.get(&HashField::Sha512).unwrap(),
            "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f"
        );
    }

    #[test]
    fn test_compute_hashes_multiple_algorithms() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let mut algorithms = HashSet::new();
        algorithms.insert(HashField::Md5);
        algorithms.insert(HashField::Sha256);

        let hashes = compute_hashes_parallel(&file_path, &algorithms);

        assert!(hashes.contains_key(&HashField::Md5));
        assert!(hashes.contains_key(&HashField::Sha256));
        assert!(!hashes.contains_key(&HashField::Sha1));
        assert!(!hashes.contains_key(&HashField::Sha512));
    }

    #[test]
    fn test_compute_hashes_empty_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("empty.txt");
        File::create(&file_path).unwrap();

        let mut algorithms = HashSet::new();
        algorithms.insert(HashField::Md5);

        let hashes = compute_hashes_parallel(&file_path, &algorithms);

        assert!(hashes.contains_key(&HashField::Md5));
        assert_eq!(
            hashes.get(&HashField::Md5).unwrap(),
            "d41d8cd98f00b204e9800998ecf8427e"
        );
    }

    #[test]
    fn test_compute_hashes_nonexistent_file() {
        let file_path = PathBuf::from("/nonexistent/file.txt");

        let mut algorithms = HashSet::new();
        algorithms.insert(HashField::Md5);

        let hashes = compute_hashes_parallel(&file_path, &algorithms);

        assert!(hashes.is_empty());
    }

    #[test]
    fn test_get_hashes() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();

        let meta = std::fs::metadata(&file_path).unwrap();
        let mut algorithms = HashSet::new();
        algorithms.insert(HashField::Md5);
        algorithms.insert(HashField::Sha256);

        let result = get_hashes(&file_path, &meta, &algorithms);

        assert_eq!(result.size, 12);
        assert!(result.md5.is_some());
        assert!(result.sha1.is_none());
        assert!(result.sha256.is_some());
        assert!(result.sha512.is_none());
    }

    #[test]
    fn test_large_file_md5() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("large.bin");

        let pattern = b"abcdefghijklmnopqrstuvwxyz";
        let full_size = 2 * 1024 * 1024 + 512 * 1024;
        let repeats = full_size / pattern.len();
        let remainder = full_size % pattern.len();

        let mut file = File::create(&file_path).unwrap();
        for _ in 0..repeats {
            file.write_all(pattern).unwrap();
        }
        file.write_all(&pattern[..remainder]).unwrap();

        let mut algorithms = HashSet::new();
        algorithms.insert(HashField::Md5);

        let hashes = compute_hashes_parallel(&file_path, &algorithms);

        assert!(hashes.contains_key(&HashField::Md5));
        assert_eq!(
            hashes.get(&HashField::Md5).unwrap(),
            "9ee1ab5fe19767a21f8fb10bea8f0964"
        );
    }

    #[test]
    fn test_large_file_sha1() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("large.bin");

        let pattern = b"abcdefghijklmnopqrstuvwxyz";
        let full_size = 2 * 1024 * 1024 + 512 * 1024;
        let repeats = full_size / pattern.len();
        let remainder = full_size % pattern.len();

        let mut file = File::create(&file_path).unwrap();
        for _ in 0..repeats {
            file.write_all(pattern).unwrap();
        }
        file.write_all(&pattern[..remainder]).unwrap();

        let mut algorithms = HashSet::new();
        algorithms.insert(HashField::Sha1);

        let hashes = compute_hashes_parallel(&file_path, &algorithms);

        assert!(hashes.contains_key(&HashField::Sha1));
        assert_eq!(
            hashes.get(&HashField::Sha1).unwrap(),
            "984720ef9d2c861e8454499fc06265d77ee28869"
        );
    }

    #[test]
    fn test_large_file_sha256() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("large.bin");

        let pattern = b"abcdefghijklmnopqrstuvwxyz";
        let full_size = 2 * 1024 * 1024 + 512 * 1024;
        let repeats = full_size / pattern.len();
        let remainder = full_size % pattern.len();

        let mut file = File::create(&file_path).unwrap();
        for _ in 0..repeats {
            file.write_all(pattern).unwrap();
        }
        file.write_all(&pattern[..remainder]).unwrap();

        let mut algorithms = HashSet::new();
        algorithms.insert(HashField::Sha256);

        let hashes = compute_hashes_parallel(&file_path, &algorithms);

        assert!(hashes.contains_key(&HashField::Sha256));
        assert_eq!(
            hashes.get(&HashField::Sha256).unwrap(),
            "dafb34e8137381655cc42ba644bacd3fd393c3971d702adac029582e92d06d14"
        );
    }

    #[test]
    fn test_large_file_sha512() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("large.bin");

        let pattern = b"abcdefghijklmnopqrstuvwxyz";
        let full_size = 2 * 1024 * 1024 + 512 * 1024;
        let repeats = full_size / pattern.len();
        let remainder = full_size % pattern.len();

        let mut file = File::create(&file_path).unwrap();
        for _ in 0..repeats {
            file.write_all(pattern).unwrap();
        }
        file.write_all(&pattern[..remainder]).unwrap();

        let mut algorithms = HashSet::new();
        algorithms.insert(HashField::Sha512);

        let hashes = compute_hashes_parallel(&file_path, &algorithms);

        assert!(hashes.contains_key(&HashField::Sha512));
        assert_eq!(
            hashes.get(&HashField::Sha512).unwrap(),
            "6e1c7f095ecb08647a5bf63965ab96c0cff27e3165199435f627e61345fc9891d21f3e7e180c5e6a5f9569c619bdbcbdc524e2c701e5c15ba16b50e7fe3f1f81"
        );
    }

    #[test]
    fn test_large_file_all_algorithms() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("large.bin");

        let pattern = b"abcdefghijklmnopqrstuvwxyz";
        let full_size = 2 * 1024 * 1024 + 512 * 1024;
        let repeats = full_size / pattern.len();
        let remainder = full_size % pattern.len();

        let mut file = File::create(&file_path).unwrap();
        for _ in 0..repeats {
            file.write_all(pattern).unwrap();
        }
        file.write_all(&pattern[..remainder]).unwrap();

        let mut algorithms = HashSet::new();
        algorithms.insert(HashField::Md5);
        algorithms.insert(HashField::Sha1);
        algorithms.insert(HashField::Sha256);
        algorithms.insert(HashField::Sha512);

        let hashes = compute_hashes_parallel(&file_path, &algorithms);

        assert_eq!(
            hashes.get(&HashField::Md5).unwrap(),
            "9ee1ab5fe19767a21f8fb10bea8f0964"
        );
        assert_eq!(
            hashes.get(&HashField::Sha1).unwrap(),
            "984720ef9d2c861e8454499fc06265d77ee28869"
        );
        assert_eq!(
            hashes.get(&HashField::Sha256).unwrap(),
            "dafb34e8137381655cc42ba644bacd3fd393c3971d702adac029582e92d06d14"
        );
        assert_eq!(
            hashes.get(&HashField::Sha512).unwrap(),
            "6e1c7f095ecb08647a5bf63965ab96c0cff27e3165199435f627e61345fc9891d21f3e7e180c5e6a5f9569c619bdbcbdc524e2c701e5c15ba16b50e7fe3f1f81"
        );
    }

    #[test]
    fn test_chunk_boundary_all_algorithms() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("exactly_1mb.bin");

        let chunk_size = 1024 * 1024;
        let pattern = b"abcdefghijklmnopqrstuvwxyz";
        let repeats = chunk_size / pattern.len();
        let remainder = chunk_size % pattern.len();

        let mut file = File::create(&file_path).unwrap();
        for _ in 0..repeats {
            file.write_all(pattern).unwrap();
        }
        file.write_all(&pattern[..remainder]).unwrap();

        let meta = std::fs::metadata(&file_path).unwrap();
        assert_eq!(meta.len(), chunk_size as u64);

        let mut algorithms = HashSet::new();
        algorithms.insert(HashField::Md5);
        algorithms.insert(HashField::Sha1);
        algorithms.insert(HashField::Sha256);
        algorithms.insert(HashField::Sha512);

        let hashes = compute_hashes_parallel(&file_path, &algorithms);

        assert_eq!(
            hashes.get(&HashField::Md5).unwrap(),
            "b63ba06de0e8a9626d5bcf27e93bf32d"
        );
        assert_eq!(
            hashes.get(&HashField::Sha1).unwrap(),
            "dd89d1965604bd939ec68a6ca4552788f0eb1f88"
        );
        assert_eq!(
            hashes.get(&HashField::Sha256).unwrap(),
            "8816f31ba2861e2a7ad907085905efdea5b458d26ed6fe4929ae21467ba1fa97"
        );
        assert_eq!(
            hashes.get(&HashField::Sha512).unwrap(),
            "11b5ba0f12243c7ffe8de4570fc0c5cf4e3929f580885f5e9acb947bc502392c7f29ae088ac3e0d51b047a8c53038f2092c5788e66e7b992903f0f4c970798cf"
        );
    }
}

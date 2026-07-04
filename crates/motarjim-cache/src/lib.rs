#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! Content-addressable cache for compiled artifacts.
//!
//! Uses SHA-256 file hashing for cache keys to enable efficient
//! artifact reuse and invalidation. Artifacts are stored on disk
//! in a directory structure keyed by [`CacheKey`].

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// A cache key uniquely identifying a compilation artifact.
///
/// The triple of source hash, platform, and config hash disambiguates
/// artifacts across builds, targets, and configuration changes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// SHA-256 hash of the source file contents.
    pub source_hash: [u8; 32],
    /// Target platform identifier (e.g. `"wasm32"`, `"x86_64-linux"`).
    pub platform: String,
    /// SHA-256 hash of the compiler configuration.
    pub config_hash: [u8; 32],
}

impl CacheKey {
    /// Create a new cache key.
    #[must_use]
    pub fn new(source_hash: [u8; 32], platform: impl Into<String>, config_hash: [u8; 32]) -> Self {
        Self {
            source_hash,
            platform: platform.into(),
            config_hash,
        }
    }
}

/// Metadata about a cached artifact.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Size of the cached data in bytes.
    pub size: u64,
    /// Path to the cached artifact on disk.
    pub path: PathBuf,
}

/// Errors that can occur during cache operations.
#[derive(Debug, Clone)]
pub enum CacheError {
    /// An I/O error occurred.
    Io(String),
    /// The cache key contains invalid data.
    InvalidKey(String),
    /// Stored data failed checksum verification.
    ChecksumMismatch,
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "I/O error: {msg}"),
            Self::InvalidKey(msg) => write!(f, "invalid cache key: {msg}"),
            Self::ChecksumMismatch => write!(f, "checksum mismatch"),
        }
    }
}

impl std::error::Error for CacheError {}

impl From<std::io::Error> for CacheError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

/// Statistics about cache usage.
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Number of entries in the cache.
    pub entries: u64,
    /// Number of cache hits since creation.
    pub hits: u64,
    /// Number of cache misses since creation.
    pub misses: u64,
    /// Total size of cached artifacts in bytes.
    pub size_bytes: u64,
}

/// A content-addressable cache for compiled artifacts.
///
/// Artifacts are stored on disk at `{cache_dir}/{platform}/{source_hex}_{config_hex}`.
#[derive(Debug)]
pub struct ArtifactCache {
    /// Root directory for cached artifacts.
    cache_dir: PathBuf,
    /// Total number of cache hits.
    hits: AtomicU64,
    /// Total number of cache misses.
    misses: AtomicU64,
}

impl ArtifactCache {
    /// Create a new artifact cache rooted at `cache_dir`.
    ///
    /// The directory is created lazily on the first [`store`](Self::store) call.
    #[must_use]
    pub const fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    /// Compute the on-disk path for a cache key.
    fn key_path(&self, key: &CacheKey) -> PathBuf {
        let source_hex = hex_encode(&key.source_hash);
        let config_hex = hex_encode(&key.config_hash);
        self.cache_dir
            .join(&key.platform)
            .join(format!("{source_hex}_{config_hex}"))
    }

    /// Store data in the cache under the given key.
    ///
    /// # Errors
    /// Returns [`CacheError::Io`] if the data cannot be written.
    pub fn store(&self, key: &CacheKey, data: &[u8]) -> Result<CacheEntry, CacheError> {
        let path = self.key_path(key);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, data)?;
        Ok(CacheEntry {
            size: data.len() as u64,
            path,
        })
    }

    /// Load data from the cache.
    ///
    /// Returns `Ok(None)` if no entry exists for the given key.
    ///
    /// # Errors
    /// Returns [`CacheError::Io`] if the data cannot be read.
    pub fn load(&self, key: &CacheKey) -> Result<Option<Vec<u8>>, CacheError> {
        let path = self.key_path(key);
        if path.exists() {
            let data = std::fs::read(&path)?;
            self.hits.fetch_add(1, Ordering::Relaxed);
            Ok(Some(data))
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            Ok(None)
        }
    }

    /// Check whether an entry exists for the given key.
    #[must_use]
    pub fn has(&self, key: &CacheKey) -> bool {
        self.key_path(key).exists()
    }

    /// Clear all cached artifacts.
    ///
    /// Removes and re-creates the cache directory.
    ///
    /// # Errors
    /// Returns [`CacheError::Io`] if the directory cannot be cleared.
    pub fn clear(&self) -> Result<(), CacheError> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
        }
        std::fs::create_dir_all(&self.cache_dir)?;
        Ok(())
    }

    /// Return cache usage statistics.
    #[must_use]
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: Self::count_entries(&self.cache_dir),
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            size_bytes: Self::compute_size(&self.cache_dir),
        }
    }

    /// Count the number of cached entries (files) recursively.
    fn count_entries(dir: &Path) -> u64 {
        let mut count = 0;
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    count += 1;
                } else if path.is_dir() {
                    count += Self::count_entries(&path);
                }
            }
        }
        count
    }

    /// Compute total byte size of cached entries recursively.
    fn compute_size(dir: &Path) -> u64 {
        let mut total = 0;
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(meta) = path.metadata() {
                        total += meta.len();
                    }
                } else if path.is_dir() {
                    total += Self::compute_size(&path);
                }
            }
        }
        total
    }
}

/// Encode bytes as a lowercase hex string.
fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(hex_nibble(b >> 4));
        s.push(hex_nibble(b & 0x0f));
    }
    s
}

/// Encodes a 4-bit value as a hexadecimal character.
fn hex_nibble(byte: u8) -> char {
    match byte {
        0..=9 => char::from(b'0' + byte),
        10..=15 => char::from(b'a' + byte - 10),
        _ => '0',
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;

    static TEST_CNT: AtomicU64 = AtomicU64::new(0);

    fn test_key() -> CacheKey {
        CacheKey {
            source_hash: [1; 32],
            platform: "test".to_string(),
            config_hash: [2; 32],
        }
    }

    fn temp_dir() -> PathBuf {
        let n = TEST_CNT.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("m_cache_test_{}_{}", std::process::id(), n));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn test_store_and_load() {
        let dir = temp_dir();
        let cache = ArtifactCache::new(dir.clone());
        let key = test_key();
        let data = b"hello world";
        let entry = cache.store(&key, data).unwrap();
        assert_eq!(entry.size, 11);
        let loaded = cache.load(&key).unwrap();
        assert_eq!(loaded, Some(b"hello world".to_vec()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_missing() {
        let dir = temp_dir();
        let cache = ArtifactCache::new(dir.clone());
        let loaded = cache.load(&test_key()).unwrap();
        assert!(loaded.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_has() {
        let dir = temp_dir();
        let cache = ArtifactCache::new(dir.clone());
        let key = test_key();
        assert!(!cache.has(&key));
        cache.store(&key, b"data").unwrap();
        assert!(cache.has(&key));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_clear() {
        let dir = temp_dir();
        let cache = ArtifactCache::new(dir.clone());
        cache.store(&test_key(), b"data").unwrap();
        assert!(cache.has(&test_key()));
        cache.clear().unwrap();
        assert!(!cache.has(&test_key()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_stats() {
        let dir = temp_dir();
        let cache = ArtifactCache::new(dir.clone());
        let key = test_key();
        let s = cache.stats();
        assert_eq!(s.entries, 0);
        assert_eq!(s.hits, 0);
        assert_eq!(s.misses, 0);

        cache.store(&key, b"hello").unwrap();
        let _ = cache.load(&key).unwrap();

        let other = CacheKey::new([9; 32], "test", [9; 32]);
        let _ = cache.load(&other).unwrap();

        let s = cache.stats();
        assert_eq!(s.entries, 1);
        assert_eq!(s.hits, 1);
        assert_eq!(s.misses, 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_different_keys_isolated() {
        let dir = temp_dir();
        let cache = ArtifactCache::new(dir.clone());
        let k1 = CacheKey::new([1; 32], "linux", [1; 32]);
        let k2 = CacheKey::new([2; 32], "linux", [2; 32]);
        cache.store(&k1, b"data1").unwrap();
        cache.store(&k2, b"data2").unwrap();
        assert_eq!(cache.load(&k1).unwrap(), Some(b"data1".to_vec()));
        assert_eq!(cache.load(&k2).unwrap(), Some(b"data2".to_vec()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_platform_isolation() {
        let dir = temp_dir();
        let cache = ArtifactCache::new(dir.clone());
        let k_linux = CacheKey::new([1; 32], "linux", [2; 32]);
        let k_wasm = CacheKey::new([1; 32], "wasm", [2; 32]);
        cache.store(&k_linux, b"linux").unwrap();
        cache.store(&k_wasm, b"wasm").unwrap();
        assert_eq!(cache.load(&k_linux).unwrap(), Some(b"linux".to_vec()));
        assert_eq!(cache.load(&k_wasm).unwrap(), Some(b"wasm".to_vec()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_stats_reset_after_clear() {
        let dir = temp_dir();
        let cache = ArtifactCache::new(dir.clone());
        cache.store(&test_key(), b"data").unwrap();
        assert_eq!(cache.stats().entries, 1);
        cache.clear().unwrap();
        assert_eq!(cache.stats().entries, 0);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! Incremental compilation tracker.
//!
//! Tracks file content hashes across compilation sessions and
//! determines which files need recompilation when they change.

use std::collections::HashMap;
use std::fmt::{self, Write};
use std::path::{Path, PathBuf};

/// The compilation status of a tracked file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilationStatus {
    /// File is up-to-date — its hash matches the recorded state.
    UpToDate,
    /// File has changed since it was last compiled.
    Stale,
    /// File has never been tracked before.
    New,
}

/// Persistent state for a single tracked file.
#[derive(Debug, Clone)]
pub struct FileState {
    /// Absolute or workspace-relative path to the file.
    pub path: PathBuf,
    /// SHA-256 hash of the file contents at the time of last compilation.
    pub hash: [u8; 32],
}

impl FileState {
    /// Create a new file state.
    #[must_use]
    pub const fn new(path: PathBuf, hash: [u8; 32]) -> Self {
        Self { path, hash }
    }
}

/// Describes a single file change detected during a build.
#[derive(Debug, Clone)]
pub struct FileChange {
    /// Path of the changed file.
    pub path: PathBuf,
    /// New SHA-256 hash of the file contents.
    pub new_hash: [u8; 32],
}

impl FileChange {
    /// Create a new file change descriptor.
    #[must_use]
    pub const fn new(path: PathBuf, new_hash: [u8; 32]) -> Self {
        Self { path, new_hash }
    }
}

/// Errors that can occur in the incremental compilation engine.
#[derive(Debug, Clone)]
pub enum IncrementalError {
    /// An I/O error occurred reading or writing state.
    Io(String),
    /// The persisted state file is corrupted or unparseable.
    CorruptState(String),
}

impl fmt::Display for IncrementalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "I/O error: {msg}"),
            Self::CorruptState(msg) => write!(f, "corrupt state: {msg}"),
        }
    }
}

impl std::error::Error for IncrementalError {}

impl From<std::io::Error> for IncrementalError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

/// Tracks file states across compilation sessions and determines
/// which files need recompilation.
#[derive(Debug)]
pub struct IncrementalEngine {
    /// Directory for persisting compilation state.
    state_dir: PathBuf,
    /// Tracked file states keyed by path.
    files: HashMap<PathBuf, FileState>,
}

impl IncrementalEngine {
    /// Create a new incremental compilation engine.
    ///
    /// The `state_dir` is used to persist file hashes between
    /// compilation sessions via [`save_state`](Self::save_state)
    /// and [`load_state`](Self::load_state).
    #[must_use]
    pub fn new(state_dir: PathBuf) -> Self {
        Self {
            state_dir,
            files: HashMap::new(),
        }
    }

    /// Register or update a file's tracked hash.
    pub fn register_file(&mut self, path: &Path, hash: [u8; 32]) {
        self.files
            .insert(path.to_path_buf(), FileState::new(path.to_path_buf(), hash));
    }

    /// Check whether a file's hash differs from its registered state.
    ///
    /// Returns `true` if the file has never been registered or if the
    /// hash differs from the registered value.
    #[must_use]
    pub fn has_changed(&self, path: &Path, hash: [u8; 32]) -> bool {
        self.files
            .get(path)
            .is_none_or(|state| state.hash != hash)
    }

    /// Return the subset of changes that need recompilation.
    ///
    /// A change needs recompilation if the file is not registered or
    /// its hash has changed.
    #[must_use]
    pub fn files_needing_recompile(&self, changes: &[FileChange]) -> Vec<PathBuf> {
        changes
            .iter()
            .filter(|change| self.has_changed(&change.path, change.new_hash))
            .map(|change| change.path.clone())
            .collect()
    }

    /// Return the compilation status of a file given its current hash.
    #[must_use]
    pub fn status(&self, path: &Path, current_hash: [u8; 32]) -> CompilationStatus {
        match self.files.get(path) {
            None => CompilationStatus::New,
            Some(state) if state.hash != current_hash => CompilationStatus::Stale,
            Some(_) => CompilationStatus::UpToDate,
        }
    }

    /// Persist the current file state to disk.
    ///
    /// State is written as a line-delimited CSV file
    /// (`path,hex_hash`) at `{state_dir}/incremental.state`.
    ///
    /// # Errors
    /// Returns [`IncrementalError::Io`] if the file cannot be written.
    pub fn save_state(&self) -> Result<(), IncrementalError> {
        let state_path = self.state_dir.join("incremental.state");
        if let Some(parent) = state_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut content = String::new();
        for state in self.files.values() {
            let _ = writeln!(
                content,
                "{},{}",
                state.path.display(),
                hex_encode(&state.hash)
            );
        }
        std::fs::write(&state_path, content.as_bytes())?;
        Ok(())
    }

    /// Load file state from disk, replacing any in-memory state.
    ///
    /// If no state file exists the engine starts with an empty map.
    ///
    /// # Errors
    /// Returns [`IncrementalError::Io`] if the file cannot be read, or
    /// [`IncrementalError::CorruptState`] if the format is invalid.
    pub fn load_state(&mut self) -> Result<(), IncrementalError> {
        let state_path = self.state_dir.join("incremental.state");
        if !state_path.exists() {
            return Ok(());
        }
        let content = std::fs::read_to_string(&state_path)?;
        self.files.clear();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let mut parts = line.splitn(2, ',');
            let path_str = parts.next().ok_or_else(|| {
                IncrementalError::CorruptState("missing path field".to_string())
            })?;
            let hash_str = parts.next().ok_or_else(|| {
                IncrementalError::CorruptState("missing hash field".to_string())
            })?;
            let hash = hex_decode(hash_str).ok_or_else(|| {
                IncrementalError::CorruptState(format!("invalid hex hash: {hash_str}"))
            })?;
            self.files.insert(
                PathBuf::from(path_str),
                FileState::new(PathBuf::from(path_str), hash),
            );
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Hex encoding / decoding helpers
// ---------------------------------------------------------------------------

/// Encodes a byte slice as a lowercase hex string.
fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(hex_nibble(b >> 4));
        s.push(hex_nibble(b & 0x0f));
    }
    s
}

/// Decodes a 64-char hex string into a 32-byte array.
fn hex_decode(hex: &str) -> Option<[u8; 32]> {
    if hex.len() != 64 {
        return None;
    }
    let mut bytes = [0u8; 32];
    for (i, byte) in bytes.iter_mut().enumerate() {
        let hi = hex.as_bytes()[i * 2];
        let lo = hex.as_bytes()[i * 2 + 1];
        *byte = (hex_value(hi)? << 4) | hex_value(lo)?;
    }
    Some(bytes)
}

/// Encodes a 4-bit value as a hexadecimal character.
fn hex_nibble(byte: u8) -> char {
    match byte {
        0..=9 => char::from(b'0' + byte),
        10..=15 => char::from(b'a' + byte - 10),
        _ => '0',
    }
}

/// Converts a hex ASCII byte to its numeric value.
const fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_CNT: AtomicU64 = AtomicU64::new(0);

    fn test_hash(value: u8) -> [u8; 32] {
        [value; 32]
    }

    fn temp_dir() -> PathBuf {
        let n = TEST_CNT.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("m_incr_test_{}_{}", std::process::id(), n));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn test_register_and_unchanged() {
        let mut eng = IncrementalEngine::new(temp_dir());
        let path = Path::new("src/main.rs");
        eng.register_file(path, test_hash(1));
        assert!(!eng.has_changed(path, test_hash(1)));
    }

    #[test]
    fn test_register_and_changed() {
        let mut eng = IncrementalEngine::new(temp_dir());
        let path = Path::new("src/main.rs");
        eng.register_file(path, test_hash(1));
        assert!(eng.has_changed(path, test_hash(2)));
    }

    #[test]
    fn test_unregistered_always_changed() {
        let eng = IncrementalEngine::new(temp_dir());
        assert!(eng.has_changed(Path::new("unknown.rs"), test_hash(0)));
    }

    #[test]
    fn test_files_needing_recompile() {
        let mut eng = IncrementalEngine::new(temp_dir());
        eng.register_file(Path::new("a.rs"), test_hash(1));
        eng.register_file(Path::new("b.rs"), test_hash(2));

        let changes = vec![
            FileChange::new(PathBuf::from("a.rs"), test_hash(1)),
            FileChange::new(PathBuf::from("b.rs"), test_hash(3)),
            FileChange::new(PathBuf::from("c.rs"), test_hash(4)),
        ];
        let needs = eng.files_needing_recompile(&changes);
        assert_eq!(needs.len(), 2);
        assert!(needs.contains(&PathBuf::from("b.rs")));
        assert!(needs.contains(&PathBuf::from("c.rs")));
    }

    #[test]
    fn test_save_and_load_state() {
        let dir = temp_dir();
        {
            let mut eng = IncrementalEngine::new(dir.clone());
            eng.register_file(Path::new("a.rs"), test_hash(1));
            eng.register_file(Path::new("b.rs"), test_hash(2));
            eng.save_state().unwrap();
        }
        {
            let mut eng = IncrementalEngine::new(dir.clone());
            eng.load_state().unwrap();
            assert!(!eng.has_changed(Path::new("a.rs"), test_hash(1)));
            assert!(eng.has_changed(Path::new("a.rs"), test_hash(99)));
            assert!(!eng.has_changed(Path::new("b.rs"), test_hash(2)));
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_empty_missing_state() {
        let dir = temp_dir();
        let mut eng = IncrementalEngine::new(dir.clone());
        eng.load_state().unwrap();
        assert!(eng.files.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_file_state_creation() {
        let s = FileState::new(PathBuf::from("foo.rs"), test_hash(42));
        assert_eq!(s.path, PathBuf::from("foo.rs"));
        assert_eq!(s.hash, test_hash(42));
    }

    #[test]
    fn test_file_change_creation() {
        let c = FileChange::new(PathBuf::from("bar.rs"), test_hash(7));
        assert_eq!(c.path, PathBuf::from("bar.rs"));
        assert_eq!(c.new_hash, test_hash(7));
    }

    #[test]
    fn test_hex_roundtrip() {
        let original = [
            0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45,
            0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01,
            0x23, 0x45, 0x67, 0x89,
        ];
        let encoded = hex_encode(&original);
        assert_eq!(encoded.len(), 64);
        let decoded = hex_decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_status() {
        let mut eng = IncrementalEngine::new(temp_dir());
        let path = Path::new("main.rs");
        assert_eq!(eng.status(path, test_hash(0)), CompilationStatus::New);

        eng.register_file(path, test_hash(1));
        assert_eq!(eng.status(path, test_hash(1)), CompilationStatus::UpToDate);
        assert_eq!(eng.status(path, test_hash(2)), CompilationStatus::Stale);
    }
}

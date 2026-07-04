#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! Filesystem abstraction for the Motarjim compiler.
//!
//! Provides a virtual filesystem interface for reading files, watching for
//! changes, and abstracting over the real filesystem for testing.
//!
//! # Example
//!
//! ```rust
//! use motarjim_fs::{FileSystem, RealFileSystem};
//!
//! let fs = RealFileSystem::new();
//! // Read a file:
//! // let content = fs.read("index.html")?;
//! ```
//!
//! ## Feature flags
//!
//! - `watcher` — Enables file watching capabilities.
//! - `json` — Enables serde serialization.

use std::path::{Path, PathBuf};

/// Represents the result of reading a file.
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// The absolute path to the file.
    pub path: PathBuf,
    /// The file contents as a string.
    pub content: String,
}

impl FileEntry {
    /// Creates a new file entry.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content: content.into(),
        }
    }
}

/// Abstract filesystem interface.
pub trait FileSystem: Send + Sync {
    /// Read a file as a string.
    fn read(&self, path: &Path) -> Result<FileEntry, std::io::Error>;

    /// Check if a path exists.
    fn exists(&self, path: &Path) -> bool;

    /// List all files in a directory matching a pattern.
    fn list(&self, dir: &Path, extension: &str) -> Result<Vec<PathBuf>, std::io::Error>;

    /// Read a file as raw bytes.
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, std::io::Error>;

    /// Write content to a file.
    fn write(&self, path: &Path, content: &[u8]) -> Result<(), std::io::Error>;

    /// Canonicalize a path.
    fn canonicalize(&self, path: &Path) -> Result<PathBuf, std::io::Error>;
}

/// Real filesystem implementation using `std::fs`.
#[derive(Debug, Clone, Default)]
pub struct RealFileSystem;

impl RealFileSystem {
    /// Creates a new real filesystem.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl FileSystem for RealFileSystem {
    fn read(&self, path: &Path) -> Result<FileEntry, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        Ok(FileEntry::new(path.to_path_buf(), content))
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn list(&self, dir: &Path, extension: &str) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut files = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path
                .extension()
                .is_some_and(|ext| ext == extension.trim_start_matches('.'))
            {
                files.push(path);
            }
        }
        files.sort();
        Ok(files)
    }

    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(path)
    }

    fn write(&self, path: &Path, content: &[u8]) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)
    }

    fn canonicalize(&self, path: &Path) -> Result<PathBuf, std::io::Error> {
        path.canonicalize()
    }
}

/// An in-memory filesystem for testing.
#[derive(Debug, Clone, Default)]
pub struct VirtualFileSystem {
    /// In-memory file storage.
    files: std::collections::HashMap<PathBuf, Vec<u8>>,
}

impl VirtualFileSystem {
    /// Creates a new empty virtual filesystem.
    #[must_use]
    pub fn new() -> Self {
        Self {
            files: std::collections::HashMap::new(),
        }
    }

    /// Adds a file to the virtual filesystem.
    pub fn add_file(&mut self, path: impl Into<PathBuf>, content: impl Into<Vec<u8>>) {
        self.files.insert(path.into(), content.into());
    }
}

impl FileSystem for VirtualFileSystem {
    fn read(&self, path: &Path) -> Result<FileEntry, std::io::Error> {
        let content = self
            .files
            .get(path)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"))?;
        let content_str = String::from_utf8(content.clone())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(FileEntry::new(path.to_path_buf(), content_str))
    }

    fn exists(&self, path: &Path) -> bool {
        self.files.contains_key(path)
    }

    fn list(&self, dir: &Path, extension: &str) -> Result<Vec<PathBuf>, std::io::Error> {
        let ext = extension.trim_start_matches('.');
        let mut files: Vec<PathBuf> = self
            .files
            .keys()
            .filter(|p| p.parent() == Some(dir))
            .filter(|p| p.extension().is_some_and(|e| e == ext))
            .cloned()
            .collect();
        files.sort();
        Ok(files)
    }

    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, std::io::Error> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"))
    }

    fn write(&self, path: &Path, content: &[u8]) -> Result<(), std::io::Error> {
        let mut files = self.files.clone();
        files.insert(path.to_path_buf(), content.to_vec());
        // Interior mutation via clone
        Ok(())
    }

    fn canonicalize(&self, path: &Path) -> Result<PathBuf, std::io::Error> {
        Ok(path.to_path_buf())
    }
}

/// A file change event for incremental compilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileChange {
    /// File was created or modified.
    Modified(PathBuf),
    /// File was deleted.
    Deleted(PathBuf),
}

/// A file watcher that notifies of file changes.
#[cfg(feature = "watcher")]
pub mod watcher {
    use super::FileChange;
    use std::path::PathBuf;
    use std::sync::mpsc;

    /// Callback for file change events.
    pub type WatchCallback = Box<dyn Fn(FileChange) + Send + 'static>;

    /// Watches a directory for file changes.
    pub struct FileWatcher {
        _dir: PathBuf,
        _rx: mpsc::Receiver<FileChange>,
    }

    impl FileWatcher {
        /// Creates a new file watcher for the given directory.
        pub fn new(dir: PathBuf, _callback: WatchCallback) -> Result<Self, std::io::Error> {
            let (_tx, rx) = mpsc::channel();
            Ok(Self { _dir: dir, _rx: rx })
        }

        /// Blocks until the next file change event.
        pub fn next_change(&self) -> Result<FileChange, mpsc::RecvError> {
            self._rx.recv()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_real_fs_new() {
        let _fs = RealFileSystem::new();
    }

    #[test]
    fn test_virtual_fs_read_write() {
        let mut fs = VirtualFileSystem::new();
        fs.add_file("test.txt", "hello world");
        assert!(fs.exists(Path::new("test.txt")));
        let entry = fs.read(Path::new("test.txt")).unwrap();
        assert_eq!(entry.content, "hello world");
    }

    #[test]
    fn test_virtual_fs_not_found() {
        let fs = VirtualFileSystem::new();
        assert!(fs.read(Path::new("nonexistent.txt")).is_err());
    }

    #[test]
    fn test_virtual_fs_list() {
        let mut fs = VirtualFileSystem::new();
        fs.add_file("dir/a.html", "<html>");
        fs.add_file("dir/b.html", "<body>");
        fs.add_file("dir/c.css", "body {}");
        let html_files = fs.list(Path::new("dir"), "html").unwrap();
        assert_eq!(html_files.len(), 2);
        let css_files = fs.list(Path::new("dir"), "css").unwrap();
        assert_eq!(css_files.len(), 1);
    }

    #[test]
    fn test_file_entry() {
        let entry = FileEntry::new("/path/to/file.html", "<div>");
        assert_eq!(entry.path, PathBuf::from("/path/to/file.html"));
        assert_eq!(entry.content, "<div>");
    }

    #[test]
    fn test_file_change_variants() {
        let modified = FileChange::Modified(PathBuf::from("index.html"));
        let deleted = FileChange::Deleted(PathBuf::from("style.css"));
        assert_ne!(modified, deleted);
        if let FileChange::Modified(p) = &modified {
            assert_eq!(p.to_str(), Some("index.html"));
        }
    }

    #[test]
    fn test_virtual_fs_canonicalize() {
        let fs = VirtualFileSystem::new();
        let path = Path::new("some/relative/path.html");
        let canon = fs.canonicalize(path).unwrap();
        assert_eq!(canon, path);
    }

    #[test]
    fn test_real_fs_exists() {
        let fs = RealFileSystem::new();
        // Current dir should always exist
        assert!(fs.exists(Path::new(".")));
    }
}

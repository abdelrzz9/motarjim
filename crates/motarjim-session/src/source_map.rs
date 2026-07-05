use std::collections::HashMap;
use std::path::{Path, PathBuf};

use motarjim_span::SourceFile;

/// A registry of source files keyed by their filesystem path.
///
/// Provides fast lookup from a file path to its [`SourceFile`], which is
/// used for resolving [`SourceSpan`](motarjim_span::SourceSpan) information
/// and producing rich diagnostic output.
#[derive(Debug, Clone)]
pub struct SourceMap {
    files: HashMap<PathBuf, SourceFile>,
}

impl SourceMap {
    /// Creates an empty source map.
    #[must_use]
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    /// Inserts a source file into the map, keyed by its path.
    ///
    /// If a file was already registered for the same path it is replaced
    /// and the old value is returned.
    pub fn add(&mut self, file: SourceFile) -> Option<SourceFile> {
        self.files.insert(file.path.clone(), file)
    }

    /// Returns a reference to the source file at the given path, if any.
    #[must_use]
    pub fn get(&self, path: &Path) -> Option<&SourceFile> {
        self.files.get(path)
    }

    /// Removes the source file at `path`, returning it if it existed.
    pub fn remove(&mut self, path: &Path) -> Option<SourceFile> {
        self.files.remove(path)
    }

    /// Returns `true` if a source file is registered for `path`.
    #[must_use]
    pub fn contains(&self, path: &Path) -> bool {
        self.files.contains_key(path)
    }

    /// Iterates over all registered source files.
    pub fn iter(&self) -> impl Iterator<Item = &SourceFile> {
        self.files.values()
    }

    /// Returns the number of registered source files.
    #[must_use]
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Returns `true` if no source files are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IntoIterator for &'a SourceMap {
    type Item = &'a SourceFile;
    type IntoIter = std::collections::hash_map::Values<'a, PathBuf, SourceFile>;

    fn into_iter(self) -> Self::IntoIter {
        self.files.values()
    }
}

impl Extend<SourceFile> for SourceMap {
    fn extend<T: IntoIterator<Item = SourceFile>>(&mut self, iter: T) {
        for file in iter {
            self.add(file);
        }
    }
}

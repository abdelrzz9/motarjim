#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]

use crate::{SourceLocation, SourceSpan};
use std::path::{Path, PathBuf};

/// Represents a source file with path and content.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceFile {
    /// The path to this source file.
    pub path: PathBuf,
    /// The full source text.
    pub content: String,
    /// Precomputed line start byte offsets.
    #[cfg_attr(feature = "serialize", serde(skip))]
    pub line_starts: Vec<usize>,
}

impl SourceFile {
    /// Creates a new source file from path and content.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>, content: String) -> Self {
        let path = path.into();
        let line_starts = compute_line_starts(&content);
        Self { path, content, line_starts }
    }

    /// Returns the source location for a given byte offset.
    #[must_use]
    pub fn location(&self, offset: usize) -> Option<SourceLocation> {
        if offset > self.content.len() {
            return None;
        }
        let line = match self.line_starts.binary_search(&offset) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        let line_start = self.line_starts.get(line.saturating_sub(1)).copied().unwrap_or(0);
        let column = offset.saturating_sub(line_start) + 1;
        Some(SourceLocation::new(line as u32, column as u32, offset))
    }

    /// Returns the source text for a given span.
    #[must_use]
    pub fn text(&self, span: &SourceSpan) -> Option<&str> {
        if span.end.offset > self.content.len() || span.start.offset > span.end.offset {
            return None;
        }
        Some(&self.content[span.start.offset..span.end.offset])
    }

    /// Returns a formatted source snippet for a span (compatibility alias).
    #[must_use]
    pub fn snippet(&self, span: &SourceSpan, padding: usize) -> String {
        let lines = self.context(span, padding);
        let mut result = String::new();
        for (line_num, text, _is_active) in &lines {
            result.push_str(&format!("{:4} | {text}\n", line_num));
        }
        result
    }

    /// Returns context lines around a location (for diagnostic display).
    #[must_use]
    pub fn context(&self, span: &SourceSpan, padding: usize) -> Vec<(usize, String, bool)> {
        let start_line = span.start.line.saturating_sub(1).saturating_sub(padding as u32);
        let end_line = span.end.line.saturating_sub(1).saturating_add(padding as u32);
        let mut lines = Vec::new();
        for line in start_line..=end_line {
            let idx = line as usize;
            if idx < self.line_starts.len() {
                let line_start = self.line_starts[idx];
                let line_end = self.line_starts.get(idx + 1).copied().unwrap_or(self.content.len());
                let text = &self.content[line_start..line_end];
                let is_active = line >= span.start.line.saturating_sub(1) && line <= span.end.line.saturating_sub(1);
                lines.push((idx + 1, text.trim_end_matches('\n').to_string(), is_active));
            }
        }
        lines
    }
}

fn compute_line_starts(content: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (i, c) in content.char_indices() {
        if c == '\n' {
            starts.push(i + 1);
        }
    }
    starts
}

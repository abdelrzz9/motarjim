#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

//! Golden/snapshot test utilities for the Motarjim compiler.
//!
//! Provides tools for running golden tests that compile HTML/CSS fixtures
//! and compare the output against saved expected output files.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use motarjim_config::{Config, OutputFormat};
use motarjim_core::{CompileOptions, CompileResult, Compiler};
use motarjim_fs::RealFileSystem;

/// Represents a single golden test case combining HTML/CSS input and a target platform.
#[derive(Debug, Clone)]
pub struct GoldenTest {
    /// Test case name (derived from filename stem).
    pub name: String,
    /// Path to the HTML input file.
    pub html_path: PathBuf,
    /// Optional path to the CSS input file.
    pub css_path: Option<PathBuf>,
    /// Target platform output format.
    pub platform: OutputFormat,
}

impl GoldenTest {
    /// Creates a new golden test case.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        html_path: impl Into<PathBuf>,
        css_path: Option<PathBuf>,
        platform: OutputFormat,
    ) -> Self {
        Self {
            name: name.into(),
            html_path: html_path.into(),
            css_path,
            platform,
        }
    }

    /// Load the HTML input content from disk.
    ///
    /// # Errors
    /// Returns an IO error if the file cannot be read.
    pub fn load_html(&self) -> Result<String, std::io::Error> {
        std::fs::read_to_string(&self.html_path)
    }

    /// Load the optional CSS input content from disk.
    ///
    /// # Errors
    /// Returns an IO error if the file cannot be read.
    pub fn load_css(&self) -> Result<Option<String>, std::io::Error> {
        match &self.css_path {
            Some(path) => std::fs::read_to_string(path).map(Some),
            None => Ok(None),
        }
    }

    /// Build the full HTML input, optionally inlining CSS inside a `<style>` tag.
    fn build_input(&self) -> Result<String, std::io::Error> {
        let html = self.load_html()?;
        let css = self.load_css()?;
        Ok(match css {
            Some(ref css_content) if !css_content.trim().is_empty() => {
                format!("<style>\n{css_content}\n</style>\n{html}")
            }
            _ => html,
        })
    }

    /// Compile the input and return the result.
    ///
    /// # Errors
    /// Returns a vector of diagnostics if compilation fails.
    pub fn compile(&self) -> Result<CompileResult, Vec<motarjim_diag::Diagnostic>> {
        let config = Config::new();
        let fs: Arc<RealFileSystem> = Arc::new(RealFileSystem::new());
        let compiler = Compiler::new(config, fs);

        let options = CompileOptions {
            platform: self.platform,
            minify: false,
            source_maps: false,
            strict: false,
        };

        let input = self.build_input().map_err(|e| {
            vec![motarjim_diag::Diagnostic::new(
                motarjim_diag::Severity::Error,
                motarjim_diag::codes::CONFIG_FILE_NOT_FOUND,
                format!("Failed to build input: {e}"),
            )]
        })?;

        compiler.compile(&input, &options)
    }

    /// Get the expected output file path for this test case.
    #[must_use]
    pub fn expected_output_path(&self, golden_dir: &Path) -> PathBuf {
        let platform_dir = match self.platform {
            OutputFormat::Dart => "flutter",
            OutputFormat::Kotlin => "compose",
            OutputFormat::Swift => "swiftui",
        };
        let ext = match self.platform {
            OutputFormat::Dart => "dart",
            OutputFormat::Kotlin => "kt",
            OutputFormat::Swift => "swift",
        };
        golden_dir
            .join("output")
            .join(platform_dir)
            .join(format!("{}.{}", self.name, ext))
    }
}

/// Discovers golden test cases from a fixtures directory and runs them.
pub struct GoldenTestRunner {
    /// Base directory containing golden fixtures.
    golden_dir: PathBuf,
    /// Whether to update golden files instead of comparing.
    update_expect: bool,
}

impl GoldenTestRunner {
    /// Creates a new runner with the given golden directory.
    ///
    /// Checks the `UPDATE_EXPECT` environment variable to determine update mode.
    #[must_use]
    pub fn new(golden_dir: PathBuf) -> Self {
        let update_expect = std::env::var("UPDATE_EXPECT").is_ok();
        Self {
            golden_dir,
            update_expect,
        }
    }

    /// Discover all golden test cases from the fixtures directory.
    ///
    /// Scans `{golden_dir}/html/` for `.html` files and looks for matching
    /// `.css` files in `{golden_dir}/css/`. Creates a test case for each
    /// platform (Flutter, Compose, `SwiftUI`).
    ///
    /// # Errors
    /// Returns an IO error if the fixtures directory cannot be read.
    pub fn discover(&self) -> Result<Vec<GoldenTest>, std::io::Error> {
        let html_dir = self.golden_dir.join("html");
        let css_dir = self.golden_dir.join("css");
        let mut tests: Vec<GoldenTest> = Vec::new();

        if !html_dir.exists() {
            return Ok(tests);
        }

        let read_dir =
            std::fs::read_dir(&html_dir).map_err(|e| std::io::Error::new(e.kind(), e.to_string()))?;

        for entry in read_dir {
            let entry =
                entry.map_err(|e| std::io::Error::new(e.kind(), format!("read_dir entry: {e}")))?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "html") {
                let stem = path
                    .file_stem()
                    .and_then(std::ffi::OsStr::to_str)
                    .map(String::from)
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("invalid filename: {}", path.display()),
                        )
                    })?;

                let css_path = {
                    let css_file = css_dir.join(format!("{stem}.css"));
                    if css_file.exists() {
                        Some(css_file)
                    } else {
                        None
                    }
                };

                for &platform in &[OutputFormat::Dart, OutputFormat::Kotlin, OutputFormat::Swift] {
                    tests.push(GoldenTest::new(
                        stem.clone(),
                        path.clone(),
                        css_path.clone(),
                        platform,
                    ));
                }
            }
        }

        Ok(tests)
    }

    /// Run a single golden test case.
    ///
    /// Compiles the test and either writes the output as the new golden file
    /// (in update mode) or compares against the existing golden file.
    ///
    /// # Errors
    /// Returns a string describing what went wrong.
    pub fn run_test(&self, test: &GoldenTest) -> Result<(), String> {
        let platform_label = match test.platform {
            OutputFormat::Dart => "flutter",
            OutputFormat::Kotlin => "compose",
            OutputFormat::Swift => "swiftui",
        };

        let result = test.compile().map_err(|diags| {
            let msgs: Vec<String> = diags
                .iter()
                .map(|d| format!("[{}] {}: {}", d.severity().as_str(), d.code().message, d.message()))
                .collect();
            format!(
                "[{platform_label}] {}: Compilation failed: {}",
                test.name,
                msgs.join("; ")
            )
        })?;

        let expected_path = test.expected_output_path(&self.golden_dir);

        if self.update_expect {
            if let Some(parent) = expected_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    format!(
                        "[{platform_label}] {}: Failed to create output dir: {e}",
                        test.name
                    )
                })?;
            }
            std::fs::write(&expected_path, &result.output).map_err(|e| {
                format!(
                    "[{platform_label}] {}: Failed to write golden file: {e}",
                    test.name
                )
            })?;
            Ok(())
        } else {
            let expected = std::fs::read_to_string(&expected_path).map_err(|e| {
                format!(
                    "[{platform_label}] {}: Missing golden file at {} (set UPDATE_EXPECT=1 to create): {e}",
                    test.name,
                    expected_path.display()
                )
            })?;

            compare_output(&result.output, &expected).map_err(|diff| {
                format!("[{platform_label}] {}: {diff}", test.name)
            })
        }
    }

    /// Run all discovered tests.
    ///
    /// # Errors
    /// Returns the first error encountered.
    pub fn run_all(&self) -> Result<(), String> {
        let tests = self
            .discover()
            .map_err(|e| format!("Failed to discover tests: {e}"))?;
        if tests.is_empty() {
            return Err("No golden tests discovered".to_string());
        }
        for test in &tests {
            self.run_test(test)?;
        }
        Ok(())
    }

    /// Check if the runner is in update mode (`UPDATE_EXPECT` is set).
    #[must_use]
    pub const fn is_update_mode(&self) -> bool {
        self.update_expect
    }
}

/// Compare actual output with expected, returning `Ok(())` if they match.
///
/// # Errors
/// Returns a diff as a string if they differ.
pub fn compare_output(actual: &str, expected: &str) -> Result<(), String> {
    if actual == expected {
        return Ok(());
    }

    let actual_lines: Vec<&str> = actual.lines().collect();
    let expected_lines: Vec<&str> = expected.lines().collect();

    let mut diff = String::from("Output differs from golden:\n");
    let max_lines = actual_lines.len().max(expected_lines.len());
    let width = max_lines.to_string().len();

    for i in 0..max_lines {
        let a = actual_lines.get(i).copied().unwrap_or("");
        let e = expected_lines.get(i).copied().unwrap_or("");
        if a != e {
            diff.push_str(&format!(
                "  {:>width$}| expected: {:?}\n  {:>width$}| actual:   {:?}\n",
                i + 1,
                e,
                i + 1,
                a,
                width = width
            ));
        }
    }

    Err(diff)
}

/// Load a golden HTML fixture by name from `tests/golden/html/`.
///
/// # Errors
/// Returns an IO error if the file cannot be read.
pub fn load_golden_html(name: &str) -> Result<String, std::io::Error> {
    let path = PathBuf::from("tests/golden/html").join(format!("{name}.html"));
    std::fs::read_to_string(path)
}

/// Load a golden CSS fixture by name from `tests/golden/css/`.
///
/// Returns `None` if no matching CSS file exists.
///
/// # Errors
/// Returns an IO error if the file exists but cannot be read.
pub fn load_golden_css(name: &str) -> Result<Option<String>, std::io::Error> {
    let path = PathBuf::from("tests/golden/css").join(format!("{name}.css"));
    if path.exists() {
        std::fs::read_to_string(path).map(Some)
    } else {
        Ok(None)
    }
}

/// Read expected golden output for a given test name and platform.
///
/// The `platform` parameter should be one of `"flutter"`, `"compose"`, or `"swiftui"`.
///
/// # Errors
/// Returns an IO error if the file cannot be read.
pub fn read_golden_output(name: &str, platform: &str) -> Result<String, std::io::Error> {
    let ext = platform_extension(platform);
    let path = PathBuf::from("tests/golden/output")
        .join(platform)
        .join(format!("{name}.{ext}"));
    std::fs::read_to_string(path)
}

/// Write actual output as a golden file for a given test name and platform.
///
/// The `platform` parameter should be one of `"flutter"`, `"compose"`, or `"swiftui"`.
///
/// # Errors
/// Returns an IO error if the file cannot be written.
pub fn write_golden_output(name: &str, platform: &str, content: &str) -> Result<(), std::io::Error> {
    let ext = platform_extension(platform);
    let path = PathBuf::from("tests/golden/output")
        .join(platform)
        .join(format!("{name}.{ext}"));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)
}

/// Map a platform name to its file extension.
fn platform_extension(platform: &str) -> &'static str {
    match platform {
        "flutter" | "dart" => "dart",
        "compose" | "kotlin" => "kt",
        "swiftui" | "swift" => "swift",
        _ => "txt",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_output_identical() {
        assert!(compare_output("hello\nworld", "hello\nworld").is_ok());
    }

    #[test]
    fn test_compare_output_different() {
        let result = compare_output("hello", "world");
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_output_different_lines() {
        let result = compare_output("line1\nline2", "line1\nchanged");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("2|"), "Expected diff to mention line 2, got: {err}");
        assert!(err.contains("changed"), "Expected diff to contain 'changed', got: {err}");
    }

    #[test]
    fn test_platform_extension() {
        assert_eq!(platform_extension("flutter"), "dart");
        assert_eq!(platform_extension("compose"), "kt");
        assert_eq!(platform_extension("swiftui"), "swift");
        assert_eq!(platform_extension("unknown"), "txt");
    }

    #[test]
    fn test_golden_test_expected_output_path() {
        let test = GoldenTest::new(
            "simple-div",
            PathBuf::from("tests/golden/html/simple-div.html"),
            None,
            OutputFormat::Dart,
        );
        let path = test.expected_output_path(Path::new("tests/golden"));
        assert!(path.to_str().map_or(false, |s| s.contains("output/flutter/simple-div.dart")));
    }

    #[test]
    fn test_golden_test_expected_output_path_compose() {
        let test = GoldenTest::new(
            "simple-div",
            PathBuf::from("tests/golden/html/simple-div.html"),
            None,
            OutputFormat::Kotlin,
        );
        let path = test.expected_output_path(Path::new("tests/golden"));
        assert!(path.to_str().map_or(false, |s| s.contains("output/compose/simple-div.kt")));
    }

    #[test]
    fn test_golden_test_expected_output_path_swiftui() {
        let test = GoldenTest::new(
            "simple-div",
            PathBuf::from("tests/golden/html/simple-div.html"),
            None,
            OutputFormat::Swift,
        );
        let path = test.expected_output_path(Path::new("tests/golden"));
        assert!(path.to_str().map_or(false, |s| s.contains("output/swiftui/simple-div.swift")));
    }

    #[test]
    fn test_golden_test_new() {
        let test = GoldenTest::new(
            "test-name",
            PathBuf::from("path/to.html"),
            None,
            OutputFormat::Dart,
        );
        assert_eq!(test.name, "test-name");
        assert!(test.css_path.is_none());
        assert_eq!(test.platform, OutputFormat::Dart);
    }

    #[test]
    fn test_golden_runner_update_mode() {
        let runner = GoldenTestRunner::new(PathBuf::from("tests/golden"));
        // UPDATE_EXPECT is not set in tests
        assert!(!runner.is_update_mode());
    }
}

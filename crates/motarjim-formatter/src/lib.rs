#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! Code output formatting for the Motarjim compiler.
//!
//! Provides a [`CodeWriter`] for generating indented, formatted code output,
//! along with platform-specific formatting rules for Dart (Flutter),
//! Kotlin (Jetpack Compose), and Swift (`SwiftUI`).
//!
//! # Example
//!
//! ```rust
//! use motarjim_formatter::CodeWriter;
//!
//! let mut w = CodeWriter::new(2);
//! w.write_line("void main() {");
//! w.indent();
//! w.write_line("runApp(MyApp());");
//! w.dedent();
//! w.write_line("}");
//! assert_eq!(w.as_str(), "void main() {\n  runApp(MyApp());\n}\n");
//! ```

/// A code writer that tracks indentation level and produces formatted output.
#[derive(Debug, Clone)]
pub struct CodeWriter {
    /// The output buffer.
    buffer: String,
    /// The current indentation level.
    indent_level: usize,
    /// The number of spaces per indentation level.
    indent_size: usize,
    /// Whether the current line is at the start (no content written yet).
    line_start: bool,
}

impl CodeWriter {
    /// Creates a new `CodeWriter` with the given indent size.
    #[must_use]
    pub const fn new(indent_size: usize) -> Self {
        Self {
            buffer: String::new(),
            indent_level: 0,
            indent_size,
            line_start: true,
        }
    }

    /// Creates a new `CodeWriter` with default indent size (2 spaces).
    #[must_use]
    pub const fn default() -> Self {
        Self::new(2)
    }

    /// Increases the indentation level.
    pub const fn indent(&mut self) {
        self.indent_level += 1;
    }

    /// Decreases the indentation level.
    pub const fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    /// Writes a string without adding a newline.
    pub fn write(&mut self, s: &str) {
        if self.line_start {
            for _ in 0..self.indent_level * self.indent_size {
                self.buffer.push(' ');
            }
            self.line_start = false;
        }
        self.buffer.push_str(s);
    }

    /// Writes a string followed by a newline.
    pub fn write_line(&mut self, s: &str) {
        self.write(s);
        self.buffer.push('\n');
        self.line_start = true;
    }

    /// Writes an indented block: indent, execute closure, dedent.
    pub fn block<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.indent();
        f(self);
        self.dedent();
    }

    /// Writes a tagged block: `name {` then indent, content, dedent, `}`.
    pub fn write_block(&mut self, header: &str, f: impl FnOnce(&mut Self)) {
        self.write_line(&format!("{header} {{"));
        self.block(f);
        self.write_line("}");
    }

    /// Writes a line with a semicolon at the end.
    pub fn write_stmt(&mut self, s: &str) {
        self.write_line(&format!("{s};"));
    }

    /// Writes a blank line.
    pub fn blank_line(&mut self) {
        self.buffer.push('\n');
        self.line_start = true;
    }

    /// Returns a reference to the output string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.buffer
    }

    /// Consumes the writer and returns the output string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.buffer
    }

    /// Returns the current indentation level.
    #[must_use]
    pub const fn indent_level(&self) -> usize {
        self.indent_level
    }

    /// Clears the output buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.indent_level = 0;
        self.line_start = true;
    }
}

/// Platform-specific formatting rules.
pub mod dart {
    use super::CodeWriter;

    /// Creates a `CodeWriter` configured for Dart formatting conventions.
    #[must_use]
    pub const fn writer() -> CodeWriter {
        CodeWriter::new(2)
    }

    /// Writes a Dart class definition.
    pub fn write_class(w: &mut CodeWriter, name: &str, extends: Option<&str>, body: impl FnOnce(&mut CodeWriter)) {
        let header = match extends {
            Some(parent) => format!("class {name} extends {parent}"),
            None => format!("class {name}"),
        };
        w.write_block(&header, body);
    }

    /// Writes a Dart const constructor.
    pub fn write_const_constructor(w: &mut CodeWriter, class: &str, args: &[&str]) {
        let args_str = args.join(", ");
        w.write_line(&format!("  const {class}({{{args_str}}});"));
    }

    /// Writes a Dart `@override` annotation.
    pub fn write_override(w: &mut CodeWriter) {
        w.write_line("  @override");
    }
}

/// Platform-specific formatting rules.
pub mod kotlin {
    use super::CodeWriter;

    /// Creates a `CodeWriter` configured for Kotlin formatting conventions.
    #[must_use]
    pub const fn writer() -> CodeWriter {
        CodeWriter::new(4)
    }

    /// Writes a Kotlin function.
    pub fn write_fun(w: &mut CodeWriter, name: &str, params: &str, return_type: &str, body: impl FnOnce(&mut CodeWriter)) {
        let sig = if return_type.is_empty() {
            format!("fun {name}({params})")
        } else {
            format!("fun {name}({params}): {return_type}")
        };
        w.write_block(&sig, body);
    }

    /// Writes a Kotlin `@Composable` annotation.
    pub fn write_composable(w: &mut CodeWriter) {
        w.write_line("@Composable");
    }

    /// Writes a Kotlin import statement.
    pub fn write_import(w: &mut CodeWriter, pkg: &str) {
        w.write_line(&format!("import {pkg}"));
    }
}

/// Platform-specific formatting rules.
pub mod swift {
    use super::CodeWriter;

    /// Creates a `CodeWriter` configured for Swift formatting conventions.
    #[must_use]
    pub const fn writer() -> CodeWriter {
        CodeWriter::new(4)
    }

    /// Writes a Swift struct definition.
    pub fn write_struct(w: &mut CodeWriter, name: &str, conformances: &[&str], body: impl FnOnce(&mut CodeWriter)) {
        let header = if conformances.is_empty() {
            format!("struct {name}")
        } else {
            format!("struct {name}: {}", conformances.join(", "))
        };
        w.write_block(&header, body);
    }

    /// Writes a Swift `var body: some View` block.
    pub fn write_body(w: &mut CodeWriter, body: impl FnOnce(&mut CodeWriter)) {
        w.write_block("var body: some View", body);
    }

    /// Writes a Swift import statement.
    pub fn write_import(w: &mut CodeWriter, module: &str) {
        w.write_line(&format!("import {module}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_writer_basic() {
        let mut w = CodeWriter::new(2);
        w.write_line("hello");
        w.indent();
        w.write_line("world");
        w.dedent();
        w.write_line("done");
        assert_eq!(w.as_str(), "hello\n  world\ndone\n");
    }

    #[test]
    fn test_code_writer_block() {
        let mut w = CodeWriter::new(2);
        w.write_block("if (true)", |w| {
            w.write_line("return 1;");
        });
        assert_eq!(w.as_str(), "if (true) {\n  return 1;\n}\n");
    }

    #[test]
    fn test_code_writer_stmt() {
        let mut w = CodeWriter::new(2);
        w.write_stmt("let x = 42");
        assert_eq!(w.as_str(), "let x = 42;\n");
    }

    #[test]
    fn test_code_writer_indent_dedent() {
        let mut w = CodeWriter::new(4);
        w.write_line("fn main() {");
        w.indent();
        w.write_line("println!(\"hello\");");
        w.dedent();
        w.write_line("}");
        assert_eq!(w.as_str(), "fn main() {\n    println!(\"hello\");\n}\n");
    }

    #[test]
    fn test_code_writer_multi_level() {
        let mut w = CodeWriter::new(2);
        w.write_line("a {");
        w.indent();
        w.write_line("b {");
        w.indent();
        w.write_line("c");
        w.dedent();
        w.write_line("}");
        w.dedent();
        w.write_line("}");
        assert_eq!(w.as_str(), "a {\n  b {\n    c\n  }\n}\n");
    }

    #[test]
    fn test_code_writer_clear() {
        let mut w = CodeWriter::new(2);
        w.write_line("hello");
        w.clear();
        assert_eq!(w.as_str(), "");
        assert_eq!(w.indent_level(), 0);
    }

    #[test]
    fn test_code_writer_blank_line() {
        let mut w = CodeWriter::new(2);
        w.write_line("a");
        w.blank_line();
        w.write_line("b");
        assert!(w.as_str().contains("\n\n"));
    }

    #[test]
    fn test_dart_writer() {
        let mut w = dart::writer();
        dart::write_class(&mut w, "MyApp", Some("StatelessWidget"), |w| {
            dart::write_const_constructor(w, "MyApp", &["Key? key"]);
            dart::write_override(w);
            w.write_block("Widget build(BuildContext context)", |w| {
                w.write_line("return Container();");
            });
        });
        let output = w.as_str();
        assert!(output.contains("class MyApp extends StatelessWidget"));
        assert!(output.contains("@override"));
    }

    #[test]
    fn test_kotlin_writer() {
        let mut w = kotlin::writer();
        kotlin::write_composable(&mut w);
        kotlin::write_fun(&mut w, "Greeting", "name: String", "Unit", |w| {
            w.write_line("Text(text = name)");
        });
        let output = w.as_str();
        assert!(output.contains("@Composable"));
        assert!(output.contains("fun Greeting(name: String): Unit {"));
    }

    #[test]
    fn test_swift_writer() {
        let mut w = swift::writer();
        swift::write_struct(&mut w, "ContentView", &["View"], |sw| {
            swift::write_body(sw, |sw| {
                sw.write_line("Text(\"Hello\")");
            });
        });
        let output = w.as_str();
        assert!(output.contains("struct ContentView: View"));
        assert!(output.contains("var body: some View"));
    }

    #[test]
    fn test_swift_import() {
        let mut w = swift::writer();
        swift::write_import(&mut w, "SwiftUI");
        assert!(w.as_str().contains("import SwiftUI"));
    }

    #[test]
    fn test_kotlin_import() {
        let mut w = kotlin::writer();
        kotlin::write_import(&mut w, "androidx.compose.foundation.layout.*");
        assert!(w.as_str().contains("import androidx.compose.foundation.layout.*"));
    }

    #[test]
    fn test_code_writer_default() {
        let w = CodeWriter::default();
        assert_eq!(w.indent_size, 2);
    }

    #[test]
    fn test_into_string() {
        let mut w = CodeWriter::new(2);
        w.write_line("hello");
        assert_eq!(w.into_string(), "hello\n");
    }
}

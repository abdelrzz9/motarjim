use crate::diagnostic::{Diagnostic, DiagnosticCode, Severity};

/// Collects diagnostics during compilation.
///
/// Provides methods for adding diagnostics, checking for errors/warnings,
/// and iterating over all collected diagnostics.
#[derive(Debug, Clone)]
pub struct DiagnosticBag {
    /// Collected diagnostics.
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticBag {
    /// Creates a new, empty `DiagnosticBag`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    /// Pushes a diagnostic into the bag.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Convenience method to push an error diagnostic.
    pub fn push_error(&mut self, code: DiagnosticCode, message: impl Into<String>) {
        self.diagnostics.push(Diagnostic::new(
            Severity::Error,
            code,
            message.into(),
        ));
    }

    /// Convenience method to push a warning diagnostic.
    pub fn push_warning(&mut self, code: DiagnosticCode, message: impl Into<String>) {
        self.diagnostics.push(Diagnostic::new(
            Severity::Warning,
            code,
            message.into(),
        ));
    }

    /// Convenience method to push an info diagnostic.
    pub fn push_info(&mut self, code: DiagnosticCode, message: impl Into<String>) {
        self.diagnostics.push(Diagnostic::new(
            Severity::Info,
            code,
            message.into(),
        ));
    }

    /// Returns `true` if any diagnostic in the bag has `Error` severity.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity.is_error())
    }

    /// Returns `true` if any diagnostic in the bag has `Warning` severity.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity.is_warning())
    }

    /// Returns the number of diagnostics in the bag.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Returns `true` if the bag contains no diagnostics.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Returns an iterator over the diagnostics.
    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }

    /// Consumes the bag and returns the underlying `Vec<Diagnostic>`.
    #[must_use]
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    /// Moves all diagnostics from `other` into this bag.
    pub fn extend(&mut self, other: Self) {
        self.diagnostics.extend(other.diagnostics);
    }

    /// Removes all diagnostics from the bag.
    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }
}

impl Default for DiagnosticBag {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for DiagnosticBag {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}

impl Extend<Diagnostic> for DiagnosticBag {
    fn extend<T: IntoIterator<Item = Diagnostic>>(&mut self, iter: T) {
        self.diagnostics.extend(iter);
    }
}

impl<'a> IntoIterator for &'a DiagnosticBag {
    type Item = &'a Diagnostic;
    type IntoIter = std::slice::Iter<'a, Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codes;

    fn test_code() -> DiagnosticCode {
        DiagnosticCode::new(999, "Test code")
    }

    #[test]
    fn test_new_bag_is_empty() {
        let bag = DiagnosticBag::new();
        assert!(bag.is_empty());
        assert_eq!(bag.len(), 0);
        assert!(!bag.has_errors());
        assert!(!bag.has_warnings());
    }

    #[test]
    fn test_push_diagnostic() {
        let mut bag = DiagnosticBag::new();
        let diag = Diagnostic::new(Severity::Error, test_code(), "test error");
        bag.push(diag);
        assert_eq!(bag.len(), 1);
        assert!(bag.has_errors());
    }

    #[test]
    fn test_push_error() {
        let mut bag = DiagnosticBag::new();
        bag.push_error(codes::PARSER_UNEXPECTED_TOKEN, "unexpected token");
        assert_eq!(bag.len(), 1);
        assert!(bag.has_errors());
    }

    #[test]
    fn test_push_warning() {
        let mut bag = DiagnosticBag::new();
        bag.push_warning(codes::A11Y_MISSING_ALT, "missing alt");
        assert_eq!(bag.len(), 1);
        assert!(bag.has_warnings());
        assert!(!bag.has_errors());
    }

    #[test]
    fn test_push_info() {
        let mut bag = DiagnosticBag::new();
        bag.push_info(codes::CONFIG_FILE_NOT_FOUND, "config not found");
        assert_eq!(bag.len(), 1);
        assert!(!bag.has_errors());
        assert!(!bag.has_warnings());
    }

    #[test]
    fn test_has_errors_warnings() {
        let mut bag = DiagnosticBag::new();
        bag.push_warning(codes::A11Y_MISSING_ALT, "warn");
        assert!(!bag.has_errors());
        assert!(bag.has_warnings());
        bag.push_error(codes::PARSER_UNEXPECTED_TOKEN, "err");
        assert!(bag.has_errors());
        assert!(bag.has_warnings());
    }

    #[test]
    fn test_extend() {
        let mut bag1 = DiagnosticBag::new();
        bag1.push_error(test_code(), "error 1");

        let mut bag2 = DiagnosticBag::new();
        bag2.push_error(test_code(), "error 2");

        bag1.extend(bag2);
        assert_eq!(bag1.len(), 2);
    }

    #[test]
    fn test_extend_trait() {
        let mut bag = DiagnosticBag::new();
        let diags = vec![
            Diagnostic::new(Severity::Error, test_code(), "err1"),
            Diagnostic::new(Severity::Warning, test_code(), "warn1"),
        ];
        Extend::<Diagnostic>::extend(&mut bag, diags);
        assert_eq!(bag.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut bag = DiagnosticBag::new();
        bag.push_error(test_code(), "err");
        assert_eq!(bag.len(), 1);
        bag.clear();
        assert!(bag.is_empty());
    }

    #[test]
    fn test_into_diagnostics() {
        let mut bag = DiagnosticBag::new();
        bag.push_error(test_code(), "err");
        let diags = bag.into_diagnostics();
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_iter() {
        let mut bag = DiagnosticBag::new();
        bag.push_error(test_code(), "err");
        bag.push_warning(test_code(), "warn");

        let count = bag.iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_into_iterator() {
        let mut bag = DiagnosticBag::new();
        bag.push_error(test_code(), "err");
        bag.push_warning(test_code(), "warn");

        let count = bag.into_iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_ref_into_iterator() {
        let mut bag = DiagnosticBag::new();
        bag.push_error(test_code(), "err");
        let count = (&bag).into_iter().count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_default() {
        let bag: DiagnosticBag = DiagnosticBag::default();
        assert!(bag.is_empty());
    }
}

#[cfg(test)]
mod proptests {
    use proptest::prelude::*;
    use crate::DiagnosticBag;

    fn test_code() -> crate::DiagnosticCode {
        crate::DiagnosticCode::new(999, "Test code")
    }

    proptest! {
        #[test]
        fn diagnostic_bag_collects_all(msg in "\\PC*") {
            let mut bag = DiagnosticBag::new();
            bag.push_error(test_code(), &msg);
            prop_assert_eq!(bag.len(), 1);
            prop_assert!(bag.has_errors());

            let diags = bag.into_diagnostics();
            prop_assert_eq!(diags.len(), 1);
            prop_assert_eq!(diags[0].message(), &msg);
        }

        #[test]
        fn no_error_on_empty_message(msg in "") {
            let mut bag = DiagnosticBag::new();
            bag.push_error(test_code(), &msg);
            prop_assert_eq!(bag.len(), 1);

            let diags = bag.into_diagnostics();
            prop_assert_eq!(diags[0].message(), "");
        }
    }
}

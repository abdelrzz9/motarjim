use motarjim_span::SourceSpan;

/// A token produced by the lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<T> {
    /// The token kind.
    pub kind: T,
    /// The source span of this token.
    pub span: SourceSpan,
    /// The raw text of this token (borrowed from source).
    pub raw: String,
}

impl<T> Token<T> {
    /// Creates a new token.
    pub fn new(kind: T, span: SourceSpan, raw: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            raw: raw.into(),
        }
    }
}

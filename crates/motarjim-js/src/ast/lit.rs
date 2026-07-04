//! Literal AST nodes: numbers, strings, booleans, regex.

use motarjim_span::SourceSpan;

#[derive(Debug, Clone, PartialEq)]
pub struct NumberLit {
    pub value: f64,
    pub raw: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BigIntLit {
    pub value: String,
    pub raw: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringLit {
    pub value: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoolLit {
    pub value: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegexLit {
    pub pattern: String,
    pub flags: String,
    pub span: SourceSpan,
}

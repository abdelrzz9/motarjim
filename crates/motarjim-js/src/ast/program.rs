//! Program root and source type.

use motarjim_span::SourceSpan;

use crate::ast::stmt::Statement;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub body: Vec<Statement>,
    pub span: SourceSpan,
    pub source_type: SourceType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    Script,
    Module,
}

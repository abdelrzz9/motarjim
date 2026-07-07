//! The `Node` trait for span-bearing AST nodes.

use motarjim_span::SourceSpan;

#[allow(dead_code)]
pub trait Node {
    fn span(&self) -> SourceSpan;
}

//! The `Node` trait for span-bearing AST nodes.

use motarjim_span::SourceSpan;

pub trait Node {
    fn span(&self) -> SourceSpan;
}

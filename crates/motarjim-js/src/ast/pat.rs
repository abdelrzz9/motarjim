//! Pattern AST nodes (destructuring, binding patterns).

use motarjim_span::SourceSpan;

use crate::ast::expr::Expression;

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Ident(String, SourceSpan),
    Object(ObjectPat),
    Array(ArrayPat),
    Assign(Box<AssignPat>),
    Rest(Box<Pattern>),
    Member(Box<MemberPat>),
    Default(Box<DefaultPat>),
}

impl Pattern {
    pub fn span(&self) -> SourceSpan {
        match self {
            Self::Ident(_, span) => *span,
            Self::Object(p) => p.span,
            Self::Array(p) => p.span,
            Self::Assign(p) => p.span,
            Self::Rest(p) => p.span(),
            Self::Member(p) => p.span,
            Self::Default(p) => p.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectPat {
    pub props: Vec<ObjectPatProp>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectPatProp {
    KeyValue {
        key: PropKey,
        value: Box<Pattern>,
        span: SourceSpan,
    },
    Shorthand {
        name: String,
        span: SourceSpan,
    },
    Rest(Box<Pattern>, SourceSpan),
}

use crate::ast::expr::PropKey;

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayPat {
    pub elements: Vec<Option<Pattern>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignPat {
    pub left: Pattern,
    pub right: Expression,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemberPat {
    pub object: Box<Expression>,
    pub property: MemberPatProp,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemberPatProp {
    Ident(String),
    Computed(Box<Expression>),
}

use crate::ast::expr::MemberProp;

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPat {
    pub left: Pattern,
    pub right: Expression,
    pub span: SourceSpan,
}

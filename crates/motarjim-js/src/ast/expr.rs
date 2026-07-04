//! Expression AST nodes for ECMAScript.

use motarjim_span::SourceSpan;

use crate::ast::lit::{BigIntLit, BoolLit, NumberLit, RegexLit, StringLit};
use crate::ast::pat::Pattern;
use crate::ast::stmt::BlockStmt;

#[derive(Debug, Clone, PartialEq)]
pub struct TemplateLiteral {
    pub quasis: Vec<String>,
    pub exprs: Vec<Expression>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayLit {
    pub elements: Vec<ArrayElement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrayElement {
    Some(Expression),
    None(SourceSpan),
    Spread(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropKey {
    Ident(String),
    String(String),
    Computed(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectProp {
    pub key: PropKey,
    pub value: Expression,
    pub shorthand: bool,
    pub computed: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectLit {
    pub props: Vec<ObjectProp>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionExpr {
    pub id: Option<Box<Pattern>>,
    pub params: Vec<Param>,
    pub body: BlockStmt,
    pub generator: bool,
    pub r#async: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrowBody {
    Block(BlockStmt),
    Expr(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrowFunction {
    pub params: Vec<Param>,
    pub body: ArrowBody,
    pub r#async: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub pat: Pattern,
    pub default: Option<Expression>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Minus,
    Plus,
    Not,
    BitwiseNot,
    Typeof,
    Void,
    Delete,
    Increment,
    Decrement,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub argument: Box<Expression>,
    pub prefix: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,
    Eq,
    StrictEq,
    NotEq,
    StrictNotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    In,
    Instanceof,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LShift,
    RShift,
    RtShift,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOp {
    And,
    Or,
    NullishCoalesce,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicalExpr {
    pub op: LogicalOp,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    ExpAssign,
    BitwiseAndAssign,
    BitwiseOrAssign,
    BitwiseXorAssign,
    LShiftAssign,
    RShiftAssign,
    RtShiftAssign,
    LogicalAndAssign,
    LogicalOrAssign,
    NullishAssign,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignExpr {
    pub op: AssignOp,
    pub target: Box<Expression>,
    pub value: Box<Expression>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CondExpr {
    pub test: Box<Expression>,
    pub consequent: Box<Expression>,
    pub alternate: Box<Expression>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    pub callee: Box<Expression>,
    pub args: Vec<Expression>,
    pub optional: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewExpr {
    pub callee: Box<Expression>,
    pub args: Vec<Expression>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemberProp {
    Ident(String),
    Computed(Box<Expression>),
    PrivateIdent(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemberExpr {
    pub object: Box<Expression>,
    pub property: MemberProp,
    pub optional: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SequenceExpr {
    pub exprs: Vec<Expression>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct YieldExpr {
    pub argument: Option<Box<Expression>>,
    pub delegate: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AwaitExpr {
    pub argument: Box<Expression>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetaProperty {
    pub meta: String,
    pub property: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(String, SourceSpan),
    PrivateIdentifier(String, SourceSpan),
    Number(Box<NumberLit>),
    BigInt(Box<BigIntLit>),
    String(Box<StringLit>),
    Bool(Box<BoolLit>),
    Regex(Box<RegexLit>),
    Null(SourceSpan),
    Undefined(SourceSpan),
    This(SourceSpan),
    Super(SourceSpan),
    TemplateLiteral(Box<TemplateLiteral>),
    Array(Box<ArrayLit>),
    Object(Box<ObjectLit>),
    Function(Box<FunctionExpr>),
    Arrow(Box<ArrowFunction>),
    Unary(Box<UnaryExpr>),
    Binary(Box<BinaryExpr>),
    Logical(Box<LogicalExpr>),
    Assignment(Box<AssignExpr>),
    Conditional(Box<CondExpr>),
    Call(Box<CallExpr>),
    New(Box<NewExpr>),
    Member(Box<MemberExpr>),
    Sequence(Box<SequenceExpr>),
    Yield(Box<YieldExpr>),
    Await(Box<AwaitExpr>),
    MetaProperty(Box<MetaProperty>),
    Update(Box<UnaryExpr>),
    Spread(Box<Expression>),
    Parenthesized(Box<Expression>),
}

impl Expression {
    pub fn span(&self) -> SourceSpan {
        match self {
            Self::Identifier(_, span)
            | Self::PrivateIdentifier(_, span)
            | Self::Null(span)
            | Self::Undefined(span)
            | Self::This(span)
            | Self::Super(span) => *span,
            Self::Number(lit) => lit.span,
            Self::BigInt(lit) => lit.span,
            Self::String(lit) => lit.span,
            Self::Bool(lit) => lit.span,
            Self::Regex(lit) => lit.span,
            Self::TemplateLiteral(e) => e.span,
            Self::Array(e) => e.span,
            Self::Object(e) => e.span,
            Self::Function(e) => e.span,
            Self::Arrow(e) => e.span,
            Self::Unary(e) => e.span,
            Self::Binary(e) => e.span,
            Self::Logical(e) => e.span,
            Self::Assignment(e) => e.span,
            Self::Conditional(e) => e.span,
            Self::Call(e) => e.span,
            Self::New(e) => e.span,
            Self::Member(e) => e.span,
            Self::Sequence(e) => e.span,
            Self::Yield(e) => e.span,
            Self::Await(e) => e.span,
            Self::MetaProperty(e) => e.span,
            Self::Update(e) => e.span,
            Self::Spread(e) => e.span(),
            Self::Parenthesized(e) => e.span(),
        }
    }
}

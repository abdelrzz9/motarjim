//! Statement AST nodes for ECMAScript.

use motarjim_span::SourceSpan;

use crate::ast::expr::{Expression, FunctionExpr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarKind {
    Var,
    Let,
    Const,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarDeclarator {
    pub name: Pattern,
    pub init: Option<Expression>,
    pub span: SourceSpan,
}

use crate::ast::pat::Pattern;

#[derive(Debug, Clone, PartialEq)]
pub struct VarDecl {
    pub kind: VarKind,
    pub declarators: Vec<VarDeclarator>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: Pattern,
    pub params: Vec<Param>,
    pub body: BlockStmt,
    pub generator: bool,
    pub r#async: bool,
    pub span: SourceSpan,
}

use crate::ast::expr::Param;

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStmt {
    pub body: Vec<Statement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStmt {
    pub argument: Option<Expression>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub test: Expression,
    pub consequent: Box<Statement>,
    pub alternate: Option<Box<Statement>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    pub test: Option<Expression>,
    pub consequent: Vec<Statement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchStmt {
    pub discriminant: Expression,
    pub cases: Vec<SwitchCase>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForInit {
    VarDecl(VarDecl),
    Expr(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForStmt {
    pub init: Option<ForInit>,
    pub test: Option<Expression>,
    pub update: Option<Expression>,
    pub body: Box<Statement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForOfStmt {
    pub left: Pattern,
    pub right: Expression,
    pub body: Box<Statement>,
    pub r#await: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForInStmt {
    pub left: Pattern,
    pub right: Expression,
    pub body: Box<Statement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    pub test: Expression,
    pub body: Box<Statement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DoWhileStmt {
    pub body: Box<Statement>,
    pub test: Expression,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprStmt {
    pub expr: Expression,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportSpecifier {
    pub imported: String,
    pub local: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportDecl {
    pub default: Option<String>,
    pub namespace: Option<String>,
    pub named: Vec<ImportSpecifier>,
    pub source: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportNamedDecl {
    pub declaration: Option<Box<Statement>>,
    pub specifiers: Vec<ExportSpecifier>,
    pub source: Option<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportSpecifier {
    pub local: String,
    pub exported: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportDefaultDecl {
    pub declaration: ExportDefaultKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportDefaultKind {
    Expression(Expression),
    FunctionDecl(FunctionDecl),
    ClassDecl(ClassDecl),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassBody {
    pub body: Vec<ClassMember>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassMember {
    Method(ClassMethod),
    Property(ClassProperty),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassMethod {
    pub key: PropKey,
    pub kind: MethodKind,
    pub function: FunctionExpr,
    pub computed: bool,
    pub r#static: bool,
    pub span: SourceSpan,
}

use crate::ast::expr::PropKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodKind {
    Method,
    Get,
    Set,
    Constructor,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassProperty {
    pub key: PropKey,
    pub value: Option<Expression>,
    pub computed: bool,
    pub r#static: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDecl {
    pub name: Pattern,
    pub super_class: Option<Expression>,
    pub body: ClassBody,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThrowStmt {
    pub argument: Expression,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryStmt {
    pub block: BlockStmt,
    pub handler: Option<CatchClause>,
    pub finalizer: Option<BlockStmt>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    pub param: Option<Pattern>,
    pub body: BlockStmt,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DebuggerStmt {
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VarDecl(VarDecl),
    FunctionDecl(FunctionDecl),
    ClassDecl(ClassDecl),
    Return(ReturnStmt),
    If(IfStmt),
    Switch(SwitchStmt),
    For(Box<ForStmt>),
    ForOf(ForOfStmt),
    ForIn(ForInStmt),
    While(WhileStmt),
    DoWhile(DoWhileStmt),
    Block(BlockStmt),
    Break(SourceSpan),
    Continue(SourceSpan),
    Throw(ThrowStmt),
    Try(TryStmt),
    Debugger(DebuggerStmt),
    Expr(ExprStmt),
    Import(ImportDecl),
    ExportNamed(ExportNamedDecl),
    ExportDefault(ExportDefaultDecl),
    Empty(SourceSpan),
    Labelled { label: String, body: Box<Statement>, span: SourceSpan },
}

impl Statement {
    pub fn span(&self) -> SourceSpan {
        match self {
            Self::VarDecl(s) => s.span,
            Self::FunctionDecl(s) => s.span,
            Self::ClassDecl(s) => s.span,
            Self::Return(s) => s.span,
            Self::If(s) => s.span,
            Self::Switch(s) => s.span,
            Self::For(s) => s.span,
            Self::ForOf(s) => s.span,
            Self::ForIn(s) => s.span,
            Self::While(s) => s.span,
            Self::DoWhile(s) => s.span,
            Self::Block(s) => s.span,
            Self::Break(span) | Self::Continue(span) | Self::Empty(span) | Self::Debugger(DebuggerStmt { span }) => *span,
            Self::Throw(s) => s.span,
            Self::Try(s) => s.span,
            Self::Expr(s) => s.span,
            Self::Import(s) => s.span,
            Self::ExportNamed(s) => s.span,
            Self::ExportDefault(s) => s.span,
            Self::Labelled { span, .. } => *span,
        }
    }
}
